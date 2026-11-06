use crate::importer::validate_catalog;
use crate::storage::atomic_write;
use crate::Catalog;
use std::path::PathBuf;

pub use crate::importer::{import_documents, SourceDocument};
pub struct Sources {
    pub recipes: String,
    pub reagents: String,
    pub localization: String,
}

pub fn builtin() -> Catalog {
    serde_json::from_str(include_str!("../data/sunrise-catalog.json"))
        .expect("validated complete Sunrise snapshot")
}
pub fn parse_sources(sources: &Sources) -> Result<Catalog, String> {
    import_documents(
        &[
            SourceDocument {
                path: "Resources/Prototypes/Recipes/Reactions/medicine.yml".into(),
                content: sources.recipes.clone(),
            },
            SourceDocument {
                path: "Resources/Prototypes/Reagents/medicine.yml".into(),
                content: sources.reagents.clone(),
            },
            SourceDocument {
                path: "Resources/Locale/ru-RU/reagents.ftl".into(),
                content: sources.localization.clone(),
            },
        ],
        "provided-sources",
    )
}

pub struct CatalogRepository(PathBuf);
impl CatalogRepository {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }
    pub fn snapshot_path(&self) -> PathBuf {
        self.0.join("catalog.json")
    }
    pub fn load(&self) -> Catalog {
        if let Ok(bytes) = std::fs::read(self.snapshot_path()) {
            if let Ok(mut catalog) = serde_json::from_slice::<Catalog>(&bytes) {
                // Version 1 caches contain only 35/87 medicines. Never let them
                // shadow the complete bundled catalog after an upgrade.
                if catalog.metadata.schema == 2 && validate_catalog(&catalog).is_ok() {
                    catalog.source = "cache".into();
                    return catalog;
                }
            }
        }
        builtin()
    }
    pub fn replace(&self, sources: &Sources) -> Result<Catalog, String> {
        self.persist(parse_sources(sources)?)
    }
    pub fn persist(&self, catalog: Catalog) -> Result<Catalog, String> {
        validate_catalog(&catalog)?;
        atomic_write(
            &self.snapshot_path(),
            &serde_json::to_vec(&catalog).map_err(|e| e.to_string())?,
        )?;
        Ok(catalog)
    }
    pub fn refresh(&self) -> Result<Catalog, String> {
        let catalog = crate::sunrise::refresh(&self.load())?;
        self.persist(catalog)
    }
}
