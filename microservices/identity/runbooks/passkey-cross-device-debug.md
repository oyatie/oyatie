---
doc_class: Runbook
runbook_id: identity-passkey-cross-device-debug
microservice: identity
sev: Sev-3
owner_team: axis-identity + axis-developer-experience
date: 2026-05-18
---

# Runbook: Passkey cross-device sign-in (caBLE) debug

## Symptom

User on desktop tries to sign in with Passkey synced to their mobile device. The QR-code flow (caBLE: Cloud-Assisted BLE Endpoint) does not complete.

## Common failure modes

| Symptom | Cause | Fix |
|---|---|---|
| QR code does not appear | Browser too old; WebAuthn L3 not supported | Upgrade browser to Chrome 109+ / Safari 16+ / Firefox 122+ |
| QR scans but mobile shows "no matching key" | Mobile Passkey not synced to the user's Apple/Google account currently signed-in on the phone | Sign in to the correct Apple/Google account on mobile |
| Scan completes but desktop hangs | Bluetooth not paired between desktop and mobile (caBLE handshake fails) | Enable BT on desktop; enable BT on mobile; retry |
| "Network error" mid-ceremony | Cloud relay endpoint (Apple/Google) blocked by corporate firewall | IT must allowlist Apple's iCloud Keychain endpoints + Google's Passkey backend |
| Desktop browser cancels after 90s | timeout exceeded (challenge expired) | Retry; cause was likely BT pairing delay |
| Mobile shows "Verify required" loop | Mobile lock-screen biometric failed; user must enter passcode | Use passcode fallback |

## Diagnostics

1. **Browser console**: open DevTools; look for `NotAllowedError`, `InvalidStateError`, `NotSupportedError`.
2. **Server-side challenge log**:
   - `oya identity webauthn challenge-trace --user <user_id> --since 5m` — shows challenge issuance, expiry, and any finish attempt.
3. **Mobile-side**: check user's authenticator UI for declined operations.
4. **Network**: check whether outbound HTTPS to `*.icloud.com` / `*.google.com` is allowed from desktop network.

## Mitigation

- Provide user with a YubiKey (USB or NFC) as a non-caBLE alternative.
- Document this in tenant's per-user onboarding playbook.

## Long-term: corporate-network compatibility

For tenants with locked-down networks, the recommended posture is:
- Device-bound Passkey on each user's primary device (no sync; no caBLE needed).
- Hardware key (YubiKey) as backup.
- This eliminates caBLE dependency entirely.

## Cross-references

- ADR-0188 §"Cross-device sign-in (caBLE)"
- W3C WebAuthn Level 3 §"Hybrid transport"

## Postmortem trigger

If caBLE failure affects > 20% of attempts for a tenant: review tenant's network policy with their IT.
