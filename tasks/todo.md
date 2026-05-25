# Messenger/Mail/Community/Social Task List

Status legend: ⬜ pending · 🟦 in-progress · ✅ done

- ✅ CS-BACKBONE-MSG-001 — Messenger governance invariants and tests.
- ✅ CS-BACKBONE-MAIL-001 — Mail governance invariants and tests.
- ✅ CS-BACKBONE-SOCIAL-001 — Connect Social domain scaffold, catalog, manifest binding.
- ✅ CS-BACKBONE-COMMUNITY-001 — Community post-store domain scaffold and catalog.
- ✅ CS-BACKBONE-API-USECASE-001 — Protocol-neutral API + usecase crates for messenger, mail, social, and community.
- ✅ CS-BACKBONE-REST-001 — Framework-free REST/OpenAPI boundary crates for messenger, mail, social, and community write paths.
- ✅ CS-BACKBONE-POLICY-001 — Cedar policy-decision seams and negative cross-context API/REST tests.
- ✅ CS-BACKBONE-ROUTE-CATALOG-001 — OpenAPI operation route catalogs for messenger, mail, social, and community REST crates.
- ✅ CS-BACKBONE-PERSISTENCE-001 — Postgres/Citus migration bundles with tenant/cell/shard keys for messenger, mail, social, and community write stores.
- ✅ CS-BACKBONE-REPOSITORY-COMMANDS-001 — Tenant-scoped repository SQL command seams for messenger, mail, social, and community Postgres adapters.
- ✅ CS-BACKBONE-MANIFEST-CONSISTENCY-001 — Manifest layer lists now reflect implemented API/usecase crates for messenger, mail, and social.
- ✅ CS-BACKBONE-TELEMETRY-BINDINGS-001 — Canonical request telemetry metric-name bindings for implemented write routes.
- ✅ CS-BACKBONE-REST-DISPATCH-001 — Fail-closed 501 dispatch for OpenAPI-declared contract-only REST routes.
- ✅ CS-BACKBONE-REST-WRITE-DISPATCH-001 — Framework-free method/path dispatch now calls implemented REST business handlers instead of requiring out-of-band typed entrypoints.
- ✅ CS-BACKBONE-PROTOCOL-PARITY-001 — AsyncAPI/proto parity bindings for implemented write receipts.
- ✅ CS-BACKBONE-CI-MATRIX-001 — GitHub Actions/Jenkins package matrices for implemented backbone crates.
- ✅ CS-BACKBONE-APP-ORCHESTRATION-001 — Runtime-neutral app write plans composing usecase, persistence, and protocol event seams.
- ✅ CS-BACKBONE-REST-APP-PLAN-001 — REST write-plan entrypoints exposing app orchestration outputs for implemented write routes.
- ✅ CS-BACKBONE-TRANSPORT-PLAN-001 — Runtime-neutral broker/gRPC transport plans for implemented write routes.
- ✅ CS-BACKBONE-PROTOCOL-PAYLOAD-CODEC-001 — Deterministic proto-json payload codec seam for protocol transport planning.
- ✅ CS-BACKBONE-PROTO-CONTRACTS-001 — Source-controlled proto3 contracts and validation registry for backbone write RPCs.
- ✅ CS-BACKBONE-GRPC-GENERATED-001 — Compile-time tonic/prost generated bindings and prost binary round-trip tests for backbone write RPCs.
- ✅ CS-BACKBONE-GRPC-PLAN-ADAPTER-001 — Generated gRPC request adapters now bridge tonic/prost messages into existing write-plan boundaries without sockets.
- ✅ CS-BACKBONE-GRPC-TRANSPORT-ADAPTER-001 — Live tonic transport server/client socket seam exercises generated write RPCs over local TCP without SQL/broker execution.
- ✅ CS-BACKBONE-TRANSPORT-ACK-001 — Recording broker/gRPC acknowledgement contract for planned write transport bundles.
- ✅ CS-BACKBONE-TRANSPORT-RETRY-APP-001 — Bounded retry/dead-letter decision seam for broker/gRPC transport execution.
- ✅ CS-BACKBONE-BROKER-HTTP-PUBLISHER-001 — Live HTTP broker-gateway publisher adapter posts outbox metadata over local TCP and captures broker acks without DB/gRPC execution.
- ✅ CS-BACKBONE-HTTP-OUTBOX-EXECUTOR-001 — HTTP broker publisher implements the outbox transport executor seam for worker injection with explicit no-gRPC-execution ack refs.
- ✅ CS-BACKBONE-SQL-EXECUTION-CONTRACT-001 — Postgres pool and SQL execution contracts attached to app write plans.
- ✅ CS-BACKBONE-SQLX-EXECUTOR-001 — SQLx-backed Postgres command executor adapter seam for tenant-scoped write plans.
- ✅ CS-BACKBONE-LIVE-POSTGRES-RLS-HARNESS-001 — Env-gated SQLx live Postgres RLS probe harness with optional Citus distribution check, skipped unless explicitly enabled.
- ✅ CS-BACKBONE-TRANSACTIONAL-OUTBOX-001 — Transactional outbox command seam appended to backbone write plans.
- ✅ CS-BACKBONE-OUTBOX-SQLX-DRAIN-001 — SQLx transactional outbox drain adapter seam using queue-safe row claims.
- ✅ CS-BACKBONE-OUTBOX-DISPATCH-APP-001 — Runtime-neutral outbox dispatch app over claimed protocol outbox rows.
- ✅ CS-BACKBONE-OUTBOX-WORKER-APP-001 — SQLx outbox worker-cycle seam composing claim, dispatch, and SQL state mutation.
- ✅ CS-BACKBONE-OUTBOX-POLLER-APP-001 — Bounded outbox poller scheduler-policy seam with idle/error stop rules.
- ✅ CS-BACKBONE-OUTBOX-TOKIO-RUNTIME-001 — Tokio runtime sleeper seam for bounded outbox poller loops.
- ✅ CS-BACKBONE-OUTBOX-SERVICE-LOOP-001 — Bounded outbox service-loop lifecycle seam for future daemon composition.
- ✅ CS-BACKBONE-OUTBOX-SUPERVISOR-LIFECYCLE-001 — Supervised outbox service lifecycle envelope records start/ready/shutdown/stopped events around the bounded Tokio service loop.
- ✅ CS-BACKBONE-CEDAR-RUNTIME-001 — Cedar runtime evaluator and backbone write-policy conformance pack.
- ✅ CS-BACKBONE-RUNTIME-OBSERVABILITY-001 — Runtime metrics exercise, SLO burn model, and Prometheus adapter coverage.
- ✅ CS-BACKBONE-OTLP-METRICS-HARNESS-001 — Env-gated OpenTelemetry OTLP/HTTP metrics exporter harness for shared hyperscaler metrics, disabled unless explicitly enabled.
- ✅ CS-BACKBONE-REST-PROBE-READINESS-001 — Framework-free `/health` and `/ready` probe dispatch now exists for messenger, mail, social, and community REST boundaries; Kubernetes readiness/liveness probes point at those declared routes.
- ✅ CS-BACKBONE-ARGOCD-PROMOTION-001 — ArgoCD ApplicationSet promotion manifest for backbone services.
- ✅ CS-BACKBONE-EDGE-TLS-HARDENING-001 — Static messenger/community edge WAF, ECH, and PQC certificate manifests align them with existing mail/social edge hardening posture.
- ✅ CS-BACKBONE-GOVERNANCE-VALIDATOR-UNBLOCK-001 — Global catalog coverage and kernel data-class annotation cleanup now unblock local catalog/data-class validators; remote GitHub Actions failures are identified as Actions-budget-prevented jobs, not runner test failures; full local `oya verify --ci-required` remains non-green on recorded repo-global/local mirror lanes.
- ✅ CS-BACKBONE-ARCHITECTURE-BOUNDARY-ALIGNMENT-001 — REST/gRPC/outbox/API DTO catalog-role alignment and architecture-boundaries role-matrix coverage now make the local architecture-boundaries lane pass for the current workspace/catalog snapshot.
- ✅ CS-BACKBONE-VERIFY-RECURSION-FIXTURE-001 — The oya verify CI mirror integration fixture now clears inherited parent recursion guards, making local D-4 nextest pass under `OYA_VERIFY_RUNNING=1` without weakening the production recursion guard.
- ✅ CS-BACKBONE-GATEWAY-HTTPROUTE-001 — Messenger, mail, social, and community Helm charts now carry disabled-by-default Gateway API HTTPRoute templates plus OpenAPI/TLS/ECH/PQC references and CI path/static-smoke coverage; this is source-controlled chart readiness only, not a live Gateway/DNS/TLS rollout.
- ✅ CS-BACKBONE-REST-HYPER-RUNTIME-001 — A shared REST runtime adapter now binds messenger, mail, social, and community OpenAPI route catalogs to the canonical Hyper loopback seam for probes and contract-only responses, while explicitly not claiming production Gateway/TLS rollout or generic JSON write-body execution.
- ✅ CS-BACKBONE-REST-JSON-WRITE-BINDING-001 — Bind stateless messenger/mail/social/community REST write routes from Hyper-collected JSON bodies into the existing service-owned typed REST dispatchers; stateful community vote/moderation remains an honest 501 seam.
- ✅ CS-BACKBONE-CLOUD-TENANT-WORKLOAD-LABELS-001 — Add source-controlled Oyatie Cloud dogfood tenant/cost/workload/regulatory labels to messenger, mail, social, and community Kubernetes Helm Deployment templates plus CI static smoke checks; this is static chart contract evidence only, not a live cluster/OpenCost/ArgoCD deployment claim.
- ✅ CS-BACKBONE-RENDERED-TENANT-COST-LABEL-COVERAGE-001 — Add static rendered Deployment manifest snapshots for messenger, mail, social, and community so the ADR-0199 tenant-cost-label advisory gate scans FD-001 dogfood workloads with zero findings; this remains static rendered evidence only, not a cluster apply, admission, OpenCost, or ArgoCD sync claim.
- ✅ CS-BACKBONE-ARGOCD-FD001-TENANT-METADATA-001 — Align the static ArgoCD ApplicationSet with the FD-001 dogfood tenant/cost/workload/regulatory label block and Helm `workloadLabels.*` parameters, with CI semantic smoke coverage; this remains GitOps desired-state intent only, not an ArgoCD sync/health, namespace creation, OpenCost, or production deployment claim.

## Remaining toward full requested microservices

- ⬜ Broader contract-only REST/OpenAPI endpoint implementation, vendor broker integration, live production gateway/TLS rollout evidence, full live outbox poller-to-publisher deployment, stateful community vote/moderation HTTP binding, and acknowledgements beyond framework-free implemented write-route dispatch, shared REST Hyper loopback/runtime catalog binding, stateless JSON write binding over local Hyper, static Oyatie Cloud dogfood tenant workload labels plus rendered tenant-cost-label snapshots and static ArgoCD FD-001 tenant metadata, static messenger/community edge WAF/ECH/PQC manifests, disabled-by-default Gateway API HTTPRoute chart templates, transport planning metadata, deterministic payload codec, source-controlled proto3 contracts, generated write-plan conversion, local tonic loopback server/client seams, local HTTP broker-gateway publish/executor seams, retry decisions, transactional outbox command/drain/dispatch/worker-cycle/poller-policy/Tokio-sleeper/service-loop/supervisor-lifecycle seams, and recording acknowledgement contracts.
- ⬜ Live Postgres/Citus RLS integration runs, backup/restore drills, and Citus rebalance evidence beyond generated write batches, execution contracts, the compile-checked SQLx executor adapter seam, and the env-gated live RLS/Citus harness.
- ⬜ Live Cedar PDP deployment and service-gateway enforcement evidence beyond the in-process evaluator/conformance pack.
- ⬜ Live OpenTelemetry collector deployment, production SLO burn alert firing evidence, and production backpressure/circuit-breaker drills beyond the in-process runtime metrics exercise, Prometheus adapter tests, and env-gated OTLP/HTTP exporter harness.
- ⬜ Live ArgoCD sync/health evidence, branch-protection live evidence, and full CI runtime-run evidence beyond static matrix definitions, static ApplicationSet manifests with FD-001 tenant metadata, framework-free probe route handlers, and Helm probe path alignment. Remote GitHub Actions jobs for PR #179 remain budget-prevented, so no remote green CI claim is made.
- ✅ Oya VCS verify/done/promote lifecycle closeout recorded for expanded ChangeBundles promoted through `cb-backbone-microservices-governance-validator-unblock-20260524` and PR closeout evidence.
- ✅ Pull request against `dev` opened from isolated worktree branch `agent/backbone-microservices-20260523T081210Z`: https://github.com/jason931225/oyatie/pull/179.
