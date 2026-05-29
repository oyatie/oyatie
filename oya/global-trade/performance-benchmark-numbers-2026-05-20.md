# Global Trade Performance Benchmark Numbers - 2026-05-20

Target service: `global-trade`.
Target path: `microservices/global-trade/`.
Counterpart set: SAP Global Trade Services / Thomson Reuters ONESOURCE Global Trade / Descartes.
Report shape: single industry-leader target set with deployment-context and tenant-class overlays.
Excluded shape: old feature-tier rows or headings.
Methodology warning: the three counterpart vendors publish capability surfaces and some business-result numbers, but they do not publish directly comparable public API p50/p95/p99/rps benchmarks for the exact workloads below.
Methodology consequence: counterpart API latency, throughput, and concurrency figures are labeled as estimates.
Methodology consequence: public non-API numbers are labeled as public-source figures.
Local SLO anchor: `ADR-GT-001:207-214` gives availability, release-decision read, sanctions screening, export classification, broker callback, event emission, and recovery-point objectives.
Local load anchor: `ADR-GT-001:259` gives a load-test target of 200 screening rps plus 100 broker callback rps for 30 minutes.
Local OpenSLO anchor: `slos/global-trade-availability.openslo.yaml:27-29` sets availability target 0.999.
Local OpenSLO anchor: `slos/global-trade-latency-p99.openslo.yaml:21-29` sets a 0.35-second p99 bucket with 0.99 objective.
Local OpenSLO anchor: `slos/global-trade-throughput.openslo.yaml:21-29` sets accepted/received throughput target 0.995.
Local capacity anchor: `capacity-model.md:24-25` defines partition key and replay lease key shape.
Canonical context anchor: `specs/master-plan-sequencing.json:704-745` defines the six deployment contexts.
Canonical IaC anchor: `specs/master-plan-sequencing.json:747-775` binds deployment to OpenTofu.
Canonical OCI anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514-3571` defines the OCI Always Free profile budget envelope.
Canonical tenant-class anchor: current batch directive requires `demo_trial`, `paid`, and `revenue_share` overlays without feature stratification.
Public SAP source: SAP Global Trade Services page describes import/export management, optimized tariff classification, free-trade agreements, customs procedures, self-filing, broker communication, sanctioned-party screening, customs management, real-time checks, and special customs procedures.
Public Thomson Reuters source: ONESOURCE Global Trade page states 220+ countries/territories, 500+ free-trade agreements, 750+ sanctions/restricted-party lists, 150 researchers, 1,300 sources, 130 million regulatory updates annually, and changes within 24 hours.
Public Descartes source: Descartes Global Trade Intelligence page states 180+ countries, over 6 million regulatory sources, AI sanctions-screening false-positive reduction up to 60 percent, global trade data visibility by 95 percent, average duty/tariff savings by 30 percent, and manual screening-time reduction by 75 percent.

## 1. Methodology

1. Benchmark dimension: release-decision read latency.
2. Benchmark dimension: sanctioned-party or denied-party single-screen latency.
3. Benchmark dimension: 100-party batch-screen latency.
4. Benchmark dimension: continuous rescreen subscription update latency.
5. Benchmark dimension: export-control classification latency per line.
6. Benchmark dimension: HS/HTS/ECCN classification latency per line.
7. Benchmark dimension: customs declaration command acceptance latency.
8. Benchmark dimension: broker filing callback ingestion latency.
9. Benchmark dimension: trade document generation latency.
10. Benchmark dimension: certificate-of-origin generation latency.
11. Benchmark dimension: duty-drawback claim state transition latency.
12. Benchmark dimension: quota/origin decision latency.
13. Benchmark dimension: embargo audit-chain anchor latency.
14. Benchmark dimension: event-emission end-to-end latency.
15. Benchmark dimension: regulatory-content update propagation latency.
16. Benchmark dimension: audit replay recovery point.
17. Benchmark dimension: sustained requests per second.
18. Benchmark dimension: burst requests per second.
19. Benchmark dimension: concurrent in-flight operations.
20. Benchmark dimension: concurrent tenants per cell.
21. Benchmark dimension: availability.
22. Benchmark dimension: success ratio.
23. Test workload: single screening request with one party, one jurisdiction, and one list family.
24. Test workload: batch screening request with 100 parties, three jurisdictions, and five list families.
25. Test workload: continuous rescreening update with 10,000 active screened entities.
26. Test workload: customs declaration command with 100 lines.
27. Test workload: export classification for 500 declaration lines.
28. Test workload: HS/HTS/ECCN classification for 500 product lines.
29. Test workload: broker callback ingest with signed external payload.
30. Test workload: duty drawback claim transition with paid-duty evidence attached.
31. Test workload: FTA origin determination with bill-of-materials evidence.
32. Test workload: event emission through AsyncAPI channel.
33. Test workload: audit replay from latest sealed event.
34. OS disclosure: no service-local `supported-oses.json` exists, so OS-specific performance claims are not yet proven.
35. Architecture disclosure: Rust implementation files are absent, so Oyatie numbers below are target numbers, not measured service results.
36. IaC disclosure: per-context OpenTofu modules are absent, so context overlays are canonical targets and constraints, not measured deployments.
37. Deployment context disclosure: all six canonical contexts are included because the user task assumes all six unless audit finds otherwise.
38. Tenant-class disclosure: `demo_trial`, `paid`, and `revenue_share` are commercial/infrastructure overlays, not feature-quality bands.
39. `demo_trial` disclosure: demo/trial tenants use the OCI Always Free profile where infrastructure is constrained.
40. `paid` disclosure: paid tenants scale with contractual SLO, compliance pack, BYOK, and paid substrate.
41. `revenue_share` disclosure: revenue-share tenants run at-cost or zero-margin substrate and carry gross-revenue settlement evidence.
42. Public-source standard: cite vendor pages for published capability or business-result numbers.
43. Estimate standard: label API p50/p95/p99/rps numbers as estimates when vendors do not publish comparable API figures.
44. Target standard: Oyatie targets meet or beat the strongest counterpart estimate where infrastructure context does not constrain throughput.
45. Overlay standard: OCI Always Free profile can cap throughput and concurrency while preserving feature quality.
46. Overlay standard: on-prem and colo deployments can be facility-constrained while preserving the service target contract once provisioned.
47. Overlay standard: oyatie-public-cloud and oyatie-as-cloud-provider carry elasticity assumptions once OpenTofu modules exist.
48. Verification standard: current repo can verify documentation evidence only; execution benchmarks require future Rust implementation and tests.
49. Non-use standard: this document does not use old feature-tier rows.
50. Non-use standard: this document does not author a tenant-class delta report.

## 2. Counterpart Numbers

### 2.1 SAP Global Trade Services

1. SAP public figure: pricing metric is user-based on SAP page; source: SAP public product/pricing page.
2. SAP public capability number: sanctioned-party screening spans sales, finance, HR, procurement, and distribution; source: SAP public product page.
3. SAP public capability number: special customs procedures include at least FTZ, China processing trade, bonded warehousing, inward processing, Intrastat, and EMCS; source: SAP public product page.
4. SAP estimated API target: real-time compliance check p50 250 ms, p95 1.5 s, p99 5 s; source: estimated from SAP real-time compliance surface and enterprise workflow expectations.
5. SAP estimated API target: single sanctioned-party screen p50 300 ms, p95 2 s, p99 10 s; source: estimated from sanctioned-party screening workflow class.
6. SAP estimated API target: 100-party batch screening p50 3 s, p95 12 s, p99 45 s; source: estimated from batch compliance workload class.
7. SAP estimated API target: customs declaration command accept p50 150 ms, p95 700 ms, p99 2 s; source: estimated from import/export management command flow.
8. SAP estimated API target: broker communication callback ingest p50 250 ms, p95 1.5 s, p99 6 s; source: estimated from broker communication surface.
9. SAP estimated API target: export-license decision read p50 100 ms, p95 750 ms, p99 3 s; source: estimated from export legal-control workflow.
10. SAP estimated API target: product/tariff classification per line p50 150 ms, p95 750 ms, p99 2.5 s; source: estimated from classification workflow.
11. SAP estimated throughput: sustained mixed trade-compliance commands 150 rps per enterprise cell; source: estimated from enterprise private-edition deployment shape.
12. SAP estimated throughput: burst mixed trade-compliance commands 500 rps per enterprise cell for 5 minutes; source: estimated from enterprise private-edition deployment shape.
13. SAP estimated availability: 99.9 percent or better for enterprise deployment; source: estimated from enterprise SaaS/private-edition expectation, not public API benchmark.
14. SAP estimated recovery point: less than 5 minutes for audit replay-ready transaction evidence; source: estimated from archived compliance documentation requirements.
15. SAP evidence-quality note: SAP publishes capability breadth, not directly comparable public API benchmark tables.

### 2.2 Thomson Reuters ONESOURCE Global Trade

1. Thomson Reuters public figure: HS codes and duty rates for 220+ countries and territories; source: public ONESOURCE Global Trade page.
2. Thomson Reuters public figure: detailed requirements for 500+ free-trade agreements; source: public ONESOURCE Global Trade page.
3. Thomson Reuters public figure: 750+ global sanctions and restricted-party lists; source: public ONESOURCE Global Trade page.
4. Thomson Reuters public figure: changes provided within 24 hours; source: public ONESOURCE Global Trade page.
5. Thomson Reuters public figure: 150 dedicated researchers; source: public ONESOURCE Global Trade page.
6. Thomson Reuters public figure: over 1,300 global sources monitored; source: public ONESOURCE Global Trade page.
7. Thomson Reuters public figure: 130 million regulatory updates delivered annually; source: public ONESOURCE Global Trade page.
8. Thomson Reuters estimated API target: product classification p50 200 ms, p95 1.2 s, p99 4 s per line; source: estimated from product classification workflow.
9. Thomson Reuters estimated API target: denied-party single screen p50 250 ms, p95 1.5 s, p99 7 s; source: estimated from denied-party screening workflow.
10. Thomson Reuters estimated API target: 100-party denied-party batch p50 2.5 s, p95 10 s, p99 35 s; source: estimated from batch screening workload.
11. Thomson Reuters estimated API target: regulatory-content lookup p50 100 ms, p95 500 ms, p99 1.5 s; source: estimated from content API/extract surface.
12. Thomson Reuters estimated API target: content update propagation p95 24 h, p99 30 h; source: public 24-hour change claim plus estimate band.
13. Thomson Reuters estimated throughput: sustained classification and screening 200 rps per enterprise cell; source: estimated from content-platform workload class.
14. Thomson Reuters estimated availability: 99.9 percent or better; source: estimated enterprise SaaS expectation.
15. Thomson Reuters evidence-quality note: content breadth numbers are public; API latency and rps numbers are estimates.

### 2.3 Descartes

1. Descartes public figure: 180+ countries for global trade content accessibility; source: public Descartes Global Trade Intelligence page.
2. Descartes public figure: over 6 million regulatory sources; source: public Descartes Global Trade Intelligence page.
3. Descartes public figure: AI sanctions-screening false-positive reduction up to 60 percent; source: public Descartes Global Trade Intelligence page.
4. Descartes public figure: global trade data visibility by 95 percent; source: public Descartes Global Trade Intelligence page.
5. Descartes public figure: average duty and tariff savings by 30 percent; source: public Descartes Global Trade Intelligence page.
6. Descartes public figure: manual screening-time reduction by 75 percent; source: public Descartes Global Trade Intelligence page.
7. Descartes public figure: Meggitt case references more than US$300,000 in labor cost savings; source: public Descartes case page.
8. Descartes public capability number: Meggitt case references 60 businesses in 16 countries; source: public Descartes case page.
9. Descartes estimated API target: denied-party single screen p50 200 ms, p95 1 s, p99 5 s; source: estimated from denied-party screening tooling class.
10. Descartes estimated API target: bulk screening 100 parties p50 2 s, p95 8 s, p99 30 s; source: estimated from bulk screening and daily rescreening class.
11. Descartes estimated API target: HTS classification p50 200 ms, p95 1 s, p99 3 s per line; source: estimated from CustomsInfo-style classification workflow.
12. Descartes estimated API target: daily rescreening update p95 24 h, p99 30 h; source: public daily/nightly rescreening descriptions plus estimate band.
13. Descartes estimated throughput: sustained screening/classification 250 rps per enterprise cell; source: estimated from SaaS compliance-data workload class.
14. Descartes estimated availability: 99.9 percent or better; source: estimated enterprise SaaS expectation.
15. Descartes evidence-quality note: business-result and content-scale numbers are public; direct API benchmark numbers are estimates.

## 3. Oyatie Target Numbers - Single Industry-Leader Target Set

### 3.1 Canonical Targets

1. Availability target: 99.95 percent for paid and revenue-share production cells.
2. Availability target source: `ADR-GT-001:207`.
3. Demo/trial availability target: best-effort within OCI Always Free profile, with no feature-quality reduction.
4. Release-decision read latency target: p50 25 ms, p95 75 ms, p99 200 ms.
5. Release-decision source: meets and tightens `ADR-GT-001:208`.
6. Single sanctions/denied-party screen target: p50 150 ms, p95 1 s, p99 5 s.
7. Single screen comparison basis: beats SAP estimate p95 2 s, Thomson estimate p95 1.5 s, and matches/bets Descartes estimate p95 1 s.
8. 100-party batch screen target: p50 1.5 s, p95 8 s, p99 25 s.
9. Batch screen comparison basis: matches Descartes p95 estimate and beats SAP/Thomson estimate bands.
10. Continuous rescreening update target: p95 12 h, p99 24 h.
11. Continuous rescreening comparison basis: beats public/estimated 24-hour vendor update/rescreening rhythm.
12. Export-control classification per-line target: p50 75 ms, p95 350 ms, p99 1.5 s.
13. Export-classification source: beats `ADR-GT-001:211` p95/p99 envelope.
14. HS/HTS/ECCN classification per-line target: p50 100 ms, p95 500 ms, p99 2 s.
15. Classification comparison basis: beats estimated counterpart p95 bands.
16. 500-line declaration classification target: p50 12 s, p95 35 s, p99 90 s.
17. Customs declaration command accept target: p50 75 ms, p95 300 ms, p99 1 s.
18. Broker filing callback ingest target: p50 125 ms, p95 750 ms, p99 3 s.
19. Broker callback source: beats `ADR-GT-001:212` p95/p99 envelope.
20. Trade document generation target: p50 250 ms, p95 1.5 s, p99 5 s for standard documents.
21. Certificate-of-origin generation target: p50 300 ms, p95 2 s, p99 8 s.
22. Duty-drawback claim transition target: p50 100 ms, p95 500 ms, p99 2 s excluding external agency waits.
23. Quota/origin decision target: p50 150 ms, p95 750 ms, p99 3 s.
24. Embargo audit-chain anchor target: p50 100 ms, p95 500 ms, p99 2 s.
25. Event emission target: p50 250 ms, p95 1 s, p99 3 s.
26. Event emission source: matches `ADR-GT-001:213`.
27. Audit replay RPO target: 15 minutes or better.
28. Audit replay source: `ADR-GT-001:214`.
29. Sustained screening throughput target: 500 rps per production cell.
30. Sustained broker callback throughput target: 250 rps per production cell.
31. Load-test acceptance target: at least 200 screening rps plus 100 broker callback rps for 30 minutes.
32. Load-test source: `ADR-GT-001:259`.
33. Burst screening throughput target: 2,000 rps per production cell for 5 minutes with queue backpressure.
34. Mixed trade-compliance command target: 1,000 rps per production cell.
35. Concurrent in-flight operations target: 10,000 per production cell.
36. Concurrent tenant target: 1,000 active tenants per production cell before planned cell split.
37. Regulatory-content lookup target: p50 50 ms, p95 250 ms, p99 1 s.
38. Regulatory-content propagation target: p95 6 h, p99 12 h after trusted-source ingestion.
39. Screening-source freshness target: p95 6 h, p99 12 h after trusted-source ingestion.
40. Classification-source freshness target: p95 6 h, p99 12 h after trusted-source ingestion.
41. Compliance evidence sealing target: p50 50 ms, p95 250 ms, p99 1 s.
42. Metrics emission target: p99 15 s to queryable Prometheus-compatible surface.
43. Trace availability target: p99 60 s to queryable trace backend.
44. Error budget target: 0.05 percent monthly for paid/revenue-share production cells.
45. Success ratio target: customs declaration success 0.999.
46. Success ratio source: `slos/customs-declaration-success-rate.openslo.yaml:27-29`.
47. Throughput quality target: accepted/received ratio 0.995 or better.
48. Throughput quality source: `slos/global-trade-throughput.openslo.yaml:27-29`.
49. Latency quality target: 99 percent of aggregate global-trade requests within 350 ms where the request is not an external agency wait.
50. Latency quality source: `slos/global-trade-latency-p99.openslo.yaml:21-29`.

### 3.2 Deployment-Context Overlay

1. `oyatie-public-cloud` overlay: all canonical targets apply.
2. `oyatie-public-cloud` throughput overlay: production cells scale horizontally; sustained screening target 500 rps per cell and burst 2,000 rps per cell.
3. `oyatie-public-cloud` concurrency overlay: 10,000 in-flight operations per cell, 1,000 active tenants per cell.
4. `guest-on-aws` overlay: all canonical targets apply when customer-provided substrate meets CPU, network, and storage envelopes.
5. `guest-on-aws` throughput overlay: target 400 rps screening per cell until admission proves customer substrate can sustain 500 rps.
6. `guest-on-aws` concurrency overlay: 8,000 in-flight operations per cell baseline.
7. `guest-on-oci` overlay: all canonical targets apply for paid or revenue-share cells with sufficient paid substrate.
8. `guest-on-oci` OCI Always Free profile overlay: demo/trial sustained screening cap 25 rps.
9. `guest-on-oci` OCI Always Free profile overlay: demo/trial broker callback cap 15 rps.
10. `guest-on-oci` OCI Always Free profile overlay: demo/trial mixed command cap 50 rps.
11. `guest-on-oci` OCI Always Free profile overlay: demo/trial concurrent in-flight operation cap 500.
12. `guest-on-oci` OCI Always Free profile overlay: demo/trial p99 latency may be up to 2x canonical target under CPU or bandwidth saturation.
13. `guest-on-oci` OCI Always Free profile overlay: feature quality remains uniform; limits are usage and infrastructure caps.
14. `on-prem` overlay: all canonical targets apply after facility sizing passes admission.
15. `on-prem` throughput overlay: target 300 rps screening per cell before facility-specific scaling evidence.
16. `on-prem` concurrency overlay: 5,000 in-flight operations per cell baseline.
17. `on-prem` latency overlay: external agency and local network latency are customer-facility constraints and must be separated in traces.
18. `colo` overlay: all canonical targets apply after colo admission.
19. `colo` throughput overlay: target 400 rps screening per cell before facility-specific scaling evidence.
20. `colo` concurrency overlay: 8,000 in-flight operations per cell baseline.
21. `oyatie-as-cloud-provider` overlay: all canonical targets apply with provider-owned elasticity.
22. `oyatie-as-cloud-provider` throughput overlay: sustained screening target 750 rps per cell and burst 3,000 rps per cell when provider substrate is provisioned.
23. `oyatie-as-cloud-provider` concurrency overlay: 15,000 in-flight operations per cell baseline.
24. Overlay evidence gap: current service path has no per-context OpenTofu modules, so these are target overlays rather than measured context results.
25. Overlay remediation: add OpenTofu module, benchmark workload, and evidence capture per deployment context.

### 3.3 Tenant-Class Overlay

1. `demo_trial` overlay: feature set remains industry-leader grade.
2. `demo_trial` overlay: usage is capped by time, request count, storage, and OCI Always Free profile budgets.
3. `demo_trial` overlay: best-effort SLO and no compliance-pack or BYOK commitments.
4. `demo_trial` overlay: screening sustained target 25 rps on OCI Always Free profile.
5. `demo_trial` overlay: batch screening maximum 100 parties per batch and 10 batches per minute.
6. `demo_trial` overlay: regulatory-content propagation target p99 24 h because best-effort operations can queue updates.
7. `paid` overlay: all canonical targets apply.
8. `paid` overlay: throughput scales with paid substrate and contractual SLO.
9. `paid` overlay: compliance packs may be enabled.
10. `paid` overlay: BYOK may be enabled where platform key management is available.
11. `paid` overlay: sustained screening baseline 500 rps per production cell.
12. `paid` overlay: burst screening baseline 2,000 rps per production cell.
13. `revenue_share` overlay: feature set remains industry-leader grade.
14. `revenue_share` overlay: substrate runs at-cost or zero-margin according to commercial contract.
15. `revenue_share` overlay: marketplace settlement evidence must record gross-revenue reference.
16. `revenue_share` overlay: sustained screening target 500 rps per production cell unless at-cost substrate budget caps are explicitly set.
17. `revenue_share` overlay: burst target is negotiated by gross-revenue risk and substrate cost.
18. `revenue_share` overlay: revenue events must not slow compliance holds; settlement can reconcile asynchronously.
19. Tenant-class gap: current service path has no `tenant_class` field or policy.
20. Tenant-class remediation: replace old generic tier fields with tenant-class and commercial-arrangement fields.

## 4. Comparison Narrative

1. Release-decision read: Oyatie target p95 75 ms is ahead of estimated counterpart decision-read bands.
2. Release-decision read: target is credible only after Rust implementation exists.
3. Single screening: Oyatie target p95 1 s is ahead of SAP and Thomson estimates and at parity/ahead with Descartes estimate.
4. Single screening: demo/trial OCI Always Free may double p99 under saturation but preserves same screening logic.
5. Batch screening: Oyatie target p95 8 s is at parity with the strongest Descartes estimate and ahead of SAP/Thomson estimates.
6. Continuous rescreening: Oyatie target p99 24 h is parity-to-ahead against public 24-hour or daily rescreening rhythms.
7. Continuous rescreening: p95 12 h is an ahead target if implemented.
8. Export classification: Oyatie target p95 350 ms per line is ahead of estimated SAP/Thomson/Descartes classification bands.
9. HS/HTS/ECCN classification: Oyatie target p95 500 ms per line is parity-to-ahead against counterpart estimates.
10. 500-line classification: Oyatie target p95 35 s is aggressive but consistent with the per-line target plus batching.
11. Customs command accept: Oyatie target p95 300 ms is ahead of estimated SAP customs command accept.
12. Broker callback: Oyatie target p95 750 ms is ahead of local ADR p95 1 s and ahead of counterpart estimate bands.
13. Trade document generation: Oyatie target p95 1.5 s is competitive for standard documents.
14. Certificate generation: Oyatie target p95 2 s is competitive if document templates and source evidence are cached.
15. Duty drawback state transition: Oyatie target p95 500 ms is ahead for internal state movement but excludes external agency waits.
16. Quota/origin decision: Oyatie target p95 750 ms is competitive if source data is local and versioned.
17. Embargo audit-chain anchor: Oyatie target p95 500 ms is ahead for event-local anchoring.
18. Event emission: Oyatie target p99 3 s matches local ADR and is industry-grade.
19. Audit replay RPO: Oyatie target 15 minutes is conservative and enterprise-grade.
20. Sustained screening throughput: Oyatie 500 rps per production cell is ahead of estimated enterprise baseline.
21. Burst screening throughput: Oyatie 2,000 rps per cell is ahead if queue backpressure and horizontal scale exist.
22. OCI Always Free demo/trial throughput: 25 rps is below production counterpart scale by design.
23. OCI Always Free demo/trial posture: lower throughput is an infrastructure cap, not a quality downgrade.
24. Regulatory-content propagation: Oyatie p95 6 h and p99 12 h is ahead of public 24-hour Thomson/Descartes-style content rhythm.
25. Regulatory-content propagation: this is not yet credible because source-ingestion design is absent.
26. Content breadth: Oyatie has no current evidence matching Thomson's 220+ countries, 500+ FTAs, 750+ lists, 1,300 sources, or 130 million annual updates.
27. Content breadth: Oyatie has no current evidence matching Descartes' 180+ countries or 6 million regulatory sources.
28. Business-result metrics: Oyatie has no current evidence matching Descartes' public 60 percent false-positive reduction claim.
29. Business-result metrics: Oyatie has no current evidence matching Descartes' public 95 percent data-visibility claim.
30. Business-result metrics: Oyatie has no current evidence matching Descartes' public 30 percent duty/tariff savings claim.
31. Business-result metrics: Oyatie has no current evidence matching Descartes' public 75 percent manual-screening-time reduction claim.
32. Availability: Oyatie 99.95 percent target is ahead of the conservative 99.9 percent counterpart estimate.
33. Availability: target is not yet OpenTofu/context-proven.
34. OS coverage: no service-local OS benchmark can be claimed.
35. Deployment contexts: no context-specific benchmark can be claimed until OpenTofu modules exist.
36. Tenant classes: no tenant-class-specific benchmark can be claimed until fields and policies exist.
37. Implementation evidence: no runtime benchmark can be claimed until Rust code and tests exist.
38. Current verdict: target numbers are industry-leader grade.
39. Current verdict: measured evidence is absent.
40. Current verdict: first remediation is executable benchmark harness plus contract expansion.

## 5. Benchmark Workload Acceptance Gates

1. Gate 1: add Rust implementation for six original bounded contexts.
2. Gate 2: add Rust implementation for accepted expanded IP-016 through IP-023 surfaces.
3. Gate 3: add contract fields for tenant_class and commercial arrangement.
4. Gate 4: add OpenTofu modules for all six deployment contexts.
5. Gate 5: add OCI Always Free profile module for demo/trial infrastructure.
6. Gate 6: add supported OS manifest and OS-specific CI lanes.
7. Gate 7: add benchmark harness for single screening.
8. Gate 8: add benchmark harness for 100-party batch screening.
9. Gate 9: add benchmark harness for continuous rescreening update.
10. Gate 10: add benchmark harness for export-control classification.
11. Gate 11: add benchmark harness for HS/HTS/ECCN classification.
12. Gate 12: add benchmark harness for customs declaration command accept.
13. Gate 13: add benchmark harness for broker callback ingest.
14. Gate 14: add benchmark harness for trade document generation.
15. Gate 15: add benchmark harness for certificate generation.
16. Gate 16: add benchmark harness for duty-drawback state transition.
17. Gate 17: add benchmark harness for quota/origin decision.
18. Gate 18: add benchmark harness for embargo audit-chain anchor.
19. Gate 19: add benchmark harness for event-emission p99.
20. Gate 20: add content-source freshness workload.
21. Gate 21: add per-context benchmark overlays.
22. Gate 22: add per-tenant-class benchmark overlays.
23. Gate 23: collect p50/p95/p99 for every workload.
24. Gate 24: collect sustained and burst rps for every production context.
25. Gate 25: collect concurrency saturation for every production context.
26. Gate 26: collect OCI Always Free demo/trial saturation results.
27. Gate 27: compare measured results against the single target set.
28. Gate 28: publish failures as remediation issues, not as target downgrades.
29. Gate 29: keep feature quality uniform across tenant classes.
30. Gate 30: rerun after every material contract or runtime change.

## 6. Stop Condition

1. This report is complete when it states methodology, counterpart numbers, Oyatie target numbers, overlays, and comparison narrative.
2. This report is not a runtime benchmark result.
3. The next runtime benchmark result requires source implementation, test harnesses, deployment modules, OS manifest, and tenant-class fields.
4. No old feature-tier benchmark rows are created here.
5. No fourth tier-delta deliverable is created here.
