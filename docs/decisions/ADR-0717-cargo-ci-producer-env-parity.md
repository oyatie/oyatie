---
doc_status: published
id: ADR-0717
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
  - id: ADR-0717-D1
    description: "The new cargo workflow's test job must set the two env keys two fail-closed cross-artifact tests require: OYA_HISTORY_ONLY_RETIREMENT_FACTS (the materialized history-only retirement facts face path) and OYA_ADR_INDEX_PRODUCER_BIN (the sanctioned ADR-index producer binary). Both are job-level env, not run-step env."
    exit_criteria: "The test job env block declares both keys; the fail-closed cross-artifact tests resolve the materialized face and exec the producer binary without local overrides."
    verified_by: "cargo test --workspace with the CI env on the PR head"
  - id: ADR-0717-D2
    description: "The 'Build enforcement-liveness producer' step also builds the ADR-index producer binary (`cargo build --locked -p marketplace-dev-cli --bin oya`) so OYA_ADR_INDEX_PRODUCER_BIN resolves in the same step that already builds the accounting-registry producer."
    exit_criteria: "The step run builds both binaries; the automation-language-policy inline-shell baseline accepts the step's run-content change through a reviewed replacement window (schema_version +1, substantive reason, this ADR)."
    verified_by: "ci-automation-language-policy live-corpus tests on the PR head"
---

# Cargo CI producer env parity

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
