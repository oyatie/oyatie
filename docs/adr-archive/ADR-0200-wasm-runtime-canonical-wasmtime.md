---
id: ADR-0200
status: Superseded
superseded_by: [ADR-709]
---

# ADR-0200 — WASM runtime canonical: Wasmtime

- Status: Accepted
- Date: 2026-05-18
- Deciders: Substrate architecture authority (oya-architecture-authority)
- Tags: substrate, runtime, sandboxing, extensibility, supply-chain
- Supersedes: none
- Superseded by: none
- Related: ADR-0147 (container sandboxing runtime ladder),
  ADR-0173 (vendor lock-in avoidance and stack ownership),
  ADR-0174 (FinOps cost attribution chargeback — quotas apply to
  WASM execution as well),
  ADR-0182 (north-south vs east-west separation — WASM is the
  preferred surface for Envoy filter logic),
  ADR-0183 (policy engine separation — WASM filters are NOT a
  policy engine; Cedar + Kyverno own that lane).

## Context

Three loosely coupled use cases inside the oyatie substrate need
sandboxed execution of untrusted-yet-deterministic bytecode at a
sub-process trust boundary:

1. **Envoy north-south WASM filters.** ADR-0182 places Coraza-class
   WAF, regulatory response shaping (ADR-0174), and bespoke
   authz/observability filters on the gateway data plane. Each
   filter must be hot-loadable per tenant pack without recompiling
   Envoy.
2. **Workflow Studio user-supplied node logic.** Per ADR-0185 the
   visual editor accepts user-authored expressions, mappers, and
   custom transformer nodes. These must run inside a capability-
   limited sandbox the tenant cannot escape, with deterministic
   CPU + memory ceilings the scheduler can enforce.
3. **Foundry tool execution sandbox.** ADR-0136 and the Foundry
   substrate execute LLM-driven tool calls. Tool implementations
   range from first-party Rust code (no sandbox required) to
   third-party / agent-supplied code (sandbox mandatory).

Each tier today either (a) hand-rolls a sub-process boundary, (b)
chooses an ad-hoc WASM engine, or (c) bypasses sandboxing entirely.
Without a canonical choice the substrate accumulates three runtimes
with three different threat models, three different observability
shapes, three different upgrade cadences, and three different
license profiles. The cohesion lane (ADR-0056 port-in-kernel) and
the supply-chain lane (license + provenance) both regress.

## Decision

The canonical WASM runtime for oyatie is **Wasmtime** — a
BytecodeAlliance project, CNCF graduated, run in production at
hyperscaler edge (notably Fastly Compute@Edge, whose engineering
team is a primary upstream contributor and whose deployment
profile is the reference operational model for our edge / Envoy
filter use case). Footnoted floor: Wasmtime 30.x LTS line as of
2026-05-18 — workspace `Cargo.toml` will pin the actual patch on
introduction; live-source version verification is owed to the
parent-wiring step (manifest delta) so the floor is not blindly
trusted.

### Scalability — concrete invariants per sandbox class

Per-class hard numbers (no aspirational; the kernel enforces):

| Class                  | Fuel ceiling | Memory cap | Wall-clock | Spawn p99 | Throughput target |
| ---------------------- | ------------ | ---------- | ---------- | --------- | ----------------- |
| `envoy-filter`         | 1,000,000    | 16 MiB     | 50 ms      | ≤ 2 ms    | 50k req/s/pod     |
| `workflow-studio-node` | 50,000,000   | 128 MiB    | 5 s        | ≤ 10 ms   | 5k req/s/pod      |
| `foundry-tool`         | 500,000,000  | 512 MiB    | 60 s       | ≤ 50 ms   | 200 req/s/pod     |

- **Instance pooling**: Wasmtime instances pool with size = 16 ×
  CPU cores per pod; instance reuse amortizes spawn cost.
- **Per-tenant fuel budget**: hard cap surfaced via
  `WasmRuntimeError::FuelExhausted`. ADR-0174 FinOps lane
  receives the metering.
- **Horizontal scale**: HPA tied to instance-pool utilization
  custom metric; no cluster reaches > 70% steady-state pool
  utilization at peak forecast.

### Maturity (doubt-driven check)

WASI Preview 2 + Component Model entered "stable" upstream
status in late 2023 (Wasmtime 14+) and has been the
production-grade ABI at Fastly Compute@Edge since 2024. Any
reviewer asking "is this mature?" should answer yes: it runs
public hyperscale workloads today.

### Security model — no ambient authority

The WASM sandbox follows the capability-based access model
(WASI Preview 2 Component Model). Concretely:

- No ambient filesystem, network, or clock access. Every host
  call is an explicit imported capability.
- Each sandbox class declares an `import allowlist`. Imports
  outside the allowlist link-fail at instantiation.
- Capability tokens are bound to the (tenant, sandbox-class,
  call-site) tuple and rotated per tenant key cycle.
- Memory + fuel ceilings are pre-instantiation invariants, not
  best-effort runtime checks.
- This is the Cloudflare Workers / Fastly Compute@Edge security
  posture; any reviewer from those teams should recognize it.

Three integration constraints follow:

1. **WASI Preview 2 + Component Model** is the canonical ABI. No
   `wasi_unstable` / `wasi_snapshot_preview1`-only modules accepted
   except via documented compatibility shim with a sunset date.
2. **Single substrate crate.** `crates/oya-shared-wasm-runtime-kernel`
   owns the `WasmRuntime` trait + real `WasmtimeRuntime` impl. No
   µservice imports `wasmtime` directly. Discipline is enforced by
   `crates/oya-check-wasm-runtime-discipline`.
3. **Sandbox class registry.** Each WASM call site declares a
   sandbox class (`envoy-filter`, `workflow-studio-node`,
   `foundry-tool`). Each class has a fixed fuel budget, memory
   ceiling, and import allowlist. Classes are versioned and
   reviewed at ADR cadence.

## Alternatives considered

- **Wasmer** — Commercial-leaning license clauses (some surfaces
  shifted to a non-OSI variant), smaller plugin ecosystem,
  weaker upstream alignment with WASI Preview 2 + Component
  Model as of 2026-05-18. Rejected.
- **wasmCloud** — Opinionated service-mesh-shaped runtime; the
  cohesion + multi-cluster federation lanes (ADR-0171) and the
  north/south vs east/west boundary (ADR-0182) are already owned
  by other substrates. wasmCloud overlaps and rebuilds those
  layers. Rejected.
- **V8 isolates** — Browser-rooted engine; embedding outside Node /
  Chromium incurs an opaque maintenance surface and a JS-only
  authoring model. Rejected.
- **Lucet** — Upstream effectively deprecated and folded into
  Wasmtime since 2021. Rejected.
- **Native sub-process per tenant** — Already the fallback when
  WASM cannot host a workload (e.g. ADR-0147 container
  sandboxing). Not a substitute for in-process bytecode
  sandboxing; the two coexist.

## Consequences

- Every WASM-touching µservice consumes the kernel crate and
  declares a sandbox class. Direct `wasmtime` (or `wasmer`,
  `wasmedge`) dependency outside the kernel becomes a CI
  violation via `oya-check-wasm-runtime-discipline`.
- Envoy filter authors target the canonical sandbox class
  `envoy-filter` and inherit fuel + memory ceilings. Tenant
  packs (ADR-0064) ship filter bundles via the standard
  pack-overlay path.
- Workflow Studio gains a uniform secure-execution surface that
  the perf budget lane (cross-microservice latency) can model.
- Foundry tools authored outside the trusted set execute inside
  the `foundry-tool` sandbox class; existing first-party tools
  may opt in for defense-in-depth.
- License + provenance: Wasmtime is Apache-2.0 (with LLVM
  exception), satisfying ADR-0173 vendor lock-in avoidance.
- Memory accounting flows into the FinOps lane (ADR-0174) so
  tenant chargeback reflects WASM execution cost.

## Standards anchor

- `docs/standards/wasm-runtime-canonical.md` — when to reach for
  WASM vs sub-process vs container vs gVisor (ADR-0147).
- `crates/oya-shared-wasm-runtime-kernel/src/lib.rs` — trait
  surface + sandbox-class enum.
- `crates/oya-check-wasm-runtime-discipline/src/lib.rs` —
  advisory gate.

## Migration

- T+0 (this ADR): ADR + kernel crate + check crate land.
- T+30d: All existing WASM usage routes through the kernel.
- T+60d: `oya-check-wasm-runtime-discipline` is a BLOCKER lane on
  µservice promotion to staging.

## In-house roadmap

- **Phase 0 (today)**: Adopt **Wasmtime** as the canonical runtime.
  Wasmtime *is* the community standard — BytecodeAlliance project,
  CNCF graduated, Apache-2.0 + LLVM exception, run in production
  by Fastly Compute@Edge, Shopify, and Microsoft Azure (component
  model is jointly developed). Adopting Wasmtime is itself the
  "in-house" posture because the substrate that hyperscalers
  contribute to and run in production is exactly the substrate
  oyatie inherits.
- **Phase 2 (deferred, conditional)**: No in-house WASM engine is
  planned. Building a separate runtime would re-invent what
  Wasmtime already provides, fragment the ecosystem, and break
  ADR-0173 (vendor lock-in avoidance) by introducing Oya-exclusive
  lock-in. The "in-house" path here is to **own the integration
  layer** (sandbox classes, capability tokens, observability
  emission) rather than the runtime itself.
- **In-house contribution path**: oyatie engineers contribute
  upstream to Wasmtime + the WASI Preview 2 Component Model
  working group when fixes / features land in our adapter that
  belong upstream. ADR-0173 contribution-back policy applies.

## Open questions

- Should `foundry-tool` sandbox class allow outbound HTTP via the
  capability registry, or only synchronous returns? Deferred to a
  follow-up ADR after Workflow Studio integration sign-off.
