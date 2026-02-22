# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Database Snapshot Manager — a Tauri 2 desktop application providing git-like version control for local development databases. Currently at MVP stage: snapshot/restore operations are simulated in the UI with localStorage persistence (no real `mysqldump`/`pg_dump` execution yet).

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

- **`src/main.ts`** — Single-file vanilla TypeScript frontend. All UI logic lives here: type definitions (`ConnectionProfile`, `Snapshot`), state management, DOM rendering, and localStorage persistence (`dsm.profiles.v1`, `dsm.snapshots.v1`). No framework — direct DOM manipulation.
- **`src-tauri/src/lib.rs`** — Rust backend entry point. Currently minimal (only a `greet` test command). Future home of subprocess execution (database dumps), credential management, and file I/O.
- **`src-tauri/src/main.rs`** — Thin launcher that calls `lib.rs::run()`.
- **`src-tauri/tauri.conf.json`** — Tauri config: window size (800×600), bundle targets, dev server URL, build commands.

**Data flow:** Frontend manages all state in-memory arrays, persists to localStorage. Rust backend is scaffolded but not yet used for data operations.

## Key Technical Decisions

- **No frontend framework** — vanilla TypeScript with direct DOM manipulation
- **ES modules** (`"type": "module"` in package.json), target ES2020
- **TypeScript strict mode** enabled with `noUnusedLocals` and `noUnusedParameters`
- **macOS only** target for Phase 1
- **localStorage** for MVP persistence; SQLite metadata backend planned for production

## Reference Documents

- `DATABASE-SNAPSHOT-MANAGER.md` — Full product concept, technical architecture, phased roadmap, risk assessment, and open questions
- `BRANDING.md` — Brand identity, naming candidates (primary: "Timemachine"), colour palette, visual direction
