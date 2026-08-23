---
doc_class: Template
template_id: TPL-MS
status: Accepted
date: 2026-05-13
purpose: |
  Canonical skeleton for adding a new µservice to the oyatie workspace.
  Covers naming justification (BNF v4.1), Cargo workspace entries, layer
  skeleton, BC list, [package.metadata.oya] schema, test scaffolding, and
  µservice without escalation.
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - docs/templates/prd-template.md
  - docs/templates/bounded-context-registration-template.md
  - docs/standards/bounded-contexts.md
  - docs/templates/INDEX.md
adrs_cited:
  - ADR-0056  # BNF v4.1 + 12-enum layer
  - ADR-0057  # LEAN checks
  - ADR-0054  # scaffold-claim pattern
doc_status: published
---

# New µservice scaffold: `oyatie-<µservice-name>-*`

## Name Justification (BNF v4.1)

Mandatory for every new artifact name introduced by this scaffold
(per `feedback_naming_justification.md`). Complete one block per crate family.

```
MICROSERVICE: <µservice-name>
JUSTIFICATION:
- microservice = <kebab-token(s)>: <product/capability name; must be registered
  in [workspace.metadata.oyatie.microservices] in root Cargo.toml; cite ADR-0056
  v4.1 flat BNF — no shared|vertical bisection; every µservice is equal in
  the flat catalog>
- Glossary check:
    "shared" not "platform" ✓ (platform retired per feedback_glossary_shared_not_platform.md)
    "Ontology" not "Object Graph" ✓
    "Application" not "Shell" ✓
    flat catalog; no "Arm" / "Product Group" ✓
```

Per-crate justification blocks (fill one per layer crate you scaffold):

```
NAME: oyatie-<µservice>[-<bc-tokens>]-<layer>
JUSTIFICATION:
- microservice = <µservice-name>: <rationale>
- bc-tokens = <bc-name> (OPTIONAL): <include if multiple BCs or binaries;
  omit if µservice has a single concept at this layer; ADR-0056 v4.1>
- layer = <layer>: <12-enum value; ADR-0056 §"Layer semantics":
    domain          — pure business logic; port-traits + entities; no I/O
    usecase         — use-case orchestrators; holds port-trait bounds
    infrastructure  — framework/driver implementations of port-traits
    kernel          — shared types + value objects (cross-layer)
    adapter         — adapter-layer glue (e.g. event-bus bridge)
    rest            — HTTP/REST handler wiring
    grpc            — gRPC handler wiring
    worker          — background job / queue consumer runner
    cli             — command-line binary
    sdk             — generated/contract-bound client surface
    api             — protocol-neutral API contract surface
    app             — composition-root binary (main.rs)
- exemptions: <none | cite ADR-0056 exception>
```

---

## Cargo Workspace Member Entries

Add to root `Cargo.toml` `[workspace.members]`:

```toml
[workspace.members]
# ... existing members ...
"crates/oyatie-<µservice>-<bc>-domain",
"crates/oyatie-<µservice>-<bc>-usecase",
"crates/oyatie-<µservice>-<bc>-infrastructure",
"crates/oyatie-<µservice>-<bc>-rest",        # if REST surface needed
"crates/oyatie-<µservice>-<bc>-grpc",        # if gRPC surface needed
"crates/oyatie-<µservice>-<bc>-worker",      # if background workers needed
"crates/oyatie-<µservice>-<bc>-app",         # composition-root binary

[workspace.metadata.oyatie.microservices]
<µservice-name> = { prd = "docs/prds/<µservice-name>.md", milestone_first_ship = "M0X" }
```

---

## Clean Architecture Layer Map

Full 12-layer enum reference for this µservice. Ship only layers the µservice
needs; justify omissions. Dependency direction is strictly inward-only
(per `feedback_clean_architecture_requirements.md` §2):

```
{cli, rest, grpc, worker, sdk, api}   ← presentation / contract surfaces
         ↑ depends on
    {adapter, infrastructure}         ← outer adapters
         ↑ depends on
       usecase                        ← use cases
         ↑ depends on
         domain                       ← business logic
         ↑ depends on
         kernel                       ← pure types + port traits  ← sdk
         ↑
         app (composition root; unrestricted inward; wiring only)
```

| Layer | Crate name (BNF v4.1) | Ship? | Reason if omitted |
|---|---|---|---|
| `kernel` | `oyatie-<ms>[-<bc>]-kernel` | Yes (always) | — |
| `domain` | `oyatie-<ms>[-<bc>]-domain` | Yes (always) | — |
| `usecase` | `oyatie-<ms>[-<bc>]-usecase` | Yes (always) | — |
| `adapter` | `oyatie-<ms>[-<bc>]-adapter` | Yes (if state) | Omit for stateless-only µservices |
| `infrastructure` | `oyatie-<ms>[-<bc>]-infrastructure` | Conditional | Use adapter instead where possible |
| `rest` | `oyatie-<ms>[-<bc>]-rest` | Conditional | Omit if gRPC-only surface |
| `grpc` | `oyatie-<ms>[-<bc>]-grpc` | Conditional | Omit if REST-only surface |
| `worker` | `oyatie-<ms>[-<bc>]-worker` | Conditional | Required if background jobs exist |
| `cli` | `oyatie-<ms>[-<bc>]-cli` | Conditional | Internal tooling µservices only |
| `sdk` | `oyatie-<ms>[-<bc>]-sdk` | Conditional | External client consumers only |
| `api` | `oyatie-<ms>[-<bc>]-api` | Conditional | Protocol-neutral contract surface |
| `app` | `oyatie-<ms>-app` | Yes (always) | Composition root binary |

**Port trait rule**: ALL port traits live in `kernel`. Never in `domain`.
`domain` calls *through* port traits; it does not declare them.

```rust
// CORRECT — oyatie-<ms>-<bc>-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait <RepositoryPort>: Send + Sync + sealed::Sealed {
    async fn find_by_id(&self, id: &<IdType>) -> Result<<Entity>, <Err>>;
    async fn save(&self, entity: &<Entity>) -> Result<(), <Err>>;
}

// WRONG — never put port traits in domain or application
```

Implementations live in `oyatie-<ms>-<bc>-adapter`. Domain imports `kernel`
only; never imports `adapter`. `app` imports everything to wire.

---

## Layer Skeleton

Ship only the layers that the µservice actually needs. Justify omissions.
Standard order: `domain` → `application` → `infrastructure` → `{rest|grpc}` → `worker` → `app`.

### `crates/oyatie-<µservice>-<bc>-domain/`

```
Cargo.toml
src/
  lib.rs          — pub mod declarations; pub use surface
  <entity>.rs     — entity struct + invariants
  <repo_trait>.rs — repository port-trait (async_trait)
  error.rs        — domain error enum
```

`Cargo.toml` dependencies: `async-trait`, `serde`, `thiserror`; NO framework crates.

### `crates/oyatie-<µservice>-<bc>-application/`

```
Cargo.toml
src/
  lib.rs
  <use_case>.rs   — use-case struct holding port-trait generics
  commands.rs     — command types (CQRS command side)
  queries.rs      — query types (CQRS query side)
```

`Cargo.toml` dependencies: domain crate; `async-trait`; NO infrastructure crates.

### `crates/oyatie-<µservice>-<bc>-infrastructure/`

```
Cargo.toml
src/
  lib.rs
  db/
    <repo_impl>.rs   — repository trait implementation (sqlx / diesel)
    migrations/      — SQL migration files
  event/
    <publisher>.rs   — outbox event publisher
```

`Cargo.toml` dependencies: domain crate; `sqlx` or `diesel`; `tokio`.

### `crates/oyatie-<µservice>-<bc>-rest/` (omit if REST not needed)

```
Cargo.toml
src/
  lib.rs
  routes.rs      — axum Router assembly
  handlers/
    <resource>.rs  — handler fns; maps HTTP → application commands/queries
  dto/
    <resource>.rs  — request/response DTOs (serde)
```

### `crates/oyatie-<µservice>-<bc>-worker/` (omit if no background workers)

```
Cargo.toml
src/
  lib.rs
  <worker_name>.rs  — tokio task; consumes Workflow events or outbox
```

Layer note: worker crates MUST be stateless (no module-level mutable state);
`check-statelessness-cli` CI lane enforces this.

### `crates/oyatie-<µservice>-<bc>-app/`

```
Cargo.toml
src/
  main.rs   — composition root; wires infrastructure impls into application ports;
              starts HTTP server; starts workers; runs migrations
```

---

## BC List

Register each BC in `docs/standards/bounded-contexts.md` using
`docs/templates/bounded-context-registration-template.md`.

| BC name (kebab) | Crate family | Key entities | Workflow events | Ontology types |
|---|---|---|---|---|
| `<bc-name>` | `oyatie-<ms>-<bc>-{domain,application,...}` | `<Entity>` | `<EventType>` | `<ObjectType>` |

---

## `[package.metadata.oya]` Block Schema

Every crate in the µservice must carry this block in its `Cargo.toml`:

```toml
[package.metadata.oya]
microservice   = "<µservice-name>"
bc             = "<bc-name>"           # omit if no BC slot
layer          = "<layer-enum-value>"  # one of 12 canonical values
prd            = "docs/prds/<µservice-name>.md"
milestone      = "M0X"
active_active_compatibility = "stateless-compatible | single-writer-compatible"
# ↑ required per feedback_quality_performance_scalability_bar.md
```

---

## Test Scaffolding

```
crates/oyatie-<µservice>-<bc>-domain/src/<entity>.rs
  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn test_<entity>_<scenario>() { ... }
  }

crates/oyatie-<µservice>-<bc>-application/tests/
  <use_case>_test.rs   — in-process integration test; mock infrastructure

crates/oyatie-<µservice>-<bc>-infrastructure/tests/
  <repo>_integration_test.rs  — requires test DB; #[ignore] by default; run via CI lane

tests/load/
  smoke-<µservice>-<bc>.js   — k6 smoke; p99 ≤200ms per PRD §"Performance Targets"
```

---


Register the µservice's symbol space before scaffolding files
(per `feedback_grit_claim_work_done.md`):

```bash
  --agent <agent-id> \
  --intent "scaffold oyatie-<µservice>-* crates" \
  --ttl 3600 \
  crates/oyatie-<µservice>-<bc>-domain/src/lib.rs::root \
  crates/oyatie-<µservice>-<bc>-application/src/lib.rs::root \
  crates/oyatie-<µservice>-<bc>-infrastructure/src/lib.rs::root \
  crates/oyatie-<µservice>-<bc>-app/src/main.rs::main
```


---

## References

- PRD: `docs/prds/<µservice-name>.md`
- BC registrations: `docs/standards/bounded-contexts.md`
- ADR-0056 BNF v4.1 (naming authority)
- ADR-0057 LEAN checks (cross-vertical enforcement)
- ADR-0054 scaffold-claim pattern
- Memory: `feedback_naming_justification.md`, `feedback_flat_product_catalog.md`,
  `feedback_quality_performance_scalability_bar.md`
