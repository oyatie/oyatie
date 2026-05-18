# foundry-supervisor

Supervisor + control plane for oyatie's Foundry stack. Per ADR-0131 §"Foundry split", one of six independent flat µservices that replace the legacy `foundry` bundle: `foundry-providers`, `foundry-runtime`, **`foundry-supervisor`**, `foundry-evidence`, `foundry-guardrails`, `foundry-eval`.

This µservice owns:

- Capability deployment (admit → canary → roll-forward / roll-back)
- Agent-fleet lifecycle (Kubernetes Operator pattern; per-tenant namespaces)
- Autonomy-tier policy enforcement (Cedar v4 default-deny)
- Supervision event bus (Valkey Streams (Redis wire-compat) + AMQP; Ed25519-signed)
- Kill-switch / circuit-breaker (p99 ≤ 1 s engage; 2-person rule on fleet-wide)

## Documents

| Doc | Purpose |
|---|---|
| [`PRD.md`](PRD.md) | Product Requirements Document |
| [`PHASE-01-CONTROL-PLANE-LANDING.md`](PHASE-01-CONTROL-PLANE-LANDING.md) | Phase spec: land control plane end-to-end |
| [`threat-model.md`](threat-model.md) | STRIDE + LINDDUN + EU AI Act risk catalog |
| [`dpia.md`](dpia.md) | GDPR Art. 35 DPIA + EU AI Act Art. 27 FRIA |
| [`compliance.md`](compliance.md) | SOC 2 / ISO 27001 / GDPR / EU AI Act / HIPAA / KR PIPA / APPI / LGPD / DPDPA / PDPL mappings |
| [`failure-modes.md`](failure-modes.md) | 15 failure modes with detection, RTO, recovery runbook |
| [`incident-response.md`](incident-response.md) | Severity classification + escalation + regulatory notifications |
| [`multi-region.md`](multi-region.md) | 11-pack topology + BCDR |
| [`capacity-model.md`](capacity-model.md) | Sizing formulas + reference baselines |
| [`cost-budget.md`](cost-budget.md) | Per-component monthly cost + FinOps levers |
| [`backfill-replay.md`](backfill-replay.md) | Backfill + replay contract |
| [`sdk-plan.md`](sdk-plan.md) | Multi-language SDK strategy |
| [`competitor-parity-matrix.md`](competitor-parity-matrix.md) | AWS Bedrock / Anthropic / OpenAI / Vertex AI / Databricks parity |

Policy fragments under [`policy/`](policy/):
- `supervisor-isolation.md` — per-tenant fleet boundaries (Postgres RLS + Valkey ACL + K8s ns + Cedar)
- `data-residency.md` — pack-pinning + cross-pack-forbidden + retention
- `tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`

Runbooks under [`runbooks/`](runbooks/):
- `kill-switch-engage.md`, `deployment-rollback.md`, `fleet-state-recovery.md`, `autonomy-violation.md`, `kubernetes-operator-restart.md`, `supervision-bus-replay.md`

Contracts under [`contracts/`](contracts/): OpenAPI 3.2 (REST), AsyncAPI 3.1 (events), proto (gRPC).

Capabilities under [`capabilities/`](capabilities/): `deploy-capability`, `engage-kill-switch`, `query-fleet-state`.

Dashboards under [`dashboards/`](dashboards/): `deployment-rate.json`, `kill-switch-coverage.json`, `autonomy-violation-rate.json`.

Catalog under [`catalog/`](catalog/): 46 crate records (5 BCs × per-layer).

IaC under [`iac/`](iac/): Helm charts for `supervisor-controller`, `postgres` (HA + Patroni), `redis` (Cluster); Kustomize `base` + `overlays/pack-kr`.

15 Implementation Plans: [`IP-001-postgres-layer-a-iac.md`](IP-001-postgres-layer-a-iac.md) → [`IP-015-e2e-drills-and-dashboards.md`](IP-015-e2e-drills-and-dashboards.md).

## Bounded Contexts

Five BCs (per PRD §"Bounded Contexts"):

1. **agent-fleet-lifecycle** (11 crates) — register, drain, evict, replace agents via K8s CRDs
2. **capability-deployment** (10 crates) — admit + canary + roll-forward + roll-back; SLO-gated
3. **autonomy-policy-enforcement** (8 crates) — Cedar v4 + tenant entitlements; per-invocation precondition
4. **supervision-event-bus** (7 crates) — Valkey Streams (Redis wire-compat) + AMQP; Ed25519-signed
5. **kill-switch-circuit-breaker** (10 crates incl. `-adapter-k8s-operator`) — p99 ≤ 1 s; multi-scope; 2-person rule fleet-wide

Total Rust crates: 46.

## Performance Targets

- Kill-switch engage p99 ≤ 1 s (mandated by ADR-0133 HG-FND-SUP claim)
- Capability admit → 100 % rollout p99 ≤ 5 min
- Supervision event emission lag p99 ≤ 200 ms
- Cedar evaluation p99 ≤ 15 ms

## Cross-µservice integration

Consumes:
- `tenancy` → `TenantRegistered`, `TenantSuspended`
- `observability` → `EligibilityChanged` (per ADR-0139)
- `foundry-guardrails` → `GuardrailViolation`
- `foundry-eval` → `EvalRegression`

Produces:
- `CapabilityDeployed`, `KillSwitchEngaged`, `KillSwitchDisengaged`, `AutonomyViolated`, `FleetDrained`, `AgentEvicted`, `DeploymentRolledBack`

All cross-µservice flow via Workflow events + Ontology reads/writes (zero direct imports). LEAN-A2 lane enforces.

## Status

M01-foundation, Phase 01 (P01-control-plane-landing). 15 IPs pending implementation per PHASE-01-CONTROL-PLANE-LANDING.md.

## References

ADR-0024, ADR-0028 (Bominal), ADR-0056, ADR-0105, ADR-0106, ADR-0110, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145). EU AI Act 2024/1689. AWS Bedrock Agents control-plane (peer benchmark).
