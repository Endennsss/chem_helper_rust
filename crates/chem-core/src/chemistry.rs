use crate::{Calculation, Catalog, Guide, ProductionItem, Row, Step};
use std::collections::{BTreeMap, BTreeSet};

const MAX_QUANTITY: f64 = 1_000_000_000.0;

impl Catalog {
    pub fn display_name(&self, id: &str) -> String {
        self.recipes
            .get(id)
            .map(|r| r.name.clone())
            .or_else(|| self.names.get(id).cloned())
            .unwrap_or_else(|| id.into())
    }

    pub fn resolve(&self, name: &str) -> Option<String> {
        let key = name.trim().to_lowercase();
        let key = if key == "геморргинол" {
            "геморринол".into()
        } else {
            key
        };
        if self.recipes.contains_key(name.trim()) {
            return Some(name.trim().into());
        }
        self.recipes
            .iter()
            .find(|(id, r)| id.to_lowercase() == key || r.name.to_lowercase() == key)
            .map(|(id, _)| id.clone())
    }

    pub fn calculate(&self, items: Vec<ProductionItem>) -> Result<Calculation, String> {
        if items.len() > 1000 {
            return Err("Слишком длинная очередь (максимум 1000)".into());
        }
        let mut direct = BTreeMap::new();
        let mut base = BTreeMap::new();
        let mut normalized = Vec::new();
        let mut guides = Vec::new();
        for item in items {
            if !item.quantity.is_finite() || item.quantity <= 0.0 || item.quantity > MAX_QUANTITY {
                return Err(
                    "Количество должно быть конечным числом от 0 до 1 000 000 000, не включая 0"
                        .into(),
                );
            }
            let id = self
                .resolve(&item.name)
                .ok_or_else(|| format!("Материал не найден: {}", item.name))?;
            let selected = &self.recipes[&id];
            let mut amount = item.quantity;
            if selected.quantized {
                if selected.unit == "реакц." && amount.fract().abs() > 1e-9 {
                    return Err("Для этой реакции нужно целое число порций".into());
                }
                let batch = selected.batch_size.max(1e-9);
                amount = ((amount / batch) - 1e-10).ceil() * batch;
            }
            let name = self.display_name(&id);
            for (ingredient, ratio) in &self.recipes[&id].ingredients {
                add(&mut direct, ingredient, amount * ratio)?;
            }
            let mut demands = BTreeMap::from([(id.clone(), amount)]);
            let mut order = Vec::new();
            self.collect(
                &id,
                &mut Vec::new(),
                &mut order,
                &mut BTreeSet::new(),
                &mut 0,
            )?;
            for product in order.iter().rev() {
                let recipe = &self.recipes[product];
                let wanted = demands[product];
                let produced = if recipe.quantized {
                    let batch = recipe.batch_size.max(1e-9);
                    ((wanted / batch) - 1e-10).ceil() * batch
                } else {
                    wanted
                };
                demands.insert(product.clone(), produced);
                for (ingredient, ratio) in &recipe.ingredients {
                    if self.recipes.contains_key(ingredient) {
                        add(&mut demands, ingredient, produced * ratio)?;
                    } else {
                        add(&mut base, ingredient, produced * ratio)?;
                    }
                }
            }
            let steps = order
                .into_iter()
                .map(|product: String| {
                    let recipe = &self.recipes[&product];
                    let amount = demands[&product];
                    let ingredients = recipe
                        .ingredients
                        .iter()
                        .map(|(key, ratio)| (key.clone(), amount * ratio))
                        .collect();
                    Step {
                        product: self.display_name(&product),
                        quantity: precise(amount),
                        temperature: recipe.temperature,
                        products: self.rows(
                            &recipe
                                .products
                                .iter()
                                .map(|(id, ratio)| (id.clone(), amount * ratio))
                                .collect(),
                        ),
                        conditions: recipe.conditions.clone(),
                        effects: recipe.reaction_effects.clone(),
                        max_temperature: recipe.max_temperature,
                        unit: recipe.unit.clone(),
                        ingredients: self.rows(&ingredients),
                        // Catalysts are required amounts, not consumed ingredients.
                        catalysts: recipe
                            .catalysts
                            .iter()
                            .map(|(key, amount)| Row {
                                name: self.display_name(key),
                                quantity: *amount,
                            })
                            .collect(),
                    }
                })
                .collect();
            normalized.push(ProductionItem {
                name: name.clone(),
                quantity: item.quantity,
            });
            guides.push(Guide {
                name,
                quantity: precise(amount),
                steps,
                unit: if selected.unit.is_empty() {
                    "u".into()
                } else {
                    selected.unit.clone()
                },
            });
        }
        guides.reverse();
        Ok(Calculation {
            items: normalized,
            direct: self.rows(&direct),
            base: self.rows(&base),
            guides,
        })
    }

    fn collect(
        &self,
        id: &str,
        trail: &mut Vec<String>,
        order: &mut Vec<String>,
        emitted: &mut BTreeSet<String>,
        visits: &mut usize,
    ) -> Result<(), String> {
        *visits += 1;
        if *visits > 100_000 || trail.len() > 64 {
            return Err("Слишком сложный граф рецептов".into());
        }
        if trail.iter().any(|part| part == id) {
            return Err(format!(
                "Обнаружен цикл рецептов: {} -> {id}",
                trail.join(" -> ")
            ));
        }
        if emitted.contains(id) {
            return Ok(());
        }
        let Some(recipe) = self.recipes.get(id) else {
            return Ok(());
        };
        trail.push(id.into());
        for (ingredient, ratio) in &recipe.ingredients {
            if !ratio.is_finite() || *ratio <= 0.0 {
                return Err(format!("Некорректная пропорция: {id}"));
            }
            self.collect(ingredient, trail, order, emitted, visits)?;
        }
        trail.pop();
        emitted.insert(id.into());
        order.push(id.into());
        Ok(())
    }

    fn rows(&self, amounts: &BTreeMap<String, f64>) -> Vec<Row> {
        let mut rows: Vec<_> = amounts
            .iter()
            .map(|(id, amount)| Row {
                name: self.display_name(id),
                quantity: precise(*amount),
            })
            .collect();
        rows.sort_by_key(|row| row.name.to_lowercase());
        rows
    }
}

fn add(totals: &mut BTreeMap<String, f64>, id: &str, amount: f64) -> Result<(), String> {
    let total = totals.entry(id.into()).or_default();
    *total += amount;
    if !total.is_finite() || *total > 1e15 {
        return Err("Количество ингредиентов слишком велико".into());
    }
    Ok(())
}

fn precise(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
