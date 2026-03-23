# Scénarios de test minimaux

Les scénarios ci-dessous sont stockés sous forme de fixtures Rust dans `apps/desktop/src-tauri/src/tests/scenario_tests.rs`.

## Couverture minimale

1. `protect_hypercarry`
2. `plan_flou`
3. `double_tank_sans_tank_shred`
4. `zone_control_setup_objectif`
5. `split_qui_casse_la_coherence`
6. `engage_vs_poke`
7. `anti_dive_scaling`
8. `pick_comp_sans_follow`
9. `front_to_back`
10. `comp_trop_difficile_a_executer`

## Contrat de fixture

Chaque scénario stocke au minimum :

- l'entrée `draft_state` ;
- le `candidate_pool` ;
- un objet `preferences` sérialisable ;
- le top 3 attendu ;
- les codes d'alertes attendus ;
- les identités attendues ;
- un fragment de win condition attendu.

## Lecture attendue

Ces tests ne cherchent pas à rejouer un match réel complet. Ils garantissent plutôt que le moteur garde une lecture cohérente et explicable pour des situations de draft considérées comme indispensables au produit.

## Logs de test

Les exécutions de scénarios sont de bons candidats pour alimenter les loggers structurés :

- `app` pour le démarrage/fin de suite ;
- `scoring` pour le ranking attendu vs obtenu ;
- `draft_watch` pour les bascules live/sandbox simulées ;
- `riot_adapter` uniquement pour des tests d'intégration live, jamais pour injecter des secrets réseau ;
- `db` si les fixtures sont un jour persistées pour relecture historique.
