---
doc_class: PerformanceBenchmarkNumbers
microservice: warehouse
status: authored
audit_date: 2026-05-20
authored_on: 2026-05-21
deliverable: 3_of_3
counterparts:
  - SAP Extended Warehouse Management
  - Manhattan Active WM
  - Blue Yonder WMS
target_model: single-industry-leader-target-set-with-context-and-tenant-class-overlays
---

# Warehouse Performance Benchmark Numbers - 2026-05-20

## Header Citation Anchor Block

1. Canonical multi-context anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1736-1749`.
2. Canonical OpenTofu anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2249`.
3. Canonical OCI Always Free anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514-3577` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3666-3697`.
4. Canonical tenant-class amendment anchor: current batch instruction plus `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability activation_2026_05_20.md:10-64`.
5. Local warehouse SLO anchor: `slos/warehouse-latency-p99.openslo.yaml:21-29`, `slos/warehouse-availability.openslo.yaml:21-29`, `slos/warehouse-throughput.openslo.yaml:21-29`.
6. Local warehouse capacity anchor: `capacity-model.md:13-23` and `capacity-model.md:26-65`.
7. SAP public capability anchor: `https://www.sap.com/australia/products/scm/extended-warehouse-management/features.html:66-92`.
8. SAP case-number anchor: `https://leverx.com/case-studies/warehouse-management-transformation:114-238`.
9. Manhattan public capability anchor: `https://www.manh.com/solutions/supply-chain-management-software/warehouse-management:297-355`.
10. Manhattan case-number anchors: `https://www.manh.com/our-insights/resources/case-study/semir-implements-agile-supply-chain-support-multi-channel-business:152-162`, `https://www.manh.com/our-insights/resources/case-study/lifestyle-retailer-boosts-digital-commerce-throughput-30:152-163`, `https://www.manh.com/customer-stories/mercury:156-176`, `https://www.manh.com/customer-stories/staples:159-181`.
11. Blue Yonder public capability anchor: `https://blueyonder.com/solutions/warehouse-management:419-555`.
12. Blue Yonder case-number anchor: `https://blueyonder.com/customers/prinsel:389-411`.

## Methodology Disclosure

1. This report does not pretend that WMS vendors publish comparable API latency or requests-per-second benchmarks.
2. Public WMS evidence is mostly customer outcome evidence, not raw service-internal latency.
3. The counterpart-number section therefore separates three evidence types.
4. Evidence type A: public customer KPI numbers from vendor or implementation case studies.
5. Evidence type B: public capability numbers such as shipment volume, sites, years, or implementation counts.
6. Evidence type C: estimated technical benchmark numbers derived from local warehouse workloads and stated methodology.
7. Every estimated number is marked `estimated`.
8. Every public number is marked `source`.
9. The Oyatie target section uses a single industry-leader target set.
10. The target section does not segment targets by feature tier.
11. Deployment-context overlays describe infrastructure constraints.
12. Tenant-class overlays describe usage caps, contractual scale, or revenue-share cost posture.
13. Tenant-class overlays do not reduce product quality.
14. Latency dimensions cover command acceptance, policy decision, idempotency reservation, audit append, event publish, and operator-visible response.
15. Throughput dimensions cover accepted warehouse commands per second, source import replay events per minute, and wave/labor planning jobs.
16. Scale-ceiling dimensions cover facility count, active operators, open tasks, item/bin cardinality, and automation endpoints.
17. Workload W1 is inbound receipt command for 1 to 100 lines.
18. Workload W2 is outbound release for up to 5,000 delivery lines.
19. Workload W3 is picking-wave optimization for up to 2,000 stops.
20. Workload W4 is labor assignment for up to 500 active workers.
21. Workload W5 is yard appointment check-in and dock assignment.
22. Workload W6 is source import replay from legacy WMS.
23. Workload W7 is robotics/WES command handoff; local artifacts do not yet implement it.
24. Workload W8 is slotting analysis over bins, quants, velocities, and constraints.
25. OS disclosure: warehouse currently has no `supported-oses.json`; targets assume the canonical OS matrix from ADR-0328 until a manifest exists.
26. Architecture disclosure: local runtime currently returns adapter stubs at `src/adapter/http.rs:65-67`, `src/adapter/grpc.rs:57-59`, and `src/adapter/asyncapi.rs:58-60`.
27. Therefore Oyatie target numbers are contractual targets for remediation, not measured current runtime numbers.
28. Current local measured performance is not available because active benchmark harnesses and fixture tests are absent.
29. Local SLO evidence currently sets a 350 ms latency-bucket threshold with 0.99 target at `slos/warehouse-latency-p99.openslo.yaml:21-29`.
30. Local availability evidence currently sets 0.999 at `slos/warehouse-availability.openslo.yaml:27-29`.
31. Local throughput evidence currently sets accepted-over-received ratio target 0.995 at `slos/warehouse-throughput.openslo.yaml:21-29`.
32. Local capacity model has tier-shaped assumptions that must be retired, but its numeric rows are useful as raw workload references at `capacity-model.md:16-65`.
33. This file uses those raw references without preserving the retired tier model.
34. Method stop condition: produce competitor numbers, Oyatie targets, overlays, and comparison narrative with no tier headings or rows.

## 1. Methodology

1. Benchmark dimension: command accept latency.
2. Unit: milliseconds.
3. Test workload: W1, W2, W4, W5.
4. Measurement point: edge request accepted after tenant scope, Cedar permit, idempotency reservation, audit enqueue, and event enqueue.
5. Benchmark dimension: end-to-end operator response latency.
6. Unit: milliseconds.
7. Test workload: RF scan, voice-pick response, yard check-in, labor accept.
8. Benchmark dimension: asynchronous planning job latency.
9. Unit: seconds or minutes.
10. Test workload: W3 route optimization, W8 slotting analysis, source import validation.
11. Benchmark dimension: command throughput.
12. Unit: accepted commands per second.
13. Test workload: mixed inbound/outbound/putaway/picking/labor/yard command stream.
14. Benchmark dimension: replay/import throughput.
15. Unit: events per minute.
16. Test workload: W6 source import replay with idempotency and audit seal.
17. Benchmark dimension: physical-operation outcome.
18. Unit: picking efficiency, labor productivity, fulfillment time, stockout rate, throughput volume.
19. Test workload: vendor public customer case metrics.
20. Benchmark dimension: availability.
21. Unit: percentage and error-budget ratio.
22. Test workload: all command and event surfaces.
23. Benchmark dimension: scale ceiling.
24. Unit: sites, centers, SKUs, boxes, orders, pieces, active workers, and automation endpoints.
25. Deployment context: `oyatie-public-cloud` assumes elastic managed substrate.
26. Deployment context: `guest-on-aws` assumes customer AWS account limits.
27. Deployment context: `guest-on-oci` assumes customer OCI account limits.
28. Deployment context: `on-prem` assumes facility network, device, and hardware constraints.
29. Deployment context: `colo` assumes fixed capacity plus controlled network perimeter.
30. Deployment context: `oyatie-as-cloud-provider` assumes Oyatie-owned cloud substrate.
31. Tenant class: `demo_trial` uses the OCI Always Free profile where applicable and has hard usage caps.
32. Tenant class: `paid` scales with contract, per-seat licensing, and usage-based billing.
33. Tenant class: `revenue_share` runs at-cost or zero-margin substrate with settlement evidence tied to gross revenue.
34. Benchmark apparatus required later: Rust load driver, synthetic warehouse fixture generator, OpenAPI/gRPC/AsyncAPI fixture replay, Prometheus scrape, and audit-chain seal verifier.
35. Benchmark apparatus currently missing: active runtime handlers and active contract fixture tests.

## 2. Counterpart Numbers

### 2.1 SAP Extended Warehouse Management Numbers

1. SAP-01 source: SAP case study reports days in inventory reduced from 68.4 days to 47.3 days.
2. SAP-01 computed: 21.1 fewer days and about 31% reduction.
3. Citation: `https://leverx.com/case-studies/warehouse-management-transformation:128-136`.
4. SAP-02 source: SAP case study reports labor productivity moving from 153 FTEs per $1B revenue to 90 FTEs per $1B revenue.
5. SAP-02 computed: about 41% labor productivity gain.
6. Citation: `https://leverx.com/case-studies/warehouse-management-transformation:137-143`.
7. SAP-03 source: capacity-planning labor time reduced from 5 hours per week per planner to 0 hours.
8. SAP-03 computed: 100% removal of that planning labor task in the case.
9. Citation: `https://leverx.com/case-studies/warehouse-management-transformation:145-151`.
10. SAP-04 source: inventory carrying cost reduced from $3.2M/year to $2.6M/year.
11. SAP-04 computed: $650K/year savings.
12. Citation: `https://leverx.com/case-studies/warehouse-management-transformation:161-169`.
13. SAP-05 source: annual recurring benefit range is $467,500 conservative to $650,000 optimistic.
14. SAP-05 source: payback is reported at 15 months with 187% ROI.
15. Citation: `https://leverx.com/case-studies/warehouse-management-transformation:221-238`.
16. SAP-06 source: other measurable improvements include 25% inventory-days improvement.
17. SAP-06 source: other measurable improvements include 20% labor productivity increase.
18. SAP-06 source: other measurable improvements include 5% logistics-cost reduction.
19. Citation: `https://leverx.com/case-studies/warehouse-management-transformation:229-234`.
20. SAP-07 source: SAP feature page includes inbound processing, storage/internal process, outbound waves, catch weights, dock appointments, yard, labor, value-added services, kitting, cross-docking, and robotics.
21. SAP-07 interpretation: benchmark scope has at least 11 major operational capability families.
22. Citation: `https://www.sap.com/australia/products/scm/extended-warehouse-management/features.html:75-92`.
23. SAP-08 estimated: interactive command p99 target for SAP-class parity should be 350 ms or lower because local SLO already uses `le="0.35"`.
24. SAP-08 citation: `slos/warehouse-latency-p99.openslo.yaml:21-29`.
25. SAP-09 estimated: outbound release planning should support 5,000 delivery lines within 700 ms p95 because local IP-008 declares that target.
26. SAP-09 citation: `IP-008-usecase-layer-for-outbound-delivery.md:147-153`.
27. SAP-10 estimated: enterprise command throughput should reach at least 2,500 rps because the local capacity model uses 2,500 rps as the highest raw command-rate row in its assumptions.
28. SAP-10 citation: `capacity-model.md:16-23`.
29. SAP-11 estimated: SAP-class robotics handoff should be less than 250 ms p95 from warehouse decision to automation command enqueue because robotics must remain physically safe and operator-visible.
30. SAP-11 citation: SAP robotics source `https://www.sap.com/australia/products/scm/extended-warehouse-management/features.html:89-92`.
31. SAP-12 estimated: migration/import replay should sustain at least 100,000 sealed events per minute in elastic public cloud to make SAP cutover windows practical for high-volume warehouses.
32. SAP-12 citation: local replay need in `backfill-replay.md:1-120` and SAP high-volume scope in `https://www.sap.com/hk/products/scm/extended-warehouse-management.html:61-72`.

### 2.2 Manhattan Active WM Numbers

1. MAN-01 source: Semir reports 60% picking efficiency improvement.
2. MAN-01 citation: `https://www.manh.com/our-insights/resources/case-study/semir-implements-agile-supply-chain-support-multi-channel-business:152-162`.
3. MAN-02 source: Semir reports 40% labor cost reduction.
4. MAN-02 citation: `https://www.manh.com/our-insights/resources/case-study/semir-implements-agile-supply-chain-support-multi-channel-business:152-162`.
5. MAN-03 source: Semir reports 30% space utilization improvement.
6. MAN-03 citation: `https://www.manh.com/our-insights/resources/case-study/semir-implements-agile-supply-chain-support-multi-channel-business:152-162`.
7. MAN-04 source: Semir reports 350,000 pieces processed in a 7-hour order-preparation window.
8. MAN-04 computed: about 50,000 pieces per hour.
9. MAN-04 citation: `https://www.manh.com/customer-stories/semir:159-176`.
10. MAN-05 source: lifestyle retailer reports 30% overall throughput increase with Order Streaming.
11. MAN-05 citation: `https://www.manh.com/our-insights/resources/case-study/lifestyle-retailer-boosts-digital-commerce-throughput-30:152-163`.
12. MAN-06 source: lifestyle retailer reports click-to-ship time reduced by 38%.
13. MAN-06 citation: `https://www.manh.com/our-insights/resources/case-study/lifestyle-retailer-boosts-digital-commerce-throughput-30:152-163`.
14. MAN-07 source: Mercury reports inventory errors decreased from 0.05% to 0.003%.
15. MAN-07 source: Mercury labels that as 94% increase in accuracy.
16. MAN-07 citation: `https://www.manh.com/customer-stories/mercury:159-176`.
17. MAN-08 source: Mercury reports staffing fell 88% from 430 to 50 workers while maintaining shipping efficiency and throughput.
18. MAN-08 citation: `https://www.manh.com/customer-stories/mercury:167-172`.
19. MAN-09 source: Mercury reports over 200,000 orders processed in three days instead of seven.
20. MAN-09 source: Mercury labels that as 133.33% fulfillment speed increase.
21. MAN-09 citation: `https://www.manh.com/customer-stories/mercury:173-176`.
22. MAN-10 source: Staples reports 2.5 million units shipped each day across 36 centers and 220 locations.
23. MAN-10 citation: `https://www.manh.com/customer-stories/staples:159-170`.
24. MAN-11 source: Staples reports next-day service to over 98% of the United States.
25. MAN-11 citation: `https://www.manh.com/customer-stories/staples:172-176`.
26. MAN-12 source: Staples reports nine B2B sites implemented in 18 months.
27. MAN-12 citation: `https://www.manh.com/customer-stories/staples:178-181`.
28. MAN-13 source: Manhattan says Order Streaming adjusts to new orders, labor changes, equipment failures, and more.
29. MAN-13 citation: `https://www.manh.com/solutions/supply-chain-management-software/warehouse-management:314-324`.
30. MAN-14 source: Manhattan says WES inside WMS integrates sortation equipment, put walls, automated storage/retrieval, and robotic solutions.
31. MAN-14 citation: `https://www.manh.com/solutions/supply-chain-management-software/warehouse-management:342-355`.
32. MAN-15 estimated: Manhattan-class order streaming requires sub-1-second replan feedback for facility task queues because the public feature describes real-time changes.
33. MAN-15 citation: `https://www.manh.com/solutions/supply-chain-management-software/warehouse-management:321-324`.

### 2.3 Blue Yonder WMS Numbers

1. BY-01 source: Prinsel operates two busy distribution centers.
2. BY-01 citation: `https://blueyonder.com/customers/prinsel:389-391`.
3. BY-02 source: Prinsel manages over 1,000 SKUs.
4. BY-02 citation: `https://blueyonder.com/customers/prinsel:389-391`.
5. BY-03 source: Prinsel manages 7,000 shipments.
6. BY-03 citation: `https://blueyonder.com/customers/prinsel:389-391`.
7. BY-04 source: Prinsel manages 2.5 million individual boxes each year.
8. BY-04 citation: `https://blueyonder.com/customers/prinsel:389-391`.
9. BY-05 source: Prinsel reports 2% inventory reduction.
10. BY-05 citation: `https://blueyonder.com/customers/prinsel:399-404`.
11. BY-06 source: Prinsel reports 30% warehouse-efficiency improvement.
12. BY-06 citation: `https://blueyonder.com/customers/prinsel:399-404`.
13. BY-07 source: Prinsel reports 62% reduction in fulfillment time.
14. BY-07 citation: `https://blueyonder.com/customers/prinsel:402-405`.
15. BY-08 source: Prinsel reports fulfillment time moved from two hours to 45 minutes.
16. BY-08 citation: `https://blueyonder.com/customers/prinsel:402-405`.
17. BY-09 source: Blue Yonder and Argano report over 400 customer implementations in 20 countries.
18. BY-09 citation: `https://blueyonder.com/customers/prinsel:407-408`.
19. BY-10 source: Argano relies on more than 170 accredited developers and consultants and provides 24/7 support.
20. BY-10 citation: `https://blueyonder.com/customers/prinsel:410-411`.
21. BY-11 source: Blue Yonder public page lists AI Agents, resource forecasting, resource orchestration, robotics hub, warehouse labor, advanced slotting, load building, yard management, execution, returns processing, and analyst workbench.
22. BY-11 citation: `https://blueyonder.com/solutions/warehouse-management:419-444`.
23. BY-12 source: Blue Yonder says AI recommendations are real-time and understandable.
24. BY-12 citation: `https://blueyonder.com/solutions/warehouse-management:453-456`.
25. BY-13 source: Blue Yonder says resource orchestration continuously assigns the best available resource to every task during the day.
26. BY-13 citation: `https://blueyonder.com/solutions/warehouse-management:466-469`.
27. BY-14 source: Blue Yonder says Robotics Hub onboards robotics vendors and warehouse automation solutions to a single platform.
28. BY-14 citation: `https://blueyonder.com/solutions/warehouse-management:471-473`.
29. BY-15 estimated: Blue Yonder-class AI task recommendation p95 should be 500 ms or lower for operator-facing recommendations because the product claim is real-time.
30. BY-15 citation: `https://blueyonder.com/solutions/warehouse-management:453-456` and `https://blueyonder.com/solutions/warehouse-management:559-561`.

## 3. Oyatie Target Numbers - Single Industry-Leader Target Set

1. Target OYA-01 command accept latency p50: <= 40 ms.
2. Target OYA-01 command accept latency p95: <= 150 ms.
3. Target OYA-01 command accept latency p99: <= 300 ms.
4. Rationale: local SLO bucket is 350 ms at 0.99, so the target tightens it by about 14%.
5. Citation: `slos/warehouse-latency-p99.openslo.yaml:21-29`.
6. Target OYA-02 policy decision latency p95: <= 20 ms from Cedar context construction to decision.
7. Target OYA-03 idempotency reservation p95: <= 25 ms in public-cloud contexts.
8. Target OYA-04 audit enqueue p95: <= 30 ms with durable outbox.
9. Target OYA-05 event publish enqueue p95: <= 30 ms.
10. Target OYA-06 RF/voice operator response p95: <= 200 ms after command acceptance.
11. Target OYA-07 outbound release p95: <= 700 ms for 5,000 delivery lines.
12. Citation: `IP-008-usecase-layer-for-outbound-delivery.md:147-153`.
13. Target OYA-08 picking route optimization p95: <= 30 seconds for 2,000 stops.
14. Target OYA-09 slotting analysis p95: <= 5 minutes for 50,000 bins and 1 million quants.
15. Target OYA-10 labor assignment p95: <= 250 ms for 500 active workers.
16. Target OYA-11 yard appointment check-in p95: <= 300 ms.
17. Target OYA-12 cross-dock candidate decision p95: <= 250 ms.
18. Target OYA-13 replenishment trigger evaluation p95: <= 500 ms per wave demand batch.
19. Target OYA-14 robotics/WES command handoff p95: <= 250 ms after safe decision.
20. Target OYA-15 source import replay public-cloud throughput: >= 100,000 sealed events per minute.
21. Target OYA-16 source import replay constrained-edge throughput: >= 10,000 sealed events per minute.
22. Target OYA-17 accepted command throughput public cloud: >= 10,000 rps per cell with horizontal tenant partitioning.
23. Target OYA-18 accepted command throughput single paid tenant baseline: >= 2,500 rps.
24. Citation: `capacity-model.md:16-23`.
25. Target OYA-19 accepted-over-received ratio: >= 0.995.
26. Citation: `slos/warehouse-throughput.openslo.yaml:21-29`.
27. Target OYA-20 availability baseline: >= 99.9%.
28. Citation: `slos/warehouse-availability.openslo.yaml:27-29`.
29. Target OYA-21 paid contractual availability floor for production contexts: >= 99.95% where the selected deployment substrate supports it.
30. Target OYA-22 demo-trial hard cap: <= 50 accepted commands per second per tenant unless explicitly upgraded.
31. Target OYA-23 demo-trial source import cap: <= 250,000 imported legacy rows per tenant.
32. Target OYA-24 paid tenant cap: governed by purchased usage and per-seat license, with no artificial feature ceiling.
33. Target OYA-25 revenue-share tenant cap: governed by at-cost substrate budget and revenue settlement terms, with no artificial feature ceiling.
34. Target OYA-26 active facility scale: >= 500 facilities per paid tenant across cells.
35. Target OYA-27 active operator scale: >= 100,000 active workers/devices per paid tenant across cells.
36. Target OYA-28 open task scale: >= 50 million open or recent warehouse tasks per paid tenant across cells.
37. Target OYA-29 automation endpoint scale: >= 10,000 AMR/conveyor/WCS endpoints per paid tenant once WES surface exists.
38. Target OYA-30 inventory accuracy target: >= 99.95% stock-position correctness for scanned/validated bins.
39. Target OYA-31 picking efficiency target: at least 30% improvement over pre-Oyatie baseline in first controlled rollout.
40. Target OYA-32 labor productivity target: at least 20% improvement over pre-Oyatie baseline in first controlled rollout.
41. Target OYA-33 fulfillment-time target: at least 38% reduction over pre-Oyatie baseline where order streaming is enabled.
42. Target OYA-34 stockout-rate target: at least 30% reduction over pre-Oyatie baseline where replenishment and slotting are enabled.
43. Target OYA-35 migration cutover target: rollback decision within 15 minutes and source import resume within 30 minutes after defect isolation.

### 3.1 Deployment-Context Overlays

1. `oyatie-public-cloud`: OYA-01 through OYA-35 are binding targets, with elastic scale and managed observability.
2. `guest-on-aws`: OYA-01 through OYA-35 apply when customer AWS quotas, instance classes, and network design satisfy sizing.
3. `guest-on-oci`: OYA-01 through OYA-35 apply when customer OCI quotas and Ampere/shape selection satisfy sizing.
4. `guest-on-oci` OCI Always Free profile: command throughput capped at 50 rps per demo-trial tenant; replay capped at 10,000 events/minute; storage capped by Always Free limits.
5. `on-prem`: latency targets apply inside facility LAN; replay and analytics targets depend on hardware and storage media.
6. `colo`: public-cloud latency may be matched if network path and storage match reference architecture; elasticity is capacity-reservation based.
7. `oyatie-as-cloud-provider`: same targets as public cloud, with Oyatie-owned cloud-k8s/cloud-iac/cloud-secrets substrate.
8. Context gap: warehouse currently has no context directories, so these are target overlays, not implemented overlays.
9. Citation: missing IaC inventory plus ADR-0328 context requirement at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854-3885`.

### 3.2 Tenant-Class Overlays

1. `demo_trial`: use the OCI Always Free profile when possible.
2. `demo_trial`: cap command throughput, source import volume, retention, and facility count.
3. `demo_trial`: retain best-effort SLO but preserve the same correctness semantics.
4. `demo_trial`: no compliance packs and no BYOK.
5. `paid`: contractual SLO, compliance packs allowed, BYOK allowed, any deployment context.
6. `paid`: throughput and retention scale with per-seat license and usage-based billing.
7. `revenue_share`: substrate runs at-cost or zero-margin as settlement economics require.
8. `revenue_share`: throughput scales to marketplace seller/B2C/operator demand when gross-revenue share justifies capacity.
9. `revenue_share`: compliance and BYOK follow contract, not feature tiers.
10. Tenant-class gap: warehouse currently has no tenant-class fields.
11. Citation: tenant-class scan evidence and `manifest.json:1-295`.

## 4. Comparison Narrative

1. Command latency: Oyatie target p99 <= 300 ms is ahead of the current local SLO bucket of 350 ms.
2. Command latency: counterpart public pages do not publish directly comparable API p99 values.
3. Command latency verdict: target is defensible, measurement missing.
4. Throughput: Oyatie paid baseline of 2,500 rps matches the highest raw local capacity assumption.
5. Throughput: Oyatie public-cloud per-cell target of 10,000 rps is an industry-leader target, not yet proven.
6. Throughput: Manhattan Semir's 350,000 pieces in seven hours is about 50,000 pieces/hour physical throughput.
7. Throughput verdict: Oyatie target is ahead in software-command terms, but physical validation is missing.
8. Picking efficiency: Manhattan Semir reports 60% improvement; Oyatie first-rollout target is 30% improvement.
9. Picking efficiency verdict: catch-up target; Manhattan public case is higher.
10. Fulfillment time: Blue Yonder Prinsel reports 62% fulfillment time reduction from two hours to 45 minutes.
11. Fulfillment time: Manhattan lifestyle retailer reports 38% click-to-ship reduction.
12. Fulfillment time: Oyatie target is at least 38% reduction after order streaming exists.
13. Fulfillment-time verdict: parity with Manhattan case target, catch-up against Blue Yonder Prinsel.
14. Labor productivity: SAP case reports about 41% gain and separate 20% measurable improvement.
15. Labor productivity: Manhattan Semir reports 40% labor cost reduction; Mercury reports 88% staffing reduction while sustaining throughput.
16. Labor productivity: Oyatie target is at least 20% improvement on first rollout.
17. Labor verdict: conservative catch-up until predictive labor and engineered labor standards exist.
18. Inventory accuracy: Manhattan Mercury reports errors moving from 0.05% to 0.003%.
19. Inventory accuracy: Oyatie target is >= 99.95% validated stock-position correctness.
20. Inventory verdict: parity target, but requires serial/batch/catch-weight contract fields not yet present.
21. Replenishment signal: Blue Yonder Prinsel reports 2% inventory reduction alongside faster fulfillment.
22. Stockout reduction: Oyatie target is 30% reduction when replenishment and slotting are enabled, treated as an internal target rather than a directly cited counterpart number.
23. Stockout verdict: target is aspirational until warehouse adds replenishment contract evidence and a public counterpart baseline is selected.
24. Automation: SAP, Manhattan, and Blue Yonder all publish robotics/automation capabilities.
25. Automation: Oyatie has no WES/robotics artifact, so OYA-14 and OYA-29 are not currently implementable.
26. Automation verdict: gap.
27. Availability: local target is 99.9%; paid production target is 99.95% where substrate supports it.
28. Availability verdict: reasonable target, but context overlays and runbooks need automation scenarios.
29. Migration: SAP case reports 15-month payback and 187% ROI; Oyatie has no migration playbooks.
30. Migration verdict: gap until SAP/Manhattan/Blue Yonder migration playbooks exist.
31. OCI Always Free: target demo-trial cap is intentionally lower than paid/revenue-share scale because infrastructure is constrained.
32. OCI Always Free verdict: canonical overlay exists in this target document, but no local IaC profile exists.
33. Final benchmark verdict: targets are industry-leader-grade but mostly unproven.
34. Required proof gate: implement runtime handlers, add active fixture tests, add load harness, create context IaC, and publish measured results.
35. Required claim language until then: `target`, `estimate`, or `planned`; never `measured production performance`.
