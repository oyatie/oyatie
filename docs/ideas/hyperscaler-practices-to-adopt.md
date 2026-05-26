# Hyperscaler practices to adopt — backlog (2026-05-26 research)

Two best-practice-research passes (Google; then AWS/Meta/MS/Netflix). Excludes already-adopted
baseline (ADR-SSOT+generated masterplan, Diátaxis, docs-as-code+CODEOWNERS, trunk monorepo +
affected-targets + merge-queue + caching, contract/schema governance, cells+shuffle-shard,
SLO-gated progressive delivery, SLSA-L3+cosign, OSI-strict). Theme for a solo-founder + AI-agent
fleet dogfooding its own cloud: **make implicit safety explicit + machine-enforced, because agents
merge unsupervised.** Each adopt-item should land as a gate/spec/ADR/tool (per ADR-0365 pipeline).

## Prioritized adopt backlog (highest leverage first)

| # | Practice | Source-co | Manifests here as | Effort | When |
|---|----------|-----------|-------------------|--------|------|
| 1 | **one-way/two-way-door field on ADRs** | AWS | `door:` front-matter + `decision-door` gate → agent autonomy boundary (ADR-0365 D5) | S | now |
| 2 | **error-budget *policy* + freeze gate** | Google | `specs/error-budget-policy.json` + promotion-freeze when over budget | S | now |
| 3 | **COE → gate flywheel (5-whys→infra)** | AWS+Meta | `evidence/coe/` + `coe-to-gate` gate; bans "agent error" root cause (ADR-0365 D6) | M | now |
| 4 | **flaky-test auto-quarantine** | Google | nextest retry + flake ledger; quarantine off blocking gate (protects auto-merge) | M | now |
| 5 | **Renovate: grouped auto-merge + allowlist gate** | Google | `renovate.json` cargo-workspace; auto-merge patch behind gate + blocks off-allowlist | S | now |
| 6 | **DORA four keys from audit-chain** | Google | `oya dora` from evidence/audit-chain.jsonl + ArgoCD events; closure metric | S/M | now |
| 7 | **Definition-of-Done as completion gate** | Google/DORA | `specs/definition-of-done.json` (tests+docs+ADR+SLO+changelog+evidence) | S | now |
| 8 | **PR-FAQ before any new service/lane** | AWS | `specs/pr-faq/<x>.md` gate before a manifest exists (whether-to-build) | S | now |
| 9 | **Automated Canary Analysis (Kayenta-style)** | Netflix | Argo Rollouts AnalysisTemplate + Prometheus → statistical promotion judge | M | now |
| 10 | **ORR promotion gate that self-grows from COEs** | AWS | `oya verify --orr` pre-prod checklist; each COE adds a question | M | now |
| 11 | **continuous fuzzing on untrusted-input crates** | Google | cargo-fuzz on http-router-kernel/IaC/metering + ClusterFuzzLite postsubmit | M | now |
| 12 | **SDL threat-model-as-gate, re-validated on change** | MS | `specs/threat-model/<svc>.json` required at promotion; pairs w/ T3 identity | M | now |
| 13 | **semantic code index (agent-queryable xref)** | Google | Sourcegraph-OSS / rust-analyzer index; `oya xref`; force-multiplies agents | S/M | now |
| 14 | **single-threaded owner = owner-agent per service (full-cycle)** | AWS+NFLX | owner persona in catalog YAML, spans PR-FAQ→deploy→SLO→COE | S/M | now |
| 15 | **project-of-record + drift gate** | Google | one canonical tracker; extend planning-drift gate (resolves task #50) | S/M | now |
| 16 | **presubmit/postsubmit two-tier + regression attribution** | Google | `oya verify --presubmit` vs scheduled postsubmit that bisects culprit | S | now |
| 17 | **"Oyatie 1ES" — name+lock the one blessed system** | MS | doc Forgejo+Jenkins+ArgoCD+oya as SSOT; gate: no alternate CI path | S | now |
| 18 | **chaos engineering / GameDay on k3s farm** | NFLX+AWS | Chaos Mesh pod-kill/latency, SLO-gated | M | now |
| 19 | **stacked diffs on Forgejo** | Meta | ghstack-style small-PR stacks per lane; merge-queue throughput | M | later |
| 20 | **go/ links resolver (own µservice — dogfood)** | Google | `golinks.yaml` → resolver; stable indirection for agents/ADRs | S | now |
| 21 | **deprecation lifecycle spec + sunset gate** | Google | `specs/deprecation-lifecycle.json`; flag `#[deprecated]` past sunset | S | now |
| 22 | **changelog discipline (git-cliff + conventional commits)** | Google | per-crate CHANGELOG generated; gate public-API change → entry | S | now |
| 23 | **Buck2 as Rust-native affected-target/RBE backbone** | Meta | evaluate behind `oya verify --affected` + farm cache | L | later |
| 24 | **ring-based Safe Deployment for *policy* changes** | MS | ring-deploy Kyverno/gate changes, not just app code | M | later |

## How these enter the system
Per ADR-0365, each adopt-item is itself a decision → flows research → consensus-plan → ADR (with
`door:` classification + `deliverables[]` + `affected_surfaces[]`) → auto-propagates. The top ~10
are solo-now, high-leverage, and mostly land as gates/specs (cheap to enforce). Cross-cutting framing:
**Context-not-Control** (Netflix) — give agents context + the door boundary, not exhaustive rules.

Sources: full citations in the 2026-05-26 research agent transcripts; key URLs — abseil.io SWE-book,
sre.google, cloud.google.com/DORA, aws.amazon.com working-backwards/COE/ORR, learn.microsoft.com SDL,
netflixtechblog.com paved-road/chaos/Kayenta.
