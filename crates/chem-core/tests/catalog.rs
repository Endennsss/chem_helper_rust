use chem_core::catalog::{parse_sources, CatalogRepository, Sources};

fn fixture() -> Sources {
    Sources { recipes: "- type: reaction\n  id: Test\n  reactants:\n    Raw: {amount: 2}\n    Plasma: {amount: 1, catalyst: true}\n  products: {Test: 3}\n  minTemp: 370\n  effects:\n  - !type:CreateGas\n    gas: Oxygen\n".into(),
    reagents: "- type: reagent\n  id: Test\n  name: reagent-name-test\n  color: '#123456'\n  metabolisms:\n    Medicine:\n      effects:\n      - !type:HealDamage\n        damage: 1\n".into(),
    localization: "reagent-name-test = препарат\nreagent-desc-test = Описание\n    продолжение\n".into() }
}
#[test]
fn parses_tagged_reactions_and_metadata() {
    let catalog = parse_sources(&fixture()).unwrap();
    let r = &catalog.recipes["Test"];
    assert_eq!(r.name, "Препарат");
    assert_eq!(r.ingredients["Raw"], 2.0 / 3.0);
    assert_eq!(r.catalysts["Plasma"], 1.0);
    assert_eq!(r.temperature, Some(370.0));
    assert_eq!(r.color.as_deref(), Some("#123456"));
    assert!(r.description.contains("продолжение"));
    assert!(r.effects.iter().any(|effect| effect == "HealDamage"));
}
#[test]
fn failed_refresh_preserves_valid_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let repository = CatalogRepository::new(directory.path().into());
    let good = repository.replace(&fixture()).unwrap();
    let old = std::fs::read(repository.snapshot_path()).unwrap();
    let mut bad = fixture();
    bad.recipes = "<html>upstream down</html>".into();
    assert!(repository.replace(&bad).is_err());
    assert_eq!(std::fs::read(repository.snapshot_path()).unwrap(), old);
    assert_eq!(repository.load().recipes.len(), good.recipes.len());
}
#[test]
fn rejects_invalid_recipe_yields() {
    let mut source = fixture();
    source.recipes = source.recipes.replace("Test: 3", "Test: 0");
    assert!(parse_sources(&source).is_err());
}
#[test]
fn clean_install_has_working_offline_recipes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = CatalogRepository::new(directory.path().into()).load();
    assert_eq!(catalog.entries().count(), 386);
    assert!(catalog
        .calculate(vec![chem_core::ProductionItem {
            name: "Аммиак".into(),
            quantity: 30.0
        }])
        .is_ok());
}
#[test]
fn malformed_snapshot_falls_back_without_panicking() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("catalog.json"), "broken").unwrap();
    let catalog = CatalogRepository::new(directory.path().into()).load();
    assert_eq!(catalog.source, "bundled");
    assert!(!catalog.recipes.is_empty());
}
#[test]
fn catalog_with_invalid_proportions_does_not_replace_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let repository = CatalogRepository::new(directory.path().into());
    repository.replace(&fixture()).unwrap();
    let mut source = fixture();
    source.recipes = source
        .recipes
        .replace("Raw: {amount: 2}", "Raw: {amount: -2}");
    assert!(repository.replace(&source).is_err());
}
#[test]
fn ignores_unneeded_huge_yaml_metadata() {
    let mut source = fixture();
    source
        .reagents
        .push_str("  boilingPoint: 340282300000000000000000000000000000000\n");
    assert!(parse_sources(&source).is_ok());
}
#[test]
fn handles_bom_between_concatenated_upstream_files() {
    let mut source = fixture();
    source
        .reagents
        .push_str("\u{feff}- type: reagent\n  id: Extra\n");
    assert!(parse_sources(&source).is_ok());
}
#[test]
fn equal_translations_keep_distinct_selectable_products() {
    let mut source = fixture();
    source.recipes.push_str(
        "- type: reaction\n  id: Other\n  reactants: {Raw: {amount: 1}}\n  products: {Other: 1}\n",
    );
    source.reagents.push_str("- type: reagent\n  id: Other\n");
    source
        .localization
        .push_str("reagent-name-other = препарат\n");
    let catalog = parse_sources(&source).unwrap();
    assert_ne!(
        catalog.recipes["reaction:Test"].name,
        catalog.recipes["reaction:Other"].name
    );
}
#[test]
fn retains_original_first_razorium_recipe_policy() {
    let mut source = fixture();
    source.recipes.push_str("- type: reaction\n  id: First\n  reactants: {Raw: {amount: 1}}\n  products: {Razorium: 1}\n- type: reaction\n  id: Second\n  reactants: {Rare: {amount: 2}}\n  products: {Razorium: 1}\n");
    let catalog = parse_sources(&source).unwrap();
    assert!(catalog.recipes["Razorium"].ingredients.contains_key("Raw"));
    assert!(!catalog.recipes["Razorium"].ingredients.contains_key("Rare"));
}
