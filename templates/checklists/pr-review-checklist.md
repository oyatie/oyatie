---
doc_class: Checklist
checklist_id: CHK-PRREV
status: pending approval
purpose: |
  Reviewer agent's verification list per change class. Walked by the reviewer agent named in `docs/AGENTS.md §Per-change-class reviewer agents` before signing `## Code Review` at merge.
lift_target: oyatie/docs/checklists/pr-review.md
enforcing_fitness_lane: guard-pr-merge-review.mjs
owner_team: axis-foundry + per change-class team
related:
  - docs/AGENTS.md
  - /templates/pull-request-template.md
  - /templates/checklists/done-definition-checklist.md
---

# PR Review Checklist

> The reviewer agent walks this checklist **before** writing `## Code Review`. Verdict is `APPROVE` or `REQUEST CHANGES`. Without a `## Code Review` H2, `guard-pr-merge-review.mjs` refuses the merge.

## Universal review (every PR)

- [ ] **R1** PR body has all 5 canonical H2s: `## Issue / Summary / Verification / Traceability / Evidence`. *Lane:* `traceability-validator`.
- [ ] **R2** `## Issue` names the change class on a single line.
- [ ] **R3** `## Summary` states *why*, not only *what*.
- [ ] **R4** `## Verification` pastes actual command output (not hand-waves). Every required check has a `PASS` token.
- [ ] **R5** `## Traceability` cites canonical docs read, ADRs cited, cross-axis contracts touched, IP ID (if applicable). Legacy ADR-NNNN forbidden in active text.
- [ ] **R6** `## Evidence` lists audit-chain emission ID + (if binary) Cosign signature + SBOM + SLSA level.
- [ ] **R7** Done-definition rows D1-D18 walked (see `/templates/checklists/done-definition-checklist.md`).
- [ ] **R8** No `--no-verify`, no hook bypass, no signing skip in the commits. *Lane:* `oya-governance-bypass`.
- [ ] **R9** No untyped values at API boundaries (per `docs/standards/error-handling.md`). *(advisory; per-language reviewer enforces)*
- [ ] **R10** Linus good-taste audit row present in `## Code Review`. Empty = `REQUEST CHANGES`.

## Per-change-class additions

### `*.rs` (rust-reviewer)
- [ ] Lane-owned Buck2 build/test/lint targets for touched Rust crates PASS.
- [ ] `buck2 build //:buck2-authority-policy-check` PASS when build/test authority, toolchain, or dependency posture changed.
- [ ] `buck2 build //:repo-hygiene-automation-check` PASS when shared templates, repo hygiene, tooling, or governance surfaces changed.
- [ ] Dependency additions are registered in `registry/dependency-rationales.json` and `registry/dependency-blessed-allowlist.json`; latest stable/LTS posture or waiver is cited.
- [ ] `unsafe` blocks (if any) carry `// SAFETY:` comments with invariant docs. *(advisory)*
- [ ] `thiserror` in libraries / `anyhow|eyre` at the edge (per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md §Domain 3 error handling`).

### Non-Rust tooling exception
- [ ] The surface is registered in the repo-hygiene inventory with a deletion owner/date and an explanation for why Rust is not the correct fit.
- [ ] The verifier is invoked through Buck2 and does not introduce package-manager, runtime, or script authority outside the registered exception.
- [ ] No product/frontend or durable governance surface is added in a non-Rust runtime.

### migrations / SQL (database-reviewer)
- [ ] Up + down + dry-run + per-tenant + per-cell rollback present. *Lane:* `oya-governance-schema-migration`.
- [ ] Migration is idempotent or has a guard.

### auth / secret / payment paths (security-reviewer)
- [ ] No secrets in repo, fixtures, logs, or commits. *Lane:* `oya-governance-secret-scan`.
- [ ] `SecretReference` newtype used (never raw strings).
- [ ] Audit emission on every secret read.

### privacy / consent / DSR paths (privacy-reviewer)
- [ ] `data_class` annotation present on every new kernel field. *Lane:* `oya-governance-data-class`.
- [ ] DPIA referenced if PHI/PII direct identifier touched.
- [ ] DSR cascade path validated per `docs/checklists/dsr-cascade.md`.

### feature / bugfix (tdd-guide)
- [ ] Test written **before** the fix; test fails on buggy commit, passes on fix. *(advisory; TDD)*
- [ ] Coverage delta non-negative for touched files.

### error-handling change (silent-failure-hunter)
- [ ] No `unwrap()` / `expect()` on user-input paths.
- [ ] No `catch {}` swallowing.
- [ ] Errors propagate with context (per `docs/standards/error-handling.md`).

### API / contract change (doc-updater)
- [ ] `contracts/<surface>.<format>` updated.
- [ ] `docs/SPEC.md` cite refreshed.
- [ ] Buck2-projected public API / semver compatibility check is clean, or the breaking-change rationale and migration plan are documented.

### doc-only (doc-style-reviewer)
- [ ] `doc-style.md` (length cap, voice, dual-audience) honored. *Lane:* `oya-governance-doc-style`.
- [ ] `DOC-CATALOG.md` trigger event cited.

### capability publish (capability-reviewer)
- [ ] Capability record validates against schema. *Lane:* `capability-schema-validator`.
- [ ] Eval set min-pass-rate met.
- [ ] Cedar + runtime gate present if tier ≥ T2.
- [ ] Cosign keyless OIDC + SBOM + SLSA L2+ present.

### performance change (perf-reviewer)
- [ ] Benchmark + ≥2 stress scenarios attached. *Lane:* `oya-governance-perf-evidence`.
- [ ] Regression budget defined; result within budget.

## Sign-off

```
Reviewer agent: <name>
Verdict: <APPROVE | REQUEST CHANGES>
Resolved items: <list>
Deferred items: <list with owners + follow-up issue refs>
Linus good-taste audit row: <special cases eliminated | "none — no candidates">
```

## Anti-patterns (auto-`REQUEST CHANGES`)

- Bundling unrelated changes — one PR per concern.
- Skipping `## Verification` outputs.
- Citing legacy ADR-NNNN in active text.
- Worker agent attempting to add `## Code Review` itself (lead-only).
- Force-pushing over reviewer-agent-resolved threads.
