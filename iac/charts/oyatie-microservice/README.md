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

## What is shared, and what is deliberately not

Shared here: `configmap.yaml`, `service.yaml`, `deployment.yaml`.

**`cedar.yaml` is NOT shared, and must never be.** It carries each service's
authorization policy. Render-equivalence testing found that sharing one Cedar
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
  `observability`, `oya-ci-webhook-gateway`, `oya-community`, `secrets-kms`.

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

Current state: **80 render, 1 skipped, 0 failing.**
