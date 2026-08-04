# G025 libs disposition draft — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`
Authority: origin/dev path census only. Heuristic dispositions are NOT admitted move plans.

Total libs Cargo.toml: **129**

## Disposition counters (draft)
- `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL`: 70
- `KEEP_IN_LIBS_PENDING`: 19
- `KEEP_OR_SPLIT_BY_CAPABILITY`: 19
- `MOVE_TO_GOVERNANCE_OR_CI_CHECK`: 16
- `MOVE_TO_CI_FACILITY`: 5

## Rules for turning this into executable move plans
1. One capability (or one KEEP bucket) per plan file; codemod schema only.
2. Catalog ArtifactMove 1:1 with cargo rename stems under registry/catalog/.
3. Face grammar core/ports/adapters/facade only where the destination capability defines those faces.
4. No permanent dual authority; execution is atomic after independent review + protected CI + promoted observation.
5. `oya-check-*` mass should prefer governance/check or ci/facade patterns already landed (see governance-check-move-plan.json), not a new mega-libs face.

## Draft rows
| crate | path | draft_disposition | draft_destination |
|---|---|---|---|
| `oya-advisory-mirror-kernel` | `libs/oya-advisory-mirror-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-buck-syntax-kernel` | `libs/oya-buck-syntax-kernel/Cargo.toml` | `MOVE_TO_CI_FACILITY` | ci/facade/* |
| `oya-cargo-lock-transform-kernel` | `libs/oya-cargo-lock-transform-kernel/Cargo.toml` | `MOVE_TO_CI_FACILITY` | ci/facade/* |
| `oya-check-adr-index` | `libs/oya-check-adr-index/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-adr-placeholders` | `libs/oya-check-adr-placeholders/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-backup-retention-discipline` | `libs/oya-check-backup-retention-discipline/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-brand-residue` | `libs/oya-check-brand-residue/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-claim-ceiling` | `libs/oya-check-claim-ceiling/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-compliance-evidence-coverage` | `libs/oya-check-compliance-evidence-coverage/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-container-base-image` | `libs/oya-check-container-base-image/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-cost-budget` | `libs/oya-check-cost-budget/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-dependency-seam` | `libs/oya-check-dependency-seam/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-doc-axis` | `libs/oya-check-doc-axis/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-i18n-coverage` | `libs/oya-check-i18n-coverage/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-license-policy` | `libs/oya-check-license-policy/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-realtime-transport-tier` | `libs/oya-check-realtime-transport-tier/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-saga-shape` | `libs/oya-check-saga-shape/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-slo-coverage` | `libs/oya-check-slo-coverage/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-check-step-up-auth-coverage` | `libs/oya-check-step-up-auth-coverage/Cargo.toml` | `MOVE_TO_GOVERNANCE_OR_CI_CHECK` | governance/check/* or ci/facade/* (existing pattern) |
| `oya-ci-config` | `libs/oya-ci-config/Cargo.toml` | `MOVE_TO_CI_FACILITY` | ci/facade/* |
| `oya-ci-gate-contract` | `libs/oya-ci-gate-contract/Cargo.toml` | `MOVE_TO_CI_FACILITY` | ci/facade/* |
| `oya-ci-materializer-kernel` | `libs/oya-ci-materializer-kernel/Cargo.toml` | `MOVE_TO_CI_FACILITY` | ci/facade/* |
| `oya-crate-registrar-app` | `libs/oya-crate-registrar-app/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-crate-registrar-kernel` | `libs/oya-crate-registrar-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-data-boundary-kernel` | `libs/oya-data-boundary-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-data-outbox-adapter-postgres` | `libs/oya-data-outbox-adapter-postgres/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-data-outbox-kernel` | `libs/oya-data-outbox-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-data-sql-adapter-sqlx` | `libs/oya-data-sql-adapter-sqlx/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-data-sql-kernel` | `libs/oya-data-sql-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-adapter-with-no-importer-kernel` | `libs/oya-governance-adapter-with-no-importer-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-adr-shape-kernel` | `libs/oya-governance-adr-shape-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-agentic-navigability-kernel` | `libs/oya-governance-agentic-navigability-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-architecture-map-freshness-kernel` | `libs/oya-governance-architecture-map-freshness-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-audit-event-emission` | `libs/oya-governance-audit-event-emission/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-authoritative-tracked-kernel` | `libs/oya-governance-authoritative-tracked-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-banned-primitives-kernel` | `libs/oya-governance-banned-primitives-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-byok-disambiguation` | `libs/oya-governance-byok-disambiguation/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-bypass-kernel` | `libs/oya-governance-bypass-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-capability-tier-coverage` | `libs/oya-governance-capability-tier-coverage/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-cedar-coverage` | `libs/oya-governance-cedar-coverage/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-claim-ceiling-kernel` | `libs/oya-governance-claim-ceiling-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-cohesion-kernel` | `libs/oya-governance-cohesion-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-doc-freshness-kernel` | `libs/oya-governance-doc-freshness-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-doc-style-kernel` | `libs/oya-governance-doc-style-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-eval-domain` | `libs/oya-governance-eval-domain/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-eval-usecase` | `libs/oya-governance-eval-usecase/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-gate-catalog-domain` | `libs/oya-governance-gate-catalog-domain/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-image-discipline-kernel` | `libs/oya-governance-image-discipline-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-license-policy-kernel` | `libs/oya-governance-license-policy-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-lifecycle-kernel` | `libs/oya-governance-lifecycle-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-mistakes-ledger-kernel` | `libs/oya-governance-mistakes-ledger-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-naming-justifications` | `libs/oya-governance-naming-justifications/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-no-template-stamping` | `libs/oya-governance-no-template-stamping/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-orphan-detection-kernel` | `libs/oya-governance-orphan-detection-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-pack-overlay-completeness` | `libs/oya-governance-pack-overlay-completeness/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-portfolio-citation-kernel` | `libs/oya-governance-portfolio-citation-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-pr-merge-gate-kernel` | `libs/oya-governance-pr-merge-gate-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-pr-traceability-kernel` | `libs/oya-governance-pr-traceability-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-pre-push-kernel` | `libs/oya-governance-pre-push-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-predictable-naming-kernel` | `libs/oya-governance-predictable-naming-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-provider-coupling-kernel` | `libs/oya-governance-provider-coupling-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-purpose-kernel` | `libs/oya-governance-purpose-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-quality-lane-kernel` | `libs/oya-governance-quality-lane-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-substance-bar` | `libs/oya-governance-substance-bar/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-governance-sunset-lifecycle-kernel` | `libs/oya-governance-sunset-lifecycle-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-supply-chain-kernel` | `libs/oya-governance-supply-chain-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-tos-policy-kernel` | `libs/oya-governance-tos-policy-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-governance-upstream-api-drift-kernel` | `libs/oya-governance-upstream-api-drift-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-http-latency-budget-middleware-infrastructure` | `libs/oya-http-latency-budget-middleware-infrastructure/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-http-middleware-kernel` | `libs/oya-http-middleware-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-http-router-kernel` | `libs/oya-http-router-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-http-runtime-hyper-adapter` | `libs/oya-http-runtime-hyper-adapter/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-http-sse-kernel` | `libs/oya-http-sse-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-http-telemetry-middleware-infrastructure` | `libs/oya-http-telemetry-middleware-infrastructure/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-http-tenant-middleware-infrastructure` | `libs/oya-http-tenant-middleware-infrastructure/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-http-wide-event-middleware-infrastructure` | `libs/oya-http-wide-event-middleware-infrastructure/Cargo.toml` | `KEEP_IN_LIBS_PENDING` |  |
| `oya-json-kernel` | `libs/oya-json-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-architecture-check-cli` | `libs/oya-shared-architecture-check-cli/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-audit-chain-client-kernel` | `libs/oya-shared-audit-chain-client-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-audit-digest-adapter-awslc` | `libs/oya-shared-audit-digest-adapter-awslc/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-audit-event-kernel` | `libs/oya-shared-audit-event-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-backbone-grpc-generated-adapter` | `libs/oya-shared-backbone-grpc-generated-adapter/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-backbone-grpc-transport-adapter` | `libs/oya-shared-backbone-grpc-transport-adapter/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-backbone-proto-contracts-kernel` | `libs/oya-shared-backbone-proto-contracts-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-backbone-rest-runtime-adapter` | `libs/oya-shared-backbone-rest-runtime-adapter/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-backup-kernel` | `libs/oya-shared-backup-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-bounded-contexts-check-cli` | `libs/oya-shared-bounded-contexts-check-cli/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-compliance-evidence-kernel` | `libs/oya-shared-compliance-evidence-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-connector-kernel` | `libs/oya-shared-connector-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-cursor-pagination-kernel` | `libs/oya-shared-cursor-pagination-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-email-comms-kernel` | `libs/oya-shared-email-comms-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-hyperscaler-metrics-adapter-otlp` | `libs/oya-shared-hyperscaler-metrics-adapter-otlp/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-hyperscaler-metrics-adapter-prometheus` | `libs/oya-shared-hyperscaler-metrics-adapter-prometheus/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-hyperscaler-metrics-kernel` | `libs/oya-shared-hyperscaler-metrics-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-i18n-kernel` | `libs/oya-shared-i18n-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-idempotency-key-kernel` | `libs/oya-shared-idempotency-key-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-oidc-client-kernel` | `libs/oya-shared-oidc-client-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-olap-clickhouse-adapter` | `libs/oya-shared-olap-clickhouse-adapter/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-olap-client-kernel` | `libs/oya-shared-olap-client-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-outbox-broker-http-adapter` | `libs/oya-shared-outbox-broker-http-adapter/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-outbox-pattern-kernel` | `libs/oya-shared-outbox-pattern-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-pdp-kernel` | `libs/oya-shared-pdp-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-platform-contracts-kernel` | `libs/oya-shared-platform-contracts-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-postgres-command-adapter-sqlx` | `libs/oya-shared-postgres-command-adapter-sqlx/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-postgres-command-kernel` | `libs/oya-shared-postgres-command-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-presence-kernel` | `libs/oya-shared-presence-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-protocol-parity-kernel` | `libs/oya-shared-protocol-parity-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-protocol-transport-kernel` | `libs/oya-shared-protocol-transport-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-protocol-transport-retry-app` | `libs/oya-shared-protocol-transport-retry-app/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-realtime-transport-kernel` | `libs/oya-shared-realtime-transport-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-resource-provider-contract-kernel` | `libs/oya-shared-resource-provider-contract-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-scim-server-kernel` | `libs/oya-shared-scim-server-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-semver-check-cli` | `libs/oya-shared-semver-check-cli/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-supply-chain-check-cli` | `libs/oya-shared-supply-chain-check-cli/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-tenant-quota-kernel` | `libs/oya-shared-tenant-quota-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-timeseries-kernel` | `libs/oya-shared-timeseries-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-tracing-client-kernel` | `libs/oya-shared-tracing-client-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-transactional-outbox-adapter-sqlx` | `libs/oya-shared-transactional-outbox-adapter-sqlx/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-transactional-outbox-dispatch-app` | `libs/oya-shared-transactional-outbox-dispatch-app/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-transactional-outbox-kernel` | `libs/oya-shared-transactional-outbox-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-transactional-outbox-poller-app` | `libs/oya-shared-transactional-outbox-poller-app/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-transactional-outbox-runtime-tokio-app` | `libs/oya-shared-transactional-outbox-runtime-tokio-app/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-transactional-outbox-worker-app` | `libs/oya-shared-transactional-outbox-worker-app/Cargo.toml` | `KEEP_OR_SPLIT_BY_CAPABILITY` | split if capability-specific; keep only true cross-cutting kernels |
| `oya-shared-ulid-id-kernel` | `libs/oya-shared-ulid-id-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-vector-store-kernel` | `libs/oya-shared-vector-store-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-wasm-runtime-kernel` | `libs/oya-shared-wasm-runtime-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-webauthn-server-kernel` | `libs/oya-shared-webauthn-server-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-shared-webhook-delivery-kernel` | `libs/oya-shared-webhook-delivery-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
| `oya-workspace-members-kernel` | `libs/oya-workspace-members-kernel/Cargo.toml` | `MOVE_TO_CAPABILITY_CORE_OR_LIBS_KERNEL` | capability core/ or retain pure kernel in libs only if cross-cutting |
