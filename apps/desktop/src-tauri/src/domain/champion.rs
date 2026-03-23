use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::composition::CompositionIdentity;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bottom,
    Support,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageProfile {
    Physical,
    Magical,
    Mixed,
    True,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionDemand {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanePattern {
    Bully,
    Stable,
    Scaling,
    Roaming,
    Utility,
}

pub type RoleConfidenceMap = BTreeMap<Role, f32>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Champion {
    pub id: String,
    pub name: String,
    pub roles: Vec<Role>,
    pub role_confidence_map: RoleConfidenceMap,
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

impl Champion {
    #[must_use]
    pub fn primary_role(&self) -> Option<Role> {
        self.role_confidence_map
            .iter()
            .max_by(|left, right| left.1.total_cmp(right.1))
            .map(|(role, _)| role.clone())
            .or_else(|| self.roles.first().cloned())
    }

    #[must_use]
    pub fn role_confidence(&self, role: &Role) -> f32 {
        self.role_confidence_map
            .get(role)
            .copied()
            .unwrap_or_default()
    }
}
