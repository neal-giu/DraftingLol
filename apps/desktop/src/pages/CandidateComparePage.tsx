import { Card, Pill } from '../components/ui';
import { buildRapidReadSummary } from '../utils';
import type { RecommendationResponse, TeamConfig } from '../types';

export function CandidateComparePage({ recommendations, config }: { recommendations: RecommendationResponse; config: TeamConfig }) {
  return (
    <div className="page-grid compare-grid">
      {recommendations.top_5.slice(0, 3).map((candidate) => {
        const summary = buildRapidReadSummary(candidate, config);
        return (
          <Card key={candidate.champion.id} title={candidate.champion.name}>
            <div className="rapid-card">
              <div className="score-block">
                <span>Score global</span>
                <strong>{summary.score}</strong>
              </div>
              <div className="rapid-line"><span>Confort joueur</span><strong>{summary.playerComfort}</strong></div>
              <div className="rapid-line"><span>Identité</span><strong>{summary.resultingIdentity}</strong></div>
              <div>
                <small>3 bonus max</small>
                <div className="pill-row">
                  {summary.bonuses.map((bonus) => <Pill key={bonus} tone="success">{bonus}</Pill>)}
                </div>
              </div>
              <div>
                <small>2 risques max</small>
                <div className="pill-row">
                  {summary.risks.map((risk) => <Pill key={risk} tone="warning">{risk}</Pill>)}
                </div>
              </div>
            </div>
          </Card>
        );
      })}
    </div>
  );
}
