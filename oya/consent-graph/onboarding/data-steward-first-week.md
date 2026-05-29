---
doc_class: Onboarding
microservice: consent-graph
persona: data-steward + partnership-manager
related_adrs: [ADR-0214, ADR-SVC-CG-001, ADR-SVC-CG-003, ADR-SVC-CG-005, ADR-0131]
date: 2026-05-20
doc_status: published
---

# Data Steward onboarding — first 5 working days on `consent-graph`

Audience: a new data-steward or partnership-manager joining the `consent-graph` rotation. By Day-5 they will have: drafted a DataSharingAgreement, walked the three sharing modes, exercised the real-time revocation drill, processed a cross-tenant DSAR conflict, and shadowed a Snowflake-Secure-Data-Share comparison demo.

## Day 1 — Tour the substrate

1. Read `PRD.md` § 1 problem statement + § 3 goals (∼ 30 min) + `decisions/ADR-SVC-CG-001-bilateral-chain-link-schema.md` + `decisions/ADR-SVC-CG-002-cedar-cache-invalidation.md` + `decisions/ADR-SVC-CG-003-three-sharing-modes.md` (∼ 90 min).
2. Open the Grafana folder `consent-graph`. Identify boards: `cg-agreement-state-machine`, `cg-projection-freshness`, `cg-cedar-cache-invalidation-latency`, `cg-revocation-propagation`, `cg-cross-tenant-projection-rows-per-second`, `cg-dsar-conflict-rate`.
3. Walk the runbook index. On-call runbooks: `revocation-propagation-stalled.md`, `cedar-cache-divergence.md`, `cross-tenant-projection-storm.md`, `dsar-on-active-agreement.md`, `bilateral-chain-integrity-failure.md`, `partner-directory-trust-anchor-rotated.md`, `cross-border-transfer-blocked.md`.
4. Read EU GDPR Art. 28 (sub-processor) + Art. 46 (cross-border transfer) + KR PIPA cross-border-transfer rules + US-HIPAA BAA model.

Acceptance: you can articulate the 5 guarantees from PRD §1: audit defensibility, revocability, sovereignty, scope narrowness, real-time.

## Day 2 — Draft a DataSharingAgreement

Read `decisions/ADR-SVC-CG-001-bilateral-chain-link-schema.md`. The agreement lifecycle:

1. `draft` — grantor authoring.
2. `offered` — grantor publishes; grantee can review.
3. `accepted` — both sides sign.
4. `active` — projections flow.
5. `paused` — temporary suspension (e.g., for review).
6. `revoked` — terminated.
7. `archived` — read-only historical record.

Draft an agreement:

```sh
oya consent-graph agreement draft \
    --grantor drill-acme \
    --grantee drill-partner-co \
    --scope-spec ./agreement-drafts/acme-partner-projection.yaml \
    --sharing-mode projection \
    --window 2026-06-01..2027-05-31 \
    --geographic-constraint same-region \
    --output draft-agreement.json
```

The scope-spec defines:

- The Ontology entity (e.g., `Order`).
- Field narrowing (e.g., share `order_id, total_amount, created_at` but NOT `customer_pii`).
- Predicate narrowing (e.g., share orders where `status = 'shipped'`).
- Optional k-anonymity for Aggregate mode.

Offer the agreement to the grantee:

```sh
oya consent-graph agreement offer \
    --agreement-id ag-draft-2026-05-20 \
    --notify-grantee true
```

Acceptance from the grantee side:

```sh
# Acting as drill-partner-co
oya consent-graph agreement accept \
    --agreement-id ag-draft-2026-05-20 \
    --grantee-evidence ./acceptance-evidence.pdf
```

Audit chain emits `agreement_offered`, `agreement_accepted`, `agreement_activated`.

Acceptance: you can articulate the 7-state lifecycle + the bilateral nature (BOTH sides emit audit-chain events at each transition).

## Day 3 — Walk the three sharing modes

Read `decisions/ADR-SVC-CG-003-three-sharing-modes.md`.

**Projection mode**: each row that satisfies the scope spec appears in the grantee's projection topic; the grantee's app subscribes via SDK.

```sh
oya consent-graph projection subscribe \
    --grantee drill-partner-co \
    --agreement-id ag-active-2026 \
    --target-topic drill-partner-co.projections.acme-orders
```

The projection-gateway:

1. Cedar gate `consent-graph::projection::read` evaluates per row.
2. Mints a per-projection ACL bound to the agreement + the grantee's principal.
3. Emits each qualifying row to the grantee's topic; emits `projection_row_emitted` audit event.
4. Grantor's row never physically migrates (per PRD goal §5; per IP-009 + IP-010).

**Aggregate mode**: instead of per-row data, the grantee receives statistical aggregates (count, mean, sum, percentile) over the scope spec. Includes k-anonymity threshold (e.g., k ≥ 10 means no aggregate computed for groups < 10 rows).

```sh
oya consent-graph agreement update \
    --agreement-id ag-active-2026 \
    --add-sharing-mode aggregate \
    --aggregate-spec ./agreement-drafts/acme-partner-aggregate.yaml
```

The aggregate-spec defines the aggregation function + the k-anonymity threshold.

**AttestedQuery mode**: the grantee submits a query (must match the scope spec); the grantor evaluates it; the grantor signs the result + sends back as an audit-attested response.

```sh
oya consent-graph attested-query submit \
    --grantee drill-partner-co \
    --agreement-id ag-active-2026 \
    --query "SELECT COUNT(*) FROM orders WHERE shipped_to_country = 'CA' AND order_date >= '2026-05-01'"
```

The grantor's side:

1. Receives the query via Pulsar.
2. Cedar gate evaluates against the scope spec (allowed queries are constrained).
3. Executes the query on grantor's data.
4. Signs the result + sends back.
5. Emits `attested_query_executed` audit event on BOTH sides.

Acceptance: you can articulate when each mode is appropriate (Projection: operational use; Aggregate: analytics; AttestedQuery: ad-hoc + audit-trail-critical).

## Day 4 — Real-time revocation drill

Read `runbooks/revocation-propagation-stalled.md`.

Drill a revocation:

```sh
oya consent-graph drill revocation \
    --agreement-id ag-active-2026 \
    --revoker drill-acme \
    --reason business-relationship-ended \
    --propagation-target p99-1s
```

The drill measures:

- Revocation initiated → Cedar cache invalidation pulsar fanout → grantee's projection-gateway rejects next read.
- Target: p99 ≤ 1 s (per PRD goal §4).

Watch the `cg-revocation-propagation` dashboard. Expected:

- Pulsar fanout from revocation event → all 12 Cedar evaluator workers receive within ~ 200 ms.
- Each worker invalidates its local Cedar cache within ~ 100 ms.
- Next projection read by the grantee within the cache TTL window → denied (Cedar policy is the new "revoked" state).

Now provoke the divergence case (a Cedar worker missed the invalidation):

```sh
oya consent-graph drill cedar-cache-divergence \
    --agreement-id ag-active-2026 \
    --simulate-pulsar-worker-isolated true
```

The isolated worker would continue to serve cached "active" policy until cache TTL expires. The cell's invariant: cache TTL is bounded ≤ 30 s; after TTL, the worker re-queries the source of truth + sees "revoked".

Acceptance: you can articulate the trade-off between cache-warm performance + revocation latency (lower TTL = faster revocation but more cache misses).

## Day 5 — Cross-tenant DSAR conflict + Snowflake comparison

Walk a DSAR conflict:

The data subject (an end-user of the grantor `drill-acme`) files a GDPR Art. 17 right-to-erasure. The user's data is currently being projected to `drill-partner-co` per an active agreement.

```sh
oya consent-graph drill dsar-on-active-agreement \
    --grantor drill-acme \
    --grantee drill-partner-co \
    --data-subject drill-user-z \
    --dsar-source gdpr-art-17 \
    --agreement-id ag-active-2026
```

The system response:

1. DSAR received → `dsar_received` audit event.
2. The user's rows are immediately suppressed from the projection (the projection-gateway adds a "tombstone" record).
3. The grantee is notified.
4. Per ADR-SVC-CG-005, the data-subject's request reaches the grantee within the GDPR 30-d window.
5. The grantee processes the deletion on their side; emits `data_subject_deleted_on_grantee_side` audit event.
6. After all grantees confirm, the grantor's original record may be deleted (or anonymized per the GDPR Art. 17 erasure right).

If the grantee REFUSES to delete (e.g., legal hold per GDPR Art. 17(3)), the agreement is paused + escalated to the joint DPO meeting.

Acceptance: you can articulate the cross-tenant DSAR coordination + the role of the bilateral chain in proving deletion happened on the grantee side.

Now shadow the Snowflake comparison demo:

```sh
oya consent-graph demo snowflake-comparison \
    --shape "all 5 guarantees side-by-side"
```

The demo walks through the same use case (B2B partner data sharing) implemented in:

- Snowflake Secure Data Share (passes guarantees 1 + 3; fails 2 + 4 + 5).
- EDI (passes none).
- Hyperledger (passes 1; burns the others).
- oyatie consent-graph (passes all 5).

Acceptance: you can demo the differentiators to a non-technical audience.

## What you've learned

- The DataSharingAgreement state machine + the bilateral chain shape.
- The 3 sharing modes + when each is appropriate.
- The Cedar cache invalidation + the revocation propagation budget.
- The cross-tenant DSAR coordination.
- The differentiators vs Snowflake / EDI / Hyperledger.

Next week: partner-directory handshake walkthrough, cross-pack federation (KR + EU) shadow, bilateral Merkle chain integrity drill.
