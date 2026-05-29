---
doc_class: Standard
status: Accepted
date: 2026-05-20
authority_tier: 2
shape: Reference
purpose: |
  Codify the v2.4.0 multispectrum-review cadence, per-facet dispatch rules,
  rate-limit ordering lessons, verdict-file convention, synthesizer step,
  per-ADR promotion review, lane sunset, anti-patterns, worked example, and
  references. This is the durable operative standard; thin-pointer-gateway
  doc-shape is intentionally NOT used here because this document IS the
  content (per documentation-rigor.md §2 Standard class requirements).
canonical_authority: /specs/multispectrum-review.json
companion_docs:
  - docs/standards/multispectrum-review.md (thin gateway, still valid)
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/keystone-bundle-2026-05-20-lessons-learned.md
related_adrs:
  - ADR-0056 (12/13-layer enum + BNF v4.1)
  - ADR-0062 (Quality/Performance/Scalability bar)
  - ADR-0069 (active-artifact-contract)
  - ADR-0092 (workspace dependency-seam policy)
  - ADR-0105 (13-layer enum canonical)
  - ADR-0145 (inter-microservice communication reform)
  - ADR-0242 (oyatie-is-a-tenant)
related_memories:
  - feedback_multispectrum_review_v22
  - feedback_multispectrum_adherence_facets
  - feedback_consensus_debate_spectrum_lens_subagents
  - feedback_codex_bulk_resolve_antipattern
  - feedback_no_silent_regression
planned_enforcement_ref: oya-check-dependency-seam (sub-check consensus-debate-evidence; NotYetArmed; BLOCKER from 2026-07-15)
version: 2.4.0
supersedes: multispectrum-review v2.2.0 (docs/standards/multispectrum-review.md thin-gateway baseline)
---

# Multispectrum Review v2.4.0 — Operative Cadence Standard

> **RFC-2119 usage.** The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL
> NOT, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this document are
> to be interpreted as described in RFC 2119.

---

## §1 Doctrine

### §1.1 Purpose

The multispectrum review is the mechanism by which architectural and code
changes receive independent critique across a fixed set of analytical lenses
before their CI enforcement lanes promote from advisory to BLOCKER. Its
invariant is: **no single agent or reviewer wears more than one lens on a
given change**.

The doctrine exists because bias-collapse — the failure mode where a single
reviewer unconsciously harmonises its own critique across facets — produces
reviews that look thorough but systematically suppress the uncomfortable
findings that any given facet, examined in isolation, would surface. The
2026-05-14 user directive formalised this as a structural requirement; v2.2.0
codified it; v2.4.0 adds rate-limit dispatch ordering, per-ADR promotion
review, and the worked example from the 2026-05-20 keystone bundle review.

### §1.2 The 21 Facets

Version 2.4.0 recognises 21 facets in three families.

**F-family (critique facets):** Applied to the substance of the change.

| Facet ID | Name | Lens summary | Recommended subagent_type |
|---|---|---|---|
| F1 | Correctness (Linus) | Is the logic correct? Are the invariants upheld? Would Linus Torvalds NACK this? | critic |
| F2 | Hyperscaler fitness | Does this match how AWS / GCP / Azure / Stripe operate at scale? Are the cited hyperscaler patterns actually used for this decision class? | architect |
| F3 | Readability | Can a programming-capable intern read and understand this? Is the prose/code clear? | critic |
| F4 | Architecture | Does this honour the 12/13-layer enum, inward-only flow, port-in-kernel, ADR-0145 no-universal-mediator? | architect |
| F5 | Security | Adversarial threat modelling: malicious tenant, compromised admin, supply-chain, nation-state | security-reviewer |
| F6 | Performance | Do latency/throughput budgets have measurement evidence? Is performance reasoning honest? | executor / scientist |
| F7 | Supply chain | Dependencies: pinned, audited, FIPS/HSM tier where required? | architect |
| F8 | Maintenance | Tech-debt, upgrade paths, observability for future maintainers | general-purpose |
| F9 | Operations | Runbooks, incident procedures, on-call surface; is the operator story complete? | verifier |
| F10 | Reversibility | Can this be undone? Is the rollback cost documented? | debugger |
| F11 | Observability | New HTTP routes, state machines, background workers — are they traced and metered? | tracer |
| F13 | Compliance / migration | Breaking changes, superseded ADRs, regulatory obligations | architect |

**M-family (meta facets):** Applied to the review corpus and process.

| Facet ID | Name | Lens summary | Recommended subagent_type |
|---|---|---|---|
| M1 | Challenge-assumption | Are the load-bearing premises argued or merely asserted? | critic (RED-TEAM posture) |
| M2 | Meta-review | Is the review process itself sound? Are facets independent? | architect |

**A-family (adherence facets):** Applied to own-policy compliance. The
closed-enum cap is RELAXED for the A-family: additional A-facets MAY be added
in future versions without requiring a bundle-wide re-review (per
`feedback_multispectrum_adherence_facets`).

| Facet ID | Name | Lens summary | Trigger condition | Recommended subagent_type |
|---|---|---|---|---|
| A1 | Naming | BNF v4.1 + 13-layer-enum conformance; naming-justification table present | Any new file, crate, ADR, or identifier introduced | critic |
| A2 | Documentation | Intern-buildability bar per `documentation-rigor.md`; doc-class matrix met | Any new or modified canonical doc | general-purpose |
| A3 | Structure | Per-µservice flat layout (ADR-0131); crate placement; directory conventions | Any new µservice, crate, or file placement | critic |
| A4 | Architecture adherence | Layer enum inward-only flow; port-in-kernel; no cross-layer leakage | Any kernel/adapter boundary change | architect |
| A5 | Dependency | New `Cargo.toml` dependency; semver pinning; workspace policy (ADR-0092) | Any new dependency | verifier |
| A6 | Schema | New or modified JSON Schema, Postgres schema, Cedar entity type | Any schema introduction or change | verifier |
| A7 | Algorithm | New algorithm, heuristic, or mathematical derivation | Any non-trivial computation introduced | scientist / executor |

### §1.3 Change Classes and Required Facets

The rigor matrix determines which facets are REQUIRED versus OPTIONAL for a
given change class. The change class MUST be declared explicitly in the review
manifest before dispatch.

| Change class | Code | Required facets | Consensus debate |
|---|---|---|---|
| Kernel public API | CC-1 | F1-F11 + F13 + M1 + M2 + A1-A7 (20 facets) | REQUIRED |
| Adapter or infrastructure | CC-2 | F1 + F2 + F3 + F5 + F6 + F7 deep; F4 scan; M2 | RECOMMENDED |
| Application or domain | CC-3 | F1 + F4 + F5 + F6 + F7 deep; F2 + F3 scan | OPTIONAL |
| Pure refactor | CC-4 | F1 + F3 scan | OPTIONAL |
| Doc only | CC-5 | A2 + A1 scan | OPTIONAL |
| Generated or vendored | CC-6 | F7 scan | OPTIONAL |
| Test or fixture | CC-7 | F1 scan | OPTIONAL |

Trigger conditions that independently REQUIRE a facet regardless of change
class:

- **F10 Reversibility:** REQUIRED when change class is CC-1 or CC-2, OR when
  the change introduces microservice boundary changes, OR when it introduces
  or modifies a state machine, OR when it contains deliberate placeholders
  per `feedback_autonomous_implementation_artifacts`.
- **F11 Observability:** REQUIRED when the change adds new HTTP routes, new
  state machines, new background workers, or new observability emission
  contracts.
- **F13 Compliance:** REQUIRED when the change contains `supersedes:` or
  `amends:` frontmatter entries pointing to prior ADRs, OR when it introduces
  a breaking schema or API change.
- **M1 Challenge-assumption:** REQUIRED for CC-1 and for any new ADR,
  standard, or spec file.
- **M2 Meta-review:** REQUIRED whenever consensus debate is invoked.
- **A1-A7:** Each is independently triggered per the trigger conditions in
  §1.2. Every A-facet whose trigger condition is met MUST be dispatched
  regardless of change class.

### §1.4 Version History

| Version | Date | Key changes |
|---|---|---|
| v2.0 | 2026-05-10 | Initial facet doctrine (F1-F9 + M1+M2) |
| v2.1.0 | 2026-05-12 | Added F10 reversibility, F11 observability, F13 migration triggers |
| v2.2.0 | 2026-05-14 | Per-facet subagent isolation mandate; bias-collapse prevention; executor_topology rule |
| v2.3.0 | 2026-05-17 | A1-A7 adherence facets; closed-enum cap relaxed for A-family |
| v2.4.0 | 2026-05-20 | Rate-limit dispatch ordering (§3); per-ADR promotion review (§6); verdict-file schema hardened (§4); worked example added (§9) |

---

## §2 Per-Facet Dispatch

### §2.1 The Isolation Invariant

Each facet MUST be dispatched as a separate subagent or separate teammate
session. A single agent MUST NOT wear multiple facets within a single
change_id. The lane (`oya-check-dependency-seam` sub-check
`consensus-debate-evidence`) MUST refuse promotion when any `reviewer_id`
appears across multiple facet `r1.json` files for the same `change_id`.

This is the structural requirement introduced by the 2026-05-14 user directive
and formalised as the `executor_topology` rule in v2.2.0. It is load-bearing:
without it, a polished multi-facet document produced by one agent merely
simulates the adversarial tension that the separate-session structure enforces.

### §2.2 Reviewer ID Format

```
reviewer_id ::= <tool> "-" <facet_id> "-" <change_id>
```

Example: `claude-critic-F1_correctness-keystone-bundle-2026-05-20`

The `reviewer_id` MUST be unique per (facet, change_id) tuple. Two subagent
sessions MUST NOT share a reviewer_id even if run by the same underlying model.

### §2.3 Subagent Isolation Requirements

Each subagent MUST:

1. Receive the prompt with **exactly one facet** declared as its lens.
2. Read the corpus independently — it MUST NOT be seeded with another
   facet's conclusions before writing its `r1.json`.
3. Emit its `r1.json` before reading any sibling facet's `r1.json` (round-1
   independence).
4. In round 2, read all sibling `r1.json` files and emit its `r2.json`
   reflecting agreements, disagreements, and shifted positions.

A pre-debate adversarial evidence package (such as an audit report or a
deep-dive document produced before dispatch) MAY be supplied to all subagents
as INPUT. It is not the review itself. Each subagent processes that evidence
independently through its own lens.

### §2.4 Subagent Type Mapping

RECOMMENDED subagent types per facet are listed in §1.2. The mapping is a
recommendation, not a constraint. An orchestrator MAY use a different
subagent type if the capability set is equivalent. The TYPE affects what
tools are available, not the facet lens.

---

## §3 Rate-Limit Dispatch Ordering

### §3.1 The 2026-05-20 Lesson

During the keystone-bundle-2026-05-20 review, 21 facet agents were dispatched
simultaneously. This saturated rate limits and caused 7 facets
(F3/F5/A2/A4/A6/F13/A1) to hit context window or quota errors requiring
re-dispatch. The re-dispatch added latency and created an ambiguous partial
state in the evidence directory where some r1.json files were present and some
were absent, making it difficult to determine whether missing files represented
completed facets or failed dispatches.

This section codifies the ordering rule that prevents recurrence.

### §3.2 Wave Dispatch Protocol

Facet dispatch MUST proceed in waves of **at most 8 facets at a time**. A
new wave MUST NOT be started until at least half of the previous wave's
r1.json files have been written to `evidence/debate/`. The synthesis MUST
NOT begin until all waves have completed and all r1.json files are present.

**REQUIRED wave structure for a CC-1 change (20 facets):**

```
Wave 1 (≤8): F1, F2, F3, F4, F5, F6, F7, F8
  — Wait until ≥4 of 8 r1.json files are written —
Wave 2 (≤8): F9, F10, F11, F13, M1, M2, A1, A2
  — Wait until ≥4 of 8 r1.json files are written —
Wave 3 (≤5): A3, A4, A5, A6, A7
  — Wait until all 5 r1.json files are written —
Proceed to Round 2 dispatch (all 20 facets read siblings)
Proceed to Synthesis
```

The orchestrator MUST track in-flight count. If an in-flight count drops below
the wave minimum before all facets have emitted (indicating a partial failure),
the orchestrator MUST re-dispatch the missing facets before beginning the next
wave.

### §3.3 In-Flight Tracking

The orchestrator SHOULD maintain a dispatch manifest at
`evidence/debate/<change_id>-dispatch-manifest.json` with the following
structure:

```json
{
  "change_id": "<change_id>",
  "total_facets": 20,
  "waves": [
    {
      "wave_number": 1,
      "facets": ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8"],
      "dispatched_at": "<iso8601>",
      "completed_facets": [],
      "failed_facets": []
    }
  ]
}
```

The manifest is updated as r1.json files land. It is NOT itself a verdict
file — it is operational bookkeeping and SHOULD be deleted after synthesis
completes.

### §3.4 Rationale for ≤8 Wave Size

The wave-size limit of 8 is derived from observed rate-limit behaviour during
the 2026-05-20 review. It is NOT a hard technical ceiling — a given session
MAY dispatch fewer if context or quota warrants. The limit is a MAXIMUM, not
a target. When in doubt, dispatch 4-6 per wave.

The ordering of facets within waves is RECOMMENDED as shown above but is not
normative. The orchestrator MAY reorder within waves to parallelise heavy
facets (e.g., F5 Security with its adversarial threat-model prompt is
expensive; placing it in wave 1 ensures it starts earliest and reduces tail
latency).

---

## §4 Verdict File Convention

### §4.1 File Naming and Location

Every facet verdict file MUST be written to:

```
evidence/debate/<bundle-or-change-id>-<date>-<facet-id>-r<round>.json
```

Examples:
- `evidence/debate/keystone-bundle-2026-05-20-F1-correctness-r1.json`
- `evidence/debate/keystone-bundle-2026-05-20-A3-structure-r1.json`
- `evidence/debate/keystone-bundle-2026-05-20-M2-meta-review-r1.json`
- `evidence/debate/keystone-bundle-2026-05-20-F5-security-r2.json`

The `<date>` segment MUST be the review date in `YYYY-MM-DD` format. The
`<facet-id>` segment MUST use the canonical facet identifier from §1.2
(e.g., `F1-correctness`, `A3-structure`, `M1-challenge-assumption`). The
`r<round>` segment is `r1` for round-1 independent, `r2` for round-2
rebuttal.

### §4.2 Required Top-Level Keys

Every `r1.json` verdict file MUST contain the following top-level keys:

| Key | Type | Description |
|---|---|---|
| `schema_version` | string | `multispectrum-review.consensus_debate_protocol.round_1_independent.v2.4.0` |
| `change_id` | string | The canonical change identifier |
| `facet_id` | string | The facet identifier from §1.2 |
| `facet_family` | string | One of `critique`, `meta`, `adherence` |
| `facet_lens` | string | One-sentence description of the lens for this facet |
| `round` | integer | 1 for r1, 2 for r2 |
| `reviewer_id` | string | Unique per facet+change (see §2.2) |
| `review_date` | string | ISO 8601 date |
| `corpus_scope` | array[string] | List of docs/ADRs/specs consulted |
| `verdict` | string | The verdict value (see §4.3) |
| `findings` | array[Finding] | Per-finding objects (see §4.4) |
| `recommendations` | array[Recommendation] | Prioritised recommendations |

Round-2 (`r2.json`) files MUST additionally contain:

| Key | Type | Description |
|---|---|---|
| `agreements_with_sibling_r1` | array[string] | Sibling facet findings this reviewer agrees with |
| `disagreements_with_sibling_r1` | array[string] | Sibling facet findings this reviewer disputes, with rationale |
| `shifted_position` | string or null | Description of any position change since r1, or null |
| `new_evidence` | array[string] | Any new evidence discovered from sibling r1 files |

### §4.3 Verdict Values

Valid values for the `verdict` field:

| Verdict | Meaning |
|---|---|
| `APPROVE` | No material findings; change is ready to promote |
| `APPROVE_WITH_FINDINGS` | Minor findings; change may merge; promotion gated on listed fixes |
| `APPROVE_WITH_CONDITIONS` | Conditions must be met before merge or promotion (conditions listed in findings) |
| `CONDITIONAL_APPROVE` | Same as APPROVE_WITH_CONDITIONS; alias accepted |
| `CONDITIONAL_PASS` | Same as APPROVE_WITH_CONDITIONS; alias accepted |
| `WARN_GO_WITH_FIXES` | Concerns present; recommend merging with tracking issues filed |
| `REVISE` | Changes required before merge |
| `REQUEST_CHANGES` | Same as REVISE; alias accepted |
| `NO_GO` | This facet recommends rejecting the change entirely |
| `NO_GO_AS_BUNDLE` | This facet recommends rejecting the change in its current bundled form (accepts wave-split or deferred path) |
| `CONDITIONAL_GO_AFTER_PROCESS_REMEDIATION` | Process gaps must be closed; substantive content approved |

The synthesizer MUST map all verdict variants to a binary `merge_ok` /
`promotion_blocked` pair when producing the synthesis document.

### §4.4 Finding Schema

Each object in the `findings` array MUST contain:

| Field | Type | Description |
|---|---|---|
| `id` | string | Facet-scoped finding identifier, e.g., `F5-243-01` or `M2-3` |
| `title` | string | One-line summary |
| `severity` | string | One of: `BLOCKER`, `CRITICAL`, `HIGH`, `MEDIUM`, `LOW`, `INFO`, `POSITIVE` |
| `description` | string | Detailed description of the finding |
| `evidence_pointer` | string | Path or citation to the specific doc, section, or code that evidences the finding |
| `recommendation` | string | Concrete fix or action |

The `POSITIVE` severity is reserved for findings that document strengths to
preserve. They MUST NOT be omitted from the report — reviewers who only report
problems produce adversarial noise; reviewers who also document what is working
produce usable signal.

The `BLOCKER` severity indicates the finding MUST be resolved before merge.
The `CRITICAL` severity indicates the finding MUST be resolved before
promotion of the relevant ADR from `Proposed` to `Accepted`. Both are
distinct from `HIGH`, which SHOULD be resolved but does not block promotion
on its own.

### §4.5 Evidence Pointer Format

The `evidence_pointer` field SHOULD use the format:

```
<relative-path>[#<section>][:<line>]
```

Examples:
- `docs/decisions/ADR-0243.md#§D-10`
- `evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json#F5-243-01`
- `specs/tenant-model.json#properties.provider_credential_mode`

When a finding extends or cross-references a finding in another facet's r1
file, the `evidence_pointer` SHOULD cite that file directly. This enables
synthesizer agents to trace finding provenance without re-reading the entire
corpus.

---

## §5 Synthesizer Step

### §5.1 Purpose

After all facet r1.json and r2.json files are written, a synthesizer
produces the single authoritative GO/NO-GO recommendation for the change.
The synthesizer is REQUIRED for CC-1 changes. It is RECOMMENDED for CC-2
changes that surface CRITICAL or BLOCKER findings across two or more facets.

### §5.2 Synthesizer Inputs

The synthesizer MUST read:

1. All r1.json files for the change_id.
2. All r2.json files for the change_id.
3. Any pre-debate evidence packages (audit report, deep-dive) supplied as
   input to the facet subagents.
4. The relevant ADRs, specs, and memories cited across the facet verdicts.

The synthesizer MUST NOT be the same agent as any facet subagent. It operates
as an integrator, not as a 22nd facet. It MUST NOT introduce new findings —
its role is to weigh, adjudicate, and produce a recommended action.

### §5.3 Synthesizer Output

The synthesizer MUST produce two artifacts:

**a) A synthesis JSON file** at:
```
evidence/debate/<change_id>-synthesis.json
```

Required keys per `feedback_multispectrum_review_v22`:

| Key | Description |
|---|---|
| `consensus_points` | Findings where ≥75% of relevant facets agree |
| `consensus_shifts_from_round_1_to_round_2` | Position changes documented in r2 files |
| `unresolved_tensions` | Findings where facets disagree and the tension cannot be mechanically resolved |
| `final_recommendation` | GO / NO-GO / GO-WITH-GATES with gate list |
| `termination_reason` | One of: `consensus_reached`, `escalation_to_human`, `rounds >= max_rounds` |

**b) A synthesis narrative document** at:
```
docs/architecture/<change_id>-synthesis.md
```

The narrative document MUST include:

- The bottom-line verdict (GO / NO-GO / GO-WITH-GATES).
- A facet verdict table mapping each of the 20+ facets to its r1 verdict
  and its weight in the synthesis.
- Adjudication of any NO-GO or NO-GO-AS-BUNDLE verdicts — what was the
  disposition and why.
- A promotion gate set with one gate per CRITICAL or BLOCKER finding, each
  gate pointing to the relevant facet finding by ID.
- A merge sequence with explicit T+N timelines per gate.

### §5.4 GO/NO-GO Determination Rules

The synthesizer MUST apply the following rules:

1. **BLOCKER finding from any facet:** The change MUST NOT merge until the
   BLOCKER is resolved. This is absolute.
2. **NO-GO-AS-BUNDLE from M1:** This is evidence in the synthesis, not an
   automatic veto. The synthesizer MAY override a NO-GO-AS-BUNDLE verdict if
   the F-family + A-family majority vote is APPROVE-WITH-CONDITIONS AND the
   M1 concerns are addressable as promotion gates rather than as merge
   blockers. See §9 for the worked example.
3. **CRITICAL findings from F5 Security:** These MUST map to named promotion
   gates. They do not block merge but they do block the relevant ADR's
   promotion to `Accepted`.
4. **Facet majority:** When the synthesis determines GO-WITH-GATES, the
   merge proceeds and the gates are tracked to closure before each ADR
   promotes.

### §5.5 Audit Chain Emission

After the synthesis document is written, the synthesizer MUST emit an
`consensus_debate_complete` event to the audit chain at
`evidence/audit-chain.jsonl` with:

- `change_id`
- `termination_reason`
- `reviewer_id_list` (all facet reviewer IDs)
- `facet_count`
- `final_recommendation`

### §5.6 FixupTask Filing

Non-blocking findings from the synthesis MUST be transcribed to
`registries/cross-cutting/fixuptasks.jsonl` as CONV-N or TEN-N entries
with an owner and an IP placement. This MUST be done after synthesis lands,
not before. Bulk-resolving FixupTasks without individual assessment is
FORBIDDEN per `feedback_codex_bulk_resolve_antipattern`.

---

## §6 Per-ADR Promotion Review

### §6.1 Purpose

When an ADR amendment lands after the initial bundle review, running the
full 20-facet review again would be disproportionate. The per-ADR promotion
review is a scoped re-review that covers only the facets affected by the
amendment.

### §6.2 Trigger

A per-ADR promotion review is REQUIRED when:

- An ADR amendment lands that addresses one or more promotion gates from the
  bundle synthesis.
- The amendment changes a decision already reviewed by one or more facets.
- The ADR is about to promote from `Proposed` to `Accepted`.

A per-ADR promotion review is NOT REQUIRED for:

- Purely additive documentation fixes (wording clarity, cross-reference
  corrections) that do not change a decision.
- Renaming or typo fixes.

### §6.3 Required Facets for Per-ADR Review

The default facet set for a per-ADR promotion review is:

```
F4 Architecture + F5 Security + F7 Supply-chain + A1 Naming + A2 Documentation
+ A3 Structure + A4 Architecture-adherence + A5 Dependency + A6 Schema + A7 Algorithm
```

plus any facet that originally raised a CRITICAL or BLOCKER finding against
the ADR being reviewed.

The orchestrator MAY add facets if the amendment touches additional concerns.
The orchestrator SHOULD NOT run the full 20-facet suite unless the amendment
is equivalent in scope to a new CC-1 change.

### §6.4 Evidence File Naming

Per-ADR promotion review verdict files MUST use the format:

```
evidence/debate/<adr-id>-<date>-<facet-id>-promotion-r1.json
```

Example:
```
evidence/debate/ADR-0243-2026-05-28-F5-security-promotion-r1.json
```

### §6.5 Promotion Gate Closure

An ADR MAY promote from `Proposed` to `Accepted` when:

1. All BLOCKER and CRITICAL findings from the original bundle synthesis
   that targeted that ADR have been addressed by an amendment.
2. The per-ADR promotion review for those amendments has completed with no
   new BLOCKER findings.
3. The CI lane for that ADR reports green.

Promotion MUST be recorded in the original synthesis document by appending a
`## Gate Closures` section with the date, the addressing amendment ADR or
commit, and the promotion review file path.

---

## §7 Lane Sunset

### §7.1 Current Status

The `oya-check-dependency-seam` sub-check `consensus-debate-evidence` is
currently **NotYetArmed**. As of 2026-05-20 it is advisory: the sub-check
reports violations but does not block merge.

The sub-check **MUST activate as BLOCKER on 2026-07-15**. This date is fixed.
It MAY be extended only by a follow-up ADR that amends this standard and
provides evidence that the corpus upgrade pass is not yet complete.

### §7.2 What the Lane Checks

When armed, the lane MUST refuse promotion of any changeset where:

1. The change_class is CC-1 and fewer than 18 of the 20 required facet
   r1.json files are present in `evidence/debate/` for the change_id.
2. Any reviewer_id appears across multiple facet r1.json files for the
   same change_id.
3. The synthesis document is absent.
4. Any BLOCKER finding in any facet verdict file is unresolved (i.e., has
   no corresponding gate-closure entry in the synthesis document).

### §7.3 Pre-Arm Window (Now to 2026-07-15)

The pre-arm window is used to:

1. Complete the corpus upgrade pass: all existing changesets that are
   missing per-facet r1.json files SHOULD file FixupTasks.
2. Validate the lane logic against the keystone-bundle-2026-05-20 evidence
   set (which MUST pass the lane's deterministic checks when it arms).
3. Train orchestrators on the rate-limit dispatch ordering (§3).

Changes that ship before 2026-07-15 MAY use the advisory-phase process but
SHOULD apply the v2.4.0 cadence as if the lane were armed.

---

## §8 Anti-Patterns

The following anti-patterns are FORBIDDEN under v2.4.0 doctrine.

### §8.1 Single-Agent-Across-Facets

**Pattern:** One agent or one document covers multiple facets. Example: a
deep-dive document with frontmatter `review_facets_targeted: F1..F9 +
M1+M2 + A1..A7` written by one agent.

**Why forbidden:** Bias-collapse. The single agent unconsciously harmonises
its critique across facets, suppressing findings that would surface if each
lens were applied independently. The document may be high quality as input
evidence, but it is NOT a substitute for per-facet subagent dispatch.

**Correct pattern:** The deep-dive document is classified as INPUT EVIDENCE
and supplied to each facet subagent. Each subagent reads it and processes
it independently through its own lens.

### §8.2 Skipping the Synthesizer Step

**Pattern:** Per-facet r1.json files are present but no synthesis document
is produced. The change merges on the basis of "most facets approved."

**Why forbidden:** The synthesizer step performs three functions that
individual facet files cannot: adjudication of conflicts between facets,
promotion gate assignment, and audit-chain completion. Without it, BLOCKER
findings may be overlooked, NO-GO facets may be silently ignored, and the
audit chain remains incomplete.

**Correct pattern:** All facet r1.json and r2.json files MUST be present
before the synthesizer runs. The synthesizer produces both the JSON synthesis
file and the narrative synthesis document.

### §8.3 Treating M1 NO-GO as Automatic Veto

**Pattern:** The M1 Challenge-assumption facet returns NO-GO-AS-BUNDLE and
the change is immediately rejected without running the synthesis step.

**Why forbidden:** M1 is evidence in the synthesis, not a veto. The M1 lens
is designed to attack load-bearing premises with adversarial rigour. A
NO-GO-AS-BUNDLE from M1 means the premises need explicit defence, not that
the change is wrong. The F-family + A-family majority is the primary vote;
M1's NO-GO informs the promotion gate set and the conditions under which the
change may land.

**Correct pattern:** The synthesizer weighs M1's NO-GO-AS-BUNDLE against the
F-family and A-family majority. If the majority is APPROVE-WITH-CONDITIONS
and M1's specific concerns are addressable as promotion gates, the synthesis
issues GO-WITH-GATES and folds M1's concerns into the gate set. If M1
surfaces a fundamental premise failure that the F-family did not address, the
synthesis escalates to human with both positions documented.

### §8.4 Batch-Resolving Findings

**Pattern:** A reviewer or agent marks multiple P1/P2 findings as
"resolved" in a single sweep without individual assessment. Example: "all
P2 findings reviewed and closed."

**Why forbidden:** P2 findings are not ignorable. The 2026-05-17 PR #96 case
documented that all P2 findings swept in a batch were real validator defects.
The sweeper pattern is REPORT-ONLY, not RESOLVE. Each finding MUST be
individually assessed before closure. See `feedback_codex_bulk_resolve_antipattern`.

**Correct pattern:** Each finding is assessed individually. If a finding is
not actionable, the closure MUST document WHY with an evidence pointer (not a
blanket "won't fix" without rationale).

### §8.5 Parallel Full-Platform Dispatch

**Pattern:** All 20+ facets dispatched simultaneously without wave ordering.

**Why forbidden:** Rate-limit saturation. The 2026-05-20 review demonstrated
that parallel dispatch of 21 facets caused 7 re-dispatches and ambiguous
partial evidence state. See §3.

**Correct pattern:** Wave dispatch per §3.2. Maximum 8 facets per wave.

### §8.6 Using the Pre-Debate Package as the Review

**Pattern:** An audit report or pre-debate deep-dive exists; the orchestrator
treats it as satisfying the per-facet evidence requirement and skips dispatch.

**Why forbidden:** The pre-debate package is input evidence. It was produced
by one or two agents before the per-facet dispatch and therefore violates the
executor_topology invariant (one reviewer per facet, no shared reviewer_id
across facets). The fact that the package is high quality does not fix the
structural violation.

**Correct pattern:** The pre-debate package is supplied to each facet subagent
as input. Each subagent may agree with, extend, or reject its findings — the
independence is what matters.

---

## §9 Worked Example — Keystone Bundle 2026-05-20

This section documents the keystone-bundle-2026-05-20 review as the canonical
worked example for v2.4.0. All file paths are relative to the repository root.

### §9.1 Bundle Scope

The keystone bundle comprised 14 foundational ADRs (ADR-0242 through
ADR-0255) plus 7 remediation ADRs (0263, 0272, 0273, 0276, 0280, 0284,
0292), 4 specs, 4 PRDs, 2 user-story compendia, and 4 standards — 28+
documents total. Change class: CC-1 kernel_public_api (explicitly declared in
synthesis; the bundle introduces canonical public spec contracts in
`specs/platform-architecture.json` and 14 keystone ADRs that supersede or
amend prior ADRs).

### §9.2 Pre-Debate Evidence Package

Two input artefacts were produced before facet dispatch:

- `docs/architecture/keystone-bundle-audit-report.md` (1611 lines) — honest
  conditional-GO recommendation with 8 enumerated gaps
- `docs/architecture/keystone-bundle-idea-refine-deep-dive.md` (3588 lines)
  — adversarial red-team analysis with 25+ F-MISSED-* and 10+ F-ANTI-*
  findings

These were classified as INPUT EVIDENCE, not as the review itself. Each was
supplied to every facet subagent.

### §9.3 Facet Dispatch

21 facets were dispatched across 3 waves (the 2026-05-20 review originally
dispatched in parallel, surfacing the rate-limit lesson documented in §3;
the wave protocol is the corrective derived from that lesson).

**Wave 1:** F1, F2, F3, F4, F5, F6, F7, F8
**Wave 2:** F9, F10, F11, F13, M1, M2, A1, A2
**Wave 3:** A3, A4, A5, A6, A7

**Facets that hit rate limits in the original parallel dispatch:**
F3, F5, A2, A4, A6, F13, A1 (7 of 21). Root cause: simultaneous dispatch.
Mitigation: wave protocol per §3.

**F5 Security specific:** F5 was re-dispatched 3 times. Root cause: the
adversarial threat-model prompt (covering 10 ADRs × 3 adversary models =
30 threat-model scenarios) exceeded the available context window on the first
two attempts. Mitigation for future reviews: split large F5 reviews into
sub-scoped F5-a / F5-b segments if the corpus exceeds 10 ADRs.

### §9.4 Verdict Files

All 21 r1.json files are in `evidence/debate/` with path pattern:
`evidence/debate/keystone-bundle-2026-05-20-<facet>-r1.json`

| Facet | Verdict | Merge | Promotion |
|---|---|---|---|
| F1 Correctness | WARN — GO-WITH-FIXES | OK | Gated |
| F2 Hyperscaler fitness | APPROVE-WITH-FINDINGS | OK | OK after minor fixes |
| F3 Readability | WARN — GO-WITH-FIXES | OK | Gated |
| F4 Architecture | APPROVE_WITH_CONDITIONS | OK | Gated (library-first amendments) |
| F5 Security | CONDITIONAL-PASS-WITH-BLOCKING-FINDINGS | OK | BLOCKED (4 CRITICAL) |
| F6 Performance | REVISE (3 budget honesty blockers) | OK | BLOCKED |
| F7 Supply chain | APPROVE_WITH_CONDITIONS (1 P0) | OK | Gated |
| F8 Maintenance | APPROVE_WITH_REVISIONS (B+) | OK | OK |
| F9 Operations | REVISE (9+ runbooks missing) | OK | BLOCKED |
| F10 Frontend/UX | CONDITIONAL_PASS | OK | OK |
| F11 i18n | APPROVE_WITH_RESERVATIONS | OK | OK |
| F13 Compliance | APPROVE-WITH-FINDINGS (0 blockers) | OK | OK |
| M1 Challenge-assumption | NO-GO-AS-BUNDLE | Overridden per §5.4 | Gated (5 findings → promotion gates) |
| M2 Meta-review | CONDITIONAL-GO-AFTER-PROCESS-REMEDIATION | OK | Process gate (this document) |
| A1 Naming | REVISE (4 BLOCKERs) | OK | BLOCKED |
| A2 Documentation | AMBER | OK | OK |
| A3 Structure | REQUEST_CHANGES (3 BLOCKERs) | OK | BLOCKED |
| A4 Architecture adherence | CONDITIONAL_APPROVE | OK | OK |
| A5 Dependency | CONDITIONAL-APPROVE | OK | OK |
| A6 Schema | pass-with-minor | OK | OK |
| A7 Algorithm | APPROVE_WITH_FINDINGS (math errata) | Pre-merge fix | OK after errata |

### §9.5 M1 Adjudication

M1 returned NO-GO-AS-BUNDLE, recommending a 3-wave split (ADR-0242+0244+0245
/ ADR-0247+0248 / ADR-0243+0246+0255). The synthesis overrode the form (one
bundled landing) while honouring the spirit (no operational enforcement until
gates close). Rationale: the 20-of-21 F+A-family majority found the bundle
mergeable in `Proposed` state; M1's gating concerns were converted into
promotion gates §5.1 through §5.15 of the synthesis.

This is the canonical application of §5.4 rule 2 in this standard.

### §9.6 Synthesis Document

`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` — the
authoritative synthesis for this bundle. Contains the bottom-line verdict,
21-facet verdict table, M1 adjudication, BYOK clarification, 15 promotion
gates, merge sequence, and post-merge tracking instructions.

### §9.7 Process Lessons

Documented in `docs/architecture/keystone-bundle-2026-05-20-lessons-learned.md`.
Key lessons folded into this standard: wave dispatch (§3), per-ADR promotion
review (§6), F5 context-window risk.

---

## §10 References

### §10.1 Canonical Specs

- `/specs/multispectrum-review.json` — machine-readable schema, enums, rigor
  matrix, evidence contract (canonical authority for all mechanical checks)
- `/specs/iterative-fix-loop.json` — loop state machine
- `/templates/checklists/pre-pr-multispectrum.json` — evidence template
- `/registry/fixuptasks.jsonl` — FixupTask registry
- `/evidence/audit-chain.jsonl` — audit-chain stream

### §10.2 Related Standards

- `docs/standards/multispectrum-review.md` — thin gateway (remains valid as
  a pointer; this document is the content)
- `docs/standards/documentation-rigor.md` — intern-buildability bar
- `docs/standards/doc-style.md` — voice, tone, RFC-2119, frontmatter
- `docs/standards/code-review.md` — Linus-grade code review bar
- `docs/standards/prevention-doctrine.md` — no-silent-regression controls

### §10.3 Related ADRs

- ADR-0056 — Rust Clean Architecture BNF v4.1 + 12-layer enum
- ADR-0062 — Quality/Performance/Scalability bar
- ADR-0069 — Active-artifact-contract
- ADR-0092 — Workspace dependency-seam policy
- ADR-0105 — 13-layer enum canonical
- ADR-0145 — Inter-microservice communication reform (no-universal-mediator)
- ADR-0242 — oyatie-is-a-tenant doctrine

### §10.4 Related Memories

- `feedback_multispectrum_review_v22` — v2.2.0 doctrine + executor_topology rule
- `feedback_multispectrum_adherence_facets` — A1-A7 adherence facets + trigger conditions
- `feedback_consensus_debate_spectrum_lens_subagents` — single-facet-per-subagent mandate
- `feedback_codex_bulk_resolve_antipattern` — P2 findings are not ignorable
- `feedback_no_silent_regression` — public-contract protection
- `feedback_pipeline_clog_gotchas_2026_05_17` — pipeline clog lessons

### §10.5 Worked Example Files

The 21 verdict files for the keystone-bundle-2026-05-20 review:

```
evidence/debate/keystone-bundle-2026-05-20-F1-correctness-r1.json
evidence/debate/keystone-bundle-2026-05-20-F2-hyperscaler-fitness-r1.json
evidence/debate/keystone-bundle-2026-05-20-F3-readability-r1.json
evidence/debate/keystone-bundle-2026-05-20-F4-architecture-r1.json
evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json
evidence/debate/keystone-bundle-2026-05-20-F6-performance-r1.json
evidence/debate/keystone-bundle-2026-05-20-F7-supply-chain-r1.json
evidence/debate/keystone-bundle-2026-05-20-F8-maintenance-r1.json
evidence/debate/keystone-bundle-2026-05-20-F9-ops-r1.json
evidence/debate/keystone-bundle-2026-05-20-F10-frontend-ux-r1.json
evidence/debate/keystone-bundle-2026-05-20-F11-i18n-r1.json
evidence/debate/keystone-bundle-2026-05-20-F13-compliance-r1.json
evidence/debate/keystone-bundle-2026-05-20-M1-challenge-assumption-r1.json
evidence/debate/keystone-bundle-2026-05-20-M2-meta-review-r1.json
evidence/debate/keystone-bundle-2026-05-20-A1-naming-r1.json
evidence/debate/keystone-bundle-2026-05-20-A2-documentation-r1.json
evidence/debate/keystone-bundle-2026-05-20-A3-structure-r1.json
evidence/debate/keystone-bundle-2026-05-20-A4-architecture-adherence-r1.json
evidence/debate/keystone-bundle-2026-05-20-A5-dependency-r1.json
evidence/debate/keystone-bundle-2026-05-20-A6-schema-r1.json
evidence/debate/keystone-bundle-2026-05-20-A7-algorithm-r1.json
```

Synthesis narrative: `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`

---

## §11 CI Lane and Enforcement

CI lane: `oya-check-dependency-seam` sub-check `consensus-debate-evidence`.

Status as of 2026-05-20: **advisory**.

Becomes **BLOCKER** on **2026-07-15**.

The lane is deterministic: same input (evidence files present/absent,
reviewer_id uniqueness, synthesis file present/absent, BLOCKER finding
resolution status) → same verdict. It does NOT perform judgment — that is
the synthesizer's role. The lane validates the STRUCTURE of the evidence, not
its quality.

Lane extension beyond 2026-07-15 requires an ADR amendment to this standard
with documented evidence that the corpus upgrade pass is in progress and a
revised sunset date.

---

## §12 Change Log

| Date | Version | Author | Change |
|---|---|---|---|
| 2026-05-20 | 2.4.0 | M2 process-remediation subagent | Initial publication. Derived from keystone-bundle-2026-05-20 M2 verdict + synthesis §5.7. Codifies wave dispatch (§3), per-ADR promotion review (§6), verdict file schema (§4), synthesizer step (§5), anti-patterns (§8), and worked example (§9). |
