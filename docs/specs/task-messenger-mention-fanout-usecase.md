# Spec: messenger-mention-fanout-usecase

## Objective

Add a pure, deterministic `mention_fanout` module to the
`messenger-message-stream-usecase` crate that derives a deduped, sorted notification
target set from explicit mention tokens in a message body intersected against a
caller-supplied channel-membership roster.

## Crate boundary

**Only crate modified:** `messenger-message-stream-usecase`
No new workspace member. No root `Cargo.toml` edit. No async / I/O.

## Mod layout (flat clean architecture per ADR-0509)

```
src/
  lib.rs                  ← pub mod mention_fanout; pub use …
  delivery_receipt.rs     ← existing
  mention_fanout.rs       ← NEW
```

## Public API

```rust
/// Input for the fanout derivation.
pub struct MentionFanoutInput<'a> {
    pub message_id: &'a str,
    pub channel_id: &'a str,
    pub author_ref: &'a str,
    pub body: &'a str,
    pub channel_members: &'a [&'a str],
}

/// Result of the fanout derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentionFanout {
    pub message_id: String,
    pub channel_id: String,
    pub targets: Vec<String>,   // deduped, sorted, author excluded
}

pub fn derive_mention_fanout(
    ctx: &AuthorizedMessengerContext,
    input: MentionFanoutInput<'_>,
) -> Result<MentionFanout, MessengerUsecaseError>
```

## Contracts

| Concern | Contract |
|---|---|
| Auth | `ctx.validate()` must pass; error → `MessengerUsecaseError::Api(e)` |
| Principal | `input.author_ref == ctx.principal_ref`; mismatch → `PrincipalMismatch` |
| Mention parsing | Split `body` on ASCII whitespace; collect tokens starting with `@`; strip leading `@` to get the ref |
| Membership | Only refs that appear in `channel_members` are retained |
| Self-suppression | `author_ref` is removed from targets even if present in membership |
| Dedup | Multiple occurrences of the same mention collapse to one entry |
| Sort | `targets` is sorted with `sort_unstable()` for determinism |
| I/O | None; no async |
| New deps | None |
| New error variants | None |

## Observability / SLO

No new SLOs introduced by this slice. The usecase layer is pure compute; OTel
instrumentation lives at the adapter/REST layer above this crate.

## Testing strategy

All tests in `#[cfg(test)]` mod inside `mention_fanout.rs`. Hermetic unit tests only.

| Test | Asserts |
|---|---|
| `happy_path_mentions_members` | mentioned members appear in sorted targets |
| `self_mention_suppressed` | author's own mention not in targets |
| `non_member_mention_dropped` | unknown ref not in targets |
| `duplicate_mentions_collapsed` | same member mentioned twice → one entry |
| `no_mentions_empty_targets` | body with no `@` tokens → empty targets |
| `all_self_or_non_member_empty` | only self + non-members mentioned → empty |
| `invalid_ctx_returns_api_error` | missing idempotency key → `Api(MissingIdempotencyKey)` |
| `principal_mismatch_rejected` | author_ref ≠ ctx.principal_ref → `PrincipalMismatch` |
| `deterministic_sort` | targets always sorted regardless of mention order in body |

## Acceptance checklist

- [ ] `pub mod mention_fanout` + `pub use derive_mention_fanout, MentionFanout` in `lib.rs`
- [ ] Self-mention suppressed
- [ ] Non-member mentions dropped
- [ ] Duplicates collapsed
- [ ] `targets` sorted (deterministic)
- [ ] Invalid ctx → `Api(e)`
- [ ] Wrong author → `PrincipalMismatch`
- [ ] Full `#[cfg(test)]` coverage
- [ ] `cargo nextest run -p messenger-message-stream-usecase` green
- [ ] No new workspace member / root Cargo.toml edit
- [ ] No async / I/O
