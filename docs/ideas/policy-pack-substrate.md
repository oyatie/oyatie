# Policy Pack Substrate (localization packs on the policy engine)

> Status: refined idea (2026-07-02). Converges ADR-0064, ADR-0206, ADR-0251, ADR-0316.
> Needs its own ADR before the wedge lands.

## Problem Statement

How might we make jurisdiction-specific behavior (regulation, compliance, statutory
rules) a first-class, versioned, composable artifact of the owned policy engine —
authored no-code, dogfooded by us first — instead of hardcoded match arms scattered
across services?

## Grounding (what already exists)

- The idea is partially schematized: `specs/pack-overlay-schema.json` (ADR-0251/0316)
  defines Compliance Pack Overlays with `jurisdiction_scope`, `cedar_policy_overlays`,
  `precedence`, `promotion_gates`, evaluated during request handling.
- The engine is ready: `AuthorizationResponse`
  (`libs/oya-shared-platform-contracts-kernel/src/pdp.rs`) carries typed obligations +
  advice, `decision_id`, and `policy_version` zookie. Obligations/advice modeled in
  `iam/core/policy-cedar-domain/src/obligations.rs`; ReBAC in `.../rebac.rs`.
  PolicyBundles are content-addressed, Ed25519-signed artifacts (`PolicyBundleStore`).
- Localization packs are a named concept: ADR-0064 (canonical base + localization
  packs), ADR-0206 (Fluent/ICU), `docs/localization-packs/kr/pack.yaml`,
  `libs/oya-shared-i18n-kernel`.
- The dogfood case is hardcoded today: `billing/core/accounting-journal/src/lib.rs`
  has `enum Jurisdiction { Korea, UnitedStates, EuropeanUnion }` with match arms;
  KR is the only VAT path (others early-return `Ok(None)`). An embryonic statutory
  rulepack manifest exists in that crate's tests.
- Fragmentation risk: five pack-like concepts drift independently — translation
  packs, compliance overlays, regional packs (`cell/core/regional-pack`), statutory
  rulepacks, PolicyBundles.

## Recommended Direction

**One pack substrate, N pack kinds, two evaluation planes.**

Unify the five pack concepts under a single lifecycle: authoring → schema
validation → signing → semver → distribution → composition/precedence → audit
provenance. Generalize the PDP's `PolicyBundle` mechanics (the most mature of the
five) as that lifecycle.

Evaluation splits by semantics, not by name:

- **Behavioral packs** (statutory, compliance, residency, consent) evaluate on the
  Cedar+ReBAC engine as jurisdiction-scoped overlays whose decisions carry typed
  obligations (`Allow` + `apply_vat{schedule: kr_2026_q3}`).
- **Content packs** (Fluent translations, formats) stay on the i18n resolver.
- The failure semantics force this split: regulated decisions fail closed (PDP
  doctrine), translation misses fail open to the canonical base locale (ADR-0064).
- **Obligations are the bridge**: policy selects the key, the content/compute plane
  resolves the value.

**Computation stays out of the policy language.** Packs carry declarative selection
plus data (rate schedules, thresholds); owned Rust kernels execute the math.
Cedar's non-Turing-completeness keeps packs SMT-analyzable — the safety basis for
no-code authoring and intelligence-authored packs ("this pack never widens access"
proven before promotion via `promotion_gates`).

**Authoring ladder:**
1. Internal compliance/legal (v1, dogfood) — reviewed `pack.yaml` + Cedar overlay
   through the governance pipeline.
2. Tenant admins — needs editor + simulator.
3. Marketplace third parties — Stripe Tax/Avalara model; marketplace substrate exists.
4. Cross-company / B2B2C federation — packs from different trust domains stack
   (platform ≥ jurisdiction ≥ tenant ≥ sub-tenant/consumer, more-restrictive-wins)
   over the ReBAC relationship graph; the pack schema registry doubles as the shared
   ontology that workflow and intelligence integrate against.

## Key Assumptions to Validate

- [ ] Obligation parameters (today `BTreeMap<String,String>`) suffice once given
      per-kind typed schemas — validate by defining the `apply_vat` obligation
      schema in the wedge.
- [ ] Multi-pack precedence has deterministic, explainable semantics (forbid-wins
      today; extend to more-restrictive-wins across obligation conflicts) —
      validate against the j100 multi-pack-conflict journey as test fixtures.
- [ ] An internal compliance author can author a pack via reviewed `pack.yaml` +
      Cedar overlay through the governance pipeline — validate with one real KR
      statutory change before building any editor.

## MVP Scope (wedge: billing statutory rulepacks)

- `PackKind::Statutory` riding the signed PolicyBundle lifecycle.
- KR VAT rulepack: jurisdiction-scoped Cedar overlay + `apply_vat` obligation +
  rate-schedule data tables in the pack payload.
- Billing kernel consumes the obligation (existing `tax_rate_basis_points`
  parameter becomes obligation-fed); delete the `Jurisdiction` enum match arms in
  `billing/core/accounting-journal`.
- Unsupported jurisdiction becomes an explicit fail-closed decision, not a silent
  `Ok(None)`.
- Audit: `decision_id` + pack version provenance stamped on every journal entry.
- Full test ladder: contract tests on the obligation schema, RED/GREEN fixtures for
  pack composition and precedence.

## Not Doing (and Why)

- **Strings through the PDP** — opposite failure semantics + latency; translation
  packs share the lifecycle, never the evaluation path.
- **No-code editor in v1** — v1 authors are internal and go through the governance
  pipeline; the editor ships with the tenant-admin rung, informed by real authoring
  friction.
- **Marketplace + third-party trust machinery** — later rung; needs review/
  attestation product that the internal rungs will specify.
- **Cross-company federation implementation** — design the trust-domain precedence
  now (so port shapes survive cutover), implement after the tenant-admin rung
  proves composition.
- **Expression language in packs** — rejected; forfeits analyzability, which is the
  entire safety basis for no-code and agent authoring.

## Open Questions

- **Owned policy language vs Cedar-as-north-star** (founder raised 2026-07-02):
  Cedar may be benchmark + input dialect only. The owned decision model already
  exceeds vanilla Cedar (obligations/advice, zookies, ReBAC — none are in Cedar
  proper). Candidate shape: owned Policy IR + decision model as the north star;
  Cedar-compatible ingestion for transition; analyzability (decidable, no loops,
  SMT-encodable) preserved as the non-negotiable invariant.
  **Planning path ratified 2026-07-02** (interview session interview_20260702_112146):
  benchmark-first — Core-6 hands-on spike study (Cedar/SpiceDB/OpenFGA/OPA full,
  CEL/Biscuit bounded) as design-input harvest + transient selection, per-gate
  NATIVE/EMULATED/UNSUPPORTED grades under a pre-registered frozen rubric
  (logged amendments), temporal ruled data-plane/harvest-only, fixtures anchored
  on the KR+US statutory workload and a ≥3-hop cross-company ReBAC scenario,
  completed matrix auto-unlocks the owned Policy IR ADR; the billing wedge
  proceeds in parallel with its rulepack format forward-declared as pack-substrate
  v0-draft and Fixture-1 vectors as its acceptance tests.
- Capability home for the pack lifecycle: inside `iam/` (evaluation already lives
  there) vs its own capability — same open question as the authz/ vs iam/ home in
  the authz northstar.
- Should the pack schema registry be a corpus-graph citizen (typed AST) from day
  one, given the ontology/intelligence integration goal?
- Ratification path: needs an ADR converging ADR-0064/0206/0251/0316 before the
  wedge lands.
