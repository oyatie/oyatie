# ADR-MS-002 — Vendor-neutral AI image analysis marketplace

`microservice: imaging`
`status: ACCEPTED`
`date: 2026-05-21`
`wave: 15M-G`
`authority: ADR-0132 + user directive 2026-05-21`

## Context

AI image analysis (CADe / CADx / triage / quantification) is a fast-moving market with ≥50 FDA-cleared / CE-marked vendors in 2026 covering:

- Stroke LVO triage (RapidAI, Viz.ai)
- Pulmonary embolism detection (Aidoc, Viz.ai)
- Brain hemorrhage detection (Aidoc, Qure.ai)
- Lung nodule detection (Annalise.ai, Aidoc, Riverain ClearRead)
- Coronary AI quantification (Cleerly, HeartFlow, Arterys)
- Mammography CAD (ScreenPoint, Lunit, Therapixel, Hologic)
- Bone fracture detection (Annalise.ai, Aidoc, BoneView)
- Cardiac quantification (Arterys, Caption Health, Ultromics)
- Opportunistic body composition + bone density (Zebra Medical, AI-Rad-Companion)
- DBT / tomosynthesis CAD (Hologic, iCAD, ScreenPoint)
- Image denoising / acceleration (Subtle Medical)
- Report-generation NLP (Rad AI, Nuance PowerScribe)

Legacy PACS vendors integrate AI through their own walled gardens:

- GE Edison platform — GE-curated marketplace.
- Philips IntelliSpace AI Workflow Suite — Philips-curated.
- Sectra Amplifier marketplace — Sectra-curated.

These walled gardens slow vendor enrollment, fragment vendor distribution, and introduce per-PACS-vendor integration cost for AI vendors. The result: hospitals end up picking a PACS BEFORE picking AI, even if the AI vendor is the more critical clinical choice.

## Decision

**The imaging µservice exposes a single `AiVendorPort` adapter trait that all AI vendors implement.** No walled-garden marketplace. No PACS-vendor-curated allowlist.

Implementation:

1. `AiVendorPort` trait (Rust) abstracts dispatch / get-result / health / FDA-CE-clearance metadata.
2. Per-vendor adapter crate (`oya-imaging-adapter-ai-vendor-<vendor>`) implements the trait.
3. Tenant configures enabled vendors per (modality, body-part, indication).
4. PHI de-identification per HIPAA Safe Harbor + ISO/TS 25237 is mandatory BEFORE vendor egress (Cedar policy `ai-model-can-read-deidentified.cedar`).
5. FDA / CE / KFDA / PMDA / ANVISA clearance metadata is stored per vendor model version; off-label inference is Cedar-denied.
6. Drift detection per FR-AI-005 monitors PPV / sensitivity / specificity week-over-week.
7. The µservice ships GA with ≥15 third-party vendors (Aidoc, Viz.ai, Cleerly, Rad AI, Annalise.ai, Lunit, Qure.ai, Zebra Medical, Arterys, Caption Health, RapidAI, Subtle Medical, Imagia, Behold.ai, ScreenPoint).

## Consequences

### Positive

- Vendor neutrality is a Stripe-of-AI differentiator vs. GE Edison / Philips ISyntax-AI / Sectra Amplifier.
- Hospitals can pick AI independent of PACS choice.
- New vendor enrollment is a single-adapter-crate operation.
- PHI de-identification is enforceable at the policy layer.

### Negative

- More adapter crates to maintain. Mitigated by adapter-template scaffolding.
- Vendor-API drift requires per-vendor regression suites.
- Drift detection adds telemetry surface area.

### Neutral

- Walled-garden vendors (e.g., GE Edison) cannot be wholly imported, but the µservice can adapt their open APIs as available.

## Alternatives Considered

- **Walled-garden marketplace** (GE Edison pattern). Rejected: locks hospitals out of vendor diversity.
- **Per-tenant custom adapter only** (no µservice-shipped adapters). Rejected: high integration cost for tenants.
- **AI runs in `ai-substrate` µservice; imaging is a thin caller**. Considered for Wave 16+; deferred — imaging owns the de-identification + clinical-finding correlation logic, which sits in the imaging boundary.

## References

- HIPAA Safe Harbor (45 CFR 164.514(b)(2))
- ISO/TS 25237 Pseudonymization in Healthcare
- ADR-0243 (Cedar universal gate)
- ADR-0251 (compliance pack primitive)
- ADR-0255 (intelligence two-layer substrate)
- FDA 510(k) AI/ML-Based Software as a Medical Device guidance (2024)
