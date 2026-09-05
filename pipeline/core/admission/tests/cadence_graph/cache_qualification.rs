#[test]
fn cache_qualification_is_required_by_the_real_fan_in() {
    let workflow = super::read(".github/workflows/presubmit.yml");
    let script = workflow
        .split_once("      - name: Fan-in verdict\n")
        .and_then(|(_, body)| body.split_once("        run: |\n"))
        .map(|(_, script)| script)
        .expect("fan-in shell");
    for result in ["success", "skipped", "cancelled", "failure", ""] {
        let mut rendered = script.replace("${{ needs.build-cache-qualification.result }}", result);
        while let Some(start) = rendered.find("${{") {
            let end = start + rendered[start..].find("}}").expect("closed expression") + 2;
            rendered.replace_range(start..end, "success");
        }
        let output = std::process::Command::new("bash")
            .args(["-euo", "pipefail", "-c", &rendered])
            .env("BACKBONE_POSTGRES", "true")
            .env("COMPUTE_LIFECYCLE_POSTGRES", "true")
            .env("REINDEER", "true")
            .env("EVENT", "pull_request")
            .output()
            .expect("run real fan-in");
        assert_eq!(
            output.status.success(),
            result == "success",
            "cache result {result:?}"
        );
    }
}

#[test]
fn cache_qualification_has_an_unconditional_layout_dependent_consumer() {
    let workflow = super::read(".github/workflows/presubmit.yml");
    let job = workflow
        .split_once("\n  build-cache-qualification:\n")
        .and_then(|(_, body)| body.split_once("\n  presubmit:\n"))
        .map(|(body, _)| body)
        .expect("required cache qualification job");
    assert!(job.contains("needs: [layout]"));
    assert!(!job.contains("    if:") && !job.contains("secrets:"));
    // Fully qualified, not `./`: the required workflow is pinned at
    // `refs/heads/dev`, so a local reference resolves against the candidate
    // and the run never starts.
    assert!(
        job.contains("uses: oyatie/oyatie/.github/workflows/build-cache-qualification.yml@dev")
    );
    assert!(job.contains("permissions: {contents: read}"));
}

#[test]
fn cache_qualification_runs_pinned_real_inputs_without_privileged_context() {
    let workflow = std::fs::read_to_string(
        super::repo_root().join(".github/workflows/build-cache-qualification.yml"),
    )
    .expect("cache qualification workflow");
    for line in workflow.lines() {
        if let Some(action) = line.trim().strip_prefix("uses: ") {
            let (_, revision) = action.split_once('@').expect("immutable action revision");
            assert!(revision.len() == 40 && revision.bytes().all(|b| b.is_ascii_hexdigit()));
        }
    }
    for required in [
        "workflow_call:",
        "workflow_dispatch:",
        "pull_request:\n    branches: [dev]\n    paths:\n      - build/iac/nativelink/**\n      - .github/workflows/build-cache-qualification.yml",
        "permissions:\n  contents: read",
        "runner: ubuntu-24.04\n",
        "runner: ubuntu-24.04-arm\n",
        "fail-fast: false",
        "ref: ${{ github.sha }}",
        "WORKFLOW_REVISION: ${{ github.workflow_sha }}",
        "ref: 77ec630134abbf9aa525f921eee4e5d11dc20f7e",
        "ref: 64aa30b277168edd20efee0c9ceb4ca01248931d",
        "sha256sum --check --strict",
        "v0.22.1/go-containerregistry_Linux_",
        "v1.9.3/grpcurl_1.9.3_",
        "helm-v3.20.0-",
        "minitest-5.25.5.gem",
        "ruby build/iac/nativelink/tests/chart_test.rb",
        "ruby build/iac/nativelink/tests/fixture_test.rb",
        "ruby build/iac/nativelink/tests/runtime_images_test.rb",
        "ruby build/iac/nativelink/tests/store_test.rb",
        "ruby build/iac/nativelink/tests/gateway_test.rb",
    ] {
        assert!(
            workflow.contains(required),
            "missing qualification input: {required}"
        );
    }
    for forbidden in [
        "pull_request_target:",
        "secrets:",
        "secrets.",
        "persist-credentials: true",
        "continue-on-error:",
        "self-hosted",
        "id-token:",
        "sudo ",
    ] {
        assert!(
            !workflow.contains(forbidden),
            "unexpected authority: {forbidden}"
        );
    }
    assert_eq!(workflow.matches("persist-credentials: false").count(), 3);
}

#[test]
fn qualified_executables_are_acquired_from_rendered_chart_images() {
    let workflow = super::read(".github/workflows/build-cache-qualification.yml");
    assert!(
        workflow.contains("ruby build/iac/nativelink/tests/runtime_images.rb"),
        "qualification must derive executables from the rendered chart image references"
    );
    for divergent_source in [
        "TraceMachina/nativelink/releases/download",
        "envoyproxy/envoy/releases/download",
        "native_sha256:",
        "envoy_sha256:",
    ] {
        assert!(
            !workflow.contains(divergent_source),
            "divergent source: {divergent_source}"
        );
    }
}
