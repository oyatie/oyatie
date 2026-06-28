//! Binary entrypoint for the CI webhook gateway.
//!
//! Wires the env-resolved config + the HMAC secret + the Jenkins-backed
//! dispatcher into the axum receiver, and serves until SIGINT/SIGTERM.
//!
//! The Jenkins "kick" is a real HTTP/1.1 POST over a tokio TCP stream (no
//! extra dependency beyond the blessed tokio); it targets the Jenkins
//! generic-webhook-trigger / build-token endpoint configured via
//! `OYA_JENKINS_DISPATCH_URL`. When that URL is unset the dispatcher returns a
//! typed transport error rather than silently "succeeding".

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::sync::Arc;

use oya_ci_webhook_gateway_app::{
    WEBHOOK_PATH,
    config::{self, DispatcherKind, GatewayConfig},
    dispatch::{ControllerDispatcher, GateRunBody, JenkinsDispatcher, PipelineKickoff},
    receiver::{ReceiverState, router},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let config = Arc::new(GatewayConfig::from_env());
    let secret = Arc::new(config::resolve_secret(|k| std::env::var(k).ok()));

    if secret.is_empty() {
        tracing::warn!(
            "{} is unset/empty: the gateway will FAIL CLOSED on every delivery \
             until the webhook secret is provisioned (see SETUP-RUNBOOK.md).",
            config::ENV_WEBHOOK_SECRET
        );
    }
    if config.dispatcher_kind == DispatcherKind::Jenkins && config.jenkins_dispatch_url.is_none() {
        tracing::warn!(
            "{} is unset: PR events will verify + route but dispatch will return \
             a typed transport error (no silent success).",
            config::ENV_JENKINS_DISPATCH_URL
        );
    }
    if config.dispatcher_kind == DispatcherKind::Controller && config.controller_url.is_none() {
        tracing::warn!(
            "{} is unset: controller dispatcher selected but URL not configured; \
             PR dispatch will return a typed transport error (no silent success).",
            config::ENV_CONTROLLER_URL
        );
    }

    let dispatcher: Arc<dyn oya_ci_webhook_gateway_app::dispatch::PipelineDispatcher> =
        match config.dispatcher_kind {
            DispatcherKind::Jenkins => {
                tracing::info!("dispatcher: jenkins");
                Arc::new(JenkinsDispatcher::new(
                    config.jenkins_dispatch_url.clone(),
                    kick_jenkins,
                ))
            }
            DispatcherKind::Controller => {
                tracing::info!("dispatcher: controller (oya-ci-controller /gate-run)");
                if config.controller_bearer.is_none() {
                    tracing::warn!(
                        "{} is unset: controller dispatcher will POST /gate-run WITHOUT a bearer; \
                         the controller fails closed (401) once it enforces keystone-1.",
                        config::ENV_CONTROLLER_BEARER
                    );
                }
                let bearer = config.controller_bearer.clone();
                Arc::new(ControllerDispatcher::new(
                    config.controller_url.clone(),
                    move |url, body| post_controller(url, body, bearer.clone()),
                ))
            }
        };

    // Resolve optional ed25519 public key from OYA_WEBHOOK_ED25519_PUBKEY
    // (base64-encoded 32-byte ed25519 verifying key). When absent, the gateway
    // requires HMAC-SHA256 via the HMAC secret path.
    let ed25519_key = config::resolve_ed25519_key(|k| std::env::var(k).ok())
        .map(|key| {
            tracing::info!("ed25519 webhook public key loaded; ed25519 signature path enabled");
            Arc::new(key)
        })
        .or_else(|| {
            tracing::info!(
                "{} unset; ed25519 signature path disabled (HMAC-SHA256 required)",
                config::ENV_WEBHOOK_ED25519_PUBKEY
            );
            None
        });

    let state = ReceiverState {
        config: config.clone(),
        secret,
        ed25519_key,
        dispatcher,
    };

    let bind = config.bind_addr.clone();
    let listener = match tokio::net::TcpListener::bind(&bind).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%bind, %error, "failed to bind receiver socket");
            return std::process::ExitCode::FAILURE;
        }
    };
    tracing::info!(%bind, path = WEBHOOK_PATH, "ci-webhook-gateway listening");

    let app = router(state);
    let serve = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());
    if let Err(error) = serve.await {
        tracing::error!(%error, "receiver terminated with error");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

/// Resolve on the first SIGINT (Ctrl-C) or SIGTERM (k8s pod stop).
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
    tracing::info!("shutdown signal received; draining");
}

/// Kick the Jenkins pipeline with a minimal HTTP/1.1 POST over TCP.
///
/// Real transport (no stub): connects to the host:port from `url`, writes the
/// request, and treats a 2xx status line as success. TLS is intentionally not
/// handled here — the in-cluster Jenkins endpoint is plain HTTP on the mesh
/// (per `oyaCiLane.groovy`'s `FORGE_API` precedent); a TLS ingress terminates
/// externally. Returns `Err(reason)` on any transport/parse/non-2xx outcome so
/// the dispatcher surfaces a typed `DispatchTransport` error.
fn kick_jenkins(url: String, kickoff: PipelineKickoff) -> std::result::Result<(), String> {
    // Block on a small dedicated runtime task; the dispatcher calls this from
    // async context but the kick itself is a short, bounded request.
    let (host, port, path) = parse_http_url(&url)?;
    let body = kickoff_json(&kickoff).to_string();

    let handle = tokio::runtime::Handle::try_current()
        .map_err(|e| format!("no tokio runtime for Jenkins kick: {e}"))?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\n\
         Content-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len(),
    );

    // Run the blocking-ish IO on the current runtime via block_in_place to
    // avoid stalling the async reactor.
    tokio::task::block_in_place(|| {
        handle.block_on(async move {
            let mut stream = TcpStream::connect((host.as_str(), port))
                .await
                .map_err(|e| format!("connect {host}:{port}: {e}"))?;
            stream
                .write_all(request.as_bytes())
                .await
                .map_err(|e| format!("write: {e}"))?;
            let mut buf = Vec::with_capacity(256);
            // Read just enough for the status line.
            let mut chunk = [0u8; 256];
            let n = stream
                .read(&mut chunk)
                .await
                .map_err(|e| format!("read: {e}"))?;
            buf.extend_from_slice(&chunk[..n]);
            let head = String::from_utf8_lossy(&buf);
            let status_ok = head
                .split_whitespace()
                .nth(1)
                .and_then(|code| code.parse::<u16>().ok())
                .map(|code| (200..300).contains(&code))
                .unwrap_or(false);
            if status_ok {
                Ok(())
            } else {
                Err(format!(
                    "Jenkins kick non-2xx: {}",
                    head.lines().next().unwrap_or("<no status line>")
                ))
            }
        })
    })
}

/// POST the gate-run body to the oya-ci-controller `/gate-run` endpoint.
///
/// Uses the same minimal HTTP/1.1-over-TCP transport as `kick_jenkins` — no
/// extra dependencies (in-cluster plain HTTP; TLS terminates at the mesh
/// ingress). When `bearer` is `Some`, an `Authorization: Bearer <token>` header
/// is sent so the controller's fail-closed `/gate-run` (keystone-1) admits the
/// request. Returns `Err(reason)` on any transport/parse/non-2xx outcome.
fn post_controller(
    url: String,
    body: GateRunBody,
    bearer: Option<String>,
) -> std::result::Result<(), String> {
    let (host, port, path) = parse_http_url(&url)?;
    let json_body = serde_json::json!({
        "pr_number": body.pr_number,
        "head_sha": body.head_sha,
        "base_ref": body.base_ref,
    })
    .to_string();

    let handle = tokio::runtime::Handle::try_current()
        .map_err(|e| format!("no tokio runtime for controller post: {e}"))?;
    // Only emit a non-empty bearer; an empty/whitespace token is treated as unset.
    let auth_header = match bearer.as_deref().map(str::trim) {
        Some(token) if !token.is_empty() => format!("Authorization: Bearer {token}\r\n"),
        _ => String::new(),
    };
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\n\
         {auth_header}Content-Length: {len}\r\nConnection: close\r\n\r\n{json_body}",
        len = json_body.len(),
    );

    tokio::task::block_in_place(|| {
        handle.block_on(async move {
            let mut stream = TcpStream::connect((host.as_str(), port))
                .await
                .map_err(|e| format!("connect {host}:{port}: {e}"))?;
            stream
                .write_all(request.as_bytes())
                .await
                .map_err(|e| format!("write: {e}"))?;
            let mut buf = Vec::with_capacity(256);
            let mut chunk = [0u8; 256];
            let n = stream
                .read(&mut chunk)
                .await
                .map_err(|e| format!("read: {e}"))?;
            buf.extend_from_slice(&chunk[..n]);
            let head = String::from_utf8_lossy(&buf);
            let status_ok = head
                .split_whitespace()
                .nth(1)
                .and_then(|code| code.parse::<u16>().ok())
                .map(|code| (200..300).contains(&code))
                .unwrap_or(false);
            if status_ok {
                Ok(())
            } else {
                Err(format!(
                    "controller /gate-run non-2xx: {}",
                    head.lines().next().unwrap_or("<no status line>")
                ))
            }
        })
    })
}

fn kickoff_json(kickoff: &PipelineKickoff) -> serde_json::Value {
    match kickoff {
        PipelineKickoff::PullRequest {
            pr_number,
            head_ref,
            head_sha,
            revalidation,
        } => serde_json::json!({
            "kind": kickoff.kind(),
            "pr_number": pr_number,
            "head_ref": head_ref,
            "head_sha": head_sha,
            "revalidation": revalidation,
        }),
        PipelineKickoff::IssueSnapshot {
            issue_number,
            action,
            repository_full_name,
            snapshot_id,
            labels,
        } => serde_json::json!({
            "kind": kickoff.kind(),
            "issue_number": issue_number,
            "action": action.id(),
            "repository_full_name": repository_full_name,
            "snapshot_id": snapshot_id,
            "labels": labels,
        }),
        PipelineKickoff::PushSnapshot {
            reference,
            deliverable_id,
            before_sha,
            after_sha,
            sender_login,
            pusher_login,
            repository_full_name,
            commits,
            snapshot_id,
        } => serde_json::json!({
            "kind": kickoff.kind(),
            "reference": reference,
            "deliverable_id": deliverable_id,
            "before_sha": before_sha,
            "after_sha": after_sha,
            "sender_login": sender_login,
            "pusher_login": pusher_login,
            "repository_full_name": repository_full_name,
            "commits": commits,
            "snapshot_id": snapshot_id,
        }),
    }
}

/// Parse `http://host[:port]/path` → (host, port, path). HTTPS is rejected
/// (the in-cluster endpoint is plain HTTP; TLS terminates at ingress).
fn parse_http_url(url: &str) -> std::result::Result<(String, u16, String), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("dispatch URL must be http://… (got {url:?})"))?;
    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (
            h.to_owned(),
            p.parse::<u16>()
                .map_err(|_| format!("bad port in {url:?}"))?,
        ),
        None => (authority.to_owned(), 80u16),
    };
    if host.is_empty() {
        return Err(format!("empty host in {url:?}"));
    }
    Ok((host, port, path.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_with_port_and_path() {
        let (h, p, path) =
            parse_http_url("http://jenkins.oya-ci-jenkins.svc:8080/job/x/build").unwrap();
        assert_eq!(h, "jenkins.oya-ci-jenkins.svc");
        assert_eq!(p, 8080);
        assert_eq!(path, "/job/x/build");
    }

    #[test]
    fn parse_url_default_port() {
        let (h, p, path) = parse_http_url("http://jenkins/build").unwrap();
        assert_eq!(h, "jenkins");
        assert_eq!(p, 80);
        assert_eq!(path, "/build");
    }

    #[test]
    fn https_is_rejected() {
        assert!(parse_http_url("https://jenkins/build").is_err());
    }

    #[test]
    fn no_path_defaults_to_root() {
        let (_h, _p, path) = parse_http_url("http://jenkins").unwrap();
        assert_eq!(path, "/");
    }
}
