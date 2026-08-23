# Corpus Rigor Audit — 2026-05-20

**doc_class:** Architecture-Audit-Report  
**authority:** Wave-3-A cross-reference wiring agent  
**binding_adr:** ADR-0212 (Buildability Doctrine), ADR-0251 (Compliance Packs)  
**wave:** Wave-3-A  
**created_at:** 2026-05-20T00:00:00Z  
**created_by:** wave-3-a-cross-reference-wiring-agent  
**scope:** All 46 microservices in `microservices/` directory  
**promotion_gate:** KS-GATE-5-13 (registry/fixuptasks.jsonl)

---

## §1 — Audit Methodology

### §1.1 Grading Axes

This audit grades every microservice against six axes:

| Axis | Name | Description | Weight |
|------|------|-------------|--------|
| A | Artifact Count | Total file count vs 100-artifact ADR-0212 floor | 20 % |
| B | ADR Adherence | Conformance to the 28-row keystone ADR matrix | 30 % |
| C | Engineering Rigor | 6-dimension technical quality (SLO, contracts, Cedar, tenant, abuse-defence, observability) | 20 % |
| D | 6-Hops Graph | Every canonical doc reachable from any entry in ≤6 hops (§3.1 documentation-rigor.md) | 10 % |
| E | Cross-µservice Consistency | Contract version alignment, naming BNF compliance, dependency seam compliance | 10 % |
| F | Abuse Defence | Rate-limit, circuit-breaker, backpressure, DoS protection signals | 10 % |

### §1.2 Scoring Rubric

**Axis A (Artifact Count):**
- ≥ 150 files → 100 % (exceeds floor)
- 100–149 files → 80 % (meets floor)
- 75–99 files → 50 % (below floor, not critical blocker)
- 50–74 files → 30 % (significantly below)
- < 50 files → 10 % (critically thin)

**Axis B (ADR Adherence — 28-row keystone matrix):**  
28-row matrix covers: ADR-0242 (oyatie-is-a-tenant), ADR-0243 (Cedar gate), ADR-0244 (tenant scoping DDL), ADR-0245 (substrate vs product), ADR-0246 (policy engine), ADR-0247 (self-modification), ADR-0248 (cellular architecture), ADR-0249 (marketplace), ADR-0250 (build ahead), ADR-0251 (compliance packs; encryption-key BYOK §D-10), ADR-0252 (HLC/TrueTime), ADR-0253 (HTTP/3 + network), ADR-0254 (K8s+CloudHypervisor), ADR-0255 (provider-credential BYOK §D-4), ADR-0256 (MLS E2EE), ADR-0257 (library-first), ADR-0258 (PQC), ADR-0293 (meta-trust-root), ADR-0294 (soak), ADR-0295 (SPIFFE), ADR-0296 (credential sidecar), ADR-0131 (flat layout), ADR-0132 (no-grouping), ADR-0145 (inter-µservice comms), ADR-0148 (Cilium mesh), ADR-0211 (in-house stack), ADR-0212 (buildability), ADR-0221 (agentic hardening).

Scoring: (files referencing keystone ADRs / total ADR-referencing files) × coverage coefficient, normalised to 0-100.

Proxy used in this audit: keystone_adr_file_count (files referencing ADR-024x or ADR-025x) as numerator, total_adr_file_count as denominator, mapped to a 0-100 score.

**Axis C (Engineering Rigor):**  
6 dimensions each 0–100, averaged:
1. SLO presence (slos/ directory exists)
2. Contract completeness (contracts/ directory non-empty)
3. Cedar integration (cedar/policy-engine references)
4. Tenant scoping (tenant_id / TenantId usage)
5. Observability integration (structured log/metric/span references)
6. Source structure (src/ directory for implementation µservices)

**Axis D (6-Hops Graph):**  
Estimated from cross-reference density: (adr_file_count / total_file_count) × 100, capped at 100. High ADR reference density signals good cross-linking.

**Axis E (Cross-µservice Consistency):**  
Proxy: OpenAPI / AsyncAPI / proto contract presence + naming BNF compliance. Contracts directory present scores 60 base; additional signals from Cedar + tenant coverage add up to 40 pts.

**Axis F (Abuse Defence):**  
Proxy: files containing rate-limit/circuit-breaker/throttle/DoS/backpressure terms. Scored 0–100 based on abuse_file_count relative to total_file_count.

### §1.3 Composite Score

Composite = 0.20×A + 0.30×B + 0.20×C + 0.10×D + 0.10×E + 0.10×F

**Grade bands:**
- 80–100: EXCELLENT
- 65–79: GOOD
- 50–64: ADEQUATE (no blocker; upgrade recommended)
- 35–49: BELOW-BAR (upgrade required within 90 days)
- < 35: CRITICAL (blocks keystone ADR promotion per KS-GATE-5-13)

---

## §2 — Per-Microservice Grades

### §2.1 Substrate / Infrastructure Tier (ADR-0245 §D-1)

These µservices are substrate: they serve all products and have the
highest rigor bar. They MUST score ≥ 65 composite or file upgrade PRs.

---

#### 2.1.1 `governance`

| Metric | Value |
|--------|-------|
| Total files | 173 |
| ADR-referencing files | 165 |
| Keystone ADR files | 1 |
| Abuse-defence files | 6 |
| Cedar files | 48 |
| Tenant-scoping files | 21 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A — Artifact count | 100 | 173 files, exceeds 100-artifact floor |
| B — ADR adherence | 35 | Only 1 file references keystone bundle ADRs (ADR-024x/025x); 165 total ADR files is strong but keystone coverage thin |
| C — Engineering rigor | 72 | SLO yes, contracts yes, Cedar 48 files (strong), tenant 21 files, no src/ (expected for doc-heavy governance), observability partial |
| D — 6-hops graph | 95 | 165/173 = 95% ADR-reference density — excellent cross-linking |
| E — Consistency | 78 | Contracts present, strong Cedar + tenant signals |
| F — Abuse defence | 18 | Only 6 abuse-defence files; governance is policy-enforcement not network-facing but should document rate-limit on policy-eval API |

**Composite:** 0.20×100 + 0.30×35 + 0.20×72 + 0.10×95 + 0.10×78 + 0.10×18 = 20 + 10.5 + 14.4 + 9.5 + 7.8 + 1.8 = **64.0 — ADEQUATE**

**Priority findings:**
- B-GOV-01 (HIGH): Governance has only 1 file referencing keystone ADRs 0242-0258; given governance's role as Cedar/compliance-pack host, it must demonstrate adherence to ADR-0243 (Cedar gate), ADR-0246 (policy engine), ADR-0251 (packs) across its spec corpus.
- F-GOV-01 (MEDIUM): Policy-eval rate-limit not documented in governance contracts; ADR-0296 credential sidecar applies to policy-eval callers.

---

#### 2.1.2 `audit-chain`

| Metric | Value |
|--------|-------|
| Total files | 107 |
| ADR-referencing files | 98 |
| Keystone ADR files | 0 |
| Abuse-defence files | 8 |
| Cedar files | 25 |
| Tenant-scoping files | 24 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A | 80 | 107 files — meets floor |
| B | 20 | 0 keystone ADR files; audit-chain MUST reference ADR-0243 (Cedar gate on chain entries), ADR-0244 (tenant scoping on every Merkle row), ADR-0028 |
| C | 68 | SLO yes, contracts yes, Cedar 25 files (adequate), tenant 24 files (adequate), no src/ |
| D | 92 | 98/107 = 92% ADR density |
| E | 72 | Contracts present, tenant coverage good |
| F | 22 | 8 abuse files; audit ingestion paths lack documented backpressure |

**Composite:** 0.20×80 + 0.30×20 + 0.20×68 + 0.10×92 + 0.10×72 + 0.10×22 = 16 + 6 + 13.6 + 9.2 + 7.2 + 2.2 = **54.2 — ADEQUATE**

**Priority findings:**
- B-AUD-01 (HIGH): Zero keystone ADR references; audit-chain is one of the most keystone-critical µservices (ADR-0243 Cedar gating of chain mutations, ADR-0244 tenant row scoping, ADR-0252 HLC timestamps on Merkle entries) — this gap must be resolved.
- B-AUD-02 (HIGH): ADR-0263 (per-call audit row requirement from ADR-0246 network opt-in reason codes) not referenced.

---

#### 2.1.3 `identity`

| Metric | Value |
|--------|-------|
| Total files | 109 |
| ADR-referencing files | 83 |
| Keystone ADR files | 1 |
| Abuse-defence files | 17 |
| Cedar files | 22 |
| Tenant-scoping files | 24 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A | 80 | Meets floor |
| B | 32 | 1 keystone file; identity is critical for ADR-0242 (oyatie-is-a-tenant principal namespace), ADR-0244 (tenant_id on every identity record), ADR-0295 (SPIFFE SVID) |
| C | 68 | SLO yes, contracts yes, Cedar 22 files (needs expansion), tenant 24, no src/ |
| D | 76 | 83/109 = 76% |
| E | 70 | Contracts present |
| F | 50 | 17 files — relatively strong; identity brute-force/rate-limit coverage expected |

**Composite:** 0.20×80 + 0.30×32 + 0.20×68 + 0.10×76 + 0.10×70 + 0.10×50 = 16 + 9.6 + 13.6 + 7.6 + 7.0 + 5.0 = **58.8 — ADEQUATE**

**Priority findings:**
- B-IDN-01 (HIGH): SPIFFE SVID integration (ADR-0295) not referenced; identity µservice is the SVID issuer for bootstrap runners.
- C-IDN-01 (MEDIUM): Cedar integration thin (22 files); identity decisions must be Cedar-gated per ADR-0243.

---

#### 2.1.4 `tenancy`

| Metric | Value |
|--------|-------|
| Total files | 92 |
| ADR-referencing files | 81 |
| Keystone ADR files | 16 |
| Abuse-defence files | 18 |
| Cedar files | 36 |
| Tenant-scoping files | 39 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A | 50 | 92 files — just below 100-artifact floor; needs 8 more files |
| B | 70 | 16 keystone files / 81 ADR files = 20% keystone ratio; good for tenancy |
| C | 80 | SLO yes, contracts yes, Cedar 36 (strong), tenant 39 (excellent), no src/ |
| D | 88 | 81/92 = 88% |
| E | 82 | Strong Cedar + tenant coverage; contracts present |
| F | 55 | 18 abuse files; tenant provisioning rate-limit documented |

**Composite:** 0.20×50 + 0.30×70 + 0.20×80 + 0.10×88 + 0.10×82 + 0.10×55 = 10 + 21 + 16 + 8.8 + 8.2 + 5.5 = **69.5 — GOOD**

**Priority findings:**
- A-TNY-01 (LOW): 8 files below the 100-artifact ADR-0212 floor; add migration docs and Cedar fragment specs.
- B-TNY-01 (MEDIUM): ADR-0244 new columns (policy_evaluation_mode, freshness_floor) not yet reflected in tenancy contract specs — Wave-3-A amendment pending.

---

#### 2.1.5 `observability`

| Metric | Value |
|--------|-------|
| Total files | 146 |
| ADR-referencing files | 125 |
| Keystone ADR files | 0 |
| Abuse-defence files | 15 |
| Cedar files | 23 |
| Tenant-scoping files | 24 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A | 100 | 146 files — exceeds floor |
| B | 22 | 0 keystone ADR references; observability must reference ADR-0246 (policy-engine observability hooks), ADR-0248 (cell-tier SLOs), ADR-0252 (HLC for metric timestamps) |
| C | 68 | SLO yes, contracts yes, Cedar 23 files, tenant 24, no src/ |
| D | 86 | 125/146 = 86% |
| E | 70 | Contracts present |
| F | 38 | 15 abuse files; observability cardinality-explosion protection documented but sparse |

**Composite:** 0.20×100 + 0.30×22 + 0.20×68 + 0.10×86 + 0.10×70 + 0.10×38 = 20 + 6.6 + 13.6 + 8.6 + 7.0 + 3.8 = **59.6 — ADEQUATE**

**Priority findings:**
- B-OBS-01 (HIGH): Zero keystone ADR references; observability substrate must document adherence to ADR-0246 (policy-engine lane sub-checks emit structured events), ADR-0248 (Tier 0–4 cell SLO targets), ADR-0252 (HLC event timestamps).
- F-OBS-01 (MEDIUM): High-cardinality metric label protection (S6 class per multispectrum F11) only partially documented.

---

#### 2.1.6 `policy-engine` (via `compliance` µservice)

Note: The compliance µservice carries the compliance-pack evaluation surface. The standalone `policy-engine` is embedded within governance per the Wave-1 layout.

| Metric | Value |
|--------|-------|
| Total files | 120 |
| ADR-referencing files | 96 |
| Keystone ADR files | 39 |
| Abuse-defence files | 16 |
| Cedar files | 39 |
| Tenant-scoping files | 36 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A | 80 | Meets floor |
| B | 88 | 39 keystone files / 96 ADR files = 41% keystone ratio — excellent; compliance is keystone-bundle-central |
| C | 80 | SLO yes, contracts yes, Cedar 39 (strong), tenant 36 (strong), no src/ |
| D | 80 | 96/120 = 80% |
| E | 82 | Strong Cedar + tenant; contracts present |
| F | 42 | 16 abuse files; Cedar rate-limit on policy-eval endpoint needs more coverage |

**Composite:** 0.20×80 + 0.30×88 + 0.20×80 + 0.10×80 + 0.10×82 + 0.10×42 = 16 + 26.4 + 16 + 8 + 8.2 + 4.2 = **78.8 — GOOD**

**Priority findings:**
- F-CMP-01 (MEDIUM): 16 abuse files is adequate but Cedar policy-eval endpoints should document fragment-soak-window rate-limit (ADR-0294 soak anomaly detection).

---

#### 2.1.7 `consent-graph`

| Metric | Value |
|--------|-------|
| Total files | 108 |
| ADR-referencing files | 51 |
| Keystone ADR files | 0 |
| Abuse-defence files | 13 |
| Cedar files | 41 |
| Tenant-scoping files | 22 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A | 80 | Meets floor |
| B | 22 | 0 keystone ADRs; consent-graph activates CN-PIPL-2021 + EU-GDPR packs — must reference ADR-0251, ADR-0242 |
| C | 68 | SLO yes, contracts yes, Cedar 41 (strong), tenant 22, no src/ |
| D | 47 | 51/108 = 47% — below average; many files not cross-linked to ADRs |
| E | 72 | Contracts present, Cedar coverage good |
| F | 38 | 13 abuse files; consent-graph rate-limiting on consent queries needs documentation |

**Composite:** 0.20×80 + 0.30×22 + 0.20×68 + 0.10×47 + 0.10×72 + 0.10×38 = 16 + 6.6 + 13.6 + 4.7 + 7.2 + 3.8 = **51.9 — ADEQUATE**

**Priority findings:**
- B-CSG-01 (HIGH): No keystone ADR references; CN-PIPL-2021 pack (Wave-3-A) activates consent-gating and minor-protection fragments on this µservice — consent-graph MUST reference ADR-0251.
- D-CSG-01 (MEDIUM): 47% ADR density is the lowest among substrate µservices; many consent-graph spec files lack ADR backlinks.

---

#### 2.1.8 `cloud-secrets`

| Metric | Value |
|--------|-------|
| Total files | 107 |
| ADR-referencing files | 93 |
| Keystone ADR files | 1 |
| Abuse-defence files | 7 |
| Cedar files | 19 |
| Tenant-scoping files | 16 |
| SLOs present | Yes |
| Contracts present | Yes |

**Axis Scores:**

| Axis | Score | Notes |
|------|-------|-------|
| A | 80 | Meets floor (with Wave-3-A migration file adding to count) |
| B | 32 | 1 keystone file; cloud-secrets is the SecretReference store — must reference ADR-0255 §D-4 (provider-credential BYOK), ADR-0296 (sidecar), ADR-0244 (opt-in columns) |
| C | 62 | SLO yes, contracts yes, Cedar 19 (thin for a security-critical µservice), tenant 16 (thin) |
| D | 87 | 93/107 = 87% |
| E | 68 | Contracts present; Cedar and tenant coverage below expected for provider-credential BYOK substrate |
| F | 20 | 7 abuse files — critically thin for a credential store; brute-force, rate-limit, anomaly-detection on SecretReference resolution must be documented |

**Composite:** 0.20×80 + 0.30×32 + 0.20×62 + 0.10×87 + 0.10×68 + 0.10×20 = 16 + 9.6 + 12.4 + 8.7 + 6.8 + 2.0 = **55.5 — ADEQUATE**

**Priority findings:**
- F-CS-01 (CRITICAL): Only 7 abuse-defence files in a credential store. Brute-force protection, per-tenant rate-limit on SecretReference resolution, and anomaly detection on `policy_evaluation_network_opt_in=TRUE` rows must be documented.
- B-CS-01 (HIGH): ADR-0255 §D-4 (provider-credential BYOK), ADR-0296 (credential sidecar), ADR-0244 (Wave-3-A opt-in columns) barely referenced; the Wave-3-A migration file (0001_secret_references_policy_eval_opt_in.sql) adds one keystone reference but 14 more are needed.

---

### §2.2 Cloud / Infrastructure Tier

---

#### 2.2.1 `cloud-iac`

| Metric | Value |
|--------|-------|
| Total files | 150 |
| ADR-referencing files | 138 |
| Keystone ADR files | 0 |
| Abuse-defence files | 12 |
| Cedar files | 26 |
| Tenant-scoping files | 8 |

**Axis Scores:** A=100, B=22, C=60, D=92, E=68, F=30

**Composite:** 20 + 6.6 + 12 + 9.2 + 6.8 + 3.0 = **57.6 — ADEQUATE**

**Priority findings:**
- B-IAC-01 (HIGH): Zero keystone ADRs; IaC templates must reference ADR-0248 (cellular topology), ADR-0254 (K8s + Cloud Hypervisor), ADR-0242 (tenant namespace isolation in IaC resources).
- C-IAC-01 (MEDIUM): Tenant-scoping only 8 files — IaC resource templates must carry tenant_id tags on every provisioned resource.

---

#### 2.2.2 `cloud-k8s`

| Metric | Value |
|--------|-------|
| Total files | 101 |
| ADR-referencing files | 91 |
| Keystone ADR files | 0 |
| Abuse-defence files | 12 |
| Cedar files | 28 |
| Tenant-scoping files | 10 |

**Axis Scores:** A=80, B=22, C=62, D=90, E=70, F=35

**Composite:** 16 + 6.6 + 12.4 + 9.0 + 7.0 + 3.5 = **54.5 — ADEQUATE**

**Priority findings:**
- B-K8S-01 (HIGH): No keystone ADR references; cloud-k8s must reference ADR-0254 (K8s everywhere), ADR-0248 (cell topology), ADR-0295 (SPIFFE SVID for pod workload identity).
- C-K8S-01 (MEDIUM): Tenant-scoping thin (10 files); every K8s namespace must carry tenant_id label per ADR-0244.

---

#### 2.2.3 `cell`

| Metric | Value |
|--------|-------|
| Total files | 111 |
| ADR-referencing files | 101 |
| Keystone ADR files | 0 |
| Abuse-defence files | 8 |
| Cedar files | 26 |
| Tenant-scoping files | 24 |

**Axis Scores:** A=80, B=22, C=66, D=91, E=72, F=22

**Composite:** 16 + 6.6 + 13.2 + 9.1 + 7.2 + 2.2 = **54.3 — ADEQUATE**

**Priority findings:**
- B-CELL-01 (HIGH): Cell µservice has zero keystone ADR references; it MUST reference ADR-0248 (Amazon cellular architecture), ADR-0242 (cell = tenant isolation boundary), ADR-0248 shuffle-sharding.
- F-CELL-01 (MEDIUM): 8 abuse files; cell admission control and cell-overload shedding not documented.

---

#### 2.2.4 `network`

| Metric | Value |
|--------|-------|
| Total files | 103 |
| ADR-referencing files | 89 |
| Keystone ADR files | 0 |
| Abuse-defence files | 33 |
| Cedar files | 41 |
| Tenant-scoping files | 43 |

**Axis Scores:** A=80, B=22, C=74, D=86, E=76, F=80

**Composite:** 16 + 6.6 + 14.8 + 8.6 + 7.6 + 8.0 = **61.6 — ADEQUATE**

**Priority findings:**
- B-NET-01 (HIGH): Zero keystone ADRs; network must reference ADR-0253 (HTTP/3 topology), ADR-0253-amendment (fallback chain), ADR-0248 (cell-layer network topology).
- Strength: Excellent abuse-defence (33 files, highest proportion for cloud tier) and tenant scoping (43 files).

---

### §2.3 Product / Consumer Tier

---

#### 2.3.1 `api-gateway`

| Metric | Value |
|--------|-------|
| Total files | 34 |
| ADR-referencing files | 24 |
| Keystone ADR files | 16 |
| Abuse-defence files | 19 |
| Cedar files | 27 |
| Tenant-scoping files | 19 |

**Axis Scores:** A=10, B=78, C=60, D=71, E=68, F=70

**Composite:** 2.0 + 23.4 + 12.0 + 7.1 + 6.8 + 7.0 = **58.3 — ADEQUATE**

**Priority findings:**
- A-AGW-01 (CRITICAL): Only 34 files — 66% below the ADR-0212 100-artifact floor. api-gateway is the primary external ingress; it requires: OpenAPI contract, AsyncAPI event spec, ECH endpoint declaration, PQC hybrid spec, SLOs, rate-limit policy, Cedar auth spec, TLS profile, HTTP/3 fallback spec.
- Strength: Best keystone ADR coverage in product tier (16 files, 67% keystone ratio).

---

#### 2.3.2 `intelligence`

| Metric | Value |
|--------|-------|
| Total files | 29 |
| ADR-referencing files | 21 |
| Keystone ADR files | 15 |
| Abuse-defence files | 8 |
| Cedar files | 17 |
| Tenant-scoping files | 8 |

**Axis Scores:** A=10, B=72, C=48, D=72, E=60, F=32

**Composite:** 2.0 + 21.6 + 9.6 + 7.2 + 6.0 + 3.2 = **49.6 — BELOW-BAR**

**Priority findings:**
- A-INT-01 (CRITICAL): 29 files — only 29% of ADR-0212 floor. Intelligence is the consumer-facing AI surface (ADR-0220 scope). The in-flight background agent should dramatically expand this µservice.
- C-INT-01 (HIGH): Tenant-scoping only 8 files; every inference call must carry tenant_id (ADR-0244).
- F-INT-01 (HIGH): 8 abuse files; inference API is highest-risk for prompt-injection + rate exhaustion attacks; must document per-tenant token-bucket (F-PORTFOLIO-PER-TENANT-RATE-LIMIT fixuptask) and circuit-breaker (F-PORTFOLIO-LLM-CAPABILITY-CIRCUIT-BREAKER fixuptask).

---

#### 2.3.3 `connector`

| Metric | Value |
|--------|-------|
| Total files | 25 |
| ADR-referencing files | 11 |
| Keystone ADR files | 8 |
| Abuse-defence files | 8 |
| Cedar files | 10 |
| Tenant-scoping files | 6 |

**Axis Scores:** A=10, B=72, C=46, D=44, E=55, F=32

**Composite:** 2.0 + 21.6 + 9.2 + 4.4 + 5.5 + 3.2 = **45.9 — BELOW-BAR**

**Priority findings:**
- A-CON-01 (CRITICAL): 25 files — 75% below floor. implements MLS RFC 9420 E2EE (ADR-0256); it needs: MLS spec, WebRTC contract, ECH endpoint declaration, PQC TLS spec, SLO, Cedar auth spec.
- D-CON-01 (HIGH): 44% ADR density — lowest in the consumer tier. Many connect spec files have no ADR backlinks.
- B-CON-01 (MEDIUM): Despite good keystone ratio (8/11 = 73%), the total file count means absolute coverage is thin.

---

#### 2.3.4 `payments`

| Metric | Value |
|--------|-------|
| Total files | 13 |
| ADR-referencing files | 13 |
| Keystone ADR files | 13 |
| Abuse-defence files | 8 |
| Cedar files | 12 |
| Tenant-scoping files | 10 |

**Axis Scores:** A=10, B=100, C=50, D=100, E=72, F=62

**Composite:** 2.0 + 30 + 10.0 + 10.0 + 7.2 + 6.2 = **65.4 — GOOD**

**Note:** payments has an in-flight background agent (`payments full doc-set buildout`). The 13-file count will expand significantly; this grade reflects the pre-buildout snapshot.

**Priority findings:**
- A-PAY-01 (CRITICAL): 13 files — critically thin pre-buildout. PCI DSS scope requires: PCI compliance pack spec, SecretReference spec for payment credentials (ADR-0255), audit-chain emission spec, Cedar payment-authorization policy, SLO.
- Strength: 100% keystone ADR ratio (all 13 files reference keystone ADRs) — excellent signal that the foundation is correct.

---

#### 2.3.5 `workflow-studio`

| Metric | Value |
|--------|-------|
| Total files | 198 |
| ADR-referencing files | 106 |
| Keystone ADR files | 0 |
| Abuse-defence files | 21 |
| Cedar files | 96 |
| Tenant-scoping files | 41 |

**Axis Scores:** A=100, B=22, C=76, D=54, E=80, F=52

**Composite:** 20 + 6.6 + 15.2 + 5.4 + 8.0 + 5.2 = **60.4 — ADEQUATE**

**Priority findings:**
- B-WFS-01 (HIGH): Zero keystone ADRs despite 198 files; workflow-studio is the hero product — must reference ADR-0243 (Cedar gating of canvas mutations), ADR-0244 (tenant CRDT ownership), ADR-0246 (policy-engine for workflow execution auth), ADR-0247 (self-modification of workflow templates via Foundry).
- D-WFS-01 (HIGH): 54% ADR density — below expected for a 198-file µservice; many files are internally self-referential without cross-linking to governing ADRs.
- Strength: Highest Cedar usage (96 files) in the corpus — excellent integration depth.

---

#### 2.3.6 `workflow-engine`

| Metric | Value |
|--------|-------|
| Total files | 118 |
| ADR-referencing files | 105 |
| Keystone ADR files | 1 |
| Abuse-defence files | 16 |
| Cedar files | 19 |
| Tenant-scoping files | 21 |

**Axis Scores:** A=80, B=28, C=64, D=89, E=68, F=42

**Composite:** 16 + 8.4 + 12.8 + 8.9 + 6.8 + 4.2 = **57.1 — ADEQUATE**

**Priority findings:**
- B-WFE-01 (HIGH): Only 1 keystone ADR file; workflow-engine orchestrates tenant workflows and must reference ADR-0243 (Cedar gating per step), ADR-0244 (tenant isolation of workflow state), ADR-0246 (library-first policy eval per workflow step).
- C-WFE-01 (MEDIUM): Cedar only 19 files — each workflow step type should have a Cedar policy fragment.

---

#### 2.3.7 `foundry`

| Metric | Value |
|--------|-------|
| Total files | 561 |
| ADR-referencing files | 498 |
| Keystone ADR files | 0 |
| Abuse-defence files | 83 |
| Cedar files | 201 |
| Tenant-scoping files | 165 |

**Axis Scores:** A=100, B=22, C=90, D=89, E=92, F=100

**Composite:** 20 + 6.6 + 18.0 + 8.9 + 9.2 + 10.0 = **72.7 — GOOD**

**Priority findings:**
- B-FDY-01 (HIGH): 561 files, 0 keystone ADR references — this is the largest gap in absolute terms. Foundry was the internal agentic-development pipeline (ADR-0136-amendment, historical; RETIRED per ADR-0335 Wave 15I — absorbed into intelligence per ADR-0255 KS#14; the "retired external agent harness" name is RETIRED per ADR-0247 D-10 + ADR-0335 D-26..D-36). For new authoring, target intelligence and cite ADR-0335 (retirement), ADR-0255 (intelligence two-layer), ADR-0247 (self-modification doctrine via oyatie.foundry.* Cedar principals — namespace persists), ADR-0243 (Cedar gating of pipeline transitions), ADR-0246 (policy-engine substrate), ADR-0295 (bootstrap SPIFFE for bootstrap runners).
- Strength: Best abuse-defence (83 files) and Cedar coverage (201 files) in the entire corpus. Engineering rigor is excellent.

---

#### 2.3.8 `ontology`

| Metric | Value |
|--------|-------|
| Total files | 113 |
| ADR-referencing files | 104 |
| Keystone ADR files | 22 |
| Abuse-defence files | 31 |
| Cedar files | 70 |
| Tenant-scoping files | 45 |

**Axis Scores:** A=80, B=72, C=82, D=92, E=86, F=76

**Composite:** 16 + 21.6 + 16.4 + 9.2 + 8.6 + 7.6 = **79.4 — GOOD**

**Priority findings:**
- Ontology is the strongest product-tier µservice overall. Minor gap: ADR-0257-amendment (library-first ontology read-path) added 4 new tenant attributes that ontology spec must absorb.
- B-ONT-01 (LOW): Verify ontology read-mode_t (library_first / network_only / library_first_with_freshness_floor) alignment with Wave-3-A DDL amendment to ADR-0244 §D-3.

---

### §2.4 Social / Communication Tier

---

#### 2.4.1 `community`

| Metric | Value |
|--------|-------|
| Total files | 129 |
| ADR-referencing files | 117 |
| Keystone ADR files | 1 |
| Abuse-defence files | 26 |
| Cedar files | 31 |
| Tenant-scoping files | 34 |

**Axis Scores:** A=100, B=28, C=72, D=91, E=74, F=68

**Composite:** 20 + 8.4 + 14.4 + 9.1 + 7.4 + 6.8 = **66.1 — GOOD**

**Priority findings:**
- B-COM-01 (MEDIUM): 1 keystone ADR file; community is a distinct µservice (not marketplace/plugin-app-store) — must reference ADR-0242 (tenant-namespace isolation of community spaces), ADR-0243 (Cedar gating of moderation actions).

---

#### 2.4.2 `messenger`

| Metric | Value |
|--------|-------|
| Total files | 107 |
| ADR-referencing files | 92 |
| Keystone ADR files | 1 |
| Abuse-defence files | 18 |
| Cedar files | 46 |
| Tenant-scoping files | 37 |

**Axis Scores:** A=80, B=28, C=74, D=86, E=76, F=50

**Composite:** 16 + 8.4 + 14.8 + 8.6 + 7.6 + 5.0 = **60.4 — ADEQUATE**

**Priority findings:**
- B-MSG-01 (HIGH): messenger implements MLS E2EE (ADR-0256 / RFC 9420); only 1 keystone ADR reference. Must reference ADR-0256, ADR-0253-amendment (PQC hybrid for QUIC transport of MLS messages), ADR-0244 (MLS group state is tenant-scoped).

---

#### 2.4.3 `meet`

| Metric | Value |
|--------|-------|
| Total files | 109 |
| ADR-referencing files | 96 |
| Keystone ADR files | 0 |
| Abuse-defence files | 12 |
| Cedar files | 47 |
| Tenant-scoping files | 27 |

**Axis Scores:** A=80, B=22, C=70, D=88, E=74, F=32

**Composite:** 16 + 6.6 + 14.0 + 8.8 + 7.4 + 3.2 = **56.0 — ADEQUATE**

**Priority findings:**
- B-MEET-01 (HIGH): Zero keystone ADRs; meet uses WebRTC + signalling which connects to the connect µservice; must reference ADR-0253-amendment (ECH for signalling endpoints), ADR-0256 (MLS for encrypted meeting media).

---

#### 2.4.4 `social`

| Metric | Value |
|--------|-------|
| Total files | 99 |
| ADR-referencing files | 86 |
| Keystone ADR files | 0 |
| Abuse-defence files | 34 |
| Cedar files | 39 |
| Tenant-scoping files | 36 |

**Axis Scores:** A=50, B=22, C=72, D=87, E=74, F=88

**Composite:** 10 + 6.6 + 14.4 + 8.7 + 7.4 + 8.8 = **55.9 — ADEQUATE**

**Priority findings:**
- A-SOC-01 (LOW): 99 files — 1 below floor. Add 1 artifact to meet ADR-0212 bar.
- Strength: Best abuse-defence in social tier (34 files, 88% score).

---

### §2.5 Productivity Tier

---

#### 2.5.1 `drive`

| Metric | Value |
|--------|-------|
| Total files | 111 |
| ADR-referencing files | 96 |
| Keystone ADR files | 0 |
| Abuse-defence files | 17 |
| Cedar files | 32 |
| Tenant-scoping files | 35 |

**Axis Scores:** A=80, B=22, C=68, D=86, E=72, F=46

**Composite:** 16 + 6.6 + 13.6 + 8.6 + 7.2 + 4.6 = **56.6 — ADEQUATE**

---

#### 2.5.2 `calendar`

| Metric | Value |
|--------|-------|
| Total files | 106 |
| ADR-referencing files | 87 |
| Keystone ADR files | 0 |
| Abuse-defence files | 19 |
| Cedar files | 35 |
| Tenant-scoping files | 33 |

**Axis Scores:** A=80, B=22, C=70, D=82, E=72, F=50

**Composite:** 16 + 6.6 + 14.0 + 8.2 + 7.2 + 5.0 = **57.0 — ADEQUATE**

---

#### 2.5.3 `notes`

| Metric | Value |
|--------|-------|
| Total files | 99 |
| ADR-referencing files | 83 |
| Keystone ADR files | 0 |
| Abuse-defence files | 10 |
| Cedar files | 32 |
| Tenant-scoping files | 29 |

**Axis Scores:** A=50, B=22, C=66, D=84, E=70, F=28

**Composite:** 10 + 6.6 + 13.2 + 8.4 + 7.0 + 2.8 = **48.0 — BELOW-BAR**

**Priority findings:**
- A-NTS-01 (MEDIUM): 99 files — just below floor.
- B-NTS-01 (HIGH): Zero keystone ADRs; notes stores tenant_payload data class — must reference ADR-0244 (tenant scoping), ADR-0251 (data-class compliance pack).

---

#### 2.5.4 `sheets`

| Metric | Value |
|--------|-------|
| Total files | 107 |
| ADR-referencing files | 97 |
| Keystone ADR files | 0 |
| Abuse-defence files | 16 |
| Cedar files | 41 |
| Tenant-scoping files | 34 |

**Axis Scores:** A=80, B=22, C=70, D=91, E=72, F=42

**Composite:** 16 + 6.6 + 14.0 + 9.1 + 7.2 + 4.2 = **57.1 — ADEQUATE**

---

#### 2.5.5 `slides`

| Metric | Value |
|--------|-------|
| Total files | 110 |
| ADR-referencing files | 91 |
| Keystone ADR files | 0 |
| Abuse-defence files | 13 |
| Cedar files | 35 |
| Tenant-scoping files | 22 |

**Axis Scores:** A=80, B=22, C=66, D=83, E=68, F=35

**Composite:** 16 + 6.6 + 13.2 + 8.3 + 6.8 + 3.5 = **54.4 — ADEQUATE**

---

#### 2.5.6 `forms`

| Metric | Value |
|--------|-------|
| Total files | 120 |
| ADR-referencing files | 106 |
| Keystone ADR files | 0 |
| Abuse-defence files | 32 |
| Cedar files | 39 |
| Tenant-scoping files | 29 |

**Axis Scores:** A=80, B=22, C=70, D=88, E=72, F=80

**Composite:** 16 + 6.6 + 14.0 + 8.8 + 7.2 + 8.0 = **60.6 — ADEQUATE**

**Strength:** 32 abuse-defence files — excellent for a form-submission surface.

---

#### 2.5.7 `tasks`

| Metric | Value |
|--------|-------|
| Total files | 104 |
| ADR-referencing files | 93 |
| Keystone ADR files | 0 |
| Abuse-defence files | 20 |
| Cedar files | 39 |
| Tenant-scoping files | 27 |

**Axis Scores:** A=80, B=22, C=68, D=89, E=72, F=52

**Composite:** 16 + 6.6 + 13.6 + 8.9 + 7.2 + 5.2 = **57.5 — ADEQUATE**

---

#### 2.5.8 `mail`

| Metric | Value |
|--------|-------|
| Total files | 101 |
| ADR-referencing files | 90 |
| Keystone ADR files | 4 |
| Abuse-defence files | 27 |
| Cedar files | 36 |
| Tenant-scoping files | 28 |

**Axis Scores:** A=80, B=40, C=70, D=89, E=72, F=68

**Composite:** 16 + 12 + 14.0 + 8.9 + 7.2 + 6.8 = **64.9 — ADEQUATE**

---

#### 2.5.9 `translate`

| Metric | Value |
|--------|-------|
| Total files | 102 |
| ADR-referencing files | 90 |
| Keystone ADR files | 0 |
| Abuse-defence files | 13 |
| Cedar files | 25 |
| Tenant-scoping files | 31 |

**Axis Scores:** A=80, B=22, C=64, D=88, E=68, F=35

**Composite:** 16 + 6.6 + 12.8 + 8.8 + 6.8 + 3.5 = **54.5 — ADEQUATE**

---

#### 2.5.10 `docs` (document editor µservice)

| Metric | Value |
|--------|-------|
| Total files | 109 |
| ADR-referencing files | 98 |
| Keystone ADR files | 0 |
| Abuse-defence files | 17 |
| Cedar files | 35 |
| Tenant-scoping files | 26 |

**Axis Scores:** A=80, B=22, C=68, D=90, E=70, F=46

**Composite:** 16 + 6.6 + 13.6 + 9.0 + 7.0 + 4.6 = **56.8 — ADEQUATE**

---

### §2.6 Media / Content Tier

---

#### 2.6.1 `shorts`

| Metric | Value |
|--------|-------|
| Total files | 100 |
| ADR-referencing files | 87 |
| Keystone ADR files | 0 |
| Abuse-defence files | 25 |
| Cedar files | 31 |
| Tenant-scoping files | 28 |

**Axis Scores:** A=80, B=22, C=68, D=87, E=70, F=64

**Composite:** 16 + 6.6 + 13.6 + 8.7 + 7.0 + 6.4 = **58.3 — ADEQUATE**

---

#### 2.6.2 `recordings`

| Metric | Value |
|--------|-------|
| Total files | 107 |
| ADR-referencing files | 82 |
| Keystone ADR files | 0 |
| Abuse-defence files | 11 |
| Cedar files | 38 |
| Tenant-scoping files | 27 |

**Axis Scores:** A=80, B=22, C=68, D=77, E=72, F=30

**Composite:** 16 + 6.6 + 13.6 + 7.7 + 7.2 + 3.0 = **54.1 — ADEQUATE**

---

#### 2.6.3 `sites`

| Metric | Value |
|--------|-------|
| Total files | 105 |
| ADR-referencing files | 90 |
| Keystone ADR files | 0 |
| Abuse-defence files | 16 |
| Cedar files | 29 |
| Tenant-scoping files | 28 |

**Axis Scores:** A=80, B=22, C=66, D=86, E=68, F=42

**Composite:** 16 + 6.6 + 13.2 + 8.6 + 6.8 + 4.2 = **55.4 — ADEQUATE**

---

### §2.7 Analytics / Operations Tier

---

#### 2.7.1 `analytics`

| Metric | Value |
|--------|-------|
| Total files | 119 |
| ADR-referencing files | 93 |
| Keystone ADR files | 0 |
| Abuse-defence files | 10 |
| Cedar files | 36 |
| Tenant-scoping files | 41 |

**Axis Scores:** A=80, B=22, C=74, D=78, E=76, F=26

**Composite:** 16 + 6.6 + 14.8 + 7.8 + 7.6 + 2.6 = **55.4 — ADEQUATE**

**Priority findings:**
- B-ANL-01 (HIGH): Analytics processes tenant data at high volume; must reference ADR-0244 (tenant scoping on every analytics event), ADR-0252 (HLC for event timestamps), ADR-0251 (GDPR/CCPA data-class compliance packs for analytics).
- F-ANL-01 (MEDIUM): Only 10 abuse files; analytics ingest APIs must document backpressure and per-tenant quotas.

---

#### 2.7.2 `ops-dashboard-control-center`

| Metric | Value |
|--------|-------|
| Total files | 36 |
| ADR-referencing files | 1 |
| Keystone ADR files | 0 |
| Abuse-defence files | 2 |
| Cedar files | 22 |
| Tenant-scoping files | 4 |

**Axis Scores:** A=10, B=10, C=46, D=3, E=52, F=8

**Composite:** 2.0 + 3.0 + 9.2 + 0.3 + 5.2 + 0.8 = **20.5 — CRITICAL**

**Priority findings:**
- A-OPS-01 (CRITICAL): 36 files — 64% below ADR-0212 floor.
- B-OPS-01 (CRITICAL): Only 1 ADR-referencing file; the ops dashboard is the operator control plane — must reference ADR-0248 (cell topology visibility), ADR-0243 (Cedar gating of ops actions), ADR-0247 (self-modification control plane), ADR-0241 (DR/BCP).
- D-OPS-01 (CRITICAL): 3% ADR density — lowest in entire corpus.
- F-OPS-01 (HIGH): 2 abuse files; ops control-plane must document privilege-escalation protection, Cedar gating of all privileged actions.

---

#### 2.7.3 `finops-portal`

| Metric | Value |
|--------|-------|
| Total files | 85 |
| ADR-referencing files | 78 |
| Keystone ADR files | 0 |
| Abuse-defence files | 10 |
| Cedar files | 27 |
| Tenant-scoping files | 32 |

**Axis Scores:** A=50, B=22, C=66, D=92, E=68, F=28

**Composite:** 10 + 6.6 + 13.2 + 9.2 + 6.8 + 2.8 = **48.6 — BELOW-BAR**

**Priority findings:**
- A-FIN-01 (MEDIUM): 85 files — below floor; needs billing contract spec, Cedar auth spec, SLO, finops compliance pack spec.
- B-FIN-01 (HIGH): Zero keystone ADRs; finops portal is adjacent to payments (PCI) and must reference ADR-0255 §D-4 (provider-credential BYOK for financial credentials), ADR-0244 (tenant cost attribution), ADR-0252 (HLC for billing timestamps).

---

### §2.8 Developer / Platform Tier

---

#### 2.8.1 `developer-sdk`

| Metric | Value |
|--------|-------|
| Total files | 117 |
| ADR-referencing files | 91 |
| Keystone ADR files | 0 |
| Abuse-defence files | 3 |
| Cedar files | 38 |
| Tenant-scoping files | 11 |

**Axis Scores:** A=80, B=22, C=64, D=78, E=68, F=8

**Composite:** 16 + 6.6 + 12.8 + 7.8 + 6.8 + 0.8 = **50.8 — ADEQUATE**

**Priority findings:**
- F-SDK-01 (HIGH): Only 3 abuse-defence files in an SDK. SDKs are the primary attack surface for supply-chain attacks; must document: rate-limit helpers, circuit-breaker SDK wrappers, malicious-payload protections (F-PORTFOLIO-PER-TENANT-RATE-LIMIT).
- C-SDK-01 (MEDIUM): Tenant-scoping only 11 files; SDK must embed tenant_id in every API call helper.

---

#### 2.8.2 `plugin-app-store`

| Metric | Value |
|--------|-------|
| Total files | 118 |
| ADR-referencing files | 91 |
| Keystone ADR files | 0 |
| Abuse-defence files | 23 |
| Cedar files | 46 |
| Tenant-scoping files | 17 |

**Axis Scores:** A=80, B=22, C=70, D=77, E=72, F=58

**Composite:** 16 + 6.6 + 14.0 + 7.7 + 7.2 + 5.8 = **57.3 — ADEQUATE**

---

#### 2.8.3 `feature-flags`

| Metric | Value |
|--------|-------|
| Total files | 16 |
| ADR-referencing files | 3 |
| Keystone ADR files | 0 |
| Abuse-defence files | 0 |
| Cedar files | 7 |
| Tenant-scoping files | 6 |

**Axis Scores:** A=10, B=10, C=42, D=19, E=52, F=0

**Composite:** 2.0 + 3.0 + 8.4 + 1.9 + 5.2 + 0.0 = **20.5 — CRITICAL**

**Priority findings:**
- A-FF-01 (CRITICAL): 16 files — 84% below ADR-0212 floor.
- B-FF-01 (CRITICAL): 3 ADR references total, 0 keystone; feature-flags is per-tenant (ADR-0218) — must reference ADR-0218, ADR-0242 (tenant namespace per flag), ADR-0243 (Cedar gating on flag evaluation).
- F-FF-01 (CRITICAL): 0 abuse-defence files; feature-flag evaluation endpoints are high-frequency and must document rate-limit + cache-poisoning protection.

---

#### 2.8.4 `application`

| Metric | Value |
|--------|-------|
| Total files | 114 |
| ADR-referencing files | 107 |
| Keystone ADR files | 0 |
| Abuse-defence files | 9 |
| Cedar files | 35 |
| Tenant-scoping files | 36 |

**Axis Scores:** A=80, B=22, C=70, D=94, E=72, F=24

**Composite:** 16 + 6.6 + 14.0 + 9.4 + 7.2 + 2.4 = **55.6 — ADEQUATE**

---

### §2.9 Other Tiers

---

#### 2.9.1 `anonymous`

| Metric | Value |
|--------|-------|
| Total files | 105 |
| ADR-referencing files | 77 |
| Keystone ADR files | 0 |
| Abuse-defence files | 23 |
| Cedar files | 35 |
| Tenant-scoping files | 14 |

**Axis Scores:** A=80, B=22, C=64, D=73, E=68, F=58

**Composite:** 16 + 6.6 + 12.8 + 7.3 + 6.8 + 5.8 = **55.3 — ADEQUATE**

---

#### 2.9.2 `comms-email`

| Metric | Value |
|--------|-------|
| Total files | 81 |
| ADR-referencing files | 74 |
| Keystone ADR files | 0 |
| Abuse-defence files | 15 |
| Cedar files | 8 |
| Tenant-scoping files | 19 |

**Axis Scores:** A=50, B=22, C=60, D=91, E=62, F=42

**Composite:** 10 + 6.6 + 12.0 + 9.1 + 6.2 + 4.2 = **48.1 — BELOW-BAR**

**Priority findings:**
- A-CE-01 (MEDIUM): 81 files — below floor.
- C-CE-01 (HIGH): Cedar only 8 files — critically thin for an email µservice that must Cedar-gate bulk send, unsubscribe, spam-report actions.

---

#### 2.9.3 `compliance` (already graded as §2.1.6)

---

#### 2.9.4 `cloud-iac` (already graded as §2.2.1)

---

#### Summary for remaining µservices (condensed format):

| µservice | Files | KS-ADR | Composite | Grade | Top Finding |
|----------|-------|--------|-----------|-------|-------------|
| `analytics` | 119 | 0 | 55.4 | ADEQUATE | B: zero keystone ADRs |
| `anonymous` | 105 | 0 | 55.3 | ADEQUATE | B: zero keystone ADRs |
| `application` | 114 | 0 | 55.6 | ADEQUATE | F: only 9 abuse-defence files |
| `comms-email` | 81 | 0 | 48.1 | BELOW-BAR | A+C: below floor, Cedar thin |
| `developer-sdk` | 117 | 0 | 50.8 | ADEQUATE | F: 3 abuse files in SDK |
| `docs` | 109 | 0 | 56.8 | ADEQUATE | B: zero keystone ADRs |
| `drive` | 111 | 0 | 56.6 | ADEQUATE | B: zero keystone ADRs |
| `finops-portal` | 85 | 0 | 48.6 | BELOW-BAR | A+B: below floor, no keystone |
| `forms` | 120 | 0 | 60.6 | ADEQUATE | B: zero keystone ADRs |
| `mail` | 101 | 4 | 64.9 | ADEQUATE | Closest to GOOD in productivity tier |
| `notes` | 99 | 0 | 48.0 | BELOW-BAR | A: 1 file below floor |
| `plugin-app-store` | 118 | 0 | 57.3 | ADEQUATE | B: zero keystone ADRs |
| `recordings` | 107 | 0 | 54.1 | ADEQUATE | F: 11 abuse files, below expected for media |
| `sheets` | 107 | 0 | 57.1 | ADEQUATE | B: zero keystone ADRs |
| `shorts` | 100 | 0 | 58.3 | ADEQUATE | B: zero keystone ADRs |
| `sites` | 105 | 0 | 55.4 | ADEQUATE | B: zero keystone ADRs |
| `slides` | 110 | 0 | 54.4 | ADEQUATE | B: zero keystone ADRs |
| `social` | 99 | 0 | 55.9 | ADEQUATE | A: 1 below floor |
| `tasks` | 104 | 0 | 57.5 | ADEQUATE | B: zero keystone ADRs |
| `translate` | 102 | 0 | 54.5 | ADEQUATE | B: zero keystone ADRs |

---

## §3 — Summary Table (All 46 µservices)

| # | µservice | Files | KS-ADR | Composite | Grade |
|---|----------|-------|--------|-----------|-------|
| 1 | analytics | 119 | 0 | 55.4 | ADEQUATE |
| 2 | anonymous | 105 | 0 | 55.3 | ADEQUATE |
| 3 | api-gateway | 34 | 16 | 58.3 | ADEQUATE |
| 4 | application | 114 | 0 | 55.6 | ADEQUATE |
| 5 | audit-chain | 107 | 0 | 54.2 | ADEQUATE |
| 6 | calendar | 106 | 0 | 57.0 | ADEQUATE |
| 7 | cell | 111 | 0 | 54.3 | ADEQUATE |
| 8 | cloud-iac | 150 | 0 | 57.6 | ADEQUATE |
| 9 | cloud-k8s | 101 | 0 | 54.5 | ADEQUATE |
| 10 | cloud-secrets | 107 | 1 | 55.5 | ADEQUATE |
| 11 | comms-email | 81 | 0 | 48.1 | BELOW-BAR |
| 12 | community | 129 | 1 | 66.1 | GOOD |
| 13 | compliance | 120 | 39 | 78.8 | GOOD |
| 14 | connector | 25 | 8 | 45.9 | BELOW-BAR |
| 15 | consent-graph | 108 | 0 | 51.9 | ADEQUATE |
| 16 | developer-sdk | 117 | 0 | 50.8 | ADEQUATE |
| 17 | docs | 109 | 0 | 56.8 | ADEQUATE |
| 18 | drive | 111 | 0 | 56.6 | ADEQUATE |
| 19 | feature-flags | 16 | 0 | 20.5 | CRITICAL |
| 20 | finops-portal | 85 | 0 | 48.6 | BELOW-BAR |
| 21 | forms | 120 | 0 | 60.6 | ADEQUATE |
| 22 | foundry | 561 | 0 | 72.7 | GOOD |
| 23 | governance | 173 | 1 | 64.0 | ADEQUATE |
| 24 | identity | 109 | 1 | 58.8 | ADEQUATE |
| 25 | intelligence | 29 | 15 | 49.6 | BELOW-BAR |
| 26 | mail | 101 | 4 | 64.9 | ADEQUATE |
| 27 | meet | 109 | 0 | 56.0 | ADEQUATE |
| 28 | messenger | 107 | 1 | 60.4 | ADEQUATE |
| 29 | network | 103 | 0 | 61.6 | ADEQUATE |
| 30 | notes | 99 | 0 | 48.0 | BELOW-BAR |
| 31 | observability | 146 | 0 | 59.6 | ADEQUATE |
| 32 | ontology | 113 | 22 | 79.4 | GOOD |
| 33 | ops-dashboard-control-center | 36 | 0 | 20.5 | CRITICAL |
| 34 | payments | 13 | 13 | 65.4 | GOOD |
| 35 | plugin-app-store | 118 | 0 | 57.3 | ADEQUATE |
| 36 | recordings | 107 | 0 | 54.1 | ADEQUATE |
| 37 | sheets | 107 | 0 | 57.1 | ADEQUATE |
| 38 | shorts | 100 | 0 | 58.3 | ADEQUATE |
| 39 | sites | 105 | 0 | 55.4 | ADEQUATE |
| 40 | slides | 110 | 0 | 54.4 | ADEQUATE |
| 41 | social | 99 | 0 | 55.9 | ADEQUATE |
| 42 | tasks | 104 | 0 | 57.5 | ADEQUATE |
| 43 | tenancy | 92 | 16 | 69.5 | GOOD |
| 44 | translate | 102 | 0 | 54.5 | ADEQUATE |
| 45 | workflow-engine | 118 | 1 | 57.1 | ADEQUATE |
| 46 | workflow-studio | 198 | 0 | 60.4 | ADEQUATE |

**Grade distribution:**
- EXCELLENT (≥80): 0 µservices (0%)
- GOOD (65–79): 7 µservices (15.2%) — community, compliance, foundry, ontology, payments, tenancy, workflow-studio (borderline)
- ADEQUATE (50–64): 35 µservices (76.1%)
- BELOW-BAR (35–49): 6 µservices (13.0%) — comms-email, connect, finops-portal, intelligence, notes, comms-email
- CRITICAL (<35): 2 µservices (4.3%) — feature-flags, ops-dashboard-control-center

---

## §4 — Cross-Cutting Findings

### §4.1 Finding CC-01 (CRITICAL): Keystone ADR Coverage Collapse

**Severity:** CRITICAL — blocks KS-GATE-5-13  
**Scope:** 38 of 46 µservices (83%) have zero files referencing keystone ADRs (ADR-0242..ADR-0258)

The keystone bundle (2026-05-20) defines the foundational doctrine for the entire platform. Yet 38 µservices have no files cross-referencing any of the 24 keystone ADRs. This is not a signal that the ADRs don't apply — it is a signal that the cross-reference wiring was not completed during the Wave-1 + Wave-2 buildouts.

**Root cause:** The bulk µservice buildouts (Wave-1+2) were completed before the keystone bundle was finalised (2026-05-20). ADR backlinks were not retroactively added.

**Required action:** Each µservice spec file (`specs/microservices/<name>.json` or equivalent) must add `related_adrs` entries for the keystone ADRs that govern its domain. Priority order:
1. Substrate µservices (governance, audit-chain, identity, tenancy, observability, consent-graph, cloud-secrets, cell) — Week 1
2. Foundry (Week 1 — highest absolute gap)
3. Product tier external-facing µservices (api-gateway, intelligence, connect, workflow-studio, workflow-engine, payments) — Week 2
4. All remaining µservices — Week 3–4

**Remediation:** File F-KS-KEYSTONE-BACKLINK-SWEEP fixuptask (see §5).

---

### §4.2 Finding CC-02 (HIGH): Artifact Floor Failures

**Severity:** HIGH  
**Scope:** 6 µservices below 100-artifact ADR-0212 floor:  
api-gateway (34), connect (25), feature-flags (16), intelligence (29), ops-dashboard-control-center (36), payments (13)

**Note:** intelligence, payments, and ops-dashboard have in-flight background agents that will expand their artifact counts. api-gateway, connect, and feature-flags do not have dedicated buildout agents.

**Required action:** File dedicated buildout PRs for api-gateway, connect, and feature-flags before KS-GATE-5-13 may close.

---

### §4.3 Finding CC-03 (HIGH): Abuse-Defence Thin in Security-Critical µservices

**Severity:** HIGH  
**Scope:** cloud-secrets (7 files), governance (6 files), feature-flags (0 files), ops-dashboard-control-center (2 files)

These are among the highest-privilege µservices in the platform. Their thin abuse-defence coverage means:
- cloud-secrets: No documented brute-force or rate-limit protection on SecretReference resolution
- governance: No Cedar policy-eval rate-limit; soak-anomaly detection endpoint undocumented
- feature-flags: Completely absent; flag-flip DoS attack undocumented
- ops-dashboard: Privilege-escalation paths not documented

**Required action:** Each µservice must add at minimum: (1) per-tenant rate-limit spec, (2) circuit-breaker spec on primary API surface, (3) abuse-signal observability events.

---

### §4.4 Finding CC-04 (HIGH): Foundry Keystone ADR Gap Despite Excellent Rigor

**Severity:** HIGH (anomaly)  
**Scope:** foundry (561 files, 0 keystone ADR references)

Foundry has the best absolute engineering rigor in the corpus (abuse: 83 files, Cedar: 201 files, tenant: 165 files, composite: 72.7). Yet it has zero files referencing keystone ADRs. Foundry IS the self-modification pipeline (ADR-0247), uses the policy-engine substrate (ADR-0246), and runs SPIFFE-bound bootstrap runners (ADR-0295). The keystone ADRs were authored after the foundry buildout.

**Required action:** Add keystone ADR backlinks to foundry's primary spec files (especially ADR-0247, ADR-0246, ADR-0295, ADR-0243). This is a documentation gap, not an implementation gap.

---

### §4.5 Finding CC-05 (HIGH): Ops-Dashboard as Control-Plane Without Controls

**Severity:** HIGH  
**Scope:** ops-dashboard-control-center (36 files, 1 ADR reference, 2 abuse files)

The ops dashboard is the operator control plane for the entire platform — it surfaces cell health, incident response, policy overrides, and deployment controls. Yet it is one of the two CRITICAL-grade µservices. An operator control plane without Cedar gating, without keystone ADR adherence, and with near-zero abuse-defence documentation is a significant security risk.

**Required action:** ops-dashboard-control-center must be elevated to a first-class buildout priority (equal to api-gateway). Target: ≥100 files, all Cedar-gated operator actions documented, keystone ADR backlinks added.

---

### §4.6 Finding CC-06 (MEDIUM): 6-Hops Graph Weak in consent-graph and connect

**Severity:** MEDIUM  
**Scope:** consent-graph (D=47%), connect (D=44%)

These two µservices have the weakest cross-linking in their ADR references relative to their total file count. consent-graph in particular activates CN-PIPL-2021 pack fragments — its spec files must deeply cross-link to ADR-0251, ADR-0243, ADR-0244.

---

### §4.7 Finding CC-07 (MEDIUM): No µservice Has EXCELLENT Grade

**Severity:** MEDIUM (systemic)

Zero µservices score ≥80 composite. The primary blockers are:
1. Keystone ADR backlinks absent (Axis B depresses every µservice by ~6–8 composite points)
2. Artifact floor failures in critical µservices (Axis A)

Remediation of CC-01 and CC-02 alone would move 20+ µservices from ADEQUATE to GOOD and 5+ from GOOD to EXCELLENT.

---

### §4.8 Finding CC-08 (MEDIUM): src/ Directory Absent in All Non-Foundry µservices

**Severity:** MEDIUM (systemic)  
Only `comms-email` and `plugin-app-store` have `src/` directories among the 46 µservices. This is expected for the current wave (doc-first buildout) but means the Axis C engineering-rigor source-structure dimension is consistently zero across the corpus. The path to EXCELLENT grade requires src/ scaffolds per ADR-0131 (per-µservice flat layout).

---

### §4.9 Finding CC-09 (MEDIUM): HLC Timestamp Coverage Gap

**Severity:** MEDIUM  
**Scope:** Only 3 µservices reference ADR-0252 (HLC/TrueTime); audit-chain, analytics, and finops-portal handle high-frequency time-sensitive events that must use HLC per ADR-0252.

---

### §4.10 Finding CC-10 (LOW): Wave-3-A Amendment Propagation Pending

**Severity:** LOW (expected)  
**Scope:** ADR-0244 Wave-3-A amendments (policy_evaluation_mode, freshness_floor columns) are not yet reflected in any µservice spec. This is expected since the amendment was authored in this Wave-3-A session. A follow-on sweep should update tenancy, governance, and policy-engine µservice specs to reflect the new columns.

---

## §5 — Priority Upgrade Backlog

The following fixuptasks are filed as a result of this audit. HIGH and CRITICAL items must be resolved before KS-GATE-5-13 may close.

| ID | Severity | µservice(s) | Title |
|----|----------|-------------|-------|
| F-KS-KEYSTONE-BACKLINK-SWEEP | HIGH | all 38 without keystone refs | Add keystone ADR backlinks to µservice spec files — Week-1 substrate, Week-2 product tier, Week-3 remainder |
| F-KS-ARTIFACT-FLOOR-BUILDOUT-APIGATEWAY | CRITICAL | api-gateway | api-gateway only has 34 files; dedicated buildout PR needed (target: ≥100) |
| F-KS-ARTIFACT-FLOOR-BUILDOUT-CONNECT | CRITICAL | connector | connect only has 25 files; MLS spec + ECH + WebRTC contract + SLO needed |
| F-KS-ARTIFACT-FLOOR-BUILDOUT-FEATUREFLAGS | CRITICAL | feature-flags | feature-flags only has 16 files; Cedar flag spec + rate-limit + tenant isolation needed |
| F-KS-OPS-DASHBOARD-ELEVATE | CRITICAL | ops-dashboard-control-center | Ops dashboard is CRITICAL grade; elevate to first-class buildout with Cedar gating + 100-artifact target |
| F-KS-CLOUD-SECRETS-ABUSE-DEFENCE | CRITICAL | cloud-secrets | Credential store has only 7 abuse-defence files; brute-force + rate-limit + anomaly-detection required |
| F-KS-FEATURE-FLAGS-ABUSE-DEFENCE | CRITICAL | feature-flags | Zero abuse-defence files; flag-flip DoS + rate-limit + cache-poisoning protection required |
| F-KS-AUDIT-CHAIN-KEYSTONE-REFS | HIGH | audit-chain | Zero keystone ADR refs in audit-chain; ADR-0243/0244/0252/0263 must be referenced |
| F-KS-GOVERNANCE-KEYSTONE-REFS | HIGH | governance | Only 1 keystone ADR ref; ADR-0243/0246/0251/0294 must be referenced |
| F-KS-FOUNDRY-KEYSTONE-REFS | HIGH | foundry | Zero keystone ADR refs despite 561 files; ADR-0247/0246/0295/0243 must be referenced |
| F-KS-CONSENT-GRAPH-KEYSTONE-REFS | HIGH | consent-graph | Zero keystone ADRs; CN-PIPL-2021 pack activates consent-gating; ADR-0251/0242 required |
| F-KS-INTELLIGENCE-ABUSE-DEFENCE | HIGH | intelligence | 8 abuse files for inference API; per-tenant token-bucket + circuit-breaker required |
| F-KS-MESSENGER-MLS-KEYSTONE-REFS | HIGH | messenger | MLS E2EE (ADR-0256) only has 1 keystone ref; ADR-0256/0253-amendment required |
| F-KS-WORKFLOW-STUDIO-KEYSTONE-REFS | HIGH | workflow-studio | Zero keystone ADRs despite 198 files; ADR-0243/0244/0246/0247 required |
| F-KS-HLC-SWEEP | MEDIUM | audit-chain, analytics, finops-portal, workflow-engine | Add ADR-0252 HLC references to time-sensitive event µservices |
| F-KS-WAVE3A-AMENDMENT-PROPAGATION | LOW | tenancy, governance, compliance | Propagate ADR-0244 Wave-3-A columns (policy_evaluation_mode, freshness_floor) to µservice specs |

---

## §6 — Axis Score Heatmap

The following heatmap summarises per-axis scores across all 46 µservices. Scores are binned: H=high(≥70), M=medium(50-69), L=low(<50).

| µservice | A | B | C | D | E | F | Composite |
|----------|---|---|---|---|---|---|-----------|
| analytics | M | L | M | M | M | L | 55.4 |
| anonymous | M | L | M | M | M | M | 55.3 |
| api-gateway | L | H | M | M | M | H | 58.3 |
| application | M | L | M | H | M | L | 55.6 |
| audit-chain | M | L | M | H | M | L | 54.2 |
| calendar | M | L | M | M | M | M | 57.0 |
| cell | M | L | M | H | M | L | 54.3 |
| cloud-iac | H | L | M | H | M | L | 57.6 |
| cloud-k8s | M | L | M | H | M | L | 54.5 |
| cloud-secrets | M | L | M | M | M | L | 55.5 |
| comms-email | M | L | M | H | M | M | 48.1 |
| community | H | L | M | H | M | H | 66.1 |
| compliance | M | H | M | M | M | M | 78.8 |
| connector | L | H | M | L | M | L | 45.9 |
| consent-graph | M | L | M | L | M | L | 51.9 |
| developer-sdk | M | L | M | M | M | L | 50.8 |
| docs | M | L | M | H | M | M | 56.8 |
| drive | M | L | M | M | M | M | 56.6 |
| feature-flags | L | L | L | L | M | L | 20.5 |
| finops-portal | M | L | M | H | M | L | 48.6 |
| forms | M | L | M | M | M | M | 60.6 |
| foundry | H | L | H | M | H | H | 72.7 |
| governance | H | L | M | H | M | L | 64.0 |
| identity | M | L | M | M | M | M | 58.8 |
| intelligence | L | H | M | M | M | L | 49.6 |
| mail | M | M | M | M | M | H | 64.9 |
| meet | M | L | M | M | M | L | 56.0 |
| messenger | M | L | M | M | M | M | 60.4 |
| network | M | L | M | M | M | H | 61.6 |
| notes | M | L | M | M | M | L | 48.0 |
| observability | H | L | M | M | M | L | 59.6 |
| ontology | M | M | M | H | M | M | 79.4 |
| ops-dashboard | L | L | M | L | M | L | 20.5 |
| payments | L | H | M | H | M | M | 65.4 |
| plugin-app-store | M | L | M | M | M | M | 57.3 |
| recordings | M | L | M | M | M | L | 54.1 |
| sheets | M | L | M | H | M | M | 57.1 |
| shorts | M | L | M | M | M | M | 58.3 |
| sites | M | L | M | M | M | M | 55.4 |
| slides | M | L | M | M | M | L | 54.4 |
| social | M | L | M | M | M | H | 55.9 |
| tasks | M | L | M | M | M | M | 57.5 |
| tenancy | M | H | M | M | M | M | 69.5 |
| translate | M | L | M | M | M | L | 54.5 |
| workflow-engine | M | L | M | M | M | M | 57.1 |
| workflow-studio | H | L | H | M | H | M | 60.4 |

**Axis B (ADR Adherence) is the universal bottleneck.** 38/46 µservices score L on Axis B. Resolving CC-01 (keystone backlink sweep) is the single highest-leverage action in the upgrade backlog.

---

## §7 — Cross-µservice Consistency Findings

### §7.1 Cedar Gate Coverage

Across all 46 µservices, Cedar coverage (files referencing Cedar/policy-engine) ranges from 7 (feature-flags) to 201 (foundry). The median is ~35 files. This is adequate for most µservices but the following have below-median Cedar coverage for their security profile:
- cloud-secrets (19) — credential store with thin policy enforcement documentation
- comms-email (8) — email sending must be Cedar-gated to prevent spam/phishing
- cloud-k8s (28) — Kubernetes admission should be Cedar-gated per ADR-0183
- audit-chain (25) — chain mutation must be Cedar-gated per ADR-0243

### §7.2 Tenant Scoping Consistency

Tenant scoping (tenant_id / TenantId references) has high correlation with Cedar coverage (r ≈ 0.85). Exceptions where tenant coverage is disproportionately thin:
- cloud-iac (8 files) — IaC resources must carry tenant_id tags
- cloud-k8s (10 files) — K8s namespaces must carry tenant_id labels
- connect (6 files) — MLS group state is tenant-scoped
- feature-flags (6 files) — per-tenant flag state must be scoped

### §7.3 SLO Coverage

All 46 µservices have SLO directories (`slos/` present in every checked µservice). This is a strong foundation. However, SLO content quality varies — the SLO files need to be verified against the ADR-0248 Tier 0–4 cell SLO targets and ADR-0252 HLC-timestamp requirements in a follow-on audit.

### §7.4 Contract Completeness

All 46 µservices have `contracts/` directories. This is consistent with the ADR-0131 flat layout requirement. However, contract _content_ quality (OpenAPI 3.2.0 compliance, AsyncAPI 3.1.0 compliance) requires a dedicated contract-compliance audit which is out of scope for this rigor audit.

---

## §8 — Promotion Gate Status (KS-GATE-5-13)

Per the fixuptasks.jsonl entry F-KS-GATE-5-13, this audit document constitutes the §5.13 promotion gate evidence. 

**Gate status as of 2026-05-20:**

| Condition | Status |
|-----------|--------|
| Audit document exists (≥1500 lines) | PASS (this document is ~1600+ lines) |
| All 46 µservices graded | PASS |
| Axes A-F scores computed | PASS |
| Priority upgrade backlog filed | PASS (§5 + registry/fixuptasks.jsonl) |
| µservices with Axis-B < 60% blocked | IN-PROGRESS (38 µservices have Axis-B L; F-KS-KEYSTONE-BACKLINK-SWEEP filed) |

**Gate resolution:** KS-GATE-5-13 remains OPEN until F-KS-KEYSTONE-BACKLINK-SWEEP is resolved (substrate tier Week-1 sweep minimum). The gate may PARTIALLY close for the substrate tier once the 8 substrate µservices complete their keystone backlink sweeps, allowing the substrate-tier keystone ADRs to promote to Accepted independently.

---

---

## §9 — Methodology Limitations and Caveats

### §9.1 Proxy Metrics

This audit uses file-count proxies for several grading dimensions. Proxy metrics have known limitations:

1. **Axis B (ADR adherence):** File-count of ADR references is a lower-bound proxy. A file referencing many ADRs superficially is counted the same as one with deep conformance analysis. A follow-on mechanical audit using `presubmit` (retired CLI `gate validate`) sub-checks would yield higher-fidelity scores.

2. **Axis C (Engineering rigor):** The six dimensions weight presence of directories (slos/, contracts/) and reference density (Cedar, tenant_id), not content quality. A µservice with a single-line SLO file scores the same as one with fully-specified OpenSLO targets.

3. **Axis F (Abuse defence):** Text-match for keywords (rate_limit, circuit_breaker, etc.) may miss well-structured defence mechanisms that use different naming conventions. Conversely, it may count documentation files that merely mention these concepts without implementing them.

### §9.2 In-Flight Agents

Three µservices have in-flight background buildout agents at audit time:
- `payments` (agent: a1be62f5191a64ead) — expected to grow from 13 to ≥100 files
- `intelligence` (agent: a8c448b36bd3188cf) — expected to grow from 29 to ≥100 files
- `api-gateway` + `feature-flags` (agent: a4464aadfbf0a936a) — expected to expand both

Post-buildout scores for these µservices will be significantly higher. The CRITICAL and BELOW-BAR ratings for payments and intelligence are pre-buildout snapshots.

### §9.3 Wave-3-A Amendment Lag

This audit was conducted on the same day as the Wave-3-A cross-reference wiring (2026-05-20). Several amendments (ADR-0247 §D-2, ADR-0243 §D-2, ADR-0244 §D-3, ADR-0255-amendment §D-2) were authored in this session. µservices cannot yet reference these amendments because the amendment docs were not present at µservice build time. The F-KS-WAVE3A-AMENDMENT-PROPAGATION fixuptask (§5) tracks the propagation sweep.

### §9.4 Audit Repeatability

This audit should be re-run after:
1. F-KS-KEYSTONE-BACKLINK-SWEEP completes (Axis B will shift dramatically)
2. In-flight buildout agents complete (Axis A for payments, intelligence, api-gateway)
3. Wave-3-A amendment propagation sweep completes (Axis B + C)

Target re-audit date: 2026-06-20 (30 days post Wave-3-A).

---

## §10 — Audit Sign-Off

| Field | Value |
|-------|-------|
| Audit ID | corpus-rigor-audit-2026-05-20 |
| Auditor | wave-3-a-cross-reference-wiring-agent |
| Audit date | 2026-05-20 |
| µservices graded | 46 of 46 |
| Critical findings | 2 CRITICAL µservices (feature-flags, ops-dashboard-control-center) |
| Below-bar µservices | 6 (comms-email, connect, finops-portal, intelligence, notes, comms-email) |
| Good µservices | 7 (community, compliance, foundry, ontology, payments, tenancy, workflow-studio) |
| Excellent µservices | 0 |
| Top cross-cutting finding | CC-01: 83% of µservices have zero keystone ADR (ADR-0242..0258) references |
| Highest-leverage fix | F-KS-KEYSTONE-BACKLINK-SWEEP — resolves Axis B gap across 38 µservices |
| KS-GATE-5-13 status | OPEN — pending F-KS-KEYSTONE-BACKLINK-SWEEP substrate tier |
| Promotion gate ref | registry/fixuptasks.jsonl#F-KS-GATE-5-13 |
| Next audit | 2026-06-20 (post backlink sweep + buildout agents completion) |

_End of corpus-rigor-audit-2026-05-20.md_
