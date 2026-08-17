use comms_messenger_stream_api::AuthorizedMessengerContext;

use crate::MessengerUsecaseError;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Input for the mention fanout derivation. Caller-supplied; no I/O.
pub struct MentionFanoutInput<'a> {
    pub message_id: &'a str,
    pub channel_id: &'a str,
    /// Must equal `ctx.principal_ref`; enforced before any processing.
    pub author_ref: &'a str,
    /// Raw message body to parse mention tokens from.
    pub body: &'a str,
    /// Roster of channel members.  Only refs present here may appear in targets.
    pub channel_members: &'a [&'a str],
}

/// Deterministic, deduped notification target set derived from a message's
/// explicit @-mention tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentionFanout {
    pub message_id: String,
    pub channel_id: String,
    /// Sorted, deduped member refs that were mentioned in the body,
    /// excluding the author's own ref.
    pub targets: Vec<String>,
}

// ---------------------------------------------------------------------------
// Derive function
// ---------------------------------------------------------------------------

/// Derive a deterministic, deduped mention-fanout target set.
///
/// Steps:
/// 1. Validate `ctx` → map `MessengerApiError` to `MessengerUsecaseError::Api`.
/// 2. Principal check: `input.author_ref` must equal `ctx.principal_ref`.
/// 3. Parse mention tokens: split `body` on ASCII whitespace, collect tokens
///    starting with `@`, strip the leading `@` to obtain the ref string.
/// 4. Retain only refs that appear in `input.channel_members`.
/// 5. Suppress the author's own ref.
/// 6. Dedup and sort for determinism.
pub fn derive_mention_fanout(
    ctx: &AuthorizedMessengerContext,
    input: MentionFanoutInput<'_>,
) -> Result<MentionFanout, MessengerUsecaseError> {
    ctx.validate().map_err(MessengerUsecaseError::Api)?;
    if input.author_ref != ctx.principal_ref {
        return Err(MessengerUsecaseError::PrincipalMismatch);
    }

    let member_set: std::collections::HashSet<&str> =
        input.channel_members.iter().copied().collect();

    let mut targets: Vec<String> = input
        .body
        .split_ascii_whitespace()
        .filter_map(|token| token.strip_prefix('@'))
        .map(|raw| {
            raw.trim_end_matches(|c: char| !c.is_alphanumeric() && c != ':' && c != '_' && c != '-')
        })
        .filter(|ref_str| !ref_str.is_empty())
        .filter(|&ref_str| member_set.contains(ref_str))
        .filter(|&ref_str| ref_str != input.author_ref)
        .map(str::to_owned)
        .collect();

    targets.sort_unstable();
    targets.dedup();

    Ok(MentionFanout {
        message_id: input.message_id.to_owned(),
        channel_id: input.channel_id.to_owned(),
        targets,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use comms_messenger_stream_api::{MessengerApiContext, MessengerApiError};

    fn work_ctx(principal: &str) -> AuthorizedMessengerContext {
        AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: principal.into(),
            idempotency_key: "idem-ctx".into(),
            policy_decision_ref: "cedar:allow:message-fanout".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    fn input<'a>(
        author_ref: &'a str,
        body: &'a str,
        channel_members: &'a [&'a str],
    ) -> MentionFanoutInput<'a> {
        MentionFanoutInput {
            message_id: "msg:1",
            channel_id: "chan:1",
            author_ref,
            body,
            channel_members,
        }
    }

    #[test]
    fn happy_path_mentions_members() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice", "user:bob", "user:carol"];
        let fanout = derive_mention_fanout(
            &ctx,
            input("user:alice", "Hello @user:bob and @user:carol!", &members),
        )
        .unwrap();
        assert_eq!(fanout.message_id, "msg:1");
        assert_eq!(fanout.channel_id, "chan:1");
        assert_eq!(fanout.targets, vec!["user:bob", "user:carol"]);
    }

    #[test]
    fn self_mention_suppressed() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice", "user:bob"];
        let fanout = derive_mention_fanout(
            &ctx,
            input("user:alice", "Hey @user:alice and @user:bob", &members),
        )
        .unwrap();
        // alice is the author — must not appear in targets
        assert_eq!(fanout.targets, vec!["user:bob"]);
    }

    #[test]
    fn non_member_mention_dropped() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice", "user:bob"];
        let fanout = derive_mention_fanout(
            &ctx,
            input(
                "user:alice",
                "@user:outsider please join @user:bob",
                &members,
            ),
        )
        .unwrap();
        // user:outsider is not a member — must be dropped
        assert_eq!(fanout.targets, vec!["user:bob"]);
    }

    #[test]
    fn duplicate_mentions_collapsed() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice", "user:bob"];
        let fanout = derive_mention_fanout(
            &ctx,
            input(
                "user:alice",
                "@user:bob did you see @user:bob's message?",
                &members,
            ),
        )
        .unwrap();
        assert_eq!(fanout.targets, vec!["user:bob"]);
    }

    #[test]
    fn no_mentions_empty_targets() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice", "user:bob"];
        let fanout =
            derive_mention_fanout(&ctx, input("user:alice", "No mentions here.", &members))
                .unwrap();
        assert!(fanout.targets.is_empty());
    }

    #[test]
    fn all_self_or_non_member_empty() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice"];
        let fanout = derive_mention_fanout(
            &ctx,
            input(
                "user:alice",
                "@user:alice @user:ghost @user:nobody",
                &members,
            ),
        )
        .unwrap();
        assert!(fanout.targets.is_empty());
    }

    #[test]
    fn invalid_ctx_returns_api_error() {
        let bad_ctx = AuthorizedMessengerContext {
            context: MessengerApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:alice".into(),
            idempotency_key: "".into(), // triggers MissingIdempotencyKey
            policy_decision_ref: "cedar:allow".into(),
            audit_correlation_id: "audit".into(),
        };
        let members = ["user:alice", "user:bob"];
        let err = derive_mention_fanout(&bad_ctx, input("user:alice", "@user:bob hello", &members))
            .unwrap_err();
        assert_eq!(
            err,
            MessengerUsecaseError::Api(MessengerApiError::MissingIdempotencyKey)
        );
    }

    #[test]
    fn principal_mismatch_rejected() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice", "user:bob"];
        let err = derive_mention_fanout(&ctx, input("user:mallory", "@user:bob hello", &members))
            .unwrap_err();
        assert_eq!(err, MessengerUsecaseError::PrincipalMismatch);
    }

    #[test]
    fn deterministic_sort() {
        let ctx = work_ctx("user:alice");
        let members = ["user:alice", "user:carol", "user:bob"];
        // Mention bob before carol in body
        let fanout1 =
            derive_mention_fanout(&ctx, input("user:alice", "@user:bob @user:carol", &members))
                .unwrap();
        // Mention carol before bob in body
        let fanout2 =
            derive_mention_fanout(&ctx, input("user:alice", "@user:carol @user:bob", &members))
                .unwrap();
        assert_eq!(fanout1.targets, fanout2.targets);
        assert_eq!(fanout1.targets, vec!["user:bob", "user:carol"]);
    }
}
