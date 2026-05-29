# Plan: messenger-mention-fanout-usecase

## Objective

Add a pure, deterministic `mention_fanout` module to `oya-messenger-message-stream-usecase`
that derives a notification target set from explicit mention tokens in a message body,
intersected against a supplied channel-membership roster.

## Requirements Analysis

### Core behaviour
1. Parse mention tokens from `body` — tokens of the form `@<ref>` where `<ref>` is a
   non-whitespace sequence.
2. Intersect parsed mentions against `channel_members` roster (a `&[&str]` / `&[String]`
   caller-supplied slice — no I/O).
3. Suppress the author's own self-mention (author comes from the request, not the ctx
   principal, to match `send_message` behaviour).
4. Collapse duplicates.
5. Return a deterministically **sorted** `Vec<String>` as `MentionFanout.targets`.
6. `message_id` and `channel_id` are threaded through from the inputs so the caller can
   correlate the fanout result back to the originating message.

### Error cases
- Invalid `ctx` (fails `validate()`) → `MessengerUsecaseError::Api(e)`
- `author_ref != ctx.principal_ref` → `MessengerUsecaseError::PrincipalMismatch`
- No new error variants needed; reuse existing `MessengerUsecaseError`.

### Acceptance criteria
- New `pub mod mention_fanout` wired into `lib.rs` (`pub mod` + `pub use` of the public
  fn and result type).
- Self-mention is suppressed in `targets`.
- Mentions of non-members are dropped.
- Duplicate mentions collapse to a single entry.
- `targets` is always sorted (deterministic output regardless of mention order).
- Invalid ctx → `Api(e)`.
- Wrong author → `PrincipalMismatch`.
- Full `#[cfg(test)]` coverage.
- No async / I/O anywhere.
- No new workspace member; no root `Cargo.toml` edit.

### Edge cases
- Body with zero mentions → empty targets.
- All mentions are self or non-members → empty targets.
- Member mentioned twice → appears once, sorted.
- Mention token includes trailing punctuation — tokenisation must not strip trailing
  punctuation (keep parsing simple: split on whitespace, collect tokens starting with `@`,
  strip the leading `@`).

## Subtasks (ordered)

1. Write `tasks/messenger-mention-fanout-usecase-plan.md` (this file). ✓
2. Write `docs/specs/task-messenger-mention-fanout-usecase.md`.
3. Write `crates/oya-messenger-message-stream-usecase/src/mention_fanout.rs` with
   `#[cfg(test)]` tests that compile but fail (red phase) — i.e. stub implementation.
4. Wire `pub mod mention_fanout` + `pub use` into `lib.rs`.
5. Confirm tests compile and fail (`cargo check --all-targets`).
6. Implement minimum code in `mention_fanout.rs` to pass all tests (green phase).
7. Run `cargo nextest run -p oya-messenger-message-stream-usecase` — must be green.
8. Self-review (correctness / architecture / security / performance / cloud-native).
9. Simplify (guard clauses, naming, dead code). Re-run nextest.
10. Commit only the allowed paths, push, open PR.
