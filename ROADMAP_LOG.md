# Amber Roadmap Log

## Cycle: 2026-03-25 06:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber at 15 pending (13 functional + 2 design system) — at the rebalancing threshold. No additions warranted; category coverage is complete across Features, UX/UI, Performance, Quality, Distribution, and Innovation. No stale items (all added within 6 days). Scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest development pair. The concurrent operation guard (P2, S) is a quick safety win worth prioritising alongside them. Twelve completed items remain misplaced under the "Pending" heading — structural cleanup recommended.

## Cycle: 2026-03-24 05:00
- **Items added:**
  - [Quality] Add operation history log recording all snapshot, restore, and cleanup actions with timestamps for audit trail and debugging (P3, S)
- **Items archived:** none
- **Observations:** Amber at 14 pending (12 functional + 2 design system). Added one Quality item filling an observability gap: as automated features (scheduled snapshots, pre-restore safety snapshots, retention cleanup) progress, operations will happen without direct user initiation and need an audit trail for debugging. The operation log provides the accountability layer that autonomous database operations require. Scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest development pair. No stale items. Twelve completed items remain under the "Pending" heading — structural cleanup recommended.

## Cycle: 2026-03-24 23:30
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber at 15 pending (13 functional + 2 design system) — at the rebalancing threshold. Category coverage is complete across Features, UX/UI, Performance, Quality, Distribution, and Innovation. No stale items (all added within 6 days). Scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest development pair for transforming Amber into a proactive database guardian. Twelve completed items remain misplaced under the "Pending" heading — structural cleanup recommended.

## Cycle: 2026-03-25 01:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber at 15 pending (13 functional + 2 design system) — at the rebalancing threshold. No additions warranted. Scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest development pair. The auto-updater (P2, M) aligns with the portfolio-wide distribution pattern. Twelve completed items remain misplaced under the "Pending" heading — structural cleanup would improve readability. No stale items.

## Cycle: 2026-03-24 23:00
- **Items added:**
  - [UX/UI] Add native macOS notification for completed long-running snapshot and restore operations (P3, S)
- **Items archived:** none
- **Observations:** Amber has 13 pending functional items + 2 design system = 15 total — at the rebalancing threshold. Added one UX/UI item addressing a real workflow gap: users switching away during long dump/restore operations have no way to know when the operation completes. Native notifications close this feedback loop with minimal implementation effort. The concurrent operation guard (P2, S) and scheduled snapshots (P2, M) remain the strongest development pair. No stale items. Next cycle should consider archiving lower-priority items if any new additions are needed.

## Cycle: 2026-03-24 21:00
- **Items added:**
  - [Quality] Add concurrent operation guard preventing simultaneous snapshot and restore operations on the same profile (P2, S)
- **Items archived:** none
- **Observations:** Amber has 12 pending functional items + 2 design system = 14 total — approaching the 15-item rebalancing threshold. Added one Quality item addressing a data safety gap — no concurrency guard exists for simultaneous operations on the same profile, which could corrupt database state. This is a small but critical safety feature that complements existing integrity checks. No stale items (all added within the last 5 days). Scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest development pair.

## Cycle: 2026-03-24 18:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber has 11 pending functional items + 2 design system = 13 total — well below the rebalancing threshold. No additions warranted — the roadmap is comprehensive across all categories and already received 1 item earlier today (pre-restore safety snapshot). Scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest development pair. The auto-updater (P2, M) continues to align with the portfolio-wide distribution pattern.

## Cycle: 2026-03-24 15:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber has 11 pending functional items + 2 design system = 13 total — approaching the 15-item threshold. No additions warranted — the roadmap is comprehensive across all categories. Twelve completed items remain under the "Pending" heading rather than the "Completed" section — an organisational inconsistency noted in previous cycles. The scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest pair for transforming Amber from a manual tool into a proactive, always-running database guardian. The auto-updater (P2, M) aligns with the portfolio-wide distribution pattern.

## Cycle: 2026-03-24 09:00
- **Items added:**
  - [Quality] Add automatic pre-restore safety snapshot capturing current database state before any restore operation (P2, S)
- **Items archived:** none
- **Observations:** Amber has 12 pending items (10 functional + 2 design system) — approaching the 15-item threshold but still within range. Added one Quality item addressing a data safety gap — restoring a snapshot currently overwrites the current database state with no automatic rollback path. A pre-restore safety snapshot is a small but high-impact safety net that leverages existing snapshot infrastructure. Scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest pair for the next development session. The auto-updater (P2, M) continues to align with the portfolio-wide distribution pattern.

## Cycle: 2026-03-23 21:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber has 9 pending functional items + 2 design system = 11 total, well below the 15-item threshold. No additions warranted — the roadmap is comprehensive across all categories including the recently added auto-updater (Distribution). The scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest pair for the next build session. Pre-migration auto-capture (P3, M) remains the highest-differentiation feature.

## Cycle: 2026-03-23 15:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber has 8 functional pending items + 2 design system = 10 total — healthy and well below the 15-item threshold. No additions warranted this cycle. The roadmap is comprehensive across all categories. Twelve completed items remain under the "Pending" heading rather than the "Completed" section — an organisational inconsistency that should be corrected during the next development session. The scheduled snapshots (P2, M) and connection health monitoring (P2, S) remain the strongest pair for the next build session — together they transform Amber from a manual tool into a proactive, always-running database guardian. The auto-updater (P2, M) aligns with the portfolio-wide distribution pattern (Grove, Fuse, Drift all have similar items pending or planned). Pre-migration auto-capture (P3, M) remains the highest-differentiation feature for a database snapshot tool.

## Cycle: 2026-03-23 09:00
- **Items added:**
  - [Distribution] Add Tauri auto-updater with release notes display for seamless version delivery (P2, M)
- **Items archived:** none
- **Observations:** Added one item filling the Distribution category gap — Amber had no pending Distribution items after the first-run setup wizard was completed. The auto-updater aligns with the same items planned for Grove and Fuse, establishing a portfolio-wide update mechanism pattern. Amber has 8 functional pending items + 2 design system = 10 total, well within the 15-item threshold. Snapshot content browser was completed since last cycle. The scheduled snapshots item (P2, M) and connection health monitoring (P2, S) form the strongest pair for the next development session — together they make Amber a proactive, always-running guardian rather than a manual backup tool.

## Cycle: 2026-03-23 03:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber has 8 pending functional items + 2 design system = 10 total. No items added this cycle — 8 pending functional items is the joint-highest in the portfolio and adding more would risk scope bloat. The snapshot content browser was completed since last cycle (2026-03-22). Several items in the Pending section are marked as completed but remain under that heading — a consistency issue to note but not something this cycle should modify. Pre-migration auto-capture (P3, M) and scheduled snapshots (P2, M) remain the highest-value pending features as they would transform Amber from a manual tool into a proactive guardian. The next cycle should focus on checking for completion progress rather than adding items.

## Cycle: 2026-03-22 21:00
- **Items added:**
  - [Feature] Import external SQL dump files as managed snapshots (P3, S)
  - [Quality] Add disk space pre-flight check before snapshot creation (P2, S)
  - [UX/UI] Add snapshot name auto-generation with configurable templates (P3, S)
- **Items archived:** none
- **Observations:** Amber has 8 pending functional items + 2 design system = 10 total. The three additions address practical friction points in daily snapshot workflows. The disk space check (P2, S) prevents the most frustrating failure mode — running out of space mid-dump producing a corrupted snapshot — and pairs naturally with the existing size estimation feature. External dump import (P3, S) makes Amber the single management point for all database snapshots regardless of origin, valuable for teams where dumps come from CI, colleagues, or other tools. Name auto-generation (P3, S) reduces friction for frequent snapshot creation, especially when combined with the scheduled snapshots item (pending). The snapshot content browser was completed since last cycle. Pre-migration auto-capture (P3, M) and scheduled snapshots (P2, M) remain the highest-value pending features.

## Cycle: 2026-03-22 15:00
- **Items added:**
  - [Feature] Add scheduled automatic snapshots on configurable intervals per profile (P2, M)
  - [UX/UI] Add snapshot calendar view showing capture and restore history over time (P3, S)
  - [Quality] Add database connection health monitoring with proactive status indicator (P2, S)
- **Items archived:** none
- **Observations:** Amber had the fewest pending items in the portfolio (1 functional + 2 design system = 3 total) after the snapshot content browser was completed. The three additions address the next maturity layer: scheduled snapshots (P2, M) automate continuous protection beyond manual and pre-migration captures; connection health monitoring (P2, S) surfaces connectivity issues before they cause operation failures; and the calendar view (P3, S) provides temporal navigation as the snapshot catalogue grows. The scheduled snapshot item extends the existing retention policy infrastructure. Amber is now at 4 pending functional items + 2 design system = 6 total. The pre-migration auto-capture (P3, M) and scheduled snapshots (P2, M) together would make Amber an intelligent, proactive database guardian.

## Cycle: 2026-03-20 12:50
- **Items added:** none
- **Items completed:**
  - [UX/UI] Add snapshot size estimation before capture (P2, M) — completed 2026-03-20
  - [Performance] Implement streaming progress for large database dumps (P2, M) — completed 2026-03-20
  - [UX/UI] Add snapshot tagging and search (P3, S) — completed 2026-03-20
  - [Distribution] Add first-run setup wizard for database tool discovery (P2, M) — completed 2026-03-20
  - [Feature] Add snapshot retention policies and disk usage monitoring (P2, S) — completed 2026-03-20
  - [Feature] Compare schema differences between two snapshots (P2, M) — completed 2026-03-20
  - [UX/UI] Add snapshot restore dry-run preview (P2, S) — completed 2026-03-20
  - [Quality] Add database tool version compatibility checking (P2, S) — completed 2026-03-20
  - [Feature] Add snapshot export as portable SQL file (P3, S) — completed 2026-03-20
  - [Feature] Add snapshot restore to alternate database (P2, M) — completed 2026-03-20
  - [UX/UI] Add keyboard shortcuts (P2, S) — completed 2026-03-20
- **Items archived:** none
- **Observations:** Batch execution of all 11 remaining functional roadmap items. Amber now has comprehensive snapshot management: size estimation, streaming progress with table tracking, tagging and search, tool discovery wizard, retention policies with pinned protection, schema comparison between snapshots, restore dry-run preview, tool version compatibility checking, SQL export, alternate database restore, and keyboard shortcuts with help overlay. Remaining pending items: pre-migration auto-capture (P3/M, requires file watcher/shell hook), snapshot content browser (P3/M, requires SQL parsing from compressed files), and Design System Adoption section (2 items, needs Danny's input). Amber is now at 4 pending items (2 functional + 2 design system).

## Cycle: 2026-03-19 08:00
- **Items added:**
  - [Quality] Migrate persistence from localStorage to SQLite (P1, L)
  - [UX/UI] Add snapshot size estimation before capture (P2, M)
  - [Performance] Implement streaming progress for large database dumps (P2, M)
- **Items archived:** none
- **Observations:** Initial roadmap seeding. Amber is in active MVP development with solid architecture but using localStorage for persistence — the SQLite migration is the most impactful next step. The backend dump infrastructure exists but needs to be wired to real database tools with proper progress reporting.

## Cycle: 2026-03-19 15:00
- **Items added:**
  - [Feature] Implement real database dump and restore via subprocess execution (P1, L)
- **Items archived:** none
- **Observations:** Added the most critical functional gap — the app currently simulates all snapshot/restore operations without executing real database tools. This P1 Feature item complements the existing P1 SQLite migration: together they form the foundation for Amber to transition from prototype to functional tool. Remaining 3 items from initial seeding are still pending. Category coverage now includes Feature alongside Quality, UX/UI, and Performance.

## Cycle: 2026-03-19 22:00
- **Items added:**
  - [Quality] Add structured error handling for database connectivity failures (P2, S)
  - [UX/UI] Add snapshot tagging and search (P3, S)
  - [Distribution] Add first-run setup wizard for database tool discovery (P2, M)
- **Items archived:** none
- **Observations:** Filled the Distribution and additional Quality/UX gaps. The setup wizard (Distribution) is strategically important — it directly supports the P1 subprocess execution item by ensuring database tools are discoverable before users attempt their first real snapshot. Error handling for connectivity failures complements the existing P1 items by providing the user-facing error layer. Snapshot tagging is lower priority (P3) but earns its place as the snapshot catalogue will grow quickly once real dumps are operational. Amber now has 7 pending items across 5 categories; only Innovation is unrepresented, which is appropriate for an MVP-stage tool.

## Cycle: 2026-03-20 06:00
- **Items added:**
  - [Innovation] Add pre-migration automatic snapshot capture (P3, M)
- **Items archived:** none
- **Observations:** Filled the last remaining category gap (Innovation). Pre-migration auto-capture is the single highest-differentiation feature for a database snapshot tool — it protects users at the exact moment they need it most. Amber now has 11 pending items across all 6 categories plus Design System. No items are stale (all added 2026-03-19). The P1 cluster (SQLite migration + real subprocess execution) remains the critical path; this P3 Innovation item can be addressed once the foundation is solid.

## Cycle: 2026-03-20 12:00
- **Items added:**
  - [Feature] Add snapshot retention policies and disk usage monitoring (P2, S)
  - [Quality] Add snapshot integrity verification before restore operations (P2, S)
- **Items archived:** none
- **Observations:** Added two lifecycle management items that become critical once real database dumps are operational. Retention policies prevent disk exhaustion as snapshots accumulate — the Innovation auto-snapshot item mentions retention for automatic captures, but manual snapshots need the same governance. Integrity verification before restore protects users from silent data corruption, which is a non-negotiable for a database tool. Both items are deliberately small (S) as they build on existing snapshot metadata infrastructure. Amber now has 13 pending items (10 functional + 3 design system). The P1 cluster (SQLite migration + real subprocess execution) remains the critical path.

## Cycle: 2026-03-19 22:30
- **Items added (Design System Adoption section):**
  - [Foundation] Integrate @stuntrocket/ui shared component library and design tokens (P1, M)
  - [UI Migration] Replace all UI elements with @stuntrocket/ui shared components (P1, XL)
  - [Polish] Achieve full @stuntrocket/ui styleguide visual conformance (P2, L)
- **Items archived:** none
- **Observations:** Added Design System Adoption section with 3 staged items. Amber is unique in the portfolio as a vanilla TypeScript app with no framework — the UI migration may require a Vue 3 adoption or CSS-only consumption of @stuntrocket/ui. This is the most disruptive design adoption across all apps but also the most transformative, as Amber currently has the least visual polish. Foundation → Migration → Polish ordering ensures progressive adoption without blocking functional work on the existing P1 items.

## Cycle: 2026-03-20 18:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber is at 14 pending items (11 functional + 3 design system), just below the 15-item threshold. No items have moved to in-progress since the roadmap was seeded. Adding more items without execution progress would inflate the backlog without value. The P1 cluster (SQLite migration + real subprocess execution) remains the critical path and should be the focus of the next development session. Holding at 14 items until execution begins.

## Cycle: 2026-03-20 22:00
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber remains at 14 pending items (11 functional + 3 design system), one below the rebalancing threshold. No items have moved to in-progress since the roadmap was seeded — the portfolio-wide execution stall continues. Adding further items would inflate the backlog without value. The P1 cluster (SQLite migration + real subprocess execution) remains the critical path and the clear starting point for the first development session. Recommend starting with the subprocess execution item as it delivers the app's core value proposition; the SQLite migration can follow once real dump data needs a durable store.

## Cycle: 2026-03-21 14:00
- **Items added:**
  - [UX/UI] Add snapshot restore dry-run preview showing schema and data impact (P2, S)
  - [Quality] Add database tool version compatibility checking before operations (P2, S)
  - [Feature] Add snapshot export as portable SQL file for cross-tool compatibility (P3, S)
- **Items archived:** none
- **Observations:** Amber dropped to 9 pending items (7 functional + 2 design system) after four completions (SQLite migration, subprocess execution, error handling, integrity verification) — now well below the 15-item threshold. The three additions address the next layer of operational maturity for a database tool with real dump/restore capabilities: the dry-run preview protects users from schema-mismatched restores (critical after migrations), version compatibility checking prevents silent cross-version issues (a common pain point with mysqldump), and SQL export makes Amber's snapshots portable for team sharing and CI seeding. All three are small (S) and build on the now-operational snapshot infrastructure. Amber is now at 12 pending items (10 functional + 2 design system). The P2 cluster (streaming progress, size estimation, setup wizard, retention policies, restore preview, version checking) forms the strongest functional batch.

## Cycle: 2026-03-19 23:29
- **Items added:**
  - [Feature] Compare schema differences between two snapshots (P2, M)
- **Items archived:** none
- **Observations:** Added only 1 item this cycle — Amber is at 14 pending items (11 functional + 3 design system), approaching the 15-item threshold. The schema diff feature elevates Amber from a backup tool to a database change audit tool, which is a strong differentiator. It complements the P1 subprocess execution item — once real dumps work, comparing them becomes the natural next question. The P1 cluster (SQLite migration + real subprocess execution) remains the critical path. No items are stale (all < 1 day old).

## Cycle: 2026-03-20 22:30
- **Items added:**
  - [Feature] Add snapshot restore to alternate database for safe data inspection (P2, M)
  - [UX/UI] Add keyboard shortcuts for common snapshot operations (P2, S)
  - [Feature] Add snapshot content browser for inspecting data without restoring (P3, M)
- **Items archived:** none
- **Observations:** Three additions filling distinct gaps. Alternate restore (P2, M) addresses the single biggest usability limitation of the current restore workflow — it's always destructive, even when users only want to inspect old data. Keyboard shortcuts (P2, S) bring Amber in line with every other app in the portfolio, all of which have shortcuts implemented or planned. The content browser (P3, M) complements the existing restore dry-run preview by letting users inspect actual data, not just schema differences. Four items completed (SQLite migration, subprocess execution, error handling, integrity verification) demonstrate strong execution velocity. Amber is now at 15 pending items (13 functional + 2 design system) — at the rebalancing threshold. No items stale (all added within the past 2 days). The P2 cluster (streaming progress, size estimation, setup wizard, retention policies, schema compare, restore preview, version checking, alternate restore, keyboard shortcuts) is large at 9 items — recommend starting with keyboard shortcuts (P2, S) and streaming progress (P2, M) as the pair that delivers the most immediate UX improvement during daily snapshot operations.

## Cycle: 2026-03-20 20:30
- **Items added:** none
- **Items archived:** none
- **Observations:** Amber remains at 15 pending items (13 functional + 2 design system) — at the rebalancing threshold. Four completed items (SQLite migration, subprocess execution, error handling, integrity verification) show strong execution velocity. No additions warranted — the roadmap is comprehensive across all categories and adding further items without execution progress would inflate the backlog. The P2 cluster is the largest in the portfolio at 9 items, providing ample execution targets. Recommend starting with keyboard shortcuts (P2, S) as the quickest win, followed by streaming progress (P2, M) and the first-run setup wizard (P2, M) as the pair that delivers the best new-user experience.

## Cycle: 2026-03-24 09:00
- **Items added:** None
- **Items archived:** [UX/UI] Snapshot calendar view — visualisation with limited practical impact; list view with filtering covers temporal navigation needs
- **Observations:** Amber's roadmap is comprehensive with strong coverage across all categories. 13 functional pending items plus 2 design system items sits at the rebalancing threshold. The pre-migration auto-capture and scheduled snapshots (both pending) represent the app's most differentiating unrealised features. Priority focus should be on the concurrent operation guard (P2) and disk space pre-flight check (P2) — both prevent data corruption scenarios that would erode user trust in a database tool.
