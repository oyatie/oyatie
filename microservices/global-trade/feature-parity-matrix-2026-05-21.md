# Global Trade Feature-Parity Matrix - 2026-05-20

Target service: `global-trade`.
Target path: `microservices/global-trade/`.
Counterpart set: SAP Global Trade Services / Thomson Reuters ONESOURCE Global Trade / Descartes.
Audit batch rule: no tenant-class delta deliverable.
Matrix model: union coverage against three industry counterparts.
Tenant-class rule: uniform feature quality across `demo_trial`, `paid`, and `revenue_share`; commercial or infrastructure caps are overlays, not feature downgrades.
Local scope anchor: `PRD.md:24-32` defines SAP GTS parity for customs declarations, sanctions screening, export controls, trade documents, denied-party hits, and broker filing.
Local architecture anchor: `ARCHITECTURE.md:19-22` defines owned and non-owned surfaces.
Local contract anchor: `contracts/openapi-v1.yaml:16-129` exposes six mutation endpoints.
Local event anchor: `contracts/asyncapi-v1.yaml:8-37` exposes six event channels.
Local proto anchor: `contracts/global-trade-v1.proto:94-100` exposes six RPCs.
Chat-history anchor: `.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:17219` names this batch's three counterpart families.
Public SAP anchor: SAP Global Trade Services product page lists sanctioned-party screening, customs management, import/export management, real-time compliance checks, special customs procedures, and HANA-backed analytics.
Public Thomson Reuters anchor: ONESOURCE Global Trade page lists product classification, denied-party screening, import/export operations, duty optimization, 220+ countries/territories, 500+ free-trade agreements, 750+ sanctions/restricted-party lists, 150 researchers, 1,300 sources, and 130 million regulatory updates annually.
Public Descartes anchor: Descartes Global Trade Intelligence page lists import/export data, duty/tariff content, export compliance, classification, denied-party screening, FTZ management, AI sanctions screening, 180+ countries, 6 million regulatory sources, 30 percent duty/tariff savings, and 75 percent manual screening-time reduction.

## Counterpart 1 - SAP Global Trade Services Capability Surface

1. SAP surface: sanctioned-party list screening across transaction participants.
2. SAP surface: blocked-document worklists and escalation workflows.
3. SAP surface: customs management for import and export procedures.
4. SAP surface: product and tariff classification for customs documents.
5. SAP surface: import/export management connected to logistics processes.
6. SAP surface: government customs-system connectivity.
7. SAP surface: embargo checks.
8. SAP surface: export license management.
9. SAP surface: inline blocking and release for non-compliant transactions.
10. SAP surface: real-time compliance checks.
11. SAP surface: integration with order and shipment processes.
12. SAP surface: special customs procedures.
13. SAP surface: foreign trade zones.
14. SAP surface: processing trade in China.
15. SAP surface: bonded warehousing.
16. SAP surface: inward processing settlement.
17. SAP surface: Intrastat support.
18. SAP surface: EMCS support.
19. SAP surface: centralized compliance-data repository.
20. SAP surface: HANA-backed accelerated analysis.
21. SAP surface: data modeling and lifecycle management.
22. SAP surface: document archiving and audit continuity.
23. SAP surface: compliance-management integration with customs-management documents.
24. SAP surface: business-partner sanctioned-party screening.
25. SAP surface: customs-document screening.
26. SAP surface: legal-control checks for exports.
27. SAP surface: license and regulatory evidence capture.
28. SAP surface: workflow escalation for questionable hits.
29. SAP surface: self-filing and broker communication.
30. SAP surface: tariff and duty strategy.
31. Local coverage: original six contexts cover customs declaration, sanctions screening, export-control classification, trade documents, denied-party hits, and broker filing.
32. Local coverage: `PRD.md:70-164` defines those six functional areas.
33. Local coverage: `ADR-GT-001:59-91` defines a deterministic TradeHold state machine.
34. Local gap: special customs procedures are not first-class in core contracts.
35. Local gap: FTZ is not first-class in core contracts.
36. Local gap: Intrastat and EMCS are not first-class in core contracts.
37. Local gap: HANA-style accelerated analytics is not mapped to an Oyatie equivalent beyond dashboards and capacity rows.
38. Local gap: government customs-system integrations are not decomposed by jurisdiction or protocol.
39. Local gap: self-filing and broker communication are present as broker filing but not adapter-specific.
40. SAP parity verdict: partial current coverage, stronger plan coverage, incomplete contract coverage.

## Counterpart 2 - Thomson Reuters ONESOURCE Global Trade Capability Surface

1. Thomson Reuters surface: unified global-trade lifecycle platform.
2. Thomson Reuters surface: product classification.
3. Thomson Reuters surface: denied-party screening.
4. Thomson Reuters surface: import operations.
5. Thomson Reuters surface: export operations.
6. Thomson Reuters surface: duty optimization programs.
7. Thomson Reuters surface: complete HS code and duty-rate content for 220+ countries and territories.
8. Thomson Reuters surface: antidumping and countervailing duty content.
9. Thomson Reuters surface: ECCN and munitions-code content.
10. Thomson Reuters surface: detailed requirements for 500+ free-trade agreements.
11. Thomson Reuters surface: OGA/PGA requirements.
12. Thomson Reuters surface: restricted-items rules.
13. Thomson Reuters surface: licensing rules.
14. Thomson Reuters surface: 750+ sanctions and restricted-party lists.
15. Thomson Reuters surface: expert-maintained policy changes.
16. Thomson Reuters surface: regulation and enforcement-guidance changes.
17. Thomson Reuters surface: updates within 24 hours.
18. Thomson Reuters surface: 150 dedicated researchers.
19. Thomson Reuters surface: monitoring over 1,300 global sources.
20. Thomson Reuters surface: 130 million regulatory updates annually.
21. Thomson Reuters surface: content extracts and APIs.
22. Thomson Reuters surface: import/export compliance automation.
23. Thomson Reuters surface: classification search and discovery.
24. Thomson Reuters surface: denied-party onboarding and administration.
25. Thomson Reuters surface: escalation settings for screening results.
26. Thomson Reuters surface: classification, origin, FTA, entry verification, import, export, DPS, content, analyzer, ABI, and knowledge-network navigation surfaces.
27. Local coverage: `IP-016` covers HS code classification with FTA preference attach.
28. Local coverage: `IP-017` covers denied-party screening lookup with Cedar consent.
29. Local coverage: `IP-020` covers duty-drawback claim workflow.
30. Local coverage: `IP-023` covers preferential-trade agreement origin determination logic.
31. Local coverage: `contracts/openapi-v1.yaml:35-53` includes sanctions-screening mutation.
32. Local coverage: `contracts/openapi-v1.yaml:54-72` includes export-control-classification mutation.
33. Local coverage: `contracts/openapi-v1.yaml:16-34` includes customs-declaration mutation.
34. Local gap: ONESOURCE regulatory-content breadth is not modeled in service contracts.
35. Local gap: 24-hour content-update workflow is not modeled as an SLO.
36. Local gap: FTA count and source-monitoring lineage are absent.
37. Local gap: ABI/entry-verification surfaces are not in core contracts.
38. Local gap: content extracts and APIs are not separated from command APIs.
39. Local gap: denied-party administration is not first-class beyond screening mutation.
40. Thomson Reuters parity verdict: planned coverage exists for core decisions, but content intelligence and lifecycle breadth are under-specified.

## Counterpart 3 - Descartes Capability Surface

1. Descartes surface: global trade intelligence.
2. Descartes surface: import/export data and trade research.
3. Descartes surface: duty and tariff data solutions.
4. Descartes surface: export compliance solutions.
5. Descartes surface: export license management.
6. Descartes surface: global trade data analytics.
7. Descartes surface: product classification.
8. Descartes surface: duty determination.
9. Descartes surface: denied-party screening tools.
10. Descartes surface: foreign trade zone management.
11. Descartes surface: trade compliance content.
12. Descartes surface: AI-assisted sanctions screening.
13. Descartes surface: screening accuracy and false-positive reduction.
14. Descartes surface: global trade content accessibility.
15. Descartes surface: 180+ country coverage.
16. Descartes surface: over 6 million regulatory sources.
17. Descartes surface: hundreds of sanctions and restricted-party lists.
18. Descartes surface: daily rescreening.
19. Descartes surface: bulk screening.
20. Descartes surface: HTS classification.
21. Descartes surface: automated denied-party list data feeds.
22. Descartes surface: SAP GTS environment feed integration.
23. Descartes surface: organization-wide rollout.
24. Descartes surface: centralized compliance activities.
25. Descartes surface: customer labor-cost savings.
26. Descartes surface: manual screening-time reduction.
27. Local coverage: `PRD.md:86-100` defines sanctions screening.
28. Local coverage: `PRD.md:102-116` defines export-control classification.
29. Local coverage: `IP-016` covers HS classification.
30. Local coverage: `IP-017` covers denied-party lookup.
31. Local coverage: `IP-021` covers quota management with country-of-origin tracking.
32. Local coverage: `IP-022` covers embargo event audit-chain anchoring.
33. Local coverage: `capacity-model.md:24-25` defines partition keys.
34. Local gap: FTZ management is absent.
35. Local gap: AI sanctions-screening false-positive reduction is not specified.
36. Local gap: global trade intelligence and trade-research datasets are not modeled.
37. Local gap: source-count and country-count content assertions are absent.
38. Local gap: daily rescreening is not contractually explicit.
39. Local gap: bulk screening is not contractually explicit.
40. Descartes parity verdict: partial compliance-workflow coverage, weak global-trade intelligence coverage.

## Union-Coverage Matrix

| # | Capability family | SAP GTS | Thomson Reuters ONESOURCE | Descartes | Current Oyatie evidence | Coverage |
|---|---|---|---|---|---|
| 1 | Customs declaration command | strong | strong | moderate | `PRD.md:70-84`; `contracts/openapi-v1.yaml:16-34` | partial |
| 2 | Customs declaration eventing | strong | strong | moderate | `contracts/asyncapi-v1.yaml:8-12` | partial |
| 3 | Customs document audit evidence | strong | strong | moderate | `ADR-GT-001:59-91`; `contracts/asyncapi-v1.yaml:40-58` | partial |
| 4 | Import management | strong | strong | strong | `PRD.md:70-84`; `IP-019`; `IP-021` | partial |
| 5 | Export management | strong | strong | strong | `PRD.md:102-116`; `contracts/openapi-v1.yaml:54-72` | partial |
| 6 | Sanctioned-party screening | strong | strong | strong | `PRD.md:86-100`; `contracts/openapi-v1.yaml:35-53` | partial |
| 7 | Denied-party screening | strong | strong | strong | `PRD.md:134-148`; `IP-017` | partial |
| 8 | Denied-party adjudication worklist | strong | strong | strong | `PRD.md:134-148`; missing ADR-named runbook | gap |
| 9 | Continuous rescreening | moderate | strong | strong | no explicit contract | gap |
| 10 | Bulk screening | moderate | strong | strong | no explicit contract | gap |
| 11 | Screening administration | moderate | strong | moderate | no explicit admin contract | gap |
| 12 | Screening escalation settings | strong | strong | moderate | Cedar policies only | partial |
| 13 | Screening false-positive reduction | moderate | moderate | strong | no model or target | gap |
| 14 | Embargo checks | strong | strong | strong | `IP-022`; `ADR-GT-001:59-91` | partial |
| 15 | Export-control classification | strong | strong | strong | `PRD.md:102-116`; `contracts/openapi-v1.yaml:54-72` | partial |
| 16 | ECCN content | moderate | strong | strong | no content-source contract | gap |
| 17 | Munitions code content | moderate | strong | moderate | no content-source contract | gap |
| 18 | Restricted-items rules | strong | strong | strong | no content-source contract | gap |
| 19 | License management | strong | strong | strong | `PRD.md:102-116`; no license endpoint | partial |
| 20 | License evidence chain | strong | strong | moderate | `ADR-GT-001:59-91`; missing specific fields | partial |
| 21 | HS classification | strong | strong | strong | `IP-016`; no public contract | partial |
| 22 | HTS classification | strong | strong | strong | `IP-016`; no public contract | partial |
| 23 | Product and tariff classification | strong | strong | strong | `PRD.md:102-116`; `IP-016` | partial |
| 24 | Duty-rate lookup | moderate | strong | strong | `IP-016`; no content contract | partial |
| 25 | Duty determination | moderate | strong | strong | `IP-020`; no public contract | partial |
| 26 | Duty optimization | moderate | strong | strong | `IP-020`; no target model | partial |
| 27 | Duty drawback workflow | moderate | moderate | moderate | `IP-020` | partial |
| 28 | Free-trade agreement coverage | strong | strong | moderate | `IP-016`; `IP-023` | partial |
| 29 | Preferential origin determination | strong | strong | moderate | `IP-023` | partial |
| 30 | Country-of-origin tracking | strong | strong | strong | `IP-021` | partial |
| 31 | Quota management | strong | moderate | moderate | `IP-021` | partial |
| 32 | Customs broker filing | strong | moderate | strong | `PRD.md:150-164`; `contracts/openapi-v1.yaml:111-129` | partial |
| 33 | Broker communication | strong | moderate | strong | `PRD.md:150-164`; missing adapter detail | partial |
| 34 | Self-filing | strong | moderate | moderate | no dedicated endpoint | gap |
| 35 | CUSDEC ingestion | moderate | moderate | strong | `IP-019`; no public contract | partial |
| 36 | ABI integration | moderate | strong | moderate | no explicit contract | gap |
| 37 | AES integration | moderate | strong | moderate | no explicit contract | gap |
| 38 | EMCS support | strong | moderate | moderate | no explicit contract | gap |
| 39 | Intrastat support | strong | moderate | moderate | no explicit contract | gap |
| 40 | FTZ management | strong | moderate | strong | no explicit contract | gap |
| 41 | Bonded warehousing | strong | moderate | moderate | no explicit contract | gap |
| 42 | Inward processing | strong | moderate | moderate | no explicit contract | gap |
| 43 | Processing trade in China | strong | moderate | moderate | no explicit contract | gap |
| 44 | Trade document generation | strong | strong | moderate | `PRD.md:118-132`; `contracts/openapi-v1.yaml:73-91` | partial |
| 45 | Certificate of origin | moderate | strong | moderate | `IP-018`; no public contract | partial |
| 46 | Compliance certificate generation | moderate | strong | moderate | `IP-018`; no public contract | partial |
| 47 | Import/export document archival | strong | moderate | moderate | audit events only | partial |
| 48 | Regulatory-content repository | strong | strong | strong | not modeled as content repository | gap |
| 49 | Country/territory content breadth | moderate | strong | strong | not specified | gap |
| 50 | Sanctions list breadth | strong | strong | strong | not specified | gap |
| 51 | Regulatory-source monitoring | moderate | strong | strong | not specified | gap |
| 52 | Content update latency | moderate | strong | strong | no SLO | gap |
| 53 | Content extracts | moderate | strong | moderate | no extract API | gap |
| 54 | Content APIs | moderate | strong | strong | command APIs only | partial |
| 55 | Trade data analytics | strong | moderate | strong | dashboards only | partial |
| 56 | Global trade intelligence | moderate | moderate | strong | no dataset model | gap |
| 57 | Supplier/buyer research | low | moderate | strong | no product surface | gap |
| 58 | Competitive trade intelligence | low | moderate | strong | no product surface | gap |
| 59 | Government customs connectivity | strong | strong | strong | no per-government adapter model | gap |
| 60 | OGA/PGA requirements | moderate | strong | moderate | not specified | gap |
| 61 | Restricted party list data feeds | strong | strong | strong | no feed contract | gap |
| 62 | SAP GTS feed integration | strong | moderate | strong | no integration contract | gap |
| 63 | ERP/logistics integration | strong | strong | moderate | dependency docs, no handoff doc | partial |
| 64 | Order/shipment process integration | strong | moderate | moderate | architecture integration only | partial |
| 65 | Marketplace settlement reference | low | low | low | `contracts/openapi-v1.yaml:146-147` and peers | differentiator |
| 66 | Tenant scoping | moderate | moderate | moderate | `manifest.json:54-58`; contract required fields | strong |
| 67 | Cedar default-deny authorization | low | low | low | `PRD.md:36`; policy files | differentiator |
| 68 | Audit-chain events | strong | strong | moderate | `contracts/asyncapi-v1.yaml:40-153` | partial |
| 69 | Trade hold state machine | strong | moderate | moderate | `ADR-GT-001:59-91` | strong |
| 70 | Hold release policy | strong | moderate | moderate | ADR references missing policy | partial |
| 71 | Worklists | strong | strong | moderate | no explicit worklist API | gap |
| 72 | Escalation workflows | strong | strong | moderate | workflow handoff only | partial |
| 73 | Compliance pack overlays | moderate | moderate | moderate | `manifest.json:59-67` | partial |
| 74 | BYOK handling | moderate | moderate | moderate | no service-specific contract | gap |
| 75 | Tenant-class behavior | low | low | low | absent | gap |
| 76 | Demo/trial infrastructure caps | low | low | low | absent | gap |
| 77 | Paid contractual SLO semantics | moderate | moderate | moderate | absent | gap |
| 78 | Revenue-share settlement semantics | low | low | low | marketplace ref only | partial |
| 79 | Multi-context deployment | moderate | moderate | moderate | no per-context modules | gap |
| 80 | OpenTofu modules | low | low | low | `iac/terraform-module/main.tf:1-7` only | gap |
| 81 | OCI Always Free profile | low | low | low | absent | gap |
| 82 | OS support matrix | low | low | low | absent | gap |
| 83 | Rust implementation | low | low | low | no `src/` files | gap |
| 84 | Regression tests | low | low | low | no `tests/` files | gap |
| 85 | Availability SLO | strong | strong | strong | `slos/global-trade-availability.openslo.yaml:27-29` | partial |
| 86 | Latency SLO | strong | strong | strong | `slos/global-trade-latency-p99.openslo.yaml:21-29` | partial |
| 87 | Throughput SLO | strong | strong | strong | `slos/global-trade-throughput.openslo.yaml:21-29` | partial |
| 88 | Customs success SLO | strong | strong | moderate | `slos/customs-declaration-success-rate.openslo.yaml:27-29` | partial |
| 89 | Screening p95/p99 SLO | strong | strong | strong | `ADR-GT-001:209-210`; not OpenSLO | partial |
| 90 | Broker callback SLO | strong | moderate | strong | `ADR-GT-001:212`; not OpenSLO | partial |
| 91 | Load-test acceptance | strong | strong | strong | `ADR-GT-001:259`; no test files | partial |
| 92 | Incident response | strong | strong | strong | `incident-response.md`; runbooks present | partial |
| 93 | Denied-party adjudication runbook | strong | strong | strong | ADR references missing named runbook | gap |
| 94 | Broker filing retry runbook | strong | moderate | strong | ADR references missing named runbook | gap |
| 95 | Data residency | strong | strong | strong | `policy/data-residency.md`; no context modules | partial |
| 96 | Tenant isolation | strong | strong | strong | `policy/tenant-isolation.md` | partial |
| 97 | Abuse defense | moderate | moderate | moderate | `policy/abuse-defence.cedar` | partial |
| 98 | Dashboarding | strong | strong | strong | three dashboard files | partial |
| 99 | Backfill/replay | moderate | strong | strong | `backfill-replay.md` | partial |
| 100 | Migration playbooks | strong | strong | strong | missing directory | gap |
| 101 | Onboarding docs | strong | strong | strong | missing directory | gap |
| 102 | FAQ docs | moderate | strong | moderate | missing directory | gap |
| 103 | Tutorials | moderate | strong | moderate | missing directory | gap |
| 104 | Reference implementations | moderate | strong | strong | missing directory | gap |
| 105 | SDK plan | moderate | strong | moderate | `sdk-plan.md` | partial |
| 106 | API versioning | strong | strong | strong | `contracts/openapi-v1.yaml:1-5` | partial |
| 107 | Event versioning | strong | strong | strong | `contracts/asyncapi-v1.yaml:1-6` | partial |
| 108 | Proto versioning | strong | moderate | moderate | `contracts/global-trade-v1.proto:1-4` | partial |
| 109 | Marketplace reseller use case | low | low | low | settlement ref only | partial |
| 110 | At-cost substrate accounting | low | low | low | absent | gap |

## Family Summary

1. Customs family status: original command/event/RPC skeleton exists.
2. Customs family gap: special customs procedures are absent.
3. Customs family gap: FTZ, bonded warehousing, Intrastat, EMCS, ABI, AES, and government-system adapter detail are absent.
4. Screening family status: sanctions screening and denied-party-hit skeletons exist.
5. Screening family gap: continuous rescreening, bulk screening, false-positive reduction, and screening administration are absent.
6. Export-control family status: export-control-classification skeleton exists.
7. Export-control family gap: ECCN, munitions-code, restricted-item, export-license, and content-source lineage are not modeled.
8. Classification family status: HS classification is planned in IP-016.
9. Classification family gap: HS/HTS/ECCN contract fields are not first-class in OpenAPI, AsyncAPI, or proto.
10. Trade-document family status: trade-document skeleton exists.
11. Trade-document family gap: certificates, country-of-origin, and compliance-document generation are only planned in IP-018.
12. Duty family status: duty drawback appears in IP-020.
13. Duty family gap: duty optimization, landed-cost strategy, and tariff savings targets are not in canonical contracts.
14. FTA/origin family status: IP-016 and IP-023 cover preference attach and origin determination.
15. FTA/origin family gap: 500+ FTA breadth, source freshness, and content update SLOs are absent.
16. Quota family status: IP-021 covers quota and country-of-origin tracking.
17. Quota family gap: quota exhaustion alerts and broker/government filing effects are not in contracts.
18. Embargo family status: IP-022 and ADR hold-state logic cover embargo evidence.
19. Embargo family gap: embargo data-source freshness and audit-anchor read APIs are not in contracts.
20. Broker family status: broker filing endpoint and IP-019 exist.
21. Broker family gap: adapter protocols and retry runbook are incomplete.
22. Intelligence family status: dashboards and capacity docs exist.
23. Intelligence family gap: Descartes-style trade intelligence and source analytics are not modeled.
24. Content family status: compliance packs exist.
25. Content family gap: regulatory-content repository, extraction APIs, source monitoring, and source lineage are absent.
26. Deployment family status: flat Kubernetes and security IaC files exist.
27. Deployment family gap: six-context OpenTofu modules are absent.
28. Tenant family status: tenant_id is a required contract field.
29. Tenant family gap: tenant_class is absent.
30. Commercial family status: marketplace settlement refs exist.
31. Commercial family gap: revenue-share semantics and at-cost substrate accounting are absent.
32. Quality family status: SLO docs exist.
33. Quality family gap: SLO docs do not cover all ADR performance dimensions.
34. Implementation family status: contracts are concrete.
35. Implementation family gap: Rust code and tests are absent.

## Headline Gap Analysis

1. Gap H1: The service is currently SAP-led, not union-led.
2. Evidence H1: `PRD.md:24-32` and `README.md:11-14` emphasize SAP GTS.
3. Evidence H1: `competitor-parity-matrix.md:13-20` uses SAP, Oracle, Workday, NetSuite, and Microsoft.
4. Impact H1: Thomson Reuters and Descartes requirements are underweighted.
5. Remediation H1: recast the authoritative matrix around SAP, Thomson Reuters, and Descartes.
6. Gap H2: Content intelligence is not first-class.
7. Evidence H2: no contract fields for regulatory source, update timestamp, source jurisdiction, or list corpus.
8. Impact H2: Thomson Reuters and Descartes parity cannot be proven.
9. Remediation H2: add content-source, classification-source, and screening-source contract models.
10. Gap H3: Continuous and bulk screening are missing.
11. Evidence H3: `contracts/openapi-v1.yaml:35-53` has a simple sanctions-screening mutation only.
12. Impact H3: both ONESOURCE and Descartes screening expectations remain incomplete.
13. Remediation H3: add single-screen, batch-screen, rescreen, and adjudication contracts.
14. Gap H4: Customs special procedures are missing.
15. Evidence H4: no current endpoint covers FTZ, bonded warehousing, Intrastat, EMCS, or inward processing.
16. Impact H4: SAP GTS customs depth is not matched.
17. Remediation H4: split special customs procedures into explicit domain models or mark out-of-scope with handoff.
18. Gap H5: Government/broker adapter detail is missing.
19. Evidence H5: `PRD.md:150-164` describes broker filing, but no adapter roster exists.
20. Impact H5: self-filing, CUSDEC, ABI, AES, and customs connectivity are not proven.
21. Remediation H5: add broker/gateway adapter contracts and retry runbooks.
22. Gap H6: Tenant-class semantics are absent.
23. Evidence H6: no `tenant_class`, `demo_trial`, or `revenue_share` hits in the service path.
24. Impact H6: the service cannot replace old tier semantics cleanly.
25. Remediation H6: add tenant-class field semantics to contracts, metrics, cost, and IaC overlays.
26. Gap H7: Old tier-model references remain.
27. Evidence H7: 509 generic old tier-model hits across service docs.
28. Impact H7: service docs still imply feature stratification.
29. Remediation H7: Wave 15J retirement pass.
30. Gap H8: Six-context deployment evidence is absent.
31. Evidence H8: no per-context `iac/` directories and no OCI Always Free profile.
32. Impact H8: deployable-context parity is not proven.
33. Remediation H8: add OpenTofu context modules and admission evidence.
34. Gap H9: Rust implementation evidence is absent.
35. Evidence H9: empty `src/` and `tests/`.
36. Impact H9: feature parity is currently documentary, not executable.
37. Remediation H9: add Rust service implementation and regression tests.
38. Gap H10: Business performance SLOs lag ADR detail.
39. Evidence H10: OpenSLO files cover aggregate availability, p99, throughput, and customs success only.
40. Impact H10: screening, classification, callback, and event-emission targets are not machine-verifiable.
41. Remediation H10: add business-specific OpenSLOs aligned to ADR-GT-001.
42. Gap H11: adoption docs are absent.
43. Evidence H11: no onboarding, tutorials, FAQs, migration playbooks, or reference implementations.
44. Impact H11: counterpart-level enterprise adoption readiness is not demonstrated.
45. Remediation H11: add adoption docs for customs, screening, classification, and broker flows.
46. Gap H12: source freshness is unmodeled.
47. Evidence H12: no source-update SLO or source lineage model.
48. Impact H12: ONESOURCE and Descartes content-leadership parity is weak.
49. Remediation H12: add `regulatory_source`, `source_version`, `effective_at`, and `last_verified_at` fields.
50. Gap H13: Descartes-style global trade intelligence is absent.
51. Evidence H13: dashboards are operational, not trade-data intelligence products.
52. Impact H13: supplier/buyer research and trade-data analytics are outside current scope without handoff.
53. Remediation H13: either add intelligence surfaces or explicitly hand off to a trade-data service.
54. Gap H14: current docs over-index on generated row volume.
55. Evidence H14: `competitor-parity-matrix.md:24-220` and `capacity-model.md:26-220` repeat patterned rows.
56. Impact H14: line count hides capability gaps.
57. Remediation H14: replace generated rows with evidence-bound capability families.
58. Gap H15: previous audit closure is too narrow.
59. Evidence H15: `AUDIT-FINDINGS-2026-05-21.json:10-62` closes six doc-suite rows only.
60. Impact H15: canonical cross-cutting gaps remain open.

## Additive Surface For Industry-Leader Coverage

1. Add `TradeContentSource` with source jurisdiction, source family, source version, effective time, and evidence hash.
2. Add `ScreeningCorpus` with sanctions-list family, restricted-party-list family, update timestamp, and researcher/source lineage.
3. Add `ScreeningRequest` for single party screening.
4. Add `BatchScreeningRequest` for bulk screening.
5. Add `ContinuousRescreeningSubscription` for rescreening active entities.
6. Add `ScreeningHitAdjudication` with reviewer, reason, confidence, escalation, and release/hold decision.
7. Add `FalsePositiveControl` with match rationale, resolver notes, and audit replay.
8. Add `ClassificationRequest` with HS, HTS, ECCN, munitions-code, and jurisdiction fields.
9. Add `ClassificationSourceEvidence` with rule source, country, effective date, and source hash.
10. Add `FTAEligibilityRequest` with origin, bill-of-materials evidence, supplier declaration, and agreement code.
11. Add `OriginDetermination` as a first-class outcome.
12. Add `QuotaReservation` with origin, jurisdiction, quota program, reservation amount, and expiry.
13. Add `EmbargoCheck` with jurisdiction, party, destination, product, and route evidence.
14. Add `ExportLicenseEvidence` with license id, agency, jurisdiction, expiry, and covered products.
15. Add `CustomsSpecialProcedure` for FTZ, bonded warehousing, inward processing, processing trade, Intrastat, and EMCS.
16. Add `GovernmentFilingAdapter` with protocol, jurisdiction, submission id, retry policy, and callback schema.
17. Add `BrokerFilingAdapter` with CUSDEC, ABI, AES, EMCS, and manual-upload capability flags.
18. Add `DutyDetermination` with tariff code, valuation basis, duty amount, and source rules.
19. Add `DutyDrawbackClaim` public contract aligned with IP-020.
20. Add `TradeDocumentCertificate` for certificate of origin and compliance certificates.
21. Add `RegulatoryUpdateEvent` with source-count, affected rules, and customer impact.
22. Add `ContentFreshnessSLO` for update propagation.
23. Add `DeniedPartyAdjudicationRunbook` referenced by ADR-GT-001.
24. Add `BrokerFilingRetryRunbook` referenced by ADR-GT-001.
25. Add tenant-class overlay fields without feature degradation.
26. Add `tenant_class` metrics dimension to replace old generic feature-tier dimensions.
27. Add `commercial_arrangement` or equivalent field only where settlement model matters.
28. Add `revenue_share_settlement_ref` where marketplace sellers or embedded resellers need gross-revenue evidence.
29. Add demo/trial usage cap policy tied to OCI Always Free profile budgets.
30. Add paid contractual SLO policy tied to deployment context and compliance pack.
31. Add revenue-share at-cost substrate accounting policy.
32. Add per-context OpenTofu module references to every deployable context claim.
33. Add `supported-oses.json` and map tests to the OS matrix.
34. Add Rust crate surfaces for domain, usecase, adapter, rest, worker, and test fixtures.
35. Add regression tests for single screening, batch screening, denied-party adjudication, export hold, broker callback, and duty drawback.
36. Add public API examples for customs declaration, sanctions screening, classification, and broker filing.
37. Add reference implementation for a marketplace seller onboarding path.
38. Add reference implementation for a manufacturer/exporter classification path.
39. Add reference implementation for a broker filing callback path.
40. Add migration playbook from SAP GTS data exports.
41. Add migration playbook from ONESOURCE classification and screening data.
42. Add migration playbook from Descartes Visual Compliance and CustomsInfo data.
43. Add onboarding doc for demo/trial tenants with usage caps.
44. Add onboarding doc for paid tenants with contractual SLOs.
45. Add onboarding doc for revenue-share tenants with settlement evidence.
46. Add FAQ for customs and broker filing ownership boundaries.
47. Add FAQ for compliance pack, BYOK, and tenant-class semantics.
48. Add tutorial for continuous rescreening.
49. Add tutorial for FTA preference origin determination.
50. Add tutorial for duty drawback claim workflow.
51. Add tutorial for government filing adapter setup.
52. Add benchmark workloads for screening p50/p95/p99, classification p50/p95/p99, callback p50/p95/p99, and event-emission p99.
53. Add benchmark overlays for six deployment contexts.
54. Add benchmark overlays for tenant classes.
55. Add content freshness benchmark for regulatory updates.
56. Add evidence that Descartes-style trade intelligence is either in scope or delegated.
57. Add explicit handoff if supplier/buyer trade research belongs outside `global-trade`.
58. Add explicit handoff if government tariff datasets are owned by `regional-pack` or `compliance`.
59. Add explicit handoff if billing and revenue-share settlement are owned by marketplace.
60. Add one authoritative union-coverage matrix as the replacement for the stale competitor-parity roster.

## Parity Verdict

1. SAP parity status: partial.
2. SAP strongest local evidence: original six bounded contexts, TradeHold ADR, customs/sanctions/export/broker skeletons.
3. SAP strongest local gap: special customs procedures, government-system adapters, FTZ, Intrastat, EMCS, and HANA-equivalent analytics.
4. Thomson Reuters parity status: partial-to-gap.
5. Thomson Reuters strongest local evidence: IP-016, IP-017, IP-020, and IP-023 planning surfaces.
6. Thomson Reuters strongest local gap: regulatory content breadth, source freshness, FTA scale, denied-party admin, and content APIs.
7. Descartes parity status: partial-to-gap.
8. Descartes strongest local evidence: screening, classification planning, duty and origin planning, and audit-chain events.
9. Descartes strongest local gap: global trade intelligence, FTZ, AI screening accuracy, daily rescreening, bulk screening, and source-count evidence.
10. Overall union verdict: current artifacts prove a credible starting concept, not industry-leader parity.
11. Remediation priority one: update authoritative counterpart roster.
12. Remediation priority two: add missing contract surfaces for content, screening, classification, broker adapters, and special customs procedures.
13. Remediation priority three: add six-context OpenTofu and tenant-class overlays.
14. Remediation priority four: add Rust implementation and tests.
15. Remediation priority five: replace generated parity rows with evidence-bound capability families.
