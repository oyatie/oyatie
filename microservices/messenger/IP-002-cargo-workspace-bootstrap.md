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
| `microservices/messenger/src/crates/oya-messenger-{channel-store,message-stream,thread-tree,read-receipt-tracker,file-attachment,mention-router,presence}-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,adapter-redis,adapter-redis-streams,adapter-websocket,adapter-meilisearch,adapter-livekit,adapter-opswat,rest,worker,sdk,app}/Cargo.toml` | create — per PRD §BC layer mapping |

## Crate Naming

Per ADR-0056 v4.1: `oya-messenger-<bc>-<layer>` with optional
`-adapter-<backend>` per ADR-0105 Amendment 3. All 52 crates pre-validated.

## Acceptance Gates

```bash
cargo build -p messenger --workspace
cargo clippy -p messenger --workspace --all-targets -- -D warnings
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice messenger
cargo run -p oya-dev-cli -- gate validate bnf-v4-1 --microservice messenger
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
