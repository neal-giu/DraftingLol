use serde::{Deserialize, Serialize};

use super::champion::{Champion, Role};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamSide {
    Blue,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftPhase {
    BanPhaseOne,
    PickPhaseOne,
    BanPhaseTwo,
    PickPhaseTwo,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamDraft {
    pub champions: Vec<Champion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftState {
    pub patch: String,
    pub side: TeamSide,
    pub phase: DraftPhase,
    pub ally: TeamDraft,
    pub enemy: TeamDraft,
    pub ally_bans: Vec<String>,
    pub enemy_bans: Vec<String>,
    pub contested_roles: Vec<Role>,
}

impl DraftState {
    #[must_use]
    pub fn missing_roles(&self) -> Vec<Role> {
        let mut missing = vec![
            Role::Top,
            Role::Jungle,
            Role::Mid,
            Role::Bottom,
            Role::Support,
        ];

        for champion in &self.ally.champions {
            if let Some(role) = champion.primary_role() {
                missing.retain(|candidate| candidate != &role);
            }
        }

        missing
    }
}
