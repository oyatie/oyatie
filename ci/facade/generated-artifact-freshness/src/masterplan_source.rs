//! Filesystem adapter for the controller-owned masterplan projection core.
//!
//! The planning-projection crate remains pure. This adapter owns deterministic discovery and reads
//! of the declared ADR source corpus; callers own output placement.

use std::fs;
use std::path::Path;

use ci_planning_projection::{
    MasterplanProjection, PlanningAdr, parse_planning_impact_adr, render_masterplan_projection,
};

/// Read all canonical planning-impact ADRs in deterministic path order.
///
/// # Errors
///
/// Returns an error if the decisions directory or any selected ADR cannot be read.
pub fn read_planning_impact_adrs(decisions_dir: &Path) -> Result<Vec<PlanningAdr>, String> {
    let entries = fs::read_dir(decisions_dir).map_err(|error| {
        format!(
            "ADR decisions dir unreadable {}: {error}",
            decisions_dir.display()
        )
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("ADR decisions dir entry unreadable: {error}"))?;
        let path = entry.path();
        let is_adr = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name.starts_with("ADR-") && name.ends_with(".md") && !name.contains("-amendment-")
            });
        if is_adr {
            paths.push(path);
        }
    }
    paths.sort();

    let mut adrs = Vec::new();
    for path in paths {
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("ADR unreadable {}: {error}", path.display()))?;
        if let Some(adr) = parse_planning_impact_adr(&path, &contents) {
            adrs.push(adr);
        }
    }
    Ok(adrs)
}

/// Render the controller-owned masterplan projection from its canonical decisions directory.
///
/// This function performs no writes and is suitable for controller materialization or isolated
/// Cargo test output.
///
/// # Errors
///
/// Returns an error if source discovery, source reads, or projection serialization fails.
pub fn render_masterplan_projection_from_decisions(
    decisions_dir: &Path,
) -> Result<(MasterplanProjection, String), String> {
    let adrs = read_planning_impact_adrs(decisions_dir)?;
    render_masterplan_projection(&adrs)
}
