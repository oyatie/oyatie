//! Facade driver wiring: composes kernel entry points with W0-B adapters.
//!
//! Slice 7 wires hashing + embedded rulepack v0; CLI exposes `digest` / `rulepack` / `plan`.

use port_engine_api::{Digest, RulePack, SourceModel, UnitId, w0_ready as api_ready};
use port_engine_frontend_go::w0_ready as frontend_ready;
use port_engine_hash::{digest_str, w0_ready as hash_ready};
use port_engine_rulepack::{LoadedRulePack, RulepackError, w0_ready as rulepack_ready};
use port_engine_rust_ir::{EmptyRenderer, RustIr, SynQuoteRenderer};
use port_engine_source_pin::{load_embedded, receipt_pin};

/// Slice 7 readiness: prior adapters + hash + rulepack wired.
pub const fn w0_ready() -> bool {
    api_ready()
        && port_engine_source_pin::w0_ready()
        && port_engine_rust_ir::w0_ready()
        && frontend_ready()
        && hash_ready()
        && rulepack_ready()
}

/// Load the fleet upstream pin (adapter boundary).
///
/// # Errors
/// Propagates [`port_engine_source_pin::PinError`] from the pin loader.
pub fn fleet_pin() -> Result<String, port_engine_source_pin::PinError> {
    let pin = load_embedded()?;
    Ok(receipt_pin(&pin))
}

/// Smoke the render seam with the Slice 3 empty renderer stub.
///
/// # Errors
/// Propagates [`port_engine_api::PortError`] from [`port_engine_kernel::emit`].
pub fn smoke_render_stub() -> Result<(), port_engine_api::PortError> {
    let ir = RustIr::new(&["stub"]);
    let renderer = EmptyRenderer::new("slice3-fmt-stub");
    let _ = port_engine_kernel::emit(&renderer, &ir)?;
    Ok(())
}

/// Smoke the Slice 5 syn/quote typed emit path (not the fail-closed dyn Renderer).
///
/// # Errors
/// Propagates [`port_engine_api::PortError`] from [`SynQuoteRenderer::render_rust_ir`].
pub fn smoke_syn_quote_render() -> Result<(), port_engine_api::PortError> {
    let mut ir = RustIr::new(&["stub"]);
    ir.set_file_from_str("stub", "pub fn stub() {}")?;
    let renderer = SynQuoteRenderer::new("slice5-fmt-syn-quote");
    let out = renderer.render_rust_ir(&ir)?;
    if out.len() != 1 {
        return Err(port_engine_api::PortError::Render {
            detail: format!("expected 1 region, got {}", out.len()),
        });
    }
    Ok(())
}

/// Hash UTF-8 text via the Slice 7 hashing adapter.
#[must_use]
pub fn smoke_digest(text: &str) -> Digest {
    digest_str(text)
}

/// Load embedded neutral rulepack v0 and return its content digest.
///
/// # Errors
/// Propagates [`RulepackError`] from the rulepack loader.
pub fn smoke_rulepack() -> Result<Digest, RulepackError> {
    let pack = LoadedRulePack::load_embedded()?;
    Ok(pack.digest())
}

/// Plan the embedded v0 rulepack against its declared example units.
///
/// # Errors
/// [`RulepackError`] on load failure, or [`port_engine_api::PortError`] from kernel plan.
pub fn smoke_plan() -> Result<usize, PlanSmokeError> {
    let pack = LoadedRulePack::load_embedded().map_err(PlanSmokeError::Rulepack)?;
    let model = RulepackModel;
    let plan = port_engine_kernel::plan(&model, &pack).map_err(PlanSmokeError::Port)?;
    Ok(plan.steps.len())
}

/// Typed refusal from the Slice 7 plan smoke.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanSmokeError {
    /// Rulepack load failed.
    Rulepack(RulepackError),
    /// Kernel plan refused.
    Port(port_engine_api::PortError),
}

impl std::fmt::Display for PlanSmokeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rulepack(err) => write!(f, "plan smoke rulepack: {err}"),
            Self::Port(err) => write!(f, "plan smoke kernel: {err}"),
        }
    }
}

impl std::error::Error for PlanSmokeError {}

struct RulepackModel;

impl SourceModel for RulepackModel {
    fn language(&self) -> &str {
        "go"
    }

    fn snapshot_digest(&self) -> Digest {
        digest_str("slice7-rulepack-model")
    }

    fn units(&self) -> Vec<UnitId> {
        vec![
            UnitId("example.com/a".into()),
            UnitId("example.com/b".into()),
        ]
    }
}

/// Re-export neutral kernel entry points for downstream CLI wiring.
pub use port_engine_kernel::{emit, plan, verify, Verdict};

/// Adapter readiness snapshot for diagnostics.
///
/// Order: `(pin, rust_ir, frontend, hash, rulepack)`.
#[must_use]
pub fn adapter_readiness() -> (bool, bool, bool, bool, bool) {
    (
        port_engine_source_pin::w0_ready(),
        port_engine_rust_ir::w0_ready(),
        frontend_ready(),
        hash_ready(),
        rulepack_ready(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice7_driver_wiring_is_ready() {
        assert!(w0_ready());
        fleet_pin().expect("fleet pin must load");
        smoke_render_stub().expect("empty renderer stub must emit");
        smoke_syn_quote_render().expect("syn/quote path must emit");
        let d = smoke_digest("port-engine");
        assert!(d.0.starts_with("sha256:"));
        let pack_digest = smoke_rulepack().expect("rulepack must load");
        assert!(pack_digest.0.starts_with("sha256:"));
        let steps = smoke_plan().expect("plan smoke must succeed");
        assert_eq!(steps, 3);
        let (_pin, rust_ir, frontend, hash, rulepack) = adapter_readiness();
        assert!(rust_ir);
        assert!(frontend);
        assert!(hash);
        assert!(rulepack);
    }
}
