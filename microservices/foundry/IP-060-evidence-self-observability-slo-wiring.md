---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-015-self-observability-slo-wiring
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence + axis-observability
acceptance_lanes: [agentic-slo-gated-promotion, hyperscaler-maturity-claims]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: Self-observability + HG-FOUNDRY-EVIDENCE registration

## Intent

Wire foundry-evidence's SLI emission into the `observability` µservice substrate; register `HG-FOUNDRY-EVIDENCE` gate in `/specs/hyperscaler-gates.json`; close the ADR-0130 SLO-gated-promotion loop for this µservice.

## ChangeSet boundary

Cross-cutting: SLI emit code paths, hyperscaler-gates registry update, branch-protection update, claim-matrix update.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-*/src/observability.rs` | create or edit | per-crate SLI emission (use OpenTelemetry Prometheus exporter) |
| `/specs/hyperscaler-gates.json` | edit | register `HG-FOUNDRY-EVIDENCE` per ADR-0123 with declared targets from `PRD.md` NFR table |
| `.github/branch-protection.yaml` | edit | add `oya-foundry-evidence-self-verification` lane to `dev` required_status_checks |
| `microservices/foundry/contracts/openapi/foundry-evidence.yaml` | edit | wire `/transparency/claim-matrix` endpoint to load from a generated registry + claim assertions |
| `microservices/foundry/capabilities/eval/claim-matrix.json` | create | machine-readable claim-matrix sourced for the public-read endpoint |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate self-slo-coverage --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate agentic-slo-gated-promotion --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice foundry-evidence
```

## Halt Conditions

- Any declared NFR target in `PRD.md` lacks a matching SLI + Mimir series — block.
- Claim-matrix declares "asserted" for a claim that lacks a CI lane — block.
- HG gate target violated in load-drill — block.

## Phase exit

This IP closes phase `P01-foundry-evidence-frontend`. Per `PHASE-01-FOUNDRY-EVIDENCE-FRONTEND.md` `exit_gate`, the full acceptance-gates list must pass.

## Next phase

`P02-foundry-evidence-vertical-overlays` (subsequent-to-M01-completion): pack-specific evidence-pack schema extensions.

## References

- ADR-0130 (agentic SLO-gated promotion).
- ADR-0123 (hyperscaler-grade gates).
- ADR-0133 (honest claims).
