---
doc_class: JudgmentNote
title: integ/build root_manifests hygiene + registry elevate
status: Accepted
date: 2026-08-10
ssot_todo: cargo-lock-sole-owner
---

# Envelope hygiene (#1662)

1. Stripped `registry/catalog/port-engine-*.yaml` OOB → forever `integ/registry` (#1707 / tip-free). Kept.
2. **REVERTED premature** `ci/controller/**` workspace membership (sibling `863beefce`). Paths exist only on `#1646@9316e2bcd` (MERGEABLE after affected-set restack); this tip had members with no crates → `cargo metadata` hard-fail. Chesterton: forever `#planes.root_manifests` owner is correct, but membership lines must not land before destination paths exist on the same tip / `origin/dev`.

## cargo-lock-sole-owner NEXT (parked, not deepened)

| Step | Status | Blocker |
|---|---|---|
| Specs tip-free amend membership-travel→waiver-only | WAIT | `#1644` tip-free packet (no push here) |
| Absorb `ci/controller/**` members into root `Cargo.toml` | WAIT_ON | `#1646` land → paths on `origin/dev` |
| `Cargo.lock` sole-owner refresh | WAIT_ON | members present + path exists; no third writer |
| Expire os waiver | WAIT | specs packet / `#1644` |

Unblocks when sequenced: every membership tip blocked on root_manifests sole-owner; ci tip no longer needs Cargo.toml (elevate stands on `#1646`).

**No `Cargo.lock` edit this tip** — lock == `origin/dev` (1529 pkgs); refresh only after controller members are real.
