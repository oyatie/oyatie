---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

# IP-001: Social IaC bootstrap

## A. Problem
Social cannot support feed, posting, moderation, search, federation, media, minor protection, and DSA evidence if its runtime footprint remains a generic Helm shell.

## B. Approach
Promote the existing Helm/Kustomize/IaC artifacts as the deployable substrate for the cataloged social workloads: app, user profile, follow graph, post composition, feed timeline, search, content moderation, federation, CSAM classifier, DSA transparency, profile verification, and sock-puppet detection.

## C. Deliverables
| Artifact | Role |
|---|---|
| `iac/helm/social/Chart.yaml` and `values.yaml` | Social runtime chart and pinned dependency source. |
| `iac/helm/social/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | Kubernetes runtime, traffic, resilience, telemetry, and alert primitives. |
| `iac/kustomize/base/` and `iac/kustomize/overlays/pack-kr/`, `pack-us-healthcare/` | Existing pack overlays. |
| `iac/{edge-waf,ech-config,openbao-policy,pqc-cert,secret-bindings}.yaml` | Existing social edge/security bindings. |

## D. Ordered implementation steps
1. Lint the social chart and confirm every Deployment maps to a cataloged crate or explicit worker.
2. Validate image pins, runtime classes, resource requests, and HPA thresholds.
3. Confirm NetworkPolicy allows only required edge, OpenBao, Postgres, Valkey, Meilisearch, scanner, audit, and observability paths.
4. Dry-run KR and US healthcare overlays.
5. Verify Prometheus rules cover feed, post, moderation, CSAM, minor-protection, and notification SLOs.
6. Confirm edge WAF and ECH/PQC bindings do not bypass Cedar or app policy.
7. Capture dry-run and chart evidence before PR promotion.

## E. Acceptance
- `helm lint microservices/social/iac/helm/social` passes.
- `kubectl --dry-run=client apply -k microservices/social/iac/kustomize/overlays/pack-kr` passes.
- `kubectl --dry-run=client apply -k microservices/social/iac/kustomize/overlays/pack-us-healthcare` passes.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice social` passes.
- SLO paths under `microservices/social/slos/` resolve from the PrometheusRule template.

## F. Evidence
- Product and runtime authority: `PRD.md`, `ARCHITECTURE.md`, `manifest.json`.
- IaC: `iac/helm/social/`, `iac/kustomize/`, `iac/edge-waf.yaml`, `iac/openbao-policy.yaml`.
- Operational closure: `runbooks/content-moderation-rollback.md`, `runbooks/feed-cache-rebuild.md`, `runbooks/csam-detect-and-ncmec-report.md`.

## G. Counterpart comparison
Counterparts include X, Bluesky, Mastodon, Threads, Instagram, TikTok, Snapchat, LinkedIn, and Reddit. Their public systems prove scale and feature pressure; this IP gives Oyatie a tenant-deployable substrate with OpenSLO gates, Cedar policy, pack overlays, and audit evidence instead of opaque vendor operations.

## H. Foundation delivery expansion
- Deliverable detail: chart renders separate workloads for profile, graph, posting, feed, search, moderation, federation, CSAM, DSA, verification, and sock-puppet detection.
- Deliverable detail: values pin Postgres, Valkey, Meilisearch, scanner, media, and worker images.
- Deliverable detail: edge WAF, ECH, PQC, OpenBao, and secret-binding files are deployment inputs, not optional notes.
- Deliverable detail: NetworkPolicy allows only explicit paths to edge, OpenBao, stores, scanners, audit, observability, and federation egress.
- Deliverable detail: Prometheus rules bind feed, moderation, abuse, CSAM, federation, and minor-protection alerts.
- Deliverable detail: pack overlays keep KR and US healthcare behavior separate from default runtime values.
- Deliverable detail: deploy evidence includes rendered manifests and not just source chart paths.
- Deliverable detail: Slack community/channel moderation is counterpart pressure for deployable trust-and-safety controls.

## I. Acceptance expansion
- Acceptance detail: chart lint must prove all template references resolve with default social values.
- Acceptance detail: dry-run output must include each cataloged social workload and worker.
- Acceptance detail: image pin checks must reject floating tags for app, scanners, media processors, and workers.
- Acceptance detail: NetworkPolicy checks must deny generic egress and preserve explicit federation controls.
- Acceptance detail: Prometheus rules must reference known SLO ids and runbooks.
- Acceptance detail: pack overlay dry-runs must not loosen minor-protection or DSA policy defaults.
- Acceptance detail: OpenBao secret references must fail closed when missing.
- Acceptance detail: Slack, Reddit, and Discord-style community pressure must remain moderation/runtime evidence, not a new dependency.

## J. Evidence expansion
- Evidence detail: capture `helm lint microservices/social/iac/helm/social`.
- Evidence detail: capture `kubectl --dry-run=client apply -k` output for KR and US healthcare overlays.
- Evidence detail: capture rendered NetworkPolicy excerpts for edge, scanners, stores, and federation.
- Evidence detail: capture PrometheusRule names tied to moderation, feed, abuse, and minor-protection SLOs.
- Evidence detail: cite `iac/edge-waf.yaml` and `iac/openbao-policy.yaml` for edge/security binding.
- Evidence detail: cite `dashboards/moderation-and-safety.json` for runtime trust-and-safety evidence.
- Evidence detail: cite Slack as community/channel moderation deployment pressure alongside X and Reddit.
