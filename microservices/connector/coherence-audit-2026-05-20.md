# Ownership-Coherence Audit - 2026-05-20

Audit owner: single-agent connect audit.
Target microservice path: `microservices/connector/`.
Counterpart coverage bar for this batch: Twilio, Sendbird, Stream.
Deliverable set for this batch: coherence audit, feature parity matrix, performance benchmark numbers.
Retired deliverable: capability-tier deltas, removed by the 2026-05-20 no-capability-tiers directive.
Audit stop condition: three reports landed, line floors met, no new capability-tier scaffolding introduced, and scoped verification run.

Citation anchor block:
- Canonical sequence and audit dimensions: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-4235`.
- Master-plan deployment, IaC, OS, language, and OCI profile controls: `specs/master-plan-sequencing.json:704-868`.
- Brief-template microservice audit, multi-context, IaC, OS, Rust, and anti-pattern anchors: `docs/standards/brief-template.md:666-1854`.
- Constraint memory package: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-92`.
- Tier retirement and tenant-class replacement package: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tracks_2026_05_20.md:10-59` and `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-142`.

## §1 Purpose

1. This audit evaluates whether `connector` is internally coherent as a microservice ownership surface, not whether the whole Oyatie estate is ready.
2. The active product reading is that `connector` is the integration substrate for connector catalog, OAuth broker, webhook receiver, signature verification, payload canonicalization, connector adapter, data mapping, and retry/DLQ behavior.
3. The product reading is grounded in `microservices/connector/PRD.md:29-39`, which says the service owns shared connector integration so workflow-engine, marketplace, agents, and ops products do not reimplement vendor integration.
4. The same reading is grounded in `microservices/connector/README.md:16-32`, which defines the service as an integration substrate and says it is not workflow-engine, api-gateway, or a credential store.
5. Chat history confirms the same boundary: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:6706-6709` describes connect as the integration substrate with connector adapters, OAuth dance, and webhook receiver substrate.
6. Later chat history sharpens the boundary: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8923-8926` says connect sits at the integration/event notification boundary and is not an iPaaS, workflow-trigger surface, or data-pipeline surface.
7. This audit therefore treats connector substrate coherence as the primary ownership question.
8. It treats cross-tenant chat federation material as historical residue unless an artifact ties it back to connector/OAuth/webhook substrate ownership.
9. It treats umbrella-retirement material as historical residue unless an artifact cleanly explains the transition from retirement surface to active integration service.
10. It treats deployable contexts as all six canonical contexts unless the audited artifact set proves a narrower scope, because the user supplied all six as the expected default and ADR-0328 D-15 makes all six the default Phase 0 and Phase 1 posture.
11. It treats OpenTofu as the only acceptable IaC substrate, because ADR-0328 D-16 and `specs/master-plan-sequencing.json:747-776` forbid Terraform, Pulumi, CloudFormation, ARM templates, and hand-run scripts as canonical deployment mechanisms.
12. It treats Rust as the backend and automation language, with Swift, Kotlin, WinUI 3, and Leptos/WASM SSR plus selective hydration as frontend exceptions, because ADR-0328 D-18 and the Rust-strict memory file define that boundary.
13. It treats the three tenant classes requested in this batch as `demo_trial`, `paid`, and `revenue_share`, because the prompt for this audit is the current task authority.
14. It notes a source conflict: the tenant-class memory file states two tenant classes and billing components, while this batch prompt names three tenant classes; this report follows the batch prompt while citing the memory file as doctrine history.
15. It treats capability-tier language as a retirement candidate, not an implementation scaffold, because `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tracks_2026_05_20.md:10-45` retires the old feature-tier model.
16. It evaluates all findings against the nine dimensions in ADR-0328 D-20.
17. It uses severity P0 for immediate correctness or safety blockers, P1 for canonical deployment/product-contract blockers, P2 for documentation or readiness gaps that would mislead implementation, and P3 for lower-risk cleanup and clarity improvements.
18. It does not create a fourth deliverable.
19. It does not create any new source, tests, contracts, IaC, or shared docs outside `microservices/connector/`.
20. It does not commit changes.

## §2 Inventory

1. Inventory command: `rg --files microservices/connector | sort`.
2. Inventory file count: 182 files.
3. Inventory line count across service path before this audit content: 31,551 lines.
4. Chat-history raw `connector` matches counted: 563.
5. Chat-history filtered service-specific matches processed: 355.
6. Code implementation folders sampled by existence: no `microservices/connector/src/` directory was present.
7. Test folders sampled by existence: no `microservices/connector/tests/` directory was present.
8. IaC context folders sampled by existence: no canonical `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-iaas/` folders were present.
9. OCI Always Free profile folder sampled by existence: no `microservices/connector/iac/oci-guest/always-free/` folder was present.
10. `supported-oses.json` sampled by existence: no service-local file was present.
11. `cross-microservice-handoffs.md` sampled by existence: no service-local file was present.
12. Forbidden backend language file scan for `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, and `.fs` returned no source files under the service path.
13. Full file inventory follows.

### §2.1 Full File Inventory

1. `microservices/connector/ARCHITECTURE.md`
2. `microservices/connector/AUDIT-FINDINGS-2026-05-18.json`
3. `microservices/connector/AUDIT-FINDINGS-2026-05-20.json`
4. `microservices/connector/CHANGELOG.md`
5. `microservices/connector/IP-001-connect-retirement-design-readiness.md`
6. `microservices/connector/IP-002-connector-catalog-domain-kernel.md`
7. `microservices/connector/IP-003-oauth-broker-domain-kernel.md`
8. `microservices/connector/IP-004-webhook-receiver-domain.md`
9. `microservices/connector/IP-005-connector-adapter-domain.md`
10. `microservices/connector/IP-006-data-mapping-domain.md`
11. `microservices/connector/IP-007-retry-dlq-domain.md`
12. `microservices/connector/IP-008-rest-surfaces.md`
13. `microservices/connector/IP-009-connector-catalog-seed.md`
14. `microservices/connector/IP-010-iac-postgres-openbao.md`
15. `microservices/connector/IP-011-slos-dashboards-observability.md`
16. `microservices/connector/IP-012-wave2-connectors.md`
17. `microservices/connector/IP-013-connector-adapter-trait.md`
18. `microservices/connector/IP-014-compliance-critical-path.md`
19. `microservices/connector/IP-015-connector-adapter-trait-doc.md`
20. `microservices/connector/IP-journey-j100-pack-rollout-first-action.md`
21. `microservices/connector/IP-journey-j102-external-rail-adapter.md`
22. `microservices/connector/IP-journey-j103-external-rail-adapter.md`
23. `microservices/connector/IP-journey-j104-external-rail-adapter.md`
24. `microservices/connector/IP-journey-j106-external-rail-adapter.md`
25. `microservices/connector/IP-journey-j107-external-rail-adapter.md`
26. `microservices/connector/IP-journey-j11-offline-shell-state.md`
27. `microservices/connector/IP-journey-j120-bank-liquidity-provider-adapter.md`
28. `microservices/connector/IP-journey-j121-bank-core-adapter.md`
29. `microservices/connector/IP-journey-j122-bank-rail-payout-adapter.md`
30. `microservices/connector/IP-journey-j128-irs-mef-state-adapters.md`
31. `microservices/connector/IP-journey-j136-benefits-provider-bulk-push-and-reconcile.md`
32. `microservices/connector/IP-journey-j144-job-board-adapters.md`
33. `microservices/connector/IP-journey-j148-carrier-and-recycler-adapters.md`
34. `microservices/connector/IP-journey-j149-platform-adapter-roster.md`
35. `microservices/connector/IP-journey-j26-device-ingest.md`
36. `microservices/connector/IP-journey-j29-shipping-label-ingest.md`
37. `microservices/connector/IP-journey-j37-adp-payroll-export.md`
38. `microservices/connector/IP-journey-j44-ehr-export.md`
39. `microservices/connector/IP-journey-j46-pharmacy-adapter.md`
40. `microservices/connector/IP-journey-j47-insurance-claim-submit.md`
41. `microservices/connector/IP-journey-j48-adp-kr-export.md`
42. `microservices/connector/IP-journey-j49-external-marketplace-adapter.md`
43. `microservices/connector/IP-journey-j52-kr-postal-api.md`
44. `microservices/connector/IP-journey-j56-scim-provisioning.md`
45. `microservices/connector/IP-journey-j62-pharmacy-and-insurance-api.md`
46. `microservices/connector/IP-journey-j66-regulator-api.md`
47. `microservices/connector/IP-journey-j91-us-msb-mtl-overlay.md`
48. `microservices/connector/IP-journey-j92-br-lgpd-us-parent-dsar.md`
49. `microservices/connector/IP-journey-j93-in-dpdpa-rbi-overlay.md`
50. `microservices/connector/IP-journey-j94-sox404-public-company-controls.md`
51. `microservices/connector/IP-journey-j95-iso27001-soc2-annual-audit.md`
52. `microservices/connector/IP-journey-j96-ksa-uae-mena-onboarding.md`
53. `microservices/connector/IP-journey-j97-sg-pdpa-mas-tenant.md`
54. `microservices/connector/IP-journey-j98-au-privacy-apra-cps234.md`
55. `microservices/connector/IP-journey-j99-multi-pack-conflict-resolution.md`
56. `microservices/connector/PHASE-01-INTEGRATION-SUBSTRATE-FOUNDATION.md`
57. `microservices/connector/PRD.md`
58. `microservices/connector/README.md`
59. `microservices/connector/RETIREMENT-PLAN.md`
60. `microservices/connector/backfill-replay.md`
61. `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md`
62. `microservices/connector/capabilities/connector-invoke.yaml`
63. `microservices/connector/capabilities/oauth-grant-initiate.yaml`
64. `microservices/connector/capabilities/umbrella-retirement-readiness.yaml`
65. `microservices/connector/capabilities/webhook-endpoint-register.yaml`
66. `microservices/connector/capability-tiers/tier-matrix.md`
67. `microservices/connector/capacity-model.md`
68. `microservices/connector/catalog/connectors/airtable.yaml`
69. `microservices/connector/catalog/connectors/asana.yaml`
70. `microservices/connector/catalog/connectors/bigquery.yaml`
71. `microservices/connector/catalog/connectors/clickup.yaml`
72. `microservices/connector/catalog/connectors/datadog.yaml`
73. `microservices/connector/catalog/connectors/discord.yaml`
74. `microservices/connector/catalog/connectors/dropbox.yaml`
75. `microservices/connector/catalog/connectors/github.yaml`
76. `microservices/connector/catalog/connectors/gitlab.yaml`
77. `microservices/connector/catalog/connectors/gmail.yaml`
78. `microservices/connector/catalog/connectors/google-drive.yaml`
79. `microservices/connector/catalog/connectors/google-sheets.yaml`
80. `microservices/connector/catalog/connectors/hubspot.yaml`
81. `microservices/connector/catalog/connectors/jira.yaml`
82. `microservices/connector/catalog/connectors/kakaopay.yaml`
83. `microservices/connector/catalog/connectors/launchdarkly.yaml`
84. `microservices/connector/catalog/connectors/linear.yaml`
85. `microservices/connector/catalog/connectors/mailgun.yaml`
86. `microservices/connector/catalog/connectors/mixpanel.yaml`
87. `microservices/connector/catalog/connectors/monday.yaml`
88. `microservices/connector/catalog/connectors/notion.yaml`
89. `microservices/connector/catalog/connectors/opsgenie.yaml`
90. `microservices/connector/catalog/connectors/outlook.yaml`
91. `microservices/connector/catalog/connectors/pagerduty.yaml`
92. `microservices/connector/catalog/connectors/postgres-direct.yaml`
93. `microservices/connector/catalog/connectors/salesforce.yaml`
94. `microservices/connector/catalog/connectors/segment.yaml`
95. `microservices/connector/catalog/connectors/sendgrid.yaml`
96. `microservices/connector/catalog/connectors/sentry.yaml`
97. `microservices/connector/catalog/connectors/shopify.yaml`
98. `microservices/connector/catalog/connectors/slack.yaml`
99. `microservices/connector/catalog/connectors/snowflake.yaml`
100. `microservices/connector/catalog/connectors/stripe.yaml`
101. `microservices/connector/catalog/connectors/toss-payments.yaml`
102. `microservices/connector/catalog/connectors/trello.yaml`
103. `microservices/connector/catalog/connectors/twilio.yaml`
104. `microservices/connector/catalog/oya-connector-adapter-domain.yaml`
105. `microservices/connector/catalog/oya-connector-catalog-api.yaml`
106. `microservices/connector/catalog/oya-connector-catalog-domain.yaml`
107. `microservices/connector/catalog/oya-connector-catalog-kernel.yaml`
108. `microservices/connector/catalog/oya-connector-catalog-usecase.yaml`
109. `microservices/connector/catalog/oya-connector-data-mapping-domain.yaml`
110. `microservices/connector/catalog/oya-connector-oauth-broker-domain.yaml`
111. `microservices/connector/catalog/oya-connector-oauth-broker-kernel.yaml`
112. `microservices/connector/catalog/oya-connector-payload-canonicalization-domain.yaml`
113. `microservices/connector/catalog/oya-connector-retry-dlq-domain.yaml`
114. `microservices/connector/catalog/oya-connector-signature-verification-kernel.yaml`
115. `microservices/connector/catalog/oya-connector-webhook-receiver-domain.yaml`
116. `microservices/connector/competitor-parity-matrix.md`
117. `microservices/connector/compliance.md`
118. `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`
119. `microservices/connector/contracts/connect-retirement.asyncapi.yaml`
120. `microservices/connector/contracts/connect-retirement.openapi.yaml`
121. `microservices/connector/contracts/connect_retirement.proto`
122. `microservices/connector/contracts/connector-adapter-trait.md`
123. `microservices/connector/contracts/metric-naming-convention.md`
124. `microservices/connector/contracts/openapi/connector-integration.yaml`
125. `microservices/connector/contracts/proto/connector_integration.proto`
126. `microservices/connector/cost-budget.md`
127. `microservices/connector/dashboards/connector-usage-by-tenant.json`
128. `microservices/connector/dashboards/dlq-state.json`
129. `microservices/connector/dashboards/oauth-token-health.md`
130. `microservices/connector/dashboards/webhook-receiver-throughput.json`
131. `microservices/connector/decisions/ADR-MS-001-connector-broker-webhook-and-dlq-contract.md`
132. `microservices/connector/dpia.md`
133. `microservices/connector/failure-modes.md`
134. `microservices/connector/faqs/federation-engineer-faq.md`
135. `microservices/connector/iac/ech-config.yaml`
136. `microservices/connector/iac/edge-waf.yaml`
137. `microservices/connector/iac/external-secret.yaml`
138. `microservices/connector/iac/helm-values-connector.yaml`
139. `microservices/connector/iac/ingress-production.yaml`
140. `microservices/connector/iac/kustomize-base.yaml`
141. `microservices/connector/iac/network-policy.yaml`
142. `microservices/connector/iac/openbao-policy.hcl`
143. `microservices/connector/iac/postgres-migration-001.sql`
144. `microservices/connector/iac/pqc-cert.yaml`
145. `microservices/connector/iac/spiffe-workload-identity.yaml`
146. `microservices/connector/incident-response.md`
147. `microservices/connector/manifest.json`
148. `microservices/connector/migration-playbooks/from-slack-connect-and-teams-external.md`
149. `microservices/connector/multi-region.md`
150. `microservices/connector/onboarding/federation-engineer-first-week.md`
151. `microservices/connector/operational-boundaries.md`
152. `microservices/connector/policy/abuse-defence.cedar`
153. `microservices/connector/policy/auditor-scope.cedar`
154. `microservices/connector/policy/ci-scope.cedar`
155. `microservices/connector/policy/connector-authorization.cedar`
156. `microservices/connector/policy/connector-catalog-publishing.cedar`
157. `microservices/connector/policy/data-residency.md`
158. `microservices/connector/policy/no-new-runtime-scope.cedar`
159. `microservices/connector/policy/oauth-broker-authorization.cedar`
160. `microservices/connector/policy/payload-signature-verification.cedar`
161. `microservices/connector/policy/tenant-isolation.md`
162. `microservices/connector/policy/webhook-receiver-gating.cedar`
163. `microservices/connector/reference-implementations/cross-tenant-message-rust-sdk.md`
164. `microservices/connector/runbooks/connector-attestation-revoked.md`
165. `microservices/connector/runbooks/connector-cascade-failure.md`
166. `microservices/connector/runbooks/connector-onboarding.md`
167. `microservices/connector/runbooks/connector-rate-limit-saturation.md`
168. `microservices/connector/runbooks/dlq-overflow.md`
169. `microservices/connector/runbooks/oauth-token-revocation-cascade.md`
170. `microservices/connector/runbooks/pii-leak-via-connector.md`
171. `microservices/connector/runbooks/retirement-status-drift.md`
172. `microservices/connector/runbooks/signature-verification-cascade-failure.md`
173. `microservices/connector/runbooks/webhook-replay-attack-detected.md`
174. `microservices/connector/scorecards/overrides.json`
175. `microservices/connector/sdk-plan.md`
176. `microservices/connector/slos/connect-retirement.openslo.yaml`
177. `microservices/connector/slos/connector-availability.openslo.yaml`
178. `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml`
179. `microservices/connector/slos/oauth-token-health.openslo.yaml`
180. `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml`
181. `microservices/connector/threat-model.md`
182. `microservices/connector/tutorials/establish-cross-tenant-channel-with-mls-and-cedar.md`

### §2.2 Artifact Families Read

1. Product definition: `PRD.md`, `README.md`, `ARCHITECTURE.md`, `PHASE-01-INTEGRATION-SUBSTRATE-FOUNDATION.md`.
2. Governance definition: `manifest.json`, `AUDIT-FINDINGS-2026-05-18.json`, `AUDIT-FINDINGS-2026-05-20.json`, `CHANGELOG.md`, `RETIREMENT-PLAN.md`.
3. Architecture decision coverage: `decisions/ADR-MS-001-connector-broker-webhook-and-dlq-contract.md`.
4. Implementation plans: all `IP-*.md` and all `IP-journey-*.md` files in the service root.
5. Contracts: OpenAPI, AsyncAPI, proto, adapter-trait, retirement contracts, and metric naming convention.
6. SLOs: all five OpenSLO files under `slos/`.
7. Operational docs: capacity, failure modes, incident response, cost budget, DPIA, compliance, backfill replay, multi-region, operational boundaries, threat model, SDK plan.
8. Benchmarks: `benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md`.
9. Parity: `competitor-parity-matrix.md`.
10. Capability model: all `capabilities/*.yaml` and `capability-tiers/tier-matrix.md`.
11. Catalog: connector catalog records and bounded-context catalog records.
12. IaC: all files under flat `iac/`.
13. Policies: all Cedar and policy markdown files.
14. Dashboards: all dashboard JSON and markdown files.
15. Runbooks: all runbooks under `runbooks/`.
16. User-facing docs: FAQ, onboarding, migration playbook, reference implementation, tutorial.
17. Representative code sample classification: the path contains no Rust source tree, so sampling shifted to contracts, policies, catalog YAML, and IaC-like files.
18. Read-depth note: the top-level documents and required domain docs were read to substantive line counts; smaller files were read in full through search and line-specific inspection.

## §3 Nine-Dimension Audit

### §3.1 Dimension 1 - Product Purpose And Ownership

1. Status: mixed, with an active product core and conflicting retirement-era metadata.
2. The active purpose is integration substrate, cited by `microservices/connector/PRD.md:29-39`.
3. The active purpose includes connector catalog, OAuth broker, webhook receiver, and retry/DLQ substrate, cited by `microservices/connector/PRD.md:128-137`.
4. The README confirms that `connector` is not workflow-engine, api-gateway, or a credential store, cited by `microservices/connector/README.md:28-32`.
5. The architecture roster defines bounded contexts for connector-catalog, oauth-broker, webhook-receiver, signature-verification, payload-canonicalization, connector-adapter, data-mapping, and retry-and-DLQ, cited by `microservices/connector/ARCHITECTURE.md:26-38`.
6. Chat history confirms the same ownership frame with a 500-plus connector target, cited by `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:6706-6709`.
7. Chat history later restricts the boundary away from iPaaS, workflow triggers, and data pipelines, cited by `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8923-8926`.
8. Conflict: `microservices/connector/manifest.json:3-6` says the service purpose is a retiring umbrella coordination surface.
9. Conflict: `microservices/connector/manifest.json:21-24` points to retirement contracts rather than integration contracts.
10. Conflict: `microservices/connector/RETIREMENT-PLAN.md` exists as a top-level service artifact while the PRD and architecture define active substrate work.
11. Conflict: `microservices/connector/operational-boundaries.md:1-5` still frames the operational boundary as Retirement.
12. Risk: future agents may implement retirement verification instead of connector substrate delivery.
13. Risk: control surfaces may index the retired manifest purpose while humans read the active PRD.
14. Risk: feature parity work may follow stale federation or iPaaS surfaces rather than the Twilio, Sendbird, and Stream union requested for this audit.
15. Coherence call: the product core is coherent when PRD, README, architecture, contracts, SLOs, capacity, and chat are read together.
16. Coherence call: the manifest and retirement artifacts must be reconciled before connect can be treated as ready for implementation sequencing.
17. Severity: P1 for the manifest-purpose conflict because the manifest is a machine-readable control surface.

### §3.2 Dimension 2 - Artifact Completeness And Depth

1. Status: broad artifact coverage with several critical control-surface gaps.
2. Core docs are present: PRD, README, architecture, capacity model, failure modes, incident response, cost budget, DPIA, compliance, threat model, and SDK plan.
3. The PRD has specific performance and scale requirements, including connector action overhead and webhook ack thresholds, cited by `microservices/connector/PRD.md:143-158`.
4. The architecture has bounded contexts and verification hooks, cited by `microservices/connector/ARCHITECTURE.md:26-38` and `microservices/connector/ARCHITECTURE.md:996-1001`.
5. The compliance map has day-one certification readiness posture, cited by `microservices/connector/compliance.md:21-33`.
6. The failure-mode inventory names specific failure classes across connector-catalog, OAuth, webhook, signature verification, adapters, data mapping, retry, and DLQ, cited by `microservices/connector/failure-modes.md:15-84`.
7. The incident response doc includes severity triggers and playbooks, cited by `microservices/connector/incident-response.md:20-68`.
8. The cost budget includes a per-call cost model and annual run-rate estimate, cited by `microservices/connector/cost-budget.md:13-34`.
9. The DPIA includes purpose, lawful basis, risk table, and mitigation lines, cited by `microservices/connector/dpia.md:20-69`.
10. The OpenAPI contract covers catalog lookup, OAuth grant initiation, callback, grant status/deletion, and webhook endpoint registration/rotation, cited by `microservices/connector/contracts/openapi/connector-integration.yaml:43-218`.
11. The AsyncAPI contract covers connector, OAuth, webhook, and DLQ event channels, cited by `microservices/connector/contracts/asyncapi/connector-integration-events.yaml:24-188`.
12. The proto contract covers catalog, OAuth, action, webhook endpoint, and DLQ services, cited by `microservices/connector/contracts/proto/connector_integration.proto:19-178`.
13. Missing artifact: no `cross-microservice-handoffs.md` exists, despite manifest dependencies on api-gateway, workflow-engine, marketplace, cloud-secrets, policy-engine, observability, billing-ledger, and others in `microservices/connector/manifest.json:108-128`.
14. Missing artifact: no service-local `supported-oses.json` exists, despite ADR-0328 D-17 requiring a manifest-shaped OS declaration.
15. Missing artifact: no canonical six-context IaC directories exist under `microservices/connector/iac/`.
16. Missing artifact: no `src/` and no `tests/` tree exists, so implementation and regression evidence cannot be sampled.
17. Existing audit evidence says content depth was only partial after prior passes, cited by `microservices/connector/AUDIT-FINDINGS-2026-05-20.json:50-54`.
18. Existing audit evidence includes an unresolved content-pass marker at `microservices/connector/AUDIT-FINDINGS-2026-05-20.json:48`.
19. Coherence call: documentation breadth is high, but control-surface readiness is not yet coherent enough for deployment claims.
20. Severity: P2 for missing handoff, OS manifest, and implementation evidence; P1 for missing canonical deployment-context IaC.

### §3.3 Dimension 3 - Product Surface Versus Counterparts

1. Status: current internal counterpart set is stale for this batch.
2. The README lists Zapier, n8n, Workato, Boomi, MuleSoft, Tray, Pipedream, and EventBridge as precedents, cited by `microservices/connector/README.md:34-36`.
3. The competitor parity matrix uses the same iPaaS/integration-platform set, cited by `microservices/connector/competitor-parity-matrix.md:13-36`.
4. The existing benchmark compares Slack Connect, Teams External Access, Matrix federation, and Discord guest threads, cited by `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:9-13`.
5. This batch instead requires Twilio, Sendbird, and Stream as union-coverage counterparts.
6. Twilio overlap is strongest for Conversations-like participants, messages, webhooks, phone/SMS/email integration, rate limits, and delivery receipts.
7. Sendbird overlap is strongest for chat API rate limits, channel/member management, moderation, webhooks, push, announcements, and supergroup/open-channel scaling.
8. Stream overlap is strongest for chat connect rates, channel/member limits, webhooks, moderation, attachments, push, unread counts, and real-time client events.
9. Connect’s active purpose is not only chat; it is a connector substrate that can include Twilio and Sendbird/Stream adapters plus webhook and OAuth fabric.
10. Therefore the union bar should be interpreted as communication-integration substrate coverage, not as a command to turn connect into a chat product.
11. Existing `catalog/connectors/twilio.yaml` is positive evidence that Twilio can be modeled as a connector, cited by `microservices/connector/catalog/connectors/twilio.yaml`.
12. Existing `catalog/connectors/discord.yaml` and `catalog/connectors/slack.yaml` are positive evidence that communication apps already sit in connector catalog scope, cited by `microservices/connector/catalog/connectors/discord.yaml` and `microservices/connector/catalog/connectors/slack.yaml`.
13. Existing `catalog/connectors/sendgrid.yaml` and `catalog/connectors/mailgun.yaml` are positive evidence for messaging-adjacent delivery providers, cited by `microservices/connector/catalog/connectors/sendgrid.yaml` and `microservices/connector/catalog/connectors/mailgun.yaml`.
14. Missing: no Sendbird connector catalog record was found.
15. Missing: no Stream connector catalog record was found.
16. Missing: current feature matrix does not evaluate Twilio, Sendbird, or Stream.
17. Missing: current benchmark document is tied to cross-tenant chat federation and old capability tiers.
18. Coherence call: connector can own the union coverage if framed as connector/webhook/OAuth/event substrate, but current parity docs do not yet express that frame.
19. Severity: P2 for stale counterpart artifacts.

### §3.4 Dimension 4 - Canonical-Direction Alignment

1. Status: materially misaligned on deployment context, IaC substrate, tenant-class adoption, and retired capability-tier vocabulary.
2. ADR-0328 D-15 requires all microservice briefs to declare the six deployment contexts unless scoped out with evidence, cited by `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2240`.
3. `specs/master-plan-sequencing.json:704-746` names the six deployment context ids and IaC targets.
4. `microservices/connector/manifest.json` does not declare those six context ids.
5. `microservices/connector/PRD.md` does not declare a six-context deployment support matrix.
6. `microservices/connector/compliance.md:1018` mentions on-prem facility attestation, but that is not a six-context support matrix.
7. ADR-0328 D-16 requires OpenTofu modules and forbids Terraform, Pulumi, CloudFormation, ARM, and hand-run scripts as canonical deployment mechanisms, cited by `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2644`.
8. `microservices/connector/README.md:50` still describes `iac/` as Helm, Terraform, and Kustomize.
9. `microservices/connector/CHANGELOG.md:24` still refers to a Helm chart and Terraform module.
10. `microservices/connector/PHASE-01-INTEGRATION-SUBSTRATE-FOUNDATION.md:30` still refers to a Helm chart and Terraform module.
11. The flat `iac/` folder contains YAML, HCL policy, and SQL files, but not OpenTofu context modules.
12. ADR-0328 D-17 requires service OS matrices, and no service-local `supported-oses.json` exists.
13. ADR-0328 D-18 requires Rust strict backend/tooling policy; no forbidden source language files were found in the path.
14. Rust-strict implementation evidence remains incomplete because no `src/`, `tests/`, or service-local Cargo manifest was found.
15. ADR-0328 D-19 requires OCI Always Free profile treatment with `iac/oci-guest/always-free/`; that folder is absent.
16. The no-capability-tiers memory retires old capability tiers and old OCI naming, cited by `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tracks_2026_05_20.md:10-45`.
17. This audit uses `OCI Always Free profile` and `demo_trial tenant_class infrastructure` wording, not the retired old OCI tier label.

#### §3.4.C - Tenant-Class Adoption Gaps

1. Required tenant_class labels for this batch: `demo_trial`, `paid`, and `revenue_share`.
2. Exact search for `tenant_class` under `microservices/connector/` found no hits.
3. Exact search for `demo_trial` under `microservices/connector/` found no hits.
4. Exact search for `revenue_share` under `microservices/connector/` found no hits.
5. Search for `paid` found product and tier-adaptive fragments, not a tenant_class enum.
6. `microservices/connector/PRD.md:47` mentions a marketplace publisher listing a connector adapter as a paid marketplace item.
7. `microservices/connector/ARCHITECTURE.md:715` mentions a paid API tier for legitimate bulk consumers.
8. `microservices/connector/ARCHITECTURE.md:723` says tenant-tier-adaptive sensitivity lowers sensitivity for paid tiers.
9. `microservices/connector/policy/abuse-defence.cedar:80` repeats tenant-tier-adaptive paid-tier language.
10. No artifact binds `demo_trial` to OCI Always Free profile caps.
11. No artifact binds `paid` to per-seat plus usage-based billing and contractual SLO eligibility.
12. No artifact binds `revenue_share` to at-cost or zero-margin substrate controls.
13. No manifest enum, contract field, policy field, billing handoff, SLO overlay, rate-limit profile, or capacity overlay currently expresses those tenant classes.
14. Tenant-class adoption gap: yes.
15. Default severity: P2, because the absence creates policy, billing, SLO, and infrastructure ambiguity but does not by itself prove runtime failure.

#### §3.4.T - Tier Retirement Candidates

1. `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:13` - paid reference, Wave 15J retirement candidate, severity P2.
2. `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:21` - paid reference, Wave 15J retirement candidate, severity P2.
3. `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:28` - paid reference, Wave 15J retirement candidate, severity P2.
4. `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:48` - paid reference, Wave 15J retirement candidate, severity P2.
5. `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:83` - paid reference, Wave 15J retirement candidate, severity P2.
6. `microservices/connector/capability-tiers/tier-matrix.md:13` - demo_trial reference, Wave 15J retirement candidate, severity P2.
7. `microservices/connector/capability-tiers/tier-matrix.md:21` - demo_trial reference, Wave 15J retirement candidate, severity P2.
8. `microservices/connector/capability-tiers/tier-matrix.md:48` - paid reference, Wave 15J retirement candidate, severity P2.
9. `microservices/connector/capability-tiers/tier-matrix.md:50` - demo_trial reference, Wave 15J retirement candidate, severity P2.
10. `microservices/connector/capability-tiers/tier-matrix.md:79` - paid reference, Wave 15J retirement candidate, severity P2.
11. `microservices/connector/capability-tiers/tier-matrix.md:81` - paid reference, Wave 15J retirement candidate, severity P2.
12. `microservices/connector/capability-tiers/tier-matrix.md:109` - paid reference, Wave 15J retirement candidate, severity P2.
13. `microservices/connector/capability-tiers/tier-matrix.md:113` - compliance_pack-bound paid reference, Wave 15J retirement candidate, severity P2.
14. `microservices/connector/capability-tiers/tier-matrix.md:115` - paid reference, Wave 15J retirement candidate, severity P2.
15. `microservices/connector/capability-tiers/tier-matrix.md:127` - paid reference, Wave 15J retirement candidate, severity P2.
16. `microservices/connector/capability-tiers/tier-matrix.md:129` - paid reference, Wave 15J retirement candidate, severity P2.
17. `microservices/connector/capability-tiers/tier-matrix.md:142` - demo_trial, paid, paid, and compliance_pack-bound paid references, Wave 15J retirement candidate, severity P2.
18. `microservices/connector/faqs/federation-engineer-faq.md:74` - paid reference, Wave 15J retirement candidate, severity P2.
19. `microservices/connector/faqs/federation-engineer-faq.md:76` - compliance_pack-bound paid reference, Wave 15J retirement candidate, severity P2.
20. `microservices/connector/tutorials/establish-cross-tenant-channel-with-mls-and-cedar.md:15` - paid reference, Wave 15J retirement candidate, severity P2.
21. `microservices/connector/tutorials/establish-cross-tenant-channel-with-mls-and-cedar.md:176` - compliance_pack-bound paid reference, Wave 15J retirement candidate, severity P2.
22. Additional tier-vocabulary residue exists in `microservices/connector/manifest.json:57-60`, `microservices/connector/manifest.json:89`, `microservices/connector/ARCHITECTURE.md:715`, `microservices/connector/ARCHITECTURE.md:723`, and `microservices/connector/policy/abuse-defence.cedar:80`.
23. The exact named capability-tier candidate count for this audit is 21.
24. The broader vocabulary-residue count is not included in the 21 exact candidate count but should be included in Wave 15J cleanup.

### §3.5 Dimension 5 - Contracts, APIs, Events, And SLOs

1. Status: strong integration-surface contract draft, weak manifest binding.
2. OpenAPI contract is integration-focused and cites REST substrate scope, `microservices/connector/contracts/openapi/connector-integration.yaml:1-16`.
3. OpenAPI covers catalog search/list/read, `microservices/connector/contracts/openapi/connector-integration.yaml:43-96`.
4. OpenAPI covers OAuth grant initiation and callback, `microservices/connector/contracts/openapi/connector-integration.yaml:97-147`.
5. OpenAPI covers OAuth grant read and deletion, `microservices/connector/contracts/openapi/connector-integration.yaml:148-175`.
6. OpenAPI covers webhook endpoint registration and rotation, `microservices/connector/contracts/openapi/connector-integration.yaml:176-218`.
7. AsyncAPI covers connector action events, OAuth grant events, webhook received events, and DLQ item events, `microservices/connector/contracts/asyncapi/connector-integration-events.yaml:24-188`.
8. Proto covers catalog service, OAuth grant service, connector action service, webhook endpoint service, and DLQ service, `microservices/connector/contracts/proto/connector_integration.proto:19-178`.
9. SLO set includes connector availability target 0.999, cited by `microservices/connector/slos/connector-availability.openslo.yaml:4-38`.
10. SLO set includes webhook receiver throughput target 0.995, cited by `microservices/connector/slos/webhook-receiver-throughput.openslo.yaml:4-39`.
11. SLO set includes OAuth token health target 0.995, cited by `microservices/connector/slos/oauth-token-health.openslo.yaml:4-38`.
12. SLO set includes DLQ overflow prevention target 0.99, cited by `microservices/connector/slos/dlq-overflow-prevention.openslo.yaml:4-38`.
13. SLO set also includes retirement SLO, cited by `microservices/connector/slos/connect-retirement.openslo.yaml:4-8`.
14. Contract gap: manifest references retirement contracts, not the integration OpenAPI, AsyncAPI, or proto, cited by `microservices/connector/manifest.json:21-24`.
15. Contract gap: no manifest-level `deployment_contexts`, `supported_oses`, or tenant-class capability overlays tie contracts to runtime contexts.
16. Contract gap: no counterpart-specific connector capability schema for Twilio, Sendbird, and Stream is declared in the active contracts.
17. Coherence call: the human-readable contract set is mostly aligned to integration substrate, but the machine-readable manifest is not aligned.
18. Severity: P2 for contract index split; P1 only if the manifest is used as the admission control source.

### §3.6 Dimension 6 - Deployment Contexts And Operational Readiness

1. Status: not ready for all-six deployment claims.
2. Canonical six context ids are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`, cited by `specs/master-plan-sequencing.json:704-746`.
3. ADR-0328 D-15 says default Phase 0 and Phase 1 expectation is all six contexts, cited by `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2116-2119`.
4. No service-local `iac/oyatie-public-cloud/` directory exists.
5. No service-local `iac/guest-on-aws/` directory exists.
6. No service-local `iac/oci-guest/` directory exists.
7. No service-local `iac/on-prem/` directory exists.
8. No service-local `iac/colo/` directory exists.
9. No service-local `iac/oyatie-iaas/` directory exists.
10. No service-local `iac/oci-guest/always-free/` directory exists.
11. Flat `iac/` files exist, including `helm-values-connector.yaml`, `kustomize-base.yaml`, `ingress-production.yaml`, `external-secret.yaml`, and `openbao-policy.hcl`.
12. Those files are useful substrate fragments but do not satisfy the canonical OpenTofu context-module requirement.
13. Compliance has an on-prem facility attestation section, cited by `microservices/connector/compliance.md:1018`.
14. Multi-region has pack region map and cross-pack movement constraints, cited by `microservices/connector/multi-region.md:15-48`.
15. Capacity model has webhook receiver scaling and worker counts, cited by `microservices/connector/capacity-model.md:25-43`.
16. Incident response includes Sev-1 and secret-leak playbooks, cited by `microservices/connector/incident-response.md:36-68`.
17. Operational readiness is stronger at runbook and capacity-story level than at deployable substrate level.
18. Coherence call: deployment-context coverage should be treated as failed until each context has an OpenTofu module or an explicit, cited non-applicability proof.
19. Severity: P1 for all-six context and OCI Always Free profile absence.

### §3.7 Dimension 7 - Security, Privacy, Compliance, And Abuse Resistance

1. Status: security and compliance documentation is unusually broad, with tier-language cleanup required.
2. PRD security requires per-tenant OAuth encryption, no secret logging, signed webhooks, replay windows, and policy-gated connector actions, cited by `microservices/connector/PRD.md:184-188`.
3. PRD data residency requires tenant-region pinning and no cross-region connector payload storage unless policy allows it, cited by `microservices/connector/PRD.md:194-196`.
4. Threat model identifies trust boundaries across external connector APIs, webhook senders, tenant operators, marketplace publishers, cloud-secrets, policy-engine, and workflow-engine, cited by `microservices/connector/threat-model.md:20-32`.
5. Threat model uses STRIDE across bounded contexts, cited by `microservices/connector/threat-model.md:34-98`.
6. DPIA identifies connector payload sensitivity, OAuth token risk, webhook replay risk, and connector metadata leakage, cited by `microservices/connector/dpia.md:47-58`.
7. DPIA mitigations include OpenBao-managed tokens, signed webhook replay windows, tenant-scoped catalog visibility, DLQ redaction, and region policy checks, cited by `microservices/connector/dpia.md:60-69`.
8. Compliance defines pack readiness for SOC 2, ISO 27001, HIPAA, GDPR, FedRAMP, PCI, and regional overlays, cited by `microservices/connector/compliance.md:21-89`.
9. Abuse defense policy has explicit anti-scrape controls in architecture, cited by `microservices/connector/ARCHITECTURE.md:711-725`.
10. Abuse defense policy still uses tier-adaptive paid-tier language, cited by `microservices/connector/policy/abuse-defence.cedar:80`.
11. Architecture repeats the same paid-tier language, cited by `microservices/connector/ARCHITECTURE.md:715` and `microservices/connector/ARCHITECTURE.md:723`.
12. Security docs are not yet connected to tenant_class labels required by this batch.
13. No BYOK or compliance-pack eligibility overlay by tenant_class was found.
14. No demo-trial best-effort SLO security posture was found.
15. No revenue-share at-cost substrate abuse-budget model was found.
16. Coherence call: security depth is good, but pricing and tenant-class semantics still use retired vocabulary.
17. Severity: P2 for tier-language cleanup and tenant-class security overlay gap.

### §3.8 Dimension 8 - Implementation, Tests, And Language Policy

1. Status: language scan is clean, but implementation proof is absent from this path.
2. ADR-0328 D-18 requires Rust for backend, runtime, CLI, validation, codegen, scripts, CI harnesses, migrations, and operational automation, cited by `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3045-3489`.
3. Frontend exceptions are Swift, Kotlin, WinUI 3, and Leptos/WASM SSR with selective island hydration, cited by `docs/standards/brief-template.md:1125-1374`.
4. Forbidden file-extension scan found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files under `microservices/connector/`.
5. Proto contains `go_package` and `java_package` options at `microservices/connector/contracts/proto/connector_integration.proto:9-10`; those are schema generator metadata, not service implementation source files.
6. No `microservices/connector/src/` directory exists.
7. No `microservices/connector/tests/` directory exists.
8. No service-local Cargo manifest was found in the inventory.
9. No Rust module path proves connector catalog, OAuth broker, webhook receiver, adapter trait, data mapping, or retry/DLQ implementation.
10. No regression tests prove the PRD performance, scale, authorization, replay, or DLQ behavior.
11. No IaC tests prove context modules because context modules are absent.
12. No OS test matrix can be sampled because `supported-oses.json` is absent.
13. Existing implementation plans are useful, but they are not executable proof.
14. Coherence call: the service passes the strict-language absence check and fails the implementation-evidence check.
15. Severity: P2 until code and tests exist or the service is explicitly scoped as documentation-only for this phase.

### §3.9 Dimension 9 - Lifecycle, Sequencing, And Verification Readiness

1. Status: sequencing evidence exists, but lifecycle state conflicts remain.
2. PRD status is accepted, cited by `microservices/connector/PRD.md:6`.
3. Architecture status is accepted, cited by `microservices/connector/ARCHITECTURE.md:1-18`.
4. ADR-MS-001 status is proposed, cited by `microservices/connector/decisions/ADR-MS-001-connector-broker-webhook-and-dlq-contract.md:4`.
5. Manifest status is retiring, cited by `microservices/connector/manifest.json:3-6`.
6. Retirement contracts exist alongside integration contracts, cited by `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, and `microservices/connector/contracts/connect_retirement.proto`.
7. Integration contracts exist and are more aligned with the active PRD, cited by `microservices/connector/contracts/openapi/connector-integration.yaml`, `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, and `microservices/connector/contracts/proto/connector_integration.proto`.
8. Existing audit file says connector catalog seed was materially incomplete, cited by `microservices/connector/AUDIT-FINDINGS-2026-05-20.json:25-30`.
9. Chat history repeats that 470-plus connector catalog entries remained queued, cited by `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:7377-7385`.
10. Chat history later says the 470-plus connector catalog seed entries stay in connect while downstream service actions stay in destination services, cited by `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8923-8926`.
11. PRD requires at least 500 connectors in the catalog, cited by `microservices/connector/PRD.md:156`.
12. Current connector catalog file count is 36 connector YAML records, including Twilio but not Sendbird or Stream.
13. Lifecycle risk: accepted PRD plus retiring manifest plus proposed ADR creates admission ambiguity.
14. Lifecycle risk: retirement-era benchmark and capability documents can pollute future target setting.
15. Verification readiness risk: SLOs exist, but there is no test harness and no deployment-context IaC to run against.
16. Coherence call: connector is ready for targeted cleanup and implementation planning, not ready for deployment-readiness claims.
17. Severity: P1 for manifest lifecycle conflict; P2 for status mismatch and P3 for connector catalog count as a known large backlog.

## §4 Findings Table

| ID | Severity | Finding | Evidence | Required correction |
| --- | --- | --- | --- | --- |
| F-01 | P1 | Machine-readable manifest says connect is retiring while PRD and README define an active integration substrate. | `microservices/connector/manifest.json:3-6`; `microservices/connector/PRD.md:29-39`; `microservices/connector/README.md:16-32`; chat `8f603fc7...jsonl:6706-6709` | Align manifest purpose/status/contracts with active integration-substrate scope or explicitly split retirement lineage from active service. |
| F-02 | P1 | Six canonical deployment contexts are not expressed as service-local deployment support or context modules. | `specs/master-plan-sequencing.json:704-746`; absent `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, `iac/oyatie-iaas/` | Add six-context declaration and OpenTofu context modules, or add cited non-applicability decisions per context. |
| F-03 | P1 | IaC documentation still points to Terraform/Helm/Kustomize instead of OpenTofu-only canonical modules. | `microservices/connector/README.md:50`; `microservices/connector/CHANGELOG.md:24`; `microservices/connector/PHASE-01-INTEGRATION-SUBSTRATE-FOUNDATION.md:30`; ADR-0328 D-16 | Replace deprecated deployment wording with OpenTofu module structure and remove Terraform as a canonical substrate. |
| F-04 | P1 | OCI Always Free profile is absent. | absent `microservices/connector/iac/oci-guest/always-free/`; `specs/master-plan-sequencing.json:857-868`; ADR-0328 D-19 | Add demo_trial tenant_class infrastructure profile for OCI Always Free or cite why connect is excluded. |
| F-05 | P2 | Service-local OS support manifest is absent. | absent `microservices/connector/supported-oses.json`; ADR-0328 D-17; `docs/standards/brief-template.md:967-1124` | Add supported OS and architecture matrix with Tier-1, test-only, and out-of-scope declarations. |
| F-06 | P2 | Tenant-class semantics are absent. | no `tenant_class`, no `demo_trial`, no `revenue_share`; `microservices/connector/ARCHITECTURE.md:715`; `microservices/connector/policy/abuse-defence.cedar:80` | Adopt `demo_trial`, `paid`, and `revenue_share` in manifest, capacity, SLO, policy, and billing handoffs. |
| F-07 | P2 | Exact retired capability-tier references remain in 21 locations. | §3.4.T list; no-capability-tiers memory `feedback_no_capability_tracks_2026_05_20.md:10-45` | Retire or rewrite old capability-tier documents during Wave 15J; do not use them for new implementation targets. |
| F-08 | P2 | Existing benchmark uses stale chat-federation counterparts and retired capability-tier rows. | `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:9-13`; `:21`; `:48`; `:83` | Replace benchmark framing with Twilio, Sendbird, Stream, and one industry-leader target set. |
| F-09 | P2 | Capability-tier matrix describes cross-tenant federation rather than connector substrate ownership. | `microservices/connector/capability-tiers/tier-matrix.md:11-21`; `microservices/connector/PRD.md:29-39` | Treat the file as a Wave 15J retirement candidate, not an active connect source of truth. |
| F-10 | P2 | Cross-microservice handoff document is missing despite many service dependencies. | absent `microservices/connector/cross-microservice-handoffs.md`; `microservices/connector/manifest.json:108-128` | Add explicit handoffs to api-gateway, workflow-engine, marketplace, cloud-secrets, policy-engine, observability, billing, and downstream services. |
| F-11 | P2 | Lifecycle status is split between accepted PRD/architecture and proposed ADR. | `microservices/connector/PRD.md:6`; `microservices/connector/ARCHITECTURE.md:1-18`; `microservices/connector/decisions/ADR-MS-001-connector-broker-webhook-and-dlq-contract.md:4` | Decide whether ADR-MS-001 is adopted, superseded, or blocked, and align status. |
| F-12 | P2 | Manifest contract index points to retirement contracts, not integration contracts. | `microservices/connector/manifest.json:21-24`; integration contracts under `contracts/openapi`, `contracts/asyncapi`, and `contracts/proto` | Rebind manifest contract fields to active integration contracts and keep retirement contracts as historical if needed. |
| F-13 | P2 | Existing audit metadata still contains an unresolved content-pass marker and partial-depth note. | `microservices/connector/AUDIT-FINDINGS-2026-05-20.json:48-54` | Replace unresolved marker with final evidence or retire the audit file from active readiness decisions. |
| F-14 | P2 | Rust-strict source scan is clean, but there is no implementation or regression-test evidence. | no `src/`; no `tests/`; ADR-0328 D-18 | Add Rust implementation and Rust tests, or mark service as documentation-only for the current phase. |
| F-15 | P2 | Abuse and anti-scrape posture still uses paid-tier vocabulary instead of tenant_class policy. | `microservices/connector/ARCHITECTURE.md:715`; `microservices/connector/ARCHITECTURE.md:723`; `microservices/connector/policy/abuse-defence.cedar:80` | Rewrite policy posture around tenant classes and objective abuse-risk signals. |
| F-16 | P2 | Existing parity docs frame connect as generic iPaaS while current chat says not iPaaS. | `microservices/connector/competitor-parity-matrix.md:13-36`; chat `8f603fc7...jsonl:8923-8926` | Reframe parity around connector substrate, communications integration, webhook/OAuth fabric, and bounded downstream handoffs. |
| F-17 | P3 | Connector catalog seed is far below the PRD target. | `microservices/connector/PRD.md:156`; `microservices/connector/AUDIT-FINDINGS-2026-05-20.json:25-30`; chat `8f603fc7...jsonl:7377-7385` | Continue catalog seeding with ownership boundaries preserved. |
| F-18 | P3 | Capacity model includes a corrected-formula note that should be simplified in the final version. | `microservices/connector/capacity-model.md:48` | Rewrite formula section so only the accepted calculation path remains. |
| F-19 | P3 | Generated expansion fragments and retirement-era text reduce review confidence despite high line count. | `microservices/connector/ARCHITECTURE.md:53-94`; `microservices/connector/compliance.md:35-72`; docs-substance memory `feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` | Replace generated expansion residue with service-specific substance or move lineage notes to a historical appendix. |

## §5 Open Questions

1. Should `manifest.json` be the canonical machine-readable source for connect, or should a new service-control artifact supersede it for active integration-substrate scope?
2. Should retirement contracts remain in the service path as historical lineage, or should they move under an archive namespace so active contract discovery does not select them?
3. Should the Twilio, Sendbird, and Stream counterpart bar be satisfied by first-party connector records, by parity against communications platform semantics, or by both?
4. Should Sendbird and Stream be explicit catalog seed records before the next readiness gate?
5. Should `connector` own adapter marketplace publisher flows directly, or should marketplace own commercial listing and connect own only adapter certification and invocation substrate?
6. Which service owns tenant_class assignment and billing components for `demo_trial`, `paid`, and `revenue_share`: connector, billing-ledger, cloud-billing, account-auth-gateway, or a shared policy service?
7. Should `demo_trial` caps be expressed in connect rate-limit policy, OpenTofu capacity overlays, billing policy, or all three?
8. Should on-prem and colo support include a minimal connector set where external SaaS egress is not available, or should all connectors remain declarative but disabled by tenant policy?
9. Should webhook receiver SLOs differ by deployment context, or should the canonical SLO stay uniform with explicit infrastructure caps for constrained profiles?
10. Should the Rust implementation begin with connector-catalog and OAuth broker before webhook receiver, or should webhook receiver come first because it is the highest external attack surface?
11. Should `cross-microservice-handoffs.md` be required before any implementation plan is promoted, given the number of dependencies in `manifest.json:108-128`?
12. Should `supported-oses.json` be generated from the master-plan OS matrix or curated manually per microservice?
13. Should flat Kubernetes YAML under `iac/` be retained as OpenTofu-rendered artifact examples or removed from canonical deployment source paths?
14. Should the old cross-tenant federation tutorial be moved to another service if that behavior belongs outside connector substrate?
15. Should the reference implementation named `cross-tenant-message-rust-sdk.md` be re-scoped or retired because it appears unrelated to connector integration substrate?
16. Should the current `connector-adapter-trait.md` become the first implementation contract for Rust source generation?
17. Should the SLO set include explicit Twilio, Sendbird, and Stream connector-provider degradation objectives?
18. Should the capacity model distinguish provider-side rate limits from Oyatie-side rate limits for all communications connectors?
19. Should the abuse policy treat anonymous reads, demo trials, paid tenants, and revenue-share tenants as billing contexts only, with security sensitivity driven by risk signals?
20. Should the next batch require evidence of an executable Rust smoke test before treating the documentation set as implementation-ready?

### §5.1 Evidence-Backed Remediation Sequence

1. Remediation R-01 should start with the manifest purpose/status conflict because `microservices/connector/manifest.json:3-6` is machine-readable and conflicts with `microservices/connector/PRD.md:29-39`.
2. R-01 acceptance evidence should be a manifest purpose aligned to integration substrate, active contracts, dependencies, deployment contexts, OS manifest path, and tenant_class fields.
3. R-01 should preserve retirement lineage only as historical metadata, because retirement contracts still exist at `microservices/connector/contracts/connect-retirement.openapi.yaml`, `microservices/connector/contracts/connect-retirement.asyncapi.yaml`, and `microservices/connector/contracts/connect_retirement.proto`.
4. Remediation R-02 should rebind manifest contracts to `microservices/connector/contracts/openapi/connector-integration.yaml`, `microservices/connector/contracts/asyncapi/connector-integration-events.yaml`, and `microservices/connector/contracts/proto/connector_integration.proto`.
5. R-02 acceptance evidence should include an explicit active-contract index and a historical-contract index if retirement artifacts remain.
6. Remediation R-03 should add `cross-microservice-handoffs.md` because `microservices/connector/manifest.json:108-128` lists many dependencies without a service-local handoff contract.
7. R-03 acceptance evidence should name api-gateway, workflow-engine, marketplace, cloud-secrets, policy-engine, observability, billing-ledger, and downstream product services.
8. R-03 should state that workflow-engine consumes normalized events but does not own provider webhook verification.
9. R-03 should state that marketplace owns listing economics while connector owns adapter certification and invocation substrate.
10. R-03 should state that cloud-secrets owns secret storage while connector owns credential profile and rotation workflows.
11. R-03 should state that policy-engine owns policy decisions while connector owns policy-decision points and event evidence.
12. R-03 should state that observability owns shared telemetry substrate while connector owns provider, tenant_class, deployment_context, and action_family labels.
13. Remediation R-04 should replace old IaC wording because `microservices/connector/README.md:50`, `microservices/connector/CHANGELOG.md:24`, and `microservices/connector/PHASE-01-INTEGRATION-SUBSTRATE-FOUNDATION.md:30` still point at deprecated deployment surfaces.
14. R-04 acceptance evidence should include OpenTofu-only wording and no canonical Terraform deployment instruction.
15. Remediation R-05 should add six context modules because `specs/master-plan-sequencing.json:704-746` names the context set.
16. R-05 acceptance evidence should include `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`.
17. R-05 should include context-local `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md` equivalents using OpenTofu semantics.
18. R-05 should keep shared provider assumptions out of handwritten shell flows because ADR-0328 D-16 forbids hand-run deployment scripts as the canonical substrate.
19. Remediation R-06 should add `iac/oci-guest/always-free/` because ADR-0328 D-19 and `specs/master-plan-sequencing.json:857-868` require the OCI Always Free profile.
20. R-06 acceptance evidence should declare 4 OCPU and 24 GB memory envelope as a demo_trial infrastructure cap.
21. R-06 should cap throughput and storage without reducing correctness, webhook verification, credential isolation, or DLQ semantics.
22. Remediation R-07 should add `supported-oses.json` because ADR-0328 D-17 requires a service-local OS declaration.
23. R-07 acceptance evidence should enumerate the canonical Linux and macOS Apple Silicon support set plus test-only architecture classes and out-of-scope operating systems.
24. R-07 should explicitly avoid vague "Linux support" wording because ADR-0328 D-17 flags vague OS claims as forbidden.
25. Remediation R-08 should adopt tenant_class semantics because no exact `tenant_class`, `demo_trial`, or `revenue_share` hits exist in the service path.
26. R-08 acceptance evidence should include `demo_trial`, `paid`, and `revenue_share` in manifest, capacity, SLO overlays, policy, and billing handoffs.
27. R-08 should map demo_trial to usage caps and OCI Always Free profile where feasible.
28. R-08 should map paid to per-seat plus usage-based billing and contractual SLO eligibility.
29. R-08 should map revenue_share to at-cost or zero-margin substrate accounting.
30. Remediation R-09 should remove old capability-tier documents from active source-of-truth status because `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_tracks_2026_05_20.md:10-45` retires that model.
31. R-09 acceptance evidence should archive or rewrite `microservices/connector/capability-tiers/tier-matrix.md`.
32. R-09 should rewrite exact references listed in §3.4.T across benchmark, FAQ, tutorial, and capability-matrix documents.
33. R-09 should also rewrite broader tier-vocabulary residue in `microservices/connector/manifest.json:57-60`, `microservices/connector/manifest.json:89`, `microservices/connector/ARCHITECTURE.md:715`, `microservices/connector/ARCHITECTURE.md:723`, and `microservices/connector/policy/abuse-defence.cedar:80`.
34. Remediation R-10 should replace the stale benchmark because `microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:9-13` uses old counterpart framing.
35. R-10 acceptance evidence should use Twilio, Sendbird, and Stream as counterpart anchors while preserving connect as an integration substrate.
36. R-10 should use one industry-leader target set with deployment-context and tenant_class overlays.
37. Remediation R-11 should replace or supersede `microservices/connector/competitor-parity-matrix.md:13-36` because it uses a generic integration-platform peer set rather than this batch’s counterpart set.
38. R-11 acceptance evidence should distinguish provider connector parity from product-category parity.
39. Remediation R-12 should add Sendbird and Stream connector catalog records because the current inventory includes Twilio but not those two counterpart providers.
40. R-12 acceptance evidence should include provider auth, action families, webhook events, rate-limit metadata, and data-handling caveats.
41. Remediation R-13 should deepen the existing Twilio connector record because Twilio public limits include participants, connections, media, sender throughput, and request-rate envelopes.
42. R-13 acceptance evidence should include Twilio product-family facets rather than a single generic provider tag.
43. Remediation R-14 should add provider conformance fixtures because no `microservices/connector/tests/` directory exists.
44. R-14 acceptance evidence should cover webhook signature, duplicate event, rate-limit backoff, outbound action idempotency, and DLQ replay.
45. Remediation R-15 should add Rust source modules because no `microservices/connector/src/` directory exists.
46. R-15 acceptance evidence should include catalog, credential profile, webhook verification, connector action, data mapping, retry, and DLQ modules.
47. R-15 should avoid non-Rust backend tooling because ADR-0328 D-18 and the Rust-strict memory file require Rust for backend and automation.
48. Remediation R-16 should align ADR-MS-001 status because it is proposed at `microservices/connector/decisions/ADR-MS-001-connector-broker-webhook-and-dlq-contract.md:4` while the PRD and architecture are accepted.
49. R-16 acceptance evidence should mark the ADR accepted, superseded, or blocked with rationale.
50. Remediation R-17 should simplify `microservices/connector/capacity-model.md:48` so the accepted formula path is clear and future reviewers do not preserve correction chatter.
51. R-17 acceptance evidence should keep the corrected computation only and bind it to load-test acceptance criteria.
52. Remediation R-18 should preserve strong security material because `microservices/connector/threat-model.md:20-107`, `microservices/connector/dpia.md:47-69`, and `microservices/connector/PRD.md:184-196` are useful substrate anchors.
53. R-18 should only replace retired pricing vocabulary, not weaken credential isolation, replay protection, data residency, or audit requirements.
54. Remediation R-19 should convert existing runbooks into provider-aware runbooks for Twilio, Sendbird, and Stream.
55. R-19 acceptance evidence should include provider degradation, provider throttling, invalid signature, credential revocation cascade, PII leak, and DLQ overflow scenarios.
56. Remediation R-20 should update onboarding, FAQ, migration playbooks, and tutorial content because current user-facing docs are federation-heavy.
57. R-20 acceptance evidence should teach connector setup, credential profile, webhook verification, event normalization, and DLQ replay, not cross-tenant chat federation.
58. Remediation R-21 should add provider observability dimensions to dashboard contracts.
59. R-21 acceptance evidence should include provider, tenant_class, deployment_context, action_family, error_family, and DLQ_state labels.
60. Remediation R-22 should update cost-budget attribution because `microservices/connector/cost-budget.md:42-47` is generic and does not yet bind provider and tenant_class dimensions.
61. R-22 acceptance evidence should show how paid and revenue_share economics flow to billing-ledger without making connect own billing authority.
62. Remediation R-23 should preserve the 500-connector product target from `microservices/connector/PRD.md:156`.
63. R-23 acceptance evidence should show catalog seed growth without moving downstream product actions into connect.
64. Remediation R-24 should keep the boundary from chat history `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:8923-8926`.
65. R-24 acceptance evidence should say connect is not iPaaS, not workflow trigger ownership, and not data-pipeline ownership while still owning connector substrate.
66. Remediation R-25 should run documentation review after implementation evidence exists because `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` requires substance over scaffold.
67. R-25 acceptance evidence should remove generated-expansion residue in `microservices/connector/ARCHITECTURE.md:53-94` and `microservices/connector/compliance.md:35-72`.
68. Remediation R-26 should run a scoped audit after the changes and compare against this report’s F-01 through F-19.
69. R-26 acceptance evidence should include no exact old capability-tier references, tenant_class adoption, OpenTofu context modules, OS manifest, Rust source, tests, and updated counterpart parity.
70. Stop condition: connector becomes coherence-ready when P1 findings are closed, P2 findings have either landed corrections or accepted phase-bound exceptions, and P3 findings are tracked without blocking the next milestone.

<!-- ORCHESTRATOR REPORT
  µservice: connector
  deliverables_landed:
    - microservices/connector/coherence-audit-2026-05-20.md (655 lines)
    - microservices/connector/feature-parity-matrix-2026-05-20.md (407 lines)
    - microservices/connector/performance-benchmark-numbers-2026-05-20.md (315 lines)
  inventory_files_seen: 182
  inventory_lines_read: 31551
  chat_history_matches_processed: 355 filtered connect-specific matches processed; 563 raw connect matches counted
  findings_p0: 0
  findings_p1: 4
  findings_p2: 12
  findings_p3: 3
  tier_retirement_candidates_found: 21
    - microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:13
    - microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:21
    - microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:28
    - microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:48
    - microservices/connector/benchmarks/connect-vs-slack-connect-vs-teams-external-vs-discord.md:83
    - microservices/connector/capability-tiers/tier-matrix.md:13
    - microservices/connector/capability-tiers/tier-matrix.md:21
    - microservices/connector/capability-tiers/tier-matrix.md:48
    - microservices/connector/capability-tiers/tier-matrix.md:50
    - microservices/connector/capability-tiers/tier-matrix.md:79
    - microservices/connector/capability-tiers/tier-matrix.md:81
    - microservices/connector/capability-tiers/tier-matrix.md:109
    - microservices/connector/capability-tiers/tier-matrix.md:113
    - microservices/connector/capability-tiers/tier-matrix.md:115
    - microservices/connector/capability-tiers/tier-matrix.md:127
    - microservices/connector/capability-tiers/tier-matrix.md:129
    - microservices/connector/capability-tiers/tier-matrix.md:142
    - microservices/connector/faqs/federation-engineer-faq.md:74
    - microservices/connector/faqs/federation-engineer-faq.md:76
    - microservices/connector/tutorials/establish-cross-tenant-channel-with-mls-and-cedar.md:15
    - microservices/connector/tutorials/establish-cross-tenant-channel-with-mls-and-cedar.md:176
  tenant_class_adoption_gaps: yes - no exact tenant_class/demo_trial/revenue_share semantics; old paid-tier language only
  top_3_counterparts_confirmed: Twilio / Sendbird / Stream
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1377
-->
