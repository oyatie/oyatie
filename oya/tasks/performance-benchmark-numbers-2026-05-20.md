# tasks performance benchmark numbers - 2026-05-20

Audit owner: solo Codex audit lane for `microservices/tasks`.
Target microservice: `tasks`.
Counterpart set: Linear, Jira Software, Asana.
Benchmark model: single industry-leader target set with deployment-context overlays and tenant_class overlays.
Retired model notice: this report does not segment targets by the old capability ladder.
Methodology disclosure: public counterpart documents publish API limits and guardrails more reliably than end-user UI latency numbers, so UI latency values below use existing local benchmark estimates and are marked as estimated.
Methodology disclosure: Oyatie target numbers are canonical audit targets, not measured production results, because there is no buildable tasks source tree or load-test harness under `microservices/tasks`.
Methodology disclosure: all target numbers require future validation through Rust service tests, OpenSLO checks, load tests, dashboard evidence, and deployment-context OpenTofu plans.

Citation anchor 1: `microservices/tasks/PRD.md:289` through `microservices/tasks/PRD.md:302` defines current local performance targets.
Citation anchor 2: `microservices/tasks/capacity-model.md:37` through `microservices/tasks/capacity-model.md:49` defines per-cell capacity targets.
Citation anchor 3: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:18` through `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:21` gives existing local board-render estimates.
Citation anchor 4: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:33` through `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:36` gives existing local bulk-update estimates.
Citation anchor 5: `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139` through `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:142` requires the Batch 3.2 single-target performance form.
Citation anchor 6: Linear rate-limit documentation at `https://linear.app/developers/rate-limiting` provides public request and complexity limits.
Citation anchor 7: Asana rate-limit documentation at `https://developers.asana.com/docs/rate-limits` provides public request, concurrency, and job limits.
Citation anchor 8: Jira Cloud rate-limit documentation at `https://developer.atlassian.com/cloud/jira/platform/rate-limiting/` provides quota, burst, cost, and header semantics.
Citation anchor 9: Jira Cloud data-limit and guardrail documentation at `https://support.atlassian.com/jira-cloud-administration/docs/data-limits-and-guardrails/` provides scale-guardrail context.
Citation anchor 10: `specs/master-plan-sequencing.json:857` through `specs/master-plan-sequencing.json:868` defines OCI Always Free profile constraints.

## §1 Methodology

Benchmark dimension 1: task read latency at p50, p95, and p99.
Benchmark dimension 2: task create latency at p50, p95, and p99.
Benchmark dimension 3: task update latency at p50, p95, and p99.
Benchmark dimension 4: board or list render latency for 10,000 visible work items.
Benchmark dimension 5: bulk edit duration for 30,000 task updates.
Benchmark dimension 6: dependency-cycle check latency for a large dependency graph.
Benchmark dimension 7: search freshness from write commit to searchable result.
Benchmark dimension 8: API throughput in requests per second.
Benchmark dimension 9: concurrent GET and write operations.
Benchmark dimension 10: importer throughput under counterpart API limits.
Benchmark dimension 11: webhook fanout latency.
Benchmark dimension 12: recurrence generation lag.
Benchmark dimension 13: portfolio or roadmap rollup latency.
Benchmark dimension 14: automation rule execution latency.
Benchmark dimension 15: error-budget burn and availability target.
Benchmark dimension 16: resource ceiling under constrained deployment profiles.
Benchmark workload 1: single task create with project, assignee, status, and due date.
Benchmark workload 2: task update with custom fields and audit projection.
Benchmark workload 3: dependency edge create with cycle check.
Benchmark workload 4: 10,000-item board/list render query.
Benchmark workload 5: 30,000-row bulk status update.
Benchmark workload 6: 250,000-task full search reindex.
Benchmark workload 7: 50,000-task Jira import with idempotency and backoff.
Benchmark workload 8: 50,000-task Asana import with idempotency and backoff.
Benchmark workload 9: 25,000-task Linear import with issue relationship preservation.
Benchmark workload 10: 10,000 webhook deliveries to mixed subscribers.
Benchmark workload 11: 100,000 recurrence rule evaluations.
Benchmark workload 12: 5,000-project portfolio rollup.
Benchmark OS baseline: service targets must eventually be proven across the canonical OS matrix, but no tasks `supported-oses.json` exists yet.
Benchmark architecture baseline: x86_64 and arm64 server paths should be tested once the Rust workspace exists.
Benchmark deployment context 1: `oyatie-public-cloud` targets elastic scale and full SLO enforcement.
Benchmark deployment context 2: `guest-on-aws` targets customer-account deployment with cloud-specific quota overlays.
Benchmark deployment context 3: `guest-on-oci` targets customer-account deployment and includes the OCI Always Free profile.
Benchmark deployment context 4: `on-prem` targets customer-managed hardware with facility-specific ceiling disclosure.
Benchmark deployment context 5: `colo` targets co-location with network and storage ceiling disclosure.
Benchmark deployment context 6: `oyatie-as-cloud-provider` targets provider-owned infrastructure with high elasticity.
Benchmark tenant_class 1: `demo_trial` uses hard usage caps and best-effort SLO while preserving the same feature-quality bar.
Benchmark tenant_class 2: `paid` uses composable billing components drawn from `revenue_share`, `per_seat`, and `per_usage`, and scales with paid capacity.
Benchmark evidence caveat: no cargo tests, load-test harness, or dashboards can currently prove these Oyatie target numbers.
Benchmark evidence caveat: existing local benchmark docs contain retired vocabulary, so their numbers are historical estimates only.
Benchmark evidence caveat: counterpart API limits are not equivalent to UI latency, but they constrain importers, sync jobs, and integrations.
Benchmark acceptance rule: a future implementation can claim a number only after measured p50, p95, p99, throughput, and error data are attached.

## §2 Counterpart numbers

### §2.1 Linear numbers

Linear number 1: unauthenticated API budget is 600 requests per hour; source: Linear public rate-limit documentation.
Linear number 2: API-key request budget is documented in the public table as 2,500 requests per hour; source: Linear public rate-limit documentation.
Linear number 3: OAuth-app request budget is documented as 5,000 requests per hour; source: Linear public rate-limit documentation.
Linear number 4: unauthenticated complexity budget is 100,000 complexity units per hour; source: Linear public rate-limit documentation.
Linear number 5: API-key complexity budget is 3,000,000 complexity units per hour; source: Linear public rate-limit documentation.
Linear number 6: OAuth-app complexity budget is 2,000,000 complexity units per hour; source: Linear public rate-limit documentation.
Linear number 7: maximum complexity for one query is 10,000; source: Linear public rate-limit documentation.
Linear number 8: board render p50 estimate is 380 ms for the local comparison workload; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:18`.
Linear number 9: board render p95 estimate is 980 ms for the local comparison workload; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:18`.
Linear number 10: board render frame-rate estimate is 60 fps; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:18`.
Linear number 11: bulk update estimate is 9 seconds for 30,000 tasks; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:36`.
Linear number 12: bulk update throughput estimate is 3,333 rows per minute; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:36`.
Linear number 13: dependency cycle-check estimate is 18 ms; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:49`.
Linear number 14: portfolio cold query estimate is 1.4 seconds; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:62`.
Linear number 15: portfolio warm query estimate is 0.6 seconds; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:62`.
Linear interpretation: Linear sets the speed bar for issue-first engineering workflows.
Linear interpretation: importers must respect Linear request and complexity budgets independently.
Linear interpretation: Oyatie should beat the local Linear estimates where local targets control the workload.
Linear interpretation: Oyatie should not copy Linear's API shape if OpenAPI/proto/AsyncAPI remain canonical locally.

### §2.2 Jira Software numbers

Jira number 1: rate-limit responses can be quota-based, burst-based, or cost-based; source: Jira Cloud public rate-limit documentation.
Jira number 2: a documented quota example is 65,000 requests per 3,600 seconds; source: Jira Cloud public rate-limit documentation.
Jira number 3: a documented burst example is 100 requests per 1 second; source: Jira Cloud public rate-limit documentation.
Jira number 4: near-limit signaling begins below 20 percent remaining budget; source: Jira Cloud public rate-limit documentation.
Jira number 5: retry timing is expressed through `Retry-After` and reset headers on constrained requests; source: Jira Cloud public rate-limit documentation.
Jira number 6: local playbook states a Jira Cloud importer hard cap of 1,000 requests per minute; source: `microservices/tasks/migration-playbooks/from-jira-cloud.md:128`.
Jira number 7: local playbook states an importer sustained rate of 600 requests per minute; source: `microservices/tasks/migration-playbooks/from-jira-cloud.md:128`.
Jira number 8: board render p50 estimate is 920 ms for the local comparison workload; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:20`.
Jira number 9: board render p95 estimate is 2,400 ms for the local comparison workload; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:20`.
Jira number 10: board render frame-rate estimate is 38 fps; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:20`.
Jira number 11: bulk update estimate is 18 seconds for 30,000 tasks; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:35`.
Jira number 12: bulk update throughput estimate is 1,667 rows per minute; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:35`.
Jira number 13: dependency cycle-check estimate is 78 ms; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:48`.
Jira number 14: portfolio cold query estimate is 7.1 seconds; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:61`.
Jira number 15: portfolio warm query estimate is 3.2 seconds; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:61`.
Jira interpretation: Jira sets the enterprise configurability and scale-guardrail bar.
Jira interpretation: Oyatie should be stricter than Jira about explicit backoff and rate-budget telemetry.
Jira interpretation: Jira importer numbers should be validated against actual API responses during implementation.
Jira interpretation: local Jira estimates are useful comparative targets but not production measurements.

### §2.3 Asana numbers

Asana number 1: paid domains allow 1,500 requests per minute per access token authorization; source: Asana public rate-limit documentation.
Asana number 2: free domains allow 150 requests per minute per access token authorization; source: Asana public rate-limit documentation.
Asana number 3: search API usage is limited to 60 requests per minute; source: Asana public rate-limit documentation.
Asana number 4: concurrent GET requests are limited to 50; source: Asana public rate-limit documentation.
Asana number 5: concurrent write requests are limited to 15; source: Asana public rate-limit documentation.
Asana number 6: concurrent jobs are limited to 5; source: Asana public rate-limit documentation.
Asana number 7: local tutorial cites 1,500 requests per minute as an Asana throughput claim; source: `microservices/tasks/tutorials/migrate-asana-project.md:104`.
Asana number 8: board render p50 estimate is 680 ms for the local comparison workload; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:19`.
Asana number 9: board render p95 estimate is 1,850 ms for the local comparison workload; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:19`.
Asana number 10: board render frame-rate estimate is 45 fps; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:19`.
Asana number 11: bulk update estimate is 78 seconds for 30,000 tasks; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:34`.
Asana number 12: bulk update throughput estimate is 385 rows per minute; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:34`.
Asana number 13: dependency cycle-check estimate is 280 ms; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:47`.
Asana number 14: portfolio cold query estimate is 4.2 seconds; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:60`.
Asana number 15: portfolio warm query estimate is 1.8 seconds; source: `microservices/tasks/benchmarks/tasks-vs-asana-jira-linear-monday.md:60`.
Asana interpretation: Asana sets the broad work-management and importer-concurrency bar.
Asana interpretation: Asana search limits are especially important for migration and sync design.
Asana interpretation: Oyatie should target lower UI latency and higher bulk throughput than local Asana estimates.
Asana interpretation: demo_trial importers must cap usage without degrading correctness.

## §3 Oyatie target numbers - single industry-leader set

Target 1: task read latency canonical target is p50 40 ms, p95 120 ms, p99 250 ms.
Target 1 overlay: `oyatie-public-cloud` can autoscale read replicas to preserve p99 250 ms.
Target 1 overlay: `guest-on-aws` and `guest-on-oci` preserve the target until account quotas constrain replicas.
Target 1 overlay: OCI Always Free profile caps concurrent readers rather than relaxing correctness.
Target 1 overlay: `on-prem` and `colo` publish facility ceilings if storage or network cannot sustain target load.
Target 1 tenant overlay: `demo_trial` caps read volume; `paid` scales with paid usage and any contracted billing components.

Target 2: task create latency canonical target is p50 70 ms, p95 180 ms, p99 400 ms.
Target 2 overlay: public cloud contexts may scale write partitions automatically.
Target 2 overlay: customer-controlled contexts must expose write-quota saturation before p99 breaches.
Target 2 tenant overlay: demo_trial limits write rate; paid preserves latency with provisioned write capacity.

Target 3: task update latency canonical target is p50 60 ms, p95 160 ms, p99 350 ms.
Target 3 overlay: cross-region write acknowledgement may add context-specific p99 disclosure.
Target 3 overlay: on-prem storage controller limits must be reported as a deployment overlay.
Target 3 tenant overlay: demo_trial throttles update bursts; paid scales by contracted capacity.

Target 4: dependency edge create with cycle check canonical target is p50 5 ms, p95 15 ms, p99 40 ms for common graph sizes.
Target 4 overlay: very large enterprise graphs must publish graph-size thresholds.
Target 4 overlay: constrained OCI Always Free profile caps graph size for demo_trial tenants.
Target 4 tenant overlay: all tenant classes get the same cycle correctness guarantee.

Target 5: task list or board render backend query canonical target is p50 150 ms, p95 350 ms, p99 700 ms for 10,000 visible work items.
Target 5 overlay: public cloud and provider contexts preserve target through elastic read/query replicas.
Target 5 overlay: guest and on-prem contexts disclose storage and cache ceilings.
Target 5 tenant overlay: demo_trial caps project size; paid scales project size by purchased or agreed capacity.

Target 6: UI frame target is stable 60 fps during normal board interaction.
Target 6 overlay: web, Swift, Kotlin, and WinUI3 clients must each prove this separately when built.
Target 6 overlay: remote or thin-client contexts must disclose network-dependent frame variance.
Target 6 tenant overlay: no tenant class gets degraded interaction quality.

Target 7: bulk update canonical target is 30,000 task updates in 2.5 seconds after admission.
Target 7 overlay: customer-controlled deployments may use admission limits to prevent storage overload.
Target 7 overlay: OCI Always Free profile caps batch size rather than changing algorithmic correctness.
Target 7 tenant overlay: demo_trial has smaller maximum batch size; paid scales batch size with capacity.

Target 8: bulk update throughput canonical target is 12,000 updates per second inside the service boundary.
Target 8 overlay: external API importers remain limited by counterpart API budgets.
Target 8 overlay: on-prem and colo deployments disclose network and disk ceilings.
Target 8 tenant overlay: paid can buy or negotiate higher sustained throughput through billing components and capacity policy.

Target 9: search freshness canonical target is p95 2 seconds and p99 8 seconds from committed write to searchable result.
Target 9 overlay: disconnected on-prem deployments may publish local search rebuild windows.
Target 9 overlay: public cloud contexts should maintain p99 8 seconds through indexer autoscaling.
Target 9 tenant overlay: demo_trial caps index size; paid scales index partitions.

Target 10: saved-view query canonical target is p50 120 ms, p95 300 ms, p99 650 ms.
Target 10 overlay: custom field cardinality must be included in context-specific query budgets.
Target 10 overlay: customer-managed storage limits may reduce admitted view complexity.
Target 10 tenant overlay: quality is uniform; admitted view complexity varies by usage budget.

Target 11: portfolio rollup canonical target is p50 250 ms, p95 700 ms, p99 1.5 seconds for cached organization rollups.
Target 11 overlay: cold rollups may take up to 2.5 seconds when cache is empty.
Target 11 overlay: public cloud contexts should precompute heavy rollups.
Target 11 tenant overlay: demo_trial caps portfolio count; paid scales precompute resources.

Target 12: recurrence generation canonical target is p95 60 seconds from scheduled due time to generated task.
Target 12 overlay: disconnected deployments report queue lag locally.
Target 12 overlay: public cloud contexts should keep p99 under 180 seconds during regional failover.
Target 12 tenant overlay: all tenant classes keep recurrence correctness; volume caps differ.

Target 13: webhook fanout canonical target is p95 500 ms to enqueue and p99 5 seconds to first delivery attempt.
Target 13 overlay: external endpoint slowness is isolated from task write latency.
Target 13 overlay: on-prem egress rules may require deployment-specific delivery queues.
Target 13 tenant overlay: demo_trial caps subscription count; paid scales fanout.

Target 14: import from Asana canonical target is 1,200 requests per minute sustained without violating Asana paid-domain rate limits.
Target 14 overlay: free-domain imports must obey the 150 requests per minute public limit.
Target 14 overlay: search-heavy imports obey Asana's 60 requests per minute search limit.
Target 14 tenant overlay: demo_trial imports cap total objects; paid supports larger staged imports.

Target 15: import from Jira canonical target is 600 requests per minute sustained unless live headers demand lower backoff.
Target 15 overlay: importer must respond to Jira quota, burst, and cost headers.
Target 15 overlay: deployment context does not override external API limits.
Target 15 tenant overlay: demo_trial imports cap total object count; paid supports larger migration windows.

Target 16: import from Linear canonical target is 2,000 requests per hour per API key until Linear headers require backoff.
Target 16 overlay: complexity budget must be tracked alongside request budget.
Target 16 overlay: deployment context does not override external API limits.
Target 16 tenant overlay: demo_trial imports cap project count; paid supports scheduled migration jobs.

Target 17: automation rule execution canonical target is p95 1 second for synchronous rule effects and p99 30 seconds for asynchronous fanout effects.
Target 17 overlay: constrained contexts may cap rules per project.
Target 17 overlay: public cloud contexts should isolate rule queues by tenant.
Target 17 tenant overlay: quality is uniform; admitted rule count differs by usage cap.

Target 18: task-create availability canonical target is 99.95 percent monthly for paid production contexts.
Target 18 overlay: demo_trial uses best-effort SLO while preserving correctness.
Target 18 overlay: on-prem and colo require facility-specific SLO contracts.
Target 18 tenant overlay: class changes SLO contract, not feature correctness.

Target 19: read throughput canonical target is 30,000 task fetch requests per second per production cell.
Target 19 evidence: this matches local capacity model at `microservices/tasks/capacity-model.md:39`.
Target 19 overlay: OCI Always Free profile uses a lower admitted throughput ceiling based on available OCPU and memory.
Target 19 tenant overlay: demo_trial enforces hard usage caps; paid scales cells.

Target 20: write throughput canonical target is 5,000 task writes per second per production cell.
Target 20 evidence: this matches local capacity model at `microservices/tasks/capacity-model.md:40`.
Target 20 overlay: customer-managed storage must prove fsync and replication behavior.
Target 20 tenant overlay: demo_trial write bursts are capped; paid scales partitions.

Target 21: status-update throughput canonical target is 20,000 status updates per second per production cell.
Target 21 evidence: this matches local capacity model at `microservices/tasks/capacity-model.md:41`.
Target 21 overlay: bulk operations must be admitted separately from interactive updates.
Target 21 tenant overlay: demo_trial caps bulk operations; paid uses paid capacity.

Target 22: importer idempotency target is zero duplicate committed tasks across retry storms.
Target 22 overlay: all deployment contexts must preserve idempotency across process restarts.
Target 22 tenant overlay: all tenant classes get identical idempotency guarantees.
Target 22 evidence: Rust reference implementation exists at `microservices/tasks/reference-implementations/rust/importer-idempotency.rs`.

Target 23: event publication target is p99 2 seconds from task commit to event visible on the internal bus.
Target 23 overlay: offline on-prem deployments disclose queue sync lag separately.
Target 23 tenant overlay: demo_trial caps event volume; paid scales queues.
Target 23 evidence gap: AsyncAPI exists, but runtime event publication is not implemented.

Target 24: API error-rate target is below 0.1 percent for valid requests during steady state.
Target 24 overlay: backpressure responses are counted separately when admission controls are working as designed.
Target 24 tenant overlay: usage-cap rejections must be explicit and auditable.
Target 24 evidence gap: no service implementation exists to measure this.

Target 25: p99 dashboard query latency target is 2 seconds for operational dashboards backed by pre-aggregated metrics.
Target 25 overlay: public cloud contexts should preserve target during normal region load.
Target 25 tenant overlay: demo_trial dashboard history can be shorter; paid retains longer windows.
Target 25 evidence: dashboard JSON files exist, but no deployed metric pipeline is evidenced.

## §4 Comparison narrative

Headline 1: task read and write latency targets are ahead of the local counterpart estimates.
Headline 1 basis: Oyatie p95 read target 120 ms is below the local board-render p95 estimates for Linear, Jira, and Asana.
Headline 1 caveat: board render and task read are not identical workloads.
Headline 1 evidence gap: no implementation exists to measure the target.

Headline 2: board/list target is parity-to-ahead against Linear and ahead of Jira and Asana local estimates.
Headline 2 basis: Oyatie p95 board backend query target is 350 ms against local Linear 980 ms, Asana 1,850 ms, and Jira 2,400 ms estimates.
Headline 2 caveat: UI frame rate must be measured separately on the allowed frontend stacks.
Headline 2 required validation: measure backend query, client render, and interaction frame stability separately.

Headline 3: bulk update target is materially ahead of all three local counterpart estimates.
Headline 3 basis: Oyatie target 2.5 seconds for 30,000 updates beats local Linear 9 seconds, Jira 18 seconds, and Asana 78 seconds estimates.
Headline 3 caveat: admission, audit, event publication, and search reindex must all be included in the measured workload.
Headline 3 required validation: load-test interactive and queued bulk paths separately.

Headline 4: dependency cycle-check target is ahead of local counterpart estimates.
Headline 4 basis: Oyatie p95 15 ms and p99 40 ms are at or ahead of the local 18 ms Linear estimate and ahead of Jira and Asana estimates.
Headline 4 caveat: graph size and edge density must be specified in the test harness.
Headline 4 required validation: include cold-cache, warm-cache, and pathological graph cases.

Headline 5: Asana importer target is parity with paid-domain public API limits and cautious for search.
Headline 5 basis: target 1,200 requests per minute is below Asana's 1,500 requests per minute paid-domain public limit.
Headline 5 caveat: search-heavy imports must obey the 60 requests per minute search limit.
Headline 5 required validation: importer must dynamically back off from headers and response classes.

Headline 6: Jira importer target is cautious against local playbook guidance.
Headline 6 basis: target 600 requests per minute sustained matches the local playbook's sustained value.
Headline 6 caveat: Jira public rate limits use quota, burst, and cost dimensions that vary by request.
Headline 6 required validation: backoff logic must consume live headers, not hard-code one budget.

Headline 7: Linear importer target is deliberately below the documented API-key request budget.
Headline 7 basis: target 2,000 requests per hour leaves margin below the public table's API-key request budget.
Headline 7 caveat: complexity budget can be exhausted before request budget.
Headline 7 required validation: importer must track request and complexity budgets together.

Headline 8: throughput targets match the existing local capacity model but are not yet proven.
Headline 8 basis: read, write, and status-update values match `capacity-model.md`.
Headline 8 caveat: capacity numbers need Rust benchmarks and deployment-context OpenTofu plans.
Headline 8 required validation: test per-cell capacity, cross-cell routing, and tenant admission separately.

Headline 9: demo_trial constrains usage, not quality.
Headline 9 basis: current doctrine requires uniform industry-leader quality across tenant classes.
Headline 9 caveat: OCI Always Free profile creates infrastructure ceilings.
Headline 9 required validation: demo caps must be expressed as quotas and admission decisions, not feature removals.

Headline 10: paid tenant_class scales through capacity policy and billing components, not different feature promises.
Headline 10 basis: the current dispatch defines paid billing components separately from feature quality.
Headline 10 caveat: cost-budget docs still use old customer classes and need rewriting.
Headline 10 required validation: billing, usage, and deployment context metadata must be present in runtime events.

Headline 11: public-cloud and provider contexts can target the full elastic numbers first.
Headline 11 basis: those contexts can be controlled by Oyatie infrastructure.
Headline 11 caveat: no OpenTofu context modules exist yet.
Headline 11 required validation: plan/apply evidence, load evidence, and dashboard evidence per context.

Headline 12: customer-controlled contexts require explicit ceiling disclosure.
Headline 12 basis: on-prem, colo, guest-on-aws, and guest-on-oci depend on customer quotas, hardware, and network.
Headline 12 caveat: the product should still expose the same behavior under admitted workloads.
Headline 12 required validation: context modules must encode resource assumptions and expose observability.

Headline 13: the current local benchmark doc should be treated as historical evidence.
Headline 13 basis: it contains old segmentation vocabulary and old hardware framing.
Headline 13 caveat: its numeric estimates are still useful as comparative workload seeds.
Headline 13 required validation: replace it with reproducible tests and measured artifacts.

Headline 14: no number in this report is a shipped performance claim.
Headline 14 basis: tasks has no buildable source tree or tests.
Headline 14 caveat: the targets are intentionally aggressive because the counterpart bar is industry-leader-grade.
Headline 14 required validation: a future implementation must attach raw run output, environment, git SHA, and deployment context.

Headline 15: the immediate performance blocker is substrate, not target selection.
Headline 15 basis: OpenTofu, OS support, Rust workspace, and load harness are absent.
Headline 15 caveat: target selection still matters because it prevents under-building the service.
Headline 15 required validation: build the substrate before claiming any target has been met.
