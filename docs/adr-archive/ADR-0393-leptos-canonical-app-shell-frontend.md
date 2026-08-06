---
id: ADR-0393
title: "Leptos canonical app-shell frontend (Rust/WASM SSR+hydration; supersedes ADR-0372 SolidJS)"
status: Superseded
planning_impact: true
deciders: founder, council-architecture
date: 2026-05-29
owner: council-architecture
supersedes: [ADR-0372]
superseded_by: [ADR-709]
amends: []
related: [ADR-0001, ADR-0013, ADR-0023, ADR-0090, ADR-0394, ADR-0509]
related_specs:
  - /specs/platform-architecture.json
  - /specs/http-stack-policy.json
milestone: M-APP-SHELL-FRONTEND
depends_on: []
door: two-way
numbering_note: "decisions.json next_adr is ADR-0392, but ADR-0392 and ADR-0408 are already allocated out-of-band by the Buck2 build/CI reversal lane (branch feat/adr-0392-0408-buck2-reversal-2026-05-29, not yet merged to dev). To avoid a number collision this ADR is deliberately allocated ADR-0393 (the next clean number after the Buck2 ADR-0392 allocation) and its sibling is ADR-0394. The numbering gaps ADR-0377..ADR-0391 and ADR-0395..ADR-0407 are left open and are NOT claimed by this lane; the ADR index will record ADR-0393/ADR-0394 as non-contiguous allocations alongside the existing documented gaps and the Buck2 ADR-0392/ADR-0408 allocations."
affected_surfaces:
  crates: [oya-application-shell-frontend-prototype, oya-ops-workspace-shell-rest, oya-ops-workspace-shell-app]
  microservices: [app-shell-frontend]
  specs: [/specs/platform-architecture.json, /specs/http-stack-policy.json]
---

# ADR-0393: Leptos canonical app-shell frontend (Rust/WASM SSR+hydration; supersedes ADR-0372)

## Status

Accepted — 2026-06-01 (founder-confirmed: all frontend work is Leptos/Rust-WASM; SolidJS is retired). Supersedes ADR-0372 (SolidJS). Originally drafted Proposed 2026-05-29 to overturn an Accepted decision and reconcile live codebase drift; founder confirmation on 2026-06-01 promotes it to Accepted. Downstream (tracked in the readiness backlog): regenerate the ADR index / machine-readable decisions from source; migrate the live `oya/app-shell-frontend` SolidJS app to Leptos; flip the `oya-ci-deck` reference (ADR-0513) to Leptos; add a superseded-reference lint so SolidJS cannot reappear as canonical.

## Date

2026-05-29

## Supersedes

ADR-0372 ("Frontend stack — SolidJS/TS app-shell + Rust→WASM compute modules"), in full. ADR-0372 chose TypeScript + SolidJS (+ SolidStart) as the canonical app-shell and prescribed a Leptos→SolidJS migration. That decision is reversed: Leptos (Rust/WASM, SSR+hydration) is the canonical app-shell frontend. SolidJS is NOT an acceptable canonical target and is not retained as an evaluation track. The two backend-direction conclusions that ADR-0372 reached — backend stays Rust, and WASM is for compute — are unaffected and re-affirmed here (Rust→WASM compute islands are retained).

## Superseded-by

—

## Related

ADR-0001 (one-product cohesion), ADR-0013 (OSI license policy), ADR-0023 (Wasmtime sandbox — server-side, not frontend), ADR-0090 (hyper canonical HTTP backbone + 2026-05-29 strategic hyper/axum split — the leptos_axum SSR host rides this), ADR-0394 (bespoke-Rust IDP central hub — the portal whose shell this ADR makes canonical), ADR-0509 (hyperscaler service decomposition — the flat single-crate-per-service layout the portal-shell crates follow).

## Owner

council-architecture (with founder as deciding authority — this is a doctrine reversal of an Accepted ADR).

## Context

### The standing founder directive

The founder directive is unambiguous: the app-shell frontend is **Leptos** (Rust/WASM full-stack — SSR + hydration). SolidJS is NOT acceptable as the canonical frontend.

### What ADR-0372 said, and the amendment that already started the reversal

ADR-0372 (Accepted 2026-05-26) chose SolidJS + SolidStart for the DOM-heavy app-shell, with Rust→WASM only for compute-bound widgets and OpenAPI 3.2.0→TS codegen for type safety. It prescribed retiring the Leptos prototype (deliverable D4).

ADR-0372 was then **amended 2026-05-27** — the supersede-Leptos stance was REVERSED: "Leptos (`crates/oya-application-shell-frontend-prototype`, Rust→WASM) is RETAINED as the **canonical** app-shell frontend. SolidJS is retained as an **evaluation track only** … SolidJS replaces Leptos as canonical ONLY if it demonstrates superiority **at massive scale**; absent that proof, Leptos stays. Deliverable D4 (retire the Leptos prototype) is WITHDRAWN." So the latest authored intent on `dev` already says **Leptos canonical**.

### The drift this ADR resolves (honest accounting)

The codebase reflects the PRE-amendment SolidJS direction, not the amended Leptos-canonical intent. The ADR text and the code are out of sync; the code is the thing that drifted:

- `microservices/app-shell-frontend/` is a **live SolidJS app**: `package.json` + `pnpm-lock.yaml` + `pnpm-workspace.yaml` + `app.config.ts` (SolidStart), with real TSX surfaces (`src/app.tsx`, `src/components/ShellRail.tsx`, `ShellHeader.tsx`, `DashboardIsland.tsx`, `src/entry-client.tsx`, `src/entry-server.tsx`, `src/lib/render-envelope.ts`) and a generated TS client (`generated/ops-workspace-shell.d.ts`).
- `microservices/app-shell-frontend/MIGRATION-PLAN.md` drives the **Leptos→SolidJS** migration ("Authority: ADR-0372 … Migration plan: Leptos prototype → SolidJS app-shell"), and its surface-inventory table marks **Slice 1 as Done** (App root, ShellRail, ShellHeader, HeroPanel, DashboardIsland, MetricCard, ModuleCard, WorkflowCanvas-linear, node inspector). This is the WRONG migration direction under the amended intent.
- `crates/oya-application-shell-frontend-prototype/` is **real Leptos** (`leptos = "=0.8.19"`, `crate-type = ["cdylib","rlib"]`, features `csr`/`hydrate`/`ssr`) but is labeled a throwaway: its `Cargo.toml` description reads "Mock-only Leptos prototype scaffold for the Oyatie application shell control center," and it serves `server_mock_catalog` rather than live data.

So the production-bearing app is SolidJS while the labeled-as-mock prototype is the very Leptos stack the founder and the 2026-05-27 amendment designate as canonical. Leaving this unresolved means the IDP central hub (ADR-0394) would be built on the wrong shell.

### Why a clean superseding ADR (not another amendment)

ADR-0372's header/title still reads "SolidJS," it carries five SolidJS-committed deliverables (D1–D5), and its body still presents the SolidJS-canonical analysis as the decision with the Leptos retention bolted on as an amendment. Stacking a second amendment onto a document whose title, deliverables, and decision section all say SolidJS produces an unreadable governance record. A clean superseding ADR that states "Leptos canonical" as the headline decision, plus a bidirectional supersession marker on ADR-0372, is the honest record.

## Decision

1. **Leptos is the canonical app-shell / portal-shell frontend.** Full-stack Rust/WASM with **SSR + hydration** (`leptos` 0.8.x, `csr`/`hydrate`/`ssr` features as already present in `crates/oya-application-shell-frontend-prototype`). SolidJS is NOT a canonical target and is NOT retained as an evaluation track.

2. **Promote the Leptos prototype to the production portal-shell.** `crates/oya-application-shell-frontend-prototype` is re-designated from "throwaway mock" to the **production portal-shell**. Concretely:
   - Strip the "Mock-only … scaffold" framing from its `Cargo.toml` description and module docs; it is the production shell.
   - Replace `server_mock_catalog` with live calls to the IDP ops-BFF (ADR-0394) via the `render_envelope` SSR endpoint.
   - Bind the live `leptos_axum` SSR/server-fn host in the composition-root app crate (the Hyper/axum binding currently deferred), per the http-stack split below.

3. **Retire the SolidJS app-shell-frontend.** `microservices/app-shell-frontend/` (the SolidStart/pnpm/TS app) is **frozen and retired**:
   - `MIGRATION-PLAN.md` is **frozen** — no further Leptos→SolidJS slices are executed; the document is marked SUPERSEDED-BY-ADR-0393 and archived as historical.
   - The SolidJS slices already authored (Slice 1 surfaces) are **archived**, not extended. The directory is quarantined from the build/promotion path (no SolidJS app promotes past `dev`).
   - The actual file moves/deletions are an **implementation follow-up** (a separate code PR); this docs-only ADR records the decision and authorizes the retirement, it does not itself remove the SolidJS tree.

4. **`render_envelope` stays the SSR data contract.** The `GET /api/render-envelope/:context` Rust-struct contract (`crates/oya-application-shell-frontend-prototype/src/render_envelope.rs`) is retained as the canonical server→shell SSR data contract. The SolidJS `src/lib/render-envelope.ts` mirror is retired along with the SolidJS app; the Rust struct is the single source of truth.

5. **Rust→WASM compute islands are retained.** The compute-island pattern from ADR-0372 D3 (workflow canvas / Workflow Studio, virtualized data-grids with client-side sort/filter/aggregate, client-side crypto, heavy transforms) is RETAINED — now as native **Leptos islands** inside the Leptos shell rather than wasm-bindgen modules mounted into a TS shell. The shell is end-to-end Rust/WASM; there is no JS↔WASM bridge to maintain.

6. **http-stack discipline for the SSR host.** The Leptos SSR/server-fn host binds via **`leptos_axum`** on an axum router. Per `specs/http-stack-policy.json` (axum = sanctioned strategic exception requiring a recorded per-crate justification), the portal-shell app crate and the ops-BFF rest crate MUST register an `axum` justification in `justified_crates.axum` — same discipline as `oya-identity-workload-rest` — because the SSR + server-fn surface is a CRUD/extractor-heavy control-plane surface, not a latency-critical data path. Latency-critical panel feeds (SSE/WS) may use bare-hyper per the same policy. (Authoring those justifications is an implementation follow-up in the code PR.)

## Rejected alternatives

- **Keep SolidJS canonical (status quo of the code).** Rejected: contradicts the explicit founder directive and ADR-0372's own 2026-05-27 amendment. The performance/TTI rationale that originally favored SolidJS does not override the founder's one-Rust-toolchain directive for this surface; the IDP central hub is an internal operator console, not a massive-scale consumer surface, and the founder has not accepted the "evaluation track" hedge.
- **Keep both (SolidJS canonical + Leptos evaluation, or vice-versa).** Rejected: two app-shell stacks is exactly the drift this ADR closes; it doubles maintenance and leaves the canonical surface ambiguous.
- **Second amendment to ADR-0372.** Rejected: ADR-0372's title, five deliverables, and decision body all say SolidJS; a third revision on top makes the governance record unreadable. A clean supersede + bidirectional marker is the honest record.

## Consequences

### Positive
- One canonical frontend stack (Leptos/Rust/WASM), end-to-end Rust from shell to BFF to services; no JS↔WASM bridge, no OpenAPI→TS codegen step to keep green, no pnpm/Node toolchain in the frontend path (consistent with the container/hyperscaler-lens doctrine and ADR-0394's Node/React-forbidden stance).
- Unblocks ADR-0394 (bespoke-Rust IDP central hub) to build on the correct shell.
- The already-real Leptos prototype becomes the production shell rather than being discarded; the `render_envelope` contract and compute-island pattern survive intact.

### Negative / cost
- The SolidJS Slice-1 work in `microservices/app-shell-frontend/` is written off (archived). This is sunk cost; the alternative (continuing the wrong-direction migration) is worse.
- WASM's known costs return for the shell: larger cold-start floor (binary + wasm-bindgen glue + instantiation) and the WASM↔DOM/UTF-transcode tax that ADR-0372 documented. Accepted for an internal operator console where the one-Rust-toolchain + dogfooding value outweighs raw TTI; mitigated by SSR+hydration (server-rendered first paint) and by keeping heavy compute in purpose-built islands.
- Rust-frontend talent is scarcer than TS-frontend talent. Accepted as a deliberate doctrine cost.

### Neutral
- WASM stays in the codebase for its real strengths: server-side plugin sandboxing (ADR-0023 Wasmtime) and client-side compute islands — and now also the full shell render layer.

## Verification

- ADR-0372 carries the bidirectional supersession markers (`superseded_by: [ADR-0393]`, status `Superseded`) — see the companion edit in this PR.
- `crates/oya-application-shell-frontend-prototype` is no longer labeled "mock-only" and serves live BFF data via `render_envelope` (implementation follow-up; tracked, not in this docs-only PR).
- `microservices/app-shell-frontend/MIGRATION-PLAN.md` is frozen/marked superseded and the SolidJS tree is quarantined from the build/promotion path (implementation follow-up).
- The portal-shell app crate + ops-BFF rest crate register `axum` justifications in `specs/http-stack-policy.json#justified_crates.axum` (implementation follow-up).

### 2026-06-27 implementation follow-up

The ADR-0393 retirement follow-up removes the tracked SolidJS/SolidStart app-shell skeleton
and adds producer-visible CI guardrails so the retired stack cannot silently re-enter active
manifests:

- `marketplace/facade/dev-cli/tests/client_stack_discipline_cli.rs` is the crate-shaped
  client-stack CLI regression that proves the active Leptos app-shell manifest is scanned and
  that a SolidJS/SolidStart manifest fails closed.
- `marketplace/facade/dev-cli/tests/masterplan_cli.rs` is the masterplan projection regression
  that proves Superseded/Proposed SolidJS planning ADRs are excluded from live masterplan output
  while Accepted planning status variants remain included.
- `evidence/multispectrum/retire-solidjs-app-shell-20260627-1782545469.json` is the PR-local
  evidence record for the retirement, generated-artifact controls, rust-first accounting shrink,
  and client-stack gate verification.
- `evidence/multispectrum/solidjs-masterplan-projection-guard-20260627-1782566248.json` is the
  follow-up PR evidence record for the live-source masterplan projection guard that prevents
  superseded ADR-0372 planning text from re-entering controller-owned planning projections.

## References

- ADR-0372 — the superseded SolidJS frontend decision + its 2026-05-27 Leptos-retention amendment (this PR adds the `superseded_by` marker).
- ADR-0394 — bespoke-Rust IDP central hub (the portal this shell fronts).
- ADR-0090 — hyper canonical HTTP backbone (+ 2026-05-29 hyper/axum split).
- ADR-0509 — hyperscaler service decomposition (flat single-crate-per-service layout for the portal-shell crates).
- `crates/oya-application-shell-frontend-prototype/` — the Leptos 0.8.19 prototype promoted to production shell. The ADR-0562 capability-first reorg relocated it to `oya/application/crates/oya-application-shell-frontend/` (the sanctioned pre-move home for the `product-developer-application-shell` composition member); the paths below are that crate's current live artifacts.

### Implementation artifacts of the promoted shell

The native Axum/Tokio SSR host that replaced the crate's hand-rolled TCP listener and HTTP
parser, its bounded live-server regression suite, and the crate's ownership marker:

- `oya/application/crates/oya-application-shell-frontend/src/server.rs` — the Axum route graph and
  streaming-SSR host. It mounts explicit non-mutating reads only: no `/api/{*fn_name}`
  server-function route is registered, because the workspace declares zero `#[server]` functions
  and a wildcard POST over an empty registry is an unauthenticated control plane.
- `oya/application/crates/oya-application-shell-frontend/tests/live_server.rs` — the bounded
  live-TCP regression suite over that host, including the assertion that
  `POST /api/not-a-server-function` returns 404.
- `oya/application/crates/oya-application-shell-frontend/OWNERS` — the crate's ownership marker
  (ADR-0555 born-accounting).
- `microservices/app-shell-frontend/` + `MIGRATION-PLAN.md` — the SolidJS app retired by this ADR.
- `specs/http-stack-policy.json` — axum sanctioned-with-justification policy the SSR host obeys.
- `.omc/idp-central-hub-campaign.json` — the IDP campaign design that surfaced this drift.

## Historical residual from ADR-372 (E3 fold 2026-08-06)

**Title:** ADR-0372-frontend-stack-solidjs-ts-with-rust-wasm-compute-modules

**Preserved decision gist:** **TypeScript + SolidJS for the app-shell, hybrid with Rust→WASM for compute-only widgets:** 1. **Shell = SolidJS + SolidStart.** Fine-grained signals (the same reactive model as Leptos, so the smallest conceptual migration), top-tier near-vanilla runtime, ~7 KB core, low per-tab memory, excellent SSR/streaming (~2× Next on 10k-row dashboards). Svelte 5/SvelteKit is the equally-defensible alternative; React/Next only if ecosystem breadth ever outweighs efficiency. 2. **Type-safety = OpenAPI 3.2.0 → TS codegen** (orval / openapi-typescript) generated in CI from the Rust backend contracts → typed

_Source file archived after fold; full body in git history / docs/adr-archive/._
