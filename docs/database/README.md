# Schéma SQLite et responsabilités de persistance

## Principes

Le stockage SQLite sert de source locale de vérité pour :

- la configuration d'équipe ;
- les pools champions individuels ;
- les préférences de patch ;
- l'historique détaillé de draft et de recommandations ;
- les réglages applicatifs.

La connexion est ouverte en SQLite WAL avec `foreign_keys = ON` et migrations automatiques au démarrage.

## Tables

| Table | Rôle | Clés / contraintes notables |
| --- | --- | --- |
| `teams` | équipe suivie par l'application | `id` PK |
| `players` | roster d'une équipe | FK `team_id`, cascade delete |
| `champions` | catalogue canonique | `slug` unique |
| `champion_versions` | payloads par patch et compatibilité | FK `champion_id`, unique `(champion_id, patch)` |
| `player_champion_pools` | pool joueur par patch | FK `player_id`, FK `champion_id`, unique `(player_id, champion_id, patch)` |
| `team_preferences` | préférences d'équipe par patch | FK `team_id`, unique `(team_id, patch)` |
| `draft_sessions` | session live ou sandbox | FK optionnelle `team_id` |
| `draft_events` | timeline de bans/picks/phases | FK `session_id`, unique `(session_id, sequence)` |
| `draft_recommendations` | snapshot de ranking pour une session | FK `session_id`, FK candidate |
| `draft_final_reviews` | review finale unique par session | FK `session_id` unique |
| `app_settings` | réglages clé/valeur JSON | `key` PK |

## Relations

```text
teams 1---n players 1---n player_champion_pools n---1 champions
teams 1---n team_preferences
teams 0..1---n draft_sessions 1---n draft_events
                           \---n draft_recommendations
                           \---1 draft_final_reviews
champions 1---n champion_versions
```

## Mapping code <-> SQL

| SQL | Modèle Rust |
| --- | --- |
| `teams` | `TeamRecord` |
| `players` | `PlayerRecord` |
| `champions` | `ChampionRecord` |
| `champion_versions` | `ChampionVersionRecord` |
| `player_champion_pools` | `PlayerChampionPoolRecord` |
| `team_preferences` | `TeamPreferenceRecord` |
| `draft_sessions` | `DraftSessionRecord` |
| `draft_events` | `DraftEventRecord` |
| `draft_recommendations` | `DraftRecommendationRecord` |
| `draft_final_reviews` | `DraftFinalReviewRecord` |
| `app_settings` | `AppSettingRecord` |

## Invariants recommandés

- un `draft_final_review` existe au plus une fois par session ;
- les recommandations conservent un `ranking` stable pour rejouer un top N historique ;
- `reasoning_json`, `review_json` et `metadata_json` doivent rester sérialisables sans secrets réseau ;
- toute donnée importée par patch doit référencer une version champion compatible.

## Journalisation `db`

Événements recommandés :

- `migration_started` / `migration_applied`
- `repository_upsert_completed`
- `history_loaded`
- `export_bundle_created`
- `transaction_rolled_back`

Bonnes pratiques de log :

- logguer les identifiants fonctionnels (`team_id`, `session_id`, `patch`) plutôt que les blobs JSON complets ;
- tronquer ou omettre `preferences_json`, `reasoning_json`, `review_json` si un dump est demandé ;
- ne jamais logger des URLs locales Riot contenant des secrets temporaires.
