# Plan: social-context-scoped-story-sweep

**Vertical**: community  
**Crate**: `oya-connect-social-domain`  
**Branch**: `feat/task-social-context-scoped-story-sweep-2026-05-28`

---

## Subtasks

### [scs-1] story_sweep — context/pillar-scoped batch expiry aggregation

Add `story_sweep(context: SocialContextKind, pillar: OwnershipPillar, posts: &[SocialPost], now: u64) -> Result<BTreeSet<PurgeTarget>, SocialError>`.

**Behaviour:**
- Reject the entire batch if any post's `context` or `pillar` mismatches the supplied values → `Err(SocialError::CrossContextArtifactRef)` (reuses `context_snapshot` guard semantics).
- For each matching post that is a `Story` and whose `story_expires_at` is `<= now`, union the purge targets returned by the existing `story_purge` call into the accumulator.
- Return `Ok(accumulated_set)`.

**Acceptance:**
- `cargo check -p oya-connect-social-domain --all-targets` passes.
- `#[test] sweep_mismatched_context_yields_cross_context_artifact_ref` — a single post whose context differs from the sweep context returns `Err(CrossContextArtifactRef)`.
- `#[test] sweep_mixed_expiry_returns_only_expired_purge_targets` — a two-post batch (one expired story, one unexpired story, same context/pillar) returns purge targets only for the expired one.
- `cargo nextest run -p oya-connect-social-domain` green.

---

### [scs-2] Skip non-Story artifacts without error

`story_sweep` must silently skip `FeedPost` and `CollaborativePost` artifacts rather than returning `StoryRequiresTtl`.

**Behaviour:**
- Posts whose `kind != Story` contribute no purge targets and do not cause an error.

**Acceptance:**
- `#[test] sweep_heterogeneous_batch_skips_non_story` — batch containing a `FeedPost` and an expired `Story` returns purge targets only for the story; no `SocialError::StoryRequiresTtl` raised.
- `cargo nextest run -p oya-connect-social-domain` green.

---

### [scs-3] Empty / none-expired guard + determinism doc comment

**Behaviour:**
- Empty slice → `Ok(BTreeSet::new())`.
- Batch where no story is expired → `Ok(BTreeSet::new())`.

**Doc comment on `story_sweep` must state:** results are returned as a `BTreeSet<PurgeTarget>`, which is deterministically ordered by `PurgeTarget`'s derived `Ord` implementation; callers may rely on this ordering across invocations.

**Acceptance:**
- `#[test] sweep_empty_slice_returns_empty_set` asserts `Ok(empty)`.
- `#[test] sweep_all_unexpired_returns_empty_set` asserts `Ok(empty)`.
- `cargo check -p oya-connect-social-domain --all-targets` + `cargo nextest run -p oya-connect-social-domain` both green.

---

## Acceptance gate (all subtasks)

```
cargo check -p oya-connect-social-domain --all-targets
cargo nextest run -p oya-connect-social-domain
```

Both must exit 0 with all tests passing before the PR is opened.
