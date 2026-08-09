---
doc_class: Implementation-Plan
doc_id: IP-ADR-0341-Cellular-Promotion-Gates
microservice: cell-lifecycle
status: PROPOSED
date: 2026-05-21
owner_team: axis-cellular
bounded_context: cell-logical-state-machine
implementation_phase: documentation-stage-only
rust_code_status: not-authored-in-this-wave
source_adrs:
  - ADR-0341
  - ADR-0248
  - ADR-0351
  - ADR-0322
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0186
  - ADR-0044
  - ADR-0330
enforced_by:
  - oya-check-cell-promotion-gates
  - oya-governance-cell-tier-promotion-evidence
  - oya-governance-cell-tier-numbering-convention
  - oya-governance-cell-promotion-quiet-window
  - oya-governance-cell-orchestrator-binding
  - Kyverno enforce-cell-promotion-gates
---

# IP-ADR-0341: cell-lifecycle adoption of Cellular Promotion Gates

## 1. Status and Lifecycle
001. This IP is PROPOSED for the documentation-stage propagation of ADR-0341 into `cell-lifecycle`.
002. The IP amends only documentation and machine-readable manifest declarations in this wave.
003. The IP becomes ACCEPTED only when a later implementation wave proves the gate-evaluation behavior through Rust tests, API contracts, observability receipts, and audit-chain events.
004. Acceptance requires the downstream implementation to preserve the single-concern boundary from ADR-0351.
005. The downstream implementation must not move infrastructure provisioning into `cell-lifecycle`; cloud-iac remains the provisioning authority.
006. The downstream implementation must not move tenant migration into `cell-lifecycle`; cell-rebalancer remains the migration workflow authority.
007. The downstream implementation must not move request routing into `cell-lifecycle`; api-gateway remains the routing authority.
008. The downstream implementation must not rewrite lifecycle history; history remains append-only per the current PRD and ARCH.
009. The downstream implementation must not silently accept promotion evidence that lacks the six ADR-0341 gate inputs.
010. This IP is intentionally stricter than the existing IP-CL-002 evidence-pack plan because ADR-0341 binds the exact Tier 0..4 gates and enforcement lanes.
011. This IP treats the existing `Promoted-T4` through `Promoted-T0` states as local state-machine projections of ADR-0248 cellular tier eligibility.
012. The local state-machine names remain stable even if future topology terms change, because audit-chain replay needs durable transition labels.
013. A future replacement of a telemetry primitive changes adapter mapping, not Cell aggregate semantics.
014. A future replacement of a policy admission primitive changes the admission adapter, not the gate contract.
015. A future replacement of a manifest-update primitive changes the self-modification writer, not the required `cell_promotion_history` semantics.
016. The primary authoring risk is drifting from ADR-0341 thresholds while trying to describe local nuance.
017. This IP resolves that risk by treating ADR-0341 as normative and describing only the `cell-lifecycle` contribution around it.
018. The stop condition for this document-stage slice is a manifest declaration, PRD adoption block, ARCH integration block, and this IP passing citation/cohesion checks.

## 2. Purpose
019. ADR-0341 defines explicit machine-checkable promotion and demotion criteria for every cellular Tier 0..4 edge.
020. `cell-lifecycle` contributes by owning the logical lifecycle state machine that records whether a cell is Registered, Activated, promoted for a tier, Draining, or Decommissioned.
021. The service is not the only gate input producer; it is the transition authority that refuses to persist an eligible state without evidence from the input owners.
022. The six gates are error-budget intact, warm-soak duration, canary cohort SLO compliance, cell-mesh health, tenant-class coverage, and compliance-pack coverage.
023. Error-budget evidence comes through observability and OpenSLO surfaces, then lands in the lifecycle transition evidence pack.
024. Warm-soak evidence comes from cell activation and tier-entry timestamps stored in lifecycle history with HLC ordering.
025. Canary cohort evidence comes from ADR-0186 canary windows and is accepted as a digest, not as raw telemetry payload.
026. Cell-mesh health evidence comes from ADR-0044 inter-cell tunnel metrics and is accepted as a bounded receipt.
027. Tenant-class coverage comes from tenancy and must prove both demo_trial and paid coverage per ADR-0330 before promotion proceeds.
028. Compliance-pack coverage comes from pack sign-off records per ADR-0251 and is stored by receipt id plus digest.
029. `cell-lifecycle` must reject a promotion command when any gate receipt is missing, stale, mismatched to the cell id, or mismatched to the target tier edge.
030. `cell-lifecycle` must reject a promotion command when the gate snapshot hash does not match the evidence pack hash recorded in the request.
031. `cell-lifecycle` must reject a promotion command when a Cedar permit authorizes the caller but the evidence pack fails the domain gate rules.
032. `cell-lifecycle` must reject a promotion command when evidence is green for a different direction than the requested transition.
033. `cell-lifecycle` must record all refusals that matter for audit replay without widening caller permissions.
034. The service exists to make the durable state transition boring, replayable, and mechanically explainable.

## 3. Boundaries
035. `cell-lifecycle` owns the Cell aggregate state, LifecycleHistory append log, and EvidencePack references.
036. `cell-lifecycle` owns transition validation for `Activated -> Promoted-T4`, `Promoted-T4 -> Promoted-T3`, `Promoted-T3 -> Promoted-T2`, `Promoted-T2 -> Promoted-T1`, and `Promoted-T1 -> Promoted-T0`.
037. `cell-lifecycle` owns demotion and rollback state transitions that move a cell to a less critical eligibility level or to Draining.
038. `cell-lifecycle` owns command idempotency, current-state compare-and-swap, and HLC timestamp ordering for lifecycle transitions.
039. cloud-iac owns cell infrastructure readiness, node labels, node pools, OpenTofu state, and Kyverno-admitted topology mutation.
040. cell-rebalancer owns resident migration, drain job execution, hot-cell relief movement, and resident-zero proofs.
041. tenancy owns tenant-to-cell residency, tenant_class coverage, and per-tenant pack applicability records.
042. observability owns SLO budget, canary cohort, mesh success, alert burst, quiet-window, and telemetry-window receipts.
043. audit-chain owns event sealing, Merkle proofs, transition event immutability, and evidence pack attestations.
044. policy-cedar owns authorization decisions, principal scoping, and operator or automation permit evaluation.
045. api-gateway owns request routing, carrier conflict handling, unsupported version responses, and user-visible routing behavior.
046. `cell-lifecycle` never fabricates gate evidence when a dependency is down.
047. A dependency outage maps to a typed refusal or delayed transition, not to a guessed green gate.
048. This boundary preserves ADR-0245 substrate layering by keeping the product surface unaware of tier movement mechanics.

## 4. Tier Transition Model
049. Tier 0 remains the highest blast-radius and most isolated tier per ADR-0248 and ADR-0341.
050. Tier 4 remains best-effort or edge-class and is the lowest blast-radius tier in this model.
051. `Activated -> Promoted-T4` means the cell is eligible for the lowest criticality placement after initial readiness evidence.
052. `Promoted-T4 -> Promoted-T3` means the cell has completed the Tier 3 warm-soak, quiet-window, and six-gate evidence package.
053. `Promoted-T3 -> Promoted-T2` means the cell has completed stronger capability-class gates and can accept capability workloads.
054. `Promoted-T2 -> Promoted-T1` means the cell can accept substrate workloads under Tier 1 Kata isolation expectations.
055. `Promoted-T1 -> Promoted-T0` means the cell can accept foundation workloads and therefore requires the strongest evidence trail.
056. `cell-lifecycle` records the state transition only after the gate package names the exact `from_tier` and `to_tier`.
057. The state transition stores both lexical state names and numeric tiers to make audits resilient to display-name changes.
058. Promotion direction is interpreted as moving toward a more critical tier when the target number is lower.
059. Demotion direction is interpreted as moving toward a less critical tier when the target number is higher.
060. The lane `oya-governance-cell-tier-numbering-convention` protects this interpretation from inversion.
061. Local command handlers must return tier-numbering refusal reasons clearly because wrong direction is a safety issue, not a data-entry mistake.
062. Emergency override evidence still records the skipped gate snapshot; it does not erase the normal gate contract.
063. The domain model must support override event references without making override the normal command path.
064. The state machine must keep the same idempotency discipline for auto-promotion, demotion, manual commands, and emergency override.

## 5. Gate Evidence Responsibilities
065. Gate 1 evidence uses an error-budget receipt from observability proving at least 99 percent of the SLO budget remains on the current tier.
066. The receipt must include service, cell_id, current_tier, target_tier, measurement window, SLO id, budget remaining, and digest.
067. Gate 1 refusal is mandatory when the SLO id does not match `cell-lifecycle` manifest SLO references.
068. Gate 2 evidence uses lifecycle history to prove the warm-soak floor for the requested edge.
069. The warm-soak floor is 7 days for Tier 0 to 1, 14 days for Tier 1 to 2, 28 days for Tier 2 to 3, and 56 days for Tier 3 to 4, with inverse edges requiring symmetric proof when graduating into more critical placement.
070. The local domain must compare HLC timestamps with clock-skew tolerance recorded in the evidence pack, not with wall-clock strings alone.
071. Gate 3 evidence uses canary cohort SLO compliance proving at least 99.5 percent success over the warm-soak window.
072. The receipt must bind the canary cohort id to the same cell_id and candidate tier edge.
073. Gate 4 evidence uses cross-cell call success proving at least 99.95 percent mesh success over the warm-soak window.
074. The receipt must name the inter-cell mesh telemetry source and the cell-pair coverage basis.
075. Gate 5 evidence uses tenancy receipts proving demo_trial and paid coverage on the current tier.
076. The tenant-class receipt must distinguish zero-resident applicability from missing-class evidence.
077. Gate 6 evidence uses compliance-pack receipts proving each applicable pack is signed off on the current tier.
078. The compliance receipt must name pack ids from the manifest and cannot satisfy the gate with a pack family label alone.
079. `cell-lifecycle` stores gate receipt ids and hashes, never raw tenant payloads, canary traces, compliance dossiers, or mesh logs.
080. This keeps the service inside metadata and audit evidence boundaries while still making transition replay possible.

## 6. Manifest Declaration
081. The manifest gains `cell_promotion_gates` to describe local gate applicability.
082. `applicable_tiers` is `[0, 1, 2, 3, 4]` because `cell-lifecycle` can record every lifecycle promotion edge.
083. `cellular_deployment_pattern` is `substrate_dedicated` because lifecycle state is a substrate control-plane concern rather than a product feature.
084. `default_initial_tier` is `0` because the `cell-lifecycle` service itself is Tier 0 substrate control-plane state.
085. `promotion_window_per_edge_seconds` follows ADR-0341 floors exactly and is recorded in seconds for machine comparison.
086. `quiet_window_per_edge_seconds` records the no-alert window for each edge so automation cannot infer a missing default.
087. `compliance_pack_floor` lists the manifest's applicable packs at the service boundary.
088. `evidence_sources` maps each gate to the owning microservice or substrate surface.
089. `enforced_by` records the exact ADR-0341 lanes that will govern the declaration.
090. `lifecycle_state_mapping` maps numeric tiers to local states so manifest readers do not need to parse prose.
091. The manifest gains `cell_promotion_history` as an initially empty array.
092. Empty history is meaningful: no promotion event has been recorded by the documentation-stage artifact.
093. Future entries must include `event_id`, `from_tier`, `to_tier`, `evaluator_version`, and `gate_snapshot_sha256`.
094. Future entries should include `cell_id` and `recorded_at` when the manifest schema accepts those fields.
095. Future entries must be written only by the self-modification path or by a reviewed operational evidence PR.
096. Manual edits to history without an audit-chain event id should fail `oya-governance-cell-tier-promotion-evidence`.

## 7. PRD Adoption
097. The PRD adoption block must state that ADR-0341 is now a functional requirement for promotion behavior.
098. The PRD must preserve the existing product boundary: logical state machine only.
099. The PRD must make the six gate inputs a product acceptance condition for the `promote-cell` capability.
100. The PRD must say the lifecycle API exposes evidence failures as typed refusal reasons.
101. The PRD must not claim the Rust implementation exists in this wave.
102. The PRD must not imply a tenant-visible feature; tier movement remains substrate work.
103. The PRD must name the difference between routine promotion, demotion, and emergency override.
104. The PRD must describe status lifecycle: this IP remains PROPOSED until implementation evidence lands.
105. The PRD must name OpenAPI 3.2.0 because downstream request and response schemas carry evidence digests.
106. The PRD can mention AsyncAPI 3.1.0 only as a downstream event surface if lifecycle events later publish channels.

## 8. ARCH Integration
107. The ARCH integration block must attach ADR-0341 to the hexagonal ports already documented.
108. `ObservabilityGatePort` becomes the source for error-budget, canary, mesh, alert, and quiet-window receipts.
109. `TenancyResidentCountPort` becomes the source for tenant-class coverage receipts.
110. A compliance-pack receipt port remains implied by tenancy, compliance, and policy integration until implementation narrows the adapter.
111. `AuditChainEmitterPort` seals `cell.promotion.executed`, `cell.promotion.demoted`, and `cell.promotion.override`.
112. `CedarAuthorizationPort` checks whether a principal may request or automate a transition.
113. `CellRegistryRepository` preserves state, lifecycle_version, and current tier.
114. `LifecycleHistoryRepository` preserves from_state, to_state, from_tier, to_tier, gate_snapshot_sha256, and audit event reference.
115. `HotLookupCache` can cache current state but never becomes a gate source of truth.
116. Postgres remains the source of truth for lifecycle and evidence references.
117. Valkey remains bounded hot lookup only.
118. The architecture must fail closed when evidence is unavailable.
119. The architecture must preserve deterministic replay across regions with HLC ordering.
120. Under Accepted ADR-0632, the architecture must limit any public surface to REST/OpenAPI 3.2.0, versioned webhooks described by AsyncAPI 3.1.0 with CloudEvents 1.0.2 where its stable HTTP binding applies, and deliberate SSE/WebSocket streaming as applicable; capable public edges prefer HTTP/3 with mandatory HTTP/2 fallback. gRPC/proto3 remains internal-only over HTTP/2 with mTLS and TLS 1.3, and neither public gRPC nor GraphQL command paths are allowed.

## 9. API and Event Surface
121. The existing OpenAPI 3.2.0 contract should grow promotion request fields only in the later implementation wave.
122. Required promotion request fields should include `cell_id`, `from_tier`, `to_tier`, `evidence_pack_id`, `gate_snapshot_sha256`, and `idempotency_key`.
123. Required promotion response fields should include `cell_id`, `state`, `tier`, `lifecycle_version`, `audit_chain_event_id`, and `gate_snapshot_sha256`.
124. Refusal responses should include `refusal_code`, `missing_gate`, `stale_gate`, `mismatched_tier_edge`, or `cedar_decision_id` where applicable.
125. The contract should keep raw evidence links out of user-visible responses unless the caller is authorized for audit detail.
126. AsyncAPI 3.1.0 is applicable if lifecycle transitions publish event channels for internal consumers.
127. Candidate channels are `cell.lifecycle.transition.accepted`, `cell.lifecycle.transition.rejected`, `cell.promotion.executed`, `cell.promotion.demoted`, and `cell.promotion.override`.
128. AsyncAPI messages should carry event id, cell id, tier edge, HLC timestamp, audit-chain event id, and evidence digest.
129. AsyncAPI messages should not carry raw compliance documents, raw canary traces, or tenant payloads.
130. The downstream implementation should use proto3 only for internal gRPC automation surfaces over HTTP/2 with mTLS and TLS 1.3, and reserve stable field tags for tier edges and snapshot digests.
131. The public API version carrier remains governed by the existing tenant_version_pinning block and ADR-0342-era carrier discipline already present in the manifest.
132. `cell-lifecycle` does not define a separate public versioning doctrine for ADR-0341.

## 10. Hyperscaler Precedents
133. AWS cellular architecture informs the core shape: small cells bound blast radius, and promotion must be a control-plane decision backed by objective health signals.
134. AWS shuffle sharding informs tenant impact control: a lifecycle transition should never widen correlated tenant failure by moving cells without evidence.
135. AWS static stability informs failure posture: existing cell state must remain usable during control-plane outages, while new promotions pause until evidence resumes.
136. Stripe cellular architecture informs financial-grade transition evidence: account or tenant binding should be sticky, explicit, and auditable before movement.
137. Stripe-style payment reliability informs demotion policy: protective movement needs a faster path than routine promotion, because preserving blast radius is more important than promotion throughput.
138. Palantir ontology practice informs typed evidence references: transition facts should be first-class objects with stable ids, not opaque log snippets.
139. Linear's product quality bar informs refusal ergonomics: operators need crisp missing-evidence reasons instead of generic failure pages.
140. Microsoft and Google SRE practice inform error-budget discipline: promotion should consume live SLO facts, not release optimism.
141. The design choice borrowed from AWS is cell isolation plus wave-gated movement, not provider-specific AWS services.
142. The design choice borrowed from Stripe is evidence-backed cell movement for high-stakes workflows, not Stripe-specific payment rail coupling.
143. The design choice borrowed from Palantir is ontology-grade typed state and audit replay, not a monolithic ontology service.
144. The design choice borrowed from Linear is narrow, readable operational surface, not a consumer SaaS UI replication.

## 11. Industry-Leading Comparison
145. Compared with AWS, Oyatie differentiates by binding cellular promotion evidence directly into per-microservice manifests and audit-chain rows.
146. AWS public precedent shows cellular blast-radius control, while Oyatie adds corpus-level manifest propagation so agents and CI can inspect every microservice's declared gate posture.
147. Compared with Stripe, Oyatie differentiates by making demo_trial and paid tenant-class coverage an explicit gate input rather than treating production account coverage as implicit.
148. Compared with Palantir, Oyatie differentiates by constraining lifecycle state to a single microservice instead of allowing platform-wide workflow ownership to sprawl.
149. Compared with Linear, Oyatie differentiates by combining operator clarity with machine-enforced promotion gates, so readable refusal reasons do not weaken safety.
150. The local service does not try to outscale AWS cell operations directly; it composes the same topology principle with auditability, open contracts, and tenant-class doctrine.
151. The local service does not try to out-specialize Stripe payments; it extracts the cell-evidence discipline and applies it to every workload class.
152. The local service does not try to centralize all state like an ontology platform; it references typed facts and leaves source-of-truth ownership distributed.
153. The competitive bar is a lifecycle control plane that can explain every state transition under regulator, SRE, and customer-success pressure.
154. At scale, differentiation comes from reducing promotion ambiguity to a deterministic evidence interface that future agents can safely maintain.

## 12. Twenty-Four-Month Maintainability Outlook
155. Month 0 through 3 focuses on document-stage adoption and schema consistency across manifest, PRD, ARCH, and IP surfaces.
156. Month 3 through 6 should land the Rust gate-validation domain types and OpenAPI request/response schema additions.
157. Month 6 through 9 should land integration tests with fake observability, tenancy, audit-chain, and policy ports.
158. Month 9 through 12 should land the first continuous-evaluation path that submits transition proposals without bypassing Cedar.
159. Month 12 through 15 should move refusal analytics into dashboards so operators can see which gate blocks promotion most often.
160. Month 15 through 18 should add contract tests that prove manifest `cell_promotion_gates` declarations match API and SLO surfaces.
161. Month 18 through 21 should add pack-specific warm-soak overrides when ADR-0251 packs require stricter floors.
162. Month 21 through 24 should run quarterly evidence reviews and prune only dead adapter aliases, never state-machine labels.
163. The invariant that persists for all 24 months is append-only lifecycle history.
164. The invariant that persists for all 24 months is six-gate evidence before routine promotion.
165. The invariant that persists for all 24 months is fail-closed behavior on missing evidence.
166. The invariant that persists for all 24 months is single-concern ownership for lifecycle state.
167. The likely change over 24 months is telemetry source richness, not gate semantics.
168. The likely change over 24 months is compliance-pack floor strictness, not manifest field names.
169. The likely change over 24 months is API response detail for operators, not tenant-visible behavior.
170. The replacement path for a primitive is adapter-first: add new adapter, dual-read receipts, prove identical gate decisions, then sunset the old adapter through ADR-0108.
171. The replacement path never rewrites historical `gate_snapshot_sha256` values.
172. The replacement path never recasts old transition direction or tier numbering.

## 13. Five-Year Outlook
173. At five-year scale, `cell-lifecycle` should be a small but critical substrate service with high audit value and low write volume.
174. The service should serve many read queries for current cell state while accepting relatively few writes for state transitions.
175. The service should run active-active by region with deterministic HLC merge of lifecycle history.
176. The service should retain transition evidence references long enough to satisfy SOC2, ISO27001, HIPAA, PCI, KR-ISMS-P, CSAP, and EU AI Act review windows.
177. The service should support thousands of cells without increasing per-transition gate complexity.
178. The service should support millions of tenants indirectly by consuming tenant-class coverage receipts rather than enumerating tenant rows.
179. The service should preserve stable transition labels so five-year-old audit packs remain intelligible.
180. The service should expose enough contract metadata for SDKs and automation to diagnose refusal reasons without privileged database access.
181. The service should remain boring: promotion decisions are evidence comparisons plus state transitions, not a new planning engine.
182. The service should make high-criticality tier movement slower and lower-criticality recovery faster when safety requires it.
183. The service should keep cost growth sublinear relative to tenants because it scales by cells and transitions, not by every tenant request.
184. The service should keep carbon growth sublinear by caching read projections and avoiding high-cardinality telemetry joins inside the lifecycle service.
185. The service should keep watts per transition bounded by moving raw telemetry aggregation to observability and accepting only compact receipts.
186. The service should keep USD cost dominated by Postgres history retention and audit-chain sealing, not CPU-heavy evaluation loops.
187. The contractual invariant is that a transition can always be explained from manifest declaration, evidence pack digest, LifecycleHistory row, and audit-chain event.
188. The contractual invariant is that no product team can bypass lifecycle state by claiming a local tier exception.

## 14. Horizontal Scalability Path
189. At 10x current cell count, one regional `cell-lifecycle` deployment can scale by increasing read replicas and Valkey cache capacity.
190. The 10x bottleneck is likely Postgres write IOPS during transition bursts, not CPU.
191. The 10x mitigation is batching audit-independent read traffic through cached current-state projections while keeping writes strongly ordered.
192. At 100x current cell count, the service should shard LifecycleHistory by region and cell_id while keeping a compact global index for lookup.
193. The 100x bottleneck is likely cross-region replication lag and audit-chain seal throughput.
194. The 100x mitigation is per-region active writers with HLC conflict rules and asynchronous global evidence views.
195. At 1000x current cell count, the service should partition by cellular region family and compliance pack floor.
196. The 1000x bottleneck is likely compliance receipt fan-in and connection count to dependency services, not local domain computation.
197. The 1000x mitigation is receipt snapshots published by observability, tenancy, and compliance owners on constant-work cadence.
198. CPU scaling dimension is per transition and per evidence verification, not per request routed through api-gateway.
199. RAM scaling dimension is hot lookup cache for current state and refusal reason templates.
200. IOPS scaling dimension is LifecycleHistory append volume plus idempotency-key lookup.
201. Connection scaling dimension is bounded outbound ports to observability, tenancy, audit-chain, policy-cedar, cloud-iac, and cell-rebalancer.
202. Per-cell sharding strategy is `region_id + cell_id` for state and `region_id + cell_id + lifecycle_version` for history.
203. Tier 0 and Tier 1 cells receive dedicated substrate placement and stricter runtime isolation.
204. Tier 2 and Tier 3 cells can use shared capability and application placements while preserving lifecycle state in substrate cells.
205. Tier 4 edge-class cells can use read-heavy projections near edge while writes return to regional substrate authority.
206. The service should not scale by creating one lifecycle database per tenant; that would violate the cell-level control-plane design.

## 15. Cost, CO2, and Energy Outlook
207. Cost trajectory at 10x is dominated by storage retention for lifecycle history and audit-chain references.
208. Cost trajectory at 100x adds regional read replica and cross-region replication overhead.
209. Cost trajectory at 1000x adds compliance-pack receipt fan-in and dashboard cardinality pressure if labels are not bounded.
210. The service keeps USD cost low by storing digests and receipt ids instead of raw evidence payloads.
211. The service keeps watt-hours low by avoiding raw telemetry aggregation inside transition handlers.
212. The service keeps CO2 lower by using constant-work receipt snapshots rather than per-caller evidence recomputation.
213. The service should expose cost attribution labels for transition evaluation, evidence validation, and audit seal phases.
214. The service should track `watt_hours_per_transition` as an operational estimate once sustainability telemetry from ADR-0344 lands.
215. The service should track `co2e_grams_per_transition` using the same regional electricity factor used by platform FinOps.
216. The service should track `usd_per_1000_transitions` because lifecycle writes are rare enough for unit economics to stay intelligible.
217. High write cost is acceptable for Tier 0 transitions because they protect blast radius.
218. High read cost is not acceptable for ordinary state lookup; read path should use bounded cache and pagination.
219. Compliance-heavy cells may cost more per transition due to pack receipts; that cost should be visible rather than hidden.
220. The long-term target is cost transparency, not minimizing away evidence that regulators and SREs require.

## 16. Non-Obvious Gotchas
221. Gate 2 warm-soak is not the same as service uptime; it is tier-edge residency time for a candidate promotion path.
222. Gate 5 tenant-class coverage can fail even when the cell has healthy paid tenants if demo_trial has no coverage receipt.
223. Gate 6 compliance coverage can fail when a pack is applicable but the cell is technically healthy.
224. A Cedar allow decision does not imply evidence sufficiency; authorization and evidence are separate gates.
225. A green canary cohort does not imply mesh health; Gate 3 and Gate 4 are deliberately separate.
226. A successful demotion may be safer than holding a cell at a prestigious tier; tier numbers must not be treated as status badges.
227. A missing quiet-window receipt should block auto-promotion even when all six gates are green at one instant.
228. A stale evidence pack should block promotion even when each individual receipt looks valid in isolation.
229. A retry with the same idempotency key must not append a second LifecycleHistory row.
230. A retry with a different idempotency key must not bypass stale-current-state protection.
231. A Valkey cache hit must not satisfy gate evidence because cache is a projection, not the source of truth.
232. A manifest history update must not precede the audit-chain event it references.
233. A pack-specific warm-soak stricter than ADR-0341 must win over the default floor.
234. A future schema addition must preserve old history replay by making new fields additive or versioned.

## 17. Alternatives Considered
235. Alternative A was to keep the existing IP-CL-002 promotion evidence plan as the only ADR-0341 adoption artifact.
236. Alternative A was rejected because IP-CL-002 predates the exact ADR-0341 lane and manifest field shape.
237. Alternative B was to place all ADR-0341 content in PRD and ARCH only.
238. Alternative B was rejected because ADR-0322 requires a bespoke IP-level substance artifact for doctrine adoption.
239. Alternative C was to make cell-rebalancer own promotion gate state because it already handles movement.
240. Alternative C was rejected because cell-rebalancer owns tenant migration workflow, while `cell-lifecycle` owns Cell state.
241. Alternative D was to let observability auto-promote directly after green metrics.
242. Alternative D was rejected because observability is a gate input owner, not the state-machine authority.
243. Alternative E was to let cloud-iac mutate node labels directly after infrastructure readiness.
244. Alternative E was rejected because infrastructure readiness is only one prerequisite and cannot replace gate evidence.
245. Alternative F was to store raw evidence payloads inside LifecycleHistory.
246. Alternative F was rejected because it increases privacy, compliance, storage, and replay risk without improving gate decision determinism.
247. Alternative G was to allow operator override without audit-chain event reference.
248. Alternative G was rejected because ADR-0341 makes audit evidence mandatory even for emergency override.

## 18. Security and Compliance
249. Every privileged transition requires Cedar authorization before state mutation.
250. Every accepted privileged transition emits an audit-chain event before success is returned.
251. Every refused privileged transition records enough detail for audit and SRE remediation without leaking raw evidence.
252. Evidence packs store digest references and source receipt ids rather than regulated payloads.
253. Compliance-pack receipts must bind pack id, cell id, tier edge, signer, and validity window.
254. Emergency override requires multiparty authorization and records gate evidence even when the gate condition is bypassed.
255. The service should expose a compliance review query that lists promotions by pack, tier edge, and evidence status.
256. The service should expose an SRE review query that lists refusals by missing gate and dependency owner.
257. The service should expose an audit review query that verifies every manifest history entry has an audit-chain event id.
258. The service should preserve tenant context per ADR-0244 in audit rows while avoiding raw tenant payload storage.
259. The service should keep EU AI Act and healthcare pack receipts separate enough for pack-specific reviewers to inspect without full platform privilege.
260. The service should use least-privilege service principals for automated promotion proposals.

## 19. Observability
261. Metric `cell_lifecycle_promotion_attempts_total` should count attempts by result, from_tier, to_tier, and refusal_code.
262. Metric `cell_lifecycle_gate_missing_total` should count missing gate inputs by gate name and dependency owner.
263. Metric `cell_lifecycle_gate_stale_total` should count stale receipts by gate name and evidence age bucket.
264. Metric `cell_lifecycle_transition_duration_seconds` should measure accepted transition latency by phase.
265. Metric `cell_lifecycle_audit_seal_latency_seconds` should isolate audit-chain seal time.
266. Metric `cell_lifecycle_history_append_latency_seconds` should isolate repository write time.
267. Metric labels must avoid tenant id, user id, raw cell name variants, and unbounded evidence ids.
268. Traces should link transition request span, dependency receipt spans, audit-chain seal span, and repository append span.
269. Logs should carry request id, cell id hash, from_tier, to_tier, result, and bounded refusal code.
270. Dashboards should separate routine promotion, demotion, and emergency override.
271. Alerts should fire on audit seal failure, evidence fan-in lag, gate receipt staleness, and HLC replay conflict.
272. Alert routing should go to ops-sre-reliability and axis-cellular, with pack-specific escalation when Gate 6 fails repeatedly.

## 20. Verification Plan
273. Static verification confirms this file has at least 300 lines.
274. Static verification confirms this file references ADR-0341 and ADR-0248 by exact ID.
275. Static verification confirms this file names all six gate inputs.
276. Static verification confirms this file names the ADR-0341 enforced_by lanes.
277. Static verification confirms this file contains no Rust implementation.
278. Static verification confirms manifest JSON parses after adding the ADR-0341 fields.
279. Static verification confirms PRD has a single `ADR-0341 adoption` section.
280. Static verification confirms ARCH has a single `ADR-0341 integration` section.
281. Citation verification runs `cargo run -q -p oya-dev-cli -- gate validate adr-citation --docs-dir docs --decisions-dir docs/decisions`.
282. Cohesion verification runs `cargo run -q -p oya-dev-cli -- gate validate cohesion`.
283. Inventory refresh runs `cargo run -q -p oya-dev-cli -- doc inventory --write`.
284. Downstream implementation verification must add unit tests for tier direction, stale evidence, missing gate, idempotency retry, and audit-chain ordering.
285. Downstream implementation verification must add integration tests with fake observability, tenancy, audit-chain, and policy adapters.
286. Downstream implementation verification must prove no code path stores raw telemetry or compliance payloads in LifecycleHistory.

## 21. Acceptance Criteria
287. AC-001: `microservices/cell-lifecycle/manifest.json` declares `cell_promotion_gates`.
288. AC-002: `microservices/cell-lifecycle/manifest.json` declares `cell_promotion_history`.
289. AC-003: `cell_promotion_gates.applicable_tiers` includes 0, 1, 2, 3, and 4.
290. AC-004: `cell_promotion_gates.cellular_deployment_pattern` is `substrate_dedicated`.
291. AC-005: `cell_promotion_gates.default_initial_tier` is 0 for the Tier 0 substrate placement of `cell-lifecycle`.
292. AC-006: `promotion_window_per_edge_seconds` matches ADR-0341 floors.
293. AC-007: `quiet_window_per_edge_seconds` is explicit.
294. AC-008: `evidence_sources` maps all six gates.
295. AC-009: `cell_promotion_history` is an array.
296. AC-010: PRD adoption explains six gate inputs and lifecycle status.
297. AC-011: ARCH integration explains port mapping and fail-closed behavior.
298. AC-012: This IP names 24-month maintainability behavior.
299. AC-013: This IP names five-year outlook.
300. AC-014: This IP names 10x, 100x, and 1000x scalability path.
301. AC-015: This IP names CO2, watt-hours, and USD cost trajectory.
302. AC-016: This IP cites AWS, Stripe, Palantir, Linear, Microsoft, and Google precedents.
303. AC-017: This IP compares local differentiation against industry leaders.
304. AC-018: This IP records alternatives considered and rejected.
305. AC-019: This IP references OpenAPI 3.2.0 and AsyncAPI 3.1.0 where applicable.
306. AC-020: This IP remains documentation-stage only.

## 22. Handoff to Implementation
307. Implementation wave must start from this IP plus IP-CL-002, not from either document alone.
308. Implementation wave must add typed gate receipt structs before writing handler code.
309. Implementation wave must add state-machine tests before enabling promotion commands.
310. Implementation wave must add repository tests for append-only LifecycleHistory before wiring adapters.
311. Implementation wave must add fake adapter tests for each evidence source before calling real dependencies.
312. Implementation wave must add audit-chain ordering tests before claiming success responses are safe.
313. Implementation wave must add OpenAPI schema changes for evidence fields before SDK generation.
314. Implementation wave must add AsyncAPI channels only if lifecycle transition events are published.
315. Implementation wave must update manifest history only with real audit-chain event ids.
316. Implementation wave must keep cell-rebalancer and cloud-iac as external ports.
317. Implementation wave must keep tenant-class coverage as a tenancy receipt.
318. Implementation wave must keep SLO, canary, and mesh health as observability receipts.
319. Implementation wave must keep compliance pack sign-off as pack receipt references.
320. Implementation wave must keep Cedar authorization separate from evidence sufficiency.
321. Implementation wave must keep emergency override rare, signed, and reviewable.
322. Implementation wave must run full verification before moving this IP to ACCEPTED.

## 23. Stop Condition
323. Stop when the new IP, manifest fields, PRD block, ARCH block, inventory refresh, VCS verification, and signed commit are complete.
324. Stop only after validation output is recorded in the final report.
325. Stop with a plain blocker if signed commit, validation, or push cannot complete.
326. Stop without starting Rust implementation because this wave is document-stage propagation only.
