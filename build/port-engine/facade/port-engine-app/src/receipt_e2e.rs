//! Six-axis receipt end-to-end demos for W0-B Slice 6 (ADR-0637 D2 / receipt axes).
//!
//! Builds complete [`Receipt`] values from the fleet pin + stub digests, emits previous/current
//! trees via empty and syn/quote paths, and classifies with [`port_engine_kernel::verify`].
//! No filesystem hashing adapter yet — digests are explicit W0 stubs so the seam is exercised.

use std::collections::BTreeMap;
use std::fmt;

use port_engine_api::{Digest, PortError, Receipt, ReceiptAxis, RegionId, RECEIPT_AXES};
use port_engine_kernel::{verify, Delta, Verdict, Verification};
use port_engine_rust_ir::{EmptyRenderer, RustIr, SynQuoteRenderer};

use crate::driver;

/// Stable engine-binary digest stub until a hashing adapter lands.
pub const ENGINE_DIGEST_STUB: &str = "port-engine-app-slice6-v0";
/// Stable rule-pack digest stub (`specs/port-rules/**` still unauthored).
pub const RULEPACK_DIGEST_STUB: &str = "rulepack-absent-v0";
/// Stable toolchain digest stub (cell remap PARKED).
pub const TOOLCHAIN_DIGEST_STUB: &str = "toolchain-stub-v0";
/// Snapshot digest for the empty-stub emit path.
pub const SNAPSHOT_EMPTY: &str = "snapshot-empty-stub-v0";
/// Snapshot digest for the syn/quote emit path.
pub const SNAPSHOT_SYN: &str = "snapshot-syn-quote-v0";

/// One named verify scenario and its outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenarioResult {
    /// Scenario name (`unchanged` / `explained` / `unexplained` / `incomplete`).
    pub name: &'static str,
    /// Kernel verification outcome.
    pub verification: Verification,
}

/// Aggregate of the four W0 Slice 6 receipt scenarios.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SixAxisReport {
    /// Fleet pin identity used on every complete receipt.
    pub pin: String,
    /// Per-scenario results in declaration order.
    pub scenarios: Vec<ScenarioResult>,
}

impl SixAxisReport {
    /// True when every scenario matched its expected verdict/delta class.
    #[must_use]
    pub fn all_expected(&self) -> bool {
        self.scenarios.iter().all(|s| scenario_matches(s))
    }
}

fn scenario_matches(s: &ScenarioResult) -> bool {
    match s.name {
        "unchanged" => {
            s.verification.verdict == Verdict::Green
                && matches!(s.verification.delta, Delta::Unchanged)
        }
        "explained" => {
            s.verification.verdict == Verdict::Green
                && matches!(
                    &s.verification.delta,
                    Delta::Explained { axes, .. }
                        if axes.contains(&ReceiptAxis::Formatter)
                            && axes.contains(&ReceiptAxis::Snapshot)
                )
        }
        "unexplained" => {
            s.verification.verdict == Verdict::Red
                && matches!(s.verification.delta, Delta::Unexplained { .. })
        }
        "incomplete" => {
            s.verification.verdict == Verdict::Red
                && matches!(s.verification.delta, Delta::IncompleteReceipt { .. })
        }
        _ => false,
    }
}

/// Build a complete six-axis receipt (every axis non-empty).
#[must_use]
pub fn complete_receipt(pin: &str, snapshot: &str, formatter: &str) -> Receipt {
    Receipt {
        pin: pin.to_owned(),
        snapshot_digest: Digest(snapshot.to_owned()),
        engine_digest: Digest(ENGINE_DIGEST_STUB.to_owned()),
        rulepack_digest: Digest(RULEPACK_DIGEST_STUB.to_owned()),
        toolchain_digest: Digest(TOOLCHAIN_DIGEST_STUB.to_owned()),
        formatter_digest: Digest(formatter.to_owned()),
    }
}

/// Prove the closed receipt axis set is exactly six (compile-time contract surface).
#[must_use]
pub fn receipt_axis_count() -> usize {
    RECEIPT_AXES.len()
}

/// Emit empty-stub region bytes via kernel [`emit`](port_engine_kernel::emit).
///
/// # Errors
/// [`PortError`] from the empty renderer / kernel emit seam.
pub fn emit_empty_tree(formatter: &str) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
    let ir = RustIr::new(&["stub"]);
    let renderer = EmptyRenderer::new(formatter);
    port_engine_kernel::emit(&renderer, &ir)
}

/// Emit syn/quote region bytes via the typed Slice 5 path.
///
/// # Errors
/// [`PortError`] from syn parse or [`SynQuoteRenderer::render_rust_ir`].
pub fn emit_syn_tree(formatter: &str, src: &str) -> Result<BTreeMap<RegionId, Vec<u8>>, PortError> {
    let mut ir = RustIr::new(&["stub"]);
    ir.set_file_from_str("stub", src)?;
    let renderer = SynQuoteRenderer::new(formatter);
    renderer.render_rust_ir(&ir)
}

/// Typed refusal from the Slice 6 e2e harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum E2eError {
    /// Fleet pin could not load.
    Pin(String),
    /// Emit / render refused.
    Port(PortError),
    /// A scenario did not match its expected verdict class.
    Unexpected {
        /// Scenario name.
        name: &'static str,
        /// Debug form of the actual verification.
        actual: String,
    },
}

impl fmt::Display for E2eError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pin(detail) => write!(f, "receipt e2e pin load failed: {detail}"),
            Self::Port(err) => write!(f, "receipt e2e emit failed: {err}"),
            Self::Unexpected { name, actual } => {
                write!(f, "receipt e2e scenario `{name}` unexpected: {actual}")
            }
        }
    }
}

impl std::error::Error for E2eError {}

/// Run the four six-axis receipt scenarios and refuse if any expectation misses.
///
/// # Errors
/// [`E2eError`] on pin/emit failure or unexpected verdict.
pub fn run_six_axis_e2e() -> Result<SixAxisReport, E2eError> {
    if receipt_axis_count() != 6 {
        return Err(E2eError::Unexpected {
            name: "axis-count",
            actual: format!("expected 6 axes, got {}", receipt_axis_count()),
        });
    }

    let pin = driver::fleet_pin().map_err(|err| E2eError::Pin(err.to_string()))?;

    let empty_a = emit_empty_tree("fmt-empty-a").map_err(E2eError::Port)?;
    let empty_b = emit_empty_tree("fmt-empty-a").map_err(E2eError::Port)?;
    let syn = emit_syn_tree("fmt-syn-b", "pub fn stub() {}").map_err(E2eError::Port)?;

    let receipt_empty = complete_receipt(&pin, SNAPSHOT_EMPTY, "fmt-empty-a");
    let receipt_syn = complete_receipt(&pin, SNAPSHOT_SYN, "fmt-syn-b");
    let receipt_incomplete = Receipt {
        pin: String::new(),
        snapshot_digest: Digest(SNAPSHOT_SYN.to_owned()),
        engine_digest: Digest(ENGINE_DIGEST_STUB.to_owned()),
        rulepack_digest: Digest(RULEPACK_DIGEST_STUB.to_owned()),
        toolchain_digest: Digest(TOOLCHAIN_DIGEST_STUB.to_owned()),
        formatter_digest: Digest("fmt-syn-b".to_owned()),
    };

    let scenarios = vec![
        ScenarioResult {
            name: "unchanged",
            verification: verify(&receipt_empty, &empty_a, &receipt_empty, &empty_b),
        },
        ScenarioResult {
            name: "explained",
            verification: verify(&receipt_empty, &empty_a, &receipt_syn, &syn),
        },
        ScenarioResult {
            name: "unexplained",
            verification: verify(&receipt_empty, &empty_a, &receipt_empty, &syn),
        },
        ScenarioResult {
            name: "incomplete",
            verification: verify(&receipt_empty, &empty_a, &receipt_incomplete, &syn),
        },
    ];

    let report = SixAxisReport { pin, scenarios };
    for s in &report.scenarios {
        if !scenario_matches(s) {
            return Err(E2eError::Unexpected {
                name: s.name,
                actual: format!("{:?}", s.verification),
            });
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_axis_e2e_matches_expected_verdicts() {
        let report = run_six_axis_e2e().expect("slice6 e2e must hold");
        assert!(report.all_expected());
        assert_eq!(report.scenarios.len(), 4);
        assert!(!report.pin.is_empty());
        assert_eq!(receipt_axis_count(), 6);
    }

    #[test]
    fn complete_receipt_has_no_incomplete_axes() {
        let r = complete_receipt("pin", "snap", "fmt");
        assert!(r.incomplete_axes().is_empty());
    }
}
