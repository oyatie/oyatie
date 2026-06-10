# PHASE-0 PRODUCER DRAFT — the live `oya-ci-required` producer (GitHub)

> **DRAFT for founder review. No `source` mutated.** On sign-off → implement in `source` on a `phase0/producer` branch, local RED/GREEN autonomously, **HALT before the live GitHub ruleset flip** (founder-paired, needs GitHub admin). Source-backed: every code shape lifted from existing, tested source.

## 0. What already exists (verified, cited) — the producer is mostly assembly
- **Controller seam** `ForgejoStatusPoster::post(sha, state: ForgejoState, context, description, target_url)` — already **forge-neutral in shape**; `oya-ci-controller-kernel/src/lib.rs:620-631`. `ForgejoState = {Pending,Success,Failure,Error}` (`:63-67`), `GATE_CONTEXT="oya-ci-required"` (`:471`).
- **Controller Forgejo adapter** (the impl template, incl. 240-char description truncation, ADR-0083 Tier-3, `data_class: INTERNAL_ONLY`): `oya-ci-controller-forgejo-adapter/src/lib.rs:72-122`.
- **PROVEN GitHub commit-status POST** (lift this HTTP shape): `oya-ci-webhook-gateway-github-adapter/src/lib.rs:68-96` — `POST {base}/repos/{owner}/{repo}/statuses/{sha}` · `bearer_auth` · `X-GitHub-Api-Version: 2022-11-28` · `Accept: application/vnd.github+json` · accept `resp.status().is_success()` · `.with_api_base()` test seam. Tested: `…-github-adapter/tests/d5_github_status_poster.rs` (httpmock). The gateway **already names its trait `CommitStatusPoster`** — so aligning the controller to that name is codebase-consistent, not new vocab.
- **Policy engine + fixtures** (the gate logic + RED/GREEN corpus): `kernel` policy engine + `source/specs/fixtures/phase0-ci-enforcement-baseline/` (GREEN `tc-0.0-good-…` + 10 RED `tc-0.0.1*/0.0.2/0.0.3`); result schema `source/specs/phase0-ci-enforcement-result-schema.json`.
- **GitHub states map 1:1** — GitHub accepts exactly `{pending,success,failure,error}`; `ForgejoState::as_str()` already emits those → **no mapping code**, just the rename.

## 1. Build step A — forge-neutral seam (eradicate `Forgejo*` vocab at the seam)
Rename in `oya-ci-controller-kernel`: `ForgejoStatusPoster` → **`CommitStatusPoster`**, `ForgejoState` → **`CommitState`** (variants unchanged). Update the 3 in-repo users: the existing `…-forgejo-adapter` (rename type `ForgejoCommitStatusPoster`→`ForgeCommitStatusPoster`, keep as a *bridge impl, deletion-tagged* — Forgejo is dropped), the `…-app` wiring, and the kernel tests. Contained rename; the gateway's existing `CommitStatusPoster` proves the name. *(This is the A-CI/Forgejo-eradication seam landed early because the producer needs it.)*

## 2. Build step B — `oya-ci-controller-github-adapter` (new crate; the actual draft)
```rust
//! # oya-ci-controller-github-adapter
//! GitHub commit-status poster for the oya-ci controller (the `oya-ci-required` producer).
//! Implements [`CommitStatusPoster`] via reqwest blocking. HTTP shape lifted from the
//! proven oya-ci-webhook-gateway-github-adapter (ADR-0387 D5). Forge-of-record = GitHub (D2/D-FORGE).
//! ADR-0083 Tier-3: no unwrap/expect/panic on the request path.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_ci_controller_kernel::{CommitState, CommitStatusPoster, KernelError, Result};
use serde::Serialize;

const GITHUB_API_VERSION: &str = "2022-11-28";
const GITHUB_API_BASE: &str = "https://api.github.com";
/// Forge of record (D2): GitHub interim until the Sapling-inspired bespoke SCM.
const DEFAULT_REPO_OWNER: &str = "jason931225";
const DEFAULT_REPO_NAME: &str = "oyatie";

pub struct GitHubCommitStatusPoster {
    repo_owner: String,   // data_class: INTERNAL_ONLY
    repo_name: String,    // data_class: INTERNAL_ONLY
    github_token: String, // data_class: INTERNAL_ONLY  (controller crier token ONLY; never to runner)
    api_base: String,     // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl GitHubCommitStatusPoster {
    pub fn new(repo_owner: &str, repo_name: &str, github_token: &str) -> Self {
        Self { repo_owner: repo_owner.to_owned(), repo_name: repo_name.to_owned(),
            github_token: github_token.to_owned(), api_base: GITHUB_API_BASE.to_owned(),
            client: reqwest::blocking::Client::new() }
    }
    pub fn with_defaults(token: &str) -> Self { Self::new(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME, token) }
    pub fn with_api_base(mut self, base: &str) -> Self { self.api_base = base.trim_end_matches('/').to_owned(); self }
    fn statuses_url(&self, sha: &str) -> String {
        format!("{}/repos/{}/{}/statuses/{}", self.api_base, self.repo_owner, self.repo_name, sha)
    }
}

impl CommitStatusPoster for GitHubCommitStatusPoster {
    fn post(&self, sha: &str, state: CommitState, context: &str, description: &str,
            target_url: Option<&str>) -> Result<()> {
        // GitHub/Forgejo 240-char description limit; slice on char boundary (from forgejo-adapter:84-93).
        let description: &str = if description.chars().count() > 240 {
            let end = description.char_indices().nth(240).map(|(i, _)| i).unwrap_or(description.len());
            &description[..end]
        } else { description };

        let body = GitHubStatusBody { state: state.as_str().to_owned(), context: context.to_owned(),
            description: description.to_owned(), target_url: target_url.map(ToOwned::to_owned) };

        let resp = self.client.post(self.statuses_url(sha))
            .bearer_auth(&self.github_token)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "oya-ci-controller")   // GitHub requires a UA
            .json(&body).send()
            .map_err(|e| KernelError::DownstreamTransport(format!("github status post: {e}")))?;

        if resp.status().is_success() { return Ok(()); }   // GitHub returns 201 Created
        Err(KernelError::DownstreamTransport(format!("github status returned HTTP {}", resp.status())))
    }
}

#[derive(Serialize)]
struct GitHubStatusBody {
    state: String, context: String, description: String,
    #[serde(skip_serializing_if = "Option::is_none")] target_url: Option<String>,
}
```
*(Delta vs the gateway adapter: implements the controller's arg-based `post()` signature [not a request-struct], adds the 240-char truncation + a `User-Agent` header [GitHub rejects UA-less requests]. Everything else is the proven gateway shape.)*

## 3. Build step C — wiring (`oya-ci-controller-app`)
- Select the GitHub poster as the producer: `GitHubCommitStatusPoster::with_defaults(&env GITHUB_CI_TOKEN)` (forge-of-record GitHub). Keep the Forge bridge impl behind a `OYA_CI_FORGE` switch, **default `github`**, deletion-tagged.
- Token discipline: `GITHUB_CI_TOKEN` (scope `statuses:write`/`repo:status`) injected to the **controller pod only, never the runner** (the existing `FORGEJO_CI_TOKEN`-never-to-runner rule, `k8s-adapter:167-175`).
- Fix the tide context to match: set `OYA_TIDE_REQUIRED_STATUS_CONTEXT=oya-ci-required` (default is `oya-ci-gate`, `oya-ci-tide-kernel/src/lib.rs:76`).

## 4. RED/GREEN proof — autonomous, BEFORE go-live
- **Unit (lift `d5_github_status_poster.rs`):** httpmock the GitHub statuses endpoint; assert success path (201) + the 3 GitHub error modes map to `KernelError::DownstreamTransport`; assert body `{state,context,description,target_url}` + headers (`Authorization: Bearer`, `X-GitHub-Api-Version`, UA). GREEN must pass; a mismatched context/SHA must fail.
- **Policy engine (existing):** the 11-fixture corpus in `specs/fixtures/phase0-ci-enforcement-baseline/` (1 GREEN + 10 RED) — already real `cargo test`; rerun to confirm the rename didn't break them.
- **Dual build:** `cargo` + `buck2` for the new crate; **if buck2 first-party RED on Linux → HALT (CP-BUCK2-LINUX)**.

## 5. Go-live — FOUNDER-PAIRED (NOT autonomous; door:one-way)
1. Deploy the controller (drop `nonClaim` helm markers), inject `GITHUB_CI_TOKEN` (controller SA only).
2. **You (GitHub admin):** flip the live `dev` ruleset `required_status_checks` → exactly `["oya-ci-required"]` (`scripts/branch-protection-apply.sh --apply`), replacing the current `[cargo-* + oya-pr-review(501)]`.
3. **Prove it BLOCKS:** GREEN PR → controller posts `oya-ci-required=success` (assert via `gh api …/commits/<sha>/statuses`), PR mergeable; RED PR (failing trusted-target) → `=failure`, **GitHub refuses merge**; tamper PR (edit gate script) → still `failure` (trunk-sourced). Record `required_status_checks == ["oya-ci-required"]`; flip `tc-0.12-…` from RED. Only then may `claim_boundary.p0_0_green` = true.

## 6. Source touch-points (on sign-off)
- `oya-ci-controller-kernel`: rename trait+enum (+ tests). · NEW `oya-ci-controller-github-adapter/` (Cargo.toml + BUCK + src/lib.rs above + tests). · `oya-ci-controller-forgejo-adapter`: rename type, tag bridge/deletion. · `oya-ci-controller-app`: wiring + forge switch. · `oya-ci-tide`: context default. · root `Cargo.toml` workspace member. All signed, pushed `github-mirror`, **no dev PR**.

## 7. Autonomy boundary (explicit)
**Autonomous:** steps 1–4 + 6 (seam rename, GitHub adapter, wiring, RED/GREEN, local dual-build) on `phase0/producer`. **HALT at step 5** — the live ruleset flip + prove-it-blocks is founder-paired (GitHub admin). If `oya-ci-required` is found already-required live mid-build → HALT (CP-AUTH-FLIP/G0).

*Draft. Authority: D-SEQUENCE (firewall-first) · D-CICD/ADR-0515 · D-FORGE (GitHub-interim, Forgejo dropped) · PHASE-0-FIREWALL-PLAN §4. Code shapes cited from gateway-github-adapter + controller kernel/forgejo-adapter. On sign-off → implement; HALT before go-live.*
