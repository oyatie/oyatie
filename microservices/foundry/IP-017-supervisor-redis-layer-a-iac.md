---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-002-redis-layer-a-iac
status: pending
execution_unit: ChangeSet
owner: ops-sre-reliability
acceptance_lanes: [helm-lint, helm-install-smoke, oya-check-redis-acl-enforced]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Valkey Cluster Layer-A IaC

## Intent

Helm chart for Valkey Cluster (3 shards × 2 replicas per pack region); kill-switch state cache + supervision-event-bus Valkey Streams (Redis wire-compat); per-pod ACL tokens.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/foundry/iac/helm/redis/Chart.yaml` | create |
| `microservices/foundry/iac/helm/redis/values.yaml` | create |
| `microservices/foundry/iac/kustomize/base/kustomization.yaml` | update |

## Substrate selections

- Valkey 8.1 (Redis wire-compat) (cite redis.io/docs/management/scaling/).
- Cluster mode with 3 shards × 2 replicas.
- AOF every-second.
- Per-user ACL with pattern-bounded key access.
- OpenBao-issued ACL tokens (rotated 30d).

## Acceptance Gates

```bash
helm lint microservices/foundry/iac/helm/redis
helm install --dry-run --debug -n foundry-supervisor redis microservices/foundry/iac/helm/redis
cargo run -p oya-dev-cli -- gate validate redis-acl-enforced --microservice foundry-supervisor
```

## Test Plan

| Test | Verifies |
|---|---|
| Helm lint | chart syntactic |
| Helm install smoke (kind cluster) | cluster forms 3 shards × 2 replicas |
| AOF every-second | verified via `redis-cli config get appendfsync` |
| ACL pattern test | per-tenant token cannot read other-tenant keys |

## Halt Conditions

- ACL `default` user has any non-default access.
- AOF disabled.

## Next IP

[`IP-003-k8s-operator-iac.md`](IP-003-k8s-operator-iac.md)

## References

- `policy/supervisor-isolation.md` TI-R-*.
- Valkey Cluster — `redis.io/docs/management/scaling/`.
- Valkey ACL — `redis.io/docs/management/security/acl/`.
- `capacity-model.md` §"Valkey Cluster Sizing".

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `supervisor`-bounded-context slice for `IP-002: Valkey Cluster Layer-A IaC`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: tenant-visible fleet control with kill-switch and capability deployment evidence. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/supervisor-deploy-capability.yaml`, `microservices/foundry/capabilities/supervisor-engage-kill-switch.yaml`, `microservices/foundry/capabilities/supervisor-query-fleet-state.yaml`, `microservices/foundry/contracts/openapi/supervisor-foundry-supervisor.yaml`, and the policy set `microservices/foundry/policy/supervisor-tenant-scope.cedar`, `microservices/foundry/policy/supervisor-supervisor-isolation.md`, `microservices/foundry/policy/supervisor-ci-scope.cedar`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `supervisor` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-supervisor-kernel/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/supervisor-foundry-supervisor.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`, and `microservices/foundry/contracts/proto/supervisor-foundry-supervisor.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/supervisor-tenant-scope.cedar`, `microservices/foundry/policy/supervisor-supervisor-isolation.md`, `microservices/foundry/policy/supervisor-ci-scope.cedar`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/supervisor-command-propagation.openslo.yaml`, `microservices/foundry/slos/supervisor-fleet-state-freshness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/supervisor-deploy-capability.yaml`, `microservices/foundry/capabilities/supervisor-engage-kill-switch.yaml`, `microservices/foundry/capabilities/supervisor-query-fleet-state.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `supervisor` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-supervisor-kernel/src/lib.rs`.
- Contract parity for `microservices/foundry/contracts/openapi/supervisor-foundry-supervisor.yaml` and `microservices/foundry/contracts/proto/supervisor-foundry-supervisor.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/supervisor-tenant-scope.cedar`, `microservices/foundry/policy/supervisor-supervisor-isolation.md`, `microservices/foundry/policy/supervisor-ci-scope.cedar`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/supervisor-command-propagation.openslo.yaml`, `microservices/foundry/slos/supervisor-fleet-state-freshness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| Palantir AIP Operator and Azure AI Foundry deployment controls | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
