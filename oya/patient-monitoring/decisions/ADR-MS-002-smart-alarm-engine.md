# ADR-MS-002 — Smart-alarm engine: declarative rules + Cedar-gated suppression

**Status**: Accepted
**Date**: 2026-05-21
**Microservice**: patient-monitoring
**Scope**: µservice-internal
**Author**: axis-clinical-realtime
**Binding to**: ADR-0243 (Cedar universal gate), ADR-0244 (tenant scoping),
ADR-0251 (compliance pack + cell certification), ADR-0332 (clinical-realtime substrate).

---

## 1. Context

Alarm fatigue is the dominant clinician-safety problem in ICU/CCU/PACU/ED units. Industry
data: nurses receive 150-700 alarms per patient per day across vendor systems; ≥ 80% are
false-positive or nuisance. The Joint Commission has flagged alarm fatigue as a National
Patient Safety Goal (NPSG.06.01.01) since 2014. ECRI Institute consistently lists alarm
hazards in their top-10 health-tech hazards.

The patient-monitoring µservice must:

- Reduce alarm-fire count ≥ 40% vs. the dumb-threshold baseline
- Maintain 100% sensitivity for `critical` + `life-threatening` events
- Provide authority-bounded suppression (a single clinician cannot indefinitely silence the
  µservice; suppression carries justification + duration)
- Comply with FDA 21 CFR Part 11 (every alarm action is an electronic signature event)
- Comply with IEC 62304 SaMD Class C (life-supporting software lifecycle)

Existing approaches surveyed:

1. **Pure-ML alarm-fatigue model**: highest reduction potential but opaque; FDA SaMD Class C
   requires interpretability for life-supporting software; rejected as primary.
2. **Static threshold tuning per unit**: insufficient; misses compound conditions and
   patient-specific variance.
3. **Vendor-proprietary smart-alarm features (e.g., Philips "smart-alarm filtering",
   Masimo "Sensor Compass")**: closed-source; not portable; rejected.
4. **Declarative rule engine + Cedar-gated suppression + optional ML augmentation**: chosen.
   The rule engine handles the safety-critical default path (interpretable, auditable); the
   ML augmentation (per ADR-MS-003) refines but cannot suppress the rule-engine output.

## 2. Decision

The patient-monitoring µservice implements the smart-alarm engine as a **Rust-native
declarative rule engine** with the following capability set:

1. **Validity check**: drop sample if `lead_confidence < 0.5` or `quality_indicator = 'invalid'`.
2. **Persistence requirement**: a threshold breach must persist N samples before firing
   (default N=5 for HR, N=10 for SpO2, N=3 for BP; per-parameter, per-pack configurable).
3. **Compound condition**: multi-parameter rules expressed as DSL — e.g., `HR > 130 AND
   SpO2 < 92 AND RR > 28` fires as a single `critical` event rather than three.
4. **Patient-specific thresholds**: per-patient overrides (e.g., chronic-AFib patient with
   baseline HR > 100 gets the single-parameter HR alarm relaxed but the compound preserved).
5. **Diurnal adaptation**: optional night-time threshold shift (e.g., HR < 50 alarm threshold
   relaxed to HR < 45 between 22:00-06:00 local, if patient is stable and asleep).
6. **Trend gating**: a slow-creep threshold breach may be re-classified as `trend-derived`
   (lower urgency) if the trend has been stable for > 30 minutes.
7. **Dedup window**: identical alarms on the same channel within a 5-minute rolling window
   collapse into the existing alarm record unless severity increases.

**Cedar gate**: alarm-suppression actions require Cedar evaluation against the
`policies/alarm-suppression-requires-justification.cedar` bundle. The principal must be a
`physician_attending` (or equivalent role per tenant pack); the suppression must include
non-empty justification text and a duration ≤ 4 hours.

**Audit**: every alarm-fire, alarm-ack, alarm-suppress, and alarm-clear is emitted to
audit-chain µservice as a hash-chained event with HIPAA + 21 CFR Part 11 electronic-signature
properties (clinician identity + HLC timestamp + device).

**ML augmentation (per ADR-MS-003)**: optional. The ML model may **lower priority** of an
alarm based on context (e.g., known artifact pattern) or **raise priority** based on
multi-parameter deterioration trend. The ML augmentation may **never suppress** a
`critical` or `life-threatening` rule-engine output.

## 3. Rule DSL

The DSL is a constrained subset of Cedar-style expressions with the following primitives:

```
RULE my-tachycardia-with-hypoxia
WHEN  HR > 130 PERSIST 5 AND SpO2 < 92 PERSIST 10
WITH  lead_confidence(HR) >= 0.5 AND lead_confidence(SpO2) >= 0.5
GATE  patient.age >= 18 AND patient.has_chronic_afib != true
FIRES critical
```

Each rule produces a `SmartAlarmEvaluation` record with rule ID, input snapshot, decision,
and lineage trail. Evaluations are retained 7Y per HIPAA + 21 CFR.

## 4. Performance budget

| Operation | p99 |
|---|---|
| Per-sample rule eval (single parameter) | ≤ 5 ms |
| Per-sample rule eval (compound, 4 parameters) | ≤ 10 ms |
| Rule-bundle reload (on tenant pack update) | ≤ 30 s |
| Cedar suppression eval | ≤ 5 ms |
| Alarm-fire → AsyncAPI emit | ≤ 50 ms |
| Audit-chain emit per alarm event | ≤ 100 ms |

## 5. Consequences

### 5.1 Positive

- Interpretable; clinical-engineer and SaMD reviewer can read a rule and predict its
  behavior.
- Cedar-gated suppression is auditable; supports FDA inspector review of suppression
  patterns (UJ-35).
- Patient-specific thresholds + diurnal adaptation + trend gating reduce nuisance alarms
  ≥ 40% vs. dumb-threshold baseline.
- ML augmentation can lift the model without compromising the rule-engine safety floor.

### 5.2 Negative

- Rule DSL adds a per-tenant configuration surface that must be maintained by the unit's
  clinical engineer or biomedical staff. Mitigation: ship a default rule bundle per
  clinical setting (general acute, ICU, NICU, PICU, RPM, ED) that covers ≥ 80% of installs.
- ML augmentation requires careful guardrails to prevent it from suppressing a safety-critical
  rule-engine output. Mitigation: ADR-MS-003 enforces the `ml_cannot_suppress_critical`
  invariant at the boundary.

### 5.3 Neutral

- The compound-condition primitive is a superset of every vendor's smart-alarm feature
  but presented in an interpretable form; clinical-engineer adoption may require training.

## 6. Implementation

### 6.1 Crate layout

```
crates/patient-monitoring-smart-alarm-domain/
  src/lib.rs                    # domain types + invariants
  src/rule_dsl.rs               # DSL parser
  src/evaluator.rs              # rule evaluator
  src/persistence_buffer.rs     # N-sample persistence
  src/compound.rs               # compound-condition combinator
  src/diurnal.rs                # diurnal adaptation
  src/trend_gate.rs             # trend-gating
  src/dedup.rs                  # rolling-window dedup
  src/suppression.rs            # Cedar-gated suppression
  src/audit_emit.rs             # audit-chain emit
```

### 6.2 Test posture

- Unit tests: every primitive (validity, persistence, compound, diurnal, trend-gate, dedup,
  suppression) has table-driven tests.
- Integration tests: rule-engine replay against historical vital-signs traces from MIMIC-IV
  (de-identified) — verify ≥ 40% reduction vs. dumb-threshold baseline; verify 100%
  sensitivity for life-threatening events.
- Chaos tests: rule-engine restart, rule-bundle hot-reload, Cedar engine timeout.

## 7. References

- ADR-0243 Cedar universal gate
- ADR-0244 Tenant scoping
- ADR-0251 Compliance pack + cell certification
- ADR-0332 Clinical-realtime substrate
- TJC NPSG.06.01.01 (clinical alarm safety)
- ECRI Institute top-10 health-tech hazards (alarm fatigue)
- AAMI EC57 (testing physiologic monitor algorithms)
- FDA 21 CFR Part 11 (electronic records + electronic signatures)
- IEC 62304:2015 (medical device software lifecycle)
