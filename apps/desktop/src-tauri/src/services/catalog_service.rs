use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CatalogService {
    source_database: PathBuf,
}

impl CatalogService {
    #[must_use]
    pub fn new(source_database: PathBuf) -> Self {
        Self { source_database }
    }

    #[must_use]
    pub fn source_database(&self) -> &Path {
        &self.source_database
    }
}
