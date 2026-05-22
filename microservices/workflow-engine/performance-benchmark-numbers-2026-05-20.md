# workflow-engine performance benchmark numbers - 2026-05-20

Service: `workflow-engine`.
Benchmark posture: single industry-leader target set with deployment-context and tenant-class overlays.
No capability-tier segmentation is used in this document.
Target counterpart set: Temporal, Camunda Platform 8, Apache Airflow.
Methodology status: counterpart numbers are public limits, defaults, and official benchmark disclosures where available; Oyatie numbers are target numbers unless explicitly cited as existing local targets.
Validation status: no live workflow-engine benchmark harness ran in this audit because no source/test tree exists under `microservices/workflow-engine/`.

Citation anchor 1: Temporal Cloud Limits, `https://docs.temporal.io/cloud/limits`, used for APS/RPS/OPS, payload, history, schedule, visibility, poller, retention, batch, update, and deployment limits.
Citation anchor 2: Temporal Event History and workflow limits, `https://docs.temporal.io/cloud/limits`, used for 51,200 events, 50 MiB history, command, signal, update, and payload ceilings.
Citation anchor 3: Camunda 8 performance tuning docs, `https://docs.camunda.io/docs/self-managed/operational-guides/performance/performance-tuning/`, used for partition, CPU, disk, exporter, and backpressure guidance.
Citation anchor 4: Camunda official benchmark blog, `https://camunda.com/blog/2019/10/benchmarking-camunda-bpm-platform-7-10/`, used for published TPS, latency, and external-task benchmark examples.
Citation anchor 5: Apache Airflow configuration reference, `https://airflow.apache.org/docs/apache-airflow/stable/configurations-ref.html`, used for scheduler heartbeat, task heartbeat, query batch, mapping, parser, DAG, and task defaults.
Local anchor 1: `microservices/workflow-engine/PRD.md:755-763`, used for existing Oyatie latency, replay, audit seal, cold-start, and cross-cell targets.
Local anchor 2: `microservices/workflow-engine/PRD.md:773-779`, used for existing Oyatie per-cell active run, step dispatch, and event bus scale targets.
Local anchor 3: `microservices/workflow-engine/capacity-model.md:39-48`, used for existing throughput formulas.
Local anchor 4: `microservices/workflow-engine/capacity-model.md:69-89`, used for existing replica formulas.
Local anchor 5: `microservices/workflow-engine/ARCHITECTURE.md:691-697`, used for cross-service dependencies that constrain performance overlays.

## 1. Methodology

1. Benchmark dimension 01: workflow start latency p50, p95, and p99.
2. Benchmark dimension 02: event-to-action latency p50, p95, and p99.
3. Benchmark dimension 03: step dispatch throughput per cell.
4. Benchmark dimension 04: event bus throughput per cell.
5. Benchmark dimension 05: concurrent active runs per cell.
6. Benchmark dimension 06: replay throughput per worker.
7. Benchmark dimension 07: audit seal latency.
8. Benchmark dimension 08: cold-start worker pod latency.
9. Benchmark dimension 09: cross-cell spawn round-trip.
10. Benchmark dimension 10: payload size and event history ceilings.
11. Benchmark dimension 11: task queue or worker-poller scale.
12. Benchmark dimension 12: schedule and timer throughput.
13. Benchmark dimension 13: visibility/query throughput.
14. Benchmark dimension 14: retention and long-duration workflow viability.
15. Benchmark dimension 15: failure recovery time, including worker crash, region outage, and replay storm response.
16. Test workload A: short five-step saga with one external activity, one timer, one signal, one audit seal, and one compensation branch.
17. Test workload B: long-running human approval workflow with 30-day duration, daily timer wakeups, and externally delivered signals.
18. Test workload C: fan-out/fan-in workflow with 100 child branches and idempotent external calls.
19. Test workload D: event-driven workflow triggered by the event bus and replayed from offset.
20. Test workload E: foundry self-modification workflow that publishes a new workflow version, runs policy preflight, and emits audit evidence.
21. OS disclosure: target OS matrix must come from a future `supported-oses.json`; current service path does not contain it.
22. Architecture disclosure: backend target language is Rust, not Python/JavaScript/TypeScript/Ruby/Go/Java/Scala/Groovy/PHP/F#.
23. Deployment context disclosure: six contexts are required: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
24. Deployment context gap: current target path has Helm/Kustomize but no six-context OpenTofu modules.
25. Tenant-class disclosure: `demo_trial`, `paid`, and `revenue_share` are commercial/capacity overlays, not feature-quality tiers.
26. Tenant-class gap: current target path has no `tenant_class`, `demo_trial`, or `revenue_share` semantics.
27. Measurement disclosure: no numbers below are claimed as measured for current code unless cited to an existing Oyatie target table.
28. Counterpart disclosure: public counterpart numbers are often limits, defaults, or sample benchmarks, not normalized apples-to-apples workloads.
29. Target disclosure: Oyatie target numbers are chosen to meet or beat the top public counterpart for comparable durable workflow workloads while respecting audit, residency, and Cedar overhead.
30. Evidence floor: every target number is either grounded in local PRD/capacity model lines or explicitly marked as a proposed target derived from counterpart public limits.

## 2. Counterpart numbers

### 2.1 Temporal numbers

1. Temporal number 01: default namespace actions-per-second public limit is 500 APS; source: Temporal Cloud Limits.
2. Temporal number 02: namespace RPS public limit is 2,000 RPS; source: Temporal Cloud Limits.
3. Temporal number 03: namespace persistence operations public limit is 4,000 OPS; source: Temporal Cloud Limits.
4. Temporal number 04: schedules RPS public limit is 10 RPS; source: Temporal Cloud Limits.
5. Temporal number 05: visibility RPS public limit is 30 RPS; source: Temporal Cloud Limits.
6. Temporal number 06: worker pollers may scale to 20,000 activity pollers plus 20,000 workflow pollers per namespace before higher-limit discussion; source: Temporal Cloud Limits.
7. Temporal number 07: workflow execution history limit is 51,200 events; source: Temporal Cloud Limits.
8. Temporal number 08: workflow execution history size limit is 50 MiB; source: Temporal Cloud Limits.
9. Temporal number 09: gRPC message size limit is 4 MiB; source: Temporal Cloud Limits.
10. Temporal number 10: payload size warning starts at 512 KiB and payload failure occurs at 2 MiB; source: Temporal Cloud Limits.
11. Temporal number 11: transaction size limit is 4 MiB; source: Temporal Cloud Limits.
12. Temporal number 12: incomplete command count limit is 2,000, with 500 listed as the recommended command count; source: Temporal Cloud Limits.
13. Temporal number 13: incomplete signals limit is 10,000; source: Temporal Cloud Limits.
14. Temporal number 14: in-flight workflow update limit is 10 and total update limit is 2,000; source: Temporal Cloud Limits.
15. Temporal number 15: workflow execution retention defaults to 30 days and can be set from 1 to 90 days; source: Temporal Cloud Limits.
16. Temporal number 16: batch operations can operate on 50 workflows per second; source: Temporal Cloud Limits.
17. Temporal number 17: maximum workflow run timeout is 10 years; source: Temporal Cloud Limits.
18. Temporal number 18: maximum workflow execution timeout is 10 years; source: Temporal Cloud Limits.
19. Temporal number 19: maximum open workflow executions per namespace is governed by rate limit policy rather than a single universal public ceiling; source: Temporal Cloud Limits.
20. Temporal interpretation: Temporal publishes clear cloud limits, but comparable end-to-end latency depends heavily on namespace limits, worker code, persistence, payload, and queue topology.

### 2.2 Camunda Platform 8 numbers

1. Camunda number 01: Zeebe performance guidance treats partitions as the unit that distributes workflow instances; source: Camunda 8 performance tuning docs.
2. Camunda number 02: Camunda guidance recommends benchmarking with production-like load because workflow shape and job worker behavior dominate throughput; source: Camunda 8 performance tuning docs.
3. Camunda number 03: Camunda guidance emphasizes CPU and I/O as dominant bottlenecks for Zeebe brokers; source: Camunda 8 performance tuning docs.
4. Camunda number 04: Camunda guidance recommends fast SSD/NVMe class disks for broker log and snapshot performance; source: Camunda 8 performance tuning docs.
5. Camunda number 05: Camunda guidance states exporters can add backpressure and must be sized as part of throughput; source: Camunda 8 performance tuning docs.
6. Camunda number 06: Camunda guidance treats job activation and completion rates as key workload dimensions; source: Camunda 8 performance tuning docs.
7. Camunda number 07: Camunda benchmark blog includes a 10 TPS benchmark case for an eight-hour run on a single-topic workload; source: Camunda benchmark blog.
8. Camunda number 08: Camunda benchmark blog reports an external-task fetch-and-lock average around 700 TPS in the benchmarked setup; source: Camunda benchmark blog.
9. Camunda number 09: Camunda benchmark blog reports TP99 latency below 500 ms in a benchmarked scenario; source: Camunda benchmark blog.
10. Camunda number 10: Camunda benchmark blog uses 45 minute to 1 hour benchmark runs and recommends 12 hour certification runs for higher confidence; source: Camunda benchmark blog.
11. Camunda number 11: Camunda benchmark blog compares workflow creation throughput in the tens to hundreds per second across versions and setups; source: Camunda benchmark blog.
12. Camunda number 12: Camunda benchmark blog reports total workflows in sample runs in the hundreds of thousands to millions depending on version/setup; source: Camunda benchmark blog.
13. Camunda number 13: Camunda self-managed deployments commonly scale by partition count, broker resources, worker resources, and exporter/storage capacity rather than a single universal SaaS limit; source: Camunda 8 performance tuning docs.
14. Camunda number 14: Camunda latency and throughput are sensitive to BPMN model complexity, job worker speed, and exporter backlog; source: Camunda 8 performance tuning docs.
15. Camunda interpretation: public numbers prove Camunda can hit strong process throughput, but they are workload-specific and less direct than Temporal's published cloud limits.

### 2.3 Apache Airflow numbers

1. Airflow number 01: default scheduler heartbeat is 5 seconds; source: Apache Airflow configuration reference.
2. Airflow number 02: default task heartbeat timeout is 300 seconds; source: Apache Airflow configuration reference.
3. Airflow number 03: default max task instances per scheduler query is 16; source: Apache Airflow configuration reference.
4. Airflow number 04: default maximum mapped task expansion length is 1,024; source: Apache Airflow configuration reference.
5. Airflow number 05: default scheduler parsing loop and file-processing settings are configuration driven rather than fixed product limits; source: Apache Airflow configuration reference.
6. Airflow number 06: default global parallelism is configurable and often starts at 32 in standard Airflow deployments; source: Apache Airflow configuration reference.
7. Airflow number 07: max active tasks per DAG is configurable in the core config; source: Apache Airflow configuration reference.
8. Airflow number 08: max active runs per DAG is configurable in the core config; source: Apache Airflow configuration reference.
9. Airflow number 09: task queued timeout is configurable and defines how long queued tasks can wait before failing; source: Apache Airflow configuration reference.
10. Airflow number 10: scheduler health check threshold is configurable; source: Apache Airflow configuration reference.
11. Airflow number 11: DAG file parsing timeout is configurable; source: Apache Airflow configuration reference.
12. Airflow number 12: task success overtime is configurable; source: Apache Airflow configuration reference.
13. Airflow number 13: asset/dataset scheduling behavior is configuration and executor dependent; source: Apache Airflow configuration reference.
14. Airflow number 14: task execution scale depends on executor choice, database capacity, scheduler count, worker count, DAG parse cost, and operator behavior.
15. Airflow number 15: Airflow's public documentation provides many scheduler and executor settings but does not publish a universal durable workflow throughput number comparable to Temporal APS.
16. Airflow interpretation: Airflow is a DAG scheduler and operational orchestrator, so Oyatie should benchmark against DAG scheduling and backfill workloads separately from durable state-machine workloads.

## 3. Oyatie target numbers - single industry-leader target set

### 3.1 Canonical target set

1. Target 01: workflow start latency p50 <= 50 ms.
2. Target 02: workflow start latency p95 <= 300 ms.
3. Target 03: workflow start latency p99 <= 500 ms.
4. Target evidence: local PRD event-to-action latency table already sets 50/300/500 ms; citation: `microservices/workflow-engine/PRD.md:755-756`.
5. Target 04: event-to-action latency p50 <= 50 ms.
6. Target 05: event-to-action latency p95 <= 300 ms.
7. Target 06: event-to-action latency p99 <= 500 ms.
8. Target 07: state persistence per step p50 <= 5 ms.
9. Target 08: state persistence per step p95 <= 18 ms.
10. Target 09: state persistence per step p99 <= 25 ms.
11. Target evidence: local PRD state persistence table already sets 5/18/25 ms; citation: `microservices/workflow-engine/PRD.md:755-756`.
12. Target 10: replay throughput >= 1,000 steps/s/worker.
13. Target evidence: local PRD replay table sets 1000 steps/s/worker; citation: `microservices/workflow-engine/PRD.md:757`.
14. Target 11: audit seal p95 <= 800 ms.
15. Target 12: audit seal p99 <= 1,000 ms.
16. Target evidence: local PRD audit seal table and audit section set 800 ms and p99 <= 1 s; citations: `microservices/workflow-engine/PRD.md:758` and `microservices/workflow-engine/PRD.md:800`.
17. Target 13: cold-start worker pod p95 <= 400 ms.
18. Target 14: cold-start worker pod p99 <= 500 ms.
19. Target evidence: local PRD cold-start table sets 400/500 ms; citation: `microservices/workflow-engine/PRD.md:759`.
20. Target 15: cross-cell spawn p50 <= 100 ms.
21. Target 16: cross-cell spawn p95 <= 400 ms.
22. Target 17: cross-cell spawn p99 <= 800 ms.
23. Target evidence: local PRD cross-cell table sets 100/400/800 ms; citation: `microservices/workflow-engine/PRD.md:760`.
24. Target 18: active runs baseline >= 10,000 per cell.
25. Target 19: active runs ceiling >= 500,000 per cell when infrastructure is sized for it.
26. Target evidence: local PRD scalability section sets 10,000 baseline and 500,000 max active runs per cell; citation: `microservices/workflow-engine/PRD.md:775`.
27. Target 20: step dispatch baseline >= 5,000 steps/s per cell.
28. Target 21: step dispatch ceiling >= 200,000 steps/s per cell when infrastructure is sized for it.
29. Target evidence: local PRD scalability section sets 5,000 baseline and 200,000 max steps/s per cell; citation: `microservices/workflow-engine/PRD.md:776`.
30. Target 22: event bus baseline >= 10,000 events/s per cell.
31. Target 23: event bus ceiling >= 1,000,000 events/s per cell when infrastructure is sized for it.
32. Target evidence: local PRD scalability section sets 10,000 baseline and 1,000,000 max events/s per cell; citation: `microservices/workflow-engine/PRD.md:777`.
33. Target 24: payload accepted hard ceiling <= 2 MiB by default unless workflow spec declares an approved external blob reference.
34. Target 25: payload warning threshold <= 512 KiB.
35. Target rationale: match Temporal's public payload warning/failure profile to avoid unbounded history growth.
36. Target 26: execution history soft rollover before 50,000 events or 50 MiB.
37. Target rationale: stay inside Temporal-like event history limits while allowing deterministic continue-as-new behavior.
38. Target 27: workflow retention default 30 days for completed event history.
39. Target 28: completed history retention configurable up to 90 days for paid or revenue-share contracts when jurisdiction allows.
40. Target 29: paused state retention default 90 days.
41. Target evidence: local PRD sets 90-day paused-state retention; citation: `microservices/workflow-engine/PRD.md:786`.
42. Target 30: schedule creation/control RPS >= 10 RPS per tenant control plane shard.
43. Target rationale: parity with Temporal schedule RPS public limit, scaled by shard rather than global namespace.
44. Target 31: visibility/query RPS >= 30 RPS per tenant control plane shard.
45. Target rationale: parity with Temporal visibility RPS public limit, scaled by shard.
46. Target 32: workflow batch maintenance >= 50 workflows/s per maintenance shard.
47. Target rationale: parity with Temporal batch operation public limit.
48. Target 33: worker poller ceiling should support at least 20,000 workflow pollers and 20,000 activity pollers per large cell when storage and network are sized.
49. Target rationale: parity with Temporal poller public limit.
50. Target 34: deterministic replay mismatch rate target is zero accepted mismatches.
51. Target evidence: replay determinism OpenSLO is registered in manifest; citation: `microservices/workflow-engine/manifest.json:107-110`.
52. Target 35: payload-budget correctness target is 100 percent within budget for accepted events.
53. Target evidence: payload budget OpenSLO is registered in manifest; citation: `microservices/workflow-engine/manifest.json:99-105`.
54. Target 36: worker poll availability target should remain at or above 99.95 percent for production cells.
55. Target evidence: worker polling SLO file exists in manifest registry; citation: `microservices/workflow-engine/manifest.json:99-135`.
56. Target 37: workflow completion availability target should remain at or above 99.95 percent monthly for production cells.
57. Target evidence: PRD availability table sets execution path 99.95 percent monthly; citation: `microservices/workflow-engine/PRD.md:764-771`.
58. Target 38: replay backend availability target should remain at or above 99.9 percent monthly.
59. Target evidence: PRD availability table sets replay backend 99.9 percent; citation: `microservices/workflow-engine/PRD.md:764-771`.
60. Target 39: spec store availability target should remain at or above 99.99 percent monthly.
61. Target evidence: PRD availability table sets spec store 99.99 percent; citation: `microservices/workflow-engine/PRD.md:764-771`.

### 3.2 Deployment-context overlays

1. `oyatie-public-cloud` overlay: target full canonical numbers with horizontal elasticity, multi-AZ storage, and automated OpenTofu capacity expansion.
2. `oyatie-public-cloud` latency overlay: p99 500 ms event-to-action remains the production SLO target.
3. `oyatie-public-cloud` throughput overlay: 10,000 active runs, 5,000 steps/s, and 10,000 events/s are baseline per cell; higher ceilings require scaled worker/storage cells.
4. `guest-on-aws` overlay: target full feature set, but throughput ceiling depends on customer AWS account quotas, network egress, EBS/io2 or local NVMe choices, and region support.
5. `guest-on-aws` latency overlay: p99 500 ms remains target inside the guest account when the account grants required compute, storage, and network posture.
6. `guest-on-oci` overlay: target full feature set, with demo-trial OCI Always Free profile using hard usage caps rather than reduced feature quality.
7. `guest-on-oci` latency overlay: p99 500 ms is the target for paid/revenue-share capacity; demo_trial may queue above capped throughput.
8. `on-prem` overlay: target full feature set, with facility-specific storage/network ceiling documented before contractual SLO.
9. `on-prem` latency overlay: p99 500 ms remains the engine target when facility storage meets the required write and fsync profile.
10. `colo` overlay: target full feature set, with cross-connect latency and facility redundancy included in the SLO attachment.
11. `colo` latency overlay: p99 500 ms remains local-cell target; cross-cell spawn must disclose facility-to-facility latency.
12. `oyatie-as-cloud-provider` overlay: target full numbers plus provider-control-plane obligations for tenant onboarding, usage metering, and provider-grade elasticity.
13. Context blocker: none of these overlays are executable yet because context-specific OpenTofu modules are absent under the microservice path.
14. Context blocker evidence: current IaC inventory is Helm/Kustomize only and IP-001 still references a forbidden Terraform path; citation: `microservices/workflow-engine/IP-001-layer-a-postgres-citus-valkey-clickhouse-iac.md:30-36`.

### 3.3 Tenant-class overlays

1. `demo_trial` overlay: feature quality remains industry-leader grade, but throughput, retention, and active-run caps are hard-limited to stay inside the OCI Always Free profile or equivalent free-trial budget.
2. `demo_trial` starts cap target: 10 starts/s per trial tenant until measured OCI Always Free profile proves a higher cap.
3. `demo_trial` step cap target: 100 steps/s per trial tenant until measured OCI Always Free profile proves a higher cap.
4. `demo_trial` active-run cap target: 100 concurrent active runs per trial tenant.
5. `demo_trial` event retention target: 7 days for completed history, unless compliance requires shorter deletion.
6. `demo_trial` replay retention target: 7 days for replayable completed history.
7. `demo_trial` SLO: best-effort SLO with the same correctness floor; queuing is allowed when caps are exceeded.
8. `paid` overlay: per-seat license plus usage billing scales capacity by contract and payment.
9. `paid` starts cap target: at least 5,000 starts/s per paid production cell when infrastructure is sized.
10. `paid` step cap target: at least 200,000 steps/s per paid production cell when infrastructure is sized.
11. `paid` active-run cap target: at least 500,000 active runs per production cell when infrastructure is sized.
12. `paid` event retention target: 30 days default and up to 90 days contractually where jurisdiction allows.
13. `paid` SLO: contractual SLO with compliance packs and BYOK allowed.
14. `revenue_share` overlay: capacity scales at-cost or zero-margin substrate tied to gross-revenue-share economics.
15. `revenue_share` starts cap target: same technical scale as paid when revenue-share terms fund the substrate.
16. `revenue_share` step cap target: same technical scale as paid when revenue-share terms fund the substrate.
17. `revenue_share` active-run cap target: same technical scale as paid when revenue-share terms fund the substrate.
18. `revenue_share` event retention target: same as paid unless marketplace/regulatory terms require longer or shorter windows.
19. `revenue_share` SLO: contractual SLO if revenue-share agreement includes it; otherwise at-cost capacity with explicit cap disclosure.
20. Tenant-class blocker: these overlays are target policy, not current service artifact truth, because no `tenant_class` semantics exist under the target path.

## 4. Comparison narrative

1. Workflow start/event-to-action p99: Oyatie target 500 ms is parity with Camunda benchmark-blog sub-500 ms TP99 examples and more workload-specific than Airflow scheduler heartbeat defaults.
2. Workflow start/event-to-action p95: Oyatie target 300 ms is ahead of scheduler-loop oriented Airflow behavior for event-driven workflows and appropriate for Temporal-like durable execution if storage and worker queues are sized.
3. State persistence p99: Oyatie target 25 ms is aggressive and requires local NVMe/fast block storage, tuned Postgres/Citus, and bounded payloads.
4. Replay throughput: Oyatie target 1,000 steps/s/worker is ahead of Airflow-style rerun/backfill semantics and should be compared to Temporal replay under deterministic workload fixtures.
5. Audit seal latency: Oyatie target p99 1 second is an additive constraint absent from the counterpart union and should be benchmarked separately from dispatch latency.
6. Active runs baseline: Oyatie 10,000 active runs/cell is a credible industry-leader baseline but below the proposed max of 500,000 active runs/cell.
7. Step dispatch baseline: Oyatie 5,000 steps/s/cell is strong compared with public Camunda examples and Airflow scheduler defaults, but it needs source and benchmark harness proof.
8. Event bus baseline: Oyatie 10,000 events/s/cell is ahead of Airflow DAG scheduling expectations and must be proven under backpressure and replay.
9. Event bus ceiling: Oyatie 1,000,000 events/s/cell is aspirational until storage, queue, and consumer-lag tests exist.
10. Payload ceiling: matching Temporal's 512 KiB warning and 2 MiB failure profile is a conservative correctness choice.
11. History ceiling: rolling over before 50,000 events or 50 MiB keeps Oyatie within a Temporal-like durability envelope and protects replay cost.
12. Schedule RPS: target 10 RPS per control-plane shard matches Temporal's public schedule limit while allowing horizontal sharding.
13. Visibility RPS: target 30 RPS per shard matches Temporal's public visibility limit while requiring bounded cardinality and indexed queries.
14. Batch maintenance: target 50 workflows/s per maintenance shard matches Temporal's public batch operation number.
15. Worker pollers: target 20,000 workflow and 20,000 activity pollers per large cell matches Temporal's public poller envelope and is ahead of default Airflow scheduler assumptions.
16. Airflow comparison: Airflow remains strong for Python DAG ecosystems, but workflow-engine should not import Python runtime semantics under Rust-strict policy.
17. Camunda comparison: Camunda remains strong for BPMN and human process UX, so workflow-engine needs Workflow Studio and incident/tasklist handoff docs to claim end-to-end parity.
18. Temporal comparison: Temporal remains the strongest durable-execution benchmark; workflow-engine's main additive difference is Cedar/audit/residency/foundry self-hosting integration.
19. Demo-trial comparison: demo_trial caps are commercial and infrastructure caps; they do not reduce the correctness, replay, authorization, or audit feature set.
20. Paid comparison: paid tenants should meet the canonical target set when the deployment context is provisioned to the required storage, worker, and event-bus profile.
21. Revenue-share comparison: revenue_share tenants should meet paid-class technical targets when gross-revenue-share economics fund substrate cost.
22. Deployment comparison: public-cloud context can target full elasticity first, while guest/on-prem/colo contexts require local quota and facility evidence.
23. Context risk: without OpenTofu context modules, none of the overlay numbers have deployable proof.
24. Source risk: without Rust source/tests under the path or a canonical source pointer, none of the target numbers have executable proof.
25. Benchmark risk: existing benchmark docs are stale because they segment Oyatie by retired capability tiers rather than this single-target plus overlay model.
26. Final performance verdict: target numbers are coherent and industry-leader-grade, but current workflow-engine artifacts need source, tests, OpenTofu, OS manifest, and tenant-class overlays before measured claims are defensible.

## 5. Measurement plan

1. Measurement step 01: create a five-step saga fixture with one activity, one timer, one signal, one compensation branch, and one audit seal.
2. Measurement step 02: create a long-running workflow fixture with daily wakeups and external signals for at least 30 simulated days.
3. Measurement step 03: create a fan-out fixture with 100 child branches and deterministic fan-in.
4. Measurement step 04: create an event-driven fixture fed by the event bus with replay from offset.
5. Measurement step 05: create a foundry self-modification fixture that publishes a workflow version, runs policy preflight, and emits sealed evidence.
6. Measurement step 06: run each fixture in a single-cell development profile to establish code-path correctness.
7. Measurement step 07: run each fixture in `guest-on-oci` demo_trial profile to measure OCI Always Free caps.
8. Measurement step 08: run each fixture in a paid-class public-cloud profile to measure elastic targets.
9. Measurement step 09: run each fixture in a revenue-share at-cost profile when cost-accounting hooks exist.
10. Measurement step 10: run replay fixtures under worker crash and pod eviction.
11. Measurement step 11: run event-bus fixtures under subscriber lag and backpressure.
12. Measurement step 12: run spec-store fixtures under spec version pinning and hot reload.
13. Measurement step 13: run payload fixtures at 512 KiB warning, 2 MiB failure, and external blob-reference boundaries.
14. Measurement step 14: run history fixtures at 10,000, 25,000, and 50,000 events to prove continue-as-new behavior before the ceiling.
15. Measurement step 15: run audit-seal fixtures with normal and degraded audit-chain latency.
16. Measurement step 16: run Cedar-deny fixtures to verify denied mutations still emit evidence without contaminating throughput success counts.
17. Measurement step 17: run pack-residency fixtures that attempt cross-pack state replication and verify refusal by default.
18. Measurement step 18: run worker-poller scale tests at 1,000, 5,000, 10,000, and 20,000 pollers when infrastructure allows.
19. Measurement step 19: run active-run saturation at 10,000 baseline before attempting 100,000 and 500,000 ceilings.
20. Measurement step 20: run event-bus saturation at 10,000 events/s before attempting higher scaled ceilings.
21. Measurement step 21: run step-dispatch saturation at 5,000 steps/s before attempting higher scaled ceilings.
22. Measurement step 22: run visibility/query tests at 30 RPS per shard and scale by shard count.
23. Measurement step 23: run schedule-control tests at 10 RPS per shard and scale by shard count.
24. Measurement step 24: record p50, p95, p99, max, error rate, retry count, queue depth, storage write latency, audit latency, and Cedar latency for every fixture.
25. Measurement step 25: record context, tenant_class, OS, CPU architecture, storage class, network class, worker count, partition count, and database topology for every run.
26. Measurement step 26: reject any benchmark run that lacks source SHA, config digest, OpenTofu plan digest, and contract/schema version.
27. Measurement step 27: publish benchmark outputs as machine-readable artifacts before writing narrative claims.
28. Measurement step 28: compare Temporal-like workloads separately from Camunda-like BPMN/process workloads and Airflow-like DAG scheduling workloads.
29. Measurement step 29: keep audit/Cedar/residency overhead in the benchmark path because it is part of Oyatie's product surface.
30. Measurement step 30: publish both saturated throughput and correctness-preserving throughput; correctness-preserving throughput is the only externally claimable number.

## 6. Acceptance gates for future measured claims

1. Gate 01: source tree exists or a canonical source pointer resolves from `manifest.json`.
2. Gate 02: deterministic replay test passes for all workload fixtures.
3. Gate 03: payload-budget OpenSLO is green during benchmark windows.
4. Gate 04: replay-determinism OpenSLO is green during benchmark windows.
5. Gate 05: worker-poll availability OpenSLO is green during benchmark windows.
6. Gate 06: workflow-completion availability OpenSLO is green during benchmark windows.
7. Gate 07: workflow-start latency OpenSLO is green during benchmark windows.
8. Gate 08: workflow-step execute latency OpenSLO is green during benchmark windows.
9. Gate 09: Cedar deny decisions emit audit evidence without counting as successful mutations.
10. Gate 10: audit-chain backpressure behavior matches failure-mode docs.
11. Gate 11: worker crash recovery matches runbook recovery time.
12. Gate 12: replay storm mitigation matches failure-mode docs.
13. Gate 13: Valkey failover matches runbook and failure-mode docs.
14. Gate 14: cross-tenant leak tests fail closed.
15. Gate 15: cross-pack replication tests fail closed by default.
16. Gate 16: OpenTofu context module exists for the measured deployment context.
17. Gate 17: OS/architecture tuple is listed in `supported-oses.json`.
18. Gate 18: tenant_class is captured in benchmark metadata.
19. Gate 19: demo_trial runs prove caps and queuing behavior without reducing feature quality.
20. Gate 20: paid runs prove contractual target numbers at provisioned capacity.
21. Gate 21: revenue_share runs prove at-cost scaling assumptions or declare the exact commercial cap.
22. Gate 22: external counterpart numbers are cited with source date and benchmark comparability notes.
23. Gate 23: stale retired segmentation language is absent from the benchmark artifact.
24. Gate 24: benchmark output can be traced to source, config, contract, policy, and infrastructure digests.
25. Gate 25: final claim wording distinguishes measured, target, estimated, and source-derived numbers.

## 7. Performance report conclusion

1. The local PRD already sets credible latency, replay, audit, cold-start, and cross-cell targets.
2. The local PRD already sets credible per-cell active-run, step-dispatch, and event-bus targets.
3. Temporal remains the most useful public comparison for durable workflow ceilings and limits.
4. Camunda remains the most useful public comparison for process-engine throughput and latency examples.
5. Airflow remains the most useful public comparison for scheduler configuration and DAG operational constraints.
6. Oyatie's performance target must include Cedar, audit, and residency overhead because those are first-class product capabilities.
7. The old benchmark document should be superseded by this single-target overlay model after measured harnesses exist.
8. The current performance posture is target-defined and evidence-cited, not measured from code.
9. Promotion should wait for source, tests, OpenTofu context modules, OS manifest, and tenant-class metadata.
10. The next benchmark artifact should be machine-readable first and prose second.
