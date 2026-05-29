---
ip_id: IP-014
microservice: cloud-billing
title: Cell-aware billing-data residency — per-ADR-0248 cellular topology
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0248, ADR-0244, ADR-0252, ADR-0263, ADR-0131]
counterpart_parity: [AWS regional billing aggregation, Stripe per-region routing, Azure ARM zone topology, GCP per-region projects]
capabilities_touched:
  - cap.cloud.billing.read_tenant_class
  - cap.cloud.billing.issue_invoice
  - cap.cloud.billing.emit_usage_event
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-014 — Cell-aware billing-data residency

## §A Objective

Document cloud-billing's behavior under the ADR-0248 Amazon-shape cellular architecture. Per ADR-0248 every µservice runs in Tier-0 through Tier-4 cells with shuffle-sharding for blast-radius limitation. cloud-billing's invoice + settlement + meter data is **cell-scoped** by default — a tenant's billing data lives in the same cell as the tenant's identity and resource fleet (per ADR-0244 tenant scoping × ADR-0248 cellular).

This means cross-cell queries are bounded; cross-cell failures don't cascade; per-region data residency rules (GDPR Art. 5, China PIPL, Russia Federal Law 242-FZ, Vietnam Decree 53/2022, India DPDP Act) are enforced at the cellular layer.

## §B Scope

In scope:

- Cell topology integration: cloud-billing as a per-cell µservice deployment.
- Tenant-to-cell binding: how a tenant_id maps to a home cell.
- Cross-cell read paths (e.g. global summary in finops-portal).
- Cross-cell failover (cell-loss disaster recovery).
- Per-region data residency enforcement.
- Cell-aware audit-chain emission (audit-chain Merkle trees are per-cell with cross-cell roots).
- Shuffle-sharding for per-tenant cell selection.

Out of scope:

- Cell substrate provisioning (cloud-iac µservice owns provisioning per ADR-0331).
- Shuffle-sharding algorithm internals (oya-shuffle-sharding crate per ADR-0333).
- Per-cell observability (observability µservice owns metrics).

## §C Architecture

### §C.1 Tier-0 through Tier-4 cell topology

Per ADR-0248:

- **Tier-0 cell**: Single tenant, fully isolated (FedRAMP High, K-FSI sovereign, BCBS-239).
- **Tier-1 cell**: Small tenant cohort (≤ 100 tenants), strong isolation.
- **Tier-2 cell**: Medium tenant cohort (100–1000 tenants), shared substrate.
- **Tier-3 cell**: Large tenant cohort (1000–10000 tenants), shared with shuffle-sharding.
- **Tier-4 cell**: Standard tenant cohort (10000+ tenants), maximum density.

cloud-billing is deployed per-cell. Each cell runs:

- One Kubernetes deployment of cloud-billing (Tier-3 cells = 3 replicas; Tier-0 = 5 replicas with affinity to cell-pinned nodes).
- One per-cell `cloud-billing-uel-buffer` (write-ahead log for outage tolerance per IP-010 §D.2).
- One per-cell connection pool to cloud-data (the canonical storage µservice).

### §C.2 Tenant-to-cell binding

A tenant is bound to a single **home cell** at creation time:

1. tenancy µservice receives signup request.
2. tenancy calls `cloud-iac` to select a home cell using the `oya-shuffle-sharding` algorithm (per ADR-0333):
   - Inputs: tenant_id (hashed), required_data_residency (from prospect's locale), tenant_class (demo_trial typically lands in Tier-3 Always-Free pool; paid lands in Tier-2 or Tier-1 per contract).
   - Output: `home_cell_id` (e.g. `cell-us-west-2-tier3-001`).
3. tenant_id is permanently associated with home_cell_id.
4. cloud-billing creates the BillingAccount in the home cell.

The `home_cell_id` is a tenant attribute readable via cloud-iam.

### §C.3 Cross-cell reads

For finops-portal global summary (e.g. "tenant's total cost across all cells where they have resources"):

1. finops-portal queries cloud-iam for the tenant's `home_cell_id`.
2. finops-portal calls cloud-billing in that cell via cross-cell gRPC.
3. If the tenant has multi-cell deployments (e.g. paid tenant with cell-replicated workload), each remote cell's cloud-billing returns the cell-local subset.
4. finops-portal aggregates the per-cell responses.

Cross-cell gRPC uses HTTP/3 over the cell-mesh substrate (per ADR-0253). Each cross-cell call carries the audit-chain seal hash; the aggregation is itself a Cedar-gated read (`cap.cloud.billing.read_invoice` x N).

### §C.4 Cross-cell failover

If a cell becomes unavailable (zone failure, regional outage):

1. cloud-iac detects via per-cell health metrics.
2. tenants in that cell are flagged `cell_unavailable = true`.
3. cloud-billing's read APIs (in unaffected cells) report cell-availability in their response metadata.
4. Write APIs for the affected cell fail closed (no cross-cell write).
5. Cell recovery is the responsibility of cloud-iac (per ADR-0331 per-µservice flat layout) + observability.

For tenants with multi-cell active-active deployment (paid Tier-1 / Tier-0):

1. cloud-iac promotes a secondary cell to primary.
2. cloud-billing in the secondary cell continues to receive usage events.
3. Settlement at month-end aggregates both primary + secondary cell records.

### §C.5 Per-region data residency

Per ADR-0244 tenant scoping + ADR-0248 cellular:

- Each cell is provisioned in exactly one region (e.g. `cell-us-west-2-tier3-001` runs in `us-west-2`).
- A tenant whose contract requires EU residency lands in a `eu-central-1` or `eu-west-1` cell.
- A tenant whose contract requires PIPL residency lands in a `cn-north-1` cell.
- cloud-billing's data — invoices, settlement statements, usage events, audit-chain entries — never crosses the cell boundary unless explicitly opted-in.

Cross-region replication for disaster recovery is:

- Disabled by default for sovereign-pack tenants (KR-CSAP, K-FSI, MAS-TRM, China PIPL).
- Configurable for global tenants (active-active multi-region, with explicit consent).

### §C.6 Shuffle-sharding for blast-radius

Per ADR-0333 oya-shuffle-sharding:

- A Tier-3 cell hosts ~1000–10000 tenants.
- Each tenant's data is shuffle-sharded across a subset of the cell's storage nodes (typically 2-of-N).
- A single storage node failure affects only ~1/N of tenants in the cell, not all.
- cloud-billing's per-tenant ledger is constructed on cell-local storage with shuffle-sharded redundancy.

### §C.7 Cell-aware audit-chain

audit-chain entries are per-cell:

- Each cell maintains its own Merkle tree.
- Cross-cell root hashes are aggregated to a global Merkle tree (per audit-chain µservice topology, not cloud-billing).
- An invoice issued in `cell-us-west-2-tier3-001` carries an audit_chain_hash that resolves through that cell's tree.
- Cross-cell audit query is supported by audit-chain (out of scope here).

### §C.8 Cell migration (rare)

If a tenant must move cells (e.g. CSAP certification means moving from a Tier-3 commercial cell to a Tier-0 sovereign cell):

1. Migration is a planned, audited operation initiated by oyatie-finance-operator + reviewer.
2. cloud-billing-cell-migration-worker exports the tenant's ledger from the source cell.
3. Imports into the destination cell with new audit-chain entries chained to the source.
4. Original source cell's data retained until retention floor exhausted (per IP-013).

## §D Lifecycle

### §D.1 Home cell selection at signup

1. tenancy receives signup.
2. cloud-iac selects home_cell_id via shuffle-sharding.
3. cloud-billing deploys account in that cell.
4. Tenant principal STS token carries home_cell_id.

### §D.2 Usage event emission (cell-local)

1. Phase-0/1/2 µservice in cell X emits `EmitUsageEvent` for a resource in cell X.
2. cloud-billing in cell X receives.
3. CloudBillingLedger insertion stays local.
4. audit-chain in cell X seals.

### §D.3 Cross-cell finops query

1. finops-portal in cell Y queries cloud-iam for `home_cell_id`.
2. finops-portal in cell Y calls cloud-billing in cell X via cross-cell gRPC.
3. Cedar evaluates `cap.cloud.billing.read_invoice` at cell X boundary.
4. Response returned.

### §D.4 Settlement across cells

For multi-cell tenants (rare):

1. cloud-billing-settlement-worker queries all cells containing tenant data.
2. Aggregates by tenant_id.
3. Computes SettlementStatement.
4. Statement persisted in tenant's home cell with cross-cell provenance hashes.

### §D.5 Failure modes

- Home cell unreachable → write deny; reads continue via secondary if active-active.
- Cross-cell call timeout → aggregation returns partial result with explicit `cell_unavailable` flag.
- Cell migration partial state → rollback via reverse-import; original cell records retained.

## §E Cedar Policy Bindings

cloud-billing's Cedar gates do not currently directly inspect `home_cell_id` — cell scoping is enforced by the cell-mesh substrate (cell-boundary interceptor). Future enhancement (REMEDIATION-NOTES item):

- `cap.cloud.billing.cell.deny_cross_cell_write` — explicit deny for cross-cell writes.
- `cap.cloud.billing.cell.permit_aggregation_read` — explicit permit for finops aggregation.

For now, cross-cell traffic is gated by the cell-mesh policy (out of scope here).

## §F Evidence

### §F.1 Source files (cell topology)

- IaC scaffold under `microservices/cloud-billing/iac/oyatie-public-cloud/` (cell-aware deployment plan).
- IaC `microservices/cloud-billing/iac/oci-guest/always-free/` (Tier-3 cell for demo_trial).
- proto3 `DeploymentContext` enum (cloud-billing.proto lines 33–41) signals cell context per RPC.

### §F.2 Cross-µservice integration

- cloud-iac µservice owns cell provisioning (per ADR-0331).
- oya-shuffle-sharding crate owns selection algorithm (per ADR-0333).
- cloud-iam principal STS carries home_cell_id claim.

### §F.3 ADR anchors

- ADR-0248 Amazon-shape cellular architecture (master).
- ADR-0244 tenant scoping (cell × tenant cross-product).
- ADR-0252 HLC default (cross-cell causal ordering).
- ADR-0263 audit-chain per cell with global roots.
- ADR-0131 per-µservice flat layout (cloud-billing as flat µservice).
- ADR-0333 oya-shuffle-sharding crate.

### §F.4 REMEDIATION-NOTES

- Cedar gates for cross-cell write deny not yet authored — planned in IP-014-extension after cell-mesh policy fragment publishes.
- Cell migration runbook not yet authored — planned in `microservices/cloud-billing/runbooks/cell-migration.md`.

## §G Counterpart parity

| Counterpart | Their cell/region model | Oyatie equivalent | Delta |
|---|---|---|---|
| AWS regional billing | Per-region billing tables + Cost & Usage Reports aggregated globally | Per-cell cloud-billing + cross-cell finops aggregation | AWS aggregates at the report layer; oyatie aggregates at the RPC layer (real-time). |
| Stripe per-region routing | Per-country API endpoints for residency (e.g. stripe.com.br for Brazil) | Per-cell deployments per region | Stripe routes by country; oyatie routes by cell (cell may host multiple countries). |
| Azure ARM zone topology | Resource groups scoped to regions; per-region billing | Cells scoped to regions; per-cell cloud-billing | Direct parity. |
| GCP per-region projects | Projects pinned to regions; per-project billing | Tenants pinned to cells; per-cell ledger | Direct parity. |
| AWS Outposts | On-prem AWS-region extension | on-prem deployment context + per-cell cloud-billing | Same shape; oyatie's on-prem is cell-equivalent. |
| Snowflake account-region binding | Account pinned to region; cross-region replication opt-in | Tenant pinned to cell; cross-cell replication opt-in for paid only | Direct parity. |
| Databricks workspace-region | Workspace pinned to region | Tenant pinned to cell | Direct parity. |

## §H Open questions

- Whether cell-migration should be exposed as a self-serve operation or require operator review. Current decision: operator-reviewed only (high risk of state drift); revisit if customer demand justifies self-serve.
- Whether cross-cell active-active should be default for paid Tier-1 tenants or opt-in. Current decision: opt-in — replication cost is non-trivial; tenants who require it sign explicit contract terms.
- Whether shuffle-sharding granularity should be at the tenant level or at the per-resource level inside the tenant. Current decision: tenant-level shuffle; per-resource shuffle is overkill for billing-data scale.
