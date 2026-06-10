# ADR Audit — source-28

- **Side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **Chunk:** source-28 (slice lines 190–196 of `ls -1 docs/decisions/ADR-*.md | sort`)
- **Range:** ADR-0215 → ADR-0221
- **ADRs reviewed:** 7 (0215, 0216, 0217, 0218, 0219, 0220, 0221) — all `Accepted`
- **Cohort:** the PR #143 "#E substrate doctrines" cluster (all dated 2026-05-18, owner `council-architecture`, lane `governance / substrate-doctrine`, all sourced from `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json`). ADR-0221 is the close-out ADR that codifies and supersedes that very checkpoint.
- **Auditor disposition summary:** 6 KEEP, 1 KEEP+AMEND (0217 filename/title drift); 0 ARCHIVE; 0 unaccounted Proposals (none are Proposed). Two cohort-wide retired-vocab leaks (`Foundry` brand, `capability tier`) flagged as AMEND-on-touch but not disposition-changing.

---

### ADR-0215 — Multi-Context Platform Architecture

- **decision_atom:** One human principal holds many independently-scoped data contexts (work-per-employer / personal / healthcare-patient / healthcare-provider / education-student / government-citizen), each with its own storage, Cedar policy, audit-chain, residency, retention, and tenant admin boundary; cross-context visibility happens only through explicit consent-graph grants (ADR-0214), never an implicit join.
- **domain:** tenancy, identity-authn (cross-cutting: the multi-context principal is an identity primitive that drives tenancy/isolation boundaries).
- **current_status:** Accepted.
- **disposition:** KEEP. Current, internally coherent, non-conflicting; explicitly layers above the IdP (Alt-4 rejection) so it does not collide with the Zitadel/OIDC posture (map §3 identity). Reinforced by 0218 (tenant-admin can't see unrelated contexts) and 0220 (context-scoped AI memory).
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL. Carries strong planning_impact (mandatory context-id on every user-data contract, CI gate to reject context-optional write paths, 3-phase rollout) but the file carries no explicit `planning_impact`/`masterplan_ref` front-matter field — binding is prose-only, consistent with the repo's 8.8%-bound reality (map §4).
- **tensions:** (1) References ADR-0239 as "Foundry internal scope; consumer AI belongs to Intelligence" — the `Foundry` brand is RETIRED (ADR-0335/0347, map §2); the citation's *substance* survives (consumer AI = intelligence) but the brand label is stale-on-touch. (2) Asserts "Cedar policies … remain context-scoped" — aligned with the Cedar universal-gate posture (ADR-0243/0246), no conflict. (3) "cell isolation" / context-scoped partitions lean on `cell` — fine as a deployment *pattern* (ADR-0333) but watch for cell-as-service drift.
- **hyperscaler_challenge:** ALIGNED. Google (Managed vs personal account), Apple (Managed Apple ID vs personal), Microsoft (work vs personal M365) all ship exactly this hard work/personal separation; the ADR cites them. A hyperscaler would make this decision. Argues neither amend nor archive on substance.
- **ai_slop:** None material. Well-structured, real industry anchors, genuine alternatives. The "Bominal-ADR-0215" compatibility note is a legitimate disambiguation artifact, not slop.
- **refinement:** On next touch, respell the ADR-0239 reference to drop the live "Foundry" brand (use intelligence/governance per 0335).
- **consensus_needed:** None. (Minor: does multi-context principal vs tenant-class (0329) need an explicit precedence note when a healthcare-provider context is also a paid tenant? Worth a one-line cross-ref but not blocking.)

---

### ADR-0216 — Open Integration and Migration-Out Policy

- **decision_atom:** Every customer-facing microservice that owns portable business data must ship first-party importers/exporters for its top-3 incumbents (or a neutral standards archive), canonical OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 contracts, Wasmtime-sandboxed plugin seams, and open-standard compatibility — "contact support for export" is forbidden; anti-lock-in openness is the moat.
- **domain:** api-contracts, product-ux (cross-cutting: contract/integration doctrine that is also a product trust promise; secondary touch on security-supplychain via plugin governance).
- **current_status:** Accepted.
- **disposition:** KEEP. Current, correct, non-conflicting. Pinned versions (OpenAPI 3.2.0, AsyncAPI 3.1.0) are consistent with ADR-0221's M-01 lesson (3.2.0 is the verified-correct pin that replaced the hallucinated "3.3"); Wasmtime is the canonical WASM runtime (map §3 isolation, ADR-0200).
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL. Heavy planning_impact (PRD `Import/export adapter list` section mandatory; CI gate fails customer-facing µsvcs missing importer/exporter declarations; 4-phase rollout) but again prose-only binding, no `masterplan_ref` field.
- **tensions:** (1) References ADR-0213 (Plugin App Store / Developer SDK) — not in my slice; the plugin-app-store-vs-marketplace taxonomy was itself an M-03 mistake in 0221, so verify 0213 uses the canonical glossary term. (2) Version pins (OpenAPI 3.2.0 / AsyncAPI 3.1.0) must carry source-URL citation per 0221 Gate-3 `version-pin-source-cited` — this ADR states the versions but does not cite an upstream source URL inline, a (advisory-grade) self-violation of the cohort's own gate.
- **hyperscaler_challenge:** ALIGNED. AWS/Stripe/Shopify/Google all monetize ecosystems while shipping export + import paths; "trust through openness lowers adoption risk" is mainstream hyperscaler doctrine. They would make this decision. No amend/archive pressure on substance.
- **ai_slop:** None material. The "top three competitors" framing is slightly formulaic but operationalized concretely (adapter-list PRD section, audit events, lossy-field disclosure).
- **refinement:** Add inline source-URL citations for the 3.2.0 / 3.1.0 pins to satisfy the cohort's own version-pin gate (0221 Gate-3). Confirm ADR-0213 plugin/marketplace term is canonical.
- **consensus_needed:** None substantive. (Open: "top three competitors" is unbounded maintenance debt — does the founder want a fixed adapter-count floor or a market-share threshold? Minor.)

---

### ADR-0217 — Service Packaging Rollout Order  (filename: `vertical-slice-rollout-order`)

- **decision_atom:** Plan the full microservice surface up front, but promote production-GA claims only through tenancy/RBAC service-packaging evidence (depth-before-breadth) — order 1 = Tenancy+RBAC core set (core, messenger, mail, community, infra, Ops/Control-Center, intelligence, Workflow, Ontology, canonical base, Korea pack) at full production depth; order 2 = sector distribution bundles that *compose* the same flat services, never product/module forks.
- **domain:** governance-process, docs-ssot-masterplan (cross-cutting: this is the canonical sequencing/rollout doctrine that any generated masterplan ordering must inherit).
- **current_status:** Accepted.
- **disposition:** KEEP + **AMEND**. The decision is sound and current, but front-matter title is "Service Packaging Rollout Order" while the **filename is `ADR-0217-vertical-slice-rollout-order.md`** — a title/filename drift exactly of the class ADR-0221 M-12 (stale refs after rename) warns about. Amend to reconcile (the MASTERPLAN front-matter, map §4, says "development order is vertical-slice," so the *filename* may be the intended canonical term and the title drifted, or vice-versa). Amend = naming/ref fix only; substance KEEP.
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a.
- **truth_flag:** PARTIAL. Decision substance TRUE; the title-vs-filename mismatch is a STALE-naming artifact. Also contains retired-vocab risk: forbids "product/module groupings (Connect, enterprise, healthcare)" — which is correct and *consistent with* ADR-0362 (flat-only catalog, grouping retired); but the long In-house-roadmap paragraph still leans on "packaging axis" phrasing that overlaps the retired grouping vocabulary and should be checked against 0362's "grouping = presentation tag" rule.
- **in_masterplan:** YES. This is arguably the most masterplan-binding ADR in the slice — it *is* the rollout-order authority ("Roadmap artifacts must tag work to the canonical service packaging order"; "first deliverable" scope list). A generated masterplan's sequencing section should cite this directly.
- **tensions:** (1) Mentions `MVP/preview/reduced scope` only to *forbid* them ("It is explicitly not an MVP, preview, or reduced-scope launch") — this is the correct disposition of the retired M0–M3/MVP vocabulary (map §2), not a leak. (2) "Korea localization pack" as a hard first-deliverable gate is a strong, specific binding — confirm it survives in current masterplan. (3) References 0220 as "Consumer Intelligence must remain separated from internal cloud intelligence" — brand-clean (no "Foundry"). (4) Title/filename drift (above). (5) The huge gate list (Talos/Ubuntu/RHEL/macOS one-command setup, distroless-by-default, remote Talos cluster-join) aligns with map §3 orchestration (Talos node-OS, ADR-0375) — coherent, no conflict.
- **hyperscaler_challenge:** ALIGNED. AWS/Stripe/Palantir/Shopify all ship depth-first, service-by-service GA with per-service SLOs/limits/docs before horizontal sprawl; the ADR cites them accurately. A hyperscaler would make this decision. Does NOT argue for archive; argues only for the naming amend.
- **ai_slop:** Borderline. The In-house-roadmap paragraph is an ~800-word run-on that restates the Decision/Gate/Operational sections three times (gate list ≈ operational list ≈ roadmap prose). This is *verbosity/redundancy*, not fabrication — every clause maps to a real requirement — but it's the slop-iest prose in the slice and a candidate for compression on amend.
- **refinement:** (a) Reconcile title vs filename (`Service Packaging` vs `vertical-slice`); (b) compress the triplicated gate/operational/roadmap prose to one canonical gate list; (c) verify "packaging axis" phrasing against ADR-0362 grouping-retirement.
- **consensus_needed:** FOUNDER QUESTION — Is the canonical name "service-packaging rollout order" or "vertical-slice rollout order"? The MASTERPLAN says development order is "vertical-slice"; the ADR title says "service packaging." Pick one canonical term so the generated masterplan's sequencing label is stable.

---

### ADR-0218 — Tenant Granular Control Surface

- **decision_atom:** Ship a Tenant Admin Console (inside the Application B2B shell, ADR-0061) as the canonical tenant-facing control plane for roles/SCIM users, a-la-carte product enablement, visual Cedar-fragment authoring, tenant data-class labels, approval workflows, per-tenant audit slicing, env tiers, API keys, IdP federation, and JIT grants — tenant-authored policy *composes with* platform-base-deny and can never disable base-deny, sovereignty, audit emission, or personal-context isolation.
- **domain:** tenancy, authz-policy (cross-cutting: a tenancy control surface whose core is Cedar-fragment composition).
- **current_status:** Accepted.
- **disposition:** KEEP. Current, correct, non-conflicting. Sits cleanly atop the canonical posture: Cedar universal gate (ADR-0243/0246), tenant = universal scoping primitive (ADR-0244), per-tenant audit slicing (ADR-0163), no-code-first builders (0219). Explicitly preserves platform guardrails (Alt-4 rejection).
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL. Real planning_impact (Tenant Admin Console is a named first-deliverable surface, 4-phase rollout, audit/rollback operational reqs) but prose-only binding, no `masterplan_ref` field.
- **tensions:** (1) "environment tiers: test, staging, production" and "API keys per tier" use the word **tier** — but this is the *environment-tier* axis, NOT the retired tenant capability-`tier-system` (ADR-0329); harmless, though a glossary check could trip on bare "tier." (2) "Tenancy RLS … and cell isolation" leans on `cell` — fine as deployment pattern (0333). (3) "capability tier" does NOT appear here (it appears in 0220) — clean.
- **hyperscaler_challenge:** ALIGNED. Okta/Entra admin centers, AWS IAM+Organizations (local policy composition under provider guardrails), Stripe Dashboard (keys/envs/scopes), Google Workspace Admin — every hyperscaler ships a delegated tenant control plane with hard provider guardrails. They would make this decision. No amend/archive pressure.
- **ai_slop:** None material. Concrete control-plane enumeration with real guardrail semantics.
- **refinement:** Optional: disambiguate "environment tier" vs retired "capability tier" in a one-line glossary note so automated tier-vocab lints don't false-positive.
- **consensus_needed:** None.

---

### ADR-0219 — No-Code-First UX with Optional AI-Assist

- **decision_atom:** Visual deterministic builders are the primary UX for professional/admin tasks (workflows, approvals, schema, reports, data-class tagging, roles, API keys, audit query, simple Cedar matrices); AI assist (via `microservices/intelligence/`) is an opt-in accelerator for fuzzy/semantic work that always drafts *into* the builder for human review and is never auto-applied.
- **domain:** product-ux, intelligence-ai (cross-cutting: a UX-primacy doctrine whose AI half binds the consumer-intelligence substrate).
- **current_status:** Accepted.
- **disposition:** KEEP. Current, correct, non-conflicting. Routes AI through `microservices/intelligence/` (brand-clean per 0220/0335, no "Foundry" for consumer AI). The "no hidden shadow loops / AI tokens only on invocation" rule is a sound cost+audit guardrail.
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a.
- **truth_flag:** TRUE.
- **in_masterplan:** PARTIAL. Planning_impact present (builders are first-deliverable surfaces; AI-draft provenance/cost-attribution operational reqs; 4-phase rollout) but prose-only binding.
- **tensions:** None material. Fully consistent with 0218 (visual builders as primary control path) and 0220 (Intelligence supplies the AI-assist + cost attribution + audit). The "AI auto-apply after confidence threshold" rejection (Alt-4) is the correct safety posture.
- **hyperscaler_challenge:** ALIGNED. Notion/Zapier/n8n (visual builders), Looker/Tableau (drag-drop analytics), Microsoft Copilot / Google Gemini (AI drafts *into* existing surfaces, not auto-apply) — the "AI drafts, human activates" pattern is exactly current hyperscaler product doctrine. They would make this decision. No amend/archive pressure.
- **ai_slop:** None material. Genuinely useful deterministic-vs-fuzzy task partition.
- **refinement:** None required.
- **consensus_needed:** None.

---

### ADR-0220 — Consumer Intelligence Substrate

- **decision_atom:** Create `microservices/intelligence/` (brand label "oyatie intelligence", crate prefix `oya-intelligence-*`) as the single consumer-facing AI substrate owning per-tenant/per-user AI memory, prompt history, cross-product orchestration, model routing, cost attribution, consent/opt-out, EU-AI-Act tracking, DSAR deletion, and AI-decision audit — strictly separated by audience from internal Foundry, which keeps only internal dev/CI/eval workloads (shared *runtime* substrate like Milvus/Wasmtime/Cedar/audit-chain is allowed, shared *audience* is not).
- **domain:** intelligence-ai, agentic-platform (cross-cutting: defines the consumer-AI substrate AND draws the boundary to the internal agentic/Foundry platform).
- **current_status:** Accepted.
- **disposition:** KEEP (substance) with **AMEND-on-touch** for retired brand. The *decision* — split consumer-AI (`intelligence`) from internal platform — is exactly the boundary that ADR-0335 later formalizes ("Foundry retired → absorbed by intelligence; Governance stays separate"). So 0220 is a TRUE precursor/keystone, NOT superseded; 0335 *amends* 0220's audience model (per 0335's amends list which includes 0220), it does not archive it. The leak is purely the live "Foundry" *brand* used as the internal-platform label.
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a (not superseded; ADR-0335 *amends* it).
- **truth_flag:** PARTIAL. Core split TRUE and still canonical. Two stale items: (a) the internal side is still branded **"Foundry"** / **"Hermes"** — both RETIRED vocab (map §2: Foundry→intelligence/governance per 0335/0347; Hermes→intelligence per GLOSSARY). Post-0335 the internal platform is "intelligence (internal self-modification layer) + governance," so "Foundry remains internal only" is stale framing even though the *separation principle* is right. (b) "capability tier" appears twice ("model routing by … capability tier", "EU AI Act … per capability tier") — `tier` here is an AI-capability/model axis, NOT the retired tenant `tier-system` (0329) and NOT the autonomy T1–T4 axis; harmless but a third distinct "tier" meaning in the corpus worth a glossary disambiguation.
- **in_masterplan:** PARTIAL. Strong planning_impact (canonical path/crate-prefix mandate, shared-substrate isolation table, 4-phase rollout, per-call audit-record schema) and the naming rules are effectively normative; prose-only binding.
- **tensions:** (1) **Retired-brand drift** — uses "Foundry" and "Hermes" as live internal labels; per map §1.3 foundry-dissolution chain (0136→0239→0247/0255→0335) the brand is dead. This is the same M-04 (Foundry/intelligence audience conflation) mistake that 0221 codified a CI gate against — ironically 0220 *is* the audience-split ADR yet still spells the internal side "Foundry." (2) `axis-foundry` listed as a Decider — a retired axis label. (3) ADR-0136 cited as "Foundry internal only, Accepted" — but map §1.3 flags 0136 as stale-front-matter (declared superseded by 0335/0247 yet still reads Accepted on disk); trust the superseding ADR. (4) Shared Milvus aligns with map §3 data posture (Milvus canonical, ADR-0192) — no conflict.
- **hyperscaler_challenge:** ALIGNED. Apple Intelligence (consumer AI brand across surfaces), Microsoft Copilot (shared substrate, audience-specific boundaries), Google Gemini (centralized memory/consent/routing), Palantir (internal "Foundry" platform brand distinct from customer AI) — the audience-split + centralized consumer-AI substrate is exactly hyperscaler practice. They would make this decision. Argues for AMEND (rebrand internal side off "Foundry"), not archive.
- **ai_slop:** None material. The shared-substrate isolation table and per-call audit schema are concrete. The slop risk is purely the retired-brand residue, not fabrication.
- **refinement:** On next touch, rebrand the internal side from "Foundry"/"Hermes" to "intelligence (internal layer) + governance" per ADR-0335/0347; update `axis-foundry` decider label; re-point the ADR-0136 citation to 0335 as the governing audience-split authority. Add a one-line note that "capability tier" here = AI/model axis, distinct from retired tenant-tier and from autonomy-T1–T4.
- **consensus_needed:** FOUNDER QUESTION (low heat, mostly settled by 0335): confirm that 0220's "Foundry internal / Intelligence consumer" split is now read as "intelligence-internal-layer + governance / intelligence-consumer-layer" — i.e., 0335 governs the labels and 0220 is the historical origin of the split. (Map §3 already lists 0220 as "historical" under Intelligence — so this is documentation hygiene, not a live contest.)

---

### ADR-0221 — Agentic Development Pipeline Hardening

- **decision_atom:** Codify 15 recurring agentic-dev mistakes into durable doctrine + automation — pre-dispatch validation templates (§Audience/§Abstraction-rationale/§Catalog-collision-check), 4 CI governance gates (vacuous-green, orphan-ADR-citation, version-pin-source-cited, buildability-line-count), nightly doctrine-intake→ADR-scaffold, and PR-charter scope-lock — under an "encouragement-over-prevention" rule where hooks encourage and CI hard-blocks only irreversible-class violations.
- **domain:** ci-cd-build, governance-process (cross-cutting: the agentic-pipeline quality-gate doctrine that is both CI substrate and process governance).
- **current_status:** Accepted.
- **disposition:** KEEP (substance) with **AMEND-on-touch** for retired vocab + stale CI naming. The doctrine (mistake-corpus → fitness gates, encourage-vs-block split, scope-lock) is current and self-aware; it correctly supersedes the PR-#143 conversation checkpoint (its own References call the checkpoint "now superseded by on-disk ADRs"). Amend, not archive, because of (a) retired `oya-foundry-*` crate/lane naming and (b) the now-stale `oya vcs`/grit-pipeline VCS framing.
- **proposed_resolution:** NA (Accepted).
- **governing:** n/a (it is itself a governing/close-out ADR for the cohort; archives the checkpoint JSON, not an ADR).
- **truth_flag:** PARTIAL. Mistake corpus + gate doctrine TRUE and durable. Stale/retired items: (a) **Architecture-map automation crates** named `oya-foundry-architecture-map-kernel` / `oya-foundry-architecture-map-app` — `oya-foundry-*` is RETIRED → `oya-governance-*` (ADR-0347, map §2); the file even *uses* `oya-governance-architecture-map-freshness` alongside, so the residue is internally inconsistent. (b) **VCS primitive correction** declares `oya vcs` canonical and "grit retired per ADR-0116" — but the *forge* canon has since moved (ADR-0363 retires the bespoke `oya vcs`/changeset-SM agentic-VCS entirely in favor of plain git + Forgejo PRs + Prow-shaped CI; map §5 / §1.3 VCS chain). So "oya vcs is canonical, direct git/gh forbidden" is **STALE/WRONG** as of ADR-0363/0510 — current canon is plain git + Forgejo. (c) Mentions `axis-foundry` decider + "lives in Foundry" for the arch-map automation — retired brand. (d) `master-plan-sequencing.json#forbidden_primitives` reference and `crates/oya-check-*` future-port names are forward-looking, fine.
- **in_masterplan:** PARTIAL. The 4 CI gates are real, wired (`tools/governance/adr-0221-governance-gates.sh` in `pr-tests.yml` under `oya-governance-*` lane) and ratchet the buildability/version-pin/orphan-citation discipline the whole corpus depends on — strong planning_impact. But it binds via tooling/lanes, not a `masterplan_ref` field.
- **tensions:** (1) **VCS contradiction with ADR-0363/0510** (above) — 0221 says "oya vcs canonical, git/gh forbidden"; current forge canon is plain git + Forgejo, bespoke VCS retired (then reframed transitory per 0510). This is the sharpest staleness in the slice. (2) **Retired `oya-foundry-*` crate residue** vs ADR-0347 governance-rename — directly the kind of brand-residue lane (MFL-0002/0003) the map §2 lint-signal warns about. (3) **Self-merge contract path** ("multispectrum evidence + reviewer-agent verdict + self-merge") tensions with the audit's own separation-of-passes principle and with PR #605 being a normal merged PR — verify self-merge is still sanctioned under current forge/CI canon (Argo Workflows + Forgejo Commit Status gates, map §1.3). (4) Cites ADR-0136 "amendment, Foundry internal-scope" — stale per map §1.3. (5) Internally consistent on encouragement-over-prevention; no contradiction there.
- **hyperscaler_challenge:** ALIGNED (on the doctrine), QUESTIONABLE (on the bespoke-VCS implication). Stripe/Cloudflare/Linear/AWS all run mistake-corpus→fitness-function + scope-lock-at-proposal practices (cited accurately) — a hyperscaler would absolutely make the "codify mistakes into CI gates" decision. BUT the embedded "oya vcs canonical, forbid direct git" stance is the kind of own-the-VCS choice hyperscalers split on (Google does run bespoke Piper/CitC; most do not) — and SOURCE itself has since reversed toward plain git + Forgejo (0363). So the VCS clause argues for AMEND (strip/repoint the stale VCS framing); the core gate doctrine argues KEEP.
- **ai_slop:** Low. This ADR is unusually evidence-dense and self-critical (it's literally an anti-slop / anti-mistake ADR; M-06 vacuous-green and M-11 thin-IP gates are direct anti-slop controls). The residue is retired-vocab and stale-cross-canon, not fabrication. Note the meta-irony: an ADR whose whole purpose is preventing drift (M-04 Foundry/intelligence conflation, M-12 stale-refs-after-rename, M-13 orphan-citation) itself now carries Foundry-brand residue and a stale VCS cross-canon — exactly the drift classes it polices.
- **refinement:** (a) Rename `oya-foundry-architecture-map-*` → `oya-governance-architecture-map-*` (ADR-0347); (b) repoint/retire the `oya vcs`-canonical + "git forbidden" clause to current forge canon (plain git + Forgejo PRs, ADR-0363; transitory-host note per 0510); (c) update `axis-foundry` decider + "lives in Foundry" → intelligence/governance; (d) re-point ADR-0136 citation to 0335. None of these change the gate doctrine.
- **consensus_needed:** FOUNDER QUESTION — Does ADR-0221's "oya vcs is canonical; direct git/gh forbidden" clause still hold, given ADR-0363 retired the bespoke agentic-VCS for plain git + Forgejo and the founder's migration directive is GitHub `jason931225/oyatie`? This is a genuine cross-canon contradiction (0221 vs 0363/0510 vs founder GitHub directive) and should be resolved before the masterplan generator ingests 0221's forbidden-primitives clause.

---

## Chunk notes

**Cohort identity.** ADR-0215–0221 are a single tightly-coupled batch: the PR #143 "#E substrate doctrines" follow-up, all dated 2026-05-18, all `Accepted`, all `council-architecture`-owned, all sourced from the same checkpoint JSON, and all closed out by ADR-0221 (which lists "0211/0212/0215-0220" as M-08, the queued ADRs that "lived only in conversation; almost lost"). They should be read and (if the masterplan is generated) ingested as a coherent doctrine block: multi-context principal (0215) → anti-lock-in contracts (0216) → depth-first rollout order (0217) → tenant control surface (0218) → no-code-first UX (0219) → consumer-intelligence substrate (0220) → the meta-gates that protect all of it (0221).

**No unaccounted proposals.** All 7 are `Accepted`; zero Proposed ADRs in this slice, so no RATIFY/DROP calls are owed.

**Disposition tally.** 5 clean KEEP (0215, 0216, 0218, 0219) + KEEP-with-AMEND on three (0217 title/filename drift; 0220 Foundry/Hermes brand residue; 0221 Foundry-crate + stale-VCS cross-canon). Zero ARCHIVE/SUPERSEDE/MERGE — none of these are superseded; 0220 and 0221 are *amended-by/refined-by* later ADRs (0335 for 0220's audience labels; 0363/0510 obsolete 0221's VCS clause) but their core decisions remain live keystones.

**Two cohort-wide retired-vocab leaks** (both AMEND-on-touch, neither disposition-changing):
1. **Foundry brand residue** — surfaces in 0215 (ADR-0239 ref), 0220 (whole internal-side framing + `axis-foundry` decider + Hermes), and 0221 (`oya-foundry-architecture-map-*` crates + `axis-foundry` + "lives in Foundry"). Per map §2 the brand is dead (→ intelligence/governance, ADR-0335/0347). Ironically 0220 is the audience-split ADR and 0221 codifies the anti-Foundry-conflation gate (M-04) — yet both carry the residue.
2. **"tier" overload** — 0218 ("environment tiers", "API keys per tier") and 0220 ("capability tier" ×2) both use bare "tier" in senses *distinct* from the retired tenant `tier-system` (ADR-0329) AND from the live autonomy T1–T4 axis. Harmless individually but the corpus now has ≥3 live "tier" meanings; a glossary disambiguation would de-risk automated tier-vocab lints.

**Sharpest live contradiction in the slice:** ADR-0221's "oya vcs is canonical; direct git/gh forbidden" clause vs the current forge canon (ADR-0363 plain-git + Forgejo, ADR-0510 transitory-host) vs the founder's GitHub `jason931225/oyatie` migration directive (map §5 fault-line #4). The bespoke-VCS retirement is the one item here that has been *materially overtaken by later canon*, and it should be flagged to the masterplan generator so 0221's `forbidden_primitives` don't get ingested as live.

**Masterplan-binding standout:** ADR-0217 is the rollout-order/sequencing authority for the cohort and is the most directly masterplan-load-bearing ADR in the slice (in_masterplan: YES). If the masterplan is generated from ADRs, 0217 supplies the canonical sequencing block — so resolving its title/filename naming (service-packaging vs vertical-slice) matters more than the cosmetic class would suggest.

**Front-matter binding gap (cohort-wide):** none of the 7 carries an explicit `planning_impact`/`masterplan_ref` front-matter field; all binding is prose-only. Consistent with the repo's measured 8.8% ADR-binding reality (map §4). If the founder lands the *generated-from-ADRs* design (planning-ssot-consolidation.md), these 7 would each need machine-readable `planning_impact`/`deliverables` front-matter added on re-authoring; if the *masterplan-as-authority* design wins, they need `masterplan_ref` back-binding. Both readings flagged per map §4 instruction — do not assume direction.
