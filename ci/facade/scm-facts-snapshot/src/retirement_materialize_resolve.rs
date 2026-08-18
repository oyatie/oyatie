//! Resolve event identity, control plane, and receipt inventories.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::super::{
    CONTROL_PLANE_PATH, EquivalenceIndex, ReceiptStage, RetirementControlPlane,
    RetirementMaterializationContext, RetirementObjectSource, TreeEntry, build_equivalence_index,
    canonical_value_sha256, classify_stage, control_entry_value, entries_by_path,
    expected_receipt_paths, parse_closed_json, receipt_root_inventory, require_regular,
    sha256_digest, validate_control_plane, validate_event_identity, validate_oid,
    validate_predecessor_inputs, validate_receipt_population, validate_selector_coverage,
};

pub(super) struct ResolvedRetirement {
    pub(super) protected: String,
    pub(super) candidate: String,
    pub(super) subject: String,
    pub(super) protected_tree: String,
    pub(super) candidate_tree: String,
    pub(super) protected_entries: BTreeMap<String, TreeEntry>,
    pub(super) candidate_entries: BTreeMap<String, TreeEntry>,
    pub(super) predecessor_entries: BTreeMap<String, TreeEntry>,
    pub(super) predecessor: String,
    pub(super) predecessor_tree: String,
    pub(super) control_plane: RetirementControlPlane,
    pub(super) candidate_control_oid: String,
    pub(super) candidate_control_bytes: Vec<u8>,
    pub(super) protected_control_oid: Option<String>,
    pub(super) protected_control_bytes: Option<Vec<u8>>,
    pub(super) bootstrap: bool,
    pub(super) stages: Vec<ReceiptStage>,
    pub(super) active_receipt_population: bool,
    pub(super) equivalence_index: EquivalenceIndex,
    pub(super) protected_receipt_inventory: BTreeSet<String>,
    pub(super) candidate_receipt_inventory: BTreeSet<String>,
    pub(super) expected_receipt_paths: BTreeSet<String>,
    pub(super) unexpected_protected_receipt_paths: Vec<String>,
    pub(super) unexpected_candidate_receipt_paths: Vec<String>,
    pub(super) control_plane_entries: Vec<Value>,
    pub(super) control_plane_entry_hashes: Vec<Value>,
    pub(super) protected_control_sha: Option<String>,
    pub(super) candidate_control_sha: String,
}

pub(super) fn resolve_retirement_materialization(
    source: &impl RetirementObjectSource,
    context: &RetirementMaterializationContext<'_>,
) -> Result<ResolvedRetirement, String> {
    if context.control_plane_path != CONTROL_PLANE_PATH {
        return Err(format!(
            "retirement control-plane path must be {CONTROL_PLANE_PATH}"
        ));
    }

    for (label, requested) in [
        (
            "requested protected base commit",
            context.protected_base_commit,
        ),
        ("requested evaluated commit", context.evaluated_commit),
        ("requested subject commit", context.subject_commit),
    ] {
        validate_oid(requested, label)?;
    }
    let candidate = source.resolve_commit(context.evaluated_commit)?;
    let head = source.resolve_commit("HEAD")?;
    if candidate != head {
        return Err(format!(
            "retirement candidate {candidate} is not exact HEAD {head}"
        ));
    }
    let protected = source.resolve_commit(context.protected_base_commit)?;
    let subject = source.resolve_commit(context.subject_commit)?;
    if protected != context.protected_base_commit
        || candidate != context.evaluated_commit
        || subject != context.subject_commit
    {
        return Err(
            "requested retirement event identity must equal resolved commit identity".to_owned(),
        );
    }
    validate_event_identity(
        source,
        context.scm_event_name,
        context.scm_event_ref,
        context.scm_event_base_ref,
        &protected,
        &candidate,
        &subject,
    )?;
    let first_parent = source.first_parent(&candidate)?;
    if protected != first_parent {
        return Err(format!(
            "retirement protected base {protected} is not candidate first parent {first_parent}"
        ));
    }
    if !source.is_ancestor(&protected, &candidate)? {
        return Err("retirement protected base is not an ancestor of candidate".to_owned());
    }
    let protected_tree = source.tree_for_commit(&protected)?;
    let candidate_tree = source.tree_for_commit(&candidate)?;
    let protected_entries = entries_by_path(source.tree_entries(&protected)?)?;
    let candidate_entries = entries_by_path(source.tree_entries(&candidate)?)?;

    let candidate_control_entry = candidate_entries
        .get(CONTROL_PLANE_PATH)
        .ok_or_else(|| "candidate retirement control plane is absent".to_owned())?;
    require_regular(
        candidate_control_entry,
        "candidate retirement control plane",
    )?;
    let candidate_control_oid = candidate_control_entry.oid.clone();
    let candidate_control_bytes = source.read_blob(&candidate_control_oid)?;
    let control_plane: RetirementControlPlane = parse_closed_json(&candidate_control_bytes)?;
    validate_control_plane(&control_plane)?;

    let predecessor = source.resolve_commit(&control_plane.predecessor_snapshot.commit_oid)?;
    if predecessor != control_plane.predecessor_snapshot.commit_oid {
        return Err("retirement predecessor commit is not canonical".to_owned());
    }
    let predecessor_tree = source.tree_for_commit(&predecessor)?;
    if predecessor_tree != control_plane.predecessor_snapshot.tree_oid {
        return Err("retirement predecessor commit/tree binding does not match Git".to_owned());
    }
    if !source.is_ancestor(&predecessor, &protected)? {
        return Err("retirement predecessor is not an ancestor of protected base".to_owned());
    }
    let predecessor_entries = entries_by_path(source.tree_entries(&predecessor)?)?;
    let predecessor_input_bodies =
        validate_predecessor_inputs(source, &control_plane, &predecessor_entries)?;
    validate_selector_coverage(&control_plane, &predecessor_entries, "predecessor")?;
    validate_selector_coverage(&control_plane, &protected_entries, "protected")?;
    validate_selector_coverage(&control_plane, &candidate_entries, "candidate")?;

    let protected_control = protected_entries.get(CONTROL_PLANE_PATH);
    let bootstrap = protected_control.is_none();
    let protected_control_oid = protected_control.map(|entry| entry.oid.clone());
    let protected_control_bytes = match protected_control {
        None => None,
        Some(entry) => {
            require_regular(entry, "protected retirement control plane")?;
            let bytes = source.read_blob(&entry.oid)?;
            if bytes != candidate_control_bytes || entry.oid != candidate_control_oid {
                return Err(
                    "protected and candidate retirement control planes are not immutable-identical"
                        .to_owned(),
                );
            }
            Some(bytes)
        }
    };

    let stages = control_plane
        .entries
        .iter()
        .map(|entry| classify_stage(entry, &protected_entries, &candidate_entries))
        .collect::<Result<Vec<_>, _>>()?;
    let active_receipt_population = validate_receipt_population(&stages)?;
    if bootstrap && active_receipt_population {
        return Err("retirement bootstrap may not add receipts".to_owned());
    }
    let equivalence_index = active_receipt_population
        .then(|| {
            build_equivalence_index(
                source,
                &predecessor_input_bodies,
                &protected_entries,
                &candidate_entries,
            )
        })
        .transpose()?
        .unwrap_or_default();

    let protected_receipt_inventory = receipt_root_inventory(&protected_entries);
    let candidate_receipt_inventory = receipt_root_inventory(&candidate_entries);
    let expected_receipt_paths = expected_receipt_paths(&control_plane);
    let unexpected_protected_receipt_paths = protected_receipt_inventory
        .difference(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_candidate_receipt_paths = candidate_receipt_inventory
        .difference(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();

    let control_plane_entries = control_plane
        .entries
        .iter()
        .map(control_entry_value)
        .collect::<Result<Vec<_>, _>>()?;
    let control_plane_entry_hashes = control_plane
        .entries
        .iter()
        .zip(control_plane_entries.iter())
        .map(|(entry, value)| {
            Ok(json!({
                "scope_ref": entry.scope_ref,
                "sha256": canonical_value_sha256(value)?,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let protected_control_sha = protected_control_bytes.as_deref().map(sha256_digest);
    let candidate_control_sha = sha256_digest(&candidate_control_bytes);
    Ok(ResolvedRetirement {
        protected,
        candidate,
        subject,
        protected_tree,
        candidate_tree,
        protected_entries,
        candidate_entries,
        predecessor_entries,
        predecessor,
        predecessor_tree,
        control_plane,
        candidate_control_oid,
        candidate_control_bytes,
        protected_control_oid,
        protected_control_bytes,
        bootstrap,
        stages,
        active_receipt_population,
        equivalence_index,
        protected_receipt_inventory,
        candidate_receipt_inventory,
        expected_receipt_paths,
        unexpected_protected_receipt_paths,
        unexpected_candidate_receipt_paths,
        control_plane_entries,
        control_plane_entry_hashes,
        protected_control_sha,
        candidate_control_sha,
    })
}
