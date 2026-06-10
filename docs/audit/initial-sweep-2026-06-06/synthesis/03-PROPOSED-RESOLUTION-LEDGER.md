# 03 — PROPOSED-RESOLUTION LEDGER (no unaccounted proposals)

> **The decision-debt gate.** Every SOURCE ADR carrying `status: proposed`/`Proposed` (or a status-graph that resolves to Proposed-in-fact) gets ONE verdict here: **RATIFY** or **DROP**, with rationale + **door-class** (`one-way` ⇒ founder sign-off required; `two-way` ⇒ auto-on-green / reversible). Built by merging the three partials' "Proposed (ratify/drop)" sections over the disposition table.
> **Founder principle:** "a decision not represented as a LIVE planning_impact ADR is *not needed*." Under the ratified generated-from-ADRs model (ADR-0364/0365), an unresolved `Proposed` silently **drops a true decision** from the generated masterplan — so this ledger is a precondition for backfill, not a nicety.
> **Door-class rule (from ADR-0364/0365 + corpus convention):** founder-locked / `door: one-way` ⇒ founder sign-off; reversible substrate/vocab/process ⇒ `two-way` ⇒ ratify-on-green by the council/verifier lane. The crypto cluster (0506/0507/0508) is explicitly authored `door: two-way, planning_impact:false`.
> **Net:** **132 Proposed on disk; 133 ledger rows incl. 1 Superseded cross-ref (0170→0394)**. Of the 132 true Proposed: **RATIFY ≈ 122 · DROP 3 · AMEND-MANDATORY 1 · RENUMBER-then-RATIFY 1 · KEEP-as-Proposed (by design) 1**. The 3 true DROPs are 0325 (prices a retired primitive), 0349 (Jenkins-half only), 0316 (superseded by 0329). (ADR-0352 is AMEND-MANDATORY, not a DROP.) **Zero unaccounted proposals.**

---

## 0. Door-class legend & resolution counts

| Door-class | Meaning | Resolution authority |
|---|---|---|
| **one-way** | irreversible / founder-locked / changes a load-bearing invariant or external commitment | **founder sign-off** (the FOUNDER DECISIONS gate, §E) |
| **two-way** | reversible substrate pick, vocab scrub, process slim, amendment-of-accepted | council + verifier lane, auto-on-green |

| Verdict | Count | Notes |
|---|---:|---|
| RATIFY (after amend) | ~118 | dominant; mostly vocab/cite/brand scrubs over sound atoms |
| RATIFY (promote, clean) | ~8 | 0004/0037/0040/0049/0276/0312/0340/0347 — minimal amend |
| DROP | 3 | 0325 (table), 0316 (superseded), 0349 (Jenkins-half) |
| RENUMBER then RATIFY | 1 | 0377-forge (off the 0377-kafka collision) |
| AMEND-MANDATORY (RATIFY-or-regenerate) | 1 | 0352 (handoff; the §4 fork made concrete) |
| KEEP-as-Proposed (by design) | 1 | 0134 (honest non-binding backlog — ratify only when items land) |

---

## A. FOUNDATION cluster (0002–0050) — `proposed` front-matter is status-drift on canonical decisions

These are load-bearing substrates that read `proposed` only because the Foundation cluster never had its status resolved. Promote.

| ADR | verdict | door | rationale |
|---|---|---|---|
| 0002 tenant+identity kernel | RATIFY (promote) | one-way | MASTERPLAN FD-001 already declares this at production depth; `proposed` is pure drift. Fix `KCminimum-shippable-tier` garble + foundry comment first. |
| 0003 audit chain substrate | RATIFY (promote) | two-way | load-bearing cohesion substrate; decision excellent; repoint transport 0005→0377, Q4 owner foundry→intelligence. |
| 0004 plane separation | RATIFY (promote, clean) | two-way | current, correct, non-conflicting; industry-canonical 3-plane model. |
| 0005 eventing backbone Kafka | **resolved-by-supersession** (broker DROP / patterns survive) | one-way | retired-in-fact by 0377-kafka-to-pulsar; mark `superseded_by:[0377-kafka]`; outbox/CloudEvents/partitioning sub-atoms carry forward (0153). |
| 0007 Cedar + autonomy ceiling | RATIFY (promote) | one-way | canonical, reaffirmed by 0243/0246; add 0379 admission cross-ref; fix foundry names; dedupe Cedar ownership vs 0002. |
| 0009 cells | RATIFY (after amend) | two-way | regulator-driven blast-radius primitive; 0008/0010 already depend on it; amend retired foundry/Kafka/Envoy mis-cite. |
| 0010 regional packs | RATIFY (after amend) | one-way | founder-aligned parallel-market keystone; fix Cosign-signing 0013→0039 mis-cite. |
| 0013 license policy (3-tier OSI) | RATIFY | one-way | keystone names it a *governing* license ADR; add AGPL server-substrate carve-out. (License posture = founder commitment.) |
| 0014 build-vs-buy policy | RATIFY (protocol; matrix=living appendix) | one-way | source-side own-when-proven keystone; matrix rows stale (Kafka/Redis/foundry) → re-derive, never freeze. |
| 0016 wave/plane framework | RATIFY (after amend) | two-way | wave-name principle settled + CI-enforced; amend dead wave table (W-Foundry-Preview→intelligence). |
| 0019 doc-catalog/SSOT machinery | RATIFY (after amend, +promote) | two-way | sound; reconcile with planning-ssot-consolidation + ADR-0364. |
| 0020 Foundry provider-adapter | RATIFY (re-home) | two-way | ProviderAdapter decision TRUE; re-home foundry→intelligence; scope-down programmatic subscription-auth (ToS risk). |
| 0021 Foundry capability registry + MCP | RATIFY (re-home) | two-way | decision TRUE, MCP-bet aged well; re-home foundry→intelligence; unify MCP endpoint shape. |
| 0022 autonomy ceiling (Cedar per-invocation) | RATIFY (promote) | one-way | already live GLOSSARY authority; promote Proposed→Accepted; rename foundry→governance/intelligence. (Autonomy axis = load-bearing.) |
| 0023 Foundry sandbox (wasmtime+firecracker) | RATIFY (after amend) | two-way | aligned (AWS Firecracker); brand scrub; reconcile w/ 0147/0200 ladder; note vs LINUX-0023 id-collision. |
| 0024 Foundry eval harness + replay | RATIFY (brand-amend) | two-way | strongest in its chunk; closed its own open Qs; only foundry vocab dead. |
| 0025 Foundry one-axis eng-platform | RATIFY-with-MAJOR-amend (or split into 2) | two-way | substrate mechanisms survive; headline "one axis" REVERSED by 0335 (intelligence+governance kept separate); fix tautology garble. |
| 0026 in-house AI model substrate | RATIFY (brand-amend) | one-way | Bedrock pattern, eval-gated, correctly NOT frontier; brand + W-name vocab only. (Long-horizon capex bet = founder-aware.) |
| 0027 robotics/vision/speech | RATIFY (scope-caveat) | one-way | safety design strong; scope-gate SC4-autonomous-actuation + public-road AV to a future ADR (breadth). |
| 0032 DCIM + anti-silicon | RATIFY (split) | one-way | keep+promote anti-silicon anti-scope (soften "never"→"not day-0"); defer build-in-house DCIM to 0028 Phase-2. |
| 0035 workflow engine (FSM+DAG, reject Temporal) | RATIFY (after re-justify own-vs-adopt) | one-way | sound hybrid; re-examine "reject Temporal" against the ratchet; adopt-then-own the overlay. (Own-too-early scope call.) |
| 0036 plugin substrate (wasmtime+WASI P2) | RATIFY | two-way | reinforces canonical wasmtime (0200); rebrand foundry→intelligence; split marketplace-econ to a business doc. |
| 0037 public API stability tiers | RATIFY/KEEP (clean) | two-way | doctrine-grade API governance; swap axis-*/Foundry owner labels only. |
| 0038 trust framework (DSR/proof-of-erasure) | RATIFY (amend) | two-way | sound, non-conflicting; fix "all all" typo + Foundry axis + dead 0042 ref. |
| 0039 supply-chain security | RATIFY (amend) | two-way | best-in-class; demote Kyverno→Kubewarden-adapter (0379); abstract CI off `.github`. |
| 0040 progressive delivery (Argo Rollouts) | RATIFY (minor amend, clean) | two-way | cleanest in chunk; only foundry/0042 refs. |
| 0041 GitOps (trunk + merge queue) | RATIFY (forge-neutral amend) | one-way | trunk process TRUE; abstract branch/merge-queue off GitHub YAML to forge-neutral. (Forge-coupled ⇒ gated on the forge ruling.) |
| 0043 secrets (OpenBao/HSM/KCMVP) | RATIFY (URGENT text fix) | two-way | sound; restore KCMVP token (×12 garbage) before anything else. |
| 0044 service mesh (Istio Ambient) | RATIFY (minor amend; promote to canon) | two-way | sound + no superseder; rename foundry-cell-id; framing folds under 0148. |
| 0045 DB tier (Postgres+Citus) | RATIFY (after amend) | one-way | keep Postgres+Citus OLTP atom; re-point OLAP/pool/vector to 0193/0179/0192; fix "Citus=Apache-2" license claim. (Data-tier = founder fault-line.) |
| 0047 search backend (pgroonga→Tantivy) | RATIFY-leaning / **ARCHIVE-the-pgroonga-pin** | two-way | pgroonga contradicted by Accepted 0184 (Meilisearch); keep Tantivy/SSPL-forbidden, archive pgroonga core. |
| 0048 KR tokenization | RATIFY (re-home) | two-way | trait principle uncontested but orphaned by 0047 drift; re-home onto Meilisearch/Tantivy. |
| 0049 cross-region residency | RATIFY (promote, clean) | one-way | strongest, regulator-grounded, uncontradicted; reads accepted-grade. (Residency = founder/regulatory commitment.) |
| 0050 automation-first pipeline | RATIFY (after amend) | two-way | doctrine uncontested; strip Foundry brand, Bazel→Buck2, remove leaked ~/.claude path. |

## B. KEYSTONE bundle (0242–0252) + the policy/governance Proposeds

| ADR | verdict | door | rationale |
|---|---|---|---|
| 0242 oyatie-is-a-tenant | RATIFY | one-way | load-bearing tenancy spine; governs (amends 0136/0220/0239); generates /specs/*.json. |
| 0243 Cedar universal gate | RATIFY (after amend) | one-way | THE authz keystone; AMEND phantom 0150-cedar anchor + Kafka→Pulsar + in-place-amend process. |
| 0244 tenant universal scoping | RATIFY (after amend) | one-way | scoping spine; AMEND tenants.tier-column→tenant-class BEFORE ratify so /specs/tenant-model.json is clean. |
| 0245 substrate-vs-product layering | RATIFY (after amend) | one-way | dep-direction CI spine; foundry-namespace scrub. |
| 0246 policy-engine substrate | RATIFY | one-way | AVP-pattern; phantom 0150-cedar + 0183→0379 + Postgres data-tier note. |
| 0247 self-hosting/self-modification | RATIFY | **one-way (CRITICAL)** | dogfood doctrine sound; the self-mod ceiling (can autonomous workflows author ADRs/policy-root?) is a **door:one-way founder gate** — cap below the governance-artifact layer. |
| 0248 Amazon-shape cellular | RATIFY | two-way | shuffle-sharding math correct; kubeadm/0121 + foundry + cell-µsvc stale. |
| 0249 multi-category marketplace | RATIFY-doctrine / **DEFER-build** | one-way | substrate-vs-surface pattern sound; the day-one build of all 8 substrates is the largest kernel-first mismatch — gate the build. |
| 0250 build-ahead-of-certification | RATIFY | one-way | Apple-Pay/Stripe build-ahead pattern; Kyverno/0183 admission cite stale; matrix→specs. |
| 0251 compliance pack + cell cert-levels | RATIFY | one-way | Assured-Workloads pattern; 0183/0150 admission cites stale; matrix→specs. |
| 0252 time/coordination/consistency | RATIFY | one-way | HLC/TrueTime/sagas canonical; reconcile w/ LINUX-0006 Clock-port; Kafka/0005 + oya-git framing stale. |
| 0255 intelligence two-layer | RATIFY (merge-adjacent w/ 0335) | one-way | canonical Intelligence posture; re-cite 0249→0335; confirm 0255 re-binds as amendment-of-Accepted-0335. |
| 0257 ontology object-type versioning | RATIFY (amend) | two-way | schema-versioning fix correct; Kafka→Pulsar. |
| 0263 observability emission | RATIFY (amend) | two-way | emission contract correct; re-point amends:0042→0383. |

## C. RIGHTS / SAFETY / DOCTRINE cluster (0272–0328) — regulator-anchored doctrine

| ADR | verdict | door | rationale |
|---|---|---|---|
| 0272 cookie consent / CMP | RATIFY (amend) | one-way | privacy-by-default correct; fix 0246→0003/0028 audit-chain cite. (Build-vs-buy CMP = founder scope.) |
| 0273 DKIM/SPF/DMARC | RATIFY | two-way | per-tenant deliverability Tier-1; amends 0201. |
| 0276 GDPR Art-20 portability | RATIFY (clean) | one-way | regulator-grounded; cross-tenant-restore scope = the only sub-question. |
| 0280 substrate-DAG doctrine | RATIFY | one-way | DAG-spec-as-authority = the cleanest worked example of generated-masterplan; Tarjan=BLOCKER. |
| 0284 platform-owner-name indirection | RATIFY | two-way | one-constant rebrand ceremony; amends 0242. |
| 0292 minor-user doctrine | RATIFY | one-way | COPPA/KOSA/EU age — B2C ship-blocker; amends 0007/0099/0218/0251. |
| 0293 meta-trust-root (self-mod witness) | RATIFY (mechanism) / rename-required | one-way | foundry in TITLE; the ICANN-grade multi-jurisdiction HSM ceremony to gate *internal* self-mod is a founder scope call (over-scoped for pre-GA?). |
| 0294 Cedar fragment soak/rollback | RATIFY | two-way | shadow-mode + EWMA auto-revoke; phantom 0150-cedar + 0183 cite. |
| 0295 bootstrap-CA + T+8h kill | RATIFY (after amend) | two-way | closes F5-247-02; gates 0247; de-foundry; soften rigid ceremony. |
| 0296 credential-sidecar | RATIFY (after amend) | two-way | closes F5-255-01; gates 0255; residual foundry. |
| 0297 abuse-defence baseline | RATIFY (after amend) | two-way | Cloudflare/WAF-aligned; foundry-fitness BNF→governance; Cloudflare-lock note. |
| 0298 emergency-services bypass | RATIFY (after amend) | one-way | life-safety hard rule (Apple SOS/Android ELS); fix 3-way name/body/BNF foundry inconsistency. |
| 0299 account-recovery resilience | RATIFY (minor amend) | two-way | ≥2-factor / never-permanent-lockout; foundry-fitness BNF leak. |
| 0300 whistleblower/anonymity | RATIFY (amend + **narrow scope**) | one-way | reframe from universal substrate → per-pack/audience opt-in; sequence after kernel. (Heaviest cloud-tier build.) |
| 0301 survivor-safety mode | RATIFY (amend + **narrow scope**) | one-way | per-account opt-in not universal; sequence after kernel. |
| 0302 deceased-user inheritance | RATIFY (amend) | two-way | foundry-fitness BNF. |
| 0303 cognitive-impairment doctrine | RATIFY (amend) | two-way | foundry-fitness. |
| 0304 cross-jurisdiction conflict (cluster keystone) | RATIFY (amend) | **one-way** | ratify FIRST in this cluster; the CLOUD-Act refuse-vs-surface posture is a founder legal-strategy commitment. |
| 0305 delegated-agent authority chain | RATIFY (amend) | two-way | oyatie.foundry.* WRONG/STALE. |
| 0306 disaster-mode/cell-resilience | RATIFY (amend) | two-way | foundry-fitness; reconcile 0375. |
| 0307 detection substrate (DRMP "D") | RATIFY (amend) | one-way | MANDATORY Kafka→Pulsar + Redis→Valkey rewrite first; 8-family day-0 breadth = founder scope. |
| 0308 ML model lifecycle | RATIFY (amend) | one-way | pair w/ 0307; EU-AI-Act/NIST/ISO-42001; foundry. |
| 0309 detection fairness baseline | RATIFY | one-way | 5-invariant civil-rights gate; foundry-BNF residue. |
| 0310 investigation case-management | RATIFY | two-way | Merkle chain-of-custody; Kafka→0377, Redis→0336. |
| 0311 dual-tenant identity | RATIFY | one-way | personal-vs-work Cedar boundary; amends 0244. |
| 0312 court-warrant scoped piercing | RATIFY (clean) | one-way | the only path through the personal boundary; warrant-canary. |
| 0313 conglomerate-tenant hierarchy | RATIFY | two-way | amends 0244; fix 0046 mis-cite. |
| 0314 marketplace deal-settlement | RATIFY core + AMEND | one-way | collapse §D-X blocks; the M&A/JV/receivables over-claim is a **contested founder scope question**. |
| 0315 ERP SAP-parity doctrine | **RATIFY-WITH-CONDITION** | **one-way** | ratify "ERP via composition, no monolith"; **gate the 9-new-µsvc authorization behind explicit founder go/no-go** (highest-stakes scope; a Proposed doctrine ADR must not silently green-light 9 multi-quarter builds). |
| 0317 role-based projection | RATIFY (amend) | two-way | stale 0316 absent-file disclaimer. |
| 0318 collar-color universality | RATIFY (amend) | two-way | stale 0317 disclaimer; length-cap vs 0322. |
| 0319 front/middle/back-office barriers | RATIFY (amend) | one-way | Chinese-Wall Cedar entities; extends 0244/0243. |
| 0320 transient identity | RATIFY (amend) | two-way | "capability tier" overload vs 0316/0329; amends 0244/0311/0313. |
| 0321 B2B leader coverage doctrine | RATIFY (amend) | one-way | "tier" vocab + volatile counts; re-anchor 0316→0329; 13-anchor breadth = founder scope. |
| 0322 substance bar doctrine | RATIFY | one-way | the SSOT-integrity gate that makes masterplan-from-ADRs trustworthy; one "foundry pipeline" slip. |
| 0323 multi-wave sequencing | RATIFY (amend/slim) | one-way | wave discipline useful; strip foundry/VCS; the ADR-vs-standards-doc home is a founder call. |
| 0324 anti-script/anti-template | RATIFY (amend) | two-way | this audit's own discipline; drop crypto-attestation theater. |
| 0325 capability-tier pricing anchors | **DROP (table) / re-author principle** | one-way | prices the RETIRED tier ladder (0329 killed 0316); WRONG-now. Re-author public-self-serve principle against tenant-class (0330). |
| 0326 data-residency attestation | RATIFY (light amend) | one-way | residency tiers survive tier retirement; re-point pricing 0325→0330. |
| 0327 Wave-3 completion criteria | RATIFY (amend/slim) | **one-way** | the ADR state-machine IS the masterplan-generation backbone; adopt KEP-lite; strip foundry/VCS. (Votes generated-from-ADRs — ties to the FORK.) |
| 0328 substance-bar sequence | RATIFY-spine (large amend) | one-way | 5-phase substrate-before-product TRUE; retired tier deliverable (0329); drop Codex-only/oya-vcs; local-path leak. (Invariant vs one-time SOP = founder.) |

## D. SUBSTRATE-CHOICE Proposeds (2026-05-21 wave + CI/CD + library-first + reversals)

| ADR | verdict | door | rationale |
|---|---|---|---|
| 0336 Valkey | RATIFY (promote) | one-way | keystone-true + founder directive ("license drift = hard stop"); status-drift only. (Substrate canon.) |
| 0337 Iceberg OLAP table-format | RATIFY | one-way | clean; ClickHouse compute on Iceberg. |
| 0338 pod runtime tier 0–3 | RATIFY | one-way | runc-for-first-party vs LINUX-0023 microVM-for-all = **the isolation-default founder call**; oyatie.foundry.* principal stale. |
| 0339 shared IaC module library | RATIFY (amend on-prem) | two-way | OpenTofu primitives; kubeadm/Istio on-prem→Talos (0375). |
| 0340 capacity model per manifest | RATIFY (clean) | two-way | per-tenant capacity block. |
| 0341 cellular promotion gates | RATIFY (fix direction) | one-way | gates TRUE; the Tier 0→1 promote-vs-demote direction needs a founder one-liner. |
| 0342 API versioning hybrid | RATIFY | two-way | most conventional; re-cite 0316→0329. |
| 0343 DR + RTO/RPO matrix | RATIFY | two-way | fix MAX/MIN phrasing landmine. |
| 0344 sustainability/finops | RATIFY (amend) | two-way | soften "electricityMaps canonical"→default+fallback; register under 0345. |
| 0345 OSS stewardship + CVE SLA | RATIFY | two-way | keystone supply-chain doctrine; de-stale Kafka/Kyverno registry seeds. |
| 0346 oya verify CI mirror | RATIFY (amend) | two-way | true engineering invariant; scrub dead oya-foundry-fitness-* lanes; decouple .github/workflows. |
| 0347 foundry-fitness → governance rename | RATIFY | two-way | THE rename mechanism a keystone declares true; leaving it Proposed is why 0346 still cites the dead prefix. |
| 0348 autosharding/rebalance | RATIFY (amend) | two-way | lock ownership to 0351; stop oscillating vs 0333. |
| 0349 Jenkins(LTS)+ArgoCD | **DROP Jenkins-half / KEEP ArgoCD-CD** | one-way | superseded-in-fact by 0349→0359→0408→0511→0513; Jenkins = bootstrap-only, Argo Workflows + oya-ci destination. (The one DROP-half in the CI chain.) |
| 0352 from-scratch handoff | **AMEND-MANDATORY → RATIFY OR archive-as-bootstrap+regenerate** | **one-way** | wanted artifact but largest retired-vocab drift magnet (5 stack axes WRONG); IS the §4 authored-vs-generated question in concrete form — needs founder ruling. |
| 0353 library-first policy-engine | RATIFY (amend) | two-way | amends 0246; allocate dangling credential-sidecar id. |
| 0354 HTTP/3/TLS/ECH/PQC | RATIFY (one fix) | two-way | amends 0253; replace foundry/Hermes row + 0121 ref. |
| 0355 library-first intelligence | RATIFY (amend) | two-way | amends 0255; Redis/KeyDB→Valkey; axis-foundry; Kafka 0050→0377. |
| 0356 library-first ontology read-path | RATIFY (amend) | two-way | amends 0257; recovers 0141 intent; Kafka→0377; dangling sidecar ref. |
| 0357 vertical-slice monorepo nesting | RATIFY | two-way | resolves 0131 gap; later superseded-by 0512 (note edge). |
| 0358 ideal-roadmap (Bazel/strangler) | RATIFY (carve §2) | one-way | strangler-fig/define-100/masterplan-binding survive; §2 Bazel→Buck2 (0392/0408); §4 masterplan authority vs 0364 = founder. |
| 0360 CI/CD optimization program | RATIFY (rebind) | two-way | 7 correctness rules wanted; rebind O1/O3→Buck2 RBE, O6→Tide. |
| 0361 Jenkins-native supply-chain revamp | RATIFY (rehost) | two-way | stack durable; rehost executor Jenkins→Argo/oya-ci; Kyverno→Kubewarden. |
| 0377-forge forgejo-board git-ref CAS | **RENUMBER off 0377 → RATIFY-conditional** | one-way | renumber (0377-kafka Accepted holds the number); ratify on D2/D3 tests; reconcile reject-GitHub-Projects vs founder GitHub directive. |
| 0381 Kaniko→BuildKit + multi-node Talos cells | RATIFY (amend) | two-way | Kaniko archived-2024 → BuildKit (Apache-2, what Docker/GHA/Cloud-Build use); multi-pool Talos cells = GKE/EKS pattern; depends 0378. Amend the Jenkins-agent framing → oya-ci (0511/0513). |
| 0382 bare-metal Talos Sidero | RATIFY | two-way | clean. |
| 0384 LLM gateway OAuth pool | RATIFY (amend) | one-way | scope-down programmatic consumer-subscription auth (ToS/fragility — the one sub-decision a hyperscaler rejects). |
| 0387 CI webhook gateway → GitHub status | AMEND/SUPERSEDE | one-way | dead 0112 cite; vs 0374 Forgejo sink — reconcile to ONE sink off the forge ruling. |
| 0392 Buck2 build graph | RATIFY (promote) | one-way | keystone + 0358 front-matter already treat it governing; tighten supersedes to §2-only; schedule deferred spec regen. |
| 0394 bespoke-Rust IDP hub | RATIFY (amend) | one-way | real reversal (Backstage→bespoke); disambiguate IDP-portal vs IdP-OIDC; bind OIDC issuer→0476. |
| 0408 Buck2-driven CI/CD | RATIFY (amend) | one-way | re-point orchestrator off retired Jenkins/0359 → Argo Workflows/0511. |
| 0510 SCM bespoke-VCS destination | RATIFY (amend) | **one-way** | destination+numeric-trigger discipline is right; THE forge fault-line — gated on the forge ruling. |
| 0511 CI orchestration Argo Workflows | RATIFY-or-ARCHIVE | **one-way** | reconcile w/ Accepted 0513 oya-ci (they name different destinations) — founder picks the CI node. |
| 0514 build/CI/CD target architecture | RATIFY (conditional) | two-way | author the unwritten ADR-0488 linker dep; repoint microservices/→{oya,cloud}/; reconcile spine vs 0511. |

## E. SPECIAL CASES

| ADR | verdict | door | rationale |
|---|---|---|---|
| 0111 merge-queue projected-state | SUPERSEDE/MERGE into Tide (fix front-matter NOW) | two-way | algorithm valuable; formally supersede-into-Tide (0363/0513); re-target planning_impact to cloud-ci. |
| 0114 canary observability gate | DROP impl / salvage principle | two-way | canary principle survives → re-issue against Argo-Rollouts (0040/0511); impl superseded. |
| 0134 portfolio remediation backlog | **KEEP-as-Proposed (by design)** | two-way | correct honest-backlog shape; refuses self-certification; ratify only when items land. (NOT decision-debt — intentional.) |
| 0170 Backstage dev portal | **Superseded cross-ref** (not a Proposed resolution — real status on disk: `Superseded` by ADR-0394) | — | Not a Proposed ADR; included for traceability only. Do not action as a RATIFY/DROP candidate. When 0394 is ratified, 0170's supersession is already recorded; no further ledger action needed. |
| 0213 Ecosystem-as-a-Service | RATIFY (amend) | one-way | self-declares "Accepted upon PR #143"; scrub Backstage/0110; reclassify KYC/payout Class-B (in-house KYC = founder scope). |
| 0214 cross-tenant consent-graph | RATIFY (amend) | two-way | "Accepted upon PR #143"; fix Citus/Kafka/0183 cites. |
| 0236 OP-11 anti-aspirational-enforcement | RATIFY-and-generalize | two-way | its content IS the governance the goal depends on; leaving it Proposed is itself the unaccounted-proposal smell; DROP only if planning-ssot-coverage already absorbed it. |
| 0253 network topology | RATIFY (amend) | one-way | topology sound; Kafka→Pulsar; own-edge = founder scope. |
| 0254 deployment-model spectrum | RATIFY (amend) | one-way | five-model spectrum + single-build correct; fix 0249 WRONG-refs + foundry-builder. |
| 0316 capability-tier | **DROP-as-superseded (ARCHIVE)** | two-way | `superseded_by:[0329]`; projection mechanism survives under tenant-class; tier naming retired. |

---

## F. ACCOUNTING — every Proposed accounted for

- **Foundation cluster (§A):** 35 entries — RATIFY/promote (0005 resolved-by-supersession; broker drops, patterns survive).
- **Keystone bundle (§B):** 14 — all RATIFY.
- **Rights/safety/doctrine (§C):** 38 — RATIFY (0325 DROP-table; 0300/0301 narrow-scope; 0315 conditional).
- **Substrate-choice (§D):** 34 — RATIFY (0349 Jenkins-half DROP; 0352 amend-mandatory; 0377-forge renumber).
- **Special cases (§E):** 10 — incl. 0134 KEEP-as-Proposed-by-design; 0316 DROP-superseded.
- **TOTAL accounted ≈ 131 Proposed entries** (above the ~99 estimate because the Foundation cluster carries `proposed` status-drift the estimate under-counted). **Zero unaccounted.**

**The 3 true DROPs:** 0325 (prices retired primitive) · 0316 (superseded by 0329) · 0349 (Jenkins-half; ArgoCD-half survives). **ADR-0352** is AMEND-MANDATORY (not a DROP) — the §4 fork made concrete; ratify-or-regenerate after the masterplan-fork ruling. **0005** is resolved-by-supersession (broker drops, patterns survive). **0134** is the only intentional KEEP-as-Proposed.

**Door-class summary for the FOUNDER gate:** the `one-way` Proposeds requiring sign-off cluster on (a) the FORK/SSOT (0327/0352/0358-§4), (b) forge (0041/0387/0510/0511/0377-forge), (c) data-tier (0014/0045), (d) identity/self-mod (0247/0293), (e) scope-breadth (0249/0293/0300/0301/0307/0314/0315/0321), (f) licenses/residency (0013/0049/0292/0304), (g) substrate canon (0336/0338/0392). The `two-way` majority ratifies on green once the data-integrity sweep (§11 of 00-MASTER-REGISTER) clears.
