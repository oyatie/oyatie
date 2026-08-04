#![forbid(unsafe_code)]
//! Cache-wiring resolver + canary toolkit (ADR-0560). LOCAL BRIDGE feedback only
//! (founder CLI-retirement directive): merge authority is the conformance gate
//! test, never this binary; CI workflows invoke it as a job step (`buck2 run`),
//! and its successors are reconcilers per ADR-0556 D4 / ADR-0555 D4.
//!
//! Subcommands:
//!   resolve --build-class C [--require-bypass]
//!   run (--build-class C | --warm-probe | --workflow-mode reader|writer)
//!       [--prelicense-seed] [--prelicense-probe true|false] [--mode-out PATH] -- COMMAND [ARG...]
//!   license-state                       (prints `warm_licensed=<bool>` for $GITHUB_OUTPUT)
//!   report --record PATH --build-class C [--mode M] [--out PATH]
//!   assert-warm --record PATH --build-class C --mode M
//!   assert-cold --record PATH
//!   hash-outputs --show-output PATH [--out PATH]
//!   writer-receipt --record PATH --manifest PATH --outputs PATH [--out PATH]
//!   canary-verdict --cold PATH [--warm PATH --warm-record PATH] [--out PATH]
//!   canary-targets                      (prints the pinned target set, one per line)
//!   issue-identity --role R --pki-mount M --pki-role R --uri-san URI

use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};
use std::sync::Arc;
use std::time::Duration;

use ci_build_cache_policy as app;
use prost::Message;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde_json::Value;

const WRITER_RECEIPT_SCHEMA: &str = "oya-ci/cas-writer-receipt/v1";

fn fail(message: &str) -> ExitCode {
    eprintln!("oya-cloud-ci-cache-wiring: {message}");
    ExitCode::from(2)
}

fn repo_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("current_dir: {e}"))?;
    app::repo_root_from(&cwd)
        .ok_or_else(|| "failed to locate repo root (specs/root-hub-pointers.json)".to_string())
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn bool_flag_value(args: &[String], flag: &str) -> Result<bool, String> {
    let Some(index) = args.iter().position(|argument| argument == flag) else {
        return Ok(false);
    };
    match args.get(index + 1).map(String::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(value) if value.starts_with("--") => Err(format!(
            "{flag} requires an explicit `true` or `false` value before `{value}`"
        )),
        None => Err(format!(
            "{flag} requires an explicit `true` or `false` value"
        )),
        Some(value) => Err(format!("{flag} must be `true` or `false`, got `{value}`")),
    }
}

fn read_json(path: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {path}: {e}"))
}

fn write_out(out: Option<String>, payload: &str) -> Result<(), String> {
    match out {
        Some(path) => {
            if let Some(parent) = Path::new(&path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
            }
            fs::write(&path, payload).map_err(|e| format!("write {path}: {e}"))
        }
        None => {
            println!("{payload}");
            Ok(())
        }
    }
}

fn child_exit(status: ExitStatus) -> ExitCode {
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from)
}

fn run_child(root: &Path, command: &[String]) -> Result<ExitStatus, String> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| "run requires a child command after `--`".to_string())?;
    Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|error| format!("spawn child `{program}`: {error}"))
}

fn controlled_buck2_command(command: &[String]) -> Result<(Vec<String>, String), String> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| "run requires a child command after `--`".to_string())?;
    if Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        != Some("buck2")
    {
        return Err(
            "warm controller child must be `buck2` so its daemon boundary is explicit".to_string(),
        );
    }
    let mut child = command.to_vec();
    let isolation = args
        .windows(2)
        .find(|pair| pair[0] == "--isolation-dir")
        .map(|pair| pair[1].clone())
        .or_else(|| {
            args.iter()
                .find_map(|arg| arg.strip_prefix("--isolation-dir=").map(str::to_owned))
        })
        .unwrap_or_else(|| {
            child.insert(1, "--isolation-dir".to_string());
            child.insert(2, "oya-cache-controlled".to_string());
            "oya-cache-controlled".to_string()
        });
    Ok((child, isolation))
}

fn kill_buck2(root: &Path, isolation: &str) -> Result<(), String> {
    let status = Command::new("buck2")
        .args(["--isolation-dir", isolation, "kill"])
        .current_dir(root)
        .status()
        .map_err(|error| format!("spawn `buck2 kill`: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "`buck2 --isolation-dir {isolation} kill` failed with {status}"
        ))
    }
}

fn controlled_child(
    root: &Path,
    resolution: &app::Resolution,
    command: &[String],
) -> Result<ExitCode, String> {
    let local = root.join(".buckconfig.local");
    if local.exists() {
        return Err(format!(
            "refusing pre-existing {}: controller must own the entire cache config lifecycle",
            local.display()
        ));
    }
    if resolution.mode == app::CacheMode::Bypass {
        return run_child(root, command).map(child_exit);
    }

    let overlay_path = match resolution.mode {
        app::CacheMode::WarmReadOnly => app::OVERLAY_RO_PATH,
        app::CacheMode::WarmReadWrite => app::OVERLAY_RW_PATH,
        app::CacheMode::Bypass => unreachable!(),
    };
    let overlay = fs::read_to_string(root.join(overlay_path))
        .map_err(|error| format!("read {overlay_path}: {error}"))?;
    let cert = std::env::var(app::CLIENT_CERT_ENV).ok();
    let ca = std::env::var(app::TLS_CA_CERTS_ENV).ok();
    let config = app::effective_buckconfig(resolution, &overlay, cert.as_deref(), ca.as_deref())?
        .ok_or_else(|| "warm resolution produced no effective config".to_string())?;

    // Buck2 reads RE client configuration only at daemon startup. The two kills
    // are the boundary: start the child after the private config exists, then
    // stop that daemon before deleting the config so a later cold child cannot
    // inherit warm state.
    let (child_command, isolation) = controlled_buck2_command(command)?;
    kill_buck2(root, &isolation)?;
    let path = app::install_local_buckconfig(root, &config)?;
    let child = run_child(root, &child_command);
    let stop = kill_buck2(root, &isolation);
    let remove = app::remove_local_buckconfig(&path);
    match (stop, remove) {
        (Err(stop), Err(remove)) => {
            return Err(format!(
                "{stop}; additionally failed cache config cleanup: {remove}"
            ));
        }
        (Err(error), Ok(())) | (Ok(()), Err(error)) => return Err(error),
        (Ok(()), Ok(())) => {}
    }
    child.map(child_exit)
}

fn required_env(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("required environment variable {name} is missing"))
}

const WRITER_ENDPOINT: &str = "nativelink-cas-writer.oya-ci.svc.cluster.local:50051";
const READER_ENDPOINT: &str = "nativelink-cas-reader.oya-ci.svc.cluster.local:50052";
const CAPABILITIES_PATH: &str = "/build.bazel.remote.execution.v2.Capabilities/GetCapabilities";

#[derive(Clone, PartialEq, Message)]
struct GetCapabilitiesRequest {
    #[prost(string, tag = "1")]
    instance_name: String,
}

#[derive(Clone, PartialEq, Message)]
struct ServerCapabilities {
    #[prost(message, optional, tag = "1")]
    cache_capabilities: Option<CacheCapabilities>,
    #[prost(message, optional, tag = "2")]
    execution_capabilities: Option<ExecutionCapabilities>,
    #[prost(message, optional, tag = "4")]
    low_api_version: Option<SemVer>,
    #[prost(message, optional, tag = "5")]
    high_api_version: Option<SemVer>,
}

#[derive(Clone, PartialEq, Message)]
struct CacheCapabilities {
    #[prost(int32, repeated, packed = "true", tag = "1")]
    digest_functions: Vec<i32>,
}

#[derive(Clone, PartialEq, Message)]
struct ExecutionCapabilities {}

#[derive(Clone, PartialEq, Message)]
struct SemVer {
    #[prost(int32, tag = "1")]
    major: i32,
    #[prost(int32, tag = "2")]
    minor: i32,
    #[prost(int32, tag = "3")]
    patch: i32,
    #[prost(string, tag = "4")]
    prerelease: String,
}

fn curl_metadata(stdout: &[u8]) -> Result<(&str, &str), String> {
    let stdout =
        std::str::from_utf8(stdout).map_err(|_| "curl metadata was not UTF-8".to_string())?;
    let mut lines = stdout.lines();
    let version = lines.next().unwrap_or_default();
    let status = lines.next().unwrap_or_default();
    if lines.next().is_some() {
        return Err("curl metadata contained unexpected extra lines".to_string());
    }
    Ok((version, status))
}

fn grpc_frame(payload: &[u8]) -> Result<&[u8], String> {
    let header: [u8; 5] = payload
        .get(..5)
        .ok_or_else(|| "Capabilities response missing the five-byte gRPC frame header".to_string())?
        .try_into()
        .map_err(|_| "Capabilities response gRPC header was not five bytes".to_string())?;
    if header[0] != 0 {
        return Err("Capabilities response used unsupported gRPC compression".to_string());
    }
    let length = u32::from_be_bytes(
        header[1..]
            .try_into()
            .map_err(|_| "Capabilities response gRPC length was not four bytes".to_string())?,
    ) as usize;
    if payload.len() != 5 + length {
        return Err(format!(
            "Capabilities response gRPC frame length mismatch: declared {length}, received {}",
            payload.len().saturating_sub(5)
        ));
    }
    Ok(&payload[5..])
}

fn grpc_message(message: &[u8]) -> Result<Vec<u8>, String> {
    let length = u32::try_from(message.len())
        .map_err(|_| "gRPC request message exceeded the four-byte length field".to_string())?;
    let mut frame = Vec::with_capacity(5 + message.len());
    frame.push(0);
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(message);
    Ok(frame)
}

fn validate_server_capabilities(message: &[u8]) -> Result<(), String> {
    let capabilities = ServerCapabilities::decode(message)
        .map_err(|error| format!("decode REAPI ServerCapabilities: {error}"))?;
    let cache = capabilities
        .cache_capabilities
        .ok_or_else(|| "ServerCapabilities omitted cache_capabilities".to_string())?;
    if !cache.digest_functions.contains(&1) {
        return Err("cache_capabilities did not advertise SHA256 digest function".to_string());
    }
    if capabilities.execution_capabilities.is_some() {
        return Err(
            "cache-only endpoint unexpectedly advertised execution_capabilities".to_string(),
        );
    }
    let low = capabilities
        .low_api_version
        .ok_or_else(|| "ServerCapabilities omitted low_api_version".to_string())?;
    let high = capabilities
        .high_api_version
        .ok_or_else(|| "ServerCapabilities omitted high_api_version".to_string())?;
    let valid_component = |version: &SemVer| {
        version.major == 2
            && version.minor >= 0
            && version.patch >= 0
            && version.prerelease.is_empty()
    };
    if !valid_component(&low) || !valid_component(&high) {
        return Err(format!(
            "REAPI version range must contain released major-2 versions, got {}.{}.{}..{}.{}.{}",
            low.major, low.minor, low.patch, high.major, high.minor, high.patch
        ));
    }
    if (low.major, low.minor, low.patch) > (high.major, high.minor, high.patch) {
        return Err("REAPI low_api_version exceeded high_api_version".to_string());
    }
    Ok(())
}

fn github_green_provenance_from(
    status: app::CanaryStatus,
    env: impl Fn(&str) -> Option<String>,
) -> Result<Option<Value>, String> {
    if status != app::CanaryStatus::Green || env("GITHUB_ACTIONS").as_deref() != Some("true") {
        return Ok(None);
    }
    let required = |name| {
        env(name)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                format!("GREEN GitHub Actions verdict requires non-empty {name} provenance")
            })
    };
    Ok(Some(serde_json::json!({
        "github_sha": required("GITHUB_SHA")?,
        "github_run_id": required("GITHUB_RUN_ID")?,
        "github_job": required("GITHUB_JOB")?,
        "github_run_attempt": required("GITHUB_RUN_ATTEMPT")?,
    })))
}

fn bind_github_green_provenance(
    status: app::CanaryStatus,
    verdict: &mut Value,
) -> Result<(), String> {
    if let Some(provenance) = github_green_provenance_from(status, |name| std::env::var(name).ok())?
    {
        verdict["provenance"] = provenance;
    }
    Ok(())
}

fn required_value(name: &str, env: &impl Fn(&str) -> Option<String>) -> Result<String, String> {
    env(name)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("writer receipt requires non-empty {name}"))
}

fn writer_receipt_from(
    record: &Value,
    manifest: &Value,
    outputs: &str,
    env: impl Fn(&str) -> Option<String>,
) -> Result<Value, String> {
    app::assert_writer_seed_record(record).map_err(|findings| {
        format!(
            "writer receipt refused an invalid seed record: {}",
            findings.join("; ")
        )
    })?;
    let supplied_manifest = app::manifest_from_json(manifest)?;
    let recomputed_manifest = app::digest_manifest_from_show_output(outputs)?;
    if supplied_manifest != recomputed_manifest {
        return Err(
            "writer receipt manifest differs from freshly hashed Buck output bindings".to_string(),
        );
    }
    let required = |name| required_value(name, &env);
    let repository = required("GITHUB_REPOSITORY")?;
    let event = required("GITHUB_EVENT_NAME")?;
    let git_ref = required("GITHUB_REF")?;
    let workflow_ref = required("GITHUB_WORKFLOW_REF")?;
    if env("GITHUB_ACTIONS").as_deref() != Some("true")
        || repository != "jason931225/oyatie"
        || event != "push"
        || git_ref != "refs/heads/dev"
        || workflow_ref != "jason931225/oyatie/.github/workflows/oya-ci-required.yml@refs/heads/dev"
    {
        return Err(
            "writer receipt requires the trusted dev-push oya-ci-required workflow".to_string(),
        );
    }
    let run_id = required("GITHUB_RUN_ID")?;
    if !run_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("writer receipt GITHUB_RUN_ID must contain only digits".to_string());
    }
    let output_lines = outputs
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if output_lines.is_empty() {
        return Err("writer receipt requires non-empty Buck output bindings".to_string());
    }
    Ok(serde_json::json!({
        "schema": WRITER_RECEIPT_SCHEMA,
        "github_repository": repository,
        "github_sha": required("GITHUB_SHA")?,
        "github_run_id": run_id,
        "github_job": required("GITHUB_JOB")?,
        "github_run_attempt": required("GITHUB_RUN_ATTEMPT")?,
        "github_workflow_ref": workflow_ref,
        "github_event_name": event,
        "github_ref": git_ref,
        "runner_name": required("RUNNER_NAME")?,
        "runner_pod_name": required("OYA_RUNNER_POD_NAME")?,
        "runner_pod_uid": required("OYA_RUNNER_POD_UID")?,
        "runner_node_name": required("OYA_RUNNER_NODE_NAME")?,
        "outputs": output_lines,
        "manifest": manifest,
        "cache_report": app::cache_hit_report(
            record,
            "postmerge-dev-trunk-prelicense-seed",
            "warm-rw"
        ),
    }))
}

fn receipt_string<'a>(receipt: &'a Value, key: &str) -> Result<&'a str, String> {
    receipt
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("writer receipt field {key} missing or non-string"))
}

fn validate_writer_receipt_from(
    receipt: &Value,
    writer_manifest: &Value,
    cold: &std::collections::BTreeMap<String, String>,
    warm: &std::collections::BTreeMap<String, String>,
    expected_run_id: &str,
    env: impl Fn(&str) -> Option<String>,
) -> Result<Value, String> {
    if expected_run_id.is_empty() || !expected_run_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("writer run id must contain only digits".to_string());
    }
    if receipt.get("schema").and_then(Value::as_str) != Some(WRITER_RECEIPT_SCHEMA) {
        return Err("writer receipt schema mismatch".to_string());
    }
    let required = |name| required_value(name, &env);
    let current_sha = required("GITHUB_SHA")?;
    for (key, expected) in [
        ("github_repository", "jason931225/oyatie"),
        ("github_sha", current_sha.as_str()),
        ("github_run_id", expected_run_id),
        ("github_event_name", "push"),
        ("github_ref", "refs/heads/dev"),
        (
            "github_workflow_ref",
            "jason931225/oyatie/.github/workflows/oya-ci-required.yml@refs/heads/dev",
        ),
    ] {
        if receipt_string(receipt, key)? != expected {
            return Err(format!("writer receipt {key} did not match {expected:?}"));
        }
    }
    if receipt.get("manifest") != Some(writer_manifest) {
        return Err("writer receipt embedded manifest differs from artifact manifest".to_string());
    }
    for key in ["github_job", "runner_pod_name", "runner_node_name"] {
        receipt_string(receipt, key)?;
    }
    let attempt = receipt_string(receipt, "github_run_attempt")?;
    if !attempt.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("writer receipt github_run_attempt must contain only digits".to_string());
    }
    let _outputs = receipt
        .get("outputs")
        .and_then(Value::as_array)
        .filter(|outputs| {
            !outputs.is_empty()
                && outputs
                    .iter()
                    .all(|output| output.as_str().is_some_and(|value| !value.is_empty()))
        })
        .ok_or_else(|| "writer receipt outputs missing or malformed".to_string())?;
    if receipt
        .pointer("/cache_report/schema")
        .and_then(Value::as_str)
        != Some(app::CACHE_HIT_REPORT_SCHEMA)
    {
        return Err("writer receipt cache_report schema mismatch".to_string());
    }
    if receipt_string(receipt, "runner_name")? == required("RUNNER_NAME")? {
        return Err("writer and reader used the same GitHub runner".to_string());
    }
    if receipt_string(receipt, "runner_pod_uid")? == required("OYA_RUNNER_POD_UID")? {
        return Err("writer and reader used the same ARC pod UID".to_string());
    }
    let writer = app::manifest_from_json(writer_manifest)?;
    if &writer != cold || &writer != warm {
        return Err("writer/cold/reader manifest parity gap".to_string());
    }
    Ok(serde_json::json!({
        "github_repository": receipt_string(receipt, "github_repository")?,
        "github_sha": receipt_string(receipt, "github_sha")?,
        "github_run_id": receipt_string(receipt, "github_run_id")?,
        "github_job": receipt_string(receipt, "github_job")?,
        "github_run_attempt": receipt_string(receipt, "github_run_attempt")?,
        "runner_name": receipt_string(receipt, "runner_name")?,
        "runner_pod_uid": receipt_string(receipt, "runner_pod_uid")?,
        "runner_node_name": receipt_string(receipt, "runner_node_name")?,
    }))
}

fn grpc_status(headers: &str) -> Result<u16, String> {
    let statuses = headers
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.trim().eq_ignore_ascii_case("grpc-status"))
        .map(|(_, value)| value.trim())
        .collect::<Vec<_>>();
    if statuses.len() != 1 {
        return Err(format!(
            "Capabilities response must contain exactly one grpc-status trailer, found {}",
            statuses.len()
        ));
    }
    statuses[0]
        .parse()
        .map_err(|_| "Capabilities grpc-status was not an unsigned integer".to_string())
}

fn typed_client_auth_alert(error: &std::io::Error) -> Result<&'static str, String> {
    let Some(rustls::Error::AlertReceived(alert)) = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<rustls::Error>())
    else {
        return Err("reader-to-writer failure was not rustls::Error::AlertReceived".to_string());
    };
    match alert {
        rustls::AlertDescription::AccessDenied => Ok("access_denied"),
        rustls::AlertDescription::BadCertificate => Ok("bad_certificate"),
        rustls::AlertDescription::CertificateRequired => Ok("certificate_required"),
        rustls::AlertDescription::CertificateUnknown => Ok("certificate_unknown"),
        rustls::AlertDescription::UnknownCA => Ok("unknown_ca"),
        other => Err(format!(
            "reader-to-writer peer TLS alert was not a client-auth rejection: {other:?}"
        )),
    }
}

fn read_client_auth_rejection(
    connection: &mut rustls::ClientConnection,
    socket: &mut TcpStream,
) -> Result<String, String> {
    for _ in 0..2 {
        match connection.complete_io(socket) {
            Err(error) => return typed_client_auth_alert(&error).map(str::to_string),
            Ok(_) if connection.is_handshaking() => continue,
            // TLS 1.3 peers may send the fatal client-auth alert only after
            // receiving the client's Certificate/CertificateVerify/Finished.
            // One additional read is mandatory before concluding acceptance.
            Ok(_) => continue,
        }
    }
    Err(format!(
        "reader identity completed the writer TLS handshake (ALPN {:?}); expected a typed peer alert before HTTP/2/gRPC",
        connection.alpn_protocol().map(String::from_utf8_lossy)
    ))
}

fn require_reader_tls_rejected_by_writer() -> Result<String, String> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, pem::PemObject};

    let identity_pem = fs::read(required_env(app::CLIENT_CERT_ENV)?)
        .map_err(|error| format!("read reader identity: {error}"))?;
    let ca_pem = fs::read(required_env(app::TLS_CA_CERTS_ENV)?)
        .map_err(|error| format!("read NativeLink server CA: {error}"))?;
    let certificates = CertificateDer::pem_slice_iter(&identity_pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("parse reader certificate chain: {error}"))?;
    if certificates.is_empty() {
        return Err("reader identity contained no certificates".to_string());
    }
    let private_key = PrivateKeyDer::from_pem_slice(&identity_pem)
        .map_err(|error| format!("parse reader private key: {error}"))?;
    let roots = CertificateDer::pem_slice_iter(&ca_pem)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("parse NativeLink server CA: {error}"))?;
    let mut root_store = rustls::RootCertStore::empty();
    let (accepted, rejected) = root_store.add_parsable_certificates(roots);
    if accepted == 0 || rejected != 0 {
        return Err(format!(
            "NativeLink server CA set was not fully parseable ({accepted} accepted, {rejected} rejected)"
        ));
    }
    let mut config =
        rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
            .with_root_certificates(root_store)
            .with_client_auth_cert(certificates, private_key)
            .map_err(|error| format!("configure reader mTLS identity: {error}"))?;
    config.alpn_protocols = vec![b"h2".to_vec()];

    let host = WRITER_ENDPOINT
        .split_once(':')
        .map(|(host, _)| host)
        .ok_or_else(|| "writer endpoint missing port".to_string())?;
    let addresses = WRITER_ENDPOINT
        .to_socket_addrs()
        .map_err(|error| format!("resolve {WRITER_ENDPOINT}: {error}"))?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(format!("resolve {WRITER_ENDPOINT}: no addresses"));
    }
    let mut socket = addresses
        .iter()
        .find_map(|address| TcpStream::connect_timeout(address, Duration::from_secs(5)).ok())
        .ok_or_else(|| format!("connect {WRITER_ENDPOINT}: all addresses failed"))?;
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .and_then(|()| socket.set_write_timeout(Some(Duration::from_secs(10))))
        .map_err(|error| format!("set {WRITER_ENDPOINT} TLS timeout: {error}"))?;
    let server_name = ServerName::try_from(host.to_string())
        .map_err(|error| format!("invalid writer TLS server name: {error}"))?;
    let mut connection = rustls::ClientConnection::new(Arc::new(config), server_name)
        .map_err(|error| format!("create reader-to-writer TLS client: {error}"))?;
    read_client_auth_rejection(&mut connection, &mut socket)
}

fn curl_capabilities(endpoint: &str) -> Result<(), String> {
    let client_pem = required_env(app::CLIENT_CERT_ENV)?;
    let ca_pem = required_env(app::TLS_CA_CERTS_ENV)?;
    let scratch = std::env::temp_dir().join(format!(
        "oya-cas-capabilities-{}-{}",
        std::process::id(),
        endpoint.rsplit(':').next().unwrap_or("unknown")
    ));
    fs::create_dir(&scratch).map_err(|error| format!("create {}: {error}", scratch.display()))?;
    let request = scratch.join("request.grpc");
    let response = scratch.join("response.grpc");
    let headers = scratch.join("headers.txt");
    let request_message = GetCapabilitiesRequest {
        instance_name: "main".to_string(),
    }
    .encode_to_vec();
    fs::write(&request, grpc_message(&request_message)?)
        .map_err(|error| format!("write {}: {error}", request.display()))?;

    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--http2",
            "--tlsv1.3",
            "--tls-max",
            "1.3",
            "--connect-timeout",
            "5",
            "--max-time",
            "15",
            "--cert",
            &client_pem,
            "--key",
            &client_pem,
            "--cacert",
            &ca_pem,
            "--header",
            "content-type: application/grpc",
            "--header",
            "te: trailers",
            "--data-binary",
            &format!("@{}", request.display()),
            "--dump-header",
            headers.to_str().ok_or("non-UTF-8 scratch header path")?,
            "--output",
            response.to_str().ok_or("non-UTF-8 scratch response path")?,
            "--write-out",
            "%{http_version}\n%{http_code}",
            &format!("https://{endpoint}{CAPABILITIES_PATH}"),
        ])
        .output()
        .map_err(|error| format!("spawn curl Capabilities probe: {error}"));

    let result = (|| {
        let output = output?;
        if !output.status.success() {
            return Err(format!(
                "Capabilities probe failed before a typed REAPI response (curl exit {:?}): {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let (version, http_status) = curl_metadata(&output.stdout)?;
        if version != "2" || http_status != "200" {
            return Err(format!(
                "Capabilities probe requires negotiated HTTP/2 and HTTP 200, got HTTP/{version} {http_status}"
            ));
        }
        let header_text = fs::read_to_string(&headers)
            .map_err(|error| format!("read {}: {error}", headers.display()))?;
        if !header_text.lines().any(|line| {
            line.split_once(':').is_some_and(|(name, value)| {
                name.trim().eq_ignore_ascii_case("content-type")
                    && value
                        .trim()
                        .to_ascii_lowercase()
                        .starts_with("application/grpc")
            })
        }) {
            return Err("Capabilities response was not application/grpc".to_string());
        }
        let status = grpc_status(&header_text)?;
        if status != 0 {
            return Err(format!("Capabilities returned grpc-status {status}"));
        }
        validate_server_capabilities(grpc_frame(
            &fs::read(&response)
                .map_err(|error| format!("read {}: {error}", response.display()))?,
        )?)?;
        Ok(())
    })();
    let cleanup = fs::remove_dir_all(&scratch)
        .map_err(|error| format!("remove {}: {error}", scratch.display()));
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}; probe cleanup also failed: {cleanup}")),
    }
}

fn prove_identity_boundary(mode: &str) -> Result<(), String> {
    let positive_endpoint = match mode {
        "writer" => WRITER_ENDPOINT,
        "reader" => READER_ENDPOINT,
        other => return Err(format!("unsupported identity boundary mode `{other}`")),
    };
    curl_capabilities(positive_endpoint)?;
    if mode == "reader" {
        require_reader_tls_rejected_by_writer()?;
    }
    Ok(())
}

fn invocation_record_path(child: &[String]) -> Result<&str, String> {
    child
        .windows(2)
        .find_map(|pair| {
            (pair[0] == "--unstable-write-invocation-record").then_some(pair[1].as_str())
        })
        .or_else(|| {
            child
                .iter()
                .find_map(|argument| argument.strip_prefix("--unstable-write-invocation-record="))
        })
        .ok_or_else(|| {
            "workflow cache proof requires --unstable-write-invocation-record".to_string()
        })
}

fn validate_workflow_record(mode: &str, child: &[String]) -> Result<(), String> {
    let path = invocation_record_path(child)?;
    let document = read_json(path)?;
    let record = app::invocation_record(&document)?;
    let result = match mode {
        "writer" => app::assert_writer_seed_record(record),
        "reader" => app::assert_complete_warm_cache_coverage(record),
        other => return Err(format!("unsupported workflow record mode `{other}`")),
    };
    result.map_err(|findings| {
        format!(
            "{mode} invocation record did not prove the required cache behavior: {}",
            findings.join("; ")
        )
    })
}

fn issue_identity(options: &[String]) -> Result<(), String> {
    let role = flag_value(options, "--role")
        .ok_or_else(|| "issue-identity requires --role".to_string())?;
    let pki_mount = flag_value(options, "--pki-mount")
        .ok_or_else(|| "issue-identity requires --pki-mount".to_string())?;
    let pki_role = flag_value(options, "--pki-role")
        .ok_or_else(|| "issue-identity requires --pki-role".to_string())?;
    let uri_san = flag_value(options, "--uri-san")
        .ok_or_else(|| "issue-identity requires --uri-san".to_string())?;
    let request_url = required_env("ACTIONS_ID_TOKEN_REQUEST_URL")?;
    let request_token = required_env("ACTIONS_ID_TOKEN_REQUEST_TOKEN")?;
    let bao_addr = required_env(app::OPENBAO_ADDR_ENV)?;
    let bao_ca = required_env(app::OPENBAO_CA_ENV)?;
    let cache_server_ca = required_env(app::CACHE_SERVER_CA_ENV)?;
    let client_pem = PathBuf::from(required_env(app::CLIENT_CERT_ENV)?);
    let ca_pem = PathBuf::from(required_env(app::TLS_CA_CERTS_ENV)?);
    if !bao_addr.starts_with("https://") || !uri_san.starts_with("spiffe://oyatie.dev/ci/") {
        return Err(
            "identity issuance requires HTTPS OpenBao and an Oyatie CI SPIFFE URI".to_string(),
        );
    }
    let allowed = [
        (
            "github-cas-writer-dev-push",
            "pki_cas_writer",
            "cas-writer",
            "spiffe://oyatie.dev/ci/cas-writer",
        ),
        (
            "github-cas-reader-integrity-canary",
            "pki_cas_reader",
            "cas-reader",
            "spiffe://oyatie.dev/ci/cas-reader",
        ),
    ];
    if !allowed.contains(&(
        role.as_str(),
        pki_mount.as_str(),
        pki_role.as_str(),
        uri_san.as_str(),
    )) {
        return Err(
            "identity role, PKI mount, PKI role, and URI SAN do not match a trusted tuple"
                .to_string(),
        );
    }

    let ca = reqwest::Certificate::from_pem(
        &fs::read(&bao_ca).map_err(|error| format!("read {bao_ca}: {error}"))?,
    )
    .map_err(|error| format!("parse OpenBao public CA: {error}"))?;
    let client = Client::builder()
        .add_root_certificate(ca)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("build HTTPS client: {error}"))?;
    let oidc_url = format!(
        "{request_url}{}audience=oya-openbao",
        if request_url.contains('?') { "&" } else { "?" }
    );
    let mut oidc_authorization = HeaderValue::from_str(&format!("Bearer {request_token}"))
        .map_err(|error| format!("invalid GitHub OIDC authorization header: {error}"))?;
    oidc_authorization.set_sensitive(true);
    let oidc: Value = client
        .get(oidc_url)
        .header(AUTHORIZATION, oidc_authorization)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(reqwest::blocking::Response::json)
        .map_err(|error| format!("request GitHub OIDC token: {error}"))?;
    let jwt = oidc["value"]
        .as_str()
        .ok_or_else(|| "GitHub OIDC response missing value".to_string())?;
    let login: Value = client
        .post(format!(
            "{}/v1/auth/jwt/login",
            bao_addr.trim_end_matches('/')
        ))
        .json(&serde_json::json!({"role": role, "jwt": jwt}))
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(reqwest::blocking::Response::json)
        .map_err(|error| format!("OpenBao JWT login: {error}"))?;
    let token = login["auth"]["client_token"]
        .as_str()
        .ok_or_else(|| "OpenBao login response missing auth.client_token".to_string())?;
    let mut token_header = HeaderValue::from_str(token)
        .map_err(|error| format!("invalid OpenBao token header: {error}"))?;
    token_header.set_sensitive(true);
    let leaf: Value = client
        .post(format!(
            "{}/v1/{pki_mount}/issue/{pki_role}",
            bao_addr.trim_end_matches('/')
        ))
        .header("X-Vault-Token", token_header)
        .json(&serde_json::json!({"uri_sans": uri_san, "ttl": "3h"}))
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .and_then(reqwest::blocking::Response::json)
        .map_err(|error| format!("OpenBao PKI issue: {error}"))?;
    let certificate = leaf["data"]["certificate"]
        .as_str()
        .ok_or_else(|| "OpenBao PKI response missing certificate".to_string())?;
    let private_key = leaf["data"]["private_key"]
        .as_str()
        .ok_or_else(|| "OpenBao PKI response missing private_key".to_string())?;
    app::write_private_file(&client_pem, &format!("{certificate}\n{private_key}\n"))?;
    app::write_private_file(
        &ca_pem,
        &fs::read_to_string(&cache_server_ca)
            .map_err(|error| format!("read {cache_server_ca}: {error}"))?,
    )?;
    Ok(())
}

fn fixed_identity_options(mode: &str) -> Result<Vec<String>, String> {
    let values = match mode {
        "reader" => [
            "github-cas-reader-integrity-canary",
            "pki_cas_reader",
            "cas-reader",
            "spiffe://oyatie.dev/ci/cas-reader",
        ],
        "writer" => [
            "github-cas-writer-dev-push",
            "pki_cas_writer",
            "cas-writer",
            "spiffe://oyatie.dev/ci/cas-writer",
        ],
        other => {
            return Err(format!(
                "--workflow-mode must be `reader` or `writer`, got `{other}`"
            ));
        }
    };
    Ok([
        "--role",
        values[0],
        "--pki-mount",
        values[1],
        "--pki-role",
        values[2],
        "--uri-san",
        values[3],
    ]
    .into_iter()
    .map(str::to_string)
    .collect())
}

fn remove_identity_files() -> Result<(), String> {
    let mut failures = Vec::new();
    for name in [app::CLIENT_CERT_ENV, app::TLS_CA_CERTS_ENV] {
        let path = PathBuf::from(required_env(name)?);
        if let Err(error) = fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            failures.push(format!("remove {}: {error}", path.display()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let Some(command) = args.first().cloned() else {
        return Err(
            "missing subcommand (resolve | run | issue-identity | license-state | report | \
                    assert-warm | assert-cold | hash-outputs | writer-receipt | canary-verdict | canary-targets)"
                .to_string(),
        );
    };
    let rest = &args[1..];

    match command.as_str() {
        "issue-identity" => {
            issue_identity(rest)?;
            Ok(ExitCode::SUCCESS)
        }
        "resolve" => {
            let build_class = flag_value(rest, "--build-class")
                .ok_or_else(|| "resolve requires --build-class".to_string())?;
            let root = repo_root()?;
            let policy = app::load_policy(&root)?;
            let license = app::load_license(&root)?;
            let resolution = app::resolve(&policy, &license, &build_class)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&resolution.to_json())
                    .map_err(|e| format!("serialize resolution: {e}"))?
            );
            if has_flag(rest, "--require-bypass") && resolution.mode != app::CacheMode::Bypass {
                return Err(format!(
                    "--require-bypass: class `{build_class}` resolved `{}` — refusing",
                    resolution.mode
                ));
            }
            Ok(ExitCode::SUCCESS)
        }
        "run" => {
            let separator = rest
                .iter()
                .position(|argument| argument == "--")
                .ok_or_else(|| "run requires `-- COMMAND [ARG...]`".to_string())?;
            let options = &rest[..separator];
            let child = &rest[separator + 1..];
            let root = repo_root()?;
            let policy = app::load_policy(&root)?;
            let license = app::load_license(&root)?;
            let workflow_mode = flag_value(options, "--workflow-mode");
            let workflow_identity = workflow_mode
                .as_deref()
                .map(fixed_identity_options)
                .transpose()?;
            let resolution = if workflow_mode.as_deref() == Some("writer")
                || has_flag(options, "--prelicense-seed")
            {
                let trusted_seed = std::env::var("GITHUB_EVENT_NAME").as_deref() == Ok("push")
                    && std::env::var("GITHUB_REF").as_deref() == Ok("refs/heads/dev")
                    && std::env::var("OYA_CAS_IDENTITY_PROOF_ENABLED").as_deref() == Ok("true");
                if !trusted_seed {
                    return Err(
                        "pre-license seed requires the explicitly enabled trusted dev-push job"
                            .to_string(),
                    );
                }
                app::Resolution {
                    build_class: "postmerge-dev-trunk-prelicense-seed".to_string(),
                    mode: app::CacheMode::WarmReadWrite,
                    reasons: vec![
                        "explicit non-authoritative trusted seed for first integrity proof"
                            .to_string(),
                    ],
                }
            } else if workflow_mode.as_deref() == Some("reader")
                || has_flag(options, "--warm-probe")
            {
                let licensed = license
                    .get("warm_reads_licensed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let prelicense_requested = bool_flag_value(options, "--prelicense-probe")?;
                let prelicense = prelicense_requested
                    && std::env::var("GITHUB_EVENT_NAME").as_deref() == Ok("workflow_dispatch")
                    && std::env::var("GITHUB_REF").as_deref() == Ok("refs/heads/dev");
                if !licensed && !prelicense {
                    return Err("warm probe requires warm_reads_licensed=true".to_string());
                }
                app::Resolution {
                    build_class: format!(
                        "{}-warm-probe",
                        app::canary_class(&policy).ok_or_else(|| {
                            "policy missing trust_invariant.canary_build_class".to_string()
                        })?
                    ),
                    mode: app::CacheMode::WarmReadOnly,
                    reasons: vec![if licensed {
                        "licensed integrity-canary retrieval probe".to_string()
                    } else {
                        "explicit dev workflow_dispatch pre-license proof".to_string()
                    }],
                }
            } else {
                let build_class = flag_value(options, "--build-class")
                    .ok_or_else(|| "run requires --build-class or --warm-probe".to_string())?;
                app::resolve(&policy, &license, &build_class)?
            };
            if let Some(path) = flag_value(options, "--mode-out") {
                fs::write(&path, format!("{}\n", resolution.mode))
                    .map_err(|error| format!("write {path}: {error}"))?;
            }
            let Some(identity) = workflow_identity else {
                return controlled_child(&root, &resolution, child);
            };
            if let Err(error) = issue_identity(&identity) {
                let cleanup = remove_identity_files();
                return Err(match cleanup {
                    Ok(()) => error,
                    Err(cleanup) => format!("{error}; identity cleanup also failed: {cleanup}"),
                });
            }
            let mode = workflow_mode
                .as_deref()
                .ok_or_else(|| "workflow identity requires reader or writer mode".to_string())?;
            if let Err(error) = prove_identity_boundary(mode) {
                let cleanup = remove_identity_files();
                return Err(match cleanup {
                    Ok(()) => error,
                    Err(cleanup) => format!("{error}; identity cleanup also failed: {cleanup}"),
                });
            }
            let child_result = controlled_child(&root, &resolution, child).and_then(|code| {
                if code == ExitCode::SUCCESS {
                    validate_workflow_record(mode, child)?;
                }
                Ok(code)
            });
            let cleanup = remove_identity_files();
            match (child_result, cleanup) {
                (Ok(code), Ok(())) => Ok(code),
                (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
                (Err(error), Err(cleanup)) => {
                    Err(format!("{error}; identity cleanup also failed: {cleanup}"))
                }
            }
        }
        "license-state" => {
            let root = repo_root()?;
            let license = app::load_license(&root)?;
            let licensed = license
                .get("warm_reads_licensed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            println!("warm_licensed={licensed}");
            Ok(ExitCode::SUCCESS)
        }
        "report" => {
            let record_path = flag_value(rest, "--record")
                .ok_or_else(|| "report requires --record".to_string())?;
            let build_class = flag_value(rest, "--build-class")
                .ok_or_else(|| "report requires --build-class".to_string())?;
            let mode = flag_value(rest, "--mode").unwrap_or_else(|| "bypass".to_string());
            let doc = read_json(&record_path)?;
            let record = app::invocation_record(&doc)?;
            let report = app::cache_hit_report(record, &build_class, &mode);
            let payload = serde_json::to_string_pretty(&report)
                .map_err(|e| format!("serialize report: {e}"))?;
            write_out(flag_value(rest, "--out"), &payload)?;
            Ok(ExitCode::SUCCESS)
        }
        "assert-warm" => {
            let record_path = flag_value(rest, "--record")
                .ok_or_else(|| "assert-warm requires --record".to_string())?;
            let build_class = flag_value(rest, "--build-class")
                .ok_or_else(|| "assert-warm requires --build-class".to_string())?;
            let mode = flag_value(rest, "--mode")
                .ok_or_else(|| "assert-warm requires --mode".to_string())?;
            let doc = read_json(&record_path)?;
            let record = app::invocation_record(&doc)?;
            match app::assert_warm_cache_participation(record, &build_class, &mode) {
                Ok(()) => {
                    println!("warm-cache guard OK for class {build_class} in mode {mode}");
                    Ok(ExitCode::SUCCESS)
                }
                Err(findings) => {
                    for finding in findings {
                        eprintln!("{finding}");
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        "assert-cold" => {
            let record_path = flag_value(rest, "--record")
                .ok_or_else(|| "assert-cold requires --record".to_string())?;
            let doc = read_json(&record_path)?;
            let record = app::invocation_record(&doc)?;
            match app::assert_cold(record) {
                Ok(()) => {
                    println!("cold-proof OK: zero cache participation in {record_path}");
                    Ok(ExitCode::SUCCESS)
                }
                Err(findings) => {
                    for finding in findings {
                        eprintln!("{finding}");
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        "hash-outputs" => {
            let show_output = flag_value(rest, "--show-output")
                .ok_or_else(|| "hash-outputs requires --show-output".to_string())?;
            let text =
                fs::read_to_string(&show_output).map_err(|e| format!("read {show_output}: {e}"))?;
            let entries = app::digest_manifest_from_show_output(&text)?;
            let payload = serde_json::to_string_pretty(&app::manifest_to_json(&entries))
                .map_err(|e| format!("serialize manifest: {e}"))?;
            write_out(flag_value(rest, "--out"), &payload)?;
            Ok(ExitCode::SUCCESS)
        }
        "writer-receipt" => {
            let record_path = flag_value(rest, "--record")
                .ok_or_else(|| "writer-receipt requires --record".to_string())?;
            let manifest_path = flag_value(rest, "--manifest")
                .ok_or_else(|| "writer-receipt requires --manifest".to_string())?;
            let outputs_path = flag_value(rest, "--outputs")
                .ok_or_else(|| "writer-receipt requires --outputs".to_string())?;
            let document = read_json(&record_path)?;
            let record = app::invocation_record(&document)?;
            let manifest = read_json(&manifest_path)?;
            let outputs = fs::read_to_string(&outputs_path)
                .map_err(|error| format!("read {outputs_path}: {error}"))?;
            let receipt =
                writer_receipt_from(record, &manifest, &outputs, |name| std::env::var(name).ok())?;
            let payload = serde_json::to_string_pretty(&receipt)
                .map_err(|error| format!("serialize writer receipt: {error}"))?;
            write_out(flag_value(rest, "--out"), &payload)?;
            Ok(ExitCode::SUCCESS)
        }
        "canary-verdict" => {
            let cold_path = flag_value(rest, "--cold")
                .ok_or_else(|| "canary-verdict requires --cold".to_string())?;
            let cold = app::manifest_from_json(&read_json(&cold_path)?)?;
            let warm = match flag_value(rest, "--warm") {
                Some(path) => Some(app::manifest_from_json(&read_json(&path)?)?),
                None => None,
            };
            // FAIL-CLOSED coupling to the kill-switch: while warm reads are
            // LICENSED, a verdict without a warm manifest is a misconfigured
            // canary (e.g. the probe step was dropped), not an INACTIVE state —
            // emitting INACTIVE (exit 0) there would let warm reads continue
            // without their trust anchor.
            if warm.is_none() {
                let root = repo_root()?;
                let license = app::load_license(&root)?;
                if license
                    .get("warm_reads_licensed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    return Err(
                        "canary-verdict: warm reads are LICENSED but no --warm manifest was \
                         supplied — the warm-probe step is missing or failed silently; refusing \
                         to emit INACTIVE while warmth is live (ADR-0556 D2)"
                            .to_string(),
                    );
                }
            }
            // FAIL-CLOSED on a NON-PARTICIPATING probe. A warm manifest proves nothing
            // about the cache unless the probe that produced it actually fetched from
            // the cache: canary_verdict compares target->OUTPUT-DIGEST pairs, and a probe
            // that served zero blobs and rebuilt everything locally yields digests
            // byte-identical to the cold build, full label overlap, zero divergence — and
            // therefore GREEN. That GREEN is the verdict that licenses warm reads
            // FLEET-WIDE, so it must never be reachable without proven participation.
            //
            // assert_warm_cache_participation is the existing, unit-tested predicate for
            // exactly this (cache_hit_rate != 0, run_action_cache_count != 0,
            // last_snapshot.re_action_cache_started != 0). It was written and never wired.
            if warm.is_some() {
                let record_path = flag_value(rest, "--warm-record").ok_or_else(|| {
                    "canary-verdict: a --warm manifest requires --warm-record (the probe's \
                     buck2 invocation record). Without it the probe's cache participation is \
                     unproven, and a zero-fetch local rebuild would emit GREEN and license \
                     warm reads fleet-wide (ADR-0556 D2)"
                        .to_string()
                })?;
                let doc = read_json(&record_path)?;
                let record = app::invocation_record(&doc)?;
                if let Err(findings) = app::assert_complete_warm_cache_coverage(record) {
                    return Err(format!(
                        "canary-verdict: the warm probe was NOT fully served by the cache, so its \
                         manifest includes local/remote rebuilds rather than complete evidence — refusing to compare \
                         it (ADR-0556 D2). Findings: {}",
                        findings.join("; ")
                    ));
                }
            }
            let writer_flags = [
                flag_value(rest, "--writer-receipt"),
                flag_value(rest, "--writer-manifest"),
                flag_value(rest, "--writer-run-id"),
            ];
            let writer_provenance = if writer_flags.iter().any(Option::is_some) {
                let [Some(receipt_path), Some(manifest_path), Some(run_id)] = writer_flags else {
                    return Err(
                        "canary-verdict requires --writer-receipt, --writer-manifest, and --writer-run-id together"
                            .to_string(),
                    );
                };
                let warm = warm.as_ref().ok_or_else(|| {
                    "writer commissioning proof requires a reader warm manifest".to_string()
                })?;
                Some(validate_writer_receipt_from(
                    &read_json(&receipt_path)?,
                    &read_json(&manifest_path)?,
                    &cold,
                    warm,
                    &run_id,
                    |name| std::env::var(name).ok(),
                )?)
            } else {
                None
            };
            let (status, mut verdict) = app::canary_verdict(&cold, warm.as_ref());
            bind_github_green_provenance(status, &mut verdict)?;
            if status == app::CanaryStatus::Green
                && let Some(provenance) = writer_provenance
            {
                verdict["writer_provenance"] = provenance;
            }
            let payload = serde_json::to_string_pretty(&verdict)
                .map_err(|e| format!("serialize verdict: {e}"))?;
            write_out(flag_value(rest, "--out"), &payload)?;
            eprintln!("canary verdict: {}", status.as_str());
            if status.is_failure() {
                Ok(ExitCode::FAILURE)
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
        "canary-targets" => {
            let policy = app::canary_policy()?;
            let targets = policy
                .get("pinned_targets")
                .and_then(Value::as_array)
                .ok_or_else(|| "canary policy missing pinned_targets".to_string())?;
            for target in targets {
                if let Some(t) = target.as_str() {
                    println!("{t}");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown subcommand `{other}`")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(code) => code,
        Err(message) => fail(&message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Debug)]
    struct RejectEveryClientCertificate;

    impl rustls::server::danger::ClientCertVerifier for RejectEveryClientCertificate {
        fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] {
            &[]
        }

        fn verify_client_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer<'_>,
            _intermediates: &[rustls::pki_types::CertificateDer<'_>],
            _now: rustls::pki_types::UnixTime,
        ) -> Result<rustls::server::danger::ClientCertVerified, rustls::Error> {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ))
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Err(rustls::Error::General(
                "unreachable TLS 1.2 verifier".to_string(),
            ))
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer<'_>,
            _dss: &rustls::DigitallySignedStruct,
        ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
            Err(rustls::Error::General(
                "unreachable after certificate rejection".to_string(),
            ))
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            vec![rustls::SignatureScheme::ECDSA_NISTP256_SHA256]
        }
    }

    #[test]
    fn workflow_boolean_flags_are_explicit_and_fail_closed() {
        let true_value = vec!["--prelicense-probe".into(), "true".into()];
        let false_value = vec!["--prelicense-probe".into(), "false".into()];
        let flag_shaped_value = vec!["--prelicense-probe".into(), "--mode-out".into()];
        let missing_value = vec!["--prelicense-probe".into()];
        let invalid_value = vec!["--prelicense-probe".into(), "yes".into()];

        assert!(bool_flag_value(&true_value, "--prelicense-probe").unwrap());
        assert!(!bool_flag_value(&false_value, "--prelicense-probe").unwrap());
        assert!(bool_flag_value(&flag_shaped_value, "--prelicense-probe").is_err());
        assert!(bool_flag_value(&missing_value, "--prelicense-probe").is_err());
        assert!(!bool_flag_value(&[], "--prelicense-probe").unwrap());
        assert!(bool_flag_value(&invalid_value, "--prelicense-probe").is_err());
    }

    #[test]
    fn workflow_modes_select_only_the_fixed_reader_and_writer_identities() {
        let reader = fixed_identity_options("reader").unwrap();
        let writer = fixed_identity_options("writer").unwrap();

        assert_eq!(reader[1], "github-cas-reader-integrity-canary");
        assert_eq!(reader[3], "pki_cas_reader");
        assert_eq!(reader[5], "cas-reader");
        assert_eq!(reader[7], "spiffe://oyatie.dev/ci/cas-reader");
        assert_eq!(writer[1], "github-cas-writer-dev-push");
        assert_eq!(writer[3], "pki_cas_writer");
        assert_eq!(writer[5], "cas-writer");
        assert_eq!(writer[7], "spiffe://oyatie.dev/ci/cas-writer");
        assert!(fixed_identity_options("other").is_err());
    }

    #[test]
    fn reader_observes_real_server_client_auth_rejection() {
        use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
        use std::net::TcpListener;

        let rcgen::CertifiedKey {
            cert: server_cert,
            signing_key: server_key,
        } = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        let rcgen::CertifiedKey {
            cert: client_cert,
            signing_key: client_key,
        } = rcgen::generate_simple_self_signed(vec!["client".to_string()]).unwrap();

        let mut server_config =
            rustls::ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
                .with_client_cert_verifier(Arc::new(RejectEveryClientCertificate))
                .with_single_cert(
                    vec![server_cert.der().clone()],
                    PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(server_key.serialize_der())),
                )
                .unwrap();
        server_config.alpn_protocols = vec![b"h2".to_vec()];

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            socket
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut connection = rustls::ServerConnection::new(Arc::new(server_config)).unwrap();
            connection.complete_io(&mut socket).unwrap_err()
        });

        let mut roots = rustls::RootCertStore::empty();
        roots.add(server_cert.der().clone()).unwrap();
        let mut client_config =
            rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
                .with_root_certificates(roots)
                .with_client_auth_cert(
                    vec![client_cert.der().clone()],
                    PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(client_key.serialize_der())),
                )
                .unwrap();
        client_config.alpn_protocols = vec![b"h2".to_vec()];
        let mut socket = TcpStream::connect(address).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut connection = rustls::ClientConnection::new(
            Arc::new(client_config),
            ServerName::try_from("localhost").unwrap(),
        )
        .unwrap();

        let alert = read_client_auth_rejection(&mut connection, &mut socket).unwrap();
        assert!(matches!(
            alert.as_str(),
            "access_denied"
                | "bad_certificate"
                | "certificate_required"
                | "certificate_unknown"
                | "unknown_ca"
        ));
        server.join().unwrap();
    }

    #[test]
    fn reader_writer_denial_accepts_only_named_peer_tls_alerts() {
        let alert = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rustls::Error::AlertReceived(rustls::AlertDescription::AccessDenied),
        );
        assert_eq!(typed_client_auth_alert(&alert).unwrap(), "access_denied");
        let generic_eof = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "generic EOF");
        let server_ca = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer),
        );
        let protocol_alert = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            rustls::Error::AlertReceived(rustls::AlertDescription::ProtocolVersion),
        );
        assert!(typed_client_auth_alert(&generic_eof).is_err());
        assert!(typed_client_auth_alert(&server_ca).is_err());
        assert!(typed_client_auth_alert(&protocol_alert).is_err());
    }

    #[test]
    fn capabilities_proof_requires_exact_grpc_framing_status_and_http_metadata() {
        assert_eq!(grpc_frame(&[0, 0, 0, 0, 2, 8, 1]).unwrap(), &[8, 1]);
        assert!(grpc_frame(&[0, 0, 0, 0, 2, 8]).is_err());
        assert!(grpc_frame(&[1, 0, 0, 0, 0]).is_err());
        assert_eq!(grpc_status("HTTP/2 200\r\ngrpc-status: 0\r\n").unwrap(), 0);
        assert!(grpc_status("HTTP/2 200\r\n").is_err());
        assert_eq!(curl_metadata(b"2\n200").unwrap(), ("2", "200"));
        assert!(curl_metadata(b"1.1\n200\nextra").is_err());

        let request = GetCapabilitiesRequest {
            instance_name: "main".to_string(),
        }
        .encode_to_vec();
        assert_eq!(request, [0x0a, 0x04, b'm', b'a', b'i', b'n']);
        assert_eq!(
            grpc_message(&request).unwrap(),
            [0, 0, 0, 0, 6, 0x0a, 4, b'm', b'a', b'i', b'n']
        );

        let cache_only = ServerCapabilities {
            cache_capabilities: Some(CacheCapabilities {
                digest_functions: vec![1],
            }),
            execution_capabilities: None,
            low_api_version: Some(SemVer {
                major: 2,
                minor: 0,
                patch: 0,
                prerelease: String::new(),
            }),
            high_api_version: Some(SemVer {
                major: 2,
                minor: 3,
                patch: 0,
                prerelease: String::new(),
            }),
        };
        assert!(validate_server_capabilities(&cache_only.encode_to_vec()).is_ok());
        let mut execution_endpoint = cache_only;
        execution_endpoint.execution_capabilities = Some(ExecutionCapabilities {});
        assert!(validate_server_capabilities(&execution_endpoint.encode_to_vec()).is_err());
    }

    #[test]
    fn green_github_verdict_requires_and_binds_run_provenance() {
        let lookup = |name: &str| match name {
            "GITHUB_ACTIONS" => Some("true".to_string()),
            "GITHUB_SHA" => Some("abc123".to_string()),
            "GITHUB_RUN_ID" => Some("42".to_string()),
            "GITHUB_JOB" => Some("cache-integrity".to_string()),
            "GITHUB_RUN_ATTEMPT" => Some("2".to_string()),
            _ => None,
        };
        let provenance = github_green_provenance_from(app::CanaryStatus::Green, lookup)
            .unwrap()
            .unwrap();
        assert_eq!(provenance["github_sha"], "abc123");
        assert_eq!(provenance["github_run_attempt"], "2");
        assert!(
            github_green_provenance_from(app::CanaryStatus::Green, |name| {
                (name == "GITHUB_ACTIONS").then(|| "true".to_string())
            })
            .is_err()
        );
    }

    fn writer_record_fixture() -> Value {
        json!({
            "cache_hit_rate": 0.0,
            "run_action_cache_count": 0,
            "run_local_count": 2,
            "run_remote_count": 0,
            "cache_upload_attempt_count": 2,
            "cache_upload_count": 2,
            "re_upload_bytes": 1024,
            "re_download_bytes": 0,
            "exit_result_name": "SUCCESS",
            "run_command_failure_count": 0,
            "errors": [],
            "daemon_connection_failure": false,
            "last_snapshot": {
                "re_action_cache_started": 0,
                "re_action_cache_finished_successfully": 0,
                "re_action_cache_finished_with_error": 0,
                "re_upload_bytes": 1024,
                "re_uploads_started": 2,
                "re_uploads_finished_successfully": 2,
                "re_uploads_finished_with_error": 0,
                "re_downloads_finished_with_error": 0,
                "re_executes_started": 0,
                "re_executes_finished_successfully": 0,
                "re_executes_finished_with_error": 0,
                "re_write_action_results_started": 2,
                "re_write_action_results_finished_successfully": 2,
                "re_write_action_results_finished_with_error": 0
            }
        })
    }

    fn proof_env(name: &str) -> Option<String> {
        match name {
            "GITHUB_ACTIONS" => Some("true".into()),
            "GITHUB_REPOSITORY" => Some("jason931225/oyatie".into()),
            "GITHUB_SHA" => Some("abc123".into()),
            "GITHUB_RUN_ID" => Some("42".into()),
            "GITHUB_JOB" => Some("cache-writer-identity".into()),
            "GITHUB_RUN_ATTEMPT" => Some("1".into()),
            "GITHUB_WORKFLOW_REF" => Some(
                "jason931225/oyatie/.github/workflows/oya-ci-required.yml@refs/heads/dev".into(),
            ),
            "GITHUB_EVENT_NAME" => Some("push".into()),
            "GITHUB_REF" => Some("refs/heads/dev".into()),
            "RUNNER_NAME" => Some("writer-runner".into()),
            "OYA_RUNNER_POD_NAME" => Some("writer-pod".into()),
            "OYA_RUNNER_POD_UID" => Some("writer-uid".into()),
            "OYA_RUNNER_NODE_NAME" => Some("worker-1".into()),
            _ => None,
        }
    }

    fn reader_env(name: &str) -> Option<String> {
        match name {
            "GITHUB_SHA" => Some("abc123".into()),
            "RUNNER_NAME" => Some("reader-runner".into()),
            "OYA_RUNNER_POD_UID" => Some("reader-uid".into()),
            _ => None,
        }
    }

    #[test]
    fn writer_receipt_binds_trusted_run_distinct_pods_and_three_way_parity() {
        let root =
            std::env::temp_dir().join(format!("oya-cache-writer-receipt-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let output = root.join("output");
        fs::write(&output, b"writer-output").unwrap();
        let output_binding = format!("//canary:target {}\n", output.display());
        let manifest =
            app::manifest_to_json(&app::digest_manifest_from_show_output(&output_binding).unwrap());
        let receipt = writer_receipt_from(
            &writer_record_fixture(),
            &manifest,
            &output_binding,
            proof_env,
        )
        .unwrap();
        let mismatched_manifest = json!({
            "schema": app::DIGEST_MANIFEST_SCHEMA,
            "entries": {"//canary:target": "sha256:not-the-output"}
        });
        assert!(
            writer_receipt_from(
                &writer_record_fixture(),
                &mismatched_manifest,
                &output_binding,
                proof_env,
            )
            .is_err()
        );
        let entries = app::manifest_from_json(&manifest).unwrap();
        let provenance =
            validate_writer_receipt_from(&receipt, &manifest, &entries, &entries, "42", reader_env)
                .unwrap();
        assert_eq!(provenance["github_run_id"], "42");
        assert_eq!(provenance["runner_pod_uid"], "writer-uid");

        for (key, value) in [
            ("github_repository", json!("other/repo")),
            ("github_sha", json!("wrong")),
            ("github_run_id", json!("41")),
            ("github_event_name", json!("workflow_dispatch")),
            ("github_ref", json!("refs/heads/other")),
            ("github_workflow_ref", json!("other")),
            ("runner_name", json!("reader-runner")),
            ("runner_pod_uid", json!("reader-uid")),
        ] {
            let mut invalid = receipt.clone();
            invalid[key] = value;
            assert!(
                validate_writer_receipt_from(
                    &invalid, &manifest, &entries, &entries, "42", reader_env,
                )
                .is_err(),
                "accepted invalid writer receipt field {key}"
            );
        }
        let other_manifest = json!({
            "schema": app::DIGEST_MANIFEST_SCHEMA,
            "entries": {"//canary:target": "sha256:other"}
        });
        assert!(
            validate_writer_receipt_from(
                &receipt,
                &other_manifest,
                &entries,
                &entries,
                "42",
                reader_env,
            )
            .is_err()
        );
        let different = app::manifest_from_json(&other_manifest).unwrap();
        assert!(
            validate_writer_receipt_from(
                &receipt, &manifest, &different, &entries, "42", reader_env,
            )
            .is_err()
        );
        assert!(
            validate_writer_receipt_from(
                &receipt,
                &manifest,
                &entries,
                &entries,
                "not-digits",
                reader_env,
            )
            .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canary_verdict_accepts_wrapped_warm_invocation_record() {
        let root =
            std::env::temp_dir().join(format!("oya-cache-canary-command-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let cold = root.join("cold.json");
        let warm = root.join("warm.json");
        let record = root.join("record.json");
        let out = root.join("verdict.json");
        let manifest = json!({"entries": {"//canary:target": "sha256:fixture"}});
        fs::write(&cold, serde_json::to_vec(&manifest).unwrap()).unwrap();
        fs::write(&warm, serde_json::to_vec(&manifest).unwrap()).unwrap();
        fs::write(
            &record,
            serde_json::to_vec(&json!({
                "data": {"Record": {"data": {"InvocationRecord": {
                    "cache_hit_rate": 1.0,
                    "run_action_cache_count": 1,
                    "run_local_count": 0,
                    "run_remote_count": 0,
                    "run_skipped_count": 0,
                    "cache_upload_attempt_count": 0,
                    "cache_upload_count": 0,
                    "re_upload_bytes": 0,
                    "re_download_bytes": 1024,
                    "exit_result_name": "SUCCESS",
                    "run_command_failure_count": 0,
                    "errors": [],
                    "daemon_connection_failure": false,
                    "last_snapshot": {
                        "re_action_cache_started": 1,
                        "re_action_cache_finished_successfully": 1,
                        "re_action_cache_finished_with_error": 0,
                        "re_upload_bytes": 0,
                        "re_uploads_started": 0,
                        "re_uploads_finished_successfully": 0,
                        "re_uploads_finished_with_error": 0,
                        "re_download_bytes": 1024,
                        "re_downloads_started": 1,
                        "re_downloads_finished_successfully": 1,
                        "re_downloads_finished_with_error": 0,
                        "re_executes_started": 0,
                        "re_executes_finished_successfully": 0,
                        "re_executes_finished_with_error": 0,
                        "re_write_action_results_started": 0,
                        "re_write_action_results_finished_successfully": 0,
                        "re_write_action_results_finished_with_error": 0
                    }
                }}}}
            }))
            .unwrap(),
        )
        .unwrap();

        let result = run(vec![
            "canary-verdict".into(),
            "--cold".into(),
            cold.display().to_string(),
            "--warm".into(),
            warm.display().to_string(),
            "--warm-record".into(),
            record.display().to_string(),
            "--out".into(),
            out.display().to_string(),
        ]);

        assert_eq!(result.unwrap(), ExitCode::SUCCESS);
        assert_eq!(read_json(out.to_str().unwrap()).unwrap()["status"], "GREEN");
        fs::remove_dir_all(root).unwrap();
    }
}
