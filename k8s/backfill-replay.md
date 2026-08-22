---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud
deciders: axis-cloud, council-architecture, ops-sre-reliability
related_adrs: [ADR-0121, ADR-0131]
related_artifacts:
  - microservices/cloud-k8s/PRD.md
  - microservices/cloud-k8s/capacity-model.md
  - microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md
  - microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (cloud-k8s µservice)

## Purpose

Specify how cloud-k8s handles two scenarios:

1. **Backfill** — after etcd restore from snapshot OR after migrating a workload between clusters, replay the cluster-event stream to bring sibling µservices' state machines back into sync.
2. **Replay** — an existing audit-chain record needs re-emission (e.g., audit-chain µservice repair after a chain-integrity issue; forensic re-derivation).

## Backfill

### Contract

After an etcd restore (per `runbooks/etcd-quorum-recovery.md`) or cluster migration, the cluster state ledger between snapshot-time and restore-time is lost from the live etcd but **preserved in audit-chain**.

cloud-k8s backfill primitive:

1. Receives `BackfillRequested` event from `audit-chain` µservice (the source of truth for cluster events).
2. Reads audit-chain records for the affected (cluster_id, window) range.
3. Computes the delta between snapshot-state and audit-chain-state:
   - `NodeJoined` events for nodes added in the window → re-emit `NodeJoined` for current sibling µservices
   - `NetworkPolicyApplied` events → re-emit (idempotent; sibling µservices treat as no-op if state matches)
   - `IstioPolicyChanged` events → re-emit
4. Backfilled events carry label `backfilled=true` so consumers distinguish:
   - `cell` µservice: backfilled events DO NOT trigger re-scheduling
   - `observability` µservice: backfilled events emit metrics with `kind=backfilled`
   - `workflow-engine`: backfilled events DO NOT trigger downstream workflows

### Constraints

- Backfill does NOT change historical audit-chain records. Audit-chain is immutable per Bominal ADR-0028.
- Backfill is bounded by audit-chain retention (≥ 2y default; ≥ 6y for pack-us-healthcare).
- Per-cluster backfill rate-limited: max 1 backfill per cluster per 24h (anti-abuse).

### Verification

- Integration test: induce etcd restore in test cluster; verify backfill re-syncs sibling µservices within 15min.
- Idempotency: re-running same backfill produces same delta.

## Replay

### Contract

Replay re-emits cluster events for a specific (cluster_id, window) without state mutation. Triggers:

- audit-chain repair after chain-integrity issue (rare; audit-chain self-heals from peer chains)
- Forensic re-derivation during incident investigation
- Compliance audit: auditor asks for cluster-event sequence at point-in-time

### Procedure

1. Operator invokes: `cargo run -p dev-cli -- cloud-k8s replay-events --cluster <id> --from <ts> --to <ts> --reason "<rfc>"`.
2. CLI requires 2-person rule + ops-security approval (replay can shift historical "perceived truth" for downstream consumers).
3. Engine reads audit-chain records for the window; emits as Workflow events with `replayed=true` label.
4. Audit-chain seal: the replay is itself sealed, distinguishing it from the original event.

### Constraints

- Replay does NOT mutate the original event records.
- Replay cannot exceed audit-chain retention.
- Replay output is read-only for downstream consumers; sibling µservices see `replayed=true` and treat as informational.

### Verification

- Integration test: replay over a known window; verify event count + signatures match audit-chain.
- Audit-chain integrity: replay event is sealed; original chain unaffected.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill after etcd restore | per-restore (rare) | ~$5 (read audit-chain + emit ≤ thousands of events) |
| Replay for forensic | per-incident | ~$2 (single cluster × bounded window) |
| Replay for compliance audit | per-engagement | ~$5 (auditor's window scoped + audit-chain read) |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers" — backfill/replay budgeted as part of cluster-bootstrap-worker compute envelope.

## Limitations

- Backfill quality bounded by audit-chain retention (≥ 2y default).
- Replay assumes audit-chain integrity; if chain has been compromised, replay output may be inaccurate (the chain-integrity check itself is the trust root).
- Replay cannot reconstruct in-flight (non-sealed) events; only sealed events replayable.

## References

- `microservices/cloud-k8s/PRD.md`.
- `microservices/cloud-k8s/capacity-model.md`.
- `microservices/cloud-k8s/cost-budget.md`.
- `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`.
- `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`.
- ADR-0028 (Bominal audit-chain); ADR-0121.
- Google SRE Workbook ch. 5 (managing operational load).
