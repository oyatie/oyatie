---
doc_class: Program-Operations-Journal-Index
doc_status: published
authority_tier: 2
---
# Kubernetes Port Operations Journal
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

## Purpose and routing

This is the primary record for per-run and per-wave operational facts: what was judged, what was fixed, rule-change rationale, red gates and root causes. Locate entries by incident class and lane first; use wave and run chronology second. A completed wave has a non-empty journal entry even when its finding count is zero, its result is no-op, or its run is interrupted.

Create a journal entry for each run and each wave gate. Store it under this lane by wave and stable run identifier. Its filename is the run identifier; its H1 is the same stable entry identifier. A rule change MUST cite the entry identifier in its change record, and the entry MUST name every touched rule ID or state `no rule change` with the reason.

## Required entry schema

A journal entry is valid only when it contains each required field below. Values are facts observed for that run; unknown values are recorded explicitly as `unknown` with the reason and terminal state, never omitted.

| Field | Required content |
|---|---|
| Entry identity | Stable entry ID, wave, run ID, incident or judgment class, and recording date. |
| Scope and inputs | Repository base and head SHA; Kubernetes pin; all six receipt axes when a receipt exists; relevant registry or source references. |
| Judgment | What was judged, the evidence considered, and the result. |
| Change disposition | What was fixed or why no fix was made; every touched rule ID and why it changed, or the exact `no rule change` reason. |
| Gate result | Each red gate, its root cause, and the result after repair or the reason it remains red. Record no-red-gate explicitly when that is the observed result. |
| Reproduction | Commands executed, arguments or configuration identity, and required resources: CPU, memory, disk, IOPS, runner or worktree limits, and external inputs. |
| Review | Reviewer role, verdict, review evidence reference, resolved and deferred findings. |
| Terminal state | `passed`, `failed`, `interrupted`, `no-op`, or `unknown`, plus the durable evidence or blocker reference. |
| Graduation links | Related prescription, doctrine, ADR, rule change, and wave-gate review references, when applicable. |

Use this document shape for each entry, with the table fields represented as H2 sections in the same order:

```text
# <stable journal entry ID>

## Entry identity
## Scope and inputs
## Judgment
## Change disposition
## Gate result
## Reproduction
## Review
## Terminal state
## Graduation links
```

The heading tokens are a schema, not a run entry. No journal entries are listed by this initial index.

## R-DOC enforcement

`ci/facade/k8s-program-docs` is fail-closed: it is RED when a program document lacks the required baseline header; a completed wave has zero journal entries; a rule change has no journal reference; a doctrine entry is older than one wave without an ADR; or the prescriptions lane is empty while the operations-journal lane grows across two consecutive gates. Its population-liveness predicate is **RED when the journal has zero entries for a completed wave (broken probe), not merely when a finding count is zero.**

A journal population is a liveness counter, not a success counter. It is valid for a live completed wave to have zero findings; it is invalid for a completed wave to have no journal entry. Determinism-gate liveness is separately fail-closed: zero scanned population is RED unconditionally, while zero findings with a nonzero scanned population is GREEN.

## Secondary chronology

Chronological listings may be added only as derived navigation after entries exist. They MUST link back to the lane-first entry and MUST NOT become the authority for classification, extraction, or review.
