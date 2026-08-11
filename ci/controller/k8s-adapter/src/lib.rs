//! # oya-ci-controller-k8s-adapter
//!
//! K8s Job spawn + watch adapter for the oya-ci controller.
//!
//! Implements:
//! - [`JobSpawner`] — creates a labeled `batch/v1 Job` for a gate run
//! - [`observe_job`] — projects a live Job + its owned Pods into a kernel
//!   [`JobObservation`] for the pure state machine
//!
//! ## Job design (from oya-ci-bespoke-prow.md)
//!
//! - Labels (immutable identity): `oya.io/ci-controller=oya-ci-gate`,
//!   `oya.io/ci-pr-number=<N>`, `oya.io/ci-head-sha=<sha>`,
//!   `oya.io/ci-delivery-id=<id>`, `app.kubernetes.io/part-of=oyatie-microservices`
//! - `backoffLimit: 0` — fail-closed, no silent retry
//! - `activeDeadlineSeconds` — from spec (mirrors the legacy CI 60 min timeout)
//! - `ttlSecondsAfterFinished` — GC (Prow sinker equivalent)
//! - `restartPolicy: Never`
//!
//! ## Trunk-sourcing (security invariant)
//!
//! The Job's init/main container:
//! 1. Clones `dev` from GitHub (trusted control state / gate command source)
//! 2. Fetches PR ref as DATA (untrusted)
//! 3. Captures trusted build/test target inventories before checking out the
//!    candidate bytes, then runs the candidate against those immutable label
//!    sets. Candidate code cannot remove targets from the required context,
//!    change the status context, producer, or branch-protection mapping.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the hot path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use k8s_openapi::{
    api::{
        batch::v1::{Job, JobSpec},
        core::v1::{Container, EnvVar, PodSpec, PodTemplateSpec},
    },
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};
use kube::{
    Api, Client,
    api::{ListParams, PostParams},
};
use ci_controller_kernel::{
    CommitState, GATE_CONTEXT, GateRunSpec, JobCondition, JobConditionType, JobHandle,
    JobObservation, JobSpawner, KernelError, PodReason, Result as KernelResult,
};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Label / annotation key constants
// ---------------------------------------------------------------------------

pub const LABEL_CI_CONTROLLER: &str = "oya.io/ci-controller";
pub const LABEL_CI_PR_NUMBER: &str = "oya.io/ci-pr-number";
pub const LABEL_CI_HEAD_SHA: &str = "oya.io/ci-head-sha";
pub const LABEL_CI_DELIVERY_ID: &str = "oya.io/ci-delivery-id";
pub const LABEL_PART_OF: &str = "app.kubernetes.io/part-of";

pub const ANNOT_CI_BASE_REF: &str = "oya.io/ci-base-ref";
pub const ANNOT_CI_STATUS_POSTED: &str = "oya.io/ci-status-posted";
pub const ANNOT_CI_REQUIRED_CONTEXT: &str = "oya.io/ci-required-context";
pub const ANNOT_CI_PRODUCER_KIND: &str = "oya.io/ci-producer-kind";
pub const ANNOT_CI_PRODUCER_CONTROLLER: &str = "oya.io/ci-producer-controller";
pub const ANNOT_CI_CANDIDATE_BYTES_POLICY: &str = "oya.io/ci-candidate-bytes-policy";
pub const ANNOT_CI_GATE_DEFINITION_SOURCE: &str = "oya.io/ci-gate-definition-source";

/// Value for the LABEL_CI_CONTROLLER label — the watcher selector.
pub const CI_CONTROLLER_VALUE: &str = "oya-ci-gate";

/// Label selector string for the kube-rs watcher.
pub const WATCHER_LABEL_SELECTOR: &str = "oya.io/ci-controller=oya-ci-gate";

// ---------------------------------------------------------------------------
// Build the K8s Job spec
// ---------------------------------------------------------------------------

/// Build a `batch/v1 Job` for the given gate run spec.
///
/// The Job:
/// - Has `backoffLimit: 0` (fail-closed)
/// - Runs a single container that clones dev from GitHub, fetches the PR
///   ref as untrusted candidate data, snapshots trusted build/test target
///   inventories, then runs the candidate tree against those immutable
///   inventories.
/// - Sets `HOME=/home/ci/agent` (matches rust-ci image expectations).
pub fn build_gate_job(spec: &GateRunSpec) -> Job {
    let job_name = spec.run.job_name();
    let sha = &spec.run.head_sha;
    let pr_number = spec.run.pr_number;
    let base_ref = &spec.run.base_ref;
    let clone_url = &spec.forge_clone_url;

    // The gate command (TRUNK/CONTROLLER-SOURCED security invariant):
    // 1. Clone dev so controller-owned gate commands and metadata are fixed.
    // 2. Fetch the PR ref as untrusted candidate bytes.
    // 3. Snapshot trusted build/test target inventories before candidate
    //    checkout.
    // 4. Checkout the candidate only after the gate command and target labels
    //    are fixed, then run the candidate tree against those immutable lists.
    //    A PR can change code under test, but deleting/omitting a trusted
    //    target makes the required context fail instead of silently shrinking
    //    scope.
    let gate_cmd = format!(
        r#"set -euo pipefail
git clone --depth=1 --branch {base_ref} {clone_url} /workspace/repo
cd /workspace/repo
git fetch --unshallow
git fetch origin refs/pull/{pr_number}/head:refs/remotes/origin/pr-{pr_number}
buck2 targets //... | sort -u > /workspace/trusted-build-targets.txt
buck2 uquery 'kind(".*test.*", //...)' | sort -u > /workspace/trusted-test-targets.txt
test -s /workspace/trusted-build-targets.txt
test -s /workspace/trusted-test-targets.txt
resolved_sha="$(git rev-parse refs/remotes/origin/pr-{pr_number})"
test "$resolved_sha" = "{sha}"
git checkout --detach {sha}
xargs -a /workspace/trusted-build-targets.txt buck2 build
xargs -a /workspace/trusted-test-targets.txt buck2 test"#,
    );

    // Labels (immutable identity — used as the watcher selector)
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(
        LABEL_CI_CONTROLLER.to_owned(),
        CI_CONTROLLER_VALUE.to_owned(),
    );
    labels.insert(LABEL_CI_PR_NUMBER.to_owned(), pr_number.to_string());
    labels.insert(LABEL_CI_HEAD_SHA.to_owned(), sha.clone());
    labels.insert(
        LABEL_CI_DELIVERY_ID.to_owned(),
        spec.run.delivery_id.clone(),
    );
    labels.insert(LABEL_PART_OF.to_owned(), "oyatie-microservices".to_owned());

    // Annotations (mutable bookkeeping)
    let mut annotations: BTreeMap<String, String> = BTreeMap::new();
    annotations.insert(ANNOT_CI_BASE_REF.to_owned(), base_ref.clone());
    annotations.insert(
        ANNOT_CI_REQUIRED_CONTEXT.to_owned(),
        GATE_CONTEXT.to_owned(),
    );
    annotations.insert(
        ANNOT_CI_PRODUCER_KIND.to_owned(),
        "oya-ci-controller".to_owned(),
    );
    annotations.insert(
        ANNOT_CI_PRODUCER_CONTROLLER.to_owned(),
        "oya-ci-controller".to_owned(),
    );
    annotations.insert(
        ANNOT_CI_CANDIDATE_BYTES_POLICY.to_owned(),
        "untrusted_input_only".to_owned(),
    );
    annotations.insert(
        ANNOT_CI_GATE_DEFINITION_SOURCE.to_owned(),
        "trusted_dev_or_controller_state".to_owned(),
    );

    let container = Container {
        name: "gate".to_owned(),
        image: Some(spec.image.clone()),
        command: Some(vec!["/bin/sh".to_owned(), "-c".to_owned(), gate_cmd]),
        // SECURITY: GITHUB_CI_TOKEN MUST NOT be injected here.
        // The gate Job runs untrusted PR code; a token in the container env
        // would allow a malicious PR to exfiltrate it and post arbitrary
        // commit statuses. Only the controller (crier) holds the token.
        env: Some(vec![EnvVar {
            name: "HOME".to_owned(),
            value: Some("/home/ci/agent".to_owned()),
            ..Default::default()
        }]),
        security_context: Some(k8s_openapi::api::core::v1::SecurityContext {
            allow_privilege_escalation: Some(false),
            read_only_root_filesystem: Some(false), // gate needs /tmp, /home/ci/agent
            run_as_non_root: Some(true),
            capabilities: Some(k8s_openapi::api::core::v1::Capabilities {
                drop: Some(vec!["ALL".to_owned()]),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let pod_spec = PodSpec {
        restart_policy: Some("Never".to_owned()),
        service_account_name: Some(spec.runner_service_account.clone()),
        automount_service_account_token: Some(false),
        containers: vec![container],
        security_context: Some(k8s_openapi::api::core::v1::PodSecurityContext {
            run_as_non_root: Some(true),
            seccomp_profile: Some(k8s_openapi::api::core::v1::SeccompProfile {
                type_: "RuntimeDefault".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    Job {
        metadata: ObjectMeta {
            name: Some(job_name),
            namespace: Some(spec.namespace.clone()),
            labels: Some(labels),
            annotations: Some(annotations),
            ..Default::default()
        },
        spec: Some(JobSpec {
            backoff_limit: Some(0),
            active_deadline_seconds: Some(spec.active_deadline_seconds),
            ttl_seconds_after_finished: Some(spec.ttl_seconds_after_finished),
            template: PodTemplateSpec {
                spec: Some(pod_spec),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// K8sJobSpawner — implements kernel::JobSpawner
// ---------------------------------------------------------------------------

/// K8s-backed [`JobSpawner`]. Creates the gate Job via the kube API.
pub struct K8sJobSpawner {
    client: Client,
}

impl K8sJobSpawner {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl JobSpawner for K8sJobSpawner {
    fn spawn(&self, spec: &GateRunSpec) -> KernelResult<JobHandle> {
        let job = build_gate_job(spec);
        let job_name = spec.run.job_name();
        let namespace = spec.namespace.clone();

        let api: Api<Job> = Api::namespaced(self.client.clone(), &namespace);

        // Use a one-shot tokio runtime to drive the async kube call from the
        // synchronous trait method. The app layer is async; this adapter can
        // also be called from the async reconcile via spawn_blocking if needed.
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| KernelError::DownstreamTransport(format!("tokio build: {e}")))?;

        let result = rt.block_on(api.create(&PostParams::default(), &job));

        match result {
            Ok(_) => Ok(JobHandle {
                job_name,
                namespace,
                already_exists: false,
            }),
            Err(kube::Error::Api(err)) if err.code == 409 => {
                // Conflict = already exists — idempotent
                Ok(JobHandle {
                    job_name,
                    namespace,
                    already_exists: true,
                })
            }
            Err(e) => Err(KernelError::DownstreamTransport(format!("job create: {e}"))),
        }
    }
}

// ---------------------------------------------------------------------------
// observe_job — project live Job + Pods into JobObservation
// ---------------------------------------------------------------------------

/// Project a live [`Job`] and its owned Pods into a kernel [`JobObservation`].
///
/// `waiting_cycles` is maintained by the caller (reconcile loop) — it counts
/// how many consecutive reconciles the Job has been in a waiting-pod-reason
/// state without a terminal condition appearing.
pub fn observe_job(
    job: &Job,
    pods: &[k8s_openapi::api::core::v1::Pod],
    waiting_cycles: u32,
) -> JobObservation {
    let status = job.status.as_ref();

    let active = status.and_then(|s| s.active).unwrap_or(0);
    let succeeded = status.and_then(|s| s.succeeded).unwrap_or(0);
    let failed = status.and_then(|s| s.failed).unwrap_or(0);

    // Parse conditions
    let conditions: Vec<JobCondition> = status
        .and_then(|s| s.conditions.as_ref())
        .map(|conds| {
            conds
                .iter()
                .filter_map(|c| {
                    let ct = match c.type_.as_str() {
                        "Complete" => JobConditionType::Complete,
                        "Failed" => JobConditionType::Failed,
                        _ => return None,
                    };
                    Some(JobCondition {
                        condition_type: ct,
                        reason: c.reason.clone(),
                        status: c.status == "True",
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Parse pod reasons
    let pod_reasons: Vec<PodReason> = pods.iter().flat_map(extract_pod_reasons).collect();

    // Read the status-posted annotation
    let posted_annotation = job
        .metadata
        .annotations
        .as_ref()
        .and_then(|a| a.get(ANNOT_CI_STATUS_POSTED))
        .map(|v| v.as_str());

    let terminal_status_already_posted = match posted_annotation {
        Some("success") => Some(CommitState::Success),
        Some("failure") => Some(CommitState::Failure),
        Some("error") => Some(CommitState::Error),
        _ => None,
    };

    let pending_status_already_posted = matches!(posted_annotation, Some("pending"));

    JobObservation {
        active,
        succeeded,
        failed,
        conditions,
        pod_reasons,
        waiting_cycles,
        job_not_found: false, // only set by the reconcile loop on NotFound
        terminal_status_already_posted,
        pending_status_already_posted,
    }
}

/// Extract relevant [`PodReason`] values from a single Pod.
fn extract_pod_reasons(pod: &k8s_openapi::api::core::v1::Pod) -> Vec<PodReason> {
    let mut reasons = Vec::new();

    // Pod-level status.reason (e.g. "Evicted")
    if let Some(status) = &pod.status {
        if let Some(r) = &status.reason
            && !r.is_empty()
        {
            reasons.push(PodReason::from(r.as_str()));
        }

        // Container statuses
        for cs in status.container_statuses.as_deref().unwrap_or_default() {
            if let Some(state) = &cs.state {
                // Terminated reason (e.g. OOMKilled)
                if let Some(terminated) = &state.terminated
                    && let Some(r) = &terminated.reason
                    && !r.is_empty()
                {
                    reasons.push(PodReason::from(r.as_str()));
                }
                // Waiting reason (e.g. ImagePullBackOff, CrashLoopBackOff)
                if let Some(waiting) = &state.waiting
                    && let Some(r) = &waiting.reason
                    && !r.is_empty()
                {
                    reasons.push(PodReason::from(r.as_str()));
                }
            }
        }
    }

    reasons
}

/// Build a [`ListParams`] that filters only oya-ci-gate Jobs.
pub fn gate_job_list_params() -> ListParams {
    ListParams::default().labels(WATCHER_LABEL_SELECTOR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ci_controller_kernel::{GateRun, GateRunSpec};

    fn gate_spec() -> GateRunSpec {
        GateRunSpec {
            run: GateRun {
                pr_number: 42,
                head_sha: "abcdef1234567890abcdef1234567890abcdef12".to_owned(),
                delivery_id: "delivery-1".to_owned(),
                base_ref: "dev".to_owned(),
                repo: "oya-admin/oyatie".to_owned(),
            },
            image: "registry.local/rust-ci:dev".to_owned(),
            forge_clone_url: "https://github.com/jason931225/oyatie.git".to_owned(),
            active_deadline_seconds: 3600,
            ttl_seconds_after_finished: 600,
            namespace: "oya-ci".to_owned(),
            runner_service_account: "oya-ci-gate-runner".to_owned(),
        }
    }

    #[test]
    fn gate_job_carries_required_context_and_trusted_producer_metadata() {
        let job = build_gate_job(&gate_spec());
        let annotations = job
            .metadata
            .annotations
            .as_ref()
            .expect("gate job should carry annotations");

        assert_eq!(
            annotations
                .get(ANNOT_CI_REQUIRED_CONTEXT)
                .map(String::as_str),
            Some(GATE_CONTEXT)
        );
        assert_eq!(
            annotations.get(ANNOT_CI_PRODUCER_KIND).map(String::as_str),
            Some("oya-ci-controller")
        );
        assert_eq!(
            annotations
                .get(ANNOT_CI_PRODUCER_CONTROLLER)
                .map(String::as_str),
            Some("oya-ci-controller")
        );
        assert_eq!(
            annotations
                .get(ANNOT_CI_CANDIDATE_BYTES_POLICY)
                .map(String::as_str),
            Some("untrusted_input_only")
        );
        assert_eq!(
            annotations
                .get(ANNOT_CI_GATE_DEFINITION_SOURCE)
                .map(String::as_str),
            Some("trusted_dev_or_controller_state")
        );
    }

    #[test]
    fn gate_job_uses_controller_owned_required_matrix_not_affected_only_bridge() {
        let job = build_gate_job(&gate_spec());
        let pod_spec = job
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .expect("gate job should carry pod spec");
        let command = pod_spec.containers[0]
            .command
            .as_ref()
            .expect("gate container should carry command")
            .join("\n");

        assert!(
            command.contains("git clone --depth=1 --branch dev"),
            "gate should start from trusted base branch"
        );
        assert!(
            command.contains("git fetch origin refs/pull/42/head:refs/remotes/origin/pr-42"),
            "gate should fetch candidate bytes as an explicit PR ref"
        );
        assert!(
            command.contains("buck2 targets //... | sort -u > /workspace/trusted-build-targets.txt")
                && command.contains(
                    "buck2 uquery 'kind(\".*test.*\", //...)' | sort -u > /workspace/trusted-test-targets.txt"
                )
                && command.contains("test -s /workspace/trusted-build-targets.txt")
                && command.contains("test -s /workspace/trusted-test-targets.txt"),
            "gate should snapshot trusted build/test inventories before candidate checkout"
        );
        assert!(
            command.contains("resolved_sha=\"$(git rev-parse refs/remotes/origin/pr-42)\"")
                && command.contains(
                    "test \"$resolved_sha\" = \"abcdef1234567890abcdef1234567890abcdef12\""
                )
                && command
                    .contains("git checkout --detach abcdef1234567890abcdef1234567890abcdef12"),
            "gate should verify the fetched PR ref matches the exact candidate SHA before checkout"
        );
        assert!(
            command.contains("xargs -a /workspace/trusted-build-targets.txt buck2 build")
                && command.contains("xargs -a /workspace/trusted-test-targets.txt buck2 test")
                && !command.contains("buck2 build //...[check]")
                && !command.contains("buck2 test //..."),
            "gate should run the candidate against trusted inventories, not candidate-selected target globs"
        );
        assert!(
            !command.contains("buck2-affected-gate.sh") && !command.contains("oya "),
            "required context must not delegate to affected-only bridge or oya CLI"
        );

        let env = pod_spec.containers[0].env.as_ref().expect("env exists");
        assert!(
            env.iter().all(|var| var.name != "GITHUB_CI_TOKEN"),
            "runner job must not receive status-posting credentials"
        );
    }
}
