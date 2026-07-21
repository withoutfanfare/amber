# Database Snapshot Manager

<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" alt="Amber app icon" width="144" height="144">
</p>

> Git-like version control for local development databases. Built with Tauri 2 + Rust for developers who need confidence before every migration.

## Features

- **Connection profiles** — save and manage MySQL, PostgreSQL, and SQLite connections with per-project organisation.
- **Named snapshots** — create compressed snapshots with custom names and notes using `mysqldump`, `pg_dump`, or SQLite `VACUUM INTO`.
- **Real-time progress** — streaming progress updates during snapshot creation and restore via Tauri IPC channels.
- **Flexible restore** — restore to the original profile or override the target profile and/or database name. Cross-type restores (e.g. MySQL to PostgreSQL) are rejected with a clear error.
- **SSH tunnel support** — snapshot and restore through SSH tunnels for remote development databases.
- **Secure credentials** — passwords and SSH keys stored in the macOS Keychain, never in plaintext.
- **Storage management** — per-project disk usage breakdown with manual deletion.
- **Dashboard** — KPI cards showing profile count, snapshot count, total storage, and last restore.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2 |
| Backend | Rust (tokio, rusqlite, keyring) |
| Frontend | Vue 3, Pinia, Vue Router, Tailwind CSS 4 |
| Build tooling | Vite 6, vue-tsc |
| Testing | Vitest (frontend), cargo test (backend) |

## Development Commands

```bash
npm install                # Install frontend dependencies
npm run tauri dev          # Full-stack development (Rust backend + Vite frontend)
npm run tauri build        # Production build
npm run dev                # Frontend-only Vite dev server (port 1420)
npm run test               # Run frontend Vitest tests
npm run test:watch         # Run tests in watch mode
npm run typecheck          # TypeScript type checking (vue-tsc --noEmit)
npm run lint               # ESLint
npm run format             # Prettier formatting
```

Rust tests:

```bash
cd src-tauri && cargo test
```

## Architecture

```bash
src/                          # Vue 3 frontend
  components/
    features/                 # Domain components (ProfileCard, SnapshotTable, etc.)
    layout/                   # AppSidebar, PageHeader
    ui/                       # Reusable primitives (Button, FormInput, ConfirmDialog, etc.)
  composables/                # useFocusTrap, useToast
  stores/                     # Pinia stores (profiles, snapshots, settings)
  views/                      # Route-level views
  types/                      # TypeScript interfaces
  test/                       # Test setup and helpers

src-tauri/src/                # Rust backend
  lib.rs                      # Application bootstrap, Tauri plugin and command registration
  commands.rs                 # Tauri IPC command handlers (profiles, snapshots, storage, settings)
  db.rs                       # SQLite metadata store initialisation and migrations
  credentials.rs              # macOS Keychain credential storage
  dump.rs                     # Subprocess command builders for dump/restore tools
  ensure_db.rs                # Database creation helpers for flexible restore
  ssh.rs                      # SSH tunnel management
  compress.rs                 # Gzip compression/decompression streaming
  progress.rs                 # Snapshot progress event types
  error.rs                    # Structured error enum with JSON serialisation
```

### Data Flow

**Snapshot creation:**
1. Frontend calls `invoke('snapshot_create', { profileId, name, note, onProgress })`
2. Rust resolves the profile, decrypts credentials, opens SSH tunnel if needed
3. Rust executes `mysqldump` / `pg_dump` / `sqlite3 VACUUM INTO` as a subprocess
4. Output is gzip-compressed and stored under `~/Library/Application Support/...`
5. Metadata is recorded in the local SQLite store

**Snapshot restore (with optional target override):**
1. Frontend calls `invoke('snapshot_restore', { snapshotId, options, onProgress })`
2. Rust resolves the effective profile (original or overridden target)
3. If overriding the database name, Rust ensures the target database exists first
4. Rust decompresses the snapshot and pipes it into `mysql` / `psql` or copies to the SQLite path
5. `restored_at` timestamp is updated on the snapshot metadata

## Reference Documents

- `DATABASE-SNAPSHOT-MANAGER.md` — full product concept, technical architecture, phased roadmap
- `BRANDING.md` — brand identity, naming candidates, colour palette
- `docs/plans/` — detailed implementation plans for Phase 1
