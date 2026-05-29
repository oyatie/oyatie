# Slides microservice ownership-coherence audit

Audit date: 2026-05-20.
Audited microservice: `microservices/slides/`.
Auditor lane: Wave 3 Batch 3.2 ownership-coherence audit.
Scope rule: one agent, one microservice, no shared deliverable edits.
Deliverables in this batch: coherence audit, feature parity matrix, performance benchmark numbers.
Retired deliverable: capability-tier deltas, removed by the 2026-05-20 no-tenant-class-adoption directive.
Target counterparts: Google Slides, Microsoft PowerPoint Online, Pitch.
Assumed deployable contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`, unless evidence below constrains the claim.
Primary product verdict: slides is intended to be Oyatie's collaborative presentation authoring, live presentation, broadcast, import/export, accessibility, and AI-assisted deck-generation surface.

Citation anchor A: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2204` defines the six-context deployment bar and forbids prose-only support claims.
Citation anchor B: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2311` requires OpenTofu, per-context IaC directories, and context-local required files.
Citation anchor C: `specs/master-plan-sequencing.json:704-857` records deployment contexts, OpenTofu as IaC substrate, OS support, Rust-first language policy, and OCI Always Free profile.
Citation anchor D: `docs/standards/brief-template.md:666-1366` defines the multi-context, OpenTofu, OS, and Rust/Leptos sections expected in briefs.
Citation anchor E: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_automation_risk_classes_2026_05_20.md:10-45` retires demo_trial/paid/paid/compliance_pack tier framing.
Citation anchor F: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:88-142` drops the fourth tier-delta deliverable and replaces tier benchmark rows with tenant-class/billing semantics.
Citation anchor G: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-84` requires complete service ownership, artifact reading, chat-history search, and evidence-bound findings.
Citation anchor H: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` rejects scaffold-only documentation.
Citation anchor I: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-53` requires substance verification beyond line counts and chat-history review.

## Section 1 - Purpose

1. This audit determines whether the slides microservice owns a coherent product surface rather than a scattered set of deck-authoring notes.
2. It checks whether service artifacts match the current canonical direction for deployment contexts, infrastructure, OS support, language policy, and tenant classes.
3. It checks whether the service's stated product purpose can stand against the union surface of Google Slides, Microsoft PowerPoint Online, and Pitch.
4. It checks whether the service's current documentation contradicts itself, especially around retired capability tiers.
5. It checks whether the service's contracts, SLOs, runbooks, ADRs, implementation plans, and operating docs support the same product boundary.
6. It is not an implementation review of runtime code because `microservices/slides/src/` is not present in the service inventory.
7. It is not a shared-platform audit of tenancy, identity, audit-chain, cloud-iac, or foundry-runtime except where slides depends on them.
8. It is not a request to author a fourth capability-tier-delta report, because that deliverable is explicitly retired by the user directive and by the 2026-05-20 tier retirement memory.
9. Success means the three requested audit files land under `microservices/slides/`, line floors pass, and every substantive gap is backed by a citation.
10. Stop condition: the audit is complete when the three deliverables exist, the final HTML orchestrator report is appended here, line counts are verified, and no new cross-microservice files are touched.

### 1.1 Product purpose as evidenced

11. `PRD.md:24-36` defines slides as a collaborative presentation authoring and live broadcast microservice.
12. `PRD.md:25` places it in the Google Slides, PowerPoint Web, Keynote, Pitch, Beautiful.ai, and Canva class.
13. `PRD.md:27` says it owns deck authoring, CRDT collaboration, present/broadcast, engagement, import/export, AI generation, embeds, and LiveKit signaling reuse.
14. `PRD.md:32-36` assigns product-critical surfaces such as editing, comments, version history, presenter view, audience view, PPTX/ODP/PDF/Keynote/MP4, and chart embeds.
15. `PRD.md:40-47` defines tenant outcomes around deck-open latency, no silent loss, cursor sync, present transitions, PPTX fidelity, live charts, AI risk classification, and per-pack residency.
16. `contracts/openapi/slides.yaml:171` has an editor session endpoint with per-seat license checking.
17. `contracts/asyncapi/slides-events.yaml:51-77` defines workflow bus events produced and consumed by slides.
18. `contracts/proto/slides.proto:222-268` defines a `SlidesService` with deck, slide, ACL, export, import, broadcast, and chart request shapes.
19. `competitor-parity-matrix.md:15-130` shows the intended product surface is broad enough to be compared across authoring, collaboration, present mode, import/export, accessibility, AI, governance, and performance.
20. `PHASE-01-SLIDES-FOUNDATION.md` and `IP-001` through `IP-015` give a staged implementation map, but the root `ARCHITECTURE.md:1-3` says it was created by an anchor sweep and needs content-pass expansion.

### 1.2 Product purpose versus counterparts

21. Google Slides counterpart scope includes browser-native collaborative slide authoring, Google Drive/Meet/Sheets integration, comments, version history, themes, import/export, sharing, Q&A, and web presentation.
22. Microsoft PowerPoint Online counterpart scope includes web PowerPoint editing, native OOXML lineage, coauthoring, Microsoft 365 integration, Designer/Copilot surfaces, live presentation, comments, version history, and import/export.
23. Pitch counterpart scope includes collaborative pitch deck creation, templates, comments, sharing, analytics-oriented presentation workflows, AI-assisted creation, and streamlined export/share flows.
24. Slides correctly aims above simple deck rendering because `PRD.md:27` includes authoring, collaboration, broadcast, import/export, AI, embeds, and SDK reuse.
25. Slides also aims above generic document collaboration because `ADR-SLIDES-0002:34-40` calls out slide-specific present-mode, animation, broadcast, and accessibility constraints.
26. Slides is under-evidenced for actual deployability because its service-local IaC inventory is Helm/Kustomize-only, while canonical direction requires OpenTofu context directories.
27. Slides is under-evidenced for tenant-class semantics because no `tenant_class`, `demo_trial`, `paid`, or `revenue_share` token appears in the service path.
28. Slides is under-evidenced for OS support because no service-local supported-OS manifest exists.
29. Slides is internally coherent at product ambition level but not yet coherent at canonical deployment and tenant model level.
30. The strongest product evidence is `PRD.md`, the accepted ADR set, contracts, SLOs, runbooks, and competitor parity matrix.

## Section 2 - Inventory

### 2.1 Inventory method

31. Inventory command: `find microservices/slides -type f | sort`.
32. File count before authoring these three audit deliverables: 129.
33. Service-local line count before authoring these three audit deliverables: 17,999 total lines by `wc -l`.
34. Chat-history search command: `rg -n "slides" /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
35. Chat-history matches processed: 71.
36. Forbidden implementation-language file scan returned no `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, or `*.fs` files under `microservices/slides/`.
37. Tenant-class term search returned no `tenant_class`, `demo_trial`, `revenue_share`, or canonical tenant-class markers under `microservices/slides/`.
38. Tier retirement term search found 30 demo_trial/paid/paid/compliance_pack references under `microservices/slides/`.
39. IaC inventory found Helm and Kustomize assets, not canonical OpenTofu context directories.
40. No `microservices/slides/src/` directory was present.
41. No `microservices/slides/tests/` directory was present.
42. No `microservices/slides/README.md` file was present.

### 2.2 Complete file inventory seen

43. `microservices/slides/ARCHITECTURE.md`
44. `microservices/slides/AUDIT-FINDINGS-2026-05-18.json`
45. `microservices/slides/IP-001-layer-a-cdn-postgres-valkey-s3-ws-gateway-iac.md`
46. `microservices/slides/IP-002-presentation-slide-kernel-domain.md`
47. `microservices/slides/IP-003-slide-layout-text-box-shape-kernel-domain.md`
48. `microservices/slides/IP-004-asset-bcs-image-video-audio-adapters.md`
49. `microservices/slides/IP-005-real-time-collaboration-loro-kernel-domain-adapter.md`
50. `microservices/slides/IP-006-real-time-collaboration-worker-sdk.md`
51. `microservices/slides/IP-007-chart-embed-bridge-to-sheets.md`
52. `microservices/slides/IP-008-themes-templates-master-slide-editor.md`
53. `microservices/slides/IP-009-animations-transitions-reduced-motion.md`
54. `microservices/slides/IP-010-presenter-audience-view-broadcast-mode-livekit.md`
55. `microservices/slides/IP-011-import-export-pptx-pdf-mp4-pipeline.md`
56. `microservices/slides/IP-012-accessibility-ai-design-ai-content-generation.md`
57. `microservices/slides/IP-013-acl-comments-version-history-embed-bridge.md`
58. `microservices/slides/IP-014-visual-canvas-leptos-wasm-rest-sdk-app.md`
59. `microservices/slides/IP-015-hg-slides-registration-and-branch-protection.md`
60. `microservices/slides/IP-journey-j100-pack-rollout-first-action.md`
61. `microservices/slides/IP-journey-j91-us-msb-mtl-overlay.md`
62. `microservices/slides/IP-journey-j92-br-lgpd-us-parent-dsar.md`
63. `microservices/slides/IP-journey-j93-in-dpdpa-rbi-overlay.md`
64. `microservices/slides/IP-journey-j94-sox404-public-company-controls.md`
65. `microservices/slides/IP-journey-j95-iso27001-soc2-annual-audit.md`
66. `microservices/slides/IP-journey-j96-ksa-uae-mena-onboarding.md`
67. `microservices/slides/IP-journey-j97-sg-pdpa-mas-tenant.md`
68. `microservices/slides/IP-journey-j98-au-privacy-apra-cps234.md`
69. `microservices/slides/IP-journey-j99-multi-pack-conflict-resolution.md`
70. `microservices/slides/PHASE-01-SLIDES-FOUNDATION.md`
71. `microservices/slides/PRD.md`
72. `microservices/slides/backfill-replay.md`
73. `microservices/slides/benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md`
74. `microservices/slides/capabilities/T0-suggest.yaml`
75. `microservices/slides/capabilities/T1-assist.yaml`
76. `microservices/slides/capabilities/T2-auto.yaml`
77. `microservices/slides/tenant-class-adoption/tenant-class-adoption-record.md`
78. `microservices/slides/capacity-model.md`
79. `microservices/slides/catalog/oya-slides-acl-adapter-postgres.yaml`
80. `microservices/slides/catalog/oya-slides-acl-domain.yaml`
81. `microservices/slides/catalog/oya-slides-acl-kernel.yaml`
82. `microservices/slides/catalog/oya-slides-ai-content-generation-domain.yaml`
83. `microservices/slides/catalog/oya-slides-broadcast-mode-adapter-livekit.yaml`
84. `microservices/slides/catalog/oya-slides-image-adapter-clamav.yaml`
85. `microservices/slides/catalog/oya-slides-image-adapter-imagemagick.yaml`
86. `microservices/slides/catalog/oya-slides-image-adapter-opswat.yaml`
87. `microservices/slides/catalog/oya-slides-import-export-adapter-chromium-headless.yaml`
88. `microservices/slides/catalog/oya-slides-import-export-adapter-ffmpeg.yaml`
89. `microservices/slides/catalog/oya-slides-import-export-adapter-pandoc.yaml`
90. `microservices/slides/catalog/oya-slides-import-export-adapter-weasyprint.yaml`
91. `microservices/slides/catalog/oya-slides-presentation-adapter-postgres.yaml`
92. `microservices/slides/catalog/oya-slides-presentation-adapter-s3.yaml`
93. `microservices/slides/catalog/oya-slides-presentation-domain.yaml`
94. `microservices/slides/catalog/oya-slides-presentation-kernel.yaml`
95. `microservices/slides/catalog/oya-slides-presentation-rest.yaml`
96. `microservices/slides/catalog/oya-slides-real-time-collaboration-adapter-loro.yaml`
97. `microservices/slides/catalog/oya-slides-real-time-collaboration-adapter-valkey.yaml`
98. `microservices/slides/catalog/oya-slides-real-time-collaboration-domain.yaml`
99. `microservices/slides/catalog/oya-slides-real-time-collaboration-kernel.yaml`
100. `microservices/slides/catalog/oya-slides-real-time-collaboration-worker.yaml`
101. `microservices/slides/catalog/oya-slides-slide-adapter-leptos-wasm.yaml`
102. `microservices/slides/catalog/oya-slides-slide-domain.yaml`
103. `microservices/slides/catalog/oya-slides-slide-kernel.yaml`
104. `microservices/slides/competitor-parity-matrix.md`
105. `microservices/slides/compliance.md`
106. `microservices/slides/contracts/asyncapi/slides-events.yaml`
107. `microservices/slides/contracts/openapi/slides.yaml`
108. `microservices/slides/contracts/proto/slides.proto`
109. `microservices/slides/cost-budget.md`
110. `microservices/slides/dashboards/editor-experience.json`
111. `microservices/slides/dashboards/export-and-import-pipeline.json`
112. `microservices/slides/dashboards/present-and-broadcast.json`
113. `microservices/slides/decisions/ADR-SLD-001-svg-first-render-pipeline-vs-canvas.md`
114. `microservices/slides/decisions/ADR-SLIDES-0001-crdt-library-selection.md`
115. `microservices/slides/decisions/ADR-SLIDES-0002-rendering-canvas-substrate.md`
116. `microservices/slides/decisions/ADR-SLIDES-0003-export-pipeline-fidelity.md`
117. `microservices/slides/decisions/ADR-SLIDES-0004-animation-engine-and-reduced-motion.md`
118. `microservices/slides/decisions/ADR-SLIDES-0005-broadcast-mode-and-livekit-reuse.md`
119. `microservices/slides/decisions/ADR-SLIDES-0006-ai-design-and-content-generation-bounds.md`
120. `microservices/slides/decisions/ADR-SLIDES-0007-per-slide-acl-granularity.md`
121. `microservices/slides/decisions/ADR-SLIDES-0008-chart-live-link-to-sheets.md`
122. `microservices/slides/decisions/README.md`
123. `microservices/slides/dpia.md`
124. `microservices/slides/failure-modes.md`
125. `microservices/slides/faqs/slides-engineer-faq.md`
126. `microservices/slides/iac/helm/Chart.yaml`
127. `microservices/slides/iac/helm/templates/deployment.yaml`
128. `microservices/slides/iac/helm/templates/hpa.yaml`
129. `microservices/slides/iac/helm/templates/networkpolicy.yaml`
130. `microservices/slides/iac/helm/templates/pdb.yaml`
131. `microservices/slides/iac/helm/templates/prometheusrule.yaml`
132. `microservices/slides/iac/helm/templates/service.yaml`
133. `microservices/slides/iac/helm/templates/servicemonitor.yaml`
134. `microservices/slides/iac/helm/values.yaml`
135. `microservices/slides/iac/kustomize/base/kustomization.yaml`
136. `microservices/slides/iac/kustomize/base/namespace.yaml`
137. `microservices/slides/iac/kustomize/overlays/pack-eu/kustomization.yaml`
138. `microservices/slides/iac/kustomize/overlays/pack-kr/kustomization.yaml`
139. `microservices/slides/incident-response.md`
140. `microservices/slides/manifest.json`
141. `microservices/slides/migration-playbooks/from-google-slides-and-powerpoint.md`
142. `microservices/slides/multi-region.md`
143. `microservices/slides/onboarding/slides-engineer-first-week.md`
144. `microservices/slides/policy/auditor-scope.cedar`
145. `microservices/slides/policy/ci-scope.cedar`
146. `microservices/slides/policy/data-residency.md`
147. `microservices/slides/policy/editor-isolation.md`
148. `microservices/slides/policy/public-read.cedar`
149. `microservices/slides/policy/tenant-scope.cedar`
150. `microservices/slides/reference-implementations/create-deck-and-export-rust-sdk.md`
151. `microservices/slides/runbooks/animation-engine-rollback.md`
152. `microservices/slides/runbooks/attachment-restore.md`
153. `microservices/slides/runbooks/broadcast-mode-degraded.md`
154. `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`
155. `microservices/slides/runbooks/export-pipeline-failure-pptx.md`
156. `microservices/slides/runbooks/share-acl-drift.md`
157. `microservices/slides/runbooks/theme-corruption.md`
158. `microservices/slides/scorecards/overrides.json`
159. `microservices/slides/sdk-plan.md`
160. `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`
161. `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`
162. `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`
163. `microservices/slides/slos/deck-open-latency.openslo.yaml`
164. `microservices/slides/slos/export-mp4-latency.openslo.yaml`
165. `microservices/slides/slos/export-pdf-latency.openslo.yaml`
166. `microservices/slides/slos/export-pptx-latency.openslo.yaml`
167. `microservices/slides/slos/present-mode-transition-latency.openslo.yaml`
168. `microservices/slides/slos/save-latency.openslo.yaml`
169. `microservices/slides/slos/slide-render-latency.openslo.yaml`
170. `microservices/slides/threat-model.md`
171. `microservices/slides/tutorials/build-investor-deck-with-charts-and-collab.md`

### 2.3 Artifact families read

172. Product requirement source: `PRD.md`, especially `PRD.md:24-47`, `PRD.md:51-87`, `PRD.md:90-108`, `PRD.md:374-438`, and `PRD.md:461-487`.
173. Architecture source: `ARCHITECTURE.md`, especially `ARCHITECTURE.md:1-3`, `ARCHITECTURE.md:22-35`, `ARCHITECTURE.md:506-518`, and `ARCHITECTURE.md:568-579`.
174. Decision sources: `decisions/ADR-SLD-001-svg-first-render-pipeline-vs-canvas.md` plus `decisions/ADR-SLIDES-0001` through `ADR-SLIDES-0008`.
175. Implementation-plan sources: `IP-001` through `IP-015` and `IP-journey-j91` through `IP-journey-j100`.
176. Contract sources: `contracts/openapi/slides.yaml`, `contracts/asyncapi/slides-events.yaml`, and `contracts/proto/slides.proto`.
177. SLO sources: all nine `slos/*.openslo.yaml` files.
178. Deprecated tier source: `tenant-class-adoption/tenant-class-adoption-record.md`.
179. Capability tier-adjacent YAML sources: `capabilities/T0-suggest.yaml`, `capabilities/T1-assist.yaml`, and `capabilities/T2-auto.yaml`.
180. Operational sources: `capacity-model.md`, `failure-modes.md`, `incident-response.md`, `cost-budget.md`, `dpia.md`, `compliance.md`, and all seven runbooks.
181. User-facing sources: onboarding, FAQ, tutorial, migration playbook, and Rust SDK reference implementation.
182. IaC sources: Helm chart files and Kustomize base/overlay files under `iac/`.
183. Policy sources: Cedar files and data-residency/editor-isolation Markdown files.
184. Dashboard sources: editor experience, export/import, and present/broadcast JSON dashboards.
185. Chat-history source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.

### 2.4 Chat-history evidence

186. Chat search term: `slides`.
187. Raw match count: 71.
188. Relevant orchestration hit: chat line `16424` refers to current audit activity including docs, sheets, slides, forms, and connect.
189. Relevant orchestration hit: chat line `16439` says Phase 3 first cohort includes `mail / drive / calendar / meet / recordings / notes / docs / sheets / slides / forms / connect`.
190. Relevant orchestration hit: chat line `16439` also says agents should verify deliverables after that cohort fires.
191. The searched chat history did not add a product-specific slides requirement beyond the audit cohort and deliverable verification context.
192. The absence of additional product-specific chat overrides means the current user prompt and canonical sources remain the controlling direction.
193. Chat history was used only to confirm this service is in the active audit wave and that deliverable verification is expected.

### 2.5 Inventory quality notes

194. Strong inventory family: PRD, ADRs, implementation plans, contracts, SLOs, runbooks, dashboards, policies, and user-facing docs are present.
195. Missing inventory family: root `README.md` is absent.
196. Missing inventory family: source code directory is absent.
197. Missing inventory family: tests directory is absent.
198. Missing canonical deployment family: `iac/oyatie-public-cloud/` is absent.
199. Missing canonical deployment family: `iac/guest-on-aws/` is absent.
200. Missing canonical deployment family: `iac/oci-guest/` is absent.
201. Missing canonical deployment family: `iac/on-prem/` is absent.
202. Missing canonical deployment family: `iac/colo/` is absent.
203. Missing canonical deployment family: `iac/oyatie-iaas/` is absent.
204. Missing OCI profile family: `iac/oci-guest/always-free/` is absent.
205. Missing canonical OpenTofu files: no service-local context `main.tf`, `variables.tf`, `outputs.tf`, or `versions.tf` were found.
206. Existing IaC family: Helm chart assets under `iac/helm/`.
207. Existing IaC family: Kustomize base and two overlays under `iac/kustomize/`.
208. Existing contract family is broad but has a workflow-event drift noted in Finding SLD-009.

## Section 3 - Nine-dimension audit

### 3.1 Dimension 1 - Product ownership boundary

209. Verdict: strong product boundary, with implementation evidence absent.
210. Evidence: `PRD.md:24-36` defines slides as the presentation authoring, collaboration, present/broadcast, import/export, AI, embed, and SDK service.
211. Evidence: `PRD.md:51-87` gives 34 functional requirements covering deck CRUD, slide CRUD, CRDT collaboration, templates, notes, present mode, broadcast, import/export, AI, ACL, comments, version history, and cross-product publishing.
212. Evidence: `competitor-parity-matrix.md:15-130` compares the service against product families expected of a presentation suite.
213. Evidence: `contracts/proto/slides.proto:222-268` exposes service RPCs for deck retrieval, mutation, ACL, export/import, broadcast, and charts.
214. Evidence: `contracts/openapi/slides.yaml:327-360` includes AI design-assist, full-deck generation, and alt-text endpoints.
215. Evidence: `contracts/asyncapi/slides-events.yaml:51-77` defines workflow bus inputs and outputs.
216. The service boundary is not merely "render slides"; it includes authoring state, collaboration state, presentation state, export/import state, and controlled AI state.
217. The boundary properly excludes owning identity, tenancy, audit-chain, observability, sheets, forms, messenger, and application shell as core services, because `PRD.md:287-302` describes those as dependencies or SDK surfaces.
218. Gap: runtime implementation is not evidenced under this service path because no `src/` directory exists.
219. Gap: automated behavior proof is not evidenced under this service path because no `tests/` directory exists.
220. Impact: product ownership is coherent as a documentation/specification boundary, not yet as a service-local executable boundary.
221. Remediation: keep the product boundary, but mark deployability claims as specification-stage until source, test, and OpenTofu evidence lands.

### 3.2 Dimension 2 - Artifact internal consistency

222. Verdict: mostly coherent across PRD, ADRs, contracts, and SLOs, with several stale seams.
223. Evidence: `PRD.md:90-108` performance targets align with SLO files such as `slos/deck-open-latency.openslo.yaml:16`, `slos/save-latency.openslo.yaml:16`, and `slos/collab-cursor-sync-latency.openslo.yaml:16`.
224. Evidence: `ADR-SLIDES-0002-rendering-canvas-substrate.md:51-68` resolves the PRD rendering open question by selecting Leptos WASM, SVG baseline, canvas-2d, and WebGL fallback.
225. Evidence: `ADR-SLIDES-0003-export-pipeline-fidelity.md:53-76` resolves the PRD export pipeline question by selecting Pandoc import, Rust PPTX serializer, WeasyPrint/Chromium PDF paths, and ffmpeg MP4.
226. Evidence: `ADR-SLIDES-0005-broadcast-mode-and-livekit-reuse.md` aligns with `PRD.md:69-70` present/broadcast requirements.
227. Evidence: `ADR-SLIDES-0007-per-slide-acl-granularity.md` aligns with `PRD.md:77-82` ACL and sharing requirements.
228. Drift: `PRD.md:485-487` still lists rendering and export substrate as open questions even though ADR-SLIDES-0002 and ADR-SLIDES-0003 are accepted.
229. Drift: `PRD.md:469-470` names `tests/load/*.js` acceptance lanes even though the Rust-strict backend/frontend allowlist rejects JavaScript as a service implementation or test-language default outside browser bootstrap.
230. Drift: `PRD.md:340` lists `AltTextSuggested` as a produced workflow event, while `contracts/asyncapi/slides-events.yaml:55-67` lists AI events but not `AltTextSuggested`.
231. Drift: `ARCHITECTURE.md:1-3` says the architecture file was produced by an anchor sweep and must be expanded during content-pass review.
232. Drift: `ARCHITECTURE.md:24-35` uses `slides.unknown` and `unknown` bounded-context markers.
233. Drift: `ARCHITECTURE.md:568-579` describes Helm/Kustomize deployment and "lower" deployment levels rather than current OpenTofu context modules.
234. Impact: stale artifacts could mislead later implementers into reopening already accepted ADR decisions or implementing forbidden test surfaces.
235. Remediation: PRD open questions and acceptance-test language should be refreshed in a Wave 15J-safe update that preserves accepted ADR decisions.

### 3.3 Dimension 3 - Substance bar and scaffold risk

236. Verdict: mixed; product documents are rich, but `ARCHITECTURE.md` is an explicit scaffold.
237. Evidence: `PRD.md:24-518` is substantive, with product purpose, functional requirements, performance targets, security, audit events, data residency, bounded contexts, dependency map, CI lanes, event tables, and acceptance criteria.
238. Evidence: accepted ADRs contain detailed context, decisions, alternatives, consequences, risks, and downstream impacts.
239. Evidence: runbooks and operational docs are present for broadcast degradation, CRDT conflicts, export failure, share ACL drift, theme corruption, animation rollback, and attachment restore.
240. Evidence: `competitor-parity-matrix.md:15-130` maps a broad competitor surface.
241. Scaffold evidence: `ARCHITECTURE.md:1-3` says it was created by the Wave-3-C anchor sweep and instructs readers to expand stubs.
242. Scaffold evidence: `ARCHITECTURE.md:22-35` repeats generic "slides.unknown" markers for context and event binding.
243. Scaffold evidence: `ARCHITECTURE.md:506-518` lists generic runtime, policy, telemetry, and Helm/Kustomize evidence rather than a slides-specific OpenTofu context plan.
244. Canonical pressure: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` rejects thin or scaffold-like docs.
245. Canonical pressure: `docs/standards/brief-template.md:1727-1740` identifies scaffold-without-substance and line-count-as-completion as anti-patterns.
246. Impact: a future reader could over-trust `ARCHITECTURE.md` as authoritative even though it admits it is not content-complete.
247. Remediation: demote the anchor-sweep architecture file to draft status or rewrite it into a service-specific architecture that cites PRD, ADR, contracts, SLOs, and OpenTofu context files.

### 3.4 Dimension 4 - Canonical-direction alignment

248. Verdict: product-level direction is close, but canonical deployment, tenant-class, tier-retirement, OS, and OpenTofu alignment are incomplete.
249. Multi-context canonical evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1736-1983` enumerates six deployment contexts.
250. Multi-context canonical evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2086` says service manifests must name supported contexts.
251. Multi-context canonical evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2198-2204` forbids prose-only support claims.
252. OpenTofu canonical evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2251` says OpenTofu is canonical and not Terraform.
253. OpenTofu canonical evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2281-2311` requires per-context directories and `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md`.
254. OS canonical evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-31` defines the OS support obligation.
255. Rust strict canonical evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-60` defines Rust backend, frontend allowlist, and forbidden languages.
256. OCI canonical evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:65-82` requires an OCI Always Free per-service profile and sizing notes.
257. Tier-retirement canonical evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_automation_risk_classes_2026_05_20.md:10-45` says capability tiers are being retired and demo_trial/paid/paid/compliance_pack should not be perpetuated.
258. Tenant-class canonical evidence: current user directive defines `demo_trial`, `paid`, and `revenue_share` as the replacement audit model for this batch.
259. Local alignment pass: `PRD.md:27-36` correctly targets a full presentation product, not a minimal slide renderer.
260. Local alignment pass: `ADR-SLIDES-0002-rendering-canvas-substrate.md:55-68` follows Leptos/WASM direction for browser UI.
261. Local alignment pass: no actual forbidden implementation-language files are present under `microservices/slides/`.
262. Local alignment gap: `iac/` is Helm/Kustomize-only and lacks canonical OpenTofu context directories.
263. Local alignment gap: no `supported_oses.json` or equivalent service-local OS support manifest is present.
264. Local alignment gap: no `tenant_class` semantics are present.
265. Local alignment gap: thirty demo_trial/paid/paid/compliance_pack references remain.
266. Local alignment gap: `PRD.md:469-470` still names JavaScript load-test files.
267. Local alignment gap: `ADR-SLIDES-0003-export-pipeline-fidelity.md:74-75` adopts WeasyPrint, which is a Python renderer, while the current language doctrine requires strict classification and exceptions.

#### 3.4.T - Tier retirement candidates

268. Classification rule: every line below is a Wave 15J retirement candidate, default severity P2, because it references demo_trial/paid/paid/compliance_pack capability-vocabulary.
269. Candidate 1: `onboarding/slides-engineer-first-week.md:45` references a paid target.
270. Candidate 2: `onboarding/slides-engineer-first-week.md:46` references a paid target.
271. Candidate 3: `onboarding/slides-engineer-first-week.md:71` references expected fidelity at paid.
272. Candidate 4: `migration-playbooks/from-google-slides-and-powerpoint.md:89` references direct migration at paid or above.
273. Candidate 5: `migration-playbooks/from-google-slides-and-powerpoint.md:99` references direct migration at paid or above.
274. Candidate 6: `tutorials/build-investor-deck-with-charts-and-collab.md:15` references a paid-plus tier slides cell.
275. Candidate 7: `tutorials/build-investor-deck-with-charts-and-collab.md:164` references sync lag at paid.
276. Candidate 8: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:13` references hardware as oyatie paid.
277. Candidate 9: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:21` references oyatie slides paid.
278. Candidate 10: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:29` says oyatie paid leads web-based platforms.
279. Candidate 11: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:31` references a paid deck-open target.
280. Candidate 12: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:37` references oyatie slides paid GPU SVG.
281. Candidate 13: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:74` references oyatie slides paid AI T2.
282. Candidate 14: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:86` references oyatie slides paid.
283. Candidate 15: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:87` references oyatie slides paid.
284. Candidate 16: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:95` references oyatie paid.
285. Candidate 17: `faqs/slides-engineer-faq.md:22` references paid GPU SVG render.
286. Candidate 18: `faqs/slides-engineer-faq.md:43` references paid pre-render.
287. Candidate 19: `faqs/slides-engineer-faq.md:74` references paid tier cross-deck embedding.
288. Candidate 20: `tenant-class-adoption/tenant-class-adoption-record.md:13` defines a demo_trial preview tier.
289. Candidate 21: `tenant-class-adoption/tenant-class-adoption-record.md:48` defines a paid production-default level.
290. Candidate 22: `tenant-class-adoption/tenant-class-adoption-record.md:50` says paid adds to demo_trial.
291. Candidate 23: `tenant-class-adoption/tenant-class-adoption-record.md:85` defines a paid multi-region level.
292. Candidate 24: `tenant-class-adoption/tenant-class-adoption-record.md:87` says paid adds to paid.
293. Candidate 25: `tenant-class-adoption/tenant-class-adoption-record.md:117` compares cost against paid.
294. Candidate 26: `tenant-class-adoption/tenant-class-adoption-record.md:121` defines a compliance_pack sovereign-pack-bound level.
295. Candidate 27: `tenant-class-adoption/tenant-class-adoption-record.md:123` says compliance_pack adds to paid.
296. Candidate 28: `tenant-class-adoption/tenant-class-adoption-record.md:135` says same operational latency as paid.
297. Candidate 29: `tenant-class-adoption/tenant-class-adoption-record.md:137` says same SLO posture as paid.
298. Candidate 30: `tenant-class-adoption/tenant-class-adoption-record.md:150` defines demo_trial to paid to paid to compliance_pack promotion paths.
299. Retirement note: `tenant-class-adoption/tenant-class-adoption-record.md:9-13` is not reusable as a tenant-class replacement because it stratifies features and quality.
300. Retirement note: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:13-31` must be rewritten before its numbers can be used as current benchmark targets.
301. Retirement note: `onboarding`, `tutorials`, `faqs`, and `migration-playbooks` need wording updates so new engineers and users do not learn retired vocabulary.

#### 3.4.C - Tenant-class adoption gaps

302. Search result: no `tenant_class` token appears under `microservices/slides/`.
303. Search result: no `demo_trial` token appears under `microservices/slides/`.
304. Search result: no `revenue_share` token appears under `microservices/slides/`.
305. Search result: only legacy commercial language such as per-seat licensing appears, including `contracts/openapi/slides.yaml:171`, `contracts/openapi/slides.yaml:409`, `sdk-plan.md:71`, and `failure-modes.md:53`.
306. Search result: `cost-budget.md:74` references an Enterprise unlimited/per-seat cost row, which is not the requested three-class model.
307. Gap: the service does not express `demo_trial`, `paid`, or `revenue_share` as tenant classes.
308. Gap: the service does not distinguish usage caps for demo/trial tenants from contractual SLO and compliance/BYOK allowances for paid tenants.
309. Gap: the service does not describe revenue-share operation at cost or zero-margin substrate for embedded sellers, B2C operators, or marketplace partners.
310. Gap: the service does not map present/broadcast, export, AI generation, or import scanning quotas to tenant classes.
311. Gap: the service does not state that product quality remains industry-leader-grade across all tenant classes.
312. Gap: the service has no service-local schema, manifest field, contract field, or policy file for tenant class.
313. Severity: P2 documentation gap unless implementation later claims production tenant onboarding without this model.
314. Remediation: add tenant-class semantics to the service manifest, capacity model, cost budget, runbooks, and OpenTofu variables once the canonical schema is codified.

### 3.5 Dimension 5 - Counterpart product parity

315. Verdict: strong parity ambition, with risk concentrated in import/export fidelity, coauthoring proof, live broadcast proof, and AI governance proof.
316. Google Slides coverage evidence: `competitor-parity-matrix.md:17-36` maps authoring and canvas features against Google Slides.
317. Google Slides coverage evidence: `competitor-parity-matrix.md:38-49` maps collaboration features.
318. Google Slides coverage evidence: `competitor-parity-matrix.md:51-61` maps present and broadcast features.
319. Microsoft PowerPoint Online coverage evidence: `competitor-parity-matrix.md:63-73` maps import/export and PowerPoint-native PPTX advantage.
320. Pitch coverage evidence: `competitor-parity-matrix.md:87-102` maps AI and modern presentation features including full-deck generation.
321. Oyatie differentiator evidence: `competitor-parity-matrix.md:48-49` marks per-slide and named-block ACL as unique.
322. Oyatie differentiator evidence: `competitor-parity-matrix.md:100-102` marks EU AI Act risk-class stamps and high-risk refusal as unique.
323. Oyatie differentiator evidence: `competitor-parity-matrix.md:112-115` marks audit-chain, SLSA, and WASM SRI as governance advantages.
324. Parity risk: `ADR-SLIDES-0003:150-165` rejects full strict PPTX round-trip and accepts a subset strategy; this is realistic but must be visible to tenants.
325. Parity risk: `PRD.md:105-107` targets high active sessions and broadcast viewers, but no service-local load-test artifacts exist.
326. Parity risk: `PRD.md:426-438` gives capacity envelopes, but no per-context OpenTofu or load harness proves those envelopes.
327. Parity risk: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:15` says comparators were measured against docs and Pitch performance blog, but the service-local benchmark artifact contains retired vocabulary and no reproduced raw results.
328. Required follow-up: benchmark claims should be re-issued in `performance-benchmark-numbers-2026-05-20.md` with single target numbers and no retired schema rows.

### 3.6 Dimension 6 - Multi-context deployability

329. Verdict: not yet coherent for all six contexts.
330. Canonical source: `specs/master-plan-sequencing.json:704-746` lists the six deployment contexts.
331. Canonical source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2086` requires service manifests to name supported contexts.
332. Local evidence: `manifest.json` exists, but the current audit did not find a complete six-context support declaration that satisfies ADR-0328.
333. Local evidence: `iac/helm/Chart.yaml` and Helm templates exist.
334. Local evidence: `iac/kustomize/base/kustomization.yaml` exists.
335. Local evidence: `iac/kustomize/overlays/pack-eu/kustomization.yaml` exists.
336. Local evidence: `iac/kustomize/overlays/pack-kr/kustomization.yaml` exists.
337. Missing evidence: no `iac/oyatie-public-cloud/` directory exists.
338. Missing evidence: no `iac/guest-on-aws/` directory exists.
339. Missing evidence: no `iac/oci-guest/` directory exists.
340. Missing evidence: no `iac/on-prem/` directory exists.
341. Missing evidence: no `iac/colo/` directory exists.
342. Missing evidence: no `iac/oyatie-iaas/` directory exists.
343. Missing evidence: no per-context sizing and admission variables exist under OpenTofu.
344. Existing architecture drift: `ARCHITECTURE.md:515` and `ARCHITECTURE.md:575` cite Helm/Kustomize rather than OpenTofu context modules.
345. Impact: the service cannot honestly claim all-six-context deployability from local artifacts alone.
346. Remediation: add context-specific OpenTofu modules or mark unsupported contexts explicitly with rationale.

### 3.7 Dimension 7 - OpenTofu and OCI Always Free profile

347. Verdict: OpenTofu substrate absent; OCI Always Free profile absent.
348. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-18` requires OpenTofu modules, not clickops or handrolled provisioning.
349. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35` names per-service directory and forbidden provisioning patterns.
350. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:65-82` requires per-service OCI Always Free profile handling.
351. Local evidence: `iac/helm/Chart.yaml` shows a Helm path exists.
352. Local evidence: `iac/helm/templates/deployment.yaml` shows Kubernetes deployment templating exists.
353. Local evidence: `iac/kustomize/base/kustomization.yaml` and overlay files show Kustomize packing exists.
354. Local evidence: `compliance.md:1000` references a dependency inventory spanning Helm, Kustomize, and OpenTofu, but no OpenTofu files are present in this service path.
355. Missing evidence: no service-local `.tf` files are present in canonical context directories.
356. Missing evidence: no `iac/oci-guest/always-free/` path exists.
357. Missing evidence: no demo-trial OCI Always Free capacity cap is expressed.
358. Missing evidence: no OpenTofu variable model maps tenant class to resource limits.
359. Missing evidence: no context-local outputs establish service endpoint, SLO, scaling, or observability wiring.
360. Impact: IaC currently supports Kubernetes packaging evidence, not canonical provider-agnostic provisioning evidence.
361. Remediation: preserve Helm/Kustomize as workload packaging where needed, but make OpenTofu the service provisioning entry point.

### 3.8 Dimension 8 - OS support matrix

362. Verdict: missing service-local OS support evidence.
363. Canonical source: `specs/master-plan-sequencing.json:777-816` records supported OS policy.
364. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-31` defines Linux, BSD/Solaris, and client OS obligations.
365. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-76` shows expected manifest shape and treats missing coverage as a gap.
366. Local evidence: no `supported_oses.json` file exists under `microservices/slides/`.
367. Local evidence: no service-local manifest enumerates Linux distributions.
368. Local evidence: no service-local manifest enumerates BSD or Solaris build/test claims.
369. Local evidence: no service-local manifest enumerates desktop/mobile frontend target claims.
370. Local evidence: `ADR-SLIDES-0002:57-68` defines browser/WASM behavior but not OS matrix coverage.
371. Local evidence: `reference-implementations/create-deck-and-export-rust-sdk.md` demonstrates SDK use but not OS support certification.
372. Impact: OS support cannot be audited beyond browser/WASM intent and Rust SDK examples.
373. Remediation: add a service-local OS support manifest that separates server runtime support, web/browser support, native frontend support, and unsupported surfaces.

### 3.9 Dimension 9 - Rust-strict and implementation-language posture

374. Verdict: actual file scan passes; documentation posture needs cleanup.
375. Canonical source: `specs/master-plan-sequencing.json:817-856` records Rust backend and frontend allowlist policy.
376. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-18` requires Rust strictness.
377. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:38-49` lists allowed non-Rust formats and frontend technologies.
378. Canonical source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60` lists forbidden languages and scan obligation.
379. Local scan pass: no forbidden-language source files were found under `microservices/slides/`.
380. Local Rust alignment: `ADR-SLIDES-0002:55-68` chooses Leptos/WASM and Rust-side signals rather than TypeScript canvas frameworks.
381. Local Rust alignment: `ADR-SLIDES-0002:72-100` rejects React, SolidJS, and Svelte because they break code-share and Rust-native strategy.
382. Local Rust alignment: `reference-implementations/create-deck-and-export-rust-sdk.md` uses Rust SDK examples.
383. Documentation drift: `PRD.md:469-470` names `tests/load/*.js` lanes.
384. Documentation drift: `ADR-SLIDES-0003:74-75` chooses WeasyPrint as the default PDF renderer, which brings a Python dependency into the export-worker supply chain.
385. Documentation drift: `catalog/oya-slides-import-export-adapter-weasyprint.yaml` exists, matching the WeasyPrint adapter decision.
386. Documentation nuance: WeasyPrint as a sandboxed third-party binary may be an allowed adapter dependency only if canonical policy grants an explicit exception.
387. Documentation risk: without an exception record, future implementers could mistake WeasyPrint for permission to add Python service logic.
388. Impact: runtime files pass, but docs need to classify JS load testing and Python renderer dependencies under the strict policy.
389. Remediation: replace `tests/load/*.js` with Rust-native or allowed tooling lanes, and either justify or replace WeasyPrint under a formal exception.

## Section 4 - Findings table

| ID | Severity | Finding | Evidence | Required action |
|---|---|---|---|---|
| SLD-001 | P1 | All-six-context deployability is not evidenced by service-local IaC. | Missing canonical context dirs; `ADR-0328:2281-2311`; `ARCHITECTURE.md:515`; `ARCHITECTURE.md:575`. | Add OpenTofu context modules or mark unsupported contexts. |
| SLD-002 | P1 | OpenTofu is absent and Helm/Kustomize is the only local IaC surface. | `iac/helm/*`; `iac/kustomize/*`; no `.tf` context modules; zero-handroll memory `10-35`. | Make OpenTofu the provisioning entry point. |
| SLD-003 | P1 | OCI Always Free profile is absent. | No `iac/oci-guest/always-free/`; OCI memory `65-82`. | Add demo-trial OCI Always Free profile with service-specific caps. |
| SLD-004 | P1 | Service-local OS support matrix is absent. | No `supported_oses.json`; OS memory `56-76`; master plan `777-816`. | Add supported-OS manifest and test posture. |
| SLD-005 | P1 | Architecture document is explicitly scaffold-stage. | `ARCHITECTURE.md:1-3`; `ARCHITECTURE.md:22-35`; docs-substance memory `10-18`. | Rewrite or demote architecture doc. |
| SLD-006 | P2 | Thirty demo_trial/paid/paid/compliance_pack references remain. | Candidates listed in §3.4.T; no-tenant-class-adoption memory `10-45`. | Retire or rewrite in Wave 15J. |
| SLD-007 | P2 | Tenant-class adoption is absent. | No `tenant_class`, `demo_trial`, or `revenue_share`; current user directive; tenant memory `88-142`. | Add `demo_trial`, `paid`, `revenue_share` semantics. |
| SLD-008 | P2 | PRD open questions are stale after accepted ADRs. | `PRD.md:485-487`; `ADR-SLIDES-0002:51-68`; `ADR-SLIDES-0003:53-76`. | Replace open questions with accepted decision references. |
| SLD-009 | P2 | `AltTextSuggested` event appears in PRD but not AsyncAPI workflow output. | `PRD.md:340`; `contracts/asyncapi/slides-events.yaml:55-67`. | Add event to AsyncAPI or revise PRD event table. |
| SLD-010 | P2 | PRD references JavaScript load tests despite Rust-strict policy. | `PRD.md:469-470`; Rust memory `51-60`. | Replace with Rust-native load or approved tool lane. |
| SLD-011 | P2 | WeasyPrint dependency needs explicit exception or replacement. | `ADR-SLIDES-0003:74-75`; `catalog/oya-slides-import-export-adapter-weasyprint.yaml`; Rust memory `10-60`. | Document exception or migrate PDF default. |
| SLD-012 | P2 | Legacy benchmark artifact is tier-segmented and cannot be canonical. | `benchmarks/...md:13-31`; `benchmarks/...md:84-95`; no-tenant-class-drift memory `10-45`. | Use the new single-target benchmark report. |
| SLD-013 | P2 | Root service README is absent. | Inventory; no `microservices/slides/README.md`. | Add concise service entrypoint after architecture rewrite. |
| SLD-014 | P2 | Service manifests use adjacent tier terminology beyond retired commercial tiers. | `manifest.json:342-345`; `capabilities/T0-suggest.yaml`; `capabilities/T1-assist.yaml`; `capabilities/T2-auto.yaml`. | Decide whether T0/T1/T2 is capability-level taxonomy or must be renamed. |
| SLD-015 | P2 | Capacity targets are not tied to deployment-context overlays. | `PRD.md:426-438`; missing OpenTofu context dirs. | Add context overlays in capacity and IaC. |
| SLD-016 | P2 | Per-seat licensing exists but is not mapped to current tenant classes. | `contracts/openapi/slides.yaml:171`; `contracts/openapi/slides.yaml:409`; `sdk-plan.md:71`. | Map per-seat licensing to `paid` and revenue-share billing semantics. |
| SLD-017 | P3 | Chat history contains audit-cohort context but no extra slides product decisions. | Chat lines `16424` and `16439`. | Keep current prompt and canonical docs as controlling sources. |
| SLD-018 | P3 | Existing product parity matrix includes sources by label but not durable public URLs for every row. | `competitor-parity-matrix.md:142-146`. | Attach source URLs in future parity matrix refreshes. |

### 4.1 Finding severity counts

390. P0 findings: 0.
391. P1 findings: 5.
392. P2 findings: 11.
393. P3 findings: 2.
394. Highest severity rationale: missing OpenTofu, six-context, OCI Always Free, OS matrix, and architecture-substance evidence block canonical deployment claims.
395. No P0 was assigned because no production service runtime failure was evidenced inside the audit scope.
396. P1 findings are documentation/specification blockers for truthful deployability claims.
397. P2 findings are material coherence and doctrine gaps that can be remediated without proving the service is unsafe.
398. P3 findings are low-risk audit hygiene issues.

### 4.2 Five cross-cutting constraints evaluation

399. Multi-context evaluated: yes.
400. Multi-context result: fail for local evidence; six contexts are not represented under service-local OpenTofu IaC.
401. OpenTofu IaC evaluated: yes.
402. OpenTofu result: fail; Helm/Kustomize exists but OpenTofu context modules do not.
403. OS support evaluated: yes.
404. OS support result: fail; no service-local supported OS manifest was found.
405. Rust strict evaluated: yes.
406. Rust strict result: pass for actual file extensions, fail for documentation drift around JS load tests and WeasyPrint classification.
407. OCI Always Free evaluated: yes.
408. OCI Always Free result: fail; no `iac/oci-guest/always-free/` profile exists.

## Section 5 - Open questions

409. OQ-001: Should `capabilities/T0/T1/T2` remain as internal AI autonomy labels, or should they be renamed during Wave 15J to avoid any "tier" semantics leaking into commercial policy?
410. OQ-002: Should WeasyPrint remain as a sandboxed third-party renderer under an explicit exception, or should the default PDF path move to Chromium-headless or a Rust-native renderer?
411. OQ-003: Should the accepted PPTX round-trippable subset be published as a user-facing compatibility document before migration tooling encourages PowerPoint imports?
412. OQ-004: Should slides have a service-local OpenTofu module for each of all six contexts, or should some contexts be explicitly marked unsupported with rationale?
413. OQ-005: What is the canonical schema for `tenant_class` in service manifests, contracts, and OpenTofu variables after the current prompt's three-class model is reconciled with the older two-class memory note?
414. OQ-006: Should demo-trial broadcast viewer caps be hard-coded by tenant policy, by OpenTofu variable, by entitlement service, or by all three?
415. OQ-007: Should revenue-share tenants be allowed AI full-deck generation at cost, or must AI generation be separately usage-billed to prevent gross-margin leakage?
416. OQ-008: Should present/broadcast SLOs differ by deployment context because on-prem and colo network quality is tenant-controlled?
417. OQ-009: Should the missing `AltTextSuggested` AsyncAPI event be added, or is `AiDesignSuggested` intended to subsume it?
418. OQ-010: Should the root `README.md` be required for every microservice, given this audit prompt asked to read it and slides lacks it?
419. OQ-011: Should architecture anchor-sweep files be visibly marked `draft` until content-pass review completes?
420. OQ-012: Should the old benchmark artifact be retired entirely once the new no-tenant-class-drift benchmark report exists?
421. OQ-013: Should the service-level OS support manifest include browser matrices for Leptos/WASM in addition to server operating systems?
422. OQ-014: Should per-slide ACL and named-block ACL be treated as paid-only entitlements, or should all tenant classes receive the same feature quality with only usage caps changing?
423. OQ-015: Should charts embedded from sheets degrade read-only under demo-trial caps, or should they block once usage caps are exhausted?
424. OQ-016: Should import scanners be disabled for demo-trial tenants to save cost, or is that forbidden because quality/security must remain uniform across tenant classes?
425. OQ-017: Should MP4 export be disabled by quota in demo-trial tenants, or should it be available with hard rate limits?
426. OQ-018: Should `cost-budget.md:74` be rewritten from Enterprise/per-seat language into `paid` plus `revenue_share` overlays?
427. OQ-019: Should chat-history references remain outside service docs, or should audit deliverables capture only durable repo/file evidence?
428. OQ-020: Should future audit automation reject service docs that contain demo_trial/paid/paid/compliance_pack after Wave 15J?

### 5.1 Recommended next control actions

429. Action 1: rewrite `ARCHITECTURE.md` into a service-specific architecture or mark it draft.
430. Action 2: retire or rewrite `tenant-class-adoption/tenant-class-adoption-record.md` under Wave 15J.
431. Action 3: update onboarding, FAQ, tutorial, migration playbook, and benchmark artifacts that still teach retired vocabulary.
432. Action 4: add tenant-class fields to manifest, capacity model, cost budget, contracts, and OpenTofu variables once the canonical schema is available.
433. Action 5: create OpenTofu context directories for all six deployment contexts or explicitly mark unsupported contexts.
434. Action 6: add `iac/oci-guest/always-free/` for demo-trial infrastructure caps.
435. Action 7: add a service-local supported OS manifest.
436. Action 8: refresh PRD open questions to cite accepted ADRs.
437. Action 9: reconcile `AltTextSuggested` between PRD and AsyncAPI.
438. Action 10: replace JS load-test references with Rust-native or approved load tooling.
439. Action 11: classify WeasyPrint as a deliberate exception or replace it.
440. Action 12: keep the new performance report as the benchmark target source until a live harness produces current measurements.

### 5.2 Evidence ledger by artifact family

441. Ledger PRD-001: `PRD.md:24-36` proves the product is a presentation suite, not a generic media renderer.
442. Ledger PRD-002: `PRD.md:40-47` proves the service already has tenant-outcome claims that can be benchmarked.
443. Ledger PRD-003: `PRD.md:51-87` proves the feature surface includes authoring, collaboration, presenting, export, AI, ACL, and publishing.
444. Ledger PRD-004: `PRD.md:90-108` proves the service has a numeric performance target base.
445. Ledger PRD-005: `PRD.md:127-133` proves audit-chain seals are first-class, not a later add-on.
446. Ledger PRD-006: `PRD.md:145-148` proves data residency belongs in the slides service boundary.
447. Ledger PRD-007: `PRD.md:287-302` proves slides depends on other microservices through SDK/API surfaces rather than owning their internals.
448. Ledger PRD-008: `PRD.md:321-352` proves event ownership is intended, but the event list needs contract reconciliation.
449. Ledger PRD-009: `PRD.md:374-404` proves counterpart awareness already existed before this audit.
450. Ledger PRD-010: `PRD.md:426-438` proves capacity numbers exist, but not by deployment context.
451. Ledger ADR-001: `ADR-SLIDES-0001` is the CRDT decision anchor for collaboration semantics.
452. Ledger ADR-002: `ADR-SLIDES-0002-rendering-canvas-substrate.md:55-68` is the rendering and Leptos/WASM decision anchor.
453. Ledger ADR-003: `ADR-SLIDES-0002-rendering-canvas-substrate.md:72-140` rejects JS framework alternatives for code-share and frame-budget reasons.
454. Ledger ADR-004: `ADR-SLIDES-0003-export-pipeline-fidelity.md:57-91` is the export/import decision anchor.
455. Ledger ADR-005: `ADR-SLIDES-0003-export-pipeline-fidelity.md:193-204` defines CI lanes and supply-chain controls for export workers.
456. Ledger ADR-006: `ADR-SLIDES-0004` is the reduced-motion and animation decision anchor.
457. Ledger ADR-007: `ADR-SLIDES-0005` is the broadcast-mode and LiveKit reuse anchor.
458. Ledger ADR-008: `ADR-SLIDES-0006` is the AI generation and governance anchor.
459. Ledger ADR-009: `ADR-SLIDES-0007` is the per-slide ACL granularity anchor.
460. Ledger ADR-010: `ADR-SLIDES-0008` is the chart live-link to sheets anchor.
461. Ledger CONTRACT-001: `contracts/openapi/slides.yaml:171` proves per-seat license checks exist in the editor session contract.
462. Ledger CONTRACT-002: `contracts/openapi/slides.yaml:327-360` proves AI design, full-deck generation, and alt-text endpoints are exposed.
463. Ledger CONTRACT-003: `contracts/openapi/slides.yaml:409` proves per-seat exceeded response semantics exist.
464. Ledger CONTRACT-004: `contracts/asyncapi/slides-events.yaml:51-77` proves workflow bus output and input channels exist.
465. Ledger CONTRACT-005: `contracts/proto/slides.proto:222-268` proves RPC shape exists for deck, slide, ACL, export, import, broadcast, and chart operations.
466. Ledger SLO-001: `slos/deck-open-latency.openslo.yaml:16` anchors deck-open SLO evidence.
467. Ledger SLO-002: `slos/save-latency.openslo.yaml:16` anchors save latency evidence.
468. Ledger SLO-003: `slos/collab-cursor-sync-latency.openslo.yaml:16` anchors cursor sync evidence.
469. Ledger SLO-004: `slos/present-mode-transition-latency.openslo.yaml:16` anchors present-mode transition evidence.
470. Ledger SLO-005: `slos/export-pptx-latency.openslo.yaml:16` anchors PPTX export evidence.
471. Ledger SLO-006: `slos/export-pdf-latency.openslo.yaml:16` anchors PDF export evidence.
472. Ledger SLO-007: `slos/export-mp4-latency.openslo.yaml:16` anchors MP4 export evidence.
473. Ledger SLO-008: `slos/crdt-merge-no-silent-loss.openslo.yaml` anchors no-silent-loss evidence.
474. Ledger SLO-009: `slos/broadcast-mode-availability.openslo.yaml:16` anchors broadcast availability evidence.
475. Ledger OPS-001: `failure-modes.md:53` proves tenancy failure falls back to fail-closed per-seat ACL evaluation.
476. Ledger OPS-002: `threat-model.md:65` proves deck save repudiation is mitigated by audit-chain seals.
477. Ledger OPS-003: `cost-budget.md:74` proves old Enterprise/per-seat wording still needs tenant-class remapping.
478. Ledger OPS-004: `sdk-plan.md:71` proves SDK planning uses per-seat licensing vocabulary.
479. Ledger OPS-005: runbook coverage exists for animation rollback, attachment restore, broadcast degradation, CRDT conflict, export failure, ACL drift, and theme corruption.
480. Ledger IAC-001: `iac/helm/Chart.yaml` proves Helm packaging exists.
481. Ledger IAC-002: `iac/helm/templates/deployment.yaml` proves a Kubernetes deployment template exists.
482. Ledger IAC-003: `iac/kustomize/base/kustomization.yaml` proves Kustomize base packaging exists.
483. Ledger IAC-004: `iac/kustomize/overlays/pack-eu/kustomization.yaml` proves pack-specific overlay packaging exists.
484. Ledger IAC-005: `iac/kustomize/overlays/pack-kr/kustomization.yaml` proves pack-specific overlay packaging exists.
485. Ledger IAC-006: no canonical OpenTofu context directory is present in the service-local inventory.
486. Ledger IAC-007: no OCI Always Free profile directory is present in the service-local inventory.
487. Ledger DOC-001: `ARCHITECTURE.md:1-3` proves the architecture document is an anchor-sweep artifact requiring content pass.
488. Ledger DOC-002: `ARCHITECTURE.md:24-35` proves the architecture document contains unknown context markers.
489. Ledger DOC-003: `ARCHITECTURE.md:506-518` proves the architecture document references generic runtime/IaC checks rather than slides-specific OpenTofu modules.
490. Ledger DOC-004: `ARCHITECTURE.md:568-579` proves deployment text still centers Helm/Kustomize.
491. Ledger COUNTERPART-001: `competitor-parity-matrix.md:17-36` covers authoring parity.
492. Ledger COUNTERPART-002: `competitor-parity-matrix.md:38-49` covers collaboration parity.
493. Ledger COUNTERPART-003: `competitor-parity-matrix.md:51-61` covers present and broadcast parity.
494. Ledger COUNTERPART-004: `competitor-parity-matrix.md:63-73` covers import/export parity.
495. Ledger COUNTERPART-005: `competitor-parity-matrix.md:75-85` covers accessibility parity.
496. Ledger COUNTERPART-006: `competitor-parity-matrix.md:87-102` covers AI parity.
497. Ledger COUNTERPART-007: `competitor-parity-matrix.md:104-115` covers security and governance parity.
498. Ledger COUNTERPART-008: `competitor-parity-matrix.md:117-130` covers historical performance parity estimates.
499. Ledger RETIRE-001: `tenant-class-adoption/tenant-class-adoption-record.md:9-150` is a complete legacy commercial-level matrix and should be retired, not patched into new tenant classes.
500. Ledger RETIRE-002: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:13-31` contains historical benchmark rows with retired labels.
501. Ledger RETIRE-003: `benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:84-95` contains historical cost rows with retired labels.
502. Ledger RETIRE-004: `onboarding/slides-engineer-first-week.md:45-71` teaches retired terminology to engineers.
503. Ledger RETIRE-005: `migration-playbooks/from-google-slides-and-powerpoint.md:89-99` teaches retired migration eligibility terms.
504. Ledger RETIRE-006: `tutorials/build-investor-deck-with-charts-and-collab.md:15-164` teaches retired capacity terms to tutorial readers.
505. Ledger RETIRE-007: `faqs/slides-engineer-faq.md:22-74` teaches retired hardware/capacity terms to engineers.
506. Ledger TENANT-001: no service-local file uses `tenant_class`.
507. Ledger TENANT-002: no service-local file uses `demo_trial`.
508. Ledger TENANT-003: no service-local file uses `revenue_share`.
509. Ledger TENANT-004: existing per-seat terms appear in contract and SDK docs, but they do not express the three-class model requested for this audit.
510. Ledger CHAT-001: chat line `16424` places slides inside the current active audit wave.
511. Ledger CHAT-002: chat line `16439` places slides inside the Phase 3 first cohort and emphasizes deliverable verification.
512. Ledger CHAT-003: no chat-history match superseded the current user prompt's no-tenant-class-drift and three-tenant-class instructions.

### 5.3 Handoff boundaries

513. Handoff B-001: cloud-iac owns reusable OpenTofu substrate patterns, but slides owns its service-specific context modules and variables.
514. Handoff B-002: tenancy owns entitlement truth, but slides owns how per-seat and tenant-class signals affect editor, broadcast, export, and AI behavior.
515. Handoff B-003: audit-chain owns seal verification, but slides owns the event emission points listed in `PRD.md:127-133`.
516. Handoff B-004: sheets owns spreadsheet data, but slides owns chart embed refresh, revocation, and render semantics.
517. Handoff B-005: messenger or LiveKit substrate owns media room primitives, but slides owns broadcast-mode presentation semantics.
518. Handoff B-006: foundry-runtime owns AI model execution, but slides owns deck-generation policy, review gates, provenance, and UX outcomes.
519. Handoff B-007: application shell owns hosting navigation, but slides owns editor/presenter surface behavior and SLOs.
520. Handoff B-008: observability owns common telemetry infrastructure, but slides owns metric names, SLO thresholds, and alert playbooks.
521. Handoff B-009: security platform owns scanner updates and sandbox primitives, but slides owns import/export trust boundaries.
522. Handoff B-010: compliance platform owns pack definitions, but slides owns per-pack data residency, retention, and presentation-specific compliance behavior.

<!-- ORCHESTRATOR REPORT
  µservice: slides
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/slides/coherence-audit-2026-05-20.md: 666 lines
    - /Users/jasonlee/oyatie/microservices/slides/feature-parity-matrix-2026-05-20.md: 409 lines
    - /Users/jasonlee/oyatie/microservices/slides/performance-benchmark-numbers-2026-05-20.md: 328 lines
  inventory_files_seen: 129
  inventory_lines_read: 17999
  chat_history_matches_processed: 71
  findings_p0: 0
  findings_p1: 5
  findings_p2: 11
  findings_p3: 2
  tier_retirement_candidates_found: 30 + onboarding/slides-engineer-first-week.md:45,46,71; migration-playbooks/from-google-slides-and-powerpoint.md:89,99; tutorials/build-investor-deck-with-charts-and-collab.md:15,164; benchmarks/slides-vs-google-slides-vs-powerpoint-vs-keynote-vs-pitch.md:13,21,29,31,37,74,86,87,95; faqs/slides-engineer-faq.md:22,43,74; tenant-class-adoption/tenant-class-adoption-record.md:13,48,50,85,87,117,121,123,135,137,150
  tenant_class_adoption_gaps: yes - no tenant_class/demo_trial/revenue_share semantics found in the slides path; per-seat language exists but is not mapped to the three-class model.
  top_3_counterparts_confirmed: Google Slides / Microsoft PowerPoint Online / Pitch
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1403
-->
