//! Cost tracking — pure-kernel spend accounting for the cloud-intelligence
//! subscription pool (feeds the D6 EventSink / ClickHouse spend sink).
//!
//! Three pure capabilities, zero I/O:
//!
//! - [`PriceBook`] — policy-as-data [`RateCard`] table keyed by provider and
//!   canonical upstream model id. Rates are loaded from a typed table
//!   ([`PriceBookFile`]) via the pure [`PriceBook::from_file`] constructor.
//!
//! Per the kernel's hexagonal layering, JSON *parsing* lives in the adapter/test
//! layer (which owns `serde_json`): deserialize a FinOps/Billing-owned table
//! into a [`PriceBookFile`], then hand it to [`PriceBook::from_file`]. The
//! kernel lib itself takes no `serde_json` dependency and does not select the
//! canonical rate version — it only validates typed rate data and applies pure
//! math.
//! - [`UsageExtractor`] — normalizes the three provider usage shapes (Anthropic
//!   Messages, OpenAI Responses, Gemini `usageMetadata`) into one canonical
//!   [`TokenUsage`] with five disjoint billable token classes.
//! - [`cost_of`] — pure fn mapping `(TokenUsage, RateCard)` to a [`CostRecord`].
//!
//! ## Money discipline
//!
//! No floating point anywhere on the money path. Rates are authored as integer
//! `nanos_usd_per_mtok` (nanoUSD per 1,000,000 tokens). Computed cost is
//! reported in **picoUSD** (`i128`, 1e-12 USD) so a per-token cost is exact for
//! every rate quoted to `$1e-6/MTok` granularity or coarser (every published
//! provider price). The adapter SUMs picoUSD into the spend sink and renders a
//! decimal-USD string only at the display edge.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ponytail: JSON parsing stays in the adapter/test layer (which owns
// serde_json). The kernel lib keeps only `serde` derives + pure logic, matching
// the established xproxy_parity convention and the no-new-lib-dep rule.

/// Number of picoUSD (1e-12 USD) in one nanoUSD (1e-9 USD).
const PICOS_PER_NANO: i128 = 1_000;
/// Tokens per "megatoken" — the unit rates are quoted in.
const TOKENS_PER_MTOK: i128 = 1_000_000;

// ---------------------------------------------------------------------------
// Canonical token usage (provider-normalized)
// ---------------------------------------------------------------------------

/// Provider-normalized token usage with five **disjoint** billable classes.
///
/// Normalization guarantees the classes never double-count: a token billed as
/// `cache_read` is excluded from `input`, and `output` already subsumes any
/// reasoning/thinking tokens (those are billed at the output rate). The raw
/// `reasoning` count is retained for observability only and is **not** a
/// separate billing class.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Uncached prompt tokens billed at the input rate.
    pub input: u64,
    /// Completion tokens (including reasoning/thinking) billed at the output rate.
    pub output: u64,
    /// Cache-hit prompt tokens billed at the discounted cache-read rate.
    pub cache_read: u64,
    /// Five-minute prompt-cache write tokens billed at the 5m cache-write rate (Anthropic).
    pub cache_write_5m: u64,
    /// One-hour prompt-cache write tokens billed at the 1h cache-write rate (Anthropic).
    pub cache_write_1h: u64,
    /// Reasoning/thinking tokens — a subset of `output`, retained for
    /// observability. Not billed separately.
    pub reasoning: u64,
}

impl TokenUsage {
    /// Total billable tokens across the disjoint classes (excludes the
    /// observability-only `reasoning` subset).
    pub fn total_billable(&self) -> u64 {
        self.input
            .saturating_add(self.output)
            .saturating_add(self.cache_read)
            .saturating_add(self.cache_write_5m)
            .saturating_add(self.cache_write_1h)
    }
}

// ---------------------------------------------------------------------------
// Provider raw usage shapes (Deserialize-only; adapters parse the JSON)
// ---------------------------------------------------------------------------

/// Anthropic Messages API `usage` block. `input_tokens` does **not** include
/// cache tokens — Anthropic reports cache reads/writes as separate counts.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    /// Newer responses break cache creation into TTL buckets. When present,
    /// detailed per-TTL values win; the legacy scalar is fallback-only.
    #[serde(default)]
    pub cache_creation: Option<AnthropicCacheCreation>,
}

/// Per-TTL cache-creation breakdown (Anthropic extended prompt caching).
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct AnthropicCacheCreation {
    #[serde(default)]
    pub ephemeral_5m_input_tokens: u64,
    #[serde(default)]
    pub ephemeral_1h_input_tokens: u64,
}

/// OpenAI Responses API `usage` block. `input_tokens` **includes**
/// `input_tokens_details.cached_tokens`; `output_tokens` **includes**
/// `output_tokens_details.reasoning_tokens`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct OpenAiResponsesUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub input_tokens_details: OpenAiInputTokensDetails,
    #[serde(default)]
    pub output_tokens_details: OpenAiOutputTokensDetails,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct OpenAiInputTokensDetails {
    #[serde(default)]
    pub cached_tokens: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct OpenAiOutputTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: u64,
}

/// Gemini `generateContent` `usageMetadata` block. `promptTokenCount`
/// **includes** `cachedContentTokenCount`; thinking models report
/// `thoughtsTokenCount` separately from `candidatesTokenCount`, billed at the
/// output rate.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiUsageMetadata {
    pub prompt_token_count: u64,
    pub candidates_token_count: u64,
    #[serde(default)]
    pub cached_content_token_count: u64,
    #[serde(default)]
    pub thoughts_token_count: u64,
}

// ---------------------------------------------------------------------------
// UsageExtractor — provider shape -> canonical TokenUsage
// ---------------------------------------------------------------------------

/// Stateless normalizer from each provider's raw usage shape into the canonical
/// [`TokenUsage`]. Adapters deserialize the provider JSON into the typed shapes
/// above (keeping JSON parsing in the adapter layer) and call these.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UsageExtractor;

impl UsageExtractor {
    /// Normalize an Anthropic Messages `usage` block.
    ///
    /// Anthropic input is already cache-exclusive. Detailed per-TTL cache
    /// creation wins when present; the legacy scalar is a scalar-only fallback
    /// treated as the default five-minute cache write.
    pub fn from_anthropic(&self, raw: &AnthropicUsage) -> TokenUsage {
        let (cache_write_5m, cache_write_1h) = if let Some(cache_creation) = &raw.cache_creation {
            (
                cache_creation.ephemeral_5m_input_tokens,
                cache_creation.ephemeral_1h_input_tokens,
            )
        } else if raw.cache_creation_input_tokens > 0 {
            (raw.cache_creation_input_tokens, 0)
        } else {
            (0, 0)
        };
        TokenUsage {
            input: raw.input_tokens,
            output: raw.output_tokens,
            cache_read: raw.cache_read_input_tokens,
            cache_write_5m,
            cache_write_1h,
            reasoning: 0,
        }
    }

    /// Normalize an OpenAI Responses `usage` block.
    ///
    /// `input_tokens` includes cached tokens, so uncached input is the
    /// difference. OpenAI has no per-token cache-write charge. Reasoning tokens
    /// are a subset of `output_tokens` (already billed at the output rate).
    pub fn from_openai_responses(&self, raw: &OpenAiResponsesUsage) -> TokenUsage {
        let cache_read = raw.input_tokens_details.cached_tokens;
        TokenUsage {
            input: raw.input_tokens.saturating_sub(cache_read),
            output: raw.output_tokens,
            cache_read,
            cache_write_5m: 0,
            cache_write_1h: 0,
            reasoning: raw.output_tokens_details.reasoning_tokens,
        }
    }

    /// Normalize a Gemini `usageMetadata` block.
    ///
    /// `promptTokenCount` includes cached content, so uncached input is the
    /// difference. `thoughtsTokenCount` is reported separately from
    /// `candidatesTokenCount` and is billed at the output rate, so output is
    /// their sum. Gemini context caching has no per-token write charge here
    /// (storage is billed out-of-band).
    pub fn from_gemini(&self, raw: &GeminiUsageMetadata) -> TokenUsage {
        let cache_read = raw.cached_content_token_count;
        TokenUsage {
            input: raw.prompt_token_count.saturating_sub(cache_read),
            output: raw
                .candidates_token_count
                .saturating_add(raw.thoughts_token_count),
            cache_read,
            cache_write_5m: 0,
            cache_write_1h: 0,
            reasoning: raw.thoughts_token_count,
        }
    }
}

// ---------------------------------------------------------------------------
// PriceBook — policy-as-data rate table
// ---------------------------------------------------------------------------

/// Per-model rate card. Rates are integer nanoUSD per 1,000,000 tokens.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RateCard {
    pub input_nanos_per_mtok: i64,
    pub output_nanos_per_mtok: i64,
    pub cache_read_nanos_per_mtok: i64,
    pub cache_write_5m_nanos_per_mtok: i64,
    pub cache_write_1h_nanos_per_mtok: i64,
}

impl RateCard {
    /// Reject negative or absurd rates so a malformed table cannot mint a
    /// credit or silently misbill.
    fn validate(&self) -> Result<(), CostError> {
        for rate in [
            self.input_nanos_per_mtok,
            self.output_nanos_per_mtok,
            self.cache_read_nanos_per_mtok,
            self.cache_write_5m_nanos_per_mtok,
            self.cache_write_1h_nanos_per_mtok,
        ] {
            if rate < 0 {
                return Err(CostError::NegativeRate);
            }
        }
        Ok(())
    }
}

/// One row in the typed rate table. Deserialized by the adapter/test layer
/// (which owns `serde_json`) and validated/indexed by [`PriceBook::from_file`].
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub struct RateCardRow {
    pub model: String,
    pub provider: String,
    pub input_nanos_per_mtok: i64,
    pub output_nanos_per_mtok: i64,
    pub cache_read_nanos_per_mtok: i64,
    pub cache_write_5m_nanos_per_mtok: i64,
    pub cache_write_1h_nanos_per_mtok: i64,
}

/// The typed rate table supplied by the Billing/FinOps adapter.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
pub struct PriceBookFile {
    #[serde(default)]
    pub schema_version: String,
    pub effective_at: String,
    pub cards: Vec<RateCardRow>,
}

/// Policy-as-data rate table keyed by provider + canonical upstream model id
/// (lower-cased on load and lookup so casing never causes a silent miss).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PriceBook {
    schema_version: String,
    effective_at: String,
    cards: BTreeMap<RateCardKey, RateCard>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RateCardKey {
    provider: String,
    model: String,
}

impl PriceBook {
    /// Build a validated, indexed price book from a typed table.
    ///
    /// Pure: no I/O, no JSON parsing. The caller deserializes
    /// a Billing/FinOps-owned table into a [`PriceBookFile`] and hands it here.
    /// Fails closed on an empty key, duplicate provider/model, or negative rate
    /// so a malformed table cannot misbill.
    pub fn from_file(file: PriceBookFile) -> Result<Self, CostError> {
        if file.effective_at.trim().is_empty() {
            return Err(CostError::EmptyEffectiveAt);
        }
        let mut cards = BTreeMap::new();
        for row in file.cards {
            let provider = normalize_provider_key(&row.provider);
            if provider.is_empty() {
                return Err(CostError::EmptyProviderKey);
            }
            let model = normalize_model_key(&row.model);
            if model.is_empty() {
                return Err(CostError::EmptyModelKey);
            }
            let key = RateCardKey { provider, model };
            let card = RateCard {
                input_nanos_per_mtok: row.input_nanos_per_mtok,
                output_nanos_per_mtok: row.output_nanos_per_mtok,
                cache_read_nanos_per_mtok: row.cache_read_nanos_per_mtok,
                cache_write_5m_nanos_per_mtok: row.cache_write_5m_nanos_per_mtok,
                cache_write_1h_nanos_per_mtok: row.cache_write_1h_nanos_per_mtok,
            };
            card.validate()?;
            if cards.insert(key.clone(), card).is_some() {
                return Err(CostError::DuplicateRateCard {
                    provider: key.provider,
                    model: key.model,
                });
            }
        }
        Ok(Self {
            schema_version: file.schema_version,
            effective_at: file.effective_at,
            cards,
        })
    }

    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn effective_at(&self) -> &str {
        &self.effective_at
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Look up the rate card for a provider/model pair (case-insensitive).
    pub fn rate_card(&self, provider: &str, model: &str) -> Option<&RateCard> {
        self.cards.get(&rate_card_key(provider, model)?)
    }

    /// Price a normalized usage against the provider/model card, failing closed
    /// (never zero-cost) when the card is absent from the table.
    pub fn cost_for(
        &self,
        provider: &str,
        model: &str,
        usage: &TokenUsage,
    ) -> Result<CostRecord, CostError> {
        let card = self
            .rate_card(provider, model)
            .ok_or_else(|| CostError::UnknownRateCard {
                provider: provider.to_string(),
                model: model.to_string(),
            })?;
        Ok(cost_of(usage, card))
    }
}

fn rate_card_key(provider: &str, model: &str) -> Option<RateCardKey> {
    let provider = normalize_provider_key(provider);
    let model = normalize_model_key(model);
    if provider.is_empty() || model.is_empty() {
        return None;
    }
    Some(RateCardKey { provider, model })
}

fn normalize_provider_key(provider: &str) -> String {
    provider.trim().to_ascii_lowercase()
}

fn normalize_model_key(model: &str) -> String {
    model.trim().to_ascii_lowercase()
}

// ---------------------------------------------------------------------------
// CostRecord + cost_of
// ---------------------------------------------------------------------------

/// Per-class and total cost in picoUSD (1e-12 USD). This is the spend breakdown
/// the adapter joins with [`crate::LlmGatewayEvent`] identity fields before
/// emitting to the ClickHouse spend sink.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CostRecord {
    pub input_pico_usd: i128,
    pub output_pico_usd: i128,
    pub cache_read_pico_usd: i128,
    pub cache_write_5m_pico_usd: i128,
    pub cache_write_1h_pico_usd: i128,
    pub total_pico_usd: i128,
}

impl CostRecord {
    /// Whole micro-USD (1e-6 USD), truncated toward zero — convenience for
    /// dashboards that aggregate in micros. Precise accounting uses picoUSD.
    pub fn total_micro_usd_floor(&self) -> i128 {
        self.total_pico_usd / 1_000_000
    }

    /// Render the total as a fixed 6-decimal USD string (micro-USD precision).
    pub fn format_usd(&self) -> String {
        let micros = self.total_micro_usd_floor();
        let sign = if micros < 0 { "-" } else { "" };
        let micros = micros.unsigned_abs();
        format!("{sign}{}.{:06}", micros / 1_000_000, micros % 1_000_000)
    }
}

/// Cost a single billable token class: `tokens * nanos_per_mtok / 1000`, in
/// picoUSD. Saturating arithmetic — a money path must never panic on overflow.
fn class_cost_pico(tokens: u64, nanos_per_mtok: i64) -> i128 {
    // picoUSD = tokens / TOKENS_PER_MTOK * nanos_per_mtok * PICOS_PER_NANO
    //         = tokens * nanos_per_mtok * PICOS_PER_NANO / TOKENS_PER_MTOK
    //         = tokens * nanos_per_mtok / 1000   (PICOS_PER_NANO=1e3, MTOK=1e6)
    (tokens as i128)
        .saturating_mul(nanos_per_mtok as i128)
        .saturating_mul(PICOS_PER_NANO)
        / TOKENS_PER_MTOK
}

/// Pure cost function: map normalized [`TokenUsage`] + a [`RateCard`] to a
/// [`CostRecord`] in picoUSD. No I/O, no allocation, deterministic.
pub fn cost_of(usage: &TokenUsage, card: &RateCard) -> CostRecord {
    let input_pico_usd = class_cost_pico(usage.input, card.input_nanos_per_mtok);
    let output_pico_usd = class_cost_pico(usage.output, card.output_nanos_per_mtok);
    let cache_read_pico_usd = class_cost_pico(usage.cache_read, card.cache_read_nanos_per_mtok);
    let cache_write_5m_pico_usd =
        class_cost_pico(usage.cache_write_5m, card.cache_write_5m_nanos_per_mtok);
    let cache_write_1h_pico_usd =
        class_cost_pico(usage.cache_write_1h, card.cache_write_1h_nanos_per_mtok);
    let total_pico_usd = input_pico_usd
        .saturating_add(output_pico_usd)
        .saturating_add(cache_read_pico_usd)
        .saturating_add(cache_write_5m_pico_usd)
        .saturating_add(cache_write_1h_pico_usd);
    CostRecord {
        input_pico_usd,
        output_pico_usd,
        cache_read_pico_usd,
        cache_write_5m_pico_usd,
        cache_write_1h_pico_usd,
        total_pico_usd,
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CostError {
    /// A rate table row had an empty provider key.
    EmptyProviderKey,
    /// A rate table row had an empty model key.
    EmptyModelKey,
    /// The table omitted its effective timestamp/version boundary.
    EmptyEffectiveAt,
    /// Two rows resolved to the same provider/model key.
    DuplicateRateCard { provider: String, model: String },
    /// A rate was negative.
    NegativeRate,
    /// No rate card exists for the requested provider/model — fail closed, never $0.
    UnknownRateCard { provider: String, model: String },
}

impl std::fmt::Display for CostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyProviderKey => write!(f, "pricebook row has empty provider key"),
            Self::EmptyModelKey => write!(f, "pricebook row has empty model key"),
            Self::EmptyEffectiveAt => write!(f, "pricebook effective_at is empty"),
            Self::DuplicateRateCard { provider, model } => {
                write!(
                    f,
                    "pricebook has duplicate rate-card key: {provider}/{model}"
                )
            }
            Self::NegativeRate => write!(f, "pricebook rate is negative"),
            Self::UnknownRateCard { provider, model } => {
                write!(f, "no rate card for provider/model: {provider}/{model}")
            }
        }
    }
}

impl std::error::Error for CostError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn opus_card() -> RateCard {
        // $5 / $25 / $0.50 / $6.25 per MTok.
        RateCard {
            input_nanos_per_mtok: 5_000_000_000,
            output_nanos_per_mtok: 25_000_000_000,
            cache_read_nanos_per_mtok: 500_000_000,
            cache_write_5m_nanos_per_mtok: 6_250_000_000,
            cache_write_1h_nanos_per_mtok: 10_000_000_000,
        }
    }

    #[test]
    fn class_cost_is_exact_for_published_rates() {
        // 1 token at $0.0375/MTok = 37_500_000 nanos/MTok = 37_500 picoUSD, exact.
        assert_eq!(class_cost_pico(1, 37_500_000), 37_500);
        // 1_000_000 tokens at $5/MTok = $5 = 5_000_000_000_000 picoUSD.
        assert_eq!(class_cost_pico(1_000_000, 5_000_000_000), 5_000_000_000_000);
        // Zero rate (no cache-write charge) costs nothing.
        assert_eq!(class_cost_pico(10_000, 0), 0);
    }

    #[test]
    fn cost_of_sums_disjoint_classes() {
        let usage = TokenUsage {
            input: 1_000_000,
            output: 1_000_000,
            cache_read: 1_000_000,
            cache_write_5m: 1_000_000,
            cache_write_1h: 0,
            reasoning: 250_000,
        };
        let rec = cost_of(&usage, &opus_card());
        assert_eq!(rec.input_pico_usd, 5_000_000_000_000);
        assert_eq!(rec.output_pico_usd, 25_000_000_000_000);
        assert_eq!(rec.cache_read_pico_usd, 500_000_000_000);
        assert_eq!(rec.cache_write_5m_pico_usd, 6_250_000_000_000);
        assert_eq!(rec.cache_write_1h_pico_usd, 0);
        assert_eq!(
            rec.total_pico_usd,
            5_000_000_000_000 + 25_000_000_000_000 + 500_000_000_000 + 6_250_000_000_000
        );
        // $36.75 total.
        assert_eq!(rec.format_usd(), "36.750000");
    }

    #[test]
    fn anthropic_normalization_keeps_input_cache_disjoint() {
        let raw = AnthropicUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: 200,
            cache_read_input_tokens: 300,
            cache_creation: None,
        };
        let u = UsageExtractor.from_anthropic(&raw);
        assert_eq!(
            u,
            TokenUsage {
                input: 100,
                output: 50,
                cache_read: 300,
                cache_write_5m: 200,
                cache_write_1h: 0,
                reasoning: 0,
            }
        );
    }

    #[test]
    fn anthropic_prefers_per_ttl_cache_creation_over_legacy_scalar() {
        let raw = AnthropicUsage {
            input_tokens: 10,
            output_tokens: 5,
            cache_creation_input_tokens: 100,
            cache_read_input_tokens: 0,
            cache_creation: Some(AnthropicCacheCreation {
                ephemeral_5m_input_tokens: 40,
                ephemeral_1h_input_tokens: 60,
            }),
        };
        let usage = UsageExtractor.from_anthropic(&raw);
        assert_eq!(usage.cache_write_5m, 40);
        assert_eq!(usage.cache_write_1h, 60);
    }

    #[test]
    fn openai_subtracts_cached_from_input_and_keeps_reasoning_in_output() {
        let raw = OpenAiResponsesUsage {
            input_tokens: 100, // includes 30 cached
            output_tokens: 80, // includes 20 reasoning
            input_tokens_details: OpenAiInputTokensDetails { cached_tokens: 30 },
            output_tokens_details: OpenAiOutputTokensDetails {
                reasoning_tokens: 20,
            },
        };
        let u = UsageExtractor.from_openai_responses(&raw);
        assert_eq!(
            u,
            TokenUsage {
                input: 70,
                output: 80,
                cache_read: 30,
                cache_write_5m: 0,
                cache_write_1h: 0,
                reasoning: 20,
            }
        );
    }

    #[test]
    fn gemini_subtracts_cached_and_adds_thoughts_to_output() {
        let raw = GeminiUsageMetadata {
            prompt_token_count: 100, // includes 25 cached
            candidates_token_count: 40,
            cached_content_token_count: 25,
            thoughts_token_count: 15,
        };
        let u = UsageExtractor.from_gemini(&raw);
        assert_eq!(
            u,
            TokenUsage {
                input: 75,
                output: 55, // 40 candidates + 15 thoughts
                cache_read: 25,
                cache_write_5m: 0,
                cache_write_1h: 0,
                reasoning: 15,
            }
        );
    }

    #[test]
    fn malformed_provider_counts_saturate_not_panic() {
        // cached > input must not underflow.
        let raw = OpenAiResponsesUsage {
            input_tokens: 10,
            output_tokens: 0,
            input_tokens_details: OpenAiInputTokensDetails { cached_tokens: 999 },
            output_tokens_details: OpenAiOutputTokensDetails::default(),
        };
        assert_eq!(UsageExtractor.from_openai_responses(&raw).input, 0);
    }

    fn row(provider: &str, model: &str, input: i64) -> RateCardRow {
        RateCardRow {
            model: model.to_string(),
            provider: provider.to_string(),
            input_nanos_per_mtok: input,
            output_nanos_per_mtok: 0,
            cache_read_nanos_per_mtok: 0,
            cache_write_5m_nanos_per_mtok: 0,
            cache_write_1h_nanos_per_mtok: 0,
        }
    }

    #[test]
    fn from_file_indexes_and_prices_case_insensitively() {
        let pb = PriceBook::from_file(PriceBookFile {
            schema_version: "cost-pricebook/v1".to_string(),
            effective_at: "2026-06-26T00:00:00Z".to_string(),
            cards: vec![row("anthropic", "claude-sonnet-4-5", 3_000_000_000)],
        })
        .expect("valid table");
        assert_eq!(pb.len(), 1);
        assert_eq!(pb.schema_version(), "cost-pricebook/v1");
        assert_eq!(pb.effective_at(), "2026-06-26T00:00:00Z");
        // Case-insensitive lookup.
        assert!(pb.rate_card("ANTHROPIC", "CLAUDE-SONNET-4-5").is_some());
        let rec = pb
            .cost_for(
                "anthropic",
                "claude-sonnet-4-5",
                &TokenUsage {
                    input: 1_000_000,
                    ..Default::default()
                },
            )
            .expect("known model");
        // $3 / MTok input.
        assert_eq!(rec.total_pico_usd, 3_000_000_000_000);
    }

    #[test]
    fn unknown_model_fails_closed() {
        let pb = PriceBook::from_file(PriceBookFile {
            schema_version: "v".to_string(),
            effective_at: "2026-06-26T00:00:00Z".to_string(),
            cards: vec![row("anthropic", "known", 1)],
        })
        .unwrap();
        assert_eq!(
            pb.cost_for("anthropic", "no-such-model", &TokenUsage::default()),
            Err(CostError::UnknownRateCard {
                provider: "anthropic".to_string(),
                model: "no-such-model".to_string()
            })
        );
    }

    #[test]
    fn empty_provider_key_rejected() {
        let err = PriceBook::from_file(PriceBookFile {
            schema_version: "v".to_string(),
            effective_at: "2026-06-26T00:00:00Z".to_string(),
            cards: vec![row("   ", "m", 1)],
        });
        assert_eq!(err, Err(CostError::EmptyProviderKey));
    }

    #[test]
    fn empty_model_key_rejected() {
        let err = PriceBook::from_file(PriceBookFile {
            schema_version: "v".to_string(),
            effective_at: "2026-06-26T00:00:00Z".to_string(),
            cards: vec![row("anthropic", "   ", 1)],
        });
        assert_eq!(err, Err(CostError::EmptyModelKey));
    }

    #[test]
    fn empty_effective_at_rejected() {
        let err = PriceBook::from_file(PriceBookFile {
            schema_version: "v".to_string(),
            effective_at: "   ".to_string(),
            cards: vec![row("anthropic", "m", 1)],
        });
        assert_eq!(err, Err(CostError::EmptyEffectiveAt));
    }

    #[test]
    fn negative_rate_rejected() {
        let err = PriceBook::from_file(PriceBookFile {
            schema_version: "v".to_string(),
            effective_at: "2026-06-26T00:00:00Z".to_string(),
            cards: vec![row("anthropic", "m", -1)],
        });
        assert_eq!(err, Err(CostError::NegativeRate));
    }

    #[test]
    fn duplicate_provider_model_rejected() {
        let err = PriceBook::from_file(PriceBookFile {
            schema_version: "v".to_string(),
            effective_at: "2026-06-26T00:00:00Z".to_string(),
            cards: vec![row("anthropic", "M", 1), row("ANTHROPIC", "m", 2)],
        });
        assert_eq!(
            err,
            Err(CostError::DuplicateRateCard {
                provider: "anthropic".to_string(),
                model: "m".to_string()
            })
        );
    }

    #[test]
    fn same_model_can_exist_under_different_provider() {
        let pb = PriceBook::from_file(PriceBookFile {
            schema_version: "v".to_string(),
            effective_at: "2026-06-26T00:00:00Z".to_string(),
            cards: vec![row("anthropic", "shared", 1), row("openai", "shared", 2)],
        });
        assert!(pb.is_ok());
    }
}
