# `cloud-kms` µservice — KMS Engineer FAQ

20 real questions raised against `cloud-kms` (the µservice that owns Oyatie's key-management substrate).

---

**Q1. Does `cloud-kms` replace AWS KMS / GCP Cloud KMS / Azure Key Vault?**

`cloud-kms` is the canonical surface for tenants of Oyatie; AWS/GCP/Azure KMS are downstream surfaces. When a tenant's workload
needs a key in AWS Account X to encrypt an S3 object, `cloud-kms` either issues the key locally (via its own HSM cluster) or
brokers an AWS KMS CMK on the tenant's behalf (the CMK is owned by the tenant's AWS account; `cloud-kms` controls policy via
the AWS KMS resource policy + Cedar). Tenants never see raw key material from any of these backends.

---

**Q2. Why Marvell LiquidSecurity 2 + Thales Luna 7 + Utimaco SecurityServer for the three tiers?**

Different certifications + price points:
- **Marvell LiquidSecurity 2** (paid): AWS CloudHSM's default; FIPS 140-3 L2; lowest cost; ~$1.60/HSM-hour.
- **Thales Luna Network HSM 7** (paid): FIPS 140-3 L3; multi-region cluster mode; ~$1,800/HSM/month.
- **Utimaco SecurityServer Se Gen2** (paid): FIPS 140-3 L3 + Common Criteria EAL 4+ + BSI (German) approval; required for
  EU sovereign and K-FSI; ~$3,500/HSM/month.

Higher tenant_classes also stack — paid runs Utimaco AND Thales for failover.

---

**Q3. What's the difference between a CMK, KEK, and DEK?**

- **CMK** (customer master key): the logical tenant-scoped key alias. Identified by `cmk-2026-05-20-…`. Lives forever (until cryptoshredding).
- **KEK** (key-encryption key): the actual cryptographic material backing a CMK at a point in time. Rotates per the cadence; an
  old KEK is decrypt-only after rotation, then destroyed after the grace window.
- **DEK** (data-encryption key): per-record (or per-blob) key issued from a CMK's current KEK. The plaintext DEK encrypts the
  payload; the ciphertext DEK (encrypted by the KEK) is stored next to the payload.

This is the standard envelope-encryption pattern, mandatory in Oyatie per ADR-0251 §D-10.

---

**Q4. Why is FIPS 140-3 mandatory at paid tenant_class?**

US FedRAMP, US DoD IL2+, EU SOG-IS, KR CSAP, JP CRYPTREC, IN MeitY — most regulatory regimes that B2B SMB+ customers care about
either require or strongly recommend FIPS 140-3-validated modules. paid buys you in for the smallest cost (FIPS 140-3 L2 via
AWS CloudHSM); paid/paid upgrade to L3 for higher-assurance regulated workloads.

---

**Q5. Can a tenant bring its own KEK (BYOK)?**

Yes, at paid tenant_class. Two paths:
1. **PKCS#11 import** — tenant uploads the KEK material wrapped under a `cloud-kms` import-public-key (X25519 envelope).
   The KEK lives in a tenant-isolated HSM partition.
2. **AWS XKS / External Key Store** — the KEK lives in the tenant's own HSM; `cloud-kms` proxies operations via the
   AWS XKS protocol.

BYOK is opt-in per `pack.tenant.provider_credential_mode` config; see ADR-0255 §D-4 for the BYOK contract.

---

**Q6. How does cryptoshredding actually work?**

It's a destructive Cedar action: `cloud_kms::Action::Cryptoshred` on a CMK. The runner:
1. Marks the CMK as `state: Cryptoshredded` in the metadata store.
2. Issues a destroy command to every HSM partition holding the KEK material — the HSM zeroizes the key.
3. Issues an HSM attestation receipt confirming zeroization.
4. The receipt is anchored to `audit-chain` within tenant_class SLO (30 min paid, 5 min paid, 60 s paid).

After cryptoshredding, every payload encrypted under the CMK is unrecoverable. This is the GDPR Art. 17 / CCPA / KR PIPA right-to-delete primitive.

---

**Q7. What's the AAD (additional authenticated data) for?**

AAD binds a DEK to a context — typically `tenant_id` + a record identifier. Without AAD, an attacker who steals a ciphertext
DEK from tenant X could decrypt it in a request claiming to be tenant Y. With AAD verified at decrypt time, the wrong tenant's
request fails with `AeadAuthFailure`. AAD is mandatory in `cloud-kms` ≥ v0.42 — DEKs without AAD are refused.

---

**Q8. How are HSMs themselves attested?**

Every HSM partition is **measured at boot** (TPM 2.0 measured boot + Intel TDX / AMD SEV-SNP attestation report). On each
`Sign` operation at paid tenant_class, the HSM emits an attestation receipt referencing its measured state. `cloud-kms` validates the
receipt against the canonical "known good" measurement registered at HSM provisioning time. Drift refuses operations.

---

**Q9. Can I store a key forever (no rotation)?**

demo_trial: yes, with `--rotation-cadence none`. paid tenant_class: refused for production CMKs; allowed for explicitly tagged dev CMKs only
(`policy: dev-no-rotate`). PCI DSS v4.0 requires ≤ 24-month rotation for cardholder-data CMKs; SOC 2 best practice is annual.

---

**Q10. What's the PQC (post-quantum cryptography) story?**

paid tenant_class ships:
- **ML-KEM-768** (FIPS 203, formerly Kyber) for transit key-encapsulation.
- **ML-DSA-65** (FIPS 204, formerly Dilithium) for signing.

Hybrid mode (X25519 + ML-KEM) is the transit default at paid; pure-PQC for regulated CSF tenants. paid adds **FALCON-1024**
and **Classic McEliece** for specific regulatory regimes (KR K-FSI mandates PQC by 2028).

---

**Q11. Where do operator quorum keys live (M-of-N HSM custody)?**

paid HSM operations require M-of-N operator approval (e.g. 3-of-5). Operator keys are PIV/CAC cards held by the tenant's
designated key custodians; `cloud-kms` runs an HSM-side script that refuses operations without the quorum. Operator card
inventory is reconciled monthly; a missing card raises a P1 incident.

---

**Q12. How does cross-region key replication work?**

paid: KEKs replicate async to a secondary region within ≤ 60 s; a CMK's `current_kek_id` points to the primary region's KEK;
failover requires explicit promotion (`oya kms cmk promote-region`).

paid: KEKs replicate synchronously across ≥ 3 regions; operations route to the nearest region; replication is via
Spanner-class TrueTime (per ADR-0252).

---

**Q13. Is the HSM the only cryptographic boundary?**

For paid tenant_class — yes. The plaintext KEK never leaves the HSM. DEK encrypt/decrypt happens inside the HSM (HSM emits the
plaintext DEK in a one-shot, immediately-zeroed buffer; signing happens inside the HSM and the HSM emits the signature).
For demo_trial — the boundary is the `cloud-kms` process address space; memory is `mlock`'d and zeroized after use.

---

**Q14. How does `cloud-kms` integrate with `cloud-iam`?**

Every `cloud-kms` action is authorised via Cedar — `cloud-kms` is a Cedar resource type, and `cloud-iam` is the authority.
A typical call sequence: workload presents a `cloud-iam` token → `cloud-kms` calls `cloud-iam::authorize(action=IssueDek, resource=CMK::"…")` →
on Allow, the HSM operation proceeds; on Deny, the request fails with `KmsError::Forbidden`.

---

**Q15. What happens if an HSM fails mid-operation?**

The runner has a 3 s deadline per operation. On timeout:
1. The operation is retried on a peer HSM in the same cluster (paid tenant_class have ≥ 3-HSM clusters).
2. If all HSMs in the cluster are unreachable, the runner returns `KmsError::HsmClusterDegraded`.
3. The on-call rotation gets paged; an `infra-incident` ticket is auto-opened.
4. Decryption falls back to a peer region (paid tenant_class) if the cluster is region-isolated.

---

**Q16. Can `cloud-kms` sign certificates?**

Yes — `cloud_kms::Action::IssueClientCert` (paid tenant_class) issues X.509 client certificates from a tenant CA. The CA private key
is HSM-bound; the issued cert is signed inside the HSM. Tenants typically use this for SPIFFE SVID issuance or mTLS client certs.

---

**Q17. How is the HSM bootstrapped initially?**

The HSM is provisioned via an offline ceremony: operator quorum cards are generated, the HSM master key is initialised, and
the device is hashed-photographed for inventory. `cloud-kms` then registers the HSM in the HSM-inventory table, recording
the measured-boot baseline + the operator card public keys.

---

**Q18. Can I use Vault Enterprise instead of the platform HSMs?**

Yes — Vault Enterprise with HSM-backed seal can be the CMK backend at paid. The HSM still has to be FIPS 140-3 L3 + CC EAL 4+.
Vault is supported via a `cloud-kms` adapter; see `crates/oya-cloud-kms-adapter-vault-enterprise/`.

---

**Q19. Where does Foundry hook in?**

Foundry pipelines that need to sign deployment artefacts (cosign signatures, in-toto attestations) call `cloud-kms` as the
`oyatie.foundry.<pipeline-id>` principal. The Cedar permits are narrow: only `Sign` + `Verify`; no `Cryptoshred`, no `RotateCmk`,
no `IssueClientCert`. Foundry's signing CMK is itself rotated weekly (paid policy applied to a single tenant).

---

**Q20. How do I roll back a bad rotation?**

If the rotation just happened (within grace window): the previous KEK is still alive. Rollback:
```bash
./bin/oya kms cmk rollback-rotation \
  --tenant <t> \
  --cmk <alias> \
  --to-version <n-1>
```

The previous KEK is restored to "encrypting" state; the new KEK is destroyed. Cedar-gated (`cloud_kms::Action::RollbackRotation`).

If the grace window has passed: the previous KEK is destroyed; data encrypted under it is unrecoverable. There is no rollback
after grace expiry — design rotation cadences with care.
