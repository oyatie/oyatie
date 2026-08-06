---
id: ADR-0544
title: "Friction-ledger closed-loop accounting meta-gate"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0083, ADR-0132, ADR-0363, ADR-0515, ADR-0540]
amends: []
related: [ADR-0017, ADR-0083, ADR-0131, ADR-0132, ADR-0363, ADR-0515, ADR-0538, ADR-0539, ADR-0540, ADR-0541]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Friction-ledger closed-loop accounting

# ADR-0544: Friction-ledger closed-loop accounting meta-gate

## Status

**Proposed - 2026-06-10 (authored for founder sign-off; door: one-way).**

## Context

The friction ledger (`.omc/ultragoal/friction-ledger.jsonl`) is the running record of every pipeline
defect the agent fleet hits. The founder decision of 2026-06-10 makes it a first-class governed
surface: *every friction-ledger row must terminate in a gate, an automation, or an explicit
accepted-risk entry, enforced by a gate so unconverted frictions block merges like code debt.* Until
now the ledger was append-only and ungoverned — a friction could be logged and then silently never
disposed, exactly the failure the founder's automation-maximalism doctrine targets (manual-twice =
write the automation).

This is the Google SRE **postmortem action-item** model: a postmortem is not "done" until each action
item has an OWNER and VERIFIABLE CLOSURE; deferred items are explicitly accepted with a recorded
rationale, not dropped. The friction ledger is our postmortem-AI register. We adopt that proven
production methodology and reimplement it Rust-native (founder doctrine: proven patterns, Rust
reimplementation) as a cloud-ci gate. We diverge from a centralized issue-tracker only because the
ledger is a content-addressed git-tracked artifact and the enforcement point is merge admission, not
a separate service — the closed-loop *property* is identical.

The motivating frictions: FRIC-1781126000 / FRIC-1781127000 (unconverted frictions accumulating as
untracked debt — the founder decision's direct cause) and FRIC-1781112000 (baseline-block-on-new
gates are launderable when the baseline is regenerated in the same PR; a ratchet must freeze its
reference against a non-regenerable point).

The mechanical base measurement on `dev` found 68 physical ledger rows folding to 60 distinct
frictions under an **event-sourced append model**: a PRIMARY row carries the full record
(`id, seen_at, friction, pipeline_defect, enforcement_fix, status`), and later UPDATE rows reuse the
same `id` to carry a `status_update` plus closing `evidence`. Status is FREE-TEXT (50+ distinct
values), not an enum; an `evidence` field already exists organically on the rows that close a
friction (citing PR number, merged `dev` SHA, gate id, or ADR). The closure-integrity property
already holds on the live corpus: every friction whose effective status is terminal carries evidence.

## Decision

Add `ci/facade/action-item-accounting` as a pure cloud-ci meta-gate.

NAME: oya-cloud-ci-friction-accounting-app
JUSTIFICATION:
- microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515.
- bc-tokens = friction-accounting: the bounded concern is closed-loop friction-ledger accounting.
- layer = app: the crate is an executable CI gate surface with a pure evaluator kernel.
- exemptions claimed: none.

**Born pack-shaped (founder R0).** The ledger path, the free-text status taxonomy
(`status -> {open | terminal | accepted-risk}`), the required-field set, and the evidence-on-closure
rules are DATA in `friction-accounting-policy.json`. The Rust kernel hardcodes no repo path nor any
oyatie string; another repo adopts the gate by repointing the policy at its own ledger. The kernel
DOES fix the ledger ROW SCHEMA — the field names (`id`, `seen_at`, `status`, `status_update`,
`friction`, `enforcement_fix`, `evidence`) and the primary-vs-update shape are the engine's contract,
not per-repo pack values; an adopting repo maps its ledger columns onto these names. The engine is
neutral on policy; the row schema is the contract.

**Kernel contract.** `collect_observed_frictions(root, policy) -> {rows:[..]}` performs the only I/O
(read-only ledger read, no temp files). `evaluate_keyed(policy, observed) -> BTreeSet<Finding>` is
pure and unit-testable without a filesystem: it folds the event-sourced append rows onto each
friction `id` (effective status = latest `status_update` else primary `status`; evidence/disposition
= present on any row for that id) and applies the closed-loop invariants. `evaluate` is the bare-code
projection of `evaluate_keyed`.

The evaluator emits stable `Finding{code,key,detail}` rows (key = the friction `id`; `<policy>` for
the gate-id sentinel):

- `friction_policy_gate_id_mismatch` (frozen-empty): the policy `gate_id` is not the gate's id.
- `friction_missing_required_field` (baseline-block-on-new): a PRIMARY row omits/blanks a required
  field.
- `friction_unknown_status` (frozen-empty): a friction's effective status maps to no taxonomy class.
- `friction_no_disposition` (baseline-block-on-new): a friction declares no non-blank
  `enforcement_fix` and is not accepted-risk.
- `friction_closed_without_evidence` (baseline-block-on-new): a terminal-class friction cites no
  evidence.
- `friction_accepted_risk_without_evidence` (baseline-block-on-new): an accepted-risk
  (escalated/founder-held) friction cites no evidence for the holder/decision.
- `friction_duplicate_primary_row` (frozen-empty): two PRIMARY rows share one `id` (appends are
  legitimate event-sourcing and never count as duplicates).
- `friction_orphan_update_row` (baseline-block-on-new): a friction id has ONLY update-shaped rows and
  no anchoring PRIMARY record. Without a primary the schema/disposition checks cannot bind, so an
  update-only row would otherwise fold to a clean terminal-with-evidence state and evade every check;
  the missing primary is itself the (sole) violation for that id. Three pre-existing orphan ids are
  baselined as shrinkable legacy debt; `friction_no_disposition` is consequently born-blocking-clean.

**Ratchet semantics that never discourage logging.** Appending a friction row never fails the gate by
itself. Schema/orphan/closure codes baseline today's legacy debt in a reviewed, NON-regenerated
`friction-accounting-baseline.json` (set-equality, shrink-only) plus independent reviewed ceilings —
a NEW friction triggering any of these adds a key not in the baseline and fails closed, while frozen
legacy debt ages out as rows are fixed. The closure-integrity codes ship born-blocking frozen-empty
(the live ledger satisfies them today).

The anti-laundering property is **review-visible, not yet structural** (FRIC-1781112000). The good
parts are real: the live-repo test asserts set-equality on `BTreeSet`s (not counts), the ceilings are
hand-fixed constants in test source (not derived from any generated artifact), and the baseline is
not producer-materialized. But the baseline, ceilings, and test are all same-PR-editable; nothing yet
mechanically compares the baseline against `origin/dev`. So laundering new debt requires a
review-visible edit to a frozen file that a reviewer must approve — it is not structurally impossible.
A baseline-shrink-only meta-check that diffs against the merge-base is the named follow-up
(FRIC-1781112000's full fix); until it lands this gate matches the ADR-0540 target-parity posture.

**Escalated / founder-held rows (D2).** Statuses such as `interim-accepted`,
`awaiting-founder-pairing`, and `escalated-to-leader-*` map to the **accepted-risk** class: they are
neither agent-closeable nor open debt; their disposition IS the recorded acceptance, proven by an
evidence citation naming the holder/decision. This mirrors the SRE "explicitly deferred/accepted"
action-item state.

**Integration model.** Unlike producer-face gates, this gate does NOT route through the central
`gate-baseline.generated.json` firewall: the producer's `RawCorpusCollector` dispatch is hardwired to
the single brand-residue collector, so registering there would mis-wire the producer. Instead — like
`cloud-ci-rust-first-automation-hygiene` — the gate is a standalone born-blocking buck2 self-test that
owns its own committed baseline + ceilings and runs as its `oya-cloud-ci-friction-accounting-app-gate`
`rust_test` under the binding `buck2 test //cloud/cloud-ci/...` CI job (plus a labeled per-crate
matrix check in `oya-ci-required`). Same firewall semantics, local enforcement. Extending the central
producer with a real friction-accounting collector is deferred (it would touch the shared producer and
regenerate all faces for no enforcement gain today).

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `ci/facade/action-item-accounting/` | create gate crate + policy + baseline | `oya-cloud-ci-friction-accounting-app` | app |
| `.github/workflows/oya-ci-required.yml` | add one gate matrix line | - | - |
| `docs/oya-ci/gate-catalog.md` | document gate, input kind, key shape, frozen-empty codes | - | - |

The gate-crate files owned by this ADR are:
`ci/facade/action-item-accounting/BUCK`,
`ci/facade/action-item-accounting/Cargo.toml`,
`ci/facade/action-item-accounting/friction-accounting-policy.json`,
`ci/facade/action-item-accounting/friction-accounting-baseline.json`,
`ci/facade/action-item-accounting/src/lib.rs`,
`ci/facade/action-item-accounting/tests/friction_accounting.rs`.

### Integration via Workflow + Ontology

Not applicable. This ADR changes repository admission checks only; it does not emit Workflow events,
consume Workflow events, or write Ontology objects.

### Positive

- A logged friction can no longer silently lack a disposition: every row must declare an
  `enforcement_fix` (the gate/automation it converts to) or be an explicitly evidenced accepted-risk.
- Closing a friction now requires verifiable closure (evidence: gate id / ADR / merged SHA / tool
  path), matching SRE action-item closure discipline.
- The free-text status taxonomy is DATA, so the ledger's organic vocabulary is honored without forcing
  a schema migration; new repos adopt the gate by editing one JSON file.
- Frozen-empty closure-integrity codes are born-blocking; the legacy schema/disposition debt is
  mechanically frozen and shrink-only.

### Negative

- The status taxonomy must be maintained as the ledger's vocabulary evolves; a genuinely new status
  fails closed (`friction_unknown_status`) until classified — a deliberate forcing function, but it
  requires a one-line policy edit to admit a new disposition vocabulary. Because the taxonomy edit is
  neither ratcheted nor reviewed by a meta-check, the policy file is the unguarded pressure valve: the
  fastest unblock for a novel status is a same-PR taxonomy edit, which can quietly reclassify around
  the forcing function. A taxonomy-change review discipline (or meta-gate) is a follow-up.
- `status_match=prefix` trades the fail-closed unknown-status property for tolerance of the ledger's
  verbose vocabulary: a new status sharing a registered prefix classifies silently. Prefixes are kept
  deliberately narrow; this is an explicit, documented trade, not an oversight.
- Existing schema/closure debt (4 missing-field, 3 orphan-update, 2 closed-without-evidence, 7
  accepted-risk-without-evidence keys) remains visible in the baseline until each friction is fixed.
- The gate enforces its own baseline locally rather than through the central firewall until the
  producer's raw-corpus dispatch is generalized.
- **SRE precedent half-applied:** this gate enforces the CLOSURE half of the postmortem action-item
  model (verifiable evidence) but not the OWNER half (no per-row owner field is required or checked)
  nor TIME-BOUND termination (an open friction with a declared `enforcement_fix` is green forever; the
  founder language "every row must *terminate*" is enforced as "must *declare a disposition*"). Adding
  an owner field and an aging dimension (the in-tree GATE-3 staleness precedent) is deferred.
- **Undeclared buck2 input:** the live-repo test walks up to the real repo root to read the ledger
  (the established convention across the gate-test family), so the ledger is not a declared buck2
  input and a warm-cache `buck2 test //cloud/cloud-ci/...` can serve a stale verdict after a ledger
  edit. Merge authority is unaffected — the `oya-ci-required` matrix leg runs
  `cargo test --locked -p oya-cloud-ci-friction-accounting-app` on a fresh runner that re-reads the
  ledger every run — but this gate is the worst case for the convention (its input changes on nearly
  every lane), so a repo-wide friction for declared-input gate tests is warranted (and this gate would
  then account it).

### Operational

- Buck2 remains the binding local verification surface for the gate.
- The baseline + ceilings are reviewed, hand-shrunk artifacts; they are never regenerated to absorb
  new debt (FRIC-1781112000).

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` | Affected | App crate depends inward on serde_json only; no producer/kernel coupling. |
| `cross-product-refusal` | Not affected | No product boundary is introduced. |
| `port-location` | Not affected | No new port traits. |
| `layer-correctness` | Affected | New gate declares the `app` layer in its BNF name. |
| `composition-root-only` | Not affected | No long-running composition root is introduced. |
| `sdk-kernel-only` | Not affected | No SDK kernel boundary change. |

## Alternatives Considered

**Alternative 1 - Leave the friction ledger ungoverned (append-only, no gate)**
- Description: continue logging frictions with no enforced disposition or closure.
- Pros: zero gate maintenance; logging stays frictionless.
- Cons: preserves the exact untracked-debt class the founder decision targets; frictions vanish
  without conversion to a gate/automation/accepted-risk.
- Reason rejected: the founder decision makes closed-loop accounting mandatory.

**Alternative 2 - Migrate the ledger to a strict status enum and validate against it**
- Description: replace the free-text status with a fixed enum and reject any other value.
- Pros: simpler classification.
- Cons: a breaking schema migration of 68 heterogeneous rows; discourages the organic, descriptive
  status vocabulary the fleet already uses; couples the engine to oyatie's vocabulary.
- Reason rejected: a DATA taxonomy (status -> class, prefix-matched) honors the existing ledger and
  keeps the engine neutral/pack-shaped.

**Alternative 3 - Route the gate through the central producer firewall baseline**
- Description: register as a producer-face/raw-corpus gate in `oya-ci.toml` + `libs/oya-ci-config`.
- Pros: one canonical firewall path.
- Cons: the producer's `RawCorpusCollector` dispatch is hardwired to the brand-residue collector, so
  registration mis-wires the producer and corrupts the central baseline; fixing it touches the shared
  producer and regenerates every face.
- Reason rejected: a standalone born-blocking self-test (the rust-first-automation-hygiene precedent)
  achieves identical semantics with minimal collision surface; central integration is deferred.

## Verification

- `buck2 build //ci/facade/action-item-accounting/...`
- `buck2 test //ci/facade/action-item-accounting:oya-cloud-ci-friction-accounting-app-unittest`
- `buck2 test //ci/facade/action-item-accounting:oya-cloud-ci-friction-accounting-app-gate`

## References

- Founder decision 2026-06-10: friction ledger closed-loop accounting (every row terminates in a
  gate, an automation, or an explicit accepted-risk entry, enforced by a gate).
- FRIC-1781126000 / FRIC-1781127000: unconverted frictions accumulating as untracked debt.
- FRIC-1781112000: same-PR baseline regeneration launders new debt; freeze the ratchet against a
  non-regenerable reference.
- Google SRE "Postmortem Culture": action items must have owners and verifiable closure.
- ADR-0540: target-parity gate (shrink-only baseline + reviewed ceiling precedent).
- ADR-0515: cloud-ci required status context as merge authority.
