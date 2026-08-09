# PRD: cell-lifecycle microservice

Status: Wave 15-ZD scaffold, authoring-only; Rust implementation is deferred to Wave 15-ZD-impl.
Bounded context: cell-logical-state-machine.
Primary authority: ADR-0276 D-3, ADR-0266, ADR-0204, ADR-0276, ADR-0099.
Service purpose: own logical cell lifecycle transitions without owning infrastructure provisioning, tenant migration, or routing.

## 1. Product Boundary
001. cell-lifecycle is the authority for the logical Cell aggregate and the state machine that moves a cell from Registered through activation, promotion, drain, and decommission.
002. The service does not create Kubernetes clusters, OpenTofu modules, nodepools, subnets, or storage; that responsibility remains with cloud-iac.
003. The service does not migrate tenants between cells; it asks cell-rebalancer to drain residents and accepts tenancy resident-count evidence before decommission.
004. The service does not route traffic; api-gateway interprets cell status and owns request routing decisions.
005. The service consumes evidence packs and promotion-gate telemetry rather than inventing telemetry or auditing rules inside the lifecycle command path.
006. The service applies Cedar decisions as a hard gate before privileged transitions and records the Cedar decision id into LifecycleHistory.
007. The service handles both demo_trial and paid tenant classes because ADR-0266 requires both tenant classes in promotion evidence.
008. The service is a Tier 0 substrate registry per ADR-0204 because a wrong lifecycle transition can widen blast radius across many downstream workloads.

## 2. State Machine Contract
### 2.1 State: Registered
- Meaning: cell identity exists, cloud-iac has provisioned or reserved infrastructure, no production traffic is admitted.
- Storage invariant: the Cell aggregate stores state=Registered, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Registered require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Registered keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Registered refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Registered records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Registered emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Registered emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Registered emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.2 State: Activated
- Meaning: cell is eligible for low-risk placement checks, telemetry is flowing, and baseline compliance evidence exists.
- Storage invariant: the Cell aggregate stores state=Activated, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Activated require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Activated keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Activated refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Activated records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Activated emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Activated emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Activated emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.3 State: Promoted-T4
- Meaning: cell is permitted for Tier 4 best-effort or edge-class residency after evidence validation.
- Storage invariant: the Cell aggregate stores state=Promoted-T4, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Promoted-T4 require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Promoted-T4 keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Promoted-T4 refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Promoted-T4 records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Promoted-T4 emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Promoted-T4 emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Promoted-T4 emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.4 State: Promoted-T3
- Meaning: cell is permitted for Tier 3 application-class residency after warm-soak and quiet-window gates.
- Storage invariant: the Cell aggregate stores state=Promoted-T3, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Promoted-T3 require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Promoted-T3 keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Promoted-T3 refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Promoted-T3 records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Promoted-T3 emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Promoted-T3 emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Promoted-T3 emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.5 State: Promoted-T2
- Meaning: cell is permitted for Tier 2 capability-class residency with stronger canary and mesh evidence.
- Storage invariant: the Cell aggregate stores state=Promoted-T2, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Promoted-T2 require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Promoted-T2 keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Promoted-T2 refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Promoted-T2 records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Promoted-T2 emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Promoted-T2 emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Promoted-T2 emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.6 State: Promoted-T1
- Meaning: cell is permitted for Tier 1 substrate residency and receives Kata runtime scrutiny.
- Storage invariant: the Cell aggregate stores state=Promoted-T1, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Promoted-T1 require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Promoted-T1 keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Promoted-T1 refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Promoted-T1 records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Promoted-T1 emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Promoted-T1 emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Promoted-T1 emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.7 State: Promoted-T0
- Meaning: cell is permitted for Tier 0 foundation residency and highest blast-radius isolation.
- Storage invariant: the Cell aggregate stores state=Promoted-T0, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Promoted-T0 require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Promoted-T0 keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Promoted-T0 refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Promoted-T0 records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Promoted-T0 emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Promoted-T0 emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Promoted-T0 emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.8 State: Draining
- Meaning: new placement is refused, tenant migration is delegated to cell-rebalancer, and resident count trends toward zero.
- Storage invariant: the Cell aggregate stores state=Draining, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Draining require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Draining keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Draining refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Draining records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Draining emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Draining emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Draining emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.
### 2.9 State: Decommissioned
- Meaning: cell has zero residents, no active routes, sealed audit history, and no new lifecycle transitions.
- Storage invariant: the Cell aggregate stores state=Decommissioned, an HLC update timestamp, a monotonic lifecycle_version, and the latest audit_chain_event_id.
- Authorization invariant: transitions entering or leaving Decommissioned require a Cedar decision id unless the state is Decommissioned and read-only.
- Evidence invariant: the LifecycleHistory row for Decommissioned keeps evidence_pack_id, gate_snapshot_sha256, and request_id for replay.
- Blast-radius invariant: operations in Decommissioned refuse to expand placement beyond the state's ADR-0204 tier permit.
- Tenant-class invariant: evidence for Decommissioned records demo_trial and paid applicability even when one class has zero current residents.
- Observability invariant: every transition touching Decommissioned emits oya_cell_lifecycle_transition_total with from_state, to_state, result, and tier labels.
- Audit invariant: every accepted transition into Decommissioned emits audit-chain event cell.lifecycle.transition.accepted before the response is returned.
- Rejection invariant: every refused transition involving Decommissioned emits cell.lifecycle.transition.rejected with refusal_code and cedar_decision_id when available.

## 3. Allowed Transitions
### 3.1 Registered -> Activated
- Trigger: activate command with cloud-iac readiness receipt and telemetry bootstrap evidence.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Registered; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::activated for the caller and the Cell resource.
- Audit: pre_state=Registered, post_state=Activated, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Activated, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.2 Activated -> Promoted-T4
- Trigger: promote command with Tier 4 gate evidence pack.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Activated; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::promoted_t4 for the caller and the Cell resource.
- Audit: pre_state=Activated, post_state=Promoted-T4, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Promoted-T4, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.3 Promoted-T4 -> Promoted-T3
- Trigger: promote command with Tier 3 warm-soak, canary, mesh, tenant-class, and pack evidence.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T4; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::promoted_t3 for the caller and the Cell resource.
- Audit: pre_state=Promoted-T4, post_state=Promoted-T3, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Promoted-T3, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.4 Promoted-T3 -> Promoted-T2
- Trigger: promote command with Tier 2 evidence and no alert burst during quiet window.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T3; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::promoted_t2 for the caller and the Cell resource.
- Audit: pre_state=Promoted-T3, post_state=Promoted-T2, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Promoted-T2, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.5 Promoted-T2 -> Promoted-T1
- Trigger: promote command with substrate isolation, pack coverage, and Cedar promotion permit.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T2; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::promoted_t1 for the caller and the Cell resource.
- Audit: pre_state=Promoted-T2, post_state=Promoted-T1, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Promoted-T1, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.6 Promoted-T1 -> Promoted-T0
- Trigger: promote command with foundation-cell evidence and council-grade authorization.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T1; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::promoted_t0 for the caller and the Cell resource.
- Audit: pre_state=Promoted-T1, post_state=Promoted-T0, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Promoted-T0, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.7 Promoted-T0 -> Draining
- Trigger: drain command on planned retirement, critical hardware failure, or blast-radius containment.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T0; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::draining for the caller and the Cell resource.
- Audit: pre_state=Promoted-T0, post_state=Draining, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Draining, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.8 Promoted-T1 -> Draining
- Trigger: drain command on failure, compliance withdrawal, or manual SRE action.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T1; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::draining for the caller and the Cell resource.
- Audit: pre_state=Promoted-T1, post_state=Draining, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Draining, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.9 Promoted-T2 -> Draining
- Trigger: drain command on load-skew, failure, or tenant-safety decision.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T2; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::draining for the caller and the Cell resource.
- Audit: pre_state=Promoted-T2, post_state=Draining, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Draining, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.10 Promoted-T3 -> Draining
- Trigger: drain command on load-skew, failure, or promotion rollback.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T3; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::draining for the caller and the Cell resource.
- Audit: pre_state=Promoted-T3, post_state=Draining, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Draining, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.11 Promoted-T4 -> Draining
- Trigger: drain command on hardware failure or placement withdrawal.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Promoted-T4; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::draining for the caller and the Cell resource.
- Audit: pre_state=Promoted-T4, post_state=Draining, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Draining, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.12 Activated -> Draining
- Trigger: drain command before production promotion if readiness evidence is invalidated.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Activated; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::draining for the caller and the Cell resource.
- Audit: pre_state=Activated, post_state=Draining, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Draining, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.
### 3.13 Draining -> Decommissioned
- Trigger: decommission command after tenancy reports zero residents and audit-chain seals final history.
- Command idempotency: repeated command with the same idempotency_key returns the original transition result and does not create another history row.
- Guard: current state must equal Draining; any stale caller view returns 409 Conflict with latest lifecycle_version.
- Evidence: request must carry evidence_pack_id or an explicit empty-evidence reason accepted only for Registered creation.
- Cedar: the policy decision must authorize action lifecycle::decommissioned for the caller and the Cell resource.
- Audit: pre_state=Draining, post_state=Decommissioned, cell_id, tenant_class_scope, HLC timestamp, and audit_chain_seal are emitted per ADR-0217.
- Rollback posture: rollback is a new explicit transition or emergency drain, never mutation of existing LifecycleHistory.
- Dependency posture: cloud-iac, cell-rebalancer, tenancy, observability, audit-chain, and policy-cedar remain external adapters.
- Success response: includes cell_id, state=Decommissioned, lifecycle_version, audit_chain_event_id, and the carrier triplet version value.

## 4. Personas and Jobs
### 4.1 Persona: ops-cellular
- Description: human operations owner performing manual promote, rollback, emergency drain, and decommission commands.
- Primary job 1: inspect lifecycle history and confirm the latest state with audit-chain evidence before acting.
- Primary job 2: submit promote, drain, rollback, or decommission requests with an idempotency key and evidence pack reference.
- Primary job 3: resolve rejection reasons by collecting missing gate evidence rather than overriding the state machine.
- Needs: deterministic error codes, current state, missing evidence list, Cedar refusal reason, and next legal transitions.
- Must not: provision infrastructure, bypass cell-rebalancer migration, mutate api-gateway routes, or edit history rows directly.
- Evidence obligation: every manual action by ops-cellular carries a human or automation identity, incident id when present, and permit id.
- UX expectation: operations tools can list cells by state, tier, region, compliance pack, and stale evidence age.
### 4.2 Persona: foundry-cell-orchestrator
- Description: automation principal oyatie.foundry.cell-lifecycle that may prepare transition proposals but may not bypass permits.
- Primary job 1: inspect lifecycle history and confirm the latest state with audit-chain evidence before acting.
- Primary job 2: submit promote, drain, rollback, or decommission requests with an idempotency key and evidence pack reference.
- Primary job 3: resolve rejection reasons by collecting missing gate evidence rather than overriding the state machine.
- Needs: deterministic error codes, current state, missing evidence list, Cedar refusal reason, and next legal transitions.
- Must not: provision infrastructure, bypass cell-rebalancer migration, mutate api-gateway routes, or edit history rows directly.
- Evidence obligation: every manual action by foundry-cell-orchestrator carries a human or automation identity, incident id when present, and permit id.
- UX expectation: operations tools can list cells by state, tier, region, compliance pack, and stale evidence age.
### 4.3 Persona: cellular SRE
- Description: on-call engineer accountable for incident triage, SLO burn interpretation, and evidence-pack completeness.
- Primary job 1: inspect lifecycle history and confirm the latest state with audit-chain evidence before acting.
- Primary job 2: submit promote, drain, rollback, or decommission requests with an idempotency key and evidence pack reference.
- Primary job 3: resolve rejection reasons by collecting missing gate evidence rather than overriding the state machine.
- Needs: deterministic error codes, current state, missing evidence list, Cedar refusal reason, and next legal transitions.
- Must not: provision infrastructure, bypass cell-rebalancer migration, mutate api-gateway routes, or edit history rows directly.
- Evidence obligation: every manual action by cellular SRE carries a human or automation identity, incident id when present, and permit id.
- UX expectation: operations tools can list cells by state, tier, region, compliance pack, and stale evidence age.

## 5. Trigger Catalog
### 5.1 Trigger: ops manual promote
- Source: ops-cellular requests promotion after observing full ADR-0266 gate coverage and a complete evidence pack.
- Required request metadata: request_id, idempotency_key, principal, reason_code, evidence_pack_id, and tenant_class_scope.
- Validation sequence: current state check, dependency receipt check, Cedar authorization, gate/evidence validation, audit-chain precommit, state write, event emission.
- Failure result: no partial state change is visible; a rejected history row is optional but accepted history rows are never rewritten.
- Operator feedback: response names the first failed gate and the complete missing-evidence vector.
- Automation feedback: response includes retry_after_seconds only when the failure is caused by dependency lag rather than policy refusal.
### 5.2 Trigger: evidence-pack-availability check
- Source: foundry-cell-orchestrator polls or receives a signal that a sealed ADR-0217 evidence pack is ready.
- Required request metadata: request_id, idempotency_key, principal, reason_code, evidence_pack_id, and tenant_class_scope.
- Validation sequence: current state check, dependency receipt check, Cedar authorization, gate/evidence validation, audit-chain precommit, state write, event emission.
- Failure result: no partial state change is visible; a rejected history row is optional but accepted history rows are never rewritten.
- Operator feedback: response names the first failed gate and the complete missing-evidence vector.
- Automation feedback: response includes retry_after_seconds only when the failure is caused by dependency lag rather than policy refusal.
### 5.3 Trigger: drain on hardware failure
- Source: cellular SRE starts drain after cloud-iac, observability, or hardware telemetry identifies a cell-level risk.
- Required request metadata: request_id, idempotency_key, principal, reason_code, evidence_pack_id, and tenant_class_scope.
- Validation sequence: current state check, dependency receipt check, Cedar authorization, gate/evidence validation, audit-chain precommit, state write, event emission.
- Failure result: no partial state change is visible; a rejected history row is optional but accepted history rows are never rewritten.
- Operator feedback: response names the first failed gate and the complete missing-evidence vector.
- Automation feedback: response includes retry_after_seconds only when the failure is caused by dependency lag rather than policy refusal.
### 5.4 Trigger: compliance pack withdrawal
- Source: compliance pack validity changes and promotion tier no longer satisfies ADR-0207 certification invariants.
- Required request metadata: request_id, idempotency_key, principal, reason_code, evidence_pack_id, and tenant_class_scope.
- Validation sequence: current state check, dependency receipt check, Cedar authorization, gate/evidence validation, audit-chain precommit, state write, event emission.
- Failure result: no partial state change is visible; a rejected history row is optional but accepted history rows are never rewritten.
- Operator feedback: response names the first failed gate and the complete missing-evidence vector.
- Automation feedback: response includes retry_after_seconds only when the failure is caused by dependency lag rather than policy refusal.
### 5.5 Trigger: promotion rollback
- Source: SRE observes regression after promotion and initiates drain or demotion-safe containment according to runbook.
- Required request metadata: request_id, idempotency_key, principal, reason_code, evidence_pack_id, and tenant_class_scope.
- Validation sequence: current state check, dependency receipt check, Cedar authorization, gate/evidence validation, audit-chain precommit, state write, event emission.
- Failure result: no partial state change is visible; a rejected history row is optional but accepted history rows are never rewritten.
- Operator feedback: response names the first failed gate and the complete missing-evidence vector.
- Automation feedback: response includes retry_after_seconds only when the failure is caused by dependency lag rather than policy refusal.
### 5.6 Trigger: planned decommission
- Source: cloud-iac or ops retires capacity and cell-rebalancer confirms every resident tenant has moved.
- Required request metadata: request_id, idempotency_key, principal, reason_code, evidence_pack_id, and tenant_class_scope.
- Validation sequence: current state check, dependency receipt check, Cedar authorization, gate/evidence validation, audit-chain precommit, state write, event emission.
- Failure result: no partial state change is visible; a rejected history row is optional but accepted history rows are never rewritten.
- Operator feedback: response names the first failed gate and the complete missing-evidence vector.
- Automation feedback: response includes retry_after_seconds only when the failure is caused by dependency lag rather than policy refusal.

## 6. Out of Scope and Delegated Responsibilities
### 6.1 Delegated surface: cloud-iac
- Delegation: owns infrastructure provisioning and returns readiness receipts; cell-lifecycle never provisions infrastructure.
- Contract stance: cell-lifecycle calls cloud-iac through a port and persists only the receipt or evidence digest needed for lifecycle history.
- Non-goal: no cloud-iac implementation detail, retry policy beyond port semantics, or direct datastore ownership is embedded in cell-lifecycle domain logic.
- Failure posture: if cloud-iac is unavailable, lifecycle transition returns dependency_unavailable and leaves Cell state unchanged.
- Audit posture: dependency receipt digests are included in the audit-chain evidence envelope; raw dependency payloads stay with cloud-iac.
### 6.2 Delegated surface: cell-rebalancer
- Delegation: owns tenant migration and drain execution; cell-lifecycle issues drain intent and waits for resident-count convergence.
- Contract stance: cell-lifecycle calls cell-rebalancer through a port and persists only the receipt or evidence digest needed for lifecycle history.
- Non-goal: no cell-rebalancer implementation detail, retry policy beyond port semantics, or direct datastore ownership is embedded in cell-lifecycle domain logic.
- Failure posture: if cell-rebalancer is unavailable, lifecycle transition returns dependency_unavailable and leaves Cell state unchanged.
- Audit posture: dependency receipt digests are included in the audit-chain evidence envelope; raw dependency payloads stay with cell-rebalancer.
### 6.3 Delegated surface: tenancy
- Delegation: owns resident count, tenant-class coverage, placement registry, and resident zero proof.
- Contract stance: cell-lifecycle calls tenancy through a port and persists only the receipt or evidence digest needed for lifecycle history.
- Non-goal: no tenancy implementation detail, retry policy beyond port semantics, or direct datastore ownership is embedded in cell-lifecycle domain logic.
- Failure posture: if tenancy is unavailable, lifecycle transition returns dependency_unavailable and leaves Cell state unchanged.
- Audit posture: dependency receipt digests are included in the audit-chain evidence envelope; raw dependency payloads stay with tenancy.
### 6.4 Delegated surface: observability
- Delegation: owns SLO, canary, mesh, latency, and transition metrics used by promotion gates.
- Contract stance: cell-lifecycle calls observability through a port and persists only the receipt or evidence digest needed for lifecycle history.
- Non-goal: no observability implementation detail, retry policy beyond port semantics, or direct datastore ownership is embedded in cell-lifecycle domain logic.
- Failure posture: if observability is unavailable, lifecycle transition returns dependency_unavailable and leaves Cell state unchanged.
- Audit posture: dependency receipt digests are included in the audit-chain evidence envelope; raw dependency payloads stay with observability.
### 6.5 Delegated surface: audit-chain
- Delegation: owns tamper-evident event emission and evidence pack seals.
- Contract stance: cell-lifecycle calls audit-chain through a port and persists only the receipt or evidence digest needed for lifecycle history.
- Non-goal: no audit-chain implementation detail, retry policy beyond port semantics, or direct datastore ownership is embedded in cell-lifecycle domain logic.
- Failure posture: if audit-chain is unavailable, lifecycle transition returns dependency_unavailable and leaves Cell state unchanged.
- Audit posture: dependency receipt digests are included in the audit-chain evidence envelope; raw dependency payloads stay with audit-chain.
### 6.6 Delegated surface: policy-cedar
- Delegation: owns Cedar evaluation and permit validation for privileged transitions.
- Contract stance: cell-lifecycle calls policy-cedar through a port and persists only the receipt or evidence digest needed for lifecycle history.
- Non-goal: no policy-cedar implementation detail, retry policy beyond port semantics, or direct datastore ownership is embedded in cell-lifecycle domain logic.
- Failure posture: if policy-cedar is unavailable, lifecycle transition returns dependency_unavailable and leaves Cell state unchanged.
- Audit posture: dependency receipt digests are included in the audit-chain evidence envelope; raw dependency payloads stay with policy-cedar.
### 6.7 Delegated surface: api-gateway
- Delegation: owns public routing and carrier-triplet enforcement at the edge.
- Contract stance: cell-lifecycle calls api-gateway through a port and persists only the receipt or evidence digest needed for lifecycle history.
- Non-goal: no api-gateway implementation detail, retry policy beyond port semantics, or direct datastore ownership is embedded in cell-lifecycle domain logic.
- Failure posture: if api-gateway is unavailable, lifecycle transition returns dependency_unavailable and leaves Cell state unchanged.
- Audit posture: dependency receipt digests are included in the audit-chain evidence envelope; raw dependency payloads stay with api-gateway.

## 7. Promotion Gates per ADR-0266
### 7.1 Promotion target T4: best-effort-edge
- Timing floor: 56d warm-soak when moving outward, 168h quiet window, lowest blast-radius traffic only.
- G1 error budget: at least 99 percent of the relevant OpenSLO error budget remains for the current tier window; target_tier=T4; evidence key=promotion.t4.gate_1.
- G2 warm soak: the cell has remained in the source tier for the ADR-0266 per-edge minimum duration; target_tier=T4; evidence key=promotion.t4.gate_2.
- G3 canary cohort: canary cohort SLO is at least 99.5 percent over the warm-soak window; target_tier=T4; evidence key=promotion.t4.gate_3.
- G4 cross-cell mesh: cross-cell call success is at least 99.95 percent over the warm-soak window; target_tier=T4; evidence key=promotion.t4.gate_4.
- G5 tenant class coverage: both demo_trial and paid tenant classes are represented in the evidence pack; target_tier=T4; evidence key=promotion.t4.gate_5.
- G6 compliance pack coverage: every applicable ADR-0207 compliance pack has signed evidence coverage; target_tier=T4; evidence key=promotion.t4.gate_6.
- Cedar permit: action PromoteToT4 requires principal role, evidence digest, gate snapshot hash, and no active incident block unless emergency override is signed.
- Compliance invariant: every pack applicable to tenant_class_scope must be valid for T4 before promotion is accepted.
- Blast-radius invariant: T4 promotion must not raise effective tenant exposure beyond the ADR-0204 class declared in manifest capacity_model.cell_placement_class.
- Audit result: accepted promotion emits cell.promotion.executed with source tier, target tier T4, evaluator version, and gate_snapshot_sha256.
### 7.2 Promotion target T3: application
- Timing floor: 28d warm-soak, 96h quiet window, application workload SLO evidence.
- G1 error budget: at least 99 percent of the relevant OpenSLO error budget remains for the current tier window; target_tier=T3; evidence key=promotion.t3.gate_1.
- G2 warm soak: the cell has remained in the source tier for the ADR-0266 per-edge minimum duration; target_tier=T3; evidence key=promotion.t3.gate_2.
- G3 canary cohort: canary cohort SLO is at least 99.5 percent over the warm-soak window; target_tier=T3; evidence key=promotion.t3.gate_3.
- G4 cross-cell mesh: cross-cell call success is at least 99.95 percent over the warm-soak window; target_tier=T3; evidence key=promotion.t3.gate_4.
- G5 tenant class coverage: both demo_trial and paid tenant classes are represented in the evidence pack; target_tier=T3; evidence key=promotion.t3.gate_5.
- G6 compliance pack coverage: every applicable ADR-0207 compliance pack has signed evidence coverage; target_tier=T3; evidence key=promotion.t3.gate_6.
- Cedar permit: action PromoteToT3 requires principal role, evidence digest, gate snapshot hash, and no active incident block unless emergency override is signed.
- Compliance invariant: every pack applicable to tenant_class_scope must be valid for T3 before promotion is accepted.
- Blast-radius invariant: T3 promotion must not raise effective tenant exposure beyond the ADR-0204 class declared in manifest capacity_model.cell_placement_class.
- Audit result: accepted promotion emits cell.promotion.executed with source tier, target tier T3, evaluator version, and gate_snapshot_sha256.
### 7.3 Promotion target T2: capability
- Timing floor: 14d warm-soak, 48h quiet window, capability and data-plane dependency evidence.
- G1 error budget: at least 99 percent of the relevant OpenSLO error budget remains for the current tier window; target_tier=T2; evidence key=promotion.t2.gate_1.
- G2 warm soak: the cell has remained in the source tier for the ADR-0266 per-edge minimum duration; target_tier=T2; evidence key=promotion.t2.gate_2.
- G3 canary cohort: canary cohort SLO is at least 99.5 percent over the warm-soak window; target_tier=T2; evidence key=promotion.t2.gate_3.
- G4 cross-cell mesh: cross-cell call success is at least 99.95 percent over the warm-soak window; target_tier=T2; evidence key=promotion.t2.gate_4.
- G5 tenant class coverage: both demo_trial and paid tenant classes are represented in the evidence pack; target_tier=T2; evidence key=promotion.t2.gate_5.
- G6 compliance pack coverage: every applicable ADR-0207 compliance pack has signed evidence coverage; target_tier=T2; evidence key=promotion.t2.gate_6.
- Cedar permit: action PromoteToT2 requires principal role, evidence digest, gate snapshot hash, and no active incident block unless emergency override is signed.
- Compliance invariant: every pack applicable to tenant_class_scope must be valid for T2 before promotion is accepted.
- Blast-radius invariant: T2 promotion must not raise effective tenant exposure beyond the ADR-0204 class declared in manifest capacity_model.cell_placement_class.
- Audit result: accepted promotion emits cell.promotion.executed with source tier, target tier T2, evaluator version, and gate_snapshot_sha256.
### 7.4 Promotion target T1: substrate
- Timing floor: 7d warm-soak, 24h quiet window, Kata runtime and tenant-data-plane controls.
- G1 error budget: at least 99 percent of the relevant OpenSLO error budget remains for the current tier window; target_tier=T1; evidence key=promotion.t1.gate_1.
- G2 warm soak: the cell has remained in the source tier for the ADR-0266 per-edge minimum duration; target_tier=T1; evidence key=promotion.t1.gate_2.
- G3 canary cohort: canary cohort SLO is at least 99.5 percent over the warm-soak window; target_tier=T1; evidence key=promotion.t1.gate_3.
- G4 cross-cell mesh: cross-cell call success is at least 99.95 percent over the warm-soak window; target_tier=T1; evidence key=promotion.t1.gate_4.
- G5 tenant class coverage: both demo_trial and paid tenant classes are represented in the evidence pack; target_tier=T1; evidence key=promotion.t1.gate_5.
- G6 compliance pack coverage: every applicable ADR-0207 compliance pack has signed evidence coverage; target_tier=T1; evidence key=promotion.t1.gate_6.
- Cedar permit: action PromoteToT1 requires principal role, evidence digest, gate snapshot hash, and no active incident block unless emergency override is signed.
- Compliance invariant: every pack applicable to tenant_class_scope must be valid for T1 before promotion is accepted.
- Blast-radius invariant: T1 promotion must not raise effective tenant exposure beyond the ADR-0204 class declared in manifest capacity_model.cell_placement_class.
- Audit result: accepted promotion emits cell.promotion.executed with source tier, target tier T1, evaluator version, and gate_snapshot_sha256.
### 7.5 Promotion target T0: foundation
- Timing floor: same inbound criteria plus highest isolation, council-security permit, and critical registry DR evidence.
- G1 error budget: at least 99 percent of the relevant OpenSLO error budget remains for the current tier window; target_tier=T0; evidence key=promotion.t0.gate_1.
- G2 warm soak: the cell has remained in the source tier for the ADR-0266 per-edge minimum duration; target_tier=T0; evidence key=promotion.t0.gate_2.
- G3 canary cohort: canary cohort SLO is at least 99.5 percent over the warm-soak window; target_tier=T0; evidence key=promotion.t0.gate_3.
- G4 cross-cell mesh: cross-cell call success is at least 99.95 percent over the warm-soak window; target_tier=T0; evidence key=promotion.t0.gate_4.
- G5 tenant class coverage: both demo_trial and paid tenant classes are represented in the evidence pack; target_tier=T0; evidence key=promotion.t0.gate_5.
- G6 compliance pack coverage: every applicable ADR-0207 compliance pack has signed evidence coverage; target_tier=T0; evidence key=promotion.t0.gate_6.
- Cedar permit: action PromoteToT0 requires principal role, evidence digest, gate snapshot hash, and no active incident block unless emergency override is signed.
- Compliance invariant: every pack applicable to tenant_class_scope must be valid for T0 before promotion is accepted.
- Blast-radius invariant: T0 promotion must not raise effective tenant exposure beyond the ADR-0204 class declared in manifest capacity_model.cell_placement_class.
- Audit result: accepted promotion emits cell.promotion.executed with source tier, target tier T0, evaluator version, and gate_snapshot_sha256.

## 8. Evidence Pack Consumption per ADR-0217
### 8.1 Evidence field: evidence_pack_id
- Requirement: evidence_pack_id is a stable reference, not an embedded blob; raw evidence remains in the owning evidence store.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.2 Evidence field: evidence_pack_sha256
- Requirement: evidence_pack_sha256 is required to bind the request to the exact pack validated by the gate evaluator.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.3 Evidence field: gate_snapshot_sha256
- Requirement: gate_snapshot_sha256 captures the six ADR-0266 gate inputs as observed at decision time.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.4 Evidence field: cedar_decision_id
- Requirement: cedar_decision_id records the policy verdict and principal context used for the transition.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.5 Evidence field: audit_chain_event_id
- Requirement: audit_chain_event_id records the signed event after the state write succeeds.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.6 Evidence field: cloud_iac_receipt_id
- Requirement: cloud_iac_receipt_id records provisioning or readiness proof for Registered and Activated transitions.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.7 Evidence field: cell_rebalancer_receipt_id
- Requirement: cell_rebalancer_receipt_id records drain plan acceptance and migration completion proof.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.8 Evidence field: tenancy_resident_count_snapshot
- Requirement: tenancy_resident_count_snapshot records the resident count used for decommission checks.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.9 Evidence field: observability_window_id
- Requirement: observability_window_id records SLO, canary, mesh, and latency windows used in promotion validation.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.10 Evidence field: compliance_pack_receipts
- Requirement: compliance_pack_receipts record ADR-0207 pack coverage and any pack-specific stricter floor.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.11 Evidence field: blast_radius_check_id
- Requirement: blast_radius_check_id records the ADR-0204 tier and placement safety check.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.
### 8.12 Evidence field: hlc_timestamp
- Requirement: hlc_timestamp records the cross-region ordering point for deterministic replay.
- Read behavior: GET lifecycle returns the identifier, digest, result, and timestamp, but not secret material or raw pack contents.
- Write behavior: transition commands validate presence and digest shape before invoking downstream validators.
- Retention behavior: evidence references remain immutable for the lifetime of the Cell aggregate plus compliance retention windows.

## 9. Cedar Authorization Model
### 9.1 Cedar rule requirement
- Foundry automation principal is oyatie.foundry.cell-lifecycle and remains in the foundry namespace retained by ADR-0203 even after foundry service retirement.
- Refusal is explicit: API returns 403 with cedar_decision_id, refusal_code, and missing_context keys when policy denies.
- Domain logic treats missing Cedar context as deny, never as allow.
### 9.2 Cedar rule requirement
- Promotion permits are tier-specific: PromoteToT4, PromoteToT3, PromoteToT2, PromoteToT1, and PromoteToT0 are not interchangeable.
- Refusal is explicit: API returns 403 with cedar_decision_id, refusal_code, and missing_context keys when policy denies.
- Domain logic treats missing Cedar context as deny, never as allow.
### 9.3 Cedar rule requirement
- Drain authorization requires a drain evidence pack and a Cedar permit; a hardware failure reason still needs policy authorization.
- Refusal is explicit: API returns 403 with cedar_decision_id, refusal_code, and missing_context keys when policy denies.
- Domain logic treats missing Cedar context as deny, never as allow.
### 9.4 Cedar rule requirement
- Decommission authorization is forbidden unless the current state is Draining and tenancy proves resident_count == 0.
- Refusal is explicit: API returns 403 with cedar_decision_id, refusal_code, and missing_context keys when policy denies.
- Domain logic treats missing Cedar context as deny, never as allow.
### 9.5 Cedar rule requirement
- Manual ops principals require ops-cellular or cellular-sre role plus incident context for emergency override paths.
- Refusal is explicit: API returns 403 with cedar_decision_id, refusal_code, and missing_context keys when policy denies.
- Domain logic treats missing Cedar context as deny, never as allow.
### 9.6 Cedar rule requirement
- Every policy decision id is persisted into LifecycleHistory and emitted to audit-chain.
- Refusal is explicit: API returns 403 with cedar_decision_id, refusal_code, and missing_context keys when policy denies.
- Domain logic treats missing Cedar context as deny, never as allow.

## 10. Compliance Pack Invariants per ADR-0207
### 10.1 Pack invariant: kr
- Promotion into any tier that claims kr coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve kr audit evidence even after the cell has zero residents.
- Cross-region replication for kr metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing kr receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.2 Pack invariant: eu
- Promotion into any tier that claims eu coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve eu audit evidence even after the cell has zero residents.
- Cross-region replication for eu metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing eu receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.3 Pack invariant: us
- Promotion into any tier that claims us coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve us audit evidence even after the cell has zero residents.
- Cross-region replication for us metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing us receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.4 Pack invariant: us-healthcare
- Promotion into any tier that claims us-healthcare coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve us-healthcare audit evidence even after the cell has zero residents.
- Cross-region replication for us-healthcare metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing us-healthcare receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.5 Pack invariant: jp
- Promotion into any tier that claims jp coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve jp audit evidence even after the cell has zero residents.
- Cross-region replication for jp metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing jp receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.6 Pack invariant: sg
- Promotion into any tier that claims sg coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve sg audit evidence even after the cell has zero residents.
- Cross-region replication for sg metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing sg receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.7 Pack invariant: au
- Promotion into any tier that claims au coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve au audit evidence even after the cell has zero residents.
- Cross-region replication for au metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing au receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.8 Pack invariant: in
- Promotion into any tier that claims in coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve in audit evidence even after the cell has zero residents.
- Cross-region replication for in metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing in receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.9 Pack invariant: br
- Promotion into any tier that claims br coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve br audit evidence even after the cell has zero residents.
- Cross-region replication for br metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing br receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.10 Pack invariant: ae
- Promotion into any tier that claims ae coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve ae audit evidence even after the cell has zero residents.
- Cross-region replication for ae metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing ae receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.11 Pack invariant: ksa
- Promotion into any tier that claims ksa coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve ksa audit evidence even after the cell has zero residents.
- Cross-region replication for ksa metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing ksa receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.12 Pack invariant: soc2
- Promotion into any tier that claims soc2 coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve soc2 audit evidence even after the cell has zero residents.
- Cross-region replication for soc2 metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing soc2 receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.13 Pack invariant: iso27001
- Promotion into any tier that claims iso27001 coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve iso27001 audit evidence even after the cell has zero residents.
- Cross-region replication for iso27001 metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing iso27001 receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.14 Pack invariant: gdpr
- Promotion into any tier that claims gdpr coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve gdpr audit evidence even after the cell has zero residents.
- Cross-region replication for gdpr metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing gdpr receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.15 Pack invariant: hipaa
- Promotion into any tier that claims hipaa coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve hipaa audit evidence even after the cell has zero residents.
- Cross-region replication for hipaa metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing hipaa receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.16 Pack invariant: pci-dss
- Promotion into any tier that claims pci-dss coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve pci-dss audit evidence even after the cell has zero residents.
- Cross-region replication for pci-dss metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing pci-dss receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.17 Pack invariant: fedramp-high
- Promotion into any tier that claims fedramp-high coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve fedramp-high audit evidence even after the cell has zero residents.
- Cross-region replication for fedramp-high metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing fedramp-high receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.
### 10.18 Pack invariant: eu-ai-act
- Promotion into any tier that claims eu-ai-act coverage requires signed pack receipt and current certification status.
- Drain and decommission preserve eu-ai-act audit evidence even after the cell has zero residents.
- Cross-region replication for eu-ai-act metadata follows HLC ordering and does not move tenant data outside residency constraints.
- Missing eu-ai-act receipt blocks promotion but does not block emergency drain when blast-radius containment requires movement.

## 11. Blast-Radius Checks per ADR-0204
### 11.1 Blast-radius tier T4
- Classification: best-effort-edge.
- Check: target tier T4 must match the Cell aggregate placement_class and the manifest capacity_model.cell_placement_class contract.
- Check: promotion cannot widen allowed resident classes unless gate evidence includes both tenant classes and all applicable packs.
- Check: drain cannot be skipped for decommission because stale routing or resident tenants would leak blast radius into retired capacity.
- Evidence: blast_radius_check_id is included in every promotion and drain audit event.
### 11.2 Blast-radius tier T3
- Classification: application.
- Check: target tier T3 must match the Cell aggregate placement_class and the manifest capacity_model.cell_placement_class contract.
- Check: promotion cannot widen allowed resident classes unless gate evidence includes both tenant classes and all applicable packs.
- Check: drain cannot be skipped for decommission because stale routing or resident tenants would leak blast radius into retired capacity.
- Evidence: blast_radius_check_id is included in every promotion and drain audit event.
### 11.3 Blast-radius tier T2
- Classification: capability.
- Check: target tier T2 must match the Cell aggregate placement_class and the manifest capacity_model.cell_placement_class contract.
- Check: promotion cannot widen allowed resident classes unless gate evidence includes both tenant classes and all applicable packs.
- Check: drain cannot be skipped for decommission because stale routing or resident tenants would leak blast radius into retired capacity.
- Evidence: blast_radius_check_id is included in every promotion and drain audit event.
### 11.4 Blast-radius tier T1
- Classification: substrate.
- Check: target tier T1 must match the Cell aggregate placement_class and the manifest capacity_model.cell_placement_class contract.
- Check: promotion cannot widen allowed resident classes unless gate evidence includes both tenant classes and all applicable packs.
- Check: drain cannot be skipped for decommission because stale routing or resident tenants would leak blast radius into retired capacity.
- Evidence: blast_radius_check_id is included in every promotion and drain audit event.
### 11.5 Blast-radius tier T0
- Classification: foundation.
- Check: target tier T0 must match the Cell aggregate placement_class and the manifest capacity_model.cell_placement_class contract.
- Check: promotion cannot widen allowed resident classes unless gate evidence includes both tenant classes and all applicable packs.
- Check: drain cannot be skipped for decommission because stale routing or resident tenants would leak blast radius into retired capacity.
- Evidence: blast_radius_check_id is included in every promotion and drain audit event.

## 12. Functional Requirements
FR-001: Register a logical cell after cloud-iac readiness or reservation evidence is available.
Acceptance FR-001: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-002: Activate a registered cell only after telemetry, audit-chain, Cedar, and pack-readiness checks are reachable.
Acceptance FR-002: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-003: Promote a cell through T4, T3, T2, T1, and T0 in order; skipping requires emergency override evidence and explicit policy.
Acceptance FR-003: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-004: Drain any active promoted or activated cell and delegate tenant migration to cell-rebalancer.
Acceptance FR-004: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-005: Decommission only from Draining and only when tenancy reports zero residents.
Acceptance FR-005: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-006: List cells with state, region, tier, tenant_class scope, stale evidence age, and next legal transitions.
Acceptance FR-006: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-007: Return immutable lifecycle history with audit-chain event references and evidence digests.
Acceptance FR-007: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-008: Preserve idempotency across retries and cross-region replication lag.
Acceptance FR-008: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-009: Under Accepted ADR-0632, limit any downstream public surface to HTTPS REST/OpenAPI 3.2.0, versioned webhooks described by AsyncAPI 3.1.0 with CloudEvents 1.0.2 where its stable HTTP binding applies, SSE, and WebSocket as applicable; prefer HTTP/3 at capable public edges with mandatory HTTP/2 fallback. Keep gRPC/proto3 internal-only over HTTP/2 with mTLS and TLS 1.3; do not add public gRPC or GraphQL.
Acceptance FR-009: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.
FR-010: Emit OpenSLO metrics for register, promote, drain, lookup, evidence validation, and drain-to-decommission duration.
Acceptance FR-010: request, state, evidence, Cedar, audit-chain, and observability outcomes are testable without Rust implementation in this scaffold.

## 13. Nonfunctional Requirements
NFR-001: register p99 <= 100 ms excluding dependency cold-start latency
Validation NFR-001: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-002: promote p99 <= 500 ms excluding asynchronous evidence pack construction
Validation NFR-002: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-003: drain intent p99 <= 200 ms excluding tenant migration duration
Validation NFR-003: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-004: lookup p99 <= 50 ms using Valkey hot lookup for non-sensitive state projections
Validation NFR-004: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-005: promotion evidence validation p99 <= 30 s for complete sealed packs
Validation NFR-005: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-006: drain-to-decommission maximum duration <= 168 h unless compliance hold prevents tenant migration
Validation NFR-006: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-007: RTO <= 5 minutes and RPO <= 30 seconds for registry and lifecycle history
Validation NFR-007: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-008: cross-region replication uses HLC ordering and preserves monotonic lifecycle_version increments
Validation NFR-008: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-009: all privileged transitions are Cedar-gated and audit-chain sealed
Validation NFR-009: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.
NFR-010: no direct caller can mutate LifecycleHistory after append
Validation NFR-010: represented in manifest, OpenSLO, OpenAPI schema, Cedar policy, runbook, or IP evidence.

## 14. Foundry Agent Boundary
FAB-001: The foundry-cell-orchestrator may propose transitions but cannot apply state mutation without Cedar authorization and evidence pack validation.
Control FAB-001: enforced by Cedar policy, API request schema, and runbook review sections.
FAB-002: Self-modification is limited to preparing follow-up PRs for policy, manifest, or runbook updates; runtime state remains inside Postgres and audit-chain.
Control FAB-002: enforced by Cedar policy, API request schema, and runbook review sections.
FAB-003: The principal oyatie.foundry.cell-lifecycle is narrow to this service and distinct from oyatie.foundry.cell-orchestrator.
Control FAB-003: enforced by Cedar policy, API request schema, and runbook review sections.
FAB-004: Automation may not create infrastructure, migrate tenants, or edit api-gateway routing directly.
Control FAB-004: enforced by Cedar policy, API request schema, and runbook review sections.
FAB-005: Automation must emit audit-chain evidence for every proposal, acceptance, refusal, and emergency override.
Control FAB-005: enforced by Cedar policy, API request schema, and runbook review sections.

## 15. Acceptance Matrix
AC-001: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-002: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-003: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-004: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-005: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-006: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-007: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-008: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-009: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-010: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-011: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-012: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-013: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-014: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-015: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-016: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-017: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-018: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-019: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-020: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-021: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-022: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-023: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-024: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-025: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-026: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-027: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-028: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-029: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-030: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-031: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-032: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-033: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-034: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-035: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-036: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-037: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-038: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-039: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-040: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-041: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-042: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-043: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-044: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-045: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-046: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-047: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-048: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-049: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-050: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-051: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-052: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-053: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-054: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-055: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-056: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-057: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-058: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-059: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-060: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-061: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-062: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-063: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-064: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-065: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-066: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-067: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-068: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-069: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-070: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-071: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-072: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-073: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-074: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-075: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-076: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-077: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-078: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-079: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-080: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-081: The scaffold documents evidence pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-082: The scaffold documents Cedar permit behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-083: The scaffold documents compliance pack behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-084: The scaffold documents blast radius behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-085: The scaffold documents tenant class behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-086: The scaffold documents dependency boundary behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-087: The scaffold documents observability behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-088: The scaffold documents DR behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-089: The scaffold documents API versioning behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.
AC-090: The scaffold documents state machine behavior with a concrete validator hook, failure mode, or downstream implementation instruction traceable to ADR-0276 D-3.

## 16. Implementation-Plan Mapping
### 16.1 IP-CL-001: bounded context and state machine
- PRD trace: IP-CL-001 owns the implementation plan for bounded context and state machine and carries acceptance criteria into Wave 15-ZD-impl.
- Scaffold state: authored here as documentation only; no Rust code is created.
### 16.2 IP-CL-002: promotion gate evidence pack
- PRD trace: IP-CL-002 owns the implementation plan for promotion gate evidence pack and carries acceptance criteria into Wave 15-ZD-impl.
- Scaffold state: authored here as documentation only; no Rust code is created.
### 16.3 IP-CL-003: drain and decommission coordination with cell-rebalancer
- PRD trace: IP-CL-003 owns the implementation plan for drain and decommission coordination with cell-rebalancer and carries acceptance criteria into Wave 15-ZD-impl.
- Scaffold state: authored here as documentation only; no Rust code is created.
### 16.4 IP-CL-004: public API, events, streaming, and internal RPC boundary
- PRD trace: Under Accepted ADR-0632, IP-CL-004 owns the downstream selection and implementation plan for applicable public REST/OpenAPI, versioned webhooks, AsyncAPI/CloudEvents, SSE/WebSocket, HTTP/3 edge preference with HTTP/2 fallback, and internal-only gRPC/proto3 over HTTP/2 with mTLS and TLS 1.3; it carries acceptance criteria into Wave 15-ZD-impl without claiming an implemented generator or runtime.
- Scaffold state: authored here as documentation only; no Rust code is created.
### 16.5 IP-CL-005: Cedar authorization and promotion permit
- PRD trace: IP-CL-005 owns the implementation plan for Cedar authorization and promotion permit and carries acceptance criteria into Wave 15-ZD-impl.
- Scaffold state: authored here as documentation only; no Rust code is created.
### 16.6 IP-CL-006: audit-chain transition evidence
- PRD trace: IP-CL-006 owns the implementation plan for audit-chain transition evidence and carries acceptance criteria into Wave 15-ZD-impl.
- Scaffold state: authored here as documentation only; no Rust code is created.
### 16.7 IP-CL-007: SLO metrics and observability
- PRD trace: IP-CL-007 owns the implementation plan for SLO metrics and observability and carries acceptance criteria into Wave 15-ZD-impl.
- Scaffold state: authored here as documentation only; no Rust code is created.
### 16.8 IP-CL-008: foundry self-modification boundary
- PRD trace: IP-CL-008 owns the implementation plan for foundry self-modification boundary and carries acceptance criteria into Wave 15-ZD-impl.
- Scaffold state: authored here as documentation only; no Rust code is created.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0341 adoption
ADOPT-0341-001: `cell-lifecycle` adopts ADR-0341 as the product requirement for routine promotion, demotion, and emergency override state transitions across ADR-0248 Tier 0..4.
ADOPT-0341-002: The `promote-cell` capability must require all six gate inputs before the service records a promoted state: error-budget intact, warm-soak floor, canary cohort SLO, cell-mesh health, tenant-class coverage, and compliance-pack coverage.
ADOPT-0341-003: The service remains the logical state-machine authority only; observability, tenancy, compliance, audit-chain, policy-cedar, cloud-iac, cell-rebalancer, and api-gateway keep their existing ownership.
ADOPT-0341-004: The product contract for routine promotion is fail-closed: missing, stale, mismatched, or wrong-direction gate receipts produce typed refusal reasons rather than partial state changes.
ADOPT-0341-005: Demotion is a first-class safety outcome and uses ADR-0341's stricter thresholds to protect blast radius without waiting for the routine promotion quiet window.
ADOPT-0341-006: Emergency override remains exceptional and still requires signed evidence, audit-chain emission, and reviewable gate snapshot references.
ADOPT-0341-007: OpenAPI 3.2.0 promotion request and response surfaces in a downstream implementation must carry `evidence_pack_id`, `gate_snapshot_sha256`, tier edge fields, idempotency key, lifecycle version, and audit-chain event reference.
ADOPT-0341-008: AsyncAPI 3.1.0 becomes applicable only if lifecycle transition events are published; event payloads must carry bounded identifiers and digests, not raw telemetry, raw compliance material, or tenant payloads.
ADOPT-0341-009: The manifest declaration `cell_promotion_gates` is the machine-readable source for applicable tiers, promotion windows, quiet windows, evidence sources, and enforced-by lanes.
ADOPT-0341-010: The manifest declaration `cell_promotion_history` starts empty in this documentation-stage wave and accepts entries only when real audit-chain promotion events exist.
ADOPT-0341-011: This adoption remains PROPOSED until downstream Rust implementation, contract tests, fake-adapter integration tests, audit-chain ordering tests, and validation evidence prove the behavior.
ADOPT-0341-012: Acceptance for this PRD block is limited to doctrine propagation; it must not be read as a claim that transition handlers have been implemented.
