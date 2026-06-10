# A.0-2 — ADR-0000+ CONSOLIDATION DESIGN-SET (cluster-level)

> **STATUS: PROVISIONAL — pending founder door:one-way freeze.** This is the Wave-0 PROVISIONAL design-freeze candidate (AMENDMENT-PLAN §A.0-2 / L1.0-CONS, R3/Finding-2). The founder signs the **design-level classification** (which old Accepted source ADRs are *intended* to fold into / be archived by which clean ADR-0000+ cluster), **NOT a guaranteed-final file map.** The file-level fold is the OUTPUT of Wave-1 authoring (L1.1) and is CONFIRMED-not-assumed by the Wave-1 **L1.1-CONS-CONFIRM** checkpoint; **any delta = re-review trigger that re-opens this door:one-way + revert-on-fold, NOT a silent amend** (AMENDMENT-PLAN.md:117-118,129,210-211).
>
> **Role of producer:** independent verifier/producer, READ-ONLY on `source/`. Every classification below is grounded in a primary source — the founder cluster rulings (`synthesis/decision-record-oyatie-canon.md` = SSOT) and the per-ADR disposition rows (`synthesis/01-ADR-DISPOSITION-TABLE.md`) + Proposed ledger (`synthesis/03-PROPOSED-RESOLUTION-LEDGER.md`). Citations are `decision-record:LINE` (SSOT) / `disposition:row-id` / `ledger:§`.
> **Date:** 2026-06-06.

---

## 0. What this partition is FOR (the Wave-0 contract)

This set is the **partition that tells every Wave-0 lane which files are *provisionally* throwaway-by-re-foundation vs live-and-amended** (AMENDMENT-PLAN.md:117):

- **ARCHIVED-by-refoundation** ⇒ the old ADR is `consolidates`/`supersedes`-folded into a clean ADR-0000+ doc and archived (immutability: archived, not deleted — `decision-record:43` D13; AMENDMENT-PLAN.md:33). **L2.2 SKIPS the foundry rename** on these files (the rename lands ONCE on the new ADR-0000+ text — AMENDMENT-PLAN.md:139). The re-foundation ADR (the ratifier) is authored in Wave-1.
- **AMENDED-in-place** ⇒ the ADR is NOT re-founded; de-foundried/de-stale via **L1-amend** (supersede-on-touch where the ADR is frozen-immutable). **The foundry rename lands here, in place, once** (AMENDMENT-PLAN.md:118).
- **superseded-on-cutover (pending build+proof)** ⇒ the **build-first-cutover-later** third state (`decision-record:26` D-META; AMENDMENT-PLAN.md:34,232). An ADR superseded by a **not-yet-built owned thing** is marked `superseded-on-cutover` and **stays OPERATIVE — NOT archived now** (Jenkins/Argo/data-bridges/Zitadel/Cedar-vendored-engine). For the Wave-0 rename partition these behave as **AMENDED-in-place** (rename in place; they are NOT thrown away), carrying a required `cutover_trigger:` field (AMENDMENT-PLAN.md:233). They are listed distinctly because misclassifying a bridge as ARCHIVED would retire it before its owned replacement is proven — the exact defect D-META forbids.

> **The throwaway-by-re-foundation set (skip-rename) is SMALL.** Only the ADRs being *consolidated into a clean ADR-0000+ ratifier whose owned replacement is authored now* are archived-by-refoundation. The endpoints that are still being BUILT (CI-owned-engine, oya-identity-as-shipped, PARC engine, framekernel) either already exist as Accepted bespoke ADRs (amended-in-place into the 0000+ series) or their vendored bridges stay operative (superseded-on-cutover). Most cluster members are **AMENDED-in-place**.

---

## 1. THE DESIGN-SET TABLE — re-foundation clusters

> Columns: **old ADR id+title | source status | SSOT/disposition disposition | TARGET ADR-0000+ cluster | classification (ARCHIVED-by-refoundation / AMENDED-in-place / superseded-on-cutover) | primary-source cite**.
> "TARGET ADR-0000+ cluster" names the *clean generative-series cluster* the SSOT rules it folds into (the literal 0000+ id is assigned at Wave-1 authoring / L1.0 MAP — `decision-record:43` D13; not pinned here, by design).

### CLUSTER A — oya-ci (D3; `decision-record:116-121`)

The clean ratifier reshapes ADR-0513 into ONE ADR-0000+ doc (`decision-record:121` "seed ONE clean ratifying ADR (ADR-0000+ series) reshaping 0513").

| old ADR | source status | disposition | TARGET 0000+ cluster | classification | cite |
|---|---|---|---|---|---|
| **0513** oya-ci bespoke-Rust Prow | Accepted (founder-locked) | KEEP→reshape into ratifier | **CI-RATIFIER** (the seed 0000+ ADR) | **ARCHIVED-by-refoundation** (its content IS reshaped into the seed; folds 0111/0116 already) | `decision-record:117-121`; `disposition:0513` |
| **0511** CI orchestration Argo Workflows | Proposed | RATIFY-or-ARCHIVE; reconcile w/ 0513 | **CI-RATIFIER** (supersede/relate) | **ARCHIVED-by-refoundation** (superseded/related INTO ratifier — "keep Argo's DAG/event IDEAS, drop its etcd-CRD substrate") | `decision-record:118,121`; `disposition:0511`; `ledger:§D` |
| **0124** own merge-queue (webhook GH-free) | Accepted (stale FM) | ARCHIVE (salvage projected-state→Tide) | **CI-RATIFIER** (supersede) | **ARCHIVED-by-refoundation** ("supersedes 0124 — file-overlap → graph-exact `conflicts(a,b)`") | `decision-record:118,121`; `disposition:0124` |
| **0408** Buck2-driven CI/CD | Proposed | AMEND→RATIFY (adopt as core substrate) | **CI-RATIFIER** (adopt as substrate) | **AMENDED-in-place** (adopted as the build-graph core — `decision-record:118` "adopts 0408/0514 as the core substrate"; substrate KEPT, not archived) | `decision-record:118`; `disposition:0408`; `ledger:§D` |
| **0514** build/CI/CD target architecture | Proposed | AMEND→RATIFY (author 0488 linker dep; microservices/→{oya,cloud}/) | **CI-RATIFIER** (adopt as substrate) | **AMENDED-in-place** (adopted as core substrate, repointed) | `decision-record:118`; `disposition:0514`; `ledger:§D` |
| **0369** gated stacked-trunk + speculative merge-train | Accepted | KEEP (phase) | **CI-RATIFIER** (phase-3, queue-depth-gated) | **AMENDED-in-place** (PHASED — speculation deferred to Phase-3; kept, re-sequenced) | `decision-record:118`; `disposition:0369` |
| **0367** trustless pre-merge verification gateway | Accepted | KEEP (phase) | **CI-RATIFIER** (phase-early, trustless gate) | **AMENDED-in-place** (PHASED — built early; kept) | `decision-record:118`; `disposition:0367` |
| **0366** agentic high-throughput self-repairing pipeline | Accepted | KEEP (phase) | **CI-RATIFIER** (phase-last, agentic self-repair) | **AMENDED-in-place** (PHASED — sequenced last; kept) | `decision-record:118`; `disposition:0366` |
| **0349** Jenkins(LTS)+ArgoCD | Proposed | **DROP Jenkins-half / KEEP ArgoCD** | (bridge) | **superseded-on-cutover** (Jenkins-half DROPPED per D14; ArgoCD-half stays operative until oya-ci proven — `decision-record:121` "Jenkins (0349/0359/0361)+Argo stay OPERATIVE until oya-ci built and proven → cutover → THEN retire"). Jenkins-half DROP is a `ledger` DROP, not a fold. | `decision-record:46,121`; `disposition:0349`; `ledger:§D,§F` |
| **0359** Jenkins replaces GHA | Superseded | ARCHIVE (Jenkins dead; principle→0511→ratifier) | (bridge) | **superseded-on-cutover** (Jenkins transitory; principle carried into ratifier; archived-when-proven, not now) | `decision-record:121`; `disposition:0359` |
| **0361** execute Jenkins-native CI/CD revamp | Proposed | AMEND-rehost→RATIFY (Jenkins→Argo/oya-ci) | **CI-RATIFIER** (rehost executor) | **superseded-on-cutover** (supply-chain stack durable+rehosted; Jenkins executor stays operative until cutover) | `decision-record:121`; `disposition:0361`; `ledger:§D` |

### CLUSTER B — identity (D5; `decision-record:31`)

| old ADR | source status | disposition | TARGET 0000+ cluster | classification | cite |
|---|---|---|---|---|---|
| **0476** oya-identity bespoke Rust | Accepted | AMEND (add supersedes:[0187]; fix phantom 0421; Cedar mis-cite 0083) | **IDENTITY-ENDPOINT** (owned endpoint) | **AMENDED-in-place** (the owned endpoint already Accepted; amended into the 0000+ series, NOT archived — it IS the endpoint) | `decision-record:31`; `disposition:0476`; `ledger:§D` |
| **0187** canonical OIDC IdP Zitadel | Accepted | SUPERSEDE/AMEND (Zitadel→bridge; mark superseded_by:0476) | **IDENTITY-ENDPOINT** (bridge under 0476) | **superseded-on-cutover** (Zitadel = vendored OIDC bridge, demoted from canonical→Phase-1 bridge; superseded-as-endpoint by 0476 but bridge RETAINED operative — `decision-record:31` "0187 status → superseded-as-endpoint/bridge-retained") | `decision-record:31`; `disposition:0187`; `ledger:§B` |
| **0506** aws-lc-rs crypto provider | Accepted (founder-locked, two-way) | KEEP (conditional on C-4→0476) | **IDENTITY-ENDPOINT** (reused behind port) | **AMENDED-in-place** (foundational crypto reused behind ports, "not differentiators" — `decision-record:31`; KEPT) | `decision-record:31`; `disposition:0506` |
| **0507** webauthn-rs relying party | Accepted (founder-locked, two-way) | KEEP (conditional on C-4→0476) | **IDENTITY-ENDPOINT** (reused behind port) | **AMENDED-in-place** (reused behind port; KEPT) | `decision-record:31`; `disposition:0507` |
| **0508** OpenSK authenticator reference | Accepted (founder-locked, two-way) | KEEP (conditional on C-4→0476) | **IDENTITY-ENDPOINT** (reused behind port) | **AMENDED-in-place** (reused behind port; KEPT) | `decision-record:31`; `disposition:0508` |

### CLUSTER C — policy (Cedar contract + PARC engine; D6; `decision-record:34`)

| old ADR | source status | disposition | TARGET 0000+ cluster | classification | cite |
|---|---|---|---|---|---|
| **phantom-0150-cedar** (the mis-anchored Cedar-engine decision) | PHANTOM (0150 is cursor-pagination, NOT policy-engine) | re-author the phantom Cedar-engine anchor; assign a REAL id | **POLICY-ENGINE (PARC)** (new owned-engine ADR) | **ARCHIVED-by-refoundation** (the *phantom anchor* is re-authored into a real ADR-0000+ PARC-engine doc — there is no real file to archive; the bad cross-refs are re-pointed) | `decision-record:34`; `disposition:0150` ("re-key the map: 0150≠policy-engine") |
| **0021** Foundry capability registry + MCP | Proposed | AMEND→RATIFY (re-home→intelligence) | **POLICY-ENGINE (PARC)** — *NOTE: 0021 is the linux compile-to-Rust PARC ref* | **AMENDED-in-place** (the owned PARC engine ref; `decision-record:34` "own the EVALUATION ENGINE behind it (compile-to-Rust PARC, linux 0021)") — KEPT/re-homed | `decision-record:34`; `disposition:0021` |
| **0007** Cedar RBAC/ABAC + autonomy ceiling | Proposed | AMEND (promote; dedupe vs 0002; +0379) | **POLICY-CONTRACT** (Cedar-contract refs retained) | **AMENDED-in-place** ("keep 0007/0183/0243/0246 Cedar-contract refs, retarget engine refs to owned PARC"); also see Cluster G (autonomy) | `decision-record:34`; `disposition:0007` |
| **0183** K8s policy-engine separation Cedar vs Kyverno | Superseded | ARCHIVE (separation principle survives; →0379 Kubewarden) | **POLICY-CONTRACT** (Cedar-contract ref retained; admission→0379) | **AMENDED-in-place** (Cedar-contract ref kept+retargeted; the admission half already superseded-on-disk to 0379) | `decision-record:34`; `disposition:0183` |
| **0243** Cedar as universal gate (keystone) | Proposed | AMEND+RATIFY (phantom 0150-cedar; Kafka→Pulsar) | **POLICY-CONTRACT** (Cedar-contract ref retained) | **AMENDED-in-place** (Cedar-contract kept; engine refs retargeted to PARC; phantom-0150 anchor fixed) | `decision-record:34`; `disposition:0243`; `ledger:§B` |
| **0246** policy-engine substrate promotion | Proposed | KEEP/RATIFY (phantom 0150; 0183→0379) | **POLICY-CONTRACT / POLICY-ENGINE** (engine ref retargeted) | **AMENDED-in-place** (kept; engine refs retargeted to owned PARC) | `decision-record:34`; `disposition:0246`; `ledger:§B` |

### CLUSTER D — isolation (D7; `decision-record:36-37`)

| old ADR | source status | disposition | TARGET 0000+ cluster | classification | cite |
|---|---|---|---|---|---|
| **0023** Foundry sandbox wasmtime+firecracker | Proposed | AMEND→RATIFY (brand; reconcile 0147/0200; assume-breach default affirmed) | **ISOLATION-ENDPOINT** (affirmed default) | **AMENDED-in-place** ("affirm ADR-0023 default" = assume-breach microVM fleet default holds — `decision-record:37`; KEPT, brand-scrubbed) | `decision-record:37`; `disposition:0023`; `ledger:§A` |
| **0147** container sandboxing runtime ladder | Amended | AMEND (reconcile body; post-amend still says gVisor-default) | **ISOLATION-ENDPOINT** (ladder reconciled) | **AMENDED-in-place** (body reconciled; ladder kept) | `decision-record:37`; `disposition:0147` |
| **0338** pod runtime tier 0..3 (Kata untrusted; runc first-party) | Proposed | RATIFY (vs LINUX-0023 default; foundry principal stale) | **ISOLATION-ENDPOINT** (the isolation-default founder call) | **AMENDED-in-place** (runc-for-first-party vs microVM-for-all is "the isolation-default founder call"; KEPT) | `decision-record:37`(implied via D7 ladder); `disposition:0338`; `ledger:§D` |
| *(Talos/Kata/Firecracker/wasmtime substrate ADRs — e.g. 0375/0382 node-OS)* | (various) | tag transitional bridges | **ISOLATION-ENDPOINT** (bridges under framekernel) | **superseded-on-cutover** (vendored bridges to the framekernel+owned-VMM committed endpoint; stay operative until kernel-level ratchet — `decision-record:37`) | `decision-record:37` |

> **Note (isolation endpoint):** framekernel + owned-VMM is the COMMITTED owned endpoint (`decision-record:37`), but it lives in the **linux** repo (L-0023/L-0025 ranges renumbered →0515+ at merge per D13 `:43`), NOT in the source 0000+ cluster. **L7 container build engine = DEFER_VENDORED** (BuildKit now). So this cluster's source-side action is mostly *tag bridges transitional + affirm 0023 default* — no source ADR is archived-by-refoundation here.

### CLUSTER E — autonomy (D16; `decision-record:39-40`)

| old ADR | source status | disposition | TARGET 0000+ cluster | classification | cite |
|---|---|---|---|---|---|
| **0022** autonomy ceiling: Cedar runtime enforcement per invocation | Proposed | AMEND→RATIFY (promote; foundry→governance) | **AUTONOMY-GATE** (governs; runtime hard gate) | **AMENDED-in-place** ("ADR-0022 governs — hard Cedar gate at every invocation; owned by `governance`"; promoted+re-homed, KEPT) | `decision-record:40`; `disposition:0022`; `ledger:§A` |
| **0007** Cedar RBAC/ABAC + persona-tier autonomy ceiling | Proposed | AMEND (advisory framing demoted) | **AUTONOMY-GATE** (advisory→design-time guidance) | **AMENDED-in-place** ("0007's advisory framing demoted to design-time guidance"; reconciled 0007↔0022, KEPT) — also Cluster C (policy-contract) | `decision-record:40`; `disposition:0007` |

> **Cross-cluster note:** 0007 appears in BOTH Cluster C (policy-contract refs retained) and Cluster E (autonomy advisory-demoted). It is a SINGLE AMENDED-in-place file touched by two rulings — not double-folded. L1.0-VERIFY surjection check (AMENDMENT-PLAN.md:120,212) must confirm 0007 maps to exactly one disposition (AMEND-in-place), with both rulings applied in one amendment.

### CLUSTER F — data-tier (+ Pulsar; D4/D-EVENT; `decision-record:20-22,107`)

| old ADR | source status | disposition | TARGET 0000+ cluster | classification | cite |
|---|---|---|---|---|---|
| **0005** eventing backbone Apache Kafka | Proposed (retired-in-fact) | ARCHIVE broker / SUPERSEDE (patterns survive); superseded_by 0377-kafka | **DATA-TIER / EVENTING** (Pulsar bridge) | **superseded-on-cutover** (broker retired-in-fact → 0377-kafka(→Pulsar); outbox/CloudEvents/partitioning sub-atoms carry forward via 0153; Pulsar = vendored bridge, owned engine = endpoint later — `decision-record:22,107`; NOT archived-by-refoundation, the streaming substrate stays operative) | `decision-record:22,107`; `disposition:0005`; `ledger:§A,§F` |
| **0377-kafka** Kafka→Pulsar | (the superseder) | canonical eventing/streaming bridge | **DATA-TIER / EVENTING** (Pulsar bridge) | **superseded-on-cutover** (Pulsar = canonical bridge; owned eventing engine = endpoint later via ratchet — `decision-record:107`) | `decision-record:107` D-EVENT |
| **0179** Postgres+Citus pool / **0192** Milvus / **0193** ClickHouse / **0194** TimescaleDB / **0196** SeaweedFS / **0337** Iceberg / **0336** Valkey | Accepted (mostly) | KEEP; tag as transitional vendored substrate behind ports | **DATA-TIER** (vendored substrates behind ports) | **superseded-on-cutover** ("transitional vendored substrates behind ports, NOT endpoints"; ratcheted to owned distributed multi-model engine WHEN proven — `decision-record:21-22` D4; bridges stay operative) | `decision-record:21-22`; `disposition:0179/0192/0193/0194/0196/0337/0336` |
| **0045** DB tier (Postgres+Citus) | Proposed | AMEND→RATIFY (repoint OLAP/pool/vector; "vendored OLTP until owned engine proves parity") | **DATA-TIER** (vendored OLTP substrate) | **superseded-on-cutover** ("Postgres+Citus is the vendored OLTP substrate UNTIL owned engine proves parity"; retained-for-now, KEPT operative — `decision-record:22`) | `decision-record:22`; `disposition:0045`; `ledger:§A` |

> **Note (data-tier):** the owned endpoint is the distributed multi-model engine (cloud-data / **linux L-0001**, renumbered →0515+ at merge per D13). NO source data-tier ADR is archived-by-refoundation; every best-of-breed pick is AMENDED-in-place-tagged-`superseded-on-cutover` (vendored-behind-port). The owned-engine endpoint ADR is authored fresh (Wave-2 L4/L5), not a fold of a source ADR.

### CLUSTER G — masterplan-wiring (D1/D15; `decision-record:9-16,48-49`)

| old ADR | source status | disposition | TARGET 0000+ cluster | classification | cite |
|---|---|---|---|---|---|
| **0364** generative ADR template; masterplan generated from ADR log | **Accepted** | KEEP (settles the FORK; apex authority) | **MASTERPLAN-WIRING** (apex; `domain` field added) | **AMENDED-in-place** (apex generator ADR; D15 adds the `domain` field to the 0364 template via a domain-cohesion meta-ADR — `decision-record:49`; KEPT, NOT archived) | `decision-record:10,49`; `disposition:0364` |
| **0365** automated ADR lifecycle | **Accepted** | KEEP (settles the FORK) | **MASTERPLAN-WIRING** (apex; cohesion gate added) | **AMENDED-in-place** (apex lifecycle ADR; D15 adds the `domain-cohesion` gate to the 0365 lifecycle via the meta-ADR; KEPT, NOT archived) | `decision-record:10,49`; `disposition:0365` |
| **decision-record D1** (the generator-status record) | (this audit's own SSOT) | supersede-record the generator `cutover_status:` | **MASTERPLAN-WIRING** (meta-ADR + supersede) | **AMENDED-in-place** (a paired supersede of `decision-record D1` records the generator `cutover_status:` field — AMENDMENT-PLAN.md:156; new meta-ADR, no source ADR archived) | AMENDMENT-PLAN.md:156; `decision-record:9-16` |

> **Note (masterplan-wiring):** 0364/0365 are the **apex** ADRs the whole generated-from-ADRs model rests on — they are NOT re-founded/archived. The "wiring" is a **NEW domain-cohesion meta-ADR + a masterplan-generated-wiring meta-ADR** (both Wave-2 L5.1, also tagged L1.1-shared — AMENDMENT-PLAN.md:156) that AMEND 0364/0365 by adding the `domain` field + cohesion gate. The new meta-ADRs are authored fresh in the 0000+ series; the source apex ADRs are amended-in-place, not archived.

---

## 2. CLUSTER → FOLDED-IDS COUNTS (the summary)

| cluster | old-ids in cluster | ARCHIVED-by-refoundation | AMENDED-in-place | superseded-on-cutover (bridge, stays operative) | SSOT ruling |
|---|---:|---|---|---|---|
| **A oya-ci** | 11 (0513,0511,0124,0408,0514,0369,0367,0366,0349,0359,0361) | **3** (0513, 0511, 0124) | 5 (0408, 0514, 0369, 0367, 0366) | 3 (0349-Jenkins-half DROP, 0359, 0361) | D3 `:116-121` |
| **B identity** | 5 (0476,0187,0506,0507,0508) | **0** | 4 (0476, 0506, 0507, 0508) | 1 (0187 Zitadel-bridge) | D5 `:31` |
| **C policy** | 6 (phantom-0150-cedar,0021,0007,0183,0243,0246) | **1** (phantom-0150-cedar anchor re-authored → real PARC id; no real file archived) | 5 (0021, 0007, 0183, 0243, 0246) | 0 | D6 `:34` |
| **D isolation** | 3 explicit (0023,0147,0338) + node-OS bridges | **0** | 3 (0023, 0147, 0338) | node-OS bridges (0375/0382-class) | D7 `:36-37` |
| **E autonomy** | 2 (0022,0007†) | **0** | 2 (0022, 0007†) | 0 | D16 `:39-40` |
| **F data-tier** | 9+ (0005,0377-kafka,0045,0179,0192,0193,0194,0196,0337,0336) | **0** | 0 | all (0005 broker + every best-of-breed pick = vendored-behind-port) | D4 `:20-22` / D-EVENT `:107` |
| **G masterplan-wiring** | 2 apex (0364,0365) + D1 record | **0** | 3 (0364, 0365, decision-record-D1) | 0 | D1 `:9-16` / D15 `:48-49` |

† 0007 is one file counted in both C and E (single AMENDED-in-place; two rulings applied in one amendment — not double-counted as two folds).

**Headline:** the **ARCHIVED-by-refoundation (skip-foundry-rename) set is only ~4 source files** — `{0513, 0511, 0124}` (oya-ci ratifier) + the `phantom-0150-cedar` anchor (no real file; re-authored to a real PARC id). **Everything else in the re-foundation clusters is AMENDED-in-place (rename in place once) OR superseded-on-cutover (vendored bridge, stays operative, rename in place).** This is the direct consequence of **build-first-cutover-later** (`decision-record:26` D-META; AMENDMENT-PLAN.md:34): you do not archive an ADR until its owned replacement is built and proven, so the only things archived NOW are those whose ratifier text is authored NOW (the oya-ci reshape) or that have no real file (the phantom).

---

## 3. COMPLETENESS NOTE (closure criterion)

**Every foundry-sense re-foundation candidate the task names is classified** (the L1.0-VERIFY closure criterion — AMENDMENT-PLAN.md:120,212: "every Accepted ADR carrying a foundry-sense term that is also a re-foundation candidate is classified consolidate-or-amend, none unclassified"):

- The 8 re-foundation clusters the task enumerates are ALL covered: oya-ci (A), identity (B), policy incl. Cedar+PARC + phantom-0150 (C), isolation (D), autonomy 0007/0022 (E), data-tier +0005 Pulsar (F), masterplan-wiring 0364/0365 (G).
- Every old ADR id the task names by number is present in §1 with an explicit classification: 0513/0511/0124/0369/0367/0366 (A); 0476/0187 (B); phantom-0150, Cedar 0007/0183/0243/0246, PARC 0021 (C); isolation bridges (D); 0007/0022 (E); 0005/0377-kafka (F); 0364/0365 (G).
- **No re-foundation candidate is left unclassified.** Each is exactly one of {ARCHIVED-by-refoundation, AMENDED-in-place, superseded-on-cutover}. The classification is at the **design-intent** granularity the SSOT rules (cluster-level fold INTENT), NOT a file-level fold map — that is Wave-1's L1.1-CONS-CONFIRM output (AMENDMENT-PLAN.md:117,129).
- **0007 surjection caveat (flagged for L1.0-VERIFY):** 0007 is the one id appearing in two clusters (C policy-contract + E autonomy). It is a single AMENDED-in-place file; the verifier's surjection check (AMENDMENT-PLAN.md:120) must confirm it maps to ONE disposition with both rulings folded into one amendment — not two competing folds.

---

## 4. AMBIGUOUS CLASSIFICATIONS NEEDING FOUNDER INPUT (at the door:one-way freeze)

These are the design-level classifications where the SSOT is either silent on the *fold target* or where the ARCHIVE-vs-amend call is genuinely founder-bounded (raised for the founder at sign-off, NOT resolved by the producer):

1. **A / 0408 + 0514 — "adopt as core substrate" vs "fold into ratifier."** D3 (`:118`) says oya-ci "adopts 0408/0514 as the core substrate." Classified AMENDED-in-place (substrate KEPT, repointed). **Founder call:** do 0408/0514 stay as standalone substrate ADRs that the ratifier *references*, or are they CONSOLIDATED into the ratifier text (which would make them ARCHIVED-by-refoundation)? The SSOT rules adoption, not the fold mechanism. (Disposition rows treat both as AMEND→RATIFY, supporting AMENDED-in-place — the provisional call here.)

2. **A / 0349 Jenkins-half — DROP vs superseded-on-cutover.** D14 (`:46`) DROPs the Jenkins-half; D3 (`:121`) keeps Jenkins OPERATIVE-until-cutover. These are reconcilable (DROP the *decision to canonicalize Jenkins*; keep the *running Jenkins bridge* until oya-ci proven) but the founder should confirm the Jenkins-half DROP does not strand the operative bridge. Provisionally: ArgoCD-half + operative Jenkins = superseded-on-cutover; the Jenkins-*canonicalization* claim = DROP (L1.2-DROP inbound-safety gate, AMENDMENT-PLAN.md:131).

3. **C / phantom-0150-cedar — which real id.** D6 (`:34`) says "assign the cedar-engine decision a real id." Classified ARCHIVED-by-refoundation (re-authored into a fresh PARC-engine 0000+ ADR). **Founder/Wave-1 call:** the literal target id is assigned at L1.0 MAP / Wave-1 authoring; the design intent (a real owned-PARC-engine ADR exists, 0150 stays cursor-pagination) is what is frozen here.

4. **F / data-tier owned-engine endpoint home.** The owned distributed multi-model engine endpoint (D4 `:21`) is authored fresh (cloud-data / linux L-0001 renumbered →0515+). **Founder call:** is the owned-engine endpoint ONE new 0000+ ADR that the vendored-substrate ADRs point UP to, or does it live in the linux-merged 0515+ range only? This affects whether any source data-tier ADR is later re-classified — but at Wave-0 NONE is archived-by-refoundation, so it does not change this freeze.

5. **G / masterplan-wiring — meta-ADR count.** D15 (`:49`) adds the `domain` field + cohesion gate; AMENDMENT-PLAN.md:156 names a domain-cohesion meta-ADR AND a masterplan-generated-wiring meta-ADR. **Founder call:** one combined meta-ADR or two? Either way 0364/0365 are AMENDED-in-place (apex, not archived) — this is a count question, not a classification question.

> **None of the 5 ambiguities changes the Wave-0 throwaway-vs-amend partition** (the skip-rename set stays `{0513, 0511, 0124, phantom-0150}`). They are fold-*mechanism* / target-*id* / meta-ADR-*count* questions the founder resolves at the door:one-way freeze and that Wave-1 L1.1-CONS-CONFIRM then verifies against the authored ADRs (delta = re-review trigger, AMENDMENT-PLAN.md:129).

---

## 5. PROVENANCE / VERIFICATION

- **SSOT cluster rulings verified verbatim:** D3 oya-ci (`decision-record-oyatie-canon.md:116-121`), D4 data-tier (`:20-22`), D5 identity (`:31`), D6 policy (`:34`), D7 isolation (`:36-37`), D16 autonomy (`:39-40`), D1 masterplan-authority (`:9-16`), D15 domain-enum (`:48-49`), D-EVENT Pulsar (`:107`), D-META build-first-cutover-later (`:26`).
- **Per-ADR dispositions verified** against `synthesis/01-ADR-DISPOSITION-TABLE.md` (rows 0005/0007/0021/0022/0023/0045/0124/0147/0150/0179/0183/0187/0192/0193/0194/0196/0243/0246/0335/0336/0337/0338/0347/0349/0359/0361/0363/0364/0365/0366/0367/0369/0374/0376/0377/0408/0476/0506/0507/0508/0510/0511/0513/0514) + Proposed ledger `synthesis/03-PROPOSED-RESOLUTION-LEDGER.md` §A/§B/§D/§E/§F.
- **Three-state classification grounded** in AMENDMENT-PLAN.md §A.1 principles 2-3 (`:33-34`): supersede-never-edit immutability (archived, not deleted) + build-first-cutover-later (`superseded-on-cutover (pending build+proof)`, NOT archived now).
- **Provisional-freeze model grounded** in AMENDMENT-PLAN.md:117-118,129,210-211 (R3/Finding-2): Wave-0 freezes design intent; Wave-1 L1.1-CONS-CONFIRM verifies file-level fold; delta = re-review trigger + revert-on-fold.
- **No source file was edited** producing this artifact (READ-ONLY on `source/` honored; this is the single named output under `_execution`).
