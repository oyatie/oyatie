---
doc_class: ImplementationPlan
ip_id: IP-021
microservice: treasury
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0319]
journey_id: j106-multi-currency-cross-border-payment
journey_link: docs/user-journeys/j106-multi-currency-cross-border-payment/story.md
status: Accepted
date: 2026-05-20
owner: axis-treasury
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [TRM-CM Bank Account Management, TRM-CM Cash Pool Structure, TRM-RM Bank Counterparty Risk]
---

# IP-021: Bank account hierarchy graph

## Intent
Implement a bank account hierarchy graph that models tenant bank structures, cash-pool membership, signer controls, regional ownership, and counterparty risk relationships.
The graph gives treasury a governed replacement for SAP bank account management hierarchy views and external signatory spreadsheets.
The graph becomes the account topology source for cash pooling, daily rollup, payment factory routing, and bank counterparty risk.
The implementation must support directed relationships with effective dates and evidence-backed approval.
The implementation must expose graph read APIs without allowing cross-tenant traversal.
The implementation must keep account secrets and credentials out of graph rows.
The implementation must emit audit events for node creation, edge creation, edge expiration, and policy denies.
The implementation must use Cedar hooks for relationship mutation and sensitive read paths.
The implementation must support migration from SAP house bank, account id, and bank master structures.
The implementation must remain buildable without a separate graph database in the first slice.

## Context
Why: treasury teams cannot safely operate cash pooling, payment routing, and account rationalization unless account hierarchy is explicit and queryable.
Why: SAP bank master and house bank account tables do not capture all operational relationships such as parent account, virtual account, pool member, signer set, and risk group.
Why: Oyatie needs account topology as a first-class domain model rather than repeated account relationship columns in every feature.
Journey leg: j106 bank-account rationalization validates signer and currency coverage before cross-border payment routing.
Named persona: Nora Lind, Treasury Controls Manager at NorthStream Industries, owns bank-account hierarchy governance.
Supporting persona: Dev Patel, Treasury Platform Engineer, needs graph queries for payment route and sweep plan services.
Pain point: signatory changes are tracked in bank portals but not connected to internal account ownership.
Pain point: virtual accounts and physical accounts are confused during cash rollups.
Pain point: bank counterparty exposure is calculated manually because account-to-bank group relationships are incomplete.
SAP parity: TRM-CM bank account management, house bank hierarchy, cash pool structures, and TRM-RM bank counterparty risk grouping.
Product outcome: a tenant can query account ancestors, descendants, pool memberships, signers, bank group exposure, and route-eligible accounts.
Non-goal: bank credential vaulting remains in secret management and bank connectivity.
Non-goal: user identity lifecycle remains in identity.
Non-goal: cash sweep execution remains in IP-016.
Invariant: graph edges are effective dated and never physically deleted.
Invariant: graph traversal is tenant-scoped and cell-local.
Invariant: no account can be active without at least one owner legal entity edge.
Invariant: signer edges require dual approval for high-risk accounts.
Acceptance anchor: an intern can implement relational graph tables, traversal queries, policies, APIs, and migration checks from this file.

## Data Model Deltas
Table `treasury.bank_account_node`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `bank_account_id UUID NOT NULL`.
Column `node_type TEXT NOT NULL CHECK (node_type IN ('PhysicalAccount','VirtualAccount','PoolHeader','BankGroup','LegalEntity','SignerGroup'))`.
Column `display_name TEXT NOT NULL`.
Column `country_code CHAR(2)`.
Column `currency CHAR(3)`.
Column `active BOOLEAN NOT NULL DEFAULT true`.
Column `risk_tier TEXT NOT NULL CHECK (risk_tier IN ('Low','Medium','High','Restricted'))`.
Column `created_by_principal_id UUID NOT NULL`.
Column `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
Constraint `UNIQUE (tenant_id, bank_account_id, node_type)`.
Index `ix_bank_account_node_tenant_type` on `(tenant_id, node_type, active)`.
Table `treasury.bank_account_graph_edge`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `from_node_id UUID NOT NULL REFERENCES treasury.bank_account_node(id)`.
Column `to_node_id UUID NOT NULL REFERENCES treasury.bank_account_node(id)`.
Column `edge_type TEXT NOT NULL CHECK (edge_type IN ('Owns','ParentOf','PoolMemberOf','SweepsTo','SignsFor','RiskGroupedUnder','RoutesThrough','Mirrors'))`.
Column `effective_from DATE NOT NULL`.
Column `effective_to DATE`.
Column `approval_status TEXT NOT NULL CHECK (approval_status IN ('Draft','PendingApproval','Approved','Expired','Rejected'))`.
Column `evidence_ref TEXT NOT NULL`.
Column `cedar_decision_id UUID NOT NULL`.
Column `created_by_principal_id UUID NOT NULL`.
Column `approved_by_principal_id UUID`.
Column `approved_at TIMESTAMPTZ`.
Constraint `UNIQUE (tenant_id, from_node_id, to_node_id, edge_type, effective_from)`.
Index `ix_bank_account_edge_from` on `(tenant_id, from_node_id, edge_type, effective_to)`.
Index `ix_bank_account_edge_to` on `(tenant_id, to_node_id, edge_type, effective_to)`.
Table `treasury.bank_account_graph_snapshot`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `as_of DATE NOT NULL`.
Column `node_count INTEGER NOT NULL`.
Column `edge_count INTEGER NOT NULL`.
Column `orphan_account_count INTEGER NOT NULL`.
Column `high_risk_signer_gap_count INTEGER NOT NULL`.
Column `evidence_hash TEXT NOT NULL`.
Column `computed_at TIMESTAMPTZ NOT NULL`.
Constraint `UNIQUE (tenant_id, as_of)`.
Table `treasury.bank_account_graph_violation`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `snapshot_id UUID NOT NULL REFERENCES treasury.bank_account_graph_snapshot(id)`.
Column `node_id UUID`.
Column `edge_id UUID`.
Column `severity TEXT NOT NULL CHECK (severity IN ('Info','Warning','Blocking'))`.
Column `code TEXT NOT NULL`.
Column `message TEXT NOT NULL`.
Column `resolved_at TIMESTAMPTZ`.
Storage rule: edges are append-only; expiration writes `effective_to` and status Expired through controlled API only.
Traversal rule: use recursive SQL CTEs with max depth 12 in first slice.
Retention rule: retain graph history and snapshots for ten years.

## API Endpoints
REST `POST /v1/treasury/bank-account-graph/nodes`.
Request example:
```json
{
  "bank_account_id": "0c8ed990-1111-4999-a000-111122223333",
  "node_type": "PhysicalAccount",
  "display_name": "Nordea SEK Operating 4102",
  "country_code": "SE",
  "currency": "SEK",
  "risk_tier": "High"
}
```
Response example:
```json
{
  "node_id": "3d6d69d9-2222-4c9c-b333-444455556666",
  "active": true,
  "audit_event_id": "TreasuryBankAccountGraphNodeCreated:..."
}
```
REST `POST /v1/treasury/bank-account-graph/edges`.
Edge request includes `from_node_id`, `to_node_id`, `edge_type`, `effective_from`, and `evidence_ref`.
Edge response returns edge id, approval status, and Cedar decision id.
REST `POST /v1/treasury/bank-account-graph/edges/{edge_id}/approve`.
REST `POST /v1/treasury/bank-account-graph/edges/{edge_id}/expire`.
REST `GET /v1/treasury/bank-account-graph/nodes/{node_id}/ancestors?edge_type=ParentOf&as_of=2026-05-20`.
REST `GET /v1/treasury/bank-account-graph/nodes/{node_id}/descendants?max_depth=6`.
REST `GET /v1/treasury/bank-account-graph/accounts/{bank_account_id}/topology`.
REST `POST /v1/treasury/bank-account-graph/snapshots`.
Snapshot response returns node count, edge count, violation count, and evidence hash.
gRPC `TreasuryBankAccountGraphService.GetTopology(GetTopologyRequest) returns (BankAccountTopology)`.
gRPC `TreasuryBankAccountGraphService.ValidateGraph(ValidateGraphRequest) returns (GraphValidationReport)`.
Error `409 EDGE_CYCLE_NOT_ALLOWED` when ParentOf or SweepsTo would create a forbidden cycle.
Error `403 BANK_ACCOUNT_GRAPH_SCOPE_DENIED` when principal lacks region or legal-entity scope.
Error `422 HIGH_RISK_SIGNER_DUAL_APPROVAL_REQUIRED` when approval rules are unmet.

## Cedar Policy Hooks
Principal shape: `User::{ id, tenant_id, roles, region_scope, legal_entity_scope, signer_admin_scope }`.
Action `Action::"create_bank_account_graph_node"`.
Action `Action::"create_bank_account_graph_edge"`.
Action `Action::"approve_bank_account_graph_edge"`.
Action `Action::"read_sensitive_bank_account_topology"`.
Resource `BankAccountGraphEdge::{ tenant_id, edge_type, from_risk_tier, to_risk_tier, country_code, legal_entity_id, approval_status }`.
Context `BankGraphContext::{ now, device_posture, request_purpose, creates_cycle, has_second_approver }`.
Permit treasury account admins to create nodes for tenant-scoped accounts.
Permit graph edge creation when principal region scope covers both endpoint nodes.
Forbid ParentOf and SweepsTo edges when `context.creates_cycle == true`.
Forbid approving own edge creation unless edge risk tier is Low.
Forbid approving high-risk SignsFor edge unless `context.has_second_approver == true`.
Forbid sensitive topology reads unless request purpose is `TreasuryOperations`, `Audit`, or `Risk`.
Emit `BankAccountGraphPolicyDenied` on deny.
Policy fixture `policy/bank-account-graph-cycle-deny.json`.
Policy fixture `policy/bank-account-graph-high-risk-signer-deny.json`.
Policy fixture `policy/bank-account-graph-region-scope-deny.json`.

## Ontology Projection
SAP house bank maps to `Oyatie::Treasury::BankAccountNode` with node type BankGroup or PhysicalAccount.
SAP house bank account maps to node type PhysicalAccount.
SAP bank master `BNKA` maps to BankGroup node.
SAP cash pool header maps to PoolHeader node.
SAP signatory workflow maps to SignerGroup and SignsFor edges when available.
Kyriba account structure maps to nodes and ParentOf edges.
GTreasury bank account hierarchy maps to nodes and RiskGroupedUnder edges.
Oracle bank account owner maps to Owns edge from legal entity to account.
Ontology field `BankAccountNode.displayName` maps from `display_name`.
Ontology field `BankAccountNode.riskTier` maps from `risk_tier`.
Ontology field `BankAccountEdge.edgeType` maps from `edge_type`.
Ontology field `BankAccountEdge.effectiveFrom` maps from `effective_from`.
Ontology field `BankAccountEdge.effectiveTo` maps from `effective_to`.
Ontology edge `LEGAL_ENTITY_OWNS_ACCOUNT` maps from Owns.
Ontology edge `ACCOUNT_PARENT_OF_ACCOUNT` maps from ParentOf.
Ontology edge `ACCOUNT_MEMBER_OF_POOL` maps from PoolMemberOf.
Ontology edge `ACCOUNT_SWEEPS_TO_ACCOUNT` maps from SweepsTo.
Ontology edge `SIGNER_GROUP_SIGNS_FOR_ACCOUNT` maps from SignsFor.
Ontology edge `ACCOUNT_RISK_GROUPED_UNDER_BANK` maps from RiskGroupedUnder.
Projection must exclude Draft and Rejected edges unless caller requests audit mode.

## Workflow Steps
Workflow `treasury.bank_account_graph.edge_create`.
Node `load_endpoint_nodes` validates tenant and active status.
Node `validate_edge_type_pair` verifies allowed node type combinations.
Node `detect_forbidden_cycle` runs recursive traversal for ParentOf and SweepsTo.
Node `cedar_create_edge_check` evaluates principal scope and cycle context.
Node `persist_edge_draft_or_pending` stores edge with approval status.
Node `emit_edge_created`.
Branch `low_risk_auto_approved` marks low-risk non-signer edges Approved.
Branch `high_risk_pending_approval` leaves edge PendingApproval.
Workflow `treasury.bank_account_graph.edge_approve`.
Node `reload_edge_for_update`.
Node `cedar_approve_edge_check`.
Node `validate_second_approver_if_required`.
Node `mark_edge_approved`.
Node `emit_edge_approved`.
Workflow `treasury.bank_account_graph.snapshot_validate`.
Node `load_active_nodes_and_edges`.
Node `detect_orphan_accounts`.
Node `detect_missing_owner_edges`.
Node `detect_high_risk_signer_gaps`.
Node `detect_sweep_cycles`.
Node `compute_graph_evidence_hash`.
Node `persist_snapshot_and_violations`.
Node `emit_snapshot_validated`.

## Audit Events
Audit event class `TreasuryBankAccountGraphNodeCreated`.
Audit event class `TreasuryBankAccountGraphNodeUpdated`.
Audit event class `TreasuryBankAccountGraphEdgeCreated`.
Audit event class `TreasuryBankAccountGraphEdgeApproved`.
Audit event class `TreasuryBankAccountGraphEdgeRejected`.
Audit event class `TreasuryBankAccountGraphEdgeExpired`.
Audit event class `TreasuryBankAccountGraphSnapshotComputed`.
Audit event class `TreasuryBankAccountGraphViolationRaised`.
Audit event class `TreasuryBankAccountGraphViolationResolved`.
Audit event class `TreasuryBankAccountGraphPolicyDenied`.
Audit payload must include tenant id, node id or edge id, edge type, effective dates, and evidence ref.
Audit payload for sensitive reads must include request purpose and principal id.
Audit payload for policy denies must include Cedar decision id and deny reason.
Audit retention class is `TreasuryBankAccountGovernance`.
Audit ordering key is `tenant_id:bank_account_id:edge_type`.

## SLO Targets
p50 topology read latency for depth 6 traversal: 60 ms.
p95 topology read latency for depth 6 traversal: 180 ms.
p99 topology read latency for depth 6 traversal: 350 ms.
p50 edge create latency: 90 ms.
p95 edge create latency: 300 ms.
p99 edge create latency: 700 ms.
p50 graph validation snapshot for 10000 nodes and 50000 edges: 900 ms.
p95 graph validation snapshot for 10000 nodes and 50000 edges: 3500 ms.
p99 graph validation snapshot for 10000 nodes and 50000 edges: 7000 ms.
Throughput target: 1000 topology reads per minute per cell.
Throughput target: 500 edge mutations per minute per cell.
Availability target for topology reads: 99.99 percent monthly.
Availability target for graph mutations: 99.95 percent monthly.
Rationale: payment routing and cash pooling are read-heavy and need stable low-latency traversals.
Rationale: validation can be slower because it is scheduled and operator-triggered.

## Failure Modes + Recovery
Failure `EDGE_CYCLE_NOT_ALLOWED`: detect recursive traversal; recover by rejecting edge and showing cycle path.
Failure `MISSING_OWNER_EDGE`: detect validation snapshot; recover by routing to account admin.
Failure `HIGH_RISK_SIGNER_GAP`: detect no approved SignsFor edge; recover by adding signer group and dual approval.
Failure `STALE_TOPOLOGY_CACHE`: detect evidence hash mismatch; recover by invalidating cache and re-reading.
Failure `CROSS_TENANT_NODE_REFERENCE`: detect tenant mismatch; recover by hard rejecting request and security audit.
Failure `EDGE_APPROVAL_SELF_APPROVAL`: detect creator equals approver; recover by assigning second approver.
Failure `GRAPH_SNAPSHOT_WRITE_CONFLICT`: detect unique key conflict; recover by returning existing snapshot for same as-of.
Failure `AUDIT_APPEND_FAILED`: detect audit-chain error; recover by aborting mutation.
Failure `MIGRATION_ORPHAN_ACCOUNT`: detect migration dry run; recover by creating legal entity or excluding account.
Failure `TRAVERSAL_DEPTH_EXCEEDED`: detect max depth guard; recover by validation ticket for hierarchy cleanup.
Recovery worker `treasury.bank_account_graph.cache_refresh` rebuilds topology cache after approved edge changes.
Runbook entry `runbooks/bank-account-graph-governance-failure.md` should describe signer gaps, cycles, and cache refresh.

## Migration Notes
Source vendor surface: SAP house bank master.
Source vendor surface: SAP house bank account.
Source vendor surface: SAP bank master table `BNKA`.
Source vendor surface: SAP cash pool configuration.
Source vendor surface: SAP signatory extracts when available.
Source vendor surface: Kyriba bank account hierarchy.
Source vendor surface: GTreasury account administration.
Source vendor surface: Oracle Cash Management bank account owners.
Migration maps SAP company code to LegalEntity node and Owns edge.
Migration maps house bank id to BankGroup node.
Migration maps house bank account id to PhysicalAccount node.
Migration maps virtual account identifiers to VirtualAccount nodes when present.
Migration maps cash pool member relation to PoolMemberOf edge.
Migration imports unknown signers as SignerGroup nodes with PendingApproval status.
Migration dry-run report lists accounts without owners, currencies, or country codes.
Migration acceptance requires no blocking orphan accounts for active migrated accounts.

## Cross-microservice Handoffs
Handoff to `identity`: resolve signer principals and signer groups.
Handoff to `legal-entity`: resolve legal entity ownership and regional scope.
Handoff to `payments`: provide route-eligible source accounts and signer controls.
Handoff to `cash-position`: provide account hierarchy and pool grouping.
Handoff to `cash-pooling`: provide PoolMemberOf and SweepsTo relationships.
Handoff to `risk`: provide bank group exposure topology.
Handoff to `workflow`: execute approval and validation workflows.
Handoff to `ontology`: publish graph nodes and typed edges.
Handoff to `audit-chain`: seal topology mutations, sensitive reads, and validation snapshots.
Handoff to `ops-dashboard`: expose orphan accounts, signer gaps, and cycle violations.

## Build Notes
Add database migration for node, edge, snapshot, and violation tables.
Add recursive SQL traversal repository with max depth parameter.
Add domain service `BankAccountHierarchyGraphService`.
Add cycle detection tests for ParentOf and SweepsTo.
Add Cedar entity schema for graph edge and graph context.
Add REST handlers for nodes, edges, approvals, expiration, topology, and snapshots.
Add gRPC handlers for topology and validation.
Add contract tests for cross-tenant node reference rejection.
Add workflow tests for high-risk signer dual approval.
Add load fixture with 10000 nodes and 50000 edges.
Add migration fixture with SAP house bank and cash pool export.
Add dashboard panels for topology read latency, orphan count, signer gap count, and policy denies.
