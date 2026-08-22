---
doc_class: Audit Report
shape: Reference
status: Final
date: 2026-05-20
auditor: Executor agent (claude-sonnet-4-6)
authority_standard: docs/standards/documentation-rigor.md
scope_corpora:
  - memory: /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/
  - specs: /Users/jasonlee/oyatie/specs/
  - runbooks: /Users/jasonlee/oyatie/docs/runbooks/
---

# Memory · Spec · Runbook Audit — 2026-05-20

---

## §1 Scope

### Corpus counts

| Corpus | Files audited | Notes |
|--------|--------------|-------|
| Memory | 54 files (53 feedback_\* + project_\* + MEMORY.md index) | Directory: `.claude/projects/-Users-jasonlee-oyatie/memory/` |
| Specs | 57 JSON files (top-level `specs/*.json`) | Subdirectories (`capabilities/`, `catalog/`, `policy/`, `products/`) excluded from this pass |
| Runbooks | 153 `.md` files | `docs/runbooks/*.md` (flat + subdirectories counted via `find`) |

### Audit standard applied

`docs/standards/documentation-rigor.md` (published 2026-05-20, authority tier 2). The relevant rows from its §2 table:

- **Spec row:** `_meta` block mandatory with `purpose`, `industry_citations`, `related_adrs`, `status`, `enforcement_status`, `version`; every `properties` field needs `description` ≥1 sentence + ≥1 `examples` entry; cross-reference to binding ADR via `_meta.binding_adr`; valid JSON.
- **Runbook row:** §A Trigger / §B Pre-checks / §C Procedure (≥10 numbered steps) / §D Verification / §E Rollback / §F Post-incident / §G References; every step has command/API/Cedar permit/OpenBao path; timing budget per step; audit-stream tag emitted; explicit "if this step fails" branch; cross-reference to ≥2 related runbooks; frontmatter: `status`, `owner`, `last-updated`, `related-ADRs`.
- **Memory body structure (feedback/project type):** `description:` frontmatter summarises body; `type: feedback` or `project` in metadata; `Why:` and `How to apply:` sections populated; no prohibited content (code patterns, file paths, git history, ephemeral state, doc content verbatim).

---

## §2 Memory Corpus Findings

### 2.1 MEMORY.md index integrity

**Files on disk:** 54 (excluding MEMORY.md itself)
**Files cited in MEMORY.md:** 52
**Orphaned files (on disk, not cited in MEMORY.md):** 2

| Orphaned file | Impact |
|--------------|--------|
| `feedback_lifecycle_automation_universal.md` | Active, non-superseded memory with substantive "How to apply" content. Contains the lifecycle-automation-universal doctrine (fitness lane for every lifecycle state machine). Not reachable via MEMORY.md index — agents will not load it. **P0.** |
| `feedback_no_exceptions_canonical.md` | Active, non-superseded memory citing ADR-0083/0105/0107. Body prescribes sunset-clause canonicity rules. Not reachable via MEMORY.md index. **P0.** |

**Action:** Add both to MEMORY.md under appropriate headings before the next agent session.

### 2.2 Superseded memories still present in MEMORY.md as canonical entries

MEMORY.md uses inline `SUPERSEDED` annotations but does not segregate these into a clearly demarcated "Superseded / History" section. Five superseded entries appear in the active canonical list, interleaved with current entries:

| Entry | Superseded by | Position in MEMORY.md | Risk |
|-------|-------------|----------------------|------|
| `feedback_grit_claim_work_done.md` | `feedback_deprecate_external_agent_coord_tooling.md` (2026-05-16) | Listed under active entries with inline note | Agents may follow grit claim/work/done protocol, which is retired |
| `feedback_rtk_proxy_fmt_silent_passthrough.md` | `feedback_deprecate_external_agent_coord_tooling.md` (2026-05-16) | Listed under active entries with inline note | Agents may attempt rtk bypass path for cargo |
| `feedback_vcs_canonical_2026_05_16.md` | `feedback_git_canonical_2026_05_18.md` (2026-05-18) | Listed with "(SUPERSEDED)" in title | Body still contains detailed `oya vcs claim/work/done` agent flow as "How to apply" |
| `feedback_layer_enum_12_value_canonical.md` | `feedback_layer_enum_adr_0105_13_canonical.md` (2026-05-16) | Listed with "(SUPERSEDED)" in title | Body content is stub/correct, low risk |
| `feedback_self_merge_on_ci_green.md` | `feedback_self_merge_via_contract_path.md` (2026-05-16) | Listed with "(SUPERSEDED)" in title | Body correctly warns; low risk |

**Recommendation (P1):** Move all five to a `## Superseded (history only)` section at the bottom of MEMORY.md. The mixed list increases the probability an agent loads a superseded memory as authoritative context.

### 2.3 Body-structure failures — missing `Why:` and `How to apply:` sections

The documentation-rigor standard §2 (memory body structure) requires `Why:` and `How to apply:` sections for every feedback and project memory. Audit result:

**Files WITH both `Why:` and `How to apply:`:** 18 of 53 feedback files (34%)
**Files MISSING one or both sections:** 35 of 53 (66%)

Files missing both sections (P1 — agents cannot easily extract actionable guidance):

| File | Category | Notes |
|------|----------|-------|
| `feedback_automate_everything.md` | Operational doctrine | Body is imperative prose but no labelled Why/How sections |
| `feedback_autonomous_decision_principles.md` | Core decision doctrine | ADR-dense; no labelled sections |
| `feedback_autonomous_implementation_artifacts.md` | Operational doctrine | Contains code-path snippets; no labelled sections |
| `feedback_canonical_base_localization.md` | Architecture rule | No labelled sections |
| `feedback_clean_architecture_requirements.md` | Architecture rule | Dense ADR list; no labelled sections |
| `feedback_codex_bulk_resolve_antipattern.md` | Process anti-pattern | No labelled sections |
| `feedback_consensus_debate_spectrum_lens_subagents.md` | Process rule | No labelled sections |
| `feedback_deprecate_external_agent_coord_tooling.md` | Tooling policy | No labelled Why/How (body is imperative) |
| `feedback_doc_coverage_enforced.md` | CI/doc policy | No labelled sections |
| `feedback_flat_product_catalog.md` | Architecture | No labelled sections |
| `feedback_governance_pipeline_canonical.md` | Workflow | No labelled sections |
| `feedback_glossary_ontology_not_object_graph.md` | Glossary | No labelled sections |
| `feedback_glossary_shared_not_platform.md` | Glossary | No labelled sections |
| `feedback_grit_claim_work_done.md` | SUPERSEDED | No labelled sections (superseded, lower urgency) |
| `feedback_layer_enum_adr_0105_13_canonical.md` | Architecture | No labelled sections |
| `feedback_mcc_folds_into_m01.md` | Planning structure | No labelled sections |
| `feedback_milestone_phase_hierarchy.md` | Planning structure | No labelled sections |
| `feedback_model_routing.md` | Operational | No labelled sections |
| `feedback_multispectrum_adherence_facets.md` | Process | No labelled sections |
| `feedback_multispectrum_review_v22.md` | Process | No labelled sections |
| `feedback_naming_justification.md` | Naming | No labelled sections |
| `feedback_no_exceptions_canonical.md` | Architecture | No labelled sections (also orphaned — §2.1) |
| `feedback_no_silent_regression.md` | Quality bar | No labelled sections |
| `feedback_git_canonical_2026_05_18.md` | VCS canonical | No labelled sections |
| `feedback_pipeline_clog_gotchas_2026_05_17.md` | Operational | No labelled sections |
| `feedback_pr82_dishonest_exit_gate.md` | Process | No labelled sections |
| `feedback_quality_performance_scalability_bar.md` | Quality bar | No labelled sections |
| `feedback_repeat_mistake_prevention.md` | Process | No labelled sections |
| `feedback_rtk_proxy_fmt_silent_passthrough.md` | SUPERSEDED | No labelled sections |
| `feedback_self_merge_via_contract_path.md` | Process | No labelled sections |
| `feedback_workflow_is_shared.md` | Architecture | No labelled sections |
| `feedback_workflow_objectgraph_adapter_layer.md` | Architecture | No labelled sections (body is very dense; structure is implied but not labelled) |
| `feedback_workflow_studio_scope.md` | Product scope | No labelled sections |
| `feedback_lifecycle_automation_universal.md` | Operational doctrine | Has How to apply but no explicit Why header (also orphaned) |
| `feedback_mls_rfc_9420_e2ee_personal_messenger.md` | Architecture | Has labelled Why/How — compliant |

### 2.4 Description frontmatter accuracy failures

Several memories have `description:` fields that no longer accurately reflect the body, or that describe a tool/protocol that has since been superseded without the description being updated:

| File | description: field issue |
|------|------------------------|
| `feedback_grit_claim_work_done.md` | Description says "rtk-ai/grit claim→work→done is the canonical agent-coordination primitive" — this is now the *superseded* position. Description does not start with SUPERSEDED unlike other superseded files. **P1** |
| `feedback_workflow_objectgraph_adapter_layer.md` | Description still names "Object Graph" (old name); MEMORY.md index itself notes it is RETIRED per ADR-0145. Body body text uses "Object Graph" throughout, not "Ontology" per `feedback_glossary_ontology_not_object_graph.md`. **P1 — semantic inconsistency between memories** |
| `feedback_flat_product_catalog.md` | Description and body both use "Object Graph" terminology (old); also lists `intelligence-grit-cli` and `intelligence-icm-cli` as planned crates under Foundry — both using deprecated tooling names. **P1** |
| `feedback_autonomous_implementation_artifacts.md` | Body references `grit claim` and `grit done` as the acceptance gate flow (lines ~18-20), which is the retired grit protocol. Not superseded file. **P1** |
| `feedback_repeat_mistake_prevention.md` | "Pre-flight runbook for every sanctioned primitive (grit, icm, tooling-agent-read)" — names retired tools as sanctioned primitives. "After any error: search ICM with `rtk icm recall`" — uses retired rtk/icm invocation syntax. **P1** |
| `feedback_doc_coverage_enforced.md` | "Every Impl-Plan must contain: `## Grit Claim Symbols`, `## ICM Rows to Emit`" — retired tool references in a currently-active compliance rule. **P1** |
| `feedback_autonomous_decision_principles.md` | Lists `grit done --agent <id> succeeds` as a completion criterion. Retired protocol. **P1** |
| `feedback_naming_justification.md` | "Scaffold-time: emit the JUSTIFICATION block as ICM" — uses retired ICM invocation. **P1** |
| `feedback_clean_architecture_requirements.md` | "direct git/gh use with `icm store -t direct-tool-invocations` rationale" — retired ICM invocation in compliance rule. **P1** |
| `feedback_multispectrum_review_v22.md` | MEMORY.md description says "v2.2.0 doctrine" but the canonical spec `multispectrum-review.json` is now at **v2.4.0** (supersedes v2.2.0 and v2.3.0). Memory file named `feedback_multispectrum_review_v22.md` with description "v2.2.0" is presented as current doctrine. A separate file `feedback_multispectrum_adherence_facets.md` covers v2.3.0 A-family additions. Neither covers the v2.4.0 (21-facet Wave-3-A roster). **P0 — no memory covers current canonical multispectrum version.** |

### 2.5 Retired tool references in non-superseded memories

The following non-superseded memory files contain instructional references to deprecated tools (grit, rtk, icm) in their `How to apply` or procedure sections. This is distinct from historically citing these tools — the concern is where an agent reading the file would be instructed to use them:

**Active-instruction references to retired tools (P1):**

| File | Retired tool cited in active instruction | Severity |
|------|----------------------------------------|----------|
| `feedback_automate_everything.md` | "ICM/git/grit hygiene (claim → work → done)" as example of what to script | P1 |
| `feedback_autonomous_decision_principles.md` | `grit done --agent <id>` as success criterion | P1 |
| `feedback_autonomous_implementation_artifacts.md` | `grit claim symbols` / `grit done` as acceptance gate flow | P1 |
| `feedback_clean_architecture_requirements.md` | `icm store -t direct-tool-invocations` as rationale requirement | P1 |
| `feedback_doc_coverage_enforced.md` | `## Grit Claim Symbols` and `## ICM Rows to Emit` as required Impl-Plan sections | P1 |
| `feedback_naming_justification.md` | "emit the JUSTIFICATION block as ICM" at scaffold time | P1 |
| `feedback_repeat_mistake_prevention.md` | "Pre-flight runbook for grit, icm" / "`rtk icm recall`" / stale-grit-worktree instructions | P1 |
| `feedback_milestone_phase_hierarchy.md` | "grit claims operate at Implementation-plan granularity" | P2 |
| `feedback_no_silent_regression.md` | "Grit symbol-lock contract" in compliance table | P2 |
| `feedback_multispectrum_review_v22.md` | `[[grit-claim-work-done]]` backlink | P2 |

### 2.6 Prohibited content in memory files

Per documentation-rigor.md §"What NOT to save in memory": code patterns, file paths, git history, ephemeral state (PR numbers, issue numbers, working-tree state) should not be stored as memory.

Highest-violation files:

| File | File path refs | Code blocks | Git/PR refs | Notes |
|------|---------------|-------------|------------|-------|
| `feedback_autonomous_implementation_artifacts.md` | 35 | 1 | 0 | Dense crate path listing — closer to doc content than memory |
| `feedback_doc_coverage_enforced.md` | 33 | 0 | 1 | Lists exact crate paths, layer rules — duplicates spec content |
| `feedback_flat_product_catalog.md` | 0 | 2 | 25 | 25 git-history-style references to ADR branches/revisions |
| `feedback_clean_architecture_requirements.md` | 2 | 1 | 20 | 20 references to git revision-style ADR anchors |
| `feedback_glossary_shared_not_platform.md` | 0 | 0 | 29 | 29 git-history-style references — almost entirely historical context |
| `feedback_pipeline_clog_gotchas_2026_05_17.md` | 14 | 0 | 14 | 14 PR number refs ("PR #96", "PR #97"), ephemeral state |
| `feedback_multispectrum_review_v22.md` | 15 | 0 | 3 | File path lists for evidence directories |
| `feedback_workflow_is_shared.md` | 3 | 0 | 14 | 14 git-history refs |

**Assessment:** Most violations are borderline — the memories are serving as lightweight ADR summaries rather than true "lessons" memory. The most clear violations are `feedback_pipeline_clog_gotchas_2026_05_17.md` (specific PR numbers = ephemeral state) and `feedback_flat_product_catalog.md` / `feedback_glossary_shared_not_platform.md` (git-history narrative that belongs in ADRs, not memory).

### 2.7 Cross-references to superseded ADRs in memory files

Several memories reference ADRs that have been superseded or are known to be in-flight conflicts:

| Memory file | ADR cited | Issue |
|------------|----------|-------|
| `feedback_layer_enum_adr_0105_13_canonical.md` | ADR-0056 (12-value) | References ADR-0056 as the baseline being superseded — correct historical cite, but the description should clarify ADR-0056 remains the authority for naming; only the layer *count* is superseded by ADR-0105 |
| `feedback_bominal_inheritance_precedence.md` | ADR-0190, ADR-0208, ADR-0210, ADR-0215, ADR-0223, ADR-0224, ADR-0231, ADR-0232 | These are high-numbered Bominal ADRs. No verification possible from this corpus that these ADRs exist in the oyatie repo — possible phantom ADR references carried over from Bominal context |
| `feedback_workflow_objectgraph_adapter_layer.md` | ADR-0006, ADR-0035, ADR-0103, ADR-0106, ADR-0107 | MEMORY.md itself marks this memory RETIRED per ADR-0145. Body does not reflect ADR-0145 supersession. |
| `feedback_multispectrum_review_v22.md` | None cited | No binding ADR reference despite describing a major process change (v2.2.0). **P1** |

### 2.8 Multispectrum version gap (P0)

The current canonical spec is `multispectrum-review.json` at **version 2.4.0** (Wave-3-A, 21-facet roster, 2026-05-20). The memory corpus covers:

- v2.2.0 via `feedback_multispectrum_review_v22.md` (MEMORY.md entry: "11-13 facets")
- v2.3.0 A-family additions via `feedback_multispectrum_adherence_facets.md`
- **v2.4.0: NO MEMORY FILE EXISTS**

Consequence: agents relying on memory for multispectrum review protocol will use the v2.2.0 11-13 facet model instead of the v2.4.0 21-facet model. This will produce non-conformant review artifacts.

---

## §3 Specs Corpus Findings

### 3.1 JSON validity

All 57 top-level `specs/*.json` files parse as valid JSON. Zero invalid-JSON findings.

### 3.2 Missing `_meta` block (P0)

15 of 57 specs (26%) have no `_meta` block at all — a hard violation of the documentation-rigor §2 spec row requirement:

| Spec file | Contains data? | Risk |
|-----------|---------------|------|
| `chaos-engineering-substrate-canonical.json` | Yes — has schema content | No purpose, version, status, ADR binding discoverable by tooling |
| `csi-storage-class-canonical.json` | Yes | Same |
| `design-spec-maturity-claims.json` | Yes | Same |
| `evidence-taxonomy.json` | Yes | Same |
| `feature-flag-substrate-canonical.json` | Yes | Same |
| `hyperscaler-gates.json` | Yes | Same |
| `master-plan-sequencing.json` | Yes — root navigation doc | No version; agents reading the root pointer have no authoritative version |
| `multi-region-disposition-canonical.json` | Yes | Same |
| `per-tenant-audit-log-slicing-canonical.json` | Yes | Same |
| `schema-registry-canonical.json` | Yes | Same |
| `score-cards.json` | Yes | Same |
| `sovereign-cloud-air-gapped-canonical.json` | Yes | Same |
| `stop-conditions.json` | Yes | Same |
| `tenant-environment-tiers-canonical.json` | Yes | Same |
| `workspace-hygiene.json` | Yes — uses `$schema`, `id`, `status` top-level | Non-standard structure; no `_meta`; effectively undiscoverable by meta tooling |

### 3.3 Missing `version` field (P0)

4 specs have a `_meta` block but are missing the `version` field:

| Spec file | Status | Notes |
|-----------|--------|-------|
| `cedar-fragment-schema.json` | Proposed | No version despite being a "Proposed" published schema |
| `compliance-pack-schema.json` | Proposed | No version — used by compliance-pack onboarding tooling |
| `final-report-schema.json` | draft-sidecar | No version |
| `tenant-model.json` | Proposed | No version — foundational tenant model consumed by multiple µservices |

### 3.4 Missing `purpose` field in `_meta` (P1)

2 specs have a `_meta` block but are missing `purpose`:

| Spec file | Notes |
|-----------|-------|
| `deployment-ops-contract.json` | `_meta` exists; `purpose` is absent |
| `planning-closure-status-closure-ledger.json` | `_meta` exists with `doc_class`, `spec_id`, `version`, `status`, `created_at`, `owner_team` — no `purpose` |

### 3.5 Missing `industry_citations` field (P1)

The documentation-rigor §2 spec row requires `_meta.industry_citations`. Of the 42 specs with a `_meta` block, 39 are missing `industry_citations`. Only 3 include it:

- `cedar-fragment-schema.json`
- `compliance-pack-schema.json`
- `tenant-model.json`
- `per-microservice-flat-layout.json`
- `industry-best-practice-conformance.json` (implicitly — no `_meta`)

This is systemic. The 39 missing include critical specs (`decision-principles.json`, `forbidden-operations.json`, `multispectrum-review.json`, `dr-business-continuity.json`, `throttling-tiers.json`, etc.).

### 3.6 Missing `enforcement_status` field (P1)

12 of 57 specs have `enforcement_status`. 45 do not:

- 15 have no `_meta` at all (counted in §3.2)
- 30 have `_meta` but no `enforcement_status`

The 30 with `_meta` but missing `enforcement_status` include foundational specs: `decision-principles.json`, `forbidden-operations.json`, `governance-amendment.json`, `oyatie-doctrine.json`, `multispectrum-review.json`, `dr-business-continuity.json`, `gitops-vcs-replacement.json`, `iterative-fix-loop.json`, `knowledge-graph-schema.json`, `markdown-retirement-policy.json`, `masterplan.json`, `merge-queue-parked-pr.json`, `microservice-migration-tooling.json`, `planning-closure-contract.json`, `plan-schema.json`, `root-hub-pointers.json`, `test-standard.json`, `crate-naming-audit.json`, `codeview-read-surface.json`, `agentic-slo-gated-promotion.json`, and others.

### 3.7 Missing `related_adrs` / `binding_adr` field (P1)

20+ specs have neither `related_adrs`, `binding_adr`, nor `adr` in their `_meta`. Selected critical missing cases:

| Spec file | Why it matters |
|-----------|---------------|
| `decision-principles.json` | Foundational doc; no ADR binding |
| `decision-rights.json` | Foundational doc; no ADR binding |
| `forbidden-operations.json` | CI-enforced; no ADR binding |
| `governance-amendment.json` | Constitutional-drift spec; no ADR binding |
| `oyatie-doctrine.json` | P0..P16 doctrine; no ADR binding |
| `planning-closure-contract.json` | Gate spec; no ADR binding |
| `iterative-fix-loop.json` | Has `planned_enforcement_refs` but no ADR binding |
| `knowledge-graph-schema.json` | Has `related` but no `binding_adr` |
| `markdown-retirement-policy.json` | Has `user_directives_2026_05_13_cascade` but no binding ADR |
| `masterplan.json` | Has multiple session directives but no binding ADR |

### 3.8 Properties missing `description` or `examples` (P1)

Sampled 6 key specs for per-property completeness:

| Spec | Total properties | Missing description | Missing examples |
|------|-----------------|--------------------|--------------------|
| `tenant-model.json` | 29 | 0 | 5 (`merchant_status`, `kyc_status`, `provider_credential_mode`, `locked`, `created_at`) |
| `compliance-pack-schema.json` | 22 | 1 (`created_at`) | 5 (`signature`, `cedar_fragments`, `audit_chain_requirements`, `provider_byok_required`, `encryption_byok_required`) |
| `cedar-fragment-schema.json` | 19 | 0 | 5 (`effective_at`, `sunset_at`, `signature`, `coverage_role`, `body_hash`) |
| `saga-shape.json` | 7 | 5 (all except `saga_id` and `axis`) | 7 (all) |
| `throttling-tiers.json` | 5 | 5 (all) | 5 (all) |
| `tenant-lifecycle.json` | 5 | 5 (all) | 5 (all) |

`saga-shape.json`, `throttling-tiers.json`, and `tenant-lifecycle.json` have schema structures with zero property descriptions and zero examples — despite being Accepted specs with `enforcement_status` set (throttling, saga) or version 1.0.0 (lifecycle). These are full violations of the documentation-rigor spec row.

### 3.9 BYOK terminology disambiguation (post-2026-05-20 requirement)

Per `feedback_byok_everywhere_credentials.md` (ADR-0255 §D-4 KS#10) and `feedback_compliance_pack_primitive.md` (ADR-0251 §D-10), there are two distinct BYOK concepts that must be disambiguated:

- **provider-BYOK:** `provider_credential_mode ∈ {platform_default, byok, byok_required_by_pack}` — opt-in LLM/AI provider credentials
- **encryption-BYOK:** KMS/HSM root key tenant ownership per ADR-0251 §D-10

Audit findings:

| Spec | Issue |
|------|-------|
| `compliance-pack-schema.json` | Contains BOTH `provider_byok_required` (boolean, old field) AND `provider_credential_mode` (new field). **Field drift: `provider_byok_required` is the old `byok_enabled` equivalent and should be removed or renamed.** Has both `encryption_byok_required` (correct, separate concern) and the old `provider_byok_required` (ambiguous). **P1** |
| `tenant-model.json` | Uses correct `provider_credential_mode` with enum `["platform_default", "byok", "byok_required_by_pack"]`. Compliant. |
| `platform-architecture.json` | Has `encryption_byok` as a category and "BYOK eligibility" minimum fragments — ambiguous which BYOK type is meant in the Cedar gate. No disambiguation comment. **P2** |

No spec uses the old standalone `byok_enabled` boolean as the sole field — zero full field-drift violations. The `compliance-pack-schema.json` case is partial drift (old field co-exists with new).

### 3.10 OpenAPI / AsyncAPI / proto3 applicability

`documentation-rigor.md` §2 notes "OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3 where applicable." No spec in `specs/*.json` claims to be or embed an OpenAPI/AsyncAPI document. This requirement applies to API surface specs that are not in this directory — no violation within the audited corpus. Noted for completeness.

### 3.11 Status field casing inconsistency (P2)

`_meta.status` casing is inconsistent across specs:

| Status value used | Count |
|-------------------|-------|
| `Accepted` (title case) | ~18 specs |
| `accepted` (lower case) | ~6 specs (`agentic-slo-gated-promotion.json`, `industry-best-practice-conformance.json`, `microservice-migration-tooling.json`, `per-microservice-flat-layout.json`, `planning-closure-status-closure-ledger.json`) |
| `Proposed` | ~4 specs |
| `Scaffolded` | 2 specs |
| `draft-sidecar` | 3 specs |
| `implemented-masterplan-json-live-hierarchy-index` | 1 spec (masterplan.json) — non-standard status value |
| `Accepted-for-masterplan-P00` | 1 spec (gitops-vcs-replacement.json) — non-standard status value |

The `masterplan.json` and `gitops-vcs-replacement.json` status values are freeform and will break any tooling that parses `status` against an enum.

---

## §4 Runbook Corpus Findings

### 4.1 Stub prevalence (P0)

**136 of 153 runbooks (89%) are stubs.** The stub text is:

> `> **Last verified:** 2026-05-09 (stub authored to satisfy doc-link integrity; full procedure lands at W-Foundation gate)`

Stub runbooks contain:
- No frontmatter `status:` or `owner:` fields
- Exactly 4 numbered steps (generic placeholders)
- Zero cross-runbook references
- Zero audit-stream tags
- Generic "Confirm SLO error budget recovers" as the sole verification
- No timing budgets

The 136 stub runbooks include critical operational procedures: `sev1-incident-response.md`, `cell-provision.md`, `tenant-onboarding.md`, `dsr-cascade-with-evidence.md`, `cve-critical-patch.md`, `region-failover.md`, `security-incident-response.md`, `iam-key-rotation.md`, `capacity-scaling-emergency.md`, `release-rollback.md`, and 126 others.

This is a systemic corpus-level P0 finding. Per documentation-rigor.md §2, a stub does not constitute a valid runbook.

### 4.2 Full runbooks — section coverage audit

**17 of 153 runbooks (11%) are full (non-stub).** Their coverage against the 7-section requirement:

| Runbook | §A | §B | §C (≥10 steps) | §D | §E | §F | §G | Cross-refs ≥2 | Audit tag | Timing | Fail branch | Status FM | Owner FM | ADR FM |
|---------|----|----|------|----|----|----|----|---------------|-----------|--------|-------------|-----------|----------|--------|
| `byok-rotation-provider-tenant-duress.md` | ✓ | ✓ | ✓ 18 | ✓ | ✓ | ✓ | ✓ | ✓ 5 | ✓ | ✓ | ✓ | ✓ prose | ✓ prose | ✓ prose |
| `byok-rotation-encryption-tenant-duress.md` | ✓ | ✓ | ✓ 16 | ✓ | ✓ | ✓ | ✓ | ✓ 3 | ✓ | ✓ | ✓ | ✓ prose | ✓ prose | ✓ prose |
| `bootstrap-ci-compromise.md` | ✓ | ✓ | ✓ 15 | ✓ | ✓ | ✓ | ✓ | ✓ 3 | ✓ | ✓ | ✓ | ✓ prose | ✓ prose | ✓ prose |
| `cell-evacuation.md` | ✓ | ✓ | ✓ 20 | ✓ | ✓ | ✓ | ✓ | ✓ 5 | ✗ | ✓ | ✓ | ✓ prose | ✓ prose | ✓ prose |
| `cedar-fragment-emergency-rollback.md` | ✓ | ✓ | ✓ 22 | ✓ | ✓ | ✓ | ✓ | ✓ 6 | ✗ | ✓ | ✗ | ✓ prose | ✓ prose | ✓ prose |
| `compliance-pack-emergency-suspension.md` | ✓ | ✓ | ✓ 31 | ✓ | ✓ | ✓ | ✓ | ✓ 3 | ✗ | ✓ | ✗ | ✓ prose | ✓ prose | ✓ prose |
| `compliance-pack-revocation.md` | ✓ | ✓ | ✓ 25 | ✓ | ✓ | ✓ | ✓ | ✓ 7 | ✗ | ✓ | ✗ | ✓ prose | ✓ prose | ✓ prose |
| `meta-trust-root-recovery.md` | ✓ | ✓ | ✓ 16 | ✓ | ✓ | ✓ | ✓ | ✓ 5 | ✗ | ✓ | ✓ | ✓ prose | ✓ prose | ✓ prose |
| `self-modification-rollback.md` | ✓ | ✓ | ✓ 20 | ✓ | ✓ | ✓ | ✓ | ✓ 6 | ✓ | ✓ | ✗ | ✓ prose | ✓ prose | ✓ prose |
| `shamir-share-loss-or-coercion.md` | ✓ | ✓ | ✓ 22 | ✓ | ✓ | ✓ | ✓ | ✓ 5 | ✗ | ✓ | ✗ | ✓ prose | ✓ prose | ✓ prose |
| `tenant-data-residency-violation.md` | ✓ | ✓ | ✓ 19 | ✓ | ✓ | ✓ | ✓ | ✓ 6 | ✓ | ✓ | ✓ | ✓ prose | ✓ prose | ✓ prose |
| `provider-credential-leak-response.md` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | ✓ prose | ✓ prose | ✓ prose |
| `flat-crates-move-pr.md` | ✗ | ✗ | ✗ 10 | ✗ | ✗ | ✗ | ✗ | ✗ 0 | ✗ | ✗ | ✓ | ✓ prose | ✓ prose | ✗ |
| `foundry-autonomy-break-glass.md` | ✗ | ✗ | ✗ 7 | ✗ | ✗ | ✗ | ✗ | ✗ 0 | ✗ | ✗ | ✓ | ✓ prose | ✓ prose | ✗ |
| `grit-session-bug-upstream.md` | ✗ | ✗ | ✗ 0 | ✗ | ✗ | ✗ | ✗ | ✗ 0 | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| `per-context-flatten-phase.md` | ✗ | ✗ | ✗ 5 | ✗ | ✗ | ✗ | ✗ | ✗ 0 | ✗ | ✗ | ✓ | ✓ prose | ✗ | ✗ |
| `workspace-members-merge-queue.md` | ✗ | ✗ | ✗ 5 | ✗ | ✗ | ✗ | ✗ | ✗ 0 | ✗ | ✗ | ✓ | ✓ prose | ✗ | ✗ |

**Legend:** ✓ = present / passing; ✗ = absent / failing; "prose" = present in prose frontmatter block (not YAML key-value frontmatter); FM = YAML frontmatter field.

**Summary of full runbook failures:**
- `flat-crates-move-pr.md`, `foundry-autonomy-break-glass.md`, `grit-session-bug-upstream.md`, `per-context-flatten-phase.md`, `workspace-members-merge-queue.md` — all 5 are partial-build runbooks with no section structure, fewer than 10 numbered steps, no cross-references, no timing budgets, no audit-stream tags, no §A-§G sections. These 5 are "full" only in the sense of not containing the standard stub text — they are incomplete runbooks.
- `grit-session-bug-upstream.md` — references grit, a retired tool. Zero steps. Should be retired or repurposed as a historical note.

### 4.3 Frontmatter format inconsistency (P1)

**None of the 153 runbooks uses YAML key-value frontmatter for `status:`, `owner:`, `last-updated:`, or `related-adrs:`.**

The 12 structurally-complete full runbooks use inline prose metadata in the format:
```
> **Status:** Active
> **Owner:** ops-security + axis-intelligence
> **Last updated:** 2026-05-20
> **Related ADRs:** ADR-0255 §D-4, ...
```

While human-readable, this format is not machine-parseable. Per documentation-rigor.md §2 runbook row, frontmatter fields should be parseable by CI tooling. 1 runbook (`sev1-incident-response.md`) has zero frontmatter of any kind.

### 4.4 Audit-stream tag coverage (P1)

Only 6 of 153 runbooks (4%) contain audit-stream tags. Among the 17 full runbooks: 3 fully compliant (`byok-rotation-provider-tenant-duress.md`, `byok-rotation-encryption-tenant-duress.md`, `bootstrap-ci-compromise.md`), 1 partially compliant (`self-modification-rollback.md` and `tenant-data-residency-violation.md`), and 9 have no audit-stream tags despite having §A-§G sections.

Per documentation-rigor.md §2: "every step has … audit-stream tag emitted." The missing audit tags in `cedar-fragment-emergency-rollback.md`, `cell-evacuation.md`, `compliance-pack-emergency-suspension.md`, `compliance-pack-revocation.md`, `meta-trust-root-recovery.md`, `shamir-share-loss-or-coercion.md` are P1 violations.

### 4.5 Timing budget coverage

15 of 153 runbooks contain per-step timing budgets. All 15 are among the 17 full runbooks. The 5 partial-build full runbooks (`flat-crates-move-pr.md`, etc.) have no timing. All 136 stub runbooks have no timing. Among the 12 structurally-complete full runbooks, timing is present in all 12.

### 4.6 "If this step fails" branch coverage (P1)

- `byok-rotation-provider-tenant-duress.md` — ✓ (explicit "If the new credential fails validation (Step 3)…")
- `byok-rotation-encryption-tenant-duress.md` — ✓
- `bootstrap-ci-compromise.md` — ✓
- `cedar-fragment-emergency-rollback.md` — ✗ absent
- `cell-evacuation.md` — ✓ (7 instances)
- `compliance-pack-emergency-suspension.md` — ✗ absent
- `compliance-pack-revocation.md` — ✗ absent
- `meta-trust-root-recovery.md` — ✓
- `self-modification-rollback.md` — ✗ absent
- `shamir-share-loss-or-coercion.md` — ✗ absent
- `tenant-data-residency-violation.md` — ✓

5 of the 12 structurally-complete runbooks have no explicit step-failure branches. This violates the documentation-rigor §2 runbook row requirement.

### 4.7 Cross-runbook reference coverage

- 12 of 153 runbooks have any cross-runbook reference (8%)
- All 12 are among the 17 full runbooks
- Of the 17 full runbooks, all 12 structurally-complete ones have ≥2 cross-references (compliant)
- The 5 partial-build full runbooks have 0 cross-references (non-compliant)
- 136 stub runbooks have 0 cross-references (all non-compliant)

### 4.8 Forbidden patterns

**"Restore from backup" as entire rollback:** 0 violations found across all 153 runbooks.

**"Notify on-call" without channel + escalation list:** 0 violations found. The full runbooks that mention notification specify owners ("ops-security + axis-intelligence + ops-compliance") in prose frontmatter, not without attribution.

**Staleness (dates >12mo cited as current):** No violations. The stub date "2026-05-09" is ~11 days old. Full runbooks cite "Last updated: 2026-05-20".

**Retired tool references in runbooks:** `grit-session-bug-upstream.md` is the one full runbook that references grit by name in a way that implies the reader should understand or use the grit protocol. This runbook has zero steps and should be retired.

---

## §5 Top P0/P1 Findings Across All Three Corpora

### P0 Findings (blockers — actively incorrect or missing)

| # | ID | Corpus | File | Finding |
|---|----|--------|------|---------|
| 1 | P0-MEM-01 | Memory | `MEMORY.md` | `feedback_lifecycle_automation_universal.md` is orphaned — not indexed in MEMORY.md. Contains the lifecycle-automation-universal doctrine (fitness lane for every state machine). Agents will not load it. |
| 2 | P0-MEM-02 | Memory | `MEMORY.md` | `feedback_no_exceptions_canonical.md` is orphaned — not indexed in MEMORY.md. Contains ADR-0083/0105/0107 sunset-clause canonicity rules. |
| 3 | P0-MEM-03 | Memory | Multiple (10+ files) | Active non-superseded memories contain instructional references to retired tools (grit claim/work/done, `rtk icm recall`, `icm store`, `## Grit Claim Symbols`). Agents following these instructions will attempt to use deprecated tooling. |
| 4 | P0-MEM-04 | Memory | `feedback_multispectrum_review_v22.md` + `MEMORY.md` | No memory covers multispectrum-review v2.4.0 (current canonical). Memory index describes v2.2.0 (11-13 facets) as current doctrine. The 21-facet v2.4.0 roster is not represented in any memory file. |
| 5 | P0-SPEC-01 | Specs | 15 files | 15 of 57 specs missing `_meta` block entirely. Includes `chaos-engineering-substrate-canonical.json`, `feature-flag-substrate-canonical.json`, `master-plan-sequencing.json`, `tenant-environment-tiers-canonical.json`, `evidence-taxonomy.json`, and 10 others. |
| 6 | P0-SPEC-02 | Specs | 4 files | 4 specs missing `version` in `_meta`: `tenant-model.json`, `compliance-pack-schema.json`, `cedar-fragment-schema.json`, `final-report-schema.json`. Foundational model `tenant-model.json` with no version cannot be safely evolved. |
| 7 | P0-RUN-01 | Runbooks | 136 files | 89% of the runbook corpus (136/153) are stubs. Critical runbooks including `sev1-incident-response.md`, `cell-provision.md`, `region-failover.md`, `tenant-onboarding.md`, `cve-critical-patch.md`, `iam-key-rotation.md` are stubs with 4 generic placeholder steps. |

### P1 Findings (significant — violate a documented requirement)

| # | ID | Corpus | File(s) | Finding |
|---|----|--------|---------|---------|
| 8 | P1-MEM-05 | Memory | 5 files | Superseded memories (`grit_claim_work_done`, `rtk_proxy_fmt`, `vcs_2026_05_16`, `layer_enum_12_value`, `self_merge_on_ci_green`) interleaved with canonical entries in MEMORY.md with no section break. Risk of agent confusion. |
| 9 | P1-MEM-06 | Memory | 35 files | 66% of feedback memories missing `Why:` and `How to apply:` body sections required by documentation-rigor §2. |
| 10 | P1-MEM-07 | Memory | `feedback_grit_claim_work_done.md` | Description frontmatter says "rtk-ai/grit claim→work→done is the canonical agent-coordination primitive" — not marked SUPERSEDED in its description, only in MEMORY.md. Misleading description. |
| 11 | P1-MEM-08 | Memory | `feedback_workflow_objectgraph_adapter_layer.md` | Body uses "Object Graph" throughout; MEMORY.md marks it RETIRED per ADR-0145; body does not reflect retirement or Ontology rename. |
| 12 | P1-SPEC-03 | Specs | 30 files | 30 specs with `_meta` block missing `enforcement_status`. Includes foundational specs: `decision-principles.json`, `forbidden-operations.json`, `multispectrum-review.json`, `governance-amendment.json`. |
| 13 | P1-SPEC-04 | Specs | 39 files | 39 of 42 specs with `_meta` missing `industry_citations`. Systemic gap. |
| 14 | P1-SPEC-05 | Specs | `compliance-pack-schema.json` | BYOK field drift: contains both deprecated `provider_byok_required` boolean and correct `provider_credential_mode` enum. Old field not removed. |
| 15 | P1-SPEC-06 | Specs | `saga-shape.json`, `throttling-tiers.json`, `tenant-lifecycle.json` | Accepted specs where 100% of properties are missing both `description` and `examples`. Full violation of documentation-rigor §2 spec row. |
| 16 | P1-SPEC-07 | Specs | `masterplan.json`, `gitops-vcs-replacement.json` | Non-standard `status` values (`implemented-masterplan-json-live-hierarchy-index`, `Accepted-for-masterplan-P00`) break enum-based status tooling. |
| 17 | P1-RUN-02 | Runbooks | `cedar-fragment-emergency-rollback.md`, `compliance-pack-emergency-suspension.md`, `compliance-pack-revocation.md`, `shamir-share-loss-or-coercion.md`, `self-modification-rollback.md` | Full §A-§G runbooks missing audit-stream tags in procedure steps. |
| 18 | P1-RUN-03 | Runbooks | `cedar-fragment-emergency-rollback.md`, `compliance-pack-emergency-suspension.md`, `compliance-pack-revocation.md`, `shamir-share-loss-or-coercion.md` | Full §A-§G runbooks missing explicit "if this step fails" branches. |
| 19 | P1-RUN-04 | Runbooks | All 153 | No runbook uses machine-parseable YAML frontmatter for `status:`, `owner:`, `last-updated:`, `related-adrs:`. Prose-inline metadata is not CI-parseable. |
| 20 | P1-RUN-05 | Runbooks | `flat-crates-move-pr.md`, `foundry-autonomy-break-glass.md`, `per-context-flatten-phase.md`, `workspace-members-merge-queue.md` | "Full" (non-stub) runbooks missing §A-§G structure, <10 steps, no cross-references, no timing, no audit tags. |

---

## §6 Recommended Remediation Actions (per file)

### 6.1 Priority order

**Wave 1 (immediate — P0, blocks agent correctness):**

1. **MEMORY.md — add orphaned files** (P0-MEM-01, P0-MEM-02)
   - Add `feedback_lifecycle_automation_universal.md` and `feedback_no_exceptions_canonical.md` to MEMORY.md index under appropriate headings.
   - Estimated effort: 5 min.

2. **MEMORY.md — add multispectrum v2.4.0 memory** (P0-MEM-04)
   - Create `feedback_multispectrum_review_v24.md` documenting the 21-facet canonical roster per `multispectrum-review.json` v2.4.0.
   - Add to MEMORY.md index. Update the v2.2.0 entry to note it is superseded by v2.4.0.
   - Estimated effort: 30 min.

3. **MEMORY.md — segregate superseded entries** (P1-MEM-05)
   - Move the 5 superseded entries to a `## Superseded (history only — do not apply)` section at the bottom of MEMORY.md.
   - Estimated effort: 10 min.

4. **Batch-update active memories with retired tool references** (P0-MEM-03)
   - Target files: `feedback_autonomous_implementation_artifacts.md`, `feedback_autonomous_decision_principles.md`, `feedback_repeat_mistake_prevention.md`, `feedback_doc_coverage_enforced.md`, `feedback_naming_justification.md`, `feedback_clean_architecture_requirements.md`, `feedback_automate_everything.md`, `feedback_milestone_phase_hierarchy.md`.
   - Remove or replace all `grit claim/work/done`, `rtk icm recall`, `icm store`, `## Grit Claim Symbols`, `## ICM Rows to Emit` references with the current Foundry pipeline equivalents (git worktree + gh pr create + oya verify).
   - Estimated effort: 2-3h.

**Wave 2 (high — P0 spec, P1 cross-cutting):**

5. **Add `_meta` blocks to 15 specs missing them** (P0-SPEC-01)
   - Priority order: `master-plan-sequencing.json` (root navigation), `feature-flag-substrate-canonical.json`, `tenant-environment-tiers-canonical.json`, `evidence-taxonomy.json`, `chaos-engineering-substrate-canonical.json`, then remaining 10.
   - Minimum `_meta`: `doc_class`, `spec_id`, `version`, `status`, `purpose`, `owner_team`, `created_at`.
   - Estimated effort: 1-2h for all 15.

6. **Add `version` to 4 specs** (P0-SPEC-02)
   - `tenant-model.json` → `"version": "1.0.0"` (Proposed status)
   - `compliance-pack-schema.json` → `"version": "1.0.0"`
   - `cedar-fragment-schema.json` → `"version": "1.0.0"`
   - `final-report-schema.json` → `"version": "0.1.0"` (draft-sidecar)

7. **Fix BYOK field drift in `compliance-pack-schema.json`** (P1-SPEC-05)
   - Remove `provider_byok_required` boolean field.
   - The correct field is `provider_credential_mode` with enum `{platform_default, byok, byok_required_by_pack}` per ADR-0255 §D-4.
   - If a boolean convenience field is needed, derive it from `provider_credential_mode != 'platform_default'` at query time; do not store it.

8. **Add property `description` + `examples` to `saga-shape.json`, `throttling-tiers.json`, `tenant-lifecycle.json`** (P1-SPEC-06)
   - All properties in these 3 specs need both fields. These are Accepted specs and their property definitions are CI-enforced shapes.

**Wave 3 (systemic — P1, improve compliance posture):**

9. **Add `enforcement_status` to 30 specs with `_meta` but missing it** (P1-SPEC-03)
   - Priority: `decision-principles.json`, `forbidden-operations.json`, `governance-amendment.json`, `multispectrum-review.json`, `oyatie-doctrine.json`.
   - Valid values observed in corpus: `enforced`, `planned`, `draft` — standardize before populating.

10. **Add `industry_citations` to 39 specs** (P1-SPEC-04)
    - This is systemic; recommend a single pass adding at minimum one authoritative citation per spec (NIST, ISO, RFC, academic paper, or industry-canonical reference matching the spec's domain).

11. **Fix `masterplan.json` and `gitops-vcs-replacement.json` status values** (P1-SPEC-07)
    - `masterplan.json`: change status from `implemented-masterplan-json-live-hierarchy-index` to `Accepted` (the freeform string is narrative, not a valid status enum value).
    - `gitops-vcs-replacement.json`: change status from `Accepted-for-masterplan-P00` to `Accepted`.

12. **Add `purpose` to `deployment-ops-contract.json` and `planning-closure-status-closure-ledger.json`** (§3.4)
    - Single-line purpose field in `_meta`.

13. **Add `Why:` and `How to apply:` to high-priority memory files** (P1-MEM-06)
    - Priority subset: `feedback_naming_justification.md`, `feedback_no_silent_regression.md`, `feedback_governance_pipeline_canonical.md`, `feedback_self_merge_via_contract_path.md`, `feedback_git_canonical_2026_05_18.md`, `feedback_layer_enum_adr_0105_13_canonical.md`.

14. **Fix `feedback_grit_claim_work_done.md` description** (P1-MEM-07)
    - Change description to: `"SUPERSEDED 2026-05-16 by [[deprecate-external-agent-coord-tooling]] — grit/rtk/icm retired; Foundry pipeline is the canonical agentic workflow"`

15. **Update `feedback_workflow_objectgraph_adapter_layer.md`** (P1-MEM-08)
    - Add RETIRED note at top of body citing ADR-0145.
    - Update "Object Graph" references to "Ontology" throughout, per `feedback_glossary_ontology_not_object_graph.md`.

**Wave 4 (runbook build-out — requires sustained effort):**

16. **Implement machine-parseable YAML frontmatter for all 153 runbooks** (P1-RUN-04)
    - Define a standard header block:
      ```yaml
      ---
      status: Active | Draft | Deprecated
      owner: <team>
      last_updated: YYYY-MM-DD
      related_adrs:
        - ADR-NNNN
      ---
      ```
    - Apply to all 17 full runbooks first; include in stub template for when stubs are fleshed out.

17. **Add audit-stream tags to 9 full runbooks missing them** (P1-RUN-02)
    - `cedar-fragment-emergency-rollback.md`, `cell-evacuation.md`, `compliance-pack-emergency-suspension.md`, `compliance-pack-revocation.md`, `meta-trust-root-recovery.md`, `shamir-share-loss-or-coercion.md`.
    - Pattern from `bootstrap-ci-compromise.md`: `audit-emit <EventClass> --operator <id> --reason <reason>`.

18. **Add "if this step fails" branches to 5 full runbooks** (P1-RUN-03)
    - `cedar-fragment-emergency-rollback.md`, `compliance-pack-emergency-suspension.md`, `compliance-pack-revocation.md`, `shamir-share-loss-or-coercion.md`, `self-modification-rollback.md`.

19. **Restructure 5 partial-build runbooks** (P1-RUN-05)
    - `flat-crates-move-pr.md`, `foundry-autonomy-break-glass.md`, `per-context-flatten-phase.md`, `workspace-members-merge-queue.md`, `grit-session-bug-upstream.md` — add §A-§G structure, ≥10 steps, cross-references, timing budgets. Or retire `grit-session-bug-upstream.md` as the protocol is deprecated.

20. **Begin stub-to-full promotion for critical runbooks** (P0-RUN-01)
    - Prioritise by operational risk: `sev1-incident-response.md`, `region-failover.md`, `security-incident-response.md`, `cve-critical-patch.md`, `iam-key-rotation.md`, `tenant-onboarding.md`, `cell-provision.md`, `dsr-cascade-with-evidence.md`, `capacity-scaling-emergency.md`, `release-rollback.md`.
    - Use `bootstrap-ci-compromise.md` and `byok-rotation-provider-tenant-duress.md` as canonical quality templates.

---

## §7 Cross-Corpus Consistency Findings

### 7.1 MEMORY.md cites ADR numbers that may be misaligned

MEMORY.md index entries reference the following ADRs by number in their description text:

| MEMORY.md entry | ADR cited | Consistency check |
|----------------|----------|-------------------|
| `[Doc-coverage enforced]` | ADR-0063, ADR-0053 | ADR-0063 cited in `documentation-rigor.md` as a related ADR — consistent |
| `[No silent regression]` | 2026-05-20 keystone bundle ADR-0242..0255 | MEMORY.md says "Reinforced by the 2026-05-20 keystone bundle" — dates consistent |
| `[Layer enum ADR-0105]` | ADR-0105 | `feedback_layer_enum_adr_0105_13_canonical.md` body: in-flight ADR. MEMORY.md says "Active in-flight" — consistent |
| `[Workflow + Ontology = adapter layer]` | ADR-0145 | MEMORY.md says "RETIRED per ADR-0145" — consistent with body |
| `[Self-merge via contract path]` | No ADR cited | Gap: the self-merge contract is described but no binding ADR is cited. The process is governed by `docs/AGENTS.md` Operating Contract. |

### 7.2 Specs cite memories that are superseded or retired

| Spec | Memory referenced | Issue |
|------|------------------|-------|
| `multispectrum-review.json` `_meta.v2_directive` | Refers to v2.0.0 directives; `v2_4_directive` key present but v2.2.0/v2.3.0 memories still listed as current in MEMORY.md | Spec is current at v2.4.0; memories lag by 2 versions |
| `agent-durable-goal.json` `_meta.originating_directive` | References a session directive; no corresponding memory file — directive lives only in the spec `_meta`. No orphan issue but no memory redundancy either | Low risk |

### 7.3 Runbooks reference ADRs not in spec corpus

The full runbooks collectively cite: ADR-0241, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0250, ADR-0251, ADR-0252, ADR-0253, ADR-0254, ADR-0255. These are the 2026-05-20 keystone bundle ADRs (KS#1-KS#14). All are referenced in memory files as well. Cross-corpus consistency is good for this family.

Runbooks also cite ADR-0253 (HTTP/3 + QUIC default) but no spec in `specs/*.json` is directly labelled as implementing ADR-0253. The `brownout-degradation-signal.json` spec cites ADR-0253 implicitly via its HTTP response-header shape — acceptable.

### 7.4 Deprecated tool references create cross-corpus inconsistency

- **Memory** (`feedback_deprecate_external_agent_coord_tooling.md`): grit/rtk/icm/vox are deprecated as of 2026-05-16.
- **Memory** (10+ non-superseded files): Still prescribe grit/rtk/icm usage in procedure sections.
- **Runbooks** (`grit-session-bug-upstream.md`): References grit as if still in use.
- **Specs** (`gitops-vcs-replacement.json`): Refers to the Foundry pipeline as the canonical replacement — consistent with the deprecation.
- **Inconsistency level:** High within the memory corpus; runbook corpus has one instance; spec corpus is consistent with the deprecation.

### 7.5 Multispectrum version chain

| Version | Memory file | Spec | Status |
|---------|------------|------|--------|
| v1.0.0 | — | Superseded by v2.0.0 per `multispectrum-review.json` `_meta.supersedes_version` | Historic |
| v2.0.0 | — | Superseded | Historic |
| v2.1.0 | — | Superseded | Historic |
| v2.2.0 | `feedback_multispectrum_review_v22.md` | Superseded by v2.3.0 | MEMORY.md presents as current |
| v2.3.0 | `feedback_multispectrum_adherence_facets.md` | Superseded by v2.4.0 | MEMORY.md presents as current (A-family adds) |
| v2.4.0 | **NO MEMORY FILE** | `multispectrum-review.json` — current canonical | **Gap** |

An agent loading memory and attempting a multispectrum review will use the v2.2.0 11-13 facet model (the v2.2.0 memory is the most recent one indexed). The 21-facet v2.4.0 model will not be applied. Any reviews produced during this gap will be non-conformant.

### 7.6 Object Graph vs Ontology terminology inconsistency

- `feedback_glossary_ontology_not_object_graph.md`: "Object Graph" renamed to "Ontology" — canonical.
- `feedback_workflow_objectgraph_adapter_layer.md`: Body uses "Object Graph" throughout (old name). MEMORY.md marks this memory RETIRED per ADR-0145 but the body has not been updated.
- `feedback_flat_product_catalog.md`: Body uses "Object Graph" in 6 places.
- `specs/platform-architecture.json`: Uses "object_graph" as a key in policy sections.
- **Inconsistency:** 3 memory files + 1 spec still use the old "Object Graph" terminology. Agents reading these will use the wrong term in new code.

---

## Appendix A — Full spec `_meta` gap table

| Spec | version | purpose | industry_citations | related_adrs | enforcement_status | status |
|------|---------|---------|--------------------|-------------|-------------------|--------|
| active-machine-readable-artifact-contract.json | ✓ 3.0.0 | ✓ | ✗ | ✗ | ✗ | draft-sidecar |
| agent-durable-goal.json | ✓ 1.5.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| agentic-slo-gated-promotion.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✗ | accepted |
| api-surface-separation.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| artifact-profile-defaults.json | ✓ 1.1.0 | ✓ | ✗ | ✗ | ✗ | draft-sidecar |
| brownout-degradation-signal.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| cedar-fragment-schema.json | ✗ | ✓ | ✓ | ✓ | ✗ | Proposed |
| chaos-engineering-substrate-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| ci-fix-loop-context-bundle.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Scaffolded |
| codeview-read-surface.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✗ | Accepted |
| compliance-pack-schema.json | ✗ | ✓ | ✓ | ✓ | ✗ | Proposed |
| crate-naming-audit.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| csi-storage-class-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| decision-principles.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| decision-rights.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| deployment-ops-contract.json | ✓ 1.0.0 | ✗ | ✗ | ✗ | ✗ | Accepted |
| design-spec-maturity-claims.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| dr-business-continuity.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| evidence-taxonomy.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| feature-flag-substrate-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| final-report-schema.json | ✗ | ✓ | ✗ | ✗ | ✗ | draft-sidecar |
| finops-cost-attribution.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| forbidden-operations.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| gitops-vcs-replacement.json | ✓ 1.7.0 | ✓ | ✗ | ✗ | ✗ | Accepted-for-masterplan-P00 |
| governance-amendment.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| hyperscaler-architecture-invariants.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| hyperscaler-gates.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| industry-best-practice-conformance.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✗ | accepted |
| iterative-fix-loop.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| knowledge-graph-schema.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | draft-sidecar |
| markdown-retirement-policy.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| master-plan-sequencing.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| masterplan.json | ✓ 1.1.0 | ✓ | ✗ | ✗ | ✗ | implemented-…[non-standard] |
| merge-queue-parked-pr.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Scaffolded |
| microservice-migration-tooling.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✗ | accepted |
| multi-region-disposition-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| multispectrum-review.json | ✓ 2.4.0 | ✓ | ✗ | ✓ | ✗ | Accepted |
| oyatie-doctrine.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| per-microservice-flat-layout.json | ✓ 1.0.0 | ✓ | ✓ | ✓ | ✗ | accepted |
| per-tenant-audit-log-slicing-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| plan-schema.json | ✓ 1.1.0 | ✓ | ✗ | ✗ | ✗ | draft-sidecar |
| planning-closure-contract.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| planning-closure-status-closure-ledger.json | ✓ 1.0.0 | ✗ | ✗ | ✗ | ✗ | accepted |
| platform-architecture.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Proposed |
| root-hub-pointers.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| saga-shape.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| schema-registry-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| score-cards.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| sovereign-cloud-air-gapped-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| sovereign-cloud-overlays.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| stop-conditions.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| tenant-environment-tiers-canonical.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |
| tenant-lifecycle.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| tenant-model.json | ✗ | ✓ | ✓ | ✓ | ✗ | Proposed |
| test-standard.json | ✓ 1.0.0 | ✓ | ✗ | ✗ | ✗ | Accepted |
| throttling-tiers.json | ✓ 1.0.0 | ✓ | ✗ | ✓ | ✓ | Accepted |
| workspace-hygiene.json | ✗ NO META | ✗ | ✗ | ✗ | ✗ | — |

**Totals:** version ✓ 37 / ✗ 20 | purpose ✓ 40 / ✗ 17 | industry_citations ✓ 5 / ✗ 52 | related_adrs ✓ 21 / ✗ 36 | enforcement_status ✓ 12 / ✗ 45

---

## Appendix B — Runbook stub list (136 files)

The following 136 runbooks are stubs and do not meet the documentation-rigor §2 runbook row requirements. They contain the marker:
`> **Last verified:** 2026-05-09 (stub authored to satisfy doc-link integrity; full procedure lands at W-Foundation gate)`

Prioritised by operational risk class:

**SEV-1 / incident response (highest urgency to flesh out):**
`sev1-incident-response.md`, `security-incident-response.md`, `region-failover.md`, `capacity-scaling-emergency.md`, `cell-failover-intra-region.md`, `cell-isolation-breach.md`, `cell-tier-promotion.md`, `supply-chain-compromise.md`, `supply-chain-trivy-alert.md`, `cve-critical-patch.md`, `api-gateway-rate-limit-incident.md`, `error-budget-exhaustion.md`, `release-rollback.md`, `outbox-poller-recovery.md`, `outbox-relay-lag.md`

**Tenant / data lifecycle (regulatory risk):**
`tenant-onboarding.md`, `dsr-cascade-with-evidence.md`, `dsr-cascade-orchestration.md`, `dsr-cascade-proof-of-erasure.md`, `dsr-compliance-report.md`, `consent-withdrawal-cascade.md`, `data-class-transition-approval.md`, `tenant-escalation-management.md`, `cross-pack-tenant-residency.md`

**IAM / key management:**
`iam-key-rotation.md`, `identity-provider-federation.md`, `per-cell-hsm-rotation.md`, `og-ciphertext-key-shred.md`

**Compliance / regulatory:**
`compliance-pack-emergency-suspension.md` (listed as full but missing some fields — see §4.2), `regulatory-change-response.md`, `regulatory-replay.md`, `regulator-evidence-pack-regen.md`, `breach-notification.md`, `breach-notification-council-escalation.md`

**Foundry / agentic pipeline:**
`foundry-platform-incident.md`, `foundry-model-cutover.md`, `foundry-fitness-rollback.md`, `foundry-capability-publish.md`, `foundry-sandbox-escape.md`, `foundry-autonomy-policy-rollback.md`, `claim-ceiling-bypass-expiry.md`

**Remaining 86:** Various operational, vertical (healthcare, fintech, logistics, industrial), and maintenance runbooks — all stubs. Full list available via `grep -rl "stub authored" docs/runbooks/`.

---

*End of audit report. 153 runbooks, 57 specs, 54 memory files examined. Zero files modified. Audit-only.*
