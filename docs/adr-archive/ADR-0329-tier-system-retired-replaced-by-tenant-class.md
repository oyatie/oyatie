---
id: ADR-0329
title: Tier system retired; replaced by tenant-class model
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - council-product
  - council-engineering
  - council-privacy
  - council-security
  - axis-policy-engine
  - axis-tenancy
  - axis-foundry
  - ops-compliance
  - ops-sre-reliability
owners:
  - council-architecture
  - council-product
  - council-engineering
  - council-privacy
  - council-security
  - axis-policy-engine
  - axis-tenancy
  - axis-foundry
  - ops-compliance
  - ops-sre-reliability
supersedes:
  - ADR-0316
superseded_by: [ADR-0702]
amends:
  - ADR-0316-capability-tier-over-product-fragmentation.md (retires every capability-tier clause; converts capability-tier-grant doctrine into tenant-class plus billing-component doctrine)
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (§D-19 OCI Always Free reworded from "OCI Bronze = Always Free" into tenant-class-conditioned "demo_trial defaults to OCI Always Free profile")
  - ADR-0064-canonical-base-localization (removes tier-gated localization clauses; localization activation is per-pack and per-tenant-class)
  - ADR-0249-multi-category-marketplace-doctrine.md (marketplace offers are not tier-segmented; offer eligibility is tenant_class-gated where required)
  - ADR-0251-compliance-pack-primitive (compliance-pack activation is tenant_class-gated, not tier-gated)
  - ADR-0255-byok-everywhere-credentials (BYOK opt-in is tenant_class-gated, not tier-gated)
related:
  - ADR-0108
  - ADR-0138
  - ADR-0244
  - ADR-0245
  - ADR-0248
  - ADR-0249
  - ADR-0251
  - ADR-0255
  - ADR-0263
  - ADR-0316
  - ADR-0322
  - ADR-0324
  - ADR-0327
  - ADR-0328
  - ADR-0330
  - ADR-0331
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/markdown-retirement-policy.json
  - /specs/microservices/manifest-schema.json
  - /specs/tenant-model.json
  - /specs/compliance-pack-schema.json
  - /specs/cedar-fragment-schema.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md
  - docs/decisions/ADR-0108-sunset-lifecycle-automation.md
  - docs/decisions/ADR-0138-intelligence-six-path-deprecation.md
  - docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md
  - docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
inbound_citations:
  - .omc/state/realignment-review-2026-05-21.md
  - .omc/state/wave-findings-aggregation-2026-05-21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 800
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322-plus-ADR-0328
enforcement_status: Accepted; retirement migration scheduled for Wave 15J
enforced_by:
  - oya-governance-tier-vocabulary-zero-residue
  - oya-governance-tenant-class-claim-binding
  - oya-governance-no-capability-tier-grants
  - oya-governance-bronze-silver-gold-platinum-zero-occurrence
  - oya-governance-registry-capability-tiers-deletion
  - oya-governance-naming-bnf-n014-n015-amendment
  - oya-governance-adr-0316-supersession-marker
  - oya-governance-cellular-criticality-tier-preservation
purpose: >
  Retire ADR-0316 and the entire Bronze/Silver/Gold/Platinum capability-tier
  doctrine. Capability tiers introduced feature gating between paying customers,
  fragmented the substance-bar quality contract across stratified service
  levels, multiplied per-microservice authoring debt, and created
  vocabulary residue measured at roughly 3,000 distinct call-sites across 77
  active microservices. The replacement is the two-class tenant model
  (demo_trial and paid) with composable billing components
  (revenue_share, per_seat, per_usage) codified by ADR-0330 and applied
  per microservice by ADR-0331. This ADR is the canonical retirement
  decision. It preserves ADR-0248 Amazon-shape cellular criticality
  tier-0..tier-3 vocabulary because that vocabulary is an
  infrastructure-availability classification, not a customer-facing
  capability-tier ladder. It sequences and binds the Wave 15J retirement
  migration, names every deliverable that must be deleted or amended,
  routes the cross-microservice cleanup through ADR-0108 sunset and
  ADR-0138 six-path deprecation patterns, and locks the cellular-criticality
  preservation rule so that no future cleanup wave accidentally deletes
  the surviving vocabulary.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0329: Tier system retired; replaced by tenant-class model

## Status

Accepted on 2026-05-21.

This ADR is immediately authoritative. It supersedes ADR-0316 per the
ADR-0108 sunset-lifecycle-automation contract and routes the retirement
migration through the ADR-0138 Strangler-pattern adaptation reused from
the foundry six-path consolidation. It is paired with ADR-0330
(tenant-class replacement model) and ADR-0331 (per-microservice
adoption). Until ADR-0330 and ADR-0331 land, this ADR alone is enough to
mark ADR-0316 superseded, to prohibit any new authoring that introduces
Bronze, Silver, Gold, or Platinum capability-tier vocabulary, and to
freeze the capability-tier registry against further activation.

The Wave 15J migration window is bounded by ADR-0328 §D batch
discipline. It will be authored as a coordinated remediation sub-wave
rather than as ad-hoc per-microservice patches. Until Wave 15J executes,
the retirement is enforced by the soft-gate lanes named in
§Enforcement-by-lanes below. After Wave 15J ships, the gates promote to
hard-blocker status per ADR-0138 Phase 2.

No new microservice is introduced by this decision.

No new product surface is introduced by this decision.

No deprecation of cellular criticality tier-0..tier-3 vocabulary is
introduced by this decision.

No deprecation of public-API stability tiers from ADR-0037 is introduced
by this decision; those are an unrelated stability ladder governed by
ADR-0037 and are out of scope here.

## Date

2026-05-21.

## Context

### A.1 Named pressure: tier vocabulary became corpus residue at scale

The Oyatie corpus is large. After the 2026-05-20 cross-microservice
audit wave aggregated in
`/Users/jasonlee/oyatie/.omc/state/wave-findings-aggregation-2026-05-21.md`
and the cross-cutting review at
`/Users/jasonlee/oyatie/.omc/state/realignment-review-2026-05-21.md`,
the residue of capability-tier vocabulary has been measured rather than
estimated. The numbers below are the audit observations as of the
realignment review snapshot at 2026-05-21.

A.1.1 Forty-eight microservices completed full coherence audits as of
the snapshot. The aggregated tier-retirement candidate count from the
agent self-reports across that audited set crosses 1,680 distinct
call-sites by the conservative extrapolation in the review.

A.1.2 The earlier scope-audit that counted character-level Bronze,
Silver, Gold, and Platinum occurrences reported approximately 9,300
character matches. Distinct call-sites (the unit that has to be
remediated) are fewer than character matches because a single clause
may contain several literal occurrences. The conservative cross-audit
extrapolation lands the cleanup unit count at roughly 3,000 distinct
call-sites across the 77 active microservices.

A.1.3 Sixty-one microservice directories under
`/Users/jasonlee/oyatie/microservices/<service>/capability-tiers/`
exist on disk at retirement time. Sixty of those directories contain a
`tier-matrix.md` artifact. The remaining one directory is a scaffold
without a matrix file.

A.1.4 Sixteen `capability-tier-deltas-vs-counterparts-2026-05-20.md`
deliverables exist on disk from Wave 2 Batch 2.1 and Wave 3 Batch 3.1.
Those deliverables are now superseded by this ADR; their content is
retained as historical evidence in the wave findings aggregation. They
are not re-authored under the tenant-class model; the substance-bar
replacement is the per-microservice tenant-class adoption record that
ADR-0331 will codify.

A.1.5 The centralised registry
`/Users/jasonlee/oyatie/registry/capability-tiers/` contains canonical
tier JSON files (`bronze.json`, `silver.json`, `gold.json`,
`platinum.json`), an `index.json`, a `checkpoint.json`, a
`microservice-tier-mapping.yaml`, and a `vendor-tier-mapping.yaml`. All
seven of those artifacts are retired by this ADR.

A.1.6 Seventeen standards documents under
`/Users/jasonlee/oyatie/docs/standards/` carry capability-tier
vocabulary or cross-reference ADR-0316. The list spans
`capability-tier-matrix.md` (the standards-level binding of the
retired registry), `capability-authoring.md`, `autonomy-ceiling.md`,
`brief-template.md`, `hyperscaler-best-practices.md`, `observability.md`
(per-tier rituals), `on-call.md` (per-tier escalation), `workflow-substrate-engine.md`,
`asyncapi-3-1-authoring.md`, `proto3-authoring.md`,
`ontology-projection-substrate.md`, `naming-convention-bnf-v4.md`
(N-014/N-015 forms), `tracing.md`, `slos.md`, `documentation-rigor.md`,
`mlops.md`, and the master `documentation-index.md`. Each of these gets
scrubbed in Wave 15J per the per-artifact replacement table in
Section D-3.

A.1.7 The naming-convention BNF v4 N-014 and N-015 rules currently
encode a `<microservice>.<capability>.<tier>` form (for example
`marketplace.deal.offer.t2`) and require the naming checker to scan
`registry/capability-tiers/*.yaml`. Both N-014 and N-015 are amended
in Wave 15J to drop the `.<tier>` segment and to drop the registry
scan obligation, per Section D-9 below.

A.1.8 ADR-0328 §D-19 contains the cross-reference "OCI Bronze = Always
Free" which is exactly the kind of indirect tier-vocabulary
contamination that this ADR retires. Section D-10 below carries the
exact reword: "demo_trial tenants default to the OCI Always Free
profile; paid tenants on OCI use paid OCI but may still allocate
Always-Free sandbox sub-tenancies."

A.1.9 The wave findings aggregation also records the universal
adoption-gap: zero of the 48 audited microservices currently model the
`{demo_trial, paid}` tenant_class enum plus the composable
`billing_components ⊆ {revenue_share, per_seat, per_usage}` set. The
adoption fix is owned by ADR-0331 (not this ADR), but the universality
of the gap is a context driver for retiring the existing tier
vocabulary before adoption begins, so that no microservice plumbs
tenant_class on top of a still-live capability-tier registry.

A.1.10 The realignment review also documents that earlier-audited
microservices used the older four-deliverable schema (including the
`capability-tier-deltas-vs-counterparts-2026-05-20.md` deliverable)
because the tier-retirement directive landed mid-session. Wave 15J does
not re-audit those microservices; their existing findings remain valid
under the new doctrine. The mismatch between the four-deliverable and
three-deliverable schemas does not require a corpus-wide back-fill.

### A.2 Named pressure: capability tiers fought multiple senior
doctrines simultaneously

A.2.1 The capability-tier ladder created feature gating between paying
customers. That outcome contradicts
`feedback_flat_product_catalog.md`, which says every customer gets the
same flat substrate, and contradicts
`feedback_quality_performance_scalability_bar.md`, which says the
quality bar is uniform at industry-leader grade rather than tiered.

A.2.2 The tier ladder also created lock-in mechanics
(`bronze → silver → gold → platinum` upgrade pressure) that conflict
with ADR-0216 (no-vendor-lock-in posture) and with the substance-bar
doctrine in ADR-0322 (substance is a blocker-class requirement, not
something that scales by tier).

A.2.3 The tier ladder demanded per-tier authoring evidence:
per-tier SLOs, per-tier compliance evidence, per-tier observability
rituals, per-tier feature flags, per-tier Cedar policy fragments,
per-tier UX shells, per-tier billing logic. That evidence multiplied
substance-bar load per microservice without producing a different
product. The ADR-0322 doctrine was strained because the substance unit
was the microservice, but the artifact unit became "microservice × four
tiers" by accident of the tier ladder.

A.2.4 The tier ladder also created vendor parity confusion. Counterpart
software (Stripe Standard/Plus/Premium, AWS Basic/Business/Enterprise
support, Salesforce Essentials/Professional/Enterprise) uses tier
ladders for pricing, not for feature stratification of the same
capability surface. Oyatie's tier ladder was authored as a parity move
("we have tiers because they have tiers") rather than as a structural
need. The user directive of 2026-05-20 declared this parity-driven
adoption invalid: "we don't have tiers."

A.2.5 The tier ladder created an "intelligent default" trap. Several
audits show capability-tier defaults assigned to features that should
have been universal (community access, audit-chain emission, basic
observability). Defaulting universal capabilities into Silver or Gold
created a false signal that paying customers had to upgrade just to
access basic substrate hygiene.

### A.3 Named pressure: capability tiers and cellular criticality were
becoming confusable

A.3.1 ADR-0248 (Amazon-shape cellular architecture) uses the words
"Tier 0," "Tier 1," "Tier 2," "Tier 3," and "Tier 4" to classify cells
by criticality and isolation guarantees: Tier 0 is the external
substrate, Tier 1 is the control plane, Tier 2 is the service-cell
fleet, Tier 3 is the data plane, and Tier 4 is the high-isolation
sovereign or financial-grade cell. ADR-0248 also uses "tier" inside
service-cell shuffle-sharding math, DR/BC classes (`dr_tier: T1` and
`dr_tier: T2`), and per-cell Helm chart naming (`cell-tier-1/`,
`cell-tier-2/`, `cell-tier-3/`).

A.3.2 ADR-0248's tier vocabulary is an infrastructure-availability
classification, not a customer-facing capability-tier ladder. The two
vocabularies share the word "tier" but address different concerns. A
customer never receives a Tier 1 cell as a product upgrade. A cell's
Tier 3 designation is an internal availability classification used by
Helm placement and SRE on-call.

A.3.3 The mid-2026-05-20 user directive ("we don't have tiers") refers
to the ADR-0316 capability-tier ladder, not the ADR-0248 cellular
criticality tier vocabulary. This ADR locks that distinction so that no
future Wave 15J cleanup pass accidentally deletes the ADR-0248 cellular
vocabulary while scrubbing ADR-0316's capability-tier vocabulary.

A.3.4 The lane that enforces tier-vocabulary zero-residue therefore has
an allow-list rule: it permits the strings "Tier 0," "Tier 1," "Tier
2," "Tier 3," "Tier 4," and the lowercase forms inside the
`microservices/cloud-iac/iac/helm/cell-tier-*` filename paths, the
`dr_tier` field, and the cellular criticality clauses of ADR-0248. It
refuses the strings "Bronze," "Silver," "Gold," "Platinum" used as
capability-tier names anywhere except inside ADR-0316 and inside this
ADR's history sections.

A.3.5 The lane also has an allow-list for the ADR-0037 public-API
stability tiers (preview, stable, GA) because those are an unrelated
versioning ladder. Section D-1.7 enumerates the full allow-list.

### A.4 Named pressure: the timing window matters

A.4.1 The retirement decision lands during the realignment wave that
brief-template.md, ADR-0328, master-plan-sequencing.json, and the
unified-ecosystem-thesis-2026-05-21 anchored. That wave is the only
context in which 77 microservices' worth of vocabulary can be
realigned in coordinated batches without producing an artifact churn
storm.

A.4.2 If retirement waited for a later quarter, the tier vocabulary
would propagate into newly authored microservices that the post-Wave-13
plan adds (B2B leaders, Big-8 expansion). That propagation would
double the cleanup cost. Retiring during the realignment wave clamps
the scope at roughly 3,000 call-sites instead of letting it grow.

A.4.3 The retirement also has to land before Wave 15A starts because
Wave 15A is the remediation wave that fixes the P0 backlog
(crm rewrite, cloud-billing spec sprint, marketplace 6-category
completion). Wave 15A would otherwise have to re-fix tier vocabulary
in every microservice it touches, multiplying its scope.

### A.5 Anchors this ADR binds

Anchor 1: the user directive "we don't have tiers" of 2026-05-20.

Anchor 2: the canonical retirement protocol in ADR-0108 sunset-lifecycle
automation.

Anchor 3: the Strangler-pattern adaptation in ADR-0138 for the
foundry six-path consolidation.

Anchor 4: the substance-bar canonical sequence in ADR-0328.

Anchor 5: the cellular-criticality preservation rule rooted in
ADR-0248.

Anchor 6: the memory files
`feedback_no_capability_tiers_2026_05_20.md` and
`feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`,
which crystallised the user directive into a retirement-plus-replacement
program.

Anchor 7: the cross-cutting findings in
`.omc/state/realignment-review-2026-05-21.md` and
`.omc/state/wave-findings-aggregation-2026-05-21.md`, which supply
the empirical scope.

### A.6 What this ADR does not assert

A.6.1 This ADR does not author the replacement tenant-class doctrine.
That doctrine is owned by ADR-0330. This ADR is the retirement
decision; ADR-0330 is the replacement decision.

A.6.2 This ADR does not author the per-microservice tenant-class
adoption plumbing. That plumbing is owned by ADR-0331 (cross-microservice
tenant-class adoption). This ADR scopes the migration but does not
prescribe the per-microservice plumbing IP.

A.6.3 This ADR does not delete any artifact in the source tree by
itself. Wave 15J executes the deletion under separate change sets per
ADR-0138 Phase 0 atomic-consolidation discipline.

A.6.4 This ADR does not remove the ADR-0037 public-API stability tier
vocabulary (preview, stable, GA), the ADR-0248 cellular criticality
tier vocabulary (Tier 0..Tier 4 and dr_tier classes), the ADR-0083
Rust error-handling tier vocabulary (Tier 1/Tier 2 library
classification), or any other internal "tier" usage that is not the
capability-tier ladder. Section D-1 enumerates the surviving
vocabularies.

A.6.5 This ADR does not promise that every capability-tier-named asset
will be deleted in one change set. Wave 15J phases the deletion per
ADR-0138's Strangler discipline to keep evidence intact and to keep CI
green during cutover.

### A.7 Cross-reference density

Inbound citations to ADR-0316 from inside the repo (other ADRs,
standards docs, microservice manifests, registry files) exceed 400
distinct file references at retirement time. The cross-reference scrub
is part of Wave 15J. The cross-reference fix-up rule is: replace
"capability tier" with "tenant-class capability" or with the relevant
per-microservice capability artifact name; replace "ADR-0316" with
"ADR-0329 (superseder) and ADR-0330 (replacement)"; remove standalone
Bronze/Silver/Gold/Platinum tokens entirely (no replacement is needed
because the substance-bar doctrine in ADR-0322 already provides the
uniform quality contract).

## Decision

### B.1 Decision statement

The Bronze/Silver/Gold/Platinum capability-tier doctrine codified by
ADR-0316 is retired in full. The capability-tier-grant primitive, the
per-microservice `capability-tiers/tier-matrix.md` artifact, the
centralised `registry/capability-tiers/` directory, the
`capability-tier-deltas-vs-counterparts-*.md` audit deliverable, the
N-014 and N-015 naming forms that suffix capability ids with `.<tier>`,
and every standards-doc clause that cross-references ADR-0316 as a live
authority are all retired. The replacement model is the tenant-class
binary (demo_trial and paid) with composable billing components
(revenue_share, per_seat, per_usage), codified by ADR-0330 and applied
per microservice by ADR-0331. The retirement uses ADR-0108 sunset
discipline and ADR-0138 Strangler phasing. The ADR-0248 cellular
criticality tier-0..tier-3 vocabulary is preserved because it is an
unrelated availability classification. ADR-0037 public-API stability
tier vocabulary is preserved because it is an unrelated versioning
ladder. ADR-0083 Rust error-handling tier vocabulary is preserved
because it is an unrelated library-discipline ladder.

### B.2 Numbered decision clauses

B2.001. Bronze, Silver, Gold, and Platinum vocabulary is retired as a
capability-tier ladder across the entire Oyatie corpus.

B2.002. The capability-tier-grant primitive is retired. No new
`tenant_capability_tier_grants` rows are created. The Postgres table
defined in ADR-0316 §D-7 is retained read-only as historical evidence
until Wave 15J Phase 2 archives it.

B2.003. The `microservice_capability_tier_contributions` table from
ADR-0316 §D-7 is retired. The table is retained read-only as historical
evidence until Wave 15J Phase 2 archives it.

B2.004. The shared registry crate at
`/Users/jasonlee/oyatie/registry/capability-tiers/` is retired. Wave
15J deletes the directory contents and removes the index from the
checkpoint registry.

B2.005. The seven files `bronze.json`, `silver.json`, `gold.json`,
`platinum.json`, `index.json`, `microservice-tier-mapping.yaml`, and
`vendor-tier-mapping.yaml` under
`/Users/jasonlee/oyatie/registry/capability-tiers/` are retired and
deleted in Wave 15J Phase 0.

B2.006. The 60 per-microservice `tier-matrix.md` files under
`/Users/jasonlee/oyatie/microservices/<service>/capability-tiers/tier-matrix.md`
are retired and deleted in Wave 15J Phase 0.

B2.007. The 61 per-microservice `capability-tiers/` directories are
retired and removed in Wave 15J Phase 0. After removal, the directory
shape is forbidden by the Wave 15J Phase 2 BLOCKER lane.

B2.008. The 16 `capability-tier-deltas-vs-counterparts-2026-05-20.md`
audit deliverables already on disk are retained as historical evidence
in their current form for a 6-month soak window per ADR-0138 Phase 1
discipline. After the soak window closes, Wave 15J Phase 2 archives
them under `microservices/<service>/_archive/2026-05-20-tier-deltas/`
so that the active directory is clean while the historical evidence
survives.

B2.009. The four-deliverable audit schema that produced the
`capability-tier-deltas-vs-counterparts-*.md` deliverable is retired. The
three-deliverable schema in ADR-0328 (coherence audit, feature parity,
performance benchmark numbers) is the canonical audit shape going
forward.

B2.010. ADR-0316 is marked `status: Superseded by ADR-0329` per
ADR-0108 sunset schema. The ADR file is retained as historical evidence
with a top-of-file Supersession Banner authored in Wave 15J Phase 0.

B2.011. The capability-tier authoring rules in
`/Users/jasonlee/oyatie/docs/standards/capability-authoring.md` are
retired in Wave 15J. The file is either deleted, or its contents are
absorbed into a new `tenant-class-authoring.md` standard owned by
ADR-0331. ADR-0331 owns the choice between delete and absorb.

B2.012. The per-tier autonomy ceiling clauses in
`/Users/jasonlee/oyatie/docs/standards/autonomy-ceiling.md` are
retired in Wave 15J. The autonomy ceiling is re-expressed as
tenant-class-aware where applicable; the underlying ceiling concept
survives because it is not tier-specific.

B2.013. The per-tier observability rituals in
`/Users/jasonlee/oyatie/docs/standards/observability.md` are retired
in Wave 15J. The retained observability contract is uniform at
industry-leader grade across both tenant classes per the substance-bar
doctrine in ADR-0322.

B2.014. The per-tier on-call escalation clauses in
`/Users/jasonlee/oyatie/docs/standards/on-call.md` are retired in
Wave 15J. The replacement is tenant-class-aware support gating
(demo_trial gets community/self-serve support; paid gets contractual
SLA-backed support per the tenant's contract).

B2.015. The per-tier workflow grants in
`/Users/jasonlee/oyatie/docs/standards/workflow-substrate-engine.md`
are retired in Wave 15J. Workflow grants are re-expressed against
tenant_class plus Cedar permit sets per ADR-0331's per-microservice
adoption IP.

B2.016. The per-tier event grants in
`/Users/jasonlee/oyatie/docs/standards/asyncapi-3-1-authoring.md` are
retired in Wave 15J. Event grants are re-expressed against tenant_class
plus channel-level Cedar permits.

B2.017. The per-tier proto contract grants in
`/Users/jasonlee/oyatie/docs/standards/proto3-authoring.md` are retired
in Wave 15J. Proto grants are re-expressed against tenant_class plus
RPC-level Cedar permits.

B2.018. The capability-tier projection clauses in
`/Users/jasonlee/oyatie/docs/standards/ontology-projection-substrate.md`
are retired in Wave 15J. Ontology projections are re-expressed against
tenant_class plus per-projection ADR-0257 schema-revision pins.

B2.019. The N-014 capability id form
`cap.<microservice>.<capability>.<tier>` from
`/Users/jasonlee/oyatie/docs/standards/naming-convention-bnf-v4.md` is
retired in Wave 15J. The replacement form is
`cap.<microservice>.<capability>` with no tier suffix.

B2.020. The N-015 capability-tier id form from the same standards file
is retired in Wave 15J. There is no replacement form because the
capability-tier id concept is being eliminated.

B2.021. The naming-checker scan of `registry/capability-tiers/*.yaml`
from BNF-SB-008 is retired in Wave 15J. The naming checker continues
to scan capability ids under the new N-014 form.

B2.022. The retired ADR-0316 enforcement lanes
`oya-governance-capability-tier-registry-shape`,
`oya-governance-no-product-fragmentation-microservices`,
`oya-governance-capability-tier-cedar-coverage`,
`oya-governance-capability-tier-ontology-projection-pin`,
`oya-governance-capability-tier-workflow-template-coverage`,
`oya-governance-capability-tier-ux-shell-coverage`,
`oya-governance-capability-tier-compliance-overlay-coverage`, and
`oya-governance-capability-tier-migration-declaration` are retired in
Wave 15J. The "no product fragmentation" anti-pattern that
ADR-0316 also asserted survives in ADR-0245 (substrate-vs-product
layering) and ADR-0132 (product platform and bundle dissolution); those
two ADRs already enforce the structural rule independently of the
capability-tier ladder.

B2.023. The Wave 15J retirement runs as a coordinated sub-wave under
ADR-0328 batch discipline. Phase 0 is atomic per ADR-0138 (single
change set per microservice does the file moves and deletions, mirror
of foundry six-path consolidation). Phase 1 is REPORT-ONLY soak (90
days from Phase 0 exit; shorter than the foundry six-month soak because
zero live tenants currently rely on the capability-tier surface).
Phase 2 is BLOCKER enforcement (the gates promote from REPORT-ONLY to
BLOCKER). Phase 3 is terminal (no further action).

B2.024. The Wave 15J Phase 0 change-set ceiling is one microservice per
change set. The total Wave 15J Phase 0 footprint is 61
microservice-level change sets, plus one registry-level change set, plus
one standards-doc scrub change set, plus one ADR-0316 supersession
change set, plus one ADR-0328 §D-19 reword change set. The total is 65
change sets in Wave 15J Phase 0.

B2.025. The Wave 15J Phase 0 change sets are landed under the
realignment wave's normal evidence pipeline (Foundry-pipeline +
multispectrum-review v2.4.0). No exception to evidence discipline is
granted.

B2.026. ADR-0328 §D-19 ("OCI Bronze = Always Free") is reworded in Wave
15J to read: "OCI Always Free profile is selected when
tenant_class IN ['demo_trial']; paid tenants on OCI use paid OCI
infrastructure but may still allocate Always-Free sandbox sub-tenancies
under their own paid tenancy umbrella for development workloads."

B2.027. The replacement model (ADR-0330) introduces the tenant_class
binary {demo_trial, paid} and the composable billing components
{revenue_share, per_seat, per_usage}. This ADR does not author that
content but binds the replacement reference so that downstream
microservice docs cite ADR-0330 as the canonical replacement.

B2.028. The per-microservice adoption mechanics (ADR-0331) author the
tenant_class claim binding in `cloud-iam` and `identity`, the
billing_components context attribute consumed by `cloud-billing`, the
Cedar policy gate templates that read tenant_class from the principal,
and the per-microservice plumbing IP that drives adoption to zero gaps
across the 77 microservices. This ADR does not author that content but
binds the adoption reference so that downstream microservice docs cite
ADR-0331 as the canonical adoption mechanic.

B2.029. The ADR-0316 super-broad claim that "a product name MUST NOT
create a microservice by itself" survives in ADR-0245 (substrate-vs-product
layering), ADR-0132 (product platform and bundle dissolution), and
ADR-0316's surviving Section D-10 test (which is migrated to ADR-0245
as Section D-10 of ADR-0245 in Wave 15J Phase 0).

B2.030. The ADR-0316 capability-tier composition table (Section D-6
"sales-cloud-class composed from workflow-engine, ontology, community,
marketplace, messenger, mail, intelligence, analytics" and the analogous
rows for service, marketing, HR, LMS, CLM, FP&A, ITSM) is preserved as
a description of substrate composition (which is true and useful) but
is reframed as "product surface composition" rather than
"capability-tier composition." The reframing is authored in Wave 15J
Phase 0 as part of the ADR-0316 supersession banner.

B2.031. The ADR-0316 jurisdiction-pack overlay rules (Section D-9) are
preserved because they describe a real interaction between Cedar packs
and capability projections. The rules are migrated to ADR-0251
(compliance-pack primitive) where they fit better. Wave 15J Phase 0
authors the migration as part of the supersession banner.

B2.032. The ADR-0316 entity-type Cedar schema fragment (Section D-1) is
retired. The replacement Cedar entity-type fragment is authored by
ADR-0330 and ADR-0331 against the tenant_class principal claim.

B2.033. The ADR-0316 Postgres schema fragments (Section D-7) are
retained read-only as historical evidence in the migration ledger. No
new rows are written. After Wave 15J Phase 2, the tables are dropped
under a follow-up migration authored by ADR-0331.

B2.034. The ADR-0316 Appendix sections (S "cross-jurisdiction
interaction ledger," T "new-service exception examples," U "reviewer
closeout checklist") are preserved as historical reference material in
the superseded ADR file. They are not re-authored against the
tenant-class model in this ADR because their content survives in
neighbouring ADRs: jurisdiction interactions in ADR-0251 packs, new
service exceptions in ADR-0245 substrate-vs-product gates, reviewer
closeout in ADR-0322 substance-bar doctrine.

B2.035. The retirement is irreversible. No rollback path exists. Once
ADR-0316 is marked superseded and Wave 15J Phase 0 lands, the
capability-tier ladder is gone. Section F documents this.

B2.036. The retirement preserves ADR-0248 cellular criticality tier
vocabulary in full. The Wave 15J enforcement lane has an allow-list
that explicitly permits the strings "Tier 0," "Tier 1," "Tier 2,"
"Tier 3," "Tier 4," "cell-tier-1," "cell-tier-2," "cell-tier-3,"
"cell-tier-4-financial-grade," "cell-tier-4-fulfillment-grade,"
"cell-tier-4-il5," "dr_tier: T1," "dr_tier: T2," and analogous cellular
classifications.

B2.037. The retirement preserves ADR-0037 public-API stability tier
vocabulary in full. The lane permits "preview," "stable," "GA," and the
deprecation lifecycle vocabulary.

B2.038. The retirement preserves ADR-0083 Rust error-handling tier
vocabulary in full. The lane permits "Tier 1 library," "Tier 2 binary,"
and "no-unwrap" disciplines.

B2.039. The retirement preserves any other "tier" usage that is not the
capability-tier ladder. Section D-1.7 enumerates the surviving
vocabularies completely.

B2.040. The retirement does not affect Cedar's underlying policy
engine. Cedar continues to evaluate permits and forbids over principals,
actions, and resources. What changes is the input shape: tenant_class
enters the principal claim, capability-tier exits the principal claim.

B2.041. The retirement does not affect the workflow-engine's underlying
execution model. Workflow templates continue to load and execute. What
changes is the binding shape: workflow grants are gated by tenant_class
and Cedar permits, not by capability-tier id.

B2.042. The retirement does not affect the ontology projection
mechanism. Ontology projections continue to surface object types and
relation types. What changes is the binding shape: projection
manifests are pinned to ADR-0257 schema revisions and gated by
tenant_class plus Cedar permits, not by capability-tier id.

B2.043. The retirement does not affect the UX shell mechanism. UX
shells continue to render navigation, widgets, and dashboards. What
changes is the binding shape: shell selection is per-product-surface
plus per-tenant-class, not per-capability-tier.

B2.044. The retirement does not affect the compliance-pack overlay
mechanism. Compliance packs continue to bind to data class,
jurisdiction, residency cell, and regulated-decision rules. What
changes is the activation gate: compliance packs are activated for
tenant_class = paid only (demo_trial cannot activate compliance packs
because demo_trial does not host regulated workloads), and pack
selection is per-tenant per ADR-0251.

B2.045. The retirement does not affect observability cardinality
budgets. The audit-chain, metric, trace, and log dimensions continue
to include tenant_id and home_cell_id. What changes is the
removal of capability_tier_id from those dimensions. Replacement
dimensions include tenant_class and (optionally) billing_components for
FinOps allocation.

B2.046. The retirement does not affect cost-attribution mechanics. The
finops-portal continues to attribute cost by tenant_id, home_cell_id,
microservice, workflow, ontology projection, mail send, and analytics
query. What changes is the removal of capability_tier_id from cost
dimensions and the addition of tenant_class plus billing_components.

B2.047. The retirement does not affect marketplace mechanics. The
marketplace continues to offer plugins, apps, workflows, agents,
models, and datasets. What changes is the removal of capability-tier
gating on marketplace offers. Marketplace eligibility is tenant_class
gated (paid tenants only for purchases) per ADR-0249, and per-offer
Cedar permits otherwise.

B2.048. The retirement does not affect the substance-bar doctrine in
ADR-0322. Substance is a blocker-class requirement uniformly. The
substance bar does not vary by tenant_class. Demo_trial tenants get
the same substance-bar quality as paid tenants.

B2.049. The retirement does not affect the "industry-leader quality
bar" doctrine in `feedback_quality_performance_scalability_bar.md`.
Quality is uniform across tenant_class. The only differences between
demo_trial and paid are usage caps, time gating, support class, SLO
gating, and compliance-pack eligibility (per ADR-0330).

B2.050. The retirement does not affect ADR-0132 (product platform and
bundle dissolution). The "no new suite or bundle microservice" rule
survives independently.

B2.051. The retirement does not affect ADR-0245 (substrate vs product
layering). The substrate-vs-product separation survives independently.

B2.052. The retirement does not affect ADR-0249 (multi-category
marketplace). The six-category marketplace (plugins, apps, workflows,
agents, models, datasets) survives independently.

B2.053. The retirement does not affect ADR-0251 (compliance-pack
primitive). Packs continue to activate per tenant. Activation is
tenant_class gated (paid only).

B2.054. The retirement does not affect ADR-0255 (BYOK opt-in). BYOK
continues to allow tenants to supply their own LLM credentials.
BYOK activation is tenant_class gated (paid only).

B2.055. The retirement does not affect ADR-0244 (tenant as universal
scoping primitive). Tenant scoping survives; tenant_class extends the
tenant principal claim.

B2.056. The retirement does not affect ADR-0243 (Cedar as universal
gate). Cedar remains the universal authorization mechanism. The
tenant_class attribute enters Cedar's principal context.

B2.057. The retirement does not affect ADR-0263 (audit emission). Audit
events continue to emit with tenant_id, home_cell_id, action, and
resource. The capability_tier_id field is removed from the audit
schema in Wave 15J Phase 0.

B2.058. The retirement does not affect the canonical-base + localization
doctrine in ADR-0064. Localization activation is per-pack (KR pack,
EU pack, etc.) and per-tenant_class where applicable; tier-gated
localization clauses are removed.

B2.059. The retirement does not affect the autonomous-decision
principles. Long-term-right-over-short-term-cost continues to apply.
Hyperscaler-thinking applies. Linus-grade criticism applies.

B2.060. The retirement does not authorize any scripted bulk-rewrite of
substantive content. Wave 15J authoring is bespoke per microservice
per ADR-0322. The only mechanically applied changes are the file
deletions and the cross-reference scrubs (where the scrub is a literal
replacement, not authoring).

B2.061. The retirement authorises one Rust crate for cross-microservice
tier-vocabulary detection. The crate is named `oyatie-tier-retirement`.
It is a detection-and-suggestion tool, not a bulk-rewrite tool. Per
B2.060, it suggests replacements but does not author them.

B2.062. The retirement does not authorize any new microservice. The
77-microservice roster does not grow by this decision.

B2.063. The retirement does not authorize any new product surface.
Every product surface remains a composition of substrate microservices
gated by tenant_class plus Cedar permits.

B2.064. The retirement authorises the new CI lane
`oya-governance-tier-vocabulary-zero-residue` (Section E lists the
enforcement lanes). The lane runs as REPORT-ONLY during Wave 15J Phase
1 and as BLOCKER after Wave 15J Phase 2.

B2.065. The retirement authorises the new CI lane
`oya-governance-tenant-class-claim-binding` to verify that every
microservice that issues principals binds the tenant_class claim. The
lane runs as REPORT-ONLY during Wave 15J Phase 1 and as BLOCKER after
ADR-0331's per-microservice adoption IP lands.

B2.066. The retirement authorises the new CI lane
`oya-governance-no-capability-tier-grants` to verify that the retired
Postgres tables `tenant_capability_tier_grants` and
`microservice_capability_tier_contributions` receive no new writes.
The lane runs as REPORT-ONLY during Wave 15J Phase 1 and as BLOCKER
after Wave 15J Phase 2.

B2.067. The retirement authorises the new CI lane
`oya-governance-bronze-silver-gold-platinum-zero-occurrence` to scan
the corpus for literal Bronze/Silver/Gold/Platinum tokens, with an
allow-list for ADR-0316 (the superseded ADR file), this ADR
(ADR-0329), the wave findings aggregation, the realignment review, the
retired capability-tier-deltas-vs-counterparts deliverables (during
soak), and any explicit historical references that the supersession
banner declares allowed.

B2.068. The retirement authorises the new CI lane
`oya-governance-registry-capability-tiers-deletion` to verify that the
`/Users/jasonlee/oyatie/registry/capability-tiers/` directory is empty
after Wave 15J Phase 0 lands. The lane runs as BLOCKER from Phase 0
exit onward.

B2.069. The retirement authorises the new CI lane
`oya-governance-naming-bnf-n014-n015-amendment` to verify that the
naming-convention BNF v4 N-014 and N-015 rules drop the
`.<tier>` segment after Wave 15J Phase 0. The lane runs as BLOCKER from
Phase 0 exit onward.

B2.070. The retirement authorises the new CI lane
`oya-governance-adr-0316-supersession-marker` to verify that ADR-0316
carries the `status: Superseded by ADR-0329` frontmatter and the
supersession banner after Wave 15J Phase 0 lands. The lane runs as
BLOCKER from Phase 0 exit onward.

B2.071. The retirement authorises the new CI lane
`oya-governance-cellular-criticality-tier-preservation` to verify that
the ADR-0248 cellular criticality vocabulary (Tier 0..Tier 4, cell-tier-
filenames, dr_tier values) remains intact across the corpus. The lane
runs as BLOCKER continuously, even before Wave 15J Phase 0, because
the preservation guarantee is immediate.

B2.072. The retirement does not block any microservice's own retirement
or consolidation work. ADR-0138 foundry six-path consolidation,
ADR-0136 foundry topology, and other microservice-level
reorganisations are independent of this decision.

B2.073. The retirement does not block any microservice's spec authoring
sprint. ADR-0328 batch discipline already governs spec-authoring
sprints (cloud-billing in Wave 15B per the realignment review). Those
sprints proceed.

B2.074. The retirement does not block the Big-8 expansion. The crm
rewrite (Wave 15A), HR family elevation, ERP family expansion, ITSM
family elevation, marketing-automation family elevation, learning-
management family elevation, financial-planning family elevation, and
contract-lifecycle family elevation all proceed under their own
sub-waves. Each sub-wave inherits the tenant-class model from ADR-0330
and ADR-0331 instead of authoring against the retired capability-tier
ladder.

B2.075. The retirement does not authorize any waiver. No "exception"
clause exists in this ADR. If a microservice's PRD or IP claims a
capability-tier dependency, that PRD or IP is rewritten in Wave 15J
under the tenant-class model. If a vendor contract or marketing
collateral references Bronze/Silver/Gold/Platinum, that contract or
collateral is rewritten in Wave 15J under the tenant-class model.

B2.076. The retirement does not require a vote, a council session, or
a multispectrum-review escalation. The user directive of 2026-05-20
("we don't have tiers") is the authoritative signal. The
multispectrum-review v2.4.0 lane evaluates this ADR's own substance
bar (per ADR-0322 and ADR-0328) but does not re-litigate the user
directive.

B2.077. The retirement does not require pre-deletion archival of the
retired capability-tier registry, because the registry is content-
addressed and stored in git history. After Wave 15J Phase 0, the
registry is recoverable from git history if needed for audit purposes.

B2.078. The retirement does not affect the existing
`evidence/multispectrum-review/` evidence rooted in capability-tier
audits. That evidence is dated and historical; it is preserved as-is.

B2.079. The retirement does not affect existing audit-chain evidence
that contains capability_tier_id fields. The field is deprecated but
not deleted from historical events. New events do not emit the field.

B2.080. The retirement does not affect any third-party integration that
parses capability_tier_id from Oyatie API responses. No third-party
integration exists today because Oyatie is pre-launch. If any external
integration is added during the soak window, it integrates against the
tenant-class model (ADR-0330) directly.

B2.081. The retirement is sequenced ahead of Wave 15A (crm rewrite) so
that crm rewrite authors against the tenant-class model directly. The
realignment review notes that crm has the worst tier-vocabulary
entrenchment of any microservice; rewriting crm under the retired
tier model would be wasted authoring effort.

B2.082. The retirement is sequenced ahead of Wave 15B (cloud-billing
spec sprint) so that cloud-billing's spec sprint models tenant_class
plus billing_components directly. cloud-billing is the source-of-truth
microservice for both primitives; its spec sprint must reflect the
post-retirement doctrine.

B2.083. The retirement is sequenced before Wave 15K (network →
community merge) so that the merged community service inherits the
tenant-class model directly without any capability-tier residue from
either source service.

B2.084. The retirement is sequenced before Wave 15L (cell retirement)
so that the absorbing services (tenancy, cloud-iac, observability,
oyatie-shuffle-sharding crate, api-gateway, audit-chain) absorb the
cellular criticality vocabulary cleanly without confusion against
capability-tier vocabulary.

B2.085. The retirement explicitly authorises one historical reference
in this ADR's body to the names Bronze, Silver, Gold, and Platinum so
that future engineers can search the corpus for the retired vocabulary
and find this retirement decision. Without that authorised mention,
the zero-residue lane would flag this ADR.

B2.086. The retirement authorises the supersession banner at the top
of ADR-0316 to mention Bronze, Silver, Gold, and Platinum exactly once
each for the same reason as B2.085.

B2.087. The retirement is binding on every contributor (human and
agent) immediately upon Acceptance. There is no transition window for
authoring new tier-vocabulary content. Any pull request submitted
after Acceptance that introduces Bronze/Silver/Gold/Platinum vocabulary
as capability tiers is rejected by the report-only lane (during Phase
1) or blocked by the BLOCKER lane (during Phase 2).

B2.088. The retirement is announced in the wave findings aggregation,
the realignment review, and the next ADR-0327 promotion gate report.
No additional announcement channel is required because Oyatie's
project-state surfaces are the canonical announcement medium.

B2.089. The retirement does not consume any external compute budget or
external services beyond what ADR-0328 already authorises for the
realignment wave. Wave 15J runs under the same Codex-only dispatch
ceiling as the rest of the realignment wave.

B2.090. The retirement does not require any new role or org chart
change. Council-architecture owns the supersession banner; axis-foundry
owns the lane authoring; ops-sre-reliability owns the soak window
monitoring. All three roles already exist.

B2.091. The retirement clears the way for ADR-0330 and ADR-0331 to
land. ADR-0330 authors the replacement model; ADR-0331 authors the
per-microservice adoption IP. Both ADRs assume this ADR has landed
first.

B2.092. The retirement also clears the way for the Wave 14 final
aggregation. The aggregation cannot polish into a canonical deliverable
while ADR-0316 is still authoritative. After Acceptance, the
aggregation can pin its remediation backlog to ADR-0329, ADR-0330, and
ADR-0331 as the canonical retirement-plus-replacement triple.

B2.093. The retirement allows the Wave 15J sub-wave to be scoped as a
single coordinated batch under ADR-0328 batch discipline, not as 61
independent microservice patches. The batch ceiling is one
microservice per change set, but the batch sequence runs under one
coordinated plan.

B2.094. The retirement does not affect the master-plan-sequencing.json
file. That file already lists the Wave 15J sub-wave as
"tier-system-retirement." After Acceptance, master-plan-sequencing.json
is amended to point to ADR-0329, ADR-0330, and ADR-0331 instead of
"pending."

B2.095. The retirement does not affect the brief-template.md file's
five-citation header. The header continues to require five canonical
citations; the citations include this ADR for any Wave 15J-era brief.

B2.096. The retirement does not affect the ADR-0145 inter-microservice
communication reform. Direct gRPC plus three invariants survives
independently. The capability-tier ladder was orthogonal to the
communication reform.

B2.097. The retirement does not affect ADR-0247 (foundry self-modification
doctrine). The Foundry continues to run as oyatie.foundry.* principals
under Cedar. Foundry's principals carry tenant_class claims like every
other principal.

B2.098. The retirement does not affect ADR-0253 (HTTP/3 + QUIC default
protocol). The transport protocol is orthogonal to tier vocabulary.

B2.099. The retirement does not affect ADR-0252 (HLC default,
TrueTime tier). The HLC-vs-TrueTime distinction is a clock-discipline
ladder, not a capability-tier ladder. The word "tier" appears in
ADR-0252's title but is preserved per B2.039.

B2.100. The retirement is final on Acceptance. No further capability-
tier doctrine authoring is sanctioned in any Oyatie surface.

## Consequences

### C.1 Maintainability

C.1.1 Per-microservice authoring debt drops by one entire deliverable
(the `capability-tier-deltas-vs-counterparts-*.md` file). The
three-deliverable schema in ADR-0328 is the canonical going-forward
shape.

C.1.2 The 60 `tier-matrix.md` artifacts disappear. Future microservice
authoring does not need to fill a tier matrix. New microservice
authoring takes 1-2 fewer artifact slots per microservice as a result.

C.1.3 The standards docs lose 17 cross-references to ADR-0316 and the
clauses that depend on it. Standards authoring becomes simpler because
the per-tier rituals collapse into one uniform contract.

C.1.4 The Cedar policy authoring becomes simpler because the principal
context drops one attribute (capability_tier_id) and adds one
attribute (tenant_class). The net is approximately the same number of
attributes but the new attribute is simpler (binary enum vs four-tier
enum).

C.1.5 The Ontology projection manifests lose one pin field
(capability_tier_id) and gain none. Projection authoring becomes
slightly simpler.

C.1.6 The Workflow template library bindings lose one binding field
(capability_tier_id). Template binding becomes slightly simpler.

C.1.7 The UX shell manifests lose one selector field (capability_tier_id).
Shell selection becomes slightly simpler.

C.1.8 The compliance-pack overlays lose one activation field
(capability_tier_id) and gain one (tenant_class with paid-only gate).
Net authoring complexity is approximately the same.

C.1.9 The observability dimensions lose one cardinality dimension
(capability_tier_id, four values) and gain one (tenant_class, two
values). Net cardinality drops by approximately half on this axis.

C.1.10 Cost attribution becomes simpler because cost dimensions drop
the capability-tier axis and gain the tenant_class plus
billing_components axes. The billing_components axis is more
informative than capability_tier_id because billing_components describes
actual money flow (revenue_share, per_seat, per_usage), while
capability_tier_id was an abstract label.

### C.2 Observability

C.2.1 Cardinality drops on the capability_tier_id axis (four values)
and rises on the tenant_class axis (two values). The net cardinality
budget improves.

C.2.2 The audit-chain schema drops capability_tier_id from new event
emissions. Old events retain the field as historical evidence.

C.2.3 Trace spans drop the capability_tier_id annotation. New traces
emit tenant_class instead.

C.2.4 Log schemas drop the capability_tier_id field. New logs include
tenant_class.

C.2.5 Metrics drop the capability_tier_id label. New metrics include
tenant_class.

C.2.6 Dashboards that filter by capability_tier_id are rewritten in
Wave 15J Phase 0 to filter by tenant_class. The dashboard rewrite is
mechanical; no substantive content change is needed.

C.2.7 SLO definitions that were tier-segmented (per-tier p95 latency
targets) collapse into uniform substance-bar SLO targets per ADR-0328.

### C.3 Scalability

C.3.1 Shared substrates continue to scale on their actual bottlenecks.
ADR-0316's claim that "shared substrates scale on their actual
bottlenecks, while tier grants control tenant-visible activation"
survives in spirit; the activation mechanism is tenant_class plus
Cedar permits, not capability-tier grants.

C.3.2 The capability-tier grant registry stops scaling because it is
retired. The capacity that the registry was projected to consume
(tenant grant rows, microservice contribution rows) is reclaimed.

C.3.3 The tenant_class plus billing_components state lives in
cloud-billing. cloud-billing's capacity model already accounts for
tenant state.

### C.4 Performance

C.4.1 The Cedar authorization path drops one context attribute lookup
(capability_tier_id resolution from the grant registry). The
authorization path adds one context attribute lookup (tenant_class
resolution from the tenant principal). Net authorization latency is
approximately unchanged.

C.4.2 The Ontology projection path drops the capability-tier
projection-pin resolution. The projection path adds tenant_class
gating. Net projection latency is approximately unchanged.

C.4.3 The Workflow template selection path drops the capability-tier
library lookup. The selection path adds tenant_class gating. Net
selection latency is approximately unchanged.

C.4.4 The UX shell selection path drops the capability-tier manifest
lookup. The selection path adds tenant_class gating. Net rendering
latency is approximately unchanged.

C.4.5 ADR-0316 §C.4 set a p95 budget of 250 ms for capability-tier
permit evaluation plus projection lookup on warm cache. The replacement
budget is the same 250 ms for tenant_class permit evaluation plus
projection lookup on warm cache. The budget moves to ADR-0330.

### C.5 Optimisation

C.5.1 Cost attribution per tenant_class plus billing_components is
more useful for FinOps than cost attribution per capability_tier. The
new attribution can drive actual business decisions (which billing
component is most profitable, which tenant_class has highest cost-per-
tenant, etc.).

C.5.2 P&L visibility per product surface stays available because
product surfaces are composed from substrate microservices and
attribution is per microservice (not per tier).

C.5.3 The FinOps portal dashboards drop the capability-tier axis and
gain the tenant_class plus billing_components axes. The dashboard
rewrite is mechanical.

### C.6 Code quality

C.6.1 The shared registry crate at `registry/capability-tiers/` is
retired. The Rust crate(s) that read from it are retired or rewritten
to read from `registry/tenant-classes/` (authored by ADR-0331).

C.6.2 The property tests for capability-tier grant lifecycle are
retired. New property tests for tenant_class state transitions are
authored by ADR-0330 and ADR-0331.

C.6.3 The migration tests for capability-tier grant tables are retired
once Wave 15J Phase 2 drops the tables. New migration tests for
tenant_class tables are authored by ADR-0331.

C.6.4 The Cedar fixture tests for capability-tier permits are retired.
New Cedar fixture tests for tenant_class permits are authored by
ADR-0331.

C.6.5 The manifest lint for capability-tier manifests is retired. New
manifest lint for tenant_class manifests is authored by ADR-0331.

C.6.6 The dependency-seam check for capability-tier registry imports
is retired. The check is replaced by the tenant-classes registry
import check authored by ADR-0331.

### C.7 Risks and mitigations

C.7.1 Risk: a microservice's PRD claims a capability-tier dependency
that has not been migrated. Mitigation: Wave 15J Phase 1 REPORT-ONLY
lane surfaces the dependency for triage before Phase 2 BLOCKER
enforcement.

C.7.2 Risk: a third-party document (vendor contract, marketing
collateral, partner integration spec) references Bronze/Silver/Gold/
Platinum. Mitigation: Wave 15J Phase 0 scrubs the in-repo references;
external references are handled by separate communication channels
that are not in scope for this ADR.

C.7.3 Risk: an in-flight Wave 3 audit completes after Acceptance and
authors against the retired tier model. Mitigation: the Wave 4-rolling
orchestrator already pivoted to the three-deliverable schema; any
completed-but-not-yet-aggregated audit is rolled into the wave
findings aggregation as historical evidence and not back-corrected.

C.7.4 Risk: a future contributor (human or agent) reads ADR-0316
without noticing the supersession banner and authors against the
retired model. Mitigation: ADR-0316's frontmatter carries
`status: Superseded by ADR-0329`, the lane refuses any new
capability-tier-vocabulary authoring, and the supersession banner is
the first content block in the file.

C.7.5 Risk: the Wave 15J scrub deletes a file that should have been
preserved. Mitigation: the scrub is per-microservice change-set; each
change set goes through multispectrum-review v2.4.0; preserve-vs-delete
is decided per artifact under reviewer evidence.

C.7.6 Risk: the Wave 15J scrub introduces a regression in Cedar policy
or Ontology projection. Mitigation: each change set is gated by the
existing CI lanes for Cedar coverage, Ontology projection coverage,
and dependency seam.

C.7.7 Risk: the soak window (90 days) is too short. Mitigation: the
soak window is REPORT-ONLY, not silent; any non-zero residue triggers
investigation. If residue is non-zero at day 90, the soak extends until
residue reaches zero before Phase 2 promotion.

C.7.8 Risk: the registry deletion leaves dangling pointers in other
microservices' configs. Mitigation: the cross-reference scrub in
Wave 15J Phase 0 removes every pointer before the registry directory
is deleted. The dependency-seam lane refuses any post-deletion pointer.

C.7.9 Risk: ADR-0330 or ADR-0331 takes longer than expected to land.
Mitigation: this ADR is independently authoritative. Even without
ADR-0330 and ADR-0331, the retirement is complete and capability-
tier authoring is prohibited. The replacement model lands when ADR-0330
and ADR-0331 land; no urgent dependency exists.

C.7.10 Risk: the cellular criticality preservation lane misfires and
deletes ADR-0248 vocabulary. Mitigation: the lane has explicit
allow-list rules, not exclude-list rules. The default is BLOCK on
unknown tier vocabulary; the allow-list is the only path to PASS.
ADR-0248's vocabulary is on the allow-list by Section D-1.7.

## D. Implementation footprint

### D-1 Surviving tier vocabularies (allow-list)

D-1.1 ADR-0248 cellular criticality vocabulary survives in full. The
words "Tier 0," "Tier 1," "Tier 2," "Tier 3," and "Tier 4" used as
cellular criticality classifications are preserved. The filename
fragments `cell-tier-1`, `cell-tier-2`, `cell-tier-3`,
`cell-tier-4-financial-grade`, `cell-tier-4-fulfillment-grade`,
`cell-tier-4-il5` are preserved. The fields `dr_tier: T1` and
`dr_tier: T2` are preserved.

D-1.2 ADR-0037 public-API stability tier vocabulary survives in full.
The words "preview," "stable," "GA" are preserved. The 6/12-month
deprecation discipline is preserved.

D-1.3 ADR-0083 Rust error-handling tier vocabulary survives in full.
The words "Tier 1 library" and "Tier 2 binary" are preserved. The
"no-unwrap" discipline is preserved.

D-1.4 ADR-0252 HLC vs TrueTime tier vocabulary survives in full. The
clock-discipline ladder is unchanged.

D-1.5 ADR-0250 build-ahead-of-certification compliance tier vocabulary
survives in full. Certification readiness ladders are not capability
tiers.

D-1.6 Generic English uses of "tier" survive (for example, "the top
tier of vendors," "a higher tier of complexity"). The lane refuses
only the specific capability-tier ladder vocabulary
(Bronze/Silver/Gold/Platinum tokens used as capability-tier labels;
the literal phrase "capability tier" used as a primitive name;
references to ADR-0316 as a live authority).

D-1.7 The lane allow-list, expressed as positive patterns:

| Pattern | Provenance | Preserved? |
|---|---|---|
| `Tier 0`, `Tier 1`, `Tier 2`, `Tier 3`, `Tier 4` (cellular) | ADR-0248 | Yes |
| `cell-tier-1`, `cell-tier-2`, `cell-tier-3`, `cell-tier-4-*` | ADR-0248 | Yes |
| `dr_tier: T1`, `dr_tier: T2` | ADR-0248 | Yes |
| `preview`, `stable`, `GA` (API stability) | ADR-0037 | Yes |
| `Tier 1 library`, `Tier 2 binary` (Rust error handling) | ADR-0083 | Yes |
| `HLC tier`, `TrueTime tier` (clock discipline) | ADR-0252 | Yes |
| Generic English `tier` | n/a | Yes when not labelled "capability tier" |
| `Bronze`, `Silver`, `Gold`, `Platinum` (capability tier) | ADR-0316 | NO — retired |
| `capability tier` (primitive) | ADR-0316 | NO — retired |
| `capability-tier-grant` | ADR-0316 | NO — retired |
| `capability_tier_id` | ADR-0316 | NO — retired |
| `tenant_capability_tier_grants` | ADR-0316 | NO — retired |
| `microservice_capability_tier_contributions` | ADR-0316 | NO — retired |
| References to ADR-0316 as live authority | ADR-0316 | NO — supersession-banner only |

### D-2 Per-microservice retirement tasks

Each affected microservice receives one change set in Wave 15J Phase 0.
The change set contains:

D-2.1 Delete `microservices/<service>/capability-tiers/tier-matrix.md`.

D-2.2 Delete the `microservices/<service>/capability-tiers/` directory.

D-2.3 Scrub the microservice's PRD for capability-tier vocabulary.
Replace "capability tier" mentions with "tenant-class capability" or
delete entirely (per reviewer evidence).

D-2.4 Scrub the microservice's ARCHITECTURE document for capability-
tier vocabulary.

D-2.5 Scrub the microservice's README for capability-tier vocabulary.

D-2.6 Scrub the microservice's IP slices for capability-tier vocabulary.

D-2.7 Scrub the microservice's runbooks for capability-tier vocabulary.

D-2.8 Scrub the microservice's per-microservice ADRs for capability-tier
vocabulary.

D-2.9 Scrub the microservice's Cedar policies for `capability_tier_id`
attribute references. Replace with tenant_class attribute per
ADR-0330's Cedar schema.

D-2.10 Scrub the microservice's Ontology projection manifests for
capability-tier projection pins.

D-2.11 Scrub the microservice's Workflow template bindings for
capability-tier library references.

D-2.12 Scrub the microservice's UX shell manifests for capability-tier
selector references.

D-2.13 Scrub the microservice's compliance-pack overlays for
capability-tier activation references.

D-2.14 Scrub the microservice's observability profile for
capability_tier_id dimensions.

D-2.15 Scrub the microservice's cost profile for capability_tier_id
dimensions. Replace with tenant_class plus billing_components per
ADR-0330.

D-2.16 Scrub the microservice's CI lane definitions for any
capability-tier-named lane references.

D-2.17 Scrub the microservice's catalog YAMLs for capability-tier
contribution references.

D-2.18 Retain the microservice's `capability-tier-deltas-vs-counterparts-2026-05-20.md`
deliverable in place (do not delete; archive after soak).

D-2.19 Author the change set's evidence record citing this ADR
(ADR-0329) and the relevant subsection of ADR-0331 (per-microservice
adoption IP). The evidence record is the per-microservice retirement
ledger.

D-2.20 Verify that the change set passes the existing CI lanes plus
the new lanes from B2.064-B2.071.

### D-3 Standards-doc retirement tasks

D-3.1 `docs/standards/capability-tier-matrix.md` is retired in Wave
15J Phase 0. The file is either deleted or moved to
`docs/standards/_archive/capability-tier-matrix-2026-05-20.md` for
historical evidence. ADR-0331 chooses delete-vs-archive.

D-3.2 `docs/standards/capability-authoring.md` is retired in Wave 15J
Phase 0. The substance-bar authoring discipline survives in
`docs/standards/documentation-rigor.md`. The capability-specific
content is either deleted or absorbed into a new
`docs/standards/tenant-class-authoring.md` authored by ADR-0331.

D-3.3 `docs/standards/autonomy-ceiling.md` is amended in Wave 15J Phase
0 to remove per-tier autonomy ceiling rows. The autonomy ceiling
concept survives without per-tier stratification.

D-3.4 `docs/standards/brief-template.md` is amended in Wave 15J Phase
0 to remove the per-tier anchor template. The five-citation header is
preserved.

D-3.5 `docs/standards/hyperscaler-best-practices.md` is amended in
Wave 15J Phase 0 to remove the "capability-tier (T1-T4) autonomy
ceiling" mention. The autonomy-ceiling sentence is rewritten to cite
the (now non-tier-stratified) autonomy-ceiling concept.

D-3.6 `docs/standards/observability.md` is amended in Wave 15J Phase 0
to remove per-tier observability ritual rows. The uniform substance-
bar observability contract survives.

D-3.7 `docs/standards/on-call.md` is amended in Wave 15J Phase 0 to
remove per-tier escalation rows. The tenant-class-aware support gating
replaces per-tier escalation.

D-3.8 `docs/standards/workflow-substrate-engine.md` is amended in
Wave 15J Phase 0 to remove "workflow.capability-tier-grant.v1" event
class. The replacement event class is owned by ADR-0331.

D-3.9 `docs/standards/asyncapi-3-1-authoring.md` is amended in Wave
15J Phase 0 to remove per-tier event grant rows.

D-3.10 `docs/standards/proto3-authoring.md` is amended in Wave 15J
Phase 0 to remove per-tier proto contract grant rows.

D-3.11 `docs/standards/ontology-projection-substrate.md` is amended in
Wave 15J Phase 0 to remove the capability-tier projection pin clauses.

D-3.12 `docs/standards/naming-convention-bnf-v4.md` N-014 and N-015
rules are amended in Wave 15J Phase 0 to drop the `.<tier>` segment.
The naming-checker scan of `registry/capability-tiers/*.yaml` is also
removed. BNF-SB-008 is amended accordingly.

D-3.13 `docs/standards/tracing.md` is amended in Wave 15J Phase 0 to
remove any per-tier trace span annotations.

D-3.14 `docs/standards/slos.md` is amended in Wave 15J Phase 0 to
remove any per-tier SLO target rows. The uniform substance-bar SLO
contract survives.

D-3.15 `docs/standards/documentation-rigor.md` is amended in Wave 15J
Phase 0 to remove any per-tier documentation-rigor row.

D-3.16 `docs/standards/mlops.md` is amended in Wave 15J Phase 0 to
remove any per-tier MLOps row.

D-3.17 `docs/standards/documentation-index.md` is amended in Wave 15J
Phase 0 to remove the index entry for `capability-tier-matrix.md` and
to add an index entry for the new `tenant-class-authoring.md` (if
ADR-0331 chooses absorb over delete).

### D-4 Registry retirement tasks

D-4.1 `registry/capability-tiers/bronze.json` is deleted in Wave 15J
Phase 0.

D-4.2 `registry/capability-tiers/silver.json` is deleted in Wave 15J
Phase 0.

D-4.3 `registry/capability-tiers/gold.json` is deleted in Wave 15J
Phase 0.

D-4.4 `registry/capability-tiers/platinum.json` is deleted in Wave 15J
Phase 0.

D-4.5 `registry/capability-tiers/index.json` is deleted in Wave 15J
Phase 0.

D-4.6 `registry/capability-tiers/checkpoint.json` is deleted in Wave
15J Phase 0.

D-4.7 `registry/capability-tiers/microservice-tier-mapping.yaml` is
deleted in Wave 15J Phase 0.

D-4.8 `registry/capability-tiers/vendor-tier-mapping.yaml` is deleted
in Wave 15J Phase 0.

D-4.9 The `registry/capability-tiers/` directory itself is removed in
Wave 15J Phase 0 after the contents are deleted.

D-4.10 ADR-0331 authors a `registry/tenant-classes/` directory with
`demo-trial.json` and `paid.json` entries plus an `index.json` and a
`checkpoint.json`. The new directory is independent of the old
capability-tier registry; it does not inherit any rows.

### D-5 ADR-0316 supersession tasks

D-5.1 ADR-0316's frontmatter `status:` field is set to
`Superseded by ADR-0329` in Wave 15J Phase 0.

D-5.2 ADR-0316's frontmatter `superseded_by:` field is set to
`[ADR-0329]` in Wave 15J Phase 0.

D-5.3 ADR-0316's top-of-file content gets a supersession banner that
reads (paraphrase): "This ADR is superseded by ADR-0329 (tier-system-
retired-replaced-by-tenant-class) on 2026-05-21. The Bronze/Silver/Gold/
Platinum capability-tier ladder described below is retired. The
replacement model is the tenant-class binary (demo_trial, paid) with
composable billing components, codified by ADR-0330. Per-microservice
adoption is authored by ADR-0331. The content below is preserved as
historical evidence." The banner is authored in Wave 15J Phase 0.

D-5.4 ADR-0316 retains its full content body unchanged after the
banner. The decision-clauses, appendix sections, and reference tables
remain as historical evidence.

D-5.5 ADR-0316's `sunset_at` field is set to the Wave 15J Phase 0
landing date (the actual ISO date at change-set merge time).

D-5.6 ADR-0316's `sunset_milestone` field is set to `WAVE-15J-PHASE-0`
per the master-plan-sequencing.json convention.

D-5.7 ADR-0316's `sunset_topic` field is set to
`adr-0316-capability-tier-retirement`.

### D-6 ADR-0328 §D-19 reword task

D-6.1 ADR-0328 §D-19 contains a clause approximately rendered as: "OCI
deployment profile Bronze = Always Free." This clause is reworded in
Wave 15J Phase 0 to: "OCI Always Free profile is selected when
tenant_class IN ['demo_trial']; paid tenants on OCI use paid OCI
infrastructure but may still allocate Always-Free sandbox sub-tenancies
under their own paid tenancy umbrella for development workloads. The
selection respects ADR-0330's tenant-class model and ADR-0331's
per-microservice adoption IP."

D-6.2 ADR-0328's frontmatter `related_adrs:` array is amended in Wave
15J Phase 0 to add ADR-0329, ADR-0330, and ADR-0331.

### D-7 Naming-BNF amendment task

D-7.1 Naming-convention-BNF-v4.md N-014 is amended in Wave 15J Phase 0
to drop the `.<tier>` segment. The new form is
`cap.<microservice>.<capability>`.

D-7.2 Naming-convention-BNF-v4.md N-015 is retired in Wave 15J Phase 0
because the capability-tier id concept is gone.

D-7.3 Naming-convention-BNF-v4.md BNF-SB-008 is amended to drop the
`registry/capability-tiers/*.yaml` scan obligation.

D-7.4 Naming-convention-BNF-v4.md BNF-SURF-019 (capability id) is
amended to drop the `.t2` suffix example. The new example is
`cap.workflow.template.start`.

D-7.5 Naming-convention-BNF-v4.md's frontmatter `companion_docs:` list
is amended to remove `docs/standards/capability-tier-matrix.md` and
to add `docs/standards/tenant-class-authoring.md` (if ADR-0331 chose
absorb).

### D-8 Capability-tier-deltas deliverable handling

D-8.1 The 16 existing `capability-tier-deltas-vs-counterparts-2026-05-20.md`
deliverables are retained in place during the Wave 15J Phase 1 soak
window (90 days).

D-8.2 At Wave 15J Phase 1 entry, the deliverables are not deleted but
are flagged in their frontmatter as `status: superseded by ADR-0329`.

D-8.3 At Wave 15J Phase 2 entry, the deliverables are archived to
`microservices/<service>/_archive/2026-05-20-tier-deltas/` so that
the active directory is clean while the historical evidence survives.

D-8.4 The audit deliverable schema in ADR-0328 is updated in Wave 15J
Phase 0 to declare three deliverables per microservice (coherence
audit, feature parity, performance benchmark numbers) instead of four.

D-8.5 The wave findings aggregation in
`.omc/state/wave-findings-aggregation-2026-05-21.md` is amended in
Wave 15J Phase 0 to note that the four-deliverable schema is retired
and that the `Tier-Deltas` column in the aggregation tables is
historical.

### D-9 Rust-crate retirement detection tool

D-9.1 The new Rust crate `oyatie-tier-retirement` is authored under
`tools/oyatie-tier-retirement/`. Its job is to detect tier vocabulary
patterns and suggest replacements.

D-9.2 The crate is a Tier-2 binary per ADR-0083.

D-9.3 The crate ships a CLI verb `oyatie-tier-retirement scan` that
walks the corpus and lists every occurrence of the retired vocabulary.

D-9.4 The crate ships a CLI verb `oyatie-tier-retirement suggest` that
reads the scan output and proposes per-occurrence replacements.

D-9.5 The crate does NOT ship a bulk-rewrite verb. Per B2.060,
substantive content rewrites are bespoke per-microservice authoring,
not scripted. The crate is detection-and-suggestion only.

D-9.6 The crate's CI lane is the
`oya-governance-tier-vocabulary-zero-residue` lane authored under
B2.064. The lane runs `oyatie-tier-retirement scan` and refuses any
non-zero residue outside the allow-list.

D-9.7 The crate is authored in Wave 15J Phase 0 alongside the scrub
work. It is not a prerequisite for the scrub; it is the enforcement
tool for the BLOCKER lane after the scrub.

### D-10 OCI Always Free re-binding

D-10.1 The OCI Always Free profile selection is now expressed as:
`tenant_class IN ['demo_trial'] → OCI Always Free profile`.

D-10.2 Paid tenants on OCI use paid OCI infrastructure (Compute
shapes, Block Volume, Object Storage, paid Autonomous Database).

D-10.3 Paid tenants may allocate Always-Free sandbox sub-tenancies
under their own paid tenancy umbrella for development workloads.
Sub-tenancies do not migrate the paid tenant to demo_trial; the parent
tenant remains paid.

D-10.4 demo_trial tenants are provisioned via OpenTofu under
`iac/oci-guest/always-free/` modules. The `oyatie-cloud` repo's
OpenTofu modules for OCI Always Free are tenant_class-aware: they
refuse to provision if the requesting tenant_class is `paid` (paid
tenants get paid modules instead).

D-10.5 The OCI Always Free re-binding is authored in Wave 15J Phase 0
as the ADR-0328 §D-19 reword.

D-10.6 The re-binding does not affect any other deployment context
(AWS-guest, on-prem, colo, oyatie-as-cloud-provider). Those contexts
have their own deployment-profile selection rules per ADR-0328 §D-15.

### D-11 Cross-reference scrub patterns

D-11.1 Replace `ADR-0316` references with `ADR-0316 (superseded by
ADR-0329)` in any non-supersession-banner content. The supersession
banner uses the canonical "Superseded by ADR-0329" frontmatter; in-
body references update to the compound form.

D-11.2 Replace `capability tier` with `tenant-class capability` in
substantive content where the meaning is "a feature surface for
tenants." Where the meaning is "the retired ladder," delete entirely.

D-11.3 Replace `capability_tier_id` with `tenant_class` in Cedar
policy fragments and Ontology projection manifests.

D-11.4 Replace `bronze.json` / `silver.json` / `gold.json` /
`platinum.json` registry path references with the appropriate
`demo-trial.json` / `paid.json` references per ADR-0331.

D-11.5 Replace `tenant_capability_tier_grants` SQL references with the
new tenant_class table references per ADR-0331.

D-11.6 Replace `microservice_capability_tier_contributions` SQL
references with the new microservice contribution table references
per ADR-0331.

D-11.7 Replace `workflow.capability-tier-grant.v1` event class
references with the new event class references per ADR-0331.

D-11.8 Replace `oya-governance-capability-tier-*` lane name references
with the new lane names per Section E.

D-11.9 Replace per-tier autonomy ceiling references (T1/T2/T3/T4) with
the (now non-tier-stratified) autonomy ceiling concept per ADR-0331.
Note: the T1/T2/T3/T4 in ADR-0022 is the capability-tier autonomy
ladder, which is retired; the T1/T2 in ADR-0083 is the Rust error-
handling tier ladder, which is preserved.

D-11.10 Replace `Bronze` / `Silver` / `Gold` / `Platinum` token uses
that label a capability tier with empty content (delete the token).
The exception is the historical references inside this ADR (ADR-0329)
and the supersession banner inside ADR-0316.

### D-12 Migration timing

D-12.1 Wave 15J is the migration window. It runs after Wave 14
aggregation lands and before Wave 15A (crm rewrite).

D-12.2 Wave 15J Phase 0 (atomic consolidation): 65 change sets per
B2.024.

D-12.3 Wave 15J Phase 1 (REPORT-ONLY soak): 90 days from Phase 0 exit.

D-12.4 Wave 15J Phase 2 (BLOCKER enforcement): from day 91 onward,
indefinitely.

D-12.5 Wave 15J Phase 3 (terminal): permanent; no further action.

### D-13 Foundry, foundry-fitness, oya-governance rename context

D-13.1 The ADR-0316 enforcement lanes were named
`oya-governance-capability-tier-*`. The current canonical
governance-lane prefix is `oya-governance-*` per
`feedback_microservice_layout_authority`. Wave 15J renames the
retired lanes per Section E; the replacement lanes also use the
`oya-governance-*` prefix.

D-13.2 The foundry directory `microservices/foundry/` is currently
retiring per ADR-0136 + ADR-0137 + ADR-0138. The capability-tier
retirement does not depend on the foundry retirement; both proceed
independently.

D-13.3 The `oya-governance-fitness-*` lanes retain their existing names
until each is renamed in its own migration IP per the new
governance-lane-prefix doctrine. Wave 15J does not rename them.

### D-14 Coordination with ADR-0330 and ADR-0331

D-14.1 ADR-0330 (tenant-class replacement model) is authored after
ADR-0329 Acceptance. ADR-0330 codifies the `{demo_trial, paid}` enum,
the composable `billing_components ⊆ {revenue_share, per_seat,
per_usage}` set, the Cedar schema for tenant_class principals, the
Postgres schema for tenant-class state, the demo_trial cap-breach
flow, the demo_trial → paid conversion flow, the per-billing-component
metering shape, and the per-billing-component settlement shape.

D-14.2 ADR-0331 (per-microservice tenant-class adoption) is authored
after ADR-0330 Acceptance. ADR-0331 codifies the per-microservice
plumbing IP template (tenant_class claim binding, billing_components
context attribute, Cedar policy gate templates, demo_trial cap-breach
behavior).

D-14.3 ADR-0329 does not block on ADR-0330 or ADR-0331. The retirement
is independently authoritative.

D-14.4 ADR-0330 and ADR-0331 do not require this ADR to land before
they can be drafted, but they require this ADR to land before they
can land. The dependency chain is:
ADR-0329 (retirement) lands first → ADR-0330 (replacement) lands
second → ADR-0331 (adoption) lands third.

D-14.5 The Wave 15J Phase 0 change sets MAY reference ADR-0330 and
ADR-0331 as "pending" while authoring; the references become live
when ADR-0330 and ADR-0331 are Accepted.

D-14.6 If ADR-0330 or ADR-0331 are delayed beyond the Wave 15J Phase
1 soak window, the soak window extends until the replacement ADRs
land. The BLOCKER lane in Phase 2 cannot promote before ADR-0330 is
Accepted.

### D-15 What this ADR does not touch (final enumeration)

D-15.1 The cellular cell topology (ADR-0248). The cellular criticality
tier vocabulary stays. The four-tier model stays. The dr_tier vocabulary
stays.

D-15.2 The public-API stability ladder (ADR-0037). Preview/stable/GA
stays.

D-15.3 The Rust error-handling ladder (ADR-0083). Tier 1 library / Tier
2 binary stays.

D-15.4 The HLC vs TrueTime clock-discipline ladder (ADR-0252).

D-15.5 The build-ahead-of-certification compliance-readiness ladder
(ADR-0250).

D-15.6 The substance-bar doctrine (ADR-0322). Substance is uniform at
industry-leader grade across all tenants.

D-15.7 The realignment canonical sequence (ADR-0328). The five-phase
sequence and Big-8 sub-sequence stay.

D-15.8 The foundry retirement (ADR-0136, ADR-0137, ADR-0138). The
foundry six-path consolidation proceeds independently.

D-15.9 The inter-microservice communication reform (ADR-0145). Direct
gRPC plus three invariants stays.

D-15.10 The Foundry self-modification doctrine (ADR-0247). Foundry
principals continue to run under Cedar.

D-15.11 Every existing audit-chain event, trace, log, and metric is
unchanged in shape until Wave 15J Phase 0. After Phase 0, new events
emit tenant_class instead of capability_tier_id.

D-15.12 Every existing third-party API surface is unchanged.

D-15.13 Every existing marketplace listing is unchanged. Marketplace
offers are not capability-tier-segmented; they are per-offer Cedar-
permit-gated.

D-15.14 Every existing OpenTofu module is unchanged until Wave 15J
Phase 0 amends the OCI Always Free module to tenant_class gating per
D-10.

D-15.15 Every existing CI lane outside Section E is unchanged.

D-15.16 Every existing ADR outside the supersedes/amends list is
unchanged.

## E. Verification

### E.1 New enforcement lanes

E.1.1 `oya-governance-tier-vocabulary-zero-residue`. Scans the corpus
for the retired vocabulary (Bronze, Silver, Gold, Platinum as
capability-tier labels; "capability tier" as a primitive name;
references to ADR-0316 as live authority). Allow-list per Section D-1.7.
REPORT-ONLY in Wave 15J Phase 1; BLOCKER from Wave 15J Phase 2 onward.

E.1.2 `oya-governance-tenant-class-claim-binding`. Verifies that every
microservice that issues principals binds the tenant_class claim. The
lane runs against ADR-0331's per-microservice adoption IP. REPORT-ONLY
during Wave 15J Phase 1; BLOCKER after ADR-0331's adoption IP lands.

E.1.3 `oya-governance-no-capability-tier-grants`. Verifies that the
retired Postgres tables (`tenant_capability_tier_grants` and
`microservice_capability_tier_contributions`) receive no new writes.
REPORT-ONLY during Wave 15J Phase 1; BLOCKER from Wave 15J Phase 2
onward.

E.1.4 `oya-governance-bronze-silver-gold-platinum-zero-occurrence`.
Stricter than E.1.1; scans for the literal Bronze/Silver/Gold/Platinum
tokens used as capability-tier labels. Allow-list includes ADR-0316
file body (historical), ADR-0329 file body (this ADR, historical),
ADR-0316's supersession banner, the wave findings aggregation, the
realignment review, and the 16 archived `capability-tier-deltas-vs-
counterparts-*.md` deliverables. REPORT-ONLY during Wave 15J Phase 1;
BLOCKER from Wave 15J Phase 2 onward.

E.1.5 `oya-governance-registry-capability-tiers-deletion`. Verifies
that `/Users/jasonlee/oyatie/registry/capability-tiers/` is absent
after Wave 15J Phase 0 lands. BLOCKER from Phase 0 exit onward.

E.1.6 `oya-governance-naming-bnf-n014-n015-amendment`. Verifies that
the naming-convention BNF v4 N-014 and N-015 rules drop the
`.<tier>` segment and that BNF-SB-008 drops the
`registry/capability-tiers/*.yaml` scan obligation. BLOCKER from Phase
0 exit onward.

E.1.7 `oya-governance-adr-0316-supersession-marker`. Verifies that
ADR-0316 carries `status: Superseded by ADR-0329` frontmatter, that
`superseded_by:` includes `ADR-0329`, and that the supersession banner
is the first content block after the frontmatter. BLOCKER from Phase 0
exit onward.

E.1.8 `oya-governance-cellular-criticality-tier-preservation`.
Verifies that the ADR-0248 cellular criticality vocabulary remains
intact (Tier 0..Tier 4 references, cell-tier-* filenames, dr_tier
values). BLOCKER continuously, even before Wave 15J Phase 0.

### E.2 Per-microservice verification

E.2.1 Each Wave 15J Phase 0 change set runs the existing CI lanes
plus the new lanes from E.1.

E.2.2 The change set's evidence record cites this ADR (ADR-0329) and,
once ADR-0331 lands, cites the relevant subsection of ADR-0331.

E.2.3 The change set's reviewer-agent verdict is "APPROVE on green CI
plus zero residue in the affected microservice."

E.2.4 The change set lands via the Foundry-pipeline plus
multispectrum-review v2.4.0 path. No exception is granted.

### E.3 Wave-level verification

E.3.1 Wave 15J Phase 0 exit gate: all 65 change sets land. Each change
set passes the existing CI lanes plus the new lanes from E.1. The
wave findings aggregation reflects the retirement.

E.3.2 Wave 15J Phase 1 entry gate: Phase 0 exit gate green; lanes E.1.1
through E.1.8 are armed in REPORT-ONLY mode.

E.3.3 Wave 15J Phase 1 exit gate: 90 days elapsed since Phase 0 exit;
zero residue reported during the soak window.

E.3.4 Wave 15J Phase 2 entry gate: Phase 1 exit gate green; lanes E.1.1
through E.1.8 promote from REPORT-ONLY to BLOCKER.

E.3.5 Wave 15J Phase 3 entry gate: terminal; no further action.

### E.4 Corpus-level verification

E.4.1 The `oyatie-tier-retirement scan` CLI returns zero occurrences
of the retired vocabulary outside the allow-list, at all times after
Wave 15J Phase 0 lands.

E.4.2 The wave findings aggregation reflects the retirement as a
"closed" remediation backlog item.

E.4.3 The master-plan-sequencing.json file records Wave 15J as
"completed" once Phase 2 entry gate is green.

E.4.4 The ADR-INDEX.md file records ADR-0329 as Accepted and ADR-0316
as Superseded.

### E.5 Doctrine-level verification

E.5.1 The five-canonical-anchor cross-check (per ADR-0328 §D-15) is
satisfied: six deployment contexts, OpenTofu-not-Terraform, macOS-M5+-
only, Rust-strict+frontend-Swift/Kotlin/WinUI/Leptos, OCI Always Free.
None of these depend on capability tiers.

E.5.2 The substance bar (per ADR-0322) is satisfied: the retirement is
authored as bespoke content, not scripted, with named precedent
(ADR-0316), failure-mode tree (tier vocabulary fragments quality bar
across customer classes), capacity math (3,000+ call-sites; 61
directories; 17 standards docs; 7 registry artifacts), observability
hooks (8 new lanes), rollback path (none — see Section F), multi-region
awareness (cellular criticality preserved), sovereign-cell awareness
(Tier-4 cells preserved), and versioning plus deprecation (ADR-0108 +
ADR-0138 patterns followed).

E.5.3 The "industry-leader quality bar" doctrine is satisfied: the
retirement ensures quality stays uniform across tenant_class. No tier-
stratified quality contract survives.

E.5.4 The "flat product catalog" doctrine is satisfied: every customer
gets the same flat substrate. No tier-segmented product surface
survives.

E.5.5 The "no silent regression" doctrine is satisfied: the retirement
is loud, authored, ADR-anchored, lane-enforced, and soak-window-gated.
No silent change occurs.

## F. Rollback

Not applicable.

F.1 No rollback path exists. Once Wave 15J Phase 0 lands and ADR-0316
is marked Superseded, the capability-tier ladder is permanently
retired.

F.2 The git history retains the deleted artifacts for audit and
historical research, but the tree-state restoration of capability-tier
content is explicitly not authorised.

F.3 The historical evidence sufficient to reconstruct the retired
doctrine is preserved in:

F.3.1 ADR-0316 file body (retained with supersession banner).

F.3.2 The 16 `capability-tier-deltas-vs-counterparts-2026-05-20.md`
deliverables (retained during soak; archived after Phase 2).

F.3.3 The wave findings aggregation
`.omc/state/wave-findings-aggregation-2026-05-21.md` (retained as
durable evidence).

F.3.4 The realignment review
`.omc/state/realignment-review-2026-05-21.md` (retained as durable
evidence).

F.3.5 This ADR (ADR-0329) (retained as authoritative retirement
decision).

F.4 If a future doctrine evolution decides the tier ladder is needed
again (no current expectation that this will happen), the new doctrine
must author a fresh ADR rather than restore ADR-0316. The fresh ADR
must justify why the user directive of 2026-05-20 ("we don't have
tiers") is being reversed and must provide its own evidence base. The
historical record from ADR-0316 may inform but does not authorise the
new doctrine.

F.5 The retirement is final because the alternative (a soft-deprecation
with rollback path) would leave tier vocabulary alive in the corpus
for an indefinite window, contradict the user directive, and create
ambiguity about which doctrine is canonical at any given moment. The
hard-retirement-no-rollback discipline matches the ADR-0138 Strangler
terminal-state pattern.

## G. Open questions

### G.1 Replacement-doctrine sequencing

G.1.1 ADR-0330's exact line-floor and section structure remains to be
authored. This ADR binds the reference but does not constrain
ADR-0330's authoring.

G.1.2 ADR-0331's per-microservice plumbing IP template remains to be
authored. This ADR binds the reference but does not constrain
ADR-0331's authoring.

G.1.3 The decision between delete-vs-absorb for
`docs/standards/capability-authoring.md` is owned by ADR-0331 per
B2.011 and D-3.2.

G.1.4 The decision between delete-vs-archive for
`docs/standards/capability-tier-matrix.md` is owned by ADR-0331 per
D-3.1.

### G.2 Wave 14 final aggregation polish

G.2.1 After Acceptance, the wave findings aggregation polishes into a
canonical Wave 14 deliverable at
`.omc/state/wave-14-aggregation.md`. The polishing is owned by the
realignment orchestrator, not by this ADR.

G.2.2 The polished Wave 14 aggregation records ADR-0329, ADR-0330, and
ADR-0331 as the retirement-plus-replacement triple.

### G.3 Crm rewrite sequencing

G.3.1 The crm rewrite (Wave 15A) follows the tier retirement. The crm
rewrite authors against the tenant-class model directly.

G.3.2 If the crm rewrite begins before ADR-0330 lands, the rewrite
references this ADR (ADR-0329) plus the memory file
`feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`
as the authoritative tenant-class source until ADR-0330 lands.

### G.4 Cloud-billing spec sprint sequencing

G.4.1 The cloud-billing spec sprint (Wave 15B) follows the tier
retirement. The spec sprint authors against the tenant-class model
directly.

G.4.2 cloud-billing is the source-of-truth microservice for the
tenant_class enum and billing_components set. Its kernel is already
substance-grade per the realignment review; the spec sprint covers
PRD, ARCHITECTURE, README, contracts, SLOs, Cedar policies, and
runbooks.

### G.5 Marketplace 6-category completion sequencing

G.5.1 The marketplace 6-category completion (also Wave 15A scope per
the realignment review) follows the tier retirement. The completion
authors against the tenant-class model directly.

G.5.2 Marketplace offer eligibility is tenant_class-gated (paid only
for purchases). Per-offer Cedar permits gate finer-grained access.

### G.6 Network → community merge sequencing

G.6.1 Wave 15K (network → community merge) follows this ADR. The
merged community service inherits the tenant-class model directly.

### G.7 Cell retirement sequencing

G.7.1 Wave 15L (cell µservice retire) follows this ADR. The absorbing
services (tenancy, cloud-iac, observability, oyatie-shuffle-sharding
crate, api-gateway, audit-chain) inherit the tenant-class model
directly. The cellular criticality tier-0..tier-3 vocabulary stays in
the absorbing services per B2.036 and D-1.1.

### G.8 Long-tail phase 4 vertical audits

G.8.1 Long-tail Phase 4 vertical audits (real-estate, plant-maintenance,
etc.) follow the tier retirement. The audits author against the
tenant-class model directly.

G.8.2 The realignment review notes that some long-tail audits may
benefit from a leaner format (150-line "scope present + counterpart
anchor" instead of the full 600+400+300 triple). That decision is
owned by the realignment orchestrator and is independent of this ADR.

### G.9 Wave 15J change-set ordering

G.9.1 Wave 15J Phase 0's 65 change sets do not have a strict ordering.
They MAY land in any order. The atomic-consolidation principle from
ADR-0138 applies per change set.

G.9.2 The recommended ordering is: registry deletion first, ADR-0316
supersession second, standards-doc scrubs third, naming-BNF amendments
fourth, per-microservice scrubs fifth. The ordering is a heuristic,
not a rule.

G.9.3 The recommended batch ceiling per day is 5-10 change sets, per
ADR-0328 batch discipline.

### G.10 Vendor and partner communication

G.10.1 The retirement is internal-corpus-scoped. External vendor
contracts and partner integrations are not in scope for this ADR.

G.10.2 If external vendors or partners need notification of the
retirement, the notification is owned by council-product and ops-
compliance, not by this ADR. No external surface currently exists
that exposes capability-tier vocabulary to vendors or partners.

## H. Cross-references

### H.1 Superseded

H.1.1 ADR-0316 (capability-tier-over-product-fragmentation): superseded
by this ADR. ADR-0316 is retained as historical evidence with a
supersession banner.

### H.2 Replaces (the retired doctrine is replaced by)

H.2.1 ADR-0330 (tenant-class replacement model): codifies the
`{demo_trial, paid}` enum and composable `billing_components`. Pending
authoring at ADR-0329 Acceptance.

H.2.2 ADR-0331 (per-microservice tenant-class adoption): codifies the
per-microservice plumbing IP template. Pending authoring at ADR-0329
Acceptance.

### H.3 Inherits retirement pattern from

H.3.1 ADR-0108 (sunset-lifecycle-automation): supplies the machine-
readable sunset schema. ADR-0316's frontmatter `sunset_at`,
`sunset_milestone`, `sunset_topic` fields are set per ADR-0108.

H.3.2 ADR-0138 (foundry six-path deprecation): supplies the Strangler-
pattern adaptation. Wave 15J uses the atomic-consolidation variant
adapted to the zero-current-tenant state.

### H.4 Inherits sequence discipline from

H.4.1 ADR-0328 (substance-bar as canonical sequence and batch
discipline): supplies the wave-batch ceiling and the substance-bar
authoring requirement. Wave 15J runs as a coordinated sub-wave under
ADR-0328 discipline.

H.4.2 ADR-0322 (substance-bar as doctrine and CI enforcement): supplies
the substance-as-blocker-class doctrine. The retirement is authored at
substance-bar quality.

H.4.3 ADR-0327 (Wave 3 completion criteria and promotion gates):
supplies the promotion-gate model. This ADR's Acceptance is direct
(user directive); promotion gates apply to subsequent waves.

H.4.4 ADR-0324 (anti-stamping authoring doctrine): supplies the
no-template-stamping rule. The retirement is authored bespoke, not
stamped.

### H.5 Preserves (these doctrines and vocabularies survive intact)

H.5.1 ADR-0248 (Amazon-shape cellular architecture): cellular
criticality tier-0..tier-3 vocabulary, cell-tier-* filenames, and
dr_tier fields preserved in full.

H.5.2 ADR-0037 (public-API stability tiers and deprecation):
preview/stable/GA vocabulary preserved.

H.5.3 ADR-0083 (Rust error-handling tier decision): Tier 1 library /
Tier 2 binary vocabulary preserved.

H.5.4 ADR-0252 (HLC default, TrueTime tier): clock-discipline ladder
preserved.

H.5.5 ADR-0250 (build-ahead-of-certification): compliance-readiness
ladder preserved.

H.5.6 ADR-0022 (autonomy-ceiling runtime enforcement): autonomy
ceiling concept preserved (per-tier stratification retired; concept
survives without stratification).

### H.6 Coordinates with

H.6.1 ADR-0244 (tenant-as-universal-scoping-primitive): tenant_class
extends the tenant principal claim.

H.6.2 ADR-0243 (Cedar-as-universal-gate): tenant_class enters Cedar's
principal context.

H.6.3 ADR-0245 (substrate vs product layering): substrate-vs-product
separation survives independently.

H.6.4 ADR-0132 (product platform and bundle dissolution): no-grouping no-
bundle rule survives independently.

H.6.5 ADR-0249 (multi-category marketplace): six-category marketplace
survives; offer eligibility is tenant_class-gated.

H.6.6 ADR-0251 (compliance-pack primitive): pack activation is
tenant_class-gated (paid only).

H.6.7 ADR-0255 (BYOK opt-in): BYOK activation is tenant_class-gated
(paid only).

H.6.8 ADR-0263 (audit emission): capability_tier_id field removed
from new event emissions; tenant_class field added.

H.6.9 ADR-0257 (ontology object-type versioning and deprecation
handshake): ontology projection pins to schema revisions; capability-
tier projection pin retired.

H.6.10 ADR-0064 (canonical-base + localization): localisation
activation is per-pack and per-tenant_class; tier-gated localisation
retired.

H.6.11 ADR-0145 (inter-microservice communication reform): direct gRPC
plus three invariants unaffected.

H.6.12 ADR-0247 (foundry self-modification): foundry principals carry
tenant_class claims.

H.6.13 ADR-0253 (HTTP/3 + QUIC default): transport protocol
unaffected.

### H.7 Internal evidence anchors

H.7.1 `.omc/state/realignment-review-2026-05-21.md` (cross-cutting
findings).

H.7.2 `.omc/state/wave-findings-aggregation-2026-05-21.md` (per-
microservice findings rollup; 48 audited microservices at retirement
time).

H.7.3 `feedback_no_capability_tiers_2026_05_20.md` (memory: user
directive "we don't have tiers").

H.7.4 `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`
(memory: replacement model crystallisation).

H.7.5 `feedback_drift_too_big_2026_05_20.md` (memory: drift requires
reconciliation; tier system is part of the drift).

H.7.6 `feedback_flat_product_catalog.md` (memory: everyone is shared;
no tier discrimination).

H.7.7 `feedback_quality_performance_scalability_bar.md` (memory:
industry-leader quality bar; uniform, not tier-stratified).

H.7.8 `feedback_microservice_layout_authority` (memory: oya-governance-*
lane prefix doctrine).

H.7.9 `feedback_no_silent_regression.md` (memory: public structural
changes require ADR plus CI-enforced sunset window; this retirement
follows that doctrine).

H.7.10 `feedback_bominal_inheritance_precedence.md` (memory: Oyatie
session decisions override Bominal inheritance; the retirement is an
Oyatie-session decision).

### H.8 External precedent (for context only — not authoritative for
this retirement)

H.8.1 Stripe pricing tier vs feature tier separation: Stripe uses
Standard/Plus/Premium for pricing but maintains a uniform feature
surface for all paying customers. Oyatie's retirement aligns with this
precedent.

H.8.2 AWS support tier (Basic/Business/Enterprise/Enterprise On-Ramp):
AWS's support tier is a support-class ladder, not a feature-access
ladder. Oyatie's tenant-class model treats support gating as a
support-class distinction (per ADR-0330), not as a capability ladder.

H.8.3 Microsoft 365 licence SKU stratification: Microsoft does
fragment feature access by SKU (E1/E3/E5, Business Basic/Standard/
Premium). Oyatie's retirement explicitly rejects this stratification
model. The user directive of 2026-05-20 says Oyatie does not use the
Microsoft pattern.

H.8.4 Salesforce edition stratification (Essentials/Professional/
Enterprise/Unlimited): same as Microsoft 365. Oyatie's retirement
rejects this stratification model.

H.8.5 GitHub plan stratification (Free/Team/Enterprise/Enterprise
Cloud): GitHub does feature-gate by plan. Oyatie's retirement rejects
this stratification model.

H.8.6 Notion plan stratification (Free/Plus/Business/Enterprise):
Notion does feature-gate by plan. Oyatie's retirement rejects this
stratification model.

H.8.7 The pattern of feature-gating by plan/tier/SKU is the dominant
SaaS-industry pattern. Oyatie's retirement is a deliberate departure
from that pattern. The substantive rationale is in `feedback_quality_
performance_scalability_bar.md` (industry-leader quality bar is
uniform, not stratified) and `feedback_flat_product_catalog.md`
(everyone is shared, no feature gating between paying customers).

## I. Appendix - Retirement scope numbers (reference)

### I.1 Source-tree footprint at retirement time

I.1.1 Capability-tier directories under microservices: 61.

I.1.2 `tier-matrix.md` files: 60 (1 directory is a scaffold without a
matrix).

I.1.3 Registry artifacts in `registry/capability-tiers/`: 7 (bronze.json,
silver.json, gold.json, platinum.json, index.json, checkpoint.json,
microservice-tier-mapping.yaml; the eighth artifact
`vendor-tier-mapping.yaml` is also retired per B2.005).

I.1.4 Standards docs with capability-tier vocabulary: 17.

I.1.5 `capability-tier-deltas-vs-counterparts-2026-05-20.md`
deliverables: 16 (retained during soak; archived after Phase 2).

I.1.6 ADRs that cross-reference ADR-0316 as a live authority:
approximately 400 distinct file references (estimated from the
cross-reference scan in the realignment review).

I.1.7 Microservices with non-trivial tier-vocabulary residue (per the
realignment review tier-retirement candidate counts):

| Microservice | Tier refs reported |
|---|---:|
| shorts | 155 |
| mail | 73 |
| community | 56 |
| workflow-studio | 48 |
| workflow-engine | 47 |
| ontology | 45 |
| api-gateway | 43 |
| network | 35 |
| intelligence | 29 |
| developer-sdk | 28 |
| application | 27 |
| cloud-billing-tax | 12 |
| cloud-billing | 10 |
| messenger | 8 files |
| marketplace | 5 files |
| crm | ~104 |
| plant-maintenance | 401 |
| global-trade | 509 |

I.1.8 Total distinct call-sites (conservative cross-audit extrapolation):
approximately 3,000+ across 77 active microservices.

I.1.9 Total character-level Bronze/Silver/Gold/Platinum occurrences
(earlier scope-audit): approximately 9,300.

### I.2 Wave 15J change-set footprint

I.2.1 Per-microservice change sets: 61.

I.2.2 Registry-deletion change set: 1.

I.2.3 Standards-doc scrub change set: 1 (covers all 17 standards docs
in one coordinated change set, or N change sets per ADR-0328 batch
ceiling).

I.2.4 ADR-0316 supersession change set: 1.

I.2.5 ADR-0328 §D-19 reword change set: 1.

I.2.6 Total Wave 15J Phase 0 change sets: 65.

### I.3 Wave 15J soak window numbers

I.3.1 Soak window length: 90 days.

I.3.2 Soak window start: Wave 15J Phase 0 exit date (actual ISO date at
change-set merge time).

I.3.3 Soak window end: Phase 0 exit + 90 days.

I.3.4 Expected non-zero residue count during soak: zero.

I.3.5 Investigation trigger: any non-zero residue detected by the
REPORT-ONLY lanes.

### I.4 Wave 15J BLOCKER promotion numbers

I.4.1 Promotion target: Phase 1 exit + 1 day.

I.4.2 Lanes promoted: 8 (per E.1.1 through E.1.8).

I.4.3 Lane status post-promotion: BLOCKER.

I.4.4 Indefinite duration: yes; Phase 2 has no terminal exit.

## J. Appendix - Wave 15J change-set evidence template

### J.1 Per-microservice change-set evidence

Each Wave 15J Phase 0 per-microservice change set carries an evidence
record with the following structure:

J.1.1 `change_set_id`: globally unique id.

J.1.2 `microservice_id`: the affected microservice.

J.1.3 `adrs_cited`: `[ADR-0329, ADR-0331]` (ADR-0330 if available).

J.1.4 `files_deleted`: the list of deleted files.

J.1.5 `files_amended`: the list of amended files.

J.1.6 `cross_references_scrubbed`: count of cross-reference replacements.

J.1.7 `ci_lanes_green`: the list of CI lanes that passed for this
change set.

J.1.8 `reviewer_agent_verdict`: "APPROVE on green CI plus zero residue
in the affected microservice."

J.1.9 `wave_id`: `WAVE-15J-PHASE-0`.

J.1.10 `landing_date`: actual ISO date at change-set merge time.

### J.2 Registry-deletion change-set evidence

J.2.1 `change_set_id`: globally unique id.

J.2.2 `adrs_cited`: `[ADR-0329]`.

J.2.3 `files_deleted`: the 8 files under `registry/capability-tiers/`
plus the directory itself.

J.2.4 `ci_lanes_green`: includes `oya-governance-registry-capability-
tiers-deletion`.

J.2.5 `reviewer_agent_verdict`: "APPROVE on green CI."

### J.3 Standards-doc scrub change-set evidence

J.3.1 `change_set_id`: globally unique id.

J.3.2 `adrs_cited`: `[ADR-0329, ADR-0331]`.

J.3.3 `files_amended`: 17 standards docs.

J.3.4 `cross_references_scrubbed`: count.

J.3.5 `ci_lanes_green`: includes the existing standards-doc lanes plus
the new lanes from E.1.

J.3.6 `reviewer_agent_verdict`: "APPROVE on green CI plus zero residue
in the standards docs."

### J.4 ADR-0316 supersession change-set evidence

J.4.1 `change_set_id`: globally unique id.

J.4.2 `adrs_cited`: `[ADR-0329, ADR-0108, ADR-0138]`.

J.4.3 `files_amended`: ADR-0316 (frontmatter + supersession banner).

J.4.4 `ci_lanes_green`: includes `oya-governance-adr-0316-supersession-
marker` and the existing ADR-frontmatter lint lanes.

J.4.5 `reviewer_agent_verdict`: "APPROVE on green CI."

### J.5 ADR-0328 §D-19 reword change-set evidence

J.5.1 `change_set_id`: globally unique id.

J.5.2 `adrs_cited`: `[ADR-0329, ADR-0328]`.

J.5.3 `files_amended`: ADR-0328 (§D-19 reword; `related_adrs:` array
amendment).

J.5.4 `ci_lanes_green`: includes the existing ADR-frontmatter lint
lanes plus `oya-governance-tier-vocabulary-zero-residue`.

J.5.5 `reviewer_agent_verdict`: "APPROVE on green CI."

## K. Appendix - Lane allow-list patterns (machine-readable spec)

### K.1 Permit patterns

The `oya-governance-tier-vocabulary-zero-residue` lane permits the
following patterns:

K.1.1 `Tier 0` (whitespace-bounded), `Tier 1` (whitespace-bounded),
`Tier 2` (whitespace-bounded), `Tier 3` (whitespace-bounded),
`Tier 4` (whitespace-bounded), with provenance check: the surrounding
context must reference ADR-0248 cellular criticality, ADR-0037
public-API stability, ADR-0083 Rust error handling, ADR-0022 autonomy
ceiling (during the transition window only), ADR-0252 clock
discipline, or ADR-0250 compliance readiness.

K.1.2 `cell-tier-1`, `cell-tier-2`, `cell-tier-3`,
`cell-tier-4-financial-grade`, `cell-tier-4-fulfillment-grade`,
`cell-tier-4-il5` (filename or path segment, ADR-0248).

K.1.3 `dr_tier: T1`, `dr_tier: T2` (manifest field, ADR-0248).

K.1.4 `preview`, `stable`, `GA` (manifest field or doc body, ADR-0037).

K.1.5 `Tier 1 library`, `Tier 2 binary` (doc body, ADR-0083).

K.1.6 Generic English `tier` not in a capability-tier context.

K.1.7 References to "ADR-0316" inside ADR-0316 (the superseded ADR
file), ADR-0329 (this ADR), the wave findings aggregation, the
realignment review, and any explicit historical-reference allow-list
entries.

### K.2 Forbid patterns

The lane refuses the following patterns:

K.2.1 `Bronze` (whitespace-bounded), `Silver` (whitespace-bounded),
`Gold` (whitespace-bounded), `Platinum` (whitespace-bounded), when
used as a capability-tier label.

K.2.2 `capability tier` (case-insensitive, whitespace-bounded), `capability-tier`
(hyphenated), used as a primitive name.

K.2.3 `capability_tier_id` (field name).

K.2.4 `capability-tier-grant` (primitive name).

K.2.5 `tenant_capability_tier_grants` (Postgres table name).

K.2.6 `microservice_capability_tier_contributions` (Postgres table
name).

K.2.7 `registry/capability-tiers/` (path).

K.2.8 `capability-tiers/tier-matrix.md` (path).

K.2.9 References to `ADR-0316` as a live authority (i.e., outside the
allow-list).

K.2.10 `oya-governance-capability-tier-*` (retired lane name prefix).

### K.3 Lane verdict shape

K.3.1 `verdict: PASS` when zero forbid-pattern matches occur outside
the allow-list.

K.3.2 `verdict: FAIL` when at least one forbid-pattern match occurs
outside the allow-list.

K.3.3 During Wave 15J Phase 1 (REPORT-ONLY), `FAIL` does not block
merges but pages axis-foundry for investigation.

K.3.4 During Wave 15J Phase 2 (BLOCKER), `FAIL` blocks merges.

## L. Appendix - Migration ledger schema

### L.1 Per-microservice retirement record

The Wave 15J Phase 0 migration ledger carries one record per
microservice. The record shape:

L.1.1 `microservice_id`: the affected microservice.

L.1.2 `directory_deleted`: yes/no for `microservices/<service>/capability-tiers/`.

L.1.3 `tier_matrix_deleted`: yes/no for the `tier-matrix.md` file.

L.1.4 `prd_scrubbed`: yes/no for the PRD scrub.

L.1.5 `architecture_scrubbed`: yes/no for the ARCHITECTURE scrub.

L.1.6 `readme_scrubbed`: yes/no for the README scrub.

L.1.7 `ips_scrubbed`: count of IP slices scrubbed.

L.1.8 `runbooks_scrubbed`: count of runbooks scrubbed.

L.1.9 `cedar_scrubbed`: yes/no for the Cedar policy scrub.

L.1.10 `ontology_scrubbed`: yes/no for the Ontology projection
manifest scrub.

L.1.11 `workflow_scrubbed`: yes/no for the Workflow template binding
scrub.

L.1.12 `ux_shell_scrubbed`: yes/no for the UX shell manifest scrub.

L.1.13 `compliance_overlay_scrubbed`: yes/no for the compliance-pack
overlay scrub.

L.1.14 `observability_scrubbed`: yes/no for the observability profile
scrub.

L.1.15 `cost_profile_scrubbed`: yes/no for the cost profile scrub.

L.1.16 `ci_lane_definitions_scrubbed`: yes/no for the CI lane scrub.

L.1.17 `catalog_yamls_scrubbed`: count of catalog YAMLs scrubbed.

L.1.18 `change_set_id`: the change-set id.

L.1.19 `landing_date`: actual ISO date at change-set merge time.

L.1.20 `reviewer_agent_verdict`: per multispectrum-review v2.4.0
output.

### L.2 Registry-deletion record

L.2.1 `files_deleted`: list of 8 files.

L.2.2 `directory_deleted`: yes for `registry/capability-tiers/`.

L.2.3 `change_set_id`: the change-set id.

L.2.4 `landing_date`: actual ISO date.

L.2.5 `reviewer_agent_verdict`: per multispectrum-review v2.4.0.

### L.3 Standards-doc scrub record

L.3.1 `files_amended`: list of 17 standards docs.

L.3.2 `cross_references_scrubbed`: count.

L.3.3 `change_set_id`: the change-set id.

L.3.4 `landing_date`: actual ISO date.

L.3.5 `reviewer_agent_verdict`: per multispectrum-review v2.4.0.

### L.4 ADR-0316 supersession record

L.4.1 `adr_file`: `docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md`.

L.4.2 `frontmatter_amended`: yes (status, superseded_by, sunset_at,
sunset_milestone, sunset_topic).

L.4.3 `supersession_banner_authored`: yes.

L.4.4 `change_set_id`: the change-set id.

L.4.5 `landing_date`: actual ISO date.

L.4.6 `reviewer_agent_verdict`: per multispectrum-review v2.4.0.

### L.5 ADR-0328 §D-19 reword record

L.5.1 `adr_file`: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`.

L.5.2 `section_reworded`: §D-19.

L.5.3 `related_adrs_amended`: yes (added ADR-0329, ADR-0330, ADR-0331).

L.5.4 `change_set_id`: the change-set id.

L.5.5 `landing_date`: actual ISO date.

L.5.6 `reviewer_agent_verdict`: per multispectrum-review v2.4.0.

## M. Closeout statement

The capability-tier ladder retires on Acceptance of this ADR. The
canonical replacement model (ADR-0330) and per-microservice adoption
mechanic (ADR-0331) follow. The substance-bar quality contract
(ADR-0322), the flat-product-catalog doctrine, the industry-leader
quality bar, the no-silent-regression discipline, and the no-vendor-
lock-in posture are reinforced by this retirement because each of those
doctrines was strained by the tier ladder. The cellular criticality
tier-0..tier-3 vocabulary (ADR-0248), the public-API stability tier
vocabulary (ADR-0037), and the Rust error-handling tier vocabulary
(ADR-0083) are preserved. The retirement is final, ADR-anchored, lane-
enforced, and irreversible.

<!-- ADR-0329 COMPLETION REPORT
  output: /Users/jasonlee/oyatie/docs/decisions/ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  line_count: 2555
  supersedes: ADR-0316
  replaces_with: ADR-0330 + ADR-0331
  microservices_affected: 77 (estimated 3,000+ tier references; 61 capability-tiers/ directories; 60 tier-matrix.md files; 16 capability-tier-deltas deliverables; 17 standards docs; 8 registry artifacts)
  preserves: ADR-0248 cellular criticality tier-0..tier-3 vocabulary (Tier 0..Tier 4, cell-tier-* filenames, dr_tier values); ADR-0037 preview/stable/GA stability tiers; ADR-0083 Tier 1 library / Tier 2 binary; ADR-0252 HLC vs TrueTime tier; ADR-0250 compliance-readiness tier
  numbered_clauses: 100 (B2.001 — B2.100)
  sections: A Context, B Decision, C Consequences, D Implementation footprint, E Verification, F Rollback, G Open questions, H Cross-references, I-L Appendices, M Closeout
  halt_cleanly: yes
-->

## Historical residual from ADR-316 (E3 fold 2026-08-06)

**Title:** ADR-0316-capability-tier-over-product-fragmentation

**Preserved decision gist:** This ADR's canonical decision is expanded in Section B. Adjacent enterprise product categories become tenant-granted capability tiers over shared primitives unless Section D-10 proves that a separate reusable substrate service is required.

_Source file archived after fold; full body in git history / docs/adr-archive/._
