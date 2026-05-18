---
doc_class: Runbook
runbook_id: identity-jwks-rotation
microservice: identity
sev: Sev-2 (planned) / Sev-1 (emergency)
owner_team: axis-identity + ops-security
date: 2026-05-18
---

# Runbook: JWKS rotation (scheduled + emergency)

## Scheduled rotation (every 90 days)

### Pre-flight

- Confirm next-key-id available in OpenBao: `oya cloud-secrets show secret/identity/<pack>/jwt-signing-key-next`.
- Verify HSM partition (regulated packs) has spare slot for new key.
- Notify consumer µservices via maintenance-window broadcast 24h prior.

### Procedure

1. `oya identity jwks rotate --pack <pack> --mode scheduled` — initiates rotation.
2. Effect:
   - New `kid` added to JWKS endpoint.
   - Old `kid` remains in JWKS for 24h grace.
   - Zitadel begins signing with new key.
3. Verify within 5min:
   - `curl https://identity-<pack>.oyatie.dev/oauth/v2/keys | jq` — 2 kids present.
   - Issue a test token; verify new `kid` in header.
4. Monitor for 24h:
   - Watch `jwks-availability` SLO.
   - Watch consumer `IdentityOidcVerifyFailed{error=unknown_kid}` rate.
5. After 24h:
   - Remove old `kid` from JWKS.
   - Mark old `kid` for sunset in OpenBao (key remains for forensics 7y).
6. Emit `IdentityJwksRotated(pack, old_kid, new_kid, mode=scheduled)` event.

## Emergency rotation (suspected key compromise)

### Sev-1 PAGE ops-security FIRST

### Procedure

1. `oya identity jwks rotate --pack <pack> --mode emergency` — immediate rotation, NO 24h grace.
2. Effect:
   - New `kid` published.
   - Old `kid` IMMEDIATELY removed from JWKS (no grace).
   - All in-flight tokens signed under old key INVALIDATED.
3. Push notice to consumers: `IdentityJwksEmergencyRotation` broadcast.
4. Force JWKS cache flush on every consumer:
   - `oya identity jwks force-cache-flush --pack <pack> --broadcast`.
5. Force re-authentication of all active sessions:
   - `oya identity sessions force-reauth-all --pack <pack> --reason key-compromise`.
6. Audit cascade: enumerate every token-issuance event under the compromised `kid`; emit alarms for anomalous use.
7. Forensics: lock OpenBao audit-log for the key path; preserve for incident investigation.
8. Communications:
   - Status page entry within 15min.
   - Tenant emergency channel within 30min.
   - GDPR Art. 33 if PII exposure suspected.

### Recovery

- New key signing begins immediately.
- All consumers refresh JWKS within 60s (forced).
- Sessions re-establish over next 15min (user-initiated re-sign-in).
- Pen-test re-validation within 7 days.

## Verification

- `oya identity jwks show --pack <pack>` — only new kid present (emergency) or 2 kids (scheduled).
- `oya identity oidc test-token --pack <pack>` — issuance with new kid succeeds.
- Test consumer verify with both new and (during grace) old kid.
- No `unknown_kid` error rate uptick beyond baseline.

## Rollback (scheduled rotation only — NOT emergency)

If scheduled rotation reveals issue within first 5min:
- Re-add old kid to JWKS (no key change needed; only metadata).
- Roll Zitadel signing back to old kid.
- Investigate; reschedule when fixed.

Emergency rotation CANNOT roll back — the old key is presumed compromised.

## Postmortem trigger

Emergency rotation (Sev-1) → blameless postmortem within 48h; ADR if architectural fix needed.
