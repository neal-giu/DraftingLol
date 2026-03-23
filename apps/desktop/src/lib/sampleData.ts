import type {
  CandidateEvaluation,
  Champion,
  CompositionIdentity,
  DraftDiagnosis,
  DraftState,
  HistoryEntry,
  RecommendationResponse,
  SandboxSimulationResponse,
  TeamConfig,
} from '../types';

const roleConfidence = (role: Champion['roles'][number]) => ({ [role]: 1 });

export const championCatalog: Champion[] = [
  {
    id: 'ornn',
    name: 'Ornn',
    roles: ['top'],
    role_confidence_map: roleConfidence('top'),
    identities: ['engage', 'front_to_back'],
    damage_profile: 'mixed',
    execution_demand: 'low',
    lane_pattern: 'stable',
    crowd_control: 4,
    engage: 5,
    scaling: 4,
    durability: 5,
    mobility: 1,
  },
  {
    id: 'sejuani',
    name: 'Sejuani',
    roles: ['jungle'],
    role_confidence_map: roleConfidence('jungle'),
    identities: ['engage', 'front_to_back', 'pick'],
    damage_profile: 'mixed',
    execution_demand: 'low',
    lane_pattern: 'utility',
    crowd_control: 5,
    engage: 5,
    scaling: 3,
    durability: 4,
    mobility: 2,
  },
  {
    id: 'ahri',
    name: 'Ahri',
    roles: ['mid'],
    role_confidence_map: roleConfidence('mid'),
    identities: ['pick', 'skirmish'],
    damage_profile: 'magical',
    execution_demand: 'medium',
    lane_pattern: 'roaming',
    crowd_control: 3,
    engage: 2,
    scaling: 3,
    durability: 1,
    mobility: 4,
  },
  {
    id: 'smolder',
    name: 'Smolder',
    roles: ['bottom'],
    role_confidence_map: roleConfidence('bottom'),
    identities: ['front_to_back', 'protect_carry'],
    damage_profile: 'physical',
    execution_demand: 'medium',
    lane_pattern: 'scaling',
    crowd_control: 1,
    engage: 0,
    scaling: 5,
    durability: 1,
    mobility: 2,
  },
  {
    id: 'nautilus',
    name: 'Nautilus',
    roles: ['support'],
    role_confidence_map: roleConfidence('support'),
    identities: ['engage', 'pick'],
    damage_profile: 'magical',
    execution_demand: 'low',
    lane_pattern: 'utility',
    crowd_control: 5,
    engage: 5,
    scaling: 2,
    durability: 4,
    mobility: 1,
  },
  {
    id: 'renata',
    name: 'Renata Glasc',
    roles: ['support'],
    role_confidence_map: roleConfidence('support'),
    identities: ['protect_carry', 'pick'],
    damage_profile: 'magical',
    execution_demand: 'medium',
    lane_pattern: 'utility',
    crowd_control: 3,
    engage: 1,
    scaling: 3,
    durability: 2,
    mobility: 1,
  },
  {
    id: 'jax',
    name: 'Jax',
    roles: ['top'],
    role_confidence_map: roleConfidence('top'),
    identities: ['split_push', 'skirmish'],
    damage_profile: 'mixed',
    execution_demand: 'medium',
    lane_pattern: 'scaling',
    crowd_control: 2,
    engage: 2,
    scaling: 4,
    durability: 3,
    mobility: 3,
  },
];

export const defaultConfig: TeamConfig = {
  teamName: 'Drafting Laboratory',
  patch: '15.6',
  preferredStyle: 'Front-to-back propre avec enclenchement fort',
  targetIdentity: ['engage', 'front_to_back', 'protect_carry'],
  members: [
    {
      id: 'top-alto',
      handle: 'Alto',
      role: 'top',
      comfortPicks: ['Ornn', 'Jax', 'Ksante'],
      focus: 'frontline stable',
    },
    {
      id: 'jungle-mira',
      handle: 'Mira',
      role: 'jungle',
      comfortPicks: ['Sejuani', 'Vi', 'Poppy'],
      focus: 'setup engage',
    },
    {
      id: 'mid-zen',
      handle: 'Zen',
      role: 'mid',
      comfortPicks: ['Ahri', 'Orianna', 'Taliyah'],
      focus: 'prio mid + picks',
    },
    {
      id: 'adc-lyra',
      handle: 'Lyra',
      role: 'bottom',
      comfortPicks: ['Smolder', 'Jinx', 'Zeri'],
      focus: 'late game carry',
    },
    {
      id: 'sup-kai',
      handle: 'Kai',
      role: 'support',
      comfortPicks: ['Nautilus', 'Renata', 'Rell'],
      focus: 'vision control',
    },
  ],
};

export const defaultDraftState: DraftState = {
  patch: '15.6',
  side: 'blue',
  phase: 'pick_phase_two',
  ally: {
    champions: championCatalog.filter((champion) => ['ornn', 'sejuani', 'ahri', 'smolder'].includes(champion.id)),
  },
  enemy: {
    champions: [
      {
        id: 'gnar',
        name: 'Gnar',
        roles: ['top'],
        role_confidence_map: roleConfidence('top'),
        identities: ['poke', 'front_to_back'],
        damage_profile: 'physical',
        execution_demand: 'medium',
        lane_pattern: 'bully',
        crowd_control: 3,
        engage: 2,
        scaling: 3,
        durability: 3,
        mobility: 3,
      },
      {
        id: 'xin-zhao',
        name: 'Xin Zhao',
        roles: ['jungle'],
        role_confidence_map: roleConfidence('jungle'),
        identities: ['engage', 'skirmish'],
        damage_profile: 'physical',
        execution_demand: 'low',
        lane_pattern: 'roaming',
        crowd_control: 2,
        engage: 4,
        scaling: 2,
        durability: 3,
        mobility: 3,
      },
      {
        id: 'azir',
        name: 'Azir',
        roles: ['mid'],
        role_confidence_map: roleConfidence('mid'),
        identities: ['poke', 'front_to_back'],
        damage_profile: 'magical',
        execution_demand: 'high',
        lane_pattern: 'scaling',
        crowd_control: 2,
        engage: 1,
        scaling: 5,
        durability: 1,
        mobility: 2,
      },
    ],
  },
  ally_bans: ['kalista', 'corki', 'skarner'],
  enemy_bans: ['rell', 'vi', 'orianna'],
  contested_roles: ['support'],
};

function candidate(
  championId: string,
  finalScore: number,
  bonuses: string[],
  risks: string[],
  winCondition: string,
  draftCall: string,
): CandidateEvaluation {
  const champion = championCatalog.find((entry) => entry.id === championId)!;
  const makeSubScore = (score: number, bonusLabels: string[], riskLabels: string[]) => ({
    raw_score: score,
    weight: 0.2,
    weighted_score: score * 0.2,
    contributors: [
      ...bonusLabels.map((label) => ({
        dimension: 'internal_coherence' as const,
        polarity: 'bonus' as const,
        label,
        value: 8,
        detail: `${label} renforce immédiatement la lecture rapide de la draft.`,
      })),
      ...riskLabels.map((label) => ({
        dimension: 'enemy_matchup' as const,
        polarity: 'malus' as const,
        label,
        value: -5,
        detail: `${label} demande une couverture claire dans les prochains tours.`,
      })),
    ],
  });

  return {
    champion,
    score_breakdown: {
      internal_coherence: makeSubScore(finalScore + 4, bonuses, []),
      enemy_matchup: makeSubScore(finalScore - 2, bonuses.slice(0, 1), risks),
      roster_fit: makeSubScore(finalScore + 1, bonuses.slice(0, 2), []),
      execution_simplicity: makeSubScore(finalScore - 5, bonuses.slice(0, 1), risks.slice(0, 1)),
      lane_stability: makeSubScore(finalScore - 3, bonuses.slice(0, 1), risks.slice(0, 1)),
      final_score: finalScore,
    },
    alerts: risks.map((risk, index) => ({
      category: index === 0 ? 'roster' : 'matchup',
      severity: index === 0 ? 'warning' : 'info',
      code: `${championId}-risk-${index}`,
      title: risk,
      detail: `${risk} à monitorer si le pick est verrouillé.`,
    })),
    explanation: bonuses,
    win_condition_after_pick: winCondition,
    draft_call: draftCall,
  };
}

export const defaultRecommendations: RecommendationResponse = {
  composition: {
    identities: ['engage', 'front_to_back', 'pick'],
    profile: {
      engage: 4,
      disengage: 1,
      pick: 3,
      poke: 0,
      front_to_back: 4,
      split_push: 0,
      scaling: 3,
      wave_clear: 3,
    },
    alerts: [
      {
        category: 'roster',
        severity: 'warning',
        code: 'support-open',
        title: 'Support encore ouvert',
        detail: 'La dernière rotation doit fermer l’accès engage + peel sans casser la courbe.',
      },
    ],
  },
  top_5: [
    candidate(
      'nautilus',
      89,
      ['Engage immédiat', 'Confort élevé pour Kai', 'Bloque les fenêtres Azir'],
      ['Expose un carry à la range adverse', 'Peut surcharger les timings d’entrée'],
      'Jouer la vision avancée et forcer les fights autour du premier point de contact.',
      'Call: verrouiller Nautilus si la priorité est d’avoir un bouton engage simple.',
    ),
    candidate(
      'renata',
      84,
      ['Peel premium sur Smolder', 'Très bon anti-dive', 'Maintient un plan front-to-back'],
      ['Moins d’initiation primaire', 'Fenêtres de catch plus rares'],
      'Temporiser, protéger Smolder, puis retourner l’engage adverse avec les ultimes.',
      'Call: Renata si l’adversaire montre déjà suffisamment d’initiative.',
    ),
    candidate(
      'jax',
      76,
      ['Ouvre une side lane forte', 'Punition de Gnar sur temps faibles', 'Ajoute du split push'],
      ['Rend la composition moins lisible', 'Exécution macro plus exigeante'],
      'Étendre la carte et jouer les side waves avant de converger sur objectif.',
      'Call: seulement si le plan de jeu bascule vers 1-3-1.',
    ),
  ],
  evaluated_candidates: [],
};
defaultRecommendations.evaluated_candidates = defaultRecommendations.top_5;

export const defaultDiagnosis: DraftDiagnosis = {
  composition: defaultRecommendations.composition,
  win_condition:
    'Win condition principale : sécuriser la vision des flancs puis déclencher un front-to-back net autour de Smolder.',
  draft_call:
    'Review : draft stable, lecture simple, et priorité à un support qui apporte engage ou anti-dive selon le dernier pick ennemi.',
  review_notes: [
    'Le coeur Ornn + Sejuani + Smolder donne une colonne vertébrale très lisible en teamfight.',
    'Ahri garde l’accès aux picks sans compromettre la couverture de lane.',
    'Le dernier slot doit arbitrer entre engage primaire et protection du carry.',
  ],
};

export const historyEntries: HistoryEntry[] = [
  {
    id: 'review-1',
    patch: '15.6',
    opponent: 'Team Solaris',
    result: 'win',
    reviewHeadline: 'Draft gagnante grâce à une identity engage/peel cohérente.',
    draftCall: 'Front-to-back propre, objectif contrôlé au troisième dragon.',
  },
  {
    id: 'review-2',
    patch: '15.5',
    opponent: 'Night Owls',
    result: 'loss',
    reviewHeadline: 'Comp trop exigeante mécaniquement pour une exécution sur scène.',
    draftCall: 'Le split push n’a jamais trouvé de timings propres.',
  },
];

export const defaultSandboxResponse: SandboxSimulationResponse = {
  baseline: defaultRecommendations,
  scenarios: [
    {
      id: 'safe-engage',
      label: 'Safe engage',
      pick: 'Nautilus',
      projected_score: 90,
      summary: 'Stabilise les fights, ferme le setup vision, et garde un call unique.',
      risks: ['Prévisible si l’adversaire garde cleanse', 'Faible portée si le tempo ralentit'],
    },
    {
      id: 'protect-carry',
      label: 'Protect carry',
      pick: 'Renata Glasc',
      projected_score: 86,
      summary: 'Augmente la sécurité de Smolder et améliore la lecture défensive.',
      risks: ['Peu de hard engage', 'Nécessite de laisser venir'],
    },
    {
      id: 'macro-split',
      label: 'Macro split',
      pick: 'Jax',
      projected_score: 77,
      summary: 'Variante de laboratoire pour tester un plan 1-3-1 et side priority.',
      risks: ['Calls plus complexes', 'Moins bon 5v5 immédiat'],
    },
  ],
};

export function identityLabel(identity: CompositionIdentity): string {
  return identity.split('_').join(' ');
}
