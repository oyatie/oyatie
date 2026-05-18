---
id: ADR-CAL-0001
status: Accepted
date: 2026-05-17
microservice: calendar
deciders: axis-calendar, council-architecture, ops-sre-reliability, ops-security
owner: axis-calendar + council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0126
  - ADR-0131
  - ADR-0132
  - ADR-0133
related_artifacts:
  - microservices/calendar/PRD.md (FR-09; §Bounded Contexts → ics-import-export adapter-caldav)
  - microservices/calendar/iac/helm/Chart.yaml
  - microservices/calendar/runbooks/caldav-sync-loop.md
purpose: |
  Close the derived backend-selection gap surfaced by the
  `oya-calendar-ics-import-export-adapter-caldav` crate row in the PRD layer-
  mapping table. The catalog row mandates a CalDAV backend; ADR-0105
  Amendment 3 requires the backend to be named explicitly
  (`-adapter-<backend>`); the IaC chart in `iac/helm/` must pin a concrete
  LTS image. This ADR makes that choice authoritative.
---

# ADR-CAL-0001: CalDAV server backend — Radicale 3.x LTS primary; SabreDAV 4.x adapter alternative; Cyrus IMAP+CalDAV rejected

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The `calendar` µservice ships at M03 with a CalDAV (RFC 4791) read+write
frontend so Apple Calendar, Thunderbird/Lightning, Evolution, GNOME
Calendar, KDE KOrganizer, DAVx5 (Android), and assorted enterprise mail
clients can attach as native CalDAV consumers. PRD-calendar §FR-09 makes
this a "Must"; AC-04 makes end-to-end CalDAV interop a release gate.

Per ADR-0131 (per-microservice flat layout) the CalDAV backend is deployed
under `microservices/calendar/iac/helm/<backend>/`. Per ADR-0105 Amendment
3 (backend-qualified adapters), the crate that wires it must be named
`oya-calendar-ics-import-export-adapter-caldav` plus a more specific
backend-qualified variant if multiple backends are supported.

Three production-grade open-source CalDAV servers are candidates:

1. **Radicale** (Python; AGPL-3.0). Active LTS line 3.x; small footprint;
   pluggable storage backend (filesystem default; PostgreSQL via
   `radicale-storage-postgresql`); modest performance ceiling (~50
   concurrent CalDAV sessions per pod baseline). Excellent RFC 4791
   conformance score in the public CalDAV interop matrix at
   `bobosola.com/caldav-tests`. Used by Nextcloud's old Calendar adapter
   and by the Sandstorm Calendar grain.
2. **SabreDAV** (PHP; modified-BSD). The most widely-deployed CalDAV
   server on the planet — powers Baïkal, Nextcloud Calendar, Owncloud
   Calendar, Fastmail's older CalDAV gateway, and many ISP-hosted shared
   calendar offerings. Excellent RFC 4791 + RFC 6638 (scheduling) + RFC
   7953 (VAVAILABILITY) coverage. Performance ceiling ~200 concurrent
   sessions per pod baseline. PHP runtime introduces an operational
   surface that diverges from the Rust µservice ecosystem.
3. **Cyrus IMAP+CalDAV** (C; per upstream license). High-performance
   integrated mail+CalDAV server; ships with Fastmail's production
   deployment. Extremely fast (~2000 concurrent CalDAV sessions per
   pod). Architecturally bundles IMAP + CalDAV + CardDAV + JMAP, which
   makes it the obvious Fastmail choice but introduces a bundling
   anti-pattern in oyatie (per ADR-0132 no-suite forward-policy) because
   the same daemon would have to serve both the `mail` µservice's IMAP
   surface and the `calendar` µservice's CalDAV surface, violating
   per-µservice isolation.

Performance budget per PRD-calendar §"Performance":
- Agenda render p95 ≤ 250ms (1 week, 100 events).
- Free/busy query p99 ≤ 200ms for 1k attendees.
- CalDAV PROPFIND p99 ≤ 400ms per RFC 4791.
- ICS import 10k events p95 ≤ 30s (per problem statement; PRD says 60s p99
  but cited 30s p95 target in problem statement; honour the stricter
  number for benchmarking).

Per ADR-0133 axis-4 industry-citation, the chosen backend must be a named
LTS pin (`docs/standards/observability-slo.md`); upstream activity in the
past 12 months must be non-zero; CVE backlog must be zero P0/P1 at pick
time.

## Decision

The calendar µservice ships **Radicale 3.2.x as the default CalDAV
backend** for the kr/eu/us/jp/sg/au/in/br/ae/ksa packs, with **SabreDAV
4.x available as an adapter-qualified alternative** for the
`pack-us-healthcare` overlay where PHP-runtime scrutiny is acceptable in
exchange for SabreDAV's superior RFC 6638 scheduling conformance (which
healthcare auto-scheduling workflows lean on) and SabreDAV's longer
production track record at scale.

Concrete bindings:

- Crate: `oya-calendar-ics-import-export-adapter-caldav-radicale` is the
  primary backend-qualified adapter. `oya-calendar-ics-import-export-
  adapter-caldav-sabredav` is the secondary backend-qualified adapter,
  shipped under the same `CalDavBackend` port trait (PRD §"Port traits
  declared in each kernel" row 8) for hot-swap parity.
- IaC: `microservices/calendar/iac/helm/radicale/` is the default Helm
  chart; `microservices/calendar/iac/helm/sabredav/` ships alongside but
  is enabled only via the `pack-us-healthcare` Kustomize overlay.
- LTS pins:
  - `radicale: "3.2.3"` (verified active upstream releases 2024-2026; AGPL-3.0
    accepted under the per-tenant tier-3 deploy carve-out; oyatie ships its
    own modifications back upstream).
  - `sabredav: "4.6.0"` (modified-BSD; no copyleft exposure).
  - `radicale-storage-postgresql: "0.4.0"` for Postgres backing (per PRD
    §"State strategy: mixed — Postgres event-store").
- Both backends speak RFC 4791 (CalDAV core), RFC 6638 (calendar
  auto-scheduling), and RFC 7953 (VAVAILABILITY). Both refuse XML
  external-entity expansion at the parser layer (XXE hardening).
- Both backends run behind the same mTLS + per-tenant API key + Cedar
  policy boundary as the rest of the µservice.

Both modes share three invariants:

- **Per-tenant filesystem / db root.** Radicale's tenant data lives in
  `${OPENBAO_TENANT_DATA_ROOT}/calendar/<tenant_id>/` with mode 700 and
  per-tenant uid; SabreDAV uses per-tenant Postgres schemas with RLS.
- **Event payload encryption-at-rest.** Both backends store
  Tenant-DEK-wrapped event blobs (per Bominal ADR-0111 envelope
  encryption); plaintext is never persisted.
- **Auto-scheduling refused for cross-tenant invitations unless
  Cedar-policy admits.** RFC 6638 auto-scheduling between two attendees
  in different tenants is gated by `tenant-scope.cedar` cross-tenant
  invite grant per PRD §"Cross-Tenant Invite".

## Alternatives Considered

### A. Cyrus IMAP+CalDAV as the integrated mail+calendar backend

- **Pros**:
  - Highest performance ceiling (~2000 concurrent CalDAV sessions per
    pod); used at Fastmail scale.
  - Excellent RFC 4791 + RFC 6638 + RFC 7953 + JMAP Calendars conformance
    (the same daemon also serves JMAP Calendar drafts to Fastmail's
    iOS/macOS clients).
  - Single operational surface for mail + calendar — simpler ops in a
    bundled product.
- **Cons**:
  - Architecturally bundles IMAP + CalDAV + CardDAV + JMAP in one daemon.
    Per ADR-0132 no-suite forward-policy, the `mail` µservice and the
    `calendar` µservice MUST be independently operable, independently
    scalable, and independently SLO'd; Cyrus IMAP+CalDAV makes that
    structurally impossible.
  - Forces the `mail` µservice to either coexist with calendar inside
    the same Cyrus pod (violating per-µservice isolation) or to deploy a
    separate Cyrus instance with the calendar surfaces disabled (wasting
    the integration benefit while paying the bundling cost).
  - Cyrus's CalDAV is implemented as an IMAP extension; the data model
    treats events as IMAP messages with `text/calendar` MIME parts. This
    bleeds IMAP semantics into the calendar surface (e.g., UID stability
    is tied to IMAP UIDVALIDITY).
- **Rejected** because the bundling violates ADR-0132 outright.
  Performance ceiling is irrelevant when the architectural premise is
  refused.

### B. Custom Rust-native CalDAV server (in-house implementation)

- **Pros**:
  - Single Rust runtime end-to-end — no Python (Radicale) or PHP
    (SabreDAV) operational surface.
  - Direct integration with the rest of the µservice's port traits — no
    adapter-layer translation.
  - Full control over the data model + lock granularity + transaction
    boundary.
- **Cons**:
  - CalDAV is RFC 4791 + RFC 6638 + RFC 7953 + RFC 4918 (WebDAV core) +
    RFC 5689 (WebDAV current-principal) + RFC 5995 (POST extensions for
    WebDAV) — ~8 RFCs of surface area. A from-scratch implementation
    that passes the public CalDAV interop corpus is a 6–12
    engineer-month undertaking before any oyatie-specific features.
  - Hyrum's-Law-bound client behaviour (Apple Calendar in particular is
    famously sensitive to PROPFIND/REPORT edge cases) means any new
    server has a long interop tail.
  - No upstream maintenance share — every CVE, every RFC clarification,
    every client quirk falls entirely on axis-calendar.
- **Rejected** under the "buy not build" principle per
  `feedback_automate_everything.md` (anything mechanical = scripted;
  anything well-trodden = use the existing implementation). Revisit only
  if both Radicale and SabreDAV upstream go cold.

### C. Apple Server.app CalDAV (proprietary fork)

- **Pros**:
  - Direct lineage to the Apple CalDAV reference implementation —
    perfect Apple Calendar interop by construction.
- **Cons**:
  - Proprietary; macOS-only deployment; no Linux container support.
  - Discontinued upstream (Server.app deprecated by Apple in macOS
    Sonoma); no security maintenance after 2024.
  - Unrunnable in oyatie's Linux Kubernetes substrate.
- **Rejected** outright — not deployable, not maintained.

### D. Baïkal (SabreDAV preconfigured stack)

- **Pros**:
  - SabreDAV preconfigured with a web admin UI; minimal ops effort to
    stand up.
- **Cons**:
  - Adds a web admin UI surface that oyatie does not need (oya-portal
    provides admin UX through its own port traits).
  - Bundles a specific MySQL backend; we want Postgres per PRD.
  - One more upstream to track CVEs for, on top of SabreDAV itself.
- **Rejected** in favour of consuming SabreDAV directly with our own
  config.

### E. Radicale 3.x as primary + SabreDAV 4.x as healthcare-pack alternative  ← **CHOSEN**

- **Pros**:
  - Both are proven OSS CalDAV servers with active upstreams and
    excellent RFC 4791 conformance.
  - Radicale's smaller footprint matches starter/pro tenant density;
    SabreDAV's higher session ceiling matches healthcare's heavier
    auto-scheduling workloads.
  - Two-backend posture provides hot-swap escape hatch — if Radicale
    upstream cools, we promote SabreDAV to primary; if SabreDAV upstream
    cools, we keep Radicale and re-pick the secondary.
  - ADR-0105 Amendment 3 backend-qualified adapter pattern admits this
    directly — `*-adapter-caldav-radicale` + `*-adapter-caldav-sabredav`
    are both first-class ports.
- **Cons**:
  - Two operational surfaces to maintain (Python runtime for Radicale,
    PHP runtime for SabreDAV).
  - Two CVE-tracking lanes — one per upstream.
  - Interop matrix doubles per release — every CalDAV client
    integration test must run against both backends per cell.
- **Accepted** — the operational cost is bounded; the architectural
  escape hatch is valuable; both backends are RFC-conformant LTS pins.

## Consequences

### Positive

- **CalDAV interop covered end-to-end.** Apple Calendar / Thunderbird /
  Evolution / DAVx5 / GNOME Calendar all work against Radicale by
  construction; SabreDAV provides a documented fallback for any client
  that exposes a Radicale-specific incompatibility.
- **No bundling anti-pattern.** Per ADR-0132, mail and calendar deploy
  independently; this ADR makes that possible despite the existence of
  Cyrus.
- **Hot-swap escape hatch.** ADR-0105 Amendment 3's backend-qualified
  adapter pattern means switching between Radicale and SabreDAV at the
  tenant level is a config flip, not a code change.
- **Healthcare-pack hardening route.** `pack-us-healthcare` gets the
  higher-throughput backend (SabreDAV) for free; the per-pack overlay
  is the right granularity for this choice.

### Negative

- **Two upstreams to track.** Radicale Python + SabreDAV PHP CVE feeds
  must both be monitored by ops-security. Mitigation: both upstreams
  publish CVEs to NVD; existing security automation covers both.
- **Backend-specific quirks.** Radicale's filesystem-storage default
  doesn't transactionally serialise concurrent PUTs to the same VEVENT;
  the Postgres storage backend (which we use) fixes this but adds a
  Postgres dependency in the CalDAV path. SabreDAV's PHP runtime needs
  PHP-FPM management; we run it under `php:8.3-fpm-alpine`.
- **Interop matrix doubled.** Every Apple Calendar / Thunderbird /
  Evolution / DAVx5 E2E test in `tests/e2e/caldav-clients.rs` runs
  against both backends in CI — doubled cost.

### Operational

- **New CI lane `oya-governance-caldav-backend-conformance`** (BLOCKER
  from M03): validates that
  - both `oya-calendar-ics-import-export-adapter-caldav-radicale` and
    `oya-calendar-ics-import-export-adapter-caldav-sabredav` exist;
  - both pass the libical RFC 4791 conformance corpus;
  - both expose the `CalDavBackend` port trait identically (Cedar policy
    + audit-chain seal emitted by both at the same surface points).
- **Helm chart pin policy**: `radicale: "3.2.3"` and `sabredav: "4.6.0"`
  declared in `microservices/calendar/iac/helm/Chart.yaml` `dependencies`;
  `oya-governance-version-pinning-conformance` lane refuses unpinned
  versions.
- **Per-pack overlay**: only `pack-us-healthcare` enables the SabreDAV
  chart by default; other packs ship Radicale-only and may
  tenant-opt-into SabreDAV via a tenant-tier flag.
- **Runbook `caldav-sync-loop.md`** documents the most common backend-
  visible failure mode (a misbehaving Apple Calendar client looping on
  `If-Match` + ETag mismatches); the runbook is backend-agnostic at the
  symptom layer but cites backend-specific log lines per backend.

### Regulatory

- **GDPR Art. 32** (security of processing): both backends pass the
  "appropriate technical measures" bar — TLS 1.3 in transit, Tenant-DEK
  envelope at rest, per-tenant filesystem/schema isolation.
- **KR PIPA Art. 29** (technical/managerial measures): per-tenant
  filesystem isolation (Radicale) or schema-level RLS (SabreDAV)
  satisfies the access-control-by-default requirement.
- **HIPAA 45 CFR §164.312(a)(2)(iv)** (encryption/decryption controls):
  Tenant-DEK envelope satisfies; SabreDAV's heavier scheduling workload
  in pack-us-healthcare does NOT change the encryption story.
- **EU AI Act**: out of scope (CalDAV is not AI).

## Verification

Per the agent-skills documentation-and-adrs SKILL.md §"Verification":

- [ ] **Both backends pass libical RFC 4791 conformance corpus** —
  `cargo nextest run -p oya-calendar-ics-import-export-adapter-caldav-radicale -- rfc_4791_corpus`
  and equivalent for `-sabredav`.
- [ ] **CalDAV E2E suite passes against both backends** —
  `cargo nextest run -p tests --test e2e_caldav_clients`.
- [ ] **Helm chart versions pinned** —
  `cargo run -p oya-dev-cli -- gate validate version-pinning-conformance --microservice calendar`.

## References

- RFC 4791 — Calendaring Extensions to WebDAV (CalDAV).
- RFC 4918 — HTTP Extensions for Web Distributed Authoring and Versioning (WebDAV).
- RFC 5545 — iCalendar.
- RFC 5689 — Extended MKCOL for WebDAV.
- RFC 5995 — Using POST to Add Members to WebDAV Collections.
- RFC 6638 — Scheduling Extensions to CalDAV.
- RFC 7953 — Calendar Availability (VAVAILABILITY).
- Radicale upstream — `radicale.org`; LTS 3.2.x release notes.
- SabreDAV upstream — `sabre.io/dav/`; LTS 4.6 release notes.
- Cyrus IMAP+CalDAV — `cyrusimap.org` (rejected reference).
- libical CalDAV conformance corpus — `github.com/libical/libical`.
- Apple CalDAV interop tests — `github.com/apple/ccs-calendarserver` (historical reference).
- ADR-0056 (BNF v4.1); ADR-0105 Amendment 3 (backend-qualified adapters); ADR-0126; ADR-0131; ADR-0132; ADR-0133.
- `microservices/calendar/PRD.md` FR-09 + AC-04 + §Bounded Contexts row 6.
- `microservices/calendar/iac/helm/Chart.yaml`.
- `microservices/calendar/runbooks/caldav-sync-loop.md`.
