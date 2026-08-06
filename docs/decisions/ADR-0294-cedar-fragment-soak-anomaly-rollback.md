---
id: ADR-0294
status: Accepted
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - axis-policy-engine
  - axis-audit-chain
  - axis-cell
  - ops-sre-reliability
  - ops-compliance
supersedes: []
amends: []
requires_amendment_to:
  - ADR-0243-cedar-as-universal-gate.md (§D-2 fragment lifecycle gains a `soaking` stage between `Published` and `Activated`; §D-10 hot-reload <5s window now applies to soak-phase shadow evaluation, not enforcement; §D-11 fallback semantics gain anomaly-rollback as a third causal path)
  - ADR-0246-policy-engine-substrate-promotion.md (fragment-publisher admission gate enforces `sunset_at - activate_at >= 60s` invariant)
superseded_by: []
related:
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0140-cedar-policy-enforcement.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0293-governance-meta-trust-root.md
related_adrs:
  - ADR-0297
  - ADR-0311
  - ADR-0313
  - ADR-0319
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/policy-engine.json
  - /specs/cedar-fragment-schema.json
  - /specs/cedar-fragment-soak-protocol.json
  - /specs/policy-gate-coverage.json
related_memory:
  - feedback_cedar_as_universal_gate
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_automate_everything
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: promotion-gate-fix-2-of-4
authority_for_existence: docs/architecture/keystone-bundle-2026-05-20-synthesis.md §5.2
closes_findings:
  - F5-243-01 (Cedar fragment hot-reload TOCTOU, CRITICAL)
  - M1-KB-F3 (Cedar fragment hot-reload TOCTOU)
  - TMG-05 (Cedar fragment race-condition / cache-poisoning threat absent)
naming_justifications:
  - name: oyatie.policy-engine.fragment-soak-detector
    bnf_v4_1: tenant=`oyatie` · sub_scope=`policy-engine.fragment-soak-detector` (kebab-case, hyphenated, no underscores)
    layer_enum_adr_0105: observability
    rationale: Per-cell automation that monitors permit-rate / denial-rate / latency for fragments in soak phase; emits anomaly verdicts; sits in the observability layer per ADR-0105
  - name: oyatie.policy-engine.fragment-anomaly-revoker
    bnf_v4_1: tenant=`oyatie` · sub_scope=`policy-engine.fragment-anomaly-revoker` · arity=3
    layer_enum_adr_0105: automation
    rationale: Reacts to anomaly verdicts by auto-revoking the offending fragment; distinct principal from the soak-detector to maintain separation-of-duty (detector observes; revoker actuates)
  - name: oya-shared-fragment-soak-protocol
    bnf_v4_1: shared-domain crate per `feedback_glossary_shared_not_platform`; kebab-case
    layer_enum_adr_0105: shared
    rationale: Shared crate exposing the soak + anomaly-detector primitives; consumed by policy-engine + cedar-fragment-registry + observability
  - name: oya-check-fragment-soak-window
    bnf_v4_1: gate-name convention `oya-check-<predicate>` per ADR-0212
    layer_enum_adr_0105: gate
    rationale: CI lane verifying fragment publication carries `sunset_at - activate_at >= 60s` invariant + soak-phase evidence
enforcement_status: advisory-until-soak-detector-deployed
enforced_by:
  - oya gate validate fragment-soak-window
  - oya gate validate fragment-anomaly-detector-coverage
  - oya gate validate fragment-publisher-admission
  - oya gate validate fragment-rollback-evidence-emitted
---

# ADR-0294: Cedar Fragment Soak + Anomaly-Rollback

## Status

Proposed — 2026-05-20.

Promotion-gate fix **2 of 4** for the keystone bundle 2026-05-20
(`docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.2).
This ADR closes F5-Security finding **F5-243-01** (CRITICAL, Cedar
fragment hot-reload TOCTOU + no anomaly rollback) and M1-KB-F3 +
TMG-05. ADR-0243 cannot promote from `Proposed` to `Accepted` until
this ADR's soak + anomaly-rollback mechanics are implemented.

Enforcement is `advisory-until-soak-detector-deployed`. The CI lanes
that enforce this ADR become BLOCKER once:

1. The per-cell soak-detector pod is deployed in every Tier-2 +
   Tier-3 cell and reports green for ≥ 7 consecutive days.
2. The fragment-publisher admission gate (per §D-6 below) refuses
   any fragment whose `sunset_at - activate_at < 60s` and emits the
   refusal to audit-chain.
3. An end-to-end rehearsal in `dev-tools-cell-staging` has executed
   a malicious-fragment injection, the soak detector has flagged
   it, the anomaly-revoker has auto-revoked it, and the rollback
   evidence is verifiable in audit-chain.
4. `oya-check-fragment-soak-window` lane scans every fragment in
   `microservices/policy-engine/fragments/` and reports zero
   missing-soak-evidence findings on baseline-scope fragments.

## Date

2026-05-20.

## Context

### What F5-243-01 actually says

F5-Security's r1 verdict (CRITICAL) reads:

> ADR-0243 §D-6 + §D-10 specify <5s p99 hot-reload across cell
> replicas via Kafka pub-sub. Adversary model: a compromised
> intermediate signing key (council-security or pack-owner) signs a
> malicious fragment that opens a wide permit for 3 seconds and is
> then reverted. Cedar SDK fallback (§D-11) is 'last-known-good
> cached decision, 30s TTL' — but the cache key is (action, tenant,
> last 1024 decisions). A novel high-value action (e.g., an
> emergency-cancel-DSAR) that has never been cached gets evaluated
> fresh against the malicious fragment. No rate-anomaly detection
> on permit-count surge is specified in §D-10.

The exploit path:

1. Adversary obtains a pack-owner-key (e.g., council-privacy laptop
   compromise via supply-chain Cargo dependency CVE).
2. Adversary signs `pack/EU-GDPR-2018-baseline-v2024/temporary-
   permit.cedar` that permits `Action::ExportTenantData` with no
   tenant restriction.
3. Adversary activates at HH:MM:00 with sunset at HH:MM:03 (a
   3-second window).
4. Within those 3 seconds, the compromised principal exfiltrates
   targeted data by issuing `ExportTenantData` requests against
   high-value tenants.
5. The fragment is then revoked at HH:MM:03; the Cedar audit trail
   shows the permit + revocation cleanly. The data is already in
   attacker hands.

F5 ranks this CRITICAL because:

1. The 5-second hot-reload window is **window-bounded but real**.
   The platform's defensive posture against pack-owner-key
   compromise depends on multispectrum review at publication time,
   but a compromised pack-owner key BYPASSES the review (it can
   sign arbitrary fragments).
2. There is **no rate-of-evaluation anomaly detector** specified.
   A fragment that grants `ExportTenantData` to every tenant for
   3 seconds would generate an enormous permit-count spike that
   should be a SEV-1 signal — but ADR-0243 does not specify
   detection.
3. The Cedar SDK fallback **does not protect** against the attack.
   The fallback applies only when policy-engine is unreachable;
   when policy-engine is reachable and the malicious fragment is
   active, the fallback never engages.

M1-KB-F3 + TMG-05 reinforce the finding from independent angles:
M1 observes the TOCTOU window between activation and enforcement;
TMG-05 observes the absence of a formal adversary model for cache
poisoning across cells.

### Why soak + anomaly-rollback specifically

Four alternatives were considered before settling on the soak +
anomaly-rollback shape:

| Alternative | Why rejected |
|---|---|
| **A. Pre-publication human review per fragment.** Require ≥ 2 humans approve every fragment before publication. | Defeats hot-reload's operational purpose. Some fragments (e.g., emergency forbid fragments per ADR-0243 Appendix B) must publish in <30s; pre-publication human review adds days. |
| **B. Post-publication anomaly-rollback alone, no soak window.** Publish to enforcement immediately; rollback if anomaly detected. | Anomaly detection requires a baseline; a freshly-published fragment has no baseline. The first few seconds of evaluation are exactly when the attack lands. Soak provides the baseline-building window. |
| **C. Soak window alone, no anomaly-rollback.** Wait 60s in shadow-mode; if no human raises a flag, promote to enforcement. | Without anomaly-rollback, the soak window is wasted — the platform still relies on humans to catch the anomaly. |
| **D. Soak window + anomaly-rollback combined.** During soak, the fragment is evaluated SHADOW-MODE (decision computed but not enforced) alongside the prior fragment; if anomaly detector finds the new fragment's behaviour diverges >3σ from the prior, auto-revoke; otherwise promote to enforcement. | Selected. Provides both the baseline-building window AND the automated revocation — closing the F5-243-01 exploit window. |

The selected resolution matches three named precedents:

- **AWS IAM Access Analyzer policy validation (2019+) + AWS Access
  Analyzer findings reachability analysis.** AWS publishes policy
  changes through a "verify reachability" step that simulates the
  policy against a sample of recent access patterns; only after the
  simulation produces no surprising results does the policy
  activate. This is exactly the shadow-mode + anomaly-rollback
  pattern.
- **Google SRE Workbook §16 "Canarying Releases."** Canary
  deployments emit shadow traffic to the new version and compare
  metrics against the baseline before promoting. The pattern
  generalizes to policy changes: a fragment is a configuration
  release.
- **Cloudflare Wrangler progressive deployment + Cloudflare
  Workers' "version overrides" pattern (2023+).** Workers deploy
  new versions to a configurable percentage of traffic for an
  observation window before full rollout.

### Boundary with ADR-0293 (meta-trust-root)

ADR-0293 closes F5-247-01 + the Shamir-expansion arm of F5-243-02:
it adds an independent witness signature for self-modification.

ADR-0294 closes F5-243-01: it adds a soak + anomaly-rollback for ALL
fragment publications, including but not limited to self-
modification fragments.

The two are **complementary and stack**: a self-modification fragment
both (a) requires a meta-trust-root witness signature (ADR-0293),
AND (b) enters the soak + anomaly-rollback window (ADR-0294) before
broad enforcement. A pack-owner compromise that defeats ADR-0293's
witness is still caught by ADR-0294's anomaly detector if the
malicious fragment behaviour diverges from baseline; conversely, a
behaviourally-innocuous-looking fragment that nonetheless authorises
an exfiltration is caught by ADR-0293's witness requirement.

### Why now (2026-05-20)

Three forcing functions:

1. **F5-243-01 is one of the keystone bundle's four CRITICAL
   findings.** ADR-0243 cannot promote to `Accepted` until it is
   closed. The synthesis doc (§5.2) names this ADR as the
   resolution.
2. **The soak detector has substrate dependencies that must be in
   place before the first production Cedar fragment publishes.**
   Per ADR-0243 §D-10 hot-reload uses Kafka pub-sub; per ADR-0263
   observability emission contract; per ADR-0250 audit-chain.
   These substrates must be deployed before the detector can
   function.
3. **The fragment-publisher admission gate is a new CI lane that
   blocks a class of fragment authoring errors detected during
   the multispectrum review.** Reviewers caught at least three
   fragments in the keystone bundle's own corpus whose `sunset_at -
   activate_at` was implicitly less than 60s; the admission gate
   prevents this class permanently.

## Decision

The keystone establishes nine decision sub-sections, D-1 through
D-9.

### D-1. Fragment lifecycle gains a `Soaking` stage

ADR-0243 §D-2 currently defines five fragment lifecycle stages:
`Authored → Reviewed → Signed → Published → Activated → Audited`.
ADR-0294 inserts a sixth stage **between Published and Activated**:

```
Authored
    │
    ▼
Reviewed (multispectrum-review v2.4.0 per ADR-0243 §D-2)
    │
    ▼
Signed (intermediate-key signature per ADR-0243 §D-5)
    │
    ▼
Published (written to cedar-fragment-registry; cosign-attested)
    │
    ▼
─── NEW STAGE ───────────────────────────────────────────
Soaking (≥ 60s shadow-mode evaluation; anomaly detector active)
─── END NEW STAGE ───────────────────────────────────────
    │
    ▼
Activated (full enforcement; replaces prior version if any)
    │
    ▼
Audited (every evaluation emits to audit-chain per ADR-0243 §D-2)
```

The `Soaking` stage has the following invariants:

| Invariant | Description |
|---|---|
| **Mandatory minimum duration** | `sunset_at - activate_at >= 60s` ALWAYS, even for emergency fragments. The 60s minimum is non-negotiable; emergency fragments may set `sunset_at - activate_at = 60s` exactly but no less. |
| **Shadow-mode evaluation** | During soak, the policy-engine evaluates the fragment alongside the prior fragment (if any) and emits both decisions to audit-chain. Only the prior-fragment decision is enforced; the soak-mode decision is recorded but not enforced. |
| **Anomaly detector active** | Per §D-2, the per-cell `oyatie.policy-engine.fragment-soak-detector` monitors permit-rate, denial-rate, P99 latency, and per-resource-class evaluation count for the new fragment's scope; emits an anomaly verdict if any signal diverges >3σ from the prior baseline. |
| **Auto-revoke on anomaly** | Per §D-3, the `oyatie.policy-engine.fragment-anomaly-revoker` reacts to anomaly verdicts by publishing a revocation entry within ≤5s; the fragment never reaches `Activated`. |
| **Promotion to `Activated`** | Per §D-4, after 60s of soak with no anomaly, the fragment automatically transitions to `Activated`. The prior fragment (if any) is retired. |

#### D-1.1. Soaking -> Active -> Sunset continuity

Downstream ADRs sometimes say `Active`; ADR-0243/ADR-0294 persist
the same state as `Activated`. The continuity invariant is therefore:

```
Soaking -> Activated (Active in downstream prose) -> Sunset -> Audited
```

`Sunset` is the explicit retirement state for a fragment that was
previously `Activated` but has been superseded or reached `sunset_at`.
Sunset fragments are not evaluated, not shadow-evaluated, and not
eligible for last-known-good fallback except by an incident-specific
manual override under §D-7. The transition to `Sunset` MUST invalidate
intersecting cache entries, preserve `prior_fragment_id` /
`successor_fragment_id`, and emit `FragmentSunset` with the activation
event that caused retirement.

Reverse-dependency registry for this state machine:

| Dependent ADR | Dependency on this ADR | Reverse pointer here |
|---|---|---|
| ADR-0297 | Abuse-defence Cedar fragments use soak and activation evidence before anti-bot policy packs enforce. | `related_adrs: ADR-0297` |
| ADR-0311 | Dual-tenant boundary fragments cite ADR-0294 for startup validation, publish soak, cache invalidation, and rollback. | `related_adrs: ADR-0311` |
| ADR-0313 | Conglomerate grant and information-barrier fragments depend on soak before parent-read and refusal policies enforce. | `related_adrs: ADR-0313` |
| ADR-0319 | Office-boundary information-barrier packs depend on one policy-engine lifecycle for assignment, clearance, taint, restricted-deal, and overlay fragments. | `related_adrs: ADR-0319` |

### D-2. Soak detector — primitive design

The `oyatie.policy-engine.fragment-soak-detector` is deployed as a
per-cell pod adjacent to the policy-engine evaluator. It monitors
four signal classes:

| Signal | Definition | Anomaly threshold | Window |
|---|---|---|---|
| **Permit rate** | Number of `Permit` decisions per second within the fragment's scope (`applies_to_actions × applies_to_resources`) | >3σ above prior 7-day rolling baseline OR ≥10× absolute jump regardless of σ | 1s sliding window |
| **Denial rate** | Number of `Forbid` decisions per second within the fragment's scope | >3σ above prior 7-day rolling baseline OR ≥10× absolute jump | 1s sliding window |
| **P99 latency** | 99th percentile evaluation latency for the fragment's scope | >3σ above prior 7-day rolling baseline OR >50ms absolute | 5s sliding window |
| **Unique-resource count** | Number of distinct resource IDs evaluated against the fragment per second | >3σ above prior 7-day rolling baseline | 1s sliding window |

#### D-2.1. Statistical model

The baseline is computed as a 7-day rolling exponentially-weighted
moving average + standard deviation per signal per
(actions × resources) scope. Computation:

```rust
// crates/oya-shared-fragment-soak-protocol/src/baseline.rs

pub struct RollingBaseline {
    /// Exponentially-weighted moving average with half-life = 24 hours.
    ewma: f64,
    /// Exponentially-weighted moving standard deviation, same half-life.
    ewmsd: f64,
    /// Last update timestamp for half-life decay calculation.
    last_update: SystemTime,
    /// Number of samples ingested (for warm-up gating).
    sample_count: u64,
    /// Minimum samples before baseline is "warm" enough to drive
    /// anomaly verdicts.
    warm_threshold: u64,  // default 1_000 samples
}

impl RollingBaseline {
    pub fn update(&mut self, observation: f64) {
        let now = SystemTime::now();
        let decay = self.compute_decay(self.last_update, now);
        let weight = 1.0 - decay;
        let new_mean = self.ewma * decay + observation * weight;
        let new_var = (self.ewmsd * self.ewmsd) * decay
            + (observation - new_mean).powi(2) * weight;
        self.ewma = new_mean;
        self.ewmsd = new_var.sqrt();
        self.last_update = now;
        self.sample_count = self.sample_count.saturating_add(1);
    }

    pub fn is_warm(&self) -> bool {
        self.sample_count >= self.warm_threshold
    }

    pub fn z_score(&self, observation: f64) -> Option<f64> {
        if !self.is_warm() || self.ewmsd < 1e-9 {
            return None;
        }
        Some((observation - self.ewma) / self.ewmsd)
    }
}
```

#### D-2.2. Signal smoothing + false-positive budget

To prevent noise-driven false revocations:

1. **Multi-signal correlation.** A fragment is flagged ONLY when
   ≥ 2 of the 4 signals exceed their 3σ thresholds within the same
   1s evaluation window. Single-signal anomalies are recorded but
   do not trigger revocation (they emit a `SoakSingleSignalWarning`
   to the SoC).
2. **Sustained-signal requirement.** A signal must remain above
   threshold for ≥ 3 consecutive 1s windows to be considered
   sustained. Transient spikes (single-window) emit a
   `SoakTransientSpike` warning but do not revoke.
3. **Cold-baseline carve-out.** If the rolling baseline is not
   warm (per §D-2.1), the anomaly detector falls back to absolute
   thresholds: permit-rate > 1000/s OR denial-rate > 1000/s OR
   P99 latency > 100ms OR unique-resource-count > 500/s. Fragments
   in newly-created cells or for newly-introduced actions
   activate against the absolute thresholds for their first 7 days
   until the rolling baseline warms.
4. **Per-fragment false-positive budget.** Each fragment has a
   budget of 1 false-positive-revocation per 30-day rolling window.
   If a fragment is revoked, re-published, and revoked again with
   the second revocation later declared a false-positive by
   council-security, the third re-publication's soak window
   widens from 60s to 600s + requires an additional human
   approver. This budget enforcement creates back-pressure against
   fragment churn.

#### D-2.3. Per-fragment soak scope

The soak detector observes signals scoped to the
**(actions × resources) intersection** of the new fragment and the
prior fragment (if any). Signals outside this intersection are
ignored because the new fragment cannot influence them.

For a brand-new fragment (no prior), the soak scope is the
fragment's full `applies_to_actions × applies_to_resources` set,
and the detector uses the cold-baseline carve-out per §D-2.2.3
because there is no prior baseline for "this fragment's permits."

### D-3. Anomaly revoker — actuator design

The `oyatie.policy-engine.fragment-anomaly-revoker` is a separate
principal from the soak-detector to maintain separation of duty.
The detector observes and emits verdicts; the revoker actuates.

#### D-3.1. Revocation flow

```
1. soak-detector emits AnomalyVerdict to the
   `cedar-fragment-soak-anomaly` Kafka topic. Verdict carries:
   - fragment_id + version
   - signals_above_threshold[]
   - z_scores[]
   - sustained_window_count
   - observation_timestamp
   - cell_id

2. anomaly-revoker consumes the verdict. Within ≤500ms:
   2.1. Verifies the verdict signature against the soak-detector's
        per-cell signing key.
   2.2. Looks up the fragment's current lifecycle stage; verifies
        it is in `Soaking`.
   2.3. Constructs a RevocationEntry with the fragment_id, the
        revocation reason ("soak-anomaly"), the verdict reference,
        and a fresh nonce.
   2.4. Signs the RevocationEntry with the anomaly-revoker's
        intermediate key (chained to org root per ADR-0243 §D-5).
   2.5. Publishes the RevocationEntry to the
        `cedar-fragment-registry-revocations` Kafka topic.

3. Cedar evaluator (every cell) consumes the RevocationEntry:
   3.1. Marks the fragment as `Revoked` in its in-memory cache.
   3.2. From this point forward, the fragment is neither shadow-
        evaluated nor enforced.
   3.3. Subsequent evaluation requests fall back to the prior
        fragment (if any) or to default-deny per ADR-0243 §D-3.

4. audit-chain emits FragmentSoakAnomalyRevoked evidence row;
   SEV-2 alert paged to ops-sre-reliability + council-security.

5. post-incident workflow:
   - oyatie.foundry.adr-drafter automatically opens an incident
     issue summarising the verdict.
   - The fragment's signer (the pack-owner or council-security
     member whose intermediate key signed the fragment) is paged.
   - Within 24 hours, council-security investigates whether the
     anomaly is a false-positive, a fragment-authoring error, or
     a key compromise. The verdict determines:
       * false-positive → fragment re-published (within budget per §D-2.2.4)
       * authoring error → fragment fix + re-publication
       * key compromise → key revocation + meta-trust-root
         ceremony per ADR-0293
```

#### D-3.2. Revoker rate-limit guard

To prevent denial-of-service via spurious AnomalyVerdicts (e.g.,
soak-detector compromise):

- A single soak-detector may issue at most 3 AnomalyVerdicts per
  rolling 5-minute window; the 4th verdict triggers a
  council-security investigation of the detector itself.
- The platform-wide rate ceiling is 10 fragment revocations per
  rolling 1-hour window before SEV-1 escalation; this is more
  than 10× the historic fragment-revocation rate of any
  hyperscaler-class platform (per AWS IAM Access Analyzer
  published metrics).

### D-4. Promotion to `Activated`

After the soak window elapses with no anomaly verdict, the fragment
transitions to `Activated`:

```
1. cedar-fragment-registry detects `current_time >= activate_at +
   60s` AND no AnomalyVerdict referencing the fragment.

2. cedar-fragment-registry writes a FragmentActivationEvent to the
   `cedar-fragment-registry-activations` Kafka topic.

3. Cedar evaluator (every cell) consumes the event:
   3.1. Promotes the fragment from `Soaking` to `Activated`.
   3.2. Retires the prior fragment (if any); the prior fragment's
        `sunset_at` is set to the FragmentActivationEvent timestamp.

4. audit-chain emits FragmentActivated evidence row.

5. If a prior fragment was retired, audit-chain also emits
   FragmentSunset with `prior_fragment_id`, `successor_fragment_id`,
   `sunset_at`, `retired_by_activation_event_id`, and affected
   `(actions × resources × tenant)` scope.
```

The 60s soak window is the MINIMUM. Fragment authors MAY specify a
longer soak window via the `soak_duration` field in the fragment
front matter (default 60s; max 86400s = 24h). Longer soak windows
are appropriate for fragments touching the trust chain or
high-stakes scopes (e.g., HIPAA pack updates) where additional
observation time reduces residual risk.

### D-5. Cache invalidation during soak + activation

Per F5-243-01 recommendation (c): "Cache should be invalidated, not
preserved, when a new fragment with overlapping (actions × resources)
lands."

The cache invalidation policy:

| Event | Cache action |
|---|---|
| Fragment enters `Soaking` | Cached decisions whose `(action, resource, tenant)` triple intersects the new fragment's scope are **marked stale-but-readable** for the soak duration. Stale-readable means: a cache hit during soak returns the cached decision but ALSO triggers a shadow re-evaluation against the new fragment for soak-detector observation. |
| Fragment promoted to `Activated` | Cached decisions whose scope intersects the activated fragment's scope are **invalidated immediately**. Subsequent evaluations bypass the cache and recompute against the new fragment. The cache warms again over the subsequent minute. |
| Fragment reaches `Sunset` | Cached decisions whose scope intersects the sunset fragment are **invalidated immediately**. The evaluator refuses the sunset fragment as an active or shadow candidate and reloads against the successor fragment, prior still-active fragment, or default-deny path. |
| Fragment revoked by anomaly detector | Cached decisions whose scope intersects the revoked fragment's scope are **invalidated immediately**. The cache warms against the prior fragment (if any). |

The cache used is the per-cell Valkey hot-cache per ADR-0243 §D-10
with 1s default TTL. The soak-stale-but-readable state is a new
state added in this ADR; it is implemented as a per-entry flag in
the Valkey value envelope.

### D-6. Fragment-publisher admission gate

Per §5.2 of the synthesis doc: "`sunset_at - activate_at >= 60s`
invariant enforced at fragment-publisher admission."

The admission gate is enforced at three layers:

1. **CI lane.** `oya-check-fragment-soak-window` scans the
   `microservices/policy-engine/fragments/` tree and verifies every
   fragment's `sunset_at - activate_at >= 60s`. Fragments without
   `sunset_at` (long-lived fragments) are exempt; the invariant
   applies only when both fields are present.
2. **cedar-fragment-registry write-time check.** The registry's
   `INSERT` and `UPDATE` paths reject any fragment row where
   `sunset_at IS NOT NULL AND sunset_at - activate_at < INTERVAL
   '60 seconds'`. A SQL CHECK constraint enforces this at the
   database layer.
3. **policy-engine load-time check.** When a Cedar evaluator pod
   loads a fragment, it verifies the invariant; loading is refused
   for violations. This is the last-line-of-defense in case the
   prior two layers are bypassed.

The SQL CHECK:

```sql
-- microservices/cedar-fragment-registry/migrations/0044_soak_window_invariant.sql

ALTER TABLE cedar_fragments
ADD CONSTRAINT soak_window_minimum_60s
    CHECK (
        sunset_at IS NULL
        OR sunset_at - activate_at >= INTERVAL '60 seconds'
    );

ALTER TABLE cedar_fragments
ADD COLUMN soak_phase_started_at TIMESTAMPTZ;

ALTER TABLE cedar_fragments
ADD COLUMN soak_phase_ended_at TIMESTAMPTZ;

ALTER TABLE cedar_fragments
ADD COLUMN soak_anomaly_verdict_id UUID
    REFERENCES cedar_fragment_soak_anomaly_verdicts(verdict_id);

CREATE TABLE cedar_fragment_soak_anomaly_verdicts (
    verdict_id              UUID PRIMARY KEY,
    fragment_id             TEXT NOT NULL,
    fragment_version        TEXT NOT NULL,
    cell_id                 TEXT NOT NULL,
    observed_at             TIMESTAMPTZ NOT NULL,
    signals_above_threshold TEXT[] NOT NULL,
    z_scores                JSONB NOT NULL,
    sustained_window_count  SMALLINT NOT NULL,
    detector_signing_key    BYTEA NOT NULL,
    detector_signature      BYTEA NOT NULL,
    false_positive_declared_at TIMESTAMPTZ,
    false_positive_declared_by TEXT,
    audit_emission_hash     BYTEA NOT NULL,
    UNIQUE (fragment_id, fragment_version, observed_at)
);

CREATE INDEX cedar_fragment_soak_anomaly_verdicts_fragment_idx
    ON cedar_fragment_soak_anomaly_verdicts (fragment_id, observed_at DESC);

CREATE TABLE cedar_fragment_revocations (
    revocation_id           UUID PRIMARY KEY,
    fragment_id             TEXT NOT NULL,
    fragment_version        TEXT NOT NULL,
    revocation_reason       TEXT NOT NULL CHECK (revocation_reason IN (
        'soak-anomaly', 'manual-override', 'sunset-elapsed',
        'duress-revocation', 'authoring-error', 'key-compromise'
    )),
    revocation_verdict_ref  UUID REFERENCES cedar_fragment_soak_anomaly_verdicts(verdict_id),
    revoked_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_by              TEXT NOT NULL,
    revocation_signature    BYTEA NOT NULL,
    audit_emission_hash     BYTEA NOT NULL,
    UNIQUE (fragment_id, fragment_version, revocation_reason)
);

CREATE TABLE cedar_fragment_false_positive_budget (
    fragment_id             TEXT PRIMARY KEY,
    revocation_count_30d    SMALLINT NOT NULL DEFAULT 0,
    last_false_positive_at  TIMESTAMPTZ,
    soak_widening_factor    SMALLINT NOT NULL DEFAULT 1,  -- 1x = 60s; 10x = 600s; ...
    additional_approver_required BOOLEAN NOT NULL DEFAULT FALSE
);
```

### D-7. Manual override path

Per the synthesis doc: "the manual override path." Some operational
scenarios legitimately require bypassing the soak window:

1. **Emergency forbid fragment (already-existing per ADR-0243
   Appendix B).** A forbid fragment that closes a live security
   incident MAY publish with `soak_duration = 60s` (the minimum)
   AND with a council-security + council-architecture co-signature
   that explicitly grants soak-window-minimum-only. The fragment
   still soaks for 60s; the override is that 60s is the entire
   soak, not 600s or longer.
2. **Anomaly-revoker false-positive resolution.** If the anomaly
   revoker revokes a legitimate fragment, council-security may
   declare false-positive within 24 hours (per §D-2.2.4). The
   declaration triggers fragment re-publication WITH the same
   60s soak window but exempts the second publication from the
   budget penalty if the false-positive is confirmed (i.e., the
   `revocation_count_30d` is decremented, not incremented).
3. **Indefinite-soak override.** For fragments touching the
   meta-permit (per ADR-0247 §D-8) or the meta-trust-root scope
   (per ADR-0293), council-security MAY require an indefinite
   soak (`sunset_at = NULL` is interpreted as "never auto-
   activate; activation requires explicit council-security
   ratification within the soak window"). The fragment remains in
   `Soaking` until ratified; the ratification action is itself
   gated by ADR-0247 §D-8.

The override paths are themselves logged in audit-chain as
`FragmentSoakOverride` events with explicit reason codes.

### D-8. SDK fallback semantics gain anomaly-rollback as a third causal path

ADR-0243 §D-11 currently defines two fallback paths:

1. policy-engine unreachable → use last-known-good cached decision
   for ≤ 30s.
2. cached decision stale → fail-closed (default-deny).

This ADR adds a third path:

3. **Fragment in `Revoked` state due to soak-anomaly** → the
   Cedar SDK rejects evaluation requests that would have been
   resolved by the revoked fragment. The SDK does NOT silently
   fall back to the prior fragment; instead, it returns a
   `Decision::Forbid { reason: "fragment-revoked-by-soak-anomaly" }`
   to the caller, forcing the caller to explicitly handle the
   denial.

The rationale: silent fallback would mask the revocation from the
caller. The soak-anomaly revocation is a SIGNAL that something is
wrong; callers must be able to see it.

### D-9. Per-tenant overlay soak semantics

Per ADR-0243 §D-1.4, per-tenant overlays compose at evaluation
time. The soak window applies per overlay scope: a tenant-scope
overlay's soak detector observes signals scoped to that tenant's
evaluation traffic only, not the platform-wide aggregate.

This means a tenant-scope overlay fragment in soak does not need to
generate platform-wide traffic; the per-tenant baseline is computed
from the tenant's own traffic history. For new tenants whose history
is too short to warm the baseline, the cold-baseline carve-out per
§D-2.2.3 applies.

## Consequences

### Positive

1. **The F5-243-01 exploit window is closed.** A compromised pack-
   owner-key publishing a malicious fragment with a 3-second
   activation window is rejected at the fragment-publisher
   admission gate (§D-6); even if the admission gate is bypassed,
   the soak detector observes the permit-rate spike and the
   anomaly-revoker auto-revokes within 500ms; the fragment never
   reaches `Activated`.
2. **The soak window provides a baseline for anomaly detection.**
   Fresh fragments enter shadow-mode evaluation; the detector
   builds a baseline against real traffic before the fragment
   enforces.
3. **The anomaly detector is bounded by statistical false-positive
   budget.** The 3σ + multi-signal correlation + sustained-window
   + per-fragment budget tunes the detector against operational
   noise without blunting its sensitivity to adversarial
   patterns.
4. **The cache invalidation policy preserves performance while
   preserving correctness.** The stale-but-readable state allows
   the cache to keep serving during soak; full invalidation at
   `Activated` ensures no stale evaluations linger.
5. **The mechanism composes cleanly with ADR-0293.** Self-
   modification fragments require BOTH meta-trust-root witness AND
   soak + anomaly-rollback; the two ADRs provide independent
   defense layers.

### Negative

1. **Operational latency increases for all fragment publications.**
   Every fragment now takes at least 60s longer to reach
   enforcement. For routine fragment updates this is acceptable;
   for true emergencies, the 60s minimum is the operational
   floor.
2. **Compute overhead for shadow evaluation.** During soak, the
   policy-engine evaluates every applicable request against BOTH
   the new and the prior fragment. For high-traffic actions this
   ~ doubles the policy-engine CPU during the soak window. The
   amortized cost is small (60s per fragment publication) but the
   peak provisioning must accommodate it.
3. **The detector's 3σ threshold may produce false-positives in
   bursty traffic.** Mitigated by multi-signal correlation +
   sustained-window requirements, but not eliminated. The
   false-positive budget (§D-2.2.4) creates back-pressure.
4. **The anomaly-revoker is itself a privileged actor whose
   compromise could DoS the platform.** Mitigated by:
   - Separation of duty (detector emits; revoker actuates).
   - Rate-limit guard (§D-3.2): at most 10 revocations/hour
     before SEV-1 escalation.
   - The revoker's intermediate key is rotated quarterly (more
     frequently than other intermediate keys per the elevated
     blast radius).
5. **Brand-new actions / brand-new tenants run on cold-baseline
   absolute thresholds.** During the first 7 days, the detector
   uses absolute thresholds that may be either too strict (false
   positives) or too lax (missed adversarial patterns). The
   transition to rolling-baseline at day-7 is monitored.

### Neutral

1. **Long-lived baseline fragments (`sunset_at = NULL`) are
   exempt from the `sunset_at - activate_at >= 60s` invariant.**
   The 60s minimum applies only to fragments with a sunset; baseline
   fragments are by definition not bounded.
2. **The soak mechanism is invisible to customer-tenant
   principals.** Customer tenants never see fragments in soak
   directly; the Cedar SDK's decision response is identical
   regardless of whether a fragment is in soak or activated (the
   prior fragment's decision is the one returned during soak).

## Detailed Mechanics

### D-1 expanded — fragment front matter gains `soak_*` fields

```yaml
# microservices/policy-engine/fragments/baseline/example-fragment.cedar
# (front matter, before Cedar code)

---
fragment_id: baseline/example-fragment
version: v3
scope: baseline
applies_to_actions:
  - Workflow::Action::Read
  - Workflow::Action::Update
applies_to_resources:
  - Workflow
effective_at: 2026-05-21T10:00:00Z
activate_at: 2026-05-21T10:00:00Z
sunset_at: null  # long-lived

# NEW SOAK FIELDS (per ADR-0294)
soak_duration_seconds: 60  # default; must be ≥ 60
soak_phase_started_at: null  # populated by cedar-fragment-registry
soak_phase_ended_at: null    # populated on activation OR revocation
soak_anomaly_verdict_id: null  # populated on revocation

# Existing fields
signed_by:
  signer_key_id: org-baseline-key-ed25519-fingerprint
  signature: <ed25519-bytes>
  cosign_attestation_id: <cosign-uuid>
---
```

### D-2 expanded — detector deployment topology

```
Per Tier-2 / Tier-3 cell:

┌─────────────────────────────────────────────────────────────┐
│                       Cell N                                 │
│                                                              │
│   ┌────────────────────┐   ┌─────────────────────────────┐  │
│   │ policy-engine pod  │   │ fragment-soak-detector pod  │  │
│   │ (Cedar evaluator)  │◄──┤ (consumes Kafka audit       │  │
│   │                    │   │  topic + Cedar emission)    │  │
│   │ Emits per-eval     │   │                              │  │
│   │ events to Kafka    │   │ Maintains rolling baselines  │  │
│   │                    │   │ per fragment per signal      │  │
│   │                    │   │                              │  │
│   │                    │   │ Emits AnomalyVerdict events  │  │
│   └────────────────────┘   └─────────────────────────────┘  │
│            │                            │                     │
│            ▼                            ▼                     │
│   ┌─────────────────────────────────────────────────────┐   │
│   │              Cell-local Kafka broker                  │   │
│   │                                                       │   │
│   │  Topics:                                              │   │
│   │  - cedar-fragment-evaluation-events                  │   │
│   │  - cedar-fragment-soak-anomaly                       │   │
│   │  - cedar-fragment-registry-revocations               │   │
│   │  - cedar-fragment-registry-activations               │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

Detector pod resource budget:
- CPU: 500m baseline / 2 cores peak (during soak)
- Memory: 512Mi baseline / 2Gi peak
- Disk: 50Gi for 7-day rolling baseline state
- Network: Kafka consume + emit; ≤ 50Mbit/s peak

### D-3 expanded — revoker as a Cedar fragment publisher

The anomaly revoker is itself authorized to publish a class of Cedar
fragments — specifically, REVOCATION fragments:

```cedar
// microservices/policy-engine/fragments/baseline/anomaly-revoker-permits.cedar
// SCOPE: baseline
// SIGNED BY: org-baseline-key
// SOAK_DURATION: 600 (10 minutes — extended soak for the revoker's
//                     own permits, given its blast radius)

permit (
  principal == Principal::"oyatie.policy-engine.fragment-anomaly-revoker",
  action == Cedar::Action::PublishFragmentRevocation,
  resource is CedarFragment
)
when {
  // Revocation must reference a valid soak-anomaly verdict
  context.revocation.reason == "soak-anomaly"
  && context.revocation.verdict_id != null
  && context.revocation.verdict_signature_valid
  // Revocation must target a fragment currently in Soaking
  && resource.lifecycle_stage == "Soaking"
  // Revocation must occur within the soak window
  && context.now < resource.activate_at + duration_seconds(resource.soak_duration_seconds)
};
```

### D-5 expanded — cache state machine

```
State transitions per cache entry per fragment scope:

[Fresh] ──evaluation──► [Cached]
   │                       │
   │                       │ TTL expires (1s default)
   │                       ▼
   │                   [Fresh]
   │
   │ Fragment enters Soaking
   ▼
[Cached-StaleReadable] ──evaluation──► (a) Return cached + shadow-eval new fragment
   │                                  (b) Emit shadow result to soak-detector
   │
   │ Fragment promoted to Activated
   ▼
[Invalidated] ──evaluation──► [Fresh] (recompute from new fragment)
   │
   │ Fragment revoked by soak-anomaly
   ▼
[Invalidated] ──evaluation──► [Fresh] (recompute from prior fragment)
```

### D-6 expanded — admission gate failure modes

```
Failure mode 1: CI lane detects sunset_at - activate_at < 60s in PR
  PR review fails; author must adjust sunset_at upward
  Audit row: `oya-check-fragment-soak-window FAIL`

Failure mode 2: PR review passes CI but registry CHECK refuses INSERT
  cedar-fragment-registry returns SQL constraint violation
  Audit row: `FragmentRegistryAdmissionRejected`
  Investigation: how did the PR pass CI if the constraint violates?

Failure mode 3: Registry accepts but policy-engine refuses to load
  Suggests cedar-fragment-registry schema drift or in-flight DDL
  Audit row: `FragmentLoadRejected`; SEV-2 alert
```

## Implementation Footprint

### Microservice scope

| Microservice | Change | Effort |
|---|---|---|
| `microservices/policy-engine/` | Add Soaking lifecycle stage; add shadow-mode evaluation; add SDK fallback semantics for revoked fragments | ≈ 3 weeks |
| `microservices/cedar-fragment-registry/` | Add migrations for soak fields; add admission CHECK constraints; add revocation + verdict tables | ≈ 2 weeks |
| `microservices/observability/` | Add per-cell soak-detector pod chart + dashboard | ≈ 2 weeks |
| `crates/oya-shared-fragment-soak-protocol/` (new) | Shared crate exposing baseline + verdict + revocation primitives | ≈ 3 weeks |
| `crates/oya-policy-engine-fragment-soak-detector/` (new) | Soak detector binary; per-cell deployment | ≈ 3 weeks |
| `crates/oya-policy-engine-fragment-anomaly-revoker/` (new) | Anomaly revoker binary; central per-region deployment | ≈ 2 weeks |

Total: ≈ 15 weeks engineering effort across crews. Parallelizable;
calendar time ≈ 4 weeks.

### CI lane scope

| CI lane | Behavior |
|---|---|
| `oya-check-fragment-soak-window` | Scans fragments under `microservices/policy-engine/fragments/`; verifies `sunset_at IS NULL OR sunset_at - activate_at >= 60s`; emits findings for violations |
| `oya-check-fragment-anomaly-detector-coverage` | Verifies every Tier-2 + Tier-3 cell has an active soak-detector pod reporting green heartbeats within last 60s |
| `oya-check-fragment-publisher-admission` | Static analysis of cedar-fragment-registry schema; verifies the SQL CHECK constraint is in place |
| `oya-check-fragment-rollback-evidence-emitted` | Verifies that every recorded `cedar_fragment_revocations` row has a corresponding audit-chain Merkle leaf |

### Observability scope

The soak detector emits to the cell-local Kafka + the per-region
observability rollup per ADR-0263. Dashboards required:

1. **Per-cell fragment-soak panel.** For each fragment in Soaking,
   show the 4 signals + 3σ thresholds + warm-baseline status.
2. **Cross-cell soak anomaly heatmap.** Aggregates AnomalyVerdicts
   by fragment + cell; surfaces patterns (e.g., one fragment
   anomalous in only one cell suggests a cell-specific issue;
   anomalous in all cells suggests a fragment issue).
3. **Fragment lifecycle Sankey.** Tracks fragments through
   Authored → Reviewed → Signed → Published → Soaking → Activated
   / Revoked; surfaces drop-off rates and revocation patterns.

## Migration

### Stage 0 — Schema migration + shared crate (T+0 to T+3w)

| Step | Action |
|---|---|
| 0.1 | `oya-shared-fragment-soak-protocol` crate scaffolded with baseline, verdict, revocation, and false-positive-budget primitives |
| 0.2 | cedar-fragment-registry migrations 0044 (soak invariant), 0045 (verdict table), 0046 (revocation table), 0047 (false-positive budget) applied to all environments |
| 0.3 | Existing fragments backfilled with `soak_duration_seconds = 60` default |

### Stage 1 — Detector + revoker deployment (T+3w to T+6w)

| Step | Action |
|---|---|
| 1.1 | `oya-policy-engine-fragment-soak-detector` deployed to `dev-tools-cell-staging` first; soaks for 7 days collecting baseline |
| 1.2 | `oya-policy-engine-fragment-anomaly-revoker` deployed to staging; integration tests cover happy path + 6 failure modes |
| 1.3 | Detector + revoker promoted to `dev-tools-cell-dev` and Tier-3 cells progressively |
| 1.4 | All cells report green heartbeats; baselines warm |

### Stage 2 — Lifecycle integration (T+6w to T+8w)

| Step | Action |
|---|---|
| 2.1 | policy-engine pods upgraded to include the Soaking stage in the lifecycle state machine |
| 2.2 | Cedar SDKs (Rust, TypeScript, Python) upgraded to surface the new `Decision::Forbid { reason: "fragment-revoked-by-soak-anomaly" }` |
| 2.3 | Cache state machine extended with Stale-Readable state |
| 2.4 | End-to-end rehearsal: a controlled malicious-looking fragment is published; soak detector flags; anomaly revoker auto-revokes; audit trail verified |

### Stage 3 — Advisory → BLOCKER (T+8w)

| Step | Action |
|---|---|
| 3.1 | All four CI lanes (`oya-check-fragment-soak-window` et al.) flip from advisory to BLOCKER |
| 3.2 | ADR-0243 §D-2 amended (per the `requires_amendment_to` list above) |
| 3.3 | ADR-0246 amended (per the `requires_amendment_to` list above) |
| 3.4 | The bundle's promotion gate for ADR-0243 closes |

## References

### Primary

- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` §5.2 —
  authority for this ADR's existence.
- `evidence/debate/keystone-bundle-2026-05-20-F5-security-r1.json` —
  F5-243-01 (CRITICAL).
- `evidence/debate/keystone-bundle-2026-05-20-M1-challenge-r1.json`
  — M1-KB-F3 (Cedar fragment hot-reload TOCTOU; same finding from
  a different reviewer).

### Related ADRs

- ADR-0243 (Cedar as Universal Gate) — fragment lifecycle origin;
  this ADR adds the Soaking stage.
- ADR-0246 (Policy Engine Substrate Promotion) — fragment-publisher
  admission gate; this ADR adds the soak-window invariant
  enforcement.
- ADR-0247 (Self-Hosting / Self-Modification Doctrine) — self-
  modification fragments enter both ADR-0293's witness path AND
  this ADR's soak window.
- ADR-0293 (Foundry Meta-Trust-Root) — complementary defense
  layer; both ADRs stack for self-modification fragments.
- ADR-0263 (Observability Emission Contract) — soak detector
  emits per this contract.
- ADR-0297 (Abuse Defence Baseline) — anti-bot/spoof/scrape Cedar
  fragments depend on this soak, activation, and sunset lifecycle.
- ADR-0311 (Dual Tenant Identity) — work/personal boundary fragments
  cite this ADR for soak validation, cache invalidation, and rollback.
- ADR-0313 (Conglomerate-Tenant Hierarchy) — grant and barrier
  fragments depend on this one-policy-engine lifecycle.
- ADR-0319 (Front/Middle/Back Office Information Barrier) — office
  barrier policy packs depend on this soak-to-active-to-sunset path.

### Industry references

- **AWS IAM Access Analyzer (2019 + ongoing).** Reachability
  analysis + policy validation precedent.
- **Google SRE Workbook Chapter 16 "Canarying Releases."** Shadow-
  mode evaluation + baseline comparison precedent.
- **Cloudflare Wrangler progressive deployment + Workers' version
  overrides (2023).** Per-version traffic-split deployment.
- **Netflix Hystrix circuit breaker + Spinnaker canary analysis
  (Kayenta).** Statistical canary analysis with 3σ thresholds is
  Kayenta's primary mechanism.
- **Microsoft Azure Front Door progressive deployment.** Multi-
  region staged rollout with anomaly auto-rollback.
- **HashiCorp Sentinel policy-as-code with sentinel test command.**
  Pre-publication policy simulation precedent.

### Statistical references

- **Welford's online algorithm (Welford 1962) + West (1979)
  "Updating mean and variance estimates: an improved method."**
  The rolling-EWMA + EWMSD computation in §D-2.1.
- **Hyndman & Athanasopoulos (2018) "Forecasting: principles and
  practice."** Sliding-window anomaly detection.
- **Cleveland (1979) "Robust Locally Weighted Regression and
  Smoothing Scatterplots."** LOWESS smoothing as alternative if
  EWMA proves too brittle.
- **NIST SP 800-94 Rev. 1 "Guide to Intrusion Detection and
  Prevention Systems."** Multi-signal correlation precedent for
  anomaly verdict combination.

### Slice cross-references

- **Slice 1 (runbooks):**
  `docs/runbooks/cedar-fragment-soak-anomaly-incident-response.md`,
  `docs/runbooks/cedar-fragment-emergency-rollback.md`,
  `docs/runbooks/cedar-fragment-soak-detector-tuning.md` are
  required by this ADR's CI lanes; their authoring is in Slice 1
  scope.
- **Slice 3 (ADR-0246 amendment):** The
  fragment-publisher admission gate (§D-6) requires an amendment
  to ADR-0246's fragment-validation lane catalogue; the actual
  amendment to ADR-0246 is in Slice 3 scope.
- **Slice 4 (naming justifications):** The four new names
  (`oyatie.policy-engine.fragment-soak-detector`,
  `oyatie.policy-engine.fragment-anomaly-revoker`,
  `oya-shared-fragment-soak-protocol`,
  `oya-check-fragment-soak-window`) are justified in this ADR's
  front matter `naming_justifications:` block per
  `feedback_naming_justification`.

### Specifications

- `/specs/cedar-fragment-soak-protocol.json` (new) — canonical
  machine-readable record of the soak protocol, signal thresholds,
  warm-baseline criteria, and false-positive-budget rules.
- `/specs/cedar-fragment-schema.json` — extended with
  `soak_duration_seconds`, `soak_phase_started_at`,
  `soak_phase_ended_at`, and `soak_anomaly_verdict_id` fields.
- `/specs/policy-gate-coverage.json` — updated to require the
  4 new soak-related lanes in the coverage report.

### Memory references

- `feedback_cedar_as_universal_gate` — Cedar evaluation is the
  enforcement primitive; the soak mechanism extends the Cedar
  lifecycle.
- `feedback_no_silent_regression` — the v1 ADR-0243 lifecycle is
  amended (not silently changed); fragment lifecycle is now
  6-stage rather than 5-stage; the change is documented in this
  ADR and CI-enforced.
- `feedback_quality_performance_scalability_bar` — the 60s soak +
  ≤500ms anomaly-revoker latency match the hyperscaler-grade
  defensive posture; the detector + revoker scale per-cell.
- `feedback_clean_architecture_requirements` — the separation of
  duty (detector observes; revoker actuates) maintains clean
  architecture and reduces blast radius from any single principal
  compromise.
- `feedback_automate_everything` — the anomaly-rollback is
  automated end-to-end; humans intervene only on false-positive
  declaration.
- `feedback_naming_justification` — the four new names carry
  inline justification.

---

**End of ADR-0294.**
