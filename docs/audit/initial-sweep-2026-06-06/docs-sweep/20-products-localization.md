# WF2 Lane — products / localization-packs / prds (canon sweep)

**Lane:** `products-localization`
**Scope ruling:** canon #10 (global-canonical core + localization packs; cloud DOGFOODS oya products; KR = first pack) and #11 (maximal vertical scope, M0-gated parallel lanes, own-endpoint/vendor-bridge/ratchet). Cross-checked against the full canon set (#1–#12).
**Source root:** `/Users/jasonlee/Developer/source/docs`

## Coverage honesty (no silent truncation)

| Cluster | Files | Treatment |
|---|---|---|
| `products/` top-level | README.md, _TEMPLATE.md, product-docs-w1-checkpoint.md | **deep-read in full** |
| `products/*/PRD.md` (4 live) | cloud, foundry, erp-coverage, workplace-integration | foundry **deep-read** (head + grep-targeted body); cloud **deep-read head + grep body**; erp-coverage + workplace-integration **grep-targeted canon-signal read** (each ~2,500 lines; not line-by-line) |
| `products/foundry/supervisor/**` (21 files) | jsonl/settings/supervisor adapters+kernels | **counted + name-scanned only** (grep footprint); not deep-read — flagged as a cluster below |
| `products/foundry/PHASE-00-SPEC.md` | 1 | grep-scanned |
| `localization-packs/` | INDEX.md, kr.md | **deep-read in full** |
| `localization-packs/kr/evidence/*` (12) | application…workflow-studio | foundry.md **deep-read**; rest **name/grep-scanned** (each ~39 lines, templated) |
| `localization-packs/kr/{pack.yaml,corpus.lock}` | 2 | not opened (YAML/lock; manifest is authoritative per kr.md — noted, not audited here) |
| `prds/` | INDEX, foundry, tenancy, ontology **deep-read**; accounting/application/communications/hr/payroll/workflow **grep-scanned** (frontmatter + canon signals) |

Mechanical term counts (this lane): `foundry` 591 / `Cedar` 278 / `Postgres` 77 / `Kafka` 29 / `ClickHouse` 25 / `Citus` 19 / `Redis` 15 / `Milvus` 1 / `Jenkins` 4 (incidental). Dropped-SCM term count was zero in this lane.

---

## GENUINE CANON-CONTRADICTIONS (ranked — lead findings)

### C1 — "Foundry" brand is live AND doing BOTH jobs that canon #2 splits apart  *(canon #2; also #1)*
Canon #2 retires the "foundry" brand → **cloud-intelligence** (AI/agent substrate) **or** **governance** (fitness/policy lane), per context. The product corpus still uses "foundry" as a live brand/axis name (591 hits this lane) and, worse, uses the ONE name for BOTH of canon #2's split targets:

- **As the AI/agent substrate (→ should be cloud-intelligence):** `products/foundry/PRD.md:35` "Foundry — AI Agent Runtime"; `:70` "the unified AI agent runtime + control plane … the *substrate* that every other axis depends on"; `:73` agent runtime / provider adapters (Codex/Claude/Gemini). This is exactly the substrate canon #2 renames to cloud-intelligence.
- **As the fitness/governance/engineering lane (→ should be governance):** `products/foundry/PRD.md:75` "Foundry engineering platform surfaces (repoctl … fitness functions … scorecards … supply-chain attestation)"; `prds/foundry.md:20-24` "Foundry is the **internal-only** engineering engine … Proof Ladder L0→L7 … fitness lanes." This is the governance/fitness lane.

`prds/foundry.md` and `products/foundry/PRD.md` therefore describe two *different* products under one retired brand. **Fix:** split per canon #2 — agent-runtime content → `cloud-intelligence`; fitness/Proof-Ladder/CI-gate content → `governance`. Rename files, frontmatter (`product: foundry`, `microservice: foundry`, `owner_team: council-foundry`, `axis-foundry`), and crate prefixes accordingly.

### C2 — Half-completed `foundry→intelligence` rename, internally inconsistent inside single tables  *(canon #2 + AI-slop internal-contradiction)*
`products/foundry/PRD.md` is mid-rename: `oya-intelligence-*` crates sit in the SAME tables/rows as `oya-foundry-*` crates, several describing the same seam:
- `:148` `oya-intelligence-capability-domain` beside `:147,149` `oya-foundry-capability-kernel/app`.
- `:153` `oya-intelligence-run-domain`; `:170` `oya-intelligence-policy-domain`; `:172` `oya-intelligence-policy-api` ("…over `oya-foundry-policy-kernel`"); `:175` `oya-intelligence-registry-api`; `:178` `oya-intelligence-rag-api`.
- `:228` autonomy-ceiling seam: trait in `oya-foundry-policy-kernel`, publish fn in `oya-intelligence-policy-api` — one seam, two brand prefixes.
- `:195-197` fitness crates already renamed `oya-governance-*` (matches canon #2 governance target) while the doc still calls the lane "Foundry engineering platform."
**Fix:** complete the rename in one pass per canon #2 split; an architecture/contract fitness lane should reject mixed `oya-foundry-*` + `oya-intelligence-*` for the same context.

### C3 — KR pack evidence labelled `foundry` but evidence points to `intelligence` µservice  *(canon #2 + #10)*
`localization-packs/kr/evidence/foundry.md` — frontmatter `fd001_surface: foundry`, `source_microservice: foundry`, title "Foundry KR evidence" (`:4-10`) — but the cited evidence is `microservices/intelligence/manifest.json` + `microservices/intelligence/PRD.md` (`:17-18`). The µservice is already `intelligence`; the pack evidence label/filename is stale `foundry`. Also `kr.md:43` lists "Foundry" among pack-neutral layers. **Fix:** rename evidence doc + FD surface to `intelligence` (or `cloud-intelligence`) per canon #2; update `kr.md:43` and `pack.yaml` scope.

### C4 — M0–M3 / M01–M12 milestone wave-vocab is pervasive  *(canon #9: M0-M3/MVP wave-vocab RETIRED → gate-defined waves)*
This is the single highest-count contradiction in `prds/`:
- `prds/INDEX.md` "Milestone" column with `M02b-substrate-ready`, `M03-first-paying-tenant` (`:21-38`); `:51` "M04+ µservices … Wave 3+".
- Every substrate/enterprise PRD frontmatter `milestone_first_ship: M02b…/M03…/M02b…` (tenancy:9, ontology:9, application:9, workflow:9, communications:9, accounting:9, hr:9, payroll:9).
- `prds/foundry.md:26` "the substrate on which **M01–M12 milestones** depend"; `:65` "milestone dirs".
- `localization-packs/INDEX.md:31-47` "Lead milestone M01–M07 / M09,M11 / M12+"; `kr.md:7` `lead_milestones: [M01…M07]`, `:18` "M01–M07 ship", regulatory bindings table keyed by `M03/M04/M06/M07` (`:51-76`); `:139-141` "M01-P05 already green; M02-P22 / M03-P08".
- `products/foundry/PHASE-00-SPEC.md:399` "Foundry Phase 00 **milestone**".
**Note (NOT a violation):** the four `products/*/PRD.md` use **W-Foundation / W-*-Preview / W-Stable / W-Public-GA / W-Region-Fan-Out** wave names — this IS the canon #9 "gate-defined waves" shape and should be the template the `prds/` + localization milestone vocab migrates to. **Fix:** replace M0x/M01–M12 with gate-defined wave IDs; reconcile the two vocabularies (products W-* vs prds M0x) which currently disagree.

### C5 — `products/README.md` describes a "7-axis + 14-vertical" taxonomy that (a) is mostly broken links and (b) contradicts the flat-catalog ruling  *(canon #1 reachability; internal-contradiction; relates #11)*
- `README.md:19-48` lists 7 axis PRDs + 14 vertical PRDs with paths (`saas-platform/PRD.md`, `workspace/PRD.md`, `search/PRD.md`, `ads-analytics/PRD.md`, `vertical-corporate/PRD.md`, `vertical-healthcare/PRD.md`, … 16+ paths). **None exist** — the tree has only `cloud/`, `foundry/`, `erp-coverage/`, `workplace-integration/`. `product-docs-w1-2026-05-20-checkpoint.md:15` confirms "the four live product PRDs." → ~16 dead links / aspirational catalog.
- The "7 axes / Arm / Product Group" framing directly contradicts `prds/INDEX.md:42-44,63` which **forbids** "axes/arms/product-group" grouping ("No PRD may use retired glossary: no 'platform' … no 'Product Group', no 'Arm'") and mandates a flat µservice catalog. Two product taxonomies coexist, one explicitly retired by the other.
**Fix:** regenerate `products/README.md` from the live tree + flat-catalog ruling; either author the missing PRDs or drop the rows; remove axis/arm framing.

### C6 — `cloud/PRD.md` names Foundry as the cloud control-plane operator  *(canon #2)*
`cloud/PRD.md:49` "agent-operated (**Foundry** runs the control plane)"; persona table `:60` "Foundry agent … Cloud control-plane API surfaced as capabilities"; `:73` "Foundry capability surface (cloud.compute.provision …)". The cloud product is dogf-fooded by the agent substrate (good — matches canon #10 dogfood layering), but the substrate is still named "Foundry" → rename to cloud-intelligence per canon #2.

---

## STALE FRAMING (canon-adjacent; transitional tech framed as endpoint)

### S1 — Data-tier vendors framed as the destination, not transitional bridges  *(canon #5)*
Canon #5: OWN the whole tier (endpoint); Postgres/Citus/Milvus/ClickHouse = transitional bridges; Redis→Valkey; Kafka→Pulsar. Corpus is **inconsistent** (partial migration):
- **Already canon-correct (Valkey):** `tenancy.md:79,81,171,173,194` and `ontology.md:87,262` use **Valkey** for caching (Redis→Valkey done in the newer substrate PRDs). Good — these are the migration template.
- **Still stale Redis/Kafka:** `products/foundry/PRD.md:118` "Foundry uses Outbox + **Kafka** per ADR-0046" (→ Pulsar); `:182` memory backend "Postgres + **Redis**" (→ Valkey); `:139` adapter layer "…eventing, Postgres"; `cloud/PRD.md:73` managed-service list "Postgres / Citus / pgvector / **Redis** / **Kafka** / ClickHouse."
  - *Nuance:* `cloud/PRD.md:73` is a **managed-service catalog sold to tenants** (tenant workloads) — offering Redis/Kafka-compatible managed services to customers can be legitimate even post-cutover; but the wording presents them as the canonical names rather than "Valkey/Pulsar (Redis/Kafka-compatible)". Flag as framing, not hard contradiction.
- **Transitional bridges named without own-endpoint framing:** `Citus` (tenancy:183, ontology:201,272, workplace:1779), `ClickHouse` (ontology:203,264,274,311; foundry:740), `Milvus`/`pgvector` (cloud:74 gated). None carry the "transitional bridge → owned tier" framing canon #5 wants. Open questions even ask "ClickHouse M02 or deferred" (ontology:311) as if vendor-permanent.
- **Cloud infra bridges:** `cloud/PRD.md:60` "Linkerd → Istio Ambient, VictoriaMetrics → Mimir, Harbor, OpenBao, OpenTofu, Argo Rollouts" — the `→` shows transition-awareness (good) but the *terminal* is a vendor (Istio/Mimir), not an owned endpoint (canon #4/#5/#11 own-endpoint). Stale destination.

### S2 — Autonomy-ceiling ownership muddied vs sole-governance  *(canon #8)*
Canon #8: autonomy ceiling = runtime-enforced hard gate **owned by governance**. `products/foundry/PRD.md:830` lists owner "**Foundry + Governance**" (co-ownership) and `:74,228` describe the ceiling as a Foundry-owned seam. Post-#2 split this should land cleanly in governance. Minor framing fix (tighten to governance-owned; agent-runtime is a *consumer*).

### S3 — Unified governance-owned safety-gate set is absent, not contradicted  *(canon #12 — gap)*
The product docs reference HITL/human-in-the-loop and the autonomy ceiling (`foundry/PRD.md:877,899,914`) but **do not** reference the unified governance-owned safety-gate set (no-actuation / biometric-off / no-lethal). This is an **absence/gap** to fill when authoring vertical PRDs (esp. net-new defense/power-grid per task #18), not a present contradiction.
- *False-positive guard:* `workplace-integration/PRD.md:435,447,777,1578,1681` "biometric confirmation (FaceID/TouchID)" is **end-user auth**, NOT agent biometric surveillance — canon #12 "biometric-off" targets the agent safety-gate set, so this is **not** a violation. Do not mechanically rewrite.

---

## AI-SLOP / templated filler

### A1 — `erp-coverage/PRD.md`: ~150x verbatim-repeated acceptance boilerplate  *(AI-slop: templated repetition / fabricated-precision)*
Lines `:79–~400+` repeat, once per SAP module (FI/CO/MM/SD/PP/QM/PM/HCM/PS…40+ modules), the IDENTICAL sentence: *"Acceptance: the <Module> flow has Cedar authorization, ontology object mapping, workflow evidence, audit-chain event emission, and module-specific SLOs."* This is filler that conveys no per-module information and inflates the doc to 2,514 lines. **Fix:** collapse to one cross-module acceptance contract + a table of module-specific deltas; flag remainder for deslop pass.

### A2 — `products/foundry/supervisor/**` (21 files) — unaudited bominal-port cluster  *(stale / orphan candidate)*
21 files (`jsonl-supervisor-adapter`, `settings-template-{adapter,kernel}`, `supervisor-{app,kernel}` × README/ARCHITECTURE/BENCHMARKS/OPERATIONS/SECURITY). These are a Bominal "Foundry supervisor" port (the `foundry/PRD.md:45-64` "Foundry corpus cross-cite" points at `bominal/agents/ultragoal/*`). Not deep-read this pass. **Likely actions:** (a) all carry the retired `foundry` brand (canon #2); (b) BENCHMARKS files are a known fabricated-precision risk — verify numbers are real, not generated. Reachability: **ORPHAN candidate** unless reachable from the post-#2 cloud-intelligence PRD.

---

## REFINEMENT OPPORTUNITIES

- **R1 — Reconcile the two product taxonomies (C5).** Single source: flat µservice catalog (`prds/INDEX.md`) + the 4 live product PRDs; `products/README.md` should be GENERATED from the tree, not hand-listed (mirrors canon #1 masterplan-generated principle).
- **R2 — Reconcile wave vocab (C4).** Adopt the products `W-*` gate-defined waves everywhere; delete M0x from `prds/` frontmatter + localization milestone columns.
- **R3 — Cedar usage is canon-#6-consistent — leave alone.** Cedar is used as the policy **contract/gate** throughout (`tenancy:90`, `ontology:91,299`, `erp:*`, `workplace:303,458,1210`, `foundry:171,252`). No doc claims "Cedar = engine." `PARC` is *absent* (0 hits) — when authoring, name the owned engine **PARC** explicitly so the Cedar=contract / PARC=engine split (canon #6) is legible. Do NOT mechanically swap Cedar (it is correct as the contract).
- **R4 — `_TEMPLATE.md` is the leverage point.** It seeds every future PRD; it already uses W-wave language (good) and `tenant_class`-compatible framing, but `:159,166` hardcode "Redis" in the caching/optimization rows and `:66` "Kafka consumers" in the layer diagram → fix the template so new PRDs inherit Valkey/Pulsar (canon #5).
- **R5 — `tenant_class` is already canon-#9-correct everywhere** (`foundry:32`, `cloud:33`, `workplace:40`, `erp:33` all `tenant_class: ["demo_trial","paid"]`). No tenant-tier/tier-system hits in this lane. Leave alone; cite as the good pattern.

---

## REACHABILITY CLASSIFICATION

| Doc(s) | Class | Note |
|---|---|---|
| `products/{cloud,foundry,erp-coverage,workplace-integration}/PRD.md` | GENERATED-REFERENCE (from ADRs/catalog) | live; must be re-derived post-#2 rename |
| `prds/{tenancy,ontology,application,workflow,hr,payroll,accounting,communications,foundry}.md` | GENERATED-REFERENCE | per-µservice PRDs; `authority_chain: MASTERPLAN→ADR→PRD` already declared — canon #1-consistent shape |
| `products/README.md` | ORPHAN→needs-regeneration | broken-link aspirational catalog (C5) |
| `prds/INDEX.md`, `localization-packs/INDEX.md`, `kr.md` | GENERATED-REFERENCE (anchor/index) | keep; fix milestone vocab + foundry refs |
| `localization-packs/kr/evidence/*.md` | GENERATED-REFERENCE (evidence) | foundry.md mislabelled (C3); rest are thin templated stubs (39 lines) — verify they are real evidence, not slop |
| `products/_TEMPLATE.md` | INSTRUCTION (authoring template) | high-leverage; fix per R4 |
| `products/product-docs-w1-2026-05-20-checkpoint.md` | ORPHAN→not-needed | one-shot wave checkpoint; archive |
| `products/foundry/supervisor/**` (21) | ORPHAN candidate | unaudited bominal port (A2) |
| `products/foundry/PHASE-00-SPEC.md` | GENERATED-REFERENCE | foundry-branded (C1) + milestone vocab (C4) |

---

## DIGEST (counts)

- **Canon-contradictions:** 6 ranked (C1 foundry-brand-doing-both-jobs; C2 half-done foundry→intelligence rename; C3 KR foundry-evidence→intelligence mismatch; C4 M0x milestone vocab pervasive; C5 README phantom 7-axis taxonomy + flat-catalog conflict; C6 cloud "Foundry runs control plane").
- **Stale framing:** 3 (S1 data-tier vendors-as-endpoint [partial Valkey migration]; S2 autonomy-ceiling co-ownership vs sole-governance; S3 safety-gate-set absence).
- **AI-slop:** 2 (A1 erp ~150x repeated acceptance boilerplate; A2 21-file supervisor cluster unaudited / benchmark-fabrication risk).
- **Refinements:** 5 (taxonomy regen; wave-vocab reconcile; PARC-naming; template fix; tenant_class = good pattern).
- **False-positives explicitly cleared:** Palantir "Foundry Ontology" (`ontology.md:241`, competitor name — not our brand); workplace biometric (end-user FaceID — not canon #12 agent-biometric); Cedar (correct as contract, do NOT swap); `tenant_class` (already correct); W-* waves in products PRDs (already canon #9-compliant).
