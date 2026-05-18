# Canonical Primitives Cheat Sheet — 2026-05-18

Single source of truth for hook payloads. Hooks `cat` or `grep` this file rather
than duplicating strings. Keep sections machine-parseable (no nested bullets).

---

## VCS

Canonical invocation:
  oya vcs <subcommand>
  cargo run --quiet -p oya-dev-cli -- vcs <subcommand>

Retired (do NOT use): grit, rtk, icm, vox
Authority: ADR-0116, feedback_oya_vcs_canonical_2026_05_16

---

## Contracts

OpenAPI version : 3.2.0  (NOT 3.3, NOT 3.0.0, NOT 3.1.0)
AsyncAPI version: 3.1.0  (NOT 3.0.0, NOT 2.x)
Schema language : proto3  (NOT proto2)
Reference (OpenAPI): https://spec.openapis.org/oas/v3.2.0
Reference (AsyncAPI): https://www.asyncapi.com/docs/reference/specification/v3.1.0

---

## AI Substrate

microservices/intelligence/  — consumer-facing AI product surface
microservices/foundry/       — internal Hermes dev pipeline (NOT consumer-facing)
Authority: ADR-0136-amendment, ADR-0220

---

## Taxonomy

plugin-app-store   — curated plugin distribution channel (distinct µservice)
marketplace        — general commerce surface (distinct µservice)
community          — social/forum surface (distinct µservice)
These three are NOT synonyms. Each is a separate µservice with its own contracts.

---

## Quality Bar

Artifact threshold: 100+ artifacts per µservice (files across docs/, src/, slos/, contracts/)
Authority: ADR-0212 (Buildability Doctrine)

---

## Doctrines In-Flight (ADR-0211..ADR-0221)

ADR-0211: In-house tech stack preference (Rust-primary)
ADR-0212: Buildability doctrine — every µservice buildable end-to-end, 100+ artifacts
ADR-0215: Multi-context platform — same engine, multiple deployment contexts
ADR-0216: Open integration — standard APIs; no vendor lock-in
ADR-0217: Vertical-slice rollout — ship one slice at a time, not horizontal sprawl
ADR-0218: Tenant granular control — per-tenant feature flags + policy
ADR-0219: No-code-first UX with optional AI-assist layer
ADR-0220: Intelligence µservice scope — consumer-facing only
ADR-0221: Agentic pipeline hardening — hooks are GUIDANCE, not enforcement; CI gates enforce
ADR-0136-amendment: Foundry internal scope — Hermes pipeline only, not consumer

---

## Retired Tooling

grit — retired per ADR-0116; use: oya vcs
rtk  — retired per ADR-0116; use: oya vcs
icm  — retired per ADR-0116; use: oya vcs
vox  — retired per ADR-0116; use: oya vcs

---

## Forbidden Primitives

See: specs/master-plan-sequencing.json#forbidden_primitives
Summary: grit, rtk, icm, vox in Bash commands; OpenAPI != 3.2.0; AsyncAPI != 3.1.0

---

## Common Pitfalls

1. Using `grit done` instead of `oya vcs` commands
2. Writing `openapi: 3.3.0` (no such released version as of 2026-05-18)
3. Writing `asyncapi: 3.0.0` (use 3.1.0)
4. Treating microservices/foundry/ as consumer-facing (it is Hermes-internal only)
5. Conflating plugin-app-store / marketplace / community
6. Creating µservices with <100 artifacts (buildability bar)
7. Bundling multiple concerns into one µservice (ADR-0132 no-suite policy)
8. ADR references in docs without corresponding docs/decisions/ADR-NNNN-*.md files
9. Vacuous-green gates: test passes on empty input (M-08 per ADR-0221)
10. Scope creep: creating new µservices outside the current PR's declared vertical slice

---

## oya-dev-cli Invocation Pattern

Direct:     cargo run --quiet -p oya-dev-cli -- <subcommand> [args]
Via wrapper: ./bin/oya <subcommand> [args]  (after PATH_add bin via .envrc)
Top-level subcommands: vcs, gate, governance-gates, foundation-audit-gates,
                        catalog, check, demo, doc, lint, onprem, ops, submit,
                        supply-chain, verify

---

## Lifecycle Skill Map

Vendored at tools/agent-skills/skills/
Source: https://github.com/addyosmani/agent-skills (MIT — Addy Osmani and contributors)

Define phase:
  interview-me                  — extract real requirements before writing code
  idea-refine                   — stress-test ideas before committing to a plan
  spec-driven-development       — write spec before writing code

Plan phase:
  planning-and-task-breakdown   — break work into ordered atomic tasks

Build phase:
  incremental-implementation    — build one step at a time with verification
  test-driven-development       — failing tests first, then implementation
  source-driven-development     — implementation grounded in source evidence
  doubt-driven-development      — challenge assumptions before proceeding
  context-engineering           — optimize agent context for quality output
  api-and-interface-design      — design contracts before implementation
  frontend-ui-engineering       — UI-specific build patterns

Verify phase:
  browser-testing-with-devtools — browser-based test execution
  debugging-and-error-recovery  — systematic root-cause diagnosis

Review phase:
  code-review-and-quality       — multi-axis review (correctness/readability/security/perf)
  code-simplification           — reduce complexity without changing behavior
  security-and-hardening        — security review with remediation
  performance-optimization      — measure first, then optimize

Ship phase:
  git-workflow-and-versioning   — branching, commits, tagging
  ci-cd-and-automation          — pipeline setup and quality gates
  deprecation-and-migration     — safe removal of old APIs/systems
  documentation-and-adrs        — ADR authoring and doc coverage
  shipping-and-launch           — final checklist before merge/release

Persona agents (tools/agent-skills/agents/):
  code-reviewer    — use for review tasks
  security-auditor — use for security tasks
  test-engineer    — use for testing tasks

Discovery rule: invoke the skill matching the task phase BEFORE producing output.
Process skills (Define/Plan) come before implementation skills (Build/Verify/Ship).
