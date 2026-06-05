---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-check, cargo-build, oya-governance-per-microservice-layout, oya-governance-bnf-v4-1]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Cargo workspace bootstrap (per-µservice flat layout)

## Intent

Create the flat Cargo workspace at `microservices/messenger/src/crates/`
per ADR-0131. Scaffold the 52 crates listed in PRD §"Bounded Contexts"
with workspace-level lints, MSRV pin, and `#![deny(unsafe_code)]` everywhere.

## ChangeSet boundary

One ChangeSet:
- `microservices/messenger/Cargo.toml` (workspace root)
- 52 child crate directories with `Cargo.toml` + `src/lib.rs` (empty + module skeletons)
- workspace-level `clippy.toml`, `deny.toml`, `rustfmt.toml`

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/messenger/Cargo.toml` | create — workspace manifest, members glob |
| `microservices/messenger/src/crates/oya-messenger-channel-store-kernel/Cargo.toml` | create |
| `microservices/messenger/src/crates/oya-messenger-channel-store-kernel/src/lib.rs` | create — pub mod traits; pub mod entities |
| `microservices/messenger/src/crates/oya-messenger-{channel-store,message-stream,thread-tree,read-receipt-tracker,file-attachment,mention-router,presence}-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,adapter-valkey,adapter-valkey-streams,adapter-websocket,adapter-meilisearch,adapter-livekit,adapter-opswat,rest,worker,sdk,app}/Cargo.toml` | create — per PRD §BC layer mapping |

## Crate Naming

Per ADR-0056 v4.1: `oya-messenger-<bc>-<layer>` with optional
`-adapter-<backend>` per ADR-0105 Amendment 3. All 52 crates pre-validated.

## Acceptance Gates

```bash
cargo build -p messenger --workspace
cargo clippy -p messenger --workspace --all-targets -- -D warnings
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice messenger
buck2 build //:quality-lane-registry-authority-check # lane=bnf-v4-1 --microservice messenger
```

## Test Plan

- Each kernel crate: ≥ 1 doctest asserting trait shape.
- Each domain crate: ≥ 1 unit test on a core rule.

## Halt Conditions

- Workspace cycles — fix by re-checking ADR-0105 dependency-direction.
- Naming-validator failures — fix the name, do NOT add exemptions.

## Next IP

[`IP-003-channel-store-kernel-domain.md`](IP-003-channel-store-kernel-domain.md)

## References

- ADR-0056; ADR-0105; ADR-0106; ADR-0131.
- `microservices/messenger/PRD.md` §"Bounded Contexts".

## Wave 15 substance conversion — Cargo workspace bootstrap

### §A Problem

The messenger PRD names many bounded contexts, but without a flat Rust workspace those plans cannot be claimed,
tested, or governed independently.
This IP closes the repository layout gap for the hero messenger product.

### §B Approach

Create the per-microservice flat workspace under `microservices/messenger/src/crates/` using BNF v4.1 crate names.
The bootstrap is intentionally skeletal but not content-free: it pins lints, dependency direction, MSRV, and
unsafe-code policy before domain work lands.

### §C Deliverables

- `microservices/messenger/Cargo.toml`
- workspace `clippy.toml`, `deny.toml`, and `rustfmt.toml`
- child crate manifests for channel-store, message-stream, presence, attachment, thread, receipt, huddle, and REST/app layers

### §D Implementation

1. Declare workspace members with explicit globs under `src/crates`.
2. Add `#![deny(unsafe_code)]` to every created crate root.
3. Apply dependency-direction linting so adapters cannot define domain types.
4. Create minimal doctests for kernel trait modules.
5. Create one invariant unit test for each domain crate.
6. Run BNF and per-microservice layout gates before downstream IPs claim crates.

### §E Acceptance

Cargo build/clippy plus layout and BNF gates must pass, and workspace-cycle checks must prove clean architecture
dependency direction.

### §F Evidence

Local anchors: `manifest.json` bounded contexts, `PRD.md` product scope, `ARCHITECTURE.md`, ADR-0056, ADR-0105,
ADR-0131.

### §G Counterparts

Mattermost and Matrix demonstrate self-hosted modularity; Slack/Teams demonstrate broad product surface. Oyatie
closes the execution gap with a crate graph that can support both product breadth and governed substrate use.
