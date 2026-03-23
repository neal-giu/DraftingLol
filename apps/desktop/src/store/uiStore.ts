import { create } from '../lib/zustand';

import { tauriClient } from '../lib/tauri';
import { defaultDiagnosis, defaultRecommendations, defaultSandboxResponse, historyEntries } from '../lib/sampleData';
import type {
  AppPage,
  DraftDiagnosis,
  DraftState,
  HistoryEntry,
  RecommendationResponse,
  SandboxSimulationResponse,
  TeamConfig,
  ViewMode,
} from '../types';

interface UIState {
  page: AppPage;
  viewMode: ViewMode;
  config: TeamConfig;
  draftState: DraftState;
  recommendations: RecommendationResponse;
  diagnosis: DraftDiagnosis;
  sandbox: SandboxSimulationResponse;
  history: HistoryEntry[];
  isBusy: boolean;
  lastSavedAt: string | null;
  setPage: (page: AppPage) => void;
  toggleViewMode: () => void;
  updateConfig: (config: TeamConfig) => void;
  initialize: () => Promise<void>;
  saveConfig: () => Promise<void>;
  refreshLiveDraft: () => Promise<void>;
  refreshDiagnostics: () => Promise<void>;
  runSandbox: () => Promise<void>;
}

export const useUIStore = create<UIState>((set, get) => ({
  page: 'dashboard',
  viewMode: 'normal',
  config: tauriClient.defaults.config,
  draftState: tauriClient.defaults.draftState,
  recommendations: defaultRecommendations,
  diagnosis: defaultDiagnosis,
  sandbox: defaultSandboxResponse,
  history: historyEntries,
  isBusy: false,
  lastSavedAt: null,
  setPage: (page) => set({ page }),
  toggleViewMode: () =>
    set((state) => ({
      viewMode: state.viewMode === 'normal' ? 'compact_draft' : 'normal',
    })),
  updateConfig: (config) => set({ config }),
  initialize: async () => {
    set({ isBusy: true });

    const [config, recommendations, diagnosis, sandbox, history] = await Promise.all([
      tauriClient.loadConfig(),
      tauriClient.getLiveRecommendations(get().draftState),
      tauriClient.getDraftDiagnostics(get().draftState),
      tauriClient.runSandboxSimulation(get().draftState),
      tauriClient.loadHistory(),
    ]);

    set({
      config,
      recommendations,
      diagnosis,
      sandbox,
      history,
      isBusy: false,
    });
  },
  saveConfig: async () => {
    set({ isBusy: true });
    const config = await tauriClient.saveConfig(get().config);
    set({
      config,
      isBusy: false,
      lastSavedAt: new Date().toISOString(),
    });
  },
  refreshLiveDraft: async () => {
    set({ isBusy: true });
    const recommendations = await tauriClient.getLiveRecommendations(get().draftState);
    set({ recommendations, isBusy: false, page: 'live_draft' });
  },
  refreshDiagnostics: async () => {
    set({ isBusy: true });
    const diagnosis = await tauriClient.getDraftDiagnostics(get().draftState);
    set({ diagnosis, isBusy: false, page: 'draft_diagnosis' });
  },
  runSandbox: async () => {
    set({ isBusy: true });
    const sandbox = await tauriClient.runSandboxSimulation(get().draftState);
    set({ sandbox, isBusy: false, page: 'sandbox' });
  },
}));
