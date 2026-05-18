---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-015-hg-recordings
status: pending
owner: axis-recordings + ops-sre-reliability
acceptance_lanes: [authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-015-hg-recordings
depends_on_changesets: [CS-RECORDINGS-IP-008-redaction-bc, CS-RECORDINGS-IP-010-retention-legal-hold-bcs, CS-RECORDINGS-IP-012-export-ediscovery-bcs, CS-RECORDINGS-IP-014-strangler-migration-adapter]
parallel_safe_with_changesets: []
enables: []
acceptance_status: ga
phase_exit: true
load_bearing: true
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | HG-RECORDINGS registered in `registry/artifact-capabilities-registry.json` with 10 SLO refs + 2 load-bearing correctness refs | `cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice recordings` |
| AC-02 | All 10 OpenSLO objectives green over 30d in dev cluster | ADR-0130 ledger query |
| AC-03 | Retention-policy-correctness + legal-hold-chain-of-custody-correctness both green | governance lanes |
| AC-04 | Prometheus burn-rate alerts wired to promotion ledger via Helm template | `kubectl apply --dry-run` + `promtool check rules` |
| AC-05 | Branch-protection adds HG-RECORDINGS as required check on `dev` | `gh api` query of branch protection rules |

## Build Sequence

1. Append HG-RECORDINGS entry to `registry/artifact-capabilities-registry.json`.
2. Register in `crates/oya-foundry-gate-catalog-domain`.
3. Author `microservices/recordings/iac/helm/recordings/templates/prometheusrule.yaml`.
4. Update `.github/branch-protection.yaml` to require HG-RECORDINGS on `dev`.
5. Run `cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice recordings`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-recordings | full surface (FR-01..FR-17, AC-01..AC-15) |
| ADR | ADR-0123 (HG gates), ADR-0130 (SLO-gated promotion), ADR-0133 (industry best-practice conformance) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| HG marked green while load-bearing correctness AC silently failing | Two load-bearing zero-tolerance lanes block claim |
| SLO window resets prematurely | Burn-rate cool-down policy per ADR-0130 |
| Branch-protection update accidentally bypassed | `oya-check-branch-protection-conformance` lane refuses drift |

## References

- ADR-0123 (HG gates), ADR-0130 (SLO-gated promotion), ADR-0133 (industry
  best-practice conformance).
- Google SRE Workbook — "Alerting on SLOs" (Beyer et al., O'Reilly 2018).
- AWS Well-Architected Framework — Reliability Pillar.
- OpenSLO v1 specification (`openslo.com`).
