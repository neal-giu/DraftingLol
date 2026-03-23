import { Card, Pill } from '../components/ui';
import type { TeamConfig } from '../types';

export function RosterManagerPage({ config }: { config: TeamConfig }) {
  return (
    <div className="page-grid two-columns">
      <Card title="Roster Manager">
        <div className="roster-header">
          <div>
            <h2>{config.teamName}</h2>
            <p>{config.preferredStyle}</p>
          </div>
          <Pill tone="accent">Patch {config.patch}</Pill>
        </div>
        <div className="roster-table">
          {config.members.map((member) => (
            <article key={member.id} className="roster-row">
              <div>
                <strong>{member.handle}</strong>
                <span>{member.role}</span>
              </div>
              <div>
                <small>Comfort</small>
                <p>{member.comfortPicks.join(' · ')}</p>
              </div>
              <div>
                <small>Focus</small>
                <p>{member.focus}</p>
              </div>
            </article>
          ))}
        </div>
      </Card>

      <Card title="Draft prep">
        <ul className="bullets">
          <li>Préparer des scripts par rôle pour chaque phase de draft.</li>
          <li>Limiter à 3 bonus coach par joueur pour garder la lecture dense.</li>
          <li>Mettre à jour la comfort map avant la série.</li>
        </ul>
      </Card>
    </div>
  );
}
