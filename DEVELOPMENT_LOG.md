# Amber Development Log

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
- Branch: feature/roadmap-items-batch
- Tests passing: yes (cargo test 23/23, vite build clean)
- Build status: vite build success
- Notes: Implemented 11 functional roadmap items in a single batch. Database migrations 5-8 added for tags, pinned flag, retention policies, and tool paths tables. Snapshot struct extended with pinned and tags fields. Fixed pre-existing SPageHeader import issues (component was removed during design system integration but references remained). Recreated PageHeader.vue and KpiCard.vue local components. Skipped pre-migration auto-capture (P3/M, complex file watcher), snapshot content browser (P3/M, complex SQL parsing), and Design System Adoption section (needs Danny's input).

## Cycle: 2026-03-20 20:10
- App: amber
- Items completed:
  - [Foundation] Integrate @stuntrocket/ui shared component library and design tokens (P1/M) — installed @stuntrocket/ui v0.5.0 from local Verdaccio registry, replaced bespoke CSS with Scooda tokens.css/base.css/scrollbar.css/ambient.css imports, added Poppins font via Google Fonts, enabled class-based dark mode (.dark on html), migrated all form components to @stuntrocket/ui shared components (SInput, SSelect, SButton, SFormField, STextarea, SEmptyState, SConfirmDialog, Card). Fixed TypeScript errors in number-type input bindings. Amber accent override (#F59E0B) preserved for brand identity.
- Items attempted but failed: none
- Branch: feature/scooda-design-tokens
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
- Branch: feature/error-classification-and-snapshot-integrity
- Tests passing: yes (cargo test 23/23, vitest 20/20, vue-tsc clean, clippy clean)
- Build status: success
- Notes: The two P1/L items were already fully implemented in the initial commit but not marked as completed on the roadmap. The two P2/S items were implemented this cycle: connection error classification (classify.rs with 10 unit tests) and snapshot integrity verification (checksum.rs with 5 unit tests, plus frontend integration). Removed duplicate roadmap entry for snapshot integrity in the Pending section.
