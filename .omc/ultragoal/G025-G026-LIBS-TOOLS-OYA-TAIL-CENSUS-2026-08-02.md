# G025/G026 libs + tools + oya product-tail census — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`
Authority: `origin/dev` tree only (no canonical dirty checkout trust).

## Counts
- libs Cargo.toml: **129**
- tools Cargo.toml: **21**
- oya Cargo.toml: **180**
- existing reorg plans under specs/reorg: 10

## libs top-level buckets (by path segment after libs/)
- `oya-advisory-mirror-kernel`: 1
- `oya-buck-syntax-kernel`: 1
- `oya-cargo-lock-transform-kernel`: 1
- `oya-check-adr-index`: 1
- `oya-check-adr-placeholders`: 1
- `oya-check-backup-retention-discipline`: 1
- `oya-check-brand-residue`: 1
- `oya-check-claim-ceiling`: 1
- `oya-check-compliance-evidence-coverage`: 1
- `oya-check-container-base-image`: 1
- `oya-check-cost-budget`: 1
- `oya-check-dependency-seam`: 1
- `oya-check-doc-axis`: 1
- `oya-check-i18n-coverage`: 1
- `oya-check-license-policy`: 1
- `oya-check-realtime-transport-tier`: 1
- `oya-check-saga-shape`: 1
- `oya-check-slo-coverage`: 1
- `oya-check-step-up-auth-coverage`: 1
- `oya-ci-config`: 1
- `oya-ci-gate-contract`: 1
- `oya-ci-materializer-kernel`: 1
- `oya-crate-registrar-app`: 1
- `oya-crate-registrar-kernel`: 1
- `oya-data-boundary-kernel`: 1
- `oya-data-outbox-adapter-postgres`: 1
- `oya-data-outbox-kernel`: 1
- `oya-data-sql-adapter-sqlx`: 1
- `oya-data-sql-kernel`: 1
- `oya-governance-adapter-with-no-importer-kernel`: 1
- `oya-governance-adr-shape-kernel`: 1
- `oya-governance-agentic-navigability-kernel`: 1
- `oya-governance-architecture-map-freshness-kernel`: 1
- `oya-governance-audit-event-emission`: 1
- `oya-governance-authoritative-tracked-kernel`: 1
- `oya-governance-banned-primitives-kernel`: 1
- `oya-governance-byok-disambiguation`: 1
- `oya-governance-bypass-kernel`: 1
- `oya-governance-capability-tier-coverage`: 1
- `oya-governance-cedar-coverage`: 1
- `oya-governance-claim-ceiling-kernel`: 1
- `oya-governance-cohesion-kernel`: 1
- `oya-governance-doc-freshness-kernel`: 1
- `oya-governance-doc-style-kernel`: 1
- `oya-governance-eval-domain`: 1
- `oya-governance-eval-usecase`: 1
- `oya-governance-gate-catalog-domain`: 1
- `oya-governance-image-discipline-kernel`: 1
- `oya-governance-license-policy-kernel`: 1
- `oya-governance-lifecycle-kernel`: 1
- `oya-governance-mistakes-ledger-kernel`: 1
- `oya-governance-naming-justifications`: 1
- `oya-governance-no-template-stamping`: 1
- `oya-governance-orphan-detection-kernel`: 1
- `oya-governance-pack-overlay-completeness`: 1
- `oya-governance-portfolio-citation-kernel`: 1
- `oya-governance-pr-merge-gate-kernel`: 1
- `oya-governance-pr-traceability-kernel`: 1
- `oya-governance-pre-push-kernel`: 1
- `oya-governance-predictable-naming-kernel`: 1
- `oya-governance-provider-coupling-kernel`: 1
- `oya-governance-purpose-kernel`: 1
- `oya-governance-quality-lane-kernel`: 1
- `oya-governance-substance-bar`: 1
- `oya-governance-sunset-lifecycle-kernel`: 1
- `oya-governance-supply-chain-kernel`: 1
- `oya-governance-tos-policy-kernel`: 1
- `oya-governance-upstream-api-drift-kernel`: 1
- `oya-http-latency-budget-middleware-infrastructure`: 1
- `oya-http-middleware-kernel`: 1
- `oya-http-router-kernel`: 1
- `oya-http-runtime-hyper-adapter`: 1
- `oya-http-sse-kernel`: 1
- `oya-http-telemetry-middleware-infrastructure`: 1
- `oya-http-tenant-middleware-infrastructure`: 1
- `oya-http-wide-event-middleware-infrastructure`: 1
- `oya-json-kernel`: 1
- `oya-shared-architecture-check-cli`: 1
- `oya-shared-audit-chain-client-kernel`: 1
- `oya-shared-audit-digest-adapter-awslc`: 1
- `oya-shared-audit-event-kernel`: 1
- `oya-shared-backbone-grpc-generated-adapter`: 1
- `oya-shared-backbone-grpc-transport-adapter`: 1
- `oya-shared-backbone-proto-contracts-kernel`: 1
- `oya-shared-backbone-rest-runtime-adapter`: 1
- `oya-shared-backup-kernel`: 1
- `oya-shared-bounded-contexts-check-cli`: 1
- `oya-shared-compliance-evidence-kernel`: 1
- `oya-shared-connector-kernel`: 1
- `oya-shared-cursor-pagination-kernel`: 1
- `oya-shared-email-comms-kernel`: 1
- `oya-shared-hyperscaler-metrics-adapter-otlp`: 1
- `oya-shared-hyperscaler-metrics-adapter-prometheus`: 1
- `oya-shared-hyperscaler-metrics-kernel`: 1
- `oya-shared-i18n-kernel`: 1
- `oya-shared-idempotency-key-kernel`: 1
- `oya-shared-oidc-client-kernel`: 1
- `oya-shared-olap-clickhouse-adapter`: 1
- `oya-shared-olap-client-kernel`: 1
- `oya-shared-outbox-broker-http-adapter`: 1
- `oya-shared-outbox-pattern-kernel`: 1
- `oya-shared-pdp-kernel`: 1
- `oya-shared-platform-contracts-kernel`: 1
- `oya-shared-postgres-command-adapter-sqlx`: 1
- `oya-shared-postgres-command-kernel`: 1
- `oya-shared-presence-kernel`: 1
- `oya-shared-protocol-parity-kernel`: 1
- `oya-shared-protocol-transport-kernel`: 1
- `oya-shared-protocol-transport-retry-app`: 1
- `oya-shared-realtime-transport-kernel`: 1
- `oya-shared-resource-provider-contract-kernel`: 1
- `oya-shared-scim-server-kernel`: 1
- `oya-shared-semver-check-cli`: 1
- `oya-shared-supply-chain-check-cli`: 1
- `oya-shared-tenant-quota-kernel`: 1
- `oya-shared-timeseries-kernel`: 1
- `oya-shared-tracing-client-kernel`: 1
- `oya-shared-transactional-outbox-adapter-sqlx`: 1
- `oya-shared-transactional-outbox-dispatch-app`: 1
- `oya-shared-transactional-outbox-kernel`: 1
- `oya-shared-transactional-outbox-poller-app`: 1
- `oya-shared-transactional-outbox-runtime-tokio-app`: 1
- `oya-shared-transactional-outbox-worker-app`: 1
- `oya-shared-ulid-id-kernel`: 1
- `oya-shared-vector-store-kernel`: 1
- `oya-shared-wasm-runtime-kernel`: 1
- `oya-shared-webauthn-server-kernel`: 1
- `oya-shared-webhook-delivery-kernel`: 1
- `oya-workspace-members-kernel`: 1

## tools top-level buckets
- `fixup-ledger-merge-driver-app`: 1
- `oya-adapter-substitution-test-app`: 1
- `oya-architecture-graph-generator-app`: 1
- `oya-bot-autofix-app`: 1
- `oya-buck-test-wiring-app`: 1
- `oya-cargo-lock-merge-driver-app`: 1
- `oya-checkout-guard-app`: 1
- `oya-fabric-loop-state-app`: 1
- `oya-friction-ledger-merge-driver-app`: 1
- `oya-governance-adapter-with-no-importer-app`: 1
- `oya-governance-adr-shape-app`: 1
- `oya-governance-authoritative-tracked-app`: 1
- `oya-governance-banned-primitives-app`: 1
- `oya-governance-portfolio-citation-app`: 1
- `oya-governance-predictable-naming-app`: 1
- `oya-governance-purpose-audit-app`: 1
- `oya-governance-sunset-lifecycle-app`: 1
- `oya-lane-supervisor-app`: 1
- `oya-reorg-codemod-app`: 1
- `oya-tooling-agent-read`: 1
- `oya-xtask-metadata-augment-app`: 1

## oya top-level buckets (product tail / legacy services)
- `intelligence`: 78
- `office`: 19
- `community`: 14
- `application`: 8
- `itsm`: 6
- `ci-webhook-gateway`: 5
- `hr`: 5
- `payroll`: 5
- `ci-controller`: 4
- `ci-tide`: 3
- `crm`: 3
- `plant-maintenance`: 2
- `production-planning`: 2
- `quality-management`: 2
- `real-estate`: 2
- `supply-chain-planning`: 2
- `treasury`: 2
- `warehouse`: 2
- `contract-lifecycle-management`: 1
- `design-collaboration`: 1
- `docs`: 1
- `financial-planning`: 1
- `global-trade`: 1
- `incident-management`: 1
- `learning-management`: 1
- `marketing-automation`: 1
- `notes`: 1
- `performance-management`: 1
- `sheets`: 1
- `sites`: 1
- `slides`: 1
- `translate`: 1
- `whiteboard`: 1
- `workplace-integration`: 1

## Existing plans
- `specs/reorg/ci-move-plan.json`
- `specs/reorg/governance-check-move-plan.json`
- `specs/reorg/iam-pdp-cedar-move-plan.json`
- `specs/reorg/intelligence-move-plan.json`
- `specs/reorg/intelligence-sinkbatch-move-plan.json`
- `specs/reorg/intelligence-supervisor-move-plan.json`
- `specs/reorg/kernel-move-plan.BLOCKED.json`
- `specs/reorg/messaging-boundary-kernels-move-plan.json`
- `specs/reorg/messaging-substrate-kernel-move-plan.json`
- `specs/reorg/os-move-plan.json`

## Sequencing constraints (binding)
- Materializer remains `BLOCKED_NO_EXECUTABLE_MOVE_PLAN` until a non-BLOCKED kernel plan exists and is admitted.
- G024 intelligence remainder plan is plan-only and not admitted; no code move.
- G023 cloud-kernel deletion is behind W0 / #1523 / promoted corpus repair (#1526).
- One isolated writer at a time for any move execution; plan authoring may proceed non-conflicting.
- Catalog ArtifactMove rows must co-move; debrand targets forbid leading oya-/cloud- brands.

## Next executable moves (not activated)
1. G025: disposition matrix for every libs crate — KEEP_IN_LIBS | MOVE_TO_CAPABILITY | DELETE | REWRITE, with destination face grammar and catalog row.
2. G026: tools disposition — productize into capability facades vs retire vs keep as build-only under tools/ with explicit registry.
3. oya product-tail: per-service strangler batches only after capability destination exists; no dark dual authority.

## Explicit non-claims
- No move plan JSON authored in this census package.
- No code moved, no PR opened, no admission.
