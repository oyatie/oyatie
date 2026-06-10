# ADR Audit — source-42

- **side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **chunk:** 42 (slice rows 288–294 of `ls -1 docs/decisions/ADR-*.md | sort`)
- **range:** ADR-0351 → ADR-0357
- **ADRs reviewed:** 7 — ADR-0351, ADR-0352, ADR-0353, ADR-0354, ADR-0355, ADR-0356, ADR-0357
- **auditor posture:** ADRs = immutable SSOT; masterplan GENERATED from the live ADR log; a decision not represented as a LIVE ADR is "not needed." Keystone map (`_map/canonical-posture-and-supersession-map.md`) is the shared baseline for TRUE/RETIRED/SUPERSEDED. Read each file fully (0353/0355/0356 tails are CI-lane/appendix boilerplate; decision atoms captured from D-sections).

---

### ADR-0351 — Cell-rebalancer + cell-lifecycle microservices (amends ADR-0333)

- **decision_atom:** Carve two single-concern substrate microservices out of the ADR-0333 cell-absorption — `cell-rebalancer` (cross-cell tenant migration as a durable, abortable, audit-emitting workflow) and `cell-lifecycle` (the logical cell-entity state machine Registered→Activated→Promoted→Drained→Decommissioned) — leaving identity/routing/audit/placement/telemetry absorbed as ADR-0333 decided.
- **domain:** orchestration-scheduling (secondary: tenancy)
- **current_status:** Accepted (2026-05-21).
- **disposition:** KEEP (with a light AMEND flag — see truth_flag).
- **proposed_resolution:** NA (Accepted, not Proposed).
- **governing:** n/a.
- **truth_flag:** PARTIAL. Core decision is TRUE and coherent with ADR-0333/0348. But it leaks retired/contested vocab: "Tier 0..4" cell promotion (live cellular-tier concept, OK — distinct from retired tenant tier-system, the ADR itself flags this via `[[no-capability-tiers-2026-05-20]]`), and Cedar fragment principal `oyatie.foundry.cell-orchestrator` / R-11 "Foundry agent boundary" — "Foundry" the brand is RETIRED (ADR-0335/0347). The `oyatie.foundry.*` *principal namespace* survives per ADR-0352 §D-13 and ADR-0335, so this is naming-residue not a dead decision, but the prose framing ("Foundry agents", "Foundry recipe" in A-5) is stale. D-2.3 persists state in "PostgreSQL per-tenancy shard" — consistent with source canon (Postgres+pgcat), no Valkey/Redis leakage here.
- **in_masterplan:** PARTIAL — the cell-rebalancer + cell-lifecycle services ARE named in ADR-0352 §D-8 (the from-scratch handoff), so they have downstream representation, but as two of "77→79" µservices not yet a masterplan-bound atom.
- **tensions:** (1) µservice-count claim "77→79" is a hard-coded number that will drift (ADR-0352 enumerates a different, larger service list). (2) Amends ADR-0348's "within tenancy + observability" placement — depends on ADR-0348 amendment block actually landing (acceptance criteria show it UNCHECKED). (3) LINUX framekernel "we are the host" posture (LINUX ADR-0018/0025) is orthogonal but a cell-lifecycle-as-µservice assumes the Talos+K8s substrate, not the owned-kernel substrate — cross-side fault-line #3.
- **hyperscaler_challenge:** ALIGNED. The ADR's own C-4 precedent (AWS Lambda Placement Service + Cell Lifecycle Manager, Spanner tablet-movement vs location, Cassandra nodetool move/decommission) is exactly how hyperscalers split location/identity from movement-workflow from lifecycle-state-machine. Google/AWS would make this decision. Implication: KEEP; only amend the "Foundry" naming residue.
- **ai_slop:** Low. Verbose (12 rationale bullets R-1…R-12, 7 alternatives) but each is substantive and the hyperscaler mapping is real. The single-concern justification is sound, not padding.
- **refinement:** Strip "Foundry"-brand prose (keep `oyatie.foundry.*` principal id as the operational namespace per ADR-0335); replace hard count "77→79" with a generated-count reference; land the ADR-0333/0348 amendment blocks (open acceptance items).
- **consensus_needed:** None contested. (Minor founder question: is cell-lifecycle/cell-rebalancer in-scope for the FD-001 first-deliverable, or a later wave? ADR-0352 sequences them in Phase 0/1 "cloud cell context / cell-pattern support" — confirm.)

---

### ADR-0352 — Oyatie from-scratch architecture handoff

- **decision_atom:** Maintain a single self-contained greenfield architecture handoff that an implementation agent can build from without following pointers — encoding doctrine, build sequence, service-boundary reconciliations, stack, contracts, tenancy/policy/cell/intelligence models, DR/audit/FinOps, and a verbatim implementation prompt.
- **domain:** docs-ssot-masterplan (cross-cutting: governance-process). This ADR is itself a masterplan-shaped artifact.
- **current_status:** Proposed (2026-05-22).
- **disposition:** AMEND (sound intent, badly stale stack section) — borderline ARCHIVE-and-regenerate.
- **proposed_resolution:** RATIFY-WITH-MANDATORY-AMENDMENT. Why: the handoff's *intent* (one inline, pointer-free greenfield contract that refuses retired boundaries) is valuable and uniquely fills the "implementer-facing single doc" gap — but as written it is the single largest concentration of RETIRED-VOCAB drift in this chunk and would actively re-introduce dead decisions if ratified verbatim. Do not DROP (the artifact class is wanted); do not RATIFY as-is.
- **governing:** retired-vocab governing ADRs that this file contradicts: ADR-0377-kafka-to-pulsar (Kafka), ADR-0336 (Redis→Valkey — note this file does correctly say "Redis is not canonical / Valkey is canonical" in §D-4, but §D-11 still says "Eventing uses **Kafka** with outbox"), ADR-0375 (Talos+CAPI+ArgoCD supersedes kubeadm/containerd onprem — §D-11 still says "kubeadm plus containerd"), ADR-0511 (Argo Workflows is the CI destination — §D-11 names "**Jenkins LTS** … ArgoCD" with no Argo Workflows), ADR-0379 (Kubewarden default admission — §D-7/§D-11 name **Kyverno** for admission), ADR-0363/0510 (forge: this file embeds bespoke "Oya VCS / oya git" §D-14 as canonical — ADR-0363 retired exactly that agentic-VCS in favour of plain git + Forgejo; founder directive = GitHub).
- **truth_flag:** STALE (intent TRUE; concrete bindings WRONG against current canon on at least five axes: eventing=Kafka, CI=Jenkins-as-destination, onprem=kubeadm/containerd, admission=Kyverno, VCS=bespoke-oya-VCS). Several decisions ARE current and correct (Cedar v4.2 app-authz, Zitadel/OIDC, OpenBao, Milvus, SeaweedFS, Iceberg OLAP write-path, UUIDv7, HLC, flat single-concern µservices, oyatie-is-a-tenant, library-first network-opt-in, distroless/multi-arch, SLSA/cosign).
- **in_masterplan:** PARTIAL — this file is *adjacent to* the masterplan and overlaps it heavily; under the keystone §4 OPEN founder question (authored-as-SSOT vs generated-from-ADRs), a self-contained inline handoff is in direct tension with the "masterplan = generated projection, ADRs = SSOT" design (`planning-ssot-consolidation.md`). It duplicates substance that should be generated.
- **tensions:** MAJOR. (1) Architectural tension with the generated-masterplan doctrine: a hand-maintained inline mega-doc is exactly the duplication `planning-ssot-consolidation.md` wants to eliminate — its own §Consequences/Negative admits "if upstream doctrine changes, this ADR must be amended directly." (2) Retired-vocab tensions (above) make it a drift magnet. (3) §D-2 "first deliverable is **Tenant RBAC view plus Tenant RBAC view**" is a literal duplicated phrase (the Implementation-Agent-Prompt copy says "Tenant RBAC view plus **SMB Generic**") — internal inconsistency about what FD-001 actually is. (4) "M0–M3/MVP/preview" are retired (GLOSSARY) — this file is careful to say "not MVP, not preview," good, but uses "Wave 15-ZD"-style wave naming elsewhere in the corpus that this file does not reconcile.
- **hyperscaler_challenge:** QUESTIONABLE *as written*. The shape (flat services, clean-arch layers, cell architecture, library-first, contracts-before-handlers, SLSA) is exactly what a hyperscaler greenfield would do — ALIGNED in doctrine. But the *stack choices it freezes* are partly behind the org's own newer ADRs (Jenkins/Kafka/Kyverno/kubeadm), so a hyperscaler reviewing THIS document would flag it as out-of-date relative to the org's own later decisions. Implication: AMEND (regenerate the stack section from the current canonical-posture table) rather than archive — the doctrine is keep-worthy.
- **ai_slop:** MEDIUM-HIGH risk surface. 1120 lines, much of it list-restating other ADRs; the duplicated "Tenant RBAC view plus Tenant RBAC view" and the Kafka-vs-its-own-Valkey-paragraph contradiction are slop tells. Not fabricated — but it is the kind of self-contained mega-doc that rots fastest and is hardest to keep true. The keystone's "generated-from-ADRs" design exists partly to kill documents like this.
- **refinement:** Either (a) AMEND: cut §D-11 stack to a pointer/generated-table and fix the five retired-vocab bindings, fix the FD-001 phrase, OR (b) re-cast as a *generated* artifact (rendered from ADR front-matter) rather than authored — which is the keystone-preferred direction.
- **consensus_needed:** **Founder question (contested):** "Do we keep a hand-authored self-contained from-scratch handoff ADR at all, or is the from-scratch contract henceforth GENERATED from the ADR log + canonical-posture table (so ADR-0352 is archived as a one-time bootstrap)?" This is the §4 authored-vs-generated question made concrete — ADR-0352 is the strongest single instance of the authored-SSOT pattern.

---

### ADR-0353 — Amendment: Library-First / Network-Opt-In Clarification (Policy-Engine)

- **decision_atom:** Amend ADR-0246 so the Policy-Engine substrate's per-call Cedar *evaluation* is library-first/in-process by default (`oya-shared-policy-engine-client-*` embedding `cedar-policy` v4.2), with the `microservices/policy-engine/` µservice retained only for cross-cutting control-plane concerns (fragment authoring/signing-chain/hot-reload/coverage-audit/cross-cell fan-out/untrusted-caller mediation); network evaluation is per-tenant + per-SecretReference opt-in.
- **domain:** authz-policy (secondary: api-contracts / architecture)
- **current_status:** Proposed (2026-05-20).
- **disposition:** KEEP (ratify). One of three symmetric "library-first" amendments (0353 Policy-Engine / 0355 Intelligence / 0356 Ontology-read).
- **proposed_resolution:** RATIFY. Why: it forecloses re-introduction of the universal-mediator anti-pattern that ADR-0145 retired, on the highest-frequency axis (Cedar is consulted on every state-changing call per ADR-0243). Dropping it would leave ADR-0246 §D-3/§D-5 readable as mandating a per-call gRPC hop — the exact regret ADR-0145 closed.
- **governing:** n/a (amends, does not supersede, ADR-0246).
- **truth_flag:** TRUE. Hyperscaler precedent is correctly stated (AWS IAM caller-side eval, AWS Verified Permissions cached/batch client, OPA embedded, Cedar v4.2 embedded). Self-consistent with 0355/0356. Notes a dependency `ADR-NNNN-library-first-credential-sidecar` (Slice-2 sidecar) that is a **dangling/unallocated ADR id** — a real provenance gap.
- **in_masterplan:** PARTIAL — depends on ADR-0246 being in masterplan; the library-first delivery-shape doctrine is canon-aligned (keystone §3 Policy/authz row: Cedar universal gate) but the masterplan does not yet bind the delivery-topology nuance.
- **tensions:** (1) Dangling `ADR-NNNN-library-first-credential-sidecar` reference (also in 0356) — must be allocated a real number or the dependency is unverifiable. (2) Eight-item "promote to BLOCKER when…" checklist is an enforcement-gate, not a decision — its presence is fine but it ties ratification to unfinished scaffolding. (3) Three near-identical amendments (0353/0355/0356) could be MERGE candidates into one "library-first substrate delivery doctrine" ADR — see Chunk notes.
- **hyperscaler_challenge:** ALIGNED. AWS/Google would absolutely NOT put a per-call RPC mediator on every authorization decision; caller-side/embedded policy eval is the hyperscaler norm. This amendment moves *toward* the hyperscaler shape. Implication: KEEP.
- **ai_slop:** LOW-MEDIUM. Heavily parallel to 0355/0356 (same six-failure-mode list, same Hamilton static-stability framing, same alternatives) — the repetition across three ADRs is the main slop signal, arguing for MERGE rather than three separate ~1600-line files.
- **refinement:** Allocate the real number for the credential-sidecar ADR; consider merging the three symmetric amendments.
- **consensus_needed:** "Should the three library-first amendments (Policy-Engine/Intelligence/Ontology-read) be one consolidated doctrine ADR instead of three?" (contested only on form, not substance).

---

### ADR-0354 — Amendment: HTTP/3 Fallback, Strict TLS 1.3, ECH, PQC Hybrid

- **decision_atom:** Amend ADR-0253 to make binding the HTTP/3→HTTP/2→HTTP/1.1 fallback chain with timeout budgets, a closed-list strict TLS 1.3 profile, mandatory Encrypted Client Hello (RFC 9460) on external endpoints, and PQC hybrid (X25519MLKEM768 KEM + ed25519+ml_dsa_65 composite signature) with a per-µservice protocol applicability table and four CI lanes.
- **domain:** networking-mesh (secondary: crypto-keymgmt / security-supplychain)
- **current_status:** Proposed (2026-05-20).
- **disposition:** KEEP (ratify) with one AMEND fix.
- **proposed_resolution:** RATIFY. Why: it is concrete, standards-grounded (FIPS 203/204, RFC 9114/9460/9000, draft-ietf-lamps-pq-composite-sigs), and operationalizes ADR-0253's advisory TLS/protocol language into enforceable specs. Harvest-now-decrypt-later justification for PQC is correct and the build-ahead-of-certification doctrine (ADR-0250) supports day-one PQC.
- **governing:** n/a (amends ADR-0253).
- **truth_flag:** PARTIAL. Crypto/protocol substance is TRUE and current. BUT the §B-5 per-µservice protocol table still lists a `foundry` row ("Hermes pipeline; internal") — **both "foundry" the µservice AND "Hermes" are RETIRED** (ADR-0335 foundry→intelligence; GLOSSARY L241/L1042 Hermes retired). That row is stale-vocab and should read `intelligence` (or be dropped). The table also lists `governance` and `policy-engine` as separate rows — consistent with 0335/0347 (foundry split into intelligence + governance), so the framework is right; only the literal `foundry` row is wrong.
- **in_masterplan:** PARTIAL — TLS/PQC posture aligns with keystone "License/security" posture but is below the masterplan's altitude; the binding parameter tables would be spec-bound, not masterplan-bound.
- **tensions:** (1) Retired `foundry`/`Hermes` row (above). (2) ECH cites "RFC 9460" for Encrypted Client Hello — ECH is actually `draft-ietf-tls-esni` (the ADR's own §D-3 correctly says "draft-ietf-tls-esni-18 (RFC 9460 when published)"); RFC 9460 is SVCB/HTTPS records. Minor technical-citation imprecision, not load-bearing. (3) References ADR-0121 (onprem kubeadm/containerd/istio) in `related` — that stack is superseded by ADR-0375 (Talos); the network amendment should reference the Talos/Cilium substrate, not the retired istio stack.
- **hyperscaler_challenge:** ALIGNED. Cloudflare/Google/AWS are exactly the deployers of HTTP/3 + ECH + X25519MLKEM768 hybrid (this is current 2024–2026 industry frontier). A hyperscaler would make this decision. Implication: KEEP; amend the foundry/Hermes row + istio reference.
- **ai_slop:** LOW. Dense but genuinely technical; code-points, OIDs, and SLO thresholds are specific and checkable. Not padding.
- **refinement:** Replace §B-5 `foundry`/"Hermes" row with `intelligence`; swap ADR-0121 reference for ADR-0375; tighten the RFC-9460-vs-ECH-draft citation.
- **consensus_needed:** None substantive.

---

### ADR-0355 — Amendment: Library-First / Network-Opt-In Clarification (Intelligence)

- **decision_atom:** Amend ADR-0255 so the Intelligence substrate's per-call LLM dispatch is library-first/in-process by default (`oya-shared-intelligence-client-*` with per-provider adapters calling providers directly over HTTPS), with `microservices/intelligence/` retained only for cross-cutting state (shared credential/rate budgets, cost/observability rollup, adapter/tool registries, Layer-B brand surfaces); network coordination is per-SecretReference + per-Cedar opt-in.
- **domain:** intelligence-ai (secondary: api-contracts / agentic-platform)
- **current_status:** Proposed (2026-05-20).
- **disposition:** KEEP (ratify) with one AMEND fix.
- **proposed_resolution:** RATIFY. Why: closes F-ANTI-1 (the #1 12-month regret per the cited PR#143 review) before any code is written; aligns Intelligence with the AWS-SDK/Bedrock, Anthropic-SDK, OpenAI-SDK in-process-client reality. This is the keystone-canonical Intelligence posture (two-layer substrate, absorbs retired Foundry).
- **governing:** n/a (amends ADR-0255).
- **truth_flag:** PARTIAL. Doctrine TRUE and hyperscaler-correct. BUT §D-4 specifies the shared per-credential budget store as "**Redis/KeyDB** per cell" — **Redis is RETIRED → Valkey** (ADR-0336). That is direct retired-vocab leakage in a Proposed ADR and must be amended to Valkey. Also front-matter owner `axis-foundry` and §References ADR-0247 "`oyatie.foundry.*` workflows" — the principal namespace survives (OK) but the `axis-foundry` owner label is brand-residue.
- **in_masterplan:** PARTIAL — Intelligence two-layer substrate is keystone-canonical (§3 Intelligence row); the library-first delivery shape is the correct refinement but not yet masterplan-bound.
- **tensions:** (1) Redis/KeyDB vs Valkey (above) — concrete retired-tech bug. (2) `axis-foundry` owner + Foundry references — brand residue (ADR-0335/0347). (3) MERGE candidate with 0353/0356 (same doctrine, triplicated). (4) References "ADR-0296 §D-2" credential-sidecar plus the dangling `ADR-NNNN-library-first-credential-sidecar` — two different ids for the sidecar concept; provenance must be reconciled.
- **hyperscaler_challenge:** ALIGNED. No hyperscaler routes every LLM call through an in-VPC gateway µservice; SDK-in-process + direct-to-provider is the universal pattern (AWS/Azure/Anthropic/OpenAI/Google all cited correctly). Implication: KEEP; fix Redis→Valkey and foundry residue.
- **ai_slop:** LOW-MEDIUM. Strong, well-cited; the ~20-crate provider-adapter enumeration is plausibly real scope but the triplication with 0353/0356 is the main slop tell.
- **refinement:** Redis/KeyDB → Valkey; drop `axis-foundry` owner / Foundry-brand prose (keep `oyatie.foundry.*` principal); reconcile ADR-0296 vs ADR-NNNN sidecar id; consider MERGE.
- **consensus_needed:** Same form question as 0353 (merge the three?).

---

### ADR-0356 — Amendment: Library-First Ontology Read-Path Clarification

- **decision_atom:** Amend ADR-0257 so cross-µservice Ontology *reads* are library-first/in-process by default via a per-tenant per-Object-Type CRDT projection (`oya-shared-ontology-read-*`) kept fresh by a Kafka CDC stream, while ALL writes/schema-evolution/deprecation-handshake/tombstones remain mediated by `microservices/ontology/`; network reads (cross-cell, cross-tenant, untrusted-tier, `network_only` tenants) are Cedar/attribute opt-in.
- **domain:** workflow-ontology (secondary: data-storage / authz-policy)
- **current_status:** Proposed (2026-05-20).
- **disposition:** KEEP (ratify) with AMEND fixes; closes a real gap (no standalone ontology-read-path doctrine existed — landed against nearest ADR-0257).
- **proposed_resolution:** RATIFY. Why: completes the library-first symmetry (3-of-3) and restores ADR-0141's correct read/write-split intent inside ADR-0145's "no universal mediator / Ontology is SUBSTRATE not GATEWAY" frame. The read-vs-write asymmetry (reads library, writes µservice) is intentional and hyperscaler-correct.
- **governing:** n/a (amends ADR-0257; explicitly recovers superseded ADR-0141's intent — note ADR-0141 is in keystone §1.1 as Superseded by ADR-0145).
- **truth_flag:** PARTIAL/TRUE. Doctrine TRUE; CRDT/CDC delivery-shape is well-precedented (Palantir Foundry caller-side projections, DynamoDB DAX, S3 Express One Zone client, Spanner staleness reads, Stripe.js cache). BUT the **eventing substrate is named "Kafka" throughout** (CDC topics `ontology.entity.{cell}.{tenant}`) — per ADR-0377-kafka-to-pulsar the canonical eventing is **Pulsar 4.x + Oxia (KoP wire-compat)**, so "Kafka" is retired-vocab; the KoP wire-compat means the topic shape survives but the substrate name is stale. Also references the dangling `ADR-NNNN-library-first-credential-sidecar`.
- **in_masterplan:** PARTIAL — Ontology-as-read-substrate (ADR-0145 Invariant 3) is canon; this delivery-shape refinement is correct but not yet masterplan-bound.
- **tensions:** (1) Kafka vs Pulsar/Oxia (ADR-0377) — retired-substrate naming. (2) Builds on ADR-0257 which is itself Proposed (amendment-on-a-proposal chain) and on the also-Proposed ADR-0246-amendment (ADR-0353) — a stack of three Proposed ADRs none yet Accepted; ratification should be coordinated as a bundle. (3) Dangling sidecar id (shared with 0353/0355). (4) MERGE candidate.
- **hyperscaler_challenge:** ALIGNED. Caller-side materialized read projection + CDC invalidation, central write authority — this is exactly Palantir/DynamoDB-DAX/Spanner shape. A hyperscaler would make this read/write-split decision. Implication: KEEP; rename Kafka→Pulsar(+Oxia/KoP).
- **ai_slop:** LOW-MEDIUM. Technically rich (CRDT LWW-per-(entity,revision), per-property merge strategy, freshness floor); the triplication with 0353/0355 and the long crate enumeration are the slop tells.
- **refinement:** Kafka → Pulsar 4.x + Oxia (KoP wire-compat) per ADR-0377; allocate the real sidecar ADR id; coordinate ratification with ADR-0257/0353; consider MERGE.
- **consensus_needed:** "Ratify the 0257 + 0353 + 0355 + 0356 (+ sidecar) library-first bundle together, or sequence?" Plus the merge-the-three question.

---

### ADR-0357 — Vertical-slice monorepo nesting

- **decision_atom:** Move each service's crates from flat `crates/oya-*` into co-located `microservices/<ms>/crates/oya-<service>-<layer>` (with shared/`*-kernel` libs under top-level `libs/`), flipping the `architecture-boundaries` gate to enforce nested code paths, executed as a single mechanical git-mv migration after wave-3 worktree consolidation lands green; package names and dependency graph unchanged.
- **domain:** ci-cd-build (secondary: forge-vcs / repo-structure)
- **current_status:** Proposed (2026-05-25); prerequisites note dated 2026-05-29.
- **disposition:** KEEP (ratify), pending the stated prerequisite.
- **proposed_resolution:** RATIFY. Why: resolves a real internal conflict — ADR-0131's stated intent was `microservices/<ms>/` code roots, but the `architecture-boundaries` gate currently enforces flat `crates/` and has rejected µservice-local paths. The migration closes that intent-vs-enforcement gap and matches Google google3/Meta Buck2 vertical-slice locality. Low-risk (mechanical, package names stable).
- **governing:** n/a (amends nothing formally; resolves ADR-0131 conflict — should carry an `amends: ADR-0131` once ratified).
- **truth_flag:** TRUE (clean, no retired-vocab leakage). One internal number-drift: §Context says "546 code crates" / "188 in-flight" while §Consequences says "~734 crates" migrate — the crate-count figures are inconsistent and will be stale at execution time.
- **in_masterplan:** NA — repo-layout mechanics, below masterplan altitude (but the `architecture-boundaries` gate behaviour is a governance artifact).
- **tensions:** (1) Crate-count inconsistency 546/734 (above). (2) Sequencing dependency on "PR #363 merges to dev" — a transient state captured in an ADR (couples an immutable decision to a mutable PR status). (3) References ADR-0349/ADR-0360/ADR-0111/ADR-0105/ADR-0106/ADR-0131 — the build-cache reference (ADR-0349 sccache→SeaweedFS) sits in the CI churn chain (keystone §1.3) now superseded toward Buck2 (ADR-0392) + Argo Workflows (ADR-0511); the nesting decision is build-tool-agnostic so this is not a contradiction, but the ADR predates the Buck2 cutover and should confirm nesting is compatible with Buck2 target paths.
- **hyperscaler_challenge:** ALIGNED. The ADR's own justification (google3/Bazel, Buck2 vertical-slice locality, path-evident ownership, one-version policy) is precisely hyperscaler monorepo practice. Google/Meta would make this decision. Implication: KEEP.
- **ai_slop:** NONE. Tight, concrete, honest about cost. Model of a good short ADR.
- **refinement:** Replace hard crate counts with a generated figure; remove the transient "PR #363" coupling (or move it to the migration plan/task, not the ADR body); add `amends: ADR-0131`; confirm Buck2-target-path compatibility.
- **consensus_needed:** None.

---

## Chunk notes

**Shape of the chunk.** Seven ADRs in two clusters: one **cellular/topology** decision (0351), one **mega-handoff** (0352), three **library-first symmetry amendments** (0353/0355/0356, the F-ANTI-1/2/3 set), one **network-crypto amendment** (0354), and one **monorepo-layout** decision (0357). Six of seven are `Proposed`; only 0351 is `Accepted`.

**Net disposition.** KEEP/RATIFY six (0351 Accepted; 0353/0354/0355/0356/0357 ratify-with-minor-amend). AMEND-heavily-or-regenerate one (0352). No ARCHIVE/SUPERSEDE candidates in this range — none is superseded by a governing ADR.

**Dominant defect = retired-vocab leakage in Proposed ADRs.** Despite the keystone retired-vocab map, this chunk leaks dead terms in five places, all in still-Proposed (editable) ADRs:
- ADR-0352 §D-11: "Eventing uses **Kafka**" (→ Pulsar 4.x + Oxia, ADR-0377); "**kubeadm plus containerd**" onprem (→ Talos, ADR-0375); "**Jenkins LTS … ArgoCD**" as CI destination with no Argo Workflows (→ ADR-0511); "**Kyverno**" admission (→ Kubewarden, ADR-0379); bespoke "**Oya VCS / oya git**" as canonical (→ retired by ADR-0363).
- ADR-0354 §B-5: a literal `foundry` µservice row labeled "**Hermes** pipeline" (→ intelligence; both terms retired, ADR-0335/0347, GLOSSARY).
- ADR-0355 §D-4: per-credential budget store "**Redis/KeyDB** per cell" (→ Valkey, ADR-0336); `axis-foundry` owner.
- ADR-0356: CDC over "**Kafka**" topics (→ Pulsar/Oxia KoP, ADR-0377).
- ADR-0351: `oyatie.foundry.cell-orchestrator` / "Foundry agent" prose (principal id survives; brand prose retired).
Recommendation: a single mechanical retired-vocab pass over these five before any are ratified.

**Two systemic provenance gaps to surface to the founder:**
1. **Dangling ADR id.** `ADR-NNNN-library-first-credential-sidecar` (the "Slice-2 sidecar key-holder") is a load-bearing dependency of 0353, 0355, and 0356 but has no allocated number; 0355 separately cites "ADR-0296 §D-2" for the same sidecar concept. The sidecar ADR must be located/allocated or the three amendments rest on an unverifiable dependency.
2. **Stacked Proposed-on-Proposed amendments.** 0356 amends 0257 (Proposed) and depends on 0353 (amends 0246) and 0355 (amends 0255), none Accepted. The library-first doctrine is sound but nothing in the chain is ratified — it should be ratified as one coordinated bundle.

**MERGE recommendation (form, not substance).** ADR-0353 / 0355 / 0356 are three ~1.2–1.7k-line files repeating the identical doctrine (ADR-0145 no-universal-mediator), the identical six-failure-mode list, the identical Hamilton static-stability framing, and the identical three-alternatives structure, differing only in the substrate (Policy-Engine / Intelligence / Ontology-read). Strong candidate to consolidate into one **"Library-first substrate delivery doctrine"** ADR with three per-substrate sections — reduces drift surface and slop. Founder question raised under each.

**The 0352 founder question is the keystone §4 question in concrete form.** ADR-0352 is the single biggest instance of the *authored-self-contained-SSOT* pattern. Whether it is KEPT (authored handoff) or REGENERATED/ARCHIVED (masterplan generated from ADRs) is exactly the unresolved "authored-vs-generated masterplan" decision. Under the brief's stated posture ("masterplan GENERATED from the ADR log; ADRs = immutable SSOT"), ADR-0352 trends toward **archive-as-bootstrap + regenerate** — but this is explicitly the OPEN founder question, so flagged for decision, not resolved here.

**Hyperscaler verdict for the chunk.** Every *doctrinal* choice here is hyperscaler-aligned (cell split per AWS/Spanner/Cassandra; library-first/embedded eval per AWS-IAM/Bedrock-SDK/Palantir/DAX; HTTP3+ECH+PQC per Cloudflare/Google; vertical-slice monorepo per google3/Buck2). The only hyperscaler-*questionable* item is ADR-0352, and only because its frozen stack section lags the org's own newer ADRs — i.e., a staleness problem, not a wrong-direction problem.
