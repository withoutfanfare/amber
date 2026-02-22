# Phase 1 Full Implementation Design

> Database Snapshot Manager — from simulated MVP to working snapshot/restore tool

**Date:** 19 February 2026
**Status:** Approved
**Scope:** Full Phase 1 — real database operations, Vue 3 frontend rebuild, macOS only

---

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Frontend framework | Vue 3 + Tailwind CSS v4 | Matches Spool design system in Styleguide.md, scales for Phase 2+ features |
| Implementation approach | Vertical slices | Each feature delivered end-to-end (Vue + Rust), testable as built |
| Credential storage | macOS Keychain via tauri-plugin-keyring | Full OS keychain integration for secure credential management |
| Snapshot compression | gzip (flate2 crate) | Simple, well-supported, good enough for SQL dumps |
| UI polish level | Full Spool design system | Ambient blobs, glassmorphic panels, all component variants from day one |
| SSH tunnel support | Yes — transparent per-profile | `ssh -L` subprocess managed by Rust backend |

---

## Architecture

### Frontend

- **Vue 3** with Composition API and `<script setup>` syntax
- **Vue Router** for page navigation
- **Pinia** for state management (profiles, snapshots, settings stores)
- **Tailwind CSS v4** with design tokens from Styleguide.md
- Full Spool design system: ambient background blobs, glassmorphic cards, all shared components

### Backend (Rust)

- **SQLite metadata store** via `rusqlite` — profiles, snapshot metadata, settings
- **Credential storage** via `tauri-plugin-keyring` — macOS Keychain
- **Subprocess execution** — `std::process::Command` for `mysqldump`, `pg_dump`, `sqlite3 VACUUM INTO`
- **File I/O** — gzip compression via `flate2` crate
- **SSH tunnels** — `std::process::Command` spawning `ssh -L` for remote connections

### Storage Layout

```text
~/Library/Application Support/DatabaseSnapshotManager/
    metadata.db              (SQLite metadata store)
    snapshots/
        [project-slug]/
            [snapshot-id].sql.gz
    tmp/                     (cleaned on startup)
```

### Tauri IPC Commands

- `profile_create`, `profile_update`, `profile_delete`, `profile_list`, `profile_test_connection`
- `snapshot_create`, `snapshot_list`, `snapshot_restore`, `snapshot_delete`
- `storage_usage`

---

## Data Model

### SQLite Schema (`metadata.db`)

```sql
CREATE TABLE profiles (
    id              TEXT PRIMARY KEY,
    project         TEXT NOT NULL,
    name            TEXT NOT NULL,
    db_type         TEXT NOT NULL CHECK(db_type IN ('mysql','postgresql','sqlite')),
    host            TEXT,
    port            INTEGER,
    database_name   TEXT NOT NULL,
    username        TEXT,
    ssh_enabled     INTEGER DEFAULT 0,
    ssh_host        TEXT,
    ssh_port        INTEGER DEFAULT 22,
    ssh_user        TEXT,
    notes           TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE snapshots (
    id                TEXT PRIMARY KEY,
    profile_id        TEXT NOT NULL REFERENCES profiles(id),
    name              TEXT NOT NULL,
    note              TEXT,
    file_path         TEXT NOT NULL,
    size_bytes        INTEGER NOT NULL,
    db_version        TEXT,
    dump_tool_version TEXT,
    created_at        TEXT NOT NULL,
    restored_at       TEXT
);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### Keychain Entries

- Service: `com.dsm.profile.{profile_id}`
- Value: JSON `{"password": "...", "ssh_key_path": "...", "ssh_password": "..."}`

### Pinia Stores

- `useProfileStore` — profiles CRUD, active profile selection
- `useSnapshotStore` — snapshots for active profile, create/restore/delete actions
- `useSettingsStore` — app preferences

---

## UI Components & Routes

### Routes

| Path | View | Description |
|---|---|---|
| `/` | `DashboardView` | KPI cards, recent snapshots, quick actions |
| `/profiles` | `ProfilesView` | Profile list with CRUD |
| `/profiles/create` | `ProfileCreateView` | New profile form with connection test |
| `/profiles/:id/edit` | `ProfileEditView` | Edit profile |
| `/snapshots` | `SnapshotsView` | Snapshot browser for active profile |
| `/snapshots/create` | `SnapshotCreateView` | Create snapshot form |
| `/storage` | `StorageView` | Disk usage breakdown, pruning controls |
| `/settings` | `SettingsView` | App preferences |

### Layout Components

- `AppShell` — titlebar drag region, sidebar, main content area, ambient blobs
- `AppSidebar` — nav links, profile selector dropdown, service status footer
- `PageHeader` — sticky header with title + action slots

### Shared Components (Spool Design System)

- `Button`, `Badge`, `Card`, `FormInput`, `FormSelect`, `FormTextarea`
- `ConfirmDialog`, `ToastNotification`, `EmptyState`, `Skeleton`

### Feature Components

- `ProfileCard` — connection details, test status, edit/delete actions
- `SnapshotTable` — sortable table with name, note, date, size, actions
- `SnapshotRestoreDialog` — confirmation with snapshot details preview
- `StorageBreakdown` — per-project disk usage chart
- `ConnectionTestIndicator` — live test result with spinner/success/error
- `SshTunnelConfig` — SSH host/port/user/key fields, toggled by checkbox

### Key Interaction Flows

1. **Create snapshot:** Select profile -> name + note -> progress bar (Tauri event stream) -> success toast
2. **Restore:** Click restore on snapshot row -> ConfirmDialog with details -> progress bar -> success toast
3. **Test connection:** Click test on profile -> spinner -> success/error badge (SSH tunnel opened transparently if configured)

---

## Error Handling

- **Rust backend:** All Tauri commands return `Result<T, String>`. Errors include raw command output. Categories: `ConnectionError`, `DumpError`, `RestoreError`, `FileSystemError`, `KeychainError`.
- **Frontend:** Pinia actions catch errors and surface via toast system. Connection test errors inline on profile form. Snapshot/restore errors in detail dialog with copyable raw output.
- **SSH tunnels:** 10-second timeout. If tunnel drops mid-operation, dump/restore fails naturally with error surfaced.
- **Missing tools:** Check for `mysqldump`/`pg_dump`/`sqlite3` on PATH. Actionable error with Homebrew install command.

---

## Testing

- **Rust unit tests:** SQLite operations, command construction, compression, SSH command construction.
- **Rust integration tests:** Actual dump/restore against local test databases (document setup in README).
- **Frontend:** Vitest for Pinia stores and utility functions. Component tests for critical flows.
- **E2E:** Not in Phase 1. Manual testing against real databases.

---

## Implementation Order (Vertical Slices)

1. **Foundation** — Vue 3 + Tailwind + Spool design system scaffold, Rust SQLite + keychain setup
2. **Connection Profiles** — full CRUD, credential storage, connection testing, SSH tunnel config
3. **Snapshot Creation** — subprocess handling, progress reporting, gzip compression, file storage
4. **Snapshot Browser + Restore** — list, search, sort, restore with confirmation dialog
5. **Storage Management** — usage view per project, deletion, pruning controls
6. **Dashboard** — KPI cards, recent activity, quick actions

---

*Design approved 19 February 2026*
