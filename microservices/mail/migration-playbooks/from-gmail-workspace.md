---
doc_class: MigrationPlaybook
microservice: mail
vendor: Gmail Workspace (Business Starter, Standard, Plus, Enterprise)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Gmail Workspace → oyatie mail

Audience: a team running Gmail Workspace (Business Starter through Enterprise) for organization mail. Drivers: per-tenant DKIM key custody + sovereign-pack residency + cryptographic audit-chain + EU AI Act compliance + 50% TCO reduction at 10k+ seat scale + JMAP RFC 8620 support.

Note: counterpart vendors use pricing tiers; Oyatie does not — `tenant_class` is binary.

## Why this migration matters

Gmail Workspace is excellent at:

- Best-in-class spam classifier (decades of training data).
- Massive integration ecosystem (Google Workspace Marketplace).
- Seamless Google Calendar + Drive + Meet bundling.
- Mobile clients (Android native; iOS first-class).
- Generous vendor free pricing tier for small businesses.

oyatie mail adds:

- **Per-tenant DKIM key custody in OpenBao + HSM** (Google holds Gmail keys; tenants cannot prove key separation).
- **JMAP RFC 8620** for modern client integration (Gmail doesn't support JMAP).
- **Cryptographic audit-chain** (Gmail's audit log is server-mutable).
- **EU AI Act Annex III compliance** for spam classifier (Gmail's classifier scope vs EU AI Act is unclear; oyatie has explicit pack-gated mode).
- **Sovereign-pack residency** (KR-PIPA, EU-GDPR Art 9, US-HIPAA, FedRAMP-High, CN-PIPL).
- **No per-seat licensing in this self-hosted comparison** — paid `tenant_class` self-hosted at ~ $85/seat/year for 10k seats vs Gmail Business Standard at $144/seat/year.
- **eDiscovery without server-side plaintext access** through paid tenant_class custody packs.

The trade-off: Gmail's spam classifier accuracy is higher (F1 ≈ 0.987 vs oyatie paid hybrid 0.980). For most enterprises, the 0.7 percentage-point gap is acceptable for the compliance + custody benefits. Bridge mode (post-import) can route inbound through Gmail's classifier for 30-90 d during transition.

## Step 1 — Inventory the Gmail Workspace estate (≤ 1-2 weeks)

```bash
# Google Workspace Admin Console → Data Export
# Or use Google Vault Export API (Enterprise required for full corporate export)
gam create export "ProjectExport" \
    --query "from:* OR to:*" \
    --query-period "2020-01-01:2026-05-20" \
    --include-shared-drives true \
    --include-rooms true

# Or use Google Workspace Migration Service (GWMS) for bulk export
gwms export \
    --organization acme-corp.example \
    --include-mail,calendar,contacts,drive \
    --output ./gws-export/
```

Document:

- User count + vendor license-tier distribution (Starter/Standard/Plus/Enterprise).
- Total mail volume (typical: 100 GB-10 TB total).
- Active sending domains + DKIM key configuration.
- Marketplace apps installed + their replacement plan.
- Active Vault retention policies.
- SAML/OIDC IdP (Okta, Entra ID, OneLogin, etc.).
- Mail forwarding rules + filters.
- Shared mailboxes + delegated access patterns.
- Calendar resources (rooms, equipment).
- Custom domain DKIM setup (per-domain).

Typical mid-size Gmail Workspace: 1k-10k seats, 5-50 sending domains, 10-100 marketplace apps.

## Step 2 — Map Gmail Workspace concepts to oyatie mail (≤ 1 week)

| Gmail Workspace concept | oyatie mail equivalent |
|---|---|
| Organization (workspace) | Tenant |
| User account | Principal + mailbox |
| Group | Cedar role + mailbox group |
| Custom domain | TenantMailDomain (per ADR-MAIL-001) |
| Sending alias | Per-user sending alias |
| Vault retention policy | `compliance` µservice pack retention class |
| Vault legal hold | `compliance` µservice legal-hold lock |
| Vault eDiscovery search | oyatie eDiscovery + audit-chain query |
| 2-step verification | identity µservice passkey-primary (ADR-ID-001) |
| Google Workspace SSO | identity µservice external IdP federation (Okta/Entra/etc.) |
| Marketplace app | oyatie plugin SDK |
| Gmail filters | JMAP filter rules (RFC 8620) |
| Mail forwarding | JMAP mailbox forwarding |
| Shared mailbox | oyatie mailbox + Cedar role delegation |
| Google Groups (mailing list) | oyatie mailing list (mailbox group with cross-tenant routing) |

## Step 3 — DNS preparation + DKIM key generation (≤ 1 week)

Before any data migration, prepare oyatie side:

```sh
# 1. Create tenant + verify each sending domain
oya mail tenant create --cell prod-us-east-1 --tenant-id acme-corp
oya mail domain verify-init --tenant acme-corp --fqdn acme.com
# (Add DNS TXT record for verification; complete verification)

# 2. Generate DKIM selector pairs for each domain
oya mail dkim selector create-pair \
    --tenant acme-corp \
    --domain dom_acme_com \
    --algorithm Ed25519   # New Ed25519 selectors; tenant publishes DNS

oya mail dkim selector create-pair \
    --tenant acme-corp \
    --domain dom_acme_com \
    --algorithm RSA-2048   # Keep RSA fallback for receivers without Ed25519 support
```

Tenant publishes the new DKIM DNS records ALONGSIDE the existing Gmail Workspace DKIM records (Google's DKIM uses selector `google._domainkey`). Both can coexist.

## Step 4 — Data migration (≤ 4-12 weeks per TB)

```sh
oya mail migrate import-gmail-workspace \
    --tenant acme-corp \
    --gws-export-dir ./gws-export/ \
    --map-domain-to-tenant acme.com=acme-corp \
    --preserve-message-id true \
    --preserve-thread-id true \
    --preserve-labels true \
    --convert-labels-to-folders false \
    --throttle-rate 1000-msgs-per-sec \
    --include-vault-archive true
```

The migration:

1. Creates oyatie principals from Google users (preserve primary email + recovery email + display name).
2. Imports mailboxes preserving folder structure + Gmail labels (as IMAP/JMAP labels).
3. Imports messages preserving Message-ID + In-Reply-To + References headers (so threading is preserved cross-system).
4. Imports Vault archives into pack-retention-class-aware archive tier.
5. **No password import** — passwords are never migrated. Users authenticate via SSO or passkey-bootstrap on first login.
6. Replays Google Group memberships → oyatie mailing list memberships.

Backfill rate ~ 1k msgs/sec for paid `tenant_class` standard migration context. 1 TB of mail (typical 100M messages) → ~ 28 hours.

Verify post-import counts:

```sh
oya mail mailbox stats --tenant acme-corp --user u-alice@acme-corp
# Output:
#   total_messages: 38 421
#   total_size: 4.2 GiB
#   folder_count: 47
#   imported_from: gmail-workspace
```

Cross-check against the Gmail Workspace export's per-user counts.

## Step 5 — SSO + identity migration (≤ 1-2 weeks)

```sh
# Configure identity µservice OIDC federation with the existing IdP
oya identity oidc-federation configure \
    --tenant acme-corp \
    --idp okta-acme-corp \
    --idp-discovery-url https://acme-corp.okta.com/.well-known/openid-configuration \
    --client-id <okta-client-id> \
    --client-secret-bao-ref secret/acme-corp/identity/okta-client-secret

# Enable passkey-primary authentication
oya identity tenant update \
    --tenant acme-corp \
    --auth-policy passkey-primary \
    --webauthn-rp-id acme-corp.example \
    --recovery-envelope-required true

# Users keep their Okta IdP login, but Oyatie issues its own session tokens with `acr=aal2_passkey_uv` on passkey-enroll
```

## Step 6 — Bridge mode + shadow run (≤ 30-60 d)

**Bridge mode**: route inbound mail through Gmail Workspace for the first 30-60 d while users transition. Outbound goes through oyatie (with new DKIM signing).

DNS pre-cutover:

```
# MX records (continue Gmail Workspace receiving)
acme.com.   MX 10 ASPMX.L.GOOGLE.COM.
acme.com.   MX 20 ALT1.ASPMX.L.GOOGLE.COM.
...

# SPF (add oyatie as sender alongside Google)
acme.com.   TXT "v=spf1 include:_spf.google.com include:_spf.oyatie.local -all"

# DKIM (publish oyatie selectors in addition to Gmail's google selector)
google._domainkey.acme.com.   TXT "<existing Gmail DKIM>"
s20260520a._domainkey.acme.com.   TXT "<new oyatie Ed25519 DKIM>"
s20260520b._domainkey.acme.com.   TXT "<new oyatie Ed25519 DKIM, second>"

# DMARC (start at none for soak; stay at Gmail's existing policy initially)
_dmarc.acme.com.   TXT "v=DMARC1; p=none; rua=mailto:dmarc-rua@acme.com"
```

## Step 7 — DNS cutover (≤ 1 day)

After 30-60 d of clean bridge mode:

```sh
# 1. Verify all users have JMAP/IMAP clients pointed to oyatie + can send/receive
oya mail tenant cutover-readiness-check --tenant acme-corp
# Output:
#   active_users_with_oyatie_client: 9 847 / 10 000 (98.5%)
#   pending_users: 153 (mostly inactive accounts)
#   dmarc_failure_rate: 0.04% (safe to cutover)
#   recommended_action: PROCEED

# 2. DNS flip
# MX records (oyatie receiving)
acme.com.   MX 10 mx1.prod-us-east-1.oyatie.local.
acme.com.   MX 20 mx2.prod-us-east-2.oyatie.local.

oya audit emit \
    --tenant acme-corp \
    --event-class governance.mail_substrate.cut_over \
    --payload '{"from":"gmail-workspace","to":"oyatie","cutover_at":"2026-08-15T14:00:00Z"}'
```

## Step 8 — Gmail Workspace decommission (≤ 90-180 d post-cutover)

After ≥ 90 d:

- Export final Vault state for compliance retention.
- Cancel Google Workspace contract.
- Retain Vault archive read-only access for legal-hold duration.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Spam classifier accuracy gap (~0.7 F1 vs Gmail) | Medium | Bridge mode through Gmail for 60-90 d; train oyatie classifier on tenant-specific corpus during bridge |
| Marketplace app gap (oyatie plugin SDK has fewer integrations at launch) | High | Pre-audit; port top-20 marketplace apps to plugin SDK before cutover; defer long-tail apps to community |
| Google Groups migration | Medium | Map to oyatie mailing lists; preserve email-list addresses + member lists |
| Custom domain DKIM key extraction not possible (Google holds keys) | High | Cannot import Gmail's DKIM private keys; oyatie generates new keys; tenant publishes DNS for both during bridge |
| User SSO continuity | Low | Reuse existing IdP via `identity` µservice OIDC federation |
| Password migration | High | Passwords never migrated (security best practice); users enroll passkey or use SSO |
| Mail filter rules complexity | Medium | Auto-convert Gmail filters to JMAP RFC 8620 filter rules; some 1:1 mapping, complex ones may need manual review |
| Calendar invites (iMIP) | Low | Handle via `calendar` µservice mail handoff |
| Mobile client transition (Gmail mobile vs oyatie mobile) | Medium | Side-by-side iOS/Android available; rolling cutover by device |
| Vault retention conversion to pack-retention-class | Medium | Map via `compliance` µservice (ADR-COMP-001 effective policy); legal-hold preserved |
| Google Groups list emails | Medium | Recreate as oyatie mailing lists with member migration; preserve list address |
| Shared mailboxes + delegated access | Medium | Map to oyatie shared mailbox + Cedar role delegation; SCIM provisioning recommended |
| Inbox tabbed categories (Promotions, Social, etc.) | Low | oyatie has equivalent classification labels |
| Sender reputation | High | Bridge mode + DMARC `none` initial keeps Google reputation; gradually shift outbound IPs |
| Inbound delivery to Gmail recipients (post-cutover from oyatie) | High | Warm up oyatie outbound IPs gradually; monitor Postmaster Tools + DMARC reports |
| Search functionality | Medium | OpenSearch index built post-import; full-text quality differs from Gmail's search; warm up + tune relevance |
| Backup + archive | Low | oyatie archive tier with audit-chain seal; retention per pack |
| Two-step verification migration | Low | Users re-enroll passkey on first oyatie login (better than 2FA codes) |
| Gmail-specific features (Smart Compose, smart reply) | Low | oyatie has equivalent via `intelligence` µservice LLM-bridge; pack-gated for EU-AI-Act tenants |
| Google Drive integration with mail (attachments stored on Drive) | Medium | Map to oyatie `drive` µservice; attachments stored as drive files with per-DEK envelope (ADR-DRIVE-001) |
| Google Meet integration | Medium | Map to `meet` µservice or `messenger` huddles |
| URL preview cards in mail | Low | oyatie has equivalent; respect pack DLP policies |
