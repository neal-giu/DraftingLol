import { Card, Pill } from '../components/ui';
import type { HistoryEntry } from '../types';

export function HistoryReviewPage({ history }: { history: HistoryEntry[] }) {
  return (
    <div className="page-grid">
      <Card title="History / Review">
        <div className="scenario-list">
          {history.map((entry) => (
            <article key={entry.id} className="history-card">
              <div className="candidate-row">
                <div>
                  <strong>{entry.opponent}</strong>
                  <p>{entry.reviewHeadline}</p>
                </div>
                <Pill tone={entry.result === 'win' ? 'success' : 'danger'}>{entry.result}</Pill>
              </div>
              <small>Patch {entry.patch}</small>
              <p>{entry.draftCall}</p>
            </article>
          ))}
        </div>
      </Card>
    </div>
  );
}
