# ADR Audit — SOURCE chunk 26

- **Side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`, `docs/decisions/`)
- **Chunk:** source-26 (slice 176–182 of the sorted `ADR-*.md` listing)
- **Range:** ADR-0201 … ADR-0207 (contiguous)
- **ADRs reviewed:** 7 (0201, 0202, 0203, 0204, 0205, 0206, 0207)
- **Auditor posture:** READ-ONLY; only this artifact written. Keystone map consulted for supersession/retired-vocab.
- **Cluster character:** This is the **front-end / client-substrate / product-UX batch** (all dated 2026-05-18, mostly `council-architecture`). Six of seven are "pick a community standard now, name the in-house Phase-2 successor, gate the vendor behind an adapter" decisions. The recurring risk in this batch is **retired-vocabulary leakage** (Backstage/ADR-0170, Foundry, ADR-0183-Kyverno) embedded as load-bearing references, not the core technology choices, which are mostly sound.

---

### ADR-0201 — Email + transactional comms adapter substrate

- **decision_atom:** Transactional email is a substrate-level concern owned by a single `comms-email` µservice exposing an `EmailComms` trait with four real adapters (SES default-cloud, Postal self-hosted/sovereign, Mailgun second-source, generic SMTP fallback), mandatory per-tenant DKIM/SPF/DMARC, MJML+Liquid templating, and a gated Phase-2 in-house Rust SMTP relay (`oya-comms-email-server`) triggered by numeric scale/sovereignty thresholds.
- **domain:** comms-notify
- **current_status:** Accepted
- **disposition:** AMEND
- **proposed_resolution:** NA (already Accepted)
- **governing:** n/a (no superseding ADR); amend is internal-reference hygiene only
- **truth_flag:** PARTIAL — core decision TRUE; one STALE reference: Context line 28 lists "Foundry (long-running run completion + cost-cap warnings)" as a sender. **Foundry is RETIRED → intelligence** (ADR-0335/0347 per keystone §2). The sender still exists, but its name should read "intelligence." Also references ADR-0202's "Tier-B OpenTofu DNS module" (valid, in-chunk) and OpenBao/ADR-0173 (valid).
- **in_masterplan:** PARTIAL — carries a substrate decision + manifest delta knobs (`comms.email.provider`, rate-limit, dkim_rotation, default_from_domain) that are masterplan-bindable; no explicit `planning_impact`/`masterplan_ref` front-matter (this ADR uses the older prose-header format, not YAML front-matter).
- **tensions:** Postal adapter is AGPL (Ruby/RabbitMQ/MariaDB) — collides with the OSI-strict/no-AGPL-in-product posture (keystone §3 license row, ADR-0013/0211/0345). The ADR itself flags this (sovereign packs may reject the Ruby stack) and routes around it via Phase-2 Rust relay, so it is an acknowledged carve-out, not a contradiction. Mailgun is commercial — explicitly gated, never default (consistent with ADR-0173).
- **hyperscaler_challenge:** ALIGNED. Google/AWS/Azure all treat transactional email as a managed substrate behind a provider-abstraction (AWS SES/Pinpoint, GCP via SendGrid partnership, Azure Communication Services). The adapter-trait + "never assume SES" posture is exactly hyperscaler practice. The Phase-2 "build our own Rust MTA" ambition is where a hyperscaler would push back: they buy/partner for deliverability reputation (IP warm-up, feedback loops with Gmail/Outlook) rather than build an MTA — argues mildly for keeping Phase-2 firmly gated (it already is, behind ≥1M sends/day numeric triggers). Net: not an amend/archive driver.
- **ai_slop:** Low. Specific, numerate (SES baseline $0.10/1k, 75% trigger, send-rate ceilings), names real crates (`mrml`, `liquid`). The "no Noop fallback" discipline is a genuine signal, not slop.
- **refinement:** Replace "Foundry" with "intelligence" in the sender list; confirm OpenBao vs the keystone's secret-store canon (keystone uses OpenBao consistently — OK).
- **consensus_needed:** none beyond the standing AGPL-carve-out question (does Postal's AGPL stack clear the OSI-strict bar for sovereign self-host? — already an accepted server-side carve-out pattern, so likely no new escalation).

---

### ADR-0202 — GitOps + IaC + Cluster lifecycle: three-tier separation

- **decision_atom:** Infrastructure responsibility splits into three non-overlapping tiers — ArgoCD (Tier-A app deployment), OpenTofu (Tier-B cloud-side resources: VPC/IAM/DNS/KMS/namespace/ArgoCD-project bootstrap), Cluster API (Tier-C cluster lifecycle) — enforced by a boundary table and the `oya-check-iac-tier-discipline` gate, with no in-house replacement planned.
- **domain:** orchestration-scheduling (cross-cuts ci-cd-build / security-supplychain)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted)
- **governing:** n/a
- **truth_flag:** TRUE. Fully consistent with the canonical orchestration posture: ArgoCD + Cluster API are exactly what ADR-0171/0375 endorse (Talos+CAPI+ArgoCD substrate, keystone §3). OpenTofu-over-Terraform is the explicit ADR-0173 anti-BSL call and matches retired-vocab table (Terraform BSL → OpenTofu). One soft note: ADR-0183 is cited (Related, line 16) for "Kyverno admission policy applied during GitOps reconciliation" — **ADR-0183 is Superseded by ADR-0379 (Kubewarden default admission)** per keystone §1.1. The reference is contextual (admission happens at reconcile time, still true) not load-bearing on the tier choice, so this is a minor stale-ref, not enough to downgrade from KEEP.
- **in_masterplan:** PARTIAL — strong substrate decision + boundary table + manifest implications; no YAML front-matter binding (prose-header format).
- **tensions:** None material. Pulumi/Crossplane retained as per-tenant exceptions (clean). The only watch-item is the stale ADR-0183/Kyverno reference (now Kubewarden) — worth a one-line amend if a sweep touches this file, but does not change the decision.
- **hyperscaler_challenge:** ALIGNED. The three-tier separation (app-deploy / cloud-resource / cluster-lifecycle) is precisely how Google (Config Sync + Config Connector + GKE fleet), AWS (ArgoCD/Flux + CloudFormation/CDK + EKS), and Azure (Flux + Bicep + AKS) structure it. The ADR's own framing — "hyperscalers ship opinionated overlap; our value-add is the discipline that keeps them separate" — is an honest, correct read. No amend/archive pressure.
- **ai_slop:** Very low. The boundary table is concrete and operationally testable. Real licenses cited correctly (ArgoCD Apache-2.0, OpenTofu MPL-2.0, CAPI upstream).
- **refinement:** Update the ADR-0183 "Kyverno" related-note to ADR-0379/Kubewarden on next edit (non-blocking).
- **consensus_needed:** none.

---

### ADR-0203 — Documentation engine: three-tier separation

- **decision_atom:** Documentation splits into three engines over a single Markdown source-of-truth — mdbook (Tier-1 in-repo technical), Backstage TechDocs (Tier-2 service catalog), SvelteKit (Tier-3 public/marketing, with Redoc/AsyncAPI API reference) — with a Phase-2 in-house `oya-developer-portal` gated only if Backstage outgrows "service catalog + TechDocs."
- **domain:** docs-ssot-masterplan (cross-cuts product-ux)
- **current_status:** Accepted
- **disposition:** AMEND
- **proposed_resolution:** NA (Accepted)
- **governing:** ADR-0394 (bespoke-Rust IDP; Backstage quarantined) supersedes ADR-0170 — the Tier-2 binding
- **truth_flag:** STALE (PARTIAL). The mdbook (Tier-1) and SvelteKit (Tier-3) choices are TRUE and fully aligned. **The Tier-2 binding is stale:** this ADR pins Backstage TechDocs via ADR-0170, but **ADR-0170 is Superseded by ADR-0394** (bespoke-Rust IDP; "Backstage quarantined" / feature-reference-only) per keystone §1.1 + §2 retired-vocab. The ADR's own Phase-2 escape ("`oya-developer-portal` triggered if Backstage scope grows") actually anticipates this — but post-0394 the trigger has effectively *already fired* (Backstage is now demoted to reference, not the canonical Tier-2 engine). So Tier-2 must be re-pointed from "Backstage TechDocs (canonical)" to "bespoke-Rust IDP/TechDocs-equivalent per ADR-0394; Backstage = feature reference."
- **in_masterplan:** PARTIAL — substrate decision, three-tier model; no YAML front-matter binding.
- **tensions:** Direct tension with ADR-0394 (Backstage demotion). Secondary: Tier-2's TechDocs uses MkDocs (pure Python) — at odds with the Rust-primary stance the ADR itself professes for Tiers 1+3; ADR-0394's bespoke-Rust IDP resolves this. Also references ADR-0185 SvelteKit-Phase-1 (valid, that chain is intact in this batch).
- **hyperscaler_challenge:** QUESTIONABLE (on Tier-2 only). Google (g3doc), AWS, Azure run a single internal docs-as-code pipeline rather than three engines; the three-audience split is defensible (internal-eng / catalog / public-marketing genuinely differ), but the *Backstage* dependency is the part a hyperscaler would not keep — they build the catalog in-house (Google's internal service catalog, AWS internal portals). This argues for AMEND toward the ADR-0394 in-house IDP, exactly the direction source already chose elsewhere.
- **ai_slop:** Low on structure; the slop-risk is the *stale dependency*, not fabrication. OpenAPI 3.2.0 / AsyncAPI 3.1.0 version pins are specific and checkable.
- **refinement:** Re-bind Tier-2 to ADR-0394 (bespoke-Rust IDP); reframe Backstage TechDocs as the Phase-0 reference/transitional engine, not canonical; the existing Phase-2 trigger language can be reused near-verbatim.
- **consensus_needed:** "Post-ADR-0394, is Backstage TechDocs still the *transitional* Tier-2 docs renderer, or is it fully removed in favor of the bespoke-Rust IDP from day one?" — founder call on transition window.

---

### ADR-0204 — Workflow Studio canvas / node-editor library

- **decision_atom:** Workflow Studio's canvas uses svelte-flow (`@xyflow/svelte`, MIT) in Phase-1 SvelteKit behind a `CanvasAdapter` trait, with an in-house Rust-native `oya-canvas` (Leptos + native platform canvases) as the gated Phase-2 successor, committing to concrete perf bars (60fps @ 1000 nodes, viewport virtualization + 3-tier LOD, WebGL escape hatch >5k nodes).
- **domain:** product-ux (cross-cuts agentic-platform — Workflow Studio is the hero workflow product)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted)
- **governing:** n/a
- **truth_flag:** TRUE. Has proper YAML front-matter (`id/status/supersedes:[]/superseded_by:[]/related/related_specs`) and binds to `/specs/products/workflow-studio.json` — this is the *newer, cleaner* ADR format and is masterplan-binding-ready. Performance claims are honestly hedged ("2000+ nodes drops to 30-45fps without WebGL — n8n hit the same wall"), which is a strong anti-slop signal. References Loro CRDT via ADR-0145 (valid; ADR-0145 is the inter-µsvc-comms-reform ADR, live).
- **in_masterplan:** YES — `related_specs: /specs/products/workflow-studio.json`, concrete numeric triggers, adapter contract. This is the kind of ADR the generated-masterplan design wants.
- **tensions:** None internal. Minor cross-batch note: depends on ADR-0185 SvelteKit→Leptos rollout; if that stack chain ever shifts, the Phase-2 `oya-canvas-leptos` plan moves with it (correctly coupled, not a conflict). Loro CRDT is a hard external dependency pin (MIT) — acceptable.
- **hyperscaler_challenge:** ALIGNED. The "use the best OSS canvas now behind an adapter, build in-house only when you outgrow it (the n8n arc)" is precisely how a hyperscaler product org would sequence a node-editor — Google/AWS would not burn 3-6 person-months on a from-scratch canvas pre-PMF. The numeric Phase-2 trigger (≥10k nodes median OR p99 >16.67ms) is exactly the discipline they'd demand. No amend/archive pressure.
- **ai_slop:** Very low. Benchmark file paths, frame-budget math, hardware baseline, and rejected-alternatives (React Flow stack-mismatch, tldraw wrong-primitive) are all concrete and correct.
- **refinement:** none material.
- **consensus_needed:** none.

---

### ADR-0205 — Code editor canonical: CodeMirror 6

- **decision_atom:** CodeMirror 6 is the canonical code editor for every in-product web code surface (Lezer-parsed, headless, <200KB gzip, a11y-first, per-language `@codemirror/lang-*` packs + an in-house Cedar grammar), with native shells using platform-native text systems and no Phase-2 in-house rebuild (commodity layer).
- **domain:** product-ux (cross-cuts authz-policy via the Cedar grammar binding)
- **current_status:** Accepted
- **disposition:** AMEND
- **proposed_resolution:** NA (Accepted)
- **governing:** ADR-0335/0347 (Foundry retired → intelligence) for the naming fix; ADR-0379 (Kubewarden) / ADR-0243-0246 (Cedar universal gate) for the ADR-0183 stale-ref
- **truth_flag:** PARTIAL (TRUE core, two stale refs). CodeMirror 6 choice is TRUE and well-argued (Monaco bundle/a11y/React-coupling rejection is correct). **Stale ref #1:** Context line 25 lists "**Foundry**: tool definition authoring (Cedar policy fragments, OpenAPI tool spec, prompt templates)" as a code surface — Foundry is RETIRED → **intelligence** (ADR-0335/0347). The surface persists; the name should be "intelligence." This recurs at line 64 ("Foundry tool development"). **Stale ref #2:** line 62 cites **ADR-0183** ("Cedar is oyatie's authz language per ADR-0183") — ADR-0183 is Superseded; the live Cedar-as-universal-gate authority is ADR-0243/0246 (admission moved to Kubewarden/ADR-0379). Cedar-the-language is still correct, only the citation is stale.
- **in_masterplan:** YES — clean YAML front-matter, `related_specs: /specs/products/workflow-studio.json`; masterplan-binding-ready.
- **tensions:** Only the retired-vocab leakage above. The "where CM6 is NOT right" nuance (full IDE-class → revisit, don't silently extend) is good discipline, not a tension.
- **hyperscaler_challenge:** ALIGNED. CodeMirror 6 is the documented choice at Sourcegraph (which *migrated off* Monaco), Replit, Linear — i.e., the exact precedent set a hyperscaler-grade product org follows for focused/inline code surfaces. The "don't reinvent a 2-person-year editor with no differentiation" call is the correct build-vs-buy verdict. No amend/archive pressure on the *decision*; amend is citation-hygiene only.
- **ai_slop:** Low. Concrete caps (≤50k lines / ≤5MB / ≤10k tokens/sec before LSP auto-disables), real precedent companies, correct license (MIT). The Foundry/ADR-0183 references are stale-vocab, not fabrication.
- **refinement:** s/Foundry/intelligence/ (×2); re-cite Cedar authority to ADR-0243/0246 instead of the superseded ADR-0183.
- **consensus_needed:** none.

---

### ADR-0206 — i18n substrate: Fluent (Mozilla) + ICU MessageFormat

- **decision_atom:** Fluent (Mozilla, via `fluent-rs`) is the canonical single-source-of-truth authoring format for translatable strings with ICU MessageFormat plural/select/gender grammar at the runtime surface, per-stack adapter generators compiling Fluent to each platform's native catalog, locale-tag-driven RTL bidi, and an advisory `oya-check-i18n-coverage` gate (default 95%).
- **domain:** product-ux (cross-cuts compliance-residency via per-regional-pack overlays / ADR-0064)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted)
- **governing:** n/a
- **truth_flag:** TRUE. Clean YAML front-matter; no retired-vocab leakage; references are all live (ADR-0064 pack model, ADR-0185 client stack, ADR-0207 a11y/RTL). Rust-native rationale (`fluent-rs` works in Leptos + kernel tooling) is genuine and consistent with the Rust-primary substrate posture. MF2-deferral with a concrete revisit trigger (CLDR-TC Recommendation status) is honest.
- **in_masterplan:** YES — front-matter + `related_specs` binding; coverage gate is a concrete enforceable artifact.
- **tensions:** None. The locale-routing precedence (tenant default → user override → Accept-Language → en-US) is clean and consistent with the tenant-scoping primitive (keystone §3 tenancy). RTL locale subtag list is correct.
- **hyperscaler_challenge:** ALIGNED. Single-source-of-truth + per-platform catalog generation is exactly the Google (CLDR/ICU origin) / Apple (String Catalogs) / Mozilla (Fluent, operated at Firefox+AMO+MDN scale) practice. Choosing Fluent-source + ICU-surface over gettext is the modern, defensible call a hyperscaler i18n team would make. No amend/archive pressure.
- **ai_slop:** Very low. Correct version pins (ICU 76+, MF2 Tech Preview), real tooling ecosystem (Pontoon/Localazy/Crowdin), correct rejection of i18next (JS-only) for a cross-Rust stack.
- **refinement:** none.
- **consensus_needed:** none.

---

### ADR-0207 — Accessibility bar: WCAG 2.2 AA (AAA on regulated surfaces)

- **decision_atom:** WCAG 2.2 AA is the production-minimum accessibility bar for every user-facing surface (AAA on HIPAA/EU-AI-Act-high-risk/government packs), enforced cross-stack via per-stack automated runners (axe-core/pa11y on web, native scanners on Apple/Android/GTK/WinUI) and the advisory `oya-check-a11y-discipline` gate, with concrete keyboard/contrast/reduced-motion/form/canvas-a11y commitments.
- **domain:** product-ux (cross-cuts compliance-residency — AAA tied to HIPAA/EU-AI-Act regulated packs)
- **current_status:** Accepted
- **disposition:** KEEP
- **proposed_resolution:** NA (Accepted)
- **governing:** n/a
- **truth_flag:** TRUE. Clean YAML front-matter; binds `related_specs: /specs/hyperscaler-architecture-invariants.json` (the only ADR in this chunk to bind that spec — appropriate, a11y is an invariant). All cross-refs live (ADR-0204 canvas drag-and-drop a11y, ADR-0205 editor, ADR-0206 RTL, ADR-0064 per-pack AAA uplift). WCAG 2.2-over-2.1/3.0/508 rationale is correct (2.2 = current W3C Rec Oct-2023; 3.0 still Working Draft).
- **in_masterplan:** YES — front-matter + hyperscaler-invariants spec binding; CI-gated criteria are concrete and enforceable.
- **tensions:** None. The AAA-on-regulated-surfaces carve-out is correctly scoped (not full-fleet). Cross-references the canvas-a11y obligation of ADR-0204 (keyboard drag-and-drop) — properly coupled.
- **hyperscaler_challenge:** ALIGNED. WCAG 2.2 AA is the literal bar Apple, Google, GitHub, Stripe, Microsoft enforce; axe-core (Deque) is the de-facto industry a11y engine. "Comply, don't fork the standard; build only the gate + recipes + runner integration in-house" is exactly the correct build-vs-buy line. No amend/archive pressure.
- **ai_slop:** Very low. Names specific WCAG 2.2 success criteria by number (2.4.11 Focus Not Obscured, 2.5.7 Dragging), correct runner-per-stack table, correct licenses (axe-core MPL-2.0, pa11y MIT).
- **refinement:** none.
- **consensus_needed:** none.

---

## Chunk notes

**Disposition tally:** KEEP ×4 (0202, 0204, 0206, 0207) · AMEND ×3 (0201, 0203, 0205) · ARCHIVE/SUPERSEDE/MERGE ×0 · UNCLEAR ×0. **No Proposed ADRs in this slice** — all seven are Accepted, so no RATIFY/DROP decisions are owed.

**Dominant finding — retired-vocabulary leakage, not bad decisions.** Every AMEND here is a *reference-hygiene* fix, never a technology reversal. The three recurring poisons, all confirmed against the keystone map:
1. **Foundry → intelligence** (ADR-0335/0347): leaks into ADR-0201 (sender list) and ADR-0205 (×2, "Foundry tool definition authoring" / "Foundry tool development"). The named surfaces are real; only the brand is dead.
2. **Backstage / ADR-0170 → ADR-0394 bespoke-Rust IDP**: this is the *only structural* stale binding in the chunk — ADR-0203's entire Tier-2 rests on the now-superseded ADR-0170. Post-0394 the Backstage trigger has effectively already fired; Tier-2 needs re-pointing. This is the chunk's single highest-value amend.
3. **ADR-0183 (Kyverno/Cedar-separation) → ADR-0379 (Kubewarden) + ADR-0243/0246 (Cedar universal gate)**: stale *citation* in ADR-0202 (Kyverno admission) and ADR-0205 (Cedar-as-authz-language source). Cedar and reconcile-time admission are both still true; only the ADR number cited is superseded.

**Format split worth flagging for the masterplan-generation question (keystone §4).** This chunk straddles the two ADR eras cleanly: ADR-0201/0202/0203 use the **old prose-header format** (no YAML front-matter, no `related_specs`, status in a bullet), while ADR-0204/0205/0206/0207 use the **new YAML front-matter format** with `id/status/supersedes/superseded_by/related/related_specs` and bind to `/specs/*.json`. Under the *generated-from-ADRs* masterplan design, the four front-matter ADRs are already machine-harvestable (they carry the `related_specs` binding the drift-prevention gate wants); the three prose ADRs would need front-matter backfill before they could feed a generated masterplan. None of the three prose ADRs is wrong — they are simply pre-format and would be the first re-authoring candidates if the founder picks the ADRs-generate-masterplan direction.

**Quality signal — this batch is healthy.** All seven are concrete, numerate, honestly-hedged (svelte-flow's 2k-node wall admitted; SES MTA-reputation realism; MF2 deferral), name real licenses correctly, and use the consistent "community-standard-now / gated-in-house-Phase-2 / numeric-trigger" doctrine that matches both the keystone's "own when proven" ratchet (§5) and hyperscaler practice. None fabricates a posture; none is garbage. The cross-ADR coupling in the 0204–0207 product-UX quartet (canvas ↔ editor ↔ i18n ↔ a11y, all hanging off ADR-0185's SvelteKit→Leptos stack) is internally coherent.

**No number collisions** in this slice (0201–0207 are unique and contiguous; the keystone's known collisions are at 0377/0055/0145, outside this range). **No LINUX-pilot overlap** of concern beyond the guaranteed renumber-on-merge that applies to all linux ADRs.
