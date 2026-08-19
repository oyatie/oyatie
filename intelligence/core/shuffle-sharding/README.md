# oya-shuffle-sharding

Pure Rust shuffle-sharding library for deterministic tenant-to-cell selection.

## Purpose

`oya-shuffle-sharding` preserves the ADR-0248 cellular architecture doctrine
without keeping a standalone `cell` microservice. The crate has no service
runtime, storage adapter, network client, or global registry. It ranks candidate
cells supplied by callers and returns a deterministic shard for one tenant.

## Ownership

- `tenancy` calls the crate during tenant provisioning and records the
  tenant-to-cell assignment.
- `cloud-iac` owns the candidate registry through OpenTofu state and lifecycle
  operations.
- `observability` owns health, SLO burn, and blast-radius signals that decide
  whether a cell is eligible for new tenants.
- `audit-chain` seals assignment and migration evidence.
- `api-gateway` reads the tenant principal's assigned cell for routing.

## Algorithm

1. Validate the tenant id, placement salt, shard width, and candidate shape.
2. Reject duplicate cell ids before filtering.
3. Filter to candidates that accept new tenants and match the optional pack and
   region constraints.
4. Rank every eligible cell with a stable FNV-1a hash over tenant id,
   placement salt, and cell id, then apply a SplitMix64 finalizer.
5. Sort by rank, with cell id as deterministic tie-breaker.
6. Return the first `shard_width` cell ids.

The placement salt is the intentional rebalance lever. A caller should change it
only when accepting a new assignment epoch and emitting corresponding audit
evidence.

## Commands

```sh
RUSTC_WRAPPER= cargo test --manifest-path crates/oya-shuffle-sharding/Cargo.toml
RUSTC_WRAPPER= cargo doc --manifest-path crates/oya-shuffle-sharding/Cargo.toml --no-deps
```

`RUSTC_WRAPPER=` avoids local `sccache` permission failures in this workspace.
