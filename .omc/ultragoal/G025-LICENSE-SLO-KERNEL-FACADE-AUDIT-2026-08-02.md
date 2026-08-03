# G025 license/SLO kernel-facade pair audit — 2026-08-02

State: **PLANNING_ONLY — REFACTOR REQUIRED, NO MOVE/DELETE AUTHORIZED**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.

## Result

The two apparent dual homes are intentional kernel/facade dependency pairs, not duplicates:

| Kernel | Facade | Direct relationship | Disposition |
|---|---|---|---|
| `libs/oya-check-license-policy` | `ci/facade/license-policy` | facade Cargo + BUCK directly depend on kernel | **KEEP / REFACTOR_CANDIDATE** |
| `libs/oya-check-slo-coverage` | `ci/facade/slo-coverage` | facade Cargo + BUCK directly depend on kernel | **KEEP / REFACTOR_CANDIDATE** |

Neither row is a safe MOVE or DELETE candidate today. The smallest honest future slice is an explicit kernel-into-facade refactor (or face-internal colocation supported by the registry grammar), not a path-only codemod move.

## License-policy evidence

- `libs/oya-check-license-policy/Cargo.toml:2` declares package `oya-check-license-policy`; its pure library exposes `LicensePolicy` and `LicensePolicyError`.
- `ci/facade/license-policy/Cargo.toml:9-11` explicitly calls itself the portable conformance gate and says it reuses the existing kernel.
- `ci/facade/license-policy/Cargo.toml:20` directly depends on `../../../libs/oya-check-license-policy`.
- `ci/facade/license-policy/src/lib.rs:5-7` states the purpose: keep the legacy predicate and cloud-CI gate from drifting.
- `ci/facade/license-policy/src/lib.rs:17` imports `LicensePolicy` and `LicensePolicyError`; `:109` instantiates the kernel policy.
- `ci/facade/artifact-inventory-registry/Cargo.toml:36` consumes the facade package `ci-license-policy`.
- `marketplace/facade/dev-cli/Cargo.toml` and its BUCK target consume the kernel directly. This is a retired CLI surface, but retirement is not evidence that the CI facade stopped depending on the kernel.
- module-membership registers `libs/oya-check-license-policy`; crate-catalog coverage registers `ci-license-policy`. Both homes are presently accounted for.

## SLO-coverage evidence

- `libs/oya-check-slo-coverage/Cargo.toml:2` declares package `oya-check-slo-coverage`; its pure API exposes `validate_slo_coverage` and typed records/report/error.
- `ci/facade/slo-coverage/Cargo.toml:9-11` explicitly says the producer owns registry/catalog I/O while the gate reuses the existing pure kernel.
- `ci/facade/slo-coverage/Cargo.toml:20` directly depends on `../../../libs/oya-check-slo-coverage`.
- `ci/facade/slo-coverage/src/lib.rs` wraps the kernel in the portable keyed-finding gate contract.
- `ci/facade/artifact-inventory-registry/Cargo.toml:35` and `ci/facade/crate-registration/Cargo.toml:32` consume `ci-slo-coverage`.
- `marketplace/facade/dev-cli/Cargo.toml` and BUCK consume the kernel directly.
- module-membership registers `libs/oya-check-slo-coverage`; crate-catalog coverage registers `ci-slo-coverage`.

## Why the earlier Candidate A is rejected

Name equality at the leaf level (`license-policy`, `slo-coverage`) hid a real architectural seam:

- `libs` packages are typed pure predicates/kernels.
- `ci/facade` packages translate producer-owned JSON rows into stable gate findings and IDs.
- The facades compile against the kernels today.

Therefore:

1. **DELETE is invalid**: live CI facade importers remain.
2. **MOVE into the existing leaf is not a mechanical codemod move**: the leaf already contains a different Cargo package with its own API and tests.
3. **Blind merge is invalid**: it would need API/test/BUCK/Cargo/importer rewrites and registry projection updates.
4. **KEEP forever is not the destination**: `libs/` remains a forbidden tail under the reorg northstar; the eventual operation is REFACTOR/ABSORB, not MOVE.

## Smallest safe future executable slice

After #1526 and #1523 promoted green, independently design and review one pair at a time:

1. Inline or module-absorb the pure kernel into its existing `ci/facade/<leaf>` package while preserving the typed API and differential tests.
2. Rewrite all direct kernel importers to the facade-owned pure module/API.
3. Retire the dev-CLI importer rather than preserving a CLI-only dependency.
4. Prove zero importers of `oya-check-<leaf>` across Cargo, BUCK, and Rust source.
5. Remove the libs package, regenerate controller-owned membership/catalog projections, and cold Buck2 verify.

No move-plan JSON, code move, deletion, registry edit, push, or activation occurred here. Independent agent transport failed again; this coordinator audit is evidence for planning, not independent APPROVE.
