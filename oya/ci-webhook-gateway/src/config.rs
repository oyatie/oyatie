//! Gateway configuration, resolved from the environment at startup.
//!
//! Secrets are NEVER read from files committed to the repo. The webhook HMAC
//! secret is injected by the deploy substrate (OpenBao + External Secrets
//! Operator per ADR-0043) at:
//!   `sref://openbao/oya/ci/forgejo-webhook-secret`
//! and surfaced to the process as the env var `OYA_FORGEJO_WEBHOOK_SECRET`.
//! The SETUP-RUNBOOK.md documents the exact human provisioning steps.

use crate::signature::{WebhookEd25519Key, WebhookSecret};

/// Env var carrying the HMAC secret (injected from the `sref` above).
pub const ENV_WEBHOOK_SECRET: &str = "OYA_FORGEJO_WEBHOOK_SECRET";
/// Env var carrying the Jenkins dispatch base URL (the generic-webhook-trigger
/// or build-token endpoint that kicks the `oyaCiLane` pipeline).
pub const ENV_JENKINS_DISPATCH_URL: &str = "OYA_JENKINS_DISPATCH_URL";
/// Env var carrying the branch the gateway gates PRs against (default `dev`).
pub const ENV_TARGET_BRANCH: &str = "OYA_GATEWAY_TARGET_BRANCH";
/// Env var carrying the bind address (default `0.0.0.0:8099`).
pub const ENV_BIND_ADDR: &str = "OYA_GATEWAY_BIND_ADDR";
/// Env var carrying the ed25519 public key for `X-Forgejo-Signature` webhook
/// verification (base64-encoded 32-byte ed25519 compressed point, standard
/// RFC 4648 base64, with or without padding). When unset the HMAC path is the
/// only accepted signature scheme.
pub const ENV_WEBHOOK_ED25519_PUBKEY: &str = "OYA_FORGEJO_WEBHOOK_ED25519_PUBKEY";

/// The default branch the gateway gates PRs against, per ADR-0363 (dev is the
/// PR target; promotion to staging/production is fast-forward).
pub const DEFAULT_TARGET_BRANCH: &str = "dev";
/// Default bind address for the receiver.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8099";

/// Resolved, validated runtime configuration.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub bind_addr: String,
    pub target_branch: String,
    /// `None` when the Jenkins dispatch URL is unset — the gateway still runs
    /// (verify + parse + route) but the dispatch stage returns a typed
    /// transport error instead of silently succeeding.
    pub jenkins_dispatch_url: Option<String>,
    pub secret_present: bool,
}

impl GatewayConfig {
    /// Build config from a key→value lookup (injectable for tests).
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Self {
        let bind_addr = get(ENV_BIND_ADDR).unwrap_or_else(|| DEFAULT_BIND_ADDR.to_owned());
        let target_branch =
            get(ENV_TARGET_BRANCH).unwrap_or_else(|| DEFAULT_TARGET_BRANCH.to_owned());
        let jenkins_dispatch_url = get(ENV_JENKINS_DISPATCH_URL).filter(|v| !v.trim().is_empty());
        let secret_present = get(ENV_WEBHOOK_SECRET)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        GatewayConfig {
            bind_addr,
            target_branch,
            jenkins_dispatch_url,
            secret_present,
        }
    }

    /// Build config from the process environment.
    pub fn from_env() -> Self {
        Self::from_lookup(|key| std::env::var(key).ok())
    }
}

/// Resolve the webhook secret from the environment. Returns an empty secret
/// (which `signature` treats as fail-closed `SecretUnavailable`) when unset,
/// so a misconfigured deploy can never accidentally accept unsigned traffic.
pub fn resolve_secret(get: impl Fn(&str) -> Option<String>) -> WebhookSecret {
    match get(ENV_WEBHOOK_SECRET) {
        Some(value) if !value.trim().is_empty() => WebhookSecret::new(value.into_bytes()),
        _ => WebhookSecret::new(Vec::new()),
    }
}

/// Resolve the optional ed25519 public key from the environment.
///
/// Returns `Some(WebhookEd25519Key)` when `OYA_FORGEJO_WEBHOOK_ED25519_PUBKEY`
/// is set to a non-empty, valid base64-encoded 32-byte ed25519 verifying key.
/// Returns `None` when the variable is absent, empty, or malformed (malformed
/// keys are silently ignored here — the gateway falls back to HMAC-only).
pub fn resolve_ed25519_key(
    get: impl Fn(&str) -> Option<String>,
) -> Option<WebhookEd25519Key> {
    use base64::Engine as _;
    let raw = get(ENV_WEBHOOK_ED25519_PUBKEY).filter(|v| !v.trim().is_empty())?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(raw.trim())
        .ok()?;
    let arr: &[u8; 32] = bytes.as_slice().try_into().ok()?;
    WebhookEd25519Key::from_bytes(arr).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup(map: HashMap<&'static str, &'static str>) -> impl Fn(&str) -> Option<String> {
        move |k: &str| map.get(k).map(|v| (*v).to_owned())
    }

    #[test]
    fn defaults_apply_when_unset() {
        let cfg = GatewayConfig::from_lookup(lookup(HashMap::new()));
        assert_eq!(cfg.bind_addr, DEFAULT_BIND_ADDR);
        assert_eq!(cfg.target_branch, "dev");
        assert!(cfg.jenkins_dispatch_url.is_none());
        assert!(!cfg.secret_present);
    }

    #[test]
    fn overrides_apply() {
        let mut map = HashMap::new();
        map.insert(ENV_TARGET_BRANCH, "dev");
        map.insert(
            ENV_JENKINS_DISPATCH_URL,
            "http://jenkins.oya-ci-jenkins.svc:8080/job/oya/build",
        );
        map.insert(ENV_WEBHOOK_SECRET, "s3cr3t");
        let cfg = GatewayConfig::from_lookup(lookup(map));
        assert!(cfg.jenkins_dispatch_url.is_some());
        assert!(cfg.secret_present);
    }

    #[test]
    fn blank_secret_is_absent() {
        let mut map = HashMap::new();
        map.insert(ENV_WEBHOOK_SECRET, "   ");
        let secret = resolve_secret(lookup(map));
        assert!(secret.is_empty());
    }
}
