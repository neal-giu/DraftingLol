export type ViewMode = 'normal' | 'compact_draft';
export type AppPage =
  | 'dashboard'
  | 'roster_manager'
  | 'live_draft'
  | 'candidate_compare'
  | 'draft_diagnosis'
  | 'sandbox'
  | 'history_review';

export type Role = 'top' | 'jungle' | 'mid' | 'bottom' | 'support';
export type TeamSide = 'blue' | 'red';
export type DraftPhase =
  | 'ban_phase_one'
  | 'pick_phase_one'
  | 'ban_phase_two'
  | 'pick_phase_two'
  | 'complete';
export type CompositionIdentity =
  | 'engage'
  | 'pick'
  | 'poke'
  | 'siege'
  | 'front_to_back'
  | 'split_push'
  | 'skirmish'
  | 'protect_carry';
export type AlertSeverity = 'info' | 'warning' | 'critical';
export type AlertCategory = 'structural' | 'matchup' | 'roster';
export type ScoreDimension =
  | 'internal_coherence'
  | 'enemy_matchup'
  | 'roster_fit'
  | 'execution_simplicity'
  | 'lane_stability';
export type ContributionPolarity = 'bonus' | 'malus';

export interface TeamMember {
  id: string;
  handle: string;
  role: Role;
  comfortPicks: string[];
  focus: string;
}

export interface TeamConfig {
  teamName: string;
  patch: string;
  preferredStyle: string;
  targetIdentity: CompositionIdentity[];
  members: TeamMember[];
}

export interface Champion {
  id: string;
  name: string;
  roles: Role[];
  role_confidence_map: Partial<Record<Role, number>>;
  identities: CompositionIdentity[];
  damage_profile: 'physical' | 'magical' | 'mixed' | 'true';
  execution_demand: 'low' | 'medium' | 'high';
  lane_pattern: 'bully' | 'stable' | 'scaling' | 'roaming' | 'utility';
  crowd_control: number;
  engage: number;
  scaling: number;
  durability: number;
  mobility: number;
}

export interface TeamDraft {
  champions: Champion[];
}

export interface DraftState {
  patch: string;
  side: TeamSide;
  phase: DraftPhase;
  ally: TeamDraft;
  enemy: TeamDraft;
  ally_bans: string[];
  enemy_bans: string[];
  contested_roles: Role[];
}

export interface DraftAlert {
  category: AlertCategory;
  severity: AlertSeverity;
  code: string;
  title: string;
  detail: string;
}

export interface CompositionProfile {
  engage: number;
  disengage: number;
  pick: number;
  poke: number;
  front_to_back: number;
  split_push: number;
  scaling: number;
  wave_clear: number;
}

export interface CompositionSnapshot {
  identities: CompositionIdentity[];
  profile: CompositionProfile;
  alerts: DraftAlert[];
}

export interface ScoreContributor {
  dimension: ScoreDimension;
  polarity: ContributionPolarity;
  label: string;
  value: number;
  detail: string;
}

export interface ExplainedSubScore {
  raw_score: number;
  weight: number;
  weighted_score: number;
  contributors: ScoreContributor[];
}

export interface ScoreBreakdown {
  internal_coherence: ExplainedSubScore;
  enemy_matchup: ExplainedSubScore;
  roster_fit: ExplainedSubScore;
  execution_simplicity: ExplainedSubScore;
  lane_stability: ExplainedSubScore;
  final_score: number;
}

export interface CandidateEvaluation {
  champion: Champion;
  score_breakdown: ScoreBreakdown;
  alerts: DraftAlert[];
  explanation: string[];
  win_condition_after_pick: string;
  draft_call: string;
}

export interface RecommendationResponse {
  composition: CompositionSnapshot;
  top_5: CandidateEvaluation[];
  evaluated_candidates: CandidateEvaluation[];
}

export interface DraftDiagnosis {
  composition: CompositionSnapshot;
  win_condition: string;
  draft_call: string;
  review_notes: string[];
}

export interface SandboxScenario {
  id: string;
  label: string;
  pick: string;
  projected_score: number;
  summary: string;
  risks: string[];
}

export interface SandboxSimulationResponse {
  baseline: RecommendationResponse;
  scenarios: SandboxScenario[];
}

export interface HistoryEntry {
  id: string;
  patch: string;
  opponent: string;
  result: 'win' | 'loss';
  reviewHeadline: string;
  draftCall: string;
}

export interface RapidReadSummary {
  score: number;
  playerComfort: string;
  bonuses: string[];
  risks: string[];
  resultingIdentity: string;
}
