---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-014-observability-slo
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + axis-observability
acceptance_lanes: [openslo-schema-validate, grafana-dashboard-lint, oya-governance-promotion-readiness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: OpenSLO manifests + Grafana dashboards + per-pack runbooks wiring

## Intent

Wire social into the observability µservice's agentic SLO-gated promotion
substrate per ADR-0139. Author all 8 OpenSLO manifests; cross-link 3 Grafana
dashboards; register burn-rate alerts via PrometheusRule; configure per-pack
runbook URLs.

## ChangeSet boundary

OpenSLO + Grafana + PrometheusRule + runbook hookups.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/social/slos/{feed-render-latency,profile-render-availability,post-create-latency,follow-action-latency,notification-fanout-latency,search-people-latency,moderation-classifier-latency,content-policy-enforcement-correctness}.openslo.yaml` | exists |
| `microservices/social/dashboards/{feed-experience,moderation-and-safety,federation-and-cross-context}.json` | exists |
| `microservices/social/iac/helm/social/templates/prometheusrule.yaml` | exists — burn-rate alerts |
| `microservices/social/iac/helm/social/templates/servicemonitor.yaml` | exists |

## Acceptance Gates

```bash
oya gate validate openslo-schema --microservice social
oya gate validate promotion-readiness --microservice social
grafana-cli plugins validate microservices/social/dashboards/
```

## Test Plan

- Synthetic load against pack-kr cluster; burn-rate alerts fire per SLO threshold.
- Dashboards render in Grafana with per-pack templating var.
- ADR-0139 promotion-readiness gate exits 0 for social.

## Halt Conditions

- OpenSLO schema validation fails — fix schema.
- Burn-rate alert thresholds inconsistent with capacity-model — reconcile.

## Next IP

[`IP-015-hg-social-registration-and-branch-protection.md`](IP-015-hg-social-registration-and-branch-protection.md)

## References

- ADR-0139 (SLO-gated promotion).
- `microservices/observability/PRD.md`.
