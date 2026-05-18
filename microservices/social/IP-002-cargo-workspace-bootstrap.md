---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-build, oya-governance-per-microservice-layout, oya-governance-bnf-v4-1]
---

# IP-002: Cargo workspace bootstrap (per-µservice flat layout)

## Intent

Create the flat Cargo workspace at `microservices/social/src/crates/`
per ADR-0131. Scaffold ~115 crates listed in PRD §"Bounded Contexts"
with workspace-level lints, MSRV pin, and `#![deny(unsafe_code)]` everywhere.

## ChangeSet boundary

One ChangeSet:
- `microservices/social/Cargo.toml` (workspace root)
- ~115 child crate directories with `Cargo.toml` + `src/lib.rs` (empty + module skeletons)
- workspace-level `clippy.toml`, `deny.toml`, `rustfmt.toml`

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/social/Cargo.toml` | create — workspace manifest, members glob |
| `microservices/social/src/crates/oya-social-user-profile-kernel/Cargo.toml` | create |
| `microservices/social/src/crates/oya-social-user-profile-kernel/src/lib.rs` | create — pub mod traits; pub mod entities |
| `microservices/social/src/crates/oya-social-{user-profile,follow-graph,post-composition,feed-timeline,reactions,mentions,hashtags,trending-topics,notifications,content-moderation,bookmarks,lists,search,profile-verification,age-verification,federation-gateway}-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,adapter-s3,adapter-meilisearch,adapter-imagemagick,adapter-ffmpeg,adapter-clamav,adapter-opswat,adapter-activitypub,rest,worker,sdk,app}/Cargo.toml` | create — per PRD §BC layer mapping |

## Crate Naming

Per ADR-0056 v4.1: `oya-social-<bc>-<layer>` with optional
`-adapter-<backend>` per ADR-0105 Amendment 3. All ~115 crates pre-validated.

Backend-qualified adapter naming (ADR-0105 Amendment 3):
- `-adapter-postgres`, `-adapter-redis`, `-adapter-s3`, `-adapter-meilisearch`, `-adapter-imagemagick`, `-adapter-ffmpeg`, `-adapter-clamav`, `-adapter-opswat`, `-adapter-activitypub`.

## Acceptance Gates

```bash
cargo build -p social --workspace
cargo clippy -p social --workspace --all-targets -- -D warnings
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice social
cargo run -p oya-dev-cli -- gate validate bnf-v4-1 --microservice social
```

## Test Plan

- Each kernel crate: ≥ 1 doctest asserting trait shape.
- Each domain crate: ≥ 1 unit test on a core rule.

## Halt Conditions

- Workspace cycles — fix by re-checking ADR-0105 dependency-direction.
- Naming-validator failures — fix the name, do NOT add exemptions.

## Next IP

[`IP-003-user-profile-bc.md`](IP-003-user-profile-bc.md)

## References

- ADR-0056; ADR-0105 + Amendment 3; ADR-0106; ADR-0131.
- `microservices/social/PRD.md` §"Bounded Contexts".
