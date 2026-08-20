//! CI webhook gateway binary entry point (ADR-0387).
//!
//! Reads configuration from environment variables:
//!
//! | Variable                        | Default        | Description                       |
//! |---------------------------------|----------------|-----------------------------------|
//! | `OYA_CI_WEBHOOK_LISTEN_ADDR`    | `0.0.0.0:8080` | Bind address                      |
//! | `OYA_CI_WEBHOOK_GITHUB_OWNER`   | (required)     | GitHub repo owner                 |
//! | `OYA_CI_WEBHOOK_GITHUB_REPO`    | (required)     | GitHub repo name                  |
//! | `OYA_CI_WEBHOOK_GITHUB_TOKEN`   | (required)     | GitHub token for status posting   |
//! | `OYA_CI_WEBHOOK_ED25519_PUBKEY` | (required)     | Hex-encoded 32-byte ed25519 pubkey|
//! | `OYA_CI_WEBHOOK_TARGET_BRANCH`  | `dev`          | Gated base branch                 |

use ci_webhook_gateway_app::{AppState, build_router, replay::DeliveryGuard};
use ci_webhook_gateway_authz_cedar_adapter::CedarWebhookGate;
use ci_webhook_gateway_ed25519_adapter::Ed25519Verifier;
use ci_webhook_gateway_github_adapter::GitHubStatusPoster;
use ed25519_dalek::VerifyingKey;
use std::sync::{Arc, Mutex};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()))
        .init();

    let listen_addr = env_or("OYA_CI_WEBHOOK_LISTEN_ADDR", "0.0.0.0:8080");
    let github_owner = env_required("OYA_CI_WEBHOOK_GITHUB_OWNER");
    let github_repo = env_required("OYA_CI_WEBHOOK_GITHUB_REPO");
    let github_token = env_required("OYA_CI_WEBHOOK_GITHUB_TOKEN");
    let pubkey_hex = env_required("OYA_CI_WEBHOOK_ED25519_PUBKEY");
    let target_branch = env_or("OYA_CI_WEBHOOK_TARGET_BRANCH", "dev");

    // Decode the ed25519 public key from hex.
    let pubkey_bytes = hex_decode_32(&pubkey_hex);
    let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes).unwrap_or_else(|e| {
        eprintln!("invalid OYA_CI_WEBHOOK_ED25519_PUBKEY: {e}");
        std::process::exit(1);
    });

    let verifier = Arc::new(Ed25519Verifier::new(verifying_key));
    let authz = Arc::new(CedarWebhookGate::with_default_policy().unwrap_or_else(|e| {
        eprintln!("Cedar policy load failed: {e}");
        std::process::exit(1);
    }));
    let status_poster = Arc::new(GitHubStatusPoster::new(
        &github_owner,
        &github_repo,
        &github_token,
    ));

    let state = AppState {
        verifier,
        authz,
        status_poster,
        target_branch,
        github_owner,
        github_repo,
        delivery_guard: Arc::new(Mutex::new(DeliveryGuard::with_default_ttl())),
    };

    let app = build_router(state);

    info!(listen_addr, "ci-webhook-gateway starting");

    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to bind {listen_addr}: {e}");
            std::process::exit(1);
        });

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        eprintln!("server error: {e}");
        std::process::exit(1);
    });
}

fn env_required(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| {
        eprintln!("required env var {key} is not set");
        std::process::exit(1);
    })
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn hex_decode_32(hex: &str) -> [u8; 32] {
    let hex = hex.trim();
    if hex.len() != 64 {
        eprintln!(
            "OYA_CI_WEBHOOK_ED25519_PUBKEY must be 64 hex chars (32 bytes), got {}",
            hex.len()
        );
        std::process::exit(1);
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hi = from_hex_nibble(chunk[0]);
        let lo = from_hex_nibble(chunk[1]);
        out[i] = (hi << 4) | lo;
    }
    out
}

fn from_hex_nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => {
            eprintln!("invalid hex character in OYA_CI_WEBHOOK_ED25519_PUBKEY");
            std::process::exit(1);
        }
    }
}
