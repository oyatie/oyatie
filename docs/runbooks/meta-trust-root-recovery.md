---
purpose: Oyatie Runbook — Meta-Trust-Root Key Loss or Compromise Recovery
doc_status: published
---

# Oyatie Runbook — Meta-Trust-Root Key Loss or Compromise Recovery

> **Status:** Active
> **Owner:** council-security (quorum required)
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0247 §D-8, ADR-0243 §D-5, ADR-0247 §§5.1 (synthesis), ADR-0246

---

## §A Trigger Conditions

This runbook handles recovery from loss or compromise of the `oyatie.foundry.meta-trust-root` signing key — the separately-rooted key introduced by synthesis §5.1 (F5-247-01 fix) to break the circular `is_automated_with_baseline_signed_workflow` predicate in ADR-0247 §D-8.

The `meta-trust-root` key:
- Lives in an **offline HSM** (never network-connected in operational state).
- Is Shamir-shared **5-of-9** across ≥3 geographic jurisdictions (per synthesis §5.5, F5-243-02 / M1-KB-F4 fix; raised from 3-of-5).
- Signs the 2-human-approval gate in `platform-self-modification-permits.cedar` when `is_human_approval_present(min_approvers: 2)` is not available.
- Is distinct from the `org-baseline-key` (which signs day-to-day Cedar fragments) and from the `compliance-office` key (which signs compliance packs).

**Trigger conditions:**

- **Key loss** — a Shamir-share holder is uncontactable, deceased, or has lost their share medium. With 5-of-9 threshold, up to 4 losses are tolerable before threshold drops below quorum. Initiate this runbook when ≥3 shares are confirmed lost or uncontactable.
- **Key compromise** — a share holder reports coercion, their share medium is recovered by an adversary, or a share is found on a compromised system. Initiate immediately on any single confirmed share compromise (rotation is required before threshold erosion).
- **HSM failure** — the offline HSM holding the assembled key material fails. Initiate if the HSM cannot be repaired by the HSM vendor within 30 days.
- **Annual ceremony** — the meta-trust-root ceremony is scheduled annually. This runbook covers the key rotation portion of the ceremony.

**Do NOT use this runbook for:**
- Day-to-day Cedar fragment signing (use `org-baseline-key` ceremony).
- Compliance pack signing (use `compliance-office` key ceremony).
- Tenant KMS root rotation (see `docs/runbooks/byok-rotation-encryption-tenant-duress.md`).

---

## §B Pre-Checks

Estimated time: **1–3 days** (human coordination across ≥3 jurisdictions). This is a planned ceremony, not a 60-minute runbook. However, the operational steps once share-holders are assembled are time-bounded below.

1. **Assess share-holder status.** Contact all 9 share holders to determine available count:
   ```
   # Internal roster at evidence/meta-trust-root/shamir-holder-roster.json (access: council-security quorum)
   jq '.holders[] | {id, jurisdiction, contact, last_verified}' \
     evidence/meta-trust-root/shamir-holder-roster.json
   ```

2. **Confirm threshold is met.** Count confirmed reachable share holders. Must be ≥5 to proceed with key reconstitution. If <5, stop and escalate to the board for emergency share-holder replacement before proceeding.

3. **Verify offline HSM integrity.** The offline HSM is at the primary ceremony location. Before transport:
   - Verify physical tamper-evident seals (document serial numbers).
   - Verify HSM's own attestation report matches expected firmware hash.
   - Chain of custody documentation reviewed by ≥2 council-security members.

4. **Jurisdiction distribution check.** Confirm that the ≥5 participating share holders span ≥3 jurisdictions:
   ```
   jq '[.holders[] | select(.available == true) | .jurisdiction] | unique | length' \
     evidence/meta-trust-root/shamir-holder-roster.json
   ```
   Must be ≥3. If <3, the ceremony must be deferred until jurisdiction diversity is restored (to prevent compelled disclosure under a single jurisdiction's legal authority).

5. **Declare incident severity.** SEV-1 for active compromise. Planned ceremony: file as change request with council-security + council-architecture approval.

---

## §C Procedure

### Step 1 — Ceremony logistics (target: 1–3 days lead time)

Assemble ≥5 of 9 share holders at the ceremony location (or via secure multi-party ceremony protocol for remote participants). Requirements:
- Physical attendance preferred; remote attendance via encrypted video with identity verification acceptable.
- Each share holder presents government-issued ID + their share medium.
- ≥2 independent witnesses from council-security present throughout.
- All phones and network-connected devices quarantined during key reconstitution.
- Video recording of the ceremony stored in `evidence/meta-trust-root/ceremony-<DATE>/`.

### Step 2 — Reconstitute the current meta-trust-root key from Shamir shares (target: ≤60 min)

On the air-gapped ceremony workstation (no network connectivity; verified with Faraday cage or physical network-cable removal):

```
# Shamir reconstitution (using ssss or equivalent FIPS-compliant tool):
ssss-combine -t 5 -n 9
# Input: 5 of the 9 share holders enter their shares in sequence.
# Output: Reconstituted key material in memory only — never write to disk.
```

Load reconstituted key into offline HSM:
```
hsm-cli key import \
  --mechanism AES-KEY-WRAP \
  --label "meta-trust-root-current-<DATE>" \
  --extractable false \
  --sensitive true
```

### Step 3 — Generate new meta-trust-root key (target: ≤30 min)

Generate a new Ed25519 key pair on the HSM:

```
hsm-cli key generate \
  --mechanism ED25519 \
  --label "meta-trust-root-<NEW_DATE>" \
  --extractable false \
  --sensitive true \
  --id <NEW_KEY_ID>
```

Extract the public key for registration (private key never leaves HSM):
```
hsm-cli key export-public \
  --label "meta-trust-root-<NEW_DATE>" \
  --format PEM \
  > /tmp/meta-trust-root-<NEW_DATE>.pub
```

### Step 4 — Split new key into new 5-of-9 Shamir shares (target: ≤60 min)

Export key material to air-gapped memory, split into 9 shares across ≥3 jurisdictions:

```
# Extract key material to memory only (not disk):
KEY_MATERIAL=$(hsm-cli key export-private --label "meta-trust-root-<NEW_DATE>" --format raw)

# Split into 5-of-9 shares:
echo "$KEY_MATERIAL" | ssss-split -t 5 -n 9 -w meta-trust-root-<NEW_DATE>
```

Distribute shares:
- Each of the 9 designated share holders receives their share on a hardware security key (YubiKey HSM 2 or equivalent FIPS 140-2 L3 device).
- Share holders span at minimum: KR (2 holders), EU (2 holders), US (2 holders), remaining 3 in other jurisdictions or distributed across existing.
- Document each holder's share number (not the share content) in `evidence/meta-trust-root/shamir-holder-roster-<NEW_DATE>.json`.

### Step 5 — Issue new intermediate signing keys chained to new meta-trust-root (target: ≤60 min)

The meta-trust-root signs a new set of intermediate keys. These are the keys used in day-to-day operations:

```
# For each intermediate key (org-baseline-key, workflow-publisher-key, fragment-author-key):
hsm-cli cert issue \
  --signing-key "meta-trust-root-<NEW_DATE>" \
  --subject-key <INTERMEDIATE_KEY_ID> \
  --validity-days 365 \
  --purpose "self-modification-gate" \
  --output /tmp/<INTERMEDIATE_KEY_NAME>-cert-<NEW_DATE>.pem
```

Publish the new intermediate key certificates to the key registry and Sigstore Rekor:

```
cosign upload blob --key /hsm/meta-trust-root-<NEW_DATE> \
  /tmp/org-baseline-key-cert-<NEW_DATE>.pem
```

### Step 6 — Re-attest the self-modification Cedar fragment (target: ≤30 min)

The `platform-self-modification-permits.cedar` fragment (ADR-0247 §D-8) references the meta-trust-root via `is_signed_with_org_root_key_intermediate`. Re-sign the fragment under the new intermediate key:

```
policy-engine-cli fragment re-sign \
  --fragment-id "baseline/platform-self-modification-permits.cedar" \
  --new-signing-key-id <NEW_INTERMEDIATE_KEY_ID> \
  --operator oyatie.council-security.<ceremony-lead>
```

This requires ≥3 human approvers (council-security + council-architecture per ADR-0247 §D-8 meta-permit gate):

```
policy-engine-cli fragment approve \
  --fragment-id "baseline/platform-self-modification-permits.cedar" \
  --approver-1 oyatie.council-security.<approver-1-id> \
  --approver-2 oyatie.council-security.<approver-2-id> \
  --approver-3 oyatie.council-architecture.<approver-3-id>
```

Activate the re-signed fragment:
```
policy-engine-cli fragment activate \
  --fragment-id "baseline/platform-self-modification-permits.cedar" \
  --bypass-soak-window false   # soak window applies; ≥60s required
```

Wait ≥60s soak window, then verify anomaly detector green.

### Step 7 — Revoke old meta-trust-root and publish revocation

In the key registry, mark the old meta-trust-root as revoked:

```
psql -c "UPDATE signing_keys SET status = 'REVOKED', revoked_at = now(),
  revocation_reason = '<REASON: scheduled-rotation|compromise>',
  revocation_ceremony_evidence_ref = 'evidence/meta-trust-root/ceremony-<DATE>/'
  WHERE key_label = 'meta-trust-root-<OLD_DATE>';"
```

Publish Rekor revocation:
```
cosign attest \
  --predicate /tmp/meta-trust-root-revocation-predicate.json \
  --key /hsm/meta-trust-root-<NEW_DATE> \
  <OLD_META_TRUST_ROOT_PUBLIC_KEY_ARTIFACT_DIGEST>
```

### Step 8 — Down-chain re-attestation

All artifacts previously signed by old intermediate keys must be re-attested under new intermediate keys. This is a batched job:

```
microservices/policy-engine/bin/reatttest-chain \
  --old-root-key-label "meta-trust-root-<OLD_DATE>" \
  --new-root-key-label "meta-trust-root-<NEW_DATE>" \
  --artifact-types "cedar-fragment,workflow-version,substrate-artifact" \
  --batch-size 100 \
  --progress
```

---

## §D Verification

1. **New meta-trust-root is ACTIVE in key registry:**
   ```
   psql -c "SELECT key_label, status, issued_at FROM signing_keys
     WHERE key_label LIKE 'meta-trust-root-%' ORDER BY issued_at DESC;"
   ```

2. **Self-modification fragment is active under new key chain:**
   ```
   policy-engine-cli fragment verify-signature \
     --fragment-id "baseline/platform-self-modification-permits.cedar" \
     --expected-root-key "meta-trust-root-<NEW_DATE>"
   ```

3. **Anomaly detector green after 60s soak:**
   ```
   policy-engine-cli anomaly-status \
     --fragment-id "baseline/platform-self-modification-permits.cedar"
   ```

4. **Jurisdiction distribution of new shares verified** (≥3 jurisdictions):
   Confirm from ceremony documentation.

5. **All 9 new share holders have confirmed receipt** of their share medium.

6. **Ceremony evidence archived:**
   ```
   ls evidence/meta-trust-root/ceremony-<DATE>/
   # Expected: video.enc, holder-roster.json, witness-attestations/, rekor-entries.json
   ```

---

## §E Rollback

Key rotation ceremonies cannot be rolled back once old shares are destroyed. Before old shares are destroyed:

- If Step 6 fails (fragment re-signing fails quorum), abort: keep old meta-trust-root active, destroy new key material, reschedule ceremony.
- If Step 7 is not yet executed, the old root is still valid; do not destroy new material until re-attestation (Step 8) is complete.

---

## §F Post-Incident

1. Update `evidence/meta-trust-root/shamir-holder-roster.json` with new holder assignments.
2. Schedule next annual ceremony at `<NEW_DATE> + 365 days`.
3. File ceremony completion evidence in `evidence/meta-trust-root/ceremony-<DATE>/` with Merkle proof.
4. If triggered by compromise: full security review of how the share was compromised; file in `evidence/incidents/`.
5. Update `docs/runbooks/shamir-share-loss-or-coercion.md` if new holder roster affects its guidance.

---

## §G References

- ADR-0247 §D-8 (Self-modification Cedar fragment; meta-trust-root requirement)
- ADR-0243 §D-5 (Bootstrap chain of trust)
- ADR-0246 (Policy-engine substrate)
- Synthesis §5.1 (F5-247-01: separately-rooted meta-trust-root requirement)
- Synthesis §5.5 (F5-243-02: 5-of-9 Shamir ≥3 jurisdictions)
- `docs/runbooks/cedar-fragment-emergency-rollback.md`
- `docs/runbooks/shamir-share-loss-or-coercion.md`
- `docs/runbooks/bootstrap-ci-compromise.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
