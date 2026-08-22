---
doc_class: Standard
title: Agentic Dev-Team Optimisation (cross-cutting)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-architecture + axis-foundry
deciders: council-architecture, axis-foundry, ops-sre-reliability
related_adrs: [ADR-0056, ADR-0105, ADR-0110, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/industry-best-practice-conformance.json, /specs/microservice-migration-tooling.json, /specs/agent-durable-goal.json]
applies_to: every microservice + every artifact in oyatie repo
enforced_by: governance-industry-best-practice-conformance §"axis-5-practices"
review_cadence: annually + on every major-tooling change affecting agent runtime
doc_status: published
---

# Standard: Agentic Dev-Team Optimisation

## Purpose

Cross-cutting principles that distinguish oyatie's repo from a human-developer-team repo. Industry best practices assume a human typing into an editor; oyatie's developer is a fully-agentic team executing in parallel. This standard codifies the optimisation vectors that make every artifact agent-friendly: structured, parallel-safe, idempotent, audit-chain-sealed, fail-closed, smallest-actionable.

Every artifact authored in oyatie MUST satisfy these principles. The `governance-industry-best-practice-conformance` CI lane (per ADR-0133) enforces. The retired `oya vcs` claim/verify/done/promote ratchet is historical only; live coordination uses plain git branches, PRs, Jenkins, and `oya gate` / `oya verify`.

## Principle 1 — Semantic branch scope

**Rule:** Agents scope each isolated worktree branch to semantic AST/artifact units (a port trait, a use case, a manifest, a runbook section), not line hunks or whole files.

**Why:** Per `feedback_durable_goal_spec` ("Agents claim semantic AST/artifact units instead of line hunks or whole files, so unrelated edits in the same file or project can proceed in parallel"). Two agents touching different parts of the same file cannot collide.

**How to apply:**
- Declare the PR/worktree scope as semantic paths or symbols; do not treat a whole-file glob as ownership when unrelated edits can proceed independently.
- Migrations use `oya dev migrate-microservice` (per `/specs/microservice-migration-tooling.json`) which claims at µservice-scope, not at repo-scope.
- ast-grep / rust-analyzer / clippy refactors operate on AST nodes; never blanket-sed (see Principle 7).

**Verification:** PR description and governance evidence identify the semantic scope; reviewer/governance gates reject oversized or ambiguous scope claims.

## Principle 2 — Parallel-safe operations

**Rule:** Operations are independent unless explicitly chained. Per ADR-0110 ChangeSet contract, each IP is claimable, verifiable, bundleable, promotable independently.

**Why:** Agentic teams scale horizontally. A serial-only pipeline limits throughput to one agent.

**How to apply:**
- IP dependency graphs use explicit DAG ordering with parallel tiers (per `ADR-0131 §"Migration DAG"`).
- Migrations touch disjoint paths (per `migrate-microservice` parallel_safety contract).
- CI lanes parallelise per PR branch; cross-µservice ordering only on explicit dependency edges.

**Verification:** `oya gate run-all` / reviewer governance evidence asserts the PR branch affected paths do not overlap with known in-flight ownership.

## Principle 3 — Idempotent operations

**Rule:** Every CLI subcommand re-runnable without ill effect. Re-running a migration on an already-migrated µservice = no-op + audit-chain note "migration_already_complete". Re-evaluating an SLO verdict for the same (microservice, source_sha, target_env) tuple at the same evaluator_version = same verdict; ledger row written only on transition.

**Why:** Agents retry on transient failures; retries must not cascade unintended mutations. Per Stripe / Twilio / Google idempotency-key conventions.

**How to apply:**
- Every external webhook, CI dispatch, and evidence emission carries a stable idempotency key derived from the PR/change id + state.
- Mimir verdict emissions key on `(microservice, source_sha, target_env, verdict, evaluator_version)`; duplicate writes deduplicated by the metric label hash.
- File migrations honour `--dry-run` default; `--no-dry-run` is opt-in per `microservice-migration-tooling.json`.

**Verification:** integration test per CLI subcommand: invoke twice, assert second invocation is no-op.

## Principle 4 — Audit-chain seals on every state transition

**Rule:** Every state transition emits an audit-chain record (Ed25519 + Merkle per Bominal ADR-0028). No state mutation is observable without its corresponding audit row.

**Why:** Agents read state from durable evidence, not from in-flight memory. Audit-chain is the agentic single-source-of-truth.

**How to apply:**
- Every promotion emits `PromotionExecuted` event (per `microservices/observability/contracts/asyncapi/eligibility-events.yaml`).
- Every rollback emits `RollbackExecuted`.
- Every Cedar policy decision emits per `policy/auditor-scope.cedar` audit trail.
- Per-changeset multispectrum evidence at `microservices/<ms>/evidence/multispectrum/<change_id>-*.json` (per `docs/AGENTS.md §changeset`) seals what shipped.

**Verification:** `oya gate validate audit-chain-coverage --microservice <ms>` asserts every state-transition emitter is wired.

## Principle 5 — Fail-closed on every gate

**Rule:** Default-deny in every authorisation surface (Cedar) + default-`held` in every promotion gate (ADR-0139) + default-`reject` when an OpenSLO manifest is missing. Absence of an explicit permit is a refusal.

**Why:** Agents recover from rejection faster than they recover from incorrect approval. False-positive deny is operational delay; false-positive permit is a breach.

**How to apply:**
- Cedar fragments author `forbid` first, `permit` only with explicit guard clauses (per `tenant-scope.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, `public-read.cedar`).
- SLO engine worker fail-closed during cold-start (≥3 evaluator cycles clean before emitting `eligible`) per `IP-008-slo-engine-worker.md`.
- CI lanes exit non-zero on any unhandled error condition; never silently pass.

**Verification:** Cedar test set: every fragment has at least one assertion that an unauthenticated principal is denied.

## Principle 6 — Smallest-actionable artifact format

**Rule:** Every agent-readable Markdown, JSON, YAML, hook output, settings file is smallest-actionable: clear intent, well placed, no repeated memory dumps, no raw logs, no stale status unless it changes the next action.

**Why:** Per durable user preference (multiple session reminders). Agents waste tokens parsing fluff. Tight artifacts compose better in batched evaluations.

**How to apply:**
- Each ADR / PRD / phase-spec / IP has a single canonical purpose; no narrative repetition across documents — cross-reference instead.
- Status updates surface what changed; not what's unchanged.
- Frontmatter carries machine-readable structure; body carries why-not-what.
- No commented-out code; no `TODO` markers that should be filed as fixuptasks.

**Verification:** `oya gate validate smallest-actionable --path <artifact>` (TBD; queued per ADR-0133 axis-5 enforcement) measures token-count vs information-density and flags below-threshold artifacts.

## Principle 7 — No blanket-sed

**Rule:** Repo-wide multi-file substitutions use AST-aware tooling (ast-grep, rust-analyzer, clippy, custom Cargo workspace operations) rather than blanket `sed -i`.

**Why:** Per durable user directive ("The implementation rebrand `oyatie-*` → `oya-*` MUST proceed as a coordinated multi-batch migration; blanket-sed is forbidden"). Blanket-sed silently mangles strings inside comments, doc-comments, test fixtures, and unrelated semantic contexts.

**How to apply:**
- `oya dev migrate-microservice` uses ast-grep for Markdown-link patterns + Cargo workspace operations for `[workspace.members]` updates.
- Per-µservice rename operations are batched per ADR-0110 ChangeSet contract.
- Targeted `sed -i` is allowed ONLY when the pattern is uniquely-identifying (e.g., an `id:` line in YAML frontmatter); use of `sed` requires explicit justification in the PR description.

**Verification:** `governance-changeset-state` (existing lane) plus a `commit-msg` hook that flags any commit message claiming "sed-based" rename.

## Principle 8 — No-deeper-hole rule

**Rule:** New code MUST NOT use an external framework directly when an Oya façade exists or can be added cheaply.

**Why:** Per durable user preference. Reducing framework leakage keeps the repo replaceable; agents author against stable Oya types, not against shifting third-party APIs.

**How to apply:**
- New crates depend on `oya-*` façades (e.g., `observability-slo-engine-kernel` for SLO types, not `prometheus-client` directly).
- When no façade exists and adding one is < 100 lines of code, add it; otherwise file a fixuptask for the missing façade.
- Adapter layers isolate external framework imports per ADR-0105 layer enum.

**Verification:** `oya gate validate no-deeper-hole --crate <crate>` (TBD; queued per ADR-0133 axis-5 enforcement) compares per-crate external-framework imports against the Oya-façade catalog.

## Enforcement

The `governance-industry-best-practice-conformance` CI lane (BLOCKER on dev; per ADR-0133) enforces these 8 principles on every artifact. Existing legacy violations are recorded as `severity: legacy-grandfathered` with a remediation IP filed.

## References

- ADR-0133 (this standard's parent ADR)
- ADR-0110 (ChangeSet state machine — Principle 2)
- ADR-0028 (Bominal audit-chain — Principle 4)
- ADR-0139 (SLO gate fail-closed — Principle 5)
- ADR-0140 (retired per ADR-0145) (Cedar default-deny — Principle 5)
- `feedback_durable_goal_spec` (Principle 1)
- `feedback_no_silent_regression` (Principle 5)
- `/specs/microservice-migration-tooling.json` §"parallel_safety" + "idempotency" (Principles 2, 3)
- `/specs/agent-durable-goal.json`
- AWS / Twilio / Stripe idempotency-key conventions
- Cedar Policy Language — `cedarpolicy.com`
