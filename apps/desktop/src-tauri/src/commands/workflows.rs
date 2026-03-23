use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    application::{
        recommendations::{recommend_candidates, RecommendationRequest, RecommendationResponse},
        review::{review_draft, DraftReviewRequest, DraftReviewResponse},
    },
    domain::{
        champion::{Champion, DamageProfile, ExecutionDemand, LanePattern, Role},
        composition::CompositionIdentity,
        draft::{DraftPhase, DraftState, TeamDraft, TeamSide},
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftStatePayload {
    pub draft_state: DraftState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SandboxScenario {
    pub id: String,
    pub label: String,
    pub pick: String,
    pub projected_score: f32,
    pub summary: String,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SandboxSimulationResponse {
    pub baseline: RecommendationResponse,
    pub scenarios: Vec<SandboxScenario>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub patch: String,
    pub opponent: String,
    pub result: String,
    pub review_headline: String,
    pub draft_call: String,
}

fn sample_candidates() -> Vec<Champion> {
    vec![
        champion(
            "nautilus",
            "Nautilus",
            vec![Role::Support],
            vec![CompositionIdentity::Engage, CompositionIdentity::Pick],
            DamageProfile::Magical,
            ExecutionDemand::Low,
            LanePattern::Utility,
            5,
            5,
            2,
            4,
            1,
        ),
        champion(
            "renata",
            "Renata Glasc",
            vec![Role::Support],
            vec![CompositionIdentity::ProtectCarry, CompositionIdentity::Pick],
            DamageProfile::Magical,
            ExecutionDemand::Medium,
            LanePattern::Utility,
            3,
            1,
            3,
            2,
            1,
        ),
        champion(
            "jax",
            "Jax",
            vec![Role::Top],
            vec![CompositionIdentity::SplitPush, CompositionIdentity::Skirmish],
            DamageProfile::Mixed,
            ExecutionDemand::Medium,
            LanePattern::Scaling,
            2,
            2,
            4,
            3,
            3,
        ),
    ]
}

fn champion(
    id: &str,
    name: &str,
    roles: Vec<Role>,
    identities: Vec<CompositionIdentity>,
    damage_profile: DamageProfile,
    execution_demand: ExecutionDemand,
    lane_pattern: LanePattern,
    crowd_control: u8,
    engage: u8,
    scaling: u8,
    durability: u8,
    mobility: u8,
) -> Champion {
    let role_confidence_map = roles
        .iter()
        .cloned()
        .map(|role| (role, 1.0_f32))
        .collect::<BTreeMap<_, _>>();

    Champion {
        id: id.into(),
        name: name.into(),
        roles,
        role_confidence_map,
        identities,
        damage_profile,
        execution_demand,
        lane_pattern,
        crowd_control,
        engage,
        scaling,
        durability,
        mobility,
    }
}

fn default_draft_state() -> DraftState {
    DraftState {
        patch: "15.6".into(),
        side: TeamSide::Blue,
        phase: DraftPhase::PickPhaseTwo,
        ally: TeamDraft {
            champions: vec![
                champion(
                    "ornn",
                    "Ornn",
                    vec![Role::Top],
                    vec![CompositionIdentity::Engage, CompositionIdentity::FrontToBack],
                    DamageProfile::Mixed,
                    ExecutionDemand::Low,
                    LanePattern::Stable,
                    4,
                    5,
                    4,
                    5,
                    1,
                ),
                champion(
                    "sejuani",
                    "Sejuani",
                    vec![Role::Jungle],
                    vec![
                        CompositionIdentity::Engage,
                        CompositionIdentity::FrontToBack,
                        CompositionIdentity::Pick,
                    ],
                    DamageProfile::Mixed,
                    ExecutionDemand::Low,
                    LanePattern::Utility,
                    5,
                    5,
                    3,
                    4,
                    2,
                ),
                champion(
                    "ahri",
                    "Ahri",
                    vec![Role::Mid],
                    vec![CompositionIdentity::Pick, CompositionIdentity::Skirmish],
                    DamageProfile::Magical,
                    ExecutionDemand::Medium,
                    LanePattern::Roaming,
                    3,
                    2,
                    3,
                    1,
                    4,
                ),
                champion(
                    "smolder",
                    "Smolder",
                    vec![Role::Bottom],
                    vec![CompositionIdentity::FrontToBack, CompositionIdentity::ProtectCarry],
                    DamageProfile::Physical,
                    ExecutionDemand::Medium,
                    LanePattern::Scaling,
                    1,
                    0,
                    5,
                    1,
                    2,
                ),
            ],
        },
        enemy: TeamDraft {
            champions: vec![
                champion(
                    "gnar",
                    "Gnar",
                    vec![Role::Top],
                    vec![CompositionIdentity::Poke, CompositionIdentity::FrontToBack],
                    DamageProfile::Physical,
                    ExecutionDemand::Medium,
                    LanePattern::Bully,
                    3,
                    2,
                    3,
                    3,
                    3,
                ),
                champion(
                    "xin-zhao",
                    "Xin Zhao",
                    vec![Role::Jungle],
                    vec![CompositionIdentity::Engage, CompositionIdentity::Skirmish],
                    DamageProfile::Physical,
                    ExecutionDemand::Low,
                    LanePattern::Roaming,
                    2,
                    4,
                    2,
                    3,
                    3,
                ),
                champion(
                    "azir",
                    "Azir",
                    vec![Role::Mid],
                    vec![CompositionIdentity::Poke, CompositionIdentity::FrontToBack],
                    DamageProfile::Magical,
                    ExecutionDemand::High,
                    LanePattern::Scaling,
                    2,
                    1,
                    5,
                    1,
                    2,
                ),
            ],
        },
        ally_bans: vec!["kalista".into(), "corki".into(), "skarner".into()],
        enemy_bans: vec!["rell".into(), "vi".into(), "orianna".into()],
        contested_roles: vec![Role::Support],
    }
}

#[tauri::command]
pub fn get_live_draft_recommendations(
    payload: Option<DraftStatePayload>,
) -> RecommendationResponse {
    let draft_state = payload.map_or_else(default_draft_state, |inner| inner.draft_state);

    recommend_candidates(RecommendationRequest {
        draft_state,
        candidates: sample_candidates(),
    })
}

#[tauri::command]
pub fn get_draft_diagnostics(payload: Option<DraftStatePayload>) -> DraftReviewResponse {
    let draft_state = payload.map_or_else(default_draft_state, |inner| inner.draft_state);

    review_draft(DraftReviewRequest { draft_state })
}

#[tauri::command]
pub fn run_sandbox_simulation(payload: Option<DraftStatePayload>) -> SandboxSimulationResponse {
    let draft_state = payload.map_or_else(default_draft_state, |inner| inner.draft_state);
    let baseline = recommend_candidates(RecommendationRequest {
        draft_state,
        candidates: sample_candidates(),
    });

    let scenarios = baseline
        .top_5
        .iter()
        .take(3)
        .enumerate()
        .map(|(index, evaluation)| SandboxScenario {
            id: format!("scenario-{}", index + 1),
            label: match index {
                0 => "Safe engage".into(),
                1 => "Protect carry".into(),
                _ => "Macro split".into(),
            },
            pick: evaluation.champion.name.clone(),
            projected_score: evaluation.score_breakdown.final_score,
            summary: evaluation.win_condition_after_pick.clone(),
            risks: evaluation
                .alerts
                .iter()
                .take(2)
                .map(|alert| alert.title.clone())
                .collect(),
        })
        .collect();

    SandboxSimulationResponse { baseline, scenarios }
}

#[tauri::command]
pub fn load_history_reviews() -> Vec<HistoryEntry> {
    vec![
        HistoryEntry {
            id: "review-1".into(),
            patch: "15.6".into(),
            opponent: "Team Solaris".into(),
            result: "win".into(),
            review_headline: "Draft gagnante grâce à une identity engage/peel cohérente.".into(),
            draft_call: "Front-to-back propre, objectif contrôlé au troisième dragon.".into(),
        },
        HistoryEntry {
            id: "review-2".into(),
            patch: "15.5".into(),
            opponent: "Night Owls".into(),
            result: "loss".into(),
            review_headline: "Comp trop exigeante mécaniquement pour une exécution sur scène.".into(),
            draft_call: "Le split push n’a jamais trouvé de timings propres.".into(),
        },
    ]
}
