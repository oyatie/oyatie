---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-mail + gtm-customer-success + council-architecture
deciders: axis-mail, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/mail/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-MAIL)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (mail µservice)

## Purpose

Quantitative + qualitative parity comparison vs industry-leading mail products. Drives `oya-governance-hyperscaler-maturity-claims` per ADR-0123 (HG-MAIL gate). Tells gtm what to claim and what NOT to claim.

## Competitor set

| Competitor | Surface | Primary differentiator | Source |
|---|---|---|---|
| Google Workspace Gmail | Enterprise mail + Vault | scale + Vault legal-hold | `support.google.com/vault` |
| Microsoft Exchange Online + Purview | Enterprise mail + retention + eDiscovery | mature Purview suite | `learn.microsoft.com/exchange` |
| Apple iCloud Mail | Consumer mail | iOS / macOS integration | `developer.apple.com/icloud` |
| Proton Mail | Privacy-first E2E | E2E-encrypted personal pillar | `proton.me/support/mail` |
| Tutanota | Privacy-first E2E | end-to-end + EU-based | `tuta.com` |
| Fastmail | JMAP-native + standards-first | JMAP authoring + IMAP excellence | `fastmail.com/help/jmap` |
| Hey (Basecamp) | Productivity-led inbox | screener + organize-by-default UX | `hey.com` |
| Zoho Mail | Enterprise + admin maturity | per-user retention + SMTP relay | `zoho.com/mail/admin` |
| Naver Works Mail | KR-FSS regulated | KR PIPA + 5y retention | `naver.worksmobile.com` |
| Daou Cyworks Mail / Hancom Office Mail | KR-regulated mail | KR-FSS-recognised | vendor docs |
| Stalwart Mail Server (OSS) | Self-hosted unified | modern Rust SMTP+IMAP+JMAP | `stalw.art` |
| Postfix + Dovecot (DIY) | Self-hosted classic | standards reference | `postfix.org` + `dovecot.org` |

## Feature parity matrix

### Mail protocol surface

| Capability | oyatie | Gmail | Exchange | Proton | Fastmail | Tutanota | Naver | Stalwart | Postfix+Dovecot |
|---|---|---|---|---|---|---|---|---|---|
| SMTP receive (RFC 5321 + STARTTLS RFC 8314) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| SMTP submission (RFC 6409) | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (proprietary) | ✅ | ✅ | ✅ |
| IMAP4rev2 (RFC 9051) | ✅ | partial (IMAP4rev1) | ✅ | partial (bridge) | ✅ | ❌ | ✅ | ✅ | ✅ |
| JMAP-Core + Mail (RFC 8620/8621) | ✅ | ❌ | ❌ | ❌ | ✅ (canonical) | ❌ | ❌ | ✅ | ❌ |
| DKIM + Ed25519 (RFC 8463) | ✅ | RSA-only | RSA-only | ✅ | ✅ | ✅ | RSA-only | ✅ | RSA-default |
| SPF + DMARC + ARC | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | manual |
| MTA-STS + TLS-RPT | ✅ | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ | manual |
| S/MIME (RFC 8551) | ✅ | ✅ (Workspace) | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | manual |
| OpenPGP (RFC 9580 / 4880) | ✅ | partial (CSE) | partial | ✅ (canonical) | ✅ | ✅ | ❌ | partial | manual |

### Privacy + isolation

| Capability | oyatie | Gmail | Exchange | Proton | Fastmail | Tutanota | Naver |
|---|---|---|---|---|---|---|---|
| Dual-context isolation at kernel (Personal/Professional) | ✅ unique | ❌ | ❌ | partial | ❌ | ❌ | ❌ |
| Org admin cannot decrypt user's personal mail | ✅ unique | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Per-tenant DEK + per-user DEK escrow | ✅ | partial | partial | ✅ | partial | ✅ | partial |
| Encrypted-token search (no plaintext index) | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ |
| Cross-pillar refusal CI-enforced | ✅ unique | n/a | n/a | n/a | n/a | n/a | n/a |

### Compliance + legal-hold

| Capability | oyatie | Gmail Vault | Exchange Purview | Proton | Fastmail | Naver |
|---|---|---|---|---|---|---|
| Scoped legal hold | ✅ | ✅ | ✅ | ❌ | partial | ✅ |
| Four-eyes approval at engage + release | ✅ unique | ❌ | partial | ❌ | ❌ | ❌ |
| Personal-pillar hold FORBIDDEN at kernel | ✅ unique | n/a (no pillar concept) | n/a | n/a | n/a | n/a |
| eDiscovery export with re-derivable digest | ✅ unique | provider-asserted | provider-asserted | ❌ | ❌ | ❌ |
| EDRM XML 1.2 native | ✅ | ✅ | ✅ | ❌ | ❌ | partial |
| Per-pack retention floor (statutory) | ✅ (11 packs) | partial | ✅ | partial | partial | ✅ (KR only) |
| HIPAA BAA | conditional | ✅ | ✅ | ✅ | ✅ | partial |
| KR PIPA + 전자문서법 Art. 5 | ✅ | partial | partial | ❌ | ❌ | ✅ |
| EU GDPR DPA | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| EU AI Act conformity for classifiers | ✅ (per overlay) | partial (Gmail Smart Compose has classifier; no published conformity statement as of 2026-05-17) | partial (Outlook Copilot has classifier; conformity statement scheduled per Microsoft EU AI Act compliance roadmap 2025-Q4) | ❌ | ❌ | partial |
| SOC 2 Type 2 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| ISO 27001:2022 + 27018 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Workflow + ontology integration

| Capability | oyatie | Gmail | Exchange | Hey | Fastmail |
|---|---|---|---|---|---|
| Mail-to-Workflow handoff with explicit consent + audit | ✅ unique | partial (Apps Script triggers; no audit) | partial (Power Automate; no consent gate) | ❌ | ❌ |
| Per-event audit-chain Ed25519 seal | ✅ unique | n/a | partial (audit log; not Ed25519) | n/a | n/a |
| Ontology-write on every send/receive/hold | ✅ unique | ❌ | partial | ❌ | ❌ |

### Operations + deliverability

| Capability | oyatie | Gmail | Exchange | Proton | Fastmail | Naver |
|---|---|---|---|---|---|---|
| Per-tenant SMTP IP pool as FinOps surface | ✅ unique | opaque | shared pool | shared pool | partial | partial |
| Per-tenant deliverability dashboard | ✅ | partial | partial | ❌ | ❌ | partial |
| Per-tenant reputation auto-throttle | ✅ | n/a (Google manages) | n/a | n/a | n/a | n/a |
| DKIM key rotation runbook | ✅ (90d) | automated | automated | automated | automated | 90d |
| RBL delisting workflow | ✅ | provider-side | provider-side | provider-side | provider-side | per-tenant |

## Quantitative performance parity

| Metric | oyatie target | Gmail ref | Exchange ref | Notes |
|---|---|---|---|---|
| Inbox open p95 | ≤ 300ms | ≤ 500ms typical | ≤ 800ms typical | oyatie faster on observed M03 reference workload |
| Thread render p95 | ≤ 150ms | ≤ 400ms | ≤ 500ms | parity-advantage |
| Search p95 (1M-message corpus) | ≤ 500ms | ≤ 800ms | ≤ 1.5s | parity-advantage |
| IMAP fetch p99 (latest 50 headers) | ≤ 300ms | n/a (Gmail rate-limits IMAP) | ≤ 500ms | parity-advantage |
| Outbound submission p99 | ≤ 300ms | ≤ 500ms | ≤ 500ms | parity |
| Delivery p99 (queue → recipient MX) | ≤ 30s | ≤ 30s | ≤ 30s | parity |
| eDiscovery export 5GB | ≤ 24h | ≤ 24h | ≤ 24h | parity |
| Mailbox restore 5GB | ≤ 15min | hours typically | hours typically | parity-advantage |

## Key parity gaps to close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Mobile-native UX parity vs Apple Mail / Gmail-iOS at M03 | gtm-mobile + axis-mail | M04 |
| 2 | Calendar-bridge maturity (ICS handling depth) | axis-mail + axis-calendar | M04 |
| 3 | Multi-language SDK breadth (Go / JVM / .NET) | axis-mail | M04-M05 |
| 4 | Spam classifier accuracy vs Gmail (closed dataset benchmark) | axis-trust-safety | M04 |
| 5 | Per-tenant deliverability dashboard UI polish | gtm | M04 |
| 6 | OAuth-as-Sign-In flow with Apple ID / Google ID for personal-pillar | axis-mail + identity | M04 |

## Key oyatie differentiators (NOT in any competitor)

1. **Dual-context isolation at the kernel layer**: org admin structurally cannot read user's personal mail. Bombproof at code level (CI lane `dual-context-cross-boundary`).
2. **Four-eyes legal-hold + eDiscovery**: re-derivable seal from source blocks; not provider-asserted.
3. **Per-tenant SMTP IP pool with FinOps + ops surface**: tenants see + manage their own reputation.
4. **Multi-pack residency (11 packs)**: cross-pack replication forbidden; SCC exception path.
5. **Self-hosted with no vendor lock**: tenants can pin to Postfix or Stalwart backend.
6. **Workflow handoff with explicit consent + audit**: never silent mining.
7. **Encrypted-token search**: even oyatie operators cannot read tenant mail content.
8. **KR-FSS pack at launch**: most international mail vendors don't ship pack-kr as first-class.

## Claim-boundary rules

Sales claims permitted:
- ✅ "Dual-context (Personal/Professional) isolation at the kernel layer is unique to oyatie mail" (bi-annually re-verified)
- ✅ "Multi-pack residency exceeds Gmail's region offering" (Gmail has ~14 regions but pack-residency at oyatie is at GDPR/PIPA/HIPAA level; differentiated)
- ✅ "Encrypted-token search; we cannot read your mail" (true; verifiable)
- ✅ "Self-hosted; no vendor lock" (true; Postfix + Stalwart adapter choice)

Sales claims FORBIDDEN (per ADR-0123):
- ❌ "Faster than Gmail" (no public benchmark; would be unsourced superiority)
- ❌ "HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare; do not claim universal)
- ❌ "GDPR-compliant" (universal claim; in fact requires DPA + pack-eu)
- ❌ "Cheaper than Exchange" (depends on workload; per-tenant economics vary)
- ❌ "AI-Act compliant" (universal; in fact requires conformity assessment per high-risk classifier deployment)

## Bi-annual refresh

| Step | Owner |
|---|---|
| 1. Survey competitor docs + pricing | gtm |
| 2. Update matrix + cite sources | axis-mail |
| 3. Re-run quantitative benchmarks (staging mail cluster) | ops-sre-reliability |
| 4. Council-architecture review of claim boundaries | council-architecture |
| 5. Publish + notify sales | gtm |

## References

- `microservices/mail/PRD.md` §Competitive Benchmark
- `/specs/hyperscaler-gates.json` HG-MAIL
- ADR-0123 (hyperscaler-maturity-claim-gate)
- ADR-0135 (super-app dissolution)
- ADR-0132 (no-grouping forward policy)
- ADR-0133 (cross-tenant mail-server pattern)
- Competitor docs as cited inline
- M3AAWG Sender Best Common Practices v3
- RFC 5321, 6409, 9051, 8620, 8621, 6376, 7208, 7489, 8617
