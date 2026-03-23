# Formalisation du modèle métier et des identités de composition

## Agrégats principaux

### `Champion`

Attributs métier minimaux :

- `id`, `name`
- `roles`, `role_confidence_map`
- `identities`
- `damage_profile`
- `execution_demand`
- `lane_pattern`
- `crowd_control`, `engage`, `scaling`, `durability`, `mobility`

Le champion est l'unité de base utilisée à la fois pour la draft courante, le pool candidat et l'historique.

### `DraftState`

`DraftState` est la représentation canonique d'une draft :

- contexte patch (`patch`)
- côté (`side`)
- phase (`phase`)
- picks alliés / ennemis
- bans alliés / ennemis
- rôles contestés

La méthode `missing_roles()` formalise le besoin de couverture de rôle restant.

### `CompositionSnapshot`

Une composition est projetée en trois vues complémentaires :

1. `identities` : intentions stratégiques détectées ;
2. `profile` : intensité agrégée par axes lisibles ;
3. `alerts` : trous structurels, de matchup ou de roster.

### `CandidateEvaluation`

Chaque candidat produit :

- un `ScoreBreakdown` pondéré et explicable ;
- des alertes héritées ou nouvelles ;
- une explication textuelle ;
- une `win_condition_after_pick` ;
- un `draft_call` exploitable en coaching.

## Identités de composition

Les identités actuellement formalisées sont :

| Identité | Signification | Signal type |
| --- | --- | --- |
| `engage` | capacité à démarrer un fight de façon fiable | initiation, contrôle fort |
| `pick` | création de catches et surnombres | mobilité, CC ponctuel, vision |
| `poke` | usure avant engagement | portée, pression à distance |
| `siege` | pression sur structures / zones sans all-in immédiat | wave clear, portée, setup tourelles |
| `front_to_back` | teamfight cadré derrière une ligne avant | frontline, DPS soutenu, discipline |
| `split_push` | séparation des ressources et side lane | duel, mobilité latérale, tempo map |
| `skirmish` | combats courts et répétés | timing river/jungle, burst local |
| `protect_carry` | peel et protection d'une source principale de DPS | disengage, zone denial, utilitaire |

## Profil de composition

Le `CompositionProfile` agrège huit intensités bornées entre `0` et `5` :

- `engage`
- `disengage`
- `pick`
- `poke`
- `front_to_back`
- `split_push`
- `scaling`
- `wave_clear`

La méthode `clamped()` impose la borne supérieure pour conserver un profil comparable entre drafts incomplètes et complètes.

## Alertes métier

| Catégorie | Usage |
| --- | --- |
| `structural` | trou de plan de jeu ou outil obligatoire manquant |
| `matchup` | exposition particulière face à la draft adverse |
| `roster` | couverture de rôle, flexibilité, redondance |

| Sévérité | Usage |
| --- | --- |
| `info` | à surveiller, sans urgence immédiate |
| `warning` | risque clair mais compensable |
| `critical` | défaut central qui doit influencer la recommandation |

## Règles de lecture métier

- une identité n'est pas une compo complète : elle signale seulement un axe plausible ;
- plusieurs identités peuvent coexister si elles ne se contredisent pas ;
- les alertes servent à expliquer les limites du modèle autant que les défauts de draft ;
- la cohérence est évaluée à la fois sur le plan de jeu, la couverture de rôle et le coût d'exécution.
