# Product model and operating modes

## Produit

Le backend Tauri modélise un assistant de draft League of Legends centré sur cinq capacités métier :

1. **lire l'état de draft** depuis une session live Riot ou depuis un scénario sandbox ;
2. **qualifier la composition alliée** par identités, profil de puissance et alertes structurelles ;
3. **noter un pool de candidats** à partir d'un scoring explicable ;
4. **conserver l'historique** des recommandations, reviews finales et préférences d'équipe en SQLite ;
5. **exposer des réponses stables** aux commandes desktop et aux workflows de simulation.

## Modèle produit

| Agrégat | Rôle produit | Références de code |
| --- | --- | --- |
| `DraftState` | photographie canonique de la draft courante | `src/domain/draft.rs` |
| `CompositionSnapshot` | synthèse des identités, du profil et des alertes de compo | `src/domain/composition.rs`, `src/application/recommendations.rs` |
| `CandidateEvaluation` | explication détaillée d'un pick candidat | `src/domain/scoring.rs`, `src/application/recommendations.rs` |
| `DraftReviewResponse` | review macro après verrouillage de la draft | `src/application/review.rs` |
| `DraftSessionHistory` | historique persistant d'une session complète | `src/storage/models.rs` |

## Modes d'exécution : live, patch et sandbox

### Chaîne live

1. `riot_adapter` lit la session de champ select.
2. le patch courant est résolu ;
3. le transformeur convertit la session Riot vers `DraftState` ;
4. `draft_watch` détecte les transitions (début, changement de phase, picks, bans, fin) ;
5. `app` déclenche recommendation/review et sérialise la réponse.

### Fallbacks patch/live

Le moniteur passe en mode sandbox quand une lecture live devient indisponible. Les raisons formalisées sont :

- `client_absent`
- `lockfile_absent`
- `lockfile_inaccessible`
- `local_ssl_failed`
- `endpoint_changed`
- `patch_unknown`
- `request_failed`

Le comportement attendu est le suivant :

| Situation | Décision produit | Effet observable |
| --- | --- | --- |
| session live lisible + patch résolu | rester en live | `availability = live` |
| session live lisible mais patch inconnu | fallback sandbox | `availability = sandbox`, raison `patch_unknown` |
| client Riot absent/inaccessible | fallback sandbox | la UI reste testable sans couper les workflows |
| fin de session live | remettre le watcher en idle | aucun snapshot actif |

### Contrat sandbox

Le sandbox doit toujours retourner un `DraftState` valide, sérialisable et suffisant pour exécuter les mêmes fonctions de scoring que le live. Il sert à la fois :

- d'environnement de démo ;
- de garde-fou produit lors d'une panne locale Riot ;
- de support pour les tests de scénarios.

## Journalisation structurée

Les loggers suivants sont réservés et doivent partager un format commun : `timestamp`, `level`, `logger`, `event`, `draft_session_id?`, `patch?`, `side?`, `team_id?`, `error_code?`, `duration_ms?`, `payload_redacted?`.

| Logger | Portée | Exemples d'événements |
| --- | --- | --- |
| `app` | orchestration globale, commandes Tauri, démarrage/arrêt | `command_started`, `command_completed`, `healthcheck_failed` |
| `riot_adapter` | lecture lockfile, handshake SSL local, appels Riot | `lockfile_read`, `session_fetch_failed`, `patch_resolved` |
| `draft_watch` | polling, transitions de session, diffusion des snapshots | `poll_tick`, `session_started`, `pick_changed`, `fallback_to_sandbox` |
| `scoring` | scoring détaillé, alertes générées, top picks | `candidate_scored`, `composition_analyzed`, `critical_alert_emitted` |
| `db` | migrations, lectures/écritures SQLite, import/export | `migration_applied`, `query_completed`, `transaction_rolled_back` |

## Non-divulgation des secrets réseau dans les logs

Règles minimales :

- ne jamais logger le contenu brut du lockfile Riot ;
- ne jamais logger mots de passe, tokens, cookies, entêtes d'authentification, certificats ou chemins contenant des secrets ;
- remplacer toute valeur sensible par une forme redacted (`***`, hash court, ou booléen de présence) ;
- préférer des codes d'erreur métiers (`lockfile_inaccessible`, `request_failed`) à des dumps réseau complets ;
- si un payload doit être tracé pour débogage, ne conserver que les champs non sensibles et ajouter `payload_redacted = true`.

## Convention de décision produit

- une recommandation doit rester **explicable** ;
- une alerte critique doit être visible même si le score final reste acceptable ;
- le fallback sandbox ne doit pas masquer la raison source du basculement ;
- la review finale doit reformuler la win condition en langage coaching plutôt qu'en simple score.
