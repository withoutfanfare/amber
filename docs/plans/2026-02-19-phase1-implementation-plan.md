# Phase 1 Implementation Plan

> Database Snapshot Manager — from simulated MVP to working snapshot/restore tool

**Date:** 19 February 2026
**Status:** Ready for implementation
**Design doc:** [Phase 1 Design](./2026-02-19-phase1-full-implementation-design.md)
**Detail plans:** [Backend](./backend-plan.md) | [Frontend](./frontend-plan.md) | [Testing](./testing-plan.md)

---

## Prerequisites

- Node.js 22+, npm
- Rust stable toolchain with `clippy` and `rustfmt` components
- macOS (Phase 1 target platform)
- MySQL and/or PostgreSQL installed locally for testing

---

## Step 1: Project Foundation — Tooling & Dependencies

### 1.1 Install frontend dependencies

```bash
# Vue 3 core
npm install vue vue-router pinia @tauri-apps/api @tauri-apps/plugin-opener

# Build tooling
npm install -D @vitejs/plugin-vue vue-tsc tailwindcss @tailwindcss/vite @tauri-apps/cli vite typescript

# Testing & linting
npm install -D vitest @vue/test-utils happy-dom @pinia/testing
npm install -D eslint @eslint/js eslint-plugin-vue typescript-eslint globals eslint-config-prettier
npm install -D prettier prettier-plugin-tailwindcss
npm install -D lefthook
```

### 1.2 Update Rust dependencies

**File:** `src-tauri/Cargo.toml`

Replace dependencies block with:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
rusqlite_migration = "1"
flate2 = "1.1"
uuid = { version = "1", features = ["v4"] }
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
keyring = { version = "3", features = ["apple-native"] }
fix-path-env = { git = "https://github.com/tauri-apps/fix-path-env-rs" }
tokio = { version = "1", features = ["process", "io-util", "time", "fs"] }
which = "7"

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
```

### 1.3 Create config files

| File | Content |
|---|---|
| `eslint.config.js` | ESLint v9 flat config (TypeScript + Vue 3 + Prettier) |
| `.prettierrc` | `{ "semi": true, "singleQuote": false, "tabWidth": 2, "trailingComma": "all", "printWidth": 100, "plugins": ["prettier-plugin-tailwindcss"] }` |
| `.prettierignore` | `dist/`, `src-tauri/target/`, `node_modules/`, `*.md` |
| `lefthook.yml` | Pre-commit: eslint, prettier, typecheck, rustfmt, clippy. Pre-push: tests |
| `src-tauri/rustfmt.toml` | `edition = "2021"`, `max_width = 100` |
| `.vscode/extensions.json` | tauri-vscode, rust-analyzer, Volar, ESLint, Prettier, Tailwind IntelliSense |
| `.vscode/settings.json` | Format on save, flat ESLint config |
| `.github/workflows/ci.yml` | Two parallel jobs: Rust checks + Frontend checks on macos-latest |
| `tsconfig.node.json` | Node-specific TS config for vite.config.ts |
| `src/env.d.ts` | Vue SFC type declarations |

See [testing-plan.md](./testing-plan.md) sections 4-7 for exact file contents.

### 1.4 Update vite.config.ts

Add Vue plugin, Tailwind v4 plugin, `@` path alias. See [frontend-plan.md](./frontend-plan.md) section 1.3.

### 1.5 Update tsconfig.json

Add `jsx: "preserve"`, `paths: { "@/*": ["src/*"] }`, include `src/**/*.vue`. See [frontend-plan.md](./frontend-plan.md) section 1.4.

### 1.6 Update package.json scripts

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix",
    "format": "prettier --write \"src/**/*.{ts,vue,css,html}\"",
    "format:check": "prettier --check \"src/**/*.{ts,vue,css,html}\"",
    "typecheck": "vue-tsc --noEmit",
    "postinstall": "lefthook install"
  }
}
```

### 1.7 Verify

- `npm run dev` starts Vite with Vue plugin
- `cd src-tauri && cargo check` compiles with new dependencies
- `npm run lint` and `npm run format:check` pass
- `cd src-tauri && cargo clippy` passes

---

## Step 2: Foundation — Rust Backend Scaffold

### 2.1 Create Rust module structure

```bash
src-tauri/src/
    lib.rs              # Tauri app builder, state setup, command registration
    main.rs             # Thin launcher (already exists)
    error.rs            # DsmError enum with thiserror + Serialize
    db.rs               # SQLite init, migrations, DbState
    credentials.rs      # Keychain store/get/delete via keyring crate
    dump.rs             # Subprocess command builders for dump/restore
    ssh.rs              # SshTunnel struct
    compress.rs         # gzip streaming helpers
    progress.rs         # SnapshotProgress enum for Channel events
    commands.rs         # All #[tauri::command] functions + IPC data types
```

### 2.2 Implement error.rs

`DsmError` enum with variants: `ConnectionError`, `DumpError`, `RestoreError`, `FileSystemError`, `KeychainError`, `DatabaseError`, `SshError`, `ToolNotFound`, `SerdeError`, `ProfileNotFound`, `SnapshotNotFound`. Custom `Serialize` impl producing `{kind, message}` JSON.

See [backend-plan.md](./backend-plan.md) section 8.

### 2.3 Implement db.rs

- `init_db()` — open/create SQLite, enable WAL + foreign keys, run migrations
- `DbState(Mutex<Connection>)` managed state
- `AppPaths { data_dir, snapshots_dir, tmp_dir }` managed state
- Embedded schema migration via `rusqlite_migration`

See [backend-plan.md](./backend-plan.md) section 2.

### 2.4 Implement credentials.rs

- `store_credentials(profile_id, ProfileCredentials)` — JSON to macOS Keychain
- `get_credentials(profile_id)` — retrieve from Keychain
- `delete_credentials(profile_id)` — remove from Keychain
- Service format: `com.dsm.profile.{id}`

See [backend-plan.md](./backend-plan.md) section 3.

### 2.5 Implement progress.rs

`SnapshotProgress` tagged enum with `Started`, `Phase`, `Progress`, `Completed`, `Failed` variants. Used with Tauri `Channel<SnapshotProgress>`.

See [backend-plan.md](./backend-plan.md) section 7.

### 2.6 Implement commands.rs — data types only

Define all IPC structs: `Profile`, `CreateProfileInput`, `UpdateProfileInput`, `Snapshot`, `ConnectionTestResult`, `StorageInfo`, `ProjectStorage`. Stub all command functions with `todo!()`.

See [backend-plan.md](./backend-plan.md) section 9.

### 2.7 Update lib.rs

- Call `fix_path_env::fix()` at startup
- In `setup()`: init DB, create directories, clean tmp, register managed state
- Register all commands in `invoke_handler`

See [backend-plan.md](./backend-plan.md) section 2 (connection management).

### 2.8 Verify

- `cd src-tauri && cargo check` — compiles
- `cd src-tauri && cargo test` — db unit tests pass (schema creation, insert, constraint checks)

---

## Step 3: Foundation — Vue 3 Frontend & Design System

### 3.1 Create main.css with Spool design tokens

**File:** `src/assets/main.css`

- `@import "tailwindcss"`
- `@theme {}` block with ALL tokens: surfaces, text, accent, status, borders, typography (sm=13px, base=15px, lg=17px), radii, shadows, animations
- Global classes: `.card`, `.card-inset`, `.sidebar`, `.page-header`, `.backdrop`, `.modal`, `.segment-chip`, `.control-field-sm`
- Ambient blob CSS (3 blobs with drift animations + reduced motion)
- Stagger fade-in, titlebar drag region, scrollbar styling
- Base reset (15px font, antialiasing)

See [frontend-plan.md](./frontend-plan.md) sections 3-4 for exact CSS.

### 3.2 Create shared UI components

All in `src/components/ui/`:

| Component | Key props | Notes |
|---|---|---|
| `Button.vue` | variant, size, to, disabled, loading | 8 variants, 4 sizes, scoped CSS for solid base |
| `Badge.vue` | label, variant | 7 variants, `color-mix()` in scoped CSS |
| `Card.vue` | variant (default/interactive/flush/inset) | Wraps `.card` class |
| `FormInput.vue` | modelValue, label, type, size, error | Scoped `.form-input` CSS |
| `FormSelect.vue` | modelValue, label, options, size, error | Custom chevron, `appearance: none` |
| `FormTextarea.vue` | modelValue, label, rows, error | `min-height: 5rem`, `resize: vertical` |
| `ConfirmDialog.vue` | open, title, message, danger, typedConfirmation | Teleport, focus trap, Escape key |
| `ToastNotification.vue` | message, type, duration | Glassmorphic, ARIA roles, auto-dismiss |
| `EmptyState.vue` | message, actionLabel, icon | Centred layout, optional action button |
| `Skeleton.vue` | width, height, rounded | Pulse animation, semi-transparent |

See [frontend-plan.md](./frontend-plan.md) section 5 for full prop interfaces and implementation details.

### 3.3 Create layout components

**`src/components/layout/AppShell.vue`** — Root layout: ambient blobs, titlebar drag region, skip link, sidebar + main area, toast container. Provides toast context via `useToastProvider()`.

**`src/components/layout/AppSidebar.vue`** — `w-56`, `.sidebar` class, nav links (Dashboard, Profiles, Snapshots, Storage, Settings), active state: `text-accent font-semibold`. Inline SVG icons.

**`src/components/layout/PageHeader.vue`** — `.page-header` sticky header with `#prepend` and `#actions` slots.

See [frontend-plan.md](./frontend-plan.md) section 12 for template markup.

### 3.4 Create composables

**`src/composables/useFocusTrap.ts`** — Tab key trapping for modals, stores previously focused element.

**`src/composables/useToast.ts`** — provide/inject pattern. `ToastKey` injection key, `useToastProvider()` (in AppShell), `useToast()` (in components). Methods: `success()`, `error()`, `info()`, `dismiss()`.

See [frontend-plan.md](./frontend-plan.md) section 11.

### 3.5 Create types

**`src/types/index.ts`** — Shared TypeScript interfaces: `DbType`, `Profile`, `ProfileCreatePayload`, `ProfileUpdatePayload`, `Snapshot`, `SnapshotCreatePayload`, `StorageUsage`, `ConnectionTestResult`.

See [frontend-plan.md](./frontend-plan.md) section 7.1.

### 3.6 Create Vue Router

**`src/router/index.ts`** — 8 routes, all lazy-loaded via dynamic `import()`:
- `/` → DashboardView
- `/profiles` → ProfilesView
- `/profiles/create` → ProfileCreateView
- `/profiles/:id/edit` → ProfileEditView
- `/snapshots` → SnapshotsView
- `/snapshots/create` → SnapshotCreateView
- `/storage` → StorageView
- `/settings` → SettingsView

### 3.7 Create Pinia stores (stubs)

**`src/stores/profiles.ts`** — `useProfileStore` with state, getters, and action stubs.
**`src/stores/snapshots.ts`** — `useSnapshotStore` with state, getters, and action stubs.
**`src/stores/settings.ts`** — `useSettingsStore` with state and action stubs.

### 3.8 Bootstrap the app

**`src/main.ts`** — Create Vue app, install Pinia + Router, import main.css, mount to `#app`.

**`src/App.vue`** — Imports AppShell, adds visibility change listener for blob animation pause.

**`index.html`** — Clean up to minimal HTML5 with `<div id="app">`.

### 3.9 Create stub views

Create all 8 view files as minimal stubs with `PageHeader` and placeholder text. Enough to verify routing works.

### 3.10 Delete old files

Remove `src/main.ts` (old vanilla TS), `src/styles.css` (old MVP styles), old SVG assets if unused.

### 3.11 Verify

- `npm run tauri dev` — app launches, ambient blobs visible, sidebar navigates between stub views
- All UI components render correctly (create a temporary test page if needed)
- `npm run lint`, `npm run typecheck`, `npm run test` all pass

---

## Step 4: Connection Profiles (Full Vertical Slice)

### 4.1 Rust — Profile commands

**File:** `src-tauri/src/commands.rs` (replace stubs)

Implement:
- `profile_create` — generate UUID, insert into SQLite, store credentials in Keychain, return Profile
- `profile_update` — verify exists, update changed fields, update credentials if changed
- `profile_delete` — delete snapshots (files + metadata), delete credentials, delete profile
- `profile_list` — SELECT all profiles ordered by project, name
- `profile_test_connection` — load profile + credentials, open SSH tunnel if enabled, attempt DB connection (`mysql --execute="SELECT VERSION()"` / `psql -c "SELECT version()"`), measure latency, return result

### 4.2 Rust — Tool detection and SSH tunnels

Implement `find_tool()` in `dump.rs` using `which` crate.
Implement `SshTunnel::open()` and `SshTunnel::close()` in `ssh.rs`.

See [backend-plan.md](./backend-plan.md) sections 4-5.

### 4.3 Frontend — Profile store

**File:** `src/stores/profiles.ts`

Implement all actions with real `invoke()` calls: `fetchAll`, `create`, `update`, `remove`, `testConnection`, `setActive`.

See [frontend-plan.md](./frontend-plan.md) section 7.2.

### 4.4 Frontend — Profile components

**`src/components/features/ProfileCard.vue`** — Profile display with db_type badge, host:port/database, SSH indicator, test/edit/delete actions.

**`src/components/features/ProfileForm.vue`** — Shared create/edit form. Fields: project, name, db_type (select), host, port, database_name, username, password, notes. Includes SshTunnelConfig.

**`src/components/features/SshTunnelConfig.vue`** — SSH host, port, user, key path, password. Hidden until checkbox enabled.

**`src/components/features/ConnectionTestIndicator.vue`** — Spinner / green check + latency / red X + error.

### 4.5 Frontend — Profile views

**`src/views/ProfilesView.vue`** — PageHeader + "New Profile" button, ProfileCard list grouped by project, EmptyState when empty.

**`src/views/ProfileCreateView.vue`** — PageHeader + back button, ProfileForm in create mode, sticky footer with Cancel + Create.

**`src/views/ProfileEditView.vue`** — Same as create but pre-populated, delete button in header.

### 4.6 Frontend — Sidebar profile selector

Update AppSidebar to show active profile dropdown near bottom, using `useProfileStore`.

### 4.7 Tests

- **Rust:** Unit tests for profile CRUD operations against in-memory SQLite
- **Frontend:** Pinia store tests mocking invoke, ProfileCard component test

### 4.8 Verify

- Create a profile via the UI, verify it appears in the list
- Edit and delete profiles
- Test connection (requires a running MySQL/PostgreSQL)
- SSH tunnel config toggles correctly

---

## Step 5: Snapshot Creation (Full Vertical Slice)

### 5.1 Rust — Dump execution

**File:** `src-tauri/src/dump.rs`

Implement:
- `write_mysql_defaults_file()` — temp .cnf with 0600 permissions
- `build_mysqldump_command()` — `--defaults-extra-file`, `--single-transaction`, `--routines`, `--triggers`
- `build_pg_dump_command()` — `PGPASSWORD` env var, `--format=plain`, `--no-owner`
- `build_sqlite_dump_command()` — `sqlite3 VACUUM INTO`

See [backend-plan.md](./backend-plan.md) section 4.

### 5.2 Rust — Compression

**File:** `src-tauri/src/compress.rs`

Implement:
- `compress_from_reader()` — pipe subprocess stdout through `GzEncoder` to `.sql.gz`
- `decompress_to_writer()` — stream `.sql.gz` through `GzDecoder` to writer
- `compress_file()` — compress existing file

See [backend-plan.md](./backend-plan.md) section 6.

### 5.3 Rust — snapshot_create command

**File:** `src-tauri/src/commands.rs`

Implement the full pipeline:
1. Load profile + credentials
2. Send `Started` progress via Channel
3. Open SSH tunnel if needed
4. Find dump tool on PATH
5. Build dump command
6. Spawn subprocess, pipe stdout through GzEncoder to `.sql.gz`
7. Check exit code, capture stderr on failure
8. Record metadata in SQLite (name, note, file_path, size_bytes, db_version, dump_tool_version)
9. Clean up temp files, close SSH tunnel
10. Send `Completed` progress
11. Return Snapshot

### 5.4 Rust — snapshot_list command

SELECT snapshots for a given profile_id, ordered by created_at DESC.

### 5.5 Frontend — Snapshot store (create action)

**File:** `src/stores/snapshots.ts`

Implement `create()` with progress Channel integration:

```ts
const onProgress = new Channel<SnapshotProgress>()
onProgress.onmessage = (msg) => { /* update progress state */ }
await invoke('snapshot_create', { profileId, name, note, onProgress })
```

### 5.6 Frontend — Snapshot create view

**`src/views/SnapshotCreateView.vue`** — PageHeader + back button, profile selector (defaults to active), FormInput for name, FormTextarea for note, progress bar during creation, sticky footer.

### 5.7 Tests

- **Rust:** Unit tests for command construction (verify flags, credentials handling). Compression roundtrip test.
- **Frontend:** Snapshot store test mocking invoke.

### 5.8 Verify

- Create a snapshot of a real local MySQL/PostgreSQL/SQLite database
- `.sql.gz` file appears in `~/Library/Application Support/DatabaseSnapshotManager/snapshots/`
- Progress updates during creation
- Error handling when tool not found, connection refused, etc.

---

## Step 6: Snapshot Browser & Restore (Full Vertical Slice)

### 6.1 Rust — Restore commands

Implement in `dump.rs`:
- `build_mysql_restore_command()` — `mysql --defaults-extra-file` with piped stdin
- `build_psql_restore_command()` — `psql` with `PGPASSWORD` and piped stdin
- SQLite restore: decompress `.sql.gz` to temp, copy over original (or `sqlite3` import)

### 6.2 Rust — snapshot_restore command

Full pipeline:
1. Load snapshot + profile + credentials
2. Open SSH tunnel if needed
3. Find restore tool on PATH
4. Decompress `.sql.gz` and pipe into restore command stdin
5. Check exit code
6. Update `restored_at` in SQLite
7. Clean up

### 6.3 Rust — snapshot_delete command

Delete `.sql.gz` file from disk, delete record from snapshots table.

### 6.4 Frontend — Snapshot store (remaining actions)

Implement `fetchForProfile()`, `restore()`, `remove()` in `useSnapshotStore`.

### 6.5 Frontend — Snapshot components

**`src/components/features/SnapshotTable.vue`** — Sortable columns (name, note, created, size, actions). Row hover via scoped CSS. Human-readable size formatting. Restore and Delete buttons per row.

**`src/components/features/SnapshotRestoreDialog.vue`** — Wraps ConfirmDialog with snapshot details preview (name, profile, date, size). Warning about destructive operation.

### 6.6 Frontend — Snapshots view

**`src/views/SnapshotsView.vue`** — PageHeader + "New Snapshot" button, profile filter dropdown, SnapshotTable, EmptyState when empty.

### 6.7 Tests

- **Rust:** Restore command construction tests
- **Frontend:** SnapshotTable sorting test, store tests

### 6.8 Verify

- Browse snapshots for a profile
- Sort by name, date, size
- Restore a snapshot (verify database content reverted)
- Delete a snapshot (verify file removed)
- Error handling on restore failure

---

## Step 7: Storage Management (Full Vertical Slice)

### 7.1 Rust — storage_usage command

Walk the snapshots directory, group by project slug, sum file sizes per project.

### 7.2 Frontend — Storage components

**`src/components/features/StorageBreakdown.vue`** — Per-project list with snapshot count, total size, proportion bar.

### 7.3 Frontend — Storage view

**`src/views/StorageView.vue`** — PageHeader, total storage KPI, StorageBreakdown, pruning controls (delete older than N days). Uses ConfirmDialog for bulk deletion.

### 7.4 Verify

- Storage view shows accurate disk usage
- Per-project breakdown matches actual files
- Pruning deletes old snapshots correctly

---

## Step 8: Dashboard & Settings (Full Vertical Slice)

### 8.1 Frontend — Dashboard

**`src/components/features/KpiCard.vue`** — Icon + label + large value in `.card rounded-xl`.

**`src/views/DashboardView.vue`** — 4 KPI cards (profiles, snapshots, storage, last restore), recent snapshots mini-table, quick action buttons (Create Snapshot, Create Profile, View Storage).

### 8.2 Frontend — Settings

**`src/stores/settings.ts`** — Implement `fetchAll()`, `get()`, `set()`.

**`src/views/SettingsView.vue`** — App preferences: snapshot directory (display only), auto-prune toggle + days threshold, compression level.

### 8.3 Rust — Settings commands (if needed)

Add `settings_list` and `settings_set` commands to read/write from the settings table.

### 8.4 Verify

- Dashboard KPIs reflect real data
- Recent snapshots list is accurate
- Settings persist across app restarts

---

## Step 9: Polish & Documentation

### 9.1 Update CLAUDE.md

Update to reflect the new architecture (Vue 3 + Tailwind, Rust backend with real DB operations).

### 9.2 Update README.md

- Updated setup instructions
- New development commands (lint, test, format)
- Architecture overview
- Testing instructions (including integration test setup)

### 9.3 Final verification

- Full workflow: create profile -> test connection -> create snapshot -> browse -> restore -> delete
- SSH tunnel workflow
- Storage management
- All lint/format/test passes
- CI pipeline green

---

## File Summary

### New frontend files (~35 files)

```text
src/
  assets/main.css
  components/ui/Button.vue, Badge.vue, Card.vue, FormInput.vue, FormSelect.vue,
    FormTextarea.vue, ConfirmDialog.vue, ToastNotification.vue, EmptyState.vue, Skeleton.vue
  components/layout/AppShell.vue, AppSidebar.vue, PageHeader.vue
  components/features/ProfileCard.vue, ProfileForm.vue, SshTunnelConfig.vue,
    ConnectionTestIndicator.vue, SnapshotTable.vue, SnapshotRestoreDialog.vue,
    StorageBreakdown.vue, KpiCard.vue
  composables/useFocusTrap.ts, useToast.ts
  stores/profiles.ts, snapshots.ts, settings.ts
  views/DashboardView.vue, ProfilesView.vue, ProfileCreateView.vue, ProfileEditView.vue,
    SnapshotsView.vue, SnapshotCreateView.vue, StorageView.vue, SettingsView.vue
  router/index.ts
  types/index.ts
  App.vue
  main.ts
  env.d.ts
```

### New Rust files (~8 files)

```text
src-tauri/src/
  error.rs, db.rs, credentials.rs, dump.rs, ssh.rs,
  compress.rs, progress.rs, commands.rs
```

### Modified files

```text
src-tauri/Cargo.toml, src-tauri/src/lib.rs
package.json, vite.config.ts, tsconfig.json, index.html
.vscode/extensions.json
CLAUDE.md, README.md
```

### New config files (~10 files)

```text
eslint.config.js, .prettierrc, .prettierignore, lefthook.yml
src-tauri/rustfmt.toml, tsconfig.node.json
.vscode/settings.json, .github/workflows/ci.yml
src/test/setup.ts
scripts/test-db-setup.sh, scripts/test-db-teardown.sh
```

---

*Plan assembled 19 February 2026*
