---
doc_status: published
---

# Checklist: Pre-push

> **When:** Before every `git push`. Mechanically enforced by the registered Rust/Buck2 pre-push surface when installed. Never skip with `--no-verify`.
> **Owner:** Author of the change.
> **Validator:** Buck2 affected build/test/check targets plus repository policy targets; protected-branch authority is `oya-ci-required`.

---

1. ☐ **Workspace clean** — no untracked files surprise; `git status` matches expectations.
2. ☐ **Affected-set tested** — lane-owned Buck2 test target passes (or scoped Buck2 subset for fast iteration; required target set before push).
3. ☐ **Format clean** — Buck2/Rust formatting target passes; direct `rustfmt --edition 2024 --check` is acceptable for the touched Rust files.
4. ☐ **Lint clean** — Buck2 lint/static-analysis target passes for touched crates and downstream consumers.
5. ☐ **Architecture boundaries** — Buck2 architecture-boundary target passes for the affected graph.
6. ☐ **License gate** — Buck2 dependency-policy target passes; no new dependency without registry and stable/LTS evidence.
7. ☐ **Schema-class annotations** — every new struct field in a kernel crate has a `data_class` per [PRIVACY-PROGRAM §2.2.1](../PRIVACY-PROGRAM.md).
8. ☐ **YAML date integrity** — every YAML date is quoted (per mistakes-and-fixes-ledger).
9. ☐ **Forward-reference discipline** — no markdown link to a path not yet on `origin/main` (per Issue #1433).
10. ☐ **Catalog record up-to-date** — if a crate added/role-changed, `registry/catalog/<crate>.yaml` exists + matches `[package] name`.
11. ☐ **Capability record up-to-date** — if a Foundry capability added/changed, `registry/capability-templates/<id>.yaml` updated; eval-set passes.
12. ☐ **Audit-chain emission** — if the change touches a regulated capability, emission is wired (per ADR-0003).
13. ☐ **PR body shape ready** — 5 H2 sections per CLAUDE.md (`## Issue / Summary / Verification / Traceability / Evidence`).
14. ☐ **Branch protection** — pushing to a feature branch (never directly to `main` or `release/*`).
15. ☐ **Sensitive files** — no `.env`, no API key, no PHI/PII/PCI fixture; `git diff --name-only` reviewed.
16. ☐ **Cohesion-fitness preview** — if change touches a DESIGN §10 cross-axis contract row, the cross-axis label is set + reviewers from each affected axis pinged.
17. ☐ **Migration ledger** — if this is a flat-crates move PR (per ADR-0015), `registry/migrations/2026-flat-crate-migration/` entry added.
18. ☐ **Rebrand check** — no new `Oyatie` brand string in product code (per ADR-0017); repo path / GitHub slug exception OK.
19. ☐ **License-tier residue** — no new AGPL/GPL/SSPL/BUSL dependency in product code (per drafted License Policy ADR).
20. ☐ **Run the lane-owned Buck2 verification set** — final mechanical verification before push.

---

## Anti-patterns

- `git push --no-verify` — **never** unless explicitly authorized + logged
- Skipping clippy — never
- Pushing to `main` directly — never
- Pushing without running tests — never; even on a hotfix, run the affected-set
- Adding a dependency without ledger entry — CI will fail; pre-empt
