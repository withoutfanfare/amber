# Amber — Public-Readiness TODO

> Companion to [`../Tauri Apps - Public Readiness Plan.md`](../Tauri%20Apps%20-%20Public%20Readiness%20Plan.md).
> Every item below is transcribed from that report — see **Source coverage** at the bottom to confirm nothing was dropped.

| | |
|---|---|
| Path (git root) | `amber/` |
| Remote | `withoutfanfare/amber` |
| Visibility | **PUBLIC** |
| Bundle ID | `com.dannyharding.amber` |
| Overall risk (report §10) | **Medium** |
| CSP | `null` (report §6.3) |

## How to use
`- [ ]` open · `- [x]` done · `- [~]` blocked / decision needed (note why). Phases mirror report §9. **P0 → P1 are gating.**

## §4 "Safely public" scorecard (transcribed from report §10)

| # | Criterion | Status | Note |
|---|---|---|---|
| 1 | No secrets / PII in git history | ✅ | clean (report §7) |
| 2 | No secrets / PII in working tree | ✅ | DB pw via `PGPASSWORD` env; creds in Keychain (§7) |
| 3 | LICENSE present | ❌ | none (§6.1) |
| 4 | Builds from clean clone | ❌ | `@stuntrocket/ui` via `file:../stuntrocket-ui` (§6.2) |
| 5 | CSP not null | ❌ | `csp: null` (§6.3) |
| 6 | Least-privilege capabilities | ✅ | `core:default`, `opener`, `window-state` (§7) |
| 7 | No dangerous code paths | ✅ | `Command::new` ssh/mysqldump/pg_dump, discrete args (§7) |
| 8 | No undisclosed telemetry | ✅ | no telemetry (§1) |
| 9 | No confidential client data | ✅ | (§10) |
| 10 | README adequate | ⚠️ | needs work (§10) |
| 11 | Secret-scanning in CI | ❌ | none (§6.7) |

## P0 — Incident response
_None for Amber_ (no incident; history and working tree clean).

## P1 — Blockers before publicising (gating)
- [ ] **Add LICENSE** to repo root — decision report §8.1 (default: MIT). Set `license` + `author` in `package.json`. (report §6.1, §9 P1.1)
- [ ] **Replace `@stuntrocket/ui` `file:` dep** — pending §8.2 distribution decision (default: publish to public npm). Remove `file:../stuntrocket-ui`. (report §6.2, §9 P1.2)
- [ ] **Add CSP** to `tauri.conf.json` → `app.security.csp` using report Appendix E baseline (adapt `connect-src`/`img-src` per app). (report §6.3, §9 P1.3)

## P2 — Security hardening
- [ ] **Secret-scanning CI** — add `gitleaks` pre-commit + GitHub Actions workflow (report Appendix C). (report §6.7, §9 P2.4)
- [ ] **Standardise `.gitignore`** to report Appendix D (Tauri+Vue+Rust). (report §6.7, §9 P2.6)
- [ ] **Resolve `npm audit` 1 high** — identify and patch/upgrade. (report §7 amber)
- [ ] **Wire `npm audit` / `cargo audit` into CI** with fail-on-high gate. (report §9 P2.7)
- [ ] **Review `StrictHostKeyChecking=accept-new`** (TOFU, mild MITM risk) in `src-tauri/src/dump.rs` — accept or tighten. (report §7 amber)

## P3 — Polish & privacy presentation
- [ ] **Bundle-ID decision** — `com.dannyharding.amber` → unified scheme (default `co.stuntrocket.amber`). ⚠️ decide **before** notarisation (report §6.5, §8.3, §9 P3.1).
- [ ] **Scrub `/Users/dannyharding/...`** from docs; review `SettingsView.vue` app-support path reference. (report §6.6, §7 amber)
- [ ] **Improve README** (currently ⚠️) — what it is + how to run. (report §7, §9 P3.4)
- [ ] **Add privacy statement** to README (report Appendix F). (report §9 P3.5)
- [ ] **Add `SECURITY.md`** (report Appendix G). (report §9 P3.6)

## Source coverage
Maps **every Amber mention in the main report** to a row above (all copied ✅).

| Report ref | What it says about Amber | Landed in | Copied |
|---|---|---|---|
| §2 table | path/remote/visibility/bundle id | header | ✅ |
| §6.1 | no LICENSE | P1 | ✅ |
| §6.2 | `@stuntrocket/ui` `file:` — unbuildable for cloners | P1 | ✅ |
| §6.3 | `csp: null` | P1 (scorecard #5) | ✅ |
| §6.5 | bundle id `com.dannyharding.amber` | P3 | ✅ |
| §6.6 | `/Users/dannyharding` paths; `SettingsView.vue` app-support ref | P3 | ✅ |
| §6.7 | no secret-scanning CI; weak gitignore | P2 | ✅ |
| §7 Amber | history/tree clean; PGPASSWORD; dump.rs; StrictHostKeyChecking TOFU; npm audit 1 high; fixes list | scorecard + P1/P2 | ✅ |
| §8.1 / §8.2 / §8.3 | licence / UI distribution / bundle-id decisions | P1/P3 | ✅ |
| §9 P1.1–P1.3 | LICENSE, UI, CSP | P1 | ✅ |
| §9 P2.4/P2.6/P2.7 | gitleaks, gitignore, audit CI | P2 | ✅ |
| §9 P3.1/P3.4/P3.5/P3.6 | bundle id, README, privacy, SECURITY.md | P3 | ✅ |
| §10 row | full scorecard | scorecard | ✅ |
