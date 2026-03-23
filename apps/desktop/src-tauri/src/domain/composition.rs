use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionIdentity {
    Engage,
    Pick,
    Poke,
    Siege,
    FrontToBack,
    SplitPush,
    Skirmish,
    ProtectCarry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertCategory {
    Structural,
    Matchup,
    Roster,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DraftAlert {
    pub category: AlertCategory,
    pub severity: AlertSeverity,
    pub code: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CompositionProfile {
    pub engage: u8,
    pub disengage: u8,
    pub pick: u8,
    pub poke: u8,
    pub front_to_back: u8,
    pub split_push: u8,
    pub scaling: u8,
    pub wave_clear: u8,
}

impl CompositionProfile {
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            engage: self.engage.min(5),
            disengage: self.disengage.min(5),
            pick: self.pick.min(5),
            poke: self.poke.min(5),
            front_to_back: self.front_to_back.min(5),
            split_push: self.split_push.min(5),
            scaling: self.scaling.min(5),
            wave_clear: self.wave_clear.min(5),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionSnapshot {
    pub identities: Vec<CompositionIdentity>,
    pub profile: CompositionProfile,
    pub alerts: Vec<DraftAlert>,
}
