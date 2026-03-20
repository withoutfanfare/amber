# Amber Roadmap

Database snapshot manager for local development databases (MySQL, PostgreSQL, SQLite).

## Completed

### [Quality] Migrate persistence from localStorage to SQLite
- **Priority:** P1 (critical)
- **Size:** L (3-8hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-19
- **Description:** The MVP currently stores profiles, snapshots, and settings in localStorage, which is volatile and size-limited. Migrating to a local SQLite database (as already planned in the technical documentation) will ensure data durability across app updates, prevent silent data loss from storage eviction, and enable richer querying as the snapshot catalogue grows.
- **Acceptance criteria:**
  - All profile, snapshot metadata, and restore history records persisted in a local SQLite database
  - Existing localStorage data migrated automatically on first launch after upgrade
  - localStorage fallback removed; app functions correctly with SQLite as sole store
  - Unit tests cover migration path and CRUD operations

### [Feature] Implement real database dump and restore via subprocess execution
- **Priority:** P1 (critical)
- **Size:** L (3-8hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-19
- **Description:** The MVP currently simulates snapshot and restore operations in the UI without executing real database tools. The core value proposition — reliable database snapshots — depends on wiring up actual `mysqldump`/`pg_dump` execution in the Rust backend via subprocess calls, with proper credential handling, error capture, and output streaming. Without this, the app is a prototype rather than a functional tool.
- **Acceptance criteria:**
  - Rust backend executes `mysqldump` (MySQL) and `pg_dump` (PostgreSQL) as subprocesses with configurable binary paths
  - Database credentials passed securely via environment variables or temp config files (never logged or persisted in plaintext)
  - Stdout captured and written to compressed snapshot files on disk
  - Stderr captured and surfaced to the user as structured error messages
  - Restore operations execute the corresponding import commands (`mysql`, `psql`)
  - SQLite databases handled via file copy with integrity check

### [Quality] Add structured error handling for database connectivity failures
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** When users configure a database connection profile, the app must handle common connectivity failures gracefully — wrong credentials, server not running, port blocked, unknown host. Clear, actionable error messages (not raw driver errors) are essential for a database tool that targets developers who may be troubleshooting their local environment.
- **Acceptance criteria:**
  - Connection test command returns structured errors with human-readable messages for: authentication failure, connection refused, timeout, unknown host, permission denied
  - Error messages include suggested remediation (e.g. "Check that MySQL is running on port 3306")
  - Connection test callable from the profile creation/edit dialog before saving
  - Errors surfaced in the UI with appropriate severity styling (not raw stack traces)

### [Quality] Add snapshot integrity verification before restore operations
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-20
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Restoring a corrupted snapshot to a development database could cause silent data issues or leave the database in an inconsistent state. Verifying snapshot file integrity (checksum validation, compressed archive integrity, SQL syntax check on first few lines) before executing a restore would protect users from silent corruption and provide a clear error message instead of a cryptic database import failure.
- **Acceptance criteria:**
  - SHA-256 checksum recorded at snapshot creation time and stored in metadata
  - Checksum verified before any restore operation begins
  - Compressed archive integrity validated (gzip/zstd decompression test without full extraction)
  - Restore blocked with clear error message if integrity check fails
  - Integrity status indicator visible on each snapshot in the list view

## Pending

### [UX/UI] Add snapshot size estimation before capture
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Users currently have no visibility into how large a snapshot will be before creating it. Displaying an estimated size (based on database size and compression ratio history) in the snapshot creation dialog helps users manage disk space proactively and avoid surprises with large databases.
- **Acceptance criteria:**
  - Snapshot creation dialog shows estimated compressed size before user confirms
  - Estimation uses actual database size and historical compression ratio (or a sensible default for first snapshot)
  - Estimation completes within 2 seconds for databases up to 1 GB
  - "Unable to estimate" fallback displayed gracefully if estimation fails

### [Performance] Implement streaming progress for large database dumps
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** The progress reporting infrastructure exists but needs to be connected to real `mysqldump`/`pg_dump` execution. For large databases, users need accurate, granular progress feedback rather than an indeterminate spinner, so they know the operation is progressing and can estimate completion time.
- **Acceptance criteria:**
  - Progress bar updates reflect actual dump progress (table count or byte throughput)
  - Progress events emitted at least every 2 seconds during active dump
  - User can see which table is currently being dumped (MySQL/PostgreSQL)
  - Cancellation button remains responsive during long-running dumps

### [UX/UI] Add snapshot tagging and search
- **Priority:** P3 (nice-to-have)
- **Size:** S (< 1hr)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** As the snapshot catalogue grows, users need to organise and find snapshots quickly. Tagging snapshots with labels (e.g. "pre-migration", "clean-state", "v2.1-baseline") and searching by tag, name, or database would prevent the snapshot list from becoming unmanageable, especially for developers working across multiple projects.
- **Acceptance criteria:**
  - Users can add one or more tags when creating a snapshot or editing an existing one
  - Snapshot list view supports filtering by tag (click-to-filter) and free-text search
  - Search matches against snapshot name, tags, database name, and profile name
  - Tags displayed as pills/badges in the snapshot list rows
  - Tag autocomplete suggests previously used tags

### [Distribution] Add first-run setup wizard for database tool discovery
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Amber depends on external database tools (mysqldump, pg_dump, sqlite3) being available on the user's system. A first-run wizard that auto-discovers these tools, validates their versions, and guides the user through initial configuration would dramatically reduce setup friction and prevent confusing errors when tools are missing or incompatible.
- **Acceptance criteria:**
  - First-run wizard triggers automatically when no profiles exist
  - Auto-scans PATH for mysqldump, pg_dump, psql, mysql, sqlite3 and reports found/missing tools with versions
  - Users can manually specify tool paths if auto-discovery fails (e.g. Homebrew non-standard locations)
  - Wizard can be re-run from settings at any time
  - Minimum version requirements clearly stated for each tool

### [Innovation] Add pre-migration automatic snapshot capture
- **Priority:** P3 (nice-to-have)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** pending
- **Description:** The most dangerous moment for a development database is right before running a migration. Amber could differentiate itself by detecting when common migration commands (`php artisan migrate`, `rails db:migrate`, `knex migrate:latest`) are about to run and automatically capturing a snapshot as a safety net. This transforms Amber from a manual backup tool into an intelligent guardian that protects against the exact scenario where snapshots matter most.
- **Acceptance criteria:**
  - File system watcher or shell hook integration detects migration command execution in registered project directories
  - Automatic snapshot created with "[auto] pre-migration" tag and timestamp
  - Snapshot captured before the migration command proceeds (blocking or near-instant)
  - Auto-capture configurable per profile (enable/disable, which commands to watch)
  - Auto-snapshots subject to a configurable retention limit (e.g. keep last 5) to prevent disk bloat

### [Feature] Add snapshot retention policies and disk usage monitoring
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-20
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Once real database dumps are operational, snapshots will consume significant disk space. Without retention policies, users must manually track and delete old snapshots to prevent disk exhaustion. Configurable retention rules (keep last N snapshots per profile, maximum total size, age-based expiry) with a disk usage overview would let users manage storage proactively. The Innovation auto-snapshot item proposes retention for automatic captures, but manual snapshots need the same governance.
- **Acceptance criteria:**
  - Configurable retention policies per profile: keep last N snapshots, maximum total size, age-based expiry
  - Disk usage overview showing total snapshot storage, per-profile breakdown, and largest snapshots
  - Warning notification when total snapshot storage exceeds a configurable threshold
  - Retention enforcement runs on app launch and after new snapshot creation
  - Manual override: users can protect specific snapshots from auto-cleanup ("pinned" flag)

### [Feature] Compare schema differences between two snapshots
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** After a migration or manual schema change, developers need to understand exactly what changed in their database. Comparing two snapshots side by side — highlighting added/removed tables, column changes, index differences, and row count deltas — would turn Amber from a backup tool into a database change audit tool, giving developers confidence about what each migration actually modified.
- **Acceptance criteria:**
  - Select any two snapshots from the same profile for comparison
  - Schema diff shows: added/removed tables, column type changes, new/dropped indexes
  - Data diff shows: row count changes per table, approximate data size delta
  - Diff displayed in a split-view with colour-coded additions (green) and removals (red)
  - Comparison exportable as Markdown for inclusion in pull request descriptions

### [UX/UI] Add snapshot restore dry-run preview showing schema and data impact
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-21
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Before committing to a full restore, users should be able to preview what the restore will do — whether the snapshot's schema matches the current database, approximate row count changes per table, and whether any tables in the current database would be lost. This is especially important when restoring snapshots taken before a migration, as the schema may have diverged. The integrity verification item validates the snapshot file is intact; this item validates the restore would produce the expected result.
- **Acceptance criteria:**
  - "Preview restore" action available alongside the standard restore button
  - Preview compares snapshot schema against current database schema (tables, columns, indexes)
  - Schema differences displayed as a diff: added/removed/modified tables and columns
  - Approximate row count changes shown per table (snapshot vs current)
  - Warning displayed if the restore would drop tables that exist in the current database but not in the snapshot
  - Preview does not modify the database (read-only operation)

### [Quality] Add database tool version compatibility checking before operations
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-21
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** The mysqldump and pg_dump tools have version-specific behaviours — a dump captured with mysqldump 8.0 may produce SQL that fails to import on MySQL 5.7, or uses features not available in the restore environment. Storing the dump tool version and database server version at snapshot creation time, and comparing against the current restore environment's versions, would prevent silent cross-version compatibility issues that produce corrupted restores or cryptic import errors.
- **Acceptance criteria:**
  - Snapshot metadata records: dump tool name, dump tool version, database server version at capture time
  - Before restore, compare recorded versions against current tool and server versions
  - Warning displayed if major version mismatch detected (e.g. MySQL 8.0 dump restoring to MySQL 5.7)
  - Warning includes specific compatibility concerns for the detected version combination
  - User can proceed with restore despite warnings (advisory, not blocking)
  - Version information displayed on snapshot cards in the list view

### [Feature] Add snapshot export as portable SQL file for cross-tool compatibility
- **Priority:** P3 (nice-to-have)
- **Size:** S (< 1hr)
- **Added:** 2026-03-21
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Snapshots are stored in Amber's internal compressed format, which is efficient for the app's own restore workflow but not portable. Developers sometimes need a plain SQL file to use with other database tools, share with teammates who don't have Amber, seed CI databases, or import into database management GUIs. Exporting a snapshot as a standard uncompressed SQL dump file would make Amber's snapshots interoperable with the wider database tooling ecosystem.
- **Acceptance criteria:**
  - "Export as SQL" action available on each snapshot in the list view and detail panel
  - Export decompresses the snapshot and writes a standard SQL file to a user-selected directory
  - Exported SQL file is valid for direct import via mysql/psql CLI tools
  - Export includes a header comment with snapshot metadata (database name, capture date, tool version, Amber snapshot ID)
  - Progress indicator shown for large snapshots (decompression can be slow)
  - File saved with a descriptive name (e.g. mydb-2026-03-21-pre-migration.sql)

### [Feature] Add snapshot restore to alternate database for safe data inspection
- **Priority:** P2 (important)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-20
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Currently, restoring a snapshot overwrites the original database, making it a destructive operation even when the user only wants to inspect old data. Supporting restore-to-alternate-target — a temporary or user-specified database name — would let developers inspect snapshot contents, run queries against historical data, or verify a snapshot's integrity without disrupting their current development state. This is especially valuable after a migration, when developers want to compare pre-migration data with the current state without rolling back.
- **Acceptance criteria:**
  - Restore dialog offers "Restore to original" and "Restore to new database" options
  - "Restore to new database" prompts for a target database name (with auto-suggested name, e.g. mydb_snapshot_20260320)
  - Alternate restore creates the target database if it doesn't exist
  - Restored data accessible via the user's normal database tools (mysql, psql CLI or GUI)
  - Original database untouched during alternate restore
  - Cleanup action available to drop temporary restore databases when no longer needed

### [UX/UI] Add keyboard shortcuts for common snapshot operations
- **Priority:** P2 (important)
- **Size:** S (< 1hr)
- **Added:** 2026-03-20
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Every other app in the Tauri portfolio has keyboard shortcuts implemented or planned, but Amber has none. Developers using Amber alongside their editor and terminal need rapid access to core operations — creating snapshots, switching profiles, triggering restores — without reaching for the mouse. Standard keyboard shortcuts would bring Amber's interaction speed in line with the portfolio standard and match the keyboard-driven workflow expectations of its developer audience.
- **Acceptance criteria:**
  - Cmd+N creates a new snapshot for the active profile
  - Cmd+R initiates restore for the selected snapshot (with confirmation)
  - Cmd+1 through Cmd+9 switches between profiles
  - Cmd+F focuses the snapshot search/filter input
  - j/k navigates the snapshot list
  - All shortcuts documented in a help overlay (Cmd+/)
  - No conflicts with system-level macOS shortcuts

### [Feature] Add snapshot content browser for inspecting data without restoring
- **Priority:** P3 (nice-to-have)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-20
- **Status:** pending
- **Description:** Before committing to a restore, developers sometimes need to inspect the actual data in a snapshot — not just the schema difference (covered by the restore dry-run preview), but table row counts, sample records, and specific values. A read-only content browser that extracts table metadata and sample rows from a compressed snapshot file would let users verify they have the right snapshot without the risk and time cost of a full restore. This is particularly useful when snapshots accumulate and names alone are insufficient to identify the correct one.
- **Acceptance criteria:**
  - "Browse contents" action available on each snapshot in the list view
  - Browser displays table list with row counts extracted from the snapshot
  - Clicking a table shows the first 50 rows in a scrollable data grid
  - Content extracted from the compressed snapshot without importing to a live database
  - Browser is read-only — no modification of snapshot or database state
  - Works for MySQL and PostgreSQL snapshots (SQLite snapshots use file inspection)

## Design System Adoption

These items implement the @stuntrocket/ui design system to achieve premium visual uniformity across all Tauri applications. Items are ordered by dependency — foundation must complete before migration, migration before polish.

### [Foundation] Integrate @stuntrocket/ui shared component library and design tokens
- **Priority:** P1 (critical)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Amber currently uses vanilla TypeScript with direct DOM manipulation and no design system. Adopting the @stuntrocket/ui design system requires installing the @stuntrocket/ui package from the local Verdaccio registry, importing the shared design tokens (tokens.css), loading Poppins as the primary font, and configuring the @stuntrocket/ui colour palette, spacing scale, and typography scale. This is the prerequisite for all subsequent design work and will transform the app from a bare-bones prototype appearance to the premium @stuntrocket/ui visual identity.
- **Acceptance criteria:**
  - .npmrc configured with @stuntrocket:registry=http://localhost:4873
  - @stuntrocket/ui installed as a dependency
  - @stuntrocket/ui tokens.css imported (colour palette, typography scale, spacing, shadows, border radii)
  - Poppins font loaded via Google Fonts or local font files
  - Tailwind CSS v4 installed and configured with @stuntrocket/ui @theme block
  - App renders with correct @stuntrocket/ui colour palette and typography in both light and dark modes
  - Ambient background blobs CSS imported and rendering

### [UI Migration] Replace all UI elements with @stuntrocket/ui shared components
- **Priority:** P1 (critical)
- **Size:** XL (8hrs+)
- **Added:** 2026-03-19
- **Status:** pending
- **Note:** Skipped: too large for autonomous cycle, needs Danny's input.
- **Description:** Systematically replace all locally-defined UI elements with @stuntrocket/ui equivalents. Since Amber uses vanilla TypeScript rather than Vue, this may require either migrating to Vue 3 (matching the rest of the portfolio) or consuming the CSS-only layer of @stuntrocket/ui with custom rendering. Every button, input, card, badge, modal, and toast must match the @stuntrocket/ui styleguide exactly. The goal is zero bespoke UI primitives.
- **Acceptance criteria:**
  - All buttons use @stuntrocket/ui Button variants (primary CTA, secondary, icon)
  - All form controls use @stuntrocket/ui Input, Select, Textarea styles
  - All cards use @stuntrocket/ui Card variants (content header, sidebar, list)
  - All toasts and notifications use @stuntrocket/ui Toast component
  - Profile management views use @stuntrocket/ui Modal and form patterns
  - Snapshot list uses @stuntrocket/ui list card pattern with hover states
  - Navigation uses @stuntrocket/ui sidebar link pattern
  - No locally-defined UI primitive styles remain
  - Dark mode renders correctly with all shared components

### [Polish] Achieve full @stuntrocket/ui styleguide visual conformance
- **Priority:** P2 (important)
- **Size:** L (3-8hrs)
- **Added:** 2026-03-19
- **Status:** pending
- **Description:** After component migration, apply the remaining @stuntrocket/ui styleguide specifications that go beyond individual components: ambient background blobs with correct colours and animation timing, custom accent-tinted scrollbars, micro-animations on all interactive elements, macOS native titlebar integration with traffic light spacing, correct z-index layering, and full accessibility compliance (focus rings, skip-to-content, ARIA patterns, reduced motion support). Visual QA against the Dalil reference app to verify conformance.
- **Acceptance criteria:**
  - Ambient background blobs rendered with correct colours (accent, violet, cyan) and 20-30s drift animations
  - Custom scrollbars with accent-tinted thumb and gradient overlay
  - Micro-animations: scale(0.98) on sidebar links, scale(0.94) on toolbar buttons, translateY(-1px) on tabs
  - macOS titlebar with -webkit-app-region: drag and 78px traffic light padding
  - Z-index layering matches styleguide scale (blobs:1, sidebar:10, topbar:50, modals:100, toasts:200)
  - prefers-reduced-motion disables all animations
  - Focus ring: 2px solid accent/55% with 2px offset on all interactive elements
  - Visual side-by-side comparison with Dalil passes review
