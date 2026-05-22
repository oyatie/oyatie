---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet
acceptance_lanes: [cargo-check, cargo-build, oya-governance-per-microservice-layout, oya-governance-bnf-v4-1]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Cargo workspace bootstrap (per-µservice flat layout)

## Intent

Create the flat Cargo workspace at `microservices/meet/src/crates/` per ADR-0131. Scaffold the ~80 crates listed in PRD §"Bounded Contexts" with workspace-level lints, MSRV pin, and `#![deny(unsafe_code)]` everywhere (exception: ffmpeg adapter may need unsafe FFI; isolated to `-adapter-ffmpeg` with audit comment).

## ChangeSet boundary

One ChangeSet:
- `microservices/meet/Cargo.toml` (workspace root)
- ~80 child crate directories with `Cargo.toml` + `src/lib.rs` (empty + module skeletons)
- workspace-level `clippy.toml`, `deny.toml`, `rustfmt.toml`

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/meet/Cargo.toml` | create — workspace manifest, members glob |
| `microservices/meet/src/crates/oya-meet-meeting-room-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-meeting-instance-{kernel,domain,usecase,api,adapter-postgres,adapter-livekit,rest,worker,sdk,app}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-participant-{kernel,domain,usecase,api,adapter-postgres,adapter-valkey,rest,worker,sdk,app}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-{audio,video,screen-share}-{kernel,domain,usecase,adapter-livekit,worker,sdk}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-recording-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,adapter-ffmpeg,rest,worker,sdk,app}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-transcription-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,adapter-whisper,adapter-meilisearch,rest,worker,sdk,app}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-webinar-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-live-stream-egress-{kernel,domain,usecase,api,adapter-srs,adapter-ffmpeg,worker,sdk}/Cargo.toml` | create |
| `microservices/meet/src/crates/oya-meet-e2e-encryption-{kernel,domain,usecase,adapter-mls,sdk}/Cargo.toml` | create |

## Crate Naming

Per ADR-0056 v4.1: `oya-meet-<bc>-<layer>` with optional `-adapter-<backend>` per ADR-0105 Amendment 3. All ~80 crates pre-validated.

## Acceptance Gates

```bash
cargo build -p meet --workspace
cargo clippy -p meet --workspace --all-targets -- -D warnings
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice meet
cargo run -p oya-dev-cli -- gate validate bnf-v4-1 --microservice meet
```

## Test Plan

- Each kernel crate: ≥ 1 doctest asserting trait shape.
- Each domain crate: ≥ 1 unit test on a core rule.

## Halt Conditions

- Workspace cycles — fix by re-checking ADR-0105 dependency-direction.
- Naming-validator failures — fix the name, do NOT add exemptions.
- Unsafe-code escape in non-FFI crate — refuse; isolate.

## Next IP

[`IP-003-meeting-room-kernel-domain.md`](IP-003-meeting-room-kernel-domain.md)

## References

- ADR-0056; ADR-0105; ADR-0106; ADR-0131.
- `microservices/meet/PRD.md` §"Bounded Contexts".
