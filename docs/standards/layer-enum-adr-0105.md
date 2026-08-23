---
doc_class: Standard
title: ADR-0105 Layer Enum Standard
status: Accepted
date: 2026-05-20
owner: council-architecture + axis-foundry
related_oyatie_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0115
  - ADR-0131
enforced_by:
  - governance-layered-architecture
  - governance-layer-enum
  - governance-naming-convention
canonical_paths:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/standards/clean-architecture.md
  - docs/standards/crate-naming-convention.md
  - crates/dev-cli/src/layered_architecture_gates.rs
---

# ADR-0105 Layer Enum Standard

ADR-0105 introduced the canonical layer enum used by checker families and by
crate naming. This standard gives implementers the operational version: what
each layer means, what it may import, which artifact types may claim it, and how
CI proves that a declared layer is not just a filename suffix.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to every `oyatie-*` crate.

It applies to every `registry/catalog/*.yaml` layer declaration.

It applies to every microservice manifest under `microservices/*/manifest.json`.

It applies to every capability contribution that names an implementation crate.

It applies to every checker that classifies dependency direction.

It applies to every design doc that says a component is kernel, domain, usecase,
app, adapter, infrastructure, rest, grpc, worker, cli, or sdk.

It applies retroactively to legacy labels such as `application` when they remain
in compatibility rows.

It does not decide whether a microservice should exist.

It does not decide whether an API is REST, gRPC, event-driven, or realtime.

It does not authorize cross-layer imports for convenience.

It does not replace `clean-architecture.md`; it supplies the closed enum that
`clean-architecture.md` enforces.

## Canonical Enum

L-001. `kernel` is the innermost layer for pure types, identifiers, invariants, and port traits.

L-002. `domain` is the business-rules layer over kernel types and ports.

L-003. `usecase` is orchestration over domain operations without concrete adapters.

L-004. `app` is the composition root for usecases, adapters, and inbound surfaces.

L-005. `adapter` is an outbound provider implementation for one port.

L-006. `infrastructure` is reusable runtime substrate that is not a deployable app.

L-007. `rest` is an inbound HTTP REST surface.

L-008. `grpc` is an inbound or internal gRPC surface.

L-009. Reserved; removed from the active owned-stack layer vocabulary by ADR-0565.

L-010. `worker` is an inbound async, scheduled, stream, or queue processor.

L-011. `cli` is a terminal or automation command surface.

L-012. `sdk` is a generated or hand-authored consumer client surface.

L-013. `check` is the governance checker family layer used only by `check-*` crates.

The enum is closed.

Adding a new value requires an ADR.

Renaming a value requires an ADR.

Retiring a value requires an ADR and migration ledger.

Aliases require an expiration date.

New crates MUST NOT use aliases.

## Normative Requirements

R-001. A crate MUST declare exactly one layer.

R-002. The declared layer MUST match the terminal crate-name token.

R-003. The declared layer MUST match `registry/catalog/<crate>.yaml`.

R-004. The declared layer MUST match `microservices/<ms>/manifest.json` when listed.

R-005. `kernel` crates MUST NOT depend on workspace crates outside approved base kernels.

R-006. `kernel` crates MUST NOT perform network I/O.

R-007. `kernel` crates MUST NOT perform filesystem I/O.

R-008. `kernel` crates MUST NOT depend on async runtimes.

R-009. `kernel` crates MUST define value invariants in constructors or smart types.

R-010. `kernel` crates SHOULD define port traits when the port operates on kernel types.

R-011. `domain` crates MUST depend inward only.

R-012. `domain` crates MUST NOT depend on adapters.

R-013. `domain` crates MUST NOT depend on REST, gRPC, worker, CLI, or SDK layers.

R-014. `domain` crates SHOULD hold pure business workflows.

R-015. `domain` crates MUST translate invalid state into typed domain errors.

R-016. `usecase` crates MUST orchestrate a single actor-intent or system-intent flow.

R-017. `usecase` crates MUST depend on ports, not providers.

R-018. `usecase` crates MUST NOT import `sqlx`, `reqwest`, provider SDKs, or cloud clients.

R-019. `usecase` crates MUST emit audit-intent data through a port.

R-020. `usecase` crates SHOULD be the home for idempotency decisions.

R-021. `app` crates MAY wire concrete adapters.

R-022. `app` crates MUST be deployable only when paired with an inbound surface or runtime.

R-023. `app` crates MUST NOT be imported by another app crate.

R-024. `app` crates SHOULD contain minimal composition code.

R-025. `app` crates MUST not bury business rules that belong in domain or usecase.

R-026. `adapter` crates MUST implement exactly one provider or backend family unless an ADR permits a multiplexer.

R-027. `adapter` crates MUST translate provider errors before crossing inward boundaries.

R-028. `adapter` crates MUST NOT call other adapters directly.

R-029. `adapter` crates MUST expose narrow port implementations.

R-030. `adapter` crates MUST keep provider configuration at the edge.

R-031. `infrastructure` crates MUST describe why they are not product services.

R-032. `infrastructure` crates MUST NOT become dumping grounds for shared code.

R-033. `infrastructure` crates MUST expose stable APIs with SemVer policy.

R-034. `rest` crates MUST own OpenAPI request and response serialization.

R-035. `rest` crates MUST call usecase or app surfaces instead of duplicating business rules.

R-036. `rest` crates MUST validate idempotency, tenant, principal, data class, and request id.

R-037. `rest` crates MUST not call provider adapters directly unless the app layer explicitly delegates.

R-038. `rest` crates MUST bind to `openapi-3-2-authoring.md`.

R-039. `grpc` crates MUST own proto3 service bindings and generated glue.

R-040. `grpc` crates MUST bind to `proto3-authoring.md`.

R-041. Reserved; removed from the active owned-stack layer vocabulary by ADR-0565.

R-042. Reserved; removed from the active owned-stack layer vocabulary by ADR-0565.

R-043. `worker` crates MUST own queue, stream, timer, or batch entrypoints.

R-044. `worker` crates MUST declare retry and dead-letter semantics.

R-045. `worker` crates MUST declare idempotency keys.

R-046. `worker` crates MUST emit audit-chain events for state mutations.

R-047. `cli` crates MUST be operator or developer surfaces.

R-048. `cli` crates MUST not become hidden production runtimes.

R-049. `cli` crates MUST document destructive subcommands.

R-050. `sdk` crates MUST be consumer safe.

R-051. `sdk` crates MUST not import server-only app crates.

R-052. `sdk` crates MUST pin public contract versions.

R-053. `check` crates MUST be governance validators.

R-054. `check` crates MUST produce deterministic diagnostics.

R-055. `check` crates MUST include fixture coverage for each refused shape.

R-056. A layer change MUST be treated as an architectural change.

R-057. A layer change MUST update the catalog.

R-058. A layer change MUST update the dependency graph evidence.

R-059. A layer change MUST update standards cross-references if semantics move.

R-060. A layer change MUST run architecture gates before merge.

R-061. Every crate MUST be classifiable without reading prose docs.

R-062. Every layer declaration MUST be machine-readable.

R-063. Every layer declaration SHOULD be human-readable in the crate README.

R-064. Every checker MUST treat unknown layer values as BLOCKER.

R-065. Every checker MUST treat mixed layer claims as BLOCKER.

R-066. Every checker MUST treat missing layer claims as BLOCKER for new crates.

R-067. Legacy crates MAY carry advisory findings only when migration rows exist.

R-068. Legacy crates MUST NOT be copied as templates.

R-069. A generated crate MUST inherit the generator's target layer explicitly.

R-070. A generated SDK MUST never be classified as server `app`.

R-071. A generated REST server stub MUST be classified as `rest`, not `app`.

R-072. A generated gRPC binding MUST be classified as `grpc`.

R-073. A schema-only support crate SHOULD be `kernel` or `sdk` depending on audience.

R-074. A policy-only evaluator crate SHOULD be `domain` or `adapter` depending on provider coupling.

R-075. A database repository implementation MUST be `adapter`.

R-076. A Postgres migration crate MUST be `infrastructure` unless it exposes runtime behavior.

R-077. A Kubernetes operator controller MUST be `worker` or `app` with explicit controller rationale.

R-078. A one-shot migration binary MUST be `cli` unless it runs in production automation.

R-079. A recurring migration controller MUST be `worker`.

R-080. A public client package MUST be `sdk`.

## Worked Examples

### Example 1: Workflow domain crate

Valid crate:

```text
crates/workflow-engine-state-machine-domain
```

It may depend on:

```text
workflow-engine-state-machine-kernel
```

It may not depend on:

```text
workflow-engine-state-machine-rest
workflow-engine-state-machine-adapter-postgres
workflow-engine-state-machine-app
```

The domain crate owns state transition invariants.

The rest crate owns HTTP mapping.

The adapter crate owns persistence.

### Example 2: Tenancy adapter crate

Valid crate:

```text
crates/tenancy-sub-scope-registry-adapter-postgres
```

It implements a kernel or domain port.

It may import `sqlx`.

It must translate `sqlx::Error`.

It must not export `sqlx::Pool` through the port interface.

### Example 3: REST layer split

Valid files:

```text
crates/cloud-compute-vm-rest
contracts/cloud-compute-vm-v1.openapi.yaml
```

The REST crate owns transport-level validation.

The OpenAPI document owns contract shape.

The usecase owns provisioning intent.

The adapter owns cloud-provider calls.

### Example 4: Worker layer split

Valid worker:

```text
crates/workflow-engine-deadline-worker
```

Required declarations:

```yaml
retry_policy: exponential-jitter
dead_letter_topic: workflow.deadline.deadletter.v1
idempotency_key: workflow_id + deadline_id + epoch
audit_event: EVT-WORKFLOW-DEADLINE-FIRED-V1
```

The worker is inbound because a timer enters the system.

### Example 5: Checker layer split

Valid checker:

```text
crates/check-layered-architecture
```

It may read manifests.

It may read Cargo metadata.

It may not become an app runtime.

It must produce deterministic diagnostics.

## Verification

Primary command:

```bash
presubmit (retired CLI gate validate) layered-architecture --scope crates
```

Additional commands:

```bash
presubmit (retired CLI gate validate) naming-convention --scope crates
presubmit (retired CLI gate validate) flat-crates --scope crates
presubmit (retired CLI gate validate) catalog-id-discipline --scope registry
```

The layer checker MUST parse `cargo metadata --no-deps`.

The layer checker MUST parse `[package.metadata.oya]`.

The layer checker MUST parse registry catalog records.

The layer checker MUST compare crate-name suffixes against layer declarations.

The layer checker MUST build an import graph.

The layer checker MUST reject inward-layer imports from outward layers.

The layer checker MUST reject peer adapter imports.

The layer checker MUST reject app-to-app imports.

The layer checker MUST reject REST-to-worker imports.

The layer checker MUST reject SDK-to-server imports.

The layer checker MUST reject unknown layer tokens.

The layer checker MUST report the exact dependency edge.

The layer checker MUST report the owning ADR.

The layer checker MUST report the owning standard.

The layer checker SHOULD include a suggested split.

The layer checker SHOULD include a migration classification.

The layer checker MAY allow advisory legacy exceptions until the migration sunset.

## Common Anti-Patterns

Putting `sqlx` in a kernel crate is an anti-pattern.

Putting `reqwest` in a domain crate is an anti-pattern.

Putting provider SDKs in a usecase crate is an anti-pattern.

Putting business rules in a REST handler is an anti-pattern.

Putting retry policy in a domain entity is an anti-pattern.

Putting app composition in an SDK is an anti-pattern.

Putting background timers in an app crate without worker classification is an anti-pattern.

Putting generated server stubs in `kernel` is an anti-pattern.

Putting Cedar authorization bypasses in presentation handlers is an anti-pattern.

Putting OpenAPI request types in a domain crate is an anti-pattern.

Putting proto generated types in unrelated domain crates is an anti-pattern.

Putting audit event emission behind a provider adapter with no port is an anti-pattern.

Putting a checker in a product service crate is an anti-pattern.

Putting multiple layer meanings in one crate because it is small is an anti-pattern.

Using `infrastructure` as a miscellaneous bucket is an anti-pattern.

Using `app` as a synonym for `usecase` is an anti-pattern.

Using `application` for new crates is an anti-pattern after ADR-0106.

Using `service` as a crate layer is an anti-pattern unless the migration ledger still permits it.

Using a suffix that does not match `[package.metadata.oya].layer` is an anti-pattern.

Suppressing a layer finding without an ADR is an anti-pattern.

## Cross-References

`docs/decisions/ADR-0709-general-live-apex.md` is the binding decision.

`docs/decisions/ADR-0700-ci-admission-live-apex.md` binds crate-name grammar.

`docs/decisions/ADR-0703-cas-cache-live-apex.md` binds the transition away from `application`.

`docs/decisions/ADR-0701-monorepo-capability-live-apex.md` binds per-service layout.

`docs/standards/clean-architecture.md` gives dependency-direction semantics.

`docs/standards/crate-naming-convention.md` gives legacy crate grammar.

`docs/standards/naming-convention-bnf-v4.md` gives cross-artifact id grammar.

`docs/standards/openapi-3-2-authoring.md` gives REST contract authoring.

`docs/standards/proto3-authoring.md` gives gRPC contract authoring.

`registry/catalog/` records machine-readable crate ownership.

`crates/dev-cli/src/layered_architecture_gates.rs` is the local checker path.

## Substance Bar Compliance Checklist

LAY-SB-001. Verify every crate declares one layer.

LAY-SB-002. Verify every layer token is in the ADR-0105 enum.

LAY-SB-003. Verify `kernel` crates have no outward dependency.

LAY-SB-004. Verify `domain` crates depend inward only.

LAY-SB-005. Verify `usecase` crates avoid concrete providers.

LAY-SB-006. Verify `app` crates are not imported by app peers.

LAY-SB-007. Verify `adapter` crates implement provider ports.

LAY-SB-008. Verify `infrastructure` crates are not miscellaneous buckets.

LAY-SB-009. Verify `rest` crates own HTTP mapping only.

LAY-SB-010. Verify `grpc` crates own proto service mapping only.

LAY-SB-011. Reserved; removed from the active owned-stack layer vocabulary by ADR-0565.

LAY-SB-012. Verify `worker` crates own queue or timer entrypoints.

LAY-SB-013. Verify `cli` crates do not run hidden production loops.

LAY-SB-014. Verify `sdk` crates avoid server-only imports.

LAY-SB-015. Verify `check` crates are deterministic validators.

LAY-SB-016. Reject `kernel -> domain` imports.

LAY-SB-017. Reject `kernel -> adapter` imports.

LAY-SB-018. Reject `domain -> app` imports.

LAY-SB-019. Reject `domain -> rest` imports.

LAY-SB-020. Reject `usecase -> adapter` imports.

LAY-SB-021. Reject `app -> app` imports.

LAY-SB-022. Reject `adapter -> adapter` imports.

LAY-SB-023. Reject `rest -> worker` imports.

LAY-SB-024. Reject `worker -> rest` imports.

LAY-SB-025. Reject `sdk -> app` imports.

LAY-SB-026. Reject unknown layer declarations.

LAY-SB-027. Reject missing layer declarations on new crates.

LAY-SB-028. Reject crate suffix and metadata mismatch.

LAY-SB-029. Reject catalog layer and Cargo metadata mismatch.

LAY-SB-030. Reject manifest layer and catalog mismatch.

LAY-SB-031. Require ADR for layer enum addition.

LAY-SB-032. Require migration ledger for layer enum retirement.

LAY-SB-033. Require fixture for every refused dependency edge.

LAY-SB-034. Require diagnostic to name edge source.

LAY-SB-035. Require diagnostic to name edge target.

LAY-SB-036. Require diagnostic to name expected direction.

LAY-SB-037. Require diagnostic to cite this standard.

LAY-SB-038. Require diagnostic to cite ADR-0105.

LAY-SB-039. Require remediation hint for obvious split.

LAY-SB-040. Require advisory classification for tolerated legacy row.

LAY-SB-041. Check `workflow-engine-state-machine-domain`.

LAY-SB-042. Check `workflow-engine-state-machine-rest`.

LAY-SB-043. Check `workflow-engine-deadline-worker`.

LAY-SB-044. Check `tenancy-sub-scope-registry-kernel`.

LAY-SB-045. Check `tenancy-sub-scope-registry-adapter-postgres`.

LAY-SB-046. Check `policy-cedar-domain`.

LAY-SB-047. Check `policy-cedar-api`.

LAY-SB-048. Check `cloud-compute-vm-api`.

LAY-SB-049. Check `cloud-kms-domain`.

LAY-SB-050. Check `intelligence-evidence-domain`.

LAY-SB-051. Verify generated REST stubs classify as `rest`.

LAY-SB-052. Verify generated gRPC stubs classify as `grpc`.

LAY-SB-053. Verify generated clients classify as `sdk`.

LAY-SB-054. Verify one-shot admin tools classify as `cli`.

LAY-SB-055. Verify recurring queue processors classify as `worker`.

LAY-SB-056. Verify database repositories classify as `adapter`.

LAY-SB-057. Verify domain services classify as `domain`.

LAY-SB-058. Verify invariant types classify as `kernel`.

LAY-SB-059. Verify composition roots classify as `app`.

LAY-SB-060. Verify shared runtime substrate classifies as `infrastructure`.

LAY-SB-061. Emit crate count by layer.

LAY-SB-062. Emit forbidden edge count.

LAY-SB-063. Emit missing metadata count.

LAY-SB-064. Emit legacy exception count.

LAY-SB-065. Emit generated-crate count.

LAY-SB-066. Emit checker-crate count.

LAY-SB-067. Emit dependency graph hash.

LAY-SB-068. Emit catalog graph hash.

LAY-SB-069. Emit ADR coverage count.

LAY-SB-070. Emit service manifest coverage count.

LAY-SB-071. Preserve `application` only as migration language.

LAY-SB-072. Preserve `service` only in old ADR prose.

LAY-SB-073. Preserve aliases only with sunset.

LAY-SB-074. Preserve app imports only at runtime composition.

LAY-SB-075. Preserve adapter imports only behind ports.

LAY-SB-076. Preserve provider imports only in adapter or infrastructure.

LAY-SB-077. Preserve generated code only outside kernel.

LAY-SB-078. Preserve transport structs only outside domain.

LAY-SB-079. Preserve business invariants only inside kernel or domain.

LAY-SB-080. Preserve orchestration logic inside usecase or workflow.

## Extended Worked Example: Layer-Safe Workflow Cancellation Slice

The following example shows the same capability divided across layers. Each
path has one reason to exist and no path imports from a lower-authority or
higher-volatility layer in the wrong direction.

```text
crates/
  workflow-cancel-kernel/
    src/
      cancellation_state.rs
      cancellation_rule.rs
      cancellation_reason.rs
      mod.rs
  workflow-cancel-domain/
    src/
      aggregate.rs
      command.rs
      event.rs
      error.rs
      mod.rs
  workflow-cancel-usecase/
    src/
      cancel_workflow.rs
      ports.rs
      policy.rs
      mod.rs
  workflow-cancel-adapter-postgres/
    src/
      repository.rs
      outbox.rs
      mod.rs
  workflow-cancel-adapter-cedar/
    src/
      authorizer.rs
      schema.rs
      mod.rs
  workflow-cancel-runtime/
    src/
      main.rs
      config.rs
      http.rs
      grpc.rs
      telemetry.rs
```

```rust
// kernel: pure invariant, no async, no database, no transport.
pub enum CancellationState {
    Requested,
    Accepted,
    Compensating,
    Completed,
    Rejected,
}

pub fn may_cancel(current_state: &str) -> bool {
    matches!(current_state, "pending" | "running" | "waiting_for_timer")
}

// domain: business command and event vocabulary.
pub struct CancelWorkflowCommand {
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub actor_id: ActorId,
    pub reason: CancellationReason,
}

pub enum WorkflowCancellationEvent {
    Requested { workflow_id: WorkflowId, actor_id: ActorId },
    Rejected { workflow_id: WorkflowId, code: RejectionCode },
    CompensationScheduled { workflow_id: WorkflowId, step_count: u32 },
    Completed { workflow_id: WorkflowId },
}

// usecase: orchestration through ports.
pub trait WorkflowRepository {
    async fn load(&self, id: WorkflowId) -> Result<WorkflowAggregate, WorkflowError>;
    async fn save(&self, aggregate: WorkflowAggregate) -> Result<(), WorkflowError>;
}

pub trait WorkflowAuthorizer {
    async fn may_cancel(&self, actor: ActorId, workflow: WorkflowId) -> Result<bool, WorkflowError>;
}

// adapter: concrete Postgres details stay here.
pub struct PostgresWorkflowRepository {
    pool: sqlx::PgPool,
}

// runtime: process, config, telemetry, and transport binding.
pub async fn main_runtime() -> anyhow::Result<()> {
    let config = RuntimeConfig::from_env()?;
    let telemetry = init_telemetry(&config)?;
    let pool = connect_postgres(&config.database_url).await?;
    serve_http(config.bind_addr, pool, telemetry).await
}
```

## Extended Layer Compliance Matrix

| ID | Importer | Allowed import | Forbidden import | Reason |
|---|---|---|---|---|
| LAY-MAT-001 | kernel | `std`, value objects | `sqlx` | Kernel stays deterministic. |
| LAY-MAT-002 | kernel | pure validation helpers | `tokio` | Kernel has no scheduler dependency. |
| LAY-MAT-003 | kernel | generated enums only if owned | OpenAPI handlers | Transport cannot define invariant shape. |
| LAY-MAT-004 | domain | kernel | Postgres repositories | Domain expresses business events. |
| LAY-MAT-005 | domain | domain errors | `anyhow::Error` | Public domain errors stay typed. |
| LAY-MAT-006 | domain | serde value objects | Axum extractors | HTTP stays outside domain. |
| LAY-MAT-007 | usecase | domain ports | SQL migrations | Usecase orchestrates ports only. |
| LAY-MAT-008 | usecase | Cedar port trait | Cedar SDK client | SDK is adapter detail. |
| LAY-MAT-009 | usecase | outbox port trait | Kafka producer | Broker is adapter detail. |
| LAY-MAT-010 | adapter | usecase port trait | runtime config singleton | Adapter is reusable. |
| LAY-MAT-011 | adapter | `sqlx` | HTTP request body | Storage adapter knows storage only. |
| LAY-MAT-012 | adapter | Cedar SDK | business state machine mutation | Adapter evaluates, not decides. |
| LAY-MAT-013 | runtime | adapter constructors | kernel private modules | Runtime wires surfaces. |
| LAY-MAT-014 | runtime | tracing subscriber | domain invariant edits | Runtime is composition root. |
| LAY-MAT-015 | contract | schema definitions | persistence internals | Contracts describe external shape. |
| LAY-MAT-016 | migration | DDL | domain commands | Migration is storage authority. |
| LAY-MAT-017 | fixture | contract examples | production secrets | Fixtures are deterministic. |
| LAY-MAT-018 | policy | Cedar schema | Rust database type | Policy speaks authorization vocabulary. |
| LAY-MAT-019 | SLO | service indicators | function-private names | SLOs track user-visible health. |
| LAY-MAT-020 | ADR | architectural decision | stale local exception | ADR owns persistent rationale. |
| LAY-MAT-021 | Standard | cross-cutting rule | single-PR workaround | Standard owns repeatable discipline. |
| LAY-MAT-022 | Runbook | operator action | product roadmap | Runbook fixes live incidents. |
| LAY-MAT-023 | API | stable schema | private aggregate layout | API version is public contract. |
| LAY-MAT-024 | AsyncAPI | event envelope | direct database table | Event stream is not table mirror. |
| LAY-MAT-025 | Proto | service/rpc shape | UI copy | gRPC contract is machine surface. |
| LAY-MAT-026 | SDK | generated client | policy decision | SDK calls, does not authorize. |
| LAY-MAT-027 | Frontend shell | API client | database pool | Shell consumes public contract. |
| LAY-MAT-028 | Observability | metric labels | secret values | Telemetry is safe evidence. |
| LAY-MAT-029 | Audit | event schema | free-form logs | Audit rows are canonical. |
| LAY-MAT-030 | Governance | checker crates | runtime-only flags | Governance is promotion gate. |

## Extended Verification Checklist

LAY-REV-001. Run `cargo metadata` and confirm every `kernel` crate is dependency-light.

LAY-REV-002. Run `cargo run -p check-layer-enum --quiet`.

LAY-REV-003. Run `cargo run -p check-flat-crate-boundaries --quiet`.

LAY-REV-004. Confirm every `domain` crate exposes typed errors.

LAY-REV-005. Confirm every `usecase` crate defines ports for external effects.

LAY-REV-006. Confirm every adapter implements a port owned by usecase or domain.

LAY-REV-007. Confirm no adapter imports an unrelated adapter.

LAY-REV-008. Confirm runtime is the only composition root.

LAY-REV-009. Confirm generated transport types do not enter kernel invariants.

LAY-REV-010. Confirm OpenAPI handlers convert into domain commands.

LAY-REV-011. Confirm Proto handlers convert into domain commands.

LAY-REV-012. Confirm AsyncAPI consumers convert event payloads before mutation.

LAY-REV-013. Confirm Cedar policy results are mapped to domain rejection variants.

LAY-REV-014. Confirm audit emissions happen at usecase boundaries.

LAY-REV-015. Confirm storage migrations do not define business enums.

LAY-REV-016. Confirm business enums live in kernel or domain only.

LAY-REV-017. Confirm saga compensation logic is not embedded in HTTP handlers.

LAY-REV-018. Confirm retry logic is in workflow/usecase, not kernel.

LAY-REV-019. Confirm tracing setup is runtime-only.

LAY-REV-020. Confirm config loading is runtime-only.

LAY-REV-021. Confirm secrets never enter domain constructors.

LAY-REV-022. Confirm DTO names do not leak into aggregate names.

LAY-REV-023. Confirm domain tests do not require a database.

LAY-REV-024. Confirm kernel tests run without async runtime.

LAY-REV-025. Confirm adapter tests own integration fixtures.

LAY-REV-026. Confirm runtime smoke tests are the only process-level tests.

LAY-REV-027. Confirm architecture exceptions cite an ADR.

LAY-REV-028. Confirm ADR-0105 appears in violation diagnostics.

LAY-REV-029. Confirm the promote bundle includes layer checker output.

LAY-REV-030. Confirm no new layer token is introduced without this standard changing first.
