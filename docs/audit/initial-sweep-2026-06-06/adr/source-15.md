# ADR Audit — SOURCE chunk 15

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 15
- **Slice range (ls sed 99,105p):** ADR-0121, ADR-0122, ADR-0123, ADR-0124, ADR-0128, ADR-0129, ADR-0130
- **ADRs actually reviewed:** 7 (0121, 0122, 0123, 0124, 0128, 0129, 0130)

> Read-only audit. Trust the *superseding* ADR over stale front-matter (keystone map §6). The slice is non-contiguous (0125/0126/0127 fall outside the 99–105 window).

---

### ADR-0121 — On-prem Kubernetes stack: vanilla kubeadm + containerd + Istio + Envoy

- **decision_atom:** The on-prem KR primary cell runs vanilla upstream Kubernetes (kubeadm + containerd + Istio/Envoy) for maximum OKE-parity — *now superseded by Talos+CAPI+ArgoCD as the canonical node-OS/fleet substrate.*
- **current_status:** `Superseded` (front-matter `superseded_by: [ADR-0375]`); body header confirms "Superseded by ADR-0375".
- **disposition:** ARCHIVE.
- **governing:** ADR-0375 (Talos immutable node-OS + CAPI + ArgoCD; keystone §1.1 + §3 Orchestration). Also retires onprem k3s install IP.
- **truth_flag:** STALE (was TRUE at decision time 2026-05-16; the kubeadm/containerd/Istio stack is now the abandoned approach — keystone §2 "Talos rejected for primary cell" is itself the line that got reversed).
- **in_masterplan:** NO. Not referenced in MASTERPLAN.md; carries no `planning_impact`/`masterplan_ref`. Correctly so — it is superseded; only ADR-0375's decision belongs in the masterplan.
- **tensions:** (1) Body explicitly *rejects* Talos Linux for the primary cell ("k0s, MicroK8s, Talos Linux are also rejected … Same OKE-parity argument") — directly contradicted by ADR-0375 which *adopts* Talos. Clean supersession, but a sharp reversal worth recording in provenance. (2) Istio/Envoy service-mesh assumption (ADR-0044) — check whether ADR-0375's stack retains Istio or moves to a different mesh; not resolved here. (3) Retired-brand leakage: "Foundry workloads", `axis-foundry` owner — `foundry` brand is RETIRED (keystone §2). (4) Fault-line §3: SOURCE assembles substrate (Talos) vs LINUX ADR-0025 wants a Rust "Talos" — 0121 is the pre-Talos ancestor of that tension.
- **hyperscaler_challenge:** ALIGNED-but-superseded. Google/AWS/Azure would absolutely pick vanilla upstream k8s over k3s for a regulated primary cluster (conformance, parity). But hyperscalers run immutable, declaratively-managed node OSes (GKE COS, Bottlerocket, Azure Linux) — which is *exactly* the move to Talos. So the hyperscaler lens argues FOR the archive: the successor (0375) is the more hyperscaler-native choice. Verdict: archive.
- **ai_slop:** Low. The renumber_note is unusually long but is legitimate provenance (genuine ADR-0119 collision). Minor fabricated-precision smell: exact RAM idle figures (70/80/250 MB) and "~10 min setup" stated as fact. The Debian-13 nftables/kube-proxy-segfault detail is oddly specific operational lore for an ADR.
- **refinement:** This ADR is fully superseded; no refinement needed beyond confirming archive. If kept as historical record, add a one-line pointer at top noting Istio's fate under ADR-0375. Strip/annotate `axis-foundry` owner token as retired.
- **consensus_needed:** no. Supersession is clean and on-disk.

---

### ADR-0122 — Ontology crate rename (retire "object-graph" naming)

- **decision_atom:** Rename the `oya-platform-object-graph-kernel` crate (and siblings) to `oya-ontology-*`, matching the ratified Object-Graph→Ontology glossary rename and Palantir-Foundry terminology.
- **current_status:** `Accepted` (front-matter `supersedes: [] / superseded_by: []`).
- **disposition:** KEEP (terminal rename; well-formed). Candidate to MERGE into a single "ontology naming + KG consolidation" record alongside ADR-0130 and ADR-0055 for masterplan compactness.
- **governing:** n/a (live). It is itself the governing rename, alongside ADR-0055 (object-graph→ontology) and ADR-0130 (KG-registry→ontology type-system); keystone §2 retired-vocab "object-graph → ontology".
- **truth_flag:** TRUE.
- **in_masterplan:** NO/PARTIAL. Not cited in MASTERPLAN.md. The *outcome* (ontology is the canonical term; crates are `oya-ontology-*`) belongs in the masterplan vocabulary/naming surface; the mechanical git-mv is execution detail, not masterplan-worthy.
- **tensions:** (1) Pairs tightly with ADR-0130 (same day-after, same "ontology owns the type system" doctrine, same D-09 "Ontology+KG are one product" pick) — they should be cross-linked; 0130's `References` cites 0122 but 0122 does not forward-reference 0130. (2) Retired-brand note: "Palantir-Foundry term" is an external-product citation (fine), but the doctrine it enacts predates the `foundry`-brand retirement — no conflict, just era. (3) `/specs/microservices/ontology.json` is the binding spec; verify the rename was actually propagated there (ADR claims `crate_refs_current` updated — unverified this pass).
- **hyperscaler_challenge:** ALIGNED. Naming hygiene / eliminating a stale token from the canonical surface is exactly what a mature platform org does (Google's internal style councils, AWS service-rename discipline). The "fix drift in-PR, not as deferred successor-IP" (OP-11) posture is hyperscaler-grade. Verdict: keep.
- **ai_slop:** Low-moderate. Some ceremony: a full "Sunset/Reversal" + "data_loss_class: none" + verification-grep block for a pure `git mv` is heavyweight, but it matches house style. The "BNF slot-2 cross-cutting backbone / 12-layer enum" naming-justification prose is borderline fabricated-precision — invokes an elaborate internal grammar to justify an obvious rename.
- **refinement:** Fold the naming justification to one line. Cross-link forward to ADR-0130. In a masterplan, this becomes a single vocabulary entry, not a standalone decision.
- **consensus_needed:** no.

---

### ADR-0123 — Hyperscaler maturity claim gate

- **decision_atom:** "We are hyperscaler mature" is a forbidden phrase unless an evidence-gated registry (`/specs/hyperscaler-gates.json`) marks the claim allowed and all required maturity gates (product depth, pipeline, toolchain, CI/CD, UX, guardrails, safety, competitive response, HG-VCS) carry fresh evidence; the retired HG-GRIT gate must be absent.
- **current_status:** `Accepted`.
- **disposition:** KEEP (governance gate, sound, currently `claim_status=blocked` by design) — with an AMEND flag for retired-vocab drift (see tensions).
- **governing:** n/a (live). Pairs with ADR-0128 (the invariant *content* this claim gate references) and ADR-0129 (honest-claims gate). Retired HG-GRIT per ADR-0116.
- **truth_flag:** PARTIAL. The decision (evidence-gate the maturity claim) is TRUE and durable. But specific gate IDs/vocabulary are drifting: it names **HG-VCS**, "**Oya VCS** claim/verify/done/promote", and "**oya-vcs-admission**" as forward authority — the entire bespoke `oya vcs` layer was *retired* by ADR-0363 (merge/CI authority moved to Prow-shaped cloud-ci/oya-ci + Tide). So the HG-VCS sub-gate and the `oya vcs` migration_note are STALE.
- **in_masterplan:** PARTIAL. The *principle* (claims are evidence-gated, no unsourced "hyperscaler mature") is exactly the kind of governance invariant the masterplan should assert. The specific gate registry is a `/specs/*.json` artifact, consistent with masterplan-as-authority + specs-bind-in. Not currently cited in MASTERPLAN.md.
- **tensions:** (1) Forward-references `oya-vcs-admission`/HG-VCS as "forward authority" — superseded by ADR-0363/0513 (`oya-ci` Tide). Needs reconciliation. (2) "Foundry/Oya VCS pipeline" string (L37) — retired `foundry` brand. (3) Workflow Studio framed as "first hero product" — cross-check against ADR-0334 (shorts→social) / product-catalog churn; not contradicted but verify the 11-product list matches ADR-0128's. (4) Competitor list (n8n/Temporal/Camunda/Argo/Zapier/Power Automate/Linear) is a moving target — "source-backed competitor rows" requirement is good, but the named set will rot.
- **hyperscaler_challenge:** ALIGNED (strongly). This is precisely how AWS/Google govern maturity/GA-readiness claims — operational-readiness reviews, evidence-gated launch checklists, banned marketing superlatives without data. "The claim gate is mature before the product can honestly claim maturity" is a genuinely hyperscaler-grade stance. Verdict: keep; amend only the retired VCS gate vocabulary.
- **ai_slop:** Moderate. The "Context" paragraph is somewhat sermon-like ("Oyatie is an ecosystem, not a single feature"). The long capability-surface enumerations (plan, pipeline, toolchain, CI/CD, development cycle, product depth, UX, ease of use, guardrails, safety, competitive response) read as exhaustive-list hedging. Core decision is crisp though.
- **refinement:** Replace HG-VCS / `oya vcs` references with the post-0363 authority (`oya-ci`/Tide, cloud-ci required contexts). Decouple the competitor list into a versioned spec so ADR text doesn't rot. Confirm the 11-product set is single-sourced with ADR-0128.
- **consensus_needed:** no (the decision stands; the VCS-vocabulary amendment is mechanical, governed by ADR-0363).

---

### ADR-0124 — Own merge-queue policy (webhook-driven, GitHub-merge-queue-free)

- **decision_atom:** Because GitHub's native merge queue is unavailable on the repo plan, run `dev` with `strict:false` (IaC-encoded + drift-checked) and implement a bespoke webhook-driven merge queue (Foundry VCS kernels) with file-overlap clustering to break the O(N²) rebase cascade — *now retired: the entire bespoke VCS/merge-queue substrate is superseded.*
- **current_status:** Front-matter `status: accepted`, `superseded_by: none` — **STALE.** De facto superseded.
- **disposition:** ARCHIVE (superseded).
- **governing:** **ADR-0363** (retire agentic-VCS Foundry → plain git + Forgejo; explicitly supersedes ADR-0110/0112/0113, the exact substrate 0124 is built on) → **ADR-0513** (merge automation moves to Prow-shaped cloud-ci/oya-ci **Tide**, folding ADR-0111 projected-state semantics into CI/admission). ADR-0363 body: "merge automation belongs in cloud-ci/oya-ci Tide (ADR-0513), not in `oya vcs` or Forgejo custom patches"; "GitHub merge-queue/branch-protection as the substrate — rejected: GitHub is bootstrap-only."
- **truth_flag:** STALE/WRONG-now. The O(N²) problem analysis was TRUE; the chosen solution (own a GitHub-Actions-webhook-driven merge queue on Foundry VCS kernels) is now WRONG vocabulary and WRONG substrate — those ~13 orchestration crates (changeset-SM, merge-queue, merge-queue-conflict, webhook-receiver) are documented in ADR-0363 as "0–1 dependents, never deployed," scheduled for deletion.
- **in_masterplan:** NO (and must NOT be). Not in MASTERPLAN.md. The masterplan should carry only the post-0363/0513 merge story (plain git + Forgejo branch-protection/required-checks/auto-merge + Tide).
- **tensions:** (1) **Forge fault-line (keystone §5):** 0124 is built entirely on **GitHub** (GitHub Actions workflows, GitHub branch-protection REST, GitHub webhook events, owner `jason931225`). Source canon retired this for self-hosted Forgejo (ADR-0363/0374/0387). Founder's migration directive = GitHub — so 0124 *aligns with the founder's GitHub directive* but *contradicts source canon*. This is the cleanest in-corpus artifact of the GitHub-vs-Forgejo three-way tension. (2) Depends on ADR-0111 which is still `Proposed` (not Accepted) — 0124 built production branch-protection policy on a Proposed dependency. (3) Retired-brand: 8 `foundry`/`oya-foundry-vcs-*` references (crate names) — retired per ADR-0335/0363. (4) ADR-0129 later binds "honest claims" partly because merge-queue plans over-claimed lanes — 0124's 20-row blocker taxonomy with "planned" health/poller lanes is exactly the deferred-claim pattern 0129 polices.
- **hyperscaler_challenge:** MISALIGNED (as built) / the *problem* is real. A hyperscaler would NOT hand-roll a webhook-driven merge queue on a CI runner — they run trunk-based dev with a server-side speculative-merge engine (Google's submit queue / Tide / Zuul / GitHub MergeQueue / GitLab merge trains). ADR-0363/0513's move to **Tide** (Prow's merge-automation component) is the actually-hyperscaler-native answer. So the hyperscaler lens strongly argues FOR archive in favor of 0513. Verdict: archive; adopt Tide.
- **ai_slop:** Moderate-high. The 20-row merge-blocker taxonomy is impressively thorough but is largely *speculative spec for un-built code* (Phases 2/3 are "follow-on PR", lanes "planned") — fabricated operational precision for a system ADR-0363 says was never deployed. "25 open PRs drain on first-CI-green" is a confident claim about un-wired logic.
- **refinement:** Archive with a pointer to ADR-0363→0513. If any salvage: file-overlap clustering insight and the blocker taxonomy could inform `oya-ci` Tide config, but the substrate (GitHub Actions + Foundry kernels) is dead. Fix the front-matter (`superseded_by: [ADR-0363, ADR-0513]`).
- **consensus_needed:** **YES** — this is load-bearing for the forge fault-line. Question phrased below.

---

### ADR-0128 — Hyperscaler architecture invariants (canonical spec + portfolio binding)

- **decision_atom:** `specs/hyperscaler-architecture-invariants.json` (35 INV-* invariants across reliability/security/operational-excellence/performance/cost/sustainability) is the canonical machine-readable definition of "hyperscaler-grade"; product PRDs must cite their required INV-* set with fresh evidence (enforcement advisory until the validator+tests+workflow+branch-protection land together).
- **current_status:** `Accepted`; `enforcement_status: advisory-until-product-prd-validator`.
- **disposition:** KEEP (sound, well-scoped, honestly time-boxed enforcement).
- **governing:** n/a (live). Complements ADR-0123 (claim gate) and ADR-0114 (canary/progressive-delivery, referenced by INV-PROGRESSIVE-DELIVERY). Demotes `docs/standards/hyperscaler-best-practices.md` to research context.
- **truth_flag:** TRUE (with one PARTIAL caveat: the 11-product list embeds the retired `foundry` product — see tensions).
- **in_masterplan:** PARTIAL/should-be-YES. This is the single most masterplan-relevant ADR in the chunk: it defines portfolio-wide architectural invariants — exactly the "true decisions every product must honor" the founder wants captured. The invariants live in a `/specs/*.json`, consistent with both masterplan readings (authored-authority binds specs in; or ADR-front-matter generates). Not currently cited in MASTERPLAN.md → backfill candidate.
- **tensions:** (1) Lists "**foundry**" as one of 11 products (L83) — `foundry` brand RETIRED (ADR-0335→intelligence + governance; keystone §2). The INV catalog's `per_product_required_compliance[foundry]` is stale and must remap to intelligence/governance. (2) Honest about advisory enforcement — but that means the "binding source of truth" is not yet binding; mild claim/reality gap that ADR-0129's honest-claims gate exists to police (0128 cites 0129's sibling correctly via 0123). (3) Cites ADR-0119 flat-root topology as the schema basis (note: ADR-0119 itself has the renumber collision history per 0121's renumber_note — namespace hygiene). (4) Invariant set is genuinely AWS/Google-derived (Builders Library, SRE Book cited) — low fabrication risk.
- **hyperscaler_challenge:** ALIGNED (strongly). This IS the hyperscaler playbook codified: cell isolation + shuffle-sharding (AWS), static stability, idempotency, SLSA/Cosign supply chain, four golden signals + USE method, progressive delivery, data residency/perimeter, FinOps tagging. A single versioned spec + one binding ADR (rejecting "35 ADRs for 35 rules") is the right granularity. The honest "advisory-until" posture is mature. Verdict: keep; the only amend is remapping the retired `foundry` product slot.
- **ai_slop:** Low. Citations are real and load-bearing (AWS Builders Library, Google SRE, Azure WAF, Stripe, Palantir, Linear). The category/count table is generated-from-data. Some breadth but justified for a portfolio standard.
- **refinement:** Remap the `foundry` product entry to `intelligence`/`governance` in both the ADR's 11-product list and the spec's `per_product_required_compliance`. Backfill the 35 INV-* set into the masterplan as the architectural-invariant baseline. Track the advisory→active enforcement transition as a dated gate.
- **consensus_needed:** no on the decision; minor flag: confirm the canonical product roster (is it 11? does it include retired `foundry`/`shorts`?) — but that's a vocabulary cleanup, not a contested ruling.

---

### ADR-0129 — ChangeSet Plan DAG and Honest Claims Gate

- **decision_atom:** The existing ImplementationPlan front-matter `id` (Mxx-Pxx-IP-xxx) IS the canonical ChangeSet identity (no new `changeset_id` field); the active `oya gate validate honest-claims` lane blocks deferred-but-active claims, unsourced "hyperscaler mature" claims, and invalid plan-graph edges (duplicate IDs, cycles, asymmetric serialization, unguarded global-artifact write conflicts).
- **current_status:** `Accepted`; `enforcement_status: active`.
- **disposition:** KEEP (active gate, full enforcement slice landed, well-formed).
- **governing:** n/a (live). Companion to ADR-0123 (claim gate) and ADR-0128 (invariants); references ADR-0116/0124.
- **truth_flag:** TRUE (with a STALE *reference* to ADR-0124 in `related`, since 0124 is now superseded by 0363 — see tensions).
- **in_masterplan:** PARTIAL. The doctrine (plan IDs = ChangeSet identities; honest-claims is enforced) is masterplan-relevant as a planning/governance invariant. Note the **direct collision** with the masterplan-SSOT open question: ADR-0129 binds the *existing* IP `id` corpus as canonical, which is the "ADRs/plans are the authored SSOT" lineage — relevant to the authored-vs-generated founder question (keystone §4).
- **tensions:** (1) `related: ADR-0124` — 0124 is superseded by 0363; the merge-queue context that motivated 0129 ("two related gaps in the merge-queue plan") is now historical. The honest-claims *gate* survives the merge-queue's retirement, but the cross-ref should point at the surviving CI authority. (2) Plan-graph dir default `.omc/plans/milestones` — verify this path is still canonical post-consolidation. (3) Reinforces masterplan authored-vs-generated tension: "IP `id` is the narrowest canonical ChangeSet identity available today" leans authored-corpus-is-SSOT, partially in tension with planning-ssot-drift-prevention.md's "masterplan.json is the one authority." (4) `oya gate validate honest-claims` naming uses the live `oya-governance-*` prefix (good — not the retired `oya-foundry-fitness-*`).
- **hyperscaler_challenge:** ALIGNED. Reusing existing stable IDs instead of minting a parallel `changeset_id` (rejecting a mass-edit migration) is exactly the pragmatic, drift-minimizing call Google/AWS make. A pre-merge DAG validator (cycles, serialization symmetry, global-write conflicts) mirrors monorepo presubmit gating. Banning "active claim + deferred delivery" without an advisory boundary is genuine honesty-engineering. Verdict: keep.
- **ai_slop:** Low. Crisp field-contract table; concrete CLI/test verification. Minimal filler.
- **refinement:** Update `related` to reference the surviving merge/CI authority (ADR-0363/0513) instead of (or alongside) the retired 0124. Confirm `.omc/plans/milestones` is current. Tag this ADR in the masterplan authored-vs-generated decision file as evidence for the "plans-are-authored-SSOT" reading.
- **consensus_needed:** no on the gate itself. (It does feed the broader masterplan-SSOT consensus question, owned elsewhere.)

---

### ADR-0130 — Deprecate `registry/knowledge-graph-semantic.json`, migrate to Ontology type system

- **decision_atom:** Delete `registry/knowledge-graph-semantic.json` and inline its full content (36 node types, 27 edge types, 19 invariants, 11 query examples) into `specs/products/ontology.json#type_system`, so the Ontology product owns its type system directly (Palantir pattern), with no tombstone (OP-11 no-compat-seams); kinetic/dynamic KG layers untouched.
- **current_status:** `Accepted` (no front-matter block — status in body header only; see ai_slop).
- **disposition:** KEEP — strong candidate to MERGE with ADR-0122 (+ ADR-0055) into one "Ontology consolidation" masterplan entry.
- **governing:** n/a (live). Implements D-09 (Ontology+KG = one product) and ADR-0122 (crate rename). Keystone §2 retired-vocab "knowledge-graph-registry → ontology"; keystone §1.2 lists ADR-0130 as deprecating the KG-registry file.
- **truth_flag:** TRUE.
- **in_masterplan:** NO/PARTIAL. Not in MASTERPLAN.md. The durable fact (Ontology product owns the semantic type system inline; KG-semantic registry retired) belongs in the masterplan's data/ontology posture. This ADR also notably lacks YAML front-matter entirely — so under the "masterplan generated from ADR front-matter" design (keystone §4) it would be **invisible to generation** — a concrete instance of the 8.8%-binding problem.
- **tensions:** (1) **No YAML front-matter** — only a Markdown header block (`**Status:** Accepted`). Inconsistent with every sibling in this chunk; breaks any front-matter-keyed index or masterplan generator (directly material to the authored-vs-generated founder question). (2) Tightly coupled to ADR-0122 (same doctrine, adjacent dates) — should be merged/cross-referenced; 0130 cites 0122 but not vice-versa. (3) Asserts "no further standalone evolution expected" for the KG semantic layer — a prediction, not a decision; mild over-claim. (4) D-09 "Ontology and KG are one product" — verify against the product roster in ADR-0128 (which lists `ontology` as a distinct product) for consistency.
- **hyperscaler_challenge:** ALIGNED. Eliminating a pointer-hop and giving the owning service direct authority over its schema (vs an external registry file) is correct domain-ownership design — matches how mature platforms avoid shared-mutable-config drift. The "no tombstone for internal infra, migrate consumers atomically" stance is clean. Verdict: keep.
- **ai_slop:** Low-moderate. Content is concrete (exact node/edge/invariant counts, consumer list, verification grep). The main defect is structural, not slop: missing front-matter. "~450 lines" and "size is a quality of the document, not a reason to fragment it" is mild editorializing.
- **refinement:** **Add YAML front-matter** (id, status, date, supersedes/superseded_by, related, related_specs) to match the corpus and make it generator-visible. Cross-link bidirectionally with ADR-0122. Merge 0122+0130(+0055) into one masterplan ontology-consolidation record.
- **consensus_needed:** no on the decision. (The missing-front-matter issue is a data point for the masterplan authored-vs-generated question, not a separate ruling.)

---

## Chunk notes for synthesis

**Cluster A — Ontology/KG consolidation (0122, 0130; +0055 cross-chunk).** Two adjacent, mutually-reinforcing ADRs enacting the single doctrine "Object-Graph→Ontology rename; Ontology owns its type system inline (Palantir pattern); KG-semantic registry retired." They should be **merged into one masterplan vocabulary+data entry**. Both are TRUE/KEEP. 0130's *missing front-matter* is the chunk's clearest concrete instance of the masterplan generated-from-ADR-front-matter risk (keystone §4): an authored, accepted decision that a front-matter generator would silently miss.

**Cluster B — Hyperscaler governance trio (0123, 0128, 0129).** A coherent, genuinely hyperscaler-grade governance stack: 0128 = the *content* (35 INV-* invariants), 0123 = the *claim gate* (may we say "mature"?), 0129 = the *honest-claims gate* (no active-claim-with-deferred-delivery). All KEEP, all strong-ALIGNED on the hyperscaler lens, all **prime masterplan backfill** — these are exactly the "true decisions every product must honor" the founder wants captured. Two cleanups: (a) ADR-0128's 11-product list embeds the retired `foundry` product → remap to intelligence/governance; (b) ADR-0123 names the retired `oya vcs`/HG-VCS layer as "forward authority" → must point at post-0363 cloud-ci/oya-ci Tide.

**Cluster C — The forge/VCS reversal (0121, 0124).** Both are the *pre-reversal* losing side of major fault-lines and both should ARCHIVE:
- **0121** (kubeadm/Istio onprem, explicitly rejecting Talos) → superseded by **ADR-0375** (Talos). Clean front-matter (`superseded_by:[ADR-0375]`).
- **0124** (bespoke GitHub-Actions webhook merge-queue on Foundry VCS kernels) → de-facto superseded by **ADR-0363→0513** (retire agentic-VCS; merge automation → Tide), but its front-matter still reads `superseded_by: none` — **stale-drift to flag** (keystone §6 pattern: trust the superseding ADR).

**Cross-chunk tension — the forge three-way (highest-signal item in this chunk).** ADR-0124 is the corpus's cleanest artifact of the GitHub-vs-Forgejo-vs-bespoke tension (keystone §5, fault-line #4): it is built entirely on **GitHub** (Actions, branch-protection REST, webhooks) and authored by `jason931225` — so it *agrees with the founder's GitHub migration directive* while *directly contradicting* source canon (ADR-0363/0374/0387: Forgejo canonical, GitHub mirror; ADR-0510: bespoke VCS is the destination, GitHub bootstrap-only). 0124 even rejects relying on GitHub's *native* merge queue — yet ADR-0363 rejects GitHub *as substrate*. Surfacing, not resolving, per instructions. **This needs a founder ruling.**

**Retired-vocabulary leakage (audit signal).** `foundry`-brand residue appears in 0121 (`axis-foundry` owner, "Foundry workloads"), 0123 ("Foundry/Oya VCS pipeline"), 0124 (`oya-foundry-vcs-*` crate names, ×8), 0128 (`foundry` as a product). Per keystone §2 + the MFL-0002/0003 brand-residue lanes, treat all as retired-vocab; the live posture is cloud-intelligence (consumer AI) + governance (CI/gates).

**Masterplan-binding status across chunk:** 0/7 are cited in MASTERPLAN.md. The KEEP-and-backfill set (0122, 0123, 0128, 0129, 0130) carries proper planning-relevant decisions but is not bound — consistent with the keystone §4 finding of only 8.8% ADR binding. ADR-0130 additionally cannot be generator-bound at all (no front-matter). The two ARCHIVE ADRs (0121, 0124) correctly should NOT enter the masterplan; only their successors (0375; 0363/0513) should.

**Dependency-on-Proposed hazard:** ADR-0124 (Accepted) was built on ADR-0111 (still `Proposed`). Worth flagging as a maturity-discipline gap independent of the supersession.
