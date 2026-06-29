//! oya-ci-controller binary entry point.
//!
//! Reads configuration from environment variables:
//!
//! | Variable                               | Default                                                                  | Description                                    |
//! |----------------------------------------|--------------------------------------------------------------------------|------------------------------------------------|
//! | `OYA_CI_CONTROLLER_LISTEN_ADDR`        | `0.0.0.0:8081`                                                           | Bind address for healthz/metrics/gate-run      |
//! | `OYA_CI_CONTROLLER_NAMESPACE`          | `oya-ci`                                                                 | Namespace to watch Jobs in + spawn Jobs into   |
//! | `OYA_CI_CONTROLLER_REPO_OWNER`         | `jason931225`                                                            | GitHub repo owner (forge of record)            |
//! | `OYA_CI_CONTROLLER_REPO_NAME`          | `oyatie`                                                                 | GitHub repo name (forge of record)             |
//! | `GITHUB_CI_TOKEN`                      | (required)                                                               | GitHub token for status posting (controller-only) |
//! | `OYA_CI_GATE_RUN_BEARER`               | (required)                                                               | Fail-closed bearer for POST /gate-run (keystone-1) |
//! | `OYA_CI_CONTROLLER_GRACE_CYCLES`       | `12`                                                                     | Waiting-pod-reason grace threshold             |
//! | `OYA_CI_GATE_IMAGE`                    | `registry.oya-registry.svc.cluster.local:5000/rust-ci:dev`               | Rust-CI image for gate runner Pods             |
//! | `OYA_CI_GATE_FORGE_CLONE_URL`          | `https://github.com/jason931225/oyatie.git`                              | Git clone URL for gate Job init container      |
//! | `OYA_CI_GATE_ACTIVE_DEADLINE_SECS`     | `3600`                                                                   | Gate Job active deadline (seconds)             |
//! | `OYA_CI_GATE_TTL_AFTER_FINISHED_SECS`  | `86400`                                                                  | Gate Job TTL after finished for GC (seconds)   |
//! | `OYA_CI_GATE_RUNNER_SA`                | `oya-ci-gate-runner`                                                     | Low-privilege SA for gate runner Pods          |
//! | `OYA_CI_STATUS_API_BASE_URL`             | unset                                                                    | Optional base URL for `/gate-runs/<run_id>` debug links |

use oya_ci_controller_app::{
    AllowVerifiedTriggerAuthz, CiTriggerAuthenticator, CiTriggerAuthz,
    ConfiguredBearerCiTriggerAuthenticator, ControllerState, GateSpecConfig, ServerState,
    StreamExt, build_router, run_controller,
};
use oya_ci_controller_github_adapter::GitHubCommitStatusPoster;
use oya_ci_controller_k8s_adapter::K8sJobSpawner;
use oya_ci_controller_kernel::CommitStatusPoster;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned()))
        .init();

    let listen_addr = env_or("OYA_CI_CONTROLLER_LISTEN_ADDR", "0.0.0.0:8081");
    let namespace = env_or("OYA_CI_CONTROLLER_NAMESPACE", "oya-ci");
    // Forge of record (D-FORGE): GitHub interim until the Sapling-inspired
    // bespoke SCM. The controller posts commit statuses to GitHub only.
    let repo_owner = env_or("OYA_CI_CONTROLLER_REPO_OWNER", "jason931225");
    let repo_name = env_or("OYA_CI_CONTROLLER_REPO_NAME", "oyatie");
    let grace_cycles: u32 = env_or("OYA_CI_CONTROLLER_GRACE_CYCLES", "12")
        .parse()
        .unwrap_or(12);

    // Gate Job spawn configuration (for POST /gate-run — plank role).
    let gate_image = env_or(
        "OYA_CI_GATE_IMAGE",
        "registry.oya-registry.svc.cluster.local:5000/rust-ci:dev",
    );
    let gate_forge_clone_url = env_or(
        "OYA_CI_GATE_FORGE_CLONE_URL",
        "https://github.com/jason931225/oyatie.git",
    );
    let gate_active_deadline_secs: i64 = env_or("OYA_CI_GATE_ACTIVE_DEADLINE_SECS", "3600")
        .parse()
        .unwrap_or(3600);
    let gate_ttl_after_finished_secs: i32 = env_or("OYA_CI_GATE_TTL_AFTER_FINISHED_SECS", "86400")
        .parse()
        .unwrap_or(86400);
    let gate_runner_sa = env_or("OYA_CI_GATE_RUNNER_SA", "oya-ci-gate-runner");
    let status_api_base_url = env_optional("OYA_CI_STATUS_API_BASE_URL");

    // Build kube client (uses in-cluster SA token when deployed; falls back to
    // kubeconfig for local dev).
    let kube_client = kube::Client::try_default().await.unwrap_or_else(|e| {
        eprintln!("failed to build kube client: {e}");
        std::process::exit(1);
    });

    // Select the commit-status producer. GitHub is the forge of record
    // (D-FORGE; GitHub interim until the Sapling-inspired bespoke SCM); the
    // GITHUB_CI_TOKEN is read here and stays controller-only — it is never
    // threaded into the gate Job / runner environment.
    let github_token = env_required("GITHUB_CI_TOKEN");
    info!(
        forge = "github",
        "using GitHub commit-status producer (oya-ci-required)"
    );
    let status_poster: Arc<dyn CommitStatusPoster> = Arc::new(GitHubCommitStatusPoster::new(
        &repo_owner,
        &repo_name,
        &github_token,
    ));

    let controller_state = ControllerState {
        client: kube_client.clone(),
        status_poster,
        namespace: namespace.clone(),
        grace_cycles,
        status_api_base_url: status_api_base_url.clone(),
    };

    // Fail-closed bearer for POST /gate-run (keystone-1). Required at startup,
    // exactly like GITHUB_CI_TOKEN: the controller refuses to start without it,
    // so it can never serve an unauthenticated Job-spawn endpoint.
    let gate_run_bearer = env_required("OYA_CI_GATE_RUN_BEARER");
    let authenticator: Arc<dyn CiTriggerAuthenticator> =
        Arc::new(ConfiguredBearerCiTriggerAuthenticator::new(gate_run_bearer));
    let authz: Arc<dyn CiTriggerAuthz> = Arc::new(AllowVerifiedTriggerAuthz);

    // K8sJobSpawner — used by POST /gate-run to create gate Jobs.
    let job_spawner = Arc::new(K8sJobSpawner::new(kube_client.clone()));

    let gate_spec_config = GateSpecConfig {
        image: gate_image,
        forge_clone_url: gate_forge_clone_url,
        active_deadline_seconds: gate_active_deadline_secs,
        ttl_seconds_after_finished: gate_ttl_after_finished_secs,
        namespace: namespace.clone(),
        runner_service_account: gate_runner_sa,
        repo: format!("{repo_owner}/{repo_name}"),
        status_api_base_url,
    };

    let server_state = ServerState {
        controller_namespace: namespace.clone(),
        job_spawner,
        gate_spec_config,
        authenticator,
        authz,
        status_client: Some(kube_client.clone()),
        status_grace_cycles: grace_cycles,
    };

    info!(listen_addr = %listen_addr, namespace = %namespace, "oya-ci-controller starting");

    // Run health/metrics/gate-run server and controller concurrently.
    let app = build_router(server_state);
    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to bind {listen_addr}: {e}");
            std::process::exit(1);
        });

    tokio::select! {
        _ = axum::serve(listener, app) => {
            eprintln!("health server exited");
        }
        _ = run_controller(controller_state) => {
            eprintln!("controller exited");
        }
    }
}

fn env_required(key: &str) -> String {
    match std::env::var(key) {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            eprintln!("required env var {key} is not set or is empty");
            std::process::exit(1);
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn env_optional(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
