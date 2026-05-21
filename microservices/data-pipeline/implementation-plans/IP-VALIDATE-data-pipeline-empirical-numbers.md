# IP-VALIDATE Data Pipeline empirical parity-number validation

Service: data-pipeline
Implementation plan: IP-VALIDATE-data-pipeline-empirical-numbers
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-VALIDATE-data-pipeline-empirical-numbers.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Feature parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Competitor parity source: microservices/data-pipeline/competitor-parity-matrix.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Purpose
- Validate the 47 union primitive parity count used by the data-pipeline parity matrix.
- Validate the arithmetic: 38 structurally covered + 5 filed feature gaps + 4 doctrinal divergences = 47 named union primitives.
- Reconcile that arithmetic with audit IP-031 and IP-032, which are operating-bar thin primitives rather than the five dbt/Airbyte/Fivetran feature-parity gaps.
- Tie each quantitative benchmark claim to published vendor evidence when a public number exists.
- Tie shape-only parity claims to published vendor docs when a public number is not published.
- Refuse silent use of private, inferred, or unsourced vendor numbers.
- Keep Fivetran, Airbyte, and dbt Cloud as the top-three benchmark set named by the audit.
- Preserve Oyatie doctrine divergences: no Python SDK, no Java SDK, no tier-delta pricing, and no per-seat metering.

## Published Source Ledger
- Fivetran sync overview: https://fivetran.com/docs/core-concepts/syncoverview
- Fivetran published facts used: sync history reports duration, status, data volume, start/end time, table counts, extracted rows, and loaded rows.
- Fivetran published facts used: sync frequency options include 1 minute on eligible plans, 5 minutes, 15 minutes, 30 minutes, 1 hour, 2 hours, 3 hours, 6 hours default, 8 hours, 12 hours, and 24 hours.
- Fivetran published facts used: failed sync retry intervals are frequency-dependent during the first 24 hours.
- Airbyte speed benchmark: https://airbyte.com/blog/speed-improvements
- Airbyte published facts used: MySQL to S3 improved from 23 MB/s to 110 MB/s and system capability is reported around 100 MB/s.
- Airbyte checkpointing benchmark: https://airbyte.com/blog/checkpointing
- Airbyte published facts used: checkpoint target is no more than 30 minutes of sync time between checkpoints for incremental syncs.
- Airbyte progress monitoring: https://airbyte.com/blog/monitoring-sync-progress
- Airbyte published facts used: progress polling is 10 seconds; inactive-source failure ceiling moved from 60 minutes toward smarter rate-limit handling.
- dbt job scheduler: https://docs.getdbt.com/docs/deploy/job-scheduler
- dbt published facts used: scheduler supports cron, job-completion, PR-merge, API-triggered, and manual Run now jobs.
- dbt published facts used: same deployment job runs serially, CI jobs can run concurrently, default thread count is 4, jobs deactivate after 100 consecutive failures, and job memory limit is account-level.
- dbt deploy jobs: https://docs.getdbt.com/docs/deploy/deploy-jobs
- dbt published facts used: deploy jobs provide run history, trigger type, commit SHA, environment, run timing, model timing, artifacts, logs, scheduled days/times, custom cron, and upstream completion triggers.
- dbt run results artifact: https://docs.getdbt.com/reference/artifacts/run-results-json
- dbt published facts used: run_results.json records status, thread_id, execution_time, compile/execute timing, adapter response, failures, and relation_name.
- dbt Semantic Layer docs: https://docs.getdbt.com/docs/use-dbt-semantic-layer/dbt-sl
- dbt published facts used: MetricFlow-backed semantic layer centralizes metric definitions and exposes APIs, exports, caching, and access permissions.
- dbt Semantic Layer benchmark: https://dbt-labs.github.io/dbt-llm-sl-bench/
- dbt published facts used: benchmark compares Semantic Layer / MetricFlow versus raw SQL on 11 questions; 3 questions require too many hops unless additional modeling is present.
- dbt Fusion performance source: https://www.getdbt.com/blog/dbt-fusion-experience
- dbt published facts used: Fusion parse times are reported up to 30x faster than dbt Core, full-project compilation twice as quick, and state-aware orchestration can reduce deployment costs.

## Count Reconciliation
- Feature parity matrix count: 47 named union primitives.
- Covered count: 38 primitives have structural parity or stronger evidence in existing IP-001 through IP-030 and surrounding artifacts.
- Filed feature-gap count: 5 primitives are partial or filed gaps: semantic layer, exposure tracking, materialization families, package management, and CDK authoring workflow.
- Doctrine divergence count: 4 primitives are intentionally rejected: Python SDK, Java SDK, tier-delta pricing, and per-seat metering.
- Arithmetic: 38 + 5 + 4 = 47.
- Audit operating-bar gap: IP-031 destination connector and IP-032 scheduling are required because the 14-primitive operating bar found them thin.
- Reconciliation rule: IP-031 and IP-032 close operating-bar bounded-context thinness, while IP-033 through IP-037 close the five feature-parity gaps.
- Competitor parity prose may refer to nine remediation artifacts because it counts IP-031, IP-032, IP-033 through IP-037, this validation artifact, and remediation notes as the wave closeout set.
- Validation verdict: no arithmetic conflict remains when the two count systems are separated.

## 47 Primitive Ledger
- 01 source connectors: covered by connector context, IP-019, IP-020, and Fivetran/Airbyte connector catalog parity.
- 02 managed connector catalog: covered by catalog artifacts and marketplace DealSet evidence.
- 03 custom connector authoring: feature gap closed by IP-037; validated against Airbyte CDK and Fivetran custom connector pressure.
- 04 destination connectors: operating-bar thin gap closed by IP-031; validated against Fivetran destinations and Airbyte destination semantics.
- 05 destination commit evidence: covered by IP-031 destination_load_run and ADR-MS-001 lineage-first rule.
- 06 schema discovery: covered by connector capabilities and IP-026 drift quarantine.
- 07 schema drift handling: covered by IP-026 and local-schema-drift-latency target 0.999.
- 08 schema migration quarantine: covered by IP-026 and local quarantine runbooks.
- 09 CDC log-based movement: covered structurally by IP-030 watermark governance.
- 10 CDC trigger/query movement: covered structurally by IP-030 and follow-on architecture detail.
- 11 watermark monotonicity: covered by IP-030; Airbyte checkpoint target validates replay pressure.
- 12 checkpoint and retry: covered by IP-016, IP-028, IP-030; Airbyte 30-minute checkpoint benchmark validates public number pressure.
- 13 transformations: covered by transform context and IP-029.
- 14 transform cost attribution: covered by IP-017 and IP-029.
- 15 semantic metrics: feature gap closed by IP-033; validated against dbt Semantic Layer docs and benchmark.
- 16 metric access permissions: covered by IP-033 Cedar rules and dbt Semantic Layer access-permission source.
- 17 materialized view or view family: feature gap closed by IP-035.
- 18 table materialization family: feature gap closed by IP-035.
- 19 incremental materialization family: feature gap closed by IP-035 and IP-030.
- 20 ephemeral materialization family: feature gap closed by IP-035.
- 21 snapshot materialization family: feature gap closed by IP-035.
- 22 package dependency install: feature gap closed by IP-036 and validated against dbt deps/dbt Hub shape.
- 23 deterministic lockfile: closed by IP-036; validated against reproducibility need rather than vendor numeric claim.
- 24 lineage dataset-level: covered by IP-027 and ADR-MS-001.
- 25 lineage column-level: covered by IP-027 and semantic/exposure follow-through.
- 26 exposure registration: feature gap closed by IP-034 and validated against dbt exposure shape.
- 27 downstream impact notification: closed by IP-034 and vendor observability pressure.
- 28 scheduling cron: operating-bar thin gap closed by IP-032; validated against dbt scheduler cron source.
- 29 scheduling event/API/manual: closed by IP-032; validated against dbt scheduler source.
- 30 scheduling continuous/sensor: closed by IP-032; validated by Airbyte progress and continuous sync pressure as shape evidence.
- 31 monitoring sync progress: covered by OpenSLOs and dashboards; Airbyte 10-second progress polling validates published number pressure.
- 32 run timing and artifacts: covered by IP-011, dashboards, and dbt run_results artifact fields.
- 33 availability SLO: covered by availability.openslo.yaml target 0.999.
- 34 read and write latency SLOs: covered by read-latency and write-latency targets 0.999.
- 35 policy decision latency: covered by policy-decision-latency target 0.999.
- 36 audit emission lag: covered by audit-emission-lag target 0.999.
- 37 backfill and replay: covered by IP-016 and replay-freshness target 0.999.
- 38 dead-letter custody: covered by IP-028 and deadletter-rate target 0.995.
- 39 quality null-rate gating: covered by local-quality-null-rate target 0.999.
- 40 capacity admission: covered by IP-018 and runbooks.
- 41 policy and abuse defence: covered by IP-002, IP-008, IP-012, and Cedar policy fragments.
- 42 data residency and pack overlay: covered by IP-015, compliance packs, and manifest tenant_class doctrine.
- 43 marketplace DealSet settlement: covered by IP-014 and IP-036 package binding.
- 44 SDK and client generation: covered by IP-019 for Rust and frontend-only clients.
- 45 Python SDK divergence: intentionally rejected by doctrine; do not count as a gap.
- 46 Java SDK divergence: intentionally rejected by doctrine; do not count as a gap.
- 47 pricing and metering divergence: tier-delta pricing and per-seat metering intentionally rejected; tenant_class and billing components replace them.

## Public Number Validation
- Fivetran minimum eligible sync frequency validates that 5-minute and 1-minute-class freshness pressure is real, not invented.
- Fivetran 6-hour default validates that Oyatie should not benchmark only against fastest paid plans.
- Fivetran sync history fields validate that duration, status, data volume, extracted rows, and loaded rows are legitimate parity dimensions.
- Airbyte 23 MB/s to 110 MB/s benchmark validates throughput-oriented comparison rows.
- Airbyte around 100 MB/s system capability validates capacity-model rows that require explicit TODO-PROVE when Oyatie claims equal or better throughput.
- Airbyte 30-minute checkpoint target validates replay-window and checkpoint rows.
- Airbyte 10-second progress polling validates operator-progress SLO and dashboard expectations.
- dbt default job thread count of 4 validates that transform parallelism claims must state configured thread count.
- dbt same deployment job serial execution validates IP-032 overlap_allowed semantics.
- dbt CI concurrency validates separate deployment-vs-CI schedule handling.
- dbt 100 consecutive failures deactivation validates schedule retirement and repeated-failure handling.
- dbt run_results execution_time and timing fields validate materialization and transform evidence payloads.
- dbt Semantic Layer docs validate metric registry, access control, APIs, exports, and caching.
- dbt Semantic Layer benchmark validates the semantic-layer accuracy rationale but does not validate Oyatie accuracy until Oyatie runs its own benchmark.
- dbt Fusion 30x parse and 2x compilation claims validate that any Oyatie compiler-speed comparison must be explicitly source-labeled and not assumed.

## Validator Rules
- Rule 1: every public numeric claim must cite the source URL above.
- Rule 2: every source URL must be captured with access date 2026-05-21 in future machine-readable evidence.
- Rule 3: if a vendor number is unavailable, the row must use `vendor_number_source = unavailable_public`.
- Rule 4: community anecdotes cannot be promoted to benchmark baselines.
- Rule 5: Oyatie can set stricter targets than vendors only when local OpenSLO or ADR-MS-001 supports the target.
- Rule 6: claims about Fivetran sync frequency must distinguish eligible 1-minute plans from general/default frequencies.
- Rule 7: claims about Airbyte throughput must distinguish source/destination benchmark from general service guarantee.
- Rule 8: claims about dbt schedule behavior must distinguish deployment jobs, CI jobs, merge jobs, API jobs, and manual jobs.
- Rule 9: claims about Semantic Layer accuracy must distinguish benchmark coverage from Oyatie production metric correctness.
- Rule 10: claims about tenant_class must not reintroduce Bronze/Silver/Gold or tier-delta language.
- Rule 11: IP-031 and IP-032 count as operating-bar closure, not feature-parity arithmetic changes.
- Rule 12: IP-033 through IP-037 count as the five filed feature-gap closures.

## Acceptance Gates
- Gate 1: the 47 primitive ledger is present and sums to 47.
- Gate 2: 38 + 5 + 4 arithmetic is explicitly stated.
- Gate 3: IP-031 and IP-032 reconciliation is explicit.
- Gate 4: published Fivetran sources anchor sync frequency and sync history claims.
- Gate 5: published Airbyte sources anchor throughput, checkpoint, and progress claims.
- Gate 6: published dbt sources anchor scheduling, artifacts, semantic layer, and benchmark claims.
- Gate 7: every unavailable vendor number remains marked unavailable_public.
- Gate 8: no doctrinal divergence is listed as an implementation gap.
- Gate 9: ADR-MS-001 remains the local authority for lineage-first replay and audit evidence.
- Gate 10: remediation notes cite this validation file as the empirical-number closeout.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-VALIDATE-data-pipeline-empirical-numbers.md:90` - - 31 monitoring sync progress: covered by OpenSLOs and dashboards; Airbyte 10-second progress polling validates published number pressure.; `microservices/data-pipeline/implementation-plans/IP-VALIDATE-data-pipeline-empirical-numbers.md:92` - - 33 availability SLO: covered by availability.openslo.yaml target 0.999..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-VALIDATE-data-pipeline-empirical-numbers.md:46` - - dbt published facts used: Fusion parse times are reported up to 30x faster than dbt Core, full-project compilation twice as quick, and state-aware orchestration can...; `microservices/data-pipeline/implementation-plans/IP-VALIDATE-data-pipeline-empirical-numbers.md:73` - - 14 transform cost attribution: covered by IP-017 and IP-029..
