import { Card, Pill } from '../components/ui';
import type { SandboxSimulationResponse } from '../types';

export function SandboxPage({ sandbox }: { sandbox: SandboxSimulationResponse }) {
  return (
    <div className="page-grid two-columns">
      <Card title="Sandbox">
        <p>Simulations rapides pour tester plusieurs lectures sans polluer l’écran live.</p>
        <div className="scenario-list">
          {sandbox.scenarios.map((scenario) => (
            <article key={scenario.id} className="scenario-card">
              <div className="candidate-row">
                <strong>{scenario.label}</strong>
                <Pill tone="success">{scenario.projected_score}</Pill>
              </div>
              <p>{scenario.summary}</p>
              <div className="pill-row">
                <Pill tone="accent">{scenario.pick}</Pill>
                {scenario.risks.slice(0, 2).map((risk) => (
                  <Pill key={risk} tone="warning">
                    {risk}
                  </Pill>
                ))}
              </div>
            </article>
          ))}
        </div>
      </Card>

      <Card title="Baseline">
        <p>Top pick baseline : {sandbox.baseline.top_5[0]?.champion.name}</p>
        <p>Comparer les scénarios uniquement avec une lecture claire de la win condition.</p>
      </Card>
    </div>
  );
}
