/// Runtime context accepted by the facts materializer.
///
/// Public solely so the package-local integration target can exercise the
/// real Git boundary without duplicating production materialization logic.
#[derive(Debug, Clone)]
pub struct RetirementMaterializationContext<'a> {
    pub control_plane_path: &'a str,
    pub protected_base_commit: &'a str,
    pub evaluated_commit: &'a str,
    pub scm_event_name: &'a str,
    pub scm_event_ref: &'a str,
    pub scm_event_base_ref: &'a str,
    pub subject_commit: &'a str,
}

/// Select the immutable revision a policy must inspect from a controller-supplied SCM event
/// tuple.  The evaluated object must be the checkout HEAD; pull requests select only their
/// exact second-parent subject, while push and merge-group select their evaluated object.
pub(super) fn census_revision_from_event(
    repo_root: &Path,
    context: &RetirementMaterializationContext<'_>,
) -> Result<String, String> {
    let source = GitCliRetirementObjectSource::new(repo_root.to_path_buf());
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
    let head = source.resolve_commit("HEAD")?;
    let protected = source.resolve_commit(context.protected_base_commit)?;
    let evaluated = source.resolve_commit(context.evaluated_commit)?;
    let subject = source.resolve_commit(context.subject_commit)?;
    if head != evaluated
        || protected != context.protected_base_commit
        || evaluated != context.evaluated_commit
        || subject != context.subject_commit
    {
        return Err(
            "retirement SCM event identity must equal immutable resolved commit identity"
                .to_owned(),
        );
    }
    validate_event_identity(
        &source,
        context.scm_event_name,
        context.scm_event_ref,
        context.scm_event_base_ref,
        &protected,
        &evaluated,
        &subject,
    )?;
    if source.first_parent(&evaluated)? != protected {
        return Err("retirement protected base is not evaluated first parent".to_owned());
    }
    Ok(if context.scm_event_name == "pull_request" {
        subject
    } else {
        evaluated
    })
}

/// Derive the only historical dev-push tuple accepted for a clean merge-base worktree.
/// Absence of the control plane preserves bootstrap materialization; all present-control-plane
/// topologies bind an exact expected head and exactly one first parent.
pub fn historical_dev_push_context(
    repo_root: &Path,
    expected_head: &str,
) -> Result<Option<(String, String)>, String> {
    let source = GitCliRetirementObjectSource::new(repo_root.to_path_buf());
    historical_dev_push_context_from_source(&source, expected_head)
}

fn historical_dev_push_context_from_source(
    source: &impl RetirementObjectSource,
    expected_head: &str,
) -> Result<Option<(String, String)>, String> {
    let evaluated = source.resolve_commit(expected_head)?;
    if evaluated != expected_head {
        return Err("historical dev-push expected head does not resolve exactly".to_owned());
    }
    if !source
        .tree_entries(&evaluated)?
        .iter()
        .any(|entry| entry.path == CONTROL_PLANE_PATH)
    {
        return Ok(None);
    }
    let parents = source.parents(&evaluated)?;
    if parents.len() != 1 {
        return Err("historical dev-push requires exactly one parent".to_owned());
    }
    let first_parent = source.first_parent(&evaluated)?;
    if first_parent != parents[0] {
        return Err("historical dev-push first-parent resolution drifted".to_owned());
    }
    Ok(Some((evaluated, first_parent)))
}

/// Materialize facts through the sanctioned Git boundary.
///
/// This is public for the package-local integration target; it emits facts and
/// never produces a PASS or dispatch decision.
pub fn emit_history_only_retirement_facts(
    repo_root: &Path,
    context: &RetirementMaterializationContext<'_>,
    output_path: &Path,
) -> Result<(), String> {
    canonical_generated_facts_output_path(repo_root, output_path)?;
    let source = GitCliRetirementObjectSource::new(repo_root.to_path_buf());
    let value = materialize_history_only_retirement_facts(&source, context)?;
    let bytes = to_canonical_json(&value)
        .map_err(|error| format!("serialize history-only retirement facts: {error}"))?;
    write_canonical_retirement_facts(repo_root, bytes.as_bytes())
}
