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

## Known defects this surfaced

- `intelligence` keeps a **`BUCK` file inside `templates/`**. Helm renders
  everything under `templates/`, so it is parsed as a manifest and fails. No CI
  job renders any chart, which is why this was never caught.
- `observability` declares chart dependencies that are not vendored.
- `iam-cloud-iam` needs a second digest (`svidOperator.image.digest`).
