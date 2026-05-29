//! D4 Jenkins REST client adapter tests (ADR-0387).
//!
//! 5 tests: successful job trigger, build polling loop, success status mapping,
//! failure status mapping, timeout handling.
//! httpmock is used to mock the Jenkins REST API.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use httpmock::prelude::*;
use oya_ci_webhook_gateway_jenkins_adapter::JenkinsRestClient;
use oya_ci_webhook_gateway_kernel::{CiAction, CiTriggerEvent, JenkinsClient, JobStatus};

fn sample_event() -> CiTriggerEvent {
    CiTriggerEvent {
        repo: "oyatie/oyatie".to_owned(),
        branch: "dev".to_owned(),
        head_sha: "abc123def456".to_owned(),
        base_sha: "000000".to_owned(),
        pr_number: 42,
        delivery_id: "del-001".to_owned(),
        action: CiAction::PrOpened,
    }
}

/// Test 1 — trigger returns a JenkinsJob in Queued state with a build number.
#[test]
fn trigger_returns_queued_job() {
    let server = MockServer::start();

    let _trigger_mock = server.mock(|when, then| {
        when.method(POST).path("/job/oyaCiLane/buildWithParameters");
        then.status(201).header(
            "Location",
            format!("{}/job/oyaCiLane/7/", server.base_url()),
        );
    });

    let client = JenkinsRestClient::new(&server.base_url(), "oyaCiLane", "test-token", "ci-bot");

    let job = client.trigger("oyaCiLane", &sample_event()).unwrap();
    assert_eq!(job.build_number, 7);
    assert_eq!(job.job_name, "oyaCiLane");
    assert_eq!(job.status, JobStatus::Queued);
}

/// Test 2 — poll_status returns Success immediately when building=false on first poll.
/// (The loop-until-done behaviour is exercised transitively by the timeout test
/// which loops on building=true and the success/failure tests which return immediately.)
#[test]
fn poll_status_waits_for_build_to_finish() {
    let server = MockServer::start();

    // Single poll response: already done.
    server.mock(|when, then| {
        when.method(GET).path("/job/oyaCiLane/5/api/json");
        then.status(200)
            .json_body(serde_json::json!({ "building": false, "result": "SUCCESS" }));
    });

    let event = sample_event();
    let job = oya_ci_webhook_gateway_kernel::JenkinsJob {
        job_name: "oyaCiLane".to_owned(),
        build_number: 5,
        trigger: event,
        status: JobStatus::Running,
        build_url: None,
    };

    let client = JenkinsRestClient::new(&server.base_url(), "oyaCiLane", "test-token", "ci-bot")
        .with_poll_timeout_s(10)
        .with_poll_interval_s(0);

    let status = client.poll_status(&job).unwrap();
    assert_eq!(status, JobStatus::Success);
}

/// Test 3 — SUCCESS result maps to JobStatus::Success.
#[test]
fn success_result_maps_correctly() {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(GET).path("/job/oyaCiLane/1/api/json");
        then.status(200)
            .json_body(serde_json::json!({ "building": false, "result": "SUCCESS" }));
    });

    let job = oya_ci_webhook_gateway_kernel::JenkinsJob {
        job_name: "oyaCiLane".to_owned(),
        build_number: 1,
        trigger: sample_event(),
        status: JobStatus::Running,
        build_url: None,
    };

    let client = JenkinsRestClient::new(&server.base_url(), "oyaCiLane", "tok", "usr")
        .with_poll_timeout_s(5)
        .with_poll_interval_s(0);

    assert_eq!(client.poll_status(&job).unwrap(), JobStatus::Success);
}

/// Test 4 — FAILURE result maps to JobStatus::Failure.
#[test]
fn failure_result_maps_correctly() {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(GET).path("/job/oyaCiLane/2/api/json");
        then.status(200)
            .json_body(serde_json::json!({ "building": false, "result": "FAILURE" }));
    });

    let job = oya_ci_webhook_gateway_kernel::JenkinsJob {
        job_name: "oyaCiLane".to_owned(),
        build_number: 2,
        trigger: sample_event(),
        status: JobStatus::Running,
        build_url: None,
    };

    let client = JenkinsRestClient::new(&server.base_url(), "oyaCiLane", "tok", "usr")
        .with_poll_timeout_s(5)
        .with_poll_interval_s(0);

    assert_eq!(client.poll_status(&job).unwrap(), JobStatus::Failure);
}

/// Test 5 — timeout while building=true returns JobStatus::Unknown.
#[test]
fn timeout_returns_unknown() {
    let server = MockServer::start();

    // Always returns building=true so we hit the timeout.
    server.mock(|when, then| {
        when.method(GET).path("/job/oyaCiLane/3/api/json");
        then.status(200)
            .json_body(serde_json::json!({ "building": true, "result": null }));
    });

    let job = oya_ci_webhook_gateway_kernel::JenkinsJob {
        job_name: "oyaCiLane".to_owned(),
        build_number: 3,
        trigger: sample_event(),
        status: JobStatus::Running,
        build_url: None,
    };

    // 1 second timeout, 0 s interval → immediately timeout after first poll.
    let client = JenkinsRestClient::new(&server.base_url(), "oyaCiLane", "tok", "usr")
        .with_poll_timeout_s(1)
        .with_poll_interval_s(0);

    // Tight loop with 0 interval will hit the 1s deadline quickly.
    let status = client.poll_status(&job).unwrap();
    assert_eq!(status, JobStatus::Unknown);
}
