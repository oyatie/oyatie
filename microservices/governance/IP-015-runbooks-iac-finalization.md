---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-015-runbooks-iac-finalization
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability
acceptance_lanes: [cargo-check, cargo-build, runbook-index, runbook-freshness, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: Finalize 6 runbooks + Helm charts + Kustomize overlays + HG-GOV registration

## Intent

Finalize operational + IaC artifacts; register HG-GOV gate per ADR-0123; final validation of full phase exit.

## ChangeSet boundary

- 6 runbooks (already authored in this Turn): smoke-test that all paths in runbooks actually exist; verify CLI commands work.
- Helm charts at `iac/helm/{lane-runner-pool,postgres,evidence-store}/`.
- Kustomize overlays at `iac/kustomize/{base,overlays/pack-kr}/`.
- Terraform module stubs at `iac/terraform/` for pack onboarding.
- Register `HG-GOV` in `/specs/hyperscaler-gates.json`.
- Author `microservices/governance/audit/industry-best-practice-conformance.md` per-µservice audit overlay.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/helm/lane-runner-pool/{Chart.yaml,values.yaml}` | create — ARC runner pool Helm chart |
| `iac/helm/postgres/{Chart.yaml,values.yaml}` | create — HA Postgres (Patroni) Helm chart |
| `iac/helm/evidence-store/{Chart.yaml,values.yaml}` | create — S3 bucket + Postgres for blob index |
| `iac/kustomize/base/kustomization.yaml` | create |
| `iac/kustomize/overlays/pack-kr/kustomization.yaml` | create |
| `iac/terraform/pack-kr/{main.tf,variables.tf,outputs.tf}` | create — placeholder |
| `/specs/hyperscaler-gates.json` | edit — register `HG-GOV` |
| `microservices/governance/audit/industry-best-practice-conformance.md` | create — per-axis self-audit overlay |
| `microservices/governance/runbooks/error-budget-policy.md` | create — derive from SLO targets |

## Code Shape

```yaml
# iac/helm/lane-runner-pool/values.yaml
arc:
  enabled: true
  controller:
    replicas: 2
  runners:
    minReplicas: 8
    maxReplicas: 200
    image: ghcr.io/oyatie/governance-runner:0.1.0
    resources:
      requests: { cpu: "1", memory: "2Gi" }
      limits: { cpu: "4", memory: "8Gi" }
    networkPolicy:
      egress:
        allowedHosts:
          - "crates.io"
          - "github.com"
          - "postgres.governance.svc.cluster.local"
          - "s3.ap-seoul-1.oci.example.com"
          - "audit-chain.audit-chain.svc.cluster.local"
          - "openbao.cloud-secrets.svc.cluster.local"
    spiffe:
      enabled: true
      audience: "spiffe://oyatie/governance/lane-runner"
```

```json
# /specs/hyperscaler-gates.json (registration excerpt)
{
  "gates": {
    "HG-GOV": {
      "name": "Governance Maturity Gate",
      "microservice": "governance",
      "owner": "axis-foundry",
      "claims": {
        "industry-benchmarked-conformance": "ADR-0133 §6 Axes",
        "audit-replayable-evidence": "SOC 2 CC7.4 + ISO 27001 A.5.34",
        "slsa-build-source-l3": "SLSA v1.0",
        "self-application": "Invariant 10 per policy/lane-execution.md"
      },
      "verification": "cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice governance",
      "added_in_adr": "ADR-0123 + ADR-0133",
      "registered_at": "2026-05-17"
    }
  }
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate runbook-index --microservice governance
cargo run -p oya-dev-cli -- gate validate runbook-freshness --microservice governance
cargo run -p oya-dev-cli -- gate validate authority-cohesion       # HG-GOV registered
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice governance
# IaC smoke test
helm lint iac/helm/lane-runner-pool
helm lint iac/helm/postgres
helm lint iac/helm/evidence-store
kubectl apply -k iac/kustomize/overlays/pack-kr/ --dry-run=client
# Phase exit:
cargo nextest run --workspace --all-features
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice governance
cargo run -p oya-dev-cli -- gate validate industry-best-practice-conformance --microservice governance
```

## Test Plan

| Test | Verifies |
|---|---|
| Per-runbook smoke: every CLI command in runbook is invocable | runbooks usable |
| Helm lint passes | chart sanity |
| Kustomize dry-run resolves | overlay sanity |
| HG-GOV registration recognized | authority-cohesion lane green |
| Full ~50-lane suite passes on governance HEAD | phase exit |

## Halt Conditions

- HG-GOV registration refused by authority-cohesion lane → fix `/specs/hyperscaler-gates.json` schema.
- Runbook freshness lane refuses (a runbook older than its review-cadence) → update review date.
- Phase exit gate fails → halt; resolve; retry.

## Phase Exit

When this IP merges:
- `oya gate validate authority-cohesion` exit 0 with HG-GOV green.
- `oya gate validate industry-best-practice-conformance --microservice governance` exit 0.
- `oya gate validate per-microservice-layout --microservice governance` exit 0.
- All 15 IPs merged.
- `evidence/audits/industry-best-practice-conformance/2026-Q2.json` written.
- PHASE-01 exit gate per `PHASE-01-CI-FITNESS-CONSOLIDATION.md` satisfied.

## References

- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0131 §"iac/" mandatory subfolder.
- `microservices/governance/runbooks/*.md` (6 runbooks).
- `microservices/governance/multi-region.md` §"Topology per pack".
- `microservices/governance/cost-budget.md` (sizing references for Helm values).
