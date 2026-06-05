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

# IP-001: IaC bootstrap for Calendar

## A. Problem
Calendar cannot claim first-party scheduling parity while its runtime substrate is an implied chart shell. The PRD requires event writes, CalDAV, free/busy, room booking, invitation fanout, and tzdb refresh to run with region-pinned data, OpenBao-backed secrets, and SLO evidence.

## B. Approach
Promote the existing Helm and Kustomize assets into the deployable baseline for the six calendar bounded contexts plus Radicale, SabreDAV for the US healthcare overlay, Postgres, Valkey, and the tzdb CronJob. Keep all runtime coupling outside the domain crates and use NetworkPolicy plus SecretReference patterns to make mail, audit-chain, OpenBao, Postgres, Valkey, and IANA tzdb access explicit.

## C. Deliverables
| Artifact | Role |
|---|---|
| `microservices/calendar/iac/helm/Chart.yaml` | Calendar chart and dependency declaration for Radicale, SabreDAV, Postgres, and Valkey. |
| `microservices/calendar/iac/helm/values.yaml` | Per-BC replica sizing, CalDAV backend choice, and OpenBao secret references. |
| `microservices/calendar/iac/helm/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule,cronjob}.yaml` | Runtime, availability, telemetry, alerting, and tzdb refresh primitives. |
| `microservices/calendar/iac/kustomize/base/` | Namespace and service-account base. |
| `microservices/calendar/iac/kustomize/overlays/pack-kr/` and `pack-us-healthcare/` | First pack overlays already present in this repo. |

## D. Ordered implementation steps
1. Lint the chart and confirm every declared workload maps to a PRD bounded context.
2. Verify `values.yaml` pins Postgres, Valkey, Radicale, SabreDAV, and worker images rather than floating tags.
3. Check the NetworkPolicy allows only mesh ingress and named egress to OpenBao, Postgres, Valkey, mail, audit-chain, observability, and tzdb sources.
4. Dry-run the `pack-kr` and `pack-us-healthcare` overlays.
5. Confirm `servicemonitor.yaml` and `prometheusrule.yaml` cover the OpenSLO files under `slos/`.
6. Confirm the tzdb CronJob binds to `catalog/oya-calendar-tzdb-refresh-worker.yaml` and the ADR-CAL-0004 refresh policy.
7. Record promotion evidence in the changeset before opening the PR.

## E. Acceptance
- `helm lint microservices/calendar/iac/helm` passes.
- `kubectl --dry-run=client apply -k microservices/calendar/iac/kustomize/overlays/pack-kr` passes.
- `kubectl --dry-run=client apply -k microservices/calendar/iac/kustomize/overlays/pack-us-healthcare` passes.
- `buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice calendar` passes.
- SLO bindings resolve for `slos/caldav-availability.openslo.yaml`, `slos/freebusy-query-latency.openslo.yaml`, and `slos/tzdb-staleness-bound.openslo.yaml`.

## F. Evidence
- PRD runtime needs: `microservices/calendar/PRD.md`.
- Architecture anchors: `microservices/calendar/ARCHITECTURE.md`.
- IaC evidence: `microservices/calendar/iac/helm/` and `microservices/calendar/iac/kustomize/`.
- Policy evidence: `microservices/calendar/policy/data-residency.md` and `microservices/calendar/policy/event-isolation.md`.
- Operational evidence: `microservices/calendar/runbooks/timezone-db-refresh.md`, `runbooks/caldav-sync-loop.md`, and `runbooks/tzdb-rollback.md`.

## G. Counterpart comparison
Counterpart pressure comes from Google Calendar, Microsoft Outlook Calendar, Apple Calendar, Fastmail, Proton Calendar, Cal.com, Calendly, Doodle, and Naver Works. Google and Outlook set the enterprise deployment and sync expectation; Fastmail and Apple set CalDAV expectations; Calendly and Cal.com set scheduling-link expectations. Calendar's advantage is not another generic chart: it is a deployable, policy-bound substrate with pack overlays, OpenSLO promotion gates, per-tenant tzdb pinning, and audit evidence competitors do not expose as first-class tenant controls.

## H. Foundation delivery expansion
- Deliverable detail: render one Deployment per calendar bounded context, not a shared catch-all pod.
- Deliverable detail: bind Radicale and SabreDAV behind adapter-specific Service names so pack overlays can select one backend.
- Deliverable detail: include OpenBao SecretReference names for database, CalDAV, Valkey, and tzdb fetch credentials.
- Deliverable detail: set PDB and HPA defaults for event-store, availability, invitation, and tzdb worker workloads.
- Deliverable detail: include NetworkPolicy egress labels for mail, audit-chain, observability, OpenBao, Postgres, Valkey, and tzdb source.
- Deliverable detail: map ServiceMonitor labels to the SLO names under `slos/`.
- Deliverable detail: ensure chart values expose pack overlay switches for KR and US healthcare without changing template paths.
- Deliverable detail: keep chart rendering independent of Google-style or Slack-style external calendar integration assumptions.

## I. Acceptance expansion
- Acceptance detail: chart lint must prove all template references resolve with default values.
- Acceptance detail: dry-run output must include named workloads for event-store, recurrence, availability, rooms, invitations, CalDAV, and tzdb.
- Acceptance detail: network policy tests must deny generic internet egress except explicit tzdb fetch endpoints.
- Acceptance detail: secret binding checks must fail closed when OpenBao references are absent.
- Acceptance detail: SLO binding checks must connect rendered Prometheus rules to `freebusy`, `caldav`, and `tzdb` SLO ids.
- Acceptance detail: pack overlay dry-runs must not mutate tenant/context policy defaults.
- Acceptance detail: branch promotion must attach rendered manifest evidence, not just source YAML paths.
- Acceptance detail: Slack collaboration-calendar interop is pressure for notification and presence egress, but not a reason to bypass mail/audit ports.

## J. Evidence expansion
- Evidence detail: capture `helm lint` output with chart version and dependency versions.
- Evidence detail: capture `kubectl --dry-run=client apply -k` output for both pack overlays.
- Evidence detail: capture a rendered `NetworkPolicy` excerpt showing explicit allowed destinations.
- Evidence detail: capture a rendered `CronJob` excerpt proving tzdb worker scheduling and image pinning.
- Evidence detail: capture ServiceMonitor and PrometheusRule names for each SLO-bound workload.
- Evidence detail: cite `ADR-CAL-0001` for CalDAV backend selection when SabreDAV/Radicale choices differ by pack.
- Evidence detail: cite the Slack comparison only as enterprise collaboration pressure, not as a calendar protocol substitute.
