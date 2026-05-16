//! Webhook-receiver app — ADR-0112 wave-A library surface.
//!
//! This module holds the I/O-bearing pieces that orchestrate the
//! pure-domain `oya-foundry-webhook-receiver-kernel`. Everything here
//! is intentionally test-friendly: the routing pipeline is exposed as
//! [`process_simulated_delivery`] so the `--simulate-delivery` binary
//! flag and the in-crate unit tests share one code path.
//!
//! The HTTP server (axum) lives in `main.rs`. The two share the same
//! [`Dispatch`] state object so simulating a delivery from disk is
//! literally `Dispatch::from_paths(...).process(delivery)`.
//!
//! ADR-0083 Tier 1: production code does not use `.unwrap()` /
//! `.expect()` / `panic!()`. Tests carry the `cfg(test)` exemption.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use oya_foundry_webhook_receiver_kernel::{
    DedupLookup, DedupOutcome, DeliveryLogEntry, EventRouterRow, HmacVerificationError,
    find_dedup_status, parse_delivery_log, route_event, verify_hmac_sha256,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------
// Synthetic delivery format (used by --simulate-delivery + tests)
// ---------------------------------------------------------------------

/// One synthetic delivery as accepted by `--simulate-delivery`.
///
/// Mirrors the four pieces a real GitHub delivery carries:
/// - `delivery_id` ↔ `X-GitHub-Delivery`
/// - `event` ↔ `X-GitHub-Event`
/// - `signature` ↔ `X-Hub-Signature-256`
/// - `payload` ↔ raw request body (JSON object with an `action` field
///   for events that carry one)
///
/// `signature` is optional in synthetic deliveries because the most
/// common local-test posture is to skip HMAC for a known-good payload
/// (the `--skip-hmac` flag controls this). For real production wiring
/// the receiver always requires a valid signature.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulatedDelivery {
    pub delivery_id: String,
    pub event: String,
    #[serde(default)]
    pub signature: Option<String>,
    pub payload: serde_json::Value,
}

impl SimulatedDelivery {
    /// Extract the `action` field from the payload (returns `""` if
    /// the payload is not an object, has no `action`, or carries a
    /// non-string value — that matches the wire convention for events
    /// like `push` that don't carry an `action`).
    pub fn action(&self) -> &str {
        self.payload
            .as_object()
            .and_then(|m| m.get("action"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    /// Extract the `conclusion` field from the payload, scanning the
    /// two nested locations GitHub uses:
    /// - `workflow_run.conclusion` (for `workflow_run.completed`)
    /// - `check_suite.conclusion` (for `check_suite.completed`)
    ///
    /// Returns `""` when no conclusion is present so the router falls
    /// through cleanly to its action-only and wildcard rules.
    pub fn conclusion(&self) -> &str {
        let obj = match self.payload.as_object() {
            Some(o) => o,
            None => return "",
        };
        for key in ["workflow_run", "check_suite"] {
            if let Some(nested) = obj.get(key).and_then(|v| v.as_object())
                && let Some(c) = nested.get("conclusion").and_then(|v| v.as_str())
            {
                return c;
            }
        }
        ""
    }

    /// Raw payload bytes for HMAC verification.
    pub fn payload_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.payload).unwrap_or_default()
    }
}

// ---------------------------------------------------------------------
// Routing outcomes
// ---------------------------------------------------------------------

/// What the receiver decided to do with a delivery.
#[derive(Clone, Debug, PartialEq)]
pub enum DispatchOutcome {
    /// First time we saw this delivery; routed to `agent`.
    Accepted { agent: String, purpose: String },
    /// Same `delivery_id` seen before within the TTL window.
    Deduplicated {
        prior_outcome: DedupOutcome,
        at_seconds: u64,
    },
    /// Prior delivery is older than the 7-day TTL; routed to `agent`.
    AcceptedAfterExpiry {
        agent: String,
        purpose: String,
        prior_at_seconds: u64,
    },
    /// `(event, action)` did not appear in the router table.
    RoutingFailed { event: String, action: String },
    /// HMAC was required but missing/malformed/mismatched.
    HmacRejected(HmacVerificationError),
    /// Anomaly: two prior rows with the same `delivery_id` carry
    /// conflicting `dedup_outcome` values.
    ConflictingPriorOutcomes,
}

// ---------------------------------------------------------------------
// Dispatch state
// ---------------------------------------------------------------------

/// All the disk-loaded state the receiver needs to route one delivery:
/// the event-router table, the delivery log, and (optionally) the
/// HMAC secret.
pub struct Dispatch {
    pub router: Vec<EventRouterRow>,
    pub log: Vec<DeliveryLogEntry>,
    pub secret: Option<String>,
}

/// Configuration for [`Dispatch::from_paths`].
pub struct DispatchPaths<'a> {
    pub router_yaml: &'a Path,
    pub delivery_log_json: &'a Path,
    /// If `Some`, the HMAC secret is read from this file. Production
    /// wiring reads from OpenBao at `sref://openbao/oya/foundry/github-webhook-secret`;
    /// for now the binary supports a file-backed fallback at
    /// `~/.openbao/oya/foundry/github-webhook-secret`.
    /// TODO: replace with the OpenBao adapter once the SecretReference
    /// surface lands (per ADR-0112 §"Signature handling").
    pub secret_path: Option<&'a Path>,
}

impl Dispatch {
    /// Load all dispatch state from disk.
    pub fn from_paths(paths: &DispatchPaths<'_>) -> Result<Self, DispatchLoadError> {
        let router_text = fs::read_to_string(paths.router_yaml)
            .map_err(|e| DispatchLoadError::RouterIo(paths.router_yaml.into(), e.to_string()))?;
        let router = parse_router_yaml(&router_text)
            .map_err(|e| DispatchLoadError::RouterParse(paths.router_yaml.into(), e))?;
        let log_text = fs::read_to_string(paths.delivery_log_json)
            .map_err(|e| DispatchLoadError::LogIo(paths.delivery_log_json.into(), e.to_string()))?;
        let log = parse_delivery_log(&log_text).map_err(|e| {
            DispatchLoadError::LogParse(paths.delivery_log_json.into(), format!("{e:?}"))
        })?;
        let secret = match paths.secret_path {
            Some(p) => match fs::read_to_string(p) {
                Ok(s) => Some(s.trim().to_string()),
                Err(e) => return Err(DispatchLoadError::SecretIo(p.into(), e.to_string())),
            },
            None => None,
        };
        Ok(Self {
            router,
            log,
            secret,
        })
    }

    /// Process one synthetic or real delivery through the full kernel
    /// pipeline: HMAC verify (when secret present + not skipped),
    /// dedup lookup, then router-table lookup.
    ///
    /// `now_seconds` is the wall-clock injected by the caller (the
    /// binary uses `SystemTime::now()`; tests pass deterministic
    /// values).
    pub fn process(
        &self,
        delivery: &SimulatedDelivery,
        now_seconds: u64,
        skip_hmac: bool,
    ) -> DispatchOutcome {
        // 1. HMAC fail-closed gate (ADR-0112 §"Signature handling").
        if !skip_hmac && let Some(secret) = self.secret.as_deref() {
            let signature_header = delivery.signature.as_deref().unwrap_or("");
            if let Err(err) =
                verify_hmac_sha256(&delivery.payload_bytes(), signature_header, secret)
            {
                return DispatchOutcome::HmacRejected(err);
            }
        }
        // 2. Dedup lookup.
        match find_dedup_status(&self.log, &delivery.delivery_id, now_seconds) {
            DedupLookup::FirstDelivery => self.route_or_fail(delivery, None),
            DedupLookup::Deduplicated {
                outcome,
                at_seconds,
            } => DispatchOutcome::Deduplicated {
                prior_outcome: outcome,
                at_seconds,
            },
            DedupLookup::Expired { at_seconds } => self.route_or_fail(delivery, Some(at_seconds)),
            DedupLookup::ConflictingOutcomes => DispatchOutcome::ConflictingPriorOutcomes,
        }
    }

    fn route_or_fail(
        &self,
        delivery: &SimulatedDelivery,
        prior_at_seconds: Option<u64>,
    ) -> DispatchOutcome {
        let action = delivery.action();
        let conclusion = delivery.conclusion();
        match route_event(&delivery.event, action, conclusion, &self.router) {
            Some(row) => match prior_at_seconds {
                Some(prior) => DispatchOutcome::AcceptedAfterExpiry {
                    agent: row.agent.clone(),
                    purpose: row.purpose.clone(),
                    prior_at_seconds: prior,
                },
                None => DispatchOutcome::Accepted {
                    agent: row.agent.clone(),
                    purpose: row.purpose.clone(),
                },
            },
            None => DispatchOutcome::RoutingFailed {
                event: delivery.event.clone(),
                action: action.to_string(),
            },
        }
    }
}

#[derive(Debug)]
pub enum DispatchLoadError {
    RouterIo(PathBuf, String),
    RouterParse(PathBuf, String),
    LogIo(PathBuf, String),
    LogParse(PathBuf, String),
    SecretIo(PathBuf, String),
}

impl std::fmt::Display for DispatchLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RouterIo(p, e) => write!(f, "router I/O error at {}: {e}", p.display()),
            Self::RouterParse(p, e) => {
                write!(f, "router parse error at {}: {e}", p.display())
            }
            Self::LogIo(p, e) => write!(f, "delivery-log I/O error at {}: {e}", p.display()),
            Self::LogParse(p, e) => {
                write!(f, "delivery-log parse error at {}: {e}", p.display())
            }
            Self::SecretIo(p, e) => write!(f, "secret I/O error at {}: {e}", p.display()),
        }
    }
}

impl std::error::Error for DispatchLoadError {}

// ---------------------------------------------------------------------
// Router YAML parsing
// ---------------------------------------------------------------------

#[derive(Deserialize)]
struct RawRouterRow {
    event: String,
    #[serde(default)]
    action: String,
    /// Optional `conclusion` discriminator. Empty / missing string is
    /// normalized to `None` (row matches any conclusion).
    #[serde(default)]
    conclusion: String,
    agent: String,
    #[serde(default)]
    purpose: String,
}

#[derive(Deserialize)]
struct RawRouter {
    rows: Vec<RawRouterRow>,
}

fn parse_router_yaml(text: &str) -> Result<Vec<EventRouterRow>, String> {
    let raw: RawRouter = serde_yaml::from_str(text).map_err(|e| e.to_string())?;
    Ok(raw
        .rows
        .into_iter()
        .map(|r| EventRouterRow {
            event: r.event,
            action: r.action,
            conclusion: if r.conclusion.is_empty() {
                None
            } else {
                Some(r.conclusion)
            },
            agent: r.agent,
            purpose: r.purpose,
        })
        .collect())
}

// ---------------------------------------------------------------------
// Simulate-delivery entrypoint shared between bin + tests
// ---------------------------------------------------------------------

/// Load a synthetic delivery from disk and route it through the full
/// kernel pipeline. Returns the dispatch outcome.
///
/// `skip_hmac` skips HMAC verification (useful for local testing
/// against payloads we don't have a signed copy of). The bin sets it
/// to `true` when no `--secret-path` is provided.
pub fn process_simulated_delivery(
    dispatch: &Dispatch,
    delivery_path: &Path,
    skip_hmac: bool,
) -> Result<DispatchOutcome, ProcessError> {
    let text = fs::read_to_string(delivery_path)
        .map_err(|e| ProcessError::Io(delivery_path.into(), e.to_string()))?;
    let delivery: SimulatedDelivery = serde_json::from_str(&text)
        .map_err(|e| ProcessError::Parse(delivery_path.into(), e.to_string()))?;
    let now_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(dispatch.process(&delivery, now_seconds, skip_hmac))
}

#[derive(Debug)]
pub enum ProcessError {
    Io(PathBuf, String),
    Parse(PathBuf, String),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(p, e) => write!(f, "delivery I/O error at {}: {e}", p.display()),
            Self::Parse(p, e) => write!(f, "delivery parse error at {}: {e}", p.display()),
        }
    }
}

impl std::error::Error for ProcessError {}

// ---------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn router() -> Vec<EventRouterRow> {
        vec![
            EventRouterRow {
                event: "pull_request".to_string(),
                action: "opened".to_string(),
                conclusion: None,
                agent: "oya-foundry-vcs-orchestrator-app".to_string(),
                purpose: "Begin changeset state transition to pr_open".to_string(),
            },
            EventRouterRow {
                event: "workflow_run".to_string(),
                action: "completed".to_string(),
                conclusion: Some("success".to_string()),
                agent: "IP-004 dispatcher".to_string(),
                purpose: "Run multispectrum review".to_string(),
            },
            EventRouterRow {
                event: "workflow_run".to_string(),
                action: "completed".to_string(),
                conclusion: Some("failure".to_string()),
                agent: "IP-005 dispatcher".to_string(),
                purpose: "Run fix-loop with retry budget".to_string(),
            },
        ]
    }

    fn dispatch_no_secret(log: Vec<DeliveryLogEntry>) -> Dispatch {
        Dispatch {
            router: router(),
            log,
            secret: None,
        }
    }

    fn delivery(delivery_id: &str, event: &str, action: &str) -> SimulatedDelivery {
        SimulatedDelivery {
            delivery_id: delivery_id.to_string(),
            event: event.to_string(),
            signature: None,
            payload: json!({ "action": action, "number": 42 }),
        }
    }

    #[test]
    fn simulated_pr_opened_routes_to_orchestrator() {
        let dispatch = dispatch_no_secret(Vec::new());
        let outcome = dispatch.process(
            &delivery("delivery-AAA", "pull_request", "opened"),
            1_715_000_000,
            true,
        );
        match outcome {
            DispatchOutcome::Accepted { agent, .. } => {
                assert_eq!(agent, "oya-foundry-vcs-orchestrator-app");
            }
            other => panic!("expected Accepted; got {other:?}"),
        }
    }

    #[test]
    fn unknown_event_yields_routing_failed() {
        let dispatch = dispatch_no_secret(Vec::new());
        let outcome = dispatch.process(
            &delivery("delivery-BBB", "issue_comment", "created"),
            1_715_000_000,
            true,
        );
        assert!(matches!(outcome, DispatchOutcome::RoutingFailed { .. }));
    }

    #[test]
    fn redelivery_short_circuits() {
        let prior = DeliveryLogEntry {
            delivery_id: "delivery-CCC".to_string(),
            event: "pull_request".to_string(),
            action: "opened".to_string(),
            dedup_outcome: DedupOutcome::Accepted,
            at_seconds: 1_715_000_000,
        };
        let dispatch = dispatch_no_secret(vec![prior]);
        let outcome = dispatch.process(
            &delivery("delivery-CCC", "pull_request", "opened"),
            1_715_000_500,
            true,
        );
        assert!(matches!(outcome, DispatchOutcome::Deduplicated { .. }));
    }

    #[test]
    fn router_yaml_parses_expected_shape() {
        let text = r#"rows:
  - event: pull_request
    action: opened
    agent: oya-foundry-vcs-orchestrator-app
    purpose: Begin changeset state transition to pr_open
  - event: workflow_run
    action: completed
    conclusion: success
    agent: IP-004 dispatcher
    purpose: Multispectrum review
  - event: push
    agent: promotion workflow
    purpose: Trigger promotion
"#;
        let parsed = parse_router_yaml(text).expect("parses");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].event, "pull_request");
        assert_eq!(parsed[0].conclusion, None);
        assert_eq!(parsed[1].conclusion.as_deref(), Some("success"));
        assert_eq!(parsed[2].action, "");
        assert_eq!(parsed[2].conclusion, None);
    }

    #[test]
    fn workflow_run_completed_routes_by_conclusion() {
        let dispatch = dispatch_no_secret(Vec::new());
        let success = SimulatedDelivery {
            delivery_id: "delivery-WF-S".to_string(),
            event: "workflow_run".to_string(),
            signature: None,
            payload: json!({
                "action": "completed",
                "workflow_run": { "conclusion": "success", "id": 1 }
            }),
        };
        let outcome = dispatch.process(&success, 1_715_000_000, true);
        match outcome {
            DispatchOutcome::Accepted { agent, .. } => assert_eq!(agent, "IP-004 dispatcher"),
            other => panic!("expected Accepted -> IP-004; got {other:?}"),
        }

        let failure = SimulatedDelivery {
            delivery_id: "delivery-WF-F".to_string(),
            event: "workflow_run".to_string(),
            signature: None,
            payload: json!({
                "action": "completed",
                "workflow_run": { "conclusion": "failure", "id": 2 }
            }),
        };
        let outcome = dispatch.process(&failure, 1_715_000_000, true);
        match outcome {
            DispatchOutcome::Accepted { agent, .. } => assert_eq!(agent, "IP-005 dispatcher"),
            other => panic!("expected Accepted -> IP-005; got {other:?}"),
        }
    }
}
