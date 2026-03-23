import {
  defaultConfig,
  defaultDiagnosis,
  defaultDraftState,
  defaultRecommendations,
  defaultSandboxResponse,
  historyEntries,
} from './sampleData';
import type {
  DraftDiagnosis,
  DraftState,
  HistoryEntry,
  RecommendationResponse,
  SandboxSimulationResponse,
  TeamConfig,
} from '../types';

declare global {
  interface Window {
    __TAURI__?: unknown;
    __TAURI_INTERNALS__?: unknown;
  }
}

interface SaveConfigPayload {
  config: TeamConfig;
}


const commandNames = {
  loadConfig: 'load_app_config',
  saveConfig: 'save_app_config',
  liveRecommendations: 'get_live_draft_recommendations',
  diagnostics: 'get_draft_diagnostics',
  sandbox: 'run_sandbox_simulation',
  history: 'load_history_reviews',
} as const;

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && Boolean(window.__TAURI__ || window.__TAURI_INTERNALS__);
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime()) {
    return mockInvoke<T>(command, args);
  }

  const tauriInvoke = (window as Window & { __TAURI__?: { core?: { invoke?: <R>(cmd: string, args?: Record<string, unknown>) => Promise<R> } } }).__TAURI__?.core?.invoke;

  if (!tauriInvoke) {
    return mockInvoke<T>(command, args);
  }

  return tauriInvoke<T>(command, args);
}

async function mockInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  switch (command) {
    case commandNames.loadConfig:
      return structuredClone(defaultConfig) as T;
    case commandNames.saveConfig:
      return ((args as unknown as { payload: SaveConfigPayload }).payload.config) as T;
    case commandNames.liveRecommendations:
      return {
        ...structuredClone(defaultRecommendations),
        composition: {
          ...defaultRecommendations.composition,
          alerts: defaultRecommendations.composition.alerts,
        },
      } as T;
    case commandNames.diagnostics:
      return structuredClone(defaultDiagnosis) as T;
    case commandNames.sandbox:
      return structuredClone(defaultSandboxResponse) as T;
    case commandNames.history:
      return structuredClone(historyEntries) as T;
    default:
      throw new Error(`Unknown mocked command: ${command} ${JSON.stringify(args)}`);
  }
}

export const tauriClient = {
  loadConfig: () => invokeCommand<TeamConfig>(commandNames.loadConfig),
  saveConfig: (config: TeamConfig) =>
    invokeCommand<TeamConfig>(commandNames.saveConfig, { payload: { config } }),
  getLiveRecommendations: (draftState: DraftState) =>
    invokeCommand<RecommendationResponse>(commandNames.liveRecommendations, { payload: { draftState } }),
  getDraftDiagnostics: (draftState: DraftState) =>
    invokeCommand<DraftDiagnosis>(commandNames.diagnostics, { payload: { draftState } }),
  runSandboxSimulation: (draftState: DraftState) =>
    invokeCommand<SandboxSimulationResponse>(commandNames.sandbox, { payload: { draftState } }),
  loadHistory: () => invokeCommand<HistoryEntry[]>(commandNames.history),
  defaults: {
    config: defaultConfig,
    draftState: defaultDraftState,
  },
};
