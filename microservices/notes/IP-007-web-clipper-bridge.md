---
doc_class: ImplementationPlan
impl_plan_id: IP-007-web-clipper-bridge
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes + ops-security
acceptance_lanes: [cargo-check, cargo-test, oya-governance-port-location, web-extension-security-review]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: web-clipper-bridge + browser extensions (Chrome MV3 + Firefox MV3 + Safari Web Extensions + Edge Add-ons)

## Intent

Land `oya-notes-web-clipper-bridge-*` (server-side ingest) + browser extension code (Chrome MV3 + Firefox MV3 + Safari Web Extensions + Edge Add-ons).

Per-installation token rotation 90d. MV3 isolated-world execution. Minimum-permission manifest (no broad host_permissions; `activeTab` only).

## Extension Manifest (MV3)

```json
{
  "manifest_version": 3,
  "name": "oyatie notes — Web Clipper",
  "version": "1.0.0",
  "permissions": ["activeTab", "storage", "notifications"],
  "host_permissions": [],
  "background": {"service_worker": "service-worker.js"},
  "action": {"default_popup": "popup.html"},
  "content_security_policy": {"extension_pages": "default-src 'self'; connect-src https://*.oyatie.dev"}
}
```

## Test Plan

- Capture latency p95 ≤ 500ms.
- Installation token rotation 90d enforced.
- Per-installation token never exposed via DOM (MV3 isolated world).
- Local-queue mode (offline) replays on reconnect.

## Acceptance Gates

```bash
cargo check -p oya-notes-web-clipper-bridge-kernel
npm run lint --prefix extensions/chrome
npm run test --prefix extensions/chrome
cargo run -p oya-dev-cli -- gate validate web-extension-security
```

## Halt Conditions

- Extension fails security review (XSS via clipped HTML; token leakage via DOM) — block.
- Manifest grows beyond minimum-permission spec — review.

## Next IP

[`IP-008-share-link-and-embed.md`](IP-008-share-link-and-embed.md)
