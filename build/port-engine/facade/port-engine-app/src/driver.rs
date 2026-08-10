//! Facade driver wiring: composes kernel entry points with W0-B adapters.
//!
//! Slice 9 wires engine identity + dual-home toolchain digests and a pin→admit→plan→emit→receipt
//! pipeline. Cell remap (`.buckconfig`) remains PARKED; toolchain axis digests dual-home bytes.

use port_engine_api::{Digest, Receipt, RulePack, SourceModel, UnitId, w0_ready as api_ready};
use port_engine_frontend_go::w0_ready as frontend_ready;
use port_engine_hash::{digest_str, w0_ready as hash_ready};
use port_engine_identity::{engine_digest, w0_ready as identity_ready};
use port_engine_rulepack::{LoadedRulePack, RulepackError, w0_ready as rulepack_ready};
use port_engine_rust_ir::{EmptyRenderer, RustIr, SynQuoteRenderer};
use port_engine_snapshot::{
    admit_embedded_fixture, AdmittedSnapshot, AdmitError, w0_ready as snapshot_ready,
};
use port_engine_source_pin::{load_embedded, receipt_pin};
use port_engine_toolchain::{toolchain_digest, w0_ready as toolchain_ready};

/// Slice 9 readiness: prior adapters + engine identity + toolchain axis digests.
pub const fn w0_ready() -> bool {
    api_ready()
        && port_engine_source_pin::w0_ready()
        && port_engine_rust_ir::w0_ready()
        && frontend_ready()
        && hash_ready()
        && rulepack_ready()
        && snapshot_ready()
        && identity_ready()
        && toolchain_ready()
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

/// Admit the hermetic OOB bootstrap snapshot fixture (Slice 8).
///
/// # Errors
/// Propagates [`AdmitError`] from snapshot admission.
pub fn smoke_admit_snapshot() -> Result<AdmittedSnapshot, AdmitError> {
    admit_embedded_fixture()
}

/// Slice 9 engine identity digest.
#[must_use]
pub fn smoke_engine_digest() -> Digest {
    engine_digest()
}

/// Slice 9 dual-home toolchain corpus digest.
#[must_use]
pub fn smoke_toolchain_digest() -> Digest {
    toolchain_digest()
}

/// Outcome of the Slice 9 pin→admit→plan→emit→receipt pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipelineReport {
    /// Bound six-axis receipt.
    pub receipt: Receipt,
    /// Kernel plan step count.
    pub plan_steps: usize,
    /// Emitted region count from syn/quote path.
    pub emit_regions: usize,
}

/// Typed refusal from the Slice 9 pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PipelineError {
    /// Fleet pin could not load.
    Pin(port_engine_source_pin::PinError),
    /// Snapshot admission refused.
    Admit(AdmitError),
    /// Rulepack load failed.
    Rulepack(RulepackError),
    /// Kernel plan refused.
    Plan(port_engine_api::PortError),
    /// Syn/quote emit refused.
    Emit(port_engine_api::PortError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pin(err) => write!(f, "pipeline pin: {err}"),
            Self::Admit(err) => write!(f, "pipeline admit: {err}"),
            Self::Rulepack(err) => write!(f, "pipeline rulepack: {err}"),
            Self::Plan(err) => write!(f, "pipeline plan: {err}"),
            Self::Emit(err) => write!(f, "pipeline emit: {err}"),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Run pin → admit → plan → syn emit → six-axis receipt (all digests content-addressed).
///
/// # Errors
/// [`PipelineError`] on any stage refusal.
pub fn smoke_pipeline() -> Result<PipelineReport, PipelineError> {
    let pin = fleet_pin().map_err(PipelineError::Pin)?;
    let admitted = smoke_admit_snapshot().map_err(PipelineError::Admit)?;
    let pack = LoadedRulePack::load_embedded().map_err(PipelineError::Rulepack)?;
    let plan = port_engine_kernel::plan(admitted.as_model(), &pack).map_err(PipelineError::Plan)?;

    let mut ir = RustIr::new(&["stub"]);
    ir.set_file_from_str("stub", "pub fn stub() {}")
        .map_err(PipelineError::Emit)?;
    let formatter_label = "fmt-pipeline-syn-v0";
    let renderer = SynQuoteRenderer::new(formatter_label);
    let emitted = renderer
        .render_rust_ir(&ir)
        .map_err(PipelineError::Emit)?;

    let receipt = Receipt {
        pin,
        snapshot_digest: admitted.snapshot_digest.clone(),
        engine_digest: engine_digest(),
        rulepack_digest: pack.digest(),
        toolchain_digest: toolchain_digest(),
        formatter_digest: digest_str(formatter_label),
    };
    if !receipt.incomplete_axes().is_empty() {
        return Err(PipelineError::Emit(port_engine_api::PortError::Render {
            detail: format!(
                "pipeline receipt incomplete axes: {:?}",
                receipt.incomplete_axes()
            ),
        }));
    }

    Ok(PipelineReport {
        receipt,
        plan_steps: plan.steps.len(),
        emit_regions: emitted.len(),
    })
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
/// Order: `(pin, rust_ir, frontend, hash, rulepack, snapshot, identity, toolchain)`.
#[must_use]
pub fn adapter_readiness() -> (bool, bool, bool, bool, bool, bool, bool, bool) {
    (
        port_engine_source_pin::w0_ready(),
        port_engine_rust_ir::w0_ready(),
        frontend_ready(),
        hash_ready(),
        rulepack_ready(),
        snapshot_ready(),
        identity_ready(),
        toolchain_ready(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice9_driver_wiring_is_ready() {
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
        let admitted = smoke_admit_snapshot().expect("snapshot fixture must admit");
        assert!(admitted.snapshot_digest.0.starts_with("sha256:"));
        let eng = smoke_engine_digest();
        assert!(eng.0.starts_with("sha256:"));
        let tc = smoke_toolchain_digest();
        assert_eq!(
            tc.0,
            "sha256:419e00d0e9c4d25f07431224dc50f89083d772adb9c59751a9a7d78c28f01cbd"
        );
        let report = smoke_pipeline().expect("pipeline must succeed");
        assert_eq!(report.plan_steps, 3);
        assert_eq!(report.emit_regions, 1);
        assert!(report.receipt.incomplete_axes().is_empty());
        assert_eq!(report.receipt.engine_digest, eng);
        assert_eq!(report.receipt.toolchain_digest, tc);
        let (_pin, rust_ir, frontend, hash, rulepack, snapshot, identity, toolchain) =
            adapter_readiness();
        assert!(rust_ir && frontend && hash && rulepack && snapshot && identity && toolchain);
    }
}
