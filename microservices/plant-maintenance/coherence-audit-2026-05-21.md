# Plant Maintenance Ownership-Coherence Audit - 2026-05-20

Audit owner: sole Codex audit owner for `microservices/plant-maintenance/`.
Target microservice: `plant-maintenance`.
Assigned counterpart set: SAP Plant Maintenance / IBM Maximo / UpKeep.
Deliverable set: three reports only; the tenant-class delta report is retired.
Audit stance: read-only investigation followed by report authoring inside this microservice path.
Stop condition: all required reports authored, line floors verified, and the orchestrator report appended here.
Inventory scope: every file returned by `rg --files microservices/plant-maintenance`.
Inventory file count: 151 files.
Inventory line count: 20,250 existing lines read or sampled by required artifact class.
Tier-language scan count: 401 line-level hits across the service path.
Exact color-name scan: one non-feature standards citation at `IP-022-mtbf-weibull-fitting-reliability-analytics.md:328`.
Tenant-class scan result: no `tenant_class`, `demo_trial`, `revenue_share`, per-seat, usage-based, or billing component adoption found in this service path.

## 1. Purpose

1. This audit checks whether `plant-maintenance` owns one coherent product surface rather than a copied scaffold.
2. The target product surface is plant maintenance and EAM parity for equipment, maintenance plans, work orders, spare reservations, technician dispatch, and downtime windows.
3. The service PRD states the parity target directly: equipment masters, maintenance plans, work orders, spare reservations, technician dispatch, and downtime windows are in scope at `PRD.md:28-32`.
4. The service manifest repeats six bounded contexts, including `technician-dispatch`, at `manifest.json:31-38`.
5. The current Rust domain implementation exposes only five capabilities at `src/domain/mod.rs:20-26`.
6. The current Rust domain enum exposes only five bounded contexts at `src/domain/mod.rs:48-54`.
7. The current HTTP route table exposes only five command routes at `src/adapter/http.rs:30-62`.
8. The product coherence question is therefore not abstract; the docs and contracts name six contexts while the implementation path omits one.
9. The canonical direction question is also not abstract; ADR-0328 binds all Wave-3 ownership audits to nine dimensions at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3831-3852`.
10. The deployability question is concrete; `master-plan-sequencing.json` defines six deployment contexts at `specs/master-plan-sequencing.json:704-745`.
11. The IaC question is concrete; the master plan requires OpenTofu and forbids Terraform, Pulumi, CloudFormation, and ARM as IaC substrates at `specs/master-plan-sequencing.json:747-775`.
12. The OS question is concrete; the master plan requires `supported-oses.json` and the tier-1 OS matrix at `specs/master-plan-sequencing.json:777-815`.
13. The language question is concrete; the master plan requires Rust backend and only the allowed frontend surfaces at `specs/master-plan-sequencing.json:817-856`.
14. The OCI profile question is concrete; the master plan names `iac/oci-guest/always-free/` as the required OCI Always Free profile path at `specs/master-plan-sequencing.json:857-868`.
15. The tier-retirement question is concrete; `feedback_no_tenant_classes_2026_05_20.md:10-24` retires tenant classes, and this prompt retires the fourth tier-delta deliverable.
16. The replacement commercial model in this audit is tenant class semantics: `demo_trial`, `paid`, and `revenue_share`.
17. The quality bar is uniform industry-leader grade across those tenant classes.
18. The audit does not create feature tiers.
19. The audit treats old tier words as retirement candidates, not as a structure to perpetuate.
20. The audit compares the service to SAP Plant Maintenance, IBM Maximo, and UpKeep because the prompt assigns that union-coverage bar.
21. Local history confirms the service was originally introduced as an SAP PM parity microservice with equipment master, work orders, preventive maintenance, and spare parts in the chat trace at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8869`.
22. Local history also confirms `plant-maintenance` was dispatched with SAP Plant Maintenance, IBM Maximo, and UpKeep as counterparts at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17216-17219`.
23. The current manifest still lists Microsoft Dynamics 365 Field Service instead of UpKeep at `manifest.json:25-29`.
24. The current PRD lists Infor EAM instead of UpKeep at `PRD.md:55-60`.
25. The contract benchmark roster lists Oracle, Workday, NetSuite, and Microsoft instead of IBM Maximo and UpKeep at `contracts/openapi-v1.yaml:9-14`.
26. That mismatch is a product-direction drift, not a stylistic difference.
27. The audit classifies severity using ADR-0328 Dimension 9 guidance for product, canonical, and implementation gaps at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4094-4121`.
28. P0 means the service cannot be represented as canonically ready for Wave-3 deployability.
29. P1 means a product or implementation path would mislead implementers or break parity.
30. P2 means the documentation or doctrine needs Wave-15J cleanup or hardening before promotion.
31. P3 means a useful supporting artifact exists but still needs follow-through or linkage.
32. This report uses file:line citations for findings and records absences by command evidence where no file can provide a line citation.
33. Absence evidence includes the exact inventory and searches described in this header.
34. No files outside `microservices/plant-maintenance/` are modified by this audit.
35. No commit is made by this audit.

## 2. Complete Inventory

### 2.1 Inventory summary

1. Inventory command: `rg --files microservices/plant-maintenance | sort`.
2. Inventory result: 151 files.
3. Existing line count command: `wc -l $(rg --files microservices/plant-maintenance) | tail -1`.
4. Existing line count result: 20,250 lines.
5. Top-level product docs present: `PRD.md`, `ARCHITECTURE.md`, `README.md`, `manifest.json`, and companion governance docs.
6. Implementation-plan docs present as top-level `IP-001` through `IP-025`.
7. ADR-MS decision docs are absent; no `decisions/ADR-MS-*.md` files exist in the inventory.
8. Formal `implementation-plans/` directory is absent; the implementation plans are top-level `IP-*.md`.
9. Contracts are present in OpenAPI, AsyncAPI, and proto forms.
10. SLO files are present in OpenSLO YAML form.
11. `tenant-classs/` directory is absent; old tier language is embedded in other artifacts instead.
12. `cross-microservice-handoffs.md` is absent; cross-service ownership is scattered through the PRD and manifest.
13. `benchmarks/` directory is absent; performance data is spread across SLO and capacity docs.
14. `faqs/`, `onboarding/`, `migration-playbooks/`, `reference-implementations/`, and `tutorials/` directories are absent.
15. `iac/` exists, but it has no six-context subdirectories.
16. `iac/terraform-module/main.tf` exists, which conflicts with the OpenTofu-only direction.
17. `supported-oses.json` is absent.
18. Rust source exists under `src/`.
19. One integration test file exists under `tests/`.
20. No forbidden backend-language source files were found by extension scan.

### 2.2 Files seen

1. `microservices/plant-maintenance/ARCHITECTURE.md`
2. `microservices/plant-maintenance/AUDIT-FINDINGS-2026-05-21.json`
3. `microservices/plant-maintenance/CHANGELOG.md`
4. `microservices/plant-maintenance/Cargo.toml`
5. `microservices/plant-maintenance/IP-001-domain-layer-for-equipment-master.md`
6. `microservices/plant-maintenance/IP-002-domain-layer-for-maintenance-plan.md`
7. `microservices/plant-maintenance/IP-003-domain-layer-for-work-order.md`
8. `microservices/plant-maintenance/IP-004-domain-layer-for-spare-part-reservation.md`
9. `microservices/plant-maintenance/IP-005-domain-layer-for-technician-dispatch.md`
10. `microservices/plant-maintenance/IP-006-domain-layer-for-downtime-window.md`
11. `microservices/plant-maintenance/IP-007-usecase-layer-for-equipment-master.md`
12. `microservices/plant-maintenance/IP-008-usecase-layer-for-maintenance-plan.md`
13. `microservices/plant-maintenance/IP-009-usecase-layer-for-work-order.md`
14. `microservices/plant-maintenance/IP-010-usecase-layer-for-spare-part-reservation.md`
15. `microservices/plant-maintenance/IP-011-usecase-layer-for-technician-dispatch.md`
16. `microservices/plant-maintenance/IP-012-usecase-layer-for-downtime-window.md`
17. `microservices/plant-maintenance/IP-013-adapter-integrations-for-plant-maintenance.md`
18. `microservices/plant-maintenance/IP-014-rest-grpc-and-worker-surfaces-for-plant-maintenance.md`
19. `microservices/plant-maintenance/IP-015-integration-tests-for-plant-maintenance.md`
20. `microservices/plant-maintenance/IP-016-safety-loto-9-state-machine-with-audit-chain.md`
21. `microservices/plant-maintenance/IP-017-permit-to-work-issuance-workflow.md`
22. `microservices/plant-maintenance/IP-018-work-order-release-cedar-gate-skill-matrix-verification.md`
23. `microservices/plant-maintenance/IP-019-spare-parts-mrp-linkage.md`
24. `microservices/plant-maintenance/IP-020-condition-based-maintenance-iot-signal-ingestion.md`
25. `microservices/plant-maintenance/IP-021-reliability-centered-maintenance-decision-logic.md`
26. `microservices/plant-maintenance/IP-022-mtbf-weibull-fitting-reliability-analytics.md`
27. `microservices/plant-maintenance/IP-023-maintenance-kpi-scorecard-oee-mttr-firsttimefix.md`
28. `microservices/plant-maintenance/IP-024-maintenance-strategy-cycle-generation-due-date-calculator.md`
29. `microservices/plant-maintenance/IP-025-equipment-hierarchy-class-characteristic-schema-and-relocation.md`
30. `microservices/plant-maintenance/PHASE-01-PLANT-MAINTENANCE-PARITY.md`
31. `microservices/plant-maintenance/PRD.md`
32. `microservices/plant-maintenance/README.md`
33. `microservices/plant-maintenance/backfill-replay.md`
34. `microservices/plant-maintenance/capabilities/equipment-master-command.yaml`
35. `microservices/plant-maintenance/capabilities/maintenance-plan-reconcile.yaml`
36. `microservices/plant-maintenance/capabilities/work-order-export.yaml`
37. `microservices/plant-maintenance/capacity-model.md`
38. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-adapter.yaml`
39. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-api.yaml`
40. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-application.yaml`
41. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-domain.yaml`
42. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-governance.yaml`
43. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-kernel.yaml`
44. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-rest.yaml`
45. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-usecase.yaml`
46. `microservices/plant-maintenance/catalog/oya-plant-maintenance-downtime-window-worker.yaml`
47. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-adapter.yaml`
48. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-api.yaml`
49. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-application.yaml`
50. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-domain.yaml`
51. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-governance.yaml`
52. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-kernel.yaml`
53. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-rest.yaml`
54. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-usecase.yaml`
55. `microservices/plant-maintenance/catalog/oya-plant-maintenance-equipment-master-worker.yaml`
56. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-adapter.yaml`
57. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-api.yaml`
58. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-application.yaml`
59. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-domain.yaml`
60. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-governance.yaml`
61. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-kernel.yaml`
62. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-rest.yaml`
63. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-usecase.yaml`
64. `microservices/plant-maintenance/catalog/oya-plant-maintenance-maintenance-plan-worker.yaml`
65. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-adapter.yaml`
66. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-api.yaml`
67. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-application.yaml`
68. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-domain.yaml`
69. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-governance.yaml`
70. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-kernel.yaml`
71. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-rest.yaml`
72. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-usecase.yaml`
73. `microservices/plant-maintenance/catalog/oya-plant-maintenance-spare-part-reservation-worker.yaml`
74. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-adapter.yaml`
75. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-api.yaml`
76. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-application.yaml`
77. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-domain.yaml`
78. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-governance.yaml`
79. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-kernel.yaml`
80. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-rest.yaml`
81. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-usecase.yaml`
82. `microservices/plant-maintenance/catalog/oya-plant-maintenance-technician-dispatch-worker.yaml`
83. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-adapter.yaml`
84. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-api.yaml`
85. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-application.yaml`
86. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-domain.yaml`
87. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-governance.yaml`
88. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-kernel.yaml`
89. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-rest.yaml`
90. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-usecase.yaml`
91. `microservices/plant-maintenance/catalog/oya-plant-maintenance-work-order-worker.yaml`
92. `microservices/plant-maintenance/competitor-parity-matrix.md`
93. `microservices/plant-maintenance/compliance.md`
94. `microservices/plant-maintenance/contracts/asyncapi-v1.yaml`
95. `microservices/plant-maintenance/contracts/openapi-v1.yaml`
96. `microservices/plant-maintenance/contracts/plant-maintenance-v1.proto`
97. `microservices/plant-maintenance/cost-budget.md`
98. `microservices/plant-maintenance/dashboards/equipment-master-health.json`
99. `microservices/plant-maintenance/dashboards/maintenance-plan-residency.md`
100. `microservices/plant-maintenance/dashboards/plant-maintenance-overview.json`
101. `microservices/plant-maintenance/dpia.md`
102. `microservices/plant-maintenance/failure-modes.md`
103. `microservices/plant-maintenance/iac/ech-config.yaml`
104. `microservices/plant-maintenance/iac/edge-waf.yaml`
105. `microservices/plant-maintenance/iac/helm-values.yaml`
106. `microservices/plant-maintenance/iac/k8s-deployment.yaml`
107. `microservices/plant-maintenance/iac/network-policy.yaml`
108. `microservices/plant-maintenance/iac/openbao-policy.hcl`
109. `microservices/plant-maintenance/iac/pqc-cert.yaml`
110. `microservices/plant-maintenance/iac/secret-bindings.yaml`
111. `microservices/plant-maintenance/iac/terraform-module/main.tf`
112. `microservices/plant-maintenance/incident-response.md`
113. `microservices/plant-maintenance/manifest.json`
114. `microservices/plant-maintenance/multi-region.md`
115. `microservices/plant-maintenance/policy/abuse-defence.cedar`
116. `microservices/plant-maintenance/policy/auditor-scope.cedar`
117. `microservices/plant-maintenance/policy/ci-scope.cedar`
118. `microservices/plant-maintenance/policy/data-residency.md`
119. `microservices/plant-maintenance/policy/downtime-window-authorization.cedar`
120. `microservices/plant-maintenance/policy/emergency-services-bypass.cedar`
121. `microservices/plant-maintenance/policy/equipment-master-authorization.cedar`
122. `microservices/plant-maintenance/policy/maintenance-plan-authorization.cedar`
123. `microservices/plant-maintenance/policy/pack-overlay-authorization.cedar`
124. `microservices/plant-maintenance/policy/spare-part-reservation-authorization.cedar`
125. `microservices/plant-maintenance/policy/technician-dispatch-authorization.cedar`
126. `microservices/plant-maintenance/policy/tenant-isolation.md`
127. `microservices/plant-maintenance/policy/work-order-authorization.cedar`
128. `microservices/plant-maintenance/runbooks/approval-deadletter.md`
129. `microservices/plant-maintenance/runbooks/capacity-saturation.md`
130. `microservices/plant-maintenance/runbooks/marketplace-settlement-blocked.md`
131. `microservices/plant-maintenance/runbooks/policy-deny-spike.md`
132. `microservices/plant-maintenance/runbooks/regional-failover.md`
133. `microservices/plant-maintenance/runbooks/source-import-stalled.md`
134. `microservices/plant-maintenance/scorecards/overrides.json`
135. `microservices/plant-maintenance/sdk-plan.md`
136. `microservices/plant-maintenance/slos/equipment-master-success-rate.openslo.yaml`
137. `microservices/plant-maintenance/slos/plant-maintenance-availability.openslo.yaml`
138. `microservices/plant-maintenance/slos/plant-maintenance-latency-p99.openslo.yaml`
139. `microservices/plant-maintenance/slos/plant-maintenance-throughput.openslo.yaml`
140. `microservices/plant-maintenance/src/adapter/asyncapi.rs`
141. `microservices/plant-maintenance/src/adapter/grpc.rs`
142. `microservices/plant-maintenance/src/adapter/http.rs`
143. `microservices/plant-maintenance/src/adapter/mod.rs`
144. `microservices/plant-maintenance/src/config.rs`
145. `microservices/plant-maintenance/src/domain/mod.rs`
146. `microservices/plant-maintenance/src/error.rs`
147. `microservices/plant-maintenance/src/lib.rs`
148. `microservices/plant-maintenance/src/main.rs`
149. `microservices/plant-maintenance/src/usecase/mod.rs`
150. `microservices/plant-maintenance/tests/integration.rs`
151. `microservices/plant-maintenance/threat-model.md`

### 2.3 Required artifacts read or sampled

1. `PRD.md` was read across the product vision, capability, policy, telemetry, capacity, tier, and reference sections.
2. `ARCHITECTURE.md` was read for service boundary, layers, bounded contexts, integrations, failure modes, and contract map.
3. `README.md` was read for purpose, bounded contexts, contracts, operating posture, and evidence-row substance.
4. `manifest.json` was read for comparator roster, contexts, integration points, transport, deployment shape, tier fields, and dependencies.
5. All `IP-001` through `IP-025` files were inventoried; targeted tier and capability-language scans covered all of them.
6. `contracts/openapi-v1.yaml` was read for benchmark roster, path surface, and schema specificity.
7. `contracts/asyncapi-v1.yaml` was read for event channels and payload fields.
8. `contracts/plant-maintenance-v1.proto` was read for RPC surface and command/result messages.
9. All four OpenSLO files were read for latency, throughput, availability, and equipment-master success targets.
10. No `tenant-classs/` directory existed, so the tier audit scanned the whole service path.
11. `capacity-model.md` was read for Little's Law assumptions, command latency, arrival rates, and old tier assumptions.
12. `failure-modes.md` was read for failure taxonomy and response evidence.
13. `incident-response.md` was read for severity and response model.
14. `cost-budget.md` was read for unit-cost assumptions.
15. `dpia.md` and `compliance.md` were read for privacy and control posture.
16. `competitor-parity-matrix.md` and `PHASE-01-PLANT-MAINTENANCE-PARITY.md` were read for stale counterpart evidence.
17. `iac/` was inventoried and sampled; the only nested module is `iac/terraform-module/main.tf`.
18. Rust files sampled include `src/lib.rs`, `src/main.rs`, `src/config.rs`, `src/domain/mod.rs`, `src/usecase/mod.rs`, and `src/adapter/http.rs`.
19. `tests/integration.rs` was read for verification coverage and ignored tests.
20. Chat history was searched for `plant-maintenance` and six relevant match clusters were processed.

## 3. Nine-Dimension Audit

### 3.1 Dimension 1 - Product purpose and ownership

1. Verdict: coherent purpose at the PRD level, partial ownership at the implementation level.
2. Evidence: the PRD defines SAP PM/EAM coverage at `PRD.md:28-32`.
3. Evidence: the manifest names SAP PM surfaces at `manifest.json:17-24`.
4. Evidence: the manifest lists six bounded contexts at `manifest.json:31-38`.
5. Evidence: the architecture doc says the service owns equipment master, maintenance work orders, preventive schedules, spare parts, facility reliability, and adjacent-service boundaries at `ARCHITECTURE.md:21-22`.
6. Product ownership is strong for core PM nouns: equipment, plan, order, spare reservation, dispatch, downtime.
7. Product ownership is weaker for reliability analytics because the IP set adds LOTO, permit-to-work, condition-based maintenance, reliability-centered maintenance, Weibull analytics, KPI scorecards, and equipment relocation without a clear source-of-truth boundary in the Rust domain.
8. `IP-016` through `IP-025` add real EAM depth, but the Rust implementation has not absorbed those nouns into typed domain state.
9. The current source code still presents an early scaffold: `src/lib.rs:49-75` exposes layer and API descriptions rather than domain behavior.
10. The current CLI is local manifest/config validation only at `src/main.rs:1-55`.
11. The service is not a monolithic ERP suite; that is a positive alignment with `PRD.md:49-53`.
12. The service does not yet prove it can own daily plant maintenance execution end to end.
13. The strongest ownership evidence is the PRD plus manifest.
14. The weakest ownership evidence is the Rust domain and adapter implementation.
15. Severity for this dimension: P1 because missing implementation ownership affects parity.

### 3.2 Dimension 2 - Artifact completeness and buildability

1. Verdict: broad artifact scaffold exists, but buildability is not proven to industry-leader depth.
2. Evidence: contracts exist in REST, event, and proto forms at `contracts/openapi-v1.yaml:1-38`, `contracts/asyncapi-v1.yaml:1-37`, and `contracts/plant-maintenance-v1.proto:94-100`.
3. Evidence: four OpenSLO files exist, including p99 latency, throughput, availability, and equipment-master success rate.
4. Evidence: policy coverage exists for all named contexts, including `policy/technician-dispatch-authorization.cedar`.
5. Evidence: only three capability YAML records exist, while the PRD names six contexts at `PRD.md:64-68`.
6. Evidence: no `cross-microservice-handoffs.md` file exists in the 151-file inventory.
7. Evidence: no `benchmarks/` directory exists in the 151-file inventory.
8. Evidence: no `onboarding/`, `migration-playbooks/`, `reference-implementations/`, `tutorials/`, or `faqs/` directories exist in the 151-file inventory.
9. Evidence: `tests/integration.rs:59-77` marks contract, proto, AsyncAPI, Cedar, and repository tests as ignored.
10. Buildability is limited by generic payloads in OpenAPI schemas at `contracts/openapi-v1.yaml:132-176`.
11. Buildability is also limited by the HTTP handler returning `contract_stub("http")` at `src/adapter/http.rs:65-67`.
12. The artifact count looks high, but many companion docs use repeated row forms rather than implementer-specific instruction.
13. `README.md:32-200` starts a repeated evidence-row pattern.
14. `ARCHITECTURE.md:111-200` repeats trace-style rows after the core architecture.
15. The existing audit JSON claims second-pass closure at `AUDIT-FINDINGS-2026-05-21.json:3-8`, but this audit finds canonical blockers still open.
16. Severity for this dimension: P1 because buildability claims overrun verified implementation depth.

### 3.3 Dimension 3 - Counterpart union coverage

1. Verdict: the service targets a valid SAP PM/EAM family but drifts from the assigned union set.
2. Required counterpart set for this audit: SAP Plant Maintenance, IBM Maximo, and UpKeep.
3. Evidence: chat dispatch recorded plant-maintenance with SAP Plant Maintenance, IBM Maximo, and UpKeep at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17216-17219`.
4. Evidence: the manifest comparator list has SAP S/4HANA PM, IBM Maximo, and Microsoft Dynamics 365 Field Service at `manifest.json:25-29`.
5. Evidence: the PRD comparator list has SAP S/4HANA Asset Management, SAP Plant Maintenance, IBM Maximo, and Infor EAM at `PRD.md:55-60`.
6. Evidence: OpenAPI benchmark metadata names SAP, Oracle, Workday, NetSuite, and Microsoft at `contracts/openapi-v1.yaml:9-14`.
7. Evidence: `competitor-parity-matrix.md:13-20` follows the same stale SAP/Oracle/Workday/NetSuite/Microsoft roster.
8. SAP Plant Maintenance coverage appears strongest for equipment, maintenance plans, maintenance orders, spare parts, and downtime.
9. IBM Maximo coverage is partial in docs because Maximo appears in the manifest and PRD but not in the contract benchmark roster.
10. UpKeep coverage is effectively absent in current artifacts.
11. UpKeep gaps matter because UpKeep emphasizes mobile-first work orders, offline sync, requesters, parts inventory, analytics, and AI-assisted PM.
12. The current Oyatie surface includes technician dispatch in docs but omits it from Rust domain and HTTP routes, weakening both Maximo and UpKeep parity.
13. The current Oyatie surface includes spare-part reservation, but not inventory PO/reorder depth comparable to Maximo inventory or UpKeep parts workflows.
14. The current Oyatie surface includes downtime windows, which is a useful additive surface for plant operations.
15. The current Oyatie surface includes compliance packs and Cedar policy, which can exceed smaller CMMS tools if implemented.
16. Severity for this dimension: P1 because counterpart drift changes what the service is expected to cover.

### 3.4 Dimension 4 - Canonical-direction alignment

1. Verdict: canonical alignment is blocked by deployment-context, IaC, OS, OCI profile, tenant-class, and tier-retirement gaps.
2. Multi-context evidence: the master plan names six deployment contexts at `specs/master-plan-sequencing.json:704-745`.
3. Service evidence: no `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, or `oyatie-as-cloud-provider` text appears in the service path.
4. Service evidence: `manifest.json:161-164` only says Kubernetes plus Cloud Hypervisor/Kata-compatible isolation.
5. Gap: the service does not prove deployability across the six required contexts.
6. OpenTofu evidence: the master plan requires OpenTofu and forbids Terraform/Pulumi/CloudFormation/ARM IaC at `specs/master-plan-sequencing.json:747-775`.
7. Service evidence: `iac/terraform-module/main.tf` is present and has only variables/outputs, not context modules.
8. Service evidence: no `tofu` or `OpenTofu` references appear in the service path.
9. Gap: the service has no `iac/oyatie-public-cloud/`, `iac/aws-guest/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-as-cloud-provider/` module set.
10. OS evidence: the master plan requires `supported-oses.json` and tier-1 OS coverage at `specs/master-plan-sequencing.json:777-815`.
11. Service evidence: `supported-oses.json` is absent from the inventory.
12. Gap: the service cannot claim supported OS coverage.
13. Rust-strict evidence: backend Rust and frontend allowlist are bound at `specs/master-plan-sequencing.json:817-856`.
14. Service evidence: extension scan found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files.
15. Service evidence: `Cargo.toml:1-62` defines a Rust crate with workspace lints.
16. Positive alignment: Rust-strict backend file-type posture passes.
17. OCI evidence: the master plan requires `iac/oci-guest/always-free/` at `specs/master-plan-sequencing.json:857-868`.
18. Service evidence: no `iac/oci-guest/always-free/` path exists in inventory.
19. Gap: the service has no OCI Always Free profile for `demo_trial` infrastructure.
20. Severity for multi-context, OpenTofu, and OS gaps: P0.
21. Severity for OCI Always Free profile gap: P2 because it is a concrete missing sub-profile inside the broader P0 IaC blocker.

### 3.4.T Tier retirement candidates

1. Exact color-name scan found no tenant-class uses of demo_trial, demo_trial, paid_core_safe, or paid_high_assurance.
2. Exact color-name scan found one `IEEE Std 493` standards citation at `IP-022-mtbf-weibull-fitting-reliability-analytics.md:328`.
3. `IEEE Std 493` is a standards reference, not a feature tier, so it is excluded from Wave-15J tier retirement.
4. Whole-path tier-language scan found 401 line-level hits that should be reviewed under Wave 15J.
5. Manifest candidate: `manifest.json:7-8` uses `tier` and `tier_subtype`.
6. Manifest candidate: `manifest.json:136-140` uses `Tier 0` through `Tier 3` for cell eligibility.
7. Manifest candidate: `manifest.json:253-255` uses `tenant_classes`.
8. Manifest candidate: `manifest.json:269` uses `tier_classification`.
9. Manifest candidate: `manifest.json:290-291` uses tiered failure and criticality fields.
10. Cargo candidate: `Cargo.toml:18` uses `criticality_tier`.
11. PRD candidate: `PRD.md:31` binds ADR-0316 tenant-class activation.
12. PRD candidate: `PRD.md:36` scopes reads to tenant plus tenant class.
13. PRD candidate: `PRD.md:46` tells developer partners to build through tenant classes.
14. PRD candidate: `PRD.md:854` starts activation from tenant tenant-class selection.
15. PRD candidate: `PRD.md:941` binds policy coverage to tenant-class activation.
16. PRD candidate: `PRD.md:945-955` repeats tenant-class checks in Cedar hook prose.
17. PRD candidate: `PRD.md:1038` and `PRD.md:1058` name tenant-class promotion fixtures.
18. PRD candidate: `PRD.md:1068`, `1078`, `1088`, `1098`, `1108`, and `1118` use `tier` as telemetry dimension.
19. PRD candidate: `PRD.md:1132` sends `tenant_class` to finops.
20. PRD candidate: `PRD.md:1330-1412` is an entire tenant-class section and promotion-gate section.
21. Capacity candidate: `capacity-model.md:16-23` uses old tier assumptions for load models.
22. IP candidates: each `IP-001` through `IP-025` uses `tenant_class: substrate` in front matter at line 8.
23. IP candidate: `IP-001-domain-layer-for-equipment-master.md:29` uses cell-tier residency.
24. IP candidate: `IP-001-domain-layer-for-equipment-master.md:43` uses tier promotion.
25. IP candidate: `IP-013-adapter-integrations-for-plant-maintenance.md:29` uses per-cell-tier mapping.
26. These candidates should be rewritten to tenant class, deployment context, criticality class, or cell role as appropriate.
27. The rewrite should not create four replacement tiers.
28. The rewrite should preserve true infrastructure limits, especially the OCI Always Free profile for `demo_trial`.
29. Default severity: P2 for retirement cleanup, unless the tier reference gates runtime authorization or billing.
30. Runtime/billing tier references should be promoted to P1 during implementation because they can misauthorize tenants.

### 3.4.C Tenant-class adoption gaps

1. Required tenant classes for this audit: `demo_trial`, `paid`, and `revenue_share`.
2. Search result: no `tenant_class` string appears in the service path.
3. Search result: no `demo_trial` string appears in the service path.
4. Search result: no `revenue_share` string appears in the service path.
5. Search result: no per-seat or usage-based billing marker appears in the service path.
6. `src/config.rs:18-23` defines tenant config as tenant id, home cell, and data residency pack only.
7. `contracts/openapi-v1.yaml:132-176` schemas require tenant, principal, idempotency, payload, and compliance packs, but not tenant class.
8. `contracts/asyncapi-v1.yaml` payloads include tenant and event data, but not tenant class.
9. `PRD.md:1132` still sends `tenant_class` to finops instead of tenant class plus billing model.
10. `manifest.json:54-58` says tenant scope is required, but does not classify tenants.
11. Gap: the service cannot distinguish free demo-trial usage caps from paid contractual scaling or revenue-share at-cost substrate.
12. Gap: the service cannot express OCI Always Free profile as demo-trial infrastructure.
13. Gap: policy text still gates by tenant class instead of tenant class plus entitlement.
14. Gap: SLO text does not specify how demo-trial caps differ from paid and revenue-share scale envelopes.
15. Severity: P2 documentation and contract gap, rising to P1 before any billing or entitlement implementation.

### 3.5 Dimension 5 - Contract, API, and event coherence

1. Verdict: contract shells exist, but they are too generic and drift from source routes.
2. OpenAPI has six POST endpoints in `contracts/openapi-v1.yaml:15-129`.
3. HTTP source has five routes in `src/adapter/http.rs:30-62`.
4. The OpenAPI route for technician dispatch has no corresponding Rust HTTP route.
5. OpenAPI command schemas require `payload` as a generic object at `contracts/openapi-v1.yaml:132-176`.
6. A generic object cannot prove SAP PM equipment attributes, Maximo work-order labor/materials, or UpKeep mobile closeout fields.
7. AsyncAPI has event channels for six bounded contexts, which is positive.
8. Proto service exposes six RPCs at `contracts/plant-maintenance-v1.proto:94-100`, which is positive.
9. Rust domain exposes only five bounded contexts at `src/domain/mod.rs:48-54`.
10. The contract set is ahead of the source implementation.
11. Contract metadata uses stale benchmark rosters at `contracts/openapi-v1.yaml:9-14`.
12. Contract schemas do not expose tenant class.
13. Contract schemas do not expose deployment context.
14. Contract schemas do not expose OCI Always Free cap evidence.
15. Contract schemas do not expose technician skill matrix, permit-to-work, LOTO, meter readings, failure codes, or inventory reorder semantics.
16. The current contract layer is a useful starting envelope.
17. It is not yet a parity contract for SAP Plant Maintenance, IBM Maximo, and UpKeep.
18. Severity: P1.

### 3.6 Dimension 6 - Implementation and runtime coherence

1. Verdict: Rust scaffold is clean but incomplete.
2. Positive evidence: `src/lib.rs:1-8` keeps module boundaries small.
3. Positive evidence: `src/lib.rs:20-28` defines service package constants.
4. Positive evidence: `src/error.rs` centralizes service errors.
5. Gap evidence: `src/domain/mod.rs:20-26` omits `TechnicianDispatch`.
6. Gap evidence: `src/domain/mod.rs:48-64` omits `TechnicianDispatch` in both bounded context and capability enums.
7. Gap evidence: `src/domain/mod.rs:244-256` maps every capability descriptor to `BoundedContext::WorkOrder`.
8. Gap evidence: `src/adapter/http.rs:30-62` lacks a technician-dispatch route.
9. Gap evidence: `src/adapter/http.rs:65-67` returns a contract stub error for all HTTP handling.
10. Gap evidence: `src/config.rs:58-91` defaults to localhost dependencies and does not model deployment context.
11. Gap evidence: `src/config.rs:18-23` does not model tenant class.
12. Gap evidence: `tests/integration.rs:59-77` ignores contract, proto, AsyncAPI, Cedar, and repository fixture tests.
13. The implementation is internally tidy, but it cannot be called a complete PM runtime.
14. The most urgent runtime fix is to align the six bounded contexts across domain, routes, contracts, policy, tests, and catalog.
15. The second urgent runtime fix is to replace stub handlers with validation and usecase execution.
16. The third urgent runtime fix is to make capability descriptors map to their real bounded contexts.
17. Severity: P1.

### 3.7 Dimension 7 - Operational readiness, SLOs, capacity, and cost

1. Verdict: operational artifacts exist, but deployment overlays and benchmark provenance are incomplete.
2. Availability SLO exists at `slos/plant-maintenance-availability.openslo.yaml`.
3. Latency SLO exists at `slos/plant-maintenance-latency-p99.openslo.yaml`.
4. Throughput SLO exists at `slos/plant-maintenance-throughput.openslo.yaml`.
5. Equipment-master success SLO exists at `slos/equipment-master-success-rate.openslo.yaml`.
6. PRD telemetry states p50 under 120 ms, p95 under 300 ms, and p99 under 750 ms for lightweight mutations at `PRD.md:1088-1089` and matching context sections.
7. The p99 SLO file uses a 0.35-second threshold in the PromQL bucket at `slos/plant-maintenance-latency-p99.openslo.yaml:21`.
8. The PRD and SLO therefore need a reconciliation note: 350 ms bucket target versus 750 ms prose target.
9. Capacity model uses Little's Law and service-time assumptions at `capacity-model.md:13-23`.
10. PRD capacity states that a 300 ms p95 command at 1000 commands per second requires 300 worker slots at `PRD.md:1127-1130`.
11. Cost budget has unit-cost rows, but those rows are not mapped to tenant class or deployment context.
12. No performance benchmark report existed before this audit.
13. No `benchmarks/` directory exists.
14. No OCI Always Free throughput cap exists.
15. No on-prem or colo infrastructure constraint overlay exists.
16. Runbooks exist for approval deadletters, capacity saturation, settlement blocked, policy deny spike, regional failover, and source import stall.
17. These runbooks are valuable but not enough to prove six-context operability.
18. Severity: P2 for docs, P1 where SLO conflicts would mislead production sizing.

### 3.8 Dimension 8 - Security, compliance, privacy, and policy

1. Verdict: security and compliance posture is broader than runtime depth.
2. Policy files cover all six named contexts, including technician dispatch and downtime window.
3. PRD states Cedar is the only application authorization language at `PRD.md:938-942`.
4. Manifest tenant scope is required at `manifest.json:54-58`.
5. Manifest lists compliance packs at `manifest.json:59-67` and `manifest.json:271-280`.
6. The service has `dpia.md`, `threat-model.md`, `compliance.md`, and `policy/data-residency.md`.
7. Break-glass language exists in PRD policy prose at `PRD.md:955-956`.
8. Policy prose still uses tenant class checks at `PRD.md:945-955`.
9. Tenant class is absent from policy, contracts, and config.
10. Revenue-share tenants are not modeled as a commercial substrate state.
11. Demo-trial limitations are not modeled as policy, rate limit, or deployment profile.
12. BYOK/compliance-pack allowance is not connected to `paid` or `revenue_share`.
13. Audit-chain events are named for six contexts in manifest seal events at `manifest.json:235-244`.
14. The policy file count is good.
15. The policy semantics need Wave-15J retirement and tenant-class replacement.
16. Severity: P2, with P1 risk if stale tier gates reach runtime authorization.

### 3.9 Dimension 9 - Documentation substance, contradictions, and execution risk

1. Verdict: the service has many documents, but several are repetitive or contradict canonical direction.
2. Documentation substance problem: README evidence rows begin at `README.md:32-200` and repeat a pattern rather than teaching implementation.
3. Documentation substance problem: architecture evidence rows begin at `ARCHITECTURE.md:111-200` and repeat trace rows.
4. Documentation substance problem: `PHASE-01-PLANT-MAINTENANCE-PARITY.md` repeats parity evidence rows without resolving canonical blockers.
5. Contradiction: `AUDIT-FINDINGS-2026-05-21.json:3-8` says second-pass authored, while six-context IaC, OpenTofu, OS, tenant-class, and implementation gaps remain.
6. Contradiction: the prompt's counterpart set includes UpKeep, but manifest and PRD do not.
7. Contradiction: contracts expose six contexts, while source exposes five.
8. Contradiction: docs promise tenant-class activation while current doctrine retires tiers.
9. Contradiction: PRD p99 target and SLO bucket target do not match.
10. Execution risk: teams could implement against generic OpenAPI payloads and miss equipment attributes, meter schedules, labor plans, safety gates, and spare reservations.
11. Execution risk: teams could ship Kubernetes YAML and call it deployable without the six context IaC modules.
12. Execution risk: teams could retain tier terminology in telemetry and billing, blocking tenant-class migration.
13. Execution risk: teams could trust the audit JSON closure and skip blockers.
14. The remediation path is straightforward: align canonical artifacts first, then implementation.
15. Severity: P1 for contradictions that can produce wrong implementation; P2 for docs cleanup.

## 4. Findings Table

| ID | Severity | Finding | Evidence | Closure condition |
| --- | --- | --- | --- | --- |
| PM-AUD-P0-01 | P0 | Six deployable contexts are not declared or implemented for the service. | Canon requires six contexts at `specs/master-plan-sequencing.json:704-745`; service only says Kubernetes isolation at `manifest.json:161-164`; inventory has no context IaC dirs. | Add and verify all six deployment-context records and `iac/<context>/` OpenTofu modules or documented non-applicability approved by canonical source. |
| PM-AUD-P0-02 | P0 | OpenTofu substrate is missing and a Terraform-named module exists. | Canon requires OpenTofu at `specs/master-plan-sequencing.json:747-775`; service has `iac/terraform-module/main.tf`. | Replace Terraform-named substrate with OpenTofu module layout and `tofu init/plan/apply` evidence per context. |
| PM-AUD-P0-03 | P0 | Supported OS manifest is absent. | Canon requires OS matrix at `specs/master-plan-sequencing.json:777-815`; inventory has no `supported-oses.json`. | Add `supported-oses.json` with required tier-1, tier-2, excluded OS, arch, and validation posture. |
| PM-AUD-P1-01 | P1 | Technician dispatch is documented and contracted but missing from Rust domain and HTTP routes. | Context exists at `manifest.json:31-38`; source omits it at `src/domain/mod.rs:20-26` and `src/adapter/http.rs:30-62`. | Add typed technician-dispatch bounded context, capability, route, usecase, tests, and contract fixtures. |
| PM-AUD-P1-02 | P1 | Capability descriptors map every capability to work order. | `src/domain/mod.rs:244-256`. | Map each capability descriptor to its real bounded context and assert in tests. |
| PM-AUD-P1-03 | P1 | Runtime handlers and verification remain stubbed. | HTTP handler returns a stub at `src/adapter/http.rs:65-67`; key tests are ignored at `tests/integration.rs:59-77`. | Implement handler/usecase path and unignore passing contract, Cedar, AsyncAPI, proto, and repository tests. |
| PM-AUD-P1-04 | P1 | Counterpart roster drifts from assigned SAP / Maximo / UpKeep set. | Assigned by chat at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17216-17219`; service rosters differ at `manifest.json:25-29`, `PRD.md:55-60`, and `contracts/openapi-v1.yaml:9-14`. | Normalize counterpart metadata, feature parity, benchmark notes, and contract annotations to SAP Plant Maintenance / IBM Maximo / UpKeep. |
| PM-AUD-P1-05 | P1 | OpenAPI command payloads are generic and do not encode PM-specific invariants. | `contracts/openapi-v1.yaml:132-176`. | Replace generic payloads with typed schemas for equipment, plans, work orders, spare reservations, technician dispatch, and downtime windows. |
| PM-AUD-P1-06 | P1 | Cross-service handoffs are not captured in a dedicated handoff artifact. | Dependencies are listed at `manifest.json:39-46` and `manifest.json:282-288`; inventory has no `cross-microservice-handoffs.md`. | Add or generate a handoff artifact with ownership, contracts, failure modes, and verification links for warehouse, real-estate, workflow-engine, ontology, finops, and quality-management. |
| PM-AUD-P1-07 | P1 | Audit closure is overclaimed. | `AUDIT-FINDINGS-2026-05-21.json:3-8` says second-pass authored while P0 gaps remain. | Reopen audit status until canonical deployability, OpenTofu, OS, tenant-class, and source parity gaps are closed. |
| PM-AUD-P2-01 | P2 | Tenant-class semantics are absent. | No tenant-class scan hits; config has only tenant/home-cell/data-residency at `src/config.rs:18-23`; OpenAPI lacks class fields at `contracts/openapi-v1.yaml:132-176`. | Add `tenant_class` semantics for `demo_trial`, `paid`, and `revenue_share` in contracts, config, policy, billing, SLO overlays, and docs. |
| PM-AUD-P2-02 | P2 | Whole-path tier-language retirement backlog is large. | 401 line-level tier-language hits; examples at `manifest.json:7-8`, `manifest.json:136-140`, `PRD.md:1330-1412`, and all IP front matter line 8. | Retire tenant-class language under Wave 15J and replace with tenant class, deployment context, criticality class, or cell role. |
| PM-AUD-P2-03 | P2 | PRD section J is an obsolete tenant-class model. | `PRD.md:1330-1412`. | Replace Section J with tenant-class and entitlement semantics without adding feature tiers. |
| PM-AUD-P2-04 | P2 | Implementation-plan front matter perpetuates `tenant_class`. | `IP-001` through `IP-025` line 8. | Replace front-matter field with accepted non-tier metadata. |
| PM-AUD-P2-05 | P2 | Capacity model uses retired tier assumptions. | `capacity-model.md:16-23`. | Rewrite load envelopes as single industry-leader target with deployment-context and tenant-class overlays. |
| PM-AUD-P2-06 | P2 | Telemetry uses `tier` label dimensions. | `PRD.md:1068`, `1078`, `1088`, `1098`, `1108`, `1118`. | Replace telemetry labels with bounded `tenant_class`, `deployment_context`, and `cell_role` dimensions. |
| PM-AUD-P2-07 | P2 | Manifest embeds tier and criticality tier fields. | `manifest.json:7-8`, `manifest.json:253-255`, `manifest.json:269`, `manifest.json:290-291`. | Update manifest schema usage to non-tier terms and preserve real criticality separately. |
| PM-AUD-P2-08 | P2 | OCI Always Free profile is missing. | Canon path at `specs/master-plan-sequencing.json:857-868`; inventory has no `iac/oci-guest/always-free/`. | Add OCI Always Free profile tied to `demo_trial` infrastructure caps. |
| PM-AUD-P2-09 | P2 | Capability YAML records cover only half the six contexts. | PRD lists only three records at `PRD.md:64-68`; inventory confirms three capability files. | Add or generate records for spare-part reservation, technician dispatch, and downtime window. |
| PM-AUD-P2-10 | P2 | Several docs read as repeated scaffolds instead of implementer instructions. | `README.md:32-200`, `ARCHITECTURE.md:111-200`, and companion docs repeat evidence rows. | Replace row repetition with command examples, schemas, validation cases, and handoff-specific guidance. |
| PM-AUD-P2-11 | P2 | SLO and benchmark rosters are stale. | OpenAPI roster at `contracts/openapi-v1.yaml:9-14`; SLO descriptions follow stale competitor groups. | Align SLO and benchmark source notes to SAP Plant Maintenance / IBM Maximo / UpKeep. |
| PM-AUD-P2-12 | P2 | No benchmark directory or raw performance appendix exists. | Inventory has no `benchmarks/` directory. | Keep this audit report or add a canonical benchmark artifact with methodology, data provenance, and context overlays. |
| PM-AUD-P2-13 | P2 | Onboarding, migration, tutorial, FAQ, and reference implementation directories are absent. | Inventory has no `faqs/`, `onboarding/`, `migration-playbooks/`, `reference-implementations/`, or `tutorials/`. | Add targeted operator and integrator material after contracts stabilize. |
| PM-AUD-P2-14 | P2 | Billing and finops still use tenant-class dimensions instead of tenant class. | `PRD.md:1132`. | Emit tenant class, usage dimension, deployment context, and commercial model to finops. |
| PM-AUD-P3-01 | P3 | Rust-strict extension posture passes. | Forbidden extension scan returned no Python, JS, TS, Ruby, Go, Java, Scala, Groovy, PHP, or F# source files. | Keep Rust-only backend posture; add checks to CI. |
| PM-AUD-P3-02 | P3 | Cedar policy file set exists across contexts. | Policy inventory contains equipment, plan, work-order, spare, technician-dispatch, downtime, tenant, auditor, CI, and emergency files. | Connect policies to executable fixtures and tenant-class semantics. |
| PM-AUD-P3-03 | P3 | SLO files exist for core operations. | Four OpenSLO files exist under `slos/`. | Reconcile thresholds and add context overlays. |
| PM-AUD-P3-04 | P3 | REST, event, and proto contracts exist. | Contract files exist under `contracts/`. | Replace generic payloads and align with source. |
| PM-AUD-P3-05 | P3 | Catalog covers six context-layer combinations. | Inventory includes catalog files for downtime, equipment, maintenance, spare, technician-dispatch, and work-order layers. | Bind catalog entries to real source modules and verification. |
| PM-AUD-P3-06 | P3 | Runbook family exists. | Six runbooks exist under `runbooks/`. | Link runbooks to SLO alerts, deployment contexts, and tested drills. |

### 4.1 Severity totals

1. P0 findings: 3.
2. P1 findings: 7.
3. P2 findings: 14.
4. P3 findings: 6.
5. Total findings: 30.
6. P0 blockers are canonical deployability blockers.
7. P1 blockers are product/source coherence blockers.
8. P2 blockers are doctrine, documentation, and adoption gaps.
9. P3 findings are positive assets that need linkage.
10. No P0 finding is caused by missing desire; all are caused by missing evidence.

## 5. Open Questions

1. Should `technician-dispatch` be implemented as a full bounded context in Rust, or should the docs/contracts be narrowed to remove it?
2. Should `permit-to-work`, LOTO, condition-based maintenance, RCM, Weibull analytics, and KPI scorecards become first-class capabilities or remain future IP slices?
3. Which team owns the six deployment-context OpenTofu modules for this service: plant-maintenance, cloud-iac, or a shared generator?
4. What is the authoritative manifest field replacing `tier`, `tier_subtype`, `criticality_tier`, and `tenant_classes`?
5. What is the authoritative policy context shape for `tenant_class` across Cedar, OpenAPI, AsyncAPI, proto, config, and finops events?
6. Should the p99 target be 350 ms, matching the OpenSLO bucket, or 750 ms, matching PRD prose?
7. Should UpKeep mobile/offline/requester workflows become explicit requirements in the PRD or remain additive parity opportunities?
8. Should Maximo inventory and scheduler depth be modeled inside this microservice or delegated to warehouse and workflow-engine?
9. Should the old `AUDIT-FINDINGS-2026-05-21.json` status be changed in a follow-up slice after these reports land?
10. Should the service add `benchmarks/` as a directory, or should this audit's performance report be promoted as the benchmark anchor?
11. Should `demo_trial` be deployable only through OCI Always Free, or also through oyatie-public-cloud with equivalent hard usage caps?
12. Should `revenue_share` tenants receive paid-class technical SLOs by default, with commercial settlement handled only by marketplace and finops?
13. Should tenant class appear in telemetry as a bounded label, or should it be attached only to audit and billing events to avoid cardinality risk?
14. Should the capability YAML count match the six bounded contexts before any new deep EAM IP slices begin?
15. Should cross-microservice handoffs be authored as a single file or generated from manifest dependency declarations?
16. Should source-level route generation be derived from OpenAPI to prevent another five-versus-six context drift?
17. Should ignored tests in `tests/integration.rs` become the first remediation gate?
18. Should old competitor references to Oracle, Workday, NetSuite, Microsoft, and Infor be removed or kept as secondary references after the assigned top-three set?
19. Should `IEEE Std 493` in `IP-022` be left untouched because it is an IEEE standards nickname and not a feature tier?
20. Should this service adopt a runtime contract that rejects unknown deployment context and tenant class before accepting any command?

## 6. Evidence-Bound Closure Plan

1. Closure lane 1: canonical deployability must land before the service can move from scaffolded artifact to deployable product.
2. Closure lane 1 evidence to add: `iac/oyatie-public-cloud/`, `iac/aws-guest/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-as-cloud-provider/`.
3. Closure lane 1 evidence to add: `iac/oci-guest/always-free/` with demo-trial caps and explicit 4 OCPU plus 24 GB memory assumptions.
4. Closure lane 1 evidence to add: OpenTofu module files, variables, outputs, provider pins, and state-backend notes.
5. Closure lane 1 evidence to remove: Terraform-named module path or any implication that Terraform is the service substrate.
6. Closure lane 2: OS support must become a checked manifest, not prose.
7. Closure lane 2 evidence to add: `supported-oses.json` with required tier-1 OS rows, arch rows, exclusion rows, and validation status.
8. Closure lane 2 evidence to add: a test or static check proving the manifest is present and schema-valid.
9. Closure lane 3: product shape must be six-context consistent.
10. Closure lane 3 evidence to add: `TechnicianDispatch` in `BoundedContext`, `Capability`, route registry, usecase path, tests, and fixtures.
11. Closure lane 3 evidence to add: descriptor tests proving each capability maps to the correct bounded context.
12. Closure lane 3 evidence to add: contract-to-route checks so OpenAPI, AsyncAPI, proto, source, policy, and catalog cannot drift silently.
13. Closure lane 4: contracts must stop accepting opaque PM payloads.
14. Closure lane 4 evidence to add: typed schemas for equipment hierarchy, maintenance plan schedule, work-order operation, spare reservation, technician assignment, and downtime window.
15. Closure lane 4 evidence to add: fixture examples for source-system import, idempotency, Cedar deny, audit seal, and replay.
16. Closure lane 5: Wave-15J retirement must be local and mechanical before policy or billing changes.
17. Closure lane 5 evidence to add: zero new tier-language hits except non-feature standards references.
18. Closure lane 5 evidence to add: manifest fields replacing old tier fields with tenant class, deployment context, cell role, and criticality class where appropriate.
19. Closure lane 5 evidence to add: policy prose and telemetry labels that do not depend on retired tenant-class gates.
20. Closure lane 6: tenant-class adoption must be explicit in contracts, config, policy, finops, SLO overlays, and benchmark reports.
21. Closure lane 6 evidence to add: `tenant_class` validation for `demo_trial`, `paid`, and `revenue_share`.
22. Closure lane 6 evidence to add: usage caps for `demo_trial`, contract scaling for `paid`, and at-cost substrate rules for `revenue_share`.
23. Closure lane 6 evidence to add: all three tenant classes retain the same product correctness and audit-integrity guarantees.
24. Closure lane 7: counterpart rosters must be normalized.
25. Closure lane 7 evidence to add: SAP Plant Maintenance, IBM Maximo, and UpKeep in manifest, PRD, contracts, SLO descriptions, parity matrix, and benchmark report.
26. Closure lane 7 evidence to remove or demote: stale primary rosters centered on Oracle, Workday, NetSuite, Microsoft, or Infor.
27. Closure lane 8: UpKeep-style mobile/frontline workflows need a deliberate product decision.
28. Closure lane 8 evidence to add: mobile/offline/requester fields or an explicit out-of-scope decision with rationale.
29. Closure lane 8 evidence to add: technician assignment and closeout evidence as first-class workflows if UpKeep remains a top-three benchmark.
30. Closure lane 9: Maximo-style inventory and scheduling depth need either implementation or handoff contracts.
31. Closure lane 9 evidence to add: warehouse handoff for parts inventory, bins, lots, reorder points, POs, and vendor state.
32. Closure lane 9 evidence to add: workflow-engine handoff for scheduling, approvals, technician queues, and dispatch changes.
33. Closure lane 10: existing audit closure state should be treated as stale.
34. Closure lane 10 evidence to add: a successor audit status that references this report and names the open P0/P1 blockers.
35. Closure lane 11: performance targets should be measured only after runtime exists.
36. Closure lane 11 evidence to add: a benchmark harness that emits p50, p95, p99, throughput, batch import, audit seal, policy decision, event lag, and projection lag.
37. Closure lane 11 evidence to add: each benchmark result must name OS, arch, deployment context, tenant class, cell count, storage class, and fixture set.
38. Closure lane 12: docs should be rewritten for buildability, not line volume.
39. Closure lane 12 evidence to add: compact implementer instructions, command examples, schema examples, failure drills, and handoff examples.
40. Closure lane 12 evidence to remove: repeated trace rows that do not help an implementer build or verify behavior.
41. Closure lane 13: no deliverable should be treated as complete by line count alone.
42. Closure lane 13 evidence to add: line count, source citations, contradiction list, and executable validation status in each future audit artifact.
43. Closure lane 14: no commit is required for this audit; the landing evidence is the three uncommitted report files and the line-count verification.
44. Closure lane 15: clean handoff state is achieved when the orchestrator report below contains exact line counts and no pending placeholders.
45. Closure lane 16: the stop condition is satisfied only after the final verification commands show all three report files over their line floors.

<!-- ORCHESTRATOR REPORT
  µservice: plant-maintenance
  deliverables_landed: microservices/plant-maintenance/coherence-audit-2026-05-20.md=623 lines; microservices/plant-maintenance/feature-parity-matrix-2026-05-20.md=435 lines; microservices/plant-maintenance/performance-benchmark-numbers-2026-05-20.md=309 lines
  inventory_files_seen: 151
  inventory_lines_read: 20250
  chat_history_matches_processed: 6
  findings_p0: 3
  findings_p1: 7
  findings_p2: 14
  findings_p3: 6
  tier_retirement_candidates_found: 401 tier-language line hits; exact color-tier feature hits 0; non-feature IEEE Std 493 standards citation excluded at IP-022-mtbf-weibull-fitting-reliability-analytics.md:328; key retirement citations manifest.json:7-8, manifest.json:136-140, manifest.json:253-255, manifest.json:269, manifest.json:290-291, Cargo.toml:18, PRD.md:31, PRD.md:36, PRD.md:46, PRD.md:854, PRD.md:941, PRD.md:945-955, PRD.md:1038, PRD.md:1058, PRD.md:1068, PRD.md:1078, PRD.md:1088, PRD.md:1098, PRD.md:1108, PRD.md:1118, PRD.md:1132, PRD.md:1330-1412, capacity-model.md:16-23, IP-001..IP-025:8
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/paid/revenue_share semantics found in contracts, config, policy, SLOs, or billing events
  top_3_counterparts_confirmed: SAP Plant Maintenance / IBM Maximo / UpKeep
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1367
-->
