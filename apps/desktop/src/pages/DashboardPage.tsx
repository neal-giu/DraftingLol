import { Card, Pill, Stat } from '../components/ui';
import { identityLabel } from '../lib/sampleData';
import type { DraftDiagnosis, RecommendationResponse, TeamConfig } from '../types';

export function DashboardPage({ config, recommendations, diagnosis }: { config: TeamConfig; recommendations: RecommendationResponse; diagnosis: DraftDiagnosis }) {
  const topCandidate = recommendations.top_5[0];

  return (
    <div className="page-grid">
      <Card title="Dashboard" className="page-hero">
        <div className="hero-grid">
          <div>
            <h2>{config.teamName}</h2>
            <p>
              Lecture dense du patch {config.patch}, synthèse des identités prioritaires et accès rapide aux calls live.
            </p>
          </div>
          <div className="pill-row">
            {config.targetIdentity.map((identity) => (
              <Pill key={identity} tone="accent">
                {identityLabel(identity)}
              </Pill>
            ))}
          </div>
        </div>
      </Card>

      <Card title="Lecture rapide" className="dense-grid">
        <Stat label="Top pick actuel" value={topCandidate.champion.name} hint={topCandidate.draft_call} />
        <Stat label="Score global" value={Math.round(topCandidate.score_breakdown.final_score)} hint="pondéré live" />
        <Stat label="Win condition" value={diagnosis.win_condition.slice(0, 48) + '…'} />
        <Stat label="Alertes" value={recommendations.composition.alerts.length} hint="à traiter en bas d’écran" />
      </Card>

      <Card title="Coach summary">
        <ul className="bullets compact-list">
          {diagnosis.review_notes.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
      </Card>

      <Card title="Top picks live">
        <div className="candidate-stack">
          {recommendations.top_5.map((candidate) => (
            <article key={candidate.champion.id} className="candidate-row">
              <div>
                <strong>{candidate.champion.name}</strong>
                <p>{candidate.explanation.slice(0, 2).join(' · ')}</p>
              </div>
              <Pill tone="success">{Math.round(candidate.score_breakdown.final_score)}</Pill>
            </article>
          ))}
        </div>
      </Card>
    </div>
  );
}
