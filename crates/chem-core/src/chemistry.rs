use crate::{Calculation, Catalog, Guide, ProductionItem, Row, Step};
use std::collections::{BTreeMap, BTreeSet};

const MAX_QUANTITY: f64 = 1_000_000_000.0;

impl Catalog {
    pub fn display_name(&self, id: &str) -> String {
        self.recipes.get(id).map(|r| r.name.clone())
            .or_else(|| self.names.get(id).cloned()).unwrap_or_else(|| id.into())
    }

    pub fn resolve(&self, name: &str) -> Option<String> {
        let key = name.trim().to_lowercase();
        let key = if key == "геморргинол" { "геморринол".into() } else { key };
        self.recipes.iter().find(|(id, r)| id.to_lowercase() == key || r.name.to_lowercase() == key).map(|(id, _)| id.clone())
    }

    pub fn calculate(&self, items: Vec<ProductionItem>) -> Result<Calculation, String> {
        if items.len() > 1000 { return Err("Слишком длинная очередь (максимум 1000)".into()); }
        let mut direct = BTreeMap::new();
        let mut base = BTreeMap::new();
        let mut normalized = Vec::new();
        let mut guides = Vec::new();
        for item in items {
            if !item.quantity.is_finite() || item.quantity <= 0.0 || item.quantity > MAX_QUANTITY {
                return Err("Количество должно быть конечным числом от 0 до 1 000 000 000, не включая 0".into());
            }
            let id = self.resolve(&item.name).ok_or_else(|| format!("Материал не найден: {}", item.name))?;
            let name = self.display_name(&id);
            for (ingredient, ratio) in &self.recipes[&id].ingredients {
                add(&mut direct, ingredient, item.quantity * ratio)?;
            }
            let mut demands = BTreeMap::new();
            let mut order = Vec::new();
            let mut emitted = BTreeSet::new();
            let mut visits = 0;
            self.expand(&id, item.quantity, &mut Vec::new(), &mut demands, &mut base, &mut order, &mut emitted, &mut visits)?;
            let steps = order.into_iter().map(|product: String| {
                let recipe = &self.recipes[&product];
                let amount = demands[&product];
                let ingredients = recipe.ingredients.iter().map(|(key, ratio)| (key.clone(), amount * ratio)).collect();
                Step {
                    product: self.display_name(&product), quantity: amount.round(),
                    temperature: recipe.temperature, ingredients: self.rows(&ingredients),
                    // Catalysts are required amounts, not consumed ingredients.
                    catalysts: recipe.catalysts.iter().map(|(key, amount)| Row { name: self.display_name(key), quantity: *amount }).collect(),
                }
            }).collect();
            normalized.push(ProductionItem { name: name.clone(), quantity: item.quantity });
            guides.push(Guide { name, quantity: item.quantity.round(), steps });
        }
        guides.reverse();
        Ok(Calculation { items: normalized, direct: self.rows(&direct), base: self.rows(&base), guides })
    }

    #[allow(clippy::too_many_arguments)]
    fn expand(&self, id: &str, amount: f64, trail: &mut Vec<String>, demands: &mut BTreeMap<String, f64>, base: &mut BTreeMap<String, f64>, order: &mut Vec<String>, emitted: &mut BTreeSet<String>, visits: &mut usize) -> Result<(), String> {
        *visits += 1;
        if *visits > 100_000 || trail.len() > 64 { return Err("Слишком сложный граф рецептов".into()); }
        if trail.iter().any(|part| part == id) {
            return Err(format!("Обнаружен цикл рецептов: {} -> {id}", trail.join(" -> ")));
        }
        let Some(recipe) = self.recipes.get(id) else { return add(base, id, amount); };
        add(demands, id, amount)?;
        trail.push(id.into());
        for (ingredient, ratio) in &recipe.ingredients {
            if !ratio.is_finite() || *ratio <= 0.0 { return Err(format!("Некорректная пропорция: {id}")); }
            self.expand(ingredient, amount * ratio, trail, demands, base, order, emitted, visits)?;
        }
        trail.pop();
        if emitted.insert(id.into()) { order.push(id.into()); }
        Ok(())
    }

    fn rows(&self, amounts: &BTreeMap<String, f64>) -> Vec<Row> {
        let mut rows: Vec<_> = amounts.iter().map(|(id, amount)| Row { name: self.display_name(id), quantity: amount.round() }).collect();
        rows.sort_by_key(|row| row.name.to_lowercase());
        rows
    }
}

fn add(totals: &mut BTreeMap<String, f64>, id: &str, amount: f64) -> Result<(), String> {
    let total = totals.entry(id.into()).or_default();
    *total += amount;
    if !total.is_finite() || *total > 1e15 { return Err("Количество ингредиентов слишком велико".into()); }
    Ok(())
}
