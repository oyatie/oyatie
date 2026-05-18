---
doc_class: SDKPlan
template_id: TPL-SDK-PLAN
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + axis-sdk
related_adrs: [ADR-ANON-0001, ADR-ANON-0002]
doc_status: published
---

# SDK Plan: anonymous µservice

## SDK targets

| SDK | Audience | Privacy-bar | Status |
|---|---|---|---|
| `oya-anonymous-sdk-rust` | Server-side integrators + internal Rust services | Full I1-I7 invariant assistance | P01 (this phase) |
| `oya-anonymous-sdk-typescript` | Web client + Node integrators | Full I1-I7 + bundle-size budget | P01 (this phase) |
| `oya-anonymous-sdk-swift` | iOS native | Full I1-I7 + iOS-keychain integration | P02 |
| `oya-anonymous-sdk-kotlin` | Android native | Full I1-I7 + Android-keystore integration | P02 |

## API surface (P01)

- Authentication: OIDC for affinity-IdP linkage ONLY; once linked, all subsequent posting auth via blind-signed credential.
- Affinity attestation: client-side `verify_attestation(claim, proof)` runs BBS+ verify locally + cache.
- Posting: `publish_post(body, visibility, parent_post_id?)` returns `{ post_id, blinded_commitment_id }`; client never sees real-user identifier in response.
- Vote: `upvote(post_id)` / `downvote(post_id)` with proof-of-credential-binding.
- Feed render: `render_feed(affinity_id, mode)` with cursor + Cedar filtering server-side.
- Notification subscription: SSE / WebSocket subscription with opaque-handle payloads.
- Anonymous-DM (P02): MLS group operations.

## SDK invariant assistance

| Invariant | SDK assistance |
|---|---|
| I1 — no platform-side correlation | Client cache for blind-signed credentials; client refuses to send user_id in any header; explicit no-user-id type-bound on request structs |
| I2 — affinity reveals affinity not identity | Client SDK helper for BBS+ proof generation; selective-disclosure UI explainer |
| I3 — short retention | Client SDK exposes `retention_tier()` to display "your post will expire in N days" |
| I4 — no third-party trackers | Client SDK build-time refuses to import any third-party telemetry; SBOM check at release |
| I5 — no federation | Client SDK has no federation API; refuses inbound federation traffic |
| I6 — E2E for DM | Client SDK manages MLS keystore; server-bound state is ciphertext only |
| I7 — legal-process | Client SDK exposes transparency-report viewer; renders quarterly disclosures |

## Bundle size budget (TypeScript)

| Bundle slice | Budget |
|---|---|
| Core SDK (auth + posting + voting) | ≤ 30 KB gzipped |
| BBS+ verify (subtle-crypto + WASM fallback) | ≤ 50 KB gzipped |
| Blind-signature client (`@oyatie/ring-wasm` slice) | ≤ 45 KB gzipped |
| MLS client (anonymous-DM; P02) | ≤ 80 KB gzipped |
| Total core (P01) | ≤ 125 KB gzipped |
| Total with DM (P02) | ≤ 205 KB gzipped |

## SDK error model

- `ANONYMITY_INVARIANT_VIOLATION` — server returned a payload containing a forbidden field (user_id); client SDK refuses to consume + emits client-side telemetry to OBS (without user_id of course; just a refusal-count)
- `AFFINITY_ATTESTATION_INVALID` — BBS+ proof failed verify
- `K_ANONYMITY_FLOOR_REACHED` — community membership below floor; advisory shown to user
- `RETENTION_TIER_BOUND_EXCEEDED` — tenant-policy violation at attempt
- `LEGAL_PROCESS_PENDING` — UI-only; ensures correct accessibility of transparency notice flow

## SDK release process

- Per-release SLSA L3 provenance.
- Per-release SBOM emit + LEAN lane scan for third-party tracker fingerprints.
- Per-release breaking-change check via `oya-check-no-silent-regression`.
- Versioning: semver with `major.minor.patch`; major bump requires migration guide + 6-month sunset of prior major.

## SDK documentation surface

- Per-method reference docs.
- Conceptual guides: "Understanding cryptographic blinding", "Affinity attestation explained", "Why we cannot federate", "Reading the transparency report".
- Compliance overlays per pack: "EU DSA Art. 14 disclosures via SDK", "KR PIPA Art. 24-2 conformance via SDK", etc.

## Stability tiers

| Tier | API surface | Stability promise |
|---|---|---|
| Stable | core posting + vote + feed + auth + attestation verify | 12-month deprecation notice |
| Preview | anonymous-DM (MLS) — P02 release | 6-month deprecation notice |
| Experimental | algorithmic-ranking client integration | no stability promise |
