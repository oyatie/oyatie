//! # oya-ci-controller-github-adapter
//!
//! GitHub commit-status poster for the oya-ci controller (the `oya-ci-required`
//! producer).
//!
//! Implements [`CommitStatusPoster`] via reqwest blocking HTTP. The HTTP shape
//! is lifted from the proven oya-ci-webhook-gateway-github-adapter (ADR-0387 D5).
//! Forge-of-record = GitHub (D2/D-FORGE; GitHub dropped).
//!
//! ## Endpoint
//!
//! `POST https://api.github.com/repos/<owner>/<repo>/statuses/<sha>`
//!
//! ## Headers
//!
//! - `Authorization: Bearer <token>`
//! - `X-GitHub-Api-Version: 2022-11-28`
//! - `Accept: application/vnd.github+json`
//! - `User-Agent: oya-ci-controller` (GitHub rejects UA-less requests)
//!
//! Accepts any 2xx (`is_success`; GitHub returns 201 Created). Any other status
//! -> `KernelError::DownstreamTransport`.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::time::Duration;

use ci_controller_kernel::{
    CommitState, CommitStatusPoster, GitHubPrincipal, KernelError, REVIEW_CONTEXT, Result,
    ReviewAdmissionInput, ReviewAdmissionPacket, ReviewAdmissionPolicy, ReviewAdmissionProducer,
    ReviewEvidence, ReviewVerdict, admit_review,
};
use reqwest::{Method, blocking::RequestBuilder};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// GitHub Statuses API version header value.
const GITHUB_API_VERSION: &str = "2022-11-28";
const GITHUB_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const REVIEWS_PER_PAGE: usize = 100;
const MAX_REVIEW_PAGES: usize = 100;
/// Default GitHub API base URL.
const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_WEB_BASE: &str = "https://github.com";
/// Forge of record (D2): GitHub interim until the Sapling-inspired bespoke SCM.
const DEFAULT_REPO_OWNER: &str = "jason931225";
const DEFAULT_REPO_NAME: &str = "oyatie";

/// GitHub commit-status poster backed by reqwest blocking.
pub struct GitHubCommitStatusPoster {
    repo_owner: String,   // data_class: INTERNAL_ONLY
    repo_name: String,    // data_class: INTERNAL_ONLY
    github_token: String, // data_class: INTERNAL_ONLY  (controller crier token ONLY; never to runner)
    api_base: String,     // data_class: INTERNAL_ONLY
    web_base: String,     // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
    request_timeout: Duration,
}

impl GitHubCommitStatusPoster {
    /// Construct with the given repository coordinates and GitHub token.
    pub fn new(repo_owner: &str, repo_name: &str, github_token: &str) -> Self {
        Self {
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            github_token: github_token.to_owned(),
            api_base: GITHUB_API_BASE.to_owned(),
            web_base: GITHUB_WEB_BASE.to_owned(),
            client: reqwest::blocking::Client::new(),
            request_timeout: GITHUB_REQUEST_TIMEOUT,
        }
    }

    /// Construct with the forge-of-record defaults (jason931225/oyatie).
    pub fn with_defaults(token: &str) -> Self {
        Self::new(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME, token)
    }

    /// Override the API base URL (useful in tests to point at a local test server).
    pub fn with_api_base(mut self, base: &str) -> Self {
        self.api_base = base.trim_end_matches('/').to_owned();
        self
    }

    /// Override the GitHub web base URL (useful for an explicit forge mirror).
    pub fn with_web_base(mut self, base: &str) -> Self {
        self.web_base = base.trim_end_matches('/').to_owned();
        self
    }

    /// Override the finite timeout applied to every GitHub HTTP request.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    fn authed_request(&self, method: Method, url: &str) -> RequestBuilder {
        self.client
            .request(method, url)
            .bearer_auth(&self.github_token)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "oya-ci-controller")
            .timeout(self.request_timeout)
    }

    fn statuses_url(&self, sha: &str) -> String {
        format!(
            "{}/repos/{}/{}/statuses/{}",
            self.api_base, self.repo_owner, self.repo_name, sha
        )
    }

    fn pull_url(&self, pr_number: u64) -> String {
        format!(
            "{}/repos/{}/{}/pulls/{pr_number}",
            self.api_base, self.repo_owner, self.repo_name
        )
    }

    fn reviews_url(&self, pr_number: u64) -> String {
        format!("{}/reviews", self.pull_url(pr_number))
    }

    fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self
            .authed_request(Method::GET, url)
            .send()
            .map_err(|error| {
                KernelError::DownstreamTransport(format!("github review fetch: {error}"))
            })?;

        if !response.status().is_success() {
            return Err(KernelError::DownstreamTransport(format!(
                "github review fetch returned HTTP {}",
                response.status()
            )));
        }

        response.json().map_err(|error| {
            KernelError::DownstreamTransport(format!("github review response decode: {error}"))
        })
    }

    /// Fetch trusted GitHub PR/review API data, validate it against the expected
    /// candidate head, and post the `oya-pr-review` commit status.
    ///
    /// Success carries the exact durable GitHub review URL. Invalid or missing
    /// evidence posts a terminal failure on the expected head before returning
    /// the validation error, so a wired fan-in fails closed.
    pub fn produce_review_admission_status(
        &self,
        pr_number: u64,
        expected_head_sha: &str,
        policy: &ReviewAdmissionPolicy,
        producer: &ReviewAdmissionProducer,
        evaluated_at_unix_s: i64,
    ) -> Result<ReviewAdmissionPacket> {
        let result = self
            .fetch_review_admission(
                pr_number,
                expected_head_sha,
                policy,
                producer,
                evaluated_at_unix_s,
            )
            .and_then(|packet| {
                self.verify_current_pull_head(pr_number, expected_head_sha)?;
                Ok(packet)
            });
        match result {
            Ok(packet) => {
                self.post(
                    expected_head_sha,
                    CommitState::Success,
                    REVIEW_CONTEXT,
                    &format!("oya-pr-review approved by {}", packet.reviewer.login),
                    Some(&packet.evidence_url),
                )?;
                Ok(packet)
            }
            Err(error) => {
                let state = match &error {
                    KernelError::InvalidInput(_) => CommitState::Failure,
                    KernelError::DownstreamTransport(_) => CommitState::Error,
                };
                let description = format!("oya-pr-review rejected: {error}");
                if let Err(post_error) =
                    self.post(expected_head_sha, state, REVIEW_CONTEXT, &description, None)
                {
                    return Err(KernelError::DownstreamTransport(format!(
                        "{error}; additionally failed to post {REVIEW_CONTEXT}: {post_error}"
                    )));
                }
                Err(error)
            }
        }
    }

    fn fetch_review_admission(
        &self,
        pr_number: u64,
        expected_head_sha: &str,
        policy: &ReviewAdmissionPolicy,
        producer: &ReviewAdmissionProducer,
        evaluated_at_unix_s: i64,
    ) -> Result<ReviewAdmissionPacket> {
        let pull: GitHubPullResponse = self.get_json(&self.pull_url(pr_number))?;
        if pull.number != pr_number {
            return Err(KernelError::InvalidInput(format!(
                "review admission PR mismatch: expected {pr_number}, observed {}",
                pull.number
            )));
        }

        // Reject a moved PR head before fetching review evidence. An approval
        // for any other SHA can never satisfy this candidate status.
        if pull.head.sha != expected_head_sha {
            return admit_review(&ReviewAdmissionInput {
                pr_number,
                expected_head_sha: expected_head_sha.to_owned(),
                observed_head_sha: pull.head.sha,
                author: github_principal(pull.user),
                policy: policy.clone(),
                evaluated_at_unix_s,
                producer: producer.clone(),
                reviews: Vec::new(),
            });
        }

        let reviews = self.fetch_all_reviews(pr_number)?;

        let mut invalid_review_url_present = false;
        let evidence = reviews
            .into_iter()
            .map(|review| {
                let evidence_url = review.html_url.unwrap_or_default();
                let evidence_url = if self.is_exact_review_url(&evidence_url, pr_number, review.id)
                {
                    evidence_url
                } else {
                    invalid_review_url_present = true;
                    String::new()
                };
                ReviewEvidence {
                    review_id: review.id,
                    head_sha: review.commit_id.unwrap_or_default(),
                    reviewer: review
                        .user
                        .map(github_principal)
                        .unwrap_or(GitHubPrincipal {
                            id: 0,
                            account_type: ci_controller_kernel::GitHubAccountType::User,
                            login: String::new(),
                        }),
                    verdict: github_review_verdict(&review.state),
                    evidence_url,
                }
            })
            .collect::<Vec<_>>();

        let result = admit_review(&ReviewAdmissionInput {
            pr_number,
            expected_head_sha: expected_head_sha.to_owned(),
            observed_head_sha: pull.head.sha,
            author: github_principal(pull.user),
            policy: policy.clone(),
            evaluated_at_unix_s,
            producer: producer.clone(),
            reviews: evidence,
        });
        match result {
            Err(KernelError::InvalidInput(message))
                if invalid_review_url_present
                    && message == "approved review is missing a durable HTTP(S) evidence URL" =>
            {
                Err(KernelError::InvalidInput(
                    "review evidence URL does not bind the configured repository, PR, and review"
                        .to_owned(),
                ))
            }
            other => other,
        }
    }

    fn verify_current_pull_head(&self, pr_number: u64, expected_head_sha: &str) -> Result<()> {
        let pull: GitHubPullResponse = self.get_json(&self.pull_url(pr_number))?;
        if pull.number != pr_number || pull.head.sha != expected_head_sha {
            return Err(KernelError::InvalidInput(
                "review admission final PR-head readback changed before status emission".to_owned(),
            ));
        }
        Ok(())
    }

    fn is_exact_review_url(&self, evidence_url: &str, pr_number: u64, review_id: u64) -> bool {
        evidence_url
            == format!(
                "{}/{}/{}/pull/{pr_number}#pullrequestreview-{review_id}",
                self.web_base, self.repo_owner, self.repo_name
            )
    }

    fn fetch_all_reviews(&self, pr_number: u64) -> Result<Vec<GitHubReviewResponse>> {
        let mut reviews = Vec::new();
        for page in 1..=MAX_REVIEW_PAGES {
            let url = format!(
                "{}?per_page={REVIEWS_PER_PAGE}&page={page}",
                self.reviews_url(pr_number)
            );
            let response = self
                .authed_request(Method::GET, &url)
                .send()
                .map_err(|error| {
                    KernelError::DownstreamTransport(format!("github review fetch: {error}"))
                })?
                .error_for_status()
                .map_err(|error| {
                    KernelError::DownstreamTransport(format!("github review fetch: {error}"))
                })?;

            let link_header = response
                .headers()
                .get("Link")
                .map(|value| {
                    value.to_str().map(str::to_owned).map_err(|error| {
                        KernelError::DownstreamTransport(format!(
                            "github review pagination header decode: {error}"
                        ))
                    })
                })
                .transpose()?;
            let has_next = link_header
                .as_deref()
                .is_some_and(header_has_next_link_relation);
            let page_reviews: Vec<GitHubReviewResponse> = response.json().map_err(|error| {
                KernelError::DownstreamTransport(format!("github review response decode: {error}"))
            })?;

            if page_reviews.len() >= REVIEWS_PER_PAGE && !has_next {
                return Err(KernelError::DownstreamTransport(
                    "github review pagination completeness cannot be proven".to_owned(),
                ));
            }
            if has_next && page_reviews.is_empty() {
                return Err(KernelError::DownstreamTransport(
                    "github review pagination advertised an empty next page".to_owned(),
                ));
            }
            reviews.extend(page_reviews);

            if !has_next {
                return Ok(reviews);
            }
        }

        Err(KernelError::DownstreamTransport(format!(
            "github review pagination exceeded {MAX_REVIEW_PAGES} pages"
        )))
    }
}

fn header_has_next_link_relation(header: &str) -> bool {
    let mut in_uri = false;
    let mut in_quotes = false;
    let mut escaped = false;
    let mut start = 0;

    for (index, character) in header.char_indices() {
        match character {
            '\\' if in_quotes => escaped = !escaped,
            '"' if !escaped => in_quotes = !in_quotes,
            '<' if !in_quotes => in_uri = true,
            '>' if !in_quotes => in_uri = false,
            ',' if !in_quotes && !in_uri => {
                if link_relation_is_next(&header[start..index]) {
                    return true;
                }
                start = index + character.len_utf8();
            }
            _ => escaped = false,
        }
    }

    link_relation_is_next(&header[start..])
}

fn link_relation_is_next(link: &str) -> bool {
    let Some((_, parameters)) = link.split_once('>') else {
        return false;
    };

    let mut in_quotes = false;
    let mut escaped = false;
    let mut start = 0;
    for (index, character) in parameters.char_indices() {
        match character {
            '\\' if in_quotes => escaped = !escaped,
            '"' if !escaped => in_quotes = !in_quotes,
            ';' if !in_quotes => {
                if relation_parameter_is_next(&parameters[start..index]) {
                    return true;
                }
                start = index + character.len_utf8();
            }
            _ => escaped = false,
        }
    }

    relation_parameter_is_next(&parameters[start..])
}

fn relation_parameter_is_next(parameter: &str) -> bool {
    let Some((name, value)) = parameter.trim().split_once('=') else {
        return false;
    };

    name.trim().eq_ignore_ascii_case("rel")
        && value
            .trim()
            .trim_matches('"')
            .split_ascii_whitespace()
            .any(|relation| relation.eq_ignore_ascii_case("next"))
}

impl CommitStatusPoster for GitHubCommitStatusPoster {
    fn post(
        &self,
        sha: &str,
        state: CommitState,
        context: &str,
        description: &str,
        target_url: Option<&str>,
    ) -> Result<()> {
        // Truncate description to 240 Unicode scalar values (GitHub / GitHub
        // limit). Byte-slicing is unsafe on multibyte boundaries; use char
        // indices to find the correct byte offset.
        let description: &str = if description.chars().count() > 240 {
            let byte_end = description
                .char_indices()
                .nth(240)
                .map(|(i, _)| i)
                .unwrap_or(description.len());
            &description[..byte_end]
        } else {
            description
        };

        let body = GitHubStatusBody {
            state: state.as_str().to_owned(),
            context: context.to_owned(),
            description: description.to_owned(),
            target_url: target_url.map(ToOwned::to_owned),
        };

        let statuses_url = self.statuses_url(sha);
        let resp = self
            .authed_request(Method::POST, &statuses_url)
            .json(&body)
            .send()
            .map_err(|e| KernelError::DownstreamTransport(format!("github status post: {e}")))?;

        // GitHub returns 201 Created on success.
        if resp.status().is_success() {
            return Ok(());
        }

        Err(KernelError::DownstreamTransport(format!(
            "github status returned HTTP {}",
            resp.status()
        )))
    }
}

/// JSON body for `POST /repos/<owner>/<repo>/statuses/<sha>`.
#[derive(Serialize)]
struct GitHubStatusBody {
    state: String,
    context: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubPullResponse {
    number: u64,
    user: GitHubUser,
    head: GitHubHead,
}

#[derive(Deserialize)]
struct GitHubHead {
    sha: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: u64,
    login: String,
    #[serde(rename = "type")]
    account_type: ci_controller_kernel::GitHubAccountType,
}

#[derive(Deserialize)]
struct GitHubReviewResponse {
    id: u64,
    state: String,
    commit_id: Option<String>,
    html_url: Option<String>,
    user: Option<GitHubUser>,
}

fn github_review_verdict(state: &str) -> ReviewVerdict {
    match state {
        "APPROVED" => ReviewVerdict::Approved,
        "CHANGES_REQUESTED" => ReviewVerdict::ChangesRequested,
        "DISMISSED" => ReviewVerdict::Dismissed,
        _ => ReviewVerdict::Commented,
    }
}

fn github_principal(user: GitHubUser) -> GitHubPrincipal {
    GitHubPrincipal {
        id: user.id,
        account_type: user.account_type,
        login: user.login,
    }
}
