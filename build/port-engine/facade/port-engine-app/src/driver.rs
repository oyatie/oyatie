//! Facade driver wiring: composes kernel entry points with W0-B adapters.
//!
//! Slice 14 wires canary materialize round-trip + planted-defect detect (no bulk `k8s/`).
//! Toolchain receipts bind the canonical `build/toolchains` cell through hermetic mirrors.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use port_engine_api::{
    Digest, Receipt, RegionId, RulePack, SourceModel, TargetIr, UnitId, w0_ready as api_ready,
};
use port_engine_emit::{
    emit_canary_checked, materialize_canary_roundtrip, select_canary, CanaryArtifact, EmitError,
    w0_ready as emit_ready,
};
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
use port_engine_transform::{apply, TransformError, w0_ready as transform_ready};

use crate::receipt_codec::{emit_tree_digest, format_receipt, matches_golden};

/// Slice 14 readiness: prior adapters + canary emit / materialize seams.
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
        && transform_ready()
        && emit_ready()
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

/// Load embedded fixture-gated rulepack v0; return digest + selecting-fixture count.
///
/// # Errors
/// Propagates [`RulepackError`] from the rulepack loader.
pub fn smoke_rulepack() -> Result<(Digest, usize), RulepackError> {
    let pack = LoadedRulePack::load_embedded()?;
    Ok((pack.digest(), pack.selecting_fixture_count()))
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

/// Outcome of the pin→admit→plan→transform→emit→receipt pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipelineReport {
    /// Bound six-axis receipt.
    pub receipt: Receipt,
    /// Kernel plan step count.
    pub plan_steps: usize,
    /// Emitted region count from syn/quote path.
    pub emit_regions: usize,
    /// Emitted region tree (for verify/delta / determinism).
    pub emitted: BTreeMap<RegionId, Vec<u8>>,
    /// Content digest of [`Self::emitted`].
    pub emit_digest: Digest,
}

/// Typed refusal from the Slice 11 pipeline.
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
    /// Construction/precondition transform refused.
    Transform(TransformError),
    /// Syn/quote emit refused.
    Emit(port_engine_api::PortError),
    /// Canary single-fixture emit refused.
    Canary(EmitError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pin(err) => write!(f, "pipeline pin: {err}"),
            Self::Admit(err) => write!(f, "pipeline admit: {err}"),
            Self::Rulepack(err) => write!(f, "pipeline rulepack: {err}"),
            Self::Plan(err) => write!(f, "pipeline plan: {err}"),
            Self::Transform(err) => write!(f, "pipeline transform: {err}"),
            Self::Emit(err) => write!(f, "pipeline emit: {err}"),
            Self::Canary(err) => write!(f, "pipeline canary: {err}"),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Admit → plan → transform → RustIr (Slice 11 smoke without emit/receipt).
///
/// # Errors
/// [`PipelineError`] on admit/rulepack/plan/transform refusal.
pub fn smoke_transform() -> Result<usize, PipelineError> {
    let admitted = smoke_admit_snapshot().map_err(PipelineError::Admit)?;
    let pack = LoadedRulePack::load_embedded().map_err(PipelineError::Rulepack)?;
    let plan = port_engine_kernel::plan(admitted.as_model(), &pack).map_err(PipelineError::Plan)?;
    let ir = apply(&plan, &pack, admitted.as_model()).map_err(PipelineError::Transform)?;
    Ok(ir.regions().len())
}

/// Run pin → admit → plan → transform → syn emit → six-axis receipt.
///
/// # Errors
/// [`PipelineError`] on any stage refusal.
pub fn smoke_pipeline() -> Result<PipelineReport, PipelineError> {
    let pin = fleet_pin().map_err(PipelineError::Pin)?;
    let admitted = smoke_admit_snapshot().map_err(PipelineError::Admit)?;
    let pack = LoadedRulePack::load_embedded().map_err(PipelineError::Rulepack)?;
    let plan = port_engine_kernel::plan(admitted.as_model(), &pack).map_err(PipelineError::Plan)?;
    let ir = apply(&plan, &pack, admitted.as_model()).map_err(PipelineError::Transform)?;

    let formatter_label = "fmt-pipeline-transform-v0";
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
        plan_steps: plan.steps.len(),
        emit_regions: emitted.len(),
        emit_digest: emit_tree_digest(&emitted),
        emitted,
        receipt,
    })
}

/// Transform + syn emit (Slice 12 `render` entrypoint).
///
/// # Errors
/// [`PipelineError`] on admit/rulepack/plan/transform/emit refusal.
pub fn smoke_render() -> Result<(usize, Digest), PipelineError> {
    let report = smoke_pipeline()?;
    Ok((report.emit_regions, report.emit_digest))
}

/// Re-run the pipeline twice and classify with kernel [`verify`](port_engine_kernel::verify).
///
/// Identical six axes + identical emit bytes → `Unchanged` / Green (W0-B Slice 6 acceptance).
///
/// # Errors
/// [`PipelineError`] on pipeline failure, or unexpected verdict.
pub fn smoke_delta() -> Result<port_engine_kernel::Verification, PipelineError> {
    let previous = smoke_pipeline()?;
    let current = smoke_pipeline()?;
    let verification = port_engine_kernel::verify(
        &previous.receipt,
        &previous.emitted,
        &current.receipt,
        &current.emitted,
    );
    if verification.verdict != port_engine_kernel::Verdict::Green
        || !matches!(
            verification.delta,
            port_engine_kernel::Delta::Unchanged
        )
    {
        return Err(PipelineError::Emit(port_engine_api::PortError::Render {
            detail: format!(
                "deterministic re-run expected Unchanged/Green, got {:?}",
                verification
            ),
        }));
    }
    if previous.receipt != current.receipt || previous.emit_digest != current.emit_digest {
        return Err(PipelineError::Emit(port_engine_api::PortError::Render {
            detail: "deterministic re-run receipt/emit digests diverged".into(),
        }));
    }
    Ok(verification)
}

/// Pipeline receipt must match the embedded golden (fail closed).
///
/// # Errors
/// [`PipelineError`] on pipeline failure or golden mismatch.
pub fn smoke_receipt_golden() -> Result<String, PipelineError> {
    let report = smoke_pipeline()?;
    let text = format_receipt(&report.receipt);
    if !matches_golden(&report.receipt) {
        return Err(PipelineError::Emit(port_engine_api::PortError::Render {
            detail: format!("receipt golden mismatch:\n{text}"),
        }));
    }
    Ok(text)
}

/// Select the single canary region from a pipeline emit tree and check its golden.
///
/// # Errors
/// [`PipelineError`] on pipeline or canary emit refusal.
pub fn smoke_emit_canary() -> Result<CanaryArtifact, PipelineError> {
    let report = smoke_pipeline()?;
    emit_canary_checked(&report.emitted).map_err(PipelineError::Canary)
}

/// Pipeline → canary golden → materialize single file → read-back round-trip.
///
/// `out_dir` basename must be [`port_engine_emit::CANARY_OUT_DIRNAME`]; never `k8s/`.
///
/// # Errors
/// [`PipelineError`] on pipeline, canary, path, or round-trip refusal.
pub fn smoke_materialize_canary(out_dir: &Path) -> Result<(CanaryArtifact, PathBuf), PipelineError> {
    let artifact = smoke_emit_canary()?;
    let dest = materialize_canary_roundtrip(out_dir, &artifact).map_err(PipelineError::Canary)?;
    Ok((artifact, dest))
}

/// Plant a byte defect in the canary region; expect kernel Red / Unexplained.
///
/// Proves the single-fixture canary is on the verify surface without bulk corpus emit.
///
/// # Errors
/// [`PipelineError`] when selection fails or the planted defect is not detected as Unexplained.
pub fn smoke_canary_planted_defect() -> Result<port_engine_kernel::Verification, PipelineError> {
    let previous = smoke_pipeline()?;
    let canary = select_canary(&previous.emitted).map_err(PipelineError::Canary)?;
    let mut current_emitted = previous.emitted.clone();
    current_emitted.insert(
        canary.region.clone(),
        b"pub fn planted_canary_defect () { }".to_vec(),
    );
    let verification = port_engine_kernel::verify(
        &previous.receipt,
        &previous.emitted,
        &previous.receipt,
        &current_emitted,
    );
    let expected_red = matches!(
        (&verification.verdict, &verification.delta),
        (
            port_engine_kernel::Verdict::Red,
            port_engine_kernel::Delta::Unexplained { regions }
        ) if regions.contains(&canary.region)
    );
    if !expected_red {
        return Err(PipelineError::Emit(port_engine_api::PortError::Render {
            detail: format!(
                "planted canary defect expected Red/Unexplained containing `{}`, got {:?}",
                canary.region.0, verification
            ),
        }));
    }
    Ok(verification)
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
/// Order: `(pin, rust_ir, frontend, hash, rulepack, snapshot, identity, toolchain, transform, emit)`.
#[must_use]
pub fn adapter_readiness() -> (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) {
    (
        port_engine_source_pin::w0_ready(),
        port_engine_rust_ir::w0_ready(),
        frontend_ready(),
        hash_ready(),
        rulepack_ready(),
        snapshot_ready(),
        identity_ready(),
        toolchain_ready(),
        transform_ready(),
        emit_ready(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice14_driver_wiring_is_ready() {
        use std::time::{SystemTime, UNIX_EPOCH};

        assert!(w0_ready());
        fleet_pin().expect("fleet pin must load");
        smoke_render_stub().expect("empty renderer stub must emit");
        smoke_syn_quote_render().expect("syn/quote path must emit");
        let d = smoke_digest("port-engine");
        assert!(d.0.starts_with("sha256:"));
        let (pack_digest, fixtures) = smoke_rulepack().expect("rulepack must load");
        assert!(pack_digest.0.starts_with("sha256:"));
        assert!(fixtures >= 2);
        let steps = smoke_plan().expect("plan smoke must succeed");
        assert_eq!(steps, 3);
        let admitted = smoke_admit_snapshot().expect("snapshot fixture must admit");
        assert!(admitted.snapshot_digest.0.starts_with("sha256:"));
        let eng = smoke_engine_digest();
        assert!(eng.0.starts_with("sha256:"));
        let tc = smoke_toolchain_digest();
        assert_eq!(
            tc.0,
            "sha256:13738b998e63359c3b3294f5e7c6de003649ad13cad4da4c667f576549ef5f44"
        );
        let regions = smoke_transform().expect("transform must succeed");
        assert_eq!(regions, 3);
        let report = smoke_pipeline().expect("pipeline must succeed");
        assert_eq!(report.plan_steps, 3);
        assert_eq!(report.emit_regions, 3);
        assert!(report.receipt.incomplete_axes().is_empty());
        assert_eq!(report.receipt.engine_digest, eng);
        assert_eq!(report.receipt.toolchain_digest, tc);
        assert!(matches_golden(&report.receipt));
        let golden = smoke_receipt_golden().expect("golden receipt");
        assert!(golden.contains("snapshot_digest=sha256:"));
        let (render_regions, render_digest) = smoke_render().expect("render");
        assert_eq!(render_regions, 3);
        assert_eq!(render_digest, report.emit_digest);
        let verification = smoke_delta().expect("delta re-run");
        assert_eq!(verification.verdict, port_engine_kernel::Verdict::Green);
        let canary = smoke_emit_canary().expect("canary emit");
        assert!(canary.region.0.ends_with("__canary_empty_unit"));
        let planted = smoke_canary_planted_defect().expect("planted defect");
        assert_eq!(planted.verdict, port_engine_kernel::Verdict::Red);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let out = std::env::temp_dir()
            .join(format!("pe-facade-canary-{nanos}"))
            .join(port_engine_emit::CANARY_OUT_DIRNAME);
        let (art, dest) = smoke_materialize_canary(&out).expect("materialize");
        assert_eq!(art.digest, canary.digest);
        assert_eq!(
            dest.file_name().and_then(|s| s.to_str()),
            Some(port_engine_emit::CANARY_FILENAME)
        );
        let _ = std::fs::remove_dir_all(out.parent().expect("parent"));
        let (
            _pin,
            rust_ir,
            frontend,
            hash,
            rulepack,
            snapshot,
            identity,
            toolchain,
            transform,
            emit,
        ) = adapter_readiness();
        assert!(
            rust_ir
                && frontend
                && hash
                && rulepack
                && snapshot
                && identity
                && toolchain
                && transform
                && emit
        );
    }
}
