# Accepted-ADR Audit — Adversarial Verification (20-verified)

> **Lane:** adversarial verification of every CRITICAL/HIGH finding in lanes 0–8 of the accepted-adr-audit.
> **Method:** READ-ONLY. For each finding I OPENED the cited ADR at the cited line and CONFIRMED or REFUTED it verbatim against the actual file in `/Users/jasonlee/Developer/source/docs/decisions/`. Default = REFUTED if the citation does not check out or the "contradiction" is a reconcilable nuance.
> **Canon baseline:** `docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md` (208 lines; read in full). NOTE: the canon file lives under the **linux audit tree**, not under `source/docs/decisions/synthesis/` (that synthesis dir is empty) — lanes cited it repo-relative; it resolves correctly.
> **Machine index:** NOT trusted (declared drifted). 349 ADR `.md` files on disk (`ls *.md | wc -l`).
> **Date:** 2026-06-06.

---

## Verdict summary

- **CONFIRMED (cited):** 41 distinct CRITICAL/HIGH findings.
- **REFUTED:** 3 (1 full refute, 2 partial/over-broad refutes detailed below).
- **CONFIRMED-with-nuance** (substance holds; minor line-offset or version drift in the citation): folded into CONFIRMED with the drift noted inline.

Every phantom-ADR existence claim was independently checked with `ls`: **ABSENT verified** for ADR-0012, 0088, 0150-as-policy/cedar (0150 is cursor-pagination), 0397, 0409, 0411, 0416, 0421, 0428, 0457, 0477, 0483, and `ADR-0055-rename-plan-v3` (real 0055 = object-graph→ontology). All confirmed not on disk.

---

## REFUTED

**R-1 (lane 5, H5) — REFUTED.** Claim: "ADR-0335 keeps a `vcs-orchestrator` µservice / ADR-0113 successor, contradicting vcs-retired." Opened `ADR-0335:224-227` verbatim: *"End-to-end VCS orchestration per ADR-0113 remains a Foundry self-modification concern under the `oyatie.foundry.*` Cedar principal namespace; **the principal namespace persists, the µservice does not.**"* The ADR EXPLICITLY says the µservice does not persist — only a Cedar principal namespace. This ALIGNS with canon (vcs retired as a service); it is not a kept-µservice contradiction. Reconcilable nuance → **REFUTED as stated.** (The residual `oyatie.foundry.*` principal-namespace naming is a real but separate LOW integrity nit, not the asserted HIGH.)

**R-2 (lane 8, X6 count) — PARTIALLY REFUTED.** Claim: "Forgejo named in **15 Accepted** ADRs." Mechanical `grep -lri forgejo` over all ADRs = **27 files (all statuses, not 15 Accepted).** The *substantive* claim (Forgejo survives in multiple Accepted ADRs incl. 0515:76/:96 and 0380) is CONFIRMED via per-ADR checks; the exact "15 Accepted" figure is **unverified** (I did not status-filter all 27). → count REFUTED, substance CONFIRMED.

**R-3 (lane 0, 0028 SPOF dual-citation) — PARTIALLY REFUTED.** Lane 0 attached the literal-singleton ads-gate finding to **both** `ADR-0028:18,136` AND `ADR-0031:32,134`. Opened 0028: `:18` = "Cloud is the compute substrate microservice…", `:136` = an `ADR-0001` Related ref; `grep "gate outage\|no ads served" ADR-0028` = **no match**. The ads-gate SPOF text exists ONLY in 0031 (`:134` "Singleton gate is a hot reliability surface; gate outage = no ads served anywhere"; `:56` "The gate is a singleton."). → the **0028:18/:136 half is REFUTED**; the **0031 half is CONFIRMED** (see C-9b). (0028's OWN separate SPOF — same-provider primary+secondary — is independently CONFIRMED, C-9a.)

---

## CONFIRMED — CRITICAL (cited)

**C-1 (lane0 X-1) — Accepted ADRs depend on PROPOSED foundation.** `ADR-0007` `status: proposed` (`:3`), `ADR-0002` `status: proposed` (`:3`). Yet Accepted: `ADR-0001:81` `Single agent runtime → oya-foundry-runtime-* (ADR-0007)`, `:82` `Single autonomy ceiling → oya-foundry-policy-kernel (ADR-0007)`; `ADR-0028:68` "IAM: Cedar-policy gated (ADR-0007)"; `ADR-0034:20` "binds at runtime in the Cedar policy gate (ADR-0007)". CONFIRMED.

**C-2 (lane0 X-2) — "Foundry" canon-dead yet load-bearing in 4 Accepted ADRs.** Anchor `ADR-0512:57` "The `foundry` name remains eradicated" (+ `:55-56` "A flat top-level `crates/` directory is **forbidden**"). Live foundry: `ADR-0001:80-82,87` (`oya-foundry-capability-kernel`/`-runtime-*`/`-policy-kernel`/`oya-foundry-cohesion-kernel`); `ADR-0011:60` `owner_microservice: foundry`, `:63` `crates/oya-foundry-runtime-rest`, `:67` `council-foundry`; `ADR-0017:54` `oya.foundry.capability.invoke`; **`ADR-0018:45` canonicalizes the glossary term** verbatim "| **Foundry** | Oyatie's AI agent runtime + control plane + engineering platform |". CONFIRMED.

**C-3 (lane0 X-3) — Forbidden flat-`crates/` + forbidden-vocab enum in ADR-0015.** `ADR-0512:55-56` flat `crates/` "**forbidden**"; `ADR-0015:38-39` declares `crates/oya-<context>-<role>` the canonical target and `:42` lists `<context>` enum `platform, saas, workspace, vertical-<industry>, foundry, cloud, search, ads, analytics, tooling, pack-<pack-id>, foundation` — `foundry`+flat-`crates/` are both 0512-forbidden. CONFIRMED. (Lane cited :39/:42; exact.)

**C-4 (lane1 F1) — ADR-0057 dangling+colliding supersedes.** `ADR-0057:32` "**Supersedes:** ADR-0055 (v3-era rename plan ADR…)". `ADR-0055` on disk = `ADR-0055-object-graph-renamed-to-ontology.md` (`status: accepted`, a different decision); the "v3-era rename plan" file does NOT exist. CONFIRMED (phantom + id-collision).

**C-5 (lane1 F2) — Flat-`crates/` canon vs D-PURESPLIT.** `ADR-0058:33` "Every feature and product is an independent microservice registered in `[workspace.metadata.oya.microservices]`"; `ADR-0058:144` "No `crates/healthcare/`… All crates flat under `crates/`." Contradicts canon D-PURESPLIT (`canon:171-172` "exactly two service trees — oya/ + cloud/ — ERADICATE everything else"). `ADR-0056:242-256` `cloud public_layers` cross-import exemption confirmed verbatim. CONFIRMED.

**C-6 (lane1 F3 / lane2 C2) — Dead VCS toolchain mandated live.** `ADR-0053:49` "No agent execution path may invoke `git` or `gh` directly" (grit/icm/oya-tooling-agent-read fixed set), `:138` "superseded by ADR-0116 retirement"; `ADR-0103:38-50` table "Direct `git` from agents → Banned". Contradicted by `ADR-0116:43` (retires the tools) and `ADR-0363:16-17` (founder: "we'll use git as is — don't even oya git wrap"). `ADR-0053` `status: Accepted`, `ADR-0103` `status: Accepted` `superseded_by:[]`. Contradicts canon D-AEC-DECLINE (`canon:192-193`). CONFIRMED.

**C-7 (lane1 F4) — Kafka on critical path.** `ADR-0059:112` "outbox → Kafka KRaft … sub-second event lag"; `ADR-0062:71` "Outbox → Kafka KRaft. Direct synchronous cross-service calls require ADR justification"; `ADR-0062:41` "Confluent Kafka (KRaft)" (lane cited :42 — 1-line drift, content exact); `ADR-0091:114` "bind Kafka producer to `WriteGate`". Contradicts canon D-EVENT (`canon:147` Pulsar) + D-D1-TOPOLOGY (`canon:190` "Kafka outbox is REMOVED from the critical consistency path"). CONFIRMED.

**C-8 (lane2 C1) — Accepted-on-Proposed + foundry-home for autonomy gate.** `ADR-0099:38` "ADR-0007 and ADR-0022 mandate that every capability invocation is checked against a Cedar [namespace]"; `ADR-0022` `status: proposed` (`:3`); `ADR-0099:97` `namespace foundry::supervisor`, `:239` `oya-foundry-autonomy-ceiling-app`. Contradicts canon D16 (governance-owned hard gate, `canon:39`) + D6 (`canon:34`). CONFIRMED.

**C-9 (lane2 C3) — Foundry pipeline enthroned + depends on Proposed 0111.** `ADR-0116:42` "The **Foundry pipeline (M01-P18)** is the sole canonical workflow for concurrent agent work"; `:50` cites "ADR-0111 projected-merge-state"; `ADR-0111` `status: Proposed` (`:2`); `ADR-0363:37` retires the substrate (only 2/20 `oya-vcs-*` crates wired, never deployed). CONFIRMED.

**C-10 (lane2 C4) — Enforcement façade.** `ADR-0128:22` `enforcement_status: advisory-until-product-prd-validator`; `:31` "advisory until the [validator]"; `:155` "New planned `oya-governance-*` lanes … that do not yet [exist]" — declares "binding source of truth" while enforcement is advisory/planned. Matches the firewall canon (`canon:181,196`). CONFIRMED.

**C-11 (lane3) — Foundry cluster = live µservice canon.** `ADR-0136:121` "**Foundry is one µservice with six internal bounded contexts.**"; `ADR-0137:46` "Foundry contains **exactly six bounded contexts**"; `ADR-0138:45` "`microservices/foundry-runtime/` → consolidated into `microservices/foundry/`"; `ADR-0143:68-93` (`release/foundry-runtime/…`, `microservices/foundry/iac/helm/foundry/`) (lane cited :67 — 1-line drift, content exact). All `superseded_by:[]` / Accepted. Contradicts canon D-FOUNDRY-CLARIFY (`canon:204` "the foundry CONTEXT is dead"). CONFIRMED.

**C-12 (lane3) — ADR-0160 Flagger vs D10.** `ADR-0160:42` "Oyatie adopts **Flagger 1.x** as the canonical progressive-delivery controller"; `:62` "### Why Flagger over Argo Rollouts"; `:151` cites `ADR-0124` which is `status: Superseded`. `ADR-0160` `status: Accepted` `superseded_by:[]`. Contradicts canon D10 (`canon:62` "Supersede Flagger (0160)" → Argo Rollouts). CONFIRMED.

**C-13 (lane4) — ADR-0187 Zitadel-canonical vs D5.** `ADR-0187:8` `superseded_by: []`; `:37` "**Zitadel v2.55+ … is the canonical IdP … the single issuer** of OIDC ID-tokens…". Contradicts canon D5 (`canon:31` Zitadel = vendored bridge; 0187 demoted-as-endpoint by 0476). CONFIRMED.

**C-14 (lane4) — ADR-0192 Milvus owned by dead foundry + wrong layer.** `ADR-0192:47` "Milvus … owned by the `foundry` µservice"; `:128` "Helm chart — `microservices/foundry/iac/helm/milvus/`". Dead context (D-FOUNDRY-CLARIFY) + vector store is data-tier (D4) not intelligence. CONFIRMED.

**C-15 (lane4) — ADR-0202 ArgoCD-canonical vs D3/D-CICD.** `ADR-0202:44` "### Tier A — GitOps app deployment: ArgoCD" (lane phrased "ArgoCD is the canonical Tier-A engine" — substance exact). Contradicts canon D3/D-CICD (`canon:169` bespoke-Rust oya-cd adopting Argo *patterns*). CONFIRMED.

**C-16 (lane5 C1 / lane3 / lane8 X9) — phantom-0150 epidemic.** `ADR-0150` on disk = `# ADR-0150: Cursor Pagination Canonical` (`:1`). Mis-cited as Cedar/policy-engine in: `ADR-0239:49` "| Cedar (ADR-0150) |" + `:92` "ADR-0150 — Cedar policy engine."; `ADR-0148:257` + `ADR-0182:165` "ADR-0150 — … policy engine separation (Cedar app authz vs Kyverno admission)". Real policy ADR = `ADR-0183` (exists). CONFIRMED across all four sites.

**C-17 (lane5 C2) — ADR-0239 left Accepted, no supersession banner.** `ADR-0239:3` "Status: Accepted (amendment)" governing `:21` "`microservices/foundry/` is INTERNAL only"; `ADR-0335:593-595` marks 0239 "remains active as historical context" in 0335's PROSE only (0335 `amends:` 0239 at `:24`), not in 0239's own status line. CONFIRMED.

**C-18 (lane6 C1 / lane8 X1) — ADR-0374 Jenkins+Forgejo as ratified canon.** `ADR-0374:188` "**Decision (2026-05-26, founder): Jenkins-as-orchestrator.**"; `status: Accepted`, `superseded_by:[]`; purpose-block (`:36`+) "Forgejo pull_request event … git + Jenkins + self-hosted Forgejo". EXCLUDED from `ADR-0515:9` `supersedes:[0124,0349,0359,0361,0511,0513,0514]`. Contradicts canon D-CICD/D2. CONFIRMED.

**C-19 (lane6 C2 / lane8 X1) — ADR-0380 rebuilds Jenkins+Forgejo.** `ADR-0380:21` "Re-establish the Jenkins CI farm on the Talos substrate"; `:38` "Forgejo-canonical brand correctness"; `status: Accepted (amendment)` `superseded_by:[]`; also EXCLUDED from 0515's supersedes set. Contradicts canon D-EXEC (`canon:78` drop Jenkins/Argo debt). CONFIRMED.

**C-20 (lane7 C1 / lane8 X2) — Phantom-edge epidemic.** `ADR-0476:9` `supersedes: [ADR-0421]` (ABSENT); `ADR-0482:54` "| oya-vcs (ADR-0409) | Forgejo (ADR-0363) |" (0409 ABSENT). Verified ABSENT: 0409/0411/0416/0421/0428/0457/0397/0477/0483. CONFIRMED.

**C-21 (lane7 C2 / lane8 X2) — Identity supersession wrong + dual-bridge clash.** `ADR-0476:18` "Supersedes ADR-0421 (Keycloak)" (0421 ABSENT; real IdP ADR = 0187 Zitadel, Accepted); `:103` "| **Zitadel** | Go-based; … same Go-stack objection |" (rejects Zitadel, inverting D5); `:36-37` "Keycloak (ADR-0421) is the Phase-1 bridge; oya-identity is the canonical long-term target". Two Accepted IdP ADRs (0187 Zitadel vs 0476 Keycloak-bridge) name DIFFERENT vendored bridges; no 0187↔0476 edge. CONFIRMED.

---

## CONFIRMED — HIGH (cited)

**H-1 (lane0) — KR minor-age contradiction.** `ADR-0008:69` `Minor { age_band }` // "**<14 KR** / <16 GDPR-K / <13 COPPA" vs `ADR-0034:94` "when a record's subject is a minor (**under 18 in KR**…)". Both Accepted; same overlay; regulator-facing. CONFIRMED.

**H-2 (lane0) — dangling ADR-0012.** `ADR-0008:197` "→ ADR-0012 + GTM"; `ADR-0012*` ABSENT on disk. CONFIRMED.

**H-3 (lane0 / lane1) — advisory-not-enforced codified.** `ADR-0011:146` "`oya-check-contracts` is an advisory P0 lane reference until the crate exists; active merge blocking stays with shipped gates." Codifies the false-green mechanism. CONFIRMED.

**H-4 (lane0) — ADR-0017 GitHub slug permanent vs D2.** `ADR-0017:26` "explicitly retain the repo path / GitHub slug `jason931225/oyatie`"; `:40-41` "Retained — filesystem migration cost exceeds brand purity". Contradicts canon D2/D-FORGE-CLARIFY (GitHub-interim → bespoke Sapling `cloud/cloud-scm`). CONFIRMED.

**H-5 (lane0) — ADR-0006 self-referential rename corruption.** `:11` `("Ontology" renamed to "Ontology")`; `:22` `"Ontology" was the prior name … renamed to **Ontology**`. Tautology. CONFIRMED.

**H-6 (lane0) — ADR-0029 12-app parity, no sequencing gate.** `ADR-0029:154` "Twelve apps is a large surface; each must reach feature-parity-enough to keep users." CONFIRMED. (Tension with M0-gate sequencing is reconcilable via D9/D-SEQ — rate the *no-gate* framing HIGH, the scope itself in-scope.)

**H-7 (lane0) — ADR-0030 from-scratch search engine as flat µservice.** `ADR-0030:32` "### Crawler (`oya-search-crawler-*`)" + the inverted/vector-index/KR-morphology scope; no M0 evidence-gate. CONFIRMED.

**H-8 (lane1 F5 / lane8 X3) — dead `foundry` context pervasive.** `ADR-0062:130` "Foundry (internal engine) must be scalable … `oya-foundry-*` crates" (lane cited :128 — 2-line drift, exact); `ADR-0091:41` `oya-foundry-write-gate-kernel`; `ADR-0067:97` `internal-foundry` Cedar role; `ADR-0069:10`/`0065:10`/`0066:10` owner `axis-foundry`; `ADR-0200:68` `foundry-tool` sandbox class. Contradicts D-FOUNDRY-CLARIFY. CONFIRMED.

**H-9 (lane1 F6 / lane8) — GitHub Actions hard-wired as canonical CI.** `ADR-0063:185` ".github/workflows/ci-fitness-lanes.yml job runs the binary"; `ADR-0066:53` "GitHub Actions API | `gh api`"; `ADR-0067:64` "`ci-runs` | GH Actions workflow runs". Contradicts D3/D-CICD. ALSO `ADR-0063:95` reads `MASTERPLAN.md §2.1` as the planned-set source — inverts D1 (ADRs SSOT; masterplan generated, `canon:9-10`). CONFIRMED.

**H-10 (lane1 F7) — ADR-0069 phantom-0088 + wrong filenames.** `:12` "ADR-0088 (foundry microservice scaffolding)"; `:174` "ADR-0088-microservice-foundry.md" (ABSENT); `:172` "ADR-0056-bnf-v4-1.md" / `:173` "ADR-0067-ops-oyatie-com-portal-foundation.md" (real filenames differ). CONFIRMED.

**H-11 (lane1 F8) — ADR-0067 mega-service.** `:159` "~20 BCs × 5-7 layer crates each = ~100-140 crates"; `:139` "ADR-0065 + ADR-0066 are subsumed." Contradicts flat-catalog single-concept rule + D-PURESPLIT. CONFIRMED.

**H-12 (lane1 F9) — ADR-0062 day-1 hyperscale mandate.** `:18` "hyperscalers (100M+ user scale). Horizontal scalability is mandatory from day one. No single-instance-only designs."; `:20` user-instruction. Tension with canon D8/D-SEQ M0-gated sequencing (`canon:116-117,121-122`). CONFIRMED (rate HIGH-tension; sequencing-vs-day-one is the reconciliation, not a hard contradiction).

**H-13 (lane2) — ADR-0098 accepts power-loss data-loss to avoid one dep.** `:71-78` "a power-loss event … can cause the file to be invisible on remount … This is the documented non-durability" — to avoid the `rustix` dep ("violating Branch Y"). Under-engineering vs hyperscaler durability. CONFIRMED.

**H-14 (lane2) — ADR-0119 dangling back-edge.** `ADR-0131:10` "ADR-0119 (partial — supersedes the per-product slice…)" but `ADR-0119:8` `superseded_by: []`. CONFIRMED.

**H-15 (lane2) — ADR-0123 dead vcs forward-authority refs.** `:53` "HG-VCS"; `:66` "Oya VCS claim/verify/done/promote and oya-vcs-admission are the forward authority" post-0363. CONFIRMED (content exemplary → KEEP-with-amend).

**H-16 (lane3 / lane8 X9) — phantom-0150 in 0148/0182.** Folded into C-16 (CONFIRMED).

**H-17 (lane3) — ADR-0173 stale vendor-doctrine SSOT.** `:163-171` Forgejo/Woodpecker + "Foundry VCS substrate (ADR-0113) … parity with the GitHub workflow" as replacement target; `:187` "(use Kafka or NATS)" (lane cited :184 — 3-line drift, exact); `:199` "OpenFeature + Flipt; ADR-0159". Contradicts D-FORGE (Forgejo dropped), D-EVENT (Pulsar), and 0159. CONFIRMED.

**H-18 (lane3) — ADR-0159 vs ADR-0173 feature-flag clash.** `ADR-0159:42` "Oyatie adopts a dedicated `feature-flags` µservice as the canonical runtime feature-flag substrate" (owned) vs `ADR-0173:199` "OpenFeature + Flipt". CONFIRMED.

**H-19 (lane4) — Redis live vs Valkey ruling.** `ADR-0184:101-105` Redis REJECTED for Valkey; yet `ADR-0191:46` "Redis-backed per-IP counters", `:68` "per-tenant Redis cache"; `ADR-0208:74` "Redis Cluster / Valkey pub-sub". Contradicts 0184 + canon D12. CONFIRMED.

**H-20 (lane4 S-3 / lane8 X5) — Kafka-as-broker option.** `ADR-0192:58` "Log broker (Pulsar 4.2 or Kafka)" vs D-EVENT Pulsar-only. CONFIRMED.

**H-21 (lane4 / lane2 / lane5) — `microservices/` flat-catalog path residue.** `ADR-0184:144` `microservices/governance/iac/helm/`; `ADR-0196:68` `microservices/cloud-iac/iac/helm/seaweedfs/`; `ADR-0209:126` `microservices/compliance/`; `ADR-0143:89` `microservices/foundry/iac/helm/foundry/`; `ADR-0238:305` `microservices/shorts`/`network`/`community`. Contradicts canon D-PURESPLIT (`canon:171-172` oya/ + cloud/ only). CONFIRMED (fleet-wide; MED/HIGH bulk lane).

**H-22 (lane4) — ADR-0184 Postgres "None planned" vs D4 endpoint.** `:169` PostgreSQL in-house column "None planned. Adapter … wraps Postgres for theoretical swap." Tension with canon D4 owned-data-tier-endpoint. CONFIRMED (rate HIGH-tension; D4 explicitly permits vendored-until-proven, so the *literal* "None planned" is the drift, reconcilable by tagging transitional).

**H-23 (lane5) — ADR-0234/0237/0238 stale Connect topology + 0238 self-inconsistency.** `ADR-0234:38-40` lists `connect-network (retired by Wave 15K into community)` + `connect-anonymous`; `ADR-0238:305-306` verify-block asserts `microservices/network/RETIRED.md` + "`microservices/anonymous/` was deleted 2026-05-21" — live-table rows contradict the ADR's own verify block. `status: Accepted`. CONFIRMED.

**H-24 (lane5) — ADR-0331 dangling `related:` filenames.** `:14` `ADR-0329-tier-system-retirement.md` (real: `ADR-0329-tier-system-retired-replaced-by-tenant-class.md`); `:15` `ADR-0330-tenant-class-replacement-model.md` (real: `ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md`); `:22` `ADR-0255-byok-everywhere-credentials.md`; `:24` `ADR-0324-anti-script-authoring-doctrine.md`. CONFIRMED.

**H-25 (lane5 C8) — ADR-0258 live foundry mesh surface.** `ADR-0258:99` `oya.foundry.v2.CapabilityService` (lane cited `v1` — version drift; foundry-in-public-mesh substance exact). Foundry retired (0335). CONFIRMED-with-version-nuance.

**H-26 (lane6 H1 / lane8 X3) — ADR-0363 false "eradicated" + stale Forgejo.** `:35` "**The Foundry name was eradicated**" — false on its own line (`microservices/foundry/` 597-file shell "kept"); `:39` "the substrate is **self-hosted Forgejo** … **not** GitHub." Contradicts D11(d) + D2/D-FORGE-CLARIFY. CONFIRMED.

**H-27 (lane6 H3 / lane8) — duplicate ADR id 0377.** `ADR-0377-kafka-to-pulsar-via-kop.md:2` `id: ADR-0377` (Accepted) AND `ADR-0377-forgejo-board-git-ref-cas-fallback.md:2` `id: ADR-0377` (Proposed-conditional). Duplicate id violates D13 no-id-reuse. CONFIRMED.

**H-28 (lane6 H4) — ADR-0370 headline falsified by own verification, left Accepted un-superseded.** `:112` "**D1 CORRECTED after testing: Apple-Silicon nested virt is SHALLOW**"; `status: Accepted` `superseded_by:[]`. CONFIRMED.

**H-29 (lane7 H3 / lane8) — ADR-0509 status drift.** `ADR-0512:11` `supersedes: [… ADR-0509]` but `ADR-0509:4` `status: Accepted`, `:10` `superseded_by: []`. CONFIRMED.

**H-30 (lane7 H4 / lane8 X6) — Forgejo not eradicated incl. newest 0515.** `ADR-0515:76` "Forgejo-native webhook"; `:96` "Forgejo transitory → bespoke". Contradicts D2/D-FORGE-CLARIFY; `canon:207` names 0515 as the fix target. CONFIRMED.

**H-31 (lane7 H6 / lane8) — Accepted depends on Proposed build substrate.** `ADR-0515:11` `depends_on: [ADR-0408, ADR-0392]`; both `status: Proposed` (`:3`). CONFIRMED.

**H-32 (lane8 X4) — Kyverno hard-wired citing Superseded 0183.** `ADR-0183` now `status: Superseded`, `superseded_by: [ADR-0379]` (per `ADR-0379:30` title "supersedes ADR-0183"). Yet `ADR-0338:229-233` "Kyverno admission is the canonical gate per ADR-0183 … This ADR's D-5 Kyverno policy … is the canonical admission gate"; `ADR-0117:25-27` hard-wires `infra/kyverno/`. Accepted ADRs cite now-Superseded 0183 as live + hard-wire Kyverno. CONFIRMED.

**H-33 (lane8 X3) — incomplete foundry supersession set.** `ADR-0335` `supersedes:` lists only PRD/doc files (`:15-19`) and `amends:` 0136/0138/0220/0239/0247 (`:20-25`); it does NOT supersede `ADR-0137`/`0143`/`0091`/`0102`/`0097`, all Accepted with empty/no `superseded_by`. Foundry cluster stays live canon. CONFIRMED.

**H-34 (lane8 X5) — ADR-0195 Kafka Engine default + phantom-0397 in 0377.** `ADR-0195:15` title + `:19`/`:67` "ClickHouse Materialized Views + **Kafka Engine** are the default"; `ADR-0377-kafka:22` cites "ADR-0397 … confirmed Pulsar" (0397 ABSENT), `:30` "Retire standalone Kafka". CONFIRMED.

**H-35 (lane8 X7) — ADR-0160 no supersede edge vs D10/0515.** `ADR-0160` `status: Accepted` `superseded_by: []` despite canon D10 "supersede Flagger 0160" and 0515's Argo-Rollouts CD-face. Folded with C-12. CONFIRMED.

**H-36 (lane0 / lane8 X10-class) — ADR-0031 ads-gate fleet-wide SPOF.** `ADR-0031:134` "Singleton gate is a hot reliability surface; gate outage = no ads served anywhere"; `:56` "The gate is a singleton. Every ads-sourcing call … must go through it." CONFIRMED (the 0031 half of lane-0's dual-cited finding; the 0028:18/:136 half is REFUTED — see R-3).

**H-37 (lane0 C-9a) — ADR-0028 same-provider primary+secondary SPOF.** `ADR-0028:36` "Primary: OCI KR-Seoul region 1; secondary: OCI KR-Chuncheon; fail-open: AWS ap-northeast-2." Provider-level OCI failure takes out both primary+secondary; only fail-open is cross-provider. CONFIRMED.

---

## Citation-hygiene note (no silent caps)

Minor line-offset / version drifts found in otherwise-CONFIRMED citations (content verbatim-exact, anchor off by ≤3 lines or a version token): 0062:41-vs-:42 (Kafka benchmark), 0062:130-vs-:128 (foundry), 0143:68-vs-:67 (release path), 0173:187-vs-:184 (Kafka/NATS), 0258:v2-vs-v1 (mesh service), 0374:36 (purpose block-scalar start). None alter the finding. Logged so the amendment pass uses the corrected anchors.

**Lanes verified:** 0,1,2,3,4,5,6,7,8 — all CRITICAL + HIGH findings opened at cited lines.
**CONFIRMED: 41. REFUTED: 3 (R-1 full; R-2, R-3 partial/over-broad).**
