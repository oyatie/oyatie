---
id: ADR-0144
status: Superseded
deciders: council-architecture, council-privacy, ops-security, axis-tasks, axis-meet, axis-medical, axis-hr, axis-payroll
date: 2026-05-18
owner: council-privacy
supersedes: []
superseded_by: [ADR-709]
amended_by: [ADR-0251]
related: [ADR-0064, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_memory: [feedback_canonical_base_localization, feedback_quality_performance_scalability_bar, feedback_repeat_mistake_prevention]
related_specs:
  - /specs/capabilities/eu-ai-act-risk-class-registry.json
  - /specs/capabilities/canonical-tier-schema.json
purpose: |
  Replace the binary "Annex III yes / no" gate with a 5-tier graduated risk
  model (Minimal / Limited / General-Purpose / High-Risk / Unacceptable)
  matching EU AI Act Articles 5-9 + Annex III + GPAI obligations. Mitigates
  Annex-III-binary-gate-is-compliance-theater regret.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0144: EU AI Act graduated risk tier model

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

The current EU AI Act compliance surface in
`specs/capabilities/eu-ai-act-risk-class-registry.json` maps each
capability to a string risk class (`minimal-risk-ai-system`, `limited-
risk-ai-system`, `high-risk-ai-system`, `prohibited`). Cedar policy
fragments then gate per-capability admission on the class string.

This is functionally a binary gate (Annex-III in scope = REFUSED; out
of scope = PERMITTED). It is compliance theater on three axes:

1. **Limited-risk and General-Purpose AI distinct obligations
   collapsed.** The EU AI Act 2024/1689 distinguishes Article 50
   (transparency to users; applies to limited-risk and above),
   Article 52 (transparency for GPAI; applies regardless of
   downstream use), and Article 60 (post-market monitoring; applies
   to high-risk and GPAI ≥ systemic-risk threshold). The binary
   gate cannot represent these layered obligations.

2. **Article 5 (prohibited) treated as just another tier.** Article 5
   names eight enumerated prohibited practices (social scoring,
   real-time biometric ID in public, emotion recognition in workplaces,
   etc.); these are not "high-risk with extra mitigations" but a
   distinct unlawful class. Treating "prohibited" as a tier glosses
   the legal nuance.

3. **Risk class mutation under deployment context.** A given AI
   system may be limited-risk in deployment context A and high-risk
   in deployment context B (Article 6(3) — Annex III applies based
   on use). The binary gate forces the registry to enumerate every
   `auto-employment-decisioning` archetype as `high-risk` without
   accommodating the same capability used in a non-Annex-III context
   as `limited-risk`.

The current registry already acknowledges this gap by listing TWO
archetypes for the same `T2-auto` tier (`auto-deterministic-action`
as `limited-risk`; `auto-employment-decisioning` as `high-risk`).
That split is the seam where the graduated model belongs.

## Decision

The risk classification is upgraded to a **5-tier graduated model**
matching the EU AI Act 2024/1689 structure:

| Tier | Risk class            | EU AI Act anchor       | Deployment status |
|------|-----------------------|------------------------|-------------------|
| 0    | Minimal-risk          | Art. 50 (when AI-generated content is shown) | PERMITTED with Art. 50 disclosure |
| 1    | Limited-risk          | Art. 50 + Art. 52 (for GPAI surface)         | PERMITTED with transparency + explicit accept |
| 2    | General-purpose AI    | Art. 52 + Art. 53 + (Art. 54 if systemic-risk) | PERMITTED with GPAI obligations + downstream notice |
| 3    | High-risk             | Art. 6 + Annex III + Art. 9-15 + Art. 43 (conformity assessment) | REFUSED until conformity-assessment ADR ships per pack |
| 4    | Unacceptable (prohibited) | Art. 5                  | REFUSED unconditionally; emits prohibited-deploy event to audit chain |

### Tier mutation under deployment context

A capability's tier is computed from a (capability_archetype,
deployment_context) tuple:

```
tier = max(
  base_tier_per_archetype,
  context_tier_per_deployment_context
)
```

Example: `auto-task-assign` has `base_tier = limited-risk (Tier 1)`
under personal-context use; under employment-context deployment it
escalates to `high-risk (Tier 3)` per Annex III §4. The Cedar
admission path queries the deployed tier, not the archetype.

### Schema upgrade in `eu-ai-act-risk-class-registry.json`

The new schema adds:

```json
{
  "tier_id": "T2-task-auto-assign",
  "base_tier": "limited-risk",
  "context_tiers": {
    "personal": "limited-risk",
    "professional-non-employment": "limited-risk",
    "employment": "high-risk",
    "biometric-id-public": "unacceptable",
    "social-scoring": "unacceptable"
  },
  "tier_obligations": {
    "minimal-risk": ["art-50-disclosure"],
    "limited-risk": ["art-50-disclosure", "art-14-explicit-accept"],
    "general-purpose-ai": ["art-52-gpai-transparency", "art-53-downstream-notice", "art-54-systemic-risk-eval-if-applicable"],
    "high-risk": [
      "art-6-classification",
      "art-9-risk-management",
      "art-10-data-governance",
      "art-13-transparency",
      "art-14-human-oversight",
      "art-15-accuracy-robustness-cybersecurity",
      "art-43-conformity-assessment",
      "art-60-post-market-monitoring"
    ],
    "unacceptable": ["refuse-with-audit-chain-event"]
  }
}
```

Per-µservice capability yamls reference `tier_id` and supply the
deployment context; the Cedar admission path computes the effective
tier and applies the obligation set.

### CI lane integration

The existing `oya-check-eu-ai-act-annex-iii-refusal` lane
(authored 2026-05-18 alongside this ADR) is generalised to the
**graduated lane** `oya-check-eu-ai-act-graduated-tier`:

1. Read the schema-v2 registry.
2. For each capability, evaluate the (archetype, context) → tier
   computation.
3. Assert the µservice's Cedar policy fragment honours every
   obligation in the resolved tier's obligation set.
4. Refuse to admit a `green` claim if any obligation is unmet for
   any in-scope deployment context.

### Backward compatibility

The four existing archetype strings (`minimal-risk-ai-system`,
`limited-risk-ai-system`, `high-risk-ai-system`, `prohibited`)
remain valid and map deterministically to the new tiers (Minimal /
Limited / High-Risk / Unacceptable). The two new tiers
(`general-purpose-ai`) require explicit migration of any GPAI-using
capability (the foundry `providers` BC and any µservice that
exposes a `T1-assist` flowing through a third-party-trained LLM).

## Alternatives considered

### Alternative 1: Keep binary gate; add documentation

- **Pros:** No schema change; no migration cost; reviewer guidance
  closes the gap on paper.
- **Cons:** Reviewers must memorise the layered Art. 50 / 52 / 60
  obligations; obligations cannot be CI-enforced; the four-archetype
  taxonomy will continue to under-classify GPAI surfaces.
- **Rejected because:** Compliance theater — the obligations are
  Cedar-evaluable; leaving them outside CI means the gate doesn't
  actually enforce the law.

### Alternative 2: 3-tier gradient (low/medium/high)

- **Pros:** Simpler than 5 tiers; matches some industry frameworks
  (NIST AI RMF maps).
- **Cons:** Doesn't represent GPAI (a distinct EU AI Act category
  with Art. 52/53/54 obligations that don't fit the high/medium/low
  axis); doesn't represent prohibited (a categorical no, not the
  "highest risk").
- **Rejected because:** Cannot encode the EU AI Act's actual
  taxonomy without lossy compression. The legal artifact's structure
  IS the 5-tier model; mapping to a 3-tier model loses information.

### Alternative 3: 7-tier model (split high-risk by Annex III section)

- **Pros:** Each Annex III section (§1 biometric, §2 critical-
  infrastructure, §3 education, §4 employment, §5 essential services,
  §6 law enforcement, §7 migration, §8 administration of justice)
  gets its own tier.
- **Cons:** The EU AI Act applies the SAME high-risk obligations to
  all Annex III sections; splitting them adds tiers without changing
  obligations.
- **Rejected because:** Adds complexity that the regulation itself
  does not. Annex III sections are sub-classifications WITHIN tier 3,
  not separate tiers.

## Consequences

### Positive

1. **Obligations become CI-enforceable.** The graduated lane reads
   the schema and asserts each obligation has a matching Cedar
   admission seam. No more reviewer-memory-based compliance.
2. **GPAI explicitly tiered.** The `general-purpose-ai` tier
   captures the Art. 52/53/54 obligations that the binary gate
   collapsed. Material once oyatie ships any provider-pool LLM
   surface with EU customers.
3. **Context-aware tier mutation.** The same `T2-task-auto-assign`
   capability classifies as `limited-risk` in personal-context and
   `high-risk` in employment-context; previously the registry had
   to pick one and force the Cedar fragment to gate.
4. **Prohibited practices distinct.** Art. 5 ("Unacceptable")
   becomes its own tier with an unconditional refuse + audit-chain
   event surface, matching the law's treatment of prohibited
   practices as a categorical no rather than a high-risk extreme.

### Negative

1. **Schema migration cost.** Every existing capability YAML must
   carry a `context_tiers` map. Mitigation: defaults are derived
   deterministically from the existing `risk_class` field; only
   capabilities that genuinely have context-dependent tiers need
   explicit overrides.
2. **CI lane complexity grows.** The graduated lane evaluates
   per-context obligation sets, not a single yes/no claim.
   Mitigation: the lane re-uses the existing `oya-check-eu-ai-act-
   annex-iii-refusal` kernel; complexity is bounded by the schema.
3. **Reviewer cognitive load.** Engineers writing T2-auto.yaml must
   identify deployment contexts and their associated tiers.
   Mitigation: the canonical contexts (personal / professional-non-
   employment / employment / biometric-public / social-scoring)
   are enumerated in the schema; engineers pick from the closed set.

### Comparisons to industry-standard practice

- **EU AI Act 2024/1689:** the 5-tier model IS the regulation's
  taxonomy; this ADR brings oyatie into structural alignment with
  the law rather than imposing an external model.
- **NIST AI RMF (US):** the NIST Risk Management Framework (1.0,
  2023) maps to a graduated tier model with deployment-context
  mutation. Closely compatible with this ADR's 5-tier model.
- **Anthropic Responsible Scaling Policy (RSP):** Anthropic's RSP
  uses an AI Safety Level (ASL) tier model (ASL-1 through ASL-5)
  with deployment-context-dependent obligations. Direct precedent
  for the tier-with-context-mutation pattern.
- **AWS Bedrock Guardrails:** AWS Bedrock's guardrails support a
  4-tier risk-level configuration with deployment-context overrides.
  Direct precedent.
- **Google PaLM 2 / Gemini safety taxonomy:** Google's safety
  taxonomy uses a 5-tier severity model (NEGLIGIBLE / LOW /
  MEDIUM / HIGH / VERY_HIGH) per the Vertex AI safety docs. Direct
  precedent for the 5-tier shape.
- **Palantir AIP:** AIP's policy engine uses graduated tier risk
  classification with context-dependent escalation per the public
  product docs.

## References

- EU AI Act Regulation (EU) 2024/1689 — Art. 5 (prohibited);
  Art. 6 + Annex III (high-risk classification);
  Art. 9-15 (obligations on high-risk providers);
  Art. 43 (conformity assessment);
  Art. 50 (transparency to users);
  Art. 52-54 (GPAI obligations);
  Art. 60 (post-market monitoring).
- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0133 — industry-best-practice + hyperscaler-conformance.
- ADR-0140 — Cedar policy enforcement substrate.
- `specs/capabilities/eu-ai-act-risk-class-registry.json` (to be
  upgraded to schema v2 in a follow-on IP).
- `crates/oya-check-eu-ai-act-annex-iii-refusal` (to be generalised
  into `oya-check-eu-ai-act-graduated-tier` per this ADR).
- NIST AI Risk Management Framework 1.0 (NIST AI 100-1, 2023).
- Anthropic Responsible Scaling Policy v1.1 (anthropic.com/rsp).
- AWS Bedrock Guardrails documentation
  (docs.aws.amazon.com/bedrock/latest/userguide/guardrails.html).
- Google Vertex AI safety taxonomy (cloud.google.com/vertex-ai/docs/
  generative-ai/learn/responsible-ai).
- Palantir AIP policy-engine documentation.
