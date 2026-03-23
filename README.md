# Draft Team App Monorepo

Ce dépôt fournit la structure de base du projet **Draft Team App** avec :

- une application desktop dans `apps/desktop` basée sur **Vite + React 18 + TypeScript** ;
- un backend desktop dans `apps/desktop/src-tauri` basé sur **Tauri v2 + Rust** ;
- des packages partagés dans `packages/` ;
- de la documentation dans `docs/` ;
- des scripts d'import et de seed dans `scripts/`.

## Structure

```text
apps/
  desktop/
    src/
      app/
      components/
      features/
      hooks/
      lib/
      pages/
      stores/
      types/
    src-tauri/
      src/
        adapters/
        application/
        commands/
        domain/
        services/
        storage/
        tests/
packages/
  champion-data/
  shared-types/
  team-config-schema/
docs/
  architecture/
  setup/
scripts/
  import_ddragon/
  seed_data/
```

## Tooling inclus

- **ESLint** : `eslint.config.js`
- **Prettier** : `.prettierrc.json`
- **Vitest** : `vitest.config.ts`
- **rustfmt** : `rustfmt.toml`
- **clippy** : `.clippy.toml`
- **npm workspaces** via `package.json`
- **Cargo workspace** via `Cargo.toml`

## Démarrage rapide

```bash
npm install
npm run dev:desktop
```

Pour la partie Tauri/Rust :

```bash
cargo check
```

## Notes

Cette base pose la structure et les fichiers de configuration minimaux pour démarrer l'implémentation métier.
