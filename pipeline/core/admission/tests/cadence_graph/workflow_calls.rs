pub(super) fn validate(workflow: &str) -> Result<(), String> {
    let document: serde_yaml::Value =
        serde_yaml::from_str(workflow).map_err(|error| format!("workflow YAML: {error}"))?;
    let jobs = document
        .as_mapping()
        .and_then(|root| root.get("jobs"))
        .and_then(serde_yaml::Value::as_mapping)
        .filter(|jobs| !jobs.is_empty())
        .ok_or("workflow requires a nonempty jobs mapping")?;
    for (name, job) in jobs {
        let name = name.as_str().ok_or("job name must be a string")?;
        let job = job
            .as_mapping()
            .ok_or_else(|| format!("job {name}: expected a mapping"))?;
        if let Some(call) = job.get("uses") {
            let file = call
                .as_str()
                .and_then(|call| call.strip_prefix("oyatie/oyatie/.github/workflows/"))
                .and_then(|call| call.strip_suffix("@dev"))
                .and_then(|file| {
                    file.strip_suffix(".yml")
                        .or_else(|| file.strip_suffix(".yaml"))
                });
            if !file.is_some_and(|name| !name.is_empty() && !name.contains('/')) {
                return Err(format!(
                    "job {name}: reusable workflow must use an Oyatie workflow at @dev"
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn reusable_calls_require_protected_source_independent_of_scalar_style() {
    for value in [
        "./.github/workflows/qualification.yml",
        "'./.github/workflows/qualification.yml'",
        "\"./.github/workflows/qualification.yml\"",
        "oyatie/oyatie/.github/workflows/qualification.yml@candidate",
    ] {
        let workflow = format!("jobs:\n  qualification:\n    uses: {value}\n");
        assert!(validate(&workflow).is_err(), "unprotected call: {value}");
    }
}

#[test]
fn local_steps_and_script_text_are_not_reusable_workflow_calls() {
    let workflow = "jobs:\n  test:\n    runs-on: ubuntu-24.04\n    steps:\n      - name: local step\n        uses: ./local-action\n      - run: |\n          uses: ./example\n";
    assert!(validate(workflow).is_ok());
}

#[test]
fn protected_calls_accept_semantically_equivalent_yaml() {
    for workflow in [
        "jobs:\n  qualification:\n    uses: oyatie/oyatie/.github/workflows/qualification.yml@dev\n",
        "jobs: {qualification: {uses: 'oyatie/oyatie/.github/workflows/qualification.yml@dev'}}",
        "jobs: {qualification: {uses: 'oyatie/oyatie/.github/workflows/qualification.v2.yml@dev'}}",
    ] {
        assert!(validate(workflow).is_ok());
    }
}

#[test]
fn aliased_call_values_retain_their_source_identity() {
    for (source, accepted) in [
        (
            "oyatie/oyatie/.github/workflows/qualification.yml@dev",
            true,
        ),
        ("./.github/workflows/qualification.yml", false),
    ] {
        let workflow =
            format!("env:\n  CALL: &call '{source}'\njobs:\n  qualification:\n    uses: *call\n");
        assert_eq!(validate(&workflow).is_ok(), accepted);
    }
}

#[test]
fn malformed_or_ambiguous_job_graph_is_refused() {
    for workflow in [
        "jobs: [",
        "jobs: []",
        "name: no jobs",
        "jobs: {qualification: {uses: 3}}",
        "jobs: {qualification: {uses: first, uses: second}}",
    ] {
        assert!(validate(workflow).is_err(), "invalid graph: {workflow}");
    }
}

#[test]
fn cadence_test_modules_fit_native_file_budget() {
    for path in [
        "pipeline/core/admission/tests/cadence_graph.rs",
        "pipeline/core/admission/tests/cadence_graph/workflow_calls.rs",
    ] {
        let violations =
            pipeline_admission::file_budget_violations(path, super::read(path).as_bytes());
        assert!(violations.is_empty(), "{violations:?}");
    }
}
