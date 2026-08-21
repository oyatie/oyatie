---
purpose: Hosted multi-arch CI + lab CAS direction for agent fleet (idea-refine accepted direction; not merge authority).
doc_status: drafted
related_adrs: [ADR-0515, ADR-0560, ADR-0630]
---

# Hosted multi-arch CI + lab CAS (agent fleet)

**Status:** Accepted direction (idea-refine 2026-08-05); **ARC overflow RETIRED 2026-08-11**. Soft multi-arch stays GitHub-hosted. Lab CAS = founder **laptop** NativeLink (+ tunnel) — sibling CAS track owns bring-up; this doc owns ARC retirement prose.  
**Related:** [buck2#183](https://github.com/facebook/buck2/issues/183), [njaremko/quokka](https://github.com/njaremko/quokka), NativeLink ADR-0560, #1541 secret posture.  
**Runner labels (source):** [GitHub-hosted runners reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

## Problem Statement

How might we give a multi-agent fleet **merge-ready CI without self-hosted babysitting**, **multi-platform portability signal**, **non-zero cache reuse**, and **public-ready secrets**—using GitHub-hosted multi-arch workers as the ephemeral plane and a **lab NativeLink** as the durable cache plane?

## Is `oya-arm64` the best choice?

**No — and the label is RETIRED.** Custom ARC is not merge authority and is no longer lab overflow.

| Option | Role | Verdict |
|--------|------|---------|
| **Lab ARC `oya-arm64` (RETIRED)** | — | **Retired 2026-08-11.** Tip `maxRunners: 0`; remove Argo apps after drain. Do not resurrect. |
| **`ubuntu-latest` (linux/amd64)** | **Binding merge plane** | **Yes.** Standard hosted unit, widest package support, private-repo default. |
| **`ubuntu-24.04-arm` (linux/arm64)** | Soft platform smoke | **Yes as soft.** Hosted arm without babysitting; may be plan-gated on private. |
| **`windows-latest`** | Soft platform smoke | **Yes as soft.** Unlocks cfg(windows) / MSVC reality; minutes cost. |
| **`macos-latest` (arm64)** | Soft platform smoke | **Yes as soft.** Darwin signal; higher $; not product-critical until mac ship. |
| **Matrix everything on all jobs** | — | **No.** FinOps blowup; hyperscalers tier platforms. |

**Hyperscaler pattern we copy:** ephemeral multi-platform workers + durable content-addressed cache + identity at the boundary. Lab metal is one failure domain, not the fleet.

## Recommended Direction

1. **Binding merge CI** on **GitHub-hosted linux/amd64** (`ubuntu-latest`). Full suite (`buck2`, affected-set, freshness, …) stays here.
2. **Multi-arch smoke** (soft, `continue-on-error`) on the lightweight `gate` matrix: linux-arm64, windows-amd64, macos-arm64 — same scm-facts receipt, no 4× full suite.
3. **CAS on founder laptop** (NativeLink cache-only + Cloudflare Tunnel / Access). Hosted workers dial laptop CAS after canary — **not** ARC. Never public unauthenticated gRPC. Warm CAS flip is sibling track; do not couple to ARC teardown.
4. **Any hit metric counts** first (GHA actions/cache + Buck AC). RE/quokka only after measured AC.
5. **Public prep** now; human-only visibility flip. Forks never get lab CAS credentials.

## Platform tiers (FinOps + blast-radius)

| Tier | Platforms | Jobs | Merge effect |
|------|-----------|------|--------------|
| **T0 binding** | `ubuntu-latest` | producer, buck2, affected-set, freshness, drift, firewall, live-postgres, fan-in | Required green |
| **T1 soft smoke** | `ubuntu-24.04-arm`, `windows-latest`, `macos-latest` | `gate` matrix only | Soft red does not block |
| **T2 promote later** | windows-11-arm, macos-intel, larger runners | when product needs them | After T1 green history |
| **Lab overflow** | ~~ARC `oya-arm64` / live-postgres~~ **RETIRED** | — | Soft multi-arch = hosted only; lab CAS = laptop |

Promote a soft platform to binding only when: (a) green for N consecutive PR days, (b) product ships that OS/arch, (c) minute budget accepted.

## MVP Scope

| Wave | In | Out |
|------|----|-----|
| **W1** | Hosted linux-amd64 binding; multi-arch soft gate matrix; hosted Postgres services; rustup via `install-buck2.sh`; multi-OS buck2 pins (linux/darwin/windows × amd64/arm64) | Auto-public; 4× full suite |
| **W2** | ~~ARC client cert mounts~~ **RETIRED with ARC** | — |
| **W3** | Secure tunnel hosted→**laptop** CAS | Public CAS; ARC overflow gone |
| **W4** | Public readiness + secret scan | Auto visibility flip |
| **W5** | RE / quokka if measured | RE workers in MVP |

## W1–W2 ship checklist

- [x] Binding jobs → `ubuntu-latest` (linux-amd64)
- [x] `gate` multi-arch soft matrix (arm/win/mac)
- [x] live-postgres → GH `postgres:16` services + dual bootstrap
- [x] `install-buck2.sh`: rustup bootstrap + Darwin/Windows arm pins
- [x] ARC mTLS client mount + env paths (**historical** — ARC retired)
- [x] This one-pager (multi-arch decision recorded)
- [x] ARC overflow retired in tip (`maxRunners: 0` + workflow path-filter strip)
- [ ] Founder live ops: sync scale-to-zero → drain Pods/PVCs → remove Argo apps; clear forced `oya-arm64` / `oya-live-postgres-arm64` labels (see `infra/arc/README.md`)

## Not Doing (and Why)

- **ARC as primary merge path or overflow** — **RETIRED**; ops tax > benefit; hosted + laptop CAS instead.
- **Full suite × 4 platforms** — constant-work / FinOps; soft smoke first.
- **RE before working CAS** — action cache first industry order.
- **Public unauthenticated CAS** — zero-trust / #1541.
- **New `.sh` exception growth** — merge-base shrink-only ceiling; extend `install-buck2.sh`.

## Public readiness (W4 — human flip only)

1. Secret scan clean on default history.  
2. No lab CAS keys / runner tokens in git.  
3. Forks: no CAS credentials; warm license fail-closed.  
4. Explicit human “go public”; agents never flip visibility.  
5. After public: re-evaluate free arm concurrency and promote soft→binding if green.

## quokka / buck2#183 (W5 only)

Buck2 does not cache test results by default ([#183](https://github.com/facebook/buck2/issues/183)). quokka is deferred until compile AC hit rate is non-zero and wall-clock is still test-bound.

## Ops: ARC client certs — RETIRED

ARC runner mounts are retired with the scale sets. Laptop CAS trust uses Cloudflare Access / mTLS + CAS keys (#1541) on the tunnel path — sibling CAS track. Warm license stays false until integrity-canary green (do not flip in this retirement).

## Lessons from [asterinas/asterinas Actions](https://github.com/asterinas/asterinas/actions)

Public open-source OS project; heavy free-tier GHA use; multi-arch test surface. Distilled for Oyatie (cloud monorepo, single required merge context `oya-ci-required`, Buck2, lab CAS).

### What they do well (copy the pattern, not the product)

| Asterinas pattern | Evidence | Apply to Oyatie |
|-------------------|----------|-----------------|
| **One workflow per platform plane** | `Test x86-64`, `Test riscv64`, `Test loongarch64` as separate Actions entries | After public: split soft smoke into `Test linux-arm64` / `Test windows-amd64` / `Test macos-arm64` workflows so the Actions UI filters like a hyperscaler board — not one opaque mega-run. |
| **Verb-first human names** | Test / Benchmark / Publish / Check / Validate | Rename **display** names toward that taxonomy; keep machine ids stable where branch protection / gates pin them. |
| **snake_case workflow files** | `test_x86.yml`, `publish_docker_images.yml`, `check_licenses.yml` | New workflows: `{verb}_{subject}.yml`. Avoid camelCase and cryptic acronyms in *filenames*. |
| **Job id = kebab role** | `basic-test`, `boot-test`, `regression-test`, `conformance-test` | Prefer `freshness`, `registry-drift`, `workspace-buck2` style over cryptic internal nicknames in **check titles**. |
| **Matrix by concern, fail-fast: false** | lint/compile/usermode/ktest; boot variants | Keep; never fail-fast across platforms (isolate signal). |
| **Hermetic pinned container** | `asterinas/asterinas:0.18.0-…` | We already pin Buck2 digest + rust-toolchain; optional later: public container for contributor parity (not required for W1). |
| **Composite actions** | `./.github/actions/test`, `benchmark` | Prefer shared steps when we add platforms so install-buck2 / materialize stay one path (already `infra/ci/install-buck2.sh`). |
| **Self-hosted only for expensive continuous work** | Benchmarks on `self-hosted` + cron | Laptop NativeLink = durable cache; soft multi-arch = hosted; **not** PR merge authority. ARC overflow retired. |
| **Publish path-filtered + multi-arch images** | `platforms: linux/amd64,linux/arm64` | When publishing images post-public: dual-arch manifests; never single-arch “works on my lab.” |
| **Free-tier disk hygiene** | rm android/dotnet/gcloud; `free-disk-space` action | On public free GHA, add disk reclaim **on heavy jobs only** (we already have runner-disk-reclaim policy for hosted). |
| **Cancel superseded PR runs** | Newer synchronize cancels older | Keep `concurrency: cancel-in-progress` (already). |
| **Check vs Test vs Publish separation** | licenses fast-path ≠ full test ≠ docker publish | Fast policy checks (license/docs) stay thin; full Buck suite stays fat; publish never on untrusted PR forks. |

### What not to copy

- Trailing spaces in workflow `name:` (`"Test x86-64  "`).  
- Unpinned `actions/checkout@master` in licenses workflow.  
- Putting **all** arches on every PR as hard required without tiering (they can afford OS-kernel scope; we must FinOps-tier a monorepo).  
- Renaming the **required status context** casually — ours is ADR-0515 **`oya-ci-required`** and must stay the single branch-protection key until an explicit ADR changes it.

### Once the repo is public (human flip only)

Free linux-arm64 hosted runners and higher free concurrency become real. Sequence:

1. **Day 0 public** — binding remains `Test linux-amd64` plane inside `oya-ci-required`; soft multi-arch stays soft until green streak.  
2. **Day 0+** — free `ubuntu-24.04-arm` no longer “maybe plan-gated”; promote **linux-arm64 smoke → binding** only after N green PR days.  
3. **Fork PRs** — no lab CAS secrets, no writer identity, warm license fail-closed (same as now).  
4. **Optional split workflows** (Asterinas-style Actions board):

   | Workflow file | Display name | Binding? |
   |---------------|--------------|----------|
   | `oya-ci-required.yml` | **Required** (fan-in; keep machine name) | Yes — protected context |
   | `test_linux_amd64.yml` | Test linux-amd64 | Later extract from mega file if needed |
   | `test_linux_arm64.yml` | Test linux-arm64 | Soft → binding |
   | `test_windows_amd64.yml` | Test windows-amd64 | Soft |
   | `test_macos_arm64.yml` | Test macos-arm64 | Soft |
   | `check_licenses.yml` / docs | Check … | Fast, non-Buck |
   | `publish_*.yml` | Publish … | push to release tags only |
   | `benchmark_*.yml` | Benchmark … | schedule + self-hosted/lab only |
   | `cache_integrity_canary.yml` | Cache integrity canary | schedule / trusted |

5. **Never** auto-flip visibility; secret scan + #1541 posture first.

## CI naming standard (hyperscaler-aligned)

**Goals:** scannable Actions UI, stable merge authority, arch explicit, verb-first, machine ids stable under ratchets.

### Layers

| Layer | Rule | Example |
|-------|------|---------|
| **Protected context** | Stable, singular, product-owned | `oya-ci-required` (do not rename without ADR) |
| **Workflow `name:` (UI)** | `Verb [object] [platform]` Title Case | `Test linux-amd64`, `Check docs graph`, `Publish container images`, `Benchmark cache hit` |
| **Workflow file** | `snake_case` `{verb}_{object}.yml` | `test_linux_amd64.yml`, `check_docs_graph.yml` |
| **Job `id`** | kebab-case role | `workspace-buck2`, `live-postgres-adapters` |
| **Job / matrix `name:` (check run title)** | `gate · <discipline> (<platform>)` or `test · <plane>` | `gate · ADR census (linux-amd64 binding)` |
| **Step names** | Stable under automation-language-policy ceiling | Never rename baselined steps in the same PR as logic |

### Verb taxonomy (use only these families)

| Verb | Meaning | Binding? |
|------|---------|----------|
| **Required** | Single fan-in / admission | Always (the protected context) |
| **Test** | Build + test on a platform plane | Tiered by platform |
| **Check** | Policy / license / docs / graph (fast) | Usually yes if cheap |
| **Cache** | Integrity canary / warm license probes | Schedule or trusted |
| **Publish** | Images, crates, docs, releases | Never from fork PRs |
| **Benchmark** | Perf / regression continuous | Schedule; lab or larger runners |
| **Validate** | Schema / SCML-style external contracts | As needed |

### Platform tokens (explicit, no slang)

Use: `linux-amd64`, `linux-arm64`, `windows-amd64`, `windows-arm64`, `macos-arm64`, `macos-amd64`.  
Avoid: `x64` alone, `arm` alone, `oya-arm64` in **public** check titles (label **retired** — do not reintroduce).

### Current → target mapping (incremental; no big-bang rename)

| Today | Target display name | Note |
|-------|---------------------|------|
| `oya-ci-required` | **Required** (keep workflow `name` / context id) | Fan-in machine name stays |
| job `buck2` | `test · workspace (linux-amd64)` | Display only when safe |
| job `gate-affected-target-set` | `test · affected-set (linux-amd64)` | |
| job `gate-generated-artifact-freshness` | `check · freshness` | |
| `docs-graph-drift` | `check · docs graph` | |
| `cache-integrity-canary` | `cache · integrity canary` | |
| soft matrix labels | already `gate · platform smoke (… soft)` | Asterinas-aligned |

Rename **check titles** before **workflow filenames**; filenames last (history + links). Never break `required_status_checks: [oya-ci-required]`.

## Open Questions

- When to promote linux-arm64 soft → binding (after public free arm vs private larger runners)?  
- Tunnel product (Cloudflare vs Tailscale) for W3.  
- Writer cert only on trusted cache-writer vs short-lived OIDC leaves on GHA.  
- Whether macOS minutes are worth binding for this cloud/K8s-first product.  
- Whether to extract Asterinas-style per-arch workflow files in W4 or keep one fan-in file with matrix labels.

## Hyperscaler principles (this change)

1. **Ephemeral multi-platform workers**, durable cache.  
2. **Identity at every hop** (mTLS).  
3. **Tier platforms** — don’t tax every PR with every OS.  
4. **Measure before RE / before soft→binding.**  
5. **Lab = one AZ**, not the global fleet.  
6. **Question every runs-on** — “is oya-arm64 best?” → no; **retired**. Soft multi-arch stays hosted.  
7. **Name for operators** — verb + platform + concern; one stable admission context.  
8. **Public free tier is a product** — disk hygiene, cancel-in-progress, fork fail-closed.
