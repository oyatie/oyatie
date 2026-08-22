---
doc_class: Runbook
title: Secret leak detected (Sev-1 ALWAYS)
microservice: cloud-secrets
owner_team: ops-security + axis-cloud-secrets
date: 2026-05-17
severity_default: Sev-1
---

# Runbook: Secret-leak detected

> **Mandatory Sev-1.** A raw secret has been observed in repo, chat, checkpoint, log, or third-party disclosure. Assume the secret is public. Revoke first; investigate after.

## When to use

Triggered by ANY of:
- LEAN-A11 lane fails on a PR with a credential-shaped finding (pre-merge BLOCKER — handle as Sev-2 unless merged; if merged, Sev-1).
- Quarterly retroactive scanner finds a credential-shaped string.
- External responsible-disclosure email.
- Observability anomaly: spike in `cloud_secrets_resolve_total{path="…"}` from unexpected source.
- Tenant or operator reports having pasted a secret in chat / checkpoint / ticket.

## Step 1 — Triage (t+0 → t+5min)

1. Confirm true positive: pattern matched, value resembles a real credential format.
2. Identify the affected secret path(s): the SecretReference URI(s) whose values may have leaked.
3. Classify: if value is confirmed publicly accessible → Sev-1; if only at-risk → Sev-1 (treat as worst case).
4. Open incident in grafana-oncall; assign incident_id ULID; broadcast to `#incident-cloud-secrets`.

## Step 2 — Containment (t+5 → t+15min)

For each affected secret path:

```bash
# Revoke immediately
cargo run -p cloud-secrets-secret-reference-resolver-app -- admin revoke \
    --path "secret/<tenant>/<microservice>/<name>" \
    --incident-id "<ulid>" \
    --reason "leaked-in-<repo|chat|checkpoint|log|disclosure>"

# Verify revocation propagation (consumer SDKs should have flushed within 5s)
cargo run -p cloud-secrets-secret-reference-resolver-app -- admin verify-revocation \
    --path "secret/<tenant>/<microservice>/<name>" \
    --sla-seconds 5
```

Confirm SecretRevoked event sealed in audit-chain:

```bash
cargo run -p audit-chain-app -- query \
    --event-type SecretRevoked \
    --since "5 minutes ago" \
    --filter "incident_id=<ulid>"
```

## Step 3 — Cascade rotation (t+15 → t+30min)

Identify dependents (DEKs encrypted by a revoked KEK; downstream credentials):

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- cascade list \
    --root "secret/<tenant>/<microservice>/<name>"
```

Trigger immediate rotation for the full cascade:

```bash
cargo run -p cloud-secrets-key-rotation-scheduler-app -- rotate \
    --path "secret/<tenant>/<microservice>/<name>" \
    --cascade \
    --priority immediate \
    --incident-id "<ulid>"
```

Cascade completion is event-driven; monitor `cloud_secrets_cascade_rotation_completed_total{incident_id="<ulid>"}` until count matches `cascade list` size.

## Step 4 — Forensic (t+30min → t+24h)

Identify the leak origin:

1. **If repo**: search git history for the credential-shaped string OR its prefix; identify the commit; identify the author + PR. Note: rewriting git history is futile (assume already cloned/cached); rely on revocation.
2. **If chat**: find the agent session id; export the transcript window; identify the prompt that emitted the value. Mask in the canonical transcript export.
3. **If checkpoint**: `.omc/state/sessions/<session_id>/` — find the file; mask; consider that file the leak vector for future patterns.
4. **If log**: identify the consumer µservice + log line; check if `Secret<T>` newtype was bypassed; file LEAN-A11 enhancement.
5. **If external disclosure**: confirm the source; thank-and-track; consider bug bounty if applicable.

Document the leak vector in the post-mortem.

## Step 5 — Tenant notification (t+24h → t+72h max)

If the leaked secret was tenant-scoped (not shared substrate):

1. Identify the tenant + DPA-listed DPO contact at `legal/tenant-dpo-contacts.md`.
2. Send notification using the template below.
3. Audit-emit `tenant_breach_notified{tenant_id_hash, incident_id, notified_at}`.

### Tenant notification template

```
Subject: [oyatie Sev-1 Security Incident] Secret rotation event affecting your tenancy

Dear <DPO Name>,

On <YYYY-MM-DD HH:MM UTC>, our automated secret-leak detection identified
that the following secret reference was potentially exposed:

  Path: secret/<TENANT_HASH>/<MICROSERVICE>/<NAME>
  Class: <SECRET sub-class>
  Detected via: <repo | chat | checkpoint | log | external-disclosure>

Immediate action: we revoked the affected credential within <N> seconds of
detection (<UTC TIMESTAMP>). Cascade rotation of dependent credentials
completed at <UTC TIMESTAMP>. Your tenancy was not interrupted because our
SecretReference SDK transparently re-resolves to the rotated credential.

Per GDPR Art. 33 (and applicable per-pack obligations), this notice is sent
within 72 hours of detection. A detailed post-mortem will be shared by
<DATE + 2 weeks>.

You may request a full audit-chain export of every access to this secret
during the at-risk window <T-MIN, T-DETECTED> by replying to this email.

Regards,
oyatie Security Response (ops-security@oyatie.com)
Incident ID: <ULID>
```

## Step 6 — Regulator notification (per pack SLA)

Per `incident-response.md` §"Regulator Contact Cadence":

- pack-eu: DPA 72h via DPA portal
- pack-kr: PIPC 24h
- pack-us-hc: HHS OCR 60 days for ≥500 affected
- pack-br: ANPD 24h
- pack-au: OAIC 30 days
- pack-in: DPB 72h

Audit-emit `regulator_notified{pack, regulator, incident_id, notified_at}`.

## Step 7 — Post-mortem (t+2 weeks)

Blameless 5-whys + contributing factors. Output:
1. Root cause.
2. LEAN-A11 pattern update PR (mandatory if new pattern caused leak).
3. Process update (training, tool change).
4. Tenant DPA addendum if needed.
5. ADR if architectural change is warranted.
6. Drill cadence update.

Post-mortem lives at `evidence/incidents/<incident_id>/post-mortem.md`. Sealed in audit-chain.

## §"Revocation cascade" — when revoke push isn't propagating

If `cloud_secrets_revocation_propagation_lag_seconds > 5`:

```bash
# List lagging consumers
cargo run -p cloud-secrets-secret-reference-resolver-app -- admin lagging-consumers \
    --path "secret/<tenant>/<microservice>/<name>"

# Force-flush via SDK admin endpoint OR restart consumer pod
kubectl -n <ns> rollout restart deploy/<consumer-deployment>
```

Investigate SSE/WebSocket transport stability if pattern persists.

## §"Residency breach" — cross-pack write attempt detected

If audit-emit shows `cross_pack_write_attempt`:

1. Confirm Cedar deny intercepted (default).
2. Forensic: identify the policy mis-author OR operator action OR code bug.
3. If breach succeeded (unlikely; defence-in-depth should prevent):
   - Quarantine destination namespace.
   - Cryptographic-erase any cross-pack copies.
   - Regulator notification per pack.

## §"Policy mis-author" — OpenBao policy granted over-scope

1. Revert the offending policy.
2. Audit retroactively: query audit-chain for reads under the over-scope window.
3. If leaked-read occurred: convert to Sev-1 raw-secret-leak; per Step 1.
4. Tighten LEAN-A12 pattern; add policy test.

## Verification (post-resolution)

```bash
# Confirm revocation event sealed
cargo run -p audit-chain-app -- verify-seal --incident-id <ulid>

# Confirm cascade complete
cargo run -p cloud-secrets-key-rotation-scheduler-app -- cascade status --incident-id <ulid>

# Run chaos drill at next monthly cadence
```

## References

- `microservices/cloud-secrets/incident-response.md`
- `microservices/cloud-secrets/threat-model.md` T-I-01 + T-I-02
- `microservices/cloud-secrets/failure-modes.md` FM-06 + FM-08 + FM-09 + FM-11
- `microservices/cloud-secrets/policy/secret-isolation.md` §"TI-03 + TI-05"
