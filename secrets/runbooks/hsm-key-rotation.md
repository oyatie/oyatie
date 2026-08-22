---
doc_class: Runbook
title: HSM key rotation + HSM compromise response
microservice: cloud-secrets
owner_team: ops-security + axis-cloud-secrets
date: 2026-05-17
severity_default: Sev-2 (routine rotation); Sev-1 (compromise)
---

# Runbook: HSM key rotation + HSM compromise

## When to use

- **Routine KEK rotation**: per pack rotation cadence (365d default).
- **HSM partition unavailability**: PKCS#11 error rate spike; partition heartbeat fail.
- **HSM attestation failure**: daily attestation verification fails.
- **Vendor compromise disclosure**: HSM vendor publishes incident or CVE affecting our partition.

## §A — Routine KEK rotation (Sev-2)

### Pre-flight

```bash
# Verify HSM partitions healthy
cargo run -p cloud-secrets-hsm-integration-app -- partition status --pack <pack>

# Verify Postgres + OpenBao healthy
cargo run -p cloud-secrets-openbao-operator-app -- cluster status --pack <pack>
```

### Step 1 — Generate new KEK in HSM

```bash
cargo run -p cloud-secrets-hsm-integration-app -- kek generate \
    --pack <pack> \
    --alias "kek-<yyyy-mm>" \
    --algorithm AES-256-GCM \
    --witness-1 <ops-security-witness-1-spiffe> \
    --witness-2 <ops-security-witness-2-spiffe>
```

The 4-eye witness model is enforced; OpenBao Sentinel policy `4_eye_approval` is required.

### Step 2 — Re-wrap all DEKs under new KEK

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- kek-rotate \
    --pack <pack> \
    --from-kek "kek-<yyyy-mm-prev>" \
    --to-kek "kek-<yyyy-mm>" \
    --batch-size 100
```

This step is event-driven; OpenBao Transit re-wraps DEKs in batches. Monitor `cloud_secrets_dek_rewrap_completed_total` until matches DEK count.

### Step 3 — Promote new KEK as primary

```bash
cargo run -p cloud-secrets-hsm-integration-app -- kek promote \
    --pack <pack> \
    --alias "kek-<yyyy-mm>"
```

Old KEK retained for 30 days (decrypt-only) to permit late-arriving ciphertext.

### Step 4 — Audit + verify

```bash
cargo run -p audit-chain-app -- query \
    --event-type KekRotated \
    --since "1 hour ago" \
    --filter "pack=<pack>"

# Verify all consumers' resolve operations succeed against new KEK
cargo run -p cloud-secrets-secret-reference-resolver-app -- bench resolve \
    --pack <pack> \
    --duration 5m \
    --acceptance "p99 ≤ 25ms"
```

### Step 5 — Decommission old KEK (t+30d)

```bash
cargo run -p cloud-secrets-hsm-integration-app -- kek decommission \
    --pack <pack> \
    --alias "kek-<yyyy-mm-prev>"
```

This destroys the old KEK in the HSM partition. Confirm via `KekDestroyed` audit event.

## §B — HSM partition unavailability (Sev-2 running; Sev-1 if blocks unseal)

### Diagnosis

```bash
# Check partition heartbeat
cargo run -p cloud-secrets-hsm-integration-app -- partition heartbeat --pack <pack>

# Check PKCS#11 client error rate
# Loki: {namespace="cloud-secrets",app="openbao"} |~ "pkcs11" | json | __error__="" | rate by ()
```

### Step 1 — Confirm HA partition healthy

```bash
cargo run -p cloud-secrets-hsm-integration-app -- partition status \
    --pack <pack> \
    --partition-id <ha-partition-id>
```

### Step 2 — Failover to HA partition

PKCS#11 client config has HA partition listed as fallback; automatic failover should engage. If not:

```bash
cargo run -p cloud-secrets-hsm-integration-app -- partition failover \
    --pack <pack> \
    --from <primary-partition-id> \
    --to <ha-partition-id>
```

OpenBao must restart for unseal to use new partition; rolling restart:

```bash
kubectl -n cloud-secrets-<pack> rollout restart statefulset/openbao
```

Monitor unseal:

```bash
kubectl -n cloud-secrets-<pack> exec openbao-0 -- bao status
```

### Step 3 — Engage vendor on primary partition

Open ticket with HSM vendor (OCI Cloud-HSM or Thales) for diagnostic.

### Step 4 — Sev-1 escalation if both partitions unavailable

If both partitions unreachable: full Sev-1 per `incident-response.md`. Likely requires:
- Vendor incident escalation.
- Decision: wait for partition recovery vs. switch to alternate HSM (different vendor partition; KEK ceremony needed).

## §C — HSM attestation failure (Sev-1)

Daily attestation cron runs at 03:00 UTC. Failure pages immediately.

### Step 1 — Confirm true failure

```bash
cargo run -p cloud-secrets-hsm-integration-app -- attestation verify \
    --pack <pack> \
    --report-id <attestation-report-id> \
    --verbose
```

Possible causes:
- HSM firmware upgrade lag (vendor changes attestation key; benign if announced).
- Attestation public-key rotation by vendor (benign; update verifier config).
- True compromise (alarming; proceed to Step 2).

### Step 2 — Cross-check with vendor

OCI Cloud-HSM: cross-check attestation chain against OCI vendor portal.
Thales Luna: cross-check against Thales attestation registry.

If vendor confirms benign upgrade: update verifier config; document; close.

If vendor cannot explain: proceed to Step 3.

### Step 3 — Assume compromise; full Sev-1

Per `incident-response.md` §"Sev-1 Response: HSM Compromise":
1. Halt new ops to affected partition.
2. KEK ceremony in alternate HSM partition (potentially different vendor).
3. Cascade re-wrap all DEKs under new KEK.
4. Decommission compromised partition.
5. Tenant + regulator notification.

## §D — Vendor compromise disclosure

If HSM vendor publishes a CVE or incident affecting our partition:

### Step 1 — Assess scope

- Which firmware/version is affected?
- Is our partition in the affected version range?
- What is the disclosed risk (key extraction, signing bypass, side-channel)?

### Step 2 — Decision matrix

| Disclosed risk | Action |
|---|---|
| Side-channel timing leak (low practical risk) | Apply firmware update at next maintenance window; document |
| Signing bypass (high practical risk) | Sev-1; switch to alternate partition/vendor; KEK ceremony |
| Key extraction (critical) | Sev-1; assume KEK compromised; full ceremony + cascade re-wrap |

### Step 3 — Communication

Per `incident-response.md` Sev-1 communication tree.

### Step 4 — Vendor switch consideration

If trust in vendor is meaningfully reduced:
- pack-kr can fall back to OCI Cloud-HSM if Thales Luna is the affected vendor.
- Other packs can move between OCI partitions or to alternate vendor.
- Decision involves ops-finance (cost) + ops-legal (residency) + council-privacy.

## Verification (post-rotation)

```bash
# KEK is fresh
cargo run -p cloud-secrets-hsm-integration-app -- kek list --pack <pack>

# Attestation passes
cargo run -p cloud-secrets-hsm-integration-app -- attestation verify --pack <pack>

# Consumer resolve operations succeed
cargo run -p cloud-secrets-secret-reference-resolver-app -- bench resolve \
    --pack <pack> --acceptance "p99 ≤ 25ms"

# Audit-chain has KekRotated + KekAttested events
cargo run -p audit-chain-app -- query \
    --event-type KekRotated --since "1 hour ago"
```

## References

- `microservices/cloud-secrets/threat-model.md` T-I-04 + T-D-02
- `microservices/cloud-secrets/failure-modes.md` FM-02 + FM-10
- `microservices/cloud-secrets/incident-response.md` §"Sev-1 Response: HSM Compromise"
- `microservices/cloud-secrets/policy/data-residency.md` "KEK Lifecycle by Pack"
- OCI Cloud-HSM documentation
- Thales Luna HSM documentation
- NIST SP 800-57 Part 1
