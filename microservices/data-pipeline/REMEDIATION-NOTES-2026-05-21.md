# Data Pipeline Remediation Notes 2026-05-21

Service: data-pipeline
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope: microservices/data-pipeline/
Audit source: coherence-audit-2026-05-20.md
Primary ADR: decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md
Completion target: author IP-031 through IP-037 under implementation-plans, add empirical validation, and record remediation status.

## 1. Remediation Summary
- 001. This note records the closeout for the data-pipeline finalizer wave.
- 002. The triggering gap was that prior work filed IP-031 through IP-037 inline but did not author the requested implementation-plans artifacts.
- 003. The new authored location is `microservices/data-pipeline/implementation-plans/`.
- 004. The root-level IP files remain historical evidence and were not moved or deleted.
- 005. This remediation note does not claim code implementation has occurred.
- 006. This remediation note records implementation plan authoring and evidence alignment.
- 007. The write scope stayed inside microservices/data-pipeline.
- 008. No adjacent microservice files were edited.
- 009. No commits were made.
- 010. No scripting was used to generate the content.
- 011. Line-count validation is expected to run after authoring.
- 012. The audit source remains the controlling evidence for why the IPs exist.
- 013. ADR-MS-001 remains the local design authority for lineage-first movement.
- 014. The 12 existing OpenSLO files remain the current service SLO substrate.
- 015. IP-001 through IP-030 remain the existing implementation-plan corpus.
- 016. IP-031 through IP-037 now exist as explicit finalizer handoff artifacts.
- 017. IP-VALIDATE-data-pipeline-empirical-numbers.md now records the parity-number validation method.
- 018. This note records all three P0 remediation classes.
- 019. This note records tenant_class adoption.
- 020. This note records tier-retirement status.

## 2. P0-1 Competitor Parity Matrix Template Stamping
- 021. Audit Section 3.1.1 identified the prior competitor-parity-matrix.md as template-stamped.
- 022. The failure pattern was repeated vendor rows with substituted bounded-context names.
- 023. The audit also recorded the industry parity dimension as red.
- 024. The current competitor-parity-matrix.md declares its prior revision state as template-stamped.
- 025. The current competitor-parity-matrix.md now uses bespoke vendor prose.
- 026. The matrix separates Fivetran, Airbyte, and dbt Cloud terminology.
- 027. The matrix names Oyatie canonical primitives instead of copying vendor names.
- 028. The matrix references IP-031 through IP-037 as the wave closure set.
- 029. The matrix records the 47 primitive count and the 38 plus 5 plus 4 arithmetic.
- 030. This note does not re-author the matrix.
- 031. This note confirms the finalizer IPs support the matrix closure claim.
- 032. IP-031 supplies the destination connector substance that the matrix references.
- 033. IP-032 supplies the scheduling substance that the matrix references.
- 034. IP-033 supplies semantic-layer substance.
- 035. IP-034 supplies exposure-tracking substance.
- 036. IP-035 supplies materialization-family substance.
- 037. IP-036 supplies package-management substance.
- 038. IP-037 supplies CDK authoring substance.
- 039. IP-VALIDATE supplies the source-bound numerical validation method.
- 040. Status: P0-1 is document-remediated for the finalizer lane.

## 3. P0-2 PRD Template Stamping
- 041. Audit Section 3.1.1 identified PRD Sections B, C, D, and H as template-stamped.
- 042. The failure pattern was persona and requirement multiplication without bespoke domain detail.
- 043. The current PRD front matter records a 2026-05-21 wave-15A remediation history.
- 044. The current PRD references the finalizer wave and the 37 implementation plans.
- 045. The current PRD references 12 OpenSLOs, 20 runbooks, Cedar fragments, and ADR-MS-001.
- 046. The current PRD includes concrete user stories for scheduled pipelines, semantic metrics, materializations, package reuse, and destination loads.
- 047. The PRD now reflects tenant_class doctrine.
- 048. The PRD still contains some product taxonomy fields such as capability tier.
- 049. Those fields are not customer-facing price tiers.
- 050. This finalizer work avoids adding new PRD prose.
- 051. Instead, it supplies the missing implementation-plan depth the PRD references.
- 052. IP-031 gives PRD destination-load acceptance gates a concrete handoff.
- 053. IP-032 gives PRD schedule acceptance gates a concrete handoff.
- 054. IP-033 gives PRD semantic metric acceptance gates a concrete handoff.
- 055. IP-034 gives PRD exposure impact acceptance gates a concrete handoff.
- 056. IP-035 gives PRD materialization acceptance gates a concrete handoff.
- 057. IP-036 gives PRD package reuse acceptance gates a concrete handoff.
- 058. IP-037 gives PRD CDK acceptance gates a concrete handoff.
- 059. IP-VALIDATE gives PRD empirical-number claims a validation handoff.
- 060. Status: P0-2 is document-remediated for the finalizer lane.

## 4. P0-3 Architecture Anchor Template Stamping
- 061. Audit Section 3.1.1 identified ARCHITECTURE Section F anchor expansions as template-stamped.
- 062. The failure pattern was repeated depth-detail expansion with only anchor names changed.
- 063. The current ARCHITECTURE front matter records remediation history.
- 064. The current ARCHITECTURE Section C names the bounded contexts and sub-contexts added in this wave.
- 065. The current ARCHITECTURE names destination-connector and schedule as primary bounded contexts.
- 066. The current ARCHITECTURE names semantic-layer under transform.
- 067. The current ARCHITECTURE names materialization-policy under transform.
- 068. The current ARCHITECTURE names package-management under transform.
- 069. The current ARCHITECTURE names exposure-tracking under lineage.
- 070. The current ARCHITECTURE names cdk-authoring under connector.
- 071. The finalizer IPs now provide the detailed implementation handoff behind those architecture rows.
- 072. IP-031 defines destination authority allocation with data-warehouse as substrate dependency.
- 073. IP-032 defines workflow-engine delegation without losing local schedule ownership.
- 074. IP-033 defines the semantic metric registry boundary.
- 075. IP-034 defines exposure registration and impact notification.
- 076. IP-035 defines materialization family behavior.
- 077. IP-036 defines package registry and lockfile behavior.
- 078. IP-037 defines the Rust CDK authoring workflow.
- 079. No architecture file was edited in this finalizer step.
- 080. Status: P0-3 is document-remediated for the finalizer lane.

## 5. IP-031 Substance
- 081. IP-031 establishes destination-connector as a first-class bounded context.
- 082. The plan separates source extraction from destination loading.
- 083. The plan defines destination_load_run as the aggregate.
- 084. The plan covers warehouse, lakehouse, object-lake, streaming, ontology, analytics, reverse-ETL, and custom destination classes.
- 085. The plan defines REST, gRPC, AsyncAPI, capability, policy, runbook, and SLO surfaces.
- 086. The plan binds destination commit to IP-030 landed watermark advancement.
- 087. The plan binds schema fingerprint checks to IP-026 drift disposition.
- 088. The plan binds rollback to rollback_bundle_id and IP-028 custody.
- 089. The plan binds marketplace reverse-ETL connectors to IP-014 DealSet.
- 090. The plan cites all 12 existing OpenSLOs and adds local-destination-commit-latency.
- 091. The plan cites IP-001 through IP-030 with destination-specific dependency language.
- 092. The plan cites ADR-MS-001 lineage-first output publication.
- 093. The plan defines acceptance gates for manifest, contracts, Cedar, watermark, rollback, and SLO.
- 094. Status: IP-031 substance gate is satisfied for plan authoring.

## 6. IP-032 Substance
- 095. IP-032 establishes schedule as the local bounded context for cadence and fire evidence.
- 096. The plan keeps workflow-engine as orchestrator while data-pipeline owns schedule facts.
- 097. The plan covers cron, interval, event, sensor, continuous, and manual cadences.
- 098. The plan defines pipeline_schedule and scheduled_run_instance.
- 099. The plan defines schedule.define, arm, fire, pause, retire, resolve-sensor, and lease-renew paths.
- 100. The plan binds fire checks to IP-017 cost, IP-018 capacity, IP-026 drift, and IP-030 freshness.
- 101. The plan defines HLC-backed tick behavior.
- 102. The plan defines Cedar denial rules for quota, pack overlay, drift, cost, HLC drift, and audit outage.
- 103. The plan defines schedule events including missed and continuous lease renewal.
- 104. The plan cites all 12 existing OpenSLOs and adds local-schedule-fire-jitter.
- 105. The plan cites IP-001 through IP-030 with schedule-specific dependency language.
- 106. The plan cites ADR-MS-001 preconditions for ActionAccepted.
- 107. The plan defines acceptance gates for cadence tests, workflow contract, Cedar, and SLO.
- 108. Status: IP-032 substance gate is satisfied for plan authoring.

## 7. IP-033 Substance
- 109. IP-033 adds semantic-layer as a transform sub-context.
- 110. The plan defines semantic_metric_definition as the aggregate.
- 111. The plan covers metric names, versions, expressions, dimensions, entity joins, time grains, and materialization pointers.
- 112. The plan defines define, amend, approve, deprecate, query-plan, and read surfaces.
- 113. The plan binds pack overlays to dimension and time-grain authorization.
- 114. The plan binds ontology and analytics consumers through a registry contract.
- 115. The plan binds metric materialization to IP-035.
- 116. The plan binds metric exposures to IP-034.
- 117. The plan defines events for define, approve, amend, deprecate, query planned, query denied, and rollback.
- 118. The plan cites all 12 existing OpenSLOs and adds local-semantic-metric-read-latency.
- 119. The plan cites IP-001 through IP-030 with semantic-layer-specific dependency language.
- 120. The plan cites ADR-MS-001 transform lineage and quality quarantine rules.
- 121. The plan defines acceptance gates for contracts, Cedar, pack tests, lineage, materialization, and exposure refs.
- 122. Status: IP-033 substance gate is satisfied for plan authoring.

## 8. IP-034 Substance
- 123. IP-034 adds exposure-tracking as a lineage sub-context.
- 124. The plan defines data_exposure as the aggregate.
- 125. The plan covers dashboard, ml_model, customer_api, marketplace_app, marketplace_workflow, ontology_projection, partner_integration, regulatory_report, and internal_report.
- 126. The plan defines register, amend, promote, deprecate, notify-impact, upstream query, and downstream query surfaces.
- 127. The plan binds marketplace exposure types to IP-014 DealSet.
- 128. The plan binds production maturity to owner_team, oncall_contact, and runbook_url.
- 129. The plan binds impact notification to IP-026, IP-031, IP-033, IP-035, IP-036, and IP-037 changes.
- 130. The plan defines notification dead-letter handling through IP-028.
- 131. The plan defines events for registered, amended, promoted, deprecated, impact_notified, dead_letter, and rollback.
- 132. The plan cites all 12 existing OpenSLOs and adds local-exposure-impact-notify-lag.
- 133. The plan cites IP-001 through IP-030 with exposure-specific dependency language.
- 134. The plan cites ADR-MS-001 OpenLineage-compatible facets.
- 135. The plan defines acceptance gates for nine exposure types, DealSet, notification, and lineage query.
- 136. Status: IP-034 substance gate is satisfied for plan authoring.

## 9. IP-035 Substance
- 137. IP-035 adds materialization-policy as a transform sub-context.
- 138. The plan defines materialization_policy_binding as the aggregate.
- 139. The plan covers view, table, incremental, ephemeral, and snapshot families.
- 140. The plan binds table, incremental, and snapshot to IP-031 destination_load_run.
- 141. The plan binds refresh cadence to IP-032 schedule.
- 142. The plan binds incremental cursor behavior to IP-030 watermarks.
- 143. The plan binds semantic metric query planning to IP-033.
- 144. The plan binds downstream impact to IP-034.
- 145. The plan defines events for defined, amended, promoted, refresh_started, refreshed, refresh_failed, deprecated.
- 146. The plan cites all 12 existing OpenSLOs and adds two materialization SLOs.
- 147. The plan cites IP-001 through IP-030 with materialization-specific dependency language.
- 148. The plan cites ADR-MS-001 transform output authority and quality quarantine.
- 149. The plan defines acceptance gates for families, Cedar, watermark, destination load, schedule, semantic, exposure, and SLO.
- 150. Status: IP-035 substance gate is satisfied for plan authoring.

## 10. IP-036 Substance
- 151. IP-036 adds package-management as a transform sub-context.
- 152. The plan defines package_manifest_binding as the aggregate.
- 153. The plan covers transform_package, connector_package, semantic_metric_package, materialization_template_package, exposure_template_package, compliance_pack_extension_package, runbook_package, and dataset_package.
- 154. The plan defines publish, install, uninstall, pin, update, verify-signature, and lockfile read surfaces.
- 155. The plan defines deterministic lockfile semantics.
- 156. The plan binds marketplace package categories to IP-014 DealSet.
- 157. The plan binds connector packages to IP-037.
- 158. The plan binds semantic metric, materialization, and exposure packages to IP-033, IP-035, and IP-034.
- 159. The plan defines package events for publish, signature verification, lockfile resolution, install, pin, update, and uninstall.
- 160. The plan cites all 12 existing OpenSLOs and adds local-package-install-latency.
- 161. The plan cites IP-001 through IP-030 with package-specific dependency language.
- 162. The plan cites ADR-MS-001 replay lockfile and lineage-first implications.
- 163. The plan defines acceptance gates for package categories, lockfile determinism, signatures, DealSet, Foundry approval, and drift denial.
- 164. Status: IP-036 substance gate is satisfied for plan authoring.

## 11. IP-037 Substance
- 165. IP-037 adds cdk-authoring as a connector sub-context.
- 166. The plan defines custom_connector_authoring_case as the aggregate.
- 167. The plan is Rust-strict and explicitly rejects Python, TypeScript, and Java runtime surfaces for data-pipeline CDK.
- 168. The plan covers ten scaffold kinds across source and destination connectors.
- 169. The plan defines scaffold, test, lint, package, publish, and withdraw surfaces.
- 170. The plan requires integration, contract, replay, drift, and watermark monotonicity test suites.
- 171. The plan packages CDK output through IP-036 connector_package.
- 172. The plan binds marketplace publish to IP-014 DealSet.
- 173. The plan requires human approval for Foundry-authored marketplace publish.
- 174. The plan defines CDK events for scaffold, test, lint, package, publish, withdraw, and publish_blocked.
- 175. The plan cites all 12 existing OpenSLOs and adds two CDK SLOs.
- 176. The plan cites IP-001 through IP-030 with CDK-specific dependency language.
- 177. The plan cites ADR-MS-001 connector evidence, replay, drift, dead-letter, and audit requirements.
- 178. The plan defines acceptance gates for Rust traits, scaffold fixtures, no-Python enforcement, package integration, DealSet, and suite outputs.
- 179. Status: IP-037 substance gate is satisfied for plan authoring.

## 12. Empirical Validation Substance
- 180. IP-VALIDATE-data-pipeline-empirical-numbers.md validates the 47 union primitive count.
- 181. The validation file states the arithmetic as 38 covered plus 5 feature gaps plus 4 doctrinal divergences.
- 182. The validation file reconciles IP-031 and IP-032 as operating-bar closures rather than feature-parity arithmetic changes.
- 183. The validation file cites published Fivetran sync overview evidence.
- 184. The validation file cites published Airbyte speed, checkpointing, and progress-monitoring evidence.
- 185. The validation file cites published dbt scheduler, deploy job, run-results, Semantic Layer, benchmark, and Fusion performance evidence.
- 186. The validation file distinguishes public numeric benchmarks from shape-only published documentation.
- 187. The validation file refuses unavailable public vendor numbers by requiring `vendor_number_source = unavailable_public`.
- 188. The validation file preserves the four doctrine divergences as deliberate decisions, not implementation gaps.
- 189. The validation file links all claims back to ADR-MS-001 where local lineage, replay, and audit evidence control.
- 190. Status: empirical validation substance gate is satisfied for plan authoring.

## 13. Tenant Class Adoption
- 191. tenant_class is the customer-facing discrimination axis.
- 192. Allowed tenant_class values are demo_trial and paid.
- 193. The manifest declares tenant_class_doctrine with feature_surface identical across classes.
- 194. The manifest declares discriminator as metering, capacity, and billing.
- 195. The finalizer IPs carry tenant_class in event shapes.
- 196. The finalizer IPs use tenant_class for capacity, billing, package eligibility, and policy context.
- 197. The finalizer IPs do not add Bronze, Silver, Gold, Platinum, Starter, Pro, or Enterprise customer tiers.
- 198. Demo_trial may have capacity caps and expiry windows.
- 199. Paid may have composable billing components.
- 200. Neither class receives a different feature surface in these plans.
- 201. Compliance packs remain pack overlays, not customer tiers.
- 202. Marketplace DealSet remains commercial settlement, not a tier system.
- 203. Foundry principal classes remain Cedar-governed actor classes, not pricing classes.
- 204. Status: tenant_class adoption is reflected in the finalizer IP corpus.

## 14. Tier Retirement Status
- 205. Customer-facing tier deltas remain retired for this service.
- 206. The audit allowed ADR-0248 cellular tiers as topology language only.
- 207. The manifest still includes `tier`, `tier_subtype`, and `cell_eligibility.eligible_tiers`.
- 208. Those terms are documented as product classification and ADR-0248 cell topology, not customer pricing tiers.
- 209. The finalizer IPs avoid customer tier terminology.
- 210. The finalizer IPs use tenant_class for customer axis language.
- 211. The finalizer IPs use home_cell and pack overlay for topology and residency.
- 212. The finalizer IPs use billing components for paid metering.
- 213. The finalizer IPs use DealSet for marketplace commercial terms.
- 214. The finalizer IPs use capacity and quota for operational throttles.
- 215. No tier-retirement rollback is introduced.
- 216. No feature tier matrix is introduced.
- 217. No per-seat metering is introduced.
- 218. Any remaining `tier-1`, `tier-2`, or `tier-3` phrase must be read as ADR-0248 cellular topology.
- 219. Future edits should prefer `home_cell_class` or `cell_topology_tier` if ambiguity persists.
- 220. Status: tier retirement remains active and preserved.

## 15. SLO and Corpus Coverage
- 221. All seven IPs cite the 12 existing OpenSLO files.
- 222. The cited OpenSLOs are availability, read-latency, write-latency, policy-decision-latency, audit-emission-lag, local-ingest-freshness, local-schema-drift-latency, local-transform-latency, local-lineage-capture, local-quality-null-rate, replay-freshness, and local-deadletter-rate.
- 223. All seven IPs cite IP-001 through IP-030.
- 224. Each IP uses a dependency list tailored to its bounded-context purpose.
- 225. IP-031 focuses on destination commit, rollback, landed watermark, and data-warehouse boundary.
- 226. IP-032 focuses on cadence, workflow-engine handoff, cost, capacity, drift, and HLC.
- 227. IP-033 focuses on semantic registry, dimensions, pack overlays, ontology, and analytics.
- 228. IP-034 focuses on downstream consumers, exposure maturity, DealSet, and impact notifications.
- 229. IP-035 focuses on materialization family rules, refresh, watermarks, and rollback.
- 230. IP-036 focuses on package categories, lockfiles, signatures, DealSet, and replay.
- 231. IP-037 focuses on Rust CDK scaffold, suites, no-Python, publishing, and marketplace approval.
- 232. ADR-MS-001 is cited in each IP.
- 233. The audit source is cited in each IP.
- 234. The feature parity source is cited in the validation artifact.
- 235. The competitor parity source is cited in the validation artifact.
- 236. Status: corpus coverage requirement is satisfied for authored plans.

## 16. Remaining Risks
- 237. These are implementation plans, not executable code.
- 238. Some contracts named by the IPs may still need separate implementation files.
- 239. Some SLO projections named by the IPs may still need separate OpenSLO files.
- 240. Some runbooks named by the IPs may still need separate authoring.
- 241. Cross-microservice contracts with data-warehouse, workflow-engine, ontology, analytics, and marketplace remain coordinated follow-up work.
- 242. Published vendor benchmark facts are cited in the validation file, but local Oyatie load tests remain required before claiming equal or better throughput.
- 243. The validation file intentionally marks unavailable public vendor numbers as unavailable_public.
- 244. The root-level IP-031 through IP-037 remain present and may be redundant with implementation-plans copies.
- 245. This finalizer did not delete redundant root-level files because deletion was not requested.
- 246. This finalizer did not edit manifest, PRD, ARCHITECTURE, or parity matrix.
- 247. If future automation expects only root-level IPs, it must be taught to read implementation-plans or a separate migration should consolidate.
- 248. If future automation expects only implementation-plans, root-level IPs should be archived in a separate explicit change.
- 249. No destructive cleanup was performed.
- 250. Status: risks are documented and bounded.

## 17. Completion Status
- 251. IP-031 destination connector implementation plan authored.
- 252. IP-032 scheduling implementation plan authored.
- 253. IP-033 semantic layer implementation plan authored.
- 254. IP-034 exposure tracking implementation plan authored.
- 255. IP-035 materialization families implementation plan authored.
- 256. IP-036 package management implementation plan authored.
- 257. IP-037 CDK authoring workflow implementation plan authored.
- 258. IP-VALIDATE-data-pipeline-empirical-numbers.md authored.
- 259. REMEDIATION-NOTES-2026-05-21.md authored.
- 260. Completion report is embedded below as an HTML comment.

<!--
COMPLETION REPORT
wave: 15A-DATA-PIPELINE-FINALIZER
service: data-pipeline
scope: microservices/data-pipeline/
authored_files:
  - microservices/data-pipeline/implementation-plans/IP-031-destination-connector.md
  - microservices/data-pipeline/implementation-plans/IP-032-scheduling.md
  - microservices/data-pipeline/implementation-plans/IP-033-semantic-layer.md
  - microservices/data-pipeline/implementation-plans/IP-034-exposure-tracking.md
  - microservices/data-pipeline/implementation-plans/IP-035-materialization-families.md
  - microservices/data-pipeline/implementation-plans/IP-036-package-management.md
  - microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md
  - microservices/data-pipeline/implementation-plans/IP-VALIDATE-data-pipeline-empirical-numbers.md
  - microservices/data-pipeline/REMEDIATION-NOTES-2026-05-21.md
p0s_covered:
  - competitor-parity-matrix template stamping
  - PRD B/C/D/H template stamping
  - ARCHITECTURE F anchor template stamping
audit_gaps_covered:
  - destination connectors named bounded context
  - scheduling named bounded context
  - semantic-layer metrics
  - exposure tracking
  - materialization families
  - package management
  - CDK authoring workflow
tenant_class_status: adopted as customer axis with demo_trial and paid
tier_retirement_status: customer-facing tier deltas remain retired; ADR-0248 cellular tiers are topology only
commits: none
write_scope: microservices/data-pipeline only
-->

## Wave 15-IP-substance scrub (2026-05-21)

- Rewritten in place: none. The data-pipeline IP set was already substantive, with 148-321 line bespoke implementation plans grounded in PRD, ARCHITECTURE, contracts, Cedar policy, SLOs, runbooks, and Fivetran/Airbyte/dbt parity.
- Preserved as already substantive: all 38 data-pipeline IP files.
- Deleted as duplicative: none.
- Counterpart anchors added: grep-visible verification note naming Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab while preserving the existing Fivetran/Airbyte/dbt Cloud primary parity model.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-adapter-valkey.yaml`
- `microservices/data-pipeline/coherence-audit-2026-05-20.md`
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md`

Counterpart-fact preservations:
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` preserves the Redis connector class as a Fivetran/Airbyte counterpart-fact while routing Oyatie coverage to adapter-valkey.

Files renamed (git mv):
- `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-adapter-redis.yaml` -> `microservices/data-pipeline/catalog/oya-data-pipeline-lineage-replay-adapter-valkey.yaml` (untracked file moved with `mv`)
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 46.
- Trigger A matched: 16.
- Trigger B matched: 46.
- Trigger C matched: 39.
- Trigger D matched: 5.
- IPs unmatched: 0.

### IP changes
- `microservices/data-pipeline/IP-001-tenant-scope-kernel.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-002-cedar-default-deny.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-003-ontology-projection.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-004-workflow-template-library.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-005-rest-contract-surface.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-006-async-event-surface.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-007-grpc-internal-surface.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-008-policy-eval-library-binding.md` — added DR posture.
- `microservices/data-pipeline/IP-009-credential-sidecar-binding.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-010-multi-region-cell-layout.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-011-observability-audit-events.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-012-abuse-defence-edge-waf.md` — added API Versioning, DR posture.
- `microservices/data-pipeline/IP-013-emergency-services-bypass.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-015-data-residency-pack-overlays.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-016-backfill-replay-worker.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-017-cost-budget-enforcer.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-018-capacity-admission-control.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-019-sdk-client-generation.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-020-catalog-layer-registration.md` — added DR posture.
- `microservices/data-pipeline/IP-021-slo-gated-promotion.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-022-chaos-drill-pack.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-023-dpia-evidence-packet.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-024-threat-model-control-map.md` — added DR posture, Sustainability emission, Pod runtime tier.
- `microservices/data-pipeline/IP-025-audit-findings-closeout.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-026-connector-schema-drift-quarantine.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md` — added API Versioning, DR posture.
- `microservices/data-pipeline/IP-028-dead-letter-replay-custody.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-029-transform-cost-attribution.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md` — added API Versioning, DR posture.
- `microservices/data-pipeline/IP-031-destination-connector.md` — added API Versioning, DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-032-scheduling.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-033-semantic-layer.md` — added DR posture.
- `microservices/data-pipeline/IP-034-exposure-tracking.md` — added DR posture, Pod runtime tier.
- `microservices/data-pipeline/IP-035-materialization-families.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/IP-036-package-management.md` — added DR posture, Sustainability emission, Pod runtime tier.
- `microservices/data-pipeline/IP-037-cdk-authoring-workflow.md` — added DR posture, Sustainability emission, Pod runtime tier.
- `microservices/data-pipeline/IP-VALIDATE-empirical-numbers.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/implementation-plans/IP-031-destination-connector.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/implementation-plans/IP-032-scheduling.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/implementation-plans/IP-033-semantic-layer.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/implementation-plans/IP-034-exposure-tracking.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/implementation-plans/IP-035-materialization-families.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/implementation-plans/IP-036-package-management.md` — added DR posture, Sustainability emission.
- `microservices/data-pipeline/implementation-plans/IP-037-cdk-authoring-workflow.md` — added DR posture, Sustainability emission, Pod runtime tier.
- `microservices/data-pipeline/implementation-plans/IP-VALIDATE-data-pipeline-empirical-numbers.md` — added DR posture, Sustainability emission.

### Follow-up
- `microservices/data-pipeline/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: set effective RTO/RPO to 3600s/300s under ADR-0343 because HIPAA-2024 and KR-PIPA protected-data floors dominate SOC2/ISO/PCI for this service's active compliance packs. Alternative considered: leave availability at the existing 99.9% SLO only; rejected because replay and lineage custody need explicit recovery loss windows. Cost: cursor checkpoints, dead-letter custody, and pack-aware active-active control planes become required evidence.
- Capacity model: anchored ADR-0340 to worker tokens: 0.5 vCPU/1GiB per connector/transform token, 5GiB metadata/checkpoint storage, 10 connections, and a 32-token per-tenant cap unless IP-018 grants an override. Alternative considered: one shared pipeline worker pool; rejected because backfills would starve CDC freshness. Cost: more quota accounting across connector, transform, destination, replay, and schedule contexts.
- Sustainability + cost attribution: kept IP-029 as the tenant surface and required ADR-0344 fields on connector, transform, materialization, replay, and package rows. Alternative considered: bill only destination bytes and connector calls; rejected because transform and replay are the expensive tenant-visible decisions. Cost: every movement/replay event must carry six FinOps axes and carbon metrics.
- API versioning posture: adopted ADR-0342 carrier triplet, SDK semver, last-three/180-day support, and tenant pinning for connector packages/CDK clients. Alternative considered: package lockfiles only; rejected because API drift can still break long-running migrations. Cost: route, proto, and SDK compatibility lanes must stay live across three public contract dates.


## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- baseline_cpu_per_tenant: 0.45 vCPU; baseline_ram_per_tenant: 768 MiB; storage_per_tenant: 20 GB.
- connections_per_tenant: valkey=3, postgres=5, outbound_http=16.
- scaling_dimension: per_workflow_run; cell_placement_class: Tier-2.
- ADR: ADR-0340 capacity-model doctrine plus ADR-0248 cell criticality numbering.
- Why: 0.45 vCPU / 768 MiB / 20 GB covers connector state, replay checkpoints, lineage metadata, and destination buffering per tenant.
- Rejected: per_request sizing was rejected because pipeline work is batch/run shaped and can continue after the initiating request exits.
- Cost: Tier-2 placement reserves capability-cell capacity for data movement without paying Tier-1 substrate isolation costs.

### Block 2: dr
- rto_p99_seconds: 3600; rpo_p99_seconds: 300; multi_region_active_active: true.
- backup_substrate: postgres_wal_g, iceberg_snapshot, object_storage_versioned, valkey; failover_runbook: runbooks/local-pipeline-replay-window.md; replication_shape: active-active-multi-az-cross-region.
- ADR: ADR-0343 recoverability doctrine and compliance-pack floors.
- Why: RTO 3600s / RPO 300s follows HIPAA-style tenant data floors while relying on replayable pipeline checkpoints.
- Rejected: RPO 900s was rejected because lineage and destination reconciliation would lose too much tenant change history.
- Cost: Recovery SLOs now require drill evidence that proves the declared substrate set, not only service process restart.

### Block 3: pod_runtime_tier
- pod_runtime_tier: 2; evidence: microservices/data-pipeline/PRD.md, microservices/data-pipeline/ARCHITECTURE.md, microservices/data-pipeline/IP-016-backfill-replay-worker.md, microservices/data-pipeline/contracts/openapi-v1.yaml.
- ADR: ADR-0338 pod runtime tier doctrine and ADR-0340 D-6 cell/runtime co-variance.
- Why: Data Pipeline is a first-party tenant data movement capability; it handles tenant datasets but does not execute tenant-customer code or own key/audit substrate duties called out for Tier 0 or Tier 1.
- Rejected: Tier 1 substrate placement was rejected because the service consumes secrets and data substrates but does not own the key/audit substrate itself.
- Cost: Admission, scheduling, and isolation tests must preserve this tier when runtime surfaces move.

### Block 4: tenant_version_pinning
- declared_versions: 2025-11-21, 2026-02-21, 2026-05-21; default_version: 2026-05-21.
- supported_window_size: 3; supported_window_minimum_days: 180; supports_per_tenant_pinning: true.
- ADR: ADR-0342 tenant version pinning doctrine.
- Why: Public contracts are tenant-visible and must remain selectable across the minimum support window.
- Rejected: internal-only versioning was rejected because connectors and destination contracts are tenant-visible integration surfaces.
- Cost: Release work must carry compatibility tests and deprecation-calendar updates before any breaking contract change.

### Block 5: consumes_upstream_oss
- consumes_upstream_oss: postgresql, valkey, kafka, iceberg, cedar, openbao, opentofu.
- oss_stewardship_class_overrides: none; registry defaults in specs/oss-stewardship-registry.json remain authoritative.
- ADR: ADR-0345 OSS stewardship doctrine.
- Why: Postgres, Valkey, Kafka, Iceberg, Cedar, OpenBao, and OpenTofu cover run state, queues, lake snapshots, policy, secrets, and shared IaC.
- Rejected: service-local stewardship classes without registry backing.
- Cost: CVE response ownership must follow the registry/default ownership for every declared upstream.

### Block 6: iac_module_invocations
- iac_module_invocations: oci-guest/k8s-namespace-bootstrap@v1, oci-guest/secrets-bootstrap@v1, oci-guest/vpc@v1.
- ADR: ADR-0339 shared IaC module doctrine.
- Why: Namespace, secret, and VPC module declarations reflect connector egress and isolated data-plane networking.
- Rejected: per-connector bespoke network modules were rejected because ADR-0339 requires shared cloud primitives.
- Cost: Cloud primitive changes now flow through shared module pins instead of service-local drift.
