//! # oya-ci-tide-forgejo-adapter
//!
//! Forgejo API client adapter for the oya-ci tide component.
//!
//! Implements [`ForgejoClient`] via reqwest blocking HTTP.
//!
//! ## Endpoints consumed
//!
//! - `GET /api/v1/repos/<owner>/<repo>/pulls?state=open&base=<branch>&limit=50&page=N`
//! - `GET /api/v1/repos/<owner>/<repo>/commits/<sha>/statuses?limit=50&page=N`
//! - `GET /api/v1/repos/<owner>/<repo>/pulls/<number>/reviews?limit=50&page=N`
//! - `GET /api/v1/repos/<owner>/<repo>/pulls/<number>`
//! - `POST /api/v1/repos/<owner>/<repo>/pulls/<number>/merge`
//!
//! Pagination follows Forgejo `Link: rel="next"` first and falls back to
//! total-count metadata when available, so short intermediate pages do not hide
//! eligible PRs, statuses, or stale-review blockers.
//!
//! ## Authentication
//!
//! `Authorization: token <OYA_FORGEJO_TOKEN>` — token read from env at
//! construction time via [`ForgejoHttpClient::from_config`]. Never hardcoded.
//!
//! ## Pattern
//!
//! Lifted from `oya-ci-controller-forgejo-adapter`: same reqwest blocking
//! client, same `Authorization: token` header, same 200/201/204 acceptance,
//! same `KernelError::DownstreamTransport`-equivalent error mapping.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_ci_tide_kernel::{
    CommitStatusState, ForgejoClient, MergeMethod, PullRequest, Result, Review, ReviewState,
    TideError,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;

const PAGE_LIMIT: u32 = 50;
const MAX_PAGES: u32 = 200;

// ---------------------------------------------------------------------------
// ForgejoHttpClient
// ---------------------------------------------------------------------------

/// Forgejo API client backed by reqwest blocking HTTP.
pub struct ForgejoHttpClient {
    api_base: String,   // data_class: INTERNAL_ONLY
    repo_owner: String, // data_class: INTERNAL_ONLY
    repo_name: String,  // data_class: INTERNAL_ONLY
    token: String,      // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl ForgejoHttpClient {
    /// Construct with explicit values (used in tests and from config).
    pub fn new(api_base: &str, repo_owner: &str, repo_name: &str, token: &str) -> Self {
        Self {
            api_base: api_base.trim_end_matches('/').to_owned(),
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            token: token.to_owned(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Construct from a [`TideConfig`] + resolved token.
    pub fn from_config(config: &oya_ci_tide_kernel::TideConfig, token: &str) -> Self {
        Self::new(
            &config.forgejo_base_url,
            &config.repo_owner,
            &config.repo_name,
            token,
        )
    }

    fn repo_url(&self) -> String {
        format!(
            "{}/api/v1/repos/{}/{}",
            self.api_base, self.repo_owner, self.repo_name
        )
    }

    fn auth_header(&self) -> String {
        format!("token {}", self.token)
    }

    fn resolve_page_url(&self, page_url: &str) -> String {
        if page_url.starts_with("http://") || page_url.starts_with("https://") {
            return page_url.to_owned();
        }
        if page_url.starts_with('/') {
            return format!("{}{}", self.api_base, page_url);
        }
        page_url.to_owned()
    }

    fn fetch_all_pages<T>(&self, initial_url: String, operation: &str) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let mut next_url = initial_url.clone();
        let mut fallback_page = 2u32;
        let mut all_items = Vec::new();

        for _ in 0..MAX_PAGES {
            let page = self.fetch_page::<T>(&next_url, operation)?;
            let fetched = page.items.len();
            all_items.extend(page.items);

            if let Some(link_next) = page.next_url {
                next_url = self.resolve_page_url(&link_next);
                continue;
            }

            if page
                .total_count
                .is_some_and(|total_count| all_items.len() < total_count)
                && fetched > 0
            {
                next_url = with_page(&initial_url, fallback_page);
                fallback_page += 1;
                continue;
            }

            return Ok(all_items);
        }

        Err(TideError::Downstream(format!(
            "{operation} pagination exceeded {MAX_PAGES} pages"
        )))
    }

    fn fetch_page<T>(&self, url: &str, operation: &str) -> Result<Page<T>>
    where
        T: DeserializeOwned,
    {
        let resp = self
            .client
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| TideError::Downstream(format!("{operation} GET: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(TideError::Downstream(format!(
                "{operation} returned HTTP {status}"
            )));
        }

        let next_url = resp
            .headers()
            .get(reqwest::header::LINK)
            .and_then(|value| value.to_str().ok())
            .and_then(parse_next_link_header);
        let total_count = parse_total_count(resp.headers());
        let items: Vec<T> = resp
            .json()
            .map_err(|e| TideError::Downstream(format!("{operation} decode: {e}")))?;

        Ok(Page {
            items,
            next_url,
            total_count,
        })
    }
}

struct Page<T> {
    items: Vec<T>,
    next_url: Option<String>,
    total_count: Option<usize>,
}

// ---------------------------------------------------------------------------
// Wire-format types (Forgejo JSON shapes)
// ---------------------------------------------------------------------------

/// Forgejo pull-request list item (projected fields only).
#[derive(Debug, Deserialize)]
struct ForgejoPr {
    number: u64,
    title: String,
    head: ForgejoPrRef,
    base: ForgejoPrRef,
    /// `null` while Forgejo is still computing, `true`/`false` otherwise.
    mergeable: Option<bool>,
    #[serde(default)]
    labels: Vec<ForgejoLabel>,
}

#[derive(Debug, Deserialize)]
struct ForgejoPrRef {
    sha: String,
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct ForgejoLabel {
    name: String,
}

/// Forgejo commit-status item.
#[derive(Debug, Deserialize)]
struct ForgejoStatus {
    context: String,
    state: String,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
}

/// Forgejo review item.
#[derive(Debug, Deserialize)]
struct ForgejoReview {
    user: ForgejoUser,
    state: String,
    #[serde(default)]
    id: Option<u64>,
    #[serde(default, alias = "submitted")]
    submitted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ForgejoUser {
    login: String,
}

/// Forgejo merge request body (`POST /pulls/<n>/merge`).
#[derive(serde::Serialize)]
struct ForgejoMergeBody {
    #[serde(rename = "Do")]
    do_method: String,
    merge_message_field: String,
}

// ---------------------------------------------------------------------------
// ForgejoClient impl
// ---------------------------------------------------------------------------

impl ForgejoClient for ForgejoHttpClient {
    fn list_open_pulls(&self, base_branch: &str) -> Result<Vec<PullRequest>> {
        // Branch names in this codebase (e.g. "dev") contain only characters
        // that are safe in a query-string value; no percent-encoding needed.
        let url = format!(
            "{}/pulls?state=open&base={}&limit={PAGE_LIMIT}&page=1",
            self.repo_url(),
            base_branch
        );
        let items: Vec<ForgejoPr> = self.fetch_all_pages(url, "list_open_pulls")?;
        Ok(items.into_iter().map(pr_from_wire).collect())
    }

    fn get_commit_status(&self, sha: &str, required_context: &str) -> Result<CommitStatusState> {
        let url = format!(
            "{}/commits/{}/statuses?limit={PAGE_LIMIT}&page=1",
            self.repo_url(),
            sha
        );
        let statuses: Vec<ForgejoStatus> = self.fetch_all_pages(url, "get_commit_status")?;

        let found = statuses
            .iter()
            .enumerate()
            .filter(|(_, status)| status.context == required_context)
            .max_by(|left, right| compare_status_recency(left, right));
        Ok(match found {
            Some((_, status)) => CommitStatusState::from_str(&status.state),
            None => CommitStatusState::Missing,
        })
    }

    fn list_reviews(&self, pr_number: u64) -> Result<Vec<Review>> {
        let url = format!(
            "{}/pulls/{}/reviews?limit={PAGE_LIMIT}&page=1",
            self.repo_url(),
            pr_number
        );
        let items: Vec<ForgejoReview> = self.fetch_all_pages(url, "list_reviews")?;

        Ok(items
            .into_iter()
            .enumerate()
            .map(|(index, r)| Review {
                reviewer: r.user.login,
                state: ReviewState::from_str(&r.state),
                submitted_at: r.submitted_at,
                id: r.id,
                api_order: index as u64,
            })
            .collect())
    }

    fn get_pull(&self, pr_number: u64) -> Result<PullRequest> {
        let url = format!("{}/pulls/{}", self.repo_url(), pr_number);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| TideError::Downstream(format!("get_pull GET: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(TideError::Downstream(format!(
                "get_pull returned HTTP {status}"
            )));
        }

        let pr: ForgejoPr = resp
            .json()
            .map_err(|e| TideError::Downstream(format!("get_pull decode: {e}")))?;

        Ok(pr_from_wire(pr))
    }

    fn merge_pull(&self, pr_number: u64, method: MergeMethod) -> Result<()> {
        let url = format!("{}/pulls/{}/merge", self.repo_url(), pr_number);
        let body = ForgejoMergeBody {
            do_method: method.as_str().to_owned(),
            merge_message_field: String::new(),
        };
        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| TideError::Downstream(format!("merge_pull POST: {e}")))?;

        let status = resp.status();
        // Forgejo returns 200 OK or 204 No Content on success.
        if status.as_u16() == 200 || status.as_u16() == 204 {
            return Ok(());
        }

        Err(TideError::Downstream(format!(
            "merge_pull returned HTTP {status}"
        )))
    }
}

// ---------------------------------------------------------------------------
// Wire → domain projection
// ---------------------------------------------------------------------------

fn pr_from_wire(pr: ForgejoPr) -> PullRequest {
    PullRequest {
        number: pr.number,
        title: pr.title,
        head_sha: pr.head.sha,
        base_ref: pr.base.ref_name,
        mergeable: pr.mergeable,
        labels: pr.labels.into_iter().map(|l| l.name).collect(),
    }
}

fn parse_next_link_header(header: &str) -> Option<String> {
    header.split(',').find_map(|part| {
        let trimmed = part.trim();
        if !trimmed.contains("rel=\"next\"") && !trimmed.contains("rel=next") {
            return None;
        }
        let start = trimmed.find('<')? + 1;
        let end = trimmed[start..].find('>')? + start;
        Some(trimmed[start..end].to_owned())
    })
}

fn parse_total_count(headers: &reqwest::header::HeaderMap) -> Option<usize> {
    headers
        .get("x-total-count")
        .or_else(|| headers.get("X-Total-Count"))
        .or_else(|| headers.get("x-total"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<usize>().ok())
}

fn with_page(initial_url: &str, page: u32) -> String {
    if let Some(index) = initial_url.find("page=") {
        let value_start = index + "page=".len();
        let value_end = initial_url[value_start..]
            .find('&')
            .map(|offset| value_start + offset)
            .unwrap_or(initial_url.len());
        let mut next = String::with_capacity(initial_url.len() + 4);
        next.push_str(&initial_url[..value_start]);
        next.push_str(&page.to_string());
        next.push_str(&initial_url[value_end..]);
        return next;
    }

    let separator = if initial_url.contains('?') { '&' } else { '?' };
    format!("{initial_url}{separator}page={page}")
}

fn compare_status_recency(
    left: &(usize, &ForgejoStatus),
    right: &(usize, &ForgejoStatus),
) -> std::cmp::Ordering {
    let left_timestamp = left.1.updated_at.as_ref().or(left.1.created_at.as_ref());
    let right_timestamp = right.1.updated_at.as_ref().or(right.1.created_at.as_ref());

    match (left_timestamp, right_timestamp) {
        (Some(left_ts), Some(right_ts)) if left_ts != right_ts => return left_ts.cmp(right_ts),
        _ => {}
    }

    match (left.1.id, right.1.id) {
        (Some(left_id), Some(right_id)) if left_id != right_id => return left_id.cmp(&right_id),
        _ => {}
    }

    left.0.cmp(&right.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_next_link_from_multi_link_header() {
        let header = concat!(
            "<https://forge/api/v1/repos/o/r/pulls?page=1>; rel=\"prev\", ",
            "<https://forge/api/v1/repos/o/r/pulls?page=3>; rel=\"next\""
        );
        assert_eq!(
            parse_next_link_header(header),
            Some("https://forge/api/v1/repos/o/r/pulls?page=3".to_owned())
        );
    }

    #[test]
    fn replaces_existing_page_query_parameter() {
        assert_eq!(
            with_page("https://forge/pulls?state=open&page=1&limit=50", 2),
            "https://forge/pulls?state=open&page=2&limit=50"
        );
    }

    #[test]
    fn total_count_fallback_fetches_after_short_intermediate_page() {
        let mut pages = vec![
            Page {
                items: vec![1, 2],
                next_url: None,
                total_count: Some(3),
            },
            Page {
                items: vec![3],
                next_url: None,
                total_count: Some(3),
            },
        ];
        let mut urls = Vec::new();
        let mut next_url = "https://forge/pulls?limit=50&page=1".to_owned();
        let mut fallback_page = 2u32;
        let mut all_items = Vec::new();

        loop {
            urls.push(next_url.clone());
            let page = pages.remove(0);
            let fetched = page.items.len();
            all_items.extend(page.items);
            if page
                .total_count
                .is_some_and(|total_count| all_items.len() < total_count)
                && fetched > 0
            {
                next_url = with_page("https://forge/pulls?limit=50&page=1", fallback_page);
                fallback_page += 1;
                continue;
            }
            break;
        }

        assert_eq!(all_items, vec![1, 2, 3]);
        assert_eq!(
            urls,
            vec![
                "https://forge/pulls?limit=50&page=1".to_owned(),
                "https://forge/pulls?limit=50&page=2".to_owned(),
            ]
        );
    }

    #[test]
    fn status_recency_prefers_timestamp_then_id_then_api_order() {
        let older = ForgejoStatus {
            context: "gate".to_owned(),
            state: "success".to_owned(),
            id: Some(10),
            updated_at: Some("2026-06-01T00:00:00Z".to_owned()),
            created_at: None,
        };
        let newer = ForgejoStatus {
            context: "gate".to_owned(),
            state: "failure".to_owned(),
            id: Some(9),
            updated_at: Some("2026-06-01T00:01:00Z".to_owned()),
            created_at: None,
        };

        assert_eq!(
            compare_status_recency(&(0, &older), &(1, &newer)),
            std::cmp::Ordering::Less
        );
    }
}
