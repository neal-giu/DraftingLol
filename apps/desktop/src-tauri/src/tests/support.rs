use std::collections::BTreeMap;

use serde_json::Value;

use crate::{
    application::recommendations::RecommendationRequest,
    domain::{
        champion::{Champion, DamageProfile, ExecutionDemand, LanePattern, Role},
        composition::CompositionIdentity,
        draft::{DraftPhase, DraftState, TeamDraft, TeamSide},
    },
};

#[derive(Debug, Clone)]
pub struct ScenarioFixture {
    pub name: &'static str,
    pub request: RecommendationRequest,
    pub preferences: Value,
    pub expected_top_3: [&'static str; 3],
    pub expected_alert_codes: &'static [&'static str],
    pub expected_identities: &'static [CompositionIdentity],
    pub expected_win_condition_fragment: &'static str,
}

#[derive(Debug, Clone)]
pub struct ChampionSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub roles: Vec<Role>,
    pub identities: Vec<CompositionIdentity>,
    pub damage_profile: DamageProfile,
    pub execution_demand: ExecutionDemand,
    pub lane_pattern: LanePattern,
    pub crowd_control: u8,
    pub engage: u8,
    pub scaling: u8,
    pub durability: u8,
    pub mobility: u8,
}

pub fn champion(spec: ChampionSpec) -> Champion {
    let role_confidence_map = spec
        .roles
        .iter()
        .cloned()
        .map(|role| (role, 1.0_f32))
        .collect::<BTreeMap<_, _>>();

    Champion {
        id: spec.id.into(),
        name: spec.name.into(),
        roles: spec.roles,
        role_confidence_map,
        identities: spec.identities,
        damage_profile: spec.damage_profile,
        execution_demand: spec.execution_demand,
        lane_pattern: spec.lane_pattern,
        crowd_control: spec.crowd_control,
        engage: spec.engage,
        scaling: spec.scaling,
        durability: spec.durability,
        mobility: spec.mobility,
    }
}

pub fn draft_state(
    ally: Vec<Champion>,
    enemy: Vec<Champion>,
    contested_roles: Vec<Role>,
) -> DraftState {
    DraftState {
        patch: "15.6".into(),
        side: TeamSide::Blue,
        phase: DraftPhase::PickPhaseTwo,
        ally: TeamDraft { champions: ally },
        enemy: TeamDraft { champions: enemy },
        ally_bans: vec!["skarner".into(), "kalista".into()],
        enemy_bans: vec!["rell".into(), "vi".into()],
        contested_roles,
    }
}

pub fn support(
    name: &'static str,
    identities: Vec<CompositionIdentity>,
    execution_demand: ExecutionDemand,
    engage: u8,
    scaling: u8,
    durability: u8,
    mobility: u8,
    crowd_control: u8,
) -> Champion {
    champion(ChampionSpec {
        id: name,
        name,
        roles: vec![Role::Support],
        identities,
        damage_profile: DamageProfile::Magical,
        execution_demand,
        lane_pattern: LanePattern::Utility,
        crowd_control,
        engage,
        scaling,
        durability,
        mobility,
    })
}

pub fn top(
    name: &'static str,
    identities: Vec<CompositionIdentity>,
    execution_demand: ExecutionDemand,
    engage: u8,
    scaling: u8,
    durability: u8,
    mobility: u8,
    crowd_control: u8,
    lane_pattern: LanePattern,
) -> Champion {
    champion(ChampionSpec {
        id: name,
        name,
        roles: vec![Role::Top],
        identities,
        damage_profile: DamageProfile::Mixed,
        execution_demand,
        lane_pattern,
        crowd_control,
        engage,
        scaling,
        durability,
        mobility,
    })
}

pub fn jungle(
    name: &'static str,
    identities: Vec<CompositionIdentity>,
    execution_demand: ExecutionDemand,
    engage: u8,
    scaling: u8,
    durability: u8,
    mobility: u8,
    crowd_control: u8,
) -> Champion {
    champion(ChampionSpec {
        id: name,
        name,
        roles: vec![Role::Jungle],
        identities,
        damage_profile: DamageProfile::Mixed,
        execution_demand,
        lane_pattern: LanePattern::Roaming,
        crowd_control,
        engage,
        scaling,
        durability,
        mobility,
    })
}

pub fn mid(
    name: &'static str,
    identities: Vec<CompositionIdentity>,
    execution_demand: ExecutionDemand,
    engage: u8,
    scaling: u8,
    durability: u8,
    mobility: u8,
    crowd_control: u8,
    lane_pattern: LanePattern,
) -> Champion {
    champion(ChampionSpec {
        id: name,
        name,
        roles: vec![Role::Mid],
        identities,
        damage_profile: DamageProfile::Magical,
        execution_demand,
        lane_pattern,
        crowd_control,
        engage,
        scaling,
        durability,
        mobility,
    })
}

pub fn bottom(
    name: &'static str,
    identities: Vec<CompositionIdentity>,
    execution_demand: ExecutionDemand,
    scaling: u8,
    mobility: u8,
) -> Champion {
    champion(ChampionSpec {
        id: name,
        name,
        roles: vec![Role::Bottom],
        identities,
        damage_profile: DamageProfile::Physical,
        execution_demand,
        lane_pattern: LanePattern::Scaling,
        crowd_control: 1,
        engage: 0,
        scaling,
        durability: 1,
        mobility,
    })
}
