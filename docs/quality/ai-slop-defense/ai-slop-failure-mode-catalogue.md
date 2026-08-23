---
doc_class: AiSlopFailureModeCatalogue
parent: INDEX.md
status: Accepted
purpose: |
  Catalogue the failure modes that AI coding agents inject into a
  Rust + multi-language hyperscaler-bar codebase. Each entry binds a
  mechanical-prevention strategy to a named fitness lane.
owner: axis-foundry + ops-security
date: 2026-05-12
adr_citations:
  - ADR-0053
  - ADR-0055
doc_status: published
---

# AI-Slop Failure Mode Catalogue

> 42 catalogued modes across 16 classes. Severity scale:
> **C** (catastrophic — Sev-1-capable) · **H** (high — Sev-2-capable) ·
> **M** (medium — drift / quality erosion) · **L** (low — hygiene).
>
> Accepted per ADR-0053 (fitness-lane governance) and ADR-0055
> (impossible-to-fail environment contract).

## Class 1 — Hallucinated APIs / dependencies

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-001** | Hallucinated package on PyPI / crates.io / npm — "slopsquatting" attack surface; ~20% of AI suggestions reference nonexistent packages, 43% repeat across re-asks ([Snyk Slopsquatting](https://snyk.io/articles/slopsquatting-mitigation-strategies/), [USENIX 2025 package-hallucinations](https://www.usenix.org/system/files/conference/usenixsecurity25/sec25cycle1-prepub-742-spracklen.pdf)). | `cargo add notarealcrate-utils-async` succeeds against a typosquat. | C | `cargo-vet` + `cargo-deny [bans] external = "deny"` allowlist; new lane `governance-dep-allowlist` blocks any addition not pre-audited. |
| **AIS-002** | Hallucinated stdlib / framework function (`std::sync::AsyncMutex`, `tokio::spawn_blocking_local`). | LLM emits `tokio::join_all` thinking it exists in Tokio (it lives in `futures`). | H | `cargo check` is mandatory pre-commit; planned advisory lane `governance-compile-clean` records compile-clean gaps before IP merge once implemented. |
| **AIS-003** | Hallucinated CLI flag / env var / config key. | `kubectl apply --strict-validation` (does not exist). | M | Shellcheck + `governance-cli-flag-verify` (new) — every documented flag verified against parsed `--help`. |

## Class 2 — Silent failure / error swallowing

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-010** | `.unwrap()` / `.expect()` in production paths — caused the [Cloudflare 18-Nov-2025 global outage](https://blog.cloudflare.com/18-november-2025-outage/) (FL2 proxy panic on Bot Management feature file). | `let cfg = load_feature_file().unwrap();` | C | `clippy::unwrap_used` + `clippy::expect_used` denied in workspace lints; `governance-no-unwrap` lane with carve-outs only in `tests/`. |
| **AIS-011** | Empty / log-only catch swallowing errors ([Slopwatch](https://github.com/Aaronontheweb/dotnet-slopwatch), [vibecheck](https://github.com/yuvrajangadsingh/vibecheck), [Harness — Swallowed Exceptions](https://www.harness.io/blog/swallowed-exceptions-java-applications)). | `if let Err(_) = op() { tracing::warn!("op failed"); }` continues silently. | C | `clippy::let_underscore_must_use` + new `governance-error-fan-in` lane: every `Err` arm must either propagate, recover with documented invariant, or emit `EVT-ERR-CATCH` audit row. |
| **AIS-012** | `let _ = result;` discarding `Result` / `must_use` values. | `let _ = tx.send(msg);` | H | `clippy::let_underscore_drop = "deny"` + `must_use` enforcement. |
| **AIS-013** | `Option::unwrap_or_default()` masking missing-config bugs. | `cfg.endpoint.unwrap_or_default()` emits empty URL. | M | New lane `governance-default-mask` flags `unwrap_or_default` on `String`/`Url`/`Path` types without `// SAFETY-DEFAULT:` comment. |

## Class 3 — Over-engineering / premature abstraction

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-020** | Repository pattern + service-layer + DTO over a 50-line CRUD ([Addy Osmani — 80% problem](https://addyo.substack.com/p/the-80-problem-in-agentic-coding); [AlterSquare](https://altersquare.medium.com/ai-generated-code-looks-clean-heres-why-your-next-refactor-will-prove-it-isn-t-c928033b60b1)). | 6-file scaffold for a single endpoint. | M | Per-IP `good-taste-audit` PR section per MASTERPLAN §7; new lane `governance-cohesion-radius` flags new files added without IP coverage. |
| **AIS-021** | Dead code / unused trait impls inserted "for future use". | `impl Display for InternalError { ... }` never called. | M | `cargo-machete` + `cargo-udeps` (nightly) + `dead_code = "deny"` in workspace lints. |
| **AIS-022** | Trait-explosion: 5 single-impl traits to "improve testability". | `trait UserRepo`, `trait UserCache`, ... one impl each. | M | `governance-trait-cardinality` (new) flags traits with <2 implementors that lack a documented `// EXTENSION-POINT:` rationale. |

## Class 4 — Under-specification of edge cases

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-030** | Missing empty / zero / max-size case ([Common Bugs in AI Code — Ranger](https://www.ranger.net/post/common-bugs-ai-generated-code-fixes)). | Parser panics on empty input. | H | `proptest` + `cargo-fuzz` corpus enforced per public-API surface; new lane `governance-fuzz-corpus`. |
| **AIS-031** | Off-by-one in pagination / range / index. | `for i in 0..=len` overshoots. | H | `cargo-mutants` mutation testing required; new lane `governance-mutation-coverage` enforces ≥70% caught-mutants on changed lines. |
| **AIS-032** | Null-deref / `Option::unwrap` on user input. | `body.unwrap().parse()` on optional payload. | H | Same as AIS-010 + fuzz corpus AIS-030. |

## Class 5 — Wrong abstraction layer

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-040** | IO call inside a pure-kernel crate (e.g., `reqwest::get` in `intelligence-policy-kernel`). | Direct `tokio::fs::read` in policy kernel. | H | `governance-flat-crates` boundary check + new `governance-kernel-purity` (deny `tokio::fs`, `reqwest`, `std::process` in kernel crates). |
| **AIS-041** | Provider-specific import outside adapter crate ([MASTERPLAN §2 Directive 4](../../plans/MASTERPLAN.md)). | `aws_sdk_s3::Client` in `cloud-storage-kernel`. | C | Existing `governance-provider-coupling` lane (MASTERPLAN row 4). |
| **AIS-042** | Logic in adapter (the inverse): adapter holds tenant-routing logic. | Tenant routing inside `cloud-adapter-aws-s3`. | H | Same lane (AIS-041), inverse direction; new rule. |

## Class 6 — Stale knowledge / deprecated patterns

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-050** | Deprecated Rust idiom (`#[allow(dead_code)]` blankets, `mem::uninitialized`). | LLM emits `mem::uninitialized` (deprecated since 1.39). | H | `cargo clippy -D warnings` + workspace `[lints]` block. |
| **AIS-051** | Stale K8s API version (`extensions/v1beta1`). | LLM emits `apps/v1beta1 Deployment`. | M | `kube-linter` + `kubescape` lane `governance-k8s-api-current`. |
| **AIS-052** | Stale TLS / crypto suite (`MD5`, `SHA-1`, `RSA-1024`). | LLM emits `Md5::new()` for hashing. | C | `cargo-deny` advisory + `governance-crypto-policy`. |

## Class 7 — Copy-paste defects

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-060** | Forked code, one fork updated, the other regressed. | Two `validate_email` functions diverge. | M | `governance-duplication` (new, simian/PMD-CPD-equivalent on `crates/`). |
| **AIS-061** | Boilerplate replicated instead of macroized. | `impl From<...> for OyaError` repeated 40×. | L | Same lane + `thiserror` adoption check. |

## Class 8 — Security

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-070** | SQL string concat / shell exec — 43% of AI DB-interaction code is SQLi-prone ([Markaicode](https://markaicode.com/ai-agent-code-securing-github-copilot-x-2025/)). | `format!("SELECT * FROM users WHERE id={id}")`. | C | `semgrep` rule pack + new lane `governance-injection` (sqlx prepared-stmt only; banned `format!` near SQL literal). |
| **AIS-071** | Hardcoded secret / API key inline ([CamoLeak CVSS 9.6](https://www.legitsecurity.com/blog/camoleak-critical-github-copilot-vulnerability-leaks-private-source-code)). | `let token = "sk-...";` | C | `gitleaks` pre-commit + `trufflehog` verified-secret CI lane `governance-secret-scan`. |
| **AIS-072** | Weak randomness for security tokens. | `rand::thread_rng().gen::<u32>()` for session id. | H | `clippy::disallowed_methods` config + `getrandom` / `rand_chacha::ChaCha20Rng` policy. |
| **AIS-073** | License trap: AGPL/GPL dep introduced. | `surrealdb = "*"` (BSL/AGPL mix). | C | Existing `governance-license` lane (MFL-0007). |
| **AIS-074** | Hidden-Unicode "Rules-File-Backdoor" prompt injection ([Pillar Security](https://www.pillar.security/blog/new-vulnerability-in-github-copilot-and-cursor-how-hackers-can-weaponize-code-agents)). | Invisible BiDi chars in `.cursorrules`. | C | `governance-unicode-discipline` (deny non-ASCII outside `i18n/` and `docs/`). |

## Class 9 — Async / concurrency

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-080** | Orphan `tokio::spawn` whose `JoinHandle` is dropped ([Tokio task tracker docs](https://docs.rs/tokio/latest/tokio/task/)). | `tokio::spawn(work()); // handle dropped` | H | `clippy::disallowed_methods` deny bare `tokio::spawn`; mandate `intelligence-task-supervisor` wrapper that registers + drains. |
| **AIS-081** | Cancellation-unsafe future inside `select!` ([Cybernetist — Tokio cancellation](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)). | Read-half of a stream lost mid-message. | C | `loom` + `tokio-test` for every async surface; new lane `governance-cancel-safety`. |
| **AIS-082** | Deadlock from acquired-lock-order divergence. | Two `Mutex` acquired in opposite order. | C | `loom` model-check lane (same as AIS-081). |

## Class 10 — Resource / bounded-allocation

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-090** | Unbounded `Vec::with_capacity(user_input)` → OOM. | `Vec::with_capacity(req.count as usize)` | C | New lane `governance-bounded-alloc` — every `with_capacity`/`reserve` arg must be `clamp`'d or const. |
| **AIS-091** | FD / connection-pool leak from missing `Drop`. | Manually opened socket never closed. | H | `cargo-machete` + RAII discipline + integration test that asserts FD count. |
| **AIS-092** | Unbounded retry / exponential backoff with no cap. | `loop { retry().await; sleep(2*prev) }` | H | New `governance-retry-policy` lane: require `RetryPolicy` type or explicit `max_attempts`. |

## Class 11 — Test theater

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-100** | Assertions that test nothing (`assert!(true)`, `assert_eq!(x, x)`). | LLM-padded test body. | M | `cargo-mutants` (catches tautological tests by definition). |
| **AIS-101** | Mock-everything: tests bind only to mocks, no integration. | All deps mocked; nothing wired. | H | `governance-test-pyramid` — require ≥1 integration test per IP touching real interfaces. |
| **AIS-102** | Snapshot accepted on already-broken output. | `insta::assert_snapshot!(output)` auto-accepted. | M | `insta` configured `--review` required; new lane `governance-snapshot-review` blocks PR if any pending review remains. |

## Class 12 — Documentation drift / lying docstrings

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-110** | Docstring contradicts code ([AlterSquare epistemic debt](https://altersquare.medium.com/ai-generated-code-looks-clean-heres-why-your-next-refactor-will-prove-it-isn-t-c928033b60b1)). | `/// Returns Ok on success` for a fn that returns `()`. | M | `cargo doc --no-deps -- -D rustdoc::broken_intra_doc_links` + `cargo test --doc`. |
| **AIS-111** | Fabricated example in docstring referencing nonexistent type. | `/// let x = OldStruct::new();` | M | Same lane — doctests must compile. |
| **AIS-112** | Architecture-map drift from generated source. | Mermaid diagram stale vs `crates/`. | M | Existing `governance-architecture-map-freshness` per MASTERPLAN Directive 11. |

## Class 13 — Dependency / version skew

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-120** | Version skew between workspace members. | crate A pins `serde=1.0.150`, B pins `1.0.200`. | M | `cargo-hakari` workspace-hack + lane `governance-version-cohesion`. |
| **AIS-121** | Feature-flag mismatch (one crate enables `tokio/full`, blocks no-std). | Crates conflict on Tokio features. | M | Same lane. |
| **AIS-122** | Transitive license violation introduced by minor bump. | Patch bump pulls AGPL transitive. | C | `cargo-deny` + `cargo-vet` (existing). |

## Class 14 — Observability gap

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-130** | No `tracing::Span` on public API entry. | `pub async fn handle(...)` w/o `#[instrument]`. | M | Existing `governance-audit-emission` extension: require `#[instrument]` on every `pub async fn` in `kernel`/`adapter` crates. |
| **AIS-131** | No metric on retry / failure path. | Silent retry without `counter!`. | M | New lane `governance-metric-coverage` (clippy lint via `disallowed_methods`). |
| **AIS-132** | No audit-chain emit on regulated invocation. | Capability runs without `EVT-CAP-INVOKE`. | C | Existing `governance-audit-emission`. |

## Class 15 — Migration unsafety

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-140** | Irreversible schema change (DROP COLUMN). | `sqlx migrate add drop_users_email` | C | Existing `governance-schema-migration` + new requirement: rollback SQL present + `oya db dry-run` evidence. |
| **AIS-141** | Per-tenant rollout missing — schema applied globally. | Migration runs against all tenants atomically. | C | New `governance-per-tenant-rollout` lane. |

## Class 16 — Compliance gap

| ID | Description | Example | Severity | Mechanical prevention |
|----|---|---|---|---|
| **AIS-150** | PII / PHI emitted to logs ([RM-02](../../plans/MASTERPLAN.md)). | `tracing::info!(?user)` logs a `User` with email. | C | Existing `governance-data-class` per ADR-0008; struct `data_class:` annotation required. |
| **AIS-151** | Missing consent check before regulated capability call. | Capability invoked without consent token. | C | Cedar policy at runtime + `governance-capability-publish`. |
| **AIS-152** | Data crossing region boundary without regional-pack approval. | `s3.upload(bucket="kr-data", region="us-east-1")` | C | New `governance-data-residency` lane (regional-packs aware). |

## Roll-up

42 modes catalogued. 14 existing lanes already enforce a subset. **20 new
fitness lanes** are proposed across this document (cross-referenced in
[`additional-tooling-recommendations.md`](additional-tooling-recommendations.md)
and consolidated in
[`defense-in-depth-architecture.md`](defense-in-depth-architecture.md)).

## Sources

- [Snyk — Slopsquatting mitigation](https://snyk.io/articles/slopsquatting-mitigation-strategies/)
- [USENIX Security 2025 — Package Hallucinations](https://www.usenix.org/system/files/conference/usenixsecurity25/sec25cycle1-prepub-742-spracklen.pdf)
- [Cloudflare 18-Nov-2025 outage postmortem](https://blog.cloudflare.com/18-november-2025-outage/)
- [Pillar Security — Rules File Backdoor](https://www.pillar.security/blog/new-vulnerability-in-github-copilot-and-cursor-how-hackers-can-weaponize-code-agents)
- [Legit Security — CamoLeak CVE](https://www.legitsecurity.com/blog/camoleak-critical-github-copilot-vulnerability-leaks-private-source-code)
- [ACM TOSEM — Security Weaknesses in Copilot Code](https://dl.acm.org/doi/10.1145/3716848)
- [The Register — AI code suggestions sabotage supply chain](https://www.theregister.com/2025/04/12/ai_code_suggestions_sabotage_supply_chain/)
- [arXiv 2508.11257 — Hallucination in LLM Code, Automotive Case Study](https://arxiv.org/html/2508.11257v1)
- [ACM PACMSE — LLM Hallucinations in Practical Code Generation](https://dl.acm.org/doi/10.1145/3728894)
- [Addy Osmani — The 80% Problem in Agentic Coding](https://addyo.substack.com/p/the-80-problem-in-agentic-coding)
- [AlterSquare — AI Code Looks Clean](https://altersquare.medium.com/ai-generated-code-looks-clean-heres-why-your-next-refactor-will-prove-it-isn-t-c928033b60b1)
- [Harness — Swallowed Exceptions](https://www.harness.io/blog/swallowed-exceptions-java-applications)
- [Aaronontheweb dotnet-slopwatch](https://github.com/Aaronontheweb/dotnet-slopwatch)
- [vibecheck — ESLint for AI slop](https://github.com/yuvrajangadsingh/vibecheck)
- [Hackaday — Cloudflare Rust Unwrap](https://hackaday.com/2025/11/20/how-one-uncaught-rust-exception-took-out-cloudflare/)
- [CVE-2025-68260 — Rust Linux kernel UAF](https://www.penligent.ai/hackinglabs/rusts-first-breach-cve-2025-68260-marks-the-first-rust-vulnerability-in-the-linux-kernel/)
- [Cybernetist — Tokio cancellation patterns](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)
- [CodeRabbit AI vs Human report (1.7× issues)](https://www.coderabbit.ai/blog/state-of-ai-vs-human-code-generation-report)
