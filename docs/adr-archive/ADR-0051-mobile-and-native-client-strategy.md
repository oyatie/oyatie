---
id: ADR-0051
status: Superseded
superseded_by: [ADR-709]
doc_status: published
amended_by: [ADR-0632]
---
# ADR-0051: Mobile and Native Client Strategy

- **Status:** Accepted
- **Date:** 2026-05-09
- **Owner:** `axis-workspace` (productivity native apps) + `axis-saas` (tenant-builder native apps) + `council-architecture`
- **Supersedes:** the legacy mobile-clients quality-bar, multi-form-factor, and Compose↔SwiftUI parity cluster per [`ADR-LEGACY-REGRESSION-MAPPING.md`](../ADR-LEGACY-REGRESSION-MAPPING.md)
- **Related:** ADR-0001 (cohesion thesis — single brand surface across all clients), ADR-0010 (regional-pack architecture — per-region store / privacy variations), ADR-0017 (brand naming + repo layout), ADR-0044 (gateway / mTLS — mobile-to-API edge)

---

## ADR-0632 product-protocol reconciliation

Native clients consume the same public HTTPS REST surface documented by OpenAPI 3.2.0, signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE streams, and bidirectional WebSocket sessions as web clients. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. Protobuf is reserved for internal-only gRPC/proto3 over HTTP/2 and is never a native-client or gateway contract.

## Context

The 2026-05-09 consolidation collapsed 127 legacy ADRs into a 50-ADR pack. The legacy pack contained a mobile-clients sub-cluster covering iOS / Android native shells for Connect, Corporate, Patient, and per-vertical workspaces. The Codex Round 2 verdict flagged the absence of a mobile/native ADR as a sole-source-of-truth gap: with the legacy pack retired, there was no consolidated ADR documenting how Oyatie ships native mobile, when, by whom, and against which canonical contracts.

This ADR fills that gap and decides scope, sequencing, and ownership rather than tech selection (concrete tech is per-product PRD).

## Decision

### 1. Web is the canonical surface

Every Oyatie product surface ships web-first. Web (Leptos for engineering surfaces; SvelteKit for tenant-facing UIs per per-product PRD) is the **canonical** rendering of every capability and the conformance reference for every native client. A capability is **not** considered shipped on a native platform until web parity is verified.

### 2. Native mobile is in scope at W-Workspace-Stable / W-Vertical-Pilot, not earlier

Native iOS / Android clients land in the wave sequence as follows:
- **W-Foundry-Preview / W-SaaS-Preview / W-Workspace-Preview**: web-only.
- **W-Workspace-Stable**: native mobile shells for Workspace Mail / Calendar / Meet / Chat / Drive ship in lockstep with the public Workspace GA (parity gate).
- **W-Vertical-Pilot**: per-vertical native shells ship only when the per-vertical PRD declares native a hard requirement (e.g. clinical bedside, industrial floor, logistics-driver). Tenant-facing apps without a native-only requirement remain web-only / PWA.
- **W-Vertical-Fan-Out**: native fan-out follows the per-vertical demand signal; not all 14 verticals get native.

### 3. Per-product PRD owns tech selection

Concrete tech (Tauri 2.x desktop / mobile, Dioxus, Compose Multiplatform, SwiftUI / Jetpack Compose native, KMM) is **scheduled-for-distinct-tracked-work to the per-product PRD** that authors the native shell. The decision is a per-product trade-off (binary size, native-feature parity, team skill, store policy), not a cross-microservice architecture decision. Each per-product PRD must:
- declare its mobile target matrix (iOS version floor, Android API floor, tablet support);
- declare its tech stack with rationale citing this ADR;
- adopt the per-pack regional store-policy bindings (Apple App Store / Google Play / Korea ONE Store / Huawei AppGallery / Samsung Galaxy Store / Naver Smart Store);
- adopt the per-pack content-safety + age-gating bindings per ADR-0010 + PRIVACY-PROGRAM §2.2.3 (children-product overrides apply to mobile equally);
- bind to the canonical capability registry per ADR-0021 + autonomy ceiling per ADR-0022 (no native-only autonomy paths).

### 4. Single brand surface holds across native

All native shells render the `Oyatie` brand per ADR-0017. App-store identifiers (bundle IDs, package names) align to `com.oyatie.<axis>.<product>` going forward. Migration from any pre-existing identifier is its own per-product PRD initiative (treated as identity work, not cosmetic rebrand).

### 5. Native shells consume the same canonical contracts

Native clients consume the same public HTTPS REST/OpenAPI contracts that web does, through the same gateway per ADR-0044 with mTLS device attestation; public Connect is not a client contract. There is no native-only API. Per-product PRDs may define **client-only convenience endpoints** (e.g. push-notification token registration, offline-sync deltas), but those endpoints stay in the canonical contract registry per ADR-0011 and obey the Data Use Boundary per ADR-0008.

### 6. Per-pack store-policy + per-region distribution

Regional packs per ADR-0010 supply per-region store-policy bindings. The canonical pack ships:
- App Store policy fitness lane (privacy nutrition labels, ATT prompt, IDFA opt-in, KR PIPC marketplace registration where required)
- Per-region store-listing template with i18n + content-safety + age-gating bound to the regional pack
- Per-region update / staged-rollout playbook

### 7. Quality bar (replaces the legacy mobile quality-bar decision)

Native shells must satisfy a CI fitness lane `oya-governance-mobile-native` that enforces:
- Crash-free sessions ≥ 99.5% per release (release blocker if regressed > 0.2%)
- Per-screen p99 cold-start ≤ 2s on the per-product PRD reference device set
- Accessibility audit pass (VoiceOver iOS / TalkBack Android)
- Per-pack store-policy validator pass
- Capability-invocation parity vs web (web-canonical reference)
- No native binary blob without an SBOM (per ADR-0039 supply-chain attestation)

### 8. Native shells respect the same audit chain

Every regulated capability invocation from a native client emits to the same audit chain per ADR-0003. Native does not get an audit-chain bypass. Offline / queued invocations replay into the chain on reconnect with original-timestamp + replay-anchor metadata.

## Consequences

### Positive
- Mobile is a tracked, sequenced, owned decision rather than a silent gap.
- Web-canonical reference forces capability-parity discipline.
- Per-product PRD authority avoids over-architecting one mobile stack across 14 verticals.
- Per-pack store-policy seam keeps mobile compatible with the regional-pack architecture.

### Negative
- Earlier waves (W-Foundry-Preview through W-SaaS-Preview) ship without native; KR-mobile-first user expectation is partly unmet pre-Workspace-Stable. Mitigation: web is mobile-responsive + PWA-installable as a stopgap.
- Per-product mobile decisions diverge in tech choice; cross-product native-engineer hiring becomes a heterogeneous skill matrix.
- Per-store-policy fragmentation (Apple / Google / KR ONE Store / Naver Smart Store / Huawei / Samsung / Galaxy Store) requires per-pack maintenance.

## Alternatives considered

- **Native-first parallel to web** — rejected; doubles W-Foundry-Preview / W-SaaS-Preview budget without proven mobile demand for engineering surfaces.
- **One mobile stack for all 14 verticals** (e.g. mandate KMM) — rejected; over-prescribes a per-product trade-off; incompatible with per-vertical native-only requirements (e.g. clinical bedside Apple Vision Pro / industrial Android-rugged).
- **PWA-only forever** — rejected; KR-market expectation, store-distribution + push-notification + offline-first parity for verticals like clinical / industrial / logistics-driver requires native.
- **Tauri-everywhere desktop-first** — defer to per-product PRD; valid for engineering surfaces (`oya admin`, `oya dev`) per ADR-0017 persona-split CLI but not load-bearing for tenant productivity.

## Adoption + verification

- Per-product PRDs that declare native-in-scope cite this ADR and emit a per-product mobile addendum.
- The planned advisory lane `oya-governance-mobile-native` lands as part of W-Foundry-Preview foundation work (lane scaffolded, blocking gates activate at W-Workspace-Stable).
- The Mobile Native Strategy is reviewed at every wave gate (Foundry-Preview through Region-Fan-Out); deviation requires founder + council-architecture sign-off.

## Sources
- Codex Round 2 verdict (`docs/raw/codex-verdict-round2-cleaned.md` § "Sole-source-of-truth gaps")
- [`ADR-LEGACY-REGRESSION-MAPPING.md`](../ADR-LEGACY-REGRESSION-MAPPING.md) §3.1 (legacy mobile cluster)
- [`PRD.md`](../PRD.md) §3.1 (wave sequence)
- [`PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md) §2.2.3 (children-product overrides)
- ADR-0001, ADR-0003, ADR-0008, ADR-0010, ADR-0011, ADR-0017, ADR-0021, ADR-0022, ADR-0039, ADR-0044
