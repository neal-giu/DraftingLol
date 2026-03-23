PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    tag TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS players (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    handle TEXT NOT NULL,
    role TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'starter',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_players_team_id ON players(team_id);

CREATE TABLE IF NOT EXISTS champions (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    canonical_role TEXT NOT NULL,
    archetypes_json TEXT NOT NULL,
    incomplete_profile INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS champion_versions (
    id TEXT PRIMARY KEY,
    champion_id TEXT NOT NULL,
    patch TEXT NOT NULL,
    source_path TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    compatible_since TEXT NOT NULL,
    is_latest INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (champion_id) REFERENCES champions(id) ON DELETE CASCADE,
    UNIQUE (champion_id, patch)
);
CREATE INDEX IF NOT EXISTS idx_champion_versions_lookup ON champion_versions(champion_id, compatible_since DESC, patch DESC);

CREATE TABLE IF NOT EXISTS player_champion_pools (
    id TEXT PRIMARY KEY,
    player_id TEXT NOT NULL,
    champion_id TEXT NOT NULL,
    patch TEXT NOT NULL,
    mastery_score REAL NOT NULL,
    proficiency_tier TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (player_id) REFERENCES players(id) ON DELETE CASCADE,
    FOREIGN KEY (champion_id) REFERENCES champions(id) ON DELETE CASCADE,
    UNIQUE (player_id, champion_id, patch)
);
CREATE INDEX IF NOT EXISTS idx_player_champion_pools_player_id ON player_champion_pools(player_id);

CREATE TABLE IF NOT EXISTS team_preferences (
    id TEXT PRIMARY KEY,
    team_id TEXT NOT NULL,
    patch TEXT NOT NULL,
    preferences_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
    UNIQUE (team_id, patch)
);

CREATE TABLE IF NOT EXISTS draft_sessions (
    id TEXT PRIMARY KEY,
    team_id TEXT,
    mode TEXT NOT NULL,
    patch TEXT NOT NULL,
    side TEXT NOT NULL,
    status TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TEXT,
    FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_draft_sessions_team_status ON draft_sessions(team_id, status, started_at DESC);

CREATE TABLE IF NOT EXISTS draft_events (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    phase TEXT NOT NULL,
    team TEXT NOT NULL,
    action TEXT NOT NULL,
    champion_id TEXT,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES draft_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (champion_id) REFERENCES champions(id) ON DELETE SET NULL,
    UNIQUE (session_id, sequence)
);
CREATE INDEX IF NOT EXISTS idx_draft_events_session_id ON draft_events(session_id, sequence ASC);

CREATE TABLE IF NOT EXISTS draft_recommendations (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    event_id TEXT,
    candidate_champion_id TEXT NOT NULL,
    ranking INTEGER NOT NULL,
    score REAL NOT NULL,
    reasoning_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES draft_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (event_id) REFERENCES draft_events(id) ON DELETE SET NULL,
    FOREIGN KEY (candidate_champion_id) REFERENCES champions(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_draft_recommendations_session_id ON draft_recommendations(session_id, ranking ASC);

CREATE TABLE IF NOT EXISTS draft_final_reviews (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    summary TEXT NOT NULL,
    review_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES draft_sessions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
