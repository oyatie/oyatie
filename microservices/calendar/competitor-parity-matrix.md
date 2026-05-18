---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: calendar
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-calendar + council-architecture
deciders: axis-calendar, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-CAL-0001, ADR-CAL-0002, ADR-CAL-0003]
related_artifacts:
  - microservices/calendar/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-CALENDAR gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (calendar µservice)

## Purpose

Quantitative + qualitative parity comparison against industry-leading
calendar + scheduling products. Drives `oya-governance-hyperscaler-
maturity-claims` gate per HG-CALENDAR (ADR-0123) and constrains what
gtm-customer-success can claim in tenant sales conversations. Re-
validated bi-annually because the calendar / scheduling landscape moves
quickly (Google Calendar's AI scheduling, Microsoft Copilot in
Calendar, Cal.com OSS rise, Notion Calendar acquisition).

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Google Calendar | Workspace Calendar | enterprise-grade; AI scheduling; multi-org cross-share; mobile-first | `developers.google.com/calendar` |
| Microsoft Outlook Calendar | M365 Calendar | M365 integration; Exchange ActiveSync; Copilot in Calendar | `learn.microsoft.com/graph/api/resources/event` |
| Apple Calendar / iCloud | iCloud Calendar | native macOS + iOS integration; CalDAV server; family sharing | `developer.apple.com/documentation/eventkit` |
| Fastmail Calendars | Fastmail bundle | JMAP Calendars (only production impl); strong CalDAV | `jmap.io` + `fastmail.com/calendar` |
| Proton Calendar | Proton Mail bundle | E2E-encrypted; privacy-first; CalDAV | `proton.me/calendar` |
| Calendly | scheduling links + bookings | external scheduling; round-robin; availability windows; integrations | `developer.calendly.com` |
| Cal.com | OSS Calendly | self-hosted; webhook; teams; routing forms | `cal.com/docs/api-reference` |
| SavvyCal | scheduling links | superior UX for ranked-time scheduling | (consumer; no public API) |
| Doodle | poll-based scheduling | meeting-time polls; lightweight UX | `developer.doodle.com` |
| Notion Calendar (Cron) | calendar app | unified Google + Outlook + iCloud; menu bar | (proprietary; no public API at this layer) |
| Naver Works Calendar | KR enterprise | KR-locale; KR holidays; group calendars | `developers.worksmobile.com/kr` |
| Fantastical | Mac/iOS calendar app | natural-language event input; weather; conferencing | (consumer app; no public API) |

## Feature Parity Matrix

### Core event management

| Capability | oyatie | Google | Outlook | Apple | Fastmail | Proton | Cal.com | Calendly |
|---|---|---|---|---|---|---|---|---|
| Events (single + recurring) | ✅ RFC 5545 strict | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| RRULE with all BY* + BYSETPOS | ✅ rrule-rs 100% libical | ✅ | ✅ | ✅ | ✅ | ✅ | partial | partial |
| Attendees + RSVP (iTIP + iMIP) | ✅ RFC 5546 + 6047 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Counter-proposal (RFC 5546 §3.2.7) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | partial | ❌ |
| Free/busy queries | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Cross-org free/busy w/ policy gating | ✅ Cedar-gated (differentiator) | partial (calendar permissions) | partial (Exchange Tenant) | ❌ | partial | ❌ | partial | partial |
| Room / resource booking | ✅ | ✅ | ✅ | partial | ✅ | ❌ | ✅ | ❌ |
| 100% double-booking refusal at write | ✅ AC-09 SLO | ✅ | ✅ | partial | ✅ | n/a | partial | n/a |
| Legal hold on events | ✅ | ✅ (Vault) | ✅ (eDiscovery) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Auto-scheduling (RFC 6638) | ✅ | partial | ✅ | ✅ | ✅ | partial | partial | n/a |
| Meeting-time polls | ✅ (M03-onward1) | ✅ (Find a Time) | ✅ (FindTime) | ❌ | partial | ❌ | ❌ | partial |

### Protocols + interop

| Protocol | oyatie | Google | Outlook | Apple | Fastmail | Proton | Cal.com | Calendly |
|---|---|---|---|---|---|---|---|---|
| CalDAV (RFC 4791) read+write | ✅ M03 (Radicale + SabreDAV) | ✅ | partial | ✅ | ✅ | ✅ | ❌ | ❌ |
| CalDAV scheduling (RFC 6638) | ✅ M03 | ✅ | partial | ✅ | ✅ | partial | ❌ | n/a |
| VAVAILABILITY (RFC 7953) | ✅ M03 | ✅ | partial | ✅ | ✅ | partial | ❌ | partial |
| iCalendar .ics import | ✅ 10k events ≤30s | ✅ | ✅ | ✅ | ✅ | ✅ | partial | ❌ |
| iCalendar .ics export | ✅ 10k events ≤30s | ✅ | ✅ | ✅ | ✅ | ✅ | partial | partial |
| iTIP (RFC 5546) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | partial | partial |
| iMIP (RFC 6047) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | partial | partial |
| JSCalendar (RFC 8984) | ✅ M04 | partial | partial | partial | ✅ | partial | ❌ | ❌ |
| JMAP Calendars (draft) | ✅ M04 | ❌ | ❌ | ❌ (WWDC '24 hint) | ✅ | ❌ | ❌ | ❌ |
| Exchange ActiveSync | ❌ M05-onward | partial | ✅ | partial | partial | ❌ | ❌ | ❌ |
| Google Calendar API compat shim | ✅ M04 (read-only) | n/a | ❌ | ❌ | ❌ | ❌ | partial | ❌ |
| Outlook Graph API compat shim | ✅ M04 (read-only) | ❌ | n/a | ❌ | ❌ | ❌ | partial | ❌ |

### Calendar systems + tz

| Capability | oyatie | Google | Outlook | Apple | Fastmail | Proton | Cal.com | Calendly |
|---|---|---|---|---|---|---|---|---|
| IANA tzdb with 30d refresh window | ✅ ADR-CAL-0004 | ✅ | ✅ | ✅ | ✅ | partial | partial | partial |
| Per-tenant tzdb pin (audit reproducibility) | ✅ M03 (differentiator) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Hijri calendar overlay (ICU4X) | ✅ M03 (pack-ae + pack-ksa) | partial | ✅ | partial | ❌ | ❌ | ❌ | ❌ |
| Japanese imperial calendar | ✅ M03 (pack-jp) | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Buddhist calendar | ✅ M03 | ✅ | partial | ✅ | ❌ | ❌ | ❌ | ❌ |
| Korean public holidays | ✅ M03 (pack-kr) | partial | partial | partial | ❌ | ❌ | ❌ | ❌ |
| DST spring-forward / fall-back strict | ✅ ADR-CAL-0002 named cases | ✅ | ✅ | ✅ | ✅ | partial | partial | partial |
| Floating time semantics (RFC 5545 §3.3.5) | ✅ M03 (querier-tz resolves) | partial | partial | ✅ | ✅ | partial | ❌ | ❌ |

### Privacy + isolation

| Capability | oyatie | Google | Outlook | Apple | Fastmail | Proton | Cal.com | Calendly |
|---|---|---|---|---|---|---|---|---|
| Dual-context (Personal/Professional) structural isolation | ✅ (differentiator) | ❌ (acct switching only) | ❌ (acct switching) | ❌ | ❌ | partial | ❌ | ❌ |
| Cross-org sharing with policy-bounded disclosure (Cedar) | ✅ (differentiator) | partial (calendar permissions) | partial | ❌ | ❌ | ❌ | ❌ | ❌ |
| E2E encryption at rest (Tenant-DEK) | ✅ professional context | ❌ | ❌ | ❌ | ❌ | ✅ (E2E whole-app) | ❌ | ❌ |
| Audit-chain for every event lifecycle | ✅ Ed25519 + Merkle | partial (Vault) | partial (UAL) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Per-jurisdiction retention (11 packs) | ✅ M03 | partial | partial | ❌ | ❌ | ❌ | ❌ | ❌ |

### AI + assist (autonomy tiers)

| Capability | oyatie | Google | Outlook | Apple | Fastmail | Proton | Cal.com | Calendly |
|---|---|---|---|---|---|---|---|---|
| T0 smart-time suggestion | ✅ M03 | ✅ Find a Time | ✅ FindTime | ❌ | ❌ | ❌ | partial | partial |
| T0 title / agenda suggestion | ✅ M03 | ✅ Duet AI | ✅ Copilot | ❌ | ❌ | ❌ | ❌ | ❌ |
| T1 smart-scheduling on behalf | ✅ M03-onward1 | partial (Duet) | partial (Copilot) | ❌ | ❌ | ❌ | ❌ | ❌ |
| T1 auto-decline conflicts | ✅ M03-onward1 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| T2 auto-block focus | ✅ M04 | partial (Focus Time) | partial (Focus) | ❌ | ❌ | ❌ | ❌ | ❌ |
| EU AI Act Annex III §3 conformity (HR overlay) | ✅ (refused at Cedar layer until ADR-CAL-XXXX) | unclear | unclear | n/a | n/a | n/a | unclear | unclear |

## Key differentiators (ordered)

1. **Cross-tenant availability with policy-bounded disclosure** — Cedar-gated. No competitor enforces cross-org free/busy with structured policy + audit-chain.
2. **Dual-context (Personal / Professional) structural isolation enforced in code** — competitor solutions are policy-only or account-switching.
3. **Per-tenant tzdb pin for audit reproducibility** — unique. Healthcare/regulated tenants get appointment-history reproducibility.
4. **Strict RFC 5545 RRULE conformance (rrule-rs + libical corpus 100%)** — most competitors are 90-95% conformant; 5-10% edge cases differ.
5. **11-pack regulatory overlay** — Hijri / imperial / Buddhist / locale-specific holidays at per-pack overlay granularity.
6. **Legal hold + retention on events** — Google Vault + Outlook eDiscovery cover this, but only at suite granularity, not per-event.
7. **Audit-chain (Ed25519 + Merkle) on every event lifecycle** — beyond what enterprise competitors offer.

## Gap closing plan (M03 → M05)

| Gap | Current state | Plan | Target |
|---|---|---|---|
| JMAP Calendars (draft) | M04 once IETF stabilises | adapter-jmap crate; OpenSLO; SDK Swift integration | M04 |
| Exchange ActiveSync | Not planned at M03 | review at M05 based on enterprise demand | M05-onward |
| Google Calendar API compat shim (read-only) | M04 | adapter behind feature flag | M04 |
| Outlook Graph API compat shim (read-only) | M04 | adapter behind feature flag | M04 |
| Meeting-time polls | M03-onward1 | extension of invitation-flow worker | M03-onward1 |
| EU AI Act conformity assessment (HR overlay) | refused at Cedar layer | dedicated ADR-CAL-XXXX | M04-onward |
| Mobile-app native CalDAV optimisation | depends on Apple/Android CalDAV stack | tune Radicale for mobile poll patterns | M04 |

## Verification

- HG-CALENDAR gate validates this matrix is consistent with the
  PRD `§Competitive Benchmark` row.
- gtm-customer-success references this matrix in sales materials;
  any claim of parity / superiority that diverges from this matrix
  is a process violation.
- Bi-annual review re-validates each row against current competitor
  release notes; new competitor entrants get added.

## References

- ADR-0123 — Hyperscaler maturity claim gate.
- ADR-0135; ADR-0131; ADR-0132; ADR-0133.
- ADR-CAL-0001 — CalDAV backend selection.
- ADR-CAL-0002 — RRULE engine conformance.
- ADR-CAL-0003 — CalDAV-first frontend; JMAP at M04.
- `microservices/calendar/PRD.md` §Competitive Benchmark.
- Google Calendar API — `developers.google.com/calendar`.
- Microsoft Graph (Outlook) — `learn.microsoft.com/graph/api/resources/event`.
- Apple EventKit — `developer.apple.com/documentation/eventkit`.
- Fastmail JMAP Calendars — `jmap.io`.
- Proton Calendar — `proton.me/calendar`.
- Calendly API — `developer.calendly.com`.
- Cal.com — `cal.com/docs/api-reference`.
- Doodle Developer — `developer.doodle.com`.
- Naver Works KR — `developers.worksmobile.com/kr`.
- `microservices/mail/competitor-parity-matrix.md` — sibling reference.
- `microservices/messenger/competitor-parity-matrix.md` — sibling reference.
