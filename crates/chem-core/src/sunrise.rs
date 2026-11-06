use crate::importer::{has_chemical_prototypes, import_documents, SourceDocument};
use crate::Catalog;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

const API: &str = "https://api.github.com/repos/space-sunrise/sunrise-station";
const RAW: &str = "https://raw.githubusercontent.com/space-sunrise/sunrise-station";

#[derive(Debug, Default, Deserialize)]
pub struct GitTree {
    #[serde(default)]
    pub truncated: bool,
    pub tree: Vec<GitEntry>,
}
#[derive(Debug, Deserialize)]
pub struct GitEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub sha: String,
}
fn text(client: &reqwest::blocking::Client, url: &str) -> Result<String, String> {
    let mut response = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| format!("{url}: {e}"))?;
    let mut bytes = Vec::new();
    response
        .by_ref()
        .take(8_000_001)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 8_000_000 {
        return Err("Источник данных слишком большой".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn json(client: &reqwest::blocking::Client, url: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str(&text(client, url)?).map_err(|e| format!("Ошибка ответа GitHub: {e}"))
}
fn tree(client: &reqwest::blocking::Client, sha: &str, recursive: bool) -> Result<GitTree, String> {
    let suffix = if recursive { "?recursive=1" } else { "" };
    serde_json::from_str(&text(client, &format!("{API}/git/trees/{sha}{suffix}"))?)
        .map_err(|e| e.to_string())
}
fn child(tree: &GitTree, name: &str) -> Result<String, String> {
    if tree.truncated {
        return Err("GitHub вернул неполный список файлов".into());
    }
    tree.tree
        .iter()
        .find(|e| e.kind == "tree" && e.path == name)
        .map(|e| e.sha.clone())
        .ok_or_else(|| format!("Не найдена папка {name}"))
}
pub fn discover_paths(
    prototypes: &GitTree,
    locales: &GitTree,
    _known: &[String],
) -> Result<Vec<String>, String> {
    if prototypes.truncated || locales.truncated {
        return Err("GitHub вернул обрезанный список: обновление отменено".into());
    }
    let mut paths = BTreeSet::new();
    for entry in prototypes.tree.iter().filter(|e| e.kind == "blob") {
        let full = format!("Resources/Prototypes/{}", entry.path);
        let lower = full.to_lowercase();
        if lower.ends_with(".yml") || lower.ends_with(".yaml") {
            paths.insert(full);
        }
    }
    for entry in locales.tree.iter().filter(|e| e.kind == "blob") {
        let full = format!("Resources/Locale/ru-RU/{}", entry.path);
        let lower = full.to_lowercase();
        if lower.ends_with(".ftl") {
            paths.insert(full);
        }
    }
    Ok(paths.into_iter().collect())
}

pub fn refresh(current: &Catalog) -> Result<Catalog, String> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(8)
        .user_agent("Chem-Helper-Rust/1.1")
        .build()
        .map_err(|e| e.to_string())?;
    let branch = json(&client, &format!("{API}/branches/master"))?;
    let revision = branch
        .pointer("/commit/sha")
        .and_then(serde_json::Value::as_str)
        .ok_or("В ответе GitHub нет commit SHA")?;
    if current.metadata.schema == 2 && current.metadata.revision == revision {
        let mut verified = current.clone();
        verified.source = "online".into();
        return Ok(verified);
    }
    let root_sha = branch
        .pointer("/commit/commit/tree/sha")
        .and_then(serde_json::Value::as_str)
        .ok_or("В ответе GitHub нет tree SHA")?;
    let root = tree(&client, root_sha, false)?;
    let resources = tree(&client, &child(&root, "Resources")?, false)?;
    let prototypes = tree(&client, &child(&resources, "Prototypes")?, true)?;
    let locale = tree(&client, &child(&resources, "Locale")?, false)?;
    let russian = tree(&client, &child(&locale, "ru-RU")?, true)?;
    let paths = discover_paths(&prototypes, &russian, &current.metadata.source_paths)?;
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    let results = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| loop {
                if failed.load(Ordering::Relaxed) {
                    break;
                }
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(path) = paths.get(index) else {
                    break;
                };
                let result = text(&client, &format!("{RAW}/{revision}/{path}")).map(|content| {
                    SourceDocument {
                        path: path.clone(),
                        content,
                    }
                });
                if result.is_err() {
                    failed.store(true, Ordering::Relaxed);
                }
                results.lock().unwrap().push(result);
            });
        }
    });
    let mut documents = Vec::new();
    for result in results.into_inner().map_err(|e| e.to_string())? {
        documents.push(result?);
    }
    if documents.len() != paths.len() {
        return Err("Загружены не все файлы: старый каталог сохранён".into());
    }
    // Paths are not a reliable prototype type: forks can put chemistry anywhere.
    // Inspect every YAML before filtering, and abort on any failed file/parse.
    let mut chemical_documents = Vec::new();
    for document in documents {
        if document.path.ends_with(".ftl")
            || has_chemical_prototypes(&document.content)
                .map_err(|error| format!("{}: {error}", document.path))?
        {
            chemical_documents.push(document);
        }
    }
    chemical_documents.sort_by(|a, b| a.path.cmp(&b.path));
    import_documents(&chemical_documents, revision)
}
