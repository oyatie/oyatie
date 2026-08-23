---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Canonical locale-routing precedence: per-tenant default → per-user override → Accept-Language →
  source-locale fallback. Drives shared-i18n-kernel runtime resolution + per-pack overlays.
canonical_authority: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
related_adrs:
  - ADR-0064
  - ADR-0185
  - ADR-0206
---

# Locale Routing Standard

## Authority

This standard implements the locale-routing precedence called out in ADR-0206.

## Precedence (highest-wins)

1. **Per-user override** — set in the user profile (`user.preferred_locale`).
   Resolved via tenancy + identity µservices; cached per session.
2. **Per-tenant default** — set in the tenant admin (`tenant.default_locale`).
   Tenant-admin-mutable; falls back to source-locale when unset.
3. **`Accept-Language` header** — RFC 9110; weighted q-values.
   Server picks the highest-weighted locale that matches the tenant's enabled locale set.
4. **Source locale** — `en-US`. Ultimate fallback.

## Per-tenant locale gating

A tenant's enabled locale set lives in `tenant.enabled_locales`. The server filters
`Accept-Language` candidates to those in the enabled set BEFORE applying user override.
This prevents accidental exposure of an unsanctioned-locale catalog (which may be
incomplete or pre-release).

## Per-regional pack overlay (per ADR-0064)

Regional packs (e.g., KR pack, UAE/SA pack, EU pack) ship per-locale message overrides at
`microservices/<ms>/clients/i18n/packs/<pack>/<locale>.ftl`. Resolution order:

1. Per-pack per-locale overlay (most specific).
2. Per-locale catalog (canonical).
3. Source locale.

## Server-side rendering

For SSR (SvelteKit / Leptos), the locale resolution runs once per request before any
template renders. The resolved locale flows through the request context and is available
to every component via the framework's i18n adapter.

## Client-side hydration

The resolved locale is serialized to the HTML root (`<html lang="ar-SA" dir="rtl">`) so the
client-side hydration agrees with SSR (avoids hydration mismatch warnings).

## Caching

- Per-tenant default: cached in tenancy µservice (5-minute TTL).
- Per-user override: cached in session (per-request).
- `Accept-Language` parsing: stateless.

## Anti-patterns

1. Reading locale from a cookie that survives logout — leaks across users on shared devices.
2. Caching per-locale rendered HTML at the CDN without `Vary: Accept-Language, Cookie`.
3. Mixing locales mid-page (e.g., header in `en-US`, content in `ko-KR`) — must be one locale per page.

## Cross-references

- ADR-0206 — i18n substrate.
- ADR-0064 — canonical base + localization (per-pack).
- `i18n-canonical.md` — Fluent + ICU.
- `rtl-rendering.md` — RTL bidi.
- RFC 9110 — HTTP Semantics (Accept-Language).
