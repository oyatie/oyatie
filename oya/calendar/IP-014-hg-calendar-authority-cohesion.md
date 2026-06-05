---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-014-hg-calendar-authority-cohesion
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + council-architecture
acceptance_lanes: [hyperscaler-maturity-claim, parity-matrix-sync, doc-link-resolve]
---

# IP-014: HG-CALENDAR authority cohesion

## A. Problem
Calendar has many claim surfaces: PRD, architecture, competitor matrix, feature matrix, benchmarks, contracts, SLOs, policies, and runbooks. Hyperscaler maturity cannot be claimed if these disagree.

## B. Approach
Bind the HG-CALENDAR claim to repo-local evidence only. The authority chain starts at PRD and manifest, then traces through architecture, parity matrices, performance targets, contracts, policies, SLOs, dashboards, runbooks, and catalog entries.

## C. Deliverables
| Artifact | Role |
|---|---|
| `PRD.md` | Product and requirement authority. |
| `ARCHITECTURE.md` | Operating-contract and anchor authority. |
| `manifest.json` | Machine-readable service inventory. |
| `competitor-parity-matrix.md` and `feature-parity-matrix-2026-05-20.md` | Counterpart claim boundaries. |
| `performance-benchmark-numbers-2026-05-20.md` and `benchmarks/gcal-outlook-calendly-vs-oyatie.md` | Target and workload evidence. |

## D. Ordered implementation steps
1. Resolve every manifest IP, contract, policy, SLO, and catalog path.
2. Cross-check PRD must-haves against OpenAPI/AsyncAPI/proto surfaces.
3. Cross-check competitor claims against the two parity matrices.
4. Ensure SLO names map to dashboards and runbooks.
5. Flag future claims that depend on missing implementation as targets, not measured evidence.
6. Run the hyperscaler maturity gate.
7. Record accepted gaps and rejected marketing claims in the changeset.

## E. Acceptance
- `buck2 build //:quality-lane-registry-authority-check # lane=hyperscaler-maturity --microservice calendar` passes.
- Doc-link resolution passes for every cited local file.
- Manifest JSON parses and every listed IP file exists.
- No parity claim contradicts `competitor-parity-matrix.md`.
- Missing source/tests remain explicitly marked as plan targets, not production proof.

## F. Evidence
- `microservices/calendar/PRD.md`.
- `microservices/calendar/ARCHITECTURE.md`.
- `microservices/calendar/manifest.json`.
- `microservices/calendar/competitor-parity-matrix.md`.
- `microservices/calendar/feature-parity-matrix-2026-05-20.md`.
- `microservices/calendar/performance-benchmark-numbers-2026-05-20.md`.

## G. Counterpart comparison
Google, Outlook, Apple, Fastmail, Proton, Cal.com, Calendly, and Doodle each define part of the maturity bar. This IP does not implement a feature; it prevents overstating Oyatie by requiring every counterpart claim to trace to repo evidence and every gap to remain bounded.

## H. Foundation delivery expansion
- Deliverable detail: authority graph starts at PRD, manifest, architecture, and this IP set.
- Deliverable detail: parity claims resolve to competitor and feature matrices before they appear in status summaries.
- Deliverable detail: performance claims resolve to benchmark and SLO files, not prose assertions.
- Deliverable detail: contract claims resolve to OpenAPI, AsyncAPI, and proto paths.
- Deliverable detail: policy claims resolve to Cedar files, data-residency notes, compliance, and DPIA files.
- Deliverable detail: runbook claims resolve to exact operational files for each SLO-backed failure mode.
- Deliverable detail: missing source implementation stays labeled as planned evidence.
- Deliverable detail: Slack collaboration-calendar pressure is recorded as a comparison vector, not hidden under Google/Outlook only.

## I. Acceptance expansion
- Acceptance detail: doc-link resolution must cover every file listed in PRD, architecture, manifest, parity, SLO, policy, and runbook sections.
- Acceptance detail: hyperscaler gate must fail when a competitor claim lacks a local evidence path.
- Acceptance detail: manifest parsing must prove every foundation IP file exists.
- Acceptance detail: feature-parity and competitor matrices must not contradict each other.
- Acceptance detail: performance benchmark claims must distinguish target from measured evidence.
- Acceptance detail: remediation notes must record count and scope of the foundation IP repair.
- Acceptance detail: branch promotion must include an evidence bundle for this authority graph.
- Acceptance detail: Slack, Google, Outlook, and Cal.com comparisons must be named where they drive different requirements.

## J. Evidence expansion
- Evidence detail: capture hyperscaler-maturity gate output for calendar.
- Evidence detail: capture doc-link resolution output over calendar files.
- Evidence detail: capture manifest parse output.
- Evidence detail: cite `competitor-parity-matrix.md` for counterpart boundaries.
- Evidence detail: cite `feature-parity-matrix-2026-05-20.md` for feature-by-feature status.
- Evidence detail: cite `performance-benchmark-numbers-2026-05-20.md` for target numbers.
- Evidence detail: cite Slack as collaboration-calendar interop pressure that must remain explicit in the claim map.
