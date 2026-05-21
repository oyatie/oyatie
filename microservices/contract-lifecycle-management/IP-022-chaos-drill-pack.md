# IP-022 Contract Lifecycle Management chaos-drill-pack
Service: contract-lifecycle-management
IP title: chaos-drill-pack
Deepening target: CLM failure drill package
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
References used: PRD.md, ARCHITECTURE.md, contracts/openapi-v1.yaml, contracts/asyncapi-v1.yaml, contracts/contract-lifecycle-management-v1.proto, policy/data-residency.md, threat-model.md, dpia.md, failure-modes.md, backfill-replay.md.

## Core Design
1. chaos-drill-pack is a CLM-owned surface, not a vendor workflow replica.
2. The boundary is contract state, clause control, obligation work, approval evidence, renewal analysis, signature evidence, and DealSet binding.
3. This IP names legal object model and implementation deltas for CLM failure drill package.
4. Command rule: drills prove CLM survives provider outage, audit backpressure, Cedar drift, region loss, and replay corruption.
5. Canonical workflow: armed -> injected -> observed -> recovered -> debriefed.
6. Cedar guardrail: chaos actor may inject only approved failure modes in non-production or controlled cells.
7. Proto/OpenAPI emphasis: DrillRun, InjectionPoint, RecoveryEvidence.
8. Async emphasis: drill.started, drill.injected, drill.recovered.

## Domain Objects And State
9. DrillScenario summarizes chaos-drill-pack identity with tenant_scope_ref, source_vendor_ref, policy_decision_id, and audit_event_ref.
10. DrillScenario transition records actor_role, workflow_run_id, policy_bundle_hash, and contract_version_ref before the next legal state is visible.
11. DrillScenario maps DocuSign CLM, Icertis, Ironclad, ContractPodAi, Agiloft, LinkSquares, and Conga CLM ids into provenance-only fields.
12. DrillScenario rollback uses rollback_anchor_ref plus compensating_event_id to reverse CLM failure drill package while retaining imported documents and callbacks.
13. DrillScenario export includes data_class, residency_label, retention_rule_id, redaction_profile, and evidence_packet_ref.
14. InjectionPoint exports chaos-drill-pack identity with tenant_scope_ref, source_vendor_ref, policy_decision_id, and audit_event_ref.
15. InjectionPoint transition records actor_role, workflow_run_id, policy_bundle_hash, and contract_version_ref before the next legal state is visible.
16. InjectionPoint maps DocuSign CLM, Icertis, Ironclad, ContractPodAi, Agiloft, LinkSquares, and Conga CLM ids into provenance-only fields.
17. InjectionPoint rollback uses rollback_anchor_ref plus compensating_event_id to reverse CLM failure drill package while retaining imported documents and callbacks.
18. InjectionPoint export includes data_class, residency_label, retention_rule_id, redaction_profile, and evidence_packet_ref.
19. ExpectedRefusal summarizes chaos-drill-pack identity with tenant_scope_ref, source_vendor_ref, policy_decision_id, and audit_event_ref.
20. ExpectedRefusal transition records actor_role, workflow_run_id, policy_bundle_hash, and contract_version_ref before the next legal state is visible.
21. ExpectedRefusal maps DocuSign CLM, Icertis, Ironclad, ContractPodAi, Agiloft, LinkSquares, and Conga CLM ids into provenance-only fields.
22. ExpectedRefusal rollback uses rollback_anchor_ref plus compensating_event_id to reverse CLM failure drill package while retaining imported documents and callbacks.
23. ExpectedRefusal export includes data_class, residency_label, retention_rule_id, redaction_profile, and evidence_packet_ref.
24. RecoveryTimer exports chaos-drill-pack identity with tenant_scope_ref, source_vendor_ref, policy_decision_id, and audit_event_ref.
25. RecoveryTimer transition records actor_role, workflow_run_id, policy_bundle_hash, and contract_version_ref before the next legal state is visible.
26. RecoveryTimer maps DocuSign CLM, Icertis, Ironclad, ContractPodAi, Agiloft, LinkSquares, and Conga CLM ids into provenance-only fields.
27. RecoveryTimer rollback uses rollback_anchor_ref plus compensating_event_id to reverse CLM failure drill package while retaining imported documents and callbacks.
28. RecoveryTimer export includes data_class, residency_label, retention_rule_id, redaction_profile, and evidence_packet_ref.
29. EvidenceBundle summarizes chaos-drill-pack identity with tenant_scope_ref, source_vendor_ref, policy_decision_id, and audit_event_ref.
30. EvidenceBundle transition records actor_role, workflow_run_id, policy_bundle_hash, and contract_version_ref before the next legal state is visible.
31. EvidenceBundle maps DocuSign CLM, Icertis, Ironclad, ContractPodAi, Agiloft, LinkSquares, and Conga CLM ids into provenance-only fields.
32. EvidenceBundle rollback uses rollback_anchor_ref plus compensating_event_id to reverse CLM failure drill package while retaining imported documents and callbacks.
33. EvidenceBundle export includes data_class, residency_label, retention_rule_id, redaction_profile, and evidence_packet_ref.

## Command And Response Deltas
34. `redlineThreadOpen.create` OpenAPI body carries tenant_id, principal_id, legal purpose, data class, idempotency key, workflow run, and audit event pointer.
35. `redlineThreadOpen.create` output can be audited because domain_ref, policy_decision_id, evidence_packet_ref, replay_cursor, and refusal code are explicit.
36. `redlineThreadOpen.create` stops before adapter access when source_vendor_id, tenant scope, pack overlay, or legal hold are inconsistent.
37. `obligationReviewCommit.preview` cannot be formed without tenant_id, principal_id, purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref.
38. `obligationReviewCommit.preview` response keeps domain_ref separate from provider ids and includes policy_decision_id, evidence_packet_ref, replay_cursor, and refusal_info.
39. `obligationReviewCommit.preview` blocks adapter execution when the source vendor record would cross tenant, pack, or hold boundaries.
40. `renewalRiskScore.amend` request shape for chaos-drill-pack: tenant_id and principal_id lead the body; purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref are mandatory.
41. `renewalRiskScore.amend` returns a replay-safe envelope with domain_ref, Cedar decision, evidence packet, replay cursor, and refusal payload.
42. `renewalRiskScore.amend` short-circuits with CLM refusal evidence if provider provenance conflicts with tenant scope, pack overlay, or legal hold.
43. `clausePolicyEvaluate.evaluate` SDK call exposes tenant_id, principal_id, purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref as named fields.
44. `clausePolicyEvaluate.evaluate` success or refusal body names domain_ref, policy_decision_id, evidence_packet_ref, replay_cursor, and typed refusal details.
45. `clausePolicyEvaluate.evaluate` does not open a provider call until source_vendor_id, tenant scope, pack overlay, and legal hold status agree.
46. `clausePolicyEvaluate.route` OpenAPI body carries tenant_id, principal_id, legal purpose, data class, idempotency key, workflow run, and audit event pointer.
47. `clausePolicyEvaluate.route` output can be audited because domain_ref, policy_decision_id, evidence_packet_ref, replay_cursor, and refusal code are explicit.
48. `clausePolicyEvaluate.route` stops before adapter access when source_vendor_id, tenant scope, pack overlay, or legal hold are inconsistent.
49. `obligationReviewCommit.approve` cannot be formed without tenant_id, principal_id, purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref.
50. `obligationReviewCommit.approve` response keeps domain_ref separate from provider ids and includes policy_decision_id, evidence_packet_ref, replay_cursor, and refusal_info.
51. `obligationReviewCommit.approve` blocks adapter execution when the source vendor record would cross tenant, pack, or hold boundaries.
52. `dealsetContractBind.extract` request shape for chaos-drill-pack: tenant_id and principal_id lead the body; purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref are mandatory.
53. `dealsetContractBind.extract` returns a replay-safe envelope with domain_ref, Cedar decision, evidence packet, replay cursor, and refusal payload.
54. `dealsetContractBind.extract` short-circuits with CLM refusal evidence if provider provenance conflicts with tenant scope, pack overlay, or legal hold.
55. `contractDraftCreate.score` SDK call exposes tenant_id, principal_id, purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref as named fields.
56. `contractDraftCreate.score` success or refusal body names domain_ref, policy_decision_id, evidence_packet_ref, replay_cursor, and typed refusal details.
57. `contractDraftCreate.score` does not open a provider call until source_vendor_id, tenant scope, pack overlay, and legal hold status agree.
58. `approvalRoutePlan.prepare` OpenAPI body carries tenant_id, principal_id, legal purpose, data class, idempotency key, workflow run, and audit event pointer.
59. `approvalRoutePlan.prepare` output can be audited because domain_ref, policy_decision_id, evidence_packet_ref, replay_cursor, and refusal code are explicit.
60. `approvalRoutePlan.prepare` stops before adapter access when source_vendor_id, tenant scope, pack overlay, or legal hold are inconsistent.
61. `approvalRoutePlan.bind` cannot be formed without tenant_id, principal_id, purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref.
62. `approvalRoutePlan.bind` response keeps domain_ref separate from provider ids and includes policy_decision_id, evidence_packet_ref, replay_cursor, and refusal_info.
63. `approvalRoutePlan.bind` blocks adapter execution when the source vendor record would cross tenant, pack, or hold boundaries.
64. `signaturePacketPrepare.export` request shape for chaos-drill-pack: tenant_id and principal_id lead the body; purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref are mandatory.
65. `signaturePacketPrepare.export` returns a replay-safe envelope with domain_ref, Cedar decision, evidence packet, replay cursor, and refusal payload.
66. `signaturePacketPrepare.export` short-circuits with CLM refusal evidence if provider provenance conflicts with tenant scope, pack overlay, or legal hold.
67. `contractImportPreview.replay` SDK call exposes tenant_id, principal_id, purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref as named fields.
68. `contractImportPreview.replay` success or refusal body names domain_ref, policy_decision_id, evidence_packet_ref, replay_cursor, and typed refusal details.
69. `contractImportPreview.replay` does not open a provider call until source_vendor_id, tenant scope, pack overlay, and legal hold status agree.
70. `approvalRoutePlan.reverse` OpenAPI body carries tenant_id, principal_id, legal purpose, data class, idempotency key, workflow run, and audit event pointer.
71. `approvalRoutePlan.reverse` output can be audited because domain_ref, policy_decision_id, evidence_packet_ref, replay_cursor, and refusal code are explicit.
72. `approvalRoutePlan.reverse` stops before adapter access when source_vendor_id, tenant scope, pack overlay, or legal hold are inconsistent.
73. `dealsetContractBind.quarantine` cannot be formed without tenant_id, principal_id, purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref.
74. `dealsetContractBind.quarantine` response keeps domain_ref separate from provider ids and includes policy_decision_id, evidence_packet_ref, replay_cursor, and refusal_info.
75. `dealsetContractBind.quarantine` blocks adapter execution when the source vendor record would cross tenant, pack, or hold boundaries.
76. `renewalRiskScore.close` request shape for chaos-drill-pack: tenant_id and principal_id lead the body; purpose, data_class, idempotency_key, workflow_run_id, and audit_event_ref are mandatory.
77. `renewalRiskScore.close` returns a replay-safe envelope with domain_ref, Cedar decision, evidence packet, replay cursor, and refusal payload.
78. `renewalRiskScore.close` short-circuits with CLM refusal evidence if provider provenance conflicts with tenant scope, pack overlay, or legal hold.

## Async Events
79. Event `drill.started` publishes `chaos_drill_pack_ref` beside causation_id, correlation_id, contract_record_id, and source_vendor_id.
80. Event `drill.started` is replayable only when transform_version, policy_decision_id, and replay_cursor match stored evidence.
81. Event `drill.started` redacts counterparty contacts unless the subscriber has CLM legal-operations purpose.
82. Event `drill.injected` publishes `chaos_drill_pack_ref` beside causation_id, correlation_id, contract_record_id, and source_vendor_id.
83. Event `drill.injected` is replayable only when transform_version, policy_decision_id, and replay_cursor match stored evidence.
84. Event `drill.injected` redacts counterparty contacts unless the subscriber has CLM legal-operations purpose.
85. Event `drill.recovered` publishes `chaos_drill_pack_ref` beside causation_id, correlation_id, contract_record_id, and source_vendor_id.
86. Event `drill.recovered` is replayable only when transform_version, policy_decision_id, and replay_cursor match stored evidence.
87. Event `drill.recovered` redacts counterparty contacts unless the subscriber has CLM legal-operations purpose.
88. Derived event `chaos.drill.pack.requested` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
89. Derived event `chaos.drill.pack.accepted` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
90. Derived event `chaos.drill.pack.rejected` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
91. Derived event `chaos.drill.pack.quarantined` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
92. Derived event `chaos.drill.pack.replayed` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
93. Derived event `chaos.drill.pack.reversed` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
94. Derived event `chaos.drill.pack.exported` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
95. Derived event `chaos.drill.pack.reviewed` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
96. Derived event `chaos.drill.pack.timed_out` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.
97. Derived event `chaos.drill.pack.stale` maps the CLM failure drill package lifecycle into AsyncAPI without copying a vendor event taxonomy.

## Proto And OpenAPI Changes
98. Proto field `DrillRun` prevents chaos-drill-pack evidence from becoming an untyped metadata blob.
99. Proto field `DrillRun` has an explicit validation error so SDKs do not collapse CLM legal refusals into transport failures.
100. Proto field `DrillRun` appears in replay fixtures to prove migration behavior remains deterministic.
101. Proto field `InjectionPoint` prevents chaos-drill-pack evidence from becoming an untyped metadata blob.
102. Proto field `InjectionPoint` has an explicit validation error so SDKs do not collapse CLM legal refusals into transport failures.
103. Proto field `InjectionPoint` appears in replay fixtures to prove migration behavior remains deterministic.
104. Proto field `RecoveryEvidence` prevents chaos-drill-pack evidence from becoming an untyped metadata blob.
105. Proto field `RecoveryEvidence` has an explicit validation error so SDKs do not collapse CLM legal refusals into transport failures.
106. Proto field `RecoveryEvidence` appears in replay fixtures to prove migration behavior remains deterministic.
107. OpenAPI `ChaosDrillPackCreateRequest` names legal purpose, pack overlay, source vendor, domain reference, and evidence link.
108. OpenAPI `ChaosDrillPackPreviewResponse` names legal purpose, pack overlay, source vendor, domain reference, and evidence link.
109. OpenAPI `ChaosDrillPackReviewRequest` names legal purpose, pack overlay, source vendor, domain reference, and evidence link.
110. OpenAPI `ChaosDrillPackDecisionResponse` names legal purpose, pack overlay, source vendor, domain reference, and evidence link.
111. OpenAPI `ChaosDrillPackReplayRequest` names legal purpose, pack overlay, source vendor, domain reference, and evidence link.
112. OpenAPI `ChaosDrillPackRollbackReceipt` names legal purpose, pack overlay, source vendor, domain reference, and evidence link.
113. OpenAPI `ChaosDrillPackEvidenceExport` names legal purpose, pack overlay, source vendor, domain reference, and evidence link.

## Cedar Facts
114. Cedar evaluates `principal tenant membership` before chaos-drill-pack can mutate a CLM aggregate or invoke a provider adapter.
115. Failed `principal tenant membership` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
116. Cedar requires `legal operations role` for chaos-drill-pack; missing facts produce appealable refusal evidence.
117. Failed `legal operations role` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
118. Cedar fact `counterparty bounded scope` is part of the chaos-drill-pack permit context and is logged in the denial receipt when false.
119. Failed `counterparty bounded scope` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
120. Cedar evaluates `contract data class` before chaos-drill-pack can mutate a CLM aggregate or invoke a provider adapter.
121. Failed `contract data class` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
122. Cedar requires `pack overlay restriction` for chaos-drill-pack; missing facts produce appealable refusal evidence.
123. Failed `pack overlay restriction` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
124. Cedar fact `workflow delegation` is part of the chaos-drill-pack permit context and is logged in the denial receipt when false.
125. Failed `workflow delegation` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
126. Cedar evaluates `provider callback proof` before chaos-drill-pack can mutate a CLM aggregate or invoke a provider adapter.
127. Failed `provider callback proof` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
128. Cedar requires `source object provenance` for chaos-drill-pack; missing facts produce appealable refusal evidence.
129. Failed `source object provenance` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
130. Cedar fact `legal hold status` is part of the chaos-drill-pack permit context and is logged in the denial receipt when false.
131. Failed `legal hold status` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
132. Cedar evaluates `DealSet settlement authority` before chaos-drill-pack can mutate a CLM aggregate or invoke a provider adapter.
133. Failed `DealSet settlement authority` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
134. Cedar requires `audit export purpose` for chaos-drill-pack; missing facts produce appealable refusal evidence.
135. Failed `audit export purpose` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.
136. Cedar fact `break-glass expiry` is part of the chaos-drill-pack permit context and is logged in the denial receipt when false.
137. Failed `break-glass expiry` writes a denial receipt with actor, action, resource, purpose, pack, and appeal route.

## Workflow Decisions
138. Workflow state `armed` owns a visible operator decision for chaos-drill-pack; imported provider status cannot skip it.
139. Transition out of `armed` records owner_ref, decision_reason, evidence_packet_ref, and rollback_anchor_ref.
140. Workflow state `injected` owns a visible operator decision for chaos-drill-pack; imported provider status cannot skip it.
141. Transition out of `injected` records owner_ref, decision_reason, evidence_packet_ref, and rollback_anchor_ref.
142. Workflow state `observed` owns a visible operator decision for chaos-drill-pack; imported provider status cannot skip it.
143. Transition out of `observed` records owner_ref, decision_reason, evidence_packet_ref, and rollback_anchor_ref.
144. Workflow state `recovered` owns a visible operator decision for chaos-drill-pack; imported provider status cannot skip it.
145. Transition out of `recovered` records owner_ref, decision_reason, evidence_packet_ref, and rollback_anchor_ref.
146. Workflow state `debriefed` owns a visible operator decision for chaos-drill-pack; imported provider status cannot skip it.
147. Transition out of `debriefed` records owner_ref, decision_reason, evidence_packet_ref, and rollback_anchor_ref.
148. Handoff to `drive` passes CLM domain refs and policy decisions, not vendor ids as authority.
149. Handoff to `workflow-engine` passes CLM domain refs and policy decisions, not vendor ids as authority.
150. Handoff to `ontology` passes CLM domain refs and policy decisions, not vendor ids as authority.
151. Handoff to `audit-chain` passes CLM domain refs and policy decisions, not vendor ids as authority.
152. Handoff to `marketplace` passes CLM domain refs and policy decisions, not vendor ids as authority.
153. Handoff to `payments` passes CLM domain refs and policy decisions, not vendor ids as authority.
154. Handoff to `workplace-integration` passes CLM domain refs and policy decisions, not vendor ids as authority.

## Failure And Mitigation Cases
155. Failure `tenant mismatch` mitigation for chaos-drill-pack: freeze the attempted command before aggregate hydration, attach the mismatched tenant pair, and route a scope-dispute task to legal ops.
156. Recovery for `tenant mismatch`: re-run the command with the accepted tenant_scope_ref and quarantine only the bad source mapping.
157. Failure `policy denial` mitigation for chaos-drill-pack: return the Cedar denial receipt with appeal_route_id and leave the CLM aggregate untouched.
158. Recovery for `policy denial`: replay only after a newer policy_bundle_hash is approved and the denial receipt remains in audit history.
159. Failure `provider timeout` mitigation for chaos-drill-pack: hold the adapter call in pending_provider state and keep the legal workflow visible to the operator.
160. Recovery for `provider timeout`: resume from provider_request_id with the original idempotency key and suppress duplicate callbacks.
161. Failure `audit-chain backpressure` mitigation for chaos-drill-pack: pause high-risk mutations, buffer signed evidence locally, and expose degraded audit status.
162. Recovery for `audit-chain backpressure`: flush buffered evidence in order before releasing the paused legal transition.
163. Failure `source object drift` mitigation for chaos-drill-pack: compare source_version_hash against the accepted transform input and open a migration drift review.
164. Recovery for `source object drift`: create a compensating event that points to both the stale and refreshed source spans.
165. Failure `redline tamper suspicion` mitigation for chaos-drill-pack: lock the affected redline thread, preserve the uploaded binary, and request provenance review.
166. Recovery for `redline tamper suspicion`: accept a rebuilt lineage only after the tamper finding is closed by counsel.
167. Failure `obligation ambiguity` mitigation for chaos-drill-pack: send the candidate to confidence review with source span, sentence window, and proposed owner.
168. Recovery for `obligation ambiguity`: commit the obligation only from the reviewer decision id, not from model output alone.
169. Failure `renewal stale input` mitigation for chaos-drill-pack: mark the score stale, hide automated recommendations, and keep notice deadlines visible.
170. Recovery for `renewal stale input`: recompute from the same contract version plus refreshed obligation and usage inputs.
171. Failure `signature callback replay` mitigation for chaos-drill-pack: dedupe by packet id, provider event id, signer ref, and callback timestamp tolerance.
172. Recovery for `signature callback replay`: append a duplicate-callback audit note while preserving the original signature state.
173. Failure `DealSet settlement hold` mitigation for chaos-drill-pack: block settlement release and show contract evidence gaps to marketplace operations.
174. Recovery for `DealSet settlement hold`: release only after the CLM binding event, signature status, and entitlement delta agree.
175. Failure `residency conflict` mitigation for chaos-drill-pack: apply the most restrictive pack and stop export or replication before document assembly.
176. Recovery for `residency conflict`: rerun the export with the resolved residency label and regulator packet reference.
177. Failure `DPIA missing purpose` mitigation for chaos-drill-pack: block processing that touches personal data and request a purpose-code amendment.
178. Recovery for `DPIA missing purpose`: resume after DPIA signoff ties data class, purpose, retention, and reviewer.
179. Failure `SDK idempotency collision` mitigation for chaos-drill-pack: return the original operation result when payload hashes match and refuse divergent payload reuse.
180. Recovery for `SDK idempotency collision`: issue a typed idempotency error that includes the first command id and replay cursor.
181. Failure `operator break-glass expiry` mitigation for chaos-drill-pack: close the emergency grant, deny further reads, and queue post-review evidence.
182. Recovery for `operator break-glass expiry`: new access requires a fresh counsel approval and cannot inherit the expired grant.
183. Failure `catalog owner mismatch` mitigation for chaos-drill-pack: block promotion and assign catalog correction to the service owner group.
184. Recovery for `catalog owner mismatch`: allow retry only after catalog owner, layer, SLO, and runbook references align.

## Migration And Replay Fixtures
185. DocuSign CLM fixture for chaos-drill-pack: envelope and signature concepts are imported as provider evidence, while CLM owns packet, tenant, and audit state.
186. DocuSign CLM replay check stores source_object_id, transform_version, accepted_domain_ref, refusal_reason, and export_evidence_ref.
187. Icertis fixture for chaos-drill-pack: agreement intelligence is treated as migration input, while Oyatie keeps clause, obligation, and renewal decisions explainable.
188. Icertis replay check stores source_object_id, transform_version, accepted_domain_ref, refusal_reason, and export_evidence_ref.
189. Ironclad fixture for chaos-drill-pack: workflow collaboration is displaced by workflow_run_id plus CLM approval and deviation records.
190. Ironclad replay check stores source_object_id, transform_version, accepted_domain_ref, refusal_reason, and export_evidence_ref.
191. ContractPodAi fixture for chaos-drill-pack: matter automation maps to CLM playbooks without turning vendor matter ids into domain ids.
192. ContractPodAi replay check stores source_object_id, transform_version, accepted_domain_ref, refusal_reason, and export_evidence_ref.
193. Agiloft fixture for chaos-drill-pack: configurable tables map to explicit CLM aggregates with policy gates and replay fixtures.
194. Agiloft replay check stores source_object_id, transform_version, accepted_domain_ref, refusal_reason, and export_evidence_ref.
195. LinkSquares fixture for chaos-drill-pack: repository search maps to source discovery and provenance, not canonical contract storage.
196. LinkSquares replay check stores source_object_id, transform_version, accepted_domain_ref, refusal_reason, and export_evidence_ref.
197. Conga CLM fixture for chaos-drill-pack: package lifecycle maps to CLM commands, events, and DealSet settlement evidence.
198. Conga CLM replay check stores source_object_id, transform_version, accepted_domain_ref, refusal_reason, and export_evidence_ref.
199. Replay fixture `happy_path_import` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
200. Replay fixture `cross_tenant_refusal` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
201. Replay fixture `provider_id_collision` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
202. Replay fixture `policy_bundle_upgrade` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
203. Replay fixture `pack_overlay_conflict` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
204. Replay fixture `audit_backpressure_resume` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
205. Replay fixture `redline_lineage_rebuild` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
206. Replay fixture `obligation_review_replay` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
207. Replay fixture `renewal_score_recompute` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.
208. Replay fixture `signature_callback_duplicate` proves chaos-drill-pack can rerun from backfill-replay.md without widening scope or rewriting accepted history.

## SLOs And Runbooks
209. SLO `local-contract-cycle-time.openslo.yaml` records a chaos-drill-pack dimension when this IP contributes latency, freshness, completeness, or provider success risk.
210. SLO `local-clause-policy-eval-latency.openslo.yaml` records a chaos-drill-pack dimension when this IP contributes latency, freshness, completeness, or provider success risk.
211. SLO `local-obligation-extract-completeness.openslo.yaml` records a chaos-drill-pack dimension when this IP contributes latency, freshness, completeness, or provider success risk.
212. SLO `local-redline-turnaround-latency.openslo.yaml` records a chaos-drill-pack dimension when this IP contributes latency, freshness, completeness, or provider success risk.
213. SLO `local-renewal-risk-freshness.openslo.yaml` records a chaos-drill-pack dimension when this IP contributes latency, freshness, completeness, or provider success risk.
214. SLO `local-signature-provider-success.openslo.yaml` records a chaos-drill-pack dimension when this IP contributes latency, freshness, completeness, or provider success risk.
215. Runbook `local-clause-policy-latency-burn.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
216. Runbook `local-contract-cycle-time-burn.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
217. Runbook `local-obligation-extract-gap.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
218. Runbook `local-redline-turnaround-lag.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
219. Runbook `local-renewal-risk-stale.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
220. Runbook `local-signature-provider-outage.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
221. Runbook `local-counterparty-scope-dispute.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
222. Runbook `local-dealset-contract-hold.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
223. Runbook `local-legal-hold-activation.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.
224. Runbook `local-template-version-rollback.md` includes operator notes for diagnosing chaos-drill-pack without changing records outside the affected tenant and contract family.

## Acceptance Tests
225. Test `missing required field` for chaos-drill-pack: expects the named field in OpenAPI problem.details and no domain id allocation.
226. Test `missing required field` evidence: checks no AsyncAPI event is emitted and the idempotency key can be reused safely.
227. Test `wrong tenant` for chaos-drill-pack: expects Cedar denial plus tenant_scope_ref mismatch evidence.
228. Test `wrong tenant` evidence: checks replay cannot move the source object into a different tenant.
229. Test `wrong actor role` for chaos-drill-pack: expects role-specific denial rather than a generic forbidden response.
230. Test `wrong actor role` evidence: checks appeal_route_id and support escalation visibility.
231. Test `counterparty overreach` for chaos-drill-pack: expects counterparty_scope_id to limit contract, clause, and comment reads.
232. Test `counterparty overreach` evidence: checks tenant approval graph and obligation queues remain hidden.
233. Test `provider duplicate callback` for chaos-drill-pack: expects duplicate provider event suppression with audit annotation.
234. Test `provider duplicate callback` evidence: checks signature or import state is not advanced twice.
235. Test `stale policy bundle` for chaos-drill-pack: expects policy_bundle_hash mismatch and no mutation commit.
236. Test `stale policy bundle` evidence: checks replay after bundle upgrade keeps old denial evidence.
237. Test `stale ontology projection` for chaos-drill-pack: expects projection_stale status and operator-visible rebuild task.
238. Test `stale ontology projection` evidence: checks canonical contract state remains readable while projection is repaired.
239. Test `redacted export` for chaos-drill-pack: expects data-class redaction markers in exported evidence.
240. Test `redacted export` evidence: checks raw counterparty contacts are absent from unauthorized packets.
241. Test `legal hold freeze` for chaos-drill-pack: expects writes blocked and reads marked legal_hold_active.
242. Test `legal hold freeze` evidence: checks rollback cannot remove the hold audit event.
243. Test `residency conflict` for chaos-drill-pack: expects pack conflict details and export refusal.
244. Test `residency conflict` evidence: checks no cross-cell document replica is created.
245. Test `idempotent replay` for chaos-drill-pack: expects same result for same command hash and same tenant scope.
246. Test `idempotent replay` evidence: checks divergent replay payload gets a typed collision error.
247. Test `rollback receipt` for chaos-drill-pack: expects prior_version_ref, compensating_event_id, and operator_reason.
248. Test `rollback receipt` evidence: checks provider artifacts are detached rather than deleted.
249. Test `SDK typed error` for chaos-drill-pack: expects generated client error enum with CLM-specific code.
250. Test `SDK typed error` evidence: checks retry guidance is explicit and not transport-only.
251. Test `audit evidence completeness` for chaos-drill-pack: expects actor, action, resource, purpose, policy, and evidence refs.
252. Test `audit evidence completeness` evidence: checks audit-chain export can reconstruct the decision path.
253. Test `benchmark import displacement` for chaos-drill-pack: expects source vendor ids preserved only as provenance.
254. Test `benchmark import displacement` evidence: checks canonical ids remain Oyatie CLM ids.
255. Test `DealSet hold` for chaos-drill-pack: expects settlement_hold_reason and missing evidence pointers.
256. Test `DealSet hold` evidence: checks marketplace release is impossible before CLM proof.
257. Test `signature outage` for chaos-drill-pack: expects provider route degraded status and alternate route eligibility.
258. Test `signature outage` evidence: checks signer intent and packet evidence survive failover.
259. Test `renewal stale score` for chaos-drill-pack: expects stale score hidden from automation but visible to reviewer.
260. Test `renewal stale score` evidence: checks recompute records the refreshed input set.
261. Test `obligation low confidence` for chaos-drill-pack: expects candidate routed to review with source span highlighted.
262. Test `obligation low confidence` evidence: checks no obligation task is assigned before reviewer decision.
263. Test `redline provenance dispute` for chaos-drill-pack: expects lineage lock and tamper/dispute status.
264. Test `redline provenance dispute` evidence: checks comment history export names disputed spans.

## Rollback Rules
265. Rollback action `detach provider artifact` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
266. Rollback action `restore prior domain version` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
267. Rollback action `mark projection stale` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
268. Rollback action `close pending workflow task` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
269. Rollback action `suspend obligation task` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
270. Rollback action `hide renewal score` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
271. Rollback action `cancel signature packet` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
272. Rollback action `hold DealSet settlement` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
273. Rollback action `emit audit reversal` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.
274. Rollback action `export regulator packet` is valid for chaos-drill-pack only when the receipt names contract_record_id, policy_decision_id, and evidence_packet_ref.

## Final Substance Checks

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/contract-lifecycle-management/contracts/asyncapi-v1.yaml`, `microservices/contract-lifecycle-management/contracts/contract-lifecycle-management-v1.proto`, `microservices/contract-lifecycle-management/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`, `asyncapi`, `.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/contract-lifecycle-management/IP-022-chaos-drill-pack.md` matched [`payment`, `SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/contract-lifecycle-management/IP-022-chaos-drill-pack.md`, `microservices/contract-lifecycle-management/manifest.json`, `microservices/contract-lifecycle-management/ARCHITECTURE.md`, `microservices/contract-lifecycle-management/PRD.md`, `microservices/contract-lifecycle-management/multi-region.md`, `microservices/contract-lifecycle-management/capacity-model.md`].
