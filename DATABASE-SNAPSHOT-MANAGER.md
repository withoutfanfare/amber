# Database Snapshot Manager

> Git-like version control for local development databases. Built with Tauri 2 + Rust for developers who need confidence before every migration.

**Document status**: Concept enrichment — 18 February 2026
**Audience**: Solo developer / small agency planning use

---

## Table of Contents

1. [Product Vision](#1-product-vision)
2. [Target Users](#2-target-users)
3. [Competitive Analysis](#3-competitive-analysis)
4. [Core Features](#4-core-features)
5. [Technical Architecture](#5-technical-architecture)
6. [Risk Assessment](#6-risk-assessment)
7. [MVP Scope](#7-mvp-scope)
8. [Monetisation Strategy](#8-monetisation-strategy)
9. [Implementation Phases](#9-implementation-phases)
10. [Success Metrics](#10-success-metrics)
11. [Open Questions](#11-open-questions)

---

## 1. Product Vision

### Vision Statement

Database Snapshot Manager gives local development databases the same safety net that Git gives source code. Before every risky migration, schema change, or data experiment, developers create a named snapshot in one click — and restore to that exact state in seconds if anything goes wrong.

The tool is not a backup solution. It is not a production database tool. It is a development-time confidence layer that sits quietly alongside your existing workflow and removes the single most common reason developers hesitate before running `php artisan migrate`: the fear of losing hours of carefully seeded local data.

### The Problem

Local development databases are stateful and fragile. Developers invest significant time building up realistic local data — seeded records, manually tested edge cases, specific user journeys — and a failed migration or poorly written rollback can erase that work entirely. The existing solutions to this problem are all painful:

- **Manual mysqldump commands** are slow, easy to forget, and produce unlabelled files scattered across the filesystem.
- **Rollback migrations** require the discipline to write them perfectly, every time, which rarely happens under deadline pressure.
- **Docker volume snapshots** are opaque and tied to container lifecycle rather than code events.
- **Fresh seeds from scratch** take minutes and never quite reproduce the exact state you had before.

The result is that most developers either accept data loss as a fact of local development life, or they slow down and become more cautious in a way that reduces iteration speed. Neither outcome is acceptable.

### The Opportunity

There is no desktop GUI tool that treats local development database state management as a first-class workflow. CLI tools exist and serve power users. Enterprise platforms like DBSnapper serve compliance and data-ops teams. But there is nothing in between — nothing that gives a single developer or a small team a visual, intuitive interface for snapshotting, diffing, and branching their local development data.

This gap is the opportunity.

### Core Promise

"Create a named snapshot before your migration runs. If it breaks, restore in seconds. Compare exactly what changed. Branch your data to test multiple scenarios at once."

### Internal Scope and AI Constraints

This is an internal agency tool intended to improve developer confidence and quality of life. It is not intended for the open market and is not positioned against external competitors. Success is measured by fewer lost local datasets and faster recovery after failed migrations.

AI is not required for the core workflow. If we add AI assistance later (e.g. auto-labelling snapshots or summarising changes), it must use local subscription AI on the developer's laptop rather than API calls. The app would generate prompts for manual use and accept pasted results, with a non-AI fallback by default.

---

## 2. Target Users

### Primary Persona — The Laravel / PHP Developer

**Profile**: A developer working on one or several client projects simultaneously. Uses Laravel with MySQL or PostgreSQL. Works locally via MAMP, Laravel Valet, Laravel Herd, or Docker. Runs migrations regularly as part of feature development.

**Key frustrations**:
- Running `php artisan migrate` and having the rollback fail, leaving the database in a broken intermediate state.
- Spending 20 minutes re-seeding a database to reproduce a specific bug after accidentally overwriting it.
- Having no easy way to share a specific database state with a colleague for debugging.
- Forgetting to dump before a migration and only realising when it is too late.

**What they want from this tool**:
- Automatic snapshots triggered by `php artisan migrate` so they never have to think about it.
- Named snapshots with human-readable notes: "after payment refactor", "with 500 orders seeded".
- One-click restore with a clear preview of what will be overwritten.
- The ability to export a snapshot as a Laravel seeder so they can share database states without sharing dumps.

**Likely path to tool**: GitHub, Laravel News, r/laravel, word of mouth from other developers.

---

### Secondary Persona — The Full-Stack Agency Developer

**Profile**: Works across multiple client projects, each with its own database configuration. Often switches between projects several times per day. Works with a small team (2–5 developers) where local database states need to occasionally be shared or replicated.

**Key frustrations**:
- Managing connection details for 8 different projects across MySQL and PostgreSQL.
- Onboarding a new developer who needs realistic but sanitised data rather than a production dump.
- Reproducing a client-reported bug that only happens with a very specific data configuration.

**What they want from this tool**:
- Connection profiles with per-project organisation and quick switching.
- Snapshot branching to run two different test scenarios without losing either state.
- Seeder export that strips PII from snapshots before sharing with new team members.
- Storage management to stop old snapshots consuming disk space.

**Likely path to tool**: Same channels as above, plus agency-focused newsletters and Slack communities.

---

### Tertiary Persona — The Solo SaaS Developer

**Profile**: Building their own product, working alone, iterating rapidly. May use any combination of MySQL, PostgreSQL, or SQLite. Prioritises speed of iteration above all else.

**Key frustrations**:
- The cognitive overhead of remembering to snapshot before experiments.
- SQLite databases in particular being easy to corrupt or accidentally overwrite.
- Wanting to test "what does the app look like with 10,000 records" vs "what does it look like with 50" without choosing between the two states.

**What they want from this tool**:
- Near-zero friction snapshots — ideally automatic.
- Branching to support parallel experimentation.
- Fast restore times that do not interrupt their flow.

---

### Who This Tool Is NOT For

- **Production database administrators** — this tool is explicitly scoped to local development.
- **Enterprise data-ops teams** — DBSnapper and similar tools serve that space adequately.
- **Developers working exclusively with cloud databases** — the tool targets local connections only, at least initially.
- **Non-technical users** — this is a developer tool and makes no attempt to hide that.

---

## 3. Competitive Analysis

**Internal scope note**: retained for awareness only. This tool is for internal agency use and is not intended to compete in the open market.

### Comparison Table

| Feature | Database Snapshot Manager | Stellar | DSLR | DBSnapper | DBeaver / Beekeeper |
|---|---|---|---|---|---|
| **Interface** | Desktop GUI | CLI only | CLI only | CLI + CI integrations | Desktop GUI |
| **MySQL support** | Yes | Partial | No | Yes | Yes |
| **PostgreSQL support** | Yes | Yes | Yes | Yes | Yes |
| **SQLite support** | Yes | No | No | No | Yes |
| **Named snapshots** | Yes | Yes | Yes | Yes | Basic backup only |
| **Row-level diff viewer** | Yes | No | No | No | Limited |
| **Snapshot branching** | Yes | No | No | No | No |
| **Auto-snapshot hooks** | Yes (migrate hook) | No | No | No | No |
| **Docker awareness** | Yes | No | No | No | No |
| **Seeder / sanitised export** | Yes (Laravel) | No | No | Yes (subsetting) | No |
| **Storage management UI** | Yes | No | No | Yes | No |
| **Pricing** | TBD (free tier likely) | Free / open source | Free / open source | Enterprise ($15k/mo) | Free / paid tiers |
| **Target user** | Solo dev / small agency | Power users | Power users | Enterprise / data-ops | General database work |
| **Laravel-specific features** | Yes | No | No | No | No |
| **Local-first / offline** | Yes | Yes | Yes | Partial | Yes |

### Competitive Assessment

**Stellar** is the closest technical ancestor. Its headline feature — restoring PostgreSQL at 140x the speed of pg_dump/pg_restore by using PostgreSQL's template database mechanism — is genuinely impressive. However it is Python-based, CLI-only, has no diff capability, no branching, and MySQL support is partial and maintained reluctantly. It solves a narrow problem well and has not been actively developed in several years. There is no credible GUI competitor in this space building on top of Stellar's approach.

**DSLR** is similarly fast for PostgreSQL specifically, using filesystem-level copy rather than dump/restore. It is elegant within its constraints but is explicitly PostgreSQL-only and CLI-only. Not a threat; potentially an inspiration for implementation approach.

**DBSnapper** occupies an entirely different market position. At $15,000 per month for self-hosted enterprise, it is not competing for the same customers. Its features — subsetting, sanitisation, compliance workflows, Terraform integration, GitHub Actions — are built for data-ops and security teams, not individual developers. It validates that there is commercial appetite in the broader snapshot management space, but it leaves the developer tooling segment completely unaddressed.

**DBeaver and Beekeeper Studio** are general-purpose database GUIs. They have backup and restore functionality in the sense that any database tool does, but they have no concept of a snapshot management workflow — no naming, no branching, no diff viewer, no integration with development lifecycle events like migrations. They are not competitors; they are the tools developers already use alongside which this product would sit.

### Unique Value Proposition

Database Snapshot Manager is the only tool that combines:

1. A visual desktop GUI (not CLI)
2. Cross-database support (MySQL, PostgreSQL, SQLite)
3. Row-level diff between named snapshots
4. Snapshot branching for parallel experimentation
5. Development lifecycle integration (pre-migration hooks)
6. Laravel-specific seeder export
7. Docker-aware dump execution
8. Local-first, zero-cloud, single-developer focus

No existing tool offers more than two of these in combination. The gap is real.

---

## 4. Core Features

### 4.1 Connection Profiles

**Description**: Save and manage multiple local database connections across projects. Each profile stores the connection type (MySQL, PostgreSQL, SQLite), host, port, credentials, and optional notes. Profiles are organised by project and can be quickly switched between.

**User story**: As a developer working across multiple client projects, I want to save my database connection details once per project so that I can switch between them without re-entering credentials every time.

**Acceptance criteria**:
- Can create, edit, and delete connection profiles.
- Profiles support MySQL, PostgreSQL, and SQLite.
- SQLite profiles require only a file path.
- Connections can be tested before saving.
- Profiles are stored locally and encrypted at rest (credentials only).
- Projects can be used as a grouping layer above connections.

**Complexity**: Low. Well-understood problem with established patterns.

---

### 4.2 Named Snapshots

**Description**: Create a snapshot of the entire database with a custom name and optional note. Snapshots are stored as compressed dump files with metadata recorded in a local SQLite store. One-click creation from the project dashboard.

**User story**: As a developer about to run a risky migration, I want to create a named snapshot in one click so that I have a clear restore point if something goes wrong.

**Acceptance criteria**:
- Snapshot creation takes a name (required) and note (optional).
- Snapshot is created using mysqldump, pg_dump, or file copy for SQLite.
- Progress is shown during dump creation.
- Snapshot appears immediately in the browser with timestamp, name, note, and file size.
- Snapshot creation does not require the application to remain open (background task with notification).

**Complexity**: Medium. Core functionality; requires reliable subprocess handling in Rust and good error reporting.

---

### 4.3 Instant Restore

**Description**: Restore any snapshot with a confirmation step that shows a diff preview of what will change. The developer must explicitly confirm before any data is overwritten. Restore uses the same dump tooling as creation.

**User story**: As a developer whose migration has corrupted the database, I want to restore my last snapshot with a clear preview of what will be overwritten so that I can verify I am restoring the right state before committing.

**Acceptance criteria**:
- Restore requires explicit confirmation.
- Confirmation step shows: snapshot name, creation timestamp, current database state (row counts per table), and what the restored state will look like.
- Restore progress is shown.
- On failure, the user is notified with the raw error output for debugging.
- Cannot restore a snapshot to a different database without an explicit override warning.

**Complexity**: Medium. The diff preview before restore requires a fast lightweight comparison that does not require loading full snapshot data.

---

### 4.4 Row-Level Diff Viewer

**Description**: Compare any two snapshots side-by-side with table-level filtering. Differences are shown at row level — added rows in green, removed rows in red, changed cells highlighted. Tables with no changes are collapsed by default.

**User story**: As a developer reviewing the impact of a data migration, I want to see exactly which rows changed between two snapshots so that I can verify the migration behaved as expected.

**Acceptance criteria**:
- Can select any two snapshots from the same connection for comparison.
- Diff is presented table by table with changed tables surfaced first.
- Added rows, removed rows, and modified rows are visually distinct.
- Modified rows show column-level highlighting on changed cells.
- Large tables with many changes can be paginated.
- Diff can be filtered to a single table.
- Diff can be exported as a CSV or JSON report.

**Complexity**: High. Diffing large databases row by row is computationally intensive and requires careful decisions about what to load into memory vs what to stream. This is the technically riskiest feature.

---

### 4.5 Auto-Snapshot Hooks

**Description**: A companion shell function (bash/zsh) that wraps `php artisan migrate` to automatically trigger a snapshot before the migration runs. The snapshot is named automatically with the timestamp and a "pre-migrate" prefix, with the optional addition of the migration filename.

**User story**: As a developer who often forgets to snapshot before running migrations, I want my terminal to automatically create a snapshot whenever I run `php artisan migrate` so that I always have a restore point without needing to remember.

**Acceptance criteria**:
- Shell hook is installable via a one-line command generated by the app.
- Hook detects the current project directory and matches it to a connection profile.
- Hook triggers a snapshot via the app's local API or CLI bridge before the migration runs.
- If the app is not running, the hook either launches it silently or falls back gracefully with a warning.
- Hook does not block the migration — it runs asynchronously where possible, or with a configurable timeout.
- Snapshots created by the hook are labelled clearly in the browser.

**Complexity**: Medium-High. Requires a local IPC mechanism (Unix socket or local HTTP) between the shell and the Tauri app. Cross-platform considerations (bash vs zsh vs fish) add complexity.

---

### 4.6 Snapshot Branching

**Description**: Fork any snapshot into a named branch. Changes made to the database while on a branch do not affect the main snapshot history. Branches can be created from any snapshot, allowing multiple scenarios to be tested from the same base state.

**User story**: As a developer testing two different approaches to a feature, I want to branch my current database state so that I can test both approaches independently and restore either branch without losing the other.

**Acceptance criteria**:
- Any snapshot can be branched with a name.
- The branch is represented as a separate snapshot lineage in the UI.
- Switching branches restores the database to the branch's most recent state.
- Branches can be merged (manual, not automatic — just a restore + rename workflow).
- Branches can be deleted, with confirmation.
- The UI clearly shows which branch is currently active.

**Complexity**: High. The branching concept itself is straightforward (it is just forked snapshot history) but the UI for representing branches clearly without overwhelming the user requires careful design. This is more of a UX challenge than a technical one.

---

### 4.7 Seeder Export

**Description**: Export any snapshot as a set of Laravel seeders. The export process sanitises PII fields (configurable per table), replaces real values with factory-compatible placeholders, and generates valid PHP seeder classes that can be dropped into a Laravel project.

**User story**: As a developer onboarding a new team member, I want to export a sanitised version of my local database state as Laravel seeders so that the new developer can replicate a realistic dataset without receiving any real user data.

**Acceptance criteria**:
- Any snapshot can be exported as seeders.
- PII sanitisation rules are configurable per table and column (e.g. "replace users.email with faker->email").
- Generated seeders are valid PHP and compatible with Laravel's standard seeder structure.
- Export respects foreign key relationships and generates seeders in the correct insertion order.
- Output is a zip of PHP files ready to drop into `database/seeders/`.
- Integration note: this feature may share logic with the standalone Laravel seeder generator project.

**Complexity**: Medium-High. Generating correct, relationship-aware seeders from an arbitrary database schema is non-trivial. PII detection heuristics (for auto-suggesting sanitisation rules) add further complexity but are optional for MVP.

---

### 4.8 Docker Awareness

**Description**: The app detects running Docker containers by name and transparently executes dump commands inside the appropriate container rather than against the host. No manual configuration required if the container is named predictably.

**User story**: As a developer running MySQL inside a Docker container, I want snapshots to work without me having to configure the dump command manually so that Docker-based projects work identically to local ones.

**Acceptance criteria**:
- App lists running Docker containers on startup and when creating a snapshot.
- If a connection profile matches a running container (by host, port, or container name heuristic), it offers to exec the dump inside the container.
- `docker exec` is used transparently — the developer does not need to write the command.
- Falls back to host-level connection if Docker is not running or the container is not found.
- Works with both mysqldump inside a MySQL container and pg_dump inside a PostgreSQL container.

**Complexity**: Medium. Docker CLI is well-documented and `docker exec` is straightforward. The heuristic for matching a connection profile to a container is the fiddliest part.

---

### 4.9 Storage Management

**Description**: A dedicated storage view shows all snapshots across all projects, with file sizes, creation dates, and the ability to set per-project retention limits. Auto-clean can be configured to remove snapshots older than a set number of days or beyond a maximum count per project.

**User story**: As a developer who has been using the tool for six months, I want to see how much disk space my snapshots are consuming and automatically remove old ones so that my machine does not run out of storage.

**Acceptance criteria**:
- Storage view shows total disk usage and per-project breakdown.
- Individual snapshots show their compressed size.
- Retention rules can be set per project (e.g. "keep last 10" or "keep last 30 days").
- Auto-clean runs on a configurable schedule or on app launch.
- Manual deletion with confirmation.
- Snapshots that are referenced by branches cannot be deleted without first deleting the branch.

**Complexity**: Low. Mostly UI and file system operations.

---

## 5. Technical Architecture

### Overview

The application follows the standard Tauri 2 architecture: a Rust backend handling all system-level operations, and a Vue 3 frontend handling all UI. The two communicate via Tauri's command system (invoke/emit). All data storage is local and offline-first.

### Technology Choices

**Tauri 2 + Rust (backend)**
- Handles subprocess execution (mysqldump, pg_dump, docker exec).
- Manages the local SQLite metadata store.
- Handles file I/O for snapshot storage (compressed dump files).
- Exposes a local Unix socket or HTTP server for the shell hook IPC.
- Responsible for encryption of stored credentials.

**Vue 3 + Vite (frontend)**
- Connection profile management UI.
- Snapshot browser and detail views.
- Row-level diff viewer (the most UI-complex component).
- Branch visualisation.
- Storage management dashboard.

**SQLite (metadata store)**
- Stores connection profiles (credentials encrypted separately).
- Stores snapshot metadata: name, note, timestamp, file path, file size, parent snapshot ID (for branching), tags.
- Does not store the actual database dump content — that lives as files on disk.

**mysqldump / pg_dump / SQLite file copy**
- The actual dump mechanism is the existing native tooling.
- For MySQL and PostgreSQL, the app shells out to the appropriate dump utility.
- For SQLite, a simple file copy (with WAL checkpoint) is sufficient.
- This is a pragmatic choice: it avoids implementing dump logic in Rust and ensures compatibility with all database versions.

**Docker CLI**
- Accessed via shell subprocess.
- `docker ps` to list running containers.
- `docker exec` to run dump commands inside containers.
- No Docker SDK dependency — keeps the dependency footprint minimal.

### Data Flow — Snapshot Creation

```bash
User clicks "Create Snapshot"
    -> Frontend calls Tauri invoke("create_snapshot", { profile_id, name, note })
    -> Rust handler resolves connection profile (decrypts credentials)
    -> Rust checks if Docker container is involved (docker ps)
    -> Rust constructs dump command (mysqldump / pg_dump / file copy)
    -> Rust executes dump command, streaming output to a temp file
    -> On success: temp file is compressed and moved to snapshot storage directory
    -> SQLite metadata record is created (name, note, timestamp, file path, size, parent_id)
    -> Frontend receives success event and refreshes snapshot list
    -> On failure: temp file is deleted, error is surfaced with raw command output
```

### Data Flow — Row-Level Diff

```bash
User selects two snapshots for diff
    -> Frontend calls Tauri invoke("create_diff", { snapshot_id_a, snapshot_id_b })
    -> Rust decompresses both snapshots to temp directories
    -> Rust parses dump files table by table (streaming, not full load)
    -> For each table: Rust computes row-level diff using hash comparison
    -> Diff result is written to a temp SQLite database (added/removed/changed rows)
    -> Frontend paginates through diff results via additional invoke calls
    -> Temp files are cleaned up when diff view is closed
```

The diff computation is the most performance-sensitive operation. Hashing rows rather than full string comparison reduces memory pressure. The decision to write diff results to a temp SQLite database rather than keeping them in memory allows arbitrarily large diffs to be navigated without loading everything at once.

### Data Flow — Shell Hook IPC

```bash
Developer runs "php artisan migrate" in terminal
    -> Shell hook intercepts the command
    -> Hook sends HTTP POST to localhost:PORT/snapshot (local only, not exposed externally)
    -> Tauri app receives request on its local HTTP listener
    -> App matches the request's working directory to a connection profile
    -> App creates a snapshot in the background (named "pre-migrate [timestamp]")
    -> App sends response to shell hook (success or error)
    -> Shell hook proceeds with the actual migrate command
    -> If app is not running: hook logs a warning and proceeds with migration
```

### Storage Layout

```text
~/Library/Application Support/DatabaseSnapshotManager/   (macOS)
    metadata.db                  (SQLite metadata store)
    snapshots/
        [project-slug]/
            [snapshot-id].sql.gz
            [snapshot-id].sql.gz
        [project-slug]/
            ...
    tmp/                         (cleaned on startup)
```

### Security Considerations

- Database credentials are stored using the OS keychain (via Tauri's keyring plugin) rather than in plaintext in SQLite.
- The local HTTP listener for shell hook IPC binds to 127.0.0.1 only and uses a per-session token to prevent local privilege escalation.
- Snapshot files are stored locally on the developer's machine — no cloud sync, no telemetry.
- Seeder export sanitisation is the developer's responsibility to configure correctly; the app provides tooling but cannot guarantee PII removal without correct configuration.

---

## 6. Risk Assessment

### Technical Risks

**Risk: Diff performance on large databases**
Diffing two snapshots of a database with millions of rows will be slow. The row-level diff viewer is the feature most likely to feel broken on real-world projects.
*Mitigation*: Set clear expectations in the UI (show estimated diff time before starting). Implement streaming diff with early results. Consider limiting diff to tables under a configurable row count threshold, with a manual override. Make diff a background operation with progress indication.

**Risk: mysqldump / pg_dump version incompatibility**
Developers use a wide range of MySQL and PostgreSQL versions. Dump formats change between major versions, and restoring a dump created with one version to a server running a different version can fail silently or produce corrupted data.
*Mitigation*: Record the dump utility version and server version in snapshot metadata. Warn the user if restoring a snapshot created with a different major version. Document this limitation clearly.

**Risk: Docker container detection heuristics are fragile**
The logic for matching a connection profile to a running Docker container will fail for non-standard container naming conventions and complex Docker Compose setups.
*Mitigation*: Make Docker detection opt-in and configurable rather than fully automatic. Allow the developer to explicitly associate a connection profile with a container name. Treat auto-detection as a convenience, not a requirement.

**Risk: Shell hook reliability across environments**
The pre-migrate shell hook needs to work across bash, zsh, and fish, on macOS and Linux, across different Laravel project structures, and when the Tauri app may or may not be running.
*Mitigation*: Scope Phase 1 to bash/zsh on macOS only. Make the hook fail gracefully (log a warning, proceed with migration) rather than blocking. Test extensively on common Laravel development setups (Valet, Herd, Docker Compose).

**Risk: SQLite dump "copy" approach and WAL mode**
Copying a SQLite file while it is in use (WAL mode) requires a proper checkpoint, not just a file copy. Getting this wrong produces a corrupt snapshot.
*Mitigation*: Use the SQLite `VACUUM INTO` command rather than file copy. This creates a defragmented, checkpoint-complete copy atomically. Well-established approach.

**Risk: Seeder export for complex schemas**
Generating correct seeders for databases with complex foreign key relationships, polymorphic relationships, or unusual column types (JSON, geometry, binary) is genuinely difficult.
*Mitigation*: Scope seeder export to standard Laravel Eloquent relationship patterns initially. Document unsupported column types explicitly. Add type-specific handling incrementally.

---

### Adoption Risks

**Risk: Low internal adoption**
If the tool adds friction or feels slower than manual dumps, it will not become part of the daily workflow.
*Mitigation*: Keep snapshot creation one click, auto-hook into migrations where possible, and ensure restore is faster than re-seeding.

**Risk: Competing workflows already exist**
Some developers rely on Docker volume snapshots or custom scripts and may not switch.
*Mitigation*: Integrate with existing workflows rather than replacing them outright (e.g. hook into `php artisan migrate`).

**Risk: Scope creep**
Trying to solve every database edge case risks delaying useful internal value.
*Mitigation*: Ship the core snapshot/restore loop first, then expand based on internal feedback.

---

## 7. MVP Scope

The MVP is Phase 1, stripped to the minimum that delivers real value and can be tested by real users.

### MVP Must-Have

- **Connection profiles**: MySQL, PostgreSQL, and SQLite. Create, edit, delete, test connection. No project grouping required initially.
- **Snapshot creation**: Named snapshot with optional note. Uses mysqldump/pg_dump/VACUUM INTO. Shows progress. Stores metadata in SQLite.
- **Snapshot browser**: List of all snapshots for the active connection. Shows name, note, timestamp, size. Sorted by date descending.
- **Basic restore**: Restore any snapshot with a confirmation step. Shows snapshot details before confirming. No diff preview required for MVP.
- **Snapshot deletion**: Delete individual snapshots with confirmation.
- **macOS only**: Do not attempt cross-platform in Phase 1. The target user base is primarily macOS.

### MVP Nice-to-Have (cut if timeline slips)

- Snapshot search and filtering by name/note.
- Basic storage usage view (total size per project).
- Compression of dump files.

### MVP Explicitly Out of Scope

- Row-level diff viewer.
- Snapshot branching.
- Auto-snapshot hooks.
- Docker awareness.
- Seeder export.
- Linux or Windows support.
- Team sharing.
- Cloud sync of any kind.

### What Makes the MVP Shippable

The MVP is shippable when a developer can: install the app, add a connection profile, create a named snapshot of their local MySQL or PostgreSQL database, browse their snapshots, and restore one. That complete workflow, working reliably, is enough to be genuinely useful and to begin gathering user feedback.

### MVP Build Estimate

For a solo developer familiar with Tauri and Vue, the MVP represents approximately 6–10 weeks of part-time development, or 3–4 weeks full-time. The Rust subprocess handling and SQLite integration are the areas of highest unknown — budget extra time there.

---

## 8. Monetisation Strategy

This is an internal agency tool and is not intended for sale or external distribution. There is no monetisation model. The value case is measured in reduced data loss, faster recovery after failed migrations, and smoother onboarding.

**Cost profile (internal)**:

- Build and maintenance time.
- Disk usage for local snapshots.
- Occasional updates when database engines or migration tooling change.

---

## 9. Implementation Phases

### Phase 1 — Snapshot Core

**Goal**: Ship a working MVP that developers can use in real projects.

**Deliverables**:
- Connection profiles (MySQL, PostgreSQL, SQLite).
- Named snapshot creation with progress.
- Snapshot browser with search.
- Basic restore with confirmation.
- Snapshot deletion and storage view.
- macOS only.

**Dependencies**: Tauri 2 setup, Rust subprocess handling, SQLite integration, Vue 3 UI scaffolding.

**Estimated complexity**: Medium. The individual pieces are well-understood; the integration and error handling are where the time goes.

**Estimated timeline**: 6–10 weeks part-time.

**Key technical decisions to make in Phase 1**:
- Credential storage mechanism (OS keychain via Tauri plugin).
- Snapshot file format and compression (gzip is simplest; zstd for better ratio if worth the dependency).
- Dump file parsing approach for Phase 2 diff (ideally design the storage format now to support it).

---

### Phase 2 — Diff and Branch

**Goal**: Add the features that differentiate this tool from simple backup utilities.

**Deliverables**:
- Row-level diff viewer (the most complex feature in the entire product).
- Snapshot branching with branch switching UI.
- Snapshot tagging and advanced search.
- Branch merge (manual workflow) and branch deletion.

**Dependencies**: Phase 1 complete. Diff viewer requires decisions made in Phase 1 about dump file format.

**Estimated complexity**: High. The diff viewer is the hardest feature in the product. Branching is conceptually simple but UX-challenging.

**Estimated timeline**: 10–16 weeks part-time, heavily weighted towards the diff viewer.

**Key risks**: Diff performance on large databases (see Risk Assessment). Strongly recommended to prototype the diff engine against a real-world 500MB MySQL dump before committing to the full implementation.

---

### Phase 3 — Integration

**Goal**: Make the tool feel native to the Laravel development workflow and extend reach to Docker-based setups.

**Deliverables**:
- Pre-migrate auto-snapshot hook (bash/zsh on macOS).
- Docker container detection and exec support.
- Seeder export with PII sanitisation configuration.
- Laravel project auto-detection (reads .env for DB credentials).
- Windows and Linux support (if demand justifies).

**Dependencies**: Phase 2 complete. Seeder export may share logic with the separate Laravel seeder generator project.

**Estimated complexity**: Medium-High. Each feature is independently manageable but collectively Phase 3 is broad.

**Estimated timeline**: 8–12 weeks part-time.

**Key risks**: Shell hook compatibility across environments. Seeder export quality for complex schemas. Laravel auto-detection brittleness across different project structures.

---

### Phase 4 — Team and Commercial Features (Optional)

**Goal**: Extend the tool for small team and agency use cases; establish commercial viability.

**Deliverables**:
- Snapshot export/import bundles for sharing between developers.
- Team licence management.
- Advanced PII sanitisation presets.
- In-app update mechanism.
- Marketing website and launch campaign.

**Dependencies**: Phase 3 complete. Requires product-market fit signal from Phase 1–2 launch.

**Estimated timeline**: 8–10 weeks part-time, plus ongoing marketing.

---

## 10. Success Metrics

### Phase 1 Success (within 3 months of internal rollout)

- **Internal adoption**: At least 5 developers use the tool weekly.
- **Activation rate**: 60%+ of internal installs create at least one snapshot.
- **Retention signal**: 30%+ of activated users create a second snapshot within 7 days.
- **Error rate**: Fewer than 5% of snapshot creation attempts fail due to app errors (as opposed to user configuration errors).

### Phase 2 Success (within 6 months of Phase 2 launch)

- **Diff viewer usage**: 40%+ of active users have used the diff viewer at least once.
- **Branch usage**: 20%+ of active users have created at least one branch.

### Phase 3 Success

- **Hook adoption**: 50%+ of active users have installed the pre-migrate hook.
- **Seeder export usage**: Used at least once by 25%+ of active users.
- **Docker usage**: 30%+ of connections use Docker-aware mode where applicable.

### Long-Term Health Metrics

- **Support burden**: Average time to respond to internal queries stays under 24 hours without consuming more than 2 hours per week total.
- **Reliability**: Restore success rate above 95% for supported database engines.

### Anti-Metrics (Signs Things Are Going Wrong)

- High download count with low activation suggests the landing page overpromises or the onboarding is broken.
- High activation with low retention suggests the tool is used once out of curiosity but not in real workflow.
- High crash/error rate in Phase 1 suggests the subprocess handling needs more investment before adding features.

---

## 11. Open Questions

### Product Questions

1. **Should branching be represented as a tree or a flat list?** A tree visualisation is more accurate but adds significant UI complexity and may confuse users who are not familiar with git branching. A flat list of named branches with a "created from" reference may be sufficient for the use cases targeted.

2. **How should the tool handle databases with sensitive production data that has been copied locally?** Some developers run partial production copies locally for debugging. The tool should not actively prevent this but may want to display a configurable warning when restoring snapshots of databases above a certain size or age, as a prompt to consider PII.

3. **Is SQLite support actually worth the Phase 1 investment?** SQLite projects typically have no separation between the database and the application code — the file is just copied. The VACUUM INTO approach is clean, but the user base for SQLite in active development (as opposed to mobile or embedded) may be smaller than MySQL/PostgreSQL. Consider deferring SQLite to Phase 2.

4. **What is the right level of granularity for branching?** Should branches be independent snapshot histories (like git branches) or should they be a simple fork-and-label of a single snapshot? The simpler model is easier to implement and explain; the richer model is more powerful but significantly harder to build and communicate.

5. **Should the diff viewer attempt schema diffing as well as data diffing?** Schema changes (added columns, changed types) are often more important to surface than row-level data changes after a migration. Adding schema diff would increase the value of Phase 2 significantly but also its complexity.

### Technical Questions

6. **What is the best approach to parsing mysqldump output for the diff engine?** mysqldump output is not a stable, easily-parsed format — it includes comments, conditional statements, and version-specific extensions. Alternatives include using the database's native query interface to read rows directly from a restored snapshot into a temp database, rather than parsing the dump file. This is slower but more reliable.

7. **How should the local HTTP listener for the shell hook be secured?** A random per-session port is cumbersome. A fixed port with a per-session HMAC token in the request header is more practical. The threat model is local processes on the same machine — the security requirement is modest but not zero.

8. **Should the app attempt to detect mysqldump / pg_dump on the PATH, or bundle its own?** Bundling ensures version consistency but significantly increases the app bundle size and complicates licensing. Detecting system tools is simpler but may fail if the developer's PATH is not set up correctly in the app's subprocess environment (a known macOS Tauri issue with GUI apps that do not inherit the user's shell PATH).

9. **How should credentials be handled when mysqldump / pg_dump are invoked as subprocesses?** Passing credentials as command-line arguments exposes them in process listings. Using a config file (MySQL's .my.cnf, PostgreSQL's .pgpass) is safer but requires the app to write and clean up temp config files. This needs a deliberate decision.

10. **What is the minimum viable approach to the diff viewer that ships Phase 2 without excessive risk?** One option: limit Phase 2 diff to databases under 100MB and tables under 50,000 rows, with a clear message when limits are exceeded. This allows shipping a useful diff viewer without solving the full performance problem, and buys time to optimise in Phase 3.

### Internal Questions

11. **Should this be shared internally as a standard tool?** If so, which teams should pilot first?

12. **Is there an integration opportunity with existing internal tooling (Herd/Valet configs, project bootstrap scripts)?** If so, prioritise low-friction hooks.

13. **What is the timeline pressure?** Is there a reason to rush Phase 1 internally, or can it follow the next internal tooling sprint?

14. **How does this relate to the Laravel seeder generator project?** If both tools are being developed in parallel, there may be shared logic (seeder generation, database introspection) that should be extracted into a shared library rather than duplicated. Planning this connection early avoids significant refactoring later.

---

*Document last updated: 18 February 2026*
*Status: Concept enrichment complete — ready for Phase 1 scoping and prototype work*
