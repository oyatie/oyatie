# ADR-MS-002 — Mass-Casualty Mode: Lifecycle, Composability, Partition Behavior

Status: Proposed
Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Date: 2026-05-21
Supersedes: none
Authority: ADR-0332 (in flight) | ADR-0248 | ADR-0251

---

## Context

Mass-Casualty Incident (MCI) mode is the most disruptive workflow ED-IS owns. It must:

- Activate quickly with strict Cedar gating (`mci-mode-activation.cedar`).
- Accept patients before their identity is known (tag-number primary key).
- Overlay the tracking board without erasing concurrent non-MCI patients.
- Support both START and SALT triage as configurable modalities.
- Function under cell partition (a regional disaster may sever cross-cell links).
- Compose with Drill mode without polluting production metrics.
- Reconcile MCI patients to canonical Patient records after the event.

Counterpart products handle this differently:

- T-System provides an MCI add-on with limited offline capability.
- Wellsoft has MCI workflows but no native disaster ICS integration.
- Cerner FirstNet supports MCI via add-on modules but couples tightly to Millennium.
- Epic ASAP has best-in-class MCI support inside the Epic stack but is not portable.

ED-IS must support an MCI activation that may outlast the partition that triggered it.

## Decision

MCI mode is a first-class state machine at the facility-cell level. Activation requires:

- Cedar-gated principal (attending or facility-emergency-manager).
- Explicit `drillModeFlagPresent = true` if it's a drill (drill activations write to a parallel ledger).
- Explicit choice of START or SALT triage modality.

MCI mode opens a local `MCIActivation` aggregate with append-only `MCIPatient` rows keyed by `tag_number`. The tracking board acquires an MCI overlay banner; non-MCI patients remain visible but de-prioritized.

Under cell partition:

- The local cell continues to accept MCI activations and triage writes.
- Cross-cell reconciliation events are buffered locally.
- On partition heal, an `ed.mci.reconciliation` event flushes buffered state to the control cell.
- Audit chain entries are local-first and replay on heal.

Reconciliation:

- A separate workflow merges `MCIPatient` rows to canonical `Patient` records once identity lands.
- The merge preserves the tag-number history as an alternate identifier on the Patient record.
- Identity reconciliation is performed by `identity` µservice's reconciler with attending sign-off.

Drill mode:

- Drill activations carry a `drillMode = true` flag end-to-end.
- Drill metrics flow to a separate `oya_emergency_drill_*` Prometheus namespace.
- Drill events are visibly distinguished on dashboards and never alter production SLOs.

## Consequences

Positive:

- MCI activation never blocks because of cross-cell coordination failure.
- Drill mode is safe to exercise during normal operations.
- Reconciliation is decoupled from the time-critical triage flow.

Negative / cost:

- Local-first behavior means cross-cell consistency is bounded-staleness during partition.
- Reconciliation requires a manual sign-off step, which may lag in a chaotic post-MCI cleanup.

## Alternatives Considered

- **Hot-active across cells via Paxos/Raft** — rejected: would block on cross-cell quorum, defeating the very property MCI mode needs (local autonomy under partition).
- **Single MCI mode flag without drill separation** — rejected: would force every drill to live in a parallel deployment, raising cost and reducing realism.
- **No tag-number identifier (require identity at registration)** — rejected: breaks the canonical MCI workflow where dozens of patients arrive before identification is possible.

## Open Items

- Tag-number namespace allocation across facilities — needs an `identity` µservice convention; handled in IP-004.
- Cross-pack START vs SALT default — likely US default START; EU pack defaults to SALT.
- Decedent handling under MCI — composes with `decedent-affairs` workflow once that µservice lands.

## Authority Trail

- ADR-0332 (in flight) — the parent µservice ADR.
- ADR-0248 — Amazon-shape cellular architecture (partition tolerance).
- ADR-0251 — Compliance pack primitive (TJC EM chapter).
- ADR-0064 — Canonical-base neutrality.
