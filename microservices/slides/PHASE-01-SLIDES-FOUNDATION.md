---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-workspace-preview
phase: P01-slides-foundation
status: Active
entry_gate: |
  PRD-slides accepted; ADR-0135 (Connect dissolution) accepted; ADR-0131 per-microservice flat layout in
  force; sibling docs + sheets µservices co-authored under same Loro CRDT family; messenger µservice
  LiveKit infra available for broadcast-mode signaling reuse; foundry-runtime SDK available for T0/T1/T2
  AI capabilities; tenancy + audit-chain + ontology + observability SDKs available; Layer-A IaC available
  via cloud-iac µservice (CDN + WebSocket gateway + Postgres + Valkey + S3 + gVisor sandbox for export
  workers); cargo workspace ready to accept the ~210 new crates under microservices/slides/src/crates/;
  ADR-SLIDES-0001 through ADR-SLIDES-0008 all merged.
exit_gate: |
  All 15 IPs merged; slides binary deployed to dev cluster (with WASM bundle on CDN); slides-pptx-roundtrip-subset
  CI lane present in .github/branch-protection.yaml required_status_checks on dev and staging;
  release/slides/{staging,production} pattern protection live; PPTX round-trip subset drill passes
  (import 100 reference decks, emit, reimport byte-equal over round-trippable subset ≥ 95%); collab CRDT
  merge drill passes (10 concurrent users, no silent loss); per-slide Cedar ACL drill passes;
  broadcast-mode LiveKit-bridge drill passes (200 concurrent viewers); reduced-motion fallback drill
  passes; ai-act-risk-class-stamp lane green; cargo nextest run --workspace exits 0; oya gate validate
  per-microservice-layout --microservice slides exits 0; oya gate validate authority-cohesion exits 0;
  HG-SLIDES gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: slides SLO promotion gate must exist before slides itself can be advanced past dev
  - milestone: M02b-substrate-ready
    phase: workspace-substrate prerequisites
    reason: docs + sheets co-authored; messenger LiveKit available; foundry-runtime gating wired
owner_team: axis-workspace + council-design-system
related_adrs: [ADR-0056, ADR-0065, ADR-0105, ADR-0106, ADR-0123, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-slides-foundation: Land the slides µservice end-to-end

## Purpose

Ship the slides µservice — presentation authoring + collab + present + broadcast + import/export + AI-assist — as one phase in M03-workspace-preview. Slides is a workspace hero product; every workspace tenant routes presentation authoring through it.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (CDN-cached WASM + per-tenant CRDT collab + Cedar per-slide ACL + gVisor sandbox for import/export).
- Nothing scheduled-for-distinct-tracked-work (every legacy connect-slides-* trail is structurally absent per ADR-0135; no compat seam authored).
- No silent regression (slides-pptx-roundtrip-subset + ai-act-risk-class-stamp + reduced-motion-fallback-mandatory CI lanes are BLOCKER day 1).
- Per-microservice flat layout (ships natively under ADR-0131).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `slides` | 31 BCs per PRD §"Bounded Contexts" | All under `microservices/slides/` per ADR-0131 | ~210 crates per PRD layer mapping |

Plus repo-wide:
- `.github/branch-protection.yaml` — add `oya-governance-slides-pptx-roundtrip-subset`, `oya-governance-ai-act-risk-class-stamp`, `oya-governance-reduced-motion-fallback-mandatory`, `oya-governance-cedar-preview-required` (if not already), `oya-governance-wasm-bundle-sri` to required_status_checks on `dev` + `staging`; add pattern protection for `release/slides/{staging,production}`.
- `Cargo.toml` (workspace) — register the ~210 new crates under `microservices/slides/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-SLIDES gate per ADR-0123.

### Out-of-scope

- Audio narration recording in-deck (scheduled-for-distinct-tracked-work subsequent-to-M03-completion).
- Live-stream-to-YouTube/Twitch bridge (scheduled-for-distinct-tracked-work; social µservice owns).
- Real-time translation overlay (scheduled-for-distinct-tracked-work; subsequent-to-M03-completion capability).
- Per-tenant branding mid-render (anti-pattern per workspace standards).

## Implementation Plans

Ordered. Each IP is one ChangeSet.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| `IP-001-layer-a-cdn-postgres-valkey-s3-ws-gateway-iac.md` | IaC for CDN + Postgres + Valkey + S3 + WS gateway + gVisor export pool | pending | axis-workspace + cloud-iac | — |
| `IP-002-presentation-slide-kernel-domain.md` | presentation + slide BCs kernel + domain | pending | axis-workspace + council-design-system | — |
| `IP-003-slide-layout-text-box-shape-kernel-domain.md` | slide-layout + text-box + shape + table + equation BCs kernel + domain | pending | axis-workspace | IP-002 |
| `IP-004-asset-bcs-image-video-audio-adapters.md` | image + video-embed + audio-embed BCs with ImageMagick / ffmpeg / ClamAV / OPSWAT adapters | pending | axis-workspace + ops-security | IP-002 |
| `IP-005-real-time-collaboration-loro-kernel-domain-adapter.md` | real-time-collaboration with Loro CRDT engine — no-silent-loss invariant | pending | axis-workspace | IP-002 |
| `IP-006-real-time-collaboration-worker-sdk.md` | WS gateway worker + SDK for real-time-collaboration | pending | axis-workspace + cloud-iac | IP-005 |
| `IP-007-chart-embed-bridge-to-sheets.md` | chart BC with live-link to sheets µservice via SDK; revocation cascade | pending | axis-workspace + sheets-team | IP-002, sheets-IP-X |
| `IP-008-themes-templates-master-slide-editor.md` | themes + templates + master-slide-editor + slide-sorter + layout-engine | pending | axis-workspace + council-design-system | IP-003 |
| `IP-009-animations-transitions-reduced-motion.md` | animations + transitions BCs with prefers-reduced-motion fallback (ADR-SLIDES-0004) | pending | axis-workspace + council-design-system + ops-accessibility | IP-003 |
| `IP-010-presenter-audience-view-broadcast-mode-livekit.md` | presenter-view + audience-view + broadcast-mode (LiveKit reuse via messenger SDK; ADR-SLIDES-0005) | pending | axis-workspace + axis-realtime | IP-006 |
| `IP-011-import-export-pptx-pdf-mp4-pipeline.md` | import-export BC with Pandoc PPTX bridge + WeasyPrint/Chromium-headless PDF + ffmpeg MP4 (gVisor sandboxed; ADR-SLIDES-0003) | pending | axis-workspace + ops-security | IP-004 |
| `IP-012-accessibility-ai-design-ai-content-generation.md` | accessibility + ai-design + ai-content-generation BCs; T0/T1/T2 capability wires; EU AI Act risk-class stamp (ADR-SLIDES-0006) | pending | axis-workspace + foundry-runtime-team + ops-accessibility | IP-002 |
| `IP-013-acl-comments-version-history-embed-bridge.md` | acl (per-slide + named-block; ADR-SLIDES-0007) + comments + version-history + embed-bridge (docs quotes + forms polls) | pending | axis-workspace + ops-security | IP-005 |
| `IP-014-visual-canvas-leptos-wasm-rest-sdk-app.md` | Composition: rest + app + adapter-leptos-wasm wiring for slide / text-box / shape / table / animations / transitions / slide-sorter / master-slide-editor / presenter-view / audience-view (ADR-SLIDES-0002) | pending | axis-workspace + council-design-system | IP-008, IP-009, IP-010, IP-012, IP-013 |
| `IP-015-hg-slides-registration-and-branch-protection.md` | `.github/branch-protection.yaml` updates; `/specs/hyperscaler-gates.json` HG-SLIDES registration; release pointer creation; competitor-parity-matrix evidence pinning; end-to-end launch verification | pending | axis-workspace + council-architecture | IP-014 |

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features --target wasm32-unknown-unknown -p oya-slides-slide-adapter-leptos-wasm
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --microservice slides
oya gate validate lean-a2 --microservice slides
oya gate validate port-location --microservice slides
oya gate validate layer-correctness --microservice slides
oya gate validate per-microservice-layout --microservice slides
oya gate validate statelessness --microservice slides
oya gate validate shardability --microservice slides
oya gate validate slides-pptx-roundtrip-subset --microservice slides
oya gate validate cedar-preview-required --microservice slides
oya gate validate wasm-bundle-sri --microservice slides
oya gate validate reduced-motion-fallback-mandatory --microservice slides
oya gate validate ai-act-risk-class-stamp --microservice slides
oya gate validate authority-cohesion
```

### End-to-end drill gates

| Drill | Source | Pass condition |
|---|---|---|
| PPTX round-trip subset | 100 reference PPTX decks under `tests/reference/pptx/` | re-import byte-equal over round-trippable subset for ≥ 95 / 100 |
| Loro CRDT no-silent-loss | proptest under `crates/oya-slides-real-time-collaboration-domain/tests/no_silent_overwrite.rs` | 1000 random op-stream pairs; never silent drop |
| Present-mode 60fps | `tests/load/present-mode-frame-budget.js` on 50-slide deck | p95 transition ≤ 50ms |
| Broadcast-mode LiveKit-bridge | `tests/e2e/broadcast-200-viewers.rs` | 200 concurrent viewers; session signal stable; SLO `oya-slides-broadcast-mode-availability` green |
| Chart-live-link revocation | `tests/integration/chart-revocation-cascade.rs` | sheets ACL revoke → chart access revoke + audit row ≤ 5s |
| Reduced-motion | `tests/e2e/reduced-motion.rs` | `prefers-reduced-motion: reduce` → animations BC swaps to fade-only fallback; AC-17 lane green |
| AI Act risk-class stamp | `cargo nextest run -p oya-slides-ai-content-generation-domain --test test_risk_class_stamp` | every T2 invocation carries an enum-class; refuses if Annex III high-risk unless pack-override |
| WASM bundle SRI | `cargo nextest run -p oya-slides-slide-adapter-leptos-wasm --test test_sri` | every chunk has SHA-384 SRI |
| Per-slide ACL | `cargo nextest run -p oya-slides-acl-domain --test test_per_slide_acl` | named-block-level Cedar evaluation; cross-slide cross-tenant refusal |

## References

- PRD `microservices/slides/PRD.md`.
- ADR-SLIDES-0001 through ADR-SLIDES-0008.
- ADR-0135 Connect dissolution.
- ADR-0131 per-microservice flat layout.
