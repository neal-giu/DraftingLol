use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct TeamRecord {
    pub id: String,
    pub name: String,
    pub tag: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct PlayerRecord {
    pub id: String,
    pub team_id: String,
    pub handle: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ChampionRecord {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub canonical_role: String,
    pub archetypes_json: String,
    pub incomplete_profile: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ChampionVersionRecord {
    pub id: String,
    pub champion_id: String,
    pub patch: String,
    pub source_path: String,
    pub payload_json: String,
    pub compatible_since: String,
    pub is_latest: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct PlayerChampionPoolRecord {
    pub id: String,
    pub player_id: String,
    pub champion_id: String,
    pub patch: String,
    pub mastery_score: f64,
    pub proficiency_tier: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct TeamPreferenceRecord {
    pub id: String,
    pub team_id: String,
    pub patch: String,
    pub preferences_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DraftSessionRecord {
    pub id: String,
    pub team_id: Option<String>,
    pub mode: String,
    pub patch: String,
    pub side: String,
    pub status: String,
    pub metadata_json: String,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DraftEventRecord {
    pub id: String,
    pub session_id: String,
    pub sequence: i64,
    pub phase: String,
    pub team: String,
    pub action: String,
    pub champion_id: Option<String>,
    pub payload_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct DraftRecommendationRecord {
    pub id: String,
    pub session_id: String,
    pub event_id: Option<String>,
    pub candidate_champion_id: String,
    pub ranking: i64,
    pub score: f64,
    pub reasoning_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DraftFinalReviewRecord {
    pub id: String,
    pub session_id: String,
    pub summary: String,
    pub review_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct AppSettingRecord {
    pub key: String,
    pub value_json: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamRosterConfiguration {
    pub team: TeamRecord,
    pub players: Vec<PlayerWithChampionPool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerWithChampionPool {
    pub player: PlayerRecord,
    pub champion_pool: Vec<PlayerChampionPoolRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftSessionHistory {
    pub session: DraftSessionRecord,
    pub events: Vec<DraftEventRecord>,
    pub recommendations: Vec<DraftRecommendationRecord>,
    pub final_review: Option<DraftFinalReviewRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChampionPatchPayload {
    pub id: String,
    pub patch: String,
    pub payload_json: String,
    pub source_path: String,
    pub incomplete_profile: bool,
}
