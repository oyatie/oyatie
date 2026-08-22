---
purpose: Oyatie Runbook — Shamir Share Loss or Holder Coercion
doc_status: published
---

# Oyatie Runbook — Shamir Share Loss or Holder Coercion

> **Status:** Active
> **Owner:** council-security (quorum required)
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during `oya verify` gate repair sweep)
> **Related ADRs:** ADR-0247 §D-8, ADR-0243 §D-5, ADR-0246

---

## §A Trigger Conditions

This runbook handles lost, destroyed, or coerced Shamir-share holders for any of the three meta-level keys that use 5-of-9 Shamir secret sharing (per synthesis §5.5, F5-243-02 / M1-KB-F4 fix):

1. **`meta-trust-root`** — the offline HSM key that signs the self-modification permits gate (ADR-0247 §D-8); see also `docs/runbooks/meta-trust-root-recovery.md`.
2. **Cedar policy root** — the org root key that signs Cedar fragment intermediate keys (ADR-0243 §D-5).
3. **Compliance-pack-publisher root** — the compliance-office Ed25519 key root (ADR-0251 §D-2).

**Share threshold math:** With 5-of-9:
- Up to 4 share losses are **tolerable** — the key can still be reconstituted from the remaining 5.
- 5 or more losses → key is **irrecoverable** without extraordinary measures; escalate to the board immediately.
- Any 1 **coerced share** triggers an immediate rotation ceremony — the coerced share is assumed compromised.

**Do not confuse with 3-of-5 tenant-operational keys** (used for tenant-scoped KMS operations). This runbook covers only the 5-of-9 meta-level keys.

Initiate this runbook when:

- A share holder **reports their share medium lost or destroyed** (hardware security key lost, USB drive destroyed, HSM device failed with no backup).
- A share holder **reports coercion** — government demand, physical threat, blackmail, or legal order to disclose their share.
- A share holder is **unreachable** for >30 days without explanation (treat as potential coercion or loss until resolved).
- A share holder **dies or becomes permanently incapacitated** without having transferred their share medium.
- Security audit reveals a **share medium was exposed** — share data found on a compromised system, photographed, or digitally copied.

---

## §B Pre-Checks

Estimated time: **30 min** (assessment); ceremony scheduling: 1–7 days.

1. **Identify which key(s) are affected.** A share holder may hold shares in multiple key ceremonies. Check the roster:
   ```
   jq '.holders[] | select(.holder_id == "<HOLDER_ID>") | {keys_with_shares, jurisdiction, contact}' \
     evidence/shamir/holder-roster.json
   ```

2. **Count remaining reachable share holders per key.** For each affected key:
   ```
   jq --arg key "<KEY_NAME>" '
     .holders | map(select(.keys_with_shares[] == $key and .status == "reachable")) | length
   ' evidence/shamir/holder-roster.json
   ```

   | Reachable holders | Response |
   |---|---|
   | ≥6 of 9 | Tolerable — schedule ceremony within 30 days |
   | 5 of 9 (threshold exactly met) | Urgent — ceremony within 7 days; any additional loss causes irrecoverability |
   | ≤4 of 9 | **CRITICAL** — key may be irrecoverable; activate board escalation immediately |

3. **Assess coercion risk.** If the trigger is coercion (government demand, legal order, blackmail):
   - Notify `council-legal` immediately.
   - Determine the jurisdiction of the coercion demand (affects which other share holders in the same jurisdiction may be at risk).
   - For government subpoenas: legal counsel must assess whether compliance is required and under what timeline.

4. **Assess jurisdiction distribution after the loss.** A post-loss distribution that concentrates ≥3 remaining shares in a single jurisdiction weakens the multi-jurisdiction coercion protection:
   ```
   jq --arg key "<KEY_NAME>" '
     .holders | map(select(.keys_with_shares[] == $key and .status == "reachable"))
       | group_by(.jurisdiction) | map({jurisdiction: .[0].jurisdiction, count: length})
   ' evidence/shamir/holder-roster.json
   ```
   If any single jurisdiction has ≥3 of the remaining reachable shares, the ceremony must include reassignment to restore balance.

5. **Declare incident.** SEV-1 for coercion or ≤5 reachable holders. SEV-2 for loss with ≥6 reachable holders.

---

## §C Procedure

### Step 1 — Suspend the coerced share (immediate, ≤10 min)

For coercion cases: the coerced share is treated as compromised immediately — even before a new ceremony is held. Mark the share holder's share as suspended in the roster:

```
jq --arg holder "<HOLDER_ID>" --argjson ts "$(date +%s)" '
  (.holders[] | select(.holder_id == $holder)) |= . + {
    status: "suspended",
    suspension_reason: "coercion",
    suspended_at: $ts
  }
' evidence/shamir/holder-roster.json > evidence/shamir/holder-roster-updated.json
mv evidence/shamir/holder-roster-updated.json evidence/shamir/holder-roster.json
```

Emit:
```
audit-emit ShamirShareSuspended \
  --holder-id <HOLDER_ID> \
  --key-name <KEY_NAME> \
  --reason "coercion" \
  --operator oyatie.council-security.<operator-id>
```

**Important:** Suspension does not immediately rotate the key — it records that this share must be excluded from future ceremonies. The key is still valid with 5-of-9 from the remaining shares. Rotation ceremony (Steps 3–5) must follow within 7 days for coercion, 30 days for loss.

### Step 2 — Assess ceremony urgency and schedule

| Remaining reachable (non-suspended) shares | Required ceremony window |
|---|---|
| ≥6 | Within 30 days (standard ceremony) |
| 5 (threshold) | Within 7 days (urgent ceremony) |
| ≤4 | Immediate — attempt emergency partial reconstitution; escalate to board |

Contact all ≥5 available non-suspended share holders to schedule the ceremony. Ensure:
- Participants span ≥3 jurisdictions.
- If coercion was in jurisdiction X, none of the ceremony participants are in jurisdiction X if avoidable.
- Physical or verified-remote attendance confirmed.

### Step 3 — Convene key rotation ceremony

For the affected key(s), convene a full key rotation ceremony per `docs/runbooks/meta-trust-root-recovery.md` §C (the ceremony procedure applies to all 5-of-9 keys, not only the meta-trust-root):

1. Assemble ≥5 non-suspended share holders on the air-gapped ceremony workstation.
2. Reconstitute the current key from ≥5 valid shares (excluding suspended/lost shares).
3. Verify reconstitution produces the correct public key:
   ```
   ssss-combine -t 5 -n 9 | openssl pkey -pubout | diff - evidence/shamir/<KEY_NAME>-public.pem
   ```
   Must produce zero diff.

### Step 4 — Generate replacement share set with jurisdiction rebalancing

Generate a new set of 9 shares for the reconstituted key, with a rebalanced jurisdiction distribution that:
- Excludes the compromised/lost holder.
- Assigns 1–2 replacement holders such that no single jurisdiction holds ≥3 shares.

```
KEY_MATERIAL=$(ssss-combine -t 5 -n 9)   # reconstituted key from Step 3
echo "$KEY_MATERIAL" | ssss-split -t 5 -n 9 -w <KEY_NAME>-ceremony-<DATE>
```

New share assignment:

For each of the 9 new shares:
- Designate a holder from the approved roster.
- Confirm jurisdiction balance: distribute across ≥4 jurisdictions minimum (improvement from the 3-jurisdiction minimum, where feasible).
- Issue each share on a new hardware security key (YubiKey HSM 2 or Thales SafeNet eToken FIPS).

Update the holder roster:
```
# Update evidence/shamir/holder-roster.json with new holder assignments
# Remove the suspended/lost holder; add replacement holder(s)
```

### Step 5 — Destroy old share media

The suspended/lost holder's share medium (if physically recoverable) must be cryptographically wiped or physically destroyed:
- Hardware security key: issue `ykman fido reset` + physical destruction of the device.
- USB-stored share: `shred -vfzu /dev/sdX`; physical destruction of the drive.
- If the share medium is unrecoverable (lost or held by a coercive authority), document this as "share medium at risk — key rotation completed" in the ceremony log.

For all other holders' old share media (they received new shares in Step 4):
- Issue `ykman fido reset` or equivalent for old hardware keys.
- Physical destruction or secure erasure.

### Step 6 — Re-attest dependent trust chains

If the key being rotated is the **Cedar policy root** or **compliance-pack-publisher root**, re-attest all intermediate keys as per `docs/runbooks/meta-trust-root-recovery.md` §C Steps 5–8.

If the key is the **meta-trust-root**, follow Steps 5–8 of `meta-trust-root-recovery.md` to re-sign the self-modification Cedar fragment.

### Step 7 — Document and close

Update the official ceremony record:
```
# Create ceremony evidence:
mkdir -p evidence/shamir/ceremony-<DATE>-<KEY_NAME>/
cat > evidence/shamir/ceremony-<DATE>-<KEY_NAME>/summary.json << EOF
{
  "ceremony_date": "<DATE>",
  "key_name": "<KEY_NAME>",
  "trigger": "<loss|coercion|scheduled>",
  "affected_holder_id": "<HOLDER_ID>",
  "coercion_jurisdiction": "<JURISDICTION_IF_COERCION>",
  "shares_used_for_reconstitution": <N>,
  "new_holder_count": 9,
  "jurisdiction_distribution": {"<J1>": <N1>, "<J2>": <N2>, ...},
  "witnesses": ["<WITNESS_1>", "<WITNESS_2>"],
  "ceremony_lead": "oyatie.council-security.<operator-id>"
}
EOF
```

Emit:
```
audit-emit ShamirCeremonyCompleted \
  --key-name <KEY_NAME> \
  --ceremony-date <DATE> \
  --new-holder-count 9 \
  --operator oyatie.council-security.<operator-id>
```

---

## §D Verification

1. **New share holders can each verify their share decodes to the correct public key:**
   ```
   # Each holder (independently, not in ceremony) verifies their share is non-trivial:
   # (Do NOT combine shares outside of a supervised ceremony — this step verifies
   # the hardware key holds a share, not that the share is correct in isolation)
   ykman otp info  # confirms device has a credential loaded
   ```

2. **Jurisdiction distribution is balanced** (≥3 jurisdictions, no single jurisdiction ≥3 of 9):
   ```
   jq --arg key "<KEY_NAME>" '
     .holders | map(select(.keys_with_shares[] == $key and .status == "reachable"))
       | group_by(.jurisdiction)
       | map({j: .[0].jurisdiction, n: length})
   ' evidence/shamir/holder-roster.json
   ```

3. **Suspended holder is excluded from active roster** (status = "suspended" or "removed").

4. **No single jurisdiction holds ≥3 shares:**
   ```
   jq --arg key "<KEY_NAME>" '
     .holders | map(select(.keys_with_shares[] == $key and .status == "reachable"))
       | group_by(.jurisdiction)
       | map(.count = length) | map(select(.count >= 3)) | length
   ' evidence/shamir/holder-roster.json
   ```
   Must return `0`.

5. **Ceremony evidence archived** in `evidence/shamir/ceremony-<DATE>-<KEY_NAME>/`.

---

## §E Rollback

There is no rollback for a completed Shamir ceremony. If the ceremony produced an incorrect key:
- The error will be detected when trying to use the reconstituted key for operations (e.g., signing a Cedar fragment).
- Reconvene the ceremony immediately with the same share holders to produce the correct reconstitution.
- Do not destroy old share media until the new key is verified operational.

---

## §F Post-Incident

1. **Root-cause documentation:** why was the share lost or was the holder coerced?
2. For coercion cases: assess whether the coercive authority's jurisdiction represents an ongoing systemic risk and whether holder roster should systematically reduce that jurisdiction's share count.
3. Update the holder-roster `last_verified` dates — use this event as an opportunity to re-verify contact info for all holders.
4. Assess whether ≥4 jurisdictions can be maintained going forward (current minimum is 3).
5. Schedule next annual ceremony verification (visual check of all share media without reconstitution).
6. Post-mortem within 72h for coercion cases.

---

## §G References

- ADR-0247 §D-8 (Self-modification gate; meta-trust-root requirement)
- ADR-0243 §D-5 (Cedar bootstrap chain of trust; org root key)
- ADR-0246 (Policy-engine substrate; policy-root key)
- ADR-0251 §D-2 (Compliance-pack-publisher signing; compliance-office key)
- Synthesis §5.5 (F5-243-02: 5-of-9 Shamir ≥3 jurisdictions mandate)
- `docs/runbooks/meta-trust-root-recovery.md`
- `docs/runbooks/cedar-fragment-emergency-rollback.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
