---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-015-hg-fr-hyperscaler-gate-registration
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime + council-architecture
acceptance_lanes: [authority-cohesion, hyperscaler-maturity-claims]
---

# IP-015: Register HG-FR in /specs/hyperscaler-gates.json (ADR-0123)

## Intent

Register the foundry-runtime hyperscaler-maturity gate `HG-FR` per ADR-0123. The gate asserts foundry-runtime carries the per-µservice substrate set: PRD + Phase + IPs + threat-model + DPIA + compliance + 4 Cedar policy fragments + runtime-isolation + data-residency + 6 runbooks + 4 OpenSLO manifests + IaC + capability catalog. Final gate that closes M01 for this µservice.

## ChangeSet boundary

Edit `/specs/hyperscaler-gates.json` to add the HG-FR entry. Update `registry/artifact-capabilities-registry.json` to register the µservice's capability catalog. No Rust crate changes (the gate logic itself lives in `oya-foundry-fitness-hyperscaler-maturity-claims` shared lane crate).

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | modify (add HG-FR entry) |
| `/registry/artifact-capabilities-registry.json` | modify (register foundry-runtime catalog) |
| `/registry/knowledge-graph-semantic.json` | modify (add foundry-runtime µservice node + edges to siblings) |

## HG-FR Entry Shape

```json
{
  "HG-FR": {
    "microservice": "foundry-runtime",
    "name": "Foundry runtime maturity claim",
    "required_artifacts": [
      "microservices/foundry-runtime/PRD.md",
      "microservices/foundry-runtime/PHASE-01-AGENT-RUNTIME-AND-CAPABILITY-EXECUTION.md",
      "microservices/foundry-runtime/threat-model.md",
      "microservices/foundry-runtime/dpia.md",
      "microservices/foundry-runtime/compliance.md",
      "microservices/foundry-runtime/policy/runtime-isolation.md",
      "microservices/foundry-runtime/policy/data-residency.md",
      "microservices/foundry-runtime/policy/tenant-scope.cedar",
      "microservices/foundry-runtime/policy/ci-scope.cedar",
      "microservices/foundry-runtime/policy/auditor-scope.cedar",
      "microservices/foundry-runtime/policy/public-read.cedar",
      "microservices/foundry-runtime/cost-budget.md",
      "microservices/foundry-runtime/failure-modes.md",
      "microservices/foundry-runtime/capacity-model.md",
      "microservices/foundry-runtime/multi-region.md",
      "microservices/foundry-runtime/incident-response.md",
      "microservices/foundry-runtime/backfill-replay.md",
      "microservices/foundry-runtime/sdk-plan.md",
      "microservices/foundry-runtime/competitor-parity-matrix.md",
      "microservices/foundry-runtime/runbooks/runtime-pod-crash.md",
      "microservices/foundry-runtime/runbooks/session-state-recovery.md",
      "microservices/foundry-runtime/runbooks/capability-registry-resync.md",
      "microservices/foundry-runtime/runbooks/redis-failover.md",
      "microservices/foundry-runtime/runbooks/autonomy-violation-quarantine.md",
      "microservices/foundry-runtime/runbooks/emergency-runtime-drain.md",
      "microservices/foundry-runtime/contracts/openapi/foundry-runtime.yaml",
      "microservices/foundry-runtime/contracts/asyncapi/foundry-runtime-events.yaml",
      "microservices/foundry-runtime/contracts/proto/foundry-runtime.proto",
      "microservices/foundry-runtime/capabilities/capability-execute.yaml",
      "microservices/foundry-runtime/capabilities/session-create.yaml",
      "microservices/foundry-runtime/capabilities/session-resume.yaml",
      "microservices/foundry-runtime/dashboards/invocation-rate.json",
      "microservices/foundry-runtime/dashboards/session-pool-health.json",
      "microservices/foundry-runtime/dashboards/autonomy-tier-mix.json",
      "microservices/foundry-runtime/slos/availability.openslo.yaml",
      "microservices/foundry-runtime/slos/latency.openslo.yaml",
      "microservices/foundry-runtime/slos/correctness.openslo.yaml",
      "microservices/foundry-runtime/slos/freshness.openslo.yaml"
    ],
    "required_lanes_green": [
      "per-microservice-layout",
      "authority-cohesion",
      "lean-a1",
      "lean-a2",
      "port-location",
      "layer-correctness",
      "statelessness",
      "shardability",
      "session-prefix-isolation",
      "postgres-rls-coverage",
      "autonomy-gate-presence",
      "cedar-fragment-coverage",
      "dsr-cascade-coverage",
      "openslo-conformance",
      "foundry-runtime-iac-smoke",
      "foundry-runtime-load-latency"
    ],
    "competitor_parity_target": "AWS Bedrock Agent runtime + GCP Vertex AI Agent Builder + Azure AI Foundry runtime + LangServe + OpenAI Assistants",
    "competitor_parity_matrix": "microservices/foundry-runtime/competitor-parity-matrix.md",
    "claim_bounded_by": "ADR-0123 hyperscaler-maturity-claim-gate forbidden_claims list"
  }
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
jq '.["HG-FR"]' /specs/hyperscaler-gates.json | wc -l   # expect > 30 lines
```

## Test Plan

| Test | Verifies |
|---|---|
| HG-FR registered in /specs/hyperscaler-gates.json | jq filter returns object |
| Every required_artifact exists | file-existence loop |
| Every required_lane is registered | cross-check with `/specs/quality/lanes.yaml` |
| Competitor-parity-matrix.md exists + cites at least 5 competitors | text grep |
| Claim-bounded-by enforces forbidden_claims | gate lane refuses claims like "faster than Bedrock" |

## Halt Conditions

- A required_artifact is missing — refactor (back-fill the artifact before HG-FR registration).
- Required_lane not registered — refactor.

## Next IP

(Phase exit gate per `PHASE-01-AGENT-RUNTIME-AND-CAPABILITY-EXECUTION.md` §"Exit gate". This is the final IP.)

## References

- ADR-0123 (hyperscaler-maturity-claim-gate).
- `/specs/hyperscaler-gates.json`.
- `microservices/foundry-runtime/competitor-parity-matrix.md`.
- `microservices/observability/IP-015-canary-cohort-weighting.md` (precedent shape for HG registration IP).
