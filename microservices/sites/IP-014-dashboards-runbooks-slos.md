---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-014-dashboards-runbooks-slos
status: pending
execution_unit: ChangeSet
owner: axis-sites + axis-observability + ops-sre-reliability
acceptance_lanes: [grafana-dashboard-lint, openslo-lint, runbook-completeness]
---

# IP-014: dashboards + runbooks + OpenSLO manifests

## Intent

Author 3 Grafana dashboards (publish-and-cdn, seo-and-traffic, editor-experience), 7 runbooks (publish-pipeline-rollback, acme-cert-renewal-failure, cdn-cache-purge-cascade, custom-domain-dns-drift, asset-optimization-degraded, page-export-corruption, ai-page-build-rollback), and 9 OpenSLO manifests.

## ChangeSet boundary

3 dashboard JSON files + 7 runbook markdown files + 9 OpenSLO YAML files.

## Acceptance Gates

```bash
oya-dev-cli gate validate dashboard-completeness --microservice sites
oya-dev-cli gate validate runbook-completeness --microservice sites
oya-dev-cli gate validate openslo-lint --microservice sites
```

## References

- OpenSLO v1.
- Grafana Dashboard JSON schema.
- agent-skills shipping-and-launch SKILL.md (runbook authoring).
