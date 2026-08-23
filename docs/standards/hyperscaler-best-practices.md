---
purpose: "<!-- status: Accepted date: 2026-05-12 related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055 -->"
doc_status: research-context
canonical_spec: specs/hyperscaler-architecture-invariants.json
---

# Hyperscaler Best Practices — Research + oyatie Adoption Recommendation (2026-05-12)

Canonical binding surface: `specs/hyperscaler-architecture-invariants.json`.
This document remains research context and rationale; validators and claim
surfaces must use the machine-readable spec.

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Captured via real-time web research. Citations in each section.

---

## Executive summary (1 page)

The hyperscalers (AWS, Google, Microsoft, Oracle) have converged on a remarkably consistent engineering bar in 2026, even though each brands the practice differently. The convergent stack: narrative-driven decisions (Amazon 6-pager / Google design doc / RFC + ADR), trunk-based development with small, reviewer-gated CLs, blameless postmortems with mechanical preventions, SLO/error-budget release gating, progressive delivery via feature flags + canary, and supply-chain hardening via SLSA / Sigstore / SBOMs / signed builds. Rust-specific: workspace inheritance for deps + lints, `cargo-deny` + `cargo-audit` + `cargo-vet` triad, `cargo-nextest` for evidence runs, `thiserror`-in-libraries + `anyhow`/`eyre`-at-the-edge, Kani for unsafe verification (AWS-pioneered), and distroless/Chainguard for runtime images.

Oyatie already meets or exceeds the hyperscaler bar on several axes — RFC-2119 normative language, doc-class taxonomy, mistake-ledger doctrine, mechanical-prevention-over-process discipline, mandatory `cargo nextest` + `cargo clippy -D warnings` + `cargo deny check` evidence gates, capability-tier (T1–T4) autonomy ceiling, and audit-chain emission on cross-axis flow. These map cleanly to Google's blameless-postmortem culture, Amazon's tenet-driven decisions, and Microsoft's 1ES quality gates.

The three highest-impact gaps to close are: (1) **provenance + signing of build artifacts** — no SLSA L2+/Cosign keyless signing/SBOM emission in the current toolchain, which is now the floor for shippable software at every hyperscaler; (2) **progressive-delivery rails** — feature-flag + canary + automated-rollback infrastructure is absent from `RELEASE-MANAGEMENT.md`, so changes are atomic rather than progressive; (3) **`cargo-vet` supply-chain audit trail** — `cargo-deny` covers license/advisory but does not capture the human review chain that AWS/Mozilla now treat as table-stakes for sensitive code paths.

Top-3 immediate-adoption items: **(A)** turn on Cosign keyless OIDC signing + Syft SBOM + SLSA provenance attestation on every CI artifact emit, gated by a new `governance-supply-chain` lane; **(B)** stand up a feature-flag + canary rail with stable cohorts, automated SLO-burn-rate analysis, and automated rollback, mapped into `RELEASE-MANAGEMENT.md`; **(C)** adopt `cargo-vet` alongside existing `cargo-deny`/`cargo-audit` and pin `rust-toolchain.toml` workspace-wide to remove "stable drift" as a build hazard.

The rest of this document captures the canonical sources for each practice and the per-domain adoption ranking.

---

## Domain 1: Project Management Practices

### AWS

**6-pager memo culture + Working Backwards PRFAQ.** Bezos banned slide decks at S-team meetings in 2004 in favor of 6-page narrative memos read silently for the first ~20 min of every meeting; the standard for new-product proposals is the PRFAQ (a future-dated press release + FAQ) authored before any code is written. Writing rules are strict: <30 words/sentence, replace adjectives with data, full prose (no bullets), no author attribution on the doc. ([The Amazon Writing Culture](https://www.theprfaq.com/articles/amazon-writing-culture); [Working Backwards PR/FAQ Process](https://workingbackwards.com/concepts/working-backwards-pr-faq-process/); [a16z — Amazon Narratives](https://a16z.com/podcast/amazon-narratives-memos-working-backwards-from-release-more/))

**Two-pizza teams → Single-Threaded Leaders (STL).** Two-pizza teams were a 2000s-era heuristic that Amazon outgrew; the current model is the Single-Threaded Leader — one leader 100% accountable to a separable mission, empowered to make decisions without cross-team consensus. The team size still hovers near "small enough to be fed by two pizzas" but the discriminating attribute is the *leader's* singular focus, not headcount. ([AWS — Amazon's Two-Pizza Teams](https://aws.amazon.com/executive-insights/content/amazon-two-pizza-team/); [Inc. — Single-Threaded Leadership](https://www.inc.com/jeff-haden/when-jeff-bezoss-two-pizza-teams-fell-short-he-turned-to-brilliant-model-amazon-uses-today.html); [Working Backwards book summary](https://commoncog.com/working-backwards/))

**Bar Raiser hiring loop.** Every interview loop seats an elite cross-team "Bar Raiser" whose sole purpose is to refuse hires that would not "raise the average" — they hold veto power independent of the hiring manager. Designed to minimize ad-hoc variance and prevent dilution at scale.

**Tenets.** A team-level codification of "the how" (the mission states "the what") — 3–7 short principles, each one a single non-trivial idea, used as a compass when fast decisions need to be made consistently across people who weren't in the original room. Best practices: be memorable, challenge the reader, one idea per tenet, never codify something nobody would argue against, reference from design docs to keep them live. ([AWS — Tenets: supercharging decision-making](https://aws.amazon.com/blogs/enterprise-strategy/tenets-supercharging-decision-making/); [Tenets at Amazon — pedrodelgallego](https://pedrodelgallego.github.io/blog/amazon/mental-models/decision-making/tenets-at-amazon/))

### Google

**Design docs.** Every non-trivial change starts with a design doc reviewed before implementation begins. Format is open but converges on: context, scope, design proposal, alternatives considered, tradeoffs, risks, and an explicit reviewer list. Google maintains an internal Doc-of-Docs registry; the doc lives forever and is updated as the system evolves.

**OKRs.** Quarterly objectives + 3–5 measurable key results; 0.7 is a "good" score (1.0 means you sandbagged). Set at every level of the org, public to the entire company. The practice has fallen out of fashion at smaller companies but remains the Google standard for cross-team alignment.

**Blameless postmortems.** Mandatory for every Sev-1/Sev-2. The doctrine: assume everyone acted in good faith with the information they had; "you can't fix people, only their environment." Postmortems are widely shared because the lessons apply beyond the originating team. Internal template in Google Docs; the practice is the *single* most copied SRE export. ([Google SRE — Postmortem Culture](https://sre.google/sre-book/postmortem-culture/); [Google SRE Workbook — Postmortem Culture](https://sre.google/workbook/postmortem-culture/); [Google Cloud — Fearless Shared Postmortems](https://cloud.google.com/blog/products/gcp/fearless-shared-postmortems-cre-life-lessons))

**SRE practice: error budgets + 4 golden signals.** SLO drives release decisions: if the error budget is healthy → ship; if depleted → freeze and stabilize. The four golden signals — latency, traffic, errors, saturation — are the canonical dashboard. Burn-rate alerts (1× = on-target, 3× = exhaust budget in 1/3 of window) are now standard. ([Google SRE — Monitoring Distributed Systems](https://sre.google/sre-book/monitoring-distributed-systems/); [Google SRE Workbook — Error Budget Policy](https://sre.google/workbook/error-budget-policy/); [Splunk — Four Golden Signals](https://www.splunk.com/en_us/blog/learn/sre-metrics-four-golden-signals-of-monitoring.html))

### Microsoft

**One Engineering System (1ES).** Unified tools/processes/practices across the company: a single Azure DevOps instance hosting 100k+ engineers, thousands of repos, 20k+ pipelines. Goals: consistent processes, clear communication, resilient lifecycle. ([Azure — DevOps at Microsoft / 1ES](https://azure.microsoft.com/en-us/solutions/devops/devops-at-microsoft/one-engineering-system/); [Microsoft Inside Track — Streamlining engineering with Azure DevOps](https://www.microsoft.com/insidetrack/blog/streamlining-engineering-at-microsoft-with-azure-devops/); [naked Agility — One Engineering System](https://nkdagility.com/resources/one-engineering-system/))

**Engineering Excellence framework.** Standardized lint/CI/security/release gates applied uniformly across all Microsoft product groups; the "Customer-Connected" approach pulls field telemetry directly back into the engineering loop.

### Oracle

**OCI Well-Architected Framework.** Five pillars: security, reliability, performance, cost optimization, operational efficiency. Curated design patterns (multicloud networking, agentic, cloud-native) in the [Oracle Architecture Center](https://docs.oracle.com/en/solutions/arch-center-assets/index.html). The "Engineering Excellence Council" naming is not externally visible; Oracle's engineering-practice export is largely embedded in OCI patterns rather than process culture. ([OCI Well-Architected Framework](https://blogs.oracle.com/cloud-infrastructure/oci-wellarchitected-framework); [Oracle Architecture Center](https://docs.oracle.com/en/solutions/arch-center-assets/index.html))

### Cross-cutting synthesis

The convergent doc taxonomy is: **PRFAQ / vision doc → RFC (collect feedback) → ADR (record decision) → design doc (implementation detail) → runbook (operational)**. RFCs collect feedback; ADRs are short, immutable-ish records of *what was decided and why*; one accepted RFC can spawn multiple ADRs. Best ADR practices: keep concise (one decision per ADR), structured (Context / Decision / Consequences / Alternatives), live (allow date-stamped updates), and meeting discipline (10–15 min silent read, 30–45 min total). ([ADR.github.io](https://adr.github.io/); [ITNEXT — ADRs vs RFCs](https://itnext.io/how-to-make-architecture-decisions-rfcs-adrs-and-getting-everyone-aligned-ab82e5384d2f); [Candost — ADRs and RFCs](https://candost.blog/adrs-rfcs-differences-when-which/); [AWS — Master ADR best practices](https://aws.amazon.com/blogs/architecture/master-architecture-decision-records-adrs-best-practices-for-effective-decision-making/); [joelparkerhenderson/architecture-decision-record](https://github.com/joelparkerhenderson/architecture-decision-record))

Blameless postmortem culture is universal — every hyperscaler runs the same loop: root-cause, document, share widely, ship a mechanical prevention.

---

## Domain 2: Development Practices

### Code review

**Google.** Pre-commit review is mandatory; the bar is "LGTM" from at least one reviewer with the right context. The Standard: code health > correctness alone — does this change leave the codebase better than it found it? Small CLs (one logical change, related tests included, reviewer can hold it in their head). Median review latency target is <1 day. ([Google eng-practices](https://github.com/google/eng-practices); [Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html); [Standard of Code Review](https://google.github.io/eng-practices/review/reviewer/standard.html); [How Google takes the pain out of code reviews](https://read.engineerscodex.com/p/how-google-takes-the-pain-out-of))

**Microsoft CodeFlow.** Internal review tool used by 50k+ developers; native-app UX, comment threading, multi-line / multi-file comments, threaded resolutions. 2025: AI-powered code review assistant on >90% of PRs, ~10–20% PR-completion-time reduction. ([ACM Queue — CodeFlow](https://queue.acm.org/detail.cfm?id=3292420); [Greiler — Code Reviews at Microsoft](https://www.michaelagreiler.com/code-reviews-at-microsoft-how-to-code-review-at-a-large-software-company/); [Microsoft DevBlogs — AI-Powered Code Reviews](https://devblogs.microsoft.com/engineering-at-microsoft/enhancing-code-quality-at-scale-with-ai-powered-code-reviews/))

**AWS.** Code review bar plus the Bar Raiser doctrine extended to code: an STL or principal engineer signs off cross-cutting changes; security and operations are mandatory reviewers on their respective surfaces.

### Testing

**Test pyramid 2.0 (2025).** AI-assisted test generation across all tiers; LLMs parse code structure, propose scaffolds, and flag missing branches. Fuzzing has moved from "exotic" to "expected" — coverage-guided fuzzing is the Google default for any parser/serializer. Mutation testing complements property-based testing: property-tests specify *what should hold*, fuzz/mutation probes *where it breaks*. ([Frontiers — Test Pyramid 2.0](https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2025.1695965/full); [Full Scale — Modern Test Pyramid Guide](https://fullscale.io/blog/modern-test-pyramid-guide/); [Number Analytics — Fuzz Testing](https://www.numberanalytics.com/blog/mastering-fuzz-testing-in-property-testing))

### Branch / merge strategy

**Trunk-based development at Google and Microsoft.** Short-lived branches (hours to days), continuous integration to trunk, feature flags hide incomplete work. DORA research identifies TBD as a core capability of elite-performing engineering orgs. ([Trunk Based Development](https://trunkbaseddevelopment.com/continuous-review/); [Aviator — What is Trunk-Based Development](https://www.aviator.co/blog/trunk-based-development/))

### Documentation

**Diátaxis.** Four content types — tutorials (learning-oriented), how-tos (task-oriented), reference (info-oriented), explanation (understanding-oriented). Adopted by Python docs, Canonical/Ubuntu, Cloudflare, and many others. The discipline: each page serves *one* audience need; mixing types is the antipattern. ([Diátaxis](https://diataxis.fr/); [I'd Rather Be Writing — Diátaxis](https://idratherbewriting.com/blog/what-is-diataxis-documentation-framework); [Ubuntu — Diátaxis foundation](https://ubuntu.com/blog/diataxis-a-new-foundation-for-canonical-documentation))

### Feature flags + progressive delivery

**State of the art (2025).** LaunchDarkly / Flagsmith / Unleash are standard; combined with canary deploys (small % traffic to new version) and automated SLO-burn-rate analysis for rollback. Argo Rollouts and Flagger are the Kubernetes-native rails. Blue-green is reserved for stateful cutovers; canary + flags is the everyday default. Flag debt is a real anti-pattern — flags must have expiry. ([Flagsmith — Progressive Delivery](https://www.flagsmith.com/blog/progressive-delivery); [Unleash — Canary vs Progressive Delivery](https://www.getunleash.io/blog/canary-release-vs-progressive-delivery); [Wissen — Blue-Green, Canary, Feature Flags](https://www.wissen.com/blog/the-role-of-blue-green-canary-and-feature-flags); [Visualpath — Progressive Delivery SRE 2025](https://visualpathblogs.com/site-reliability-engineering/what-is-the-best-way-to-implement-progressive-delivery-sre-in-2025/))

### On-call / runbooks

Runbook discipline: every alert resolves to a runbook URL; every runbook is exec-readable; runbook coverage is itself a CI lane. On-call rotations: primary + secondary, no solo on-call, paid compensation, weekly handoff with explicit "what's burning" briefing.

---

## Domain 3: Rust Practices and Quirks

### Workspace structure

**AWS.** Firecracker, Bottlerocket, s2n-quic, Nitro components are flat-or-shallow Cargo workspaces with strict crate boundaries. AWS pioneered Kani (the Rust model checker) for verifying unsafe blocks in Firecracker — they treat unsafe surface as a CI-gated verification target. ([AWS — Sustainability with Rust](https://aws.amazon.com/blogs/opensource/sustainability-with-rust/); [AWS — Why AWS is the best place to run Rust](https://aws.amazon.com/blogs/devops/why-aws-is-the-best-place-to-run-rust/); [AWS — How Kani is used](https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/); [Firecracker](https://firecracker-microvm.github.io/); [DZone — 17 AWS Rust Projects](https://dzone.com/articles/17-open-source-projects-at-aws-written-in-rust))

**Microsoft.** Hyperlight (hypervisor-isolated function runtime, Rust), Azure SDK for Rust (one crate per service, MSRV pinned at workspace root). ([Microsoft — Introducing Hyperlight](https://opensource.microsoft.com/blog/2024/11/07/introducing-hyperlight-virtual-machine-based-security-for-functions-at-scale/); [Azure SDK for Rust](https://github.com/Azure/azure-sdk-for-rust); [Azure SDK Rust Guidelines](https://azure.github.io/azure-sdk/rust_introduction.html); [TNS — Microsoft Goes All-in on Rust](https://thenewstack.io/microsoft-goes-all-in-on-rust-for-core-infrastructure-and-much-more/))

### Cargo workspace inheritance

`[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]` are all stable and the de-facto standard. Pitfall: `workspace.lints` is *not* implicitly inherited — each member must declare `[lints] workspace = true`. Override via `#![allow(...)]` in lib.rs, not in Cargo.toml. `cargo-autoinherit` automates the dependency-DRY pass. ([Cargo Book — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html); [Cargo Book — Lints](https://doc.rust-lang.org/cargo/reference/lints.html); [RFC 3389 — manifest-lint](https://rust-lang.github.io/rfcs/3389-manifest-lint.html); [Mainmatter — cargo-autoinherit](https://mainmatter.com/blog/2024/03/18/cargo-autoinherit/))

### Clippy

`#![warn(clippy::pedantic)]` at workspace level with selective `#[allow(...)]` is the common pattern. Pedantic has false positives, so cherry-picking lints from the group beats blanket-deny. CI enforces `cargo clippy -- -D warnings`. ([Clippy Lints](https://rust-lang.github.io/rust-clippy/master/index.html); [Clippy Configuration](https://doc.rust-lang.org/clippy/configuration.html); [Effective Rust — Item 29: Listen to Clippy](https://effective-rust.com/clippy.html))

### Supply chain

- `cargo-audit` (Rust Secure Code WG) — RustSec advisory database.
- `cargo-deny` (Embark Studios) — license/source/advisory in one. Superset of audit for most use cases.
- `cargo-vet` (Mozilla) — distributed human-audit trail for third-party crates; supports shared audits across orgs.
- `cargo-auditable` — embeds SBOM in the binary so deployed artifacts can be retro-scanned.

([Mozilla — cargo-vet](https://mozilla.github.io/cargo-vet/); [LogRocket — Comparing Rust supply chain safety tools](https://blog.logrocket.com/comparing-rust-supply-chain-safety-tools/); [RustSec](https://rustsec.org/); [cargo-auditable](https://github.com/rust-secure-code/cargo-auditable); [cargo-deny config](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html))

### Test runner

`cargo-nextest` is the modern default — parallel by default, junit/json output, retries, slow-test detection. Used by AWS, Microsoft, Discord, etc. Oyatie already mandates it.

### Error handling

- **`thiserror` in libraries** — exposes matchable error enums for callers.
- **`anyhow` (or `eyre`) at the application edge** — type-erased + Send/Sync/'static + cheap backtrace, narrow pointer (one word).
- `tracing-error` + `miette` for diagnostic reporting.

([oneuptime — thiserror + anyhow](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view); [Luca Palmieri — Error Handling in Rust](https://www.lpalmieri.com/posts/error-handling-rust/); [Momori — thiserror, anyhow](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/); [eyre](https://github.com/eyre-rs/eyre); [Markaicode — Rust Error Handling 2025](https://markaicode.com/rust-error-handling-2025-guide/))

### Async runtime

Tokio is the *de facto* monoculture in 2026 — `async-std` is effectively abandoned, `smol` is a niche. Multi-thread scheduler is default; `spawn_blocking` for any operation >10–100 µs latency. ([Tokio docs](https://docs.rs/tokio); [corrode — State of Async Rust](https://corrode.dev/blog/async/); [TNS — Async Programming in Rust](https://thenewstack.io/async-programming-in-rust-understanding-futures-and-tokio/))

### Unsafe / FFI

AWS-pioneered pattern: every `unsafe` block carries a `// SAFETY:` comment documenting the invariants the caller must uphold; fuzz tests cover the FFI surface; Kani model-checks the critical paths. The combination is the closest thing to a formal verification standard the Rust ecosystem has.

### Build performance

`sccache` (compiler cache, S3-backed in CI), `cargo-chef` (Docker-layer dependency cache), `cargo-zigbuild` (cross-compile via zig as linker; *not* compatible with static crt). For static distroless: `muslrust` → scratch or `distroless-static` or `chainguard/static`. ([Somethings Blog — Turbocharge Rust CI](https://www.somethingsblog.com/2025/05/26/turbocharge-your-rust-projects-faster-ci-cd-pipelines-and-builds/); [muslrust](https://github.com/clux/muslrust); [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild); [cargo-chef](https://github.com/LukeMathWalker/cargo-chef); [LogRocket — Optimizing CI/CD Rust](https://blog.logrocket.com/optimizing-ci-cd-pipelines-rust-projects/))

### Semver

`cargo-semver-checks` is the de-facto linter, on track to be merged into cargo. `cargo-public-api` lists/diffs the public surface for CI gating. ([cargo-semver-checks](https://crates.io/crates/cargo-semver-checks); [cargo-public-api](https://github.com/cargo-public-api/cargo-public-api); [Rust Project Goals — cargo-semver-checks](https://rust-lang.github.io/rust-project-goals/2025h1/cargo-semver-checks.html); [Cargo Book — SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html))

### Toolchain pinning

`rust-toolchain.toml` pins the compiler version for reproducibility; `rust-version` in `Cargo.toml` declares the MSRV the package supports. Rust 2024 edition + Rust 1.84+ ships an MSRV-aware resolver that prefers dependency versions compatible with the declared MSRV. ([Cargo Book — rust-toolchain.toml](https://rust-lang.github.io/rustup/overrides.html); [Swatinem — Should I pin my Rust toolchain](https://swatinem.de/blog/rust-toolchain/); [Rust 1.84.0 Blog](https://blog.rust-lang.org/2025/01/09/Rust-1.84.0/); [RFC 3537 — MSRV resolver](https://rust-lang.github.io/rfcs/3537-msrv-resolver.html))

---

## Domain 4: CI/CD + Toolchain / Tooling

### CI/CD platforms

- **GitHub Actions** — default for OSS and GitHub-native shops; seamless with PRs.
- **CircleCI** — ~40% faster than GH Actions default runners, intelligent test-splitting, 3000+ orbs.
- **Buildkite** — hybrid (cloud control plane + self-hosted agents); dynamic pipelines; scales past CircleCI/GHA.
- **Azure Pipelines + 1ES hosted pools** — Microsoft's templated-pipeline standard; very heavy enterprise lean.

([Buildkite — GH Actions vs CircleCI](https://buildkite.com/resources/comparison/github-actions-vs-circleci/); [Northflank — CircleCI vs GH Actions](https://northflank.com/blog/circleci-vs-github-actions); [Buildkite — Alternatives to Jenkins 2025](https://buildkite.com/resources/ci-cd-perspectives/alternatives-to-jenkins-the-top-options-in-2025/))

### Build systems

- **Bazel** (Google, Java; bzlmod 2024+) — polyglot, deterministic, remote-cache + remote-execution.
- **Buck2** (Meta, Rust-rewrite) — modern UX, faster, but no native Cargo dep support (requires `reindeer`).
- **Cargo workspace + cargo-make / just** — sufficient for single-language Rust monorepos.

([Buck2 docs](https://buck2.build/docs/about/why/); [Tweag — Tour Around Buck2](https://www.tweag.io/blog/2023-07-06-buck2/); [Bazel Remote Execution](https://bazel.build/remote/rbe); [Better Programming — Blaze to Buck2](https://medium.com/better-programming/from-blaze-to-buck2-a-brief-history-of-modern-monorepo-build-systems-563becbcb987))

### Container images

Distroless (Google) and Chainguard/Wolfi (declarative, reproducible, signed-with-sigstore, SBOM-included, nightly rebuilds) are the modern floor. Chainguard's May 2025 multi-layer-per-origin strategy reduced layer-data ~70%. For Rust: `chainguard/static` + statically-linked musl binary. ([Chainguard — Overview](https://edu.chainguard.dev/chainguard/chainguard-images/overview/); [Chainguard — Getting Started Distroless](https://edu.chainguard.dev/chainguard/chainguard-images/about/getting-started-distroless/); [Wolfi Dockerfiles](https://edu.chainguard.dev/open-source/wolfi/wolfi-with-dockerfiles/))

### Supply-chain: signing + SBOM + provenance

The 2025 stack: **SBOMs (Syft / Trivy / CycloneDX) + Sigstore Cosign (keyless OIDC) + SLSA provenance attestations + Rekor transparency log + Kyverno / policy-controller for cluster-side admission.**

SLSA levels:
- **L1** — provenance exists.
- **L2** — signed, tamper-resistant provenance.
- **L3** — verified source + isolated build.

GitHub OIDC + Fulcio + Cosign + Rekor delivers SLSA L2 in weeks. ([Wiz — SLSA Framework](https://www.wiz.io/academy/application-security/slsa-framework); [SLSA Provenance v0.1](https://slsa.dev/spec/v0.1/provenance); [InfoQ — Provenance Tools Standard](https://www.infoq.com/news/2025/08/provenance/); [Faith Forge Labs — Supply-Chain Security 2025](https://faithforgelabs.com/blog_supplychain_security_2025.php); [Chainguard — Sign SBOM with Cosign](https://edu.chainguard.dev/open-source/sigstore/cosign/how-to-sign-an-sbom-with-cosign/); [SLSA — in-toto and SLSA](https://slsa.dev/blog/2023/05/in-toto-and-slsa); [Trivy SBOM](https://trivy.dev/docs/latest/supply-chain/attestation/sbom/); [Nathan Berg — Supply Chain Security in CI](https://nathanberg.io/posts/supply-chain-security-ci-sbom-slsa-sigstore/))

### Dependency management

Dependency updates: ADR-0535 supersedes earlier external-bot recommendations for Oyatie. The active path is the owned `deps.toml` contract plus an in-house Rust bump-bot that emits scm-facts ChangeSets and runs supply-chain gates before merge.

### Secret management

OpenBao (Linux Foundation fork of Vault; OSI license; near-identical architecture) is the open-source path; HashiCorp Vault (BSL → MPL after 4 years) is the commercial path. AWS Secrets Manager / Google Secret Manager / Azure Key Vault are provider-native; the right abstraction is a thin trait/interface (e.g., `SecretProvider`) backed by Vault/OpenBao for primary storage, with optional sync to the cloud-provider secret store for runtime injection. ([OpenBao](https://openbao.org/); [GitLab Handbook — ADR 007 OpenBao](https://handbook.gitlab.com/handbook/engineering/architecture/design-documents/secret_manager/decisions/007_openbao/); [Digitalis — Vault vs OpenBao](https://digitalis.io/post/choosing-a-secrets-storage-hashicorp-vault-vs-openbao); [Vault AWS Secrets Sync](https://developer.hashicorp.com/vault/docs/sync/awssm))

### Observability

OpenTelemetry collector is the universal ingestion fabric (gRPC/HTTP OTLP). Backend split: Grafana stack (Tempo / Mimir / Loki) for open-source, Honeycomb for high-cardinality query, Datadog for managed-everything. Three collector deployment patterns: agent (per-host), gateway (centralized), hierarchical (both). Standard retention: high-res metrics 7d, downsampled 90d, traces sampled 7–14d. ([Honeycomb — OpenTelemetry](https://www.honeycomb.io/platform/opentelemetry); [Markaicode — Full Stack Observability 2025](https://markaicode.com/2025-observability-opentelemetry-grafana-11-full-stack-monitoring/); [Better Stack — OTel Best Practices](https://betterstack.com/community/guides/observability/opentelemetry-best-practices/); [OpenTelemetry Collector docs](https://opentelemetry.io/docs/collector/))

### Pre-commit vs PR-time gates

Pattern that hyperscalers converge on: **fast checks (formatters, simple linters) at pre-commit; full evidence (clippy, nextest, deny, SLSA emission) at PR-time CI.** Pre-commit gives <1s feedback; CI provides authoritative gating. `pre-commit.ci` automates running the same hook set on PRs so divergence is mechanically impossible. ([gatlenculp — Pre-Commit Hooks Guide 2025](https://gatlenculp.medium.com/effortless-code-quality-the-ultimate-pre-commit-hooks-guide-for-2025-57ca501d9835); [pre-commit.ci](https://pre-commit.ci/); [helio — Quality Gates in the Age of Agentic Coding](https://blog.heliomedeiros.com/posts/2025-07-18-quality-gates-agentic-coding/))

### Mandatory CI gates for hyperscaler-quality repos (synthesis)

1. Format check (rustfmt / prettier / black).
2. Lint (`cargo clippy -D warnings` / eslint-strict).
3. Tests (`cargo nextest run --no-fail-fast` / pytest / jest).
4. Supply chain (`cargo deny check` + `cargo audit` + `cargo vet`).
5. Coverage (≥ some target on changed lines).
6. SBOM generation (Syft / Trivy).
7. Cosign keyless signing of artifacts.
8. SLSA L2+ provenance attestation.
9. Secret scan (gitleaks / trufflehog).
10. License-policy gate (Open Chain compliant).
11. Docs/ADR/runbook gates (doc-catalog lane equivalent).
12. Reviewer-agent signoff captured on PR body.

---

## Oyatie adoption recommendation

### Top-5 PM practices to adopt

| # | Practice | Effort | Impact |
|---|---|---|---|
| 1 | **Amazon-style PRFAQ for every new axis / cross-axis capability** (template under `docs/templates/`, mandatory for new product PRDs) | low | high |
| 2 | **Tenets per axis** — 3–7 single-idea principles, referenced from per-axis design docs (already partially modeled in decision-principles.json DP-01..DP-10) | low | high |
| 3 | **Single-Threaded Leader per axis** — one accountable owner per axis with veto on cross-axis contracts feeding into their surface (oyatie has axis-team-lead role; formalize STL semantics) | low | medium |
| 4 | **Blameless-postmortem template + replay-as-eval gating** — already largely present via MISTAKES-LEDGER mechanical-prevention doctrine; tighten to enforce the *replay* property on every prevention | medium | high |
| 5 | **SRE error-budget release gate** — codify per-axis SLOs in `SLO-CATALOG.md`, derive error budgets, gate cross-axis contract changes on burn-rate state | medium | high |

### Top-5 dev practices to adopt

| # | Practice | Effort | Impact |
|---|---|---|---|
| 1 | **Small-CL discipline + reviewer-agent latency target** — explicit median-review-latency SLO (target: 24h), surface in `code-review.md` | low | high |
| 2 | **Coverage-guided fuzzing on parser / serializer / FFI surfaces** — `cargo-fuzz` + libFuzzer on every public-input boundary; nightly job emits regressions to MISTAKES-LEDGER | medium | high |
| 3 | **Feature flags + canary rail** — add `flags/` crate or adopt Unleash; mandate every behavior-changing PR ships behind a flag with explicit retire-by date (flag-debt SLO 30d) | high | high |
| 4 | **Trunk-based with short-lived branches** — already implicit; codify 7-day-max branch SLO + auto-stale-detection lane | low | medium |
| 5 | **Diátaxis-typed docs** — formalize the four content types in `standards/doc-style.md` (oyatie has doc-class taxonomy; merge it with Diátaxis to get external readability) | low | medium |

### Top-5 Rust practices to adopt

| # | Practice | Effort | Impact |
|---|---|---|---|
| 1 | **`cargo-vet` alongside `cargo-deny`/`cargo-audit`** — captures the human-audit chain that license/advisory tools don't; share audits across the Rust ecosystem for AWS/Mozilla-published crates | medium | high |
| 2 | **`rust-toolchain.toml` workspace pin + Rust 2024 edition + MSRV-aware resolver** — eliminates "stable drift" as a CI failure class; oyatie should track stable-2 (current minus two) as the MSRV floor | low | high |
| 3 | **Workspace lint inheritance with `clippy::pedantic` (warn) + cherry-picked deny set** — single `[workspace.lints.clippy]` table; every member crate declares `[lints] workspace = true`; ban-list captured in `standards/code-style.md` | low | high |
| 4 | **Kani for verifying unsafe boundaries** — wherever oyatie has `unsafe` (kernel crates, FFI, perf-critical primitives), pair with Kani harness + nightly job; emit verification artifacts into audit chain | high | medium |
| 5 | **`thiserror`-in-libraries / `anyhow`-or-`eyre`-at-edge** — already implied by `standards/error-handling.md`; codify the boundary rule explicitly (lib crates: no `anyhow` deps; bin crates: no exposed `thiserror` enums in public API of internal libs) | low | high |

### Top-5 CI/CD practices to adopt

| # | Practice | Effort | Impact |
|---|---|---|---|
| 1 | **Cosign keyless OIDC + Syft SBOM + SLSA L2 provenance attestation on every artifact emit** — new `governance-supply-chain` lane; cluster-side Kyverno verification at admission | medium | critical |
| 2 | **Chainguard/Wolfi or distroless-static base images for every container** — `chainguard/static` + statically-linked musl binary; ban Debian/Alpine bases in product crates | medium | high |
| 3 | **Owned dependency bump-bot over external bots** — `deps.toml` drives a Rust actuator with grouped updates, license/advisory/version gates, and scm-facts ChangeSets | low | high |
| 4 | **OpenTelemetry collector deployed agent + gateway** — standardize OTLP emission, single ingestion fabric; route to chosen backend (Grafana stack or Honeycomb) via env-config so the choice is reversible | medium | high |
| 5 | **`SecretProvider` trait + OpenBao primary** — keep AWS Secrets Manager / GSM / Azure KV as injection-only adapters; the source of truth lives in OpenBao | medium | high |

### Top-3 gaps vs hyperscaler bar

1. **No build-artifact signing, SBOM, or SLSA provenance.** Every hyperscaler now treats this as the floor (SLSA L2 is "weeks of work" per the 2025 InfoQ piece). Oyatie has `cargo deny check` for license/advisory but no Cosign / Syft / Rekor / provenance attestation in the documented CI lanes. This is the single largest gap and is directly addressable.

2. **No progressive-delivery rail.** `RELEASE-MANAGEMENT.md` describes release-management mechanics but has no canonical feature-flag library, canary rollout pattern, or automated SLO-burn-based rollback. Every hyperscaler ships behind a flag + canary by default in 2025; oyatie's current model is closer to atomic-deploy. This bakes risk into every release.

3. **No documented `cargo-vet` audit trail.** `standards/` covers `cargo-deny`, but the human-audit chain that protects against supply-chain attacks where a license-clean crate ships malicious behavior (the `xz`-style attack) is absent. `cargo-vet` is the canonical answer and integrates with `cargo-deny`. Adopt-and-bootstrap requires only a few days for an org of oyatie's current size.

### Explicitly NOT adopting

- **Amazon's "no author attribution on docs"** — oyatie has a strict audit-chain doctrine; doc authorship is provenance and must be captured.
- **Two-pizza headcount heuristic** — superseded by Single-Threaded Leader at Amazon itself; adopt the STL semantics, ignore the pizza math.
- **Bazel or Buck2 as the build system** — Cargo workspace is sufficient for a single-language Rust core; the cost of a polyglot monorepo build system is not justified at oyatie's scale. Reassess if/when JS/Python critical-path code lands.
- **`async-std`, `smol`** — Tokio is the monoculture; multi-runtime pluralism is now a maintenance tax with no upside.
- **Microsoft 1ES "one Azure DevOps instance" pattern** — oyatie's no-cloud-lockin principle forbids this; GitHub Actions + a self-hosted Buildkite-style runner pool is the cloud-portable analog.
- **AWS-specific Bar Raiser hiring** — adopt the *principle* (every hire raises the average; cross-team interviewer with veto), skip the AWS-specific cultural overlay that requires institutional inertia oyatie does not have.
- **OKRs at the company level** — high-overhead practice that doesn't pay off below several hundred engineers; oyatie's per-wave gates already serve the same alignment function.
- **`anyhow` in library crates** — keep `thiserror` discipline at the library boundary; `anyhow`/`eyre` is application-edge only.
- **Default Debian/Alpine container bases** — go straight to distroless-static / Chainguard; don't pay the migration cost later.
- **Multi-runtime async pluralism** — pick Tokio, forbid alternatives in the workspace.

---

## Mapping to Master Plan

| Practice | Milestone / Workstream | CI lane / process gate | ADR target |
|---|---|---|---|
| PRFAQ template + mandate | M01-P14 Hyperscaler-Practice Adoption | `governance-prfaq-on-new-axis` | ADR-PM-001 Adopt PRFAQ for new-axis intake |
| Tenets per axis | M01-P14 | `governance-tenets-cite` (axis design docs cite tenets) | ADR-PM-002 Tenet structure + cardinality |
| STL semantics formalized | M01-P14 | `governance-stl-decl` (RACI declares STL per axis) | ADR-PM-003 STL per axis |
| Postmortem replay-as-eval | M01-P14 Engineering-excellence rollout | `governance-mistakes-ledger-replay` (every `mechanical` prevention has a replay harness) | ADR-EE-001 Replay-as-eval discipline |
| SRE error-budget release gate | M01-P14 | `governance-error-budget-gate` | ADR-EE-002 SLO-derived release gate |
| Median-review-latency SLO | M01-P17 | `governance-review-latency` | (extend `standards/code-review.md`) |
| Coverage-guided fuzzing | M01-P17 Test-evidence floor | `governance-fuzz-coverage` (parser/serializer/FFI surfaces) | ADR-TST-001 Fuzz-on-boundary |
| Feature flags + canary rail | M01-P17 Progressive delivery | `governance-flag-debt` + canary automation in RELEASE-MANAGEMENT | ADR-REL-001 Feature-flag substrate |
| Trunk-based branch SLO | M01-P17 | `governance-branch-age` | (extend `standards/commit-message.md`) |
| Diátaxis content types | M01-P09 Doc auto-generation + freshness | `governance-doc-class-diataxis` | (extend `standards/doc-style.md`) |
| `cargo-vet` adoption | M01-P15 Supply-chain security | `governance-cargo-vet` | ADR-SUP-001 cargo-vet baseline |
| `rust-toolchain.toml` pin + 2024 edition | M01-P15 | `governance-toolchain-pin` | ADR-RST-001 Toolchain pin policy |
| Workspace lint inheritance (clippy::pedantic warn + ban-list) | M01-P15 | `governance-workspace-lints-inherit` | ADR-RST-002 Workspace-lint policy |
| Kani for unsafe verification | M01-P15 | `governance-unsafe-kani` (every `unsafe` block in kernel crates has a Kani harness or `SAFETY:` rationale of a documented class) | ADR-RST-003 Unsafe verification policy |
| thiserror/anyhow boundary rule | M01-P15 | `governance-error-boundary` (lib crates ban `anyhow`; bin crates ban exposed `thiserror` enums in internal-lib public APIs) | (extend `standards/error-handling.md`) |
| Cosign + Syft + SLSA L2 | M01-P15 | `governance-supply-chain` (signed + SBOM-attached + provenance-attested) | ADR-SUP-002 Sigstore + SLSA L2 |
| Chainguard/distroless-static images | M01-P13 Distroless + image discipline | `governance-container-base` (ban Debian/Alpine in product crates) | ADR-INF-001 Container base policy |
| Owned dependency automation | M01-P15 | `pipeline-dependency-automation` (`deps.toml` closed-schema policy + Rust bump-bot contract) | ADR-0535 / P7 bump-bot |
| OTel collector agent+gateway | M01-P17 Pipeline maturity glue | `governance-otel-emit` (every service emits OTLP via a documented exporter) | ADR-OBS-001 OpenTelemetry as canonical fabric |
| `SecretProvider` trait + OpenBao primary | M01-P15 | `governance-secret-provider` (no direct AWS SM / GSM / Azure KV calls in product code; all via trait) | ADR-SEC-001 Secret abstraction |

---

## 300-word executive summary

Hyperscaler engineering practice in 2026 has converged across AWS, Google, Microsoft, and Oracle on a remarkably coherent stack: narrative-driven decisions (Amazon 6-pager / Google design doc / RFC + ADR), trunk-based development with small reviewer-gated CLs, blameless postmortems with mechanical preventions, SRE-style SLO + error-budget release gating, progressive delivery via feature flags + canary, and supply-chain hardening via SLSA / Sigstore / SBOMs / signed builds. Rust-specific consensus: workspace inheritance for dependencies and lints, the `cargo-deny` + `cargo-audit` + `cargo-vet` triad, `cargo-nextest` for evidence, `thiserror`-in-libraries + `anyhow`/`eyre`-at-application-edge, Kani for `unsafe` verification (AWS-pioneered on Firecracker and s2n-quic), and Chainguard/Wolfi distroless-static images for runtime.

Oyatie already meets or exceeds the hyperscaler bar on several axes — RFC-2119 normative-language discipline, doc-class taxonomy, the mistake-ledger doctrine, mechanical-prevention-over-process culture, mandatory `cargo nextest` + `cargo clippy -D warnings` + `cargo deny check` evidence gates, the capability-tier autonomy ceiling, and audit-chain emission on cross-axis data flow. These map cleanly to Google's blameless-postmortem culture, Amazon's tenet-driven decisions, and Microsoft's 1ES quality-gate philosophy. The three top-rank gaps are: (1) no build-artifact signing / SBOM / SLSA provenance — every hyperscaler now treats this as the shippable-software floor; (2) no progressive-delivery rail — releases are atomic rather than flag-and-canary; (3) no `cargo-vet` human-audit trail layered atop the existing license-and-advisory checks.

**Top-3 immediate-adoption items:** (A) turn on Cosign keyless OIDC signing + Syft SBOM + SLSA L2 provenance attestation on every CI artifact emit, gated by a new `governance-supply-chain` lane; (B) stand up a feature-flag + canary rail with stable cohorts, automated SLO-burn-rate analysis, and automated rollback, encoded into `RELEASE-MANAGEMENT.md`; (C) adopt `cargo-vet` alongside `cargo-deny`/`cargo-audit` and pin `rust-toolchain.toml` workspace-wide to remove "stable drift" as a CI failure class.
