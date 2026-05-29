---
doc_class: ProductScope
scope_id: SCOPE-WORKFLOW-STUDIO-FIRST-HERO
microservice: workflow-studio
status: Accepted
date: 2026-05-20
owner_team: axis-workflow-studio + council-design-system + council-architecture
agent: codex-workflow-studio-scope
claim_scope: microservices/workflow-studio
purpose: First-product scope deepening for Workflow Studio as both n8n-class hero product and shared authoring substrate.
authoring_boundary: This file is the only edited artifact for the 2026-05-20 workflow-studio hero-scope deepening pass.
source_authorities:
  - microservices/workflow-studio/PRD.md
  - specs/microservices/workflow-studio.json
  - specs/microservices/workflow.json
  - microservices/workflow-studio/decisions/ADR-WFS-001-yjs-crdt-for-collaborative-canvas-editing.md
  - microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md
  - docs/decisions/ADR-0220-consumer-intelligence-substrate.md
  - docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md
  - microservices/workflow-studio/policy/tenant-scope.cedar
  - microservices/workflow-studio/sdk-plan.md
  - microservices/workflow-studio/failure-modes.md
  - microservices/workflow-studio/capacity-model.md
  - microservices/workflow-studio/competitor-parity-matrix.md
  - specs/design-system/workflow-canvas.json
  - specs/design-system/workflow-node-config-panel.json
  - specs/design-system/workflow-replay-timeline.json
---

# Workflow Studio Scope

## Audit Finding
001. Current PRD identifies Workflow Studio as visual workflow authoring product.
002. Current PRD names it n8n-class first hero product.
003. Current PRD says workflow-engine owns durable execution.
004. Current PRD says Studio owns drag-drop canvas.
005. Current PRD says Studio owns canonical JSON DSL round-trip.
006. Current PRD says Studio owns collaborative multi-user editing.
007. Current PRD says Studio owns per-pack node libraries.
008. Current PRD says Studio owns jurisdiction overlay view-switching.
009. Current PRD says Studio owns replay debugger frontend.
010. Current PRD says Studio owns LLM-assist authoring.
011. Current PRD says Studio owns per-seat license-gate enforcement.
012. Current PRD says Studio owns editor session state.
013. Current PRD already says Studio is shared substrate and hero product.
014. Current JSON PRD records north star as feedback_workflow_studio_scope.
015. Current JSON PRD pairs Studio with workflow engine.
016. Current JSON PRD has multi-domain node-library target.
017. Current JSON PRD has offline buffer acceptance criteria.
018. Current JSON PRD has LLM-authored spec validation acceptance criteria.
019. Current JSON PRD has CRDT no-silent-loss acceptance criteria.
020. Current JSON PRD has Cedar policy preview acceptance criteria.
021. Current JSON PRD has per-seat Cedar enforcement acceptance criteria.
022. Current PRD contains competitive claims.
023. Current PRD does not provide a single 2000-line scope artifact.
024. Current PRD does not fully work through six domain examples.
025. Current PRD does not fully name editor screens and modals.
026. Current PRD does not fully specify node-config Cedar envelopes.
027. Current PRD does not fully bind AI generation to ADR-0220 Intelligence.
028. Current PRD does not fully specify TypeScript custom-node SDK lifecycle.
029. Current PRD does not fully specify Rust custom-node SDK lifecycle.
030. Current PRD does not fully specify Python custom-node SDK lifecycle.
031. Current PRD does not index all vendor migration playbooks.
032. Current PRD does not fully spell out execution boundary by microservice.
033. Current PRD does not provide an explicit cross-microservice handoff matrix.
034. Current PRD does not consolidate failure recovery into product scope.
035. Current PRD does not bind SLO targets by tenant tier in one artifact.
036. Current PRD does not distinguish source-backed claims from targets in scope.
037. This file closes those gaps without editing owned companion docs.
038. This file remains subordinate to machine-readable specs where they conflict.
039. This file preserves the workflow-engine execution boundary.
040. This file preserves the no raw cross-tenant access boundary.

## §1 Hero Product Identity
041. Workflow Studio is Oyatie's first hero product surface.
042. Workflow Studio is an n8n-class visual workflow authoring product.
043. Workflow Studio is also the shared workflow authoring substrate.
044. The product promise is visual, collaborative, policy-aware automation.
045. The substrate promise is canonical spec emission for every workflow-aware product.
046. End users experience Studio as a branded editor.
047. Internal microservices consume Studio as a DSL authoring and projection layer.
048. The workflow engine executes; Studio authors and debugs.
049. The ontology service types nodes; Studio renders typed configuration.
050. Tenancy scopes sessions; Studio refuses tenant ambiguity.
051. Cedar gates node configuration; Studio previews policy before save.
052. Intelligence drafts safe workflow candidates; Studio validates before render.
053. Marketplace settles templates, node packs, and commercial workflow grants.
054. Audit-chain records saves, approvals, failures, and policy decisions.
055. Observability records UX and runtime health.
056. The editor must feel useful before the engine is fully exposed.
057. The canvas must be credible for business users.
058. The DSL view must be credible for developers.
059. The replay timeline must be credible for operators.
060. The policy preview must be credible for security teams.
061. The template catalog must be credible for go-to-market.
062. The SDK must be credible for platform tenants.
063. The migration playbooks must be credible for enterprise displacement.
064. The product must not become a BPMN clone.
065. The product must not become a visual-only toy.
066. The product must not hide policy failures until runtime.
067. The product must not execute workflows in the browser.
068. The product must not let LLM output bypass validation.
069. The product must not let tenant secrets enter CRDT state.
070. The product must not create visual state above canonical spec.
071. The source of truth after publication is `workflow_spec.v1.json`.
072. The active collaborative draft source is the Yjs document.
073. The runtime source of truth is workflow-engine event history.
074. The product differentiator is the bridge between all three.
075. North-star workflow: say intent, generate draft, inspect policy, collaborate, save, execute, replay.
076. North-star operator path: open failed run, inspect timeline, confirm policy, retry or compensate.
077. North-star developer path: load JSON, edit graph, round-trip byte-equal, submit PR.
078. North-star admin path: approve node pack, set template catalog, audit Cedar decisions.
079. North-star buyer path: migrate from n8n, Zapier, Make, Workato, or Power Automate with evidence.
080. This identity makes Studio a product, not a documentation appendix.

## §2 Multi-Domain Coverage
081. Studio must ship six first-class domain modes.
082. Domain mode changes palette grouping, templates, examples, and policy hints.
083. Domain mode never changes canonical workflow semantics.
084. Domain mode is not tenant branding.
085. Domain mode is a typed node-library projection.
086. Domain mode can be filtered by pack, tenant entitlement, and jurisdiction.
087. Domain mode can be combined with developer overlays.
088. Domain mode can be combined with audit overlays.
089. Domain mode can be combined with replay overlays.
090. Domain mode must remain keyboard-operable.

### §2.1 Agentic Domain
091. Agentic domain covers human-agent and agent-agent workflows.
092. Worked example: incident triage with reviewer-agent approval.
093. Trigger node: `audit_signal.received`.
094. Context node: `ontology.lookup.incident`.
095. Agent node: `intelligence.suggest.remediation`.
096. Policy node: `cedar.evaluate.autonomy_tier`.
097. Approval node: `human.review.required`.
098. Execution node: `workflow-engine.dispatch.compensating_action`.
099. Audit node: `audit-chain.seal.agent_decision`.
100. The canvas shows autonomy tier on every agentic node.
101. The config panel shows model route, data class, and prompt fence.
102. The policy preview blocks SECRET-class prompt payloads.
103. The replay timeline shows agent output hash, not raw secret text.
104. The template catalog includes incident triage, evidence collection, and postmortem draft.
105. Agentic templates require reviewer identity for T2-cross.
106. Agentic templates require deterministic fallback path when Intelligence is unavailable.
107. Agentic templates require human-review gates for high-risk classification.
108. Agentic nodes cannot call destination microservices without Cedar grant.
109. Agentic nodes cannot claim Foundry as consumer-facing AI.
110. Agentic nodes route consumer AI through Intelligence.
111. Agentic nodes route internal eval and CI through Foundry only when internal.
112. Agentic node config exposes `capability_stage`.
113. Agentic node config exposes `model_route_policy`.
114. Agentic node config exposes `prompt_context_refs`.
115. Agentic node config exposes `human_review_required`.
116. Agentic node config exposes `audit_chain_required`.
117. Agentic node config exposes `fallback_activity`.
118. Agentic graph lint flags unbounded tool calls.
119. Agentic graph lint flags missing reviewer separation.
120. Agentic graph lint flags prompt text without data classification.

### §2.2 Dev Domain
121. Dev domain covers CI/CD, GitOps, issue triage, and developer workflows.
122. Worked example: PR readiness automation with Oya VCS gates.
123. Trigger node: `vcs.changeset.ready`.
124. Validation node: `test.registry.select`.
125. Static gate node: `oya.verify.run`.
126. Policy node: `cedar.evaluate.merge_authority`.
127. Reviewer node: `code-review.request`.
128. Promotion node: `oya-vcs.promote`.
129. Audit node: `audit-chain.seal.merge_evidence`.
130. The canvas shows source branch, target environment, and gate status.
131. The config panel shows allowed commands by runner role.
132. The policy preview blocks CI from reading tenant draft contents.
133. The replay timeline shows test lanes and artifact links.
134. Dev templates include PR validation, dependency upgrade, release train, and incident rollback.
135. Dev domain supports JSON side-by-side with graph.
136. Dev domain supports Git-backed diff preview.
137. Dev domain supports partial run for non-production dry runs.
138. Dev domain forbids production execution from Studio.
139. Dev domain emits ChangeSet references for promotion.
140. Dev domain distinguishes author, reviewer, and promoter identities.
141. Dev node config exposes `repo_ref`.
142. Dev node config exposes `changeset_id`.
143. Dev node config exposes `gate_profile`.
144. Dev node config exposes `environment`.
145. Dev node config exposes `runner_identity`.
146. Dev node config exposes `artifact_ref`.
147. Dev node config exposes `rollback_policy`.
148. Dev graph lint flags unpinned refs.
149. Dev graph lint flags missing not-tested field.
150. Dev graph lint flags bypassed VCS promotion.

### §2.3 Business Domain
151. Business domain covers HR, finance, procurement, sales, and approvals.
152. Worked example: procure-to-pay workflow.
153. Trigger node: `purchase_requisition.created`.
154. Enrichment node: `ontology.lookup.supplier`.
155. Approval node: `manager.approval.request`.
156. Policy node: `cedar.evaluate.spend_authority`.
157. Deal node: `marketplace.deal.accept`.
158. Payment node: `payments.authorize`.
159. Ledger node: `finops.record.commitment`.
160. Audit node: `audit-chain.seal.deal_transition`.
161. The canvas shows spend threshold and counterparty role.
162. The config panel shows DealSet reference and settlement policy.
163. The policy preview blocks unauthorized spend categories.
164. The replay timeline shows obligation, entitlement, and settlement state.
165. Business templates include vendor onboarding, procure-to-pay, month-end close, and asset return.
166. Business domain supports template marketplace packaging.
167. Business domain supports per-tenant catalog curation.
168. Business domain supports office-friendly labels without changing spec names.
169. Business domain prevents a visual-only spreadsheet of approvals.
170. Business domain keeps every approval tied to audit-chain evidence.
171. Business node config exposes `counterparty_role`.
172. Business node config exposes `deal_category`.
173. Business node config exposes `spend_limit`.
174. Business node config exposes `approval_matrix_ref`.
175. Business node config exposes `settlement_terms_ref`.
176. Business node config exposes `cost_center`.
177. Business node config exposes `exception_policy`.
178. Business graph lint flags missing settlement owner.
179. Business graph lint flags approval loops without timeout.
180. Business graph lint flags deal nodes without Cedar action namespace.

### §2.4 Healthcare Domain
181. Healthcare domain covers clinical operations, hospital throughput, care coordination, and compliance.
182. Worked example: discharge planning workflow.
183. Trigger node: `ehr.discharge_ready`.
184. Context node: `ontology.lookup.patient_encounter`.
185. Coordination node: `transport.schedule`.
186. Medication node: `pharmacy.prescription.verify`.
187. Notification node: `messenger.family_notice`.
188. Policy node: `cedar.evaluate.phi_access`.
189. Audit node: `audit-chain.seal.phi_workflow`.
190. The canvas shows PHI data-class markers.
191. The config panel renders secret and PHI fields as protected references.
192. The policy preview blocks users without care-team role.
193. The replay timeline redacts sensitive values by default.
194. Healthcare templates include bed availability prediction, admission, discharge, and Joint Commission prep.
195. Healthcare domain supports HIPAA and HITECH compliance flags.
196. Healthcare domain supports BAA-dependent activation.
197. Healthcare domain supports facility pack overlays.
198. Healthcare domain supports break-glass only through tenant policy.
199. Healthcare domain cannot claim compliance without tenant contract activation.
200. Healthcare domain cannot put PHI in LLM prompts without consent and route approval.
201. Healthcare node config exposes `patient_context_ref`.
202. Healthcare node config exposes `phi_data_class`.
203. Healthcare node config exposes `care_team_scope`.
204. Healthcare node config exposes `facility_id`.
205. Healthcare node config exposes `consent_ref`.
206. Healthcare node config exposes `redaction_policy`.
207. Healthcare node config exposes `clinical_timeout`.
208. Healthcare graph lint flags PHI sent to non-healthcare destination.
209. Healthcare graph lint flags notification without consent.
210. Healthcare graph lint flags unredacted replay history.

### §2.5 Supply-Chain Domain
211. Supply-chain domain covers inventory, procurement, logistics, trade, and warehouse actions.
212. Worked example: supplier disruption response workflow.
213. Trigger node: `connect.supplier_delay.imported`.
214. Context node: `ontology.lookup.purchase_order`.
215. Risk node: `global-trade.screen.route`.
216. Allocation node: `warehouse.reserve.alternate_stock`.
217. Deal node: `marketplace.deal.amend`.
218. Carrier node: `delivery.quote.reroute`.
219. Audit node: `audit-chain.seal.supply_exception`.
220. The canvas shows upstream and downstream dependency edges.
221. The config panel shows incoterm, trade hold, and supplier risk.
222. The policy preview blocks sanctioned counterparty actions.
223. The replay timeline shows stuck transitions and compensation actions.
224. Supply-chain templates include vendor onboarding, procurement, shipment exception, and return logistics.
225. Supply-chain domain supports global-trade holds.
226. Supply-chain domain supports carrier adapter handoffs.
227. Supply-chain domain supports warehouse reservation handoffs.
228. Supply-chain domain supports DealSet amendment cascades.
229. Supply-chain domain prevents hidden cross-tenant row sharing.
230. Supply-chain domain prevents unscoped supplier access.
231. Supply-chain node config exposes `supplier_ref`.
232. Supply-chain node config exposes `purchase_order_ref`.
233. Supply-chain node config exposes `trade_hold_policy`.
234. Supply-chain node config exposes `inventory_reservation_ref`.
235. Supply-chain node config exposes `carrier_service_level`.
236. Supply-chain node config exposes `fallback_supplier_strategy`.
237. Supply-chain node config exposes `compensation_policy`.
238. Supply-chain graph lint flags missing trade screen.
239. Supply-chain graph lint flags inventory mutation without idempotency.
240. Supply-chain graph lint flags carrier call without retry policy.

### §2.6 Delivery Domain
241. Delivery domain covers last-mile, field service, customer notifications, and SLA recovery.
242. Worked example: delayed delivery recovery workflow.
243. Trigger node: `delivery.eta_breach.detected`.
244. Context node: `ontology.lookup.order`.
245. Decision node: `branch.customer_priority`.
246. Notification node: `messenger.notify.customer`.
247. Compensation node: `marketplace.deal.credit.offer`.
248. Replan node: `delivery.route.optimize`.
249. Audit node: `audit-chain.seal.delivery_recovery`.
250. The canvas shows ETA, SLA, route, and compensation eligibility.
251. The config panel shows customer communication policy.
252. The policy preview blocks unauthorized compensation.
253. The replay timeline shows retryable carrier calls and final customer outcome.
254. Delivery templates include route exception, customer notice, field dispatch, and failed-delivery retry.
255. Delivery domain supports real-time signals from delivery systems.
256. Delivery domain supports idempotent customer notifications.
257. Delivery domain supports SLA-based compensation.
258. Delivery domain supports marketplace credit settlement.
259. Delivery domain prevents duplicate notifications on retry.
260. Delivery domain prevents compensation without DealSet authority.
261. Delivery node config exposes `order_ref`.
262. Delivery node config exposes `route_ref`.
263. Delivery node config exposes `customer_contact_policy`.
264. Delivery node config exposes `sla_threshold`.
265. Delivery node config exposes `compensation_cap`.
266. Delivery node config exposes `carrier_retry_policy`.
267. Delivery node config exposes `notification_idempotency_key`.
268. Delivery graph lint flags non-idempotent notification.
269. Delivery graph lint flags compensation without settlement policy.
270. Delivery graph lint flags missing customer preference check.

## §3 Shared Substrate Engine
271. Studio is not the workflow execution engine.
272. Studio is the shared authoring substrate for workflow specs.
273. Workflow-engine is the durable execution runtime.
274. Studio emits the canonical spec consumed by workflow-engine.
275. Studio loads the canonical spec produced by developer flows.
276. Studio projects collaborative Yjs state into canonical spec.
277. Studio validates projected spec before any publish attempt.
278. Studio renders engine replay streams as visual frames.
279. Studio renders engine execution state but does not own that state.
280. Studio uses ontology descriptors to type node configuration.
281. Studio uses tenancy identity to scope sessions.
282. Studio uses Cedar to preview policy impact.
283. Studio uses Intelligence for consumer-facing AI generation.
284. Studio may use Foundry only for internal evaluation flows.
285. Studio uses marketplace for template and node-pack settlement.
286. Studio uses audit-chain for save and policy evidence.
287. Studio uses observability for SLO and health telemetry.
288. Studio uses connect adapters through workflow-engine nodes.
289. Studio uses messenger and mail through workflow-engine nodes.
290. Studio uses payments through workflow-engine nodes.
291. Studio uses treasury through workflow-engine nodes.
292. Studio uses global-trade through workflow-engine nodes.
293. Studio uses delivery through workflow-engine nodes.
294. Studio uses warehouse through workflow-engine nodes.
295. Studio uses control-center for admin visibility only.
296. Studio can be embedded only through approved product surfaces.
297. Studio cannot be embedded as an arbitrary tenant iframe.
298. Studio cannot let other microservices mutate its CRDT state directly.
299. Studio cannot let other microservices bypass its spec projection.
300. Studio cannot let workflow-engine consume raw Yjs documents.
301. Studio cannot let browser state become runtime authority.
302. Studio cannot let templates skip policy preview.
303. Studio cannot let SDKs publish node packs without signing.
304. Studio cannot let SDKs publish marketplace templates directly.
305. Studio cannot persist awareness heartbeats as semantic edits.
306. Studio cannot replay stale telemetry without a warning.
307. Studio cannot show unredacted SECRET values.
308. Studio cannot send cross-tenant data to Intelligence.
309. Studio cannot route consumer AI to Foundry.
310. Studio cannot perform raw database reads against other microservices.
311. Substrate consumer: HR uses Studio for onboarding workflows.
312. Substrate consumer: Payroll uses Studio for pay-run workflows.
313. Substrate consumer: uses Studio for adapter orchestration workflows.
314. Substrate consumer: Marketplace uses Studio for deal-settlement workflows.
315. Substrate consumer: Healthcare packs use Studio for patient-flow workflows.
316. Substrate consumer: Supply-chain packs use Studio for supplier workflows.
317. Substrate consumer: Delivery packs use Studio for customer recovery workflows.
318. Substrate consumer: Foundry internal uses Studio projections for eval workflows.
319. Substrate consumer: Control Center uses Studio replay summaries for admin diagnostics.
320. Substrate consumer: Observability uses Studio telemetry for SLO posture.
321. Every consumer gets the same canonical workflow spec shape.
322. Every consumer gets domain-specific node libraries.
323. Every consumer gets tenant-scoped policy preview.
324. Every consumer gets audit-chain evidence for saves.
325. Every consumer gets deterministic round-trip validation.
326. Every consumer gets no-silent-loss collaboration semantics.
327. Every consumer gets offline draft buffering.
328. Every consumer gets replay timeline projection.
329. Every consumer gets migration import surface when applicable.
330. Every consumer gets SDK access only through tenant-scoped credentials.

## §4 End-User Visual Editor
331. Screen: Workspace Home.
332. Workspace Home lists definitions by domain, owner, status, and recent run health.
333. Workspace Home has quick actions for New Workflow, Import, Templates, and Recent Failures.
334. Workspace Home shows tenant, pack, environment, and jurisdiction in the header.
335. Workspace Home refuses to list cross-tenant definitions.
336. Workspace Home shows stale telemetry warning if run health is old.
337. Workspace Home supports keyboard search across definitions.
338. Workspace Home supports role-filtered template recommendations.
339. Workspace Home supports empty-state creation from domain examples.
340. Workspace Home supports audit export for selected definitions.
341. Screen: Canvas Editor.
342. Canvas Editor is the primary n8n-class node graph.
343. Canvas Editor supports drag node from palette.
344. Canvas Editor supports command-palette node insertion.
345. Canvas Editor supports keyboard node traversal.
346. Canvas Editor supports keyboard edge creation.
347. Canvas Editor supports marquee selection.
348. Canvas Editor supports zoom to fit.
349. Canvas Editor supports minimap.
350. Canvas Editor supports sticky notes as comments, not runtime steps.
351. Canvas Editor supports collapsible groups.
352. Canvas Editor supports sub-workflow references.
353. Canvas Editor supports invalid-edge highlighting.
354. Canvas Editor supports policy-denied-node highlighting.
355. Canvas Editor supports collaboration-conflict highlighting.
356. Canvas Editor supports unsaved-offline-buffer banner.
357. Canvas Editor supports data-class chips on nodes.
358. Canvas Editor supports autonomy-tier chips on agentic nodes.
359. Canvas Editor supports jurisdiction overlay tint.
360. Canvas Editor supports execution-status overlay.
361. Canvas Editor supports replay scrubber overlay.
362. Canvas Editor supports diff overlay from previous version.
363. Canvas Editor supports dirty-node re-execution markers.
364. Canvas Editor supports line-of-business domain group labels.
365. Canvas Editor does not use canvas-only accessible semantics.
366. Canvas Editor exposes structured outline mirror for screen readers.
367. Canvas Editor requires every graph operation to be keyboard reachable.
368. Canvas Editor validates node handles before edge creation.
369. Canvas Editor validates graph cycles against workflow spec rules.
370. Canvas Editor validates missing required node config.
371. Screen: Node Library Palette.
372. Node Library Palette groups nodes by Trigger, Action, Transform, Branch, Join, Human, AI, and External.
373. Node Library Palette filters by Agentic domain.
374. Node Library Palette filters by Dev domain.
375. Node Library Palette filters by Business domain.
376. Node Library Palette filters by Healthcare domain.
377. Node Library Palette filters by Supply-chain domain.
378. Node Library Palette filters by Delivery domain.
379. Node Library Palette shows signed library status.
380. Node Library Palette shows revocation freshness.
381. Node Library Palette shows connector count.
382. Node Library Palette shows data classes touched by each node.
383. Node Library Palette shows required entitlements.
384. Node Library Palette shows destination microservice.
385. Node Library Palette shows template usage count.
386. Node Library Palette shows migration source equivalence.
387. Node Library Palette prevents unavailable nodes from being dragged.
388. Node Library Palette explains policy-denied nodes without exposing forbidden data.
389. Node Library Palette supports favorite nodes per tenant role.
390. Node Library Palette supports recently used nodes per user.
391. Screen: Node Config Panel.
392. Node Config Panel is the inspector for selected node.
393. Node Config Panel shows typed inputs.
394. Node Config Panel shows typed outputs.
395. Node Config Panel shows schema validation.
396. Node Config Panel shows data-class markers.
397. Node Config Panel shows Cedar preview status.
398. Node Config Panel shows jurisdiction overrides.
399. Node Config Panel shows secret references as write-only fields.
400. Node Config Panel shows autonomy-tier controls for AI nodes.
401. Node Config Panel shows idempotency key rules for side-effecting nodes.
402. Node Config Panel shows retry policy.
403. Node Config Panel shows checkpoint policy.
404. Node Config Panel shows compensation policy.
405. Node Config Panel shows execution boundary.
406. Node Config Panel shows destination microservice contract.
407. Node Config Panel shows test mode toggle when permitted.
408. Node Config Panel shows live mode lock when policy denies.
409. Node Config Panel shows change impact summary.
410. Node Config Panel anchors error summary at top.
411. Modal: Cedar Policy Preview.
412. Cedar Policy Preview opens before production save.
413. Cedar Policy Preview lists allowed actions.
414. Cedar Policy Preview lists denied actions.
415. Cedar Policy Preview lists reason codes.
416. Cedar Policy Preview lists affected data classes.
417. Cedar Policy Preview lists destination microservices.
418. Cedar Policy Preview lists required approvers.
419. Cedar Policy Preview lists environment.
420. Cedar Policy Preview lists tenant pack.
421. Cedar Policy Preview lists audit-chain event shape.
422. Cedar Policy Preview requires acknowledgment for production saves.
423. Cedar Policy Preview blocks save when default deny applies.
424. Cedar Policy Preview blocks save when SECRET value would be exposed.
425. Cedar Policy Preview blocks save when node-config violates domain policy.
426. Cedar Policy Preview can export evidence.
427. Cedar Policy Preview can open policy explanation.
428. Cedar Policy Preview can request tenant-admin grant.
429. Cedar Policy Preview never shows raw policy internals beyond safe reason codes.
430. Cedar Policy Preview never converts deny into warning.
431. Modal: AI Draft Review.
432. AI Draft Review shows tenant prompt summary.
433. AI Draft Review shows prompt-fence status.
434. AI Draft Review shows retrieved template citations.
435. AI Draft Review shows generated nodes.
436. AI Draft Review shows generated edges.
437. AI Draft Review shows policy preview.
438. AI Draft Review shows schema validation diagnostics.
439. AI Draft Review shows forbidden-output diagnostics.
440. AI Draft Review shows human acceptance control.
441. AI Draft Review supports accept all.
442. AI Draft Review supports accept selected fragment.
443. AI Draft Review supports reject with reason.
444. AI Draft Review supports regenerate within same fence.
445. AI Draft Review supports open as draft only.
446. AI Draft Review never auto-saves T1 output.
447. AI Draft Review never auto-commits T2-cross output.
448. AI Draft Review records prompt hash.
449. AI Draft Review records completion hash.
450. AI Draft Review records model route.
451. Modal: Template Detail.
452. Template Detail shows domain, persona, vertical, compliance flags, and SLO.
453. Template Detail shows required node packs.
454. Template Detail shows destination microservices.
455. Template Detail shows estimated execution cost.
456. Template Detail shows test-mode availability.
457. Template Detail shows live-mode availability.
458. Template Detail shows marketplace DealSet terms when template is commercial.
459. Template Detail shows tenant catalog availability.
460. Template Detail shows migration source match.
461. Template Detail supports install to tenant catalog.
462. Template Detail supports fork to private tenant template.
463. Template Detail supports preview as read-only canvas.
464. Template Detail supports policy preview before install.
465. Template Detail supports evidence export.
466. Template Detail refuses install when entitlement missing.
467. Template Detail refuses install when signing invalid.
468. Template Detail refuses install when template pack mismatches tenant jurisdiction.
469. Template Detail never grants settlement rights itself.
470. Template Detail routes settlement through marketplace DealSet.
471. Screen: DSL Sidecar.
472. DSL Sidecar renders canonical JSON.
473. DSL Sidecar supports read-only projection mode.
474. DSL Sidecar supports developer edit mode when role permits.
475. DSL Sidecar highlights JSON pointer for selected node.
476. DSL Sidecar highlights schema error lines.
477. DSL Sidecar highlights policy impacted fields.
478. DSL Sidecar supports diff against previous version.
479. DSL Sidecar supports copy JSON pointer.
480. DSL Sidecar supports format canonical.
481. DSL Sidecar supports load from git-backed definition.
482. DSL Sidecar refuses non-canonical save.
483. DSL Sidecar refuses byte-unstable round-trip.
484. DSL Sidecar refuses edits that desync visual projection.
485. DSL Sidecar separates DSL text from Yjs authoritative draft.
486. DSL Sidecar can show generated-by-AI spans.
487. DSL Sidecar can show template-origin spans.
488. DSL Sidecar can show migration-origin spans.
489. DSL Sidecar can show version intent.
490. DSL Sidecar can show spec fingerprint.
491. Screen: Replay Debugger.
492. Replay Debugger renders live and historical run frames.
493. Replay Debugger shows step status.
494. Replay Debugger shows elapsed time.
495. Replay Debugger shows audit row id.
496. Replay Debugger shows retry eligibility.
497. Replay Debugger shows rollback eligibility.
498. Replay Debugger shows compensation eligibility.
499. Replay Debugger shows stale telemetry warnings.
500. Replay Debugger shows redacted payloads by default.
501. Replay Debugger links frame to canvas node.
502. Replay Debugger links frame to DSL JSON pointer.
503. Replay Debugger supports scrub.
504. Replay Debugger supports step forward.
505. Replay Debugger supports step backward on history.
506. Replay Debugger supports filter failed steps.
507. Replay Debugger supports filter policy-denied steps.
508. Replay Debugger supports export audit bundle.
509. Replay Debugger disables retry until Cedar and idempotency pass.
510. Replay Debugger never mutates workflow-engine state directly.
511. Modal: Migration Import Wizard.
512. Migration Import Wizard accepts n8n workflow export.
513. Migration Import Wizard accepts Zapier Enterprise Zap export.
514. Migration Import Wizard accepts Make.com scenario export.
515. Migration Import Wizard accepts Workato recipe export.
516. Migration Import Wizard accepts Power Automate flow package.
517. Migration Import Wizard maps triggers.
518. Migration Import Wizard maps actions.
519. Migration Import Wizard maps credentials to SecretRef placeholders.
520. Migration Import Wizard maps variables to ontology fields.
521. Migration Import Wizard maps schedules to scheduler nodes.
522. Migration Import Wizard maps error handling to compensation policy.
523. Migration Import Wizard maps approvals to human-review nodes.
524. Migration Import Wizard maps webhooks to signed trigger nodes.
525. Migration Import Wizard maps templates to marketplace candidates.
526. Migration Import Wizard produces a draft, not a published workflow.
527. Migration Import Wizard requires policy preview before save.
528. Migration Import Wizard requires test run before live enablement.
529. Migration Import Wizard writes migration diagnostics.
530. Migration Import Wizard writes lineage metadata.

## §5 Per-Tenant Customization
531. Tenant customization starts at identity.
532. Every editor session is scoped by tenant id.
533. Every draft is scoped by tenant id.
534. Every CRDT update is scoped by tenant id.
535. Every node-library load is scoped by pack entitlement.
536. Every LLM-assist invocation is scoped by tenant consent.
537. Every replay debugger read is scoped by tenant and entitlement.
538. Every policy preview is scoped by tenant and environment.
539. Every template install is scoped by tenant catalog.
540. Every marketplace purchase is scoped by DealSet tenant roles.
541. Node-config Cedar envelope field: `principal`.
542. Node-config Cedar envelope field: `action`.
543. Node-config Cedar envelope field: `resource`.
544. Node-config Cedar envelope field: `context`.
545. Node-config resource field: `tenant_id`.
546. Node-config resource field: `definition_id`.
547. Node-config resource field: `node_id`.
548. Node-config resource field: `node_type`.
549. Node-config resource field: `destination_microservice`.
550. Node-config resource field: `data_class`.
551. Node-config resource field: `environment`.
552. Node-config resource field: `jurisdiction`.
553. Node-config resource field: `pack`.
554. Node-config resource field: `autonomy_tier`.
555. Node-config resource field: `secret_ref_present`.
556. Node-config resource field: `policy_preview_acknowledged`.
557. Node-config resource field: `template_origin`.
558. Node-config resource field: `migration_origin`.
559. Node-config resource field: `deal_set_ref`.
560. Node-config resource field: `reviewer_required`.
561. Node-config principal field: `tenant_id`.
562. Node-config principal field: `role`.
563. Node-config principal field: `packs`.
564. Node-config principal field: `has_studio_seat_entitlement`.
565. Node-config principal field: `llm_assist_consent`.
566. Node-config principal field: `debugger_entitlement`.
567. Node-config principal field: `subscribed_definitions`.
568. Node-config principal field: `allowed_destinations`.
569. Node-config principal field: `approval_authority`.
570. Node-config principal field: `data_access_clearance`.
571. Cedar default denies all actions unless explicit permit matches.
572. Cedar permits own-tenant reads.
573. Cedar permits editor session open only with seat entitlement.
574. Cedar permits save only inside same tenant.
575. Cedar permits CRDT publish only inside same tenant.
576. Cedar permits CRDT subscribe only for subscribed definitions.
577. Cedar permits LLM-assist only with consent and seat entitlement.
578. Cedar permits debugger only with debugger entitlement.
579. Cedar permits node library only when pack is allowed.
580. Cedar forbids cross-tenant reads.
581. Cedar forbids cross-tenant writes.
582. Cedar forbids SECRET-class exposure.
583. Cedar forbids Studio execution actions.
584. Cedar forbids production save without policy-preview acknowledgment.
585. Cedar preview is mandatory when data class changes.
586. Cedar preview is mandatory when destination microservice changes.
587. Cedar preview is mandatory when autonomy tier changes.
588. Cedar preview is mandatory when template origin changes.
589. Cedar preview is mandatory when migration import creates credentials.
590. Cedar preview is mandatory when DealSet terms are bound.
591. Tenant customization includes catalog curation.
592. Tenant catalog can hide public templates.
593. Tenant catalog can pin approved templates.
594. Tenant catalog can fork templates privately.
595. Tenant catalog can attach jurisdiction overlays.
596. Tenant catalog can attach internal help text.
597. Tenant catalog can attach approval matrices.
598. Tenant catalog can attach cost centers.
599. Tenant catalog can attach data-class defaults.
600. Tenant catalog can attach package-specific node packs.
601. Tenant catalog cannot edit canonical template provenance.
602. Tenant catalog cannot bypass marketplace settlement.
603. Tenant catalog cannot suppress audit-chain sealing.
604. Tenant catalog cannot suppress Cedar preview.
605. Tenant catalog cannot hide revoked node library status.
606. Tenant catalog cannot install unsigned node packs.
607. Tenant catalog cannot install cross-pack templates without entitlement.
608. Tenant catalog cannot convert template terms into unscoped rights.
609. Tenant catalog cannot expose SECRET values in preview.
610. Tenant catalog cannot publish to public marketplace without signing flow.
611. Per-tenant customization includes design-system density.
612. Per-tenant customization includes locale.
613. Per-tenant customization includes jurisdiction default.
614. Per-tenant customization includes domain default.
615. Per-tenant customization includes template allowlist.
616. Per-tenant customization includes node pack allowlist.
617. Per-tenant customization includes AI enablement.
618. Per-tenant customization includes migration tooling enablement.
619. Per-tenant customization includes replay retention.
620. Per-tenant customization includes audit export format.
621. Per-tenant customization excludes arbitrary scripting in editor chrome.
622. Per-tenant customization excludes arbitrary iframe embedding.
623. Per-tenant customization excludes unreviewed plugin code.
624. Per-tenant customization excludes visual-only policy overrides.
625. Per-tenant customization excludes runtime execution rules owned by engine.
626. The customization contract is per tenant, per pack, per environment.
627. The customization contract is enforceable by Cedar.
628. The customization contract is visible before save.
629. The customization contract is logged on deny.
630. The customization contract is auditable after save.

## §6 AI-Assisted Generation
631. AI-assisted generation is a Studio feature backed by Intelligence.
632. Intelligence owns tenant-facing AI context.
633. Foundry remains internal.
634. AI calls record tenant, user, context, data class, model route, prompt id, cost, and audit reference.
635. Studio uses RAG over the workflow library.
636. RAG source: tenant private workflow history.
637. RAG source: approved tenant template catalog.
638. RAG source: public template marketplace metadata.
639. RAG source: node-library descriptors.
640. RAG source: ontology type descriptors.
641. RAG source: migration mapping examples.
642. RAG source: policy-safe configuration examples.
643. RAG excludes raw secrets.
644. RAG excludes unauthorized tenant data.
645. RAG excludes unapproved public templates.
646. RAG excludes Foundry internal eval corpora for consumer flows.
647. RAG excludes prompt history after DSAR deletion.
648. RAG excludes cross-tenant examples unless marketplace terms permit aggregate use.
649. RAG retrieval is tenant scoped.
650. RAG retrieval is jurisdiction scoped when data class requires.
651. Prompt fence declares allowed destination microservices.
652. Prompt fence declares forbidden destination microservices.
653. Prompt fence declares allowed node types.
654. Prompt fence declares forbidden node types.
655. Prompt fence declares data-class ceiling.
656. Prompt fence declares autonomy-tier ceiling.
657. Prompt fence declares environment.
658. Prompt fence declares tenant pack.
659. Prompt fence declares schema version.
660. Prompt fence declares output format.
661. Prompt fence declares no-secret rule.
662. Prompt fence declares no-Cedar-bypass rule.
663. Prompt fence declares no-runtime-execution rule.
664. Prompt fence declares human-review rule.
665. Prompt fence declares citation requirement.
666. Prompt fence declares rejection behavior.
667. T0 Suggest proposes next node.
668. T0 Suggest proposes parameter value.
669. T0 Suggest proposes validation remediation.
670. T0 Suggest proposes policy-safe alternative.
671. T0 Suggest never writes graph without acceptance.
672. T0 Suggest never emits cross-microservice nodes.
673. T0 Suggest requires entitlement only.
674. T0 Suggest labels output as AI-generated.
675. T0 Suggest records suggestion acceptance metrics.
676. T0 Suggest remains reversible as ordinary edit.
677. T1 Assist drafts candidate fragment.
678. T1 Assist is opt-in per tenant.
679. T1 Assist is default off.
680. T1 Assist uses tenant prose.
681. T1 Assist returns canonical candidate spec.
682. T1 Assist requires user review.
683. T1 Assist forbids cross-microservice calls by default.
684. T1 Assist may read ontology descriptors.
685. T1 Assist rejects invalid schema.
686. T1 Assist rejects policy-bypass attempts.
687. T2 Auto covers CRDT auto-merge.
688. T2 Auto covers editor session persistence.
689. T2 Auto covers retry orchestration on transient editor failures.
690. T2-cross is gated separately.
691. T2-cross requires per-destination enablement.
692. T2-cross requires Cedar permit.
693. T2-cross requires ChangeSet review.
694. T2-cross requires separate author and reviewer.
695. T2-cross requires destination SDK contract.
696. T2-cross treats high-risk AI classification as default.
697. AI output flows into DSL loader.
698. DSL loader canonicalizes AI output.
699. DSL loader rejects forbidden destination nodes.
700. DSL loader rejects missing data classification.
701. DSL loader rejects unresolvable ontology types.
702. DSL loader rejects missing idempotency for side effects.
703. DSL loader rejects raw secrets.
704. DSL loader rejects unbounded loops.
705. DSL loader rejects unpinned external connector versions.
706. DSL loader emits precise diagnostics.
707. AI Draft Review renders only after schema validation.
708. AI Draft Review renders only after prompt-fence check.
709. AI Draft Review renders only after RAG provenance capture.
710. AI Draft Review renders only after policy preview.
711. AI archive stores prompt hash.
712. AI archive stores completion hash.
713. AI archive stores retrieved document ids.
714. AI archive stores model route.
715. AI archive stores token cost.
716. AI archive stores data-class ceiling.
717. AI archive stores tenant consent.
718. AI archive stores acceptance verdict.
719. AI archive stores reviewer identity for T2-cross.
720. AI archive retention defaults to 90 days where policy allows.
721. AI safety metric: forbidden-output-blocked count.
722. AI safety metric: T1 acceptance rate.
723. AI safety metric: T2-cross revert rate.
724. AI safety metric: cross-pack routing count.
725. AI safety metric: high-risk invocation count.
726. AI safety metric: prompt-injection scrub count.
727. AI fallback: disable LLM assist per tenant after timeout cascade.
728. AI fallback: allow manual authoring while degraded.
729. AI fallback: keep templates and node palette available.
730. AI fallback: show degradation banner without blocking editor.

## §7 Custom Node SDK
731. Custom node SDK enables tenant and partner node packs.
732. SDK does not grant publishing rights by itself.
733. SDK output must pass signing and marketplace gates.
734. SDK output must pass node-library registry validation.
735. SDK output must pass Cedar policy preview.
736. SDK output must pass data-class validation.
737. SDK output must pass deterministic replay classification.
738. SDK output must pass documentation lint.
739. SDK output must pass fixture tests.
740. SDK output must pass migration mapping when replacing vendor nodes.
741. Lifecycle hook: `describe`.
742. Lifecycle hook: `configure`.
743. Lifecycle hook: `validate_config`.
744. Lifecycle hook: `preview_policy`.
745. Lifecycle hook: `infer_schema`.
746. Lifecycle hook: `plan_execution`.
747. Lifecycle hook: `prepare_input`.
748. Lifecycle hook: `execute`.
749. Lifecycle hook: `handle_signal`.
750. Lifecycle hook: `retry`.
751. Lifecycle hook: `compensate`.
752. Lifecycle hook: `serialize_state`.
753. Lifecycle hook: `redact_for_replay`.
754. Lifecycle hook: `emit_audit`.
755. Lifecycle hook: `estimate_cost`.
756. Lifecycle hook: `dispose`.
757. TypeScript SDK package: `@oyatie/workflow-studio-node-sdk`.
758. TypeScript SDK target: tenant web and integration developers.
759. TypeScript SDK runtime boundary: authoring and descriptor generation.
760. TypeScript SDK does not execute privileged workflow steps in browser.
761. TypeScript SDK produces `NodeDescriptor`.
762. TypeScript SDK produces `ConfigSchema`.
763. TypeScript SDK produces `PolicyPreviewHints`.
764. TypeScript SDK produces test fixtures.
765. TypeScript SDK supports Zod-like schema bridge only if generated from canonical schema.
766. TypeScript SDK supports editor preview components through sandboxed manifest.
767. TypeScript SDK supports npm publishing after approval.
768. TypeScript SDK supports typed SecretRef fields.
769. TypeScript SDK supports data-class annotation helpers.
770. TypeScript SDK supports template snippet helpers.
771. TypeScript SDK hook signature: `describe(): NodeDescriptor`.
772. TypeScript SDK hook signature: `configure(ctx): ConfigPanelSpec`.
773. TypeScript SDK hook signature: `validateConfig(config, ctx): Diagnostic[]`.
774. TypeScript SDK hook signature: `previewPolicy(config, ctx): PolicyPreviewRequest`.
775. TypeScript SDK hook signature: `planExecution(config, ctx): ExecutionPlan`.
776. TypeScript SDK hook signature: `redactForReplay(frame, ctx): RedactedFrame`.
777. TypeScript SDK hook signature: `estimateCost(config, ctx): CostEstimate`.
778. TypeScript SDK forbids raw network execution from editor preview.
779. TypeScript SDK forbids dynamic eval.
780. TypeScript SDK forbids untyped credential fields.
781. Rust SDK crate: `oya-workflow-studio-node-sdk`.
782. Rust SDK target: first-party and high-assurance partner nodes.
783. Rust SDK runtime boundary: descriptor, validation, and engine adapter contracts.
784. Rust SDK uses `#![deny(unsafe_code)]`.
785. Rust SDK exposes tenant-token provider trait.
786. Rust SDK exposes Cedar preview request builder.
787. Rust SDK exposes deterministic replay safety enum.
788. Rust SDK exposes idempotency key helper.
789. Rust SDK exposes compensation trait.
790. Rust SDK exposes audit emission trait.
791. Rust SDK exposes OpenTelemetry span helpers.
792. Rust SDK exposes proto bindings.
793. Rust SDK exposes WASM-compatible descriptor generation.
794. Rust SDK exposes property test harness.
795. Rust SDK exposes reference fixture loader.
796. Rust SDK hook trait: `NodeDescribe`.
797. Rust SDK hook trait: `NodeConfig`.
798. Rust SDK hook trait: `NodeValidateConfig`.
799. Rust SDK hook trait: `NodePreviewPolicy`.
800. Rust SDK hook trait: `NodePlanExecution`.
801. Rust SDK hook trait: `NodeExecute`.
802. Rust SDK hook trait: `NodeCompensate`.
803. Rust SDK hook trait: `NodeRedactForReplay`.
804. Rust SDK hook trait: `NodeEmitAudit`.
805. Rust SDK hook trait: `NodeEstimateCost`.
806. Rust SDK forbids non-deterministic plan output.
807. Rust SDK forbids side-effecting validation.
808. Rust SDK forbids plaintext secret serialization.
809. Rust SDK forbids missing idempotency for external calls.
810. Rust SDK forbids hidden cross-tenant cache keys.
811. Python SDK package: `oyatie-workflow-studio-node-sdk`.
812. Python SDK target: LLM-orchestrator tenants and data-operation teams.
813. Python SDK runtime boundary: descriptor generation and test harness.
814. Python SDK can author node descriptors.
815. Python SDK can run local fixture tests.
816. Python SDK can request policy preview.
817. Python SDK can submit draft specs.
818. Python SDK cannot publish node packs.
819. Python SDK cannot bypass two-person signing.
820. Python SDK cannot execute engine steps from editor context.
821. Python SDK supports Pydantic model generation from canonical schema.
822. Python SDK supports pytest fixture runner.
823. Python SDK supports SecretRef placeholder validation.
824. Python SDK supports data-class annotation.
825. Python SDK supports migration adapter scaffolds.
826. Python SDK hook function: `describe()`.
827. Python SDK hook function: `configure(context)`.
828. Python SDK hook function: `validate_config(config, context)`.
829. Python SDK hook function: `preview_policy(config, context)`.
830. Python SDK hook function: `plan_execution(config, context)`.
831. Python SDK hook function: `redact_for_replay(frame, context)`.
832. Python SDK hook function: `estimate_cost(config, context)`.
833. Python SDK hook function: `emit_audit(event, context)`.
834. Python SDK forbids raw credential strings.
835. Python SDK forbids dynamic package installation at validation time.
836. Python SDK forbids network calls during descriptor generation.
837. Node pack manifest field: `node_pack_id`.
838. Node pack manifest field: `version`.
839. Node pack manifest field: `publisher_tenant_id`.
840. Node pack manifest field: `domain_tags`.
841. Node pack manifest field: `destination_microservices`.
842. Node pack manifest field: `data_classes`.
843. Node pack manifest field: `required_entitlements`.
844. Node pack manifest field: `cedar_policy_ref`.
845. Node pack manifest field: `template_refs`.
846. Node pack manifest field: `migration_refs`.
847. Node pack manifest field: `signature`.
848. Node pack manifest field: `sbom_ref`.
849. Node pack manifest field: `fixture_ref`.
850. Node pack manifest field: `compatibility_matrix`.
851. Node pack publish step: build descriptors.
852. Node pack publish step: run unit tests.
853. Node pack publish step: run fixture tests.
854. Node pack publish step: run policy preview tests.
855. Node pack publish step: run replay determinism tests.
856. Node pack publish step: generate SBOM.
857. Node pack publish step: sign artifacts.
858. Node pack publish step: submit marketplace DealSet if commercial.
859. Node pack publish step: request reviewer approval.
860. Node pack publish step: register in node-library registry.
861. Node pack revoke step: mark signing key revoked.
862. Node pack revoke step: propagate CRL within 60 seconds.
863. Node pack revoke step: disable new installs.
864. Node pack revoke step: warn active editors.
865. Node pack revoke step: preserve replay history.
866. Node pack revoke step: open migration recommendation.
867. Node pack compatibility target: current Studio major.
868. Node pack compatibility target: prior Studio major.
869. Node pack compatibility target: current workflow spec version.
870. Node pack compatibility target: current ontology descriptor version.

## §8 Migration Playbooks Index
871. Migration playbooks convert competitor artifacts into Studio drafts.
872. Migration never publishes directly.
873. Migration never imports secrets as plaintext.
874. Migration never skips policy preview.
875. Migration never claims full parity without diagnostics.
876. Migration writes lineage metadata.
877. Migration writes unsupported-feature diagnostics.
878. Migration writes equivalent-node diagnostics.
879. Migration writes test-plan recommendations.
880. Migration writes template-catalog recommendation.

### §8.1 n8n Migration
881. Source vendor: n8n.
882. Input artifact: workflow export JSON.
883. Input artifact: credential references.
884. Input artifact: environment variables.
885. Input artifact: execution mode settings.
886. Input artifact: queue mode assumptions.
887. Mapping: n8n trigger becomes Studio trigger node.
888. Mapping: n8n action becomes capability-call or connector node.
889. Mapping: n8n IF becomes branch node.
890. Mapping: n8n Merge becomes join node.
891. Mapping: n8n Function becomes transform node with sandbox warning.
892. Mapping: n8n sticky note becomes comment.
893. Mapping: n8n credentials become SecretRef placeholders.
894. Mapping: n8n workflow settings become definition metadata.
895. Mapping: n8n execution history imports as replay archive only when permitted.
896. Gap: custom code nodes need review.
897. Gap: community nodes need node-pack validation.
898. Gap: credential sharing needs tenant-role mapping.
899. Gap: binary payload handling needs artifact_ref.
900. Gap: partial execution semantics need test-mode mapping.
901. Validation: round-trip emitted spec.
902. Validation: Cedar preview all nodes.
903. Validation: policy-denied nodes surfaced.
904. Validation: test run required before live.
905. Validation: compare source and target path counts.
906. Output: Studio draft.
907. Output: migration report.
908. Output: unsupported feature list.
909. Output: template recommendation.
910. Output: rollback plan.

### §8.2 Zapier Enterprise Migration
911. Source vendor: Zapier Enterprise.
912. Input artifact: Zap export or enterprise transfer package.
913. Input artifact: app connections.
914. Input artifact: trigger-action sequence.
915. Input artifact: paths.
916. Input artifact: filters.
917. Mapping: Zap trigger becomes Studio trigger node.
918. Mapping: Zap action becomes connector node.
919. Mapping: Zap path becomes branch node.
920. Mapping: Zap filter becomes predicate node.
921. Mapping: Zap Formatter becomes transform node.
922. Mapping: Zap Delay becomes scheduler node.
923. Mapping: Zap approval-like flow becomes human-review node.
924. Mapping: Zap AI-generated step becomes AI-origin annotation.
925. Mapping: Zap connections become SecretRef placeholders.
926. Gap: Zap app breadth may exceed current node packs.
927. Gap: proprietary app features need connector backlog.
928. Gap: task quota semantics need finops mapping.
929. Gap: transfer ownership needs tenant role mapping.
930. Gap: simple UI assumptions need policy education.
931. Validation: one node per Zap step.
932. Validation: field mapping diagnostics.
933. Validation: secret mapping diagnostics.
934. Validation: Cedar preview.
935. Validation: test-mode run.
936. Output: Studio draft.
937. Output: connector backlog.
938. Output: app equivalence table.
939. Output: policy preview packet.
940. Output: user acceptance checklist.

### §8.3 Make.com Migration
941. Source vendor: Make.com.
942. Input artifact: scenario blueprint.
943. Input artifact: connection references.
944. Input artifact: routers.
945. Input artifact: error handlers.
946. Input artifact: incomplete execution settings.
947. Mapping: Make module becomes Studio node.
948. Mapping: Make router becomes branch node.
949. Mapping: Make aggregator becomes join node.
950. Mapping: Make iterator becomes loop-safe transform.
951. Mapping: Make error handler becomes compensation policy.
952. Mapping: Make schedule becomes scheduler trigger.
953. Mapping: Make datastore becomes ontology or artifact_ref decision.
954. Mapping: Make webhook becomes signed trigger node.
955. Mapping: Make incomplete execution becomes resume eligibility.
956. Gap: auto-commit semantics need explicit Studio save.
957. Gap: rollback semantics need saga compensation.
958. Gap: connection scopes need Cedar mapping.
959. Gap: bundle payloads need max_payload_bytes.
960. Gap: visual layout may need graph normalization.
961. Validation: router coverage.
962. Validation: error handler coverage.
963. Validation: compensation mapping.
964. Validation: incomplete execution policy.
965. Validation: Cedar preview.
966. Output: Studio draft.
967. Output: compensation report.
968. Output: payload-size report.
969. Output: connector equivalence report.
970. Output: live-mode risk assessment.

### §8.4 Workato Migration
971. Source vendor: Workato.
972. Input artifact: recipe export.
973. Input artifact: connector list.
974. Input artifact: recipe functions.
975. Input artifact: lookup tables.
976. Input artifact: recipe copilot annotations.
977. Mapping: Workato trigger becomes Studio trigger node.
978. Mapping: Workato action becomes connector node.
979. Mapping: Workato conditional action becomes branch node.
980. Mapping: Workato repeat action becomes bounded iterator.
981. Mapping: Workato recipe function becomes reusable sub-workflow.
982. Mapping: Workato lookup table becomes ontology or artifact decision.
983. Mapping: Workato connector auth becomes SecretRef.
984. Mapping: Workato recipe copilot output becomes AI-origin annotation.
985. Mapping: Workato job history imports as replay archive when permitted.
986. Gap: connector catalog breadth needs backlog.
987. Gap: recipe function semantics need sub-workflow validation.
988. Gap: lookup table ownership needs ontology modeling.
989. Gap: enterprise governance needs tenant role mapping.
990. Gap: on-prem agent path needs connect adapter mapping.
991. Validation: recipe function equivalence.
992. Validation: connector auth scope.
993. Validation: data-class propagation.
994. Validation: Cedar preview.
995. Validation: test run.
996. Output: Studio draft.
997. Output: connector backlog.
998. Output: sub-workflow candidates.
999. Output: data-class report.
1000. Output: governance mapping.

### §8.5 Power Automate Migration
1001. Source vendor: Microsoft Power Automate.
1002. Input artifact: solution package.
1003. Input artifact: cloud flow definitions.
1004. Input artifact: approvals.
1005. Input artifact: Dataverse references.
1006. Input artifact: DLP policies.
1007. Mapping: Power Automate trigger becomes Studio trigger node.
1008. Mapping: Power Automate action becomes connector node.
1009. Mapping: Condition becomes branch node.
1010. Mapping: Scope becomes group and compensation boundary.
1011. Mapping: Approval becomes human-review node.
1012. Mapping: Dataverse table becomes ontology descriptor.
1013. Mapping: Environment becomes tenant pack and environment selector.
1014. Mapping: DLP policy becomes Cedar preview input.
1015. Mapping: Copilot-generated flow becomes AI-origin annotation.
1016. Gap: Microsoft-only connectors need connect adapters.
1017. Gap: Dataverse-specific semantics need ontology migration.
1018. Gap: DLP policies need Cedar translation.
1019. Gap: desktop flows are out of browser scope.
1020. Gap: solution ALM needs git-backed definition mapping.
1021. Validation: approval path equivalence.
1022. Validation: DLP-to-Cedar mapping.
1023. Validation: environment mapping.
1024. Validation: Dataverse ontology mapping.
1025. Validation: test run.
1026. Output: Studio draft.
1027. Output: Cedar translation report.
1028. Output: ontology migration report.
1029. Output: connector backlog.
1030. Output: ALM transition plan.

## §9 Workflow Execution Semantics
1031. Studio authors; workflow-engine executes.
1032. Studio can request validation.
1033. Studio can request deployment.
1034. Studio can request test-mode run.
1035. Studio can request replay stream.
1036. Studio cannot execute workflow steps.
1037. Studio cannot dispatch side effects.
1038. Studio cannot hold runtime leases.
1039. Studio cannot mutate engine event history.
1040. Studio cannot rewrite engine checkpoints.
1041. Durable function semantics belong to workflow-engine.
1042. Durable function source of truth is event history.
1043. Durable function replay must be deterministic.
1044. Durable function activities hold side effects.
1045. Durable function activities require idempotency key.
1046. Durable function activities require retry policy.
1047. Durable function activities require checkpoint policy.
1048. Durable function activities require replay safety class.
1049. Durable function activities require side-effect boundary.
1050. Durable function activities expose telemetry to replay timeline.
1051. Saga compensation semantics apply to irreversible external effects.
1052. Saga compensation requires compensation node or policy.
1053. Saga compensation requires audit evidence.
1054. Saga compensation requires rollback eligibility state.
1055. Saga compensation cannot delete historical evidence.
1056. Saga compensation cannot hide failed side effects.
1057. Saga compensation must be visible in replay timeline.
1058. Saga compensation must be testable before live.
1059. Saga compensation must respect Cedar policy.
1060. Saga compensation must respect destination service ownership.
1061. Deterministic replay requirement: same event history yields same decisions.
1062. Deterministic replay requirement: external calls are activities, not pure steps.
1063. Deterministic replay requirement: time reads are pinned.
1064. Deterministic replay requirement: random values are recorded.
1065. Deterministic replay requirement: model outputs are recorded as activity outputs.
1066. Deterministic replay requirement: branch decisions are recorded.
1067. Deterministic replay requirement: compensation decisions are recorded.
1068. Deterministic replay requirement: versioned workflow definitions are pinned.
1069. Deterministic replay requirement: ontology type versions are pinned.
1070. Deterministic replay requirement: policy decision ids are linked.
1071. Studio shows replay safety class in node config.
1072. Studio blocks save when side-effect node lacks idempotency.
1073. Studio blocks save when compensation is missing for irreversible effects.
1074. Studio warns when deterministic replay depends on external volatile input.
1075. Studio warns when payload exceeds inline max.
1076. Studio recommends artifact_ref for large payloads.
1077. Studio recommends worker queue when throughput requires.
1078. Studio recommends schedule concurrency policy.
1079. Studio recommends mutex or semaphore for shared resources.
1080. Studio recommends human review for high-risk AI actions.
1081. Test-mode execution is an engine run in test environment.
1082. Partial execution is an engine-supported test feature.
1083. Manual execution is an engine-supported test feature.
1084. Production execution is engine-owned.
1085. Backfill execution is engine-owned.
1086. Resume execution is engine-owned.
1087. Retry execution is engine-owned.
1088. Rollback execution is engine-owned.
1089. Studio can present controls for permitted operations.
1090. Studio cannot implement the operation itself.

## §10 Cross-Microservice Calls
1091. Boundary rule: Studio calls only typed APIs and SDKs.
1092. Boundary rule: Studio never calls another service database.
1093. Boundary rule: Studio never bypasses tenant context.
1094. Boundary rule: Studio never bypasses Cedar.
1095. Boundary rule: Studio never bypasses audit-chain for publish decisions.
1096. Handoff: workflow-engine.
1097. Studio sends `workflow_spec.v1.json` to workflow-engine.
1098. Workflow-engine returns validation diagnostics.
1099. Workflow-engine returns deployment ack.
1100. Workflow-engine returns run ids.
1101. Workflow-engine streams replay frames.
1102. Workflow-engine owns execution.
1103. Handoff: ontology.
1104. Studio requests ontology type descriptors.
1105. Studio requests relation descriptors.
1106. Studio requests object-field metadata.
1107. Studio requests deprecation handshakes.
1108. Studio receives versioned descriptor refs.
1109. Ontology owns semantic object model.
1110. Handoff: tenancy.
1111. Studio resolves tenant identity.
1112. Studio resolves seat entitlements.
1113. Studio resolves pack memberships.
1114. Studio resolves role grants.
1115. Studio resolves environment access.
1116. Tenancy owns universal tenant scope.
1117. Handoff: Cedar policy service.
1118. Studio requests policy preview.
1119. Studio requests session-open decision.
1120. Studio requests save decision.
1121. Studio requests node-library load decision.
1122. Studio requests LLM-assist decision.
1123. Cedar owns authorization outcome.
1124. Handoff: Intelligence.
1125. Studio requests tenant-facing AI draft.
1126. Studio sends prompt fence.
1127. Studio sends retrieved context refs.
1128. Studio sends data-class ceiling.
1129. Studio receives candidate spec.
1130. Intelligence owns consumer AI context.
1131. Handoff: Foundry.
1132. Studio uses Foundry for internal eval workflows only.
1133. Studio may send synthetic fixtures.
1134. Studio may receive evaluation score.
1135. Studio may receive quality gate status.
1136. Foundry owns internal eval and CI orchestration.
1137. Handoff: marketplace.
1138. Studio requests template catalog listings.
1139. Studio requests node-pack entitlement status.
1140. Studio requests DealSet settlement for commercial templates.
1141. Studio receives catalog grants.
1142. Studio receives revocation events.
1143. Marketplace owns deal settlement.
1144. Handoff: audit-chain.
1145. Studio emits editor session opened.
1146. Studio emits definition saved.
1147. Studio emits policy decision.
1148. Studio emits LLM draft accepted.
1149. Studio emits migration imported.
1150. Audit-chain owns immutable evidence.
1151. Handoff: observability.
1152. Studio emits editor latency metrics.
1153. Studio emits CRDT merge metrics.
1154. Studio emits no-silent-loss events.
1155. Studio emits LLM quality metrics.
1156. Studio emits SLO burn signals.
1157. Observability owns alerting.
1158. Handoff: connect.
1159. Studio references connect adapter nodes.
1160. Studio receives connector descriptors.
1161. Studio receives auth scope descriptors.
1162. Studio receives migration connector mapping.
1163. Studio never stores connector secrets.
1164. owns external system adapters.
1165. Handoff: messenger.
1166. Studio configures notification nodes.
1167. Studio previews message data class.
1168. Studio previews recipient policy.
1169. Studio receives descriptor metadata.
1170. Messenger owns message delivery.
1171. Handoff: mail.
1172. Studio configures email nodes.
1173. Studio previews recipient and template policy.
1174. Studio requires idempotency key for mail sends.
1175. Studio receives descriptor metadata.
1176. Mail owns email delivery.
1177. Handoff: community.
1178. Studio configures community event nodes.
1179. Studio previews moderation policy.
1180. Studio maps community object descriptors.
1181. Studio receives descriptor metadata.
1182. Community owns social/community execution.
1183. Handoff: payments.
1184. Studio configures payment authorization nodes.
1185. Studio previews PCI and settlement policy.
1186. Studio requires compensation policy.
1187. Studio receives descriptor metadata.
1188. Payments owns payment rails.
1189. Handoff: treasury.
1190. Studio configures treasury nodes.
1191. Studio previews cash and FX authority.
1192. Studio requires audit-chain evidence.
1193. Studio receives descriptor metadata.
1194. Treasury owns treasury execution.
1195. Handoff: finops-portal.
1196. Studio configures cost allocation nodes.
1197. Studio previews cost-center policy.
1198. Studio receives budget descriptors.
1199. Studio receives chargeback metadata.
1200. Finops owns cost records.
1201. Handoff: global-trade.
1202. Studio configures sanctions and customs nodes.
1203. Studio previews trade hold policy.
1204. Studio receives trade decision descriptors.
1205. Studio blocks if trade hold denies.
1206. Global-trade owns trade compliance.
1207. Handoff: warehouse.
1208. Studio configures inventory nodes.
1209. Studio previews stock mutation policy.
1210. Studio requires idempotency key.
1211. Studio receives descriptor metadata.
1212. Warehouse owns inventory execution.
1213. Handoff: delivery.
1214. Studio configures route and delivery nodes.
1215. Studio previews customer contact policy.
1216. Studio requires notification idempotency.
1217. Studio receives descriptor metadata.
1218. Delivery owns route execution.
1219. Handoff: control-center.
1220. Studio provides admin status summaries.
1221. Studio provides SLO posture.
1222. Studio provides template install posture.
1223. Studio provides migration posture.
1224. Control Center owns admin aggregate view.
1225. Handoff: core.
1226. Studio uses identity and audit primitives from core.
1227. Studio uses canonical tenant context from core.
1228. Studio receives principal descriptors.
1229. Studio emits evidence references.
1230. Core owns shared primitives.

## §11 Failure Modes + Recovery
1231. Failure mode: CRDT state divergence.
1232. Detection: state-hash mismatch or merge mismatch metric.
1233. Recovery: force resync from latest accepted state.
1234. User experience: resync required banner.
1235. Product invariant: no silent loss.
1236. Failure mode: stale WASM bundle.
1237. Detection: CDN propagation lag or browser version mismatch.
1238. Recovery: force purge and versioned URL rotation.
1239. User experience: reload prompt.
1240. Product invariant: SRI-enforced load.
1241. Failure mode: LLM-assist timeout cascade.
1242. Detection: p99 timeout threshold or circuit breaker.
1243. Recovery: disable LLM-assist per tenant.
1244. User experience: degraded AI banner.
1245. Product invariant: manual editor remains usable.
1246. Failure mode: jurisdiction overlay drift.
1247. Detection: overlay version mismatch.
1248. Recovery: cache invalidation and reload prompt.
1249. User experience: blocking stale overlay banner.
1250. Product invariant: engine and Studio share versioned overlay ref.
1251. Failure mode: Cedar evaluator crash.
1252. Detection: license-gate error rate.
1253. Recovery: restart evaluator; stay fail-closed.
1254. User experience: editor open refused until recovery.
1255. Product invariant: no failure-open authorization.
1256. Failure mode: WebSocket gateway restart.
1257. Detection: disconnect-rate spike.
1258. Recovery: browser reconnect and lease handoff.
1259. User experience: reconnecting banner with unsent count.
1260. Product invariant: CRDT state persists.
1261. Failure mode: WASM bundle corruption.
1262. Detection: SRI mismatch.
1263. Recovery: purge and republish from build origin.
1264. User experience: editor unavailable banner.
1265. Product invariant: corrupted code never runs.
1266. Failure mode: Postgres hot-session contention.
1267. Detection: lock waits and save latency.
1268. Recovery: tenant pool isolation and shard review.
1269. User experience: save delayed banner.
1270. Product invariant: tenant isolation protects neighbors.
1271. Failure mode: cross-tenant collab leak.
1272. Detection: unauthorized CRDT recipient metric.
1273. Recovery: freeze pod, revoke tokens, start forensic trace.
1274. User experience: security incident communication.
1275. Product invariant: Sev-1 and no continued exposure.
1276. Failure mode: node-library signature failure.
1277. Detection: signature invalid metric.
1278. Recovery: re-sign libraries and propagate CRL.
1279. User experience: node library unavailable banner.
1280. Product invariant: unsigned nodes never load.
1281. Failure mode: XSS injection.
1282. Detection: bug report or CSP violation spike.
1283. Recovery: hot-patch CSP, revoke tokens, reload.
1284. User experience: forced reload and incident notice.
1285. Product invariant: no raw innerHTML rendering.
1286. Failure mode: LLM prompt routed to wrong region.
1287. Detection: cross-pack routing metric.
1288. Recovery: disable LLM-assist and engage privacy incident.
1289. User experience: AI disabled for affected pack.
1290. Product invariant: residency breach is Sev-1.
1291. Failure mode: per-seat overage not detected.
1292. Detection: tenancy reconciliation.
1293. Recovery: Cedar cache invalidation.
1294. User experience: no silent billing drift.
1295. Product invariant: billing reconciles with evidence.
1296. Failure mode: round-trip byte-equality regression.
1297. Detection: workflow-spec-roundtrip gate fails.
1298. Recovery: roll back release and run corpus.
1299. User experience: save blocked or reverted.
1300. Product invariant: visual and canonical spec remain trustworthy.
1301. Failure mode: template marketplace quarantine.
1302. Detection: template signing, policy, or quality regression.
1303. Recovery: quarantine template, revoke install, preserve lineage.
1304. User experience: template unavailable with reason.
1305. Product invariant: marketplace content cannot poison tenants.
1306. Failure mode: migration mapping ambiguity.
1307. Detection: import diagnostics unresolved.
1308. Recovery: require manual mapping.
1309. User experience: wizard stops at mapping screen.
1310. Product invariant: migrated drafts never silently approximate semantics.

## §12 SLO Targets
1311. Tier Preview editor REST availability target: 99.5 percent.
1312. Tier Stable editor REST availability target: 99.9 percent.
1313. Tier GA editor REST availability target: 99.95 percent.
1314. Tier Preview editor save latency p99 target: 300 ms.
1315. Tier Stable editor save latency p99 target: 200 ms.
1316. Tier GA editor save latency p99 target: 100 ms.
1317. Tier Preview editor cold TTI p99 target: 3000 ms.
1318. Tier Stable editor cold TTI p99 target: 2500 ms.
1319. Tier GA editor cold TTI p99 target: 2000 ms.
1320. Tier Preview canvas frame success target: 98 percent.
1321. Tier Stable canvas frame success target: 98.5 percent.
1322. Tier GA canvas frame success target: 99 percent.
1323. Tier Preview CRDT merge latency p99 target: 150 ms.
1324. Tier Stable CRDT merge latency p99 target: 100 ms.
1325. Tier GA CRDT merge latency p99 target: 100 ms.
1326. Tier Preview no-silent-loss target: 100 percent.
1327. Tier Stable no-silent-loss target: 100 percent.
1328. Tier GA no-silent-loss target: 100 percent.
1329. Tier Preview license-gate Cedar availability target: 99.9 percent.
1330. Tier Stable license-gate Cedar availability target: 99.95 percent.
1331. Tier GA license-gate Cedar availability target: 99.99 percent.
1332. Tier Preview WebSocket availability target: 99.5 percent.
1333. Tier Stable WebSocket availability target: 99.9 percent.
1334. Tier GA WebSocket availability target: 99.9 percent.
1335. Tier Preview LLM-assist availability target: 99 percent.
1336. Tier Stable LLM-assist availability target: 99.5 percent.
1337. Tier GA LLM-assist availability target: 99.5 percent.
1338. Tier Preview LLM-assist p99 target: 8000 ms.
1339. Tier Stable LLM-assist p99 target: 5000 ms.
1340. Tier GA LLM-assist p99 target: 3000 ms.
1341. Tier Preview node-library load p99 target: 1000 ms.
1342. Tier Stable node-library load p99 target: 750 ms.
1343. Tier GA node-library load p99 target: 500 ms.
1344. Tier Preview replay frame render p99 target: 300 ms.
1345. Tier Stable replay frame render p99 target: 200 ms.
1346. Tier GA replay frame render p99 target: 100 ms.
1347. Tier Preview cross-tenant leak target: zero.
1348. Tier Stable cross-tenant leak target: zero.
1349. Tier GA cross-tenant leak target: zero.
1350. Tier Preview SECRET exposure target: zero.
1351. Tier Stable SECRET exposure target: zero.
1352. Tier GA SECRET exposure target: zero.
1353. Tier Preview policy preview coverage target: 95 percent.
1354. Tier Stable policy preview coverage target: 99 percent.
1355. Tier GA policy preview coverage target: 100 percent for required saves.
1356. Tier Preview template install validation target: 99 percent.
1357. Tier Stable template install validation target: 99.5 percent.
1358. Tier GA template install validation target: 99.9 percent.
1359. Tier Preview migration diagnostic completeness target: 95 percent.
1360. Tier Stable migration diagnostic completeness target: 98 percent.
1361. Tier GA migration diagnostic completeness target: 99 percent.
1362. Tier Preview RTO for editor REST: 5 minutes.
1363. Tier Stable RTO for editor REST: 1 minute.
1364. Tier GA RTO for editor REST: 30 seconds.
1365. Tier Preview RPO for editor session state: 5 seconds.
1366. Tier Stable RPO for editor session state: 1 second.
1367. Tier GA RPO for editor session state: 1 second.
1368. Tier XS active tenants reference: 20.
1369. Tier S active tenants reference: 100.
1370. Tier M active tenants reference: 1000.
1371. Tier L active tenants reference: 10000.
1372. Tier XL active tenants reference: 100000.
1373. Tier XS active editor sessions reference: 100.
1374. Tier S active editor sessions reference: 1000.
1375. Tier M active editor sessions reference: 10000.
1376. Tier L active editor sessions reference: 100000.
1377. Tier XL active editor sessions reference: 1000000.
1378. Tier XS WebSocket connections reference: 120.
1379. Tier S WebSocket connections reference: 1200.
1380. Tier M WebSocket connections reference: 12000.
1381. Tier L WebSocket connections reference: 120000.
1382. Tier XL WebSocket connections reference: 1200000.
1383. SLO verification source: OpenSLO files.
1384. SLO verification source: capacity model.
1385. SLO verification source: PRD performance table.
1386. SLO verification source: failure-mode catalog.
1387. SLO failure action: page on-call for burn-rate violation.
1388. SLO failure action: degrade LLM-assist before editor.
1389. SLO failure action: fail closed for policy evaluator.
1390. SLO failure action: block publish for no-silent-loss violations.

## §13 Competitive Positioning
1391. Competitor: n8n.
1392. n8n strength: visual workflow editor.
1393. n8n strength: broad node and integration model.
1394. n8n strength: self-hosting familiarity.
1395. Oyatie differentiator: tenant-scoped Cedar preview before save.
1396. Oyatie differentiator: CRDT no-silent-loss invariant.
1397. Oyatie differentiator: canonical spec byte-equality.
1398. Oyatie differentiator: jurisdiction overlay visual diff.
1399. Oyatie caution: do not claim more connectors pre-marketplace proof.
1400. Oyatie caution: do not claim faster editor without measured evidence.
1401. Competitor: Zapier Enterprise.
1402. Zapier strength: simple trigger-action onboarding.
1403. Zapier strength: large app ecosystem.
1404. Zapier strength: AI generation entry points.
1405. Oyatie differentiator: durable workflow-engine boundary.
1406. Oyatie differentiator: typed DSL visible to developers.
1407. Oyatie differentiator: tenant policy preview.
1408. Oyatie differentiator: audit-chain evidence per save.
1409. Oyatie caution: Zapier app breadth remains a gap.
1410. Oyatie caution: preserve low-friction onboarding.
1411. Competitor: Make.com.
1412. Make strength: visual scenarios.
1413. Make strength: routers and error handlers.
1414. Make strength: incomplete execution recovery UX.
1415. Oyatie differentiator: explicit saga compensation semantics.
1416. Oyatie differentiator: durable replay timeline.
1417. Oyatie differentiator: policy-aware node configuration.
1418. Oyatie differentiator: tenant-scoped marketplace templates.
1419. Oyatie caution: migration must respect Make recovery semantics.
1420. Oyatie caution: visual graph performance must handle large scenarios.
1421. Competitor: Workato.
1422. Workato strength: enterprise connector catalog.
1423. Workato strength: recipe functions.
1424. Workato strength: recipe copilot.
1425. Oyatie differentiator: spec-first source of truth.
1426. Oyatie differentiator: custom node SDK with replay and policy hooks.
1427. Oyatie differentiator: DealSet-backed template settlement.
1428. Oyatie differentiator: data-class markers in authoring.
1429. Oyatie caution: connector breadth is a multi-quarter gap.
1430. Oyatie caution: recipe function migration needs sub-workflow parity.
1431. Competitor: Microsoft Power Automate.
1432. Power Automate strength: Microsoft ecosystem.
1433. Power Automate strength: DLP and environment governance.
1434. Power Automate strength: Copilot.
1435. Oyatie differentiator: open workflow spec.
1436. Oyatie differentiator: ontology-backed typed node configuration.
1437. Oyatie differentiator: tenant and pack portability beyond Microsoft.
1438. Oyatie differentiator: Cedar policy preview tied to node config.
1439. Oyatie caution: DLP translation must be precise.
1440. Oyatie caution: desktop-flow migration is out of browser scope.
1441. Competitor: Tray.
1442. Tray strength: enterprise automation and connectors.
1443. Tray strength: builder experience.
1444. Tray strength: API-oriented automation posture.
1445. Oyatie differentiator: shared workflow substrate for every Oyatie product.
1446. Oyatie differentiator: marketplace DealSet settlement for templates.
1447. Oyatie differentiator: CRDT offline collaboration.
1448. Oyatie differentiator: deterministic replay boundary.
1449. Oyatie caution: API developer experience must be polished.
1450. Oyatie caution: public SDK docs need launch-quality examples.
1451. Competitive principle: claims require source evidence.
1452. Competitive principle: targets are not measured facts.
1453. Competitive principle: connector breadth must be stated as roadmap until proven.
1454. Competitive principle: uniqueness claims require refresh cadence.
1455. Competitive principle: policy preview is a defensible differentiator.
1456. Competitive principle: jurisdiction overlay is a defensible differentiator.
1457. Competitive principle: byte-equal round-trip is a defensible differentiator.
1458. Competitive principle: no-silent-loss collaboration is a defensible differentiator.
1459. Competitive principle: audit-chain evidence is a defensible differentiator.
1460. Competitive principle: use migration success evidence before displacement claims.

## §14 References
1461. Source: `microservices/workflow-studio/PRD.md` lines 27-35 establish Studio identity, execution sibling, and shared substrate.
1462. Source: `microservices/workflow-studio/PRD.md` lines 40-45 establish TTI, CRDT, LLM assist, and DSL backbone outcomes.
1463. Source: `microservices/workflow-studio/PRD.md` lines 58-64 establish CRDT, Cedar preview, LLM assist, and license gate requirements.
1464. Source: `microservices/workflow-studio/PRD.md` lines 80-81 establish CRDT and LLM p99 targets.
1465. Source: `microservices/workflow-studio/PRD.md` lines 93-100 establish Cedar, validation, and tenant isolation security.
1466. Source: `microservices/workflow-studio/PRD.md` lines 108-122 establish audit, availability, and data residency.
1467. Source: `microservices/workflow-studio/PRD.md` lines 133-137 establish bounded contexts.
1468. Source: `microservices/workflow-studio/PRD.md` lines 203-213 establish CRDT crate role.
1469. Source: `microservices/workflow-studio/PRD.md` lines 279-284 establish license-gate Cedar role.
1470. Source: `microservices/workflow-studio/PRD.md` lines 329-331 establish engine, ontology, and foundry-provider dependencies.
1471. Source: `microservices/workflow-studio/PRD.md` lines 347-348 establish Cedar preview and editor-execution-forbidden gates.
1472. Source: `microservices/workflow-studio/PRD.md` lines 361-367 establish event handoffs.
1473. Source: `microservices/workflow-studio/PRD.md` lines 398-417 establish competitor set and differentiation.
1474. Source: `microservices/workflow-studio/PRD.md` lines 429-456 establish scale metrics.
1475. Source: `microservices/workflow-studio/PRD.md` lines 481-486 establish acceptance tests.
1476. Source: `specs/microservices/workflow-studio.json` lines 17-39 establish paired engine and north star.
1477. Source: `specs/microservices/workflow-studio.json` lines 93-128 establish offline, LLM, CRDT, Cedar, and license acceptance.
1478. Source: `specs/microservices/workflow-studio.json` lines 137-155 establish preview/stable/GA scope and editor execution out-of-scope.
1479. Source: `specs/microservices/workflow-studio.json` lines 202-214 establish ontology and Foundry capability descriptors.
1480. Source: `specs/microservices/workflow-studio.json` lines 235-245 establish CDN and CRDT optimization practices.
1481. Source: `specs/microservices/workflow-studio.json` lines 370-391 establish n8n comparison inputs.
1482. Source: `specs/microservices/workflow-studio.json` lines 539-586 establish Make and Zapier comparison inputs.
1483. Source: `specs/microservices/workflow-studio.json` lines 592-713 establish Power Automate and AI-generation comparison inputs.
1484. Source: `specs/microservices/workflow-studio.json` lines 768-834 establish anti-patterns and avoidance mechanisms.
1485. Source: `specs/microservices/workflow-studio.json` lines 840-884 establish scale and secret-handling targets.
1486. Source: `specs/microservices/workflow.json` lines 39-48 establish engine durability and marketplace/LLM GA scope.
1487. Source: `specs/microservices/workflow.json` lines 52-66 establish workflow out-of-scope and consumed contracts.
1488. Source: `specs/microservices/workflow.json` lines 111-112 establish n8n and Temporal competitive lessons.
1489. Source: `microservices/workflow-studio/decisions/ADR-WFS-001-yjs-crdt-for-collaborative-canvas-editing.md` lines 22-40 establish Yjs/offline context.
1490. Source: `microservices/workflow-studio/decisions/ADR-WFS-001-yjs-crdt-for-collaborative-canvas-editing.md` lines 72-90 establish Yjs decision and projection boundary.
1491. Source: `microservices/workflow-studio/decisions/ADR-WFS-001-yjs-crdt-for-collaborative-canvas-editing.md` lines 104-119 establish snapshots, latency targets, and audit hashes.
1492. Source: `microservices/workflow-studio/decisions/ADR-WFS-001-yjs-crdt-for-collaborative-canvas-editing.md` lines 191-210 establish consequences.
1493. Source: `microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md` lines 42-45 establish T0/T1/T2 tiers.
1494. Source: `microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md` lines 64-103 establish AI generation bounds.
1495. Source: `microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md` lines 164-178 establish downstream microservice impact.
1496. Source: `microservices/workflow-studio/decisions/ADR-WS-0005-ai-copilot-node-generation-bounds.md` lines 191-195 establish AI audit duties.
1497. Source: `docs/decisions/ADR-0220-consumer-intelligence-substrate.md` lines 16-23 establish Intelligence as consumer AI substrate.
1498. Source: `docs/decisions/ADR-0220-consumer-intelligence-substrate.md` lines 31-42 establish Intelligence ownership.
1499. Source: `docs/decisions/ADR-0220-consumer-intelligence-substrate.md` lines 44-55 establish shared substrate isolation.
1500. Source: `docs/decisions/ADR-0220-consumer-intelligence-substrate.md` lines 92-108 establish AI integration and operational duties.
1501. Source: `docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md` lines 31-44 establish marketplace as deal settlement substrate.
1502. Source: `docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md` lines 46-51 establish DealSet primitive and participants.
1503. Source: `docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md` lines 71-92 establish DealSet object and Cedar action gate.
1504. Source: `docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md` lines 111-119 establish marketplace, payments, treasury, finops, ontology, workflow-engine, connect, and global-trade footprint.
1505. Source: `microservices/workflow-studio/policy/tenant-scope.cedar` lines 4-10 establish policy purpose.
1506. Source: `microservices/workflow-studio/policy/tenant-scope.cedar` lines 26-65 establish own-tenant read and seat-gated session open.
1507. Source: `microservices/workflow-studio/policy/tenant-scope.cedar` lines 71-122 establish save, CRDT, and LLM-assist permits.
1508. Source: `microservices/workflow-studio/policy/tenant-scope.cedar` lines 128-158 establish debugger and node-library permits.
1509. Source: `microservices/workflow-studio/policy/tenant-scope.cedar` lines 164-194 establish cross-tenant forbid.
1510. Source: `microservices/workflow-studio/policy/tenant-scope.cedar` lines 215-253 establish SECRET, execution, and policy-preview forbids.
1511. Source: `microservices/workflow-studio/policy/ci-scope.cedar` lines 28-39 establish CI read-only permit.
1512. Source: `microservices/workflow-studio/policy/ci-scope.cedar` lines 45-85 establish CI no-draft, no-write, no-SECRET forbids.
1513. Source: `microservices/workflow-studio/sdk-plan.md` lines 27-39 establish SDK language priorities.
1514. Source: `microservices/workflow-studio/sdk-plan.md` lines 43-73 establish Rust and generated SDK strategies.
1515. Source: `microservices/workflow-studio/sdk-plan.md` lines 75-95 establish public SDK surface and publish boundary.
1516. Source: `microservices/workflow-studio/failure-modes.md` lines 30-67 establish CRDT desync, CDN, and LLM timeout failure modes.
1517. Source: `microservices/workflow-studio/failure-modes.md` lines 82-118 establish Cedar crash, WS restart, and WASM corruption.
1518. Source: `microservices/workflow-studio/failure-modes.md` lines 134-158 establish cross-tenant leak and node-library signature failure.
1519. Source: `microservices/workflow-studio/failure-modes.md` lines 199-229 establish round-trip regression and RTO/RPO summary.
1520. Source: `microservices/workflow-studio/capacity-model.md` lines 38-70 establish throughput and storage formulas.
1521. Source: `microservices/workflow-studio/capacity-model.md` lines 98-108 establish tier baselines.
1522. Source: `microservices/workflow-studio/capacity-model.md` lines 156-197 establish XS worked example.
1523. Source: `microservices/workflow-studio/competitor-parity-matrix.md` lines 25-45 establish competitor set.
1524. Source: `microservices/workflow-studio/competitor-parity-matrix.md` lines 48-89 establish feature parity gaps.
1525. Source: `microservices/workflow-studio/competitor-parity-matrix.md` lines 90-105 establish target-vs-measured caution.
1526. Source: `microservices/workflow-studio/competitor-parity-matrix.md` lines 112-133 establish gaps and differentiators.
1527. Source: `microservices/workflow-studio/competitor-parity-matrix.md` lines 134-145 establish claim-boundary rules.
1528. Source: `microservices/workflow-studio/templates/index.json` lines 1-24 establish template count, verticals, and personas.
1529. Source: `microservices/workflow-studio/templates/index.json` lines 27-64 establish interview scheduling template shape.
1530. Source: `microservices/workflow-studio/templates/index.json` lines 146-183 establish bed availability template shape.
1531. Source: `specs/design-system/workflow-canvas.json` lines 14-29 establish canvas variants and states.
1532. Source: `specs/design-system/workflow-canvas.json` lines 30-43 establish canvas accessibility and tests.
1533. Source: `specs/design-system/workflow-node-config-panel.json` lines 14-31 establish config variants and states.
1534. Source: `specs/design-system/workflow-node-config-panel.json` lines 32-45 establish config accessibility and security.
1535. Source: `specs/design-system/workflow-replay-timeline.json` lines 14-29 establish replay variants and states.
1536. Source: `specs/design-system/workflow-replay-timeline.json` lines 30-43 establish replay accessibility and security.

## Checkpoint
1537. Checkpoint status: cleanly halted after single-file scope authoring.
1538. VCS claim agent: `codex-workflow-studio-scope`.
1539. VCS claim scope: `microservices/workflow-studio`.
1540. VCS intent: `workflow-studio-hero-scope-deepening`.
1541. File touched by this pass: `microservices/workflow-studio/SCOPE.md`.
1542. Existing PRD touched: no.
1543. Existing decisions touched: no.
1544. Existing runbooks touched: no.
1545. Existing capability stages touched: no.
1546. Existing onboarding docs touched: no.
1547. Existing FAQs touched: no.
1548. Existing tutorials touched: no.
1549. Existing benchmarks touched: no.
1550. Existing migration playbooks touched: no.
1551. Existing reference implementations touched: no.
1552. Other microservices touched: no.
1553. Docs products directory existed for Workflow Studio: no matching path found.
1554. Deepening includes multi-domain coverage: yes.
1555. Deepening includes n8n-class node-graph UX: yes.
1556. Deepening includes per-tenant Cedar node-config: yes.
1557. Deepening includes AI-assisted generation with Intelligence boundary: yes.
1558. Deepening includes custom node SDK TypeScript: yes.
1559. Deepening includes custom node SDK Rust: yes.
1560. Deepening includes custom node SDK Python: yes.
1561. Deepening includes template marketplace: yes.
1562. Deepening includes workflow execution boundary: yes.
1563. Deepening includes Yjs offline editing: yes.
1564. Deepening includes vendor migration playbooks: yes.
1565. Deepening includes failure modes: yes.
1566. Deepening includes SLO targets: yes.
1567. Deepening includes competitive positioning: yes.
1568. Deepening includes cross-microservice handoffs: yes.
1569. Verification command required next: `./bin/oya vcs verify --agent codex-workflow-studio-scope --evidence 'scope_lines:X new_or_updated:Y' microservices/workflow-studio`.
1570. Done command required after verify: `./bin/oya vcs done --agent codex-workflow-studio-scope --evidence 'scope_lines:X new_or_updated:Y' microservices/workflow-studio`.
1571. Promote command required after done: `./bin/oya vcs promote --agent codex-workflow-studio-scope --bundle workflow-studio-scope-2026-05-20 --environment dev --evidence 'scope_lines:X new_or_updated:Y' microservices/workflow-studio`.

## Appendix A Direct Handoff Authority
1572. Direct handoff means Studio has a declared dependency or contract path.
1573. Indirect handoff means Studio configures a workflow node that engine executes.
1574. Descriptor-only handoff means Studio reads metadata without owning action.
1575. Admin-summary handoff means Studio reports status to a control surface.
1576. No-handoff means current authority files do not show a Studio path.
1577. Direct target: workflow-engine.
1578. Direct basis: paired engine and visual editor split.
1579. Direct basis: Studio produces canonical workflow spec.
1580. Direct basis: Studio consumes replay frames.
1581. Direct basis: Studio submits save/deploy requests by contract.
1582. Boundary: Studio never executes workflow steps.
1583. Boundary: Studio never starts production runs outside engine API.
1584. Boundary: Studio never rewrites event history.
1585. Boundary: Studio never stores runtime checkpoints.
1586. Boundary: Studio never consumes raw engine database rows.
1587. Direct target: ontology.
1588. Direct basis: Studio consumes ontology descriptors.
1589. Direct basis: Studio hot-reloads typed node configuration.
1590. Direct basis: Studio validates object type versions.
1591. Direct basis: Studio renders typed field inputs.
1592. Boundary: Studio never bypasses ontology storage.
1593. Boundary: Studio never mutates graph entities directly.
1594. Boundary: Studio never invents unversioned object types.
1595. Boundary: Studio never downgrades field versions silently.
1596. Boundary: Studio never treats descriptor cache as authority.
1597. Direct target: tenancy.
1598. Direct basis: Studio consumes tenant context.
1599. Direct basis: Studio consumes seat entitlements.
1600. Direct basis: Studio consumes pack memberships.
1601. Direct basis: Studio uses tenant headers.
1602. Boundary: Studio never fabricates tenant id.
1603. Boundary: Studio never trusts browser tenant id without server validation.
1604. Boundary: Studio never opens a session without seat decision.
1605. Boundary: Studio never shares editor sessions across tenants.
1606. Boundary: Studio never reads tenancy internals beyond contract.
1607. Direct target: Cedar policy service.
1608. Direct basis: Studio requests policy preview.
1609. Direct basis: Studio gates save, session, LLM, debugger, and node-library actions.
1610. Direct basis: Studio displays safe denial reasons.
1611. Direct basis: Studio records policy decision ids.
1612. Boundary: Studio never turns deny into warning.
1613. Boundary: Studio never exposes SECRET policy payload.
1614. Boundary: Studio never caches allow beyond policy TTL.
1615. Boundary: Studio never bypasses preview for production saves.
1616. Boundary: Studio never executes policy locally as final authority.
1617. Direct target: Intelligence.
1618. Direct basis: Studio requests consumer-facing AI drafts.
1619. Direct basis: Studio sends prompt fence metadata.
1620. Direct basis: Studio sends RAG reference ids.
1621. Direct basis: Studio receives candidate specs for review.
1622. Boundary: Studio never sends unauthorized tenant context.
1623. Boundary: Studio never sends raw secrets.
1624. Boundary: Studio never sends Foundry internal corpora for tenant prompts.
1625. Boundary: Studio never auto-saves Intelligence output.
1626. Boundary: Studio never lets Intelligence publish node packs.
1627. Direct target: Foundry or foundry-providers.
1628. Direct basis: current Studio files mention foundry-providers for LLM assist.
1629. Corrected boundary: consumer-facing AI belongs to Intelligence per ADR-0220.
1630. Corrected boundary: Foundry remains internal eval and CI surface.
1631. Corrected boundary: any tenant-facing LLM path must be described as Intelligence-mediated unless a later spec supersedes it.
1632. Boundary: Studio never markets consumer AI as Foundry-powered.
1633. Boundary: Studio never shares internal eval corpora with tenant prompts.
1634. Boundary: Studio never gives Foundry tenant audience authority.
1635. Boundary: Studio can use Foundry for internal quality gates.
1636. Boundary: Studio can consume capability descriptors by approved contract.
1637. Direct target: marketplace.
1638. Direct basis: templates and node packs require catalog and settlement doctrine.
1639. Direct basis: ADR-0314 makes DealSet settlement universal for commercial exchanges.
1640. Direct basis: template marketplace is a GA product surface.
1641. Direct basis: node-pack entitlements can be commercial grants.
1642. Boundary: Studio never settles deals itself.
1643. Boundary: Studio never grants paid template rights without DealSet.
1644. Boundary: Studio never hides counterparty role.
1645. Boundary: Studio never duplicates marketplace order tables.
1646. Boundary: Studio never bypasses marketplace revocation.
1647. Direct target: audit-chain.
1648. Direct basis: Studio emits editor and save evidence.
1649. Direct basis: Studio emits policy and license evidence.
1650. Direct basis: Studio emits migration evidence.
1651. Direct basis: Studio emits AI draft acceptance evidence.
1652. Boundary: Studio never edits sealed audit rows.
1653. Boundary: Studio never omits save evidence.
1654. Boundary: Studio never collapses multiple decisions into one opaque row.
1655. Boundary: Studio never exports audit bundles without policy.
1656. Boundary: Studio never stores raw secrets in audit payload.
1657. Direct target: observability.
1658. Direct basis: Studio emits SLO metrics.
1659. Direct basis: Studio emits CRDT health metrics.
1660. Direct basis: Studio emits UI performance metrics.
1661. Direct basis: Studio emits LLM quality metrics.
1662. Boundary: Studio never treats metrics as authorization.
1663. Boundary: Studio never treats logs as source of truth.
1664. Boundary: Studio never hides burn-rate alerts.
1665. Boundary: Studio never silently suppresses Sev-1 security metrics.
1666. Boundary: Studio never reports stale telemetry as current.
1667. Direct target: governance.
1668. Direct basis: Studio has VCS, claim, and gate expectations.
1669. Direct basis: Studio has claim-boundary and maturity rules.
1670. Direct basis: Studio emits evidence for verification.
1671. Direct basis: Studio respects active artifact contracts.
1672. Boundary: Studio never self-attests without evidence.
1673. Boundary: Studio never weakens hyperscaler claim gates.
1674. Boundary: Studio never bypasses Oya VCS promotion.
1675. Boundary: Studio never edits unrelated microservice specs during scope pass.
1676. Boundary: Studio never leaves undocumented verification gaps.

## Appendix B Indirect Node-Call Matrix
1677. Indirect target: mail.
1678. Indirect path: Studio configures mail node.
1679. Indirect executor: workflow-engine.
1680. Indirect descriptor source: connect or mail SDK metadata.
1681. Indirect policy: Cedar validates recipient and data class.
1682. Indirect idempotency: mail send requires message key.
1683. Indirect replay: payload redacted in timeline.
1684. Indirect migration: Zapier and Power Automate email steps map here.
1685. Indirect risk: duplicate send on retry.
1686. Indirect recovery: idempotency key and compensation notice.
1687. Indirect target: messenger.
1688. Indirect path: Studio configures messenger node.
1689. Indirect executor: workflow-engine.
1690. Indirect descriptor source: messenger node descriptor.
1691. Indirect policy: Cedar validates recipient scope.
1692. Indirect idempotency: notification key required.
1693. Indirect replay: message body redacted by data class.
1694. Indirect migration: delivery and healthcare examples map here.
1695. Indirect risk: unauthorized family or customer notification.
1696. Indirect recovery: revoke message task and audit incident.
1697. Indirect target: community.
1698. Indirect path: Studio configures community moderation or posting node when descriptor exists.
1699. Indirect executor: workflow-engine.
1700. Indirect descriptor source: community product manifest or connect adapter.
1701. Indirect policy: Cedar validates moderation authority.
1702. Indirect idempotency: post or moderation action key.
1703. Indirect replay: public content hash shown.
1704. Indirect migration: social/community automations map here only when in node pack.
1705. Indirect risk: public post under wrong tenant identity.
1706. Indirect recovery: rollback action and audit-chain correction event.
1707. Indirect target: payments.
1708. Indirect path: Studio configures payment authorization node.
1709. Indirect executor: workflow-engine.
1710. Indirect descriptor source: payments capability descriptor.
1711. Indirect policy: Cedar validates payment authority.
1712. Indirect idempotency: payment idempotency key required.
1713. Indirect replay: PCI-sensitive fields redacted.
1714. Indirect migration: Workato and Zapier payment steps map here.
1715. Indirect risk: duplicate charge.
1716. Indirect recovery: refund or void compensation path.
1717. Indirect target: treasury.
1718. Indirect path: Studio configures treasury action node.
1719. Indirect executor: workflow-engine.
1720. Indirect descriptor source: treasury capability descriptor.
1721. Indirect policy: Cedar validates treasury role and region.
1722. Indirect idempotency: treasury operation key required.
1723. Indirect replay: cash movement evidence linked, not duplicated.
1724. Indirect migration: ERP settlement workflows map here.
1725. Indirect risk: unauthorized cash movement.
1726. Indirect recovery: compensating treasury workflow.
1727. Indirect target: finops-portal.
1728. Indirect path: Studio configures cost allocation node.
1729. Indirect executor: workflow-engine.
1730. Indirect descriptor source: finops capability descriptor.
1731. Indirect policy: Cedar validates cost-center authority.
1732. Indirect idempotency: chargeback event key required.
1733. Indirect replay: cost record id linked.
1734. Indirect migration: business domain workflows map here.
1735. Indirect risk: misallocated cost.
1736. Indirect recovery: reversal allocation event.
1737. Indirect target: global-trade.
1738. Indirect path: Studio configures sanctions or customs node.
1739. Indirect executor: workflow-engine.
1740. Indirect descriptor source: global-trade descriptor.
1741. Indirect policy: Cedar validates trade authority.
1742. Indirect idempotency: screen request id required.
1743. Indirect replay: trade decision id linked.
1744. Indirect migration: supply-chain workflows map here.
1745. Indirect risk: shipment proceeds under trade hold.
1746. Indirect recovery: trade hold incident workflow.
1747. Indirect target: warehouse.
1748. Indirect path: Studio configures inventory node.
1749. Indirect executor: workflow-engine.
1750. Indirect descriptor source: warehouse capability descriptor.
1751. Indirect policy: Cedar validates inventory mutation authority.
1752. Indirect idempotency: reservation key required.
1753. Indirect replay: inventory mutation id linked.
1754. Indirect migration: Make and Workato inventory scenarios map here.
1755. Indirect risk: duplicate reservation.
1756. Indirect recovery: compensating inventory release.
1757. Indirect target: delivery.
1758. Indirect path: Studio configures route or ETA node.
1759. Indirect executor: workflow-engine.
1760. Indirect descriptor source: delivery capability descriptor.
1761. Indirect policy: Cedar validates customer contact and compensation.
1762. Indirect idempotency: delivery action key required.
1763. Indirect replay: route plan id linked.
1764. Indirect migration: delivery recovery templates map here.
1765. Indirect risk: duplicate customer compensation.
1766. Indirect recovery: DealSet amendment or reversal.
1767. Indirect target: connect.
1768. Indirect path: Studio configures external-system connector node.
1769. Indirect executor: workflow-engine.
1770. Indirect descriptor source: connect adapter descriptor.
1771. Indirect policy: Cedar validates connector scope.
1772. Indirect idempotency: connector request key required.
1773. Indirect replay: external correlation id linked.
1774. Indirect migration: every vendor import depends on connector mapping.
1775. Indirect risk: external API rate-limit cascade.
1776. Indirect recovery: retry policy and backpressure.
1777. Indirect target: scheduler.
1778. Indirect path: Studio configures schedule trigger.
1779. Indirect executor: workflow-engine or scheduler contract.
1780. Indirect descriptor source: scheduler trigger descriptor.
1781. Indirect policy: Cedar validates backfill authority.
1782. Indirect idempotency: schedule event key required.
1783. Indirect replay: scheduled fire id linked.
1784. Indirect migration: Make schedule and Zapier delay map here.
1785. Indirect risk: backfill storm.
1786. Indirect recovery: concurrency policy and pause.
1787. Indirect target: notifications.
1788. Indirect path: Studio configures alert node.
1789. Indirect executor: workflow-engine.
1790. Indirect descriptor source: notification capability descriptor.
1791. Indirect policy: Cedar validates channel and recipient.
1792. Indirect idempotency: alert key required.
1793. Indirect replay: alert id linked.
1794. Indirect migration: incident templates map here.
1795. Indirect risk: notification storm.
1796. Indirect recovery: throttle and suppress policy.
1797. Indirect target: data import adapters.
1798. Indirect path: Studio configures import trigger.
1799. Indirect executor: workflow-engine through connect.
1800. Indirect descriptor source: connector schema.
1801. Indirect policy: Cedar validates data class.
1802. Indirect idempotency: import batch id required.
1803. Indirect replay: batch provenance linked.
1804. Indirect migration: Workato lookup and Power Automate Dataverse map here.
1805. Indirect risk: malformed import contaminates ontology.
1806. Indirect recovery: quarantine batch and rollback projection.

## Appendix C Product Surfaces Checklist
1807. Surface: first-run workspace.
1808. Requirement: tenant, pack, environment, jurisdiction visible.
1809. Requirement: start from blank workflow.
1810. Requirement: start from template.
1811. Requirement: import from vendor.
1812. Requirement: resume offline draft.
1813. Requirement: open failed run.
1814. Requirement: request AI draft.
1815. Requirement: open SDK docs.
1816. Requirement: open admin catalog.
1817. Surface: canvas toolbar.
1818. Requirement: undo.
1819. Requirement: redo.
1820. Requirement: zoom.
1821. Requirement: fit.
1822. Requirement: select.
1823. Requirement: pan.
1824. Requirement: validate.
1825. Requirement: preview policy.
1826. Requirement: save draft.
1827. Requirement: publish through engine.
1828. Surface: node context menu.
1829. Requirement: configure.
1830. Requirement: duplicate.
1831. Requirement: disable.
1832. Requirement: convert to sub-workflow.
1833. Requirement: view policy impact.
1834. Requirement: view data classes.
1835. Requirement: view replay frames.
1836. Requirement: view DSL pointer.
1837. Requirement: add comment.
1838. Requirement: open migration lineage.
1839. Surface: edge context menu.
1840. Requirement: retarget.
1841. Requirement: delete.
1842. Requirement: add condition.
1843. Requirement: inspect payload.
1844. Requirement: inspect retry.
1845. Requirement: inspect data class.
1846. Requirement: inspect policy.
1847. Requirement: inspect replay.
1848. Requirement: inspect conflict.
1849. Requirement: inspect DSL pointer.
1850. Surface: collaboration tray.
1851. Requirement: participant list.
1852. Requirement: cursor presence.
1853. Requirement: active selection.
1854. Requirement: offline participants.
1855. Requirement: conflict count.
1856. Requirement: resync action.
1857. Requirement: export draft.
1858. Requirement: discard local queue.
1859. Requirement: session lease state.
1860. Requirement: audit-safe activity log.
1861. Surface: offline banner.
1862. Requirement: unsynced operation count.
1863. Requirement: local storage usage.
1864. Requirement: offline age.
1865. Requirement: reconnect status.
1866. Requirement: export option.
1867. Requirement: discard option.
1868. Requirement: policy recheck warning.
1869. Requirement: conflict prediction warning.
1870. Requirement: safe reload instruction.
1871. Surface: policy preview drawer.
1872. Requirement: decisions grouped by node.
1873. Requirement: affected action namespace.
1874. Requirement: principal summary.
1875. Requirement: resource summary.
1876. Requirement: context summary.
1877. Requirement: deny reason.
1878. Requirement: required grant.
1879. Requirement: admin request action.
1880. Requirement: evidence export.
1881. Surface: replay drawer.
1882. Requirement: live stream mode.
1883. Requirement: historical mode.
1884. Requirement: failure mode.
1885. Requirement: audit export mode.
1886. Requirement: stale telemetry block.
1887. Requirement: retry eligibility.
1888. Requirement: compensation eligibility.
1889. Requirement: policy recheck.
1890. Requirement: idempotency proof.
1891. Surface: marketplace catalog.
1892. Requirement: public templates.
1893. Requirement: tenant-private templates.
1894. Requirement: approved templates.
1895. Requirement: quarantined templates.
1896. Requirement: commercial terms.
1897. Requirement: DealSet status.
1898. Requirement: node-pack requirements.
1899. Requirement: migration equivalents.
1900. Requirement: install workflow.
1901. Surface: migration report.
1902. Requirement: source vendor.
1903. Requirement: source artifact hash.
1904. Requirement: mapped nodes.
1905. Requirement: unmapped nodes.
1906. Requirement: credential placeholders.
1907. Requirement: policy blockers.
1908. Requirement: test plan.
1909. Requirement: lineage metadata.
1910. Requirement: live enablement blockers.
1911. Surface: SDK console.
1912. Requirement: API key status.
1913. Requirement: tenant binding.
1914. Requirement: SDK version.
1915. Requirement: compatibility matrix.
1916. Requirement: generated snippet.
1917. Requirement: test fixture status.
1918. Requirement: signing readiness.
1919. Requirement: publish blocker.
1920. Requirement: deprecation warnings.

## Appendix D Release Gates
1921. Preview gate: Workspace Home loads under TTI target.
1922. Preview gate: Canvas Editor supports nodes and edges.
1923. Preview gate: Node Config Panel validates required fields.
1924. Preview gate: Canonical spec save succeeds.
1925. Preview gate: Canonical spec load succeeds.
1926. Preview gate: Round-trip byte-equality corpus passes.
1927. Preview gate: Basic node library ships.
1928. Preview gate: Jurisdiction overlay view-switch works.
1929. Preview gate: Cedar preview runs before save.
1930. Preview gate: Editor execution forbidden gate passes.
1931. Stable gate: Replay Debugger streams historical frames.
1932. Stable gate: CRDT collaboration passes no-silent-loss tests.
1933. Stable gate: Offline IndexedDB queue persists drafts.
1934. Stable gate: WebSocket reconnect restores session.
1935. Stable gate: Per-seat licensing denies missing entitlement.
1936. Stable gate: Template catalog supports private tenant catalog.
1937. Stable gate: SDKs pass integration tests.
1938. Stable gate: Migration import wizard handles n8n and Zapier.
1939. Stable gate: Observability alerts fire on synthetic faults.
1940. Stable gate: Security review validates SECRET redaction.
1941. GA gate: Multi-domain node libraries cover six domains.
1942. GA gate: AI assist drafts from prose with human review.
1943. GA gate: Intelligence boundary is documented and tested.
1944. GA gate: Marketplace template settlement uses DealSet.
1945. GA gate: Vendor playbooks cover n8n, Zapier, Make, Workato, Power Automate.
1946. GA gate: Power user creates valid workflow under 15 minutes.
1947. GA gate: Developer round-trips git-backed spec.
1948. GA gate: Operator diagnoses failed run under 5 minutes.
1949. GA gate: Admin curates tenant template catalog.
1950. GA gate: Competitive claim snapshot is current.
1951. Security gate: no cross-tenant editor session.
1952. Security gate: no raw SECRET in browser state.
1953. Security gate: no raw SECRET in CRDT update.
1954. Security gate: no raw SECRET in replay frame.
1955. Security gate: no policy bypass path.
1956. Security gate: no unsigned node pack load.
1957. Security gate: no unapproved marketplace install.
1958. Security gate: no consumer AI routed to Foundry audience.
1959. Security gate: no external connector without SecretRef.
1960. Security gate: no production save without preview acknowledgment.
1961. Reliability gate: no silent loss under concurrent edit drill.
1962. Reliability gate: snapshot recovery under update limit.
1963. Reliability gate: browser reload preserves offline draft.
1964. Reliability gate: workflow-engine restart replay remains visible.
1965. Reliability gate: node-library revocation propagates.
1966. Reliability gate: LLM timeout does not break manual editor.
1967. Reliability gate: CDN SRI mismatch blocks load.
1968. Reliability gate: policy evaluator fails closed.
1969. Reliability gate: template quarantine blocks installs.
1970. Reliability gate: migration ambiguity stops import.
1971. Performance gate: editor REST save p99 within tier.
1972. Performance gate: CRDT merge p99 within tier.
1973. Performance gate: TTI p99 within tier.
1974. Performance gate: node-library load p99 within tier.
1975. Performance gate: replay render p99 within tier.
1976. Performance gate: LLM draft p99 within tier.
1977. Performance gate: 1000-node canvas remains usable.
1978. Performance gate: 5000-node cold-load target measured before claim.
1979. Performance gate: WebSocket gateway handles tier connection count.
1980. Performance gate: capacity model refreshed quarterly.

## Appendix E Remaining Product Questions
1981. Question: Should Studio launch with Tray migration playbook in addition to requested vendors?
1982. Default: no, keep Tray as competitive positioning until a tenant asks.
1983. Question: Should custom node SDK be public at preview?
1984. Default: no, keep SDK tenant/partner controlled until stable.
1985. Question: Should Python SDK publish at same time as TypeScript and Rust?
1986. Default: only if LLM-orchestrator tenant demand exists before M04.
1987. Question: Should mobile editor exist at GA?
1988. Default: no, tablet/desktop/wide desktop first.
1989. Question: Should BPMN import exist?
1990. Default: no, BPMN remains projection or migration reference only.
1991. Question: Should AI drafts include cross-microservice nodes at T1?
1992. Default: no, ADR-WS-0005 forbids cross-microservice at T1.
1993. Question: Should T2-cross auto-commit after Cedar allow?
1994. Default: no, ChangeSet review remains mandatory.
1995. Question: Should template marketplace allow community publishing at preview?
1996. Default: no, tenant-private and approved catalog first.
1997. Question: Should Studio allow iframe embedding for customers?
1998. Default: no, canonical URL only until security review changes scope.
1999. Question: Should Studio expose raw execution payloads in replay?
2000. Default: no, redaction by data class and Cedar role.
2001. Question: Should migration import copy historical execution logs?
2002. Default: no, import as replay archive only when policy permits.
2003. Question: Should node packs contain executable browser code?
2004. Default: no, preview manifests only; execution belongs to engine-side adapters.
2005. Question: Should tenant admins override Cedar in UI?
2006. Default: no, UI can request grants but cannot override deny.
2007. Question: Should Studio use Foundry for tenant-facing RAG?
2008. Default: no, ADR-0220 routes consumer AI to Intelligence.
2009. Question: Should unverified competitor numbers appear in sales materials?
2010. Default: no, targets stay internal until benchmark evidence exists.
2011. Question: Should community direct handoff be named as direct dependency?
2012. Default: no, current evidence supports indirect descriptor/node path only.
2013. Question: Should mail direct handoff be named as direct dependency?
2014. Default: no, current evidence supports indirect workflow node path only.
2015. Question: Should messenger direct handoff be named as direct dependency?
2016. Default: no, current evidence supports indirect workflow node path only.
2017. Question: Should Control Center direct dependency be named now?
2018. Default: no, current scope uses admin-summary pattern until manifest proves direct path.
2019. Question: Should core direct dependency be named now?
2020. Default: no, use shared primitive language unless manifest proves direct path.
2021. Stop condition: SCOPE.md exists.
2022. Stop condition: SCOPE.md has at least 2000 lines.
2023. Stop condition: only workflow-studio SCOPE changed.
2024. Stop condition: VCS verify accepted.
2025. Stop condition: VCS done accepted.
2026. Stop condition: VCS promote accepted.
2027. Stop condition: final summary names line count.
2028. Stop condition: final summary names files touched.
2029. Stop condition: final summary names handoff boundaries.
2030. Stop condition: final summary names remaining risks.
