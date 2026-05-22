---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-foundry-supervisor
microservice: foundry-supervisor
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs: [ADR-0024, ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/foundry-split.json, /specs/foundry-supervisor-control-plane.json]
date: 2026-05-17
owner_team: axis-foundry-control-plane
doc_status: published
---

# PRD-foundry-supervisor: Supervisor + Control Plane for Agent Fleets

## Purpose

The `foundry-supervisor` µservice is the **control plane** for oyatie's Foundry stack. Per ADR-0131 §"Foundry split" it is one of six independent flat µservices that replace the legacy `foundry` bundle: `foundry-providers`, `foundry-runtime`, `foundry-supervisor`, `foundry-evidence`, `foundry-guardrails`, `foundry-eval`. This µservice owns:

- **Capability deployment** — admit, materialize, version, roll-forward, roll-back, retire capability definitions across the tenant fleet.
- **Agent-fleet lifecycle** — register, schedule, drain, evict, replace agent workers managed via a Kubernetes Operator (Foundry CRDs: `Agent`, `AgentDeployment`, `AutonomyPolicy`, `KillSwitch`).
- **Autonomy-tier policy enforcement** — Cedar-evaluated, default-deny enforcement that an agent's runtime tier (T0/T1/T2/T3) matches the capability + tenant entitlement before any `foundry-runtime` invocation.
- **Supervision event bus** — canonical event substrate (`AgentRegistered`, `CapabilityDeployed`, `AutonomyViolated`, `KillSwitchEngaged`, `FleetDrained`) consumed by `foundry-evidence` and `observability`.
- **Kill-switch + circuit-breaker** — sub-second engagement to halt a capability, an agent, a tenant scope, or the entire fleet on policy breach, autonomy violation, runaway cost, or operator decision.

This µservice is **shared substrate**: every Foundry-resident product (Workflow Studio, Application Shell agents, tenancy-bound automation) flows through it; tenants never invoke it directly. It originates natively in oyatie; no Bominal equivalent.

Hyperscaler peers: AWS Bedrock Agents control plane, Anthropic Claude control plane, OpenAI Assistants admin API, Google Vertex AI Agent admin. Parity matrix at `microservices/foundry-supervisor/competitor-parity-matrix.md`.

## Tenant Value

- **Tenant Outcome 1 — One-click capability rollout with safe rollback.** Tenant operators publish a new capability via Workflow Studio; supervisor admits → canary deploys → rolls forward to 100 % only when SLO + autonomy + cost gates green; auto-rollback on breach. Mirrors AWS Bedrock Agents canary rollout posture; differentiator is full integration with `observability` SLO gate (ADR-0139) and `foundry-evidence` cryptographic audit chain (ADR-0028).
- **Tenant Outcome 2 — Autonomy guarantees by Cedar policy, not by hope.** Every agent invocation is gated by an autonomy-policy Cedar evaluation; tier escalations (T0 → T3) require explicit DPA-recorded entitlement; violations engage the kill-switch within p99 ≤ 1 s.
- **Tenant Outcome 3 — Kill-switch coverage they can verify.** Tenants see a per-tenant kill-switch-coverage dashboard; auditors verify Ed25519-sealed engage/disengage records; pen-test exercises confirm coverage quarterly.
- **Tenant Outcome 4 — Per-tenant fleet isolation.** Each tenant's agent fleet lives in a per-tenant Kubernetes namespace under the `foundry-supervisor` operator; cross-tenant invocation is refused at the policy layer; quotas + cost budgets are per-tenant per `cost-budget.md`.
- **Internal Outcome 5 — Substrate uniformity.** Every µservice that emits agentic work goes through the same control plane; no per-product supervisor variants.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant operator | to author a capability definition (capability.yaml) via Workflow Studio + git PR | the supervisor can admit + deploy it into my tenant's fleet | capability-deployment | Must |
| FR-02 | supervisor admit-loop | to validate a capability definition against the Cedar policy + autonomy-ceiling + cost budget | rejected capabilities never reach a runtime worker | capability-deployment + autonomy-policy-enforcement | Must |
| FR-03 | supervisor rollout-loop | to canary deploy a capability across 1 % → 10 % → 50 % → 100 % of tenant fleet | bad releases are caught at low blast radius | capability-deployment | Must |
| FR-04 | kill-switch operator | to engage the kill-switch on a (tenant, capability, agent, or fleet) scope and have it propagate p99 ≤ 1 s | runaway agents stop within human-perceivable time | kill-switch-circuit-breaker | Must |
| FR-05 | foundry-runtime worker | to query the supervisor for "may I invoke capability C at autonomy tier T for tenant X right now?" | the runtime never executes an unauthorized invocation | autonomy-policy-enforcement | Must |
| FR-06 | foundry-evidence | to receive `SupervisionEvent` records (with Ed25519 signature) within lag ≤ 200 ms | the audit-chain Merkle tree never misses a supervision event | supervision-event-bus | Must |
| FR-07 | tenant operator | to drain an agent fleet (stop new invocations; complete in-flight) gracefully | maintenance windows do not lose in-flight work | agent-fleet-lifecycle | Must |
| FR-08 | reviewer agent | to read the latest fleet-state snapshot (per-agent, per-capability, per-tenant) for promotion gating | promotion gate refuses fast-forward when supervisor reports degraded fleet | agent-fleet-lifecycle | Must |
| FR-09 | observability | to receive supervisor's own SLI metrics (kill-switch latency, deployment latency, autonomy-violation rate) | SLO gate covers supervisor itself | supervision-event-bus | Must |
| FR-10 | ops-security operator | to revoke a tenant's entire autonomy entitlement (e.g., suspected compromise) with one CLI call | incident response can de-fang a tenant in seconds | autonomy-policy-enforcement | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Kill-switch engage latency (issue → in-flight workers refused) | ≤ 250 ms | ≤ 1 s | ≤ 2 s | mandated by ADR-0133 hyperscaler safety claim; AWS Bedrock Guardrails parity |
| Capability deployment latency (admit → 100 % rollout) | ≤ 90 s | ≤ 5 min | ≤ 15 min | gated by canary + SLO observe windows |
| Supervision-event emission lag (event → bus) | ≤ 50 ms | ≤ 200 ms | ≤ 500 ms | end-to-end from controller reconcile to AsyncAPI publish |
| Autonomy-policy evaluation (Cedar) | ≤ 5 ms | ≤ 15 ms | ≤ 50 ms | per-invocation precondition |
| Fleet-state query (per-tenant snapshot) | ≤ 30 ms | ≤ 100 ms | ≤ 300 ms | Postgres + Valkey materialized view |
| Operator reconcile loop (one CRD object) | ≤ 200 ms | ≤ 1 s | ≤ 3 s | per Kubernetes Operator best-practice |
| Deployment admit-loop throughput | — | 100 capability-defs/s/cell | — | bounded by Postgres write IOPS |

### Security

- All control-plane writes (admit, deploy, engage, drain) are signed by the supervisor's per-environment Ed25519 key (per Bominal ADR-0028).
- All cross-µservice calls (to `foundry-runtime`, `foundry-guardrails`, `foundry-evidence`) use mTLS + SPIFFE identity.
- Cedar v4 evaluator, default-deny; fragments fuzzed per `oya-check-cedar-fragment-coverage`.
- Secrets (Postgres credentials, Valkey ACL tokens, signing keys, OpenBao-issued tenant entitlements) follow the OpenBao SecretReference pattern; raw secrets never enter repo or logs.
- Kill-switch authority restricted to: tenant DPO (own scope only), ops-security on-call (any scope; 2-person rule for fleet-wide), supervisor controller (autonomy-policy auto-triggered scope).

### Audit + Compliance

- Every supervision event (`CapabilityDeployed`, `KillSwitchEngaged`, `KillSwitchDisengaged`, `AutonomyViolated`, `FleetDrained`, `AgentEvicted`, `DeploymentRolledBack`) emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Audit-chain seal latency ≤ 1 s per event.
- EU AI Act high-risk-system requirements (Annex III §5 — employment, §6 — law enforcement, §7 — migration, §8 — administration of justice) covered in `compliance.md` §"EU AI Act".

### Availability + SLO

- Availability target: 99.99 % monthly for the kill-switch engage path (the safety-critical surface).
- 99.95 % monthly for the deployment admit/rollout path.
- 99.9 % monthly for fleet-state query path.
- RTO: ≤ 5 min for control-plane availability. RPO: ≤ 30 s for fleet-state Postgres + Valkey.

### Data residency

- Per-tenant fleet metadata, autonomy entitlements, deployment history inherit the tenant's `jurisdiction_code` per ADR-0117. Postgres + Valkey instances are pack-pinned; cross-pack replication forbidden by default per `policy/data-residency.md`.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), this µservice uses: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-k8s-operator`, `rest`, `worker`, `sdk`, `app`. Five Bounded Contexts:

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `agent-fleet-lifecycle` | `oya-foundry-supervisor-agent-fleet-lifecycle-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-k8s-operator,rest,worker,sdk,app}` | Register, drain, evict, replace agents managed via Kubernetes CRDs | `Agent`, `AgentDeployment`, `FleetState`, `DrainHandle` |
| `capability-deployment` | `oya-foundry-supervisor-capability-deployment-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Admit, canary rollout, roll-forward, roll-back capability definitions | `CapabilityDefinition`, `Deployment`, `CanaryCohort`, `RolloutPhase`, `RolloutVerdict` |
| `autonomy-policy-enforcement` | `oya-foundry-supervisor-autonomy-policy-enforcement-{kernel,domain,usecase,api,adapter,rest,sdk,app}` | Cedar evaluator + tenant entitlement store + per-invocation precondition check | `AutonomyLevel`, `AutonomyEntitlement`, `PolicyDecision`, `CedarFragment` |
| `supervision-event-bus` | `oya-foundry-supervisor-supervision-event-bus-{kernel,usecase,api,adapter,worker,sdk,app}` | Internal AMQP/Redis event bus; publish/subscribe with delivery guarantees | `SupervisionEvent`, `EventTopic`, `Subscription` |
| `kill-switch-circuit-breaker` | `oya-foundry-supervisor-kill-switch-circuit-breaker-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Sub-second engage/disengage; multi-scope (tenant / capability / agent / fleet); Redis-replicated state | `KillSwitch`, `KillSwitchScope`, `EngageReason`, `DisengageAuthority` |

Naming justification — `agent-fleet-lifecycle`:

```
NAME: oya-foundry-supervisor-agent-fleet-lifecycle-<layer>
JUSTIFICATION:
- microservice = foundry-supervisor: per ADR-0131 §"Foundry split"; flat layout under
  microservices/foundry-supervisor/. No legacy bundle prefix.
- bc-tokens = agent-fleet-lifecycle: primary BC for fleet membership transitions;
  ADR-0056 v4.1 BC-explicit rule honoured (sibling BCs capability-deployment,
  kill-switch-circuit-breaker exist).
- layer = <layer>: one crate per canonical layer enum value (ADR-0105).
  - kernel: port traits + entities (Agent, AgentDeployment, FleetState, DrainHandle).
    Zero I/O. data_class annotated.
  - domain: pure fleet-state arithmetic; transition validation.
  - usecase: register / drain / evict / replace orchestrators against ports.
  - api: protocol-neutral I/O contracts; depends on kernel only.
  - adapter: protocol-neutral port impls (in-memory; tests).
  - adapter-postgres: backend-qualified per ADR-0105 Amendment 3; Postgres-backed
    fleet-state repository.
  - adapter-k8s-operator: backend-qualified; controller-runtime reconciler for
    AgentDeployment CRD.
  - rest: HTTP handlers (fleet query, drain trigger).
  - worker: long-lived reconcile + drain loops.
  - sdk: client library (Rust + future TS/Python) for tenant + ops portal.
  - app: composition root.
- exemptions claimed: none. -adapter-<backend> is the canonical pattern.
```

Naming justification — `kill-switch-circuit-breaker`:

```
NAME: oya-foundry-supervisor-kill-switch-circuit-breaker-<layer>
JUSTIFICATION:
- microservice = foundry-supervisor.
- bc-tokens = kill-switch-circuit-breaker: explicit because this BC has a
  load-bearing SLO (p99 ≤ 1 s engage latency) distinct from the rest of
  agent-fleet-lifecycle; separating it lets the BC's own SLO + crate set
  evolve without dragging the wider fleet-lifecycle.
- layer = <layer>: see ADR-0105.
- exemptions claimed: none.
```

Layer mapping per BC (per ADR-0105 13-layer enum; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-k8s-operator | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `agent-fleet-lifecycle` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `capability-deployment` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| `autonomy-policy-enforcement` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | ✓ |
| `supervision-event-bus` | ✓ | — | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | ✓ |
| `kill-switch-circuit-breaker` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | ✓ | ✓ |

Total crates introduced by this µservice: **49** (11 + 10 + 9 + 7 + 9 + 3 shared `app` consolidated == 47 distinct crate names; per-BC `app` is allowed sibling per ADR-0105 §"composition-root multiplicity").

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `FleetStateRepository` | `oya-foundry-supervisor-agent-fleet-lifecycle-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `AgentDeploymentReconciler` | same | `-adapter-k8s-operator` | `BEHAVIORAL_TENANT_PRODUCT` |
| `CapabilityDefinitionStore` | `oya-foundry-supervisor-capability-deployment-kernel` | `-adapter-postgres` | `INTERNAL_ONLY`, `AUDIT` |
| `RolloutVerdictEmitter` | same | `-adapter-postgres` + supervision-event-bus | `AUDIT` |
| `AutonomyEntitlementStore` | `oya-foundry-supervisor-autonomy-policy-enforcement-kernel` | `-adapter` (OpenBao-backed) | `SENSITIVE_PIPA_ART23`, `AUDIT` |
| `CedarEvaluator` | same | `-adapter` (Cedar v4 runtime) | `INTERNAL_ONLY` |
| `SupervisionEventPublisher` | `oya-foundry-supervisor-supervision-event-bus-kernel` | `-adapter` (Valkey Streams (Redis wire-compat) + AMQP) | `AUDIT`, `BEHAVIORAL_TENANT_PRODUCT` |
| `KillSwitchStateStore` | `oya-foundry-supervisor-kill-switch-circuit-breaker-kernel` | `-adapter` (Redis-replicated) | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `KillSwitchPropagator` | same | `-adapter-k8s-operator` (CRD watch fan-out) | `BEHAVIORAL_TENANT_PRODUCT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane (under `microservices/governance/`) refuses unannotated fields at PR-time.

Cross-product rule: `foundry-supervisor` MUST NOT import any product µservice crate. All cross-product flows go through Workflow events (this µservice publishes `SupervisionEvent`) or Ontology reads/writes. LEAN-A2 lane enforces.

CI lanes that must green:

```
oya gate validate lean-a1                --microservice foundry-supervisor
oya gate validate lean-a2                --microservice foundry-supervisor
oya gate validate port-location          --microservice foundry-supervisor
oya gate validate layer-correctness      --microservice foundry-supervisor
oya gate validate per-microservice-layout --microservice foundry-supervisor
oya gate validate statelessness          --microservice foundry-supervisor
oya gate validate shardability           --microservice foundry-supervisor
oya gate validate authority-cohesion     # registers HG-FND-SUP
oya gate validate cedar-fragment-coverage --microservice foundry-supervisor
oya gate validate hyperscaler-maturity-claims # ADR-0123
```

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `CapabilityDeployed` | rollout phase advanced (1 %, 10 %, 50 %, 100 %) | `foundry-evidence`, `observability`, `application` (workflow-studio UI) | rollout-state-machine |
| `KillSwitchEngaged` | autonomy violation, cost runaway, operator manual, SLO breach | `foundry-runtime` (refuses new invocations), `foundry-evidence`, `observability`, `grafana-oncall` | kill-switch-state-machine |
| `KillSwitchDisengaged` | operator releases (2-person rule for fleet-wide); auto-release on cause-cleared | same | same |
| `AutonomyViolated` | runtime invocation refused by Cedar precondition | `foundry-evidence`, `observability` | — |
| `FleetDrained` | tenant or ops requests drain | `foundry-runtime`, `observability` | drain-state-machine |
| `AgentEvicted` | health-check failure, autonomy violation, drain completion | `foundry-runtime`, `foundry-evidence` | — |
| `DeploymentRolledBack` | canary SLO breach OR operator manual | `foundry-evidence`, `observability` | rollback-state-machine |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `TenantRegistered` | `tenancy` | `agent-fleet-lifecycle` | create per-tenant Kubernetes namespace + default kill-switch (disengaged) + default autonomy entitlement (T0) |
| `TenantSuspended` | `tenancy` | `kill-switch-circuit-breaker` | engage tenant-scope kill-switch with reason `tenant_suspended` |
| `EligibilityChanged` | `observability` (per ADR-0139) | `capability-deployment` | held → pause in-flight rollout; rollback → engage rollback flow |
| `GuardrailViolation` | `foundry-guardrails` | `kill-switch-circuit-breaker` | engage capability-scope kill-switch |
| `EvalRegression` | `foundry-eval` | `capability-deployment` | pause rollout at current canary phase; emit `RolloutPhaseHeld` |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit |
|---|---|---|---|
| `CapabilityDefinition{id, version, autonomy_level, cost_budget, …}` | `defined_for→Tenant` | `capability-deployment` | Ed25519 |
| `Deployment{capability_id, deployed_at, rollout_phase, verdict}` | `deployment_of→CapabilityDefinition` | `capability-deployment` | Ed25519 |
| `FleetState{tenant, capability, agent_count, healthy_count, draining_count}` | `fleet_for→Tenant` | `agent-fleet-lifecycle` | Ed25519 |
| `KillSwitch{scope, engaged, reason, engaged_at, disengaged_at}` | `applies_to→Tenant|Capability|Agent|Fleet` | `kill-switch-circuit-breaker` | Ed25519 |
| `AutonomyEntitlement{tenant, capability, tier, granted_by, expires_at}` | `entitles→Tenant` | `autonomy-policy-enforcement` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Tenant` | `agent-fleet-lifecycle` | `where(active=true).select(jurisdiction_code, dpa_record)` |
| `OpenSloManifest` (from `observability`) | `capability-deployment` | `where(microservice="<capability-id>", env=production)` for rollout gates |

## Competitive Benchmark

| Competitor | Product / surface | Parity dimensions | Primary source |
|---|---|---|---|
| AWS Bedrock Agents | control plane | capability versioning, canary rollout, kill-switch (Guardrails), per-tenant scoping | `docs.aws.amazon.com/bedrock/latest/userguide/agents.html` |
| Anthropic Claude | control plane (workspaces + admin) | capability admit, autonomy + safety profile, kill-switch (escalation) | `docs.anthropic.com/en/docs/admin-api` |
| OpenAI Assistants API | admin surface | assistant lifecycle, tool admission, run cancel (kill switch analog) | `platform.openai.com/docs/api-reference/assistants` |
| Google Vertex AI Agent | Agent Builder admin | versioning, canary, cancel, audit | `cloud.google.com/vertex-ai/docs/generative-ai/agents/` |
| Databricks Mosaic AI Gateway | control plane | capability admit, rate-limit, kill-switch | `docs.databricks.com/en/generative-ai/` |

Key parity gaps to close (ordered):

1. **Per-component release pointer integration** — none of the competitors gate agentic capability rollouts on µservice-level SLO verdict (ADR-0139). Closing this delivers the same "promotion-gate uniformity" oyatie has elsewhere.
2. **Cryptographic event chain over supervision events** — competitors emit unsigned admin events; oyatie seals every event Ed25519 + Merkle per Bominal ADR-0028.
3. **Default-deny Cedar autonomy policy** — competitors lean on imperative configs (Bedrock guardrails, Claude system prompts). oyatie's Cedar default-deny matches the strictest hyperscaler IAM posture.
4. **Sub-second multi-scope kill-switch** — Bedrock Guardrails fires within ~2 s p99; oyatie targets ≤ 1 s p99.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Kill-switch engage end-to-end | ≤ 250 ms | ≤ 1 s | ≤ 2 s | mandatory per ADR-0133 HG-FND-SUP claim |
| Deployment admit→100% rollout | ≤ 90 s | ≤ 5 min | ≤ 15 min | bounded by canary observe windows |
| Supervision event lag | ≤ 50 ms | ≤ 200 ms | ≤ 500 ms | controller reconcile → bus publish |
| Autonomy-policy eval (Cedar) | ≤ 5 ms | ≤ 15 ms | ≤ 50 ms | per-invocation precondition |
| Postgres write IOPS (admit-loop) | — | 100 capability-defs/s | — | XS tier baseline |
| Valkey read IOPS (kill-switch query) | — | 50k ops/s/node | — | per Valkey Cluster sizing |

Error budget:
- Monthly error budget for kill-switch engage path: 0.01 % (≈ 4 min/month). Burn-rate alarm: 14.4× over 1h triggers Sev-1 page.
- Monthly error budget for deployment admit/rollout: 0.05 % (≈ 22 min/month).
- Self-SLOs authored in `microservices/foundry-supervisor/slos/`; consumed by `observability` per ADR-0139.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres (fleet state, deployment history, entitlement store) is sharded by tenant; Valkey Cluster (kill-switch state, supervision-event-bus stream) is replicated; controllers (Kubernetes Operator) are leased-leadership stateless.

**Active-active compatibility**: controllers + REST + worker are `stateless-compatible` (leadership election via Kubernetes leases). Postgres is master-replica per pack region; Valkey Cluster is 3-replica per pack region.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active tenants per cell | 1000 | 10000 | Postgres connection-pool > 70 % |
| Concurrent agent fleets | 5000 | 50000 | controller reconcile lag > 1 s |
| Deployment admit rate | 50/s | 200/s | admit-loop queue > 60 s |
| Kill-switch state in Valkey | 10k engaged switches | 100k | Valkey cluster CPU > 70 % |

Scale-out:
- Controllers: HPA on CPU > 70 %; min 3 replicas (Kubernetes-leader-elected; only one is active reconciler per shard).
- REST: HPA on RPS; min 3 replicas.
- Worker: HPA on queue depth; min 2 replicas.
- Postgres: vertical scale + horizontal sharding by tenant hash; PgBouncer pooled.
- Valkey: horizontal scale-out via Cluster shards; per-pack replication.

Sharding:
- Postgres partitions by `tenant_hash MOD num_shards`.
- Kubernetes per-tenant namespace; controllers shard by namespace label.
- `oya-check-shardability-cli` CI lane verifies partition-key presence on every entity.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Capability definition YAML at `microservices/<tenant>/capabilities/<cap>.yaml` validates against capability v1 schema | `cargo run -p oya-foundry-supervisor-capability-deployment-rest -- validate <path>` exit 0 |
| AC-02 | Kill-switch engage end-to-end ≤ 1 s p99 across 100k workers | scripted load + chaos drill at `tests/e2e/kill-switch-latency.rs` |
| AC-03 | Capability deployment canary ramps 1 → 10 → 50 → 100 % gated by `observability` `EligibilityChanged` | e2e drill `tests/e2e/canary-rollout-gated.rs` |
| AC-04 | Autonomy-policy Cedar evaluation refuses tier escalation without DPA entitlement | unit + integration `tests/e2e/autonomy-ceiling-refusal.rs` |
| AC-05 | Supervision event published to bus within ≤ 200 ms p99 of state transition | timed integration test |
| AC-06 | Fleet drain completes with zero in-flight loss for ≤ 100 agents | e2e drill `tests/e2e/drain-no-loss.rs` |
| AC-07 | Postgres failover (master loss) recovers control-plane availability within ≤ 30 s | chaos drill |
| AC-08 | Valkey failover (one replica loss) does not breach kill-switch p99 ≤ 1 s | chaos drill |
| AC-09 | `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-supervisor` exit 0 | ADR-0131 lane |
| AC-10 | `cargo run -p oya-dev-cli -- gate validate authority-cohesion` exit 0 with HG-FND-SUP registered | ADR-0123 lane |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Kubernetes Operator framework: kube-rs vs operator-rs vs in-house controller-runtime port | axis-foundry-control-plane | resolved IP-001 — kube-rs (Rust-native, oyatie-language alignment) |
| 2 | Valkey Cluster vs Sentinel for kill-switch state | ops-sre-reliability | resolved IP-006 — Cluster (3-replica, automatic sharding) |
| 3 | Cedar fragment authoring: in-repo vs OpenBao-stored | ops-security | resolved IP-005 — in-repo (PR-reviewed) + per-tenant overlays in OpenBao |
| 4 | Kill-switch propagation channel: CRD watch vs Valkey pub-sub | axis-foundry-control-plane | resolved IP-009 — both; CRD watch primary, Valkey fallback for sub-second |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0024 | Foundry eval harness contract | this µservice consumes `EvalRegression` events |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase rename | new crates use `usecase` |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-FND-SUP registers here |
| ADR-0139 | Agentic SLO-gated promotion | consumes `EligibilityChanged` for rollout gates |
| ADR-0131 | Per-microservice flat layout (Foundry split) | this PRD lives under that split |
| ADR-0132 | Product-suite-and-bundle dissolution | flat layout precedent |
| ADR-0133 | Industry-best-practice conformance | self-SLO authoring + HG-FND-SUP claim |
| ADR-0140 | Cedar policy enforcement | autonomy-policy-enforcement BC implements |
