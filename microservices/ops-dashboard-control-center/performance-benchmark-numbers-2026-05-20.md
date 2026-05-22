# ops-dashboard-control-center Performance Benchmark Numbers - 2026-05-20

Audit target: `microservices/ops-dashboard-control-center/`.
Counterparts: Datadog, PagerDuty, AWS CloudWatch plus AWS Systems Manager.
Benchmark model: one industry-leader target set, with deployment-context overlays and tenant-class overlays.
No retired commercial tenant_class headings or rows are used in this report.
Tenant classes disclosed: `demo_trial`, `paid`, `revenue_share`.
Methodology disclosure: the Oyatie numbers below are target numbers derived from service-local capacity/SLO documents, canonical deployment constraints, and public counterpart limits; they are not measured production results until the missing Rust implementation, OpenTofu contexts, OS matrix, and CI lanes exist.

## Citation Anchor Block

Anchor 1: ADR-0328 requires performance reports to disclose OS, architecture, deployment context, and tenant class in `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4208-4209`.
Anchor 2: ODCC service capacity target is 500 named operators, 1,000 rps sustained, 5,000 rps burst, and 200 high-risk mutations per hour in `capacity-model.md:21-28`.
Anchor 3: ODCC service latency budgets include Cedar p99 at or below 5 ms and command-path budgets in `capacity-model.md:52-68`.
Anchor 4: Datadog documents event submission at 250,000 events per minute per organization and API rate-limit headers in `https://docs.datadoghq.com/api/latest/rate-limits/` lines 3296-3309.
Anchor 5: Datadog documents log intake limits of 5 MB per uncompressed payload, 1 MB per single log, 1,000 entries per array, and 18-hour past timestamp acceptance in `https://docs.datadoghq.com/api/latest/logs/` lines 4298-4308.
Anchor 6: PagerDuty documents REST rate-limit headers and 429 behavior in `https://support.pagerduty.com/main/docs/rest-api-rate-limits` lines 351-394.
Anchor 7: PagerDuty staff community guidance gives 960 requests per minute for account keys, user keys, and app-token forms in `https://community.pagerduty.com/ask-a-product-question-2/rest-api-rate-limit-151`.
Anchor 8: AWS CloudWatch documents 500 PutMetricData rps, 500 GetMetricData rps, 400 GetMetricStatistics rps, 10 GetMetricStream rps, 10 PutDashboard rps, and related API quotas in `https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/cloudwatch_limits.html` lines 74-100.
Anchor 9: AWS Systems Manager documents 100 concurrent automations, 5,000 queued automations, 25 concurrent rate-control automations, and 1,000 queued rate-control automations in `https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-automation.html` lines 74-84.
Anchor 10: AWS Systems Manager adaptive concurrency can scale from 100 concurrent automations to 500 in `https://docs.aws.amazon.com/systems-manager/latest/userguide/running-automations-scale.html` lines 8-10.

## §1 Methodology

Benchmark dimension 01: command API latency for incident declaration, deployment approval, rollback decision, and evidence export initiation.
Benchmark dimension 02: read API latency for cluster health, tenant posture, policy decision review, dashboard summaries, and audit-stream pagination.
Benchmark dimension 03: mutation throughput for high-risk operator actions.
Benchmark dimension 04: read throughput for dashboard and posture queries.
Benchmark dimension 05: event emission throughput into audit-chain and observability sinks.
Benchmark dimension 06: evidence export initiation latency and sustained export throughput.
Benchmark dimension 07: step-up authentication latency budget.
Benchmark dimension 08: Cedar authorization latency budget.
Benchmark dimension 09: command queue depth and drain time.
Benchmark dimension 10: automation concurrency and error-threshold behavior.
Benchmark dimension 11: SLO burn query freshness.
Benchmark dimension 12: multi-region propagation latency for operator-visible state.
Benchmark dimension 13: tenant posture freshness.
Benchmark dimension 14: cluster health freshness.
Benchmark dimension 15: rate-limit response behavior and retry-after semantics.
Benchmark dimension 16: operational cost and resource caps by deployment context.

Test workload A: 500 named operators with 50 concurrent active operators.
Test workload B: 1,000 rps sustained read-heavy dashboard traffic.
Test workload C: 5,000 rps burst read-heavy dashboard traffic for 60 seconds.
Test workload D: 200 high-risk mutations per hour, including deployment approvals and rollback decisions.
Test workload E: 10,000 tenant posture reads per minute with tenant, cell, pack, and residency filters.
Test workload F: 1,000 incident declarations per hour during regional disruption simulation.
Test workload G: 100 GB evidence-pack export initiated through ODCC and streamed through the evidence substrate.
Test workload H: 10,000 policy decision review reads per minute.
Test workload I: 1,000 recovery workflow state transitions per hour.
Test workload J: 100,000 audit-chain events per minute as a stress ceiling target.

OS disclosure: canonical readiness requires the master OS matrix, but this service has no `supported-oses.json`.
OS target lane: Linux amd64 and Linux arm64 must be measured first because all six deployment contexts depend on them.
OS target lane: macOS Apple Silicon M5+ matters for developer/operator tooling only if local console packaging is claimed.
OS target lane: linux/ppc64le and linux/s390x are soft-gate measurement lanes if the service is compiled as a portable Rust backend.
Architecture disclosure: primary measurement targets are linux/amd64 and linux/arm64.
Architecture disclosure: OCI Always Free profile measurement must include ARM Ampere A1 caps where applicable.
Deployment context disclosure: all six contexts are evaluated as overlays.
Context 1: `oyatie-public-cloud` uses Oyatie-managed elasticity and should meet the canonical target set.
Context 2: `guest-on-aws` maps to customer or delegated AWS accounts behind Oyatie abstractions and should meet the target set if account quotas are raised where needed.
Context 3: `guest-on-oci` maps to OCI backing resources and includes an Always Free profile subcase.
Context 4: `on-prem` depends on customer facility capacity and must publish measured local ceilings.
Context 5: `colo` depends on facility network, power, and hardware ceilings but should meet the target set for planned cells.
Context 6: `oyatie-as-cloud-provider` is an internal provider surface and should meet or exceed the target set for externally sold cloud cells.
Tenant class disclosure: `demo_trial` may apply usage caps and OCI Always Free profile caps.
Tenant class disclosure: `paid` scales with contract, payment, and deployment context.
Tenant class disclosure: `revenue_share` runs at cost or zero-margin substrate but does not receive lower safety or evidence quality.
Measurement disclosure: source files and tests are currently absent under this service path, so the report defines acceptance numbers for future implementation.
Validation disclosure: until OpenTofu modules, source crates, tests, and CI lanes exist, the numbers below are targets and gates, not measured service claims.

## §2 Counterpart Numbers

### §2.1 Datadog Public Numbers

Datadog number 01: event submission rate limit is 250,000 events per minute per organization.
Datadog number 02: event submission rate limit equals approximately 4,166 events per second per organization.
Datadog number 03: API rate-limit response uses HTTP 429 after a limit is exceeded.
Datadog number 04: API rate-limit header `X-RateLimit-Limit` reports allowed requests in the period.
Datadog number 05: API rate-limit header `X-RateLimit-Period` reports reset period in seconds.
Datadog number 06: API rate-limit header `X-RateLimit-Remaining` reports remaining allowance.
Datadog number 07: API rate-limit header `X-RateLimit-Reset` reports seconds until reset.
Datadog number 08: API rate-limit header `X-RateLimit-Name` identifies the limit bucket.
Datadog number 09: log intake v2 maximum uncompressed payload is 5 MB.
Datadog number 10: log intake maximum single-log size is 1 MB.
Datadog number 11: log intake maximum array size is 1,000 entries.
Datadog number 12: log intake accepts timestamps up to 18 hours in the past.
Datadog number 13: log intake returns 413 when a batch is above 5 MB uncompressed.
Datadog number 14: Datadog API usage metrics are generally rolled up to one-minute windows for widget use.
Datadog number 15: Datadog incident management is seat-based, so operator count directly affects incident-management cost modeling.
Datadog interpretation 01: Datadog is much broader than ODCC for telemetry ingestion and search.
Datadog interpretation 02: Datadog gives a concrete high-water event ingestion reference at 250,000 events per minute.
Datadog interpretation 03: ODCC should not copy Datadog log limits unless ODCC owns log ingestion; it should use them as pivot/export compatibility constraints.
Datadog interpretation 04: ODCC should match Datadog-style explicit rate-limit headers for operator APIs.
Datadog interpretation 05: ODCC should expose usage metrics for blocked and allowed API requests at one-minute granularity.

### §2.2 PagerDuty Public Numbers

PagerDuty number 01: public support guidance says REST APIs expose rate-limit headers.
PagerDuty number 02: documented rate-limit headers include `ratelimit-limit`.
PagerDuty number 03: documented rate-limit headers include `ratelimit-remaining`.
PagerDuty number 04: documented rate-limit headers include `ratelimit-reset`.
PagerDuty number 05: documented behavior includes HTTP 429 responses when applications reach rate limits.
PagerDuty number 06: staff community guidance states 960 requests per minute for each account API key.
PagerDuty number 07: 960 requests per minute equals 16 requests per second.
PagerDuty number 08: staff community guidance states user API keys are limited at 960 requests per minute per user across that user's keys.
PagerDuty number 09: staff community guidance states app OAuth tokens are allowed 960 requests per minute against each authorized account.
PagerDuty number 10: staff community guidance states app user OAuth tokens are allowed 960 requests per minute per authorized user.
PagerDuty number 11: support guidance says using a private scoped OAuth app plus user token can effectively double available API rate for that access pattern.
PagerDuty number 12: support guidance recommends one bot user per application deployment to keep rate limits separate.
PagerDuty number 13: PagerDuty support guidance says rate-limit headers were made available in October and November 2023.
PagerDuty number 14: PagerDuty support guidance says apps that reached limits began receiving 429 responses after rollout.
PagerDuty number 15: PagerDuty incident workflow external web API actions are documented with an account-wide throughput cap of 500 MB per minute in support documentation.
PagerDuty interpretation 01: PagerDuty's public fixed REST number is far lower than ODCC's proposed internal 1,000 rps sustained read target.
PagerDuty interpretation 02: PagerDuty remains the benchmark for operational accountability, not raw API throughput.
PagerDuty interpretation 03: ODCC should expose predictable rate-limit headers and retry guidance on every operator API.
PagerDuty interpretation 04: ODCC should keep per-operator and per-tenant rate buckets separate to avoid one tenant starving another.
PagerDuty interpretation 05: ODCC should measure notification and escalation latencies if it owns those surfaces.

### §2.3 AWS CloudWatch plus Systems Manager Public Numbers

AWS number 01: CloudWatch `PutMetricData` default quota is 500 requests per second per supported Region.
AWS number 02: CloudWatch `GetMetricData` default quota is 500 requests per second per supported Region.
AWS number 03: CloudWatch `GetMetricStatistics` default quota is 400 requests per second per supported Region.
AWS number 04: CloudWatch `GetMetricStream` default quota is 10 requests per second per supported Region.
AWS number 05: CloudWatch `GetMetricWidgetImage` default quota is 20 requests per second per supported Region.
AWS number 06: CloudWatch `ListMetrics` default quota is 25 requests per second per supported Region.
AWS number 07: CloudWatch `PutDashboard` default quota is 10 requests per second per supported Region.
AWS number 08: CloudWatch `GetDashboard` default quota is 10 requests per second per supported Region.
AWS number 09: CloudWatch `PutMetricAlarm` default quota is 3 requests per second per supported Region.
AWS number 10: CloudWatch recent `GetMetricData` datapoint fetch quota is 180,000 datapoints per second.
AWS number 11: CloudWatch Metrics Insights `GetMetricData` datapoint quota is 4,300,000 datapoints per second.
AWS number 12: Systems Manager Automation default concurrent automation quota is 100 per account.
AWS number 13: Systems Manager Automation queue quota is 5,000 automations per account.
AWS number 14: Systems Manager rate-control automation concurrent quota is 25 per account.
AWS number 15: Systems Manager rate-control automation queue quota is 1,000 per account.
AWS number 16: Systems Manager adaptive concurrency can raise concurrent automations from 100 to 500.
AWS interpretation 01: AWS gives the clearest public numbers for regional control-plane API throughput.
AWS interpretation 02: ODCC read targets should meet or exceed CloudWatch dashboard/API read quotas when running as an Oyatie-managed service.
AWS interpretation 03: ODCC mutation targets should respect that AWS alarm and dashboard mutation APIs are intentionally lower throughput than reads.
AWS interpretation 04: ODCC automation targets should model queueing and rate control explicitly, not only synchronous API latency.
AWS interpretation 05: ODCC deployment-context overlays must account for AWS account and Region service quotas in `guest-on-aws`.

## §3 Oyatie Target Numbers - Single Industry-Leader Target Set

Target 01: operator read API p50 latency <= 40 ms.
Target 02: operator read API p95 latency <= 120 ms.
Target 03: operator read API p99 latency <= 250 ms.
Target 04: operator mutation API p50 latency <= 80 ms.
Target 05: operator mutation API p95 latency <= 250 ms.
Target 06: operator mutation API p99 latency <= 500 ms.
Target 07: Cedar authorization p99 latency <= 5 ms, matching `capacity-model.md:52-55`.
Target 08: step-up authentication p95 challenge generation <= 250 ms excluding human interaction.
Target 09: step-up authentication p99 server-side verification <= 500 ms excluding human interaction.
Target 10: incident declaration p99 commit plus audit emission <= 500 ms.
Target 11: deployment approval p99 commit plus audit emission <= 500 ms.
Target 12: rollback decision p99 commit plus audit emission <= 500 ms.
Target 13: policy decision review read p99 <= 250 ms.
Target 14: cluster health summary read p99 <= 250 ms.
Target 15: tenant posture read p99 <= 250 ms.
Target 16: dashboard summary p99 <= 300 ms.
Target 17: audit-stream page p99 <= 350 ms for a 100-row page.
Target 18: evidence export initiation p99 <= 500 ms.
Target 19: evidence export sustained throughput target >= 500 MB/s when backing storage and signer allow it.
Target 20: 100 GB evidence pack target duration <= 240 seconds in elastic contexts.
Target 21: sustained read throughput target >= 1,000 rps per active cell.
Target 22: burst read throughput target >= 5,000 rps for 60 seconds per active cell.
Target 23: sustained mutation throughput target >= 200 high-risk mutations per hour.
Target 24: safe mutation burst target >= 50 high-risk mutations per minute for emergency windows.
Target 25: audit event emission target >= 100,000 events per minute per active cell.
Target 26: audit event emission ceiling should exceed Datadog's 250,000 events per minute per organization when multi-cell aggregation is enabled.
Target 27: tenant posture read target >= 10,000 reads per minute per active cell.
Target 28: policy decision review target >= 10,000 reads per minute per active cell.
Target 29: incident declaration target >= 1,000 declarations per hour per active cell.
Target 30: recovery workflow transition target >= 1,000 state transitions per hour per active cell.
Target 31: command queue redline <= 500 pending commands, matching the local capacity model in `capacity-model.md:56-68`.
Target 32: command queue normal drain time <= 60 seconds at redline with autoscaling enabled.
Target 33: command queue emergency drain time <= 15 seconds for rollback and incident-severity transitions.
Target 34: rate-limit response must include limit, remaining, reset, bucket name, and retry-after semantics.
Target 35: per-operator default read bucket >= 60 requests per minute.
Target 36: per-operator emergency command bucket >= 20 high-risk attempts per 10 minutes with Cedar and step-up gates.
Target 37: per-tenant default read bucket >= 10,000 requests per minute in elastic paid contexts.
Target 38: per-tenant mutation bucket >= 500 safe mutations per hour in elastic paid contexts.
Target 39: per-tenant high-risk mutation bucket >= 200 per hour unless incident controls tighten it.
Target 40: OpenAPI availability SLO target >= 99.9 percent until production evidence supports a stricter contract.
Target 41: command availability SLO target >= 99.9 percent, consistent with `slos/command-availability.openslo.yaml`.
Target 42: audit completeness target = 100 percent for required operator actions.
Target 43: admin-action audit seal completeness target = 100 percent.
Target 44: cluster health freshness p99 <= 30 seconds.
Target 45: tenant isolation posture freshness p99 <= 30 seconds.
Target 46: evidence pack freshness p99 <= 60 seconds after export request.
Target 47: multi-region operator-visible state propagation p99 <= 100 ms for active-active cell handoff.
Target 48: cross-region recovery state propagation p99 <= 500 ms where network path allows.
Target 49: dashboard frontend first meaningful operator summary <= 1.5 seconds on managed broadband.
Target 50: dashboard frontend interactive update after filter change <= 300 ms for cached summaries.

### §3.1 Deployment-Context Overlays

Context overlay 01: `oyatie-public-cloud` must meet targets 01-50 under managed elasticity.
Context overlay 02: `oyatie-public-cloud` may exceed target 25 by scaling audit/event partitions horizontally.
Context overlay 03: `oyatie-public-cloud` should hold target 20 at <= 240 seconds for 100 GB evidence export with sufficient signer/storage capacity.
Context overlay 04: `guest-on-aws` must meet targets 01-24 when AWS account quotas and VPC/network capacity are sufficient.
Context overlay 05: `guest-on-aws` read telemetry pivots must respect CloudWatch regional quotas, including 500 rps for `GetMetricData` and 10 rps for dashboard APIs.
Context overlay 06: `guest-on-aws` automation execution targets must respect Systems Manager default 100 concurrent automations unless adaptive concurrency or quota increases are enabled.
Context overlay 07: `guest-on-aws` must expose queue-state overlays when Systems Manager-backed automations enter pending state.
Context overlay 08: `guest-on-oci` must meet targets 01-24 for paid and revenue-share deployments with adequate tenancy resources.
Context overlay 09: `guest-on-oci` OCI Always Free profile caps demo_trial throughput to fit 4 OCPU, 24 GB RAM, 200 GB block storage, 10 GB object storage, 10 GB archive, and 10 Mbps load-balancer assumptions from the canonical profile.
Context overlay 10: `guest-on-oci` OCI Always Free profile should target read throughput >= 100 rps, burst >= 300 rps for 30 seconds, and high-risk mutation throughput >= 20 per hour until measured higher.
Context overlay 11: `guest-on-oci` OCI Always Free profile should cap evidence export payloads by usage policy so 10 GB object/archive assumptions are not exceeded.
Context overlay 12: `on-prem` target attainment depends on customer-provided hardware, storage, HSM or equivalent key custody, and network latency.
Context overlay 13: `on-prem` must publish measured ceiling values during admission and must not inherit elastic public-cloud numbers without evidence.
Context overlay 14: `colo` must meet targets 01-50 for planned Oyatie cells, except where remote-hands or facility network constraints are explicitly measured.
Context overlay 15: `colo` evidence export throughput must disclose facility storage and signer throughput.
Context overlay 16: `oyatie-as-cloud-provider` should meet or exceed all canonical targets because it is the provider substrate being sold.
Context overlay 17: `oyatie-as-cloud-provider` must expose control-plane saturation as a first-class customer-facing service quota.
Context overlay 18: every context must report OS, architecture, instance/node class, storage path, signer path, and network path with benchmark results.
Context overlay 19: every context must report rate-limit bucket settings separately for operator, tenant, emergency, and integration access.
Context overlay 20: every context must include failure-injection measurements for audit-chain outage, Cedar latency, signer latency, and observability sink outage.

### §3.2 Tenant-Class Overlays

Tenant overlay 01: `demo_trial` runs on free or constrained infrastructure when selected, commonly the OCI Always Free profile.
Tenant overlay 02: `demo_trial` receives the same safety checks, Cedar gates, step-up checks, audit completeness, and evidence integrity semantics as other classes.
Tenant overlay 03: `demo_trial` may cap read throughput at 100 rps and burst at 300 rps on OCI Always Free profile.
Tenant overlay 04: `demo_trial` may cap high-risk mutations at 20 per hour on constrained infrastructure.
Tenant overlay 05: `demo_trial` may cap evidence-export size and retention window to stay within free-profile storage.
Tenant overlay 06: `demo_trial` SLO is best effort unless promoted to a paid contract, but audit completeness target remains 100 percent.
Tenant overlay 07: `paid` scales with contract, payment, context capacity, compliance packs, and BYOK allowance.
Tenant overlay 08: `paid` should meet the full target set in all adequately provisioned contexts.
Tenant overlay 09: `paid` contractual SLO must bind availability, latency, evidence freshness, and support response.
Tenant overlay 10: `paid` may request higher rate limits after cost and risk review.
Tenant overlay 11: `paid` BYOK and compliance-pack overlays must not increase operator command p99 above 500 ms without contract-specific disclosure.
Tenant overlay 12: `revenue_share` runs at cost or zero-margin substrate, but safety and evidence quality remain uniform.
Tenant overlay 13: `revenue_share` rate limits should be shaped by gross-revenue model, marketplace traffic, and substrate cost caps.
Tenant overlay 14: `revenue_share` must expose cost saturation and margin guardrails to FinOps without weakening operator action safety.
Tenant overlay 15: `revenue_share` should meet full targets when revenue volume justifies substrate scale.
Tenant overlay 16: all tenant classes must include tenant-class in performance evidence records.
Tenant overlay 17: all tenant classes must include deployment context in performance evidence records.
Tenant overlay 18: all tenant classes must include OS and architecture in performance evidence records.
Tenant overlay 19: no tenant class may use weaker audit emission or weaker tenant isolation as a cost-saving measure.
Tenant overlay 20: no tenant class may use retired commercial tenant_class labels as benchmark rows or headings.

## §4 Comparison Narrative

Comparison 01: ODCC target read p99 of 250 ms is ahead of PagerDuty's public REST throughput posture but must be proven with implementation tests.
Comparison 02: ODCC target read p99 of 250 ms is a catch-up target against Datadog dashboard and incident workflows because Datadog has mature production console behavior.
Comparison 03: ODCC target read p99 of 250 ms is a parity target against AWS console/API expectations for regional control-plane queries.
Comparison 04: ODCC mutation p99 of 500 ms is intentionally stricter than many human-in-the-loop incident workflows.
Comparison 05: ODCC mutation p99 of 500 ms is only meaningful if audit-chain, Cedar, step-up, and idempotency are included in the measured path.
Comparison 06: ODCC audit event target of 100,000 events per minute per cell is below Datadog's 250,000 events per minute per organization for event submission, so single-cell event ingestion is catch-up.
Comparison 07: ODCC multi-cell audit target should exceed Datadog's 250,000 events per minute by aggregation, so the fleet-level target is ahead if partitioning is implemented.
Comparison 08: ODCC sustained read target of 1,000 rps per cell exceeds PagerDuty's public 16 rps per key/API-token number, but PagerDuty uses multiple identity buckets and is not positioned as a raw read API.
Comparison 09: ODCC sustained read target of 1,000 rps per cell exceeds CloudWatch `GetMetricData` 500 rps per Region for a single AWS account, but AWS also offers high datapoint throughput and quota increases.
Comparison 10: ODCC dashboard mutation target should respect AWS-like lower mutation quotas; dashboard updates are not a high-throughput path.
Comparison 11: ODCC command queue redline of 500 pending commands is more conservative than AWS Systems Manager's 5,000 automation queue, because ODCC commands are operator-visible safety actions.
Comparison 12: ODCC automation concurrency target must be expressed after implementation; AWS has a public 100 concurrent automation default and 500 adaptive ceiling.
Comparison 13: ODCC should target at least 100 concurrent mediated automations in elastic contexts to match AWS default automation posture.
Comparison 14: ODCC should target at least 25 concurrent rate-controlled high-risk workflows to match AWS rate-control automation posture.
Comparison 15: ODCC evidence export target of 100 GB in 240 seconds is aggressive and must be proven with signer and storage benchmarks.
Comparison 16: ODCC evidence export is a differentiator because none of the three counterparts maps exactly to pack-bound, signed, policy-aware Oyatie evidence exports.
Comparison 17: ODCC step-up server-side p99 of 500 ms is a parity target for secure operator workflows, while human interaction remains outside server latency.
Comparison 18: ODCC Cedar p99 target of 5 ms is required by local capacity model and is a differentiator if achieved under load.
Comparison 19: ODCC tenant posture freshness p99 of 30 seconds is necessary to compete with Datadog and AWS real-time operational views.
Comparison 20: ODCC cluster health freshness p99 of 30 seconds is a parity target against CloudWatch and Datadog infrastructure dashboards.
Comparison 21: ODCC multi-region state propagation p99 of 100 ms is ahead of many generic incident tools, but it requires measured replication proof.
Comparison 22: ODCC rate-limit headers should match or exceed Datadog and PagerDuty transparency by including bucket name, retry-after, and scope.
Comparison 23: ODCC demo_trial on OCI Always Free profile is expected to be throughput-constrained but must keep safety semantics identical.
Comparison 24: ODCC paid tenants should meet the full target set when provisioned in elastic contexts.
Comparison 25: ODCC revenue_share tenants should meet the full target set when substrate scale is funded by the revenue-share economics.
Comparison 26: ODCC guest-on-aws deployments must disclose AWS service quota constraints instead of presenting managed-cloud elastic numbers.
Comparison 27: ODCC on-prem deployments must publish measured local ceilings because facility and hardware constraints can dominate the target set.
Comparison 28: ODCC colo deployments must publish facility network and signer constraints.
Comparison 29: ODCC public-cloud and Oyatie-provider contexts should be the reference measurement lanes.
Comparison 30: ODCC cannot claim the target set today because no local source, tests, context modules, OS manifest, or CI evidence exist under this path.

## Benchmark Gate Summary

Gate 01: implement or link Rust crates for the manifest bounded contexts before measuring server latencies.
Gate 02: add tests for incident declaration, deployment approval, rollback decision, cluster health read, tenant posture read, policy decision review, and evidence export initiation.
Gate 03: add load tests for 1,000 rps sustained reads and 5,000 rps burst reads.
Gate 04: add mutation load tests for 200 high-risk actions per hour and 50 emergency actions per minute.
Gate 05: add Cedar latency benchmarks under realistic policy and entity cardinality.
Gate 06: add step-up server-side benchmarks separate from human authenticator time.
Gate 07: add audit-chain emission benchmarks and backpressure behavior.
Gate 08: add command queue depth and drain-time tests.
Gate 09: add evidence export initiation and 100 GB stream benchmarks with signer/storage disclosure.
Gate 10: add context-specific OpenTofu modules before claiming deployment overlays.
Gate 11: add `supported-oses.json` and CI lanes before claiming OS portability.
Gate 12: add tenant-class fields to benchmark evidence records.
Gate 13: add context fields to benchmark evidence records.
Gate 14: add OS and architecture fields to benchmark evidence records.
Gate 15: add rate-limit and retry-after conformance tests.
Gate 16: add AWS guest tests that simulate CloudWatch and Systems Manager quota ceilings.
Gate 17: add OCI Always Free profile tests that respect 4 OCPU, 24 GB RAM, and 10 Mbps load-balancer caps.
Gate 18: add on-prem admission benchmark template for customer facility measurement.
Gate 19: add colo benchmark template for facility network, signer, and storage constraints.
Gate 20: publish measured results only after source, tests, IaC, and OS evidence exist.

## Measurement Record Schema

Record field 01: `benchmark_id` identifies the workload and metric.
Record field 02: `service` must equal `ops-dashboard-control-center`.
Record field 03: `deployment_context` must be one of the six canonical contexts.
Record field 04: `tenant_class` must be `demo_trial`, `paid`, or `revenue_share`.
Record field 05: `os_id` records the operating system under test.
Record field 06: `arch` records linux/amd64, linux/arm64, or an approved soft-gate architecture.
Record field 07: `cell_id` records the cell or facility boundary.
Record field 08: `workload_shape` records read-heavy, mutation-heavy, export-heavy, or automation-heavy shape.
Record field 09: `p50_ms`, `p95_ms`, and `p99_ms` record latency distributions.
Record field 10: `throughput_rps` records sustained request rate for API workloads.
Record field 11: `burst_rps` records burst throughput and burst duration.
Record field 12: `queue_depth_max` records command queue pressure.
Record field 13: `audit_events_per_minute` records audit-chain emission throughput.
Record field 14: `cedar_p99_ms` records authorization budget compliance.
Record field 15: `step_up_server_p99_ms` records server-side challenge and verification cost.
Record field 16: `evidence_export_mb_per_second` records export stream throughput.
Record field 17: `rate_limit_bucket` records operator, tenant, emergency, or integration bucket.
Record field 18: `source_commit` records implementation revision.
Record field 19: `iac_module_digest` records OpenTofu module provenance.
Record field 20: `verdict` records pass, fail, or blocked with cited reason.
