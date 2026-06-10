# 20 — Top-Level Canonical Docs Review (lane: toplevel-canonical)

**Scope.** DEEP-READ of the highest-signal `~/Developer/source/docs/*.md` canonical docs vs the
12-item RULED CANON. These are the docs whose contradictions matter most because every downstream
doc inherits their framing.

**Docs reviewed (full):** `DESIGN.md` (812L), `MASTERPLAN.md` (112L), `PRD.md` (268L),
`PRD-OYATIE-FROM-SCRATCH-CANONICAL.md` (837L), `DOC-CATALOG.md` (394L), `CONTRADICTION-LEDGER.md`
(143L), `COMPETITIVE-GAP-ANALYSIS.md` (269L), `MISTAKES-LEDGER.md` (90L). **Skimmed for stale
terms:** `GLOSSARY.md` (1149L).

**Sibling lane note.** `10-stale-term-footprint.md` already owns the MECHANICAL term inventory
(foundry 731 files, tenant-tier 138, Kafka 44, Redis 49, etc.). This lane does NOT re-count; it
reads for **content/framing contradictions** vs canon and reachability. Where a term hit is just
mechanical rename, I defer to lane 10; I flag only where the *framing/decision* contradicts canon.

**Reachability legend.** DECISION→ADR ; INSTRUCTION→session-context-bundle ;
GENERATED-REFERENCE→built-from-specs ; ORPHAN→archive.

---

## TOP CONTRADICTIONS (lead — the genuine canon violations)

| # | Canon item | Doc:line | What it says vs canon |
|---|---|---|---|
| **A** | (1) masterplan GENERATED from ADRs | `MASTERPLAN.md:36`, `:44` vs `DOC-CATALOG.md:70`,`:82` | **Two competing authority models on the same shelf.** MASTERPLAN.md correctly declares "canonical master plan is `/specs/masterplan.json`… is GENERATED" — but DOC-CATALOG `doc.masterplan` row (`:70`) still treats `MASTERPLAN.md` as a hand-authored, council-owned doc with `agent_authoring_allowed: NO` and update-on-change, i.e. as *authority* not *projection*. Canon: ADRs=SSOT, masterplan is generated. The catalog row must reclassify MASTERPLAN.md as GENERATED-REFERENCE. |
| **B** | (2) "foundry" RETIRED → cloud-intelligence / governance | `DESIGN.md:19,28,68–305` (entire §1,§3); `PRD.md:16,32,§13`; `MASTERPLAN.md:60,104`; `PRD-CANONICAL.md:103,406,§9.5` | **"Foundry" is the load-bearing axis name across all four primary docs.** DESIGN §1 makes "Foundry" Axis 4 ("AI agent runtime + control plane + engineering platform"); PRD North Star sells "one foundry that compounds engineering quality." Canon retired the brand → **cloud-intelligence** (AI/agent substrate sense) or **governance** (fitness/policy lane). GLOSSARY *already records this retirement* (`GLOSSARY.md:1034` "RETIRED 2026-05-21 per ADR-0335", `:1040` successor=`microservices/intelligence/`) — so the canonical docs **contradict their own glossary.** This is the single biggest framing debt; the substrate sense (runtime/provider/capability) → intelligence, the fitness/council/amendment sense → governance. |
| **C** | (6) Cedar = CONTRACT; owned **PARC** = engine | `GLOSSARY.md:894`; `:105`; `PRD-CANONICAL.md:311`,`:793`; `DESIGN.md:535` | **Cedar is framed AS the policy ENGINE, and PARC does not appear in ANY top-level canonical doc** (grep PARC across DESIGN/PRD/PRD-CANONICAL/MASTERPLAN = 0 hits). GLOSSARY `:894`: "Cedar is Oyatie's default-deny authorization policy **engine** and the policy language." PRD-CANONICAL `:311` "Cedar policy language **and engine**", `:793` "Use Cedar for application authorization." Canon: Cedar = the **contract/policy language**; the **owned PARC** is the engine. Every "Cedar engine" assertion must split into Cedar-the-contract + PARC-the-engine. |
| **D** | (4) CI = unified **oya-ci**; Jenkins/Argo OPERATIVE-until-cutover, NOT canonical endpoint | `MASTERPLAN.md:106`; `PRD-CANONICAL.md:330,333,§8.6,:749,:802`; `PRD.md` (Development Order) | **Jenkins is framed as the canonical promotion gate**, not a build-first-cutover-later bridge. MASTERPLAN `:106`: "Promote only through… **Jenkins required checks**." PRD-CANONICAL `:330` "CI: GitHub Actions for hosted; **Jenkins LTS** for self-hosted/air-gapped/on-prem/colo parity"; `:333` + `:802` bake "Jenkins required checks" into the agent-safe lifecycle and the implementation prompt; `:749` lists Jenkins Pipeline as a source-of-truth. Canon: unified **oya-ci** (Run+graph, Prow+Tekton+Argo) is the endpoint; GitHub Actions/Jenkins/Argo are operative-until-cutover bridges. `oya-ci` appears in **zero** of these docs. |
| **E** | (5) OWN the whole data tier (endpoint); Postgres/Citus/Milvus/ClickHouse = TRANSITIONAL bridges; Redis→Valkey, Kafka→Pulsar | `PRD-CANONICAL.md:296–305,§8.4`,`:789–792`; `DESIGN.md:316,517–519`; `COMPETITIVE-GAP-ANALYSIS.md:163` | **Bridges are framed as canonical stack, not transitional.** PRD-CANONICAL §8.4 hard-codes "Primary OLTP: PostgreSQL / Horizontal: **Citus** per cell / Vector: **Milvus** / OLAP query: **ClickHouse**" with no own-the-tier endpoint and **no Pulsar** (event backbone = "**Kafka** in KRaft mode", `:299`). DESIGN §4 frameworks layer still lists "Kafka, Postgres" (`:316`); §9 "per-tenant Postgres shard", "Kafka-style partitions" (`:517–518`). *Partial canon-compliance to credit:* PRD-CANONICAL **does** use Valkey (`:299,:339`) and bans Redis (`:119,:339`) — that half is correct. The Kafka→Pulsar swap and the "these are bridges, we own the endpoint" framing are missing everywhere. |
| **F** | (7) Isolation: framekernel-host COMMITTED endpoint; assume-breach **microVM DEFAULT** | `PRD-CANONICAL.md:319,§8.5,:797`; `DESIGN.md` (no isolation-default statement) | **Default isolation is Kata + Cloud Hypervisor, with no assume-breach/microVM-default framing and no framekernel-host endpoint.** PRD-CANONICAL `:319`: "Tenant-customer untrusted code: **Kata Containers plus Cloud Hypervisor-class** VM isolation" — scoped to *untrusted tenant code* only, i.e. native-default for everything else (the exact posture canon retired). Canon: assume-breach microVM is the DEFAULT (not native-default/secure-by-default-native), framekernel-host is the committed endpoint. Neither "framekernel" nor "assume-breach" nor "microVM default" appears. |
| **G** | (6) Identity: **oya-identity** owned, Zitadel BRIDGE | `PRD-CANONICAL.md:§6.2,:144,:97`; `DESIGN.md:543`; `GLOSSARY.md:900` | Identity is framed as raw protocol primitives (OIDC/SAML/WebAuthn) with **no owned `oya-identity` product and no Zitadel-as-bridge framing**. DESIGN `:543` couples "IAM/SSO/SAML/OIDC IdP" to `oya-cloud-iam-kernel` + `oya-platform-identity-kernel` but never names the owned identity endpoint or the vendor bridge. `oya-identity` appears in zero top-level docs. (Lower severity than A–F but a clean own-endpoint/vendor-bridge gap.) |
| **H** | (9) tenant-CLASS vocab; namespace "tier"; M0-M3/MVP wave-vocab RETIRED | `DOC-CATALOG.md:51,52,200`; `DESIGN.md:78,86`; `PRD.md:159` ("per-tier") | **Wave-vocab + tenant-tier leakage in the catalog and DESIGN.** DOC-CATALOG still uses "[wave name per PRD §3.1]" placeholders (`:108,:200`) and the literal token in `:52` lists the W-* gate enum; DESIGN §3.0.1 uses "**tenant-tuned**… tenant-tier" framing (`:78`); PRD `:159` "per-tier consent target." Canon: tenant-**CLASS** (not tier); only namespace uses survive (autonomy_tier etc.). NOTE: GLOSSARY correctly records M0-M3 RETIRED (`:250,:504`) — again the docs contradict their own glossary. (Mostly mechanical → defer counts to lane 10; flagged here because DOC-CATALOG is an authority doc.) |
| **I** | (10) cloud/ DOGFOODS oya/ products; GLOBAL-CANONICAL core + localization packs; KR = FIRST pack (not KR-core) | `PRD.md:36,56,249` vs `PRD.md:91,§12`; `DESIGN.md:591` | **Internal contradiction inside PRD.md on KR-first.** PRD North Star `:36` "Korea-as-launch-locale is the test bed"; `:56` "Korea launches first; global is W4+"; decision-log `:249` "Korea-as-launch-locale (re-affirmed)" — but §3.1 `:91` and DESIGN §12 (`:591` "earlier 'Korea-as-launch-locale' framing is **retired**") assert global-canonical + KR-as-one-pack. Canon sides with the latter (global-canonical core, KR = first pack to market, NOT KR-core). PRD.md carries both framings unreconciled. |

---

## Per-doc findings

### DESIGN.md — AMEND (heavy) — reachability: DECISION-derived → should be GENERATED-REFERENCE
- **Canon B (foundry):** the doc is structurally built on the "Foundry" axis (§1 table `:28`, all of §3 `:68–305`, §13.3–13.4). Not a rename — the *axis identity* must re-home to cloud-intelligence (runtime/provider/capability/RAG/model substrate) vs governance (fitness functions, gates, scorecards, claim-ceiling, bypass ledger). §3.0.5.3 blast-radius + §9 fitness functions = governance sense; §3.0.1 model substrate + §3.0 provider adapters = intelligence sense.
- **Canon C (Cedar/PARC):** `:535` "Identity / RBAC / **Cedar policy**" contract row treats Cedar as the policy engine surface; no PARC.
- **Canon E (data bridges):** `:316` frameworks layer "Kafka, Postgres"; `:517–519` "per-tenant Postgres shard / Kafka-style partitions / Postgres streaming replication" with no bridge framing, no Pulsar/Valkey.
- **Stale framing / AI-slop:** §3.0.5.5 "Investment payback (rough estimates)" table (`:211–223`) is **fabricated precision** — "~2 weeks ⇒ thousands of $/month", "1-2 reviewers worth of leverage" with no basis; classic AI-slop. §3 has heavy hedging/marketing ("single highest-leverage investment", "compounds exponentially"). The "2026-05-09 consolidation" inline notes (`:21,:74,:692`) are change-log residue that belongs in CHANGELOG, not the design body.
- **Internal contradiction:** §3 header `:70` "Foundry is **second**, not first" vs §3 body `:279` "**Foundry is therefore… sequenced first**." Same section, opposite claims (also logged as LEDG-002, marked RESOLVED, but the contradiction text is still live in DESIGN).
- **Good (keep):** §10 cross-axis contract table, §12 regional-pack seam architecture (the sum-not-product property `:676`), §5 single-Tenant-kernel are sound and canon-compatible.

### MASTERPLAN.md — AMEND — reachability: GENERATED-REFERENCE (mislabeled as authority)
- **Canon A:** header is *internally* correct (`:30` "compatibility projection… not the implementation authority"; `:36` canonical = `/specs/masterplan.json`) — but DOC-CATALOG treats it as hand-authored authority (see Contradiction A). Reconcile the catalog, not the doc header.
- **Canon B:** `:60` + `:104` list "foundry" as an FD-001 surface and a parallel lane → re-home to intelligence/governance.
- **Canon D:** `:106` "Jenkins required checks" as the promotion gate → oya-ci endpoint.
- **Refinement:** "FD-001" first-deliverable framing (`:48–66`) is a clean gate-defined wave (good, canon-compatible) — but the surface list still says "foundry, workflow, ontology" (`:60`); swap foundry→intelligence.

### PRD.md — AMEND (heavy) — reachability: DECISION-derived → GENERATED-REFERENCE
- **Canon B:** North Star `:32,:34` "one foundry that compounds engineering quality"; §13 Foundry axis. Re-home.
- **Canon I:** internal KR-first contradiction (see Contradiction I) — `:36,:56,:249` vs §3.1/§12.
- **Stale wave-vocab:** §3.1 wave list `:96–117` is the canonical descriptive-wave source (good — M0-M3 already retired here), but `:159` "per-tier consent target" leaks tenant-tier vocab → tenant-class.
- **AI-slop:** §4.1 success-metrics "≥ 50K/week, ≥ 99.5%" and "≥ 1 block per 100 PRs" (`:145,:147`) are fabricated-precision targets with "Why this number" rationales that are circular ("Below 3 = not a product"). §1 "Why now" (`:38–48`) is marketing-deck hedging ("lose to the integrated stack within 5 years").
- **Stale dep references:** `:263` cites `~/.claude/plans/look-at-all-outstanding-buzzing-teacup.md` and `:262` `.omx/ultragoal/` as sources — dangling/local paths, ORPHAN provenance.

### PRD-OYATIE-FROM-SCRATCH-CANONICAL.md — AMEND — reachability: DECISION-derived (newest, 2026-05-22; the MOST canon-aligned but still drifted)
- **This is the closest-to-canon doc** and the cleanest writing: it owns the data tier framing partially (Iceberg canonical write path `:304`, ClickHouse "must not become the canonical write path" `:304`), uses **Valkey** + bans **Redis** (`:119,:299,:339`), names **Kyverno** for admission separate from app policy (`:57,:313`), uses **OpenBao** + **SPIFFE/SPIRE** + **Cilium**. Credit where due.
- **Still violates:** Canon C (`:311,:793` "Cedar… and engine", no PARC), Canon D (`:330,:333,:749,:802` Jenkins as canonical CI + agent-safe lifecycle gate; no oya-ci), Canon E (`:296` Citus/`:301` Milvus as canonical not bridge; `:299` Kafka not Pulsar), Canon F (`:319` Kata+Cloud-Hypervisor only for untrusted tenant code; native-default everything else; no framekernel/assume-breach-default), Canon G (`:97,:144` raw OIDC/SAML, no oya-identity/Zitadel-bridge framing).
- **Canon B (mild):** uses "Foundry" as agent-runtime surface name (`:103,:406,§9.5`) — but consistently as the *runtime/engineering-automation* sense → cleanly re-homes to **intelligence**.
- **AI-slop:** low. The NFR latency tables (§10.2 `:476–488`) are aspirational-precision ("p99 <= 5 ms" tenant-context, "p99 <= 10 ms" Cedar) flagged by the doc's own risk row `:712` "Performance targets are aspirational" — at least honestly self-flagged.
- **Strong (keep):** §18 External Source Baseline (recheck-at-impl-time discipline) and §13 acceptance gates are exactly the verify-each-step posture — canon-compatible; this doc should be the **template** the others are reconciled toward.

### DOC-CATALOG.md — AMEND — reachability: INSTRUCTION/SESSION-CONTEXT (it's the doc-update protocol)
- **Canon A:** `doc.masterplan` row `:70` treats MASTERPLAN.md as hand-authored authority (`agent_authoring_allowed: NO`, update-on-change) — must reclassify as GENERATED-REFERENCE projection of `/specs/masterplan.json` (which IS separately cataloged at `:82` `doc.spec_masterplan`). The two rows assert different authorities for the same plan.
- **Canon B:** rows `doc.foundry_supervisor_*` (`:71–75`), `products/foundry/*` (`:195`,§2.5b 26 files `:210–239`), owner team `council-foundry`/`axis-foundry` everywhere → foundry-namespaced doc IDs + owner teams need the intelligence/governance split.
- **Canon H:** `:52` wave-gate enum + `:108,:200` "[wave name per PRD §3.1]" placeholders are unresolved template residue (AI-slop: unfilled placeholder). `EVT-TENANT-CLASS-ADDED` (`:51`) is actually **canon-correct** (tenant-class, not tier) — keep.
- **Refinement:** the validation-check catalog §4 (`:304–355`) is genuinely strong (machine-checkable gates) — but `glossary-vocabulary` (`:338`) "retired vocabulary hard-fails" should now include foundry, Jenkins-as-endpoint, Cedar-as-engine, tenant-tier as retired tokens.

### CONTRADICTION-LEDGER.md — AMEND (stale) — reachability: GENERATED-REFERENCE (should be emitted from a contradictions.json)
- Whole ledger predates canon: dated "Draft v0.1 — 2026-05-09" (`:8`). LEDG-002/006/013/018/022/023 (`:44,:48,:55,:65,:69,:70`) are all *about* Foundry naming/sequencing and are marked RESOLVED/DRAFTED against the OLD "Foundry-as-axis" resolution — the canon retirement (foundry→intelligence/governance per ADR-0335) **supersedes** these resolutions, so their "RESOLVED" status is now stale-wrong.
- **Footer `:144`** explicitly says "References to `oya-governance-*`… are intentional — they describe past state" — but the body uses `oya-governance-*` as live owner (e.g. `:51,:121`), so the footer's own carve-out is self-contradicting (the text it excuses is presented as current).
- This ledger is forensic; per canon it's a GENERATED-REFERENCE or ORPHAN. The live contradiction surface is THIS audit's synthesis ledger, not a 2026-05-09 draft. Recommend ARCHIVE-or-regenerate.

### COMPETITIVE-GAP-ANALYSIS.md — AMEND (light) — reachability: GENERATED-REFERENCE / forensic
- **Canon B:** §5 "Axis 4 — Foundry" (`:85–97`) → intelligence/governance.
- **Canon E:** `:163` "ClickHouse per ADR-0045 (verify license post-fork)" + `:113` "Grafana is AGPL — replace with in-house Leptos UI long-horizon" — these are *correctly* framed as transitional/replace-when-proven (canon-compatible ratchet posture). Keep that framing; just align ClickHouse to bridge-not-endpoint.
- **AI-slop:** the score column ("Gap (catastrophic)", "Edge", "Parity (planned)") is subjective-precision without rubric anchoring; "(catastrophic without depth)" used 4× is hedging-as-emphasis. Low harm (it's an analysis doc) but the scoring should cite the methodology in §1 consistently.
- **Good:** §9 cross-cutting gaps (migration tooling `:210`, trust portal `:216`, CPaaS `:204`) are concrete and useful.

### MISTAKES-LEDGER.md — KEEP (forensic) — reachability: INSTRUCTION/SESSION-CONTEXT (institutional memory)
- Genuinely valuable and mostly canon-neutral (MFL-0001..0016 are real mechanical-prevention records).
- **Canon B residue:** `oya-foundry-*` lane names (MFL-0011, MFL-0013) and "Foundry capability `oya.mistakes.detect-pattern`" (`:70`) → intelligence/governance rename. Footer `:91` has the same self-contradicting "intentional past-state" carve-out as the contradiction ledger.
- No AI-slop. This is the model for what a forensic ledger should look like (per-entry, mechanical, dated).

### GLOSSARY.md — AMEND (reconcile internal split) — reachability: GENERATED-REFERENCE
- **Most canon-AWARE doc but internally bifurcated:** the **bottom** (`:1034–1100`) correctly records foundry RETIRED (ADR-0335), Hermes RETIRED (ADR-0247), successor=`intelligence`, and the `oyatie.foundry.*` Cedar-principal-namespace-persists nuance. The **top tables** (`:105,:140,:250,:504,:507`) still carry pre-retirement framing.
- **Canon C (the sharp one):** `:894` "Cedar is Oyatie's… policy **engine**", `:105` "Cedar… authorization policy DSL", `:577–591` whole Cedar-permit/fragment vocabulary treats Cedar as the engine. **No PARC entry exists.** This is the authoritative term definition and it directly contradicts canon #6.
- **Refinement:** glossary should add PARC (owned engine), framekernel-host, assume-breach-microVM, oya-ci, oya-identity, Pulsar, tenant-class as canonical terms, and mark Cedar-as-engine / Jenkins-as-endpoint / foundry as retired in the *top* tables too (not just the bottom narrative).

---

## Counts (this lane)

- **Docs deep-read:** 8 ; skimmed: 1 (GLOSSARY).
- **Genuine canon-contradictions:** 9 distinct (A–I), spanning canon items 1,2,4,5,6,7,9,10 (item 6 hit twice: Cedar/PARC + identity).
- **Most-violated canon item:** (2) foundry-retirement — present as load-bearing framing in all 4 primary docs + catalog + 26 supervisor doc rows.
- **Self-contradiction (doc vs its own glossary):** 3 (foundry, M0-M3, Cedar — GLOSSARY records the retirement the other docs ignore).
- **Internal contradictions within one doc:** 2 (DESIGN §3 "second" vs "sequenced first"; PRD KR-first vs global-canonical).
- **AI-slop hotspots:** DESIGN §3.0.5.5 payback table (fabricated precision), PRD §4.1 metrics (circular rationales), DOC-CATALOG unresolved "[wave name]" placeholders.
- **Best-aligned doc (use as reconciliation template):** `PRD-OYATIE-FROM-SCRATCH-CANONICAL.md` (Valkey/Kyverno/OpenBao/Iceberg/SPIFFE correct; still needs C/D/E/F/G fixes).
- **Reachability dispositions:** MASTERPLAN/DESIGN/PRD/COMPETITIVE-GAP/GLOSSARY → GENERATED-REFERENCE (currently mislabeled as hand-authored authority for the first three); DOC-CATALOG/MISTAKES-LEDGER → INSTRUCTION/SESSION-CONTEXT; CONTRADICTION-LEDGER → ARCHIVE-or-regenerate (stale forensic, superseded resolutions).
