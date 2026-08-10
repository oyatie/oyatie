---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
impl_plan_id: IP-013-canary-cohort-and-rollback-wiring
status: pending
owner: ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: Canary cohort wiring + production-tier rollback for tenancy

## Intent

Wire tenancy canary cohort weighting (per ADR-0139 §canary_cohort_weighting; 1→10→50→100 %); production-tier auto-rollback wire-up via observability gate. Tenancy is the most safety-critical µservice in the catalog; rollback discipline is correspondingly strict.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/tenancy/iac/kustomize/base/istio/virtualservice.yaml` | create — per-pack VirtualService with canary cohort |
| `microservices/tenancy/iac/kustomize/base/istio/destinationrule.yaml` | create — subsets for primary + canary |
| `tenancy/runbooks/rls-drift-recovery.md` | already authored | references rollback path |

## Code Shape

```yaml
# virtualservice.yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: tenancy-tenant-lifecycle-rest
  namespace: tenancy
spec:
  hosts: [tenancy-tenant-lifecycle-rest]
  http:
  - route:
    - destination:
        host: tenancy-tenant-lifecycle-rest
        subset: primary
      weight: 99  # ramped up from 100 during canary phase
    - destination:
        host: tenancy-tenant-lifecycle-rest
        subset: canary
      weight: 1   # initial canary
```

## Acceptance Gates

```bash
helm lint microservices/tenancy/iac/kustomize/base/istio
kubectl --dry-run=client apply -f microservices/tenancy/iac/kustomize/base/istio/
```

## Test Plan

- E2E canary ramp drill (against observability gate): tenancy canary 1→10→50→100; verify burn-rate clean signals at each step; verify abort-on-burn drains canary back to 0.
- Production rollback drill: induce synthetic burn at production tenancy; verify auto-rollback within 60s; cross-tenant probe verifies isolation preserved through rollback.

## Halt Conditions

- Canary weight changes faster than `min_duration_seconds` — bug; fix.
- Istio VirtualService apply fails — engage cloud-k8s.
- Rollback verification fails cross-tenant probe — declare Sev-1; do not promote tenancy further.

## Next IP

[`IP-014-tests-load-drills-observability-slos.md`](IP-014-tests-load-drills-observability-slos.md)
