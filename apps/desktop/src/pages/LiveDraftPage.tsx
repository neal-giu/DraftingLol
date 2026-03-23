import { Pill } from '../components/ui';
import { buildRapidReadSummary } from '../utils';
import type { DraftDiagnosis, DraftState, RecommendationResponse, TeamConfig, ViewMode } from '../types';

export function LiveDraftPage({
  draftState,
  recommendations,
  diagnosis,
  config,
  viewMode,
}: {
  draftState: DraftState;
  recommendations: RecommendationResponse;
  diagnosis: DraftDiagnosis;
  config: TeamConfig;
  viewMode: ViewMode;
}) {
  const lead = recommendations.top_5[0];
  const summary = buildRapidReadSummary(lead, config);

  return (
    <div className={`live-layout live-layout-${viewMode}`}>
      <section className="live-column enemy-column card">
        <header className="card-header">
          <div>
            <p className="eyebrow">Live Draft</p>
            <h3>Draft ennemie</h3>
          </div>
          <Pill tone="danger">{draftState.side === 'blue' ? 'Blue side' : 'Red side'}</Pill>
        </header>
        <div className="champion-list">
          {draftState.enemy.champions.map((champion) => (
            <article key={champion.id} className="draft-chip enemy">
              <strong>{champion.name}</strong>
              <span>{champion.identities.slice(0, 2).join(' · ')}</span>
            </article>
          ))}
        </div>
      </section>

      <section className="live-column core-column card">
        <header className="card-header">
          <div>
            <p className="eyebrow">Centre</p>
            <h3>État draft + bans + identités</h3>
          </div>
          <Pill tone="accent">{draftState.phase}</Pill>
        </header>
        <div className="center-grid">
          <div>
            <small>Alliés lock</small>
            <div className="champion-list compact">
              {draftState.ally.champions.map((champion) => (
                <article key={champion.id} className="draft-chip ally">
                  <strong>{champion.name}</strong>
                  <span>{champion.roles[0]}</span>
                </article>
              ))}
            </div>
          </div>
          <div>
            <small>Bans</small>
            <div className="pill-row">
              {[...draftState.ally_bans, ...draftState.enemy_bans].map((ban) => (
                <Pill key={ban}>{ban}</Pill>
              ))}
            </div>
          </div>
          <div>
            <small>Identités résultantes</small>
            <div className="pill-row">
              {recommendations.composition.identities.map((identity) => (
                <Pill key={identity} tone="accent">
                  {identity.split('_').join(' ')}
                </Pill>
              ))}
            </div>
          </div>
        </div>
      </section>

      <section className="live-column picks-column card">
        <header className="card-header">
          <div>
            <p className="eyebrow">Right rail</p>
            <h3>Top picks + explications</h3>
          </div>
          <Pill tone="success">Top 3</Pill>
        </header>
        <div className="candidate-stack">
          {recommendations.top_5.slice(0, 3).map((candidate) => {
            const item = buildRapidReadSummary(candidate, config);
            return (
              <article key={candidate.champion.id} className="pick-card">
                <div className="candidate-row">
                  <strong>{candidate.champion.name}</strong>
                  <Pill tone="success">{item.score}</Pill>
                </div>
                <div className="rapid-line"><span>Confort joueur</span><strong>{item.playerComfort}</strong></div>
                <div className="pill-row">
                  {item.bonuses.map((bonus) => (
                    <Pill key={bonus} tone="success">
                      {bonus}
                    </Pill>
                  ))}
                </div>
                <div className="pill-row">
                  {item.risks.map((risk) => (
                    <Pill key={risk} tone="warning">
                      {risk}
                    </Pill>
                  ))}
                </div>
              </article>
            );
          })}
        </div>
      </section>

      <section className="live-footer card">
        <header className="card-header">
          <div>
            <p className="eyebrow">Bottom bar</p>
            <h3>Alertes + call + win condition</h3>
          </div>
        </header>
        <div className="footer-grid">
          <div>
            <small>Alertes</small>
            <div className="pill-row">
              {recommendations.composition.alerts.map((alert) => (
                <Pill key={alert.code} tone={alert.severity === 'critical' ? 'danger' : 'warning'}>
                  {alert.title}
                </Pill>
              ))}
            </div>
          </div>
          <div>
            <small>Call</small>
            <p>{lead.draft_call}</p>
          </div>
          <div>
            <small>Win condition</small>
            <p>{diagnosis.win_condition}</p>
          </div>
          <div>
            <small>Identité résultante</small>
            <p>{summary.resultingIdentity}</p>
          </div>
        </div>
      </section>
    </div>
  );
}
