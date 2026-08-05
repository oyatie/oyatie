---
doc_class: Contract
title: SecretProvider rotation contract
microservice: cloud-secrets
owner_team: axis-cloud-secrets + ops-security
kanban_task: t_bc655724
created_at_utc: 2026-07-01T09:12:39Z
claim_ceiling: contract_runbook_only
status: specified
---

# SecretProvider rotation contract

This contract specifies the first SECRETS-001 slice: the SecretProvider abstraction, OpenBao per-cell deployment shape, KEK/DEK/HSM partition vocabulary, and the required shape of a non-production rotation drill artifact. It is contract/runbook-only and does not claim a live OpenBao cluster, HSM integration, rotation scheduler, namespace controller, audit-chain writer, REST/SDK runtime, or production readiness.

## Authority and source refs

- `docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md:26-113` is Proposed planning context for OpenBao, per-tenant per-cell HSM partitions, the per-capability SecretProvider trait, and quarterly key-rotation drills.
- `docs/decisions/ADR-0161-csi-storage-class-canonical.md:66-76` is Accepted authority that storage encryption must bind to per-pack KMS/HSM handles and topology-aware cell placement.
- `cloud/cloud-secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md:54-85,177-190,228-270` supplies the service-local SecretReference, namespace, rotation policy, audit event, SLO, and verification vocabulary.
- `cloud/cloud-secrets/crates/oya-secrets-domain/src/lib.rs:1-5,21-31,74-88,188-223,254-356` supplies the current code-backed metadata-only OpenBao SecretReference and parser boundary.
- `cloud/cloud-kms/crates/oya-cloud-kms-enclave-kernel/src/chain.rs:41-106`, `token.rs:84-196`, and `dek_cache.rs:42-182` supply the implemented KEK version-chain, WrappedKek/WrappedDek, and bounded-TTL DEK cache vocabulary.
- `docs/decisions/ADR-0536-hyperscaler-grounded-substrate-decision-matrix.md:172-188` defines the KMS one-way-door, version-rotation, bounded-TTL DEK, and crypto-shred doctrine.
- `docs/decisions/ADR-0537-dogfood-bootstrap-order-rust-owned-stack-doctrine.md:50-58` places KMS unseal before secrets + workload identity and makes fetch-fail = deploy-fail.
- `evidence/cell-topology/cell-001-contract-snapshot-20260701.md:33-56,62-65` provides the current downstream-safe cell fixture and explicitly preserves the no-runtime/no-secret-material boundary.

## Claim boundary

Allowed by this artifact:

- Design, review, and descendant spec/runbook work may cite the contract names, event names, and state transitions below.
- Non-production drills may produce artifacts conforming to the drill envelope defined here and the companion runbook.
- Consumers may model SecretProvider dependencies as a typed port rather than importing OpenBao/HSM path semantics.

Forbidden by this artifact:

- No raw secret value, API token, kubeconfig, private key, HSM PIN, OpenBao root token, Shamir share, or provider credential may be stored in source, logs, evidence, Kanban, or screenshots.
- Do not call OpenBao, HSM, Kubernetes, OpenTofu, Argo CD, provider APIs, or audit-chain writers from this contract.
- Do not claim production HSM custody, live key rotation, live failover, live CSI encryption, tenant deletion crypto-shred, or audit-chain persistence from this contract alone.
- Do not hand-edit generated JSON or add a new CI/CLI authority surface.

## SecretProvider abstraction

`SecretProvider` is the only dependency product/runtime code may see. OpenBao, HSM partitions, KEK versions, DEK cache state, policy fragments, and audit sinks stay behind the provider boundary.

Required logical methods:

| Method | Input | Output | Required behavior |
| --- | --- | --- | --- |
| `resolve(reference, context)` | `SecretReference` or `${openbao:secret/<path>[@vN]}` plus workload identity, tenant, microservice, home cell, purpose, and policy version | `SecretLease` or typed denial; never raw evidence output | Evaluates Cedar before provider access, returns material only to workload-side delivery plumbing, applies TTL ceiling, emits `SecretAccessed`, and fails closed if identity, policy, OpenBao, HSM, or audit preconditions are absent. |
| `rotate(reference, policy, context)` | reference, rotation policy id, reason, initiator, witness set, target cell | `RotationReceipt` with old/new version labels and evidence refs | Creates a new encrypt-capable KEK or secret version, keeps old version decrypt-only for the dual-publish window, never re-encrypts existing ciphertext as a default rotation behavior, and emits `SecretRotated` plus any `KekRotated`/`KekAttested` evidence refs. |
| `revoke(reference, reason, context)` | reference, reason, target version, subscriber set | `RevocationReceipt` | Refuses future resolution for the version/scope, pushes revocation to subscribers, emits `RevocationPush`, and escalates if p99 push deadline is missed. |
| `attest(scope, context)` | cell/tenant/pack scope, HSM partition ref, OpenBao namespace, audit window | `CustodyAttestation` | Reports metadata-only custody posture: HSM validation class, KEK version chain status, OpenBao namespace sealed/unsealed state, audit completeness, and non-claim flags. |

Contract invariants:

1. Consumer code depends on the provider port and typed references only; it never imports OpenBao API paths, HSM partition identifiers, or raw material types.
2. Every call is scoped by tenant, microservice/capability, home cell, purpose, and policy version.
3. Human, auditor, and CI principals may inspect metadata/evidence only; they never receive SECRET-class material.
4. Workload identity is mandatory. A pod that cannot obtain identity or a secret reference fails deployment/readiness rather than starting in an identityless or plaintext fallback mode.
5. Resolution caches are bounded by the current `MAX_SECRET_REFERENCE_CACHE_TTL_SECONDS=60` ceiling unless a stricter runtime policy applies.
6. A denied or unavailable resolution is an auditable event, not a silent timeout or local plaintext fallback.

## OpenBao per-cell deployment shape

This is the desired shape to be proven later by runtime/IaC lanes. It is not a live deployment claim.

| Plane | Contract shape | Notes |
| --- | --- | --- |
| Cell ownership | One OpenBao authority per cell or per hardened cell pair, fronted by the cell-local SecretProvider runtime. | Cell IDs come from CELL-001 topology snapshots or later runtime cell inventory. Cross-cell secret resolution is denied unless an explicit residency/failover contract permits it. |
| Namespace | `cell/<cell_id>/tenant/<tenant_hash>` for tenant material and `cell/<cell_id>/platform/<component>` for platform material. | Namespace identifiers are metadata-only; tenant hash is non-reversible in evidence. |
| KV/secret path | `secret/data/t/<tenant_id>/<microservice>/<purpose>/<secret_name>` for OpenBao storage internals. | Consumers use `secretref:v1:...` or `${openbao:secret/...}` handles, never this internal path directly. |
| Transit path | `transit/<purpose>/<key_name>` for KEK wrapping, DEK wrapping, signing, and PKI where applicable. | Adapter-specific path shapes stay behind SecretProvider/KMS provider ports. |
| HSM custody | Per-cell HSM partition supplies the sealing root and/or KEK custody. KR regulated cells require KCMVP posture plus FIPS 140-3 where applicable; global cells require FIPS 140-3 class posture. | The current code-backed KMS implementation models typed material and version chains; production HSM procurement remains a separate gate. |
| Audit sink | OpenBao audit device and SecretProvider audit events feed audit-chain once that runtime exists. | This contract only requires event shape and evidence refs. It does not claim persistence. |
| Storage | OpenBao state and audit metadata must use topology-aware encrypted storage with canonical StorageClass binding when deployed in Kubernetes. | Accepted ADR-0161 makes encryption/KMS handles and topology-aware placement mandatory for later runtime work. |

## KEK/DEK/HSM vocabulary

| Term | Contract definition |
| --- | --- |
| `HsmPartitionRef` | Metadata pointer to the per-cell/per-pack HSM partition that protects sealing roots and/or KEKs. Fields: `cell_id`, `pack_id`, `partition_id`, `validation_class`, `attestation_ref`, `valid_until`, `claim_status`. Never contains PINs, shares, or credentials. |
| `SealingRoot` | Root key material used to seal persisted KEKs. It is generated/ingested under ceremony evidence and must not cross the crypto boundary except as an approved wrapped/exported transitional form during ADR-0510 posture. |
| `KEK` | Key Encryption Key scoped by tenant, cell, purpose, and version. New writes use only the current KEK version. Prior versions are decrypt-only. |
| `DecryptOnlyKek` | Retired KEK version that can unwrap historical DEKs but has no wrap/encrypt capability. This is the enforcement primitive for version rotation. |
| `DEK` | Data Encryption Key generated for object/row/blob/payload encryption and wrapped by a specific KEK version. DEKs may be cached only in bounded TTL/cardinality caches. |
| `WrappedKekToken` | Strict encoded token carrying a KEK sealed under a root. Header fields are authenticated data; malformed or tampered tokens fail closed. |
| `WrappedDek` | Strict encoded token carrying DEK id, KEK id, KEK version, nonce, and ciphertext. The KEK version routes unwrap during decrypt-only rotation. |
| `RotationPolicy` | Stable policy id plus `credential_class`, cadence, max overdue hours, dual-publish window, emergency revoke deadline, and witness requirements. Policy ids are stable across migrations. |
| `RotationReceipt` | Metadata-only event/evidence record with old/new versions, dual-publish window, affected references, initiator/witness refs, audit refs, and non-claim flags. |
| `CustodyAttestation` | Metadata-only attestation that binds cell, pack, HSM validation class, OpenBao namespace, KEK version chain, audit completeness, and expiry. |

## Rotation state machine

Normal non-emergency rotation uses the following states:

1. `requested`: a `RotationPolicy` selects the reference/KEK scope and records reason, non-prod/prod flag, and required witnesses.
2. `preflighted`: SecretProvider confirms workload identity, Cedar policy, OpenBao namespace, HSM partition health, audit sink readiness, and no raw material in the artifact.
3. `new_version_created`: a new KEK or secret version exists and is encrypt-capable for forward writes.
4. `dual_publish`: old and new versions can decrypt; only the new version can wrap/encrypt. The old version is `DecryptOnlyKek` or equivalent.
5. `validated`: synthetic read/unwrap/resolve checks pass against old and new versions without raw material disclosure.
6. `promoted`: new version is primary for all new resolves/wraps; dependent sidecars observe revocation or refresh messages.
7. `retired`: old version exits the dual-publish window and is decommissioned or retained solely under the documented decrypt-only retention rule.
8. `sealed`: `RotationReceipt`, `KekRotated`, `SecretRotated`, `KekAttested`, and dashboard/SLO refs are sealed or recorded as explicit non-claims when audit-chain runtime is absent.

Emergency rotation may skip the ordinary cadence but must not skip identity, witness, audit, and notification evidence. A failed preflight or missing audit path fails closed.

## Non-production rotation drill artifact envelope

A non-production rotation drill is complete only when it writes a metadata-only artifact with these fields. The companion runbook `cloud/cloud-secrets/runbooks/non-prod-secretprovider-rotation-drill.md` owns the operator procedure and example fixture.

| Field | Required value shape |
| --- | --- |
| `artifact_id` | Stable id, for example `SECRETS-001-NONPROD-ROTATION-DRILL-<date>-<cell>` |
| `artifact_kind` | `secretprovider_nonprod_rotation_drill` |
| `claim_ceiling` | `non_prod_drill_metadata_only` unless runtime evidence exists and is separately reviewed |
| `tenant_scope` | Synthetic or non-production tenant id/hash only |
| `cell_id` | Cell under drill, e.g. `oyatie-cloud-provider-kr-seoul-1-a-001` from the current fixture |
| `openbao_namespace_ref` | Namespace metadata reference; no token or unseal material |
| `hsm_partition_ref` | `HsmPartitionRef` metadata; no PIN/share |
| `rotation_policy_id` | Stable policy id and cadence class |
| `references_rotated` | List of SecretReference ids or path-safe handles; no raw values |
| `kek_chain_before` / `kek_chain_after` | Current version and decrypt-only retired versions only |
| `dual_publish_window` | Start/end timestamps or planned duration |
| `preflight_checks` | Identity, Cedar, OpenBao readiness, HSM attestation, audit readiness, no-raw-material scan |
| `validation_checks` | Old-version decrypt-only, new-version encrypt-forward, TTL cache expiry/fail-closed, revocation push, audit event presence |
| `events_expected` | `SecretRotated`, `KekRotated`, `KekAttested`, `RevocationPush` where applicable |
| `rollback_plan` | Revert provider pointer to prior decrypt-only version inside the dual-publish window; never reintroduce plaintext |
| `non_claims` | Explicit no-production/no-runtime/no-raw-secret claims when applicable |

## Descendant acceptance criteria

A descendant implementation/review card may call this contract satisfied only when:

1. It proves consumers depend on SecretProvider/SecretReference surfaces, not OpenBao internals.
2. It proves the runtime cell uses a cell-local namespace and HSM partition metadata, or declares that runtime proof is still blocked.
3. It verifies KEK version rotation is encrypt-forward/decrypt-only-retired and DEK cache behavior is bounded TTL/fail-closed.
4. It produces a non-production drill artifact conforming to the envelope above.
5. It records raw-secret absence, audit/evidence refs, rollback/no-plaintext behavior, and no generated JSON hand edits.
