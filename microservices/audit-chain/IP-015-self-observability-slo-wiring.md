---
doc_class: ImplementationPlan
impl_plan_id: IP-015-self-observability-slo-wiring
status: pending
owner: axis-audit-chain + axis-observability
acceptance_lanes: [cargo-check, cargo-build, cargo-nextest, openslo-conformance, cross-pack-replication-forbidden, hyperscaler-maturity-claims]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: Self-observability SLO + HG-AUDIT registration

## Intent

Wire audit-chain SLI emission into observability substrate; register HG-AUDIT gate in /specs/hyperscaler-gates.json per ADR-0123; add LEAN lane `oya-check-cross-pack-replication-forbidden` to BLOCKER list.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/audit-chain/slos/*` | already-created in IP-002; verify emission via SDK |
| `/specs/hyperscaler-gates.json` | update — add HG-AUDIT entry |
| `.github/branch-protection.yaml` | update — add `oya-audit-chain-self-verification` + `oya-governance-cross-pack-replication-forbidden` to dev required_status_checks |
| Cross-channel root validator | create — continuous job emitting `oya:audit_chain_root_cross_channel_match:rate` |
| Mimir recording rule | create — at `microservices/observability/iac/helm/mimir/recording-rules.yaml` add audit-chain rules |

## HG-AUDIT registration

```json
{
  "id": "HG-AUDIT",
  "name": "Hyperscaler-maturity claim gate — audit-chain",
  "microservice": "audit-chain",
  "claims": [
    "per-event Merkle inclusion proofs",
    "HSM-rooted Ed25519 signing",
    "tenant-independent offline verification",
    "three-channel root publication",
    "per-pack chain locality with cross-pack-replication-forbidden",
    "DSR cascade with Merkle proof of redaction"
  ],
  "evidence": [
    "microservices/audit-chain/competitor-parity-matrix.md",
    "microservices/audit-chain/policy/seal-integrity.md",
    "microservices/audit-chain/policy/data-residency.md"
  ],
  "cadence": "bi-annual"
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims
cargo run -p oya-dev-cli -- gate validate cross-pack-replication-forbidden --microservice audit-chain
cargo run -p oya-dev-cli -- gate validate hsm-key-rotation-overlap --microservice audit-chain
cargo run -p oya-dev-cli -- gate validate authority-cohesion
# Synthetic e2e: tamper a published Mimir root metric; cross-channel validator alerts within 60s
```

## End-to-end drill

```bash
# Cross-channel tamper drill
cargo nextest run -p oya-audit-chain-sealing-worker --test cross_channel_tamper_drill
# Mutate one of the three published channels; verify divergence detected + alarmed within 60s
```

## References

- ADR-0123 (hyperscaler-maturity claim gate).
- ADR-0139 §"Authority cohesion".
- `microservices/audit-chain/competitor-parity-matrix.md`.
- `microservices/audit-chain/policy/seal-integrity.md` §"SI-04" three-channel publication.
- `microservices/audit-chain/dashboards/*.json`.
