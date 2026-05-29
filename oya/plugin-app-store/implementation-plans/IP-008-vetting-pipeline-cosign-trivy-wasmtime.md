---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-008-vetting-pipeline-cosign-trivy-wasmtime
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion, vetting-pipeline-correctness, per-plugin-permission-enforcement]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008-vetting-pipeline-cosign-trivy-wasmtime: vetting-pipeline cosign + trivy + Wasmtime-isolation validators

## Intent

Implement the three highest-load-bearing vetting validators: Cosign signature verifier (against developer-sdk-issued ED25519 keys); Trivy vulnerability scanner (CVE Critical+ unfixed → reject); Wasmtime sandbox isolation validator (ephemeral Wasmtime engine + seccomp profile + syscall trace; any escape → reject).

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-vetting-pipeline-adapter-cosign`
- `oya-plugin-app-store-vetting-pipeline-adapter-trivy`
- `oya-plugin-app-store-vetting-pipeline-adapter`
- `oya-plugin-app-store-vetting-pipeline-usecase`
- `oya-plugin-app-store-vetting-pipeline-worker`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-adapter-cosign/src/lib.rs` | create | CosignVerifier impl shelling to cosign 2.x binary or using sigstore-rs |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-adapter-trivy/src/lib.rs` | create | TrivyScanner impl shelling to trivy 0.50.x |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-adapter/src/wasmtime_isolation.rs` | create | WasmtimeIsolationValidator: ephemeral engine + seccomp + syscall trace |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-worker/src/main.rs` | create | Long-lived worker pulling submissions + running pipeline |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
// adapter-cosign
#[async_trait]
impl SignatureVerifier for CosignVerifier {
    async fn verify(&self, submission: &PluginSubmission) -> Result<(), CosignRejection> {
        let out = tokio::process::Command::new("cosign")
            .args(["verify-blob",
                   "--key", &self.public_key_path,
                   "--signature", &submission.signature_path,
                   &submission.artifact_path])
            .output().await?;
        if !out.status.success() {
            return Err(CosignRejection::SignatureInvalid {
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        Ok(())
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
cargo check -p oya-plugin-app-store-vetting-pipeline-adapter-cosign --all-features
cargo build -p oya-plugin-app-store-vetting-pipeline-adapter-cosign --all-features
cargo clippy -p oya-plugin-app-store-vetting-pipeline-adapter-cosign --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-vetting-pipeline-adapter-cosign --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-vetting-pipeline-adapter-cosign --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate port-location --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash
cargo run -p oya-dev-cli -- gate validate vetting-pipeline-correctness --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate per-plugin-permission-enforcement --microservice plugin-app-store
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_cosign_verify_signed_artifact_passes` | Known-signed fixture → Ok |
| `test_cosign_verify_unsigned_artifact_rejects` | No signature → Err |
| `test_cosign_verify_tampered_artifact_rejects` | Modified artifact → Err |
| `test_trivy_scan_clean_artifact_passes` | Clean fixture → Ok |
| `test_trivy_scan_critical_cve_rejects` | Fixture with CVE-CRITICAL-unfixed → Err |
| `test_wasmtime_isolation_no_escape_passes` | Pure-compute fixture → Ok |
| `test_wasmtime_isolation_syscall_escape_rejects` | Fixture attempting forbidden syscall → Err |
| `test_worker_polling_loop_durability` | Worker resumes from last offset after kill |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-008-vetting-pipeline-cosign-trivy-wasmtime/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Cosign verifies an unsigned artifact (false-pass).
- Trivy misses a known CVE-CRITICAL-unfixed.
- Wasmtime isolation validator permits a known syscall escape.
- Worker re-processes the same submission twice (idempotency broken).

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-008-vetting-pipeline-cosign-trivy-wasmtime`
- `microservice`: `plugin-app-store`
- `milestone`: `M04-ecosystem-substrate`
- `phase`: `P01-plugin-app-store-substrate`
- `claim_paths`: every glob declared above
- `acceptance_lanes_green`: exhaustive list of CI lanes that ran and exited 0
- `test_count`: {unit, integration, e2e}
- `coverage_pct`: float
- `multispectrum_review_facets`: F1..F9 + A1..A7 + M1..M2 minimum
- `signature`: Ed25519 signing per ADR-0181

## Next IP

[`IP-009-per-plugin-permissions-cedar`](IP-009-per-plugin-permissions-cedar.md)

## References

- ADR-0147
- ADR-0181
- ADR-0200
- Sigstore docs
- Trivy docs
