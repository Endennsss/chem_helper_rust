use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: BTreeMap<String, f64>,
    #[serde(default)]
    pub catalysts: BTreeMap<String, f64>,
    pub temperature: Option<f64>,
    #[serde(default)]
    pub description: String,
    pub color: Option<String>,
    #[serde(default)]
    pub effects: Vec<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub reaction_id: String,
    #[serde(default)]
    pub source_path: String,
    #[serde(default)]
    pub products: BTreeMap<String, f64>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub reaction_effects: Vec<String>,
    #[serde(default)]
    pub max_temperature: Option<f64>,
    #[serde(default)]
    pub quantized: bool,
    #[serde(default = "one")]
    pub batch_size: f64,
    #[serde(default = "volume_unit")]
    pub unit: String,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Catalog {
    pub recipes: BTreeMap<String, Recipe>,
    pub names: BTreeMap<String, String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub metadata: CatalogMetadata,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogMetadata {
    pub schema: u32,
    pub revision: String,
    pub source_paths: Vec<String>,
    pub reaction_ids: Vec<String>,
}
impl Catalog {
    pub fn entries(&self) -> impl Iterator<Item = &Recipe> {
        self.recipes.values().filter(|recipe| !recipe.hidden)
    }
}
fn one() -> f64 {
    1.0
}
fn volume_unit() -> String {
    "u".into()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProductionItem {
    pub name: String,
    pub quantity: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Row {
    pub name: String,
    pub quantity: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub product: String,
    pub quantity: f64,
    pub temperature: Option<f64>,
    pub ingredients: Vec<Row>,
    pub catalysts: Vec<Row>,
    pub products: Vec<Row>,
    pub conditions: Vec<String>,
    pub effects: Vec<String>,
    pub max_temperature: Option<f64>,
    pub unit: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guide {
    pub name: String,
    pub quantity: f64,
    pub steps: Vec<Step>,
    pub unit: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Calculation {
    pub items: Vec<ProductionItem>,
    pub direct: Vec<Row>,
    pub base: Vec<Row>,
    pub guides: Vec<Guide>,
}
