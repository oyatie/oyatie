---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-002-redis-and-postgres-baseline
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability
acceptance_lanes: [helm-install-smoke, postgres-rls-coverage, session-prefix-isolation, foundry-runtime-iac-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Valkey 8.1 (Redis wire-compat) + Postgres 16 LTS baseline

## Intent

Ship Helm charts for Valkey 8.1 (Redis wire-compat) OSS LTS cluster (6 shards × 1 primary + 1 replica) and Postgres 16 LTS primary + read-replica. Bind ACLs + RLS to tenant-isolation invariants TI-01..TI-05. Wire OpenBao SecretReferences for Valkey AUTH + Postgres credentials. Provision capability_mirror + session_mutation_log + invocation_lifecycle tables with RLS policies.

## ChangeSet boundary

All paths under `microservices/foundry/iac/helm/{redis,postgres}/` + `iac/postgres-schema/`. No Rust crate changes.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/redis/Chart.yaml` | create (Valkey 8.1 (Redis wire-compat) OSS LTS pin) |
| `iac/helm/redis/values.yaml` | create (cluster mode; 6 shards; TLS + AUTH; ACL declared inline) |
| `iac/helm/redis/expected-acl.txt` | create (canonical ACL for drift detection) |
| `iac/helm/postgres/Chart.yaml` | create (Postgres 16 LTS pin) |
| `iac/helm/postgres/values.yaml` | create (TDE; streaming replication; WAL archive to OCI object-storage) |
| `iac/postgres-schema/001-capability-mirror.sql` | create (table + RLS policy) |
| `iac/postgres-schema/002-session-mutation-log.sql` | create (table + RLS) |
| `iac/postgres-schema/003-invocation-lifecycle.sql` | create (table + RLS + indexes) |
| `iac/postgres-schema/004-row-level-security-policies.sql` | create (per-tenant RLS policies) |
| `iac/postgres-schema/005-audit-triggers.sql` | create (audit-chain seal triggers) |

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/redis/
helm lint microservices/foundry/iac/helm/postgres/
psql --dry-run -f microservices/foundry/iac/postgres-schema/001-capability-mirror.sql
cargo run -p oya-dev-cli -- gate validate postgres-rls-coverage --microservice foundry-runtime
cargo run -p oya-dev-cli -- gate validate session-prefix-isolation --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| Valkey ACL probe | `default` user disabled; per-tenant role refused on cross-prefix |
| Postgres RLS coverage | Every multi-tenant table has RLS policy + denies cross-tenant SELECT |
| OpenBao SecretReference materialisation | Pods receive Valkey AUTH + Postgres creds without raw values in environment |
| Streaming replication health | `pg_replication_lag_seconds < 30` for ≥5min |

## Halt Conditions

- Any table without RLS — refactor.
- Valkey `default` user enabled — refactor (security risk).
- Raw secrets in pod env — refactor.

## Next IP

[`IP-003-capability-executor-kernel.md`](IP-003-capability-executor-kernel.md)

## References

- `policy/runtime-isolation.md` TI-01..TI-05.
- Valkey 8.1 (Redis wire-compat) — `redis.io/docs/about/releases/7-4-0/`.
- Postgres 16 LTS RLS — `postgresql.org/docs/16/ddl-rowsecurity.html`.

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `runtime`-bounded-context slice for `IP-002: Valkey 8.1 (Redis wire-compat) + Postgres 16 LTS baseline`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: session-coherent hosted agent invocation without tenant-owned runtime infrastructure. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/runtime-capability-execute.yaml`, `microservices/foundry/capabilities/runtime-session-create.yaml`, `microservices/foundry/capabilities/runtime-session-resume.yaml`, `microservices/foundry/contracts/openapi/runtime-foundry-runtime.yaml`, and the policy set `microservices/foundry/policy/runtime-tenant-scope.cedar`, `microservices/foundry/policy/runtime-runtime-isolation.md`, `microservices/foundry/policy/runtime-ci-scope.cedar`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `runtime` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-foundry-api/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/runtime-foundry-runtime.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, and `microservices/foundry/contracts/proto/runtime-foundry-runtime.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/runtime-tenant-scope.cedar`, `microservices/foundry/policy/runtime-runtime-isolation.md`, `microservices/foundry/policy/runtime-ci-scope.cedar`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/runtime-latency.openslo.yaml`, `microservices/foundry/slos/runtime-availability.openslo.yaml`, `microservices/foundry/slos/runtime-freshness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/runtime-capability-execute.yaml`, `microservices/foundry/capabilities/runtime-session-create.yaml`, `microservices/foundry/capabilities/runtime-session-resume.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `runtime` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-foundry-api/src/lib.rs`.
- Contract parity for `microservices/foundry/contracts/openapi/runtime-foundry-runtime.yaml` and `microservices/foundry/contracts/proto/runtime-foundry-runtime.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/runtime-tenant-scope.cedar`, `microservices/foundry/policy/runtime-runtime-isolation.md`, `microservices/foundry/policy/runtime-ci-scope.cedar`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/runtime-latency.openslo.yaml`, `microservices/foundry/slos/runtime-availability.openslo.yaml`, `microservices/foundry/slos/runtime-freshness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| OpenAI Assistants threads/runs and AWS Bedrock Agents runtime | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
