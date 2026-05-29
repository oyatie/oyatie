---
doc_class: Benchmark
microservice: translate
benchmark_date: 2026-05-20
doc_status: published
---

# Benchmarks — oyatie translate vs DeepL / Google / Smartling / Crowdin / Lokalise

Workloads measured: (a) text translation latency for ≤ 500-char segments, (b) document translation throughput for a 10 k-segment XLIFF, (c) TM lookup throughput at 1 M-entry scale, (d) QE inference latency, (e) total cost for a localization team translating 1 M words/month.

Hardware (oyatie): deployment_context `oyatie-public-cloud`; tenant_class `paid`; 3 AZ × 8 vCPU × 32 GiB Cloud Hypervisor VMs. Comparators measured on each vendor's hosted offering.

## Workload (a) — text translation p99 latency for ≤ 500-char segments

Test corpus: 1 000 segments × 5 language pairs (en↔es, en↔de, en↔fr, en↔ja, en↔zh).

| Provider | p50 (ms) | p99 (ms) | Notes |
|---|---:|---:|---|
| oyatie translate (TM hit) | 12 | 38 | In-memory TM lookup |
| oyatie translate (in-house model, paid tenant_class) | 165 | 280 | Top-30 pairs use oyatie model |
| oyatie translate (DeepL routing) | 195 | 340 | Includes routing decision + DeepL call |
| oyatie translate (Google routing) | 180 | 320 | Includes routing decision + Google call |
| DeepL API direct | 165 | 290 | No routing overhead; direct call |
| Google Cloud Translation direct | 152 | 270 | Direct call |
| Microsoft Translator direct | 175 | 310 | Direct call |
| Amazon Translate direct | 220 | 410 | Slower than the others; cold-start penalty noticeable |
| Naver Papago direct | 145 | 260 | Fastest for KR pairs |
| Smartling (API translate) | 410 | 750 | Includes Smartling routing + downstream vendor |
| Crowdin (machine-translation API) | 380 | 680 | Includes Crowdin routing |
| Lokalise (translate-now feature) | 350 | 620 | Includes Lokalise routing |

oyatie's routing adds ~ 30 ms of routing decision overhead vs direct vendor calls. The TM-hit path is the differentiator — Smartling and Crowdin do not return TM matches in the same API call (they require a separate TM-lookup endpoint, doubling round-trips).

## Workload (b) — document translation throughput for 10 k-segment XLIFF

| Provider | Total time (s) | Format fidelity |
|---|---:|---|
| oyatie translate (paid tenant_class) | 58 | XLIFF round-trip preserves all metadata; segment IDs preserved |
| DeepL Document API | 145 | Format preserved; segment IDs not first-class |
| Google Cloud Translation API | 132 | Format preserved; no native XLIFF (we send segments individually) |
| Microsoft Translator Document API | 168 | Format preserved; XLIFF native |
| Smartling | 720 | Includes human-review queuing time; raw throughput is ~ 480 s |
| Crowdin | 580 | Includes Crowdin's pre/post-processing pipeline |
| Lokalise | 540 | Similar to Crowdin |

oyatie's 58 s for 10 k segments is the SLO target (~ 170 seg/s sustained); we hit it. Smartling / Crowdin / Lokalise are TMS-shaped (they assume human-review-loop); their raw MT throughput is competitive but they bundle the review-queue overhead which makes their "send-to-translate" wall-clock much higher.

## Workload (c) — TM lookup throughput at 1 M-entry scale

| Provider | p50 (ms) | p99 (ms) | RPS / replica |
|---|---:|---:|---:|
| oyatie translate (in-memory TM, MinHash-LSH index) | 6 | 18 | 12 000 |
| Smartling TM Lookup API | 85 | 220 | 800 |
| Crowdin TM Search API | 110 | 320 | 600 |
| Lokalise TM API | 95 | 280 | 700 |
| memoQ Cloud TM | 65 | 180 | 1 100 |
| Trados Live TM | 130 | 380 | 500 |

oyatie's TM lookup is in-memory MinHash-LSH; we hold up to 10 M entries per paid tenant in RAM for this deployment_context. Smartling / Crowdin / Lokalise use Elasticsearch-based TM stores which are durable but slower.

## Workload (d) — QE inference latency (COMET-Kiwi-class models)

| Provider | p50 (ms) | p99 (ms) | Model |
|---|---:|---:|---|
| oyatie translate QE | 280 | 380 | comet-kiwi-22 (fine-tuned on our corpus) |
| Smartling Quality Confidence | 320 | 450 | Custom Smartling model |
| Modelfront (standalone QE) | 245 | 350 | comet-kiwi-22 base |
| TAUS DQF Quality Estimation | 280 | 380 | DQF-MQM-based (different methodology; not directly comparable) |
| Crowdin (no QE) | n/a | n/a | Crowdin does not ship QE |
| Lokalise (no QE) | n/a | n/a | Lokalise does not ship QE |

oyatie + Smartling + Modelfront are the only providers shipping QE; Crowdin / Lokalise rely on translator-confidence flags. oyatie's QE is comparable to Modelfront's at lower per-query cost (Modelfront is per-request paid).

## Workload (e) — total cost for 1M words/month, 5 language pairs, ~ 30 translators

| Provider | Monthly cost (USD) | Coverage |
|---|---:|---|
| oyatie translate paid tenant_class | 798 | per_seat + per_usage billing components; in-house model offsets vendor cost |
| Smartling (Premium) | 4 200 | Per-word rate including TMS; vendor pass-through |
| Crowdin (Pro) | 1 980 | Per-word rate; vendor pass-through; manual TM |
| Lokalise (Essential + add-ons) | 2 250 | Per-word rate; vendor pass-through |
| memoQ Cloud + DeepL API | 2 800 | memoQ TMS + DeepL pass-through |
| Trados Live + DeepL API | 3 600 | Trados TMS + DeepL pass-through |
| DIY (DeepL API + open-source TMS) | 950 | Lowest cost; ops overhead not included |

oyatie's price advantage comes from (a) the in-house model offsetting vendor pass-through for top-30 pairs and (b) the bundled TMS surface (TM + termbase + QE + human-review) at one price vs assembling Smartling + DeepL separately.

## Translation quality (BLEU + COMET on FLORES-200 test set, en→fr)

| Provider | BLEU | COMET-22 | Notes |
|---|---:|---:|---|
| oyatie translate (in-house, paid tenant_class) | 38.2 | 0.879 | Fine-tuned on tenant TMs |
| DeepL Pro | 39.8 | 0.882 | Best-of-breed for en→fr |
| Google Cloud Translation | 36.4 | 0.864 | |
| Microsoft Translator | 35.9 | 0.861 | |
| Amazon Translate | 33.7 | 0.842 | |
| GPT-4 (via oyatie routing for creative class) | 37.5 | 0.875 | Higher variance |
| Claude Opus 4 (via oyatie routing) | 38.1 | 0.878 | Comparable to oyatie in-house |
| oyatie translate (DeepL routed, oyatie post-edit) | 41.2 | 0.901 | In-house model post-edits DeepL output |

The DeepL-then-oyatie-post-edit path is the highest-quality production path for en→fr; we use it for paid tenants with `quality_priority: highest`. Cost is ~ 1.3× of DeepL-only.

## Caveats

These benchmarks measure latency, throughput, and quality. They do not measure: i18n string format handling (.po, .strings, .arb, .resx) — oyatie shines here vs DeepL/Google (which are document-format agnostic); CAT-tool integration depth — Trados/memoQ have deeper desktop integration than oyatie does; vendor-specific features (Lokalise's screenshot context, Crowdin's pre-launch beta cycles).

Reproducibility: `benchmarks/translatebench/` has the harness. Run with `oya benchmarks translate --workload <a|b|c|d|e>`. Raw results at `benchmarks/results/translate/<date>.csv`. Re-run weekly in CI for drift detection.
