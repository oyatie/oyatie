# Deep Dive Trace: realign-oyatie-corpus lane 2

**Date:** 2026-05-21
**Lane:** 2 — coordination / concurrency / ownership cause
**Scope:** audit-only; source files not modified; report artifact only
**Output:** `.omc/specs/deep-dive-trace-realign-oyatie-corpus-lane-2.md`

## Hypothesis (one-liner)

The corpus drift was materially enabled by a surface-wave coordination model: many parallel agents wrote shared files or separate surfaces of the same µservice without a durable per-µservice owner, while Oya VCS claims were either bypassed, too coarse, or only conventionally serialized.

## Executive classification

| Question | Finding | Strength |
|---|---|---|
| Did a real file race occur? | Yes. ADR-0321 contains duplicate Fly.io / Cloudflare Workers / Cloudflare R2 dossiers from overlapping D-136..D-148 and D-149..D-163 author lanes. | Strong |
| Did the corpus use surface-first waves? | Yes. Doc-suite W1..W10, per-msvc ADR A..F, runbooks W1..W4, ERP/B2B IP waves, journey waves, and Rust-source waves were organized by artifact surface, not service owner. | Strong |
| Did claim ratchet prevent collision? | Not for ADR-0321. The report found claim instructions and accepted lifecycle output in many lanes, but no evidence of a durable same-file claim barrier preventing the overlapping ADR-0321 appenders. | Strong for bypass/coarse claim; weak for ledger-level collision proof |
| Did high-touch services show coherence risk? | Yes. Highest-touch services have 9-11 inferred wave tags and dozens to hundreds of missing internal references. | Strong as risk evidence; moderate as direct contradiction evidence |
| Is coordination the only cause? | No. Lane 1 brief scope and Lane 3 verification gaps are necessary co-causes; lane 2 explains the race and ownership spread. | Strong |

## Evidence FOR (top-15 with file paths + line citations + brief excerpts)

### F-01 — STRONG
ADR-0321 live file contains the exact duplicate vendor sections named in the hypothesis: D-149 Fly.io at line
19675, D-150 Cloudflare Workers at 19840, D-151 Cloudflare R2 at 20356, then D-139 Fly.io at 22240, D-140
Cloudflare Workers at 22404, and D-141 Cloudflare R2 at 22571.
Citation: `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md:19675;19840;20356;22240;22404;22571`

### F-02 — STRONG
The D-149..D-163 dispatch told a parallel author to append 15 net-new sections to the same ADR-0321 file and
offered Fly.io, Cloudflare Workers, Cloudflare R2, MongoDB Atlas, Redis Enterprise Cloud, Confluent Cloud,
PlanetScale, Neon, Supabase, and Algolia as candidates.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13047`

### F-03 — STRONG
The D-136..D-148 Codex dispatch also targeted the same file, also appended at file end, and explicitly
assigned D-139 Fly.io, D-140 Cloudflare Workers, D-141 Cloudflare R2, and D-142 MongoDB Atlas.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13215; /private/tmp/codex-out-adr-0321-finish-d136-d148.log:14-35`

### F-04 — STRONG
The D-149 agent was later stopped with only 5 of 15 sections done, leaving a partial overlap window after the
duplicate sections had landed.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13623-13628`

### F-05 — STRONG
The orchestrator directly recorded that parallel agents added D-126..D-141 plus D-149..D-153 concurrently and
that sections were appended at EOF via cat >> to avoid stale-read conflicts, producing non-monotonic ordering.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13555`

### F-06 — STRONG
ADR-0321 has no git history at the current path; git log returns empty and the file is untracked in git
status. This makes transcript/task logs and filesystem metadata the only available write-order evidence.
Citation: `git log -- docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md; git status --short`

### F-07 — STRONG
ADR-0321 filesystem metadata brackets the race: created 2026-05-20T20:20:37-0400, modified
2026-05-20T20:53:25-0400, 23,887 lines, 155 D-section headers.
Citation: `stat/wc/rg local audit`

### F-08 — STRONG
The session repeatedly ran at the declared parallel ceiling: 4 Claude plus 8 Codex = 12 agents in parallel
authoring journeys, ADR-0321, doc-suite, IPs, ADRs, personas, runbooks, pack overlays, and cross-handoff
matrices.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:11122;11436;11532;11891`

### F-09 — STRONG
Per-µservice ADR batches were organized by surface type, not service ownership. Batch A targeted
messenger/mail/drive/calendar/identity/tenancy/governance/compliance; batch B targeted observability/audit-
chain/payments/finops-portal/intelligence/ontology/workflow-engine/workflow-studio; later batches continued by
selecting remaining services.
Citation: `/private/tmp/codex-out-per-msvc-adrs-a.log:14-43; /private/tmp/codex-out-per-msvc-adrs-b.log:14-47; /private/tmp/codex-out-per-msvc-adrs-c.log:14-54; /private/tmp/codex-out-per-msvc-adrs-d.log:14-48; /private/tmp/codex-out-per-msvc-adrs-e.log:14-51; /private/tmp/codex-out-per-msvc-adrs-f.log:14-46`

### F-10 — STRONG
Runbook waves were also surface-first: W1 targeted observability/identity/tenancy/governance/compliance; W2
audit-chain/payments/finops-portal/intelligence/ontology; W3 workflow-engine/workflow-
studio/marketplace/workplace-integration/community; W4 cloud-iam/cloud-kms/cloud-billing/cloud-
network/intelligence.
Citation: `/private/tmp/codex-out-runbooks-substrate.log:14-46; /private/tmp/codex-out-runbooks-substrate-w2.log:14-47; /private/tmp/codex-out-runbooks-substrate-w3.log:14-46; /private/tmp/codex-out-runbooks-substrate-w4.log:14-42`

### F-11 — STRONG
Doc-suite waves were dispatched by batches across service sets: W2 covered
analytics/shorts/detection/recordings/network/sheets/consent-graph/sites while reporting a prior wave for
cloud-k8s/feature-flags/tasks/translate; W3 covered audit-chain/payments/finops-portal/intelligence/workflow-
engine/workflow-studio/ontology/community; W4 covered api-gateway/comms-
email/connect/meet/notes/slides/forms/cell; W5 covered application/cloud-iac/cloud-secrets/developer-
sdk/foundry/marketplace/workplace-integration.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:10465;10810;11122;11262`

### F-12 — MODERATE
W8 explicitly needed to pick untouched services and avoid W7, ERP, and B2B-leader services. That is evidence
of manual collision avoidance by prompt convention, not a durable ownership ledger.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:12281`

### F-13 — STRONG
The live matrix shows 78 service directories; the user brief and one wave-10 result refer to 79. Anonymous
exists in git deletion state but not as a live directory. This is registry/live-dir drift amid concurrent
corpus surgery.
Citation: `find microservices -mindepth 1 -maxdepth 1; git status --short microservices/anonymous; /Users/jasonlee/.claude/...jsonl:12804`

### F-14 — STRONG
The high-touch services are exactly where false cross-references concentrate: workflow-engine has 335 missing
backtick refs, audit-chain 217, compliance 227, tenancy 202, payments 123, ontology 104, intelligence 95,
developer-sdk 94.
Citation: `/private/tmp/lane2_matrix.tsv`

### F-15 — STRONG
After contradiction risk was acknowledged, the system dispatched a new ownership-coherence audit for audit-
chain that required one owner to read every artifact under the service path and cross-reference PRD, ADRs,
IPs, contracts, runbooks, capability tiers, SLOs, and migration playbooks. That protocol appeared after the
drift, not before it.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13439;13449;13462`

## Evidence AGAINST (top-10)

### A-01 — MODERATE
Some individual briefs did include bounded target lists and DO NOT TOUCH instructions. The failure is
therefore not simply no coordination text; it is that coordination was local to batches and surfaces, not to
semantic ownership.
Citation: `/private/tmp/codex-out-per-msvc-adrs-a.log:39-43; /private/tmp/codex-out-runbooks-substrate-w2.log:43-47`

### A-02 — MODERATE
Oya VCS was not absent. Multiple wave logs show claim/verify/done/promote instructions and acceptance. The
strongest claim is claim bypass/coarse granularity, not total lack of claim tooling.
Citation: `/private/tmp/codex-out-runbooks-substrate-w2.log:780-784; /private/tmp/codex-out-per-msvc-adrs-e.log:60521`

### A-03 — MODERATE
The controlled IP-010 demo proves the claim system can block a duplicate claim in an idealized case. The
incident was not that the primitive cannot ever work.
Citation: `evidence/agentic-pipeline/ip-010-parallel-claim-demo.json; evidence/agentic-pipeline/ip-010-parallel-claim-demo-transcript/agent-c-negative-claim.log`

### A-04 — MODERATE
Some drift is brief/content-scope driven. The orchestrator admitted candidate lists included cloud infra
primitives that did not belong in ADR-0321. Coordination did not create the wrong candidate list.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13524`

### A-05 — MODERATE
Some drift is verification-driven. Thin per-service ADRs and premature completion notices could have been
caught earlier by content validation even without changing ownership.
Citation: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:13333;13347`

### A-06 — WEAK
Line-count and file-count growth were sometimes genuine. For example per-msvc ADR batch E reported every ADR
>=200 lines and VCS accepted the lifecycle. That reduces the claim that every surface-wave output was shallow.
Citation: `/private/tmp/codex-out-per-msvc-adrs-e.log:60512-60521`

### A-07 — WEAK
A service can be touched by many waves without contradiction if later waves read and reconcile prior
artifacts. The matrix counts are risk indicators, not proof by themselves.
Citation: `/private/tmp/lane2_matrix.tsv`

### A-08 — WEAK
Some missing references are implementation-plan stubs that may intentionally point to future artifacts. They
still indicate drift risk, but not every absent file is a defect.
Citation: `microservices/*/IP-journey-* local scan`

### A-09 — WEAK
The ADR-0321 file race is proven; the same level of per-byte write-order proof is not available for all
microservice surfaces.
Citation: `git log empty; file untracked`

### A-10 — WEAK
The report cannot prove that two Oya VCS active claims overlapped on the same service scope at the ledger
level because a durable session-time claim ledger snapshot was not found.
Citation: `./bin/oya vcs status; find . -maxdepth 4 .oya/claim evidence`

## Brief classification matrix

| Class | Coordination shape | Collision risk | Evidence | Lane-2 meaning |
|---|---|---:|---|---|
| Shared single-file appenders | Many agents append to one mutable document | Critical | ADR-0321 D-136..D-148 and D-149..D-163 | Proven race surface |
| Surface batch over many services | One agent writes the same artifact class across many µservices | High | per-msvc ADR A..F; runbooks W1..W4 | No one owns whole service coherence |
| Doc-suite broad gapfill | One agent fills many documentation surfaces for a service set | High | doc-suite W1..W10 | Potentially creates feature claims before contracts/source exist |
| Journey wave fan-out | Journey implementation plans distributed across services | High | IP-journey files and missing refs | Creates per-journey contracts/tests/runbooks without matching surfaces |
| Source scaffold wave | Code/source surfaces added after docs/IPs | Moderate | rust-src W1..W3 | Can lag behind docs and create docs-before-code mismatch |
| Claim lifecycle present | claim/verify/done/promote appears in prompts/logs | Mixed | Oya VCS accepted several wave claims | Tool exists but did not enforce semantic ownership |
| Manual avoid list | Prompt says avoid other in-flight services | Moderate | doc-suite W8 | Collision prevention depends on human prompt hygiene |
| Per-service ownership audit | One owner reads every artifact under one service | Low once applied | audit-chain ownership audit | Introduced as remediation after drift |

## Wave catalog

The catalog below is intentionally conservative. It uses exact targets where transcript or task-log prompts exposed them, and marks dynamic or incompletely reconstructed waves as such.

| Wave | Surface | Target services / file scope | Evidence |
|---|---|---|---|
| `doc-suite-W1-bootstrap` | full-buildout + borderline gap fill | payments, api-gateway, feature-flags, intelligence, connect, ops-dashboard-control-center; borderline compliance, tenancy, comms-email, finops-portal, ontology, mail, notes, social | chat line 4073 plus dispatch lines 4221/4228/4235/6706/6713/6720 |
| `doc-suite-W2` | gapfill | analytics, shorts, detection, recordings, network, sheets, consent-graph, sites; prior wave outcome cloud-k8s, feature-flags, tasks, translate | chat line 10465 |
| `doc-suite-W3` | gapfill | audit-chain, payments, finops-portal, intelligence, workflow-engine, workflow-studio, ontology, community | chat lines 10792/10810 |
| `doc-suite-W4` | gapfill | api-gateway, comms-email, connect, meet, notes, slides, forms, cell | chat line 11122 |
| `doc-suite-W5` | gapfill | application, cloud-iac, cloud-secrets, developer-sdk, foundry, marketplace, workplace-integration, one truncated/unknown | chat line 11262 |
| `doc-suite-W6` | dynamic | 44/60 coverage progress; exact targets not fully reconstructed from available prompt lines | chat lines 11578/11891 |
| `doc-suite-W7` | dynamic | in flight during W8; protected by avoid-list rather than service owner | chat line 12281 |
| `doc-suite-W8` | dynamic collision-avoidance wave | prompt required picking 8 not touched by concurrent agents and avoiding W7/ERP/B2B leader sets | chat line 12281 |
| `doc-suite-W9` | cloud-family completion | cloud family inferred from wave state and matrix: cloud-iam/cloud-kms/cloud-billing/cloud-network/cloud-data/cloud-storage/cloud-network-dns/cloud-billing-tax | matrix + chat line 12710 |
| `doc-suite-W10-final` | final audit/fill | reported all 79 audited and 17 protected gaps only; live dirs now 78 | chat line 12804 + filesystem count |
| `per-msvc-adrs-A` | surface ADR batch | messenger, mail, drive, calendar, identity, tenancy, governance, compliance | per-msvc-adrs-a.log:14-43 |
| `per-msvc-adrs-B` | surface ADR batch | observability, audit-chain, payments, finops-portal, intelligence, ontology, workflow-engine, workflow-studio | per-msvc-adrs-b.log:14-47 |
| `per-msvc-adrs-C` | surface ADR batch | tasks, notes, sheets, slides, meet, forms, docs, comms-email | per-msvc-adrs-c.log:14-54 |
| `per-msvc-adrs-D` | surface ADR batch | shorts, recordings, network, marketplace, workplace-integration, application, cloud-iac, foundry | per-msvc-adrs-d.log:14-48 |
| `per-msvc-adrs-E` | surface ADR batch | cloud-secrets, developer-sdk, analytics, consent-graph, api-gateway, connect, contact-center, crm, data-pipeline, data-warehouse, feature-flags observed in final diffs; prompt initially listed 8 and allowed live re-pick | per-msvc-adrs-e.log:14-51;60512-60521 |
| `per-msvc-adrs-F` | surface ADR batch | live empty-decisions selection; exact final targets not fully reconstructed from first 90 prompt lines | per-msvc-adrs-f.log:14-46 |
| `runbooks-W1` | runbook surface batch | observability, identity, tenancy, governance, compliance | runbooks-substrate.log:14-46 |
| `runbooks-W2` | runbook surface batch | audit-chain, payments, finops-portal, intelligence, ontology | runbooks-substrate-w2.log:14-47 |
| `runbooks-W3` | runbook surface batch | workflow-engine, workflow-studio, marketplace, workplace-integration, community | runbooks-substrate-w3.log:14-46 |
| `runbooks-W4` | runbook surface batch | cloud-iam, cloud-kms, cloud-billing, cloud-network, intelligence | runbooks-substrate-w4.log:14-42 |
| `erp-ip-W1` | ERP IP expansion | production-planning, quality-management, plant-maintenance | erp logs + matrix |
| `erp-ip-W2` | ERP IP expansion | warehouse, real-estate, crm | warehouse/realestate/crm logs + matrix |
| `erp-ip-W3` | ERP IP expansion | treasury, supply-chain-planning, global-trade | erp-ip-w3.log + chat line 10700 |
| `b2b-ip-deepen-A` | B2B IP deepen | marketing-automation, contact-center, performance-management, learning-management | chat line 10700 + b2b log names |
| `b2b-ip-deepen-B` | B2B IP deepen | itsm, incident-management, financial-planning, data-warehouse | chat line 10700 |
| `b2b-ip-deepen-C` | B2B IP deepen | contract-lifecycle-management, whiteboard, design-collaboration, data-pipeline, healthcare-integration | chat line 10700 |
| `journeys-J001-J180` | journey IP waves | all services with IP-journey files; high counts include workflow-engine 96, audit-chain 81, payments 70, compliance 64, tenancy 61 | matrix + codex-out-j*.log corpus |
| `adr-0321-substance-W1-W10` | ADR-0321 vendor dossiers | D-001..D-110-ish staged dossier backfill before late D-111..D-155 author waves | chat lines 10465/10810/10887/11122/11262/11891 |
| `adr-0321-authors-D111-D155` | ADR-0321 late author waves | D-111..D-125 clean; D-136..D-148 and D-149..D-163 overlapped on Fly.io / Cloudflare Workers / Cloudflare R2 / MongoDB Atlas | chat lines 13047/13215/13555/13628 |
| `rust-src-W1` | source scaffold surface | marketing-automation, contact-center, incident-management, financial-planning, contract-lifecycle-management, whiteboard | rust-src logs + matrix |
| `rust-src-W2` | source scaffold surface | performance-management, learning-management, itsm, data-warehouse, design-collaboration, data-pipeline | rust-src-w2 logs + matrix |
| `rust-src-W3` | source scaffold surface | production-planning, quality-management, plant-maintenance, warehouse, real-estate, crm, treasury, supply-chain-planning, global-trade, healthcare-integration | rust-src-w3 logs + matrix |
| `capability-tier-deltas` | capability-tier surface | services with capability-tiers/ present; often added outside service ownership pass | matrix |
| `migration-playbooks` | migration-playbook surface | services with migration-playbooks/ present; often added outside service ownership pass | matrix |
| `pack-overlays` | pack/compliance overlay surface | services with pack/packs/pack-overlays indicators | matrix |
| `cross-handoff-matrix` | cross-service handoff surface | services with cross-microservice-handoffs.md | matrix |

## Specific coordination / collision incidents

### I-01 ADR-0321 duplicate vendor race
Strength: Strong
Two late author lanes targeted the same file and overlapping vendors. The D-149 lane was told to pick from a
list containing Fly.io, Cloudflare Workers, Cloudflare R2, MongoDB Atlas and others. The D-136 lane was
explicitly assigned D-139 Fly.io, D-140 Cloudflare Workers, D-141 Cloudflare R2, D-142 MongoDB Atlas. Live
file order shows D-149..D-152 before D-139..D-142. This is a direct race, not merely a stylistic issue.

### I-02 ADR-0321 append-at-EOF convention
Strength: Strong
The clean D-111..D-125 agent later acknowledged parallel agents added D-126..D-141 and D-149..D-153 while it
appended at EOF via cat >> to avoid stale-read conflicts. That workaround reduced edit-tool failure but
preserved semantic and numeric ordering collisions.

### I-03 Per-msvc ADR A-F without service ownership
Strength: Strong
The ADR waves asked each batch to author decisions/ files for eight services, with no requirement that the
same agent reconcile PRD, contracts, SLOs, runbooks, migration playbooks, and journey IPs. A later rewrite was
needed for thin ADRs.

### I-04 Doc-suite by surface type
Strength: Strong
Doc-suite waves added capability tiers, onboarding, FAQs, tutorials, benchmarks, migration playbooks,
reference implementations, and related surfaces across service sets. That means service PRDs/contracts/source
could be bypassed or lag behind claims.

### I-05 Runbook waves after doc-suite and ADR waves
Strength: Moderate
Runbooks W1-W4 rewrote or added operational surfaces after service documentation and ADRs had already landed.
A runbook author could reference dashboards/SLOs/contracts created by another wave or not created at all.

### I-06 Journey IP waves with missing refs
Strength: Strong as artifact-gap evidence
Journey IPs repeatedly cite per-journey contracts, policies, tests, dashboards, and runbooks that are absent.
This is consistent with one wave drafting plan references while no owner completes the referenced surfaces.

### I-07 Oya VCS syntax/usage instability
Strength: Moderate
The transcript shows early claim syntax exploration where --scope failed and a positional claim was accepted.
That is not the root cause by itself, but it shows the ratchet was being learned and could be inconsistently
used mid-flight.

### I-08 Long-held incomplete claim
Strength: Moderate
The j151-j175 corrected journey agent reported a completed task notification but admitted only one README plus
empty dirs and no verify/done/promote; the claim lock remained unresolved by the agent output.

### I-09 Manual avoid lists instead of ownership ledger
Strength: Moderate
Doc-suite W8 was prompted to choose services untouched by concurrent agents and avoid W7/ERP/B2B leader lists.
This is human-memory coordination, not enforceable per-service ownership.

### I-10 Ownership audit introduced after the problem
Strength: Strong
The first audit-chain ownership-coherence pass was dispatched only after design-doc contradictions were
acknowledged. That confirms the pre-existing wave model lacked the needed ownership protocol.

## ADR-0321 race reconstruction

- **File state:** `docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md` is untracked, 2,807,222 bytes, 23,887 lines, 155 `### Section D-` headers.
- **Filesystem time bracket:** birth 2026-05-20T20:20:37-0400; modify/change 2026-05-20T20:53:25-0400.
- **Git history:** `git log --oneline -- docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md` returned no commits because the file is untracked.
- **First duplicate block:** D-149 Fly.io at line 19675, D-150 Cloudflare Workers at 19840, D-151 Cloudflare R2 at 20356, D-152 MongoDB Atlas at 20513.
- **Second duplicate block:** D-139 Fly.io at line 22240, D-140 Cloudflare Workers at 22404, D-141 Cloudflare R2 at 22571, D-142 MongoDB Atlas at 22735.
- **Dispatch time 1:** D-149..D-163 agent dispatched at transcript line 13047 timestamp 2026-05-21T00:23:50.505Z.
- **Dispatch time 2:** D-136..D-148 Codex finisher dispatched at transcript line 13215 timestamp 2026-05-21T00:31:45.250Z.
- **Stop time:** D-149..D-163 agent killed at transcript lines 13623-13628 timestamp 2026-05-21T00:48:31Z, result: 5 of 15 done.
- **Race verdict:** The D-149 lane and D-136 lane both had reason to believe the vendor choices were net-new. The D-149 lane could not see the later D-139/D-142 writes; the D-136 lane did not prevent duplication with the already-started D-149 lane.

## Claim ledger audit

- **Observed CLI status:** `./bin/oya vcs status` returned: `oya vcs status accepted: action=read-status agent=- scopes=0 evidence=0`.
- **No live .oya ledger located:** A shallow search found no `.oya` ledger tree in the repository root. The visible claim evidence is mostly task-log and transcript output.
- **Positive control exists:** `evidence/agentic-pipeline/ip-010-parallel-claim-demo.json` and transcript files show a duplicate claim can be blocked in a controlled demo.
- **Session-level weakness:** For this corpus run, wave prompts often used broad scopes such as whole service paths or one shared ADR file; no evidence shows section-level or semantic ownership claims for ADR-0321.
- **Claim bypass vs claim failure:** The strongest proven failure is not that Oya VCS cannot block collisions. It is that this authoring operation either bypassed the claim barrier, used too-coarse claims, or used claims that did not encode per-section/vendor ownership.
- **Specific ratchet miss:** D-149..D-163 and D-136..D-148 both operated on the same ADR file and overlapping vendor space; the live file contains the collision. Therefore the effective ratchet failed to prevent this collision, regardless of whether a durable ledger row ever existed.
- **Open gap:** The exact active-claim ledger state at 2026-05-21T00:23-00:48Z is not recoverable from current filesystem evidence in this audit.

## Per-µservice agents-that-touched-me matrix

Interpretation rules: `wave_count` is inferred from transcript/log wave assignments plus live filesystem surface indicators. It is not a cryptographic authorship count. `missing_refs` is a backtick-path scan for `microservices/...` references in Markdown that do not currently resolve. High values are coherence-risk evidence, not automatic proof that every reference is invalid by design.

| Rank | µservice | Wave count | Surface count | Files | Journey IPs | ADR files | Runbooks | Missing refs |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | `payments` | 11 | 20 | 202 | 70 | 1 | 13 | 123 |
| 2 | `intelligence` | 11 | 20 | 176 | 38 | 1 | 17 | 95 |
| 3 | `audit-chain` | 10 | 21 | 251 | 81 | 1 | 9 | 217 |
| 4 | `workplace-integration` | 10 | 20 | 135 | 16 | 1 | 13 | 31 |
| 5 | `finops-portal` | 10 | 19 | 161 | 29 | 8 | 11 | 38 |
| 6 | `comms-email` | 10 | 19 | 136 | 10 | 6 | 10 | 5 |
| 7 | `ontology` | 10 | 18 | 154 | 30 | 2 | 13 | 104 |
| 8 | `api-gateway` | 10 | 18 | 136 | 14 | 1 | 13 | 20 |
| 9 | `contact-center` | 9 | 21 | 172 | 0 | 1 | 20 | 0 |
| 10 | `workflow-studio` | 9 | 20 | 226 | 15 | 7 | 12 | 154 |
| 11 | `cloud-iac` | 9 | 20 | 175 | 15 | 1 | 8 | 126 |
| 12 | `developer-sdk` | 9 | 20 | 137 | 11 | 7 | 8 | 94 |
| 13 | `marketplace` | 9 | 20 | 134 | 15 | 1 | 13 | 31 |
| 14 | `compliance` | 9 | 19 | 201 | 64 | 6 | 13 | 227 |
| 15 | `cloud-secrets` | 9 | 19 | 134 | 17 | 1 | 6 | 66 |
| 16 | `workflow-engine` | 9 | 18 | 226 | 96 | 1 | 9 | 335 |
| 17 | `tenancy` | 9 | 18 | 197 | 61 | 1 | 11 | 202 |
| 18 | `connect` | 9 | 18 | 184 | 36 | 1 | 10 | 44 |
| 19 | `notes` | 9 | 18 | 160 | 27 | 8 | 11 | 68 |
| 20 | `application` | 9 | 18 | 135 | 11 | 1 | 6 | 49 |
| 21 | `feature-flags` | 9 | 18 | 133 | 10 | 1 | 9 | 8 |
| 22 | `itsm` | 8 | 21 | 173 | 0 | 1 | 20 | 1 |
| 23 | `incident-management` | 8 | 21 | 172 | 0 | 1 | 20 | 1 |
| 24 | `contract-lifecycle-management` | 8 | 21 | 172 | 0 | 1 | 20 | 0 |
| 25 | `foundry` | 8 | 20 | 592 | 14 | 2 | 41 | 438 |
| 26 | `identity` | 8 | 20 | 237 | 112 | 6 | 11 | 276 |
| 27 | `mail` | 8 | 20 | 208 | 75 | 6 | 10 | 329 |
| 28 | `plugin-app-store` | 8 | 20 | 147 | 21 | 5 | 8 | 130 |
| 29 | `observability` | 8 | 19 | 214 | 51 | 1 | 13 | 225 |
| 30 | `governance` | 8 | 19 | 204 | 20 | 2 | 11 | 111 |
| 31 | `healthcare-integration` | 8 | 19 | 160 | 0 | 1 | 20 | 0 |
| 32 | `analytics` | 8 | 19 | 138 | 10 | 5 | 8 | 23 |
| 33 | `recordings` | 8 | 19 | 129 | 12 | 9 | 8 | 10 |
| 34 | `community` | 8 | 18 | 202 | 50 | 6 | 9 | 85 |
| 35 | `cell` | 8 | 18 | 147 | 26 | 1 | 6 | 90 |
| 36 | `forms` | 8 | 18 | 145 | 16 | 8 | 7 | 134 |
| 37 | `meet` | 8 | 18 | 139 | 21 | 8 | 7 | 50 |
| 38 | `consent-graph` | 8 | 18 | 135 | 19 | 5 | 8 | 52 |
| 39 | `slides` | 8 | 18 | 129 | 10 | 10 | 7 | 7 |
| 40 | `sheets` | 8 | 18 | 126 | 10 | 9 | 7 | 42 |
| 41 | `network` | 8 | 18 | 126 | 14 | 8 | 7 | 32 |
| 42 | `tasks` | 8 | 18 | 124 | 10 | 8 | 7 | 56 |
| 43 | `shorts` | 8 | 18 | 124 | 14 | 8 | 7 | 23 |
| 44 | `data-warehouse` | 8 | 15 | 166 | 0 | 1 | 20 | 0 |
| 45 | `drive` | 7 | 21 | 175 | 46 | 8 | 7 | 133 |
| 46 | `messenger` | 7 | 19 | 166 | 47 | 6 | 10 | 139 |
| 47 | `calendar` | 7 | 19 | 139 | 19 | 6 | 12 | 114 |
| 48 | `cloud-k8s` | 7 | 19 | 123 | 12 | 1 | 9 | 54 |
| 49 | `docs` | 7 | 18 | 128 | 10 | 8 | 7 | 72 |
| 50 | `sites` | 7 | 18 | 123 | 10 | 8 | 7 | 15 |
| 51 | `translate` | 7 | 18 | 122 | 11 | 7 | 7 | 24 |
| 52 | `ops-dashboard-control-center` | 7 | 17 | 154 | 24 | 0 | 11 | 74 |
| 53 | `data-pipeline` | 7 | 14 | 163 | 0 | 1 | 20 | 1 |
| 54 | `crm` | 7 | 14 | 155 | 0 | 1 | 6 | 0 |
| 55 | `social` | 7 | 11 | 144 | 14 | 7 | 12 | 35 |
| 56 | `detection` | 6 | 18 | 130 | 0 | 1 | 8 | 1 |
| 57 | `financial-planning` | 6 | 14 | 165 | 0 | 1 | 20 | 0 |
| 58 | `design-collaboration` | 6 | 14 | 163 | 0 | 1 | 20 | 0 |
| 59 | `whiteboard` | 5 | 13 | 164 | 0 | 0 | 20 | 2 |
| 60 | `marketing-automation` | 5 | 13 | 164 | 0 | 0 | 20 | 0 |
| 61 | `learning-management` | 5 | 13 | 162 | 0 | 0 | 20 | 0 |
| 62 | `performance-management` | 5 | 13 | 162 | 0 | 0 | 20 | 0 |
| 63 | `global-trade` | 5 | 11 | 138 | 0 | 1 | 6 | 0 |
| 64 | `cloud-billing` | 5 | 8 | 10 | 0 | 0 | 3 | 0 |
| 65 | `cloud-iam` | 5 | 8 | 10 | 0 | 0 | 3 | 0 |
| 66 | `cloud-kms` | 5 | 8 | 10 | 0 | 0 | 3 | 0 |
| 67 | `cloud-network` | 5 | 8 | 10 | 0 | 0 | 3 | 0 |
| 68 | `production-planning` | 4 | 12 | 152 | 0 | 0 | 6 | 0 |
| 69 | `plant-maintenance` | 4 | 12 | 151 | 0 | 0 | 6 | 1 |
| 70 | `quality-management` | 4 | 12 | 151 | 0 | 0 | 6 | 0 |
| 71 | `real-estate` | 4 | 12 | 151 | 0 | 0 | 6 | 0 |
| 72 | `treasury` | 4 | 12 | 151 | 0 | 0 | 6 | 0 |
| 73 | `warehouse` | 4 | 12 | 151 | 0 | 0 | 6 | 0 |
| 74 | `supply-chain-planning` | 4 | 12 | 149 | 0 | 0 | 6 | 0 |
| 75 | `cloud-billing-tax` | 3 | 7 | 7 | 0 | 0 | 0 | 0 |
| 76 | `cloud-data` | 3 | 7 | 7 | 0 | 0 | 0 | 0 |
| 77 | `cloud-network-dns` | 3 | 7 | 7 | 0 | 0 | 0 | 0 |
| 78 | `cloud-storage` | 3 | 7 | 7 | 0 | 0 | 0 | 0 |

## Hotspot ranking

### Hotspot 1: `payments`
- Inferred wave tags: 11
- Surface directories / root buckets: 20
- File count under service path: 202
- Journey IP files: 70
- Decision files: 1
- Runbook files: 13
- Missing internal backtick refs: 123
- Wave tags: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- Surfaces: (root):107, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:5, dashboards:8,
  decisions:1, faqs:1, iac:16, migration-playbooks:4, onboarding:1, policy:10, reference-implementations:1,
  runbooks:13, scorecards:1, security:1, slos:8, test-plans:3, tutorials:1
- Missing-ref samples: microservices/payments/IP-journey-j65-receipt-
  export.md:52->microservices/payments/contracts/openapi-j65-receipt-export.yaml; microservices/payments/IP-
  journey-j65-receipt-export.md:53->microservices/payments/contracts/asyncapi-j65-receipt-export.yaml;
  microservices/payments/IP-journey-j65-receipt-export.md:54->microservices/payments/contracts/j65-receipt-
  export.proto
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 2: `intelligence`
- Inferred wave tags: 11
- Surface directories / root buckets: 20
- File count under service path: 176
- Journey IP files: 38
- Decision files: 1
- Runbook files: 17
- Missing internal backtick refs: 95
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-W4, runbooks-surface-present
- Surfaces: (root):84, benchmarks:1, capabilities:8, capability-tiers:1, catalog:12, contracts:7, dashboards:6,
  decisions:1, faqs:1, iac:8, migration-playbooks:1, onboarding:1, policy:12, reference-implementations:1,
  runbooks:17, scorecards:1, security:1, slos:9, test-plans:3, tutorials:1
- Missing-ref samples: microservices/intelligence/IP-002-domain-layer-secret-
  reference.md:25->microservices/intelligence/crates/; microservices/intelligence/IP-002-domain-layer-secret-
  reference.md:32->microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/Cargo.toml;
  microservices/intelligence/IP-002-domain-layer-secret-reference.md:33->microservices/intelligence/crates/oya-
  intelligence-credential-resolver-domain/src/lib.rs
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 3: `audit-chain`
- Inferred wave tags: 10
- Surface directories / root buckets: 21
- File count under service path: 251
- Journey IP files: 81
- Decision files: 1
- Runbook files: 9
- Missing internal backtick refs: 217
- Wave tags: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- Surfaces: (root):115, benchmarks:1, capabilities:3, capability-tiers:1, catalog:39, contracts:3, dashboards:3,
  decisions:1, faqs:1, iac:10, migration-playbooks:1, onboarding:1, packs:5, policy:44, reference-
  implementations:1, runbooks:9, scorecards:1, security:1, slos:7, test-plans:3, tutorials:1
- Missing-ref samples: microservices/audit-chain/IP-journey-j55-dispute-seal.md:52->microservices/audit-
  chain/contracts/openapi-j55-dispute-seal.yaml; microservices/audit-chain/IP-journey-j55-dispute-
  seal.md:53->microservices/audit-chain/contracts/asyncapi-j55-dispute-seal.yaml; microservices/audit-chain/IP-
  journey-j55-dispute-seal.md:54->microservices/audit-chain/contracts/j55-dispute-seal.proto
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 4: `workplace-integration`
- Inferred wave tags: 10
- Surface directories / root buckets: 20
- File count under service path: 135
- Journey IP files: 16
- Decision files: 1
- Runbook files: 13
- Missing internal backtick refs: 31
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-W3, runbooks-surface-present, rust-src-surface-present
- Surfaces: (root):36, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:3, dashboards:5,
  decisions:1, faqs:1, iac:12, ip:25, migration-playbooks:1, onboarding:1, policies:6, reference-implementations:1,
  runbooks:13, scorecards:1, slos:6, src:1, tutorials:1
- Missing-ref samples: microservices/workplace-integration/IP-
  journey-j54-e-signature.md:52->microservices/workplace-integration/contracts/openapi-j54-e-signature.yaml;
  microservices/workplace-integration/IP-journey-j54-e-signature.md:53->microservices/workplace-
  integration/contracts/asyncapi-j54-e-signature.yaml; microservices/workplace-integration/IP-
  journey-j54-e-signature.md:54->microservices/workplace-integration/contracts/j54-e-signature.proto
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 5: `finops-portal`
- Inferred wave tags: 10
- Surface directories / root buckets: 19
- File count under service path: 161
- Journey IP files: 29
- Decision files: 8
- Runbook files: 11
- Missing internal backtick refs: 38
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- Surfaces: (root):48, benchmarks:1, capabilities:3, capability-tiers:1, catalog:6, contracts:3, dashboards:6,
  decisions:8, faqs:1, iac:19, implementation-plans:26, migration-playbooks:1, onboarding:1, policy:10, reference-
  implementations:1, runbooks:11, scorecards:5, slos:9, tutorials:1
- Missing-ref samples: microservices/finops-portal/IP-journey-j82-finance-risk-console.md:37->microservices/finops-
  portal/contracts/openapi/j82-finance-risk-console-v1.yaml; microservices/finops-portal/IP-journey-j82-finance-
  risk-console.md:38->microservices/finops-portal/contracts/asyncapi/j82-finance-risk-console-events-v1.yaml;
  microservices/finops-portal/IP-journey-j82-finance-risk-console.md:39->microservices/finops-
  portal/contracts/proto/j82-finance-risk-console-v1.proto
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 6: `comms-email`
- Inferred wave tags: 10
- Surface directories / root buckets: 19
- File count under service path: 136
- Journey IP files: 10
- Decision files: 6
- Runbook files: 10
- Missing internal backtick refs: 5
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present, rust-src-surface-present
- Surfaces: (root):54, benchmarks:1, capabilities:6, capability-tiers:1, catalog:1, contracts:3, dashboards:5,
  decisions:6, faqs:1, iac:17, migration-playbooks:1, onboarding:1, policy:12, reference-implementations:1,
  runbooks:10, scorecards:5, slos:9, src:1, tutorials:1
- Missing-ref samples: microservices/comms-email/IP-006-mjml-template-renderer.md:36->microservices/comms-
  email/templates/canonical-base/{template_id}/{locale}.mjml; microservices/comms-email/IP-006-mjml-template-
  renderer.md:37->microservices/comms-email/templates/packs/{pack}/{template_id}/{locale}.mjml;
  microservices/comms-email/IP-015-in-house-relay-roadmap-phase-2.md:71->microservices/comms-email/iac/helm/oya-
  comms-email-server/
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 7: `ontology`
- Inferred wave tags: 10
- Surface directories / root buckets: 18
- File count under service path: 154
- Journey IP files: 30
- Decision files: 2
- Runbook files: 13
- Missing internal backtick refs: 104
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- Surfaces: (root):71, benchmarks:1, capabilities:3, capability-tiers:1, catalog:18, contracts:3, dashboards:5,
  decisions:2, faqs:1, iac:16, migration-playbooks:1, onboarding:1, policy:9, reference-implementations:1,
  runbooks:13, scorecards:1, slos:6, tutorials:1
- Missing-ref samples: microservices/ontology/IP-journey-j84-typed-record-
  writer.md:37->microservices/ontology/contracts/openapi/j84-typed-record-writer-v1.yaml;
  microservices/ontology/IP-journey-j84-typed-record-
  writer.md:38->microservices/ontology/contracts/asyncapi/j84-typed-record-writer-events-v1.yaml;
  microservices/ontology/IP-journey-j84-typed-record-
  writer.md:39->microservices/ontology/contracts/proto/j84-typed-record-writer-v1.proto
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 8: `api-gateway`
- Inferred wave tags: 10
- Surface directories / root buckets: 18
- File count under service path: 136
- Journey IP files: 14
- Decision files: 1
- Runbook files: 13
- Missing internal backtick refs: 20
- Wave tags: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- Surfaces: (root):53, benchmarks:1, capabilities:4, capability-tiers:1, catalog:14, contracts:4, dashboards:8,
  decisions:1, faqs:1, iac:13, migration-playbooks:1, onboarding:1, policy:10, reference-implementations:1,
  runbooks:13, scorecards:1, slos:8, tutorials:1
- Missing-ref samples: microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:37->microservices/api-
  gateway/contracts/openapi/j78-edge-contract-gate-v1.yaml; microservices/api-gateway/IP-journey-j78-edge-contract-
  gate.md:38->microservices/api-gateway/contracts/asyncapi/j78-edge-contract-gate-events-v1.yaml;
  microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:39->microservices/api-
  gateway/contracts/proto/j78-edge-contract-gate-v1.proto
- Risk verdict: severe coherence hotspot. This service was touched by enough distinct wave families that a single ownership audit is required before trusting product claims.

### Hotspot 9: `contact-center`
- Inferred wave tags: 9
- Surface directories / root buckets: 21
- File count under service path: 172
- Journey IP files: 0
- Decision files: 1
- Runbook files: 20
- Missing internal backtick refs: 0
- Wave tags: b2b-ip-deepen-A, capability-tier-surface-present, decisions-surface-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present, rust-src-W1, rust-src-surface-present
- Surfaces: (root):49, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:6, dashboards:10,
  decisions:1, faqs:1, iac:24, migration-playbooks:1, onboarding:1, policies:6, policy:6, reference-
  implementations:1, runbooks:20, scorecards:1, slos:12, src:10, tests:1, tutorials:1
- Missing-ref samples: none from automated backtick-path scan
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 10: `workflow-studio`
- Inferred wave tags: 9
- Surface directories / root buckets: 20
- File count under service path: 226
- Journey IP files: 15
- Decision files: 7
- Runbook files: 12
- Missing internal backtick refs: 154
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W3, runbooks-surface-present
- Surfaces: (root):59, benchmarks:1, capabilities:3, capability-tiers:1, catalog:15, clients:9, contracts:3,
  dashboards:4, decisions:7, faqs:1, iac:16, migration-playbooks:1, onboarding:1, policy:6, reference-
  implementations:1, runbooks:12, scorecards:1, slos:7, templates:77, tutorials:1
- Missing-ref samples: microservices/workflow-studio/IP-027-cedar-grammar-
  impl.md:39->microservices/intelligence/specs/cedar/; microservices/workflow-studio/IP-027-cedar-grammar-
  impl.md:40->microservices/workflow-studio/decisions/ADR-0183.md; microservices/workflow-studio/backfill-
  replay.md:107->microservices/workflow-studio/contracts/asyncapi.yaml
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 11: `cloud-iac`
- Inferred wave tags: 9
- Surface directories / root buckets: 20
- File count under service path: 175
- Journey IP files: 15
- Decision files: 1
- Runbook files: 8
- Missing internal backtick refs: 126
- Wave tags: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- Surfaces: (root):55, benchmarks:1, capabilities:3, capability-tiers:1, catalog:47, contracts:3, dashboards:3,
  decisions:1, faqs:1, iac:20, implementation-plans:3, migration-playbooks:1, onboarding:1, policy:6, reference-
  implementations:1, runbooks:8, scorecards:1, slos:6, tofu:12, tutorials:1
- Missing-ref samples: microservices/cloud-iac/IP-010-rest-surfaces.md:30->microservices/cloud-iac/src/crates/oya-
  cloud-iac-iac-renderer-rest/{Cargo.toml,src/lib.rs,src/routes.rs,src/middleware.rs}; microservices/cloud-
  iac/IP-010-rest-surfaces.md:31->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-validator-
  rest/{Cargo.toml,src/lib.rs,src/routes.rs}; microservices/cloud-iac/IP-010-rest-
  surfaces.md:32->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-applier-
  rest/{Cargo.toml,src/lib.rs,src/routes.rs}
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 12: `developer-sdk`
- Inferred wave tags: 9
- Surface directories / root buckets: 20
- File count under service path: 137
- Journey IP files: 11
- Decision files: 7
- Runbook files: 8
- Missing internal backtick refs: 94
- Wave tags: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- Surfaces: (root):30, benchmarks:1, capabilities:3, capability-tiers:1, catalog:18, contracts:4, dashboards:3,
  decisions:7, faqs:1, iac:18, implementation-plans:15, migration-playbooks:1, onboarding:1, packs:10, policy:4,
  reference-implementations:1, runbooks:8, scorecards:1, slos:9, tutorials:1
- Missing-ref samples: microservices/developer-sdk/cross-microservice-handoffs.md:16->microservices/developer-
  sdk/contracts/proto/developer_sdk.proto; microservices/developer-sdk/cross-microservice-
  handoffs.md:17->microservices/developer-sdk/policies/; microservices/developer-sdk/implementation-
  plans/IP-002-developer-onboarding-kernel-domain.md:38->microservices/developer-sdk/src/crates/oya-developer-sdk-
  developer-onboarding-kernel/src/entities.rs
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 13: `marketplace`
- Inferred wave tags: 9
- Surface directories / root buckets: 20
- File count under service path: 134
- Journey IP files: 15
- Decision files: 1
- Runbook files: 13
- Missing internal backtick refs: 31
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, per-msvc-adrs-D, runbooks-W3, runbooks-surface-present, rust-src-surface-present
- Surfaces: (root):35, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:3, dashboards:5,
  decisions:1, faqs:1, iac:12, ip:25, migration-playbooks:1, onboarding:1, policies:6, reference-implementations:1,
  runbooks:13, scorecards:1, slos:6, src:1, tutorials:1
- Missing-ref samples: microservices/marketplace/IP-journey-j65-order-
  export.md:52->microservices/marketplace/contracts/openapi-j65-order-export.yaml; microservices/marketplace/IP-
  journey-j65-order-export.md:53->microservices/marketplace/contracts/asyncapi-j65-order-export.yaml;
  microservices/marketplace/IP-journey-j65-order-export.md:54->microservices/marketplace/contracts/j65-order-
  export.proto
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 14: `compliance`
- Inferred wave tags: 9
- Surface directories / root buckets: 19
- File count under service path: 201
- Journey IP files: 64
- Decision files: 6
- Runbook files: 13
- Missing internal backtick refs: 227
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-W1, runbooks-surface-present
- Surfaces: (root):109, benchmarks:1, capabilities:5, capability-tiers:1, catalog:11, contracts:4, dashboards:6,
  decisions:6, faqs:1, iac:11, migration-playbooks:1, onboarding:1, packs:5, policy:7, reference-implementations:1,
  runbooks:13, scorecards:5, slos:12, tutorials:1
- Missing-ref samples: microservices/compliance/IP-journey-j83-pack-overlay-
  regulator.md:37->microservices/compliance/contracts/openapi/j83-pack-overlay-regulator-v1.yaml;
  microservices/compliance/IP-journey-j83-pack-overlay-
  regulator.md:38->microservices/compliance/contracts/asyncapi/j83-pack-overlay-regulator-events-v1.yaml;
  microservices/compliance/IP-journey-j83-pack-overlay-
  regulator.md:39->microservices/compliance/contracts/proto/j83-pack-overlay-regulator-v1.proto
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 15: `cloud-secrets`
- Inferred wave tags: 9
- Surface directories / root buckets: 19
- File count under service path: 134
- Journey IP files: 17
- Decision files: 1
- Runbook files: 6
- Missing internal backtick refs: 66
- Wave tags: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- Surfaces: (root):49, benchmarks:1, capabilities:3, capability-tiers:1, catalog:38, contracts:3, dashboards:3,
  decisions:1, faqs:1, iac:10, migration-playbooks:1, migrations:1, onboarding:1, policy:6, reference-
  implementations:1, runbooks:6, scorecards:1, slos:6, tutorials:1
- Missing-ref samples: microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-
  byok.md:37->microservices/cloud-secrets/contracts/openapi/j88-provider-and-encryption-byok-v1.yaml;
  microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md:38->microservices/cloud-
  secrets/contracts/asyncapi/j88-provider-and-encryption-byok-events-v1.yaml; microservices/cloud-secrets/IP-
  journey-j88-provider-and-encryption-byok.md:39->microservices/cloud-secrets/contracts/proto/j88-provider-and-
  encryption-byok-v1.proto
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 16: `workflow-engine`
- Inferred wave tags: 9
- Surface directories / root buckets: 18
- File count under service path: 226
- Journey IP files: 96
- Decision files: 1
- Runbook files: 9
- Missing internal backtick refs: 335
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W3, runbooks-surface-present
- Surfaces: (root):127, benchmarks:1, capabilities:3, capability-tiers:1, catalog:47, contracts:3, dashboards:3,
  decisions:1, faqs:1, iac:12, migration-playbooks:1, onboarding:1, policy:7, reference-implementations:1,
  runbooks:9, scorecards:1, slos:6, tutorials:1
- Missing-ref samples: microservices/workflow-engine/IP-journey-j86-cadence-
  orchestrator.md:37->microservices/workflow-engine/contracts/openapi/j86-cadence-orchestrator-v1.yaml;
  microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:38->microservices/workflow-
  engine/contracts/asyncapi/j86-cadence-orchestrator-events-v1.yaml; microservices/workflow-engine/IP-
  journey-j86-cadence-orchestrator.md:39->microservices/workflow-engine/contracts/proto/j86-cadence-
  orchestrator-v1.proto
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 17: `tenancy`
- Inferred wave tags: 9
- Surface directories / root buckets: 18
- File count under service path: 197
- Journey IP files: 61
- Decision files: 1
- Runbook files: 11
- Missing internal backtick refs: 202
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-W1, runbooks-surface-present
- Surfaces: (root):106, benchmarks:2, capabilities:6, capability-tiers:1, catalog:17, contracts:3, dashboards:6,
  decisions:1, faqs:2, iac:19, migration-playbooks:2, onboarding:2, policy:10, reference-implementations:2,
  runbooks:11, scorecards:1, slos:4, tutorials:2
- Missing-ref samples: microservices/tenancy/IP-journey-j83-tenant-pack-
  scope.md:37->microservices/tenancy/contracts/openapi/j83-tenant-pack-scope-v1.yaml; microservices/tenancy/IP-
  journey-j83-tenant-pack-scope.md:38->microservices/tenancy/contracts/asyncapi/j83-tenant-pack-scope-
  events-v1.yaml; microservices/tenancy/IP-journey-j83-tenant-pack-
  scope.md:39->microservices/tenancy/contracts/proto/j83-tenant-pack-scope-v1.proto
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 18: `connect`
- Inferred wave tags: 9
- Surface directories / root buckets: 18
- File count under service path: 184
- Journey IP files: 36
- Decision files: 1
- Runbook files: 10
- Missing internal backtick refs: 44
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- Surfaces: (root):72, benchmarks:1, capabilities:4, capability-tiers:1, catalog:48, contracts:10, dashboards:4,
  decisions:1, faqs:1, iac:11, migration-playbooks:1, onboarding:1, policy:11, reference-implementations:1,
  runbooks:10, scorecards:1, slos:5, tutorials:1
- Missing-ref samples: microservices/connect/ARCHITECTURE.md:218->microservices/policy-engine/;
  microservices/connect/IP-journey-j62-pharmacy-and-insurance-
  api.md:52->microservices/connect/contracts/openapi-j62-pharmacy-and-insurance-api.yaml; microservices/connect/IP-
  journey-j62-pharmacy-and-insurance-api.md:53->microservices/connect/contracts/asyncapi-j62-pharmacy-and-
  insurance-api.yaml
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 19: `notes`
- Inferred wave tags: 9
- Surface directories / root buckets: 18
- File count under service path: 160
- Journey IP files: 27
- Decision files: 8
- Runbook files: 11
- Missing internal backtick refs: 68
- Wave tags: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- Surfaces: (root):63, benchmarks:1, capabilities:3, capability-tiers:1, catalog:19, contracts:3, dashboards:5,
  decisions:8, faqs:1, iac:18, migration-playbooks:1, onboarding:1, policy:12, reference-implementations:1,
  runbooks:11, scorecards:1, slos:10, tutorials:1
- Missing-ref samples: microservices/notes/IP-journey-j61-soap-
  note.md:52->microservices/notes/contracts/openapi-j61-soap-note.yaml; microservices/notes/IP-journey-j61-soap-
  note.md:53->microservices/notes/contracts/asyncapi-j61-soap-note.yaml; microservices/notes/IP-journey-j61-soap-
  note.md:54->microservices/notes/contracts/j61-soap-note.proto
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

### Hotspot 20: `application`
- Inferred wave tags: 9
- Surface directories / root buckets: 18
- File count under service path: 135
- Journey IP files: 11
- Decision files: 1
- Runbook files: 6
- Missing internal backtick refs: 49
- Wave tags: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- Surfaces: (root):44, benchmarks:1, capabilities:4, capability-tiers:1, catalog:43, contracts:4, dashboards:3,
  decisions:1, faqs:1, iac:10, migration-playbooks:1, onboarding:1, policy:7, reference-implementations:1,
  runbooks:6, scorecards:1, slos:5, tutorials:1
- Missing-ref samples: microservices/application/IP-002-shell-routing-
  domain.md:27->microservices/application/src/crates/oya-application-shell-routing-domain/Cargo.toml;
  microservices/application/IP-009-auth-gateway-adapters-oidc-saml.md:30->microservices/application/src/crates/oya-
  application-auth-gateway-adapter-oidc/{Cargo.toml,src/{lib,jwks,verify}.rs};
  microservices/application/IP-009-auth-gateway-adapters-oidc-saml.md:31->microservices/application/src/crates/oya-
  application-auth-gateway-adapter-saml/{Cargo.toml,src/{lib,verify,metadata}.rs}
- Risk verdict: high coherence hotspot. The service has enough distinct surface writers to require cross-artifact validation.

## Sampled internal contradictions / artifact gaps

The audit sampled high-touch services. The strongest recurring issue is not only textual contradiction; it is a doc-suite or journey surface citing concrete files, metrics, contracts, tests, or dashboards that do not exist under the same service. That is the practical effect of surface waves landing independently.

### Sample: `intelligence`
11 wave tags, 20 surfaces, 176 files, 38 journey IPs, 95 missing references. Capability-tier matrix says
routing latency p99 <=100 ms while the OpenSLO dispatch API latency file is p99 <250 ms. This may be metric-
scope divergence, but it is unresolved within one service.
Citation: `microservices/intelligence/capability-tiers/tier-matrix.md:44; microservices/intelligence/slos/dispatch-api-latency.openslo.yaml:5; /private/tmp/lane2_matrix.tsv`

### Sample: `audit-chain`
10 wave tags, 21 surfaces, 251 files, 81 journey IPs, 217 missing references. PRD declares emit availability
99.99% and verify/query 99.95%; tier matrix Bronze chain availability is 99.9%. Without a tier binding, the
service has flat PRD SLOs and tiered SLOs in tension.
Citation: `microservices/audit-chain/PRD.md:86-91; microservices/audit-chain/capability-tiers/tier-matrix.md:36-40; /private/tmp/lane2_matrix.tsv`

### Sample: `workflow-engine`
9 wave tags, 18 surfaces, 226 files, 96 journey IPs, 335 missing references. Journey IPs cite per-journey
OpenAPI/AsyncAPI/proto/policy/runbook/test files that do not exist under the same service path.
Citation: `microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:37-42; /private/tmp/lane2_matrix.tsv`

### Sample: `payments`
11 wave tags, 20 surfaces, 202 files, 70 journey IPs, 123 missing references. IP-journey files cite generated
contract/policy/test/dashboard files absent under payments; ADR claims route budgets while broader doc-suite
surfaces also add cross-microservice render handoffs.
Citation: `microservices/payments/IP-journey-j65-receipt-export.md:52-57; microservices/payments/decisions/ADR-PAY-001-multi-psp-routing-with-failover-cascade.md:190-193; /private/tmp/lane2_matrix.tsv`

### Sample: `finops-portal`
10 wave tags, 19 surfaces, 161 files, 38 missing references. Runbooks repeatedly point incident-specific
metrics at tenant-invoice-render-latency SLO, while per-journey IPs cite missing finance-risk-console
contract/policy/test artifacts.
Citation: `microservices/finops-portal/IP-journey-j82-finance-risk-console.md:37-42; microservices/finops-portal/runbooks/credit-application-reconciliation.md:42; /private/tmp/lane2_matrix.tsv`

### Sample: `ontology`
10 wave tags, 18 surfaces, 154 files, 30 journey IPs, 104 missing references. Journey IPs cite non-existent
typed-record-writer contract/policy/test/runbook files; tier matrix and runbooks reference multiple SLO
families that need reconciliation.
Citation: `microservices/ontology/IP-journey-j84-typed-record-writer.md:37-42; microservices/ontology/capability-tiers/tier-matrix.md:38-42; /private/tmp/lane2_matrix.tsv`

### Sample: `marketplace`
9 wave tags, 20 surfaces, 134 files, 15 journey IPs, 31 missing references. Marketplace was touched by doc-
suite W5, ADR batch D, runbooks W3, journey waves, and source surface; journey IPs cite absent order-export
contract/policy/test/dashboard files.
Citation: `microservices/marketplace/IP-journey-j65-order-export.md:52-57; /private/tmp/lane2_matrix.tsv`

### Sample: `workplace-integration`
10 wave tags, 20 surfaces, 135 files, 16 journey IPs, 31 missing references. Workplace integration has PRD-
level SLO promises and journey IP references to missing e-signature contract/policy/test/dashboard files after
doc-suite, ADR, runbook, journey, and source-surface passes.
Citation: `microservices/workplace-integration/PRD.md:69-85; microservices/workplace-integration/IP-journey-j54-e-signature.md:52-57; /private/tmp/lane2_matrix.tsv`

## Highest-risk false-reference clusters

1. `payments` — 123 missing refs; samples: microservices/payments/IP-journey-j65-receipt-export.md:52->microservices/payments/contracts/openapi-j65-receipt-export.yaml; microservices/payments/IP-journey-j65-receipt-export.md:53->microservices/payments/contracts/asyncapi-j65-receipt-export.yaml; microservices/payments/IP-journey-j65-receipt-export.md:54->microservices/payments/contracts/j65-receipt-export.proto
2. `intelligence` — 95 missing refs; samples: microservices/intelligence/IP-002-domain-layer-secret-reference.md:25->microservices/intelligence/crates/; microservices/intelligence/IP-002-domain-layer-secret-reference.md:32->microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/Cargo.toml; microservices/intelligence/IP-002-domain-layer-secret-reference.md:33->microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/src/lib.rs
3. `audit-chain` — 217 missing refs; samples: microservices/audit-chain/IP-journey-j55-dispute-seal.md:52->microservices/audit-chain/contracts/openapi-j55-dispute-seal.yaml; microservices/audit-chain/IP-journey-j55-dispute-seal.md:53->microservices/audit-chain/contracts/asyncapi-j55-dispute-seal.yaml; microservices/audit-chain/IP-journey-j55-dispute-seal.md:54->microservices/audit-chain/contracts/j55-dispute-seal.proto
4. `workplace-integration` — 31 missing refs; samples: microservices/workplace-integration/IP-journey-j54-e-signature.md:52->microservices/workplace-integration/contracts/openapi-j54-e-signature.yaml; microservices/workplace-integration/IP-journey-j54-e-signature.md:53->microservices/workplace-integration/contracts/asyncapi-j54-e-signature.yaml; microservices/workplace-integration/IP-journey-j54-e-signature.md:54->microservices/workplace-integration/contracts/j54-e-signature.proto
5. `finops-portal` — 38 missing refs; samples: microservices/finops-portal/IP-journey-j82-finance-risk-console.md:37->microservices/finops-portal/contracts/openapi/j82-finance-risk-console-v1.yaml; microservices/finops-portal/IP-journey-j82-finance-risk-console.md:38->microservices/finops-portal/contracts/asyncapi/j82-finance-risk-console-events-v1.yaml; microservices/finops-portal/IP-journey-j82-finance-risk-console.md:39->microservices/finops-portal/contracts/proto/j82-finance-risk-console-v1.proto
6. `comms-email` — 5 missing refs; samples: microservices/comms-email/IP-006-mjml-template-renderer.md:36->microservices/comms-email/templates/canonical-base/{template_id}/{locale}.mjml; microservices/comms-email/IP-006-mjml-template-renderer.md:37->microservices/comms-email/templates/packs/{pack}/{template_id}/{locale}.mjml; microservices/comms-email/IP-015-in-house-relay-roadmap-phase-2.md:71->microservices/comms-email/iac/helm/oya-comms-email-server/
7. `ontology` — 104 missing refs; samples: microservices/ontology/IP-journey-j84-typed-record-writer.md:37->microservices/ontology/contracts/openapi/j84-typed-record-writer-v1.yaml; microservices/ontology/IP-journey-j84-typed-record-writer.md:38->microservices/ontology/contracts/asyncapi/j84-typed-record-writer-events-v1.yaml; microservices/ontology/IP-journey-j84-typed-record-writer.md:39->microservices/ontology/contracts/proto/j84-typed-record-writer-v1.proto
8. `api-gateway` — 20 missing refs; samples: microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:37->microservices/api-gateway/contracts/openapi/j78-edge-contract-gate-v1.yaml; microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:38->microservices/api-gateway/contracts/asyncapi/j78-edge-contract-gate-events-v1.yaml; microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:39->microservices/api-gateway/contracts/proto/j78-edge-contract-gate-v1.proto
9. `workflow-studio` — 154 missing refs; samples: microservices/workflow-studio/IP-027-cedar-grammar-impl.md:39->microservices/intelligence/specs/cedar/; microservices/workflow-studio/IP-027-cedar-grammar-impl.md:40->microservices/workflow-studio/decisions/ADR-0183.md; microservices/workflow-studio/backfill-replay.md:107->microservices/workflow-studio/contracts/asyncapi.yaml
10. `cloud-iac` — 126 missing refs; samples: microservices/cloud-iac/IP-010-rest-surfaces.md:30->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-rest/{Cargo.toml,src/lib.rs,src/routes.rs,src/middleware.rs}; microservices/cloud-iac/IP-010-rest-surfaces.md:31->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-validator-rest/{Cargo.toml,src/lib.rs,src/routes.rs}; microservices/cloud-iac/IP-010-rest-surfaces.md:32->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-applier-rest/{Cargo.toml,src/lib.rs,src/routes.rs}
11. `developer-sdk` — 94 missing refs; samples: microservices/developer-sdk/cross-microservice-handoffs.md:16->microservices/developer-sdk/contracts/proto/developer_sdk.proto; microservices/developer-sdk/cross-microservice-handoffs.md:17->microservices/developer-sdk/policies/; microservices/developer-sdk/implementation-plans/IP-002-developer-onboarding-kernel-domain.md:38->microservices/developer-sdk/src/crates/oya-developer-sdk-developer-onboarding-kernel/src/entities.rs
12. `marketplace` — 31 missing refs; samples: microservices/marketplace/IP-journey-j65-order-export.md:52->microservices/marketplace/contracts/openapi-j65-order-export.yaml; microservices/marketplace/IP-journey-j65-order-export.md:53->microservices/marketplace/contracts/asyncapi-j65-order-export.yaml; microservices/marketplace/IP-journey-j65-order-export.md:54->microservices/marketplace/contracts/j65-order-export.proto
13. `compliance` — 227 missing refs; samples: microservices/compliance/IP-journey-j83-pack-overlay-regulator.md:37->microservices/compliance/contracts/openapi/j83-pack-overlay-regulator-v1.yaml; microservices/compliance/IP-journey-j83-pack-overlay-regulator.md:38->microservices/compliance/contracts/asyncapi/j83-pack-overlay-regulator-events-v1.yaml; microservices/compliance/IP-journey-j83-pack-overlay-regulator.md:39->microservices/compliance/contracts/proto/j83-pack-overlay-regulator-v1.proto
14. `cloud-secrets` — 66 missing refs; samples: microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md:37->microservices/cloud-secrets/contracts/openapi/j88-provider-and-encryption-byok-v1.yaml; microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md:38->microservices/cloud-secrets/contracts/asyncapi/j88-provider-and-encryption-byok-events-v1.yaml; microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md:39->microservices/cloud-secrets/contracts/proto/j88-provider-and-encryption-byok-v1.proto
15. `workflow-engine` — 335 missing refs; samples: microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:37->microservices/workflow-engine/contracts/openapi/j86-cadence-orchestrator-v1.yaml; microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:38->microservices/workflow-engine/contracts/asyncapi/j86-cadence-orchestrator-events-v1.yaml; microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:39->microservices/workflow-engine/contracts/proto/j86-cadence-orchestrator-v1.proto
16. `tenancy` — 202 missing refs; samples: microservices/tenancy/IP-journey-j83-tenant-pack-scope.md:37->microservices/tenancy/contracts/openapi/j83-tenant-pack-scope-v1.yaml; microservices/tenancy/IP-journey-j83-tenant-pack-scope.md:38->microservices/tenancy/contracts/asyncapi/j83-tenant-pack-scope-events-v1.yaml; microservices/tenancy/IP-journey-j83-tenant-pack-scope.md:39->microservices/tenancy/contracts/proto/j83-tenant-pack-scope-v1.proto
17. `connect` — 44 missing refs; samples: microservices/connect/ARCHITECTURE.md:218->microservices/policy-engine/; microservices/connect/IP-journey-j62-pharmacy-and-insurance-api.md:52->microservices/connect/contracts/openapi-j62-pharmacy-and-insurance-api.yaml; microservices/connect/IP-journey-j62-pharmacy-and-insurance-api.md:53->microservices/connect/contracts/asyncapi-j62-pharmacy-and-insurance-api.yaml
18. `notes` — 68 missing refs; samples: microservices/notes/IP-journey-j61-soap-note.md:52->microservices/notes/contracts/openapi-j61-soap-note.yaml; microservices/notes/IP-journey-j61-soap-note.md:53->microservices/notes/contracts/asyncapi-j61-soap-note.yaml; microservices/notes/IP-journey-j61-soap-note.md:54->microservices/notes/contracts/j61-soap-note.proto
19. `application` — 49 missing refs; samples: microservices/application/IP-002-shell-routing-domain.md:27->microservices/application/src/crates/oya-application-shell-routing-domain/Cargo.toml; microservices/application/IP-009-auth-gateway-adapters-oidc-saml.md:30->microservices/application/src/crates/oya-application-auth-gateway-adapter-oidc/{Cargo.toml,src/{lib,jwks,verify}.rs}; microservices/application/IP-009-auth-gateway-adapters-oidc-saml.md:31->microservices/application/src/crates/oya-application-auth-gateway-adapter-saml/{Cargo.toml,src/{lib,verify,metadata}.rs}
20. `feature-flags` — 8 missing refs; samples: microservices/feature-flags/ARCHITECTURE.md:269->microservices/policy-engine/; microservices/feature-flags/PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md:163->microservices/policy-engine/; microservices/feature-flags/tutorials/cohort-rollout-with-analytics.md:20->microservices/analytics/audiences/
21. `itsm` — 1 missing refs; samples: microservices/itsm/IP-002-cedar-default-deny.md:104->microservices/itsm/policies/local-service-catalog-publish-approval.cedar
22. `incident-management` — 1 missing refs; samples: microservices/incident-management/onboarding/incident-commander-first-week.md:38->microservices/incident-management/templates/runbook-template.md
23. `foundry` — 438 missing refs; samples: microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md:24->microservices/intelligence/src/crates/oya-intelligence-guardrails-prompt-classifier-kernel/; microservices/intelligence/IP-076-providers-router-kernel.md:24->microservices/intelligence/src/crates/oya-intelligence-providers-router-kernel/; microservices/intelligence/IP-076-providers-router-kernel.md:30->microservices/intelligence/src/crates/oya-intelligence-providers-router-kernel/Cargo.toml
24. `identity` — 276 missing refs; samples: microservices/identity/IP-journey-j59-sso-disable.md:52->microservices/identity/contracts/openapi-j59-sso-disable.yaml; microservices/identity/IP-journey-j59-sso-disable.md:53->microservices/identity/contracts/asyncapi-j59-sso-disable.yaml; microservices/identity/IP-journey-j59-sso-disable.md:54->microservices/identity/contracts/j59-sso-disable.proto
25. `mail` — 329 missing refs; samples: microservices/mail/IP-009-search-index.md:30->microservices/mail/src/crates/oya-mail-search-index-kernel/; microservices/mail/IP-009-search-index.md:31->microservices/mail/src/crates/oya-mail-search-index-domain/; microservices/mail/IP-009-search-index.md:32->microservices/mail/src/crates/oya-mail-search-index-usecase/
26. `plugin-app-store` — 130 missing refs; samples: microservices/plugin-app-store/IP-journey-j75-quarantine.md:52->microservices/plugin-app-store/contracts/openapi-j75-quarantine.yaml; microservices/plugin-app-store/IP-journey-j75-quarantine.md:53->microservices/plugin-app-store/contracts/asyncapi-j75-quarantine.yaml; microservices/plugin-app-store/IP-journey-j75-quarantine.md:54->microservices/plugin-app-store/contracts/j75-quarantine.proto
27. `observability` — 225 missing refs; samples: microservices/observability/IP-022-otel-to-clickhouse-bridge.md:20->microservices/observability/iac/helm/otel-collector-gateway/values.yaml; microservices/observability/IP-022-otel-to-clickhouse-bridge.md:21->microservices/observability/iac/helm/otel-collector-gateway/templates/collector-config.yaml.tpl; microservices/observability/IP-022-otel-to-clickhouse-bridge.md:22->microservices/observability/contracts/clickhouse-tables/metrics-table.sql
28. `governance` — 111 missing refs; samples: microservices/governance/IP-013-aggregation-index-generation-lane.md:34->microservices/governance/catalog/oya-check-aggregation-index-generation.yaml; microservices/governance/IP-001-scaffold-umbrella-bcs.md:24->microservices/governance/src/crates/; microservices/governance/IP-journey-j81-policy-and-attestation.md:37->microservices/governance/contracts/openapi/j81-policy-and-attestation-v1.yaml
29. `analytics` — 23 missing refs; samples: microservices/analytics/PRD.md:54->microservices/analytics/iac/helm/clickhouse/; microservices/analytics/multi-region.md:32->microservices/analytics/runbooks/incident-response.md; microservices/analytics/compliance.md:10->microservices/analytics/policy/gdpr.cedar
30. `recordings` — 10 missing refs; samples: microservices/recordings/PRD.md:378->microservices/recordings/runbooks/error-budget-policy.md; microservices/recordings/IP-001-iac-bootstrap.md:42->microservices/recordings/iac/helm/recordings/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml; microservices/recordings/IP-001-iac-bootstrap.md:44->microservices/recordings/iac/kustomize/overlays/pack-{kr,eu,us-healthcare,us-financial}/kustomization.yaml

## Evidence strength ranking

| Strength | Finding | Why |
|---|---|---|
| Strong | ADR-0321 duplicate sections | Live file line citations prove exact duplicate vendors and out-of-order duplicate blocks. |
| Strong | Parallel dispatch overlap | Transcript lines 13047 and 13215 prove overlapping same-file append instructions with overlapping vendors. |
| Strong | Surface-wave pattern | Per-msvc ADR and runbook logs explicitly enumerate batch targets by surface, not service owner. |
| Strong | High-touch hotspot matrix | Filesystem scan shows many services with 9-11 wave tags and broad surface counts. |
| Strong | False-reference density | Backtick-path scan shows hundreds of unresolved intra-service references in top hotspots. |
| Moderate | Claim-ratchet failure mode | We can prove collision happened despite claim protocol existing; we cannot prove exact ledger rows during the race. |
| Moderate | Internal SLO/tier contradictions | Samples show threshold divergence and missing references; full semantic adjudication needs service-owner read. |
| Moderate | Manual avoid-list coordination | W8 proves collision avoidance was done through prompt constraints; exact collisions outside ADR-0321 need further ledger/provenance proof. |
| Weak | Exact per-agent authorship for every file | Current untracked/dirty tree and missing ledger snapshots prevent exact authorship attribution across all services. |
| Weak | All 79 services claim | Live filesystem shows 78 service dirs; transcript says 79. The missing or folded service is an unknown rather than confirmed by this audit. |

## Critical Unknown

The critical unknown is the exact runtime ownership ledger at the moment of the ADR-0321 race and the exact per-agent write provenance for each microservice artifact. Specifically: did both ADR-0321 author lanes hold valid claims on the same file, did one bypass Oya VCS, or did the claim model permit same-file section ownership without detecting vendor overlap? Current evidence proves the collision outcome and dispatch overlap, but not the ledger state that allowed it.

## Recommended Discriminating Probe

- Run a provenance reconstruction probe over the task outputs and filesystem snapshots, not another content review.
- Inputs: transcript lines 13047, 13215, 13623-13628; task output `/private/tmp/claude-501/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d/tasks/a9e9a85b6decbfc33.output`; `/private/tmp/codex-out-adr-0321-finish-d136-d148.log`; shell history if available; Oya VCS status/ledger artifacts if persisted outside `.oya`.
- Algorithm: parse each agent output for the exact section headers it wrote, compare with ADR-0321 line offsets, reconstruct append order, then map each appended header to claimed scope and claimed vendor list.
- Discriminator: if both agents held accepted file-level claims concurrently, the claim ratchet has a locking semantics bug. If only one held a claim, the other bypassed the ratchet. If both held broad non-conflicting claims while sharing the same file, the claim model is too coarse for section/vendor-level ownership.
- Follow-on: repeat the same probe against three high-touch service paths (`payments`, `audit-chain`, `workflow-engine`) by mapping file mtimes, task logs, and missing-ref clusters to wave families. This will separate intended future-stub references from true surface-wave drift.

## Convergence/Separation Notes vs Lanes 1 + 3

- Lane 2 converges with Lane 1 on ADR-0321: the wrong candidate list supplied out-of-scope vendors, while concurrent same-file appenders made the duplicates and ordering race visible in the corpus.
- Lane 2 converges with Lane 3 on completion risk: surface waves reported done while later verification found thin ADRs, incomplete journey batches, and false references. Verification did not enforce service coherence before accepting wave outputs.
- Lane 2 is distinct from Lane 1: coordination explains why two agents created duplicate sections and why no one owned an entire µservice after many surfaces landed. Brief scope alone does not explain file races.
- Lane 2 is distinct from Lane 3: verification explains why drift persisted. It does not explain how the same file could receive overlapping duplicate authoring or why services were split by surface ownership.
- The combined causal chain is: ambiguous or overbroad brief -> many parallel surface writers -> claim/ownership barrier not discriminating enough -> shallow completion checks -> corpus drift discovered late by user review.

## Recommended remediation shape

- **R-01:** Make µservice ownership the default unit. One owner owns a service path end-to-end during a coherence-sensitive pass: PRD, ADRs, IPs, contracts, policy, SLOs, runbooks, capability tiers, dashboards, migration playbooks, and cross-service handoffs.
- **R-02:** For shared global files such as ADR-0321, introduce section/vendor claims. File-level append claims are insufficient when multiple agents can write different sections that duplicate semantic entities.
- **R-03:** Require preflight read receipts. A writer must emit the exact prior headings or service-surface inventory it read before modifying a shared file or a service path.
- **R-04:** Replace manual avoid lists with machine-readable active-scope admission. Prompt text like “avoid W7 and ERP services” should be generated from the claim ledger, not hand-maintained in the brief.
- **R-05:** Gate `done` on unresolved-reference scan for service docs. Missing backtick paths should either be created, marked as future planned artifacts in a manifest, or removed before service coherence is accepted.
- **R-06:** For doc-suite waves, use service-local manifests that list existing contracts/SLOs/policies. Authors cannot cite a feature unless the manifest says the surface exists or the author creates the referenced artifact in the same claim.
- **R-07:** For ADR-0321, de-dupe, reorder, and then run an invariant checker: one D-section per vendor identity, monotonic numeric order, and each vendor classified as B2B SaaS displaced vs composed-with infrastructure.
- **R-08:** Persist claim ledger snapshots into `.omc/evidence/` for high-concurrency waves. The absence of a replayable ledger is itself a debugging liability.

## Per-service detailed matrix appendix

Each service entry below is deliberately line-oriented so another agent can grep by service name and quickly see ownership risk inputs.

### Service 01: `analytics`
- wave_count: 8
- surface_count: 19
- files: 138
- journey_ips: 10
- decision_files: 5
- runbook_files: 8
- missing_refs: 23
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- surfaces: (root):25, benchmarks:1, capabilities:3, capability-tiers:2, catalog:18, contracts:4, dashboards:3, decisions:5, faqs:1, iac:29, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:8, scorecards:4, slos:10, specs:15, tutorials:1
- missing_ref_sample_1: microservices/analytics/PRD.md:54->microservices/analytics/iac/helm/clickhouse/
- missing_ref_sample_2: microservices/analytics/multi-region.md:32->microservices/analytics/runbooks/incident-response.md
- missing_ref_sample_3: microservices/analytics/compliance.md:10->microservices/analytics/policy/gdpr.cedar
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 02: `api-gateway`
- wave_count: 10
- surface_count: 18
- files: 136
- journey_ips: 14
- decision_files: 1
- runbook_files: 13
- missing_refs: 20
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- surfaces: (root):53, benchmarks:1, capabilities:4, capability-tiers:1, catalog:14, contracts:4, dashboards:8, decisions:1, faqs:1, iac:13, migration-playbooks:1, onboarding:1, policy:10, reference-implementations:1, runbooks:13, scorecards:1, slos:8, tutorials:1
- missing_ref_sample_1: microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:37->microservices/api-gateway/contracts/openapi/j78-edge-contract-gate-v1.yaml
- missing_ref_sample_2: microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:38->microservices/api-gateway/contracts/asyncapi/j78-edge-contract-gate-events-v1.yaml
- missing_ref_sample_3: microservices/api-gateway/IP-journey-j78-edge-contract-gate.md:39->microservices/api-gateway/contracts/proto/j78-edge-contract-gate-v1.proto
- sampled: no
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 03: `application`
- wave_count: 9
- surface_count: 18
- files: 135
- journey_ips: 11
- decision_files: 1
- runbook_files: 6
- missing_refs: 49
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- surfaces: (root):44, benchmarks:1, capabilities:4, capability-tiers:1, catalog:43, contracts:4, dashboards:3, decisions:1, faqs:1, iac:10, migration-playbooks:1, onboarding:1, policy:7, reference-implementations:1, runbooks:6, scorecards:1, slos:5, tutorials:1
- missing_ref_sample_1: microservices/application/IP-002-shell-routing-domain.md:27->microservices/application/src/crates/oya-application-shell-routing-domain/Cargo.toml
- missing_ref_sample_2: microservices/application/IP-009-auth-gateway-adapters-oidc-saml.md:30->microservices/application/src/crates/oya-application-auth-gateway-adapter-oidc/{Cargo.toml,src/{lib,jwks,verify}.rs}
- missing_ref_sample_3: microservices/application/IP-009-auth-gateway-adapters-oidc-saml.md:31->microservices/application/src/crates/oya-application-auth-gateway-adapter-saml/{Cargo.toml,src/{lib,verify,metadata}.rs}
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 04: `audit-chain`
- wave_count: 10
- surface_count: 21
- files: 251
- journey_ips: 81
- decision_files: 1
- runbook_files: 9
- missing_refs: 217
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- surfaces: (root):115, benchmarks:1, capabilities:3, capability-tiers:1, catalog:39, contracts:3, dashboards:3, decisions:1, faqs:1, iac:10, migration-playbooks:1, onboarding:1, packs:5, policy:44, reference-implementations:1, runbooks:9, scorecards:1, security:1, slos:7, test-plans:3, tutorials:1
- missing_ref_sample_1: microservices/audit-chain/IP-journey-j55-dispute-seal.md:52->microservices/audit-chain/contracts/openapi-j55-dispute-seal.yaml
- missing_ref_sample_2: microservices/audit-chain/IP-journey-j55-dispute-seal.md:53->microservices/audit-chain/contracts/asyncapi-j55-dispute-seal.yaml
- missing_ref_sample_3: microservices/audit-chain/IP-journey-j55-dispute-seal.md:54->microservices/audit-chain/contracts/j55-dispute-seal.proto
- sampled: yes
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 05: `calendar`
- wave_count: 7
- surface_count: 19
- files: 139
- journey_ips: 19
- decision_files: 6
- runbook_files: 12
- missing_refs: 114
- waves: capability-tier-surface-present, decisions-surface-present, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-surface-present
- surfaces: (root):52, benchmarks:1, capabilities:3, capability-tiers:1, catalog:17, contracts:3, dashboards:3, decisions:6, faqs:1, iac:15, migration-playbooks:1, onboarding:1, packs:5, policy:6, reference-implementations:1, runbooks:12, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/calendar/PRD.md:136->microservices/calendar/specs/naming-justification.md
- missing_ref_sample_2: microservices/calendar/IP-001-iac-bootstrap.md:53->microservices/calendar/iac/kustomize/overlays/pack-eu/kustomization.yaml
- missing_ref_sample_3: microservices/calendar/IP-001-iac-bootstrap.md:54->microservices/calendar/iac/kustomize/overlays/pack-us/kustomization.yaml
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 06: `cell`
- wave_count: 8
- surface_count: 18
- files: 147
- journey_ips: 26
- decision_files: 1
- runbook_files: 6
- missing_refs: 90
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present
- surfaces: (root):58, benchmarks:1, capabilities:3, capability-tiers:1, catalog:42, contracts:3, dashboards:3, decisions:1, faqs:1, iac:11, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:6, scorecards:1, slos:6, tutorials:1
- missing_ref_sample_1: microservices/cell/IP-009-tenant-assignment-stack.md:24->microservices/cell/src/crates/oya-cell-tenant-assignment-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}/
- missing_ref_sample_2: microservices/cell/PRD.md:246->microservices/cell/runbooks/oncall-rotation.md
- missing_ref_sample_3: microservices/cell/IP-journey-j83-sovereign-cell-placement.md:37->microservices/cell/contracts/openapi/j83-sovereign-cell-placement-v1.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 07: `cloud-billing`
- wave_count: 5
- surface_count: 8
- files: 10
- journey_ips: 0
- decision_files: 0
- runbook_files: 3
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present, runbooks-W4, runbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, runbooks:3, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 08: `cloud-billing-tax`
- wave_count: 3
- surface_count: 7
- files: 7
- journey_ips: 0
- decision_files: 0
- runbook_files: 0
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 09: `cloud-data`
- wave_count: 3
- surface_count: 7
- files: 7
- journey_ips: 0
- decision_files: 0
- runbook_files: 0
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 10: `cloud-iac`
- wave_count: 9
- surface_count: 20
- files: 175
- journey_ips: 15
- decision_files: 1
- runbook_files: 8
- missing_refs: 126
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- surfaces: (root):55, benchmarks:1, capabilities:3, capability-tiers:1, catalog:47, contracts:3, dashboards:3, decisions:1, faqs:1, iac:20, implementation-plans:3, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:8, scorecards:1, slos:6, tofu:12, tutorials:1
- missing_ref_sample_1: microservices/cloud-iac/IP-010-rest-surfaces.md:30->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-rest/{Cargo.toml,src/lib.rs,src/routes.rs,src/middleware.rs}
- missing_ref_sample_2: microservices/cloud-iac/IP-010-rest-surfaces.md:31->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-validator-rest/{Cargo.toml,src/lib.rs,src/routes.rs}
- missing_ref_sample_3: microservices/cloud-iac/IP-010-rest-surfaces.md:32->microservices/cloud-iac/src/crates/oya-cloud-iac-iac-applier-rest/{Cargo.toml,src/lib.rs,src/routes.rs}
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 11: `cloud-iam`
- wave_count: 5
- surface_count: 8
- files: 10
- journey_ips: 0
- decision_files: 0
- runbook_files: 3
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present, runbooks-W4, runbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, runbooks:3, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 12: `cloud-k8s`
- wave_count: 7
- surface_count: 19
- files: 123
- journey_ips: 12
- decision_files: 1
- runbook_files: 9
- missing_refs: 54
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present
- surfaces: (root):46, benchmarks:1, capabilities:3, capability-tiers:2, catalog:13, contracts:3, dashboards:3, decisions:1, faqs:1, iac:23, implementation-plans:1, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:9, scorecards:1, slos:6, tutorials:1
- missing_ref_sample_1: microservices/cloud-k8s/IP-009-service-mesh-control-plane-istio.md:31->microservices/cloud-k8s/catalog/oya-cloud-k8s-service-mesh-control-plane-{kernel,usecase,adapter-istio}.yaml
- missing_ref_sample_2: microservices/cloud-k8s/IP-001-layer-a-iac-kubeadm-containerd-istio-envoy.md:40->microservices/cloud-k8s/iac/helm/csi-block-volume/{Chart.yaml,values.yaml}
- missing_ref_sample_3: microservices/cloud-k8s/IP-001-layer-a-iac-kubeadm-containerd-istio-envoy.md:41->microservices/cloud-k8s/iac/helm/csi-object/{Chart.yaml,values.yaml}
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 13: `cloud-kms`
- wave_count: 5
- surface_count: 8
- files: 10
- journey_ips: 0
- decision_files: 0
- runbook_files: 3
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present, runbooks-W4, runbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, runbooks:3, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 14: `cloud-network`
- wave_count: 5
- surface_count: 8
- files: 10
- journey_ips: 0
- decision_files: 0
- runbook_files: 3
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present, runbooks-W4, runbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, runbooks:3, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 15: `cloud-network-dns`
- wave_count: 3
- surface_count: 7
- files: 7
- journey_ips: 0
- decision_files: 0
- runbook_files: 0
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 16: `cloud-secrets`
- wave_count: 9
- surface_count: 19
- files: 134
- journey_ips: 17
- decision_files: 1
- runbook_files: 6
- missing_refs: 66
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- surfaces: (root):49, benchmarks:1, capabilities:3, capability-tiers:1, catalog:38, contracts:3, dashboards:3, decisions:1, faqs:1, iac:10, migration-playbooks:1, migrations:1, onboarding:1, policy:6, reference-implementations:1, runbooks:6, scorecards:1, slos:6, tutorials:1
- missing_ref_sample_1: microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md:37->microservices/cloud-secrets/contracts/openapi/j88-provider-and-encryption-byok-v1.yaml
- missing_ref_sample_2: microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md:38->microservices/cloud-secrets/contracts/asyncapi/j88-provider-and-encryption-byok-events-v1.yaml
- missing_ref_sample_3: microservices/cloud-secrets/IP-journey-j88-provider-and-encryption-byok.md:39->microservices/cloud-secrets/contracts/proto/j88-provider-and-encryption-byok-v1.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 17: `cloud-storage`
- wave_count: 3
- surface_count: 7
- files: 7
- journey_ips: 0
- decision_files: 0
- runbook_files: 0
- missing_refs: 0
- waves: capability-tier-surface-present, doc-suite-W9-cloud-family, migration-playbooks-surface-present
- surfaces: benchmarks:1, capability-tiers:1, faqs:1, migration-playbooks:1, onboarding:1, reference-implementations:1, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 18: `comms-email`
- wave_count: 10
- surface_count: 19
- files: 136
- journey_ips: 10
- decision_files: 6
- runbook_files: 10
- missing_refs: 5
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present, rust-src-surface-present
- surfaces: (root):54, benchmarks:1, capabilities:6, capability-tiers:1, catalog:1, contracts:3, dashboards:5, decisions:6, faqs:1, iac:17, migration-playbooks:1, onboarding:1, policy:12, reference-implementations:1, runbooks:10, scorecards:5, slos:9, src:1, tutorials:1
- missing_ref_sample_1: microservices/comms-email/IP-006-mjml-template-renderer.md:36->microservices/comms-email/templates/canonical-base/{template_id}/{locale}.mjml
- missing_ref_sample_2: microservices/comms-email/IP-006-mjml-template-renderer.md:37->microservices/comms-email/templates/packs/{pack}/{template_id}/{locale}.mjml
- missing_ref_sample_3: microservices/comms-email/IP-015-in-house-relay-roadmap-phase-2.md:71->microservices/comms-email/iac/helm/oya-comms-email-server/
- sampled: no
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 19: `community`
- wave_count: 8
- surface_count: 18
- files: 202
- journey_ips: 50
- decision_files: 6
- runbook_files: 9
- missing_refs: 85
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-W3, runbooks-surface-present
- surfaces: (root):82, benchmarks:1, capabilities:10, capability-tiers:1, catalog:52, contracts:3, dashboards:3, decisions:6, faqs:1, iac:12, migration-playbooks:1, onboarding:1, policy:10, reference-implementations:1, runbooks:9, scorecards:1, slos:7, tutorials:1
- missing_ref_sample_1: microservices/community/IP-journey-j76-community-surface.md:37->microservices/community/contracts/openapi/j76-community-surface-v1.yaml
- missing_ref_sample_2: microservices/community/IP-journey-j76-community-surface.md:38->microservices/community/contracts/asyncapi/j76-community-surface-events-v1.yaml
- missing_ref_sample_3: microservices/community/IP-journey-j76-community-surface.md:39->microservices/community/contracts/proto/j76-community-surface-v1.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 20: `compliance`
- wave_count: 9
- surface_count: 19
- files: 201
- journey_ips: 64
- decision_files: 6
- runbook_files: 13
- missing_refs: 227
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-W1, runbooks-surface-present
- surfaces: (root):109, benchmarks:1, capabilities:5, capability-tiers:1, catalog:11, contracts:4, dashboards:6, decisions:6, faqs:1, iac:11, migration-playbooks:1, onboarding:1, packs:5, policy:7, reference-implementations:1, runbooks:13, scorecards:5, slos:12, tutorials:1
- missing_ref_sample_1: microservices/compliance/IP-journey-j83-pack-overlay-regulator.md:37->microservices/compliance/contracts/openapi/j83-pack-overlay-regulator-v1.yaml
- missing_ref_sample_2: microservices/compliance/IP-journey-j83-pack-overlay-regulator.md:38->microservices/compliance/contracts/asyncapi/j83-pack-overlay-regulator-events-v1.yaml
- missing_ref_sample_3: microservices/compliance/IP-journey-j83-pack-overlay-regulator.md:39->microservices/compliance/contracts/proto/j83-pack-overlay-regulator-v1.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 21: `connect`
- wave_count: 9
- surface_count: 18
- files: 184
- journey_ips: 36
- decision_files: 1
- runbook_files: 10
- missing_refs: 44
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- surfaces: (root):72, benchmarks:1, capabilities:4, capability-tiers:1, catalog:48, contracts:10, dashboards:4, decisions:1, faqs:1, iac:11, migration-playbooks:1, onboarding:1, policy:11, reference-implementations:1, runbooks:10, scorecards:1, slos:5, tutorials:1
- missing_ref_sample_1: microservices/connect/ARCHITECTURE.md:218->microservices/policy-engine/
- missing_ref_sample_2: microservices/connect/IP-journey-j62-pharmacy-and-insurance-api.md:52->microservices/connect/contracts/openapi-j62-pharmacy-and-insurance-api.yaml
- missing_ref_sample_3: microservices/connect/IP-journey-j62-pharmacy-and-insurance-api.md:53->microservices/connect/contracts/asyncapi-j62-pharmacy-and-insurance-api.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 22: `consent-graph`
- wave_count: 8
- surface_count: 18
- files: 135
- journey_ips: 19
- decision_files: 5
- runbook_files: 8
- missing_refs: 52
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- surfaces: (root):52, benchmarks:1, capabilities:3, capability-tiers:1, catalog:17, contracts:3, dashboards:3, decisions:5, faqs:1, iac:20, migration-playbooks:1, onboarding:1, policy:4, reference-implementations:1, runbooks:8, scorecards:4, slos:9, tutorials:1
- missing_ref_sample_1: microservices/consent-graph/IP-journey-j83-consent-rights-ledger.md:37->microservices/consent-graph/contracts/openapi/j83-consent-rights-ledger-v1.yaml
- missing_ref_sample_2: microservices/consent-graph/IP-journey-j83-consent-rights-ledger.md:38->microservices/consent-graph/contracts/asyncapi/j83-consent-rights-ledger-events-v1.yaml
- missing_ref_sample_3: microservices/consent-graph/IP-journey-j83-consent-rights-ledger.md:39->microservices/consent-graph/contracts/proto/j83-consent-rights-ledger-v1.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 23: `contact-center`
- wave_count: 9
- surface_count: 21
- files: 172
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-A, capability-tier-surface-present, decisions-surface-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present, rust-src-W1, rust-src-surface-present
- surfaces: (root):49, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:6, dashboards:10, decisions:1, faqs:1, iac:24, migration-playbooks:1, onboarding:1, policies:6, policy:6, reference-implementations:1, runbooks:20, scorecards:1, slos:12, src:10, tests:1, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 24: `contract-lifecycle-management`
- wave_count: 8
- surface_count: 21
- files: 172
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-C, capability-tier-surface-present, decisions-surface-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W1, rust-src-surface-present
- surfaces: (root):49, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:6, dashboards:10, decisions:1, faqs:1, iac:24, migration-playbooks:1, onboarding:1, policies:6, policy:6, reference-implementations:1, runbooks:20, scorecards:1, slos:12, src:10, tests:1, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 25: `crm`
- wave_count: 7
- surface_count: 14
- files: 155
- journey_ips: 0
- decision_files: 1
- runbook_files: 6
- missing_refs: 0
- waves: decisions-surface-present, erp-ip-W2, migration-playbooks-surface-present, per-msvc-adrs-E, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):44, capabilities:3, catalog:54, contracts:3, dashboards:3, decisions:1, iac:9, migration-playbooks:3, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 26: `data-pipeline`
- wave_count: 7
- surface_count: 14
- files: 163
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 1
- waves: b2b-ip-deepen-C, decisions-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present, rust-src-W2, rust-src-surface-present
- surfaces: (root):50, capabilities:6, catalog:13, contracts:6, dashboards:10, decisions:1, iac:24, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:7, tests:1
- missing_ref_sample_1: microservices/data-pipeline/IP-027-lineage-graph-reconciliation.md:32->microservices/data-pipeline/dashboards/local-lineage-capture-gap.md
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 27: `data-warehouse`
- wave_count: 8
- surface_count: 15
- files: 166
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-B, decisions-surface-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present, rust-src-W2, rust-src-surface-present
- surfaces: (root):50, capabilities:6, catalog:13, contracts:6, dashboards:10, decisions:1, iac:24, migration-playbooks:3, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:7, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 28: `design-collaboration`
- wave_count: 6
- surface_count: 14
- files: 163
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-C, decisions-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W2, rust-src-surface-present
- surfaces: (root):50, capabilities:6, catalog:13, contracts:6, dashboards:10, decisions:1, iac:24, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:7, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 29: `detection`
- wave_count: 6
- surface_count: 18
- files: 130
- journey_ips: 0
- decision_files: 1
- runbook_files: 8
- missing_refs: 1
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present
- surfaces: (root):42, benchmarks:1, capabilities:8, capability-tiers:2, catalog:16, contracts:4, dashboards:8, decisions:1, faqs:1, iac:10, migration-playbooks:1, onboarding:1, policy:16, reference-implementations:1, runbooks:8, scorecards:1, slos:8, tutorials:1
- missing_ref_sample_1: microservices/detection/reference-implementations/streaming-score-rust-sdk.md:212->microservices/detection/reference-implementations/streaming-example/
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 30: `developer-sdk`
- wave_count: 9
- surface_count: 20
- files: 137
- journey_ips: 11
- decision_files: 7
- runbook_files: 8
- missing_refs: 94
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- surfaces: (root):30, benchmarks:1, capabilities:3, capability-tiers:1, catalog:18, contracts:4, dashboards:3, decisions:7, faqs:1, iac:18, implementation-plans:15, migration-playbooks:1, onboarding:1, packs:10, policy:4, reference-implementations:1, runbooks:8, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/developer-sdk/cross-microservice-handoffs.md:16->microservices/developer-sdk/contracts/proto/developer_sdk.proto
- missing_ref_sample_2: microservices/developer-sdk/cross-microservice-handoffs.md:17->microservices/developer-sdk/policies/
- missing_ref_sample_3: microservices/developer-sdk/implementation-plans/IP-002-developer-onboarding-kernel-domain.md:38->microservices/developer-sdk/src/crates/oya-developer-sdk-developer-onboarding-kernel/src/entities.rs
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 31: `docs`
- wave_count: 7
- surface_count: 18
- files: 128
- journey_ips: 10
- decision_files: 8
- runbook_files: 7
- missing_refs: 72
- waves: capability-tier-surface-present, decisions-surface-present, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- surfaces: (root):48, benchmarks:1, capabilities:3, capability-tiers:1, catalog:19, contracts:3, dashboards:3, decisions:8, faqs:1, iac:14, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/docs/IP-DOCS-002-sveltekit-marketing-site.md:33->microservices/docs/sveltekit/
- missing_ref_sample_2: microservices/docs/PRD.md:149->microservices/docs/specs/naming-justification.md
- missing_ref_sample_3: microservices/docs/IP-002-document-store-kernel.md:30->microservices/docs/src/crates/oya-docs-document-store-kernel/Cargo.toml
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 32: `drive`
- wave_count: 7
- surface_count: 21
- files: 175
- journey_ips: 46
- decision_files: 8
- runbook_files: 7
- missing_refs: 133
- waves: capability-tier-surface-present, decisions-surface-present, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-surface-present
- surfaces: (root):79, benchmarks:1, capabilities:3, capability-tiers:1, catalog:24, contracts:3, dashboards:3, decisions:8, faqs:1, iac:16, migration-playbooks:1, onboarding:1, packs:5, policy:6, reference-implementations:1, runbooks:7, scorecards:1, security:1, slos:9, test-plans:3, tutorials:1
- missing_ref_sample_1: microservices/drive/backfill-replay.md:23->microservices/drive/src/crates/oya-drive-migration-{gdrive,dropbox,onedrive,box,nextcloud,s3,webdav}-app
- missing_ref_sample_2: microservices/drive/IP-journey-j66-filing-archive.md:52->microservices/drive/contracts/openapi-j66-filing-archive.yaml
- missing_ref_sample_3: microservices/drive/IP-journey-j66-filing-archive.md:53->microservices/drive/contracts/asyncapi-j66-filing-archive.yaml
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 33: `feature-flags`
- wave_count: 9
- surface_count: 18
- files: 133
- journey_ips: 10
- decision_files: 1
- runbook_files: 9
- missing_refs: 8
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-E, runbooks-surface-present
- surfaces: (root):57, benchmarks:1, capabilities:5, capability-tiers:2, catalog:12, contracts:7, dashboards:4, decisions:1, faqs:1, iac:9, migration-playbooks:1, onboarding:1, policy:12, reference-implementations:3, runbooks:9, scorecards:1, slos:5, tutorials:2
- missing_ref_sample_1: microservices/feature-flags/ARCHITECTURE.md:269->microservices/policy-engine/
- missing_ref_sample_2: microservices/feature-flags/PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md:163->microservices/policy-engine/
- missing_ref_sample_3: microservices/feature-flags/tutorials/cohort-rollout-with-analytics.md:20->microservices/analytics/audiences/
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 34: `financial-planning`
- wave_count: 6
- surface_count: 14
- files: 165
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-B, decisions-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W1, rust-src-surface-present
- surfaces: (root):49, capabilities:6, catalog:13, contracts:6, dashboards:10, decisions:1, iac:24, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 35: `finops-portal`
- wave_count: 10
- surface_count: 19
- files: 161
- journey_ips: 29
- decision_files: 8
- runbook_files: 11
- missing_refs: 38
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- surfaces: (root):48, benchmarks:1, capabilities:3, capability-tiers:1, catalog:6, contracts:3, dashboards:6, decisions:8, faqs:1, iac:19, implementation-plans:26, migration-playbooks:1, onboarding:1, policy:10, reference-implementations:1, runbooks:11, scorecards:5, slos:9, tutorials:1
- missing_ref_sample_1: microservices/finops-portal/IP-journey-j82-finance-risk-console.md:37->microservices/finops-portal/contracts/openapi/j82-finance-risk-console-v1.yaml
- missing_ref_sample_2: microservices/finops-portal/IP-journey-j82-finance-risk-console.md:38->microservices/finops-portal/contracts/asyncapi/j82-finance-risk-console-events-v1.yaml
- missing_ref_sample_3: microservices/finops-portal/IP-journey-j82-finance-risk-console.md:39->microservices/finops-portal/contracts/proto/j82-finance-risk-console-v1.proto
- sampled: yes
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 36: `forms`
- wave_count: 8
- surface_count: 18
- files: 145
- journey_ips: 16
- decision_files: 8
- runbook_files: 7
- missing_refs: 134
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- surfaces: (root):47, benchmarks:1, capabilities:3, capability-tiers:1, catalog:14, contracts:3, dashboards:3, decisions:8, faqs:1, iac:37, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/forms/IP-013-bulk-distribute-worker.md:24->microservices/forms/src/worker/bulk_distribute/worker.rs
- missing_ref_sample_2: microservices/forms/IP-013-bulk-distribute-worker.md:25->microservices/forms/src/worker/bulk_distribute/prefill_link.rs
- missing_ref_sample_3: microservices/forms/IP-013-bulk-distribute-worker.md:26->microservices/forms/src/worker/bulk_distribute/unsubscribe.rs
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 37: `foundry`
- wave_count: 8
- surface_count: 20
- files: 592
- journey_ips: 14
- decision_files: 2
- runbook_files: 41
- missing_refs: 438
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- surfaces: (root):133, bc-sources:80, benchmarks:1, capabilities:18, capability-tiers:1, catalog:135, contracts:18, dashboards:19, decisions:2, faqs:1, iac:73, migration-playbooks:1, onboarding:1, policy:41, reference-implementations:1, runbooks:41, scorecards:1, slos:16, spec:8, tutorials:1
- missing_ref_sample_1: microservices/intelligence/IP-064-guardrails-prompt-classifier-kernel.md:24->microservices/intelligence/src/crates/oya-intelligence-guardrails-prompt-classifier-kernel/
- missing_ref_sample_2: microservices/intelligence/IP-076-providers-router-kernel.md:24->microservices/intelligence/src/crates/oya-intelligence-providers-router-kernel/
- missing_ref_sample_3: microservices/intelligence/IP-076-providers-router-kernel.md:30->microservices/intelligence/src/crates/oya-intelligence-providers-router-kernel/Cargo.toml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 38: `global-trade`
- wave_count: 5
- surface_count: 11
- files: 138
- journey_ips: 0
- decision_files: 1
- runbook_files: 6
- missing_refs: 0
- waves: decisions-surface-present, erp-ip-W3, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):41, capabilities:3, catalog:54, contracts:3, dashboards:3, decisions:1, iac:9, policy:13, runbooks:6, scorecards:1, slos:4
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 39: `governance`
- wave_count: 8
- surface_count: 19
- files: 204
- journey_ips: 20
- decision_files: 2
- runbook_files: 11
- missing_refs: 111
- waves: capability-tier-surface-present, decisions-surface-present, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-W1, runbooks-surface-present
- surfaces: (root):60, benchmarks:1, capabilities:3, capability-tiers:1, catalog:41, contracts:3, dashboards:3, decisions:2, faqs:1, iac:58, migration-playbooks:1, onboarding:1, packs:1, policy:7, reference-implementations:1, runbooks:11, scorecards:1, slos:7, tutorials:1
- missing_ref_sample_1: microservices/governance/IP-013-aggregation-index-generation-lane.md:34->microservices/governance/catalog/oya-check-aggregation-index-generation.yaml
- missing_ref_sample_2: microservices/governance/IP-001-scaffold-umbrella-bcs.md:24->microservices/governance/src/crates/
- missing_ref_sample_3: microservices/governance/IP-journey-j81-policy-and-attestation.md:37->microservices/governance/contracts/openapi/j81-policy-and-attestation-v1.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 40: `healthcare-integration`
- wave_count: 8
- surface_count: 19
- files: 160
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-C, capability-tier-surface-present, decisions-surface-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):48, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:6, dashboards:10, decisions:1, faqs:1, iac:24, migration-playbooks:1, onboarding:1, policies:6, policy:6, reference-implementations:1, runbooks:20, scorecards:1, slos:12, tutorials:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 41: `identity`
- wave_count: 8
- surface_count: 20
- files: 237
- journey_ips: 112
- decision_files: 6
- runbook_files: 11
- missing_refs: 276
- waves: capability-tier-surface-present, decisions-surface-present, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-W1, runbooks-surface-present
- surfaces: (root):144, benchmarks:1, capabilities:5, capability-tiers:1, catalog:11, contracts:6, dashboards:3, decisions:6, faqs:1, iac:20, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:11, scorecards:5, security:1, slos:9, test-plans:3, tutorials:1
- missing_ref_sample_1: microservices/identity/IP-journey-j59-sso-disable.md:52->microservices/identity/contracts/openapi-j59-sso-disable.yaml
- missing_ref_sample_2: microservices/identity/IP-journey-j59-sso-disable.md:53->microservices/identity/contracts/asyncapi-j59-sso-disable.yaml
- missing_ref_sample_3: microservices/identity/IP-journey-j59-sso-disable.md:54->microservices/identity/contracts/j59-sso-disable.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 42: `incident-management`
- wave_count: 8
- surface_count: 21
- files: 172
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 1
- waves: b2b-ip-deepen-B, capability-tier-surface-present, decisions-surface-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W1, rust-src-surface-present
- surfaces: (root):49, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:6, dashboards:10, decisions:1, faqs:1, iac:24, migration-playbooks:1, onboarding:1, policies:6, policy:6, reference-implementations:1, runbooks:20, scorecards:1, slos:12, src:10, tests:1, tutorials:1
- missing_ref_sample_1: microservices/incident-management/onboarding/incident-commander-first-week.md:38->microservices/incident-management/templates/runbook-template.md
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 43: `intelligence`
- wave_count: 11
- surface_count: 20
- files: 176
- journey_ips: 38
- decision_files: 1
- runbook_files: 17
- missing_refs: 95
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-W4, runbooks-surface-present
- surfaces: (root):84, benchmarks:1, capabilities:8, capability-tiers:1, catalog:12, contracts:7, dashboards:6, decisions:1, faqs:1, iac:8, migration-playbooks:1, onboarding:1, policy:12, reference-implementations:1, runbooks:17, scorecards:1, security:1, slos:9, test-plans:3, tutorials:1
- missing_ref_sample_1: microservices/intelligence/IP-002-domain-layer-secret-reference.md:25->microservices/intelligence/crates/
- missing_ref_sample_2: microservices/intelligence/IP-002-domain-layer-secret-reference.md:32->microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/Cargo.toml
- missing_ref_sample_3: microservices/intelligence/IP-002-domain-layer-secret-reference.md:33->microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/src/lib.rs
- sampled: yes
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 44: `itsm`
- wave_count: 8
- surface_count: 21
- files: 173
- journey_ips: 0
- decision_files: 1
- runbook_files: 20
- missing_refs: 1
- waves: b2b-ip-deepen-B, capability-tier-surface-present, decisions-surface-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W2, rust-src-surface-present
- surfaces: (root):50, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:6, dashboards:10, decisions:1, faqs:1, iac:24, migration-playbooks:4, onboarding:1, policies:6, policy:6, reference-implementations:1, runbooks:20, scorecards:1, slos:12, src:7, tests:1, tutorials:1
- missing_ref_sample_1: microservices/itsm/IP-002-cedar-default-deny.md:104->microservices/itsm/policies/local-service-catalog-publish-approval.cedar
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 45: `learning-management`
- wave_count: 5
- surface_count: 13
- files: 162
- journey_ips: 0
- decision_files: 0
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-A, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W2, rust-src-surface-present
- surfaces: (root):50, capabilities:6, catalog:13, contracts:6, dashboards:10, iac:24, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:7, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 46: `mail`
- wave_count: 8
- surface_count: 20
- files: 208
- journey_ips: 75
- decision_files: 6
- runbook_files: 10
- missing_refs: 329
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-surface-present
- surfaces: (root):113, benchmarks:1, capabilities:3, capability-tiers:1, catalog:17, contracts:3, dashboards:5, decisions:6, faqs:1, iac:17, migration-playbooks:1, onboarding:1, packs:5, policy:10, reference-implementations:1, runbooks:10, scorecards:1, security:1, slos:10, tutorials:1
- missing_ref_sample_1: microservices/mail/IP-009-search-index.md:30->microservices/mail/src/crates/oya-mail-search-index-kernel/
- missing_ref_sample_2: microservices/mail/IP-009-search-index.md:31->microservices/mail/src/crates/oya-mail-search-index-domain/
- missing_ref_sample_3: microservices/mail/IP-009-search-index.md:32->microservices/mail/src/crates/oya-mail-search-index-usecase/
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 47: `marketing-automation`
- wave_count: 5
- surface_count: 13
- files: 164
- journey_ips: 0
- decision_files: 0
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-A, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W1, rust-src-surface-present
- surfaces: (root):49, capabilities:6, catalog:13, contracts:6, dashboards:10, iac:24, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 48: `marketplace`
- wave_count: 9
- surface_count: 20
- files: 134
- journey_ips: 15
- decision_files: 1
- runbook_files: 13
- missing_refs: 31
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, per-msvc-adrs-D, runbooks-W3, runbooks-surface-present, rust-src-surface-present
- surfaces: (root):35, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:3, dashboards:5, decisions:1, faqs:1, iac:12, ip:25, migration-playbooks:1, onboarding:1, policies:6, reference-implementations:1, runbooks:13, scorecards:1, slos:6, src:1, tutorials:1
- missing_ref_sample_1: microservices/marketplace/IP-journey-j65-order-export.md:52->microservices/marketplace/contracts/openapi-j65-order-export.yaml
- missing_ref_sample_2: microservices/marketplace/IP-journey-j65-order-export.md:53->microservices/marketplace/contracts/asyncapi-j65-order-export.yaml
- missing_ref_sample_3: microservices/marketplace/IP-journey-j65-order-export.md:54->microservices/marketplace/contracts/j65-order-export.proto
- sampled: yes
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 49: `meet`
- wave_count: 8
- surface_count: 18
- files: 139
- journey_ips: 21
- decision_files: 8
- runbook_files: 7
- missing_refs: 50
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- surfaces: (root):52, benchmarks:1, capabilities:3, capability-tiers:1, catalog:23, contracts:3, dashboards:3, decisions:8, faqs:1, iac:13, migration-playbooks:1, onboarding:1, policy:8, reference-implementations:1, runbooks:7, scorecards:1, slos:11, tutorials:1
- missing_ref_sample_1: microservices/meet/PRD.md:280->microservices/meet/runbooks/error-budget-policy.md
- missing_ref_sample_2: microservices/meet/PRD.md:315->microservices/meet/tests/e2e/room-record-transcribe.rs
- missing_ref_sample_3: microservices/meet/IP-001-iac-bootstrap.md:32->microservices/meet/iac/helm/meet/templates/{deployment,statefulset,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 50: `messenger`
- wave_count: 7
- surface_count: 19
- files: 166
- journey_ips: 47
- decision_files: 6
- runbook_files: 10
- missing_refs: 139
- waves: capability-tier-surface-present, decisions-surface-present, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-surface-present
- surfaces: (root):81, benchmarks:1, capabilities:3, capability-tiers:1, catalog:16, contracts:3, dashboards:3, decisions:6, faqs:1, iac:13, migration-playbooks:1, onboarding:1, policy:10, reference-implementations:1, runbooks:10, scorecards:1, slos:10, test-plans:3, tutorials:1
- missing_ref_sample_1: microservices/messenger/IP-journey-j55-support-thread.md:52->microservices/messenger/contracts/openapi-j55-support-thread.yaml
- missing_ref_sample_2: microservices/messenger/IP-journey-j55-support-thread.md:53->microservices/messenger/contracts/asyncapi-j55-support-thread.yaml
- missing_ref_sample_3: microservices/messenger/IP-journey-j55-support-thread.md:54->microservices/messenger/contracts/j55-support-thread.proto
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 51: `network`
- wave_count: 8
- surface_count: 18
- files: 126
- journey_ips: 14
- decision_files: 8
- runbook_files: 7
- missing_refs: 32
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- surfaces: (root):45, benchmarks:1, capabilities:3, capability-tiers:1, catalog:22, contracts:3, dashboards:3, decisions:8, faqs:1, iac:12, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/network/PRD.md:360->microservices/network/runbooks/error-budget-policy.md
- missing_ref_sample_2: microservices/network/PRD.md:400->microservices/network/tests/e2e/profile-connect-endorse.rs
- missing_ref_sample_3: microservices/network/threat-model.md:561->microservices/network/legal/baa-template.md
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 52: `notes`
- wave_count: 9
- surface_count: 18
- files: 160
- journey_ips: 27
- decision_files: 8
- runbook_files: 11
- missing_refs: 68
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- surfaces: (root):63, benchmarks:1, capabilities:3, capability-tiers:1, catalog:19, contracts:3, dashboards:5, decisions:8, faqs:1, iac:18, migration-playbooks:1, onboarding:1, policy:12, reference-implementations:1, runbooks:11, scorecards:1, slos:10, tutorials:1
- missing_ref_sample_1: microservices/notes/IP-journey-j61-soap-note.md:52->microservices/notes/contracts/openapi-j61-soap-note.yaml
- missing_ref_sample_2: microservices/notes/IP-journey-j61-soap-note.md:53->microservices/notes/contracts/asyncapi-j61-soap-note.yaml
- missing_ref_sample_3: microservices/notes/IP-journey-j61-soap-note.md:54->microservices/notes/contracts/j61-soap-note.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 53: `observability`
- wave_count: 8
- surface_count: 19
- files: 214
- journey_ips: 51
- decision_files: 1
- runbook_files: 13
- missing_refs: 225
- waves: capability-tier-surface-present, decisions-surface-present, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W1, runbooks-surface-present
- surfaces: (root):95, benchmarks:1, capabilities:3, capability-tiers:1, catalog:15, contracts:4, dashboards:6, decisions:1, faqs:1, iac:50, migration-playbooks:1, onboarding:1, packs:5, policy:6, reference-implementations:1, runbooks:13, scorecards:1, slos:8, tutorials:1
- missing_ref_sample_1: microservices/observability/IP-022-otel-to-clickhouse-bridge.md:20->microservices/observability/iac/helm/otel-collector-gateway/values.yaml
- missing_ref_sample_2: microservices/observability/IP-022-otel-to-clickhouse-bridge.md:21->microservices/observability/iac/helm/otel-collector-gateway/templates/collector-config.yaml.tpl
- missing_ref_sample_3: microservices/observability/IP-022-otel-to-clickhouse-bridge.md:22->microservices/observability/contracts/clickhouse-tables/metrics-table.sql
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 54: `ontology`
- wave_count: 10
- surface_count: 18
- files: 154
- journey_ips: 30
- decision_files: 2
- runbook_files: 13
- missing_refs: 104
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- surfaces: (root):71, benchmarks:1, capabilities:3, capability-tiers:1, catalog:18, contracts:3, dashboards:5, decisions:2, faqs:1, iac:16, migration-playbooks:1, onboarding:1, policy:9, reference-implementations:1, runbooks:13, scorecards:1, slos:6, tutorials:1
- missing_ref_sample_1: microservices/ontology/IP-journey-j84-typed-record-writer.md:37->microservices/ontology/contracts/openapi/j84-typed-record-writer-v1.yaml
- missing_ref_sample_2: microservices/ontology/IP-journey-j84-typed-record-writer.md:38->microservices/ontology/contracts/asyncapi/j84-typed-record-writer-events-v1.yaml
- missing_ref_sample_3: microservices/ontology/IP-journey-j84-typed-record-writer.md:39->microservices/ontology/contracts/proto/j84-typed-record-writer-v1.proto
- sampled: yes
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 55: `ops-dashboard-control-center`
- wave_count: 7
- surface_count: 17
- files: 154
- journey_ips: 24
- decision_files: 0
- runbook_files: 11
- missing_refs: 74
- waves: capability-tier-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W8-dynamic-avoidance, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present
- surfaces: (root):61, benchmarks:1, capabilities:8, capability-tiers:1, catalog:14, contracts:6, dashboards:6, faqs:1, iac:11, migration-playbooks:1, onboarding:1, policy:20, reference-implementations:1, runbooks:11, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/ops-dashboard-control-center/IP-journey-j79-operator-evidence-console.md:37->microservices/ops-dashboard-control-center/contracts/openapi/j79-operator-evidence-console-v1.yaml
- missing_ref_sample_2: microservices/ops-dashboard-control-center/IP-journey-j79-operator-evidence-console.md:38->microservices/ops-dashboard-control-center/contracts/asyncapi/j79-operator-evidence-console-events-v1.yaml
- missing_ref_sample_3: microservices/ops-dashboard-control-center/IP-journey-j79-operator-evidence-console.md:39->microservices/ops-dashboard-control-center/contracts/proto/j79-operator-evidence-console-v1.proto
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 56: `payments`
- wave_count: 11
- surface_count: 20
- files: 202
- journey_ips: 70
- decision_files: 1
- runbook_files: 13
- missing_refs: 123
- waves: capability-tier-surface-present, cross-handoff-matrix-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W2, runbooks-surface-present
- surfaces: (root):107, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:5, dashboards:8, decisions:1, faqs:1, iac:16, migration-playbooks:4, onboarding:1, policy:10, reference-implementations:1, runbooks:13, scorecards:1, security:1, slos:8, test-plans:3, tutorials:1
- missing_ref_sample_1: microservices/payments/IP-journey-j65-receipt-export.md:52->microservices/payments/contracts/openapi-j65-receipt-export.yaml
- missing_ref_sample_2: microservices/payments/IP-journey-j65-receipt-export.md:53->microservices/payments/contracts/asyncapi-j65-receipt-export.yaml
- missing_ref_sample_3: microservices/payments/IP-journey-j65-receipt-export.md:54->microservices/payments/contracts/j65-receipt-export.proto
- sampled: yes
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 57: `performance-management`
- wave_count: 5
- surface_count: 13
- files: 162
- journey_ips: 0
- decision_files: 0
- runbook_files: 20
- missing_refs: 0
- waves: b2b-ip-deepen-A, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W2, rust-src-surface-present
- surfaces: (root):50, capabilities:6, catalog:13, contracts:6, dashboards:10, iac:24, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:7, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 58: `plant-maintenance`
- wave_count: 4
- surface_count: 12
- files: 151
- journey_ips: 0
- decision_files: 0
- runbook_files: 6
- missing_refs: 1
- waves: erp-ip-W1, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):44, capabilities:3, catalog:54, contracts:3, dashboards:3, iac:9, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: microservices/plant-maintenance/IP-013-adapter-integrations-for-plant-maintenance.md:40->microservices/plant-maintenance/migrations/
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 59: `plugin-app-store`
- wave_count: 8
- surface_count: 20
- files: 147
- journey_ips: 21
- decision_files: 5
- runbook_files: 8
- missing_refs: 130
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W8-dynamic-avoidance, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-surface-present
- surfaces: (root):39, benchmarks:1, capabilities:3, capability-tiers:1, catalog:20, contracts:3, dashboards:3, decisions:7, faqs:1, iac:18, implementation-plans:15, migration-playbooks:1, onboarding:1, packs:10, policy:4, reference-implementations:1, runbooks:8, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/plugin-app-store/IP-journey-j75-quarantine.md:52->microservices/plugin-app-store/contracts/openapi-j75-quarantine.yaml
- missing_ref_sample_2: microservices/plugin-app-store/IP-journey-j75-quarantine.md:53->microservices/plugin-app-store/contracts/asyncapi-j75-quarantine.yaml
- missing_ref_sample_3: microservices/plugin-app-store/IP-journey-j75-quarantine.md:54->microservices/plugin-app-store/contracts/j75-quarantine.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 60: `production-planning`
- wave_count: 4
- surface_count: 12
- files: 152
- journey_ips: 0
- decision_files: 0
- runbook_files: 6
- missing_refs: 0
- waves: erp-ip-W1, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):45, capabilities:3, catalog:54, contracts:3, dashboards:3, iac:9, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 61: `quality-management`
- wave_count: 4
- surface_count: 12
- files: 151
- journey_ips: 0
- decision_files: 0
- runbook_files: 6
- missing_refs: 0
- waves: erp-ip-W1, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):44, capabilities:3, catalog:54, contracts:3, dashboards:3, iac:9, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 62: `real-estate`
- wave_count: 4
- surface_count: 12
- files: 151
- journey_ips: 0
- decision_files: 0
- runbook_files: 6
- missing_refs: 0
- waves: erp-ip-W2, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):44, capabilities:3, catalog:54, contracts:3, dashboards:3, iac:9, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 63: `recordings`
- wave_count: 8
- surface_count: 19
- files: 129
- journey_ips: 12
- decision_files: 9
- runbook_files: 8
- missing_refs: 10
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- surfaces: (root):45, benchmarks:1, capabilities:3, capability-tiers:2, catalog:16, contracts:3, dashboards:3, decisions:9, faqs:1, iac:15, legal:2, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:8, scorecards:1, slos:10, tutorials:1
- missing_ref_sample_1: microservices/recordings/PRD.md:378->microservices/recordings/runbooks/error-budget-policy.md
- missing_ref_sample_2: microservices/recordings/IP-001-iac-bootstrap.md:42->microservices/recordings/iac/helm/recordings/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml
- missing_ref_sample_3: microservices/recordings/IP-001-iac-bootstrap.md:44->microservices/recordings/iac/kustomize/overlays/pack-{kr,eu,us-healthcare,us-financial}/kustomization.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 64: `sheets`
- wave_count: 8
- surface_count: 18
- files: 126
- journey_ips: 10
- decision_files: 9
- runbook_files: 7
- missing_refs: 42
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- surfaces: (root):41, benchmarks:1, capabilities:3, capability-tiers:1, catalog:20, contracts:3, dashboards:3, decisions:9, faqs:1, iac:17, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/sheets/IP-002-cargo-workspace-cell-grid-kernel-domain.md:33->microservices/sheets/src/crates/oya-sheets-cell-grid-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}
- missing_ref_sample_2: microservices/sheets/IP-002-cargo-workspace-cell-grid-kernel-domain.md:34->microservices/sheets/src/crates/oya-sheets-cell-grid-domain/{Cargo.toml,src/lib.rs,src/cell_graph.rs,src/a1_notation.rs,src/dirty_marking.rs,tests/cell_graph.rs,tests/a1_parser.rs}
- missing_ref_sample_3: microservices/sheets/IP-002-cargo-workspace-cell-grid-kernel-domain.md:35->microservices/sheets/catalog/oya-sheets-cell-grid-{kernel,domain}.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 65: `shorts`
- wave_count: 8
- surface_count: 18
- files: 124
- journey_ips: 14
- decision_files: 8
- runbook_files: 7
- missing_refs: 23
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-surface-present
- surfaces: (root):45, benchmarks:1, capabilities:3, capability-tiers:2, catalog:19, contracts:3, dashboards:3, decisions:8, faqs:1, iac:12, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/shorts/PRD.md:325->microservices/shorts/runbooks/error-budget-policy.md
- missing_ref_sample_2: microservices/shorts/PRD.md:363->microservices/shorts/tests/e2e/full-cycle.rs
- missing_ref_sample_3: microservices/shorts/IP-001-iac-bootstrap.md:46->microservices/shorts/iac/helm/shorts/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 66: `sites`
- wave_count: 7
- surface_count: 18
- files: 123
- journey_ips: 10
- decision_files: 8
- runbook_files: 7
- missing_refs: 15
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present
- surfaces: (root):43, benchmarks:1, capabilities:3, capability-tiers:1, catalog:16, contracts:3, dashboards:3, decisions:8, faqs:1, iac:17, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/sites/migration-from-connect.md:29->microservices/sites/src/crates/
- missing_ref_sample_2: microservices/sites/migration-from-connect.md:38->microservices/sites/src/crates/
- missing_ref_sample_3: microservices/sites/deprecation-notice.md:27->microservices/sites/src/crates/
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 67: `slides`
- wave_count: 8
- surface_count: 18
- files: 129
- journey_ips: 10
- decision_files: 10
- runbook_files: 7
- missing_refs: 7
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W4-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- surfaces: (root):41, benchmarks:1, capabilities:3, capability-tiers:1, catalog:25, contracts:3, dashboards:3, decisions:10, faqs:1, iac:13, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:10, tutorials:1
- missing_ref_sample_1: microservices/slides/IP-002-presentation-slide-kernel-domain.md:38->microservices/slides/catalog/oya-slides-presentation-{kernel,domain}.yaml
- missing_ref_sample_2: microservices/slides/IP-002-presentation-slide-kernel-domain.md:39->microservices/slides/catalog/oya-slides-slide-{kernel,domain}.yaml
- missing_ref_sample_3: microservices/slides/sdk-plan.md:91->microservices/slides/src/crates/oya-slides-sdk/README.md
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 68: `social`
- wave_count: 7
- surface_count: 11
- files: 144
- journey_ips: 14
- decision_files: 7
- runbook_files: 12
- missing_refs: 35
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present
- surfaces: (root):50, capabilities:3, catalog:23, contracts:3, dashboards:6, decisions:7, iac:17, policy:12, runbooks:12, scorecards:1, slos:10
- missing_ref_sample_1: microservices/social/IP-journey-j90-social-moderation-surface.md:37->microservices/social/contracts/openapi/j90-social-moderation-surface-v1.yaml
- missing_ref_sample_2: microservices/social/IP-journey-j90-social-moderation-surface.md:38->microservices/social/contracts/asyncapi/j90-social-moderation-surface-events-v1.yaml
- missing_ref_sample_3: microservices/social/IP-journey-j90-social-moderation-surface.md:39->microservices/social/contracts/proto/j90-social-moderation-surface-v1.proto
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 69: `supply-chain-planning`
- wave_count: 4
- surface_count: 12
- files: 149
- journey_ips: 0
- decision_files: 0
- runbook_files: 6
- missing_refs: 0
- waves: erp-ip-W3, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):42, capabilities:3, catalog:54, contracts:3, dashboards:3, iac:9, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 70: `tasks`
- wave_count: 8
- surface_count: 18
- files: 124
- journey_ips: 10
- decision_files: 8
- runbook_files: 7
- missing_refs: 56
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-C, runbooks-surface-present
- surfaces: (root):43, benchmarks:1, capabilities:3, capability-tiers:2, catalog:19, contracts:3, dashboards:3, decisions:8, faqs:1, iac:13, migration-playbooks:1, onboarding:1, policy:7, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/tasks/IP-004-task-store-adapter-postgres.md:50->microservices/tasks/src/oya-tasks-task-store-adapter-postgres/src/lib.rs
- missing_ref_sample_2: microservices/tasks/IP-010-view-engine-and-board-realtime.md:49->microservices/tasks/src/oya-tasks-view-engine-{usecase,api,adapter,adapter-redis,rest,app}/src/lib.rs
- missing_ref_sample_3: microservices/tasks/IP-010-view-engine-and-board-realtime.md:50->microservices/tasks/src/oya-tasks-view-engine-domain/tests/rerank_determinism.rs
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 71: `tenancy`
- wave_count: 9
- surface_count: 18
- files: 197
- journey_ips: 61
- decision_files: 1
- runbook_files: 11
- missing_refs: 202
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W1-bootstrap-fullbuildout, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-A, runbooks-W1, runbooks-surface-present
- surfaces: (root):106, benchmarks:2, capabilities:6, capability-tiers:1, catalog:17, contracts:3, dashboards:6, decisions:1, faqs:2, iac:19, migration-playbooks:2, onboarding:2, policy:10, reference-implementations:2, runbooks:11, scorecards:1, slos:4, tutorials:2
- missing_ref_sample_1: microservices/tenancy/IP-journey-j83-tenant-pack-scope.md:37->microservices/tenancy/contracts/openapi/j83-tenant-pack-scope-v1.yaml
- missing_ref_sample_2: microservices/tenancy/IP-journey-j83-tenant-pack-scope.md:38->microservices/tenancy/contracts/asyncapi/j83-tenant-pack-scope-events-v1.yaml
- missing_ref_sample_3: microservices/tenancy/IP-journey-j83-tenant-pack-scope.md:39->microservices/tenancy/contracts/proto/j83-tenant-pack-scope-v1.proto
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 72: `translate`
- wave_count: 7
- surface_count: 18
- files: 122
- journey_ips: 11
- decision_files: 7
- runbook_files: 7
- missing_refs: 24
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W2-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, runbooks-surface-present
- surfaces: (root):42, benchmarks:1, capabilities:3, capability-tiers:2, catalog:19, contracts:3, dashboards:3, decisions:7, faqs:1, iac:14, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:7, scorecards:1, slos:9, tutorials:1
- missing_ref_sample_1: microservices/translate/PRD.md:134->microservices/translate/src/crates/
- missing_ref_sample_2: microservices/translate/IP-007-quality-estimation-stack.md:74->microservices/translate/capabilities/eval/qe-golden.jsonl
- missing_ref_sample_3: microservices/translate/sdk-plan.md:190->microservices/translate/src/crates/oya-translate-router-sdk
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 73: `treasury`
- wave_count: 4
- surface_count: 12
- files: 151
- journey_ips: 0
- decision_files: 0
- runbook_files: 6
- missing_refs: 0
- waves: erp-ip-W3, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):44, capabilities:3, catalog:54, contracts:3, dashboards:3, iac:9, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 74: `warehouse`
- wave_count: 4
- surface_count: 12
- files: 151
- journey_ips: 0
- decision_files: 0
- runbook_files: 6
- missing_refs: 0
- waves: erp-ip-W2, runbooks-surface-present, rust-src-W3, rust-src-surface-present
- surfaces: (root):44, capabilities:3, catalog:54, contracts:3, dashboards:3, iac:9, policy:13, runbooks:6, scorecards:1, slos:4, src:10, tests:1
- missing_ref_sample_1: none
- missing_ref_sample_2: none
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: lower
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 75: `whiteboard`
- wave_count: 5
- surface_count: 13
- files: 164
- journey_ips: 0
- decision_files: 0
- runbook_files: 20
- missing_refs: 2
- waves: b2b-ip-deepen-C, pack-overlay-or-pack-surface-present, runbooks-surface-present, rust-src-W1, rust-src-surface-present
- surfaces: (root):49, capabilities:6, catalog:13, contracts:6, dashboards:10, iac:24, policies:6, policy:6, runbooks:20, scorecards:1, slos:12, src:10, tests:1
- missing_ref_sample_1: microservices/whiteboard/IP-023-dpia-evidence-packet.md:25->microservices/whiteboard/export-render
- missing_ref_sample_2: microservices/whiteboard/IP-023-dpia-evidence-packet.md:26->microservices/whiteboard/template-marketplace-install
- missing_ref_sample_3: none
- sampled: no
- ownership_risk: moderate
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 76: `workflow-engine`
- wave_count: 9
- surface_count: 18
- files: 226
- journey_ips: 96
- decision_files: 1
- runbook_files: 9
- missing_refs: 335
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W3, runbooks-surface-present
- surfaces: (root):127, benchmarks:1, capabilities:3, capability-tiers:1, catalog:47, contracts:3, dashboards:3, decisions:1, faqs:1, iac:12, migration-playbooks:1, onboarding:1, policy:7, reference-implementations:1, runbooks:9, scorecards:1, slos:6, tutorials:1
- missing_ref_sample_1: microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:37->microservices/workflow-engine/contracts/openapi/j86-cadence-orchestrator-v1.yaml
- missing_ref_sample_2: microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:38->microservices/workflow-engine/contracts/asyncapi/j86-cadence-orchestrator-events-v1.yaml
- missing_ref_sample_3: microservices/workflow-engine/IP-journey-j86-cadence-orchestrator.md:39->microservices/workflow-engine/contracts/proto/j86-cadence-orchestrator-v1.proto
- sampled: yes
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 77: `workflow-studio`
- wave_count: 9
- surface_count: 20
- files: 226
- journey_ips: 15
- decision_files: 7
- runbook_files: 12
- missing_refs: 154
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W3-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-B, runbooks-W3, runbooks-surface-present
- surfaces: (root):59, benchmarks:1, capabilities:3, capability-tiers:1, catalog:15, clients:9, contracts:3, dashboards:4, decisions:7, faqs:1, iac:16, migration-playbooks:1, onboarding:1, policy:6, reference-implementations:1, runbooks:12, scorecards:1, slos:7, templates:77, tutorials:1
- missing_ref_sample_1: microservices/workflow-studio/IP-027-cedar-grammar-impl.md:39->microservices/intelligence/specs/cedar/
- missing_ref_sample_2: microservices/workflow-studio/IP-027-cedar-grammar-impl.md:40->microservices/workflow-studio/decisions/ADR-0183.md
- missing_ref_sample_3: microservices/workflow-studio/backfill-replay.md:107->microservices/workflow-studio/contracts/asyncapi.yaml
- sampled: no
- ownership_risk: high
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

### Service 78: `workplace-integration`
- wave_count: 10
- surface_count: 20
- files: 135
- journey_ips: 16
- decision_files: 1
- runbook_files: 13
- missing_refs: 31
- waves: capability-tier-surface-present, decisions-surface-present, doc-suite-W5-gapfill, journey-waves-present, migration-playbooks-surface-present, pack-overlay-or-pack-surface-present, per-msvc-adrs-D, runbooks-W3, runbooks-surface-present, rust-src-surface-present
- surfaces: (root):36, benchmarks:1, capabilities:6, capability-tiers:1, catalog:13, contracts:3, dashboards:5, decisions:1, faqs:1, iac:12, ip:25, migration-playbooks:1, onboarding:1, policies:6, reference-implementations:1, runbooks:13, scorecards:1, slos:6, src:1, tutorials:1
- missing_ref_sample_1: microservices/workplace-integration/IP-journey-j54-e-signature.md:52->microservices/workplace-integration/contracts/openapi-j54-e-signature.yaml
- missing_ref_sample_2: microservices/workplace-integration/IP-journey-j54-e-signature.md:53->microservices/workplace-integration/contracts/asyncapi-j54-e-signature.yaml
- missing_ref_sample_3: microservices/workplace-integration/IP-journey-j54-e-signature.md:54->microservices/workplace-integration/contracts/j54-e-signature.proto
- sampled: yes
- ownership_risk: severe
- recommended_probe: assign a single service owner to reconcile PRD, ADRs, contracts, SLOs, runbooks, journey IP references, and migration surfaces before accepting the service as coherent.

## Raw command evidence appendix

- **microservice live count:** `find microservices -mindepth 1 -maxdepth 1 -type d | wc -l` -> 78
- **ADR git history:** `git log --oneline -- docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md` -> no output
- **ADR status:** `git status --short -- docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md` -> untracked
- **ADR section count:** `rg -c "^### Section D-" docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md` -> 155
- **Oya VCS status:** `./bin/oya vcs status` -> `oya vcs status accepted: action=read-status agent=- scopes=0 evidence=0`
- **Claim evidence search:** `find . -maxdepth 4 ( .oya / claim evidence )` -> no .oya tree found; controlled IP-010 claim demo evidence found
- **Matrix generation:** `/private/tmp/lane2_matrix.tsv` generated from live filesystem and curated transcript wave sets
- **Missing reference scan:** Markdown backtick-path scan over `microservices/<svc>` for `microservices/...` references that do not currently resolve

## Final lane-2 verdict

Lane 2 is supported at **High confidence / Strong evidence** for the ADR-0321 race and for the broader surface-wave ownership pattern. It is supported at **Medium confidence / Moderate evidence** for ledger-level claim-ratchet failure because the current audit lacks the live claim ledger snapshot from the race window. The best causal phrasing is: coordination and ownership failures did not create every wrong idea, but they allowed wrong, duplicate, and internally unowned content to land at scale before any one owner reconciled a service or shared file.

## Service-owner reconciliation checklist appendix

### Checklist target: `analytics`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 23
- current_wave_count: 8

### Checklist target: `api-gateway`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 20
- current_wave_count: 10

### Checklist target: `application`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 49
- current_wave_count: 9

### Checklist target: `audit-chain`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 217
- current_wave_count: 10

### Checklist target: `calendar`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 114
- current_wave_count: 7

### Checklist target: `cell`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 90
- current_wave_count: 8

### Checklist target: `cloud-billing`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `cloud-billing-tax`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 3

### Checklist target: `cloud-data`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 3

### Checklist target: `cloud-iac`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 126
- current_wave_count: 9

### Checklist target: `cloud-iam`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `cloud-k8s`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 54
- current_wave_count: 7

### Checklist target: `cloud-kms`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `cloud-network`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `cloud-network-dns`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 3

### Checklist target: `cloud-secrets`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 66
- current_wave_count: 9

### Checklist target: `cloud-storage`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 3

### Checklist target: `comms-email`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 5
- current_wave_count: 10

### Checklist target: `community`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 85
- current_wave_count: 8

### Checklist target: `compliance`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 227
- current_wave_count: 9

### Checklist target: `connect`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 44
- current_wave_count: 9

### Checklist target: `consent-graph`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 52
- current_wave_count: 8

### Checklist target: `contact-center`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 9

### Checklist target: `contract-lifecycle-management`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 8

### Checklist target: `crm`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 7

### Checklist target: `data-pipeline`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 1
- current_wave_count: 7

### Checklist target: `data-warehouse`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 8

### Checklist target: `design-collaboration`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 6

### Checklist target: `detection`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 1
- current_wave_count: 6

### Checklist target: `developer-sdk`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 94
- current_wave_count: 9

### Checklist target: `docs`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 72
- current_wave_count: 7

### Checklist target: `drive`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 133
- current_wave_count: 7

### Checklist target: `feature-flags`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 8
- current_wave_count: 9

### Checklist target: `financial-planning`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 6

### Checklist target: `finops-portal`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 38
- current_wave_count: 10

### Checklist target: `forms`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 134
- current_wave_count: 8

### Checklist target: `foundry`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 438
- current_wave_count: 8

### Checklist target: `global-trade`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `governance`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 111
- current_wave_count: 8

### Checklist target: `healthcare-integration`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 8

### Checklist target: `identity`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 276
- current_wave_count: 8

### Checklist target: `incident-management`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 1
- current_wave_count: 8

### Checklist target: `intelligence`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 95
- current_wave_count: 11

### Checklist target: `itsm`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 1
- current_wave_count: 8

### Checklist target: `learning-management`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `mail`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 329
- current_wave_count: 8

### Checklist target: `marketing-automation`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `marketplace`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 31
- current_wave_count: 9

### Checklist target: `meet`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 50
- current_wave_count: 8

### Checklist target: `messenger`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 139
- current_wave_count: 7

### Checklist target: `network`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 32
- current_wave_count: 8

### Checklist target: `notes`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 68
- current_wave_count: 9

### Checklist target: `observability`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 225
- current_wave_count: 8

### Checklist target: `ontology`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 104
- current_wave_count: 10

### Checklist target: `ops-dashboard-control-center`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 74
- current_wave_count: 7

### Checklist target: `payments`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 123
- current_wave_count: 11

### Checklist target: `performance-management`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 5

### Checklist target: `plant-maintenance`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 1
- current_wave_count: 4

### Checklist target: `plugin-app-store`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 130
- current_wave_count: 8

### Checklist target: `production-planning`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 4

### Checklist target: `quality-management`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 4

### Checklist target: `real-estate`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 4

### Checklist target: `recordings`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 10
- current_wave_count: 8

### Checklist target: `sheets`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 42
- current_wave_count: 8

### Checklist target: `shorts`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 23
- current_wave_count: 8

### Checklist target: `sites`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 15
- current_wave_count: 7

### Checklist target: `slides`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 7
- current_wave_count: 8

### Checklist target: `social`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 35
- current_wave_count: 7

### Checklist target: `supply-chain-planning`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 4

### Checklist target: `tasks`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 56
- current_wave_count: 8

### Checklist target: `tenancy`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 202
- current_wave_count: 9

### Checklist target: `translate`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 24
- current_wave_count: 7

### Checklist target: `treasury`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 4

### Checklist target: `warehouse`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 0
- current_wave_count: 4

### Checklist target: `whiteboard`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 2
- current_wave_count: 5

### Checklist target: `workflow-engine`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 335
- current_wave_count: 9

### Checklist target: `workflow-studio`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 154
- current_wave_count: 9

### Checklist target: `workplace-integration`
- PRD read before edit: required before future done claim
- ADR decisions read before edit: required before future done claim
- contracts existence checked: required before future done claim
- SLO targets reconciled: required before future done claim
- runbook references resolved: required before future done claim
- capability-tier matrix reconciled: required before future done claim
- journey IP references resolved: required before future done claim
- migration playbooks aligned to ADR-0321: required before future done claim
- cross-service handoffs checked: required before future done claim
- Oya VCS claim scope recorded: required before future done claim
- current_missing_refs: 31
- current_wave_count: 10

