import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

export interface ChampionPatchRecord {
  id: string;
  key: string;
  name: string;
  patch: string;
  compatibleSince: string;
  roles: string[];
  archetypes: string[];
  strengths: string[];
  weaknesses: string[];
  incomplete_profile: boolean;
}

export interface ChampionResolution {
  champion: ChampionPatchRecord;
  requestedPatch: string;
  resolvedPatch: string;
  fallbackApplied: boolean;
}

const currentDir = dirname(fileURLToPath(import.meta.url));
const patchesDir = join(currentDir, 'patches');

function comparePatches(left: string, right: string): number {
  const leftParts = left.split('.').map((value) => Number.parseInt(value, 10));
  const rightParts = right.split('.').map((value) => Number.parseInt(value, 10));
  const maxLength = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < maxLength; index += 1) {
    const diff = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (diff !== 0) {
      return diff;
    }
  }

  return 0;
}

function listAvailablePatches(): string[] {
  return readdirSync(patchesDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort((left, right) => comparePatches(right, left));
}

function readChampionsForPatch(patch: string): ChampionPatchRecord[] {
  const championsDir = join(patchesDir, patch, 'champions');

  return readdirSync(championsDir)
    .filter((fileName) => fileName.endsWith('.json'))
    .sort((left, right) => left.localeCompare(right))
    .map((fileName) => {
      const filePath = join(championsDir, fileName);
      const record = JSON.parse(readFileSync(filePath, 'utf8')) as ChampionPatchRecord;
      return record;
    });
}

export const championDataByPatch = new Map(
  listAvailablePatches().map((patch) => [patch, readChampionsForPatch(patch)]),
);

export function getAvailablePatches(): string[] {
  return [...championDataByPatch.keys()];
}

export function resolveChampionByPatch(
  championId: string,
  requestedPatch: string,
): ChampionResolution | null {
  const sortedPatches = getAvailablePatches().sort((left, right) => comparePatches(right, left));

  for (const patch of sortedPatches) {
    if (comparePatches(patch, requestedPatch) > 0) {
      continue;
    }

    const candidate = championDataByPatch
      .get(patch)
      ?.find((champion) => champion.id === championId || champion.key === championId);

    if (candidate) {
      return {
        champion: candidate,
        requestedPatch,
        resolvedPatch: patch,
        fallbackApplied: patch !== requestedPatch,
      };
    }
  }

  return null;
}

export function listChampionsForPatch(requestedPatch: string): ChampionResolution[] {
  const resolvedPatch = getAvailablePatches().find(
    (patch) => comparePatches(patch, requestedPatch) <= 0,
  );

  if (!resolvedPatch) {
    return [];
  }

  return (championDataByPatch.get(resolvedPatch) ?? []).map((champion) => ({
    champion,
    requestedPatch,
    resolvedPatch,
    fallbackApplied: resolvedPatch !== requestedPatch,
  }));
}
