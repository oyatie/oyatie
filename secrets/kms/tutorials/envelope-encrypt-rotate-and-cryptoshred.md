# Tutorial — Envelope-encrypt a payload, rotate the KEK, cryptoshred

Goal: walk a full lifecycle of a tenant CMK — mint, envelope-encrypt a payload, store ciphertext + ciphertext-DEK, rotate the
KEK, verify decryption still works, cryptoshred, verify decryption now fails. End-to-end on a loopback `cloud-kms` cell.

Pre-reqs:
- Loopback kms cell: `make dev-cell.up CELL=kms-loopback-1 PROFILE=cloud-kms-dev`
- Tenant: `make dev-tenant.create T=oyatie.b2b.smb.acme-software TENANT_CLASS=paid`
- `jq`, `openssl` on PATH.

## Step 1 — mint a CMK

```bash
./bin/oya kms cmk create \
  --tenant oyatie.b2b.smb.acme-software \
  --alias acme-customer-pii \
  --algorithm AES-256-GCM \
  --rotation-cadence 30d \
  --grace-window 90d \
  --policy pii-data
```

Expected:
```
cmk_id            : cmk-2026-05-20-9ab2…
alias             : acme-customer-pii
algorithm         : AES-256-GCM
hsm_partition     : softhsm-loopback/partition-0
rotation_cadence  : 30d
grace_window      : 90d
next_rotation_at  : 2026-06-19T00:00:00Z
audit_chain_event : ce-2026-05-20T13:01:18Z-…
```

## Step 2 — envelope-encrypt a payload

We'll encrypt a customer PII row (synthetic).

```bash
PAYLOAD='{"customer_id":"cust-42","email":"jane@example.com","ssn":"123-45-6789"}'
echo "$PAYLOAD" > pii.json

./bin/oya kms encrypt \
  --tenant oyatie.b2b.smb.acme-software \
  --cmk acme-customer-pii \
  --aad "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-42" \
  --input pii.json \
  --output pii.enc
```

`pii.enc` is a binary blob containing:
- `header`: version (1 byte) + cmk_id (32 bytes) + kek_version (4 bytes) + iv (12 bytes)
- `ciphertext_dek`: 60 bytes (DEK encrypted under the KEK)
- `ciphertext`: payload bytes encrypted with the plaintext DEK
- `tag`: 16-byte GCM tag

Verify the structure:
```bash
./bin/oya kms inspect --input pii.enc
```

Output:
```
format_version   : 1
cmk_id           : cmk-2026-05-20-9ab2…
kek_version      : 1
aead_algo        : AES-256-GCM
payload_length   : 73 bytes
aad              : tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-42
```

## Step 3 — decrypt + verify

```bash
./bin/oya kms decrypt \
  --tenant oyatie.b2b.smb.acme-software \
  --aad "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-42" \
  --input pii.enc \
  --output pii.dec.json

diff pii.json pii.dec.json && echo "round-trip OK"
```

Mismatched AAD must fail:
```bash
./bin/oya kms decrypt \
  --tenant oyatie.b2b.smb.acme-software \
  --aad "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-99" \
  --input pii.enc \
  --output bad.dec.json
```
Expect:
```
ERROR: KmsError::AeadAuthFailure { reason: "aad mismatch" }
```

## Step 4 — rotate the KEK

Force an on-demand rotation:
```bash
./bin/oya kms cmk rotate \
  --tenant oyatie.b2b.smb.acme-software \
  --cmk acme-customer-pii \
  --reason "tutorial: demonstrate rotation"
```

Expected:
```
cmk_id              : cmk-2026-05-20-9ab2…
previous_kek_version: 1
new_kek_version     : 2
previous_kek_status : decrypt-only
previous_kek_destroy_at: 2026-08-18T13:01:18Z (90d grace)
audit_chain_event   : ce-2026-05-20T13:04:42Z-…
```

## Step 5 — verify old ciphertext still decrypts

```bash
./bin/oya kms decrypt \
  --tenant oyatie.b2b.smb.acme-software \
  --aad "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-42" \
  --input pii.enc \
  --output pii.dec.json

diff pii.json pii.dec.json && echo "decrypt-after-rotation OK"
```

The new KEK (version 2) is now the encrypting key; KEK version 1 is decrypt-only and still alive (90 d grace).

## Step 6 — re-encrypt with the new KEK (optional)

To migrate the ciphertext to the new KEK:
```bash
./bin/oya kms reencrypt \
  --tenant oyatie.b2b.smb.acme-software \
  --input pii.enc \
  --output pii.enc.v2 \
  --aad "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-42"
```

`pii.enc.v2` carries `kek_version: 2`. Useful before cryptoshredding because it lets you advance the protection without changing
the AAD or the plaintext.

## Step 7 — cryptoshred

This is destructive. We've already shown round-trip works; now we destroy the CMK.

```bash
./bin/oya kms cryptoshred \
  --tenant oyatie.b2b.smb.acme-software \
  --cmk acme-customer-pii \
  --reason "tutorial: demonstrate cryptoshredding" \
  --confirm-irreversible
```

Expected:
```
cmk_id                  : cmk-2026-05-20-9ab2…
state_before            : Active (kek_versions=[1 decrypt-only, 2 encrypting])
state_after             : Cryptoshredded
hsm_destroy_attestations: 1 partition; signed receipts captured
propagation_deadline    : 2026-05-20T13:34:18Z (≤ 30 min for paid)
audit_chain_event       : ce-2026-05-20T13:09:11Z-…
```

## Step 8 — verify decryption now fails

```bash
./bin/oya kms decrypt \
  --tenant oyatie.b2b.smb.acme-software \
  --aad "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-42" \
  --input pii.enc \
  --output pii.dec.json
```

Expected:
```
ERROR: KmsError::CmkCryptoshredded { cmk_id: "cmk-2026-05-20-9ab2…", at: "2026-05-20T13:09:11Z" }
```

The ciphertext is now mathematically unrecoverable. This is the GDPR Art. 17 / CCPA / KR PIPA right-to-delete primitive.

## Step 9 — verify the cryptoshred receipt on the audit chain

```bash
./bin/oya audit-chain query \
  --tenant oyatie.b2b.smb.acme-software \
  --kind cloud_kms.cryptoshred.completed \
  --since "1h ago"
```

The event includes the HSM attestation signature; you can extract it for compliance evidence:
```bash
./bin/oya audit-chain extract-receipt \
  --tenant oyatie.b2b.smb.acme-software \
  --event-id ce-2026-05-20T13:09:11Z-… \
  --output receipts/acme-customer-pii-cryptoshred.json
```

The receipt is self-contained: HSM measured-boot quote + cryptoshred command + zeroization confirmation + BLAKE3 chain anchor.

## What you just demonstrated

- Envelope encryption with AAD binding (mandatory in v0.42+).
- KEK rotation with a 90-day grace window — old ciphertexts still readable until grace expiry.
- Re-encryption to advance ciphertext to the latest KEK (optional but recommended before cryptoshred).
- Cryptoshredding as a first-class destructive Cedar action.
- HSM attestation receipts for cryptoshred — compliance evidence.
- BLAKE3 audit chain — every step recoverable from the chain.
