//! Emitting a REAL package, not only measuring one.
//!
//! `survey` counts what a snapshot the engine has never seen would translate to. That number is
//! what ranks the work, and it is not what the engine is judged on: the bar is whether the emitted
//! Rust reads as hand-written, and a count cannot be read.
//!
//! This emits it. The result is a PARTIAL crate on purpose — the declarations that refused are not
//! in it, and the report says how many and why. A partial crate is what 70% coverage actually looks
//! like, and looking at it is the only way to find out whether the 70% is any good.
//!
//! Distinct from `port_go_source`, which runs the strict pipeline over the hermetic corpus and
//! REFUSES if anything in it fails. That strictness is right there: the corpus is the engine's own
//! and a refusal in it is a regression. It is wrong here, because a real package always has
//! something the engine cannot do yet, and refusing the whole package would make the engine
//! unable to show its work until it was finished.

use std::path::Path;

use port_engine_rust_ir::RustIr;
use port_engine_snapshot::admit_reproducible_pair;
use port_engine_transform::{SurveyReport, module_name, survey};

use crate::driver::report::PipelineError;

/// A real package, emitted as far as the engine can take it.
pub struct PortedPackage {
    /// What the survey found, so a reader knows what is NOT in the source below.
    pub report: SurveyReport,
    /// The emitted Rust, one module per unit.
    pub source: String,
    /// Regions the transform built and the renderer would not take, with what it said.
    ///
    /// A refusal discovered LATE. The transform counted these as translated, and they are not —
    /// which is exactly the kind of overcount this command exists to expose.
    pub unrenderable: Vec<String>,
}

/// Port a snapshot the engine has never seen, emitting what translates.
///
/// # Errors
/// [`PipelineError`] on an unreadable or inadmissible snapshot, or an emit that will not render.
pub fn port_snapshot(path: &Path) -> Result<PortedPackage, PipelineError> {
    let bytes = std::fs::read(path).map_err(|err| {
        PipelineError::Emit(port_engine_api::PortError::Render {
            detail: format!("read snapshot {}: {err}", path.display()),
        })
    })?;
    // Admitted against ITSELF, exactly as the survey does: one artifact from one extraction, so
    // the reproducibility pair has one member and the digest check is over the encoder.
    let admitted = admit_reproducible_pair(&bytes, &bytes).map_err(PipelineError::Admit)?;
    let pack =
        port_engine_rulepack::LoadedRulePack::load_embedded_go_rust().map_err(PipelineError::Rulepack)?;
    let report = survey(admitted.as_model(), &pack);

    // Source order within a unit, exactly as the strict pipeline emits: the position each
    // declaration had in its own package is the order a reader should meet them in.
    let mut regions: Vec<&port_engine_transform::PortedRegion> = report.ported.iter().collect();
    regions.sort_by(|a, b| (&a.unit.0, a.position).cmp(&(&b.unit.0, b.position)));

    // ONE REGION AT A TIME, because a region that will not render is a refusal discovered late and
    // this is a partial port. Rendering the package as one tree makes a single bad region take the
    // whole package with it — which is what happened, and what hid every other region in it.
    let renderer = port_engine_rust_ir::RustRenderer;
    let mut emitted = std::collections::BTreeMap::new();
    let mut unrenderable = Vec::new();
    for region in &regions {
        let mut ir = RustIr::new(&[region.region.as_str()]);
        let built = ir
            .set_items(&region.region, region.items.clone())
            .map_err(PipelineError::Emit)
            .and_then(|()| renderer.render_rust_ir(&ir).map_err(PipelineError::Emit));
        match built {
            Ok(rendered) => emitted.extend(rendered),
            Err(error) => unrenderable.push(format!("{}: {error}", region.region)),
        }
    }
    let mut source = String::from(
        "// PARTIAL. Emitted by port-engine from a package it had never seen; the declarations it\n\
         // could not translate are absent, and the survey says which and why.\n",
    );
    let mut current = String::new();
    for region in &regions {
        let module = module_name(&region.unit.0);
        if module != current {
            if !current.is_empty() {
                source.push_str("}\n");
            }
            source.push_str(&format!("pub mod {module} {{\n"));
            current = module;
        }
        let Some(bytes) = emitted.get(&port_engine_api::RegionId(region.region.clone())) else {
            continue;
        };
        for line in String::from_utf8_lossy(bytes).lines() {
            match line.is_empty() {
                true => source.push('\n'),
                false => source.push_str(&format!("    {line}\n")),
            }
        }
    }
    if !current.is_empty() {
        source.push_str("}\n");
    }
    Ok(PortedPackage {
        report,
        source,
        unrenderable,
    })
}
