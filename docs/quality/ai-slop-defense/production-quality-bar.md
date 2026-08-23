---
doc_class: ProductionQualityBar
parent: INDEX.md
status: Accepted
purpose: |
  Define "hyperscaler production quality" as 38 concrete, verifiable
  dimensions. Each dimension binds to (a) a definition, (b) a
  verification method, and (c) the fitness lane that enforces it. The
  bar reflects the AWS / Google / MS / Oracle convergent practice as
  documented in `.omc/scratch/hyperscaler-best-practices-2026-05-12.md`.
owner: council-architecture
date: 2026-05-12
adr_citations:
  - ADR-0053
  - ADR-0055
doc_status: published
---

# Production Quality Bar — 38 Dimensions

> Bar definition: code that satisfies all 38 dimensions clears the AWS
> Operational Readiness Review (ORR), Google SRE Production Readiness
> Review (PRR), Microsoft 1ES Quality Gate, and Oracle OCI Well-Architected
> bar simultaneously
> ([AWS ORR](https://docs.aws.amazon.com/wellarchitected/latest/operational-readiness-reviews/the-orr-tool.html);
> [Google PRR](https://sre.google/sre-book/evolving-sre-engagement-model/);
> [Google launch checklist](https://sre.google/sre-book/launch-checklist/);
> [OCI Well-Architected](https://blogs.oracle.com/cloud-infrastructure/oci-wellarchitected-framework);
> [Microsoft 1ES](https://azure.microsoft.com/en-us/solutions/devops/devops-at-microsoft/one-engineering-system/)).
>
> Accepted per ADR-0053 (fitness-lane governance) and ADR-0055
> (impossible-to-fail measurement contract).

## Group A — API surface discipline (4)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D01** | SemVer compliance | All public APIs version with SemVer; breaking changes bump major. | `cargo-semver-checks` on `[lib]` crates; `cargo-public-api` diff. | `governance-semver` (new). |
| **D02** | Versioned contracts | OpenAPI / gRPC / proto contracts versioned + back-compat tested. | Contract-replay against prior version's golden set. | `governance-contract-parity` (existing per MFL-0013). |
| **D03** | Surface minimality | `pub` only what callers need; everything else `pub(crate)`. | `cargo-public-api` diff + Linus good-taste audit. | Same as D01. |
| **D04** | Deprecation discipline | `#[deprecated(since, note)]` for at least one minor cycle before removal. | Lint + ADR row. | `governance-deprecation-window` (new). |

## Group B — Error handling (3)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D05** | Typed errors at the edge | Library crates expose `enum Error` (via `thiserror`); apps use `anyhow`/`eyre`. | Workspace lint `clippy::map_err_ignore` + manual audit. | `governance-typed-errors` (new). |
| **D06** | No silent failures | Every `Result` consumed or propagated; no `let _ = ...`, no empty catch. | Catalogue AIS-010/011/012/013. | `governance-error-fan-in` (new). |
| **D07** | No production panics | `clippy::unwrap_used`/`expect_used`/`panic`/`todo`/`unimplemented` denied. | Workspace lints. | `governance-no-unwrap` (new). |

## Group C — Async / concurrency (3)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D08** | Cancellation-safe | All `select!` branches are cancel-safe; mpsc/oneshot for state-bearing futures. | `loom` model-check + `tokio-test` virtual time. | `governance-cancel-safety` (new). |
| **D09** | No orphan tasks | Every `spawn` registered with `TaskTracker`; drain on shutdown. | Clippy `disallowed_methods` deny bare `tokio::spawn`. | Same lane as D08. |
| **D10** | Bounded backpressure | All channels bounded; backpressure surfaces to caller. | Grep + manual audit; new lane. | `governance-backpressure` (new). |

## Group D — Resource discipline (3)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D11** | Bounded allocations | No `Vec::with_capacity(user_input)` without clamp. | Lint + audit. | `governance-bounded-alloc` (new). |
| **D12** | RAII / Drop completeness | Resources (fd, socket, conn-pool) released via `Drop`. | Integration test asserts fd count delta = 0. | `governance-fd-leak` (new). |
| **D13** | Bounded retry | Every retry has `max_attempts` + `RetryPolicy` type. | Audit + grep. | `governance-retry-policy` (new). |

## Group E — Input / authn / authz (4)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D14** | Boundary validation | All cross-trust-zone inputs validated (size, charset, struct). | `validator` crate or hand-written `TryFrom`; fuzz corpus. | `governance-boundary-validation` (new). |
| **D15** | Authn before logic | Every handler authenticates before any logic / IO. | Cedar policy + `tower` middleware enforcement; runtime gate. | `governance-authn-first` (new). |
| **D16** | Authz before logic | Every regulated capability authorizes via Cedar before logic. | Cedar policy registry + audit-chain emit. | `governance-capability-publish` (existing). |
| **D17** | Injection-proof | SQL via `sqlx::query!`; shell via `Command` with arg vector; no `format!` near sinks. | `semgrep` + `cargo-deny` ban of `format!` near `Query::raw`. | `governance-injection` (new). |

## Group F — Observability (4)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D18** | Trace coverage | Every `pub async fn` in kernel/adapter has `#[instrument]` span. | Lint + `cargo doc` cross-check. | `governance-trace-coverage` (new). |
| **D19** | Metric coverage | Every retry / failure / cache-miss path emits a metric. | `disallowed_methods` config + audit. | `governance-metric-coverage` (new). |
| **D20** | Log discipline | No PII in logs (data-class annotation); structured (JSON) only. | Existing `governance-data-class`. | Existing + extension. |
| **D21** | Audit-chain emit | Every regulated invocation emits `EVT-CAP-INVOKE` row with tenant + capability + outcome. | Existing `governance-audit-emission`. | Existing. |

## Group G — Migrations (4)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D22** | Up + down migrations | Every migration has reversible companion. | `sqlx migrate revert` in CI. | Existing `governance-schema-migration`. |
| **D23** | Dry-run rehearsal | `oya db dry-run` executed pre-merge. | Evidence in PR description. | Same lane. |
| **D24** | Per-tenant rollout | Migrations applied per-tenant with health gate. | Rollout-runbook entry. | `governance-per-tenant-rollout` (new). |
| **D25** | Per-cell rollback | Rollback procedure declared at cell granularity. | Runbook present. | Existing `governance-runbook-index-resolves`. |

## Group H — Dependencies + supply chain (5)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D26** | LTS-current | Every direct dep tracks current LTS major.minor. | `cargo-outdated` + `cargo-deny` advisory. | Existing `governance-lts-dependency`. |
| **D27** | License-clean | License policy enforced (no AGPL/GPL in product code). | `cargo-deny [licenses]`. | Existing `governance-license`. |
| **D28** | Slopsquat-proof | Every new dep audited via `cargo-vet`; allowlist-mode. | `cargo-vet check` + `cargo-deny [bans]`. | `governance-dep-allowlist` (new). |
| **D29** | Distroless image | Production binaries ship `distroless/cc-debian12` or `chainguard/static`. | Image-size budget + base-image hash assertion. | Existing `governance-image-discipline`. |
| **D30** | Signed + SBOM + provenance | Every artifact Cosign-keyless-signed; SBOM (Syft); SLSA L2+ provenance attestation; Rekor entry. | CI emits attestation; admission controller verifies ([Sigstore](https://docs.sigstore.dev/cosign/verifying/attestation/); [SLSA L3 build provenance](https://oneuptime.com/blog/post/2026-02-09-slsa-level3-build-provenance/view)). | `governance-supply-chain` (new per [hyperscaler-best-practices §Top-3 gap A](../../specs/hyperscaler-best-practices-2026-05-12.md)). |

## Group I — Testing (5)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D31** | Unit + integration + property + e2e | All four tiers present per change. | `cargo nextest run --workspace` (existing required). | Existing. |
| **D32** | Fuzz corpus | Every parser / serializer / public API has `cargo-fuzz` corpus. | Corpus checked into `fuzz/`. | `governance-fuzz-corpus` (new). |
| **D33** | Mutation coverage | `cargo-mutants` ≥70% caught on changed lines. | CI gate. | `governance-mutation-coverage` (new). |
| **D34** | Snapshot review | `insta` pending snapshots reviewed before merge. | `cargo insta review --no-pending = fail`. | `governance-snapshot-review` (new). |
| **D35** | Perf budget | Latency / throughput / memory budget per IP. | `cargo bench` + `criterion` + perf-budget JSON. | Existing `governance-perf-evidence`. |

## Group J — Docs + governance (3)

| # | Dimension | Definition | Verification | Lane |
|---|---|---|---|---|
| **D36** | Doc freshness | rustdoc compiles; doctests pass; Mermaid auto-generated. | `cargo doc -- -D rustdoc::broken_intra_doc_links` + `cargo test --doc`. | Existing `governance-doc-freshness`. |
| **D37** | ADR + CHANGELOG | Every non-trivial change links ADR + CHANGELOG row. | PR template gate. | Existing `governance-adr-citation`. |
| **D38** | MISTAKES-LEDGER pointer | Every Sev-1/2 produces an MFL row + mechanical prevention. | Postmortem gate + lane authoring. | Existing `governance-mistakes-ledger-cite`. |

## Roll-up

- **38 dimensions** total.
- **14 are enforced today** via existing lanes (audit-emission,
  brand-residue, license, data-class, schema-migration, perf-evidence,
  lts-dependency, image-discipline, doc-freshness, adr-citation,
  mistakes-ledger-cite, capability-publish, contract-parity,
  runbook-index-resolves).
- **24 need new lanes** (listed in
  [`additional-tooling-recommendations.md`](additional-tooling-recommendations.md)
  and architected in
  [`defense-in-depth-architecture.md`](defense-in-depth-architecture.md)).

## Sources

- [AWS ORR](https://docs.aws.amazon.com/wellarchitected/latest/operational-readiness-reviews/the-orr-tool.html)
- [AWS ORR custom WAR lens (awslabs)](https://github.com/awslabs/operational-readiness-review-custom-war-lens)
- [Google SRE — PRR](https://sre.google/sre-book/evolving-sre-engagement-model/)
- [Google SRE launch checklist](https://sre.google/sre-book/launch-checklist/)
- [Google SRE postmortem culture](https://sre.google/sre-book/postmortem-culture/)
- [Google SRE workbook — error-budget policy](https://sre.google/workbook/error-budget-policy/)
- [Microsoft 1ES at Azure](https://azure.microsoft.com/en-us/solutions/devops/devops-at-microsoft/one-engineering-system/)
- [Oracle OCI Well-Architected](https://blogs.oracle.com/cloud-infrastructure/oci-wellarchitected-framework)
- [DX — Production readiness checklist](https://getdx.com/blog/production-readiness-checklist/)
- [Sigstore — In-toto attestations](https://docs.sigstore.dev/cosign/verifying/attestation/)
- [SLSA L3 build provenance (OneUptime 2026)](https://oneuptime.com/blog/post/2026-02-09-slsa-level3-build-provenance/view)
- [Mozilla cargo-vet](https://mozilla.github.io/cargo-vet/)
- [cargo-mutants project](https://mutants.rs/)
- [rust-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [Cloudflare 18-Nov-2025 postmortem](https://blog.cloudflare.com/18-november-2025-outage/)
