use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{
    error::StorageError,
    models::{
        AppSettingRecord, ChampionRecord, ChampionVersionRecord, DraftEventRecord,
        DraftFinalReviewRecord, DraftRecommendationRecord, DraftSessionRecord,
        PlayerChampionPoolRecord, PlayerRecord, TeamPreferenceRecord, TeamRecord,
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageExportBundle {
    pub version: u32,
    pub teams: Vec<TeamRecord>,
    pub players: Vec<PlayerRecord>,
    pub champions: Vec<ChampionRecord>,
    pub champion_versions: Vec<ChampionVersionRecord>,
    pub player_champion_pools: Vec<PlayerChampionPoolRecord>,
    pub team_preferences: Vec<TeamPreferenceRecord>,
    pub draft_sessions: Vec<DraftSessionRecord>,
    pub draft_events: Vec<DraftEventRecord>,
    pub draft_recommendations: Vec<DraftRecommendationRecord>,
    pub draft_final_reviews: Vec<DraftFinalReviewRecord>,
    pub app_settings: Vec<AppSettingRecord>,
}

#[derive(Debug, Clone)]
pub struct StorageImportExportService<'a> {
    pool: &'a SqlitePool,
}

impl<'a> StorageImportExportService<'a> {
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn export_bundle(&self) -> Result<StorageExportBundle, StorageError> {
        Ok(StorageExportBundle {
            version: 1,
            teams: sqlx::query_as::<_, TeamRecord>("SELECT * FROM teams ORDER BY id")
                .fetch_all(self.pool)
                .await?,
            players: sqlx::query_as::<_, PlayerRecord>(
                "SELECT * FROM players ORDER BY team_id, role, handle",
            )
            .fetch_all(self.pool)
            .await?,
            champions: sqlx::query_as::<_, ChampionRecord>("SELECT * FROM champions ORDER BY slug")
                .fetch_all(self.pool)
                .await?,
            champion_versions: sqlx::query_as::<_, ChampionVersionRecord>(
                "SELECT * FROM champion_versions ORDER BY champion_id, patch DESC",
            )
            .fetch_all(self.pool)
            .await?,
            player_champion_pools: sqlx::query_as::<_, PlayerChampionPoolRecord>(
                "SELECT * FROM player_champion_pools ORDER BY player_id, mastery_score DESC",
            )
            .fetch_all(self.pool)
            .await?,
            team_preferences: sqlx::query_as::<_, TeamPreferenceRecord>(
                "SELECT * FROM team_preferences ORDER BY team_id, patch DESC",
            )
            .fetch_all(self.pool)
            .await?,
            draft_sessions: sqlx::query_as::<_, DraftSessionRecord>(
                "SELECT * FROM draft_sessions ORDER BY started_at DESC",
            )
            .fetch_all(self.pool)
            .await?,
            draft_events: sqlx::query_as::<_, DraftEventRecord>(
                "SELECT * FROM draft_events ORDER BY session_id, sequence ASC",
            )
            .fetch_all(self.pool)
            .await?,
            draft_recommendations: sqlx::query_as::<_, DraftRecommendationRecord>(
                "SELECT * FROM draft_recommendations ORDER BY session_id, ranking ASC",
            )
            .fetch_all(self.pool)
            .await?,
            draft_final_reviews: sqlx::query_as::<_, DraftFinalReviewRecord>(
                "SELECT * FROM draft_final_reviews ORDER BY created_at DESC",
            )
            .fetch_all(self.pool)
            .await?,
            app_settings: sqlx::query_as::<_, AppSettingRecord>(
                "SELECT * FROM app_settings ORDER BY key",
            )
            .fetch_all(self.pool)
            .await?,
        })
    }

    pub async fn write_bundle_to_path(&self, path: impl AsRef<Path>) -> Result<(), StorageError> {
        let bundle = self.export_bundle().await?;
        fs::write(path, serde_json::to_vec_pretty(&bundle)?)?;
        Ok(())
    }

    pub async fn import_bundle(&self, bundle: &StorageExportBundle) -> Result<(), StorageError> {
        if bundle.version != 1 {
            return Err(StorageError::InvalidData(format!(
                "unsupported export version {}",
                bundle.version
            )));
        }

        let mut transaction = self.pool.begin().await?;
        for statement in [
            "DELETE FROM draft_final_reviews",
            "DELETE FROM draft_recommendations",
            "DELETE FROM draft_events",
            "DELETE FROM draft_sessions",
            "DELETE FROM team_preferences",
            "DELETE FROM player_champion_pools",
            "DELETE FROM champion_versions",
            "DELETE FROM players",
            "DELETE FROM champions",
            "DELETE FROM teams",
            "DELETE FROM app_settings",
        ] {
            sqlx::query(statement).execute(&mut *transaction).await?;
        }

        for row in &bundle.teams {
            sqlx::query(
                "INSERT INTO teams (id, name, tag, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(&row.id)
            .bind(&row.name)
            .bind(&row.tag)
            .bind(&row.created_at)
            .bind(&row.updated_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.players {
            sqlx::query(
                "INSERT INTO players (id, team_id, handle, role, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .bind(&row.id)
            .bind(&row.team_id)
            .bind(&row.handle)
            .bind(&row.role)
            .bind(&row.status)
            .bind(&row.created_at)
            .bind(&row.updated_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.champions {
            sqlx::query(
                "INSERT INTO champions (id, slug, name, canonical_role, archetypes_json, incomplete_profile, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&row.id)
            .bind(&row.slug)
            .bind(&row.name)
            .bind(&row.canonical_role)
            .bind(&row.archetypes_json)
            .bind(row.incomplete_profile)
            .bind(&row.created_at)
            .bind(&row.updated_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.champion_versions {
            sqlx::query(
                "INSERT INTO champion_versions (id, champion_id, patch, source_path, payload_json, compatible_since, is_latest, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&row.id)
            .bind(&row.champion_id)
            .bind(&row.patch)
            .bind(&row.source_path)
            .bind(&row.payload_json)
            .bind(&row.compatible_since)
            .bind(row.is_latest)
            .bind(&row.created_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.player_champion_pools {
            sqlx::query(
                "INSERT INTO player_champion_pools (id, player_id, champion_id, patch, mastery_score, proficiency_tier, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .bind(&row.id)
            .bind(&row.player_id)
            .bind(&row.champion_id)
            .bind(&row.patch)
            .bind(row.mastery_score)
            .bind(&row.proficiency_tier)
            .bind(&row.notes)
            .bind(&row.created_at)
            .bind(&row.updated_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.team_preferences {
            sqlx::query(
                "INSERT INTO team_preferences (id, team_id, patch, preferences_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&row.id)
            .bind(&row.team_id)
            .bind(&row.patch)
            .bind(&row.preferences_json)
            .bind(&row.created_at)
            .bind(&row.updated_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.draft_sessions {
            sqlx::query(
                "INSERT INTO draft_sessions (id, team_id, mode, patch, side, status, metadata_json, started_at, ended_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .bind(&row.id)
            .bind(&row.team_id)
            .bind(&row.mode)
            .bind(&row.patch)
            .bind(&row.side)
            .bind(&row.status)
            .bind(&row.metadata_json)
            .bind(&row.started_at)
            .bind(&row.ended_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.draft_events {
            sqlx::query(
                "INSERT INTO draft_events (id, session_id, sequence, phase, team, action, champion_id, payload_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .bind(&row.id)
            .bind(&row.session_id)
            .bind(row.sequence)
            .bind(&row.phase)
            .bind(&row.team)
            .bind(&row.action)
            .bind(&row.champion_id)
            .bind(&row.payload_json)
            .bind(&row.created_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.draft_recommendations {
            sqlx::query(
                "INSERT INTO draft_recommendations (id, session_id, event_id, candidate_champion_id, ranking, score, reasoning_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .bind(&row.id)
            .bind(&row.session_id)
            .bind(&row.event_id)
            .bind(&row.candidate_champion_id)
            .bind(row.ranking)
            .bind(row.score)
            .bind(&row.reasoning_json)
            .bind(&row.created_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.draft_final_reviews {
            sqlx::query(
                "INSERT INTO draft_final_reviews (id, session_id, summary, review_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(&row.id)
            .bind(&row.session_id)
            .bind(&row.summary)
            .bind(&row.review_json)
            .bind(&row.created_at)
            .execute(&mut *transaction)
            .await?;
        }

        for row in &bundle.app_settings {
            sqlx::query(
                "INSERT INTO app_settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)",
            )
            .bind(&row.key)
            .bind(&row.value_json)
            .bind(&row.updated_at)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn read_bundle_from_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<StorageExportBundle, StorageError> {
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }
}
