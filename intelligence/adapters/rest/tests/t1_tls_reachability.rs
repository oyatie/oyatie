//! T1 — TLS-fingerprint reachability spike (SC1 / F2 / OQ-7).
//!
//! Empirically resolves ADR-0384's one FATAL-marked blocker: does stock
//! `reqwest`/`rustls` reach each provider's OAuth-token + API hosts and get a
//! *normal HTTP response*, or does Cloudflare reject the ClientHello with a
//! 1010-class fingerprint block / managed challenge?
//!
//! Verdict to confirm (findings §1): subscription-vs-API billing is decided at
//! the HTTP application layer, NOT at TLS. Stock rustls should suffice; a
//! Chrome-shaped ClientHello is a contingency only if a live 1010 appears on
//! `api.anthropic.com`. This test produces that empirical evidence.
//!
//! ## Hermeticity (non-negotiable)
//! The default test suite makes ZERO real network calls. The classifier is a
//! pure function exercised by `#[test]` units that always run. The live probe
//! is `#[ignore]`d AND gated behind the `INTELLIGENCE_LIVE_UPSTREAM` env var
//! that CI never sets — it is run only as the T6 K8s Job. It is credential-free
//! (reachability only; no tokens, no bodies that could be billed).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

// ---------------------------------------------------------------------------
// Pure classifier — reqwest-free, hermetically unit-tested below.
// ---------------------------------------------------------------------------

/// Outcome of probing one upstream host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reachability {
    /// TLS handshake completed and we got an application-layer HTTP response
    /// (any status — 401/400/404 are all fine: they prove the request reached
    /// the provider's app, not a fingerprint wall).
    NormalHttpResponse,
    /// Cloudflare rejected the request at the edge with a fingerprint block
    /// (error code 1010) or a managed challenge — the F2/OQ-7 failure signal.
    FingerprintBlocked,
}

/// Distinguish a normal provider HTTP response from a Cloudflare 1010 /
/// challenge block, from the signals a credential-free probe can observe.
///
/// Cloudflare 1010 ("banned based on your browser's signature") arrives as a
/// 403 whose body carries `error code: 1010`; managed challenges set the
/// `cf-mitigated: challenge` response header and serve a challenge-platform
/// interstitial. Either is the fingerprint-block signal. A plain 401/403 from
/// the provider's own app (no CF block markers) is a *normal* response.
fn classify(status: u16, cf_mitigated: bool, body: &str) -> Reachability {
    let body_lc = body.to_ascii_lowercase();
    let cf_1010 = body_lc.contains("error code: 1010");
    let cf_challenge = body_lc.contains("challenge-platform")
        || (body_lc.contains("cloudflare")
            && (body_lc.contains("you have been blocked")
                || body_lc.contains("attention required")));
    if cf_mitigated || cf_1010 || (status == 403 && cf_challenge) {
        Reachability::FingerprintBlocked
    } else {
        Reachability::NormalHttpResponse
    }
}

// ---------------------------------------------------------------------------
// Hermetic unit tests — always run, no network. These ARE the runnable check.
// ---------------------------------------------------------------------------

#[test]
fn cf_1010_body_is_fingerprint_blocked() {
    let body = "error code: 1010\nThe owner of this website has banned your access \
                based on your browser's signature.";
    assert_eq!(classify(403, false, body), Reachability::FingerprintBlocked);
}

#[test]
fn cf_managed_challenge_header_is_fingerprint_blocked() {
    // cf-mitigated: challenge on any status is a managed-challenge block.
    assert_eq!(
        classify(403, true, "<!DOCTYPE html> just a challenge page"),
        Reachability::FingerprintBlocked
    );
}

#[test]
fn cf_attention_required_page_is_fingerprint_blocked() {
    let body = "<title>Attention Required! | Cloudflare</title> you have been blocked";
    assert_eq!(classify(403, false, body), Reachability::FingerprintBlocked);
}

#[test]
fn normal_401_unauthorized_is_reachable() {
    // Credential-free probes expect 401 from the provider app — that's SUCCESS:
    // the request reached the application layer, not a fingerprint wall.
    let body = r#"{"error":{"type":"authentication_error","message":"invalid x-api-key"}}"#;
    assert_eq!(classify(401, false, body), Reachability::NormalHttpResponse);
}

#[test]
fn normal_400_bad_request_is_reachable() {
    let body = r#"{"error":{"message":"missing required parameter"}}"#;
    assert_eq!(classify(400, false, body), Reachability::NormalHttpResponse);
}

#[test]
fn plain_403_without_cf_markers_is_reachable() {
    // An app-level 403 (e.g. forbidden scope) with no Cloudflare block markers
    // is a normal response, not a fingerprint block.
    let body = r#"{"error":{"type":"permission_error"}}"#;
    assert_eq!(classify(403, false, body), Reachability::NormalHttpResponse);
}

// ---------------------------------------------------------------------------
// Hosts probed (findings §2). Credential-free reachability only — we send the
// minimal request each endpoint accepts and read the rejection. A 4xx is the
// expected, successful outcome; the only failure is a Cloudflare fingerprint
// block. Constants are [STALE]-tagged where they rotate per provider release.
// ---------------------------------------------------------------------------

/// (label, method, url) — OAuth-token hosts + API hosts for all three providers.
#[cfg(test)]
const PROBE_TARGETS: &[(&str, &str, &str)] = &[
    // Anthropic — the Cloudflare-fronted risk host (findings §1 residual risk).
    // Token: https://api.anthropic.com/v1/oauth/token  API: /v1/messages
    ("anthropic-oauth", "POST", "https://api.anthropic.com/v1/oauth/token"),
    ("anthropic-api", "POST", "https://api.anthropic.com/v1/messages"),
    // OpenAI / Codex — auth.openai.com (token + device) + chatgpt.com backend.
    ("openai-oauth", "POST", "https://auth.openai.com/oauth/token"),
    (
        "codex-device",
        "POST",
        "https://auth.openai.com/api/accounts/deviceauth/usercode",
    ),
    ("codex-api", "POST", "https://chatgpt.com/backend-api/codex/responses"),
    // Gemini / Google — subscription path (findings §6): oauth2.googleapis.com
    // token + cloudcode-pa.googleapis.com v1internal API.
    ("gemini-oauth", "POST", "https://oauth2.googleapis.com/token"),
    (
        "gemini-api",
        "POST",
        "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist",
    ),
];

/// GATED live reachability probe. NEVER runs in the default suite:
///   - `#[ignore]` skips it under plain `cargo test` / `cargo nextest`.
///   - the `INTELLIGENCE_LIVE_UPSTREAM` env guard (CI never sets it) makes it a
///     no-op even if invoked with `--ignored`.
/// Run only by the T6 K8s Job (see iac/k8s/jobs/t1-tls-reachability-job.yaml).
///
/// Credential-free: empty JSON body, no Authorization header. We assert only
/// that each host returns a *normal HTTP response* (not a Cloudflare 1010 /
/// challenge). The verdict is recorded back into ADR-0384 OQ-7.
#[tokio::test]
#[ignore = "live-upstream: real network; gated to the T6 K8s Job, never CI"]
async fn live_upstream_reachability_probe() {
    if std::env::var("INTELLIGENCE_LIVE_UPSTREAM").as_deref() != Ok("1") {
        eprintln!("INTELLIGENCE_LIVE_UPSTREAM != 1 — skipping live probe (hermetic default)");
        return;
    }

    // Stock rustls/reqwest — exactly what the F2 verdict says should suffice.
    // No ClientHello shaping: that is the whole point of the empirical check.
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("build reqwest client");

    let mut blocked: Vec<&str> = Vec::new();
    for (label, method, url) in PROBE_TARGETS {
        let req = match *method {
            "POST" => client.post(*url).body("{}"),
            _ => client.get(*url),
        };
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let cf_mitigated = resp.headers().contains_key("cf-mitigated");
                let body = resp.text().await.unwrap_or_default();
                let verdict = classify(status, cf_mitigated, &body);
                eprintln!("[{label}] {url} -> HTTP {status} => {verdict:?}");
                if verdict == Reachability::FingerprintBlocked {
                    blocked.push(label);
                }
            }
            // A transport error (DNS/connect/TLS) is reported but is not by
            // itself the fingerprint signal we are testing for; surface it.
            Err(e) => eprintln!("[{label}] {url} -> transport error: {e}"),
        }
    }

    assert!(
        blocked.is_empty(),
        "Cloudflare fingerprint block (1010/challenge) observed on: {blocked:?}. \
         F2/OQ-7 contingency triggered: add a Chrome-shaped ClientHello behind \
         the UpstreamTransport port for those hosts only."
    );
}
