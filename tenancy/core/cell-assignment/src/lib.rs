//! Cell-assignment controller — implementation of
//! `tenancy/IP-008-cell-assignment-controller.md`.
//!
//! Tenants live in cells: blast-radius-bounded units of the cellular
//! architecture (ADR-0248). This crate owns the assignment-decision
//! concern — which cell a tenant belongs to, which shard it hashes onto,
//! and which moves carry a lopsided or degraded placement back toward
//! balance without ever losing, duplicating or misrouting a tenant.
//!
//! # Layout
//!
//! IP-008 specifies a `-{kernel,domain,usecase,adapter,adapter-citus,worker,app}`
//! crate fan-out. The tenancy capability is capped at twelve crates and
//! this lane's `Cargo.lock` is frozen, so that fan-out is collapsed into
//! one crate as a module tree with the same layering discipline:
//!
//! - [`kernel`] — entities ([`CellId`], [`ShardKey`], [`CellHealth`],
//!   [`CellCandidate`], [`RebalanceTask`]), ports
//!   ([`CellAssignmentRepository`], [`CellHealthProbe`]) and the single
//!   error type [`CellKernelError`]. No logic.
//! - [`domain`] — pure decisions: shard derivation, cell selection,
//!   [`Placement`] bookkeeping, [`PlacementChecksum`] integrity, and
//!   [`plan_rebalance`]. No I/O, no clock, no randomness.
//! - [`usecase`] — [`CellAssignmentService`], which sequences the ports
//!   around those decisions.
//! - [`inmemory`] — reference port implementations for local development
//!   and single-process use, and the fixtures the tests run against.
//!
//! # The health state machine is three-valued on purpose
//!
//! [`CellHealth::Healthy`] receives tenants. [`CellHealth::Degraded`]
//! **keeps** the tenants it holds and receives none — degradation is
//! transient, and evacuating on it turns a one-cell wobble into a
//! multi-cell stampede. Only [`CellHealth::Unhealthy`] is drained. A cell
//! that is simply *absent* from the candidate roster is none of the
//! three: [`plan_rebalance`] refuses to plan at all
//! ([`CellKernelError::PlacementCellNotInRoster`]) rather than read an
//! incomplete inventory as a drain order.
//!
//! # Determinism
//!
//! Nothing below the port boundary reads a clock or draws randomness.
//! Selection ties break on the lexicographically smallest [`CellId`];
//! rebalance planning breaks donor, recipient and tenant ties the same
//! way; duplicate candidate entries for one cell fold to the most
//! conservative reading rather than to whichever arrived last. Two
//! processes holding the same placement and the same candidate set
//! therefore produce the identical plan, which is what makes the
//! before/after integrity checksum meaningful rather than decorative.
//!
//! # Integrity
//!
//! [`RebalancePlan::verify_applied`] is the post-execution gate and it
//! checks three independent things, because no one of them is sufficient:
//! the whole-placement fingerprint (catches a lost or duplicated tenant),
//! an exact membership check per planned task (catches a move that did
//! not happen or went to the wrong cell), and a per-cell fingerprint
//! (catches collateral movement of a tenant nobody planned to touch).
//! [`Placement::checksum`] alone is cell-agnostic and is invariant under
//! any permutation of tenants across cells — see [`PlacementChecksum`].
//!
//! # Gaps
//!
//! Deliberately deferred, and named here rather than hidden:
//!
//! - **blake3 → FNV-1a.** IP-008 derives the shard key with
//!   `blake3::hash`. `blake3` is an external dependency and this lane
//!   holds no lockfile waiver, so [`fnv1a_64`] stands in. Consequence:
//!   FNV-1a is a non-cryptographic hash with weak avalanche, so an
//!   adversary who controls tenant identifiers could craft ids that
//!   collide onto one shard and hot-spot a cell, and the additive digest
//!   fold in [`PlacementChecksum`] is likewise collidable
//!   (`h(a)+h(b) == h(c)+h(d)`) by a party who chooses tenant ids. That
//!   is acceptable while tenant ids are platform-minted; it is NOT
//!   acceptable once tenant-supplied strings drive placement. The
//!   fingerprints are therefore not the only integrity check —
//!   [`RebalancePlan::verify_applied`]'s per-task membership check
//!   involves no hashing and cannot be collided. Swapping in blake3 is a
//!   one-function change confined to [`domain::fnv1a_64`], but it
//!   re-shards every existing tenant, so it needs a migration.
//! - **Not consistent hashing.** [`cell_for_shard`] is modulo over the
//!   sorted healthy set, so a membership change remaps roughly every key
//!   rather than roughly `1/N` of them. A real ring with virtual nodes is
//!   deferred; until then `cell_for_shard` is a *placement* function and
//!   never a routing lookup — read the recorded assignment for that.
//! - **Citus / `pg_dist_shard` execution is out of scope.** IP-008's
//!   `citus_move_shard_placement` orchestrator needs `sqlx` and a live
//!   Postgres. [`CellAssignmentRepository`] is the seam it plugs into:
//!   this crate decides and verifies the moves, and that adapter will
//!   perform them. Because that adapter interpolates the cell name into
//!   SQL, [`CellId`] is validated *here*, at the mint point.
//! - **The 1-second async health-probe loop is out of scope.** It needs
//!   `tokio`. [`CellHealthProbe`] is the one-shot, synchronous seam the
//!   loop drives; the `CellHealthAlarm` emission and the probe cadence
//!   belong to the worker crate that will own the loop.
//! - **The Valkey hot-read cache is out of scope** for the same reason;
//!   [`inmemory::InMemoryCellAssignmentRepository`] is the stand-in, and
//!   it is process-local and non-durable.
//! - **Ports are synchronous.** Async would require an executor
//!   dependency. The trait shapes are chosen so an async adapter can wrap
//!   them without changing any decision in [`domain`].
//! - **Plan execution is not transactional.** If the record store fails
//!   part-way through [`CellAssignmentService::execute_plan`], the rows
//!   already written stay written and the caller's placement is left
//!   unchanged. Making that atomic is the job of the storage adapter,
//!   which has the transaction; it cannot be solved in this layer. What
//!   this layer *can* do, and does, is name the failure point:
//!   [`CellKernelError::PartialPlanExecution`] carries the index of the
//!   task that failed, so `plan.tasks()[..committed]` are exactly the
//!   moves that are durable.
//! - **No logs, metrics or events.** Emitting them needs a facade
//!   dependency and this lane's lockfile is frozen, so every diagnostic
//!   this crate can offer travels in the typed error instead. That is why
//!   the error variants carry tenant ids, cell ids and task indices
//!   rather than being bare unit variants.
//! - **Load is a single scalar** (`load_permille`). Real cell pressure is
//!   multi-dimensional (CPU, storage, connection count); collapsing it to
//!   one number is the caller's decision and this crate does not model
//!   it. Note that rebalance *recipient* choice uses actual occupancy,
//!   not this scalar.
//!
//! ADR-0083 Tier-3: production code in this crate carries no
//! `unwrap`/`expect`/`panic`, and no `as` cast that can truncate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod domain;
pub mod inmemory;
pub mod kernel;
pub mod usecase;

pub use domain::{
    FNV_OFFSET_BASIS_64, FNV_PRIME_64, Placement, PlacementChecksum, REASON_DRAIN_UNHEALTHY,
    REASON_LEVEL_LOAD, RebalancePlan, cell_for_shard, derive_shard_key, fnv1a_64, plan_rebalance,
    select_least_loaded,
};
pub use inmemory::{InMemoryCellAssignmentRepository, InMemoryCellHealthProbe};
pub use kernel::{
    CellAssignmentRepository, CellCandidate, CellHealth, CellHealthProbe, CellId, CellKernelError,
    MAX_CELL_ID_LEN, MAX_LOAD_PERMILLE, RebalanceTask, ShardKey,
};
pub use usecase::{AssignmentOutcome, CellAssignmentService};
