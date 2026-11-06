use chem_core::catalog::{import_documents, SourceDocument};
use chem_core::ProductionItem;

fn fixture() -> Vec<SourceDocument> {
    vec![
        SourceDocument { path:"Resources/Prototypes/Reagents/drink.yml".into(), content:"- type: reagent\n  id: BaseDrink\n  abstract: true\n  group: Drinks\n- type: reagent\n  id: Cocktail\n  parent: BaseDrink\n- type: reagent\n  id: Salt\n  group: Chemicals\n".into() },
        SourceDocument { path:"Resources/Locale/ru-RU/reagents.ftl".into(), content:"reagent-name-cocktail = коктейль\nreagent-name-salt = соль\nreagent-name-water = вода\n".into() },
        SourceDocument { path:"Resources/Prototypes/_Sunrise/Chemistry/Recipes/test.yml".into(), content:"- type: reaction\n  id: CocktailOne\n  reactants: {Water: {amount: 1}, Sugar: {amount: 1}}\n  products: {Cocktail: 2}\n- type: reaction\n  id: CocktailTwo\n  reactants: {Juice: {amount: 1}}\n  products: {Cocktail: 1}\n- type: reaction\n  id: Separation\n  requiredMixerCategories: [Centrifuge]\n  minTemp: 300\n  maxTemp: 400\n  reactants: {Blood: {amount: 4}}\n  products: {Water: 3, Salt: 1}\n- type: reaction\n  id: Foam\n  quantized: true\n  reactants: {Water: {amount: 2}, Catalyst: {amount: 1, catalyst: true}}\n  effects:\n  - !type:Foam\n    spreadAmount: 10\n".into() },
    ]
}
#[test]
fn retains_every_reaction_and_alternative() {
    let c = import_documents(&fixture(), "test-commit").unwrap();
    assert_eq!(c.entries().count(), 4);
    for id in ["CocktailOne", "CocktailTwo", "Separation", "Foam"] {
        assert!(c.recipes.contains_key(&format!("reaction:{id}")), "{id}");
    }
    assert_eq!(c.recipes["reaction:CocktailOne"].category, "Напитки");
    assert_eq!(c.recipes["reaction:CocktailTwo"].category, "Напитки");
    assert!(
        c.recipes.contains_key("Cocktail"),
        "legacy product selection preserved"
    );
}
#[test]
fn calculates_multiple_products_and_preserves_conditions() {
    let c = import_documents(&fixture(), "test").unwrap();
    let result = c
        .calculate(vec![ProductionItem {
            name: "reaction:Separation".into(),
            quantity: 8.0,
        }])
        .unwrap();
    let step = result.guides[0].steps.last().unwrap();
    assert_eq!(
        step.products
            .iter()
            .find(|x| x.name == "Вода")
            .unwrap()
            .quantity,
        6.0
    );
    assert_eq!(
        step.products
            .iter()
            .find(|x| x.name == "Соль")
            .unwrap()
            .quantity,
        2.0
    );
    assert_eq!(step.temperature, Some(300.0));
    assert_eq!(step.max_temperature, Some(400.0));
    assert!(step.conditions.iter().any(|s| s.contains("Центрифуга")));
}
#[test]
fn effect_only_reactions_use_batches_and_require_catalysts() {
    let c = import_documents(&fixture(), "test").unwrap();
    let recipe = &c.recipes["reaction:Foam"];
    assert_eq!(recipe.unit, "реакц.");
    let result = c
        .calculate(vec![ProductionItem {
            name: recipe.id.clone(),
            quantity: 3.0,
        }])
        .unwrap();
    assert_eq!(result.direct[0].quantity, 6.0);
    let step = result.guides[0].steps.last().unwrap();
    assert_eq!(step.catalysts[0].quantity, 1.0);
    assert!(step.products.is_empty());
    assert!(!step.effects.is_empty());
    assert!(c
        .calculate(vec![ProductionItem {
            name: recipe.id.clone(),
            quantity: 1.5
        }])
        .is_err());
}
#[test]
fn complete_bundle_is_available_on_clean_offline_start() {
    let temp = tempfile::tempdir().unwrap();
    let c = chem_core::catalog::CatalogRepository::new(temp.path().into()).load();
    assert_eq!(c.entries().count(), 386);
    for id in [
        "Powderbicardine",
        "Smoke",
        "BloodBreakdown",
        "BZElectrolysis",
    ] {
        assert!(c.recipes.contains_key(&format!("reaction:{id}")), "{id}");
    }
}
#[test]
fn reversible_reactions_remain_visible_without_recursive_loops() {
    let docs=vec![SourceDocument{path:"Recipes/Reactions/chemicals.yml".into(),content:"- type: reaction\n  id: Forward\n  reactants: {B: {amount: 1}}\n  products: {A: 1}\n- type: reaction\n  id: Reverse\n  reactants: {A: {amount: 1}}\n  products: {B: 1}\n".into()}];
    let c = import_documents(&docs, "test").unwrap();
    assert_eq!(c.entries().count(), 2);
    let result = c
        .calculate(vec![ProductionItem {
            name: "reaction:Forward".into(),
            quantity: 10.0,
        }])
        .unwrap();
    assert_eq!(result.base[0].name, "B");
    assert_eq!(result.base[0].quantity, 10.0);
    assert!(result.guides[0].steps[0]
        .conditions
        .iter()
        .any(|s| s.contains("готовые")));
}
