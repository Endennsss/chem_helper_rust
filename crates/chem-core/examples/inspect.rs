use chem_core::{catalog::CatalogRepository, ProductionItem};
fn main() -> Result<(), String> {
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
    let repository = CatalogRepository::new(directory.path().into());
    if std::env::args().any(|arg| arg == "--force") {
        let mut current = repository.load();
        current.metadata.revision = "force-discovery".into();
        repository.persist(current)?;
    }
    let catalog = if std::env::args().any(|arg| arg == "--online") {
        repository.refresh()?
    } else {
        repository.load()
    };
    println!(
        "Catalog: {} visible reactions, {} graph entries ({})",
        catalog.entries().count(),
        catalog.recipes.len(),
        catalog.source
    );
    for recipe in catalog.entries() {
        catalog.calculate(vec![ProductionItem {
            name: recipe.id.clone(),
            quantity: recipe.batch_size,
        }])?;
    }
    let cached = repository.load();
    if catalog.source == "online" {
        assert_eq!(catalog.entries().count(), cached.entries().count());
    }
    println!(
        "Every reaction calculated successfully; source revision {}",
        catalog.metadata.revision
    );
    Ok(())
}
