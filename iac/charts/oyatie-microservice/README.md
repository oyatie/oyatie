# oyatie-microservice

The single chart for Oyatie backbone microservices. A service supplies its own
`values.yaml`; it does not carry a copy of these templates.

## Why this exists

Measured on `origin/dev` @ `77835a1b3`, across 81 per-service charts:

| template | files | distinct contents | duplication |
|---|---:|---:|---:|
| `configmap.yaml` | 81 | 4 | 96% |
| `service.yaml` | 81 | 23 | 77% |
| `deployment.yaml` | 81 | 44 | 64% |

78 of the 81 `configmap.yaml` files were byte-identical. These are already
parameterised Helm templates — they were copied per service rather than shared.

## How a service uses this chart

A service supplies **only** `values.yaml`. It carries no `Chart.yaml` and no
`templates/`:

    helm template <name> iac/charts/oyatie-microservice \
      -f <service>/iac/k8s/helm/values.yaml

71 of 81 services were collapsed onto this chart after render comparison proved
their output identical. 11 keep their own chart because their rendered output
genuinely differs; they are frozen in
`ci/facade/helm-chart-shape/bespoke-charts.json` and the split is enforced
shrink-only — a new service must use the shared chart, and reconciling a bespoke
one means deleting its entry in the same change.

**This changes the invocation contract.** Anything that previously pointed at a
per-service chart directory must now point at this chart and pass the service's
values file. Note the ArgoCD ApplicationSets were ALREADY broken before this
change — all four `chartPath` values referenced `microservices/…`, a tree
deprecated months ago and absent from `dev`, and those manifests self-annotate
`non-claim: "static-promotion-manifest-only-no-live-sync"`. They are left
untouched here rather than silently repointed, because correcting them is a
deploy-enablement decision, not a refactor.

## What is shared, and what is deliberately not

Shared here: `configmap.yaml`, `service.yaml`, `deployment.yaml`, and
`cedar.yaml` — but Cedar only as a **renderer**, never as shared policy.

**Cedar POLICY is never shared**, though the ConfigMap wrapper now is. Each
service's policy text lives in its own `values.yaml` under `cedar.policy`, and
the shared template renders it with `tpl` so existing `{{ .Values… }}`
references still resolve — preserving each service's authored semantics exactly. Render-equivalence testing found that sharing one Cedar
template would have replaced deliberate fail-closed default-deny policies with
a blanket `permit(principal, action, resource)` for 24 services. `app/calendar`
is the clearest case: it ships *no* permit statements on purpose, and its own
comment cites the `cedar-deploy-parity` gate (ADR-0608) forbidding an
over-broad tenant-class blanket permit. A duplication statistic said those
files were interchangeable; they are not. Authorization policy is per-service
data, and it stays per-service.

## Equivalence evidence

Each service chart was rendered before and after, with a fixture digest
injected (`--set image.digest=sha256:1111…`, because the ADR-0181 guard refuses
to render without a real signed digest):

- **70 of 81** render byte-identically on the non-Cedar templates, ignoring only
  Helm's `# Source:` provenance comment.
- **11** have genuine deployment differences and keep bespoke charts until each
  is reviewed individually: `ci-controller`, `comms-mail`, `comms-messenger`,
  `data-analytics`, `iam-cloud-iam`, `iam-identity`, `intelligence`,
  `observability`, `ci-webhook-gateway`, `community`, `secrets-kms`.

## What the render gate surfaced, and what came of it

Introducing `helm-render-smoke` found four charts that would not render. Only
one was a defect:

- **`intelligence` kept a `BUCK` file inside `templates/`** — a real bug. Helm
  renders everything under `templates/`, so a Buck build file was parsed as a
  manifest and the chart could not render at all. The BUCK file itself is
  deliberate: it `export_file()`s `deployment.yaml`/`externalsecret.yaml` as
  declared resources so the Buck action and the ADR-0716 Cargo merge path bind
  the same bytes, and it carries the ADR-0541 corpus target. Fixed with a
  `.helmignore` so the wiring stays and Helm stops reading it.
- **`iam-cloud-iam` and `secrets-kms` were not broken.** They declare a second
  image (`svidOperator`, `operator`) and, in one case, a required `cellId` with
  no default. The gate was supplying only `image.digest`. Fixed in the gate.
- **`observability` is skipped, not failed.** Its `Chart.yaml` declares remote
  dependencies (loki, tempo from grafana.github.io), so it cannot be rendered
  offline. Counting that as a failure would be dishonest — nothing is broken.

Current state: **80 render, 1 skipped, 0 failing** — measured locally with helm.

## Why there is no `helm template` CI job

A render job would be the stronger check, but it needs the `helm` binary and
therefore inline shell in workflow YAML — which this repository retires on a
shrink-only ratchet (`rust_first_automation_unbaselined_workflow_inline_shell`:
"productize it as a Rust/Buck2 step"). There is also no precedent for invoking
helm in CI at all. Adding it would have meant adding the debt class the repo is
actively removing.

`ci/facade/helm-chart-shape` is the productized floor instead: pure Rust, no
shell, running inside the required `cargo test --workspace` job. It catches a
non-manifest file under `templates/` — the class that actually broke
`intelligence`. It does **not** catch template syntax errors, missing required
values, or unvendored dependencies; a full render would. That remaining coverage
needs a Rust renderer and is not claimed here.
