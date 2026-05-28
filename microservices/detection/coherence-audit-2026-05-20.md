---
doc_class: Ownership-Coherence-Audit
microservice: detection
audit_date: 2026-05-20
batch: wave-3-batch-3.2
owner: solo-codex-audit
status: landed
deliverables_expected: 3
deliverables_authored: 3
tier_delta_deliverable: retired-by-2026-05-20-directive
---

# Detection µservice ownership-coherence audit

## Citation anchor block
- Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` §D-15..§D-20, especially D-20.1..D-20.16 and D-20.83..D-20.121.
- Machine-readable plan: `specs/master-plan-sequencing.json` deployment contexts, OpenTofu substrate, supported OS matrix, Rust strict policy, and OCI Always Free profile.
- Brief standard: `docs/standards/brief-template.md` §3.9..§3.12, §5, and §6.
- Constraint memories: `.claude/projects/-Users-jasonlee-oyatie/memory/feedback_*_2026_05_20.md` files named in the batch prompt.
- Chat history: `.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl` line 16274 and line 16339.

## §1 Purpose
1. This audit evaluates whether `microservices/detection/` describes one coherent product surface, one implementable ownership boundary, and one deployable substrate.
2. The target service is `detection`, located at `/Users/jasonlee/oyatie/microservices/detection/`.
3. The batch-defined counterpart bar is Cloudflare Bot Management, Google reCAPTCHA Enterprise, and DataDome.
4. The audit treats the counterpart list as a union-coverage bar, not as a request to narrow Oyatie detection to bot management only.
5. The service currently describes a broad fraud, abuse, safety, AML, graph, and fairness substrate in `PRD.md:29-69`.
6. Cloudflare, reCAPTCHA Enterprise, and DataDome cover only part of that surface: bot scoring, abuse assessment, traffic fingerprinting, token/assessment risk, and mitigation.
7. The service therefore needs either explicit bot/abuse parity slices or a clear statement that bot management is one family inside a wider detection substrate.
8. The current artifact set is documentation-heavy and implementation-light.
9. Existing source and test directories are absent; shell checks returned `src_status=1` and `tests_status=1`.
10. Existing Rust-strict forbidden-language file scan returned no forbidden backend implementation files under the service path.
11. The absence of forbidden code is a pass for file-type hygiene, not a pass for implementation maturity.
12. The audit writes three reports only.
13. The retired fourth tier-deltas deliverable is intentionally not authored.
14. Existing capability-tier artifacts are read as retirement evidence, not as a model to preserve.
15. Findings are severity-coded P0 through P3.
16. P0 means product or canonical contradiction that blocks safe continuation.
17. P1 means required canonical substrate or ownership evidence is missing.
18. P2 means documentation, migration, or retired-language debt that must be scrubbed in Wave 15J or equivalent remediation.
19. P3 means small consistency, polish, or non-blocking clarification debt.
20. No P0 was found.
21. Multiple P1 findings were found around deployable context, OpenTofu, OS support, and implementation evidence.
22. Multiple P2 findings were found around retired tier language and boilerplate documentation.
23. The stop condition for this audit is three landed reports with line floors met, citations present, and no new tier scaffold beyond explicit retirement findings.

## §2 Inventory
24. Inventory source: `find microservices/detection -type f | sort`.
25. Inventory count: 130 files.
26. Existing line count: 12,264 lines.
27. Inventory file 001: `microservices/detection/ARCHITECTURE.md` — 650 lines.
28. Inventory file 002: `microservices/detection/AUDIT-FINDINGS-2026-05-21.json` — 29 lines.
29. Inventory file 003: `microservices/detection/CHANGELOG.md` — 85 lines.
30. Inventory file 004: `microservices/detection/IP-001-streaming-kernel.md` — 75 lines.
31. Inventory file 005: `microservices/detection/IP-002-streaming-worker.md` — 75 lines.
32. Inventory file 006: `microservices/detection/IP-003-batch-kernel.md` — 75 lines.
33. Inventory file 007: `microservices/detection/IP-004-batch-worker.md` — 75 lines.
34. Inventory file 008: `microservices/detection/IP-005-feature-store-domain.md` — 75 lines.
35. Inventory file 009: `microservices/detection/IP-006-feature-store-adapter.md` — 75 lines.
36. Inventory file 010: `microservices/detection/IP-007-rules-engine-kernel.md` — 75 lines.
37. Inventory file 011: `microservices/detection/IP-008-rules-engine-rest.md` — 75 lines.
38. Inventory file 012: `microservices/detection/IP-009-composite-scorer-domain.md` — 75 lines.
39. Inventory file 013: `microservices/detection/IP-010-composite-scorer-worker.md` — 75 lines.
40. Inventory file 014: `microservices/detection/IP-011-graph-store-kernel.md` — 75 lines.
41. Inventory file 015: `microservices/detection/IP-012-community-detection-worker.md` — 75 lines.
42. Inventory file 016: `microservices/detection/IP-013-investigation-bridge-usecase.md` — 75 lines.
43. Inventory file 017: `microservices/detection/IP-014-investigation-rest.md` — 75 lines.
44. Inventory file 018: `microservices/detection/IP-015-sandbox-replay-kernel.md` — 75 lines.
45. Inventory file 019: `microservices/detection/IP-016-sandbox-replay-worker.md` — 75 lines.
46. Inventory file 020: `microservices/detection/IP-017-ml-model-card-registry.md` — 75 lines.
47. Inventory file 021: `microservices/detection/IP-018-drift-detection-daily.md` — 75 lines.
48. Inventory file 022: `microservices/detection/IP-019-fairness-quarterly-audit.md` — 75 lines.
49. Inventory file 023: `microservices/detection/IP-020-appeal-adjudication-flow.md` — 75 lines.
50. Inventory file 024: `microservices/detection/IP-021-openapi-asyncapi-proto-contracts.md` — 75 lines.
51. Inventory file 025: `microservices/detection/IP-022-cedar-policy-pack.md` — 75 lines.
52. Inventory file 026: `microservices/detection/IP-023-slo-dashboard-pack.md` — 75 lines.
53. Inventory file 027: `microservices/detection/IP-024-audit-finding-closeout.md` — 75 lines.
54. Inventory file 028: `microservices/detection/PHASE-01-DETECTION-MVP.md` — 85 lines.
55. Inventory file 029: `microservices/detection/PRD.md` — 1,525 lines.
56. Inventory file 030: `microservices/detection/README.md` — 85 lines.
57. Inventory file 031: `microservices/detection/backfill-replay.md` — 85 lines.
58. Inventory file 032: `microservices/detection/benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md` — 102 lines.
59. Inventory file 033: `microservices/detection/capabilities/batch-pipeline.yaml` — 40 lines.
60. Inventory file 034: `microservices/detection/capabilities/composite-scorer.yaml` — 40 lines.
61. Inventory file 035: `microservices/detection/capabilities/feature-store.yaml` — 40 lines.
62. Inventory file 036: `microservices/detection/capabilities/graph-store-community-detection.yaml` — 40 lines.
63. Inventory file 037: `microservices/detection/capabilities/investigation-bridge.yaml` — 40 lines.
64. Inventory file 038: `microservices/detection/capabilities/rules-engine.yaml` — 40 lines.
65. Inventory file 039: `microservices/detection/capabilities/sandbox-replay.yaml` — 40 lines.
66. Inventory file 040: `microservices/detection/capabilities/streaming-pipeline.yaml` — 40 lines.
67. Inventory file 041: `microservices/detection/capability-tiers/tier-deltas-and-pricing.md` — 349 lines.
68. Inventory file 042: `microservices/detection/capability-tiers/tier-matrix.md` — 131 lines.
69. Inventory file 043: `microservices/detection/capacity-model.md` — 85 lines.
70. Inventory file 044: `microservices/detection/catalog/oya-detection-appeal-mechanism-api.yaml` — 37 lines.
71. Inventory file 045: `microservices/detection/catalog/oya-detection-batch-pipeline-domain.yaml` — 37 lines.
72. Inventory file 046: `microservices/detection/catalog/oya-detection-case-feedback-adapter.yaml` — 37 lines.
73. Inventory file 047: `microservices/detection/catalog/oya-detection-community-detection-adapter.yaml` — 37 lines.
74. Inventory file 048: `microservices/detection/catalog/oya-detection-composite-scorer-rest.yaml` — 37 lines.
75. Inventory file 049: `microservices/detection/catalog/oya-detection-fairness-audit-usecase.yaml` — 37 lines.
76. Inventory file 050: `microservices/detection/catalog/oya-detection-feature-store-usecase.yaml` — 37 lines.
77. Inventory file 051: `microservices/detection/catalog/oya-detection-graph-store-worker.yaml` — 37 lines.
78. Inventory file 052: `microservices/detection/catalog/oya-detection-investigation-bridge-sdk.yaml` — 37 lines.
79. Inventory file 053: `microservices/detection/catalog/oya-detection-ml-lifecycle-domain.yaml` — 37 lines.
80. Inventory file 054: `microservices/detection/catalog/oya-detection-model-registry-rest.yaml` — 37 lines.
81. Inventory file 055: `microservices/detection/catalog/oya-detection-policy-gate-sdk.yaml` — 37 lines.
82. Inventory file 056: `microservices/detection/catalog/oya-detection-replay-ledger-worker.yaml` — 37 lines.
83. Inventory file 057: `microservices/detection/catalog/oya-detection-rules-engine-api.yaml` — 37 lines.
84. Inventory file 058: `microservices/detection/catalog/oya-detection-sandbox-replay-kernel.yaml` — 37 lines.
85. Inventory file 059: `microservices/detection/catalog/oya-detection-streaming-pipeline-kernel.yaml` — 37 lines.
86. Inventory file 060: `microservices/detection/competitor-parity-matrix.md` — 85 lines.
87. Inventory file 061: `microservices/detection/compliance.md` — 530 lines.
88. Inventory file 062: `microservices/detection/contracts/asyncapi-v1.yaml` — 34 lines.
89. Inventory file 063: `microservices/detection/contracts/detection-rule-trait.md` — 120 lines.
90. Inventory file 064: `microservices/detection/contracts/detection-v1.proto` — 36 lines.
91. Inventory file 065: `microservices/detection/contracts/openapi-v1.yaml` — 104 lines.
92. Inventory file 066: `microservices/detection/cost-budget.md` — 85 lines.
93. Inventory file 067: `microservices/detection/dashboards/fairness-and-drift.json` — 38 lines.
94. Inventory file 068: `microservices/detection/dashboards/fairness-and-drift.md` — 70 lines.
95. Inventory file 069: `microservices/detection/dashboards/investigation-throughput.json` — 38 lines.
96. Inventory file 070: `microservices/detection/dashboards/investigation-throughput.md` — 70 lines.
97. Inventory file 071: `microservices/detection/dashboards/operator-signal-health.json` — 38 lines.
98. Inventory file 072: `microservices/detection/dashboards/operator-signal-health.md` — 70 lines.
99. Inventory file 073: `microservices/detection/dashboards/replay-and-rollback.json` — 38 lines.
100. Inventory file 074: `microservices/detection/dashboards/replay-and-rollback.md` — 70 lines.
101. Inventory file 075: `microservices/detection/decisions/ADR-DET-001-streaming-vs-batch-substrate-split.md` — 117 lines.
102. Inventory file 076: `microservices/detection/dpia.md` — 110 lines.
103. Inventory file 077: `microservices/detection/failure-modes.md` — 85 lines.
104. Inventory file 078: `microservices/detection/faqs/detection-engineer-faq.md` — 97 lines.
105. Inventory file 079: `microservices/detection/iac/ech-config.yaml` — 4 lines.
106. Inventory file 080: `microservices/detection/iac/grafana-dashboards.jsonnet` — 1 line.
107. Inventory file 081: `microservices/detection/iac/kubernetes-deployment.yaml` — 6 lines.
108. Inventory file 082: `microservices/detection/iac/kubernetes-service.yaml` — 9 lines.
109. Inventory file 083: `microservices/detection/iac/network-policy.yaml` — 4 lines.
110. Inventory file 084: `microservices/detection/iac/openbao-policy.yaml` — 3 lines.
111. Inventory file 085: `microservices/detection/iac/pqc-cert.yaml` — 3 lines.
112. Inventory file 086: `microservices/detection/iac/secret-bindings.yaml` — 4 lines.
113. Inventory file 087: `microservices/detection/iac/terraform/detection-clickhouse.tf` — 1 line.
114. Inventory file 088: `microservices/detection/iac/terraform/detection-graph-store.tf` — 1 line.
115. Inventory file 089: `microservices/detection/incident-response.md` — 85 lines.
116. Inventory file 090: `microservices/detection/manifest.json` — 196 lines.
117. Inventory file 091: `microservices/detection/migration-playbooks/from-stripe-radar-and-sift.md` — 197 lines.
118. Inventory file 092: `microservices/detection/multi-region.md` — 85 lines.
119. Inventory file 093: `microservices/detection/onboarding/detection-engineer-first-week.md` — 212 lines.
120. Inventory file 094: `microservices/detection/policy/batch-scope.cedar` — 20 lines.
121. Inventory file 095: `microservices/detection/policy/case-pii-access.cedar` — 38 lines.
122. Inventory file 096: `microservices/detection/policy/composite-scope.cedar` — 20 lines.
123. Inventory file 097: `microservices/detection/policy/default-deny.cedar` — 38 lines.
124. Inventory file 098: `microservices/detection/policy/fairness-report-read.cedar` — 38 lines.
125. Inventory file 099: `microservices/detection/policy/feature-scope.cedar` — 20 lines.
126. Inventory file 100: `microservices/detection/policy/graph-investigation-access.cedar` — 38 lines.
127. Inventory file 101: `microservices/detection/policy/graph-scope.cedar` — 20 lines.
128. Inventory file 102: `microservices/detection/policy/investigation-scope.cedar` — 21 lines.
129. Inventory file 103: `microservices/detection/policy/model-deploy.cedar` — 38 lines.
130. Inventory file 104: `microservices/detection/policy/rule-promotion.cedar` — 38 lines.
131. Inventory file 105: `microservices/detection/policy/rules-scope.cedar` — 20 lines.
132. Inventory file 106: `microservices/detection/policy/sandbox-replay-export.cedar` — 38 lines.
133. Inventory file 107: `microservices/detection/policy/sandbox-scope.cedar` — 21 lines.
134. Inventory file 108: `microservices/detection/policy/streaming-scope.cedar` — 20 lines.
135. Inventory file 109: `microservices/detection/policy/tenant-scope.cedar` — 38 lines.
136. Inventory file 110: `microservices/detection/reference-implementations/streaming-score-rust-sdk.md` — 212 lines.
137. Inventory file 111: `microservices/detection/runbooks/batch-backfill-stalled.md` — 273 lines.
138. Inventory file 112: `microservices/detection/runbooks/feature-store-drift.md` — 273 lines.
139. Inventory file 113: `microservices/detection/runbooks/graph-cluster-explosion.md` — 273 lines.
140. Inventory file 114: `microservices/detection/runbooks/investigation-queue-saturation.md` — 273 lines.
141. Inventory file 115: `microservices/detection/runbooks/model-rollback.md` — 273 lines.
142. Inventory file 116: `microservices/detection/runbooks/rule-false-positive-spike.md` — 273 lines.
143. Inventory file 117: `microservices/detection/runbooks/sandbox-replay-mismatch.md` — 273 lines.
144. Inventory file 118: `microservices/detection/runbooks/streaming-pipeline-lag.md` — 273 lines.
145. Inventory file 119: `microservices/detection/scorecards/overrides.json` — 46 lines.
146. Inventory file 120: `microservices/detection/sdk-plan.md` — 85 lines.
147. Inventory file 121: `microservices/detection/slos/batch-pipeline.openslo.yaml` — 38 lines.
148. Inventory file 122: `microservices/detection/slos/composite-scorer.openslo.yaml` — 38 lines.
149. Inventory file 123: `microservices/detection/slos/feature-store.openslo.yaml` — 38 lines.
150. Inventory file 124: `microservices/detection/slos/graph-store-community-detection.openslo.yaml` — 38 lines.
151. Inventory file 125: `microservices/detection/slos/investigation-bridge.openslo.yaml` — 38 lines.
152. Inventory file 126: `microservices/detection/slos/rules-engine.openslo.yaml` — 38 lines.
153. Inventory file 127: `microservices/detection/slos/sandbox-replay.openslo.yaml` — 38 lines.
154. Inventory file 128: `microservices/detection/slos/streaming-pipeline.openslo.yaml` — 38 lines.
155. Inventory file 129: `microservices/detection/threat-model.md` — 110 lines.
156. Inventory file 130: `microservices/detection/tutorials/build-payment-fraud-cedar-rule.md` — 288 lines.

## §2.1 Artifact presence summary
157. Present: PRD, architecture, README, one microservice ADR, 24 implementation plans, contracts, SLOs, policy fragments, dashboards, runbooks, onboarding, tutorial, FAQ, migration playbook, and reference implementation prose.
158. Present but retired-language-bearing: `capability-tiers/` directory and capability-tier benchmark references.
159. Present but non-canonical IaC: generic `iac/` Kubernetes snippets and `iac/terraform/` files.
160. Missing: `microservices/detection/cross-microservice-handoffs.md`, which the batch investigation list expected.
161. Missing: `microservices/detection/supported-oses.json`, required by D-20.83..D-20.101 and master-plan OS policy.
162. Missing: canonical per-context OpenTofu directories for all six deployment contexts.
163. Missing: `iac/oci-guest/always-free/` for the OCI Always Free profile.
164. Missing: `src/` implementation directory.
165. Missing: `tests/` verification directory.
166. Missing: runnable Cargo package for the reference implementation; the prose file itself says the runnable project will land later at `reference-implementations/streaming-example/` in `reference-implementations/streaming-score-rust-sdk.md:212`.

## §3 9-dimensional audit

### §3.1 Dimension 1 — Product purpose and ownership boundary
167. Finding: the product purpose is coherent at the high level.
168. Evidence: `PRD.md:29-35` defines a substrate-level detection service for fraud, abuse, safety, and policy signals.
169. Evidence: `PRD.md:37-69` enumerates eight detection families.
170. Evidence: `manifest.json:46-158` repeats the same eight bounded contexts.
171. Evidence: `ADR-DET-001-streaming-vs-batch-substrate-split.md:17-20` separates streaming and retrospective detection shapes.
172. Interpretation: the service is not a narrow bot-management product; it is a general detection substrate.
173. Impact: counterpart parity must compare bot-abuse surfaces as one family while preserving payment fraud, AML, graph detection, fairness, replay, and investigation commitments.
174. Coherence pass: the ownership boundary is broadly stable around scoring, rules, features, models, graph detection, replay, and case handoff.
175. Coherence gap: the counterpart documents still cite Stripe Radar, AWS GuardDuty, Google Chronicle, and Adyen rather than the batch-defined Cloudflare, reCAPTCHA Enterprise, and DataDome set.
176. Evidence: `README.md:33` names Stripe Radar, Adyen RevenueProtect, AWS GuardDuty, Google Chronicle, NCMEC PhotoDNA, and GIFCT.
177. Evidence: `competitor-parity-matrix.md:33` repeats the old counterpart family.
178. Evidence: `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:9` is titled against Stripe Radar, AWS GuardDuty, Google Chronicle, and Adyen RevenueProtect.
179. Severity: P2 for counterpart refresh, because the core purpose is broader but the batch surface requires a new union-coverage report.

### §3.2 Dimension 2 — Artifact completeness and implementability
180. Finding: artifact breadth is high, but many artifacts are scaffold-heavy.
181. Evidence: `README.md:29-75` repeats the same four bullets across Purpose, Scope, Inputs, Procedure, Metrics, Failure modes, Rollback, and References.
182. Evidence: `IP-001-streaming-kernel.md:35-44` has generic acceptance and verification criteria, followed by repeated buildability notes at `IP-001-streaming-kernel.md:46-75`.
183. Evidence: `capacity-model.md:29-85`, `cost-budget.md:29-85`, `failure-modes.md:29-85`, and `incident-response.md:29-85` repeat the same section text instead of domain-specific numbers.
184. Evidence: `runbooks/streaming-pipeline-lag.md:29-120` repeats the same inspect command sequence across sections and families rather than diagnosing streaming lag specifically.
185. Standard conflict: `docs/standards/brief-template.md` §6 rejects scaffold-without-substance, line-count-as-completion, and clause-loop padding.
186. Impact: a future implementer cannot safely derive exact data models, storage sizes, incident thresholds, or per-context deployment steps from many files.
187. Positive evidence: `ADR-DET-001-streaming-vs-batch-substrate-split.md:46-55` makes a real substrate decision: Flink for streaming and Spark for batch.
188. Positive evidence: `contracts/openapi-v1.yaml:10-49` defines two API operations.
189. Positive evidence: `contracts/detection-v1.proto:6-36` defines gRPC methods and messages.
190. Positive evidence: `policy/*.cedar` files exist, showing an intended policy pack surface.
191. Implementability gap: no source code exists under `src/`, and no test set exists under `tests/`.
192. Severity: P1 for implementation evidence gap, because docs claim runtime behavior without code or tests.

### §3.3 Dimension 3 — Contract, policy, and event coherence
193. Finding: contracts align on the core evaluate-and-replay shape.
194. Evidence: OpenAPI exposes `/signals:evaluate` and `/replay-runs` in `contracts/openapi-v1.yaml:10-49`.
195. Evidence: AsyncAPI exposes `detection.signal.emitted` in `contracts/asyncapi-v1.yaml:5-15`.
196. Evidence: proto exposes `EvaluateSignal` and `StartReplay` in `contracts/detection-v1.proto:6-9`.
197. Evidence: `manifest.json:160-172` points to OpenAPI, AsyncAPI, proto, and trait documents.
198. Gap: contract payloads lack deployment context, tenant class, model version, rule version, decision action, and mitigation outcome fields.
199. Evidence: `contracts/openapi-v1.yaml:52-87` requires tenant, principal, family, entity, trace, and compliance packs only.
200. Evidence: `contracts/detection-v1.proto:11-18` mirrors those fields without context or tenant-class semantics.
201. Impact: the contracts cannot enforce the current batch doctrine for six deployment contexts or three tenant classes.
202. Gap: AsyncAPI emits only score-level fields and omits `deployment_context`, `tenant_class`, `model_version`, `rule_version`, `explanation_version`, and `mitigation_action`.
203. Evidence: `contracts/asyncapi-v1.yaml:21-34` requires only tenant_id, audit_id, family, and score.
204. Severity: P1 for missing canonical control fields in contracts.

### §3.4 Dimension 4 — Canonical-direction alignment
205. Finding: canonical alignment is incomplete across multi-context, OpenTofu, OS support, Rust implementation, OCI Always Free, and tenant-class adoption.
206. Canonical source: ADR-0328 D-15 requires all in-scope µservices to be audited against six contexts.
207. Canonical source: ADR-0328 D-16 requires OpenTofu, not Terraform, Pulumi, CloudFormation, or hand-rolled vendor scripts.
208. Canonical source: ADR-0328 D-20 requires dimensions 6-9 to be covered in every Wave 3+ audit.
209. Canonical source: `specs/master-plan-sequencing.json:704-745` lists all six deployment contexts.
210. Canonical source: `specs/master-plan-sequencing.json:747-775` makes OpenTofu the IaC engine and forbids Terraform, Pulumi, and CloudFormation.
211. Canonical source: `specs/master-plan-sequencing.json:777-815` defines the OS matrix and manifest requirement.
212. Canonical source: `specs/master-plan-sequencing.json:817-856` defines Rust-strict backend and frontend allowlist.
213. Canonical source: `specs/master-plan-sequencing.json:857-867` requires `iac/oci-guest/always-free`.
214. Memory source: `feedback_multi_context_provider_agnostic_2026_05_20.md:10-24` defines cloud, tenant-cloud, and customer-site deployment shapes.
215. Memory source: `feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35` requires OpenTofu-only IaC and per-context modules.
216. Memory source: `feedback_os_support_matrix_2026_05_20.md:10-31` defines the supported OS doctrine.
217. Memory source: `feedback_rust_strict_only_no_python_2026_05_20.md:10-18` and `:38-67` define Rust-strict and allowed non-backend artifacts.
218. Memory source: `feedback_oci_always_free_maximization_2026_05_20.md:10-83` requires OCI Always Free profile maximization.
219. Memory source: `feedback_no_customer_class_ladder_2026_05_20.md:10-45` retires the old customer-class ladder.
220. Memory source: `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-21` records the tenant-class replacement direction, superseded in this batch by the three-class prompt shape.

#### §3.4.1 Multi-context alignment
221. Expected contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.
222. Current service artifact: no canonical context directory is present under `microservices/detection/iac/`.
223. Current `iac/` files are generic: `ech-config.yaml`, `kubernetes-deployment.yaml`, `kubernetes-service.yaml`, `network-policy.yaml`, `openbao-policy.yaml`, `pqc-cert.yaml`, `secret-bindings.yaml`, and two Terraform files.
224. Missing `iac/oyatie-public-cloud`.
225. Missing `iac/guest-on-aws`.
226. Missing `iac/oci-guest`.
227. Missing `iac/on-prem`.
228. Missing `iac/colo`.
229. Missing `iac/oyatie-iaas` or the batch spelling `iac/oyatie-as-cloud-provider`.
230. Manifest gap: `manifest.json` has no `deployment_contexts` array.
231. Contract gap: OpenAPI and proto do not carry deployment context fields.
232. Severity: P1.

#### §3.4.2 OpenTofu alignment
233. Expected substrate: OpenTofu.
234. Current violation: `microservices/detection/iac/terraform/detection-clickhouse.tf` exists.
235. Current violation: `microservices/detection/iac/terraform/detection-graph-store.tf` exists.
236. Current gap: no `versions.tf`, `providers.tf`, `main.tf`, `variables.tf`, `outputs.tf`, `policy.rego`, `module.signature`, or `README.md` per context.
237. Current gap: no `tofu.lock.hcl` or equivalent module provenance evidence.
238. Current gap: no context-specific variables for region, tenancy, cell, OS, and OCI Always Free profile.
239. Severity: P1 because ADR-0328 D-16 makes Terraform spelling and missing context modules canonical violations.

#### §3.4.3 OS support alignment
240. Expected: supported OS manifest plus build/test/install evidence across the canonical matrix.
241. Current check: `microservices/detection/supported-oses.json` is absent.
242. Current source check: no Rust crate exists to compile against the OS matrix.
243. Current docs check: runbooks and onboarding describe CLI commands but do not enumerate OS-specific packages, services, installers, or smoke tests.
244. Severity: P1 because the OS manifest is explicitly required by D-20 and the master plan.

#### §3.4.4 Rust-strict alignment
245. Forbidden-language file scan for Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, and F# returned no files.
246. Positive: no forbidden backend implementation file is present under `microservices/detection/`.
247. Gap: there is no Rust `Cargo.toml`, crate, `src/`, or test set.
248. Reference-only evidence: `reference-implementations/streaming-score-rust-sdk.md:13-35` contains a Cargo snippet and dependency plan.
249. Reference-only evidence: `reference-implementations/streaming-score-rust-sdk.md:36-201` contains Rust example prose/code, not a runnable crate in the repository.
250. Reference-only caveat: `reference-implementations/streaming-score-rust-sdk.md:212` says the runnable Cargo project will land later.
251. Severity: P1 for missing runnable Rust implementation; P3 positive note for absence of forbidden file types.

#### §3.4.5 OCI Always Free alignment
252. Expected path: `microservices/detection/iac/oci-guest/always-free/`.
253. Current check: the path is absent.
254. Expected capacity overlay: 4 OCPU, 24 GB RAM, free storage and network budgets from ADR-0328 D-19 and OCI memory.
255. Current docs: `capacity-model.md` and `cost-budget.md` are generic repeated scaffolds and do not express the OCI Always Free profile.
256. Current benchmark: old benchmark uses an enterprise-scale multi-AZ hardware shape, not demo-trial infrastructure caps.
257. Severity: P1.

#### §3.4.T Tier retirement candidates
258. Retirement rule: references to retired customer-class names are Wave 15J retirement candidates unless they are in this audit section as evidence.
259. Total uppercase capability-tier candidate lines found: 154.
260. Candidate source family 1: dedicated retired directory `microservices/detection/capability-tiers/`.
261. Candidate source family 2: benchmark rows using old named capability tiers.
262. Candidate source family 3: tutorials and reference examples that require a named capability tier.
263. Candidate `reference-implementations/streaming-score-rust-sdk.md:203` — demo_trial condition in model-card explanation.
264. Candidate `tutorials/build-payment-fraud-cedar-rule.md:15` — paid tenant_class prerequisite.
265. Candidate `capability-tiers/tier-matrix.md:13` — demo_trial heading.
266. Candidate `capability-tiers/tier-matrix.md:18` — demo_trial rules-only batch statement.
267. Candidate `capability-tiers/tier-matrix.md:20` — demo_trial graph-store absence statement.
268. Candidate `capability-tiers/tier-matrix.md:27` — demo_trial family wiring statement.
269. Candidate `capability-tiers/tier-matrix.md:43` — paid dedicated-cloud heading.
270. Candidate `capability-tiers/tier-matrix.md:45` — paid dedicated-cloud adds-to statement.
271. Candidate `capability-tiers/tier-matrix.md:70` — paid on-prem-connected heading.
272. Candidate `capability-tiers/tier-matrix.md:72` — paid on-prem-connected adds-to statement.
273. Candidate `capability-tiers/tier-matrix.md:96` — paid dedicated-cloud cost comparison.
274. Candidate `capability-tiers/tier-matrix.md:100` — paid compliance_pack heading.
275. Candidate `capability-tiers/tier-matrix.md:102` — paid compliance_pack adds-to statement.
276. Candidate `capability-tiers/tier-matrix.md:115` — paid on-prem-connected SLO reference.
277. Candidate `capability-tiers/tier-matrix.md:123` — demo_trial and paid dedicated-cloud model-card depth.
278. Candidate `capability-tiers/tier-matrix.md:129` — promotion path across retired customer-class names.
279. Candidate `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:13` — paid on-prem-connected hardware statement.
280. Candidate `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:21` — paid dedicated-cloud latency row.
281. Candidate `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:22` — paid on-prem-connected latency row.
282. Candidate `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:50` — paid on-prem-connected graph benchmark row.
283. Candidate `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:75` — paid on-prem-connected TCO row.
284. Candidate `capability-tiers/tier-deltas-and-pricing.md:18-25` — old four-level summary.
285. Candidate `capability-tiers/tier-deltas-and-pricing.md:36-43` — old optimization and heading block.
286. Candidate `capability-tiers/tier-deltas-and-pricing.md:47-54` — old bottom-level capacity block.
287. Candidate `capability-tiers/tier-deltas-and-pricing.md:83-87` — old behavior and mid-level heading block.
288. Candidate `capability-tiers/tier-deltas-and-pricing.md:91-96` — old mid-level compute block.
289. Candidate `capability-tiers/tier-deltas-and-pricing.md:123-129` — old graph absence and enterprise heading block.
290. Candidate `capability-tiers/tier-deltas-and-pricing.md:133-136` — old enterprise compute block.
291. Candidate `capability-tiers/tier-deltas-and-pricing.md:169-176` — old enterprise and sovereign behavior block.
292. Candidate `capability-tiers/tier-deltas-and-pricing.md:208-227` — old price-band block.
293. Candidate `capability-tiers/tier-deltas-and-pricing.md:235-262` — old customer-archetype block.
294. Candidate `capability-tiers/tier-deltas-and-pricing.md:268-295` — old failure-mode and cross-tier block.
295. Candidate `capability-tiers/tier-deltas-and-pricing.md:299-328` — old promotion and minimum-level block.
296. Generic tier language also appears outside the four retired words.
297. Generic candidate: `manifest.json:5` uses `"tier": "substrate"`.
298. Generic candidate: `manifest.json:20-34` uses `cell_tier` values `tier-0` through `tier-3`.
299. Generic candidate: `PRD.md:840`, `:851`, `:862`, `:873`, `:884`, `:895`, `:906`, and `:917` use "active safe tier".
300. Generic candidate: `ARCHITECTURE.md:200`, `:219`, `:238`, `:257`, `:276`, `:295`, `:314`, and `:333` use "prior active rule/model tier".
301. Non-retirement semantic use: `PRD.md:693`, `PRD.md:778`, `manifest.json:78`, and `manifest.json:148` use "online tier" or "cold tier" as storage topology terms, not capability pricing.
302. Default severity: P2 for all retired capability-tier candidates.

#### §3.4.C Tenant-class adoption gaps
303. Required replacement model in this batch: `demo_trial`, `paid`, and `revenue_share`.
304. Search result: no `tenant_class` field appears under `microservices/detection/`.
305. Search result: no `demo_trial` string appears under `microservices/detection/`.
306. Search result: no `revenue_share` string appears under `microservices/detection/`.
307. Search result: no current tenant-class schema appears in the OpenAPI, proto, AsyncAPI, manifest, SLOs, policy, runbooks, or benchmarks.
308. Partial billing language: old `capability-tiers/tier-deltas-and-pricing.md:126` says paid dedicated-cloud is default for paying tenants, but this is retired capability-tier language and does not implement the replacement model.
309. Gap: `contracts/openapi-v1.yaml:52-87` must add tenant-class semantics without lowering the quality bar.
310. Gap: `contracts/detection-v1.proto:11-18` must carry tenant class or derive it from an authoritative tenancy service.
311. Gap: SLOs and runbooks must distinguish demo-trial usage caps from paid contractual SLOs and revenue-share at-cost substrate.
312. Severity: P2 if limited to docs; P1 before runtime admission because usage caps and billing posture are enforcement-relevant.

### §3.5 Dimension 5 — Counterpart product-surface alignment
313. Cloudflare Bot Management surface: bot score, verified bots, JavaScript detections, bot detection IDs, JA3/JA4, WAF rule fields, logs, and edge bot mitigation.
314. Source: Cloudflare docs say Bot Score is an integer 1-99 and expose verified bot, static resource, JA3/JA4, detection IDs, signed agent, and log fields.
315. Google reCAPTCHA Enterprise surface: backend assessments, risk scores, action verification, token validity, reason codes, account defense, SMS defense, and fraud prevention.
316. Source: Google docs define assessment score 0.0-1.0, 11 score levels, token one-use/two-minute expiry, quota 60,000 requests per minute, and 10,000 free monthly assessments without billing.
317. DataDome surface: edge bot protection, 35+ edge PoPs, under-2-ms mitigation claim, 5 trillion signals per day, 1000+ models, 80+ integrations, and false-positive claim below 0.01%.
318. Current detection parity docs do not cover these specific surfaces.
319. Current competitor parity matrix is generic and repeats old vendor names at `competitor-parity-matrix.md:29-85`.
320. Current benchmark doc compares against Stripe Radar, Adyen, GuardDuty, Chronicle, Sift, Featurespace, feature stores, graph stores, and fairness libraries at `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:17-83`.
321. Gap: no current artifact maps bot score, verified bot allowlisting, CAPTCHA alternatives, challenge outcomes, client fingerprints, action tokens, assessment reason codes, or edge PoP latency.
322. Severity: P2 because the service is broader, but the batch-defined counterpart report must be refreshed.

### §3.6 Dimension 6 — Six deployment contexts
323. Required by ADR-0328 D-15 and D-20.
324. Current evidence: there are only generic IaC files and `iac/terraform`.
325. The service does not encode `oyatie-public-cloud`.
326. The service does not encode `guest-on-aws`.
327. The service does not encode `guest-on-oci`.
328. The service does not encode `on-prem`.
329. The service does not encode `colo`.
330. The service does not encode `oyatie-as-cloud-provider`.
331. The architecture claims multi-AZ and cross-region behavior in places, but not as six context-specific deployable modules.
332. Evidence: `ADR-DET-001-streaming-vs-batch-substrate-split.md:48-55` names compute substrates but not deployment contexts.
333. Evidence: `multi-region.md:29-85` is generic repeated reference text rather than deployable context design.
334. Severity: P1.

### §3.7 Dimension 7 — OpenTofu IaC
335. Required by ADR-0328 D-16 and master plan.
336. Present files are too small to represent real modules.
337. `iac/kubernetes-deployment.yaml` is 6 lines.
338. `iac/kubernetes-service.yaml` is 9 lines.
339. `iac/network-policy.yaml` is 4 lines.
340. `iac/openbao-policy.yaml` is 3 lines.
341. `iac/secret-bindings.yaml` is 4 lines.
342. `iac/terraform/detection-clickhouse.tf` is 1 line.
343. `iac/terraform/detection-graph-store.tf` is 1 line.
344. These are not valid context modules with state, provider, variables, outputs, signing, and plan/apply evidence.
345. Severity: P1.

### §3.8 Dimension 8 — OS support
346. Required by ADR-0328 D-17 and D-20.
347. Missing file: `supported-oses.json`.
348. Missing evidence: install scripts or package manifests for Linux, BSD, macOS, Windows, illumos, Solaris, and AIX support classes.
349. Missing evidence: architecture-specific build/test matrix for x86_64, arm64, riscv64, POWER, and s390x where required.
350. Missing evidence: OS-specific runtime dependency handling for Flink, Spark, ClickHouse, JanusGraph, ScyllaDB, or Cedar.
351. Risk: ADR-DET-001 introduces JVM dependency complexity at `ADR-DET-001-streaming-vs-batch-substrate-split.md:100-102`.
352. Risk: no OS plan resolves Java 17 and Java 21 runtime split across the support matrix.
353. Severity: P1.

### §3.9 Dimension 9 — Rust-strict implementation and forbidden language scan
354. The scan found no forbidden language files.
355. The service includes Markdown, YAML, JSON, proto, OpenSLO, Cedar, Jsonnet, Terraform, and prose Rust snippets.
356. Jsonnet is not disallowed by the batch prompt but should be checked against canonical substrate policy if it becomes runtime IaC.
357. Terraform files are prohibited by OpenTofu policy even though `.tf` extension is technically shared with OpenTofu.
358. The reference Rust SDK is prose-only, not a compiled crate.
359. `reference-implementations/streaming-score-rust-sdk.md:13-35` gives a `Cargo.toml` example.
360. `reference-implementations/streaming-score-rust-sdk.md:36-201` gives code blocks and error examples.
361. `reference-implementations/streaming-score-rust-sdk.md:212` admits the runnable project is future work.
362. Severity: P1 for implementation absence; P2 for examples that claim SDK shape before the crate exists.

## §4 Findings table
363. F-DET-001 | P1 | No six-context deployable IaC | Evidence: missing context directories; `iac/` only has generic files and `iac/terraform/*`; canonical: `specs/master-plan-sequencing.json:704-745`.
364. F-DET-002 | P1 | OpenTofu doctrine violated by Terraform path | Evidence: `iac/terraform/detection-clickhouse.tf`, `iac/terraform/detection-graph-store.tf`; canonical: `feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35`.
365. F-DET-003 | P1 | OCI Always Free profile absent | Evidence: no `iac/oci-guest/always-free/`; canonical: `specs/master-plan-sequencing.json:857-867`.
366. F-DET-004 | P1 | OS support manifest absent | Evidence: `supported_oses_status=1`; canonical: `feedback_os_support_matrix_2026_05_20.md:10-31`.
367. F-DET-005 | P1 | Runtime source and tests absent | Evidence: `src_status=1`, `tests_status=1`; existing audit JSON calls runtime crates future work at `AUDIT-FINDINGS-2026-05-21.json:21-27`.
368. F-DET-006 | P1 | Contracts lack deployment context and tenant class | Evidence: `contracts/openapi-v1.yaml:52-87`, `contracts/detection-v1.proto:11-18`, `contracts/asyncapi-v1.yaml:21-34`.
369. F-DET-007 | P1 | Implementation plans are generic and not build-sufficient | Evidence: `IP-001-streaming-kernel.md:35-75`; same pattern appears across IP-001..IP-024.
370. F-DET-008 | P1 | Runbooks are repetitive rather than incident-specific | Evidence: `runbooks/streaming-pipeline-lag.md:29-120`.
371. F-DET-009 | P2 | Retired customer-class ladder persisted after retirement directive | Evidence: deleted retired matrix and pricing files; canonical: `feedback_no_customer_class_ladder_2026_05_20.md:10-45`.
372. F-DET-010 | P2 | Tenant-class replacement model absent | Evidence: no `tenant_class`, `demo_trial`, or `revenue_share` matches under the service path.
373. F-DET-011 | P2 | Counterpart set stale for this batch | Evidence: `README.md:33`, `competitor-parity-matrix.md:33`, `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:9`.
374. F-DET-012 | P2 | Old benchmark uses retired tier rows | Evidence: `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:13`, `:21-22`, `:50`, `:75`.
375. F-DET-013 | P2 | Pricing and customer archetypes are tied to retired capability levels | Evidence: `capability-tiers/tier-deltas-and-pricing.md:213-262`.
376. F-DET-014 | P2 | Manifest uses old cell-tier model | Evidence: `manifest.json:20-34`.
377. F-DET-015 | P2 | Cross-microservice handoff doc missing | Evidence: inventory contains no `cross-microservice-handoffs.md`; required in batch investigation list.
378. F-DET-016 | P2 | Capacity and cost docs are scaffolded | Evidence: `capacity-model.md:29-85`, `cost-budget.md:29-85`.
379. F-DET-017 | P2 | Data protection impact assessment is scaffold-heavy | Evidence: `dpia.md:29-110`.
380. F-DET-018 | P2 | Threat model is scaffold-heavy | Evidence: `threat-model.md:29-110`.
381. F-DET-019 | P2 | Compliance matrix has repeated clauses across unrelated anchors | Evidence: `compliance.md:35-180`.
382. F-DET-020 | P2 | Existing audit JSON is scaffold status, not closure evidence | Evidence: `AUDIT-FINDINGS-2026-05-21.json:5-27`.
383. F-DET-021 | P3 | Product scope is broad enough to exceed current counterpart set | Evidence: `PRD.md:37-69`; action: make bot-management parity one explicit family.
384. F-DET-022 | P3 | Architecture has good substrate rationale but not context-specific deployment shape | Evidence: `ADR-DET-001-streaming-vs-batch-substrate-split.md:46-55`.
385. F-DET-023 | P3 | Forbidden-language scan is clean | Evidence: no matches for forbidden backend implementation file patterns.
386. F-DET-024 | P3 | Contract version alignment is internally consistent | Evidence: OpenAPI 3.2.0 at `contracts/openapi-v1.yaml:1`, AsyncAPI 3.1.0 at `contracts/asyncapi-v1.yaml:1`, proto3 at `contracts/detection-v1.proto:1`.
387. F-DET-025 | P2 | Reference SDK is not yet runnable | Evidence: `reference-implementations/streaming-score-rust-sdk.md:212`.
388. F-DET-026 | P2 | Tutorial relies on retired capability prerequisite | Evidence: `tutorials/build-payment-fraud-cedar-rule.md:15`.
389. F-DET-027 | P2 | Reference SDK error table relies on retired capability model | Evidence: `reference-implementations/streaming-score-rust-sdk.md:203`.
390. F-DET-028 | P1 | JVM substrate split lacks OS/package plan | Evidence: Java 17 and Java 21 split at `ADR-DET-001-streaming-vs-batch-substrate-split.md:100-102`; no supported OS manifest.
391. F-DET-029 | P2 | External chat history confirms detection was part of a Wave 4 rolling queue, not a finished final audit | Evidence: chat history line 16274 and line 16339.
392. F-DET-030 | P2 | Existing benchmark harness path is not present in inventory | Evidence: benchmark doc cites `benchmarks/detectionbench/` at `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:91-102`; inventory has no such directory.

## §5 Open questions
393. OQ-001: Should `detection` remain one broad fraud/abuse/safety substrate, or should bot-management become a separate deployable service?
394. OQ-002: If it remains broad, which bounded context owns Cloudflare/DataDome-style edge bot scoring: rules engine, composite scorer, or a new bot-signal adapter?
395. OQ-003: Should reCAPTCHA-like token assessment be implemented as an inbound SDK contract, an edge adapter, or a policy-engine handoff?
396. OQ-004: Which service owns verified-bot allowlists and signed-agent verification: detection, identity, policy-engine, or network?
397. OQ-005: Which store is authoritative for tenant class: tenancy manifest, billing ledger, policy-engine context, or detection request field?
398. OQ-006: What is the exact demo-trial usage cap for the OCI Always Free profile: requests per minute, assessments per month, stored signals, or all of those?
399. OQ-007: Does revenue-share run on the same infrastructure envelope as paid tenants with billing-margin differences, or does it require at-cost resource admission?
400. OQ-008: Should deprecated capability-tier files be deleted, archived, or rewritten into tenant-class overlays during Wave 15J?
401. OQ-009: Should the old Stripe/GuardDuty/Chronicle benchmark remain as a secondary broader-risk benchmark after the new Cloudflare/reCAPTCHA/DataDome report lands?
402. OQ-010: What is the minimum runnable implementation slice: synchronous score API, streaming pipeline, Cedar rule path, or replay path?

## §6 Evidence notes
403. The chat history read requirement was satisfied by searching the specified JSONL for `detection`.
404. Relevant line 16274 records that Wave 4 included `intelligence / ontology / workflow-engine / workflow-studio / consent-graph / detection` and recommended a foundry-absorption dimension.
405. Relevant line 16339 records `detection` as a rolling dispatcher current PID with log path `/tmp/rolling-codex-detection.log`.
406. The current audit does not rely on previous agents' deliverables as accepted truth.
407. The current audit reads local artifacts and canonical sources directly.
408. The current audit uses the user-provided 2026-05-20 tenant-class directive as the controlling replacement model.
409. The `.claude` tenant-class memory still records a two-class enum plus paid billing components, but the current batch directive declares three audit classes.
410. Therefore this report treats `demo_trial`, `paid`, and `revenue_share` as the operative audit shape.
411. The absence of `tenant_class` in the service path means no local conflict exists yet.
412. The absence of forbidden backend language files means the Rust-strict scan did not identify foreign implementation files.
413. The absence of Rust implementation means Rust-strict completion cannot be claimed.
414. The absence of OpenTofu modules is a stronger finding than the presence of small generic Kubernetes snippets.
415. The presence of Terraform-path files is a direct naming and substrate violation under the current canonical policy.
416. The old benchmark numbers are not reused as current targets because they are tied to retired capability levels.
417. The new performance report uses a single industry-leader-grade target set with deployment-context and tenant-class overlays.

## §7 Recommended remediation sequence
418. Step 1: retire or quarantine `capability-tiers/` and remove old four-level terms from benchmark, tutorial, reference SDK, and manifest.
419. Step 2: add `tenant_class` or authoritative tenant-class derivation to contracts, policy context, SLO overlays, and admission controls.
420. Step 3: replace `iac/terraform/` with canonical OpenTofu modules under all six context paths.
421. Step 4: add `iac/oci-guest/always-free/` with explicit demo-trial capacity and usage caps.
422. Step 5: add `supported-oses.json` and map Flink, Spark, ClickHouse, graph store, Cedar, and Rust runtime constraints per OS.
423. Step 6: land the smallest runnable Rust slice and tests for synchronous evaluation.
424. Step 7: update contracts with deployment context, tenant class, rule/model versions, action outcome, mitigation, and evidence IDs.
425. Step 8: rewrite scaffold-heavy operational docs into domain-specific runbooks, capacity plans, cost budgets, and DPIA/threat-model content.
426. Step 9: add bot-management counterpart coverage to the product artifacts without deleting broader fraud/abuse/security scope.
427. Step 10: verify every remediation with fresh line-level evidence before closing the audit findings.

## §8 Completion assessment
428. The service is directionally coherent as a detection substrate.
429. It is not yet deployable in the canonical six-context sense.
430. It is not yet compliant with OpenTofu-only IaC doctrine.
431. It is not yet compliant with the OS-support manifest doctrine.
432. It is not yet compliant with OCI Always Free profile doctrine.
433. It has no local adoption of the current tenant-class replacement model.
434. It contains substantial retired capability-tier material.
435. It contains many scaffold-heavy docs that meet line counts but not operational depth.
436. It has useful early anchors in ADR-DET-001, contracts, policy fragments, and SLO shells.
437. The highest-value next slice is not another prose expansion; it is a control-surface correction across manifest, contracts, OpenTofu layout, and tenant-class semantics.
438. A second high-value slice is a runnable Rust synchronous evaluation path with contract tests.
439. A third high-value slice is replacement of old counterpart docs with Cloudflare/reCAPTCHA/DataDome parity.

## §9 Closure-grade remediation register
440. R-001: Establish `tenant_class` as an explicit contract input or derived authorization context for detection requests; evidence gap is absence of local `tenant_class` matches across the service path and current contract schemas at `contracts/openapi-v1.yaml:52-104`.
441. R-002: Add `demo_trial`, `paid`, and `revenue_share` handling to admission policy or entitlement policy; controlling doctrine is the 2026-05-20 replacement model in the current batch directive and local absence is confirmed by the tenant-class scan.
442. R-003: Define demo-trial usage caps in service-owned docs and machine-readable policy; canonical OCI Always Free doctrine requires a free-trial infrastructure profile, while no `iac/oci-guest/always-free/` path exists.
443. R-004: Define paid tenant scaling semantics without feature-quality downgrades; the current service has old capability-level semantics but no current paid tenant-class model.
444. R-005: Define revenue-share tenant economics and at-cost substrate handling; current cost docs do not express revenue-share, and the replacement model requires it for marketplace, B2C, reseller, and affiliate operators.
445. R-006: Replace the `capability-tiers/` directory with retired-material handling; uppercase candidate evidence includes `capability-tiers/tier-matrix.md:13` and `capability-tiers/tier-deltas-and-pricing.md:18-25`.
446. R-007: Retire old level references from the benchmark doc before any benchmark target is reused; evidence is the legacy workload and row language at `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:11-22`.
447. R-008: Retire old level references from the tutorial before user-facing learning material is treated as current; evidence is `tutorials/build-payment-fraud-cedar-rule.md:15`.
448. R-009: Retire old level references from the reference SDK example; evidence is `reference-implementations/streaming-score-rust-sdk.md:203`.
449. R-010: Clarify `manifest.json` use of `tier` and `cell_tier`; evidence is `manifest.json:5` and `manifest.json:20-34`, and the service needs a non-conflicting resource-class or cell-class vocabulary.
450. R-011: Replace `iac/terraform/` with canonical OpenTofu naming and modules; evidence is `iac/terraform/detection-clickhouse.tf` and `iac/terraform/detection-graph-store.tf`.
451. R-012: Create `iac/oyatie-public-cloud/` with service-specific OpenTofu modules and variables; evidence gap is no such directory in the 130-file inventory.
452. R-013: Create `iac/guest-on-aws/` with provider-agnostic module boundaries; evidence gap is no such directory in the 130-file inventory.
453. R-014: Create `iac/oci-guest/` with an `always-free/` profile; evidence gap is no such directory and canonical OCI Always Free doctrine requires the explicit profile.
454. R-015: Create `iac/on-prem/` with facility-variable inputs; evidence gap is no such directory and all-context coverage is required by ADR-0328 §D-15.
455. R-016: Create `iac/colo/` with capacity and network assumptions; evidence gap is no such directory and colocation is one of the six canonical contexts.
456. R-017: Create `iac/oyatie-as-cloud-provider/` or the canonical renamed equivalent used by the repo; evidence gap is no service-owned context module for provider-mode deployment.
457. R-018: Add service-owned OpenTofu test or validation hooks once modules exist; current files are raw Kubernetes and generic config snippets, not deployable all-context IaC.
458. R-019: Add `supported-oses.json`; current scan confirms it is absent and canonical OS doctrine requires a machine-readable support matrix.
459. R-020: Map Linux, BSD, Illumos, macOS, and Windows support for the Rust evaluator path; current local docs do not bind runtime support to the master OS matrix.
460. R-021: Map Flink 1.21 and Spark 4.0 constraints separately from the Rust path; evidence is ADR-DET-001 decision lines `docs/decisions/ADR-DET-001-streaming-and-batch-detection-runtime.md:46-55`.
461. R-022: Map ClickHouse constraints and alternatives by deployment context; evidence is the planned ClickHouse IaC file under the forbidden Terraform path and the lack of context modules.
462. R-023: Map graph-store constraints by deployment context; evidence is `iac/terraform/detection-graph-store.tf` and graph-family requirements in `PRD.md:742-757`.
463. R-024: Add a runnable Rust crate or workspace member for synchronous evaluation; current `src_status=1` evidence means no local `src/` exists.
464. R-025: Add contract tests for `/signals:evaluate`; evidence is the declared OpenAPI endpoint at `contracts/openapi-v1.yaml:10-31` and absence of a `tests/` directory.
465. R-026: Add protobuf compatibility tests for `Evaluate` and `Replay`; evidence is service method declarations at `contracts/detection-v1.proto:6-9`.
466. R-027: Add AsyncAPI event conformance tests for `detection.signal.emitted`; evidence is event declaration at `contracts/asyncapi-v1.yaml:5-15`.
467. R-028: Add score-envelope tests for severity and explanation fields; evidence is response schema at `contracts/openapi-v1.yaml:88-104`.
468. R-029: Extend API contracts with action outcome and mitigation fields; current contract response exposes score, severity, explanation, and audit ID but not decision action or mitigation.
469. R-030: Extend contracts with rule version, model version, feature snapshot ID, and policy bundle ID; current schema lacks those audit-grade reproducibility anchors.
470. R-031: Extend replay APIs with deterministic replay input IDs and output diff semantics; current `ReplayRun` schema is too thin for forensic reproducibility.
471. R-032: Add counterparty-specific bot and abuse signals to the PRD; current product scope is broad, while the assigned union bar is Cloudflare Bot Management, Google reCAPTCHA Enterprise, and DataDome.
472. R-033: Add Cloudflare-equivalent device, behavior, JavaScript, ML, and heuristics coverage to the feature model; external source used in the parity report is Cloudflare bot detection engine documentation.
473. R-034: Add reCAPTCHA-equivalent assessment, token, account defender, annotation, and fraud-prevention workflows; external source used in the parity report is Google reCAPTCHA Enterprise documentation.
474. R-035: Add DataDome-equivalent endpoint coverage, device fingerprinting, mobile app protection, CAPTCHA orchestration, and attack analytics; external source used in the parity report is DataDome product documentation.
475. R-036: Rewrite `capacity-model.md` with service-specific ingest, scoring, replay, feature-store, graph, and investigation budgets; current audit classifies it as scaffold-heavy.
476. R-037: Rewrite `cost-budget.md` with context and tenant-class overlays; current cost docs do not bind usage caps, revenue share, or Always Free profile economics.
477. R-038: Rewrite `failure-modes.md` around detection-specific cases such as score drift, model rollback, feature staleness, rule misfire, event lag, and graph-store partitioning.
478. R-039: Rewrite `incident-response.md` around detection incidents, not generic service incidents; current incident material does not prove abuse-specific operational readiness.
479. R-040: Rewrite `dpia.md` around bot, fraud, identity, behavioral, device, and marketplace signals; broad personal-data handling exists but needs service-specific lawful-basis and minimization claims.
480. R-041: Rewrite `compliance.md` around compliance packs already named in `manifest.json:174-186`; current artifacts list packs but do not operationalize detection controls per pack.
481. R-042: Add domain-specific runbooks for false positives, false negatives, model drift, rules rollback, CAPTCHA degradation, replay backfill, and stream lag; current runbook coverage is narrow and repetitive.
482. R-043: Replace the old counterpart set in `README.md` and `competitor-parity-matrix.md`; evidence is old comparator language near `README.md:33` and `competitor-parity-matrix.md:33`.
483. R-044: Keep the broader fraud and abuse mission but explicitly add bot-management parity; evidence is `PRD.md:29-69`, which supports broad detection but not assigned counterpart union completeness.
484. R-045: Convert repeated buildability notes in `PRD.md:1003-1525` into concrete implementation checkpoints, or remove them from the product contract.
485. R-046: Convert repeated implementation-plan boilerplate in IP-001 through IP-024 into slice-specific acceptance criteria; IP-001 evidence is `implementation-plans/IP-001-signal-intake-and-normalization.md:35-75`.
486. R-047: Promote useful ADR-DET-001 decisions into runtime and IaC requirements; evidence is the chosen Flink and Spark combination at `ADR-DET-001:46-55`.
487. R-048: Review Java runtime implications in ADR-DET-001 against the Rust-strict backend doctrine; evidence is `ADR-DET-001:100-102`, which mentions Java constraints for Spark and code generation.
488. R-049: Add a language-policy note distinguishing managed JVM dependencies from service-owned backend implementation code; current scan finds no forbidden local source files, but runtime decisions still need doctrine alignment.
489. R-050: Add per-context SLO overlays; current SLO files exist but do not prove context-specific latency, durability, replay, or cap behavior.
490. R-051: Add demo-trial SLO disclosure as best-effort with hard usage caps; this is required by the current tenant-class directive and absent locally.
491. R-052: Add paid tenant SLO disclosure as contractual and scalable with paid usage; this is required by the current tenant-class directive and absent locally.
492. R-053: Add revenue-share SLO and cost-floor disclosure; this is required by the current tenant-class directive and absent locally.
493. R-054: Add a service-owned inventory manifest that separates authoritative docs from retired material; current inventory mixes live docs, retired capability-level docs, and repeated scaffold.
494. R-055: Add a retirement marker or ADR link to old capability-level files until Wave 15J deletes or rewrites them; current files read as active docs.
495. R-056: Do not close the audit by line count; close it only after the five cross-cutting constraints have machine-readable evidence in the service path.
496. R-057: Require all future findings to carry file:line evidence; this audit uses line evidence from canonical sources, local artifacts, memory files, chat history, and external docs.
497. R-058: Require all future benchmark claims to include methodology, workload, OS, architecture, context, tenant class, and source; the new performance document follows that structure.
498. R-059: Require local implementation proof before claiming Rust-strict compliance; absence of forbidden source files is only a non-violation, not completion.
499. R-060: Require OpenTofu validation before claiming deployability; generic Kubernetes YAML does not satisfy the canonical IaC substrate.
500. R-061: Require six-context path evidence before claiming all-context coverage; current deployable context assumption remains unproven by file layout.
501. R-062: Require OCI Always Free proof before claiming demo-trial readiness; current service has no profile directory.
502. R-063: Require OS manifest proof before claiming broad OS support; current service has no supported-oses manifest.
503. R-064: Require contract proof before claiming tenant-class readiness; current contracts have tenant IDs but not tenant classes.
504. R-065: Require policy proof before claiming entitlement readiness; current policy fragments do not express the three tenant classes.
505. R-066: Require cost proof before claiming revenue-share readiness; current cost docs do not encode gross-revenue-share economics.
506. R-067: Require runbook proof before claiming operations readiness; current runbooks do not cover enough detection-specific failure modes.
507. R-068: Require benchmark harness proof before claiming performance readiness; current old benchmark references nonexistent or unlanded harness paths.
508. R-069: Require counterpart proof before claiming market parity; current old comparator set is not the assigned union-coverage bar.
509. R-070: Require replay determinism proof before claiming investigation readiness; current replay artifacts do not specify deterministic diff behavior.
510. R-071: Require audit-log immutability proof before claiming compliance readiness; current contracts expose `audit_id` but do not define retention, integrity, or retrieval guarantees.
511. R-072: Require model-governance proof before claiming ML readiness; current PRD names model and rules families but lacks local model lifecycle artifacts.
512. R-073: Require rule-governance proof before claiming Cedar readiness; current policy needs versioning, testing, rollback, and explainability anchors.
513. R-074: Require feature-store freshness proof before claiming low-latency scoring; current PRD names feature store goals but no implementation or tests exist.
514. R-075: Require graph-store blast-radius proof before claiming account-linkage coverage; current graph-store planning is not tied to all contexts.
515. R-076: Require privacy minimization proof before ingesting device or behavior signals; counterpart parity will increase sensitive-signal scope.
516. R-077: Require abuse-safety proof for CAPTCHA or challenge orchestration; the current service does not specify challenge outcome contracts.
517. R-078: Require marketplace fraud proof for revenue-share contexts; the current service does not encode seller, affiliate, reseller, or B2C revenue-share shapes.
518. R-079: Require migration proof for existing tenant configs when old capability-level docs are retired; current docs do not provide a migration playbook for that semantic change.
519. R-080: Require a doc ownership rule for generated-looking buildability sections; current repeated blocks obscure true implementation status.
520. R-081: Final closure condition for Dimension 1 is a current PRD/architecture/contract set that explains the same product boundary with no old comparator mismatch.
521. R-082: Final closure condition for Dimension 2 is complete inventory tagging of active, retired, scaffold, and executable artifacts.
522. R-083: Final closure condition for Dimension 3 is runnable contract and implementation evidence, not only prose.
523. R-084: Final closure condition for Dimension 4 is all five cross-cutting constraints represented in machine-readable service artifacts.
524. R-085: Final closure condition for Dimension 5 is counterpart union coverage with explicit accepted gaps.
525. R-086: Final closure condition for Dimension 6 is domain-specific operational material with incident drills and SLO overlays.
526. R-087: Final closure condition for Dimension 7 is evidence-backed compliance and privacy handling for behavioral, device, fraud, and account-linkage signals.
527. R-088: Final closure condition for Dimension 8 is a benchmark harness, public methodology, and target numbers separated from deployment caps.
528. R-089: Final closure condition for Dimension 9 is an implementation backlog that deletes or rewrites retired material before adding more prose.
529. R-090: This audit can be considered complete as a detection pass when the three deliverables exist, exceed line floors, avoid new retired-level scaffolding, cite evidence, and include the orchestrator report.

<!-- ORCHESTRATOR REPORT
  µservice: detection
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/detection/coherence-audit-2026-05-20.md (622 lines)
    - /Users/jasonlee/oyatie/microservices/detection/feature-parity-matrix-2026-05-20.md (456 lines)
    - /Users/jasonlee/oyatie/microservices/detection/performance-benchmark-numbers-2026-05-20.md (389 lines)
  inventory_files_seen: 130
  inventory_lines_read: 12264
  chat_history_matches_processed: 4
  findings_p0: 0
  findings_p1: 9
  findings_p2: 18
  findings_p3: 3
  tier_retirement_candidates_found: 154 uppercase candidate lines; examples: reference-implementations/streaming-score-rust-sdk.md:203, tutorials/build-payment-fraud-cedar-rule.md:15, capability-tiers/tier-matrix.md:13, capability-tiers/tier-matrix.md:43, capability-tiers/tier-matrix.md:70, capability-tiers/tier-matrix.md:100, benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:13, benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:21, benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:22, capability-tiers/tier-deltas-and-pricing.md:18-25, capability-tiers/tier-deltas-and-pricing.md:213-227, capability-tiers/tier-deltas-and-pricing.md:299-328
  tenant_class_adoption_gaps: yes; no tenant_class, demo_trial, paid-as-class, or revenue_share semantics are represented in local contracts, manifest, SLOs, policies, IaC, or runbooks
  top_3_counterparts_confirmed: Cloudflare Bot Management / Google reCAPTCHA Enterprise / DataDome
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1467
-->
