---
doc_class: ServiceREADME
microservice: foundry-eval
status: Accepted
owner_team: axis-foundry
date: 2026-05-17
doc_status: published
---

# foundry-eval

oyatie's eval-harness substrate. Per ADR-0024 + ADR-0026 + ADR-0131.

## What this µservice does

- Runs per-capability eval sets (baseline + adversarial + linguistic + replay cohorts) against provider routes.
- Computes pass/fail aggregates + per-cohort breakdowns.
- Gates capability publish (refuse on missing eval-set, adversarial fail, linguistic minima fail, stale run).
- Runs nightly drift detection.
- Runs A/B against routing-preference changes.
- Replays past production traces with ≤ 100ms divergence tolerance.
- Emits InHouseCutoverEligible verdicts to foundry-providers per ADR-0026.
- Emits EU AI Act §15 (accuracy + robustness + cybersecurity) + §17 (logging) evidence on every eval-run.
- Shreds per-subject DEKs on DSR cascade per ADR-0024 §"Resolved 1" cryptographic shredding.

## Bounded contexts

- `eval-set-registry`: per-capability eval-set index + Cosign + Rekor verify.
- `eval-runner`: case execution; nightly orchestrator; publish-gate.
- `parity-analyzer`: two-run delta + in-house cutover decisioning.
- `replay-engine`: sample + replay + divergence detection.
- `baseline-output-store`: Cosign-signed per-subject-keyed baseline outputs.

## Entry points

| Surface | Path |
|---|---|
| PRD | `PRD.md` |
| Phase plan | `PHASE-01-EVAL-HARNESS-SUBSTRATE.md` |
| ADRs | ADR-0024, ADR-0026 (root `docs/decisions/`) |
| REST API | `contracts/openapi/eval-runner.yaml` |
| gRPC | `contracts/proto/eval_runner.proto` |
| Events | `contracts/asyncapi/eval-events.yaml` |
| Threat model | `threat-model.md` |
| DPIA + FRIA | `dpia.md` |
| Compliance mapping | `compliance.md` |
| Runbooks | `runbooks/*.md` |
| Dashboards | `dashboards/*.json` |
| Capacity model | `capacity-model.md` |
| Cost budget | `cost-budget.md` |
| Multi-region | `multi-region.md` |
| Incident response | `incident-response.md` |
| Competitor parity | `competitor-parity-matrix.md` |
| SDK plan | `sdk-plan.md` |
| Backfill + replay | `backfill-replay.md` |

## Quick-start (developer)

```bash
# Build all foundry-eval crates
cargo build --workspace -p 'intelligence-eval-*'

# Run kernel tests
cargo nextest run -p intelligence-eval-kernel

# Validate layout
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry-eval

# Deploy Layer-A (kind cluster)
helm install gpu-runner-pool microservices/intelligence-eval/iac/helm/gpu-runner-pool/
helm install postgres microservices/intelligence-eval/iac/helm/postgres/
helm install clickhouse microservices/intelligence-eval/iac/helm/clickhouse/
helm install baseline-store microservices/intelligence-eval/iac/helm/baseline-store/
```

## Status

M01-P01: in progress. Per `PHASE-01-EVAL-HARNESS-SUBSTRATE.md`.

## Owners

- Primary: axis-foundry
- On-call: per `runbooks/oncall-rotation.md` (cross-references observability)
- Security: ops-security
- Privacy: council-privacy

## Related ADRs

- ADR-0024 (foundry eval harness + replay; design)
- ADR-0026 (in-house AI model substrate roadmap; cutover gate)
- ADR-0056 (BNF v4.1)
- ADR-0105 (13-layer enum)
- ADR-0106 (application → usecase rename)
- ADR-0139 (agentic SLO-gated promotion; inherited)
- ADR-0131 (per-microservice flat layout)
- ADR-0132 (product-platform-and-bundle dissolution; foundry split)
- ADR-0133 (industry best-practice conformance program; HG-FE)
- ADR-0140 (retired per ADR-0145) (Cedar policy enforcement)
