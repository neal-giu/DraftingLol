# Formule de scoring et fallbacks d'évaluation

## Score final

Le score final est la somme pondérée de cinq sous-scores bornés entre `0` et `100`.

```text
final_score =
  internal_coherence * 0.35 +
  enemy_matchup * 0.30 +
  roster_fit * 0.20 +
  execution_simplicity * 0.10 +
  lane_stability * 0.05
```

## Dimensions

| Dimension | Poids | Intention |
| --- | --- | --- |
| `internal_coherence` | `0.35` | renforce ou répare le plan allié |
| `enemy_matchup` | `0.30` | mesure les risques/opportunités contre les picks révélés |
| `roster_fit` | `0.20` | couvre les rôles manquants et la flexibilité |
| `execution_simplicity` | `0.10` | préfère les plans lisibles et reproductibles |
| `lane_stability` | `0.05` | réduit la variance lane quand le matchup est connu |

## Contributeurs positifs et négatifs

Chaque sous-score conserve une liste de `ScoreContributor` avec :

- `dimension`
- `polarity` (`bonus` ou `malus`)
- `label`
- `value`
- `detail`

Cela permet :

- d'afficher un audit lisible côté UI ;
- de justifier le classement top 3 / top 5 ;
- d'alimenter le logger `scoring` sans réimprimer l'objet complet brut.

## Règles de calcul actuellement en place

### Cohérence interne

Bonus quand le candidat :

- partage une identité déjà présente ;
- ajoute une identité secondaire sans casser le plan ;
- apporte l'engage manquant ;
- améliore une courbe de scaling jugée trop faible.

### Matchup adverse

Bonus quand le candidat :

- absorbe correctement des dégâts mixtes ;
- profite d'adversaires pauvres en contrôle pour valoriser sa mobilité.

Malus quand le candidat :

- exige une exécution élevée contre beaucoup de contrôle adverse.

### Fit de roster

Bonus quand le candidat :

- couvre un rôle manquant avec confiance ;
- maintient un flex sur un rôle contesté.

Malus quand le candidat :

- duplique une composition déjà complète sans gain structurel net.

### Simplicité d'exécution

Le score de base dépend de `execution_demand` (`low > medium > high`) puis reçoit un bonus si le candidat renforce explicitement un plan `front_to_back` déjà lisible.

### Stabilité de lane

- bonus si le matchup direct n'est pas encore révélé ;
- bonus supplémentaire pour un profil de lane stable ;
- malus pour un pick volatil dans un matchup déjà montré.

## Fallbacks analytiques

Quand l'information exacte n'est pas disponible, le moteur doit rester déterministe :

- **patch partiellement connu** : utiliser le profil champion compatible le plus récent et marquer l'origine dans la chaîne d'import ;
- **draft live indisponible** : injecter un `DraftState` sandbox au lieu de retourner une erreur fatale ;
- **pool incomplet** : scorer uniquement les candidats disponibles, en conservant les alertes structurelles ;
- **données faibles sur l'exécution** : dégrader vers `medium` plutôt que d'inventer un score extrême.

## Journalisation `scoring`

Événements recommandés :

- `composition_analyzed`
- `candidate_scored`
- `top_candidates_ranked`
- `alert_generated`

Champs utiles :

- `candidate_id`
- `final_score`
- `dimension_scores`
- `alert_codes`
- `win_condition_preview`
- `payload_redacted`
