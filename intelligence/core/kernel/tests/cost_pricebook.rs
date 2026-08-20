//! Table-driven cost tests over REAL provider usage fixtures.
//!
//! These exercise the full seam end to end: parse an adapter-supplied pricebook
//! JSON and raw provider `usage` blocks (as they appear on the wire) with
//! `serde_json` — the same parse the REST/spend adapter performs — normalize via
//! [`UsageExtractor`], and price against the [`PriceBook`]. `serde_json` lives in
//! the test/adapter layer; the kernel lib never parses JSON itself.

use intelligence_kernel::cost::{
    AnthropicUsage, CostError, GeminiUsageMetadata, OpenAiResponsesUsage, PriceBook, PriceBookFile,
    TokenUsage, UsageExtractor, cost_of,
};

const PRICEBOOK_FIXTURE_JSON: &str = r#"{
  "schema_version": "cost-pricebook/v1",
  "effective_at": "2026-06-26T00:00:00Z",
  "cards": [
    {
      "model": "claude-opus-4-7",
      "provider": "anthropic",
      "input_nanos_per_mtok": 5000000000,
      "output_nanos_per_mtok": 25000000000,
      "cache_read_nanos_per_mtok": 500000000,
      "cache_write_5m_nanos_per_mtok": 6250000000,
      "cache_write_1h_nanos_per_mtok": 10000000000
    },
    {
      "model": "claude-opus-4-6",
      "provider": "anthropic",
      "input_nanos_per_mtok": 5000000000,
      "output_nanos_per_mtok": 25000000000,
      "cache_read_nanos_per_mtok": 500000000,
      "cache_write_5m_nanos_per_mtok": 6250000000,
      "cache_write_1h_nanos_per_mtok": 10000000000
    },
    {
      "model": "claude-opus-4-5",
      "provider": "anthropic",
      "input_nanos_per_mtok": 5000000000,
      "output_nanos_per_mtok": 25000000000,
      "cache_read_nanos_per_mtok": 500000000,
      "cache_write_5m_nanos_per_mtok": 6250000000,
      "cache_write_1h_nanos_per_mtok": 10000000000
    },
    {
      "model": "claude-sonnet-4-5",
      "provider": "anthropic",
      "input_nanos_per_mtok": 3000000000,
      "output_nanos_per_mtok": 15000000000,
      "cache_read_nanos_per_mtok": 300000000,
      "cache_write_5m_nanos_per_mtok": 3750000000,
      "cache_write_1h_nanos_per_mtok": 6000000000
    },
    {
      "model": "claude-haiku-3-5",
      "provider": "anthropic",
      "input_nanos_per_mtok": 800000000,
      "output_nanos_per_mtok": 4000000000,
      "cache_read_nanos_per_mtok": 80000000,
      "cache_write_5m_nanos_per_mtok": 1000000000,
      "cache_write_1h_nanos_per_mtok": 1600000000
    },
    {
      "model": "gpt-4.1",
      "provider": "openai",
      "input_nanos_per_mtok": 2000000000,
      "output_nanos_per_mtok": 8000000000,
      "cache_read_nanos_per_mtok": 500000000,
      "cache_write_5m_nanos_per_mtok": 0,
      "cache_write_1h_nanos_per_mtok": 0
    },
    {
      "model": "gpt-4o",
      "provider": "openai",
      "input_nanos_per_mtok": 2500000000,
      "output_nanos_per_mtok": 10000000000,
      "cache_read_nanos_per_mtok": 1250000000,
      "cache_write_5m_nanos_per_mtok": 0,
      "cache_write_1h_nanos_per_mtok": 0
    },
    {
      "model": "o3",
      "provider": "openai",
      "input_nanos_per_mtok": 2000000000,
      "output_nanos_per_mtok": 8000000000,
      "cache_read_nanos_per_mtok": 500000000,
      "cache_write_5m_nanos_per_mtok": 0,
      "cache_write_1h_nanos_per_mtok": 0
    },
    {
      "model": "gemini-2.5-pro",
      "provider": "gemini",
      "input_nanos_per_mtok": 1250000000,
      "output_nanos_per_mtok": 10000000000,
      "cache_read_nanos_per_mtok": 312500000,
      "cache_write_5m_nanos_per_mtok": 0,
      "cache_write_1h_nanos_per_mtok": 0
    },
    {
      "model": "gemini-2.5-flash",
      "provider": "gemini",
      "input_nanos_per_mtok": 300000000,
      "output_nanos_per_mtok": 2500000000,
      "cache_read_nanos_per_mtok": 75000000,
      "cache_write_5m_nanos_per_mtok": 0,
      "cache_write_1h_nanos_per_mtok": 0
    }
  ]
}"#;

fn test_pricebook() -> PriceBook {
    let file: PriceBookFile =
        serde_json::from_str(PRICEBOOK_FIXTURE_JSON).expect("pricebook fixture JSON parses");
    PriceBook::from_file(file).expect("pricebook fixture is well-formed")
}

#[test]
fn adapter_supplied_pricebook_is_loadable_and_nonempty() {
    let pb = test_pricebook();
    assert_eq!(pb.schema_version(), "cost-pricebook/v1");
    assert_eq!(pb.effective_at(), "2026-06-26T00:00:00Z");
    assert!(pb.len() >= 10, "expected the full fixture rate table");
    for (provider, model) in [
        ("anthropic", "claude-opus-4-5"),
        ("anthropic", "claude-sonnet-4-5"),
        ("anthropic", "claude-haiku-3-5"),
        ("openai", "gpt-4.1"),
        ("openai", "o3"),
        ("gemini", "gemini-2.5-pro"),
        ("gemini", "gemini-2.5-flash"),
    ] {
        assert!(
            pb.rate_card(provider, model).is_some(),
            "missing rate card: {provider}/{model}"
        );
    }
}

#[test]
fn anthropic_real_usage_fixture_prices_exactly() {
    // Real-shape Anthropic Messages `usage` block (cache-exclusive input).
    let raw: AnthropicUsage = serde_json::from_str(
        r#"{
            "input_tokens": 1000000,
            "output_tokens": 1000000,
            "cache_creation_input_tokens": 1000000,
            "cache_read_input_tokens": 1000000
        }"#,
    )
    .unwrap();
    let usage = UsageExtractor.from_anthropic(&raw);
    assert_eq!(
        usage,
        TokenUsage {
            input: 1_000_000,
            output: 1_000_000,
            cache_read: 1_000_000,
            cache_write_5m: 1_000_000,
            cache_write_1h: 0,
            reasoning: 0,
        }
    );

    let pb = test_pricebook();
    let rec = pb
        .cost_for("anthropic", "claude-sonnet-4-5", &usage)
        .unwrap();
    // Sonnet 4.5: $3 in / $15 out / $0.30 read / $3.75 5m write per MTok.
    assert_eq!(rec.input_pico_usd, 3_000_000_000_000);
    assert_eq!(rec.output_pico_usd, 15_000_000_000_000);
    assert_eq!(rec.cache_read_pico_usd, 300_000_000_000);
    assert_eq!(rec.cache_write_5m_pico_usd, 3_750_000_000_000);
    assert_eq!(rec.cache_write_1h_pico_usd, 0);
    assert_eq!(rec.total_pico_usd, 22_050_000_000_000); // $22.05
    assert_eq!(rec.format_usd(), "22.050000");
}

#[test]
fn anthropic_per_ttl_cache_creation_fixture() {
    // Newer Anthropic response: detailed per-TTL breakdown wins over the legacy scalar.
    let raw: AnthropicUsage = serde_json::from_str(
        r#"{
            "input_tokens": 500,
            "output_tokens": 250,
            "cache_creation_input_tokens": 1000,
            "cache_read_input_tokens": 100,
            "cache_creation": {
                "ephemeral_5m_input_tokens": 400,
                "ephemeral_1h_input_tokens": 600
            }
        }"#,
    )
    .unwrap();
    let usage = UsageExtractor.from_anthropic(&raw);
    assert_eq!(usage.cache_write_5m, 400);
    assert_eq!(usage.cache_write_1h, 600);
    assert_eq!(usage.input, 500);
    assert_eq!(usage.cache_read, 100);

    let pb = test_pricebook();
    let rec = pb
        .cost_for("anthropic", "claude-sonnet-4-5", &usage)
        .unwrap();
    assert_eq!(rec.cache_write_5m_pico_usd, 400 * 3_750_000_000 / 1_000);
    assert_eq!(rec.cache_write_1h_pico_usd, 600 * 6_000_000_000 / 1_000);
    assert_eq!(rec.total_pico_usd, 10_380_000_000);
}

#[test]
fn openai_responses_real_usage_fixture_prices_exactly() {
    // Real-shape OpenAI Responses `usage`: input includes cached, output
    // includes reasoning.
    let raw: OpenAiResponsesUsage = serde_json::from_str(
        r#"{
            "input_tokens": 1000000,
            "input_tokens_details": { "cached_tokens": 400000 },
            "output_tokens": 1000000,
            "output_tokens_details": { "reasoning_tokens": 250000 },
            "total_tokens": 2000000
        }"#,
    )
    .unwrap();
    let usage = UsageExtractor.from_openai_responses(&raw);
    assert_eq!(
        usage,
        TokenUsage {
            input: 600_000, // 1.0M - 0.4M cached
            output: 1_000_000,
            cache_read: 400_000,
            cache_write_5m: 0,
            cache_write_1h: 0,
            reasoning: 250_000,
        }
    );

    let pb = test_pricebook();
    let rec = pb.cost_for("openai", "gpt-4.1", &usage).unwrap();
    // gpt-4.1: $2 in / $8 out / $0.50 read per MTok, no write charge.
    assert_eq!(rec.input_pico_usd, 600_000 * 2_000_000_000 / 1_000); // 1_200_000_000_000
    assert_eq!(rec.output_pico_usd, 8_000_000_000_000);
    assert_eq!(rec.cache_read_pico_usd, 400_000 * 500_000_000 / 1_000); // 200_000_000_000
    assert_eq!(rec.cache_write_5m_pico_usd, 0);
    assert_eq!(rec.cache_write_1h_pico_usd, 0);
    assert_eq!(rec.total_pico_usd, 9_400_000_000_000); // $9.40
}

#[test]
fn gemini_real_usage_fixture_prices_exactly() {
    // Real-shape Gemini `usageMetadata`: prompt includes cached, thoughts
    // separate from candidates.
    let raw: GeminiUsageMetadata = serde_json::from_str(
        r#"{
            "promptTokenCount": 1000000,
            "candidatesTokenCount": 800000,
            "cachedContentTokenCount": 200000,
            "thoughtsTokenCount": 200000,
            "totalTokenCount": 2000000
        }"#,
    )
    .unwrap();
    let usage = UsageExtractor.from_gemini(&raw);
    assert_eq!(
        usage,
        TokenUsage {
            input: 800_000,    // 1.0M - 0.2M cached
            output: 1_000_000, // 0.8M candidates + 0.2M thoughts
            cache_read: 200_000,
            cache_write_5m: 0,
            cache_write_1h: 0,
            reasoning: 200_000,
        }
    );

    let pb = test_pricebook();
    let rec = pb.cost_for("gemini", "gemini-2.5-pro", &usage).unwrap();
    // gemini-2.5-pro: $1.25 in / $10 out / $0.3125 read per MTok.
    assert_eq!(rec.input_pico_usd, 800_000 * 1_250_000_000 / 1_000); // 1_000_000_000_000
    assert_eq!(rec.output_pico_usd, 10_000_000_000_000);
    assert_eq!(rec.cache_read_pico_usd, 200_000 * 312_500_000 / 1_000); // 62_500_000_000
    assert_eq!(rec.cache_write_5m_pico_usd, 0);
    assert_eq!(rec.cache_write_1h_pico_usd, 0);
    assert_eq!(rec.total_pico_usd, 11_062_500_000_000); // $11.0625
    assert_eq!(rec.format_usd(), "11.062500");
}

#[test]
fn malformed_usage_blocks_fail_closed() {
    assert!(serde_json::from_str::<AnthropicUsage>("{}").is_err());
    assert!(serde_json::from_str::<OpenAiResponsesUsage>("{}").is_err());
    assert!(serde_json::from_str::<GeminiUsageMetadata>("{}").is_err());
}

#[test]
fn explicit_zero_usage_prices_as_zero() {
    let raw: AnthropicUsage = serde_json::from_str(
        r#"{
            "input_tokens": 0,
            "output_tokens": 0
        }"#,
    )
    .unwrap();
    let usage = UsageExtractor.from_anthropic(&raw);
    assert_eq!(usage, TokenUsage::default());
    let pb = test_pricebook();
    let rec = pb.cost_for("anthropic", "claude-opus-4-5", &usage).unwrap();
    assert_eq!(rec.total_pico_usd, 0);
}

#[test]
fn unknown_model_fails_closed_not_free() {
    let pb = test_pricebook();
    let usage = TokenUsage {
        output: 1_000_000,
        ..Default::default()
    };
    assert_eq!(
        pb.cost_for("anthropic", "definitely-not-a-model", &usage),
        Err(CostError::UnknownRateCard {
            provider: "anthropic".to_string(),
            model: "definitely-not-a-model".to_string()
        })
    );
}

#[test]
fn cost_of_matches_pricebook_path() {
    // cost_of(usage, card) and PriceBook::cost_for agree.
    let pb = test_pricebook();
    let usage = TokenUsage {
        input: 123_456,
        output: 7_890,
        cache_read: 4_321,
        cache_write_5m: 0,
        cache_write_1h: 0,
        reasoning: 0,
    };
    let card = pb.rate_card("anthropic", "claude-haiku-3-5").unwrap();
    assert_eq!(
        cost_of(&usage, card),
        pb.cost_for("anthropic", "claude-haiku-3-5", &usage)
            .unwrap()
    );
}
