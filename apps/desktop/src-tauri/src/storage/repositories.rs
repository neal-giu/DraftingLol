use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

use super::{
    error::StorageError,
    models::{
        AppSettingRecord, ChampionPatchPayload, ChampionRecord, ChampionVersionRecord,
        DraftEventRecord, DraftFinalReviewRecord, DraftRecommendationRecord, DraftSessionHistory,
        DraftSessionRecord, PlayerChampionPoolRecord, PlayerRecord, PlayerWithChampionPool,
        TeamPreferenceRecord, TeamRecord, TeamRosterConfiguration,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertTeamInput {
    pub id: String,
    pub name: String,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpsertPlayerInput {
    pub id: String,
    pub handle: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpsertChampionPoolInput {
    pub champion_id: String,
    pub patch: String,
    pub mastery_score: f64,
    pub proficiency_tier: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftSessionInput {
    pub id: String,
    pub team_id: Option<String>,
    pub mode: String,
    pub patch: String,
    pub side: String,
    pub status: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftEventInput {
    pub id: String,
    pub sequence: i64,
    pub phase: String,
    pub team: String,
    pub action: String,
    pub champion_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftRecommendationInput {
    pub id: String,
    pub event_id: Option<String>,
    pub candidate_champion_id: String,
    pub ranking: i64,
    pub score: f64,
    pub reasoning: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftReviewInput {
    pub id: String,
    pub summary: String,
    pub review: Value,
}

#[derive(Debug, Clone)]
pub struct TeamConfigurationRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> TeamConfigurationRepository<'a> {
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert_team(&self, input: &UpsertTeamInput) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO teams (id, name, tag)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                tag = excluded.tag,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(&input.id)
        .bind(&input.name)
        .bind(&input.tag)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn replace_roster(
        &self,
        team_id: &str,
        players: &[UpsertPlayerInput],
    ) -> Result<(), StorageError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("DELETE FROM players WHERE team_id = ?1")
            .bind(team_id)
            .execute(&mut *transaction)
            .await?;

        for player in players {
            sqlx::query(
                r#"
                INSERT INTO players (id, team_id, handle, role, status)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .bind(&player.id)
            .bind(team_id)
            .bind(&player.handle)
            .bind(&player.role)
            .bind(&player.status)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn replace_player_champion_pool(
        &self,
        player_id: &str,
        entries: &[UpsertChampionPoolInput],
    ) -> Result<(), StorageError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("DELETE FROM player_champion_pools WHERE player_id = ?1")
            .bind(player_id)
            .execute(&mut *transaction)
            .await?;

        for (index, entry) in entries.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO player_champion_pools (
                    id, player_id, champion_id, patch, mastery_score, proficiency_tier, notes
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
            )
            .bind(format!("{player_id}:{index}:{}", entry.champion_id))
            .bind(player_id)
            .bind(&entry.champion_id)
            .bind(&entry.patch)
            .bind(entry.mastery_score)
            .bind(&entry.proficiency_tier)
            .bind(&entry.notes)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_roster_configuration(
        &self,
        team_id: &str,
    ) -> Result<TeamRosterConfiguration, StorageError> {
        let team = sqlx::query_as::<_, TeamRecord>("SELECT * FROM teams WHERE id = ?1")
            .bind(team_id)
            .fetch_optional(self.pool)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("team {team_id}")))?;

        let players = sqlx::query_as::<_, PlayerRecord>(
            "SELECT * FROM players WHERE team_id = ?1 ORDER BY role, handle",
        )
        .bind(team_id)
        .fetch_all(self.pool)
        .await?;

        let mut enriched_players = Vec::with_capacity(players.len());
        for player in players {
            let champion_pool = sqlx::query_as::<_, PlayerChampionPoolRecord>(
                "SELECT * FROM player_champion_pools WHERE player_id = ?1 ORDER BY mastery_score DESC, champion_id ASC",
            )
            .bind(&player.id)
            .fetch_all(self.pool)
            .await?;

            enriched_players.push(PlayerWithChampionPool {
                player,
                champion_pool,
            });
        }

        Ok(TeamRosterConfiguration {
            team,
            players: enriched_players,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TeamPreferencesRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> TeamPreferencesRepository<'a> {
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn save_preferences<T: Serialize>(
        &self,
        team_id: &str,
        patch: &str,
        preferences: &T,
    ) -> Result<(), StorageError> {
        let id = format!("{team_id}:{patch}");
        let preferences_json = serde_json::to_string(preferences)?;

        sqlx::query(
            r#"
            INSERT INTO team_preferences (id, team_id, patch, preferences_json)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                preferences_json = excluded.preferences_json,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(id)
        .bind(team_id)
        .bind(patch)
        .bind(preferences_json)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_preferences<T: DeserializeOwned>(
        &self,
        team_id: &str,
        patch: &str,
    ) -> Result<Option<T>, StorageError> {
        let record = sqlx::query_as::<_, TeamPreferenceRecord>(
            "SELECT * FROM team_preferences WHERE team_id = ?1 AND patch = ?2",
        )
        .bind(team_id)
        .bind(patch)
        .fetch_optional(self.pool)
        .await?;

        record
            .map(|row| serde_json::from_str(&row.preferences_json))
            .transpose()
            .map_err(StorageError::from)
    }
}

#[derive(Debug, Clone)]
pub struct ChampionRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ChampionRepository<'a> {
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert_champion(
        &self,
        id: &str,
        slug: &str,
        name: &str,
        canonical_role: &str,
        archetypes: &[String],
        incomplete_profile: bool,
    ) -> Result<(), StorageError> {
        let archetypes_json = serde_json::to_string(archetypes)?;

        sqlx::query(
            r#"
            INSERT INTO champions (id, slug, name, canonical_role, archetypes_json, incomplete_profile)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                slug = excluded.slug,
                name = excluded.name,
                canonical_role = excluded.canonical_role,
                archetypes_json = excluded.archetypes_json,
                incomplete_profile = excluded.incomplete_profile,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(id)
        .bind(slug)
        .bind(name)
        .bind(canonical_role)
        .bind(archetypes_json)
        .bind(incomplete_profile)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_champion_version(
        &self,
        record: &ChampionVersionRecord,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO champion_versions (
                id, champion_id, patch, source_path, payload_json, compatible_since, is_latest
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                source_path = excluded.source_path,
                payload_json = excluded.payload_json,
                compatible_since = excluded.compatible_since,
                is_latest = excluded.is_latest
            "#,
        )
        .bind(&record.id)
        .bind(&record.champion_id)
        .bind(&record.patch)
        .bind(&record.source_path)
        .bind(&record.payload_json)
        .bind(&record.compatible_since)
        .bind(record.is_latest)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn resolve_patch_payload(
        &self,
        slug: &str,
        requested_patch: &str,
    ) -> Result<Option<ChampionPatchPayload>, StorageError> {
        let champion =
            sqlx::query_as::<_, ChampionRecord>("SELECT * FROM champions WHERE slug = ?1")
                .bind(slug)
                .fetch_optional(self.pool)
                .await?;

        let Some(champion) = champion else {
            return Ok(None);
        };

        let versions = sqlx::query_as::<_, ChampionVersionRecord>(
            "SELECT * FROM champion_versions WHERE champion_id = ?1",
        )
        .bind(&champion.id)
        .fetch_all(self.pool)
        .await?;

        let matched = versions
            .into_iter()
            .filter(|version| {
                compare_patch(version.compatible_since.as_str(), requested_patch) <= 0
            })
            .max_by(|left, right| {
                compare_patch(
                    left.compatible_since.as_str(),
                    right.compatible_since.as_str(),
                )
                .then(compare_patch(left.patch.as_str(), right.patch.as_str()))
                .then(left.is_latest.cmp(&right.is_latest))
            });

        Ok(matched.map(|version| ChampionPatchPayload {
            id: champion.id,
            patch: version.patch,
            payload_json: version.payload_json,
            source_path: version.source_path,
            incomplete_profile: champion.incomplete_profile,
        }))
    }
}

fn compare_patch(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts = left
        .split('.')
        .map(|value| value.parse::<u32>().unwrap_or_default())
        .collect::<Vec<_>>();
    let right_parts = right
        .split('.')
        .map(|value| value.parse::<u32>().unwrap_or_default())
        .collect::<Vec<_>>();

    for index in 0..left_parts.len().max(right_parts.len()) {
        let ordering = left_parts
            .get(index)
            .copied()
            .unwrap_or_default()
            .cmp(&right_parts.get(index).copied().unwrap_or_default());

        if !ordering.is_eq() {
            return ordering;
        }
    }

    std::cmp::Ordering::Equal
}

#[derive(Debug, Clone)]
pub struct DraftSessionRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> DraftSessionRepository<'a> {
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_session(&self, input: &DraftSessionInput) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO draft_sessions (id, team_id, mode, patch, side, status, metadata_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                team_id = excluded.team_id,
                mode = excluded.mode,
                patch = excluded.patch,
                side = excluded.side,
                status = excluded.status,
                metadata_json = excluded.metadata_json
            "#,
        )
        .bind(&input.id)
        .bind(&input.team_id)
        .bind(&input.mode)
        .bind(&input.patch)
        .bind(&input.side)
        .bind(&input.status)
        .bind(serde_json::to_string(&input.metadata)?)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn append_event(
        &self,
        session_id: &str,
        input: &DraftEventInput,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO draft_events (id, session_id, sequence, phase, team, action, champion_id, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&input.id)
        .bind(session_id)
        .bind(input.sequence)
        .bind(&input.phase)
        .bind(&input.team)
        .bind(&input.action)
        .bind(&input.champion_id)
        .bind(serde_json::to_string(&input.payload)?)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_recommendations(
        &self,
        session_id: &str,
        recommendations: &[DraftRecommendationInput],
    ) -> Result<(), StorageError> {
        let mut transaction = self.pool.begin().await?;

        for recommendation in recommendations {
            sqlx::query(
                r#"
                INSERT INTO draft_recommendations (
                    id, session_id, event_id, candidate_champion_id, ranking, score, reasoning_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(id) DO UPDATE SET
                    event_id = excluded.event_id,
                    candidate_champion_id = excluded.candidate_champion_id,
                    ranking = excluded.ranking,
                    score = excluded.score,
                    reasoning_json = excluded.reasoning_json
                "#,
            )
            .bind(&recommendation.id)
            .bind(session_id)
            .bind(&recommendation.event_id)
            .bind(&recommendation.candidate_champion_id)
            .bind(recommendation.ranking)
            .bind(recommendation.score)
            .bind(serde_json::to_string(&recommendation.reasoning)?)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_final_review(
        &self,
        session_id: &str,
        review: &DraftReviewInput,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO draft_final_reviews (id, session_id, summary, review_json)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                summary = excluded.summary,
                review_json = excluded.review_json
            "#,
        )
        .bind(&review.id)
        .bind(session_id)
        .bind(&review.summary)
        .bind(serde_json::to_string(&review.review)?)
        .execute(self.pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE draft_sessions
            SET status = 'completed', ended_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            "#,
        )
        .bind(session_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_history(
        &self,
        session_id: &str,
    ) -> Result<Option<DraftSessionHistory>, StorageError> {
        let session =
            sqlx::query_as::<_, DraftSessionRecord>("SELECT * FROM draft_sessions WHERE id = ?1")
                .bind(session_id)
                .fetch_optional(self.pool)
                .await?;

        let Some(session) = session else {
            return Ok(None);
        };

        let events = sqlx::query_as::<_, DraftEventRecord>(
            "SELECT * FROM draft_events WHERE session_id = ?1 ORDER BY sequence ASC",
        )
        .bind(session_id)
        .fetch_all(self.pool)
        .await?;

        let recommendations = sqlx::query_as::<_, DraftRecommendationRecord>(
            "SELECT * FROM draft_recommendations WHERE session_id = ?1 ORDER BY ranking ASC, score DESC",
        )
        .bind(session_id)
        .fetch_all(self.pool)
        .await?;

        let final_review = sqlx::query_as::<_, DraftFinalReviewRecord>(
            "SELECT * FROM draft_final_reviews WHERE session_id = ?1 LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(Some(DraftSessionHistory {
            session,
            events,
            recommendations,
            final_review,
        }))
    }

    pub async fn list_recent_history(
        &self,
        limit: i64,
    ) -> Result<Vec<DraftSessionHistory>, StorageError> {
        let sessions = sqlx::query_as::<_, DraftSessionRecord>(
            "SELECT * FROM draft_sessions ORDER BY started_at DESC LIMIT ?1",
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        let mut history = Vec::with_capacity(sessions.len());
        for session in sessions {
            if let Some(item) = self.load_history(&session.id).await? {
                history.push(item);
            }
        }

        Ok(history)
    }
}

#[derive(Debug, Clone)]
pub struct SettingsRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SettingsRepository<'a> {
    #[must_use]
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value_json)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(key)
        .bind(serde_json::to_string(value)?)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StorageError> {
        let record =
            sqlx::query_as::<_, AppSettingRecord>("SELECT * FROM app_settings WHERE key = ?1")
                .bind(key)
                .fetch_optional(self.pool)
                .await?;

        record
            .map(|row| serde_json::from_str(&row.value_json))
            .transpose()
            .map_err(StorageError::from)
    }
}
