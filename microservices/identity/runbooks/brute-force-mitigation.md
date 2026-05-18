---
doc_class: Runbook
runbook_id: identity-brute-force-mitigation
microservice: identity
sev: Sev-2 (auto) / Sev-1 (escalated)
owner_team: ops-security + axis-identity
date: 2026-05-18
---

# Runbook: Brute-force mitigation

## Detection signals

- Rate alarm on `/oauth/v2/token` from a specific IP > 100 rps sustained 60s.
- `IdentitySignInFailed` rate > 50 in 5min for a single user.
- Geo anomaly: signin attempts from > 3 countries for same user in 60min.
- WAF deny rate spike from a /24 CIDR.
- TOTP attempt rate > 5 per 60s for a user.

## Automatic mitigations (already in place; runbook confirms)

1. **Per-IP rate limit** on `/oauth/v2/token`: 10 rps; 429 after.
2. **Per-user lockout**: 5 failed factor presentations in 5min → 15min cool-off; subsequent → 30min; permanent escalation to operator after 4 cool-offs.
3. **Edge geo/ASN deny-list**: IP from known botnet ASN → 403 with `X-Block-Reason: asn`.
4. **Coraza WAF**: bot signatures + OWASP CRS v4.x rules.

## Manual investigation

1. **Identify scope**:
   - `oya identity sign-in-failures --since 30m --top 10 --by ip` — top abusing IPs.
   - `oya identity sign-in-failures --since 30m --top 10 --by user_id` — top targeted users.

2. **Confirm not legitimate**:
   - For per-user spike: contact user out-of-band via known channel; ask if they are signing in (legitimate password roll? legitimate device migration?).
   - For per-IP spike: check abuse-db / threat-intel feeds.

3. **Tighten edge filters**:
   - `oya identity edge add-block --ip <cidr> --duration 24h --reason "brute-force-suspected"`.
   - `oya identity edge add-block --asn <asn> --duration 7d --reason "<reason>"`.

4. **Target-user protection**:
   - `oya identity user pin-acr --tenant <t> --user <u> --acr critical --duration 24h` — force hardware-key on next sign-in.
   - Notify user via known channel.

5. **For password-based fallback abuse**:
   - Verify user has a Passkey registered; if not, send guided re-registration.
   - Disable password fallback on the affected account.

## Escalation

If brute-force is targeting > 100 users or sustained > 4h: PAGE ops-security; consider Sev-1.

## Mitigation Sev-1

- Engage upstream DDoS mitigation (Cilium XDP tightening; Cloudflare/upstream CDN if attached).
- Switch identity endpoint to allow-list mode for known-good ASNs only (for affected pack).
- Communicate via status page.

## Recovery

- After attack subsides: keep IP/ASN block in place for 7 days.
- Review affected users; force re-registration of weak credentials (TOTP fallbacks).
- Review WAF deny logs; tighten rules that under-performed.

## Forensic preservation

- Per attack: archive 24h before / during / after to `evidence/identity/incident-brute-force-<id>-<date>.tar.zst`.

## Postmortem trigger

Sev-1 brute-force → blameless postmortem within 7 days; consider ADR if architectural fix needed.
