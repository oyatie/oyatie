---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-bnf-v4-1, oya-governance-per-microservice-layout, oya-governance-port-location, oya-governance-layer-correctness]
---

# IP-002: Cargo workspace bootstrap — ~165 crate scaffolds per BNF v4.1 + ADR-0131 flat layout

## Intent

Author the Cargo workspace for the `network` µservice per ADR-0131 per-microservice flat layout. ~165 crates across 25 BCs × 6-10 layers per BC depending on backend variety.

## ChangeSet boundary

`src/Cargo.toml` workspace + `src/crates/oya-network-<bc>-<layer>/` scaffolds.

## BC × Layer Matrix

| BC | Layers |
|---|---|
| `professional-profile` | kernel, domain, usecase, api, adapter-postgres, rest, sdk, app |
| `professional-graph` | kernel, domain, usecase, api, adapter-postgres, worker, sdk |
| `connection-request` | kernel, domain, usecase, api, adapter-postgres, worker, sdk |
| `post-composition` | kernel, domain, usecase, api, adapter-postgres, adapter-s3, adapter-imagemagick, adapter-ffmpeg, adapter-clamav, adapter-opswat, rest, worker, sdk, app |
| `feed-timeline` | kernel, domain, usecase, api, adapter-postgres, adapter-redis, worker, sdk, app |
| `reactions` | kernel, domain, usecase, api, adapter-postgres, adapter-redis, worker, sdk |
| `mentions` | kernel, domain, usecase, api, adapter, worker, sdk |
| `hashtags` | kernel, domain, usecase, api, adapter-postgres, worker, sdk |
| `trending-topics` | kernel, domain, usecase, api, adapter-postgres, adapter-redis, worker, sdk |
| `notifications` | kernel, domain, usecase, api, adapter-postgres, adapter-redis, worker, sdk, app |
| `inmail-bridge` | kernel, domain, usecase, api, adapter, adapter-messenger-bridge, worker, sdk |
| `endorsement-engine` | kernel, domain, usecase, api, adapter, adapter-postgres, worker, sdk |
| `skill-assessments` | kernel, domain, usecase, api, adapter, adapter-postgres, worker, sdk |
| `profile-verification` | kernel, domain, usecase, api, adapter, adapter-postgres, sdk |
| `pages` | kernel, domain, usecase, api, adapter, adapter-postgres, rest, sdk, app |
| `groups` | kernel, domain, usecase, api, adapter, adapter-postgres, rest, sdk, app |
| `events-bridge` | kernel, domain, usecase, api, adapter, adapter-calendar-bridge, worker, sdk |
| `jobs-handoff` | kernel, domain, usecase, api, adapter, adapter-postgres, adapter-ats-bridge, worker, sdk |
| `recruiter-stub` | kernel, domain, usecase, api, adapter, adapter-postgres, sdk |
| `services-marketplace-stub` | kernel, domain, usecase, api, adapter, adapter-postgres, sdk |
| `learning-stub` | kernel, domain, usecase, api, adapter, adapter-postgres, sdk |
| `salary-insights-stub` | kernel, domain, usecase, api, adapter, adapter-postgres, sdk |
| `search` | kernel, domain, usecase, api, adapter, adapter-meilisearch, worker, sdk |
| `accessibility-captions` | kernel, domain, usecase, api, adapter, worker, sdk |
| `abuse-reporting` | kernel, domain, usecase, api, adapter, adapter-postgres, worker, sdk |

Plus the `oya-network-app` composition root.

## Naming justification (BNF v4.1)

```
NAME: oya-network-<bc>-<layer>
JUSTIFICATION:
- microservice = network: per ADR-0131 per-microservice flat layout.
- bc-tokens = <bc>: primary BC. ADR-0056 v4.1 BC-optionality rule honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-<backend> per ADR-0105 Amendment 3.
```

## Concrete File Targets

| Path | Action |
|---|---|
| `src/Cargo.toml` | register ~165 crates as workspace members |
| `src/crates/oya-network-*/Cargo.toml` | per-crate manifest with workspace-inherited deps |
| `src/crates/oya-network-*-kernel/src/lib.rs` | port-trait + entity scaffolds (zero I/O) |
| `src/crates/oya-network-*-domain/src/lib.rs` | pure-logic scaffolds |
| `src/crates/oya-network-*-usecase/src/lib.rs` | orchestrator scaffolds |
| `src/crates/oya-network-*-adapter-postgres/migrations/0001_init.sql` | RLS + CHECK `context_kind='Professional'` |

## Acceptance Gates

```bash
cargo check --workspace
cargo nextest run --workspace --no-run  # compile-only smoke
cargo run -p oya-dev-cli -- gate validate bnf-v4-1 --microservice network
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice network
cargo run -p oya-dev-cli -- gate validate port-location --microservice network
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice network
```

## Test Plan

- All ~165 crates compile via `cargo check --workspace`.
- BNF v4.1 lane green: every crate name conforms.
- Per-µservice layout lane green: all crates under `microservices/network/src/crates/`.
- Port-location lane green: kernel crates have zero I/O imports.
- Layer-correctness lane green: usecase imports only kernel + domain; adapter imports only kernel.

## Halt Conditions

- Cargo workspace doesn't compile — diagnose and fix.
- BNF v4.1 lane fails — fix the crate name.
- Port-location lane fails — refactor I/O out of kernel.

## Next IP

[`IP-003-professional-profile-bc.md`](IP-003-professional-profile-bc.md)

## References

- ADR-0056 (BNF v4.1); ADR-0105 (13-layer enum); ADR-0106 (usecase rename); ADR-0131 (per-µservice flat layout).
- `microservices/network/PRD.md` §Bounded Contexts.
