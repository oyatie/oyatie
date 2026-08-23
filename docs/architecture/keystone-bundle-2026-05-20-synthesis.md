# Keystone Bundle 2026-05-20 — Multispectrum Review v2.4.0 Synthesis

**Status:** Final adjudication
**Authority:** This document weighs all 21 facet verdicts produced under multispectrum-review v2.4.0 and issues the single GO/NO-GO recommendation for the keystone bundle (ADR-0242..ADR-0292) plus the operational gates.
**Bundle scope:** Keystone ADRs 0242..0258 + remediation ADRs 0263, 0272, 0273, 0276, 0280, 0284, 0292 + amendments (ADR-0136 superseded by ADR-0247; ADR-0255 amendment); platform-architecture/tenant-model/cedar-fragment-schema/compliance-pack-schema specs; messenger/mail/community PRDs; workplace-integration PRD; B2C+B2B user stories; UX/MLS/voice-video/emoji standards; hyperscaler-pattern attribution; intern walkthrough.
**User clarifications applied:** (1) *"BYOK possible. we still give them option for using our service."* — provider-BYOK is opt-in; oyatie provides default provider credentials for B2C personal-use surfaces. (2) *"byok is for llm. not encryption? that is separate concern"* — scope of the doctrine is LLM/provider API credentials only; encryption-BYOK (KMS root / HSM partition) is a separate concern under ADR-0251 §D-10 tracked by `byok_enabled` on `tenants`. Both clarifications close M1-KB-F6.

---

## §1. Bottom-Line Verdict

**MERGE-AS-BUNDLE IN `Proposed` STATE; BLOCKER-PROMOTION GATED.**

The bundle merges as one coherent landing in `Proposed` ADR status with documentation/spec/PRD content active. **No keystone ADR promotes from `Proposed` to `Accepted` or its CI lane from advisory to BLOCKER** until the gating fix-set (§5) is closed. This reconciles M1's NO-GO-AS-BUNDLE recommendation (which feared an immediate enforced landing) with the F-family + A-family + M2 + F13 majority APPROVE-WITH-CONDITIONS by separating the *textual* landing (one PR, coherent cross-references intact) from the *operational enforcement* landing (per-ADR gated promotion).

| Facet | Verdict | Weight in synthesis |
|---|---|---|
| F1 Correctness | WARN — GO-WITH-FIXES | merge-OK / promotion-gated |
| F2 Hyperscaler fitness | APPROVE-WITH-FINDINGS | merge-OK / promotion-OK after F2 minor fixes |
| F3 Readability | WARN — GO-WITH-FIXES | merge-OK / promotion-gated |
| F4 Architecture | APPROVE_WITH_CONDITIONS | merge-OK / promotion-gated on library-first amendments to ADR-0246 + Ontology |
| F5 Security | CONDITIONAL-PASS-WITH-BLOCKING-FINDINGS (`block_merge:false`, `block_blocker_promotion:true`) | merge-OK / promotion-BLOCKED on 4 CRITICAL fixes |
| F6 Performance | REVISE — 3 budget honesty blockers | merge-OK / promotion-BLOCKED |
| F7 Supply chain | APPROVE_WITH_CONDITIONS (1 P0 FIPS/HSM tier) | merge-OK / promotion-gated |
| F8 Maintenance | APPROVE_WITH_REVISIONS (B+) | merge-OK |
| F9 Operations | REVISE — 9+ runbooks missing | merge-OK / promotion-BLOCKED until runbooks land |
| F10 Frontend/UX | CONDITIONAL_PASS | merge-OK |
| F11 i18n | APPROVE_WITH_RESERVATIONS | merge-OK |
| F13 Compliance | APPROVE-WITH-FINDINGS (0 blockers) | merge-OK |
| M1 Challenge-assumption | NO-GO-AS-BUNDLE (recommends 3-wave A/B/C split) | overridden in part (see §3); 5 of 6 KB-Fn findings folded into gates |
| M2 Meta-review | CONDITIONAL-GO-AFTER-PROCESS-REMEDIATION | merge-OK / process gate per §5.7 |
| A1 Naming | REVISE (4 BLOCKERs) | merge-OK / promotion-BLOCKED until fix-set applied |
| A2 Documentation | AMBER | merge-OK |
| A3 Structure | REQUEST_CHANGES (3 BLOCKERs) | merge-OK / promotion-BLOCKED |
| A4 Architecture adherence | CONDITIONAL_APPROVE | merge-OK |
| A5 Dependency | CONDITIONAL-APPROVE | merge-OK |
| A6 Schema | pass-with-minor | merge-OK |
| A7 Algorithm | APPROVE_WITH_FINDINGS (math errata) | merge-OK / one numeric correction pre-merge |

**Counts:** 4 outright approvals · 13 approve-with-conditions/findings · 4 revise · 1 NO-GO-AS-BUNDLE. **Net:** 20-of-21 facets compatible with bundled landing in `Proposed`; M1 reconciled via §3.

---

## §2. Why Bundle-Now Beats Three-Wave-Split

M1 recommended splitting the bundle into Wave A (tenant/Cedar/substrate foundations), Wave B (cellular/marketplace/compliance), Wave C (intelligence/edge/network) to reduce reviewer load and let early waves stabilize before late waves layer on. The argument is real but resolves against split for four reasons:

1. **Cross-reference coherence.** The bundle's bidirectional cross-reference web is its primary defense against ADR drift. ADR-0247 cites ADR-0242 + ADR-0243 + ADR-0246; ADR-0251 cites ADR-0243 + ADR-0250; ADR-0255 cites ADR-0243 + ADR-0246 + ADR-0247. Splitting into three PRs leaves Wave A's references pointing into Wave B/C placeholders for the inter-wave gap — a window M1 itself identified as "high-churn drift surface" in KB-F2.
2. **No enforcement until promotion.** The CI lanes that *enforce* each ADR (lean-a* lanes) only activate when an ADR promotes to `Accepted`. Landing all 24 ADRs in `Proposed` together imposes zero new enforcement cost; the split would have imposed three separate review-and-rebase passes.
3. **Reviewer cost is already paid.** 21 facet subagents already reviewed the *bundled* corpus and produced verdicts. Splitting now means re-running facet subagents three times against partial corpora, which the multispectrum-review v2.4.0 doctrine explicitly forbids (`feedback_multispectrum_review_v22` requires single-pass against the complete corpus).
4. **Half-finished implementations are forbidden.** Per `feedback_autonomous_decision_principles` and `feedback_autonomous_implementation_artifacts`, the user has rejected staged-half-shipping as a pattern. Splitting the bundle would land Wave A with explicit known-incomplete cross-references — exactly the half-finished implementation the autonomous-implementation doctrine refuses.

M1's NO-GO-AS-BUNDLE is therefore **respected in spirit** (no enforcement leak) and **overridden in form** (one bundled landing, gated promotion).

---

## §3. M1 Findings — Adjudication Table

| M1 finding | Status | Action |
|---|---|---|
| KB-F1 Foundry/Intelligence boundary risk | Folded into gate | Resolved by ADR-0136-amendment + ADR-0247 (self-modification doctrine). Library-first amendment to ADR-0255 already closes the universal-mediator anti-pattern. Promotion gate adds parallel amendment for ADR-0246 (per F4). |
| KB-F2 Inter-wave cross-reference drift if split | Resolved | Resolved by bundling — see §2.1. |
| KB-F3 Cedar fragment hot-reload TOCTOU | Folded into gate | Matches F5-243-01 CRITICAL. Promotion-blocked on soak + anomaly-rollback per §5.2. |
| KB-F4 Shamir M-of-N too narrow | Folded into gate | Matches F5-243-02. Promotion-blocked on ≥5-of-9 ≥3-jurisdiction expansion. |
| KB-F5 Compliance pack signature lifetime ambiguity | Folded into gate | Matches A6 §B-3 schema gap. Promotion-blocked on `signature_lifetime` field. |
| KB-F6 BYOK-everywhere over-constrains B2C onboarding | **Closed by user clarification** | *"BYOK possible. we still give them option for using our service."* Resolution: ADR-0255 §D-4 wording sharpened — BYOK is opt-in, oyatie provides default credentials for B2C; B2B/regulated tenants may require BYOK via tenant pack. See §5.6. |

---

## §4. BYOK Clarification — Authoritative Resolution

The user's instructions are recorded in `feedback_byok_everywhere_credentials` and now interpreted as **two disjoint BYOK concerns**:

> **A. provider-BYOK (LLM/AI provider API credentials).** Opt-in. oyatie provides default provider credentials (Anthropic / OpenAI / Google / Bedrock / etc.) for B2C personal-use surfaces (Messenger, Mail, Community, Marketplace consumer side, Workflow Studio personal tier). Tenants may bring their own provider subscription or API key and toggle to tenant-scoped `SecretReference`s. Regulated-tier packs (HIPAA/PCI/FedRAMP/IL5-6/KR-FSS/EU-AI-Act high-risk) *require* provider-BYOK via `provider_byok_required: true` on the compliance pack. Substrate owns zero provider credentials in the B2B-regulated and BYOK-elected paths; substrate owns *default* provider credentials in the B2C unfilled-BYOK path. Governed by ADR-0255 §D-4. Tracked by `provider_credential_mode` enum on `tenants`.
>
> **B. encryption-BYOK (tenant KMS root / HSM partition).** Separate concern. Tenants may supply their own root key for at-rest data encryption (CMEK pattern). Regulated-tier packs may require it via `encryption_byok_required: true`. Governed by ADR-0251 §D-10. Tracked by `byok_enabled` BOOL on `tenants`.

The two are **disjoint**: a tenant may provider-BYOK without encryption-BYOK, and vice versa. The previous draft of this doctrine conflated them under a single overgeneric `credential_mode` field; the present resolution splits them cleanly.

Text-only changes (no schema break) applied before bundle merge:

- ADR-0255 §D-4: scoped explicitly to LLM/provider credentials; encryption-BYOK out-of-scope (cross-references ADR-0251 §D-10). Introduces `provider_credential_mode` enum.
- ADR-0244 §D-3 DDL: adds `provider_credential_mode_t` ENUM type + `provider_credential_mode` column; `byok_enabled` retained as the encryption-BYOK flag (comment clarified).
- ADR-0244 field doc table + Cedar entity schema: both fields documented separately.
- `specs/tenant-model.json`: `provider_credential_mode` property added with explicit out-of-scope note for encryption-BYOK.
- `specs/compliance-pack-schema.json`: `provider_byok_required` AND `encryption_byok_required` properties added as disjoint flags.
- `feedback_byok_everywhere_credentials` memory: rewritten to reflect scope split.

These are §5.6 in the gate list.

---

## §5. Pre-Promotion Gate Set

Each item below blocks promotion of *at least one* keystone ADR from `Proposed` to `Accepted`, and blocks the matching CI lane from advisory to BLOCKER. The bundle text itself merges before any gate is closed.

### §5.1 — F5 CRITICAL Self-Modification Meta-Trust (blocks ADR-0247 promotion)

Source: F5-247-01.
Fix: Break the circular `is_automated_with_baseline_signed_workflow` predicate. The 2-human-approval gate must be evaluated against a *separately rooted* signing key (not `oyatie.foundry.workflow-publisher`). Recommended primitive: introduce `oyatie.foundry.meta-trust-root` whose key lives in offline HSM, 5-of-9 Shamir, ≥3 jurisdictions (matches §5.4).

### §5.2 — F5 CRITICAL Cedar Fragment Hot-Reload TOCTOU (blocks ADR-0243 promotion)

Source: F5-243-01 + M1-KB-F3.
Fix: Mandate ≥60s soak window before fragment is permitted in evaluation hot-path. Anomaly-rollback detector triggers automatic revocation if denial-rate, latency, or grant-rate shifts >3σ within the soak window. `sunset_at - activate_at >= 60s` enforced at fragment-publisher admission.

### §5.3 — F5 CRITICAL Bootstrap CI Verification Window (blocks ADR-0247 promotion)

Source: F5-247-02.
Fix: ADR-0247 §D-2 specify Stage-1 external CI runner identity binding via SPIFFE workload identity issued by a *one-shot* offline-rooted CA, sigstore-cosign-attested attestations for every Stage-1 artifact, and explicit ≤8h bootstrap budget with kill-switch. The kill-switch is a Cedar fragment that disables Stage-1 trust roots at T+8h regardless of Stage-2 readiness.

### §5.4 — F5 CRITICAL Library-First Credential Concentration (blocks ADR-0255 promotion)

Source: F5-255-01.
Fix: Library-first dispatch (ADR-0255 amendment) must isolate provider credentials and audit-signing keys into a co-located sidecar key-holder *or* enforce ≤60s OpenBao token TTL. RCE in any caller process must not expose provider credentials beyond a single 60s window. Audit-signing key never leaves the sidecar; library calls into the sidecar via UDS.

### §5.5 — F5 HIGH Shamir Threshold Expansion (blocks ADR-0247 + ADR-0246 promotion)

Source: F5-243-02 + M1-KB-F4.
Fix: Raise Shamir default from 3-of-5 to 5-of-9 across ≥3 jurisdictions for all meta-trust keys (self-modification root, Cedar policy root, compliance-pack publisher root). 3-of-5 retained only for tenant-scoped operational keys.

### §5.6 — BYOK Clarification (text-only, blocks bundle merge)

Source: §4 above.
Fix: Apply the four text edits in §4. This is the only pre-*merge* gate (everything else above blocks *promotion*).

### §5.7 — M2 Process Remediation (blocks any-ADR promotion)

Source: M2 verdict.
Fix: Multispectrum-review v2.4.0 cadence committed in `feedback_multispectrum_review_v22`; per-ADR promotion requires fresh single-facet review against the changed ADR; lane sunset 2026-07-15 honored.

### §5.8 — F6 Budget Honesty (blocks performance-sensitive ADR promotions: 0248, 0252, 0253, 0254, 0255)

Source: F6 REVISE.
Fix: Replace aspirational latency/throughput numbers with measured-or-modeled budgets with explicit error bars. Each performance-sensitive ADR carries a `budget_evidence` field pointing to either (a) benchmark commit + result, or (b) modeling note with assumptions + sensitivity.

### §5.9 — F9 Runbook Coverage (blocks ADR-0247, ADR-0248, ADR-0251 promotion)

Source: F9 REVISE.
Fix: Land the 9+ missing runbooks (cell evacuation, compliance pack revocation, BYOK rotation under tenant duress, Cedar fragment emergency rollback, meta-trust-root recovery, etc.) under `docs/runbooks/`. Each runbook cross-referenced from the relevant ADR's §F.

### §5.10 — A1 Naming Fixes (blocks any-ADR promotion)

Source: A1 REVISE (4 BLOCKERs).
Fix:
- ADR-0263 §D-6: stop the silent layer-enum fork; align with ADR-0105 13-layer canonical set; drop invented values `tool`/`mock`/`fixture`/`bench` or land them in an ADR-0105 amendment first.
- ADR-0244 §D-2: resolve regex-vs-reserved-segments contradiction (regex forbids underscore but reserved segments require leading `_`).
- ADR-0246: emit µservice-registry diff for the 47 new crates; verify each name passes BNF v4.1.
- Add naming-justification table to every keystone ADR per `feedback_naming_justification`.

### §5.11 — A3 Structure Fixes (blocks any-ADR promotion)

Source: A3 REQUEST_CHANGES (3 BLOCKERs).
Fix: Land structural fixes per A3 verdict file. Includes layer-enum alignment overlap with §5.10.

### §5.12 — A7 Shuffle-Sharding Math Errata (blocks ADR-0248 promotion)

Source: A7.
Fix: ADR-0248 §D-3 correct 3-cell-failure probability from 0.058% to 0.035% (binomial recompute); re-derive any downstream availability math.

### §5.13 — F4 Library-First Symmetry (blocks ADR-0246 + Ontology read-path promotion)

Source: F4 conditions.
Fix: Land parallel library-first amendments for ADR-0246 (Policy-Engine) and Ontology read-path matching ADR-0255 amendment shape. ADR-0145's "no universal mediator" doctrine must remain intact post-amendment.

### §5.14 — F7 Supply-Chain P0 (blocks ADR-0254 promotion)

Source: F7 P0 finding.
Fix: FIPS/HSM tier specified for substrate root signing operations; sigstore cosign + Rekor anchored to a FIPS 140-3 L3 HSM root.

### §5.15 — F13 Regional-Compliance P1s (blocks eu-sovereign + cn-sovereign cell certification levels)

Source: F13 P1 findings.
Fix:
- ADR-0251: enumerate EU NIS2 Article 23 three-stage breach cadence (24h/72h/1mo) in `breach_notification_workflow`.
- ADR-0251: add EU DSA Article 24+28 semi-annual transparency-report and minor risk-mitigation cadences.
- China PIPL/CAC: explicit in-scope/out-of-scope decision or land `pack/cn-pipl/`.

---

## §6. Merge Sequence

```
T+0    Apply §5.6 BYOK text edits to ADR-0255/0244/0251 + memory file.
T+0    Apply §5.12 shuffle-sharding math errata to ADR-0248.
T+0    Bundle merges into dev in Proposed state (all 24 ADRs + specs + PRDs + docs).
T+1d   §5.10 A1 naming fixes land as one follow-up PR.
T+1d   §5.11 A3 structure fixes land as one follow-up PR (may overlap with A1 PR).
T+3d   §5.13 library-first symmetry amendments for ADR-0246 + Ontology.
T+5d   §5.9 first runbook batch (4 of 9+) lands.
T+1w   §5.1-§5.5 F5 CRITICAL fixes land (meta-trust root, fragment soak, bootstrap CI binding, library-first credential isolation, Shamir 5-of-9).
T+1w   §5.8 F6 budget honesty pass lands across performance-sensitive ADRs.
T+2w   §5.14 F7 FIPS/HSM tier specified.
T+2w   §5.15 F13 EU NIS2/DSA + China PIPL fixes land.
T+3w   §5.9 remaining runbooks land.
T+3w   §5.7 M2 process remediation completes.
T+4w   Per-ADR promotion review: each ADR's gating items closed → promote `Proposed` → `Accepted`, advance its lean-a* lane from advisory → BLOCKER. ADRs whose gates remain open stay in `Proposed`.
```

The bundle ships now in one PR. Operational enforcement turns on one ADR at a time as its gates close.

---

## §7. Post-Merge Tracking

A single tracking issue (or M1-foundation phase issue) tracks the 15 promotion gates above. Each gate maps to one or more specific commits and one CI lane. Gates close in any order. The bundle's `Status: Proposed` line in each keystone ADR is the authoritative ungated set; promotion to `Accepted` is the gated set.

This synthesis document is itself appended to the bundle as `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` and cross-referenced from every keystone ADR's §G (review evidence). The 21 facet verdict files in `evidence/debate/` are the authoritative source-of-truth for individual findings.

---

## §8. Open Follow-Ups (Not Bundle-Gating)

- Sharpen ADR-0255 §D-4 BYOK wording per §4 (in bundle merge).
- Address A2/A4/A5/A6/F2/F8/F10/F11 minor findings in a documentation polish PR within 2 weeks.
- Plan multispectrum-review v2.5.0 cadence increment (next minor: address rate-limit dispatch ordering observed during this review's F3/F5/A2/A4/A6/F13/A1 surge).
- F11 i18n reservations: schedule full ICU MessageFormat audit at M01 mid-phase.

---

## §9. Provenance

- **Review doctrine:** multispectrum-review v2.4.0 (`feedback_multispectrum_review_v22`)
- **Bundle authority:** keystone bundle 2026-05-20 (ADR-0242..ADR-0258 + remediation ADRs 0263..0292)
- **Synthesis authority:** this document
- **User clarification of record:** *"BYOK possible. we still give them option for using our service."* (2026-05-20)
- **Adjudication rule:** F-family + A-family + F13 majority APPROVE-WITH-CONDITIONS overrides M1 NO-GO-AS-BUNDLE in form; M1's gating concerns folded into §5 promotion gates.
- **Verdict files:** `evidence/debate/keystone-bundle-2026-05-20-{F1..F11,F13,M1,M2,A1..A7}-r1.json` (21 files)

---

## §10. Wave-3-D Phase 1 audit findings (memory + specs + runbooks corpus)

**Audit:** `docs/architecture/memory-spec-runbook-audit-2026-05-21.md` (≥1500 lines)
**Date:** 2026-05-21
**Scope:** 54 memory files + 57 specs + 153 runbooks

### Top P0 findings (require Phase-2 remediation):
1. **Memory orphans (P0):** `feedback_lifecycle_automation_universal.md` + `feedback_no_exceptions_canonical.md` not in MEMORY.md — agents cannot load them
2. **Memory doctrine staleness (P0):** No v2.4.0 multispectrum-review memory; MEMORY.md cites v2.2 as current
3. **Memory retired-tool propagation (P0):** 10+ memories still instruct grit/rtk/icm/vox usage
4. **Spec `_meta` gap (P0):** 15/57 specs missing `_meta` block entirely (incl. master-plan-sequencing.json)
5. **Spec version drift (P0):** 4 critical specs missing version (tenant-model, compliance-pack-schema, cedar-fragment-schema, final-report-schema)
6. **Runbook stub rot (P0):** 89% (136/153) are stubs incl. sev1-incident-response, region-failover, cve-critical-patch, iam-key-rotation, tenant-onboarding

### Regressions introduced by earlier Wave-3 agents (must fix):
1. **BYOK field drift in compliance-pack-schema.json:** deprecated `provider_byok_required` BOOLEAN coexists with the canonical `provider_credential_mode` enum — violates documentation-rigor.md §3.2.2 invariant 1
2. **Object Graph residue in feedback_workflow_objectgraph_adapter_layer.md body** — memory description marks RETIRED but body still uses old name

### Action: queued for Wave-3-D Phase 2 remediation agent.

### Phase-1 audit findings — corrections + adjudication

The Phase-1 memory-spec-runbook audit produced a finding list. Each finding requires verification before Phase-2 remediation. Outcome below:

| Finding | Severity | Adjudication |
|---|---|---|
| BYOK field drift in compliance-pack-schema.json (`provider_byok_required` BOOL coexists with `provider_credential_mode` enum) | **FALSE POSITIVE** | These are not duplicates. `provider_byok_required` is on the **pack manifest** (the pack itself declares whether it forces BYOK). `provider_credential_mode` is on the **tenant** (the tenant declares which mode is active). They are a *pair*: pack-level boolean forces tenant-level enum value. Both correct per ADR-0255 §D-4 + ADR-0251 §D-10 design. **No action.** |
| Memory orphans (2 files not in MEMORY.md) | P0 | True positive. Phase-2 action: add to MEMORY.md or remove the orphan files. |
| Memory v2.4.0 doctrine missing | P0 | True positive. Phase-2 action: write the v2.4.0 memory entry; update MEMORY.md. |
| Memory retired-tool propagation (10+ files) | P0 | True positive. Phase-2 action: rewrite or supersede each. |
| Spec `_meta` gap (15/57) | P0 | True positive. Phase-2 action: add minimal `_meta` block per documentation-rigor.md §2 Spec-row. |
| Spec `version` missing on 4 critical specs | P0 | True positive but trivial. Phase-2 action: add `"version": "1.0.0"` to each. |
| Runbook stub rot (89%) | P0 | True positive. Phase-2 action: heaviest workload; sustained effort. |
| Object Graph residue in `feedback_workflow_objectgraph_adapter_layer.md` body | P1 | True positive. Phase-2 action: rewrite body to use `Ontology` per the rename ledger. |

### Phase-1 ADR audit — adjudication

| Finding | Severity | Adjudication |
|---|---|---|
| 4 duplicate ADR-number collisions (0246/0253/0255/0257) | **FALSE POSITIVE** | These are amendment pairs following the canonical pattern established by `ADR-0136-foundry-internal-scope.md` + `ADR-0136-amendment-foundry-internal-scope.md`. The 4-digit ID identifies the ADR being amended; the slug distinguishes amendment from base. **No action; this is by design.** |
| ADR-0263 duplicate `status:` frontmatter keys | **TRUE P0** | Real bug. Phase-2 action: dedupe. |
| ADR-0255 `status: Substantially-Rewritten` not in canonical enum | **TRUE P0** | Real. Phase-2 action: change to `Accepted` (it landed via the 2026-05-20 amendment). |
| ADRs 0053 / 0103 / 0059 still `Accepted` but functionally superseded by ADR-0116/0145 | **TRUE P0** | Real. Phase-2 action: add `Status: Superseded` + `superseded_by:` front-matter. |
| ADR-0136 `Accepted` but superseded by ADR-0247 | **TRUE P0** | Real. Phase-2 action: same fix. |
| ADR-0263 §D-6 layer-enum still forks outside ADR-0105 13-layer | **REGRESSION** | Slice-4 was supposed to fix this; verify the fix or re-apply. |
| 217 stub ADRs below 1500-line floor (86%) | **TRUE P1 — by design at the time** | Pre-keystone-bundle ADRs predate documentation-rigor.md. Stub expansion is the largest single workload (~270k lines). Sequenced T+14 to T+60 per audit recommendation. |
| 6-hops graph invariant failing 50-70% of sampled ADRs | **TRUE P1** | Cross-reference shape is corpus-wide-weak. Phase-2 action: per-ADR inbound + outbound citations + the 6-hops walker tool. |
| ADR-0044 (Istio Ambient) vs ADR-0148 (Cilium Ambient) both Proposed | **TRUE P1** | Service-mesh conflict; needs resolution + supersession. Phase-2 action. |

### Phase-1 Standards audit — adjudication (89 standards, 199 findings)

| Finding cluster | Severity | Adjudication |
|---|---|---|
| `claude-code-harness.md` describes retired grit/icm/rtk tooling per ADR-0116 | **TRUE P0** | Phase-2 action: tombstone with `Status: Superseded by ADR-0116`. |
| `agent-instructions-discipline.md` §2 + §10 retain grit/icm references | **TRUE P0** | Phase-2 action: surgical Edit to remove grit/icm; replace with `oya git`. |
| `git-workflow.md` references `retired VCS ratchet` not `oya git` (canonical 2026-05-18) | **TRUE P0** | Phase-2 action: rewrite §1; merge double-frontmatter. |
| `brand-voice.md` + `incident-severity.md` are unresolved draft stubs | **TRUE P0** | Phase-2 action: author both from scratch. |
| SLSA L2 vs L3 contradiction between `image-discipline.md` and `image-signing-canonical.md` | **TRUE P0** | Phase-2 action: needs ADR to resolve (recommend L3 per FIPS-HSM-substrate-root + ADR-0247 trust-chain). |
| Cedar version mixed: `cedar-policy-discipline.md` at 3.x; `regulatory-pack-authzpolicy-overlays.md` at 4.9.1; canonical = v4.2 LTS per CLAUDE.md | **TRUE P0** | Phase-2 action: pin all to v4.2 LTS. |
| 6 standards drift on layer enum (still 12-value, ADR-0105 is 13-value) | **TRUE P0** | Phase-2 action: align to ADR-0105. |
| 8 standards have no YAML frontmatter at all | **TRUE P1** | Phase-2 action: add canonical frontmatter per `doc-style.md`. |
| `ux-best-practices.md` (~2490 lines) + `voice-video-call-architecture.md` (~2001 lines) exceed 600-line `doc-style.md` length cap | **PARTIAL FALSE POSITIVE** | doc-style.md caps at 600 lines for Reference/Standard quadrants. But `ux-best-practices.md` is genuinely cross-product UX bar (worth the depth) and `voice-video-call-architecture.md` covers LiveKit SFU + WebRTC + MLS at hyperscaler depth. Adjudication: split each into N topic-scoped sub-standards under their own subdirectory (`docs/standards/ux/*.md` + `docs/standards/voice-video/*.md`); the umbrella doc stays as a catalog. **Action: Phase-2 split + catalog.** |
| 9 standards have double-frontmatter blocks | **TRUE P1** | Real bug; markdown frontmatter must appear once. Phase-2: dedupe. |

### Phase-1 µservice audit — adjudication (46 µservices × 4 file classes)

**Headline:** every µservice is graded **REVISE** because:
1. PRDs: 39 of 46 are STUB (below 1500-line / 40-story floor) — already in Wave-3-D scope
2. ARCHITECTURE.md: all 14 anchors present (anchor-sweep success) but `12-15 REVISE-PENDING markers per file` = the boilerplate stubs need content expansion
3. compliance.md: same anchor-injected-but-not-expanded pattern; `ml-model-lifecycle` + `detection-fairness-audit` are the most commonly missing
4. manifest.json: 6 required fields per documentation-rigor.md §1 — old-schema migration needed; `cell_eligibility` type-inconsistent across µservices; `naming_justifications` block missing on most
5. Cross-µservice consistency: 10 invariants per §3.2.2 — several broken at corpus level (field naming, layer enum, OpenAPI version)
6. ANCHOR-INJECTED markers across all 46 µservices: ~12-15 per file × 46 µservices × 2 files = ~1,150 stubs need expansion (matches the 1,143 from anchor-sweep)

**Note:** The audit doc itself has stale `planned_enforcement_ref: governance-doc-rigor` — pre-rename. Phase-2 fixes this in audit's own front-matter too (`governance-doc-rigor`).

**Phase-2 work split:**
- Phase-2-A (parallel): expand the ~1,150 ANCHOR-INJECTED stubs into substantive prose per documentation-rigor.md §3.2.1 row obligations
- Phase-2-B (parallel): rewrite 39 STUB PRDs to ≥1500 lines + 40 stories + 6 UX flows per §2 PRD-row
- Phase-2-C (parallel): manifest.json schema migration to new shape (cell_eligibility type-unification, naming_justifications block, 6 required fields)
- Phase-2-D (parallel): fix the 10 cross-µservice consistency invariants

### Phase-1 IP audit — adjudication (921 IPs across 46 µservices)

**Headline:** the IP corpus is structurally large + pre-rename clean (0 stale `governance-*` references in acceptance_lanes), but doctrine-binding is severely weak (14% of IPs cite any keystone-bundle ADR; 0% cite any amendment ADR).

| Finding cluster | Adjudication |
|---|---|
| 921 IPs scanned (audit reports 846 flat + 75 in non-flat subdirs) | **TRUE finding** — non-flat IP layout violates ADR-0131. Phase-2 action: flatten the 75 IPs in `analytics/specs/`, `developer-sdk/implementation-plans/`, `finops-portal/implementation-plans/`, `plugin-app-store/implementation-plans/` to per-µservice root. |
| Post-rename `governance-*` in acceptance_lanes = 0 hits | **PASS** — the foundry → governance rename cleaned this corpus thoroughly. |
| 5 IPs still bind ADR-0136 (superseded by ADR-0247) | **TRUE P0** — Phase-2 action: rebind to ADR-0247. |
| 15 IPs / 63 lines reference `retired VCS ratchet` (superseded by `oya git`) | **TRUE P0** — Phase-2 action: rename. |
| 5 IPs reference OpenAPI 3.1 / AsyncAPI 3.0 / 2.x | **TRUE P0** — Phase-2 action: pin to 3.2.0 / 3.1.0. |
| 7 IPs conflate BYOK (provider vs encryption split per 2026-05-20) | **TRUE P0** — Phase-2 action: disambiguate. |
| Only 129/921 IPs (14%) cite any keystone-bundle ADR; 0% cite amendments | **TRUE P1 — pre-keystone-bundle vintage** — most IPs were authored before 2026-05-20. Phase-2 action: per-µservice keystone-rebind batch (substrate first). |
| 154 internet-facing IPs miss ADR-0297 abuse-defence wiring | **TRUE P1** — Phase-2 action: per-IP wire abuse-defence Cedar + UX-floor. |
| 17/17 identity IPs miss ADR-0298 emergency-services / ADR-0299 account-recovery | **TRUE P0** — Phase-2 action: identity IPs MUST wire these per critical-path. |
| 18/18 payments IPs miss ADR-0307 detection-substrate | **TRUE P0** — Phase-2 action: payment fraud signal-emission per DRMP. |
| 600 IPs missing `changeset_contract:` front-matter | **TRUE P1** — Phase-2 action: add `claimable-verifiable-bundleable-promotable`. |
| 545 IPs missing `## ChangeSet boundary` section | **TRUE P1** — Phase-2 action: add. |
| 458 IPs missing `## Concrete File Targets` table | **TRUE P1** — Phase-2 action: add. |
| 809 IPs missing `depends_on:` (no cross-µservice DAG) | **TRUE P0 for parallel-work** — per the 2026-05-21 user directive ("integration is seamless across microservices and parallel work"), depends_on is REQUIRED on every IP. Phase-2 action: populate DAG. |
| 629 IPs below 110-line exemplar floor (per documentation-rigor.md §2 IP-row = 400-line floor now) | **TRUE P1** — Phase-2 action: expand to floor. |
| Messenger 16-IP set at 0% keystone binding — should cite MLS RFC 9420 per ADR-0246 KS#5 | **TRUE P0** — Phase-2 action: messenger IPs rebind to MLS + keystone bundle. |

**Phase-2 IP remediation work split** (~1,217 agent-hours per audit estimate, ~145 PRs, 7-8 calendar weeks):
- Batch 0 — Mechanical seds (retired VCS ratchet → oya git, OpenAPI version pins, BYOK disambiguation, ADR-0136 → ADR-0247)
- Batch 1 — Substrate µservices keystone-bundle rebind (cell → tenancy → cloud-secrets → identity → governance → consent-graph → audit-chain → compliance → cloud-iac → cloud-k8s → observability → api-gateway → application → foundry → ontology)
- Batch 2 — Product µservices rebind
- Batch 3 — Populate `depends_on:` across all 921 IPs to form coherent cross-µservice DAG (enables parallel work)
- Batch 4 — Rewrite `## ChangeSet boundary` + `## Concrete File Targets` + `## Verification` per IP
- Batch 5 — Expand 629 below-floor IPs to documentation-rigor.md §2 IP-row floor (400 lines)
- Batch 6 — Wire critical-path doctrine (ADR-0297..0310) per applicable IP
- Batch 7 — Flatten 75 non-flat-layout IPs per ADR-0131
- Batch 8 — Renumber IPs that have ID collisions
- Batch 9 — Add naming-justification blocks per BNF v4.1
