#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chem_core::{catalog::CatalogRepository, storage::UserStore, Catalog, ProductionItem};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex, RwLock};
use tauri::{Manager, State};

struct NativeState {
    catalog: RwLock<Catalog>,
    repository: Arc<CatalogRepository>,
    store: Mutex<UserStore>,
}

fn material_payload(catalog: &Catalog) -> Value {
    let mut recipes: Vec<_> = catalog
        .recipes
        .values()
        .map(|r| {
            json!({
                "name":r.name, "nameEn":r.id, "ingredientCount":r.ingredients.len(),
                "temperature":r.temperature, "description":r.description,
                "color":r.color, "effects":r.effects,
            })
        })
        .collect();
    recipes.sort_by_key(|r| r["name"].as_str().unwrap_or_default().to_lowercase());
    json!({ "count": recipes.len(), "materials":recipes, "source":catalog.source })
}

#[tauri::command]
fn load_materials(state: State<NativeState>) -> Result<Value, String> {
    let catalog = state.catalog.read().map_err(|e| e.to_string())?;
    Ok(material_payload(&catalog))
}
#[tauri::command]
fn calculate_production(
    items: Vec<ProductionItem>,
    state: State<NativeState>,
) -> Result<chem_core::Calculation, String> {
    state
        .catalog
        .read()
        .map_err(|e| e.to_string())?
        .calculate(items)
}
#[tauri::command]
async fn refresh_materials(state: State<'_, NativeState>) -> Result<Value, String> {
    let repository = Arc::clone(&state.repository);
    let catalog = tauri::async_runtime::spawn_blocking(move || repository.refresh())
        .await
        .map_err(|e| e.to_string())??;
    let payload = material_payload(&catalog);
    *state.catalog.write().map_err(|e| e.to_string())? = catalog;
    Ok(payload)
}
#[tauri::command]
fn load_user_state(state: State<NativeState>) -> Result<Value, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    match store.load() {
        Ok(value) => Ok(json!({"state":value})),
        Err(error) => {
            store.recover()?;
            Ok(
                json!({"state":{},"warning":format!("{error}. Исходный файл сохранён в резервной копии .corrupt")}),
            )
        }
    }
}
#[tauri::command]
fn save_user_state(state: Value, native: State<NativeState>) -> Result<(), String> {
    native.store.lock().map_err(|e| e.to_string())?.save(&state)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            let repository = Arc::new(CatalogRepository::new(directory.join("cache")));
            let catalog = repository.load();
            app.manage(NativeState {
                catalog: RwLock::new(catalog),
                repository,
                store: Mutex::new(UserStore::new(directory.join("state.json"))),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_materials,
            calculate_production,
            refresh_materials,
            load_user_state,
            save_user_state
        ])
        .run(tauri::generate_context!())
        .expect("Не удалось запустить Chem Helper Rust");
}
