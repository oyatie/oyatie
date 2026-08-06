---
id: ADR-0104
title: Ecosystem-expansion principle for check-lane + adapter crate reintroduction
status: Superseded
superseded_by: [ADR-700]
doc_status: published
owner: council-architecture
date: 2026-05-15
relates_to:
  - ADR-0056-rust-clean-architecture-bnf.md
  - ADR-0059-workflow-ontology-ecosystem-adapter-layer.md
  - ADR-0062-quality-performance-scalability-bar.md
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0104: Ecosystem-expansion principle for check-lane + adapter crate reintroduction

## Status
Accepted

## Context

Commit `34c62f2` (2026-05-15) deleted 18 placeholder-shell crates that the
audit flagged as "6-line doc-stubs claiming Accepted" — they failed the
"no stubs / no false premises" doctrine of `decision-principles.json`
DP-09 + the autonomous-implementation charter
([[feedback_autonomous_implementation_artifacts]]).

After deletion, the question was: do we reintroduce them? The user's
directive that defines the answer:

> "Reintroduce them with real implementation not just stubs and finish
> the scheduled-for-distinct-tracked-work items as well. ... Our toolchain expands with expansion
> of our ecosystem."

The principle: a crate exists when the part of the ecosystem it serves
exists. Premature crate-shelling is the failure mode; scheduled-for-distinct-tracked-work-on-ship
is the success mode. This ADR formalizes the rule and records which of
the 18 deleted crates were reintroduced now, which are scheduled-for-distinct-tracked-work to
ecosystem-trigger, and what the trigger is for each deferral.

## Decision

**Ecosystem-expansion rule.** A crate is shipped iff:

1. The kernel/domain layer it implements is itself shipped, AND
2. At least one consumer in the workspace imports it, AND
3. The crate has a real implementation (not a doc-stub).

If any condition fails, the crate is not shipped. Documentation of the
trigger that would unblock the crate lives in this ADR, in
`specs/masterplan.json`, and in the
`specs/crate-naming-audit.json` audit row.

**Toolchain (check-lane) family** — reintroduce when the check's
target surface exists.

| Crate | Status (2026-05-15) | Trigger / Rationale |
|---|---|---|
| `oya-check-statelessness` | **REINTRODUCED** with real implementation (commit after `34c62f2`). | Workspace has 79 outer-ring source files; lane runs against them today and exits clean. |
| `oya-check-shardability` | **REINTRODUCED** with real implementation. | Currently scans `migrations/` (empty until M02-P04 audit-chain ships PostgreSQL+Citus schema). Lane refuses to falsely claim pass on empty input (requires `--allow-empty`). Becomes mechanically enforcing once the substrate ships migrations. |
| `oya-check-perf-budget` | **REINTRODUCED** with real implementation. | Scans IP markdowns under `.omc/plans/milestones/`. Already surfaces 2 real violations (IP-001-saas-pairs, IP-002-cloud-pairs) — driven to zero in successor-IP. |
| `oya-check-benchmark` | **REINTRODUCED** with real implementation. | Scans PRDs. Currently surfaces 7 real violations across docs/prds/ + docs/products/. Follow-up: per-PRD `## Competitive benchmark` authoring sweep. |

**Cloud-adapter family** — scheduled-for-distinct-tracked-work to consumer-ship.

Twelve `oya-cloud-{billing,marketplace,capacity,data,finops,observability}-adapter-{aws,fake,oci}` crates were deleted. They are NOT being reintroduced today because:

1. **No consumer exists.** Importer audit before deletion confirmed only `[workspace.members]` referenced each — no `*-runtime`, no app crate, nothing called the adapter trait. Per ADR-0059 (workflow/ontology adapter layer), cross-product imports are forbidden anyway, so cloud adapters become reachable only via the planned Workflow µservice (M02-P12) or per-µservice runtime crate.
2. **AWS / OCI variants need credentials + SDK integration.** A "real" AWS billing adapter requires aws-sdk-rust crate wiring + IAM role for the runtime + integration tests against live AWS endpoints. Writing the Rust types without that integration is the original doc-stub failure pattern — name-only, behavior-zero. The honest path is to author them in the PR that brings up the consuming µservice and the cloud credentials together.
3. **"Fake" variants need an interface to fake against.** The corresponding `oya-cloud-{billing,marketplace,capacity,data,finops,observability}-kernel` crates DO ship (see workspace.members); they declare the port traits. But there is currently no test consumer that asserts substitutability between a fake and a future real adapter, so authoring a fake adapter today is premature — its API surface would be guessed against no client.

**Trigger for reintroduction.** Per cloud-domain, the trigger is "the consumer µservice ships." Specifically:

- Cloud-billing adapters land when M02-P18 `cloud-billing` µservice introduces its runtime.
- Cloud-marketplace, cloud-capacity, cloud-data, cloud-finops, cloud-observability adapters land with their respective M02-P18 µservices.
- Each reintroduction follows the patterns documented in the
  `oya-intelligence-account-adapter-inmemory` rename (commit `c7fda53`): real
  port impl, real tests, honest doc-comment about NOT-FOR-PRODUCTION
  when it's a test-double, real SDK integration when it's an AWS/OCI
  variant.

**`oya-intelligence-account-app` and `oya-intelligence-account-runtime`** — deleted, NOT being reintroduced.

The account family's composition root is `oya-intelligence-supervisor-app`,
which already wires together `oya-intelligence-claude-account-adapter` +
`oya-intelligence-codex-account-adapter` + `oya-intelligence-gemini-account-adapter`
+ `oya-intelligence-account-adapter-inmemory` against
`oya-intelligence-supervisor-kernel`'s port traits. There is no remaining
responsibility for a separate `oya-intelligence-account-app` (the supervisor
IS the account-app). The `-runtime` variant is doubly redundant per
ADR-0056 §"Concrete migration" lines 283-289, which slate `*-runtime` for
rename to `*-app` — and `oya-intelligence-account-app` already exists in the
deleted form. The pair was never coherent.

**Trigger for re-creation.** Only if a genuinely separate account composition root needs to exist alongside the supervisor (e.g., a customer-facing standalone account-management binary). At that point, name it for what it is (e.g., `oya-intelligence-account-management-app`) rather than recycling the empty crate names.

## Consequences

- **The 18-crate ledger is no longer a hidden defect.** Future audits run against the audit-named patterns and find no doc-stub residue.
- **`specs/crate-naming-audit.json` must be amended** to reflect the new state: 4 check-lane reintroductions are now compliant; 14 scheduled-for-distinct-tracked-work crates are tracked but not workspace.members.
- **`specs/masterplan.json` adds a new section** (or extends the M02-P18 phase descriptors) documenting that each cloud sub-µservice's adapter family ships in the same PR as its runtime.
- **PR template gains a new check**: any new `*-adapter-*` crate must declare a consumer in the same PR. Mechanical-prevention candidate landed separately as `oya-governance-adapter-with-no-importer-kernel`; do not extend the retired archive-orphan lane (ADR-0118).
- **The "toolchain expands with the ecosystem" principle is now ADR-anchored** — agents and humans cite this ADR to refuse premature crate creation.

## Drivers

- **DP-03** (`decision-principles.json` Mechanical prevention over process) — the 18-crate stub family was a process-failure-to-mechanically-prevent.
- **DP-09** (Bench-and-stress before claiming performance) — the original 4 scalability lanes claimed enforcement without doing it.
- **FO-01** (`forbidden-operations.json` No parallel canonical trees) — adapter-with-no-importer is a parallel-tree pattern.
- **autonomous-implementation charter** ([[feedback_autonomous_implementation_artifacts]]) — no stubs, no placeholders, no deferrals within scope.

## Alternatives Considered

1. **Reintroduce all 18 as test-doubles.** Rejected: 12 of them have no consumer, so the test-double has no client to substitute against. Authoring the port shape with no client is guessing — exactly the failure pattern.
2. **Reintroduce the 4 check lanes as stubs + ship implementations later.** Rejected: that is exactly what was just deleted. The honest path is to ship the real implementation now (which we did) or not ship the crate name.
3. **Keep all 18 deleted and document only verbally.** Rejected: the 4 check lanes have a real consumer (`oya-dev-cli gate validate`) and a real target surface today. Not reintroducing them would itself be a false premise.

## Follow-ups

1. Drive `oya gate validate perf-budget` to zero violations by authoring real `## Load test` sections. Two outstanding items (`IP-001-saas-pairs.md`, `IP-002-cloud-pairs.md`) must close before the lane greens. No exemption markers; canonical predictability requires every IP that ships an adapter to declare its load-test surface.
2. Drive `oya gate validate benchmark` to zero violations (currently 7 across `docs/prds/` + `docs/products/`).
3. When M02-P18 cloud sub-µservice runtimes ship, reintroduce the relevant cloud-adapter families with real port impls + consumers in the same PR.
4. Author `oya-governance-adapter-with-no-importer` lane as a mechanical-prevention for premature adapter creation (per Consequences). **Scaffolded** in `crates/oya-governance-adapter-with-no-importer-kernel` + `tools/oya-governance-adapter-with-no-importer` (ratchet plan in `.omc/plans/milestones/M01-foundation/phases/P03-purpose-orphan-detection/fitness-adapter-with-no-importer-lane.md`); WARN-baseline = 29 violations on first run, BLOCK ratchet across Waves B/C.
5. Amend `specs/crate-naming-audit.json` to mark the 4 reintroduced check lanes as compliant + add deferral rows for the 14 unrestored crates.

## References

- audit findings (2026-05-15) — #7 (14 placeholder-shell crates) + #11 (4 stub check lanes)
- commit `34c62f2` — the strike
- commit `c7fda53` — the OpenBaoAdapter rename pattern (test-double with honest doc-comment)
- specs/decision-principles.json DP-03, DP-09
- specs/forbidden-operations.json FO-01
- specs/crate-naming-audit.json (will be amended)
