//! Declarative, **non-secret** gateway configuration.
//!
//! This is the ConfigMap-style routing/group config. It deliberately holds NO
//! secret material — pooled API keys are sourced exclusively from OpenBao at
//! runtime (see [`crate::keystore`]). What lives here is purely routing shape:
//! which logical groups exist, which provider channel each serves, the
//! upstream base URL, the OpenBao path to load keys from, and the
//! failover/retry policy.
//!
//! Deserialized from JSON (a ConfigMap value) via [`serde`]. Validation is a
//! pure function ([`GatewayConfig::validate`]) so it can be unit-tested and
//! run before any listener binds.

use std::collections::BTreeSet;

use oya_llm_gateway_kernel::ProviderChannel;
use serde::{Deserialize, Serialize};

/// Top-level declarative config. Non-secret by construction.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GatewayConfig {
    /// Socket address the proxy listens on, e.g. `0.0.0.0:8080`.
    pub listen_addr: String,
    /// OpenBao connection settings (address + KV mount); the token is NOT
    /// here — it is read from the `BAO_TOKEN` environment variable only.
    pub openbao: OpenBaoConfig,
    /// How often (seconds) to refresh pooled keys from OpenBao. `0` disables
    /// periodic refresh (keys are loaded once at startup).
    #[serde(default = "default_refresh_secs")]
    pub key_refresh_secs: u64,
    /// Logical key-pool groups. Each maps a route prefix to a provider channel
    /// + upstream + OpenBao key path + retry policy.
    pub groups: Vec<GroupConfig>,
}

/// OpenBao (KV v2) connection shape. Secret-free: only the address and mount.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct OpenBaoConfig {
    /// Base URL of the OpenBao service, e.g.
    /// `http://openbao.oya-kms.svc.cluster.local:8200`.
    pub address: String,
    /// KV v2 mount name. Defaults to `secret`.
    #[serde(default = "default_kv_mount")]
    pub kv_mount: String,
}

/// One logical group = one pooled provider channel behind a route prefix.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GroupConfig {
    /// Group name (unique). Used as a metric label and as the second path
    /// segment of the ingress route (`/proxy/<group>/...`).
    pub name: String,
    /// Provider dialect this group speaks (`openai` | `anthropic` | `gemini`).
    pub channel: String,
    /// Upstream API base URL the proxy forwards to, e.g.
    /// `https://api.openai.com`.
    pub upstream_base_url: String,
    /// OpenBao KV v2 secret path holding this group's pooled keys, relative to
    /// the mount, e.g. `agent-gateway/openai`. The secret's data map is read
    /// as `{ "<label>": "<api-key>", ... }`.
    pub bao_key_path: String,
    /// Anthropic-only: the `anthropic-version` header value to inject. Ignored
    /// for non-Anthropic channels.
    #[serde(default)]
    pub anthropic_version: Option<String>,
    /// Per-group failover/retry policy.
    #[serde(default)]
    pub retry: RetryPolicyConfig,
    /// Per-group blacklist threshold (consecutive failures → blacklist).
    #[serde(default = "default_blacklist_threshold")]
    pub blacklist_threshold: u32,
    /// Base cooldown (ms) applied to a blacklisted key before jitter.
    #[serde(default = "default_cooldown_base_millis")]
    pub cooldown_base_millis: u64,
    /// Max extra jitter (ms) added to a blacklisted key's cooldown.
    #[serde(default = "default_cooldown_jitter_millis")]
    pub cooldown_jitter_millis: u64,
}

/// Failover/retry policy: which upstream statuses trigger a next-key rotation,
/// the retry cap, and the jittered backoff window.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RetryPolicyConfig {
    /// HTTP status codes that trigger failover to the next key. Defaults to
    /// `[429, 500, 502, 503, 504]`.
    #[serde(default = "default_retry_statuses")]
    pub retry_on_statuses: Vec<u16>,
    /// Maximum number of upstream attempts per inbound request (>= 1).
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    /// Base backoff (ms) between attempts before jitter.
    #[serde(default = "default_backoff_base_millis")]
    pub backoff_base_millis: u64,
    /// Max extra jitter (ms) added to each backoff.
    #[serde(default = "default_backoff_jitter_millis")]
    pub backoff_jitter_millis: u64,
}

impl Default for RetryPolicyConfig {
    fn default() -> Self {
        RetryPolicyConfig {
            retry_on_statuses: default_retry_statuses(),
            max_attempts: default_max_attempts(),
            backoff_base_millis: default_backoff_base_millis(),
            backoff_jitter_millis: default_backoff_jitter_millis(),
        }
    }
}

impl RetryPolicyConfig {
    /// `true` if `status` is in the configured failover set.
    #[must_use]
    pub fn should_retry(&self, status: u16) -> bool {
        self.retry_on_statuses.contains(&status)
    }

    /// Effective attempt cap, clamped to a minimum of 1.
    #[must_use]
    pub fn attempts(&self) -> u32 {
        self.max_attempts.max(1)
    }
}

fn default_refresh_secs() -> u64 {
    300
}
fn default_kv_mount() -> String {
    "secret".to_string()
}
fn default_retry_statuses() -> Vec<u16> {
    vec![429, 500, 502, 503, 504]
}
fn default_max_attempts() -> u32 {
    3
}
fn default_backoff_base_millis() -> u64 {
    200
}
fn default_backoff_jitter_millis() -> u64 {
    150
}
fn default_blacklist_threshold() -> u32 {
    5
}
fn default_cooldown_base_millis() -> u64 {
    30_000
}
fn default_cooldown_jitter_millis() -> u64 {
    10_000
}

/// Errors from validating a [`GatewayConfig`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// No groups were declared; the gateway would route nothing.
    NoGroups,
    /// A group name appeared more than once.
    DuplicateGroup(String),
    /// A group's `channel` was not a recognized provider dialect.
    UnknownChannel { group: String, channel: String },
    /// A group's upstream URL did not start with `http://` or `https://`.
    BadUpstreamUrl { group: String, url: String },
    /// A group's OpenBao key path was empty.
    EmptyBaoPath { group: String },
    /// A group's retry policy had an empty status set.
    EmptyRetryStatuses { group: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NoGroups => write!(f, "config declares no groups"),
            ConfigError::DuplicateGroup(name) => write!(f, "duplicate group name: {name}"),
            ConfigError::UnknownChannel { group, channel } => {
                write!(f, "group {group}: unknown channel {channel:?}")
            }
            ConfigError::BadUpstreamUrl { group, url } => {
                write!(
                    f,
                    "group {group}: upstream_base_url must be http(s): {url:?}"
                )
            }
            ConfigError::EmptyBaoPath { group } => {
                write!(f, "group {group}: bao_key_path must not be empty")
            }
            ConfigError::EmptyRetryStatuses { group } => {
                write!(f, "group {group}: retry_on_statuses must not be empty")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl GatewayConfig {
    /// Validate routing shape. Pure: no I/O, no secret access. Returns the
    /// first error encountered, or `Ok(())`.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.groups.is_empty() {
            return Err(ConfigError::NoGroups);
        }
        let mut seen = BTreeSet::new();
        for group in &self.groups {
            if !seen.insert(group.name.clone()) {
                return Err(ConfigError::DuplicateGroup(group.name.clone()));
            }
            if group.parsed_channel().is_none() {
                return Err(ConfigError::UnknownChannel {
                    group: group.name.clone(),
                    channel: group.channel.clone(),
                });
            }
            let url = group.upstream_base_url.trim();
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return Err(ConfigError::BadUpstreamUrl {
                    group: group.name.clone(),
                    url: group.upstream_base_url.clone(),
                });
            }
            if group.bao_key_path.trim().is_empty() {
                return Err(ConfigError::EmptyBaoPath {
                    group: group.name.clone(),
                });
            }
            if group.retry.retry_on_statuses.is_empty() {
                return Err(ConfigError::EmptyRetryStatuses {
                    group: group.name.clone(),
                });
            }
        }
        Ok(())
    }

    /// Parse from a JSON ConfigMap value, then validate.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let config: GatewayConfig =
            serde_json::from_str(text).map_err(|e| format!("config JSON parse failed: {e}"))?;
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }
}

impl GroupConfig {
    /// The provider channel this group serves, if the `channel` string is a
    /// recognized dialect.
    #[must_use]
    pub fn parsed_channel(&self) -> Option<ProviderChannel> {
        ProviderChannel::parse(&self.channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_group(name: &str, channel: &str) -> GroupConfig {
        GroupConfig {
            name: name.to_string(),
            channel: channel.to_string(),
            upstream_base_url: "https://api.example.com".to_string(),
            bao_key_path: format!("agent-gateway/{channel}"),
            anthropic_version: None,
            retry: RetryPolicyConfig::default(),
            blacklist_threshold: 5,
            cooldown_base_millis: 30_000,
            cooldown_jitter_millis: 10_000,
        }
    }

    fn sample_config(groups: Vec<GroupConfig>) -> GatewayConfig {
        GatewayConfig {
            listen_addr: "0.0.0.0:8080".to_string(),
            openbao: OpenBaoConfig {
                address: "http://openbao.oya-kms.svc.cluster.local:8200".to_string(),
                kv_mount: "secret".to_string(),
            },
            key_refresh_secs: 300,
            groups,
        }
    }

    #[test]
    fn retry_defaults_match_429_5xx() {
        let p = RetryPolicyConfig::default();
        assert!(p.should_retry(429));
        assert!(p.should_retry(500));
        assert!(p.should_retry(503));
        assert!(!p.should_retry(200));
        assert!(!p.should_retry(401));
        assert_eq!(p.attempts(), 3);
    }

    #[test]
    fn attempts_clamped_to_min_one() {
        let p = RetryPolicyConfig {
            max_attempts: 0,
            ..RetryPolicyConfig::default()
        };
        assert_eq!(p.attempts(), 1);
    }

    #[test]
    fn valid_config_passes_validation() {
        let cfg = sample_config(vec![
            sample_group("codex", "openai"),
            sample_group("claude", "anthropic"),
        ]);
        assert_eq!(cfg.validate(), Ok(()));
    }

    #[test]
    fn no_groups_is_rejected() {
        let cfg = sample_config(vec![]);
        assert_eq!(cfg.validate(), Err(ConfigError::NoGroups));
    }

    #[test]
    fn duplicate_group_is_rejected() {
        let cfg = sample_config(vec![
            sample_group("dup", "openai"),
            sample_group("dup", "anthropic"),
        ]);
        assert_eq!(
            cfg.validate(),
            Err(ConfigError::DuplicateGroup("dup".to_string()))
        );
    }

    #[test]
    fn unknown_channel_is_rejected() {
        let cfg = sample_config(vec![sample_group("g", "mistral")]);
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::UnknownChannel { .. })
        ));
    }

    #[test]
    fn non_http_upstream_is_rejected() {
        let mut g = sample_group("g", "openai");
        g.upstream_base_url = "ftp://nope".to_string();
        let cfg = sample_config(vec![g]);
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::BadUpstreamUrl { .. })
        ));
    }

    #[test]
    fn empty_bao_path_is_rejected() {
        let mut g = sample_group("g", "openai");
        g.bao_key_path = "   ".to_string();
        let cfg = sample_config(vec![g]);
        assert!(matches!(
            cfg.validate(),
            Err(ConfigError::EmptyBaoPath { .. })
        ));
    }

    #[test]
    fn from_json_roundtrips_with_defaults() {
        // Minimal JSON: omit optional fields and confirm serde defaults apply.
        let json = r#"
        {
          "listen_addr": "0.0.0.0:8080",
          "openbao": { "address": "http://openbao.oya-kms.svc.cluster.local:8200" },
          "groups": [
            {
              "name": "codex",
              "channel": "openai",
              "upstream_base_url": "https://api.openai.com",
              "bao_key_path": "agent-gateway/openai"
            }
          ]
        }
        "#;
        let cfg = GatewayConfig::from_json(json).expect("valid config");
        assert_eq!(cfg.groups.len(), 1);
        assert_eq!(cfg.openbao.kv_mount, "secret");
        assert_eq!(cfg.key_refresh_secs, 300);
        assert_eq!(cfg.groups[0].retry.attempts(), 3);
        assert_eq!(
            cfg.groups[0].parsed_channel(),
            Some(ProviderChannel::OpenAi)
        );
    }

    #[test]
    fn from_json_accepts_all_provider_channels_for_group_selection() {
        let json = r#"
        {
          "listen_addr": "127.0.0.1:0",
          "openbao": { "address": "http://bao:8200" },
          "groups": [
            {
              "name": "openai",
              "channel": "openai",
              "upstream_base_url": "https://api.openai.com",
              "bao_key_path": "agent-gateway/openai"
            },
            {
              "name": "anthropic",
              "channel": "anthropic",
              "upstream_base_url": "https://api.anthropic.com",
              "bao_key_path": "agent-gateway/anthropic",
              "anthropic_version": "2023-06-01"
            },
            {
              "name": "gemini",
              "channel": "gemini",
              "upstream_base_url": "https://generativelanguage.googleapis.com",
              "bao_key_path": "agent-gateway/gemini"
            }
          ]
        }
        "#;

        let cfg = GatewayConfig::from_json(json).expect("all provider channels are valid");
        let selected: Vec<_> = cfg
            .groups
            .iter()
            .map(|group| (group.name.as_str(), group.parsed_channel()))
            .collect();

        assert_eq!(
            selected,
            vec![
                ("openai", Some(ProviderChannel::OpenAi)),
                ("anthropic", Some(ProviderChannel::Anthropic)),
                ("gemini", Some(ProviderChannel::Gemini)),
            ]
        );
    }

    #[test]
    fn from_json_rejects_invalid_after_parse() {
        let json = r#"
        {
          "listen_addr": "0.0.0.0:8080",
          "openbao": { "address": "http://bao:8200" },
          "groups": []
        }
        "#;
        let err = GatewayConfig::from_json(json).expect_err("no groups");
        assert!(err.contains("no groups"));
    }
}
