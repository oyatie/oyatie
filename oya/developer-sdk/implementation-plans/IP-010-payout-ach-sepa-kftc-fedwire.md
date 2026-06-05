---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M06-ecosystem-developer-portal
phase: P01-developer-sdk-substrate
impl_plan_id: IP-010-payout-ach-sepa-kftc-fedwire
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion, payout-settlement-correctness, kyc-pipeline-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010-payout-ach-sepa-kftc-fedwire: Payout substrate (ACH + SEPA + KFTC + FedWire adapters)

## Intent

Daily settlement batch: aggregate developer balances; route per developer's pack-localized payment rail (ACH US, SEPA EU, KFTC KR, FedWire US-wire); reconcile against bank statements.

This IP advances PRD AC criteria per `microservices/developer-sdk/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-developer-sdk-payout-kernel`
- `oya-developer-sdk-payout-domain`
- `oya-developer-sdk-payout-usecase`
- `oya-developer-sdk-payout-adapter-ach`
- `oya-developer-sdk-payout-adapter-sepa`
- `oya-developer-sdk-payout-adapter-kftc`
- `oya-developer-sdk-payout-worker`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/developer-sdk/src/crates/oya-developer-sdk-payout-domain/src/settlement.rs` | create | pure settlement aggregation |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-payout-adapter-ach/src/lib.rs` | create | NACHA ACH file emitter |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-payout-adapter-sepa/src/lib.rs` | create | SEPA pain.001 XML emitter |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-payout-adapter-kftc/src/lib.rs` | create | KFTC firm-bank protocol emitter |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-payout-worker/src/main.rs` | create | nightly settlement worker |

| `microservices/developer-sdk/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/developer-sdk/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
pub fn settle(developers: &[Developer], ledger: &Ledger, today: NaiveDate) -> Vec<Settlement> {
    developers.iter().filter_map(|dev| {
        let balance = ledger.balance(dev.id);
        if balance >= dev.payout_threshold() {
            Some(Settlement {
                developer: dev.id,
                amount: balance,
                rail: dev.preferred_rail(),
                settlement_date: today,
            })
        } else { None }
    }).collect()
}
```

Layer assignment compliance (per ADR-0105 13-layer enum):
- `*-kernel` crates declare port traits + value types only; no dependencies on other project crates.
- `*-domain` crates implement pure domain logic; depend on `*-kernel` only.
- `*-usecase` crates orchestrate domain calls; depend on `*-kernel` + `*-domain` only.
- `*-adapter*` crates implement port traits against concrete backends; depend on `*-kernel` + `*-domain` + `*-usecase`; NEVER imported directly by `*-rest` or `*-app`.
- `*-rest` crates expose HTTP routes; depend on `*-kernel` + `*-api` + `*-usecase`.
- `*-worker` crates run long-lived loops; same dependency rules as `*-rest`.
- `*-app` crates are composition roots; the only crates allowed to wire concrete `*-adapter*` instances to `*-usecase` ports.

Port-in-kernel rule (per ADR-0064 SWEEP-I) is enforced by the `port-location` CI lane.

## Acceptance Gates

All gates must exit 0 before this IP is `verified`:

```bash
cargo check -p oya-developer-sdk-payout-kernel --all-features
cargo build -p oya-developer-sdk-payout-kernel --all-features
cargo clippy -p oya-developer-sdk-payout-kernel --all-features -- -D warnings
cargo nextest run -p oya-developer-sdk-payout-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-developer-sdk-payout-kernel --no-deps
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=lean-a2 --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash
buck2 build //:quality-lane-registry-authority-check # lane=payout-settlement-correctness --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=kyc-pipeline-correctness --microservice developer-sdk
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_settle_threshold_respected` | Balance < threshold → no settlement |
| `test_ach_nacha_file_schema_valid` | NACHA validator passes |
| `test_sepa_pain001_xsd_valid` | ISO 20022 XSD passes |
| `test_kftc_firm_bank_protocol_valid` | KFTC sample validator passes |
| `test_settlement_reconciliation_byte_equal` | Ledger total = bank-side total |
| `test_dual_signature_required_above_10k` | Manual review queue triggered |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/developer-sdk/tests/fixtures/ip-010-payout-ach-sepa-kftc-fedwire/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Settlement total mismatches ledger total.
- Dual-signature bypass on amount > $10k.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/developer-sdk/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/developer-sdk/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/developer-sdk/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-010-payout-ach-sepa-kftc-fedwire`
- `microservice`: `developer-sdk`
- `milestone`: `M06-ecosystem-developer-portal`
- `phase`: `P01-developer-sdk-substrate`
- `claim_paths`: every glob declared above
- `acceptance_lanes_green`: exhaustive list of CI lanes that ran and exited 0
- `test_count`: {unit, integration, e2e}
- `coverage_pct`: float
- `multispectrum_review_facets`: F1..F9 + A1..A7 + M1..M2 minimum
- `signature`: Ed25519 signing per ADR-0181

## Next IP

[`IP-011-tax-form-1099-vat-moss-kr-vat`](IP-011-tax-form-1099-vat-moss-kr-vat.md)

## References

- NACHA ACH file spec
- ISO 20022 pain.001 XSD
- KFTC firm-bank protocol
- Stripe payout docs
