# G025 libs disposition — existing-plan coverage — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`

- Live `libs/**/Cargo.toml`: **129**
- Existing `specs/reorg/governance-check-move-plan.json` moves: **56**
- Live libs rows already covered by that executable-plan artifact: **0**
- Live libs rows not covered by it: **129**

## Correction to the earlier heuristic draft
The earlier name-only draft classified check-like crates but did not subtract the already-authored governance-check plan. This supplement is authoritative over that draft for plan coverage. Existing plan rows must not be duplicated in a G025 plan.

## Covered by existing governance-check plan
| old | destination |
|---|---|

## Remaining group counts
- `kernel-like`: 73
- `other`: 19
- `shared-like`: 19
- `check-like-unplanned`: 16
- `ci-build-like`: 2

## Remaining live manifests
| class | path |
|---|---|
| `kernel-like` | `libs/oya-advisory-mirror-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-buck-syntax-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-cargo-lock-transform-kernel/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-adr-index/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-adr-placeholders/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-backup-retention-discipline/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-brand-residue/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-claim-ceiling/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-compliance-evidence-coverage/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-container-base-image/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-cost-budget/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-dependency-seam/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-doc-axis/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-i18n-coverage/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-license-policy/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-realtime-transport-tier/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-saga-shape/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-slo-coverage/Cargo.toml` |
| `check-like-unplanned` | `libs/oya-check-step-up-auth-coverage/Cargo.toml` |
| `ci-build-like` | `libs/oya-ci-config/Cargo.toml` |
| `ci-build-like` | `libs/oya-ci-gate-contract/Cargo.toml` |
| `kernel-like` | `libs/oya-ci-materializer-kernel/Cargo.toml` |
| `other` | `libs/oya-crate-registrar-app/Cargo.toml` |
| `kernel-like` | `libs/oya-crate-registrar-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-data-boundary-kernel/Cargo.toml` |
| `other` | `libs/oya-data-outbox-adapter-postgres/Cargo.toml` |
| `kernel-like` | `libs/oya-data-outbox-kernel/Cargo.toml` |
| `other` | `libs/oya-data-sql-adapter-sqlx/Cargo.toml` |
| `kernel-like` | `libs/oya-data-sql-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-adapter-with-no-importer-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-adr-shape-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-agentic-navigability-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-architecture-map-freshness-kernel/Cargo.toml` |
| `other` | `libs/oya-governance-audit-event-emission/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-authoritative-tracked-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-banned-primitives-kernel/Cargo.toml` |
| `other` | `libs/oya-governance-byok-disambiguation/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-bypass-kernel/Cargo.toml` |
| `other` | `libs/oya-governance-capability-tier-coverage/Cargo.toml` |
| `other` | `libs/oya-governance-cedar-coverage/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-claim-ceiling-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-cohesion-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-doc-freshness-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-doc-style-kernel/Cargo.toml` |
| `other` | `libs/oya-governance-eval-domain/Cargo.toml` |
| `other` | `libs/oya-governance-eval-usecase/Cargo.toml` |
| `other` | `libs/oya-governance-gate-catalog-domain/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-image-discipline-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-license-policy-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-lifecycle-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-mistakes-ledger-kernel/Cargo.toml` |
| `other` | `libs/oya-governance-naming-justifications/Cargo.toml` |
| `other` | `libs/oya-governance-no-template-stamping/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-orphan-detection-kernel/Cargo.toml` |
| `other` | `libs/oya-governance-pack-overlay-completeness/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-portfolio-citation-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-pr-merge-gate-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-pr-traceability-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-pre-push-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-predictable-naming-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-provider-coupling-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-purpose-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-quality-lane-kernel/Cargo.toml` |
| `other` | `libs/oya-governance-substance-bar/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-sunset-lifecycle-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-supply-chain-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-tos-policy-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-governance-upstream-api-drift-kernel/Cargo.toml` |
| `other` | `libs/oya-http-latency-budget-middleware-infrastructure/Cargo.toml` |
| `kernel-like` | `libs/oya-http-middleware-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-http-router-kernel/Cargo.toml` |
| `other` | `libs/oya-http-runtime-hyper-adapter/Cargo.toml` |
| `kernel-like` | `libs/oya-http-sse-kernel/Cargo.toml` |
| `other` | `libs/oya-http-telemetry-middleware-infrastructure/Cargo.toml` |
| `other` | `libs/oya-http-tenant-middleware-infrastructure/Cargo.toml` |
| `other` | `libs/oya-http-wide-event-middleware-infrastructure/Cargo.toml` |
| `kernel-like` | `libs/oya-json-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-architecture-check-cli/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-audit-chain-client-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-audit-digest-adapter-awslc/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-audit-event-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-backbone-grpc-generated-adapter/Cargo.toml` |
| `shared-like` | `libs/oya-shared-backbone-grpc-transport-adapter/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-backbone-proto-contracts-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-backbone-rest-runtime-adapter/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-backup-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-bounded-contexts-check-cli/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-compliance-evidence-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-connector-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-cursor-pagination-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-email-comms-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-hyperscaler-metrics-adapter-otlp/Cargo.toml` |
| `shared-like` | `libs/oya-shared-hyperscaler-metrics-adapter-prometheus/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-hyperscaler-metrics-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-i18n-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-idempotency-key-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-oidc-client-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-olap-clickhouse-adapter/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-olap-client-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-outbox-broker-http-adapter/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-outbox-pattern-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-pdp-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-platform-contracts-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-postgres-command-adapter-sqlx/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-postgres-command-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-presence-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-protocol-parity-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-protocol-transport-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-protocol-transport-retry-app/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-realtime-transport-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-resource-provider-contract-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-scim-server-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-semver-check-cli/Cargo.toml` |
| `shared-like` | `libs/oya-shared-supply-chain-check-cli/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-tenant-quota-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-timeseries-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-tracing-client-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-transactional-outbox-adapter-sqlx/Cargo.toml` |
| `shared-like` | `libs/oya-shared-transactional-outbox-dispatch-app/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-transactional-outbox-kernel/Cargo.toml` |
| `shared-like` | `libs/oya-shared-transactional-outbox-poller-app/Cargo.toml` |
| `shared-like` | `libs/oya-shared-transactional-outbox-runtime-tokio-app/Cargo.toml` |
| `shared-like` | `libs/oya-shared-transactional-outbox-worker-app/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-ulid-id-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-vector-store-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-wasm-runtime-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-webauthn-server-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-shared-webhook-delivery-kernel/Cargo.toml` |
| `kernel-like` | `libs/oya-workspace-members-kernel/Cargo.toml` |

## Activation rules
1. Do not duplicate any of the 56 governance-check rows.
2. Remaining rows need importer/dependency evidence before MOVE/REFACTOR/REWRITE/DELETE/KEEP disposition.
3. One destination capability per executable plan; no generic libs mega-capability.
4. No activation before the G019 corpus repair is promoted and observed green.
