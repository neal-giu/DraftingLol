pub mod connection;
pub mod error;
pub mod import_export;
pub mod models;
pub mod repositories;

pub use connection::StorageManager;
pub use error::StorageError;
pub use import_export::{StorageExportBundle, StorageImportExportService};
pub use models::*;
pub use repositories::*;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        connection::StorageManager,
        import_export::StorageImportExportService,
        repositories::{
            ChampionRepository, DraftEventInput, DraftRecommendationInput, DraftReviewInput,
            DraftSessionInput, DraftSessionRepository, TeamConfigurationRepository,
            TeamPreferencesRepository, UpsertChampionPoolInput, UpsertPlayerInput, UpsertTeamInput,
        },
    };

    #[tokio::test]
    async fn stores_roster_preferences_history_and_export_bundle() {
        let storage = StorageManager::in_memory().await.expect("storage");
        let pool = storage.pool();

        let team_repo = TeamConfigurationRepository::new(pool);
        let pref_repo = TeamPreferencesRepository::new(pool);
        let draft_repo = DraftSessionRepository::new(pool);
        let champion_repo = ChampionRepository::new(pool);
        let import_export = StorageImportExportService::new(pool);

        champion_repo
            .upsert_champion(
                "champ-ahri",
                "ahri",
                "Ahri",
                "mid",
                &[String::from("pick"), String::from("mobility")],
                false,
            )
            .await
            .expect("champion insert");
        champion_repo
            .upsert_champion(
                "champ-ornn",
                "ornn",
                "Ornn",
                "top",
                &[String::from("engage")],
                false,
            )
            .await
            .expect("champion insert");
        champion_repo
            .upsert_champion_version(&crate::storage::ChampionVersionRecord {
                id: "ahri:15.5".into(),
                champion_id: "champ-ahri".into(),
                patch: "15.5".into(),
                source_path: "packages/champion-data/patches/15.5/champions/ahri.json".into(),
                payload_json: json!({"name": "Ahri", "patch": "15.5"}).to_string(),
                compatible_since: "15.5".into(),
                is_latest: false,
                created_at: "2026-01-01 00:00:00".into(),
            })
            .await
            .expect("version insert");

        team_repo
            .upsert_team(&UpsertTeamInput {
                id: "team-solary".into(),
                name: "Solary Academy".into(),
                tag: Some("SLY".into()),
            })
            .await
            .expect("team insert");
        team_repo
            .replace_roster(
                "team-solary",
                &[
                    UpsertPlayerInput {
                        id: "player-mid".into(),
                        handle: "Lunaris".into(),
                        role: "mid".into(),
                        status: "starter".into(),
                    },
                    UpsertPlayerInput {
                        id: "player-top".into(),
                        handle: "Aegis".into(),
                        role: "top".into(),
                        status: "starter".into(),
                    },
                ],
            )
            .await
            .expect("roster insert");
        team_repo
            .replace_player_champion_pool(
                "player-mid",
                &[UpsertChampionPoolInput {
                    champion_id: "champ-ahri".into(),
                    patch: "15.6".into(),
                    mastery_score: 92.5,
                    proficiency_tier: "signature".into(),
                    notes: Some("Blind pick confortable".into()),
                }],
            )
            .await
            .expect("pool insert");
        pref_repo
            .save_preferences(
                "team-solary",
                "15.6",
                &json!({"tempo": "mid_game", "ban_targets": ["Vi", "Kalista"]}),
            )
            .await
            .expect("preferences save");

        draft_repo
            .create_session(&DraftSessionInput {
                id: "session-1".into(),
                team_id: Some("team-solary".into()),
                mode: "live".into(),
                patch: "15.6".into(),
                side: "blue".into(),
                status: "active".into(),
                metadata: json!({"source": "lcu"}),
            })
            .await
            .expect("session create");
        draft_repo
            .append_event(
                "session-1",
                &DraftEventInput {
                    id: "event-1".into(),
                    sequence: 1,
                    phase: "ban_phase_one".into(),
                    team: "ally".into(),
                    action: "ban".into(),
                    champion_id: Some("champ-ahri".into()),
                    payload: json!({"slot": 1}),
                },
            )
            .await
            .expect("event save");
        draft_repo
            .save_recommendations(
                "session-1",
                &[DraftRecommendationInput {
                    id: "reco-1".into(),
                    event_id: Some("event-1".into()),
                    candidate_champion_id: "champ-ornn".into(),
                    ranking: 1,
                    score: 87.3,
                    reasoning: json!({"fit": "engage", "risk": "low"}),
                }],
            )
            .await
            .expect("recommendations save");
        draft_repo
            .save_final_review(
                "session-1",
                &DraftReviewInput {
                    id: "review-1".into(),
                    summary: "Draft solide avec engage fiable".into(),
                    review: json!({"grade": "A", "notes": ["Frontline forte"]}),
                },
            )
            .await
            .expect("review save");

        let roster = team_repo
            .get_roster_configuration("team-solary")
            .await
            .expect("roster load");
        assert_eq!(roster.players.len(), 2);
        let mid_player = roster
            .players
            .iter()
            .find(|player| player.player.id == "player-mid")
            .expect("mid player");
        assert_eq!(mid_player.champion_pool.len(), 1);

        let preferences: serde_json::Value = pref_repo
            .get_preferences("team-solary", "15.6")
            .await
            .expect("preference load")
            .expect("preference value");
        assert_eq!(preferences["tempo"], "mid_game");

        let history = draft_repo
            .load_history("session-1")
            .await
            .expect("history load")
            .expect("history record");
        assert_eq!(history.events.len(), 1);
        assert_eq!(history.recommendations.len(), 1);
        assert!(history.final_review.is_some());

        let bundle = import_export.export_bundle().await.expect("bundle export");
        assert_eq!(bundle.teams.len(), 1);
        assert_eq!(bundle.draft_sessions.len(), 1);
    }

    #[tokio::test]
    async fn resolves_patch_fallback_to_latest_compatible_version() {
        let storage = StorageManager::in_memory().await.expect("storage");
        let pool = storage.pool();
        let champion_repo = ChampionRepository::new(pool);

        champion_repo
            .upsert_champion(
                "champ-ahri",
                "ahri",
                "Ahri",
                "mid",
                &[String::from("pick")],
                true,
            )
            .await
            .expect("champion insert");

        for (patch, is_latest) in [("15.5", false), ("15.6", true)] {
            champion_repo
                .upsert_champion_version(&crate::storage::ChampionVersionRecord {
                    id: format!("ahri:{patch}"),
                    champion_id: "champ-ahri".into(),
                    patch: patch.into(),
                    source_path: format!(
                        "packages/champion-data/patches/{patch}/champions/ahri.json"
                    ),
                    payload_json: json!({"patch": patch}).to_string(),
                    compatible_since: patch.into(),
                    is_latest,
                    created_at: "2026-01-01 00:00:00".into(),
                })
                .await
                .expect("version insert");
        }

        let payload = champion_repo
            .resolve_patch_payload("ahri", "15.7")
            .await
            .expect("payload lookup")
            .expect("resolved payload");

        assert_eq!(payload.patch, "15.6");
        assert!(payload.incomplete_profile);
    }
}
