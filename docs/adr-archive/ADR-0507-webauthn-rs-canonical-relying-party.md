---
id: ADR-0507
status: Superseded
planning_impact: false
deciders: founder, council-architecture
date: 2026-05-28
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0476, ADR-0482, ADR-0508]
door: two-way
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0507 — webauthn-rs canonical WebAuthn relying-party (Phase-1) + oya-webauthn Tier-2 bespoke destination

## Status

Accepted (2026-05-28).

## Context

oya-identity (ADR-0476) needs WebAuthn relying-party (RP) capability for two use-cases:

1. **OIDC-provider integration** — passkey as a first-factor or step-up authenticator within the
   OIDC flow.
2. **Standalone passkey login** — direct WebAuthn ceremony without OIDC wrapping (tenant
   portal, admin console).

`webauthn-rs` (maintained by Firstyear / Kanidm project) is the Rust-ecosystem canonical
WebAuthn RP implementation. It is pure Rust, implements the W3C WebAuthn Level 3 specification,
and is actively maintained with regular releases tracking the spec.

**Authenticator-side companion ADR**: OpenSK (authenticator-side FIDO2/CTAP2 firmware) is
adopted in parallel as **ADR-0508** — Phase-1 reference for `oya-authn-device` Tier-3 bespoke
hardware security key destination. ADR-0507 + ADR-0508 together form the **closed-loop oyatie
identity stack**: webauthn-rs handles the RP (server) side, OpenSK→oya-authn-device handles
the authenticator (hardware key) side. Founder reversal 2026-05-28: original OpenSK exclusion
overturned — adopt as Phase-1 reference with explicit in-house replacement timeline.

Per [[bespoke-over-oss-doctrine]], this establishes the Phase-1 (webauthn-rs OSS bridge) →
Tier-2 (oya-webauthn bespoke) phasing pattern. oya-webauthn is the mid-term destination,
unlocked when the parity table below is green and oya-identity Phase-2 promotion gate passes.

## Hyperscaler-lens pre-check

| Criterion | Result |
|---|---|
| Active upstream | PASS — Firstyear/Kanidm maintained; active at github.com/kanidm/webauthn-rs; regular spec-tracking releases |
| License clean | PASS — MPL-2.0; OSI-clean; file-level copyleft only, no viral linking for Rust crate consumers |
| Fully self-hostable | PASS — pure Rust library; zero managed-service dependency; links no external C library |
| Hyperscaler-internal equivalent | PASS — Google/Apple/Microsoft all run bespoke RP stacks internally; webauthn-rs is the Rust-ecosystem canonical equivalent of those internal stacks |

## Decision

1. **webauthn-rs is the canonical Phase-1 WebAuthn RP** across oyatie. All WebAuthn registration
   and authentication ceremonies are handled via webauthn-rs until oya-webauthn parity gates pass.

2. **Consumed via oya-identity's `oya-identity-webauthn-*` use-case crates**, following the
   PR #289 canonical clean-architecture pattern:
   `domain` / `usecase` / `api` / `adapter-postgres` / `rest` / `grpc` / `app`.

3. **oya-webauthn is the Tier-2 bespoke destination** (12-24 months). It is added to the
   ADR-0482 Tier-2 bespoke roadmap table with bridge=webauthn-rs and unlock-criteria=parity
   table green + oya-identity Phase-2 promotion gate.

4. **OpenSK is the closed-loop authenticator-side partner** (ADR-0508) — OpenSK is adopted as
   the Phase-1 authenticator firmware reference with `oya-authn-device` as the Tier-3 bespoke
   hardware security key destination. ADR-0507 (RP) + ADR-0508 (authenticator) together close
   the loop on the oyatie identity stack.

5. **webauthn-rs is the bridge indefinitely** until oya-webauthn parity is independently
   verified; no hard-deadline cutover.

## OpenSK — closed-loop authenticator-side companion (ADR-0508)

OpenSK (Google FIDO2/CTAP2 authenticator firmware for hardware security keys) is **not an RP
library** — it operates at the authenticator (hardware key) layer, complementary to this ADR's
RP layer. Founder reversal 2026-05-28: original scope-exclusion overturned. OpenSK is adopted
in parallel via **ADR-0508** as the Phase-1 reference for `oya-authn-device` Tier-3 bespoke
hardware security key destination (own-the-silicon ambition, aligned with kubers Phase-B).

ADR-0507 + ADR-0508 together form the **closed-loop oyatie identity stack**:
- ADR-0507 (this ADR) — webauthn-rs → oya-webauthn (RP / server side)
- ADR-0508 — OpenSK → oya-authn-device (authenticator / hardware-key side)

Do **not** introduce OpenSK as an RP candidate (it is not an RP library). Authenticator-side
scope is owned by ADR-0508.

## Feature parity target for future oya-webauthn

Required per [[bespoke-over-oss-doctrine]] — every bespoke ADR must include this table.
oya-webauthn must reach minimum parity before cutover from webauthn-rs is considered.

| Feature | OSS-substrate (Phase-1: webauthn-rs) | Bespoke minimum bar (oya-webauthn) | Phase |
|---|---|---|---|
| Registration ceremony | navigator.credentials.create flow | Same + attestation policy DSL | 2 |
| Authentication ceremony | navigator.credentials.get flow | Same + risk-scored step-up | 2 |
| Attestation formats | packed, fido-u2f, none, tpm, android-key, android-safetynet, apple | Same + custom oyatie-attestation format | 2 |
| Algorithms | ES256, ES384, ES512, RS256, EdDSA | Same + post-quantum (ML-DSA hybrid) | 2 |
| User verification | required/preferred/discouraged | Same + tenant policy override | 2 |
| Resident keys / discoverable creds | full support | Same + cross-tenant discovery scoping | 2 |
| Multi-credential per user | yes | Same + credential rotation/revocation API | 2 |
| Backup eligibility | BE/BS flag tracking | Same + cloud-sync risk policy | 2 |
| Metadata service | FIDO MDS3 | Same + private metadata catalog for enterprise | 2 |
| Replay defense | counter + signCount | Same + Cedar-policy-gated anomaly action | 2 |
| Audit | basic events | Per-event audit-chain entry + tenant attribution | 2 |
| Multi-tenancy | single-realm | Native tenant boundary in primitives | 2 |

## Bridge and migration

webauthn-rs is the bridge during Phase-1. Cutover to oya-webauthn is gated on:

- (a) All rows in the parity table above independently verified green
- (b) oya-identity Phase-2 promotion gate passes (per ADR-0476 promotion criteria)
- (c) Tenant opt-in period completed with no regressions in ceremony success rate

## Consequences

- oya-identity scaffold gains `oya-identity-webauthn-*` use-case slice:
  `domain` / `usecase` / `api` / `adapter-postgres` / `rest` / `grpc` / `app`
  (scaffold rework is a separate lane queued behind this ADR)
- Workspace `Cargo.toml` gains `webauthn-rs` in `[workspace.dependencies]`
- Lockfile churn minor — webauthn-rs is a pure Rust library with no heavy C deps
- oya-webauthn added to ADR-0482 Tier-2 bespoke roadmap table
- OpenSK is the closed-loop authenticator-side companion via ADR-0508 — not an RP candidate

## Related

- ADR-0476 — oya-identity (OIDC-provider + WebAuthn consumer)
- ADR-0482 — Bespoke Substrate Roadmap; oya-webauthn added as Tier-2 entry
- ADR-0508 — OpenSK canonical authenticator-side reference + oya-authn-device Tier-3 (closed-loop partner)
- [[bespoke-over-oss-doctrine]] — Phase-1 OSS bridge → Tier-N bespoke pattern
- [[hyperscaler-lens-architectural-filter]] — pre-check table above
