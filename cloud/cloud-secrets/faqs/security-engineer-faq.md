# `cloud-secrets` µservice — Security Engineer FAQ

22 real questions raised against the µservice that owns Oyatie's secret + credential plane.

---

**Q1. Does `cloud-secrets` replace HashiCorp Vault?**

For tenants on Oyatie, yes. The µservice covers Vault's static + dynamic + PKI secret engines, transit, and audit; the displacement is
explicit in ADR-0219. Tenants using external Vault clusters can still bridge via the `VaultExternal` adapter, but the bridge is
considered deprecated migration plumbing, not a long-term posture.

---

**Q2. Why don't we use AWS Secrets Manager / Azure Key Vault / GCP Secret Manager directly?**

They're cloud-bounded. Oyatie tenants frequently span multiple clouds (paid tenant_class supports ≤ 4 providers). A tenant secret should not
depend on which cloud you happen to be reading from. `cloud-secrets` provides one Cedar-gated surface across providers, with HPKE-encrypted
cross-region replication.

---

**Q3. How are secret values encrypted at rest?**

Envelope encryption. Each secret has a per-secret DEK (AES-256-GCM), encrypted by the tenant's KEK. The KEK is either:
- Platform KEK in AWS KMS (demo_trial + paid default)
- Tenant KEK in CloudHSM / Dedicated HSM (paid default, paid mandatory)
- Tenant KEK in customer-owned HSM (paid opt-in, paid encryption-key BYOK ceremony per ADR-0251 §D-10)

Re-keying re-wraps DEKs without re-encrypting data — fast and audit-evident.

---

**Q4. What does "lease" mean?**

Every read returns a lease, even for "static" secrets. The lease has a TTL (15 min paid, 30 min paid, 1 h demo_trial) and a fencing
token. Readers cache the value only until lease expiry; after expiry they must re-read. This makes revocation possible without
trusting clients to honor it (servers reject stale fencing tokens).

---

**Q5. How does dynamic credential issuance work?**

`cloud_secrets::Action::DynamicCredential` mints a short-lived credential against an underlying backend (Postgres user, IAM role,
SSH cert, JWT). Lifetime defaults to 15 min, max 1 h. The credential is registered in the audit chain at issuance and at revocation;
the backend automatically purges expired credentials.

---

**Q6. What's the encryption-key BYOK ceremony (ADR-0251 §D-10)?**

A 4-quorum key ceremony (2-of-3 Oyatie + 2-of-3 customer) where the tenant's KEK is generated inside the tenant-owned HSM and never
leaves it. Oyatie holds only the public KEK reference. The ceremony is scripted (`microservices/cloud-secrets/runbooks/byok-ceremony.md`)
and rehearsed in dev (`oya secrets byok-rehearsal`).

---

**Q7. How are MLS package keys handled differently?**

MLS (RFC 9420) rotates leaf node keys per-epoch — that's not a time-based cadence but a tree-state cadence. The `MlsPackageKey`
rotator hooks into the messenger µservice's epoch advancement and refreshes keys per the MLS spec's lifecycle, not the generic
30/14/7-day cadence.

---

**Q8. What happens if an HSM goes offline?**

Reads of already-leased secrets continue (DEK is in-process cache for the lease lifetime). Writes / rotations / new reads return
`HsmUnavailable` until failover. paid tenants have multi-region HSM with synchronous mirroring; paid has hot standby in-region;
paid has software fallback (with audit alert + governance escalation).

---

**Q9. How does `cloud-secrets` integrate with `cloud-iac`?**

`cloud-iac` fetches provider credentials from `cloud-secrets` per-plan-apply as short-lived leases. The credentials never persist in
Terraform/Pulumi state — `cloud-iac` writes them to the runner's in-memory environment and disposes after apply. The `IamRoleAssumption`
secret kind is the canonical handoff.

---

**Q10. Where do provider-credential BYOK credentials sit (ADR-0255 §D-4)?**

Per `feedback_byok_everywhere_credentials.md` + ADR-0255 §D-4: a tenant can opt into BYOK for LLM/provider credentials independent
of secret encryption BYOK. The `LlmProviderApiKey` secret kind has a `byok_required_by_pack` flag that forces tenant-supplied keys
when a regulated pack is active.

---

**Q11. What's the audit-chain integrity model?**

Every read/write/rotate/lease/revoke produces an audit event with `(prev_hash, event_payload_hash) → curr_hash` via BLAKE3-256.
The chain is verifiable client-side; paid tenants can anchor the chain head to a public blockchain (Bitcoin or Ethereum) at
24 h cadence for tamper evidence beyond the platform's bounds.

---

**Q12. Can a tenant rotate without changing the underlying backend?**

Only for static secrets where the rotation is pure metadata (e.g. tagging a secret as superseded). Real credential rotation always
mutates the backend (DB user password change, OAuth refresh, IAM key roll). The rotator framework enforces this — declarative
"rotation that only changes the stored value but not the backend" is a CI smell that fails `lean-a7-rotator-substance`.

---

**Q13. How fine-grained are Cedar permits?**

Per-secret-name. A principal can hold `read` on `secrets/postgres-prod-primary` but not on `secrets/postgres-prod-replica`. Pattern
permits (`secrets/postgres-prod-*`) exist but require explicit policy authoring; there's no implicit wildcard.

---

**Q14. What's the relation to `kms`?**

`kms` µservice handles **key material lifecycle** (KEK creation, rotation, revocation, attestation). `cloud-secrets` handles
**secret material lifecycle** (anything else). The line is sharp: keys are in `kms`; anything else is in `cloud-secrets`. They
communicate over an internal mTLS gRPC channel.

---

**Q15. Can I store a binary blob (e.g. a 1 MiB PFX)?**

Up to 4 MiB per secret. Larger blobs require a different µservice (`drive` for files, `kms` for key material).

---

**Q16. How does write-conflict resolution work?**

Optimistic concurrency on `version`. A writer must pass the expected current version; mismatched version → `WriteConflict`. The
client must re-read and retry. There is no "last writer wins".

---

**Q17. What happens during a tenant decommission?**

`cloud_secrets::Action::Decommission` schedules a 30-day quarantine, then crypto-shreds the tenant KEK. After KEK shred, no plaintext
recovery is possible. The quarantine period gives auditors time to extract evidence.

---

**Q18. How are secret references safe in `cloud-iac` inputs?**

Inputs use `${ref:cloud_secrets.<name>}` placeholders. The `cloud-iac` runner resolves these at apply-time by calling `cloud-secrets`
under its own principal; the resolved value never appears in the plan JSON or state file. Cleartext values in inputs trip
`lean-a4-secret-cleartext`.

---

**Q19. What's the throughput ceiling?**

50 req/s/tenant demo_trial, 500 paid, 5,000 paid, 50,000 paid. Above-ceiling traffic returns `RateLimited` with the ceiling in
the response header. paid can negotiate a higher ceiling (single-tenant cells scale higher).

---

**Q20. How does break-glass read work?**

`cloud_secrets::Action::BreakGlassRead` is paid-only. It bypasses normal Cedar for a single secret read with a 15-minute single-use
token issued by a reviewer-agent + governance human approver. Break-glass reads are doubly-logged (audit chain + a separate
break-glass register) and trigger an immediate notification to the tenant's security on-call.

---

**Q21. Can rotators be third-party plugins?**

Yes, at paid tenant_class. Third-party rotators must be cosign-signed, declare their `RotateCtx` capability footprint, and pass a tenant-side
review before activation. demo_trial + paid tenants are restricted to the first-party rotator library.

---

**Q22. What's the recovery RTO/RPO?**

RPO: 0 (synchronous replication for paid tenant_class, 5-min async snapshot for demo_trial + paid). RTO: ≤ 15 min for paid, ≤ 5 min for paid
(HSM hot standby), ≤ 1 h for demo_trial + paid.
