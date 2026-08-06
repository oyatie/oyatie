---
id: ADR-0048
status: Accepted
doc_status: published
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Korean morphology / multilingual tokenization

# ADR-0048: Korean morphology + multilingual tokenization — `Tokenizer` trait per language family, mecab-ko + khaiii FFI day-1, in-house Rust port long-horizon

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-search`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0013, ADR-0030, ADR-0033, ADR-0047

---

## Context

Korean is morphologically rich: a single eojeol can carry stem + tense + politeness + connective in one orthographic word. Generic Unicode tokenization (whitespace + ICU) destroys retrievability for Korean queries. The KR launch requires first-class Korean tokenization at the search microservice (per ADR-0030), at the Workspace search-within-Drive surface (per ADR-0029), at the Vertical-pack record search, and at the Foundry agent retrieval.

Two mature open-source KR morphology engines exist: **mecab-ko** (LGPL; the canonical KR adaptation of the Japanese MeCab analyzer) and **khaiii** (Apache-2; Kakao's KR-specific deep-learning-based analyzer). Both ship as C/C++ libraries; both require FFI binding from Rust. License posture differs: khaiii is Apache-2 (clean), mecab-ko is LGPL (legal isolation per License Policy ADR).

The pack-of-19 foundation ADRs named KR morphology as a launch requirement but did not pin the trait surface, the per-pack tokenizer impl, the FFI day-1 vs in-house long-horizon trajectory, or the multi-locale extension (KR / JP / ZH / EN / Indic / Arabic).

---

## Decision

We adopt a **`Tokenizer` trait per language family** under `crates/oya-search-tokenizer-*`; **mecab-ko + khaiii via FFI day-1** (with mecab-ko legal-isolation analysis per License Policy + Apache-2 khaiii as the cleaner option for tenants who can use it); **in-house Rust port** of the KR morphology engine long-horizon; **per-pack tokenizer impl** for JP / ZH / EN / Indic / Arabic.

### `Tokenizer` trait

```rust
// crates/oya-search-tokenizer-kernel
pub trait Tokenizer {
    fn locale(&self) -> LocaleId;
    fn tokenize(&self, text: &str) -> Result<Vec<Token>>;
    fn pos_tag(&self, tokens: &[Token]) -> Result<Vec<PosTaggedToken>>;
    fn lemmatize(&self, tokens: &[Token]) -> Result<Vec<LemmatizedToken>>;
    fn ner(&self, tokens: &[PosTaggedToken]) -> Result<Vec<NerEntity>>;
}

pub struct Token {
    pub surface: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub kind: TokenKind,
}

pub enum TokenKind {
    Word,        // generic word
    Particle,    // 조사 (KR), 助詞 (JP), etc.
    Affix,       // 어미 (KR), prefix/suffix (EN)
    Number,
    Punctuation,
    Whitespace,
    Symbol,
    Compound,    // CJK compound nouns
}
```

### Per-language-family implementations

| Family | Locale codes | Engine (day-1) | Engine (long-horizon) |
|---|---|---|---|
| Korean | `ko-KR`, `ko-KP` | mecab-ko (LGPL) + khaiii (Apache-2) via FFI | in-house Rust port (`crates/oya-search-tokenizer-ko-rs`) |
| Japanese | `ja-JP` | MeCab-ja (BSD-style) + IPADic via FFI | in-house Rust port (long-horizon) |
| Chinese (Simplified) | `zh-CN`, `zh-SG` | jieba-rs (MIT; Rust-native port) | jieba-rs upgraded |
| Chinese (Traditional) | `zh-TW`, `zh-HK` | HanLP (Apache-2) via JNI bridge — alternative: jieba-rs with TW dict | in-house Rust port |
| English | `en-US`, `en-GB`, `en-*` | NLTK-equivalent + Snowball stemmer (BSD; Rust-native) | refined |
| Indic | `hi-IN`, `bn-IN`, `ta-IN`, `te-IN`, `mr-IN`, `gu-IN`, `pa-IN`, `kn-IN`, `ml-IN`, `or-IN`, `as-IN` | IndicNLP (MIT) via FFI | in-house Rust port (long-horizon) |
| European (general) | `de-*`, `fr-*`, `es-*`, `it-*`, `pt-*`, `nl-*`, `sv-*`, `no-*`, `da-*`, `fi-*`, `pl-*`, `cs-*`, `hu-*` | Stanza (Apache-2) via Python FFI; Snowball for fallback | in-house Rust |
| Arabic | `ar-*` | Stanza-Arabic (Apache-2) + Farasa (LGPL — legal isolation) | in-house Rust port |
| Vietnamese | `vi-VN` | underthesea (Apache-2) | in-house Rust |
| Thai | `th-TH` | PyThaiNLP (Apache-2) via FFI | in-house Rust |

### Korean: day-1 mecab-ko + khaiii FFI

```rust
// crates/oya-search-tokenizer-ko-mecab
pub struct MecabKoTokenizer {
    pub tagger: mecab::Tagger,    // FFI to libmecab + mecab-ko-dic
    pub dictionary_path: PathBuf,
}

impl Tokenizer for MecabKoTokenizer { /* impl */ }

// crates/oya-search-tokenizer-ko-khaiii
pub struct KhaiiiTokenizer {
    pub khaiii: khaiii_ffi::Khaiii,    // FFI to libkhaiii
    pub model_path: PathBuf,
}

impl Tokenizer for KhaiiiTokenizer { /* impl */ }
```

- Per-tenant configuration: choose mecab-ko or khaiii (default mecab-ko for retrieval; khaiii for NER-heavy workloads).
- Per-tenant dictionary overlay: tenant-specific compound nouns appended to the dict.

### Korean: legal isolation per License Policy

mecab-ko is LGPL-2.1+. Per License Policy ADR + per FSF guidance on LGPL dynamic linking:

- mecab-ko is loaded as a dynamic library (`.so` / `.dylib` / `.dll`); not statically linked into our product binaries.
- The FFI shim (`crates/oya-search-tokenizer-ko-mecab`) is the boundary; no LGPL code is inlined.
- Per-cell deployment includes mecab-ko as a separate library; per-cell legal-isolation evidence record.
- Documented in `docs/legal/mecab-ko-legal-isolation.md`.

khaiii is Apache-2 — no isolation needed.

### Korean: in-house Rust port (long-horizon)

`crates/oya-search-tokenizer-ko-rs` long-horizon target:

- Pure-Rust implementation of mecab-class viterbi morphological analyzer.
- mecab-ko-dic compiled to Rust-native FST format.
- Per-pack dictionary maintained in-house.
- License: Apache-2 outbound (the in-house code; dictionaries we may need to license-clean separately).
- Long-horizon target: GA at W+24+ (Phase 2-aligned).

### Per-pack tokenizer impl

Each regional pack (per regional-pack architecture) declares:

- Default tokenizer for its locale(s).
- Tenant-overridable tokenizer choice.
- Per-pack dictionary overlay (e.g. KR-pack ships dict for 정부 / 의료 / 금융 domain terms).

### Tokenizer dispatch in Search

Per ADR-0030 parser stage, the per-document tokenizer is selected by detected document locale:

1. Detect document locale (CLD3-class library; first 1KB heuristic).
2. Look up per-locale tokenizer from registry.
3. Tokenize → POS tag → lemmatize → NER → emit to indexer.

Fallback: ICU word-break for unrecognized locales.

### Tokenizer dispatch elsewhere

Workspace Drive search, Vertical-pack record search, Foundry agent retrieval — all consume the same `Tokenizer` trait via `crates/oya-search-tokenizer-kernel`; no axis ships its own tokenizer.

### Per-tenant tokenizer configuration

Tenant admin can:

- Override per-locale engine (e.g. khaiii for KR).
- Add per-tenant dictionary entries (compound nouns specific to tenant domain).
- Disable specific token classes (e.g. don't index particles).

### Anti-scope

This ADR does not own the search architecture (per ADR-0030, consumes tokenizer). Does not own the search backend (per ADR-0047). Does not own embedding models (per ADR-0026 / ADR-0046).

---

## Consequences

### Positive

- Single `Tokenizer` trait across all axes — no per-microservice tokenizer drift.
- Day-1 KR launch capability via mecab-ko + khaiii FFI.
- Apache-2 khaiii path lets tenants who can't use LGPL-isolated dependencies still get top-quality KR morphology.
- In-house Rust port long-horizon eliminates dep risk + reduces FFI overhead.
- Per-pack tokenizer impl framework supports onboarding new locales without architectural change.

### Negative

- mecab-ko legal-isolation evidence is recurring legal cost.
- FFI shims to C++ libraries have build-time + runtime overhead.
- In-house Rust port is multi-year investment.
- Per-pack dictionary maintenance is per-pack ongoing cost.

### Operational

- Per-cell tokenizer health (FFI library load + dict load + per-call latency P95) monitored per ADR-0042.
- Per-tenant tokenizer config audit-chained per ADR-0003.
- Per-quarter dictionary refresh per pack.
- Per-quarter LGPL legal-isolation evidence review.
- Per-locale relevance benchmark per pack.

---

## Alternatives considered

### Alternative A — Generic Unicode tokenization (ICU only)

- **Pros:** zero per-locale work.
- **Cons:** unusable for Korean; KR queries fail.
- **Rejected because:** KR launch fails immediately.

### Alternative B — One engine for all KR (mecab-ko only)

- **Pros:** simpler.
- **Cons:** LGPL exposure for tenants that cannot tolerate it; we ship khaiii for those tenants.
- **Rejected because:** the Apache-2 khaiii option is real.

### Alternative C — In-house from day 1 (no FFI)

- **Pros:** no LGPL exposure, no FFI overhead.
- **Cons:** delays KR launch by 12-18 months while we re-implement mecab-class viterbi + dict from scratch.
- **Rejected because:** day-1 capability beats clean from day 1.

### Alternative D — Per-axis tokenizer choice

- **Pros:** axis flexibility.
- **Cons:** drift; per-microservice dict maintenance; cross-microservice search hits inconsistent results.
- **Rejected because:** tokenizer is a substrate concern.

---

## Open questions

1. **Q1.** Day-1 KR default — mecab-ko or khaiii? Default: mecab-ko (better recall for retrieval); khaiii for tenants opting out of LGPL exposure. → ADR-0013.
2. **Q2.** Per-pack dictionary contribution model — internal team or community? Default: internal at GA; community at W+18. → owner: `axis-search`.
3. **Q3.** In-house KR port GA target — W+18 or W+24? Default: W+24 conservative. → owner: `axis-search`.
4. **Q4.** Day-1 multi-locale coverage — KR + EN + JP + ZH at GA, or KR + EN only? Default: KR + EN + JP + ZH at GA; others per pack onboarding. → ADR-0033.
5. **Q5.** Stanza FFI for European locales — Python FFI is heavy; reconsider in-house Rust earlier? Default: Python FFI at GA; in-house Rust at W+18 for Stanza-equivalent. → owner: `axis-search`.

---

## References

- `docs/PRD.md` §10 (multi-locale)
- `docs/DESIGN.md` §11 (tokenization), §10 (cross-microservice contracts)
- mecab-ko + mecab-ko-dic project; khaiii (Kakao) project; Stanza (Stanford NLP); IndicNLP; PyThaiNLP
- FSF guidance on LGPL dynamic linking; OSI license review board
- ADR-0001 (cohesion), ADR-0013 (License Policy), ADR-0030 (search), ADR-0033 (vertical pack), ADR-0047 (search backend)
