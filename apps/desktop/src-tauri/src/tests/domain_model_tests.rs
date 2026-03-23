use crate::{
    application::recommendations::analyze_composition,
    domain::{
        champion::{ExecutionDemand, LanePattern, Role},
        composition::{CompositionIdentity, CompositionProfile},
        draft::DraftPhase,
        scoring::{
            ENEMY_MATCHUP_WEIGHT, EXECUTION_SIMPLICITY_WEIGHT, INTERNAL_COHERENCE_WEIGHT,
            LANE_STABILITY_WEIGHT, ROSTER_FIT_WEIGHT,
        },
    },
    tests::support::{bottom, draft_state, jungle, mid, top},
};

#[test]
fn scoring_weights_sum_to_one() {
    let total = INTERNAL_COHERENCE_WEIGHT
        + ENEMY_MATCHUP_WEIGHT
        + ROSTER_FIT_WEIGHT
        + EXECUTION_SIMPLICITY_WEIGHT
        + LANE_STABILITY_WEIGHT;

    assert!((total - 1.0).abs() < f32::EPSILON);
}

#[test]
fn composition_profile_clamps_each_axis_to_five() {
    let profile = CompositionProfile {
        engage: 9,
        disengage: 7,
        pick: 6,
        poke: 8,
        front_to_back: 11,
        split_push: 10,
        scaling: 12,
        wave_clear: 13,
    }
    .clamped();

    assert_eq!(profile.engage, 5);
    assert_eq!(profile.disengage, 5);
    assert_eq!(profile.pick, 5);
    assert_eq!(profile.poke, 5);
    assert_eq!(profile.front_to_back, 5);
    assert_eq!(profile.split_push, 5);
    assert_eq!(profile.scaling, 5);
    assert_eq!(profile.wave_clear, 5);
}

#[test]
fn analyze_composition_surfaces_missing_engage_on_unclear_plan() {
    let snapshot = analyze_composition(&crate::domain::draft::DraftState {
        phase: DraftPhase::PickPhaseTwo,
        ..draft_state(
            vec![
                top(
                    "Jayce",
                    vec![CompositionIdentity::Poke, CompositionIdentity::Siege],
                    ExecutionDemand::Medium,
                    1,
                    3,
                    2,
                    2,
                    1,
                    LanePattern::Bully,
                ),
                jungle(
                    "Viego",
                    vec![CompositionIdentity::Skirmish],
                    ExecutionDemand::Medium,
                    1,
                    3,
                    2,
                    3,
                    1,
                ),
                mid(
                    "Ahri",
                    vec![CompositionIdentity::Pick, CompositionIdentity::Skirmish],
                    ExecutionDemand::Medium,
                    2,
                    3,
                    1,
                    4,
                    3,
                    LanePattern::Roaming,
                ),
                bottom(
                    "Ezreal",
                    vec![CompositionIdentity::Poke],
                    ExecutionDemand::Medium,
                    3,
                    3,
                ),
            ],
            vec![],
            vec![Role::Support],
        )
    });

    assert!(snapshot
        .alerts
        .iter()
        .any(|alert| alert.code == "missing_primary_engage"));
}
