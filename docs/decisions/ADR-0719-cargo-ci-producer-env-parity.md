---
doc_status: published
id: ADR-0719
title: "Cargo CI producer env parity: materialized-face and ADR-index producer env vars"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-14
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
amended_by: []
depends_on: [ADR-0716]
related: [ADR-0515]
milestone: W0
deliverables:
  - id: ADR-0719-D1
    description: "The new cargo workflow's test job must set the two env keys two fail-closed cross-artifact tests require: OYA_HISTORY_ONLY_RETIREMENT_FACTS (the materialized history-only retirement facts face path) and OYA_ADR_INDEX_PRODUCER_BIN (the sanctioned ADR-index producer binary). Both are job-level env, not run-step env."
    exit_criteria: "The test job env block declares both keys; the fail-closed cross-artifact tests resolve the materialized face and exec the producer binary without local overrides."
    verified_by: "cargo test --workspace with the CI env on the PR head"
  - id: ADR-0719-D2
    description: "The 'Build enforcement-liveness producer' step also builds the ADR-index producer binary (`cargo build --locked -p marketplace-dev-cli --bin oya`) so OYA_ADR_INDEX_PRODUCER_BIN resolves in the same step that already builds the accounting-registry producer."
    exit_criteria: "The step run builds both binaries; the automation-language-policy inline-shell baseline accepts the step's run-content change through a reviewed replacement window (schema_version +1, substantive reason, this ADR)."
    verified_by: "ci-automation-language-policy live-corpus tests on the PR head"
---

# ADR-0719: Cargo CI producer env parity: materialized-face and ADR-index producer env vars

## Status

Accepted (2026-08-14).

## Context

ADR-0716 made the Cargo workspace graph the CI merge path. Two fail-closed
cross-artifact tests in the gate fleet consume artifacts the workflow must
supply through the environment: the history-only retirement facts face
(materialized before tests) and the sanctioned ADR-index producer binary.
The new cargo workflow never set those env keys, so the tests failed closed in
CI while passing locally only when a developer exported the same values.
The automation-language-policy inline-shell ratchet is a one-way shrink-only
ceiling; a deliberate run-content change to an existing workflow step requires
a reviewed replacement window (schema_version bump + reason + a new Accepted
ADR absent from the protected merge-base).

## Decision

- Add `OYA_HISTORY_ONLY_RETIREMENT_FACTS` (the
  `ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json`
  face path) and `OYA_ADR_INDEX_PRODUCER_BIN`
  (`${{ github.workspace }}/target/debug/oya`) to the test job's env block in
  `oya-ci-required.yml`.
- Extend the "Build enforcement-liveness producer" step to also build the
  ADR-index producer binary, so the env var resolves to a binary built in the
  same step.
- Record the workflow run-content change in the automation-language-policy
  replacement window (this ADR) so the inline-shell ratchet remains
  shrink-only otherwise.

## Consequences

The two fail-closed cross-artifact tests run green under the exact CI env; the
automation-language-policy ceiling stays enforced for every other workflow
step. No digest, edge, or ordering data is touched.

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `.github/workflows/oya-ci-required.yml` | update | — | — |
| `docs/decisions/ADR-0719-cargo-ci-producer-env-parity.md` | create | — | — |

No Rust crate or workspace member changes in this ADR: the two env keys are
consumed by the already-live fail-closed cross-artifact tests
(`ci/facade/cross-artifact-agreement` and the materialized-face consumers) and
the ADR-index producer binary is built from the existing
`marketplace/facade/dev-cli` crate. The only workflow content change is the
"Build enforcement-liveness producer" step additionally building the
`oya` binary (`cargo build --locked -p marketplace-dev-cli --bin oya`).

### Integration via Workflow + Ontology

Not applicable — this ADR changes CI producer environment wiring only; it does
not emit or consume Workflow events, and it writes no Ontology Object or Link
Types.

### Positive
- The two fail-closed cross-artifact tests run green under the exact CI env
  without developer-local exports.
- The automation-language-policy inline-shell ratchet stays shrink-only: the
  run-content change lands through a reviewed replacement window, not a
  silent workflow edit.

### Negative
- CI depends on a debug-profile binary path (`target/debug/oya`); a profile
  or layout change would need a coordinated env-key update.
- The producer step builds one additional binary on every test run.

### Operational
- No new CI lane; the `test (workspace + gates)` lane carries the two env keys
  and the extended producer step.
- ADR-0346 verification posture: the retired `./bin/oya verify --ci-required`
  path is historical/provenance-only; required verification is the
  `oya-ci-required` context and the Rust gate fleet.

---

## Clean Architecture Impact

No crate, layer, or port boundary changes: this ADR only supplies two
environment keys to the existing cargo test job and extends one workflow step's
build. All six LEAN lanes are therefore "Not affected".

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none |
| `cross-product-refusal` (LEAN-A2) | Not affected | none |
| `port-location` | Not affected | none |
| `layer-correctness` | Not affected | none |
| `composition-root-only` | Not affected | none |
| `sdk-kernel-only` | Not affected | none |

No port traits are introduced by this decision.

---

## Alternatives Considered

**Alternative 1 — Set the env keys as run-step env on each consuming step**
- Description: declare the two keys inline on each step that needs them
  instead of at job level.
- Pros: scopes the keys to exactly the consuming steps.
- Cons: duplicates the values across steps and makes the fail-closed tests
  depend on per-step env propagation; a future step that needs the same keys
  must remember to re-declare them.
- Reason rejected: job-level env is the CI-native single declaration point and
  matches how the other gate inputs (`OYA_*`) are already supplied.

**Alternative 2 — Let the tests locate artifacts via buck2 `$(location)` only**
- Description: keep the fail-closed tests buck2-only, as before ADR-0716.
- Pros: no workflow env changes needed.
- Cons: contradicts ADR-0716 (cargo workspace graph is the merge path) and
  leaves the tests failing closed under `cargo test --workspace`.
- Reason rejected: ADR-0716 made the Cargo graph the CI merge path; the env-key
  bridge is the minimal way to keep the same fail-closed tests green there.

---

## References

- ADR-0716: Cargo is the CI merge path; buck2 is local hermeticity plus weekly smoke
- ADR-0515: single protected `oya-ci-required` context
- Related oyatie ADRs: ADR-0716, ADR-0515
- Issues: `Refs #1975`
