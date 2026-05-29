# Global Trade Ownership-Coherence Audit - 2026-05-20

Audit owner: single-agent audit lane for `global-trade`.
Target path: `microservices/global-trade/`.
Service purpose under audit: global trade compliance, customs, screening, export-control, trade-document, denied-party, and broker-filing ownership.
Assigned counterpart bar: SAP Global Trade Services, Thomson Reuters ONESOURCE Global Trade, Descartes.
Deliverable set for this batch: three reports only.
Retired deliverable: tenant-class deltas report is not authored because the tenant-class model is retired.
New tenant-class model used for this audit: `demo_trial`, `paid`, `revenue_share`.
Quality bar: uniform industry-leader grade across tenant classes, not stratified by old feature tiers.
Evidence posture: every finding below cites local file lines, canonical-memory lines, chat-history lines, or inspected public counterpart sources.

## Citation Anchors

- Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1749` requires the six deployment contexts and per-context IaC evidence.
- Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2249` requires OpenTofu wording and treats Terraform as superseded/forbidden except as historical reference.
- Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907-2928` requires per-service OS support manifest evidence.
- Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3047-3067` binds backend/runtime/CLI/validation/codegen/scripting/CI to Rust unless an explicit exception exists.
- Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3493-3499` defines OCI Always Free as a sub-profile of guest-on-oci, not a separate architecture.
- Master plan: `specs/master-plan-sequencing.json:704-745` lists the six deployment contexts and IaC target paths.
- Master plan: `specs/master-plan-sequencing.json:747-775` binds IaC to OpenTofu and forbids Terraform, Pulumi, CloudFormation, ARM/Bicep, shell bootstrapping, and manual console provisioning.
- Master plan: `specs/master-plan-sequencing.json:777-815` binds the OS support matrix and requires per-service `supported-oses.json`.
- Master plan: `specs/master-plan-sequencing.json:817-855` binds Rust backend and the frontend allowlist.
- Brief template: `docs/standards/brief-template.md:666-807` requires multi-context claims to include explicit supported contexts and IaC references or remediation.
- Brief template: `docs/standards/brief-template.md:809-965` requires OpenTofu module evidence, state backend evidence, provider pins, signing, and forbidden-pattern scans.
- Brief template: `docs/standards/brief-template.md:967-1123` requires OS support evidence and forbids Python/Node/shell runtime prerequisites.
- Brief template: `docs/standards/brief-template.md:1125-1304` requires Rust-first backend evidence and exception-path disclosure.
- Brief template: `docs/standards/brief-template.md:1720-1854` identifies scaffold-without-substance, line-count-as-completion, recycled boilerplate, and soft contradiction as audit anti-patterns.
- Memory: `feedback_no_tenant_classes_2026_05_20.md:10-24` retires the tenant-class system.
- Memory: `feedback_no_tenant_classes_2026_05_20.md:28-45` says old tenant-class references must become retirement targets, not new report structure.
- Memory: `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139-142` says Wave 3 Batch 3.2 drops the tier-delta deliverable and performance docs must use single target sets plus overlays.
- Memory: `feedback_microservice_ownership_coherence_2026_05_20.md:18-46` requires full microservice inventory and artifact-scope audit.
- Memory: `feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-47` requires deliverable substance verification beyond line count.
- Memory: `feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` rejects scaffold as a substitute for substantive evidence.
- Chat history: `.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8869` identifies `global-trade` as the new microservice for SAP GTS export controls and sanctions screening.
- Chat history: `.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8899` maps global-trade to SAP GTS / Thomson Reuters ONESOURCE / Amber Road in an older ERP coverage matrix.
- Chat history: `.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17219` dispatches this service against SAP GTS, Thomson Reuters ONESOURCE Global Trade, and Descartes.

## 1. Purpose

1. This audit tests whether `global-trade` has a coherent service ownership story.
2. The audited ownership story is not just "SAP GTS parity"; it must cover the union of SAP Global Trade Services, Thomson Reuters ONESOURCE Global Trade, and Descartes.
3. The service must be understandable as a product surface, a runtime surface, and an operations surface.
4. The service must not claim deployability that is not backed by six-context OpenTofu evidence.
5. The service must not retain old feature-tier language as if it is still the product model.
6. The service must expose tenant-class semantics for `demo_trial`, `paid`, and `revenue_share`, or this audit must call out the gap.
7. The service must keep industry-leader quality uniform across tenant classes.
8. The service must not use the old tier-delta deliverable shape.
9. The service must align backend and tooling to Rust-strict policy.
10. The service must show OS portability evidence for the supported OS matrix.
11. The service must show OCI Always Free profile evidence for demo/trial infrastructure where guest-on-oci is in scope.
12. The service must avoid hard-coded provider assumptions because canonical direction requires provider-agnostic multi-context deployment.
13. The audit therefore examines product purpose, inventory completeness, ownership boundaries, canonical-direction alignment, counterpart parity, implementation readiness, operations readiness, compliance/security readiness, and launch verification.
14. The audit is read-only with respect to existing artifacts; the only authored files are the three required reports.
15. The audit found no exact local references to the four retired named feature tiers.
16. The audit found 509 generic old tier-model references across 12 local files.
17. The audit found zero `tenant_class`, `demo_trial`, or `revenue_share` references in the service path.
18. The audit found `paid` only as a duty-drawback claim status, not as a tenant-class model.
19. The audit found no `supported-oses.json`.
20. The audit found no `iac/oci-guest/always-free/` module.
21. The audit found no per-context OpenTofu module directories.
22. The audit found one path named `iac/terraform-module/main.tf`, which conflicts with the OpenTofu naming doctrine.
23. The audit found no forbidden Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, F#, or similar implementation files.
24. The audit found no Rust source files in `src/`.
25. The audit found no test files in `tests/`.
26. The audit found existing contracts for the original six bounded contexts.
27. The audit found expanded implementation plans for additional global-trade capabilities that are not yet reflected in the public API contract set.
28. The audit found a current comparator mismatch: local documents emphasize Oracle, Workday, NetSuite, and Microsoft, while this batch requires Thomson Reuters and Descartes.
29. The audit found useful domain direction in the ADR and PRD.
30. The audit also found high-volume repetitive rows that satisfy line count but weaken product specificity.

## 2. Inventory

Inventory command evidence: `find microservices/global-trade -type f | wc -l` returned 138 files.
Existing line-count evidence: `wc -l` across the service path returned 14,657 lines before these audit deliverables.
Directory evidence: `src/` and `tests/` exist but have no files.
Capability-tier directory evidence: no `microservices/global-trade/tenant-classs/` directory exists.
OS manifest evidence: no `microservices/global-trade/supported-oses.json` file exists.
OCI Always Free evidence: no `microservices/global-trade/iac/oci-guest/always-free/` path exists.
IaC evidence: `microservices/global-trade/iac/terraform-module/main.tf` exists and is the only HCL-like module path.
Forbidden language scan evidence: no `.py`, `.js`, `.ts`, `.tsx`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or `.fsx` files were found under this service path.

### 2.1 Complete File Inventory

1. `microservices/global-trade/ARCHITECTURE.md`
2. `microservices/global-trade/AUDIT-FINDINGS-2026-05-21.json`
3. `microservices/global-trade/CHANGELOG.md`
4. `microservices/global-trade/IP-001-domain-layer-for-customs-declaration.md`
5. `microservices/global-trade/IP-002-domain-layer-for-sanctions-screening.md`
6. `microservices/global-trade/IP-003-domain-layer-for-export-control-classification.md`
7. `microservices/global-trade/IP-004-domain-layer-for-trade-document.md`
8. `microservices/global-trade/IP-005-domain-layer-for-denied-party-hit.md`
9. `microservices/global-trade/IP-006-domain-layer-for-broker-filing.md`
10. `microservices/global-trade/IP-007-usecase-layer-for-customs-declaration.md`
11. `microservices/global-trade/IP-008-usecase-layer-for-sanctions-screening.md`
12. `microservices/global-trade/IP-009-usecase-layer-for-export-control-classification.md`
13. `microservices/global-trade/IP-010-usecase-layer-for-trade-document.md`
14. `microservices/global-trade/IP-011-usecase-layer-for-denied-party-hit.md`
15. `microservices/global-trade/IP-012-usecase-layer-for-broker-filing.md`
16. `microservices/global-trade/IP-013-adapter-integrations-for-global-trade.md`
17. `microservices/global-trade/IP-014-rest-grpc-and-worker-surfaces-for-global-trade.md`
18. `microservices/global-trade/IP-015-integration-tests-for-global-trade.md`
19. `microservices/global-trade/IP-016-hs-code-classification-engine-with-fta-preference-attach.md`
20. `microservices/global-trade/IP-017-denied-party-screening-lookup-with-cedar-consent.md`
21. `microservices/global-trade/IP-018-document-compliance-certificate-generation.md`
22. `microservices/global-trade/IP-019-broker-edi-ingestion-cusdec.md`
23. `microservices/global-trade/IP-020-duty-drawback-claim-workflow.md`
24. `microservices/global-trade/IP-021-quota-management-with-country-of-origin-tracking.md`
25. `microservices/global-trade/IP-022-embargo-event-audit-chain-anchor.md`
26. `microservices/global-trade/IP-023-preferential-trade-agreement-origin-determination-logic.md`
27. `microservices/global-trade/PHASE-01-GLOBAL-TRADE-PARITY.md`
28. `microservices/global-trade/PRD.md`
29. `microservices/global-trade/README.md`
30. `microservices/global-trade/backfill-replay.md`
31. `microservices/global-trade/capabilities/customs-declaration-command.yaml`
32. `microservices/global-trade/capabilities/export-control-classification-export.yaml`
33. `microservices/global-trade/capabilities/sanctions-screening-reconcile.yaml`
34. `microservices/global-trade/capacity-model.md`
35. `microservices/global-trade/catalog/oya-global-trade-broker-filing-adapter.yaml`
36. `microservices/global-trade/catalog/oya-global-trade-broker-filing-api.yaml`
37. `microservices/global-trade/catalog/oya-global-trade-broker-filing-application.yaml`
38. `microservices/global-trade/catalog/oya-global-trade-broker-filing-domain.yaml`
39. `microservices/global-trade/catalog/oya-global-trade-broker-filing-governance.yaml`
40. `microservices/global-trade/catalog/oya-global-trade-broker-filing-kernel.yaml`
41. `microservices/global-trade/catalog/oya-global-trade-broker-filing-rest.yaml`
42. `microservices/global-trade/catalog/oya-global-trade-broker-filing-usecase.yaml`
43. `microservices/global-trade/catalog/oya-global-trade-broker-filing-worker.yaml`
44. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-adapter.yaml`
45. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-api.yaml`
46. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-application.yaml`
47. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-domain.yaml`
48. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-governance.yaml`
49. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-kernel.yaml`
50. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-rest.yaml`
51. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-usecase.yaml`
52. `microservices/global-trade/catalog/oya-global-trade-customs-declaration-worker.yaml`
53. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-adapter.yaml`
54. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-api.yaml`
55. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-application.yaml`
56. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-domain.yaml`
57. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-governance.yaml`
58. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-kernel.yaml`
59. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-rest.yaml`
60. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-usecase.yaml`
61. `microservices/global-trade/catalog/oya-global-trade-denied-party-hit-worker.yaml`
62. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-adapter.yaml`
63. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-api.yaml`
64. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-application.yaml`
65. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-domain.yaml`
66. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-governance.yaml`
67. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-kernel.yaml`
68. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-rest.yaml`
69. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-usecase.yaml`
70. `microservices/global-trade/catalog/oya-global-trade-export-control-classification-worker.yaml`
71. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-adapter.yaml`
72. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-api.yaml`
73. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-application.yaml`
74. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-domain.yaml`
75. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-governance.yaml`
76. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-kernel.yaml`
77. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-rest.yaml`
78. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-usecase.yaml`
79. `microservices/global-trade/catalog/oya-global-trade-sanctions-screening-worker.yaml`
80. `microservices/global-trade/catalog/oya-global-trade-trade-document-adapter.yaml`
81. `microservices/global-trade/catalog/oya-global-trade-trade-document-api.yaml`
82. `microservices/global-trade/catalog/oya-global-trade-trade-document-application.yaml`
83. `microservices/global-trade/catalog/oya-global-trade-trade-document-domain.yaml`
84. `microservices/global-trade/catalog/oya-global-trade-trade-document-governance.yaml`
85. `microservices/global-trade/catalog/oya-global-trade-trade-document-kernel.yaml`
86. `microservices/global-trade/catalog/oya-global-trade-trade-document-rest.yaml`
87. `microservices/global-trade/catalog/oya-global-trade-trade-document-usecase.yaml`
88. `microservices/global-trade/catalog/oya-global-trade-trade-document-worker.yaml`
89. `microservices/global-trade/competitor-parity-matrix.md`
90. `microservices/global-trade/compliance.md`
91. `microservices/global-trade/contracts/asyncapi-v1.yaml`
92. `microservices/global-trade/contracts/global-trade-v1.proto`
93. `microservices/global-trade/contracts/openapi-v1.yaml`
94. `microservices/global-trade/cost-budget.md`
95. `microservices/global-trade/dashboards/customs-declaration-health.json`
96. `microservices/global-trade/dashboards/global-trade-overview.json`
97. `microservices/global-trade/dashboards/sanctions-screening-residency.md`
98. `microservices/global-trade/decisions/ADR-GT-001-sanctions-export-control-and-broker-filing-hold-state-machine.md`
99. `microservices/global-trade/dpia.md`
100. `microservices/global-trade/failure-modes.md`
101. `microservices/global-trade/iac/ech-config.yaml`
102. `microservices/global-trade/iac/edge-waf.yaml`
103. `microservices/global-trade/iac/helm-values.yaml`
104. `microservices/global-trade/iac/k8s-deployment.yaml`
105. `microservices/global-trade/iac/network-policy.yaml`
106. `microservices/global-trade/iac/openbao-policy.hcl`
107. `microservices/global-trade/iac/pqc-cert.yaml`
108. `microservices/global-trade/iac/secret-bindings.yaml`
109. `microservices/global-trade/iac/terraform-module/main.tf`
110. `microservices/global-trade/incident-response.md`
111. `microservices/global-trade/manifest.json`
112. `microservices/global-trade/multi-region.md`
113. `microservices/global-trade/policy/abuse-defence.cedar`
114. `microservices/global-trade/policy/auditor-scope.cedar`
115. `microservices/global-trade/policy/broker-filing-authorization.cedar`
116. `microservices/global-trade/policy/ci-scope.cedar`
117. `microservices/global-trade/policy/customs-declaration-authorization.cedar`
118. `microservices/global-trade/policy/data-residency.md`
119. `microservices/global-trade/policy/denied-party-hit-authorization.cedar`
120. `microservices/global-trade/policy/emergency-services-bypass.cedar`
121. `microservices/global-trade/policy/export-control-classification-authorization.cedar`
122. `microservices/global-trade/policy/pack-overlay-authorization.cedar`
123. `microservices/global-trade/policy/sanctions-screening-authorization.cedar`
124. `microservices/global-trade/policy/tenant-isolation.md`
125. `microservices/global-trade/policy/trade-document-authorization.cedar`
126. `microservices/global-trade/runbooks/approval-deadletter.md`
127. `microservices/global-trade/runbooks/capacity-saturation.md`
128. `microservices/global-trade/runbooks/marketplace-settlement-blocked.md`
129. `microservices/global-trade/runbooks/policy-deny-spike.md`
130. `microservices/global-trade/runbooks/regional-failover.md`
131. `microservices/global-trade/runbooks/source-import-stalled.md`
132. `microservices/global-trade/scorecards/overrides.json`
133. `microservices/global-trade/sdk-plan.md`
134. `microservices/global-trade/slos/customs-declaration-success-rate.openslo.yaml`
135. `microservices/global-trade/slos/global-trade-availability.openslo.yaml`
136. `microservices/global-trade/slos/global-trade-latency-p99.openslo.yaml`
137. `microservices/global-trade/slos/global-trade-throughput.openslo.yaml`
138. `microservices/global-trade/threat-model.md`

### 2.2 Inventory Coverage Notes

1. Primary docs are present: `PRD.md`, `ARCHITECTURE.md`, `README.md`, `PHASE-01-GLOBAL-TRADE-PARITY.md`, `competitor-parity-matrix.md`.
2. One service-local ADR is present under `decisions/`.
3. Twenty-three implementation plans are present.
4. Three contract files are present.
5. Four OpenSLO files are present.
6. Three capability YAML files are present.
7. Fifty-four catalog YAML files are present.
8. Thirteen policy files are present, including eleven Cedar files and two policy docs.
9. Six runbooks are present.
10. Nine IaC-related files are present.
11. The requested `cross-microservice-handoffs.md` file is absent.
12. The requested `benchmarks/` directory is absent.
13. The requested `faqs/` directory is absent.
14. The requested `onboarding/` directory is absent.
15. The requested `migration-playbooks/` directory is absent.
16. The requested `reference-implementations/` directory is absent.
17. The requested `tutorials/` directory is absent.
18. The requested `tenant-classs/` directory is absent, which is acceptable as a directory-retirement state, but generic old tier fields remain elsewhere.
19. No source files are present under `src/`.
20. No test files are present under `tests/`.

## 3. Nine-Dimension Audit

### 3.1 Dimension 1 - Product Purpose And Ownership

1. Current PRD purpose is clear at the top level: `PRD.md:24-32` says Global Trade provides SAP GTS parity for customs declarations, sanctions screening, export controls, trade documents, denied-party hits, and broker filing.
2. README purpose is consistent with that scope: `README.md:11-14` frames SAP GTS parity across Customs Management, Sanctioned Party Screening, Export Control, and Trade Compliance.
3. Architecture boundary is strong in one place: `ARCHITECTURE.md:19-22` says the service owns customs declarations, sanctioned-party screening, export controls, denied-party evidence, and trade-compliance holds.
4. The actual purpose is broader than the current six original bounded contexts because IP-016 through IP-023 add HS classification, denied-party lookup, certificates, broker EDI, duty drawback, quota/origin controls, embargo audit chains, and preferential trade agreement origin logic.
5. The expanded purpose matches industry global-trade management better than the original six contexts alone.
6. The service therefore has a coherent product center, but the authoritative docs lag the expanded implementation-plan surface.
7. The product owner should treat global-trade as the trade compliance operating service, not merely a SAP GTS clone.
8. The service should own compliance decision evidence, not generalized marketplace settlement, tenant identity, payments, workflow engine state, or ontology storage.
9. The architecture makes those non-ownership boundaries explicit at `ARCHITECTURE.md:19-22`.
10. The PRD non-goals confirm the same separation at `PRD.md:49-52`.
11. The chat-history seed at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8869` is consistent with the SAP GTS export-control and sanctions-screening purpose.
12. The later dispatch at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17219` upgrades the counterpart bar to SAP, Thomson Reuters, and Descartes.
13. Finding impact: current product purpose is directionally valid.
14. Finding impact: current product purpose is incompletely normalized against the assigned top-three counterpart set.
15. Finding impact: old SAP-only and Oracle/Workday/NetSuite/Microsoft rows obscure Thomson Reuters and Descartes coverage.

### 3.2 Dimension 2 - Artifact Inventory And Evidence Quality

1. Inventory breadth is high, with 138 files and 14,657 pre-audit lines.
2. Inventory substance is uneven.
3. `manifest.json:68-95` claims a follow-up artifact roster that includes many files now present.
4. `AUDIT-FINDINGS-2026-05-21.json:5-7` marks the second pass as authored with a 100 artifact target.
5. `AUDIT-FINDINGS-2026-05-21.json:10-62` closes six doc-set findings by pointing to a narrow subset of policies, catalog records, and one runbook.
6. That closure evidence is not enough for this audit because five canonical constraint dimensions remain unproven.
7. `README.md:32-200` is dominated by repeated "README evidence row" style content and outdated comparator rosters.
8. `ARCHITECTURE.md:111-200` is dominated by repeated trace rows rather than concrete architecture detail.
9. `competitor-parity-matrix.md:24-220` is a repeated row matrix that swaps counterpart names and bounded contexts without proving capability depth.
10. `capacity-model.md:26-220` repeats generated capacity rows with the same outdated counterpart roster.
11. This is the exact anti-pattern called out by `docs/standards/brief-template.md:1720-1854`.
12. The artifact set is large enough to look complete but not yet strong enough to prove ownership coherence.
13. Positive evidence: the service does contain contracts, policies, SLOs, runbooks, dashboards, catalog files, and capacity docs.
14. Negative evidence: the docs do not prove deployable context readiness, OS portability, tenant-class semantics, or implemented Rust surfaces.
15. Finding impact: document volume should not be interpreted as service readiness.

### 3.3 Dimension 3 - Ownership Boundaries And Cross-Service Handoffs

1. Architecture names integration dependencies at `ARCHITECTURE.md:80-87`: marketplace, warehouse, connect, compliance, payments, and supply-chain-planning.
2. Manifest repeats those dependencies at `manifest.json:282-288`.
3. PRD non-goals keep tenant identity, payments, and workflow execution outside this service at `PRD.md:49-52`.
4. That is a coherent boundary.
5. The service lacks the requested `cross-microservice-handoffs.md`.
6. The absence is material because trade holds, broker filing, duty drawback, and settlement references all cross service boundaries.
7. `contracts/openapi-v1.yaml:146-147`, `164-165`, `182-183`, `200-201`, `218-219`, and `236-237` include `marketplace_settlement_ref` but do not specify cross-service handoff semantics.
8. `contracts/asyncapi-v1.yaml:54-55`, `73-74`, `92-93`, `111-112`, and `130-131` include settlement refs in events but do not specify settlement failure or reconciliation ownership.
9. `IP-020-duty-drawback-claim-workflow.md:112` says paid status can be reconciled with finance, but finance posting remains outside this service.
10. That is a useful boundary statement, but it lives in one implementation plan and not the canonical cross-service handoff doc.
11. `IP-020-duty-drawback-claim-workflow.md:280-281` names finance and notification handoffs.
12. The service needs one canonical handoff map for all dependencies.
13. Without it, marketplace, compliance, payments, warehouse, connect, and supply-chain-planning integration failures remain scattered across docs.
14. Finding impact: ownership boundaries are conceptually good but operational handoff evidence is incomplete.

### 3.4 Dimension 4 - Canonical-Direction Alignment

1. Multi-context doctrine requires six deployable contexts or explicit N/A rows, per `ADR-0328:1732-1749` and `ADR-0328:2079-2084`.
2. `global-trade` has no per-context IaC directories.
3. Existing IaC files are flat under `iac/`.
4. Existing IaC files are `ech-config.yaml`, `edge-waf.yaml`, `helm-values.yaml`, `k8s-deployment.yaml`, `network-policy.yaml`, `openbao-policy.hcl`, `pqc-cert.yaml`, `secret-bindings.yaml`, and `terraform-module/main.tf`.
5. The missing expected directories are `iac/oyatie-public-cloud`, `iac/guest-on-aws`, `iac/guest-on-oci`, `iac/on-prem`, `iac/colo`, and `iac/oyatie-iaas`.
6. The missing OCI profile directory is `iac/oci-guest/always-free`.
7. The only HCL-like module is under `iac/terraform-module/main.tf`, conflicting with the OpenTofu-only naming doctrine in `ADR-0328:2243-2249`.
8. The HCL file itself is shallow: `iac/terraform-module/main.tf:1-7` only declares service name, HTTP transport defaults, ECH/PQC flags, and two outputs.
9. The HCL file lacks the required OpenTofu files named in `ADR-0328:2296-2309`: `variables.tf`, `outputs.tf`, `versions.tf`, and module README.
10. The HCL file lacks canonical variables and outputs named in `ADR-0328:2323-2355`.
11. The service lacks `supported-oses.json`, despite `ADR-0328:2907-2928` requiring per-service OS manifests.
12. Rust-strict scan found no forbidden implementation-language files.
13. Rust readiness remains unproven because `src/` and `tests/` are empty.
14. `contracts/global-trade-v1.proto:5` contains a `java_package` option; this is a proto metadata convention rather than a Java implementation file, but it should be annotated as generator metadata to avoid language-policy ambiguity.
15. OCI Always Free doctrine requires budget-aware module evidence, per `ADR-0328:3666-3685`.
16. No such module exists in this service path.
17. Tenant-class doctrine has not been adopted.
18. The service keeps old tenant-class and generic tier semantics in PRD, manifest, capacity model, ADR reference, and IP metadata.
19. Canonical-direction status: material gaps remain across multi-context, OpenTofu, OS support, OCI Always Free, and tenant-class adoption.

#### 3.4.T Tier Retirement Candidates

1. Exact named retired feature-tier scan for `demo_trial`, `demo_trial`, `paid_core_safe`, and `paid_high_assurance`: zero hits under `microservices/global-trade/`.
2. Generic old tier-model scan found 509 hits across 12 files.
3. Generic old tier-model hit summary: `PRD.md` has 480 hits across lines 16-1937.
4. Generic old tier-model hit summary: `manifest.json` has 8 hits across lines 7-290.
5. Generic old tier-model hit summary: `IP-020-duty-drawback-claim-workflow.md` has 3 hits across lines 13-165.
6. Generic old tier-model hit summary: `IP-016-hs-code-classification-engine-with-fta-preference-attach.md` has 3 hits across lines 13-162.
7. Generic old tier-model hit summary: `capacity-model.md` has 2 hits across lines 16-17.
8. Generic old tier-model hit summary: `IP-023-preferential-trade-agreement-origin-determination-logic.md` has 2 hits across lines 13-19.
9. Generic old tier-model hit summary: `IP-022-embargo-event-audit-chain-anchor.md` has 2 hits across lines 13-19.
10. Generic old tier-model hit summary: `IP-021-quota-management-with-country-of-origin-tracking.md` has 2 hits across lines 13-19.
11. Generic old tier-model hit summary: `IP-019-broker-edi-ingestion-cusdec.md` has 2 hits across lines 13-19.
12. Generic old tier-model hit summary: `IP-018-document-compliance-certificate-generation.md` has 2 hits across lines 13-19.
13. Generic old tier-model hit summary: `IP-017-denied-party-screening-lookup-with-cedar-consent.md` has 2 hits across lines 13-19.
14. Generic old tier-model hit summary: `decisions/ADR-GT-001-sanctions-export-control-and-broker-filing-hold-state-machine.md` has 1 hit at line 11.
15. PRD authority drift: `PRD.md:16` still binds ADR-0316.
16. PRD authority drift: `PRD.md:31` says ADR-0316 binds tenant-class activation.
17. PRD runtime drift: `PRD.md:36` scopes reads to tenant plus tenant class.
18. PRD persona drift: `PRD.md:46` says developer partners build through contracts and tenant classes.
19. PRD observability drift: `PRD.md:242`, `255`, `268`, `281`, `294`, and `307` put `tier` into metric dimensions.
20. PRD feature-gating drift: `PRD.md:243`, `256`, `269`, `282`, `295`, and `308` use old pack-and-tier hooks.
21. PRD story drift: `PRD.md:349-352` has a story specifically about promoting a tenant class.
22. Manifest model drift: `manifest.json:7-8` uses `tier` and `tier_subtype`.
23. Manifest cell drift: `manifest.json:136-140` lists old cell eligibility labels.
24. Manifest old field drift: `manifest.json:253-255` defines `tenant_classes`.
25. Manifest classification drift: `manifest.json:269` says `tier_classification`.
26. Manifest failure-domain drift: `manifest.json:290-291` uses old eligibility and criticality fields.
27. Capacity-model drift: `capacity-model.md:16-22` structures capacity by old service tiers.
28. IP metadata drift: `IP-016-hs-code-classification-engine-with-fta-preference-attach.md:13` binds ADR-0316.
29. IP metadata drift: `IP-016-hs-code-classification-engine-with-fta-preference-attach.md:19` defines `tenant_class`.
30. IP metadata drift: `IP-016-hs-code-classification-engine-with-fta-preference-attach.md:162` carries `tenant_class` as a context field.
31. IP metadata drift: `IP-017-denied-party-screening-lookup-with-cedar-consent.md:13` binds ADR-0316.
32. IP metadata drift: `IP-017-denied-party-screening-lookup-with-cedar-consent.md:19` defines `tenant_class`.
33. IP metadata drift: `IP-018-document-compliance-certificate-generation.md:13` binds ADR-0316.
34. IP metadata drift: `IP-018-document-compliance-certificate-generation.md:19` defines `tenant_class`.
35. IP metadata drift: `IP-019-broker-edi-ingestion-cusdec.md:13` binds ADR-0316.
36. IP metadata drift: `IP-019-broker-edi-ingestion-cusdec.md:19` defines `tenant_class`.
37. IP metadata drift: `IP-020-duty-drawback-claim-workflow.md:13` binds ADR-0316.
38. IP metadata drift: `IP-020-duty-drawback-claim-workflow.md:19` defines `tenant_class`.
39. IP metadata drift: `IP-020-duty-drawback-claim-workflow.md:165` carries `tenant_class` as a context field.
40. IP metadata drift: `IP-021-quota-management-with-country-of-origin-tracking.md:13` binds ADR-0316.
41. IP metadata drift: `IP-021-quota-management-with-country-of-origin-tracking.md:19` defines `tenant_class`.
42. IP metadata drift: `IP-022-embargo-event-audit-chain-anchor.md:13` binds ADR-0316.
43. IP metadata drift: `IP-022-embargo-event-audit-chain-anchor.md:19` defines `tenant_class`.
44. IP metadata drift: `IP-023-preferential-trade-agreement-origin-determination-logic.md:13` binds ADR-0316.
45. IP metadata drift: `IP-023-preferential-trade-agreement-origin-determination-logic.md:19` defines `tenant_class`.
46. ADR reference drift: `decisions/ADR-GT-001-sanctions-export-control-and-broker-filing-hold-state-machine.md:11` references persona-tier material.
47. Severity: default P2 documentation gap because the model is retired and the service must migrate wording and machine-readable fields.
48. P1 escalation condition: any runtime gate, billing gate, or provisioning gate that still acts on these old tier fields during implementation would escalate the gap.

#### 3.4.C Tenant-Class Adoption Gaps

1. Scan terms: `tenant_class`, `demo_trial`, `revenue_share`, and whole-word `paid`.
2. `tenant_class` appears nowhere under `microservices/global-trade/`.
3. `demo_trial` appears nowhere under `microservices/global-trade/`.
4. `revenue_share` appears nowhere under `microservices/global-trade/`.
5. Whole-word `paid` appears only in duty-drawback lifecycle language.
6. Example non-tenant-class use: `IP-020-duty-drawback-claim-workflow.md:37` says duty-drawback consumes paid duty evidence.
7. Example non-tenant-class use: `IP-020-duty-drawback-claim-workflow.md:50` includes `paid` in claim states.
8. Example non-tenant-class use: `IP-020-duty-drawback-claim-workflow.md:69` includes `paid` in the `claim_state` enum.
9. Example non-tenant-class use: `IP-020-duty-drawback-claim-workflow.md:112` says paid status can be reconciled with finance.
10. Example non-tenant-class use: `IP-020-duty-drawback-claim-workflow.md:280-281` sends paid claim alerts and references to adjacent services.
11. Tenant-class adoption gap: yes.
12. Required service semantics after migration: demo/trial usage caps and OCI Always Free profile overlay.
13. Required service semantics after migration: paid tenant class can use any deployment context, contractual SLOs, compliance packs, and BYOK where service capability allows.
14. Required service semantics after migration: revenue-share tenant class supports at-cost or zero-margin substrate with gross-revenue settlement references.
15. Required service semantics after migration: all three classes keep uniform industry-leader feature quality.
16. Replacement should update contract fields, metrics dimensions, capacity docs, IaC variables, and cost events.
17. Replacement should not create a new feature hierarchy.

### 3.5 Dimension 5 - Counterpart And Union-Coverage Alignment

1. Assigned counterpart set for this batch is SAP Global Trade Services, Thomson Reuters ONESOURCE Global Trade, and Descartes.
2. Chat dispatch confirms this set at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17219`.
3. `manifest.json:25-29` lists SAP Global Trade Services, Oracle Global Trade Management, and Descartes Global Trade Intelligence.
4. `manifest.json:203-209` lists SAP, Oracle, Workday, NetSuite, and Microsoft.
5. `competitor-parity-matrix.md:13-20` lists SAP, Oracle, Workday, NetSuite, and Microsoft.
6. `contracts/openapi-v1.yaml:9-14` lists SAP, Oracle, Workday, NetSuite, and Microsoft.
7. `contracts/global-trade-v1.proto:7` repeats SAP, Oracle, Workday, NetSuite, and Microsoft.
8. SLO descriptions repeat SAP, Oracle, Workday, NetSuite, and Microsoft at `slos/*:9`.
9. The local service does not treat Thomson Reuters ONESOURCE as a first-class current counterpart.
10. Descartes is present in `manifest.json:28`, but not consistently present in the contracts, SLOs, capacity model, or competitor matrix.
11. SAP product surface includes sanctioned-party screening, customs management, import/export management, real-time compliance checks, special customs procedures, and HANA-backed analytics according to SAP's current public product page.
12. Thomson Reuters public page emphasizes product classification, denied-party screening, import/export operations, duty optimization, HS/ECCN coverage, 220+ countries/territories, 500+ free-trade agreements, 750+ sanctions/restricted-party lists, 150 researchers, 1,300 sources, and 130 million regulatory updates annually.
13. Descartes public page emphasizes global trade intelligence, import/export data, duty and tariff data, export compliance, classification and duty determination, denied-party screening, FTZ management, trade compliance content, AI sanctions screening, 180+ countries, over 6 million regulatory sources, 30 percent duty/tariff savings, and 75 percent manual screening-time reduction.
14. Global-trade has partial coverage for SAP's original surfaces.
15. Global-trade has planned but not contract-reflected coverage for Thomson Reuters classification, denied-party, FTA, duty optimization, and sanctions-list breadth.
16. Global-trade has planned but not contract-reflected coverage for Descartes data intelligence, classification, duty, FTZ, and AI screening accuracy.
17. Finding impact: the union-coverage matrix must be re-centered on the assigned counterparts.

### 3.6 Dimension 6 - Implementation Readiness

1. No Rust source files were found under `src/`.
2. No test files were found under `tests/`.
3. The contracts define six mutation endpoints in `contracts/openapi-v1.yaml:16-129`.
4. The contracts define six command schemas in `contracts/openapi-v1.yaml:132-239`.
5. The AsyncAPI file defines six event channels in `contracts/asyncapi-v1.yaml:8-37`.
6. The proto file defines six RPCs in `contracts/global-trade-v1.proto:94-100`.
7. IP-016 through IP-023 add capabilities not in those contracts.
8. Missing contract coverage includes HS classification with FTA preference attach.
9. Missing contract coverage includes denied-party screening lookup with Cedar consent.
10. Missing contract coverage includes document compliance certificate generation.
11. Missing contract coverage includes broker EDI ingestion and CUSDEC.
12. Missing contract coverage includes duty-drawback claim workflow.
13. Missing contract coverage includes quota management with country-of-origin tracking.
14. Missing contract coverage includes embargo event audit-chain anchoring.
15. Missing contract coverage includes preferential trade agreement origin determination logic.
16. Implementation readiness is therefore at design-document readiness, not code readiness.
17. Positive signal: the service has consistent command/event/proto skeletons for original bounded contexts.
18. Negative signal: there is no executable Rust implementation or test evidence.
19. Negative signal: generated-looking plans do not prove compiled behavior.
20. Finding impact: implementation readiness should be reported as incomplete.

### 3.7 Dimension 7 - Operational Readiness

1. Four SLO files exist.
2. `slos/global-trade-availability.openslo.yaml:27-29` sets global availability target to 0.999.
3. `slos/customs-declaration-success-rate.openslo.yaml:27-29` sets customs-declaration success target to 0.999.
4. `slos/global-trade-latency-p99.openslo.yaml:21-29` defines a p99 latency bucket at 0.35 seconds with target 0.99.
5. `slos/global-trade-throughput.openslo.yaml:21-29` defines accepted/received throughput with target 0.995.
6. ADR operational targets are more detailed than the SLO files.
7. `ADR-GT-001:82-87` defines release-decision read, sanctions-screening, export-classification, broker-callback, event-emission, and recovery-point targets.
8. `ADR-GT-001:207-214` repeats availability and p95/p99 business SLOs.
9. `ADR-GT-001:259` defines load-test acceptance at 200 screening rps plus 100 broker callback rps for 30 minutes.
10. The current OpenSLO files do not encode all ADR business SLOs.
11. Runbooks exist for approval deadletter, capacity saturation, marketplace settlement blocked, policy deny spike, regional failover, and source import stalled.
12. ADR references denied-party-hit adjudication and broker-filing retry runbooks at `ADR-GT-001:218-219`.
13. Those named runbooks are absent.
14. Dashboards exist, but dashboard coverage was not enough to prove all ADR signals.
15. Capacity model exists, but its top section uses old feature-tier assumptions at `capacity-model.md:16-22`.
16. Operational readiness is useful but not yet canonical-complete.

### 3.8 Dimension 8 - Compliance, Data, And Security

1. Compliance pack roster is present in `manifest.json:59-67`.
2. Additional pack roster exists in `manifest.json:256-266`.
3. PRD requires Cedar default-deny at `PRD.md:36`.
4. Contract schemas require `tenant_id`, `principal_id`, `idempotency_key`, `payload`, and `compliance_packs` at `contracts/openapi-v1.yaml:132-239`.
5. AsyncAPI payloads require `tenant_id`, `audit_event_class`, `bounded_context`, and `occurred_at` at `contracts/asyncapi-v1.yaml:40-153`.
6. Policy files exist for the six original bounded contexts.
7. ADR references `trade-hold-release.cedar` at `ADR-GT-001:203-204`.
8. No `policy/trade-hold-release.cedar` exists in inventory.
9. Data residency is mentioned in `manifest.json:185-190` through platform-owner indirection, soak, trust chain, and OpenBao dynamic secrets.
10. Dedicated `policy/data-residency.md` exists.
11. Dedicated `policy/tenant-isolation.md` exists.
12. Compliance posture is directionally solid.
13. Missing policy evidence and missing tenant-class semantics create a gap for contractual SLO, BYOK, compliance-pack, and revenue-share behavior.
14. Security posture should not be called complete until missing policy and context modules are closed.

### 3.9 Dimension 9 - Launch And Verification Readiness

1. Existing audit file says second pass was authored at `AUDIT-FINDINGS-2026-05-21.json:5-7`.
2. Existing audit file closes doc-set issues at `AUDIT-FINDINGS-2026-05-21.json:10-62`.
3. This audit finds those closure claims too narrow for canonical Wave 3.2 constraints.
4. No lint, typecheck, build, or Rust test can be run for service code because no Rust source/test files exist.
5. No OpenTofu plan can be meaningfully run for six contexts because required context modules are absent.
6. No OS support verification can be run because `supported-oses.json` is absent.
7. No OCI Always Free profile verification can be run because the module is absent.
8. No tenant-class adoption test can be run because there is no tenant-class field or policy.
9. No current counterpart matrix can be verified because the local counterpart roster is stale.
10. Launch readiness is therefore not proven.
11. The service can proceed to remediation planning, not to completion claim.

## 4. Findings Table

| ID | Severity | Finding | Evidence | Remediation target |
|---|---|---|---|---|
| GT-COH-001 | P1 | Six-context deployability is not evidenced. | `ADR-0328:1732-1749`; `master-plan-sequencing.json:704-745`; flat `iac/` inventory. | Add per-context OpenTofu modules or explicit N/A rows. |
| GT-COH-002 | P1 | OpenTofu substrate is incomplete and one module path uses retired Terraform naming. | `ADR-0328:2243-2249`; `iac/terraform-module/main.tf:1-7`. | Rename/rebuild as OpenTofu modules with required files and outputs. |
| GT-COH-003 | P1 | OCI Always Free profile is absent. | `ADR-0328:3666-3685`; no `iac/oci-guest/always-free/`. | Add guest-on-oci Always Free profile or explicit out-of-scope rationale. |
| GT-COH-004 | P1 | OS support manifest is absent. | `ADR-0328:2907-2928`; no `supported-oses.json`. | Add supported OS manifest and CI/packaging evidence. |
| GT-COH-005 | P1 | Assigned counterpart set is not reflected in core docs and contracts. | Chat `:17219`; `manifest.json:203-209`; `competitor-parity-matrix.md:13-20`; `contracts/openapi-v1.yaml:9-14`. | Re-center on SAP, Thomson Reuters, and Descartes. |
| GT-COH-006 | P1 | Contracts cover only the original six contexts and omit expanded IP-016 through IP-023 capabilities. | `contracts/openapi-v1.yaml:16-129`; `contracts/global-trade-v1.proto:94-100`; IP files 016-023. | Add contract surfaces or mark IPs as future with explicit admission gates. |
| GT-COH-007 | P1 | ADR references policy and runbooks that do not exist. | `ADR-GT-001:203-204`; `ADR-GT-001:218-219`; inventory missing `trade-hold-release.cedar`. | Add referenced policy/runbooks or update ADR references. |
| GT-COH-008 | P1 | No executable implementation or test evidence exists. | Empty `src/`; empty `tests/`; `contracts/*` only. | Add Rust implementation and service tests. |
| GT-COH-009 | P2 | Old tier model remains embedded in service docs. | 509 generic hits; `PRD.md:16-1937`; `manifest.json:7-290`; `capacity-model.md:16-22`. | Wave 15J retirement rewrite to tenant-class semantics. |
| GT-COH-010 | P2 | Tenant-class model is absent. | zero `tenant_class`, `demo_trial`, `revenue_share` hits; `paid` only duty-drawback status. | Add `tenant_class` semantics across contracts, metrics, capacity, cost, and IaC. |
| GT-COH-011 | P2 | Capacity model is still old tier segmented. | `capacity-model.md:16-22`. | Replace with single target set plus deployment and tenant-class overlays. |
| GT-COH-012 | P2 | PRD has high-volume repetitive story rows that weaken substance. | `PRD.md:232-1937`. | Collapse generated rows into capability-specific requirements and acceptance tests. |
| GT-COH-013 | P2 | Existing competitor matrix is outdated and repetitive. | `competitor-parity-matrix.md:13-220`. | Replace with assigned union-coverage matrix. |
| GT-COH-014 | P2 | Cross-service handoff doc is absent. | `ARCHITECTURE.md:80-87`; inventory missing `cross-microservice-handoffs.md`. | Add handoff map for marketplace, warehouse, connect, compliance, payments, and supply-chain-planning. |
| GT-COH-015 | P2 | Benchmark evidence directory is absent. | inventory missing `benchmarks/`; `capacity-model.md:26-220` repeated estimates. | Add benchmark workload definitions and evidence capture. |
| GT-COH-016 | P2 | User-facing adoption docs are absent. | missing `faqs/`, `onboarding/`, `tutorials/`, `migration-playbooks/`, `reference-implementations/`. | Add docs aligned to counterpart workflows and tenant classes. |
| GT-COH-017 | P2 | Rust-strict scan passes on forbidden files but implementation boundary is not proven. | no forbidden language files; no Rust source files. | Add Cargo/Rust crate or explicit implementation-plan gate. |
| GT-COH-018 | P2 | OpenSLO files do not encode all ADR business SLOs. | `slos/*`; `ADR-GT-001:207-214`; `ADR-GT-001:259`. | Add SLOs for screening, classification, callback, event emission, and load. |
| GT-COH-019 | P2 | Existing second-pass audit closure is narrower than current canonical constraints. | `AUDIT-FINDINGS-2026-05-21.json:5-62`. | Keep closure as doc-set-only, not canonical-complete. |
| GT-COH-020 | P3 | Proto `java_package` metadata may confuse Rust-strict policy if not annotated. | `contracts/global-trade-v1.proto:5`. | Mark generator metadata as non-runtime. |
| GT-COH-021 | P3 | README/architecture follow-up wording is stale after files were generated. | `ARCHITECTURE.md:109-110`; `README.md:32-200`. | Refresh docs to current state and remove generated trace boilerplate. |

### 4.1 Constraint Evidence Register

1. Multi-context requirement source: `ADR-0328:1732-1749` defines public cloud, guest cloud, on-prem, colo, and Oyatie-provider deployment obligations.
2. Multi-context requirement source: `ADR-0328:2079-2084` requires manifests to name all six contexts or explicit N/A rows.
3. Multi-context local evidence: no `iac/oyatie-public-cloud` directory exists.
4. Multi-context local evidence: no `iac/guest-on-aws` directory exists.
5. Multi-context local evidence: no `iac/guest-on-oci` directory exists.
6. Multi-context local evidence: no `iac/on-prem` directory exists.
7. Multi-context local evidence: no `iac/colo` directory exists.
8. Multi-context local evidence: no `iac/oyatie-iaas` directory exists.
9. Multi-context local conclusion: context deployability is not proven.
10. OpenTofu requirement source: `ADR-0328:2243-2249` says OpenTofu is the canonical engine and Terraform wording is only acceptable as superseded/forbidden/historical reference.
11. OpenTofu requirement source: `ADR-0328:2296-2309` lists the required module file set.
12. OpenTofu local evidence: `iac/terraform-module/main.tf:1-7` is the only HCL-like module path found.
13. OpenTofu local evidence: no service-local `versions.tf` exists.
14. OpenTofu local evidence: no service-local `variables.tf` exists outside the minimal one-file module.
15. OpenTofu local evidence: no service-local `outputs.tf` exists outside the minimal one-file module.
16. OpenTofu local evidence: no context-module README exists.
17. OpenTofu local conclusion: IaC is scaffold-level and not context-admissible.
18. OS requirement source: `ADR-0328:2648-2854` defines the supported and excluded OS matrix.
19. OS requirement source: `ADR-0328:2907-2928` requires service-local manifest fields.
20. OS local evidence: `supported-oses.json` is absent.
21. OS local evidence: no OS-specific test matrix is present under `tests/`.
22. OS local evidence: no packaging matrix is present under this service path.
23. OS local conclusion: OS portability cannot be claimed for this service.
24. Rust requirement source: `ADR-0328:3047-3067` binds backend/runtime/CLI/validation/codegen/scripting/CI to Rust.
25. Rust requirement source: `ADR-0328:3235-3285` lists forbidden language families and shell-as-durable-logic constraints.
26. Rust local evidence: forbidden-language file scan returned no files under this service path.
27. Rust local evidence: no Rust files are present under `src/`.
28. Rust local evidence: no Rust test files are present under `tests/`.
29. Rust local evidence: no service-local Cargo manifest was found in the inventory.
30. Rust local conclusion: language-policy violation was not found, but implementation readiness is unproven.
31. OCI Always Free requirement source: `ADR-0328:3493-3499` makes Always Free a guest-on-oci sub-profile.
32. OCI Always Free requirement source: `ADR-0328:3514-3571` defines the 4 OCPU, 24 GB memory, storage, network, and egress constraints.
33. OCI Always Free requirement source: `ADR-0328:3666-3685` requires profile module files and inputs.
34. OCI local evidence: no `iac/oci-guest/always-free/` directory exists.
35. OCI local evidence: no budget variables or outputs exist for Always Free limits.
36. OCI local evidence: no demo/trial usage-cap policy exists.
37. OCI local conclusion: demo/trial infrastructure cannot be verified.
38. Tenant-class requirement source: current batch directive defines `demo_trial`, `paid`, and `revenue_share`.
39. Tenant-class local evidence: no `tenant_class` string exists under the service path.
40. Tenant-class local evidence: no `demo_trial` string exists under the service path.
41. Tenant-class local evidence: no `revenue_share` string exists under the service path.
42. Tenant-class local evidence: `paid` appears only as duty-drawback status in `IP-020`.
43. Tenant-class local conclusion: replacement commercial model is not adopted.
44. Tier-retirement requirement source: `feedback_no_tenant_classes_2026_05_20.md:10-24` retires tenant classes.
45. Tier-retirement local evidence: exact retired named-tier scan returned zero hits.
46. Tier-retirement local evidence: generic old tier-model scan returned 509 hits.
47. Tier-retirement local conclusion: no exact named retired feature-tier terms are local, but the old model still dominates fields and prose.
48. Counterpart requirement source: chat dispatch `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17219` names SAP, Thomson Reuters, and Descartes.
49. Counterpart local evidence: `manifest.json:25-29` has SAP, Oracle, and Descartes.
50. Counterpart local evidence: `manifest.json:203-209` has SAP, Oracle, Workday, NetSuite, and Microsoft.
51. Counterpart local evidence: `competitor-parity-matrix.md:13-20` has SAP, Oracle, Workday, NetSuite, and Microsoft.
52. Counterpart local conclusion: Thomson Reuters is missing from the main local matrix.
53. Contract evidence: `contracts/openapi-v1.yaml:16-129` exposes only six mutation endpoints.
54. Contract evidence: `contracts/asyncapi-v1.yaml:8-37` exposes only six event channels.
55. Contract evidence: `contracts/global-trade-v1.proto:94-100` exposes only six RPCs.
56. Contract conclusion: expanded IP-016 through IP-023 capabilities are not in public contracts.
57. Operational evidence: `slos/global-trade-availability.openslo.yaml:27-29` target is 0.999.
58. Operational evidence: `slos/global-trade-latency-p99.openslo.yaml:21-29` uses a 0.35-second bucket.
59. Operational evidence: `ADR-GT-001:207-214` has more detailed business SLOs.
60. Operational conclusion: SLO depth lags ADR depth.
61. Policy evidence: `ADR-GT-001:203-204` references `trade-hold-release.cedar`.
62. Policy evidence: no `policy/trade-hold-release.cedar` exists.
63. Runbook evidence: `ADR-GT-001:218-219` references denied-party-hit adjudication and broker-filing retry runbooks.
64. Runbook evidence: no runbook with those names exists in inventory.
65. Prior-audit evidence: `AUDIT-FINDINGS-2026-05-21.json:10-62` closes six doc-set rows.
66. Prior-audit conclusion: prior closure is useful as doc-set evidence, not as canonical completeness evidence.
67. Launch evidence: no executable test harness exists.
68. Launch evidence: no OpenTofu context plan can be run from this path.
69. Launch evidence: no OS matrix test can be run from this path.
70. Launch conclusion: remediation should start with machine-readable context, tenant-class, OS, and contract updates before any completion claim.

## 5. Open Questions

1. Should IP-016 through IP-023 become first-class current product scope now, or remain gated future expansions?
2. What is the authoritative replacement field name for old `tenant_class` references: `tenant_class`, `billing_model`, or a split of tenant class plus commercial arrangement?
3. Should global-trade expose `revenue_share` settlement evidence directly, or only carry marketplace settlement references?
4. Which compliance packs require BYOK semantics for paid tenants in this service?
5. Which trade workloads are allowed in `demo_trial` under OCI Always Free profile limits?
6. Should Descartes FTZ management be in scope for this service, or owned by an adjacent customs/warehouse service?
7. Should Thomson Reuters content-update breadth be modeled as a content-ingestion SLA owned by global-trade or by compliance/regional-pack?
8. Should HS/ECCN classification be a single endpoint or two separately audited decision surfaces?
9. Should denied-party screening support batch and continuous rescreening in the first executable slice?
10. Should trade-document certificate generation include certificate-of-origin only, or all trade certificate families?
11. Should broker EDI support CUSDEC first, or include ABI/AES/EMCS/Intrastat adapters in the first contract pass?
12. Should duty drawback be included in first launch, or separated behind explicit trade-finance admission?
13. Should quota/origin tracking be tied to FTA preference origin in one domain model?
14. Should embargo audit-chain anchors be event-only, or should they expose direct read APIs?
15. Should the service use one OpenSLO file per bounded context plus one aggregate SLO?
16. Should the old `criticality_tier` manifest field remain as an infrastructure-criticality concept, or be renamed to avoid feature-tier confusion?
17. Should `java_package` remain in the proto file for external generator compatibility?
18. Should an ADR amendment retire local ADR-0316 references across this service immediately?
19. Should the prior `AUDIT-FINDINGS-2026-05-21.json` be updated later to reflect these canonical gaps?
20. Should this service be blocked from deployable-context claims until OpenTofu modules exist?

<!-- ORCHESTRATOR REPORT
  microservice: global-trade
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/global-trade/coherence-audit-2026-05-20.md: 629 lines
    - /Users/jasonlee/oyatie/microservices/global-trade/feature-parity-matrix-2026-05-20.md: 443 lines
    - /Users/jasonlee/oyatie/microservices/global-trade/performance-benchmark-numbers-2026-05-20.md: 322 lines
  inventory_files_seen: 138
  inventory_lines_read: 14657
  chat_history_matches_processed: 73
  findings_p0: 0
  findings_p1: 8
  findings_p2: 11
  findings_p3: 2
  tier_retirement_candidates_found: exact demo_trial/demo_trial/paid_core_safe/paid_high_assurance hits 0; generic old tier-model hits 509; citations PRD.md:16-1937, manifest.json:7-290, capacity-model.md:16-17, ADR-GT-001:11, IP-016:13-162, IP-017:13-19, IP-018:13-19, IP-019:13-19, IP-020:13-165, IP-021:13-19, IP-022:13-19, IP-023:13-19
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/revenue_share references and paid appears only as duty-drawback status
  top_3_counterparts_confirmed: SAP Global Trade Services / Thomson Reuters ONESOURCE Global Trade / Descartes
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1394
-->
