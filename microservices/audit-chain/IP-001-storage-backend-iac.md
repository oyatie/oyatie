---
doc_class: ImplementationPlan
status: pending
owner: axis-audit-chain
date: 2026-05-21
wave: Wave 15-IP-substance
substance_status: rewritten-bespoke
---

# IP-001: Storage backend IaC for append-only audit storage

acceptance_lanes: [opentofu-plan, helm-lint, kustomize-build, object-lock-policy, hsm-partition-policy, cross-pack-replication-forbidden]

## §A Problem
The previous IP named storage primitives but did not reconcile the live audit-chain contract with ADR-0328's OpenTofu-only substrate. Audit-chain cannot claim CloudTrail-class immutability or tenant-verifiable roots while `IP-001` still points at a Terraform HSM file and omits the per-pack WORM/Object Lock and root-publication split described in `policy/seal-integrity.md` SI-02 through SI-04.

## §B Approach
Create service-local OpenTofu modules under `microservices/audit-chain/iac/opentofu/` for the Postgres append index, S3-compatible WORM bucket, OCI Cloud-HSM partition, OpenBao signing-session references, and GitHub-pinned root-publication manifest path. Keep Helm/Kustomize only as Kubernetes packaging inputs called by OpenTofu context modules, not as the authority for cloud resources. The design must preserve `crates/oya-audit-chain-file-adapter/src/lib.rs` semantics: append-only replay detects `ChainDiverged` before any new record is accepted.

## §C Deliverables
- `microservices/audit-chain/iac/opentofu/modules/audit-postgres/main.tf` with INSERT-only emitter and SELECT+INSERT sealer role outputs.
- `microservices/audit-chain/iac/opentofu/modules/audit-worm-store/main.tf` with Object Lock Compliance mode, retention inputs from `policy/retention-matrix.yaml`, and no cross-pack replication output.
- `microservices/audit-chain/iac/opentofu/modules/audit-hsm-partition/main.tf` replacing the stale `terraform/oci-cloud-hsm-partition.tf` target.
- `microservices/audit-chain/iac/opentofu/contexts/pack-kr/main.tf` binding KR retention, OpenBao secret refs, and pack-local HSM key ids.
- `microservices/audit-chain/iac/kustomize/base/kustomization.yaml` and `overlays/pack-kr/kustomization.yaml` as deployable manifests consumed by the OpenTofu context.

## §D Implementation Steps
1. Delete the stale Terraform target from this plan and use `opentofu` paths only.
2. Model Postgres as an append index: emitter role can insert events, sealer role can insert seal records, query role reads by tenant and period.
3. Model WORM storage as immutable raw event and signed-root blobs; reject lifecycle rules that shorten pack retention below `policy/retention-matrix.yaml`.
4. Bind HSM signing to the sealing-worker SPIFFE identity and OpenBao certificate lease described in `policy/seal-integrity.md` SI-09.
5. Publish public roots and key epochs to the GitHub-pinned manifest directory named by SI-04/SI-11 so offline verifiers do not trust Postgres alone.
6. Add dry-run evidence for `pack-kr`; do not claim other packs until their overlays exist.

## §E Acceptance
- `tofu -chdir=microservices/audit-chain/iac/opentofu/contexts/pack-kr init -backend=false` succeeds.
- `tofu -chdir=microservices/audit-chain/iac/opentofu/contexts/pack-kr validate` succeeds and contains no `terraform/`, `local-exec`, or manual SSH provisioners.
- `kubectl --dry-run=client apply -k microservices/audit-chain/iac/kustomize/overlays/pack-kr` succeeds.
- `cargo test -p oya-audit-chain-file-adapter` still proves append replay and `ChainDiverged` behavior.

## §F Evidence
- `microservices/audit-chain/policy/seal-integrity.md` SI-02 through SI-11.
- `microservices/audit-chain/coherence-audit-2026-05-20.md` flags the Terraform/OpenTofu contradiction.
- `microservices/audit-chain/competitor-parity-matrix.md` compares AWS CloudTrail digest storage, Google Cloud Audit Logs, Microsoft Purview Audit, Splunk, Datadog, Vault, and GitHub-pinned publication.

## §G Counterparts
AWS CloudTrail depends on S3-delivered digest files and integrity validation; this IP closes the storage side with WORM blobs plus Postgres index and HSM signing. Google Cloud Audit Logs and Microsoft Purview Audit provide managed immutable stores but not tenant-verifiable GitHub-pinned root manifests, so the Oyatie delta is explicit offline verification rather than generic log retention.

## Stop Conditions
Do not promote this IP on line count alone. Stop if a cited path is absent, a counterpart claim cannot be traced to `competitor-parity-matrix.md` or `feature-parity-matrix-2026-05-20.md`, or a verification command above cannot run in the current checkout.


## Wave 15 Detailed Reviewer Map

### Domain vocabulary that must appear in the implementation PR
- Pack-local chain: implementation must preserve `(pack, tenant_partition, period)` as a first-class tuple, not hide it inside a generic tenant string.
- Seal lifecycle: implementation must distinguish accepted, unsealed, sealed, published, verified, redacted, and retained states where this IP touches those transitions.
- Evidence linkage: every emitted or derived record must carry an audit id, period id, root or prior-root reference when applicable, and a provenance pointer to the producing service.
- Residency boundary: pack movement is forbidden unless the IP explicitly names a tenant-initiated export path and a receiving-tenant compliance basis.
- Key material boundary: public keys may be published; private keys, HSM handles, OpenBao leases, and provider credentials must stay out of serializable responses and logs.
- Audit-of-audit: mutating or privileged read behavior introduced by this IP must itself produce an audit event rather than relying on operator notes.

### File-reference checks before implementation starts
- Re-read `microservices/audit-chain/PRD.md` for FR ids and latency/availability targets tied to `storage backend iac`.
- Re-read `microservices/audit-chain/ARCHITECTURE.md` for layer placement, runtime assumptions, and cross-product import constraints.
- Re-read `microservices/audit-chain/manifest.json` for catalog, SLO, and contract pointers; do not invent a crate or contract absent from the manifest without updating the manifest in the same change.
- Re-read `microservices/audit-chain/policy/seal-integrity.md` when the IP touches roots, proofs, keys, HSM, publication, or verifier behavior.
- Re-read `microservices/audit-chain/competitor-parity-matrix.md` and `feature-parity-matrix-2026-05-20.md` before making any CloudTrail, Google Cloud Audit Logs, Microsoft Purview Audit, Splunk, Datadog, Vault, or GitHub comparison.
- Re-read the existing Rust crates under `crates/oya-audit-chain-*` and `crates/oya-shared-audit-chain-client-kernel` so the implementation extends live behavior instead of replacing it with a parallel scaffold.

### Negative tests or static checks expected
- Cross-tenant or cross-pack input is denied before storage or signing work begins.
- Duplicate idempotency material returns the prior result only when the canonical fingerprint matches exactly.
- Tampered proof, tampered signature, stale key epoch, or missing prior root returns a structured failure rather than a generic internal error.
- Missing GitHub-pinned root/key publication keeps the period below the claim boundary even if Postgres and WORM writes succeeded.
- A downstream outage pauses or degrades explicitly; it must not silently mark the audit action complete.
- High-cardinality fields such as tenant id and principal id are not exported as metrics labels.

### Counterpart comparison rows
| Counterpart | Relevant capability | Audit-chain requirement for this IP |
|---|---|---|
| AWS CloudTrail | Delivered audit records and integrity validation | Preserve immutable event/root evidence and make the trust boundary explicit. |
| Google Cloud Audit Logs | Admin/Data/System/Policy audit taxonomy and routed log sinks | Keep event classes and export routing typed; do not collapse policy-denied and data-access events. |
| Microsoft Purview Audit | Search/export, retention policies, and investigation workflows | Keep query/export/read paths scoped, retained, and auditor-engagement aware. |
| GitHub-pinned manifests | Third-channel root/key publication for Oyatie | Ensure roots or keys affected by `storage backend iac` can be checked outside the primary storage plane. |

### Review stop line
If the implementation PR cannot point from code to PRD row, policy invariant, SLO or runbook, and counterpart row, keep the IP in pending state. Passing a markdown line count, a generated file list, or a broad statement that audit logging exists is not enough for Wave 15 closure.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/audit-chain/IP-001-storage-backend-iac.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/audit-chain/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/policy/auditor-scope.cedar`.
