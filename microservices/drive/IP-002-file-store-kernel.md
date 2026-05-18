---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-002-file-store-kernel
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest, oya-check-data-class, oya-check-context-isolation, port-location, layer-correctness]
---

# IP-002: file-store kernel + domain + usecase + api

## Intent

Stand up the `oya-drive-file-store-{kernel,domain,usecase,api}` crates. Define port traits, entity types, data-class annotations, dual-context separation, content-address derivation, version chain invariants, and orchestrators (put-file, get-file, trash-file, purge-with-2-person-rule, apply-legal-hold, elect-worm).

## ChangeSet boundary

4 crates created; no I/O adapters yet. Pure kernel + domain + usecase + api layers. All entity fields data-class annotated; LEAN check refuses unannotated.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/drive/src/crates/oya-drive-file-store-kernel/Cargo.toml` | created |
| `microservices/drive/src/crates/oya-drive-file-store-kernel/src/lib.rs` | created |
| `microservices/drive/src/crates/oya-drive-file-store-kernel/src/entity.rs` | created — File, FileVersion, ContentAddress, RetentionPolicyRef, LegalHoldRef, FileContext{Personal,Professional} |
| `microservices/drive/src/crates/oya-drive-file-store-kernel/src/port.rs` | created — FileRepository, ContentAddressDeriver, TimeZoneResolver-equivalent (file-version timestamp), RetentionPolicyResolver, LegalHoldStore, FileContextBoundaryGuard |
| `microservices/drive/src/crates/oya-drive-file-store-domain/...` | created — pure invariant math (content-address derivation; version ordering; retention arithmetic; hold coverage; WORM monotonicity) |
| `microservices/drive/src/crates/oya-drive-file-store-usecase/...` | created — orchestrators (put-file, get-file, trash-file, purge, apply-legal-hold, elect-worm) |
| `microservices/drive/src/crates/oya-drive-file-store-api/...` | created — protocol-neutral typed contracts mirroring OpenAPI / Proto |

## Acceptance Gates

```bash
cargo build -p oya-drive-file-store-kernel -p oya-drive-file-store-domain -p oya-drive-file-store-usecase -p oya-drive-file-store-api
cargo nextest run -p oya-drive-file-store-domain
cargo nextest run -p oya-drive-file-store-usecase
cargo run -p oya-dev-cli -- gate validate data-class --microservice drive --bc file-store
cargo run -p oya-dev-cli -- gate validate context-isolation --microservice drive --bc file-store
cargo run -p oya-dev-cli -- gate validate port-location --microservice drive --bc file-store
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice drive --bc file-store
```

## Next IP

`IP-003-file-store-adapters.md`

## References

- ADR-0056 (BNF v4.1); ADR-0105 (13-layer enum); ADR-0106 (usecase rename).
- ADR-DRIVE-0002 (FastCDC); ADR-DRIVE-0006 (WORM).
- `microservices/drive/PRD.md` §Bounded Contexts row 1.
