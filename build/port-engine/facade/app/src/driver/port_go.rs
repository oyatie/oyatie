//! Porting the hermetic Go corpus, and assembling the result into compilable modules.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::engine::engine_digest;
use port_engine_api::{
    Declaration, Digest, Receipt, RegionId, Renderer, RulePack, SourceModel, TargetIr, UnitId,
};
use port_engine_emit::{
    CanaryArtifact, EmitError, emit_canary_checked, materialize_canary_roundtrip, select_canary,
};
use port_engine_hash::digest_str;
use port_engine_rulepack::{LoadedRulePack, RulepackError};
use port_engine_rust_ir::{EmptyRenderer, RustFn, RustIr, RustItem, RustRenderer, Visibility};
use port_engine_snapshot::{
    AdmitError, AdmittedSnapshot, admit_embedded_fixture, admit_embedded_fixture_drift_after_v1,
    admit_embedded_fixture_drift_before_v1, admit_embedded_fixture_failure_v1,
    admit_embedded_fixture_interface_v1, admit_embedded_fixture_ownership_v1,
    admit_embedded_fixture_unproven_v1,
    admit_embedded_fixture_refused_v1, admit_embedded_fixture_v1,
};
use port_engine_source_pin::{load_embedded, receipt_pin};
use port_engine_toolchain::toolchain_digest;
use port_engine_transform::{TransformError, apply, apply_with_provenance, module_name};

use crate::receipt_codec::{emit_tree_digest, format_receipt, matches_golden};

/// Committed golden for the assembled Go port.
const PORT_GO_GOLDEN: &str = include_str!("../port-go-golden-v1.txt");

use crate::driver::report::{PipelineError, PipelineReport};
use crate::driver::smoke::smoke_admit_snapshot;

pub fn port_go_pipeline() -> Result<PipelineReport, PipelineError> {
    port_go_from(admit_embedded_fixture_v1().map_err(PipelineError::Admit)?)
}

/// Port the EARLIER version of the upstream-drift pair.
///
/// # Errors
/// [`PipelineError`] on any pipeline defect.
pub fn port_go_drift_before() -> Result<PipelineReport, PipelineError> {
    port_go_from(admit_embedded_fixture_drift_before_v1().map_err(PipelineError::Admit)?)
}

/// Port the LATER version of the upstream-drift pair.
///
/// # Errors
/// [`PipelineError`] on any pipeline defect.
pub fn port_go_drift_after() -> Result<PipelineReport, PipelineError> {
    port_go_from(admit_embedded_fixture_drift_after_v1().map_err(PipelineError::Admit)?)
}

/// The pipeline, over whichever admitted snapshot it is handed.
///
/// Parameterised by the SNAPSHOT and by nothing else, which is what makes a re-port comparable: the
/// engine, the rules, the toolchain and the formatter are the same run of the same code, so an
/// emitted difference between two calls can only have come from the source.
///
/// # Errors
/// [`PipelineError`] on any pipeline defect.
pub fn port_go_from(admitted: AdmittedSnapshot) -> Result<PipelineReport, PipelineError> {
    let pin = admitted.pin().to_owned();
    let pack = LoadedRulePack::load_embedded_go_rust().map_err(PipelineError::Rulepack)?;
    let plan = port_engine_kernel::plan(admitted.as_model(), &pack).map_err(PipelineError::Plan)?;
    let transformed = apply_with_provenance(&plan, &pack, admitted.as_model())
        .map_err(PipelineError::Transform)?;

    let renderer = RustRenderer::new();
    let emitted = renderer
        .render_rust_ir(&transformed.ir)
        .map_err(PipelineError::Emit)?;

    let receipt = Receipt {
        pin,
        snapshot_digest: admitted.artifact_digest().clone(),
        engine_digest: engine_digest(),
        rulepack_digest: pack.digest(),
        toolchain_digest: toolchain_digest(),
        // The renderer reports its own identity and version; the digest is taken of THAT
        // rather than of a label, so the axis moves when the formatter does.
        formatter_digest: digest_str(&renderer.formatter_digest().0),
    };
    if !receipt.incomplete_axes().is_empty() {
        return Err(PipelineError::Emit(port_engine_api::PortError::Render {
            detail: format!(
                "port-go receipt incomplete axes: {:?}",
                receipt.incomplete_axes()
            ),
        }));
    }

    Ok(PipelineReport {
        plan_steps: plan.steps.len(),
        emit_regions: emitted.len(),
        emit_digest: emit_tree_digest(&emitted),
        region_units: transformed.region_units,
        dispositions: transformed.dispositions,
        emitted,
        receipt,
    })
}

/// Attempt to port the refusal corpus, returning the refusal.
///
/// # Errors
/// [`PipelineError`] — and a `Transform` refusal is the SUCCESSFUL outcome for this input, which
/// is why the caller inspects the error rather than treating it as a failure.
pub fn port_go_refused() -> Result<usize, PipelineError> {
    refuse(admit_embedded_fixture_refused_v1().map_err(PipelineError::Admit)?)
}

/// Attempt to port the ownership-refusal corpus, returning the refusal.
///
/// # Errors
/// [`PipelineError`] — a `Transform` refusal is the SUCCESSFUL outcome for this input.
pub fn port_go_refused_ownership() -> Result<usize, PipelineError> {
    refuse(admit_embedded_fixture_ownership_v1().map_err(PipelineError::Admit)?)
}

/// Attempt to port the interface-position refusal corpus, returning the refusal.
///
/// # Errors
/// [`PipelineError`] — a `Transform` refusal is the SUCCESSFUL outcome for this input.
pub fn port_go_refused_interface() -> Result<usize, PipelineError> {
    refuse(admit_embedded_fixture_interface_v1().map_err(PipelineError::Admit)?)
}

/// Attempt to port the unproven-failure corpus, returning the refusal.
///
/// # Errors
/// [`PipelineError`] — a `Transform` refusal is the SUCCESSFUL outcome for this input.
pub fn port_go_refused_unproven() -> Result<usize, PipelineError> {
    refuse(admit_embedded_fixture_unproven_v1().map_err(PipelineError::Admit)?)
}

/// Port the failure-convention corpus.
///
/// Named for the CONVENTION rather than for a refusal: what this corpus proves moved when the pack
/// decided how far to trust the source's rule that a result beside a non-nil error is not
/// guaranteed to be meaningful. It used to refuse; it now discards the companion.
///
/// # Errors
/// [`PipelineError`] on any pipeline defect.
pub fn port_go_failure_pipeline() -> Result<PipelineReport, PipelineError> {
    port_go_from(admit_embedded_fixture_failure_v1().map_err(PipelineError::Admit)?)
}

/// Port the failure corpus, reporting only how many regions it produced.
///
/// # Errors
/// [`PipelineError`] on any pipeline defect.
pub fn port_go_refused_failure() -> Result<usize, PipelineError> {
    refuse(admit_embedded_fixture_failure_v1().map_err(PipelineError::Admit)?)
}

fn refuse(admitted: AdmittedSnapshot) -> Result<usize, PipelineError> {
    let pack = LoadedRulePack::load_embedded_go_rust().map_err(PipelineError::Rulepack)?;
    let plan = port_engine_kernel::plan(admitted.as_model(), &pack).map_err(PipelineError::Plan)?;
    let ir = apply(&plan, &pack, admitted.as_model()).map_err(PipelineError::Transform)?;
    Ok(ir.regions().len())
}

/// Assemble an emitted tree into one compilable Rust source: a module per source unit.
///
/// Regions are grouped by the unit the TRANSFORM reported, never by parsing the region id — see
/// [`port_engine_transform::apply_with_provenance`] for why a sanitized id cannot be un-sanitized.
///
/// A module per unit is not cosmetic. The corpus's declarations reference each other within a unit
/// (a method returns its own struct; a function takes a locally aliased type), so they must share a
/// scope; and two units may each declare a `Point`, so they must not share one. Flattening would
/// compile today's fixture and collide on the second corpus that has a repeated name.
#[must_use]
pub fn assemble_modules(report: &PipelineReport) -> String {
    let mut by_unit: BTreeMap<String, Vec<&RegionId>> = BTreeMap::new();
    for region in report.emitted.keys() {
        let module = report
            .region_units
            .get(region)
            .map_or_else(|| "unattributed".to_owned(), |unit| module_name(&unit.0));
        by_unit.entry(module).or_default().push(region);
    }

    let mut out = String::new();
    out.push_str("// GENERATED by port-engine. Do not edit — regenerate from the rule pack.\n");
    for (module, regions) in by_unit {
        out.push_str(&format!("pub mod {module} {{\n"));
        for region in regions {
            let bytes = report.emitted.get(region).map_or(&[][..], Vec::as_slice);
            out.push_str(&format!("    // region: {}\n", region.0));
            // Indent EVERY line, not only the first. The formatter emits a multi-line item, and a
            // single leading pad left its body sitting at module indentation — legal Rust, and a
            // golden that reads like a formatting bug on every future change.
            for line in String::from_utf8_lossy(bytes).lines() {
                if line.is_empty() {
                    out.push('\n');
                } else {
                    out.push_str(&format!("    {line}\n"));
                }
            }
        }
        out.push_str("}\n");
    }
    out
}

/// Assemble the ported corpus and fail closed against the committed golden.
///
/// The golden is not a substitute for the compile proof — it cannot tell correct Rust from
/// incorrect. What it is for is REVIEW: a rule change, a type-map edit, or a corpus change shows
/// up here as a diff in emitted source, so the effect of a data change is visible in the same
/// pull request as the data change.
///
/// Returns the assembled source and whether it matches the golden. The source comes back EITHER
/// WAY, deliberately: refreshing the golden is `port-go-source > src/port-go-golden-v1.txt`, and a
/// command that refused to print the new bytes would make the only way to update the golden be to
/// hand-transcribe them from an error message.
///
/// # Errors
/// [`PipelineError`] on pipeline failure.
pub fn port_go_source() -> Result<(String, bool), PipelineError> {
    let report = port_go_pipeline()?;
    let source = assemble_modules(&report);
    let matches = source == PORT_GO_GOLDEN;
    Ok((source, matches))
}

/// The ownership record, as a reviewable artifact.
///
/// An ownership disposition is an inference over facts the reader cannot see from the emitted
/// code: `&mut self` looks identical whether it was proven or assumed. So the reasoning is a
/// SEPARATE artifact rather than a comment in the output — diffable on its own, and out of the
/// place where a rule change is hardest to review.
///
/// `unproven` marks a decision made on facts the front end could not establish. It is not an error
/// and it is not hidden: it is the difference between "safe as far as anyone looked" and "safe as
/// far as anyone looked, and nobody looked past the first call".
///
/// # Errors
/// [`PipelineError`] on pipeline failure.
pub fn port_go_dispositions() -> Result<String, PipelineError> {
    let report = port_go_pipeline()?;
    let mut out = String::new();
    out.push_str("# Ownership dispositions. Generated — regenerate rather than edit.\n");
    out.push_str("# site | rule | form | proven\n");
    for record in &report.dispositions {
        out.push_str(&format!(
            "{} | {} | {} | {}\n",
            record.site,
            record.rule_id,
            record.form,
            if record.unproven {
                "UNPROVEN"
            } else {
                "proven"
            },
        ));
    }
    Ok(out)
}

/// Re-run the Go port twice and classify with kernel `verify`.
///
/// # Errors
/// [`PipelineError`] on pipeline failure, or when the two runs are not identical.
pub fn port_go_delta() -> Result<port_engine_kernel::Verification, PipelineError> {
    let previous = port_go_pipeline()?;
    let current = port_go_pipeline()?;
    let verification = port_engine_kernel::verify(
        &previous.receipt,
        &previous.emitted,
        &current.receipt,
        &current.emitted,
    );
    if verification.verdict != port_engine_kernel::Verdict::Green
        || !matches!(verification.delta, port_engine_kernel::Delta::Unchanged)
    {
        return Err(PipelineError::Emit(port_engine_api::PortError::Render {
            detail: format!("port-go re-run expected Unchanged/Green, got {verification:?}"),
        }));
    }
    Ok(verification)
}
