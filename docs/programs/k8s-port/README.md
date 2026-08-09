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

Two rows of that table — `Gate liveness` and `Rule liveness` — name artifacts that CANNOT exist under the landed enforcing gate. They are left exactly as written rather than silently corrected, because which surface is authoritative is a governance question this program has not answered. See open question `OQ-K8SPORT-001` below.

Every program document MUST carry the baseline version header above. The `ci/facade/k8s-program-docs` R-DOC gate is fail-closed and RED when any of these conditions holds:

1. A program document lacks its baseline header.
2. A wave gate lacks a non-empty operations-journal entry.
3. A rule change lacks an operations-journal reference.
4. A doctrine entry is older than one wave without an ADR.
5. Lane (ii) is empty while lane (i) grows across two consecutive gates.

The population-liveness predicate is: **RED when the journal has zero entries for a completed wave (broken probe), not merely when a finding count is zero.** A nonzero journal population proves that the required probe ran; it does not itself prove an incident. Separately, every determinism gate has a scanned-population counter and a finding counter: zero scanned population is RED unconditionally, while zero findings with nonzero scanned population is GREEN. Registered canary regions provide the nonzero liveness floor when a determinism gate is wired.

## Open questions

Recorded so that no lane resolves one silently. An open question is closed by its owner, in a change that states which surface it made authoritative — never by a lane editing whichever surface is cheaper to edit.

| ID | Question | Owner | Opened | Status |
|---|---|---|---|---|
| `OQ-K8SPORT-001` | The traceability table above names `specs/port-rules/canary/index.json` and `specs/port-rules/index.json`. Neither path can exist while `ci/facade/k8s-program-docs` is the enforcing gate: its `load_rule_records` walks `specs/port-rules` recursively and returns `R-DOC-RULE-METADATA-MALFORMED` ("rule records must be Markdown with YAML-style front matter") for any path whose extension is not `md`, then `ensure_only_fields` rejects any front-matter key outside `rule_id`, `rule_kind`, `operations_journal_ref`. That is a LOAD error, not a finding, so a single `.json` under that root reddens the gate before any other R-DOC check evaluates. Which surface is authoritative — this table, or the landed gate — is undecided. | `council-architecture` (owns `specs/`; owns doctrine and ADR graduation per the review-roles table below). `axis-cloud-platform` is the co-accountable role for the R-DOC gate result. | 2026-08-09, by the G006 language-rule-pack lane | OPEN. Neither surface has been changed to match the other. |

Until it is closed, the G006 lane behaves as [`MAPPING-G006-go-rust-language-pack.md`](MAPPING-G006-go-rust-language-pack.md) §2 D1 rules: rule records are Markdown only, rule order is byte-lexicographic on `rule_id` (which equals the filename stem, so no index file is needed for a deterministic order), and fixtures are fenced blocks inside the record. That ruling governs one lane's conduct under the contradiction. It does not resolve the contradiction and is not a substitute for closing it.

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
