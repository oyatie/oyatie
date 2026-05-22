# consent-graph ownership-coherence audit — 2026-05-20

Audit owner: single-agent Wave 3 Batch 3.2 ownership audit.
Target microservice: `microservices/consent-graph/`.
Counterpart set: OneTrust / TrustArc / Cookiebot.
Deliverable set: three documents; capability-ladder retirement deltas deliverable retired by 2026-05-20 directive.
Audit posture: read-only investigation first, then in-service report authoring only.
Completion stance: findings are evidence-bound; no remediation edits outside these three audit reports.
Deployment assumption tested: all six deployment contexts unless contradicted by service artifacts.
Product thesis tested: real-time cross-tenant data-sharing consent and enforcement, not only cookie consent.
Uniform quality assumption: industry-leader-grade quality across tenant classes; no feature-ladder segmentation.
Tenant-class model expected by current dispatch: `demo_trial`, `paid`, `revenue_share`.

## Anchor citations

- ADR-0328 D-15 requires the six deployment contexts and per-context support evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2118`.
- ADR-0328 D-16 requires OpenTofu, canonical `tofu` vocabulary, and per-context IaC directories: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2307`.
- ADR-0328 D-20 expands the audit to nine dimensions and requires language, OS, OpenTofu, and context checks: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3831-4230`.
- Master-plan sequencing encodes OpenTofu, six contexts, supported OSes, Rust backend policy, Leptos web policy, and OCI Always Free module shape: `specs/master-plan-sequencing.json:704-865`.
- Brief template requires multi-context, OpenTofu, OS, language-policy, and anti-scaffold checks: `docs/standards/brief-template.md:666-1891`.
- Multi-context doctrine forbids provider-locked assumptions and treats missing service deployment-context declarations as a gap: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md:18-38`.
- OpenTofu doctrine requires every deployment to be automated through OpenTofu, not Terraform or manual handroll: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-37`.
- OS doctrine requires a per-service supported-OS manifest and Tier-1 OS coverage evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-78`.
- Rust-strict doctrine permits Rust, OpenTofu HCL, Cedar, YAML/JSON/proto/OpenAPI/AsyncAPI/OpenSLO/SQL/Markdown, and scoped frontend allowlist surfaces while forbidding Python, JavaScript app logic, TypeScript app logic, Ruby, PHP, Java, Scala, Groovy, Go, and F#: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-66`.
- No-capability-ladder doctrine records the 2026-05-20 directive that the prior tier system is being retired: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_ladder_2026_05_20.md:10-43`.
- Tenant-class replacement doctrine supplies the demo/paid/revenue-share economic model and Wave 15J retirement target: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:23-47`.
- Microservice ownership doctrine requires one owner to inspect PRD, ADRs, plans, contracts, SLOs, policy, handoffs, IaC, tests, runbooks, and chat history before judging coherence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-94`.
- Deliverable verification doctrine rejects line-count-only completion and requires actual scope, maturity, coherence, and chat-history checks: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-53`.
- Substance doctrine rejects scaffold-only documents and requires bespoke service-specific content: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20`.
- Chat-history anchor confirms the rolling audit queue paired `consent-graph` with OneTrust / TrustArc / Cookiebot: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`.
- Chat-history anchor confirms the same counterpart tuple after queue rebuild: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`.
- Chat-history anchor confirms the mid-batch instruction to drop capability-ladder retirement deltas and restructure performance benchmark reporting around single industry-leader targets plus deployment-context overlays: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16330`.

## §1 Purpose

1. This audit determines whether `consent-graph` is internally coherent as a microservice ownership unit.
2. The audit compares stated product purpose, contracts, SLOs, policy, runbooks, capacity, benchmarks, and IaC evidence.
3. The audit explicitly distinguishes implementation readiness from documentation richness.
4. The audit treats existing service artifacts as current evidence, not intent hidden outside the tree.
5. The audit does not remediate the microservice; it records findings with citations.
6. The audit uses OneTrust, TrustArc, and Cookiebot as the required top-three counterpart set.
7. The audit recognizes that consent-graph is broader than a web-cookie consent manager.
8. The service PRD defines the core problem as real-time cross-entity data sharing that preserves auditability, revocability, sovereignty, narrow scope, and freshness: `microservices/consent-graph/PRD.md:17-31`.
9. The PRD goals include Cedar enforcement at every cross-tenant hop, bilateral audit chain, real-time revocation, zero-copy projection, three sharing modes, and partner-directory handshake: `microservices/consent-graph/PRD.md:48-59`.
10. The PRD non-goals clarify that the service is not replacing audit-chain, ontology, billing, marketplace discovery, or token-based gateway controls: `microservices/consent-graph/PRD.md:63-73`.
11. The service has a strong product core for B2B and B2C data-sharing agreements.
12. The service has weaker evidence for deployment substrate completeness.
13. The service has weaker evidence for tenant-class semantics after the 2026-05-20 doctrine shift.
14. The service has existing retired-tier vocabulary that must be carried as findings, not perpetuated.
15. The service has high-value SLOs already expressed as OpenSLO files.
16. The service has detailed failure and incident response material.
17. The service has no visible Rust crate or `src/` implementation under its own directory.
18. The service has no visible tests under its own directory.
19. The service has Helm and Kustomize runtime packaging but no OpenTofu per-context IaC modules.
20. The service has no `supported-oses.json` or equivalent per-service OS manifest.
21. The service has no top-level `README.md`.
22. The service has no `cross-microservice-handoffs.md`.
23. The service has no `iac/oci-guest/always-free/` profile.
24. The audit therefore classifies product architecture as strong, canonical-direction alignment as incomplete, and deployability claims as unproven.
25. The audit stop condition is three landed reports, all line-count verified, with findings grounded in file, memory, chat, or public counterpart citations.

## §2 Inventory

Inventory command: `rg --files microservices/consent-graph | sort`.
Inventory file count seen: 135.
Inventory line count indexed by `wc -l`: 22690.
Top-level README presence: not present.
Top-level `src/` presence: not present.
Top-level `tests/` presence: not present.
Top-level `specs/` directory presence: present but empty by `rg --files`.
Top-level `evidence/` directory presence: present but empty by `rg --files`.
IaC directory presence: present.
OpenTofu per-context directory presence: not present.
Helm directory presence: present at `microservices/consent-graph/iac/helm/consent-graph/`.
Kustomize directory presence: present at `microservices/consent-graph/iac/kustomize/`.
Forbidden-language source file scan: no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files under the service path.

### §2.A Complete file inventory

1. `microservices/consent-graph/ARCHITECTURE.md`
2. `microservices/consent-graph/IP-001-agreement-kernel.md`
3. `microservices/consent-graph/IP-002-agreement-domain.md`
4. `microservices/consent-graph/IP-003-agreement-usecase-and-adapter.md`
5. `microservices/consent-graph/IP-004-enforcement-kernel.md`
6. `microservices/consent-graph/IP-005-enforcement-domain-cedar.md`
7. `microservices/consent-graph/IP-006-enforcement-usecase-and-adapter.md`
8. `microservices/consent-graph/IP-007-revocation-kernel-worker.md`
9. `microservices/consent-graph/IP-008-revocation-pulsar-fanout.md`
10. `microservices/consent-graph/IP-009-projection-gateway-kernel.md`
11. `microservices/consent-graph/IP-010-projection-gateway-mint-acl.md`
12. `microservices/consent-graph/IP-011-projection-scope-narrowing-aggregate.md`
13. `microservices/consent-graph/IP-012-audit-bridge-bilateral-emitter.md`
14. `microservices/consent-graph/IP-013-audit-bridge-cross-pointer-integrity.md`
15. `microservices/consent-graph/IP-014-partner-directory-handshake.md`
16. `microservices/consent-graph/IP-015-self-observability-slo-wiring.md`
17. `microservices/consent-graph/IP-journey-j01-emergency-911-dispatch-opt-in-fields.md`
18. `microservices/consent-graph/IP-journey-j04-shared-account-consent-rewrite.md`
19. `microservices/consent-graph/IP-journey-j100-pack-rollout-first-action.md`
20. `microservices/consent-graph/IP-journey-j76-consent-rights-ledger.md`
21. `microservices/consent-graph/IP-journey-j80-consent-rights-ledger.md`
22. `microservices/consent-graph/IP-journey-j83-consent-rights-ledger.md`
23. `microservices/consent-graph/IP-journey-j84-consent-rights-ledger.md`
24. `microservices/consent-graph/IP-journey-j85-consent-rights-ledger.md`
25. `microservices/consent-graph/IP-journey-j89-consent-rights-ledger.md`
26. `microservices/consent-graph/IP-journey-j90-consent-rights-ledger.md`
27. `microservices/consent-graph/IP-journey-j91-us-msb-mtl-overlay.md`
28. `microservices/consent-graph/IP-journey-j92-br-lgpd-us-parent-dsar.md`
29. `microservices/consent-graph/IP-journey-j93-in-dpdpa-rbi-overlay.md`
30. `microservices/consent-graph/IP-journey-j94-sox404-public-company-controls.md`
31. `microservices/consent-graph/IP-journey-j95-iso27001-soc2-annual-audit.md`
32. `microservices/consent-graph/IP-journey-j96-ksa-uae-mena-onboarding.md`
33. `microservices/consent-graph/IP-journey-j97-sg-pdpa-mas-tenant.md`
34. `microservices/consent-graph/IP-journey-j98-au-privacy-apra-cps234.md`
35. `microservices/consent-graph/IP-journey-j99-multi-pack-conflict-resolution.md`
36. `microservices/consent-graph/PHASE-01-CONSENT-GRAPH-SUBSTRATE.md`
37. `microservices/consent-graph/PRD.md`
38. `microservices/consent-graph/backfill-replay.md`
39. `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md`
40. `microservices/consent-graph/break-glass.md`
41. `microservices/consent-graph/capabilities/consent-enforce.yaml`
42. `microservices/consent-graph/capabilities/consent-grant.yaml`
43. `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`
44. `microservices/consent-graph/capability-ladders/tier-matrix.md`
45. `microservices/consent-graph/capacity-model.md`
46. `microservices/consent-graph/catalog/oya-consent-graph-agreement-adapter.yaml`
47. `microservices/consent-graph/catalog/oya-consent-graph-agreement-domain.yaml`
48. `microservices/consent-graph/catalog/oya-consent-graph-agreement-kernel.yaml`
49. `microservices/consent-graph/catalog/oya-consent-graph-agreement-rest.yaml`
50. `microservices/consent-graph/catalog/oya-consent-graph-agreement-sdk.yaml`
51. `microservices/consent-graph/catalog/oya-consent-graph-agreement-usecase.yaml`
52. `microservices/consent-graph/catalog/oya-consent-graph-audit-bridge-usecase.yaml`
53. `microservices/consent-graph/catalog/oya-consent-graph-enforcement-adapter-cedar.yaml`
54. `microservices/consent-graph/catalog/oya-consent-graph-enforcement-domain.yaml`
55. `microservices/consent-graph/catalog/oya-consent-graph-enforcement-kernel.yaml`
56. `microservices/consent-graph/catalog/oya-consent-graph-enforcement-sdk.yaml`
57. `microservices/consent-graph/catalog/oya-consent-graph-enforcement-usecase.yaml`
58. `microservices/consent-graph/catalog/oya-consent-graph-partner-directory-rest.yaml`
59. `microservices/consent-graph/catalog/oya-consent-graph-projection-gateway-kernel.yaml`
60. `microservices/consent-graph/catalog/oya-consent-graph-projection-gateway-usecase.yaml`
61. `microservices/consent-graph/catalog/oya-consent-graph-revocation-adapter-pulsar.yaml`
62. `microservices/consent-graph/catalog/oya-consent-graph-revocation-kernel.yaml`
63. `microservices/consent-graph/competitor-parity-matrix.md`
64. `microservices/consent-graph/compliance.md`
65. `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`
66. `microservices/consent-graph/contracts/openapi/consent-graph.yaml`
67. `microservices/consent-graph/contracts/proto/consent-graph.proto`
68. `microservices/consent-graph/cost-budget.md`
69. `microservices/consent-graph/dashboards/consent-grant-funnel.json`
70. `microservices/consent-graph/dashboards/projection-freshness.json`
71. `microservices/consent-graph/dashboards/revocation-fan-out.json`
72. `microservices/consent-graph/data-residency.md`
73. `microservices/consent-graph/decisions/ADR-SVC-CG-001-bilateral-chain-link-schema.md`
74. `microservices/consent-graph/decisions/ADR-SVC-CG-002-cedar-cache-invalidation.md`
75. `microservices/consent-graph/decisions/ADR-SVC-CG-003-three-sharing-modes.md`
76. `microservices/consent-graph/decisions/ADR-SVC-CG-004-grantor-region-topic-ownership.md`
77. `microservices/consent-graph/decisions/ADR-SVC-CG-005-self-revocation-b2c.md`
78. `microservices/consent-graph/dpia.md`
79. `microservices/consent-graph/failure-modes.md`
80. `microservices/consent-graph/faqs/data-steward-faq.md`
81. `microservices/consent-graph/iac/helm/consent-graph/Chart.yaml`
82. `microservices/consent-graph/iac/helm/consent-graph/templates/consent-graph-app-deployment.yaml`
83. `microservices/consent-graph/iac/helm/consent-graph/templates/enforcement-app-deployment.yaml`
84. `microservices/consent-graph/iac/helm/consent-graph/templates/hg-consent-registration.yaml`
85. `microservices/consent-graph/iac/helm/consent-graph/templates/rbac.yaml`
86. `microservices/consent-graph/iac/helm/consent-graph/templates/revocation-and-workers.yaml`
87. `microservices/consent-graph/iac/helm/consent-graph/values.yaml`
88. `microservices/consent-graph/iac/kustomize/base/kustomization.yaml`
89. `microservices/consent-graph/iac/kustomize/overlays/eu/kustomization.yaml`
90. `microservices/consent-graph/iac/kustomize/overlays/eu/pack-eu-values-patch.yaml`
91. `microservices/consent-graph/iac/kustomize/overlays/eu/sovereignty-rules.yaml`
92. `microservices/consent-graph/iac/kustomize/overlays/jp/kustomization.yaml`
93. `microservices/consent-graph/iac/kustomize/overlays/jp/sovereignty-rules.yaml`
94. `microservices/consent-graph/iac/kustomize/overlays/kr/kustomization.yaml`
95. `microservices/consent-graph/iac/kustomize/overlays/kr/pack-kr-values-patch.yaml`
96. `microservices/consent-graph/iac/kustomize/overlays/kr/sovereignty-rules.yaml`
97. `microservices/consent-graph/iac/kustomize/overlays/us-healthcare/kustomization.yaml`
98. `microservices/consent-graph/iac/kustomize/overlays/us-healthcare/sovereignty-rules.yaml`
99. `microservices/consent-graph/iac/kustomize/overlays/us/kustomization.yaml`
100. `microservices/consent-graph/iac/kustomize/overlays/us/sovereignty-rules.yaml`
101. `microservices/consent-graph/incident-response.md`
102. `microservices/consent-graph/manifest.json`
103. `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md`
104. `microservices/consent-graph/multi-region.md`
105. `microservices/consent-graph/onboarding/data-steward-first-week.md`
106. `microservices/consent-graph/partnership-onboarding.md`
107. `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`
108. `microservices/consent-graph/policy/break-glass-healthcare.cedar`
109. `microservices/consent-graph/policy/cross-tenant-projection.cedar`
110. `microservices/consent-graph/policy/deny-all-fallback.cedar`
111. `microservices/consent-graph/reference-implementations/projection-subscribe-rust-sdk.md`
112. `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`
113. `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`
114. `microservices/consent-graph/runbooks/consent-forgery-detected.md`
115. `microservices/consent-graph/runbooks/data-residency-enforcement.md`
116. `microservices/consent-graph/runbooks/partner-offboarding.md`
117. `microservices/consent-graph/runbooks/partner-onboarding.md`
118. `microservices/consent-graph/runbooks/regional-sovereignty-violation.md`
119. `microservices/consent-graph/runbooks/revocation-incident.md`
120. `microservices/consent-graph/scorecards/architecture-conformance.yaml`
121. `microservices/consent-graph/scorecards/compliance-coverage.yaml`
122. `microservices/consent-graph/scorecards/security-posture.yaml`
123. `microservices/consent-graph/scorecards/sre-readiness.yaml`
124. `microservices/consent-graph/sdk-plan.md`
125. `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`
126. `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`
127. `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`
128. `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`
129. `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`
130. `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`
131. `microservices/consent-graph/slos/partner-handshake-latency.openslo.yaml`
132. `microservices/consent-graph/slos/revocation-propagation-latency.openslo.yaml`
133. `microservices/consent-graph/slos/sovereignty-violation-zero.openslo.yaml`
134. `microservices/consent-graph/threat-model.md`
135. `microservices/consent-graph/tutorials/draft-and-activate-data-sharing-agreement.md`

### §2.B Required-artifact coverage

1. PRD present and substantive: `microservices/consent-graph/PRD.md:1-280`.
2. Architecture document present and long, but begins with a content-pass expansion warning: `microservices/consent-graph/ARCHITECTURE.md:3`.
3. README absent at service root.
4. Service ADRs present: five files under `microservices/consent-graph/decisions/`.
5. Implementation plans present: IP-001 through IP-015 plus journey-specific IP files.
6. OpenAPI contract present: `microservices/consent-graph/contracts/openapi/consent-graph.yaml:1-574`.
7. AsyncAPI contract present: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml:1-246`.
8. Proto contract present: `microservices/consent-graph/contracts/proto/consent-graph.proto:1-433`.
9. OpenSLO files present: nine files under `microservices/consent-graph/slos/`.
10. Capability-tier directory present and now a retirement candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:1-126`.
11. Capacity model present and detailed: `microservices/consent-graph/capacity-model.md:7-179`.
12. Failure modes present and detailed: `microservices/consent-graph/failure-modes.md:1-252`.
13. Incident response present and detailed: `microservices/consent-graph/incident-response.md:1-147`.
14. Cost budget present and detailed: `microservices/consent-graph/cost-budget.md:1-98`.
15. DPIA present.
16. Compliance map present but contains repeated anchor-generated sections and generic tier vocabulary: `microservices/consent-graph/compliance.md:160-1027`.
17. Benchmarks present, but they use retired tier vocabulary and a counterpart set broader than the required top three: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:9-16`.
18. FAQ present.
19. Onboarding guide present.
20. Migration playbook present and specifically covers OneTrust plus TrustArc: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:9-27`.
21. Reference implementation present for Rust SDK subscription usage.
22. Tutorials present.
23. Runbooks present.
24. Helm IaC present.
25. Kustomize overlays present.
26. OpenTofu IaC absent.
27. Supported-OS manifest absent.
28. Cross-microservice handoff document absent.
29. Source implementation directory absent.
30. Test directory absent.

## §3 Nine-dimension audit

### §3.1 Dimension 1 — Product purpose and ownership coherence

1. Verdict: coherent product nucleus with missing deployment-control surfaces.
2. The PRD defines consent-graph as a consent and data-sharing enforcement substrate, not merely a consent-record database: `microservices/consent-graph/PRD.md:17-31`.
3. The PRD goals cover DataSharingAgreement records, Cedar enforcement, bilateral audit-chain, revocation, zero-copy projections, three sharing modes, scope narrowing, partner directory, and compliance packs: `microservices/consent-graph/PRD.md:48-59`.
4. The bounded-context list covers agreement, enforcement, revocation, projection-gateway, audit-bridge, partner-directory, schemas, and SDK: `microservices/consent-graph/PRD.md:247-259`.
5. The manifest repeats the same bounded contexts and catalogs the crates intended for each surface: `microservices/consent-graph/manifest.json:10-95`.
6. The OpenAPI supports agreement draft, offer, accept, revoke, amend, enforcement, break-glass, partner handshake, and revocation receipt surfaces: `microservices/consent-graph/contracts/openapi/consent-graph.yaml:33-285`.
7. The AsyncAPI covers agreement lifecycle, revocation priority fanout, audit bridge, projection delivery, and projection topic minting: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml:20-65`.
8. The proto maps the same service boundaries into AgreementService, EnforcementService, RevocationService, ProjectionGatewayService, AuditBridgeService, and PartnerDirectoryService: `microservices/consent-graph/contracts/proto/consent-graph.proto:18-59`.
9. Product-scope exclusions are clear: no cross-tenant writes, no multi-grantor joins, no replacement for audit-chain or ontology, no billing, and no marketplace discovery: `microservices/consent-graph/PRD.md:63-73`.
10. The migration playbook reinforces that OneTrust and TrustArc are workflow-oriented platforms while consent-graph is the enforcement source of truth: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:13-27`.
11. The architecture's product purpose is consistent with service dependencies on identity, tenancy, policy-engine, observability, audit-chain, cloud-secrets, cell, and cloud-iac: `microservices/consent-graph/ARCHITECTURE.md:18-46`.
12. The service therefore owns the enforcement and consent-state graph, while adjacent UI, workflow, billing, audit substrate, ontology, and marketplace concerns remain external.
13. Product ownership risk is not confusion over what consent-graph is; the risk is claims that extend beyond evidenced deployment surfaces.
14. The current service tree supports strong product understanding for engineers and auditors.
15. The current service tree does not prove the service can be deployed in every required context.

### §3.2 Dimension 2 — Artifact completeness and intern-buildability

1. Verdict: documentation-rich, implementation-evidence-thin.
2. The service contains a detailed PRD and architecture document.
3. The service contains five service-level ADRs.
4. The service contains fifteen core IP files plus journey IP files.
5. The service contains OpenAPI, AsyncAPI, and proto contracts.
6. The service contains Cedar policy fragments.
7. The service contains OpenSLO definitions for key safety and freshness objectives.
8. The service contains capacity, cost, failure, incident, compliance, threat, data-residency, and multi-region documentation.
9. The service contains runbooks for DSAR, audit divergence, consent forgery, data residency, partner offboarding, partner onboarding, regional sovereignty, and revocation.
10. The service contains catalog component manifests.
11. The service contains dashboards and scorecards.
12. The service does not contain a top-level README to orient a new implementer.
13. The service does not contain `src/` source files.
14. The service does not contain `tests/` files.
15. The service does not contain Rust crate manifests under its own path.
16. The manifest nevertheless lists intended crates and says parent wiring remains to be added to the workspace and evidence files: `microservices/consent-graph/manifest.json:342-345`.
17. The manifest also labels all core implementation plans as GA: `microservices/consent-graph/manifest.json:196-211`.
18. The absence of in-path source and tests means a new engineer can understand design but cannot build or verify the service from this directory alone.
19. The architecture document contains a first-line content-pass warning, which signals that at least one generation pass knew the file required expansion review: `microservices/consent-graph/ARCHITECTURE.md:3`.
20. Compliance documentation repeats generated anchor blocks and says IaC evidence is present, but the IaC evidence is not OpenTofu per context: `microservices/consent-graph/compliance.md:219-221`.
21. The deliverable verification doctrine says line count and self-report are not enough: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-31`.
22. Artifact completeness is therefore partial: high design coverage, low implementation and canonical deployment proof.

### §3.3 Dimension 3 — Internal consistency across PRD, ADRs, contracts, policy, SLOs, and runbooks

1. Verdict: core domain consistency is strong, with notable tenant-class and deployment gaps.
2. PRD success metrics align with SLO files for grant latency, projection freshness, revocation latency, Cedar evaluation latency, audit completeness, divergence, sovereignty, bilateral link integrity, and partner handshake: `microservices/consent-graph/PRD.md:77-87`.
3. Grant latency SLO encodes 95 percent within two seconds: `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml:31-33`.
4. Projection freshness SLO encodes 95 percent within 500 ms: `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml:31-33`.
5. Cedar evaluation SLO encodes 99 percent within 10 ms: `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml:31-33`.
6. Revocation propagation SLO encodes p99 within one second: `microservices/consent-graph/slos/revocation-propagation-latency.openslo.yaml:22-34`.
7. Partner handshake SLO encodes 95 percent within 30 seconds: `microservices/consent-graph/slos/partner-handshake-latency.openslo.yaml:21-33`.
8. The capacity model repeats the year-one scale envelope of 10M active agreements, 100K new agreements per day, 1M revocations per day, 100B projection events per day, 100K enforcement evaluations per second, and 10K peers: `microservices/consent-graph/capacity-model.md:7-18`.
9. The PRD repeats the same scale envelope: `microservices/consent-graph/PRD.md:193-198`.
10. ADR-SVC-CG-002 chooses revocation-led Cedar cache invalidation, aligning with the PRD revocation target: `microservices/consent-graph/decisions/ADR-SVC-CG-002-cedar-cache-invalidation.md:47-86`.
11. ADR-SVC-CG-003 chooses projection, aggregate, and attested query modes, aligning with OpenAPI `SharingTerms.mode`: `microservices/consent-graph/decisions/ADR-SVC-CG-003-three-sharing-modes.md:48-87` and `microservices/consent-graph/contracts/openapi/consent-graph.yaml:361-377`.
12. ADR-SVC-CG-004 chooses grantor-region topic ownership, aligning with sovereignty goals and runbook coverage: `microservices/consent-graph/decisions/ADR-SVC-CG-004-grantor-region-topic-ownership.md:47-82`.
13. Failure modes consistently fail closed on identity, cache, Cedar, and sovereignty failures: `microservices/consent-graph/failure-modes.md:54-83`.
14. Incident response consistently treats sovereignty violation, audit-chain divergence, consent forgery, and deny-rate spikes as P0: `microservices/consent-graph/incident-response.md:7-14`.
15. The policy set includes explicit deny-all fallback and specific projection, aggregate, and break-glass controls.
16. The cross-tenant projection policy still gates on `tenant_class`, which is inconsistent with the current tenant-class replacement doctrine: `microservices/consent-graph/policy/cross-tenant-projection.cedar:42-50`.
17. The proto also encodes `TenantClass` and `tenant_class`: `microservices/consent-graph/contracts/proto/consent-graph.proto:122-128` and `microservices/consent-graph/contracts/proto/consent-graph.proto:263-268`.
18. The OpenAPI encodes `tenant_class` enum T0/T1/T2/T3: `microservices/consent-graph/contracts/openapi/consent-graph.yaml:458-485`.
19. Internal domain coherence is therefore high for consent/enforcement/revocation, but low for the post-tier tenant-class model.

### §3.4 Dimension 4 — Canonical-direction alignment

1. Verdict: product-level direction aligns; platform-direction control surfaces are incomplete.
2. The product purpose aligns with ADR-0328's counterpart and benchmark discipline because it can be mapped to OneTrust, TrustArc, and Cookiebot surfaces.
3. The service does not yet align with the all-six deployment-context requirement.
4. The service does not yet align with the OpenTofu per-context module requirement.
5. The service does not yet align with the supported-OS manifest requirement.
6. The service does align with Rust-strict file-extension scans because no forbidden language source files are present under the service path.
7. The service partially conflicts with Rust-strict doctrine because the PRD promises TypeScript and Python clients without a present generator/provenance boundary: `microservices/consent-graph/PRD.md:57`.
8. The service conflicts with tenant-class retirement migration doctrine because it still contains explicit tenant_class demo_trial, tenant_class paid, tenant_class paid, and compliance_pack-bound paid references.
9. The service conflicts with tenant-class replacement doctrine because no `tenant_class`, `demo_trial`, `paid`, or `revenue_share` semantics are present under the service path.
10. The service also uses generic `tier` vocabulary in manifest, architecture, compliance, cost, and partnership artifacts.
11. Generic `tier` vocabulary is lower risk than explicit retired feature tiers, but it can confuse the replacement economic model.
12. The present IaC uses Helm and Kustomize rather than OpenTofu.
13. The present IaC overlays are pack- or region-shaped, not deployment-context-shaped.
14. The canonical OpenTofu context path names are absent: `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`.
15. The OCI Always Free profile path is absent: `iac/oci-guest/always-free/`.
16. The master-plan sequencing source says the canonical path for OCI Always Free is `iac/oci-guest/always-free/`: `specs/master-plan-sequencing.json:857-865`.

#### §3.4.T Tier retirement candidates

1. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:13`.
2. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:21`.
3. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:22`.
4. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:31`.
5. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:37`.
6. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:38`.
7. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:47`.
8. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:53`.
9. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:54`.
10. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:63`.
11. Explicit retired term candidate: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:81`.
12. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:13`.
13. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:37`.
14. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:43`.
15. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:45`.
16. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:69`.
17. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:71`.
18. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:92`.
19. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:96`.
20. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:98`.
21. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:108`.
22. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:110`.
23. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:118`.
24. Explicit retired term candidate: `microservices/consent-graph/capability-ladders/tier-matrix.md:124`.
25. Explicit retired term candidate: `microservices/consent-graph/faqs/data-steward-faq.md:94`.
26. Explicit retired term candidate: `microservices/consent-graph/tutorials/draft-and-activate-data-sharing-agreement.md:15`.
27. Explicit retired term candidate: benchmark command flag `--tenant-class paid` at `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:95-99`.
28. Default classification: Wave 15J retirement candidate, severity P2 documentation gap.
29. Rationale: current directive retires retired four-label ladder feature stratification and replaces it with tenant classes plus uniform quality.
30. Remediation shape: replace feature-ladder segmentation with tenant_class overlays and deployment-context capacity overlays.
31. Remediation constraint: do not re-create a fourth capability-ladder retirement deltas report.
32. Retire candidate directory: `microservices/consent-graph/capability-ladders/`.
33. Retire candidate benchmark structure: all service benchmark rows that segment Oyatie targets by feature tier.
34. Retire candidate operational text: FAQ/tutorial lines that require a named feature tier to use fast-path or onboarding surfaces.

#### §3.4.C Tenant-class adoption gaps

1. Search result: no `tenant_class` string appears under `microservices/consent-graph/`.
2. Search result: no `demo_trial` string appears under `microservices/consent-graph/`.
3. Search result: no `revenue_share` string appears under `microservices/consent-graph/`.
4. Search result: meaningful `paid` tenant-class semantics do not appear under `microservices/consent-graph/`.
5. Existing service semantics are capability-ladder based, not tenant-class based.
6. Manifest capability classification uses T0/T2/T3 instead of tenant_class: `microservices/consent-graph/manifest.json:118-138`.
7. Manifest keeps `tenant_classs` and `tier_classification` keys: `microservices/consent-graph/manifest.json:348-377`.
8. OpenAPI enforcement request carries `tenant_class`, not tenant_class: `microservices/consent-graph/contracts/openapi/consent-graph.yaml:458-485`.
9. Proto enforcement context carries `TenantClass`, not tenant_class: `microservices/consent-graph/contracts/proto/consent-graph.proto:122-128` and `microservices/consent-graph/contracts/proto/consent-graph.proto:263-268`.
10. Cedar projection policy forbids based on `tenant_class`, not tenant_class or billing class: `microservices/consent-graph/policy/cross-tenant-projection.cedar:42-50`.
11. The adoption gap is architectural because enforcement, contracts, policy, benchmarks, docs, and FAQ all need a consistent replacement model.
12. The required new semantics are not feature quality levels; they are economic/operational tenant classes with uniform quality.
13. `demo_trial` should bind to OCI Always Free profile and usage caps.
14. `paid` should bind to contractual SLOs, compliance packs, BYOK, and scalable resource allocation.
15. `revenue_share` should bind to at-cost or zero-margin substrate with revenue-share billing controls.
16. Consent-graph currently has no place where those semantics are enforced or audited.

### §3.5 Dimension 5 — Counterpart and union-coverage alignment

1. Verdict: service has strong B2B data-sharing differentiation, but existing counterpart docs do not fully match the required top-three set.
2. Chat queue confirmed the required counterpart set as OneTrust, TrustArc, and Cookiebot: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`.
3. Existing competitor matrix centers Snowflake, Databricks, open banking, HIE, Hyperledger, marketplace, and EDI comparisons, not the required top-three union: `microservices/consent-graph/competitor-parity-matrix.md:10-27`.
4. Existing competitor matrix lists OneTrust and TrustArc only as possible B2C adjacency in year two: `microservices/consent-graph/competitor-parity-matrix.md:86-96`.
5. Existing benchmark includes OneTrust and TrustArc but also Snowflake, Databricks, AWS Data Exchange, and DIY, while omitting Cookiebot from the measured set: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:9-16`.
6. Migration playbook covers OneTrust and TrustArc in detail: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:9-27`.
7. Migration playbook explicitly says cookie-banner UI is not shipped as of 2026-05: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:102-107`.
8. Cookiebot's surface is therefore not covered by the current service except as external workflow/tooling retained during migration.
9. OneTrust and TrustArc privacy-workflow surfaces overlap consent-graph on consent records, preferences, DSAR, reporting, and integrations.
10. Cookiebot overlaps on cookie/tracker scanning, consent banners, consent logs, TCF, and Google Consent Mode.
11. Consent-graph leads the top-three on cross-tenant agreement enforcement, bilateral audit chain, revocation fanout, and zero-copy projection.
12. Consent-graph trails the top-three on website cookie scanning, banner rendering, IAB TCF web-CMP protocol, Google Consent Mode, and marketing preference center UX.
13. Feature parity therefore requires an additive surface or explicit ownership handoff for cookie CMP and preference center capabilities.

### §3.6 Dimension 6 — Multi-context deployment

1. Verdict: P1 gap.
2. Canonical contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1736-1981`.
3. ADR-0328 requires every manifest to name supported deployment contexts as an array and explain any N/A context: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2083`.
4. The service manifest contains dependencies, capabilities, SLOs, plans, regulatory packs, and metadata but no `deployment_contexts` array: `microservices/consent-graph/manifest.json:1-415`.
5. The PRD describes global use cases and compliance regions but does not declare the six deployment contexts: `microservices/consent-graph/PRD.md:91-144`.
6. The multi-region document exists, but multi-region is not the same as multi-context deployability.
7. The IaC directories are only `iac/helm` and `iac/kustomize`.
8. The context directories are absent: `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`.
9. The current audit does not find an explicit N/A justification for any context.
10. Because the user-supplied dispatch default says all six contexts unless audit finds otherwise, the audit records absence of evidence, not a valid exclusion.
11. Multi-context provider-agnostic memory says absence of multi-context specificity is a P1 architectural gap for new microservice docs: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md:32-38`.
12. Classification: P1, because the service cannot honestly claim the six deployment contexts from current artifacts.

### §3.7 Dimension 7 — OpenTofu IaC and OCI Always Free profile

1. Verdict: P1 gap.
2. Canonical substrate is OpenTofu, not Terraform, Pulumi, CloudFormation, or ARM templates: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2307`.
3. Canonical service IaC directories are per context: `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2291`.
4. Canonical OCI Always Free profile path is `iac/oci-guest/always-free/`: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2293-2307`.
5. The service IaC tree contains Helm and Kustomize directories only.
6. `find microservices/consent-graph/iac -maxdepth 4 -type d` shows `helm`, `helm/consent-graph`, `helm/consent-graph/templates`, `kustomize`, `kustomize/base`, and regional overlays.
7. No `.tf` files exist under `microservices/consent-graph/iac/`.
8. No `versions.tf`, `variables.tf`, `outputs.tf`, `main.tf`, or context README files exist under canonical context directories because those directories do not exist.
9. Helm/Kustomize can remain runtime packaging, but they do not satisfy the OpenTofu substrate requirement.
10. The OCI Always Free memory requires maximizing the free profile and wiring service modules through `iac/oci-guest/always-free/`: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:10-73`.
11. The service has no OCI Always Free profile or demo_trial infrastructure expression.
12. Classification: P1, because tenant onboarding and deployability cannot be verified through canonical IaC.

### §3.8 Dimension 8 — OS support matrix

1. Verdict: P1 gap.
2. OS doctrine requires Linux, Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, Alma, CentOS Stream, Amazon Linux, Flatcar, Photon, macOS Apple Silicon, and scoped desktop/mobile support where relevant: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-31`.
3. OS doctrine requires a per-service supported-OS manifest schema: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-72`.
4. No `supported-oses.json` file exists under `microservices/consent-graph/`.
5. No equivalent OS manifest appears in `manifest.json`.
6. No per-OS CI or package-output matrix appears under this service path.
7. The service docs contain Kubernetes, Helm, Kustomize, Postgres, Pulsar, Valkey, and OpenBao assumptions.
8. Those runtime assumptions are compatible with Linux server contexts but do not document every supported OS.
9. The service is backend-heavy and does not appear to have Swift, Kotlin, WinUI, or Leptos frontend surfaces under its own path.
10. The lack of OS manifest blocks an auditor from determining whether the service supports all required server OSes, only Kubernetes cells, or only a narrower internal target.
11. Classification: P1, because canonical OS support is an explicit Wave 3 audit dimension.

### §3.9 Dimension 9 — Rust-strict language posture

1. Verdict: source-file scan passes; SDK wording needs tightening.
2. Rust-strict doctrine forbids Python, JavaScript app logic, TypeScript app logic, Ruby, PHP, Java, Scala, Groovy, Go, and F# under service deliverables: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`.
3. Scan command over forbidden file extensions returned no files under `microservices/consent-graph/`.
4. Allowed surfaces present include Markdown, YAML, JSON, OpenAPI, AsyncAPI, proto, Cedar, OpenSLO, Helm YAML, and Kustomize YAML.
5. No Rust implementation source files are present under the service path.
6. Absence of forbidden source files is not equivalent to implementation completeness.
7. The PRD's SDK goal says typed Rust, TypeScript, and Python clients: `microservices/consent-graph/PRD.md:57`.
8. Current Rust-strict doctrine allows generated SDK surfaces only when bounded by the sanctioned generator/provenance model; this service does not document that boundary in-path.
9. No `developer-sdk` or Stainless generator evidence is visible under consent-graph.
10. The reference implementation is Markdown for a Rust SDK subscription path, which is allowed: `microservices/consent-graph/reference-implementations/projection-subscribe-rust-sdk.md`.
11. Classification: language source scan P3 pass; SDK wording P2 documentation gap.

## §4 Findings table

| ID | Severity | Finding | Evidence | Required correction shape |
|---|---:|---|---|---|
| CG-AUD-001 | P1 | Six deployment contexts are not declared or evidenced for consent-graph. | ADR context requirement `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1736-2118`; manifest lacks context array `microservices/consent-graph/manifest.json:1-415`; IaC tree only Helm/Kustomize. | Add service-level deployment_contexts and per-context N/A or support evidence. |
| CG-AUD-002 | P1 | OpenTofu per-context IaC and OCI Always Free profile are absent. | OpenTofu directory contract `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2307`; actual IaC paths are Helm/Kustomize only. | Add OpenTofu modules for supported contexts, including `iac/oci-guest/always-free/`. |
| CG-AUD-003 | P1 | Supported OS manifest and OS-specific evidence are absent. | OS manifest doctrine `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-78`; no `supported-oses.json` in inventory. | Add per-service OS support manifest and verification mapping. |
| CG-AUD-004 | P2 | Explicit retired feature-ladder vocabulary remains. | Candidate list in §3.4.T, including `microservices/consent-graph/capability-ladders/tier-matrix.md:13-124` and benchmark lines `13-99`. | Retire B/S/G/P content during Wave 15J and replace with tenant_class plus deployment overlays. |
| CG-AUD-005 | P2 | Tenant-class semantics are absent while capability-ladder fields remain in contracts and policy. | `microservices/consent-graph/contracts/openapi/consent-graph.yaml:458-485`; `microservices/consent-graph/contracts/proto/consent-graph.proto:122-128`; `microservices/consent-graph/policy/cross-tenant-projection.cedar:42-50`. | Introduce `tenant_class` semantics without feature quality stratification. |
| CG-AUD-006 | P2 | Existing parity and benchmark docs do not match required OneTrust / TrustArc / Cookiebot union. | Chat set `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`; existing matrix `microservices/consent-graph/competitor-parity-matrix.md:10-27`; benchmark omits Cookiebot `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:9-16`. | Rebase parity and benchmark evidence on the required top-three union. |
| CG-AUD-007 | P2 | Top-level README and cross-microservice handoff document are absent. | Inventory has no `README.md` or `cross-microservice-handoffs.md`. | Add concise service orientation and dependency handoff contract. |
| CG-AUD-008 | P2 | Implementation evidence is absent despite manifest GA posture. | No `src/` or `tests/`; manifest says all core IPs are GA `microservices/consent-graph/manifest.json:196-211`; parent wiring remains listed `microservices/consent-graph/manifest.json:342-345`. | Align status with implementation evidence or land crates/tests. |
| CG-AUD-009 | P2 | SDK language promise conflicts with Rust-strict boundary unless generator provenance is added. | PRD typed Rust/TypeScript/Python client goal `microservices/consent-graph/PRD.md:57`; Rust-strict doctrine `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-66`. | Route non-Rust SDKs through sanctioned generated SDK ownership, not hand-authored service code. |
| CG-AUD-010 | P2 | Architecture/compliance docs contain generated-content residue that should be reviewed before maturity claims. | Architecture content-pass warning `microservices/consent-graph/ARCHITECTURE.md:3`; repeated compliance anchors `microservices/consent-graph/compliance.md:160-1027`. | Run a substance pass focused on unique service behavior and remove stale generated framing. |
| CG-AUD-011 | P2 | Cookie/web CMP surface is acknowledged as missing, creating a top-three counterpart gap against Cookiebot. | Migration playbook says no tenant-facing cookie-banner ships as of 2026-05 `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:102-107`. | Either assign cookie CMP to another service explicitly or add consent-graph handoff contracts. |
| CG-AUD-012 | P2 | Existing benchmark numbers use retired feature-ladder segmentation. | Benchmark rows and command flag `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:13-99`. | Use one industry-leader target set plus deployment-context and tenant-class overlays. |
| CG-AUD-013 | P3 | Generic tier vocabulary remains outside explicit retired terms. | `microservices/consent-graph/ARCHITECTURE.md:22-30`; `microservices/consent-graph/cost-budget.md:88-92`; `microservices/consent-graph/compliance.md:173-180`. | Rename generic commercial or metadata usage where it risks confusion with retired feature tiers. |
| CG-AUD-014 | P3 | IaC overlays are pack-shaped rather than context-shaped, which is useful but insufficient. | Compliance says overlays live under `iac/kustomize/overlays/<pack>/` `microservices/consent-graph/compliance.md:7-9`; inventory confirms regional/pack overlays. | Preserve pack overlays as runtime config while adding canonical context modules. |

Finding count summary: P0=0, P1=3, P2=9, P3=2.

## §5 Open questions

1. Should `consent-graph` own any cookie-banner, tracker-scanning, IAB TCF, or Google Consent Mode surface, or should that be assigned to another microservice with a formal handoff?
2. Should OneTrust/TrustArc DSAR workflow migration remain in workflow-engine only, or should consent-graph expose a narrower DSAR cascade contract for partner revocation receipts?
3. Should the capability-ladder fields in OpenAPI, proto, policy, manifest, and capability YAML be renamed directly to `tenant_class`, or should capability risk class and tenant class be separated?
4. What is the canonical service-local representation for `demo_trial`, `paid`, and `revenue_share` in enforcement and cost controls?
5. Which team owns the OCI Always Free profile for consent-graph: consent-graph directly, cloud-iac, or a shared context module?
6. Is consent-graph expected to deploy in all six contexts immediately, or should any context receive an explicit N/A with business rationale?
7. Should the existing Helm/Kustomize content become generated output from OpenTofu modules, or remain separate runtime packaging consumed by OpenTofu?
8. What is the accepted evidence path for source implementation when service docs list many crates but the service directory has no `src/` and no tests?
9. Should the manifest's GA status be downgraded until workspace wiring and code evidence exist?
10. Should the existing benchmark file be retired or kept as historical evidence after the new non-tier benchmark document lands?
11. Which public metrics can be safely claimed for OneTrust, TrustArc, and Cookiebot without private testing or unsupported assumptions?
12. Should performance benchmark estimates be stored separately from published public numbers to avoid confusing evidence levels?
13. Should `supported-oses.json` be authored per service or generated from a shared service-class registry?
14. Should compliance pack overlays remain in this service path after OpenTofu context modules land, or move to a shared pack registry?
15. Should Rust-strict SDK policy allow generated Python/TypeScript clients to be referenced from service PRDs, or should those references live only in developer-sdk docs?

## §6 Evidence ledger and audit rationale

1. Evidence class: product purpose.
2. Product purpose evidence is strong because the PRD states the problem and five simultaneous requirements: auditability, revocability, sovereignty, narrow scope, and real time.
3. Product purpose evidence cite: `microservices/consent-graph/PRD.md:17-31`.
4. Evidence class: goals and non-goals.
5. Goals are specific and measurable across DSA records, Cedar enforcement, bilateral audit, revocation, projections, and partner directory.
6. Goals cite: `microservices/consent-graph/PRD.md:48-59`.
7. Non-goals prevent overclaiming marketplace, billing, audit-chain, and ontology ownership.
8. Non-goals cite: `microservices/consent-graph/PRD.md:63-73`.
9. Evidence class: customer and operator metrics.
10. The PRD includes SLO-grade numbers rather than purely qualitative aspirations.
11. Metrics cite: `microservices/consent-graph/PRD.md:77-87`.
12. Evidence class: scale envelope.
13. The PRD and capacity model agree on active agreements, new agreements, revocations, projection events, peers, and enforcement throughput.
14. Scale cite: `microservices/consent-graph/PRD.md:193-198`.
15. Capacity cite: `microservices/consent-graph/capacity-model.md:7-18`.
16. Evidence class: contracts.
17. OpenAPI covers agreement lifecycle, enforcement, break-glass, partner-directory, and revocation receipts.
18. OpenAPI cite: `microservices/consent-graph/contracts/openapi/consent-graph.yaml:33-285`.
19. AsyncAPI covers lifecycle events, revocation priority events, audit bridge events, and projection events.
20. AsyncAPI cite: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml:20-65`.
21. Proto covers six internal services and the enforcement context.
22. Proto cite: `microservices/consent-graph/contracts/proto/consent-graph.proto:18-59`.
23. Evidence class: policy enforcement.
24. Cross-tenant projection Cedar policy includes base permit, field narrowing, region forbid, capability ceiling, revocation freshness, rate-limit, and suspended-partner denial.
25. Projection policy cite: `microservices/consent-graph/policy/cross-tenant-projection.cedar:5-87`.
26. Aggregate policy includes k-anonymity, differential privacy, expiry, sensitive-category, and DP-budget forbids.
27. Aggregate policy cite: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar:5-75`.
28. Break-glass policy requires emergency purpose, expiry, counter-signer attestation, session cap, and revoked-agreement denial.
29. Break-glass cite: `microservices/consent-graph/policy/break-glass-healthcare.cedar:1-71`.
30. Deny-all fallback is explicit, not merely implicit.
31. Deny-all cite: `microservices/consent-graph/policy/deny-all-fallback.cedar:1-26`.
32. Evidence class: SLO specificity.
33. Grant latency SLO uses a 2.0 second bucket and 0.95 target.
34. Grant SLO cite: `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml:21-33`.
35. Projection freshness SLO uses a 500 ms objective and 0.95 target.
36. Projection SLO cite: `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml:27-33`.
37. Cedar evaluation SLO uses a 0.01 second bucket and 0.99 target.
38. Cedar SLO cite: `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml:21-33`.
39. Revocation SLO uses a 1.0 second bucket and 0.99 target.
40. Revocation SLO cite: `microservices/consent-graph/slos/revocation-propagation-latency.openslo.yaml:22-34`.
41. Evidence class: failure behavior.
42. Failure modes consistently prefer fail-closed behavior for enforcement uncertainty.
43. Failure behavior cite: `microservices/consent-graph/failure-modes.md:54-83`.
44. Sovereignty failures auto-suspend and burn zero-violation SLO.
45. Sovereignty failure cite: `microservices/consent-graph/failure-modes.md:140-144`.
46. Partner offboarding cascades revocation of active agreements.
47. Partner offboarding cite: `microservices/consent-graph/failure-modes.md:180-184`.
48. Evidence class: incident response.
49. Incident severity matrix makes sovereignty violation, audit-chain divergence, consent forgery, and deny-rate spike P0.
50. Incident cite: `microservices/consent-graph/incident-response.md:7-14`.
51. Incident decision tree ties alerts to concrete runbooks.
52. Decision-tree cite: `microservices/consent-graph/incident-response.md:31-43`.
53. Evidence class: compliance scope.
54. Compliance map covers KR, EU, US, US healthcare, JP, SG, AU, IN, BR, AE, and KSA.
55. Compliance cite: `microservices/consent-graph/compliance.md:11-14`.
56. Compliance map includes explicit GDPR Art. 28, Art. 44-49, Art. 17, and DPIA references.
57. GDPR cite: `microservices/consent-graph/compliance.md:31-45`.
58. Compliance map includes HIPAA min-necessary, accounting of disclosures, break-glass, and BAA sections.
59. HIPAA cite: `microservices/consent-graph/compliance.md:78-98`.
60. Evidence class: capacity and cost.
61. Capacity model gives per-pod, per-region, and global throughput for consent-graph-app, enforcement-app, revocation-app, projection-gateway-worker, and audit-bridge-worker.
62. Capacity cite: `microservices/consent-graph/capacity-model.md:19-70`.
63. Cost budget gives per-region monthly cost, global yearly cost, per-unit costs, and budget allocation.
64. Cost cite: `microservices/consent-graph/cost-budget.md:8-79`.
65. Evidence class: migration.
66. Migration playbook correctly separates OneTrust/TrustArc workflow tools from consent-graph enforcement substrate.
67. Migration cite: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:13-27`.
68. Migration playbook admits cookie-banner absence.
69. Cookie-banner absence cite: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:102-107`.
70. Evidence class: counterpart drift.
71. Existing competitor matrix does not use the required OneTrust/TrustArc/Cookiebot union as its primary matrix.
72. Existing matrix cite: `microservices/consent-graph/competitor-parity-matrix.md:10-27`.
73. Existing benchmark omits Cookiebot from its measured table and includes non-required comparators.
74. Existing benchmark cite: `microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:9-16`.
75. Evidence class: deployment gap.
76. The only service-local IaC directories are Helm and Kustomize.
77. IaC inventory cite: `microservices/consent-graph/iac/helm/consent-graph/Chart.yaml` and `microservices/consent-graph/iac/kustomize/base/kustomization.yaml`.
78. Canonical context directories are not present in the service path.
79. Canonical context requirement cite: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2291`.
80. Evidence class: OCI Always Free profile gap.
81. The required `iac/oci-guest/always-free/` directory is absent.
82. OCI profile requirement cite: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2293-2307`.
83. Evidence class: OS support gap.
84. The service path lacks `supported-oses.json`.
85. OS manifest requirement cite: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-72`.
86. Evidence class: language scan.
87. Forbidden-extension scan found no files for Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, or F# under the service path.
88. Rust-strict requirement cite: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-66`.
89. Evidence class: implementation gap.
90. No `src/` and no `tests/` are visible under the service path.
91. Manifest still claims GA for core IPs.
92. GA cite: `microservices/consent-graph/manifest.json:196-211`.
93. Parent wiring gap cite: `microservices/consent-graph/manifest.json:342-345`.
94. Evidence class: tenant-class gap.
95. No `tenant_class`, `demo_trial`, or `revenue_share` appears under the service path.
96. Capability-tier contract cite: `microservices/consent-graph/contracts/openapi/consent-graph.yaml:458-485`.
97. Capability-tier proto cite: `microservices/consent-graph/contracts/proto/consent-graph.proto:122-128`.
98. Capability-tier policy cite: `microservices/consent-graph/policy/cross-tenant-projection.cedar:42-50`.
99. Evidence class: retired vocabulary.
100. Explicit retired vocabulary was found in the benchmark, capability matrix, FAQ, and tutorial.
101. Retired vocabulary doctrine cite: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_ladder_2026_05_20.md:10-43`.
102. Evidence class: chat history.
103. Chat history confirmed this µservice's counterpart tuple as OneTrust / TrustArc / Cookiebot.
104. Chat counterpart cite: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`.
105. Chat history confirmed the dropped fourth deliverable and non-tier benchmark shape.
106. Chat benchmark cite: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16330`.
107. Audit rationale: product architecture is not the blocker.
108. Audit rationale: canonical platform conformance is the blocker.
109. Audit rationale: counterpart union coverage needs service-boundary clarity.
110. Audit rationale: tenant_class migration is a documentation and contract gap, not a request to reduce quality by customer class.
111. Audit rationale: OpenTofu and OS gaps are P1 because they block deployability claims.
112. Audit rationale: retired feature-ladder references are P2 because behavior can remain coherent while documents and schemas need migration.
113. Audit rationale: missing source/tests are P2 because the task audits artifacts but intern-buildability remains unproven.
114. Audit rationale: no P0 was assigned because no evidence showed current user-data exposure or destructive behavior in this audit pass.
115. Audit stop condition: all three reports landed, line floors verified, and no fourth deliverable authored.

<!-- ORCHESTRATOR REPORT
  µservice: consent-graph
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/consent-graph/coherence-audit-2026-05-20.md: 641 lines
    - /Users/jasonlee/oyatie/microservices/consent-graph/feature-parity-matrix-2026-05-20.md: 428 lines
    - /Users/jasonlee/oyatie/microservices/consent-graph/performance-benchmark-numbers-2026-05-20.md: 328 lines
  inventory_files_seen: 135
  inventory_lines_read: 22690
  chat_history_matches_processed: 91
  findings_p0: 0
  findings_p1: 3
  findings_p2: 9
  findings_p3: 2
  tier_retirement_candidates_found: 27; cites: microservices/consent-graph/benchmarks/consent-graph-vs-onetrust-vs-snowflake-share-vs-databricks-clean-room.md:13,21,22,31,37,38,47,53,54,63,81,95-99; microservices/consent-graph/capability-ladders/tier-matrix.md:13,37,43,45,69,71,92,96,98,108,110,118,124; microservices/consent-graph/faqs/data-steward-faq.md:94; microservices/consent-graph/tutorials/draft-and-activate-data-sharing-agreement.md:15
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/revenue_share semantics found, while manifest/contracts/proto/policy still express capability-ladder fields
  top_3_counterparts_confirmed: OneTrust / TrustArc / Cookiebot
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1397
-->
