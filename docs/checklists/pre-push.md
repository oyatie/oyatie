---
doc_status: published
---

# Checklist: Pre-push

> **When:** Before every `git push`. Mechanically enforced by `.git/hooks/pre-push`. Never skip with `--no-verify`.
> **Owner:** Author of the change.
> **Validator:** `oya verify`

---

1. ☐ **Workspace clean** — no untracked files surprise; `git status` matches expectations.
2. ☐ **Affected-set tested** — `cargo nextest run --workspace --all-features` passes (or scoped subset for fast iteration; full pass before push).
3. ☐ **Format clean** — `cargo fmt --check` passes (no diff).
4. ☐ **Lint clean** — `cargo clippy --workspace --all-features --all-targets -- -D warnings` passes.
5. ☐ **Architecture boundaries** — `cargo run -q -p oya-dev-cli -- gate validate architecture-boundaries` passes (PARTS A-G hard-fail per ADR-0015; replaces the retired `oya gate validate architecture-boundaries` per audit row B-2).
6. ☐ **License gate** — `cargo deny check` passes; no new dependency without ledger entry.
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
20. ☐ **Run** `oya verify` — final mechanical verification.

---

## Anti-patterns

- `git push --no-verify` — **never** unless explicitly authorized + logged
- Skipping clippy — never
- Pushing to `main` directly — never
- Pushing without running tests — never; even on a hotfix, run the affected-set
- Adding a dependency without ledger entry — CI will fail; pre-empt
