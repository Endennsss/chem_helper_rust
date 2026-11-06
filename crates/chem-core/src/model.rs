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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Catalog {
    pub recipes: BTreeMap<String, Recipe>,
    pub names: BTreeMap<String, String>,
    #[serde(default)]
    pub source: String,
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
pub struct Step {
    pub product: String,
    pub quantity: f64,
    pub temperature: Option<f64>,
    pub ingredients: Vec<Row>,
    pub catalysts: Vec<Row>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guide {
    pub name: String,
    pub quantity: f64,
    pub steps: Vec<Step>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Calculation {
    pub items: Vec<ProductionItem>,
    pub direct: Vec<Row>,
    pub base: Vec<Row>,
    pub guides: Vec<Guide>,
}
