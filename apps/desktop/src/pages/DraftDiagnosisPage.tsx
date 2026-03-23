import { Card, Pill } from '../components/ui';
import { identityLabel } from '../lib/sampleData';
import type { DraftDiagnosis } from '../types';

export function DraftDiagnosisPage({ diagnosis }: { diagnosis: DraftDiagnosis }) {
  return (
    <div className="page-grid two-columns">
      <Card title="Draft Diagnosis">
        <p>{diagnosis.draft_call}</p>
        <div className="pill-row">
          {diagnosis.composition.identities.map((identity) => (
            <Pill key={identity} tone="accent">
              {identityLabel(identity)}
            </Pill>
          ))}
        </div>
        <ul className="bullets compact-list">
          {diagnosis.review_notes.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
      </Card>

      <Card title="Alertes structurelles">
        <div className="alert-stack">
          {diagnosis.composition.alerts.map((alert) => (
            <article key={alert.code} className={`alert alert-${alert.severity}`}>
              <strong>{alert.title}</strong>
              <p>{alert.detail}</p>
            </article>
          ))}
        </div>
      </Card>
    </div>
  );
}
