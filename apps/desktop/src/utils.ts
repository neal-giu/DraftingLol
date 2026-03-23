import type { CandidateEvaluation, RapidReadSummary, TeamConfig } from './types';

export function buildRapidReadSummary(candidate: CandidateEvaluation, config: TeamConfig): RapidReadSummary {
  const matchingMember = config.members.find((member) =>
    member.comfortPicks.some(
      (pick) => pick.toLowerCase() === candidate.champion.name.toLowerCase() || pick.toLowerCase() === candidate.champion.id,
    ),
  );

  const bonuses = candidate.explanation.slice(0, 3);
  const risks = candidate.alerts.map((alert) => alert.title).slice(0, 2);

  return {
    score: Math.round(candidate.score_breakdown.final_score),
    playerComfort: matchingMember ? `${matchingMember.handle} · confort fort` : 'flex · confort moyen',
    bonuses,
    risks,
    resultingIdentity: candidate.champion.identities.slice(0, 2).join(' + '),
  };
}
