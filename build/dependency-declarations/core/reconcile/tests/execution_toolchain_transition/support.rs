use dependency_declarations_reconcile::{
    ExecutionToolchainAnalysis, ExecutionToolchainAnalysisRefusal,
    PatchOnlyExecutionToolchainDecision, PatchOnlyExecutionToolchainRefusal,
    analyze_execution_toolchain_transition, apply_patch_only_execution_toolchain_policy,
};

pub(crate) fn workspace(msrv: &str) -> String {
    format!("[workspace]\n[workspace.package]\nrust-version = {msrv:?}\n")
}

pub(crate) fn declaration(channel: &str) -> String {
    custom_declaration(channel, "[\"rustfmt\", \"clippy\"]", "minimal", "")
}

pub(crate) fn custom_declaration(
    channel: &str,
    components: &str,
    profile: &str,
    tail: &str,
) -> String {
    format!(
        "[toolchain]\nchannel = {channel:?}\ncomponents = {components}\nprofile = {profile:?}\n{tail}"
    )
}

pub(crate) fn analyze(
    protected: &str,
    candidate: &str,
    protected_msrv: &str,
    candidate_msrv: &str,
) -> Result<ExecutionToolchainAnalysis, ExecutionToolchainAnalysisRefusal> {
    analyze_execution_toolchain_transition(
        protected,
        candidate,
        &workspace(protected_msrv),
        &workspace(candidate_msrv),
    )
}

pub(crate) fn apply_policy(
    protected: &str,
    candidate: &str,
    protected_msrv: &str,
    candidate_msrv: &str,
) -> Result<PatchOnlyExecutionToolchainDecision, PatchOnlyExecutionToolchainRefusal> {
    let analysis = analyze(protected, candidate, protected_msrv, candidate_msrv).unwrap();
    apply_patch_only_execution_toolchain_policy(&analysis)
}
