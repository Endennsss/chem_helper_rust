use chem_core::*;
use std::collections::BTreeMap;

fn graph() -> Catalog {
    let mut catalog = Catalog::default();
    for (id, parts) in [
        ("Target", vec![("Left", 1.0), ("Right", 1.0)]),
        ("Left", vec![("Shared", 1.0)]),
        ("Right", vec![("Shared", 1.0)]),
        ("Shared", vec![("Raw", 1.0)]),
    ] {
        catalog.recipes.insert(id.into(), Recipe { id: id.into(), name: id.into(), ingredients: parts.into_iter().map(|(k,v)|(k.into(),v)).collect(), ..Recipe::default() });
    }
    catalog
}
fn item(name: &str, quantity: f64) -> ProductionItem { ProductionItem { name: name.into(), quantity } }

#[test]
fn sums_shared_intermediates_in_guide() {
    let result = graph().calculate(vec![item("Target", 10.0)]).unwrap();
    assert_eq!(result.base[0].quantity, 20.0);
    let shared = result.guides[0].steps.iter().find(|s| s.product == "Shared").unwrap();
    assert_eq!(shared.quantity, 20.0);
    assert_eq!(shared.ingredients[0].quantity, 20.0);
    assert_eq!(result.guides[0].steps.last().unwrap().product, "Target");
}
#[test]
fn rounds_after_aggregating_queue() {
    let mut catalog = graph();
    catalog.recipes.get_mut("Shared").unwrap().ingredients.insert("Raw".into(), 0.4);
    let result = catalog.calculate(vec![item("Shared",1.0),item("Shared",1.0)]).unwrap();
    assert_eq!(result.base[0].quantity, 1.0);
    assert_eq!(result.direct[0].quantity, 1.0);
}
#[test]
fn rejects_non_finite_and_non_positive_quantities() {
    for quantity in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e300] {
        assert!(graph().calculate(vec![item("Target", quantity)]).is_err());
    }
    assert!(graph().calculate(vec![item("Missing", 1.0)]).is_err());
}
#[test]
fn detects_cycles() {
    let mut catalog = graph();
    catalog.recipes.get_mut("Shared").unwrap().ingredients = BTreeMap::from([("Target".into(),1.0)]);
    let error = catalog.calculate(vec![item("Target",1.0)]).unwrap_err();
    assert!(error.contains("цикл"), "{error}");
}
#[test]
fn catalyst_is_required_but_not_consumed() {
    let mut catalog = graph();
    let shared = catalog.recipes.get_mut("Shared").unwrap();
    shared.catalysts.insert("Plasma".into(),1.0);
    shared.temperature = Some(370.0);
    let result = catalog.calculate(vec![item("Shared",3.0)]).unwrap();
    assert!(!result.base.iter().any(|r| r.name == "Plasma"));
    assert_eq!(result.guides[0].steps[0].catalysts[0].name,"Plasma");
    assert_eq!(result.guides[0].steps[0].temperature,Some(370.0));
}
#[test]
fn resolves_localized_case_insensitive_names() {
    let mut catalog = graph();
    catalog.recipes.get_mut("Target").unwrap().name = "Препарат".into();
    let result = catalog.calculate(vec![item(" препарат ",10.0)]).unwrap();
    assert_eq!(result.items[0].name,"Препарат");
}
