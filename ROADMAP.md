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
- **Status:** pending
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
- **Status:** pending
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
- **Status:** pending
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
- **Status:** pending
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
- **Status:** pending
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
- **Status:** pending
- **Description:** After a migration or manual schema change, developers need to understand exactly what changed in their database. Comparing two snapshots side by side — highlighting added/removed tables, column changes, index differences, and row count deltas — would turn Amber from a backup tool into a database change audit tool, giving developers confidence about what each migration actually modified.
- **Acceptance criteria:**
  - Select any two snapshots from the same profile for comparison
  - Schema diff shows: added/removed tables, column type changes, new/dropped indexes
  - Data diff shows: row count changes per table, approximate data size delta
  - Diff displayed in a split-view with colour-coded additions (green) and removals (red)
  - Comparison exportable as Markdown for inclusion in pull request descriptions

## Design System Adoption

These items implement the Scooda design system (derived from the Dalil app styleguide) to achieve premium visual uniformity across all Tauri applications. Items are ordered by dependency — foundation must complete before migration, migration before polish.

### [Foundation] Integrate @stuntrocket/ui shared component library and design tokens
- **Priority:** P1 (critical)
- **Size:** M (1-3hrs)
- **Added:** 2026-03-19
- **Status:** completed
- **Completed:** 2026-03-20
- **Description:** Amber currently uses vanilla TypeScript with direct DOM manipulation and no design system. Adopting the Scooda design system requires installing the @stuntrocket/ui package from the local Verdaccio registry, importing the shared design tokens (tokens.css), loading Poppins as the primary font, and configuring the Scooda colour palette, spacing scale, and typography scale. This is the prerequisite for all subsequent design work and will transform the app from a bare-bones prototype appearance to the premium Scooda visual identity.
- **Acceptance criteria:**
  - .npmrc configured with @stuntrocket:registry=http://localhost:4873
  - @stuntrocket/ui installed as a dependency
  - Scooda tokens.css imported (colour palette, typography scale, spacing, shadows, border radii)
  - Poppins font loaded via Google Fonts or local font files
  - Tailwind CSS v4 installed and configured with Scooda @theme block
  - App renders with correct Scooda colour palette and typography in both light and dark modes
  - Ambient background blobs CSS imported and rendering

### [UI Migration] Replace all UI elements with @stuntrocket/ui shared components
- **Priority:** P1 (critical)
- **Size:** XL (8hrs+)
- **Added:** 2026-03-19
- **Status:** pending
- **Note:** Skipped: too large for autonomous cycle, needs Danny's input.
- **Description:** Systematically replace all locally-defined UI elements with @stuntrocket/ui equivalents. Since Amber uses vanilla TypeScript rather than Vue, this may require either migrating to Vue 3 (matching the rest of the portfolio) or consuming the CSS-only layer of @stuntrocket/ui with custom rendering. Every button, input, card, badge, modal, and toast must match the Scooda styleguide exactly. The goal is zero bespoke UI primitives.
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

### [Polish] Achieve full Scooda styleguide visual conformance
- **Priority:** P2 (important)
- **Size:** L (3-8hrs)
- **Added:** 2026-03-19
- **Status:** pending
- **Description:** After component migration, apply the remaining Scooda styleguide specifications that go beyond individual components: ambient background blobs with correct colours and animation timing, custom accent-tinted scrollbars, micro-animations on all interactive elements, macOS native titlebar integration with traffic light spacing, correct z-index layering, and full accessibility compliance (focus rings, skip-to-content, ARIA patterns, reduced motion support). Visual QA against the Dalil reference app to verify conformance.
- **Acceptance criteria:**
  - Ambient background blobs rendered with correct colours (accent, violet, cyan) and 20-30s drift animations
  - Custom scrollbars with accent-tinted thumb and gradient overlay
  - Micro-animations: scale(0.98) on sidebar links, scale(0.94) on toolbar buttons, translateY(-1px) on tabs
  - macOS titlebar with -webkit-app-region: drag and 78px traffic light padding
  - Z-index layering matches styleguide scale (blobs:1, sidebar:10, topbar:50, modals:100, toasts:200)
  - prefers-reduced-motion disables all animations
  - Focus ring: 2px solid accent/55% with 2px offset on all interactive elements
  - Visual side-by-side comparison with Dalil passes review
