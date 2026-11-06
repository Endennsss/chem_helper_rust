use crate::{Catalog, CatalogMetadata, Recipe};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct SourceDocument {
    pub path: String,
    pub content: String,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Prototype {
    #[serde(rename = "type")]
    kind: String,
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    group: String,
    #[serde(default)]
    parent: Value,
    color: Option<String>,
    #[serde(default)]
    metabolisms: Value,
    #[serde(default)]
    reactants: BTreeMap<String, Reactant>,
    #[serde(default)]
    products: BTreeMap<String, f64>,
    #[serde(default)]
    required_mixer_categories: Vec<String>,
    min_temp: Option<f64>,
    max_temp: Option<f64>,
    #[serde(default)]
    quantized: bool,
    #[serde(default)]
    effects: Vec<Value>,
}
type RawReagent = Prototype;
type RawReaction = Prototype;
#[derive(Clone, Debug, Deserialize)]
struct Reactant {
    amount: f64,
    #[serde(default)]
    catalyst: bool,
}

pub fn has_chemical_prototypes(content: &str) -> Result<bool, String> {
    #[derive(Deserialize)]
    struct Header {
        #[serde(rename = "type")]
        kind: String,
    }
    let headers: Vec<Header> =
        serde_yaml::from_str(&content.replace('\u{feff}', "")).map_err(|e| e.to_string())?;
    Ok(headers
        .iter()
        .any(|h| h.kind == "reaction" || h.kind == "reagent"))
}

fn normalized(id: &str) -> String {
    id.to_lowercase().replace(['-', '_', ' '], "")
}
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
fn parse_locale(text: &str, result: &mut BTreeMap<String, String>) {
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                result.insert(key.clone(), value.trim().to_string());
                current = Some(key);
            } else {
                current = None;
            }
        } else if trimmed.starts_with('.') {
            current = None;
        } else {
            if let Some(key) = &current {
                let value = result.entry(key.clone()).or_default();
                value.push(' ');
                value.push_str(trimmed);
            }
        }
    }
}
fn resolve_reagent(
    id: &str,
    raw: &BTreeMap<String, RawReagent>,
    trail: &mut BTreeSet<String>,
) -> RawReagent {
    let Some(reagent) = raw.get(id) else {
        return RawReagent::default();
    };
    if !trail.insert(id.into()) {
        return reagent.clone();
    }
    let mut result = RawReagent::default();
    let parents: Vec<&str> = match &reagent.parent {
        Value::String(parent) => vec![parent],
        Value::Sequence(parents) => parents.iter().filter_map(Value::as_str).collect(),
        _ => vec![],
    };
    for parent in parents {
        let inherited = resolve_reagent(parent, raw, trail);
        if !inherited.group.is_empty() {
            result.group = inherited.group;
        }
        if inherited.color.is_some() {
            result.color = inherited.color;
        }
        if !inherited.metabolisms.is_null() {
            result.metabolisms = inherited.metabolisms;
        }
    }
    result.id = reagent.id.clone();
    result.name = reagent.name.clone();
    if !reagent.group.is_empty() {
        result.group = reagent.group.clone();
    }
    if reagent.color.is_some() {
        result.color = reagent.color.clone();
    }
    if !reagent.metabolisms.is_null() {
        result.metabolisms = reagent.metabolisms.clone();
    }
    trail.remove(id);
    result
}
fn collect_tags(value: &Value, result: &mut Vec<String>) {
    match value {
        Value::Tagged(tag) => {
            let name = tag
                .tag
                .to_string()
                .trim_start_matches('!')
                .trim_start_matches("type:")
                .to_string();
            if !name.ends_with("Condition") && !result.contains(&name) {
                result.push(name);
            }
            collect_tags(&tag.value, result);
        }
        Value::Sequence(items) => {
            for item in items {
                collect_tags(item, result);
            }
        }
        Value::Mapping(items) => {
            for item in items.values() {
                collect_tags(item, result);
            }
        }
        _ => (),
    }
}
fn reaction_effects(
    values: &[Value],
    names: &BTreeMap<String, String>,
    locale: &BTreeMap<String, String>,
) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| {
            let Value::Tagged(tag) = value else {
                return None;
            };
            let kind = tag.tag.to_string();
            let label = match kind.trim_start_matches('!').trim_start_matches("type:") {
                "SpawnEntity" => {
                    let id = tag
                        .value
                        .get("entity")
                        .and_then(Value::as_str)
                        .unwrap_or("?");
                    let name = locale
                        .get(&format!("ent-{id}"))
                        .cloned()
                        .unwrap_or_else(|| id.into());
                    format!("Создание предмета: {name}")
                }
                "CreateGas" => {
                    let id = tag.value.get("gas").and_then(Value::as_str).unwrap_or("?");
                    let name = names.get(id).cloned().unwrap_or_else(|| id.into());
                    match tag.value.get("moles").and_then(Value::as_f64) {
                        Some(moles) => format!("Выделение газа: {name}, {moles} моль на порцию"),
                        None => format!("Выделение газа: {name}"),
                    }
                }
                "Explosion" => "Взрыв".into(),
                "Emp" => "Электромагнитный импульс".into(),
                "Flash" => "Световая вспышка".into(),
                "Foam" => "Образование пены".into(),
                "AreaReactionEffect" => {
                    let area = tag
                        .value
                        .get("effectType")
                        .and_then(Value::as_str)
                        .unwrap_or("распространение смеси");
                    format!("Область воздействия: {area}")
                }
                "AdjustSolutionThermalEnergy" => "Изменение температуры раствора".into(),
                "PopupMessage" => return None,
                other => format!("Эффект реакции: {other}"),
            };
            Some(label)
        })
        .collect()
}
fn category(path: &str, reaction: &RawReaction, group: &str) -> String {
    if reaction.id.contains("Breakdown")
        || reaction.id.contains("Electrolysis")
        || reaction.id.ends_with("Decay")
    {
        return "Разложение".into();
    }
    let source = path.rsplit('/').next().unwrap_or_default();
    if source == "gas.yml" {
        return "Газы".into();
    }
    if source == "soap.yml" {
        return "Мыло".into();
    }
    let from_group = match group {
        "Medicine" => Some("Медицина"),
        "Toxins" => Some("Токсины"),
        "Narcotics" => Some("Наркотики"),
        "Drinks" => Some("Напитки"),
        "Foods" => Some("Еда"),
        "Biological" => Some("Биология"),
        "Botanical" => Some("Ботаника"),
        "Pyrotechnic" => Some("Пиротехника"),
        _ => None,
    };
    from_group
        .unwrap_or(match source {
            "medicine.yml" => "Медицина",
            "drinks.yml" => "Напитки",
            "food.yml" => "Еда",
            "biological.yml" => "Биология",
            "botany.yml" => "Ботаника",
            "chemicals.yml" => "Химикаты",
            "cleaning.yml" => "Бытовая химия",
            "pyrotechnic.yml" => "Пиротехника",
            "single_reagent.yml" => "Разложение",
            _ => "Особые реакции",
        })
        .into()
}
fn mixer_label(name: &str) -> &str {
    match name {
        "Stir" => "Мешалка",
        "Shake" => "Шейкер",
        "Centrifuge" => "Центрифуга",
        "Electrolysis" => "Электролизёр",
        other => other,
    }
}
fn known_base(id: &str) -> bool {
    matches!(
        id,
        "Water"
            | "Aluminium"
            | "Carbon"
            | "Chlorine"
            | "Copper"
            | "Ethanol"
            | "Fluorine"
            | "Sugar"
            | "Hydrogen"
            | "Iodine"
            | "Iron"
            | "Lithium"
            | "Mercury"
            | "Nitrogen"
            | "Oxygen"
            | "Phosphorus"
            | "Potassium"
            | "Radium"
            | "Silicon"
            | "Sodium"
            | "Sulfur"
            | "Plasma"
            | "WeldingFuel"
    )
}

pub fn import_documents(documents: &[SourceDocument], revision: &str) -> Result<Catalog, String> {
    let mut locale = BTreeMap::new();
    let mut raw_reagents = BTreeMap::new();
    let mut reactions = Vec::new();
    let mut seen = BTreeSet::new();
    for document in documents {
        if document.path.ends_with(".ftl") {
            parse_locale(&document.content, &mut locale);
            continue;
        }
        if document.content.trim().is_empty() {
            continue;
        }
        let prototypes: Vec<Prototype> =
            serde_yaml::from_str(&document.content.replace('\u{feff}', ""))
                .map_err(|e| format!("{}: {e}", document.path))?;
        for prototype in prototypes {
            if prototype.kind == "reagent" {
                raw_reagents.insert(prototype.id.clone(), prototype);
            } else if prototype.kind == "reaction" {
                if !seen.insert(prototype.id.clone()) {
                    return Err(format!("Дублируется ID реакции: {}", prototype.id));
                }
                reactions.push((document.path.clone(), prototype));
            }
        }
    }
    if reactions.is_empty() {
        return Err("Источник не содержит химических реакций".into());
    }
    let mut catalog = Catalog::default();
    let normalized_locale: BTreeMap<_, _> = locale
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("reagent-name-")
                .map(|id| (normalized(id), value))
        })
        .collect();
    let mut metadata = BTreeMap::new();
    for id in raw_reagents.keys() {
        let reagent = resolve_reagent(id, &raw_reagents, &mut BTreeSet::new());
        let name = locale
            .get(&reagent.name)
            .or_else(|| normalized_locale.get(&normalized(id)).copied())
            .map(|name| capitalized(name))
            .unwrap_or_else(|| id.clone());
        catalog.names.insert(id.clone(), name);
        metadata.insert(id.clone(), reagent);
    }
    // Names for raw inputs still work when an upstream file omits its reagent prototype.
    for (id, name) in &normalized_locale {
        for (_, reaction) in &reactions {
            for reagent in reaction.reactants.keys().chain(reaction.products.keys()) {
                if normalized(reagent) == *id {
                    catalog
                        .names
                        .entry(reagent.clone())
                        .or_insert_with(|| capitalized(name));
                }
            }
        }
    }
    for (path, reaction) in &reactions {
        if reaction.reactants.is_empty() {
            return Err(format!("Нет реагентов: {}", reaction.id));
        }
        let output: f64 = reaction.products.values().sum();
        if reaction
            .products
            .values()
            .any(|v| !v.is_finite() || *v <= 0.0)
            || !output.is_finite()
        {
            return Err(format!("Некорректный выход: {}", reaction.id));
        }
        let divisor = if output > 0.0 { output } else { 1.0 };
        let mut ingredients = BTreeMap::new();
        let mut catalysts = BTreeMap::new();
        for (id, reactant) in &reaction.reactants {
            if !reactant.amount.is_finite() || reactant.amount <= 0.0 {
                return Err(format!("Некорректные пропорции: {}", reaction.id));
            }
            if reactant.catalyst {
                catalysts.insert(id.clone(), reactant.amount);
            } else {
                ingredients.insert(id.clone(), reactant.amount / divisor);
            }
        }
        if ingredients.is_empty() {
            return Err(format!(
                "Реакция без расходуемых реагентов: {}",
                reaction.id
            ));
        }
        if reaction.min_temp.is_some_and(|t| !t.is_finite() || t < 0.0)
            || reaction.max_temp.is_some_and(|t| !t.is_finite() || t < 0.0)
            || matches!((reaction.min_temp,reaction.max_temp),(Some(a),Some(b)) if a>b)
        {
            return Err(format!("Некорректная температура: {}", reaction.id));
        }
        let products: BTreeMap<_, _> = reaction
            .products
            .iter()
            .map(|(id, value)| (id.clone(), value / divisor))
            .collect();
        let effects = reaction_effects(&reaction.effects, &catalog.names, &locale);
        let primary = reaction.products.keys().next();
        let reagent = primary.and_then(|id| metadata.get(id));
        let name = if products.len() == 1 {
            catalog
                .names
                .get(primary.unwrap())
                .cloned()
                .unwrap_or_else(|| primary.unwrap().clone())
        } else if products.len() > 1 {
            format!(
                "{}: {}",
                reaction.id,
                products
                    .keys()
                    .map(|id| catalog.names.get(id).cloned().unwrap_or_else(|| id.clone()))
                    .collect::<Vec<_>>()
                    .join(" + ")
            )
        } else {
            let translated = match reaction.id.as_str() {
                "Smoke" => Some("Дым"),
                "Foam" => Some("Пена"),
                "IronMetalFoam" => Some("Железная металлопена"),
                "AluminiumMetalFoam" => Some("Алюминиевая металлопена"),
                "PotassiumExplosion" => Some("Взрыв калия"),
                _ => None,
            };
            translated
                .map(str::to_string)
                .or_else(|| effects.first().cloned())
                .unwrap_or_else(|| reaction.id.clone())
        };
        let mut conditions: Vec<_> = reaction
            .required_mixer_categories
            .iter()
            .map(|c| format!("Оборудование: {}", mixer_label(c)))
            .collect();
        if reaction.quantized {
            conditions.push("Только целые порции реакции".into());
        }
        let mut metabolism = Vec::new();
        if let Some(reagent) = reagent {
            collect_tags(&reagent.metabolisms, &mut metabolism);
        }
        let description = primary
            .and_then(|id| locale.get(&format!("reagent-desc-{}", normalized(id))))
            .cloned()
            .unwrap_or_default();
        let recipe = Recipe {
            id: format!("reaction:{}", reaction.id),
            name,
            ingredients,
            catalysts,
            temperature: reaction.min_temp,
            max_temperature: reaction.max_temp,
            description,
            color: reagent.and_then(|r| r.color.clone()),
            effects: metabolism,
            category: category(
                path,
                reaction,
                reagent.map(|r| r.group.as_str()).unwrap_or_default(),
            ),
            reaction_id: reaction.id.clone(),
            source_path: path.clone(),
            products,
            conditions,
            reaction_effects: effects,
            quantized: reaction.quantized,
            batch_size: divisor,
            unit: if output > 0.0 {
                "u".into()
            } else {
                "реакц.".into()
            },
            hidden: false,
        };
        catalog.recipes.insert(recipe.id.clone(), recipe);
    }
    let mut counts = BTreeMap::new();
    for recipe in catalog.entries() {
        *counts.entry(recipe.name.clone()).or_insert(0) += 1;
    }
    let mut indices = BTreeMap::new();
    for recipe in catalog.recipes.values_mut() {
        if counts[&recipe.name] > 1 {
            let index = indices.entry(recipe.name.clone()).or_insert(0);
            *index += 1;
            recipe.name = format!("{} · вариант {}", recipe.name, index);
        }
    }
    // Prefer an acyclic production graph. Base dispenser substances stay supplied
    // rather than being recursively decomposed/recreated by reversible recipes.
    let visible: Vec<Recipe> = catalog.entries().cloned().collect();
    let producible: BTreeSet<String> = visible
        .iter()
        .filter(|r| r.products.len() == 1)
        .flat_map(|r| r.products.keys().cloned())
        .collect();
    loop {
        let mut changed = false;
        for recipe in &visible {
            // Separation/co-products are deliberately selected explicitly. Do
            // not fabricate an optimal joint-production plan by expanding each
            // co-product independently and double-counting the same batch.
            if recipe.products.len() != 1 {
                continue;
            }
            if !recipe.ingredients.keys().all(|id| {
                known_base(id)
                    || !producible.contains(id)
                    || catalog.recipes.get(id).is_some_and(|r| r.hidden)
            }) {
                continue;
            }
            for (product, ratio) in &recipe.products {
                if known_base(product) || catalog.recipes.contains_key(product) {
                    continue;
                }
                let mut alias = recipe.clone();
                alias.id = product.clone();
                alias.name = catalog
                    .names
                    .get(product)
                    .cloned()
                    .unwrap_or_else(|| product.clone());
                alias.hidden = true;
                for amount in alias.ingredients.values_mut() {
                    *amount /= ratio;
                }
                for amount in alias.products.values_mut() {
                    *amount /= ratio;
                }
                alias.batch_size *= ratio;
                catalog.recipes.insert(product.clone(), alias);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let aliases: BTreeSet<String> = catalog
        .recipes
        .iter()
        .filter(|(_, r)| r.hidden)
        .map(|(id, _)| id.clone())
        .collect();
    let all_products: BTreeSet<String> = visible
        .iter()
        .flat_map(|r| r.products.keys().cloned())
        .collect();
    for recipe in catalog.recipes.values_mut() {
        let provided: Vec<_> = recipe
            .ingredients
            .keys()
            .filter(|id| all_products.contains(*id) && !aliases.contains(*id) && !known_base(id))
            .map(|id| catalog.names.get(id).cloned().unwrap_or_else(|| id.clone()))
            .collect();
        if !provided.is_empty() {
            recipe.conditions.push(format!(
                "Возьмите готовые компоненты (разделение/взаимозависимые процессы): {}",
                provided.join(", ")
            ));
        }
    }
    catalog.metadata = CatalogMetadata {
        schema: 2,
        revision: revision.into(),
        source_paths: documents.iter().map(|d| d.path.clone()).collect(),
        reaction_ids: seen.into_iter().collect(),
    };
    catalog.source = "online".into();
    validate_catalog(&catalog)?;
    Ok(catalog)
}

pub fn validate_catalog(catalog: &Catalog) -> Result<(), String> {
    if catalog.entries().count() == 0 {
        return Err("Пустой каталог".into());
    }
    if catalog.metadata.schema >= 2 {
        let ids: BTreeSet<_> = catalog.entries().map(|r| r.reaction_id.clone()).collect();
        if ids.len() != catalog.entries().count()
            || ids != catalog.metadata.reaction_ids.iter().cloned().collect()
        {
            return Err("Неполный каталог реакций".into());
        }
    }
    for recipe in catalog.entries() {
        catalog.calculate(vec![crate::ProductionItem {
            name: recipe.id.clone(),
            quantity: recipe.batch_size.max(1.0),
        }])?;
    }
    Ok(())
}
