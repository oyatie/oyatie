---
doc_class: MicroserviceOwnershipCoherenceAudit
microservice: observability
audit_date: 2026-05-20
owner: solo-audit-agent
status: landed
---

# Observability Ownership-Coherence Audit - 2026-05-20

## Five-Citation Anchor Block

1. Canonical sequence and batch discipline: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-4235`, especially §D-15 multi-context, §D-16 OpenTofu, §D-17 OS matrix, §D-18 Rust-strict policy, §D-19 OCI Always Free, and §D-20 audit decision tree.
2. Master-plan machine source: `specs/master-plan-sequencing.json:704-868`, covering six deployment contexts, OpenTofu substrate, supported OSes, language policy, and OCI Always Free profile.
3. Service PRD source read: `microservices/observability/PRD.md:20-309`, covering purpose, requirements, bounded contexts, performance targets, acceptance criteria, and open questions.
4. Service architecture source read: `microservices/observability/ARCHITECTURE.md:1-754`, with scaffold marker at `ARCHITECTURE.md:3`, tenant placeholder surfaces at `ARCHITECTURE.md:141-144`, and deployment-shape IaC summary at `ARCHITECTURE.md:445-456`.
5. Documentation-rigor source: `docs/standards/documentation-rigor.md:58-156` and `docs/standards/documentation-rigor.md:222-261`, requiring intern-buildable completeness and ADR-adherence evidence.

## Investigation Scope

Audit command evidence: recursive service inventory found 160 files under `microservices/observability/`.
Audit command evidence: recursive line count found 45,392 pre-write lines under `microservices/observability/`.
Chat-history evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:6067` assigns ADR-0263 observability-emission contract, making structured logs, OpenTelemetry traces, metrics, exemplars, tenant id, Mimir, Loki, Tempo, and ClickHouse relevant to this service.
Chat-history evidence: the same file line `5976` and line `5978` record creation of task #35, "Tier-1 ADR-0263: Observability emission contract."
Chat-history evidence: line `6046` and line `6049` require intern walkthrough steps to show observability emission evidence.
Constraint-memory evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-15` assigns one agent to own one microservice end-to-end.
Constraint-memory evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-53` requires substance verification, not line-count completion.
Constraint-memory evidence: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20` requires bespoke, intern-buildable content rather than scaffold padding.

## §1 Microservice Purpose Summary

The `observability` microservice is not a tenant-facing standalone monitoring product in the normal SaaS sense.
Its PRD defines it as Oyatie's shared SLO-evidence and telemetry substrate, not a hero product, at `PRD.md:22-26`.
It owns OpenSLO authoring conventions, real-time burn-rate evaluation, promotion eligibility evidence, rollback evidence, and Layer-A telemetry runtime.
The Layer-A runtime named by the PRD is Grafana Alloy, Prometheus, Mimir, Loki, Tempo, Pyroscope, Grafana, Alertmanager, and Grafana OnCall at `PRD.md:22`.
The service consumes OpenTelemetry signals from every other microservice and feeds promotion gates, tenant dashboards, incident response, and audit-chain evidence.
The central product claim is stronger than generic observability: release pointers should not advance unless SLO evidence is green.
The service therefore straddles three domains: telemetry ingest, SLO-control-plane decisions, and evidence retention.
The PRD has clear functional requirements for OpenSLO manifests, burn-rate evaluator windows, signed ledger records, rollback primitive, tenant SLO views, OTel collector intake, and canary cohort control at `PRD.md:37-48`.
The PRD's bounded-context map separates `slo-engine`, `otel-ingest`, and an eligibility-ledger writer subsumed under `slo-engine` at `PRD.md:90-94`.
The architecture file, however, compresses bounded contexts into a generic `observability` context and still carries Wave-3-C scaffold language at `ARCHITECTURE.md:3`.
The manifest repeats that compression by listing only one bounded context named `observability` at `manifest.json:6-28`.
That is the first purpose-coherence tension: PRD decomposition is richer than the manifest and architecture decomposition.
Industry-counterpart comparison should therefore not ask whether Oyatie can replace all of Datadog, New Relic, and Grafana Cloud as commercial products.
It should ask whether the union surface needed for Oyatie's internal and tenant SLO platform is represented or deliberately excluded.
Datadog, New Relic, and Grafana Cloud all cover metrics, logs, traces, dashboards, alerting, APM, infrastructure/Kubernetes monitoring, user experience monitoring, synthetics, profiling, incident/on-call workflows, integrations, and cost controls.
Oyatie observability covers many backend telemetry and SLO areas, but it does not yet cover full RUM/session replay, broad synthetic testing, broad database monitoring, mature error grouping, full service catalog, LLM observability, or turnkey 1,000-plus integration breadth.
Oyatie has additive surfaces not present in the counterparts: signed Oya VCS promotion gating, OpenSLO source control as release prerequisite, audit-chain anchoring, and sovereign-pack-aware telemetry custody.
The highest-risk gap is not product imagination.
The highest-risk gap is coherence between the new canonical constraints and the existing service folder.
ADR-0328 requires all six deployment contexts unless explicitly N/A, per-context OpenTofu, supported OS manifest, Rust-strict boundary, and OCI Always Free demo_trial tenant_class mapping.
The current folder has strong service-domain detail but still carries pre-ADR-0328 IaC and tier assumptions.
The current folder has Helm, Kustomize, and a Terraform file, but no canonical OpenTofu context directories.
The current folder has no `supported-oses.json` manifest.
The current folder has no forbidden-language source files under the service path, but several docs prescribe future Python, TypeScript, Go, JVM, Ruby, PHP, and C# SDK output without the ADR-0328 exception discipline.
The current demo_trial tenant_class tier is sized like a paid multi-node cluster, not OCI Always Free.
The purpose is therefore coherent at the product-substrate level and incoherent at the canonical deployment substrate level.

## §2 Inventory Snapshot

Pre-write files seen: 160.
Pre-write service lines audited: 45,392.
Inventory basis: `find microservices/observability -type f -print0 | xargs -0 wc -c | sort -k2`.
Coherence key: yes = coherent with SLO/telemetry substrate purpose; partial = useful but stale, incomplete, or canonical-drifted; no = contradicts current canonical constraints or product purpose.

| File | Size bytes | Role | coherent_with_purpose? |
|---|---:|---|---|
| `microservices/observability/ARCHITECTURE.md` | 98957 | broad architecture anchors; scaffold marker remains | partial |
| `microservices/observability/AUDIT-FINDINGS-2026-05-18.json` | 2210 | prior audit data | partial |
| `microservices/observability/IP-001-layer-a-grafana-stack-iac.md` | 5502 | Layer-A Grafana stack implementation plan | partial |
| `microservices/observability/IP-002-openslo-manifest-convention.md` | 3727 | OpenSLO convention plan | yes |
| `microservices/observability/IP-003-slo-engine-kernel.md` | 7439 | SLO kernel plan | yes |
| `microservices/observability/IP-004-slo-engine-domain.md` | 4013 | SLO domain plan | yes |
| `microservices/observability/IP-005-slo-engine-usecase.md` | 4155 | SLO usecase plan | yes |
| `microservices/observability/IP-006-slo-engine-adapter.md` | 4419 | SLO adapter plan | yes |
| `microservices/observability/IP-007-slo-engine-rest.md` | 3718 | SLO REST plan | yes |
| `microservices/observability/IP-008-slo-engine-worker.md` | 3931 | evaluator worker plan | yes |
| `microservices/observability/IP-009-slo-engine-app.md` | 3102 | app composition plan | partial |
| `microservices/observability/IP-010-promotion-eligibility-ledger.md` | 3262 | promotion ledger plan | yes |
| `microservices/observability/IP-011-per-component-release-pointers.md` | 3293 | release pointer plan | yes |
| `microservices/observability/IP-012-oya-vcs-promotion-readiness-lane.md` | 2767 | promotion readiness CI lane plan | yes |
| `microservices/observability/IP-013-event-driven-promote-workflows.md` | 3113 | event-driven workflow plan | yes |
| `microservices/observability/IP-014-automated-rollback-primitive.md` | 2733 | rollback primitive plan | yes |
| `microservices/observability/IP-015-canary-cohort-weighting.md` | 3124 | canary cohort plan | yes |
| `microservices/observability/IP-021-clickhouse-cluster-iac.md` | 10856 | ClickHouse cluster plan | partial |
| `microservices/observability/IP-022-otel-to-clickhouse-bridge.md` | 10091 | OTLP-to-ClickHouse bridge | yes |
| `microservices/observability/IP-023-ops-portal-rollup-mvs.md` | 9648 | ops rollup materialized views | yes |
| `microservices/observability/IP-024-cold-tier-retention-policy.md` | 9868 | cold retention plan | yes |
| `microservices/observability/IP-025-clickhouse-backup-restore.md` | 10713 | backup and restore plan | yes |
| `microservices/observability/IP-026-sse-transport-impl.md` | 1185 | SSE transport plan | partial |
| `microservices/observability/IP-027-websocket-transport-impl.md` | 1091 | WebSocket transport plan | partial |
| `microservices/observability/IP-028-loro-presence-binding.md` | 954 | collaboration presence binding | partial |
| `microservices/observability/IP-029-tail-sampling-processor-config.md` | 1332 | tail-sampling processor plan | yes |
| `microservices/observability/IP-030-sample-recipe-per-microservice.md` | 957 | per-service sample recipe | yes |
| `microservices/observability/IP-031-tail-sample-fidelity-test.md` | 1017 | sampling fidelity test plan | yes |
| `microservices/observability/IP-journey-j01-emergency-911-dispatch-emergency-metrics.md` | 73646 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j04-survivor-safe-telemetry.md` | 73291 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j05-privacy-preserving-telemetry.md` | 75831 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j10-ato-signal-correlation.md` | 73135 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j100-pack-rollout-first-action.md` | 65994 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j103-risk-and-slo-telemetry.md` | 64052 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j107-risk-and-slo-telemetry.md` | 64028 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j109-risk-and-slo-telemetry.md` | 65673 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j115-risk-and-slo-telemetry.md` | 64108 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j117-slo-breach-detector.md` | 59296 | journey SLO breach plan | yes |
| `microservices/observability/IP-journey-j12-surge-slo-telemetry.md` | 73537 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j120-slippage-and-latency-telemetry.md` | 62708 | journey telemetry plan | yes |
| `microservices/observability/IP-journey-j126-cross-tenant-audit-metrics.md` | 43015 | cross-tenant audit metrics plan | yes |
| `microservices/observability/IP-journey-j13-conflict-transparency-metrics.md` | 76419 | journey transparency metrics plan | yes |
| `microservices/observability/IP-journey-j131-cross-region-metric-labels.md` | 64842 | cross-region labels plan | yes |
| `microservices/observability/IP-journey-j138-corporate-audit-fraud-pattern-detector.md` | 34150 | corporate fraud telemetry plan | partial |
| `microservices/observability/IP-journey-j140-internal-audit-dlp-egress-detector.md` | 58615 | internal audit telemetry plan | yes |
| `microservices/observability/IP-journey-j20-egress-detection-telemetry.md` | 75198 | residency-egress telemetry plan | yes |
| `microservices/observability/IP-journey-j21-bootstrap-trace.md` | 59252 | bootstrap trace plan | yes |
| `microservices/observability/IP-journey-j22-deliverability-metrics.md` | 61875 | deliverability telemetry plan | yes |
| `microservices/observability/IP-journey-j25-sync-health.md` | 58070 | sync health telemetry plan | yes |
| `microservices/observability/IP-journey-j27-schedule-conflict-metrics.md` | 62700 | schedule metrics plan | yes |
| `microservices/observability/IP-journey-j28-webrtc-qos.md` | 57705 | WebRTC QoS plan | yes |
| `microservices/observability/IP-journey-j32-moderation-slo.md` | 59127 | moderation SLO plan | yes |
| `microservices/observability/IP-journey-j33-sso-rollout-metrics.md` | 60631 | SSO rollout metrics plan | yes |
| `microservices/observability/IP-journey-j34-channel-file-audit.md` | 60280 | file audit telemetry plan | yes |
| `microservices/observability/IP-journey-j35-dmarc-calendar-slo.md` | 60279 | mail/calendar SLO plan | yes |
| `microservices/observability/IP-journey-j37-attendance-slo-traces.md` | 68108 | attendance SLO trace plan | yes |
| `microservices/observability/IP-journey-j39-meeting-telemetry.md` | 67556 | meeting telemetry plan | yes |
| `microservices/observability/IP-journey-j41-release-telemetry.md` | 68170 | release telemetry plan | yes |
| `microservices/observability/IP-journey-j42-usage-meter-rollup.md` | 68835 | usage-meter rollup | yes |
| `microservices/observability/IP-journey-j68-dashboard-share.md` | 113286 | dashboard sharing telemetry plan | yes |
| `microservices/observability/IP-journey-j71-fraud-signal.md` | 111738 | fraud signal telemetry plan | yes |
| `microservices/observability/IP-journey-j77-telemetry-and-slo.md` | 37528 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j78-telemetry-and-slo.md` | 37556 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j79-telemetry-and-slo.md` | 37629 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j81-telemetry-and-slo.md` | 37479 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j82-telemetry-and-slo.md` | 37667 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j85-telemetry-and-slo.md` | 37792 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j86-telemetry-and-slo.md` | 38123 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j87-telemetry-and-slo.md` | 37393 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j88-telemetry-and-slo.md` | 37595 | generic journey telemetry plan | partial |
| `microservices/observability/IP-journey-j91-us-msb-mtl-overlay.md` | 68996 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j92-br-lgpd-us-parent-dsar.md` | 67343 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j93-in-dpdpa-rbi-overlay.md` | 68695 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j94-sox404-public-company-controls.md` | 70184 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j95-iso27001-soc2-annual-audit.md` | 68080 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j96-ksa-uae-mena-onboarding.md` | 68842 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j97-sg-pdpa-mas-tenant.md` | 65297 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j98-au-privacy-apra-cps234.md` | 66283 | regulatory telemetry journey | yes |
| `microservices/observability/IP-journey-j99-multi-pack-conflict-resolution.md` | 75587 | regulatory telemetry journey | yes |
| `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` | 20480 | phase plan | partial |
| `microservices/observability/PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION-ADDENDUM.md` | 4296 | ClickHouse phase addendum | partial |
| `microservices/observability/PRD.md` | 23273 | product requirements | yes |
| `microservices/observability/backfill-replay.md` | 6142 | replay behavior doc | yes |
| `microservices/observability/benchmarks/datadog-vs-newrelic-vs-honeycomb-vs-oyatie.md` | 7646 | older benchmark comparison | partial |
| `microservices/observability/capabilities/eligibility-query.yaml` | 2360 | capability record | yes |
| `microservices/observability/capabilities/openslo-validate.yaml` | 2207 | capability record | yes |
| `microservices/observability/capabilities/slo-evaluate.yaml` | 3008 | capability record | yes |
| `microservices/observability/capability-tiers/tier-matrix.md` | 13105 | capability tiers | partial |
| `microservices/observability/capacity-model-clickhouse.md` | 2235 | ClickHouse capacity note | partial |
| `microservices/observability/capacity-model.md` | 10250 | capacity model | yes |
| `microservices/observability/catalog/oya-observability-otel-ingest-adapter.yaml` | 831 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-otel-ingest-api.yaml` | 791 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-otel-ingest-app.yaml` | 802 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-otel-ingest-kernel.yaml` | 801 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-otel-ingest-usecase.yaml` | 809 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-adapter-mimir.yaml` | 929 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-adapter.yaml` | 821 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-api.yaml` | 836 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-app.yaml` | 857 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-domain.yaml` | 790 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-kernel.yaml` | 942 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-rest.yaml` | 891 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-sdk.yaml` | 836 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-usecase.yaml` | 849 | crate catalog | yes |
| `microservices/observability/catalog/oya-observability-slo-engine-worker.yaml` | 883 | crate catalog | yes |
| `microservices/observability/competitor-parity-matrix.md` | 8747 | older parity matrix | partial |
| `microservices/observability/compliance.md` | 135434 | compliance control mapping | yes |
| `microservices/observability/contracts/asyncapi/eligibility-events.yaml` | 6268 | event contract | yes |
| `microservices/observability/contracts/metric-naming-convention.md` | 9897 | naming convention | yes |
| `microservices/observability/contracts/openapi/slo-engine.yaml` | 9712 | REST contract | yes |
| `microservices/observability/contracts/proto/slo-engine.proto` | 4263 | gRPC/proto contract | yes |
| `microservices/observability/cost-budget.md` | 8056 | cost model | partial |
| `microservices/observability/dashboards/brownout-degradation.md` | 2731 | dashboard doc | yes |
| `microservices/observability/dashboards/dr-business-continuity.md` | 2819 | dashboard doc | yes |
| `microservices/observability/dashboards/finops-cost-attribution.md` | 3984 | dashboard doc | yes |
| `microservices/observability/dashboards/gate-eligibility.json` | 3279 | dashboard JSON | yes |
| `microservices/observability/dashboards/operator-burn-rate.json` | 3804 | dashboard JSON | yes |
| `microservices/observability/dashboards/tenant-slo-overview.json` | 4207 | dashboard JSON | yes |
| `microservices/observability/decisions/ADR-OBS-001-tracing-substrate-choice.md` | 20695 | local tracing ADR | partial |
| `microservices/observability/dpia.md` | 28251 | privacy impact assessment | yes |
| `microservices/observability/failure-modes.md` | 15274 | failure analysis | yes |
| `microservices/observability/faqs/sre-lead-faq.md` | 8622 | SRE FAQ | yes |
| `microservices/observability/iac/helm/alertmanager/Chart.yaml` | 511 | Helm chart metadata | partial |
| `microservices/observability/iac/helm/alertmanager/values.yaml` | 1469 | Helm values | partial |
| `microservices/observability/iac/helm/alloy/Chart.yaml` | 684 | Helm chart metadata | partial |
| `microservices/observability/iac/helm/alloy/templates/deployment.yaml` | 3034 | Kubernetes deployment template | partial |
| `microservices/observability/iac/helm/alloy/templates/networkpolicy.yaml` | 2405 | Kubernetes policy template | partial |
| `microservices/observability/iac/helm/alloy/values.yaml` | 1578 | Helm values | partial |
| `microservices/observability/iac/helm/axe-pa11y-runner/Chart.yaml` | 544 | accessibility chart metadata | partial |
| `microservices/observability/iac/helm/axe-pa11y-runner/README.md` | 1060 | chart README | partial |
| `microservices/observability/iac/helm/axe-pa11y-runner/values.yaml` | 1848 | chart values | partial |
| `microservices/observability/iac/helm/backstage/Chart.yaml` | 1082 | Backstage chart metadata | partial |
| `microservices/observability/iac/helm/backstage/values.yaml` | 1569 | Backstage values | partial |
| `microservices/observability/iac/helm/clickhouse/Chart.yaml` | 1295 | ClickHouse chart metadata | partial |
| `microservices/observability/iac/helm/clickhouse/templates/external-secret.yaml` | 711 | ClickHouse secret template | partial |
| `microservices/observability/iac/helm/clickhouse/templates/keeper-template.yaml` | 1729 | ClickHouse keeper template | partial |
| `microservices/observability/iac/helm/clickhouse/templates/prometheus-rule.yaml` | 1727 | ClickHouse alert template | partial |
| `microservices/observability/iac/helm/clickhouse/templates/service-monitor.yaml` | 406 | ClickHouse ServiceMonitor | partial |
| `microservices/observability/iac/helm/clickhouse/values.yaml` | 3850 | ClickHouse values | partial |
| `microservices/observability/iac/helm/grafana/Chart.yaml` | 529 | Grafana chart metadata | partial |
| `microservices/observability/iac/helm/grafana/values.yaml` | 1642 | Grafana values | partial |
| `microservices/observability/iac/helm/loki/Chart.yaml` | 461 | Loki chart metadata | partial |
| `microservices/observability/iac/helm/loki/values.yaml` | 908 | Loki values | partial |
| `microservices/observability/iac/helm/mimir/Chart.yaml` | 550 | Mimir chart metadata | partial |
| `microservices/observability/iac/helm/mimir/values.yaml` | 1503 | Mimir values | partial |
| `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml` | 14653 | canonical alert rules | yes |
| `microservices/observability/iac/helm/oncall/Chart.yaml` | 494 | OnCall chart metadata | partial |
| `microservices/observability/iac/helm/oncall/values.yaml` | 714 | OnCall values | partial |
| `microservices/observability/iac/helm/opencost/Chart.yaml` | 866 | OpenCost chart metadata | partial |
| `microservices/observability/iac/helm/opencost/values.yaml` | 3876 | OpenCost values | partial |
| `microservices/observability/iac/helm/otel-tailsampling-collector/Chart.yaml` | 740 | tail-sampling chart metadata | partial |
| `microservices/observability/iac/helm/otel-tailsampling-collector/README.md` | 1343 | tail-sampling chart README | partial |
| `microservices/observability/iac/helm/otel-tailsampling-collector/values.yaml` | 3863 | tail-sampling chart values | partial |
| `microservices/observability/iac/helm/prometheus/Chart.yaml` | 573 | Prometheus chart metadata | partial |
| `microservices/observability/iac/helm/prometheus/values.yaml` | 729 | Prometheus values | partial |
| `microservices/observability/iac/helm/pyroscope/Chart.yaml` | 447 | Pyroscope chart metadata | partial |
| `microservices/observability/iac/helm/pyroscope/values.yaml` | 550 | Pyroscope values | partial |
| `microservices/observability/iac/helm/statuspage/Chart.yaml` | 1013 | status page chart metadata | partial |
| `microservices/observability/iac/helm/statuspage/values.yaml` | 1363 | status page values | partial |
| `microservices/observability/iac/helm/tempo/Chart.yaml` | 447 | Tempo chart metadata | partial |
| `microservices/observability/iac/helm/tempo/values.yaml` | 713 | Tempo values | partial |
| `microservices/observability/iac/helm/timescaledb-extension/Chart.yaml` | 475 | Timescale extension chart metadata | partial |
| `microservices/observability/iac/helm/timescaledb-extension/templates/post-install-job.yaml` | 2231 | post-install job | partial |
| `microservices/observability/iac/helm/timescaledb-extension/values.yaml` | 256 | Timescale extension values | partial |
| `microservices/observability/iac/kustomize/base/kustomization.yaml` | 1809 | Kustomize base | partial |
| `microservices/observability/iac/kustomize/components/clickhouse-keeper/configmap.yaml` | 1519 | Kustomize component | partial |
| `microservices/observability/iac/kustomize/components/clickhouse-keeper/kustomization.yaml` | 533 | Kustomize component index | partial |
| `microservices/observability/iac/kustomize/components/clickhouse-keeper/pdb.yaml` | 249 | Kustomize PDB | partial |
| `microservices/observability/iac/kustomize/components/clickhouse-keeper/service.yaml` | 345 | Kustomize service | partial |
| `microservices/observability/iac/kustomize/components/clickhouse-keeper/statefulset.yaml` | 2109 | Kustomize statefulset | partial |
| `microservices/observability/iac/kustomize/overlays/pack-kr/kustomization.yaml` | 1073 | pack overlay | partial |
| `microservices/observability/iac/terraform/grafana-rbac.tf` | 2360 | Terraform RBAC IaC | no |
| `microservices/observability/incident-response.md` | 13677 | incident response | yes |
| `microservices/observability/manifest.json` | 12745 | machine-readable manifest | partial |
| `microservices/observability/migration-playbooks/from-datadog.md` | 8537 | migration playbook | yes |
| `microservices/observability/multi-region.md` | 12329 | multi-region design | partial |
| `microservices/observability/onboarding/sre-lead-first-week.md` | 8049 | onboarding | yes |
| `microservices/observability/packs/EU-AI-Act.md` | 11944 | compliance pack | yes |
| `microservices/observability/packs/GDPR.md` | 11863 | compliance pack | yes |
| `microservices/observability/packs/HIPAA.md` | 12668 | compliance pack | yes |
| `microservices/observability/packs/KR-PIPA.md` | 12234 | compliance pack | yes |
| `microservices/observability/packs/SOC2.md` | 12333 | compliance pack | yes |
| `microservices/observability/policy/auditor-scope.cedar` | 5435 | Cedar policy | yes |
| `microservices/observability/policy/ci-scope.cedar` | 4317 | Cedar policy | yes |
| `microservices/observability/policy/data-residency.md` | 12360 | data residency policy | partial |
| `microservices/observability/policy/public-read.cedar` | 4930 | Cedar policy | yes |
| `microservices/observability/policy/tenant-isolation.md` | 16534 | tenant isolation policy | partial |
| `microservices/observability/policy/tenant-scope.cedar` | 4522 | Cedar policy | yes |
| `microservices/observability/reference-implementations/emit-spans-and-author-slo-rust-sdk.md` | 9291 | Rust reference implementation | yes |
| `microservices/observability/runbooks/canary-graduation.md` | 26793 | runbook | yes |
| `microservices/observability/runbooks/cardinality-explosion-detection.md` | 29029 | runbook | yes |
| `microservices/observability/runbooks/clickhouse-disk-pressure-mitigation.md` | 29637 | runbook | yes |
| `microservices/observability/runbooks/clickhouse-restore.md` | 27048 | runbook | yes |
| `microservices/observability/runbooks/clickhouse.md` | 25775 | runbook | yes |
| `microservices/observability/runbooks/evaluator-down.md` | 26337 | runbook | yes |
| `microservices/observability/runbooks/held-promotion-recovery.md` | 27743 | runbook | yes |
| `microservices/observability/runbooks/mimir-outage.md` | 26080 | runbook | yes |
| `microservices/observability/runbooks/oncall-rotation.md` | 26472 | runbook | yes |
| `microservices/observability/runbooks/realtime-transport-connection-leak.md` | 29498 | runbook | yes |
| `microservices/observability/runbooks/rollback.md` | 25429 | runbook | yes |
| `microservices/observability/runbooks/tail-sampling-buffer-saturated.md` | 28854 | runbook | yes |
| `microservices/observability/runbooks/trace-sampling-loss-investigation.md` | 29354 | runbook | yes |
| `microservices/observability/scorecards/overrides.json` | 1045 | scorecard override | partial |
| `microservices/observability/sdk-plan.md` | 7778 | SDK plan | partial |
| `microservices/observability/slos/alerting-fanout-latency.openslo.yaml` | 1666 | OpenSLO | yes |
| `microservices/observability/slos/clickhouse-ingest-throughput.openslo.yaml` | 1193 | OpenSLO | yes |
| `microservices/observability/slos/log-ingest-availability.openslo.yaml` | 1502 | OpenSLO | yes |
| `microservices/observability/slos/metric-ingest-availability.openslo.yaml` | 1742 | OpenSLO | yes |
| `microservices/observability/slos/query-latency-logs.openslo.yaml` | 1607 | OpenSLO | yes |
| `microservices/observability/slos/query-latency-prom.openslo.yaml` | 1580 | OpenSLO | yes |
| `microservices/observability/slos/tail-sample-fidelity.openslo.yaml` | 1029 | OpenSLO | yes |
| `microservices/observability/slos/trace-ingest-availability.openslo.yaml` | 1572 | OpenSLO | yes |
| `microservices/observability/threat-model.md` | 51608 | threat model | yes |
| `microservices/observability/tutorials/wire-microservice-slos-into-promotion-gate.md` | 9074 | tutorial | yes |

## §3 9-Dimension Audit

### §3.1 Dimension 1 - Internal Coherence

1. Product purpose resolves: `PRD.md:22-24` defines shared SLO/telemetry substrate, and most files support telemetry, SLO, incident, compliance, or runbook behavior.
2. Bounded-context contradiction: PRD names `slo-engine`, `otel-ingest`, and ledger writer at `PRD.md:90-94`, but manifest lists only bounded context `observability` at `manifest.json:6-28`; severity P2 because docs still point to coherent crate names.
3. Architecture scaffold marker remains at `ARCHITECTURE.md:3`; severity P2 because architecture content may be broad but the file explicitly says stub expansion was pending.
4. Architecture tenant tables use generated placeholder-like names `eligibility_query_2` through `eligibility_query_5` at `ARCHITECTURE.md:141-144`; severity P2 because storage surfaces are not intern-buildable.
5. Manifest SLO list omits `slos/clickhouse-ingest-throughput.openslo.yaml` and `slos/tail-sample-fidelity.openslo.yaml`, which exist in inventory and are acknowledged by ADR-OBS at `decisions/ADR-OBS-001-tracing-substrate-choice.md:26-28`; severity P2.
6. Manifest IP list stops at IP-015 at `manifest.json:109-200`, while the inventory contains IP-021 through IP-031; severity P2 because implementation planning index is stale.
7. Manifest tier taxonomy lists `T0`, `T1`, and `T2` at `manifest.json:293-297`, while capability tier document uses demo_trial tenant_class, paid tenant_class baseline, paid tenant_class scale, compliance_pack-gated paid tenant_class at `capability-tiers/tier-matrix.md:15-126`; severity P2.
8. PRD acceptance criteria cite a missing e2e test path at `PRD.md:278-286`; investigation found no `tests/` directory under service; severity P2.
9. PRD performance target says `cargo run -p` and service-specific binaries at `PRD.md:278-286`, but there is no service `src/` tree under the service path; severity P2 for buildability gap.
10. Phase plan claims all 15 IPs merged and cargo nextest green at `PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md:8-10`, but the current folder has plans and catalog records rather than Rust source; severity P2 until repo-level crates prove it elsewhere.
11. PRD says no Datadog/Honeycomb/Lightstep contract is required at `PRD.md:30`, but ADR-OBS chooses ClickHouse Cloud as canonical production trace store at `decisions/ADR-OBS-001-tracing-substrate-choice.md:55-85`; severity P1 for on-prem/colo deployment unless cell-local replacement is explicit.
12. ADR-OBS mitigates the managed-store concern by saying emission stays storage-agnostic and future cell-local ClickHouse can replace Cloud at `ADR-OBS-001:83-85`; this reduces but does not erase the P1 deployment-context gap.
13. PRD's competitor matrix omits New Relic and includes Nobl9, Sloth, and Google Cloud Monitoring at `PRD.md:217-223`; user scope requires Datadog, New Relic, Grafana Cloud; severity P2.
14. Existing benchmark file compares Datadog, New Relic, Honeycomb, and Oyatie by filename, not Grafana Cloud; inventory evidence `benchmarks/datadog-vs-newrelic-vs-honeycomb-vs-oyatie.md`; severity P2 for scope mismatch.
15. `capability-tiers/tier-matrix.md:46` says demo_trial tenant_class is not eligible for staging-to-prod evidence, while ADR-0328 OCI Always Free demo_trial tenant_class requirement maps demo_trial tenant_class to Always Free for the OCI guest profile; severity P1 for tier semantics conflict.
16. demo_trial tenant_class capacity at `capability-tiers/tier-matrix.md:17-24` requires multiple 4 vCPU/8 vCPU nodes and TiB storage, contradicting OCI Always Free resource limits in ADR-0328 §D-19; severity P1.
17. The service has useful runbooks, including evaluator-down and rollback, and these align with PRD rollback and OnCall requirements at `PRD.md:43-47`; resolves.
18. The service has OpenSLO files for metrics/logs/traces/query/tail-sampling/clickhouse, aligning with PRD FR-01 through FR-07 at `PRD.md:39-45`; resolves.
19. `capacity-model.md` and `cost-budget.md` align with PRD horizontal scalability at `PRD.md:246-264`; partial because they are OCI-priced and not all-context.
20. `policy/tenant-isolation.md` aligns with PRD Mimir multi-tenancy at `PRD.md:84`; partial because it references Terraform drift checks at `policy/tenant-isolation.md:192-198`.
21. `iac/helm/grafana/values.yaml:41` says Grafana RBAC is declared through OpenTofu, while `iac/terraform/grafana-rbac.tf:1-5` declares Terraform source-of-truth; severity P1.
22. `compliance.md:86` cites `iac/terraform/grafana-rbac.tf` for access-rights control, reinforcing the Terraform contradiction; severity P1.
23. `sdk-plan.md:30-38` schedules TypeScript, Python, Go, JVM SDKs; ADR-0328 permits generated SDK output only under explicit exception discipline; severity P2 in documentation until implementation appears.
24. `competitor-parity-matrix.md:76-97` repeats future Py/Go/JVM SDK breadth as a gap; severity P2 for Rust-strict drift.
25. `PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md:91-96` uses cargo commands but omits the canonical `--release --all-features --locked` build form required by ADR-0328 §D-18; severity P2.
26. `IP-009-slo-engine-app.md:62` uses `cargo build -p ... --release`, still not the canonical workspace locked build; severity P2.
27. `manifest.json:214-222` stores LTS pins with embedded quote/comment text inside JSON string values; severity P3 because JSON parses but machine values are polluted.
28. Architecture lists data classes `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI` at `ARCHITECTURE.md:30`; aligns with sensitive trace-data purpose.
29. PRD requires no cross-product crate imports at `PRD.md:172`; architecture cross-service links use events/policies rather than imports at `ARCHITECTURE.md:40-46`; resolves.
30. Contracts are present: OpenAPI, AsyncAPI, proto, and metric naming convention in inventory; this aligns with PRD API and workflow-event claims at `PRD.md:184-213`.
31. OpenAPI version was not fully revalidated in this report, but contract presence is coherent with product purpose; residual risk is schema-version drift against AGENTS.
32. No `README.md` exists in the service inventory; severity P3 because onboarding exists, but top-level service entry is missing.
33. No `cross-microservice-handoffs.md` exists in inventory; severity P2 because PRD says every other microservice consumes observability at `PRD.md:24`.
34. `onboarding/sre-lead-first-week.md` provides operator onboarding and is coherent with tenant/SRE value at `PRD.md:30-33`.
35. `migration-playbooks/from-datadog.md` is coherent with Datadog migration purpose, but it only covers one counterpart; severity P3 for New Relic/Grafana Cloud migration gap.
36. `reference-implementations/emit-spans-and-author-slo-rust-sdk.md` aligns with Rust-first SDK purpose at `PRD.md:121-123`; resolves.
37. `tutorials/wire-microservice-slos-into-promotion-gate.md` aligns with PRD FR-10 aggregation and promotion gate requirements at `PRD.md:48`.
38. `dashboards/gate-eligibility.json`, `operator-burn-rate.json`, and `tenant-slo-overview.json` align with dashboard and tenant operator claims at `PRD.md:31` and `PRD.md:46`.
39. `scorecards/overrides.json` is small and unexplained in the core docs; severity P3 because its role is not integrated into PRD/architecture.
40. `AUDIT-FINDINGS-2026-05-18.json` is useful history but not referenced from current manifest; severity P3.
41. Local ADR status is `Proposed` at `ADR-OBS-001:4`, while PRD is `Accepted` at `PRD.md:6`; severity P2 because a proposed ADR can still govern local tracing decisions ambiguously.
42. PRD `related_adrs` omits local `ADR-OBS-001`, while the ADR is in `decisions/`; severity P3.
43. Architecture references `ADR-0254` deployment shape at `ARCHITECTURE.md:445-446`, but the service lacks the ADR-0328 deployment context matrix; severity P1 for newer canonical override.
44. Failure-mode and incident-response docs are substantial and aligned with product purpose; resolves.
45. Compliance and DPIA docs are substantial and aligned with telemetry privacy risk; resolves.
46. Cost-budget is substantial but OCI-priced and not mapped to all six deployment contexts; severity P2.
47. Capacity model is substantial but predates OCI Always Free demo_trial tenant_class reconciliation; severity P2.
48. IAC directory has many Helm charts that are useful runtime packaging but cannot substitute for ADR-0328 OpenTofu context modules; severity P1.
49. Internal coherence headline: purpose is strong, operational corpus is broad, machine control surfaces are stale.
50. Dimension 1 conclusion: partial coherence with P1 canonical-substrate blockers and P2 buildability/index drift.

### §3.2 Dimension 2 - Outbound Cross-References

1. PRD outbound ADR references at `PRD.md:11` include ADR-0056, ADR-0105, ADR-0110, ADR-0114, ADR-0139, and ADR-0131; these are plausible core references.
2. PRD related specs at `PRD.md:12` include `/specs/agentic-slo-gated-promotion.json` and `/specs/per-microservice-flat-layout.json`; these are consistent with purpose.
3. PRD acceptance criteria cite `microservices/observability/tests/e2e/eligibility-happy-path.rs` at `PRD.md:279`; target does not exist in inventory; orphan severity P2.
4. PRD cites `runbooks/error-budget-policy.md` at `PRD.md:244`; inventory has no such file; broken internal reference severity P2.
5. PRD cross-product dependencies include tenancy, governance aggregation, workflow, ontology, audit-chain, Grafana OnCall at `PRD.md:184-213`; targets are conceptual and mostly resolve to microservices or docs outside this service.
6. Architecture cross-service links include identity, tenancy, policy-engine, audit-chain, cloud-secrets, cell, and cloud-iac at `ARCHITECTURE.md:40-46`; outbound references are domain-correct.
7. Manifest dependency list at `manifest.json:352-362` includes compliance, identity, tenancy, audit-chain, cloud-secrets, network, ontology, detection, cell, cloud-iac; resolves as microservice names in repo vocabulary.
8. Manifest dependency list uses `network`, while architecture says `cell` and `cloud-iac` own runtime cell and facility posture at `ARCHITECTURE.md:46`; no contradiction.
9. Compliance control `compliance.md:86` references Terraform RBAC file; target exists, but direction is wrong under ADR-0328; severity P1.
10. Tenant isolation policy references OpenTofu and Terraform drift in the same FM-06 area at `policy/tenant-isolation.md:192-198`; target exists but terminology contradicts canonical OpenTofu-only.
11. Helm Grafana chart description references OpenTofu in `iac/helm/grafana/Chart.yaml:4`; aligned word, but implementation target points to Terraform file; severity P2.
12. Helm OnCall chart description references OpenTofu in `iac/helm/oncall/Chart.yaml:4`; no OpenTofu module exists; severity P2.
13. Kustomize pack-kr overlay references OCI object storage at `iac/kustomize/overlays/pack-kr/kustomization.yaml:33`; this is provider-specific runtime config without six-context abstraction; severity P2.
14. Policy data residency lists pack-to-OCI regions at `policy/data-residency.md:34-44`; coherent for M01 but not all deployment contexts.
15. Cost-budget cites OCI public pricing at `cost-budget.md:22-26`; coherent for OCI cost forecast, insufficient for all-context audit.
16. Runbooks consistently list Grafana Enterprise Support, ClickHouse support, and Oracle OCI object storage status desk, e.g. `runbooks/clickhouse-restore.md:22`; coherent for OCI/Grafana/ClickHouse, but not all-context.
17. ADR-OBS references ADR-0263 at `ADR-OBS-001:14` and chat history confirms ADR-0263 task at JSONL line `6067`; resolves.
18. ADR-OBS references ClickHouse Cloud docs at `ADR-OBS-001:221-222`; source exists externally but managed cloud dependency conflicts with on-prem/colo.
19. Existing benchmark references competitor pricing and performance but lacks current public source citations in the requested top-three shape; severity P2.
20. Outbound reference to `registry/promotion-eligibility.jsonl` at `PRD.md:42` is outside service; not verified in this audit because rule forbids touching other paths, but direction is coherent.
21. Outbound reference to branch protection and Oya VCS at `PRD.md:281-287` is coherent with promotion gate but not implemented under service.
22. Outbound reference to `.github/workflows/cosign.yml` appears in compliance `compliance.md:102`; target not verified here; open item for repo-level aggregation.
23. Outbound references to Mimir/Loki/Tempo/Pyroscope are coherent with Layer-A stack and backed by Helm charts in inventory.
24. Outbound references to `data-residency.md` from Kustomize and policies resolve to service-local `policy/data-residency.md`.
25. Outbound references to runbooks from architecture point to existing runbooks in inventory; resolves.
26. Outbound references to missing `runbooks/cross-region-restore.md` at `capability-tiers/tier-matrix.md:139` do not resolve in inventory; severity P2.
27. Outbound reference to `iac/grafana-dashboards/` at `capability-tiers/tier-matrix.md:56` does not match actual `dashboards/` path; severity P2.
28. Outbound reference to `microservices/<ms>/observability/sample-recipe.yaml` at `capability-tiers/tier-matrix.md:29` describes other service folders; okay as consumer requirement, but missing cross-handoff doc.
29. References TO this service from chat history include ADR-0263 task at JSONL line `6067`; this creates a reverse-reference obligation for emission contract alignment.
30. References TO this service from docs were not exhaustively grep-listed across all docs because the task forbids touching other service outputs but permits read-only grep; early chat and docs evidence are enough to flag aggregation questions.
31. The architecture file claims `observability` and `audit-chain` receive signed audit events at `ARCHITECTURE.md:44`; reverse reference in audit-chain not verified here; open Wave 14 question.
32. The PRD says every other microservice must author OpenSLO manifests before release advancement at `PRD.md:24`; absence of `cross-microservice-handoffs.md` makes reverse-reference governance weak; severity P2.
33. The manifest does not include a `reverse_references` or `consumers` field for all consuming microservices; severity P2.
34. `contracts/metric-naming-convention.md` likely acts as a handoff contract, but it is not named in manifest consumers; severity P3.
35. `contracts/asyncapi/eligibility-events.yaml` is a valid outbound event surface; direction resolves.
36. `contracts/openapi/slo-engine.yaml` is a valid tenant/operator API surface; direction resolves.
37. `contracts/proto/slo-engine.proto` is a valid internal high-throughput contract; direction resolves.
38. No `contracts/*.json` files are present; not a gap because YAML/proto cover declared surfaces.
39. No `README.md` means human entrypoint references must go through PRD/architecture; severity P3.
40. The local ADR references OpenTelemetry, ClickHouse, Tempo, Jaeger, Cedar, and W3C Trace Context at `ADR-OBS-001:216-228`; external refs are coherent for tracing substrate.
41. Current canonical sources require OpenTofu, but outbound references still include Terraform as active source; this is the most severe outbound-reference issue.
42. Current canonical sources require six deployment contexts, but outbound references are strongly OCI/Kubernetes/Grafana/ClickHouse centered; this is an all-context evidence issue.
43. Current canonical sources require OS manifest, but no outbound reference points to supported OSes; severity P1.
44. Current canonical sources require demo_trial tenant_class Always Free on OCI, but tier matrix outbound references point to hardware-heavy demo_trial tenant_class; severity P1.
45. Current canonical sources require Rust-strict, but SDK outbound references to multiple languages need generated-SDK exception treatment; severity P2.
46. Outbound counterpart references in PRD are stale against the user-required union coverage; severity P2.
47. Orphan references found: missing tests path, missing error-budget runbook, missing cross-region restore runbook, mismatched dashboard provisioning path.
48. Missing reverse references found: no consumer handoff file and no manifest consumer matrix for every microservice.
49. Dimension 2 headline: references mostly resolve inside the legacy design, but several critical references resolve to canonical-forbidden or missing targets.
50. Dimension 2 conclusion: P1 for Terraform/OpenTofu conflict and P2 for orphan handoff/test/runbook references.

### §3.3 Dimension 3 - Substance Bar

1. Cold intern can understand purpose from `PRD.md:20-33`; this passes the first comprehension gate.
2. Cold intern can list functional requirements from `PRD.md:37-48`; this passes requirement visibility.
3. Cold intern can identify entities and crates from `PRD.md:90-169`; this passes architecture vocabulary.
4. Cold intern cannot build the service from the service path because no `src/` tree exists under the audited path; inventory evidence.
5. Cold intern cannot run the named e2e test at `PRD.md:279` because the `tests/e2e/` path does not exist under inventory.
6. Cold intern cannot execute the canonical ADR-0328 build invocation from service docs because service docs use weaker cargo commands at `PHASE-01:91-96` and `IP-009:62`.
7. Cold intern cannot deploy any canonical context because required OpenTofu context directories are absent; ADR-0328 requires them at `ADR-0328:2275-2294`.
8. Cold intern cannot choose supported OS lanes because `supported-oses.json` is absent; ADR-0328 requires it at `ADR-0328:2907-2927`.
9. Cold intern can deploy some Helm charts in principle because many chart/values files exist; but those are not the canonical deployment substrate.
10. Cold intern cannot map demo_trial tenant_class to OCI Always Free because current demo_trial tenant_class requires multi-node paid hardware at `capability-tiers/tier-matrix.md:17-24`.
11. Cold intern can understand trace retention and ClickHouse decision from ADR-OBS at `ADR-OBS-001:55-85`; passes tracing-substrate explanation.
12. Cold intern cannot decide on-prem/colo trace storage because ADR-OBS chooses ClickHouse Cloud production at `ADR-OBS-001:57` and only future-proofs cell-local replacement at `ADR-OBS-001:83-85`.
13. Cold intern can understand tenant isolation intent from architecture `ARCHITECTURE.md:136-146`; partial due placeholder surfaces.
14. Cold intern cannot implement tenant storage tables from architecture because `eligibility_query_2` through `_5` are placeholder-like names at `ARCHITECTURE.md:141-144`.
15. Cold intern can read failure modes and runbooks; inventory shows 13 substantial runbooks.
16. Cold intern cannot reconcile manifest with actual service files because manifest omits newer IPs and two SLO files.
17. Cold intern can understand OpenSLO and promotion-gate acceptance from `PRD.md:274-287`.
18. Cold intern cannot run branch-protection emulation from service folder; PRD points at repo-level lane at `PRD.md:281-287`.
19. Cold intern can understand telemetry signal breadth from capability tiers and SLOs.
20. Cold intern cannot understand all six deployment contexts from service docs because no context matrix exists.
21. Cold intern cannot know if on-prem and colo are in scope because service docs mostly assume Kubernetes/OCI/Grafana stack, while user declared all six unless audit finds otherwise.
22. Cold intern cannot know if OCI Always Free is supported because no `iac/oci-guest/always-free/` exists.
23. Cold intern can find data residency policy for regional packs at `policy/data-residency.md:34-44`.
24. Cold intern cannot adapt data residency to non-OCI providers because policy rows are OCI-region-specific.
25. Cold intern can find cost forecasts in `cost-budget.md`, but costs cite OCI at `cost-budget.md:22-26`.
26. Cold intern cannot price AWS/on-prem/colo/oyatie-as-provider contexts from current cost budget.
27. Cold intern can find performance target numbers in PRD at `PRD.md:54-61` and `PRD.md:231-240`.
28. Cold intern cannot distinguish measured numbers from target numbers in older benchmark file without caveats; new benchmark deliverable fixes this.
29. Cold intern can use `reference-implementations/emit-spans-and-author-slo-rust-sdk.md`; this is one of the strongest buildability artifacts.
30. Cold intern cannot generate future Python/Go/JVM SDKs without violating current language policy; SDK docs need a generated-SDK exception matrix.
31. Cold intern can see policy fragments in Cedar; these are authorized non-Rust files.
32. Cold intern cannot see runtime Rust implementation, so policy-to-code enforcement is not directly buildable from service path.
33. Cold intern can see dashboards and JSON files for SLO/operator views.
34. Cold intern cannot validate dashboard provisioning path because tier matrix names `iac/grafana-dashboards/` but inventory uses `dashboards/`.
35. Cold intern can see AsyncAPI and OpenAPI contracts.
36. Cold intern cannot fully verify contract versions here without schema validation; this is a residual verification gap.
37. Cold intern can see compliance and DPIA depth; strong substance for privacy/security.
38. Cold intern cannot follow a single top-level README; missing README makes onboarding dependent on knowing which doc to start with.
39. Cold intern can start with PRD and architecture, but architecture's scaffold marker undermines trust.
40. Cold intern can see incident-response and failure-modes docs, satisfying operational substance.
41. Cold intern cannot see CI lane definitions inside service, only references; buildability remains repo-level.
42. Cold intern can infer service owner as `axis-observability` from PRD and manifest.
43. Cold intern cannot infer a complete owner-to-consumer handoff because `cross-microservice-handoffs.md` is absent.
44. Documentation-rigor requires intern-buildable artifacts at `documentation-rigor.md:133-156`; service currently meets concept depth but fails build/deploy reproducibility.
45. Documentation-rigor requires completeness invariants at `documentation-rigor.md:58-129`; service lacks OS/context/IaC manifests now mandated.
46. Substance bar is high for domain exposition and low for canonical substrate compliance.
47. Buildability gap remediation: add `supported-oses.json`, context matrix, OpenTofu modules, canonical build command, missing tests or remove stale ACs.
48. Buildability gap remediation: reconcile manifest bounded contexts, SLO file list, IP list, and demo_trial tenant_class tier shape.
49. Buildability gap remediation: replace Terraform RBAC with OpenTofu module and update compliance/policy references.
50. Dimension 3 conclusion: not intern-buildable end-to-end despite strong conceptual corpus.

### §3.4 Dimension 4 - Canonical-Direction Alignment

1. Multi-context alignment status: drifted-fixable.
2. Evidence: ADR-0328 lists six mandatory contexts and target directories at `ADR-0328:1732-1995`.
3. Evidence: master plan lists the same contexts and IaC targets at `master-plan-sequencing.json:704-745`.
4. Current service evidence: no `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-iaas/` directories.
5. Current service evidence: architecture deployment shape lists Helm files at `ARCHITECTURE.md:449-452`, not ADR-0328 context modules.
6. Classification: P1 because observability is in-scope for all six contexts per user prompt and no correct N/A manifest exists.
7. OpenTofu alignment status: incoherent.
8. Evidence: ADR-0328 says OpenTofu is mandatory and Terraform is only allowed to name the forbidden/superseded engine at `ADR-0328:2243-2249`.
9. Evidence: service has active `iac/terraform/grafana-rbac.tf:1-5` with Terraform source-of-truth and `terraform {`.
10. Evidence: IP-001 says create `microservices/observability/iac/terraform/grafana-rbac.tf`; active design conflict.
11. Evidence: compliance maps access rights to that Terraform file at `compliance.md:86`.
12. Classification: P1 because non-P0 microservice has forbidden IaC engine in active path.
13. OS support alignment status: drifted-fixable.
14. Evidence: ADR-0328 requires `microservices/<name>/supported-oses.json` at `ADR-0328:2907-2927`.
15. Evidence: service inventory has no `supported-oses.json` and no equivalent `supported_oses` manifest field.
16. Tier-1 OS coverage: not evidenced for Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, macOS M5+.
17. Tier-2 OS coverage: not evidenced for ppc64le and s390x test-only lanes.
18. Out-of-scope OS declarations: not evidenced for Intel macOS, M1-M4 Apple Silicon, FreeBSD, OpenBSD, Windows Server, Solaris.
19. Classification: P1 because deployment substrate cannot claim portability without manifest.
20. Rust-strict alignment status: partial.
21. Evidence: ADR-0328 requires Rust for backend/runtime/CLI/codegen/scripting/CI durable behavior at `ADR-0328:3047-3064`.
22. Evidence: forbidden-language file scan under service found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, `.fsx`, `.fsi` files.
23. Evidence: authorized non-Rust files present include Markdown, YAML, JSON, proto, OpenSLO YAML, Cedar, and `.tf`; all except `.tf` are whitelisted by ADR-0328 at `ADR-0328:3085-3107`.
24. Evidence: `.tf` is infrastructure language but canonical engine forbids Terraform naming and use; classify separately under OpenTofu.
25. Evidence: `sdk-plan.md:30-38` and `sdk-plan.md:55-116` prescribe future TypeScript/Python/Go/JVM/C# SDKs.
26. Classification: P2 for doc-level future-language drift and P1 for Terraform under OpenTofu dimension.
27. OCI Always Free alignment status: incoherent.
28. Evidence: ADR-0328 says OCI Always Free is mandatory sub-profile for `guest-on-oci` and demo_trial tenant_class maps to Always Free at `ADR-0328:3493-3507`.
29. Evidence: ADR-0328 requires `iac/oci-guest/always-free/` with module files at `ADR-0328:3666-3677`.
30. Current service evidence: no `iac/oci-guest/always-free/` directory exists.
31. Current tier evidence: demo_trial tenant_class requires multiple collector, ClickHouse, Loki, Tempo, Mimir, and Grafana pods with paid-class resources at `capability-tiers/tier-matrix.md:17-24`.
32. Classification: P1 because demo_trial tenant_class semantics contradict OCI Always Free.
33. ADR-0328 D-20 audit decision tree alignment: audit was executed using all nine dimensions at `ADR-0328:3831-4083`.
34. D-20 evidence requirement for grep evidence is satisfied by no forbidden-language source matches and explicit Terraform match.
35. D-20 evidence requirement for OpenTofu/N/A evidence fails because no context modules and no correct N/A manifest exist.
36. D-20 evidence requirement for supported OS evidence fails because no manifest exists.
37. D-20 evidence requirement for tenant onboarding evidence is partial through onboarding docs but not per-context OpenTofu.
38. Brief-template §3.9 alignment: service should support all six contexts unless hard-blocked; current docs do not prove either support or N/A.
39. Brief-template §3.10 alignment: active Terraform file violates OpenTofu-only.
40. Brief-template §3.11 alignment: supported OS manifest absent.
41. Brief-template §3.12 alignment: current source files are language-clean except Terraform and future SDK documentation drift.
42. Constraint memory `feedback_multi_context_provider_agnostic_2026_05_20.md:10-38` requires all services specify multi-context support; absent here.
43. Constraint memory `feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35` requires OpenTofu-only per-service modules; violated here.
44. Constraint memory `feedback_os_support_matrix_2026_05_20.md:10-77` requires OS matrix; absent here.
45. Constraint memory `feedback_rust_strict_only_no_python_2026_05_20.md:10-64` requires Rust-strict and scan evidence; implementation scan passes, docs drift.
46. Constraint memory `feedback_oci_always_free_maximization_2026_05_20.md:10-98` requires OCI Always Free demo_trial tenant_class; violated here.
47. Canonical alignment headline: the service predates ADR-0328 and needs a substrate ratchet.
48. Most P1 findings are not product-domain disagreements; they are canonical deployment-control disagreements.
49. Corrective path is feasible without deleting domain docs: add OpenTofu modules, OS manifest, Always Free tier mapping, and manifest updates.
50. Dimension 4 conclusion: drifted-fixable in concept but currently blocked on all five cross-cutting constraints except source-language scan.

### §3.5 Dimension 5 - Industry-Counterpart Parity

1. Headline finding: partial, not full union coverage.
2. Datadog official docs list Infrastructure, Metrics, Container Monitoring, Serverless, Network Monitoring, Cloud Cost Management, APM, Universal Service Monitoring, Continuous Profiler, Database Monitoring, Data Streams, Data Observability, Log Management, Sensitive Data Scanner, Observability Pipelines, Error Tracking, RUM, Product Analytics, Session Replay, Synthetic Monitoring, and Mobile App Testing at `https://docs.datadoghq.com/`.
3. Datadog getting-started docs describe APM correlation across logs, infrastructure, service map, traces, RUM, synthetics, dashboards, monitors, and integrations at `https://docs.datadoghq.com/getting_started/application/`.
4. New Relic official docs list Alerts, APM, Browser, Dashboards, Errors inbox, Infrastructure, Kubernetes/Pixie, Logs, Mobile, OpenTelemetry, Serverless, Service levels, Synthetics, and Vulnerability management at `https://docs.newrelic.com/docs/new-relic-solutions/get-started/intro-new-relic/`.
5. New Relic OpenTelemetry APM docs say OpenTelemetry-instrumented services get a comprehensive APM UI and preserve original data for custom dashboards and alerts at `https://docs.newrelic.com/docs/opentelemetry/get-started/apm-monitoring/opentelemetry-apm-ui/`.
6. Grafana Cloud official docs cover metrics, logs, traces, profiles, Alloy collection, application observability, frontend observability, Kubernetes monitoring, private connectivity, and k6 at `https://grafana.com/docs/grafana-cloud/introduction/`.
7. Grafana Cloud trace docs cover TraceQL, traces-to-profiles, trace/log/profile linking, RED metrics from spans, service graphs, and Traces Drilldown at `https://grafana.com/docs/grafana-cloud/send-data/traces/use-traces-with-grafana/`.
8. Oyatie present: OpenSLO native SLO authoring from `PRD.md:39-41`.
9. Oyatie present: burn-rate evaluator and promotion eligibility from `PRD.md:39-48`.
10. Oyatie present: metrics/logs/traces/profiles ingest through Alloy and Grafana stack at `PRD.md:22` and `PRD.md:45`.
11. Oyatie present: dashboards, alerting, Alertmanager, and Grafana OnCall at `PRD.md:31-32` and `PRD.md:46`.
12. Oyatie present: trace analytics and ClickHouse join model from `ADR-OBS-001:55-85`.
13. Oyatie present: compliance/DPIA/threat model depth from inventory.
14. Oyatie present: runbooks for many major operational incidents from inventory.
15. Oyatie present: OpenCost chart files in inventory and `cost-budget.md`.
16. Missing or weak: RUM equivalent comparable to Datadog RUM or New Relic Browser; no browser SDK source, no session model beyond frontend docs.
17. Missing or weak: Session Replay comparable to Datadog Session Replay; no artifact identified.
18. Missing or weak: Synthetic Monitoring comparable to Datadog and New Relic; only accessibility runner chart and no broad synthetic product.
19. Missing or weak: Mobile monitoring comparable to Datadog/New Relic; no mobile telemetry SDK artifacts.
20. Missing or weak: Network performance monitoring comparable to Datadog; no network-flow telemetry product surface, only network policy.
21. Missing or weak: Database monitoring comparable to Datadog/New Relic; ClickHouse is monitored as substrate, not general tenant DBM.
22. Missing or weak: Error tracking comparable to Datadog/New Relic error grouping; SLO/error spans exist but no issue inbox.
23. Missing or weak: Service catalog/internal developer portal comparable to Datadog Software Catalog; Backstage chart exists but no full catalog UX.
24. Missing or weak: Product analytics comparable to Datadog Product Analytics; no product analytics surface.
25. Missing or weak: Cloud Security/Cloud SIEM comparable to Datadog security products; security is policy/compliance, not SIEM.
26. Missing or weak: LLM Observability comparable to Datadog; no model-specific trace schema in observability artifacts.
27. Missing or weak: CI Visibility and test optimization comparable to Datadog; promotion gates exist, but no full CI analytics product.
28. Missing or weak: Observability Pipelines comparable to Datadog; Alloy config exists, but no tenant pipeline builder.
29. Missing or weak: Sensitive Data Scanner comparable to Datadog; policies mention scrubbers, but no full scanner product.
30. Missing or weak: 1,000-plus integration catalog comparable to Datadog; Oyatie chooses OTel and Grafana OSS, not broad vendor integrations.
31. Missing or weak: New Relic Pixie/eBPF auto-telemetry equivalent; no eBPF agent artifact.
32. Missing or weak: Grafana Cloud Frontend Observability equivalent; no Faro/browser artifacts.
33. Missing or weak: Grafana Cloud k6 performance testing equivalent; no k6-like test service.
34. Missing or weak: Grafana Cloud Fleet Management for Alloy; no fleet management controller artifact.
35. Missing or weak: broad private connectivity options across AWS/Azure/GCP documented by Grafana Cloud; no all-context connectivity matrix.
36. Additive surface: signed promotion gate based on SLO evidence at `PRD.md:42-48`.
37. Additive surface: OpenSLO file as release prerequisite at `PRD.md:24` and `PRD.md:39`.
38. Additive surface: Oya VCS release-pointer rollback at `PRD.md:43`.
39. Additive surface: audit-chain evidence anchoring at `PRD.md:72-74`.
40. Additive surface: sovereign-pack telemetry custody at `capability-tiers/tier-matrix.md:107-126`.
41. Additive surface: pack-specific retention for KR-PIPA, EU AI Act, and HIPAA at `capability-tiers/tier-matrix.md:111-116`.
42. Additive surface: Cedar-scoped trace access at `ADR-OBS-001:72-73`.
43. Additive surface: no SaaS portal requirement in tier matrix vendor displacement at `capability-tiers/tier-matrix.md:153`.
44. Counterpart parity cannot be satisfied by just LGTM stack deployment because Datadog/New Relic/Grafana Cloud include user-experience, synthetic, integration, and workflow layers.
45. Counterpart parity also should not blindly import every commercial feature because PRD says observability is shared substrate, not hero product, at `PRD.md:24`.
46. Union-coverage bar for this audit: identify gaps and decide whether Oyatie observability deliberately excludes them or must implement them.
47. Required remediation: add explicit product-scope exclusions for RUM, synthetics, mobile, DBM, and security if they belong to other microservices.
48. Required remediation: add implementation hooks where the service must own features, especially error grouping, service graph, telemetry pipeline, Alloy fleet, and SLO UI.
49. Dimension 5 headline: Oyatie is ahead on release-gating and sovereign evidence, behind on commercial observability breadth.
50. Dimension 5 conclusion: partial parity with high-value additive surface but many union gaps.

### §3.6 Dimension 6 - Multi-Context Deployment Support

1. Required context `oyatie-public-cloud`: not supported by service-local OpenTofu evidence.
2. Required `oyatie-public-cloud` target directory per ADR-0328 is `iac/oyatie-public-cloud/`; absent in inventory.
3. Required context `guest-on-aws`: not supported by service-local OpenTofu evidence.
4. Required `guest-on-aws` target directory is `iac/guest-on-aws/`; absent in inventory.
5. Required context `guest-on-oci`: not supported by canonical module evidence.
6. Required `guest-on-oci` target directory is `iac/oci-guest/`; absent in inventory.
7. Required context `guest-on-oci` Always Free subdir `iac/oci-guest/always-free/`; absent in inventory.
8. Required context `on-prem`: not supported by service-local OpenTofu evidence.
9. Required `on-prem` target directory is `iac/on-prem/`; absent in inventory.
10. Required context `colo`: not supported by service-local OpenTofu evidence.
11. Required `colo` target directory is `iac/colo/`; absent in inventory.
12. Required context `oyatie-as-cloud-provider`: not supported by service-local OpenTofu evidence.
13. Required `oyatie-as-cloud-provider` target directory is `iac/oyatie-iaas/`; absent in inventory.
14. Architecture deployment section says runtime deploys as Kubernetes pods and lists Helm files at `ARCHITECTURE.md:449-452`; Helm is runtime packaging but not a full context matrix.
15. PRD cross-region story is M01 single KR region OCI at `PRD.md:266-268`; this does not satisfy all six contexts.
16. Policy data residency rows are OCI-region-specific at `policy/data-residency.md:34-44`; this is too narrow for provider-agnostic deployment.
17. Cost budget cites OCI public pricing at `cost-budget.md:22-26`; this is too narrow for all-context cost posture.
18. Runbooks list Oracle OCI object storage status desk in external dependencies, e.g. `runbooks/mimir-outage.md:22`; this is too narrow for AWS/on-prem/colo.
19. Kustomize pack-kr overlay has `OCI_OBJECTSTORAGE_ENDPOINT` at `iac/kustomize/overlays/pack-kr/kustomization.yaml:33`; direct provider endpoint in config should be hidden behind context module outputs.
20. Helm values for Mimir/Loki/Tempo/Pyroscope use OCI object storage env names; provider-specific wiring should be module-fed.
21. ADR-0328 forbids cloud-vendor APIs called directly from business logic at `ADR-0328:2192-2211`; no Rust source exists to inspect, but docs/configs are provider-specific.
22. ADR-OBS chooses ClickHouse Cloud canonical production trace store at `ADR-OBS-001:57`; this is not self-evidently valid for on-prem, colo, or air-gapped contexts.
23. ADR-OBS says local fallback is viable at `ADR-OBS-001:41` and future cell-local replacement possible at `ADR-OBS-001:83-85`; needs promotion to context matrix.
24. No service-local N/A record explains why any context is unsupported; ADR-0328 requires explicit N/A fields at `ADR-0328:2079-2084`.
25. No `deployment_contexts` field exists in `manifest.json`.
26. No context labels appear in telemetry docs as required by ADR-0328 anti-patterns at `ADR-0328:2192-2211`; gap severity P2 if telemetry omits context labels.
27. All six contexts are in scope per user prompt unless audit finds otherwise; this audit finds missing evidence, not correct N/A.
28. `oyatie-public-cloud` remediation: add OpenTofu module for Oyatie-operated cloud substrate with outputs for Grafana, Mimir, Loki, Tempo, Alertmanager, OnCall, object storage, and state backend.
29. `guest-on-aws` remediation: add OpenTofu module driven by guest AWS account credentials, no business-logic AWS SDK dependency.
30. `guest-on-oci` remediation: add OpenTofu module for OCI guest tenancy and split Always Free sub-profile.
31. `on-prem` remediation: add OpenTofu module for K8s/on-prem inputs, storage class, object-store endpoint, and offline state backend.
32. `colo` remediation: add OpenTofu module for bare-metal/colo K8s plus S3-compatible storage outputs.
33. `oyatie-as-cloud-provider` remediation: add OpenTofu module targeting Oyatie IaaS primitives.
34. Context module outputs must feed Helm values rather than hardcoding OCI endpoints.
35. Context module state backends must follow ADR-0328 per context.
36. Context modules must emit budget and telemetry labels by context.
37. Context modules must sign plan/module artifacts per sigstore/cosign.
38. Context modules must include README, versions, variables, outputs, and main files per ADR-0328.
39. Existing Helm files can remain as chart packaging beneath OpenTofu orchestration.
40. Existing Kustomize overlays can remain as generated or context-fed overlays if the OpenTofu contract owns substrate.
41. Existing Terraform file cannot remain active.
42. Existing cost budget needs per-context overlays.
43. Existing incident runbooks need context-specific vendor-status substitutions.
44. Existing data residency policy needs provider-agnostic cell identifiers, not only OCI region names.
45. Existing PRD should update M01 single-region statement to distinguish launch seed from all-context support.
46. Existing architecture should add a table for context, supported status, IaC path, state backend, object store, and N/A reason.
47. Severity: P1 for each missing in-scope context module.
48. Severity: P2 for provider-specific docs that can be parameterized.
49. Dimension 6 headline: no canonical multi-context deployment support is evidenced.
50. Dimension 6 conclusion: all six contexts are missing service-local OpenTofu support.

### §3.7 Dimension 7 - OpenTofu IaC Coverage

1. Required engine: OpenTofu, per ADR-0328 `ADR-0328:2243-2249`.
2. Required per-service IaC root: `microservices/observability/iac/`, present.
3. Required context directory `iac/oyatie-public-cloud/`: absent.
4. Required context directory `iac/guest-on-aws/`: absent.
5. Required context directory `iac/oci-guest/`: absent.
6. Required context directory `iac/oci-guest/always-free/`: absent.
7. Required context directory `iac/on-prem/`: absent.
8. Required context directory `iac/colo/`: absent.
9. Required context directory `iac/oyatie-iaas/`: absent.
10. Present IaC directory `iac/helm/`: useful chart packaging, not canonical OpenTofu substrate.
11. Present IaC directory `iac/kustomize/`: useful K8s overlay packaging, not canonical OpenTofu substrate.
12. Present IaC directory `iac/terraform/`: forbidden active engine directory.
13. Terraform reference: `iac/terraform/grafana-rbac.tf:1` says Grafana RBAC managed via Terraform.
14. Terraform reference: `iac/terraform/grafana-rbac.tf:3` says Terraform is source-of-truth.
15. Terraform syntax: `iac/terraform/grafana-rbac.tf:5` begins `terraform {`.
16. Terraform design reference: `IP-001-layer-a-grafana-stack-iac.md:41` instructs creation of Terraform RBAC file.
17. Compliance reference: `compliance.md:86` maps ISO access rights to Terraform file.
18. Policy reference: `policy/tenant-isolation.md:192-198` names Terraform drift detector while also saying OpenTofu apply restores state.
19. OpenTofu positive reference: `iac/helm/grafana/values.yaml:41` says SSO/RBAC declared via OpenTofu; no module backs it.
20. OpenTofu positive reference: `iac/helm/oncall/Chart.yaml:4` says rotation managed via OpenTofu; no module backs it.
21. Forbidden pattern `null_resource`: no evidence in service grep output.
22. Forbidden pattern `local-exec`: no evidence in service grep output.
23. Forbidden pattern `remote-exec`: no evidence in service grep output.
24. Forbidden pattern SSH provisioner: no active provisioner evidence in service grep output.
25. Forbidden pattern hand-edited tfstate: no service evidence.
26. Forbidden pattern unsigned modules: absence of module signing wiring means signing is missing, not necessarily unsigned active modules.
27. Sigstore/cosign wiring: compliance mentions signed images and `.github/workflows/cosign.yml` at `compliance.md:102`, but no service IaC module signing wiring exists.
28. State backend `oyatie-public-cloud`: absent.
29. State backend `guest-on-aws`: absent.
30. State backend `guest-on-oci`: absent.
31. State backend `on-prem`: absent.
32. State backend `colo`: absent.
33. State backend `oyatie-iaas`: absent.
34. Required module files `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, `README.md` per context are absent.
35. The lone `.tf` file is not enough because it is under forbidden `terraform/`, not context directories.
36. Helm chart inventory has enough component coverage to be referenced by OpenTofu modules later.
37. Kustomize overlay inventory can be retained as cluster overlay but not as deployment contract root.
38. ADR-0328 observability-specific example requires outputs identifying dashboard, alert, log sink, and trace collector resources at `ADR-0328:2537-2538`; absent.
39. Tenant onboarding sequence `tofu init -> tofu plan -> tofu apply` required by ADR-0328 `ADR-0328:2436-2459`; absent in service docs.
40. No signed plan workflow exists in service docs.
41. No audit event taxonomy for IaC apply exists in service docs.
42. No module provenance file exists in service IaC tree.
43. No context-specific `README.md` exists under canonical context paths.
44. The service is therefore not ready for Wave-B bootstrap under ADR-0328.
45. P1 finding: active Terraform file.
46. P1 finding: absent OpenTofu context modules.
47. P1 finding: absent OCI Always Free OpenTofu module.
48. P2 finding: Helm/Kustomize docs should be nested under OpenTofu orchestration explanation.
49. Dimension 7 headline: IaC exists as charts/overlays, but canonical OpenTofu coverage is absent.
50. Dimension 7 conclusion: fail until Terraform is retired and six-context OpenTofu modules land.

### §3.8 Dimension 8 - OS Support Matrix

1. Required manifest: `microservices/observability/supported-oses.json`, absent.
2. Alternate supported_oses field in manifest: absent from `manifest.json:1-368`.
3. Tier-1 Talos Linux coverage: not evidenced.
4. Tier-1 RHEL 9+ coverage: not evidenced.
5. Tier-1 Oracle Linux 9+ coverage: not evidenced.
6. Tier-1 SLES 15 SP6+ coverage: not evidenced.
7. Tier-1 Ubuntu 24.04 LTS+ coverage: not evidenced.
8. Tier-1 Debian 13+ coverage: not evidenced.
9. Tier-1 Rocky Linux 9+ coverage: not evidenced.
10. Tier-1 AlmaLinux 9+ coverage: not evidenced.
11. Tier-1 CentOS Stream 10+ coverage: not evidenced.
12. Tier-1 Amazon Linux 2023+ coverage: not evidenced.
13. Tier-1 Flatcar Container Linux coverage: not evidenced.
14. Tier-1 VMware Photon OS 5+ coverage: not evidenced.
15. Tier-1 macOS Apple Silicon M5+ dev/support host coverage: not evidenced.
16. Tier-2 ppc64le test-only coverage: not evidenced.
17. Tier-2 s390x test-only coverage: not evidenced.
18. Out-of-scope Intel macOS explicit declaration: absent.
19. Out-of-scope pre-M5 Apple Silicon explicit declaration: absent.
20. Out-of-scope FreeBSD explicit declaration: absent.
21. Out-of-scope OpenBSD explicit declaration: absent.
22. Out-of-scope Windows Server explicit declaration: absent.
23. Out-of-scope Solaris explicit declaration: absent.
24. Package format RPM: not evidenced.
25. Package format DEB: not evidenced.
26. Package format `.pkg`: not evidenced.
27. Package format Homebrew formula: not evidenced.
28. Package format Talos extension: not evidenced.
29. Package format Flatcar extension: not evidenced.
30. Package format container image: implied by Helm charts and PRD Kubernetes pods, but no OS manifest binds it.
31. CI lane Talos: not evidenced.
32. CI lane RHEL: not evidenced.
33. CI lane Oracle Linux: not evidenced.
34. CI lane SLES: not evidenced.
35. CI lane Ubuntu: not evidenced.
36. CI lane Debian: not evidenced.
37. CI lane Rocky: not evidenced.
38. CI lane AlmaLinux: not evidenced.
39. CI lane CentOS Stream: not evidenced.
40. CI lane Amazon Linux: not evidenced.
41. CI lane Flatcar: not evidenced.
42. CI lane Photon: not evidenced.
43. CI lane macOS M5+: not evidenced.
44. CI lane ppc64le/s390x soft gate: not evidenced.
45. PRD acceptance criteria list `kind` cluster smoke at `PRD.md:285`, but not OS matrix.
46. Architecture deployment shape says Kubernetes pods at `ARCHITECTURE.md:449-452`, but not host OS support.
47. ADR-0328 requires OS evidence even for services whose runtime is containerized because substrate and packaging differ.
48. Remediation: add `supported-oses.json` with tier, arch, package format, CI lane, container image support, and out-of-scope declarations.
49. Dimension 8 headline: OS support is undocumented rather than contradicted.
50. Dimension 8 conclusion: P1 missing manifest and P2 missing package/CI detail.

### §3.9 Dimension 9 - Rust-Strict Language Coverage

1. ADR-0328 Rust-strict source: `ADR-0328:3047-3064` requires Rust for backend/runtime/CLI/codegen/scripting/CI durable behavior.
2. Authorized non-Rust source extensions source: `ADR-0328:3085-3107`.
3. Forbidden-language scan under `microservices/observability/` found no `.py` files.
4. Forbidden-language scan under `microservices/observability/` found no `.js` files.
5. Forbidden-language scan under `microservices/observability/` found no `.ts` files.
6. Forbidden-language scan under `microservices/observability/` found no `.rb` files.
7. Forbidden-language scan under `microservices/observability/` found no `.go` files.
8. Forbidden-language scan under `microservices/observability/` found no `.java` files.
9. Forbidden-language scan under `microservices/observability/` found no `.scala` files.
10. Forbidden-language scan under `microservices/observability/` found no `.groovy` files.
11. Forbidden-language scan under `microservices/observability/` found no `.php` files.
12. Forbidden-language scan under `microservices/observability/` found no `.fs`, `.fsx`, or `.fsi` files.
13. Markdown files are authorized documentation artifacts.
14. YAML files are authorized contract/config artifacts, including OpenAPI, AsyncAPI, Helm, Kustomize, and OpenSLO.
15. JSON files are authorized machine-readable artifacts and dashboards.
16. Proto file `contracts/proto/slo-engine.proto` is authorized.
17. Cedar files under `policy/` are authorized policy files.
18. OpenSLO YAML files under `slos/` are authorized SLO artifacts.
19. `.tf` file under `iac/terraform/grafana-rbac.tf` is not a forbidden application language file, but it is forbidden by OpenTofu-only engine policy.
20. No Swift files found; no frontend Swift path exists.
21. No Kotlin files found; no frontend Android path exists.
22. No WinUI3/C# files found; no frontend Windows path exists.
23. No `frontend/<platform>/` directory exists; no frontend exception applies.
24. No generated SDK output directories exist under service path.
25. PRD says Rust SDK plus future TS/Python bindings at `PRD.md:121-123`; this is doc drift under Rust-strict unless generated-SDK exception is formalized.
26. SDK plan schedules TypeScript at M01+1, Python and Go at M02, JVM at M03 at `sdk-plan.md:30-33`; this needs an exception matrix.
27. SDK plan says generated SDKs include TypeScript/Python/Go/JVM/C# at `sdk-plan.md:55`; this needs provenance, output isolation, and non-runtime status.
28. SDK plan compatibility example includes TypeScript SDK at `sdk-plan.md:116`; doc-level drift.
29. Competitor parity matrix names multi-language SDK breadth as a gap at `competitor-parity-matrix.md:76` and `competitor-parity-matrix.md:97`; doc-level drift.
30. Canonical build invocation required by ADR-0328 is `cargo build --workspace --release --all-features --locked`.
31. PRD acceptance uses `cargo run -p ...` commands at `PRD.md:278-287`, not a canonical build invocation.
32. Phase-01 commands include `cargo check`, `cargo build`, `cargo clippy`, `cargo nextest`, `cargo doc` at `PHASE-01:91-96`, but not canonical locked release build.
33. IP-009 uses package-specific `cargo build -p ... --release` at `IP-009:62`, not canonical workspace locked build.
34. Reference implementation is Rust-focused and aligns with policy.
35. Catalog crate files are YAML records for Rust crates and align with policy.
36. No shell scripts found under service inventory; this is good under durable scripting restriction.
37. No Makefile found under service inventory; this is good.
38. No package.json found under service inventory; this is good.
39. No Gradle/Maven/Go module files found under service inventory; this is good.
40. No Python requirements/pyproject files found under service inventory; this is good.
41. Language-policy headline: implementation artifacts are clean; documentation future-roadmap is not.
42. P1 language finding is not warranted for current source files because no forbidden source exists.
43. P1 OpenTofu finding remains for `.tf`, but it belongs to IaC engine not backend language.
44. P2 finding: multi-language SDK docs need ADR-0328-compliant generated-output exception boundaries.
45. P2 finding: canonical build invocation missing in service docs.
46. Remediation: update SDK plan to Rust-first, generated-output-only, isolated path, provenance, and no runtime dependency.
47. Remediation: update PRD/Phase/IP build sections to include canonical locked release build.
48. Remediation: add CI evidence lanes for language scan and canonical build.
49. Dimension 9 headline: current files pass forbidden-language scan, but future SDK and build docs drift.
50. Dimension 9 conclusion: partial alignment with P2 doc fixes and P1 IaC-engine fix tracked elsewhere.

## §4 Findings Summary

| Severity | Dimension | Short description | Citation | Remediation hint |
|---|---|---|---|---|
| P1 | 6/7 | Missing all six canonical OpenTofu context directories | `ADR-0328:2275-2294`; service inventory | Add `iac/oyatie-public-cloud`, `guest-on-aws`, `oci-guest`, `on-prem`, `colo`, `oyatie-iaas` modules. |
| P1 | 7 | Active Terraform IaC under service path | `iac/terraform/grafana-rbac.tf:1-5` | Replace with OpenTofu module and retire Terraform references. |
| P1 | 7 | Compliance/policy still points to Terraform | `compliance.md:86`; `policy/tenant-isolation.md:192-198` | Update control mapping after OpenTofu migration. |
| P1 | 8 | Missing `supported-oses.json` | `ADR-0328:2907-2927`; service inventory | Add OS matrix manifest with Tier-1/Tier-2/out-of-scope declarations. |
| P1 | 6/7 | Missing OCI Always Free module | `ADR-0328:3666-3677`; service inventory | Add `iac/oci-guest/always-free/` with module files and budget outputs. |
| P1 | 4/6 | demo_trial tenant_class tier contradicts OCI Always Free | `capability-tiers/tier-matrix.md:17-24`; `ADR-0328:3493-3507` | Split OCI Always Free demo_trial tenant_class Always Free from paid demo_trial tenant_class/paid tenant_class baseline capacity. |
| P1 | 6 | ClickHouse Cloud canonical production store conflicts with on-prem/colo unless context-local path lands | `ADR-OBS-001:55-85` | Add context-specific ClickHouse deployment modes and promote cell-local plan. |
| P2 | 1 | Architecture still says Wave-3-C stub expansion pending | `ARCHITECTURE.md:3` | Remove scaffold marker after substantive rewrite. |
| P2 | 1 | Placeholder-like table/event names | `ARCHITECTURE.md:141-144` | Replace with actual storage/event surface names or migrations. |
| P2 | 1 | Manifest bounded contexts conflict with PRD | `PRD.md:90-94`; `manifest.json:6-28` | Align manifest to `slo-engine` and `otel-ingest`. |
| P2 | 1 | Manifest omits two SLO files | `manifest.json:71-108`; inventory | Add clickhouse and tail-sampling SLOs to manifest. |
| P2 | 1 | Manifest IP list stale | `manifest.json:109-200`; inventory | Add IP-021 through IP-031 and journey plans or separate indexes. |
| P2 | 1 | Tier taxonomy conflict T0/T1/T2 vs demo_trial tenant_class/paid tenant_class baseline/paid tenant_class scale/compliance_pack-gated paid tenant_class | `manifest.json:293-297`; `capability-tiers/tier-matrix.md:15-126` | Separate capability-risk tiers from commercial/capacity tiers. |
| P2 | 3 | Missing service tests path cited by PRD | `PRD.md:279`; service inventory | Add tests or update acceptance criteria to real paths. |
| P2 | 2 | Missing error-budget runbook cited by PRD | `PRD.md:244`; inventory | Add runbook or update citation. |
| P2 | 2 | Missing cross-region restore runbook cited by tier matrix | `capability-tiers/tier-matrix.md:139`; inventory | Add runbook or update promotion path. |
| P2 | 2 | Dashboard provisioning path mismatch | `capability-tiers/tier-matrix.md:56`; inventory | Point to `dashboards/` or create provisioning path. |
| P2 | 5 | Counterpart scope stale and missing New Relic in PRD matrix | `PRD.md:217-223` | Replace with Datadog/New Relic/Grafana Cloud union matrix. |
| P2 | 5 | Missing RUM/session replay/synthetic/mobile/DBM/error inbox parity | official counterpart docs; service inventory | Decide ownership or exclusions and add implementation hooks. |
| P2 | 9 | Future multi-language SDK docs drift from Rust-strict | `sdk-plan.md:30-38`; `sdk-plan.md:55-116` | Add generated SDK exception matrix and isolate outputs. |
| P2 | 9 | Canonical build invocation absent | `PHASE-01:91-96`; `IP-009:62` | Add `cargo build --workspace --release --all-features --locked`. |
| P2 | 6 | Provider-specific OCI docs/configs dominate all-context story | `policy/data-residency.md:34-44`; `cost-budget.md:22-26` | Add provider-agnostic context overlays. |
| P3 | 1 | No top-level service README | service inventory | Add concise README pointing to PRD, architecture, contracts, runbooks. |
| P3 | 1 | Manifest LTS pin strings contain embedded quote/comment text | `manifest.json:214-222` | Normalize strings or structured fields. |
| P3 | 1 | Local ADR status proposed while PRD is accepted | `ADR-OBS-001:4`; `PRD.md:6` | Promote/reject ADR or mark as nonbinding. |
| P3 | 2 | No migration playbooks for New Relic or Grafana Cloud | inventory | Add or document exclusion. |
| P3 | 1 | `scorecards/overrides.json` role not integrated | inventory | Reference from manifest or explain override lifecycle. |

Severity totals:
P0: 0.
P1: 7.
P2: 15.
P3: 5.

## §5 Open Questions for Wave 14 Aggregation

1. Should observability own RUM/session replay/mobile/synthetics, or should those be explicit responsibilities of another product microservice?
2. Should ClickHouse Cloud remain acceptable for public-cloud contexts while on-prem/colo use self-hosted ClickHouse, or must all contexts use one in-house substrate?
3. Should demo_trial tenant_class be globally redefined as OCI Always Free, or should the current paid demo_trial tenant_class become paid tenant_class baseline while OCI Always Free demo_trial tenant_class is a separate sub-profile?
4. Should the manifest split `risk_class`/autonomy tiers (`T0/T1/T2`) from capability commercial tiers (demo_trial tenant_class/paid tenant_class baseline/paid tenant_class scale/compliance_pack-gated paid tenant_class)?
5. Should generated external-language SDKs be allowed for tenant integrations, and if yes, what ADR records the exception and output isolation?
6. Which repo-level CI lane owns OS matrix verification for containerized microservices?
7. Which microservice owns broad Datadog/New Relic/Grafana Cloud migration playbooks beyond Datadog?
8. Does ADR-0263 require a reverse-reference manifest entry in every microservice, or only an observability emission contract?
9. Should `cross-microservice-handoffs.md` become mandatory for substrate services consumed by all microservices?
10. Should existing Helm/Kustomize charts be treated as lower-level implementation detail under OpenTofu, or should some be retired in favor of module-generated manifests?
<!-- ORCHESTRATOR REPORT
  µservice: observability
  deliverables_landed:
    - microservices/observability/coherence-audit-2026-05-20.md: 830 lines
    - microservices/observability/feature-parity-matrix-2026-05-20.md: 421 lines
    - microservices/observability/performance-benchmark-numbers-2026-05-20.md: 507 lines
    - microservices/observability/capability-tier-deltas-vs-counterparts-2026-05-20.md: 362 lines
  inventory_files_seen: 160
  inventory_lines_read: 45392
  chat_history_matches_processed: 5
  findings_p0: 0
  findings_p1: 7
  findings_p2: 15
  findings_p3: 5
  top_3_counterparts_confirmed: Datadog / New Relic / Grafana Cloud
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 2120
-->
