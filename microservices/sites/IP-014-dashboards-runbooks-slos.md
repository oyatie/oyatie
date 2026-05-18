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

## ChangeSet metadata

```yaml
changeset_id: CS-SITES-IP-014-dashboards-runbooks-slos
depends_on_changesets: [CS-SITES-IP-003-site-and-page-bcs]
parallel_safe_with_changesets: [CS-SITES-IP-012-policy-dpia-threat-model, CS-SITES-IP-013-contracts-and-capabilities]
enables: [CS-SITES-IP-015-hg-sites-maturity-claim]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | 9 OpenSLO manifests authored at `slos/*.openslo.yaml` and lint-clean | `oya-dev-cli gate validate openslo-lint --microservice sites` |
| AC-02 | 3 Grafana dashboards present + dashboard-completeness gate green | `oya-dev-cli gate validate dashboard-completeness --microservice sites` |
| AC-03 | 7 runbooks authored + each cross-references an SLO + alert + rollback step | `oya-dev-cli gate validate runbook-completeness --microservice sites` |
| AC-04 | Burn-rate alerts (14.4× 1h + 6× 6h) emit Prometheus rules | manual verification + `promtool check rules` |

## Build Sequence

1. Author OpenSLO YAML at `slos/page-render.openslo.yaml`, `publish-pipeline.openslo.yaml`, etc. (9 files).
2. Author Grafana dashboards (publish-and-cdn, seo-and-traffic, editor-experience) at `dashboards/*.json`.
3. Author runbooks at `runbooks/*.md` (7 files).
4. Author Prometheus rule manifests at `iac/helm/sites/templates/prometheusrule.yaml`.
5. Run all four gates above.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-sites NFR | Availability + SLO §; 99.99% page-render |
| PRD-sites AC | AC-14 (SLO publishing) |
| ADR | ADR-0130 (agentic SLO-gated promotion) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| SLO definition decoupled from alert rules | OpenSLO-to-Prometheus generator + lane refuses drift |
| Runbook references stale rollback step | `runbook-completeness` gate verifies referenced commands exist |
| Dashboard panel references metric that no longer exists | Dashboard-completeness gate verifies metric presence |

## References

- OpenSLO v1 specification (`openslo.com`).
- Grafana Dashboard JSON schema (`grafana.com/docs/grafana/latest/dashboards/build-dashboards`).
- Google SRE Workbook — Implementing SLOs (Beyer et al., O'Reilly 2018).
- Prometheus alerting rules (`prometheus.io/docs/prometheus/latest/configuration/alerting_rules`).
- ADR-0130 (agentic SLO-gated promotion).
- agent-skills shipping-and-launch SKILL.md (runbook authoring).
