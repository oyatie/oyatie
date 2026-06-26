//! Session pinning (pure kernel) for prompt-cache preservation.
//!
//! Rotating the upstream seat mid-conversation forces the provider to re-create
//! its prompt cache, costing 5-10x on the first turn after the switch. To avoid
//! that, the gateway pins a logical *session* to one seat for a bounded TTL and
//! only rebinds on failover (e.g. a 429 cooldown on the pinned seat — handled by
//! [`crate::SubscriptionPool`] which drops the sticky binding on any non-Ok
//! outcome).
//!
//! This module owns the two pure derivations the rest of the pool needs:
//!
//! 1. [`derive_sticky_key`] — the **dual-format** affinity-key extractor. A
//!    request either carries the client's own wire session id (authoritative,
//!    stable for the whole conversation) or, when it does not, we derive a
//!    privacy-preserving fingerprint from the first user message. We never store
//!    or echo raw prompt content or raw wire ids.
//! 2. [`prompt_cache_key`] — the `tenant::provider::session::model` cache key
//!    the proxy layer uses to address the upstream prompt cache.
//!
//! Both are deterministic and side-effect free so they can be proptested and so
//! two gateway replicas pin identical sessions to identical keys.

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use crate::{Provider, TenantId};

/// Namespace prefix for keys derived from a client-supplied wire session id.
const WIRE_SESSION_PREFIX: &str = "wsid:";
/// Namespace prefix for keys derived from a first-user-message fingerprint.
/// Kept identical to the legacy [`crate::privacy_preserving_sticky_key`] prefix
/// and width; the hash algorithm is SHA-256.
const MESSAGE_PREFIX: &str = "sticky:";

/// Width (in lowercase hex chars) of the message fingerprint. 16 hex chars =
/// 64 bits — matches the historical key width and the brief's `sha256(..)[:16]`.
// ponytail: 64-bit fingerprint; widen to 32 hex (128 bit) if cross-tenant
// cache-key collisions ever become a real adversarial concern.
const FINGERPRINT_HEX_LEN: usize = 16;

/// Derive a stable, privacy-preserving sticky-affinity key from whatever
/// session signal the request carries. Returns `None` only when neither a
/// usable wire session id nor a first user message is available — the caller
/// then leases without pinning.
///
/// Precedence: a non-blank wire session id always wins, because it is stable
/// across every turn of the conversation (the message fingerprint only matches
/// requests whose *first* message is byte-identical).
pub fn derive_sticky_key(
    wire_session_id: Option<&str>,
    first_user_message: Option<&str>,
) -> Option<String> {
    if let Some(raw) = wire_session_id {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Some(format!(
                "{WIRE_SESSION_PREFIX}{}",
                message_fingerprint(trimmed)
            ));
        }
    }
    first_user_message.map(message_sticky_key)
}

/// Build the message-derived sticky key: `sticky:<sha256(message)[:16 hex]>`.
/// Raw prompt content never appears in the output.
pub(crate) fn message_sticky_key(first_user_message: &str) -> String {
    format!(
        "{MESSAGE_PREFIX}{}",
        message_fingerprint(first_user_message)
    )
}

fn message_fingerprint(first_user_message: &str) -> String {
    let digest = Sha256::digest(first_user_message.as_bytes());
    let mut hex = String::with_capacity(FINGERPRINT_HEX_LEN);
    for byte in digest.iter() {
        if hex.len() >= FINGERPRINT_HEX_LEN {
            break;
        }
        // Two hex chars per byte; FINGERPRINT_HEX_LEN is even so this lands
        // exactly on the boundary.
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// Address the upstream prompt cache for a pinned session:
/// `tenant::provider::session::model`. The tenant segment makes the helper safe
/// for shared cache infrastructure; TTL stays caller-supplied at the pool seam.
pub fn prompt_cache_key(
    tenant_id: &TenantId,
    provider: Provider,
    sticky_key: &str,
    model: &str,
) -> String {
    let mut key = String::with_capacity(
        "v1:".len()
            + tenant_id.as_str().len()
            + provider.to_string().len()
            + sticky_key.len()
            + model.len()
            + 32,
    );
    key.push_str("v1:");
    push_segment(&mut key, 't', tenant_id.as_str());
    push_segment(&mut key, 'p', &provider.to_string());
    push_segment(&mut key, 's', sticky_key);
    push_segment(&mut key, 'm', model);
    key
}

fn push_segment(key: &mut String, tag: char, value: &str) {
    let _ = write!(key, "{tag}{}:{value}", value.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_session_id_takes_precedence_over_message() {
        let key = derive_sticky_key(Some("conv-123"), Some("hello")).expect("a signal is present");
        assert!(key.starts_with("wsid:"));
        assert!(!key.contains("conv-123"));
        assert_eq!(key.len(), "wsid:".len() + FINGERPRINT_HEX_LEN);
    }

    #[test]
    fn falls_back_to_message_fingerprint_when_no_wire_id() {
        let key = derive_sticky_key(None, Some("the user prompt")).expect("message present");
        assert!(key.starts_with("sticky:"));
        assert!(!key.contains("the user prompt"));
        assert_eq!(key.len(), "sticky:".len() + FINGERPRINT_HEX_LEN);
    }

    #[test]
    fn blank_wire_id_is_ignored_in_favor_of_message() {
        let key = derive_sticky_key(Some("   "), Some("prompt")).expect("message present");
        assert!(key.starts_with("sticky:"));
    }

    #[test]
    fn returns_none_when_no_signal_present() {
        assert_eq!(derive_sticky_key(None, None), None);
        assert_eq!(derive_sticky_key(Some(""), None), None);
    }

    #[test]
    fn message_key_is_deterministic_and_collision_distinct() {
        assert_eq!(message_sticky_key("a"), message_sticky_key("a"));
        assert_ne!(message_sticky_key("a"), message_sticky_key("b"));
    }

    #[test]
    fn cache_key_is_tenant_provider_session_model() {
        let tenant = TenantId::new("tenant-a").unwrap();
        let sticky_key = derive_sticky_key(Some("conv-1"), None).unwrap();
        let key = prompt_cache_key(&tenant, Provider::Anthropic, &sticky_key, "claude-opus-4");
        assert_eq!(
            key,
            format!(
                "v1:t8:tenant-ap9:anthropics{}:{}m13:claude-opus-4",
                sticky_key.len(),
                sticky_key
            )
        );
    }

    #[test]
    fn cache_key_is_unambiguous_when_segments_contain_delimiters() {
        let tenant = TenantId::new("a::b").unwrap();
        let other_tenant = TenantId::new("a").unwrap();
        assert_ne!(
            prompt_cache_key(&tenant, Provider::Anthropic, "c", "d"),
            prompt_cache_key(&other_tenant, Provider::Anthropic, "b::c", "d")
        );
    }
}
