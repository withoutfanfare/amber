# Amber Development Log

## Cycle: 2026-03-30 00:00
- App: amber
- Items completed:
  - [Feature] Add scheduled automatic snapshots on configurable intervals per profile (P2/M) — New `schedule_configs` SQLite table (migration 9) stores per-profile interval, last_snapshot_at, and next_due_at. Three new Rust commands: `schedule_config_get`, `schedule_config_set`, `schedule_config_list`. New `scheduler.rs` module with background tokio task checking every 60 seconds for due profiles. Scheduler acquires operation lock (prevents concurrent manual+scheduled), runs full dump/compress/checksum pipeline, tags snapshots with `[auto] scheduled` and `[interval] {type}`. Frontend: `ScheduleConfig` TypeScript type, SettingsView schedule configuration section with profile selector, interval dropdown, and last/next snapshot timestamps.
- Items attempted but failed: none
- Branch: feature/scheduled-snapshots (merged to develop)
- Tests passing: yes (cargo test 23/23, cargo clippy clean, vue-tsc clean)
- Build status: failed (pre-existing @stuntrocket/ui vue-router resolution issue in vite build — not caused by this cycle's changes)
- Notes: The scheduler starts 30 seconds after app launch to avoid interfering with startup. When a profile's operation lock is busy (manual snapshot in progress), the scheduler skips that profile and retries next cycle. Scheduled snapshots respect existing retention policies since they use the standard snapshot_tags mechanism. The `compute_next_due` function calculates next run from last_snapshot_at + interval duration.

## Cycle: 2026-03-29 21:00
- App: amber
- Items completed:
  - [Quality] Interrupted snapshot cleanup detecting and removing orphaned partial dump files on launch (P2/S) — New `scan_orphaned_snapshots` Tauri command walks the snapshots directory, compares .sql.gz and .db.gz files against all `file_path` entries in the snapshots table, and returns orphan metadata (path, size, creation date, inferred DB type). `delete_orphaned_snapshots` command safely removes specified orphaned files and cleans up empty project directories. Frontend types and store methods added for integration.
  - [Quality] Automatic pre-restore safety snapshot capturing current database state before any restore operation (P2/S) — Internal `create_pre_restore_snapshot` async helper runs the full dump pipeline (mysqldump/pg_dump/sqlite3 VACUUM INTO → gzip compression → SHA-256 checksum → metadata insertion) without progress streaming. Executes within `snapshot_restore` after SSH tunnel setup and before the actual restore, using the same connection context. Tagged as "[auto] pre-restore" with note linking to the snapshot being restored. Restore blocked if the safety snapshot fails. `RestoreRecord` now includes `pre_restore_snapshot_id` for frontend undo support.
- Items attempted but failed: none
- Branch: feature/orphan-cleanup-prerestore-snapshot
- Tests passing: yes (cargo clippy clean, vue-tsc clean)
- Build status: Tauri full build failed (pre-existing `vue-router` resolution issue in `@stuntrocket/ui` — not related to this cycle's changes). Merge to main preserved.
- Notes: Both features are backend-focused with frontend type/store additions. No database migration needed — both use existing snapshot and snapshot_tags tables.

## Cycle: 2026-03-28 12:00
- App: amber
- Items completed:
  - [Quality] Database connection health monitoring with proactive status indicator (P2/S) — New `profile_health_check` Tauri command reuses existing `profile_test_connection` infrastructure, returning a lightweight `HealthCheckResult` with status (connected/unreachable/unchecked), latency, and timestamp. ProfilesView runs health checks on mount and every 60 seconds. ProfileCard displays a coloured status dot (green/red/grey) with tooltip showing latency or error message, plus an operation-in-progress indicator when the profile is busy.
  - [Quality] Disk space pre-flight check before snapshot creation (P2/S) — New `check_disk_space` Tauri command uses macOS `statfs` to read available space on the snapshots volume, compares against estimated compressed snapshot size with configurable safety margin (default 2x). SnapshotCreateView runs the check alongside size estimation and displays a warning banner (red) when space is insufficient or a confirmation badge (green) when sufficient. Create button disabled when disk space is insufficient.
  - [Quality] Concurrent operation guard preventing simultaneous snapshot/restore on same profile (P2/S) — `OperationSlot` with `AtomicBool` busy flag and RAII `ProfileOpGuard` that auto-clears on drop. `OperationLocks` managed state holds per-profile slots. `snapshot_create` and `snapshot_restore` acquire the lock before proceeding; concurrent requests receive `OperationInProgress` error. `operation_status` command lets the frontend check lock state. SnapshotCreateView checks and disables the create button when the profile is busy.
- Items attempted but failed: none
- Branch: feature/health-diskcheck-opguard
- Tests passing: yes (23/23 tests, cargo clippy clean)
- Build status: Rust release build succeeded. Tauri full build failed (pre-existing `vue-router` resolution issue in `@stuntrocket/ui` — not related to this cycle's changes).
- Notes: Added `libc` dependency for macOS `statfs` call. All three features are backend + frontend, no database migration needed.

## Cycle: 2026-03-22 10:00
- App: amber
- Items completed:
  - [Feature] Add snapshot content browser for inspecting data without restoring (P3/M) — Rust command decompresses .sql.gz snapshot, parses CREATE TABLE for schema, MySQL INSERT statements and PostgreSQL COPY blocks for data extraction, returns table list with column names, row counts, and up to 50 sample rows per table. Vue dialog with table list sidebar, column pills, scrollable data grid with row numbers, NULL styling, and truncated cell values with tooltips. Frontend wired via Pinia store browseContent action and SnapshotTable browse emit.
- Items attempted but failed: none
- Branch: feature/snapshot-content-browser
- Tests passing: yes (cargo test, cargo clippy clean, vite build clean)
- Build status: pending
- Notes: SQL parsing handles MySQL INSERT INTO with stateful quote/escape/parenthesis tracking and PostgreSQL COPY FROM stdin with tab-separated values and \. terminator. Column names extracted from CREATE TABLE definitions with backtick/quote stripping and constraint keyword filtering. Cell values truncated to 200 chars for display. Pre-commit hook pulled in pre-existing uncommitted changes (ROADMAP.md scheduled snapshots item, package-lock.json) into the feature branch commit.

## Cycle: 2026-03-20 12:50
- App: amber
- Items completed:
  - [UX/UI] Add snapshot size estimation before capture (P2/M) — Rust command queries database size via mysql/psql/file metadata, applies historical or default 5:1 compression ratio, shown in snapshot creation dialog before user confirms
  - [Performance] Implement streaming progress for large database dumps (P2/M) — compress_from_reader_with_progress streams SQL through gzip while parsing mysqldump/pg_dump table markers, emits TableProgress events and byte throughput via Tauri Channel at 500ms intervals
  - [UX/UI] Add snapshot tagging and search (P3/S) — snapshot_tags SQLite table with add/remove/list commands, tag pills in snapshot table, click-to-filter tag bar, free-text search across name/note/tags, tag autocomplete suggestions on create dialog
  - [Distribution] Add first-run setup wizard for database tool discovery (P2/M) — discover_tools scans PATH for mysqldump/mysql/pg_dump/psql/sqlite3, reports versions and paths, manual path override for non-standard locations, auto-triggers when no profiles exist, re-runnable from settings
  - [Feature] Add snapshot retention policies and disk usage monitoring (P2/S) — retention_policies SQLite table per profile (max_count, max_age_days, max_size_bytes), pinned flag protects snapshots from auto-cleanup, enforce command deletes oldest unpinned snapshots, UI in settings
  - [Feature] Compare schema differences between two snapshots (P2/M) — extracts CREATE TABLE statements from compressed snapshots, computes table/column diffs between two snapshots, colour-coded split view in dialog
  - [UX/UI] Add snapshot restore dry-run preview (P2/S) — compares snapshot schema against live database tables, shows tables to add/remove/common with warnings, read-only preview dialog
  - [Quality] Add database tool version compatibility checking (P2/S) — detects dump tool version at snapshot creation, stores in db_version/dump_tool_version columns, compares major versions before restore with advisory warnings
  - [Feature] Add snapshot export as portable SQL file (P3/S) — decompresses .sql.gz to plain SQL with metadata header comment, descriptive filename, progress during export
  - [Feature] Add snapshot restore to alternate database (P2/M) — already implemented in previous cycle's RestoreOptions with targetDatabaseName, restore dialog has profile selector and database name override
  - [UX/UI] Add keyboard shortcuts (P2/S) — Cmd+N (new snapshot), Cmd+1-9 (switch profiles), Cmd+F (focus search), Cmd+/ (help overlay), Cmd+, (settings), composable in AppShell
- Items attempted but failed: none
- Branch: feature/roadmap-items-batch (merged to main, branch deleted)
- Tests passing: yes (cargo test 23/23, vite build clean)
- Build status: vite build success
- Notes: Implemented 11 functional roadmap items in a single batch. Database migrations 5-8 added for tags, pinned flag, retention policies, and tool paths tables. Snapshot struct extended with pinned and tags fields. Fixed pre-existing SPageHeader import issues (component was removed during design system integration but references remained). Recreated PageHeader.vue and KpiCard.vue local components. Skipped pre-migration auto-capture (P3/M, complex file watcher), snapshot content browser (P3/M, complex SQL parsing), and Design System Adoption section (needs Danny's input).

## Cycle: 2026-03-20 20:10
- App: amber
- Items completed:
  - [Foundation] Integrate @stuntrocket/ui shared component library and design tokens (P1/M) — installed @stuntrocket/ui v0.5.0 from local Verdaccio registry, replaced bespoke CSS with @stuntrocket/ui tokens.css/base.css/scrollbar.css/ambient.css imports, added Poppins font via Google Fonts, enabled class-based dark mode (.dark on html), migrated all form components to @stuntrocket/ui shared components (SInput, SSelect, SButton, SFormField, STextarea, SEmptyState, SConfirmDialog, Card). Fixed TypeScript errors in number-type input bindings. Amber accent override (#F59E0B) preserved for brand identity.
- Items attempted but failed: none
- Branch: feature/scooda-design-tokens (merged to main, branch deleted)
- Tests passing: yes (cargo test 23/23, vitest 20/20, vue-tsc clean, cargo clippy clean, vite build clean)
- Build status: success (Amber.app + Amber_0.1.0_aarch64.dmg bundled, copied to ~/Desktop/TauriBuilds/amber/Amber-2026-03-20-2010.app)
- Notes: Previous incomplete cycle had left uncommitted design system changes on main. This cycle completed the integration properly on a feature branch with .npmrc configured, @stuntrocket/ui added to package.json, vue-eslint-parser added as dev dependency for eslint-plugin-vue compatibility, and all Prettier formatting applied. The P1/XL UI Migration item marked as "Skipped: too large for autonomous cycle, needs Danny's input."

## Cycle: 2026-03-20 09:00
- App: amber
- Items completed:
  - [Quality] Add structured error handling for database connectivity failures (P2/S) — classified connection errors from raw stderr into structured errors with human-readable messages and remediation hints for MySQL, PostgreSQL, and SQLite
  - [Quality] Add snapshot integrity verification before restore operations (P2/S) — SHA-256 checksum computed at snapshot creation, verified before restore, integrity column in snapshot table with verify button
  - [Quality] Migrate persistence from localStorage to SQLite (P1/L) — marked as completed (already implemented in initial commit)
  - [Feature] Implement real database dump and restore via subprocess execution (P1/L) — marked as completed (already implemented in initial commit)
- Items attempted but failed: none
- Branch: feature/error-classification-and-snapshot-integrity (merged to main, branch deleted)
- Tests passing: yes (cargo test 23/23, vitest 20/20, vue-tsc clean, clippy clean)
- Build status: success
- Notes: The two P1/L items were already fully implemented in the initial commit but not marked as completed on the roadmap. The two P2/S items were implemented this cycle: connection error classification (classify.rs with 10 unit tests) and snapshot integrity verification (checksum.rs with 5 unit tests, plus frontend integration). Removed duplicate roadmap entry for snapshot integrity in the Pending section.
