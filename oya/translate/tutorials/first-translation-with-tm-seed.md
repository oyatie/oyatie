---
doc_class: Tutorial
microservice: translate
persona: engineer
date: 2026-05-20
doc_status: published
---

# Tutorial — First translation with TM seed in 20 minutes

You will: stand up a `translate` workspace, seed a small TM, translate a segment, observe the engine routing, and verify QE — all in 20 minutes.

## Pre-requisites

- `make cell-up` running.
- `make translate-up` running.
- `oya-dev-cli` ≥ 1.42.0.

## Step 1 — Create your tenant + workspace

```sh
oya translate tenant create \
    --tenant-id my-tutorial-tenant \
    --tenant-class demo_trial \
    --pack-residency us-default
```

The pack-residency choice affects engine eligibility. `us-default` is the broadest (8 vendors). Choose `kr-pipa`, `eu-gdpr`, `cn-pipl`, `in-dpdpa` for sovereign-residency scenarios.

Tenant class `demo_trial` enables the same quality surface under trial usage caps. For this tutorial, convert to `paid` so TM, termbase, and QE usage are billed through the paid billing components:

```sh
oya translate tenant set-class --tenant-id my-tutorial-tenant --tenant-class paid --billing-components per_seat,per_usage
```

## Step 2 — Seed a small TM (≤ 3 min)

Create a TMX file `tutorial-tm.tmx`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<tmx version="1.4">
  <header creationtool="oya-cli" creationtoolversion="1.42" segtype="sentence" o-tmf="tutorial"
          adminlang="en" srclang="en" datatype="plaintext"/>
  <body>
    <tu>
      <tuv xml:lang="en"><seg>Welcome to Oyatie.</seg></tuv>
      <tuv xml:lang="fr"><seg>Bienvenue chez Oyatie.</seg></tuv>
    </tu>
    <tu>
      <tuv xml:lang="en"><seg>Our platform helps you manage tasks efficiently.</seg></tuv>
      <tuv xml:lang="fr"><seg>Notre plateforme vous aide à gérer vos tâches efficacement.</seg></tuv>
    </tu>
    <tu>
      <tuv xml:lang="en"><seg>The patient was administered 10 mg of intravenous warfarin.</seg></tuv>
      <tuv xml:lang="fr"><seg>On a administré au patient 10 mg de warfarine intraveineuse.</seg></tuv>
    </tu>
  </body>
</tmx>
```

Import:

```sh
oya translate tm import \
    --tenant-id my-tutorial-tenant \
    --source-lang en \
    --target-lang fr \
    --file tutorial-tm.tmx
```

Output: `Imported 3 TM entries for (en, fr) pair`.

## Step 3 — Exact-match translation (≤ 2 min)

```sh
oya translate text \
    --tenant-id my-tutorial-tenant \
    --source-lang en \
    --target-lang fr \
    --source "Welcome to Oyatie." \
    --explain
```

Expected output:

```yaml
translation: "Bienvenue chez Oyatie."
source: TM
match_score: 1.00
match_type: EXACT
engine_call_suppressed: true
latency_ms: 9
```

The translate latency is 9 ms because the TM is in-memory; no engine call needed.

## Step 4 — Fuzzy-match translation (≤ 2 min)

```sh
oya translate text \
    --tenant-id my-tutorial-tenant \
    --source-lang en \
    --target-lang fr \
    --source "Welcome to oyatie translation platform." \
    --explain
```

Expected output:

```yaml
translation: "Bienvenue chez oyatie translation platform."   # from engine
alt_translation: "Bienvenue chez Oyatie."  # from TM fuzzy
match_score: 0.82
match_type: FUZZY
engine_used: deepl-pro
latency_ms: 187
```

Both TM and engine results returned. The UI lets the translator pick. `match_score: 0.82` means 82% MinHash-LSH similarity to the TM entry.

## Step 5 — Termbase override

Add a termbase entry:

```sh
oya translate termbase add \
    --tenant-id my-tutorial-tenant \
    --source-lang en \
    --target-lang fr \
    --term "Oyatie" \
    --translation "Oyatie"  # explicitly preserve casing/branding
```

This locks "Oyatie" → "Oyatie" (the brand name doesn't translate). Now request:

```sh
oya translate text \
    --tenant-id my-tutorial-tenant \
    --source-lang en \
    --target-lang fr \
    --source "Oyatie is your platform."
```

Expected output: `Oyatie est votre plateforme.` — note the brand name is preserved. The audit-chain emits `termbase_override_applied`.

## Step 6 — Quality Estimation (≤ 3 min)

```sh
oya translate text \
    --tenant-id my-tutorial-tenant \
    --source-lang en \
    --target-lang fr \
    --source "The patient was administered 10 mg of intravenous warfarin." \
    --content-class medical \
    --request-qe \
    --explain
```

Expected output:

```yaml
translation: "On a administré au patient 10 mg de warfarine intraveineuse."  # exact match from TM
source: TM
match_score: 1.00
match_type: EXACT
qe_score: 0.95
qe_model: comet-kiwi-22
confidence_band: high
eu_ai_act_decision_trace_id: ai-act-trace-1234567890
```

Notice: even with an exact-match TM result, QE is still computed when `--request-qe` is set. The QE score corroborates the TM result.

Try a non-TM-match medical content:

```sh
oya translate text \
    --tenant-id my-tutorial-tenant \
    --source-lang en \
    --target-lang fr \
    --source "The dosage of metformin was increased to 1000 mg twice daily." \
    --content-class medical \
    --request-qe
```

Expected output:

```yaml
translation: "La dose de metformine a été augmentée à 1000 mg deux fois par jour."
source: ENGINE
engine_used: deepl-pro
qe_score: 0.91
qe_model: comet-kiwi-22
confidence_band: high
eu_ai_act_decision_trace_id: ai-act-trace-1234567891
```

QE = 0.91 means high confidence; below 0.7 would trigger automatic human-review escalation for paid tenant_class workflows.

## Step 7 — Audit chain inspection

```sh
oya audit query \
    --tenant-id my-tutorial-tenant \
    --since 30m \
    --event-class translate_*
```

You should see 6-7 events:

- `tm_imported` (1)
- `termbase_term_added` (1)
- `translation_requested` (3-4)
- `tm_exact_match_applied` (2)
- `engine_translation_executed` (2)
- `termbase_override_applied` (1)
- `qe_decision_made` (2)

Each event is Ed25519-signed against the `oyatie.translate.runtime` key.

## Step 8 — Cleanup

```sh
oya translate tenant delete --tenant-id my-tutorial-tenant --confirm-i-mean-it
```

This deletes the tenant including the TM and termbase. The audit-chain rows are retained for ≥ 7 years per the retention policy in `compliance.md`.

## What you've learned

- TM exact / ICE / fuzzy match semantics.
- Engine routing and the `--explain` mode.
- Termbase override of TM results.
- QE inference and the `eu_ai_act_decision_trace_id` audit linkage.
- The full audit-chain event taxonomy for a translation flow.

Next tutorial: `tutorials/document-translation-xliff-round-trip.md` — translates a 10 k-segment XLIFF preserving format fidelity end-to-end.
