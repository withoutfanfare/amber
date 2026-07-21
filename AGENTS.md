# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Database Snapshot Manager — a Tauri 2 desktop application providing git-like version control for local development databases. Production-ready with real `mysqldump`/`pg_dump`/`sqlite3` subprocess execution, SQLite metadata persistence, and comprehensive snapshot management (tagging, retention policies, schema comparison, version compatibility, export).

## Development Commands

```bash
npm install                # Install frontend dependencies
npm run tauri dev          # Full-stack development (Rust backend + Vite frontend)
npm run tauri build        # Production build
npm run dev                # Frontend-only Vite dev server (port 1420)
npm run build              # TypeScript compilation only
```

There are no test, lint, or formatting commands configured.

## Architecture

**Tauri 2 hybrid app:** TypeScript frontend + Rust backend communicating via Tauri IPC.

- **`src/main.ts`** — Vue 3 + Pinia entry point. Frontend uses Vue SFCs with TypeScript, @stuntrocket/ui design system components, and Tauri IPC for backend communication.
- **`src-tauri/src/lib.rs`** — Rust backend entry point. Manages SQLite metadata DB, subprocess execution for database dumps/restores, credential storage via macOS Keychain, SSH tunnelling, and all Tauri commands.
- **`src-tauri/src/main.rs`** — Thin launcher that calls `lib.rs::run()`.
- **`src-tauri/tauri.conf.json`** — Tauri config: window size (800×600), bundle targets, dev server URL, build commands.

**Data flow:** Frontend uses Pinia stores that call Rust backend via `invoke()`. Rust backend manages SQLite metadata, credential storage, subprocess execution, and file I/O.

## Key Technical Decisions

- **Vue 3 + Pinia** — migrated from vanilla TypeScript; uses @stuntrocket/ui design system components
- **ES modules** (`"type": "module"` in package.json), target ES2020
- **TypeScript strict mode** enabled with `noUnusedLocals` and `noUnusedParameters`
- **macOS only** target for Phase 1
- **SQLite** for metadata persistence (profiles, snapshots, settings, tags, retention policies)

## Reference Documents

- `DATABASE-SNAPSHOT-MANAGER.md` — Full product concept, technical architecture, phased roadmap, risk assessment, and open questions
- `BRANDING.md` — Brand identity, naming candidates (primary: "Timemachine"), colour palette, visual direction
