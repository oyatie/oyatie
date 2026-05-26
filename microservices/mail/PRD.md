---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-mail
microservice: mail
status: Accepted
sales_segment: shared-substrate-and-product
service_class: hero-product
launch_modes: [B2C-personal, B2B-work, oyatie-internal-tenant]
milestone_first_ship: M03-connect-dissolution
bominal_source: [ADR-0208, ADR-0210, ADR-0215]
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0123, ADR-0131, ADR-0132, ADR-0133, ADR-0135, ADR-0139, ADR-0140, ADR-0145, ADR-0208, ADR-0210, ADR-0215, ADR-0238, ADR-0241, ADR-0242, ADR-0245, ADR-0251, ADR-0255, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345, ADR-MAIL-0003]
related_specs: [/specs/microservices/mail.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-20
owner_team: axis-mail + council-privacy + ops-deliverability
doc_status: published
---

# PRD-mail: Personal mail + Work mail as two surfaces of one mail substrate

> Hero product. Launches B2C personal + B2B work + `oyatie.*` internal-tenant on day one. Gmail/Outlook/Hey/Superhuman parity for personal; Workflow + Messenger + Calendar + Meet + HR/Payroll deep-integration for work. Per ADR-0242 oyatie is a tenant of its own platform; per ADR-0245 every mail surface is built on the same substrate and differentiated by Cedar-gated localisation overlays.

---

## 1. Purpose

The `mail` µservice is oyatie's unified mail surface. It speaks SMTP + IMAP4rev2 + JMAP + REST at the edge, stores messages in a per-tenant Postgres + SeaweedFS + Tantivy stack, and is differentiated at the application layer into two products that share the same substrate:

- **Personal Mail (B2C).** Standalone consumer mail account (`alice@oyatie.app`, custom domain optional). Best-in-class triage, smart compose, keyboard-first, anti-tracking-by-default. Targets Gmail / Outlook.com / Apple Mail / Hey / Superhuman / Proton / Fastmail switchers.
- **Work Mail (B2B).** Enterprise mailbox attached to a tenant (`alice@acme.com` on `acme` tenant). Adds delegation, shared mailboxes, DLP, retention, legal hold, eDiscovery, audit-chain, Workflow Engine triggers, e-signing, meeting invites, clocking-in / approval flows. Targets Microsoft 365 / Google Workspace / Zoho / Naver Works switchers.

A single user can hold any number of personal accounts and be a member of any number of work tenants. The kernel-layer **dual-context isolation** invariant (ADR-0135) guarantees a user's personal mailbox is structurally invisible to any org admin, even when both contexts share a physical cluster.

Both surfaces are products in their own right. The substrate (`oya-mail-*` crates) is also reusable by any other oyatie product that needs mail primitives (e.g., Messenger's mail-bridge, Calendar's iMIP receiver, the Plugin App Store's app-installation receipt mailer). Substrate-vs-product layering follows ADR-0245.

---

## 2. Audience and Tenant Modes

### 2.1 Tenant modes

| Mode | Tenant | Authoritative identity | Primary surface | Differentiated UX | Compliance posture |
|---|---|---|---|---|---|
| **B2C Personal** | implicit per-user (`__personal__/<user_id>`) | Zitadel personal IdP (email-password + WebAuthn + Apple/Google federation) | Personal Mail web/mobile/desktop apps | Hey/Superhuman-class triage, Gmail-class smart compose | GDPR + CCPA + KR-PIPA personal-data subject |
| **B2B Work** | per-org (`acme`, `naver`, etc.) | Zitadel tenant IdP (SAML/OIDC SSO, SCIM provision, MFA enforced) | Work Mail web/mobile/desktop + admin console | Workflow integration, shared mailboxes, delegation, e-signing | GDPR/HIPAA/SOC2/ISO-27001 processor; per-tenant retention floors |
| **`oyatie.*` internal** | `oyatie-corp` (axis-mail, council-*, ops-*) | Zitadel internal IdP + GitHub federation | Work Mail with internal admin overlay | Same as B2B but with dogfooding flags surfaced | Same as B2B; per ADR-0242 we ARE a tenant |

Per ADR-0242 (oyatie-is-a-tenant doctrine): the oyatie company itself runs entirely on the `oyatie-corp` tenant with no special-case code paths. Every dogfooding bug found is a paying-customer bug found.

### 2.2 Cedar gating across modes

The same Cedar policy engine (per ADR-0140 / ADR-0255) gates feature availability per mode:

- B2C: `Mail::FeatureClass::Personal` policies — `smart_compose=allow`, `anti_tracking=enforce_block_remote`, `shared_mailbox=deny`, `dlp=deny`, `legal_hold=deny`.
- B2B: `Mail::FeatureClass::Work` policies — `shared_mailbox=allow`, `dlp=allow`, `legal_hold=allow`, `workflow_trigger=allow_with_explicit_consent`, `anti_tracking=tenant_configurable`, `smart_compose=tenant_opt_in_with_dp_noise`.
- Internal: B2B policy set + `dogfooding_flag=true` + `incident_redaction=on`.

Cedar fragments live under `microservices/mail/policy/cedar/{personal,work,internal}.cedar` and are versioned with PR-review.

### 2.3 User journey

A new oyatie user signs up with `alice@gmail.com`, gets a personal mailbox at `alice@oyatie.app` (or custom domain), and uses Personal Mail. Months later her employer Acme adopts oyatie; her admin invites `alice@acme.com`. The Acme work mailbox appears in her mail client as a second account. The two are kernel-isolated: Acme's compliance officer cannot legal-hold or eDiscover anything in `alice@oyatie.app`; Alice cannot accidentally forward an Acme-confidential thread to a personal label that org admins would never see (the forward goes through DLP and audit-chain).

---

## 3. Feature Matrix vs Benchmarks

Legend:
- `Y` = fully supported on the comparator (parity).
- `P` = partial (e.g., behind paid SKU, limited region, or gimped).
- `N` = not supported.
- `Y+` = supported and oyatie target exceeds (deeper / more controllable / cheaper / open).

Sources cited inline in §14. Snapshot date: 2026-05.

### 3.1 Compose, send, reply

| Feature | Gmail | Outlook (M365) | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| Compose rich text (HTML + Markdown) | Y | Y | Y | Y | Y | Y | Y | **Y+** (Markdown-native; HTML preview) |
| Compose plain text | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Reply | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Reply-all | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Forward | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Forward-as-attachment (.eml) | Y | Y | N | Y | Y | Y | Y | **Y** |
| Attach files (per-attachment) | 25 MB | 20–150 MB | 30 MB | 25 MB | 20 MB | 25 MB | 50 MB | **50 MB (default; tenant-configurable to 150 MB; large-file handoff to SeaweedFS link)** |
| Attach total per message | 25 MB | 150 MB | 30 MB | 25 MB | 20 MB | 25 MB | 50 MB | **150 MB** |
| Inline images | Y | Y | Y | Y | Y | Y | Y | **Y** (lazy-load + tracker-strip) |
| Embedded video preview | Y | Y | N | N | P | N | P | **Y** (lazy-load) |
| Drag-and-drop attachments | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Paste image from clipboard | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Schedule send | Y | Y | N | Y | P | Y | Y | **Y+** (timezone-aware; Cedar-gated for B2B) |
| Send later (one-tap) | P | P | N | Y | N | Y | Y | **Y** (Cmd-Shift-Enter pattern) |
| Undo send | Y (30s) | Y | N | Y (60s) | N | Y | Y | **Y+** (configurable 5–120s; default 30s) |
| Smart compose (next-word suggest) | Y | Y | N | N | P | N | N | **Y** (per-user opt-in; LLM in tenant-cell; Cedar-gated) |
| Smart reply (one-tap suggestion) | Y | Y | N | N | P | N | N | **Y** (per-user opt-in) |
| Templates / canned responses | Y | Y | N | Y | Y | Y | Y | **Y** |
| Signatures (multi) | Y | Y | Y | Y | Y | Y | Y | **Y** (per-account, per-context) |
| Vacation responder | Y | Y | N | Y | Y | Y | Y | **Y** (with delegation hand-off) |
| Read receipts (request) | Y | Y | N | N | Y | N | Y | **Y** (tenant-configurable; off by default in B2C per anti-tracking) |
| Read receipts (deny by default to sender) | P | P | Y | Y | P | Y | Y | **Y+** (B2C deny; B2B tenant-configurable) |

### 3.2 Reading, threading, organisation

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| Conversation view (threading) | Y | Y | Y | Y | Y | Y | Y | **Y** (Gmail-style threading default) |
| Flat view option | P | Y | N | N | Y | Y | Y | **Y** |
| Labels (tags, multi) | Y | N | N | Y | N | Y | Y | **Y+** (multi-label; nested) |
| Folders (hierarchical) | P | Y | Y | Y | Y | Y | Y | **Y** |
| Move to folder | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Star / pin | Y | Y | N | Y | Y | Y | Y | **Y** |
| Snooze | Y | Y | N | Y | P | Y | Y | **Y+** (precise time + smart-snooze: "until weekend", "until next month") |
| Archive | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Mute thread | Y | Y | N | Y | N | P | Y | **Y** |
| Pin thread | N | P | N | Y | N | N | P | **Y** |
| Focused inbox (priority bucket) | Y | Y | N | N | N | N | P | **Y** (tenant + user opt-in) |
| Hey-style "Imbox / Feed / Paper Trail" | N | N | Y | N | N | N | N | **Y+** (opt-in; default off; called "Triage") |
| Mark as read on hover | Y | Y | N | N | N | N | N | **N** (never; anti-pattern; explicit-only) |
| Block remote images by default | N | P | Y | Y | P | Y | Y | **Y+** (default ON; explicit per-sender allow) |
| Block tracker pixels | P | P | Y | Y | N | Y | Y | **Y+** (block remote images = block trackers; also detect EV-tracking patterns) |
| Block one-click unsubscribe trackers | P | P | Y | Y | N | Y | P | **Y** |
| Strip referrer on link click | N | N | Y | Y | N | Y | P | **Y** |
| Block sender | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Show only attachments view | Y | Y | N | Y | P | P | Y | **Y** |
| Sort by sender | Y | Y | P | P | Y | Y | Y | **Y** |
| Sort by attachment | P | Y | N | P | Y | P | Y | **Y** |
| Sort by date / size / subject | Y | Y | P | P | Y | Y | Y | **Y** |
| Filter view (saved search) | Y | Y | N | Y | Y | Y | Y | **Y** |
| Quick action keyboard shortcuts | Y | Y | P | Y+ | P | Y | Y | **Y+** (Superhuman parity: E/Y/U/J/K/G+I/G+S/G+D/G+T/etc.) |
| Touch-bar / context menu | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Multi-select bulk actions | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Drag thread between folders | Y | Y | Y | Y | Y | Y | Y | **Y** |

### 3.3 Filtering, automation, intelligence

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| User-defined filters / rules | Y | Y | N | N | Y | Y | Y | **Y** (Sieve RFC 5228 + UI) |
| Server-side rules | Y | Y | N | N | N | Y | Y | **Y** (Sieve runs on `rules-engine`) |
| Sieve standard support | N | N | N | N | N | P | Y | **Y+** (RFC 5228 + ManageSieve RFC 5804) |
| Conditional auto-forward | Y | Y | N | N | Y | Y | Y | **Y** (Cedar-gated for B2B) |
| Conditional auto-label | Y | Y | N | N | Y | Y | Y | **Y** |
| Smart classification (Promotions/Social/Updates) | Y | Y | N | N | N | N | N | **N** (anti-pattern; user-configurable categories instead) |
| Hey screening for new senders | N | N | Y | N | N | N | N | **Y+** (opt-in; "first-time-sender prompt") |
| Smart unsubscribe button | Y | Y | Y | Y | P | Y | Y | **Y+** (RFC 8058 + heuristic + RFC 2369 List-Unsubscribe header) |
| Mute-with-snooze | P | P | N | Y | N | N | Y | **Y** |
| Trigger workflow from rule | N | P (Power Automate) | N | N | N | N | N | **Y+** (native Workflow Engine event emission, Cedar-gated) |
| Trigger calendar event from rule | N | Y | N | N | N | N | N | **Y** (iMIP RFC 6047) |
| Trigger HR event (timesheet/approval) | N | N | N | N | N | N | N | **Y+** (native; per §5.7) |
| LLM-powered summarisation | Y (Workspace) | Y (Copilot) | N | Y | N | N | N | **Y** (per-tenant LLM cell; per-user opt-in; DP-noise on training) |
| LLM-powered "answer for me" | Y (Workspace) | Y (Copilot) | N | Y | N | N | N | **Y** (Cedar-gated; explicit-consent at first use) |
| LLM thread digest | P | Y | N | Y | N | N | N | **Y** |

### 3.4 Protocols, sync, multi-device

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| IMAP4 | Y | Y | N | N | Y | P (Bridge) | Y | **Y** (RFC 9051 IMAP4rev2 native) |
| IMAP4rev1 (legacy) | Y | Y | N | N | Y | P | Y | **Y** (RFC 3501 backwards-compat) |
| POP3 | Y | Y | N | N | Y | P | Y | **Y** (RFC 1939; opt-in; not default) |
| SMTP submission | Y | Y | N | N | Y | P | Y | **Y** (RFC 6409 :587 + :465) |
| JMAP (RFC 8620 + 8621) | N | N | N | N | N | N | Y | **Y+** (first-class API; web/mobile native client use JMAP) |
| JMAP Push (RFC 8620 §7) | N | N | N | N | N | N | Y | **Y** |
| ActiveSync / EAS | N | Y | N | N | Y | N | N | **Roadmap** (M05; not day-one) |
| MAPI / Exchange Web Services | N | Y | N | N | P | N | N | **N** (use JMAP/IMAP) |
| CardDAV (contacts) | N | N | N | N | Y | Y | Y | **Y** (provided by `contacts` µservice; bridged) |
| CalDAV (calendar) | N | N | N | N | Y | Y | Y | **Y** (provided by `calendar` µservice; bridged) |
| Web client | Y | Y | Y | Y | P (iCloud) | Y | Y | **Y** (Next.js + Tauri PWA-ready) |
| Desktop native (macOS/Win/Linux) | N | Y | Y | Y | Y | Y | P | **Y** (Tauri-built; one codebase) |
| Mobile native (iOS/Android) | Y | Y | Y | Y | Y | Y | Y | **Y** (Swift / Kotlin clients; JMAP) |
| Offline mode | P | Y | Y | Y | Y | Y | Y | **Y** (JMAP cache + local SQLite + sync) |
| Push notifications | Y | Y | Y | Y | Y | Y | Y | **Y** (APNs + FCM + Web Push) |
| Multi-account | Y | Y | Y | Y | Y | Y | Y | **Y** (mix personal + work in one client) |

### 3.5 Identity, addressing, accounts

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| Sub-addressing (`alice+tag@`) | Y | Y | Y | Y | P | Y | Y | **Y** |
| Aliases (multiple addresses → one inbox) | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Catch-all alias (`*@my.dom`) | N | P | N | N | N | P | Y | **Y** (per custom domain) |
| Send-as (different From) | Y | Y | N | Y | Y | Y | Y | **Y** |
| Burner / disposable addresses | N | N | P | N | Hide-My-Email | SimpleLogin | Masked-Email | **Y+** (`alice.<random>@oyatie.app`; revocable; Apple-Hide-My-Email parity) |
| Custom domain | Y (paid) | Y (paid) | Y (paid) | Y (paid) | Y (paid) | Y (paid) | Y | **Y** (B2C: paid; B2B: included; auto DNS via tenant onboarding) |
| Multi-user shared mailbox | P | Y | N | N | P | N | Y | **Y+** (B2B; per §5.1) |

### 3.6 Security, signing, encryption

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| TLS (SMTP submission) | Y | Y | Y | Y | Y | Y | Y | **Y** (TLS 1.3 mandatory) |
| STARTTLS (inbound :25) | Y | Y | Y | Y | Y | Y | Y | **Y** (RFC 3207) |
| MTA-STS (RFC 8461) | Y | Y | P | P | Y | Y | Y | **Y** |
| TLS-RPT (RFC 8460) | Y | Y | P | P | Y | Y | Y | **Y** |
| DKIM sign outbound | Y | Y | Y | Y | Y | Y | Y | **Y** (RFC 6376 + RFC 8463 Ed25519) |
| DKIM verify inbound | Y | Y | Y | Y | P | Y | Y | **Y** |
| SPF check inbound | Y | Y | Y | Y | Y | Y | Y | **Y** (RFC 7208) |
| DMARC enforce inbound | Y | Y | P | Y | P | Y | Y | **Y** (RFC 7489) |
| ARC chain (RFC 8617) | Y | Y | N | N | N | P | Y | **Y** |
| S/MIME sign + verify | P | Y | N | N | Y | Y | P | **Y** (RFC 8551) |
| S/MIME encrypt | P | Y | N | N | Y | N | P | **Y** |
| OpenPGP sign + verify | N | N | N | N | P | Y | Y | **Y** (RFC 4880 + RFC 9580 crypto-refresh) |
| OpenPGP encrypt | N | N | N | N | P | Y | Y | **Y** |
| Autocrypt key discovery | N | N | N | N | N | P | Y | **Y** (RFC autocrypt-level-1) |
| WKD (Web Key Directory) | N | N | N | N | N | Y | Y | **Y** |
| E2E mail (mail-native, beyond PGP) | N | N | N | N | N | Y (PM-internal) | N | **Y+** (opt-in MLS-style for `@oyatie.app↔@oyatie.app`; falls back to PGP/SMIME otherwise) |
| encryption-BYOK (customer KMS) | Workspace EKM | M365 DKE | N | N | N | N | N | **Y+** (per ADR-0251; per-tenant KEK in tenant KMS region) |
| Anti-phishing detection | Y | Y | Y | Y | P | Y | Y | **Y** (Rspamd + URL reputation + lookalike-domain detect) |
| Anti-malware attachment scan | Y | Y | Y | P | Y | Y | Y | **Y** (ClamAV + sandbox detonation for executable + Office macro) |
| Anti-spam | Y | Y | Y | Y | Y | Y | Y | **Y** (Rspamd; per-user Bayesian learning; tenant rule overlay) |
| Block sender | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Quarantine queue | Y | Y | N | N | P | P | P | **Y** (per-user) |
| Tracker-pixel blocking | P | P | Y | Y | P | Y | Y | **Y+** (default on; remote-image block + URL-rewrite for click-tracking detection) |
| Show "this sender uses trackers" warning | N | N | Y | Y | N | Y | N | **Y** |

### 3.7 Search and retrieval

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| Full-text search (body + attachments) | Y | Y | Y | Y | Y | P (encrypted) | Y | **Y+** (encrypted-token; Tantivy/Quickwit per IP-009) |
| Advanced search operators (from:, to:, has:) | Y | Y | P | Y | Y | Y | Y | **Y** (Gmail-equivalent operator set + JMAP filter) |
| Saved searches | Y | Y | P | Y | Y | Y | Y | **Y** |
| Search within attachments | Y | Y | N | P | Y | N | Y | **Y** (PDF/Office/text extraction; ciphertext-stays-encrypted at rest) |
| Search ranking by recency | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Search across all accounts | Y | Y | N | Y | Y | P | Y | **Y** (B2C personal accounts; B2B respects tenant scope) |
| Spotlight / OS-level search hand-off | N | N | N | N | Y | N | P | **Y** (macOS Spotlight + Windows-Search via OS APIs) |

### 3.8 Threading, replies, conversations

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| In-Reply-To / References threading | Y | Y | Y | Y | Y | Y | Y | **Y** (RFC 5322) |
| Subject-based fallback threading | Y | Y | Y | Y | Y | Y | Y | **Y** |
| JMAP Thread/get | N | N | N | N | N | N | Y | **Y** (RFC 8621) |
| Collapse quoted text | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Show only unread in thread | Y | Y | N | Y | P | Y | Y | **Y** |
| Inline reply (in conversation view) | Y | Y | N | Y | P | Y | Y | **Y** |

### 3.9 Calendar, meetings, scheduling

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| Render meeting invite (iCalendar) | Y | Y | N | Y | Y | Y | Y | **Y** (RFC 5545) |
| Accept/decline inline | Y | Y | N | Y | Y | Y | Y | **Y** |
| Send meeting invite from mail | Y | Y | N | Y | Y | Y | Y | **Y** (via `calendar` µservice) |
| iMIP for invite/cancel/update | Y | Y | N | Y | Y | Y | Y | **Y** (RFC 6047) |
| Suggested times from email content | Y | Y | N | Y | N | N | N | **Y** (Workflow Engine extraction; opt-in) |
| One-click Meet/Zoom/Teams join | Y | Y | N | Y | Y | N | N | **Y** (`meet` µservice integration; Jitsi/Zoom/Meet/Teams URL detect) |

### 3.10 Enterprise admin and compliance

| Feature | Gmail Workspace | Outlook M365 | Zoho Mail | Naver Works | Hey Business | Proton Business | Fastmail Business | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| SSO (SAML / OIDC) | Y | Y | Y | Y | N | Y | P | **Y** (Zitadel; SAML + OIDC + SCIM) |
| SCIM user provisioning | Y | Y | Y | Y | N | N | N | **Y** |
| Multi-user shared mailbox | Y | Y | Y | Y | N | N | P | **Y+** |
| Mailbox delegation | Y | Y | Y | Y | N | N | P | **Y** |
| Distribution lists / groups | Y | Y | Y | Y | N | N | P | **Y** |
| DLP (data-loss prevention) | Y | Y | P | P | N | N | N | **Y+** (Cedar policy + content scan + DSAR-compliant logging) |
| Retention policy per mailbox | Y | Y | Y | Y | N | N | P | **Y** |
| Legal hold (litigation hold) | Y (Vault) | Y (Purview) | Y | P | N | N | N | **Y+** (per ADR-0215; four-eyes for plaintext disclosure) |
| eDiscovery export | Y (Vault) | Y (Purview) | P | P | N | N | N | **Y+** (sealed bundle, re-derivable digest, Ed25519 chain-of-custody) |
| Audit log | Y | Y | Y | Y | N | P | P | **Y+** (audit-chain Merkle + Ed25519 per ADR-0028) |
| Per-jurisdiction data residency | Y | Y | Y | Y | N | P | P | **Y+** (pack-kr/eu/us/jp/sg/au/in/br/ae/ksa/us-healthcare per ADR-0117) |
| encryption-BYOK customer KMS | Workspace EKM | M365 DKE | N | N | N | N | N | **Y** (per ADR-0251) |
| Per-tenant SMTP IP pool | P (shared) | P (shared) | N | N | N | N | N | **Y+** (first-class FinOps + reputation surface) |
| Custom outbound DKIM domain | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Deliverability dashboard (bounce/spam/DMARC report) | P | Y | P | P | N | P | Y | **Y+** (per-tenant; DMARC aggregate ingest + display) |

### 3.11 Productivity and lifecycle

| Feature | Gmail | Outlook | Hey | Superhuman | Apple Mail | Proton | Fastmail | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| Vacation responder | Y | Y | N | Y | Y | Y | Y | **Y** |
| Out-of-office with delegation | P | Y | N | P | P | N | P | **Y** |
| Mail-to-task / todo | Y | Y | N | Y | P | N | P | **Y+** (Workflow Engine task; explicit consent + audit) |
| Mail-to-calendar event | Y | Y | N | Y | Y | N | P | **Y** |
| Mail import from Gmail/Outlook/IMAP | Y | Y | P | P | Y | Y | Y | **Y** (per IP-005; preserves source hash, labels, retention) |
| Mail export (.mbox / .eml) | Y (Takeout) | Y | N | N | Y | Y | Y | **Y** (DSAR-compliant per GDPR Art. 20) |
| Account deletion + erasure | Y | Y | Y | Y | Y | Y | Y | **Y** (DSR cascade; GDPR Art. 17; KR-PIPA Art. 36) |

Aggregate target: oyatie mail meets-or-beats every Y in the above tables; closes the only remaining gap (ActiveSync) at M05.

---

## 4. Personal Mode (B2C) — Feature Deep-Dive

### 4.1 Account creation

Sign-up via Zitadel personal IdP: email-password (with breached-password check from HIBP) + WebAuthn (passkey-preferred) + optional Apple/Google federation. Auto-provisions `<chosen>@oyatie.app` and a primary mailbox in `__personal__/<user_id>` tenant. Custom-domain available for `tenant_class=paid` (auto DNS via `cloud-network-dns` µservice).

### 4.2 Triage (Hey-style; opt-in)

When the user enables Triage:

- New sender → goes to **The Screener**. User chooses *yes (Imbox)*, *no thanks (never reply / spam)*, *just receipts (Paper Trail)*, *just news (Feed)*. The choice is remembered per-sender.
- Known sender → routed to the previously assigned bucket.
- Buckets: **Imbox** (primary), **Feed** (newsletters, marketing), **Paper Trail** (receipts, transactional), **Snoozed**, **Set Aside**, **Reply Later**.

Default off. Discoverable via onboarding card and Settings → Triage. Enabling does not retro-classify; user must walk through The Screener for past senders if they want.

### 4.3 Superhuman-style keyboard-first

Default keyboard shortcuts (rebindable):

| Action | Key |
|---|---|
| Archive | E |
| Archive + next | Y |
| Mark read / unread | Shift+I |
| Snooze (smart) | H |
| Reply | R |
| Reply all | A |
| Forward | F |
| Compose | C |
| Search | / |
| Go to Inbox / Sent / Drafts / Trash | G I / G T / G D / G # |
| Next / previous message | J / K |
| Open / close | Enter / Esc |
| Undo last action | U |
| Toggle star | S |
| Move to label | L |
| Multi-select toggle | X |
| Send | Cmd+Enter |
| Schedule send | Cmd+Shift+Enter |

100ms target latency on every key action (frontend pre-fetch + JMAP cache).

### 4.4 Smart compose + smart reply

- **Smart compose**: as the user types, suggests next phrase grey-text style. Tab to accept; keep typing to ignore. Provided by Intelligence substrate, per-user opt-in, model runs in tenant-cell (no cross-tenant training). DP-noise applied at training time per ADR-0255.
- **Smart reply**: shows 3 short suggested replies above the reply box. One tap inserts into compose; user edits and sends. Per-user opt-in.
- **Answer for me**: per Superhuman; drafts a full reply, user reviews + sends. Opt-in; explicit-consent required at first use; every use is audit-emitted (`MailLlmDraftCreated`) so the user can see model touch-points.

### 4.5 Anti-tracking, anti-phishing

- Remote images **blocked by default**; "Show images from <sender>" tap opens an allow-once vs always-for-this-sender prompt.
- Tracking pixels (1×1 transparent GIFs, suspicious-domain image hosts, EV-prefix URLs) flagged and stripped.
- Links rewritten through `oya-mail-redirector` only for *known-tracker* domains; user sees the original URL on hover. Other links pass through unmodified.
- "This sender uses trackers" indicator shown when ≥3 tracking attempts detected.
- Lookalike-domain warning when sender domain is one-edit-distance from a contact (`acme.co` vs `acme.com`).

### 4.6 Sub-addressing, aliases, burners

- `alice+anything@oyatie.app` routes to `alice@oyatie.app` (RFC 5233 standard).
- User can create aliases via Settings → Aliases (`alice.shop@oyatie.app`).
- Burner addresses via Settings → Burners (`alice.<random>@oyatie.app`); auto-generated per service; revocable; one-tap delete revokes future deliveries. Apple Hide-My-Email parity.

### 4.7 Snooze, schedule send, undo

- Snooze: precise time picker + presets ("Tomorrow morning", "This weekend", "Next month", "Someday"). Snoozed mail returns to Inbox at the chosen time.
- Schedule send: timezone-aware ("9am their time"; uses recipient's last-known timezone from past mail). User explicit-confirms when scheduling outside their own waking hours.
- Undo send: 30s default (configurable 5–120s in Settings). Mail held in client-side outbox; pressing "Undo" returns to compose.

### 4.8 Threading and conversation view

- Gmail-style threading default: messages grouped by Message-ID/In-Reply-To/References + subject fallback (RFC 5322 + JMAP Thread per RFC 8621).
- Flat view toggle in Settings.
- Inline reply box in conversation view.
- Quoted text auto-collapsed; "Show trimmed content" reveals.
- Mute thread: future replies routed to Archive automatically (mute reason audit-trail kept for user reference).

### 4.9 Encryption

- **In transit**: TLS 1.3 mandatory for submission and access; opportunistic STARTTLS on :25 inbound.
- **At rest**: tenant DEK envelope per ADR-0255; B2C user-derived DEK so even oyatie operators cannot decrypt without user passkey.
- **S/MIME**: full RFC 8551 sign + verify + encrypt; certificates managed via Settings → Certificates; user can import existing or request via cloud-ca µservice.
- **OpenPGP**: RFC 4880 + RFC 9580 (crypto-refresh); WKD discovery (RFC 9580 §11.7); Autocrypt Level 1 header processing.
- **MLS-mail (opt-in)**: when both ends are `@oyatie.app`, opportunistic E2E using MLS-style group keys (analogous to Messenger E2E per the `messenger` µservice). Store-and-forward semantics differ from chat: server holds ciphertext + envelope key wrapped per recipient device.

### 4.10 Mobile and desktop clients

- iOS / Android native (Swift + Kotlin); JMAP under the hood.
- Tauri-built desktop (macOS / Windows / Linux), one codebase with the web client.
- Offline mode: JMAP cache + local SQLite encrypted with OS keychain; sync on reconnect.
- Push: APNs + FCM + Web Push; respects user's per-account notification preferences (e.g., personal silent during work hours).

### 4.11 Migration from competitors

Import from Gmail / Outlook / Apple Mail / Proton / Fastmail / any IMAP source:

- OAuth (Gmail / Microsoft Graph) or app-password (IMAP) per source.
- Preserves: Message-ID, internal-date, labels (Gmail labels → oyatie labels), folder structure, read/unread state, starred, attachments, source hash (for de-dup).
- Audit-chain emission per batch.
- DSAR-compliant: source remains source-of-truth until user confirms cutover.
- Reverse-export: `.mbox` and `.eml` Takeout for GDPR Art. 20 portability.

---

## 5. Work Mode (B2B) — Feature Deep-Dive

### 5.1 Shared mailboxes + delegation

- **Shared mailbox** (`support@acme.com`, `sales@acme.com`): a mailbox owned by the tenant; multiple users granted Cedar-policy access. Replies can be sent "as" the shared identity. Per-user assigned/claimed-by-X markers.
- **Delegation**: user-A grants user-B Send-On-Behalf or Send-As authority over mailbox-A. Audit-chained.
- **Substitution during OOO**: vacation responder optionally hands off triage to a delegate.

### 5.2 Distribution lists and groups

- Tenant-defined groups (`engineering@acme.com`, `all-hands@acme.com`).
- Static membership + dynamic (e.g., "everyone in HR org-unit").
- Cedar-policy gate on who can send to which group (e.g., `all-hands` restricted to leadership scope).
- Bounce + reputation per group.

### 5.3 Mail → Workflow Engine triggers

- Every mail rule (Sieve script) can emit a Workflow Engine event in addition to its standard Sieve action.
- Cedar-gated: tenant policy declares which classes of mail can trigger workflows (`policy/workflow-triggers.cedar`).
- Audit record on every trigger: source `Message-ID`, principal, policy-basis (rule-id), workflow item id.
- Example: rule `If sender == hr@acme.com AND subject contains "Approval needed"` → emit `WorkflowTriggerRequested{workflow=approval-routing, mail_ref=...}`.

### 5.4 Approval workflows (structured mail)

- A structured-mail attachment (JSON inside multipart with `application/oya-mail-approval+json`) carries an approval request.
- Receiving mail client renders an inline Approve / Reject button (HTML + JMAP custom-action extension).
- Click → workflow advance event; audit emission; mail thread updated with status.
- Use cases: expense reimbursement, time-off request, document publishing, code-deploy approval.

### 5.5 E-signing native (DocuSign / Adobe Sign parity)

- PDF attachment → "Send for e-signature" button → Workflow Engine `oya-mail-esignature` flow.
- Cryptographic signature: AdES-compliant (PAdES per ETSI EN 319 142; per ADR-0245 substrate-layered into `cloud-ca` + `audit-chain`).
- Auditable signing trail: signer identity (OIDC), wall-clock, signature digest, certificate chain.
- Multi-signer: parallel + sequential routing.
- eIDAS 910/2014 AdES posture for pack-eu tenants; KR 전자서명법 for pack-kr.
- Returns countersigned PDF to all parties; archive copy retained per retention policy.

### 5.6 Meeting invites and calendar integration

- Receive iCalendar invite (RFC 5545 `METHOD:REQUEST` per RFC 6047 iMIP); rendered inline.
- Accept / Tentative / Decline buttons emit iMIP `METHOD:REPLY` and update the `calendar` µservice.
- Send invite from mail: compose → "Add meeting" → calendar pickup; invite generated via `calendar` µservice + iMIP send.
- One-click Meet/Zoom/Teams join URL detection + render.
- Recurring events: full RFC 5545 RRULE/EXRULE/EXDATE support via `calendar` µservice; mail just renders the invite.

### 5.7 Clocking-in / HR via mail rules

- Tenant defines a Sieve+Workflow rule: "When mail from `<employee>@acme.com` arrives at `clockin@acme.com` between 7:00 and 11:00 local time, emit a timesheet entry."
- Workflow Engine adapts the event to `oya-hr-payroll` µservice (per ADR-0245 cross-product flows via Workflow + Ontology only).
- Sister rules:
  - Approval routing on `subject == "PTO request"` → `oya-hr-payroll-pto` flow.
  - Expense receipts forwarded to `expense@acme.com` → OCR + Ontology Expense object.
  - On-call escalation → Workflow `oya-incident-on-call` page.

### 5.8 DLP (data-loss prevention)

- Outbound scan for: PII patterns (SSN, RRN, credit-card, IBAN, passport), PHI patterns (when pack-us-healthcare), classification labels (Confidential / Restricted), keyword lists per tenant.
- Cedar gates: `policy/dlp.cedar` per tenant declares allow / warn / block per pattern-class per recipient-domain.
- Block: outbound message refused at submission; user shown DLP reason.
- Warn: user shown override prompt; override requires reason; audit-chained.
- Quarantine: message held for tenant-admin review.
- Inbound scan optional (detect inbound credentials, suspect attachment macros).

### 5.9 Retention, legal hold, eDiscovery (per existing PRD §"Tenant Value")

(See §13 Compliance and §11 Bounded Contexts. This subsection summarises.)

- Retention policy per-mailbox or per-tenant; statutory floors enforced by `oya-check-retention-floor-conformance` CI lane.
- Legal hold: scoped (by mailbox / date / query); hold-before-purge invariant; four-eyes for plaintext disclosure.
- eDiscovery: sealed export bundle; Ed25519 chain-of-custody; re-derivable digest; time-bound download URL.

### 5.10 Audit log

- Per-tenant audit-chain (Merkle + Ed25519 per ADR-0028 + ADR-0241).
- Events: `MessageReceived`, `MessageSent`, `MessageReadByUser`, `MessageReadByDelegate`, `MailboxDelegated`, `RuleCreated`, `LegalHoldEngaged`, `LegalHoldReleased`, `EDiscoveryExportSealed`, `DlpBlockApplied`, `DlpOverrideUsed`, `MailWorkflowHandoffCreated`, `EsignatureCreated`, `EsignatureCompleted`, `RetentionExpired`, `MailDeliverabilityReputationChanged`.
- Searchable via tenant compliance UI; exportable as part of any compliance audit.

### 5.11 SSO + provisioning

- Zitadel SAML 2.0 (RFC SAML-Core 2.0) + OIDC (RFC 6749 + OIDC 1.0).
- SCIM 2.0 (RFC 7644) for user provisioning + deprovisioning.
- MFA enforced per tenant policy (passkey-preferred; TOTP fallback).
- JIT vs pre-provisioned configurable.

### 5.12 Admin console

- Tenant admin UI under `https://admin.<tenant>.oyatie.app/mail`.
- Surfaces: users, mailboxes, aliases, domains, DKIM/SPF/DMARC, deliverability dashboard, retention policy, legal holds, eDiscovery jobs, DLP rules, Sieve rules library, audit-log search, quota usage, FinOps (per-mailbox cost + IP pool reputation).

### 5.13 Public mail address vs internal

- B2B tenants get default `<user>@<tenant>.com` (or chosen domain). Optional aliasing to `@<tenant>.oyatie.app` for testing.
- Internal-only addresses (e.g., `cron@acme.internal`) routable only within tenant.

---

## 6. User Stories (20+, step-by-step)

### Story 1 — Alice (Personal) composes new email with photo attachment + scheduled send

**Precondition.** Alice has signed up; her personal mailbox is `alice@oyatie.app`. She is in the web client on macOS Chrome.

**Steps.**
1. Alice presses **C**. Compose dialog opens (focus in To: field). Latency: <100ms.
2. Alice types `bob@example.org`, Tab → Subject: `Trip photos`, Tab → body: `Here are last weekend's photos`.
3. Alice drags 6 JPEG files (total 38 MB) into the compose body. Client uploads to SeaweedFS via JMAP `Blob/upload`; progress bar.
4. Each blob is virus-scanned by `attachment-handler`; safe; manifest returned.
5. Alice clicks the schedule-send arrow → "Tomorrow 9am Bob's time". Client resolves "Bob's time" via Bob's last-known TZ from prior mail (or falls back to Alice's TZ with a warning).
6. Alice presses **Cmd+Shift+Enter**. Mail enters scheduled-send queue. Audit emits `MessageQueuedForSchedule`.
7. At 9am Bob's time, the `outbound-smtp` worker picks up the message, DKIM-signs, submits to Bob's MX.
8. On 2xx response, audit emits `MessageDelivered`.

**Expected behaviour.** Single message delivered tomorrow; attachments preserved; Alice sees the queued state in Drafts → Scheduled folder; she can cancel until the dispatch instant.

**Edge cases.**
- Bob's MX returns 4xx (greylist): retry per RFC 5321 backoff (5min, 30min, 2h, 4h, 1d, fail). Status reflected in Sent → Scheduling-issues.
- Total attachment > 150 MB: client refuses at upload; offers SeaweedFS-link mode (sends a `https://attach.oyatie.app/<blob_id>` signed URL instead of inlining).
- Attachment fails virus scan: blocked with "This attachment is flagged as malware (signature: …). Send anyway?" requires explicit "I understand" + audit-chained override.

**Error cases.**
- Network drop mid-upload: client retains draft + uploaded-blob refs; resume on reconnect.
- TZ unresolvable: warning, default to Alice's TZ.

### Story 2 — Bob (Personal) snoozes 5 mails to weekend

**Precondition.** Bob has 30 unread mails Friday evening. Five are non-urgent newsletters.

**Steps.**
1. Bob presses **/** → searches `is:unread from:newsletter`.
2. Filter view shows 5 results.
3. Bob presses **X** to multi-select first; then **X** on remaining 4 (or Shift+Click).
4. Bob presses **H** → snooze picker shows; he picks "This weekend (Saturday 10am)".
5. All 5 disappear from Inbox; appear in Snoozed view.
6. Saturday 10am: messages move back to Inbox; bell-icon push notification batched.

**Expected.** Five messages disappear immediately, reappear together Saturday morning.

**Edge cases.**
- One mail is also starred: snooze preserves star; on return, star remains.
- Bob un-snoozes one before Saturday: it returns to Inbox immediately; audit log unchanged for personal use.

### Story 3 — Carol (Work) sets up shared mailbox `support@acme.com` with team delegation

**Precondition.** Carol is Acme's mail admin. Acme has 12 employees.

**Steps.**
1. Carol opens Admin Console → Mailboxes → New Shared Mailbox.
2. Fills: address `support@acme.com`, display name `Acme Support`, language `en`, timezone `America/New_York`.
3. Selects members: Dan, Eve, Frank (support team) — granted `read + reply-as` Cedar role.
4. Selects auto-assignment rule: round-robin between Dan/Eve/Frank; on assignment, the thread tagged `assigned:<user>`.
5. Sets auto-acknowledgement template: "Thanks for contacting Acme Support; case <id> created."
6. Saves. Audit emits `SharedMailboxCreated`, `SharedMailboxMemberGranted` × 3.
7. Within 2 minutes, MX records propagate (DKIM/SPF/DMARC already set for tenant); first inbound `support@acme.com` mail receives auto-ack and is assigned to (say) Dan.
8. Dan replies "Looking into this now"; reply sends from `support@acme.com` (Reply-As); audit emits `MessageSentAsSharedMailbox{by_user=dan, mailbox=support@acme.com}`.

**Expected.** A new shared mailbox is live with auto-ack + round-robin. Dan can reply-as without his personal mail going public.

**Edge cases.**
- Dan is OOO: round-robin skips Dan; assigns to Eve or Frank.
- All three OOO: assignment falls back to escalation address (set during config).

### Story 4 — David (Work) creates mail rule: when email from boss with subject 'Approval' arrives, trigger approval workflow

**Precondition.** David is a manager at Acme; he reports to Helena. Helena sends approval requests as mail.

**Steps.**
1. David opens Settings → Rules → New Rule.
2. Conditions: `from == helena@acme.com AND subject contains "Approval needed"`.
3. Actions: `Label: pending-approval`, `Move to: Approvals folder`, `Trigger workflow: oya-approval-routing { context: from-mail }`.
4. Saves. Tenant Cedar policy permits this trigger class for David's scope; rule saved as Sieve script + Workflow trigger.
5. Helena sends an approval mail.
6. Mail arrives in David's mailbox, hits rule, fires `WorkflowTriggerRequested`; `workflow-engine` instantiates `oya-approval-routing` with mail-ref as input.
7. Workflow extracts approval payload (LLM-assisted; opt-in; DP-noise); presents David an inline "Approve / Reject / Need info" button in the mail body.
8. David clicks Approve. Workflow advances; downstream system updated.

**Expected.** Approval workflow runs end-to-end from mail.

**Edge cases.**
- Mail subject is `"Approval needed (revised)"` — matches; same rule fires; workflow detects revision and supersedes prior instance.
- Helena CC's a colleague: rule still fires once per David's mailbox.
- David revokes the rule mid-flow: future mails skip workflow; in-flight stays.

### Story 5 — Erin (Personal) receives spam, marks as junk, learns block-sender

**Precondition.** Erin uses Personal Mail; never seen this sender before.

**Steps.**
1. Erin opens "Cheap Pharma" mail; subject screams scam.
2. Erin presses **!** (mark-as-spam shortcut).
3. Mail moves to Spam folder; Bayesian classifier records the user-feedback for her personal model; audit (personal-scope) emits `UserMarkedAsSpam`.
4. Toolbar shows "Also block sender? (B)". Erin presses **B**.
5. Sender domain `pharma-deal.ru` added to her personal block list; future mail from this domain auto-routed to Spam without inbox impression.
6. Erin closes the mail.

**Expected.** Spam learned; sender blocked; no future inbox impression from `pharma-deal.ru`.

**Edge cases.**
- Erin un-spams later (the mail was actually legit): classifier reverses; sender block lifted with explicit confirmation.

### Story 6 — Frank (Work) e-signs a contract attached in email

**Precondition.** Frank is in Acme legal; receives a contract PDF that needs his signature.

**Steps.**
1. Frank opens mail; sees PDF attachment `MSA-with-Foo-Corp.pdf`.
2. Frank clicks PDF → previews via `attachment-handler`. Toolbar shows "Send for e-signature".
3. Frank clicks → opens `oya-mail-esignature` workflow modal.
4. Frank adds his signature field (drag-drop on PDF) and a counterparty signature field; selects counterparty `legal@foo-corp.com`.
5. Frank clicks "Send for signature". Workflow:
   - Generates a signed envelope (AdES-compliant).
   - Mails counterparty with a one-click signing link.
   - Tracks signature status in workflow state.
6. Counterparty signs (via signed link). Workflow advances.
7. Counter-signed PDF returned to Frank's mailbox; audit emits `EsignatureCompleted{contract_id, signers}`.
8. Frank files the PDF; archive copy retained 7 years per Acme's retention policy.

**Expected.** Two-party signed PDF lands in Frank's inbox in <2 days; legally enforceable per AdES.

**Edge cases.**
- Counterparty doesn't sign in 7 days: reminder sent; after 30 days, workflow timeout + Frank notified.
- Counterparty rejects: workflow ends; both parties notified; audit emits `EsignatureRejected`.

### Story 7 — Gina (Personal) uses smart-reply on a meeting invite

**Precondition.** Gina receives a meeting invite from a friend.

**Steps.**
1. Gina opens mail; sees iCalendar invite inline ("Saturday brunch 11am").
2. Three smart-reply chips appear under the invite: "Sounds great!", "Can we move to 12?", "I'll get back to you".
3. Gina taps "Sounds great!"; reply pre-filled in compose.
4. She presses Accept (iCalendar) too; iMIP REPLY is also sent; calendar is auto-added to her `calendar`.
5. She sends.

**Expected.** Friend gets reply + iCalendar acceptance; Gina has the event on her calendar.

**Edge cases.**
- Recurring event invite: Gina chooses "Accept all" or "Accept this only".
- Invite is from a colleague but routed to her personal mailbox: invite rendering is identical; her acceptance reply uses her personal identity.

### Story 8 — Henry (Work) sets up vacation responder + delegation while away

**Precondition.** Henry is going on a 2-week vacation.

**Steps.**
1. Henry opens Settings → Vacation responder.
2. Configures: start `2026-07-01`, end `2026-07-14`, message `"I am out. For urgent matters, contact Iris."`
3. Toggles: send to internal-only colleagues; do NOT send to external (avoid auto-reply loops).
4. Enables delegation: Iris@acme.com granted Send-On-Behalf for the period.
5. Saves. Audit emits `VacationResponderEngaged` + `DelegationGrantedForPeriod`.
6. On 2026-07-01 00:00 Henry's TZ: responder activates.
7. Mail arrives → auto-reply queued (rate-limited 1 per sender per 4 days per RFC 3834).
8. Iris can reply-on-behalf-of-Henry; her replies bear `Reply-On-Behalf-Of: Henry`.
9. On 2026-07-14 23:59: responder + delegation auto-expire.

**Expected.** Auto-reply sent to internal-only; Iris handles urgent on-behalf-of; auto-expires.

**Edge cases.**
- Henry checks mail anyway during vacation: vacation responder remains active; manual reply does not double-send the responder.

### Story 9 — Iris (Personal) migrates from Gmail; imports mailboxes via IMAP

**Precondition.** Iris has 12 years of Gmail mail (~50 GB); wants to move to oyatie.

**Steps.**
1. Iris opens Settings → Import → "Import from Gmail".
2. OAuth flow with Google; she grants read-only mail scope.
3. Migration adapter spins up; estimates 50 GB / ~1 M messages / ~24 h with parallel fetch.
4. Iris sees a progress dashboard: messages migrated, failed, retried.
5. Migration preserves: Gmail labels → oyatie labels; starred → starred; threading via Message-ID; attachments via SeaweedFS dedup.
6. On completion (~24 h): summary "1,032,415 messages migrated; 12 retried; 3 failed (oversized attachments archived to SeaweedFS-link)".
7. Iris reviews the 3 failures.
8. Iris keeps her Gmail mailbox active for 90 days (auto-forwarding from Gmail to oyatie.app) until cutover.

**Expected.** Full Gmail archive in oyatie mail; threading preserved; user controls cutover.

**Edge cases.**
- Gmail rate-limits the IMAP fetch: adapter backs off, resumes; ETA may extend.
- Oversize attachment (>200 MB): archived to SeaweedFS-link; message kept; link inserted.

### Story 10 — Jack (Work) clocks in via dedicated mail address

**Precondition.** Acme uses oyatie mail rule for clock-in. Jack works at a field site with only mobile data.

**Steps.**
1. Jack opens Mail mobile, sends a 1-line mail `clockin@acme.com` body `start`.
2. Sieve+Workflow rule on tenant: matches sender `jack@acme.com`, recipient `clockin@acme.com`.
3. Workflow Engine emits `TimesheetEntryRequested{user=jack, kind=clock-in, at=<wall-clock>, geo=optional-from-tail-mime}`.
4. `oya-hr-payroll` µservice (consumed via Workflow event per ADR-0245) records the entry.
5. Jack receives an auto-ack mail "Clocked in at 07:42 EST".

**Expected.** Timesheet entry; auto-ack.

**Edge cases.**
- Sender not on payroll: rule rejects with bounce + audit.
- Geo not supplied: entry recorded without geo; tenant policy may require geo and reject otherwise.

### Story 11 — Kim (Personal) gets a phishing mail; client warns

**Precondition.** Kim's personal mailbox.

**Steps.**
1. Inbound mail arrives with `From: support@apple.com` but envelope-from `attacker@evil.tk`.
2. DKIM verify: fails. SPF: fails. DMARC: `p=reject` per apple.com → mail refused at receiver.
3. Refusal logged; sender never reaches Kim.

**Alternate.** Subtle attacker uses lookalike `app1e.com`:

4. DKIM/SPF/DMARC pass for `app1e.com` (their domain).
5. Heuristic detects one-edit-distance from `apple.com` (Kim's contact list).
6. Mail delivered with **"Lookalike sender" warning banner** above body.
7. Kim sees warning, reports phishing via **!P** shortcut; mail moves to Spam + reputation entry recorded; future `@app1e.com` mail auto-quarantined.

**Expected.** Most phishing blocked at protocol layer; subtle attacks flagged; user reporting feeds reputation.

### Story 12 — Liam (Work) outbound message hits DLP

**Precondition.** Liam at Acme tries to email a credit-card number to an external party.

**Steps.**
1. Liam composes mail to `vendor@foo.com`, body contains `Card: 4532-...-...-9012`.
2. Liam presses Send.
3. DLP scan finds credit-card pattern; tenant policy: block external.
4. Send refused; modal: "DLP block: credit-card number to external recipient. Reason required to override; manager approval required."
5. Liam abandons; copies to a tokenisation tool; resends with token. DLP passes.
6. Audit emits `DlpBlockApplied{reason=credit_card, recipient=vendor@foo.com}`.

**Edge cases.**
- Tenant configures warn-only: Liam can override with reason; audit logs the override.
- False positive (random 16-digit ID): user reports; tenant admin reviews the rule.

### Story 13 — Maya (Personal) searches "tax receipts 2025"

**Precondition.** Maya has thousands of mails.

**Steps.**
1. Maya presses **/** → types `tax receipts 2025`.
2. Search hits the encrypted-token index (per IP-009); query tokens HMAC'd per-tenant; matches surface in 50ms p50.
3. Result list: 27 matches, ranked by relevance + recency, with snippet preview.
4. Maya filters with `has:attachment from:irs.gov` → 8 results.
5. She clicks one to open.

**Expected.** Fast, accurate; plaintext never touches the index node.

**Edge cases.**
- Maya searches a misspelled term: client suggests "did you mean…"; expansion via stem-token table (per tenant, encrypted).

### Story 14 — Nora (Work) responds to GDPR DSAR

**Precondition.** Acme's customer requests all data Acme holds about them.

**Steps.**
1. Nora (Acme DPO) opens Admin Console → Privacy → DSAR.
2. Enters subject identity: `customer.x@example.com`.
3. Workflow scans tenant data; mail-µservice contributes: any message to/from/cc/bcc that address; aggregated audit-chain references.
4. Nora reviews the produced bundle (sealed); approves disclosure.
5. Customer downloads bundle within DSR-deadline (30d).
6. Audit emits `DsrFulfilled{subject, requester, scope, expires_at}`.

**Edge cases.**
- Subject also has Personal-context mail in same domain: Personal-pillar mail is **NOT** included; only the tenant-scoped Professional mail (per ADR-0215 cross-pillar invariant).

### Story 15 — Owen (Work) compliance officer engages legal hold

**Precondition.** Acme is in litigation; ops-legal asks Owen to preserve mailboxes for VP-John and three direct reports for case `2026-CV-1234`.

**Steps.**
1. Owen opens Admin Console → Compliance → New Legal Hold.
2. Scope: `mailboxes IN (john@acme.com, ...) AND date >= 2025-01-01`.
3. Approval: Owen submits; co-signer Petra (also compliance-officer scope) approves within 5 min.
4. Hold engages within 2s; audit emits `LegalHoldEngaged{hold_id, scope, approved_by=[owen,petra]}`.
5. Retention sweeper now skips matching messages.
6. Months later, Owen requests an eDiscovery export of the scope; four-eyes again required for plaintext disclosure; sealed bundle delivered with Ed25519 chain-of-custody.

**Expected.** No matching mail is purged; export bundle survives audit.

### Story 16 — Patrick (Personal) sets up custom domain

**Precondition.** Patrick owns `patrick-blog.com` and wants `me@patrick-blog.com`.

**Steps.**
1. Settings → Domains → Add custom domain.
2. Patrick enters `patrick-blog.com`; client shows MX, DKIM (oya._domainkey), SPF, DMARC, MTA-STS DNS records to add.
3. Patrick adds records at his registrar; presses Verify.
4. `cloud-network-dns` validates; takes <5 min.
5. Patrick adds `me@patrick-blog.com` as alias → his personal mailbox.
6. He composes; From defaults to `me@patrick-blog.com`.

### Story 17 — Quinn (Work) reads via Apple Mail using IMAP

**Precondition.** Quinn likes Apple Mail; his tenant gave him a work mailbox at `quinn@acme.com`.

**Steps.**
1. Quinn opens Apple Mail → Add Account → Other → IMAP.
2. Host: `imap.oyatie.app`. SSL: yes. Port: 993. Username: `quinn@acme.com`. Password: app-specific password generated via Settings → App Passwords.
3. SMTP host: `smtp.oyatie.app`. SSL: yes. Port: 465. Same credentials.
4. Apple Mail logs in; IMAP4rev2 SELECT/FETCH; mailbox view appears within 5s.

**Expected.** Apple Mail works exactly as with iCloud. JMAP-native clients get more features; IMAP works for legacy.

**Edge cases.**
- Tenant disables IMAP (OAuth-only policy): connection refused; user directed to JMAP-capable client.

### Story 18 — Rita (Personal) reports a sender as a tracker abuser

**Precondition.** Rita opens a marketing mail.

**Steps.**
1. Rita sees "This sender uses trackers" banner (3+ tracker pixels detected over recent mails).
2. She taps the banner → details: list of tracker pixels detected.
3. Tap "Unsubscribe (one-click)" → uses RFC 8058 List-Unsubscribe-Post header; service receives signed unsubscribe; future mails should stop.
4. Tap "Block sender domain" → adds to personal block list.

### Story 19 — Sam (Work) sends large file via SeaweedFS link

**Precondition.** Sam wants to send a 500MB video to a vendor.

**Steps.**
1. Sam composes; attaches video. Client warns: ">150MB; will use signed link (auto-expires 14d)".
2. Sam accepts. File uploads to SeaweedFS; signed URL inserted in body.
3. Recipient receives small (<50KB) mail with link.
4. Recipient clicks → SeaweedFS authenticates via tenant-issued signed URL → downloads file.
5. Sam can revoke link before expiry via Sent → Attachments.

### Story 20 — Tara (Personal) two-factor auths her client login

**Precondition.** Tara opens oyatie mail on a new device.

**Steps.**
1. Web client redirects to Zitadel sign-in.
2. Tara enters email; passkey prompt (WebAuthn); Tara taps her hardware key.
3. Sign-in completes; new device added to her account; audit emits `NewDeviceSignedIn{ua, ip, time}`.

### Story 21 — Ursa (Work) admin onboards a new employee

**Precondition.** Ursa is Acme HR admin; new hire Victor starts Monday.

**Steps.**
1. Ursa (or SCIM auto-sync from HRIS) creates Victor's identity.
2. Mail-µservice receives `TenantUserOnboarded` event → provisions `victor@acme.com` mailbox; default folders + signature template.
3. Victor receives a welcome mail with sign-in instructions.

### Story 22 — Will (Personal) reports a thread-bombing harasser

**Precondition.** A harasser sends Will dozens of mails per day.

**Steps.**
1. Will multi-selects 10 such mails.
2. Presses **!P** (report phishing/harassment) + **B** (block sender) + **D** (delete).
3. Sender domain blocked; reputation entry; harassment report flagged to ops-trust-safety; audit-chained.

### Story 23 — Xara (Work) reviews quarantine

**Precondition.** Xara is Acme mail admin; checks tenant quarantine weekly.

**Steps.**
1. Xara opens Admin → Quarantine.
2. Sees DLP-quarantined and abuse-quarantined messages.
3. Reviews; approves three legit messages back to recipients (audit logged); leaves the rest quarantined.

### Story 24 — Yael (Personal) uses Hide-My-Email burner for sign-ups

**Precondition.** Yael signs up to a new newsletter.

**Steps.**
1. Yael creates burner `yael.xyz123@oyatie.app` via Settings.
2. Pastes burner into the newsletter form.
3. Newsletter mail arrives in her inbox, tagged with the burner alias.
4. Months later, the newsletter starts spamming; Yael revokes the burner; future mails bounce.

---

## 7. UX strive-for / avoid

### 7.1 Strive

- **Keyboard-first.** Every action achievable without a mouse (Superhuman parity). J/K navigate; single-key actions (E, U, A, F, R, S, L, X).
- **100ms target latency on every interaction.** JMAP cache, optimistic UI, server-streamed updates over JMAP Push.
- **Threaded view by default for personal**; focused inbox option for work (tenant-configurable).
- **Aggressive prefetch.** Next ~10 messages prefetched in background as user reads.
- **Offline drafts.** Local SQLite encrypted with OS keychain. Sync on reconnect.
- **Smart notifications.** Per-account notification rules; respect Do Not Disturb; per-thread mute.
- **Inline image lazy-load.** Image bytes fetched only on user gesture (after anti-tracking check).
- **Attachment preview without download.** PDF, image, Office, video, code (with syntax highlighting) all preview in-place.
- **Composability with broader oyatie suite.** Mail integrates natively with Messenger, Calendar, Meet, Workflow, Tasks, Notes, Drive, HR/Payroll, Plugin App Store.
- **Honest deliverability.** Show the user "your last 90 days: 98.4% delivered" with breakdown, instead of hiding bounces.
- **Per-account context switching is one keystroke.** Personal ↔ Work in <50ms with no full-app reload.

### 7.2 Avoid (anti-patterns)

- **Tracker-pixel rendering by default.** Always block remote images at first sight; user must opt-in per sender.
- **Auto-mark-as-read on hover.** Mail is marked read only by explicit gesture (open, J/K).
- **Auto-archive after send-receive-return.** Some clients aggressively archive replied threads; this destroys workflow visibility. Default: archive only on explicit E.
- **Ad insertion.** Never inject sponsored or "Promotions" mails into Inbox. Gmail's Promotions tab is an anti-pattern when invisible to UX; never replicate.
- **Reading-pane ad insertion.** Never.
- **Persistent identity disclosure in headers.** No `X-Mailer: oyatie/<version>+<user_geo>` style leaks.
- **Cross-tenant data leakage.** Kernel-layer ContextBoundaryGuard refuses every cross-context routing.
- **Over-eager filters.** Don't auto-route to spam threads with one bad-sender hit in a conversation that's otherwise healthy.
- **Auto-loading remote content.** Remote CSS, remote images, remote iframes all blocked unless user-permits.
- **Modal pre-loading.** No "Click to enable" interstitials over mail content. Settings live in Settings.
- **Vague spam reasons.** When we mark something spam, we tell the user why ("DKIM failed", "DMARC reject", "Bayesian-spam-score 0.92").
- **Tracker-pixel allow-by-default for "newsletters".** No exception; user opts in per-sender.
- **One-click unsubscribe trackers.** Unsubscribe via RFC 8058 List-Unsubscribe-Post, not via tracker URL hops.
- **Forward-by-default to external.** External forwarding off by default in B2B; user must enable + tenant must permit.
- **AI hallucination shipping as truth.** LLM-generated replies/summaries always shown as drafts; user reviews before send.
- **Mining mail for advertising.** Never. Mail content is `PII_IDENTIFYING`. Use is bound by ADR-0008 data-use-boundary.
- **Forced upgrades that change UX without notice.** Material UX changes shipped behind a "What's new" interstitial + opt-out for 30 days.

---

## 8. Tenant Value (carried from prior PRD; refined)

- **Tenant Outcome 1 — Standards-compatible mail without vendor coupling.** Tenants get SMTP submission + SMTP relay + IMAP4rev2 + JMAP + REST; no Gmail/Outlook API dependency. Migration adapter ingests from any source.
- **Tenant Outcome 2 — Dual-context isolation by construction.** A user's Personal mailbox is invisible to org admins, legal-hold workflows, and eDiscovery export even when both share a cluster.
- **Tenant Outcome 3 — eDiscovery that survives audit.** Sealed exports with Ed25519 chain-of-custody, time-bound expiry, and four-eyes approval for plaintext disclosure.
- **Tenant Outcome 4 — Legal hold that survives retention.** Engaging a hold blocks retention expiry within scope at the kernel layer.
- **Tenant Outcome 5 — Mail-to-Workflow native.** A work mail becomes a Workflow task only with explicit user action or a tenant-declared policy basis. Every handoff emits an audit-chain record.
- **Tenant Outcome 6 — Personal product parity.** B2C users get Gmail/Outlook/Hey/Superhuman parity from day one. Personal mailboxes structurally invisible to org admins.
- **Tenant Outcome 7 — Cross-organization mail-server pattern.** Each tenant's logical mail-server is a partition (per-tenant-DEK, per-tenant SMTP IP reputation, per-tenant retention). Per ADR-0133.
- **Internal Outcome 8 — Dogfooding-grade.** oyatie itself runs entirely on `oyatie-corp` tenant of this µservice (per ADR-0242). Every dogfooding bug = paying-customer bug.

---

## 9. Substrate Dependencies

Per ADR-0245 substrate-vs-product layering: mail product builds on (consumes-by-event/Ontology-port, never direct-import-cross-product) the following:

| Dependency | Why | Interaction |
|---|---|---|
| `identity` (Zitadel) | Authn/MFA/SCIM/SAML/OIDC | All edges; B2C and B2B identity |
| `tenancy` | Tenant model + admission | TenantOnboarded / TenantOffboarded events |
| `cell` | Per-tenant home cell + DR pair | Mailbox shard placement |
| `audit-chain` | Ed25519 audit emission | Every mail event |
| `policy-engine` (Cedar) | Feature gating | DLP, retention, workflow trigger, dual-context |
| `observability` | Metrics + traces + logs | OTel SDK; per ADR-0130 / ADR-0131 |
| `cloud-secrets` (OpenBao) | DKIM keys, TLS certs, KMS DEK | Every TLS + DKIM op |
| `cloud-kms` (per ADR-0251) | encryption-BYOK + envelope encryption | Mailbox-store + S3 SSE |
| `workflow-engine` | Mail → workflow trigger | Event emission |
| `intelligence` | Smart compose, smart reply, summarisation | Per-tenant-cell LLM; opt-in |
| `ontology` | Object Type: MailMessage, Mailbox, LegalHold, etc. | Reads + writes |
| `messenger` | Mail ↔ Messenger bridge | Cross-channel event-only |
| `calendar` | iMIP + iCalendar invites | Event-only |
| `meet` | One-click join URL | Render-only |
| `hr-payroll` | Clocking-in, approvals, expenses | Workflow Engine indirect |
| `cloud-network-dns` | DKIM/SPF/DMARC/MTA-STS DNS auto-publish | Tenant onboarding |
| `cloud-ca` | S/MIME + AdES e-signature certificates | E-sign + S/MIME |
| `plugin-app-store` | Third-party mail rules + extensions | Sandboxed Cedar-gated |
| `seaweedfs` (cloud-data-blob) | MIME blob + attachment storage | All large blobs |
| `postgres` (cloud-data-citus) | Mailbox metadata | All metadata |
| `tantivy` (cloud-data-search) | Encrypted-token search index | Search |
| `rspamd` (Layer-A) | Anti-spam / anti-phishing | Inbound classifier |
| `clamav` (Layer-A) | Attachment AV | Inbound + outbound |

Cross-product rule: `mail` MUST NOT import any other product µservice crate at any layer (LEAN-A2). Cross-product flows go through Workflow (events) or Ontology (entity reads/writes).

---

## 10. Non-Functional Requirements

### 10.1 Performance (extends prior table)

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Compose-open latency | ≤50ms | ≤200ms | ≤500ms | client-side, JMAP cache primed |
| Send (submission → accepted) | ≤100ms | ≤200ms | ≤500ms | per directive |
| Fetch (JMAP Email/get latest 50 headers) | ≤40ms | ≤100ms | ≤300ms | per directive |
| Search (100k-message mailbox, encrypted-token) | ≤100ms | ≤500ms | ≤2s | per IP-009 |
| Inbound SMTP DATA → mailbox-persisted | ≤200ms | ≤1s | ≤3s | per Bominal ADR-0210 |
| Outbound submission → recipient MX | ≤5s | ≤30s | ≤5min | recipient-dependent |
| Smart compose suggestion | ≤50ms | ≤150ms | ≤500ms | streaming via tenant-cell LLM |
| Smart reply suggestion (3 chips) | ≤100ms | ≤300ms | ≤1s | precomputed on mail receipt |
| Sustained ingestion | — | 100k mails/sec per cell | — | per directive; cell-horizontal-scale |
| eDiscovery export 10y / 5GB | — | ≤24h | — | per prior PRD |
| Legal hold engage | ≤500ms | ≤2s | ≤5s | hold-before-purge invariant |
| Mailbox restore (PIT, 5GB) | — | ≤15min | — | RTO ≤15min, RPO ≤5min |
| Compose-to-Workflow handoff | ≤200ms | ≤500ms | ≤2s | tenant-visible UX |
| E-signature workflow round-trip (auto-sign-and-return) | — | ≤30min | — | tenant-side latency |
| DLP scan outbound | ≤50ms | ≤150ms | ≤500ms | blocks submission until clear |

### 10.2 Availability + SLO

- Personal Mail (B2C): T2 99.95% monthly (mail is critical).
- Work Mail (B2B): **T2 99.99% monthly** (per directive; mail outage = enterprise crisis).
- Inbound SMTP availability: 99.95% (RFC 5321 requires graceful retry on outage).
- Outbound delivery availability: 99.9% monthly.
- IMAP/JMAP availability: 99.95% monthly.
- eDiscovery export endpoint: 99.5% (admin tool; non-blocking).
- RTO: ≤15 min mailbox restore; ≤5 min SMTP frontend failover.
- RPO: ≤5 min (sync-replicated WAL).
- Error budget on inbound SMTP: 0.05% (≈22 min/month).
- Burn-rate alarm: 14.4× over 1h → page.
- Error budget policy in `runbooks/error-budget-policy.md`.

### 10.3 Storage

- **Postgres + Citus** for mailbox metadata (per-tenant RLS; Citus distributed by tenant_id when >80% single-node capacity).
- **SeaweedFS** for MIME blobs + attachments (per IP-001; SSE per-tenant DEK; content-addressable de-dup).
- **Tantivy / Quickwit** for encrypted-token search (per IP-009).
- **Per-tenant home_cell** with paired DR (cross-AZ + cross-region).
- **Per-pack residency**: pack-kr (KR), pack-eu, pack-us, pack-jp, pack-sg, pack-au, pack-in, pack-br, pack-ae, pack-ksa, pack-us-healthcare (HIPAA-eligible regions).

### 10.4 Anti-spam / anti-phishing

- **Rspamd** as primary anti-spam engine (Bayesian + greylisting + DNSBL + RBL + DKIM/SPF/DMARC + heuristics).
- **Per-user model** (B2C): user feedback fine-tunes their personal classifier.
- **Tenant rules overlay** (B2B): admin-defined keyword + sender + DLP rules layered.
- **URL reputation**: Spamhaus + SURBL + Google Safe Browsing API.
- **Lookalike-domain detect**: one-edit-distance from contacts → warn.
- **Anti-phishing heuristics**: brand-impersonation (well-known brand in display name, attacker domain), urgency words ("urgent", "verify now"), credential-asking forms in HTML.

### 10.5 Encryption

- **At rest**: per-tenant DEK envelope; encryption-BYOK in tenant KMS region per ADR-0251.
- **In transit**: TLS 1.3 everywhere (submission + IMAP + JMAP + REST). STARTTLS opportunistic on :25 inbound per RFC 8314.
- **MTA-STS** (RFC 8461) + **TLS-RPT** (RFC 8460) published per tenant.
- **DKIM** (RFC 6376 + Ed25519 per RFC 8463) outbound; verify inbound.
- **SPF** (RFC 7208) + **DMARC** (RFC 7489) outbound publish + inbound enforce.
- **ARC** (RFC 8617) for forwarded mail.
- **S/MIME** (RFC 8551) + **PGP** (RFC 4880 + RFC 9580 crypto-refresh).
- **MLS-mail** opt-in for `@oyatie.app↔@oyatie.app`.

### 10.6 Deliverability

- Per-tenant SMTP IP pool; warmed gradually.
- Per-tenant reputation tracker; auto-quarantine compromised pool members.
- Feedback-loop subscriptions with Gmail / Outlook / Yahoo / Apple iCloud postmasters.
- DKIM key rotation 90d.
- DMARC aggregate report ingestion + analysis surfaces tenant anomalies.
- Bounce categorisation: hard / soft / suspended; reputation impact.

### 10.7 JMAP server choice

Per Open Question 1 — pending ADR; default to **in-house Rust JMAP server** with optional **Stalwart** embed (`-adapter-stalwart`) per IP-001. Stalwart provides production-tested JMAP/IMAP/SMTP unified; in-house gives per-tenant policy + Cedar gate hooks. Decision deferred to per-component ADR.

### 10.8 DR posture (per ADR-0343)

- Manifest target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, `replication_shape=active-active-multi-az-cross-region-warm`. The older §10.2 5-minute SMTP and 15-minute mailbox objectives remain stretch SLOs, not the ADR-0343 manifest contract.
- Applicable pack floors from `specs/compliance-pack-floors.json`: EU-AI-ACT-2024-HIGH-RISK `1800s/300s` with multi-region required; HIPAA-2024 `3600s/300s` with multi-region required; KR-PIPA-2023 default `14400s/900s`; SOC2-T2 `14400s/900s`; ISO27001-2022 `14400s/3600s`; PCI-DSS-L1-v4 `86400s/3600s`. The effective maximum pack floor is `86400s/3600s`, but mail keeps the stricter tenant-visible target because notice delivery and legal hold are critical.
- `failover_runbook=runbooks/dr-failover.md`, resolved at `microservices/mail/runbooks/dr-failover.md`; backup substrates are `postgres_wal_g`, `object_storage_versioned`, `valkey`, `openbao_seal_unseal`, and `audit_chain_merkle_seal`.
- `multi_region_active_active=true` for ingress, submission, JMAP/IMAP reads, and queue acceptance; outbound MX delivery still respects recipient-domain retry semantics.
- Why: tenants depend on mail for legal notice, healthcare communication, passwordless verification, and incident response; failover must preserve accepted-mail evidence even when final delivery retries externally.

### 10.9 Capacity model (per ADR-0340)

- Per-tenant baseline: `0.18 vCPU`, `384 MiB RAM`, `25 GiB storage`, `connections_per_tenant={valkey:3, postgres:4, outbound_http:8}`.
- Scaling dimension: `per_message` for inbound/outbound SMTP, JMAP mutations, mailbox indexing, and queue processing.
- Cell placement class: `Tier-3` with manifest `pod_runtime_tier=2`; mailbox storage dominates footprint while SMTP/JMAP paths scale by message volume and retain hot state in Valkey.
- Autoscaling boundaries: min `3` edge/api replicas per tenant-cell, max `64` submission/JMAP replicas before mailbox shard split; eDiscovery workers use separate batch quotas so exports cannot starve inbox fetch.
- Why: mailbox tenants combine long-lived storage, bursty SMTP delivery, and high-cardinality search; this model isolates human inbox latency from admin export and anti-abuse workloads.

### 10.10 Sustainability + cost attribution (per ADR-0344)

- Each send, receive, DLP scan, search query, mailbox restore, eDiscovery export, and deliverability event audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider routing is carbon-aware for search reindex, spam-model batch recalculation, mailbox compaction, and eDiscovery exports; it is not carbon-routed for inbound SMTP acceptance, legal-hold enforcement, PHI notifications, or EU-AI high-risk mail safety decisions.
- Tenant cost transparency surface: mail admin exposes storage, send volume, search/eDiscovery export cost, DLP scan counts, and provider-delivery spend; rollups are reconciled in finops-portal.
- Why: mail is evidence-bearing communication, so CSRD, SB-253, and SEC climate-disclosure exports must distinguish human delivery, retention, legal hold, and scanning cost rather than hiding them in platform overhead.

### 10.11 API versioning posture (per ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet using `Oyatie-Version` header, `/v/YYYY-MM-DD/` REST/JMAP URL prefix for oyatie APIs, and proto3 field `string oyatie_version = 8001` for public event/contracts.
- SDK semver model: mail SDKs publish `major.minor.patch`; protocol adapters preserve SMTP/IMAP/JMAP RFC compatibility while oyatie extensions use the date carrier.
- Support window: last `N=3` public oyatie API versions for at least `180` days after deprecation.
- Per-tenant pinning: yes for enterprise tenants, regulated mail clients, and migration bridges that need a frozen JMAP/REST extension surface.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC over HTTP/3 remains tag-compatible and exempt from public carrier routing.

---

## 11. Bounded Contexts

Per existing PRD (extended). Total BCs grow from 8 to **17** to absorb the new personal+work feature surface. ADR-0105 13-layer enum applies.

| BC | Purpose | Key entities |
|---|---|---|
| `mailbox-store` | Mailbox + thread + message + folder model | `Mailbox`, `Thread`, `MailMessage`, `Folder`, `RetentionClass`, `MimeBlob` |
| `inbound-smtp` | SMTP :25 + :465; DKIM/SPF/DMARC/ARC verify; abuse classifier | `IncomingSession`, `DkimResult`, `AbuseVerdict` |
| `outbound-smtp` | SMTP :587 submission; DKIM sign; deliverability; bounce | `OutboundEnvelope`, `DeliveryAttempt`, `ReputationScore` |
| `imap-frontend` | IMAP4rev2 + JMAP + REST + ManageSieve | `ImapSession`, `JmapCommandBatch`, `SieveScript` |
| `jmap-frontend` | First-class JMAP (RFC 8620 + 8621 + 8639 + 8887) | `JmapCapabilityToken`, `JmapEventStream` |
| `search-index` | Encrypted-token search | `SearchToken`, `IndexShard`, `EncryptedQuery` |
| `attachment-handler` | SeaweedFS blob persist + virus scan + preview | `MimeBlob`, `AntiVirusVerdict`, `PreviewArtifact` |
| `anti-spam` | Rspamd integration + per-user model | `SpamVerdict`, `ClassifierModel`, `FeedbackEntry` |
| `anti-phishing` | URL reputation + lookalike detect + brand-impersonation | `PhishingVerdict`, `LookalikeMatch` |
| `dkim-spf-dmarc` | Outbound publish + inbound enforce + DMARC report ingest | `DkimKey`, `SpfRecord`, `DmarcPolicy`, `DmarcAggregateReport` |
| `threading` | Conversation grouping per RFC 5322 + JMAP Thread | `Thread`, `ThreadKey` |
| `rules-engine` | Sieve + Cedar-gated triggers | `SieveScript`, `RuleTriggerEvent` |
| `delegation-sharing` | Shared mailbox + delegation + send-on-behalf-of | `SharedMailbox`, `Delegation`, `MailRoleAssignment` |
| `signing-encryption` | S/MIME + PGP + MLS-mail | `SmimeCertificate`, `PgpKey`, `MlsGroupContext` |
| `workflow-triggers` | Mail rule → Workflow Engine event | `WorkflowTriggerSpec`, `WorkflowTriggerEvent` |
| `mail-to-messenger-bridge` | Mail ↔ Messenger | `MailMessengerBridge` |
| `dlp` | Work-mode outbound + inbound scan | `DlpRule`, `DlpVerdict`, `DlpOverride` |
| `archive-retention` | Retention policy + ledger + expiry sweep | `RetentionPolicy`, `RetentionLedgerEntry`, `ExpiryBatch` |
| `legal-hold` | Hold scope + four-eyes + chain-of-custody | `LegalHold`, `HoldApproval`, `EDiscoveryExportJob` |
| `e-discovery` | Sealed export bundle + signed download | `EDiscoveryBundle`, `ChainOfCustodySeal` |
| `dual-context-isolation` | Personal vs Professional kernel boundary | `ContextKind`, `ContextBoundaryGuard` |
| `migration-importer` | Import from Gmail/Outlook/IMAP | `MigrationJob`, `SourceProviderAdapter` |
| `deliverability-dashboard` | Per-tenant bounce/spam/DMARC report UI | `DeliverabilityMetric`, `BouncePattern` |
| `esignature` | PAdES/AdES e-signing | `SignatureRequest`, `SignatureCompletion` |

Crate count: ~150 crates (each BC has 6–11 layers). Build managed by `oya-workspace` and CI lanes per ADR-0131.

Cross-BC rule: BCs talk only via events emitted to `audit-chain` + Ontology, or through application-orchestrator usecase crates. No BC imports another BC's `kernel` or `domain`.

Port traits (extended from prior PRD; new ones bolded):

| Port trait | Kernel crate | Implemented in | Data classes |
|---|---|---|---|
| `MailboxRepository` | mailbox-store-kernel | -adapter-postgres | PII_IDENTIFYING |
| `MimeBlobStore` | mailbox-store-kernel | -adapter-seaweedfs | PII_IDENTIFYING + PHI when pack-us-healthcare |
| `RetentionLedgerWriter` | retention-policy-kernel | -adapter-postgres | AUDIT |
| `LegalHoldEngine` | legal-hold-kernel | -usecase | AUDIT |
| `EDiscoveryExporter` | e-discovery-kernel | -usecase | AUDIT + SENSITIVE_PIPA_ART23 |
| `SmtpInboundReceiver` | inbound-smtp-kernel | -adapter-smtp | PII_IDENTIFYING |
| `SmtpOutboundSubmitter` | outbound-smtp-kernel | -adapter-smtp | PII_IDENTIFYING |
| `ImapSessionHandler` | imap-frontend-kernel | -adapter | PII_IDENTIFYING |
| **`JmapMethodDispatcher`** | jmap-frontend-kernel | -adapter | PII_IDENTIFYING |
| `EncryptedTokenIndex` | search-index-kernel | -adapter-tantivy | BEHAVIORAL_TENANT_PRODUCT (encrypted tokens) |
| `DkimSigner` / `DkimVerifier` | dkim-spf-dmarc-kernel | -adapter | SECRET |
| `ContextBoundaryGuard` | dual-context-isolation-kernel | -usecase | AUDIT |
| `AbuseClassifier` | anti-spam-kernel | -adapter-rspamd | BEHAVIORAL_TENANT_PRODUCT |
| **`PhishingClassifier`** | anti-phishing-kernel | -adapter | BEHAVIORAL_TENANT_PRODUCT |
| **`DlpClassifier`** | dlp-kernel | -adapter | PII_IDENTIFYING |
| **`AntiVirusScanner`** | attachment-handler-kernel | -adapter-clamav | SECRET (engine signatures) + PII_IDENTIFYING (content) |
| **`PreviewRenderer`** | attachment-handler-kernel | -adapter | PII_IDENTIFYING |
| **`SieveCompiler`** | rules-engine-kernel | -adapter | INTERNAL_ONLY |
| **`WorkflowTriggerEmitter`** | workflow-triggers-kernel | -usecase | AUDIT |
| **`SignatureSigner`** | esignature-kernel | -adapter-cloud-ca | AUDIT |
| **`PgpKeyDiscovery`** (WKD + Autocrypt) | signing-encryption-kernel | -adapter | SECRET (private keys never leave) |
| **`MlsMailKeyAgreement`** | signing-encryption-kernel | -adapter | SECRET |
| **`DistributionGroupResolver`** | delegation-sharing-kernel | -adapter-postgres | PII_IDENTIFYING |
| **`MigrationProviderAdapter`** | migration-importer-kernel | -adapter-gmail / -adapter-microsoft-graph / -adapter-imap | PII_IDENTIFYING |
| **`DeliverabilityMetricsCollector`** | deliverability-dashboard-kernel | -adapter | BEHAVIORAL_TENANT_PRODUCT |

CI lanes (extended):

- All prior lanes (per existing PRD §"Bounded Contexts").
- New: `oya gate validate jmap-conformance` (RFC 8620/8621 method-set conformance).
- New: `oya gate validate imap-conformance` (RFC 9051 conformance against `imaptest`).
- New: `oya gate validate sieve-conformance` (RFC 5228 + RFC 5804).
- New: `oya gate validate anti-tracking-conformance` (remote images blocked by default; tracker patterns recognized).
- New: `oya gate validate dlp-cedar-coverage` (DLP rule fragment coverage).
- New: `oya gate validate dual-context-cross-boundary` (kernel-layer pillar invariant; already in prior PRD).
- New: `oya gate validate esignature-pades-conformance` (PAdES / AdES validity).

---

## 12. Acceptance Criteria (extended)

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Personal user signs up + receives mail end-to-end | scripted e2e drill `tests/e2e/personal-signup-receive.sh` |
| AC-02 | Work mailbox encrypted under tenant DEK + retention tagged | `cargo nextest -p oya-mail-mailbox-store-domain --test storage::test_work_mail_tenant_dek_and_retention` |
| AC-03 | Personal mailbox unreachable from Professional API | `cargo nextest -p oya-mail-mailbox-store-domain --test personal::test_org_admin_cannot_export_personal_mail` |
| AC-04 | Compliance officer engages legal hold; held messages survive retention | `cargo nextest -p oya-mail-legal-hold-domain --test test_hold_blocks_mail_deletion` |
| AC-05 | Tenant migrates Gmail → preserves source hash + labels + retention class | `cargo nextest -p oya-mail-migration-importer-app --test test_import_preserves_chain_of_custody` |
| AC-06 | Mail-to-Workflow handoff requires explicit policy basis; audit-emitted | `cargo nextest -p oya-mail-workflow-triggers-domain --test test_mail_to_workflow_requires_policy_basis` |
| AC-07 | Inbound DKIM verification on tampered message fails | `cargo nextest -p oya-mail-inbound-smtp-domain --test dkim::test_tampered_message_fails_dkim` |
| AC-08 | Outbound DKIM signs every message + SPF/DMARC alignment | `cargo nextest -p oya-mail-outbound-smtp-domain --test dkim::test_outbound_signs_with_dkim` |
| AC-09 | Encrypted search returns correct results without plaintext index | `cargo nextest -p oya-mail-search-index-domain --test encrypted_token::test_search_correctness_without_plaintext` |
| AC-10 | eDiscovery export of 10y archive completes in 24h SLA; digest verifies | scripted `tests/e2e/ediscovery-export.sh` |
| AC-11 | Hold engage in 2s; bypass attempts emit metric + page | `cargo nextest -p oya-mail-legal-hold-domain --test test_hold_engage_under_2s` |
| AC-12 | Cross-context routing forbidden; 403 + audit-emitted | `cargo nextest -p oya-mail-dual-context-isolation-domain --test test_cross_context_routing_refused` |
| AC-13 | Snooze precise + smart presets work end-to-end | `tests/e2e/personal-snooze.sh` |
| AC-14 | Schedule send works with recipient TZ resolution | `tests/e2e/personal-schedule-send.sh` |
| AC-15 | Undo send works within configurable window | `tests/e2e/personal-undo-send.sh` |
| AC-16 | Smart compose returns suggestion in p99 ≤150ms | `tests/perf/smart-compose-latency.sh` |
| AC-17 | Smart reply chips populated within p99 ≤300ms on mail receipt | `tests/perf/smart-reply-latency.sh` |
| AC-18 | Hey-style triage with new-sender screening; per-sender persistence | `cargo nextest -p oya-mail-triage-domain --test test_first_time_sender_screening` |
| AC-19 | Anti-tracking: remote image not loaded unless explicit allow | `cargo nextest -p oya-mail-anti-tracking-domain --test test_remote_image_blocked_by_default` |
| AC-20 | Sub-addressing (`user+tag@`) routes to base mailbox | `cargo nextest -p oya-mail-mailbox-store-domain --test test_subaddressing` |
| AC-21 | Burner alias revoke causes future bounce | `cargo nextest -p oya-mail-aliases-domain --test test_burner_revoke_bounces` |
| AC-22 | Vacation responder respects RFC 3834 dedup (4d-per-sender) | `cargo nextest -p oya-mail-rules-engine-domain --test test_vacation_dedup` |
| AC-23 | Vacation + delegation work together | `tests/e2e/work-vacation-with-delegation.sh` |
| AC-24 | Shared mailbox with round-robin assignment | `tests/e2e/work-shared-mailbox.sh` |
| AC-25 | DLP block on outbound credit-card pattern | `cargo nextest -p oya-mail-dlp-domain --test test_dlp_block_credit_card` |
| AC-26 | DLP warn-with-override audits the override | `cargo nextest -p oya-mail-dlp-domain --test test_dlp_warn_audit` |
| AC-27 | E-signature: two-party PAdES round-trip | `tests/e2e/work-esignature.sh` |
| AC-28 | iMIP invite render + accept inline; calendar updated | `tests/e2e/work-imip-accept.sh` |
| AC-29 | Clock-in via mail rule produces timesheet entry | `tests/e2e/work-clockin.sh` |
| AC-30 | SCIM provisioning creates a work mailbox in <60s | `tests/e2e/work-scim-provision.sh` |
| AC-31 | SSO via Zitadel OIDC works for IMAP + JMAP + REST | `tests/e2e/work-sso.sh` |
| AC-32 | IMAP4rev2 conformance against imaptest harness | `bash tests/imap-conformance.sh` |
| AC-33 | JMAP conformance against fastmail/jmap-test harness | `bash tests/jmap-conformance.sh` |
| AC-34 | Phishing: lookalike-domain detected and banner shown | `cargo nextest -p oya-mail-anti-phishing-domain --test test_lookalike_banner` |
| AC-35 | KR pack: 5y retention floor enforced for KR-FSS tenants | `cargo nextest -p oya-mail-retention-policy-domain --test pack_kr::test_kr_fss_5y_floor` |
| AC-36 | HIPAA pack: BAA absence refuses pack-us-healthcare onboarding | `cargo nextest -p oya-mail-mailbox-store-app --test pack_us_healthcare::test_baa_required` |
| AC-37 | GDPR DSAR export per-mailbox under 30d SLA | `tests/e2e/gdpr-dsar.sh` |
| AC-38 | GDPR Art. 17 erasure on personal account | `tests/e2e/gdpr-erasure-personal.sh` |
| AC-39 | Custom-domain DKIM/SPF/DMARC auto-publish + verify | `tests/e2e/custom-domain-setup.sh` |
| AC-40 | Per-tenant SMTP IP pool isolated; tenant A's abuse does not impact tenant B | `tests/e2e/ip-pool-isolation.sh` |
| AC-41 | Attachment AV (ClamAV) blocks EICAR test file | `cargo nextest -p oya-mail-attachment-handler-domain --test test_av_blocks_eicar` |
| AC-42 | Attachment preview without download for PDF/Office/image | `tests/e2e/attachment-preview.sh` |
| AC-43 | Burner / Hide-My-Email alias creation + revoke | `cargo nextest -p oya-mail-aliases-domain --test test_burner_lifecycle` |
| AC-44 | Sieve rule installed via ManageSieve runs on inbound | `tests/e2e/personal-sieve-rule.sh` |
| AC-45 | Search across all accounts in one personal session | `tests/e2e/personal-cross-account-search.sh` |
| AC-46 | Multi-account context-switch in <50ms | `tests/perf/context-switch.sh` |
| AC-47 | Offline draft persisted + synced on reconnect | `tests/e2e/offline-draft.sh` |
| AC-48 | Push notification delivered via APNs + FCM within p99 ≤2s after persist | `tests/perf/push-latency.sh` |
| AC-49 | One-click unsubscribe (RFC 8058) works | `tests/e2e/unsubscribe.sh` |
| AC-50 | `oya gate validate per-microservice-layout --microservice mail` exits 0 | ADR-0131 lane |
| AC-51 | `oya gate validate authority-cohesion` exits 0 | ADR-0123 lane; HG-MAIL registered |
| AC-52 | `oya gate validate jmap-conformance` exits 0 | new lane |
| AC-53 | `oya gate validate dual-context-cross-boundary` exits 0 | new lane |
| AC-54 | `oya gate validate retention-floor-conformance` exits 0 | new lane |
| AC-55 | `oya gate validate dkim-key-rotation-conformance` exits 0 | new lane |

---

## 13. Integration Points

| Producer | Event / API | Consumer | Direction | Notes |
|---|---|---|---|---|
| `mail.inbound-smtp` | `MessageReceived` | `audit-chain`, `messenger` (action-card-in-mail), `workflow-engine` (Cedar-gated) | event | per ADR-0245 |
| `mail.outbound-smtp` | `MessageSent`, `MessageDelivered`, `MessageBounced` | `audit-chain`, deliverability-dashboard | event | — |
| `mail.legal-hold` | `LegalHoldEngaged`, `LegalHoldReleased`, `EDiscoveryExportSealed` | `audit-chain`, retention-policy, compliance UI | event | — |
| `mail.workflow-triggers` | `MailWorkflowHandoffCreated` | `audit-chain`, `workflow-engine` | event | requires Cedar permit |
| `mail.esignature` | `EsignatureCreated`, `EsignatureCompleted`, `EsignatureRejected` | `audit-chain`, `workflow-engine` | event | PAdES-AdES bundle |
| `mail.delegation-sharing` | `MailboxDelegated`, `SharedMailboxCreated`, `SharedMailboxMemberGranted` | `audit-chain`, `identity` | event | — |
| `mail.dlp` | `DlpBlockApplied`, `DlpOverrideUsed` | `audit-chain` | event | — |
| `mail.dkim-spf-dmarc` | `MailDeliverabilityReputationChanged` | deliverability-dashboard, ops-mail page | event | — |
| `mail.dual-context-isolation` | `MailContextSwitched` | `audit-chain` | event | — |
| `mail.migration-importer` | `MailMigrationJobCompleted` | `audit-chain` | event | — |
| `tenancy` | `TenantOnboarded`, `TenantOffboarded` | mailbox-store + outbound-smtp + dkim-spf-dmarc + delegation-sharing | event in | — |
| `cloud-secrets` | `KmsKeyRotated` | mailbox-store + attachment-handler (DEK re-wrap) | event in | — |
| `identity` (Zitadel) | SCIM-provisioned `UserCreated`, `UserDeprovisioned` | mailbox-store + dkim-spf-dmarc | event in | — |
| `audit-chain` | `LegalHoldEngagedAcrossChannels` (cross-channel coord) | legal-hold | event in | — |
| `calendar` | iMIP REPLY events | mail (renders status) | indirect | event |
| `messenger` | `MessageEscalatedToMail` | mail (creates a mail thread) | event in | — |
| `workflow-engine` | `WorkflowHandoffCommitted`, `WorkflowAdvanced` | mailbox-store (mark handoff_committed), esignature | event in | — |
| `intelligence` | (provider-side) | smart-compose, smart-reply, summarisation, mail-to-workflow extraction | API call | per-tenant cell |
| `seaweedfs` (cloud-data-blob) | (provider-side) | attachment-handler | API | — |
| `postgres` | (provider-side) | mailbox-store, retention-policy, legal-hold | API | RLS per-tenant |
| `tantivy` | (provider-side) | search-index | API | per-tenant partition |
| `rspamd` | (provider-side) | anti-spam | API | — |
| `clamav` | (provider-side) | attachment-handler | API | — |
| `cloud-network-dns` | `DnsRecordPublished` | dkim-spf-dmarc | event in | — |
| `cloud-ca` | (provider-side) | esignature, signing-encryption (S/MIME) | API | — |
| `policy-engine` (Cedar) | (provider-side) | every BC | API | — |
| `ontology` | (provider-side; read/write) | every BC | API | per ADR-0245 cross-product flow |

---

## 14. Compliance (per ADR-0251 Compliance Pack)

Per-pack retention floors (minimum; tenant may exceed):

| Pack | Personal mail | Work mail | Audit trail | Notes |
|---|---|---|---|---|
| pack-default | User-controlled | Tenant-configurable (default 3y) | 1y minimum | — |
| pack-kr | User-controlled (PIPA Art. 28) | 5y when KR-FSS regulated tenant | 5y | KR 전자문서법 Art. 5; 전자금융감독규정 |
| pack-eu | User-controlled (GDPR Art. 5(1)(e)) | Tenant-configurable | 6y if subject to NIS2 incident records | GDPR + eIDAS + NIS2 |
| pack-us | User-controlled (CCPA) | Tenant-configurable | 1y minimum | — |
| pack-us-healthcare | User-controlled | 6y minimum (HIPAA §164.530(j)) | 6y | BAA required |
| pack-jp | User-controlled (APPI) | Tenant-configurable | per APPI Art. 22 | — |
| pack-sg | User-controlled (PDPA) | Tenant-configurable | per PDPA §11 | — |
| pack-au | User-controlled (Privacy Act APP) | Tenant-configurable | per APRA-CPS-234 | — |
| pack-in | User-controlled (DPDPA) | Tenant-configurable | per DPDPA Sch. | — |
| pack-br | User-controlled (LGPD) | Tenant-configurable | per LGPD Art. 37 | — |
| pack-ae | User-controlled (UAE PDPL) | Tenant-configurable | per UAE PDPL Art. 17 | — |
| pack-ksa | User-controlled (KSA PDPL) | Tenant-configurable | per KSA PDPL Art. 5 | — |

Per-pack DSAR / right-to-erasure handling:

- **Personal** mail: user-controlled in Settings → Privacy → Erase account; cascades through `oya-dsr-cascade-runner`.
- **Work** mail: tenant-administered in Admin → Privacy → DSAR; only Professional-context mail returned (Personal-pillar always excluded per ADR-0215).
- Legal hold overrides retention deletion (hold-before-purge invariant); on hold release, retention re-evaluates.

Framework coverage (cross-mapped with `compliance.md`):

- **SOC 2 Type 2** CC6.x + CC7.x + CC8.x: all enforced via `mail` µservice's audit-chain + Cedar policy enforcement + RLS + four-eyes + 2-person rule.
- **ISO 27001:2022** A.5.x / A.8.x: per `threat-model.md`.
- **GDPR** Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44–50.
- **HIPAA** 45 CFR §164.308, §164.310, §164.312, §164.314, §164.316, §164.502, §164.504(e).
- **CCPA / CPRA** rights to know, delete, opt-out.
- **KR-PIPA** Arts. 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2 + KR-ISMS-P §2.x.
- **KR 전자문서법** Art. 5 (electronic-document integrity via Ed25519 audit-chain).
- **KR-FSS 전자금융감독규정** when financial-services tenants are present.

---

## 15. Open Questions

| # | Question | Owner | Target ADR |
|---|---|---|---|
| 1 | JMAP server choice: in-house Rust vs Stalwart embed vs hybrid | axis-mail + ops-mail | ADR-####-jmap-server-choice |
| 2 | Per-tenant SMTP IP pool sizing + warmup protocol | ops-deliverability | ADR-####-smtp-ip-pool |
| 3 | Search index: Tantivy native vs Quickwit vs Elasticsearch | axis-mail | ADR-####-search-backend |
| 4 | Personal-mail E2E key recovery (user-only vs escrow-2-person) | council-privacy + axis-mail | adr-follow-ups.yaml#personal-mail-key-recovery |
| 5 | Cross-channel hold coordination (audit-chain owns) | resolved 2026-05-17 | — |
| 6 | Mail-to-Workflow extraction prompt safety (per-tenant default) | council-privacy + axis-workflow | adr-follow-ups.yaml#mail-workflow-extraction-default |
| 7 | Smart-reply chips: precompute on receive (cost) vs on-demand (latency) | axis-mail + axis-intelligence | ADR-####-smart-reply-precompute |
| 8 | E-signature: built-in PAdES only, or DocuSign/Adobe Sign bridges too | axis-mail + ops-legal | ADR-####-esign-scope |
| 9 | MLS-mail E2E semantics for store-and-forward (key rotation cadence) | council-privacy | ADR-####-mls-mail |
| 10 | ActiveSync support for M05 (Exchange parity for Outlook-native users) | axis-mail | ADR-####-activesync |

---

## 16. References

### Standards

- **SMTP**: RFC 5321 (2008) — Simple Mail Transfer Protocol. <https://datatracker.ietf.org/doc/html/rfc5321>
- **SMTP Submission**: RFC 6409 (2011) — Message Submission for Mail.
- **TLS for mail**: RFC 8314 (2018) — Cleartext Considered Obsolete.
- **STARTTLS**: RFC 3207 (2002).
- **MTA-STS**: RFC 8461 (2018). **TLS-RPT**: RFC 8460 (2018).
- **DKIM**: RFC 6376 (2011) + RFC 8463 Ed25519 (2018).
- **SPF**: RFC 7208 (2014). **DMARC**: RFC 7489 (2015). **ARC**: RFC 8617 (2019).
- **IMAP4rev2**: RFC 9051 (2021). IMAP4rev1: RFC 3501 (2003).
- **POP3**: RFC 1939 (1996).
- **JMAP**: RFC 8620 Core (2019) + RFC 8621 Mail (2019) + RFC 8639 Push (2019) + RFC 8887 WebSocket (2020).
- **Sieve**: RFC 5228 (2008). **ManageSieve**: RFC 5804 (2010).
- **Internet Message Format**: RFC 5322 (2008). **MIME**: RFC 2045–2049 (1996).
- **iCalendar**: RFC 5545 (2009). **iMIP**: RFC 6047 (2010).
- **S/MIME 4.0**: RFC 8551 (2019).
- **OpenPGP**: RFC 4880 (2007) + **OpenPGP crypto-refresh**: RFC 9580 (2024).
- **Sub-addressing**: RFC 5233 (2008).
- **Vacation responder**: RFC 3834 (2004) + Sieve extension RFC 5230 (2008).
- **One-click unsubscribe**: RFC 8058 (2017) + List-Unsubscribe RFC 2369 (1998).
- **SCIM 2.0**: RFC 7644 (2015). **SAML 2.0**: SAML-Core (2005). **OIDC 1.0**.
- **eIDAS** 910/2014; **PAdES** ETSI EN 319 142.
- **MLS**: RFC 9420 (2023) — Messaging Layer Security.
- **HIPAA** 45 CFR Part 164.

### Product / engineering sources (2024–2026)

- **Stalwart Mail Server** docs (2024–2026). <https://stalw.art/docs/>
- **Cyrus IMAP** docs. <https://cyrusimap.org/imap/>
- **Postfix** docs. <https://postfix.org/documentation.html>
- **Dovecot** docs. <https://doc.dovecot.org/>
- **Rspamd** docs (2024–2026). <https://rspamd.com/doc/>
- **ClamAV** docs. <https://docs.clamav.net/>
- **Tantivy** (Rust full-text) — `github.com/quickwit-oss/tantivy`.
- **Quickwit** — `quickwit.io`.
- **CloudNativePG** — `cloudnative-pg.io`.
- **Citus** — `docs.citusdata.com`.
- **SeaweedFS** — `github.com/seaweedfs/seaweedfs`.
- **Zitadel** docs. <https://zitadel.com/docs>
- **OpenBao** docs. <https://openbao.org/docs/>
- **Fastmail JMAP** docs (2024). <https://www.fastmail.com/dev/>
- **Gmail design history** — Google Workspace developer blog 2024–2026.
- **Outlook + Microsoft Graph mail API** — Microsoft Learn 2024–2026.
- **Hey product docs** — `hey.com`.
- **Superhuman engineering** blog — `blog.superhuman.com` (2024–2026; latency design + keyboard-first).
- **Proton Mail Tokenised Search** — `proton.me/blog/encrypted-search` (2024).
- **ProtonMail Bridge** — `proton.me/mail/bridge`.
- **Apple Hide My Email** docs (2024). <https://support.apple.com/hide-my-email>
- **Apple Mail / iCloud Mail** developer references.
- **DocuSign / Adobe Sign API** docs (parity benchmark only).
- **MSP / DocuSign comparison** — analyst reports 2024–2026.
- **SimpleLogin** alias service docs (parity for burner).

### Academic / threat references

- Curtmola et al. (CCS 2006); Cash et al. (CRYPTO 2013) — searchable encryption Cipher-Match.
- LINDDUN privacy methodology — Wuyts et al., KU Leuven.
- M3AAWG Sender Best Common Practices v3 (2024).

### Internal ADRs

- ADR-0008, ADR-0028 (Bominal data-use boundary + audit-chain).
- ADR-0056 (BNF v4.1). ADR-0105 (13-layer enum). ADR-0106 (usecase rename).
- ADR-0117 (data residency). ADR-0123 (HG maturity).
- ADR-0131 (per-microservice flat layout). ADR-0132 (no-suite forward policy). ADR-0133 (cross-tenant mail-server pattern).
- ADR-0135 (Connect dissolution; dual-context invariant). ADR-0139 (agentic SLO-gated promotion). ADR-0140 (Cedar policy enforcement). ADR-0145 (Cedar refinement).
- ADR-0208, ADR-0210, ADR-0215 (Bominal inheritance for dual-context + KR group mail + retention).
- ADR-0238 (Connect dissolution parallel session).
- ADR-0241 (audit-chain canonical). ADR-0242 (oyatie-is-a-tenant doctrine). ADR-0245 (substrate-vs-product layering). ADR-0251 (compliance pack). ADR-0255 (provider-BYOK + intelligence Cedar gate).
- ADR-MAIL-0003 (SDK launch order).

---

## Appendix A — Keyboard shortcut reference (Personal Mail default)

| Key | Action |
|---|---|
| C | Compose |
| R | Reply |
| A | Reply all |
| F | Forward |
| E | Archive |
| Y | Archive + next |
| ! | Mark spam |
| !P | Report phishing/harassment |
| Shift+I | Mark read/unread |
| H | Snooze |
| S | Star/un-star |
| L | Move to label |
| X | Multi-select toggle |
| / | Search |
| J / K | Next / previous |
| G I / G T / G D / G # | Go to Inbox / Sent / Drafts / Trash |
| Enter / Esc | Open / close |
| U | Undo |
| Cmd+Enter | Send |
| Cmd+Shift+Enter | Schedule send |
| Cmd+Shift+M | Mute thread |
| Cmd+Shift+S | Add to "Set aside" (Triage mode) |
| Cmd+, | Settings |
| Cmd+K | Command palette |
| ? | Show shortcuts |

All shortcuts user-rebindable. Defaults match Superhuman where overlapping; Gmail-equivalents available via alternate scheme.

### Story 25 — Zane (Personal) configures auto-forwarding to a partner address

**Precondition.** Zane wants all `@oyatie.app` mail forwarded to a secondary address `zane.backup@proton.me`.

**Steps.**
1. Zane opens Settings → Forwarding → Add destination.
2. Enters `zane.backup@proton.me`; receives verification mail at that address; clicks confirmation link (TTL 1h).
3. Zane sets conditional forward: only mail labeled "important". A Sieve rule is autogenerated.
4. Zane toggles "Keep a copy in oyatie inbox" ON.
5. New incoming "important" mail mirrors to Proton.
6. Audit emits `MailForwardingConfigured{destination, scope=label:important}`.

**Edge cases.**
- Forwarding domain on a known-spam list: forwarding refused; Zane shown an explanatory message.
- Recipient bounces 3 times in a row: forwarding auto-suspended with a warning mail to Zane.

### Story 26 — Aisha (Work) is invited to an external tenant's shared mailbox

**Precondition.** Aisha (acme.com) is a contractor for `foo-corp.com`; Foo-Corp wants her on `support@foo-corp.com`.

**Steps.**
1. Foo-Corp admin invites Aisha by `aisha@acme.com`.
2. Aisha receives a cross-tenant-collaboration invite; on accept she gets a second work-mode account in her client.
3. Foo-Corp Cedar policy gates her capabilities: `read` + `reply-as` but not `delete`, `forward-external`, or `admin`.
4. Audit-emits in BOTH tenants: `CrossTenantSharedMailboxAccepted{by=aisha@acme, from=foo-corp}`.

**Edge cases.**
- Foo-Corp's tenant policy forbids external members: invite refused; admin notified.
- Aisha later leaves Acme: Foo-Corp invite remains valid only if her oyatie identity persists (Zitadel-attached); otherwise auto-revoked.

### Story 27 — Bella (Personal) gets a calendar invite from a stranger

**Precondition.** Bella receives an iMIP invite from an unknown sender.

**Steps.**
1. Anti-phishing classifier checks the sender domain reputation; clean.
2. Mail rendered with iCalendar inline; calendar status: "From an unknown sender".
3. Bella has the option to Accept / Decline / Block sender / Report.
4. If Bella accepts, calendar adds with a "needs-review" flag; she can downgrade to Tentative.
5. Audit emits `CalendarInviteFromUnknownSenderRendered`.

### Story 28 — Cara (Work) receives PHI-tagged mail in HIPAA pack

**Precondition.** Cara (medical practice tenant on pack-us-healthcare) receives a mail with patient PHI.

**Steps.**
1. Inbound classifier detects PHI patterns (chart number, ICD-10 codes, SSN).
2. Tag the message with `data_class=PHI`; route to PHI-secured folder; encrypt under tenant DEK (already enforced); audit emits with `PHI` data class.
3. Cara views with PHI watermark visible in body header.
4. Forward action goes through PHI-aware DLP: external forward refused; internal forward requires reason; audit-chained per HITECH §13402.

### Story 29 — Drew (Work) uses LLM to draft a thoughtful reply

**Precondition.** Drew received a long client mail; wants help writing a careful reply.

**Steps.**
1. Drew presses Cmd+J → "Draft reply with LLM".
2. Intelligence substrate (tenant-cell) reads Drew's history with this client (last 30 days, opt-in scope) + the current mail + Drew's signature style profile.
3. Returns 3 candidate drafts in 1s p99 (200ms p50).
4. Drew picks one, edits, sends. Audit emits `MailLlmDraftCreated{candidate_count, picked_index}`.

**Edge cases.**
- LLM grounding finds a factual claim that contradicts a prior Drew mail: warning surfaced; LLM annotates the contradiction.
- LLM cannot complete due to tenant cell capacity: graceful degradation to client-side simple template.

### Story 30 — Eli (Personal) uses encrypted PGP with a contact

**Precondition.** Eli's friend uses PGP; Eli wants to send encrypted.

**Steps.**
1. Eli adds friend's PGP key (via WKD auto-discovery on send-to).
2. Compose to `friend@privacy.org`; toolbar shows "PGP encrypt available" badge.
3. Eli toggles encrypt. Body input remains plain (Eli's draft is encrypted at-rest under Eli's DEK; only ciphertext is sent over the wire).
4. Send. Outbound encrypts under recipient PGP public key + Eli's signature.
5. Recipient decrypts using their PGP private key.
6. Audit emits `MessageSentEncrypted{cipher=openpgp_rfc9580, recipient_keyid}`.

### Story 31 — Faye (Work) uses S/MIME for a legal contract

**Precondition.** Faye (acme.com) has S/MIME certificate from cloud-ca; sends a signed contract to external counsel.

**Steps.**
1. Compose; attach PDF. Toggle "S/MIME sign".
2. Outbound signs body + attachment with Faye's S/MIME certificate (RFC 8551).
3. External counsel verifies signature in their Outlook client; chain validates to oyatie-ca (cross-signed by public CA).
4. Audit emits `MessageSentSigned{cipher=smime_rfc8551}`.

### Story 32 — Greg (Personal) opens mail on a brand-new laptop offline

**Precondition.** Greg's new laptop just booted; no internet yet.

**Steps.**
1. Greg opens oyatie mail desktop app; cached login allows offline launch.
2. App shows last-synced state (24h ago, before he left office).
3. Greg drafts a reply; saved to local SQLite encrypted with OS keychain.
4. Internet restored 2h later; outbound queue flushes; draft → sent.

### Story 33 — Helen (Work) sets up DMARC policy for a new domain

**Precondition.** Helen onboards `helen-llc.com` to her tenant.

**Steps.**
1. Admin Console → Domains → Add → `helen-llc.com`.
2. System auto-suggests DNS: MX, SPF (`v=spf1 include:_spf.oyatie.app -all`), DKIM (`oya._domainkey.helen-llc.com`), DMARC (`v=DMARC1; p=quarantine; rua=mailto:dmarc-rua-tenant@oyatie.dev`), MTA-STS (`https://mta-sts.helen-llc.com/.well-known/mta-sts.txt`).
3. Helen pastes records at her registrar; clicks Verify.
4. DNS lookups succeed; status → "Active".
5. Initial DMARC policy is `p=none` (monitor) for 14 days; after grace, system suggests `p=quarantine` then `p=reject`.

### Story 34 — Ian (Work) reviews quarantined external mail

**Precondition.** Acme's quarantine accumulated 200 inbound messages over the week.

**Steps.**
1. Ian opens Admin → Quarantine → Inbound.
2. Filters: classifier="suspicious-phishing", recipient="anyone-in-engineering".
3. Reviews each; releases 3 legitimate (admin override) → recipients receive in inbox; rest stays quarantined.
4. Audit emits `QuarantineReleased{by=ian, message_ids=[...]}`.

### Story 35 — Jules (Personal) builds a Sieve filter via UI

**Precondition.** Jules wants newsletters auto-tagged.

**Steps.**
1. Settings → Rules → New.
2. Visual builder: if sender ends with `@*.newsletter.com` OR header `List-Id` present → label `Newsletter` + move to `Feed` folder.
3. Save. System compiles to a Sieve script (visible "View Sieve source"); also visible via ManageSieve.
4. Future inbound matches → tagged + filed.

---

## Appendix B — JMAP capability set (RFC 8620 / 8621)

Mandatory:

- `urn:ietf:params:jmap:core` — RFC 8620.
- `urn:ietf:params:jmap:mail` — RFC 8621.

Supported:

- `urn:ietf:params:jmap:submission` — RFC 8621 §7 EmailSubmission.
- `urn:ietf:params:jmap:vacationresponse` — RFC 8621 §8.
- `urn:ietf:params:jmap:mdn` — RFC 9007 (Message Disposition Notification).
- `urn:ietf:params:jmap:websocket` — RFC 8887.
- `urn:ietf:params:jmap:push` — RFC 8620 §7 + RFC 8639.
- `urn:ietf:params:jmap:quota` — RFC 9425.
- `urn:ietf:params:jmap:sieve` — draft-ietf-jmap-sieve.
- `urn:ietf:params:jmap:calendars` — draft-ietf-jmap-calendars (when `calendar` µservice deployed in same tenant cell).
- `urn:ietf:params:jmap:contacts` — draft-ietf-jmap-contacts (when `contacts` deployed).

Oyatie extensions (vendor-prefix `urn:oyatie:jmap:*`):

- `urn:oyatie:jmap:dual-context` — context-switch + context-scope query.
- `urn:oyatie:jmap:workflow-trigger` — explicit mail-to-workflow handoff.
- `urn:oyatie:jmap:esignature` — PAdES e-signature flow.
- `urn:oyatie:jmap:legal-hold` — hold + eDiscovery (compliance-officer scope only).
- `urn:oyatie:jmap:smart-reply` — pre-computed reply chips.
- `urn:oyatie:jmap:burner-aliases` — Hide-My-Email-equivalent alias lifecycle.

---

## Appendix C — Sieve extensions supported

- RFC 5228 base.
- RFC 5229 variables.
- RFC 5230 vacation.
- RFC 5232 imap4flags.
- RFC 5233 sub-addressing.
- RFC 5235 spamtest + virustest.
- RFC 5260 date.
- RFC 5293 editheader.
- RFC 5429 reject + ereject.
- RFC 5435 notify.
- RFC 5463 ihave.
- RFC 5703 mime + foreverypart + extracttext.
- RFC 6131 IMAP events for Sieve.
- RFC 6558 fileinto-mailbox-extensions.
- RFC 6609 include.
- RFC 9042 detail of "envelope".
- Oyatie extensions: `oyatie.workflow-trigger`, `oyatie.label`, `oyatie.cedar-check`.

---

(End of PRD)

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `mail` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `mail` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_message` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
