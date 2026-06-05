#![cfg_attr(test, allow(clippy::expect_used, clippy::panic, clippy::unwrap_used))]

// tools/oci/crates/oya-oci-push-app/src/main.rs
//
// oya-oci-push — build-host OCI Distribution push client.
//
// Reads a complete OCI Image Layout directory (the output of oya-oci-assemble),
// uploads every manifest-referenced blob to an OCI Distribution registry, then
// publishes the manifest under the requested tag. The command intentionally
// avoids docker/crane/skopeo so Buck2/Prow lanes can run with a narrow Rust
// build-host toolchain.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use reqwest::{
    StatusCode,
    blocking::{Client, Response},
    header::{CONTENT_LENGTH, CONTENT_TYPE, LOCATION},
};
use serde_json::Value;

const DEFAULT_MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";

#[derive(Debug, Parser)]
#[command(
    name = "oya-oci-push",
    about = "Push an OCI Image Layout directory to an OCI Distribution registry"
)]
struct Args {
    /// Directory containing oci-layout, index.json, and blobs/sha256/*.
    oci_layout_dir: PathBuf,

    /// Registry host[:port], without scheme.
    registry: String,

    /// Repository path inside the registry.
    repository: String,

    /// Tag to publish the manifest under.
    tag: String,

    /// Use plain HTTP instead of HTTPS for in-cluster registries.
    #[arg(long)]
    insecure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlobToPush {
    digest: String,
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedImage {
    manifest_digest: String,
    manifest_media_type: String,
    manifest_bytes: Vec<u8>,
    blobs: Vec<BlobToPush>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let pushed_digest = push_oci_layout(&args)?;
    println!("{pushed_digest}");
    Ok(())
}

fn push_oci_layout(args: &Args) -> Result<String> {
    let base = registry_base(&args.registry, args.insecure);
    let image = load_oci_layout(&args.oci_layout_dir)?;
    let client = Client::new();

    eprintln!("==> Pushing {}/{}:{}", base, args.repository, args.tag);

    for blob in &image.blobs {
        push_blob(&client, &base, &args.repository, blob)?;
    }

    let pushed_digest = push_manifest(
        &client,
        &base,
        &args.repository,
        &args.tag,
        &image.manifest_media_type,
        &image.manifest_bytes,
        &image.manifest_digest,
    )?;

    eprintln!("==> Pushed manifest digest: {pushed_digest}");
    Ok(pushed_digest)
}

fn registry_base(registry: &str, insecure: bool) -> String {
    let scheme = if insecure { "http" } else { "https" };
    format!("{scheme}://{}", registry.trim_end_matches('/'))
}

fn load_oci_layout(layout: &PathBuf) -> Result<LoadedImage> {
    if !layout.join("oci-layout").is_file() {
        bail!(
            "{} is not an OCI Image Layout: missing oci-layout marker",
            layout.display()
        );
    }

    let blobs_dir = layout.join("blobs").join("sha256");
    let index_path = layout.join("index.json");
    let index = read_json(&index_path)?;
    let manifests = index
        .get("manifests")
        .and_then(Value::as_array)
        .context("index.json has no manifests array")?;
    let manifest_descriptor = manifests
        .first()
        .context("index.json manifests array is empty")?;
    let manifest_digest = required_string(manifest_descriptor, "digest", "index manifest digest")?;
    let manifest_path = digest_path(&blobs_dir, manifest_digest)?;
    let manifest_bytes = fs::read(&manifest_path)
        .with_context(|| format!("failed to read manifest blob {}", manifest_path.display()))?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .with_context(|| format!("failed to parse manifest blob {}", manifest_path.display()))?;

    let manifest_media_type = manifest
        .get("mediaType")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_MANIFEST_MEDIA_TYPE)
        .to_owned();

    let mut blobs = Vec::new();
    let config = manifest
        .get("config")
        .context("manifest has no config descriptor")?;
    push_descriptor(&mut blobs, &blobs_dir, config, "manifest config")?;

    let layers = manifest
        .get("layers")
        .and_then(Value::as_array)
        .context("manifest has no layers array")?;
    for (index, layer) in layers.iter().enumerate() {
        push_descriptor(
            &mut blobs,
            &blobs_dir,
            layer,
            &format!("manifest layer {index}"),
        )?;
    }

    Ok(LoadedImage {
        manifest_digest: manifest_digest.to_owned(),
        manifest_media_type,
        manifest_bytes,
        blobs,
    })
}

fn read_json(path: &PathBuf) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn push_descriptor(
    blobs: &mut Vec<BlobToPush>,
    blobs_dir: &PathBuf,
    descriptor: &Value,
    context: &str,
) -> Result<()> {
    let digest = required_string(descriptor, "digest", context)?;
    blobs.push(BlobToPush {
        digest: digest.to_owned(),
        path: digest_path(blobs_dir, digest)?,
    });
    Ok(())
}

fn required_string<'a>(value: &'a Value, field: &str, context: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("{context} is missing string field {field}"))
}

fn digest_path(blobs_dir: &PathBuf, digest: &str) -> Result<PathBuf> {
    let hex = digest_hex(digest)?;
    Ok(blobs_dir.join(hex))
}

fn digest_hex(digest: &str) -> Result<&str> {
    let hex = digest
        .strip_prefix("sha256:")
        .with_context(|| format!("unsupported digest {digest}: expected sha256:<hex>"))?;
    if hex.len() != 64 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        bail!("invalid sha256 digest {digest}: expected 64 hex characters");
    }
    Ok(hex)
}

fn push_blob(client: &Client, base: &str, repo: &str, blob: &BlobToPush) -> Result<()> {
    let blob_url = format!("{base}/v2/{repo}/blobs/{}", blob.digest);
    let status = client
        .head(&blob_url)
        .send()
        .with_context(|| format!("HEAD {blob_url} failed"))?
        .status();
    if status == StatusCode::OK {
        eprintln!("    blob {}... already present, skip", &blob.digest[..19]);
        return Ok(());
    }

    let upload_url = format!("{base}/v2/{repo}/blobs/uploads/");
    let response = client
        .post(&upload_url)
        .send()
        .with_context(|| format!("POST {upload_url} failed"))?;
    let location = upload_location(base, response, &blob.digest)?;
    let put_url = append_digest_query(&location, &blob.digest);
    let data = fs::read(&blob.path)
        .with_context(|| format!("failed to read blob {}", blob.path.display()))?;
    let byte_count = data.len();

    let response = client
        .put(&put_url)
        .header(CONTENT_TYPE, "application/octet-stream")
        .header(CONTENT_LENGTH, byte_count.to_string())
        .body(data)
        .send()
        .with_context(|| format!("PUT {put_url} failed"))?;
    ensure_status(
        response,
        &[StatusCode::CREATED, StatusCode::ACCEPTED],
        &format!("PUT blob {}", blob.digest),
    )?;
    eprintln!(
        "    pushed blob {}... ({} bytes)",
        &blob.digest[..19],
        byte_count
    );
    Ok(())
}

fn upload_location(base: &str, response: Response, digest: &str) -> Result<String> {
    let response = ensure_status(
        response,
        &[StatusCode::CREATED, StatusCode::ACCEPTED],
        &format!("open upload for {digest}"),
    )?;
    let location = response
        .headers()
        .get(LOCATION)
        .context("upload POST returned no Location header")?
        .to_str()
        .context("upload POST Location header is not valid UTF-8")?;
    Ok(resolve_upload_location(base, location))
}

fn resolve_upload_location(base: &str, location: &str) -> String {
    if location.starts_with("http://") || location.starts_with("https://") {
        return location.to_owned();
    }
    if location.starts_with('/') {
        return format!("{}{}", base.trim_end_matches('/'), location);
    }
    format!("{}/{}", base.trim_end_matches('/'), location)
}

fn append_digest_query(location: &str, digest: &str) -> String {
    let separator = if location.contains('?') { '&' } else { '?' };
    format!("{location}{separator}digest={digest}")
}

fn push_manifest(
    client: &Client,
    base: &str,
    repo: &str,
    tag: &str,
    media_type: &str,
    manifest_bytes: &[u8],
    fallback_digest: &str,
) -> Result<String> {
    let manifest_url = format!("{base}/v2/{repo}/manifests/{tag}");
    let response = client
        .put(&manifest_url)
        .header(CONTENT_TYPE, media_type)
        .body(manifest_bytes.to_vec())
        .send()
        .with_context(|| format!("PUT {manifest_url} failed"))?;
    let response = ensure_status(
        response,
        &[StatusCode::CREATED, StatusCode::ACCEPTED],
        "PUT manifest",
    )?;
    let pushed_digest = response
        .headers()
        .get("Docker-Content-Digest")
        .and_then(|value| value.to_str().ok())
        .unwrap_or(fallback_digest)
        .to_owned();
    Ok(pushed_digest)
}

fn ensure_status(response: Response, ok_statuses: &[StatusCode], action: &str) -> Result<Response> {
    let status = response.status();
    if ok_statuses.contains(&status) {
        return Ok(response);
    }
    let body = response
        .text()
        .unwrap_or_else(|error| format!("<failed to read response body: {error}>"));
    let snippet: String = body.chars().take(300).collect();
    bail!("{action} failed: HTTP {status}: {snippet:?}")
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

    const MANIFEST_DIGEST: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const CONFIG_DIGEST: &str =
        "sha256:2222222222222222222222222222222222222222222222222222222222222222";
    const LAYER_DIGEST: &str =
        "sha256:3333333333333333333333333333333333333333333333333333333333333333";
    const REGISTRY_DIGEST: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn registry_base_selects_scheme() {
        assert_eq!(
            registry_base("registry.local:5000/", true),
            "http://registry.local:5000"
        );
        assert_eq!(
            registry_base("registry.local", false),
            "https://registry.local"
        );
    }

    #[test]
    fn digest_hex_requires_sha256_64_hex() {
        assert_eq!(
            digest_hex(CONFIG_DIGEST).unwrap(),
            "2222222222222222222222222222222222222222222222222222222222222222"
        );
        assert!(digest_hex("sha512:2222").is_err());
        assert!(digest_hex("sha256:bad").is_err());
        assert!(
            digest_hex("sha256:zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz")
                .is_err()
        );
    }

    #[test]
    fn upload_location_resolution_handles_common_registry_forms() {
        assert_eq!(
            resolve_upload_location("http://registry:5000", "/v2/repo/blobs/uploads/1"),
            "http://registry:5000/v2/repo/blobs/uploads/1"
        );
        assert_eq!(
            resolve_upload_location("http://registry:5000", "v2/repo/blobs/uploads/1"),
            "http://registry:5000/v2/repo/blobs/uploads/1"
        );
        assert_eq!(
            resolve_upload_location("http://registry:5000", "https://other/upload"),
            "https://other/upload"
        );
    }

    #[test]
    fn append_digest_query_preserves_existing_query() {
        assert_eq!(
            append_digest_query("http://registry/upload", CONFIG_DIGEST),
            format!("http://registry/upload?digest={CONFIG_DIGEST}")
        );
        assert_eq!(
            append_digest_query("http://registry/upload?mount=x", CONFIG_DIGEST),
            format!("http://registry/upload?mount=x&digest={CONFIG_DIGEST}")
        );
    }

    #[test]
    fn load_layout_extracts_manifest_and_blob_descriptors() {
        let layout = write_fixture_layout("load_layout_extracts_manifest_and_blob_descriptors");
        let image = load_oci_layout(&layout).unwrap();

        assert_eq!(image.manifest_digest, MANIFEST_DIGEST);
        assert_eq!(image.manifest_media_type, DEFAULT_MANIFEST_MEDIA_TYPE);
        assert_eq!(image.blobs.len(), 2);
        assert_eq!(image.blobs[0].digest, CONFIG_DIGEST);
        assert_eq!(image.blobs[1].digest, LAYER_DIGEST);
    }

    #[test]
    fn push_layout_uploads_missing_blobs_before_manifest() {
        let layout = write_fixture_layout("push_layout_uploads_missing_blobs_before_manifest");
        let observed_paths = Arc::new(Mutex::new(Vec::new()));
        let (address, server) = start_registry_fixture(Arc::clone(&observed_paths), 7);

        let args = Args {
            oci_layout_dir: layout,
            registry: address,
            repository: "oya-ci/controller".to_owned(),
            tag: "dev".to_owned(),
            insecure: true,
        };
        let pushed_digest = push_oci_layout(&args).unwrap();
        server.join().unwrap();

        assert_eq!(pushed_digest, REGISTRY_DIGEST);
        let paths = observed_paths.lock().unwrap();
        assert_eq!(
            paths.as_slice(),
            &[
                format!("HEAD /v2/oya-ci/controller/blobs/{CONFIG_DIGEST}"),
                "POST /v2/oya-ci/controller/blobs/uploads/".to_owned(),
                format!("PUT /v2/oya-ci/controller/blobs/uploads/session?digest={CONFIG_DIGEST}"),
                format!("HEAD /v2/oya-ci/controller/blobs/{LAYER_DIGEST}"),
                "POST /v2/oya-ci/controller/blobs/uploads/".to_owned(),
                format!("PUT /v2/oya-ci/controller/blobs/uploads/session?digest={LAYER_DIGEST}"),
                "PUT /v2/oya-ci/controller/manifests/dev".to_owned(),
            ]
        );
    }

    fn write_fixture_layout(test_name: &str) -> PathBuf {
        let root = unique_temp_dir(test_name);
        let blobs = root.join("blobs").join("sha256");
        fs::create_dir_all(&blobs).unwrap();
        fs::write(root.join("oci-layout"), r#"{"imageLayoutVersion":"1.0.0"}"#).unwrap();
        fs::write(
            root.join("index.json"),
            format!(
                r#"{{"schemaVersion":2,"manifests":[{{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"{MANIFEST_DIGEST}","size":123}}]}}"#
            ),
        )
        .unwrap();
        fs::write(blobs.join(digest_hex(CONFIG_DIGEST).unwrap()), b"config").unwrap();
        fs::write(blobs.join(digest_hex(LAYER_DIGEST).unwrap()), b"layer").unwrap();
        fs::write(
            blobs.join(digest_hex(MANIFEST_DIGEST).unwrap()),
            format!(
                r#"{{"schemaVersion":2,"mediaType":"{DEFAULT_MANIFEST_MEDIA_TYPE}","config":{{"mediaType":"application/vnd.oci.image.config.v1+json","digest":"{CONFIG_DIGEST}","size":6}},"layers":[{{"mediaType":"application/vnd.oci.image.layer.v1.tar+gzip","digest":"{LAYER_DIGEST}","size":5}}]}}"#
            ),
        )
        .unwrap();
        root
    }

    fn unique_temp_dir(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "oya-oci-push-test-{}-{test_name}-{nanos}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn start_registry_fixture(
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
                    .push(format!("{} {}", request.method, request.path));
                respond(&mut stream, &request);
            }
        });
        (address, server)
    }

    #[derive(Debug)]
    struct RequestSummary {
        method: String,
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
        let mut lines = header_text.lines();
        let request_line = lines.next().unwrap();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap().to_owned();
        let path = parts.next().unwrap().to_owned();
        let content_length = lines
            .filter_map(|line| line.split_once(':'))
            .find_map(|(name, value)| {
                if name.eq_ignore_ascii_case("content-length") {
                    value.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let mut body = vec![0_u8; content_length];
        if content_length > 0 {
            stream.read_exact(&mut body).unwrap();
        }
        RequestSummary { method, path }
    }

    fn respond(stream: &mut TcpStream, request: &RequestSummary) {
        let mut status = "201 Created";
        let mut headers = String::new();
        if request.method == "HEAD" {
            status = "404 Not Found";
        } else if request.method == "POST" {
            status = "202 Accepted";
            headers.push_str("Location: /v2/oya-ci/controller/blobs/uploads/session\r\n");
        } else if request.path == "/v2/oya-ci/controller/manifests/dev" {
            headers.push_str(&format!("Docker-Content-Digest: {REGISTRY_DIGEST}\r\n"));
        }
        write!(
            stream,
            "HTTP/1.1 {status}\r\n{headers}Content-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
    }
}
