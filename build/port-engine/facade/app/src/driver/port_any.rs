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

use port_engine_api::{Renderer, RulePack};
use std::path::Path;

use port_engine_rust_ir::RustIr;
use port_engine_snapshot::admit_reproducible_pair;
use port_engine_transform::{SurveyReport, module_name, survey};

use crate::driver::report::PipelineError;

/// One emitted file, which is one unit of the source.
pub struct PortedFile {
    /// The module's name, which is the file's name and needs no `mod` block inside it.
    pub module: String,
    /// Its contents.
    pub source: String,
}

/// A real package, emitted as far as the engine can take it.
pub struct PortedPackage {
    /// What the survey found, so a reader knows what is NOT in the source below.
    pub report: SurveyReport,
    /// The emitted Rust, one FILE per unit — which is what a crate laid out this way is.
    pub files: Vec<PortedFile>,
    /// The six-axis receipt for this emission, so two ports of one package can be compared.
    ///
    /// A port with no receipt cannot answer the question the engine exists to answer: whether the
    /// output still corresponds to the upstream it was generated from. Carrying it here is what
    /// makes the drift classification available for a REAL package rather than only for a fixture.
    pub receipt: port_engine_api::Receipt,
    /// The emitted bytes per region, which the classifier compares.
    pub emitted: std::collections::BTreeMap<port_engine_api::RegionId, Vec<u8>>,
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
    port_snapshot_pinned(path, "unpinned")
}

/// The same, told which upstream revision the snapshot was taken at.
///
/// The PIN is an argument because a snapshot does not carry one: the front end records what the
/// source says, and the source does not know its own revision. A caller extracting a package knows
/// which commit they extracted, and the receipt is incomplete without it — an empty axis buys no
/// explanation, which is exactly the false Green the classifier refuses.
///
/// # Errors
/// [`PipelineError`] on an unreadable or inadmissible snapshot, or an emit that will not render.
pub fn port_snapshot_pinned(path: &Path, pin: &str) -> Result<PortedPackage, PipelineError> {
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
    // ONE FILE PER UNIT, which is what a crate laid out this way IS. Wrapping each unit in
    // `pub mod X { .. }` inside one file is the source's `package X` header transliterated into a
    // block, and a reviewer ranked it fourth among the reasons the output reads as translated: an
    // author writing `semver.rs` never opens it with `pub mod semver`. The module is the FILE.
    let mut files: Vec<PortedFile> = Vec::new();
    let mut current = String::new();
    for region in &regions {
        let module = module_name(&region.unit.0);
        if module != current {
            files.push(PortedFile {
                module: module.clone(),
                source: String::new(),
            });
            current = module;
        }
        let Some(bytes) = emitted.get(&port_engine_api::RegionId(region.region.clone())) else {
            continue;
        };
        let into = &mut files.last_mut().unwrap_or_else(|| unreachable!()).source;
        // A BLANK LINE between declarations. Each region is one declaration's output — a type and
        // the impls that belong to it — so a break between regions is a break between things a
        // reader meets separately, and a break inside one would split a type from its own impl.
        //
        // Not cosmetic, and not something the formatter will do: `prettyplease` preserves no blank
        // lines because the tree it renders has none, so eighty-seven lines of items arrive with
        // nothing between them. A reviewer reading a real ported package said that is not how
        // authored code is laid out and that the seven near-identical blocks were unscannable
        // because of it. They were right, and no rule about the CODE would ever have fixed it.
        // An EMPTY region contributes nothing, not even a break. A declaration whose output lives
        // in a unit-level item — every sentinel but the one carrying the grouped enum — renders to
        // nothing, and giving each of those a blank line put five of them in a row.
        let rendered = String::from_utf8_lossy(bytes);
        if rendered.trim().is_empty() {
            continue;
        }
        if !into.is_empty() {
            into.push('\n');
        }
        for line in rendered.lines() {
            into.push_str(line);
            into.push('\n');
        }
    }

    let receipt = port_engine_api::Receipt {

        pin: pin.to_owned(),

        snapshot_digest: admitted.artifact_digest().clone(),

        engine_digest: crate::engine::engine_digest(),

        rulepack_digest: pack.digest(),

        toolchain_digest: port_engine_toolchain::toolchain_digest(),

        formatter_digest: port_engine_hash::digest_str(&renderer.formatter_digest().0),

    };

    Ok(PortedPackage {
        receipt,
        emitted: emitted.clone(),
        report,
        files,
        unrenderable,
    })
}
