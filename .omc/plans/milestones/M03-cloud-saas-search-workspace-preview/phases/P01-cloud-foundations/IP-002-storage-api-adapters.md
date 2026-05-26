---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P01-IP-002
title: Cloud Storage object + block API + adapter set
status: object-and-block-oci-plus-s3-object-request-contract-green-2026-05-20 (live-provider smoke pending)
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions:
  - crates/oya-cloud-storage-adapter-oci (existing; live backend refs: OCI Object Storage namespace axdotp9iv3ua bucket oyatie-audit-cold-backup; OCI Block Volume request contracts scoped to cloud compartment)
  - crates/oya-cloud-storage-adapter-s3 (request-contract only; no AWS SDK, credentials, or network calls in this slice)
purpose: Bring cloud.storage.{object,block}.* to stable; ship ≥2 provider adapters (S3/GCS/Azure-Blob/OCI-Object/OCI-Block). Current ChangeSets prove OCI Object Storage, OCI Block Volume, and S3 Object Storage request contracts; credentialed live provider smoke remains pending.
---

# M03-P01-IP-002 — Cloud Storage object + block API + adapter set

## Purpose
Bring cloud.storage.{object,block}.* to stable; ship ≥2 provider adapters (S3/GCS/Azure-Blob/OCI-Object/OCI-Block).

## Adapter target selection (2026-05-20)

This ChangeSet starts with **OCI Object Storage** because the namespace `axdotp9iv3ua`
and bucket `oyatie-audit-cold-backup` already exist in the phase ground truth.
It adds a provider-neutral object port and deterministic OCI request-shape adapter
without making credentialed network calls. The follow-up OCI Block Volume
ChangeSet adds the provider-neutral block create port plus deterministic OCI
Block Volume request shape. The S3 ChangeSet adds a second object provider
request contract without AWS SDK or credentialed network calls. Live
credentialed smoke remains pending before this IP can be marked complete.

## Symbols-to-grit-claim
```
crates/oya-cloud-storage-domain/src/lib.rs::StorageProviderObjectPort
crates/oya-cloud-storage-domain/src/lib.rs::StorageProviderBlockPort
crates/oya-cloud-storage-adapter-oci/src/lib.rs::OciObjectStorageAdapter
crates/oya-cloud-storage-adapter-oci/src/lib.rs::OciBlockStorageAdapter
crates/oya-cloud-storage-object-api/src/lib.rs::put
crates/oya-cloud-storage-object-api/src/lib.rs::get
crates/oya-cloud-storage-block-api/src/lib.rs::create
crates/oya-cloud-storage-adapter-s3/src/lib.rs::S3Adapter
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
- OCI Object Storage request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-20).
- OCI Block Volume request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-20).
- S3 Object Storage second-provider request-contract slice: targeted cargo check/test/clippy return 0 (met 2026-05-20).
- Credentialed live provider smoke remains required before marking this whole IP complete.
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
icm store -t context-oyatie -c 'M03-P01-IP-002 OCI object+block plus S3 object request contracts green; live provider smoke pending' -i high -k 'M03-P01-IP-002,partial,live-smoke-pending'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by these ChangeSets: OCI namespace, bucket, object
path, compartment, volume ref, S3 region/bucket/object path, and evidence-ref
construction no longer need to leak into Cloud Storage domain/API crates;
provider-specific object and block request shapes are confined to
`oya-cloud-storage-adapter-oci` and `oya-cloud-storage-adapter-s3` behind
`StorageProviderObjectPort` and `StorageProviderBlockPort`.

## ChangeSet evidence — cs-m03-p01-storage-object-oci-adapter-port-2026-05-20
- Added provider-neutral `StorageProviderObjectPort` plus validated put/get request and receipt types in `oya-cloud-storage-domain`.
- Added `oya-cloud-storage-adapter-oci` with deterministic OCI Object Storage PUT/GET command shapes and provider-bucket drift tests.
- Verification: `cargo test -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci`; `cargo clippy -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci`.

## ChangeSet evidence — cs-m03-p01-storage-block-oci-adapter-port-2026-05-20
- Added provider-neutral `StorageProviderBlockPort` plus validated block volume create request and receipt types in `oya-cloud-storage-domain`.
- Extended `oya-cloud-storage-adapter-oci` with deterministic OCI Block Volume create command shape and provider-volume drift tests.
- Verification: `cargo test -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci`; `cargo clippy -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci`.
- Status boundary: OCI object + block request contracts are green; second storage provider and live credentialed provider smoke remain pending.

## ChangeSet evidence — cs-m03-p01-storage-s3-object-adapter-port-2026-05-20
- Added `oya-cloud-storage-adapter-s3` with deterministic S3 Object Storage PUT/GET command shapes and provider-bucket drift tests.
- Extended `StorageProviderKind` with `S3ObjectStorage` while preserving the provider-neutral object port contract.
- Verification: `cargo test -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci -p oya-cloud-storage-adapter-s3`; `cargo clippy -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci -p oya-cloud-storage-adapter-s3 --all-targets -- -D warnings`; `cargo check -q -p oya-cloud-storage-domain -p oya-cloud-storage-adapter-oci -p oya-cloud-storage-adapter-s3`.
- Status boundary: OCI object + block and S3 object request contracts are green; credentialed live provider smoke remains pending.
