# docs/_historical — pre-v1 planning archive

Everything in this directory is **historical**. It describes what the app was planned to be before v0.6, not what it is now. Do not follow any of it as current guidance.

Kept on disk because some of it is still useful as context (the v1 PRD still roughly describes the product; the architecture doc still roughly describes the code), but there is no process for keeping any of these files in sync with reality. If you find a conflict between this directory and the root-level docs, the root-level docs win.

Authoritative docs live at:

- [../../../CLAUDE.md](../../../CLAUDE.md) — repo-wide working rules
- [../../../MONOREPO.md](../../../MONOREPO.md) — architecture, contracts, history
- [../../../TODO.md](../../../TODO.md) — current state
- [../../../RELEASE_RUNBOOK.md](../../../RELEASE_RUNBOOK.md) and [../../RELEASE.md](../../RELEASE.md) — release process

## Contents

| File | What it was for | Why archived |
|---|---|---|
| `ARCHITECTURE.md` | v1 technical architecture spec | Partially drifted from shipping code; `MONOREPO.md §2` is the current picture. |
| `BACKLOG.md` | Rolling backlog with stable IDs | Never actually used for tracking. Current work tracked in `TODO.md`. |
| `COMPONENT_INVENTORY.md` | UI component inventory for design-system work | Component list is stale; the shipping set drifted during v0.3–v0.6. |
| `HANDOVER_2026-04-23.md` | Session log from pre-monorepo-consolidation day | References "when the two repos get unified" — that merge has happened. |
| `IMPLEMENTATION_PLAN.md` | Five-phase implementation plan for v1 | All phases shipped (or deliberately deferred). |
| `PRD.md` | v1 product requirements | Still roughly describes the product, but the release process and release notes (`releases/*.md`) are the authoritative changelog now. |
| `QA_MATRIX.md` | Manual QA matrix before release | Actual smoke-test checklist is in `../../../RELEASE_RUNBOOK.md §4`. |
| `RELEASE_NOTES.md` | Pre-v0.6 aggregated release notes | Per-version release notes now live in `../../releases/`. |
| `SCHEMA.md` | SQLite migration spec | Authoritative schema is in `src-tauri/src/db/migrations/` (refinery-tracked). The `orliq.db` path here is stale; the real DB is `utterbar.db`. |
| `SCREEN_MAP.md` | Numbered screen/surface map | Route layout has drifted; the router in `src/routes/MainShell.tsx` is authoritative. |
