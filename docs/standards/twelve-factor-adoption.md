---
doc_class: Standard
title: Twelve-Factor Adoption Standard
status: Accepted
date: 2026-05-20
owner: council-architecture + ops-sre-reliability
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0211
  - ADR-0250
  - ADR-0254
enforced_by:
  - governance-twelve-factor-adoption
  - governance-deployment-model-spectrum
  - governance-provider-agnostic
canonical_paths:
  - specs/deployment-ops-contract.json
  - docs/standards/gitops-iac-cluster-tier-boundaries.md
  - docs/standards/container-image-convention.md
  - microservices/*/manifest.json
external_reference:
  - https://www.12factor.net/
---

# Twelve-Factor Adoption Standard

Oyatie adopts the Twelve-Factor App methodology as a baseline for SaaS and
cell-deployed services, then tightens it for tenant isolation, sovereign cells,
agentic promotion, Cedar policy, audit-chain evidence, and build-ahead
certification. The official Twelve-Factor reference remains the external
precedent; this standard defines the Oyatie-specific minimum.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to every deployable microservice.

It applies to every `app`, `rest`, `grpc`, `worker`, and `cli` crate
that runs in an environment.

It applies to `microservices/<ms>/manifest.json`.

It applies to Helm values, Kustomize overlays, OpenTofu modules, and Kubernetes
manifests under `microservices/<ms>/iac/`.

It applies to local development when local behavior is used as promotion evidence.

It applies to on-prem cells, BYOC cells, air-gapped cells, and Oyatie-hosted cells.

It does not require the original Twelve-Factor wording to be copied into every
microservice.

It does not override stronger compliance-pack rules.

It does not permit runtime configuration through undocumented flags.

## Normative Requirements

F-001. Codebase: every deployable service MUST have one canonical repository path.

F-002. Codebase: generated artifacts MUST identify their source generator.

F-003. Codebase: a deployable service MUST be traceable from microservice manifest to crates to contracts.

F-004. Codebase: service identity MUST be declared in `microservices/<ms>/manifest.json`.

F-005. Dependencies: Rust dependencies MUST be declared in Cargo manifests.

F-006. Dependencies: system packages MUST be declared in container or bundle manifests.

F-007. Dependencies: no deployable service MAY assume a globally installed binary.

F-008. Dependencies: provider SDKs MUST live behind adapter layers.

F-009. Dependencies: license posture MUST pass `cargo deny check`.

F-010. Config: runtime config MUST be externalized.

F-011. Config: secrets MUST come from OpenBao, KMS, or explicit local-dev fixtures.

F-012. Config: secrets MUST NOT be committed.

F-013. Config: tenant-specific config MUST be scoped by tenant id and pack.

F-014. Config: feature flags MUST use the feature-flag substrate, not ad hoc env vars.

F-015. Backing services: Postgres, NATS, object storage, OpenBao, and telemetry sinks MUST be attached resources.

F-016. Backing services: adapters MUST hide provider-specific APIs from inward layers.

F-017. Backing services: endpoint changes MUST not require recompilation.

F-018. Build/release/run: build artifacts MUST be immutable.

F-019. Build/release/run: release metadata MUST identify source commit and config bundle.

F-020. Build/release/run: runtime mutation MUST be audit-emitted.

F-021. Processes: services MUST be stateless except through declared backing services.

F-022. Processes: local disk MAY be cache only.

F-023. Processes: caches MUST be rebuildable.

F-024. Port binding: HTTP and gRPC processes MUST bind to configured ports.

F-025. Port binding: services MUST NOT rely on ambient reverse proxy magic.

F-026. Port binding: health, readiness, and metrics endpoints MUST be explicit.

F-027. Concurrency: scaling MUST be by process replica, shard, partition, or queue consumer count.

F-028. Concurrency: concurrency limits MUST be configuration.

F-029. Concurrency: worker concurrency MUST declare idempotency.

F-030. Disposability: services MUST start fast enough for rollout budgets.

F-031. Disposability: services MUST handle SIGTERM.

F-032. Disposability: shutdown MUST drain in-flight requests within the declared budget.

F-033. Dev/prod parity: local fixtures MUST model tenant, principal, pack, and data-class fields.

F-034. Dev/prod parity: local behavior MUST not bypass Cedar unless the bypass is explicit.

F-035. Dev/prod parity: staging MUST use production-like contract validators.

F-036. Logs: logs MUST be event streams.

F-037. Logs: logs MUST include request id, tenant id hash, principal class, and data class when available.

F-038. Logs: logs MUST NOT include secrets or regulated plaintext.

F-039. Admin processes: one-off tasks MUST run through `cli` or controlled job surfaces.

F-040. Admin processes: destructive admin tasks MUST require Cedar and audit-chain evidence.

F-041. Tenant boundary: every factor MUST preserve tenant isolation.

F-042. Sovereign cells: every factor MUST declare pack-specific deltas.

F-043. On-prem: every factor MUST be satisfiable without public internet during runtime.

F-044. BYOC: every factor MUST declare cloud-account boundary assumptions.

F-045. Air gap: every factor MUST declare artifact-bundle provenance.

F-046. Observability: every deployable service MUST export traces, metrics, logs, and audit events.

F-047. Reliability: every deployable service MUST declare SLOs before promotion past dev.

F-048. Rollback: every release MUST have a rollback path.

F-049. Cost: every service MUST emit cost allocation dimensions.

F-050. Compliance: every regulated service MUST map factor controls to packs.

F-051. Config schema MUST be versioned.

F-052. Config schema MUST reject unknown required fields.

F-053. Config schema MUST document defaults.

F-054. Config schema MUST separate safe defaults from tenant-specific values.

F-055. Config schema MUST declare which values are secret.

F-056. Config schema MUST declare which values are policy-controlled.

F-057. Config schema MUST declare which values require rollout.

F-058. Config schema MUST declare which values hot-reload.

F-059. Config schema MUST declare which values affect persistence.

F-060. Config schema MUST declare rollback behavior.

## Worked Examples

### Example 1: REST service config

```yaml
service: workflow-engine
process: rest
port: 8080
config_schema: microservices/workflow-engine/config/rest.v1.schema.json
secrets:
  openbao_paths:
    - secret/data/workflow-engine/prod/postgres
cedar_policy: microservices/workflow-engine/policy/tenant-scope.cedar
openslo:
  - microservices/workflow-engine/slos/availability.openslo.yaml
```

This passes because config, secrets, policy, and SLOs are externalized.

### Example 2: Invalid ambient dependency

```text
The worker assumes `protoc` exists on the host.
```

This fails because dependencies must be declared in the build image or toolchain
manifest.

### Example 3: On-prem artifact bundle

```yaml
bundle: workflow-engine-1.4.0.oab
includes:
  images:
    - workflow-engine-rest@sha256:...
  helm:
    - microservices/workflow-engine/iac/helm
  policies:
    - microservices/workflow-engine/policy/tenant-scope.cedar
  contracts:
    - microservices/workflow-engine/contracts/state-machine-v1.openapi.yaml
```

This passes because the runtime does not require internet access.

### Example 4: Worker disposability

```yaml
termination_grace_seconds: 30
drain_strategy: finish-current-message
idempotency_key: workflow_id + step_id + attempt
dead_letter_topic: workflow.step.deadletter.v1
```

The worker can die and restart without duplicating state mutation.

### Example 5: Logs as streams

```json
{
  "event": "workflow.step.completed",
  "request_id": "req_01J...",
  "tenant_id_hash": "sha256:...",
  "data_class": "BEHAVIORAL_TENANT_PRODUCT",
  "audit_event": "EVT-WORKFLOW-STEP-COMPLETED-V1"
}
```

The log is structured, streamable, and redacted.

## Verification

Primary command:

```bash
oya gate validate twelve-factor-adoption --scope microservices
```

The checker MUST read each `microservices/<ms>/manifest.json`.

The checker MUST read each deployable crate's metadata.

The checker MUST read Helm values and Kustomize overlays.

The checker MUST read OpenTofu variables for BYOC and on-prem surfaces.

The checker MUST detect ambient binary assumptions.

The checker MUST detect committed secrets.

The checker MUST detect missing config schemas.

The checker MUST detect missing OpenBao path declarations.

The checker MUST detect missing SLO manifests.

The checker MUST detect missing shutdown budgets.

The checker MUST detect missing telemetry attributes.

The checker MUST detect local-only behavior used as production evidence.

The checker MUST detect pack-specific configuration without pack labels.

The checker MUST detect runtime mutation without audit events.

The checker MUST emit per-factor pass/fail evidence.

The checker SHOULD emit a service-by-service matrix.

The checker SHOULD distinguish advisory legacy gaps from blocker new gaps.

## Common Anti-Patterns

Relying on a developer laptop binary is an anti-pattern.

Putting secrets in environment examples without redaction is an anti-pattern.

Using `.env` as a production control plane is an anti-pattern.

Treating local disk as durable storage is an anti-pattern.

Skipping SIGTERM handling is an anti-pattern.

Using unstructured text logs is an anti-pattern.

Running admin migrations through shell snippets is an anti-pattern.

Hard-coding cloud region names in code is an anti-pattern.

Hard-coding tenant ids in fixtures is an anti-pattern.

Treating on-prem deployment as a separate product is an anti-pattern.

Treating BYOC deployment as a fork is an anti-pattern.

Treating air-gap bundle creation as a manual zip is an anti-pattern.

Treating worker retries as best-effort is an anti-pattern.

Treating feature flags as environment variables is an anti-pattern.

Treating policy bypass as a dev convenience is an anti-pattern.

## Cross-References

External precedent: `https://www.12factor.net/`.

`docs/decisions/ADR-0709-general-live-apex.md` binds provider-agnostic posture.

`docs/decisions/ADR-0709-general-live-apex.md` binds self-owned substrate posture.

`docs/decisions/ADR-0709-general-live-apex.md` binds hosted, BYOC, on-prem, and air-gap deployments.

`docs/standards/gitops-iac-cluster-tier-boundaries.md` binds cluster lifecycle.

`docs/standards/container-image-convention.md` binds image shape.

`docs/standards/graceful-shutdown-canonical.md` binds process disposability.

`docs/standards/observability-slo.md` binds SLO manifests.

`docs/standards/openslo-authoring.md` binds OpenSLO details.

`docs/standards/cedar-policy-authoring.md` binds policy-as-config posture.

`specs/deployment-ops-contract.json` binds operational entrypoints.

## Substance Bar Compliance Checklist

TFA-SB-001. Verify one codebase path per deployable service.

TFA-SB-002. Verify service manifest declares deployable processes.

TFA-SB-003. Verify Cargo dependencies are explicit.

TFA-SB-004. Verify container system dependencies are explicit.

TFA-SB-005. Verify no global host binary assumption.

TFA-SB-006. Verify runtime config has schema.

TFA-SB-007. Verify secrets come from OpenBao or declared local fixtures.

TFA-SB-008. Verify no secret is committed.

TFA-SB-009. Verify backing services are attached resources.

TFA-SB-010. Verify Postgres URL is config, not code.

TFA-SB-011. Verify NATS URL is config, not code.

TFA-SB-012. Verify object storage endpoint is config, not code.

TFA-SB-013. Verify build artifacts are immutable.

TFA-SB-014. Verify release metadata names source commit.

TFA-SB-015. Verify runtime process is stateless.

TFA-SB-016. Verify local disk is cache only.

TFA-SB-017. Verify port binding is explicit.

TFA-SB-018. Verify readiness endpoint is explicit.

TFA-SB-019. Verify liveness endpoint is explicit.

TFA-SB-020. Verify metrics endpoint is explicit.

TFA-SB-021. Verify concurrency limits are config.

TFA-SB-022. Verify worker concurrency declares idempotency.

TFA-SB-023. Verify SIGTERM handling exists.

TFA-SB-024. Verify shutdown budget is declared.

TFA-SB-025. Verify dev fixtures include tenant id.

TFA-SB-026. Verify dev fixtures include principal id.

TFA-SB-027. Verify dev fixtures include data class.

TFA-SB-028. Verify dev fixtures include pack.

TFA-SB-029. Verify structured logs.

TFA-SB-030. Verify logs exclude secrets.

TFA-SB-031. Verify logs exclude regulated plaintext.

TFA-SB-032. Verify admin process is CLI or job.

TFA-SB-033. Verify destructive admin process uses Cedar.

TFA-SB-034. Verify destructive admin process emits audit.

TFA-SB-035. Verify hosted deployment has same config schema.

TFA-SB-036. Verify BYOC deployment has account boundary declaration.

TFA-SB-037. Verify on-prem deployment has offline runtime posture.

TFA-SB-038. Verify air-gap deployment has artifact bundle provenance.

TFA-SB-039. Verify OpenTofu variables map to config schema.

TFA-SB-040. Verify Helm values map to config schema.

TFA-SB-041. Check `workflow-engine` rest process.

TFA-SB-042. Check `workflow-engine` worker process.

TFA-SB-043. Check `ontology` projection process.

TFA-SB-044. Check `tenancy` lifecycle process.

TFA-SB-045. Check `policy-engine` Cedar process.

TFA-SB-046. Check `observability` telemetry process.

TFA-SB-047. Check `cloud-iac` controller process.

TFA-SB-048. Check `messenger` delivery process.

TFA-SB-049. Check `mail` delivery process.

TFA-SB-050. Check `api-gateway` edge process.

TFA-SB-051. Reject `.env` as production control plane.

TFA-SB-052. Reject hard-coded tenant id.

TFA-SB-053. Reject hard-coded cloud region.

TFA-SB-054. Reject hard-coded OpenBao token.

TFA-SB-055. Reject hidden migration shell snippet.

TFA-SB-056. Reject unstructured text logs.

TFA-SB-057. Reject feature flag as ad hoc env var.

TFA-SB-058. Reject stateful local filesystem writes.

TFA-SB-059. Reject runtime dependency on public internet for on-prem.

TFA-SB-060. Reject BYOC fork behavior.

TFA-SB-061. Emit factor pass count.

TFA-SB-062. Emit factor fail count.

TFA-SB-063. Emit service coverage count.

TFA-SB-064. Emit process coverage count.

TFA-SB-065. Emit config schema count.

TFA-SB-066. Emit secret binding count.

TFA-SB-067. Emit backing service count.

TFA-SB-068. Emit shutdown budget count.

TFA-SB-069. Emit admin process count.

TFA-SB-070. Emit deployment model count.

TFA-SB-071. Preserve Twelve-Factor codebase principle with microservice manifests.

TFA-SB-072. Preserve dependency principle with Cargo and image manifests.

TFA-SB-073. Preserve config principle with typed config schemas.

TFA-SB-074. Preserve backing-service principle with adapter boundaries.

TFA-SB-075. Preserve build-release-run principle with immutable bundles.

TFA-SB-076. Preserve process principle with stateless runtime.

TFA-SB-077. Preserve port-binding principle with explicit endpoints.

TFA-SB-078. Preserve concurrency principle with replicas and workers.

TFA-SB-079. Preserve disposability principle with graceful shutdown.

TFA-SB-080. Preserve logs principle with structured event streams.

## Extended Worked Example: Workflow Engine Twelve-Factor Manifest

The following manifest binds the classic Twelve-Factor ideas to Oyatie's
substrate rules. The important point is not that every factor is copied
verbatim, but that each factor is translated into a verifiable local contract.

```yaml
service: workflow-engine
microservice_path: microservices/workflow-engine
crate_roots:
  - crates/workflow-engine-kernel
  - crates/workflow-engine-domain
  - crates/workflow-engine-usecase
  - crates/workflow-engine-adapter-postgres
  - crates/workflow-engine-runtime
related_adrs:
  - docs/adr-archive/ADR-0105-13-layer-enum-and-check-family-patterns.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
factors:
  codebase:
    authority: one service ownership row in docs/SERVICE-CATALOG.md
    check: check-service-codebase-map
  dependencies:
    authority: Cargo.lock plus cargo-deny policy
    check: check-dependency-policy
  config:
    authority: RuntimeConfig::from_env plus OpenBao secret refs
    check: check-runtime-config
  backing_services:
    authority: explicit adapter crates and service manifest bindings
    check: check-backing-service-bindings
  build_release_run:
    authority: immutable image digest and release bundle id
    check: check-release-immutability
  processes:
    authority: stateless runtime process except declared durable stores
    check: check-process-statelessness
  port_binding:
    authority: bind address env var and service mesh registration
    check: check-port-binding
  concurrency:
    authority: worker pool and replica declarations
    check: check-concurrency-shape
  disposability:
    authority: graceful shutdown and drain deadlines
    check: check-disposability
  dev_prod_parity:
    authority: same container entrypoint across environments
    check: check-dev-prod-parity
  logs:
    authority: structured stdout events and trace ids
    check: check-structured-logs
  admin_processes:
    authority: one-shot admin jobs with audit events
    check: check-admin-processes
```

## Extended Worked Example: Runtime Configuration Schema

```rust
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub bind_addr: SocketAddr,
    pub public_base_url: Url,
    pub database_url_ref: SecretRef,
    pub cedar_bundle_ref: SecretRef,
    pub otel_endpoint: Url,
    pub max_worker_count: NonZeroUsize,
    pub shutdown_grace: Duration,
    pub environment: RuntimeEnvironment,
}

impl RuntimeConfig {
    pub fn from_env(env: &dyn EnvProvider) -> Result<Self, RuntimeConfigError> {
        Ok(Self {
            bind_addr: env.parse("OYATIE_WORKFLOW_ENGINE_BIND_ADDR")?,
            public_base_url: env.parse("OYATIE_WORKFLOW_ENGINE_PUBLIC_BASE_URL")?,
            database_url_ref: env.secret_ref("OYATIE_WORKFLOW_ENGINE_DATABASE_URL_REF")?,
            cedar_bundle_ref: env.secret_ref("OYATIE_WORKFLOW_ENGINE_CEDAR_BUNDLE_REF")?,
            otel_endpoint: env.parse("OYATIE_OTEL_EXPORTER_OTLP_ENDPOINT")?,
            max_worker_count: env.parse("OYATIE_WORKFLOW_ENGINE_MAX_WORKERS")?,
            shutdown_grace: env.parse_duration("OYATIE_WORKFLOW_ENGINE_SHUTDOWN_GRACE")?,
            environment: env.parse("OYATIE_ENVIRONMENT")?,
        })
    }
}
```

The config object MUST NOT contain deployment defaults that differ by
environment. Defaults live in deployment overlays and are rendered into
environment variables or OpenBao references before the process starts.

## Extended Factor Compliance Matrix

| ID | Factor | Oyatie requirement | Example path | Blocking checker |
|---|---|---|---|---|
| TFA-MAT-001 | Codebase | One codebase maps to one service catalog owner | `docs/SERVICE-CATALOG.md` | `check-service-codebase-map` |
| TFA-MAT-002 | Codebase | Shared logic lives in explicit library crates | `crates/common-time` | `check-shared-crate-ownership` |
| TFA-MAT-003 | Dependencies | Rust deps are pinned through workspace lockfile | `Cargo.lock` | `cargo-deny` |
| TFA-MAT-004 | Dependencies | OS deps are pinned through image digest | `iac/images/workflow-engine.lock` | `check-image-digest` |
| TFA-MAT-005 | Dependencies | Provider SDKs live only in adapter crates | `crates/mail-smtp-adapter-aws-ses` | `check-layer-enum` |
| TFA-MAT-006 | Config | Secrets are references, not literal values | `OYATIE_*_SECRET_REF` | `check-secret-literals` |
| TFA-MAT-007 | Config | Runtime config is loaded once at process start | `runtime/config.rs` | `check-runtime-config` |
| TFA-MAT-008 | Config | Tenant config is loaded through policy-governed ports | `tenancy/config-port.rs` | `check-tenant-config-port` |
| TFA-MAT-009 | Backing services | Databases are attached resources | `adapter-postgres` | `check-backing-service-bindings` |
| TFA-MAT-010 | Backing services | Brokers are attached resources | `adapter-nats` | `check-backing-service-bindings` |
| TFA-MAT-011 | Backing services | Cedar bundles are attached resources | `policy/*.cedar` | `check-cedar-policy-authoring` |
| TFA-MAT-012 | Build release run | Build artifacts are immutable | `.oab` bundle | `check-release-immutability` |
| TFA-MAT-013 | Build release run | Release config is bound by digest | `release.json` | `check-release-config-digest` |
| TFA-MAT-014 | Build release run | Runtime never compiles code | container entrypoint | `check-runtime-build-separation` |
| TFA-MAT-015 | Processes | Service process is stateless | `runtime/main.rs` | `check-process-statelessness` |
| TFA-MAT-016 | Processes | Durable state is only in declared stores | `manifest.json` | `check-durable-store-declarations` |
| TFA-MAT-017 | Port binding | Service binds an explicit address | `OYATIE_*_BIND_ADDR` | `check-port-binding` |
| TFA-MAT-018 | Port binding | Mesh registration is generated from manifest | `iac/helm/*` | `check-mesh-registration` |
| TFA-MAT-019 | Concurrency | Worker count is configurable | `OYATIE_*_MAX_WORKERS` | `check-concurrency-shape` |
| TFA-MAT-020 | Concurrency | Queue consumers are horizontally scalable | `consumer.rs` | `check-consumer-rebalance` |
| TFA-MAT-021 | Disposability | SIGTERM drains in declared grace window | `shutdown.rs` | `check-disposability` |
| TFA-MAT-022 | Disposability | In-flight workflows checkpoint before exit | `checkpoint.rs` | `check-workflow-checkpoint` |
| TFA-MAT-023 | Dev/prod parity | Same image entrypoint across envs | `Dockerfile` | `check-entrypoint-parity` |
| TFA-MAT-024 | Dev/prod parity | Local dependencies use same adapter contracts | `docker-compose.yaml` | `check-adapter-parity` |
| TFA-MAT-025 | Logs | Logs emit to stdout as structured events | `telemetry.rs` | `check-structured-logs` |
| TFA-MAT-026 | Logs | Trace ids appear on boundary logs | `tracing` fields | `check-trace-context` |
| TFA-MAT-027 | Admin processes | One-shot jobs are audited | `jobs/*.rs` | `check-admin-processes` |
| TFA-MAT-028 | Admin processes | Migrations run as release-bound jobs | `migrations/*.sql` | `check-migration-job-binding` |
| TFA-MAT-029 | Admin processes | Backfills have resumable checkpoints | `backfills/*.rs` | `check-backfill-checkpoint` |
| TFA-MAT-030 | All factors | Promotion evidence records checker output | `.omx/evidence/*` | `oya-vcs-admission` |

## Extended Review Questions

TFA-REV-001. Does the service manifest name exactly one owning µservice?

TFA-REV-002. Does each dependency have a declared owner and update cadence?

TFA-REV-003. Does every secret field use a secret reference?

TFA-REV-004. Does every backing service have an adapter boundary?

TFA-REV-005. Does the release artifact carry an immutable digest?

TFA-REV-006. Does the runtime avoid generating code or schemas on startup?

TFA-REV-007. Does the process treat local disk as ephemeral?

TFA-REV-008. Does the service bind its own port instead of relying on a sidecar?

TFA-REV-009. Does horizontal concurrency avoid shared in-memory locks?

TFA-REV-010. Does shutdown drain consumers before closing stores?

TFA-REV-011. Does local development use the same protocol contracts as production?

TFA-REV-012. Do logs remain useful without file-system access?

TFA-REV-013. Do admin jobs emit audit events?

TFA-REV-014. Do migrations run through release-bound workflow?

TFA-REV-015. Do backfills have an idempotency key?

TFA-REV-016. Does every env var have a documented parser and validation error?

TFA-REV-017. Does every OpenBao secret reference include a rotation owner?

TFA-REV-018. Does every queue consumer have a dead-letter route?

TFA-REV-019. Does every provider adapter support replacement without domain edits?

TFA-REV-020. Does every runtime flag appear in the deployment manifest?

TFA-REV-021. Does every production default appear in IaC, not Rust code?

TFA-REV-022. Does every readiness check prove dependency reachability?

TFA-REV-023. Does every liveness check avoid deep dependency calls?

TFA-REV-024. Does every metric include tenant-safe labels only?

TFA-REV-025. Does every release include rollback evidence?

TFA-REV-026. Does every environment use the same binary artifact?

TFA-REV-027. Does every admin command have least-privilege credentials?

TFA-REV-028. Does every scheduled job have single-flight protection?

TFA-REV-029. Does every batch process have progress telemetry?

TFA-REV-030. Does the promote evidence cite `check-twelve-factor-adoption`?

## Extended Environment Variable Register

The register below is an example of the minimum specificity expected from a
Twelve-Factor service. A service may add variables, but it MUST keep the same
shape: owner, parser, secret status, default authority, and verification rule.

| ID | Variable | Owner | Parser | Secret | Verification |
|---|---|---|---|---|---|
| TFA-ENV-001 | `OYATIE_ENVIRONMENT` | platform-runtime | enum | no | rejects unknown env names |
| TFA-ENV-002 | `OYATIE_REGION` | platform-runtime | region id | no | matches deployment cell |
| TFA-ENV-003 | `OYATIE_CELL_ID` | cell-runtime | cell id | no | matches cell registry |
| TFA-ENV-004 | `OYATIE_TENANT_PACKS` | tenancy | csv pack ids | no | matches pack registry |
| TFA-ENV-005 | `OYATIE_WORKFLOW_ENGINE_BIND_ADDR` | workflow-engine | socket addr | no | bind smoke test |
| TFA-ENV-006 | `OYATIE_WORKFLOW_ENGINE_PUBLIC_BASE_URL` | workflow-engine | url | no | OpenAPI server parity |
| TFA-ENV-007 | `OYATIE_WORKFLOW_ENGINE_DATABASE_URL_REF` | workflow-engine | secret ref | yes-ref | OpenBao path exists |
| TFA-ENV-008 | `OYATIE_WORKFLOW_ENGINE_CEDAR_BUNDLE_REF` | policy-engine | secret ref | yes-ref | bundle signature verifies |
| TFA-ENV-009 | `OYATIE_WORKFLOW_ENGINE_MAX_WORKERS` | workflow-engine | nonzero usize | no | capacity limit check |
| TFA-ENV-010 | `OYATIE_WORKFLOW_ENGINE_SHUTDOWN_GRACE` | workflow-engine | duration | no | drain test under limit |
| TFA-ENV-011 | `OYATIE_OTEL_EXPORTER_OTLP_ENDPOINT` | observability | url | no | collector reachable |
| TFA-ENV-012 | `OYATIE_TRACE_SAMPLING_TIER` | observability | tier id | no | sampling standard parity |
| TFA-ENV-013 | `OYATIE_AUDIT_CHAIN_ENDPOINT` | audit | url | no | audit write smoke |
| TFA-ENV-014 | `OYATIE_IDEMPOTENCY_STORE_REF` | platform-runtime | secret ref | yes-ref | store migration current |
| TFA-ENV-015 | `OYATIE_OUTBOX_BROKER_REF` | platform-runtime | secret ref | yes-ref | broker topic exists |
| TFA-ENV-016 | `OYATIE_MESH_SPIFFE_TRUST_DOMAIN` | mesh | dns label | no | SPIFFE issuer parity |
| TFA-ENV-017 | `OYATIE_CEDAR_SCHEMA_PATH` | policy-engine | path | no | schema hash pinned |
| TFA-ENV-018 | `OYATIE_RELEASE_BUNDLE_ID` | release | bundle id | no | release manifest match |
| TFA-ENV-019 | `OYATIE_IMAGE_DIGEST` | supply-chain | digest | no | image digest match |
| TFA-ENV-020 | `OYATIE_ADMIN_JOB_ID` | admin-process | job id | no | audit event carries id |

## Extended CI Evidence Contract

TFA-CI-001. `check-service-codebase-map` MUST emit the service id.

TFA-CI-002. `check-service-codebase-map` MUST emit the owning docs path.

TFA-CI-003. `check-dependency-policy` MUST emit denied dependencies.

TFA-CI-004. `check-runtime-config` MUST emit parsed variable count.

TFA-CI-005. `check-secret-literals` MUST emit literal-secret findings.

TFA-CI-006. `check-backing-service-bindings` MUST emit resource count.

TFA-CI-007. `check-release-immutability` MUST emit image digest.

TFA-CI-008. `check-process-statelessness` MUST emit durable-store list.

TFA-CI-009. `check-port-binding` MUST emit bind address contract.

TFA-CI-010. `check-concurrency-shape` MUST emit worker bounds.

TFA-CI-011. `check-disposability` MUST emit drain evidence.

TFA-CI-012. `check-entrypoint-parity` MUST emit env comparison.

TFA-CI-013. `check-structured-logs` MUST emit sample log event.

TFA-CI-014. `check-admin-processes` MUST emit audited job ids.

TFA-CI-015. `oya-vcs-admission` MUST include all checker names in promote evidence.

## Extended Anti-Pattern Catalogue

TFA-APX-001. A service reads `.env.production` at runtime; fix by rendering env before launch.

TFA-APX-002. A runtime defaults to localhost database in production; fix by requiring secret refs.

TFA-APX-003. A migration runs from `main()` on every boot; fix by release-bound admin job.

TFA-APX-004. A queue worker stores progress only in memory; fix by durable checkpoint.

TFA-APX-005. A service writes operational logs to a local file; fix by structured stdout.

TFA-APX-006. A service changes behavior based on hostname string parsing; fix by explicit env.

TFA-APX-007. A provider SDK appears in a domain crate; fix by adapter boundary.

TFA-APX-008. A feature flag lacks a retirement date; fix by adding rollout ownership.

TFA-APX-009. A readiness check calls a slow downstream API; fix by checking local dependency health.

TFA-APX-010. A one-off repair command runs without audit event; fix by admin-process wrapper.

## Extended Promotion Evidence Ledger

TFA-EVID-001. Record the service id scanned by the Twelve-Factor checker.

TFA-EVID-002. Record the immutable image digest used for the release.

TFA-EVID-003. Record the OpenBao secret-reference count.

TFA-EVID-004. Record the backing-service adapter count.

TFA-EVID-005. Record the admin-process audit-event count.

TFA-EVID-006. Record the structured-log sample hash.

TFA-EVID-007. Record the graceful-shutdown drain test result.

TFA-EVID-008. Record the dev/prod entrypoint parity result.

TFA-EVID-009. Record the runtime config parser result.

TFA-EVID-010. Record the VCS changeset id that carried the evidence.
