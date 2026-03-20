# Amber Roadmap Log

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
  - [Polish] Achieve full Scooda styleguide visual conformance (P2, L)
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

## Cycle: 2026-03-19 23:29
- **Items added:**
  - [Feature] Compare schema differences between two snapshots (P2, M)
- **Items archived:** none
- **Observations:** Added only 1 item this cycle — Amber is at 14 pending items (11 functional + 3 design system), approaching the 15-item threshold. The schema diff feature elevates Amber from a backup tool to a database change audit tool, which is a strong differentiator. It complements the P1 subprocess execution item — once real dumps work, comparing them becomes the natural next question. The P1 cluster (SQLite migration + real subprocess execution) remains the critical path. No items are stale (all < 1 day old).
