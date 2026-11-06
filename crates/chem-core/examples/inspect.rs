use chem_core::{catalog::CatalogRepository, ProductionItem};
fn main() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
    let repository = CatalogRepository::new(directory.path().into());
    let catalog = if std::env::args().any(|arg| arg == "--online") {
        repository.refresh()?
    } else {
        repository.load()
    };
    println!(
        "Catalog: {} recipes ({})",
        catalog.recipes.len(),
        catalog.source
    );
    for id in catalog.recipes.keys() {
        catalog.calculate(vec![ProductionItem {
            name: id.clone(),
            quantity: 30.0,
        }])?;
    }
    let cached = repository.load();
    if catalog.source == "online" {
        assert_eq!(catalog.recipes.len(), cached.recipes.len());
    }
    println!("Every recipe calculated successfully");
    Ok(())
}
