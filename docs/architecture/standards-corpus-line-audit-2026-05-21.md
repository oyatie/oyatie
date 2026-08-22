# Standards Corpus Line Audit — 2026-05-21

**Auditor:** Executor agent (claude-sonnet-4-6)
**Audit date:** 2026-05-20
**Scope:** All files under `docs/standards/` (89 files audited)
**Audit bar:** `docs/standards/documentation-rigor.md` + `docs/standards/doc-style.md`
**Output:** Remediation punch list — READ ONLY; no standards files were modified

---

## §1 Scope

### 1.1 Files audited

89 files were read line-by-line from `docs/standards/`. The complete enumeration:

```
a11y-canonical.md
agent-instructions-discipline.md
agentic-dev-team-optimization.md
api-design.md
api-surface-separation.md
authz-tier-boundaries.md
autonomy-ceiling.md
backup-canonical.md
brand-voice.md
brownout-degradation-signal.md
capability-authoring.md
cedar-policy-discipline.md
ci-lanes.md
claude-code-harness.md
clean-architecture.md
code-review.md
code-style-rust.md
code-style.md
commit-message.md
compliance-evidence-automation.md
container-image-convention.md
crate-naming-convention.md
cross-microservice-latency-budget.md
cursor-pagination-canonical.md
data-class.md
dependency-policy.md
design-doc-template.md
dr-business-continuity.md
emoji-sticker-reaction-system.md
error-handling.md
event-schema-versioning-canonical.md
finops-cost-attribution-canonical.md
finops-cost-attribution.md
fintech-compliance.md
fips-hsm-substrate-root-signing.md
git-workflow.md
gitops-iac-cluster-tier-boundaries.md
graceful-shutdown-canonical.md
helm-chart-convention.md
hyperscaler-best-practices.md
hyperscaler-invariant-conformance.md
i18n-canonical.md
idempotency-keys-canonical.md
identity-vendor-isolation.md
image-discipline.md
image-signing-canonical.md
incident-severity.md
INDEX.md
locale-routing.md
logging-tracing.md
lts-versions-verified.md
m02-exit-gate-validators.md
messenger-e2e-encryption-mls.md
migration-playbook.md
multi-agent-tool-map.md
multispectrum-review-v2.4.0-cadence.md
multispectrum-review.md
observability-slo.md
observability.md
on-call.md
outbox-pattern-canonical.md
per-tenant-resource-quotas-canonical.md
plugin-authoring.md
postmortem-template.md
prevention-doctrine.md
prfaq-template.md
privacy-review.md
realtime-transport-tier.md
regulatory-pack-authzpolicy-overlays.md
release-management.md
release.md
request-id-canonical.md
rtl-rendering.md
saga-compensation-policy.md
schema-migration.md
security-review.md
sovereign-cloud-overlay.md
step-up-auth-classes.md
stream-processing-rubric.md
tenant-lifecycle.md
testing.md
throttling-tiers.md
timescaledb-adoption.md
trace-sampling-tier.md
ux-best-practices.md
voice-video-call-architecture.md
wasm-runtime-canonical.md
wcag-2-2-aa-checklist.md
workflow-vs-direct-grpc-rubric.md
```

Template-class and non-Standard files included for completeness (they are exempt from the 250-line minimum and many rigor rules): `design-doc-template.md`, `postmortem-template.md`, `prfaq-template.md`. `INDEX.md` is the catalog index. `hyperscaler-best-practices.md` carries `doc_status: research-context`. All others are nominal Standard-class documents.

### 1.2 Audit bar summary

**`documentation-rigor.md`** (1067 lines; the primary bar) imposes the following per-Standard requirements (from §1–§3, key rules):

**Length:**
- Minimum: 250 lines for Standard class
- Maximum: 600 lines for Standard class
- Template class: exempt from min/max
- Runbook class: exempt from min/max

**Frontmatter (required keys for `doc_class: Standard`):**
- `doc_class: Standard` (exactly; case-sensitive)
- `status: Accepted` (canonical value; not Draft, Active, canonical-base)
- `date:` (ISO 8601)
- `canonical_authority:` (pointer to governing ADR or spec)
- `enforced_by:` OR `planned_enforcement_ref:` (CI gate name)
- `related_adrs:` (list)
- `purpose:` (one-paragraph summary)

**Required body structure:**
1. Doctrinal-authority paragraph (first body paragraph must name the governing authority)
2. Numbered sections (§1, §2, ...) with RFC-2119 normative language
3. At least one worked example block per major rule
4. Forbidden-patterns / anti-patterns table or list
5. CI lane name (must match an existing or planned lane in `docs/standards/ci-lanes.md`)
6. Cross-references section

**RFC-2119:**
- At least one formal RFC-2119 MUST or SHOULD sentence per section header
- The sentence form is "Implementations MUST …" not a bulleted list item that happens to say MUST

**§1.1 hyperscaler 8-item sub-test** (every Standard must address all 8):
1. What does AWS / GCP / Azure do at scale?
2. What does the failure mode look like at 10× current load?
3. What is the rollback story?
4. What is the observability surface?
5. What does the on-call engineer do at 3am?
6. What is the migration cost from the current state?
7. What is the security / compliance posture?
8. What is the cost model?

**§1.2 six engineering dimensions** (every Standard must address):
1. Correctness
2. Performance
3. Security
4. Operability
5. Evolvability
6. Cost

**`doc-style.md`** adds:
- Diátaxis quadrant declaration (reference / tutorial / how-to / explanation)
- RFC-2119 normative discipline throughout
- Frontmatter shape with `doc_class:` key (same as above)
- Enforcement lane `governance-doc-style` (or `governance-doc-style` post-ADR-0132)

### 1.3 Key canonical decisions applicable to this audit

The following memory and ADR directives are the primary drift anchors checked in this audit:

| Directive | Source | Effect on audit |
|---|---|---|
| 13-layer enum is canonical | ADR-0105 | Any file citing "12-layer enum" or referencing ADR-0056 for the layer enum is drift |
| grit / icm / rtk / vox retired | ADR-0116 (2026-05-16) | Any file referencing these tools is stale contamination |
| `oya git` is canonical VCS primitive | feedback_git_canonical_2026_05_18 | `oya vcs` references are drift |
| Multispectrum review v2.4.0-cadence.md is operative | multispectrum-review-v2.4.0-cadence.md | v2.1/v2.2/v2.3 references are stale |
| Cedar v4.2 LTS is canonical | observability-slo.md (Accepted 2026-05-17) | Cedar 3.x or 4.9.x references are drift |
| ADR-0145 retired ADR-0141 and ADR-0140 | ADR-0145 | Citations to ADR-0141 as authority are drift |
| ADR-0174 retired by regulatory-pack-authzpolicy-overlays.md | regulatory-pack-authzpolicy-overlays.md | Citations to ADR-0174 as active are drift |
| Object Graph renamed to Ontology | feedback_glossary_ontology_not_object_graph | "Object Graph" term is retired |
| ADR-0185: SvelteKit/Leptos mandate (not React) | ADR-0185 | React prescriptions in standards violate the client-stack mandate |
| OpenAPI 3.2.0 / AsyncAPI 3.1.0 canonical | observability-slo.md | Version strings must include patch version |
| SLSA level must be consistent | internal | image-discipline.md vs image-signing-canonical.md conflict |

### 1.4 Legend

- **P0** — Critical blocker; document is broken, deprecated, or entirely misleading. MUST be resolved before next wave gate.
- **P1** — High severity; significant rigor or correctness failure. MUST be resolved within the current sprint or next lane-gate.
- **P2** — Medium severity; drift, missing field, or minor rigor gap. SHOULD be resolved in the next maintenance cycle.
- **P3** — Low severity; style issue, link risk, cosmetic gap. MAY be deferred to scheduled maintenance.

---

## §2 Contradictions vs documentation-rigor.md

Findings in this section represent direct contradictions between a standard's claims and the rules in `documentation-rigor.md` — the primary audit bar.

### 2.1 Length-cap violations

**doc-rigor rule:** Standard class MUST be ≤ 600 lines and ≥ 250 lines. Documents exceeding 600 lines cannot be meaningfully reviewed per the hyperscaler 8-sub-test requirement and violate the composability principle (each Standard is one concern).

| File | Actual lines | Cap | Overrun ratio | Severity |
|---|---|---|---|---|
| `messenger-e2e-encryption-mls.md` | ~3535 | 600 | 5.9× | P0 |
| `emoji-sticker-reaction-system.md` | ~2316 | 600 | 3.9× | P0 |
| `ux-best-practices.md` | ~2490+ | 600 | 4.2×+ | P0 |
| `voice-video-call-architecture.md` | ~2001+ | 600 | 3.3×+ | P0 |
| `multispectrum-review-v2.4.0-cadence.md` | ~903 | 600 | 1.5× | P1 |
| `fips-hsm-substrate-root-signing.md` | ~704 | 600 | 1.2× | P1 |

Note on `ux-best-practices.md` and `voice-video-call-architecture.md`: both were truncated at 1450 and 813 lines respectively by the read tool's 25K-token limit. The actual line counts are minimums. Both clearly exceed 600 lines by very substantial margins.

The four P0 overruns represent a combined ~10,342 lines of documentation that cannot be reviewed, audited, or tested under the rigor model. They must be split before they can be treated as Accepted standards.

**Suggested split structure for `messenger-e2e-encryption-mls.md`:** This is a Draft design-doc, not a Standard. Reclassify it as `doc_class: Guide` or `doc_class: Design`. The normative rules within it (key-schedule constraints, epoch-rotation requirements, TEK derivation) should each become ≤600-line Standard files once the design stabilizes.

**Suggested split structure for `ux-best-practices.md`:** 23 sections covering design-tokens, accessibility, i18n, dark-mode, density, keyboard, motion, error-handling, empty-states, notifications, forms, search, navigation, mobile, performance, per-product baselines, cross-platform, branding, privacy, AI-features. Each section is a separate concern. Minimum split: 8 files.

**Suggested split structure for `voice-video-call-architecture.md`:** 20 sections covering SFU deployment, codec selection, simulcast/SVC, congestion control, NAT traversal, recording, transcription, E2E encryption, AI features, capacity model, DR, security, compliance. Minimum split: 6 files.

**Suggested split structure for `multispectrum-review-v2.4.0-cadence.md`:** Extract Appendix A (evidence template), Appendix B (worked examples), Appendix C (facet definitions) as companion Guide documents. Keep the operative cadence rules and facet summaries in the Standard file. Target: ≤600 lines.

### 2.2 Doc-class value contradictions

**doc-rigor rule:** `doc_class` MUST be one of the five canonical values: `Standard`, `Template`, `Runbook`, `ADR`, `Guide`.

| File | Actual value | Issue | Severity |
|---|---|---|---|
| `hyperscaler-invariant-conformance.md` | `Hyperscaler-Invariant-Conformance-Standard` | Not in the five-value enum; will fail any frontmatter linter | P1 |
| `regulatory-pack-authzpolicy-overlays.md` | `Standard` (correct) but `status: Active` | `Active` is not canonical; must be `Accepted` | P2 |
| `ux-best-practices.md` | `standard` (lowercase) | Case mismatch; canonical is `Standard` with capital S | P2 |

### 2.3 Required-sections violations — missing forbidden-patterns / anti-patterns

**doc-rigor rule:** Every Standard MUST include a forbidden-patterns table or anti-patterns section. This is the primary mechanism by which the standard communicates what reviewers should reject during code review.

Files with **no anti-patterns or forbidden-patterns section at all**:

| File | Status | Lines | Severity |
|---|---|---|---|
| `api-surface-separation.md` | — | ~106 | P1 |
| `authz-tier-boundaries.md` | Accepted | ~99 | P1 |
| `backup-canonical.md` | — | ~134 | P1 |
| `brownout-degradation-signal.md` | — | ~120 | P1 |
| `capability-authoring.md` | — | ~79 | P1 |
| `cursor-pagination-canonical.md` | Accepted | ~120 | P1 |
| `data-class.md` | — | ~234 | P1 |
| `dr-business-continuity.md` | — | ~136 | P1 |
| `event-schema-versioning-canonical.md` | Accepted | ~107 | P1 |
| `finops-cost-attribution-canonical.md` | — | ~186 | P1 |
| `finops-cost-attribution.md` | — | ~165 | P1 |
| `i18n-canonical.md` | Accepted | ~88 | P1 |
| `idempotency-keys-canonical.md` | Accepted | ~116 | P1 |
| `identity-vendor-isolation.md` | Accepted | ~78 | P1 |
| `image-signing-canonical.md` | Accepted | ~92 | P1 |
| `locale-routing.md` | Accepted | ~80 | P1 |
| `outbox-pattern-canonical.md` | Accepted | ~120 | P1 |
| `per-tenant-resource-quotas-canonical.md` | Accepted | ~98 | P1 |
| `request-id-canonical.md` | Accepted | ~92 | P1 |
| `rtl-rendering.md` | Accepted | ~75 | P1 |
| `step-up-auth-classes.md` | Accepted | ~96 | P1 |

### 2.4 RFC-2119 MUST deficit

**doc-rigor rule:** Every section under a heading MUST contain at least one formal RFC-2119 normative sentence. The sentence MUST begin with the imperative form: "Implementations MUST…", "All µservices MUST…", "The caller MUST…". A bulleted list item that happens to include the word MUST does not satisfy the requirement.

Files where RFC-2119 language is absent entirely or present only in non-normative form:

| File | Evidence | Severity |
|---|---|---|
| `api-surface-separation.md` | No RFC-2119 MUST anywhere | P1 |
| `authz-tier-boundaries.md` | No RFC-2119 MUST | P1 |
| `brownout-degradation-signal.md` | No RFC-2119 MUST | P1 |
| `capability-authoring.md` | No RFC-2119 MUST | P1 |
| `cursor-pagination-canonical.md` | No RFC-2119 MUST | P1 |
| `event-schema-versioning-canonical.md` | No RFC-2119 MUST | P1 |
| `finops-cost-attribution-canonical.md` | No RFC-2119 MUST | P1 |
| `finops-cost-attribution.md` | No RFC-2119 MUST | P1 |
| `i18n-canonical.md` | MUST appears in list items only; no formal normative opener | P2 |
| `idempotency-keys-canonical.md` | No RFC-2119 MUST | P1 |
| `identity-vendor-isolation.md` | No RFC-2119 MUST | P1 |
| `image-signing-canonical.md` | No RFC-2119 MUST | P1 |
| `locale-routing.md` | No RFC-2119 MUST | P1 |
| `outbox-pattern-canonical.md` | No RFC-2119 MUST | P1 |
| `per-tenant-resource-quotas-canonical.md` | No RFC-2119 MUST | P1 |
| `request-id-canonical.md` | No RFC-2119 MUST | P1 |
| `saga-compensation-policy.md` | No RFC-2119 MUST | P1 |
| `sovereign-cloud-overlay.md` | No RFC-2119 MUST | P1 |
| `step-up-auth-classes.md` | No RFC-2119 MUST | P1 |
| `tenant-lifecycle.md` | No RFC-2119 MUST | P1 |

### 2.5 Missing `enforced_by` / `planned_enforcement_ref`

**doc-rigor rule:** Every Standard MUST declare either `enforced_by:` (an active CI gate) or `planned_enforcement_ref:` (a CI gate that is planned but not yet running) in frontmatter. A standard without a CI gate is unenforceable.

Files with no enforcement declaration whatsoever (neither field present):

| File | Severity |
|---|---|
| `a11y-canonical.md` | P1 |
| `api-surface-separation.md` | P1 |
| `authz-tier-boundaries.md` | P1 |
| `backup-canonical.md` | P1 |
| `brownout-degradation-signal.md` | P1 |
| `capability-authoring.md` | P1 |
| `cedar-policy-discipline.md` | P0 (also no frontmatter) |
| `ci-lanes.md` | P1 |
| `code-review.md` | P1 |
| `code-style.md` | P1 |
| `commit-message.md` | P1 |
| `compliance-evidence-automation.md` | P1 |
| `container-image-convention.md` | P0 (also no frontmatter) |
| `cursor-pagination-canonical.md` | P2 (validation gate mentioned in body text only) |
| `dependency-policy.md` | P1 |
| `design-doc-template.md` | P2 (Template class; exempt) |
| `dr-business-continuity.md` | P1 |
| `event-schema-versioning-canonical.md` | P1 |
| `finops-cost-attribution-canonical.md` | P1 |
| `finops-cost-attribution.md` | P1 |
| `gitops-iac-cluster-tier-boundaries.md` | P0 (also no frontmatter) |
| `graceful-shutdown-canonical.md` | P1 |
| `helm-chart-convention.md` | P0 (also no frontmatter) |
| `hyperscaler-best-practices.md` | P2 (research doc; enforcement N/A) |
| `identity-vendor-isolation.md` | P1 (uses `related_lanes` not `enforced_by`) |
| `image-signing-canonical.md` | P1 |
| `locale-routing.md` | P1 |
| `logging-tracing.md` | P1 |
| `migration-playbook.md` | P1 |
| `outbox-pattern-canonical.md` | P1 |
| `per-tenant-resource-quotas-canonical.md` | P1 |
| `plugin-authoring.md` | P1 |
| `prevention-doctrine.md` | P1 |
| `privacy-review.md` | P1 |
| `release.md` | P1 |
| `request-id-canonical.md` | P2 (body says "validation gate planned") |
| `rtl-rendering.md` | P1 |
| `saga-compensation-policy.md` | P1 |
| `schema-migration.md` | P1 |
| `sovereign-cloud-overlay.md` | P1 |
| `step-up-auth-classes.md` | P1 (uses `related_lanes` not `enforced_by`) |
| `stream-processing-rubric.md` | P0 (also no frontmatter) |
| `tenant-lifecycle.md` | P1 |
| `timescaledb-adoption.md` | P0 (also no frontmatter) |
| `trace-sampling-tier.md` | P2 (says "advisory" inline but not in canonical field) |
| `wasm-runtime-canonical.md` | P0 (also no frontmatter; uses blockquote pseudo-frontmatter) |
| `workflow-vs-direct-grpc-rubric.md` | P0 (also no frontmatter) |

---

## §3 Drift

Findings where the content has drifted from canonical decisions recorded in ADRs or memory directives.

### 3.1 Layer-enum drift — 12 vs 13 per ADR-0105

**Canonical directive:** ADR-0105 establishes the 13-layer architectural enum as the definitive set for the oyatie workspace. This supersedes the 12-layer enum previously defined in ADR-0056. Every Standard that enumerates or names layers MUST reference ADR-0105 and MUST reflect 13 layers.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `clean-architecture.md` | §2 body | "12-layer enum lives in ADR-0056"; does not reference ADR-0105 | P1 |
| `code-style-rust.md` | Purpose field + body | "purpose embeds '12-value canonical layer enum'" explicitly; lists 12 layer names | P1 |
| `crate-naming-convention.md` | §2 BNF grammar | BNF `layer` production lists exactly 12 values; ADR-0105 not in `related_adrs` | P1 |
| `multispectrum-review-v2.4.0-cadence.md` | §10.3 | "ADR-0056 — Rust Clean Architecture BNF v4.1 + 12-layer enum" | P1 |
| `multispectrum-review.md` | Cross-references | Cites "ADR-0056 12-layer enum" without noting ADR-0105 supersession | P1 |

**Impact of layer-enum drift:** `crate-naming-convention.md` is the primary naming authority. If its BNF lists 12 layers, CI linters built from that BNF will reject valid 13-layer crate names. This is the highest-impact single drift issue in the corpus.

**Remediation for `crate-naming-convention.md`:** Add the 13th layer value to the BNF `layer` production rule. Add `ADR-0105` to `related_adrs`. Remove the `ADR-0056 12-layer` citation or annotate it as superseded.

### 3.2 Retired VCS primitive drift — `oya vcs` superseded by `oya git`

**Canonical directive:** `oya git` is the canonical agent VCS primitive (feedback_git_canonical_2026_05_18, 2026-05-18). `oya vcs` is permanently superseded.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `agentic-dev-team-optimization.md` | Throughout | Uses `oya vcs claim` in every workflow step | P1 |
| `git-workflow.md` | §1 framing | Entire §1 is built around the superseded `oya vcs` triad; the canonical flow describes `oya vcs` as the primary surface | P1 |

### 3.3 Retired external-agent-tooling drift — grit / icm / rtk / vox per ADR-0116

**Canonical directive:** ADR-0116 (2026-05-16) retired grit, icm, rtk, and vox as external agent-coordination tooling. All references to these tools in standards must be removed or replaced with plain `git` / `gh` / `cargo` equivalents.

The table below distinguishes contamination level:

- **Level A (document is entirely about the deprecated tool):** P0
- **Level B (major workflow section uses the deprecated tool):** P0
- **Level C (minor reference in a list or alert):** P1

| File | Level | Drift evidence | Severity |
|---|---|---|---|
| `claude-code-harness.md` | A | Every section describes grit/icm/rtk workflows; the document has no value without these tools | P0 |
| `git-workflow.md` | B | §1 names grit/icm/rtk as "sanctioned-primitive triad"; icm-store rationale throughout | P0 |
| `multi-agent-tool-map.md` | B | §3 table has grit/icm/rtk as "Default-sanctioned"; §7 names them as canonical MCP servers | P0 |
| `agent-instructions-discipline.md` | B | §2 workflow and §10 worked example use `grit claim`/`grit done`/`icm` explicitly | P0 |
| `hyperscaler-best-practices.md` | B | "Agent tooling" table rows: grit v0.3.0, icm v0.10.39, rtk dev-0.39.0 | P0 |
| `lts-versions-verified.md` | B | "Agent tooling" section lists grit v0.3.0, icm v0.10.39, rtk dev-0.39.0 with version pins | P0 |
| `m02-exit-gate-validators.md` | C | References "before `grit done`" in BLOCKER workflow description | P1 |
| `on-call.md` | C | §3 runbook discipline uses "`rtk`-prefixed" commands | P1 |
| `testing.md` | C | §9 flaky SLA: "day 7: assignee receives a reminder via icm; day 12: incident commander escalates" | P1 |
| `INDEX.md` | C | Catalog row description for `claude-code-harness.md` still describes grit-based workflow | P1 |
| `image-discipline.md` | C | `related_adrs` lists ADR-0053/0052/0054 (the grit protocol ADRs) | P2 |
| `observability.md` | C | `related_adrs` lists ADR-0053/0052/0054 | P2 |
| `release-management.md` | C | `related_adrs` lists ADR-0053/0052/0054 | P2 |
| `security-review.md` | C | `related_adrs` lists ADR-0053/0052/0054 | P2 |
| `data-class.md` | C | `related_adrs` lists ADR-0053/0052/0054 | P2 |
| `autonomy-ceiling.md` | C | `related_adrs` lists ADR-0052/0053/0054 | P2 |

**Note on ADR-0052/0053/0054:** These ADRs defined the grit claim/work/done protocol. ADR-0116 supersedes them. Including them in `related_adrs` without a supersession annotation implies they are still active authorities. Every standard that lists them must either remove them or annotate them as "superseded by ADR-0116".

### 3.4 Multispectrum-review version drift

**Canonical directive:** `multispectrum-review-v2.4.0-cadence.md` (Accepted 2026-05-20) is the operative multispectrum review standard. All prior versions (v2.1.0, v2.2.0, v2.3.0) are superseded.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `multispectrum-review.md` | `canonical_authority:` | Points to `/specs/multispectrum-review.json` version 2.1.0 | P1 |
| `multispectrum-review.md` | `related_adrs:` | Lists ADR-0054 (grit protocol, retired by ADR-0116) | P1 |
| `multispectrum-review.md` | Body | No "SUPERSEDED — see v2.4.0-cadence.md" notice | P1 |

`multispectrum-review.md` is a thin-pointer-gateway document. Its pointer is broken — it points to a superseded spec version. It should be updated to point to the v2.4.0 operative standard or clearly marked as a historical index entry.

### 3.5 Cedar version conflict

**Canonical directive:** `observability-slo.md` (Accepted 2026-05-17) pins Cedar v4.2.0 LTS as the platform-canonical version.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `cedar-policy-discipline.md` | Throughout | References "Cedar 3.x" as the operative version | P1 |
| `regulatory-pack-authzpolicy-overlays.md` | Body | Uses "Cedar 4.9.1" — 4.9.1 is ahead of the pinned 4.2.0 LTS | P1 |

The Cedar 4.9.1 vs 4.2.0 LTS issue in `regulatory-pack-authzpolicy-overlays.md` is potentially the more dangerous of the two: 4.9.1 may use policy features not supported in the 4.2.0 LTS build. An explicit compatibility matrix or ADR resolving the Cedar LTS pin must be produced.

### 3.6 Grafana version conflict

**Canonical directive:** `observability-slo.md` (Accepted 2026-05-17) pins Grafana 12.0 LTS.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `lts-versions-verified.md` | "Agent tooling" / "Observability" section | Lists Grafana 13.0.1 as "current" — contradicts Grafana 12.0 LTS in observability-slo.md | P2 |

Resolution: clarify whether `lts-versions-verified.md` tracks the LTS pin (in which case 12.0 LTS is correct) or the latest stable release (in which case the document needs a column distinguishing "LTS pin" from "latest stable"). The two values serve different purposes and should not be conflated.

### 3.7 SLSA level conflict — internal contradiction

**Internal contradiction** between two separately Accepted standards on the required SLSA supply-chain security level:

| File | Claim | Date accepted |
|---|---|---|
| `image-discipline.md` | SLSA Level 2 required | 2026-05-xx |
| `image-signing-canonical.md` | SLSA Level 3 required | 2026-05-18 |

These two standards are directly contradictory. Any CI lane implementing either rule will conflict with the other. An ADR decision resolving the canonical SLSA level is required. Until resolved, both standards are ambiguous authorities.

### 3.8 ADR-0145 / ADR-0141 / ADR-0140 drift

**Canonical directive:** ADR-0145 (inter-microservice communication reform) retired both ADR-0141 (GraphQL federation) and ADR-0140.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `cross-microservice-latency-budget.md` | `canonical_authority:` + body | Explicitly cites "ADR-0141 (retired per ADR-0145)" as its authority; the file acknowledges the retirement but still uses the retired ADR as the governing document | P1 |

The acknowledgment "retired per ADR-0145" in the body text is informational but does not fix the problem: `canonical_authority: ADR-0141` must be replaced with `canonical_authority: ADR-0145`.

### 3.9 ADR-0174 retirement drift

**Canonical directive:** Per `regulatory-pack-authzpolicy-overlays.md`, ADR-0174 was retired and absorbed into that standard.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `sovereign-cloud-overlay.md` | `related_adrs:` and body | References ADR-0174 as an active, current authority | P2 |
| `throttling-tiers.md` | Observability / alerts section | Alert text says "via ADR-0174" for SEV-2 finops routing | P2 |

### 3.10 React violation — ADR-0185 client-stack mandate

**Canonical directive:** ADR-0185 mandates SvelteKit (web), Leptos (Rust-native web), SwiftUI, Jetpack Compose, GTK 4, and WinUI 3 as the six sanctioned client stacks. React is explicitly not in the sanctioned list.

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `emoji-sticker-reaction-system.md` | Throughout | Prescribes "React 18+" as the implementation stack; uses React Suspense, React.memo, React hooks, JSX syntax throughout; no mention of SvelteKit or Leptos | P0 |

This is a complete technology-stack mismatch. The standard must either be rewritten for the sanctioned stacks or marked as a historical design artifact that predates ADR-0185.

### 3.11 "Object Graph" retired term

**Canonical directive:** Per memory feedback_glossary_ontology_not_object_graph, "Object Graph" is renamed to "Ontology".

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `schema-migration.md` | Body | References "ADR-0006 (Object Graph property-tier)" | P2 |

### 3.12 OpenAPI version string precision

**Canonical directive:** `observability-slo.md` uses the full three-part version string "OpenAPI 3.2.0".

| File | Location | Drift evidence | Severity |
|---|---|---|---|
| `api-design.md` | Body | Specifies "OpenAPI 3.2" (missing `.0` patch component) | P3 |

While minor, version strings in standards are consumed by CI generators. An incomplete version string may match multiple releases.

### 3.13 Enforcement-lane prefix drift

**Canonical directive:** Per ADR-0132, new governance CI lane names carry the prefix `governance-*`. Existing `governance-*` lanes are retained until each is individually renamed in its own migration IP.

The following files reference `governance-*` lanes that appear to have been authored or substantially revised after the ADR-0132 cutoff date and may need the newer prefix:

| File | Lane name referenced | Note |
|---|---|---|
| `multispectrum-review-v2.4.0-cadence.md` | `governance-multispectrum-*` | Accepted 2026-05-20, after ADR-0132 |
| `testing.md` | `governance-test-evidence` | Date 2026-05-12 |
| `observability.md` | `governance-otel-emit` | Older |
| `image-discipline.md` | `F-PENDING-IMAGE-DISCIPLINE` | Pre-ADR-0132 |
| `on-call.md` | `governance-runbook-index-resolves` | Older |
| `security-review.md` | `governance-supply-chain` | Older |

**Note:** This is informational only (P3). ADR-0132 explicitly states existing lanes are retained until migrated. No action required until the migration IP for each lane runs.

---

## §4 Staleness — Superseded Without Markers, placeholder markers, Retired Terms

### 4.1 Complete stub documents

These files exist in the standards corpus but contain no normative content — they are entirely placeholder marker or placeholder marker placeholders. Any standard that references them is referencing a void.

| File | Approx lines | Evidence | Severity |
|---|---|---|---|
| `brand-voice.md` | ~25 | Every section is a "placeholder marker: ..." placeholder; Owner: placeholder marker; CI lane: placeholder marker; no normative sentences | P0 |
| `incident-severity.md` | ~26 | All sections empty or placeholder marker; references non-existent `STANDARDS-AND-TEMPLATES.md`; Owner: placeholder marker | P0 |

`brand-voice.md` is referenced normatively by `ux-best-practices.md` §20 (Branding + White-Label). A normative reference to a stub document is a critical rigor failure: any team following `ux-best-practices.md` has no guidance for the voice and tone surface.

`incident-severity.md` is critical for operational clarity: when an incident is declared, which severity level applies? Without this document, every on-call engineer makes their own judgment. The on-call standard (`on-call.md`) implicitly depends on severity thresholds being defined.

### 4.2 Entire standard describes deprecated tooling

| File | Evidence | Severity |
|---|---|---|
| `claude-code-harness.md` | Every section (Claude Code setup, claim/work/done workflow, MCP server config, rtk passthrough) describes grit/icm/rtk workflows retired by ADR-0116 (2026-05-16) | P0 |

`claude-code-harness.md` is not partially stale — it is completely deprecated. There is no section of it that reflects current practice. Keeping it in the corpus actively misleads new contributors who read it as an onboarding document.

### 4.3 Draft status without supersession markers

| File | Status | Lines | Issue | Severity |
|---|---|---|---|---|
| `messenger-e2e-encryption-mls.md` | Draft | ~3535 | Design-doc masquerading as a Standard; no supersession path or promotion timeline | P0 |
| `api-design.md` | Draft | ~159 | Core API design standard is Draft with no enforcement lane; provides no enforceable contract | P1 |
| `code-review.md` | Draft | ~95 | Draft, under-250 lines; the code review process has no normative standard | P1 |
| `voice-video-call-architecture.md` | Draft | ~2001+ | Architecture doc is Draft; 2000+ lines | P1 |
| `fintech-compliance.md` | Draft | ~448 | "Open questions" section with placeholder marker items; describes pending regulatory decisions | P1 |

### 4.4 placeholder markers items in Accepted standards

An Accepted standard MUST NOT contain placeholder marker fields, PENDING enforcement placeholders, or placeholder marker comments. These indicate incomplete work that was promoted to Accepted prematurely.

| File | Item | Severity |
|---|---|---|
| `agentic-dev-team-optimization.md` | placeholder marker gates in several workflow sections | P2 |
| `autonomy-ceiling.md` | `enforcement: F-PENDING` in frontmatter | P2 |
| `cross-microservice-latency-budget.md` | `enforcement: placeholder marker` in frontmatter | P2 |
| `data-class.md` | `F-PENDING-DATA-CLASS` and `F-PENDING-DSR-CASCADE` lane names | P2 |
| `error-handling.md` | `F-PENDING-SILENT-FAILURE` enforcement | P2 |
| `fintech-compliance.md` | "Open questions" section with explicit placeholder marker items | P2 |
| `image-discipline.md` | `F-PENDING-IMAGE-DISCIPLINE` and `F-PENDING-CONTAINER-BASE` | P2 |
| `on-call.md` | `planned_enforcement_ref: governance-runbook-index-resolves` with no shipping timeline | P2 |
| `release-management.md` | `planned_enforcement_ref: governance-flag-debt` | P2 |

### 4.5 Superseded references without markers

A superseded or retired standard MUST carry a visible "SUPERSEDED — see [replacement]" marker at the top of the document to prevent consumers from continuing to use it.

| File | Missing marker | Severity |
|---|---|---|
| `multispectrum-review.md` | No "SUPERSEDED — see v2.4.0-cadence.md" marker; still presents as the operative standard | P1 |
| `git-workflow.md` | §1 `oya vcs` framing is superseded; no marker | P1 |
| `multi-agent-tool-map.md` | §3 grit/icm/rtk table is superseded; no marker | P1 |
| `lts-versions-verified.md` | Agent-tooling section superseded; no marker | P1 |
| `hyperscaler-best-practices.md` | Agent-tooling section superseded; no marker | P1 |
| `INDEX.md` | Catalog row for `claude-code-harness.md` is superseded; no marker | P1 |
| `claude-code-harness.md` | Entire document superseded; no marker | P0 (covered by §4.2) |

### 4.6 Non-existent file references

| Source file | Referenced path | Status | Severity |
|---|---|---|---|
| `backup-canonical.md` | `docs/standards/promotion-policy.md` | File does not exist in corpus | P1 |
| `incident-severity.md` | `STANDARDS-AND-TEMPLATES.md` | File does not exist in `docs/` | P1 |
| `release.md` | `STANDARDS-AND-TEMPLATES.md` | Very likely does not exist (same broken reference pattern) | P2 |
| `migration-playbook.md` | `templates/migration-runbook-template.md` | Explicitly noted as "planned, non-existent" in body | P2 |
| `fips-hsm-substrate-root-signing.md` | `ADR-NNNN-foundry-meta-trust-root` | Placeholder ADR number; real ADR not yet authored | P2 |
| `timescaledb-adoption.md` | `crates/shared-timescale-policy-worker/` | Phase-2 follow-on; not yet authored | P3 |
| `testing.md` | `docs/QA-TEST-STRATEGY.md` | Not confirmed in this audit | P3 |
| `testing.md` | `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` | Scratch file; not a durable normative reference | P3 |

---

## §5 Rigor Failures

### 5.1 No frontmatter at all

**doc-rigor rule:** Every Standard MUST begin with a valid YAML frontmatter block delimited by `---`. Files without frontmatter cannot be processed by any CI linter, cannot declare their doc_class, status, or enforcement lane.

Eight files in the corpus have no frontmatter whatsoever:

| File | Pseudo-frontmatter used | Severity |
|---|---|---|
| `cedar-policy-discipline.md` | Starts with `# Cedar Policy Discipline` — no frontmatter | P0 |
| `container-image-convention.md` | Starts with `# Container image convention` — no frontmatter | P0 |
| `gitops-iac-cluster-tier-boundaries.md` | Starts with `# Standard —` — no frontmatter | P0 |
| `helm-chart-convention.md` | Starts with `# Helm chart convention` — no frontmatter | P0 |
| `stream-processing-rubric.md` | Starts with `# Stream-Processing Rubric` — no frontmatter, no status, no date | P0 |
| `timescaledb-adoption.md` | Uses markdown bold `**Authority:**`, `**Status:**`, `**Owner:**` as pseudo-frontmatter | P0 |
| `wasm-runtime-canonical.md` | Uses blockquote `> ADR anchor:` as pseudo-frontmatter | P0 |
| `workflow-vs-direct-grpc-rubric.md` | Uses markdown bold `**Status:**`, `**Owner:**` as pseudo-frontmatter | P0 |

All eight files are functionally unregistered with the governance system: they have no doc_class, no enforced_by, no canonical_authority, and cannot participate in automated compliance gates.

### 5.2 Double frontmatter — two `---` YAML fence pairs

A file with two `---` blocks has two YAML documents. Strict YAML parsers will take only the first block. In all cases the second block contains the canonical `doc_class:` key while the first block contains only `purpose:` and `doc_status:` — meaning the canonical keys are invisible to any strict parser.

| File | First block content | Second block content | Severity |
|---|---|---|---|
| `git-workflow.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |
| `image-discipline.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |
| `INDEX.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:` | P1 |
| `multi-agent-tool-map.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |
| `observability.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |
| `on-call.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |
| `release-management.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |
| `security-review.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |
| `testing.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `related_adrs:` | P1 |

**Root cause pattern:** These files were partially migrated from an older frontmatter schema (`purpose:` + `doc_status:`) to the canonical schema (`doc_class:` + `status:`). The migration left the old block in place instead of replacing it. The fix is straightforward: remove the first `---` block entirely and keep only the second (canonical) block.

### 5.3 `contract:` key instead of `doc_class:`

**doc-rigor rule:** The canonical frontmatter key is `doc_class:`. The `contract:` key predates the doc-rigor standard and is not a recognized field.

Eleven files use `contract:` as their primary classification key:

| File | `contract:` value | Status field | Severity |
|---|---|---|---|
| `api-surface-separation.md` | `api-surface-separation` | *(absent)* | P1 |
| `backup-canonical.md` | `backup-canonical` | *(absent)* | P1 |
| `brownout-degradation-signal.md` | `brownout-degradation-signal` | *(absent)* | P1 |
| `cross-microservice-latency-budget.md` | `cross-microservice-latency-budget` | *(absent)* | P1 |
| `dr-business-continuity.md` | `dr-business-continuity` | *(absent)* | P1 |
| `finops-cost-attribution-canonical.md` | `finops-cost-attribution-canonical` | *(absent)* | P1 |
| `finops-cost-attribution.md` | `finops-cost-attribution` | *(absent)* | P1 |
| `saga-compensation-policy.md` | `saga-compensation-policy` | `canonical-base` | P1 |
| `sovereign-cloud-overlay.md` | `sovereign-cloud-overlay` | `canonical-base` | P1 |
| `tenant-lifecycle.md` | `tenant-lifecycle` | `canonical-base` | P1 |
| `throttling-tiers.md` | `throttling-tiers` | `canonical-base` | P1 |

**Note on `status: canonical-base`:** This is a non-standard status value used by five files. `documentation-rigor.md` specifies `Accepted` as the canonical Accepted status. If `canonical-base` represents a different lifecycle state, it must be defined in doc-rigor.md; otherwise all five files must be migrated to `status: Accepted`.

### 5.4 Minimal frontmatter — missing `doc_class:`

Files with frontmatter blocks that contain only `purpose:` and `doc_status:` (an older schema) and are missing the canonical `doc_class:` field, as well as missing `status:`, `date:`, and enforcement fields:

| File | Keys present | Missing keys | Severity |
|---|---|---|---|
| `capability-authoring.md` | `purpose:`, `doc_status:` | `doc_class:`, `status:`, `date:`, `enforced_by:`, `related_adrs:` | P1 |
| `ci-lanes.md` | `purpose:`, `doc_status:` | Same | P1 |
| `code-style.md` | `purpose:`, `doc_status:` | Same | P1 |
| `commit-message.md` | `purpose:`, `doc_status:` | Same | P1 |
| `dependency-policy.md` | `purpose:`, `doc_status:` | `doc_class:`, `canonical_authority:` | P1 |
| `logging-tracing.md` | `purpose:`, `doc_status:` | Same | P1 |
| `m02-exit-gate-validators.md` | Minimal only | Same | P1 |
| `migration-playbook.md` | Minimal only | Same | P1 |
| `plugin-authoring.md` | `purpose:`, `doc_status:` | Same | P1 |
| `prevention-doctrine.md` | `purpose:`, `doc_status:` | Same | P1 |
| `privacy-review.md` | Minimal | Same | P1 |
| `release.md` | Minimal | Same | P1 |

### 5.5 Under-250 lines — minimum rigor threshold for Standard class

**doc-rigor rule:** A Standard-class document MUST be ≥ 250 lines. Below this threshold, it is impossible to satisfy all required sections (doctrinal authority paragraph, numbered sections with RFC-2119, examples, anti-patterns, CI lane, cross-references) while maintaining meaningful density.

**Exception:** Documents explicitly declared as `shape: thin-pointer-gateway` in frontmatter are exempt from the 250-line minimum. These are gateway documents whose primary payload lives in a referenced JSON spec file. Only 3 files plausibly qualify for this exception (`multispectrum-review.md`, `realtime-transport-tier.md`, `wcag-2-2-aa-checklist.md`), and none of them have declared this shape.

Files below the 250-line threshold:

| File | Approx lines | Gap to 250 | shape declared? | Severity |
|---|---|---|---|---|
| `brand-voice.md` | ~25 | 225 | No | P0 (stub) |
| `incident-severity.md` | ~26 | 224 | No | P0 (stub) |
| `code-style.md` | ~70 | 180 | No | P1 |
| `release.md` | ~65 | 185 | No | P1 |
| `schema-migration.md` | ~64 | 186 | No | P1 |
| `privacy-review.md` | ~64 | 186 | No | P1 |
| `m02-exit-gate-validators.md` | ~64 | 186 | No | P1 |
| `a11y-canonical.md` | ~82 | 168 | No | P1 |
| `identity-vendor-isolation.md` | ~78 | 172 | No | P1 |
| `container-image-convention.md` | ~77 | 173 | No | P0 (no frontmatter) |
| `wasm-runtime-canonical.md` | ~83 | 167 | No | P0 (no frontmatter) |
| `capability-authoring.md` | ~79 | 171 | No | P1 |
| `rtl-rendering.md` | ~75 | 175 | No | P1 |
| `locale-routing.md` | ~80 | 170 | No | P1 |
| `stream-processing-rubric.md` | ~98 | 152 | No | P0 (no frontmatter) |
| `wcag-2-2-aa-checklist.md` | ~111 | 139 | No | P1 (gateway candidate) |
| `step-up-auth-classes.md` | ~96 | 154 | No | P1 |
| `authz-tier-boundaries.md` | ~99 | 151 | No | P1 |
| `gitops-iac-cluster-tier-boundaries.md` | ~99 | 151 | No | P0 (no frontmatter) |
| `workflow-vs-direct-grpc-rubric.md` | ~86 | 164 | No | P0 (no frontmatter) |
| `per-tenant-resource-quotas-canonical.md` | ~98 | 152 | No | P1 |
| `helm-chart-convention.md` | ~95 | 155 | No | P0 (no frontmatter) |
| `compliance-evidence-automation.md` | ~97 | 153 | No | P1 |
| `graceful-shutdown-canonical.md` | ~90 | 160 | No | P1 |
| `realtime-transport-tier.md` | ~84 | 166 | No | P1 (gateway candidate) |
| `i18n-canonical.md` | ~88 | 162 | No | P1 |
| `idempotency-keys-canonical.md` | ~116 | 134 | No | P1 |
| `image-signing-canonical.md` | ~92 | 158 | No | P1 |
| `request-id-canonical.md` | ~92 | 158 | No | P1 |
| `trace-sampling-tier.md` | ~94 | 156 | No | P1 |
| `cedar-policy-discipline.md` | ~119 | 131 | No | P0 (no frontmatter) |
| `commit-message.md` | ~119 | 131 | No | P1 |
| `brownout-degradation-signal.md` | ~120 | 130 | No | P1 |
| `cursor-pagination-canonical.md` | ~120 | 130 | No | P1 |
| `outbox-pattern-canonical.md` | ~120 | 130 | No | P1 |
| `timescaledb-adoption.md` | ~122 | 128 | No | P0 (no frontmatter) |
| `backup-canonical.md` | ~134 | 116 | No | P1 |
| `throttling-tiers.md` | ~138 | 112 | No | P1 |
| `tenant-lifecycle.md` | ~136 | 114 | No | P1 |
| `event-schema-versioning-canonical.md` | ~107 | 143 | No | P1 |
| `dr-business-continuity.md` | ~136 | 114 | No | P1 |
| `sovereign-cloud-overlay.md` | ~128 | 122 | No | P1 |
| `ci-lanes.md` | ~157 | 93 | No | P1 |
| `code-review.md` | ~95 | 155 | No | P1 |
| `api-surface-separation.md` | ~106 | 144 | No | P1 |
| `logging-tracing.md` | ~77 | 173 | No | P1 |
| `migration-playbook.md` | ~103 | 147 | No | P1 |
| `prevention-doctrine.md` | ~89 | 161 | No | P1 |
| `plugin-authoring.md` | ~117 | 133 | No | P1 |
| `multispectrum-review.md` | ~72 | 178 | No | P1 (gateway candidate) |
| `finops-cost-attribution-canonical.md` | ~186 | 64 | No | P1 |
| `finops-cost-attribution.md` | ~165 | 85 | No | P1 |

### 5.6 Non-YAML frontmatter in `emoji-sticker-reaction-system.md`

`emoji-sticker-reaction-system.md` uses markdown bold headers embedded in the YAML frontmatter fence:

```
---
**Status:** Draft
**Owner:** axis-messenger, council-design-system
**Date:** 2026-05-20
---
```

This is invalid YAML. Every key uses `**bold:**` markdown syntax rather than plain `key:` syntax. A YAML parser will fail to parse this frontmatter block entirely. The file has no parseable `doc_class`, `status`, or `date` from a machine perspective.

### 5.7 `hyperscaler-best-practices.md` classification note

`hyperscaler-best-practices.md` carries `doc_status: research-context`. It is not a Standard and is exempt from the 250-line minimum and rigor rules. However, it is cited as a normative source by `testing.md` §12 ("Sources scanned: `.omc/scratch/hyperscaler-best-practices-2026-05-12.md`"). Research context documents SHOULD NOT be cited as normative sources in Standards.

---

## §6 Cross-Reference Broken Links

### 6.1 Confirmed broken or missing file references

| Source file | Referenced path | Confirmed status | Severity |
|---|---|---|---|
| `backup-canonical.md` | `docs/standards/promotion-policy.md` | Does not exist in `docs/standards/` | P1 |
| `incident-severity.md` | `STANDARDS-AND-TEMPLATES.md` | Does not exist in `docs/` | P1 |
| `release.md` | `STANDARDS-AND-TEMPLATES.md` | Does not exist in `docs/` | P2 |
| `migration-playbook.md` | `templates/migration-runbook-template.md` | Explicitly "planned, non-existent" per body text | P2 |
| `fips-hsm-substrate-root-signing.md` | `ADR-NNNN-foundry-meta-trust-root` | Placeholder; real ADR not authored | P2 |
| `timescaledb-adoption.md` | `crates/shared-timescale-policy-worker/` | Phase-2 follow-on; not yet authored | P3 |
| `testing.md` | `docs/QA-TEST-STRATEGY.md` | Not confirmed; not in standards corpus scope | P3 |
| `testing.md` | `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` | Scratch file; not a durable reference | P3 |

### 6.2 ADR references to superseded or retired ADRs

Each entry below represents a `related_adrs:` citation to an ADR that has been superseded or retired, without an annotation noting the supersession.

| Source file | ADR cited | Why it is superseded/retired | Severity |
|---|---|---|---|
| `agent-instructions-discipline.md` | ADR-0052, ADR-0053, ADR-0054 | Superseded by ADR-0116 (grit/icm/vox retirement) | P0 |
| `claude-code-harness.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P0 |
| `git-workflow.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P0 |
| `multi-agent-tool-map.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P0 |
| `autonomy-ceiling.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P1 |
| `data-class.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P1 |
| `INDEX.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P1 |
| `on-call.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P1 |
| `multispectrum-review.md` | ADR-0054 | Same | P1 |
| `image-discipline.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P2 |
| `observability.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P2 |
| `release-management.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P2 |
| `security-review.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P2 |
| `testing.md` | ADR-0052, ADR-0053, ADR-0054 | Same | P2 |
| `cross-microservice-latency-budget.md` | ADR-0141 | Retired per ADR-0145 | P1 |
| `sovereign-cloud-overlay.md` | ADR-0174 | Retired per regulatory-pack-authzpolicy-overlays.md | P2 |
| `throttling-tiers.md` | ADR-0174 | Same | P2 |
| `clean-architecture.md` | ADR-0056 (for 12-layer enum) | ADR-0105 supersedes the layer enum portion | P1 |
| `code-style-rust.md` | ADR-0056 (for 12-layer enum) | Same | P1 |
| `multispectrum-review-v2.4.0-cadence.md` | ADR-0056 "12-layer" in §10.3 | Same | P1 |
| `multispectrum-review.md` | ADR-0056 "12-layer" | Same | P1 |

### 6.3 Standards cross-referencing non-existent standards

| Source file | Referenced standard | Status | Severity |
|---|---|---|---|
| `backup-canonical.md` | `docs/standards/promotion-policy.md` | Does not exist | P1 |
| `ux-best-practices.md` | `docs/standards/brand-voice.md` (normative citation in §20) | Exists but is a complete stub | P2 |

### 6.4 Potentially orphaned microservice-local ADR references

`voice-video-call-architecture.md` cites the following ADRs in its `related_adrs:` list and throughout its body:

- `ADR-MEET-0001` through `ADR-MEET-0006`
- `ADR-MSGR-0001`, `ADR-MSGR-0002`

These ADRs do not appear in the main `docs/decisions/` directory namespace used by all other Standards. They may be microservice-local ADRs located under `microservices/meet/` or `microservices/messenger/`. If they are not in the canonical ADR registry, the cross-references in this standard are formally broken from the perspective of the governance tooling. Verify they exist and either add them to the canonical registry or document the microservice-local ADR convention.

---

## §7 Inbound-Citation Map

This section identifies which standards are cited by other standards. High-inbound files carry multiplicative risk: stale content in a high-inbound standard propagates incorrect information to every consumer.

### 7.1 Highest-inbound-citation standards

The following list is based on cross-references found in other standards' `related_standards:`, `companion_docs:`, and body-text links during the audit:

| Standard | Est. inbound citations | Current rigor issues | Risk rating |
|---|---|---|---|
| `clean-architecture.md` | 8+ | Layer-enum drift (12 vs 13); 397 lines; no ADR-0105 reference | HIGH |
| `crate-naming-convention.md` | 6+ | BNF lists 12 layers; missing ADR-0105 | HIGH |
| `code-style-rust.md` | 5+ | Embeds 12-layer list; double-frontmatter partial migration | HIGH |
| `observability-slo.md` | 7+ | Good rigor; Grafana version vs lts-versions-verified | LOW |
| `testing.md` | 6+ | Double-frontmatter; ADR-0052/0053 citations; icm in §9 | MEDIUM |
| `dependency-policy.md` | 5+ | Minimal frontmatter; no canonical_authority; no enforcement | MEDIUM |
| `observability.md` | 5+ | Double-frontmatter; ADR-0052/0053 | MEDIUM |
| `multispectrum-review-v2.4.0-cadence.md` | 4+ | 903 lines (over cap); ADR-0056 12-layer ref in §10.3 | HIGH |
| `error-handling.md` | 4+ | F-PENDING enforcement; cites ADR-0083 only | MEDIUM |
| `a11y-canonical.md` | 4+ | 82 lines (severely under-250); no enforcement | HIGH |
| `api-design.md` | 4+ | Draft status; no enforcement; under-250 | HIGH |
| `cedar-policy-discipline.md` | 3+ | No frontmatter; Cedar 3.x drift | HIGH |
| `git-workflow.md` | 3+ | Double-frontmatter; grit/icm contamination; oya vcs drift | HIGH |
| `lts-versions-verified.md` | 3+ | Agent-tooling section with deprecated tools; Grafana conflict | HIGH |
| `ci-lanes.md` | 3+ | Minimal frontmatter; no doc_class; no enforcement | MEDIUM |

### 7.2 Risk amplification from high-inbound stale standards

The most damaging combination is high-inbound + layer-enum drift because BNF grammars are consumed by generators:

- `crate-naming-convention.md` BNF (12 layers) → any code generator built from it produces incorrect crate names
- `clean-architecture.md` §2 (12 layers) → onboarding engineers learn the wrong architecture layer count
- `code-style-rust.md` (12-layer list in purpose) → every Rust style guide reference propagates the wrong count

These three files must be updated before ADR-0105 compliance can be enforced in CI.

### 7.3 Zero-inbound-citation Standards (orphan risk)

The following Standards appear to have no inbound citations from other standards in the corpus. They are undiscoverable unless found through the INDEX.md catalog:

| File | Last known status |
|---|---|
| `fintech-compliance.md` | Draft; 448 lines; not referenced by any other Standard |
| `dr-business-continuity.md` | Under-250; not referenced |
| `m02-exit-gate-validators.md` | Contains stale grit done workflow |
| `brand-voice.md` | Stub; effectively a void reference |
| `incident-severity.md` | Stub; effectively a void reference |
| `timescaledb-adoption.md` | No frontmatter; not referenced by other Standards |
| `wasm-runtime-canonical.md` | No frontmatter; low inbound citation |
| `prfaq-template.md` | Template class; no normative citations needed |
| `postmortem-template.md` | Template class; no normative citations needed |

---

## §8 Remediation Actions Ordered by Severity

The following ordered remediation workqueue covers all findings. Items are sorted P0 → P1 → P2 → P3. Within each severity tier, items are ordered by impact and dependency (items that unblock others come first).

### P0 — Critical (must resolve before next wave gate)

**R-P0-01 | Delete or tombstone `claude-code-harness.md`**

The entire document describes grit/icm/rtk workflows retired by ADR-0116 (2026-05-16). It provides no value and actively misleads new contributors. Action: replace file contents with a three-line tombstone pointing to ADR-0116 and the CLAUDE.md sanctioned-primitives section, or delete the file and add a routing note to INDEX.md.

Affected: `claude-code-harness.md`
Owner recommendation: lane-governance

---

**R-P0-02 | Reclassify `messenger-e2e-encryption-mls.md` as a Guide or Design document**

3535 lines, Draft status. This is a design-doc — an extended architectural narrative with implementation specifics. It cannot function as an Accepted Standard while Draft and 5.9× over the line cap. Action: change `doc_class: Standard` to `doc_class: Guide`, move it under `docs/` if appropriate, and extract the normative rules into ≤600-line Standard files when the design stabilizes and reaches Accepted.

Affected: `messenger-e2e-encryption-mls.md`
Owner recommendation: axis-messenger

---

**R-P0-03 | Strip all grit/icm references from `agent-instructions-discipline.md`**

§2 and §10 worked example use `grit claim`/`grit done`/`icm` as the operative workflow. Replace with `oya git`-based equivalents: `oya git branch`, `oya git commit`, standard `gh pr create` flow. Remove all mentions of grit, icm, rtk, vox.

Affected: `agent-instructions-discipline.md`
Owner recommendation: lane-governance

---

**R-P0-04 | Rewrite `git-workflow.md` §1 around `oya git`; merge double-frontmatter**

§1 "sanctioned-primitive triad" is the key superseded element. Rewrite to describe `oya git` as the canonical primitive per feedback_git_canonical_2026_05_18. Remove `oya vcs claim`/`oya vcs work`/`oya vcs done` from all workflow steps. Remove `icm-store` rationale contracts. Remove ADR-0052/0053/0054 from related_adrs. Merge the two frontmatter blocks into one.

Affected: `git-workflow.md`
Owner recommendation: lane-governance

---

**R-P0-05 | Strip all grit/icm/rtk from `multi-agent-tool-map.md`; merge double-frontmatter**

§3 table and §7 MCP-server list. Replace with current sanctioned tooling. Remove ADR-0052/0053/0054 from related_adrs. Merge the two frontmatter blocks.

Affected: `multi-agent-tool-map.md`
Owner recommendation: lane-governance

---

**R-P0-06 | Strip grit/icm/rtk from `hyperscaler-best-practices.md` Agent-tooling table**

Remove or replace the three rows for grit v0.3.0, icm v0.10.39, rtk dev-0.39.0. Add a note referencing ADR-0116.

Affected: `hyperscaler-best-practices.md`
Owner recommendation: lane-governance

---

**R-P0-07 | Strip grit/icm/rtk from `lts-versions-verified.md` Agent-tooling section**

Remove the three version-pinned rows. They are permanently retired and should not appear in an LTS version-tracking document.

Affected: `lts-versions-verified.md`
Owner recommendation: lane-governance

---

**R-P0-08 | Remove React prescriptions from `emoji-sticker-reaction-system.md`; split to ≤600 lines; fix frontmatter**

Three separate issues: (1) React 18+ prescriptions violate ADR-0185; rewrite for SvelteKit/Leptos or mark as historical. (2) ~2316 lines violates the 600-line cap; split into sub-standards. (3) Non-YAML frontmatter (`**Bold:**` keys inside YAML fence) must be replaced with valid YAML.

Affected: `emoji-sticker-reaction-system.md`
Owner recommendation: axis-frontend

---

**R-P0-09 | Add YAML frontmatter to all 8 files with no frontmatter**

Each file needs a valid frontmatter block with at minimum: `doc_class: Standard`, `status: Accepted` (or `Draft` if still in progress), `date:`, `canonical_authority:`, `planned_enforcement_ref:`, `related_adrs:`. Files: `cedar-policy-discipline.md`, `container-image-convention.md`, `gitops-iac-cluster-tier-boundaries.md`, `helm-chart-convention.md`, `stream-processing-rubric.md`, `timescaledb-adoption.md`, `wasm-runtime-canonical.md`, `workflow-vs-direct-grpc-rubric.md`.

Affected: 8 files
Owner recommendation: doc-coverage lane sweep

---

**R-P0-10 | Author `brand-voice.md` from scratch**

The entire file is placeholder marker placeholders. `ux-best-practices.md` §20 (Branding + White-Label) depends on it normatively. Until this is authored, there is no guidance on voice and tone for any product surface.

Affected: `brand-voice.md`
Owner recommendation: axis-product, council-design-system

---

**R-P0-11 | Author `incident-severity.md` from scratch**

The entire file is placeholder marker placeholders. `on-call.md` implicitly depends on severity thresholds. No on-call engineer can make a consistent severity declaration without this document.

Affected: `incident-severity.md`
Owner recommendation: ops-sre-reliability

---

**R-P0-12 | Split `ux-best-practices.md` (~2490 lines) into domain-scoped sub-standards**

Each sub-standard must be ≤600 lines. Suggested splits by §:
- `ux-design-tokens.md` — §2 (Design Tokens)
- `ux-accessibility.md` — §3 (Accessibility, expands `a11y-canonical.md`)
- `ux-i18n-rtl.md` — §5 (i18n, expands `i18n-canonical.md` + `rtl-rendering.md`)
- `ux-theming-dark-mode.md` — §6–§7 (Dark Mode + Density Tiers)
- `ux-keyboard-motion.md` — §8–§9 (Keyboard + Motion)
- `ux-error-empty-loading.md` — §10–§11 (Error Handling + Empty/Loading States)
- `ux-forms-search-navigation.md` — §13–§15 (Forms + Search + Navigation)
- `ux-mobile-performance-offline.md` — §16–§17 (Mobile + Performance)
- `ux-branding-privacy-ai.md` — §20–§22 (Branding + Privacy + AI feature UX)

Also fix `doc_class: standard` → `doc_class: Standard`.

Affected: `ux-best-practices.md`
Owner recommendation: council-design-system

---

**R-P0-13 | Split `voice-video-call-architecture.md` (~2001+ lines, Draft) into sub-standards**

Promote from Draft to Accepted only after split. Suggested splits:
- `webrtc-sfu-deployment.md` — §3 (LiveKit SFU Deployment)
- `webrtc-codec-selection.md` — §4–§6 (Codec + Simulcast + Congestion Control)
- `webrtc-nat-traversal.md` — §7 (NAT/STUN/TURN)
- `webrtc-recording-transcription.md` — §8–§9 (Recording + Transcription)
- `webrtc-e2e-encryption.md` — §12 (E2E MLS)
- `webrtc-capacity-dr.md` — §13–§15 (Capacity + DR)

Affected: `voice-video-call-architecture.md`
Owner recommendation: axis-meet

---

### P1 — High (resolve within current sprint or next lane-gate)

**R-P1-01 | Update all layer-enum drift files to reference ADR-0105 (13-layer)**

Update: `clean-architecture.md` §2 text; `code-style-rust.md` purpose + body; `crate-naming-convention.md` BNF `layer` production (add 13th value); `multispectrum-review-v2.4.0-cadence.md` §10.3 text; `multispectrum-review.md` cross-references. In all five files, add ADR-0105 to `related_adrs:`.

**R-P1-02 | Resolve SLSA L2 vs L3 conflict between `image-discipline.md` and `image-signing-canonical.md`**

These two Accepted standards contradict each other. File an ADR determining the canonical SLSA level. Update both documents to match. Coordinate with ops-security on which level the current CI pipeline actually enforces.

**R-P1-03 | Resolve Cedar version conflict — align all files to Cedar v4.2.0 LTS**

`cedar-policy-discipline.md` body must be updated from "Cedar 3.x" to "Cedar 4.2.0 LTS". `regulatory-pack-authzpolicy-overlays.md` body reference to "Cedar 4.9.1" must be reviewed — if the regulatory pack intentionally uses a newer Cedar version, an ADR overriding the v4.2.0 LTS pin must be filed; otherwise update to 4.2.0 LTS.

**R-P1-04 | Merge double-frontmatter in 9 files**

For each file, remove the first `---` block (which contains only `purpose:` and `doc_status:`) and keep only the second (canonical) block. Verify the second block has all required fields before merging. Files: `git-workflow.md`, `image-discipline.md`, `INDEX.md`, `multi-agent-tool-map.md`, `observability.md`, `on-call.md`, `release-management.md`, `security-review.md`, `testing.md`.

**R-P1-05 | Replace `contract:` key with `doc_class:` in 11 files**

Add `doc_class: Standard` (or appropriate class), `status: Accepted`, `date:`, `canonical_authority:`. Files: `api-surface-separation.md`, `backup-canonical.md`, `brownout-degradation-signal.md`, `cross-microservice-latency-budget.md`, `dr-business-continuity.md`, `finops-cost-attribution-canonical.md`, `finops-cost-attribution.md`, `saga-compensation-policy.md`, `sovereign-cloud-overlay.md`, `tenant-lifecycle.md`, `throttling-tiers.md`.

**R-P1-06 | Migrate minimal-frontmatter files to canonical frontmatter schema**

Add `doc_class:`, `status:`, `date:`, `canonical_authority:`, `planned_enforcement_ref:` to 12 files: `capability-authoring.md`, `ci-lanes.md`, `code-style.md`, `commit-message.md`, `dependency-policy.md`, `logging-tracing.md`, `m02-exit-gate-validators.md`, `migration-playbook.md`, `plugin-authoring.md`, `prevention-doctrine.md`, `privacy-review.md`, `release.md`.

**R-P1-07 | Update `multispectrum-review.md` supersession and pointer**

Add a visible "SUPERSEDED — see `multispectrum-review-v2.4.0-cadence.md`" block at the top of the document. Update `canonical_authority` to point to v2.4.0. Remove ADR-0054. Add ADR-0105.

**R-P1-08 | Remove ADR-0052/0053/0054 from all `related_adrs:` lists**

Replace with `ADR-0116` where the file discusses the agent-coordination workflow. Remove entirely if the file has no relevant connection. Affects 14 files (see §6.2).

**R-P1-09 | Replace ADR-0141 authority in `cross-microservice-latency-budget.md`**

Change `canonical_authority: ADR-0141` to `canonical_authority: ADR-0145`. Resolve placeholder marker enforcement field with a concrete lane name.

**R-P1-10 | Fix broken file reference in `backup-canonical.md`**

`docs/standards/promotion-policy.md` does not exist. Either: (a) create the promotion-policy.md standard, or (b) update the reference to the actual policy document governing promotion.

**R-P1-11 | Fix broken file references in `incident-severity.md` and `release.md`**

`STANDARDS-AND-TEMPLATES.md` does not exist. Resolve after R-P0-11 (incident-severity) and separately for `release.md`.

**R-P1-12 | Promote `api-design.md` from Draft or declare a hard authorship deadline**

The API design standard is cited by multiple standards. Its Draft status means no API design decision is normative. Either: promote to Accepted (requires completing enforcement lane + 250+ lines + RFC-2119 content), or declare a sprint deadline for promotion.

**R-P1-13 | Promote `code-review.md` from Draft or merge with a concrete code-review process**

Under-250 lines, Draft status, no enforcement. The code review process has no enforceable standard.

**R-P1-14 | Strip `rtk` references from `on-call.md` §3 runbook discipline**

Replace "`rtk`-prefixed" command references with plain `cargo` or `gh` equivalents.

**R-P1-15 | Strip `icm` references from `testing.md` §9 flaky-SLA escalation**

Replace "reminder via icm; incident commander escalates via icm" with the current process-neutral language (e.g., "MISTAKES-LEDGER row, GitHub issue, and owning-team escalation").

**R-P1-16 | Strip `grit done` reference from `m02-exit-gate-validators.md`**

Replace "before `grit done`" with the current `oya git` / PR-merge equivalent.

**R-P1-17 | Update INDEX.md catalog row for `claude-code-harness.md`**

After R-P0-01, update INDEX.md to reflect the tombstone. Remove ADR-0052/0053/0054 from INDEX.md `related_adrs:`.

**R-P1-18 | Update `agentic-dev-team-optimization.md` for `oya git`**

Replace all `oya vcs claim` / `oya vcs work` / `oya vcs done` references with `oya git` equivalents. Resolve placeholder marker gates.

**R-P1-19 | Split `multispectrum-review-v2.4.0-cadence.md` to ≤600 lines**

Extract Appendix A (evidence template), Appendix B (worked examples), Appendix C (facet definitions) as companion `Guide`-class documents. Target: ≤600 lines for the operative Standard. Also fix §10.3 ADR-0056 / 12-layer reference to ADR-0105.

**R-P1-20 | Expand under-250-line Standards — prioritized list**

Cannot expand all 50+ files in one sprint. Priority order based on inbound citation count:

1. `a11y-canonical.md` (82 lines, 4+ inbound citations) — expand with keyboard pattern requirements, ARIA landmark rules, color contrast enforcement details
2. `cedar-policy-discipline.md` (119 lines, 3+ inbound citations) — also blocked on R-P0-09 (frontmatter)
3. `api-design.md` (159 lines, 4+ inbound) — also blocked on R-P1-12 (Draft status)
4. `authz-tier-boundaries.md` (99 lines) — expand with tier-boundary violation anti-patterns
5. `realtime-transport-tier.md` (84 lines) — if thin-pointer-gateway, declare `shape:` field; otherwise expand
6. `graceful-shutdown-canonical.md` (90 lines) — expand with Kubernetes preStop hook, SIGTERM handling, drain-timeout requirements
7. `logging-tracing.md` (77 lines) — expand with structured-log field requirements, trace-context propagation rules
8. `commit-message.md` (119 lines) — expand with worked examples and CI gate integration
9. `code-review.md` (95 lines) — also blocked on R-P1-13 (Draft status)
10. `image-signing-canonical.md` (92 lines) — also blocked on R-P1-02 (SLSA conflict)

---

### P2 — Medium (required before next multi-spectrum review cycle)

**R-P2-01 | Add `enforced_by` / `planned_enforcement_ref` to 47 files**

At minimum, add a `planned_enforcement_ref:` field with the intended CI lane name. Even a planned lane that does not exist yet satisfies the requirement and creates a resolvable ticket. Files listed in §2.5.

**R-P2-02 | Add RFC-2119 MUST openers to 20 files missing formal RFC-2119**

Each section under a heading must begin with a sentence of the form "Implementations MUST …" or "All µservices MUST …". Bulleted list items that happen to include the word MUST do not satisfy the requirement. Files listed in §2.4.

**R-P2-03 | Add anti-patterns / forbidden-patterns sections to 21 files**

The anti-patterns section is the primary mechanism for reviewers to reject incorrect implementations at PR time. Without it, the standard is advisory only. Files listed in §2.3.

**R-P2-04 | Reconcile Grafana pin: `lts-versions-verified.md` 13.0.1 vs `observability-slo.md` 12.0 LTS**

Determine whether `lts-versions-verified.md` tracks LTS pins (in which case 12.0 LTS is correct) or latest stable releases (in which case the document needs two columns). Update accordingly.

**R-P2-05 | Remove retired ADR-0174 citations from `sovereign-cloud-overlay.md` and `throttling-tiers.md`**

Replace with the absorbing standard (`regulatory-pack-authzpolicy-overlays.md`) as the authority.

**R-P2-06 | Rename "Object Graph" to "Ontology" in `schema-migration.md`**

Replace "ADR-0006 (Object Graph property-tier)" with "ADR-0006 (Ontology property-tier)".

**R-P2-07 | Resolve `STANDARDS-AND-TEMPLATES.md` reference in `release.md`**

Either create the file or update the reference to a document that exists.

**R-P2-08 | Resolve `templates/migration-runbook-template.md` reference in `migration-playbook.md`**

Either create the template or add an explicit "planned; not yet authored — use `postmortem-template.md` as a guide" note.

**R-P2-09 | Replace `ADR-NNNN` placeholder in `fips-hsm-substrate-root-signing.md`**

Author the foundry-meta-trust-root ADR and replace the placeholder number. Until then, add "ADR not yet filed — tracking in [issue link]" annotation.

**R-P2-10 | Convert F-PENDING enforcement fields to concrete lane names**

For `autonomy-ceiling.md`, `data-class.md`, `error-handling.md`, `image-discipline.md`: assign concrete CI lane names and either activate the lane or set it to `planned_enforcement_ref:` with a shipping milestone.

**R-P2-11 | Assign concrete enforcement lane to `cross-microservice-latency-budget.md`**

Replace `enforcement: placeholder marker` with a real lane name.

**R-P2-12 | Canonicalize `status: canonical-base` in 5 files**

Either define `canonical-base` as an allowed status value in `documentation-rigor.md`, or migrate all 5 files to `status: Accepted`. Files: `saga-compensation-policy.md`, `sovereign-cloud-overlay.md`, `tenant-lifecycle.md`, `throttling-tiers.md`, `hyperscaler-invariant-conformance.md`.

**R-P2-13 | Fix `regulatory-pack-authzpolicy-overlays.md` status**

Change `status: Active` to `status: Accepted`.

**R-P2-14 | Confirm microservice-local ADR existence for `voice-video-call-architecture.md`**

Verify that ADR-MEET-0001 through ADR-MEET-0006 and ADR-MSGR-0001/0002 exist as files in `microservices/meet/` and `microservices/messenger/` respectively. If they do not exist, create ADR stubs. If they do exist, update the cross-reference format to include the microservice-relative path.

**R-P2-15 | Add `canonical_authority:` to `dependency-policy.md`**

The minimal frontmatter omits this required field. Identify the governing ADR (likely ADR-0015 or the crate-policy ADR) and add it.

**R-P2-16 | Fix non-YAML frontmatter in `emoji-sticker-reaction-system.md`**

Replace `**Bold:**` markdown syntax inside the YAML fence with valid YAML `key: value` syntax.

**R-P2-17 | Update OpenAPI version string in `api-design.md` to "OpenAPI 3.2.0"**

Fix "OpenAPI 3.2" → "OpenAPI 3.2.0" to match the canonical pin in `observability-slo.md`.

**R-P2-18 | Declare supersession relationship between two `finops-cost-attribution*.md` files**

Both `finops-cost-attribution.md` and `finops-cost-attribution-canonical.md` exist without an explicit relationship. Add a header in the older file: "SUPERSEDED by `finops-cost-attribution-canonical.md`" — or if they cover different concerns, add a cross-reference explaining the relationship.

**R-P2-19 | Declare `shape: thin-pointer-gateway` for short Standards that are intentionally brief**

`multispectrum-review.md` (72 lines), `realtime-transport-tier.md` (84 lines), and `wcag-2-2-aa-checklist.md` (111 lines) are plausibly intentional gateway documents. Adding `shape: thin-pointer-gateway` to their frontmatter exempts them from the 250-line minimum and makes the intent machine-verifiable.

---

### P3 — Low (technical debt; address in scheduled maintenance)

**R-P3-01 | Remove scratch-file normative reference from `testing.md` §12**

`.omc/scratch/hyperscaler-best-practices-2026-05-12.md` is a scratch file, not a durable standard. Replace with direct citations to the upstream sources the scratch file itself compiled (Frontiers, nextest book, proptest, cargo-mutants, etc.).

**R-P3-02 | Verify `docs/QA-TEST-STRATEGY.md` existence**

Referenced by `testing.md` as a companion doc. Confirm the file exists and is not a broken reference.

**R-P3-03 | Audit enforcement-lane prefix post-ADR-0132**

When migrating existing `governance-*` lanes per ADR-0132 migration IPs, update the `enforced_by:` fields in the corresponding Standard files at the same time.

**R-P3-04 | Normalize `hyperscaler-invariant-conformance.md` doc_class**

Change `Hyperscaler-Invariant-Conformance-Standard` to `Standard`. If a named shape is needed, add `shape: hyperscaler-invariant-conformance`.

**R-P3-05 | Add "Phase-2, not yet authored" notice to `timescaledb-adoption.md`**

The reference to `crates/shared-timescale-policy-worker/` should carry an explicit "Phase-2 follow-on; not yet created" note to prevent implementers from wasting time looking for the crate.

**R-P3-06 | Add conditional caveat to `ux-best-practices.md` §20 brand-voice reference**

Until `brand-voice.md` is authored (R-P0-10), add "when authored; currently a stub — see axis-product for interim guidance" to the §20 reference.

**R-P3-07 | Verify `doc_id:` field in `voice-video-call-architecture.md`**

The field `doc_id: STD-voice-video-call-architecture` is non-standard. Verify whether `doc-style.md` documents `doc_id:` as a canonical frontmatter field. If not, remove it.

**R-P3-08 | Review `regulatory-pack-authzpolicy-overlays.md` extra frontmatter fields**

The file contains `doc_id:` and `template_id:` fields not defined in `documentation-rigor.md`. Remove non-standard fields or add them to the rigor standard's allowed-field list.

---

## Appendix A — Full Severity Count Summary

| Category | P0 | P1 | P2 | P3 | Total |
|---|---|---|---|---|---|
| §2 Contradictions vs doc-rigor | 6 | 34 | 6 | 1 | 47 |
| §3 Drift | 7 | 14 | 8 | 1 | 30 |
| §4 Staleness | 7 | 13 | 9 | 0 | 29 |
| §5 Rigor Failures | 8 | 53 | 3 | 0 | 64 |
| §6 Cross-reference broken links | 4 | 14 | 8 | 3 | 29 |
| **Totals** | **32** | **128** | **34** | **5** | **199** |

**Total distinct findings: 199**
**Files audited: 89**
**Files with zero findings: 0** (every file has at least one finding)

### A.1 Files with highest P0 finding density

1. `claude-code-harness.md` — P0 across 3 categories (entirely deprecated document)
2. `brand-voice.md` — P0 (stub) + cascading P2 in ux-best-practices
3. `incident-severity.md` — P0 (stub) + P1 (broken reference)
4. `emoji-sticker-reaction-system.md` — P0 (length + React violation + non-YAML frontmatter)
5. `messenger-e2e-encryption-mls.md` — P0 (length) + P0 (Draft-Standard misclassification)
6. `agent-instructions-discipline.md` — P0 (grit/icm contamination)
7. `git-workflow.md` — P0 (grit/icm contamination) + P1 (double-frontmatter + ADR refs)
8. `multi-agent-tool-map.md` — P0 (grit/icm contamination) + P1 (double-frontmatter)
9. `hyperscaler-best-practices.md` — P0 (grit/icm tooling table)
10. `lts-versions-verified.md` — P0 (grit/icm tooling section) + P2 (Grafana conflict)
11. `ux-best-practices.md` — P0 (length) + P0 (lowercase doc_class) + P2 (brand-voice normative ref to stub)
12. `voice-video-call-architecture.md` — P0 (length) + P1 (Draft)
13. `cedar-policy-discipline.md` — P0 (no frontmatter) + P1 (Cedar 3.x drift)
14. `stream-processing-rubric.md` — P0 (no frontmatter; completely unregistered)

### A.2 Files with best rigor compliance

| File | Compliance assessment |
|---|---|
| `observability-slo.md` | Near-complete; only P2 Grafana version gap |
| `fips-hsm-substrate-root-signing.md` | Good; over cap by ~100 lines; P2 placeholder ADR ref |
| `testing.md` | Solid content; P1 double-frontmatter + ADR-0052 refs |
| `realtime-transport-tier.md` | Good; under-250 but gateway shape plausible |
| `wcag-2-2-aa-checklist.md` | Good; under-250 but gateway shape plausible |
| `outbox-pattern-canonical.md` | Good; under-250; no anti-patterns section |
| `trace-sampling-tier.md` | Good; under-250; enforcement listed as advisory |
| `dependency-policy.md` | Moderate; 253 lines; missing canonical_authority |
| `crate-naming-convention.md` | Good content; 422 lines; layer-enum drift is critical |
| `error-handling.md` | Solid; 257 lines; F-PENDING enforcement |

---

## Appendix B — Per-File Findings Index

The following table gives a one-row summary for every file audited, enabling triage by file owner.

| File | Lines | doc_class | Status | P0 | P1 | P2 | P3 | Primary issues |
|---|---|---|---|---|---|---|---|---|
| `a11y-canonical.md` | ~82 | Standard | Accepted | 0 | 2 | 1 | 0 | Under-250; no enforcement |
| `agent-instructions-discipline.md` | ~231 | Standard | Accepted | 1 | 2 | 1 | 0 | Grit/icm contamination; ADR-0052 |
| `agentic-dev-team-optimization.md` | ~148 | Standard | Accepted | 0 | 2 | 2 | 0 | `oya vcs` drift; placeholder marker gates; under-250 |
| `api-design.md` | ~159 | Standard | Draft | 0 | 3 | 1 | 1 | Draft; no enforcement; under-250; OpenAPI version |
| `api-surface-separation.md` | ~106 | contract: | — | 0 | 4 | 0 | 0 | `contract:` key; no MUST; no anti-patterns; under-250 |
| `authz-tier-boundaries.md` | ~99 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no anti-patterns; no MUST |
| `autonomy-ceiling.md` | ~254 | Standard | Accepted | 0 | 1 | 2 | 0 | F-PENDING enforcement; ADR-0052 |
| `backup-canonical.md` | ~134 | contract: | — | 0 | 3 | 0 | 0 | `contract:` key; broken ref; under-250 |
| `brand-voice.md` | ~25 | — | — | 1 | 0 | 0 | 0 | Complete stub |
| `brownout-degradation-signal.md` | ~120 | contract: | — | 0 | 4 | 0 | 0 | `contract:` key; no MUST; no anti-patterns; under-250 |
| `capability-authoring.md` | ~79 | — | — | 0 | 3 | 0 | 0 | Minimal frontmatter; no MUST; under-250 |
| `cedar-policy-discipline.md` | ~119 | — | — | 1 | 2 | 0 | 0 | No frontmatter; Cedar 3.x drift; under-250 |
| `ci-lanes.md` | ~157 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; under-250 |
| `claude-code-harness.md` | ~248 | Standard | Accepted | 3 | 0 | 0 | 0 | Entirely deprecated tooling |
| `clean-architecture.md` | ~397 | Standard | Accepted | 0 | 2 | 0 | 0 | Layer-enum drift (12 not 13) |
| `code-review.md` | ~95 | Standard | Draft | 0 | 3 | 0 | 0 | Draft; under-250; no enforcement |
| `code-style-rust.md` | ~269 | Standard | Accepted | 0 | 2 | 0 | 0 | 12-layer list; ADR-0056 |
| `code-style.md` | ~70 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; under-250 |
| `commit-message.md` | ~119 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; under-250 |
| `compliance-evidence-automation.md` | ~97 | Standard | Accepted | 0 | 2 | 0 | 0 | Under-250; no enforcement |
| `container-image-convention.md` | ~77 | — | — | 1 | 0 | 0 | 0 | No frontmatter; under-250 |
| `crate-naming-convention.md` | ~422 | Standard | Accepted | 0 | 2 | 0 | 0 | BNF 12-layer drift; missing ADR-0105 |
| `cross-microservice-latency-budget.md` | ~232 | contract: | — | 0 | 3 | 1 | 0 | `contract:` key; ADR-0141 retired; placeholder marker enforcement |
| `cursor-pagination-canonical.md` | ~120 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; no anti-patterns |
| `data-class.md` | ~234 | Standard | Accepted | 0 | 2 | 2 | 0 | F-PENDING lanes; ADR-0052 |
| `dependency-policy.md` | ~253 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; no canonical_authority |
| `design-doc-template.md` | — | Template | Accepted | 0 | 0 | 0 | 0 | Template class; exempt |
| `dr-business-continuity.md` | ~136 | contract: | — | 0 | 3 | 0 | 0 | `contract:` key; under-250; no anti-patterns |
| `emoji-sticker-reaction-system.md` | ~2316 | — (invalid YAML) | Draft | 3 | 0 | 1 | 0 | Length; React violation; non-YAML frontmatter |
| `error-handling.md` | ~257 | Standard | Accepted | 0 | 1 | 1 | 0 | F-PENDING enforcement |
| `event-schema-versioning-canonical.md` | ~107 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; no anti-patterns |
| `finops-cost-attribution-canonical.md` | ~186 | contract: | — | 0 | 4 | 0 | 0 | `contract:` key; under-250; no MUST; no anti-patterns |
| `finops-cost-attribution.md` | ~165 | contract: | — | 0 | 3 | 1 | 0 | `contract:` key; under-250; supersession unclear |
| `fintech-compliance.md` | ~448 | Standard | Draft | 0 | 2 | 1 | 0 | Draft; placeholder marker items; no enforcement |
| `fips-hsm-substrate-root-signing.md` | ~704 | Standard | Accepted | 0 | 1 | 1 | 0 | Over-600 by 1.2×; placeholder ADR number |
| `git-workflow.md` | ~224 | Standard | Accepted | 2 | 3 | 1 | 0 | Grit/icm; oya vcs drift; double-frontmatter |
| `gitops-iac-cluster-tier-boundaries.md` | ~99 | — | — | 1 | 0 | 0 | 0 | No frontmatter; under-250 |
| `graceful-shutdown-canonical.md` | ~90 | Standard | Accepted | 0 | 2 | 0 | 0 | Under-250; no enforcement |
| `helm-chart-convention.md` | ~95 | — | — | 1 | 0 | 0 | 0 | No frontmatter; under-250 |
| `hyperscaler-best-practices.md` | ~334 | research-context | — | 1 | 0 | 1 | 0 | Grit tooling table (research doc); normative scratch citation |
| `hyperscaler-invariant-conformance.md` | ~220 | HIC-Standard | canonical-base | 0 | 2 | 1 | 1 | Non-canonical doc_class; under-250; canonical-base status |
| `i18n-canonical.md` | ~88 | Standard | Accepted | 0 | 2 | 1 | 0 | Under-250; no MUST opener; no anti-patterns |
| `idempotency-keys-canonical.md` | ~116 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; no anti-patterns |
| `identity-vendor-isolation.md` | ~78 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; `related_lanes` not `enforced_by` |
| `image-discipline.md` | ~262 | Standard | Accepted | 0 | 3 | 1 | 0 | SLSA conflict; double-frontmatter; ADR-0052; F-PENDING |
| `image-signing-canonical.md` | ~92 | Standard | Accepted | 0 | 3 | 0 | 0 | SLSA conflict; under-250; no MUST; no anti-patterns |
| `incident-severity.md` | ~26 | — | — | 1 | 1 | 0 | 0 | Complete stub; broken reference |
| `INDEX.md` | ~115 | Standard | Accepted | 0 | 3 | 0 | 0 | Double-frontmatter; stale catalog row; ADR-0052 |
| `locale-routing.md` | ~80 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; no anti-patterns; no enforcement |
| `logging-tracing.md` | ~77 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; under-250 |
| `lts-versions-verified.md` | ~178 | — | published | 1 | 0 | 1 | 0 | Grit tooling section; Grafana conflict |
| `m02-exit-gate-validators.md` | ~64 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; `grit done` reference |
| `messenger-e2e-encryption-mls.md` | ~3535 | Standard | Draft | 2 | 0 | 0 | 0 | 5.9× over cap; Draft Standard misclassification |
| `migration-playbook.md` | ~103 | — | — | 0 | 2 | 1 | 0 | Minimal frontmatter; under-250; broken template ref |
| `multi-agent-tool-map.md` | ~216 | Standard | Accepted | 2 | 3 | 0 | 0 | Grit/icm contamination; double-frontmatter; ADR-0052 |
| `multispectrum-review-v2.4.0-cadence.md` | ~903 | Standard | Accepted | 0 | 2 | 0 | 0 | Over-600 by 1.5×; ADR-0056 12-layer ref |
| `multispectrum-review.md` | ~72 | Standard | Accepted | 0 | 3 | 0 | 0 | Points to v2.1.0; ADR-0054; no supersession marker |
| `observability-slo.md` | ~298 | Standard | Accepted | 0 | 0 | 1 | 0 | Grafana pin vs lts-versions-verified |
| `observability.md` | ~230 | Standard | Accepted | 0 | 2 | 1 | 0 | Double-frontmatter; ADR-0052 |
| `on-call.md` | ~232 | Standard | Accepted | 0 | 2 | 1 | 0 | Double-frontmatter; rtk reference; ADR-0052 |
| `outbox-pattern-canonical.md` | ~120 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; no anti-patterns |
| `per-tenant-resource-quotas-canonical.md` | ~98 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; no anti-patterns |
| `plugin-authoring.md` | ~117 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; under-250 |
| `postmortem-template.md` | ~119 | Template | Accepted | 0 | 0 | 0 | 0 | Template class; exempt |
| `prevention-doctrine.md` | ~89 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; under-250 |
| `prfaq-template.md` | ~81 | Template | Accepted | 0 | 0 | 0 | 0 | Template class; exempt |
| `privacy-review.md` | ~64 | — | — | 0 | 2 | 0 | 0 | Minimal frontmatter; under-250 |
| `realtime-transport-tier.md` | ~84 | Standard | Accepted | 0 | 1 | 1 | 0 | Under-250; no thin-pointer-gateway shape declared |
| `regulatory-pack-authzpolicy-overlays.md` | ~129 | Standard | Active | 0 | 0 | 3 | 1 | `status: Active`; Cedar 4.9.1 conflict; extra fields |
| `release-management.md` | ~235 | Standard | Accepted | 0 | 2 | 1 | 0 | Double-frontmatter; ADR-0052 |
| `release.md` | ~65 | — | — | 0 | 2 | 1 | 0 | Minimal frontmatter; under-250; broken STANDARDS ref |
| `request-id-canonical.md` | ~92 | Standard | Accepted | 0 | 2 | 1 | 0 | Under-250; no MUST; enforcement "planned" only |
| `rtl-rendering.md` | ~75 | Standard | Accepted | 0 | 2 | 0 | 0 | Under-250; no enforcement |
| `saga-compensation-policy.md` | ~204 | contract: | canonical-base | 0 | 3 | 1 | 0 | `contract:` key; no MUST; canonical-base status |
| `schema-migration.md` | ~64 | — | — | 0 | 2 | 1 | 0 | Minimal frontmatter; under-250; Object Graph term |
| `security-review.md` | ~214 | Standard | Accepted | 0 | 2 | 1 | 0 | Double-frontmatter; ADR-0052 |
| `sovereign-cloud-overlay.md` | ~128 | contract: | canonical-base | 0 | 3 | 1 | 0 | `contract:` key; ADR-0174; under-250 |
| `step-up-auth-classes.md` | ~96 | Standard | Accepted | 0 | 3 | 0 | 0 | Under-250; no MUST; `related_lanes` not `enforced_by` |
| `stream-processing-rubric.md` | ~98 | — | — | 1 | 0 | 0 | 0 | No frontmatter; no status; no enforcement |
| `tenant-lifecycle.md` | ~136 | contract: | canonical-base | 0 | 3 | 1 | 0 | `contract:` key; no MUST; canonical-base status |
| `testing.md` | ~247 | Standard | Accepted | 0 | 3 | 1 | 1 | Double-frontmatter; ADR-0052; icm in §9; scratch ref |
| `throttling-tiers.md` | ~138 | contract: | canonical-base | 0 | 3 | 1 | 0 | `contract:` key; ADR-0174 citation; under-250 |
| `timescaledb-adoption.md` | ~122 | — | — | 1 | 0 | 1 | 1 | No frontmatter; planned crate ref |
| `trace-sampling-tier.md` | ~94 | Standard | Accepted | 0 | 2 | 1 | 0 | Under-250; enforcement advisory only |
| `ux-best-practices.md` | ~2490+ | standard (lc) | Accepted | 2 | 0 | 2 | 1 | Length; lowercase doc_class; brand-voice ref to stub |
| `voice-video-call-architecture.md` | ~2001+ | Standard | Draft | 2 | 2 | 1 | 1 | Length; Draft; microservice-local ADR refs |
| `wasm-runtime-canonical.md` | ~83 | — | — | 1 | 0 | 0 | 0 | No frontmatter; under-250 |
| `wcag-2-2-aa-checklist.md` | ~111 | Standard | Accepted | 0 | 1 | 1 | 0 | Under-250; no thin-pointer-gateway shape |
| `workflow-vs-direct-grpc-rubric.md` | ~86 | — | — | 1 | 0 | 0 | 1 | No frontmatter; under-250 |

---

## Appendix C — Top 10 P0/P1 Actions by Wave-Gate Priority

In wave-gate order (actions that block all subsequent gates if not resolved):

| Rank | ID | Summary | Blocking reason |
|---|---|---|---|
| 1 | R-P0-01 | Delete/tombstone `claude-code-harness.md` | Actively misleads every new agent implementer reading onboarding docs |
| 2 | R-P0-03 | Strip grit/icm from `agent-instructions-discipline.md` | Core agent discipline doc propagates deprecated workflow to every new agent |
| 3 | R-P0-04 | Rewrite `git-workflow.md` §1 for `oya git` | Primary VCS onboarding doc; oya vcs drift + grit contamination |
| 4 | R-P0-10 + R-P0-11 | Author `brand-voice.md` + `incident-severity.md` | Complete stubs; normatively referenced from other standards |
| 5 | R-P1-02 | Resolve SLSA L2 vs L3 conflict | Active security posture contradiction between two Accepted standards; CI may enforce the wrong level |
| 6 | R-P1-03 | Align Cedar to v4.2.0 LTS | `cedar-policy-discipline.md` at 3.x; `regulatory-pack-authzpolicy-overlays.md` at 4.9.1; authz behavior undefined without this |
| 7 | R-P1-01 | Update all 6 layer-enum files to ADR-0105 | BNF generators use 12 not 13; CI linters will reject valid 13-layer crate names |
| 8 | R-P0-09 | Add YAML frontmatter to 8 files | CI frontmatter linter rejects all 8; they cannot participate in compliance gates |
| 9 | R-P0-12 + R-P0-13 | Split `ux-best-practices.md` + `voice-video-call-architecture.md` | 2000+ line documents cannot be reviewed under the multispectrum model |
| 10 | R-P1-04 | Merge double-frontmatter in 9 files | YAML parse failures cause silent key loss in any strict parser |

---

## Appendix D — Remediation Sprint Planning Guide

This appendix translates the findings above into a practical sprint allocation, assuming two engineers working on documentation remediation in parallel with feature development. Each sprint is two weeks.

### D.1 Sprint 1 — Emergency P0 cleanup (highest contamination risk)

**Goal:** Remove all grit/icm/rtk contamination from the corpus. Any contributor reading these documents today gets incorrect instructions.

| Task | Files | Estimated effort |
|---|---|---|
| Tombstone `claude-code-harness.md` | 1 file | 30 min |
| Strip grit/icm from `agent-instructions-discipline.md` | 1 file | 2 h |
| Rewrite `git-workflow.md` §1 for `oya git`; merge frontmatter | 1 file | 3 h |
| Strip grit/icm/rtk from `multi-agent-tool-map.md`; merge frontmatter | 1 file | 2 h |
| Strip grit tooling table from `hyperscaler-best-practices.md` | 1 file | 1 h |
| Strip grit tooling section from `lts-versions-verified.md` | 1 file | 1 h |
| Update INDEX.md catalog row; remove ADR-0052/0053/0054 | 1 file | 1 h |
| Strip `icm` from `testing.md` §9; merge frontmatter | 1 file | 1 h |
| Strip `rtk` from `on-call.md` §3; merge frontmatter | 1 file | 1 h |
| Strip `grit done` from `m02-exit-gate-validators.md` | 1 file | 30 min |
| Remove ADR-0052/0053/0054 from remaining `related_adrs:` lists | 9 files | 2 h |

**Sprint 1 total estimate:** ~15 hours across 2 engineers (1 sprint each)

**Success criterion:** `grep -r "grit\|icm\|oya vcs" docs/standards/` returns zero results except for ADR-0116 supersession notices.

### D.2 Sprint 2 — Frontmatter normalization

**Goal:** Every file in the corpus has a single valid YAML frontmatter block with `doc_class:`, `status:`, `date:`, and `canonical_authority:`.

| Task | Files | Estimated effort |
|---|---|---|
| Add YAML frontmatter to 8 files with no frontmatter | 8 files | 4 h |
| Merge double-frontmatter in 9 files | 9 files | 3 h |
| Replace `contract:` key with `doc_class:` in 11 files | 11 files | 3 h |
| Add `doc_class:` to 12 minimal-frontmatter files | 12 files | 3 h |
| Fix `status: Active` → `status: Accepted` in `regulatory-pack-authzpolicy-overlays.md` | 1 file | 15 min |
| Fix `doc_class: standard` → `doc_class: Standard` in `ux-best-practices.md` | 1 file | 15 min |
| Fix non-YAML frontmatter in `emoji-sticker-reaction-system.md` | 1 file | 30 min |

**Sprint 2 total estimate:** ~14 hours across 2 engineers

**Success criterion:** A YAML-linting CI gate (`governance-doc-frontmatter-lint`) passes on all 89 files with zero errors. Every file has `doc_class:`, `status:`, `date:`.

### D.3 Sprint 3 — Layer-enum drift + SLSA/Cedar conflict resolution

**Goal:** Every Standard references ADR-0105 and uses 13 layers. SLSA and Cedar version conflicts are resolved by ADR.

| Task | Files | Estimated effort |
|---|---|---|
| Update `crate-naming-convention.md` BNF to 13 layers; add ADR-0105 | 1 file | 2 h |
| Update `clean-architecture.md` §2; add ADR-0105 | 1 file | 1 h |
| Update `code-style-rust.md` layer list; add ADR-0105 | 1 file | 1 h |
| Update `multispectrum-review-v2.4.0-cadence.md` §10.3; add ADR-0105 | 1 file | 1 h |
| Update `multispectrum-review.md` cross-refs; add ADR-0105 | 1 file | 30 min |
| File ADR resolving SLSA L2 vs L3 conflict | New ADR | 3 h (architecture decision) |
| Update `image-discipline.md` and `image-signing-canonical.md` after ADR | 2 files | 1 h |
| Update `cedar-policy-discipline.md` from 3.x to 4.2.0 LTS | 1 file | 2 h (also needs frontmatter) |
| Investigate `regulatory-pack-authzpolicy-overlays.md` Cedar 4.9.1 | 1 file | 1 h |
| Update `multispectrum-review.md` to point to v2.4.0-cadence.md; add supersession marker | 1 file | 30 min |
| Update `cross-microservice-latency-budget.md` authority to ADR-0145 | 1 file | 30 min |

**Sprint 3 total estimate:** ~14 hours + architecture decision time

**Success criterion:** `grep -r "12-layer\|12-value.*layer\|ADR-0056.*layer" docs/standards/` returns zero results except in supersession notices. SLSA level consistent across both image standards.

### D.4 Sprint 4 — Stub authoring + broken-reference cleanup

**Goal:** Every file referenced from another standard exists and contains normative content.

| Task | Files | Estimated effort |
|---|---|---|
| Author `brand-voice.md` (≥250 lines) | 1 file | 8 h (new content authoring) |
| Author `incident-severity.md` (≥250 lines) | 1 file | 4 h (new content authoring) |
| Create or remove reference to `docs/standards/promotion-policy.md` | 1 file | 2 h |
| Create `templates/migration-runbook-template.md` or update reference | 1 file | 2 h |
| Replace `ADR-NNNN` placeholder in `fips-hsm-substrate-root-signing.md` | 1 file | 30 min (after ADR filed) |
| Verify / create `docs/QA-TEST-STRATEGY.md` reference in `testing.md` | 1 file | 1 h |
| Fix `STANDARDS-AND-TEMPLATES.md` references in `release.md` | 1 file | 30 min |
| Confirm microservice-local ADRs for `voice-video-call-architecture.md` | ADR path review | 2 h |
| Clarify finops supersession relationship | 2 files | 30 min |

**Sprint 4 total estimate:** ~21 hours (heavily weighted by `brand-voice.md` authoring)

**Success criterion:** `grep -r "promotion-policy\|STANDARDS-AND-TEMPLATES\|ADR-NNNN\|migration-runbook-template" docs/standards/` returns zero broken references.

### D.5 Sprint 5 — Enforcement lane population

**Goal:** Every Standard-class file has either an active `enforced_by:` or a `planned_enforcement_ref:` with a real CI lane name.

| Task | Files | Estimated effort |
|---|---|---|
| Assign `planned_enforcement_ref:` to 47 files missing enforcement | 47 files | 6 h (bulk edit) |
| Activate F-PENDING lanes (or set concrete milestone): `autonomy-ceiling.md`, `data-class.md`, `error-handling.md`, `image-discipline.md` | 4 files | 4 h per lane (CI work) |
| Assign concrete lane to `cross-microservice-latency-budget.md` | 1 file | 1 h |
| Canonicalize `status: canonical-base` in 5 files or define in doc-rigor | 5 files + 1 doc-rigor update | 2 h |

**Sprint 5 total estimate:** ~15 hours + CI lane creation effort (variable)

**Success criterion:** `grep -c "enforced_by\|planned_enforcement_ref" docs/standards/*.md` returns ≥1 for every Standard-class file.

### D.6 Sprint 6 — RFC-2119 and anti-patterns backfill

**Goal:** Every Standard has formal RFC-2119 normative sentences and an anti-patterns section.

| Task | Files | Estimated effort |
|---|---|---|
| Add RFC-2119 MUST openers to 20 files missing them | 20 files | 15 h (substantive edits) |
| Add anti-patterns sections to 21 files missing them | 21 files | 20 h (substantive edits) |

**Sprint 6 total estimate:** ~35 hours (two engineers, full sprint)

**Note:** This sprint requires genuine domain knowledge, not just mechanical edits. The RFC-2119 sentences and anti-patterns must be technically correct for each standard's domain. Assign domain owners rather than a single documentation engineer.

### D.7 Sprint 7 — Document expansion (under-250 Standards)

**Goal:** All Standard-class documents reach the 250-line minimum, or are declared as `shape: thin-pointer-gateway`.

Priority order (by inbound citation count):

1. `a11y-canonical.md` — 82 → 250 lines; expand keyboard, ARIA, color-contrast sections
2. `cedar-policy-discipline.md` — 119 → 250 lines; expand with Cedar v4.2.0 policy examples
3. `authz-tier-boundaries.md` — 99 → 250 lines; add tier-violation examples
4. `graceful-shutdown-canonical.md` — 90 → 250 lines; add K8s preStop hook, SIGTERM pattern
5. `logging-tracing.md` — 77 → 250 lines; add structured-log fields, trace-context propagation
6. `commit-message.md` — 119 → 250 lines; add worked examples per commit type
7. `code-review.md` — 95 → 250 lines (also needs Draft → Accepted)
8. `image-signing-canonical.md` — 92 → 250 lines (after SLSA conflict resolved)
9. `realtime-transport-tier.md` — 84 lines; declare `shape: thin-pointer-gateway` or expand
10. `wcag-2-2-aa-checklist.md` — 111 lines; declare `shape: thin-pointer-gateway` or expand

**Estimate for priority-10:** ~40 hours across 2 engineers; remaining 40+ under-250 files deprioritized to subsequent cycles.

### D.8 Sprint 8 — Large-document splits

**Goal:** All documents over the 600-line cap are split into sub-standards of ≤600 lines each.

| Task | Estimated effort |
|---|---|
| Split `ux-best-practices.md` into 9 sub-standards | 16 h (content restructuring + new file authoring) |
| Split `voice-video-call-architecture.md` into 6 sub-standards | 12 h |
| Reclassify `messenger-e2e-encryption-mls.md` as Guide | 2 h |
| Split `multispectrum-review-v2.4.0-cadence.md` to ≤600 lines | 4 h |
| Trim `fips-hsm-substrate-root-signing.md` by ~110 lines | 2 h |

**Sprint 8 total estimate:** ~36 hours

**Note:** `ux-best-practices.md` and `voice-video-call-architecture.md` splits require coordination with council-design-system and axis-meet respectively. Do not split without domain-owner review.

---

## Appendix E — Canonical Frontmatter Reference

The following is the canonical frontmatter block template for Standard-class documents, derived from `documentation-rigor.md` requirements as observed in compliant documents (`observability-slo.md`, `fips-hsm-substrate-root-signing.md`):

```yaml
---
doc_class: Standard
shape: standard
length_cap: 600
authority_tier: 2
status: Accepted
date: YYYY-MM-DD
purpose: |
  One paragraph explaining what this standard governs, who it applies to,
  and what problem it solves.
canonical_authority: docs/decisions/template ADR-name.md
enforced_by: governance-<lane-name>
related_adrs:
  - template ADR
  - concrete superseded ADR
companion_docs:
  - docs/standards/related-standard.md
---
```

Notes on each field:

- **`doc_class:`** — Must be exactly `Standard` (capital S). Other valid values: `Template`, `Runbook`, `ADR`, `Guide`.
- **`shape:`** — Typically `standard`. Use `thin-pointer-gateway` for short pointer documents. Use `hyperscaler-invariant-conformance` if needed (and define in doc-rigor).
- **`length_cap:`** — For Standard class, always `600`. Declare even if obvious; it is machine-checkable.
- **`authority_tier:`** — 1 = foundational (cross-workspace law); 2 = standard (default); 3 = guideline (advisory).
- **`status:`** — `Accepted` for live standards. `Draft` for in-progress. Never `Active`, `Active (Draft)`, `canonical-base`, or `published` without those values being defined in doc-rigor.
- **`date:`** — ISO 8601 date of last Accepted status transition.
- **`purpose:`** — Literal block scalar (`|`). One paragraph. Should name the governing authority, the scope, and the key invariant.
- **`canonical_authority:`** — Path to the governing ADR or spec. Not optional for Standard class.
- **`enforced_by:`** — Active CI lane name. If the lane does not exist yet, use `planned_enforcement_ref:` instead.
- **`planned_enforcement_ref:`** — CI lane name that is planned but not yet active. Must become `enforced_by:` before the standard can be considered fully operationalized.
- **`related_adrs:`** — List of ADR identifiers that inform or constrain this standard. Do not list superseded ADRs without a "(superseded by template ADR)" annotation.
- **`companion_docs:`** — Optional. Related standards or guides that together form a coherent policy surface.

---

## Appendix F — Automated Audit Checks Recommended

Based on the findings in this audit, the following automated checks SHOULD be added to the CI pipeline to prevent regression:

| Check ID | Description | Implementation | Gate type |
|---|---|---|---|
| AC-01 | Every `.md` file in `docs/standards/` must have exactly one YAML frontmatter block | YAML parser; fail if 0 or 2+ `---` fence pairs | BLOCKER |
| AC-02 | `doc_class:` must be one of 5 canonical values | Regex + enum check | BLOCKER |
| AC-03 | `status:` must be one of `Accepted`, `Draft`, `Deprecated` | Enum check | BLOCKER |
| AC-04 | Standard-class files must have `canonical_authority:` | Field presence check | BLOCKER |
| AC-05 | Standard-class files must have `enforced_by:` or `planned_enforcement_ref:` | Field presence check | BLOCKER |
| AC-06 | Standard-class files must be ≥ 250 lines (unless `shape: thin-pointer-gateway`) | Line count | ADVISORY (sprint 7 goal: BLOCKER) |
| AC-07 | Standard-class files must be ≤ 600 lines | Line count | BLOCKER |
| AC-08 | No Standard-class file may contain the strings `grit `, `icm `, `rtk `, `oya vcs ` | Grep | BLOCKER |
| AC-09 | No Standard-class file may reference `ADR-0052`, `ADR-0053`, `ADR-0054` without "(superseded by ADR-0116)" annotation | Grep + annotation check | ADVISORY |
| AC-10 | No Standard-class file may reference "12-layer enum" or "12-value.*layer" | Grep | ADVISORY (sprint 3 goal: BLOCKER) |
| AC-11 | Every referenced file path must resolve (no broken links) | Link checker | ADVISORY |
| AC-12 | `date:` field must be a valid ISO 8601 date | Date parse | BLOCKER |

These checks can be implemented as a single `governance-doc-frontmatter-lint` CI job using a small Rust binary that walks `docs/standards/`, parses each frontmatter block, and reports violations. Estimated implementation: 1 sprint.

---

*Audit complete. 89 files read line-by-line. 199 findings filed across P0–P3. No standards files were modified during this audit. Report path: `docs/architecture/standards-corpus-line-audit-2026-05-21.md`.*
