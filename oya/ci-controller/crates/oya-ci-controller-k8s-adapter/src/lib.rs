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
//! - `activeDeadlineSeconds` — from spec (mirrors 60 min Jenkins timeout)
//! - `ttlSecondsAfterFinished` — GC (Prow sinker equivalent)
//! - `restartPolicy: Never`
//!
//! ## Trunk-sourcing (security invariant)
//!
//! The Job's init/main container:
//! 1. Clones `dev` from Forgejo (trusted gate script on disk)
//! 2. Fetches PR ref as DATA (untrusted)
//! 3. Runs `sh infra/ci/buck2-affected-gate.sh origin/dev` — uses dev's script,
//!    NOT the PR's copy. A PR cannot weaken its own gate.
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
use oya_ci_controller_kernel::{
    ForgejoState, JobCondition, JobConditionType, JobHandle, JobObservation, JobSpawner,
    KernelError, PodReason, Result as KernelResult, GateRunSpec,
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
pub const ANNOT_CI_STATUS_POSTED: &str = "oya.io/ci-forgejo-status-posted";

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
/// - Runs a single container that clones dev from Forgejo, fetches the PR
///   ref as data, then runs `sh infra/ci/buck2-affected-gate.sh origin/dev`
///   using dev's TRUSTED copy of the script.
/// - Injects `FORGEJO_CI_TOKEN` from the `forgejo-ci-token` Secret.
/// - Sets `HOME=/home/jenkins/agent` (matches rust-ci image expectations).
pub fn build_gate_job(spec: &GateRunSpec) -> Job {
    let job_name = spec.run.job_name();
    let sha = &spec.run.head_sha;
    let pr_number = spec.run.pr_number;
    let base_ref = &spec.run.base_ref;
    let clone_url = &spec.forge_clone_url;

    // The gate command (TRUNK-SOURCED security invariant):
    // 1. Clone dev (trusted gate script + infra arrive here)
    // 2. Fetch the PR ref as DATA only — never checkout to HEAD
    // 3. Run gate script from dev's trusted copy with 2-arg form:
    //    buck2-affected-gate.sh <base-ref> <head-ref>
    //
    // Working tree = dev (trunk). The PR ref is available as
    // origin/pr-<N> after the fetch; passed as head-ref so the script
    // diffs merge-base(origin/pr-N, origin/dev)..origin/pr-N.
    //
    // A PR MUST NOT be able to alter the script/Job that gates it.
    let gate_cmd = format!(
        r#"set -euo pipefail
git clone --depth=1 --branch {base_ref} {clone_url} /workspace/repo
cd /workspace/repo
git fetch --unshallow
git fetch origin refs/pull/{pr_number}/head:refs/remotes/origin/pr-{pr_number}
exec sh infra/ci/buck2-affected-gate.sh origin/{base_ref} origin/pr-{pr_number}"#,
    );

    // Labels (immutable identity — used as the watcher selector)
    let mut labels: BTreeMap<String, String> = BTreeMap::new();
    labels.insert(LABEL_CI_CONTROLLER.to_owned(), CI_CONTROLLER_VALUE.to_owned());
    labels.insert(LABEL_CI_PR_NUMBER.to_owned(), pr_number.to_string());
    labels.insert(LABEL_CI_HEAD_SHA.to_owned(), sha.clone());
    labels.insert(LABEL_CI_DELIVERY_ID.to_owned(), spec.run.delivery_id.clone());
    labels.insert(LABEL_PART_OF.to_owned(), "oyatie-microservices".to_owned());

    // Annotations (mutable bookkeeping)
    let mut annotations: BTreeMap<String, String> = BTreeMap::new();
    annotations.insert(ANNOT_CI_BASE_REF.to_owned(), base_ref.clone());

    let container = Container {
        name: "gate".to_owned(),
        image: Some(spec.image.clone()),
        command: Some(vec!["/bin/sh".to_owned(), "-c".to_owned(), gate_cmd]),
        // SECURITY: FORGEJO_CI_TOKEN MUST NOT be injected here.
        // The gate Job runs untrusted PR code; a token in the container env
        // would allow a malicious PR to exfiltrate it and post arbitrary
        // commit statuses. Only the controller (crier) holds the token.
        env: Some(vec![
            EnvVar {
                name: "HOME".to_owned(),
                value: Some("/home/jenkins/agent".to_owned()),
                ..Default::default()
            },
        ]),
        security_context: Some(k8s_openapi::api::core::v1::SecurityContext {
            allow_privilege_escalation: Some(false),
            read_only_root_filesystem: Some(false), // gate needs /tmp, /home/jenkins/agent
            run_as_non_root: Some(true),
            // rust-ci image has no non-root USER; pin uid 1000 (the build-pod uid)
            // so runAsNonRoot is satisfiable.
            run_as_user: Some(1000),
            capabilities: Some(k8s_openapi::api::core::v1::Capabilities {
                drop: Some(vec!["ALL".to_owned()]),
                ..Default::default()
            }),
            ..Default::default()
        }),
        // Writable emptyDir workspaces (uid 1000 can't mkdir under root-owned /).
        volume_mounts: Some(vec![
            k8s_openapi::api::core::v1::VolumeMount {
                name: "workspace".to_owned(),
                mount_path: "/workspace".to_owned(),
                ..Default::default()
            },
            k8s_openapi::api::core::v1::VolumeMount {
                name: "home".to_owned(),
                mount_path: "/home/jenkins/agent".to_owned(),
                ..Default::default()
            },
        ]),
        ..Default::default()
    };

    let pod_spec = PodSpec {
        restart_policy: Some("Never".to_owned()),
        service_account_name: Some(spec.runner_service_account.clone()),
        automount_service_account_token: Some(false),
        containers: vec![container],
        security_context: Some(k8s_openapi::api::core::v1::PodSecurityContext {
            run_as_non_root: Some(true),
            run_as_user: Some(1000),
            run_as_group: Some(1000),
            // fs_group chowns the emptyDir workspaces to gid 1000 so uid 1000 can write.
            fs_group: Some(1000),
            seccomp_profile: Some(k8s_openapi::api::core::v1::SeccompProfile {
                type_: "RuntimeDefault".to_owned(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        volumes: Some(vec![
            k8s_openapi::api::core::v1::Volume {
                name: "workspace".to_owned(),
                empty_dir: Some(k8s_openapi::api::core::v1::EmptyDirVolumeSource::default()),
                ..Default::default()
            },
            k8s_openapi::api::core::v1::Volume {
                name: "home".to_owned(),
                empty_dir: Some(k8s_openapi::api::core::v1::EmptyDirVolumeSource::default()),
                ..Default::default()
            },
        ]),
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
            Err(e) => Err(KernelError::DownstreamTransport(format!(
                "job create: {e}"
            ))),
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
    let pod_reasons: Vec<PodReason> = pods
        .iter()
        .flat_map(|pod| extract_pod_reasons(pod))
        .collect();

    // Read the status-posted annotation
    let posted_annotation = job
        .metadata
        .annotations
        .as_ref()
        .and_then(|a| a.get(ANNOT_CI_STATUS_POSTED))
        .map(|v| v.as_str());

    let terminal_status_already_posted = match posted_annotation {
        Some("success") => Some(ForgejoState::Success),
        Some("failure") => Some(ForgejoState::Failure),
        Some("error") => Some(ForgejoState::Error),
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
        if let Some(r) = &status.reason {
            if !r.is_empty() {
                reasons.push(PodReason::from_str(r));
            }
        }

        // Container statuses
        for cs in status.container_statuses.as_deref().unwrap_or_default() {
            if let Some(state) = &cs.state {
                // Terminated reason (e.g. OOMKilled)
                if let Some(terminated) = &state.terminated {
                    if let Some(r) = &terminated.reason {
                        if !r.is_empty() {
                            reasons.push(PodReason::from_str(r));
                        }
                    }
                }
                // Waiting reason (e.g. ImagePullBackOff, CrashLoopBackOff)
                if let Some(waiting) = &state.waiting {
                    if let Some(r) = &waiting.reason {
                        if !r.is_empty() {
                            reasons.push(PodReason::from_str(r));
                        }
                    }
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
