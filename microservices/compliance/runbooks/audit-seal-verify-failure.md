# Runbook — Audit-chain seal verify failure (Sev-1)

## Trigger

`EVT-AUDIT-SEAL-VERIFY-FAILED` event fires.

## Immediate actions (≤ 15 minutes)

1. Ack page.
2. **Flag** affected artifacts in auditor portal banner.
3. **DO NOT** delete the artifacts (forensic value).
4. Notify axis-security on-call.

## Triage (≤ 1 hour)

1. Check seal chain continuity: walk `prev_seal_hex` for the affected artifact range.
2. Check cosign trust root validity (`policy/cosign-trust-root.json`).
3. Check Sigstore Fulcio + Rekor reachability.
4. Check operator-cluster OIDC issuer mirror (fallback per ADR-0164).

## Possible root causes

| Root cause | Action |
|---|---|
| OIDC issuer rotated keys without trust root update | Update `policy/cosign-trust-root.json`; redeploy |
| Sigstore Fulcio outage | Switch to fallback issuer; re-seal affected artifacts |
| Rekor log gap | File issue with sigstore-community; rely on operator mirror |
| Artifact tampered post-emit | Sev-1 security incident — see threat-model A1 |

## Remediation

Depends on root cause; for tampered artifact:
1. Quarantine artifact bucket.
2. Forensic investigation.
3. Notify affected auditors.
4. Re-emit from cold-tier original (per IP-012).

## Cross-references

- IP-005 — audit-chain seal coverage.
- ADR-0181 — cosign keyless OIDC.
- threat-model.md A1.
