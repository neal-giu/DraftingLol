import { useEffect } from 'react';

import { DashboardPage } from './DashboardPage';
import { DraftDiagnosisPage } from './DraftDiagnosisPage';
import { HistoryReviewPage } from './HistoryReviewPage';
import { LiveDraftPage } from './LiveDraftPage';
import { RosterManagerPage } from './RosterManagerPage';
import { SandboxPage } from './SandboxPage';
import { CandidateComparePage } from './CandidateComparePage';
import { useUIStore } from '../store/uiStore';

const pageLabels = {
  dashboard: 'Dashboard',
  roster_manager: 'Roster Manager',
  live_draft: 'Live Draft',
  candidate_compare: 'Candidate Compare',
  draft_diagnosis: 'Draft Diagnosis',
  sandbox: 'Sandbox',
  history_review: 'History / Review',
} as const;

export function HomePage() {
  const {
    page,
    setPage,
    viewMode,
    toggleViewMode,
    config,
    recommendations,
    diagnosis,
    sandbox,
    history,
    draftState,
    initialize,
    refreshLiveDraft,
    refreshDiagnostics,
    runSandbox,
    saveConfig,
    isBusy,
    lastSavedAt,
  } = useUIStore();

  useEffect(() => {
    void initialize();
  }, [initialize]);

  return (
    <main className={`app-shell view-${viewMode}`}>
      <aside className="sidebar">
        <div>
          <p className="eyebrow">DraftingLol</p>
          <h1>Dense dark desktop</h1>
          <p className="sidebar-copy">
            Navigation rapide pour Dashboard, live draft, roster, compare, diagnosis, sandbox et review.
          </p>
        </div>

        <nav className="nav-stack">
          {Object.entries(pageLabels).map(([key, label]) => (
            <button
              key={key}
              type="button"
              className={`nav-button ${page === key ? 'active' : ''}`}
              onClick={() => setPage(key as keyof typeof pageLabels)}
            >
              {label}
            </button>
          ))}
        </nav>
      </aside>

      <section className="content-shell">
        <header className="topbar card">
          <div>
            <p className="eyebrow">Current view</p>
            <h2>{pageLabels[page]}</h2>
            <small>{config.teamName} · patch {config.patch}</small>
          </div>
          <div className="toolbar-actions">
            <button type="button" onClick={toggleViewMode}>
              Mode: {viewMode === 'normal' ? 'normal' : 'compact draft'}
            </button>
            <button type="button" onClick={() => void refreshLiveDraft()}>
              Refresh live
            </button>
            <button type="button" onClick={() => void refreshDiagnostics()}>
              Diagnostics
            </button>
            <button type="button" onClick={() => void runSandbox()}>
              Sandbox sim
            </button>
            <button type="button" onClick={() => void saveConfig()}>
              Save config
            </button>
          </div>
          <div className="status-line">
            <span>{isBusy ? 'Syncing…' : 'Ready'}</span>
            {lastSavedAt ? <span>Saved {new Date(lastSavedAt).toLocaleTimeString()}</span> : null}
          </div>
        </header>

        {page === 'dashboard' ? <DashboardPage config={config} recommendations={recommendations} diagnosis={diagnosis} /> : null}
        {page === 'roster_manager' ? <RosterManagerPage config={config} /> : null}
        {page === 'live_draft' ? (
          <LiveDraftPage
            draftState={draftState}
            recommendations={recommendations}
            diagnosis={diagnosis}
            config={config}
            viewMode={viewMode}
          />
        ) : null}
        {page === 'candidate_compare' ? <CandidateComparePage recommendations={recommendations} config={config} /> : null}
        {page === 'draft_diagnosis' ? <DraftDiagnosisPage diagnosis={diagnosis} /> : null}
        {page === 'sandbox' ? <SandboxPage sandbox={sandbox} /> : null}
        {page === 'history_review' ? <HistoryReviewPage history={history} /> : null}
      </section>
    </main>
  );
}
