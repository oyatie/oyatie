---
doc_class: Program-Operating-Guide
doc_status: published
authority_tier: 2
---
# Kubernetes Go-to-Rust Port Program
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
| Program authority | [ADR-0637](../../decisions/ADR-0637-owned-deterministic-go-to-rust-port-engine.md) / [ADR-0638](../../decisions/ADR-0638-mechanically-maintained-kubernetes-rust-port.md) | Accepted 2026-08-05; W0-only authority. |

## Authority

ADR-0637 and ADR-0638 govern this program. This guide operationalizes their record-lane requirement; it does not amend the ADRs, approve W1+, ratify a measured threshold, or make an unlanded engine, rule pack, front end, gate, or receipt schema operative.

The approved program plan §A and §B.5 define the baseline header and the three standing record lanes. Repository-wide operating requirements remain governed by [`docs/AGENTS.md`](../../AGENTS.md). The record lanes are evidence and routing surfaces, not substitutes for the scope, detached-surface, rule-pack, canary, or receipt registries.

## Routing

Use the lane that matches the record's purpose before looking for a date:

| Need | Primary lane | Route onward |
|---|---|---|
| A run or wave judgment, fix, gate failure, or no-rule-change rationale | [`operations/`](operations/INDEX.md) | Repeated incident class → prescription review. |
| A repeatable execution procedure | [`prescriptions/`](prescriptions/INDEX.md) | Binding judgment in a procedure → doctrine. |
| A rare, binding cross-lane judgment | [`doctrine/`](doctrine/INDEX.md) | Binding elsewhere → ADR within one wave. |

Chronological listings are secondary navigation only. An entry is found by lane and incident or judgment class first, then by wave and run.

## Cadence

- Create one operations-journal entry for every run and for every completed wave gate, including no-op, interrupted, and failed outcomes.
- At every wave gate, review journal entries since the prior gate by incident class. A class occurring two or more times MUST have a reusable prescription, unless the gate records an explicit no-extraction reason.
- Review doctrine at each wave gate. A doctrine judgment that becomes binding on another lane MUST graduate to an ADR within one wave.

## Traceability and R-DOC

| Obligation | Mechanical artifact | Enforcing gate |
|---|---|---|
| Upstream semantic fidelity | SourceModel snapshot digest plus derived `specs/k8s-port/correspondence` | `k8s-port-coverage` |
| Derived output, never authored | six-axis `verify()` receipt plus `specs/k8s-port/boundary.json` | regenerate-twice and manual-edit refusal |
| Gate liveness | `specs/port-rules/canary/index.json` | each determinism gate's scanned-population predicate |
| Rule liveness | selecting fixture for every row in `specs/port-rules/index.json` | behavioral rule-mutation canary |
| Scope discipline | `specs/k8s-port/scope.json` | scope-drift check |
| Detachment control | `specs/k8s-port/detached.json` | identity-set no-growth ratchet |
| Test totality | pin-versioned upstream test-ID disposition manifest | Layer 0 accounting |
| Generated-artifact lifecycle | registered producer and two-pass parity relation | ADR-0597 freshness gate |
| Fact ownership | `k8s/core/port-accounting-kernel` counters | ADR-0633 population-liveness predicates |
| Licensing and attribution | `upstream-pin.json`, `licensing.json`, per-file provenance, and generated `k8s/NOTICE` | supply-chain and license policy |
| Program-memory durability | operations, prescriptions, and doctrine entries | `ci/facade/k8s-program-docs` R-DOC |

Every program document MUST carry the baseline version header above. The `ci/facade/k8s-program-docs` R-DOC gate is fail-closed and RED when any of these conditions holds:

1. A program document lacks its baseline header.
2. A wave gate lacks a non-empty operations-journal entry.
3. A rule change lacks an operations-journal reference.
4. A doctrine entry is older than one wave without an ADR.
5. Lane (ii) is empty while lane (i) grows across two consecutive gates.

The population-liveness predicate is: **RED when the journal has zero entries for a completed wave (broken probe), not merely when a finding count is zero.** A nonzero journal population proves that the required probe ran; it does not itself prove an incident. Separately, every determinism gate has a scanned-population counter and a finding counter: zero scanned population is RED unconditionally, while zero findings with nonzero scanned population is GREEN. Registered canary regions provide the nonzero liveness floor when a determinism gate is wired.

## Lifecycle

1. Record an operation in the journal at run close and wave-gate close.
2. Classify the record by incident or judgment class; maintain chronology only as a secondary index.
3. At the next wave gate, extract repeatable classes into prescriptions or record the required no-extraction reason.
4. Move a prescription that states a binding judgment to doctrine.
5. Promote doctrine that binds another lane to an ADR within one wave; the ADR supersedes the doctrine entry as authority.

Entries do not disappear at graduation. They retain links to their prescription, doctrine, ADR, and relevant registry or receipt so that a later wave can reconstruct the decision path.

## Review roles

| Surface | Accountable review role | Required review focus |
|---|---|---|
| Program root and operations journal | `axis-cloud-platform` | Run/wave facts, Kubernetes and engine traceability, terminal-state evidence. |
| Prescriptions | `axis-cloud-platform` | Executability, safe prerequisites, expected evidence, and extraction basis. |
| Doctrine and ADR graduation | `council-architecture` | Whether the judgment is binding, cross-lane impact, and ADR promotion. |
| R-DOC gate result | `axis-cloud-platform` and `council-architecture` | Header, liveness, starvation, journal references, and overdue doctrine. |

These are team roles, not individual assignments. Required repository review and merge controls remain independent of lane ownership.

## Non-claims

This directory does not claim that a run occurred, that a wave completed, that an incident was fixed, that a rule changed, or that a gate passed. ADR acceptance is determined only by the ADR records. This directory creates the governed places and schemas in which operational facts must be recorded when they exist. It does not authorize manual edits to derived port output or advisory-mode determinism gates.
