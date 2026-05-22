---
doc_class: ImplementationPlan
slug: realign-oyatie-corpus
date: 2026-05-20
status: ready-for-approval
source: deep-dive (spec crystallized)
spec_path: /Users/jasonlee/oyatie/.omc/specs/deep-dive-realign-oyatie-corpus-to-canonical.md
trace_path: /Users/jasonlee/oyatie/.omc/specs/deep-dive-trace-realign-oyatie-corpus-to-canonical.md
---

# Implementation Plan: Realign Oyatie Corpus to Canonical Direction

## Overview

This plan decomposes the Oyatie corpus realignment into atomic, dependency-graphed tasks across 4 sequential waves. **No agent dispatch occurs until the user approves this plan.**

The realignment proceeds in 4 phases:
- **Wave 1** — Canonical-direction backbone (3 tasks; serial; ~1 batch of 3 codex agents)
- **Wave 2-13** — Per-µservice ownership-coherence audit (12 batches; sequenced by 5-phase canonical sequence; ~12 batches of 8 codex agents = ~96 agent-tasks)
- **Wave 14** — Audit findings aggregation + remediation prioritization (1 task; serial; orchestrator-authored)
- **Wave 15+** — Remediation + content completion per Big 8 priority (variable; sequenced by 5-phase canonical sequence)

## Architecture Decisions (from Phase 4 interview)

- 5-phase canonical build sequence: Phase 0 cloud-infra → Phase 1 foundations → Phase 2 core capability → Phase 3 comms+collab → Phase 4 distribution + B2B SaaS
- Big 8 sub-sequence within Phase 4: HR/Payroll → ERP → CRM → ServiceNow → HubSpot → Microsoft → Oracle → Adobe → Atlassian
- Audit-only first (defer remediation); 4 audit docs per µservice
- Top-3 counterparts at UNION-coverage feature parity bar
- Agent-class-specific 5-anchor verification SLA
- Codex-only dispatch (per active directive); 8-codex ceiling per batch

## Dependency Graph

```
Wave 1: Canonical-direction backbone
   │
   ├── T1.1 ADR-0328 (substance-bar-as-canonical-sequence-and-batch-discipline) ─┐
   ├── T1.2 master-plan-sequencing.json update with 5-phase sequence + Big 8 ──┤── BLOCKING gate before Wave 2+
   └── T1.3 brief-template.md with agent-class-specific anchor sets ────────────┘
       │
       ▼
Wave 2: Phase 0 cloud-infra audit (3 batches of 8 = 24 µservices)
   │
   ▼ (each batch parallel-dispatched, waves serial)
Wave 3: Phase 1 foundations audit (2 batches of 8 = 14 µservices + 2 spare slots for Phase 0 carry-over)
   │
   ▼
Wave 4: Phase 2 core capability audit (1 batch of 6 µservices)
   │
   ▼
Wave 5-7: Phase 3 comms+collab audit (3 batches of 8 = 20 µservices + 4 spare slots)
   │
   ▼
Wave 8-13: Phase 4 distribution+B2B-SaaS audit (3-6 batches = 21 µservices)
   │
   ▼
Wave 14: Aggregate findings + prioritize remediation
   │
   ▼
Wave 15+: Remediation + content completion (variable)
```

## Wave 1: Canonical-Direction Backbone (BLOCKING gate)

This wave authors the canonical anchors that ALL subsequent agents must cite. No other waves can dispatch until this completes.

### Task 1.1: Author ADR-0328 substance-bar-as-canonical-sequence-and-batch-discipline

**Description:** Author the realignment doctrine ADR covering the 5-phase build sequence, Big 8 sub-sequence, per-µservice ownership-coherence audit protocol, parity bar definition, verification SLA, agent-class-specific anchor sets, and the brief-template authoring convention.

**Acceptance criteria:**
- [ ] File exists: `/Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
- [ ] ≥800 lines BESPOKE per documentation-rigor §1.1 + ADR-0322 substance bar
- [ ] §D-1..D-14 detailed-mechanics sub-sections
- [ ] Cites ≥15 root ADRs (cluster 0297-0327 + keystone 0242-0258 + foundational 0105/0131/0244/0263/0316)
- [ ] Defines 5-phase canonical sequence by name with explicit µservice rosters per phase
- [ ] Defines Big 8 vendor families sub-sequence (HR/Payroll → ERP → CRM → ServiceNow → HubSpot → Microsoft → Oracle → Adobe → Atlassian)
- [ ] Defines per-µservice ownership-coherence audit 5-dimension protocol
- [ ] Defines top-3 counterparts UNION-coverage parity bar
- [ ] Defines agent-class-specific 5-anchor sets (≥4 examples: µservice-ownership-audit / ADR-0321-dossier-author / IP-author / per-µservice-ADR-author)
- [ ] Status: Proposed

**Verification:**
- [ ] `wc -l docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` ≥ 800
- [ ] `grep -c "^###" docs/decisions/ADR-0328-*.md` ≥ 14 (D-1..D-14 sub-sections)
- [ ] `grep -c "ADR-0" docs/decisions/ADR-0328-*.md` ≥ 15 (root ADR citations)
- [ ] `./bin/oya vcs claim --agent claude-adr-0328-author --intent author-adr-0328 docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` → verify → done → promote accepted
- [ ] Manual orchestrator-read: substance bar met; 5-phase sequence + Big 8 sub-sequence are unambiguous

**Dependencies:** None (Wave 1 task #1)
**Files likely touched:**
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`

**Estimated scope:** Medium (1 file, ~800-1200 lines authored)
**Agent:** 1 codex (gpt-5.5 xhigh) — single-agent ownership for coherent voice across §D-1..D-14

### Task 1.2: Update master-plan-sequencing.json with 5-phase sequence + Big 8

**Description:** Update the canonical machine-readable master plan spec to encode the 5-phase canonical sequence + Big 8 sub-sequence + audit-wave dependency graph + remediation-wave dependency graph.

**Acceptance criteria:**
- [ ] File updated: `/Users/jasonlee/oyatie/specs/master-plan-sequencing.json`
- [ ] Adds `canonical_build_sequence` top-level key with 5 phases enumerated
- [ ] Each phase has `phase_id` (0-4), `phase_name`, `microservices` (array of µservice names from /microservices/), `gate_criteria` (substance bar checkpoint), `dependencies` (prior phase IDs that must reach substance bar)
- [ ] Phase 4 has nested `big_8_sub_sequence` array in HR/Payroll → ERP → CRM order
- [ ] Adds `realignment_wave_sequence` top-level key with Wave 1 → Wave 2-13 → Wave 14 → Wave 15+ DAG
- [ ] Adds `forbidden_primitives` entries to reject (a) authoring without Wave 1 ADR-0328 citation; (b) cross-batch parallelism; (c) line-count-only verification
- [ ] Valid JSON (parses via `python3 -c 'import json; json.load(open("specs/master-plan-sequencing.json"))'`)
- [ ] Cites ADR-0328

**Verification:**
- [ ] JSON parses
- [ ] `jq '.canonical_build_sequence | length' specs/master-plan-sequencing.json` == 5
- [ ] `jq '.canonical_build_sequence[4].big_8_sub_sequence[0]' specs/master-plan-sequencing.json` == "workday-family-hr-payroll" (or equivalent ID)
- [ ] `./bin/oya vcs` lifecycle accepted

**Dependencies:** Task 1.1 (ADR-0328 must exist to cite)
**Files likely touched:**
- `specs/master-plan-sequencing.json`

**Estimated scope:** Small (1 file, ~200-400 line JSON delta)
**Agent:** 1 codex (single-agent JSON consistency)

### Task 1.3: Author brief-template.md with agent-class-specific anchor sets

**Description:** Author the canonical brief template that ALL future authoring/audit/remediation agent dispatches MUST use. Template includes 5-citation header block + agent-class-specific anchor sets + decision tree for in-scope/out-of-scope decisions.

**Acceptance criteria:**
- [ ] File exists: `/Users/jasonlee/oyatie/docs/standards/brief-template.md` (≥500 lines)
- [ ] Lists 4-6 agent classes with their 5-anchor template each (µservice-ownership-audit / ADR-0321-dossier-author / IP-author / per-µservice-ADR-author / journey-author / runbook-author)
- [ ] Each anchor set names the exact canonical files/sections to cite
- [ ] Decision tree for: "is vendor X in-scope?" / "is feature Y substance-bar?" / "is contradiction Z hard-or-soft?"
- [ ] Code-block examples of well-formed brief headers per agent class
- [ ] Cites ADR-0328 + documentation-rigor §1.1 + feedback memory directives

**Verification:**
- [ ] File parses
- [ ] `grep -c "^### Anchor Set —" docs/standards/brief-template.md` ≥ 4 (one per agent class)
- [ ] Manual orchestrator-read: template is intern-buildable; an agent reading the template knows exactly which 5 anchors to cite

**Dependencies:** Task 1.1, Task 1.2
**Files likely touched:**
- `docs/standards/brief-template.md`

**Estimated scope:** Small-Medium (1 file, ~500-800 lines)
**Agent:** 1 codex

### Task 1.4 (AMENDMENT — added mid-Wave-1): Multi-Context + OpenTofu + OS-Matrix + Rust-Strict + OCI-Always-Free constraints amendments

**Description:** The 3 Wave 1 agents (Tasks 1.1-1.3) were dispatched BEFORE the user added FIVE cross-cutting constraints:
- multi-context-platform / provider-agnostic / Oyatie-as-cloud-provider (per `feedback_multi_context_provider_agnostic_2026_05_20.md`)
- zero-handroll OpenTofu-only setup (per `feedback_zero_handroll_opentofu_only_2026_05_20.md`)
- OS support matrix: Talos + 11 Linux enterprise distros + macOS-Apple-Silicon-M5+ ONLY (per `feedback_os_support_matrix_2026_05_20.md`)
- Rust-strict-only for backend/µservice/scripting — no Python / no JS-app-logic / no Ruby/Perl/PHP/Java/Scala/Groovy/Go/F#; frontend bundles permitted in Swift (iOS/macOS) + Kotlin (Android) + WinUI 3 C#/.NET (Windows) scoped to `frontend/<platform>/`; backend exceptions require per-µservice ADR + sunset plan (per `feedback_rust_strict_only_no_python_2026_05_20.md`)
- OCI Always Free maximization for OCI deployment profile (per `feedback_oci_always_free_maximization_2026_05_20.md`) — Bronze tier on OCI = Always Free; per-µservice `iac/oci-guest/always-free/` OpenTofu module; provider-agnostic everywhere else

After Tasks 1.1-1.3 land + are verified, dispatch ONE codex agent to amend all three Wave 1 deliverables with all five constraints woven through.

**Acceptance criteria:**

ADR-0328 amendments:
- [ ] §D-15 "Multi-Context Platform Constraint" added — every phase + every µservice supports the 6 deployment contexts (oyatie-public-cloud / guest-on-aws / guest-on-oci / on-prem / colo / oyatie-as-cloud-provider)
- [ ] §D-16 "Zero-Handroll OpenTofu-Only Setup Constraint" added — every µservice has `iac/<context>/` OpenTofu modules; no `terraform` references; no `null_resource` / `local-exec` / SSH provisioners; sigstore-signed modules per ADR-0039
- [ ] §D-17 "cloud-iac µservice as IaC Orchestrator" — owns OpenTofu module library + per-tenant composition + portable state-backend
- [ ] §D-18 "OS Support Matrix Constraint" added — Tier-1 OSes: Talos + RHEL + Oracle Linux + SUSE + Ubuntu LTS + Debian + Rocky + AlmaLinux + CentOS Stream + Amazon Linux + Flatcar + Photon + macOS-Apple-Silicon-M5+ (ONLY); Tier-2: ppc64le / s390x. Intel macOS + pre-M5 Apple Silicon explicitly OUT-OF-SCOPE. Per-µservice manifest declares `supported_oses`; per-OS CI lane.
- [ ] §D-19 "Rust-Strict Language Constraint" added — strict Rust-only backend + frontend-scoped Swift/Kotlin/WinUI-3 exceptions; per ADR-0211 + ADR-0145; whitelist of authorized non-Rust extensions (backend: `*.tf` / `*.cedar` / `*.yaml,*.json` / `*.proto` / `*.openslo.yaml` / `*.sql` / `*.md`; frontend: `*.swift` under `frontend/ios/`+`frontend/macos/` / `*.kt,*.kts` under `frontend/android/` / `*.cs,*.xaml` under `frontend/windows/`); forbidden: Python/JS-app-logic/Ruby/Perl/PHP/Java/Scala/Groovy/Go/F#; exceptions require per-µservice ADR-MS-NNN-non-rust-justification with sunset plan
- [ ] §D-20 "OCI Always Free Maximization Sub-Profile" added — provider-agnostic overall; OCI-guest deployment context has special Always-Free sub-profile (Bronze tier on OCI = Always Free); `microservices/<name>/iac/oci-guest/always-free/` OpenTofu module composes ONLY Always Free resources (2× Ampere A1 ARM 4-OCPU+24GB / 2× Autonomous DB 20GB / 200GB block / 10GB obj / 10TB egress / Vault / LB / Streaming / Functions / API Gateway / Email Delivery / Logging); demo / sandbox / trial / dev tenants default to this profile; Silver+ adds paid OCI resources
- [ ] Audit wave dimension count: 5 → **9** (multi-context + OpenTofu + OS-matrix + Rust-strict; OCI Always Free is sub-profile of multi-context, not its own dimension)

master-plan-sequencing.json amendments:
- [ ] Top-level `deployment_contexts` array: `["oyatie-public-cloud", "guest-on-aws", "guest-on-oci", "on-prem", "colo", "oyatie-as-cloud-provider"]`
- [ ] Per-µservice `deployment_context_support` array enumerating which contexts each µservice supports
- [ ] Per-µservice `iac_modules` map: `{"aws-guest": "microservices/<name>/iac/aws-guest/", "oci-guest": ..., "on-prem": ..., "colo": ..., "oyatie-cloud-provider": ...}`
- [ ] Top-level `iac_substrate` key: `{"engine": "opentofu", "version_floor": "1.6.0", "state_backend_per_context": {...}, "module_signing": "sigstore+adr-0039"}`
- [ ] Top-level `supported_oses` array enumerating 13 Tier-1 OSes (Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, macOS-Apple-Silicon-M5+) + 2 Tier-2 (ppc64le, s390x) with version_floor + arch list per entry
- [ ] Top-level `out_of_scope_oses` array: `["macos-intel", "macos-apple-silicon-m1-m2-m3-m4-pre-m5", "freebsd", "windows-server"]` (explicit exclusions)
- [ ] Per-µservice `supported_oses` array referencing IDs from top-level (default: all Tier-1)
- [ ] Top-level `language_policy` object: `{"strict_rust_backend": true, "authorized_non_rust_extensions_global": ["*.tf","*.cedar","*.yaml","*.yml","*.json","*.proto","*.openslo.yaml","*.sql","*.md"], "frontend_native_exceptions": {"frontend/ios/": ["*.swift"], "frontend/macos/": ["*.swift"], "frontend/android/": ["*.kt","*.kts"], "frontend/windows/": ["*.cs","*.xaml"]}, "frontend_constraints": {"ios_macos": "Swift only; macOS scoped to Apple-Silicon-M5+", "android": "Kotlin only; Kotlin Multiplatform requires per-µservice ADR", "windows": "WinUI 3 / C# / .NET 8+ only; backend cannot depend on .NET"}, "forbidden_extensions": ["*.py","*.pyc","*.pyi","*.js","*.ts","*.rb","*.pl","*.php","*.java","*.scala","*.groovy","*.fs","*.vb","*.go"], "frontend_scoped_extensions_outside_frontend_dir": "audit P0", "exception_protocol": "per-µservice ADR-MS-NNN-non-rust-justification with sunset plan per ADR-0108", "canonical_backend_build": "cargo build --workspace --release --all-features --locked", "frontend_build_pipelines": {"ios_macos": "xcodebuild", "android": "gradle", "windows": "msbuild"}}`
- [ ] Forbidden primitives list adds: "Terraform CLI references (use OpenTofu)" + "null_resource / local-exec / SSH provisioners for setup logic" + "manual setup steps in deployment docs" + "Intel macOS support claims" + "pre-M5 Apple Silicon support claims" + "Python scripts of any kind without ADR-MS-NNN justification" + "JavaScript/TypeScript application logic" + "make / npm / python setup.py / poetry / pipenv backend build invocations" + "Swift/Kotlin/C# files OUTSIDE their respective frontend/<platform>/ directories" + "backend µservice referencing .NET / JVM / Apple frameworks at runtime"

brief-template.md amendments:
- [ ] §3.9 agent class "deployment-context-coverage-auditor" with 5-anchor template
- [ ] §3.10 agent class "opentofu-iac-coverage-auditor" with 5-anchor template
- [ ] §3.11 agent class "os-support-matrix-auditor" with 5-anchor template
- [ ] §3.12 agent class "rust-strict-language-auditor" with 5-anchor template
- [ ] Universal substance-bar checklist adds: (a) "every µservice doc enumerates deployment-context support explicitly" (b) "every µservice has OpenTofu modules per supported context — verify via ls microservices/<name>/iac/<context>/*.tf" (c) "every µservice manifest declares `supported_oses` enumerating Tier-1 OSes; per-OS CI lane proves the claim" (d) "every µservice path scanned for forbidden extensions (Python/JS-app/Ruby/Perl/PHP/Java-family/Go/.NET); any found without per-µservice ADR justification = P0"
- [ ] Forbidden patterns section adds: "no `terraform` binary references" + "no hand-roll setup steps" + "no AWS-specific or OCI-specific assumptions hardcoded outside iac/<context>/" + "no Intel macOS support claims" + "no pre-M5 Apple Silicon support claims (M1/M2/M3/M4 out of scope)" + "NO Python scripts of any kind without ADR-MS-NNN justification" + "NO JavaScript/TypeScript application logic" + "NO Ruby / Perl / PHP / Java-family / Go / .NET / multi-line bash beyond 3 lines"

**Verification:**
- [ ] `grep -E "(AWS|OCI|on-prem|colo|cloud-provider)" docs/decisions/ADR-0328-*.md | wc -l` ≥ 10
- [ ] `grep -c "OpenTofu" docs/decisions/ADR-0328-*.md` ≥ 8
- [ ] `grep -c "Talos" docs/decisions/ADR-0328-*.md` ≥ 3
- [ ] `grep -c "Apple Silicon M5" docs/decisions/ADR-0328-*.md` ≥ 2
- [ ] `jq '.deployment_contexts | length' specs/master-plan-sequencing.json` == 6
- [ ] `jq '.iac_substrate.engine' specs/master-plan-sequencing.json` == "opentofu"
- [ ] `jq '.supported_oses | length' specs/master-plan-sequencing.json` ≥ 13
- [ ] `jq '.out_of_scope_oses | contains(["macos-intel"])' specs/master-plan-sequencing.json` == true
- [ ] `grep -c "OpenTofu" docs/standards/brief-template.md` ≥ 5
- [ ] `grep -c "deployment-context" docs/standards/brief-template.md` ≥ 3
- [ ] `grep -c "supported_oses" docs/standards/brief-template.md` ≥ 3

**Dependencies:** Tasks 1.1 + 1.2 + 1.3 land + verified
**Files likely touched:** Same 3 files as Tasks 1.1-1.3 (amendment-style edits)
**Estimated scope:** Medium (~400-600 line deltas across 3 files)
**Agent:** 1 codex (single-agent for coherence of amendment voice)

### Checkpoint: After Wave 1 (Tasks 1.1-1.4)

- [ ] All 3 base files exist + substance bar met (Tasks 1.1-1.3)
- [ ] Multi-context-platform amendment landed (Task 1.4)
- [ ] Orchestrator sample-reads all 3 files + verifies multi-context-platform constraint woven through
- [ ] Cross-references resolve; no contradictions
- [ ] User reviews + approves Wave 1 deliverable before Wave 2 dispatch

## Wave 2-13: Per-µservice Ownership-Coherence Audits

12 batches of 8 codex agents (one agent per µservice end-to-end). Each agent produces 4 audit docs inside its µservice path. Audit is READ-ONLY (defer remediation to Wave 15+).

### Batch sequencing (per-phase order):

**Wave 2 — Phase 0 cloud-infra (Batches 2.1-2.3):**
- Batch 2.1: cloud-iam · cloud-kms · cloud-secrets · cloud-iac · cloud-network · cloud-network-dns · cloud-data · cloud-storage
- Batch 2.2: cloud-compute-functions · cloud-compute-k8s · cloud-compute-vm · cloud-billing · cloud-billing-tax · cloud-capacity · cloud-cell · cloud-dcops
- Batch 2.3: cloud-finops · cloud-marketplace · cloud-fsh · (5 carry-over slots for Phase 1)

**Wave 3 — Phase 1 foundations (Batches 3.1-3.2):**
- Batch 3.1: identity · tenancy · audit-chain · governance · compliance · observability · payments · finops-portal
- Batch 3.2: api-gateway · application · developer-sdk · network · cell · (3 carry-over slots)

  *(foundry intentionally EXCLUDED from Phase 1 — absorbed by Phase 2 per ADR-0255-amendment + ADR-0247. The `microservices/foundry/` path will be audited in a special-case Phase 2 batch + queued for retirement in Wave 15+.)*

**Wave 4 — Phase 2 core capability (Batch 4.1):**
- Batch 4.1: intelligence · ontology · workflow-engine · workflow-studio · consent-graph · detection · **foundry (special-case absorption audit — what to retain/migrate/retire)** · (1 carry-over slot)

  *(foundry's audit in Batch 4.1 has an EXTRA dimension: identify what in `microservices/foundry/` should migrate to which absorbing µservice — LLM-binding capability → intelligence; workflow templates → workflow-studio; agent state schemas → ontology; Cedar principal definitions → governance+tenancy; the rest → retirement queue.)*

**Wave 5-7 — Phase 3 comms+collab (Batches 5.1-7.3):**
- Batch 5.1: messenger · mail · drive · calendar · meet · recordings · notes · connect
- Batch 5.2: docs · sheets · slides · forms · comms-email · community · shorts · analytics
- Batch 5.3: tasks · translate · search · (5 carry-over slots for Phase 4 start)

**Wave 8-13 — Phase 4 distribution+B2B-SaaS (Batches 8.1-13.X):**
- Batch 8.1: marketplace · plugin-app-store · workplace-integration · feature-flags · ops-dashboard-control-center · brand · sites · application (PaaS-style?)
- Batch 8.2: production-planning · quality-management · plant-maintenance · warehouse · real-estate · treasury · supply-chain-planning · global-trade (Big 8 ERP)
- Batch 8.3: crm · marketing-automation · contact-center · performance-management · learning-management · itsm · incident-management · financial-planning
- Batch 8.4: contract-lifecycle-management · whiteboard · design-collaboration · data-pipeline · data-warehouse · healthcare-integration · (2 carry-over slots)

### Per-batch task template (Tasks 2.1-13.X follow this shape):

**Description:** Dispatch 8 codex agents in parallel, each owning ONE µservice end-to-end for the ownership-coherence audit. Each agent reads every file under its µservice's path, cross-references against canonical thesis + chat history + root ADRs + other µservices' interfaces, and produces 4 audit docs.

**Acceptance criteria per µservice:**
- [ ] `microservices/<name>/coherence-audit-2026-05-20.md` exists (≥1500 lines covering 5 audit dimensions)
- [ ] `microservices/<name>/feature-parity-matrix-2026-05-20.md` exists (≥800 lines with top-3 counterparts + UNION feature checklist)
- [ ] `microservices/<name>/performance-benchmark-numbers-2026-05-20.md` exists (≥400 lines with vs-counterpart numbers per tier)
- [ ] `microservices/<name>/capability-tier-deltas-vs-counterparts-2026-05-20.md` exists (≥400 lines per-tier feature deltas)
- [ ] Each doc has frontmatter (doc_class, microservice, status, date, owner)
- [ ] Each doc cites ADR-0328 + the relevant root ADRs
- [ ] Audit is READ-ONLY: no source files modified in this wave

**Per-batch verification (orchestrator runs after batch completes):**
- [ ] `for ms in <batch-µservices>; do test -f microservices/$ms/coherence-audit-2026-05-20.md || echo "MISSING $ms"; done` returns empty
- [ ] Sample-read 3 random audit docs from the 8 µservices in this batch
- [ ] Cross-check 5 agent-class-specific anchors per Wave 1 brief template
- [ ] Block batch "done" declaration until all 4 docs exist per µservice + samples pass

**Per-agent brief (encoded per ADR-0328 + brief-template.md):**
Each codex agent receives:
1. The 5-anchor header (agent-class = µservice-ownership-audit)
2. The µservice name
3. The audit 5-dimension protocol
4. The 4-doc deliverable spec
5. The HALT-CLEANLY rule (write skeleton first, deepen iteratively)
6. The codex-only directive (no scripting, no metaprogramming)
7. The oya vcs lifecycle to follow

**Dependencies:** Wave 1 must be complete + approved
**Estimated scope per agent:** Large (15-25 files read; 4 files authored per µservice)

### Checkpoint: After each batch (every 2-3 days wall-clock)

- [ ] All 8 µservices in batch have 4 audit docs each
- [ ] Orchestrator samples 3 of 8 µservices' docs + cross-checks anchors
- [ ] Aggregate batch findings into running register at `.omc/state/realignment-audit-register-2026-05-20.json`

## Wave 14: Aggregate Audit Findings + Prioritize Remediation

**Description:** Orchestrator-authored aggregation of all 12 audit batches into a master remediation backlog. Categorizes findings by severity (P0 hard contradiction / P1 substance gap / P2 cross-ref break / P3 cosmetic) + prioritizes per Big 8 priority sequence.

**Acceptance criteria:**
- [ ] File exists: `/Users/jasonlee/oyatie/.omc/plans/realignment-remediation-backlog-2026-05-20.md`
- [ ] Catalogs every finding from all 79 µservice audit docs
- [ ] Per-finding: µservice, severity, category (5 audit dimensions), file path, recommended fix
- [ ] Prioritizes per Big 8 (Big 8 hero findings ship first; long-tail later)
- [ ] Includes effort estimate (S/M/L per finding)

**Verification:**
- [ ] `wc -l .omc/plans/realignment-remediation-backlog-2026-05-20.md` ≥ 2000
- [ ] Aggregate finding count matches sum across audit docs

**Dependencies:** All Wave 2-13 batches complete
**Files likely touched:**
- `.omc/plans/realignment-remediation-backlog-2026-05-20.md`

**Estimated scope:** Medium-Large (orchestrator-authored, 1 file ~2000-4000 lines)

## Wave 15+: Remediation + Content Completion

Per audit findings + Big 8 priority. Sub-wave structure per category:

**Wave 15A — P0 hard contradiction remediation** (highest priority; ships first regardless of µservice phase)
**Wave 15B — Phase 0/1 substance gaps** (cloud-infra + foundations)
**Wave 15C — Phase 2 substance gaps** (core capability)
**Wave 15D — Phase 3 substance gaps** (comms+collab)
**Wave 15E — Phase 4 distribution + Big 8 substance gaps** (HR/Payroll first → ERP → CRM → ...)
**Wave 15F — Phase 4 long-tail B2B SaaS substance gaps**
**Wave 15G — ADR-0321 cleanup** (de-dupe + reorder + Big 8 dossier substance deepening)
**Wave 15H — P2/P3 cross-ref breaks + cosmetic**
**Wave 15I — foundry retirement + canonical-primitives Hermes-drop** — per the special-case Phase 2 audit (Batch 4.1):
  - Migrate `microservices/foundry/` LLM-binding artifacts → `microservices/intelligence/`
  - Migrate foundry workflow templates → `registry/workflow-templates/` + `microservices/workflow-studio/`
  - Migrate foundry agent-state schemas → `microservices/ontology/`
  - Migrate `oyatie.foundry.*` Cedar principal definitions → `microservices/governance/` + `microservices/tenancy/`
  - Retire `microservices/foundry/` per ADR-0138 six-path-deprecation pattern (sunset → deprecation → removal)
  - Update `tools/hooks/_canonical-primitives.md` to DROP the Hermes reference + replace with current foundry-as-capability framing
  - Author ADR-0329 codifying the foundry-as-capability vs foundry-as-µservice distinction

Each remediation wave dispatches codex agents per the brief template (5-citation header + agent-class anchors).

**Note:** Wave 15+ task breakdown will be authored AFTER Wave 14 lands (since the remediation backlog determines the task scope). This plan reserves Wave 15+ as a placeholder.

## Parallelization Opportunities

- **Safe to parallelize within a batch:** 8 µservices in same batch (different µservice paths = no file collision)
- **Must be sequential:** Wave 1 BEFORE Wave 2+ (canonical backbone gate); Wave 14 AFTER Wave 13 (aggregation gate); Wave 15+ AFTER Wave 14 (backlog gate)
- **Needs coordination:** Cross-batch handoffs where µservice A's coherence audit identifies a contradiction with µservice B → flagged for B's audit; B's audit must reciprocate

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Codex agents halt early (per past pattern) | Audit docs missing for some µservices in a batch | Per-batch verification: orchestrator runs `test -f` checks; re-dispatches failed µservices |
| Cross-µservice contradictions identified by µservice A's audit conflict with B's audit findings | Hard contradictions left unresolved | Wave 14 aggregation reconciles cross-µservice contradictions; flagged for orchestrator-authored adjudication |
| Audit doc volume overwhelms orchestrator sample-verification SLA | Some audit drift goes undetected | Sample 3 of 8 per batch (per SLA); accept some statistical risk; rely on Wave 14 aggregation to catch systemic patterns |
| Codex token budget runs out per-agent | Agents produce incomplete audits | Brief template emphasizes "skeleton first, deepen iteratively" + HALT-CLEANLY-with-checkpoint rule |
| Big 8 priority slips because audit waves take longer than estimated | Realignment ships before Big 8 hero coverage | Maintain "audit-first" discipline; if budget pressure mounts, defer Phase 4 long-tail audit to wave 15+ rather than skipping Phase 0-3 audits |

## Open Questions

None — Phase 4 interview resolved all critical unknowns.

## Pre-Dispatch Checkpoint

Before any codex dispatch occurs:
- [ ] User reviews this plan
- [ ] User approves Wave 1 brief specifically (Task 1.1 ADR-0328 + Task 1.2 master-plan-sequencing.json + Task 1.3 brief-template.md)
- [ ] Orchestrator (me) confirms 8-codex ceiling is available (current in-flight codex must finish OR be intentionally terminated to free slots for Wave 1's 3 codex)
- [ ] Existing drift agents (still in flight per session-start status) are accounted for — let them finish naturally, do not redirect

**Once user approves, Wave 1 dispatches 3 codex agents in parallel** (one per Wave 1 task). Subsequent waves dispatch after each prior wave's verification gate passes.
