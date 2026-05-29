---
id: ADR-0372
status: Superseded
deciders: founder, council-architecture
date: 2026-05-26
owner: council-architecture
supersedes: []
superseded_by: [ADR-0393]
related: [ADR-0001, ADR-0013, ADR-0023, ADR-0393]
planning_impact: true
milestone: M-APP-SHELL-FRONTEND
depends_on: []
door: two-way
affected_surfaces:
  crates: [oya-application-shell-frontend-prototype]
  microservices: []
  specs: [/specs/platform-architecture.json]
deliverables:
  - id: ADR-0372-D1
    description: "App-shell frontend on TypeScript + SolidJS (+ SolidStart for SSR/streaming) — fine-grained reactivity (the Leptos mental model) without the WASM DOM-boundary tax; near-vanilla runtime, lean payload + per-tab memory."
    exit_criteria: "the app-shell renders on SolidJS; krausest-tier runtime + cold-start TTI beat the Leptos prototype on the operator-console surfaces."
    verified_by: "frontend build + Lighthouse/TTI budget check in CI"
  - id: ADR-0372-D2
    description: "End-to-end type safety via OpenAPI 3.2.0 → TS codegen (orval / openapi-typescript) generated in CI from the Rust backend contracts — typed client + framework query hooks. (tRPC/oRPC rejected: they require a TS backend; ours is Rust.)"
    exit_criteria: "the TS client + hooks are generated from the canonical OpenAPI 3.2.0 contracts in CI; a contract change breaks the build if the client isn't regenerated."
    verified_by: "oya gate validate contract-client-codegen (to author) + CI codegen step"
  - id: ADR-0372-D3
    description: "Hybrid: Rust→WASM modules ONLY for genuinely compute-bound, DOM-light widgets (virtualized data-grids with client-side sort/filter/aggregate, canvas/diagram/workflow editors, client-side crypto, heavy transforms), mounted into the TS shell via wasm-bindgen, sharing Rust types + the same OpenAPI contracts."
    exit_criteria: "at least the heaviest compute widget (e.g. Workflow Studio canvas or a large data-grid) runs as a Rust→WASM module inside the SolidJS shell; the 95% plain-DOM shell pays zero WASM tax."
    verified_by: "the WASM compute module mounts + passes its widget tests inside the TS shell"
  - id: ADR-0372-D4
    description: "WITHDRAWN by the 2026-05-27 amendment. The Leptos prototype (crates/oya-application-shell-frontend-prototype) is RETAINED as the canonical app-shell frontend; it is NOT retired. The SolidJS shell is an evaluation track only — Leptos is replaced as canonical ONLY if SolidJS demonstrates superiority at massive scale."
    exit_criteria: "Leptos remains the canonical app-shell; the SolidJS scaffold exists as an evaluation, not a migration commitment; no Leptos retirement occurs absent proven massive-scale SolidJS superiority."
    verified_by: "the Leptos crate remains present + canonical; SolidJS is evaluation-only"
  - id: ADR-0372-D5
    description: "Frontend stays OSI-clean + hyperscaler-norm: SolidJS (MIT), SolidStart (MIT); no forbidden-license frontend deps; WASM remains the server-side-sandbox + client-compute-module pattern (ADR-0023 Wasmtime is server-side, NOT the frontend rendering layer)."
    exit_criteria: "frontend dependency licenses pass the policy gate; WASM usage is compute-modules + server-sandbox only."
    verified_by: "license/dependency policy gate green on the frontend"
purpose: Choose the app-shell FRONTEND stack on performance, scalability, resource-efficiency, and hyperscaler norms (not language ideology). Original decision — TypeScript + SolidJS (fine-grained reactivity, SolidStart SSR) for the DOM-heavy shell, with targeted Rust→WASM modules only for compute-bound widgets, and OpenAPI 3.2.0→TS codegen for end-to-end type safety. Backend/services/gateway stay Rust (settled, out of scope). AMENDED 2026-05-27 — the supersede-Leptos stance is REVERSED: Leptos is RETAINED as the canonical app-shell frontend; SolidJS is an evaluation track only, adopted as canonical solely on proven massive-scale superiority.
---

# ADR-0372: Frontend stack — SolidJS/TS app-shell + Rust→WASM compute modules

## Status
**SUPERSEDED by ADR-0393 (2026-05-29).** Leptos (Rust/WASM, SSR+hydration) is the canonical app-shell frontend; SolidJS is not a canonical target. The body below is retained as the historical record of the original SolidJS decision and its 2026-05-27 Leptos-retention amendment. See ADR-0393 for the clean Leptos-canonical decision and the reconciliation of the codebase drift.

~~Accepted — 2026-05-26. **AMENDED 2026-05-27** (see Amendment): the supersede-Leptos stance is reversed — Leptos retained canonical, SolidJS evaluation-only.~~

## Amendment (2026-05-27)
**Founder decision (2026-05-27) REVERSES the supersede-Leptos stance of this ADR.** Leptos (`crates/oya-application-shell-frontend-prototype`, Rust→WASM) is RETAINED as the **canonical** app-shell frontend. SolidJS is retained as an **evaluation track only** — the `microservices/app-shell-frontend` SolidJS scaffold (PR #201) is that evaluation, NOT a migration commitment. SolidJS replaces Leptos as canonical ONLY if it demonstrates superiority **at massive scale**; absent that proof, Leptos stays. Deliverable D4 (retire the Leptos prototype) is WITHDRAWN; D1–D3 stand as EVALUATION work, not a committed migration. The analysis below is retained as the historical rationale for the (now-reversed) decision; the backend-stays-Rust and WASM-for-compute conclusions are unaffected.

## Context
The app-shell prototype is **Leptos** (Rust→WASM; `crates/oya-application-shell-frontend-prototype`,
~9.6k lines of Rust + a 4.3k-line `static/prototype-interactions.js`). The founder's standing criterion:
*"If TS is more performant, scalable, resource-efficient than Rust, we are open to it"* — **frontend
only**; backend stays Rust. Full analysis: `.omx/plans/frontend-stack-decision.md`.

The evidence is decisive for a **DOM-heavy dashboard/app-shell**: Rust→WASM does **not** win the metrics
that matter. WASM has no direct DOM access — every DOM mutation crosses a JS bridge, and every string
crosses a Rust(UTF-8)↔JS(UTF-16) transcode. The Leptos author concedes (discussion #2627) that a
comparable JS framework is "marginally faster… almost exclusively due to… transcoding UTF-8 to UTF-16."
On the krausest benchmark, vanilla ≈ Solid > Svelte > Leptos > Vue > React. WASM also ships the worst
cold-start floor (binary + wasm-bindgen glue + instantiation → TTI regression). Leptos's ecosystem is
thin (many single-maintainer libs) and Rust-frontend hiring is scarce. **WASM's genuine win is
compute, not DOM** — and hyperscalers ship TS shells while reserving WASM for compute modules.

## Decision
**TypeScript + SolidJS for the app-shell, hybrid with Rust→WASM for compute-only widgets:**

1. **Shell = SolidJS + SolidStart.** Fine-grained signals (the same reactive model as Leptos, so the
   smallest conceptual migration), top-tier near-vanilla runtime, ~7 KB core, low per-tab memory,
   excellent SSR/streaming (~2× Next on 10k-row dashboards). Svelte 5/SvelteKit is the
   equally-defensible alternative; React/Next only if ecosystem breadth ever outweighs efficiency.
2. **Type-safety = OpenAPI 3.2.0 → TS codegen** (orval / openapi-typescript) generated in CI from the
   Rust backend contracts → typed client + Solid query hooks. (tRPC/oRPC rejected — they need a TS
   backend.) This buys the entire JS ecosystem + far better TTI/memory at a one-time CI codegen cost;
   the only thing given up vs Leptos is "free" shared Rust types, which doesn't justify the deficit.
3. **Hybrid: Rust→WASM modules only where compute dominates** — virtualized data-grids, canvas/diagram/
   workflow editors, client-side crypto, heavy transforms — mounted into the TS shell via wasm-bindgen,
   sharing Rust types + OpenAPI contracts. Pay the WASM cost exactly where it pays; zero WASM tax on the
   plain-DOM 95% of the shell.
4. **Backend unchanged: Rust.** This ADR is frontend-only.

## Rejected alternatives
- **Leptos / Rust→WASM for the whole shell** — rejected: WASM DOM-boundary + UTF transcode tax (author
  concedes JS edges it out), worst cold-start TTI, thin single-maintainer ecosystem, scarce
  Rust-frontend hiring. Kept ONLY-IF a hard org mandate forced one Rust toolchain across the *entire*
  stack AND frontend velocity were deprioritized — neither holds.
- **React 19 / Next.js** — largest ecosystem but heaviest payload/memory and slowest runtime of the
  four; choose only if hiring/ecosystem breadth is the overriding constraint.
- **tRPC / oRPC for type-safety** — rejected: require a TypeScript backend; ours is Rust → OpenAPI
  codegen is the correct polyglot path.

## Consequences
- Positive: best runtime/TTI/memory for a DOM-heavy operator console, the universal hyperscaler frontend
  norm, a 10× larger talent pool, and Rust kept exactly where it wins (backend + compute WASM modules).
- Negative/cost: a frontend migration off the Leptos prototype (mitigated — meaningful UI logic already
  lives in the 4.3k-line JS file, so the pure-Rust-frontend premise isn't currently honored); a CI
  OpenAPI→TS codegen step; loss of "free" Rust-shared types (replaced by generated typed clients).
- Neutral: WASM stays in the codebase for its real strengths — server-side plugin sandboxing (ADR-0023
  Wasmtime/WASI) and client-side compute modules — neither of which is the frontend rendering layer.

## Verification
Per-deliverable `verified_by`. The decision is met when the app-shell renders on SolidJS with the
generated OpenAPI→TS typed client, beats the Leptos prototype on TTI/runtime/memory budgets, the heaviest
compute widget runs as a mounted Rust→WASM module, and the Leptos prototype is retired — with frontend
dependency licenses passing the policy gate. Per ADR-0368 D6 this choice was made against the
hyperscaler best-practice bar (DOM-heavy shells ship TS; WASM is for compute), not language preference.

## References
ADR-0001 (one-product cohesion), ADR-0013 (OSI license policy), ADR-0023 (Wasmtime sandbox — server-side,
not frontend). Analysis + sources: `.omx/plans/frontend-stack-decision.md` (krausest benchmark, Leptos
discussion #2627, ACM Queue "WASM & DOM", SolidStart/SvelteKit/Next 2025 benchmarks, orval/OpenAPI codegen).
Canonical contract: OpenAPI 3.2.0.
