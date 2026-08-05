---
doc_class: Program-Prescriptions-Index
doc_status: published
authority_tier: 2
---
# Kubernetes Port Reusable Prescriptions
## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-05) |
|---|---|---|
| Repository baseline | `origin/dev` @ `b64eaaf4a` | Current baseline. |
| Upstream Kubernetes pin | `v1.36.1` tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Pinned program input. |
| Engine | `build/port-engine/*`, v0 — unbuilt | Not in force (W0-B). |
| Neutral rule pack | `specs/port-rules/**`, v0 — unauthored | Not in force (W0-B). |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force (W0-B). |
| Go front end | Bootstrap extractor; strategy ruled | Not in force. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force (W0-B). |
| Program authority | [ADR-0637](../../../decisions/ADR-0637-owned-deterministic-go-to-rust-port-engine.md) / [ADR-0638](../../../decisions/ADR-0638-mechanically-maintained-kubernetes-rust-port.md) | Accepted 2026-08-05; W0-only authority. |

## Purpose and extraction rule

A prescription is an executable, reusable procedure extracted from an operations-journal incident class. It is not an incident narrative and it does not create policy. Typical classes include reproducing a determinism failure, bisecting a rule regression, standing up the W2 hybrid topology, and regenerating vectors after a pin move.

At every wave gate, reviewers MUST group operations-journal records since the previous wave gate by incident class. Any class with two or more entries MUST have a prescription, unless the wave-gate record gives an explicit `no extraction` reason. The reason is a reviewable disposition, not an omission.

## Starvation condition and R-DOC

Lane (ii) starvation is **RED under R-DOC when the prescriptions lane is empty while the operations-journal lane grows across two consecutive gates.** `ci/facade/k8s-program-docs` is fail-closed: it is RED when any program document lacks the required baseline header; a completed wave has zero journal entries; a rule change has no journal reference; a doctrine entry is older than one wave without an ADR; or this starvation condition holds. A qualifying repeated class without a prescription requires an explicit no-extraction reason in the wave-gate journal entry.

The required review is population-aware. The prescriptions lane may correctly remain empty before repeated classes exist. It is not correctly empty when two consecutive gate populations show journal growth without the required extraction or recorded exception. This condition is separate from the operations-journal liveness predicate: **RED when the journal has zero entries for a completed wave (broken probe), not merely when a finding count is zero.**

## Required entry schema

A prescription is valid only when it contains each field below. It must be executable by a reviewer with the stated resources and inputs, and it must identify observed completion evidence rather than promise an outcome.

| Field | Required content |
|---|---|
| Prescription identity | Stable prescription ID, incident class, owner role, and status. |
| Extraction basis | Wave-gate reference, journal entry references, count of the incident class since the previous gate, or the exception disposition. |
| Purpose and trigger | The condition that selects this prescription; explicit non-applicability boundary. |
| Preconditions | Required repository base or pin, receipt-axis constraints, access, tools, configuration, and safety conditions. |
| Resources | CPU, memory, disk, IOPS, runner or worktree limits, network or corpus inputs, and expected duration class. |
| Procedure | Ordered commands and actions with arguments or configuration identity; each step's expected observable result and stop condition. |
| Evidence and terminal states | Required output or durable references; `passed`, `failed`, `interrupted`, `no-op`, and `unknown` handling. |
| Review and maintenance | Reviewer role, last reviewed wave, triggering journal references, and the condition for revision or retirement. |
| Escalation | Doctrine reference when the procedure contains a binding judgment; ADR reference when the judgment binds another lane. |

Use this document shape for each prescription, with the table fields represented as H2 sections in the same order:

```text
# <stable prescription ID>

## Prescription identity
## Extraction basis
## Purpose and trigger
## Preconditions
## Resources
## Procedure
## Evidence and terminal states
## Review and maintenance
## Escalation
```

The heading tokens are a schema, not a prescription. No prescriptions are listed by this initial index.

## Lane-first navigation

Index prescriptions by incident class and trigger. A chronological list, when entries exist, is secondary navigation and must link to the canonical prescription and its extraction-basis journal entries.
