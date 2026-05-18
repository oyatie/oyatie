---
id: ADR-WS-0004
title: Hybrid server pre-eval + client render strategy for the jurisdiction overlay renderer
microservice: workflow-studio
status: Accepted
date: 2026-05-17
owner: axis-workflow + council-architecture
deciders: council-architecture, axis-workflow, ops-security, axis-tenancy
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0117, ADR-0131, ADR-0140]
related_specs: [/specs/microservices/workflow-studio.json]
related_artifacts:
  - microservices/workflow-studio/PRD.md (FR-08, FR-09, FR-16, AC-04, AC-07)
  - microservices/workflow-studio/IP-010-jurisdiction-overlay-renderer-full.md
  - microservices/workflow-studio/dpia.md
  - microservices/workflow-studio/compliance.md
purpose: Establish how the workflow-studio jurisdiction-overlay-renderer BC computes and renders the 11-pack jurisdiction overlays in the visual canvas — specifically, how PII / PHI / SECRET data-class markers and cross-border-flow warnings are computed without surrendering tenant isolation or Cedar's default-deny posture.
doc_status: published
---

# ADR-WS-0004: Jurisdiction overlay renderer — hybrid server pre-evaluation + client rendering

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

PRD §"Functional Requirements" FR-08 (jurisdiction overlay switching: kr / eu / us-hc), FR-09 (Cedar policy preview before save), FR-16 (data_class markers on PII / PHI / SECRET fields), AC-04 (jurisdiction view switch with overlay-resolved render), AC-07 (PII marker + Cedar preview before save) collectively define a uniquely-oyatie capability: **the visual canvas must flag, before save, any node or edge that touches a jurisdiction-sensitive field**.

Use-cases this must serve:

1. A tenant on `pack-kr` authors a workflow that reads from a customer-information table; the canvas must visually mark the PII fields *and* mark any edge that would carry that data across the KR border (e.g., a node that calls a non-KR-region API).
2. A tenant on `pack-us-healthcare` authors a workflow that touches PHI; the canvas must surface the BAA-required-coverage warning if any node lacks a HIPAA-aware adapter.
3. A tenant on `pack-eu-gdpr` authors a workflow that reads PII; the canvas must mark the lawful-basis requirement and refuse save if no basis is asserted.

The decision concerns *where* the overlay computation happens: server-side, client-side, or split. Each option carries trust-boundary and performance consequences.

Constraints:

- **Cedar default-deny** (per ADR-0140 inherited): authorisation is server-authoritative. The client (browser) cannot be trusted to evaluate policies; a client-only render risks showing a permissive view to a user whose actual save would be denied.
- **TTI budget**: PRD §"Performance" — editor TTI cold p99 ≤ 2s. Overlay computation cannot serialize behind a synchronous server round-trip on every keystroke.
- **Frame budget**: PRD §"Performance" — collab CRDT merge p99 ≤ 100ms. Overlay re-computation on every CRDT-driven canvas change must not block the frame.
- **Tenant isolation**: jurisdiction-pack data (which fields are PII in pack-kr, which edges are cross-border) is per-tenant per-pack; the client must not download other tenants' rule sets.
- **Policy source of truth**: Cedar policies live in `oya-cedar-policy` ports server-side; the same policy bundle must drive both the client preview and the server final-save check, otherwise the preview becomes deceptive.
- **Air-gap evaluation**: certain tenants (sovereign-cloud, defence) require that no per-keystroke Cedar evaluation crosses to a region they don't control. The hybrid must support an "all-server" degenerate mode for these tenants.

PRD Open Question on this topic is implicit (FR-08, FR-09, FR-16 require the overlay; the *how* is left to per-µservice ADR). The choice is between three architectures:

- **A**: Server-rendered overlay layer (server emits visual deltas; client renders them).
- **B**: Client-side Cedar evaluation (Cedar policy bundle shipped to client; client evaluates).
- **C**: Hybrid — server pre-evaluates a compiled overlay-decision-bundle for the current spec; client renders without re-evaluating.

## Decision

Adopt **C — hybrid server pre-evaluation + client rendering** with the following design:

1. **Overlay-decision-bundle**: a typed, per-tenant, per-spec-version artifact produced server-side that contains all overlay decisions for the current spec (e.g., `[{node_id: "n42", marker: "PII", reason: "field.customer_email is data_class=PII in pack-kr"}, {edge_id: "e7", marker: "CROSS_BORDER", reason: "target node calls us-east-1 API; tenant pack-kr; egress requires KR-FSS dual-control"}]`). The bundle is JCS-canonicalized per ADR-WS-0002 and Ed25519-signed; cache-keyed by `(tenant_id, spec_version_sha, pack_id)`.
2. **Server pre-evaluation**: the `jurisdiction-overlay-renderer-domain` crate computes the bundle by running Cedar policies against the current spec's nodes and edges. Cedar evaluation is server-authoritative; tenant policy fragments are merged with pack defaults at evaluation time.
3. **Client rendering**: the visual-canvas Leptos adapter consumes the overlay-decision-bundle and renders markers per (node_id, edge_id) lookups. The client does NOT evaluate Cedar; it only renders pre-computed decisions. The client is informed which markers are "stale" (bundle was computed against an older spec_version_sha) and shades them accordingly.
4. **Incremental refresh**: on CRDT merge that changes the spec, the canvas optimistically renders prior markers as "stale-overlay" (greyed) and dispatches a server request for a refreshed bundle. The frame is not blocked. The refresh is debounced (250ms idle) for high-frequency drag interactions; force-refresh on save-button-click.
5. **Save guard**: the save path re-evaluates Cedar server-side using the canonical spec bytes; the client-rendered overlay is informational. If save-time Cedar denies an operation the client preview permitted, the diagnostic surfaces precisely which policy denied and a refresh of the overlay-decision-bundle is forced. This closes the trust-boundary loop.
6. **Bundle distribution**: the overlay-decision-bundle is served from the editor REST surface (per-tenant, per-spec-version cache key); the bundle is per-tenant — never broadcast cross-tenant. Cache TTL: bundle is invalidated on (a) spec_version_sha change, (b) tenant policy fragment change, (c) pack-policy upgrade.
7. **Air-gap degenerate mode**: tenants with `air_gap_eval=true` skip the optimistic stale-overlay render; every overlay refresh is synchronous and blocks the relevant region of the canvas. Throughput trades for tenant-controllability.
8. **Marker taxonomy** (closed set, governed by `oya-foundry-gate-catalog-domain`): `PII`, `PHI`, `SECRET`, `PCI`, `CROSS_BORDER`, `MISSING_LAWFUL_BASIS`, `MISSING_BAA_COVERAGE`, `REGIONAL_DEGRADATION`, `LICENSE_OVERSEAT`. Extension requires an ADR superseding ADR-WS-0004.

## Alternatives Considered

### Alternative A — Server-rendered overlay layer (server emits visual deltas; client mirrors)

- **Pros**
  - Single trust boundary; client renders only what server says, never makes its own decisions.
  - Cedar evaluation is server-authoritative by construction; no risk of preview/save divergence.
  - Simplifies per-tenant policy versioning — server always has the latest.
- **Cons**
  - Every canvas change requires a server round-trip to refresh the overlay; conflicts with the 16.7ms (60 fps) frame budget during drag.
  - WebSocket bandwidth multiplies: visual-delta encoding is fat, and a single drag gesture is 30+ frames.
  - Multi-region tenants (`pack-us-healthcare` + `pack-kr` for a multi-region SaaS) get region-pinned overlay computation; cross-region collab gets blocked on the slowest region's overlay refresh.
  - The server becomes the rendering bottleneck for the per-tenant canvas; degrades the canvas TTI metric.
- **Rejected reason**: incompatible with the 60-fps drag frame budget and the multi-region collab story. Server-authoritative is necessary for correctness but not sufficient for the UX.

### Alternative B — Client-side Cedar evaluation (Cedar policy bundle shipped to client)

- **Pros**
  - Zero server round-trip on overlay refresh; sub-millisecond marker updates.
  - Cedar's Rust SDK runs natively in browser-WASM; performance is excellent.
  - Per-keystroke Cedar preview is trivially fast.
- **Cons**
  - **Trust boundary violation**: the client (browser) cannot be authoritative about policy decisions because it can be modified by the tenant user. A malicious tenant user could patch the client Cedar evaluator to permit operations the server would deny — they'd then see an overly-permissive preview, hit save, and get a server-side denial. Worse, the preview-vs-save divergence creates a "the editor lied to me" UX failure.
  - Policy bundles contain rules from pack defaults; shipping the rule set to every tenant browser leaks the pack's full enforcement contract (acceptable for open packs; not acceptable for tenant-private policy fragments).
  - Cedar-runtime in browser-WASM adds ~150 KB gzip to the bundle, pushing the TTI budget.
  - Air-gap tenants would need a server-only path anyway, doubling the implementation.
- **Rejected reason**: trust boundary violation. Cedar's default-deny posture (ADR-0140) is load-bearing for security; a client-authoritative preview undermines it. The preview-vs-save divergence is a UX anti-pattern (the "Cedar policy preview before save" claim in `competitor-parity-matrix.md` §"Key oyatie Differentiators" #2 requires the preview to be *truthful*; client-side eval makes it untruthful when the client lies).

### Alternative C — Hybrid (chosen)

See Decision above.

- **Pros**
  - Server is the sole Cedar authority; preview is computed by the same code path as save (overlay-decision-bundle is derived from the same Cedar eval as save-guard).
  - Bundle caching makes per-keystroke rendering effectively free (cache hit on stable spec_version_sha).
  - Per-tenant bundle scope means no cross-tenant policy leak.
  - Air-gap mode is a configuration switch, not a separate implementation.
- **Cons**
  - Stale-overlay states must be communicated clearly to the user; "the marker you see is for the previous spec_version_sha" is a UX subtlety.
  - Two code paths for overlay rendering (synchronous air-gap vs optimistic-with-refresh standard); test surface doubles.
  - Bundle refresh on every save+CRDT-merge adds REST traffic to the editor surface (mitigated by debounce + cache keying).
- **Accepted reason**: the only architecture that simultaneously preserves Cedar's default-deny + server-authoritative posture, the 60-fps frame budget, multi-region collab, and the truthful-preview UX contract.

### Alternative D — Pre-compile overlay decisions at spec-load time only (no live refresh)

A degenerate hybrid: compute the overlay-decision-bundle once on editor open, never refresh until save.

- **Pros**
  - Simpler than the chosen design; no refresh mechanism.
- **Cons**
  - Stale overlay during a long editing session is the norm, not the exception; the canvas markers diverge from the actual canvas state.
  - "PII marker visible during edit" per AC-07 fails: a user adds a PII field mid-session and the marker doesn't appear until save.
- **Rejected reason**: AC-07 verification fails — PII markers must be visible during edit, not only at save.

## Consequences

### Architectural

- The `jurisdiction-overlay-renderer-domain` crate consumes Cedar evaluation results from a kernel-defined `OverlayResolver` port (per PRD §"Port traits") and emits the overlay-decision-bundle. The Cedar evaluation itself is in the tenancy / oya-cedar-policy server-side port.
- The `JurisdictionDiffEngine` port computes the visual diff between overlays when the user switches jurisdiction; this is pure (no Cedar) because both overlays were already pre-evaluated server-side.
- The visual-canvas adapter ingests the bundle and renders markers via keyed Leptos `<For>` components matching node_id / edge_id; this matches the ADR-WS-0003 keyed-list-rendering pattern.
- The bundle is JCS-canonicalized + Ed25519-signed; this lets the client verify the bundle was produced by the tenant's authoritative policy version and not tampered with in transit.

### Downstream impact on other µservices and IPs

1. **IP-010 (jurisdiction-overlay-renderer)** — authors the `OverlayResolver` + `JurisdictionDiffEngine` ports; integration tests cover all 11 packs.
2. **IP-012 (visual-canvas leptos-wasm + rest + sdk + app)** — adds the bundle-fetch REST endpoint + WebSocket-pushed refresh notification + client-side marker rendering.
3. **tenancy µservice** — Cedar policy fragments per tenant; tenant policy-version change invalidates bundle cache.
4. **workflow-engine µservice** — engine save path runs the same Cedar evaluation as the bundle computation; ensures preview-vs-save parity.
5. **observability µservice** — `editor-experience.json` dashboard gains `overlay_bundle_refresh_p99_ms`, `overlay_stale_render_count`, `overlay_preview_save_divergence_count` SLIs. The divergence count MUST be zero in any 24h window; non-zero is a Sev-2 alert.
6. **cloud-iac µservice** — bundle cache (Redis) sizing per pack; per-tenant cache key partitioning.
7. **All 11 jurisdiction packs** (pack-kr, pack-eu-gdpr, pack-us-healthcare, etc.) — author Cedar policy fragments + marker mappings that feed the bundle computation.

### Trust-boundary contract

- The bundle is signed; the client verifies the signature before render. A signature mismatch refuses to render any overlay (fail-closed).
- The bundle's `policy_version_sha` is shown in the canvas UI; users can verify they're seeing decisions from the current policy bundle.
- Save-time re-evaluation is mandatory and never skipped; the preview is a UX optimization, not a security control.

### SLOs and CI lanes affected

- `oya-governance-cedar-preview-required` — BLOCKER on `dev` per PHASE-01 (every save path exercises Cedar policy preview).
- `workflow-studio.overlay_bundle_refresh_p99_ms` — target ≤ 300ms p99.
- `workflow-studio.overlay_preview_save_divergence_count` — Sev-2 alert if non-zero in any 24h window.
- `workflow-studio.overlay_marker_render_p99_ms` — target ≤ 50ms p99 (client-only, cache-hit path).
- `workflow-studio.overlay_signature_verify_failures_count` — Sev-1 alert if non-zero (signature tampering or supply-chain compromise).

### Compliance + audit

- Each bundle refresh emits an audit row `overlay_bundle_emitted{tenant, spec_version_sha, policy_version_sha, pack_id, marker_count}`.
- Each save-time divergence (preview said permit, save said deny — or vice versa) emits a Sev-2 audit row + page.
- Air-gap-tenants emit `overlay_air_gap_synchronous_render{tenant, latency_ms}` for compliance evidence.

### Risk register

- **Risk**: bundle staleness window during heavy editing leads to mis-rendered markers. **Mitigation**: stale-overlay shading + debounced refresh; SLO on stale-render-count.
- **Risk**: signature key rotation breaks signature verification. **Mitigation**: bundle-signing key has a documented rotation procedure with overlap window per ADR-0140.
- **Risk**: client-side bundle parsing bug renders a permissive marker for a denied operation. **Mitigation**: save-time re-evaluation catches this; SLO on preview-save-divergence-count enforces the invariant.
- **Risk**: air-gap mode imposes unacceptable UX latency. **Mitigation**: documented as a tenant-tier trade-off; SLO is per-tier.

## References

- PRD `microservices/workflow-studio/PRD.md` FR-08, FR-09, FR-16, AC-04, AC-07.
- `microservices/workflow-studio/IP-010-jurisdiction-overlay-renderer-full.md`.
- `microservices/workflow-studio/dpia.md`.
- `microservices/workflow-studio/compliance.md`.
- ADR-0140 — Cedar policy enforcement (inherited; default-deny posture is load-bearing).
- ADR-0117 — Cloud-native infrastructure / data residency (jurisdiction-pack region pinning).
- ADR-WS-0002 — DSL canonical form (bundle canonicalization).
- ADR-WS-0003 — Leptos WASM substrate (client rendering surface).
- Cedar — `www.cedarpolicy.com`, `github.com/cedar-policy/cedar`.
- W3C Subresource Integrity — `www.w3.org/TR/SRI/` (signature verification analog).
- RFC 8785 — JSON Canonicalization Scheme (bundle canonical form).
