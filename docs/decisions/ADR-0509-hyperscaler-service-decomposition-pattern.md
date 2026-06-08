---
id: ADR-0509
title: "Hyperscaler service decomposition pattern (single-crate-per-service + mod-based subsystems)"
status: Superseded
date: 2026-05-28
authority: founder
owner: council-architecture
planning_impact: true
supersedes: []
superseded_by: [ADR-0512]
supersession_note: "ADR-0512 supersedes this; status was drifted (0512 named it as superseded but 0509 had no edge). D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-3/FC-5."
related: [ADR-0392, ADR-0476, ADR-0478, ADR-0479, ADR-0480, ADR-0481]
---

# ADR-0509 — Hyperscaler service decomposition pattern (single-crate-per-service + mod-based subsystems)

## Status

Accepted — 2026-05-28.

This ADR supersedes the **per-use-case clean-architecture convention** established by PR #289
(`microservices/intelligence/crates/oya-intelligence-*`). That convention is hereby legacy.

## Context

### The PR #289 pattern and its costs

PR #289 ("Align flat service taxonomy and harden release gates") established `microservices/intelligence/` as the
reference implementation for intelligence workloads. The structure it introduces is a **per-use-case
7-crate clean-architecture slice**:

```
microservices/intelligence/crates/
  oya-intelligence-<usecase>-kernel/      # domain entities + port traits
  oya-intelligence-<usecase>-domain/      # domain logic
  oya-intelligence-<usecase>-usecase/     # application use-case orchestration
  oya-intelligence-<usecase>-api/         # API DTOs
  oya-intelligence-<usecase>-adapter-*/   # adapter impls (postgres, redis, …)
  oya-intelligence-<usecase>-rest/        # REST handler
  oya-intelligence-<usecase>-grpc/        # gRPC handler
  oya-intelligence-<usecase>-app/         # binary wiring
```

With 5–10 use cases per service and ~50 planned services, this pattern projects to **1 500–3 000 crates**
in the Cargo workspace. Even at today's baseline the intelligence service alone touches ~50 crates.

Project costs:

| Cost | Impact |
|---|---|
| Cargo build-time | Every crate is an independent compilation unit; incremental rebuilds touch 5–10 crates per use-case change |
| Cross-crate refactor friction | Renaming a domain type requires coordinated version bumps across kernel → domain → usecase → api → adapter → rest |
| Artificial boundaries | Per-use-case crate boundaries do NOT map to team or deployment boundaries; they are a pattern artefact |
| Workspace topology | `Cargo.toml` workspace member list grows by 7–8 entries per use case |
| IDE / toolchain overhead | rust-analyzer, cargo-check, cargo-clippy all pay per-crate overhead |

### Honest hyperscaler reference-point survey

Founder direction 2026-05-28: "Is this the correct pattern for hyperscaler?" A survey of how FAANG
hyperscalers and leading Rust production services actually structure internal services:

**Google (Borg / Spanner / Colossus):**
One Bazel `cc_binary` / `go_binary` target per service binary. Internal libraries are subsystem-scoped
(storage, RPC, auth, observability) — NOT per-use-case. Protos are the SSOT for all interfaces;
in-process callers share the same generated types. No per-use-case adapter crate sprawl.

**Meta (TAO / Scuba / Everstore):**
Buck2 single-target service binary per service. Subsystem modules inside the binary crate. Use cases
map to RPC handler functions, not to separate compilation artifacts.

**Amazon (S3 / DynamoDB cell services):**
One service ≈ one repo/subtree, one binary. Subsystem decomposition (partition management, replication,
admission) as internal modules. Adapter indirection exists only where multiple genuine backends coexist
in production (e.g., Paxos log AND local NVMe path).

**Stripe Pay Server:**
Famously kept monolithic for the payment surface. Domain organisation (subscriptions, invoices, payments)
is by subsystem, not by per-use-case compile boundary.

**Tailscale / Vector / Pulumi (production Rust at scale):**
All three are single-crate-per-service (or a small set of library crates shared across services).
Per-use-case crate sprawl does not appear in any of them.

The conclusion is unambiguous: **the per-use-case 7-crate clean-architecture pattern has no hyperscaler
internal equivalent**. It is an academic pattern optimised for maximising interface isolation, not for
maximising developer velocity, build speed, or operational simplicity at scale.

## Decision

### 1. Single-crate-per-service is the canonical decomposition

`microservices/<service>/Cargo.toml` is ONE crate. `src/main.rs` compiles ONE binary per service.
There is no per-use-case compile boundary inside a service.

### 2. Subsystem mods decompose by SUBSYSTEM, not by use case

Under `src/`, directories represent subsystems (e.g., `auth/`, `oidc/`, `webauthn/`, `realms/`,
`users/`, `storage/`, `rest/`, `grpc/`, `observability/`). A subsystem may contain multiple use cases
as functions or sub-modules; the subsystem boundary is the architectural seam, not the use case.

### 3. Proto / OpenAPI 3.2.0 / AsyncAPI 3.1.0 are the SSOT for interface contracts

No internal API DTO crates for in-process callers. Generated proto/OpenAPI types are used directly
inside the service. External callers get the proto or OpenAPI contract; internal callers share the
same generated types without an extra abstraction layer.

### 4. No adapter indirection unless multiple genuine backends ship today

Single-backend storage is direct DB access (e.g., `sqlx` calls in `storage/mod.rs`). Adapter trait
indirection is introduced ONLY when 2+ genuine backend implementations exist in production simultaneously
(e.g., HTTP ingest and Pulsar ingest for the same surface). Speculative abstraction is rejected.

### 5. Workspace-level `pub trait` for genuinely-swappable seams — ONE impl crate per seam

If a storage backend or message-bus seam needs to be swappable and 2+ impls exist today, a
workspace-level trait crate (e.g., `crates/oya-storage-port`) is acceptable with ONE impl crate per
backend (not one per use case). This is the only approved deviation from single-crate-per-service.

### 6. Canonical service layout

Every new service MUST follow this layout:

```
microservices/<service>/
  Cargo.toml              # ONE crate per service; name = "oya-<service>"
  proto/                  # SSOT interfaces (proto3 / OpenAPI 3.2.0 / AsyncAPI 3.1.0)
  src/
    main.rs               # binary entry point; wires subsystems
    lib.rs                # crate root; re-exports public surface
    config.rs             # config parsing and validation (clap + serde)
    <subsystem-a>/        # pub mod; domain subsystem (e.g., auth, oidc, realms)
      mod.rs
      *.rs
    <subsystem-b>/
    storage/              # pub mod; persistence (direct DB access)
    rest/                 # pub mod; REST handlers (axum)
    grpc/                 # pub mod; gRPC handlers (tonic)
    observability/        # pub mod; tracing + metrics (OTel)
  catalog.yaml            # role / context / data classes / SLOs
  slos/                   # OpenSLO specs (mandatory before promoting past dev)
  README.md
  BUCK                    # Buck2 binary target per ADR-0392
```

## Hyperscaler-lens pre-check

Per [[hyperscaler-lens-architectural-filter]]: every structural pattern must pass the lens before
adoption. Both patterns evaluated:

| Pattern | FAANG-internal equivalent | Cargo build tax | Refactor friction | Verdict |
|---|---|---|---|---|
| Per-use-case 7-crate (legacy `intelligence/`) | NONE | Massive (~30 crates/service) | High (cross-crate version dances) | **REJECT** |
| Single-crate-per-service + mod subsystems | Google/Meta/Stripe/Amazon canonical | Minimal (1 crate/service) | Low (intra-crate refactor) | **ADOPT** (this ADR) |

The per-use-case pattern fails the hyperscaler-lens on two axes: no FAANG internal equivalent, and
no active upstream endorsement in production Rust services at scale.

## Supersession — PR #289 convention

The per-use-case 7-crate clean-architecture convention established by PR #289 for
`microservices/intelligence/crates/oya-intelligence-*` is **superseded** by this ADR as of 2026-05-28.

`microservices/intelligence/` is the **legacy pattern**. It remains in the repository pending migration
but MUST NOT be used as a scaffold reference for new services or new use cases within intelligence.

Existing 3-crate scaffolds (oya-identity with kernel/rest/app, oya-billing, oya-meter, oya-cost,
oya-flags) → collapse to single-crate-per-service as part of the rework wave (Phase 1 below).

## Migration roadmap

### Phase 1 — Collapse existing 5 scaffolds (this wave)

Rework the five existing 3-crate scaffolds to single-crate-per-service. Each becomes one PR:

| Service | Current layout | Target layout |
|---|---|---|
| `oya-identity` | kernel / rest / app | single crate + subsystem mods |
| `oya-billing` | kernel / rest / app | single crate + subsystem mods |
| `oya-meter` | kernel / rest / app | single crate + subsystem mods |
| `oya-cost` | kernel / rest / app | single crate + subsystem mods |
| `oya-flags` | kernel / rest / app | single crate + subsystem mods |

Each Phase 1 PR amends the corresponding ADR (ADR-0476 through ADR-0481) to reflect the updated layout.

### Phase 2 — `microservices/intelligence/` rework (follow-up IP)

Author `IP-INTEL-MIGRATE-CANONICAL` for the full `microservices/intelligence/` rework:
~50 per-use-case crates → single crate with subsystem mods. Heavy lift; may stretch across multiple
PRs partitioned by subsystem boundary (e.g., ingest subsystem PR, routing subsystem PR, etc.).
Intelligence remains on the legacy layout until each subsystem migration PR merges.

### Phase 3 — New service scaffolds (immediate)

All new services scaffolded from this date forward use the canonical single-crate-per-service layout.
No per-use-case clean-architecture slice is acceptable in any new service PR.

## Consequences

### Positive

- One `Cargo.toml` per service — workspace topology is dramatically simpler
- Faster builds: less cargo metadata overhead, smaller incremental rebuild surface (one crate per
  service vs. 7–30)
- Easier refactoring: intra-crate moves are zero-cost (no version bump required)
- Pattern matches what FAANG hyperscalers actually do; onboarding engineers see a familiar structure
- Buck2 single binary target per service maps cleanly to the single-crate layout (ADR-0392)

### Negative / mitigations

- Loses strict compile-time isolation of use-case boundaries from the per-use-case pattern; mitigated
  by tight `pub` surface control on each subsystem `mod` and `pub trait` seams where genuine
  swappability is needed
- PR #289's intelligence rework is now legacy; ~50 crates are flagged for migration — this is rework
  cost that must be planned and budgeted
- Subsystem mods require disciplined code review to prevent subsystem boundary erosion; mitigated by
  lint rules and PR review gates

## Related

- **ADR-0392** — Buck2 canonical build system; single-crate-per-service maps to one `rust_binary`
  Buck2 target per service
- **ADR-0476** — oya-identity service; Phase 1 migration target
- **ADR-0478** — oya-billing service; Phase 1 migration target
- **ADR-0479** — oya-meter service; Phase 1 migration target
- **ADR-0480** — oya-cost service; Phase 1 migration target
- **ADR-0481** — oya-flags service; Phase 1 migration target
- **[[hyperscaler-service-pattern]]** — doctrine memory (canonical reference)
- **[[bespoke-over-oss-doctrine]]** — bespoke ADRs still require feature-parity tables; this ADR
  governs HOW the bespoke crate is internally structured
- **[[hyperscaler-lens-architectural-filter]]** — every new component pre-checks the lens; this ADR
  governs the internal decomposition pattern that must pass the lens
- **[[oyatie-flat-no-grouping-doctrine]]** — flat µservices; no grouping or suite patterns; this ADR
  extends the doctrine to intra-service decomposition
