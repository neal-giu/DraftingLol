use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    catalog_service::CatalogService, live_service::LiveService,
    recommendation_service::RecommendationService,
};
use crate::storage::{StorageError, StorageManager};

#[derive(Debug, Clone)]
pub struct AppState {
    storage: StorageManager,
    catalog: CatalogService,
    recommendation: RecommendationService,
    live: LiveService,
    database_path: PathBuf,
}

impl AppState {
    pub async fn initialize() -> Result<Self, StorageError> {
        let database_path = Self::database_path();
        let storage = StorageManager::connect_file(&database_path).await?;

        Ok(Self {
            storage,
            catalog: CatalogService::new(database_path.clone()),
            recommendation: RecommendationService::default(),
            live: LiveService::default(),
            database_path,
        })
    }

    #[must_use]
    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    #[must_use]
    pub fn storage(&self) -> &StorageManager {
        &self.storage
    }

    #[must_use]
    pub fn catalog(&self) -> &CatalogService {
        &self.catalog
    }

    #[must_use]
    pub fn recommendation(&self) -> &RecommendationService {
        &self.recommendation
    }

    #[must_use]
    pub fn live(&self) -> &LiveService {
        &self.live
    }

    fn database_path() -> PathBuf {
        if let Ok(explicit_path) = std::env::var("DRAFT_TEAM_APP_DB_PATH") {
            return PathBuf::from(explicit_path);
        }

        std::env::current_dir()
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("draft-team-app.sqlite")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn initializes_runtime_services() {
        let db_path = unique_temp_db_path();
        std::env::set_var("DRAFT_TEAM_APP_DB_PATH", &db_path);

        let state = AppState::initialize().await.expect("app state init");

        assert_eq!(state.database_path(), db_path.as_path());
        assert_eq!(state.catalog().source_database(), db_path.as_path());
        assert_eq!(
            state.recommendation().engine_name(),
            "draft_recommendation_engine"
        );
        assert_eq!(state.live().status_label(), "not_started");
        assert!(!state.storage().pool().is_closed());

        std::env::remove_var("DRAFT_TEAM_APP_DB_PATH");
        let _ = std::fs::remove_file(db_path);
    }

    fn unique_temp_db_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();

        std::env::temp_dir().join(format!("draft-team-app-test-{nonce}.sqlite"))
    }
}
