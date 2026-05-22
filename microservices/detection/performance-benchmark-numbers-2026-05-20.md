---
doc_class: Performance-Benchmark-Numbers
microservice: detection
audit_date: 2026-05-20
batch: wave-3-batch-3.2
status: landed
counterparts:
  - Cloudflare Bot Management
  - Google reCAPTCHA Enterprise
  - DataDome
---

# Detection performance benchmark numbers

## Five-citation anchor block
- Local latency commitments: `microservices/detection/PRD.md:657-672`, `PRD.md:921-929`, and `ADR-DET-001-streaming-vs-batch-substrate-split.md:61-63`.
- Local old benchmark debt: `microservices/detection/benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:11-83`.
- Canonical deployment and infrastructure doctrine: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` §D-15..§D-20 and `specs/master-plan-sequencing.json:704-867`.
- Cloudflare sources: https://developers.cloudflare.com/bots/concepts/bot-detection-engines/, https://developers.cloudflare.com/bots/reference/bot-management-variables/, and https://developers.cloudflare.com/bots/get-started/bot-management/.
- Google sources: https://docs.cloud.google.com/recaptcha/docs/interpret-assessment-website, https://docs.cloud.google.com/recaptcha/docs/create-assessment-website, and https://docs.cloud.google.com/recaptcha/quotas.
- DataDome sources: https://datadome.co/products/bot-protection/ and https://docs.datadome.co/docs/welcome-to-datadome-platform.

## Explicit methodology disclosure
1. This document is a benchmark target and public-number audit, not a measured Oyatie load-test result.
2. Counterpart numbers are public-documentation numbers unless marked as an estimate.
3. If a counterpart does not publish a latency, this report says so instead of inventing a measured value.
4. Estimates are bounded from public product architecture statements and are marked as estimates.
5. Oyatie numbers are target numbers for remediation and future implementation verification.
6. Oyatie targets are single industry-leader-grade targets with deployment-context overlays.
7. Oyatie targets are not segmented by retired capability levels.
8. Tenant class changes admission, usage caps, billing, SLO contract, and compliance allowances.
9. Tenant class does not lower detection quality.
10. Workloads separate edge prefiltering, token assessment, synchronous scoring, graph-enriched scoring, batch replay, and fairness audit.
11. Edge prefiltering is comparable to Cloudflare and DataDome.
12. Token assessment is comparable to reCAPTCHA Enterprise.
13. Synchronous scoring is comparable to fraud and account-risk services but is broader than bot-management alone.
14. Graph and batch jobs are Oyatie-broader capabilities and are not direct Cloudflare/reCAPTCHA/DataDome equivalents.
15. Deployment contexts evaluated: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
16. Tenant classes evaluated: `demo_trial`, `paid`, and `revenue_share`.
17. Operating-system disclosure: no local `supported-oses.json` exists, so OS-specific targets are requirements rather than verified results.
18. Architecture disclosure: no local `src/` or `tests/` exist, so every Oyatie number below requires future load-test evidence.
19. IaC disclosure: canonical OpenTofu context modules are absent today.
20. OCI disclosure: the OCI Always Free profile path is absent today, so demo-trial capacity is a target overlay rather than a deployed shape.

## §1 Methodology
21. Benchmark dimension 1: edge decision latency.
22. Edge decision workload: HTTP request with headers, IP prefix, user-agent hash, TLS fingerprint, resource class, and cached tenant policy.
23. Edge decision success metric: p99 under 2 ms when the decision can be made without graph or remote model calls.
24. Benchmark dimension 2: token assessment latency.
25. Token assessment workload: one-use token verification with expected action, tenant, principal, and risk score.
26. Token assessment success metric: p99 under 100 ms at regional service boundary.
27. Benchmark dimension 3: synchronous signal scoring latency.
28. Synchronous workload: payment, account, content, or bot signal with cached features and rules.
29. Synchronous success metric: p99 under 150 ms for rules plus cached feature scoring.
30. Benchmark dimension 4: model-enriched scoring latency.
31. Model workload: LightGBM or ONNX-style scoring with explainability output and audit event.
32. Model success metric: p99 under 225 ms without graph traversal.
33. Benchmark dimension 5: graph-enriched scoring latency.
34. Graph workload: request needs entity neighborhood lookup or fraud-ring score.
35. Graph success metric: p99 under 400 ms when graph lookup is explicitly required.
36. Benchmark dimension 6: sustained throughput.
37. Throughput workload: 70% bot/account signals, 20% payment/content signals, 8% model-enriched, 2% graph-enriched.
38. Throughput success metric: no p99 SLO breach under target requests per second per deployment context.
39. Benchmark dimension 7: concurrent active tenants.
40. Concurrency workload: tenants with mixed usage caps and compliance packs.
41. Concurrency success metric: tenant isolation holds under load and no tenant violates another tenant's SLO.
42. Benchmark dimension 8: false-positive rate.
43. False-positive workload: verified-human and verified-good automation traffic.
44. False-positive success metric: visible challenge or adverse mitigation false-positive rate below 0.01% for edge bot actions.
45. Benchmark dimension 9: explanation completeness.
46. Explanation workload: every score returns stable reason codes, score source, model/rule version, and audit ID.
47. Explanation success metric: 99.99% complete explanations for synchronous decisions.
48. Benchmark dimension 10: replay determinism.
49. Replay workload: deterministic replay from stored features, rules, model versions, and seeds.
50. Replay success metric: 99.99% replay equivalence for same input corpus and versions.
51. Benchmark dimension 11: batch retrospective wall-clock.
52. Batch workload: graph and historical replay corpus.
53. Batch success metric: 4-hour overnight window for large retrospective sweeps unless context overlay caps it.
54. Benchmark dimension 12: cost admission.
55. Cost workload: context-specific substrate with per-tenant usage model.
56. Cost success metric: demo-trial stays within OCI Always Free profile; paid and revenue-share use contract or at-cost admission.

## §2 Counterpart numbers

### §2.1 Cloudflare Bot Management numbers
57. CF-001 | Bot Score range | 1 to 99 | Source: Cloudflare bot detection engines docs.
58. CF-002 | Definite bot template threshold | score equals 1 | Source: Cloudflare Bot Management get-started docs.
59. CF-003 | Likely bot template threshold | score from 2 through 29 | Source: Cloudflare Bot Management get-started docs.
60. CF-004 | Machine learning scope | majority of detections | Source: Cloudflare bot detection engines docs.
61. CF-005 | Network training scale | billions of proxied requests daily | Source: Cloudflare bot detection engines docs.
62. CF-006 | Product training scale phrasing | hundreds of billions of requests per day | Source: Cloudflare product page.
63. CF-007 | Rule variables exposed | at least 8 bot-related fields: score, verified bot, static resource, JA3, JA4, detection IDs, signed agent, and verified category | Source: Cloudflare Bot Management variables docs.
64. CF-008 | Log fields exposed | 4 named log fields: BotDetectionIDs, BotScore, BotScoreSrc, BotTags | Source: Cloudflare Bot Management variables docs.
65. CF-009 | JavaScript detection first-request limitation | first request generally has no JS detection data | Source: Cloudflare JavaScript Detections docs.
66. CF-010 | JavaScript detection result semantics | execution success or error does not itself declare human or bot | Source: Cloudflare JavaScript Detections docs.
67. CF-011 | Heuristic floor | heuristics can still yield score 1 after JS passes | Source: Cloudflare JavaScript Detections docs.
68. CF-012 | Edge latency | not publicly published in the cited docs | Source: cited Cloudflare docs do not provide a p50/p95/p99 latency table.
69. CF-013 | False-positive numeric target | not publicly published in the cited docs | Source: cited Cloudflare docs.
70. CF-014 | Request coverage | Cloudflare product page says a bot score is created for every request | Source: Cloudflare product page.
71. CF-015 | Estimate for comparison | edge decision should be treated as sub-network-hop and target under 2 ms only when deployed at edge | Estimated from edge architecture; not a Cloudflare-published latency number.

### §2.2 Google reCAPTCHA Enterprise numbers
72. G-001 | Global score range | 0.0 to 1.0 | Source: Google interpret-assessment docs.
73. G-002 | Score levels | 11 levels | Source: Google interpret-assessment docs.
74. G-003 | Pre-review visible score levels | 4 levels: 0.1, 0.3, 0.7, and 0.9 | Source: Google interpret-assessment docs.
75. G-004 | Token reuse | one use | Source: Google create-assessment docs.
76. G-005 | Token expiry | 2 minutes | Source: Google create-assessment docs.
77. G-006 | Free monthly assessment quota without billing | 10,000 requests per calendar month | Source: Google quotas docs.
78. G-007 | API request quota | 60,000 requests per minute | Source: Google quotas docs.
79. G-008 | MFA email quota | 10 requests per email address per day | Source: Google quotas docs.
80. G-009 | MFA SMS daily phone quota | 10 requests per phone number per day | Source: Google quotas docs.
81. G-010 | MFA SMS four-hour phone quota | 5 requests per phone number per 4 hours | Source: Google quotas docs.
82. G-011 | MFA total daily quota | 10,000 MFA requests per day | Source: Google quotas docs.
83. G-012 | Quota error status | HTTP 429 Resource Exhausted | Source: Google quotas docs.
84. G-013 | New-key scoring warmup | wait 48 hours before acting on returned scores for accurate analysis | Source: Google create-assessment docs.
85. G-014 | Early production drift warning | scores within 7 days of implementation can differ from long-term production | Source: Google interpret-assessment docs.
86. G-015 | SMS defense polarity | SMS risk 0.0 means low confidence of toll fraud; 1.0 means high confidence | Source: Google interpret-assessment docs.

### §2.3 DataDome numbers
87. DD-001 | Edge response time claim | under 2 ms | Source: DataDome Bot Protect product page.
88. DD-002 | Edge points of presence | 35+ PoPs | Source: DataDome Bot Protect product page.
89. DD-003 | Daily signal scale | over 5 trillion signals per day | Source: DataDome Bot Protect product page.
90. DD-004 | Signal breadth | hundreds of client-side and server-side signals | Source: DataDome Bot Protect product page.
91. DD-005 | Model breadth | 1000+ out-of-the-box and customer-specific models | Source: DataDome Bot Protect product page.
92. DD-006 | False-positive claim | below 0.01% false-positive rate | Source: DataDome Bot Protect product page.
93. DD-007 | Integration breadth | 80+ pre-built integrations | Source: DataDome Bot Protect product page.
94. DD-008 | SOC supervision | 24/7 SOC team monitoring | Source: DataDome Bot Protect product page.
95. DD-009 | Protected surfaces | websites, mobile apps, APIs, and MCP servers | Source: DataDome Bot Protect product page.
96. DD-010 | Request coverage | analyzes every request | Source: DataDome Bot Protect product page.
97. DD-011 | Platform modules visible in docs | Bot Protect, Agentic Trust, DDoS Protect, Account Protect, Ad Protect, Priority Protect, and Page Protect | Source: DataDome platform docs.
98. DD-012 | Setup time claim | setup measured in hours, not days | Source: DataDome Bot Protect product page.
99. DD-013 | Product response class | automated bot mitigation on autopilot | Source: DataDome Bot Protect product page.
100. DD-014 | Direct graph-detection benchmark | not publicly published in cited sources | Source: DataDome pages do not cover graph-ring batch detection.
101. DD-015 | Direct payment-fraud benchmark | not publicly published in cited sources | Source: DataDome bot page is broader bot/fraud but does not publish payment scoring p99.

## §3 Oyatie target numbers — single industry-leader target set
102. O-001 | Edge prefilter latency p50 | target 0.60 ms | Canonical target: parity with DataDome under-2-ms claim.
103. O-002 | Edge prefilter latency p95 | target 1.40 ms | Canonical target: preserve headroom before 2 ms.
104. O-003 | Edge prefilter latency p99 | target 1.90 ms | Canonical target: beat DataDome public under-2-ms claim.
105. O-004 | Edge prefilter throughput per public-cloud cell | target 100,000 requests/sec | Context overlay: `oyatie-public-cloud` elastic.
106. O-005 | Edge prefilter throughput per guest AWS cell | target 50,000 requests/sec | Context overlay: customer account quota dependent.
107. O-006 | Edge prefilter throughput per guest OCI cell | target 40,000 requests/sec | Context overlay: non-free OCI tenancy quota dependent.
108. O-007 | Edge prefilter throughput per OCI Always Free profile | target 400 requests/sec sustained, 1,000 requests/sec 60-second burst | Context overlay: bounded by 4 OCPU and 24 GB RAM.
109. O-008 | Edge prefilter throughput per on-prem cell | target 25,000 requests/sec per appliance pair | Context overlay: facility network and hardware dependent.
110. O-009 | Edge prefilter throughput per colo cell | target 75,000 requests/sec per edge cluster | Context overlay: rack and transit dependent.
111. O-010 | Edge prefilter throughput as Oyatie cloud provider | target 150,000 requests/sec per provider edge cell | Context overlay: provider-controlled elasticity.
112. O-011 | Token assessment latency p50 | target 18 ms | Comparable to backend assessment call class, not published competitor latency.
113. O-012 | Token assessment latency p95 | target 65 ms | Includes one-use token lookup and expected-action check.
114. O-013 | Token assessment latency p99 | target 100 ms | Maintains enough margin for login and checkout flows.
115. O-014 | Token expiry | target 120 seconds | Matches Google one-use/two-minute expiry.
116. O-015 | Token reuse rejection | target 100% duplicate-token rejection | Security invariant.
117. O-016 | Score scale for bot edge | target 1-99 compatibility projection plus native 0.0-1.0 risk | Interop with Cloudflare and Google style consumers.
118. O-017 | Score source completeness | target 99.99% decisions include source enum | Needed to match bot score source and reason-code expectations.
119. O-018 | Reason-code completeness | target 99.99% decisions include stable reason codes | Replaces free-form explanation-only response.
120. O-019 | Rules plus cached-feature scoring p50 | target 20 ms | Faster than current PRD p95/p99 wording.
121. O-020 | Rules plus cached-feature scoring p95 | target 80 ms | Synchronous fraud/account path.
122. O-021 | Rules plus cached-feature scoring p99 | target 150 ms | Beats old local p95 under 250 ms goal.
123. O-022 | Model-enriched scoring p50 | target 35 ms | Cached feature plus model.
124. O-023 | Model-enriched scoring p95 | target 140 ms | Includes explanation.
125. O-024 | Model-enriched scoring p99 | target 225 ms | Beats current PRD p95 under 450 ms model+graph envelope for non-graph calls.
126. O-025 | Graph-enriched scoring p50 | target 55 ms | Only when graph lookup explicitly required.
127. O-026 | Graph-enriched scoring p95 | target 220 ms | Graph cache required.
128. O-027 | Graph-enriched scoring p99 | target 400 ms | Keeps under the current 450-ms envelope.
129. O-028 | Batch replay determinism | target 99.99% same-input equivalence | Stronger than current PRD 99.9%.
130. O-029 | Batch replay start acceptance latency | target 2 seconds p99 | API accepts replay quickly; processing is asynchronous.
131. O-030 | Batch retrospective sweep window | target under 4 hours for planned large sweeps | Matches ADR-DET-001 overnight window claim.
132. O-031 | False-positive rate for visible bot challenge | target below 0.01% | Matches DataDome public claim.
133. O-032 | False-positive appeal acknowledgement | target p95 under 60 seconds | Aligns with existing case handoff goal.
134. O-033 | False-negative regression escape for known bot replay corpus | target below 0.10% per release | Requires replay corpus.
135. O-034 | Verified automation allow rate | target 99.99% of validated good automation not challenged | Requires verified-bot model.
136. O-035 | Assessment annotation ingestion | target p99 under 500 ms | For feedback labels.
137. O-036 | Drift alert detection delay | target under 15 minutes for edge score distribution shift | Faster than daily-only drift for bot traffic.
138. O-037 | Dashboard freshness | target p95 under 30 seconds for score and action metrics | Operational visibility target.
139. O-038 | Audit event emission | target 99.999% of decisions emit audit ID before side effect | Safety invariant.
140. O-039 | Tenant isolation leakage | target 0 cross-tenant feature reads | Safety invariant.
141. O-040 | Policy decision budget | target p99 under 5 ms for cached Cedar policy | Needed for edge gate.

## §3.1 Deployment-context overlays
142. `oyatie-public-cloud`: all target latencies apply; throughput elastically scales by adding edge cells and scoring workers.
143. `oyatie-public-cloud`: graph-enriched scoring can use managed internal graph clusters with cross-region replication.
144. `oyatie-public-cloud`: batch replay target remains under 4 hours for planned large sweeps.
145. `guest-on-aws`: target latencies apply if customer grants required regional quota and networking.
146. `guest-on-aws`: throughput target is lower until customer account limits and load-balancer quotas are verified.
147. `guest-on-aws`: BYOK and compliance packs are allowed for paid and revenue-share tenants if account policies permit.
148. `guest-on-oci`: target latencies apply for paid or revenue-share deployments outside the free profile.
149. `guest-on-oci`: the OCI Always Free profile only applies to demo-trial infrastructure.
150. `guest-on-oci`: non-free OCI tenancy target is 40,000 requests/sec per cell before additional quota.
151. `on-prem`: edge prefilter target depends on appliance CPU, NIC, and local TLS termination.
152. `on-prem`: graph-enriched scoring target requires local graph cache or a declared remote-call exception.
153. `on-prem`: batch sweeps can exceed 4 hours if customer hardware is undersized; admission must disclose this.
154. `colo`: edge targets apply with sufficient rack capacity and transit.
155. `colo`: throughput target is 75,000 requests/sec per edge cluster.
156. `colo`: HA depends on rack, power, and cross-connect design.
157. `oyatie-as-cloud-provider`: strongest elasticity target because Oyatie controls substrate, admission, and routing.
158. `oyatie-as-cloud-provider`: public-provider cell target is 150,000 requests/sec per edge cell.
159. `oyatie-as-cloud-provider`: graph and batch targets apply when service placement follows canonical control-plane placement.
160. All contexts: OpenTofu module evidence is required before any target becomes a deployed claim.

## §3.2 Tenant-class overlays
161. `demo_trial`: target quality is identical, but request volume is capped.
162. `demo_trial`: default cap target is 10,000 assessments per month when using free-profile posture, matching the reCAPTCHA free monthly reference point.
163. `demo_trial`: default rate cap target is 60 requests per minute unless a local demo needs a stricter cap.
164. `demo_trial`: OCI Always Free profile target is 400 requests/sec sustained only for local edge prefilter benchmarks; monthly caps still prevent free-profile abuse.
165. `demo_trial`: no compliance packs and no BYOK.
166. `demo_trial`: best-effort SLO, but failed quality checks still block release.
167. `paid`: target quality is identical and volume scales by contract and usage billing.
168. `paid`: contractual SLO can include all latency and false-positive targets.
169. `paid`: compliance packs and BYOK are allowed.
170. `paid`: scaling target is limited by purchased capacity and deployment context quota.
171. `revenue_share`: target quality is identical and substrate runs at cost or zero margin.
172. `revenue_share`: usage caps are tied to revenue accounting and risk budget.
173. `revenue_share`: compliance packs and BYOK are allowed when the commercial agreement funds the required substrate.
174. `revenue_share`: admission must avoid free-profile leakage into commercial high-volume workloads.
175. All tenant classes: no feature is removed because of tenant class.
176. All tenant classes: only usage, cost, SLO contract, compliance allowance, and substrate admission change.

## §4 Comparison narrative
177. Edge prefilter p99: Oyatie target 1.90 ms is ahead of DataDome's public under-2-ms claim if implemented at edge.
178. Edge prefilter p99: Oyatie has no local implementation today, so current status is catch-up until Rust and OpenTofu evidence exists.
179. Bot score range: Oyatie target includes 1-99 compatibility and 0.0-1.0 native risk, so target is parity with Cloudflare and Google.
180. Bot score range: current local contract only says number, so current status is partial.
181. Token expiry: Oyatie target 120 seconds is parity with Google.
182. Token expiry: current local artifacts have no token, so current status is missing.
183. API quota: Google publishes 60,000 requests per minute; Oyatie paid target is deployment-capacity based and should exceed this in public cloud.
184. API quota: demo-trial target deliberately mirrors a smaller free-profile posture.
185. False-positive rate: Oyatie target below 0.01% is parity with DataDome's public claim.
186. False-positive rate: current PRD lacks the numeric target, so current status is missing.
187. Request coverage: Cloudflare and DataDome claim every-request scoring/analysis.
188. Request coverage: Oyatie target requires every ingress request to receive an edge prefilter decision where the edge adapter is deployed.
189. Signal scale: DataDome claims over 5 trillion signals per day.
190. Signal scale: Oyatie does not need global vendor scale for every tenant but must prove per-context scaling and queue admission.
191. Signal scale status: catch-up for public marketing claims; parity for customer-owned deployments only after context load tests.
192. Model breadth: DataDome claims 1000+ models.
193. Model breadth: Oyatie should not target model count as quality; it should target model-card coverage and replay performance.
194. Model breadth status: current model registry is planned only.
195. Integration breadth: DataDome claims 80+ integrations.
196. Integration breadth: Oyatie has no integration catalog for bot protection; status is missing.
197. JA3/JA4: Cloudflare and Google docs expose or recommend those fingerprints.
198. JA3/JA4: Oyatie local contracts lack those fields; status is missing.
199. JavaScript/browser detection: Cloudflare and Google rely on browser-side execution paths.
200. JavaScript/browser detection: Oyatie lacks browser signal contracts; status is missing.
201. Verified automation: Cloudflare exposes verified bot and signed-agent fields; DataDome sells agent trust.
202. Verified automation: Oyatie lacks verified automation; status is missing.
203. WAF/custom rule integration: Cloudflare has WAF custom rules.
204. WAF/custom rule integration: Oyatie has Cedar rules but no edge WAF adapter; status is partial.
205. Replay determinism: Oyatie target 99.99% is ahead of bot-management counterparts because replay is a broader regulated-risk feature.
206. Replay determinism: current PRD has 99.9%, so target tightens the current metric.
207. Graph scoring: Oyatie target has no direct counterpart among the three listed products.
208. Graph scoring: treat as Oyatie-broader capability and benchmark separately.
209. Batch retrospective: Oyatie target under 4 hours follows local ADR rationale, not top-three bot-product parity.
210. Batch retrospective: current benchmark references old vendors and retired levels, so rewrite is required.
211. OCI Always Free profile: Oyatie target creates a demo-trial cap, not a reduced product tier.
212. OCI Always Free profile: current service lacks the required path, so status is missing.
213. OS matrix: no counterpart benchmark comparison; this is canonical Oyatie deployability.
214. OS matrix: current service lacks supported OS manifest, so status is missing.

## §4.1 Numeric target table
215. Metric | Canonical target | Context overlay | Tenant overlay.
216. Edge decision p50 | 0.60 ms | all edge-capable contexts | same quality all classes.
217. Edge decision p95 | 1.40 ms | all edge-capable contexts | same quality all classes.
218. Edge decision p99 | 1.90 ms | free profile capped by admission | same quality all classes.
219. Token assessment p50 | 18 ms | regional service boundary | same quality all classes.
220. Token assessment p95 | 65 ms | regional service boundary | same quality all classes.
221. Token assessment p99 | 100 ms | regional service boundary | same quality all classes.
222. Token expiry | 120 seconds | all contexts | same quality all classes.
223. Duplicate-token rejection | 100% | all contexts | same quality all classes.
224. Cached scoring p50 | 20 ms | all contexts with local cache | same quality all classes.
225. Cached scoring p95 | 80 ms | all contexts with local cache | same quality all classes.
226. Cached scoring p99 | 150 ms | all contexts with local cache | same quality all classes.
227. Model scoring p50 | 35 ms | requires local model runtime | same quality all classes.
228. Model scoring p95 | 140 ms | requires local model runtime | same quality all classes.
229. Model scoring p99 | 225 ms | requires local model runtime | same quality all classes.
230. Graph scoring p50 | 55 ms | requires graph cache | same quality all classes.
231. Graph scoring p95 | 220 ms | requires graph cache | same quality all classes.
232. Graph scoring p99 | 400 ms | requires graph cache | same quality all classes.
233. Public-cloud throughput | 100,000 rps/cell | elastic | paid/revenue-share scale by contract.
234. Guest AWS throughput | 50,000 rps/cell | quota dependent | paid/revenue-share scale by contract.
235. Guest OCI throughput | 40,000 rps/cell | quota dependent | paid/revenue-share scale by contract.
236. OCI Always Free profile throughput | 400 rps sustained | 4 OCPU and 24 GB RAM | demo-trial only.
237. On-prem throughput | 25,000 rps/appliance pair | hardware dependent | paid/revenue-share only unless demo lab.
238. Colo throughput | 75,000 rps/edge cluster | rack dependent | paid/revenue-share scale by contract.
239. Provider-edge throughput | 150,000 rps/cell | Oyatie-controlled | paid/revenue-share scale by contract.
240. False-positive challenge rate | below 0.01% | all contexts | same quality all classes.
241. Known-bot regression escape | below 0.10% | all contexts | same quality all classes.
242. Explanation completeness | 99.99% | all contexts | same quality all classes.
243. Audit emission | 99.999% | all contexts | same quality all classes.
244. Replay determinism | 99.99% | all contexts with retained corpus | same quality all classes.
245. Drift alert delay | under 15 minutes | edge telemetry dependent | same quality all classes.
246. Dashboard freshness | p95 under 30 seconds | telemetry pipeline dependent | same quality all classes.
247. Batch sweep window | under 4 hours | hardware/data dependent | demo trial can cap corpus size.
248. Annotation ingest p99 | 500 ms | regional boundary | same quality all classes.
249. Policy decision p99 | 5 ms | local policy cache required | same quality all classes.
250. Tenant leakage | 0 | all contexts | same quality all classes.

## §4.2 Evidence gap closure
251. Required evidence 1: Rust source implementing edge prefilter, token assessment, and synchronous scoring.
252. Required evidence 2: contract tests proving score scale and token one-use semantics.
253. Required evidence 3: policy tests proving tenant and deployment-context guardrails.
254. Required evidence 4: load-test harness with public-cloud, guest-cloud, customer-site, and OCI Always Free profile overlays.
255. Required evidence 5: supported OS manifest with compile and package smoke tests.
256. Required evidence 6: OpenTofu plans for all six contexts.
257. Required evidence 7: latency histograms p50/p95/p99 for each workload.
258. Required evidence 8: false-positive and false-negative replay corpus results.
259. Required evidence 9: dashboard freshness and audit emission measurements.
260. Required evidence 10: cost admission proof for demo-trial, paid, and revenue-share tenants.
261. Current blocker: no `src/` directory.
262. Current blocker: no `tests/` directory.
263. Current blocker: no `supported-oses.json`.
264. Current blocker: no canonical OpenTofu context modules.
265. Current blocker: no OCI Always Free profile module.
266. Current blocker: old benchmark tied to retired capability-level rows.

## §4.3 Final benchmark stance
267. Cloudflare comparison stance: target parity on score, rule fields, WAF integration, verified automation, and every-request edge scoring.
268. Cloudflare comparison stance: current local artifacts are missing key bot-specific fields and edge placement.
269. Google comparison stance: target parity on token assessment, action verification, score scale, quotas, and reason codes.
270. Google comparison stance: current local artifacts are missing token semantics and action verification.
271. DataDome comparison stance: target parity on under-2-ms edge gate, false-positive rate, signal breadth, and integrations.
272. DataDome comparison stance: current local artifacts are missing edge latency proof, signal taxonomy, and integration catalog.
273. Oyatie broader stance: graph, replay, fairness, appeal, and compliance-pack surfaces exceed the top-three bot-product scope.
274. Oyatie broader stance: broader surfaces must not hide missing bot-product fundamentals.
275. The single target set above is the correct replacement for old capability-level benchmark rows.
276. The target set must be enforced through deployment-context and tenant-class overlays.
277. The target set must be revalidated after runnable code and OpenTofu modules exist.
278. No current performance claim should be marketed as achieved.
279. Current status is target-defined, evidence-incomplete.
280. The next benchmark action is to delete or rewrite the old benchmark file's retired capability rows and add a load harness path that actually exists.

## §4.4 Workload-to-source trace
281. Edge bot score trace: Cloudflare score and DataDome under-2-ms claim drive O-001 through O-010.
282. Token trace: Google token one-use/two-minute expiry drives O-014 and O-015.
283. Quota trace: Google 10,000 monthly free and 60,000 rpm quotas inform demo-trial and paid admission targets.
284. False-positive trace: DataDome below-0.01% claim drives O-031.
285. Signal scale trace: DataDome 5-trillion-signal claim drives public-cloud scale pressure but not direct per-tenant requirements.
286. Model breadth trace: DataDome 1000+ model claim is not adopted as a target because model count is not quality.
287. Integration trace: DataDome 80+ integration claim drives integration catalog requirement.
288. Rules trace: Cloudflare WAF fields drive edge adapter and field schema requirements.
289. Reason trace: Google reason codes drive stable reason-code enum requirement.
290. Replay trace: local PRD and ADR drive deterministic replay targets.
291. Graph trace: local PRD and ADR drive graph latency targets.
292. OS trace: ADR-0328 and master plan drive OS manifest requirement.
293. IaC trace: ADR-0328 and master plan drive OpenTofu module requirement.
294. OCI trace: ADR-0328 and memory drive OCI Always Free profile target.
295. Tenant-class trace: current batch directive drives class overlays.

## §4.5 Non-claims
296. This report does not claim Oyatie currently meets under-2-ms edge latency.
297. This report does not claim Oyatie currently supports 100,000 requests/sec per cell.
298. This report does not claim Oyatie currently implements token assessment.
299. This report does not claim Oyatie currently implements verified automation.
300. This report does not claim Oyatie currently implements JA3/JA4 fingerprinting.
301. This report does not claim Oyatie currently implements browser JavaScript detections.
302. This report does not claim Oyatie currently implements mobile SDK protection.
303. This report does not claim Oyatie currently implements DataDome-scale integrations.
304. This report does not claim the old benchmark harness exists.
305. This report does not claim any retired capability-level row remains valid.
306. This report only defines the target benchmark surface and the evidence needed to prove it.

## §4.6 Closure checklist
307. Closure item 1: add or update contracts.
308. Closure item 2: add Rust implementation.
309. Closure item 3: add tests.
310. Closure item 4: add supported OS manifest.
311. Closure item 5: add OpenTofu context modules.
312. Closure item 6: add OCI Always Free profile.
313. Closure item 7: add load harness.
314. Closure item 8: run edge latency benchmarks.
315. Closure item 9: run token assessment benchmarks.
316. Closure item 10: run scoring latency benchmarks.
317. Closure item 11: run graph scoring benchmarks.
318. Closure item 12: run replay determinism benchmarks.
319. Closure item 13: run false-positive corpus benchmarks.
320. Closure item 14: run quota/admission tests.
321. Closure item 15: run cross-context plan validation.
322. Closure item 16: publish fresh results with source, date, and environment.
323. Closure item 17: remove stale benchmark rows that depend on retired capability levels.
324. Closure item 18: keep tenant-class overlays limited to usage, cost, SLO contract, compliance, and BYOK.
325. Closure item 19: keep the same quality bar across all tenant classes.
326. Closure item 20: update the orchestrator audit once measured evidence exists.

## Final benchmark verdict
327. Verdict: benchmark targets are now defined, but local implementation evidence is absent.
328. Cloudflare parity target: achievable only after edge field schema, WAF adapter, and verified automation exist.
329. Google parity target: achievable only after token assessment, expected-action verification, quotas, and reason codes exist.
330. DataDome parity target: achievable only after under-2-ms edge gate, integration catalog, signal taxonomy, and false-positive measurement exist.
331. Oyatie broader targets for replay, graph, fairness, and appeal remain valuable but are not substitutes for bot-product parity.
332. The next measured benchmark must use the single target set in this file.
333. The next measured benchmark must disclose deployment context, tenant class, OS, architecture, hardware, workload mix, and commit hash.
334. The next measured benchmark must not use retired capability-level headings or rows.
