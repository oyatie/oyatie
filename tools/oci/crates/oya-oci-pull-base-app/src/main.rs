#![cfg_attr(test, allow(clippy::expect_used, clippy::panic, clippy::unwrap_used))]

// tools/oci/crates/oya-oci-pull-base-app/src/main.rs
//
// oya-oci-pull-base — build-host OCI Distribution pull client.
//
// Pulls a pinned single-arch base image manifest plus its referenced config and
// layer blobs from an OCI Distribution registry, verifies sha256 digests, and
// writes the OCI Image Layout tree consumed by oya-oci-assemble.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use reqwest::{
    StatusCode,
    blocking::Client,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, USER_AGENT},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const DEFAULT_MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
const INDEX_MEDIA_TYPES: &[&str] = &[
    "application/vnd.oci.image.index.v1+json",
    "application/vnd.docker.distribution.manifest.list.v2+json",
];
const MANIFEST_ACCEPT: &str = "application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json";
const TOOL_USER_AGENT: &str = "oya-oci-pull-base/1.0 (+tools/oci/crates/oya-oci-pull-base-app)";

#[derive(Debug, Parser)]
#[command(
    name = "oya-oci-pull-base",
    about = "Pull a pinned OCI manifest and blobs into an OCI Image Layout directory"
)]
struct Args {
    /// Registry host[:port], without scheme.
    registry: String,

    /// Repository path inside the registry.
    repository: String,

    /// Single-arch image manifest digest, sha256:<hex>.
    digest: String,

    /// Output directory for oci-layout, index.json, and blobs/sha256/*.
    outdir: PathBuf,

    /// Use plain HTTP instead of HTTPS for local/in-cluster registries.
    #[arg(long)]
    insecure: bool,
}

#[derive(Debug)]
struct HttpBytes {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

#[derive(Debug)]
struct PulledManifest {
    bytes: Vec<u8>,
    document: Value,
}

fn main() -> Result<()> {
    let args = Args::parse();
    pull_oci_base(&args)
}

fn pull_oci_base(args: &Args) -> Result<()> {
    digest_hex(&args.digest)?;
    let blobs_dir = args.outdir.join("blobs").join("sha256");
    fs::create_dir_all(&blobs_dir)
        .with_context(|| format!("failed to create {}", blobs_dir.display()))?;

    let base = registry_base(&args.registry, args.insecure);
    let client = Client::new();
    let token = get_bearer_token(&client, &base, &args.registry, &args.repository)?;
    let manifest = fetch_manifest(
        &client,
        &base,
        &args.repository,
        &args.digest,
        token.as_deref(),
    )?;

    write_blob(&blobs_dir, &args.digest, &manifest.bytes)?;

    let config = manifest
        .document
        .get("config")
        .context("manifest has no config descriptor")?;
    let config_digest = required_string(config, "digest", "manifest config")?;
    let config_bytes = fetch_blob(
        &client,
        &base,
        &args.repository,
        config_digest,
        token.as_deref(),
    )?;
    write_blob(&blobs_dir, config_digest, &config_bytes)?;

    let layers = manifest
        .document
        .get("layers")
        .and_then(Value::as_array)
        .context("manifest has no layers array")?;
    if layers.is_empty() {
        bail!("manifest {} declares no layers", args.digest);
    }
    for (index, layer) in layers.iter().enumerate() {
        let layer_digest = required_string(layer, "digest", &format!("manifest layer {index}"))?;
        let layer_bytes = fetch_blob(
            &client,
            &base,
            &args.repository,
            layer_digest,
            token.as_deref(),
        )?;
        write_blob(&blobs_dir, layer_digest, &layer_bytes)?;
    }

    write_index(
        &args.outdir,
        &args.digest,
        manifest.bytes.len(),
        &manifest.document,
    )?;
    fs::write(
        args.outdir.join("oci-layout"),
        r#"{"imageLayoutVersion":"1.0.0"}"#,
    )
    .with_context(|| format!("failed to write {}/oci-layout", args.outdir.display()))?;

    eprintln!(
        "oya-oci-pull-base: wrote OCI layout to {} (manifest {}, config {}, {} layer(s))",
        args.outdir.display(),
        args.digest,
        config_digest,
        layers.len()
    );
    Ok(())
}

fn registry_base(registry: &str, insecure: bool) -> String {
    let scheme = if insecure { "http" } else { "https" };
    format!("{scheme}://{}", registry.trim_end_matches('/'))
}

fn get_bearer_token(
    client: &Client,
    base: &str,
    registry: &str,
    repo: &str,
) -> Result<Option<String>> {
    let scope = format!("repository:{repo}:pull");
    let primary_url = format!(
        "{base}/v2/token?service={}&scope={}",
        percent_encode(registry),
        percent_encode(&scope)
    );
    let response = http_get(client, &primary_url, &[])?;
    if response.status == StatusCode::OK {
        if let Some(token) = extract_token(&response.body) {
            return Ok(Some(token));
        }
    }

    let probe_url = format!("{base}/v2/");
    let response = http_get(client, &probe_url, &[])?;
    if response.status == StatusCode::OK {
        return Ok(None);
    }
    if response.status != StatusCode::UNAUTHORIZED {
        bail!(
            "registry {registry} /v2/ probe returned {} (expected 401 challenge or 200)",
            response.status
        );
    }
    let challenge = header_string(&response.headers, "WWW-Authenticate")
        .context("registry returned 401 with no WWW-Authenticate header")?;
    let params = parse_www_authenticate(&challenge);
    let realm = params
        .iter()
        .find_map(|(key, value)| (key == "realm").then_some(value.as_str()))
        .context("WWW-Authenticate challenge has no realm")?;
    let service = params
        .iter()
        .find_map(|(key, value)| (key == "service").then_some(value.as_str()))
        .unwrap_or(registry);
    let token_url = format!(
        "{realm}?service={}&scope={}",
        percent_encode(service),
        percent_encode(&scope)
    );
    let response = http_get(client, &token_url, &[])?;
    if response.status != StatusCode::OK {
        bail!("token request to {token_url} returned {}", response.status);
    }
    extract_token(&response.body)
        .map(Some)
        .context("token response had no token or access_token")
}

fn fetch_manifest(
    client: &Client,
    base: &str,
    repo: &str,
    digest: &str,
    token: Option<&str>,
) -> Result<PulledManifest> {
    let url = format!("{base}/v2/{repo}/manifests/{digest}");
    let response = http_get(client, &url, &auth_headers(token, Some(MANIFEST_ACCEPT)))?;
    if response.status != StatusCode::OK {
        bail!(
            "manifest GET {url} returned {} (body: {:?})",
            response.status,
            body_snippet(&response.body, 256)
        );
    }
    verify_digest(digest, &response.body).context("manifest digest mismatch")?;
    let document: Value = serde_json::from_slice(&response.body)
        .with_context(|| format!("manifest {digest} is not valid JSON"))?;
    let header_media_type = header_string(&response.headers, CONTENT_TYPE.as_str())
        .map(|value| content_type_without_parameters(&value))
        .unwrap_or_default();
    let document_media_type = document
        .get("mediaType")
        .and_then(Value::as_str)
        .unwrap_or(&header_media_type);
    if INDEX_MEDIA_TYPES.contains(&document_media_type) {
        bail!(
            "digest {digest} references a multi-arch image index ({document_media_type}), not a single-arch manifest"
        );
    }
    Ok(PulledManifest {
        bytes: response.body,
        document,
    })
}

fn fetch_blob(
    client: &Client,
    base: &str,
    repo: &str,
    digest: &str,
    token: Option<&str>,
) -> Result<Vec<u8>> {
    let url = format!("{base}/v2/{repo}/blobs/{digest}");
    let response = http_get(client, &url, &auth_headers(token, None))?;
    if response.status != StatusCode::OK {
        bail!("blob GET {url} returned {}", response.status);
    }
    verify_digest(digest, &response.body)
        .with_context(|| format!("blob digest mismatch for {digest}"))?;
    Ok(response.body)
}

fn http_get(client: &Client, url: &str, headers: &[(&str, String)]) -> Result<HttpBytes> {
    let mut request = client.get(url).header(USER_AGENT, TOOL_USER_AGENT);
    for (name, value) in headers {
        request = request.header(*name, value);
    }
    let response = request
        .send()
        .with_context(|| format!("GET {url} failed"))?;
    let status = response.status();
    let headers = response.headers().clone();
    let body = response
        .bytes()
        .with_context(|| format!("failed to read GET {url} response body"))?
        .to_vec();
    Ok(HttpBytes {
        status,
        headers,
        body,
    })
}

fn auth_headers(token: Option<&str>, accept: Option<&str>) -> Vec<(&'static str, String)> {
    let mut headers = Vec::new();
    if let Some(token) = token {
        headers.push((AUTHORIZATION.as_str(), format!("Bearer {token}")));
    }
    if let Some(accept) = accept {
        headers.push((ACCEPT.as_str(), accept.to_owned()));
    }
    headers
}

fn parse_www_authenticate(value: &str) -> Vec<(String, String)> {
    let value = value.trim();
    let value = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .unwrap_or(value);
    value
        .split(',')
        .filter_map(|part| {
            let (key, raw_value) = part.trim().split_once('=')?;
            Some((
                key.trim().to_owned(),
                raw_value.trim().trim_matches('"').to_owned(),
            ))
        })
        .collect()
}

fn extract_token(body: &[u8]) -> Option<String> {
    let document: Value = serde_json::from_slice(body).ok()?;
    document
        .get("token")
        .or_else(|| document.get("access_token"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn content_type_without_parameters(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn verify_digest(expected: &str, bytes: &[u8]) -> Result<()> {
    let digest = Sha256::digest(bytes);
    let actual = format!("sha256:{}", hex_lower(&digest));
    if actual != expected {
        bail!("expected {expected}, downloaded bytes hash to {actual}");
    }
    Ok(())
}

fn digest_hex(digest: &str) -> Result<&str> {
    let hex = digest
        .strip_prefix("sha256:")
        .with_context(|| format!("expected sha256:<hex> digest, got {digest:?}"))?;
    if hex.len() != 64 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        bail!("invalid sha256 digest {digest}: expected 64 hex characters");
    }
    Ok(hex)
}

fn write_blob(blobs_dir: &PathBuf, digest: &str, bytes: &[u8]) -> Result<()> {
    let path = blobs_dir.join(digest_hex(digest)?);
    fs::write(&path, bytes).with_context(|| format!("failed to write {}", path.display()))
}

fn write_index(
    outdir: &PathBuf,
    digest: &str,
    manifest_size: usize,
    manifest: &Value,
) -> Result<()> {
    let media_type = manifest
        .get("mediaType")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_MANIFEST_MEDIA_TYPE);
    let index = json!({
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.index.v1+json",
        "manifests": [{
            "mediaType": media_type,
            "digest": digest,
            "size": manifest_size,
            "platform": {"architecture": "arm64", "os": "linux"}
        }]
    });
    let mut bytes = serde_json::to_vec_pretty(&index).context("failed to serialize index.json")?;
    bytes.push(b'\n');
    fs::write(outdir.join("index.json"), bytes)
        .with_context(|| format!("failed to write {}/index.json", outdir.display()))
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

fn body_snippet(bytes: &[u8], limit: usize) -> Vec<u8> {
    bytes.iter().copied().take(limit).collect()
}

fn required_string<'a>(value: &'a Value, field: &str, context: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("{context} is missing string field {field}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        sync::{Arc, Mutex},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn parses_bearer_challenge_parameters() {
        let params = parse_www_authenticate(
            r#"Bearer realm="https://gcr.io/v2/token",service="gcr.io",scope="repository:distroless/static:pull""#,
        );
        assert_eq!(
            params,
            vec![
                ("realm".to_owned(), "https://gcr.io/v2/token".to_owned()),
                ("service".to_owned(), "gcr.io".to_owned()),
                (
                    "scope".to_owned(),
                    "repository:distroless/static:pull".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn extracts_token_variants() {
        assert_eq!(extract_token(br#"{"token":"abc"}"#).unwrap(), "abc");
        assert_eq!(extract_token(br#"{"access_token":"def"}"#).unwrap(), "def");
        assert!(extract_token(br#"not-json"#).is_none());
    }

    #[test]
    fn percent_encoding_matches_registry_token_query_needs() {
        assert_eq!(
            percent_encode("repository:distroless/static:pull"),
            "repository%3Adistroless%2Fstatic%3Apull"
        );
    }

    #[test]
    fn rejects_non_sha256_or_malformed_digests() {
        assert!(digest_hex("sha512:abcd").is_err());
        assert!(digest_hex("sha256:abcd").is_err());
        assert!(
            digest_hex("sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz")
                .is_err()
        );
    }

    #[test]
    fn pulls_manifest_config_and_layers_into_oci_layout() {
        let fixture = RegistryFixture::new();
        let observed_paths = Arc::new(Mutex::new(Vec::new()));
        let (address, server) =
            start_registry_fixture(fixture.clone(), Arc::clone(&observed_paths), 4);
        let outdir = unique_temp_dir("pulls_manifest_config_and_layers_into_oci_layout");

        let args = Args {
            registry: address,
            repository: "distroless/static".to_owned(),
            digest: fixture.manifest_digest.clone(),
            outdir: outdir.clone(),
            insecure: true,
        };
        pull_oci_base(&args).unwrap();
        server.join().unwrap();

        assert!(outdir.join("oci-layout").is_file());
        assert!(outdir.join("index.json").is_file());
        assert!(
            outdir
                .join("blobs")
                .join("sha256")
                .join(digest_hex(&fixture.manifest_digest).unwrap())
                .is_file()
        );
        assert!(
            outdir
                .join("blobs")
                .join("sha256")
                .join(digest_hex(&fixture.config_digest).unwrap())
                .is_file()
        );
        assert!(
            outdir
                .join("blobs")
                .join("sha256")
                .join(digest_hex(&fixture.layer_digest).unwrap())
                .is_file()
        );

        let index: Value =
            serde_json::from_slice(&fs::read(outdir.join("index.json")).unwrap()).unwrap();
        assert_eq!(index["manifests"][0]["digest"], fixture.manifest_digest);
        assert_eq!(index["manifests"][0]["platform"]["architecture"], "arm64");
        assert_eq!(index["manifests"][0]["platform"]["os"], "linux");

        let paths = observed_paths.lock().unwrap();
        assert_eq!(
            paths.as_slice(),
            &[
                "/v2/token".to_owned(),
                "/v2/distroless/static/manifests/".to_owned(),
                "/v2/distroless/static/blobs/".to_owned(),
                "/v2/distroless/static/blobs/".to_owned(),
            ]
        );
    }

    #[derive(Clone)]
    struct RegistryFixture {
        manifest_digest: String,
        manifest_bytes: Vec<u8>,
        config_digest: String,
        config_bytes: Vec<u8>,
        layer_digest: String,
        layer_bytes: Vec<u8>,
    }

    impl RegistryFixture {
        fn new() -> Self {
            let config_bytes = br#"{"architecture":"arm64","os":"linux"}"#.to_vec();
            let layer_bytes = b"layer-tar-gzip-bytes".to_vec();
            let config_digest = sha256_digest(&config_bytes);
            let layer_digest = sha256_digest(&layer_bytes);
            let manifest_bytes = format!(
                r#"{{"schemaVersion":2,"mediaType":"{DEFAULT_MANIFEST_MEDIA_TYPE}","config":{{"mediaType":"application/vnd.oci.image.config.v1+json","digest":"{config_digest}","size":{}}},"layers":[{{"mediaType":"application/vnd.oci.image.layer.v1.tar+gzip","digest":"{layer_digest}","size":{}}}]}}"#,
                config_bytes.len(),
                layer_bytes.len()
            )
            .into_bytes();
            let manifest_digest = sha256_digest(&manifest_bytes);
            Self {
                manifest_digest,
                manifest_bytes,
                config_digest,
                config_bytes,
                layer_digest,
                layer_bytes,
            }
        }
    }

    fn sha256_digest(bytes: &[u8]) -> String {
        let digest = Sha256::digest(bytes);
        format!("sha256:{}", hex_lower(&digest))
    }

    fn unique_temp_dir(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "oya-oci-pull-base-test-{}-{test_name}-{nanos}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn start_registry_fixture(
        fixture: RegistryFixture,
        observed_paths: Arc<Mutex<Vec<String>>>,
        request_count: usize,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let server = thread::spawn(move || {
            for _ in 0..request_count {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_http_request(&mut stream);
                observed_paths
                    .lock()
                    .unwrap()
                    .push(normalize_observed_path(&request.path));
                respond(&mut stream, &request, &fixture);
            }
        });
        (address, server)
    }

    #[derive(Debug)]
    struct RequestSummary {
        path: String,
    }

    fn read_http_request(stream: &mut TcpStream) -> RequestSummary {
        let mut header = Vec::new();
        let mut byte = [0_u8; 1];
        while !header.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            header.push(byte[0]);
        }
        let header_text = String::from_utf8(header).unwrap();
        let request_line = header_text.lines().next().unwrap();
        let path = request_line
            .split_whitespace()
            .nth(1)
            .expect("request path")
            .to_owned();
        RequestSummary { path }
    }

    fn normalize_observed_path(path: &str) -> String {
        if path.starts_with("/v2/token") {
            return "/v2/token".to_owned();
        }
        if let Some((prefix, _digest)) = path.split_once("sha256:") {
            return prefix.to_owned();
        }
        if let Some((prefix, _query)) = path.split_once("&scope=") {
            return prefix.to_owned();
        }
        path.to_owned()
    }

    fn respond(stream: &mut TcpStream, request: &RequestSummary, fixture: &RegistryFixture) {
        let (status, content_type, body): (&str, &str, Vec<u8>) =
            if request.path.starts_with("/v2/token") {
                (
                    "200 OK",
                    "application/json",
                    br#"{"token":"fixture-token"}"#.to_vec(),
                )
            } else if request.path.ends_with(&fixture.manifest_digest) {
                (
                    "200 OK",
                    DEFAULT_MANIFEST_MEDIA_TYPE,
                    fixture.manifest_bytes.clone(),
                )
            } else if request.path.ends_with(&fixture.config_digest) {
                (
                    "200 OK",
                    "application/octet-stream",
                    fixture.config_bytes.clone(),
                )
            } else if request.path.ends_with(&fixture.layer_digest) {
                (
                    "200 OK",
                    "application/octet-stream",
                    fixture.layer_bytes.clone(),
                )
            } else {
                ("404 Not Found", "text/plain", b"not found".to_vec())
            };
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(&body).unwrap();
    }
}
