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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: Register HG-FR in /specs/hyperscaler-gates.json (ADR-0123)

## Intent

Register the foundry-runtime hyperscaler-maturity gate `HG-FR` per ADR-0123. The gate asserts foundry-runtime carries the per-µservice substrate set: PRD + Phase + IPs + threat-model + DPIA + compliance + 4 Cedar policy fragments + runtime-isolation + data-residency + 6 runbooks + 4 OpenSLO manifests + IaC + capability catalog. Final gate that closes M01 for this µservice.

## ChangeSet boundary

Edit `/specs/hyperscaler-gates.json` to add the HG-FR entry. Update `registry/artifact-capabilities-registry.json` to register the µservice's capability catalog. No Rust crate changes (the gate logic itself lives in `oya-governance-hyperscaler-maturity-claims` shared lane crate).

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
      "microservices/intelligence/PRD.md",
      "microservices/intelligence/PHASE-01-AGENT-RUNTIME-AND-CAPABILITY-EXECUTION.md",
      "microservices/intelligence/threat-model.md",
      "microservices/intelligence/dpia.md",
      "microservices/intelligence/compliance.md",
      "microservices/intelligence/policy/runtime-isolation.md",
      "microservices/intelligence/policy/data-residency.md",
      "microservices/intelligence/policy/tenant-scope.cedar",
      "microservices/intelligence/policy/ci-scope.cedar",
      "microservices/intelligence/policy/auditor-scope.cedar",
      "microservices/intelligence/policy/public-read.cedar",
      "microservices/intelligence/cost-budget.md",
      "microservices/intelligence/failure-modes.md",
      "microservices/intelligence/capacity-model.md",
      "microservices/intelligence/multi-region.md",
      "microservices/intelligence/incident-response.md",
      "microservices/intelligence/backfill-replay.md",
      "microservices/intelligence/sdk-plan.md",
      "microservices/intelligence/competitor-parity-matrix.md",
      "microservices/intelligence/runbooks/runtime-pod-crash.md",
      "microservices/intelligence/runbooks/session-state-recovery.md",
      "microservices/intelligence/runbooks/capability-registry-resync.md",
      "microservices/intelligence/runbooks/redis-failover.md",
      "microservices/intelligence/runbooks/autonomy-violation-quarantine.md",
      "microservices/intelligence/runbooks/emergency-runtime-drain.md",
      "microservices/intelligence/contracts/openapi/foundry-runtime.yaml",
      "microservices/intelligence/contracts/asyncapi/foundry-runtime-events.yaml",
      "microservices/intelligence/contracts/proto/foundry-runtime.proto",
      "microservices/intelligence/capabilities/capability-execute.yaml",
      "microservices/intelligence/capabilities/session-create.yaml",
      "microservices/intelligence/capabilities/session-resume.yaml",
      "microservices/intelligence/dashboards/invocation-rate.json",
      "microservices/intelligence/dashboards/session-pool-health.json",
      "microservices/intelligence/dashboards/autonomy-ceiling-mix.json",
      "microservices/intelligence/slos/availability.openslo.yaml",
      "microservices/intelligence/slos/latency.openslo.yaml",
      "microservices/intelligence/slos/correctness.openslo.yaml",
      "microservices/intelligence/slos/freshness.openslo.yaml"
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
    "competitor_parity_matrix": "microservices/intelligence/competitor-parity-matrix.md",
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
- `microservices/intelligence/competitor-parity-matrix.md`.
- `microservices/observability/IP-015-canary-cohort-weighting.md` (precedent shape for HG registration IP).
