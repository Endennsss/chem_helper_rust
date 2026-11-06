use chem_core::storage::UserStore;
use serde_json::json;

#[test]
fn acknowledged_favorite_survives_reopening() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("nested/state.json");
    let store = UserStore::new(path.clone());
    let state = json!({"favoriteItems":[{"name":"Аммиак","quantity":30}],"productionItems":[],"animationsEnabled":false});
    store.save(&state).unwrap();
    assert_eq!(UserStore::new(path).load().unwrap(), state);
    let updated = json!({"favoriteItems":[],"productionItems":[]});
    store.save(&updated).unwrap();
    assert_eq!(store.load().unwrap(), updated);
}
#[test]
fn damaged_state_is_preserved_before_recovery() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("state.json");
    std::fs::write(&path, "damaged favorites").unwrap();
    let store = UserStore::new(path.clone());
    assert!(store.load().is_err());
    store.recover().unwrap();
    store.save(&json!({"favoriteItems":[]})).unwrap();
    assert_eq!(
        std::fs::read_to_string(path.with_extension("json.corrupt")).unwrap(),
        "damaged favorites"
    );
}
