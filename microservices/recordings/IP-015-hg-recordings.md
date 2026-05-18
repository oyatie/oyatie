---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-015-hg-recordings
status: pending
owner: axis-recordings + ops-sre-reliability
acceptance_lanes: [authority-cohesion]
---

# IP-015: HG-RECORDINGS authority-cohesion gate registration

## Intent

Register HG-RECORDINGS in the governance gate catalog per ADR-0123 +
ADR-0133 + ADR-0130 SLO-gated promotion. Wires the 10 OpenSLO objectives +
2 load-bearing zero-tolerance correctness invariants to the promotion
eligibility ledger.

## ChangeSet boundary

- `registry/artifact-capabilities-registry.json`: add HG-RECORDINGS entry
  with the 10 SLO refs + 2 load-bearing correctness refs.
- `crates/oya-foundry-gate-catalog-domain`: register HG-RECORDINGS in the
  gate catalog.
- `microservices/recordings/iac/helm/recordings/templates/prometheusrule.yaml`:
  emit the burn-rate alerts wired to the promotion ledger.
- Branch-protection: HG-RECORDINGS becomes a required status check before
  recordings releases land on dev.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice recordings
# Should accept at p99 SLOs sustained 30d before HG-RECORDINGS goes green
```

## Phase

Last IP in PHASE-01; HG-RECORDINGS green is the phase exit condition.

## References

- ADR-0123 (HG gates), ADR-0130 (SLO-gated promotion), ADR-0133 (industry
  best-practice conformance).
