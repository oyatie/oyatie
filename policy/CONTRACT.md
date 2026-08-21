---
doc_class: Specification
shape: Specification
length_cap: 400
microservice: policy
related_adrs:
  - ADR-0701
  - ADR-0702
  - ADR-0280
  - ADR-0615
inbound_citations:
  - policy/README.md
---

# The C0 snapshot-store port contract

This is the port the **static-stability invariant** (ADR-0280 §D-13.E) requires and that **no crate
in the tree implements today**. It is specified here, ahead of the crate, because the crate is
blocked on hub edits this capability's envelope may not make (see `PROMOTION.md`). Specifying it
first is deliberate: the shape is the load-bearing decision, and it is reviewable without a crate.

## What exists, and why it is not enough

`iam/core/cloud-pdp-kernel/src/lib.rs:92` carries the runtime PDP seam (the trait is defined there
and nowhere else; `libs/oya-shared-pdp-kernel` carries `PolicyBundle`, `PolicyDecisionPoint` and
`DecisionAuthorizer`, which are related but are not this trait):

```rust
pub trait PolicyBundleStore: Send + Sync {
    fn load(&self) -> Result<PolicyBundle, BundleStoreError>;
    fn describe(&self) -> String;
}
```

That signature is **binary**: it either yields a bundle or an error. The invariant is **ternary**:

> A stale snapshot must **DENY or route to the authoritative shard — never silently authorize.**

(That sentence is ADR-0280 **§D-13.D**, "Policy placement". §D-13.E is the static-stability invariant
it serves; both are quoted in `policy/README.md`.)

`load()` cannot express "route to the authoritative shard". A caller holding
`Ok(bundle)` has no way to learn that the bundle it just received is too old for the decision it is
about to make, and a caller holding `Err(..)` cannot distinguish "refuse this request" from "this
cell is not authoritative for this tenant; ask the one that is". Today that third outcome is carried
nowhere, so every implementation must re-derive it — and the failure mode of getting it wrong is
**silent authorization from stale state**, which is precisely the outcome the invariant forbids.

`iam/adapters/cloud-pdp-bundle-file` (the only implementation) is load-only and is explicitly labelled
a throwaway. There is also no ReBAC snapshot distribution at all: `RebacTupleStore` in
`iam/core/policy-cedar-domain` has **no production adapter**, takes `&mut self`, and is neither `Send`
nor `Sync` — so it cannot sit behind a shared runtime PDP as written.

## The port

Destined for `policy/ports/policy-snapshot-store` (`policy-snapshot-store`). Named
`PolicySnapshotStore`, not `SnapshotStore`: `pub trait SnapshotStore` is already taken in the tree by
`os/core/etcd-domain/src/backup.rs:19` (etcd backups). Different crate, no compile conflict, but one
name for two unrelated contracts is how a reader ends up reading the wrong one. It depends on nothing in
this capability — `core` and `adapters` depend inward on it, and `facade` composes.

```rust
/// The freshness verdict for one snapshot against one caller's tolerance.
///
/// Ternary by construction. This is the whole point of the type: a binary result cannot
/// carry `RouteToAuthoritative`, and a caller that cannot express it will silently serve
/// stale state instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotVerdict {
    /// Fresh enough for this caller. Serve the decision locally.
    Serve(VersionedSnapshot),
    /// Stale beyond tolerance and this replica is NOT authoritative for the tenant.
    /// The caller MUST retry against the named shard. It MUST NOT fall back to serving.
    RouteToAuthoritative { shard: AuthoritativeShardRef, observed_age: Duration },
    /// Stale beyond tolerance and there is no reachable authoritative shard.
    /// The caller MUST deny. This is the fail-closed terminal state.
    Deny { reason: StalenessRefusal },
}

/// A last-known-good snapshot: signed, versioned, and stamped with when it was true.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedSnapshot {
    pub snapshot_version: PolicyVersion, // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,             // data_class: TENANT_SCOPED
    pub cell_id: CellId,                 // data_class: INTERNAL_ONLY
    /// When the G plane asserted this content — NOT when this replica received it.
    /// Receive-time would reset on every redistribution and make an arbitrarily old
    /// snapshot look fresh, which is the staleness bug this field exists to prevent.
    pub asserted_at: SystemTime,         // data_class: INTERNAL_ONLY
    pub is_authoritative: bool,          // data_class: INTERNAL_ONLY
    pub signature: BundleSignature,      // data_class: INTERNAL_ONLY
}

/// The C0 store. `&self` and `Send + Sync` so one instance serves a whole cell's
/// concurrent decision traffic — deliberately unlike `RebacTupleStore`'s `&mut self`.
pub trait PolicySnapshotStore: Send + Sync {
    /// Resolve the freshness verdict for `tenant` against `tolerance`.
    ///
    /// MUST NOT perform a synchronous G-plane call. The C0 face never blocks on G
    /// (ADR-0280 §D-13.E); a store that dials the control plane here reintroduces
    /// exactly the coupling the invariant removes.
    ///
    /// # Errors
    /// Returns `SnapshotStoreError` only for faults that are neither a decision nor a
    /// route — a corrupt local store, a signature that fails verification. Staleness is
    /// NEVER an error: it is a `SnapshotVerdict`.
    fn resolve(&self, tenant: &TenantId, tolerance: Duration)
        -> Result<SnapshotVerdict, SnapshotStoreError>;

    /// The version currently served for `tenant`, for `/readyz` and decision attribution.
    fn served_version(&self, tenant: &TenantId) -> Option<PolicyVersion>;

    /// Atomically replace the served snapshot. MUST validate signature and MUST reject a
    /// version that moves backwards. A rejected swap leaves the serving snapshot intact —
    /// the same order `iam/adapters/pdp-cedar` uses: validate, THEN swap.
    ///
    /// # Errors
    /// `SignatureRejected` when verification fails against the trust anchors;
    /// `VersionRegression` when `incoming.snapshot_version <= served`.
    fn swap(&self, incoming: VersionedSnapshot) -> Result<(), SnapshotStoreError>;
}
```

## Invariants the implementation owes, and the test that proves each

| # | Invariant | Test that turns red without it |
|---|---|---|
| I1 | Staleness is never an `Err`. It is always a `SnapshotVerdict`. | `resolve` on an expired snapshot returns `Ok(Deny{..})` or `Ok(RouteToAuthoritative{..})`, never `Err` |
| I2 | `Serve` is impossible when `age > tolerance`. | property test over random `(asserted_at, tolerance)`: `Serve` implies `age <= tolerance` |
| I3 | No authoritative shard + stale ⇒ `Deny`, never `Serve`. | drop the shard from the routing table, assert `Deny` |
| I4 | `resolve` performs no network call to the G plane. | fault-injection: G plane unreachable, `resolve` still returns within its deadline |
| I5 | A rejected `swap` leaves the previous snapshot serving. | swap a bad signature, assert `served_version` is unchanged |
| I6 | `swap` refuses version regression. | swap `v1` over `v2`, assert `VersionRegression` and `served_version == v2` |
| I7 | `asserted_at` is G-plane time, not receive time. | redistribute an old snapshot through a second hop, assert the verdict is unchanged |

I4 and I7 are the two that a reasonable implementation gets wrong, and both fail **open** when wrong —
I4 by making the cell hard-depend on G, I7 by making stale content look fresh.

## Relationship to the Cedar fragments in this capability

`policy/policy/static-stability.cedar` encodes the **Deny** half of the invariant as policy: F2 refuses
a read that is both stale and non-authoritative, F3 refuses evaluation past the caller's tolerance, F4
refuses an unverified snapshot outright. Cedar answers allow/deny and structurally cannot express
"ask a different shard", so the **RouteToAuthoritative** half lives here, in the port. The two halves
are complementary and neither is sufficient alone — that split is the reason this document exists
next to the fragments rather than inside them.
