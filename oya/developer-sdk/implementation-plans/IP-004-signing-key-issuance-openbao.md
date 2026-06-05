---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M06-ecosystem-developer-portal
phase: P01-developer-sdk-substrate
impl_plan_id: IP-004-signing-key-issuance-openbao
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004-signing-key-issuance-openbao: per-developer ED25519 signing key issuance via OpenBao

## Intent

Issue ED25519 signing keys per developer via OpenBao transit engine; 30-day expiry; auto-rotate; revocation cascades.

This IP advances PRD AC criteria per `microservices/developer-sdk/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-developer-sdk-signing-key-issuance-kernel`
- `oya-developer-sdk-signing-key-issuance-domain`
- `oya-developer-sdk-signing-key-issuance-usecase`
- `oya-developer-sdk-signing-key-issuance-adapter-openbao`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/developer-sdk/src/crates/oya-developer-sdk-signing-key-issuance-adapter-openbao/src/lib.rs` | create | OpenBao transit engine adapter |

| `microservices/developer-sdk/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/developer-sdk/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
#[async_trait]
impl SigningKeyIssuer for OpenBaoIssuer {
    async fn issue(&self, developer_id: &DeveloperId) -> Result<PublicKey, IssueError> {
        let resp = self.client.post(&format!("transit/keys/dev-{}", developer_id))
            .json(&json!({"type": "ed25519", "exportable": false}))
            .send().await?.error_for_status()?;
        let pk = self.client.get(&format!("transit/keys/dev-{}", developer_id))
            .send().await?.json::<KeyInfo>().await?;
        Ok(pk.into())
    }
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
cargo check -p oya-developer-sdk-signing-key-issuance-kernel --all-features
cargo build -p oya-developer-sdk-signing-key-issuance-kernel --all-features
cargo clippy -p oya-developer-sdk-signing-key-issuance-kernel --all-features -- -D warnings
cargo nextest run -p oya-developer-sdk-signing-key-issuance-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-developer-sdk-signing-key-issuance-kernel --no-deps
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=lean-a2 --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice developer-sdk
buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash

```

## Test Plan

| Test | Verifies |
|---|---|
| `test_issue_creates_openbao_key` | Key path appears in OpenBao after issue |
| `test_issue_returns_public_key_only` | Private key never exposed |
| `test_rotate_creates_new_version` | Rotation increments version; old version still verifies |
| `test_revoke_removes_key` | Revoked key no longer signs |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/developer-sdk/tests/fixtures/ip-004-signing-key-issuance-openbao/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Private key ever leaves OpenBao perimeter.
- Revoked key still produces valid signatures.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/developer-sdk/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/developer-sdk/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/developer-sdk/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-004-signing-key-issuance-openbao`
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

[`IP-005-api-contracts-registry`](IP-005-api-contracts-registry.md)

## References

- OpenBao transit engine docs
- Cosign ED25519 docs
