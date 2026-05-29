---
doc_class: PolicyDocument
title: Lane Execution Invariants
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-security
deciders: axis-foundry, ops-security, council-architecture
related_adrs: [ADR-0110, ADR-0111, ADR-0139, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/governance/threat-model.md
  - microservices/governance/runbooks/lane-failure-triage.md
review_cadence: quarterly + on every new lane addition
doc_status: published
---

# Lane Execution Invariants

This document declares the invariants that every fitness lane under `microservices/governance/src/crates/oya-check-*` MUST satisfy. The `lane-runtime` BC enforces these at dispatch time; violations refuse the lane to register.

## Invariant 1 — Determinism

**Statement**: Given the same input (workspace tree at SHA X + rule-pack version Y), the lane MUST produce the same verdict + Finding set across re-runs.

**Test**: `microservices/governance/tests/integration/lane-determinism.rs` re-runs every lane 3× against fixture trees; verdict + Finding-hash MUST match across runs.

**Why**: Replay-ability per SOC 2 CC7.4; audit-chain replay correctness; merge-queue projected-state stability per ADR-0111.

## Invariant 2 — Hermetic Execution

**Statement**: Lane execution MUST NOT make outbound network requests except to: (a) `crates.io` (Cargo registry, read-only); (b) the in-cluster Postgres + S3 + audit-chain endpoints; (c) the OpenBao secrets endpoint. NO external HTTPS calls to industry-baseline sources at lane time; baseline pins are read from `/specs/industry-best-practice-conformance.json` only.

**Test**: Per-runner network policy (allow-list at K8s NetworkPolicy + Envoy egress filter); lane crates run with `--offline` Cargo flag where possible; `oya-check-supply-chain` lane self-verifies allow-list adherence.

**Why**: Hermeticity per SLSA Build L3; reproducibility; supply-chain attack surface minimisation (T-T-01).

## Invariant 3 — Bounded Resource Usage

**Statement**: Each lane MUST complete within: 60s wall-clock, 8 GB RSS, 4 CPU-cores. Exceeding any bound → SIGKILL → BLOCKER verdict with `lane-resource-overrun` Finding.

**Test**: per-runner cgroup limits; lane-runtime kills + emits Finding on overrun.

**Why**: Denial-of-service mitigation (T-D-02); SLO budget per PRD §"Performance Targets".

## Invariant 4 — Signed Finding Emission

**Statement**: Every Finding emitted MUST carry an Ed25519 signature over its canonical-JSON form, signed by the lane runner's per-environment signing key (rotated 90d via OpenBao).

**Test**: `oya-check-evidence-integrity` lane self-verifies every Finding's signature against the published key registry.

**Why**: Non-repudiation per SOC 2 CC6.3 + ISO 27001 A.5.16 + SLSA Source L3; audit-chain replayability.

## Invariant 5 — Severity Honesty

**Statement**: A lane MUST register itself with the registry under exactly one severity: `BLOCKER`, `WARN`, or `INFO`. A lane MAY NOT degrade its own severity at runtime. A BLOCKER lane MUST refuse the PR; a WARN lane emits a Finding but does not refuse; an INFO lane emits informational record only.

**Test**: rule-pack schema validation (TOML `severity = ...` required); `oya-check-quality-lane` validates severity-honesty.

**Why**: Prevents silent-softening (T-T-02); honest signal posture per ADR-0133 axis-5 principles.

## Invariant 6 — One-Shot Verdict per (Lane, SHA, Target)

**Statement**: For a given `(lane_id, source_sha, target_microservice)` triple, the lane MUST emit exactly one verdict per execution. Re-runs against the same triple MUST produce the same verdict (per Invariant 1).

**Test**: Postgres `findings` table has UNIQUE INDEX on `(lane_id, source_sha, target_microservice, run_id)`; insert-conflict returns `duplicate-verdict` Finding.

**Why**: Postgres lane-state idempotency; admission-gate decision determinism per ADR-0111 projected merge state.

## Invariant 7 — Citation Required for BLOCKER

**Statement**: Every BLOCKER Finding MUST carry a `citation` field referencing: (a) the rule-pack rule ID; (b) the offending file:line; (c) where applicable, the industry baseline (e.g., "SLSA Build L3 §source-provenance"); (d) the remediation URL or doc anchor.

**Test**: `oya-check-active-artifact-contract` lane validates citation schema on every BLOCKER Finding.

**Why**: Remediation actionability; audit-chain auditor-readability; honest signal per ADR-0133 §"agentic-dev-team optimisation #6 smallest-actionable artifact format".

## Invariant 8 — Idempotent Re-registration

**Statement**: Lane registration is idempotent: registering the same `(lane_id, version)` twice MUST NOT create duplicate registry entries. Version bump (new rule-pack hash) creates a new registry entry; previous entry retires (audit-trailed).

**Test**: `oya-governance-lane-runtime-rest` POST `/lanes` idempotency-key handler.

**Why**: Operational resilience; deploy-replay safety.

## Invariant 9 — No Cross-Lane State

**Statement**: A lane MUST NOT read OR write state from another lane's execution. Lanes communicate only via the canonical Postgres `findings` table after their own execution completes (which the lane runner manages on their behalf, not the lane itself).

**Test**: Per-runner SPIFFE scope limits Postgres role to write-own-findings only.

**Why**: Lane independence; parallel execution safety; failure-isolation.

## Invariant 10 — Self-Application

**Statement**: Every lane MUST be applicable to the governance µservice itself. The lane runs on every governance PR. If the lane's rule-pack requires changes for governance to pass, those changes belong in the same PR.

**Test**: `microservices/governance/tests/e2e/self-application.rs` runs the full ~50-lane set against the governance µservice's own working tree.

**Why**: Bootstrap-paradox mitigation per PRD Open Q3; honesty per ADR-0133.

## Invariant 11 — Replayability

**Statement**: Every lane execution MUST be replayable: given the SHA + rule-pack version + workspace fixture, a fresh runner MUST reproduce the verdict bit-for-bit.

**Test**: `microservices/governance/tests/integration/replay-cross-runner.rs` runs lane on runner-A, captures verdict + Finding-set, runs same lane on runner-B 24h later, asserts equality.

**Why**: External-auditor verification; forensic recovery; SOC 2 CC7.4.

## Invariant 12 — No Pre-Commit Side Effects

**Statement**: A lane MUST NOT mutate the workspace tree, the Postgres outside its scoped writes, the S3 bucket outside its scoped writes, or any external system. Mutations refused at runner sandbox level.

**Test**: Read-only workspace mount in ephemeral runner; SPIFFE scope; outbound allow-list per Invariant 2.

**Why**: ADR-0133 axis-5 principle #3 "Idempotent operations"; T-T-02 mitigation; principle of least authority.

## Lane Registration Workflow

1. **Author**: developer creates `microservices/governance/src/crates/oya-check-<topic>/` with rule pack at `rules/<topic>.toml`.
2. **Self-test**: `cargo test -p oya-check-<topic>` exits 0.
3. **PR**: opens PR; the full ~50-lane fitness suite runs against the new crate + the rest of the workspace.
4. **`oya-check-quality-lane` validation**: asserts new lane satisfies all 12 invariants above.
5. **Reviewer**: axis-foundry + ops-security CODEOWNERS approve.
6. **Merge**: lane registers automatically on next `dev` merge; appears in registry within 60s.

## Lane Retirement Workflow

1. **ADR**: file `microservices/governance/decisions/ADR-####-retire-<lane>.md` documenting rationale + replacement (if any).
2. **PR**: remove the crate + the workspace member; emits `lane-retirement-announce` Finding for one cycle.
3. **Audit**: retirement is sealed in audit-chain; previous Findings retained per retention policy.

## References

- ADR-0110 (ChangeSet state machine).
- ADR-0111 (merge-queue projected-state).
- ADR-0139 (agentic SLO-gated promotion).
- ADR-0131 §"oya-governance-per-microservice-layout" lane.
- ADR-0133 §"agentic-dev-team optimisation".
- `microservices/governance/threat-model.md`.
- `microservices/governance/runbooks/lane-failure-triage.md`.
- SLSA Build + Source L3 — `slsa.dev/spec/v1.0/levels`.
