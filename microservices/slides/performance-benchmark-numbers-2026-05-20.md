# Slides performance benchmark numbers, 2026-05-20

Audited microservice: `microservices/slides/`.
Counterparts: Google Slides, Microsoft PowerPoint Online, Pitch.
Benchmark model: single industry-leader target set, with deployment-context overlays and tenant-class usage overlays.
No capability-schema rows are used in this document.
Methodology disclosure: public vendors do not publish full p50/p95/p99 deck-open, cursor-sync, or save-latency SLOs for these products, so counterpart latency numbers below are estimates from public feature/limit documentation plus the service-local benchmark artifact, while API quota and feature-limit numbers cite public sources directly.
Source A: `microservices/slides/PRD.md:90-108` gives current Oyatie performance targets.
Source B: `microservices/slides/PRD.md:426-438` gives current Oyatie capacity envelopes.
Source C: `microservices/slides/benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:17-95` gives the existing local benchmark artifact, but that artifact contains retired commercial labels and is used here only as a historical estimate source.
Source D: Google Slides public API limits: `https://developers.google.com/workspace/slides/api/limits`.
Source E: Microsoft Graph throttling guidance: `https://learn.microsoft.com/en-us/graph/throttling-limits`.
Source F: Pitch public help for import/export, analytics, AI, and offline mode: `https://help.pitch.com/en/articles/4615453-import-a-presentation`, `https://help.pitch.com/en/articles/6713988-export-a-presentation-to-power-point`, `https://help.pitch.com/en/articles/5592127-view-presentation-analytics`, `https://help.pitch.com/en/articles/8541722-start-a-new-presentation-with-ai`, and `https://help.pitch.com/en/articles/5671537-work-offline-in-pitch`.

## Section 1 - Methodology

1. Benchmark dimension D-001: deck-open latency for a 50-slide and 100-slide deck.
2. Benchmark dimension D-002: warm deck-open latency after assets and CRDT state are cached.
3. Benchmark dimension D-003: slide render-to-display latency.
4. Benchmark dimension D-004: text/shape/cell edit-to-render latency.
5. Benchmark dimension D-005: save-delta persistence latency.
6. Benchmark dimension D-006: cursor-presence sync latency.
7. Benchmark dimension D-007: CRDT merge conflict surfacing latency.
8. Benchmark dimension D-008: present-mode transition latency.
9. Benchmark dimension D-009: present-mode per-frame budget.
10. Benchmark dimension D-010: broadcast signaling latency.
11. Benchmark dimension D-011: broadcast viewer scale per deck.
12. Benchmark dimension D-012: concurrent active editor sessions per cell.
13. Benchmark dimension D-013: API write throughput for deck operations.
14. Benchmark dimension D-014: PDF export latency for a 50-slide deck.
15. Benchmark dimension D-015: PPTX export latency for a 50-slide deck.
16. Benchmark dimension D-016: MP4 export latency per slide.
17. Benchmark dimension D-017: PPTX round-trip fidelity over the supported subset.
18. Benchmark dimension D-018: AI full-deck generation throughput.
19. Benchmark dimension D-019: link analytics event ingest throughput.
20. Benchmark dimension D-020: tenant usage-cap enforcement latency.
21. Test workload W-001: 50-slide text-heavy deck, 5 collaborators, no embedded video.
22. Test workload W-002: 100-slide mixed-media deck, 20 collaborators, 20 images, 10 charts, and 3 videos.
23. Test workload W-003: board/investor deck with linked Sheets charts, comments, and audience Q&A.
24. Test workload W-004: sales-broadcast deck with 500 and 5,000 audience viewers.
25. Test workload W-005: PowerPoint import/export corpus with standard OOXML shapes, charts, tables, media, and speaker notes.
26. Test workload W-006: AI prompt-to-deck generation from a 2,000-word outline.
27. OS disclosure O-001: service-local OS matrix is absent, so OS-specific targets are provisional until `supported_oses.json` exists.
28. Architecture disclosure A-001: current service-local IaC lacks OpenTofu context directories, so context overlays are target overlays rather than measured OpenTofu results.
29. Deployment context C-001: `oyatie-public-cloud` target assumes elastic managed substrate with multi-region capacity.
30. Deployment context C-002: `guest-on-aws` target assumes customer-owned AWS substrate and account-level quota constraints.
31. Deployment context C-003: `guest-on-oci` target assumes customer-owned OCI substrate, including an OCI Always Free profile for demo-trial infrastructure.
32. Deployment context C-004: `on-prem` target assumes tenant facility/network constraints may dominate broadcast and export throughput.
33. Deployment context C-005: `colo` target assumes fixed rack, power, GPU, and network capacity.
34. Deployment context C-006: `oyatie-as-cloud-provider` target assumes Oyatie-operated cloud-provider cell with elasticity comparable to public cloud only where capacity is allocated.
35. Tenant-class T-001: `demo_trial` target uses the same product quality but strict usage caps.
36. Tenant-class T-002: `paid` target scales with per-seat and usage-based entitlements.
37. Tenant-class T-003: `revenue_share` target scales at cost or zero-margin substrate where gross-revenue share justifies capacity.
38. Measurement disclosure M-001: public counterpart latencies are estimates unless a cited source publishes a numeric limit.
39. Measurement disclosure M-002: Google API quota numbers are public source numbers, not deck UI latency measurements.
40. Measurement disclosure M-003: Microsoft Graph throttling guidance is public source guidance, not a PowerPoint Online UI latency measurement.
41. Measurement disclosure M-004: Pitch help-center feature limits are public source numbers, not infrastructure SLOs.
42. Measurement disclosure M-005: the retired local benchmark artifact is used for historical estimates only and must not carry commercial level names forward.
43. Quality rule Q-001: no target in this document intentionally lowers feature quality by tenant class.
44. Quality rule Q-002: context overlays may cap throughput where infrastructure is physically constrained.
45. Quality rule Q-003: tenant-class overlays may cap monthly usage or concurrency, not quality of the delivered feature.
46. Quality rule Q-004: benchmark claims must be replaced by live harness results once implementation and OpenTofu context modules exist.

## Section 2 - Counterpart numbers

### 2.1 Google Slides numbers

47. Google number G-001: Slides API read requests per minute per project are publicly limited; source: Google Slides API limits.
48. Google number G-002: Slides API write requests per minute per project are publicly limited; source: Google Slides API limits.
49. Google number G-003: Slides API expensive read requests per minute per project are publicly limited; source: Google Slides API limits.
50. Google number G-004: Slides API per-user quota is lower than per-project quota; source: Google Slides API limits.
51. Google number G-005: Deck-open cold latency estimate for 100-slide deck: p50 380 ms, p99 950 ms; source: estimated from local historical benchmark `benchmarks/...md:17-31`, not a Google-published SLO.
52. Google number G-006: Slide render-to-display estimate: p50 65 ms, p99 145 ms; source: estimated from local historical benchmark `benchmarks/...md:33-43`.
53. Google number G-007: Cursor sync estimate: p50 95 ms, p99 220 ms; source: estimated from local historical benchmark `benchmarks/...md:45-55`.
54. Google number G-008: PDF export estimate for 50-slide deck: about 5 s; source: estimated from local parity table `competitor-parity-matrix.md:127`.
55. Google number G-009: PPTX export/reopen fidelity estimate: 87 percent; source: estimated from local historical benchmark `benchmarks/...md:57-68`.
56. Google number G-010: AI generation throughput estimate: 8 slides/min; source: estimated from local historical benchmark `benchmarks/...md:70-80`.
57. Google number G-011: Audience Q&A is supported in presentation mode; source: Google Slides Q&A help.
58. Google number G-012: Linked Sheets chart update is supported; source: Google linked chart help.
59. Google number G-013: Public p95/p99 cursor sync and deck-open SLOs were not found in public Google docs during this audit.
60. Google number G-014: Google-linked chart parity requires Oyatie to support update propagation and permission revocation, not just static chart rendering.
61. Google number G-015: Google API quota availability means Oyatie should publish comparable API usage caps for demo-trial and paid tenant classes.

### 2.2 Microsoft PowerPoint Online numbers

62. Microsoft number M-001: PowerPoint for the web supports browser editing for presentations stored in Microsoft cloud storage; source: Microsoft Learn service description.
63. Microsoft number M-002: Graph throttling exists across Microsoft Graph and varies by scenario; source: Microsoft Graph throttling guidance.
64. Microsoft number M-003: Deck-open cold latency estimate for 100-slide deck: p50 320 ms, p99 820 ms; source: estimated from local historical benchmark `benchmarks/...md:17-31`.
65. Microsoft number M-004: Slide render-to-display estimate: p50 78 ms, p99 168 ms; source: estimated from local historical benchmark `benchmarks/...md:33-43`.
66. Microsoft number M-005: Cursor sync estimate: p50 145 ms, p99 320 ms; source: estimated from local historical benchmark `benchmarks/...md:45-55`.
67. Microsoft number M-006: PPTX round-trip fidelity estimate for PowerPoint Online: 97 percent; source: estimated from local historical benchmark `benchmarks/...md:57-68`.
68. Microsoft number M-007: PowerPoint desktop native PPTX fidelity estimate: 99 percent; source: estimated from local historical benchmark `benchmarks/...md:57-68`; included as an upper benchmark, not the web counterpart.
69. Microsoft number M-008: PDF export estimate for 50-slide deck: about 4 s; source: estimated from local parity table `competitor-parity-matrix.md:127`.
70. Microsoft number M-009: PPTX export estimate for 50-slide deck: about 3 s because PPTX is native; source: estimated from local parity table `competitor-parity-matrix.md:128`.
71. Microsoft number M-010: AI/Designer generation throughput estimate: 6 slides/min; source: estimated from local historical benchmark `benchmarks/...md:70-80`.
72. Microsoft number M-011: PowerPoint Live supports audience engagement features, including audience device participation; source: Microsoft Present Live support.
73. Microsoft number M-012: Public p95/p99 deck-open, cursor-sync, and save-delta SLOs were not found in public Microsoft docs during this audit.
74. Microsoft number M-013: Microsoft native PPTX fidelity remains the top catch-up number for Oyatie import/export.
75. Microsoft number M-014: Microsoft web limitations versus desktop must be separated from the desktop product when setting web parity.
76. Microsoft number M-015: Microsoft Graph throttling guidance supports the need for Oyatie public API quota tables, but does not supply a PowerPoint-specific UI throughput number.

### 2.3 Pitch numbers

77. Pitch number P-001: PPTX import is supported; source: Pitch import help.
78. Pitch number P-002: PPTX export is supported; source: Pitch export help.
79. Pitch number P-003: Presentation analytics include visits, viewed slides, visit length, device/browser, and country where available; source: Pitch analytics help.
80. Pitch number P-004: Offline mode supports presentation editing with named limitations; source: Pitch offline help.
81. Pitch number P-005: AI presentation creation from prompt is supported; source: Pitch AI help.
82. Pitch number P-006: Batch creation can create up to 50 presentations per bulk run; source: Pitch batch creation help.
83. Pitch number P-007: Deck-open cold latency estimate for 100-slide deck: p50 220 ms, p99 580 ms; source: estimated from local historical benchmark `benchmarks/...md:17-31`.
84. Pitch number P-008: Slide render-to-display estimate: p50 58 ms, p99 125 ms; source: estimated from local historical benchmark `benchmarks/...md:33-43`.
85. Pitch number P-009: Cursor sync estimate: p50 88 ms, p99 195 ms; source: estimated from local historical benchmark `benchmarks/...md:45-55`.
86. Pitch number P-010: PPTX export/reopen fidelity estimate: 84 percent; source: estimated from local historical benchmark `benchmarks/...md:57-68`.
87. Pitch number P-011: AI generation throughput estimate: 10 slides/min; source: estimated from local historical benchmark `benchmarks/...md:70-80`.
88. Pitch number P-012: Public p95/p99 infrastructure SLOs were not found in Pitch help docs during this audit.
89. Pitch number P-013: Pitch analytics coverage creates an Oyatie target for external-link analytics even though the current PRD does not explicitly require it.
90. Pitch number P-014: Pitch offline support creates an Oyatie decision point for offline editing.
91. Pitch number P-015: Pitch batch generation suggests a target for template-driven bulk deck creation from structured data.

## Section 3 - Oyatie target numbers

### 3.1 Single canonical target set

92. Target O-001 deck-open cold p50: 250 ms; source: `PRD.md:90`.
93. Target O-002 deck-open cold p95: 400 ms; source: `PRD.md:90`.
94. Target O-003 deck-open cold p99: 600 ms for general target; source: `PRD.md:90`.
95. Target O-004 deck-open cold p999: 1.2 s; source: `PRD.md:90`.
96. Target O-005 deck-open warm p50: 80 ms; source: `PRD.md:91`.
97. Target O-006 deck-open warm p95: 150 ms; source: `PRD.md:91`.
98. Target O-007 deck-open warm p99: 250 ms; source: `PRD.md:91`.
99. Target O-008 deck-open warm p999: 500 ms; source: `PRD.md:91`.
100. Target O-009 slide render p50: 50 ms; source: `PRD.md:92`.
101. Target O-010 slide render p95: 100 ms; source: `PRD.md:92`.
102. Target O-011 slide render p99: 150 ms; source: `PRD.md:92`.
103. Target O-012 edit-to-render p50: 20 ms; source: `PRD.md:93`.
104. Target O-013 edit-to-render p95: 40 ms; source: `PRD.md:93`.
105. Target O-014 edit-to-render p99: 50 ms; source: `PRD.md:93`.
106. Target O-015 cursor sync p50: 60 ms; source: `PRD.md:97`.
107. Target O-016 cursor sync p95: 120 ms; source: `PRD.md:97`.
108. Target O-017 cursor sync p99: 150 ms; source: `PRD.md:97`.
109. Target O-018 save delta p50: 50 ms; source: `PRD.md:98`.
110. Target O-019 save delta p95: 100 ms; source: `PRD.md:98`.
111. Target O-020 save delta p99: 200 ms; source: `PRD.md:98`.
112. Target O-021 PDF export 50 slides p50: 1.5 s; source: `PRD.md:99`.
113. Target O-022 PDF export 50 slides p95: 3 s; source: `PRD.md:99`.
114. Target O-023 PDF export 50 slides p99: 5 s; source: `PRD.md:99`.
115. Target O-024 PPTX export 50 slides p50: 2 s; source: `PRD.md:100`.
116. Target O-025 PPTX export 50 slides p95: 5 s; source: `PRD.md:100`.
117. Target O-026 PPTX export 50 slides p99: 8 s; source: `PRD.md:100`.
118. Target O-027 MP4 export p95: slide_count times 1 s plus 5 s overhead; source: `PRD.md:101`.
119. Target O-028 chart render p95: 200 ms; source: `PRD.md:102`.
120. Target O-029 present transition p50: 16 ms; source: `PRD.md:103`.
121. Target O-030 present transition p95: 33 ms; source: `PRD.md:103`.
122. Target O-031 present transition p99: 50 ms; source: `PRD.md:103`.
123. Target O-032 per-frame present budget p99: 16.7 ms; source: `ADR-SLIDES-0002:37` and `ADR-SLIDES-0002:68`.
124. Target O-033 broadcast signaling p95: 150 ms; source: `PRD.md:104`.
125. Target O-034 broadcast signaling p99: 250 ms; source: `PRD.md:104`.
126. Target O-035 active editor sessions baseline: 10,000 per cell; source: `PRD.md:426`.
127. Target O-036 active editor sessions max: 200,000 per cell; source: `PRD.md:426`.
128. Target O-037 WebSocket connections baseline: 50,000 per cell; source: `PRD.md:428`.
129. Target O-038 WebSocket connections max: 500,000 per cell; source: `PRD.md:428`.
130. Target O-039 save RPS baseline: 1,000; source: `PRD.md:430`.
131. Target O-040 save RPS max: 100,000; source: `PRD.md:430`.
132. Target O-041 broadcast viewers per deck baseline: 500; source: `PRD.md:433`.
133. Target O-042 broadcast viewers per deck max: 5,000; source: `PRD.md:433`.
134. Target O-043 broadcast sessions baseline: 100; source: `PRD.md:434`.
135. Target O-044 broadcast sessions max: 5,000; source: `PRD.md:434`.
136. Target O-045 export jobs/sec baseline: 10; source: `PRD.md:436`.
137. Target O-046 export jobs/sec max: 200; source: `PRD.md:436`.
138. Target O-047 AI requests/sec baseline: 5; source: `PRD.md:438`.
139. Target O-048 AI requests/sec max: 200; source: `PRD.md:438`.
140. Target O-049 PPTX round-trip subset pass rate: at least 95 percent; source: `ADR-SLIDES-0003:43-66` and `ADR-SLIDES-0003:191`.
141. Target O-050 PDF/A conformance: blocker CI lane; source: `ADR-SLIDES-0003:195-197`.

### 3.2 Deployment-context overlays

142. Context overlay C-001 `oyatie-public-cloud`: canonical latency targets apply when cell capacity is provisioned.
143. Context overlay C-002 `oyatie-public-cloud`: max active editor sessions and broadcast sessions may scale toward PRD max with elastic cell capacity.
144. Context overlay C-003 `oyatie-public-cloud`: export and AI max throughput require pre-allocated worker pools and GPU capacity where AI/image generation is used.
145. Context overlay C-004 `guest-on-aws`: canonical latency targets apply only after customer account quotas and regional capacity are verified.
146. Context overlay C-005 `guest-on-aws`: save RPS and WebSocket scale are capped by customer VPC, load balancer, Valkey, and managed database quotas.
147. Context overlay C-006 `guest-on-aws`: export throughput scales with provisioned worker nodes and storage egress policy.
148. Context overlay C-007 `guest-on-oci`: canonical targets apply for paid/revenue-share tenants when OCI paid resources are provisioned.
149. Context overlay C-008 `guest-on-oci`: OCI Always Free profile caps demo-trial usage to a small-deck, low-concurrency envelope until `iac/oci-guest/always-free/` exists.
150. Context overlay C-009 `guest-on-oci`: demo-trial target should cap concurrent collaborators at 5, broadcast viewers at 25, export jobs at 1 concurrent, and AI requests at zero or very low quota unless separately funded.
151. Context overlay C-010 `on-prem`: deck-open and cursor-sync targets depend on tenant LAN/WAN quality, browser fleet, and local cluster sizing.
152. Context overlay C-011 `on-prem`: export throughput depends on tenant-provided worker hosts and sandbox runtime.
153. Context overlay C-012 `on-prem`: broadcast viewer scale must be documented per facility network and not assumed from cloud targets.
154. Context overlay C-013 `colo`: low latency is feasible for regional users, but capacity is bounded by rack/power/GPU procurement.
155. Context overlay C-014 `colo`: max sessions and export throughput require facility-specific capacity reservations.
156. Context overlay C-015 `oyatie-as-cloud-provider`: cloud-provider cells can meet canonical targets when Oyatie controls substrate and peering.
157. Context overlay C-016 `oyatie-as-cloud-provider`: at-cost revenue-share deployments should cap throughput by commercial envelope, not degrade feature quality.
158. Context overlay C-017 all contexts: every context needs explicit OpenTofu variables for deck limit, collaborator cap, broadcast cap, export worker cap, AI request cap, and storage budget.
159. Context overlay C-018 all contexts: current service lacks the OpenTofu modules needed to make these overlays enforceable.

### 3.3 Tenant-class overlays

160. Tenant overlay T-001 `demo_trial`: quality target remains industry-leader-grade for the operations allowed.
161. Tenant overlay T-002 `demo_trial`: cap deck count, slides per deck, collaborators per deck, external links, broadcast viewers, exports, and AI calls.
162. Tenant overlay T-003 `demo_trial`: map infrastructure to OCI Always Free profile where possible.
163. Tenant overlay T-004 `demo_trial`: best-effort SLO can cap availability credits, not product correctness.
164. Tenant overlay T-005 `demo_trial`: no compliance packs and no BYOK according to current prompt.
165. Tenant overlay T-006 `paid`: contractual SLOs apply.
166. Tenant overlay T-007 `paid`: per-seat licensing and usage billing scale active editor sessions, broadcast viewers, exports, AI, and storage.
167. Tenant overlay T-008 `paid`: compliance packs and BYOK are allowed.
168. Tenant overlay T-009 `paid`: deployment context may be any of the six contexts when OpenTofu evidence exists.
169. Tenant overlay T-010 `revenue_share`: capacity scales to commercial value of gross revenue share.
170. Tenant overlay T-011 `revenue_share`: at-cost or zero-margin substrate requires hard cost observability on export, AI, broadcast, and storage.
171. Tenant overlay T-012 `revenue_share`: heavy AI generation and MP4 export require explicit commercial guardrails.
172. Tenant overlay T-013 all classes: no feature quality should be degraded by class.
173. Tenant overlay T-014 all classes: current slides docs do not express these overlays.
174. Tenant overlay T-015 all classes: service manifest, capacity model, cost budget, runbooks, and OpenTofu variables need tenant-class fields.

## Section 4 - Comparison narrative

175. Comparison N-001 deck-open: Oyatie p95 400 ms targets parity or better versus estimated Google and Microsoft web numbers, but Pitch's estimated p99 is strong and requires live measurement.
176. Comparison N-002 deck-open: Oyatie's p99 600 ms is ahead of estimated Google 950 ms and Microsoft 820 ms, but near Pitch 580 ms.
177. Comparison N-003 warm open: Oyatie p95 150 ms is aggressive and must be proven with cached CRDT state and asset preloading.
178. Comparison N-004 slide render: Oyatie p95 100 ms is competitive with estimated Google and Microsoft, but p99 150 ms is only slightly better than estimated Google and near Pitch.
179. Comparison N-005 edit-to-render: Oyatie p99 50 ms aims ahead of estimated web counterparts; this depends on Leptos fine-grained reactivity from `ADR-SLIDES-0002:55-68`.
180. Comparison N-006 cursor sync: Oyatie p99 150 ms aims ahead of estimated Google 220 ms, Microsoft 320 ms, and Pitch 195 ms.
181. Comparison N-007 save delta: Oyatie p95 100 ms is aggressive and needs storage/write-path tests.
182. Comparison N-008 PDF export: Oyatie p95 3 s aims ahead of estimated Google 5 s and Microsoft 4 s.
183. Comparison N-009 PPTX export: Oyatie p95 5 s trails Microsoft native estimate around 3 s but is acceptable if subset fidelity is clearly disclosed.
184. Comparison N-010 PPTX fidelity: Oyatie 95 percent supported-subset target beats estimated Google and Pitch but trails Microsoft native/web.
185. Comparison N-011 MP4 export: Oyatie deterministic MP4 target is additive rather than catch-up because web counterparts do not expose the same deterministic guarantee.
186. Comparison N-012 present transition: Oyatie p95 33 ms and p99 50 ms targets smooth present-mode parity.
187. Comparison N-013 frame budget: p99 16.7 ms is the real 60fps invariant and is stricter than the transition-only latency number.
188. Comparison N-014 broadcast signaling: p95 150 ms should support Q&A/reaction UX, but viewer throughput needs deployment-context caps.
189. Comparison N-015 broadcast scale: 500 baseline and 5,000 max viewers per deck targets strong parity with meeting-assisted presentation tools.
190. Comparison N-016 active editor sessions: 10,000 baseline and 200,000 max per cell are platform-scale claims that cannot stand without OpenTofu and load harness evidence.
191. Comparison N-017 save RPS: 1,000 baseline and 100,000 max require backend implementation, storage partitioning, and admission control.
192. Comparison N-018 export throughput: 10 to 200 jobs/sec requires worker-pool and sandbox capacity evidence.
193. Comparison N-019 AI throughput: 5 to 200 requests/sec requires foundry-runtime capacity, model placement, cost caps, and human-gating policy.
194. Comparison N-020 API quotas: Google and Microsoft publish quota/throttling concepts; Oyatie needs public quota tables by operation and tenant class.
195. Comparison N-021 Pitch analytics: Oyatie lacks explicit external-link analytics numbers and therefore cannot claim parity with Pitch engagement workflows.
196. Comparison N-022 Pitch offline: Oyatie lacks offline editing targets and should either declare out-of-scope or add sync benchmarks.
197. Comparison N-023 OCI Always Free: demo-trial targets must cap throughput without introducing retired vocabulary.
198. Comparison N-024 on-prem: latency and throughput claims need local facility assumptions.
199. Comparison N-025 colo: fixed capacity can meet low latency but not unlimited scale.
200. Comparison N-026 revenue-share: at-cost scaling needs cost-per-operation limits tied to gross revenue.
201. Comparison N-027 paid: paid tenants should scale toward the canonical max when they buy capacity.
202. Comparison N-028 demo-trial: demo-trial should show the product well with small decks, not emulate a high-volume enterprise cell.
203. Comparison N-029 compliance packs: paid/revenue-share contexts can carry compliance packs; demo-trial should not.
204. Comparison N-030 BYOK: paid/revenue-share can use BYOK; demo-trial should not.

## Section 5 - Benchmark backlog

205. Backlog B-001: replace the retired local benchmark artifact with a harness output file that contains no capability-tier names.
206. Backlog B-002: add `benchmarks/slidesbench` or the actual Rust benchmark crate if the current command in `benchmarks/...md:99-107` is meant to be executable.
207. Backlog B-003: add deck-open workload W-001 with 50 slides.
208. Backlog B-004: add deck-open workload W-002 with 100 slides.
209. Backlog B-005: add CRDT cursor-sync workload with 5, 20, and 100 collaborators.
210. Backlog B-006: add save-delta workload at 1k, 10k, and 100k save RPS.
211. Backlog B-007: add broadcast workload at 500 and 5,000 viewers.
212. Backlog B-008: add PDF export workload for 50 slides and 500 slides.
213. Backlog B-009: add PPTX export workload for 50 slides and 500 slides.
214. Backlog B-010: add MP4 export determinism workload with sha256 repeatability.
215. Backlog B-011: add PPTX reference corpus pass-rate workload matching `ADR-SLIDES-0003:195`.
216. Backlog B-012: add AI generation throughput workload with cost and human-gate timing.
217. Backlog B-013: add linked-chart refresh workload from sheets events.
218. Backlog B-014: add Sheets ACL revoke-to-slide-block latency workload.
219. Backlog B-015: add present-mode frame-budget workload matching `ADR-SLIDES-0002:68`.
220. Backlog B-016: add reduced-motion fallback verification workload.
221. Backlog B-017: add external-link analytics ingest if Pitch analytics parity is accepted.
222. Backlog B-018: add demo-trial quota enforcement workload.
223. Backlog B-019: add paid tenant scaling workload.
224. Backlog B-020: add revenue-share cost-per-operation workload.
225. Backlog B-021: add `oyatie-public-cloud` OpenTofu benchmark variables.
226. Backlog B-022: add `guest-on-aws` OpenTofu benchmark variables.
227. Backlog B-023: add `guest-on-oci` OpenTofu benchmark variables.
228. Backlog B-024: add `on-prem` benchmark assumptions.
229. Backlog B-025: add `colo` benchmark assumptions.
230. Backlog B-026: add `oyatie-as-cloud-provider` benchmark assumptions.
231. Backlog B-027: publish benchmark confidence labels: measured, simulated, estimated, and public-source limit.
232. Backlog B-028: keep every target single-set first, then overlay by context and tenant class.
233. Backlog B-029: do not reintroduce feature-quality levels as benchmark headings.
234. Backlog B-030: archive historical benchmark rows that use retired commercial labels.

## Section 6 - Numeric summary table

| # | Metric | Canonical target | Context overlay | Tenant-class overlay | Counterpart stance |
|---|---|---:|---|---|---|
| 235 | Cold deck open p95 | 400 ms | all contexts must size cache and CRDT state | demo caps deck size | ahead of estimated web |
| 236 | Cold deck open p99 | 600 ms | on-prem/colo require facility proof | paid/revenue-share scale capacity | near Pitch estimate |
| 237 | Warm deck open p95 | 150 ms | depends on cache locality | all classes same quality | ahead target |
| 238 | Slide render p95 | 100 ms | browser/GPU dependent | all classes same quality | parity/ahead |
| 239 | Edit-to-render p99 | 50 ms | browser dependent | all classes same quality | ahead target |
| 240 | Cursor sync p99 | 150 ms | network dependent | collaborator caps differ | ahead target |
| 241 | Save delta p95 | 100 ms | storage dependent | usage caps differ | ahead target |
| 242 | PDF export p95 | 3 s | worker pool dependent | export quotas differ | ahead target |
| 243 | PPTX export p95 | 5 s | worker pool dependent | export quotas differ | catch-up to Microsoft |
| 244 | MP4 export p95 | slide_count*1s+5s | GPU/CPU dependent | export quotas differ | additive |
| 245 | Present transition p99 | 50 ms | browser dependent | all classes same quality | parity target |
| 246 | Frame budget p99 | 16.7 ms | browser/GPU dependent | all classes same quality | parity target |
| 247 | Broadcast signaling p99 | 250 ms | network dependent | viewer caps differ | parity target |
| 248 | Active editors baseline | 10,000/cell | context capacity bound | caps differ | platform claim |
| 249 | Active editors max | 200,000/cell | cloud/cell capacity bound | paid/revenue-share only | needs proof |
| 250 | Save RPS baseline | 1,000 | storage bound | caps differ | platform claim |
| 251 | Save RPS max | 100,000 | storage bound | paid/revenue-share only | needs proof |
| 252 | Broadcast viewers baseline | 500/deck | network bound | demo lower cap | parity target |
| 253 | Broadcast viewers max | 5,000/deck | network bound | paid/revenue-share only | strong target |
| 254 | Export jobs baseline | 10/sec | worker bound | demo lower cap | strong target |
| 255 | Export jobs max | 200/sec | worker bound | paid/revenue-share only | needs proof |
| 256 | AI requests baseline | 5/sec | model bound | demo low or none | parity target |
| 257 | AI requests max | 200/sec | model/GPU bound | paid/revenue-share only | needs proof |
| 258 | PPTX subset pass rate | 95 percent | corpus independent | all classes same quality | ahead except Microsoft |
| 259 | Link analytics ingest | needs target | context bound | caps differ | needed for Pitch parity |
| 260 | Offline sync | needs decision | client/storage bound | caps differ | needed for Pitch parity |

## Section 7 - Conclusions

261. Conclusion C-001: current PRD targets are aggressive enough for industry-leader parity in latency and export targets.
262. Conclusion C-002: current PRD capacity numbers are not yet enforceable because OpenTofu context modules are absent.
263. Conclusion C-003: current historical benchmark artifact cannot be canonical because it uses retired commercial labels.
264. Conclusion C-004: Google and Microsoft public docs provide quota/throttling signals but not full UI latency SLOs.
265. Conclusion C-005: Pitch public docs provide clear analytics, AI, offline, import, and export feature signals but not full infrastructure SLOs.
266. Conclusion C-006: Oyatie should publish explicit API quota and usage-cap numbers to match the transparency implied by public Google and Microsoft quota docs.
267. Conclusion C-007: Pitch analytics create a measurable parity gap outside the current PRD.
268. Conclusion C-008: Pitch offline mode creates a decision point, not an automatic requirement.
269. Conclusion C-009: Microsoft PPTX fidelity remains the hardest catch-up target.
270. Conclusion C-010: Google linked chart behavior remains the chart-integration parity target.
271. Conclusion C-011: demo-trial infrastructure should be described as OCI Always Free profile, not as a commercial level.
272. Conclusion C-012: paid tenants should scale to target numbers when they purchase capacity.
273. Conclusion C-013: revenue-share tenants should scale only where operation cost is justified by gross-revenue share.
274. Conclusion C-014: all tenant classes retain the same feature quality for allowed operations.
275. Conclusion C-015: every metric above should be revalidated by a live Rust benchmark harness before production claims.

