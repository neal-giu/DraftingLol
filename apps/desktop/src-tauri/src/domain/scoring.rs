use serde::{Deserialize, Serialize};

use super::{champion::Champion, composition::DraftAlert};

pub const INTERNAL_COHERENCE_WEIGHT: f32 = 0.35;
pub const ENEMY_MATCHUP_WEIGHT: f32 = 0.30;
pub const ROSTER_FIT_WEIGHT: f32 = 0.20;
pub const EXECUTION_SIMPLICITY_WEIGHT: f32 = 0.10;
pub const LANE_STABILITY_WEIGHT: f32 = 0.05;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreDimension {
    InternalCoherence,
    EnemyMatchup,
    RosterFit,
    ExecutionSimplicity,
    LaneStability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionPolarity {
    Bonus,
    Malus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreContributor {
    pub dimension: ScoreDimension,
    pub polarity: ContributionPolarity,
    pub label: String,
    pub value: f32,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplainedSubScore {
    pub raw_score: f32,
    pub weight: f32,
    pub weighted_score: f32,
    pub contributors: Vec<ScoreContributor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub internal_coherence: ExplainedSubScore,
    pub enemy_matchup: ExplainedSubScore,
    pub roster_fit: ExplainedSubScore,
    pub execution_simplicity: ExplainedSubScore,
    pub lane_stability: ExplainedSubScore,
    pub final_score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateEvaluation {
    pub champion: Champion,
    pub score_breakdown: ScoreBreakdown,
    pub alerts: Vec<DraftAlert>,
    pub explanation: Vec<String>,
    pub win_condition_after_pick: String,
    pub draft_call: String,
}

impl ScoreBreakdown {
    #[must_use]
    pub fn from_subscores(
        internal_coherence: ExplainedSubScore,
        enemy_matchup: ExplainedSubScore,
        roster_fit: ExplainedSubScore,
        execution_simplicity: ExplainedSubScore,
        lane_stability: ExplainedSubScore,
    ) -> Self {
        let final_score = internal_coherence.weighted_score
            + enemy_matchup.weighted_score
            + roster_fit.weighted_score
            + execution_simplicity.weighted_score
            + lane_stability.weighted_score;

        Self {
            internal_coherence,
            enemy_matchup,
            roster_fit,
            execution_simplicity,
            lane_stability,
            final_score,
        }
    }
}

#[must_use]
pub fn explained_subscore(
    raw_score: f32,
    weight: f32,
    contributors: Vec<ScoreContributor>,
) -> ExplainedSubScore {
    let bounded_score = raw_score.clamp(0.0, 100.0);

    ExplainedSubScore {
        raw_score: bounded_score,
        weight,
        weighted_score: bounded_score * weight,
        contributors,
    }
}
