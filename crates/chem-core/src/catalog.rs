use crate::storage::atomic_write;
use crate::{Catalog, ProductionItem, Recipe};
use serde::Deserialize;
use serde_yaml::Value;

#[derive(Deserialize)]
struct Reagent {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    id: String,
    color: Option<String>,
    #[serde(default)]
    metabolisms: Value,
}
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

pub struct Sources {
    pub recipes: String,
    pub reagents: String,
    pub localization: String,
}

pub fn builtin() -> Catalog {
    serde_json::from_str(include_str!("supplementary.json")).expect("validated built-in catalog")
}

fn normalized(id: &str) -> String {
    id.to_lowercase().replace(['-', '_', ' '], "")
}
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
fn localization(text: &str) -> BTreeMap<String, (String, String)> {
    let mut result = BTreeMap::<String, (String, String)>::new();
    let mut current: Option<(String, bool)> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let matched = ["reagent-name-", "reagent-desc-"]
            .iter()
            .find_map(|prefix| {
                trimmed
                    .strip_prefix(prefix)
                    .and_then(|rest| rest.split_once('='))
                    .map(|(id, value)| {
                        (
                            normalized(id.trim()),
                            *prefix == "reagent-desc-",
                            value.trim(),
                        )
                    })
            });
        if let Some((id, desc, value)) = matched {
            let entry = result.entry(id.clone()).or_default();
            if desc {
                entry.1 = value.into();
            } else {
                entry.0 = capitalized(value);
            }
            current = Some((id, desc));
        } else if line.starts_with(char::is_whitespace) && !trimmed.starts_with('.') {
            if let Some((id, desc)) = &current {
                let entry = result.get_mut(id).unwrap();
                let value = if *desc { &mut entry.1 } else { &mut entry.0 };
                value.push(' ');
                value.push_str(trimmed);
            }
        } else {
            current = None;
        }
    }
    result
}

fn effects(value: &Value, result: &mut Vec<String>) {
    match value {
        Value::Tagged(tagged) => {
            let tag = tagged.tag.to_string();
            if let Some(effect) = tag.strip_prefix("!type:") {
                if !effect.ends_with("Condition") && !result.iter().any(|e| e == effect) {
                    result.push(effect.into());
                }
            }
            effects(&tagged.value, result);
        }
        Value::Sequence(items) => {
            for item in items {
                effects(item, result);
            }
        }
        Value::Mapping(items) => {
            for (_, item) in items {
                effects(item, result);
            }
        }
        _ => {}
    }
}

pub fn parse_sources(sources: &Sources) -> Result<Catalog, String> {
    let reactions: Vec<Value> =
        serde_yaml::from_str(&sources.recipes).map_err(|e| format!("Ошибка рецептов YAML: {e}"))?;
    let reagents: Vec<Reagent> = serde_yaml::from_str(&sources.reagents.replace('\u{feff}', ""))
        .map_err(|e| format!("Ошибка реагентов YAML: {e}"))?;
    let locale = localization(&sources.localization);
    let mut catalog = builtin();
    let mut metadata = BTreeMap::new();
    for reagent in reagents {
        if reagent.kind != "reagent" || reagent.id.is_empty() {
            continue;
        }
        {
            let id = &reagent.id;
            if let Some((name, _)) = locale.get(&normalized(id)) {
                if !name.is_empty() {
                    catalog.names.insert(id.clone(), name.clone());
                }
            }
            metadata.insert(id.to_string(), reagent);
        }
    }
    let mut count = 0;
    for reaction in reactions {
        if reaction.get("type").and_then(Value::as_str) != Some("reaction") {
            continue;
        }
        let Some(products) = reaction.get("products").and_then(Value::as_mapping) else {
            continue;
        };
        if products.len() != 1 {
            continue;
        }
        let (product, amount) = products.iter().next().unwrap();
        let id = product
            .as_str()
            .ok_or("Некорректный ID продукта")?
            .to_string();
        // Preserve the original application's first Razorium alternative.
        if id == "Razorium" && catalog.recipes.contains_key(&id) {
            continue;
        }
        let output = amount
            .as_f64()
            .filter(|a| a.is_finite() && *a > 0.0)
            .ok_or("Некорректный выход реакции")?;
        let reactants = reaction
            .get("reactants")
            .and_then(Value::as_mapping)
            .ok_or("У реакции нет реагентов")?;
        let mut recipe = Recipe {
            id: id.clone(),
            name: catalog
                .names
                .get(&id)
                .cloned()
                .unwrap_or_else(|| id.clone()),
            ..Recipe::default()
        };
        for (reagent, data) in reactants {
            let reagent = reagent
                .as_str()
                .ok_or("Некорректный ID реагента")?
                .to_string();
            let amount = data
                .get("amount")
                .and_then(Value::as_f64)
                .filter(|a| a.is_finite() && *a > 0.0)
                .ok_or("Некорректное количество реагента")?;
            if data.get("catalyst").and_then(Value::as_bool) == Some(true) {
                recipe.catalysts.insert(reagent, amount);
            } else {
                recipe.ingredients.insert(reagent, amount / output);
            }
        }
        recipe.temperature = reaction.get("minTemp").and_then(Value::as_f64);
        if let Some((name, desc)) = locale.get(&normalized(&id)) {
            if !name.is_empty() {
                recipe.name = name.clone();
                catalog.names.insert(id.clone(), name.clone());
            }
            recipe.description = desc.clone();
        }
        catalog.recipes.insert(id, recipe);
        count += 1;
    }
    if count == 0 {
        return Err("Источник не содержит поддерживаемых рецептов".into());
    }
    for recipe in catalog.recipes.values_mut() {
        if let Some(reagent) = metadata.get(&recipe.id) {
            recipe.color = reagent.color.clone();
            effects(&reagent.metabolisms, &mut recipe.effects);
        }
        if let Some((name, desc)) = locale.get(&normalized(&recipe.id)) {
            if !name.is_empty() {
                recipe.name = name.clone();
            }
            recipe.description = desc.clone();
        }
    }
    let mut counts = BTreeMap::new();
    for recipe in catalog.recipes.values() {
        *counts.entry(recipe.name.to_lowercase()).or_insert(0) += 1;
    }
    for recipe in catalog.recipes.values_mut() {
        if counts[&recipe.name.to_lowercase()] > 1 {
            recipe.name = format!("{} ({})", recipe.name, recipe.id);
        }
    }
    catalog.source = "online".into();
    validate(&catalog)?;
    Ok(catalog)
}

fn validate(catalog: &Catalog) -> Result<(), String> {
    if catalog.recipes.is_empty() {
        return Err("Пустой каталог".into());
    }
    for id in catalog.recipes.keys() {
        catalog.calculate(vec![ProductionItem {
            name: id.clone(),
            quantity: 1.0,
        }])?;
    }
    Ok(())
}

pub struct CatalogRepository(PathBuf);
impl CatalogRepository {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
    pub fn snapshot_path(&self) -> PathBuf {
        self.0.join("catalog.json")
    }
    pub fn load(&self) -> Catalog {
        if let Ok(bytes) = std::fs::read(self.snapshot_path()) {
            if let Ok(mut catalog) = serde_json::from_slice::<Catalog>(&bytes) {
                if validate(&catalog).is_ok() {
                    catalog.source = "cache".into();
                    return catalog;
                }
            }
        }
        builtin()
    }
    pub fn replace(&self, sources: &Sources) -> Result<Catalog, String> {
        let catalog = parse_sources(sources)?;
        atomic_write(
            &self.snapshot_path(),
            &serde_json::to_vec(&catalog).map_err(|e| e.to_string())?,
        )?;
        Ok(catalog)
    }
    pub fn refresh(&self) -> Result<Catalog, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Chem-Helper-Rust/1.0")
            .build()
            .map_err(|e| e.to_string())?;
        let fetch = |url: String| -> Result<String, String> {
            let response = client
                .get(url)
                .send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .map_err(|e| format!("Не удалось обновить данные: {e}"))?;
            let text = response.text().map_err(|e| e.to_string())?;
            if text.len() > 8_000_000 {
                return Err("Источник данных слишком большой".into());
            }
            Ok(text)
        };
        let root = "https://raw.githubusercontent.com/space-sunrise/sunrise-station";
        let pinned = "994b1364b6a7eda7cb69bdc3b7373f262b9acd9e";
        let mut reagents = String::new();
        let mut locale = String::new();
        for category in [
            "medicine",
            "narcotics",
            "toxins",
            "elements",
            "chemicals",
            "cleaning",
            "gases",
            "pyrotechnic",
        ] {
            let revision = if category == "medicine" {
                "master"
            } else {
                pinned
            };
            reagents.push_str(&fetch(format!(
                "{root}/{revision}/Resources/Prototypes/Reagents/{category}.yml"
            ))?);
            reagents.push('\n');
            locale.push_str(&fetch(format!(
                "{root}/{pinned}/Resources/Locale/ru-RU/_strings/reagents/meta/{category}.ftl"
            ))?);
            locale.push('\n');
        }
        let recipes = fetch(format!(
            "{root}/master/Resources/Prototypes/Recipes/Reactions/medicine.yml"
        ))?;
        self.replace(&Sources {
            recipes,
            reagents,
            localization: locale,
        })
    }
}
