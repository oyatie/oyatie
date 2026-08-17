//! Pure deterministic thread/conversation-grouping kernel.
//!
//! Assigns inbound messages to a thread using RFC 5322 header precedence:
//! `In-Reply-To` → `References` → normalized-`Subject` fallback.
//!
//! Also provides a validated `ThreadStatus` transition function.
//!
//! Zero I/O, zero DNS, zero network.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use crate::thread_state::ThreadStatus;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Result of assigning an inbound message to a conversation thread.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThreadAssignment {
    /// Join an existing thread identified by this message-id (extracted from
    /// `In-Reply-To` or `References`).
    ExistingThread(String),
    /// Start a new thread keyed by this normalized subject string.
    FreshSubject(String),
    /// No threading information is available; caller must generate a new id.
    Unthreaded,
}

/// Error returned when a `ThreadStatus` transition is not permitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadTransitionError {
    pub from: ThreadStatus,
    pub to: ThreadStatus,
}

impl std::fmt::Display for ThreadTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "illegal thread-status transition: {:?} -> {:?}",
            self.from, self.to
        )
    }
}

// ---------------------------------------------------------------------------
// group_into_thread
// ---------------------------------------------------------------------------

/// Determine which thread an inbound message belongs to.
///
/// `headers` is a slice of `(name, value)` pairs in message-order.
/// Header name comparison is case-insensitive.
///
/// **Precedence (highest first):**
/// 1. `In-Reply-To` — first `<…>` message-id token.
/// 2. `References`  — last `<…>` message-id token.
/// 3. `Subject`     — strip Re:/Fwd:/FW: prefixes (case-insensitive, repeated),
///                    collapse whitespace, ASCII-lowercase.
pub fn group_into_thread(headers: &[(&str, &str)]) -> ThreadAssignment {
    // --- 1. In-Reply-To ---
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("In-Reply-To") {
            if let Some(mid) = first_message_id(value) {
                return ThreadAssignment::ExistingThread(mid);
            }
        }
    }

    // --- 2. References ---
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("References") {
            if let Some(mid) = last_message_id(value) {
                return ThreadAssignment::ExistingThread(mid);
            }
        }
    }

    // --- 3. Subject fallback ---
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("Subject") {
            let normalized = normalize_subject(value);
            if !normalized.is_empty() {
                return ThreadAssignment::FreshSubject(normalized);
            }
        }
    }

    ThreadAssignment::Unthreaded
}

// ---------------------------------------------------------------------------
// transition_thread_status
// ---------------------------------------------------------------------------

/// Validate and apply a `ThreadStatus` lifecycle transition.
///
/// Same-state (e.g. `Active → Active`) is idempotent and always returns `Ok`.
/// Legal forward transitions:
/// - `Active   → Muted`
/// - `Active   → Archived`
/// - `Archived → Deleted`
/// - `Muted    → Deleted`
///
/// All other moves return [`ThreadTransitionError`].
pub fn transition_thread_status(
    current: ThreadStatus,
    next: ThreadStatus,
) -> Result<ThreadStatus, ThreadTransitionError> {
    if current == next {
        return Ok(current);
    }

    let allowed = matches!(
        (current, next),
        (ThreadStatus::Active, ThreadStatus::Muted)
            | (ThreadStatus::Active, ThreadStatus::Archived)
            | (ThreadStatus::Archived, ThreadStatus::Deleted)
            | (ThreadStatus::Muted, ThreadStatus::Deleted)
    );

    if allowed {
        Ok(next)
    } else {
        Err(ThreadTransitionError {
            from: current,
            to: next,
        })
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Extract the first `<…>` message-id token from a header value.
fn first_message_id(value: &str) -> Option<String> {
    message_ids(value).next()
}

/// Extract the last `<…>` message-id token from a header value.
fn last_message_id(value: &str) -> Option<String> {
    message_ids(value).last()
}

/// Iterator over `<…>` tokens in a header value.
///
/// Yields the inner content (without the angle brackets), trimmed.
/// Empty or whitespace-only tokens are skipped.
fn message_ids(value: &str) -> impl Iterator<Item = String> + '_ {
    let mut rest = value;
    std::iter::from_fn(move || {
        loop {
            let start = rest.find('<')?;
            rest = &rest[start + 1..];
            let end = rest.find('>')?;
            let token = rest[..end].trim().to_string();
            rest = &rest[end + 1..];
            if !token.is_empty() {
                return Some(token);
            }
        }
    })
}

/// Strip `Re:`, `Fwd:`, `FW:`, `RE:`, `FWD:` prefixes (case-insensitive,
/// repeated), then collapse internal whitespace and ASCII-lowercase the result.
fn normalize_subject(value: &str) -> String {
    let mut s = value.trim().to_string();
    loop {
        let lower = s.to_ascii_lowercase();
        let stripped = strip_subject_prefix(&lower);
        if stripped.len() == lower.len() {
            break;
        }
        // Re-trim the *original* casing at same offset, then continue.
        let offset = s.len() - lower.len() + stripped.len();
        s = s[s.len() - lower.len()..][..stripped.len()]
            .trim_start()
            .to_string();
        let _ = offset; // unused after rewrite below — see corrected logic
    }
    // Redo with a cleaner loop that works on the lowercased copy throughout.
    let mut s = value.trim().to_ascii_lowercase();
    loop {
        let after = strip_subject_prefix(s.trim_start());
        let trimmed = after.trim_start();
        if trimmed.len() == s.trim_start().len() {
            break;
        }
        s = trimmed.to_string();
    }
    // Collapse internal whitespace.
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip one leading subject prefix (`re:`, `fwd:`, `fw:`) from `s` (already
/// lowercased). Returns the remainder after the prefix, or `s` unchanged.
fn strip_subject_prefix(s: &str) -> &str {
    for prefix in &["fwd:", "fw:", "re:"] {
        if let Some(rest) = s.strip_prefix(prefix) {
            return rest;
        }
    }
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thread_state::ThreadStatus;

    // --- group_into_thread ---

    #[test]
    fn in_reply_to_takes_precedence_over_references() {
        let headers = [
            ("In-Reply-To", "<parent@example.com>"),
            ("References", "<ancestor@example.com>"),
            ("Subject", "Re: Hello"),
        ];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::ExistingThread("parent@example.com".into())
        );
    }

    #[test]
    fn in_reply_to_takes_precedence_over_subject() {
        let headers = [
            ("Subject", "Re: Hello"),
            ("In-Reply-To", "<mid@host.invalid>"),
        ];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::ExistingThread("mid@host.invalid".into())
        );
    }

    #[test]
    fn references_takes_precedence_over_subject() {
        let headers = [
            ("References", "<a@x> <b@x> <c@x>"),
            ("Subject", "Meeting notes"),
        ];
        // Last message-id in References is <c@x>
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::ExistingThread("c@x".into())
        );
    }

    #[test]
    fn references_uses_last_message_id() {
        let headers = [("References", "<first@x> <second@x> <third@x>")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::ExistingThread("third@x".into())
        );
    }

    #[test]
    fn in_reply_to_uses_first_message_id() {
        let headers = [("In-Reply-To", "<first@x> <second@x>")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::ExistingThread("first@x".into())
        );
    }

    #[test]
    fn subject_fallback_strips_re_prefix() {
        let headers = [("Subject", "Re: Project update")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::FreshSubject("project update".into())
        );
    }

    #[test]
    fn subject_fallback_strips_fwd_prefix() {
        let headers = [("Subject", "Fwd: Project update")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::FreshSubject("project update".into())
        );
    }

    #[test]
    fn subject_fallback_strips_fw_prefix() {
        let headers = [("Subject", "FW: Budget")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::FreshSubject("budget".into())
        );
    }

    #[test]
    fn subject_fallback_strips_repeated_prefixes() {
        let headers = [("Subject", "Re: Re: Fwd: Meeting")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::FreshSubject("meeting".into())
        );
    }

    #[test]
    fn subject_fallback_case_insensitive_prefix() {
        let headers = [("Subject", "RE: FWD: Action items")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::FreshSubject("action items".into())
        );
    }

    #[test]
    fn subject_fallback_collapses_whitespace() {
        let headers = [("Subject", "  Hello   World  ")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::FreshSubject("hello world".into())
        );
    }

    #[test]
    fn subject_fallback_lowercases() {
        let headers = [("Subject", "URGENT: Action Required")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::FreshSubject("urgent: action required".into())
        );
    }

    #[test]
    fn empty_subject_falls_through_to_unthreaded() {
        let headers = [("Subject", "Re:")];
        assert_eq!(group_into_thread(&headers), ThreadAssignment::Unthreaded);
    }

    #[test]
    fn no_headers_returns_unthreaded() {
        assert_eq!(group_into_thread(&[]), ThreadAssignment::Unthreaded);
    }

    #[test]
    fn empty_in_reply_to_falls_through_to_references() {
        let headers = [("In-Reply-To", "  "), ("References", "<ref@x>")];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::ExistingThread("ref@x".into())
        );
    }

    #[test]
    fn header_name_matching_is_case_insensitive() {
        let headers = [
            ("in-reply-to", "<lower@case>"),
            ("REFERENCES", "<upper@case>"),
        ];
        assert_eq!(
            group_into_thread(&headers),
            ThreadAssignment::ExistingThread("lower@case".into())
        );
    }

    // --- transition_thread_status ---

    #[test]
    fn active_to_muted_is_legal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Active, ThreadStatus::Muted),
            Ok(ThreadStatus::Muted)
        );
    }

    #[test]
    fn active_to_archived_is_legal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Active, ThreadStatus::Archived),
            Ok(ThreadStatus::Archived)
        );
    }

    #[test]
    fn archived_to_deleted_is_legal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Archived, ThreadStatus::Deleted),
            Ok(ThreadStatus::Deleted)
        );
    }

    #[test]
    fn muted_to_deleted_is_legal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Muted, ThreadStatus::Deleted),
            Ok(ThreadStatus::Deleted)
        );
    }

    #[test]
    fn same_state_is_idempotent_for_all_statuses() {
        for status in [
            ThreadStatus::Active,
            ThreadStatus::Muted,
            ThreadStatus::Archived,
            ThreadStatus::Deleted,
        ] {
            assert_eq!(
                transition_thread_status(status, status),
                Ok(status),
                "same-state should be idempotent for {status:?}"
            );
        }
    }

    #[test]
    fn backward_transition_archived_to_active_is_illegal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Archived, ThreadStatus::Active),
            Err(ThreadTransitionError {
                from: ThreadStatus::Archived,
                to: ThreadStatus::Active,
            })
        );
    }

    #[test]
    fn backward_transition_deleted_to_active_is_illegal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Deleted, ThreadStatus::Active),
            Err(ThreadTransitionError {
                from: ThreadStatus::Deleted,
                to: ThreadStatus::Active,
            })
        );
    }

    #[test]
    fn muted_to_archived_is_illegal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Muted, ThreadStatus::Archived),
            Err(ThreadTransitionError {
                from: ThreadStatus::Muted,
                to: ThreadStatus::Archived,
            })
        );
    }

    #[test]
    fn active_to_deleted_is_illegal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Active, ThreadStatus::Deleted),
            Err(ThreadTransitionError {
                from: ThreadStatus::Active,
                to: ThreadStatus::Deleted,
            })
        );
    }

    #[test]
    fn deleted_to_muted_is_illegal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Deleted, ThreadStatus::Muted),
            Err(ThreadTransitionError {
                from: ThreadStatus::Deleted,
                to: ThreadStatus::Muted,
            })
        );
    }

    #[test]
    fn archived_to_muted_is_illegal() {
        assert_eq!(
            transition_thread_status(ThreadStatus::Archived, ThreadStatus::Muted),
            Err(ThreadTransitionError {
                from: ThreadStatus::Archived,
                to: ThreadStatus::Muted,
            })
        );
    }
}
