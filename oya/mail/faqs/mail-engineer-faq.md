---
doc_class: FAQ
microservice: mail
persona: mail-engineer + deliverability-engineer + dkim-spf-dmarc-engineer
related_adrs: [ADR-MAIL-001, ADR-MAIL-0001, ADR-MAIL-0002, ADR-MAIL-0003, ADR-MAIL-0004]
date: 2026-05-20
doc_status: published
---

# Mail Engineer FAQ — mail

## Why per-tenant DKIM key custody instead of a platform-wide signing key?

Per ADR-MAIL-001 § Alternatives Considered. A platform-wide DKIM key would:

1. Allow one compromise to affect all tenants' email reputation.
2. Make it impossible to prove per-tenant custody separation for healthcare (HIPAA), finance (SOX), or government (FedRAMP) tenants.
3. Eliminate tenant-specific selector history for audit + eDiscovery.

Per-tenant keys (one set per `(tenant_id, domain_id, selector)`) limit blast radius to one tenant + one domain + one selector epoch. Tenant admins can rotate without platform intervention. The OpenBao path `secret/<tenant_id>/mail/dkim/<domain_id>/<selector>` makes the per-tenant boundary explicit.

## Why two selectors at all times?

Per ADR-MAIL-001 § Decision: "Require tenant admins to publish at least two DKIM selectors before moving a domain to production send."

Reasons:

1. **Rotation overlap**: when rotating the active selector, the old one must remain valid for receivers to verify in-flight mail (typically 72-hour overlap for DNS TTL convergence).
2. **Compromise mitigation**: if one selector is compromised, the other can be promoted to active immediately while the compromised one is revoked.
3. **Algorithm migration**: e.g., migrating RSA-2048 → Ed25519 requires publishing the Ed25519 selector while the RSA one is still active for receivers that don't yet support RFC 8463.

Selector naming `sYYYYMMDDa` + `sYYYYMMDDb` makes rotation cadence deterministic.

## When can I use Ed25519 (RFC 8463) DKIM instead of RSA-2048?

Per ADR-MAIL-001 § Decision: "Use Ed25519 DKIM where receiver compatibility allows; default to RSA-2048 for broad receiver compatibility until pack policy upgrades the default."

Receiver support as of 2026-05:

- Gmail: yes (since 2018).
- Microsoft 365: yes (since 2024).
- Yahoo: yes (since 2023).
- Apple iCloud Mail: yes.
- AWS SES: yes for verification (SES doesn't sign with Ed25519 itself yet).
- ProtonMail, Tutanota: yes.
- Many corporate mail gateways: mixed (Cisco Email Security, Mimecast, Proofpoint vary by version).

Strategy for `tenant_class=paid` production domains: publish BOTH RSA-2048 + Ed25519 selectors. Receivers that support Ed25519 prefer it (smaller signature, faster verification); RSA-2048 acts as fallback.

## How do I promote DMARC from `none` to `quarantine` to `reject` safely?

Per ADR-MAIL-001 § Decision: "Enforce tenant-level policy progression through `none`, `quarantine`, and `reject` with a seven-day soak between levels."

Soak window guidelines:

- `none → quarantine`: 7 d minimum. Required: < 1% DMARC failure rate from sample window.
- `quarantine → reject`: 14 d minimum. Required: < 0.1% DMARC false-positive rate (legit mail being quarantined).

The soak window detects:

1. Forgotten sending domains (third-party tools sending as you).
2. Forwarder breakage (mailing lists that don't add ARC).
3. ESP misconfiguration (e.g., a marketing tool with wrong SPF include).

`oya mail dmarc policy promote` is Cedar-gated on `mail::dmarc_policy::promote` which requires failure rate below pack threshold.

## What happens when ARC chain validates but DMARC fails alignment?

Per ADR-MAIL-001 § Decision: "Validate ARC only as a modifier to forwarding trust; ARC never overrides DMARC `reject` for high-risk packs unless a tenant allowlist grants it."

Scenarios:

- **Legitimate forwarder** (e.g., mailing list with valid ARC): tenant can grant `mail::auth_override::arc-forwarder` for that forwarder; subsequent mail with valid ARC chain bypasses DMARC reject.
- **High-risk pack** (HIPAA, FedRAMP-High): Cedar forbids `mail::auth_override::grant` unless council approval is present. ARC override is rare in these packs.
- **No allowlist**: DMARC reject is final; ARC chain is ignored.

This prevents ARC chain forgery from becoming a DMARC bypass.

## Why is the spam classifier pack-gated for LLM use (ADR-MAIL-0004)?

Per ADR-MAIL-0004 § Decision. The EU AI Act (Regulation 2024/1689) classifies spam classifiers as **Annex III high-risk** when they make material decisions about user-facing content. The Act's Art 26 + Annex III require:

- Conformity assessment.
- Documented training data + algorithm transparency.
- Human-in-the-loop for high-stakes decisions.
- Right-to-explanation for affected users.

Pack-gated behavior:

- **Default tenants (non-EU-resident, non-EU-customer-serving)**: LLM-assisted classifier (e.g., Llama 3.3 70B fine-tuned on spam-corpus) enabled.
- **EU-GDPR pack tenants**: LLM classifier behind a tenant opt-in toggle; human-in-the-loop required for `quarantine` decisions; right-to-explanation surface in user-facing UI.
- **High-risk packs (HIPAA, FedRAMP-High, KR-PIPA)**: Rspamd Bayesian + RBL only; no LLM-based classification (per ADR-MAIL-0004 § Decision).

The Rspamd-only path is FIPS-compatible and avoids the EU AI Act Annex III scope.

## How does the mail-key recovery envelope work without operator decryption?

Per ADR-MAIL-0001 + the identity µservice recovery model (ADR-ID-001 § analogous pattern).

1. User generates a strong recovery passphrase (≥ 24 chars, dictionary-checked).
2. Server derives a wrapping key from the passphrase (Argon2id; 2 GiB memory; 4 iterations).
3. Server generates the per-mailbox recovery secret + wraps it with the derived key.
4. Wrapped ciphertext stored in OpenBao at `secret/<tenant_id>/mail/recovery/<subject_id>/<recovery_epoch>`.
5. **Plaintext passphrase NEVER reaches the server** — only the user knows it.
6. To recover: user provides passphrase + passkey AAL3 step-up. Cedar requires both. Server unwraps under the user-derived key + verifies passkey-bound identity claim.

Operator cannot decrypt because they don't know the passphrase + the OpenBao mount denies operator-bound principals on `mail-recovery-unwrap` action.

## What's the JMAP backend choice (ADR-MAIL-0002)?

Per ADR-MAIL-0002 § Decision. Two backends considered:

- **Cyrus IMAPd 3.10 + JMAP module**: mature, FIPS-validated, RFC 8620 conformant.
- **Stalwart Mail 0.7**: rust-native, modern; JMAP RFC 8620 + JMAP for Calendars; better tenant isolation primitives.

Decision: Stalwart for new `demo_trial` and `paid` tenants; Cyrus only for legacy compatibility. Migration path: Stalwart fully replaces Cyrus by backend standardization.

## How does eDiscovery work without exposing tenant private keys?

Per ADR-MAIL-001 Constraint MAIL-C12 + paid tenant_class mailbox DEK envelope (similar to ADR-DRIVE-001).

- **Paid tenant_class server-side encryption with tenant KEK**: server can decrypt mailbox content for eDiscovery requests with valid Cedar `mail::ediscovery::export` permission + court-order evidence + audit-chain seal.
- **Paid tenant_class per-mailbox DEK envelope with tenant CMK in HSM**: server holds ciphertext + DEK envelope rows; tenant-controlled legal-hold appliance holds the keys. Export contains ciphertext + envelope; tenant decrypts under their own custody.

The decision is per-pack:

- KR-PIPA: tenant must control keys (paid tenant_class pack default).
- US-HIPAA: BAA allows server-side decryption with audit-chain (paid tenant_class pack acceptable).
- FedRAMP-High: tenant-controlled keys recommended (paid tenant_class pack default).

## What's the SDK launch order (ADR-MAIL-0003)?

Per ADR-MAIL-0003 § Decision. SDKs ship in this order:

1. **Rust SDK** (Wave 1, first SDK; the reference implementation).
2. **TypeScript SDK** (Wave 2; for JMAP web clients + Node.js).
3. **Python SDK** (Wave 2; for SDK consumers in scientific + ML workloads).
4. **Go SDK** (Wave 3; for SaaS integrations + ops tooling).
5. **Java/Kotlin SDK** (Wave 3; for enterprise integrations).
6. **Swift + Kotlin Multiplatform mobile** (Wave 3; for native mobile clients).

The order reflects build-out priority + first-customer ask, not a quality ranking.

## What's the MTA-STS enforcement story?

Per ADR-MAIL-001 + RFC 8461. MTA-STS:

- Allows a domain to publish a policy file stating that inbound mail MUST use TLS + valid certificate.
- Receivers fetch the policy file via HTTPS + cache for the published `max_age` (typically 30 d).

At oyatie:

- **Rollout phase 0**: MTA-STS DNS record published; policy file available; but outbound enforcement is `testing` mode (report failures, don't reject).
- **Rollout phase 1**: outbound enforcement at `enforce` for high-volume paid tenants; tenant opt-in for production domains.
- **Rollout phase 2**: MTA-STS enforced by default outbound (reject TLS-failed delivery; alert tenant via TLSRPT).

TLSRPT (RFC 8460) reports inbound TLS failures to the tenant's `_smtp._tls` reporting address; tenants can dashboard via Grafana.

## How are spam emails handled per-tenant policy?

Per ADR-MAIL-0004 + tenant moderation policy. Per-message flow:

1. Inbound SMTP receives.
2. SPF + DKIM + DMARC + ARC + TLS evaluated → `MailAuthResult` typed.
3. Disposition decision:
   - DMARC `reject` + alignment fail → reject at SMTP time (no mailbox insert).
   - DMARC `quarantine` + alignment fail → quarantine folder.
   - DMARC pass → continue.
4. Anti-phishing + spam classifier (Rspamd or LLM per pack).
5. Classifier confidence ≥ 0.95 → quarantine OR auto-junk based on tenant policy.
6. Classifier confidence 0.40-0.95 → quarantine with user-visible reason.
7. Classifier confidence < 0.40 → inbox.

Quarantine folder retention: 30 d default; user can release or permanently delete.

## How does cross-tenant mail work?

Mail is intrinsically cross-tenant (RFC 5321) — that's the point of email. The per-tenant boundary is:

- Inbound: receiving tenant evaluates SPF/DKIM/DMARC for the sender's domain. If pack policy denies external mail (e.g., air-gap paid tenant_class pack), inbound is rejected at SMTP.
- Outbound: sending tenant signs with its own DKIM key. Per-tenant signing-key custody (ADR-MAIL-001) prevents cross-tenant key reuse.
- Per-tenant relay limits (rate, sending-domain validation, recipient block-lists).

For B2B intra-org cross-tenant routing (e.g., between a parent tenant and a subsidiary child tenant), per ADR-TEN-001 the conglomerate hierarchy + Cedar grants allow routing to skip external SMTP and use a direct internal mail bus.

## How do I move from Gmail Workspace to oyatie mail?

See `migration-playbooks/from-gmail-workspace.md` for the full playbook. Short version:

1. Run Gmail Workspace Data Export.
2. Run `oya mail migrate import-gmail` (preserves message-id + threading + labels).
3. Migrate DNS: update MX records to oyatie cell (after testing).
4. Migrate SSO/IdP via `identity` µservice.
5. Shadow period: dual-delivery (both Gmail + oyatie receive) for 30-60 d.
6. Cutover: DNS-flip; Gmail becomes read-only archive.
7. Decommission Gmail after 90+ d.
