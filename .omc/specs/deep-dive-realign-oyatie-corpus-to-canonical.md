---
doc_class: DeepDiveSpec
slug: realign-oyatie-corpus-to-canonical
date: 2026-05-20
status: crystallized
source: deep-dive (trace + interview)
trace_path: /Users/jasonlee/oyatie/.omc/specs/deep-dive-trace-realign-oyatie-corpus-to-canonical.md
related_oyatie_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0263-observability-emission-contract
  - ADR-0316-capability-tier-over-product-fragmentation
  - ADR-0321-b2b-saas-industry-leader-coverage
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement
  - ADR-0323-multi-wave-sequencing-doctrine
  - ADR-0324-anti-script-anti-template-doctrine
  - ADR-0327-wave-3-completion-criteria-and-promotion-gates
related_memory:
  - feedback_drift_too_big_2026_05_20
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_verify_deliverables_not_just_line_count_2026_05_20
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_go_with_original_ambition_2026_05_20
---

# Spec: Realign Oyatie Corpus to Canonical Direction

## Objective

Stop authoring drift in the Oyatie corpus and bring every existing artifact into coherence with the canonical unified-ecosystem B2B platform thesis. The corpus has grown to ~500,000+ lines of substantive content this session (30+ ADR cluster + 79 µservices + 175 user journeys + 8 localization packs + 8 compliance packs + capability-tier registry); without realignment, every further authoring wave compounds drift, which produces the WRONG PRODUCT when downstream teams build from contradictory design docs.

**Concrete outcomes:**

1. **Canonical-direction backbone** lands first: ADR-0328 + master-plan-sequencing.json + brief template encode the Big-8-priority sequence + the in-scope vendor universe + the substance bar + the µservice-ownership coherence model. Every future agent dispatch cites this backbone in its brief header.

2. **Per-µservice ownership-coherence audits** land for all 79 µservices (one codex agent per µservice end-to-end; ~10 waves of 8). Each agent reads every artifact under its µservice's path, cross-references against canonical thesis + chat history + root ADRs + other µservices, identifies + remediates internal contradictions, and produces a per-µservice coherence-audit doc.

3. **ADR-0321 cleanup** lands after the canonical backbone: de-duplicate vendor dossiers (D-139=D-149 Fly.io etc.), renumber sections to be in monotonic order, deepen the Big 8 dossier substance bar to match W3-W10 density (130-180 lines per dossier).

4. **Content completion** lands last per the new Big-8-priority sequence: complete all Big 8 vendor dossiers first → long-tail B2B SaaS → cloud-infra + PaaS → developer tools + niche.

**Success criteria:** Zero in-µservice contradictions; zero out-of-numerical-order ADR-0321 sections; zero duplicate vendor dossiers; every Big 8 vendor dossier at substance bar; every µservice has a coherence-audit doc inside its path.

## Tech Stack

(Per Oyatie canonical primitives — already in repo)

- Rust workspace (`Cargo.toml` at root); `oya-dev-cli` is the canonical CLI
- `oya git <subcommand>` for git operations (drop-in replacement); `oya vcs <claim|work|verify|done|status|symbols|queue|watch|promote>` for coordination ratchet compatibility
- OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3 for contracts
- Cedar v4.2 LTS for policy
- documentation-rigor.md §1.1 intern-buildability as the substance bar
- 13-layer enum per ADR-0105 for crate layering
- Codex CLI gpt-5.5 + model_reasoning_effort=xhigh + sandbox=workspace-write for parallel authoring

## Commands

```bash
# Coordination ratchet (required for any file edit by an agent):
./bin/oya vcs claim --agent <name> --intent <intent> <positional-scopes>
./bin/oya vcs verify --agent <name> --evidence "<key:value pairs>" <scopes>
./bin/oya vcs done --agent <name> --evidence "<...>" <scopes>
./bin/oya vcs promote --agent <name> --bundle <bundle-id> --env dev --evidence "<...>" <scopes>

# Codex dispatch pattern (codex-only per active directive):
nohup codex exec --sandbox workspace-write --skip-git-repo-check --model gpt-5.5 -c model_reasoning_effort="xhigh" --color never "<brief>" > /tmp/codex-out-<name>.log 2>&1 &

# Verification:
./bin/oya verify  # repo-wide CI gate run
grep -c "^### Section D-" docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md  # section count
ps aux | grep "codex exec" | grep -v grep | wc -l  # active codex count
```

## Project Structure (Canonical)

```
/Users/jasonlee/oyatie/
├── docs/
│   ├── decisions/        ADRs (ADR-0001..ADR-0327 + ADR-0328 incoming)
│   ├── architecture/     synthesis adjudication / executive briefings / coverage matrix / thesis docs
│   ├── standards/        documentation-rigor / naming-justification / anti-patterns
│   ├── user-journeys/    j01..j180 (175 journeys + 5 migration journeys)
│   ├── personas/         MASTER-ROSTER + ~129 dossiers
│   ├── products/         per-product PRDs
│   ├── onboarding/       intern-week-one / intern-month-one / per-role guides
│   ├── api/              per-µservice API reference docs
│   ├── tutorials/        end-user tutorials
│   ├── customer-success/ demo scripts
│   ├── governance/       risk register
│   ├── investor/         investor materials
│   ├── gtm/              GTM motion playbooks
│   ├── tests/            cross-µservice integration tests
│   ├── architecture/diagrams/  Mermaid architecture diagrams
│   └── architecture/wave-3-g-* / corpus-rigor-audit-* / six-hops-* / etc.  (audit deliverables)
├── microservices/        79 µservices (substrate + B2B-leader + ERP + consumer-product)
│   └── <name>/
│       ├── PRD.md / ARCHITECTURE.md / compliance.md / README.md
│       ├── Cargo.toml / src/
│       ├── contracts/ (OpenAPI/AsyncAPI/proto3)
│       ├── policies/ (Cedar) / runbooks/ / slos/ / dashboards/ / iac/
│       ├── ip/ + IP-*.md (root-level)
│       ├── decisions/ (per-µservice ADRs ADR-MS-*)
│       ├── capability-tiers/ / onboarding/ / faqs/ / tutorials/
│       ├── benchmarks/ / migration-playbooks/ / reference-implementations/
│       ├── packs/ / security/ / test-plans/
│       └── cross-microservice-handoffs.md
├── packs/                Localization packs (KR/EU/US/JP/IN/BR/AU/MX) + compliance overlays
├── registry/             capability-tiers/ / compliance-packs/ / dashboards/ / sample-tenants/ / slo-library/ / workflow-templates/
├── crates/               Rust crates (oya-* including oya-governance-* scaffolds)
├── benchmarks/           Performance benchmark corpus
├── tests/                Cross-µservice integration tests
└── specs/                Machine-readable canonical specs (root-hub-pointers.json / master-plan-sequencing.json / etc.)
```

## Code Style

(For authoring agents producing markdown/YAML/JSON; for Rust src/ scaffolding follows existing crate conventions.)

```markdown
---
doc_class: <Tier-1-document-class>
microservice: <name>
related_oyatie_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0263-observability-emission-contract
status: Proposed
date: 2026-05-20
owner: <team-name>
---

# Title

## §1 — Context (named pressure + named constraints)

Real-world specifics: actual vendor products with versions (e.g., "Kubernetes 1.35 LTS + Cilium 1.18" not "a CNI"). Real regulatory citations with article numbers (e.g., "HIPAA §164.312(b)" not "HIPAA"). Real Cedar permits with named principals/actions/resources/contexts. Real SLO numbers with rationale. Real failure modes named.

## §2 — Decision / Mechanics

NO template-stamping. NO clause-loop padding. Every paragraph must add information density. If you can't write substance, write a SHORTER bespoke version rather than padding.

## §References

- /Users/jasonlee/oyatie/docs/decisions/ADR-0244-*.md §D-11 audience_type enum
- /Users/jasonlee/oyatie/microservices/identity/PRD.md §4.2 passkey-primary doctrine
- /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1 intern-buildability bar
```

**Anti-patterns** (per `feedback_docs_substance_not_scaffold_2026_05_20` + ADR-0324):
- N artifacts sharing the same skeleton with one variable swap (vendor name / µservice name / persona name)
- Line-floor met with generic content
- "the µservice handles X" without naming the actual handler
- Clause-loop padding ("Thesis clause N: ..." repeating)
- Scripting-based generation producing shallow output

## Testing Strategy

Three verification layers:

1. **Per-artifact substance verification** (per ADR-0322): `oya-governance-substance-bar` crate checks line floor; `oya-governance-no-template-stamping` crate detects skeleton-sharing
2. **Per-µservice coherence verification** (per `feedback_microservice_ownership_coherence_2026_05_20`): per-µservice ownership-coherence audit doc inside each `microservices/<name>/coherence-audit-2026-05-20.md`
3. **Cross-corpus drift verification** (per `feedback_drift_too_big_2026_05_20`): six-hops reachability audit + ADR cross-reference graph audit + audit-event coverage sweep + IP cross-reference sweep

**Orchestrator verification SLA on every landing** (per `feedback_verify_deliverables_not_just_line_count_2026_05_20`):
- Read 3 random artifacts from the agent's output
- Cross-check against 5 canonical anchors (relevant root ADR + relevant µservice PRD + Wave-3-G thesis + documentation-rigor + relevant memory directive)
- Block "done" declaration if any fail

## Boundaries

**Always do:**
- Cite the canonical direction at the head of every authoring brief (5-citation header + decision tree)
- Use `oya vcs claim` before file edits; `verify`/`done`/`promote` after
- Apply per-µservice ownership: one agent owns one µservice end-to-end for the ownership-coherence audit wave
- Verify 3 random artifacts + 5 canonical anchors before declaring done
- Treat agent "completed" notifications as backgrounded-not-finished

**Ask first:**
- Adding new vendor categories to ADR-0321 not in the canonical in-scope universe (B2B SaaS / cloud-infra / PaaS / developer tools)
- Modifying canonical-direction docs (Wave-3-G thesis / keystone bundle ADRs / documentation-rigor / master-plan-sequencing.json)
- Adding NEW µservices (the 79 µservice roster is currently frozen)
- Removing or relocating large doc blocks
- Pre-empting the priority sequence (Big 8 first, then long-tail, then cloud-infra+PaaS, then dev tools)

**Never do:**
- Treat agent self-report ("completed") as proof of deliverable
- Use line counts alone as quality verification
- Spawn parallel agents on the same single file without explicit handoff coordination
- Author NEW dossiers/IPs/runbooks without consulting the µservice's own PRD/ARCHITECTURE/IPs first
- Use scripting/metaprogramming/template-substitution for content authoring (per ADR-0324)
- Skip the per-µservice ownership-coherence audit before remediation

## Success Criteria

1. **ADR-0328 + master-plan-sequencing.json + brief-template land** with explicit Big 8 priority sequence + 5-citation header + decision tree
2. **All 79 µservices have a `coherence-audit-2026-05-20.md` file** inside their path documenting findings + remediations
3. **ADR-0321 has zero duplicate vendor dossiers** + zero out-of-numerical-order sections
4. **All 8 Big 8 vendor families have ≥130-line substantive dossiers** at W3-W10 density per ADR-0321
5. **Every per-µservice ADR ≥200 lines** at substance bar (currently developer-sdk 39 / consent-graph 50 / analytics 67 / mail 136 are below)
6. **Every brief used in subsequent waves cites the 5 canonical anchors** in its header per ADR-0328 brief template
7. **`oya verify` is at 81/81** (currently was at 57/81; should be at 81/81 after governance crate impls + manifest fixes land + corpus drift remediation)
8. **Zero P0 findings in a fresh corpus-rigor audit** after all remediation lands

## Trace Findings

Per `.omc/specs/deep-dive-trace-realign-oyatie-corpus-to-canonical.md`: the drift was caused by a 3-layer compounding failure:

1. **Lane 1 — Authoring briefs lacked canonical-direction encoding.** My briefs listed candidate vendors mixed in-scope and out-of-scope without explicit in-scope filter referring to the unified-ecosystem thesis. The D-141..D-155 brief explicitly named Fly.io + MongoDB Atlas + Cloudflare R2 alongside Linear + Notion + Pendo, leaving the agent to author all of them.

2. **Lane 2 — Coordination / concurrency / ownership failed.** Parallel agents stepped on the same ADR-0321 file via `cat >>` appends, creating duplicate vendor sections + out-of-numerical-order sections. Per-µservice ADRs were authored in batches A-F without any single agent owning a full µservice's coherence; each µservice was touched by 5-15 distinct agents.

3. **Lane 3 — Orchestrator verification used line counts + self-report.** Multiple agents reported "completed" with minimal output (D-126..D-140 reported done with 2/15 sections; D-134..D-148 halted mid-sentence with 0 net-new). The drift was invisible until user manually flagged out-of-scope vendors.

The 3 lanes converge on a causal chain — not competing hypotheses. Remediation must address all 3 layers (briefs + ownership + verification) simultaneously, not just one.

**User-correction note (Phase 4 interview):** The vendor scope is BROADER than my Lane 1 hypothesis assumed — all of B2B SaaS + cloud-infra + PaaS + developer tools are IN-SCOPE for ADR-0321. The drift is NOT "wrong vendors" but "missing priority ordering + duplicates + out-of-order + sub-substance + per-µservice incoherence". Realignment is sequence-and-coherence, not removal.

## Multi-Context Platform Constraint (cross-cutting, all phases)

Per `feedback_multi_context_provider_agnostic_2026_05_20.md` (and ADR-0215 multi-context-platform + ADR-0216 open-integration + ADR-0211 in-house-tech-stack + ADR-0254 K8s+Cloud-Hypervisor + ADR-0248 Amazon-shape-cellular):

**Oyatie is a multi-context platform that must support three deployment shapes from a single codebase:**

1. **Hosted on third-party cloud (AWS / OCI / any IaaS) as guest** — same Oyatie stack runs as tenant in AWS/OCI/etc., backed by their IaaS primitives (EC2/EBS/S3/VPC or OCI Compute/Block-Volume/Object-Storage/VCN)
2. **On-prem / colo bare metal** — same Oyatie stack runs in customer-controlled DC or colocation; customer brings hardware
3. **Oyatie-as-cloud-provider** — Oyatie SELLS compute/storage/networking/IAM/KMS/billing as IaaS to external customers (Oyatie becomes a cloud provider in its own right)

**Implication for every phase:**

- Phase 0 cloud-* µservices ARE Oyatie's own IaaS surface (cloud-iam / cloud-kms / cloud-storage / cloud-compute-* / cloud-billing / cloud-network / cloud-marketplace / etc.) — they are NOT wrappers around AWS/OCI primitives; they EXPOSE the portable interface that can be BACKED by AWS/OCI/bare-metal underneath
- Phase 1-4 µservices MUST abstract their underlying storage/compute/network behind the Phase 0 cloud-* µservice interfaces — no code path can hardcode AWS-specific or OCI-specific assumptions
- Every µservice's PRD/ADR/IP must explicitly enumerate which of the three deployment contexts it supports
- Capability-tier deltas per ADR-0316 must cover deployment-context dimension (e.g., Bronze = public-cloud-only; Silver = + AWS/OCI guest; Gold = + on-prem; Platinum = + colo + sovereign + Oyatie-as-cloud-provider)
- Audit-wave coherence checks add a 6th dimension: **deployment-context support** — every µservice's docs must enumerate AWS-guest / OCI-guest / on-prem / colo / Oyatie-cloud-provider support explicitly

This constraint cross-cuts the 5-phase canonical sequence — it's not its own phase but a property of every artifact in every phase.

**OCI deployment profile — Always Free maximization sub-rule:** Per `feedback_oci_always_free_maximization_2026_05_20.md`, the OCI deployment profile specifically MUST maximize OCI Always Free tier resources (2× Ampere A1 ARM 4-OCPU+24GB / 2× Autonomous DB 20GB each / 200 GB block volume / 10 GB Object Storage / 10 TB monthly egress / Vault 3 vaults+20 keys / Always Free LB 10Mbps / Streaming 1 partition+1GB / Functions 2M invocations + 400K GB-sec / API Gateway 100M calls / Email Delivery 100/month / Logging 10GB/month / VCN+subnets+gateways free / IAM+Audit fully free). Bronze tier on OCI deployment context = Always Free; Silver+ adds paid OCI resources. Demo / sandbox / trial / dev tenants default to Always Free profile. Per-µservice `iac/oci-guest/always-free/` OpenTofu module composes ONLY Always Free resources; the regular `iac/oci-guest/` module composes paid resources for Silver+ tiers. This is OCI-only optimization — AWS / on-prem / colo / Oyatie-cloud-provider deployments have no equivalent free-tier maximization (AWS Free Tier is 12-month-limited; on-prem has no free tier; etc.). Provider-agnostic at the architectural layer; provider-specific at the cost-optimization layer.

## Zero-Handroll OpenTofu-Only Setup Constraint (cross-cutting, all phases)

Per `feedback_zero_handroll_opentofu_only_2026_05_20.md` (and ADR-0211 in-house tech stack + ADR-0216 open integration + ADR-0039 supply-chain hardening):

**Every µservice deployment for every deployment context MUST land via OpenTofu (not Terraform); zero hand-rolled steps.**

- OpenTofu binary (`tofu` CLI) — NOT Terraform; OpenTofu is the open-source community fork without HashiCorp BSL licensing
- Every µservice has an `iac/` directory with OpenTofu modules per supported deployment context:
  - `microservices/<name>/iac/aws-guest/` — AWS provider
  - `microservices/<name>/iac/oci-guest/` — OCI provider
  - `microservices/<name>/iac/on-prem/` — bare-metal providers (libvirt / metal3 / k0sproject / Cluster API)
  - `microservices/<name>/iac/colo/` — Equinix Metal / Cyxtera / etc. providers
  - `microservices/<name>/iac/oyatie-cloud-provider/` — Oyatie's own OpenTofu provider plugin
- `microservices/cloud-iac/` is the IaC orchestrator µservice — owns OpenTofu module library + per-tenant composition + state-backend abstraction (portable across contexts)
- No `null_resource` / `local-exec` / SSH provisioners — all setup logic declarative in OpenTofu resources/providers
- All modules signed via sigstore + supply-chain integrity per ADR-0039
- Tenant onboarding GTM motion leads with `tofu apply -var tenant_id=<name> -var deployment_context=<aws-guest|oci-guest|on-prem|colo|oyatie-cloud-provider>` — zero hand-roll between contract + tenant-active

**Audit-wave dimension count: 5 → 6 → 7 → 8** (added "OS support matrix coverage" as 8th dimension):

7. **OpenTofu IaC coverage** — every µservice has `iac/<context>/` OpenTofu modules for every deployment context it claims to support; missing context = audit P1 finding

8. **OS support matrix coverage** — every µservice's manifest declares `supported_oses` enumerating Talos / RHEL / Oracle Linux / SUSE / Ubuntu LTS / Debian / Rocky / AlmaLinux / CentOS Stream / Amazon Linux / Flatcar / Photon / macOS-Apple-Silicon-M5+ support; per-OS CI lane proves the claim; missing OS = audit P1 finding; Intel macOS or pre-M5 Apple Silicon references = audit P0 finding (explicit out-of-scope per `feedback_os_support_matrix_2026_05_20.md`)

9. **Rust-strict language coverage** — every µservice's path scanned for forbidden language files. Backend/µservice/scripting must be Rust-only. **Frontend bundles permitted in Swift (iOS/macOS) / Kotlin (Android) / C#-WinUI-3 (Windows)** scoped to `frontend/<platform>/` directories only. Other Python/JS-app/Ruby/Perl/PHP/Java/Scala/Groovy/Go/F# in any µservice path = audit P0 finding without per-µservice ADR justification + sunset plan. Per `feedback_rust_strict_only_no_python_2026_05_20.md`.

## Rust-Strict Language Constraint (cross-cutting, all phases)

Per `feedback_rust_strict_only_no_python_2026_05_20.md` (and ADR-0211 in-house tech stack + ADR-0145 inter-µservice direct gRPC + ADR-0324 anti-script doctrine):

**Oyatie codebase is STRICTLY Rust-only for backend/µservice/scripting unless an explicit per-µservice ADR justifies an exception with a sunset plan. Frontend native bundles are scoped exceptions.**

- All µservice runtime code → Rust
- All CLI tooling → Rust (no `make`, no shell scripts, no Python scripts)
- All validation tools → Rust crates (no `validate.py` / no `check.sh`)
- All codegen tools → Rust crates with `build.rs` patterns
- All CI logic → Rust (run via `cargo`)
- All SDK clients → Rust as primary; other-language SDKs are GENERATED from contracts (OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 codegen), never hand-authored
- **Frontend native apps → permitted in platform-native languages:**
  - **iOS / macOS frontend** → Swift (lives under `frontend/ios/` and `frontend/macos/`; macOS-Apple-Silicon-M5+ only)
  - **Android frontend** → Kotlin (lives under `frontend/android/`)
  - **Windows frontend** → WinUI 3 / C# / .NET 8+ (lives under `frontend/windows/`; .NET dependency scoped to this bundle only, backend never depends on .NET)

**Authorized non-Rust file extensions (strict whitelist):**
- `*.tf` (OpenTofu HCL; IaC only; lives under `iac/<context>/`)
- `*.cedar` (Cedar v4.2 LTS; policies)
- `*.yaml,*.yml,*.json` (config + contracts only; never logic)
- `*.proto` (proto3)
- `*.openslo.yaml` (OpenSLO 1.0)
- `*.sql` (migrations only)
- `*.md` (docs)
- `*.swift` (frontend iOS/macOS only; under `frontend/ios/` or `frontend/macos/`)
- `*.kt` / `*.kts` (frontend Android only; under `frontend/android/`)
- `*.cs` / `*.xaml` (frontend Windows WinUI 3 only; under `frontend/windows/`)

**Forbidden language extensions (audit P0 if found without justification):**
- `*.py` / `*.pyc` / `*.pyi` — Python
- `*.js` / `*.ts` (except generated SDK directories with provenance manifest) — JavaScript/TypeScript app logic
- `*.rb` / `*.pl` / `*.php` — Ruby / Perl / PHP
- `*.java` / `*.scala` / `*.groovy` — Java / Scala / Groovy (Kotlin permitted ONLY in frontend/android/)
- `*.fs` / `*.vb` — F# / VB.NET (C# permitted ONLY in frontend/windows/)
- `*.go` — Go
- Multi-line bash beyond 3 lines (short pipe commands OK)

Build/CI invocation for backend: `cargo build --workspace --release --all-features --locked` (single canonical entry point). Frontend bundles have their own platform-native build pipelines (Xcode for Swift/macOS-iOS, Gradle for Kotlin-Android, MSBuild for WinUI-3-Windows) — these are scoped to their frontend directories and don't pollute the backend `cargo` build.

## OS Support Matrix Constraint (cross-cutting, all phases)

Per `feedback_os_support_matrix_2026_05_20.md`:

**Required OSes for every µservice:**

| Tier | OS | Version floor | Arch | Notes |
|------|----|----|------|-------|
| 1 | Talos Linux | 1.8+ | linux/amd64, linux/arm64 | Container-image only; K8s-native; no shell |
| 1 | RHEL | 9.4+ / 10+ | linux/amd64, linux/arm64 | RPM + container |
| 1 | Oracle Linux | 9.4+ / 10+ (UEK-R7+) | linux/amd64, linux/arm64 | OCI-guest default; RPM + container |
| 1 | SLES | 15 SP6+ / 16 | linux/amd64, linux/arm64 | RPM + container |
| 1 | Ubuntu LTS | 24.04+ / 26.04+ | linux/amd64, linux/arm64 | DEB + container |
| 1 | Debian stable | 12+ (Bookworm) / 13+ (Trixie) | linux/amd64, linux/arm64 | DEB + container |
| 1 | Rocky Linux | 9.4+ / 10+ | linux/amd64, linux/arm64 | RPM + container |
| 1 | AlmaLinux | 9.4+ / 10+ | linux/amd64, linux/arm64 | RPM + container |
| 1 | CentOS Stream | 9 / 10 | linux/amd64, linux/arm64 | RPM + container |
| 1 | Amazon Linux | 2023+ | linux/amd64, linux/arm64 | AWS-guest default; RPM + container |
| 1 | Flatcar | stable | linux/amd64, linux/arm64 | Container-image only; immutable |
| 1 | VMware Photon | 5+ | linux/amd64, linux/arm64 | VMware-managed hosts |
| 1 | **macOS Apple Silicon M5+** | Sequoia 15.0+ / macOS 16+ | **darwin/arm64 ONLY** | **NO Intel macOS; NO pre-M5 Apple Silicon (M1/M2/M3/M4)**; `.pkg` + Homebrew |
| 2 | linux/ppc64le | RHEL-on-Power, Ubuntu-on-Power | ppc64le | Test-only; not GA-blocking |
| 2 | linux/s390x | RHEL-on-Z, Ubuntu-on-Z | s390x | Test-only; not GA-blocking |

Every µservice's manifest must declare `supported_oses`; per-OS CI lane proves the claim. Per-context × per-OS OpenTofu modules live at `microservices/<name>/iac/<context>/<os>/` where OSes require distinct provisioning (e.g., Talos uses talosctl vs RHEL uses cloud-init).

## Canonical Build Sequence (Phase 4 interview confirmed)

The realignment spec encodes this 5-phase architectural sequence into ADR-0328 + master-plan-sequencing.json. Subsequent phases cannot proceed beyond their substance-bar checkpoint until prior phase is at substance bar (per ADR-0327 promotion gates). Capability-tier coverage runs through each phase per ADR-0316.

**Phase 0 — Shared Infrastructure (cloud-* family):**
cloud-iam · cloud-kms · cloud-secrets · cloud-iac · cloud-network · cloud-network-dns · cloud-data · cloud-storage · cloud-compute-functions · cloud-compute-k8s · cloud-compute-vm · cloud-billing · cloud-billing-tax · cloud-capacity · cloud-cell · cloud-dcops · cloud-finops · cloud-marketplace · cloud-fsh

**Phase 1 — Foundations / Platform Substrate:**
identity · tenancy · audit-chain · governance · compliance · observability · payments · finops-portal · api-gateway · application · developer-sdk · network · cell

*(foundry as standalone µservice has been ABSORBED by Phase 2 µservices per ADR-0255-amendment + ADR-0247 self-modification doctrine; the agentic capability now lives across intelligence + workflow-engine + workflow-studio + ontology + governance/tenancy. The `oyatie.foundry.*` Cedar principal namespace remains canonical but is provisioned by tenancy + governance, not by a separate runtime. The `microservices/foundry/` path is queued for retirement via ADR-0138 six-path-deprecation pattern in Wave 15+. The legacy "Hermes" reference in `tools/hooks/_canonical-primitives.md` is dropped — Hermes was a sample feature exploration target only, and foundry-the-capability goes above-and-beyond Hermes.)*

**Phase 2 — Core Capability Substrate (jointly absorbs foundry capability):**
intelligence (consumer AI substrate per ADR-0220 + ADR-0255 two-layer + library-first LLM binding) · ontology (Palantir-equivalent entity graph + agent state + cross-µservice projection) · workflow-engine (durable function execution + agentic-pipeline runtime) · workflow-studio (n8n-class visual editor + AI-assisted agentic node generation) · consent-graph (per-tenant consent capture, foundational for GDPR/PIPA/EU AI Act) · detection (abuse defence + AI Act compliance per ADR-0297..0310 cluster)

*(These 6 µservices jointly provide the "foundry" capability — there is no separate foundry runtime; agentic CI/dev/automation workflows are workflow-engine workflows authored in workflow-studio, calling intelligence's library-first LLM binding, with ontology projecting agent state, all under `oyatie.foundry.*` Cedar principals defined in governance + tenancy.)*

**Phase 3 — Communication & Collaboration:**
messenger (MLS RFC 9420 e2ee per ADR-0254) · mail (per-tenant DKIM custody) · drive (per-tenant CMK per-file DEK envelope) · calendar (RFC 5545 + FREEBUSY ACL) · meet (SFU + WebRTC) · recordings · notes (Yjs CRDT block-based) · docs (collaborative editing) · sheets (formula engine) · slides (SVG-first) · forms (logic-jump) · connect (cross-tenant federation per ADR-0311 dual-tenant boundary) · comms-email · community (TeamBlind+Reddit+LinkedIn+Handshake mixture per anonymous-fold) · shorts · analytics · tasks · translate · search

**Phase 4 — Distribution + B2B Enterprise SaaS (formerly Phase 4 + Phase 5, now merged):**
First the distribution substrate: marketplace (universal deal settlement per ADR-0314) · plugin-app-store · workplace-integration (clock-in/e-sign/payroll bridge) · feature-flags

Then the Big 8 enterprise SaaS displacement layer in priority sub-sequence:

- **4A.1 HR / Payroll (Workday family)** — ships FIRST: workforce + performance-management + learning-management + payroll surfaces. Rationale: employee/payroll identity is foundational to every other B2B journey; Workday displacement is largest single-vendor TAM and identity-grounded.
- **4A.2 ERP (SAP family)** — ships SECOND: production-planning + quality-management + plant-maintenance + warehouse + real-estate + treasury + supply-chain-planning + global-trade + financial-planning. Rationale: ERP is the deepest moat; replaces the SAP S/4HANA + SAP modules surface.
- **4A.3 CRM (Salesforce family)** — ships THIRD: crm + marketing-automation + contact-center + community-sales surfaces. Rationale: Sales/marketing close after HR + ERP land because they depend on enterprise data foundations.
- **4A.4..4A.8** — ServiceNow → HubSpot → Microsoft → Oracle → Adobe → Atlassian (default order; can be refined per quarterly tactical priority)
- **4B Long-tail B2B SaaS** — contract-lifecycle-management · incident-management · data-warehouse · design-collaboration · whiteboard · data-pipeline · healthcare-integration · ops-dashboard-control-center · brand · sites · plus cloud-infra/PaaS/developer-tool dossiers (per Phase 4 interview correction — all in-scope, lowest priority).

## Audit Wave Specification (resolved in Phase 4 interview)

**Audit scope per µservice = AUDIT ONLY (defer remediation to follow-up wave).** Each ownership-audit agent produces FOUR docs inside the µservice's path:

1. `microservices/<name>/coherence-audit-2026-05-20.md` — **8-dimension** audit:
   - Internal coherence (within µservice path)
   - Outbound cross-references (root ADRs / other µservices / personas / journeys)
   - Substance bar (intern-buildability per documentation-rigor §1.1)
   - Canonical-direction alignment (Wave-3-G unified-ecosystem thesis)
   - Industry-counterpart parity (overall verdict)
   - **Multi-context deployment support** (enumerate AWS-guest / OCI-guest / on-prem / colo / Oyatie-cloud-provider support; absence in PRD = audit P1 finding)
   - **OpenTofu IaC coverage** (every claimed deployment context must have `microservices/<name>/iac/<context>/` OpenTofu modules; missing = audit P1 finding; any `terraform` reference or hand-roll script = audit P0 finding)
   - **OS support matrix coverage** (every µservice manifest declares `supported_oses` covering Talos + 11 Linux distros + macOS-Apple-Silicon-M5+; per-OS CI lane proves the claim; Intel macOS or pre-M5 Apple Silicon = P0 finding)
2. `microservices/<name>/feature-parity-matrix-2026-05-20.md` — per-µservice top-3 counterparts identified + UNION-coverage feature matrix (✓ covered / ⚠ partial / ✗ missing per major feature)
3. `microservices/<name>/performance-benchmark-numbers-2026-05-20.md` — Oyatie µservice vs top-3 counterpart benchmark numbers (latency p50/p95/p99, throughput, cost per Bronze/Silver/Gold/Platinum tier)
4. `microservices/<name>/capability-tier-deltas-vs-counterparts-2026-05-20.md` — per-tier feature deltas vs counterpart-tier-equivalents (e.g., Oyatie messenger Bronze vs Slack Business; Oyatie messenger Platinum vs Slack Enterprise Grid)

**Audit batch grouping = BY PHASE** (per 5-phase canonical sequence):
- Audit wave 1 = Phase 0 cloud-* µservices (~18 µservices in 3 batches of 8)
- Audit wave 2 = Phase 1 foundations µservices (~14 µservices in 2 batches of 8)
- Audit wave 3 = Phase 2 core capability µservices (~6 µservices in 1 batch)
- Audit wave 4 = Phase 3 communication & collaboration µservices (~20 µservices in 3 batches)
- Audit wave 5 = Phase 4 distribution + B2B enterprise SaaS µservices (~21 µservices in 3 batches)
- Total: ~79 µservices × 4 docs = ~316 audit docs across 12 batches of 8 codex agents each

**Parity bar per µservice = TOP-3 COUNTERPARTS at UNION-COVERAGE.** Each µservice identifies its top-3 industry counterparts (from ADR-0321 dossiers + capability-tier registry + benchmark catalog). Oyatie must cover the UNION of major features across the 3 — any feature offered by any of the 3 is in-scope for Oyatie's PRD/IPs/contracts. Niche/specialty features that don't fit unified-ecosystem thesis can be marked as "out-of-scope intentional".

**Verification SLA's 5 canonical anchors = AGENT-CLASS-SPECIFIC.** Each agent class gets its own anchor template encoded in ADR-0328 + brief template. Examples:

- **µservice-ownership-audit agent's 5 anchors:** (1) unified-ecosystem thesis (2) µservice's own PRD (3) µservice's existing artifacts coherence summary (4) µservice's feature-parity-matrix top-3 counterparts (5) documentation-rigor §1.1
- **ADR-0321 dossier authoring agent's 5 anchors:** (1) ADR-0321 §A scope definition (2) Wave-3-G unified-ecosystem thesis (3) ADR-0316 capability-tier doctrine (4) µservice's surface-coverage for that vendor (5) feature-parity-matrix for that specific vendor
- **IP slice authoring agent's 5 anchors:** (1) µservice's own PRD (2) ADR-0263 audit emission contract (3) ADR-0244 tenant scoping (4) µservice's journey-coverage from cross-coverage-matrix (5) substance bar
- **Per-µservice ADR authoring agent's 5 anchors:** (1) µservice's own PRD (2) Wave-3-G doctrine cluster (3) ADR-0105 layer enum (4) µservice's IP set (5) substance bar

The agent-class-specific anchor templates live in ADR-0328 §D-3 + brief-template.md.

## Open Questions

None remaining — Phase 4 interview complete; ambiguity ≤ 20% threshold.

## Phase 5 Execution Bridge

Per the SDD gated workflow + planning-and-task-breakdown skill, the next phase is **TASKS** — decompose this spec into ordered atomic tasks with acceptance criteria + verification + dependency graph + vertical slices + checkpoints. The task breakdown will be authored as `.omc/plans/realign-oyatie-corpus-plan-2026-05-20.md`.

Execution options at Phase 5 (orchestrator selects after this spec is approved):

1. **Tasks breakdown next (Recommended)** — author the per-task plan with dependency graph; user approves before any dispatch
2. **Skip task breakdown, dispatch Wave 1 directly** — author ADR-0328 + master-plan-sequencing.json + brief template via codex agents NOW (3 codex in parallel)
3. **Refine further** — return to Phase 4 interview if any of the 3 open questions need answers before proceeding
