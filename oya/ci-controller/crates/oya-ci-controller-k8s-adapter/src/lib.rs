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
//! 1. Clones `dev` from Forgejo in a trusted init container.
//! 2. Fetches PR ref as DATA (untrusted).
//! 3. Captures trusted build/test target inventories before checking out the
//!    candidate bytes, then runs the candidate against those immutable label
//!    sets. A PR cannot weaken its own required context by editing gate logic,
//!    status context, or target discovery.
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
    ForgejoState, GateRunSpec, JobCondition, JobConditionType, JobHandle, JobObservation,
    JobSpawner, KernelError, PodReason, Result as KernelResult,
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
/// - Runs a trusted init container that clones dev from Forgejo and fetches the
///   PR ref as data, then a tokenless main container that snapshots trusted
///   build/test target inventories, verifies the exact candidate SHA, and runs
///   the candidate against those immutable inventories.
/// - Injects the clone token only into the trusted init container.
/// - Sets `HOME=/home/jenkins/agent` (matches rust-ci image expectations).
pub fn build_gate_job(spec: &GateRunSpec) -> Job {
    let job_name = spec.run.job_name();
    let sha = spec.run.head_sha.to_ascii_lowercase();
    let pr_number = spec.run.pr_number;
    let base_ref = &spec.run.base_ref;
    let clone_url = &spec.forge_clone_url;

    // The gate command (TRUNK/CONTROLLER-SOURCED security invariant):
    // 1. Clone dev and fetch the PR ref in a trusted init container.
    // 2. The tokenless main container snapshots trusted build/test target
    //    inventories before candidate checkout.
    // 3. It verifies the fetched PR ref resolves to the exact requested SHA.
    // 4. It checks out the candidate only after the gate command and target
    //    labels are fixed, then runs the candidate tree against immutable
    //    target lists. A PR can change code under test, but deleting/omitting a
    //    trusted target makes the required context fail instead of silently
    //    shrinking scope.
    //
    // A PR MUST NOT be able to alter the script/Job that gates it.
    // Clone runs in a TRUSTED init container holding the clone token. The token
    // is passed via `git -c http.extraHeader` (one-shot — NOT persisted to
    // .git/config), so it never leaks into the shared workspace the untrusted
    // main container reads. clone_url carries NO embedded credentials.
    let clone_cmd = format!(
        r#"set -euo pipefail
AUTH="http.extraHeader=Authorization: token ${{FORGEJO_CLONE_TOKEN}}"
git -c "$AUTH" clone --depth=1 --branch {base_ref} {clone_url} /workspace/repo
cd /workspace/repo
git -c "$AUTH" fetch --unshallow
git -c "$AUTH" fetch origin refs/pull/{pr_number}/head:refs/remotes/origin/pr-{pr_number}"#,
    );
    // The gate runs in the UNTRUSTED main container on the pre-cloned workspace —
    // NO token, NO network clone. Working tree = dev (trunk); the PR ref is
    // origin/pr-N (data only). A PR MUST NOT alter the script/Job that gates it.
    let gate_cmd = format!(
        r#"set -euo pipefail
cd /workspace/repo
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

    let container = Container {
        name: "gate".to_owned(),
        image: Some(spec.image.clone()),
        command: Some(vec!["/bin/sh".to_owned(), "-c".to_owned(), gate_cmd]),
        // SECURITY: FORGEJO_CI_TOKEN MUST NOT be injected here.
        // The gate Job runs untrusted PR code; a token in the container env
        // would allow a malicious PR to exfiltrate it and post arbitrary
        // commit statuses. Only the controller (crier) holds the token.
        env: Some(vec![EnvVar {
            name: "HOME".to_owned(),
            value: Some("/home/jenkins/agent".to_owned()),
            ..Default::default()
        }]),
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

    // TRUSTED clone init container: holds the clone token (env only; passed to
    // git via one-shot `-c http.extraHeader`, never written to .git/config), and
    // completes BEFORE the untrusted main container starts — so PR build scripts
    // can never read it. (Interim: reuses forgejo-ci-token; a dedicated read-only
    // clone token is the hardening follow-up.)
    let clone_container = Container {
        name: "clone".to_owned(),
        image: Some(spec.image.clone()),
        command: Some(vec!["/bin/sh".to_owned(), "-c".to_owned(), clone_cmd]),
        env: Some(vec![
            EnvVar {
                name: "HOME".to_owned(),
                value: Some("/home/jenkins/agent".to_owned()),
                ..Default::default()
            },
            EnvVar {
                name: "FORGEJO_CLONE_TOKEN".to_owned(),
                value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                    secret_key_ref: Some(k8s_openapi::api::core::v1::SecretKeySelector {
                        name: "forgejo-ci-token".to_owned(),
                        key: "token".to_owned(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ]),
        security_context: Some(k8s_openapi::api::core::v1::SecurityContext {
            allow_privilege_escalation: Some(false),
            read_only_root_filesystem: Some(false),
            run_as_non_root: Some(true),
            run_as_user: Some(1000),
            capabilities: Some(k8s_openapi::api::core::v1::Capabilities {
                drop: Some(vec!["ALL".to_owned()]),
                ..Default::default()
            }),
            ..Default::default()
        }),
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
        init_containers: Some(vec![clone_container]),
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

#[cfg(test)]
mod tests {
    use super::*;
    use oya_ci_controller_kernel::{GateRun, GateRunSpec};

    const HEAD_SHA: &str = "ABCDEF1234567890ABCDEF1234567890ABCDEF12";
    const HEAD_SHA_LOWER: &str = "abcdef1234567890abcdef1234567890abcdef12";

    fn sample_spec() -> GateRunSpec {
        GateRunSpec {
            run: GateRun {
                pr_number: 42,
                head_sha: HEAD_SHA.to_owned(),
                delivery_id: format!("gate-run-pr42-{HEAD_SHA_LOWER}"),
                base_ref: "dev".to_owned(),
                repo: "oya-admin/oyatie".to_owned(),
            },
            image: "registry.local/rust-ci:dev".to_owned(),
            forge_clone_url: "http://forgejo.local/oya-admin/oyatie.git".to_owned(),
            active_deadline_seconds: 3600,
            ttl_seconds_after_finished: 86400,
            namespace: "oya-ci".to_owned(),
            runner_service_account: "oya-ci-gate-runner".to_owned(),
        }
    }

    fn pod_spec(job: &Job) -> &PodSpec {
        job.spec.as_ref().unwrap().template.spec.as_ref().unwrap()
    }

    fn main_gate_container(job: &Job) -> &Container {
        &pod_spec(job).containers[0]
    }

    fn clone_init_container(job: &Job) -> &Container {
        &pod_spec(job).init_containers.as_ref().unwrap()[0]
    }

    fn command_text(container: &Container) -> String {
        container.command.as_ref().unwrap().join("\n")
    }

    fn env_names(container: &Container) -> Vec<&str> {
        container
            .env
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|env| env.name.as_str())
            .collect()
    }

    #[test]
    fn build_gate_job_preserves_full_candidate_sha_in_name_and_labels() {
        let spec = sample_spec();
        let job = build_gate_job(&spec);
        let labels = job.metadata.labels.as_ref().unwrap();

        assert_eq!(
            job.metadata.name.as_deref(),
            Some("oya-ci-pr42-abcdef1234567890abcdef1234567890abcdef12")
        );
        assert!(job.metadata.name.as_ref().unwrap().len() <= 63);
        assert_eq!(
            labels.get(LABEL_CI_HEAD_SHA).map(String::as_str),
            Some(HEAD_SHA_LOWER)
        );
        assert_eq!(
            labels.get(LABEL_CI_PR_NUMBER).map(String::as_str),
            Some("42")
        );
        assert_eq!(
            labels.get(LABEL_CI_DELIVERY_ID).map(String::as_str),
            Some("gate-run-pr42-abcdef1234567890abcdef1234567890abcdef12")
        );
    }

    #[test]
    fn build_gate_job_keeps_clone_token_out_of_untrusted_gate_container() {
        let job = build_gate_job(&sample_spec());
        let main = main_gate_container(&job);
        let clone = clone_init_container(&job);
        let main_command = command_text(main);
        let clone_command = command_text(clone);

        assert!(!env_names(main).contains(&"FORGEJO_CLONE_TOKEN"));
        assert!(!main_command.contains("FORGEJO_CLONE_TOKEN"));
        assert!(!main_command.contains("http.extraHeader"));
        assert!(env_names(clone).contains(&"FORGEJO_CLONE_TOKEN"));
        assert!(clone_command.contains("http.extraHeader"));
        assert!(clone_command.contains("git -c \"$AUTH\" clone"));
    }

    #[test]
    fn build_gate_job_captures_trusted_targets_before_candidate_checkout() {
        let job = build_gate_job(&sample_spec());
        let command = command_text(main_gate_container(&job));
        let build_targets = command
            .find("buck2 targets //... | sort -u > /workspace/trusted-build-targets.txt")
            .unwrap();
        let test_targets = command
            .find("buck2 uquery 'kind(\".*test.*\", //...)' | sort -u > /workspace/trusted-test-targets.txt")
            .unwrap();
        let assert_build_targets = command
            .find("test -s /workspace/trusted-build-targets.txt")
            .unwrap();
        let assert_test_targets = command
            .find("test -s /workspace/trusted-test-targets.txt")
            .unwrap();
        let checkout = command.find("git checkout --detach ").unwrap();
        let build_run = command
            .find("xargs -a /workspace/trusted-build-targets.txt buck2 build")
            .unwrap();
        let test_run = command
            .find("xargs -a /workspace/trusted-test-targets.txt buck2 test")
            .unwrap();

        for snapshot_step in [
            build_targets,
            test_targets,
            assert_build_targets,
            assert_test_targets,
        ] {
            assert!(
                snapshot_step < checkout,
                "trusted target inventories must be fixed before candidate checkout"
            );
        }
        assert!(
            checkout < build_run && checkout < test_run,
            "candidate can only run against pre-captured trusted target inventories"
        );
    }

    #[test]
    fn build_gate_job_verifies_exact_pr_sha_before_checkout() {
        let job = build_gate_job(&sample_spec());
        let command = command_text(main_gate_container(&job));
        let resolve = command
            .find("resolved_sha=\"$(git rev-parse refs/remotes/origin/pr-42)\"")
            .unwrap();
        let exact_sha_check = command
            .find("test \"$resolved_sha\" = \"abcdef1234567890abcdef1234567890abcdef12\"")
            .unwrap();
        let checkout = command
            .find("git checkout --detach abcdef1234567890abcdef1234567890abcdef12")
            .unwrap();

        assert!(
            resolve < exact_sha_check && exact_sha_check < checkout,
            "the fetched PR ref must resolve to the exact candidate SHA before checkout"
        );
    }

    #[test]
    fn build_gate_job_disables_service_account_token_automount() {
        let job = build_gate_job(&sample_spec());
        let pod_spec = pod_spec(&job);

        assert_eq!(pod_spec.automount_service_account_token, Some(false));
        assert_eq!(
            pod_spec.service_account_name.as_deref(),
            Some("oya-ci-gate-runner")
        );
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
                // Conflict = already exists for the exact PR + full candidate
                // SHA identity encoded in GateRun::job_name().
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
