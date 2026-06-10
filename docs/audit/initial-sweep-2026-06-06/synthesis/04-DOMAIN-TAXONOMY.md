# 04 — DOMAIN TAXONOMY (closed enum + per-domain read-set index)

> The closed `domain` enum + the list of ADRs in each domain — the seed for the **domain-cohesion read-set index** (README binding principle: every ADR carries a `domain`; the cohesion gate resolves the read-set enum-keyed first and runs a contradiction check at decision time, so backfill is no-contradiction-by-construction).
> Two layers are given: **(1)** the **fine-grained 28-domain extractor taxonomy** actually used in `01-ADR-DISPOSITION-TABLE.md` (one row per ADR, counts grep-derived from the table), and **(2)** its reconciliation to the **closed 16-domain masterplan enum** used in `02-DECISION-ATOM-LEDGER.md` (the enum the generated masterplan + cohesion gate key on).
> **Flags:** a domain with **only 1 ADR** = merge candidate; **>40 ADRs** = split candidate. Counts include both sides (LINUX rows prefixed `L-`).
> **Count note:** the per-domain indices below were grep-derived from the table before the two coverage-stragglers were added — add **ADR-0381 → `orchestration-scheduling`** (→ 16) and **ADR-0482 → `governance-process`** (→ 42, reinforcing the SPLIT flag) when regenerating. Totals: 372 rows across the 28 fine-domains.

---

## 1. THE CLOSED 16-DOMAIN MASTERPLAN ENUM (cohesion-gate keys)

This is the enum the masterplan generator + `domain-cohesion` contradiction gate bind on (closed; new ADRs MUST declare one). It is the coarse roll-up; the §2 fine taxonomy maps into it.

| # | domain (enum) | one-line scope | rolls up fine-domains (§2) |
|---|---|---|---|
| 1 | `governance-ssot` | masterplan machinery, ADR lifecycle, repo topology, doc-as-SSOT, honesty gates, ownership process | governance-process, docs-ssot-masterplan |
| 2 | `ownership-doctrine` | the own-when-proven ratchet, vendor policy, OSS stewardship, silicon/portfolio scope | (cross-cuts; atoms in 02 §B/§16) |
| 3 | `policy-authz` | Cedar engine + contract, autonomy ceiling, admission, EU-AI-Act, data-class floor | authz-policy |
| 4 | `identity-crypto` | IdP, WebAuthn, SCIM, step-up, secrets/HSM, crypto provider | identity-authn, crypto-keymgmt |
| 5 | `data-storage` | OLTP/OLAP/vector/object/cache/TS, owned engine, time/clock, search | data-engine-db, data-storage |
| 6 | `eventing` | broker (Pulsar), outbox, schema versioning/registry, webhook delivery | (subset of data-engine-db + api-contracts) |
| 7 | `observability` | LGTM stack, OTel emission, SLO/RPO/RTO, status page | observability |
| 8 | `isolation-runtime` | runtime ladder, WASM, microVM/Kata, base image, owned-host/framekernel, container platform | isolation-runtime, kernel-frame |
| 9 | `orchestration-fleet` | Talos/CAPI/ArgoCD, node-OS, cellular, control-plane, federation, autosharding | orchestration-scheduling, node-os |
| 10 | `ci-cd` | gate engine, CI orchestration, CD/progressive-delivery, supply-chain, promotion, chaos, sweep | ci-cd-build (the CI/CD half) |
| 11 | `forge-scm` | forge host (the three-way), forge-neutral substrate, merge-queue, brand/monorepo layout | forge-vcs |
| 12 | `build-toolchain` | Buck2/Reindeer/NativeLink, layer enum/BNF, monorepo grammar | ci-cd-build (the build half) |
| 13 | `intelligence-ai` | two-layer AI substrate, provider routing, capability registry, inference gateway, supervisor, agent-execution, self-mod | intelligence-ai, agentic-platform, workflow-ontology |
| 14 | `tenancy` | tenant scoping, tenant-class, audit-slicing, quotas, lifecycle, env-stages, feature-flags, finops | tenancy, finops-cost |
| 15 | `api-surface` | stability tiers, gateway-vs-mesh, HTTP backbone, API hygiene, surface separation, degradation, CLI, gRPC, CRDT | api-contracts, networking-mesh |
| 16 | `compliance-residency` | residency, PII, DSR/trust, DUBO, license posture, DR, sovereign/air-gap, rights-safety doctrine, comms | compliance-residency, dr-resilience, comms-notify |
| (+) | `product-scope` | (FOUNDER-gated breadth bucket: verticals, clients, marketplace, ERP — not a substrate domain) | product-ux, marketplace-commerce, hardware-firmware |

> **Note:** `marketplace-commerce`, `product-ux`, and `hardware-firmware` fine-domains roll into a **product-scope** bucket that the masterplan treats as **FOUNDER breadth rulings**, not a closed substrate domain — their membership is gated by the §7/§9 scope decisions in 00-MASTER-REGISTER, not by architecture.

---

## 2. THE FINE-GRAINED 28-DOMAIN EXTRACTOR TAXONOMY (per-ADR read-set index)

One ADR = one fine-domain (the `domain` column of `01-ADR-DISPOSITION-TABLE.md`). This IS the read-set index: to author/amend an ADR, the cohesion gate first pulls the sibling set below.

### SPLIT CANDIDATES (>40 — too coarse; split before binding)

**`ci-cd-build` — [47] ⚠ SPLIT.** Conflates three concerns; split into `ci-cd` (orchestration/CD/promotion/supply-chain), `build-toolchain` (Buck2/Reindeer/NativeLink/layer-enum/monorepo grammar), and `forge-scm` (already separate as `forge-vcs`). Members:
`0014 0015 0040 0041 0050 0052 0054 0056 0062 0083 0092 0105 0106 0107 0111 0114 0116 0118 0133 0134 0138 0139 0143 0160 0221 0339 0346 0349 0357 0358 0359 0360 0366 0367 0369 0374 0380 0387 0391 0392 0408 0481 0509 0511 0512 0513 0514`
→ suggested split: **build-toolchain** {0056 0083 0092 0105 0106 0107 0357 0392 0408 0509 0512}; **ci-cd** {the rest}.

**`governance-process` — [41] ⚠ SPLIT (borderline).** Split into `governance-ssot` (masterplan/ADR-lifecycle/doc-SSOT/honesty gates) vs `repo-topology` (flat-catalog/layout/grammar/registry roots) vs `ownership-doctrine` (the ratchet). Members:
`0001 0004 0016 0025 0053 0057 0058 0060 0069 0091 0097 0104 0108 0109 0115 0117 0123 0128 0129 0131 0132 0135 0159 0211 0212 0217 0236 0237 0245 0284 0323 0324 0327 0328 0347 0365 0368 L-0010 L-0019 L-0020 L-0022`
→ suggested split: **ownership-doctrine** {0211 L-0019 L-0020 L-0022}; **repo-topology** {0058 0115 0117 0131 0132}; **governance-ssot** {the rest}.

### HEALTHY DOMAINS (2–40)

| domain | n | ADR ids (read-set) |
|---|---:|---|
| `data-engine-db` | 26 | 0005 0006 0045 0046 0047 0055 0122 0130 0142 0153 0172 0179 0184 0192 0193 0195 0252 0377-kafka L-0001 L-0002 L-0003 L-0004 L-0005 L-0006 L-0007 L-0008 |
| `product-ux` | 23 | 0029 0030 0048 0051 0061 0167 0170 0185 0204 0205 0206 0207 0219 0234 0238 0317 0318 0321 0332 0334 0372 0393 0394 |
| `compliance-residency` | 20 | 0008 0010 0038 0064 0144 0156 0158 0164 0209 0240 0250 0251 0272 0276 0292 0298 0300 0301 0304 0326 |
| `api-contracts` | 19 | 0011 0037 0090 0093 0094 0140 0141 0149 0150 0154 0166 0169 0177 0178 0216 0235 0258 0342 0350 |
| `tenancy` | 18 | 0049 0095 0155 0162 0163 0175 0214 0215 0218 0242 0244 0311 0313 0316 0329 0330 0331 0362 |
| `intelligence-ai` | 17 | 0020 0021 0024 0026 0027 0136 0137 0220 0239 0255 0308 0335 0355 0373 0384 0389 0390 |
| `authz-policy` | 16 | 0007 0022 0034 0099 0183 0191 0243 0246 0294 0303 0309 0312 0319 0353 0379 L-0021 |
| `orchestration-scheduling` | 15 | 0171 0198 0202 0222 0248 0254 0280 0333 0341 0348 0351 0376 L-0012 L-0015 L-0016 |
| `security-supplychain` | 13 | 0013 0039 0146 0173 0181 0295 0296 0297 0307 0310 0345 0361 L-0023 |
| `observability` | 11 | 0003 0042 0066 0067 0151 0168 0180 0186 0210 0263 0383 |
| `identity-authn` | 10 | 0002 0187 0188 0189 0190 0299 0302 0320 0476 0507 |
| `docs-ssot-masterplan` | 10 | 0018 0019 0063 0065 0119 0203 0322 0352 0364 0388 |
| `networking-mesh` | 9 | 0044 0145 0148 0157 0182 0208 0253 0354 0371 |
| `forge-vcs` | 9 | 0017 0103 0110 0112 0113 0124 0223 0363 0510 |
| `node-os` | 8 | 0028 0120 0121 0370 0375 0378 0382 L-0025 |
| `isolation-runtime` | 8 | 0009 0023 0036 0147 0200 0338 L-0014 L-0017 |
| `agentic-platform` | 8 | 0096 0098 0100 0101 0102 0247 0305 0377-forge |
| `finops-cost` | 7 | 0174 0199 0325 0340 0344 0479 0480 |
| `dr-resilience` | 7 | 0152 0165 0176 0197 0241 0306 0343 |
| `marketplace-commerce` | 6 | 0031 0213 0249 0314 0315 0478 |
| `kernel-frame` | 5 | L-0009 L-0011 L-0018 L-0024 L-0026 |
| `data-storage` | 5 | 0161 0194 0196 0336 0337 |
| `workflow-ontology` | 4 | 0035 0059 0257 0356 |
| `crypto-keymgmt` | 4 | 0043 0293 0506 0508 |

### MERGE CANDIDATES (singletons / n=2 — too thin to stand alone)

| domain | n | ADR ids | recommended merge |
|---|---:|---|---|
| `hardware-firmware` | 2 | 0032 L-0013 | → fold into `node-os` (both are physical-substrate posture). No true singleton, but thin. |
| `comms-notify` | 2 | 0201 0273 | → fold into `compliance-residency` (email deliverability is a regulatory Tier-1 concern) or keep as a `compliance-residency` sub-axis. |

> **No fine-domain is a true singleton (n=1)** — the corpus is well-distributed. The two n=2 domains are merge candidates; `data-storage` (n=5) and `data-engine-db` (n=26) should be reconciled (data-storage = the substrate-pick rows; data-engine-db = the engine/pattern rows) so the cohesion gate doesn't split the data tier across two read-sets.

---

## 3. CROSS-DOMAIN ADRs (declare a `domain`, but their read-set spans 2+)

The cohesion gate must treat these as multi-membership (primary domain + secondary read-set), or they generate false "no sibling" or false-contradiction signals:

| ADR | primary | also reads / amends |
|---|---|---|
| 0148 service mesh | networking-mesh | isolation-runtime (Cilium eBPF), authz-policy (Cedar ext_authz) |
| 0243 Cedar universal gate | authz-policy | governance-ssot (the phantom 0150 anchor), data-storage (fragment registry on Postgres) |
| 0338 pod runtime tier | isolation-runtime | tenancy (per-µsvc declaration), authz-policy (Kyverno→Kubewarden) |
| 0252 / L-0006 time-clock | data-engine-db | observability (HLC tick on audit rows), compliance-residency (Tier-4 IL5) |
| 0476 oya-identity | identity-authn | crypto-keymgmt (0506/0507/0508), policy-authz (Cedar over human+SPIFFE) |
| 0363 / 0510 forge | forge-vcs | ci-cd (gate sink), intelligence-ai (agentic-VCS retirement) |
| 0364 / 0365 masterplan-gen | docs-ssot-masterplan | ALL (every planning_impact ADR is an input) |
| 0307 detection substrate | security-supplychain | eventing (Pulsar), data-storage (Valkey), authz-policy (Cedar signals) |
| L-0001 owned DB engine | data-engine-db | orchestration-fleet (etcd replacement), data-storage (vector/FTS absorption) |
| L-0015 control plane | orchestration-scheduling | data-engine-db (owned datastore), governance-ssot (Manifold typed config) |

---

## 4. DOMAIN-COHESION GATE SEED (how this index is used)

1. **Authoring an ADR** → declare `domain` from the closed 16-enum (§1). The gate pulls the §2 fine-domain sibling read-set (+ §3 cross-domain neighbors).
2. **Contradiction check at decision time** → the new decision is compared against every sibling's `decision_atom` (from `02-DECISION-ATOM-LEDGER.md`); a conflicting atom in the same domain is a BLOCKER (no-contradiction-by-construction).
3. **Read-set resolution order** (README): enum-keyed first (this index), vector-recall later (for cross-domain neighbors not in §3).
4. **Generated masterplan sections** map 1:1 to the 16-enum (§1) — each enum is a masterplan `§N` in `02`'s "proposed masterplan section" column.

> **Open founder ratification (ties to §15 of 00-MASTER-REGISTER):** approve (a) the closed 16-domain enum as the cohesion-gate key, (b) the `ci-cd-build`/`governance-process` SPLITS before binding, (c) the `hardware-firmware`→`node-os` and `comms-notify`→`compliance-residency` merges, (d) `data-storage`+`data-engine-db` reconciliation. Until ratified, the fine 28-taxonomy is the working index and the 16-enum is the proposed target.
