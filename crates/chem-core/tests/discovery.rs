use chem_core::sunrise::{discover_paths, GitEntry, GitTree};
fn entry(path: &str) -> GitEntry {
    GitEntry {
        path: path.into(),
        kind: "blob".into(),
        sha: "test".into(),
    }
}
#[test]
fn finds_server_recipe_directories_and_nested_reagents() {
    let prototypes = GitTree {
        truncated: false,
        tree: vec![
            entry("Recipes/Reactions/medicine.yml"),
            entry("_Sunrise/Chemistry/Recipes/medicine.yml"),
            entry("Reagents/Consumable/Drink/alcohol.yml"),
            entry("Recipes/Lathes/tools.yml"),
        ],
    };
    let locales = GitTree {
        truncated: false,
        tree: vec![entry("_strings/_sunrise/reagents/meta/medicine.ftl")],
    };
    let result = discover_paths(&prototypes, &locales, &[]).unwrap();
    assert_eq!(result.len(), 5);
    assert!(result.iter().any(|p| p.contains("/Chemistry/Recipes/")));
    // Every YAML is a discovery candidate; contents decide whether it is chemical.
    assert!(result.iter().any(|p| p.contains("Lathes")));
}
#[test]
fn discovers_relocated_chemistry_and_localization_outside_known_folders() {
    let prototypes = GitTree {
        truncated: false,
        tree: vec![
            entry("Recipes/Reactions/medicine.yml"),
            entry("_Sunrise/Chemistry/new-reactions.yml"),
        ],
    };
    let locales = GitTree {
        truncated: false,
        tree: vec![entry("_Sunrise/chemistry/new-labels.ftl")],
    };
    let paths = discover_paths(&prototypes, &locales, &[]).unwrap();
    assert!(paths.iter().any(|p| p.ends_with("new-reactions.yml")));
    assert!(paths.iter().any(|p| p.ends_with("new-labels.ftl")));
}
#[test]
fn refuses_truncated_tree_instead_of_silently_losing_server_recipes() {
    let prototypes = GitTree {
        truncated: true,
        tree: vec![entry("Recipes/Reactions/medicine.yml")],
    };
    assert!(discover_paths(&prototypes, &GitTree::default(), &[]).is_err());
}
#[test]
fn ignores_legacy_medicine_only_cache_on_upgrade() {
    let temp = tempfile::tempdir().unwrap();
    let legacy = serde_json::json!({"recipes":{},"names":{},"source":"cache"});
    std::fs::write(temp.path().join("catalog.json"), legacy.to_string()).unwrap();
    let catalog = chem_core::catalog::CatalogRepository::new(temp.path().into()).load();
    assert_eq!(catalog.entries().count(), 386);
    assert_eq!(catalog.metadata.schema, 2);
}
#[test]
fn complete_snapshot_matches_every_audited_id() {
    let coverage: serde_json::Value =
        serde_json::from_str(include_str!("../data/coverage.json")).unwrap();
    let catalog = chem_core::catalog::builtin();
    let expected: std::collections::BTreeSet<_> = coverage["reactionIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap())
        .collect();
    let actual: std::collections::BTreeSet<_> =
        catalog.entries().map(|r| r.reaction_id.as_str()).collect();
    assert_eq!(actual, expected);
    assert_eq!(
        catalog.metadata.revision,
        coverage["revision"].as_str().unwrap()
    );
    for recipe in catalog.entries() {
        assert!(!recipe.category.is_empty());
        catalog
            .calculate(vec![chem_core::ProductionItem {
                name: recipe.id.clone(),
                quantity: recipe.batch_size,
            }])
            .unwrap();
    }
}
