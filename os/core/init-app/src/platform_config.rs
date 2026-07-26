//! Platform-aware machine-config acquisition for early boot.
//!
//! Talos discovers the current platform from `talos.platform=` and then asks
//! that platform for machine-config bytes (for metal this is `talos.config=`,
//! either a URL or the `metal-iso` sentinel). This module wires the existing
//! `talos-platform` model into PID1 while keeping the current initramfs-baked
//! `/machine-config.yaml` as a compatibility fallback for the boot harness.

use std::fmt;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;

use os_platform_domain::aws::{
    Aws, BootstrapNetworkConfig as AwsBootstrapNetworkConfig, TOKEN_HEADER as AWS_IMDS_TOKEN_HEADER,
};
use os_platform_domain::azure::Azure;
use os_platform_domain::gcp::Gcp;
use os_platform_domain::metal::Metal;
use os_platform_domain::nocloud::NoCloud;
use os_platform_domain::{ConfigSource, ConfigStore, Header, Mode, Platform};

use crate::MACHINE_CONFIG_PATH;
use crate::cmdline::CmdLine;

/// Talos kernel parameter selecting the runtime platform.
pub const KERNEL_PARAM_PLATFORM: &str = "talos.platform";

/// operating-system' compatibility default when the minimal boot harness does not pass
/// `talos.platform=`. Upstream Talos errors when this is absent; our direct
/// QEMU/vfkit harness predates platform wiring, so defaulting to metal preserves
/// bootability while still honoring an explicit platform when present.
pub const DEFAULT_PLATFORM: &str = "metal";

/// Upper bound for dependency-light HTTP config downloads in PID1.
const HTTP_MAX_CONFIG_BYTES: u64 = 16 * 1024 * 1024;

/// Short per-attempt timeout for early-boot config HTTP reads.
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);

/// Redirect cap matching Go's conservative default shape without pulling in an
/// HTTP client dependency.
const HTTP_MAX_REDIRECTS: usize = 5;

/// AWS IMDSv2 token endpoint path.
const AWS_IMDS_TOKEN_PATH: &str = "/latest/api/token";

/// AWS IMDSv2 token TTL request header.
const AWS_IMDS_TOKEN_TTL_HEADER: &str = "X-aws-ec2-metadata-token-ttl-seconds";

/// AWS SDK default token TTL shape; long enough for early config fetch.
const AWS_IMDS_TOKEN_TTL_SECONDS: &str = "21600";

/// The supported platform subset currently modeled by `talos-platform` and
/// usable by early PID1 config loading.
#[derive(Debug, Clone)]
pub enum SelectedPlatform {
    Metal(Metal),
    Aws(Aws),
    Gcp(Gcp),
    Azure(Azure),
    NoCloud(NoCloud),
}

impl SelectedPlatform {
    /// Select a platform from parsed kernel cmdline values.
    pub fn from_cmdline(cmdline: &CmdLine) -> Result<Self, ConfigLoadError> {
        match cmdline.platform().unwrap_or(DEFAULT_PLATFORM) {
            "" | "metal" => Ok(SelectedPlatform::Metal(match cmdline.config_source() {
                Some(value) if !value.is_empty() => Metal::from_cmdline(value),
                _ => Metal::new(),
            })),
            "aws" => Ok(SelectedPlatform::Aws(Aws::new())),
            "gcp" => Ok(SelectedPlatform::Gcp(Gcp::new())),
            "azure" => Ok(SelectedPlatform::Azure(Azure::new())),
            "nocloud" => Ok(SelectedPlatform::NoCloud(NoCloud::from_cidata())),
            other => Err(ConfigLoadError::UnknownPlatform(other.to_string())),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SelectedPlatform::Metal(p) => p.name(),
            SelectedPlatform::Aws(p) => p.name(),
            SelectedPlatform::Gcp(p) => p.name(),
            SelectedPlatform::Azure(p) => p.name(),
            SelectedPlatform::NoCloud(p) => p.name(),
        }
    }

    pub fn mode(&self) -> Mode {
        match self {
            SelectedPlatform::Metal(p) => p.mode(),
            SelectedPlatform::Aws(p) => p.mode(),
            SelectedPlatform::Gcp(p) => p.mode(),
            SelectedPlatform::Azure(p) => p.mode(),
            SelectedPlatform::NoCloud(p) => p.mode(),
        }
    }

    pub fn config_sources(&self) -> Vec<ConfigSource> {
        match self {
            SelectedPlatform::Metal(p) => p.config_sources(),
            SelectedPlatform::Aws(p) => p.config_sources(),
            SelectedPlatform::Gcp(p) => p.config_sources(),
            SelectedPlatform::Azure(p) => p.config_sources(),
            SelectedPlatform::NoCloud(p) => p.config_sources(),
        }
    }

    /// Platform network bootstrap needed before config acquisition.
    ///
    /// AWS publishes a bootstrap config before IMDS metadata is available so
    /// IPv4-only, IPv6-only, and dual-stack instances can all reach IMDS.
    pub fn pre_config_network_bootstrap(&self) -> Option<AwsBootstrapNetworkConfig> {
        match self {
            SelectedPlatform::Aws(p) => Some(p.bootstrap_network_config()),
            _ => None,
        }
    }
}

/// Determine whether the selected platform needs a network bootstrap before
/// machine-config acquisition can contact metadata/config endpoints.
pub fn pre_config_network_bootstrap(
    cmdline: &CmdLine,
) -> Result<Option<AwsBootstrapNetworkConfig>, ConfigLoadError> {
    Ok(SelectedPlatform::from_cmdline(cmdline)?.pre_config_network_bootstrap())
}

/// Where the resolved config came from, for stable boot logs and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigOrigin {
    /// A platform-provided source resolved successfully.
    Platform {
        platform: String,
        mode: &'static str,
        source: String,
    },
    /// Compatibility fallback to the initramfs-baked config.
    InitramfsFallback { path: String, reason: String },
}

impl fmt::Display for ConfigOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigOrigin::Platform {
                platform,
                mode,
                source,
            } => {
                write!(f, "platform={platform} mode={mode} source={source}")
            }
            ConfigOrigin::InitramfsFallback { path, reason } => {
                write!(f, "initramfs-fallback path={path} reason={reason}")
            }
        }
    }
}

/// Resolved machine-config bytes plus provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub contents: String,
    pub origin: ConfigOrigin,
    pub platform: String,
}

/// Errors that should abort config loading instead of falling back silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigLoadError {
    UnknownPlatform(String),
    InvalidUtf8,
    UnresolvedPlatformSources {
        platform: String,
        sources: Vec<String>,
    },
    MissingFallback(String),
}

impl fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigLoadError::UnknownPlatform(p) => write!(f, "unknown platform: {p:?}"),
            ConfigLoadError::InvalidUtf8 => write!(f, "machine config is not UTF-8"),
            ConfigLoadError::UnresolvedPlatformSources { platform, sources } => {
                write!(
                    f,
                    "platform {platform} config source(s) did not resolve: {}",
                    sources.join(", ")
                )
            }
            ConfigLoadError::MissingFallback(path) => {
                write!(
                    f,
                    "no platform config source resolved and fallback {path} is unavailable"
                )
            }
        }
    }
}

impl std::error::Error for ConfigLoadError {}

/// Resolve machine config from the Talos platform source, falling back to the
/// initramfs-baked config only when no platform source exists. If a platform
/// advertises a concrete source, failure to read that source is surfaced as a
/// config load error instead of silently booting stale fallback data.
pub fn resolve_config(
    cmdline: &CmdLine,
    store: &dyn ConfigStore,
    fallback: Option<&str>,
) -> Result<ResolvedConfig, ConfigLoadError> {
    let platform = SelectedPlatform::from_cmdline(cmdline)?;
    let platform_name = platform.name().to_string();
    let mode = platform.mode().as_str();
    let sources = platform.config_sources();

    if !sources.is_empty() {
        if let Some((source, bytes)) = resolve_first_source(&platform_name, &sources, store) {
            let contents = String::from_utf8(bytes).map_err(|_| ConfigLoadError::InvalidUtf8)?;
            return Ok(ResolvedConfig {
                contents,
                origin: ConfigOrigin::Platform {
                    platform: platform_name.clone(),
                    mode,
                    source: describe_source(source),
                },
                platform: platform_name,
            });
        }

        return Err(ConfigLoadError::UnresolvedPlatformSources {
            platform: platform_name,
            sources: sources.iter().map(describe_source).collect(),
        });
    }

    let contents = fallback
        .map(str::to_string)
        .ok_or_else(|| ConfigLoadError::MissingFallback(MACHINE_CONFIG_PATH.to_string()))?;

    Ok(ResolvedConfig {
        contents,
        origin: ConfigOrigin::InitramfsFallback {
            path: MACHINE_CONFIG_PATH.to_string(),
            reason: "no platform config source".to_string(),
        },
        platform: platform_name,
    })
}

fn resolve_first_source<'a>(
    platform: &str,
    sources: &'a [ConfigSource],
    store: &dyn ConfigStore,
) -> Option<(&'a ConfigSource, Vec<u8>)> {
    sources.iter().find_map(|source| {
        store
            .fetch_for_platform(platform, source)
            .and_then(|bytes| {
                (!bytes.is_empty() && !bytes.iter().all(u8::is_ascii_whitespace))
                    .then_some((source, bytes))
            })
    })
}

/// Filesystem-backed config source resolver used by real PID1.
///
/// It intentionally performs only local reads: HTTP metadata/downloads are still
/// represented by `talos-platform` but are not fetched here, because the boot
/// binary remains dependency-light: it supports plain `http://` metadata and
/// config URLs with the headers already modeled by `talos-platform`, including
/// fixed-length, chunked, bounded redirect responses, and AWS IMDSv2 token
/// bootstrap, but not TLS, proxying, or OAuth live device flows yet. `file://`
/// URLs are accepted as a deterministic test/bring-up path, while disk sources
/// are resolved against the initramfs root (e.g. `metal-iso`'s `config.yaml`
/// becomes `/config.yaml`).
#[derive(Debug, Clone)]
pub struct FileConfigStore {
    root: PathBuf,
}

impl FileConfigStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FileConfigStore { root: root.into() }
    }

    fn read_to_bytes(&self, path: &Path) -> Option<Vec<u8>> {
        std::fs::read(path).ok().filter(|b| !b.is_empty())
    }

    fn root_join(&self, path: &str) -> PathBuf {
        let clean = path.trim_start_matches('/');
        self.root.join(clean)
    }
}

impl ConfigStore for FileConfigStore {
    fn fetch(&self, source: &ConfigSource) -> Option<Vec<u8>> {
        match source {
            ConfigSource::Http { url, .. } => {
                if let Some(path) = url.strip_prefix("file://") {
                    return self.read_to_bytes(Path::new(path));
                }

                fetch_http_config(source)
            }
            ConfigSource::Disk { labels, path } => {
                // First try the path at initramfs root (`/config.yaml`), then
                // label-scoped staging paths useful for tests and future real
                // volume mounts (`/<label>/<path>`, `/run/<label>/<path>`).
                self.read_to_bytes(&self.root_join(path)).or_else(|| {
                    labels.iter().find_map(|label| {
                        self.read_to_bytes(&self.root.join(label).join(path))
                            .or_else(|| {
                                self.read_to_bytes(&self.root.join("run").join(label).join(path))
                            })
                    })
                })
            }
            ConfigSource::KernelCmdline { .. } => None,
        }
    }

    fn fetch_for_platform(&self, platform: &str, source: &ConfigSource) -> Option<Vec<u8>> {
        match (platform, source) {
            ("aws", ConfigSource::Http { url, headers }) => {
                if let Some(path) = url.strip_prefix("file://") {
                    return self.read_to_bytes(Path::new(path));
                }

                fetch_http_config_with_aws_imdsv2(source, AwsImdsV2::AttemptToken { headers })
            }
            _ => self.fetch(source),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AwsImdsV2<'a> {
    None,
    AttemptToken { headers: &'a [Header] },
}

fn fetch_http_config(source: &ConfigSource) -> Option<Vec<u8>> {
    fetch_http_config_with_aws_imdsv2(source, AwsImdsV2::None)
}

fn fetch_http_config_with_aws_imdsv2(
    source: &ConfigSource,
    aws_imdsv2: AwsImdsV2<'_>,
) -> Option<Vec<u8>> {
    let ConfigSource::Http { url, headers } = source else {
        return None;
    };

    let mut next_url = url.clone();
    let mut active_headers = headers.clone();
    for followed in 0..=HTTP_MAX_REDIRECTS {
        let request = HttpRequestTarget::parse(&next_url)?;
        let request_headers = headers_with_aws_imdsv2_token(&request, &active_headers, aws_imdsv2);
        let response = fetch_http_once(&request, &request_headers)?;
        let parsed = parse_http_response(&response)?;
        if parsed.status == 200 {
            return parsed.body_bytes();
        }

        if !is_redirect_status(parsed.status) || followed == HTTP_MAX_REDIRECTS {
            return None;
        }

        let location = parsed.header_value("Location")?;
        let redirect_url = resolve_http_redirect(&request, location)?;
        let redirect = HttpRequestTarget::parse(&redirect_url)?;
        active_headers = headers_for_redirect(&active_headers, &request, &redirect);
        next_url = redirect_url;
    }

    None
}

fn headers_with_aws_imdsv2_token(
    request: &HttpRequestTarget,
    headers: &[Header],
    aws_imdsv2: AwsImdsV2<'_>,
) -> Vec<Header> {
    let mut request_headers = headers.to_vec();
    let AwsImdsV2::AttemptToken {
        headers: source_headers,
    } = aws_imdsv2
    else {
        return request_headers;
    };
    if has_header(source_headers, AWS_IMDS_TOKEN_HEADER) || !is_aws_user_data_path(request) {
        return request_headers;
    }

    if let Some(token) = fetch_aws_imdsv2_token(request) {
        request_headers.push(Header::new(AWS_IMDS_TOKEN_HEADER, token));
    }
    request_headers
}

fn has_header(headers: &[Header], name: &str) -> bool {
    headers
        .iter()
        .any(|header| header.name.eq_ignore_ascii_case(name))
}

fn is_aws_user_data_path(request: &HttpRequestTarget) -> bool {
    request
        .path_and_query
        .split_once('?')
        .map_or(request.path_and_query.as_str(), |(path, _)| path)
        == "/latest/user-data"
}

fn fetch_aws_imdsv2_token(request: &HttpRequestTarget) -> Option<String> {
    let token_target = HttpRequestTarget {
        host: request.host.clone(),
        port: request.port,
        host_header: request.host_header.clone(),
        path_and_query: AWS_IMDS_TOKEN_PATH.to_string(),
    };
    let token_headers = [
        Header::new(AWS_IMDS_TOKEN_TTL_HEADER, AWS_IMDS_TOKEN_TTL_SECONDS),
        Header::new("Content-Length", "0"),
    ];
    let response = fetch_http_once_with_method(&token_target, HttpMethod::Put, &token_headers)?;
    let parsed = parse_http_response(&response)?;
    if parsed.status != 200 {
        return None;
    }

    let token = String::from_utf8(parsed.body_bytes()?).ok()?;
    let token = token.trim().to_string();
    (!token.is_empty()).then_some(token)
}

#[derive(Debug, Clone, Copy)]
enum HttpMethod {
    Get,
    Put,
}

impl HttpMethod {
    fn as_str(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Put => "PUT",
        }
    }
}

fn fetch_http_once(request: &HttpRequestTarget, headers: &[Header]) -> Option<Vec<u8>> {
    fetch_http_once_with_method(request, HttpMethod::Get, headers)
}

fn fetch_http_once_with_method(
    request: &HttpRequestTarget,
    method: HttpMethod,
    headers: &[Header],
) -> Option<Vec<u8>> {
    let mut stream = None;
    for addr in request.socket_addrs().ok()? {
        if let Ok(candidate) = TcpStream::connect_timeout(&addr, HTTP_TIMEOUT) {
            stream = Some(candidate);
            break;
        }
    }
    let mut stream = stream?;
    stream.set_read_timeout(Some(HTTP_TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(HTTP_TIMEOUT)).ok()?;

    write_http_request(&mut stream, method, request, headers).ok()?;

    let mut response = Vec::new();
    let mut limited = stream.take(HTTP_MAX_CONFIG_BYTES + 1);
    limited.read_to_end(&mut response).ok()?;
    if response.len() as u64 > HTTP_MAX_CONFIG_BYTES {
        return None;
    }

    Some(response)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpRequestTarget {
    host: String,
    port: u16,
    host_header: String,
    path_and_query: String,
}

impl HttpRequestTarget {
    fn parse(url: &str) -> Option<Self> {
        let rest = url.strip_prefix("http://")?;
        let (authority, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };
        if authority.is_empty() {
            return None;
        }

        let (host, port) = parse_authority(authority)?;
        Some(Self {
            host,
            port,
            host_header: authority.to_string(),
            path_and_query: path.to_string(),
        })
    }

    fn socket_addrs(&self) -> std::io::Result<Vec<std::net::SocketAddr>> {
        let addrs = (self.host.as_str(), self.port).to_socket_addrs()?;
        Ok(addrs.collect())
    }
}

fn parse_authority(authority: &str) -> Option<(String, u16)> {
    if let Some(rest) = authority.strip_prefix('[') {
        let end = rest.find(']')?;
        let host = &rest[..end];
        let after = &rest[end + 1..];
        let port = match after.strip_prefix(':') {
            Some(port) => port.parse().ok()?,
            None if after.is_empty() => 80,
            _ => return None,
        };

        return Some((host.to_string(), port));
    }

    match authority.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && !port.is_empty() => {
            Some((host.to_string(), port.parse().ok()?))
        }
        _ => Some((authority.to_string(), 80)),
    }
}

fn write_http_request(
    stream: &mut TcpStream,
    method: HttpMethod,
    target: &HttpRequestTarget,
    headers: &[Header],
) -> std::io::Result<()> {
    for header in headers {
        validate_http_header(header)?;
    }

    write!(
        stream,
        "{} {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: operating-system-init\r\nConnection: close\r\n",
        method.as_str(),
        target.path_and_query,
        target.host_header
    )?;
    for header in headers {
        write!(stream, "{}: {}\r\n", header.name, header.value)?;
    }
    stream.write_all(b"\r\n")?;
    stream.flush()
}

fn validate_http_header(header: &Header) -> std::io::Result<()> {
    let valid_name = !header.name.is_empty() && header.name.bytes().all(is_http_token_byte);
    let valid_value = !header.value.bytes().any(|b| matches!(b, b'\r' | b'\n'));
    if valid_name && valid_value {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid HTTP request header",
        ))
    }
}

fn is_http_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

#[derive(Debug, Clone, Copy)]
struct HttpResponse<'a> {
    status: u16,
    header: &'a [u8],
    body: &'a [u8],
}

impl<'a> HttpResponse<'a> {
    fn header_lines(&self) -> impl Iterator<Item = &'a [u8]> + '_ {
        self.header.split(|b| *b == b'\n').skip(1)
    }

    fn header_value(&self, name: &str) -> Option<&'a str> {
        self.header_lines()
            .find_map(|line| header_value_from_line(line, name))
    }

    fn body_bytes(&self) -> Option<Vec<u8>> {
        if self.header_lines().any(is_chunked_transfer_encoding) {
            decode_chunked_body(self.body)
        } else {
            match self.header_value("Content-Length") {
                Some(length) => {
                    let length = length.parse::<usize>().ok()?;
                    self.body.get(..length).map(<[u8]>::to_vec)
                }
                None => Some(self.body.to_vec()),
            }
        }
    }
}

fn parse_http_response(response: &[u8]) -> Option<HttpResponse<'_>> {
    let header_end = response.windows(4).position(|w| w == b"\r\n\r\n")?;
    let header = &response[..header_end];
    let body = &response[header_end + 4..];
    let status_line = header.split(|b| *b == b'\n').next()?;
    let status_line = std::str::from_utf8(status_line)
        .ok()?
        .trim_end_matches('\r');
    let mut parts = status_line.split_whitespace();
    let version = parts.next()?;
    let status = parts.next()?.parse::<u16>().ok()?;
    if !version.starts_with("HTTP/") {
        return None;
    }

    Some(HttpResponse {
        status,
        header,
        body,
    })
}

fn header_value_from_line<'a>(line: &'a [u8], name: &str) -> Option<&'a str> {
    let line = std::str::from_utf8(line).ok()?.trim_end_matches('\r');
    let (header_name, value) = line.split_once(':')?;
    if header_name.eq_ignore_ascii_case(name) {
        Some(value.trim())
    } else {
        None
    }
}

fn is_chunked_transfer_encoding(line: &[u8]) -> bool {
    let Ok(line) = std::str::from_utf8(line) else {
        return false;
    };
    let Some((name, value)) = line.trim_end_matches('\r').split_once(':') else {
        return false;
    };

    name.eq_ignore_ascii_case("Transfer-Encoding")
        && value
            .split(',')
            .any(|coding| coding.trim().eq_ignore_ascii_case("chunked"))
}

fn is_redirect_status(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn resolve_http_redirect(base: &HttpRequestTarget, location: &str) -> Option<String> {
    let location = location.trim();
    let location = location
        .split_once('#')
        .map_or(location, |(before, _)| before);
    if location.is_empty() {
        return None;
    }

    if location.starts_with("http://") {
        return Some(location.to_string());
    }
    if location.starts_with("https://") {
        return None;
    }
    if let Some(authority_path) = location.strip_prefix("//") {
        return Some(format!("http://{authority_path}"));
    }
    if has_url_scheme(location) {
        return None;
    }

    let path_and_query = if location.starts_with('/') {
        normalize_path_query(location)
    } else if location.starts_with('?') {
        let current_path = base
            .path_and_query
            .split_once('?')
            .map_or(base.path_and_query.as_str(), |(path, _)| path);
        normalize_path_query(&format!("{current_path}{location}"))
    } else {
        let current_path = base
            .path_and_query
            .split_once('?')
            .map_or(base.path_and_query.as_str(), |(path, _)| path);
        let base_dir = current_path
            .rsplit_once('/')
            .map_or("/", |(dir, _)| if dir.is_empty() { "/" } else { dir });
        normalize_path_query(&format!("{base_dir}/{location}"))
    };

    Some(format!("http://{}{}", base.host_header, path_and_query))
}

fn has_url_scheme(value: &str) -> bool {
    let first_delimiter = value
        .char_indices()
        .find_map(|(idx, c)| matches!(c, ':' | '/' | '?' | '#').then_some((idx, c)));
    matches!(first_delimiter, Some((idx, ':')) if idx > 0)
}

fn normalize_path_query(path_query: &str) -> String {
    let (path, query) = path_query
        .split_once('?')
        .map_or((path_query, ""), |(path, query)| (path, query));
    let mut segments = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }

    let mut normalized = String::from("/");
    normalized.push_str(&segments.join("/"));
    if path.ends_with('/') && !normalized.ends_with('/') {
        normalized.push('/');
    }
    if !query.is_empty() {
        normalized.push('?');
        normalized.push_str(query);
    }
    normalized
}

fn headers_for_redirect(
    headers: &[Header],
    from: &HttpRequestTarget,
    to: &HttpRequestTarget,
) -> Vec<Header> {
    if from.port == to.port && from.host.eq_ignore_ascii_case(&to.host) {
        return headers.to_vec();
    }

    headers
        .iter()
        .filter(|header| !is_sensitive_redirect_header(&header.name))
        .cloned()
        .collect()
}

fn is_sensitive_redirect_header(name: &str) -> bool {
    [
        "Authorization",
        "Proxy-Authorization",
        "Cookie",
        "Cookie2",
        AWS_IMDS_TOKEN_HEADER,
    ]
    .iter()
    .any(|sensitive| name.eq_ignore_ascii_case(sensitive))
}

fn decode_chunked_body(body: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut pos = 0usize;

    loop {
        let line_end = find_crlf(&body[pos..])? + pos;
        let size_line = std::str::from_utf8(&body[pos..line_end]).ok()?;
        let size_token = size_line.split(';').next()?.trim();
        let size = usize::from_str_radix(size_token, 16).ok()?;
        pos = line_end + 2;

        if size == 0 {
            // Trailing headers, if present, are intentionally ignored. Requiring
            // their terminating CRLF mirrors the HTTP framing boundary and
            // rejects truncated chunked bodies.
            let trailer_end = body[pos..].windows(4).position(|w| w == b"\r\n\r\n");
            if trailer_end.is_some() || body.get(pos..pos + 2) == Some(b"\r\n") {
                return Some(out);
            }
            return None;
        }

        let chunk_end = pos.checked_add(size)?;
        if chunk_end.checked_add(2)? > body.len() {
            return None;
        }
        if &body[chunk_end..chunk_end + 2] != b"\r\n" {
            return None;
        }

        out.extend_from_slice(&body[pos..chunk_end]);
        if out.len() as u64 > HTTP_MAX_CONFIG_BYTES {
            return None;
        }
        pos = chunk_end + 2;
    }
}

fn find_crlf(bytes: &[u8]) -> Option<usize> {
    bytes.windows(2).position(|w| w == b"\r\n")
}

fn describe_source(source: &ConfigSource) -> String {
    match source {
        ConfigSource::Http { url, .. } => format!("http:{url}"),
        ConfigSource::Disk { labels, path } => format!("disk:{}:{path}", labels.join("|")),
        ConfigSource::KernelCmdline { param, value } => format!("cmdline:{param}={value}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Instant;
    use os_platform_domain::MemoryStore;

    const CFG: &str = "version: v1alpha1\nmachine:\n  type: worker\n";
    const FALLBACK: &str = "version: v1alpha1\nmachine:\n  type: controlplane\n";

    #[test]
    fn selects_metal_iso_from_talos_cmdline() {
        let cmd = CmdLine::parse("talos.platform=metal talos.config=metal-iso");
        let platform = SelectedPlatform::from_cmdline(&cmd).unwrap();
        assert_eq!(platform.name(), "metal");
        let sources = platform.config_sources();
        assert_eq!(sources[0].path(), Some("config.yaml"));
        assert_eq!(sources[0].labels(), &["metal-iso"]);
    }

    #[test]
    fn aws_requests_pre_config_network_bootstrap() {
        let cmd = CmdLine::parse("talos.platform=aws");
        let bootstrap = pre_config_network_bootstrap(&cmd).unwrap().unwrap();
        assert_eq!(bootstrap.interface, "eth0");
        assert!(bootstrap.dhcp4);
        assert!(bootstrap.dhcp6);
        assert!(bootstrap.require_up);
        assert_eq!(bootstrap.route_metric, 1024);
    }

    #[test]
    fn metal_does_not_request_pre_config_network_bootstrap() {
        let cmd = CmdLine::parse("talos.platform=metal talos.config=metal-iso");
        assert_eq!(pre_config_network_bootstrap(&cmd).unwrap(), None);
    }

    #[test]
    fn pre_config_network_bootstrap_only_for_aws() {
        let aws = CmdLine::parse("talos.platform=aws");
        assert!(pre_config_network_bootstrap(&aws).unwrap().is_some());

        for platform in ["metal", "gcp", "azure", "nocloud"] {
            let cmd = CmdLine::parse(&format!("talos.platform={platform}"));
            assert_eq!(
                pre_config_network_bootstrap(&cmd).unwrap(),
                None,
                "{platform} must not request pre-config network bootstrap"
            );
        }
    }

    #[test]
    fn resolves_platform_config_before_fallback() {
        let cmd = CmdLine::parse("talos.platform=metal talos.config=metal-iso");
        let platform = SelectedPlatform::from_cmdline(&cmd).unwrap();
        let store = MemoryStore::new().with(&platform.config_sources()[0], CFG.as_bytes().to_vec());
        let resolved = resolve_config(&cmd, &store, Some(FALLBACK)).unwrap();
        assert_eq!(resolved.contents, CFG);
        assert_eq!(resolved.platform, "metal");
        assert!(matches!(resolved.origin, ConfigOrigin::Platform { .. }));
    }

    #[test]
    fn falls_back_when_no_platform_config_source_exists() {
        let cmd = CmdLine::parse("console=ttyS0");
        let resolved = resolve_config(&cmd, &MemoryStore::new(), Some(FALLBACK)).unwrap();
        assert_eq!(resolved.contents, FALLBACK);
        assert_eq!(resolved.platform, "metal");
        assert_eq!(
            resolved.origin,
            ConfigOrigin::InitramfsFallback {
                path: MACHINE_CONFIG_PATH.to_string(),
                reason: "no platform config source".to_string(),
            }
        );
    }

    #[test]
    fn explicit_unknown_platform_fails() {
        let cmd = CmdLine::parse("talos.platform=bogus");
        let err = resolve_config(&cmd, &MemoryStore::new(), Some(FALLBACK)).unwrap_err();
        assert_eq!(err, ConfigLoadError::UnknownPlatform("bogus".to_string()));
    }

    #[test]
    fn explicit_metal_url_failure_does_not_use_fallback() {
        let cmd = CmdLine::parse("talos.platform=metal talos.config=http://metadata/config.yaml");
        let err = resolve_config(&cmd, &MemoryStore::new(), Some(FALLBACK)).unwrap_err();
        assert_eq!(
            err,
            ConfigLoadError::UnresolvedPlatformSources {
                platform: "metal".to_string(),
                sources: vec!["http:http://metadata/config.yaml".to_string()],
            }
        );
    }

    #[test]
    fn metal_iso_failure_does_not_use_fallback() {
        let dir = std::env::temp_dir().join(format!(
            "operating-system-platform-config-missing-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let cmd = CmdLine::parse("talos.platform=metal talos.config=metal-iso");
        let err = resolve_config(&cmd, &FileConfigStore::new(&dir), Some(FALLBACK)).unwrap_err();
        assert_eq!(
            err,
            ConfigLoadError::UnresolvedPlatformSources {
                platform: "metal".to_string(),
                sources: vec!["disk:metal-iso:config.yaml".to_string()],
            }
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unsupported_https_url_failure_does_not_use_fallback() {
        let cmd =
            CmdLine::parse("talos.platform=metal talos.config=https://example.test/node.yaml");
        let err = resolve_config(&cmd, &FileConfigStore::new("/"), Some(FALLBACK)).unwrap_err();
        assert_eq!(
            err,
            ConfigLoadError::UnresolvedPlatformSources {
                platform: "metal".to_string(),
                sources: vec!["http:https://example.test/node.yaml".to_string()],
            }
        );
    }

    #[test]
    fn file_store_reads_metal_iso_config_from_root() {
        let dir =
            std::env::temp_dir().join(format!("operating-system-platform-config-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.yaml"), CFG).unwrap();

        let cmd = CmdLine::parse("talos.platform=metal talos.config=metal-iso");
        let resolved = resolve_config(&cmd, &FileConfigStore::new(&dir), Some(FALLBACK)).unwrap();
        assert_eq!(resolved.contents, CFG);
        assert!(matches!(resolved.origin, ConfigOrigin::Platform { .. }));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn file_store_reads_file_url_config() {
        let dir = std::env::temp_dir().join(format!(
            "operating-system-platform-config-file-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("node.yaml");
        std::fs::write(&path, CFG).unwrap();

        let cmd = CmdLine::parse(&format!(
            "talos.platform=metal talos.config=file://{}",
            path.display()
        ));
        let resolved = resolve_config(&cmd, &FileConfigStore::new("/"), Some(FALLBACK)).unwrap();
        assert_eq!(resolved.contents, CFG);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn file_store_fetches_plain_http_with_headers() {
        let (url, request_rx, server) = spawn_http_server(
            "/seed/user-data",
            b"version: v1alpha1\nmachine:\n  type: worker\n",
        );
        let source = ConfigSource::http(url, vec![Header::new("Metadata-Flavor", "Google")]);

        let bytes = FileConfigStore::new("/").fetch(&source).unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(request.starts_with("GET /seed/user-data HTTP/1.1\r\n"));
        assert!(request.contains("\r\nMetadata-Flavor: Google\r\n"));
        assert!(request.contains("\r\nConnection: close\r\n"));
        server.join().unwrap();
    }

    #[test]
    fn file_store_rejects_non_ok_http_status() {
        let (url, request_rx, server) =
            spawn_raw_http_server("/no-content.yaml", status_response(204, b""));
        let source = ConfigSource::http_no_headers(url);

        assert_eq!(FileConfigStore::new("/").fetch(&source), None);

        let request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(request.starts_with("GET /no-content.yaml HTTP/1.1\r\n"));
        server.join().unwrap();
    }

    #[test]
    fn file_store_fetches_aws_user_data_with_imdsv2_token() {
        let (base_url, request_rx, server) = spawn_sequence_http_server_for_requests(vec![
            ("PUT", "/latest/api/token", ok_response(b"tok-123")),
            ("GET", "/latest/user-data", ok_response(CFG.as_bytes())),
        ]);
        let source = ConfigSource::http_no_headers(format!("{base_url}/latest/user-data"));

        let bytes = FileConfigStore::new("/")
            .fetch_for_platform("aws", &source)
            .unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let token_request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let user_data_request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(token_request.starts_with("PUT /latest/api/token HTTP/1.1\r\n"));
        assert!(token_request.contains(&format!(
            "\r\n{AWS_IMDS_TOKEN_TTL_HEADER}: {AWS_IMDS_TOKEN_TTL_SECONDS}\r\n"
        )));
        assert!(user_data_request.starts_with("GET /latest/user-data HTTP/1.1\r\n"));
        assert!(user_data_request.contains("\r\nX-aws-ec2-metadata-token: tok-123\r\n"));
        server.join().unwrap();
    }

    #[test]
    fn file_store_fetches_aws_user_data_without_token_when_imdsv2_unavailable() {
        let (base_url, request_rx, server) = spawn_sequence_http_server_for_requests(vec![
            ("PUT", "/latest/api/token", not_found_response()),
            ("GET", "/latest/user-data", ok_response(CFG.as_bytes())),
        ]);
        let source = ConfigSource::http_no_headers(format!("{base_url}/latest/user-data"));

        let bytes = FileConfigStore::new("/")
            .fetch_for_platform("aws", &source)
            .unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let token_request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let user_data_request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(token_request.starts_with("PUT /latest/api/token HTTP/1.1\r\n"));
        assert!(user_data_request.starts_with("GET /latest/user-data HTTP/1.1\r\n"));
        assert!(!user_data_request.contains("\r\nX-aws-ec2-metadata-token:"));
        server.join().unwrap();
    }

    #[test]
    fn resolve_config_uses_http_metal_source_before_fallback() {
        let (url, request_rx, server) = spawn_http_server("/config.yaml", CFG.as_bytes());
        let cmd = CmdLine::parse(&format!("talos.platform=metal talos.config={url}"));

        let resolved = resolve_config(&cmd, &FileConfigStore::new("/"), Some(FALLBACK)).unwrap();
        assert_eq!(resolved.contents, CFG);
        assert_eq!(resolved.platform, "metal");
        assert_eq!(
            resolved.origin,
            ConfigOrigin::Platform {
                platform: "metal".to_string(),
                mode: "metal",
                source: format!("http:{url}"),
            }
        );

        let request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(request.starts_with("GET /config.yaml HTTP/1.1\r\n"));
        server.join().unwrap();
    }

    #[test]
    fn resolve_config_records_the_source_that_yielded_config() {
        let cmd = CmdLine::parse("talos.platform=aws");
        let platform = SelectedPlatform::from_cmdline(&cmd).unwrap();
        let sources = platform.config_sources();
        assert_eq!(sources.len(), 2);
        let store = MemoryStore::new().with(&sources[1], CFG.as_bytes().to_vec());

        let resolved = resolve_config(&cmd, &store, Some(FALLBACK)).unwrap();
        assert_eq!(resolved.contents, CFG);
        assert_eq!(
            resolved.origin,
            ConfigOrigin::Platform {
                platform: "aws".to_string(),
                mode: "cloud",
                source: "http:http://[fd00:ec2::254]/latest/user-data".to_string(),
            }
        );
    }

    #[test]
    fn file_store_decodes_chunked_http_config() {
        let chunked = chunked_response(&[
            b"version: v1alpha1\n".as_slice(),
            b"machine:\n  type: ",
            b"worker\n",
        ]);
        let (url, request_rx, server) = spawn_raw_http_server("/seed/user-data", chunked);
        let source = ConfigSource::http_no_headers(url);

        let bytes = FileConfigStore::new("/").fetch(&source).unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(request.starts_with("GET /seed/user-data HTTP/1.1\r\n"));
        server.join().unwrap();
    }

    #[test]
    fn chunked_parser_rejects_truncated_body() {
        let response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nabc";
        let parsed = parse_http_response(response).unwrap();
        assert_eq!(parsed.body_bytes(), None);
    }

    #[test]
    fn fixed_length_parser_rejects_truncated_body() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nabc";
        let parsed = parse_http_response(response).unwrap();
        assert_eq!(parsed.body_bytes(), None);
    }

    #[test]
    fn fixed_length_parser_ignores_trailing_bytes() {
        let response =
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabcignored";
        let parsed = parse_http_response(response).unwrap();
        assert_eq!(parsed.body_bytes(), Some(b"abc".to_vec()));
    }

    #[test]
    fn file_store_follows_absolute_http_redirect() {
        let (final_url, final_rx, final_server) = spawn_http_server("/final.yaml", CFG.as_bytes());
        let (start_url, start_rx, start_server) =
            spawn_raw_http_server("/start.yaml", redirect_response(302, &final_url));
        let source = ConfigSource::http_no_headers(start_url);

        let bytes = FileConfigStore::new("/").fetch(&source).unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let first = start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let second = final_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(first.starts_with("GET /start.yaml HTTP/1.1\r\n"));
        assert!(second.starts_with("GET /final.yaml HTTP/1.1\r\n"));
        start_server.join().unwrap();
        final_server.join().unwrap();
    }

    #[test]
    fn file_store_follows_relative_redirect_and_preserves_same_origin_headers() {
        let (base_url, request_rx, server) = spawn_sequence_http_server(vec![
            (
                "/seed/user-data",
                redirect_response(302, "../final/user-data"),
            ),
            ("/final/user-data", ok_response(CFG.as_bytes())),
        ]);
        let source = ConfigSource::http(
            format!("{base_url}/seed/user-data"),
            vec![Header::new("Metadata-Flavor", "Google")],
        );

        let bytes = FileConfigStore::new("/").fetch(&source).unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let first = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let second = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(first.starts_with("GET /seed/user-data HTTP/1.1\r\n"));
        assert!(second.starts_with("GET /final/user-data HTTP/1.1\r\n"));
        assert!(first.contains("\r\nMetadata-Flavor: Google\r\n"));
        assert!(second.contains("\r\nMetadata-Flavor: Google\r\n"));
        server.join().unwrap();
    }

    #[test]
    fn file_store_drops_authorization_on_cross_origin_redirect() {
        let (final_url, final_rx, final_server) = spawn_http_server("/final.yaml", CFG.as_bytes());
        let (start_url, start_rx, start_server) =
            spawn_raw_http_server("/start.yaml", redirect_response(302, &final_url));
        let source = ConfigSource::http(
            start_url,
            vec![
                Header::new("Authorization", "Bearer secret"),
                Header::new("Metadata-Flavor", "Google"),
            ],
        );

        let bytes = FileConfigStore::new("/").fetch(&source).unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let first = start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let second = final_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(first.contains("\r\nAuthorization: Bearer secret\r\n"));
        assert!(first.contains("\r\nMetadata-Flavor: Google\r\n"));
        assert!(!second.contains("\r\nAuthorization: Bearer secret\r\n"));
        assert!(second.contains("\r\nMetadata-Flavor: Google\r\n"));
        start_server.join().unwrap();
        final_server.join().unwrap();
    }

    #[test]
    fn file_store_drops_aws_metadata_token_on_cross_origin_redirect() {
        let (final_url, final_rx, final_server) = spawn_http_server("/final.yaml", CFG.as_bytes());
        let (start_url, start_rx, start_server) =
            spawn_raw_http_server("/latest/user-data", redirect_response(302, &final_url));
        let source = ConfigSource::http(
            start_url,
            vec![
                Header::new(AWS_IMDS_TOKEN_HEADER, "tok-secret"),
                Header::new("Metadata-Flavor", "AwsTest"),
            ],
        );

        let bytes = FileConfigStore::new("/")
            .fetch_for_platform("aws", &source)
            .unwrap();
        assert_eq!(bytes, CFG.as_bytes());

        let first = start_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let second = final_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(first.contains("\r\nX-aws-ec2-metadata-token: tok-secret\r\n"));
        assert!(first.contains("\r\nMetadata-Flavor: AwsTest\r\n"));
        assert!(!second.contains("\r\nX-aws-ec2-metadata-token:"));
        assert!(second.contains("\r\nMetadata-Flavor: AwsTest\r\n"));
        start_server.join().unwrap();
        final_server.join().unwrap();
    }

    #[test]
    fn file_store_stops_redirect_loop_at_cap() {
        let responses = (0..=HTTP_MAX_REDIRECTS)
            .map(|_| ("/loop", redirect_response(302, "/loop")))
            .collect();
        let (base_url, request_rx, server) = spawn_sequence_http_server(responses);
        let source = ConfigSource::http_no_headers(format!("{base_url}/loop"));

        assert_eq!(FileConfigStore::new("/").fetch(&source), None);

        server.join().unwrap();
        let requests: Vec<_> = request_rx.try_iter().collect();
        assert_eq!(requests.len(), HTTP_MAX_REDIRECTS + 1);
        assert!(
            requests
                .iter()
                .all(|request| request.starts_with("GET /loop HTTP/1.1\r\n"))
        );
    }

    #[test]
    fn file_store_rejects_https_redirect_without_tls() {
        let (url, request_rx, server) = spawn_raw_http_server(
            "/start.yaml",
            redirect_response(302, "https://example.test/config.yaml"),
        );
        let source = ConfigSource::http_no_headers(url);

        assert_eq!(FileConfigStore::new("/").fetch(&source), None);

        let request = request_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(request.starts_with("GET /start.yaml HTTP/1.1\r\n"));
        server.join().unwrap();
    }

    #[test]
    fn file_store_rejects_invalid_request_headers() {
        let (url, request_rx, server) = spawn_http_server("/config.yaml", CFG.as_bytes());
        let source = ConfigSource::http(url, vec![Header::new("X-Test", "ok\r\nInjected: yes")]);

        assert_eq!(FileConfigStore::new("/").fetch(&source), None);

        let request = request_rx
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or_default();
        assert!(!request.contains("Injected: yes"));
        server.join().unwrap();
    }

    fn spawn_http_server(
        path: &'static str,
        body: &'static [u8],
    ) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
        spawn_raw_http_server(path, ok_response(body))
    }

    fn chunked_response(chunks: &[&[u8]]) -> Vec<u8> {
        let mut response =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
        for chunk in chunks {
            write!(response, "{:x}\r\n", chunk.len()).unwrap();
            response.extend_from_slice(chunk);
            response.extend_from_slice(b"\r\n");
        }
        response.extend_from_slice(b"0\r\n\r\n");
        response
    }

    fn ok_response(body: &[u8]) -> Vec<u8> {
        status_response(200, body)
    }

    fn status_response(status: u16, body: &[u8]) -> Vec<u8> {
        [
            format!(
                "HTTP/1.1 {status} Status\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .into_bytes(),
            body.to_vec(),
        ]
        .concat()
    }

    fn redirect_response(status: u16, location: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 {status} Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .into_bytes()
    }

    fn not_found_response() -> Vec<u8> {
        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()
    }

    fn spawn_raw_http_server(
        path: &'static str,
        response: Vec<u8>,
    ) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
        let (base_url, rx, handle) = spawn_sequence_http_server(vec![(path, response)]);
        (format!("{base_url}{path}"), rx, handle)
    }

    fn spawn_sequence_http_server(
        routes: Vec<(&'static str, Vec<u8>)>,
    ) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
        spawn_sequence_http_server_for_requests(
            routes
                .into_iter()
                .map(|(path, response)| ("GET", path, response))
                .collect(),
        )
    }

    fn spawn_sequence_http_server_for_requests(
        routes: Vec<(&'static str, &'static str, Vec<u8>)>,
    ) -> (String, mpsc::Receiver<String>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            for (method, path, response) in routes {
                let deadline = Instant::now() + Duration::from_secs(3);
                loop {
                    let Ok((mut stream, _)) = accept_with_deadline(&listener, deadline) else {
                        break;
                    };
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    let mut request = Vec::new();
                    let mut buf = [0u8; 1024];
                    loop {
                        let read = stream.read(&mut buf).unwrap_or(0);
                        if read == 0 {
                            break;
                        }
                        request.extend_from_slice(&buf[..read]);
                        if request.windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }

                    let ok = request.starts_with(format!("{method} {path} ").as_bytes());
                    if ok {
                        let request_text = String::from_utf8_lossy(&request).into_owned();
                        let _ = tx.send(request_text);
                        let _ = stream.write_all(&response);
                        let _ = stream.flush();
                        break;
                    } else {
                        let _ = stream
                            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                        let _ = stream.flush();
                    }
                }
            }
        });

        (format!("http://{addr}"), rx, handle)
    }

    fn accept_with_deadline(
        listener: &TcpListener,
        deadline: Instant,
    ) -> std::io::Result<(std::net::TcpStream, std::net::SocketAddr)> {
        loop {
            match listener.accept() {
                Ok(stream) => return Ok(stream),
                Err(err)
                    if err.kind() == std::io::ErrorKind::WouldBlock
                        && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => return Err(err),
            }
        }
    }
}
