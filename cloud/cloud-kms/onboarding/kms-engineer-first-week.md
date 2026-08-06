# KMS Engineer — First Week on `cloud-kms`

Audience: a security engineer with HSM + PKCS#11 + AWS KMS experience joining the `oya-cloud-kms-*` lane.
Goal: by Friday EOD you can mint a CMK, issue + use a DEK via envelope encryption, rotate a KEK, and walk a cryptoshredding cycle.

## Day 1 — read before touching

- `docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md` — §D-10 binds the cryptography invariants.
- `docs/decisions/ADR-0702-identity-authz-live-apex.md` — tenant CMK isolation.
- `docs/decisions/ADR-0709-general-live-apex.md — Foundry as a tenant of `cloud-kms` for its own signing.
- `docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md` — §D-4 distinguishes encryption-key BYOK from provider BYOK.
- `microservices/cloud-kms/retired tenant_class adoption artifact` — the four tenant_classes and HSM backends.
- NIST FIPS 140-3, FIPS 203 (ML-KEM), FIPS 204 (ML-DSA) — at least skim the security policies of the certified modules.

Clone:
```bash
./bin/oya git worktree-add --base dev --branch onboarding/$USER-kms-week1 .worktrees/$USER-kms-week1
cd .worktrees/$USER-kms-week1
```

## Day 2 — bring up a loopback cloud-kms cell

The dev profile uses a **software-only HSM simulator** (SoftHSM 2.6.1) — never use this for real key material.

```bash
make dev-cell.up CELL=kms-loopback-1 PROFILE=cloud-kms-dev
make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid
```

Mint your first CMK:
```bash
./bin/oya kms cmk create \
  --tenant oyatie.b2b.smb.acme-software \
  --alias acme-default \
  --algorithm AES-256-GCM \
  --rotation-cadence 30d
```

Expected output:
```
cmk_id            : cmk-2026-05-20-9ab2…
alias             : acme-default
algorithm         : AES-256-GCM
hsm_partition     : softhsm-loopback/partition-0
fips_conformance  : none (dev)
rotation_cadence  : 30d
next_rotation_at  : 2026-06-19T00:00:00Z
audit_chain_event : ce-2026-05-20T12:01:33Z-…
```

## Day 3 — envelope encryption hands-on

Issue a DEK (data-encryption-key) under your CMK:
```bash
./bin/oya kms dek issue \
  --tenant oyatie.b2b.smb.acme-software \
  --cmk acme-default \
  --purpose "encrypt: acme-tasks-row-12345" \
  --plaintext-length 32   # bytes; for AES-256 always 32
```

Output:
```
dek_id             : dek-2026-05-20-c8d1…
plaintext_key      : (32 bytes; redacted from log)
ciphertext_key     : (encrypted-under-CMK; 60 bytes; safe to store)
key_envelope_aad   : "tenant_id=oyatie.b2b.smb.acme-software;purpose=encrypt:acme-tasks-row-12345"
```

The `ciphertext_key` is what you store next to the encrypted record. To decrypt later:
```bash
./bin/oya kms dek decrypt \
  --tenant oyatie.b2b.smb.acme-software \
  --ciphertext-key <base64> \
  --aad "tenant_id=oyatie.b2b.smb.acme-software;purpose=encrypt:acme-tasks-row-12345"
```

The AAD (additional authenticated data) is **mandatory** — a missing AAD or a mismatched AAD refuses decryption.

## Day 4 — KEK rotation

Trigger an on-demand rotation:
```bash
./bin/oya kms cmk rotate \
  --tenant oyatie.b2b.smb.acme-software \
  --cmk acme-default \
  --reason "policy: 30d cadence"
```

Behind the scenes:
1. `cloud-kms` mints a new KEK material inside the HSM partition.
2. The previous KEK is marked **decrypt-only** (cannot encrypt new DEKs).
3. The CMK metadata is bumped (`version: N → N+1`).
4. An `audit_chain` event is emitted: `cloud_kms.cmk.rotated`.
5. After the policy-defined grace period (default 90 d for paid), the previous KEK is **destroyed** — payloads encrypted under
   it become unrecoverable. This is the cryptoshred timeline; understand it before rotating.

Verify rotation:
```bash
./bin/oya kms cmk show --tenant oyatie.b2b.smb.acme-software --cmk acme-default
```

You should see `version: 2`, `previous_kek_ids: [...]`, `previous_kek_destroy_at: 2026-08-18T00:00:00Z`.

## Day 5 — cryptoshredding drill

Cryptoshredding is destructive. The dev profile uses a "throwaway" CMK so the drill is safe:

```bash
./bin/oya kms cmk create \
  --tenant oyatie.b2b.smb.acme-software \
  --alias drill-throwaway \
  --algorithm AES-256-GCM \
  --policy "dev-exportable" \
  --rotation-cadence none
```

Encrypt something:
```bash
./bin/oya kms dek issue --tenant oyatie.b2b.smb.acme-software --cmk drill-throwaway --purpose "drill"
echo "secret-payload-do-not-keep" | ./bin/oya kms encrypt --tenant oyatie.b2b.smb.acme-software --cmk drill-throwaway --aad "purpose=drill" --output drill.enc
```

Cryptoshred:
```bash
./bin/oya kms cryptoshred \
  --tenant oyatie.b2b.smb.acme-software \
  --cmk drill-throwaway \
  --reason "tutorial drill" \
  --confirm-irreversible
```

The Cedar permit `cloud_kms::Action::Cryptoshred` is gated; reviewer-agent escalation fires at paid tenant_class (drill bypasses on dev profile).
A signed cryptoshred receipt is anchored to `audit-chain` within ≤ 30 min (paid SLO).

Attempt to decrypt:
```bash
./bin/oya kms decrypt --tenant oyatie.b2b.smb.acme-software --input drill.enc
```

Expected: `KmsError::CmkCryptoshredded { cmk_id }`. The payload is now unrecoverable.

## What "done with week 1" means

- [ ] You can recite the four tenant_classes and which HSM backend each uses.
- [ ] You minted, used, and rotated a CMK end-to-end.
- [ ] You walked an envelope-encryption flow with AAD binding.
- [ ] You performed (and understood) a cryptoshred operation.
- [ ] You read ADR-0251 §D-10 + FIPS 140-3 IG.
- [ ] You can explain the difference between encryption-key BYOK (ADR-0251) and provider BYOK (ADR-0255 §D-4).

## Rookie traps

1. **Encrypting with the CMK directly.** The CMK never encrypts payloads; it encrypts DEKs. The DEK encrypts payloads. Use envelope mode.
2. **Skipping AAD.** A DEK decrypted without AAD verification opens up tenant-key cross-use; the SDK refuses but raw API misuse is possible.
3. **Forgetting the rotation grace period.** Rotating a CMK without understanding the grace window can break decryption of older
   records when the previous KEK is destroyed.
4. **Cryptoshredding without confirm-irreversible.** The CLI refuses without `--confirm-irreversible`; do not script around it.
5. **Mixing PQC and classical algorithms.** Hybrid mode (X25519 + ML-KEM) is the paid-tenant_class transit default; do not roll your own
   PQC integration — use `cloud-kms` primitives.
6. **Importing private keys.** PKCS#11 import is paid tenant_class only and refused without a governance ticket; never try at demo_trial.
