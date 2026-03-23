use serde::{Deserialize, Serialize};

use crate::{
    application::recommendations::analyze_composition,
    domain::{composition::CompositionSnapshot, draft::DraftState},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftReviewRequest {
    pub draft_state: DraftState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftReviewResponse {
    pub composition: CompositionSnapshot,
    pub win_condition: String,
    pub draft_call: String,
    pub review_notes: Vec<String>,
}

#[must_use]
pub fn review_draft(request: DraftReviewRequest) -> DraftReviewResponse {
    let composition = analyze_composition(&request.draft_state);
    let review_notes = build_review_notes(&request.draft_state, &composition);
    let win_condition = build_win_condition(&composition);
    let draft_call = build_draft_call(&composition);

    DraftReviewResponse {
        composition,
        win_condition,
        draft_call,
        review_notes,
    }
}

fn build_review_notes(draft_state: &DraftState, composition: &CompositionSnapshot) -> Vec<String> {
    let mut notes = vec![format!(
        "Patch {} : la draft alliée montre {} identité(s) clés.",
        draft_state.patch,
        composition.identities.len()
    )];

    if composition.profile.engage >= 3 {
        notes.push(
            "La draft peut dicter le tempo autour des objectifs grâce à plusieurs points d’entrée en teamfight."
                .into(),
        );
    }

    if composition.profile.scaling <= 1 {
        notes.push(
            "Le plan de jeu post-draft doit valoriser les fenêtres du mid game avant que l’adversaire ne scale."
                .into(),
        );
    }

    if !composition.alerts.is_empty() {
        notes.push(format!(
            "{} alerte(s) structurelle(s) restent présentes dans l’état final.",
            composition.alerts.len()
        ));
    }

    notes
}

fn build_win_condition(composition: &CompositionSnapshot) -> String {
    if composition.profile.front_to_back >= 3 {
        "Win condition principale : sécuriser la vision, forcer les 5v5 cadrés et jouer les objectifs avec une ligne avant disciplinée.".into()
    } else if composition.profile.pick >= 2 {
        "Win condition principale : utiliser les timings de vision pour trouver des catches puis convertir en objectifs neutres.".into()
    } else {
        "Win condition principale : jouer le tempo de lane et protéger les side waves jusqu’au prochain pic d’objets.".into()
    }
}

fn build_draft_call(composition: &CompositionSnapshot) -> String {
    if composition
        .alerts
        .iter()
        .any(|alert| alert.code == "missing_primary_engage")
    {
        "Review : composition jouable mais dépendante des setups de vision, avec peu d’outils pour forcer l’entrée en combat.".into()
    } else if composition.profile.scaling >= 3 {
        "Review : draft équilibrée, capable de temporiser puis d’imposer un front-to-back propre sur les objectifs majeurs.".into()
    } else {
        "Review : draft proactive qui doit conserver l’initiative avant les gros spikes adverses."
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        champion::{
            Champion, DamageProfile, ExecutionDemand, LanePattern, Role, RoleConfidenceMap,
        },
        composition::CompositionIdentity,
        draft::{DraftPhase, TeamDraft, TeamSide},
    };

    #[test]
    fn review_draft_generates_notes() {
        let mut role_confidence_map = RoleConfidenceMap::new();
        role_confidence_map.insert(Role::Top, 1.0);

        let champion = Champion {
            id: "ornn".into(),
            name: "Ornn".into(),
            roles: vec![Role::Top],
            role_confidence_map,
            identities: vec![
                CompositionIdentity::Engage,
                CompositionIdentity::FrontToBack,
            ],
            damage_profile: DamageProfile::Mixed,
            execution_demand: ExecutionDemand::Low,
            lane_pattern: LanePattern::Stable,
            crowd_control: 4,
            engage: 4,
            scaling: 4,
            durability: 4,
            mobility: 1,
        };

        let response = review_draft(DraftReviewRequest {
            draft_state: DraftState {
                patch: "15.6".into(),
                side: TeamSide::Blue,
                phase: DraftPhase::Complete,
                ally: TeamDraft {
                    champions: vec![champion],
                },
                enemy: TeamDraft { champions: vec![] },
                ally_bans: vec![],
                enemy_bans: vec![],
                contested_roles: vec![],
            },
        });

        assert!(!response.review_notes.is_empty());
    }
}
