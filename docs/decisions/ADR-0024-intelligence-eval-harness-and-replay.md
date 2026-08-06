---
id: ADR-0024
status: Proposed
doc_status: published
---

# ADR-0024: Foundry eval harness and replay — per-capability golden sets, A/B routing, adversarial cohorts, regional linguistic eval

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0020 (provider adapter — A/B targets), ADR-0021 (capability registry — eval gate at publish), ADR-0022 (autonomy ceiling — adversarial cohort tests bypass attempts), ADR-0023 (sandbox — replay needs deterministic tool execution), ADR-0026 (in-house substrate — eval is the gate that decides when an in-house variant supplants a provider)

---

## Context

A capability that lacks an eval set is a capability we cannot reason about. Without a golden set, regression detection is impossible; without adversarial cohorts, the autonomy ceiling and the data-class boundaries are theoretical; without per-region linguistic evaluation, a capability that performs well in English can silently degrade in Korean, Japanese, or any other supported locale; without A/B testing of provider routing, we cannot defend a routing decision under cost or quality pressure; without replay against past traces, every change is a leap of faith.

We also need an eval substrate that is the same substrate the in-house model effort (ADR-0026) will use to decide when an Oyatie-internal model variant beats a provider on a per-vertical eval set. Splitting eval per capability vs. per model would fragment authority. The eval harness must therefore be capability-shaped (the unit consumers reason about) and model-aware (the substrate the model effort consumes).

---

## Decision

Every capability publishes with a golden eval set; the eval harness gates publish, runs nightly, runs A/B against routing decisions, and replays against past production traces for regression detection. Adversarial and regional linguistic cohorts are mandatory.

### Eval kernel (`oya-intelligence-eval-kernel`)

```rust
// crates/oya-intelligence-eval-kernel/src/lib.rs
pub struct EvalSet {
    pub capability_id: CapabilityId,
    pub version: EvalSetVersion,
    pub cases: Vec<EvalCase>,
    pub metric: EvalMetric,                  // ExactMatch | F1 | Bleu | Rouge | HumanJudged | Composite
    pub pass_threshold: PassThreshold,       // e.g. {min: 0.85, p95: 0.80}
    pub adversarial: AdversarialCohort,      // prompt-injection, data-class-violation, autonomy-bypass
    pub linguistic: LinguisticCohort,        // per-locale; KR + JP + EN minimum; pack-onboarded locales required
    pub signed: CosignAttestation,           // checked-in + Cosign-signed
}

pub struct EvalCase {
    pub case_id: CaseId,
    pub input: serde_json::Value,            // matches capability.input_schema
    pub expected: ExpectedOutcome,           // exact value, structured assertion, or rubric for HumanJudged
    pub tags: Vec<EvalTag>,                  // e.g. positive, negative, edge, adversarial:prompt-injection
    pub deterministic_seed: Option<u64>,     // for replay against deterministic sandbox path
}

pub struct EvalRun {
    pub eval_set_version: EvalSetVersion,
    pub provider_route: ProviderRoute,
    pub started_at: DateTime,
    pub results_per_case: Vec<EvalCaseResult>,
    pub aggregate: EvalAggregate,            // pass-rate, p95-score, per-cohort breakdown
    pub passed: bool,                        // aggregate vs. pass_threshold
}
```

### Adversarial cohort

Mandatory per capability, with case-floors:

- **Prompt injection** — at minimum 5 cases per Anthropic / Apollo Research adversarial patterns: instruction smuggling, system-prompt extraction, tool-permission escalation, multi-turn jailbreak, encoded-payload injection. Pass criterion: the capability refuses or contains the injection without leaking the system prompt or escalating tools.
- **Data-class violation** — cases that ask the capability to act on data classes outside its declared allowlist; the capability must refuse and the autonomy gate (ADR-0022) must emit a deny.
- **Autonomy-tier bypass** — cases that attempt to invoke a higher-tier action than the capability is gated for; the runtime must refuse.
- **Tool exfiltration** — cases that attempt to use a tool to exfiltrate beyond the egress allowlist (ADR-0023); the sandbox must refuse and emit `EVT-FOUNDRY-SANDBOX-ESCAPE-ATTEMPT`.

### Linguistic cohort (per region)

- **Minimum:** Korean, Japanese, English. Each capability's eval set must include enough cases per locale to reach the per-cohort `pass_threshold`.
- **Pack-onboarded locales become mandatory.** When a regional pack is onboarded (ES, PT, HI, AR per ADR-0026 STT/TTS scope), the locale becomes a mandatory cohort for any capability the pack consumes.
- **Linguistic cases** include morphology (Korean particles, Japanese honorifics), code-switching, locale-specific date/time/currency formatting, and locale-specific safety patterns.

### A/B testing of provider routing

The eval harness runs the same case-set against the candidate route and the incumbent route; the winner is determined by the aggregate metric plus per-cohort dominance. Routing changes (ADR-0020 router preference updates) gate on a per-capability A/B win. The A/B record is checked in alongside the route change.

### Replay against past traces

Production traces (per `EVT-FOUNDRY-CAPABILITY-INVOKED` and per-step) are stored in a replay-eligible form (with deterministic seeds where ADR-0023 allows). On model upgrade or capability change, the harness replays a sampled cohort of past invocations and asserts the new behavior agrees with the old where the old was correct, and improves where the old was incorrect (per labeled regression cases).

### Publish-time eval gate

`oya admin capability publish` refuses unless:
1. An eval set exists for the capability, signed via Cosign.
2. The most recent run on the eval set passes the threshold.
3. The adversarial cohort passes (no injections succeed; no data-class breaches; no autonomy bypass).
4. The linguistic cohort passes for KR + JP + EN minimum (and any pack-required locale).

### Nightly cadence

A nightly job (CI lane `foundry-eval-nightly`) runs every published capability's eval set against its current route; failures open an issue and notify `foundry`. Drift alerts fire when the per-capability pass rate drops below the per-capability `pass_threshold` for two consecutive runs.

### CI lanes

- `foundry-eval-coverage` — refuses publish without an eval set.
- `foundry-eval-adversarial-coverage` — refuses publish without the four adversarial sub-cohorts.
- `foundry-eval-linguistic-coverage` — refuses publish without the minimum locale cohorts.
- `foundry-eval-nightly` — runs the full registry; opens issues on per-capability failures.
- `foundry-eval-route-ab` — gates router-preference changes on a per-capability win.
- `foundry-eval-replay` — runs sampled production-trace replay on model and capability upgrades.

---

## Consequences

### Positive
- Capability publish is gated on evidence, not author assertion; regression detection becomes structural.
- Adversarial cohorts make the autonomy ceiling and data-class boundaries empirically verified, not just theoretically claimed.
- Per-region linguistic eval prevents the "works in English, breaks in Korean" failure mode that fragments user trust.
- A/B testing turns routing decisions into defensible artifacts under cost and quality pressure.
- Replay turns every production trace into a regression-detection asset.
- The same harness becomes the gate for in-house substrate (ADR-0026) — when an in-house variant beats the provider per-cohort, we cut over.

### Negative
- Eval set authoring is real work; capability authors will resist the gate until it is internalized.
- Nightly runs cost provider tokens; per-capability budget for eval has to be planned.
- Replay against past traces requires storing enough trace context that we must navigate the data-retention vs. replay tradeoff.

### Operational
- Runbook: `runbooks/foundry/capability-eval-regression.md` — what to do when nightly fails for a capability.
- Runbook: `runbooks/foundry/capability-eval-regression.md` — cadence for refreshing the golden set; how to retire stale cases.
- On-call: nightly failure on a critical capability pages; drift alert on a routine capability files an issue.
- Per-release: replay cohort must pass before a model upgrade reaches stable.

---

## Alternatives considered

1. **No mandatory eval set.** Pros: faster initial publish. Cons: every capability becomes a black box; regression detection impossible. Rejected — eval is the foundation of capability lifecycle hygiene.
2. **Hosted eval product (e.g. third-party harness).** Pros: less to build. Cons: external system-of-record for our most critical decision-making artifact; license, retention, and data-flow concerns. Rejected per the build-vs-buy posture.
3. **Manual eval only (no automation).** Pros: human judgment first. Cons: cannot scale to nightly cadence; cannot gate publish; cannot run A/B; cannot replay. Rejected — manual is one input, not the substrate.
4. **Per-axis eval substrates (each axis owns its own).** Pros: axis autonomy. Cons: no cross-microservice comparability; no shared adversarial cohort; in-house substrate effort gets fragmented. Rejected — eval is Foundry-owned cross-microservice contract.

---

## Resolved (this revision, 2026-05-09)

1. **Replay retention vs DSR erasure: cryptographic shredding, not record deletion.** Replay traces are stored encrypted with per-subject-keyed envelopes (one DEK per data subject, wrapped by the per-tenant KEK per ADR-0043). When a DSR-cascade purge fires for a subject, the cascade walks the replay store and destroys the per-subject DEK; the encrypted record remains in the store but is unreplayable for the erased subject. The shred event emits to the audit chain per ADR-0003 with `EVT-REPLAY-SUBJECT-SHRED`. Cross-cohort eval continuity is preserved for non-affected subjects because their DEKs survive. Maximum non-shred replay-trace retention horizon: 24 months from emit (per privacy-program retention SLA), after which the entire trace's per-subject DEKs auto-shred and the trace becomes archive-only (replay-disabled).
2. **HumanJudged rubric scaling**: per-capability rubrics use a small Founder-or-domain-expert-seeded gold pool (≥ 50 cases per capability) and a multi-rater consistency gate (Cohen's κ ≥ 0.7). Rubrics are versioned in `decisions/eval-rubrics/<capability>/`; rubric updates re-score the gold pool first to prevent rubric drift confounding capability-quality signal.
3. **Adversarial cohorts are tenant-extensible** for verticals with regulated content (healthcare PHI-extraction patterns, fintech AML evasion patterns) but extensions land in a per-tenant adversarial subspace that does not pollute the cross-tenant cohort. Per-vertical cohort-extension governance is owned by the vertical team + `foundry`.
4. **Per-capability eval token budget** is split: the capability owner cost-center pays for the per-capability gold + adversarial + linguistic cohorts (Foundry rate-cards the eval invocations); Foundry shared budget pays for cross-capability replay infrastructure (the harness, the subject-keyed shred system, the cohort store).

## Open questions

(none material as of 2026-05-09; the four prior open questions are resolved above. Future open questions land here as they surface.)

---

## References

- Internal: ADR-0020 (router; A/B winners feed back into routing), ADR-0021 (registry; publish gate), ADR-0022 (autonomy; adversarial cohort proves the gate works), ADR-0023 (sandbox; replay determinism), ADR-0026 (in-house substrate; eval is the cutover gate).
- External: Anthropic responsible-scaling policy and adversarial prompt patterns; Apollo Research evaluations; standard NLP metrics (BLEU, ROUGE, F1).
- Capability publishing checklist: `docs/checklists/foundry-capability-publishing.md`.
