//! Metal OAuth2 kernel-parameter parsing.
//!
//! Mirrors the data-shaping part of Talos'
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/metal/oauth2.NewConfig`.
//! Upstream enables OAuth2 for URL-based metal config downloads when the kernel
//! command line includes `talos.config.oauth.client_id=`. The live device-flow
//! HTTP exchange is intentionally not modeled here; this module captures the
//! source-guided configuration parsing so higher layers can decide when to run
//! that flow and which download headers it would eventually produce.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// Kernel parameter for OAuth2 client ID.
pub const KERNEL_PARAM_CONFIG_OAUTH_CLIENT_ID: &str = "talos.config.oauth.client_id";
/// Kernel parameter for OAuth2 client secret.
pub const KERNEL_PARAM_CONFIG_OAUTH_CLIENT_SECRET: &str = "talos.config.oauth.client_secret";
/// Kernel parameter for OAuth2 audience.
pub const KERNEL_PARAM_CONFIG_OAUTH_AUDIENCE: &str = "talos.config.oauth.audience";
/// Repeated kernel parameter for OAuth2 scopes.
pub const KERNEL_PARAM_CONFIG_OAUTH_SCOPE: &str = "talos.config.oauth.scope";
/// Kernel parameter overriding the device authorization endpoint.
pub const KERNEL_PARAM_CONFIG_OAUTH_DEVICE_AUTH_URL: &str = "talos.config.oauth.device_auth_url";
/// Kernel parameter overriding the token endpoint.
pub const KERNEL_PARAM_CONFIG_OAUTH_TOKEN_URL: &str = "talos.config.oauth.token_url";
/// Repeated kernel parameter listing extra Talos metal URL variables.
pub const KERNEL_PARAM_CONFIG_OAUTH_EXTRA_VARIABLE: &str = "talos.config.oauth.extra_variable";

const DEFAULT_DEVICE_AUTH_PATH: &str = "/device/code";
const DEFAULT_TOKEN_PATH: &str = "/token";

/// Parsed OAuth2 settings for a URL-based metal config download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthConfig {
    /// OAuth2 client ID. Its presence enables the flow upstream.
    pub client_id: String,
    /// Optional OAuth2 client secret.
    pub client_secret: String,
    /// Optional audience auth parameter.
    pub audience: String,
    /// Repeated OAuth2 scopes, in cmdline order.
    pub scopes: Vec<String>,
    /// Repeated extra metal URL variables, in cmdline order.
    pub extra_variables: Vec<String>,
    /// OAuth2 device authorization endpoint.
    pub device_auth_url: String,
    /// OAuth2 token endpoint.
    pub token_url: String,
}

/// OAuth2 config parse result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthConfigError {
    /// No `talos.config.oauth.client_id` was present. Upstream returns
    /// `os.ErrNotExist` for this case.
    NotConfigured,
    /// The download URL could not be reshaped into default OAuth endpoints.
    InvalidDownloadUrl,
}

impl fmt::Display for OAuthConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OAuthConfigError::NotConfigured => write!(f, "metal OAuth2 is not configured"),
            OAuthConfigError::InvalidDownloadUrl => {
                write!(f, "invalid metal OAuth2 download URL")
            }
        }
    }
}

/// Parse an upstream-compatible OAuth2 config from a kernel command line.
///
/// Mirrors `oauth2.NewConfig(procfs.Cmdline, downloadURL)`: scalar values use
/// the first occurrence, repeated scopes/extra variables preserve all
/// occurrences in order, and absent endpoint overrides default to the
/// `download_url` origin with paths `/device/code` and `/token`.
pub fn new_config(cmdline: &str, download_url: &str) -> Result<OAuthConfig, OAuthConfigError> {
    let cmdline = KernelCmdline::parse(cmdline);
    let client_id = cmdline
        .first(KERNEL_PARAM_CONFIG_OAUTH_CLIENT_ID)
        .ok_or(OAuthConfigError::NotConfigured)?;

    let device_auth_url = match cmdline.first(KERNEL_PARAM_CONFIG_OAUTH_DEVICE_AUTH_URL) {
        Some(url) => url.to_string(),
        None => default_endpoint_url(download_url, DEFAULT_DEVICE_AUTH_PATH)?,
    };
    let token_url = match cmdline.first(KERNEL_PARAM_CONFIG_OAUTH_TOKEN_URL) {
        Some(url) => url.to_string(),
        None => default_endpoint_url(download_url, DEFAULT_TOKEN_PATH)?,
    };

    Ok(OAuthConfig {
        client_id: client_id.to_string(),
        client_secret: cmdline
            .first(KERNEL_PARAM_CONFIG_OAUTH_CLIENT_SECRET)
            .unwrap_or_default()
            .to_string(),
        audience: cmdline
            .first(KERNEL_PARAM_CONFIG_OAUTH_AUDIENCE)
            .unwrap_or_default()
            .to_string(),
        scopes: cmdline.all(KERNEL_PARAM_CONFIG_OAUTH_SCOPE),
        extra_variables: cmdline.all(KERNEL_PARAM_CONFIG_OAUTH_EXTRA_VARIABLE),
        device_auth_url,
        token_url,
    })
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct KernelCmdline {
    params: Vec<(String, String)>,
}

impl KernelCmdline {
    fn parse(input: &str) -> Self {
        let mut params = Vec::new();

        for token in tokenize(input) {
            match token.split_once('=') {
                Some((key, value)) => {
                    params.push((key.to_string(), strip_quotes(value).to_string()));
                }
                None => params.push((token, String::new())),
            }
        }

        Self { params }
    }

    fn first(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(param_key, _)| param_key == key)
            .map(|(_, value)| value.as_str())
    }

    fn all(&self, key: &str) -> Vec<String> {
        self.params
            .iter()
            .filter(|(param_key, _)| param_key == key)
            .map(|(_, value)| value.clone())
            .collect()
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;

    for ch in input.chars() {
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                } else {
                    cur.push(ch);
                }
            }
            None => {
                if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                    started = true;
                } else if ch.is_whitespace() {
                    if started {
                        tokens.push(core::mem::take(&mut cur));
                        started = false;
                    }
                } else {
                    cur.push(ch);
                    started = true;
                }
            }
        }
    }

    if started {
        tokens.push(cur);
    }

    tokens
}

fn strip_quotes(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn default_endpoint_url(
    download_url: &str,
    endpoint_path: &str,
) -> Result<String, OAuthConfigError> {
    if endpoint_path.is_empty() || !endpoint_path.starts_with('/') {
        return Err(OAuthConfigError::InvalidDownloadUrl);
    }

    let (without_fragment, fragment) = split_once(download_url, '#');
    let (without_query, query) = split_once(without_fragment, '?');
    let prefix = url_prefix_before_path(without_query);

    let mut out = String::new();
    out.push_str(prefix);
    out.push_str(endpoint_path);

    if let Some(query) = query {
        out.push('?');
        out.push_str(query);
    }

    if let Some(fragment) = fragment {
        out.push('#');
        out.push_str(fragment);
    }

    Ok(out)
}

fn split_once(input: &str, delimiter: char) -> (&str, Option<&str>) {
    match input.find(delimiter) {
        Some(idx) => (&input[..idx], Some(&input[idx + delimiter.len_utf8()..])),
        None => (input, None),
    }
}

fn url_prefix_before_path(url_without_query_or_fragment: &str) -> &str {
    if let Some(scheme_idx) = url_without_query_or_fragment.find("://") {
        let authority_start = scheme_idx + 3;
        let path_start = url_without_query_or_fragment[authority_start..]
            .find('/')
            .map_or(url_without_query_or_fragment.len(), |idx| {
                authority_start + idx
            });

        return &url_without_query_or_fragment[..path_start];
    }

    if let Some(rest) = url_without_query_or_fragment.strip_prefix("//") {
        let path_start = rest
            .find('/')
            .map_or(url_without_query_or_fragment.len(), |idx| 2 + idx);

        return &url_without_query_or_fragment[..path_start];
    }

    ""
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const DOWNLOAD_URL: &str = "https://example.com/my/config";

    #[test]
    fn missing_client_id_is_not_configured() {
        assert_eq!(
            new_config("", DOWNLOAD_URL).unwrap_err(),
            OAuthConfigError::NotConfigured
        );
    }

    #[test]
    fn only_client_id_defaults_urls_from_download_url() {
        let cfg = new_config(
            "talos.config.oauth.client_id=device_client_id",
            DOWNLOAD_URL,
        )
        .unwrap();

        assert_eq!(
            cfg,
            OAuthConfig {
                client_id: "device_client_id".to_string(),
                client_secret: String::new(),
                audience: String::new(),
                scopes: Vec::new(),
                extra_variables: Vec::new(),
                token_url: "https://example.com/token".to_string(),
                device_auth_url: "https://example.com/device/code".to_string(),
            }
        );
    }

    #[test]
    fn custom_urls_override_defaults() {
        let cfg = new_config(
            "talos.config.oauth.client_id=device_client_id \
             talos.config.oauth.token_url=https://google.com/token \
             talos.config.oauth.device_auth_url=https://google.com/device/code",
            DOWNLOAD_URL,
        )
        .unwrap();

        assert_eq!(cfg.client_id, "device_client_id");
        assert_eq!(cfg.token_url, "https://google.com/token");
        assert_eq!(cfg.device_auth_url, "https://google.com/device/code");
    }

    #[test]
    fn complete_config_preserves_repeated_values() {
        let cfg = new_config(
            "talos.config.oauth.client_id=device_client_id \
             talos.config.oauth.client_secret=device_secret \
             talos.config.oauth.token_url=https://google.com/token \
             talos.config.oauth.device_auth_url=https://google.com/device/code \
             talos.config.oauth.scope=foo talos.config.oauth.scope=bar \
             talos.config.oauth.audience=world \
             talos.config.oauth.extra_variable=uuid \
             talos.config.oauth.extra_variable=mac",
            DOWNLOAD_URL,
        )
        .unwrap();

        assert_eq!(
            cfg,
            OAuthConfig {
                client_id: "device_client_id".to_string(),
                client_secret: "device_secret".to_string(),
                audience: "world".to_string(),
                scopes: vec!["foo".to_string(), "bar".to_string()],
                extra_variables: vec!["uuid".to_string(), "mac".to_string()],
                token_url: "https://google.com/token".to_string(),
                device_auth_url: "https://google.com/device/code".to_string(),
            }
        );
    }

    #[test]
    fn scalar_values_use_first_occurrence() {
        let cfg = new_config(
            "talos.config.oauth.client_id=first talos.config.oauth.client_id=second \
             talos.config.oauth.audience=world talos.config.oauth.audience=ignored",
            DOWNLOAD_URL,
        )
        .unwrap();

        assert_eq!(cfg.client_id, "first");
        assert_eq!(cfg.audience, "world");
    }

    #[test]
    fn default_urls_preserve_query_and_fragment() {
        let cfg = new_config(
            "talos.config.oauth.client_id=device_client_id",
            "https://example.com/my/config?node=one#frag",
        )
        .unwrap();

        assert_eq!(
            cfg.device_auth_url,
            "https://example.com/device/code?node=one#frag"
        );
        assert_eq!(cfg.token_url, "https://example.com/token?node=one#frag");
    }
}
