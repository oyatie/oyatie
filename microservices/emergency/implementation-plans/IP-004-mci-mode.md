# IP-004 — Mass-Casualty Mode + START/SALT + Identity Reconciliation

Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Authority: ADR-MS-002 | ADR-0332 (in flight) | ADR-0248
Sequence: 4 / 10
Depends-on: IP-001, IP-002

---

## Scope

MCI activation with tag-number-based patient creation, START and SALT triage modalities, drill mode separation, partition-tolerant local-first behavior, and a reconciliation workflow to merge MCI patients into canonical Patient records.

## Deliverables

- `src/crates/emergency-mci/` — MCI activation + MCIPatient aggregate.
- START / SALT triage workflows.
- Tag-number identifier + reconciler.
- Drill mode flag with parallel metrics namespace.
- Cedar `mci-mode-activation.cedar` enforced.
- `MciActivate`, `MciDeactivate`, `MciTriageWrite`, `MciPatientReconcile` gRPC RPCs.
- `ed.mci.activated`, `ed.mci.deactivated` events.
- Partition-tolerance test (cell-partition replay).

## Acceptance

- MCI activation < 1 s end-to-end.
- 20-patient triage burst in MCI mode under 5 minutes wall-clock.
- Drill metrics never leak to production namespace.
- Reconciliation merges tag → canonical patient without data loss.
