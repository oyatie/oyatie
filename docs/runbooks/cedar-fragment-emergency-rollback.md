---
purpose: Oyatie Runbook — Cedar Fragment Emergency Rollback
doc_status: published
---

# Oyatie Runbook — Cedar Fragment Emergency Rollback

> **Status:** Active
> **Owner:** ops-security + axis-policy-engine + council-security
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0243 §D-2, ADR-0243 §D-5, ADR-0243 §D-10, ADR-0247 §D-8, ADR-0248 §D-9

---

## §A Trigger Conditions

Initiate this runbook when **any** of the following occur:

- **Fragment-publisher key compromise** — the signing key used to sign a Cedar fragment (intermediate key chained to the org root per ADR-0243 §D-5) is suspected or confirmed compromised. Any fragment signed by the compromised key must be treated as untrusted.
- **Malicious fragment activation** — a Cedar fragment is found to grant permissions it should not (privilege escalation, allow-all bypass, denial-of-service via over-deny), either due to authoring error or adversarial injection.
- **Anomaly-rollback detector triggered automatically** — the 60s soak-window anomaly detector (per ADR-0248 §D-2 / synthesis §5.2 F5-243-01 fix) reports that denial-rate, latency, or grant-rate has shifted >3σ within the soak window of a newly activated fragment. The detector triggers automatic rollback and pages this runbook for human confirmation.
- **Audit replay reveals unauthorized grants** — post-incident audit replay shows a fragment authorized calls it should not have, and the fragment is still active.

**Note on automatic rollback:** The soak-window detector may already have performed an automatic rollback before this runbook is invoked. In that case, proceed from §B to verify the automatic rollback succeeded, then continue to §C Step 4 onward for the post-automatic-rollback procedure.

---

## §B Pre-Checks

Estimated time: **3–5 min** (urgency-gated; do not delay Step 1 if malicious fragment is actively granting harm).

1. **Identify the fragment(s) to roll back:**
   ```
   psql -c "SELECT fragment_id, scope, version, activated_at, signed_by_key_id,
     status, anomaly_score
     FROM cedar_fragments
     WHERE status IN ('ACTIVE', 'SOAK') AND (
       signed_by_key_id = '<COMPROMISED_KEY_ID>'
       OR fragment_id = '<MALICIOUS_FRAGMENT_ID>'
       OR anomaly_triggered = true
     );"
   ```

2. **Check automatic rollback status** (if anomaly detector fired):
   ```
   psql -c "SELECT fragment_id, rollback_reason, rollback_at, rollback_operator
     FROM cedar_fragment_rollbacks
     WHERE fragment_id = '<FRAGMENT_ID>'
     ORDER BY rollback_at DESC LIMIT 5;"
   ```
   If `rollback_at` is recent (≤5 min), automatic rollback may already be in effect. Verify propagation (Step 2 of §C) before skipping to §C Step 4.

3. **Identify prior safe version** of the fragment:
   ```
   psql -c "SELECT fragment_id, version, activated_at, status
     FROM cedar_fragments
     WHERE fragment_id = '<FRAGMENT_ID>'
     ORDER BY activated_at DESC;"
   ```
   Identify `PRIOR_SAFE_VERSION` — the version immediately before the suspect version that had status `ACTIVE` or `RETIRED` (not `REVOKED`).

4. **Declare incident.** SEV-1 for active malicious grant or key compromise. SEV-2 for automatic anomaly rollback pending human review. Notify `council-security`.

5. **Verify Cedar permit for emergency revocation:**
   ```
   cedar-cli authorize \
     --principal "oyatie.council-security.<operator-id>" \
     --action "Cedar::Action::EmergencyRevokeFragment" \
     --resource "CedarFragment::\"<FRAGMENT_ID>\""
   ```

---

## §C Procedure

### Step 1 — Emergency deactivate suspect fragment(s) (target: ≤60s)

For each fragment to roll back:

```
policy-engine-cli fragment emergency-revoke \
  --fragment-id "<FRAGMENT_ID>" \
  --version "<SUSPECT_VERSION>" \
  --reason "<REASON: key-compromise|malicious-activation|anomaly-rollback>" \
  --operator oyatie.council-security.<operator-id>
```

This command:
- Sets the fragment's status to `EMERGENCY_REVOKED` in the registry.
- Publishes an out-of-band revocation record to the Tier 2 policy-engine (bypassing the 30s snapshot cadence per ADR-0248 §D-9 — emergency push is permitted for revocations).
- The policy-engine pushes an emergency bundle invalidation to all Tier 3 cells immediately.

Emit:
```
audit-emit CedarFragmentEmergencyRevoked \
  --fragment-id <FRAGMENT_ID> \
  --version <SUSPECT_VERSION> \
  --operator oyatie.council-security.<operator-id> \
  --reason "<REASON>"
```

### Step 2 — Verify revocation propagated to all cells (target: ≤60s)

Per ADR-0243 §D-10, standard hot-reload is ≤30s. Emergency push should reach all cells within 30s. Verify:

```
policy-engine-cli fragment verify-revoked \
  --fragment-id "<FRAGMENT_ID>" \
  --all-cells \
  --timeout 90s
```

If any cells are not confirming revocation within 90s, those cells are isolated from the Tier 2 control plane. Escalate to `ops-sre-reliability` to restart the fragment-pull agent on the affected cells:

```
kubectl rollout restart deployment/policy-engine-cache-agent \
  -n policy-engine --context <CELL_CONTEXT>
```

### Step 3 — Revoke compromised signing key (if key compromise is the trigger)

If the trigger is signing-key compromise, ALL fragments signed by the compromised key must be treated as suspect:

```
# List all fragments signed by the compromised key:
psql -c "SELECT fragment_id, version, scope, status
  FROM cedar_fragments WHERE signed_by_key_id = '<COMPROMISED_KEY_ID>'
  AND status = 'ACTIVE';"

# Emergency-revoke each one:
policy-engine-cli fragment emergency-revoke-by-key \
  --signing-key-id "<COMPROMISED_KEY_ID>" \
  --operator oyatie.council-security.<operator-id>
```

Revoke the signing key itself in the key registry:
```
vault write pki/revoke serial_number=<KEY_SERIAL>
cosign revoke --key /hsm/org-root-key <SIGNING_KEY_PUBLIC_KEY_PEM>
```

Rekor revocation entry:
```
cosign attest --predicate /tmp/key-revocation-predicate.json \
  --key /hsm/org-root-key \
  <SIGNING_KEY_ARTIFACT_DIGEST>
```

Record `KEY_REVOCATION_REKOR_ID`.

### Step 4 — Restore prior safe version

Activate the prior safe version of the fragment:

```
policy-engine-cli fragment activate \
  --fragment-id "<FRAGMENT_ID>" \
  --version "<PRIOR_SAFE_VERSION>" \
  --operator oyatie.council-security.<operator-id> \
  --bypass-soak-window true   # emergency restore; soak-window skipped per F5-243-01 intent
```

**Note:** The `--bypass-soak-window` flag is only valid for rollbacks to a previously-active version. New fragment activations always go through the 60s soak window per synthesis §5.2 (F5-243-01 fix).

Verify propagation:
```
policy-engine-cli fragment verify-active \
  --fragment-id "<FRAGMENT_ID>" \
  --version "<PRIOR_SAFE_VERSION>" \
  --all-cells --timeout 90s
```

### Step 5 — Cedar evaluation replay to assess blast radius

Run a post-rollback Cedar evaluation replay to determine which requests were authorized by the now-revoked fragment during its active window. This is the key blast-radius assessment:

```
audit-chain-cli cedar-replay \
  --fragment-id "<FRAGMENT_ID>" \
  --version "<SUSPECT_VERSION>" \
  --window-start "<ACTIVATED_AT>" \
  --window-end "<REVOKED_AT>" \
  --output /tmp/blast-radius-report-<INCIDENT_ID>.json
```

The replay produces a list of: principal, action, resource, decision (PERMIT/DENY), and whether the decision would differ under the prior safe version.

Review unauthorized grants:
```
jq '.grants_that_would_be_denied_under_safe_version' \
  /tmp/blast-radius-report-<INCIDENT_ID>.json | head -50
```

Store in `evidence/incidents/<INCIDENT_ID>/cedar-replay-report.json`.

### Step 6 — Remediate unauthorized actions (if any)

For each action in the blast-radius report that was improperly authorized:

1. If the action was `PublishWorkflowVersion` or `ActivateWorkflowVersion` (self-modification class per ADR-0247 §D-8): immediately invoke `docs/runbooks/self-modification-rollback.md`.
2. If the action was `CompliancePack::Action::*` affecting compliance state: invoke `docs/runbooks/compliance-pack-revocation.md` for the affected packs.
3. For data-access actions (reads of tenant data): assess whether a breach-notification is triggered per the tenant's compliance packs.

---

## §D Verification

1. **Suspect fragment is EMERGENCY_REVOKED on all cells:**
   ```
   policy-engine-cli fragment status --fragment-id "<FRAGMENT_ID>" --all-cells
   ```
   All cells must return `EMERGENCY_REVOKED`.

2. **Prior safe version is ACTIVE on all cells:**
   ```
   policy-engine-cli fragment status \
     --fragment-id "<FRAGMENT_ID>" --version "<PRIOR_SAFE_VERSION>" --all-cells
   ```

3. **Anomaly detector shows green (denial-rate, latency, grant-rate within 3σ):**
   ```
   policy-engine-cli anomaly-status --fragment-id "<FRAGMENT_ID>"
   ```

4. **Audit trail complete:** Verify `CedarFragmentEmergencyRevoked` and `CedarFragmentActivated` (prior version) events with Merkle proofs.

5. **Blast-radius report filed** in `evidence/incidents/<INCIDENT_ID>/`.

---

## §E Rollback

If the emergency revocation itself was erroneous (e.g., the anomaly detector false-fired and the fragment was legitimate):

1. Review the blast-radius report to confirm no unauthorized grants occurred.
2. Re-activate the revoked fragment version through the standard authoring + multispectrum review + soak-window flow (ADR-0243 §D-2 + §D-10). Emergency restore bypasses the soak window; a second activation of the same version does not.
3. Document the false-positive anomaly detector trigger in `evidence/incidents/` and adjust the anomaly detector thresholds if the false-positive was caused by a legitimate traffic shape change (e.g., new feature launch changing grant-rate baseline).

---

## §F Post-Incident

1. Root-cause analysis: how did the malicious/erroneous fragment pass multispectrum review (ADR-0243 §D-8)?
2. If anomaly detector auto-rollback worked: file evidence that the F5-243-01 fix (synthesis §5.2) functioned correctly.
3. If signing key was compromised: initiate `docs/runbooks/meta-trust-root-recovery.md` if the compromised key is part of the meta-trust chain.
4. Author replacement fragment under a new signing key. Route through full multispectrum review before activating.
5. Post-mortem within 72h.
6. Update anomaly detector threshold model if the trigger was a false positive.

---

## §G References

- ADR-0243 §D-2 (Fragment lifecycle)
- ADR-0243 §D-5 (Bootstrap chain of trust — signing key hierarchy)
- ADR-0243 §D-10 (Hot-reload semantics; ≤30s propagation)
- ADR-0247 §D-8 (Self-modification Cedar fragment; meta-permit)
- ADR-0248 §D-9 (Constant-work pattern; emergency push exemption)
- Synthesis §5.2 (F5-243-01: 60s soak window + anomaly-rollback requirement)
- `docs/runbooks/meta-trust-root-recovery.md`
- `docs/runbooks/self-modification-rollback.md`
- `docs/runbooks/compliance-pack-revocation.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
