---
doc_class: Onboarding
microservice: translate
persona: engineer
date: 2026-05-20
doc_status: published
---

# Engineer onboarding — first 5 working days

Audience: a new engineer on the `translate` µservice team (axis-translate). By Day-5 they will have wired one translation surface, exercised the TM lookup path, exercised the QE inference path, and shadowed an engine-routing-failure incident.

## Day 1 — Domain tour

Read in order:

1. `PRD.md` end-to-end (≤ 35 min). Focus on the engine-routing matrix in §"MT-engine routing + fallback".
2. The five Translate-specific ADRs:
   - `decisions/ADR-TRANSLATE-0001-mt-engine-routing.md` (the routing algorithm).
   - `decisions/ADR-TRANSLATE-0002-tm-leverage-match-scoring.md` (the TM matching algorithm).
   - `decisions/ADR-TRANSLATE-0003-eu-ai-act-quality-estimation-bounds.md` (the EU AI Act compliance constraints).
   - `decisions/ADR-TRANSLATE-0004-data-residency-bound-inference.md` (the sovereign data-residency invariant).
   - `decisions/ADR-TRANSLATE-0005-format-preserving-round-trip.md` (the document format toolchain).
3. The competitor-parity matrix at `competitor-parity-matrix.md` to understand which capabilities we under-index on vs Smartling / Crowdin / Lokalise.

End-of-day: you can answer "given a paid tenant in EU-GDPR sovereign mode requesting an en→fr translation of a 200-char legal text segment, which engine does the router pick and why?" without notes. (Answer: in-pack DeepL Pro EU-DE endpoint because: 1. EU-GDPR mode filters to EU-residency engines; 2. legal content-class prefers DeepL over Google for European legal pairs; 3. tenant_class does not gate model quality, and DeepL still wins for legal class.)

## Day 2 — Stand up local µservice

```sh
make cell-up
make translate-up
```

Make a test translation:

```sh
oya translate text \
    --source "Hello, world." \
    --source-lang en \
    --target-lang fr \
    --tenant my-test-tenant
```

Expected output: `Bonjour le monde.` (or similar; depends on which engine the router selects).

Look at the routing decision in the response:

```sh
oya translate text \
    --source "Hello, world." \
    --source-lang en \
    --target-lang fr \
    --tenant my-test-tenant \
    --explain
```

This adds engine-routing trace to the output. You should see: "selected engine: deepl-eu-de-pro because: pack=us-default→all engines eligible; content_class=generic→preference rank deepl > google > microsoft; tenant_class=demo_trial→usage-capped routing".

## Day 3 — TM seed + lookup

Create a small TM for your test tenant:

```sh
oya translate tm seed \
    --tenant my-test-tenant \
    --source-lang en \
    --target-lang fr \
    --pairs '
"Hello, world.|Bonjour, le monde."
"Welcome to Oyatie.|Bienvenue chez Oyatie."
"This is a test.|Ceci est un test."
'
```

Now request a translation of an exact-match source:

```sh
oya translate text \
    --source "Hello, world." \
    --source-lang en \
    --target-lang fr \
    --tenant my-test-tenant \
    --explain
```

Routing should now show: "TM exact match → return TM result; engine call suppressed; latency = 12 ms (TM-only)".

Try a fuzzy-match:

```sh
oya translate text \
    --source "Hello, my world." \
    --source-lang en \
    --target-lang fr \
    --tenant my-test-tenant \
    --explain
```

Routing should show: "TM fuzzy match score=82% (above 75% threshold but below 99% ICE); return TM result with `match_score: 0.82` annotation; engine call also performed because fuzzy ≤ 99%; engine result returned alongside TM as `alt-translation`".

The fuzzy-vs-exact behavior is the heart of TM leverage. Read `decisions/ADR-TRANSLATE-0002-tm-leverage-match-scoring.md` end-to-end now.

## Day 4 — QE inference

Submit a translation and request QE:

```sh
oya translate text \
    --source "The patient was administered 10 mg of intravenous warfarin." \
    --source-lang en \
    --target-lang es \
    --tenant my-test-tenant \
    --content-class medical \
    --request-qe
```

Output includes:

- `translation: "Al paciente se le administraron 10 mg de warfarina intravenosa."`
- `qe_score: 0.87` (predicted edit-distance score; higher = closer to a human-reviewer's reference).
- `qe_model: comet-kiwi-22`.
- `confidence_band: high` (≥ 0.85).

Notice the routing changed because `--content-class medical` triggered the medical-content-class branch. Read the routing decision in `--explain` mode.

For EU AI Act compliance: when QE is requested for content tagged `medical` or `legal` or `political`, the µservice emits a `qe_decision_made` audit-chain event with the QE model version + score + confidence + EU AI Act decision-trace pointer. Query: `oya audit query --event qe_decision_made --tenant my-test-tenant`.

## Day 5 — Engine-routing-failure incident shadow

Read `runbooks/engine-routing-failure.md` end-to-end first. This is the most-common P2 page in `translate`.

Schedule with axis-translate on-call for a 90-min incident-shadow window. During the window:

1. Trigger a simulated engine outage via the drill harness:
   ```sh
   oya translate drill engine-outage \
       --engine deepl-eu-de-pro \
       --duration 5m
   ```

2. Watch the routing layer detect the outage. Within ≤ 30 s, the engine is moved to `degraded` status; requests in-flight return cached/TM results; new requests route to the secondary (Google Cloud Translation eu-west).

3. Watch the on-call assess: is this engine-wide or per-pair? Is it transient or sustained? The decision tree is in the runbook — follow it.

4. Observe the customer-facing surface: if the outage exceeds 60 s, an incident is auto-opened (P3 for one engine in degraded state; escalates to P2 if 2+ engines simultaneously). The `status.oyatie.io` page is updated.

5. Observe the recovery: when the engine returns, the routing layer probes it for 60 s in shadow mode before re-eligibility. If shadow results match the secondary's results within tolerance, eligibility restored.

End-of-week: you can sketch the routing path including TM/QE/fallback branches from memory, and you understand why we shadow-probe before re-eligibility (premature re-eligibility caused the 2025-Q3 P1 because DeepL's outage came back with corrupted state).
