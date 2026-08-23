---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  Blue/green for stateful migrations, schema changes, runtime cutovers, capability cutovers.
  Atomic switchover via traffic-shift (not deployment-swap). Up/down/dry-run/per-tenant/per-cell rollback per D14.
planned_enforcement_ref:
  - governance-rollback-evidence
  - governance-schema-migration
related_adrs: [ADR-0040, ADR-0045, ADR-0049, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Blue/Green Specification


## 1. Scope (when blue/green, not canary)

Blue/green is mandatory for:

1. **Schema migrations** (DDL on production data, [ADR-0045](../../decisions/ADR-0045-database-tier-strategy.md)).
2. **Runtime cutovers** (WASM substrate, agent runtime, KMS roots).
3. **Capability cutovers** (Foundry capability replacing a published predecessor irreversibly).
4. **Cross-region replication topology changes** ([ADR-0049](../../decisions/ADR-0049-cross-region-replication-and-residency.md)).
5. **KMS root rotation** (HSM-backed; [ADR-0043](../../decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md)).
6. **Message-broker / event-bus upgrades** ([ADR-0005](../../decisions/ADR-0005-eventing-backbone-outbox-pattern.md)).

Everything else defaults to canary (see [`progressive-delivery-strategy.md`](progressive-delivery-strategy.md)).

## 2. Atomic switchover via traffic-shift

Switchover is **traffic-shift**, not deployment-swap. The blue and green stacks run side-by-side; the mesh routes the cutover. This means:

- Rollback = re-shift traffic; no redeploy required (≤ 60 s).
- Soak period observable in real traffic, not synthetic.
- No DNS TTL games.

Traffic-shift is driven by Argo Rollouts BlueGreen strategy or by Flagger's pre-promotion gates, both invoking the mesh (Istio Ambient per [ADR-0044](../../decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md)) via `platform-traffic-shift-kernel` (NEW; adapter pattern).

## 3. Lifecycle

```
blue (current; serving 100%)
   ↓ deploy green alongside
green (deployed; serving 0%; receiving mirrored traffic per traffic-mirror-spec)
   ↓ smoke + diff + cohort-pinned canary on green (1% non-stable cohort)
green (smoke-passed)
   ↓ atomic traffic-shift
green (serving 100%; blue retained as standby)
   ↓ soak (24 h non-regulated; 7 d regulated, per ADR-0040)
green (soaked; blue removed)
```

## 4. Stateful migration choreography (DDL)

1. **Backward-compatible schema (additive only).** Add columns/tables; readers + writers tolerate both shapes. Deploy code that **writes both** old + new shape.
2. **Backfill.** Dual-write window + async backfill until 100% covered. Evidence: backfill-completeness query stored as D14 artefact.
3. **Cut readers to new shape.** Canary-gated read path flip. Old shape still written.
4. **Cut writers to new shape only.** New code stops dual-writing.
5. **Destructive teardown (separate release).** Drop old columns; **at least 7 days after writer cutover**.

Each step is its own release; **no step compresses two**. Planned advisory lane: `governance-schema-migration` (existing lane; extend).

## 5. Up / down / dry-run / per-tenant / per-cell rollback (D14)

Every blue/green release MUST emit signed evidence covering all five rollback modes:

| Mode | Definition | Evidence required |
|---|---|---|
| **Up** | Re-shift traffic to green if a rollback to blue revealed a green-only fix | Re-shift artefact + audit-chain entry |
| **Down** | Re-shift traffic to blue (default rollback) | Traffic-shift log + blue-state snapshot |
| **Dry-run** | Execute switchover in a non-prod cell with prod-shaped traffic | Mirror-diff report + smoke-test artefact |
| **Per-tenant** | Re-shift one tenant back to blue while others stay on green | Per-tenant routing rule + cohort intersect log |
| **Per-cell** | Re-shift one cell back to blue (default unit per [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md)) | Per-cell traffic-shift log |

All five emitted by `intelligence-evidence-kernel` and validated by `governance-rollback-evidence` (NEW; BLOCKER if unsigned).


## 6. Per-tenant blue/green (regulated)

Regulated tenants in the stable cohort may **stay on blue indefinitely** until per-vertical regulatory pack approves the green schema ([ADR-0034](../../decisions/ADR-0034-per-vertical-data-class-overrides.md)). Per-tenant traffic-shift honours cohort.

## 7. Cost

Blue/green for databases = 2× capacity during cutover. Budgeted per release; cost-review gate runs at PR time via `intelligence-cost-budget-kernel`.

## 8. Hyperscaler equivalents

AWS CodeDeploy Blue/Green (ECS/EC2/Lambda); Microsoft Azure Slot Swap (App Service); Oracle OCI compartment-cutover patterns; Google Cloud Run revisions (traffic-split). Argo Rollouts BlueGreen is the open equivalent.

## 9. Compliance gates

- `governance-rollback-evidence` (NEW; BLOCKER).
- `governance-schema-migration` (existing; extend).
- `governance-cohort-honor` (NEW; HIGH).

## 10. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — blue/green soak of ≥ M hours feeds the canary-100% gate in the staging → prod 5-gate verification.
