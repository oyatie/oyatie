---
doc_class: Program-Doctrine-Index
doc_status: published
authority_tier: 2
---
# Kubernetes Port Doctrine
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

## Purpose and authority boundary

Doctrine records rare, above-daily binding judgments: direction changes, principle amendments, and assertions that rules always behave in a particular way. It is not a place for a reusable procedure, routine run fact, or repeated incident narrative. Route those records to prescriptions or the operations journal first.

Doctrine is provisional local authority only while its judgment is confined to this lane. When the judgment is binding on another lane, it MUST graduate to an ADR within one wave. The doctrine entry then cites the ADR and stops being authority; it remains a traceability record.

## Required entry schema

A doctrine entry is valid only when it contains each field below. It must distinguish observed facts from the judgment that binds later work and state the exact scope of that judgment.

| Field | Required content |
|---|---|
| Doctrine identity | Stable doctrine ID, title, authoring wave, owner role, and status. |
| Judgment | The binding judgment in normative language, its intended scope, and explicit non-scope. |
| Basis | Operations-journal, prescription, rule, receipt, registry, or external-authority references that support the judgment. |
| Alternatives | Material alternatives considered, why they were not selected, and consequences. |
| Binding analysis | Lanes affected now, the test for whether another lane is bound, and the required ADR graduation deadline. |
| Implementation impact | Required rule, gate, registry, receipt, or documentation updates and their traceability references. |
| Review | `council-architecture` review verdict, reviewer evidence, dissent, and disposition. |
| Lifecycle | Current wave, next wave review point, ADR reference when graduated, and statement that the ADR supersedes this entry as authority. |

Use this document shape for each doctrine entry, with the table fields represented as H2 sections in the same order:

```text
# <stable doctrine ID>

## Doctrine identity
## Judgment
## Basis
## Alternatives
## Binding analysis
## Implementation impact
## Review
## Lifecycle
```

The heading tokens are a schema, not a doctrine entry. No doctrine entries are listed by this initial index.

## R-DOC deadline and liveness

`ci/facade/k8s-program-docs` is fail-closed: it is RED when a program document lacks the required baseline header; a completed wave has zero journal entries; a rule change has no journal reference; a doctrine entry is older than one wave without an ADR; or the prescriptions lane is empty while the operations-journal lane grows across two consecutive gates. A doctrine entry that binds another lane MUST graduate to an ADR within one wave; it then cites the ADR and stops being authority.

R-DOC also requires the shared baseline header and completed-wave journal liveness. Its population-liveness predicate is **RED when the journal has zero entries for a completed wave (broken probe), not merely when a finding count is zero.** A zero finding count is not a reason to suppress a completed-wave journal record.

## Lane-first navigation

Index doctrine by judgment class and affected lane. Any chronology is secondary and must link to the canonical doctrine entry and its superseding ADR where applicable.
