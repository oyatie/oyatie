---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-010-retention-legal-hold-bcs
status: pending
owner: ops-compliance + axis-recordings + council-privacy
acceptance_lanes: [retention-policy-correctness, legal-hold-chain-of-custody-correctness]
load_bearing: true
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: Retention-policy BC + Legal-hold BC (LOAD-BEARING)

## Intent

Land the per-pack retention policy enforcement + load-bearing 100 %
correctness legal-hold engagement (per ADR-RECORDINGS-0002). The two
canonical load-bearing CI lanes go green here.

## Concrete crates

- `oya-recordings-retention-policy-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}`
- `oya-recordings-legal-hold-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}`

## Acceptance Gates

```bash
buck2 build //:quality-lane-registry-authority-check # lane=retention-policy-correctness --microservice recordings
buck2 build //:quality-lane-registry-authority-check # lane=legal-hold-chain-of-custody-correctness --microservice recordings
cargo nextest run -p oya-recordings-legal-hold-kernel -- engagement_latency_p99
```

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-010-retention-legal-hold-bcs
depends_on_changesets: [CS-RECORDINGS-IP-004-recording-bc]
parallel_safe_with_changesets: [CS-RECORDINGS-IP-008-redaction-bc, CS-RECORDINGS-IP-011-playback-share-link-watermark-bcs]
enables: [CS-RECORDINGS-IP-012-export-ediscovery-bcs]
acceptance_status: ga
load_bearing: true
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Per-pack retention defaults honoured (SEC 17a-4 WORM, HIPAA 6y, KR 전자문서법) | `cargo nextest run -p oya-recordings-retention-policy-domain -- pack_defaults_honoured` |
| AC-02 | Retention monotonic — cannot be shortened after applied | `cargo nextest run -p oya-recordings-retention-policy-domain -- retention_monotonic` |
| AC-03 | Legal-hold engagement p99 ≤ 1s (PRD Tenant Outcome 2) | `cargo nextest run -p oya-recordings-legal-hold-kernel -- engagement_latency_p99` |
| AC-04 | Legal-hold suspends retention purge with audit-chain seal | `cargo nextest run -p oya-recordings-legal-hold-usecase -- suspends_purge` |
| AC-05 | `oya gate validate retention-policy-correctness + legal-hold-chain-of-custody-correctness --microservice recordings` exits 0 | governance lanes |

## Build Sequence

1. Kernel: `RetentionPolicyStore`, `LegalHoldStore`, `PurgeScheduler` ports.
2. Domain: `RetentionPolicy`, `Hold`, `PurgeSchedule`.
3. Usecase: `ApplyRetention`, `EngageHold`, `ReleaseHold`, `ExecutePurge`.
4. Postgres adapter; worker that runs purge scheduler.
5. Load-bearing gates green per AC-05.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings FR | FR-08 (legal hold), FR-11 (tenant retention override) |
| PRD-recordings Tenant Outcome | TO-2 |
| ADR | ADR-RECORDINGS-0002 (load-bearing) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Purge scheduled while hold is engaged | Atomic check-hold-then-purge; hold engagement test |
| Retention shortened mid-flight | Monotonic invariant test refuses |
| Hold release without two-person rule | Cedar policy requires two distinct principals |

## References

- ADR-RECORDINGS-0002.
- SEC 17a-4(f) (Records to be made by certain exchange members, brokers and dealers).
- FINRA Rule 4511 (General Requirements).
- HIPAA §164.316 (Policies and procedures and documentation requirements).
- KR 전자문서 및 전자거래 기본법 (Electronic Documents and Transactions Framework Act).
- The Sedona Conference Commentary on Legal Holds, 2d ed. (2017).

## Next IP

[`IP-011-playback-share-link-watermark-bcs.md`](IP-011-playback-share-link-watermark-bcs.md)
