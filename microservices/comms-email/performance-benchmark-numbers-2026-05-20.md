# comms-email performance benchmark numbers — 2026-05-20

µservice: `comms-email`
Counterparts: SendGrid / Postmark / Mailgun
Target posture: one industry-leader target set, not capability-tier-segmented
Tenant-class overlays: `demo_trial`, `paid`, `revenue_share`
Deployment-context overlays: six canonical contexts plus OCI Always Free profile constraints
No retired tier headings or rows are used.

## Citation anchor block

1. Canonical six deployment contexts: `specs/master-plan-sequencing.json:704-745`.
2. Canonical OpenTofu, OS, Rust, and OCI profile constraints: `specs/master-plan-sequencing.json:747-867`.
3. Local send latency SLO: `microservices/comms-email/slos/send-latency-p99.openslo.yaml:12-44`.
4. Local capacity model: `microservices/comms-email/capacity-model.md:12-64`.
5. Counterpart public sources: SendGrid Mail Send limits, Postmark batch/webhook/retention docs, Mailgun batch/webhook/SLA docs.

## §1 Methodology

1. Benchmark dimensions are API accept latency, provider dispatch latency, delivery telemetry latency, webhook ingestion, suppression lookup, template render, DKIM signing, throughput, batch ceiling, and retention.
2. Test workload W1 is one-recipient transactional send with cached template and clean suppression status.
3. Test workload W2 is 1000-recipient personalized bulk send split by provider constraints.
4. Test workload W3 is webhook flood from delivery, bounce, complaint, open, click, and unsubscribe events.
5. Test workload W4 is inbound parse/quarantine if the product boundary keeps inbound receiving.
6. Test workload W5 is suppression import and hot-path lookup under a 50M-row mature list.
7. Test workload W6 is DKIM scheduled rotation and emergency revocation.
8. OS disclosure: no local `supported-oses.json` exists, so OS-specific benchmark claims cannot yet be closed.
9. Architecture disclosure: Rust backend is required, but local runtime crates are not in this µservice path.
10. Evidence: `src/README.md:1-11`, `src/README.md:45-47`.
11. Deployment-context disclosure: this report assumes all six contexts remain target contexts because the user prompt says all six unless audit finds otherwise.
12. Deployment-context caveat: the required per-context OpenTofu modules are absent.
13. Evidence: `find microservices/comms-email/iac -maxdepth 3 -type d | sort`.
14. Tenant-class disclosure: current service artifacts do not declare `tenant_class`.
15. Evidence: `rg tenant_class|demo_trial|revenue_share|Always Free|always-free|paid microservices/comms-email`.
16. Target numbers below are requirements for future implementation and acceptance tests.
17. Counterpart numbers are public API ceilings, public SLA claims, or local benchmark estimates clearly labeled by source.
18. Vendor latency p50/p95/p99 is rarely public; this report does not invent provider latency as a confirmed public number.
19. Where local benchmark data is used, it is labeled as local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md`.
20. Where public source data is used, the source URL and line range are shown.
21. Canonical Oyatie targets must meet the top public counterpart ceiling where practical.
22. Canonical Oyatie targets may intentionally exceed counterpart surfaces where Oyatie owns a self-hosted substrate.
23. Canonical Oyatie targets may be constrained by deployment context.
24. Canonical Oyatie targets may be constrained by tenant class through usage caps, not quality degradation.
25. `demo_trial` overlay means OCI Always Free profile and hard usage caps.
26. `paid` overlay means contractual SLO and elastic scaling where deployment context supports it.
27. `revenue_share` overlay means at-cost substrate economics with the same quality bar and commercial scale gates.
28. All p99 targets refer to service-side enqueue/provider-accept behavior, not final recipient inbox arrival.
29. Final recipient arrival is dominated by recipient MX behavior and cannot be guaranteed as a pure service latency.
30. All deliverability targets assume warmed domains/IPs, valid consent basis, clean lists, and receiver feedback loops.
31. Warmup behavior is explicit in onboarding.
32. Evidence: `onboarding/deliverability-engineer-first-week.md:55-94`.
33. Audit-chain latency target is local event creation to audit-chain seal.
34. Evidence: `slos/audit-chain-emit-lag-p99.openslo.yaml:12-37`.
35. Webhook success target is webhook event to audit-chain first-attempt success.
36. Evidence: `slos/webhook-success-rate.openslo.yaml:12-36`.
37. Suppression lookup target is p99 <= 5 ms.
38. Evidence: `slos/suppression-lookup-latency-p99.openslo.yaml:12-36`.
39. Send success target is provider-accept success >= 99.9%.
40. Evidence: `slos/send-success-rate.openslo.yaml:12-40`.

## §2 Counterpart numbers

### §2.1 SendGrid public and local benchmark numbers

1. SendGrid Mail Send API base URL for global users is `https://api.sendgrid.com`.
2. Source: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 303-310.
3. SendGrid Mail Send API base URL for regional EU subusers is `https://api.eu.sendgrid.com`.
4. Source: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 303-310.
5. SendGrid maximum personalizations per request: 1000.
6. Source: `https://www.twilio.com/docs/sendgrid/for-developers/sending-email/personalizations`, lines 224-226.
7. SendGrid maximum recipients per request: 1000.
8. Source: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 390-392.
9. SendGrid maximum message size including attachments: less than 30 MB.
10. Source: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 390-393.
11. SendGrid `reply_to_list` maximum per request: 1000.
12. Source: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 373-381.
13. SendGrid custom arguments total length limit: less than 10,000 bytes.
14. Source: `https://www.twilio.com/docs/sendgrid/api-reference/mail-send`, lines 390-394.
15. SendGrid inbound parse message-size limit: 30 MB.
16. Source: `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`, lines 149-151.
17. SendGrid inbound parse retry retention before drop: 3 days.
18. Source: `https://www.twilio.com/docs/sendgrid/for-developers/parsing-email/inbound-email`, lines 112-119.
19. SendGrid inbound parse first-hour retry cadence on 5xx: every 5 to 10 minutes.
20. Source: `https://support.sendgrid.com/hc/en-us/articles/46513815674395-Understanding-Inbound-Parse-Webhook-Retry-Logic`, lines 31-40.
21. SendGrid inbound parse hours 2 through 72 retry cadence: approximately every 3 hours.
22. Source: `https://support.sendgrid.com/hc/en-us/articles/46513815674395-Understanding-Inbound-Parse-Webhook-Retry-Logic`, lines 35-40.
23. SendGrid local benchmark API submit p50: 35 ms.
24. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:17-30`.
25. SendGrid local benchmark API submit p99: 145 ms.
26. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:17-30`.
27. SendGrid local benchmark queue-to-MX-ACK p50: 2.2 s.
28. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:32-43`.
29. SendGrid local benchmark queue-to-MX-ACK p95: 18 s.
30. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:32-43`.

### §2.2 Postmark public and local benchmark numbers

31. Postmark batch endpoint maximum messages per API call: 500.
32. Source: `https://postmarkapp.com/developer/user-guide/send-email-with-api`, lines 352-355.
33. Postmark batch endpoint maximum payload size including attachments: 50 MB.
34. Source: `https://postmarkapp.com/developer/user-guide/send-email-with-api`, lines 354-355.
35. Postmark server stream limit: up to 10 streams in a server.
36. Source: `https://postmarkapp.com/message-streams`, lines 167-174.
37. Postmark default activity and content retention: 45 days.
38. Source: `https://postmarkapp.com/support/article/how-does-the-retention-add-on-work`, lines 132-136.
39. Postmark custom retention lower bound: 7 days.
40. Source: `https://postmarkapp.com/support/article/how-does-the-retention-add-on-work`, lines 134-142.
41. Postmark custom retention upper bound: 365 days.
42. Source: `https://postmarkapp.com/support/article/how-does-the-retention-add-on-work`, lines 134-142.
43. Postmark UI CSV export limit for message events including bounces: 500 records.
44. Source: `https://postmarkapp.com/support/article/881-can-i-export-a-list-of-all-bounces`, lines 134-148.
45. Postmark Bounce API programmatic bounce lookback: past 45 days.
46. Source: `https://postmarkapp.com/support/article/881-can-i-export-a-list-of-all-bounces`, lines 146-149.
47. Postmark bounce/inbound webhook retry schedule includes 1 minute, 5 minutes, three 10-minute retries, 15 minutes, 30 minutes, 1 hour, 2 hours, and 6 hours.
48. Source: `https://postmarkapp.com/developer/webhooks/webhooks-overview`, lines 206-222.
49. Postmark click/open/delivered/subscription webhook retry schedule includes 1, 5, and 15 minutes.
50. Source: `https://postmarkapp.com/developer/webhooks/webhooks-overview`, lines 222-229.
51. Postmark local benchmark API submit p50: 18 ms.
52. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:17-30`.
53. Postmark local benchmark API submit p99: 68 ms.
54. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:17-30`.
55. Postmark local benchmark queue-to-MX-ACK p50: 1.5 s.
56. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:32-43`.
57. Postmark local benchmark queue-to-MX-ACK p95: 9 s.
58. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:32-43`.

### §2.3 Mailgun public and local benchmark numbers

59. Mailgun maximum message size: 25 MB.
60. Source: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-http`, lines 116-122.
61. Mailgun supports GZIP-compressed HTTP bodies, but uncompressed message-size limit still applies.
62. Source: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/send-http`, lines 116-122.
63. Mailgun batch sending maximum recipients: 1000.
64. Source: `https://documentation.mailgun.com/docs/mailgun/user-manual/sending-messages/batch-sending`, lines 147-150.
65. Mailgun events API entry limit per request: 300 max.
66. Source: `https://documentation.mailgun.com/docs/mailgun/api-reference/send/mailgun/events/get-v3-domain_name-events`, lines 762-765.
67. Mailgun event retention minimum through Events API: at least 3 days.
68. Source: `https://documentation.mailgun.com/docs/mailgun/api-reference/send/mailgun/events/get-v3-domain_name-events`, lines 694-728.
69. Mailgun inbound store action temporary storage: up to 3 days.
70. Source: `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/route-actions`, lines 123-130.
71. Mailgun product SLA attempted delivery number: up to 15 million messages within first five minutes.
72. Source: `https://www.mailgun.com/products/send/`, lines 164-169.
73. Mailgun product throughput SLA number: up to 72 million message requests per hour.
74. Source: `https://www.mailgun.com/products/send/`, lines 164-170.
75. Mailgun product throughput equivalent: 1.2 million message requests per minute.
76. Source: `https://www.mailgun.com/products/send/`, lines 164-170.
77. Mailgun routing action set count: 3 route actions, `forward`, `store`, and `stop`.
78. Source: `https://documentation.mailgun.com/docs/mailgun/user-manual/receive-forward-store/route-actions`, lines 105-135.
79. Mailgun local benchmark API submit p50: 32 ms.
80. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:17-30`.
81. Mailgun local benchmark API submit p99: 132 ms.
82. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:17-30`.
83. Mailgun local benchmark queue-to-MX-ACK p50: 2.5 s.
84. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:32-43`.
85. Mailgun local benchmark queue-to-MX-ACK p95: 22 s.
86. Source: local estimate from `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:32-43`.

## §3 Oyatie target numbers — single industry-leader target set

87. Canonical API submit p50 target: <= 20 ms.
88. Basis: beats SendGrid local estimate 35 ms and Mailgun local estimate 32 ms; near Postmark local estimate 18 ms.
89. Canonical API submit p95 target: <= 80 ms.
90. Basis: aligns with local capacity target and keeps margin under SendGrid/Mailgun local p99 estimates.
91. Canonical API submit p99 target: <= 120 ms service-side before provider call.
92. Deployment overlay: public cloud can autoscale API replicas to hold this target.
93. Deployment overlay: guest-on-aws depends on tenant AWS quota and regional SES latency.
94. Deployment overlay: guest-on-oci depends on OCI profile shape and outbound relay location.
95. Deployment overlay: on-prem depends on facility network path to provider or local Postal.
96. Deployment overlay: colo depends on IP reputation and peering to recipient MX clusters.
97. Deployment overlay: Oyatie-as-cloud-provider can bind cell placement and route to closest relay.
98. Tenant overlay: `demo_trial` keeps target quality but has hard volume caps.
99. Tenant overlay: `paid` receives contractual SLO and scale entitlement.
100. Tenant overlay: `revenue_share` receives same quality with at-cost economics and gross-revenue usage controls.
101. Canonical enqueue-to-provider-accept p99 target: <= 500 ms.
102. Evidence: `slos/send-latency-p99.openslo.yaml:12-44`.
103. Deployment overlay: OCI Always Free profile target remains <= 500 ms while throughput is capped.
104. Deployment overlay: on-prem local Postal may beat provider latency but facility DNS/TLS path can dominate.
105. Tenant overlay: `demo_trial` enforces send caps before violating latency.
106. Tenant overlay: `paid` scales replicas and provider quotas.
107. Tenant overlay: `revenue_share` scales when usage supports at-cost capacity.
108. Canonical preflight p99 target: <= 10 ms.
109. Evidence: `capacity-model.md:45-56`.
110. Canonical cached template render p99 target: <= 1 ms.
111. Evidence: `capacity-model.md:45-56`.
112. Canonical template compile miss p99 target: <= 50 ms.
113. Evidence: `capacity-model.md:45-56`.
114. Canonical Liquid substitution p99 target: <= 5 ms.
115. Evidence: `capacity-model.md:45-56`.
116. Canonical suppression lookup p99 target: <= 5 ms.
117. Evidence: `slos/suppression-lookup-latency-p99.openslo.yaml:12-36`.
118. Canonical DKIM sign p99 target: <= 5 ms hot path.
119. Evidence: `capacity-model.md:45-56`.
120. Canonical provider call p99 budget: <= 400 ms.
121. Evidence: `capacity-model.md:45-56`.
122. Canonical send success target: >= 99.9%.
123. Evidence: `slos/send-success-rate.openslo.yaml:12-40`.
124. Canonical DKIM signed send target: >= 99.99%.
125. Evidence: `slos/dkim-signing-rate.openslo.yaml:12-34`.
126. Canonical DMARC alignment target after warmup: >= 99%.
127. Evidence: `slos/dmarc-alignment-rate.openslo.yaml:12-35`.
128. Canonical receiver-delivery event target within 24h: >= 99.5%.
129. Evidence: `slos/deliverability-rate.openslo.yaml:12-34`.
130. Canonical webhook-to-audit first-attempt success: >= 99.99%.
131. Evidence: `slos/webhook-success-rate.openslo.yaml:12-36`.
132. Canonical audit-chain emit lag p99: <= 5 s.
133. Evidence: `slos/audit-chain-emit-lag-p99.openslo.yaml:12-37`.
134. Canonical from-domain onboarding p95: <= 24 h.
135. Evidence: `slos/from-domain-onboarding-time.openslo.yaml:12-37`.
136. Canonical sends per second per cluster p50 target: 100.
137. Evidence: `capacity-model.md:12-19`.
138. Canonical sends per second per cluster p99 target: 1000.
139. Evidence: `capacity-model.md:12-19`.
140. Canonical sends per second per cluster peak target: 10,000.
141. Evidence: `capacity-model.md:12-19`.
142. Canonical headroom target at peak: 2x, or 20,000 sends/s capacity at 10,000 sends/s peak.
143. Evidence: `capacity-model.md:58-64`.
144. Canonical concurrent template renders per cluster: <= 200.
145. Evidence: `capacity-model.md:12-19`.
146. Canonical webhook ingest QPS planning factor: 3x to 4x send QPS.
147. Evidence: `capacity-model.md:17-18`.
148. Canonical suppression list scale target: 50M rows mature baseline.
149. Evidence: `capacity-model.md:30-36`.
150. Canonical idempotency store live rows: 10M rows at 1-hour TTL.
151. Evidence: `capacity-model.md:30-36`.
152. Canonical audit emission buffer: <= 5 minutes and <= 1 GB burst.
153. Evidence: `capacity-model.md:35-36`.
154. Canonical batch recipient target: 1000 recipients per API batch.
155. Basis: matches SendGrid recipient max and Mailgun batch max; exceeds Postmark 500-message batch only when semantics are recipient batch, not message array.
156. Deployment overlay: demo profile can reduce allowed batch size to preserve free resource envelope.
157. Tenant overlay: `demo_trial` maximum batch should be capped by policy before queue pressure.
158. Tenant overlay: `paid` maximum batch can be 1000 with provider-specific splitting.
159. Tenant overlay: `revenue_share` maximum batch can be 1000 when usage economics are in bounds.
160. Canonical max accepted outbound message size: 25 MB default.
161. Basis: aligns to Mailgun 25 MB and common receiver constraints; SendGrid/Postmark can exceed this, but larger messages create deliverability risk.
162. Deployment overlay: on-prem/colo can set lower limits if facility egress requires it.
163. Tenant overlay: `demo_trial` should cap lower, likely 5 MB, to fit OCI Always Free profile.
164. Canonical inbound parse size target if inbound remains in scope: 25 MB default.
165. Basis: conservative intersection of SendGrid 30 MB, Postmark payload 50 MB, and Mailgun 25 MB.
166. Canonical event query page size target: 300 minimum.
167. Basis: matches Mailgun event API max entries per request.
168. Canonical event retention target: 90 days full resolution.
169. Evidence: `dpia.md:82-83`.
170. Deployment overlay: `demo_trial` may retain less full-resolution telemetry while preserving required audit evidence.
171. Tenant overlay: `paid` can buy longer full-resolution operational retention.
172. Tenant overlay: `revenue_share` retention follows at-cost economics and compliance pack.
173. Canonical audit-chain retention target: 7 years default where ADR-0145 applies.
174. Evidence: `dpia.md:77-78`.
175. Canonical DKIM emergency revocation target: <= 5 minutes.
176. Evidence: `decisions/SVC-ADR-001-dkim-cadence.md:16-24`.
177. Canonical scheduled DKIM overlap: 14 days.
178. Evidence: `decisions/SVC-ADR-001-dkim-cadence.md:16-24`.
179. Canonical webhook retry before DLQ: 8 retries.
180. Evidence: `decisions/SVC-ADR-003-webhook-retry-policy.md:15-24`.
181. Canonical worst-case webhook time to DLQ: about 8.5 minutes.
182. Evidence: `decisions/SVC-ADR-003-webhook-retry-policy.md:35-38`.
183. Canonical DLQ alert: >100 entries in 5 minutes and >10,000 absolute.
184. Evidence: `decisions/SVC-ADR-003-webhook-retry-policy.md:20-24`.
185. Canonical provider outage failover target: 5-minute mean time to failover where pack permits.
186. Evidence: `failure-modes.md:7-14`.
187. Canonical audit-chain outage buffer: <= 5 minutes, then reject new sends.
188. Evidence: `failure-modes.md:69-74`.
189. Canonical complaint surge threshold: >0.1%.
190. Evidence: `failure-modes.md:96-101`.
191. Canonical bounce storm threshold: hard-bounce rate >5% per tenant per hour.
192. Evidence: `incident-response.md:47-60`.
193. Canonical blacklist trigger: receiver block-rate >5% or external listing.
194. Evidence: `incident-response.md:32-46`.
195. Canonical warmup day 30 target from onboarding drill: 500,000 sends/day.
196. Evidence: `onboarding/deliverability-engineer-first-week.md:55-94`.
197. Canonical warmup bounce-rate gate: <= 2%.
198. Evidence: `onboarding/deliverability-engineer-first-week.md:85-92`.
199. Canonical warmup spam-complaint gate: <= 0.05%.
200. Evidence: `onboarding/deliverability-engineer-first-week.md:85-92`.

## §4 Comparison narrative

201. API submit latency target is ahead of SendGrid and Mailgun local estimates and near Postmark local estimate.
202. API submit latency status: parity/ahead if the future Rust crates meet <=20 ms p50 and <=120 ms p99.
203. Enqueue-to-provider-accept p99 target is a local Oyatie SLO, not a direct public vendor number.
204. Enqueue-to-provider-accept status: credible only after implementation tests land.
205. Batch recipient target of 1000 matches SendGrid and Mailgun public limits.
206. Batch recipient status: ahead of Postmark's 500-message batch where recipient-batch semantics apply.
207. Outbound message size target of 25 MB is conservative against SendGrid 30 MB and Postmark 50 MB.
208. Outbound message size status: catch-up only if the product needs large attachments; otherwise deliberate deliverability guardrail.
209. Inbound parse size target of 25 MB is conservative against SendGrid 30 MB and Mailgun 25 MB.
210. Inbound parse status: blocked by product-boundary decision and missing contracts.
211. Event retention target of 90 days full resolution beats Mailgun minimum event retention and Postmark default 45 days.
212. Event retention status: ahead if storage and query APIs are implemented.
213. Audit-chain retention of 7 years is ahead of counterpart operational logs because it is compliance evidence, not activity-feed storage.
214. Audit-chain status: ahead by design, dependent on audit-chain availability.
215. Webhook success target >=99.99% is stronger than public retry-shape claims.
216. Webhook status: credible because SLO and retry ADR exist, but needs executable tests.
217. Webhook time-to-DLQ at about 8.5 minutes is faster than SendGrid inbound parse's 3-day eventual drop.
218. Webhook status: ahead for internal webhook-to-audit events, not comparable to provider inbound parse durability.
219. DKIM emergency revocation <=5 minutes is an industry-leader operational target.
220. DKIM status: ahead if OpenBao/HSM and DNS automation prove it.
221. Provider outage failover target of 5 minutes is strong for multi-provider operation.
222. Provider failover status: partial because context-specific IaC modules are missing.
223. Cluster peak of 10,000 sends/s is below Mailgun's public throughput claim of 1.2M requests/min.
224. Throughput status: catch-up for hyperscale bulk marketing, adequate for canonical transactional substrate.
225. Headroom target of 20,000 sends/s at peak is strong for initial Oyatie cells.
226. Headroom status: credible only after implementation and load tests.
227. Suppression lookup p99 <=5 ms is strong and necessary for hot path.
228. Suppression status: credible as SLO/capacity target, not executable yet.
229. Template compile miss p99 <=50 ms is strong for MJML/Liquid.
230. Template status: needs real Rust renderer benchmarks.
231. From-domain onboarding p95 <=24h is pragmatic because DNS propagation dominates.
232. Onboarding status: parity with provider-domain verification workflows.
233. Deliverability >=99.5% is high but receiver-dependent.
234. Deliverability status: valid only for warmed, compliant, clean-list traffic.
235. DMARC alignment >=99% after warmup is credible with enforced SPF/DKIM/DMARC.
236. DMARC status: parity/ahead if RUA ingestion and alerting land.
237. Warmup day-30 500k sends/day target is conservative against large counterpart batch systems.
238. Warmup status: good for deliverability quality, not maximum throughput leadership.
239. Demo tenant-class overlay should never relax latency or correctness targets.
240. Demo overlay status: caps volume and retention, not quality.
241. Paid tenant-class overlay should keep full contractual scale where context supports it.
242. Paid overlay status: blocked until tenant-class schema exists.
243. Revenue-share tenant-class overlay should use at-cost scaling while preserving quality.
244. Revenue-share overlay status: blocked until billing/usage policy exists.
245. Public-cloud context should meet elastic targets after OpenTofu modules and autoscaling evidence land.
246. Public-cloud status: blocked by missing context module.
247. Guest-on-AWS context should meet targets when SES/Mailgun/Postal quotas are configured.
248. Guest-on-AWS status: blocked by missing context module.
249. Guest-on-OCI context should include OCI Always Free profile caps and paid OCI scaling path.
250. Guest-on-OCI status: blocked by missing `iac/oci-guest/always-free/`.
251. On-prem context should favor local Postal and explicit egress/DNS policy.
252. On-prem status: blocked by missing context module and facility assumptions.
253. Colo context should include IP reputation, peering, DNS, and MTA isolation assumptions.
254. Colo status: blocked by missing context module and acceptance tests.
255. Oyatie-as-cloud-provider context should bind cloud-iac, cell, and provider-neutral routing.
256. Oyatie-as-cloud-provider status: blocked by missing `iac/oyatie-iaas/`.
257. Overall performance posture is strong as a target model.
258. Overall performance proof is incomplete because local implementation, tests, and context IaC are absent.
259. The old benchmark report should be retired or rewritten because it labels targets with retired tier vocabulary.
260. Evidence: `benchmarks/comms-email-vs-sendgrid-vs-mailgun-vs-ses-vs-postmark.md:13-111`.
261. The replacement acceptance suite should include W1 through W6 workloads from §1.
262. The replacement acceptance suite should run per deployment context.
263. The replacement acceptance suite should produce tenant-class overlay evidence.
264. The replacement acceptance suite should produce OS matrix evidence after `supported-oses.json` exists.
265. The replacement acceptance suite should produce provider split evidence for SES, Postal, Mailgun, and SMTP fallback.
266. The replacement acceptance suite should produce inbound/list/reputation evidence only if those surfaces remain in scope.
267. No performance claim in this report should be treated as production-proven until the future implementation crates are linked and benchmarked.
268. No counterpart latency estimate in this report should be treated as a public vendor guarantee unless it is explicitly sourced from vendor public docs.
269. The clearest immediate performance remediation is to replace retired benchmark rows with this single target set.
270. The next remediation is to add load-test fixtures and OpenTofu context modules.

## §5 Acceptance numbers summary

271. API submit p50: <=20 ms.
272. API submit p95: <=80 ms.
273. API submit p99 service-side before provider call: <=120 ms.
274. Enqueue-to-provider-accept p99: <=500 ms.
275. Preflight p99: <=10 ms.
276. Cached template render p99: <=1 ms.
277. Template compile miss p99: <=50 ms.
278. Liquid substitution p99: <=5 ms.
279. Suppression lookup p99: <=5 ms.
280. DKIM sign p99 hot path: <=5 ms.
281. Provider call p99 budget: <=400 ms.
282. Send success: >=99.9%.
283. DKIM signed send rate: >=99.99%.
284. DMARC alignment after warmup: >=99%.
285. Delivery event within 24h: >=99.5%.
286. Webhook-to-audit first-attempt success: >=99.99%.
287. Audit-chain emit lag p99: <=5 s.
288. From-domain onboarding p95: <=24 h.
289. Cluster p50 sends/s: 100.
290. Cluster p99 sends/s: 1000.
291. Cluster peak sends/s: 10,000.
292. Peak headroom capacity: 20,000 sends/s.
293. Concurrent template renders per cluster: <=200.
294. Webhook ingest planning factor: 3x to 4x send QPS.
295. Suppression-list mature scale: 50M rows.
296. Idempotency store live scale: 10M rows at 1-hour TTL.
297. Audit emission buffer: <=5 minutes and <=1 GB burst.
298. Batch recipient target: 1000 recipients.
299. Outbound message size target: 25 MB default.
300. Inbound parse target if retained: 25 MB default.
301. Event query page size target: 300 entries minimum.
302. Full-resolution event retention target: 90 days.
303. Audit-chain retention target: 7 years where ADR-0145 applies.
304. DKIM emergency revocation: <=5 minutes.
305. Scheduled DKIM overlap: 14 days.
306. Webhook retry budget before DLQ: 8 retries.
307. Webhook time-to-DLQ: about 8.5 minutes.
308. DLQ alert: >100 entries in 5 minutes or >10,000 absolute.
309. Provider outage failover target where pack permits: 5 minutes.
310. Audit-chain outage buffer before rejecting new sends: <=5 minutes.
311. Complaint surge threshold: >0.1%.
312. Bounce storm threshold: >5% hard bounces per tenant per hour.
313. Blacklist trigger: receiver block-rate >5% or external listing.
314. Warmup day-30 target: 500,000 sends/day.
315. Warmup bounce-rate gate: <=2%.
316. Warmup spam-complaint gate: <=0.05%.
317. `demo_trial` overlay: enforce usage caps before quality degradation.
318. `paid` overlay: scale replicas, provider quota, and retention to contractual plan.
319. `revenue_share` overlay: scale at cost while preserving the same quality floor.
320. Missing proof blocker: no local runtime crates/tests in the µservice path.
321. Missing context blocker: no six-context OpenTofu modules.
322. Missing profile blocker: no OCI Always Free profile module.
323. Missing OS blocker: no service-level supported OS manifest.
324. Missing tenant-class blocker: no `tenant_class` schema or policy overlay.
325. Retired benchmark blocker: old benchmark uses retired tier vocabulary.
326. Final benchmark disposition: target model accepted as audit requirement; implementation proof remains open.
