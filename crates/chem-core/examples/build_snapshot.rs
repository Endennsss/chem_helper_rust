use chem_core::catalog::{import_documents, SourceDocument};
use chem_core::importer::has_chemical_prototypes;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn walk(path: &Path, extension: &str, result: &mut Vec<PathBuf>) -> Result<(), String> {
    for item in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let kind = item.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            walk(&item.path(), extension, result)?;
        } else if kind.is_file() && item.path().extension().is_some_and(|x| x == extension) {
            result.push(item.path());
        }
    }
    Ok(())
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let root = Path::new(
        args.get(1)
            .ok_or("usage: build_snapshot SUNRISE_CHECKOUT [OUTPUT_DIRECTORY]")?,
    );
    let output = Path::new(
        args.get(2)
            .map(String::as_str)
            .unwrap_or("crates/chem-core/data"),
    );
    let revision = String::from_utf8(
        std::process::Command::new("git")
            .args(["-C", root.to_str().ok_or("path")?, "rev-parse", "HEAD"])
            .output()
            .map_err(|e| e.to_string())?
            .stdout,
    )
    .map_err(|e| e.to_string())?
    .trim()
    .to_string();
    let mut paths = Vec::new();
    walk(&root.join("Resources/Prototypes"), "yml", &mut paths)?;
    paths.sort();
    let scanned = paths.len();
    let mut documents = Vec::new();
    let mut entities = BTreeSet::new();
    for path in paths {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if content.trim().is_empty() {
            continue;
        }
        if has_chemical_prototypes(&content).map_err(|e| format!("{}: {e}", path.display()))? {
            for line in content.lines() {
                if let Some(entity) = line.trim().strip_prefix("entity:") {
                    entities.insert(format!("ent-{}", entity.trim()));
                }
            }
            documents.push(SourceDocument {
                path: path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                content,
            });
        }
    }
    let mut locales = Vec::new();
    walk(&root.join("Resources/Locale/ru-RU"), "ftl", &mut locales)?;
    locales.sort();
    for path in locales {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let needed = content.contains("reagent-name-")
            || content.contains("reagent-desc-")
            || content.lines().any(|line| {
                line.split_once('=')
                    .is_some_and(|(key, _)| entities.contains(key.trim()))
            });
        if needed {
            documents.push(SourceDocument {
                path: path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
                content,
            });
        }
    }
    let mut catalog = import_documents(&documents, &revision)?;
    let mut counts = BTreeMap::<String, usize>::new();
    for recipe in catalog.entries() {
        *counts.entry(recipe.source_path.clone()).or_default() += 1;
    }
    let mut categories = BTreeMap::<String, usize>::new();
    for recipe in catalog.entries() {
        *categories.entry(recipe.category.clone()).or_default() += 1;
    }
    let coverage = serde_json::json!({"revision":revision,"prototypeFilesScanned":scanned,"reactionCount":catalog.entries().count(),"reactionIds":catalog.metadata.reaction_ids,"reactionFiles":counts,"categories":categories});
    // Bundle normalized identifiers, factual coefficients and concise labels;
    // do not redistribute upstream YAML, descriptions or metabolism source.
    for recipe in catalog.recipes.values_mut() {
        recipe.description.clear();
        recipe.effects.clear();
    }
    catalog.source = "bundled".into();
    std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
    std::fs::write(
        output.join("sunrise-catalog.json"),
        serde_json::to_vec(&catalog).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        output.join("coverage.json"),
        serde_json::to_vec_pretty(&coverage).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    println!(
        "Scanned {scanned} prototype files; {} reactions; {} source files",
        catalog.entries().count(),
        documents.len()
    );
    println!("Categories: {categories:?}");
    Ok(())
}
