---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

# IP-001: IaC bootstrap — Helm + Kustomize for Radicale + Postgres + Redis + SabreDAV (us-healthcare)

## Intent

Author Helm + Kustomize manifests for the calendar µservice substrate.
Two CalDAV backends supported per ADR-CAL-0001 (`radicale` primary +
`sabredav` for `pack-us-healthcare`); Postgres 16 LTS for event store
(RLS per-tenant per ADR-0117); Redis 7.4 LTS for the availability-
resolver cache; OpenBao for per-tenant DEK envelope encryption.
CronJob for the tzdb refresh worker per ADR-CAL-0004. Pack-aware
overlays for 11 packs.

## ChangeSet boundary

10 Helm template files + Kustomize base + per-pack overlay (pack-kr
first; eu/us/jp/sg/au/in/br/ae/ksa/us-healthcare follow). No Rust
code; pure IaC + values. All secrets via `${openbao:secret/calendar/...}`
SecretReferences.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/iac/helm/Chart.yaml` | created in this ChangeSet | dependencies: radicale 3.2.3, sabredav 4.6.0, postgres 16.4.0, redis 7.4.0 |
| `microservices/calendar/iac/helm/values.yaml` | created | per-BC replica sizing; CalDAV backend gating; OpenBao SecretReferences |
| `microservices/calendar/iac/helm/templates/deployment.yaml` | created | per-BC Deployment (6 BCs) |
| `microservices/calendar/iac/helm/templates/service.yaml` | created | per-BC Service |
| `microservices/calendar/iac/helm/templates/hpa.yaml` | created | per-BC HPA (CPU 70%; min 3 max 100) |
| `microservices/calendar/iac/helm/templates/pdb.yaml` | created | PodDisruptionBudget min-available 50% |
| `microservices/calendar/iac/helm/templates/networkpolicy.yaml` | created | mesh-only ingress; egress to OpenBao + Postgres + Redis + mail µservice + IANA tzdb |
| `microservices/calendar/iac/helm/templates/servicemonitor.yaml` | created | Prometheus scrape config |
| `microservices/calendar/iac/helm/templates/prometheusrule.yaml` | created | per-BC fast-burn + slow-burn alert rules |
| `microservices/calendar/iac/helm/templates/cronjob.yaml` | created | tzdb refresh worker (ADR-CAL-0004) |
| `microservices/calendar/iac/kustomize/base/kustomization.yaml` | created | shared base |
| `microservices/calendar/iac/kustomize/base/namespace.yaml` | created | calendar namespace |
| `microservices/calendar/iac/kustomize/base/serviceaccount.yaml` | created | oya-calendar SA + OpenBao role binding |
| `microservices/calendar/iac/kustomize/overlays/pack-kr/kustomization.yaml` | created | initial active pack |
| `microservices/calendar/iac/kustomize/overlays/pack-eu/kustomization.yaml` | successor-IP | eu pack |
| `microservices/calendar/iac/kustomize/overlays/pack-us/kustomization.yaml` | successor-IP | us pack |
| `microservices/calendar/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml` | successor-IP | HIPAA pack — enables SabreDAV backend per ADR-CAL-0001 |
| (additional packs: jp, sg, au, in, br, ae, ksa) | successor-IP | per-pack overlays |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/calendar/iac/helm
kubectl --dry-run=client apply -k microservices/calendar/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice calendar
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- helm lint + helm-test per chart against kind/k3d cluster.
- E2E smoke: spin kind cluster; apply pack-kr overlay; verify all 6
  BC deployments + Radicale + Postgres + Redis reach Ready within
  10 min.
- CalDAV smoke: PROPFIND against the Radicale Service; expect
  RFC-4791 conformant XML response.

## Halt Conditions

- Upstream chart version drifts past LTS pin — escalate per
  `docs/standards/observability-slo.md`.
- OpenBao secret-reference resolution fails — block.
- Helm chart fails kubectl-dry-run — root-cause; do not mask.

## Next IP

[`IP-002-event-store-kernel.md`](IP-002-event-store-kernel.md)

## References

- ADR-0117 (data residency); ADR-0131 (per-µservice flat layout); ADR-0133.
- ADR-CAL-0001 (CalDAV backend selection); ADR-CAL-0004 (tzdb refresh).
- RFC 4791 — CalDAV.
- Radicale — `radicale.org`; SabreDAV — `sabre.io/dav/`.
- Postgres CloudNativePG operator — `cloudnative-pg.io`.
- Redis cluster mode — `redis.io/docs/management/scaling/`.
