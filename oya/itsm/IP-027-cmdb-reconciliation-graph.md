# IP-027 ITSM cmdb-reconciliation-graph

Service: itsm
ChangeSet scope: microservices/itsm/IP-027-cmdb-reconciliation-graph.md
Benchmarks displaced: ServiceNow ITSM, Jira Service Management, BMC Helix ITSM, Ivanti Neurons, Freshservice
Binding ADRs: ADR-0105, ADR-0131, ADR-0244, ADR-0246, ADR-0258, ADR-0263, ADR-0314, ADR-0316, ADR-0321

## Objective
- Objective 001: Build a CMDB reconciliation graph that converts source-vendor CI and relation evidence into tenant-scoped canonical configuration graph state.
- Objective 002: Displace ServiceNow CMDB, Jira Assets, BMC Helix CMDB, Ivanti discovery, and Freshservice asset management without copying their object model.
- Objective 003: Keep discovery confidence, tenant scope, relation provenance, data class, and audit evidence attached to every graph node and edge.
- Objective 004: Prevent discovered assets from becoming authority for policy, ownership, or change execution.
- Objective 005: Produce deterministic graph writes, rejects, drift reports, replay results, and rollback plans.
- Objective 006: Make the graph implementation buildable from this IP plus referenced docs under documentation-rigor section 1.1.

## Graph domain
- Graph domain 001: Node type `ItsmConfigurationItem` represents a tenant-owned service, asset, endpoint, application, device, integration, or data store.
- Graph domain 002: Node type `ItsmServiceComponent` represents a logical service component that can receive incidents and changes.
- Graph domain 003: Node type `ItsmDiscoverySource` represents ServiceNow, Jira Assets, BMC Helix, Ivanti Neurons, Freshservice, native import, or manual operator input.
- Graph domain 004: Node type `ItsmOwnershipEvidence` records owner principal, group, delegated grant, confidence, and review state.
- Graph domain 005: Node type `ItsmRelationEvidence` records discovered, declared, verified, rejected, or rolled-back relation facts.
- Graph domain 006: Edge type `depends_on` connects configuration items with directional operational dependency.
- Graph domain 007: Edge type `hosted_on` connects application or service components to compute, cluster, or platform nodes.
- Graph domain 008: Edge type `monitored_by` connects CIs to observability resources and alert policies.
- Graph domain 009: Edge type `owned_by` connects CIs to owner evidence without granting mutation authority.
- Graph domain 010: Edge type `changed_by` connects CIs to change requests and implementation evidence.
- Graph domain 011: Edge type `affected_by_incident` connects CIs to incident tickets and major incident bridges.
- Graph domain 012: Edge type `related_problem` connects CIs to problem records and known-error evidence.

## Reconciliation inputs
- Input 001: Tenant context from IP-001 is mandatory for every graph write.
- Input 002: Cedar allow evidence from IP-002 is mandatory before any graph mutation.
- Input 003: Ontology object refs from IP-003 provide canonical object identity.
- Input 004: ITIL process normalizer output from IP-026 provides process family, data class, source kind, and mapping version.
- Input 005: ServiceNow CMDB payload supplies table, sys_id, class, relationship type, source, owner, and discovery timestamp.
- Input 006: Jira Assets payload supplies object schema, object type, key, relation, workspace, owner, and status.
- Input 007: BMC Helix CMDB payload supplies class id, instance id, reconciliation identity, dataset, relationship, and status reason.
- Input 008: Ivanti Neurons payload supplies device id, agent id, network identity, discovery confidence, software relation, and owner.
- Input 009: Freshservice asset payload supplies asset id, asset type, department, requester, owner, lifecycle state, and relation.
- Input 010: Native operator payload supplies declared relation, reviewer, tenant, and justification.

## Reconciliation algorithm
- Algorithm 001: Validate tenant context before reading any source payload field.
- Algorithm 002: Normalize source system kind through manifest-approved enum.
- Algorithm 003: Parse source CI candidate and map to canonical CI type.
- Algorithm 004: Parse source relation candidate and map to canonical edge type.
- Algorithm 005: Compute graph node id from tenant_id, canonical type, normalized local id, source system, and version.
- Algorithm 006: Compute graph edge id from tenant_id, edge type, source node id, target node id, and relation version.
- Algorithm 007: Reject any node candidate missing data_class or source provenance.
- Algorithm 008: Reject any edge candidate whose endpoints do not share tenant_id and home_cell_id.
- Algorithm 009: Reject any edge candidate whose confidence score is below policy threshold.
- Algorithm 010: Hold any edge candidate that would introduce a dependency cycle requiring operator review.
- Algorithm 011: Merge duplicate node candidates by deterministic id and record all source evidence refs.
- Algorithm 012: Merge duplicate edge candidates only when relation type, endpoints, tenant, and confidence policy match.
- Algorithm 013: Prefer verified operator evidence over lower-confidence discovery evidence without deleting discovery evidence.
- Algorithm 014: Preserve rejected relation evidence for audit and migration gap reports.
- Algorithm 015: Emit graph reconciliation audit started before storage write.
- Algorithm 016: Emit graph reconciliation completed, denied, held, drifted, or rolled_back after outcome.

## Vendor displacement behavior
- Vendor behavior 001: ServiceNow CMDB class names are mapped to canonical CI types and remain source metadata.
- Vendor behavior 002: ServiceNow relationship records are validated as edges rather than blindly imported.
- Vendor behavior 003: Jira Assets object schemas are mapped to canonical types and never create new ontology types automatically.
- Vendor behavior 004: Jira Assets workspace ids remain provenance and never become tenant ids.
- Vendor behavior 005: BMC Helix reconciliation identity maps to source evidence and never overwrites canonical id.
- Vendor behavior 006: BMC Helix datasets map to source confidence classes and do not bypass tenant validation.
- Vendor behavior 007: Ivanti Neurons discovery confidence becomes a graph confidence field and not a permit source.
- Vendor behavior 008: Ivanti Neurons device owner becomes ownership evidence and not mutation authority.
- Vendor behavior 009: Freshservice asset department becomes metadata and not tenant partition.
- Vendor behavior 010: Freshservice agent assignment becomes ownership evidence and not graph write authority.
- Vendor behavior 011: Native Oyatie operator declarations require reviewer evidence when they override vendor discovery.
- Vendor behavior 012: Unknown source relation types are held for review and never coerced into `depends_on`.

## Data model
- Data model 001: `cmdb_graph_node` stores tenant_id, node_id, canonical_type, data_class, source_system_kind, source_ref, confidence, and version.
- Data model 002: `cmdb_graph_edge` stores tenant_id, edge_id, edge_type, source_node_id, target_node_id, confidence, and version.
- Data model 003: `cmdb_graph_evidence` stores source payload digest, audit_preimage_id, policy_decision_id, reviewer, and outcome.
- Data model 004: `cmdb_graph_hold` stores held edge candidates with reason, reviewer queue, source refs, and expiry.
- Data model 005: `cmdb_graph_drift` stores previous graph digest, new graph digest, source kind, relation count delta, and severity.
- Data model 006: `cmdb_graph_rollback` stores rollback plan id, node refs, edge refs, audit event refs, and compensation outcome.
- Data model 007: Raw vendor payload storage is prohibited unless retention pack policy explicitly allows it.
- Data model 008: Tenant hash appears in metrics; raw tenant display name never appears in logs.

## Implementation sequence
- Implementation 001: Add canonical CMDB graph node and edge types in ITSM domain or kernel layer.
- Implementation 002: Add source mapper functions for ServiceNow CMDB, Jira Assets, BMC Helix CMDB, Ivanti Neurons, Freshservice, and native declarations.
- Implementation 003: Add graph id constructors with tenant, type, endpoints, source, and version.
- Implementation 004: Add confidence threshold policy lookup per data class and compliance pack.
- Implementation 005: Add endpoint tenant equality validation before edge construction.
- Implementation 006: Add duplicate merge behavior for node and edge candidates.
- Implementation 007: Add hold behavior for ambiguous, cyclic, or low-confidence graph candidates.
- Implementation 008: Add repository interface for idempotent node, edge, evidence, hold, drift, and rollback writes.
- Implementation 009: Add replay logic that compares graph digest and relation count.
- Implementation 010: Add rollback logic that tombstones graph writes while retaining evidence.
- Implementation 011: Add OpenAPI example for CMDB item and relation writes.
- Implementation 012: Add AsyncAPI messages for graph_reconciled, graph_denied, graph_held, graph_drifted, and graph_rolled_back.
- Implementation 013: Add metrics for candidate_count, accepted_edges, denied_edges, held_edges, drift_count, and rollback_count.
- Implementation 014: Add runbook links for relation drift, confidence gaps, and rollback failures.
- Implementation 015: Add dashboard panels for graph acceptance rate, relation confidence, held candidates, and source-vendor drift.

## Test matrix
- Test 001: Unit test maps ServiceNow CMDB CI payload to canonical graph node.
- Test 002: Unit test maps ServiceNow relationship payload to canonical graph edge.
- Test 003: Unit test maps Jira Assets object payload to canonical graph node.
- Test 004: Unit test maps Jira Assets relation payload to held candidate when type is unknown.
- Test 005: Unit test maps BMC Helix reconciliation identity to source evidence only.
- Test 006: Unit test maps Ivanti Neurons device payload with confidence threshold.
- Test 007: Unit test maps Freshservice asset payload without using department as tenant.
- Test 008: Negative test rejects edge endpoints from different tenants.
- Test 009: Negative test rejects edge endpoints from different home cells without replication policy.
- Test 010: Negative test holds cyclic dependency candidates.
- Test 011: Negative test rejects low-confidence relation writes.
- Test 012: Negative test rejects missing data_class.
- Test 013: Property test proves identical source refs in different tenants produce different node ids.
- Test 014: Property test proves edge ids are directional where edge type is directional.
- Test 015: Replay test proves identical payload digest produces identical graph digest.
- Test 016: Replay test detects graph drift when relation count changes unexpectedly.
- Test 017: Rollback test tombstones accepted edge while preserving graph evidence.
- Test 018: Contract test verifies `/itsm/v1/cmdb/items` returns ontology object ref and audit evidence.
- Test 019: Contract test verifies `/itsm/v1/cmdb/relations` returns denied evidence on endpoint mismatch.
- Test 020: Metrics test verifies source_system_kind, data_class, outcome, and tenant hash labels.

## Failure handling
- Failure 001: Missing tenant context halts reconciliation before payload inspection.
- Failure 002: Missing source-system kind returns unsupported mapping evidence.
- Failure 003: Missing data class returns validation error and no graph write.
- Failure 004: Unknown CI type returns held candidate, not guessed canonical type.
- Failure 005: Unknown relation type returns held candidate, not guessed edge.
- Failure 006: Confidence below threshold returns denied edge with source evidence.
- Failure 007: Endpoint mismatch returns denied edge and source endpoint refs.
- Failure 008: Audit-start failure prevents storage write.
- Failure 009: Storage write failure emits failed evidence and remediation hint.
- Failure 010: Audit-completion failure creates reconciliation gap for runbook repair.
- Failure 011: Replay drift failure halts mutation and emits drift report.
- Failure 012: Rollback failure emits rollback failure evidence and runbook pointer.

## Acceptance criteria
- Acceptance 001: An intern can implement graph node, graph edge, evidence, hold, drift, and rollback structures from this IP.
- Acceptance 002: An intern can implement mappers for ServiceNow, Jira Assets, BMC Helix, Ivanti Neurons, Freshservice, and native declarations.
- Acceptance 003: An intern can explain why discovery confidence is not authorization.
- Acceptance 004: An intern can explain why vendor CMDB class names are not canonical ontology types.
- Acceptance 005: An intern can implement tenant and home-cell endpoint equality checks.
- Acceptance 006: An intern can implement low-confidence denial and ambiguous relation hold behavior.
- Acceptance 007: An intern can implement deterministic replay and graph drift detection.
- Acceptance 008: An intern can implement rollback while preserving audit-chain evidence.
- Acceptance 009: An intern can produce contract examples, AsyncAPI events, metrics, dashboards, and runbook links.
- Acceptance 010: An intern can produce a PR that displaces vendor CMDB models without weakening tenant-scoped governance.

## Citations and authority trail
- Citation 001: docs/standards/documentation-rigor.md section 1.1 defines the intern-buildability bar.
- Citation 002: microservices/itsm/manifest.json defines benchmark roster, layer conformance, and compliance packs.
- Citation 003: microservices/itsm/PRD.md defines configuration-item and incident/change/problem relationships.
- Citation 004: microservices/itsm/contracts/openapi-v1.yaml defines CMDB action invocation expectations.
- Citation 005: microservices/itsm/runbooks/local-cmdb-relation-drift.md anchors graph drift remediation.
- Citation 006: microservices/itsm/slos/local-cmdb-relation-freshness.openslo.yaml anchors freshness and acceptance SLOs.
- Citation 007: ADR-0105 defines layer boundaries for graph domain, usecase, worker, adapter, REST, and governance code.
- Citation 008: ADR-0131 defines ontology graph projection expectations.
- Citation 009: ADR-0244 defines default-deny policy expectations before graph writes.
- Citation 010: ADR-0246 defines reusable mapper and library-first dispatch expectations.
- Citation 011: ADR-0258 defines graph contract versioning and replay compatibility.
- Citation 012: ADR-0263 defines audit-chain evidence for graph accepted, denied, held, drifted, and rollback outcomes.
- Citation 013: ADR-0316 prevents vendor CMDB products from becoming service or ontology boundaries.
- Citation 014: ADR-0321 defines B2B leader parity expectations for ITSM CMDB depth.

## Detailed build checklist
- Build checklist 001: Create fixture `servicenow_cmdb_ci_server_valid.json` with tenant, sys_id, class, owner, and relation source fields.
- Build checklist 002: Create fixture `servicenow_cmdb_relation_cross_tenant_denied.json` with mismatched endpoint tenant refs.
- Build checklist 003: Create fixture `jira_assets_application_valid.json` with workspace provenance and canonical service component mapping.
- Build checklist 004: Create fixture `jira_assets_unknown_relation_held.json` with relation type that lacks canonical mapping.
- Build checklist 005: Create fixture `bmc_helix_dataset_identity_valid.json` with reconciliation identity preserved as evidence.
- Build checklist 006: Create fixture `bmc_helix_dataset_identity_not_primary_key.json` proving canonical id remains tenant-scoped.
- Build checklist 007: Create fixture `ivanti_device_confidence_valid.json` with confidence above threshold.
- Build checklist 008: Create fixture `ivanti_relation_confidence_denied.json` with confidence below threshold.
- Build checklist 009: Create fixture `freshservice_asset_owner_valid.json` with owner recorded as evidence only.
- Build checklist 010: Create fixture `freshservice_department_not_tenant.json` proving department cannot partition data.
- Build checklist 011: Add canonical graph digest for one accepted ServiceNow relation.
- Build checklist 012: Add canonical graph digest for one accepted Ivanti relation.
- Build checklist 013: Add canonical graph digest for one held Jira Assets relation.
- Build checklist 014: Add canonical graph digest for one denied cross-tenant relation.
- Build checklist 015: Add graph repository fake that records nodes, edges, holds, drift reports, and rollback plans.
- Build checklist 016: Add graph service test that refuses writes before audit-start evidence exists.
- Build checklist 017: Add graph service test that writes completion evidence after node and edge persistence succeeds.
- Build checklist 018: Add graph service test that records storage failure as remediation evidence.
- Build checklist 019: Add worker test that processes reconciliation batches tenant-by-tenant.
- Build checklist 020: Add worker test that stops a batch when replay drift exceeds configured threshold.
- Build checklist 021: Add API test for CMDB item write accepted response.
- Build checklist 022: Add API test for CMDB relation endpoint mismatch denied response.
- Build checklist 023: Add API test for held relation response with reviewer queue id.
- Build checklist 024: Add AsyncAPI schema for `itsm.cmdb.graph_reconciled.v1`.
- Build checklist 025: Add AsyncAPI schema for `itsm.cmdb.graph_denied.v1`.
- Build checklist 026: Add AsyncAPI schema for `itsm.cmdb.graph_held.v1`.
- Build checklist 027: Add AsyncAPI schema for `itsm.cmdb.graph_drifted.v1`.
- Build checklist 028: Add AsyncAPI schema for `itsm.cmdb.graph_rolled_back.v1`.
- Build checklist 029: Add Grafana panel query for accepted relation rate by source vendor.
- Build checklist 030: Add Grafana panel query for held relation backlog by reason.
- Build checklist 031: Add Grafana panel query for confidence denial rate by source vendor.
- Build checklist 032: Add runbook link when held relation backlog breaches SLO.
- Build checklist 033: Add runbook link when replay drift appears after mapping version change.
- Build checklist 034: Add rollback test that preserves original audit event ids.
- Build checklist 035: Add documentation note that no manifest, journey, ADR, ERP, or other B2B leader file is touched by this IP.
- Build checklist 036: Add final verification command for line count and citation density under the batch-B report.
- Build checklist 037: Add PR summary line that names CMDB reconciliation graph as net-new IP-027.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/itsm/IP-027-cmdb-reconciliation-graph.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/itsm/IP-027-cmdb-reconciliation-graph.md`, `microservices/itsm/manifest.json`, `microservices/itsm/ARCHITECTURE.md`, `microservices/itsm/PRD.md`, `microservices/itsm/multi-region.md`, `microservices/itsm/capacity-model.md`].
