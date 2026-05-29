---
doc_class: Tutorial
microservice: consent-graph
persona: data-steward + integration-engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — Draft a DataSharingAgreement, activate, projection-stream, revoke

You will: draft a bilateral DataSharingAgreement between two tenants, offer + accept, subscribe to the projection topic from the grantee, watch live cross-tenant data flow, revoke the agreement, verify denial-on-read. Total time ≤ 75 minutes.

## Pre-requisites

- A tenant_class paid-tier+ consent-graph cell.
- Two test tenants: `drill-acme` (grantor) + `drill-partner-co` (grantee), both onboarded.
- A test Ontology entity `Order` defined in `drill-acme`.

## Step 1 — Draft a scope spec (≤ 10 min)

Create `acme-partner-scope.yaml`:

```yaml
entity: Order
field_narrowing:
  include:
    - order_id
    - total_amount_cents
    - currency
    - shipping_country_iso
    - created_at
    - status
  exclude:
    - customer_pii.email
    - customer_pii.phone
    - customer_pii.shipping_address.street_line1
    - customer_pii.shipping_address.street_line2
    - notes_internal
predicate_narrowing:
  expression: "status IN ('shipped', 'delivered')"
geographic_constraint: same-region
sharing_modes_permitted: [projection]
retention_at_grantee: 90_days_post_creation
purpose_specification: "Cross-partner shipment visibility; logistics planning + analytics."
```

The scope spec is the canonical contract. Field-narrowing is a allow-list (only fields in `include` are projected); predicate-narrowing filters rows; `same-region` forbids cross-border transfer.

## Step 2 — Draft + offer the agreement (≤ 10 min)

```sh
oya consent-graph agreement draft \
    --grantor drill-acme \
    --grantee drill-partner-co \
    --scope-spec ./acme-partner-scope.yaml \
    --sharing-mode projection \
    --window 2026-06-01..2027-05-31 \
    --grantor-data-steward drill-steward-a \
    --grantee-data-steward drill-steward-b \
    --output draft-agreement.json
```

The draft agreement is initially in `draft` state, visible only to the grantor.

```sh
oya consent-graph agreement offer \
    --agreement-id $(jq -r .agreement_id draft-agreement.json) \
    --notify-grantee true \
    --offer-message "Shipping data per the master-services-agreement section 4.2"
```

State transitions to `offered`; the grantee's data-steward receives notification.

## Step 3 — Accept the agreement (grantee side) (≤ 5 min)

Switch to grantee context:

```sh
oya auth login --tenant drill-partner-co --user drill-steward-b --role data-steward
```

Review the offered agreement:

```sh
oya consent-graph agreement list --tenant drill-partner-co --state offered
oya consent-graph agreement show --agreement-id ag-draft-2026-05-20
```

The grantee's data-steward reviews:

- Scope spec.
- Predicate narrowing.
- Geographic constraint.
- Retention requirement.
- Purpose specification.

Accept:

```sh
oya consent-graph agreement accept \
    --agreement-id ag-draft-2026-05-20 \
    --grantee-evidence ./acceptance-evidence.pdf
```

State transitions: `offered` → `accepted` → `active`. Audit chain emits both transitions (per-side).

## Step 4 — Subscribe to the projection stream (≤ 15 min)

The projection-gateway provisions a Pulsar topic for the grantee:

```sh
oya consent-graph projection topic-show \
    --agreement-id ag-draft-2026-05-20 \
    --grantee drill-partner-co
```

Output:

```yaml
projection_topic: persistent://drill-partner-co/consent-graph/ag-draft-2026-05-20-projections
schema_url: https://consent-graph.drill-syd-1.oyatie.local/schemas/ag-draft-2026-05-20-projections.json
grantee_pulsar_endpoint: pulsar+ssl://drill-partner-co.pulsar.drill-syd-1:6651
auth_mode: mtls-with-agreement-bound-jwt
```

Subscribe + watch:

```sh
oya consent-graph projection subscribe \
    --agreement-id ag-draft-2026-05-20 \
    --grantee drill-partner-co \
    --print-rows true
```

Now (in another terminal) emit a synthetic `Order` event from the grantor `drill-acme`:

```sh
oya synthetic emit-order \
    --tenant drill-acme \
    --order-id ord-12345 \
    --total-amount-cents 49900 \
    --currency USD \
    --shipping-country US \
    --customer-email "redacted@example.com" \
    --status shipped
```

Within ~ 500 ms, the projection subscriber should print:

```json
{
  "event_class": "projection_row_emitted",
  "agreement_id": "ag-draft-2026-05-20",
  "entity_type": "Order",
  "entity_id": "ord-12345",
  "fields": {
    "order_id": "ord-12345",
    "total_amount_cents": 49900,
    "currency": "USD",
    "shipping_country_iso": "US",
    "created_at": "2026-05-20T13:42:00Z",
    "status": "shipped"
  }
}
```

Note: `customer_pii.*` fields are EXCLUDED per the scope spec. The grantee never sees them.

## Step 5 — Test predicate narrowing (≤ 5 min)

Emit an Order with `status = pending` (should NOT project):

```sh
oya synthetic emit-order \
    --tenant drill-acme \
    --order-id ord-99999 \
    --total-amount-cents 99900 \
    --currency USD \
    --shipping-country US \
    --status pending
```

The subscriber should NOT print this row. Verify the predicate-narrowing dashboard:

```sh
oya consent-graph projection stats --agreement-id ag-draft-2026-05-20
```

Expected: `rows_projected: N`, `rows_filtered_by_predicate: 1` (the pending order).

## Step 6 — Test geographic constraint (≤ 5 min)

The agreement specifies `same-region`. If the grantor + grantee are in the SAME region (typical), projections flow. If the agreement is provisioned across regions, projections deny:

```sh
oya consent-graph projection stats \
    --agreement-id ag-draft-2026-05-20 \
    --field cross-border-blocked-rows
```

If `cross-border-blocked-rows > 0`, the dashboard explains which rows were blocked + why.

## Step 7 — Revoke the agreement (≤ 5 min)

```sh
oya consent-graph agreement revoke \
    --agreement-id ag-draft-2026-05-20 \
    --revoker drill-acme \
    --reason "Business relationship ended"
```

State transitions: `active` → `revoked`. Audit chain emits `agreement_revoked` from BOTH sides.

Watch the revocation propagation:

```sh
oya consent-graph drill revocation-status \
    --agreement-id ag-draft-2026-05-20
```

Expected propagation timeline:

- t=0: revoke command issued.
- t=200ms: Pulsar fanout to all 12 Cedar evaluator workers.
- t=300ms: All workers invalidate the local policy cache.
- t=500ms (p99): Next projection-read by the grantee → Cedar denies.

Test the denial. As the grantee, attempt to subscribe again:

```sh
oya consent-graph projection subscribe \
    --agreement-id ag-draft-2026-05-20 \
    --grantee drill-partner-co
```

Expected:

```
Error: agreement_revoked
The agreement ag-draft-2026-05-20 has been revoked.
No new projections are emitted.
Cedar policy: consent-graph-policy-v1 (deny on revoked agreement)
```

The grantee's HISTORICAL projection data is still on their side; the agreement specifies the retention (90 d) after which the grantee MUST delete or anonymise. The bilateral chain has a `grantee_post_revocation_deletion_attested` event that the grantee emits after deleting their copy.

## Step 8 — Audit-chain verification (≤ 10 min)

```sh
oya audit query --tenant drill-acme --since 2h --agreement-id ag-draft-2026-05-20
```

Expected events from BOTH sides:

- `agreement_drafted`
- `agreement_offered`
- `agreement_accepted` (grantee side)
- `agreement_activated`
- `projection_topic_provisioned`
- `projection_row_emitted` × N (one per qualifying row)
- `projection_row_filtered_by_predicate` × M
- `agreement_revoked` (both sides)
- `cedar_cache_invalidation_propagated`
- `projection_subscribe_denied_on_revoked` (the grantee's attempt after revocation)

Cross-verify the bilateral chain on both sides:

```sh
oya consent-graph chain verify \
    --agreement-id ag-draft-2026-05-20 \
    --side both
```

Expected: chains verified, signatures match, no gaps.

## What you've learned

- The 7-state DataSharingAgreement lifecycle.
- Scope narrowing + predicate narrowing + geographic constraint.
- The projection-gateway pattern: grantor's row never migrates physically.
- The real-time revocation budget + the Cedar cache invalidation propagation.
- The bilateral chain audit shape.

Next tutorial: `tutorials/cross-tenant-aggregate-with-k-anonymity.md` — author an Aggregate-mode agreement with k-anonymity threshold.
