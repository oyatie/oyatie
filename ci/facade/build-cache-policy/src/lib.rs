//! cloud-ci-cache-wiring (ADR-0560): policy-as-data buck2 cache wiring for the
//! NativeLink CAS warm substrate, consuming the ADR-0556 classification.
//!
//! NEUTRAL engine — all repo-specifics live in DATA:
//! - the class posture comes from `/specs/cache-warmth-policy.json` (ADR-0556 D1;
//!   this crate re-decides nothing, it only enforces the policy fail-closed);
//! - the canary-licensed kill-switch comes from `/specs/cache-warm-license.json`
//!   (the mechanical carrier of the ADR-0556 D2 trust-invariant clause (b));
//! - endpoint addresses and the REAPI instance come from `/specs/cache-endpoints.json`;
//! - the opt-in overlays live under `infra/ci/buckconfig/`; the controller
//!   fills their endpoint tokens and materializes the selected effective config
//!   privately for one child daemon.
//!
//! Fail-closed everywhere: unknown class -> bypass, unlicensed -> bypass, the
//! canary class -> bypass unconditionally, warm emission without a keyed identity
//! -> hard error. Cache telemetry is read structurally from buck2's
//! `--unstable-write-invocation-record` JSON, never grepped from logs.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// Repo-relative path of the ADR-0556 classification policy (single source).
pub const POLICY_PATH: &str = "specs/cache-warmth-policy.json";
/// Repo-relative path of the ADR-0560 warm-license kill-switch.
pub const LICENSE_PATH: &str = "specs/cache-warm-license.json";
/// Repo-relative endpoint and REAPI instance policy (single source).
pub const ENDPOINTS_PATH: &str = "specs/cache-endpoints.json";
/// Repo-relative path of the warm read+write overlay (writer endpoint).
pub const OVERLAY_RW_PATH: &str = "infra/ci/buckconfig/warm-cache-rw.buckconfig";
/// Repo-relative path of the warm read-only overlay (reader endpoint).
pub const OVERLAY_RO_PATH: &str = "infra/ci/buckconfig/warm-cache-ro.buckconfig";
/// Env var carrying the path of the lane's mTLS client certificate (keyed identity).
pub const CLIENT_CERT_ENV: &str = "OYA_CACHE_TLS_CLIENT_CERT";
/// Env var carrying the path of the CA bundle that signed the CAS server cert.
pub const TLS_CA_CERTS_ENV: &str = "OYA_CACHE_TLS_CA_CERTS";
/// Env var carrying the trusted OpenBao HTTPS endpoint.
pub const OPENBAO_ADDR_ENV: &str = "OYA_OPENBAO_ADDR";
/// Env var carrying the public CA path used to authenticate OpenBao.
pub const OPENBAO_CA_ENV: &str = "OYA_OPENBAO_CA_CERT";
/// Public CA that validates the NativeLink server certificate. This is not the
/// OpenBao HTTPS CA and must never be derived from it.
pub const CACHE_SERVER_CA_ENV: &str = "OYA_CACHE_TLS_SERVER_CA_CERT";
/// Overlay token replaced from the validated endpoint profile before Buck2 starts.
pub const RE_ADDRESS_TOKEN: &str = "__CACHE_RE_ADDRESS__";
/// Overlay token replaced from the validated endpoint profile before Buck2 starts.
pub const INSTANCE_NAME_TOKEN: &str = "__CACHE_INSTANCE_NAME__";
/// Schema id of the structured per-lane cache-hit report artifact.
pub const CACHE_HIT_REPORT_SCHEMA: &str = "oya-ci/cache-hit-report/v1";
/// Schema id of the canary digest manifest artifact.
pub const DIGEST_MANIFEST_SCHEMA: &str = "oya-ci/canary-digest-manifest/v1";
/// Schema id of the canary verdict artifact.
pub const CANARY_VERDICT_SCHEMA: &str = "oya-ci/canary-verdict/v1";

/// One NativeLink listener in both Buck2 and direct-socket forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEndpoint {
    re_address: String,
    socket_address: String,
}

/// Validated endpoint profile for one repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEndpointProfile {
    instance_name: String,
    writer: CacheEndpoint,
    reader: CacheEndpoint,
}

impl CacheEndpoint {
    pub fn re_address(&self) -> &str {
        &self.re_address
    }

    pub fn socket_address(&self) -> &str {
        &self.socket_address
    }
}

impl CacheEndpointProfile {
    pub fn instance_name(&self) -> &str {
        &self.instance_name
    }

    pub fn writer(&self) -> &CacheEndpoint {
        &self.writer
    }

    pub fn reader(&self) -> &CacheEndpoint {
        &self.reader
    }
}

/// The single role binding for a warm mode: overlay, endpoint, and upload posture.
#[derive(Debug, Clone, Copy)]
pub struct CacheModeBinding<'a> {
    pub overlay_path: &'static str,
    pub endpoint: &'a CacheEndpoint,
    pub allows_uploads: bool,
}

/// Resolve all role-specific cache wiring from one match so controller and
/// conformance code cannot independently pair an overlay with the wrong listener.
pub fn cache_mode_binding(
    profile: &CacheEndpointProfile,
    mode: CacheMode,
) -> Option<CacheModeBinding<'_>> {
    match mode {
        CacheMode::Bypass => None,
        CacheMode::WarmReadOnly => Some(CacheModeBinding {
            overlay_path: OVERLAY_RO_PATH,
            endpoint: &profile.reader,
            allows_uploads: false,
        }),
        CacheMode::WarmReadWrite => Some(CacheModeBinding {
            overlay_path: OVERLAY_RW_PATH,
            endpoint: &profile.writer,
            allows_uploads: true,
        }),
    }
}

/// The resolved cache posture for one build invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheMode {
    /// No cache participation at all: no RE client config is emitted, buck2 never
    /// dials the CAS (the ADR-0556 cold posture is bypass, not read-only).
    Bypass,
    /// Read hits allowed, uploads forbidden (reader endpoint; AC served read-only).
    WarmReadOnly,
    /// Read + write through the writer endpoint (trusted lanes only).
    WarmReadWrite,
}

impl CacheMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CacheMode::Bypass => "bypass",
            CacheMode::WarmReadOnly => "warm-ro",
            CacheMode::WarmReadWrite => "warm-rw",
        }
    }
}

impl fmt::Display for CacheMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A resolution: the mode plus the reason chain that licensed (or refused) it.
#[derive(Debug, Clone)]
pub struct Resolution {
    pub build_class: String,
    pub mode: CacheMode,
    pub reasons: Vec<String>,
}

impl Resolution {
    pub fn to_json(&self) -> Value {
        json!({
            "build_class": self.build_class,
            "mode": self.mode.as_str(),
            "reasons": self.reasons,
        })
    }
}

fn bool_field(value: &Value, key: &str) -> Result<bool, String> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("required boolean field `{key}` missing or non-boolean"))
}

fn required_string(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("required string field `{key}` missing or empty"))
}

fn validate_socket_address(value: &str) -> Result<(), String> {
    if value.chars().any(char::is_whitespace)
        || value.contains(['/', '?', '#', '@'])
        || value.matches(':').count() != 1
    {
        return Err(format!(
            "cache socket address `{value}` must use host:decimal-port grammar"
        ));
    }
    let (host, port_text) = value
        .split_once(':')
        .ok_or_else(|| format!("cache socket address `{value}` is missing a port"))?;
    let valid_host = !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
        });
    if !valid_host {
        return Err(format!(
            "cache socket address `{value}` contains an invalid host"
        ));
    }
    if port_text.is_empty() || !port_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "cache socket address `{value}` contains an invalid port"
        ));
    }
    let port = port_text
        .parse::<u16>()
        .map_err(|_| format!("cache socket address `{value}` contains an invalid port"))?;
    if port == 0 || port.to_string() != port_text {
        return Err(format!(
            "cache socket address `{value}` contains a non-canonical port"
        ));
    }
    Ok(())
}

fn parse_endpoint(value: &Value, role: &str) -> Result<CacheEndpoint, String> {
    if !value.is_object() {
        return Err(format!(
            "cache endpoint role `{role}` missing or non-object"
        ));
    }
    let re_address = required_string(value, "re_address")?;
    let socket_address = required_string(value, "socket_address")?;
    validate_socket_address(&socket_address)?;
    let re_socket = re_address.strip_prefix("grpc://").ok_or_else(|| {
        format!("cache endpoint role `{role}` re_address must use grpc://host:port grammar")
    })?;
    validate_socket_address(re_socket)?;
    if re_socket != socket_address {
        return Err(format!(
            "cache endpoint role `{role}` re_address and socket_address disagree"
        ));
    }
    Ok(CacheEndpoint {
        re_address,
        socket_address,
    })
}

/// Parse and validate one named profile from already-decoded endpoint policy DATA.
pub fn parse_endpoint_profile(
    policy: &Value,
    profile_name: &str,
) -> Result<CacheEndpointProfile, String> {
    if policy.get("policy_id").and_then(Value::as_str) != Some("cache-endpoints") {
        return Err("cache endpoint policy requires policy_id `cache-endpoints`".to_string());
    }
    if policy.get("schema_version").and_then(Value::as_str) != Some("1.0.0") {
        return Err("cache endpoint policy requires schema_version `1.0.0`".to_string());
    }
    if policy.get("adr").and_then(Value::as_str) != Some("ADR-0703") {
        return Err("cache endpoint policy requires adr `ADR-0703`".to_string());
    }
    let profile = policy
        .get("profiles")
        .and_then(|profiles| profiles.get(profile_name))
        .filter(|profile| profile.is_object())
        .ok_or_else(|| format!("cache endpoint profile `{profile_name}` missing or non-object"))?;
    let instance_name = required_string(profile, "instance_name")?;
    if instance_name.len() > 255
        || instance_name.contains("__CACHE_")
        || !instance_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
    {
        return Err(
            "cache endpoint instance_name must be 1..=255 ASCII characters from \
             [A-Za-z0-9._/-] and must not contain cache materialization tokens"
                .to_string(),
        );
    }
    let writer = profile
        .get("writer")
        .ok_or_else(|| "cache endpoint role `writer` missing".to_string())
        .and_then(|value| parse_endpoint(value, "writer"))?;
    let reader = profile
        .get("reader")
        .ok_or_else(|| "cache endpoint role `reader` missing".to_string())
        .and_then(|value| parse_endpoint(value, "reader"))?;
    if writer.socket_address == reader.socket_address {
        return Err("cache endpoint writer and reader must be distinct listeners".to_string());
    }
    Ok(CacheEndpointProfile {
        instance_name,
        writer,
        reader,
    })
}

fn parse_active_endpoint_profile(policy: &Value) -> Result<CacheEndpointProfile, String> {
    let active_profile = required_string(policy, "active_profile")?;
    parse_endpoint_profile(policy, &active_profile)
}

/// Load and fail-closed validate one repository's NativeLink endpoint profile.
pub fn load_endpoint_profile(root: &Path) -> Result<CacheEndpointProfile, String> {
    let path = root.join(ENDPOINTS_PATH);
    let text =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let policy: Value = serde_json::from_str(&text)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    parse_active_endpoint_profile(&policy)
}

/// Load + structurally validate the warmth policy.
pub fn load_policy(root: &Path) -> Result<Value, String> {
    let path = root.join(POLICY_PATH);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let policy: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if !policy.get("build_classes").is_some_and(Value::is_object) {
        return Err("policy missing `build_classes` object".to_string());
    }
    Ok(policy)
}

/// Load + structurally validate the warm-license kill-switch. A malformed or
/// missing license is an ERROR (loud), never an implicit grant.
pub fn load_license(root: &Path) -> Result<Value, String> {
    let path = root.join(LICENSE_PATH);
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let license: Value =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    bool_field(&license, "warm_reads_licensed")?;
    Ok(license)
}

/// The name of the canary build class, read from the policy's own trust-invariant
/// pointer (no class name is hardcoded here).
pub fn canary_class(policy: &Value) -> Option<&str> {
    policy
        .get("trust_invariant")
        .and_then(|t| t.get("canary_build_class"))
        .and_then(Value::as_str)
}

/// Resolve the cache posture for `build_class`, fail-closed.
pub fn resolve(policy: &Value, license: &Value, build_class: &str) -> Result<Resolution, String> {
    let mut reasons = Vec::new();
    let bypass = |reasons: Vec<String>| Resolution {
        build_class: build_class.to_string(),
        mode: CacheMode::Bypass,
        reasons,
    };

    // 1. The trust anchor may NEVER participate, independent of any other data:
    //    any cache participation makes the proof circular (ADR-0556 D1).
    if canary_class(policy) == Some(build_class) {
        reasons.push(
            "integrity-canary trust anchor: cache participation would make the warm==cold \
             proof circular (ADR-0556 D1/D2) — bypass unconditionally"
                .to_string(),
        );
        return Ok(bypass(reasons));
    }

    // 2. Unlisted classes have no warm license — warmth is granted by reviewed
    //    classification, never by omission (policy default_for_unlisted_classes).
    let Some(entry) = policy
        .get("build_classes")
        .and_then(|c| c.get(build_class))
        .filter(|e| e.is_object())
    else {
        reasons.push(format!(
            "build class `{build_class}` is not listed in {POLICY_PATH}: fail-closed default \
             (cold, no read, no write) — warmth is never granted by omission (ADR-0556 D1)"
        ));
        return Ok(bypass(reasons));
    };

    let warmth = entry.get("warmth").and_then(Value::as_str).unwrap_or("");
    let cache_read = bool_field(entry, "cache_read")?;
    let cache_write = bool_field(entry, "cache_write")?;

    // 3. Cold classes BYPASS the CAS entirely (never read-only): a cold build does
    //    not dial the CAS at all (ADR-0556 D3).
    if warmth != "warm" || !cache_read {
        reasons.push(format!(
            "class `{build_class}` is classified {warmth:?} with cache_read={cache_read}: \
             cold classes bypass the CAS entirely (ADR-0556 D3)"
        ));
        return Ok(bypass(reasons));
    }

    // 4. The kill-switch: warm is admissible IFF the most recent cold canary run is
    //    GREEN (ADR-0556 D2 clause (b), carried by the license file).
    if !bool_field(license, "warm_reads_licensed")? {
        let why = license
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("license file carries warm_reads_licensed=false");
        reasons.push(format!(
            "warm-eligible class `{build_class}` refused: warm_reads_licensed=false ({why}) — \
             the ADR-0556 D2 IFF is unsatisfied, all warm reads stay suspended"
        ));
        return Ok(bypass(reasons));
    }

    reasons.push(format!(
        "class `{build_class}` is warm-eligible and the warm license is GREEN-backed \
         (licensed_by_canary_run={})",
        license
            .get("licensed_by_canary_run")
            .map(Value::to_string)
            .unwrap_or_else(|| "null".to_string())
    ));
    let mode = if cache_write {
        CacheMode::WarmReadWrite
    } else {
        CacheMode::WarmReadOnly
    };
    Ok(Resolution {
        build_class: build_class.to_string(),
        mode,
        reasons,
    })
}

/// Return the first fail-closed path-shape/readability reason for a mounted
/// cache identity file, or `None` when the path is a usable regular file.
pub fn identity_path_unusable(path: Option<&str>) -> Option<&'static str> {
    let Some(path) = path.filter(|path| !path.trim().is_empty()) else {
        return Some("absent");
    };
    if path.contains(['\n', '\r']) {
        return Some("contains a newline");
    }
    let path = Path::new(path);
    if !path.is_absolute() {
        return Some("is not absolute");
    }
    let Ok(metadata) = fs::metadata(path) else {
        return Some("is unreadable");
    };
    if !metadata.is_file() || metadata.len() == 0 {
        return Some("is not a non-empty regular file");
    }
    if fs::File::open(path).is_err() {
        return Some("is unreadable");
    }
    None
}

/// Downgrade a warm resolution to declared cold when its mounted identity path
/// is absent, path-invalid, or unreadable. This check runs before the controller
/// emits any Buck2 RE config. PEM/key parse or correspondence faults remain hard
/// errors in the controller: only path availability is a cold fallback.
pub fn require_usable_identity_or_bypass(
    mut resolution: Resolution,
    client_cert: Option<&str>,
    tls_ca_certs: Option<&str>,
) -> Resolution {
    if resolution.mode == CacheMode::Bypass {
        return resolution;
    }
    let client_finding = identity_path_unusable(client_cert);
    let ca_finding = identity_path_unusable(tls_ca_certs);
    if client_finding.is_some() || ca_finding.is_some() {
        resolution.mode = CacheMode::Bypass;
        resolution.reasons.push(format!(
            "warm cache identity unavailable: client certificate {}; server CA {} — declared \
             cold before Buck2 startup",
            client_finding.unwrap_or("usable"),
            ca_finding.unwrap_or("usable")
        ));
    }
    resolution
}

/// Materialize the effective project configuration Buck2 actually reads at daemon
/// startup. `--config*` is deliberately not emitted: Buck2 does not apply those
/// flags to `buck2_re_client`, so doing so would produce an inert warm-cache claim.
pub fn effective_buckconfig(
    resolution: &Resolution,
    overlay: &str,
    endpoints: Option<&CacheEndpointProfile>,
    client_cert: Option<&str>,
    tls_ca_certs: Option<&str>,
) -> Result<Option<String>, String> {
    match resolution.mode {
        CacheMode::Bypass => Ok(None),
        CacheMode::WarmReadOnly | CacheMode::WarmReadWrite => {
            let endpoints = endpoints.ok_or_else(|| {
                "warm mode requires a validated cache endpoint profile".to_string()
            })?;
            let cert = client_cert.filter(|c| !c.trim().is_empty()).ok_or_else(|| {
                format!(
                    "warm mode `{}` requires the keyed mTLS client identity: set {CLIENT_CERT_ENV} \
                     to the secret-mounted client certificate path (founder 2026-05-30 keyed-auth \
                     posture; enforcement lives at the CAS service boundary)",
                    resolution.mode
                )
            })?;
            let ca = tls_ca_certs
                .filter(|path| !path.trim().is_empty())
                .ok_or_else(|| {
                    format!(
                        "warm mode `{}` requires the NativeLink server CA: set \
                         {TLS_CA_CERTS_ENV} to its secret-mounted path",
                        resolution.mode
                    )
                })?;
            if !Path::new(cert).is_absolute() || !Path::new(ca).is_absolute() {
                return Err("cache TLS certificate paths must be absolute".to_string());
            }
            if [cert, ca].iter().any(|value| value.contains(['\n', '\r'])) {
                return Err("cache TLS paths must not contain newlines".to_string());
            }
            if !overlay.contains(RE_ADDRESS_TOKEN) || !overlay.contains(INSTANCE_NAME_TOKEN) {
                return Err(
                    "warm overlay missing endpoint or instance materialization token".to_string(),
                );
            }
            let mut identity = format!("[buck2_re_client]\n  tls_client_cert = {cert}\n");
            identity.push_str(&format!("  tls_ca_certs = {ca}\n"));
            let binding = cache_mode_binding(endpoints, resolution.mode)
                .ok_or_else(|| "warm mode has no endpoint binding".to_string())?;
            let mut offset = 0;
            let mut config = None;
            for line in overlay.split_inclusive('\n') {
                if line.trim() == "[buck2_re_client]" {
                    let end = offset + line.len();
                    config = Some(format!(
                        "{}{}{}",
                        &overlay[..offset],
                        identity,
                        &overlay[end..]
                    ));
                    break;
                }
                offset += line.len();
            }
            let config = config
                .ok_or_else(|| "warm overlay missing [buck2_re_client] section".to_string())?
                .replace(RE_ADDRESS_TOKEN, &binding.endpoint.re_address)
                .replace(INSTANCE_NAME_TOKEN, &endpoints.instance_name);
            if config.contains("__CACHE_") {
                return Err("warm config retains an unmaterialized cache token".to_string());
            }
            Ok(Some(config))
        }
    }
}

/// Create the ignored machine-local project config without clobbering a human or
/// sibling controller's file. Warm credentials make 0600 mandatory.
pub fn install_local_buckconfig(root: &Path, contents: &str) -> Result<PathBuf, String> {
    let path = root.join(".buckconfig.local");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    return Err("warm-cache controller requires Unix mode-0600 file semantics".to_string());

    #[cfg(unix)]
    {
        let mut file = options
            .open(&path)
            .map_err(|error| format!("create {} without clobbering: {error}", path.display()))?;
        file.write_all(contents.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("write {}: {error}", path.display()))?;
        Ok(path)
    }
}

pub fn remove_local_buckconfig(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|error| format!("remove {}: {error}", path.display()))
}

/// Create a credential file without ever exposing group/other permissions.
pub fn write_private_file(path: &Path, contents: &str) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    return Err("cache identity requires Unix mode-0600 file semantics".to_string());

    #[cfg(unix)]
    {
        let mut file = options
            .open(path)
            .map_err(|error| format!("create {} without clobbering: {error}", path.display()))?;
        file.write_all(contents.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("write {}: {error}", path.display()))
    }
}

/// Locate the InvocationRecord payload inside a buck2
/// `--unstable-write-invocation-record` JSON document.
pub fn invocation_record(doc: &Value) -> Result<&Value, String> {
    doc.pointer("/data/Record/data/InvocationRecord")
        .ok_or_else(|| "no /data/Record/data/InvocationRecord in record JSON".to_string())
}

fn record_u64(record: &Value, key: &str) -> u64 {
    record.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn required_u64(record: &Value, key: &str, findings: &mut Vec<String>) -> Option<u64> {
    match record.get(key).and_then(Value::as_u64) {
        Some(value) => Some(value),
        None => {
            findings.push(format!(
                "record-shape violation: counter `{key}` missing from the invocation record — \
                 cannot prove warm cache behavior from an unrecognized record shape (fail closed)"
            ));
            None
        }
    }
}

fn required_last_snapshot_u64(
    record: &Value,
    key: &str,
    findings: &mut Vec<String>,
) -> Option<u64> {
    let pointer = format!("/last_snapshot/{key}");
    match record.pointer(&pointer).and_then(Value::as_u64) {
        Some(value) => Some(value),
        None => {
            findings.push(format!(
                "record-shape violation: last_snapshot.{key} missing — cannot prove warm cache \
                 behavior from an unrecognized record shape (fail closed)"
            ));
            None
        }
    }
}

fn expect_record_u64(
    record: &Value,
    key: &str,
    expected: u64,
    proof: &str,
    findings: &mut Vec<String>,
) {
    match required_u64(record, key, findings) {
        Some(value) if value == expected => {}
        Some(value) => findings.push(format!(
            "{proof} violation: {key}={value} (expected {expected})"
        )),
        None => {}
    }
}

fn expect_snapshot_u64(
    record: &Value,
    key: &str,
    expected: u64,
    proof: &str,
    findings: &mut Vec<String>,
) {
    match required_last_snapshot_u64(record, key, findings) {
        Some(value) if value == expected => {}
        Some(value) => findings.push(format!(
            "{proof} violation: last_snapshot.{key}={value} (expected {expected})"
        )),
        None => {}
    }
}

fn assert_record_success(record: &Value, proof: &str, findings: &mut Vec<String>) {
    match record.get("exit_result_name").and_then(Value::as_str) {
        Some("SUCCESS") => {}
        Some(value) => findings.push(format!(
            "{proof} violation: non-success exit_result_name={value:?} (expected SUCCESS)"
        )),
        None => findings.push(
            "record-shape violation: exit_result_name missing or non-string (fail closed)"
                .to_string(),
        ),
    }
    expect_record_u64(record, "run_command_failure_count", 0, proof, findings);
    match record.get("errors").and_then(Value::as_array) {
        Some(errors) if errors.is_empty() => {}
        Some(errors) => findings.push(format!(
            "{proof} violation: errors contains {} entry/entries (expected empty)",
            errors.len()
        )),
        None => findings
            .push("record-shape violation: errors missing or non-array (fail closed)".to_string()),
    }
    match record.get("daemon_connection_failure").and_then(Value::as_bool) {
        Some(false) => {}
        Some(true) => findings.push(format!(
            "{proof} violation: daemon_connection_failure=true (expected false)"
        )),
        None => findings.push(
            "record-shape violation: daemon_connection_failure missing or non-boolean (fail closed)"
                .to_string(),
        ),
    }
}

fn assert_no_remote_execution(record: &Value, proof: &str, findings: &mut Vec<String>) {
    for key in [
        "re_executes_started",
        "re_executes_finished_successfully",
        "re_executes_finished_with_error",
    ] {
        expect_snapshot_u64(record, key, 0, proof, findings);
    }
}

/// Build the structured per-lane cache-hit report (the audit's missing-SLO item):
/// action-cache hits, local/remote executions, upload counts, and buck2's own
/// cache_hit_rate, labeled with the lane's build class + resolved mode.
pub fn cache_hit_report(record: &Value, build_class: &str, mode: &str) -> Value {
    let field = |key: &str| record.get(key).cloned().unwrap_or(Value::Null);
    let snapshot = |key: &str| {
        record
            .get("last_snapshot")
            .and_then(|value| value.get(key))
            .cloned()
            .unwrap_or(Value::Null)
    };
    json!({
        "schema": CACHE_HIT_REPORT_SCHEMA,
        "adr": "ADR-0560",
        "build_class": build_class,
        "mode": mode,
        "cache_hit_rate": record.get("cache_hit_rate").and_then(Value::as_f64).unwrap_or(0.0),
        "run_action_cache_count": record_u64(record, "run_action_cache_count"),
        "run_local_count": record_u64(record, "run_local_count"),
        "run_remote_count": record_u64(record, "run_remote_count"),
        "run_skipped_count": record_u64(record, "run_skipped_count"),
        "cache_upload_attempt_count": record_u64(record, "cache_upload_attempt_count"),
        "cache_upload_count": record_u64(record, "cache_upload_count"),
        "exit_result_name": record.get("exit_result_name").and_then(Value::as_str).unwrap_or(""),
        "run_command_failure_count": field("run_command_failure_count"),
        "daemon_connection_failure": field("daemon_connection_failure"),
        "errors": field("errors"),
        "re_upload_bytes": field("re_upload_bytes"),
        "re_download_bytes": field("re_download_bytes"),
        "command_duration_us": field("command_duration_us"),
        "client_walltime_us": field("client_walltime_us"),
        "critical_path_duration_us": field("critical_path_duration_us"),
        "time_to_first_action_execution_ms": field("time_to_first_action_execution_ms"),
        "buck2_revision": record.pointer("/metadata/strings/buck2_revision").cloned().unwrap_or(Value::Null),
        "last_snapshot": {
            "re_action_cache_finished_successfully": snapshot("re_action_cache_finished_successfully"),
            "re_action_cache_finished_with_error": snapshot("re_action_cache_finished_with_error"),
            "re_uploads_finished_successfully": snapshot("re_uploads_finished_successfully"),
            "re_uploads_finished_with_error": snapshot("re_uploads_finished_with_error"),
            "re_downloads_finished_successfully": snapshot("re_downloads_finished_successfully"),
            "re_downloads_finished_with_error": snapshot("re_downloads_finished_with_error"),
            "re_executes_finished_successfully": snapshot("re_executes_finished_successfully"),
            "re_executes_finished_with_error": snapshot("re_executes_finished_with_error"),
            "re_write_action_results_finished_successfully": snapshot("re_write_action_results_finished_successfully"),
            "re_write_action_results_finished_with_error": snapshot("re_write_action_results_finished_with_error"),
        },
    })
}

/// Assert a build invocation succeeded and, when run in warm mode, actually
/// participated in the warm cache and observed at least one action-cache hit.
///
/// This is deliberately stricter than telemetry. A warm lane with a 0% hit rate
/// is usually an endpoint / credential / keying misconfiguration; allowing that
/// to stay green recreates the "cache exists but never hits" false-green class.
/// Bypass/cold modes prove zero cache participation with the cold guard; a
/// successful child is not enough because a stale daemon could otherwise make
/// a declared-cold run participate remotely.
pub fn assert_warm_cache_participation(
    record: &Value,
    build_class: &str,
    mode: &str,
) -> Result<(), Vec<String>> {
    let normalized = mode.trim().to_ascii_lowercase();
    let bypass_mode = matches!(normalized.as_str(), "bypass" | "cold" | "off" | "disabled");
    let warm_mode = matches!(
        normalized.as_str(),
        "warm-ro" | "warm-rw" | "warm-read-only" | "warm-read-write"
    );
    if bypass_mode {
        return assert_cold(record);
    }
    let mut findings = Vec::new();
    assert_record_success(record, "warm cache guard", &mut findings);
    if !bypass_mode && !warm_mode {
        findings.push(format!(
            "cache mode `{mode}` for class `{build_class}` is not recognized — refusing to \
             infer warm-cache correctness from an unknown mode (fail closed)"
        ));
    }

    let cache_hit_rate = match record.get("cache_hit_rate").and_then(Value::as_f64) {
        Some(value) if value.is_finite() && value >= 0.0 => Some(value),
        Some(value) => {
            findings.push(format!(
                "record-shape violation: cache_hit_rate={value:?} is not a finite non-negative \
                 number (fail closed)"
            ));
            None
        }
        None => {
            findings.push(
                "record-shape violation: cache_hit_rate missing from the invocation record — \
                 cannot prove warm cache hit rate (fail closed)"
                    .to_string(),
            );
            None
        }
    };
    let action_hits = required_u64(record, "run_action_cache_count", &mut findings);
    let _local_actions = required_u64(record, "run_local_count", &mut findings);
    let _remote_actions = required_u64(record, "run_remote_count", &mut findings);
    let action_cache_started =
        required_last_snapshot_u64(record, "re_action_cache_started", &mut findings);

    if warm_mode {
        if action_cache_started == Some(0) {
            findings.push(format!(
                "warm cache guard for class `{build_class}` ran in mode `{mode}` but \
                 last_snapshot.re_action_cache_started=0 — the action cache did not start"
            ));
        }
        if matches!(cache_hit_rate, Some(0.0)) {
            findings.push(format!(
                "warm cache guard for class `{build_class}` ran in mode `{mode}` with 0% hit rate \
                 — this is an obvious warm-cache misconfiguration, not a green presubmit"
            ));
        }
        if action_hits == Some(0) {
            findings.push(format!(
                "warm cache guard for class `{build_class}` ran in mode `{mode}` with \
                 run_action_cache_count=0 — no action-cache hit was observed"
            ));
        }
    }

    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

/// Require the integrity probe to be fully served by the action cache. One hit
/// alongside local rebuilds proves connectivity, not coverage, and cannot
/// license fleet-wide warm reads.
pub fn assert_complete_warm_cache_coverage(record: &Value) -> Result<(), Vec<String>> {
    let mut findings = assert_warm_cache_participation(record, "integrity-canary", "warm-ro")
        .err()
        .unwrap_or_default();

    assert_no_remote_execution(record, "reader warm proof", &mut findings);

    match record.get("cache_hit_rate").and_then(Value::as_f64) {
        Some(1.0) => {}
        Some(rate) if rate.is_finite() && rate >= 0.0 => findings.push(format!(
            "incomplete warm coverage: cache_hit_rate={rate} (expected 1.0)"
        )),
        _ => {}
    }
    for key in ["run_local_count", "run_remote_count"] {
        match record.get(key).and_then(Value::as_u64) {
            Some(0) => {}
            Some(value) => findings.push(format!(
                "incomplete warm coverage: {key}={value} (expected 0; every eligible action must be an action-cache hit)"
            )),
            None => {}
        }
    }
    for key in ["cache_upload_attempt_count", "cache_upload_count"] {
        match record.get(key).and_then(Value::as_u64) {
            Some(0) => {}
            Some(value) => findings.push(format!(
                "read-only warm probe violation: {key}={value} (expected 0)"
            )),
            None => findings.push(format!(
                "record-shape violation: counter `{key}` missing from the invocation record (fail closed)"
            )),
        }
    }
    for key in [
        "re_uploads_started",
        "re_uploads_finished_successfully",
        "re_uploads_finished_with_error",
        "re_downloads_finished_with_error",
        "re_action_cache_finished_with_error",
        "re_write_action_results_started",
        "re_write_action_results_finished_successfully",
        "re_write_action_results_finished_with_error",
    ] {
        match record
            .get("last_snapshot")
            .and_then(|snapshot| snapshot.get(key))
            .and_then(Value::as_u64)
        {
            Some(0) => {}
            Some(value) => findings.push(format!(
                "read-only warm probe violation: last_snapshot.{key}={value} (expected 0)"
            )),
            None => findings.push(format!(
                "record-shape violation: last_snapshot.{key} missing (fail closed)"
            )),
        }
    }
    match required_last_snapshot_u64(
        record,
        "re_action_cache_finished_successfully",
        &mut findings,
    ) {
        Some(value) if value > 0 => {}
        Some(value) => findings.push(format!(
            "warm probe did not complete an action-cache lookup: last_snapshot.re_action_cache_finished_successfully={value} (expected >0)"
        )),
        None => {}
    }
    expect_record_u64(
        record,
        "re_upload_bytes",
        0,
        "reader warm proof",
        &mut findings,
    );
    match required_u64(record, "re_download_bytes", &mut findings) {
        Some(value) if value > 0 => {}
        Some(value) => findings.push(format!(
            "warm probe did not download from CAS: re_download_bytes={value} (expected >0)"
        )),
        None => {}
    }
    for key in ["re_downloads_finished_successfully"] {
        match record
            .get("last_snapshot")
            .and_then(|snapshot| snapshot.get(key))
            .and_then(Value::as_u64)
        {
            Some(value) if value > 0 => {}
            Some(value) => findings.push(format!(
                "warm probe did not download from CAS: last_snapshot.{key}={value} (expected >0)"
            )),
            None => findings.push(format!(
                "record-shape violation: last_snapshot.{key} missing (fail closed)"
            )),
        }
    }

    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

/// Require the seed build to execute locally and successfully upload its outputs
/// without remote execution or cache transport errors.
pub fn assert_writer_seed_record(record: &Value) -> Result<(), Vec<String>> {
    let mut findings = Vec::new();
    assert_record_success(record, "writer seed", &mut findings);
    assert_no_remote_execution(record, "writer seed", &mut findings);
    match record.get("cache_hit_rate").and_then(Value::as_f64) {
        Some(0.0) => {}
        Some(value) => findings.push(format!(
            "writer seed violation: cache_hit_rate={value} (expected 0.0)"
        )),
        None => findings.push(
            "record-shape violation: cache_hit_rate missing or non-number (fail closed)"
                .to_string(),
        ),
    }
    for (key, expected_positive) in [
        ("run_action_cache_count", false),
        ("run_local_count", true),
        ("run_remote_count", false),
        ("cache_upload_attempt_count", true),
        ("cache_upload_count", true),
    ] {
        match record.get(key).and_then(Value::as_u64) {
            Some(value) if expected_positive && value > 0 => {}
            Some(0) if !expected_positive => {}
            Some(value) => findings.push(format!(
                "writer seed counter {key}={value} (expected {})",
                if expected_positive { ">0" } else { "0" }
            )),
            None => findings.push(format!(
                "record-shape violation: counter `{key}` missing from the invocation record (fail closed)"
            )),
        }
    }
    for (key, expected_positive) in [
        ("re_uploads_started", true),
        ("re_uploads_finished_successfully", true),
        ("re_uploads_finished_with_error", false),
        ("re_downloads_finished_with_error", false),
        ("re_action_cache_finished_with_error", false),
    ] {
        match record
            .get("last_snapshot")
            .and_then(|snapshot| snapshot.get(key))
            .and_then(Value::as_u64)
        {
            Some(value) if expected_positive && value > 0 => {}
            Some(0) if !expected_positive => {}
            Some(value) => findings.push(format!(
                "writer seed counter last_snapshot.{key}={value} (expected {})",
                if expected_positive { ">0" } else { "0" }
            )),
            None => findings.push(format!(
                "record-shape violation: last_snapshot.{key} missing (fail closed)"
            )),
        }
    }
    match required_u64(record, "re_upload_bytes", &mut findings) {
        Some(value) if value > 0 => {}
        Some(value) => findings.push(format!(
            "writer seed counter re_upload_bytes={value} (expected >0)"
        )),
        None => {}
    }
    expect_record_u64(record, "re_download_bytes", 0, "writer seed", &mut findings);
    for (key, expected_positive) in [
        ("re_write_action_results_started", true),
        ("re_write_action_results_finished_successfully", true),
        ("re_write_action_results_finished_with_error", false),
    ] {
        match required_last_snapshot_u64(record, key, &mut findings) {
            Some(value) if expected_positive && value > 0 => {}
            Some(0) if !expected_positive => {}
            Some(value) => findings.push(format!(
                "writer seed counter last_snapshot.{key}={value} (expected {})",
                if expected_positive { ">0" } else { "0" }
            )),
            None => {}
        }
    }
    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

/// Assert a build had ZERO cache participation (the canary's from-empty proof):
/// no action-cache hits, no remote executions, no upload attempts.
///
/// FAIL-CLOSED on shape drift: a counter that is MISSING from the record is a
/// finding, not a zero — otherwise a buck2 upgrade that renames a field would
/// silently turn the cold proof vacuous (assert-everything-absent == assert
/// nothing).
pub fn assert_cold(record: &Value) -> Result<(), Vec<String>> {
    let mut findings = Vec::new();
    assert_record_success(record, "cold proof", &mut findings);
    assert_no_remote_execution(record, "cold proof", &mut findings);
    match record.get("cache_hit_rate").and_then(Value::as_f64) {
        Some(0.0) => {}
        Some(value) => findings.push(format!(
            "cold violation: cache_hit_rate={value} (expected 0.0)"
        )),
        None => findings.push(
            "record-shape violation: cache_hit_rate missing or non-number (fail closed)"
                .to_string(),
        ),
    }
    match required_u64(record, "run_local_count", &mut findings) {
        Some(value) if value > 0 => {}
        Some(value) => findings.push(format!(
            "cold violation: run_local_count={value} (expected >0)"
        )),
        None => {}
    }
    for key in [
        "run_action_cache_count",
        "run_remote_count",
        "run_remote_dep_file_cache_count",
        "cache_upload_attempt_count",
        "cache_upload_count",
        "dep_file_upload_attempt_count",
        "dep_file_upload_count",
    ] {
        match record.get(key).and_then(Value::as_u64) {
            Some(0) => {}
            Some(v) => findings.push(format!(
                "cold violation: {key}={v} (expected 0 — the canary build must not touch any cache)"
            )),
            None => findings.push(format!(
                "record-shape violation: counter `{key}` missing from the invocation record — \
                 cannot prove coldness from an unrecognized record shape (fail closed)"
            )),
        }
    }
    for key in [
        "re_action_cache_started",
        "re_action_cache_finished_successfully",
        "re_action_cache_finished_with_error",
        "re_uploads_started",
        "re_uploads_finished_successfully",
        "re_uploads_finished_with_error",
        "re_downloads_started",
        "re_downloads_finished_successfully",
        "re_downloads_finished_with_error",
        "re_write_action_results_started",
        "re_write_action_results_finished_successfully",
        "re_write_action_results_finished_with_error",
        "re_get_digest_expirations_started",
        "re_get_digest_expirations_finished_successfully",
        "re_get_digest_expirations_finished_with_error",
        "re_materializes_started",
        "re_materializes_finished_successfully",
        "re_materializes_finished_with_error",
    ] {
        expect_snapshot_u64(record, key, 0, "cold proof", &mut findings);
    }
    expect_record_u64(record, "re_upload_bytes", 0, "cold proof", &mut findings);
    expect_record_u64(record, "re_download_bytes", 0, "cold proof", &mut findings);
    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

fn hash_file(hasher: &mut Sha256, path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(&bytes);
    Ok(())
}

fn walk_sorted(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("read_dir entry under {}: {e}", dir.display()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            walk_sorted(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// SHA-256 over an output artifact: a file hashes (len || bytes); a directory
/// hashes every file in sorted relative order as (relpath || 0 || len || bytes),
/// so the digest is deterministic across hosts.
pub fn hash_output(path: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();
    if path.is_dir() {
        let mut files = Vec::new();
        walk_sorted(path, &mut files)?;
        for file in files {
            let rel = file
                .strip_prefix(path)
                .map_err(|e| format!("strip_prefix {}: {e}", file.display()))?;
            hasher.update(rel.to_string_lossy().as_bytes());
            hasher.update([0u8]);
            hash_file(&mut hasher, &file)?;
        }
    } else {
        hash_file(&mut hasher, path)?;
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Parse `buck2 build --show-full-output` lines (`<target> <path>`) into a
/// target -> output-digest manifest.
pub fn digest_manifest_from_show_output(text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut entries = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((target, path)) = line.split_once(' ') else {
            return Err(format!(
                "malformed --show-full-output line (no space): {line}"
            ));
        };
        let digest = hash_output(Path::new(path.trim()))?;
        entries.insert(target.to_string(), digest);
    }
    Ok(entries)
}

pub fn manifest_to_json(entries: &BTreeMap<String, String>) -> Value {
    json!({
        "schema": DIGEST_MANIFEST_SCHEMA,
        "adr": "ADR-0560",
        "entries": entries,
    })
}

pub fn manifest_from_json(doc: &Value) -> Result<BTreeMap<String, String>, String> {
    let entries = doc
        .get("entries")
        .and_then(Value::as_object)
        .ok_or_else(|| "manifest missing `entries` object".to_string())?;
    let mut out = BTreeMap::new();
    for (k, v) in entries {
        let digest = v
            .as_str()
            .ok_or_else(|| format!("manifest entry `{k}` is not a string digest"))?;
        out.insert(k.clone(), digest.to_string());
    }
    Ok(out)
}

/// Canary verdict states. Anything other than `Green` licenses NOTHING.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanaryStatus {
    /// Every cold output key is present in the warm manifest and byte-identical.
    Green,
    /// At least one overlapping key DIVERGES: hermeticity/non-determinism defect,
    /// fail closed (ADR-0556 D2 RED response: suspend all warm reads fleet-wide).
    Red,
    /// No warm substrate endpoint is live: nothing to verify, warmth stays
    /// unlicensed (the slice-1 dark state) — explicitly NOT green.
    InactiveNoEndpoint,
    /// A warm manifest exists but shares zero keys with the cold build: nothing
    /// was verified, so GREEN is refused (an empty comparison must never license).
    UnverifiedEmptyOverlap,
    /// Some cold outputs were absent from the warm manifest. Partial overlap
    /// cannot license the full pinned target set.
    UnverifiedIncompleteCoverage,
}

impl CanaryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CanaryStatus::Green => "GREEN",
            CanaryStatus::Red => "RED",
            CanaryStatus::InactiveNoEndpoint => "INACTIVE_NO_ENDPOINT",
            CanaryStatus::UnverifiedEmptyOverlap => "UNVERIFIED_EMPTY_OVERLAP",
            CanaryStatus::UnverifiedIncompleteCoverage => "UNVERIFIED_INCOMPLETE_COVERAGE",
        }
    }

    /// Process exit semantics: ONLY `Green` exits zero.
    ///
    /// The canary's exit code is what becomes the scheduled job's check
    /// conclusion, and that conclusion is a declared result channel of the
    /// workflow (ADR-0556 D4.3: "result via API/artifact, never a log someone
    /// shells in to read"). So a non-Green verdict that exits zero publishes a
    /// GREEN badge for a run that verified nothing about cache integrity —
    /// measured on run 30690156857 (2026-08-01): warm-probe SKIPPED, verdict
    /// INACTIVE_NO_ENDPOINT, job conclusion `success`.
    ///
    /// Every unverified status gets the same exit semantics. This makes the type's
    /// own contract ("anything other than `Green` licenses NOTHING") mechanically
    /// true instead of merely documented.
    pub fn is_failure(self) -> bool {
        !matches!(self, CanaryStatus::Green)
    }
}

/// Compare the cold build's digests against the warm substrate's digests for the
/// same keys (ADR-0556 D2). `warm: None` = no live endpoint (INACTIVE).
pub fn canary_verdict(
    cold: &BTreeMap<String, String>,
    warm: Option<&BTreeMap<String, String>>,
) -> (CanaryStatus, Value) {
    let Some(warm) = warm else {
        let status = CanaryStatus::InactiveNoEndpoint;
        return (
            status,
            json!({
                "schema": CANARY_VERDICT_SCHEMA,
                "adr": "ADR-0560",
                "status": status.as_str(),
                "cold_keys": cold.len(),
                "compared_keys": 0,
                "divergent_keys": [],
                "note": "no live CAS endpoint — nothing to verify; warm stays unlicensed \
                         (ADR-0556 D2 clause (b) unsatisfied); this is the slice-1 dark state",
            }),
        );
    };

    let mut divergent = Vec::new();
    let mut compared = 0usize;
    for (key, cold_digest) in cold {
        if let Some(warm_digest) = warm.get(key) {
            compared += 1;
            if warm_digest != cold_digest {
                divergent.push(json!({
                    "key": key,
                    "cold": cold_digest,
                    "warm": warm_digest,
                }));
            }
        }
    }
    let uncovered = cold.len() - compared;
    let status = if !divergent.is_empty() {
        CanaryStatus::Red
    } else if compared == 0 {
        CanaryStatus::UnverifiedEmptyOverlap
    } else if uncovered != 0 {
        CanaryStatus::UnverifiedIncompleteCoverage
    } else {
        CanaryStatus::Green
    };
    let verdict = json!({
        "schema": CANARY_VERDICT_SCHEMA,
        "adr": "ADR-0560",
        "status": status.as_str(),
        "cold_keys": cold.len(),
        "compared_keys": compared,
        "uncovered_cold_keys": uncovered,
        "divergent_keys": divergent,
        "red_response": if status == CanaryStatus::Red {
            "ADR-0556 D2: suspend ALL warm reads fleet-wide — flip \
             specs/cache-warm-license.json warm_reads_licensed to false before any \
             remediation; evict divergent keys; open a friction-ledger row"
        } else { "" },
    });
    (status, verdict)
}

/// Minimal buckconfig (INI) reader for the conformance gate: sections of
/// `key = value` lines, `#` comments, no continuations (our overlays carry none).
pub fn parse_buckconfig(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut current = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = name.trim().to_string();
            sections.entry(current.clone()).or_default();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            sections
                .entry(current.clone())
                .or_default()
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    sections
}

/// Bundled canary policy DATA (pinned target set + invariants). Embedded per the
/// gate-app pattern; the file is part of the crate's srcs.
pub const CANARY_POLICY: &str = include_str!("canary-policy.json");

pub fn canary_policy() -> Result<Value, String> {
    serde_json::from_str(CANARY_POLICY)
        .map_err(|e| format!("parse bundled canary-policy.json: {e}"))
}

/// Walk up from `start` to the repo root (the standing live-corpus pattern).
pub fn repo_root_from(start: &Path) -> Option<std::path::PathBuf> {
    let mut dir = start.to_path_buf();
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_fixture() -> Value {
        json!({
            "trust_invariant": { "canary_build_class": "integrity-canary" },
            "default_for_unlisted_classes": {
                "warmth": "cold", "cache_read": false, "cache_write": false
            },
            "build_classes": {
                "integrity-canary": { "warmth": "cold", "cache_read": false, "cache_write": false },
                "release-production-image": { "warmth": "cold", "cache_read": false, "cache_write": false },
                "dev-agentic-iteration": { "warmth": "warm", "cache_read": true, "cache_write": true },
                "future-read-only-class": { "warmth": "warm", "cache_read": true, "cache_write": false }
            }
        })
    }

    fn license(licensed: bool) -> Value {
        json!({ "warm_reads_licensed": licensed, "reason": "fixture" })
    }

    fn endpoint_policy_fixture() -> Value {
        json!({
            "policy_id": "cache-endpoints",
            "schema_version": "1.0.0",
            "adr": "ADR-0703",
            "active_profile": "oyatie",
            "profiles": {
                "oyatie": {
                    "instance_name": "main",
                    "writer": {
                        "re_address": "grpc://writer.example.test:50051",
                        "socket_address": "writer.example.test:50051"
                    },
                    "reader": {
                        "re_address": "grpc://reader.example.test:50052",
                        "socket_address": "reader.example.test:50052"
                    }
                }
            }
        })
    }

    fn endpoint_profile_fixture() -> CacheEndpointProfile {
        parse_endpoint_profile(&endpoint_policy_fixture(), "oyatie").unwrap()
    }

    #[test]
    fn endpoint_profile_accepts_matching_grpc_and_socket_addresses() {
        let profile = parse_endpoint_profile(&endpoint_policy_fixture(), "oyatie").unwrap();
        assert_eq!(profile.instance_name(), "main");
        assert_eq!(
            profile.writer().socket_address(),
            "writer.example.test:50051"
        );
        assert_eq!(
            profile.reader().re_address(),
            "grpc://reader.example.test:50052"
        );
    }

    #[test]
    fn endpoint_profile_requires_both_roles() {
        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]
            .as_object_mut()
            .unwrap()
            .remove("reader");
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("role `reader` missing")
        );
    }

    #[test]
    fn endpoint_profile_rejects_invalid_address_grammar() {
        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["writer"]["socket_address"] =
            json!("writer.example.test/path:50051");
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("host:decimal-port grammar")
        );
    }

    #[test]
    fn endpoint_profile_rejects_re_and_socket_mismatch() {
        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["writer"]["re_address"] =
            json!("grpc://other.example.test:50051");
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("disagree")
        );
    }

    #[test]
    fn endpoint_profile_requires_distinct_writer_and_reader_listeners() {
        let mut policy = endpoint_policy_fixture();
        let writer = policy["profiles"]["oyatie"]["writer"].clone();
        policy["profiles"]["oyatie"]["reader"] = writer;
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("distinct listeners")
        );
    }

    #[test]
    fn endpoint_profile_rejects_noncanonical_port_spelling() {
        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["reader"]["re_address"] =
            json!("grpc://reader.example.test:050052");
        policy["profiles"]["oyatie"]["reader"]["socket_address"] =
            json!("reader.example.test:050052");
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("non-canonical port")
        );
    }

    #[test]
    fn endpoint_profile_rejects_empty_instance_name() {
        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["instance_name"] = json!("");
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("missing or empty")
        );
    }

    #[test]
    fn endpoint_profile_rejects_wrong_pack_identity() {
        let mut policy = endpoint_policy_fixture();
        policy["policy_id"] = json!("other");
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("policy_id")
        );
        let mut policy = endpoint_policy_fixture();
        policy["adr"] = json!("ADR-0560");
        assert!(
            parse_endpoint_profile(&policy, "oyatie")
                .unwrap_err()
                .contains("ADR-0703")
        );
    }

    #[test]
    fn endpoint_policy_requires_a_resolvable_active_profile() {
        let mut policy = endpoint_policy_fixture();
        policy
            .as_object_mut()
            .expect("fixture object")
            .remove("active_profile");
        assert!(
            parse_active_endpoint_profile(&policy)
                .unwrap_err()
                .contains("active_profile")
        );

        let mut policy = endpoint_policy_fixture();
        policy["active_profile"] = json!("missing");
        assert!(
            parse_active_endpoint_profile(&policy)
                .unwrap_err()
                .contains("profile `missing`")
        );
    }

    #[test]
    fn endpoint_profile_rejects_every_schema_and_grammar_boundary() {
        let mut cases = Vec::new();

        let mut policy = endpoint_policy_fixture();
        policy["schema_version"] = json!("2.0.0");
        cases.push(("schema", policy, "schema_version"));

        let mut policy = endpoint_policy_fixture();
        policy["profiles"] = json!({});
        cases.push(("missing profile", policy, "profile `oyatie` missing"));

        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["writer"] = json!("not-an-object");
        cases.push(("non-object role", policy, "non-object"));

        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["instance_name"] = json!("bad instance");
        cases.push(("whitespace instance", policy, "must be 1..=255"));

        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["instance_name"] = json!("__CACHE_RE_ADDRESS__");
        cases.push(("token instance", policy, "materialization tokens"));

        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["writer"]["re_address"] = json!("grpc://bad_host:50051");
        policy["profiles"]["oyatie"]["writer"]["socket_address"] = json!("bad_host:50051");
        cases.push(("invalid host", policy, "invalid host"));

        for (name, port, expected) in [
            ("zero port", "0", "non-canonical port"),
            ("non-decimal port", "abc", "invalid port"),
            ("overflow port", "70000", "invalid port"),
        ] {
            let mut policy = endpoint_policy_fixture();
            policy["profiles"]["oyatie"]["writer"]["re_address"] =
                json!(format!("grpc://writer.example.test:{port}"));
            policy["profiles"]["oyatie"]["writer"]["socket_address"] =
                json!(format!("writer.example.test:{port}"));
            cases.push((name, policy, expected));
        }

        let mut policy = endpoint_policy_fixture();
        policy["profiles"]["oyatie"]["writer"]["re_address"] =
            json!("https://writer.example.test:50051");
        cases.push(("wrong scheme", policy, "must use grpc://"));

        for (name, policy, expected) in cases {
            let error = parse_endpoint_profile(&policy, "oyatie").unwrap_err();
            assert!(error.contains(expected), "{name}: {error}");
        }
    }

    #[test]
    fn canary_class_always_bypasses_even_under_a_green_license() {
        let r = resolve(&policy_fixture(), &license(true), "integrity-canary").unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
    }

    #[test]
    fn canary_class_bypasses_even_if_the_data_were_tampered_warm() {
        let mut policy = policy_fixture();
        policy["build_classes"]["integrity-canary"] =
            json!({ "warmth": "warm", "cache_read": true, "cache_write": true });
        let r = resolve(&policy, &license(true), "integrity-canary").unwrap();
        assert_eq!(
            r.mode,
            CacheMode::Bypass,
            "the trust anchor must win over tampered data"
        );
    }

    #[test]
    fn unlisted_class_is_fail_closed_bypass() {
        let r = resolve(&policy_fixture(), &license(true), "no-such-class").unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
    }

    #[test]
    fn cold_class_bypasses_under_a_green_license() {
        let r = resolve(
            &policy_fixture(),
            &license(true),
            "release-production-image",
        )
        .unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
    }

    #[test]
    fn warm_class_is_suspended_while_the_kill_switch_is_false() {
        let r = resolve(&policy_fixture(), &license(false), "dev-agentic-iteration").unwrap();
        assert_eq!(r.mode, CacheMode::Bypass);
        assert!(
            r.reasons
                .iter()
                .any(|m| m.contains("warm_reads_licensed=false"))
        );
    }

    #[test]
    fn warm_class_resolves_rw_only_when_licensed() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        assert_eq!(r.mode, CacheMode::WarmReadWrite);
    }

    #[test]
    fn warm_read_only_class_resolves_ro() {
        let r = resolve(&policy_fixture(), &license(true), "future-read-only-class").unwrap();
        assert_eq!(r.mode, CacheMode::WarmReadOnly);
    }

    #[test]
    fn unusable_identity_downgrades_warm_resolution_to_declared_cold() {
        let warm = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        for (cert, ca) in [
            (None, None),
            (Some("relative.pem"), Some("/absolute/ca.pem")),
            (Some("/bad\ncert.pem"), Some("/absolute/ca.pem")),
            (Some("/does/not/exist.pem"), Some("/also/missing.pem")),
        ] {
            let resolution = require_usable_identity_or_bypass(warm.clone(), cert, ca);
            assert_eq!(resolution.mode, CacheMode::Bypass);
            assert!(resolution.reasons.last().unwrap().contains("declared cold"));
        }
    }

    #[test]
    fn nonempty_readable_identity_files_preserve_warm_resolution() {
        let root =
            std::env::temp_dir().join(format!("oya-cache-identity-path-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let cert = root.join("client.pem");
        let ca = root.join("ca.pem");
        fs::write(&cert, "certificate").unwrap();
        fs::write(&ca, "certificate authority").unwrap();
        let warm = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let resolution = require_usable_identity_or_bypass(warm, cert.to_str(), ca.to_str());
        assert_eq!(resolution.mode, CacheMode::WarmReadWrite);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn empty_identity_file_downgrades_to_declared_cold() {
        let root =
            std::env::temp_dir().join(format!("oya-cache-empty-identity-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let cert = root.join("client.pem");
        let ca = root.join("ca.pem");
        fs::write(&cert, b"").unwrap();
        fs::write(&ca, "certificate authority").unwrap();
        let warm = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let resolution = require_usable_identity_or_bypass(warm, cert.to_str(), ca.to_str());
        assert_eq!(resolution.mode, CacheMode::Bypass);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_license_is_a_loud_error_not_a_grant() {
        let err = resolve(
            &policy_fixture(),
            &json!({ "warm_reads_licensed": "yes" }),
            "dev-agentic-iteration",
        )
        .unwrap_err();
        assert!(err.contains("warm_reads_licensed"));
    }

    #[test]
    fn bypass_materializes_no_local_config() {
        let r = resolve(&policy_fixture(), &license(false), "dev-agentic-iteration").unwrap();
        let after_identity_check =
            require_usable_identity_or_bypass(r.clone(), Some("relative"), None);
        assert_eq!(after_identity_check.mode, CacheMode::Bypass);
        assert_eq!(after_identity_check.reasons, r.reasons);
        assert_eq!(
            effective_buckconfig(&r, "", None, None, None).unwrap(),
            None
        );
    }

    #[test]
    fn warm_emission_without_a_keyed_identity_is_a_hard_error() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let endpoints = endpoint_profile_fixture();
        let err = effective_buckconfig(&r, "", Some(&endpoints), None, None).unwrap_err();
        assert!(err.contains(CLIENT_CERT_ENV));
    }

    #[test]
    fn warm_identity_paths_must_be_absolute() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let endpoints = endpoint_profile_fixture();
        let err = effective_buckconfig(
            &r,
            "[buck2_re_client]\ncas_address = __CACHE_RE_ADDRESS__\ninstance_name = __CACHE_INSTANCE_NAME__\ntls = true\n",
            Some(&endpoints),
            Some("relative/client.pem"),
            Some("/secrets/ca.pem"),
        )
        .unwrap_err();
        assert!(err.contains("absolute"));
    }

    #[test]
    fn warm_emission_without_server_ca_is_a_hard_error() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let endpoints = endpoint_profile_fixture();
        let err = effective_buckconfig(
            &r,
            "[buck2_re_client]\ncas_address = __CACHE_RE_ADDRESS__\ninstance_name = __CACHE_INSTANCE_NAME__\ntls = true\n",
            Some(&endpoints),
            Some("/secrets/client.pem"),
            None,
        )
        .unwrap_err();
        assert!(err.contains(TLS_CA_CERTS_ENV));
    }

    #[test]
    fn warm_overlay_requires_both_materialization_tokens() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let endpoints = endpoint_profile_fixture();
        for overlay in [
            "[buck2_re_client]\ninstance_name = __CACHE_INSTANCE_NAME__\n",
            "[buck2_re_client]\ncas_address = __CACHE_RE_ADDRESS__\n",
        ] {
            assert!(
                effective_buckconfig(
                    &r,
                    overlay,
                    Some(&endpoints),
                    Some("/secrets/client.pem"),
                    Some("/secrets/ca.pem"),
                )
                .unwrap_err()
                .contains("materialization token")
            );
        }
        assert!(
            effective_buckconfig(
                &r,
                "[buck2_re_client]\ncas_address = __CACHE_RE_ADDRESS__\ninstance_name = __CACHE_INSTANCE_NAME__\nother = __CACHE_UNKNOWN__\n",
                Some(&endpoints),
                Some("/secrets/client.pem"),
                Some("/secrets/ca.pem"),
            )
            .unwrap_err()
            .contains("unmaterialized cache token")
        );
    }

    #[test]
    fn warm_rw_effective_config_selects_the_rw_overlay_and_carries_the_identity() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let endpoints = endpoint_profile_fixture();
        let config = effective_buckconfig(
            &r,
            "[buck2_re_client]\ncas_address = __CACHE_RE_ADDRESS__\ninstance_name = __CACHE_INSTANCE_NAME__\ntls = true\n",
            Some(&endpoints),
            Some("/secrets/writer.pem"),
            Some("/secrets/ca.pem"),
        )
        .unwrap()
        .expect("warm config");
        assert!(config.contains("tls_client_cert = /secrets/writer.pem"));
        assert!(config.contains("tls_ca_certs = /secrets/ca.pem"));
        assert!(config.contains("cas_address = grpc://writer.example.test:50051"));
        assert!(config.contains("instance_name = main"));
        assert!(!config.contains("--config"));
    }

    #[test]
    fn effective_config_replaces_only_the_real_client_section_header() {
        let r = resolve(&policy_fixture(), &license(true), "dev-agentic-iteration").unwrap();
        let endpoints = endpoint_profile_fixture();
        let config = effective_buckconfig(
            &r,
            "# [buck2_re_client] is documented here\n[buck2_re_client]\ncas_address = __CACHE_RE_ADDRESS__\ninstance_name = __CACHE_INSTANCE_NAME__\n",
            Some(&endpoints),
            Some("/secrets/writer.pem"),
            Some("/secrets/ca.pem"),
        )
        .unwrap()
        .expect("warm config");
        assert!(config.starts_with("# [buck2_re_client] is documented here\n"));
        assert_eq!(config.matches("tls_client_cert =").count(), 1);
    }

    #[test]
    fn private_local_config_is_mode_0600_and_removed() {
        let root = std::env::temp_dir().join(format!("oya-cache-config-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = install_local_buckconfig(&root, "[buck2_re_client]\ntls = true\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        remove_local_buckconfig(&path).unwrap();
        assert!(!path.exists());
        let identity = root.join("client.pem");
        write_private_file(&identity, "certificate\nprivate-key\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&identity).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert!(write_private_file(&identity, "replacement").is_err());
        fs::remove_dir_all(root).unwrap();
    }

    fn record_fixture(action_cache: u64, remote: u64, uploads: u64) -> Value {
        let cache_hit_rate = if action_cache == 0 { 0.0 } else { 0.5 };
        let mut snapshot = json!({
            "re_action_cache_started": action_cache,
            "re_action_cache_finished_successfully": action_cache,
            "re_action_cache_finished_with_error": 0,
            "re_upload_bytes": if uploads > 0 { 1024 } else { 0 },
            "re_uploads_started": uploads,
            "re_uploads_finished_successfully": uploads,
            "re_uploads_finished_with_error": 0,
            "re_download_bytes": if action_cache > 0 { 1024 } else { 0 },
            "re_downloads_started": action_cache,
            "re_downloads_finished_successfully": action_cache,
            "re_downloads_finished_with_error": 0,
            "re_executes_started": 0,
            "re_executes_finished_successfully": 0,
            "re_executes_finished_with_error": 0,
            "re_write_action_results_started": uploads,
            "re_write_action_results_finished_successfully": uploads,
            "re_write_action_results_finished_with_error": 0
        });
        for key in [
            "re_get_digest_expirations_started",
            "re_get_digest_expirations_finished_successfully",
            "re_get_digest_expirations_finished_with_error",
            "re_materializes_started",
            "re_materializes_finished_successfully",
            "re_materializes_finished_with_error",
        ] {
            snapshot[key] = json!(0);
        }
        let record = json!({
                "cache_hit_rate": cache_hit_rate,
                "run_action_cache_count": action_cache,
                "run_local_count": 7,
                "run_remote_count": remote,
                "run_skipped_count": 1,
                "cache_upload_attempt_count": uploads,
                "cache_upload_count": uploads,
                "dep_file_upload_attempt_count": 0,
                "dep_file_upload_count": 0,
                "run_remote_dep_file_cache_count": 0,
                "re_upload_bytes": if uploads > 0 { 1024 } else { 0 },
                "re_download_bytes": if action_cache > 0 { 1024 } else { 0 },
                "exit_result_name": "SUCCESS",
                "run_command_failure_count": 0,
                "errors": [],
                "daemon_connection_failure": false,
                "command_duration_us": 2000,
                "client_walltime_us": 3000,
                "critical_path_duration_us": 1000,
                "time_to_first_action_execution_ms": 4,
                "metadata": {"strings": {"buck2_revision": "fixture-revision"}},
                "last_snapshot": snapshot
        });
        json!({"data": {"Record": {"data": {"InvocationRecord": record}}}})
    }

    #[test]
    fn cache_hit_report_extracts_structured_counters() {
        let doc = record_fixture(3, 0, 2);
        let record = invocation_record(&doc).unwrap();
        let report = cache_hit_report(record, "dev-agentic-iteration", "warm-rw");
        assert_eq!(report["schema"], CACHE_HIT_REPORT_SCHEMA);
        assert_eq!(report["run_action_cache_count"], 3);
        assert_eq!(report["cache_upload_count"], 2);
        assert_eq!(report["build_class"], "dev-agentic-iteration");
        assert_eq!(report["re_upload_bytes"], 1024);
        assert_eq!(report["re_download_bytes"], 1024);
        assert_eq!(report["run_command_failure_count"], 0);
        assert_eq!(report["daemon_connection_failure"], false);
        assert_eq!(report["errors"], json!([]));
        assert_eq!(report["command_duration_us"], 2000);
        assert_eq!(report["client_walltime_us"], 3000);
        assert_eq!(report["critical_path_duration_us"], 1000);
        assert_eq!(report["time_to_first_action_execution_ms"], 4);
        assert_eq!(report["buck2_revision"], "fixture-revision");
        assert_eq!(
            report["last_snapshot"]["re_action_cache_finished_successfully"],
            3
        );
        assert_eq!(
            report["last_snapshot"]["re_uploads_finished_successfully"],
            2
        );
        assert_eq!(
            report["last_snapshot"]["re_executes_finished_with_error"],
            0
        );
    }

    #[test]
    fn writer_seed_requires_local_execution_and_successful_uploads() {
        let document = record_fixture(0, 0, 2);
        let record = invocation_record(&document).unwrap();
        assert!(assert_writer_seed_record(record).is_ok());

        let mut no_bytes = record.clone();
        no_bytes["re_upload_bytes"] = json!(0);
        let findings = assert_writer_seed_record(&no_bytes).unwrap_err();
        assert!(
            findings
                .iter()
                .any(|finding| finding.contains("re_upload_bytes=0"))
        );
    }

    fn reject_each_mutation(
        base: &Value,
        predicate: fn(&Value) -> Result<(), Vec<String>>,
        mutations: &[(&str, Value)],
    ) {
        for (pointer, value) in mutations {
            let mut record = base.clone();
            *record
                .pointer_mut(pointer)
                .unwrap_or_else(|| panic!("fixture missing {pointer}")) = value.clone();
            assert!(
                predicate(&record).is_err(),
                "predicate accepted prohibited {pointer}={value}"
            );
        }
    }

    #[test]
    fn commissioning_predicates_reject_every_prohibited_error_and_counter() {
        let writer = invocation_record(&record_fixture(0, 0, 2)).unwrap().clone();
        reject_each_mutation(
            &writer,
            assert_writer_seed_record,
            &[
                ("/run_command_failure_count", json!(1)),
                ("/errors", json!(["failure"])),
                ("/daemon_connection_failure", json!(true)),
                ("/run_action_cache_count", json!(1)),
                ("/run_remote_count", json!(1)),
                ("/re_download_bytes", json!(1)),
                ("/last_snapshot/re_executes_started", json!(1)),
                ("/last_snapshot/re_executes_finished_successfully", json!(1)),
                ("/last_snapshot/re_executes_finished_with_error", json!(1)),
                ("/last_snapshot/re_uploads_finished_with_error", json!(1)),
                ("/last_snapshot/re_downloads_finished_with_error", json!(1)),
                (
                    "/last_snapshot/re_action_cache_finished_with_error",
                    json!(1),
                ),
                (
                    "/last_snapshot/re_write_action_results_finished_with_error",
                    json!(1),
                ),
            ],
        );

        let mut reader = invocation_record(&record_fixture(4, 0, 0)).unwrap().clone();
        reader["cache_hit_rate"] = json!(1.0);
        reader["run_local_count"] = json!(0);
        reject_each_mutation(
            &reader,
            assert_complete_warm_cache_coverage,
            &[
                ("/run_command_failure_count", json!(1)),
                ("/errors", json!(["failure"])),
                ("/daemon_connection_failure", json!(true)),
                ("/run_local_count", json!(1)),
                ("/run_remote_count", json!(1)),
                ("/cache_upload_attempt_count", json!(1)),
                ("/cache_upload_count", json!(1)),
                ("/re_upload_bytes", json!(1)),
                ("/last_snapshot/re_executes_started", json!(1)),
                ("/last_snapshot/re_executes_finished_successfully", json!(1)),
                ("/last_snapshot/re_executes_finished_with_error", json!(1)),
                ("/last_snapshot/re_uploads_finished_with_error", json!(1)),
                ("/last_snapshot/re_downloads_finished_with_error", json!(1)),
                (
                    "/last_snapshot/re_action_cache_finished_with_error",
                    json!(1),
                ),
                ("/last_snapshot/re_write_action_results_started", json!(1)),
                (
                    "/last_snapshot/re_write_action_results_finished_successfully",
                    json!(1),
                ),
                (
                    "/last_snapshot/re_write_action_results_finished_with_error",
                    json!(1),
                ),
            ],
        );

        let cold = invocation_record(&record_fixture(0, 0, 0)).unwrap().clone();
        reject_each_mutation(
            &cold,
            assert_cold,
            &[
                ("/run_command_failure_count", json!(1)),
                ("/errors", json!(["failure"])),
                ("/daemon_connection_failure", json!(true)),
                ("/run_action_cache_count", json!(1)),
                ("/run_remote_count", json!(1)),
                ("/run_remote_dep_file_cache_count", json!(1)),
                ("/cache_upload_attempt_count", json!(1)),
                ("/cache_upload_count", json!(1)),
                ("/dep_file_upload_attempt_count", json!(1)),
                ("/dep_file_upload_count", json!(1)),
                ("/last_snapshot/re_action_cache_started", json!(1)),
                (
                    "/last_snapshot/re_action_cache_finished_successfully",
                    json!(1),
                ),
                (
                    "/last_snapshot/re_action_cache_finished_with_error",
                    json!(1),
                ),
                ("/re_upload_bytes", json!(1)),
                ("/last_snapshot/re_uploads_started", json!(1)),
                ("/last_snapshot/re_uploads_finished_successfully", json!(1)),
                ("/last_snapshot/re_uploads_finished_with_error", json!(1)),
                ("/re_download_bytes", json!(1)),
                ("/last_snapshot/re_downloads_started", json!(1)),
                (
                    "/last_snapshot/re_downloads_finished_successfully",
                    json!(1),
                ),
                ("/last_snapshot/re_downloads_finished_with_error", json!(1)),
                ("/last_snapshot/re_executes_started", json!(1)),
                ("/last_snapshot/re_executes_finished_successfully", json!(1)),
                ("/last_snapshot/re_executes_finished_with_error", json!(1)),
                ("/last_snapshot/re_write_action_results_started", json!(1)),
                (
                    "/last_snapshot/re_write_action_results_finished_successfully",
                    json!(1),
                ),
                (
                    "/last_snapshot/re_write_action_results_finished_with_error",
                    json!(1),
                ),
                ("/last_snapshot/re_get_digest_expirations_started", json!(1)),
                (
                    "/last_snapshot/re_get_digest_expirations_finished_successfully",
                    json!(1),
                ),
                (
                    "/last_snapshot/re_get_digest_expirations_finished_with_error",
                    json!(1),
                ),
                ("/last_snapshot/re_materializes_started", json!(1)),
                (
                    "/last_snapshot/re_materializes_finished_successfully",
                    json!(1),
                ),
                (
                    "/last_snapshot/re_materializes_finished_with_error",
                    json!(1),
                ),
            ],
        );
    }

    #[test]
    fn commissioning_predicates_fail_closed_on_current_schema_type_drift() {
        let writer = invocation_record(&record_fixture(0, 0, 2)).unwrap().clone();
        let mut reader = invocation_record(&record_fixture(4, 0, 0)).unwrap().clone();
        reader["cache_hit_rate"] = json!(1.0);
        reader["run_local_count"] = json!(0);
        let cold = invocation_record(&record_fixture(0, 0, 0)).unwrap().clone();
        for (record, predicate) in [
            (
                &writer,
                assert_writer_seed_record as fn(&Value) -> Result<(), Vec<String>>,
            ),
            (&reader, assert_complete_warm_cache_coverage),
            (&cold, assert_cold),
        ] {
            reject_each_mutation(
                record,
                predicate,
                &[
                    ("/run_command_failure_count", json!("0")),
                    ("/errors", json!({})),
                    ("/daemon_connection_failure", json!(0)),
                    ("/last_snapshot/re_executes_started", json!("0")),
                ],
            );
        }
    }

    #[test]
    fn assert_cold_is_green_on_a_zero_participation_record() {
        let doc = record_fixture(0, 0, 0);
        assert!(assert_cold(invocation_record(&doc).unwrap()).is_ok());
    }

    #[test]
    fn assert_warm_allows_explicit_bypass_mode() {
        let doc = record_fixture(0, 0, 0);
        assert!(
            assert_warm_cache_participation(
                invocation_record(&doc).unwrap(),
                "gate-fleet-shared-graph",
                "bypass"
            )
            .is_ok()
        );
    }

    #[test]
    fn assert_warm_bypass_rejects_any_cache_participation() {
        for doc in [
            record_fixture(1, 0, 0),
            record_fixture(0, 1, 0),
            record_fixture(0, 0, 1),
        ] {
            let findings = assert_warm_cache_participation(
                invocation_record(&doc).unwrap(),
                "gate-fleet-shared-graph",
                "bypass",
            )
            .unwrap_err();
            assert!(
                findings
                    .iter()
                    .any(|finding| finding.contains("cold violation"))
            );
        }
    }

    #[test]
    fn assert_warm_still_fails_on_bypass_error_record() {
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "exit_result_name": "FAILURE"
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "bypass",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("non-success")));
    }

    #[test]
    fn assert_warm_accepts_positive_cache_hits() {
        let doc = record_fixture(3, 0, 0);
        assert!(
            assert_warm_cache_participation(
                invocation_record(&doc).unwrap(),
                "gate-fleet-shared-graph",
                "warm-rw"
            )
            .is_ok()
        );
    }

    #[test]
    fn assert_warm_fails_on_zero_hit_warm_mode() {
        let doc = record_fixture(0, 0, 0);
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("0% hit rate")));
        assert!(findings.iter().any(|f| f.contains("did not start")));
    }

    #[test]
    fn assert_warm_rejects_zero_rate_even_with_positive_cache_count() {
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "cache_hit_rate": 0.0,
                "run_action_cache_count": 3,
                "run_local_count": 4,
                "run_remote_count": 0,
                "run_skipped_count": 1,
                "cache_upload_attempt_count": 0,
                "cache_upload_count": 0,
                "exit_result_name": "SUCCESS",
                "last_snapshot": { "re_action_cache_started": 3 }
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("0% hit rate")));
    }

    #[test]
    fn assert_warm_rejects_positive_rate_with_zero_cache_count() {
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "cache_hit_rate": 0.5,
                "run_action_cache_count": 0,
                "run_local_count": 4,
                "run_remote_count": 0,
                "run_skipped_count": 1,
                "cache_upload_attempt_count": 0,
                "cache_upload_count": 0,
                "exit_result_name": "SUCCESS",
                "last_snapshot": { "re_action_cache_started": 1 }
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(
            findings
                .iter()
                .any(|f| f.contains("run_action_cache_count=0"))
        );
    }

    #[test]
    fn assert_warm_fails_closed_on_error_or_missing_shape() {
        let error_doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "cache_hit_rate": 1.0,
                "run_action_cache_count": 1,
                "run_local_count": 0,
                "run_remote_count": 0,
                "exit_result_name": "FAILURE",
                "last_snapshot": { "re_action_cache_started": 1 }
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&error_doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(findings.iter().any(|f| f.contains("non-success")));

        let missing_doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "exit_result_name": "SUCCESS"
            } } } }
        });
        let findings = assert_warm_cache_participation(
            invocation_record(&missing_doc).unwrap(),
            "gate-fleet-shared-graph",
            "warm-rw",
        )
        .unwrap_err();
        assert!(
            findings
                .iter()
                .any(|f| f.contains("record-shape violation"))
        );
    }

    #[test]
    fn assert_cold_fails_closed_on_any_cache_participation() {
        for doc in [
            record_fixture(1, 0, 0),
            record_fixture(0, 1, 0),
            record_fixture(0, 0, 1),
        ] {
            let findings = assert_cold(invocation_record(&doc).unwrap()).unwrap_err();
            assert!(!findings.is_empty());
        }
    }

    #[test]
    fn assert_cold_fails_closed_on_an_unrecognized_record_shape() {
        // A record with the counters RENAMED/missing must not pass: absence is a
        // shape violation, never an implicit zero.
        let doc = json!({
            "data": { "Record": { "data": { "InvocationRecord": {
                "exit_result_name": "SUCCESS"
            } } } }
        });
        let findings = assert_cold(invocation_record(&doc).unwrap()).unwrap_err();
        assert!(
            findings
                .iter()
                .any(|f| f.contains("record-shape violation"))
        );
    }

    fn manifest(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn verdict_inactive_when_no_endpoint() {
        let (status, v) = canary_verdict(&manifest(&[("//a:a", "1")]), None);
        assert_eq!(status, CanaryStatus::InactiveNoEndpoint);
        assert_eq!(v["compared_keys"], 0);
        // RED fixture for the false-green channel: INACTIVE compared ZERO keys, so it must
        // NOT exit zero. Exit code is the scheduled job's check conclusion, and a `success`
        // conclusion on a run that verified nothing publishes a daily green badge for an
        // unestablished trust anchor (observed: run 30690156857, 2026-08-01 — warm-probe
        // skipped, verdict INACTIVE_NO_ENDPOINT, conclusion `success`). Reverting
        // `is_failure` to exclude InactiveNoEndpoint fails here.
        assert!(
            status.is_failure(),
            "INACTIVE_NO_ENDPOINT compared 0 keys; exiting zero would publish a green check \
             conclusion for a run that verified no cache integrity"
        );
    }

    #[test]
    fn only_green_exits_zero() {
        // The type's contract is "anything other than Green licenses NOTHING". Assert it over
        // EVERY variant so a new status added later cannot default into the passing arm.
        for status in [
            CanaryStatus::Red,
            CanaryStatus::InactiveNoEndpoint,
            CanaryStatus::UnverifiedEmptyOverlap,
            CanaryStatus::UnverifiedIncompleteCoverage,
        ] {
            assert!(
                status.is_failure(),
                "{} licenses nothing, so it must not exit zero",
                status.as_str()
            );
        }
        assert!(!CanaryStatus::Green.is_failure());
    }

    #[test]
    fn verdict_green_requires_complete_identical_coverage() {
        let cold = manifest(&[("//a:a", "1"), ("//b:b", "2")]);
        let warm = manifest(&[("//a:a", "1")]);
        let (status, v) = canary_verdict(&cold, Some(&warm));
        assert_eq!(status, CanaryStatus::UnverifiedIncompleteCoverage);
        assert_eq!(v["compared_keys"], 1);
        assert_eq!(v["uncovered_cold_keys"], 1);

        let complete = manifest(&[("//a:a", "1"), ("//b:b", "2")]);
        let (status, _) = canary_verdict(&cold, Some(&complete));
        assert_eq!(status, CanaryStatus::Green);
    }

    #[test]
    fn integrity_probe_rejects_one_hit_plus_local_rebuilds() {
        let partial = invocation_record(&record_fixture(1, 0, 0)).unwrap().clone();
        let mut partial = partial;
        partial["cache_hit_rate"] = json!(0.25);
        partial["run_action_cache_count"] = json!(1);
        partial["run_local_count"] = json!(3);
        let findings = assert_complete_warm_cache_coverage(&partial).unwrap_err();
        assert!(
            findings
                .iter()
                .any(|finding| finding.contains("run_local_count=3"))
        );

        partial["cache_hit_rate"] = json!(1.0);
        partial["run_action_cache_count"] = json!(4);
        partial["run_local_count"] = json!(0);
        assert!(assert_complete_warm_cache_coverage(&partial).is_ok());
    }

    #[test]
    fn verdict_red_on_any_divergent_key() {
        let cold = manifest(&[("//a:a", "1")]);
        let warm = manifest(&[("//a:a", "DIFFERENT")]);
        let (status, v) = canary_verdict(&cold, Some(&warm));
        assert_eq!(status, CanaryStatus::Red);
        assert!(status.is_failure());
        assert_eq!(v["divergent_keys"].as_array().unwrap().len(), 1);
        assert!(
            v["red_response"]
                .as_str()
                .unwrap()
                .contains("suspend ALL warm reads")
        );
    }

    #[test]
    fn verdict_refuses_green_on_empty_overlap() {
        let cold = manifest(&[("//a:a", "1")]);
        let warm = manifest(&[("//z:z", "9")]);
        let (status, _) = canary_verdict(&cold, Some(&warm));
        assert_eq!(status, CanaryStatus::UnverifiedEmptyOverlap);
        assert!(status.is_failure());
    }

    #[test]
    fn digest_manifest_hashes_files_deterministically() {
        let dir =
            std::env::temp_dir().join(format!("oya-cache-wiring-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("out.bin");
        std::fs::write(&file, b"payload").unwrap();
        let text = format!("//x:x {}\n", file.display());
        let a = digest_manifest_from_show_output(&text).unwrap();
        let b = digest_manifest_from_show_output(&text).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn parse_buckconfig_reads_sections_and_keys() {
        let cfg = parse_buckconfig("# c\n[a]\n  k = v\n[b]\nx=y\n");
        assert_eq!(cfg["a"]["k"], "v");
        assert_eq!(cfg["b"]["x"], "y");
    }

    #[test]
    fn bundled_canary_policy_parses_and_pins_targets() {
        let policy = canary_policy().unwrap();
        assert_eq!(policy["build_class"], "integrity-canary");
        assert!(!policy["pinned_targets"].as_array().unwrap().is_empty());
    }
}
