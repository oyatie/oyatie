# Mail Microservice Ownership-Coherence Audit

Audit date: 2026-05-20
Target microservice: `mail`
Audit owner: single-agent read/write audit for `microservices/mail/`
Required deliverables: three documents; the capability tier delta deliverable is retired.
Counterpart bar: Gmail, Microsoft Outlook, Proton Mail.
Deployment-context default: all six canonical contexts unless this audit finds a blocker.
Primary result: mail has strong product-depth artifacts, but it is not ownership-coherent against the 2026-05-20 canonical direction because context-specific OpenTofu, OS support evidence, tenant-class semantics, and tier-retirement cleanup are missing.

## 1. Purpose

1. This audit checks whether `microservices/mail/` is internally coherent, deployable, and aligned with the current Wave 3 canonical direction.
2. The audit is scoped to artifacts under `microservices/mail/` only.
3. The audit does not edit another microservice or shared canonical source.
4. The audit treats the current user directive as the live control surface for this batch.
5. The retired fourth deliverable is not produced.
6. The audit uses the top-three counterpart union bar: Gmail, Microsoft Outlook, and Proton Mail.
7. The audit treats `mail` as a first-class product microservice, not only a notification sidecar.
8. `PRD.md:27-34` defines the product as SMTP, IMAP4rev2, JMAP, and REST mail with mailbox storage, search, compliance, and deliverability responsibilities.
9. `ARCHITECTURE.md:43-55` similarly frames mail as an end-to-end private mail substrate with Gmail/Outlook/Proton/Fastmail-class obligations.
10. The product purpose therefore includes inbound receive, outbound delivery, mailbox storage, indexing, legal hold, DLP, anti-abuse, DKIM/SPF/DMARC, JMAP/IMAP clients, and migration tooling.
11. The audit compares that purpose against three current industry counterparts.
12. Gmail contributes the consumer and Google Workspace collaboration, API, search, and anti-abuse surface.
13. Microsoft Outlook contributes Exchange Online enterprise administration, compliance, eDiscovery, retention, mobile/desktop, transport, and hybrid enterprise expectations.
14. Proton Mail contributes privacy, E2EE, key custody, recovery, bridge/client expectations, and sovereign trust expectations.
15. The audit also evaluates five cross-cutting constraints requested by the batch.
16. Constraint one: six deployment contexts must be represented, explicitly supported, or explicitly ruled out.
17. Constraint two: infrastructure-as-code must be OpenTofu-centered, with no Terraform/Pulumi/CloudFormation/ARM handoff posture.
18. Constraint three: supported OS posture must be explicit, especially the canonical Linux and macOS M5+ support floor.
19. Constraint four: backend and durable automation must stay Rust-strict, with only the explicitly allowed frontend/config/contract exceptions.
20. Constraint five: OCI Always Free profile must be represented as a guest-on-OCI sub-profile, not as a retired capability tier.
21. The audit also runs the new 2026-05-20 tier-retirement check.
22. Existing demo_trial tenant_class/paid tenant_class baseline/paid tenant_class scale/compliance_pack-gated paid tenant_class references are findings, not modeling language for new content.
23. The replacement model is tenant class semantics: `demo_trial`, `paid`, and `revenue_share`.
24. The quality bar is uniform across those classes.
25. The class model affects caps, billing, compliance availability, and substrate economics; it must not split the feature-quality standard.
26. The audit reads the microservice artifacts as evidence, not as assumed truth.
27. A statement is treated as coherent only when the repository has a matching path, manifest entry, contract, runbook, policy, or implementation plan.
28. A missing artifact can still be acceptable if the document explains why the context is not applicable.
29. No such context-wide non-applicability finding was found for mail.
30. The default conclusion is therefore that all six deployment contexts remain in scope.
31. The audit separates product-depth gaps from canonical-direction gaps.
32. Product-depth gaps are places where mail falls short of Gmail, Outlook, or Proton Mail union coverage.
33. Canonical-direction gaps are places where mail uses stale language or missing control surfaces despite otherwise strong product docs.
34. Severity follows the batch direction: P0 blocks truth or safety, P1 blocks canonical deployment or implementation readiness, P2 blocks documentation/coherence cleanup, and P3 is lower-risk polish.
35. This document is the canonical audit summary; the feature matrix and benchmark document hold the expanded comparison surfaces.

## 2. Inventory

### 2.1 Inventory Method

1. Inventory command used: `rg --files microservices/mail | sort`.
2. Files seen: 208.
3. Existing lines under the microservice path: 55,546.
4. `src/` files found: none.
5. `tests/` files found: none.
6. Forbidden durable-source extensions sampled by path search: none found under `microservices/mail/`.
7. Infrastructure directories found under `microservices/mail/iac`: `helm`, `helm/templates`, `kustomize`, `kustomize/base`, `kustomize/overlays`, `kustomize/overlays/pack-kr`, and `kustomize/overlays/pack-us-healthcare`.
8. Canonical context OpenTofu directories were not found for `oyatie-public-cloud`, `guest-on-aws`, `oci-guest`, `oci-guest/always-free`, `on-prem`, `colo`, or `oyatie-as-cloud-provider`.
9. The inventory includes multiple untracked or modified artifacts in the working tree before this audit; this audit does not revert or normalize them.
10. The inventory below is file-complete for the microservice path at investigation time.

### 2.2 Complete File Inventory

1. `microservices/mail/ARCHITECTURE.md`
2. `microservices/mail/AUDIT-FINDINGS-2026-05-18.json`
3. `microservices/mail/CHANGELOG.md`
4. `microservices/mail/IP-001-iac-bootstrap.md`
5. `microservices/mail/IP-002-mailbox-store-kernel.md`
6. `microservices/mail/IP-003-mailbox-store-postgres-adapter.md`
7. `microservices/mail/IP-004-mailbox-store-s3-adapter.md`
8. `microservices/mail/IP-005-dual-context-isolation.md`
9. `microservices/mail/IP-006-inbound-smtp.md`
10. `microservices/mail/IP-007-outbound-smtp.md`
11. `microservices/mail/IP-008-imap-frontend.md`
12. `microservices/mail/IP-009-search-index.md`
13. `microservices/mail/IP-010-retention-policy.md`
14. `microservices/mail/IP-011-legal-hold-engine.md`
15. `microservices/mail/IP-012-ediscovery-export.md`
16. `microservices/mail/IP-013-mail-workflow-handoff.md`
17. `microservices/mail/IP-014-hg-mail-authority-cohesion.md`
18. `microservices/mail/IP-015-pack-kr-overlay.md`
19. `microservices/mail/IP-016-jmap-rfc-8620-frontend.md`
20. `microservices/mail/IP-017-anti-phishing-edge-wiring.md`
21. `microservices/mail/IP-018-hipaa-overlay-rollout.md`
22. `microservices/mail/IP-journey-j01-emergency-family-mail-fallback.md`
23. `microservices/mail/IP-journey-j04-safe-inbox-routing.md`
24. `microservices/mail/IP-journey-j07-inheritance-mail-digest.md`
25. `microservices/mail/IP-journey-j09-recovery-notice-delivery.md`
26. `microservices/mail/IP-journey-j100-pack-rollout-first-action.md`
27. `microservices/mail/IP-journey-j101-tenant-notification.md`
28. `microservices/mail/IP-journey-j105-tenant-notification.md`
29. `microservices/mail/IP-journey-j107-tenant-notification.md`
30. `microservices/mail/IP-journey-j117-customer-notification-trail.md`
31. `microservices/mail/IP-journey-j122-vendor-remittance-notices.md`
32. `microservices/mail/IP-journey-j124-supplier-and-employee-alerts.md`
33. `microservices/mail/IP-journey-j127-mail-archive-on-leaver.md`
34. `microservices/mail/IP-journey-j132-hiring-mail-cascade.md`
35. `microservices/mail/IP-journey-j133-rif-mail-templates.md`
36. `microservices/mail/IP-journey-j136-enrollment-mail-cascade.md`
37. `microservices/mail/IP-journey-j137-corporate-internal-audit-sox-controls-test-archive-reader.md`
38. `microservices/mail/IP-journey-j138-corporate-audit-targeted-correspondence-pull.md`
39. `microservices/mail/IP-journey-j142-work-mail-demotion-and-cross-tenant-packet.md`
40. `microservices/mail/IP-journey-j144-auto-reply-and-digest-delivery.md`
41. `microservices/mail/IP-journey-j145-cross-tenant-offer-letter-delivery.md`
42. `microservices/mail/IP-journey-j146-marketplace-notifications.md`
43. `microservices/mail/IP-journey-j18-authority-notice-delivery.md`
44. `microservices/mail/IP-journey-j22-first-week-inbox.md`
45. `microservices/mail/IP-journey-j23-sale-receipt.md`
46. `microservices/mail/IP-journey-j24-shipping-notices.md`
47. `microservices/mail/IP-journey-j27-imip-invite-bridge.md`
48. `microservices/mail/IP-journey-j35-workplace-deliverability.md`
49. `microservices/mail/IP-journey-j36-approval-notification-thread.md`
50. `microservices/mail/IP-journey-j38-counterparty-envelope.md`
51. `microservices/mail/IP-journey-j40-billing-receipts.md`
52. `microservices/mail/IP-journey-j45-lab-result-notice.md`
53. `microservices/mail/IP-journey-j46-rx-status-messaging.md`
54. `microservices/mail/IP-journey-j47-bill-and-eob-thread.md`
55. `microservices/mail/IP-journey-j48-tax-notice-delivery.md`
56. `microservices/mail/IP-journey-j49-support-email-bridge.md`
57. `microservices/mail/IP-journey-j51-po-ingest-sender.md`
58. `microservices/mail/IP-journey-j52-tracking-notification.md`
59. `microservices/mail/IP-journey-j53-invoice-and-reminder.md`
60. `microservices/mail/IP-journey-j54-quote-delivery.md`
61. `microservices/mail/IP-journey-j55-formal-notice.md`
62. `microservices/mail/IP-journey-j56-offer-letter.md`
63. `microservices/mail/IP-journey-j57-welcome-sequence.md`
64. `microservices/mail/IP-journey-j58-review-summary.md`
65. `microservices/mail/IP-journey-j59-forward-and-retention.md`
66. `microservices/mail/IP-journey-j60-promotion-notice.md`
67. `microservices/mail/IP-journey-j61-patient-summary.md`
68. `microservices/mail/IP-journey-j62-receipt-and-instructions.md`
69. `microservices/mail/IP-journey-j64-hipaa-referral.md`
70. `microservices/mail/IP-journey-j65-mail-export.md`
71. `microservices/mail/IP-journey-j66-regulator-notifications.md`
72. `microservices/mail/IP-journey-j67-user-and-court-notice.md`
73. `microservices/mail/IP-journey-j69-mail-triage.md`
74. `microservices/mail/IP-journey-j70-counterparty-thread.md`
75. `microservices/mail/IP-journey-j71-receipt-and-appeal.md`
76. `microservices/mail/IP-journey-j72-auto-translate-thread.md`
77. `microservices/mail/IP-journey-j73-subscriber-notice.md`
78. `microservices/mail/IP-journey-j74-plugin-mail-actions.md`
79. `microservices/mail/IP-journey-j75-admin-notice.md`
80. `microservices/mail/IP-journey-j76-notice-delivery.md`
81. `microservices/mail/IP-journey-j78-notice-delivery.md`
82. `microservices/mail/IP-journey-j79-notice-delivery.md`
83. `microservices/mail/IP-journey-j80-notice-delivery.md`
84. `microservices/mail/IP-journey-j82-notice-delivery.md`
85. `microservices/mail/IP-journey-j84-notice-delivery.md`
86. `microservices/mail/IP-journey-j85-notice-delivery.md`
87. `microservices/mail/IP-journey-j89-notice-delivery.md`
88. `microservices/mail/IP-journey-j91-us-msb-mtl-overlay.md`
89. `microservices/mail/IP-journey-j92-br-lgpd-us-parent-dsar.md`
90. `microservices/mail/IP-journey-j93-in-dpdpa-rbi-overlay.md`
91. `microservices/mail/IP-journey-j94-sox404-public-company-controls.md`
92. `microservices/mail/IP-journey-j95-iso27001-soc2-annual-audit.md`
93. `microservices/mail/IP-journey-j96-ksa-uae-mena-onboarding.md`
94. `microservices/mail/IP-journey-j97-sg-pdpa-mas-tenant.md`
95. `microservices/mail/IP-journey-j98-au-privacy-apra-cps234.md`
96. `microservices/mail/IP-journey-j99-multi-pack-conflict-resolution.md`
97. `microservices/mail/PHASE-01-MAIL-DISSOLUTION-FROM-CONNECT.md`
98. `microservices/mail/PRD.md`
99. `microservices/mail/README.md`
100. `microservices/mail/backfill-replay.md`
101. `microservices/mail/benchmarks/gmail-m365-proton-vs-oyatie.md`
102. `microservices/mail/capabilities/T0-suggest.yaml`
103. `microservices/mail/capabilities/T1-assist.yaml`
104. `microservices/mail/capabilities/T2-auto.yaml`
105. `microservices/mail/capability-tiers/tier-matrix.md`
106. `microservices/mail/capacity-model.md`
107. `microservices/mail/catalog/oya-mail-anti-phishing-kernel.yaml`
108. `microservices/mail/catalog/oya-mail-dual-context-isolation-kernel.yaml`
109. `microservices/mail/catalog/oya-mail-imap-frontend-rest.yaml`
110. `microservices/mail/catalog/oya-mail-inbound-smtp-adapter-smtp.yaml`
111. `microservices/mail/catalog/oya-mail-inbound-smtp-app.yaml`
112. `microservices/mail/catalog/oya-mail-jmap-frontend-rest.yaml`
113. `microservices/mail/catalog/oya-mail-legal-hold-app.yaml`
114. `microservices/mail/catalog/oya-mail-mailbox-store-adapter-postgres.yaml`
115. `microservices/mail/catalog/oya-mail-mailbox-store-adapter-s3.yaml`
116. `microservices/mail/catalog/oya-mail-mailbox-store-app.yaml`
117. `microservices/mail/catalog/oya-mail-mailbox-store-domain.yaml`
118. `microservices/mail/catalog/oya-mail-mailbox-store-kernel.yaml`
119. `microservices/mail/catalog/oya-mail-mailbox-store-usecase.yaml`
120. `microservices/mail/catalog/oya-mail-outbound-smtp-adapter-smtp.yaml`
121. `microservices/mail/catalog/oya-mail-phi-dlp-adapter-kernel.yaml`
122. `microservices/mail/catalog/oya-mail-retention-policy-worker.yaml`
123. `microservices/mail/catalog/oya-mail-search-index-adapter-tantivy.yaml`
124. `microservices/mail/competitor-parity-matrix.md`
125. `microservices/mail/compliance.md`
126. `microservices/mail/contracts/asyncapi/mail-events.yaml`
127. `microservices/mail/contracts/openapi/mail.yaml`
128. `microservices/mail/contracts/proto/mail.proto`
129. `microservices/mail/cost-budget.md`
130. `microservices/mail/dashboards/abuse-defence-outcomes.json`
131. `microservices/mail/dashboards/delivery-pipeline.json`
132. `microservices/mail/dashboards/dmarc-deliverability.json`
133. `microservices/mail/dashboards/inbox-experience.json`
134. `microservices/mail/dashboards/security-dlp.json`
135. `microservices/mail/decisions/ADR-MAIL-0001-personal-mail-key-recovery.md`
136. `microservices/mail/decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md`
137. `microservices/mail/decisions/ADR-MAIL-0003-sdk-launch-order.md`
138. `microservices/mail/decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md`
139. `microservices/mail/decisions/ADR-MAIL-001-dkim-spf-dmarc-tenant-signing-key-custody.md`
140. `microservices/mail/decisions/README.md`
141. `microservices/mail/deprecation-notice.md`
142. `microservices/mail/dpia.md`
143. `microservices/mail/failure-modes.md`
144. `microservices/mail/faqs/mail-engineer-faq.md`
145. `microservices/mail/iac/ech-config.yaml`
146. `microservices/mail/iac/edge-waf.yaml`
147. `microservices/mail/iac/helm/Chart.yaml`
148. `microservices/mail/iac/helm/templates/deployment.yaml`
149. `microservices/mail/iac/helm/templates/hpa.yaml`
150. `microservices/mail/iac/helm/templates/networkpolicy.yaml`
151. `microservices/mail/iac/helm/templates/pdb.yaml`
152. `microservices/mail/iac/helm/templates/prometheusrule.yaml`
153. `microservices/mail/iac/helm/templates/service.yaml`
154. `microservices/mail/iac/helm/templates/servicemonitor.yaml`
155. `microservices/mail/iac/helm/values.yaml`
156. `microservices/mail/iac/kustomize/base/kustomization.yaml`
157. `microservices/mail/iac/kustomize/overlays/pack-kr/kustomization.yaml`
158. `microservices/mail/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml`
159. `microservices/mail/iac/openbao-policy.yaml`
160. `microservices/mail/iac/pqc-cert.yaml`
161. `microservices/mail/iac/secret-bindings.yaml`
162. `microservices/mail/incident-response.md`
163. `microservices/mail/manifest.json`
164. `microservices/mail/migration-from-connect.md`
165. `microservices/mail/migration-playbooks/from-gmail-workspace.md`
166. `microservices/mail/multi-region.md`
167. `microservices/mail/onboarding/mail-engineer-first-week.md`
168. `microservices/mail/packs/EU-AI-Act.md`
169. `microservices/mail/packs/GDPR.md`
170. `microservices/mail/packs/HIPAA.md`
171. `microservices/mail/packs/KR-PIPA.md`
172. `microservices/mail/packs/SOC2.md`
173. `microservices/mail/policy/abuse-defence.cedar`
174. `microservices/mail/policy/anti-phishing.cedar`
175. `microservices/mail/policy/auditor-scope.cedar`
176. `microservices/mail/policy/ci-scope.cedar`
177. `microservices/mail/policy/data-residency.md`
178. `microservices/mail/policy/dual-context-isolation.md`
179. `microservices/mail/policy/minor-protection.cedar`
180. `microservices/mail/policy/phi-dlp.cedar`
181. `microservices/mail/policy/public-read.cedar`
182. `microservices/mail/policy/tenant-scope.cedar`
183. `microservices/mail/reference-implementations/send-signed-mail-rust-sdk.md`
184. `microservices/mail/runbooks/account-compromise-recovery.md`
185. `microservices/mail/runbooks/dkim-key-rotation.md`
186. `microservices/mail/runbooks/dlp-quarantine-release.md`
187. `microservices/mail/runbooks/dmarc-rollout-monitoring.md`
188. `microservices/mail/runbooks/e2e-encryption-key-recovery.md`
189. `microservices/mail/runbooks/mail-bot-score-recalibration.md`
190. `microservices/mail/runbooks/mailbox-restore-from-backup.md`
191. `microservices/mail/runbooks/phi-leak-recovery.md`
192. `microservices/mail/runbooks/smtp-queue-backup.md`
193. `microservices/mail/runbooks/spam-rule-rollback.md`
194. `microservices/mail/scorecards/overrides.json`
195. `microservices/mail/sdk-plan.md`
196. `microservices/mail/security/threat-model.md`
197. `microservices/mail/slos/dual-context-correctness.openslo.yaml`
198. `microservices/mail/slos/ediscovery-export-freshness.openslo.yaml`
199. `microservices/mail/slos/inbound-receive-availability.openslo.yaml`
200. `microservices/mail/slos/inbox-open-latency.openslo.yaml`
201. `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml`
202. `microservices/mail/slos/legal-hold-engage-latency.openslo.yaml`
203. `microservices/mail/slos/outbound-delivery-latency.openslo.yaml`
204. `microservices/mail/slos/search-latency.openslo.yaml`
205. `microservices/mail/slos/spam-classification-latency.openslo.yaml`
206. `microservices/mail/slos/thread-render-latency.openslo.yaml`
207. `microservices/mail/threat-model.md`
208. `microservices/mail/tutorials/promote-dmarc-policy-with-soak.md`

### 2.3 Artifact Coverage Notes

1. Product definition is substantial: `PRD.md:27-34` defines mail as a full protocol and mailbox product.
2. Architecture is substantial: `ARCHITECTURE.md:43-55` names Gmail/Outlook/Proton-class expectations and core subsystems.
3. Contracts are present: `contracts/openapi/mail.yaml:1`, `contracts/asyncapi/mail-events.yaml:1`, and `contracts/proto/mail.proto:5`.
4. SLO artifacts are present: ten OpenSLO YAML files were inventoried under `slos/`.
5. Runbooks are present: ten runbooks were inventoried under `runbooks/`.
6. Capability docs are present, but one directory is now a retirement candidate: `capability-tiers/tier-matrix.md`.
7. Infrastructure artifacts are present, but they are Kubernetes packaging artifacts rather than canonical context OpenTofu modules.
8. The `manifest.json` exists and includes component and contract surfaces, but it lacks a `tenant_class` axis.
9. Journey implementation plans are unusually broad, which is positive for downstream product workflows.
10. The broad journey inventory also increases coherence risk because mail is referenced by many domain workflows before the platform substrate is complete.
11. There is no source directory evidence for a Rust implementation under this path.
12. There is no test directory evidence under this path.
13. The lack of `src/` and `tests/` is not automatically a product design failure, but it blocks implementation-readiness claims.
14. The audit therefore treats mail as a high-substance planning package with missing canonical execution gates.
15. The package is not empty scaffold; it is also not deployment-complete.

### 2.4 Chat History Evidence

1. Chat history was searched at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
2. Matches processed: five.
3. Chat line `15947` records the user direction: "we don't have tiers."
4. Chat line `15963` records memory creation for the no-tier directive and the retirement of capability tier surfaces.
5. Chat line `16004` records Batch 3.2 dropping the fourth capability-tier-delta deliverable and requiring the performance report to use a single target with deployment-context overlays.
6. Chat line `78` contains the current batch prompt content for the mail audit, including the target microservice and counterpart bar.
7. Chat line `16521` repeats the Wave 15 retirement reminder during the broader work queue.
8. These chat lines are treated as current steering evidence for this report.

## 3. Nine-Dimension Audit

### 3.1 Dimension 1: Product Purpose and Bounded Context

1. Verdict: strong product purpose, with one major ownership-coherence risk.
2. `PRD.md:27-34` clearly owns SMTP, IMAP4rev2, JMAP, REST, mailbox storage, search, compliance, DLP, legal hold, and deliverability.
3. `ARCHITECTURE.md:43-55` confirms the same service identity through DKIM/SPF/DMARC, Cedar, spam classification, and encrypted storage.
4. The product is not merely a notification adapter.
5. The product is a full mailbox and mail-transport system.
6. Gmail counterpart coverage requires consumer inbox, Workspace administration, search, anti-spam, storage, API, migration, mobile/desktop, and abuse controls.
7. Outlook counterpart coverage requires Exchange-grade admin, transport rules, eDiscovery, retention, DLP, mobile/desktop, hybrid, and incident posture.
8. Proton counterpart coverage requires privacy-first encryption, client-side trust boundaries, recovery, bridge/client posture, and high trust in custody separation.
9. Mail artifacts cover many of those surfaces: contracts, SLOs, runbooks, packs, policies, and migration docs exist.
10. The coherence risk is that backend selection and tenancy semantics are still keyed to stale tenant-tier language in accepted ADRs.
11. `decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md:23` names the decision as backend selection per tenant tier.
12. `decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md:48-50` splits Postfix/Dovecot and Stalwart by tenant tier.
13. That split is now stale under the 2026-05-20 replacement model.
14. Product purpose remains valid, but the execution axis needs migration from tenant tier to tenant class plus workload profile.
15. Severity for product purpose: P1 for backend ownership semantics; P2 for documentation cleanup.

### 3.2 Dimension 2: Artifact Substance and Completeness

1. Verdict: high substance, not line-count padding.
2. The PRD spans detailed feature, SLO, security, migration, and acceptance criteria surfaces.
3. The architecture document maps domains, ports, adapters, policies, storage, and operational cells.
4. Five mail ADRs exist and address key custody, backend selection, SDK ordering, AI classifier scope, and DKIM/SPF/DMARC custody.
5. Eighteen numbered implementation plans exist, plus a large set of journey-specific implementation plans.
6. Contracts exist in OpenAPI, AsyncAPI, and protobuf forms.
7. Ten SLO artifacts exist under `slos/`.
8. Ten runbooks exist under `runbooks/`.
9. Data protection, compliance, cost, capacity, failure, and incident-response documents exist.
10. The substance problem is not absence of docs.
11. The substance problem is that some docs point to artifacts that do not exist.
12. `PRD.md:58` says Cedar fragments are under `policy/cedar/{personal,work,internal}.cedar`.
13. The inventory shows policy files directly under `policy/`, not under `policy/cedar/`.
14. `ARCHITECTURE.md:115-123` lists the actual policy files directly under `policy/`.
15. `incident-response.md:130-145` references multiple runbooks that are absent from the inventory.
16. `failure-modes.md:27-194` defines thirteen failure modes, several of which reference runbook names that are absent or renamed.
17. `compliance.md:85` cites `iac/terraform/cedar-rbac.tf`, which is a stale and forbidden infrastructure path under the current OpenTofu doctrine.
18. `benchmarks/gmail-m365-proton-vs-oyatie.md:113-121` describes a benchmark harness path and CLI that were not present in inventory.
19. The artifact set is therefore mature in breadth but not yet ownership-clean.
20. Severity: P2 for broken evidence paths; P1 when the broken path affects canonical deployment readiness.

### 3.3 Dimension 3: Contracts and API Surfaces

1. Verdict: contract presence is strong, but implementation evidence is absent.
2. `contracts/openapi/mail.yaml:1` provides the REST contract root.
3. `contracts/asyncapi/mail-events.yaml:1` provides event-surface evidence.
4. `contracts/proto/mail.proto:5` provides the protobuf service surface.
5. The contract family aligns with API-first direction in `specs/master-plan-sequencing.json:98-182`.
6. The OpenAPI operation list covers mailbox, message, compose, attachment, workflow, and compliance-facing endpoints.
7. The AsyncAPI contract covers mail events and integration handoffs.
8. The protobuf contract suggests internal service contracts for Rust implementation.
9. No generated SDK source tree exists under `src/`.
10. No Rust server source tree exists under `src/`.
11. No conformance test directory exists under `tests/`.
12. `PRD.md:1109-1158` lists acceptance criteria that invoke shell-style checks and conformance scripts that are absent.
13. Contract design is therefore ahead of implementation proof.
14. A contract package without implementation tests can still be useful, but it cannot satisfy production readiness.
15. Severity: P2 for implementation evidence gap; P1 if used to claim deployable readiness.

### 3.4 Dimension 4: Canonical-Direction Alignment

1. Verdict: not aligned yet.
2. Canonical direction requires six deployment contexts, OpenTofu IaC, OS support manifests, Rust-strict implementation, OCI Always Free profile, and tenant-class replacement for retired tiers.
3. Mail has strong product docs but misses most machine-readable canonical direction gates.
4. The `iac/` tree contains Helm and Kustomize artifacts, not context-specific OpenTofu modules.
5. There is no `supported-oses.json`.
6. There is no `tenant_class` field in the mail path.
7. There are 73 rank-token references under the mail path, counting the lower-case benchmark flag that still encodes the retired model.
8. There are additional stale non-exact tenant-tier references such as starter/pro/enterprise in accepted ADRs.
9. `manifest.json:418-422` still names `tenant_class_adoption`.
10. `manifest.json:451` still uses `tier_classification`.
11. These fields are now part of the Wave 15J retirement surface.
12. `feedback_no_tenant_class_adoption_2026_05_20.md:10` records the controlling doctrine: the user says there are no tiers.
13. `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139-142` records the batch-specific drop of the tier-delta deliverable and the single performance target direction.
14. The user prompt for this audit supersedes older two-class memory wording and names three classes: `demo_trial`, `paid`, and `revenue_share`.
15. Mail needs a canonical migration plan that rewrites capability gates into tenant-class plus deployment-context overlays.

#### 3.4.T Tier Retirement Candidates

1. Finding type: Wave 15J retirement candidate.
2. Default severity: P2 unless the reference controls backend/runtime selection.
3. Total rank-token retirement references found: 73.
4. `onboarding/mail-engineer-first-week.md:12` uses demo_trial tenant_class cell in the onboarding outcome.
5. `onboarding/mail-engineer-first-week.md:27` uses demo_trial tenant_class in a section heading.
6. `onboarding/mail-engineer-first-week.md:191` uses paid tenant_class baseline feature language for key recovery.
7. `onboarding/mail-engineer-first-week.md:266` uses paid tenant_class baseline tier language for Ed25519 promotion.
8. `onboarding/mail-engineer-first-week.md:294` uses demo_trial tenant_class bootstrap language.
9. `onboarding/mail-engineer-first-week.md:301` uses paid tenant_class baseline, paid tenant_class scale, and compliance_pack-gated paid tenant_class tour language.
10. `migration-playbooks/from-gmail-workspace.md:30` uses paid tenant_class baseline cost comparison.
11. `migration-playbooks/from-gmail-workspace.md:31` uses compliance_pack-gated paid tenant_class eDiscovery language.
12. `migration-playbooks/from-gmail-workspace.md:33` uses paid tenant_class scale classifier comparison.
13. `migration-playbooks/from-gmail-workspace.md:136` uses paid tenant_class baseline backfill rate language.
14. `capability-tiers/tier-matrix.md:15` defines demo_trial tenant_class capability.
15. `capability-tiers/tier-matrix.md:21` uses demo_trial tenant_class mailbox quota.
16. `capability-tiers/tier-matrix.md:37` defers LLM classification to paid tenant_class baseline.
17. `capability-tiers/tier-matrix.md:58` defines paid tenant_class baseline capability.
18. `capability-tiers/tier-matrix.md:60` adds to demo_trial tenant_class.
19. `capability-tiers/tier-matrix.md:95` defines paid tenant_class scale capability.
20. `capability-tiers/tier-matrix.md:97` adds to paid tenant_class baseline.
21. `capability-tiers/tier-matrix.md:128` compares cost delta from paid tenant_class baseline.
22. `capability-tiers/tier-matrix.md:132` defines compliance_pack-gated paid tenant_class capability.
23. `capability-tiers/tier-matrix.md:134` adds to paid tenant_class scale.
24. `capability-tiers/tier-matrix.md:156` says same as paid tenant_class scale per pack.
25. `capability-tiers/tier-matrix.md:159` says same as paid tenant_class scale plus pack-bound availability.
26. `capability-tiers/tier-matrix.md:173` defines demo_trial tenant_class-to-paid tenant_class baseline, paid tenant_class baseline-to-paid tenant_class scale, and paid tenant_class scale-to-compliance_pack-gated paid tenant_class promotion gates.
27. `capability-tiers/tier-matrix.md:180` uses paid tenant_class for DKIM Ed25519.
28. `capability-tiers/tier-matrix.md:182` uses paid tenant_class compliance pack for MTA-STS enforcement.
29. `capability-tiers/tier-matrix.md:183` uses paid tenant_class compliance pack for TLSRPT ingestion.
30. `capability-tiers/tier-matrix.md:184` uses paid tenant_class for ARC forwarder allowlist.
31. `capability-tiers/tier-matrix.md:185` uses paid tenant_class for recovery envelope.
32. `capability-tiers/tier-matrix.md:188` uses compliance_pack-gated paid tenant_class for FIPS HSM key custody.
33. `capability-tiers/tier-matrix.md:189` uses compliance_pack-gated paid tenant_class for sovereign residency.
34. `capability-tiers/tier-matrix.md:190` uses paid tenant_class for DMARC dashboards.
35. `capability-tiers/tier-matrix.md:192` uses compliance_pack-gated paid tenant_class for mailbox DEK envelope.
36. `capability-tiers/tier-matrix.md:193` uses compliance_pack-gated paid tenant_class for legal hold.
37. `reference-implementations/send-signed-mail-rust-sdk.md:123` names paid tenant_class baseline-tier expected output.
38. `benchmarks/gmail-m365-proton-vs-oyatie.md:13` names paid tenant_class baseline hardware.
39. `benchmarks/gmail-m365-proton-vs-oyatie.md:21` names paid tenant_class baseline DKIM measurement.
40. `benchmarks/gmail-m365-proton-vs-oyatie.md:22` names paid tenant_class baseline DKIM measurement.
41. `benchmarks/gmail-m365-proton-vs-oyatie.md:23` names paid tenant_class scale DKIM measurement.
42. `benchmarks/gmail-m365-proton-vs-oyatie.md:30` compares paid tenant_class baseline and paid tenant_class scale SLO posture.
43. `benchmarks/gmail-m365-proton-vs-oyatie.md:36` names paid tenant_class baseline throughput.
44. `benchmarks/gmail-m365-proton-vs-oyatie.md:37` names paid tenant_class scale throughput.
45. `benchmarks/gmail-m365-proton-vs-oyatie.md:50` names paid tenant_class baseline JMAP fetch latency.
46. `benchmarks/gmail-m365-proton-vs-oyatie.md:51` names paid tenant_class scale JMAP fetch latency.
47. `benchmarks/gmail-m365-proton-vs-oyatie.md:64` names paid tenant_class baseline Rspamd classifier.
48. `benchmarks/gmail-m365-proton-vs-oyatie.md:65` names paid tenant_class baseline LLM classifier.
49. `benchmarks/gmail-m365-proton-vs-oyatie.md:66` names paid tenant_class scale hybrid classifier.
50. `benchmarks/gmail-m365-proton-vs-oyatie.md:73` names paid tenant_class scale hybrid accuracy.
51. `benchmarks/gmail-m365-proton-vs-oyatie.md:79` names paid tenant_class baseline Stalwart backend.
52. `benchmarks/gmail-m365-proton-vs-oyatie.md:80` names paid tenant_class scale JMAP latency.
53. `benchmarks/gmail-m365-proton-vs-oyatie.md:93` names paid tenant_class baseline self-hosted cost.
54. `benchmarks/gmail-m365-proton-vs-oyatie.md:94` names paid tenant_class scale cost.
55. `benchmarks/gmail-m365-proton-vs-oyatie.md:102` compares paid tenant_class baseline and paid tenant_class scale costs.
56. `benchmarks/gmail-m365-proton-vs-oyatie.md:118` uses `--tier paid` in the benchmark command.
57. `tutorials/promote-dmarc-policy-with-soak.md:16` requires paid tenant_class for the tutorial.
58. `tutorials/promote-dmarc-policy-with-soak.md:241` names paid tenant_class baseline tier.
59. `faqs/mail-engineer-faq.md:48` uses paid tenant_class for Ed25519.
60. `faqs/mail-engineer-faq.md:116` uses paid tenant_class and demo_trial tenant_class for backend selection.
61. `faqs/mail-engineer-faq.md:120` references compliance_pack-gated paid tenant_class mailbox DEK envelope.
62. `faqs/mail-engineer-faq.md:122` references paid tenant_class baseline/paid tenant_class scale server-side encryption.
63. `faqs/mail-engineer-faq.md:123` references compliance_pack-gated paid tenant_class custody.
64. `faqs/mail-engineer-faq.md:127` says KR-PIPA defaults to compliance_pack-gated paid tenant_class.
65. `faqs/mail-engineer-faq.md:128` says HIPAA can accept paid tenant_class scale.
66. `faqs/mail-engineer-faq.md:129` says FedRAMP-High recommends compliance_pack-gated paid tenant_class.
67. `faqs/mail-engineer-faq.md:135` lists Rust SDK as demo_trial tenant_class.
68. `faqs/mail-engineer-faq.md:136` lists TypeScript SDK as paid tenant_class baseline.
69. `faqs/mail-engineer-faq.md:137` lists Python SDK as paid tenant_class baseline.
70. `faqs/mail-engineer-faq.md:138` lists Go SDK as paid tenant_class scale.
71. `faqs/mail-engineer-faq.md:139` lists Java/Kotlin SDK as paid tenant_class scale.
72. `faqs/mail-engineer-faq.md:140` lists Swift/Kotlin Multiplatform mobile as paid tenant_class scale.
73. `faqs/mail-engineer-faq.md:153-155` defines MTA-STS by demo_trial tenant_class, paid tenant_class baseline, and paid tenant_class compliance pack.
74. `faqs/mail-engineer-faq.md:180` references air-gap compliance_pack-gated paid tenant_class.
75. Retirement action: replace the capability model with tenant class plus workload profile plus context overlay.
76. Retirement action: delete or quarantine `capability-tiers/` after Wave 15J migration has a replacement registry.
77. Retirement action: rewrite benchmark rows so they express one industry-leader target and context caps.
78. Retirement action: rewrite onboarding so a new engineer bootstraps `demo_trial` OCI Always Free profile and production-grade paid/revenue-share profiles without implying lower product quality.
79. Retirement action: rewrite FAQ SDK ordering using canonical Rust backend and allowed generated SDK exception rules.
80. Retirement action: rewrite backend selection around workload profile, compliance pack, volume envelope, and deployment context, not legacy capability rank.

#### 3.4.C Tenant-Class Adoption Gaps

1. Verdict: tenant-class adoption gap exists.
2. Search for `tenant_class` returned no matches under the mail path.
3. Search for `demo_trial` returned no matches under the mail path.
4. Search for `revenue_share` returned no matches under the mail path.
5. `PRD.md:291` still says custom domain is an optional paid tier.
6. `ARCHITECTURE.md:171` uses `provider_credential_mode` with `TENANT_BYOK` for paid and `PLATFORM_MANAGED` for free tier.
7. `ARCHITECTURE.md:832` uses paid family tier language for minor-remediation account provisioning.
8. `decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md:40` says tenant-tier model segments tenants.
9. `decisions/ADR-MAIL-0001-personal-mail-key-recovery.md:56` gates escrow by tenant tier.
10. `manifest.json:418-422` uses `tenant_class_adoption`.
11. No artifact defines `demo_trial`, `paid`, or `revenue_share` as mail admission, billing, SLO, compliance, BYOK, or infrastructure overlay inputs.
12. The gap is not just naming.
13. Mail currently binds operational behavior to the retired model.
14. The replacement must express `demo_trial` as OCI Always Free profile with usage and time caps.
15. The replacement must express `paid` as per-seat plus usage billing across any supported context.
16. The replacement must express `revenue_share` as gross-revenue-share business model with at-cost or zero-margin substrate.
17. Uniform feature quality must remain true across all three.
18. The differences should be quotas, support/SLO contract, compliance availability, BYOK availability, and substrate economics.
19. Recommended owner action: add a `tenant_class` section to `manifest.json`, PRD tenancy, and backend-selection ADR replacement.
20. Recommended owner action: link tenant class to usage caps and billing, not to quality rank.

### 3.5 Dimension 5: Multi-Context Deployment Coverage

1. Verdict: P1 blocker.
2. Canonical contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
3. The user prompt defaults mail to all six unless the audit finds otherwise.
4. No artifact proves any context is not applicable for mail.
5. Mail is especially context-sensitive because deliverability, egress, DNS, abuse desk, and inbound reputation differ by context.
6. ADR-0328 says mail public cloud may rely on hosted reputation surfaces while on-prem and colo need explicit DKIM/SPF/DMARC, abuse, and egress constraints.
7. Existing `iac/` has no context directories.
8. Existing `iac/` has no `guest-on-oci/always-free` profile.
9. Existing `iac/` has no `oyatie-public-cloud` module.
10. Existing `iac/` has no `guest-on-aws` module.
11. Existing `iac/` has no `on-prem` module.
12. Existing `iac/` has no `colo` module.
13. Existing `iac/` has no `oyatie-as-cloud-provider` module.
14. Helm and Kustomize can remain useful deployment packaging, but they do not satisfy the context evidence gate alone.
15. Mail's deliverability posture requires per-context DNS, egress IP pool, bounce handling, abuse reporting, DKIM key custody, inbound MX, and secret management.
16. Those concerns cannot be inferred from generic Kubernetes deployment YAML.
17. The absence is a deployment-readiness blocker for all six contexts.
18. Severity: P1.

### 3.6 Dimension 6: OpenTofu Infrastructure-as-Code

1. Verdict: P1 blocker.
2. Canonical doctrine requires OpenTofu and `tofu init`, `tofu plan`, and `tofu apply` posture.
3. Current mail `iac/` contains YAML for Helm, Kustomize, OpenBao policy, WAF, ECH, PQC certificate, and secret bindings.
4. No `.tf` OpenTofu module files were found under `microservices/mail/iac`.
5. No `versions.tf`, `main.tf`, `variables.tf`, `outputs.tf`, or context `README.md` module families were found.
6. `compliance.md:85` cites `iac/terraform/cedar-rbac.tf`, which conflicts with the OpenTofu-only direction and the actual inventory.
7. No Pulumi, CloudFormation, ARM, or manual-console implementation files were found under mail.
8. The major failure is absence of required OpenTofu modules, not presence of forbidden module files.
9. The stale Terraform path should be scrubbed because it teaches future owners the wrong substrate.
10. Recommended owner action: create context modules for all six contexts plus OCI Always Free profile, then keep Helm/Kustomize as rendered deployment payloads if still needed.
11. Recommended owner action: add `tofu` validation evidence to the mail implementation plan.
12. Recommended owner action: ensure mail-specific DNS, DKIM, MTA-STS, TLSRPT, OpenBao, queue, object storage, and observability resources are represented as typed OpenTofu modules.
13. Severity: P1 for missing modules; P2 for stale Terraform reference.

### 3.7 Dimension 7: OS Support

1. Verdict: P1 blocker.
2. `supported-oses.json` was not found under `microservices/mail/`.
3. No alternative mail-local OS manifest was found.
4. No `src/` implementation was found, so there is no build matrix evidence.
5. No `tests/` directory was found, so there is no OS conformance evidence.
6. Mail needs Linux server packaging and macOS M5+ compatibility where developer tooling or native client-adjacent workflows apply.
7. The canonical OS list includes Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, Alma, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS M5+.
8. The canonical architecture list includes x86_64, aarch64/arm64, ppc64le test-only, and s390x test-only.
9. Mail's protocol surface also has platform-sensitive dependencies: DNS, SMTP, TLS, DKIM crypto, filesystem/object-store adapters, and mailbox search.
10. Those dependencies need explicit OS assumptions.
11. The current docs make strong production claims without machine-readable OS support.
12. Recommended owner action: add `supported-oses.json` with build, runtime, packaging, architecture, and exclusion rationale.
13. Recommended owner action: link OS manifest to OpenTofu contexts and Rust build/test plans.
14. Severity: P1 until manifest and evidence exist.

### 3.8 Dimension 8: Rust-Strict Language Posture

1. Verdict: mixed.
2. Positive evidence: no forbidden durable-source files with `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, `.fsx`, or `.cs` extensions were found under `microservices/mail/`.
3. Positive evidence: the reference implementation path is Rust-named: `reference-implementations/send-signed-mail-rust-sdk.md`.
4. Positive evidence: catalog components use Rust-oriented names such as `adapter-tantivy` and kernel/domain/usecase boundaries.
5. Negative evidence: no Rust source exists under `src/`.
6. Negative evidence: `PRD.md:169-170` names Next.js and Tauri surfaces, which do not align with canonical Leptos web SSR plus selective island hydration.
7. Negative evidence: `PRD.md:1109-1158` lists acceptance commands that call shell scripts, but no corresponding test files exist under `tests/`.
8. Negative evidence: `faqs/mail-engineer-faq.md:135-140` describes SDK ordering with TypeScript, Python, Go, Java/Kotlin, and mobile SDKs using the retired capability model.
9. SDKs can be allowed when generated from canonical contracts, but the docs need to say generated and governed rather than hand-authored application logic.
10. Web frontend must be Leptos/WASM SSR with selective island hydration under the current policy.
11. Native frontends must stay Swift, Kotlin, or WinUI 3 where applicable.
12. Backend, CLI, validators, automation, CI test tools, and codegen must be Rust.
13. Recommended owner action: replace Next.js/Tauri references with canonical Leptos web and allowed native-client language posture.
14. Recommended owner action: replace shell-script acceptance criteria with Rust conformance binaries or Cargo tasks.
15. Recommended owner action: add `src/` and `tests/` proof only when implementation work is in scope; until then avoid claiming implementation readiness.
16. Severity: P1 for stale implementation direction; P2 for SDK wording cleanup.

### 3.9 Dimension 9: SLOs, Operations, Risk, and Cross-Microservice Ownership

1. Verdict: strong operational ambition, incomplete evidence links.
2. Ten OpenSLO artifacts exist.
3. `PRD.md:952-968` gives explicit p50/p95/p99 latency targets and throughput targets.
4. `PRD.md:972-982` gives availability and recovery targets.
5. `failure-modes.md:27-194` provides a meaningful failure catalog.
6. `incident-response.md:24-33` defines severities.
7. `incident-response.md:70-105` defines notification posture.
8. `runbooks/` includes key recovery, DKIM rotation, DLP release, DMARC monitoring, mailbox restore, and spam rollback.
9. The evidence gap is link integrity.
10. Incident and failure docs reference runbook names that are not in inventory.
11. Examples include `smtp-relay-outage.md`, `deliverability-reputation-recovery.md`, `search-index-rebuild.md`, `imap-storm-throttle.md`, `mailbox-restore.md`, `legal-hold-engage.md`, `ediscovery-export.md`, and `dkim-rotation-recovery.md`.
12. Some of these may map to existing differently named runbooks.
13. The current state still requires a crosswalk or rename because incident responders need exact paths.
14. `ARCHITECTURE.md:1153-1158` lists cross-service dependencies, which is useful.
15. The architecture should be tightened so each dependency names owned inputs, outputs, failure contracts, and test evidence.
16. Severity: P2 for link integrity; P1 if used in a readiness gate.

## 4. Findings Table

| ID | Severity | Finding | Evidence | Required owner action |
| --- | --- | --- | --- | --- |
| MAIL-AUD-P1-001 | P1 | Six-context deployment evidence is missing. | Inventory shows only `iac/helm` and `iac/kustomize`; no canonical context directories. | Add OpenTofu modules for all six contexts or explicit N/A rationale per context. |
| MAIL-AUD-P1-002 | P1 | OCI Always Free profile is absent. | No `microservices/mail/iac/oci-guest/always-free/` path exists. | Add guest-on-OCI Always Free profile with mail-specific quotas, DNS, queue, and no production-overclaim. |
| MAIL-AUD-P1-003 | P1 | OS support manifest is absent. | No `microservices/mail/supported-oses.json`; no `src/`; no `tests/`. | Add supported OS manifest and link to Rust build/test evidence. |
| MAIL-AUD-P1-004 | P1 | Backend selection is still keyed to retired tenant-tier language. | `decisions/ADR-MAIL-0002-backend-tenant-tier-policy.md:23`, `:40`, `:48-54`, `:89`, `:99`, `:106`, `:113`. | Replace with tenant class plus workload, compliance, and context selection model. |
| MAIL-AUD-P1-005 | P1 | Rust-strict implementation direction is stale. | `PRD.md:169-170` names Next.js/Tauri; `PRD.md:1109-1158` lists shell-script checks. | Replace with Leptos web posture and Rust conformance/test tooling. |
| MAIL-AUD-P2-001 | P2 | Exact retired capability-rank references remain. | 73 exact references listed in section 3.4.T. | Wave 15J scrub and replacement with tenant class plus context overlays. |
| MAIL-AUD-P2-002 | P2 | Tenant-class semantics are absent. | No `tenant_class`, `demo_trial`, or `revenue_share` matches under mail path. | Add tenant class to PRD, manifest, ADRs, billing, SLO, and admission surfaces. |
| MAIL-AUD-P2-003 | P2 | PRD policy path is stale. | `PRD.md:58` says `policy/cedar/{personal,work,internal}.cedar`; actual files are direct `policy/*.cedar`; `ARCHITECTURE.md:115-123` lists actual paths. | Normalize PRD policy paths or create the referenced directory if intended. |
| MAIL-AUD-P2-004 | P2 | Terraform path remains in compliance evidence. | `compliance.md:85` cites `iac/terraform/cedar-rbac.tf`. | Replace with OpenTofu evidence path after modules exist. |
| MAIL-AUD-P2-005 | P2 | Incident/failure docs reference missing runbooks. | `incident-response.md:130-145`; `failure-modes.md:27-194`; inventory has only ten runbook files. | Add missing runbooks or a precise path crosswalk. |
| MAIL-AUD-P2-006 | P2 | AI classifier governance references missing policy/evidence paths. | `decisions/ADR-MAIL-0004-spam-classifier-eu-ai-act-scope.md:123-127` references policy and DPIA evidence paths not found in inventory. | Add missing EU AI Act policy and DPIA tenant evidence paths or revise ADR. |
| MAIL-AUD-P2-007 | P2 | Benchmark harness evidence is not present. | `benchmarks/gmail-m365-proton-vs-oyatie.md:113-121` names a harness and retired command axis; no harness path in inventory. | Replace with Rust benchmark harness evidence and current tenant/context axes. |
| MAIL-AUD-P3-001 | P3 | Counterpart set in existing docs is broader than this audit bar. | `PRD.md:21` and `ARCHITECTURE.md:43-55` include additional competitors beyond Gmail, Outlook, and Proton Mail. | Keep broader references as context, but normalize audit matrices to the required top-three set. |
| MAIL-AUD-P3-002 | P3 | Manifest still uses old capability terminology for component grouping. | `manifest.json:418-422`, `manifest.json:451`. | Rename after Wave 15J replacement registry is available. |

## 5. Open Questions

1. Should backend selection be rewritten as a new ADR that supersedes ADR-MAIL-0002, or should ADR-MAIL-0002 be amended in place?
2. Which OpenTofu module should own mail egress IP pool warm-up: mail itself, cloud-network-dns, or a shared deliverability substrate?
3. Should `demo_trial` mail allow custom domains at all, or only `@oyatie.app` hosted mail with strict daily send caps?
4. What is the exact demo_trial quota envelope for mail: daily outbound messages, recipients per message, mailbox storage, attachment size, IMAP/JMAP bandwidth, and retention duration?
5. Should `revenue_share` tenants receive the same contractual SLO mechanics as paid tenants, or a separate revenue-based scale covenant?
6. Which runbook names are canonical: the names referenced by incident/failure docs, or the ten files currently present in `runbooks/`?
7. Should JMAP be the primary public client protocol, with IMAP4rev2 as compatibility, or should both be equal public contracts?
8. Does ActiveSync remain a roadmap surface from `PRD.md:167`, or should it be retired in favor of JMAP/IMAP and native apps?
9. Should Proton-style mailbox recovery remain tenant-gated, or should every tenant class get the same cryptographic recovery quality with different support/custody economics?
10. Which mail implementation plan owns `supported-oses.json` and Rust conformance harness creation?
11. Should DKIM Ed25519, MTA-STS, TLSRPT, ARC, and BIMI be mandatory for all production tenant classes, with only rollout timing differing by context?
12. Should `capacity-model.md` be rewritten around workload profiles instead of old scale names?
13. Should the `capability-tiers/` directory be deleted in Wave 15J or retained as an archival migration input until replacement specs land?
14. Should benchmark documents cite only public counterpart limits, or also keep internal measured harness data once a Rust harness exists?
15. What is the acceptance rule for mail under OCI Always Free profile, given ADR-0328 warns OCI Email Delivery free allotment is too small for full production readiness?
16. Should mail maintain separate personal and work mailbox schemas, or enforce the current dual-context model with one schema and Cedar isolation?
17. Should all references to free/paid/starter/pro/enterprise be replaced in this microservice during Wave 15J, or only references that imply quality stratification?
18. Should SDK plans be governed by Stainless-generated SDK policy, with Rust as the reference and other SDKs generated from contracts?
19. Should mobile/native client work stay outside this microservice path and live in frontend repositories, with mail exposing only contracts and Rust backend?
20. What is the minimum evidence bundle required before mail can claim deployability in each of the six contexts?

<!-- ORCHESTRATOR REPORT
  µservice: mail
  deliverables_landed:
    - microservices/mail/coherence-audit-2026-05-20.md (642 lines)
    - microservices/mail/feature-parity-matrix-2026-05-20.md (408 lines)
    - microservices/mail/performance-benchmark-numbers-2026-05-20.md (310 lines)
  inventory_files_seen: 208
  inventory_lines_read: 55546
  chat_history_matches_processed: 5
  findings_p0: 0
  findings_p1: 5
  findings_p2: 7
  findings_p3: 2
  tier_retirement_candidates_found: 73; onboarding/mail-engineer-first-week.md:12,27,191,266,294,301; migration-playbooks/from-gmail-workspace.md:30,31,33,136; capability-tiers/tier-matrix.md:15,21,37,58,60,95,97,128,132,134,156,159,173,180,182,183,184,185,188,189,190,192,193; reference-implementations/send-signed-mail-rust-sdk.md:123; benchmarks/gmail-m365-proton-vs-oyatie.md:13,21,22,23,30,36,37,50,51,64,65,66,73,79,80,93,94,102,118; tutorials/promote-dmarc-policy-with-soak.md:16,241; faqs/mail-engineer-faq.md:48,116,120,122,123,127,128,129,135,136,137,138,139,140,153,154,155,180
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/revenue_share semantics under mail, while paid/free/starter/pro/enterprise wording remains in PRD, architecture, ADRs, and manifest
  top_3_counterparts_confirmed: Gmail / Microsoft Outlook / ProtonMail
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1360
-->
