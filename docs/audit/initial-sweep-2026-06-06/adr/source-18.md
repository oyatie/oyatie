# ADR Audit — SOURCE chunk 18

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 18
- **Slice requested:** `ls … | sort | sed -n "120,126p"` → ADR-0145 … ADR-0151
- **ADRs actually reviewed (7):** ADR-0145, ADR-0146, ADR-0147, ADR-0148, ADR-0149, ADR-0150, ADR-0151
- **Auditor posture:** READ-ONLY. Trust superseding ADR over stale front-matter. Cross-checked against keystone map `_map/canonical-posture-and-supersession-map.md`.

> **Chunk headline:** This is the "inter-µservice communication + dataplane + API-hygiene" cluster, all dated 2026-05-18, all spawned from the **PR #143 hyperscaler-shape remediation sweep**. ADR-0145 is the keystone (supersedes 0140/0141; the map's §1.1 archive verdict on 0140/0141 traces back here). 0146–0148 build the container/mesh substrate on top of it; 0149–0151 are three thin, well-formed API-hygiene canon ADRs. **Two real defects found:** (1) a corpus-wide **mislabel of ADR-0150** — the keystone map and ADR-0148's own reference list both call ADR-0150 "Kubernetes policy-engine separation (Cedar/Kyverno)", but on disk ADR-0150 is **Cursor Pagination**; the policy-engine-separation ADR is **ADR-0183** (Superseded by ADR-0379). (2) ADR-0147's runtime-ladder body is **internally contradictory** post-amendment (Cloud Hypervisor declared primary up top, but the §Consequences/§Alternatives prose still says "untrusted-content gets gVisor by default" and counts "three RuntimeClass objects: gvisor, kata-qemu, kata-fc").

---

### ADR-0145 — Inter-microservice communication: hyperscaler shape with opt-in Workflow + Ontology

- **decision_atom:** Inter-µservice communication uses **direct mTLS gRPC by default** under three weaker invariants (per-caller audit-chain seal emission, OTel trace propagation, Ontology projection of canonical entities as a read SUBSTRATE not a gateway); Workflow becomes **opt-in durable orchestration** (Step-Functions model), replacing the prior universal-mediator rule.
- **current_status:** Accepted (2026-05-18). Front-matter `supersedes: [ADR-0140, ADR-0141]`, `superseded_by: []`. Well-formed YAML front-matter.
- **disposition:** **KEEP.** Current, correct, load-bearing keystone for the whole chunk and a large downstream cluster (0146/0147/0148/0149/0151 all cite it). Non-conflicting.
- **governing:** n/a (this ADR governs; it supersedes 0140/0141 — the map's archive verdicts on 0140/0141 derive from this decision).
- **truth_flag:** **TRUE.** Reasoning is sound, evidence-cited (PR #143 review JSON artifacts), and the "no universal mediator" conclusion matches actual hyperscaler practice.
- **in_masterplan:** **PARTIAL / NA.** Decision is architecturally canonical but the ADR carries **no `planning_impact`/`masterplan_ref` front-matter** (it predates the planning-ssot-coverage gate; the map notes only 8.8% ADR binding). It DOES carry rich structured front-matter (deciders/supersedes/related/related_specs) — better than the 0149-0151 table-only ADRs. Should be bound into masterplan as the canonical comms-shape decision.
- **tensions:**
  - LINUX **ADR-0014** (one OCI/CRI frontend + pluggable IsolationBackend) and the LINUX framekernel ADRs sit downstream of this comms model — no direct conflict, but the LINUX "own-the-host" posture would reshape what "direct sibling egress" means. (Map fault-line §3.)
  - **Naming-note self-conflict:** §"Service-mesh substrate" (line 82) still cites `ADR-0148-service-mesh-cilium.md` and frames Cilium as "PRIMARY substrate, Istio Ambient Tier-2 opt-in" — but **ADR-0148 explicitly REWRITES that framing** into layered-zero-overlap. So ADR-0145's body now describes a mesh posture its own successor retired. STALE cross-ref inside an otherwise-TRUE ADR.
  - References `ADR-0140` as "Cedar policy enforcement substrate + cross-cutting-carriers exemption" yet also supersedes 0140 — fine, but note 0140 is double-duty (Cedar substrate AND carriers exemption); only the carriers-exemption portion is subsumed.
- **hyperscaler_challenge:** **ALIGNED.** Google (Stubby direct + Borg), AWS (direct service-to-service + Step Functions opt-in), Stripe (Twirp/gRPC direct) all do exactly this. The decision was explicitly made *to* match them and correctly rejects the ESB-2.0 universal mediator. No amend/archive pressure on the substance.
- **ai_slop:** Minor. The 5-vs-5 "Workflow vs direct gRPC" rubric is slightly over-precise (P99<500ms AND <2s latency budget AND read-only AND single-hop — five simultaneous ANDs is a high bar that will rarely all hold), but it's labeled a rubric with a worked-examples doc, so acceptable. The "70% 12-month regret probability" is fabricated precision (an LLM-review number presented as a metric) — flag but non-load-bearing.
- **refinement:** (1) Fix the stale §"Service-mesh substrate" framing to point at ADR-0148's layered model. (2) Add `planning_impact`/`masterplan_ref` front-matter. (3) Execute the closing instruction ("mark ADR-0140/0141 superseded_by: ADR-0145 in their front-matter") — the map §6 flags cross-ref discipline as inconsistent here.
- **consensus_needed:** **no.** Settled hyperscaler-aligned decision.

---

### ADR-0146 — Container base image: distroless `static-debian12:nonroot`

- **decision_atom:** The canonical base image for every Rust binary container is **`gcr.io/distroless/static-debian12:nonroot`** (USER 65532), enforced by the `oya gate validate container-base-image` lane; `scratch` requires an explicit per-case ADR carve-out.
- **current_status:** Accepted (2026-05-18). **Table-format header (no YAML front-matter)** — `Supersedes —`, `Superseded by —`.
- **disposition:** **KEEP** (with a minor AMEND for retired-vocab leakage). Decision is current and correct.
- **governing:** n/a.
- **truth_flag:** **TRUE** (substance) / **PARTIAL** on freshness — body contains retired-vocab residue.
- **in_masterplan:** **NO / NA.** No structured front-matter at all (table-only). Not bindable to the planning-ssot gate as written; would need front-matter backfill.
- **tensions:**
  - **Retired-vocab leakage:** body references "foundry / mail / recordings tiering" and `foundry-providers` calling Anthropic/OpenAI/Google APIs. Per map §2, **foundry is RETIRED → cloud-intelligence**. The example crate `foundry-providers` is dead branding. Also "all 33 µservices" vs ADR-0148's "32 µservices" vs ADR-0147 enumerates ~14 axes — **µservice-count drift** across the chunk.
  - Cites `oya.securityContext.podStandard65534` then says it accepts UID **65532** — a **65534-vs-65532 number mismatch** in the same sentence (line 82-83). Minor wrong detail.
  - References `oya-check-container-base-image` as "Layer-1 kernel-tier validator per ADR-0083" — consistent with the gate doctrine; fine.
- **hyperscaler_challenge:** **ALIGNED.** Google invented distroless and uses it for GCP control-plane; AWS Well-Architected + Stripe + Cloudflare all recommend distroless-for-Rust. Pinning Google's `static-debian12:nonroot` is exactly what a hyperscaler would do. No amend pressure on substance.
- **ai_slop:** Low. Citation list ("Anthropic public statements about their training-stack containers", "Stripe: distroless for everything") is lightly fabricated precision but directionally true. The standard distroless rationale (CA-certs, tzdata, /etc/passwd) is correct and concrete, not slop.
- **refinement:** (1) Strip `foundry`/`foundry-providers` retired branding → `cloud-intelligence`. (2) Fix the 65534/65532 typo. (3) Reconcile the µservice count (33 vs 32) corpus-wide. (4) Convert to YAML front-matter for masterplan binding.
- **consensus_needed:** **no.**

---

### ADR-0147 — Container sandboxing runtime ladder

- **decision_atom:** Container sandboxing is **workload-class-tiered (a runtime ladder)** not universal-gVisor: app-tier runs bare Linux+CIS restricted; untrusted-content / AI-inference / federation default to **Kata + Cloud Hypervisor (`kata-clh`)**; crypto workers to `kata-clh-sev-snp` (AMD SEV-SNP) or bare HSM; WASM to runwasi/wasmtime; gVisor demoted to an opt-in cold-start escape hatch.
- **current_status:** **Amended** (2026-05-18; amended same day to make Cloud Hypervisor primary in place of gVisor). YAML front-matter, `supersedes/superseded_by: []`.
- **disposition:** **AMEND.** The decision is sound and current, but the document is **internally contradictory** post-amendment and needs a body-reconciliation pass (not an archive).
- **governing:** n/a (lives under the §3 canonical posture for isolation/runtime alongside ADR-0254/0147/0200; consistent with the map).
- **truth_flag:** **PARTIAL.** The HEADLINE decision (Cloud Hypervisor primary, ladder model) is TRUE and map-aligned. But stale pre-amendment text was left in place, making parts WRONG-as-written:
  - §"Alternatives considered (e)" still says "untrusted-content gets **gVisor by default** with Kata available for sovereign tenants" — **contradicts** the amendment which makes `kata-clh` the default and gVisor the escape hatch.
  - §Consequences-Negative-1 counts "**Three RuntimeClass objects (`gvisor`, `kata-qemu`, `kata-fc`)**" — but the amendment adds `kata-clh`, `kata-clh-sev-snp`, `kata-clh-tdx` as the primary set; the enumerated three are exactly the legacy/secondary ones. Stale count.
  - The canonical mapping table (post-amendment, correct) and the alternatives/consequences prose (pre-amendment, stale) **disagree on the default runtime** — a reader can't tell which is binding without reading the amendment block.
- **in_masterplan:** **PARTIAL.** Good YAML front-matter + `related_specs` (hyperscaler-gates.json, iac-canonical-base.json) but no `masterplan_ref`/`planning_impact`. The `purpose:` block is well-authored.
- **tensions:**
  - `related: [… ADR-0140 (retired per ADR-0145) …]` — the ADR **correctly self-annotates** that 0140 is retired (good discipline, matches map). But it still references ADR-0140 "Cedar policy enforcement substrate + cross-cutting-carriers exemption" in §References without noting that only the carriers half is subsumed.
  - §"Cell scheduling awareness" cites "the ADR-0333 successor contract" and `microservices/tenancy/ARCHITECTURE.md` / `microservices/cloud-iac/ARCHITECTURE.md` — consistent with map §1.2 (ADR-0333 retired cell-as-µservice → pattern). This is **correctly reconciled** (someone already rewrote the dead `microservices/cell/*` path). Good.
  - **SOURCE-internal vs LINUX:** SOURCE picks Kata+CloudHypervisor+wasmtime on Talos; LINUX **ADR-0018** wants framekernel-as-host with no separate containerd. Map fault-line §3 — surfaced, not resolved.
- **hyperscaler_challenge:** **ALIGNED.** The core thesis (pick runtime per workload class, never universal) is exactly AWS (Firecracker per-class) / Google (gVisor + Confidential GKE) / Azure (Kata+Hyper-V) practice and is well-cited (Firecracker NSDI 2020, gVisor 2019, NIST 800-190). The Cloud-Hypervisor-over-gVisor amendment is defensible (Rust VMM, native SEV-SNP/TDX, lower steady-state overhead) and Azure Boost is a real precedent. Verdict argues **amend (reconcile body), not archive** — the decision is right, the prose is stale.
- **ai_slop:** Medium. Internal contradiction (the gVisor-default residue) is the main slop signal. The huge decider list (14 axes incl. `axis-shorts`, `axis-anonymous` — note `shorts` was merged into `social` per ADR-0334, so `axis-shorts` as a decider is **retired-vocab residue**). Overlong reference list with fabricated-precision overhead numbers ("~50 ms gVisor cold start versus 125-250 ms").
- **refinement:** (1) **Reconcile the body to the amendment** — rewrite Alternative-(e) and Consequence-Negative-1 so the default-runtime and RuntimeClass-count match `kata-clh`-primary; OR collapse the amendment into a clean single-state Decision (preferred). (2) Drop `axis-shorts` from deciders (retired). (3) Trim fabricated overhead numbers or cite the benchmark. (4) Consider whether "Amended" same-day should just be a re-issued Accepted.
- **consensus_needed:** **no** on substance; the contradiction is an editorial fix, not a founder decision. (If anything, the only founder-level question is whether the Talos+Kata substrate survives the LINUX framekernel ambition — that's owned by the §3 fault-line, not this ADR.)

---

### ADR-0148 — Service-mesh canonical: Cilium L3/L4 + Istio Ambient L7 (layered, zero overlap)

- **decision_atom:** The canonical service mesh is a **two-layer zero-overlap substrate**: **Cilium 1.19.x** owns CNI/L3/L4/eBPF-observability (Hubble), **Istio Ambient** owns SPIFFE mTLS (ztunnel) + L7 AuthorizationPolicy (per-namespace waypoint) with Cedar `ext_authz`; Cilium's own L7 is disabled; waypoint enrollment is per-µservice opt-in.
- **current_status:** Accepted (2026-05-18), with two in-file amendments (`amends_note` retiring the prior Cilium-primary framing + retiring ADR-0174; `amendment_2026_05_26` version-currency fix Cilium 1.16→1.19.4, Istio Ambient "track current stable"). YAML front-matter, `superseded_by: []`.
- **disposition:** **AMEND** (sound, current, but carries the same ADR-0150 mislabel + a count drift that need a reference-fix). Substantively KEEP-grade.
- **governing:** n/a (this is canonical for the mesh domain; map §3 lists it implicitly under the comms/posture rows).
- **truth_flag:** **TRUE** (substance) with one **WRONG cross-ref**:
  - **Line 257 reference: "ADR-0150 — Kubernetes policy engine separation (Cedar app authz vs Kyverno admission)."** On disk **ADR-0150 is Cursor Pagination**; the Cedar/Kyverno policy-engine-separation ADR is **ADR-0183** (`Superseded` → ADR-0379 Kubewarden). This is the **same mislabel that appears in the keystone map's §1.1** (which lists "ADR-0150 cursor-pagination" nowhere and instead attaches the policy-separation row to 0183). So ADR-0148 references a non-existent decision under number 0150. **Auditor note for synthesis: the map row for the Cedar/Kyverno separation is ADR-0183→0379, NOT 0150.**
  - §Decision and §Consequences also reference "ADR-0183 — Cedar policy compiler" (line 220) — correct — so the doc is *inconsistent with itself*: it cites both 0150 and 0183 for policy concerns.
- **in_masterplan:** **PARTIAL.** Strong YAML front-matter + `related_specs` + self-documenting amendment notes; no `masterplan_ref`. The version-currency amendment (dated 2026-05-26, with cited verification sources) is exemplary drift-hygiene — this ADR is the best-maintained in the chunk.
- **tensions:**
  - **Retired-brand residue:** the "5 µservices that handle L7-policed traffic (governance, **foundry**, audit-chain, application, workflow-studio)" — appears twice (lines 112, 193). **`foundry` is RETIRED → cloud-intelligence** (map §2). Should read `intelligence`.
  - µservice count: "32 µservices" (line 33) vs ADR-0146's "33" vs "the other 27 µservices" (line 193, implying 32 total with 5 enrolled). Internal + cross-ADR count drift.
  - References ADR-0149 and ADR-0150 in `related:` — 0149 is correct (idempotency, sibling concern), 0150 is the mislabel above.
  - Depends on ADR-0183 (Cedar compiler) which is **Superseded by ADR-0379** (Kubewarden default admission). ADR-0148's "Cedar app authz vs Kyverno admission" mental model is now **partially stale**: Kubewarden replaced Kyverno as default admission. The mesh's Cedar-ext_authz L7 path survives (map: "Cedar app-authz separation principle retained"), but any Kyverno reference is stale.
- **hyperscaler_challenge:** **ALIGNED.** "Cilium L3/L4 + Istio Ambient L7 layered" is precisely the GKE Dataplane V2 + Anthos/Solo.io reference pattern; sidecarless + per-namespace waypoint is the current hyperscaler-grade shape. CNCF-graduated primitives, no lock-in. The in-house-roadmap "IS-the-standard test" (KEEP CNCF-graduated, don't reimplement) is exactly how AWS/Google/Oracle build. No archive pressure.
- **ai_slop:** Low-medium. The ASCII 3-tier diagram is genuinely useful, not slop. Fabricated-precision micro-latencies ("~80-150 microseconds per request") and selective name-drops (Bell Canada/Capital One/Datadog) are mild. The doc is otherwise dense-but-real.
- **refinement:** (1) **Fix line 257**: ADR-0150 → ADR-0183 (and note 0183 is superseded by ADR-0379 Kubewarden; update "Kyverno admission" → "Kubewarden admission"). (2) Replace `foundry` → `intelligence` (2 occurrences). (3) Reconcile µservice count. (4) The version-currency amendment pattern here should be the template for the rest of the corpus.
- **consensus_needed:** **no** on the mesh decision. **YES** only insofar as the corpus-wide ADR-0150-vs-0183 mislabel is load-bearing for any automated `ADR-NNNN` index — see chunk notes.

---

### ADR-0149 — Idempotency Keys Canonical

- **decision_atom:** The `Idempotency-Key` header is **MANDATORY on every state-changing REST/gRPC operation** in every µservice (Stripe/AWS-ClientToken pattern), enforced by the `oya-check-idempotency-key-coverage` gate; client-supplied keys (not server-generated) to preserve at-least-once retry semantics.
- **current_status:** Accepted (2026-05-18). **Table/bullet header, no YAML front-matter.**
- **disposition:** **KEEP** (minor AMEND for front-matter + a wrong cross-ref).
- **governing:** n/a.
- **truth_flag:** **TRUE** (substance) / **PARTIAL** on a broken reference: References **"ADR-0153-outbox-pattern"** (twice). ADR-0153 on disk is `ADR-0153-outbox-pattern.md` — **correct**, the outbox ADR exists. BUT §Consequences says "Outbox pattern (**ADR-0153**) layers cleanly on top" while ADR-0148 (line 204/258) cites **"ADR-0153 — Observability backplane layering."** So **ADR-0153 is itself referenced under two different titles** across the chunk (outbox vs observability-backplane). On disk the filename is `outbox-pattern`; ADR-0148's "observability backplane" label for 0153 is likely **WRONG** (a second mislabel, parallel to the 0150 one). Flag for the chunk that owns 0152-0153.
- **in_masterplan:** **NO / NA.** No structured front-matter (table-only). Thin but complete decision; needs front-matter to bind.
- **tensions:**
  - Deciders list "axis-**foundry**, axis-all-microservices" — **`foundry` retired** (map §2). Retired-vocab decider residue (same pattern as 0147/0148/0150/0151).
  - Cross-ADR title drift on ADR-0153 (see truth_flag).
- **hyperscaler_challenge:** **ALIGNED.** Stripe has run client-supplied `Idempotency-Key` for ~10 years; AWS uses `ClientToken`. Mandatory-on-all-mutations + a coverage gate is exactly the hyperscaler API-hygiene bar. No amend pressure on substance.
- **ai_slop:** **none** of consequence. Tight, correct, minimal. The only filler is the redundant "33 µservices" repeated thrice.
- **refinement:** (1) Add YAML front-matter (id/status/supersedes/related) for masterplan binding and to match 0145/0147/0148. (2) Drop `axis-foundry` decider → `axis-intelligence`. (3) Confirm ADR-0153 title (outbox) and fix any "observability-backplane" mislabel elsewhere.
- **consensus_needed:** **no.**

---

### ADR-0150 — Cursor Pagination Canonical

- **decision_atom:** **Opaque cursor pagination is MANDATORY on every list endpoint** in every µservice and **offset pagination is BANNED** (AWS NextToken / Stripe `starting_after` pattern); cursors carry a `scope_hash` to prevent reuse across mismatched filters; enforced by `oya-check-cursor-pagination-coverage`.
- **current_status:** Accepted (2026-05-18). **Table/bullet header, no YAML front-matter.**
- **disposition:** **KEEP.** Correct, current, well-formed (for its thin class). The defect is *external* — other docs/the map mislabel this number.
- **governing:** n/a.
- **truth_flag:** **TRUE.** The decision itself is correct and complete. **CRITICAL META-FINDING:** this file is **Cursor Pagination**, NOT "Kubernetes policy-engine separation." The keystone map and ADR-0148 both attach the Cedar/Kyverno-separation identity to ADR-0150; that identity belongs to **ADR-0183** (Superseded→ADR-0379). Anyone trusting the map's number-keyed lookup for 0150 would archive the wrong thing. The cursor-pagination decision is NOT superseded and must NOT be archived.
- **in_masterplan:** **NO / NA.** No structured front-matter (table-only).
- **tensions:**
  - **Number-identity collision in the audit substrate** (not on disk): map/0148 think 0150 = policy-engine-separation; disk says 0150 = cursor-pagination. This is the single most important correction this chunk contributes. (The real policy-separation chain is ADR-0183→ADR-0379; ADR-0183 also relates to ADR-0148 the mesh.)
  - Deciders "axis-**foundry**" — retired-vocab residue.
- **hyperscaler_challenge:** **ALIGNED.** AWS NextToken, Stripe `starting_after`, GitHub cursor pagination — opaque-cursor-mandatory + offset-banned is textbook hyperscaler API hygiene. No amend pressure.
- **ai_slop:** **none.** Tight and correct. (`scope_hash` to prevent cross-filter cursor reuse is a real, non-obvious correctness detail — anti-slop, actually good.)
- **refinement:** (1) Add YAML front-matter. (2) Drop `axis-foundry` decider. (3) **Propagate the 0150-is-cursor-pagination correction** into the keystone map's supersession table and into ADR-0148 line 257.
- **consensus_needed:** **no** on the decision. The mislabel is a factual correction, not a founder ruling.

---

### ADR-0151 — X-Request-Id Propagation

- **decision_atom:** A canonical **`X-Request-Id` (ULID)** is generated at the edge if absent and propagated alongside OTel `traceparent` on every inter-µservice call for human/audit log correlation; `request_id` is **FORBIDDEN as a Prometheus/Mimir metric label** (high-cardinality) and may appear only as a Tempo span attribute / Loki log field.
- **current_status:** Accepted (2026-05-18). **Table/bullet header, no YAML front-matter.**
- **disposition:** **KEEP** (minor AMEND for front-matter).
- **governing:** n/a.
- **truth_flag:** **TRUE.** Correct, complete, and the high-cardinality-label prohibition is a genuinely correct observability rule (matches the Mimir/Prometheus cardinality doctrine and the map's observability posture, ADR-0383/0263).
- **in_masterplan:** **NO / NA.** No structured front-matter (table-only).
- **tensions:**
  - Deciders "axis-**foundry**, axis-observability" — `foundry` retired-vocab residue.
  - Correctly references ADR-0145 Invariant 2 (OTel propagation) — clean, consistent with this chunk's keystone.
  - References `docs/standards/request-id-canonical.md` and `microservices/observability/contracts/metric-naming-convention.md` — consistent path conventions; observability µservice owns the cardinality rule (aligns with the Loki/Tempo/Mimir stack of ADR-0383).
- **hyperscaler_challenge:** **ALIGNED.** AWS `x-amzn-RequestId`, GCP request-scoped IDs — a short correlation id distinct from trace context is exactly hyperscaler practice, and the "don't tag metrics with request_id" rule is precisely what AWS/Google observability guidance says. No amend pressure on substance.
- **ai_slop:** **none.** Even the "~26 bytes extra header" cost is a correct, concrete tradeoff note, not fabricated. Deferring W3C `baggage` is a sound, honest scoping decision.
- **refinement:** (1) Add YAML front-matter. (2) Drop `axis-foundry` decider. (3) Consider stating the ULID-vs-UUIDv7 choice rationale (currently asserted without justification — minor).
- **consensus_needed:** **no.**

---

## Chunk notes for synthesis

**1. One provenance, one date.** All 7 ADRs are dated 2026-05-18 and trace to the **PR #143 "hyperscaler-shape" remediation sweep**. ADR-0145 is the keystone; 0146/0147/0148 are the substrate (image → runtime → mesh); 0149/0150/0151 are three thin API-hygiene canon ADRs (idempotency / cursor-pagination / request-id). Treat the chunk as one coherent cluster when binding to masterplan: it is the "inter-µservice contract + dataplane" package.

**2. THE load-bearing correction — ADR-0150 identity mismatch.** Both the **keystone map** and **ADR-0148 (line 257)** treat **ADR-0150 as "Kubernetes policy-engine separation (Cedar/Kyverno)."** On disk **ADR-0150 = Cursor Pagination** (Accepted, not superseded). The policy-engine-separation ADR is **ADR-0183** (`Superseded` → **ADR-0379** Kubewarden). **Action for synthesis:** correct the supersession graph so the Cedar/Kyverno-separation row is keyed to 0183→0379, and never archive ADR-0150-cursor-pagination. This is the strongest evidence in this chunk that the map's number-keyed lookups must be re-derived from on-disk titles, not trusted at face value (consistent with map §6.3's own warning about index-poisoning).

**3. ADR-0153 is referenced under two contradictory titles.** ADR-0149 (and the filename) call ADR-0153 the **outbox-pattern** ADR; **ADR-0148 calls ADR-0153 "Observability backplane layering."** On disk it is `ADR-0153-outbox-pattern.md`. The 0152-0153 chunk owner should confirm which is correct and fix the mislabel — likely a second number-title drift parallel to the 0150 one. (Pattern: this corpus has multiple ADRs whose *title-as-cited* ≠ *title-on-disk*.)

**4. ADR-0147 is the one document needing real editorial work (AMEND).** Its same-day amendment made Cloud Hypervisor primary but left pre-amendment prose that still says "untrusted-content gets gVisor by default" and counts "three RuntimeClass objects (gvisor, kata-qemu, kata-fc)." Decision is TRUE and map-aligned; the *document* is internally contradictory. Either collapse the amendment into a single clean Decision or rewrite Alternative-(e)/Consequence-Negative-1. Everything else in the chunk is KEEP-grade.

**5. Corpus-wide retired-vocab leakage (`foundry`).** `foundry` appears as a live term in **5 of 7** ADRs: ADR-0146 ("foundry/mail/recordings tiering", `foundry-providers` crate); ADR-0148 (governance/**foundry**/audit-chain among the 5 waypoint µservices, ×2); ADR-0149/0150/0151 all carry **`axis-foundry`** as a decider. Per map §2 `foundry` is RETIRED → **cloud-intelligence** (consumer AI) / **governance** (CI). Every occurrence here is brand-residue (MFL-0002/0003 class). Also ADR-0147 carries **`axis-shorts`** as a decider — `shorts` merged into `social` per ADR-0334. These are mechanical retired-vocab fixes, not architecture changes.

**6. Front-matter bifurcation poisons masterplan binding.** ADR-0145/0147/0148 have rich YAML front-matter (deciders, supersedes, related, related_specs); ADR-0146/0149/0150/0151 use a **table/bullet header with NO YAML front-matter** and so cannot be bound by the planning-ssot-coverage gate (frontmatter `masterplan_ref` + supersession-aware). Given the founder GOAL (masterplan = SSOT), the four header-only ADRs need front-matter backfill before they can be machine-bound — regardless of whether the resolution is masterplan-authored-as-SSOT or generated-from-ADRs (map §4 open question). **Flag under both readings:** if generated-from-ADRs wins, these four ADRs literally cannot emit masterplan rows without front-matter; if masterplan-as-authority wins, these four can't be bound back without `masterplan_ref`. Either way they need structured front-matter.

**7. µservice count drift.** "33 µservices" (0146, 0149, 0150), "32 µservices" (0148), ~14 axes enumerated (0147 deciders). A trivial but real consistency defect across the chunk; whatever the true count, it should be a single sourced number (probably from a /specs manifest), not re-asserted per ADR.

**8. Hyperscaler verdict for the whole chunk: uniformly ALIGNED.** Every decision here is something Google/AWS/Azure would actually make — direct gRPC + opt-in Step-Functions (0145), distroless (0146), per-workload runtime ladder (0147), Cilium+Istio-Ambient layered mesh (0148), mandatory idempotency keys (0149), opaque cursor pagination (0150), request-id correlation with metric-cardinality discipline (0151). No archive-for-misalignment candidates. The only dispositions above KEEP are **AMEND-for-internal-consistency** (0147 contradiction; 0148 mislabel) and **AMEND-for-front-matter/retired-vocab** (0146/0149/0150/0151). **Zero ARCHIVE, zero SUPERSEDE, zero garbage in this chunk.**

**9. Cross-chunk tensions to hand to synthesis:**
   - ADR-0145's §"Service-mesh substrate" still describes the *pre-0148* "Cilium primary / Istio Tier-2 opt-in" framing that **ADR-0148 explicitly retired** — STALE intra-corpus cross-ref.
   - ADR-0148 depends on ADR-0183 (Cedar/Kyverno separation), which is **superseded by ADR-0379** (Kubewarden) — so 0148's "Kyverno admission" mental model is partially stale (Cedar L7 path survives; Kyverno→Kubewarden). Owner of the 0183/0379 chunk should confirm 0148 picks up the rename.
   - LINUX-vs-SOURCE isolation fault-line (map §3): SOURCE here doubles down on Kata+Cloud-Hypervisor+wasmtime on Talos (0147) and Cilium/Istio mesh (0148); LINUX ADR-0014/0018 want framekernel-as-host + owned IsolationBackend. Surfaced, not resolved.
