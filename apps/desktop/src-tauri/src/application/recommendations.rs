use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::domain::{
    champion::{Champion, DamageProfile, ExecutionDemand, Role},
    composition::{
        AlertCategory, AlertSeverity, CompositionIdentity, CompositionProfile, CompositionSnapshot,
        DraftAlert,
    },
    draft::DraftState,
    scoring::{
        explained_subscore, CandidateEvaluation, ContributionPolarity, ScoreBreakdown,
        ScoreContributor, ScoreDimension, ENEMY_MATCHUP_WEIGHT, EXECUTION_SIMPLICITY_WEIGHT,
        INTERNAL_COHERENCE_WEIGHT, LANE_STABILITY_WEIGHT, ROSTER_FIT_WEIGHT,
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub draft_state: DraftState,
    pub candidates: Vec<Champion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationResponse {
    pub composition: CompositionSnapshot,
    pub top_5: Vec<CandidateEvaluation>,
    pub evaluated_candidates: Vec<CandidateEvaluation>,
}

pub fn recommend_candidates(request: RecommendationRequest) -> RecommendationResponse {
    let composition = analyze_composition(&request.draft_state);
    let mut evaluated_candidates = request
        .candidates
        .into_iter()
        .map(|candidate| evaluate_candidate(&request.draft_state, &composition, candidate))
        .collect::<Vec<_>>();

    evaluated_candidates.sort_by(|left, right| {
        right
            .score_breakdown
            .final_score
            .total_cmp(&left.score_breakdown.final_score)
    });

    let top_5 = evaluated_candidates.iter().take(5).cloned().collect();

    RecommendationResponse {
        composition,
        top_5,
        evaluated_candidates,
    }
}

#[must_use]
pub fn analyze_composition(draft_state: &DraftState) -> CompositionSnapshot {
    let identities = collect_team_identities(&draft_state.ally.champions);
    let profile = build_profile(&draft_state.ally.champions);
    let alerts = build_structural_alerts(draft_state, &identities, &profile);

    CompositionSnapshot {
        identities,
        profile,
        alerts,
    }
}

#[must_use]
pub fn evaluate_candidate(
    draft_state: &DraftState,
    composition: &CompositionSnapshot,
    candidate: Champion,
) -> CandidateEvaluation {
    let coherence = score_internal_coherence(composition, &candidate);
    let matchup = score_enemy_matchup(draft_state, &candidate);
    let roster_fit = score_roster_fit(draft_state, &candidate);
    let execution = score_execution_simplicity(composition, &candidate);
    let lane_stability = score_lane_stability(draft_state, &candidate);
    let score_breakdown =
        ScoreBreakdown::from_subscores(coherence, matchup, roster_fit, execution, lane_stability);
    let alerts = extend_alerts_with_candidate(draft_state, composition, &candidate);
    let explanation = build_explanations(composition, &candidate, &score_breakdown, &alerts);
    let win_condition_after_pick = build_win_condition_after_pick(composition, &candidate);
    let draft_call = build_draft_call(&score_breakdown, &alerts);

    CandidateEvaluation {
        champion: candidate,
        score_breakdown,
        alerts,
        explanation,
        win_condition_after_pick,
        draft_call,
    }
}

fn collect_team_identities(champions: &[Champion]) -> Vec<CompositionIdentity> {
    let mut identities = BTreeSet::new();

    for champion in champions {
        for identity in &champion.identities {
            identities.insert(identity.clone());
        }
    }

    identities.into_iter().collect()
}

fn build_profile(champions: &[Champion]) -> CompositionProfile {
    let mut profile = CompositionProfile::default();

    for champion in champions {
        profile.engage += u8::from(champion.identities.contains(&CompositionIdentity::Engage));
        profile.disengage += u8::from(
            champion
                .identities
                .contains(&CompositionIdentity::ProtectCarry),
        );
        profile.pick += u8::from(champion.identities.contains(&CompositionIdentity::Pick));
        profile.poke += u8::from(champion.identities.contains(&CompositionIdentity::Poke));
        profile.front_to_back += u8::from(
            champion
                .identities
                .contains(&CompositionIdentity::FrontToBack),
        );
        profile.split_push += u8::from(
            champion
                .identities
                .contains(&CompositionIdentity::SplitPush),
        );
        profile.scaling += champion.scaling.min(5) / 2;
        profile.wave_clear += match champion.damage_profile {
            DamageProfile::Physical | DamageProfile::Magical | DamageProfile::Mixed => 1,
            DamageProfile::True => 0,
        };
    }

    profile.clamped()
}

fn build_structural_alerts(
    draft_state: &DraftState,
    identities: &[CompositionIdentity],
    profile: &CompositionProfile,
) -> Vec<DraftAlert> {
    let mut alerts = Vec::new();

    if !identities.contains(&CompositionIdentity::Engage) && draft_state.ally.champions.len() >= 3 {
        alerts.push(DraftAlert {
            category: AlertCategory::Structural,
            severity: AlertSeverity::Critical,
            code: "missing_primary_engage".into(),
            title: "Engage manquant".into(),
            detail:
                "La composition alliée n’a pas d’outil d’engage fiable pour lancer les combats."
                    .into(),
        });
    }

    if profile.scaling <= 1 && draft_state.phase != crate::domain::draft::DraftPhase::Complete {
        alerts.push(DraftAlert {
            category: AlertCategory::Structural,
            severity: AlertSeverity::Warning,
            code: "low_scaling_curve".into(),
            title: "Scaling faible".into(),
            detail: "La courbe de scaling actuelle pousse à terminer tôt ou à snowball la carte."
                .into(),
        });
    }

    if draft_state.missing_roles().len() >= 3 {
        alerts.push(DraftAlert {
            category: AlertCategory::Roster,
            severity: AlertSeverity::Info,
            code: "core_roles_open".into(),
            title: "Rôles encore ouverts".into(),
            detail: "Plusieurs rôles restent flexibles : la priorité doit rester la couverture de draft.".into(),
        });
    }

    alerts
}

fn score_internal_coherence(
    composition: &CompositionSnapshot,
    candidate: &Champion,
) -> crate::domain::scoring::ExplainedSubScore {
    let mut score = 50.0;
    let mut contributors = Vec::new();

    for identity in &candidate.identities {
        if composition.identities.contains(identity) {
            score += 9.0;
            contributors.push(contributor(
                ScoreDimension::InternalCoherence,
                ContributionPolarity::Bonus,
                format!("Identité partagée: {}", identity_label(identity)),
                9.0,
                "Renforce un plan de jeu déjà visible dans la draft alliée.",
            ));
        } else {
            score += 4.0;
            contributors.push(contributor(
                ScoreDimension::InternalCoherence,
                ContributionPolarity::Bonus,
                format!("Nouvelle identité: {}", identity_label(identity)),
                4.0,
                "Ajoute une option stratégique sans casser la cohérence globale.",
            ));
        }
    }

    if candidate.engage >= 3 && composition.profile.engage <= 1 {
        score += 14.0;
        contributors.push(contributor(
            ScoreDimension::InternalCoherence,
            ContributionPolarity::Bonus,
            "Apporte l’engage manquant".into(),
            14.0,
            "Corrige le principal trou structurel observé dans la composition alliée.",
        ));
    }

    if candidate.scaling >= 4 && composition.profile.scaling <= 2 {
        score += 8.0;
        contributors.push(contributor(
            ScoreDimension::InternalCoherence,
            ContributionPolarity::Bonus,
            "Stabilise la courbe de scaling".into(),
            8.0,
            "Ajoute une condition de victoire plus fiable pour les phases tardives.",
        ));
    }

    explained_subscore(score, INTERNAL_COHERENCE_WEIGHT, contributors)
}

fn score_enemy_matchup(
    draft_state: &DraftState,
    candidate: &Champion,
) -> crate::domain::scoring::ExplainedSubScore {
    let mut score = 50.0;
    let mut contributors = Vec::new();

    for enemy in &draft_state.enemy.champions {
        if candidate.durability >= 3 && enemy.damage_profile == DamageProfile::Mixed {
            score += 6.0;
            contributors.push(contributor(
                ScoreDimension::EnemyMatchup,
                ContributionPolarity::Bonus,
                format!("Résilience contre {}", enemy.name),
                6.0,
                "Le profil défensif du candidat absorbe bien une source de dégâts mixte.",
            ));
        }

        if candidate.mobility >= 3 && enemy.crowd_control <= 2 {
            score += 4.0;
            contributors.push(contributor(
                ScoreDimension::EnemyMatchup,
                ContributionPolarity::Bonus,
                format!("Fenêtres de mobilité contre {}", enemy.name),
                4.0,
                "La menace ennemie a peu d’outils pour punir les repositionnements.",
            ));
        }

        if candidate.execution_demand == ExecutionDemand::High && enemy.crowd_control >= 4 {
            score -= 6.0;
            contributors.push(contributor(
                ScoreDimension::EnemyMatchup,
                ContributionPolarity::Malus,
                format!("Exécution punissable face à {}", enemy.name),
                -6.0,
                "Le contrôle adverse complique les séquences mécaniques exigeantes.",
            ));
        }
    }

    explained_subscore(score, ENEMY_MATCHUP_WEIGHT, contributors)
}

fn score_roster_fit(
    draft_state: &DraftState,
    candidate: &Champion,
) -> crate::domain::scoring::ExplainedSubScore {
    let mut score = 45.0;
    let mut contributors = Vec::new();
    let missing_roles = draft_state.missing_roles();

    for role in &missing_roles {
        let confidence = candidate.role_confidence(role) * 30.0;
        if confidence > 0.0 {
            score += confidence;
            contributors.push(contributor(
                ScoreDimension::RosterFit,
                ContributionPolarity::Bonus,
                format!("Couvre le rôle {:?}", role).to_lowercase(),
                confidence,
                "La carte de confiance de rôle indique une couverture crédible pour ce slot.",
            ));
        }
    }

    for role in &draft_state.contested_roles {
        let confidence = candidate.role_confidence(role);
        if confidence >= 0.5 {
            score += 8.0;
            contributors.push(contributor(
                ScoreDimension::RosterFit,
                ContributionPolarity::Bonus,
                format!("Flex sur le rôle {:?}", role).to_lowercase(),
                8.0,
                "Peut maintenir l’incertitude de draft sur un rôle contesté.",
            ));
        }
    }

    if missing_roles.is_empty() {
        score -= 10.0;
        contributors.push(contributor(
            ScoreDimension::RosterFit,
            ContributionPolarity::Malus,
            "Redondance de rôle".into(),
            -10.0,
            "La composition a déjà couvert ses rôles principaux ; le pick ajoute surtout de la duplication.",
        ));
    }

    explained_subscore(score, ROSTER_FIT_WEIGHT, contributors)
}

fn score_execution_simplicity(
    composition: &CompositionSnapshot,
    candidate: &Champion,
) -> crate::domain::scoring::ExplainedSubScore {
    let mut score = match candidate.execution_demand {
        ExecutionDemand::Low => 78.0,
        ExecutionDemand::Medium => 62.0,
        ExecutionDemand::High => 46.0,
    };
    let mut contributors = vec![contributor(
        ScoreDimension::ExecutionSimplicity,
        if candidate.execution_demand == ExecutionDemand::High {
            ContributionPolarity::Malus
        } else {
            ContributionPolarity::Bonus
        },
        "Charge d’exécution".into(),
        match candidate.execution_demand {
            ExecutionDemand::Low => 12.0,
            ExecutionDemand::Medium => 4.0,
            ExecutionDemand::High => -10.0,
        },
        "Mesure la difficulté à reproduire le plan de jeu sous pression.",
    )];

    if composition
        .identities
        .contains(&CompositionIdentity::FrontToBack)
        && candidate
            .identities
            .contains(&CompositionIdentity::FrontToBack)
    {
        score += 10.0;
        contributors.push(contributor(
            ScoreDimension::ExecutionSimplicity,
            ContributionPolarity::Bonus,
            "Plan de teamfight lisible".into(),
            10.0,
            "Le candidat s’insère dans un schéma de combat simple à exécuter pour l’équipe.",
        ));
    }

    explained_subscore(score, EXECUTION_SIMPLICITY_WEIGHT, contributors)
}

fn score_lane_stability(
    draft_state: &DraftState,
    candidate: &Champion,
) -> crate::domain::scoring::ExplainedSubScore {
    let mut score = 55.0;
    let mut contributors = Vec::new();

    if let Some(primary_role) = candidate.primary_role() {
        let lane_threats = draft_state
            .enemy
            .champions
            .iter()
            .filter(|enemy| enemy.primary_role().as_ref() == Some(&primary_role))
            .count();

        if lane_threats == 0 {
            score += 8.0;
            contributors.push(contributor(
                ScoreDimension::LaneStability,
                ContributionPolarity::Bonus,
                "Matchup encore caché".into(),
                8.0,
                "L’adversaire n’a pas encore révélé le matchup direct, ce qui réduit le risque immédiat.",
            ));
        } else if candidate.lane_pattern == crate::domain::champion::LanePattern::Stable {
            score += 6.0;
            contributors.push(contributor(
                ScoreDimension::LaneStability,
                ContributionPolarity::Bonus,
                "Profil de lane stable".into(),
                6.0,
                "Le champion limite les états de lane perdants même sous pression.",
            ));
        } else {
            score -= 4.0;
            contributors.push(contributor(
                ScoreDimension::LaneStability,
                ContributionPolarity::Malus,
                "Lane volatile".into(),
                -4.0,
                "Le pick accepte une variance plus forte face à un matchup déjà montré.",
            ));
        }
    }

    explained_subscore(score, LANE_STABILITY_WEIGHT, contributors)
}

fn extend_alerts_with_candidate(
    draft_state: &DraftState,
    composition: &CompositionSnapshot,
    candidate: &Champion,
) -> Vec<DraftAlert> {
    let mut alerts = composition.alerts.clone();

    if candidate.execution_demand == ExecutionDemand::High && composition.profile.engage == 0 {
        alerts.push(DraftAlert {
            category: AlertCategory::Structural,
            severity: AlertSeverity::Warning,
            code: "high_execution_without_setup".into(),
            title: "Pick difficile à préparer".into(),
            detail: format!(
                "{} demande beaucoup d’exécution alors que la composition crée peu de setup fiable.",
                candidate.name
            ),
        });
    }

    if draft_state
        .enemy
        .champions
        .iter()
        .any(|enemy| enemy.crowd_control >= 4)
        && candidate.mobility <= 1
    {
        alerts.push(DraftAlert {
            category: AlertCategory::Matchup,
            severity: AlertSeverity::Warning,
            code: "low_mobility_into_cc".into(),
            title: "Mobilité exposée au contrôle".into(),
            detail: format!(
                "{} risque de subir les outils de catch adverses s’il rate son placement initial.",
                candidate.name
            ),
        });
    }

    if draft_state.missing_roles().is_empty() && candidate.role_confidence(&Role::Support) < 0.3 {
        alerts.push(DraftAlert {
            category: AlertCategory::Roster,
            severity: AlertSeverity::Info,
            code: "luxury_pick".into(),
            title: "Pick de confort".into(),
            detail: "La composition est déjà complète ; le pick doit justifier sa valeur par le matchup ou la synergie.".into(),
        });
    }

    alerts
}

fn build_explanations(
    composition: &CompositionSnapshot,
    candidate: &Champion,
    breakdown: &ScoreBreakdown,
    alerts: &[DraftAlert],
) -> Vec<String> {
    let mut explanations = vec![format!(
        "{} atteint {:.1}/100 grâce à une cohérence interne de {:.1} et un matchup adverse de {:.1}.",
        candidate.name,
        breakdown.final_score,
        breakdown.internal_coherence.raw_score,
        breakdown.enemy_matchup.raw_score
    )];

    if composition
        .identities
        .iter()
        .any(|identity| candidate.identities.contains(identity))
    {
        explanations.push(
            "Le candidat prolonge un axe de composition déjà visible, ce qui simplifie la lecture du plan de jeu."
                .into(),
        );
    }

    if !alerts.is_empty() {
        explanations.push(format!(
            "{} alerte(s) restent à surveiller après ce pick.",
            alerts.len()
        ));
    }

    explanations
}

fn build_win_condition_after_pick(
    composition: &CompositionSnapshot,
    candidate: &Champion,
) -> String {
    if candidate
        .identities
        .contains(&CompositionIdentity::ProtectCarry)
        || composition
            .identities
            .contains(&CompositionIdentity::ProtectCarry)
    {
        format!(
            "Jouer front-to-back autour de {} et forcer les objectifs quand la ligne avant verrouille l’espace.",
            candidate.name
        )
    } else if candidate.identities.contains(&CompositionIdentity::Pick) {
        format!(
            "Créer des surnombres avec {} sur les timings de vision, puis convertir en Nashor ou tours extérieures.",
            candidate.name
        )
    } else {
        format!(
            "Utiliser {} pour accélérer le tempo de combat et prendre l’initiative sur les objectifs neutres.",
            candidate.name
        )
    }
}

fn build_draft_call(score_breakdown: &ScoreBreakdown, alerts: &[DraftAlert]) -> String {
    let critical_alerts = alerts
        .iter()
        .filter(|alert| alert.severity == AlertSeverity::Critical)
        .count();

    if score_breakdown.final_score >= 72.0 && critical_alerts == 0 {
        "Blindable et prioritaire si la draft doit verrouiller un power pick maintenant.".into()
    } else if score_breakdown.final_score >= 60.0 {
        "Bon pick de contexte : à prendre si la priorité est la couverture de rôle ou la stabilité de plan.".into()
    } else {
        "Pick situationnel : garder en réserve si une meilleure réponse de matchup ou de structure reste disponible.".into()
    }
}

fn contributor(
    dimension: ScoreDimension,
    polarity: ContributionPolarity,
    label: String,
    value: f32,
    detail: &str,
) -> ScoreContributor {
    ScoreContributor {
        dimension,
        polarity,
        label,
        value,
        detail: detail.into(),
    }
}

fn identity_label(identity: &CompositionIdentity) -> &'static str {
    match identity {
        CompositionIdentity::Engage => "engage",
        CompositionIdentity::Pick => "pick",
        CompositionIdentity::Poke => "poke",
        CompositionIdentity::Siege => "siege",
        CompositionIdentity::FrontToBack => "front_to_back",
        CompositionIdentity::SplitPush => "split_push",
        CompositionIdentity::Skirmish => "skirmish",
        CompositionIdentity::ProtectCarry => "protect_carry",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        champion::{LanePattern, RoleConfidenceMap},
        draft::{DraftPhase, TeamDraft, TeamSide},
    };

    fn champion(
        name: &str,
        role: Role,
        confidence: f32,
        identities: Vec<CompositionIdentity>,
    ) -> Champion {
        let mut role_confidence_map = RoleConfidenceMap::new();
        role_confidence_map.insert(role.clone(), confidence);

        Champion {
            id: name.to_lowercase(),
            name: name.into(),
            roles: vec![role],
            role_confidence_map,
            identities,
            damage_profile: DamageProfile::Mixed,
            execution_demand: ExecutionDemand::Medium,
            lane_pattern: LanePattern::Stable,
            crowd_control: 3,
            engage: 3,
            scaling: 3,
            durability: 3,
            mobility: 2,
        }
    }

    fn draft_state() -> DraftState {
        DraftState {
            patch: "15.6".into(),
            side: TeamSide::Blue,
            phase: DraftPhase::PickPhaseOne,
            ally: TeamDraft {
                champions: vec![champion(
                    "Sejuani",
                    Role::Jungle,
                    1.0,
                    vec![CompositionIdentity::Engage],
                )],
            },
            enemy: TeamDraft {
                champions: vec![champion(
                    "Ahri",
                    Role::Mid,
                    1.0,
                    vec![CompositionIdentity::Pick],
                )],
            },
            ally_bans: vec![],
            enemy_bans: vec![],
            contested_roles: vec![Role::Top],
        }
    }

    #[test]
    fn recommend_candidates_returns_top_five_sorted() {
        let request = RecommendationRequest {
            draft_state: draft_state(),
            candidates: vec![
                champion(
                    "Ornn",
                    Role::Top,
                    0.9,
                    vec![
                        CompositionIdentity::Engage,
                        CompositionIdentity::FrontToBack,
                    ],
                ),
                champion(
                    "Jayce",
                    Role::Top,
                    0.9,
                    vec![CompositionIdentity::Poke, CompositionIdentity::Siege],
                ),
                champion("Gnar", Role::Top, 0.9, vec![CompositionIdentity::Skirmish]),
                champion(
                    "Renekton",
                    Role::Top,
                    0.9,
                    vec![CompositionIdentity::Skirmish],
                ),
                champion("Kennen", Role::Top, 0.9, vec![CompositionIdentity::Engage]),
                champion(
                    "Camille",
                    Role::Top,
                    0.9,
                    vec![CompositionIdentity::Pick, CompositionIdentity::SplitPush],
                ),
            ],
        };

        let response = recommend_candidates(request);

        assert_eq!(response.top_5.len(), 5);
        assert!(
            response.evaluated_candidates[0].score_breakdown.final_score
                >= response.evaluated_candidates[1].score_breakdown.final_score
        );
    }

    #[test]
    fn analyze_composition_surfaces_structural_alerts() {
        let draft_state = DraftState {
            ally: TeamDraft {
                champions: vec![champion(
                    "Ezreal",
                    Role::Bottom,
                    1.0,
                    vec![CompositionIdentity::Poke],
                )],
            },
            ..draft_state()
        };

        let snapshot = analyze_composition(&draft_state);

        assert!(snapshot
            .alerts
            .iter()
            .any(|alert| alert.code == "core_roles_open"));
    }
}
