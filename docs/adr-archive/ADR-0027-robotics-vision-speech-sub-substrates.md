---
id: ADR-0027
status: Rejected
doc_status: published
---

> **Disposition light-edit (2026-08-06):** Keep Rejected: Robotics/vision/speech — out of current operational horizon; reopen when product prioritizes

# ADR-0027: Robotics, vision, and speech sub-substrates — vision/speech model crates, robotics control plane, deterministic latency, safety-critical anti-scope

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry` (model substrates) + `vertical-industrial` (robotics control consumers)
> **Date:** 2026-05-09
> **Related:** ADR-0020 (provider adapter — vision/speech models extend the trait), ADR-0022 (autonomy ceiling — T4 disabled by default for safety-critical actuation), ADR-0024 (eval harness — per-substrate eval cohorts), ADR-0026 (in-house substrate — vision/speech models live on the same kernel)

---

## Context

The capability registry (ADR-0021) already accommodates capabilities whose model substrate is text — chat, summarization, code, retrieval. Vision (OCR, classification, detection, video analytics, scene understanding), speech (STT, TTS, voice biometrics, wake-word), and robotics control (autonomous mobile robots, robotic arms, drones, autonomous vehicles, AGVs) are first-class capability classes for our vertical surfaces — manufacturing, logistics, retail, hospitality, agriculture, public sector — but they each have failure modes that text capabilities do not: a vision misclassification can route a wrong package; a TTS hallucination can mislead a caller; a robotic-arm misactuation can injure a worker; an autonomous vehicle wrong decision can kill someone. The autonomy ceiling (ADR-0022) and the sandbox (ADR-0023) protect against software-class failures but not against the physical-actuation class.

We also have an explicit anti-scope: no defense / weaponized robotics work without a founder-and-legal carve-out. This is not just a values statement; it must be structurally enforceable in the catalog so an agent cannot author a capability that quietly drifts into the anti-scope.

---

## Decision

We define three sub-substrates as Foundry capability classes, layered on the in-house model kernel (ADR-0026): **vision**, **speech**, and **robotics control**. Real-time control loops have deterministic latency budgets; safety-critical actuation defaults to T4-disabled per ADR-0022; per-vertical safety regulators bind in the registry; simulation-first for new actuation capabilities; per-region anti-scope (defense / weaponized robotics) is structurally enforced.

### Vision substrate (`crates/oya-intelligence-model-vision-*`)

```
crates/oya-intelligence-model-vision-kernel
crates/oya-intelligence-model-vision-ocr-app
crates/oya-intelligence-model-vision-classification-app
crates/oya-intelligence-model-vision-detection-app
crates/oya-intelligence-model-vision-video-analytics-app
crates/oya-intelligence-model-vision-facial-recognition-app   // lawful-only; per-pack hard gates
crates/oya-intelligence-model-vision-scene-anomaly-app
```

```rust
// crates/oya-intelligence-model-vision-kernel/src/lib.rs
pub enum VisionTask {
    Ocr { script_set: ScriptSet, locale_hint: Option<LocaleId> },
    ImageClassification { taxonomy: TaxonomyRef },
    ObjectDetection { class_set: ClassSet },
    VideoAnalytics { frame_rate: FrameRate, retention_policy: RetentionPolicy },
    FacialRecognition {
        lawful_basis: LawfulBasis,         // per-pack required; e.g. consent | legitimate-interest | court-order
        consent_receipts: Vec<ConsentRef>,
    },
    SceneAnomaly { baseline_ref: BaselineRef },
}
```

Facial recognition carries a structurally enforced lawful-basis check: the registry refuses publish without a per-pack `LawfulBasis` declaration; the runtime refuses invocation without matching consent receipts. Per-region pack overrides may HARD_DENY facial recognition entirely.

### Speech substrate (`crates/oya-intelligence-model-speech-*`)

```
crates/oya-intelligence-model-speech-kernel
crates/oya-intelligence-model-speech-stt-app
crates/oya-intelligence-model-speech-tts-app
crates/oya-intelligence-model-speech-voice-biometric-app
crates/oya-intelligence-model-speech-wake-word-app
```

```rust
// crates/oya-intelligence-model-speech-kernel/src/lib.rs
pub enum SpeechTask {
    Stt { locale: LocaleId, streaming: bool },
    Tts { locale: LocaleId, voice_profile: VoiceProfileRef },
    VoiceBiometric { lawful_basis: LawfulBasis },
    WakeWord { custom_phrases: Vec<WakePhrase> },
}

pub struct LocaleCoverage {
    pub stt: LocaleSet, // KR + JP + EN + ES + PT + HI + AR minimum (per ADR-0026)
    pub tts: LocaleSet,
}
```

Voice biometrics carries the same lawful-basis gate as facial recognition. TTS voice cloning of a real human requires explicit consent recorded in the audit chain.

### Robotics control plane

```
crates/oya-intelligence-robotics-control-kernel        // Foundry-side control primitives
crates/oya-intelligence-robotics-control-app           // policy-bound control loop runner
crates/oya-vertical-industrial-robotics-agv-app
crates/oya-vertical-industrial-robotics-amr-app
crates/oya-vertical-industrial-robotics-arm-app
crates/oya-vertical-industrial-robotics-drone-app // lawful-only; per-pack airspace gates
crates/oya-vertical-industrial-robotics-av-app    // autonomous vehicles; per-region certification
```

```rust
// crates/oya-intelligence-robotics-control-kernel/src/lib.rs
pub struct ControlLoop {
    pub loop_id: ControlLoopId,
    pub deterministic_budget: LatencyBudget {
        pub p50_us: u32,    // microseconds, not milliseconds
        pub p99_us: u32,
        pub max_us: u32,    // hard ceiling; breach triggers safe-stop
    },
    pub safety_class: SafetyClass,           // SC1 (informational) | SC2 (advisory) | SC3 (assistive) | SC4 (autonomous-actuation)
    pub safety_regulator_binding: Vec<SafetyRegulator>, // e.g. ISO 10218 (industrial robots), ISO 13482 (personal-care robots), per-region road authority for AV
    pub simulation_certification: SimCertRef, // simulation-first; production binding requires sim-passing
    pub safe_stop_handler: SafeStopHandler,   // mandatory; any anomaly triggers
}

pub enum SafetyClass {
    Sc1Informational,    // observe-only; no actuation
    Sc2Advisory,         // recommend to a human operator
    Sc3Assistive,        // act under human supervision
    Sc4Autonomous,       // act without per-action human supervision; T4-disabled by default
}
```

### Real-time loops

Robotics control loops run in dedicated runtime profiles (`oya-intelligence-robotics-control-runtime`) — not on the general Foundry daemon — because the deterministic latency budget cannot share scheduling with capability invocations whose tail latencies are in the seconds. Per-vertical control-plane runtimes coordinate via a sealed control-bus; the bus carries safety messages with a separate priority class.

### Safety-critical actuation T4 disabled by default

Per ADR-0022, capabilities marked `SafetyClass::Sc4Autonomous` cannot reach T4 without an explicit per-tenant per-vertical override that requires:
- Per-vertical safety regulator certification (e.g. ISO 10218 for industrial arms, per-region road authority for AVs).
- Per-region pack override permitting the SafetyClass.
- Founder-and-legal sign-off recorded in the bypass ledger.
- Per-deployment simulation certification (sim-first) that the actuation path passes a regulator-aligned sim suite.

### Per-vertical safety regulator binding

Each robotics capability declares its safety regulator binding in the registry. The CI lane refuses publish if the binding is missing or if the declared SafetyClass exceeds what the regulator binding permits.

### Simulation-first

Every new actuation capability passes a regulator-aligned sim suite before the registry permits production deployment. The sim suite is part of the eval harness (ADR-0024) and has its own adversarial cohort: edge-case environments, sensor failures, communication delays, hostile actors, regulatory boundary cases.

### Per-region anti-scope

Defense and weaponized-robotics work is structurally anti-scope unless the founder + legal carve-out is recorded in a regional pack override with explicit scope:

```rust
// crates/oya-intelligence-robotics-control-kernel/src/anti_scope.rs
pub fn refuse_anti_scope(capability: &Capability) -> Result<(), AntiScopeViolation> {
    if capability.tags.contains(&CapabilityTag::Defense)
        || capability.tags.contains(&CapabilityTag::WeaponizedRobotics)
    {
        if !founder_legal_carveout_recorded(capability) {
            return Err(AntiScopeViolation::DefenseWithoutCarveOut);
        }
    }
    Ok(())
}
```

### CI lanes

- `foundry-vision-lawful-basis` — facial recognition / biometric capabilities require lawful-basis declaration.
- `foundry-speech-locale-coverage` — STT/TTS capabilities cover the minimum locale set (KR + JP + EN; pack-required others).
- `foundry-robotics-control-budget` — every control loop declares a deterministic budget; the runtime profile enforces it.
- `foundry-robotics-safety-regulator-binding` — every robotics capability declares its regulator; binding consistency with SafetyClass enforced.
- `foundry-robotics-simulation-cert` — production deployment refused without a passing sim certificate.
- `foundry-robotics-anti-scope` — defense / weaponized-robotics tags refused without recorded founder-and-legal carve-out.
- `foundry-vision-speech-eval-coverage` — per-locale + per-task eval cohorts mandatory (delegates to ADR-0024).

---

## Consequences

### Positive
- Vision and speech become first-class capability classes on the same registry, autonomy gate, sandbox, and eval substrate as text capabilities.
- Robotics control plane lives in a dedicated runtime profile so determinism is not compromised by general-capability tail latencies.
- Anti-scope is structural, not values-only — an agent cannot quietly drift a capability into defense work.
- Lawful-basis gates on facial recognition and voice biometrics make per-region compliance defensible.
- Simulation-first for actuation prevents production rollout without a regulator-aligned safety case.

### Negative
- Operating real-time control runtimes alongside general-capability runtimes is operationally heavy.
- Per-region facial-recognition and biometric law is a moving target; we must keep pack overrides current.
- Sim-suite authoring per actuation capability is real work; the gate slows initial rollout intentionally.
- Anti-scope structural enforcement may produce false positives in legitimate dual-use cases (e.g. industrial-safety AI vs. defense AI distinctions).

### Operational
- Runbook: `runbooks/foundry-robotics-safe-stop.md` — control-loop budget breach handling, safe-stop verification.
- Runbook: `runbooks/foundry-vision-lawful-basis-incident.md` — what to do when a facial-recognition invocation is challenged.
- Runbook: `runbooks/foundry-robotics-anti-scope-review.md` — handling a tagged capability that requests carve-out.
- On-call: robotics control budget breaches are immediate-page; safety-class drift detection (a capability whose declared class disagrees with its observed behavior) is high-priority.
- Quarterly: per-vertical regulator-binding refresh; per-region anti-scope audit.

---

## Alternatives considered

1. **One sub-substrate per modality (vision, speech, robotics) as separate axes.** Pros: clear ownership. Cons: fractures the registry, eval substrate, autonomy gate; vision/speech/robotics capabilities would need parallel substrates. Rejected — they extend Foundry, they don't replace it.
2. **Robotics on the general-capability runtime.** Pros: less infrastructure. Cons: deterministic-latency budgets cannot share scheduling; actuation safety would be compromised by capability tail latencies. Rejected.
3. **External vision/speech vendors as system of record.** Pros: less to build. Cons: vertical-specific tuning impossible; data-flow and residency concerns; concentration risk. Adopted partially as adapters via the `ProviderAdapter` trait (ADR-0020); rejected as system of record.
4. **Defense / weapons opt-in by tenant request.** Pros: optionality. Cons: anti-scope must be structural; tenant-request as the gate is not strong enough. Rejected — founder + legal carve-out is the only path.
5. **Simulation-optional for actuation capabilities.** Pros: faster iteration. Cons: production rollouts without a regulator-aligned sim case is the failure mode this gate exists to prevent. Rejected.

---

## Open questions

1. The deterministic-latency budgets for robotics — what is the right p99 / max ceiling per SafetyClass, and how do we measure compliance under field conditions? *Owner: `foundry` + `vertical-industrial`.*
2. How do we reconcile the per-region facial-recognition lawful-basis matrix with multi-region tenants whose data flows cross regions? *Owner: `foundry` + `platform-privacy-dub` + `vertical-public-sector`.*
3. The autonomous-vehicle scope is broad — do we restrict to closed-environment AVs (yard tractors, port AGVs) initially, or include public-road? Public-road is an order-of-magnitude harder regulatory and safety story. *Owner: `vertical-industrial` + founder.*
4. Voice cloning / TTS voice profile: how do we detect and refuse a capability that requests cloning of a public figure without consent? *Owner: `foundry` + `ops-security`.*
5. Sim-suite vendoring vs. in-house authoring per vertical — which actuation classes can ride a third-party simulator (e.g. Gazebo / Isaac Sim) vs. need an in-house sim? *Owner: `foundry` + `vertical-industrial`.*

---

## References

- Internal: ADR-0020 (provider trait extension), ADR-0022 (T4-disabled-by-default mechanism), ADR-0023 (sandbox extends to control-runtime profile), ADR-0024 (eval cohorts including sim suites), ADR-0026 (in-house model substrate).
- External: ISO 10218 (industrial robots), ISO 13482 (personal-care robots), ISO 26262 (functional safety for road vehicles), per-region road authority certifications, per-region airspace authorities (drones), per-region facial-recognition law (KR PIPA, GDPR Article 9, US state biometric laws).
