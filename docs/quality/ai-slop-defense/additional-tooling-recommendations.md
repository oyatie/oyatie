---
doc_class: ToolingRecommendations
parent: INDEX.md
status: Accepted
purpose: |
  Concrete tooling additions that close the gaps identified in
  gap-analysis-ai-vs-production.md. Each tool entry: name + purpose +
  adoption priority + fitness-lane target + ADR target.
owner: axis-foundry + council-architecture
date: 2026-05-12
adr_citations:
  - ADR-0053
  - ADR-0055
doc_status: published
---

# Additional Tooling Recommendations

> Priority scale:
> **BLOCKER** — must ship before M01 closes.
> **HIGH** — must ship before M02 (Foundry-Preview) closes.
> **MED** — must ship before M03 (Cloud/SaaS/Search/Workspace Preview).
> **LOW** — M01+ hygiene backlog.
>
> Accepted per ADR-0053 (fitness-lane governance) and ADR-0055
> (impossible-to-fail environment contract). BLOCKER tools are binding
> commitments for M01 per ADR-0055 §8 bootstrapping order.

## 1. Rust correctness + safety verification

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **cargo-mutants** | Mutation testing — catches tautological / test-theater assertions ([AIS-100, AIS-031](ai-slop-failure-mode-catalogue.md)). Killed-mutant rate ≥70% on diff. ([mutants.rs](https://mutants.rs/)) | **BLOCKER** | `governance-mutation-coverage` | ADR-XXX-mutation-coverage |
| **cargo-fuzz** (libFuzzer) | Coverage-guided fuzzing for parsers/serializers/public APIs. ([rust-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)) | **BLOCKER** | `governance-fuzz-corpus` | ADR-XXX-fuzz-policy |
| **Kani** | Model checker for `unsafe` blocks + FFI surfaces (AWS-pioneered for Firecracker). ([AWS Kani blog](https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/)) | HIGH | `governance-kani` (nightly job) | ADR-XXX-kani |
| **MIRI** | Undefined-behavior interpreter on `#[cfg(miri)]` tests. | HIGH | `governance-miri` (nightly) | ADR-XXX-miri |
| **loom** | Concurrency model-check for `async` + atomics + locks. ([Cybernetist](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)) | **BLOCKER** | `governance-cancel-safety` | ADR-XXX-loom |
| **proptest** | Property-based testing — universal-truth assertions. | HIGH | extends `governance-test-pyramid` | (part of ADR-XXX-test-pyramid) |
| **insta** | Snapshot testing with required `--review` gate to defeat AIS-102. | HIGH | `governance-snapshot-review` | ADR-XXX-snapshot-review |

## 2. Rust supply-chain + dependency hygiene

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **cargo-vet** | Human-audit trail for every third-party crate (Mozilla; AWS sharing audits). ([Mozilla cargo-vet](https://mozilla.github.io/cargo-vet/)) | **BLOCKER** | `governance-dep-allowlist` | ADR-XXX-cargo-vet |
| **cargo-deny** | License + advisory + bans (already adopted) — extend `[bans]` to enforce allowlist mode. | HIGH | extends existing `governance-license` | (extension to ADR-0013) |
| **cargo-audit** | RustSec advisory feed. (already adopted) | — | existing | — |
| **cargo-semver-checks** | Detects SemVer breakage on lib crates. ([cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)) | HIGH | `governance-semver` | ADR-XXX-semver |
| **cargo-public-api** | Surface diff for `[lib]` crates. | HIGH | same lane | same ADR |
| **cargo-hakari** | Workspace-hack — deduplicates feature flags across workspace. | MED | `governance-version-cohesion` | ADR-XXX-hakari |
| **cargo-machete** | Fast text-based unused-dep detection. | HIGH | `governance-unused-deps` | ADR-XXX-unused-deps |
| **cargo-udeps** (nightly) | Compiler-driven unused-dep — more accurate, slower. | HIGH | same lane | same ADR |
| **cargo-binstall** | Binary-install hygiene for CI runners (no opaque curl-pipe-bash). | LOW | n/a (CI policy) | ADR-XXX-binstall |
| **cargo-auditable** | Embeds SBOM in binary for retro-scan. | MED | extends supply-chain lane | — |

## 3. Static analysis + semantic scanning

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **Semgrep** | Semantic grep for AI-slop patterns + OWASP + custom rules. ([Semgrep AI](https://semgrep.dev/blog/2025/ai-powered-detection-with-semgrep/)) | **BLOCKER** | `governance-injection` + multi-class slop rules | ADR-XXX-semgrep |
| **gitleaks** | Pre-commit secret scan (fast). ([Gitleaks](https://github.com/gitleaks/gitleaks)) | **BLOCKER** | `governance-secret-scan` (pre-commit half) | ADR-XXX-secrets |
| **trufflehog** | CI verified-credential scan (deeper). ([TruffleHog](https://github.com/trufflesecurity/trufflehog)) | **BLOCKER** | same lane (CI half) | same ADR |
| **osv-scanner** | Cross-ecosystem vuln DB scan. | HIGH | `governance-osv` (new) | ADR-XXX-osv |
| **dependency-review-action** | GitHub-native dep-change review on PR. | MED | wraps `cargo-vet` evidence | — |
| **commitlint** | Conventional-commit enforcement. | LOW | `governance-commit-shape` | ADR-XXX-commits |
| **cocogitto** | Changelog automation + commit semantics. | LOW | extends `governance-changelog` | — |
| **release-please** | Release-PR automation per provider-agnostic adapter. | LOW | same lane | — |

## 4. Policy + runtime authorization

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **Cedar** | Already adopted for autonomy ceiling — extend to per-tool agent invocation. ([Cedar policy lang](https://www.strongdm.com/cedar-policy-language)) | HIGH | existing `governance-capability-publish` | extension of ADR-0007 |
| **OPA** (option) | Alternative policy engine; oyatie's decision is Cedar per existing ADR — keep this row to document the alternative. | LOW | — | — |

## 5. Container + runtime security

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **Cosign (Sigstore)** | Keyless OIDC signing + attestation. ([Sigstore](https://docs.sigstore.dev/cosign/verifying/attestation/)) | **BLOCKER** | `governance-supply-chain` | ADR-XXX-supply-chain |
| **Syft** | SBOM generation (CycloneDX + SPDX). | **BLOCKER** | same lane | same ADR |
| **SLSA L2/L3 provenance** | Build-provenance attestation. ([SLSA L3](https://oneuptime.com/blog/post/2026-02-09-slsa-level3-build-provenance/view)) | **BLOCKER** | same lane | same ADR |
| **Rekor** | Transparency log. | **BLOCKER** | same lane | same ADR |
| **Kyverno** or **cosigned** | Admission-controller image-signature verify. | HIGH | extends `governance-image-discipline` | — |
| **Falco** | eBPF runtime threat detection + syscall observability. ([Falco docs](https://falco.org/docs/)) | HIGH | `governance-runtime-threat` (new) | ADR-XXX-falco |
| **KubeLinter** | K8s YAML lint. | MED | `governance-k8s-lint` (new) | ADR-XXX-k8s-policy |
| **Kubescape** | K8s policy + posture scan (NSA/CIS). | MED | same lane | same ADR |
| **chaos-mesh** | Chaos engineering rehearsal (Layer 7 rollback evidence). ([Chaos Mesh](https://chaos-mesh.org/)) | HIGH | `governance-chaos-drill` (new) | ADR-XXX-chaos |
| **Atlantis** | Terraform PR automation if/when IaC lands. | LOW | n/a | — |

## 6. Observability + dev-loop

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **OpenTelemetry SDK + Collector** | Already in MASTERPLAN; ensure span coverage. | HIGH | `governance-trace-coverage` + `governance-metric-coverage` (new) | ADR-XXX-otel-coverage |
| **bacon** | Background `cargo check` for dev-loop. | LOW | n/a (AGENTS.md default) | — |
| **tokio-console** | Async runtime introspection. | LOW | n/a (dev tool) | — |
| **rust-analyzer** | LSP discipline (dev tool). | LOW | n/a | — |
| **sccache** | Compiler cache (S3-backed). | MED | speedup, no lane | — |
| **cargo-chef** | Docker-layer dep cache. | MED | speedup, no lane | — |

## 7. AI-slop-specific defenses

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **slopwatch / vibecheck-class linters** | AI-pattern denylist (empty catch, swallowed err, snapshot-on-broken). ([slopwatch](https://github.com/Aaronontheweb/dotnet-slopwatch); [vibecheck](https://github.com/yuvrajangadsingh/vibecheck)) | HIGH | `governance-error-fan-in` + `governance-snapshot-review` | ADR-XXX-slop-linters |
| **Unicode-discipline lane** | Block BiDi-control characters in source / `.cursorrules` / `.claude/`. Defeats Pillar Security "Rules File Backdoor". ([Pillar](https://www.pillar.security/blog/new-vulnerability-in-github-copilot-and-cursor-how-hackers-can-weaponize-code-agents)) | **BLOCKER** | `governance-unicode-discipline` | ADR-XXX-unicode |
| **Redundant-generation diff-vote** | Generate change with Claude + Codex + Gemini; merge only if 2-of-3 agree on the diff structure ([oh-my-claudecode ccg](https://github.com/.../oh-my-claudecode); `omc ccg` skill). | MED | `governance-diff-vote` (new, opt-in per high-blast-radius change) | ADR-XXX-diff-vote |
| **RepoMap / repo-graph context** | Improves AI-context quality; reduces hallucinated APIs by 6.8× ([code-review-graph](https://github.com/tirth8205/code-review-graph)). | MED | `governance-context-quality` (new) | ADR-XXX-repomap |
| **Replay-as-eval** (existing ADR-0024) | Replay past failure modes against current `main`. | HIGH | extends `governance-mistakes-ledger-cite` | extension |

## 8. Python / mixed-language tooling

| Tool | Purpose | Priority | Fitness lane | ADR target |
|---|---|---|---|---|
| **ruff** | Python lint + format (if any Python lands in `tools/` or scripts). | LOW | `governance-python-lint` (new, conditional) | — |
| **mypy** | Python type check. | LOW | same lane | — |

## Roll-up

**Top-5 highest-impact tooling additions** (BLOCKER tier, biggest gap
coverage):

1. **Workspace `[lints]` deny block + clippy `disallowed_methods`** —
   closes AIS-010/011/012/072/080 + Cloudflare-class panics in one
   move; zero new infra.
2. **Cosign + Syft + SLSA L2/L3 + Rekor** — closes the "no signed
   provenance" gap that already shipped as the #1 hyperscaler-best-practices
   gap.
3. **cargo-vet + dep-allowlist + Unicode-discipline lane** — closes
   slopsquat (AIS-001) + rules-file-backdoor (AIS-074) which are
   active 2025 attack surfaces.
4. **cargo-mutants + cargo-fuzz + loom + insta** — closes the entire
   test-theater + cancel-safety + edge-case class (AIS-030/031/081/082/100/102).
5. **Semgrep + gitleaks + trufflehog** — closes injection + hardcoded
   secret + verified-credential leakage (AIS-070/071).

## Adoption-order constraint

Tools BLOCKER before HIGH before MED before LOW; within a tier,
prefer tools that close ≥2 modes. `cargo-vet` + Cosign + Semgrep are
each multi-mode and ship first.

## Sources

- [cargo-mutants](https://mutants.rs/)
- [rust-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [AWS Kani blog](https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/)
- [Mozilla cargo-vet](https://mozilla.github.io/cargo-vet/)
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
- [Sigstore Cosign](https://docs.sigstore.dev/cosign/verifying/attestation/)
- [SLSA L3 OneUptime](https://oneuptime.com/blog/post/2026-02-09-slsa-level3-build-provenance/view)
- [Semgrep AI-powered detection](https://semgrep.dev/blog/2025/ai-powered-detection-with-semgrep/)
- [TruffleHog](https://github.com/trufflesecurity/trufflehog)
- [Gitleaks](https://github.com/gitleaks/gitleaks)
- [Falco](https://falco.org/docs/)
- [Chaos Mesh](https://chaos-mesh.org/)
- [Cedar policy language](https://www.strongdm.com/cedar-policy-language)
- [slopwatch (dotnet)](https://github.com/Aaronontheweb/dotnet-slopwatch)
- [vibecheck](https://github.com/yuvrajangadsingh/vibecheck)
- [code-review-graph](https://github.com/tirth8205/code-review-graph)
- [Pillar Security — Rules File Backdoor](https://www.pillar.security/blog/new-vulnerability-in-github-copilot-and-cursor-how-hackers-can-weaponize-code-agents)
- [Cybernetist — Tokio cancellation patterns](https://cybernetist.com/2024/04/19/rust-tokio-task-cancellation-patterns/)
