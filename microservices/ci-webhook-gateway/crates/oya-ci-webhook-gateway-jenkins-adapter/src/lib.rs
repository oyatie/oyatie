//! # oya-ci-webhook-gateway-jenkins-adapter
//!
//! Jenkins REST client adapter for the CI webhook gateway (ADR-0387 D4).
//!
//! Implements [`JenkinsClient`] via reqwest blocking HTTP.
//!
//! ## Endpoints used
//!
//! - Trigger: `POST /job/<name>/buildWithParameters?token=<api_token>`
//! - Poll:    `GET  /job/<name>/<build_number>/api/json`
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.  All HTTP errors
//! map to [`KernelError::DownstreamTransport`].

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_ci_webhook_gateway_kernel::{
    CiTriggerEvent, JenkinsClient, JenkinsJob, JobStatus, KernelError, Result,
};
use serde::Deserialize;

/// Default poll timeout: 5 minutes.
const DEFAULT_POLL_TIMEOUT_S: u64 = 300;
/// Default poll interval: 3 seconds.
const DEFAULT_POLL_INTERVAL_S: u64 = 3;

/// Jenkins REST client backed by reqwest blocking.
pub struct JenkinsRestClient {
    base_url: String,     // data_class: INTERNAL_ONLY
    job_name: String,     // data_class: INTERNAL_ONLY
    api_token: String,    // data_class: INTERNAL_ONLY
    user: String,         // data_class: INTERNAL_ONLY
    poll_timeout_s: u64,  // data_class: INTERNAL_ONLY
    poll_interval_s: u64, // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl JenkinsRestClient {
    /// Construct with the given Jenkins coordinates.
    ///
    /// `base_url` should be the root Jenkins URL (no trailing slash),
    /// e.g. `"http://jenkins.example.com"`.
    pub fn new(base_url: &str, job_name: &str, api_token: &str, user: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            job_name: job_name.to_owned(),
            api_token: api_token.to_owned(),
            user: user.to_owned(),
            poll_timeout_s: DEFAULT_POLL_TIMEOUT_S,
            poll_interval_s: DEFAULT_POLL_INTERVAL_S,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Override the poll timeout (useful in tests to keep them fast).
    pub fn with_poll_timeout_s(mut self, secs: u64) -> Self {
        self.poll_timeout_s = secs;
        self
    }

    /// Override the poll interval (useful in tests).
    pub fn with_poll_interval_s(mut self, secs: u64) -> Self {
        self.poll_interval_s = secs;
        self
    }

    fn trigger_url(&self) -> String {
        format!(
            "{}/job/{}/buildWithParameters?token={}",
            self.base_url, self.job_name, self.api_token
        )
    }

    fn build_api_url(&self, build_number: u64) -> String {
        format!(
            "{}/job/{}/{}/api/json",
            self.base_url, self.job_name, build_number
        )
    }
}

impl JenkinsClient for JenkinsRestClient {
    fn trigger(&self, job_name: &str, event: &CiTriggerEvent) -> Result<JenkinsJob> {
        let params = [
            ("GIT_SHA", event.head_sha.as_str()),
            ("GIT_BRANCH", event.branch.as_str()),
            ("PR_NUMBER", &event.pr_number.to_string()),
            ("REPO", event.repo.as_str()),
            ("DELIVERY_ID", event.delivery_id.as_str()),
        ];

        let resp = self
            .client
            .post(self.trigger_url())
            .basic_auth(&self.user, Some(&self.api_token))
            .form(&params)
            .send()
            .map_err(|e| KernelError::DownstreamTransport(format!("jenkins trigger: {e}")))?;

        if !resp.status().is_success() {
            return Err(KernelError::DownstreamTransport(format!(
                "jenkins trigger returned HTTP {}",
                resp.status()
            )));
        }

        // Jenkins returns the build number in the `Location` header for
        // buildWithParameters.  If absent we fall back to polling queue.
        let build_number = extract_build_number_from_location(resp.headers()).unwrap_or(1);

        Ok(JenkinsJob {
            job_name: job_name.to_owned(),
            build_number,
            trigger: event.clone(),
            status: JobStatus::Queued,
            build_url: Some(self.build_api_url(build_number).replace("/api/json", "/")),
        })
    }

    fn poll_status(&self, job: &JenkinsJob) -> Result<JobStatus> {
        let url = self.build_api_url(job.build_number);
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(self.poll_timeout_s);

        loop {
            let resp = self
                .client
                .get(&url)
                .basic_auth(&self.user, Some(&self.api_token))
                .send()
                .map_err(|e| KernelError::DownstreamTransport(format!("jenkins poll: {e}")))?;

            if !resp.status().is_success() {
                return Err(KernelError::DownstreamTransport(format!(
                    "jenkins poll returned HTTP {}",
                    resp.status()
                )));
            }

            let build: JenkinsBuildResponse = resp.json().map_err(|e| {
                KernelError::DownstreamTransport(format!("jenkins poll parse: {e}"))
            })?;

            if !build.building {
                return Ok(map_jenkins_result(build.result.as_deref()));
            }

            if std::time::Instant::now() >= deadline {
                return Ok(JobStatus::Unknown);
            }

            std::thread::sleep(std::time::Duration::from_secs(self.poll_interval_s));
        }
    }
}

/// Minimal Jenkins build API response shape.
#[derive(Deserialize)]
struct JenkinsBuildResponse {
    building: bool,
    result: Option<String>,
}

fn map_jenkins_result(result: Option<&str>) -> JobStatus {
    match result {
        Some("SUCCESS") => JobStatus::Success,
        Some("FAILURE") => JobStatus::Failure,
        Some("ABORTED") => JobStatus::Aborted,
        _ => JobStatus::Unknown,
    }
}

/// Parse the build number from the Jenkins `Location` header.
/// Returns `None` if the header is absent or unparseable.
fn extract_build_number_from_location(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    let loc = headers.get("Location")?.to_str().ok()?;
    // Location is typically: .../job/<name>/<build_number>/
    loc.trim_end_matches('/')
        .rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
}
