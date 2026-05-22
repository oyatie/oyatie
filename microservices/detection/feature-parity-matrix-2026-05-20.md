---
doc_class: Feature-Parity-Matrix
microservice: detection
audit_date: 2026-05-20
batch: wave-3-batch-3.2
counterparts:
  - Cloudflare Bot Management
  - Google reCAPTCHA Enterprise
  - DataDome
status: landed
---

# Detection feature-parity matrix

## Citation anchor block
- Local product purpose: `microservices/detection/PRD.md:29-69`.
- Local bounded contexts: `microservices/detection/manifest.json:46-158`.
- Local contracts: `microservices/detection/contracts/openapi-v1.yaml:10-104`, `contracts/asyncapi-v1.yaml:1-34`, and `contracts/detection-v1.proto:1-36`.
- Cloudflare source: https://developers.cloudflare.com/bots/concepts/bot-detection-engines/ and https://developers.cloudflare.com/bots/reference/bot-management-variables/.
- Google source: https://docs.cloud.google.com/recaptcha/docs/interpret-assessment-website, https://docs.cloud.google.com/recaptcha/docs/create-assessment-website, and https://docs.cloud.google.com/recaptcha/quotas.
- DataDome source: https://datadome.co/products/bot-protection/ and https://docs.datadome.co/docs/welcome-to-datadome-platform.

## Method
1. This matrix compares the local detection µservice to the union of Cloudflare Bot Management, Google reCAPTCHA Enterprise, and DataDome.
2. The comparison is product-surface parity, not implementation proof.
3. Local coverage requires a cited local artifact that names the capability or an equivalent bounded context.
4. Strong local coverage means a contract, policy, SLO, runbook, or implementation plan exists with relevant semantics.
5. Partial local coverage means the broad concept exists but the counterpart-specific capability is missing or underspecified.
6. Missing local coverage means no artifact appears to represent the counterpart capability.
7. Retired capability-level language is not used as a feature segmentation model in this report.
8. Tenant-class semantics are treated separately from feature quality.
9. The quality bar is uniform industry-leader grade across demo-trial, paid, and revenue-share classes.
10. Deployment overlays are not capability gates; they are infrastructure constraints.

## Counterpart 1 — Cloudflare Bot Management surface
11. Cloudflare capability: Bot Score from supervised machine learning.
12. Source fact: Cloudflare docs describe an ML engine that maps request features to a final 1-99 Bot Score.
13. Source fact: the product page says Cloudflare uses machine learning, behavioral analysis, and fingerprinting.
14. Local analogue: `PRD.md:725-740` defines composite scoring with LightGBM and SHAP.
15. Local analogue: `manifest.json:103-116` defines a composite-scorer bounded context.
16. Gap: local contracts expose numeric `score` but not a bot-specific 1-99 normalized score.
17. Gap: local docs do not define human-likelihood vs automation-likelihood polarity.
18. Coverage: partial.
19. Cloudflare capability: verified-bot allowlisting.
20. Source fact: Bot Management variables include `cf.bot_management.verified_bot`.
21. Source fact: Cloudflare uses reverse DNS validation, ASN blocks, public lists, internal data, and machine learning for legitimate automated traffic.
22. Local analogue: `PRD.md:759-774` defines investigation bridge and Cedar case gates.
23. Local analogue: `policy/tenant-scope.cedar` exists, but no verified-bot policy is named.
24. Gap: no verified bot entity, category, allowlist, or reverse-DNS validation appears in local artifacts.
25. Coverage: missing.
26. Cloudflare capability: static-resource exemption.
27. Source fact: Bot Management variables include static resource identification.
28. Local analogue: none found in contracts or manifest.
29. Gap: detection lacks HTTP resource classification and static-resource bypass semantics.
30. Coverage: missing.
31. Cloudflare capability: JA3 and JA4 TLS fingerprint fields.
32. Source fact: Bot Management variables expose JA3/JA4 fingerprints.
33. Local analogue: none in OpenAPI request fields.
34. Local gap: `contracts/openapi-v1.yaml:52-87` lacks JA3, JA4, TLS, header, and client fingerprint fields.
35. Coverage: missing.
36. Cloudflare capability: JavaScript detections.
37. Source fact: Cloudflare can inject JavaScript Detections and reports `js_detection.passed`.
38. Local analogue: content-abuse and account-takeover families in `PRD.md:37-69`.
39. Gap: no client-side detection script, execution result, clearance cookie, or browser signal contract exists.
40. Coverage: missing.
41. Cloudflare capability: anomaly detection against domain baseline.
42. Source fact: Cloudflare docs describe optional anomaly detection that records a baseline and scores outliers.
43. Local analogue: `PRD.md:820` mandates optimization and related quality requirements; `IP-018-drift-detection-daily.md` exists.
44. Gap: local drift detection is model-centric and not domain traffic-baseline anomaly detection.
45. Coverage: partial.
46. Cloudflare capability: bot detection IDs.
47. Source fact: Bot Management variables expose a list of heuristic detection IDs.
48. Local analogue: `contracts/openapi-v1.yaml:99-104` returns explanation strings.
49. Gap: no structured heuristic ID array exists.
50. Coverage: partial.
51. Cloudflare capability: bot score source in logs.
52. Source fact: Cloudflare log fields include BotScoreSrc.
53. Local analogue: `manifest.json:56-157` names emitted events.
54. Gap: AsyncAPI emits only tenant_id, audit_id, family, and score at `contracts/asyncapi-v1.yaml:21-34`.
55. Coverage: partial.
56. Cloudflare capability: WAF custom-rule integration.
57. Source fact: Bot Management variables are exposed in Ruleset Engine and WAF custom rules.
58. Local analogue: `PRD.md:708-723` defines a rules engine using Sigma-style DSL and Cedar gates.
59. Gap: no WAF binding, edge ruleset field, or path/method rule template exists.
60. Coverage: partial.
61. Cloudflare capability: challenge without ordinary CAPTCHA friction.
62. Source fact: Cloudflare product page emphasizes alternatives to ordinary CAPTCHAs.
63. Local analogue: adverse outcomes and appeals in `PRD.md:759-774`.
64. Gap: no managed challenge, proof-of-work, turnstile-like challenge, or fallback UX contract exists.
65. Coverage: missing.
66. Cloudflare capability: edge-scale analytics across every request.
67. Source fact: Cloudflare product page says bot score is generated for every request using large network traffic.
68. Local analogue: streaming pipeline in `PRD.md:657-672`.
69. Gap: local service does not define HTTP edge request ingestion, CDN edge deployment, or request-perimeter placement.
70. Coverage: partial.

## Counterpart 2 — Google reCAPTCHA Enterprise surface
71. Google capability: backend assessment endpoint.
72. Source fact: Google docs require backend assessment creation for token verification.
73. Local analogue: OpenAPI `/signals:evaluate` at `contracts/openapi-v1.yaml:10-34`.
74. Gap: local endpoint evaluates a detection signal, not a reCAPTCHA token.
75. Coverage: partial.
76. Google capability: one-use token with two-minute expiry.
77. Source fact: Google docs state each token can be used only once and expires after two minutes.
78. Local analogue: none in OpenAPI, proto, policy, or runbook artifacts.
79. Gap: no token expiration, replay-prevention, or one-use token validation is defined.
80. Coverage: missing.
81. Google capability: score range 0.0 to 1.0 with 11 levels.
82. Source fact: Google docs define 11 score levels from 0.0 to 1.0.
83. Local analogue: `contracts/openapi-v1.yaml:95-104` returns a numeric score.
84. Gap: no score scale, calibration bands, or action thresholds are defined.
85. Coverage: partial.
86. Google capability: action verification.
87. Source fact: Google docs require matching `action` and `expectedAction`.
88. Local analogue: `contracts/openapi-v1.yaml:68-78` has `family`, but no expected action.
89. Gap: local contracts cannot reject action spoofing.
90. Coverage: missing.
91. Google capability: reason codes.
92. Source fact: Google docs list reason codes such as automation, unexpected environment, high traffic, unexpected usage patterns, and low confidence.
93. Local analogue: response `explanation` strings in `contracts/openapi-v1.yaml:99-102`.
94. Gap: explanations are untyped strings, not stable reason enums.
95. Coverage: partial.
96. Google capability: advanced verdict reasons for enterprise subscription.
97. Source fact: Google docs distinguish reason-code access by billing/subscription state.
98. Local analogue: compliance packs in `manifest.json:174-186`.
99. Gap: local docs do not model reason-code access by tenant class or compliance pack.
100. Coverage: partial.
101. Google capability: account defense critical user actions.
102. Source fact: Google docs list actions such as login, registration, password reset, phone update, email update, account update, MFA trigger, redeem code, and payment-method listing.
103. Local analogue: account-takeover family in `PRD.md:46-49`.
104. Gap: local contracts do not enumerate critical actions or account event types.
105. Coverage: partial.
106. Google capability: payment transaction fraud prevention.
107. Source fact: Google docs describe payment fraud prevention and transaction signals.
108. Local analogue: payment-fraud family in `PRD.md:42-45`.
109. Gap: local contracts do not carry cart, instrument, transaction amount, merchant, dispute, or chargeback feature fields.
110. Coverage: partial.
111. Google capability: SMS defense with inverse risk score.
112. Source fact: Google docs define SMS defense score where 0.0 is low confidence of toll fraud and 1.0 is high confidence.
113. Local analogue: account-takeover family and investigation bridge.
114. Gap: no phone-number E.164 field, SMS event, toll-fraud verdict, or inverse score polarity exists.
115. Coverage: missing.
116. Google capability: project-level quotas.
117. Source fact: Google docs list 60,000 requests per minute and 10,000 free monthly requests without billing.
118. Local analogue: no capacity model with real quotas; `capacity-model.md:29-85` is scaffolded.
119. Gap: no admission-control quotas by tenant class.
120. Coverage: missing.
121. Google capability: annotation feedback for model tuning.
122. Source fact: Google docs recommend sending assessment IDs back to confirm true positives and true negatives or correct errors.
123. Local analogue: investigation bridge and appeal feedback in `PRD.md:759-774`.
124. Gap: no explicit assessment annotation API exists.
125. Coverage: partial.

## Counterpart 3 — DataDome surface
126. DataDome capability: edge bot protection across web, mobile, APIs, and MCP servers.
127. Source fact: DataDome product page covers websites, mobile apps, APIs, and MCP servers.
128. Local analogue: broad service substrate in `PRD.md:29-69`.
129. Gap: local docs do not place detection at HTTP edge, mobile SDK, API gateway, or MCP server boundary.
130. Coverage: partial.
131. DataDome capability: under-2-ms edge mitigation claim.
132. Source fact: DataDome product page says mitigation operates under 2 milliseconds across 35+ edge PoPs.
133. Local analogue: streaming latency goal under 250 ms rules and 450 ms model+graph at `PRD.md:657-672`.
134. Gap: local latency target is far slower than edge bot mitigation and applies to scoring pipeline, not inline edge decision.
135. Coverage: catch-up required.
136. DataDome capability: 5 trillion signals per day.
137. Source fact: DataDome product page claims analysis over 5 trillion signals per day.
138. Local analogue: no equivalent public scale target in local docs.
139. Gap: old benchmark uses 20k events/sec sustained at `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:71-83`, which is far below global edge-signal scale.
140. Coverage: missing for global benchmark; partial for tenant-scale substrate.
141. DataDome capability: 100s of client-side and server-side signals.
142. Source fact: DataDome product page says it evaluates hundreds of client-side and server-side signals.
143. Local analogue: feature store in `PRD.md:691-706`.
144. Gap: no enumerated browser, device, network, TLS, automation, or behavior signal schema exists.
145. Coverage: partial.
146. DataDome capability: 1000+ out-of-the-box and customer-specific models.
147. Source fact: DataDome product page claims 1000+ models.
148. Local analogue: model card registry plan exists at `IP-017-ml-model-card-registry.md`.
149. Gap: no model roster, model-family taxonomy, or model registry implementation exists.
150. Coverage: partial.
151. DataDome capability: false-positive rate below 0.01%.
152. Source fact: DataDome product page states an industry-leading false-positive rate below 0.01%.
153. Local analogue: fairness and appeals appear in `PRD.md:921-929` and `PRD.md:931-987`.
154. Gap: local success metrics do not set a false-positive rate target.
155. Coverage: missing.
156. DataDome capability: 80+ pre-built integrations.
157. Source fact: DataDome product page claims 80+ integrations.
158. Local analogue: no integration roster exists.
159. Gap: migration playbook covers Stripe Radar and Sift at `migration-playbooks/from-stripe-radar-and-sift.md`, not CDN, WAF, mobile, reverse proxy, or API gateway integrations.
160. Coverage: missing.
161. DataDome capability: AI agent trust.
162. Source fact: DataDome platform docs list Agentic Trust for intent-based AI traffic.
163. Local analogue: no signed-agent or AI-agent traffic class exists.
164. Gap: policy cannot distinguish legitimate AI agent traffic from malicious automation.
165. Coverage: missing.
166. DataDome capability: Account Protect.
167. Source fact: DataDome platform docs list Account Protect for fraud at login and account creation.
168. Local analogue: account-takeover family in `PRD.md:46-49`.
169. Gap: no account creation/login schema or protection workflow exists.
170. Coverage: partial.
171. DataDome capability: Ad Protect.
172. Source fact: DataDome platform docs list Ad Protect.
173. Local analogue: click fraud is not explicitly named; content-abuse and fake engagement are adjacent.
174. Gap: no ad campaign, impression, click, conversion, or bot-spend protection schema exists.
175. Coverage: missing.
176. DataDome capability: Priority Protect / virtual waiting room.
177. Source fact: DataDome platform docs list Priority Protect for high-demand events.
178. Local analogue: none found.
179. Gap: no queue, fairness, drop, or event-window control surface exists.
180. Coverage: missing.
181. DataDome capability: Page Protect.
182. Source fact: DataDome platform docs list Page Protect for client-side scripts and user data.
183. Local analogue: no client-side script inventory or browser-side data protection exists.
184. Coverage: missing.

## Union coverage matrix
185. U-001 | Real-time request scoring | Cloudflare, Google, DataDome | Local partial | `contracts/openapi-v1.yaml:10-34` evaluates signals; no request-edge schema.
186. U-002 | Bot score normalization | Cloudflare | Local missing | No 1-99 bot score or score polarity.
187. U-003 | Risk score 0.0-1.0 | Google | Local partial | Numeric score exists; no defined scale.
188. U-004 | Under-2-ms edge decision | DataDome | Local missing | Local PRD target is 250/450 ms at `PRD.md:657-672`.
189. U-005 | Verified bot classification | Cloudflare, DataDome | Local missing | No verified bot entity or allowlist.
190. U-006 | Signed agent classification | Cloudflare, DataDome | Local missing | No signed-agent field.
191. U-007 | AI agent trust | DataDome | Local missing | No agentic traffic category.
192. U-008 | JA3/JA4 fingerprints | Cloudflare, Google | Local missing | No TLS fingerprint fields.
193. U-009 | Browser JavaScript detection | Cloudflare, Google, DataDome | Local missing | No script, token, or client execution result.
194. U-010 | Token one-use expiry | Google | Local missing | No one-use token semantics.
195. U-011 | Action verification | Google | Local missing | No `expectedAction` equivalent.
196. U-012 | Reason-code enums | Google, Cloudflare | Local partial | Explanation strings exist but no stable enums.
197. U-013 | Heuristic detection IDs | Cloudflare | Local missing | No detection ID array.
198. U-014 | Bot score source in logs | Cloudflare | Local partial | Events exist but score source missing.
199. U-015 | Edge WAF rule integration | Cloudflare, DataDome | Local partial | Rules engine exists; WAF binding absent.
200. U-016 | Managed challenge / challenge result | Cloudflare, Google, DataDome | Local missing | No challenge flow.
201. U-017 | CAPTCHA minimization | Cloudflare, DataDome | Local missing | Appeals exist; user-facing friction model absent.
202. U-018 | Account takeover defense | Google, DataDome | Local partial | ATO family exists; action schema absent.
203. U-019 | Login event action | Google, DataDome | Local missing | No login event schema.
204. U-020 | Registration event action | Google, DataDome | Local missing | No registration event schema.
205. U-021 | Password reset action | Google | Local missing | No critical user action enum.
206. U-022 | Phone update action | Google | Local missing | No phone update schema.
207. U-023 | Email update action | Google | Local missing | No email update schema.
208. U-024 | MFA trigger action | Google | Local missing | No MFA event schema.
209. U-025 | Payment transaction fraud | Google | Local partial | Payment-fraud family exists; transaction fields absent.
210. U-026 | SMS toll fraud score | Google | Local missing | No SMS toll fraud model.
211. U-027 | Quota and monthly assessment cap | Google | Local missing | Capacity model scaffolded.
212. U-028 | Project-level rate limit | Google | Local missing | No per-project or tenant rate semantics.
213. U-029 | Assessment annotation feedback | Google | Local partial | Investigation feedback exists; annotation API absent.
214. U-030 | Client-side and server-side signal roster | DataDome | Local partial | Feature store exists; signal taxonomy absent.
215. U-031 | Global daily signal scale | DataDome | Local missing | No trillion-scale target.
216. U-032 | False-positive target | DataDome | Local missing | No numeric false-positive target.
217. U-033 | 24/7 SOC supervision | DataDome | Local missing | Incident docs exist; SOC operating model absent.
218. U-034 | 80+ integrations | DataDome | Local missing | No integration catalog.
219. U-035 | Reverse proxy integration | DataDome, Cloudflare | Local missing | No proxy adapter.
220. U-036 | CDN integration | Cloudflare, DataDome | Local missing | No CDN placement.
221. U-037 | Mobile app protection | Google, DataDome | Local missing | No mobile SDK fields.
222. U-038 | API protection | DataDome, Cloudflare | Local partial | OpenAPI exists; edge API protection absent.
223. U-039 | MCP server protection | DataDome | Local missing | No MCP surface.
224. U-040 | Account-protect product flow | DataDome | Local partial | ATO family only.
225. U-041 | Ad-protect product flow | DataDome | Local missing | No ad fraud schema.
226. U-042 | Priority-event protection | DataDome | Local missing | No waiting-room or queue control.
227. U-043 | Page script protection | DataDome | Local missing | No client script protection.
228. U-044 | Good bot category | Cloudflare | Local missing | No verified bot categories.
229. U-045 | Corporate proxy field | Cloudflare | Local missing | No corporate proxy field.
230. U-046 | Static resource field | Cloudflare | Local missing | No static resource field.
231. U-047 | Bot tags in logs | Cloudflare | Local missing | No bot tags.
232. U-048 | Bot analytics dashboard | Cloudflare, DataDome | Local partial | dashboards exist; bot analytics absent.
233. U-049 | Traffic timeline charts | DataDome | Local partial | dashboards exist; traffic timeline absent.
234. U-050 | Custom rules for bot protection | Cloudflare, DataDome | Local partial | rules engine exists; bot-specific templates absent.
235. U-051 | Rule promotion guard | Local stronger | Local present | `policy/rule-promotion.cedar` exists.
236. U-052 | Cedar static policy guard | Local stronger | Local present | `PRD.md:708-723` and policy files.
237. U-053 | Replay before activation | Local stronger | Local present | `PRD.md:776-791` and `/replay-runs`.
238. U-054 | Fairness audit | Local broader | Local present in docs | `PRD.md:921-987`; implementation absent.
239. U-055 | Appeal workflow | Local broader | Local present in docs | `PRD.md:759-774`; implementation absent.
240. U-056 | Graph community detection | Local broader | Local present in docs | `PRD.md:742-757`; bot counterparts do not cover this fully.
241. U-057 | AML sanctions detection | Local broader | Local present in docs | `PRD.md:52-55`.
242. U-058 | Synthetic identity detection | Local broader | Local present in docs | `PRD.md:50-51`.
243. U-059 | Insider-risk detection | Local broader | Local present in docs | `PRD.md:62-65`.
244. U-060 | Policy-violation detection | Local broader | Local present in docs | `PRD.md:66-69`.
245. U-061 | Model-card registry | Local broader | Local planned | `IP-017-ml-model-card-registry.md`; no runtime.
246. U-062 | Drift detection | Local broader | Local planned | `IP-018-drift-detection-daily.md`; no runtime.
247. U-063 | Pack-specific compliance overlays | Local broader | Local documented | `manifest.json:174-186`.
248. U-064 | Deterministic replay seed | Local broader | Local present in proto | `contracts/detection-v1.proto:27-31`.
249. U-065 | Audit ID in responses | Local broader | Local present | `contracts/openapi-v1.yaml:89-104`.
250. U-066 | Tenant ID in all requests | Local baseline | Local present | `contracts/openapi-v1.yaml:54-64`.
251. U-067 | Deployment context in request | Canonical required | Local missing | No field.
252. U-068 | Tenant class in request or derived context | Canonical required | Local missing | No field.
253. U-069 | OCI Always Free usage cap | Canonical required | Local missing | No IaC/profile.
254. U-070 | OpenTofu context module | Canonical required | Local missing | `iac/terraform/` exists instead.
255. U-071 | Supported OS manifest | Canonical required | Local missing | No `supported-oses.json`.
256. U-072 | Rust backend crate | Canonical required | Local missing | no `src/` or `Cargo.toml`.
257. U-073 | Contract test suite | Canonical required | Local missing | no `tests/`.
258. U-074 | Bot scoring calibration report | Union required | Local missing | no calibration doc.
259. U-075 | False-negative escape analysis | Union required | Local partial | runbooks exist but generic.
260. U-076 | False-positive remediation queue | Union required | Local partial | investigation bridge docs exist.
261. U-077 | Analyst case handoff | Local broader | Local present | `PRD.md:759-774`.
262. U-078 | Human review for adverse outcomes | Local broader | Local present in docs | compliance docs repeat this at `compliance.md:35-44`.
263. U-079 | Client fingerprint privacy DPIA | Union required | Local missing | DPIA scaffold has no fingerprint analysis.
264. U-080 | Bot-data retention policy | Union required | Local missing | compliance says retention is pack-specific but lacks details.
265. U-081 | Edge PoP placement | DataDome/Cloudflare | Local missing | no edge deployment module.
266. U-082 | Rate-limit action | Cloudflare/DataDome | Local missing | no mitigation action enum.
267. U-083 | Block action | All three | Local missing | no action enum.
268. U-084 | Allow action | All three | Local missing | no action enum.
269. U-085 | Challenge action | All three | Local missing | no action enum.
270. U-086 | Monitor-only mode | Google/DataDome/Cloudflare | Local missing | no enforcement mode.
271. U-087 | Customer custom model | DataDome | Local partial | model-card plan exists.
272. U-088 | Customer custom rule | Cloudflare/DataDome | Local partial | Cedar rule engine exists.
273. U-089 | Data lake export | Cloudflare | Local missing | no log export contract.
274. U-090 | SIEM integration | Cloudflare/DataDome | Local missing | no SIEM handoff.
275. U-091 | Security analytics | Cloudflare | Local partial | dashboards exist but no bot analytics.
276. U-092 | API key authentication | Google | Local missing | auth not defined in detection contract.
277. U-093 | Workload identity federation | Google | Local missing | no auth/deployment credential model.
278. U-094 | Account identifier hashing/encryption | Google | Local missing | no user identifier privacy model.
279. U-095 | Phone E.164 validation | Google | Local missing | no phone field.
280. U-096 | Payment instrument fields | Google | Local missing | no card/payment fields in contract.
281. U-097 | Model rollback | Local broader | Local planned | runbook exists but generic.
282. U-098 | Graph cluster action | Local broader | Local planned | `DetectionGraphClusterFound` in manifest.
283. U-099 | Replay mismatch action | Local broader | Local planned | runbook exists.
284. U-100 | Cross-µservice handoff | Canonical expected | Local missing | no handoff doc.

## Family summary
285. Family payment-fraud: local docs name the family and scoring intent at `PRD.md:42-45`.
286. Family payment-fraud: Google Fraud Prevention is the closest counterpart surface.
287. Family payment-fraud: DataDome Account Protect and bot mitigation are adjacent but not payment-instrument-specific in the cited product page.
288. Family payment-fraud: Cloudflare click-fraud and bot-fraud defenses are adjacent but not full payment fraud.
289. Family payment-fraud gap: local contracts need transaction amount, currency, instrument fingerprint, merchant, cart, dispute risk, and chargeback feedback fields.
290. Family account-takeover: local docs name the family at `PRD.md:46-49`.
291. Family account-takeover: Google Account Defense and DataDome Account Protect are strong counterparts.
292. Family account-takeover: Cloudflare credential stuffing coverage is relevant.
293. Family account-takeover gap: local docs need login, registration, password reset, MFA, device, IP, TLS fingerprint, and behavioral fields.
294. Family synthetic-identity: local docs name the family at `PRD.md:50-51`.
295. Family synthetic-identity: current top-three counterparts do not fully cover synthetic identity as a product family.
296. Family synthetic-identity gap: local docs need graph, document, account, device, and payment-link features.
297. Family AML-sanctions: local docs name the family at `PRD.md:52-55`.
298. Family AML-sanctions: current top-three counterparts do not cover AML/SAR depth.
299. Family AML-sanctions gap: keep broader substrate evidence separate from bot parity claims.
300. Family content-abuse: local docs name the family at `PRD.md:56-57`.
301. Family content-abuse: Cloudflare scraping and bot abuse, DataDome scraping, and reCAPTCHA spam protection are relevant.
302. Family content-abuse gap: local docs need content object, actor, upload, comment, and moderation outcome fields.
303. Family fake-reviews-engagement: local docs name the family at `PRD.md:58-61`.
304. Family fake-reviews-engagement: DataDome bot and AI-agent traffic are relevant.
305. Family fake-reviews-engagement gap: local docs need review, rating, graph, coordinated cluster, and campaign features.
306. Family insider-risk: local docs name the family at `PRD.md:62-65`.
307. Family insider-risk: current top-three counterparts do not cover insider risk.
308. Family insider-risk gap: mark as Oyatie-broader capability, not counterpart parity.
309. Family policy-violation: local docs name the family at `PRD.md:66-69`.
310. Family policy-violation: Cloudflare WAF rules and DataDome custom rules are adjacent.
311. Family policy-violation gap: local docs need policy action taxonomy, enforcement mode, and appeal semantics.

## Headline gap analysis
312. Headline gap 1: edge placement is undefined.
313. Evidence: local IaC has no per-context edge, CDN, WAF, proxy, or API gateway module.
314. Counterpart pressure: Cloudflare and DataDome both operate at the edge.
315. Required additive surface: edge request adapter and context-specific OpenTofu modules.
316. Headline gap 2: bot-specific fields are absent.
317. Evidence: OpenAPI request fields stop at tenant, principal, family, entity, trace, and compliance packs.
318. Counterpart pressure: Cloudflare exposes bot score, verified bot, static resource, JA3/JA4, detection IDs, signed agent, and categories.
319. Required additive surface: bot signal schema.
320. Headline gap 3: token assessment is absent.
321. Evidence: no token, expected action, or token-expiry field appears in local contracts.
322. Counterpart pressure: Google reCAPTCHA Enterprise is assessment-token centered.
323. Required additive surface: assessment token contract and replay-prevention policy.
324. Headline gap 4: false-positive target is absent.
325. Evidence: PRD metrics include explanation, fairness, handoff, replay, and cost, but no false-positive rate target at `PRD.md:921-929`.
326. Counterpart pressure: DataDome publishes a below-0.01% false-positive claim.
327. Required additive surface: false-positive and appeal-overturn SLO.
328. Headline gap 5: signal scale is underspecified.
329. Evidence: old benchmark uses 20k events/sec sustained at `benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md:71-83`.
330. Counterpart pressure: DataDome claims 5 trillion signals per day; Cloudflare uses network-scale traffic.
331. Required additive surface: context overlays for public cloud, guest clouds, customer sites, and OCI Always Free profile.
332. Headline gap 6: implementation proof is absent.
333. Evidence: no `src/`, no `tests/`, and reference SDK says runnable project will land later.
334. Counterpart pressure: product claims cannot be parity-complete without runnable enforcement.
335. Required additive surface: Rust crate, contract tests, replay tests, and policy tests.
336. Headline gap 7: retired capability vocabulary pollutes feature docs.
337. Evidence: old capability docs and benchmark rows use retired named levels.
338. Counterpart pressure: batch directive says quality bar is uniform, not capability-level stratified.
339. Required additive surface: tenant-class overlays for usage and billing only.
340. Headline gap 8: customer integration catalogue is absent.
341. Evidence: migration playbook covers old vendors only.
342. Counterpart pressure: DataDome claims 80+ integrations and Cloudflare integrates through WAF/rules/logs.
343. Required additive surface: integration matrix for CDN, WAF, API gateway, mobile SDK, reverse proxy, Kafka/Pulsar ingestion, SIEM, and data lake.
344. Headline gap 9: privacy treatment for fingerprints is absent.
345. Evidence: DPIA is scaffold-heavy and does not analyze JA3/JA4, browser telemetry, or device fingerprinting.
346. Counterpart pressure: bot products depend on browser, network, and device signals.
347. Required additive surface: DPIA and retention controls for client fingerprints.
348. Headline gap 10: verified AI agent treatment is absent.
349. Evidence: no signed-agent, verified AI agent, or AI crawler category in local schema.
350. Counterpart pressure: Cloudflare has signed-agent and verified categories; DataDome has Agentic Trust.
351. Required additive surface: verified-agent trust model and policy integration.

## Additive surface proposal
352. Add `DetectionBotSignal` schema.
353. `DetectionBotSignal` should include `http_method`, `path_class`, `resource_class`, `user_agent_hash`, `ip_prefix_hash`, `ja3`, `ja4`, `browser_signal_id`, and `automation_signal_ids`.
354. Add `BotScore` schema with `score`, `scale`, `polarity`, `source`, and `confidence`.
355. Add `VerifiedAutomation` schema with `verified_bot`, `verified_category`, `signed_agent`, `agent_trust_state`, and validation evidence.
356. Add `AssessmentToken` schema with `token_hash`, `issued_at`, `expires_at`, `one_use_nonce`, `action`, and `expected_action`.
357. Add `DetectionAction` enum with allow, monitor, rate-limit, challenge, block, case-open, and replay-required.
358. Add `ChallengeOutcome` schema with challenge type, issued time, solved state, and user-friction reason.
359. Add `ReasonCode` enum modeled as stable domain reasons, not free-form explanation text.
360. Add `ScoreSource` enum covering rules, model, graph, traffic baseline, client signal, verified automation, and manual override.
361. Add `TenantClassOverlay` with usage cap, billing model, compliance permission, BYOK permission, and SLO class.
362. Add `DeploymentContextOverlay` with edge placement, region, substrate, data residency, capacity envelope, and admission caps.
363. Add `FingerprintPrivacy` with retention duration, hashing, encryption, consent basis, and disclosure control.
364. Add `FalsePositiveSlo` with target, measurement window, appeal-overturn denominator, and remediation SLA.
365. Add `IntegrationAdapter` entries for Cloudflare-like WAF, reCAPTCHA-like token endpoint, DataDome-like reverse proxy, mobile SDK, API gateway, CDN, SIEM, and data lake.
366. Add `AnnotationFeedback` endpoint for true-positive, true-negative, false-positive, false-negative, appeal-overturned, and regulator-corrected labels.
367. Add `EdgePlacement` OpenTofu modules per deployment context.
368. Add OCI Always Free profile overlays only as infrastructure caps for demo-trial tenants.
369. Add Rust contract tests for score polarity, token expiry, duplicate token rejection, reason-code stability, and action verification.
370. Add Cedar policy tests for verified automation, tenant class, deployment context, and adverse action.

## Coverage summary
371. Local detection is stronger than the three counterparts on fairness, appeals, deterministic replay, and broad regulated-risk families.
372. Local detection is weaker than Cloudflare on edge bot fields, bot score semantics, WAF integration, bot analytics, and verified automation.
373. Local detection is weaker than Google on token assessment, expected-action verification, reason-code structure, SMS defense, quota semantics, and assessment annotation.
374. Local detection is weaker than DataDome on edge latency target, integration breadth, client/server signal taxonomy, false-positive target, and AI-agent trust packaging.
375. Local detection can meet the union bar without shrinking its product surface.
376. The correct move is to add a bot/abuse edge-signal slice within the broader substrate.
377. The bot/abuse slice should not replace payment-fraud, AML, graph, fairness, replay, or investigation domains.
378. The bot/abuse slice must be implemented in Rust and exposed through stable contracts.
379. The edge-signal slice must be deployable in all six contexts or explicitly marked non-applicable with canonical justification.
380. No such non-applicability justification exists today.
381. The new parity target must avoid retired capability-level segmentation.
382. Tenant class should change usage caps, billing, and SLO contracts; it should not remove product-quality capability.
383. Demo-trial class should still use the same scoring semantics but with hard volume and retention caps.
384. Paid class should allow contractual SLO, compliance packs, BYOK, and scaled throughput.
385. Revenue-share class should allow at-cost or zero-margin substrate admission with revenue accounting hooks.
386. All three classes require the same feature correctness standard.

## Acceptance criteria for closing parity gaps
387. AC-001: OpenAPI, AsyncAPI, and proto include bot signal, token assessment, action, reason code, deployment context, and tenant-class semantics.
388. AC-002: A Rust crate implements synchronous score evaluation and token replay rejection.
389. AC-003: Cedar tests enforce verified automation and tenant-class admission.
390. AC-004: OpenTofu modules exist for all six deployment contexts.
391. AC-005: OCI Always Free profile defines demo-trial usage caps without lowering score quality.
392. AC-006: Supported OS manifest exists and maps runtime dependencies.
393. AC-007: Bot analytics dashboard includes score distribution, score source, false-positive appeals, verified automation, and challenge outcomes.
394. AC-008: DPIA covers TLS/browser/device fingerprints.
395. AC-009: Runbooks distinguish false-positive spike, bot-score drift, token replay, verified-bot outage, and edge-latency regression.
396. AC-010: The migration playbook covers Cloudflare Bot Management, reCAPTCHA Enterprise, and DataDome integration paths.
397. AC-011: Performance targets include under-2-ms edge gate where detection is deployed at edge, and separate pipeline targets where batch or graph scoring is required.
398. AC-012: Benchmark methodology discloses public source, direct measurement, or estimate for every number.
399. AC-013: No new report creates old capability-level scaffolding.
400. AC-014: Retired old capability files are deleted, archived, or rewritten into tenant-class overlays in a separate remediation.
401. AC-015: Every closure claim includes file-line evidence and fresh verification output.

## Final parity verdict
402. Verdict: partial union coverage in documentation only.
403. The existing service is an ambitious broad detection substrate.
404. The existing service does not yet express the bot-management surface deeply enough to claim Cloudflare parity.
405. The existing service does not yet express token-assessment semantics deeply enough to claim reCAPTCHA Enterprise parity.
406. The existing service does not yet express edge latency, integrations, signal volume, and false-positive metrics deeply enough to claim DataDome parity.
407. The strongest local assets are replay, fairness, graph detection, and policy-governed model lifecycle.
408. The weakest local assets are edge placement, concrete implementation, per-context deployability, tenant-class adoption, and bot-specific contracts.
409. The required next artifact is not another broad competitor overview.
410. The required next artifact is a contract and implementation slice that maps the union bot/assessment surface into the detection substrate.
411. The parity bar is achievable if the service adds edge-signal contracts, token handling, verified automation, integration adapters, and false-positive SLOs.
412. The parity bar is not achievable by preserving old capability-level pricing files.
413. This report lands the requested union-coverage matrix and defers implementation to remediation.
