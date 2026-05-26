//! Webhook-receiver app — ADR-0112 wave-A binary entrypoint.
//!
//! Two modes:
//!
//! Mode 1: `--simulate-delivery <path>`. Runs a synthetic delivery JSON
//! through the full kernel pipeline (HMAC + dedup + router) and prints
//! the outcome. Canonical integration-test surface per ADR-0112 §"Open
//! questions" #3. Exits 0 on a routed or deduplicated decision, non-zero
//! on load failure or HMAC rejection.
//!
//! Mode 2 (default): binds an `axum` HTTP server at `--bind` and serves
//! `POST /webhook/github`. Each delivery is verified, dedup'd, routed,
//! and (when implemented) posted back via `gh api`.
//!
//! Secret discovery (per ADR-0112 §"Signature handling"):
//! - Production wiring SHOULD read from OpenBao at
//!   `sref://openbao/oya/foundry/github-webhook-secret`.
//! - TODO: replace the file-backed fallback with the OpenBao adapter
//!   once `oya-secrets-domain` exposes a SecretReference resolver.
//! - File-backed fallback: `~/.openbao/oya/foundry/github-webhook-secret`.
//! - `--secret-path <p>` overrides the fallback for tests.
//! - `--skip-hmac` skips the HMAC gate entirely (local dev only).

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use clap::Parser;
use oya_vcs_webhook_receiver_app::{
    Dispatch, DispatchLoadError, DispatchOutcome, DispatchPaths, process_simulated_delivery,
};
use oya_vcs_webhook_receiver_kernel::{
    DedupLookup, DedupOutcome, HmacVerificationError, find_dedup_status, route_event,
    verify_hmac_sha256,
};

/// CLI surface for `oya-vcs-webhook-receiver-app`.
#[derive(Debug, Parser)]
#[command(
    name = "oya-vcs-webhook-receiver-app",
    about = "GitHub webhook receiver substrate (ADR-0112 wave-A)."
)]
struct Cli {
    /// Bind address for the HTTP server (default `127.0.0.1:8765`).
    /// Ignored when `--simulate-delivery` is set.
    #[arg(long, default_value = "127.0.0.1:8765")]
    bind: String,

    /// Path to the canonical event-router table.
    #[arg(long, default_value = "registry/vcs/event-router.yaml")]
    router: PathBuf,

    /// Path to the append-only delivery log.
    #[arg(long, default_value = "registry/vcs/webhook-delivery-log.json")]
    delivery_log: PathBuf,

    /// Path to the HMAC secret (file-backed OpenBao fallback). If
    /// unset, the binary tries `--secret-ref`, then
    /// `~/.openbao/oya/foundry/github-webhook-secret`.
    #[arg(long)]
    secret_path: Option<PathBuf>,

    /// SecretReference URI for the HMAC secret per ADR-0112 §"Signature
    /// handling". Currently only `sref://openbao/<path>` is recognised
    /// and is **mapped to the file-backed fallback** at
    /// `$HOME/.openbao/<path>` until the OpenBao SecretReference
    /// resolver in `oya-secrets-domain` is wired up. The CLI flag is
    /// shipped now so deployment manifests can declare the canonical
    /// URI today; the resolver swap is a one-line change inside
    /// `resolve_secret_ref`.
    #[arg(long)]
    secret_ref: Option<String>,

    /// Skip HMAC verification entirely (local-dev only — never set in
    /// production).
    #[arg(long, default_value_t = false)]
    skip_hmac: bool,

    /// Synthetic delivery JSON to process instead of binding HTTP. The
    /// file follows the `SimulatedDelivery` shape — see the lib crate
    /// docs for the schema.
    #[arg(long)]
    simulate_delivery: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let secret_path: Option<PathBuf> = cli
        .secret_path
        .clone()
        .or_else(|| cli.secret_ref.as_deref().and_then(resolve_secret_ref))
        .or_else(default_secret_path);

    let paths = DispatchPaths {
        router_yaml: &cli.router,
        delivery_log_json: &cli.delivery_log,
        secret_path: secret_path.as_deref(),
    };
    let dispatch = match Dispatch::from_paths_or_optional(&paths) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("oya-vcs-webhook-receiver-app: failed to load dispatch state: {e}");
            return ExitCode::from(2);
        }
    };

    if let Some(path) = cli.simulate_delivery.as_deref() {
        return run_simulate(&dispatch, path, cli.skip_hmac);
    }

    let bind: SocketAddr = match cli.bind.parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "oya-vcs-webhook-receiver-app: --bind {:?} is not a valid socket address: {e}",
                cli.bind
            );
            return ExitCode::from(2);
        }
    };
    run_server(dispatch, bind, cli.skip_hmac)
}

fn default_secret_path() -> Option<PathBuf> {
    // File-backed OpenBao fallback (TODO: replace with SecretReference
    // resolver per ADR-0112 §"Signature handling").
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".openbao")
            .join("oya")
            .join("foundry")
            .join("github-webhook-secret"),
    )
}

/// Resolve a `sref://openbao/<path>` URI to a filesystem path.
///
/// Today this is a structural alias: the URI authority must be
/// `openbao` and the path is rooted at `$HOME/.openbao/<path>`. When
/// the real OpenBao resolver lands in `oya-secrets-domain` this
/// function will swap to call it instead — the CLI flag and deployment
/// manifests stay unchanged.
///
/// Returns `None` for malformed URIs (unknown scheme, missing
/// authority, or `$HOME` not set). Callers fall through to the file
/// fallback in that case.
fn resolve_secret_ref(uri: &str) -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    resolve_secret_ref_with_home(uri, Path::new(&home))
}

/// Test-friendly inner that takes an explicit `home` so tests don't
/// have to mutate process env (which collides with `forbid(unsafe)`).
fn resolve_secret_ref_with_home(uri: &str, home: &Path) -> Option<PathBuf> {
    let body = uri.strip_prefix("sref://")?;
    let (authority, rest) = body.split_once('/')?;
    if authority != "openbao" {
        return None;
    }
    if rest.is_empty() {
        return None;
    }
    let mut path = home.join(".openbao");
    for segment in rest.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return None;
        }
        path = path.join(segment);
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_HOME: &str = "/home/test";

    #[test]
    fn sref_openbao_maps_to_dotopenbao_under_home() {
        let resolved = resolve_secret_ref_with_home(
            "sref://openbao/oya/foundry/github-webhook-secret",
            Path::new(FAKE_HOME),
        )
        .expect("URI resolves");
        assert_eq!(
            resolved,
            PathBuf::from("/home/test/.openbao/oya/foundry/github-webhook-secret")
        );
    }

    #[test]
    fn sref_unknown_authority_returns_none() {
        assert!(
            resolve_secret_ref_with_home(
                "sref://aws-secrets-manager/foo/bar",
                Path::new(FAKE_HOME)
            )
            .is_none()
        );
    }

    #[test]
    fn sref_unknown_scheme_returns_none() {
        assert!(resolve_secret_ref_with_home("file:///etc/passwd", Path::new(FAKE_HOME)).is_none());
        assert!(
            resolve_secret_ref_with_home("https://openbao/foo", Path::new(FAKE_HOME)).is_none()
        );
    }

    #[test]
    fn sref_traversal_is_rejected() {
        assert!(
            resolve_secret_ref_with_home("sref://openbao/../../etc/passwd", Path::new(FAKE_HOME))
                .is_none()
        );
    }
}

fn run_simulate(dispatch: &Dispatch, path: &Path, skip_hmac: bool) -> ExitCode {
    match process_simulated_delivery(dispatch, path, skip_hmac) {
        Ok(outcome) => {
            print_outcome(&outcome);
            match outcome {
                DispatchOutcome::HmacRejected(_) => ExitCode::from(3),
                _ => ExitCode::SUCCESS,
            }
        }
        Err(err) => {
            eprintln!("oya-vcs-webhook-receiver-app: {err}");
            ExitCode::from(2)
        }
    }
}

fn print_outcome(outcome: &DispatchOutcome) {
    match outcome {
        DispatchOutcome::Accepted { agent, purpose } => {
            println!("accepted: routed to {agent} ({purpose})");
        }
        DispatchOutcome::AcceptedAfterExpiry {
            agent,
            purpose,
            prior_at_seconds,
        } => {
            println!(
                "accepted-after-expiry: routed to {agent} ({purpose}); prior row at {prior_at_seconds}s (GC candidate)"
            );
        }
        DispatchOutcome::Deduplicated {
            prior_outcome,
            at_seconds,
        } => {
            println!(
                "deduplicated: prior delivery at {at_seconds}s with outcome `{}`",
                prior_outcome.as_wire()
            );
        }
        DispatchOutcome::RoutingFailed { event, action } => {
            println!("routing-failed: no router row matches (event={event}, action={action})");
        }
        DispatchOutcome::HmacRejected(err) => {
            println!("hmac-rejected: {}", describe_hmac_error(err));
        }
        DispatchOutcome::ConflictingPriorOutcomes => {
            println!(
                "conflicting-prior-outcomes: integrity anomaly — see oya-foundry-fitness-webhook-delivery-log-monotonic lane"
            );
        }
    }
}

fn describe_hmac_error(err: &HmacVerificationError) -> &'static str {
    match err {
        HmacVerificationError::MissingHeader => "missing X-Hub-Signature-256 header",
        HmacVerificationError::UnsupportedScheme => "header is not `sha256=…`",
        HmacVerificationError::MalformedDigest => "hex digest is malformed",
        HmacVerificationError::MalformedSecret => "secret rejected by HMAC primitive",
        HmacVerificationError::SignatureMismatch => "computed HMAC does not match header",
    }
}

// ---------------------------------------------------------------------
// HTTP server (axum)
// ---------------------------------------------------------------------

/// Shared state handed to every request handler. Cloning is cheap
/// because the inner `Dispatch` lives inside an `Arc`.
#[derive(Clone)]
struct ServerState {
    dispatch: Arc<Dispatch>,
    skip_hmac: bool,
}

fn run_server(dispatch: Dispatch, bind: SocketAddr, skip_hmac: bool) -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("oya-vcs-webhook-receiver-app: failed to start tokio runtime: {e}");
            return ExitCode::from(2);
        }
    };
    let state = ServerState {
        dispatch: Arc::new(dispatch),
        skip_hmac,
    };
    let app = Router::new()
        .route("/webhook/github", post(handle_github_webhook))
        .with_state(state);
    runtime.block_on(async move {
        let listener = match tokio::net::TcpListener::bind(bind).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("oya-vcs-webhook-receiver-app: failed to bind {bind}: {e}");
                return ExitCode::from(2);
            }
        };
        eprintln!("oya-vcs-webhook-receiver-app: listening on http://{bind}/webhook/github");
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("oya-vcs-webhook-receiver-app: server error: {e}");
            return ExitCode::from(2);
        }
        ExitCode::SUCCESS
    })
}

async fn handle_github_webhook(
    State(state): State<ServerState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let delivery_id = headers
        .get("X-GitHub-Delivery")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let event = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // 1. HMAC fail-closed gate.
    if !state.skip_hmac
        && let Some(secret) = state.dispatch.secret.as_deref()
        && let Err(err) = verify_hmac_sha256(&body, signature, secret)
    {
        return (
            StatusCode::UNAUTHORIZED,
            format!("hmac-rejected: {}\n", describe_hmac_error(&err)),
        );
    }

    // 2. Dedup.
    let now_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    match find_dedup_status(&state.dispatch.log, &delivery_id, now_seconds) {
        DedupLookup::FirstDelivery | DedupLookup::Expired { .. } => {
            let action = extract_action(&body);
            let conclusion = extract_conclusion(&body);
            match route_event(&event, &action, &conclusion, &state.dispatch.router) {
                Some(row) => (
                    StatusCode::ACCEPTED,
                    format!("accepted: routed to {} ({})\n", row.agent, row.purpose),
                ),
                None => (
                    StatusCode::BAD_REQUEST,
                    format!(
                        "routing-failed: no router row matches (event={event}, action={action})\n"
                    ),
                ),
            }
        }
        DedupLookup::Deduplicated {
            outcome,
            at_seconds,
        } => (
            StatusCode::OK,
            format!(
                "deduplicated: prior delivery at {at_seconds}s with outcome `{}`\n",
                outcome_wire(outcome)
            ),
        ),
        DedupLookup::ConflictingOutcomes => (
            StatusCode::CONFLICT,
            "conflicting-prior-outcomes: integrity anomaly\n".to_string(),
        ),
    }
}

fn outcome_wire(o: DedupOutcome) -> &'static str {
    o.as_wire()
}

fn extract_action(body: &Bytes) -> String {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return String::new();
    };
    value
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// Extract `workflow_run.conclusion` or `check_suite.conclusion`
/// from a raw webhook body. Returns `""` when no conclusion is
/// present (most event types).
fn extract_conclusion(body: &Bytes) -> String {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return String::new();
    };
    for key in ["workflow_run", "check_suite"] {
        if let Some(c) = value
            .get(key)
            .and_then(|v| v.get("conclusion"))
            .and_then(|v| v.as_str())
        {
            return c.to_string();
        }
    }
    String::new()
}

// ---------------------------------------------------------------------
// Dispatch::from_paths_or_optional
// ---------------------------------------------------------------------
//
// We allow a missing secret file (the binary either runs with
// `--skip-hmac` or against a real OpenBao backend later). All other
// paths must exist.

trait DispatchExt: Sized {
    fn from_paths_or_optional(paths: &DispatchPaths<'_>) -> Result<Self, DispatchLoadError>;
}

impl DispatchExt for Dispatch {
    fn from_paths_or_optional(paths: &DispatchPaths<'_>) -> Result<Self, DispatchLoadError> {
        match Dispatch::from_paths(paths) {
            Ok(d) => Ok(d),
            Err(DispatchLoadError::SecretIo(_, _)) => {
                // Retry without the secret path — secret is optional
                // at startup; production deployments require it but
                // local dev with `--skip-hmac` doesn't.
                Dispatch::from_paths(&DispatchPaths {
                    router_yaml: paths.router_yaml,
                    delivery_log_json: paths.delivery_log_json,
                    secret_path: None,
                })
            }
            Err(other) => Err(other),
        }
    }
}
