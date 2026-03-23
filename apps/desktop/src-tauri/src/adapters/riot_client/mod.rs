use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
}

impl HttpMethod {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
        }
    }
}

use serde::de::DeserializeOwned;

use crate::domain::{
    champion::Champion,
    draft::{DraftPhase, DraftState, TeamDraft, TeamSide},
};

pub const RIOT_LOCKFILE_NAME: &str = "lockfile";
pub const DEFAULT_RIOT_LOCKFILE_CANDIDATES: &[&str] = &[
    r"C:\\Riot Games\\League of Legends\\lockfile",
    r"C:\\Riot Games\\LeagueClient\\lockfile",
    "/Applications/League of Legends.app/Contents/LoL/lockfile",
    "/Applications/League of Legends.app/Contents/LeagueClient/lockfile",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiotClientError {
    ClientAbsent,
    LockfileAbsent,
    LockfileInaccessible {
        path: PathBuf,
        reason: String,
    },
    InvalidLockfile {
        reason: String,
    },
    LocalSslInitializationFailed(String),
    LocalhostOnlyViolation {
        url: String,
    },
    RequestFailed(String),
    EndpointChanged {
        endpoint: RiotEndpoint,
        status_code: u16,
    },
    PatchUnknown,
}

impl fmt::Display for RiotClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientAbsent => write!(f, "league client absent"),
            Self::LockfileAbsent => write!(f, "league lockfile absent"),
            Self::LockfileInaccessible { path, reason } => {
                write!(
                    f,
                    "league lockfile inaccessible at {}: {reason}",
                    path.display()
                )
            }
            Self::InvalidLockfile { reason } => write!(f, "invalid league lockfile: {reason}"),
            Self::LocalSslInitializationFailed(reason) => {
                write!(f, "local ssl initialization failed: {reason}")
            }
            Self::LocalhostOnlyViolation { url } => {
                write!(f, "riot client adapter refused non-localhost url: {url}")
            }
            Self::RequestFailed(reason) => write!(f, "riot client request failed: {reason}"),
            Self::EndpointChanged {
                endpoint,
                status_code,
            } => write!(
                f,
                "riot endpoint {} returned unexpected status {}",
                endpoint.path(),
                status_code
            ),
            Self::PatchUnknown => write!(f, "league patch is unknown"),
        }
    }
}

impl std::error::Error for RiotClientError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockfileData {
    pub process_name: String,
    pub pid: u32,
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

impl LockfileData {
    pub fn parse(raw: &str) -> Result<Self, RiotClientError> {
        let segments = raw.trim().split(':').collect::<Vec<_>>();

        if segments.len() != 5 {
            return Err(RiotClientError::InvalidLockfile {
                reason: format!("expected 5 segments, got {}", segments.len()),
            });
        }

        let pid = segments[1]
            .parse::<u32>()
            .map_err(|error| RiotClientError::InvalidLockfile {
                reason: format!("invalid pid: {error}"),
            })?;
        let port =
            segments[2]
                .parse::<u16>()
                .map_err(|error| RiotClientError::InvalidLockfile {
                    reason: format!("invalid port: {error}"),
                })?;

        Ok(Self {
            process_name: segments[0].to_owned(),
            pid,
            port,
            password: segments[3].to_owned(),
            protocol: segments[4].to_owned(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiotEndpoint {
    ChampSelectSession,
    CurrentSummoner,
    GameVersion,
}

impl RiotEndpoint {
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::ChampSelectSession => "/lol-champ-select/v1/session",
            Self::CurrentSummoner => "/lol-summoner/v1/current-summoner",
            Self::GameVersion => "/lol-patch/v1/game-version",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalRiotHttpClient {
    base_url: String,
    credentials: LockfileData,
}

impl LocalRiotHttpClient {
    pub fn new(credentials: LockfileData) -> Result<Self, RiotClientError> {
        if credentials.protocol != "https" {
            return Err(RiotClientError::LocalSslInitializationFailed(
                "riot local client must use https".into(),
            ));
        }

        let base_url = format!("{}://127.0.0.1:{}", credentials.protocol, credentials.port);

        Ok(Self {
            base_url,
            credentials,
        })
    }

    pub fn url_for(&self, endpoint: RiotEndpoint) -> String {
        format!("{}{}", self.base_url, endpoint.path())
    }

    fn ensure_localhost(url: &str) -> Result<(), RiotClientError> {
        if url.starts_with("https://127.0.0.1:")
            || url.starts_with("https://localhost:")
            || url.starts_with("https://[::1]:")
        {
            Ok(())
        } else {
            Err(RiotClientError::LocalhostOnlyViolation {
                url: url.to_string(),
            })
        }
    }

    pub fn request_json<T: DeserializeOwned>(
        &self,
        method: HttpMethod,
        endpoint: RiotEndpoint,
    ) -> Result<T, RiotClientError> {
        let url = self.url_for(endpoint);
        Self::ensure_localhost(&url)?;

        let output = std::process::Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--insecure",
                "--user",
                &format!("riot:{}", self.credentials.password),
                "--request",
                method.as_str(),
                "--write-out",
                "\n%{http_code}",
                &url,
            ])
            .output()
            .map_err(|error| RiotClientError::RequestFailed(error.to_string()))?;

        if !output.status.success() {
            return Err(RiotClientError::RequestFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let (body, status_line) = stdout.rsplit_once('\n').ok_or_else(|| {
            RiotClientError::RequestFailed("missing http status from curl output".into())
        })?;
        let status_code = status_line
            .trim()
            .parse::<u16>()
            .map_err(|error| RiotClientError::RequestFailed(error.to_string()))?;

        if (200..=299).contains(&status_code) {
            serde_json::from_str::<T>(body)
                .map_err(|error| RiotClientError::RequestFailed(error.to_string()))
        } else {
            Err(RiotClientError::EndpointChanged {
                endpoint,
                status_code,
            })
        }
    }

    pub fn game_version(&self) -> Result<String, RiotClientError> {
        let version = self.request_json::<String>(HttpMethod::Get, RiotEndpoint::GameVersion)?;

        if version.trim().is_empty() {
            return Err(RiotClientError::PatchUnknown);
        }

        Ok(version)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LockfileDiscoverer {
    candidate_paths: Vec<PathBuf>,
}

impl LockfileDiscoverer {
    #[must_use]
    pub fn new(candidate_paths: Vec<PathBuf>) -> Self {
        Self { candidate_paths }
    }

    #[must_use]
    pub fn with_default_candidates() -> Self {
        Self {
            candidate_paths: DEFAULT_RIOT_LOCKFILE_CANDIDATES
                .iter()
                .map(PathBuf::from)
                .collect(),
        }
    }

    pub fn discover(&self) -> Result<(PathBuf, LockfileData), RiotClientError> {
        if self.candidate_paths.is_empty() {
            return Err(RiotClientError::ClientAbsent);
        }

        let mut saw_existing_parent = false;

        for candidate in &self.candidate_paths {
            if candidate.exists() {
                let content = fs::read_to_string(candidate).map_err(|error| {
                    RiotClientError::LockfileInaccessible {
                        path: candidate.clone(),
                        reason: error.to_string(),
                    }
                })?;

                return Ok((candidate.clone(), LockfileData::parse(&content)?));
            }

            if let Some(parent) = candidate.parent() {
                if parent.exists() {
                    saw_existing_parent = true;
                }
            }
        }

        if saw_existing_parent {
            Err(RiotClientError::LockfileAbsent)
        } else {
            Err(RiotClientError::ClientAbsent)
        }
    }
}

pub trait ChampionResolver {
    fn resolve(&self, champion_id: i64) -> Option<Champion>;
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub struct RiotChampSelectSession {
    #[serde(default)]
    pub local_player_cell_id: i64,
    #[serde(default)]
    pub actions: Vec<Vec<RiotAction>>,
    #[serde(default)]
    pub my_team: Vec<RiotTeamMember>,
    #[serde(default)]
    pub their_team: Vec<RiotTeamMember>,
    #[serde(default)]
    pub bans: RiotBans,
    #[serde(default)]
    pub timer: RiotTimer,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
pub struct RiotAction {
    #[serde(default)]
    pub actor_cell_id: i64,
    #[serde(default)]
    pub champion_id: i64,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub is_in_progress: bool,
    #[serde(default, rename = "type")]
    pub action_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
pub struct RiotTeamMember {
    #[serde(default)]
    pub cell_id: i64,
    #[serde(default)]
    pub champion_id: i64,
    #[serde(default)]
    pub assigned_position: String,
    #[serde(default)]
    pub champion_pick_intent: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
pub struct RiotBans {
    #[serde(default)]
    pub my_team_bans: Vec<i64>,
    #[serde(default)]
    pub their_team_bans: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
pub struct RiotTimer {
    #[serde(default)]
    pub phase: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftTransformError {
    EndpointChanged(RiotEndpoint),
    PatchUnknown,
}

impl fmt::Display for DraftTransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndpointChanged(endpoint) => {
                write!(f, "riot payload changed for endpoint {}", endpoint.path())
            }
            Self::PatchUnknown => write!(f, "riot patch version is unknown"),
        }
    }
}

impl std::error::Error for DraftTransformError {}

#[derive(Debug, Clone)]
pub struct DraftStateTransformer<'resolver, R>
where
    R: ChampionResolver,
{
    resolver: &'resolver R,
}

impl<'resolver, R> DraftStateTransformer<'resolver, R>
where
    R: ChampionResolver,
{
    #[must_use]
    pub fn new(resolver: &'resolver R) -> Self {
        Self { resolver }
    }

    pub fn transform(
        &self,
        patch: Option<&str>,
        session: &RiotChampSelectSession,
    ) -> Result<DraftState, DraftTransformError> {
        let patch = patch
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(DraftTransformError::PatchUnknown)?;

        let ally = session
            .my_team
            .iter()
            .filter_map(|member| self.resolver.resolve(member.champion_id))
            .collect::<Vec<_>>();
        let enemy = session
            .their_team
            .iter()
            .filter_map(|member| self.resolver.resolve(member.champion_id))
            .collect::<Vec<_>>();

        Ok(DraftState {
            patch: patch.to_owned(),
            side: infer_side(session),
            phase: infer_phase(session),
            ally: TeamDraft { champions: ally },
            enemy: TeamDraft { champions: enemy },
            ally_bans: session
                .bans
                .my_team_bans
                .iter()
                .map(ToString::to_string)
                .collect(),
            enemy_bans: session
                .bans
                .their_team_bans
                .iter()
                .map(ToString::to_string)
                .collect(),
            contested_roles: infer_contested_roles(session),
        })
    }
}

fn infer_side(session: &RiotChampSelectSession) -> TeamSide {
    if session.local_player_cell_id <= 4 {
        TeamSide::Blue
    } else {
        TeamSide::Red
    }
}

fn infer_phase(session: &RiotChampSelectSession) -> DraftPhase {
    match session.timer.phase.as_str() {
        "BAN_PICK" => classify_ban_pick(session),
        "FINALIZATION" | "GAME_STARTING" => DraftPhase::Complete,
        _ => classify_ban_pick(session),
    }
}

fn classify_ban_pick(session: &RiotChampSelectSession) -> DraftPhase {
    let completed_bans = session
        .actions
        .iter()
        .flatten()
        .filter(|action| action.action_type == "ban" && action.completed)
        .count();
    let completed_picks = session
        .actions
        .iter()
        .flatten()
        .filter(|action| action.action_type == "pick" && action.completed)
        .count();

    if completed_bans < 6 {
        DraftPhase::BanPhaseOne
    } else if completed_picks < 6 {
        DraftPhase::PickPhaseOne
    } else if completed_bans < 10 {
        DraftPhase::BanPhaseTwo
    } else if completed_picks < 10 {
        DraftPhase::PickPhaseTwo
    } else {
        DraftPhase::Complete
    }
}

fn infer_contested_roles(session: &RiotChampSelectSession) -> Vec<crate::domain::champion::Role> {
    use crate::domain::champion::Role;

    let mut positions = session
        .my_team
        .iter()
        .filter_map(|member| match member.assigned_position.as_str() {
            "top" => Some(Role::Top),
            "jungle" => Some(Role::Jungle),
            "middle" | "mid" => Some(Role::Mid),
            "bottom" => Some(Role::Bottom),
            "utility" | "support" => Some(Role::Support),
            _ => None,
        })
        .collect::<Vec<_>>();

    positions.sort();
    positions
        .windows(2)
        .filter_map(|window| {
            if window[0] == window[1] {
                Some(window[0].clone())
            } else {
                None
            }
        })
        .collect()
}

pub trait RiotSessionReader {
    fn fetch_patch_version(&self) -> Result<String, RiotClientError>;
    fn fetch_session(&self) -> Result<Option<RiotChampSelectSession>, RiotClientError>;
}

#[derive(Debug, Clone)]
pub struct RiotLiveClient {
    http: LocalRiotHttpClient,
}

impl RiotLiveClient {
    #[must_use]
    pub fn new(http: LocalRiotHttpClient) -> Self {
        Self { http }
    }
}

impl RiotSessionReader for RiotLiveClient {
    fn fetch_patch_version(&self) -> Result<String, RiotClientError> {
        self.http.game_version()
    }

    fn fetch_session(&self) -> Result<Option<RiotChampSelectSession>, RiotClientError> {
        match self.http.request_json::<RiotChampSelectSession>(
            HttpMethod::Get,
            RiotEndpoint::ChampSelectSession,
        ) {
            Ok(session) => Ok(Some(session)),
            Err(RiotClientError::EndpointChanged {
                endpoint,
                status_code: 404,
            }) if endpoint == RiotEndpoint::ChampSelectSession => Ok(None),
            Err(RiotClientError::RequestFailed(reason)) if reason.contains("404") => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiotClientFactory {
    discoverer: LockfileDiscoverer,
}

impl RiotClientFactory {
    #[must_use]
    pub fn new(discoverer: LockfileDiscoverer) -> Self {
        Self { discoverer }
    }

    pub fn connect(&self) -> Result<RiotLiveClient, RiotClientError> {
        let (_, lockfile) = self.discoverer.discover()?;
        let http = LocalRiotHttpClient::new(lockfile)?;
        Ok(RiotLiveClient::new(http))
    }
}

pub fn default_lockfile_candidates() -> Vec<PathBuf> {
    DEFAULT_RIOT_LOCKFILE_CANDIDATES
        .iter()
        .map(PathBuf::from)
        .collect()
}

pub fn discover_lockfile_in_directory(
    directory: &Path,
) -> Result<(PathBuf, LockfileData), RiotClientError> {
    let discoverer = LockfileDiscoverer::new(vec![directory.join(RIOT_LOCKFILE_NAME)]);
    discoverer.discover()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::domain::{
        champion::{DamageProfile, ExecutionDemand, LanePattern, Role},
        composition::CompositionIdentity,
    };

    struct TestResolver;

    impl ChampionResolver for TestResolver {
        fn resolve(&self, champion_id: i64) -> Option<Champion> {
            let role = if champion_id % 2 == 0 {
                Role::Top
            } else {
                Role::Jungle
            };
            let mut role_confidence_map = BTreeMap::new();
            role_confidence_map.insert(role.clone(), 1.0);

            Some(Champion {
                id: champion_id.to_string(),
                name: format!("Champion {champion_id}"),
                roles: vec![role],
                role_confidence_map,
                identities: vec![CompositionIdentity::Engage],
                damage_profile: DamageProfile::Mixed,
                execution_demand: ExecutionDemand::Low,
                lane_pattern: LanePattern::Stable,
                crowd_control: 2,
                engage: 2,
                scaling: 2,
                durability: 2,
                mobility: 2,
            })
        }
    }

    #[test]
    fn parses_lockfile() {
        let parsed = LockfileData::parse("LeagueClient:123:50443:password:https").unwrap();

        assert_eq!(parsed.pid, 123);
        assert_eq!(parsed.port, 50_443);
        assert_eq!(parsed.password, "password");
        assert_eq!(parsed.protocol, "https");
    }

    #[test]
    fn refuses_non_localhost_urls() {
        let error = LocalRiotHttpClient::ensure_localhost(
            "https://riotgames.com/lol-champ-select/v1/session",
        )
        .unwrap_err();

        assert!(matches!(
            error,
            RiotClientError::LocalhostOnlyViolation { .. }
        ));
    }

    #[test]
    fn discovers_lockfile_in_directory() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("draft-team-lockfile-{nonce}"));
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(
            temp_dir.join(RIOT_LOCKFILE_NAME),
            "LeagueClient:123:50443:password:https",
        )
        .unwrap();

        let (_, lockfile) = super::discover_lockfile_in_directory(&temp_dir).unwrap();

        assert_eq!(lockfile.port, 50_443);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn transforms_riot_payload_into_draft_state() {
        let transformer = DraftStateTransformer::new(&TestResolver);
        let session = RiotChampSelectSession {
            local_player_cell_id: 2,
            actions: vec![vec![RiotAction {
                action_type: "ban".into(),
                completed: true,
                ..RiotAction::default()
            }]],
            my_team: vec![RiotTeamMember {
                cell_id: 2,
                champion_id: 2,
                assigned_position: "top".into(),
                champion_pick_intent: 266,
            }],
            their_team: vec![RiotTeamMember {
                cell_id: 6,
                champion_id: 3,
                assigned_position: "jungle".into(),
                champion_pick_intent: 64,
            }],
            bans: RiotBans {
                my_team_bans: vec![12],
                their_team_bans: vec![34],
            },
            timer: RiotTimer {
                phase: "BAN_PICK".into(),
            },
        };

        let state = transformer.transform(Some("15.7"), &session).unwrap();

        assert_eq!(state.patch, "15.7");
        assert_eq!(state.side, TeamSide::Blue);
        assert_eq!(state.phase, DraftPhase::BanPhaseOne);
        assert_eq!(state.ally.champions.len(), 1);
        assert_eq!(state.enemy.champions.len(), 1);
        assert_eq!(state.ally_bans, vec!["12"]);
    }

    #[test]
    fn rejects_unknown_patch() {
        let transformer = DraftStateTransformer::new(&TestResolver);
        let error = transformer
            .transform(Some("   "), &RiotChampSelectSession::default())
            .unwrap_err();

        assert!(matches!(error, DraftTransformError::PatchUnknown));
    }
}
