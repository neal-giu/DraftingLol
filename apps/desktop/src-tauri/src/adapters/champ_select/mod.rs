use std::time::Duration;

use crate::{
    adapters::riot_client::{
        ChampionResolver, DraftStateTransformer, RiotChampSelectSession, RiotClientError,
        RiotSessionReader,
    },
    domain::draft::DraftState,
};

pub const POLL_IDLE_MS: u64 = 2_000;
pub const POLL_DRAFT_MS: u64 = 400;
pub const POLL_LOCK_IN_MS: u64 = 250;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftAvailability {
    Live,
    Sandbox { reason: DraftUnavailableReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftUnavailableReason {
    ClientAbsent,
    LockfileAbsent,
    LockfileInaccessible,
    LocalSslFailed,
    EndpointChanged,
    PatchUnknown,
    RequestFailed,
}

impl From<&RiotClientError> for DraftUnavailableReason {
    fn from(value: &RiotClientError) -> Self {
        match value {
            RiotClientError::ClientAbsent => Self::ClientAbsent,
            RiotClientError::LockfileAbsent => Self::LockfileAbsent,
            RiotClientError::LockfileInaccessible { .. } => Self::LockfileInaccessible,
            RiotClientError::LocalSslInitializationFailed(_) => Self::LocalSslFailed,
            RiotClientError::EndpointChanged { .. } => Self::EndpointChanged,
            RiotClientError::PatchUnknown => Self::PatchUnknown,
            RiotClientError::RequestFailed(_) | RiotClientError::LocalhostOnlyViolation { .. } => {
                Self::RequestFailed
            }
            RiotClientError::InvalidLockfile { .. } => Self::LockfileInaccessible,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DraftSnapshot {
    pub availability: DraftAvailability,
    pub state: DraftState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChampSelectEventKind {
    SessionStarted,
    PickChanged,
    BanChanged,
    PhaseChanged,
    SessionEnded,
}

impl ChampSelectEventKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SessionStarted => "session_started",
            Self::PickChanged => "pick_changed",
            Self::BanChanged => "ban_changed",
            Self::PhaseChanged => "phase_changed",
            Self::SessionEnded => "session_ended",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChampSelectEvent {
    pub kind: ChampSelectEventKind,
    pub state: Option<DraftSnapshot>,
}

#[derive(Debug, Clone)]
pub struct PollOutcome {
    pub next_interval: Duration,
    pub events: Vec<ChampSelectEvent>,
    pub snapshot: Option<DraftSnapshot>,
}

pub trait SandboxDraftSource {
    fn snapshot(&self, reason: DraftUnavailableReason) -> DraftState;
}

#[derive(Debug, Clone)]
pub struct ChampSelectMonitor<'resolver, Reader, Sandbox, Resolver>
where
    Reader: RiotSessionReader,
    Sandbox: SandboxDraftSource,
    Resolver: ChampionResolver,
{
    reader: Reader,
    sandbox: Sandbox,
    transformer: DraftStateTransformer<'resolver, Resolver>,
    previous: Option<DraftSnapshot>,
    previous_raw: Option<RiotChampSelectSession>,
}

impl<'resolver, Reader, Sandbox, Resolver> ChampSelectMonitor<'resolver, Reader, Sandbox, Resolver>
where
    Reader: RiotSessionReader,
    Sandbox: SandboxDraftSource,
    Resolver: ChampionResolver,
{
    #[must_use]
    pub fn new(reader: Reader, sandbox: Sandbox, resolver: &'resolver Resolver) -> Self {
        Self {
            reader,
            sandbox,
            transformer: DraftStateTransformer::new(resolver),
            previous: None,
            previous_raw: None,
        }
    }

    pub fn poll_once(&mut self) -> PollOutcome {
        match self.reader.fetch_session() {
            Ok(Some(session)) => self.handle_live_session(session),
            Ok(None) => self.handle_session_end(),
            Err(error) => self.handle_live_unavailable(error),
        }
    }

    fn handle_live_session(&mut self, session: RiotChampSelectSession) -> PollOutcome {
        match self.reader.fetch_patch_version() {
            Ok(patch) => match self.transformer.transform(Some(&patch), &session) {
                Ok(state) => {
                    let snapshot = DraftSnapshot {
                        availability: DraftAvailability::Live,
                        state,
                    };
                    let events = self.diff_events(Some(&session), Some(&snapshot));
                    self.previous = Some(snapshot.clone());
                    self.previous_raw = Some(session.clone());

                    PollOutcome {
                        next_interval: interval_for_session(&session),
                        events,
                        snapshot: Some(snapshot),
                    }
                }
                Err(_) => self.handle_live_unavailable(RiotClientError::PatchUnknown),
            },
            Err(error) => self.handle_live_unavailable(error),
        }
    }

    fn handle_session_end(&mut self) -> PollOutcome {
        let had_previous = self.previous.take().is_some();
        self.previous_raw = None;

        PollOutcome {
            next_interval: Duration::from_millis(POLL_IDLE_MS),
            events: had_previous
                .then(|| ChampSelectEvent {
                    kind: ChampSelectEventKind::SessionEnded,
                    state: None,
                })
                .into_iter()
                .collect(),
            snapshot: None,
        }
    }

    fn handle_live_unavailable(&mut self, error: RiotClientError) -> PollOutcome {
        let reason = DraftUnavailableReason::from(&error);
        let snapshot = DraftSnapshot {
            availability: DraftAvailability::Sandbox {
                reason: reason.clone(),
            },
            state: self.sandbox.snapshot(reason),
        };
        let events = self.diff_events(None, Some(&snapshot));
        self.previous = Some(snapshot.clone());
        self.previous_raw = None;

        PollOutcome {
            next_interval: Duration::from_millis(POLL_IDLE_MS),
            events,
            snapshot: Some(snapshot),
        }
    }

    fn diff_events(
        &self,
        raw_session: Option<&RiotChampSelectSession>,
        snapshot: Option<&DraftSnapshot>,
    ) -> Vec<ChampSelectEvent> {
        let mut events = Vec::new();

        match (&self.previous, snapshot) {
            (None, Some(current)) => {
                events.push(ChampSelectEvent {
                    kind: ChampSelectEventKind::SessionStarted,
                    state: Some(current.clone()),
                });
            }
            (Some(_), None) => {
                events.push(ChampSelectEvent {
                    kind: ChampSelectEventKind::SessionEnded,
                    state: None,
                });
                return events;
            }
            _ => {}
        }

        if let (Some(previous), Some(current)) = (&self.previous, snapshot) {
            if previous.state.phase != current.state.phase {
                events.push(ChampSelectEvent {
                    kind: ChampSelectEventKind::PhaseChanged,
                    state: Some(current.clone()),
                });
            }

            if previous.state.ally_bans != current.state.ally_bans
                || previous.state.enemy_bans != current.state.enemy_bans
            {
                events.push(ChampSelectEvent {
                    kind: ChampSelectEventKind::BanChanged,
                    state: Some(current.clone()),
                });
            }

            let previous_picks = (
                &previous.state.ally.champions,
                &previous.state.enemy.champions,
            );
            let current_picks = (
                &current.state.ally.champions,
                &current.state.enemy.champions,
            );
            if previous_picks != current_picks {
                events.push(ChampSelectEvent {
                    kind: ChampSelectEventKind::PickChanged,
                    state: Some(current.clone()),
                });
            }
        }

        if events.is_empty() && self.previous_raw.is_none() && raw_session.is_some() {
            events.push(ChampSelectEvent {
                kind: ChampSelectEventKind::SessionStarted,
                state: snapshot.cloned(),
            });
        }

        events
    }
}

#[must_use]
pub fn interval_for_session(session: &RiotChampSelectSession) -> Duration {
    if is_lock_in_window(session) {
        Duration::from_millis(POLL_LOCK_IN_MS)
    } else if is_draft_active(session) {
        Duration::from_millis(POLL_DRAFT_MS)
    } else {
        Duration::from_millis(POLL_IDLE_MS)
    }
}

fn is_draft_active(session: &RiotChampSelectSession) -> bool {
    !session.actions.is_empty()
        || !session.my_team.is_empty()
        || !session.their_team.is_empty()
        || matches!(session.timer.phase.as_str(), "BAN_PICK" | "FINALIZATION")
}

fn is_lock_in_window(session: &RiotChampSelectSession) -> bool {
    session
        .actions
        .iter()
        .flatten()
        .any(|action| action.action_type == "pick" && action.is_in_progress && !action.completed)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque};

    use crate::{
        adapters::riot_client::{LockfileData, RiotAction, RiotBans, RiotTeamMember, RiotTimer},
        domain::{
            champion::{Champion, DamageProfile, ExecutionDemand, LanePattern, Role},
            composition::CompositionIdentity,
            draft::{DraftPhase, TeamDraft, TeamSide},
        },
    };

    use super::*;

    #[derive(Debug, Clone)]
    struct QueueReader {
        sessions: RefCell<VecDeque<Result<Option<RiotChampSelectSession>, RiotClientError>>>,
        patches: RefCell<VecDeque<Result<String, RiotClientError>>>,
    }

    impl QueueReader {
        fn new(
            sessions: Vec<Result<Option<RiotChampSelectSession>, RiotClientError>>,
            patches: Vec<Result<String, RiotClientError>>,
        ) -> Self {
            Self {
                sessions: RefCell::new(sessions.into()),
                patches: RefCell::new(patches.into()),
            }
        }
    }

    impl RiotSessionReader for QueueReader {
        fn fetch_patch_version(&self) -> Result<String, RiotClientError> {
            self.patches
                .borrow_mut()
                .pop_front()
                .unwrap_or(Ok("15.7".into()))
        }

        fn fetch_session(&self) -> Result<Option<RiotChampSelectSession>, RiotClientError> {
            self.sessions.borrow_mut().pop_front().unwrap_or(Ok(None))
        }
    }

    #[derive(Debug, Clone)]
    struct StaticSandbox;

    impl SandboxDraftSource for StaticSandbox {
        fn snapshot(&self, _reason: DraftUnavailableReason) -> DraftState {
            DraftState {
                patch: "sandbox".into(),
                side: TeamSide::Blue,
                phase: DraftPhase::Complete,
                ally: TeamDraft { champions: vec![] },
                enemy: TeamDraft { champions: vec![] },
                ally_bans: vec![],
                enemy_bans: vec![],
                contested_roles: vec![],
            }
        }
    }

    #[derive(Debug, Clone)]
    struct TestResolver;

    impl ChampionResolver for TestResolver {
        fn resolve(&self, champion_id: i64) -> Option<Champion> {
            Some(Champion {
                id: champion_id.to_string(),
                name: format!("Champion {champion_id}"),
                roles: vec![Role::Top],
                role_confidence_map: [(Role::Top, 1.0)].into_iter().collect(),
                identities: vec![CompositionIdentity::Engage],
                damage_profile: DamageProfile::Mixed,
                execution_demand: ExecutionDemand::Low,
                lane_pattern: LanePattern::Stable,
                crowd_control: 1,
                engage: 1,
                scaling: 1,
                durability: 1,
                mobility: 1,
            })
        }
    }

    fn live_session(champion_id: i64) -> RiotChampSelectSession {
        RiotChampSelectSession {
            local_player_cell_id: 1,
            actions: vec![vec![RiotAction {
                actor_cell_id: 1,
                champion_id,
                completed: true,
                is_in_progress: false,
                action_type: "pick".into(),
            }]],
            my_team: vec![RiotTeamMember {
                cell_id: 1,
                champion_id,
                assigned_position: "top".into(),
                champion_pick_intent: champion_id,
            }],
            their_team: vec![],
            bans: RiotBans::default(),
            timer: RiotTimer {
                phase: "BAN_PICK".into(),
            },
        }
    }

    #[test]
    fn adaptive_polling_uses_expected_intervals() {
        assert_eq!(
            interval_for_session(&RiotChampSelectSession::default()),
            Duration::from_millis(POLL_IDLE_MS)
        );
        assert_eq!(
            interval_for_session(&live_session(10)),
            Duration::from_millis(POLL_DRAFT_MS)
        );

        let mut lock_in = live_session(10);
        lock_in.actions[0][0].completed = false;
        lock_in.actions[0][0].is_in_progress = true;
        assert_eq!(
            interval_for_session(&lock_in),
            Duration::from_millis(POLL_LOCK_IN_MS)
        );
    }

    #[test]
    fn emits_session_started_and_pick_changed_without_duplicates() {
        let reader = QueueReader::new(
            vec![Ok(Some(live_session(10))), Ok(Some(live_session(10)))],
            vec![Ok("15.7".into()), Ok("15.7".into())],
        );
        let mut monitor = ChampSelectMonitor::new(reader, StaticSandbox, &TestResolver);

        let first = monitor.poll_once();
        assert_eq!(first.events.len(), 1);
        assert_eq!(first.events[0].kind.as_str(), "session_started");

        let second = monitor.poll_once();
        assert!(second.events.is_empty());
    }

    #[test]
    fn emits_pick_ban_and_phase_events_when_state_changes() {
        let initial = live_session(10);
        let mut changed = live_session(20);
        changed.bans.my_team_bans = vec![99];
        changed.actions = vec![
            vec![
                RiotAction {
                    action_type: "ban".into(),
                    completed: true,
                    ..RiotAction::default()
                };
                6
            ],
            vec![RiotAction {
                action_type: "pick".into(),
                completed: true,
                champion_id: 20,
                actor_cell_id: 1,
                is_in_progress: false,
            }],
        ];

        let reader = QueueReader::new(
            vec![Ok(Some(initial)), Ok(Some(changed))],
            vec![Ok("15.7".into()), Ok("15.7".into())],
        );
        let mut monitor = ChampSelectMonitor::new(reader, StaticSandbox, &TestResolver);

        let first = monitor.poll_once();
        assert_eq!(first.events[0].kind, ChampSelectEventKind::SessionStarted);

        let second = monitor.poll_once();
        let event_kinds = second
            .events
            .iter()
            .map(|event| event.kind.clone())
            .collect::<Vec<_>>();

        assert!(event_kinds.contains(&ChampSelectEventKind::PickChanged));
        assert!(event_kinds.contains(&ChampSelectEventKind::BanChanged));
        assert!(event_kinds.contains(&ChampSelectEventKind::PhaseChanged));
    }

    #[test]
    fn falls_back_to_sandbox_when_live_is_unavailable() {
        let reader = QueueReader::new(vec![Err(RiotClientError::ClientAbsent)], vec![]);
        let mut monitor = ChampSelectMonitor::new(reader, StaticSandbox, &TestResolver);

        let outcome = monitor.poll_once();

        assert_eq!(outcome.events[0].kind, ChampSelectEventKind::SessionStarted);
        assert!(matches!(
            outcome.snapshot.unwrap().availability,
            DraftAvailability::Sandbox {
                reason: DraftUnavailableReason::ClientAbsent
            }
        ));
    }

    #[test]
    fn emits_session_ended_when_riot_session_disappears() {
        let reader = QueueReader::new(vec![Ok(None)], vec![]);
        let mut monitor = ChampSelectMonitor {
            reader,
            sandbox: StaticSandbox,
            transformer: DraftStateTransformer::new(&TestResolver),
            previous: Some(DraftSnapshot {
                availability: DraftAvailability::Live,
                state: DraftState {
                    patch: "15.7".into(),
                    side: TeamSide::Blue,
                    phase: DraftPhase::PickPhaseOne,
                    ally: TeamDraft { champions: vec![] },
                    enemy: TeamDraft { champions: vec![] },
                    ally_bans: vec![],
                    enemy_bans: vec![],
                    contested_roles: vec![],
                },
            }),
            previous_raw: Some(live_session(10)),
        };

        let outcome = monitor.poll_once();

        assert_eq!(outcome.events[0].kind, ChampSelectEventKind::SessionEnded);
    }

    #[test]
    fn maps_errors_to_expected_fallback_reasons() {
        assert_eq!(
            DraftUnavailableReason::from(&RiotClientError::LockfileAbsent),
            DraftUnavailableReason::LockfileAbsent
        );
        assert_eq!(
            DraftUnavailableReason::from(&RiotClientError::LocalSslInitializationFailed(
                "nope".into()
            )),
            DraftUnavailableReason::LocalSslFailed
        );
        assert_eq!(
            DraftUnavailableReason::from(&RiotClientError::PatchUnknown),
            DraftUnavailableReason::PatchUnknown
        );
        let _ = LockfileData::parse("LeagueClient:123:50443:password:https").unwrap();
    }
}
