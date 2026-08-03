# Spec: social-context-scoped-story-sweep

**Status**: draft  
**Vertical**: community  
**Crate**: `oya-connect-social-domain` (`crates/oya-connect-social-domain/`)  
**Branch**: `feat/task-social-context-scoped-story-sweep-2026-05-28`  
**ADR authority**: ADR-0509 (flat single-crate service), ADR-0131 (flat layout)

---

## Objective

Extend the `oya-connect-social-domain` crate with a context- and pillar-scoped batch story-expiry sweep function (`story_sweep`) and the implicit per-context purge-target filter it embodies. The function operates as a pure domain computation over existing types — no I/O, no new crate, no root `Cargo.toml` edit.

---

## Vertical and ownership

| Attribute | Value |
|---|---|
| Vertical | community |
| Owning crate | `oya-connect-social-domain` |
| Lib name | `oya_connect_social_domain` |
| Dependency | `data-boundary-kernel` (existing) |

---

## Mod layout (flat clean-arch)

All additions live inside `crates/oya-connect-social-domain/src/lib.rs` as a new public function alongside the existing `context_snapshot` and `story_purge` functions. No new modules or files are introduced; the crate contains a single `lib.rs` by established convention.

```
src/
  lib.rs          ← add story_sweep() here; #[cfg(test)] block expanded in same file
```

---

## Public contract

### Function signature

```rust
/// Batch story-expiry sweep scoped to a single context/pillar pair.
///
/// # Context guard
/// Any post whose `context` or `pillar` does not match the supplied values causes the
/// entire batch to be rejected with `SocialError::CrossContextArtifactRef`, mirroring
/// the `context_snapshot` guard semantics.
///
/// # Artifact filtering
/// Non-Story artifacts (`FeedPost`, `CollaborativePost`) are silently skipped; they
/// contribute no purge targets and do not raise `StoryRequiresTtl`.
///
/// # Expiry aggregation
/// Among matching Story posts, only those whose `story_expires_at <= now` contribute
/// purge targets. Unexpired stories are silently skipped.
///
/// # Determinism
/// The return type is `BTreeSet<PurgeTarget>`. `PurgeTarget` derives `Ord`, so the
/// ordering is deterministic and stable across invocations; callers may rely on this.
///
/// # Empty inputs
/// An empty slice returns `Ok(BTreeSet::new())`. A batch with no expired stories also
/// returns `Ok(BTreeSet::new())`.
pub fn story_sweep(
    context: SocialContextKind,
    pillar: OwnershipPillar,
    posts: &[SocialPost],
    now: u64,
) -> Result<BTreeSet<PurgeTarget>, SocialError>
```

### Error variants used

| Variant | Trigger |
|---|---|
| `CrossContextArtifactRef` | Any post's context or pillar mismatches the sweep scope |

Variants `StoryRequiresTtl` and `StoryNotExpired` are **not** propagated by `story_sweep`; non-story and unexpired posts are silently skipped.

---

## Reuse of existing domain functions

| Existing function | Reuse in story_sweep |
|---|---|
| `context_snapshot` | Guard semantics replicated inline (iterate, check context+pillar, return `CrossContextArtifactRef`). `context_snapshot` itself is not called because sweep also needs to iterate posts for expiry; the guard is the same predicate pattern. |
| `story_purge` | Called per expired Story post to obtain the canonical `BTreeSet<PurgeTarget>`; results are unioned into the accumulator via `BTreeSet::extend`. |

---

## OpenAPI / proto3 / AsyncAPI

`story_sweep` is a pure domain function with no REST, gRPC, or event surface in this task. No OpenAPI schema, proto3 service, or AsyncAPI channel is added. When a REST or gRPC adapter exposes sweep scheduling in a future task, the adapter will define its own contract referencing this domain function.

---

## Testing strategy

All tests live in the `#[cfg(test)] mod tests` block inside `src/lib.rs`.

### [scs-1] tests

| Test name | What it asserts |
|---|---|
| `sweep_mismatched_context_yields_cross_context_artifact_ref` | Post with wrong context → `Err(CrossContextArtifactRef)` |
| `sweep_mixed_expiry_returns_only_expired_purge_targets` | Two stories (expired + unexpired, same context/pillar) → purge targets only for expired one |

### [scs-2] tests

| Test name | What it asserts |
|---|---|
| `sweep_heterogeneous_batch_skips_non_story` | Batch of `FeedPost` + expired `Story` → purge targets for story only, no `StoryRequiresTtl` |

### [scs-3] tests

| Test name | What it asserts |
|---|---|
| `sweep_empty_slice_returns_empty_set` | `&[]` → `Ok(BTreeSet::new())` |
| `sweep_all_unexpired_returns_empty_set` | All stories unexpired → `Ok(BTreeSet::new())` |

---

## Acceptance gate

```sh
cargo check -p oya-connect-social-domain --all-targets
cargo nextest run -p oya-connect-social-domain
```

Both commands must exit 0 with all tests passing.

---

## Boundaries and constraints

- **No new crate** — all additions are inside the existing `oya-connect-social-domain` crate.
- **No root `Cargo.toml` edit** — the workspace manifest is untouched.
- **No I/O** — `story_sweep` is a pure function; it takes a slice and a timestamp, returns a `Result`.
- **No adjacent refactoring** — existing functions (`context_snapshot`, `story_purge`, `SocialPost::new`) are not modified.
- **Hyperscaler-lens**: no new OSS dependencies introduced.
