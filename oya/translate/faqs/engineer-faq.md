---
doc_class: FAQ
microservice: translate
persona: engineer
date: 2026-05-20
doc_status: published
---

# Engineer FAQ — translate

## Why are we routing across 8 external MT vendors instead of picking one?

Per ADR-TRANSLATE-0001: no single MT vendor is best across all language-pairs, content-classes, and residency packs. DeepL is strong on European languages but weak on Asian; Google is broadest but middle-quality; Microsoft Translator has the best en↔ar and en↔he; Naver Papago is the only viable engine for KR-PIPA sovereign-residency pairs. Routing the right engine per request is the differentiator vs Crowdin / Lokalise which let users pick a single vendor per project. Our router transparently selects per-segment.

## What's the difference between an "exact match", an "ICE", and a "fuzzy" TM match?

Per ADR-TRANSLATE-0002:

- **Exact (100%)**: source segment is byte-identical to a TM entry. Returns the TM target; engine call suppressed.
- **ICE (In-Context Exact)**: source segment is byte-identical AND the surrounding context (previous segment + next segment in the document) is also byte-identical. Higher confidence than plain exact; some TMs treat ICE as "perfect leverage". Returns the TM target.
- **Fuzzy (75-99%)**: source segment has > 75% MinHash-LSH similarity to a TM entry. Returns the TM target with a `match_score` annotation; engine result also computed and returned as `alt-translation` so the reviewer can choose.

The 75% threshold is configurable per-tenant via `workspace.config.tm_fuzzy_threshold ∈ [50, 99]`. Lower thresholds increase TM leverage but increase risk of inappropriate matches.

## How does sentence-piece chunking work for caption streaming?

Captions arrive as a stream from speech-to-text. We can't translate "Hello," then "world." separately and concatenate — the target language might require word reordering. Instead: chunk on punctuation boundaries (full-stop, question-mark, exclamation-mark) OR on time-based silence threshold (≥ 400 ms). Each chunk is translated as a unit. If a subsequent chunk reveals that an earlier translation was wrong (e.g., "Hello,| world is round." would be better as "Bonjour, le monde est rond." not "Bonjour, monde rond."), we emit a correction-replay event per ADR-TRANSLATE-0006 — the player overwrites the visible caption. UI must support correction-replay; if not, the user sees flicker.

## Why does my QE score look low for a translation that looks fine?

Two common causes. (1) The QE model is trained on European-language pairs primarily; for Asian or African languages the score has higher variance. Read `decisions/ADR-TRANSLATE-0003 §"Per-pair QE confidence calibration"` for the per-pair calibration matrix. (2) The QE model is a "no-reference" estimator (COMET-Kiwi) — it predicts edit-distance without seeing a human reference. If the source is unusual (e.g., dense technical jargon, mixed-language code-switching), the QE model is less confident. The `confidence_band` field tells you whether to trust the score.

## A tenant's translation went through Google instead of DeepL — they're complaining. Why?

Routing per ADR-TRANSLATE-0001 considers (in order): pack residency → content class → quality profile → cost → fallback. The most-common surprise: the tenant's pack-residency mode (let's say IN-DPDPA) restricts engine eligibility — DeepL doesn't have an IN-resident endpoint so it's excluded; Google has an IN-resident endpoint (asia-south1) so it's chosen. The complaint surfaces in the tenant's UI without explaining the routing; we should be more transparent about this — bug ticket `translate-0312` is open. Workaround: explain the routing via `--explain` mode in the API response.

## How do I add a new external MT vendor?

Three steps:

1. Implement the `MTVendor` trait in `crates/oya-translate-engines/src/vendors/<name>.rs`. Required methods: `translate_text`, `translate_document`, `supported_pairs`, `endpoint_regions`, `cost_per_1k_chars`.
2. Add the vendor to the routing matrix in `crates/oya-translate-routing/src/matrix.yaml` with its (pair × content-class) quality scores. Quality scores are sourced from `benchmarks/per-vendor-per-pair-quality.json` which is re-baselined quarterly.
3. Open an ADR `ADR-TRANSLATE-NNNN-add-<name>-vendor.md` documenting the rationale.

The vendor goes live in shadow mode for 14 days (routing emits the would-be result but doesn't return it to the tenant); after green shadow review, the vendor goes live in production.

## What's the difference between `translate` and the `intelligence` µservice?

`translate` is specifically the MT + TMS substrate. It does machine-translation, TM, termbase, QE, document translation, and caption streaming.

`intelligence` is the broader LLM / AI inference substrate. It does chat, summarisation, classification, embedding, completion, and other LLM tasks.

The overlap: frontier LLMs can do translation. Per ADR-TRANSLATE-0001 §"LLM routing", we can route a translation request to an LLM via `intelligence` when the content class is "creative" (marketing copy, slogans, narrative prose) — these often translate better with an LLM that understands rhetorical intent. But the LLM call goes through `intelligence`, and the result is normalised back through `translate`'s response shape so callers don't need to know.

## Why is in-pack inference so restrictive for paid tenant_class compliance packs?

KR PIPA Art. 28 + EU GDPR Art. 44-50 + CN PIPL Art. 38-43 + IN DPDPA §16 all require that personal data not cross jurisdiction during inference. For translate, the "inference" is the model run; the input is potentially-PII text. So a Korean tenant translating Korean medical records cannot have the inference run on a non-Korea-resident endpoint, even if that endpoint is the highest-quality vendor.

The practical impact: a KR-PIPA tenant's en↔ko translation routes to one of: Naver Papago (KR-native), Microsoft Translator KR region, or oyatie's in-pack model. DeepL has no KR endpoint at time of writing; Amazon Translate's ap-northeast-2 region is acceptable; Google Cloud Translation's asia-northeast3 is acceptable. The router enumerates legal engines first, then applies the content-class quality ranking among only the legal set.

## Bulk-translate of a 50 k-segment XLIFF is slow. How do I speed it up?

By design, bulk-translate is queue-bounded — we don't run all 50 k segments in parallel because that would exhaust the per-tenant rate-limit AND hit vendor-side rate-limits. The current concurrency is 16 segments simultaneously per tenant. The paid tenant_class SLO is 60 s for a 10 k-segment XLIFF (~ 167 seg/s); at 50 k that's ~ 5 min.

To speed up: paid tenants can contract for 64-way per-tenant concurrency with priority queuing. Otherwise, two options: (1) split the XLIFF into 5 × 10 k-segment files and submit in parallel from the client side (each bulk-translate gets its own queue position). (2) Pre-warm the TM by feeding the XLIFF source through TM-lookup first; many segments will TM-match exactly and skip engine routing.

## Where do TM and termbase contradict each other?

Termbase is authoritative for the terms it covers. If TM says "agile = ágil" but termbase says "agile = adaptable" (because the tenant's brand glossary translates "agile" as "adaptable" in their internal jargon), termbase wins. The router emits a `termbase_override_applied` audit event so the translator sees both the TM suggestion and the termbase override. Disagreements are resolvable per workflow-engine review.
