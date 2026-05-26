---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P01-IP-001
title: Cloud KMS provider-agnostic API + adapter set
status: adapter-port-request-contract-green-2026-05-20 (live-provider-smoke pending credentials)
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions:
  - crates/oya-cloud-kms-adapter-openbao (new; live backend: kms.oyatie.com)
  - crates/oya-cloud-kms-adapter-oci (new; live backend: bitween-default-vault in cloud compartment)
purpose: Bring cloud.kms.{encrypt,decrypt} to stable; ship ≥2 provider adapters (OpenBao + OCI KMS; AWS/GCP/Azure are deferred to subsequent IPs).
---

# M03-P01-IP-001 — Cloud KMS provider-agnostic API + adapter set

## Purpose
Bring `cloud.kms.{encrypt,decrypt}` to stable; ship ≥2 provider adapters with live integration tests.

## Adapter target selection (2026-05-16)

Per directive "≥ 2 of {AWS, OCI, GCP, Azure, NaverCloud, NHN, KT, KakaoCloud}", the two adapters shipped in this IP are **OpenBao** + **OCI KMS** because both backends are already live on this bring-up:

| Adapter | Backend | Live since | API surface |
|---|---|---|---|
| `oya-cloud-kms-adapter-openbao` | OpenBao v2.5.3 transit engine on the on-prem KR primary cell | 2026-05-16 (this session) | `POST /v1/transit/encrypt/<key>` + `POST /v1/transit/decrypt/<key>` |
| `oya-cloud-kms-adapter-oci` | OCI KMS vault `bitween-default-vault` + AES-256 master key in `cloud` compartment, region ap-chuncheon-1 | 2026-05-16 (this session, via OpenTofu) | `Encrypt` / `Decrypt` against the per-vault management endpoint |

AWS / GCP / Azure adapters are sequenced into follow-up IPs (M03-P01-IP-001a/b/c) once those tenancies are provisioned. They're not required for the ≥2-adapter acceptance criterion because OpenBao and OCI are two distinct providers under the same trait.

## Symbols-to-grit-claim
```
crates/oya-cloud-kms-api/src/lib.rs::encrypt
crates/oya-cloud-kms-api/src/lib.rs::decrypt
crates/oya-cloud-kms-adapter-openbao/src/lib.rs::OpenBaoKmsAdapter
crates/oya-cloud-kms-adapter-oci/src/lib.rs::OciKmsAdapter
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-governance-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
- Adapter port and request-shape slice: targeted cargo check/test/clippy return 0 (met 2026-05-20).
- Live-provider smoke remains required before marking this whole IP complete.
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M03-P01-IP-001 Cloud KMS provider-agnostic API + adapter set shipped; acceptance commands green' -i high -k 'M03-P01-IP-001,complete'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this ChangeSet: provider-specific KMS request paths no longer need to enter the Cloud KMS domain/API crates; OpenBao and OCI key refs are confined to adapter crates behind `KmsProviderCryptoPort`.

## ChangeSet evidence — cs-m03-p01-kms-adapter-port-2026-05-20
- Added provider-neutral `KmsProviderCryptoPort` plus validated provider request/receipt types in `oya-cloud-kms-domain`.
- Added `oya-cloud-kms-adapter-openbao` and `oya-cloud-kms-adapter-oci` crates with deterministic request-shape and provider-key drift tests.
- Verification: `cargo test -q -p oya-cloud-kms-domain -p oya-cloud-kms-adapter-openbao -p oya-cloud-kms-adapter-oci`; `cargo clippy -q -p oya-cloud-kms-domain -p oya-cloud-kms-adapter-openbao -p oya-cloud-kms-adapter-oci --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-kms-domain -p oya-cloud-kms-adapter-openbao -p oya-cloud-kms-adapter-oci`.
