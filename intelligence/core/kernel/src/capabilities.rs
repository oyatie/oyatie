//! Per-model capability registry (pure kernel, policy-as-data).
//!
//! The OAuth subscription pool fronts heterogeneous upstream models that do not
//! all support the same request features. Sending a vision payload to a
//! text-only model, function-calling to a model without tool support, or an
//! over-long context to a small window is a guaranteed upstream 4xx that wastes
//! a pooled seat's quota. This module lets callers **pre-flight reject** such
//! requests *before* a seat is leased and a dispatch is attempted.
//!
//! Shape mirrors [`crate::model_routing::ModelRouter`]: a [`CapabilityRegistry`]
//! engine over a static policy table, with pure lookup + validation methods and
//! zero I/O. The table is keyed by the **canonical upstream model name** that
//! [`crate::model_routing::ModelRouter`] resolves to (e.g. `claude-opus-4-5`,
//! `gpt-4o`, `gemini-2.5-pro`), so routing and capability enforcement stay
//! aligned on a single identifier.
//!
//! Fail-closed: an unknown model is rejected, never waved through.

use crate::model_routing::ModelCapability;
use serde::{Deserialize, Serialize};

/// Capability flags + token windows for a single upstream model.
///
/// Policy-as-data: every field is a tunable knob carried in the static table
/// below. Token windows are upstream-published ceilings; the request-side
/// estimate must stay within them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Tool / function calling (Anthropic `tools`, OpenAI `tools`, Gemini
    /// `functionDeclarations`).
    pub supports_function_calling: bool,
    /// Image (and other non-text) input parts.
    pub supports_vision: bool,
    /// Provider-side prompt / context caching (Anthropic `cache_control`,
    /// OpenAI automatic prompt caching, Gemini context caching).
    pub supports_prompt_caching: bool,
    /// Extended-thinking / reasoning effort (Anthropic thinking, OpenAI
    /// o-series reasoning, Gemini 2.5 thinking).
    pub supports_reasoning: bool,
    /// Incremental token streaming (SSE).
    pub supports_streaming: bool,
    /// Maximum combined input (context window) in tokens.
    pub max_input_tokens: u32,
    /// Maximum tokens the model will generate in one response.
    pub max_output_tokens: u32,
}

impl ModelCapabilities {
    /// Apply routing-derived capability overrides to the base table entry.
    ///
    /// The router emits [`ModelCapability::OneMillionContext`] for `[1m]`-tagged
    /// / `*1m` alias models; that widens the input window to one million tokens
    /// so the pre-flight check does not reject a legitimately-extended context.
    #[must_use]
    pub fn with_routing_capabilities(mut self, routing: &[ModelCapability]) -> Self {
        if routing.contains(&ModelCapability::OneMillionContext) {
            self.max_input_tokens = self.max_input_tokens.max(1_000_000);
        }
        self
    }
}

/// Features a single inbound request needs from the target model.
///
/// Built by the REST/translation adapter from the parsed request body (presence
/// of image parts, `tools`, `stream`, thinking config, token estimate). The
/// kernel only sees this distilled, content-free shape.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CallRequirements {
    pub needs_function_calling: bool,
    pub needs_vision: bool,
    pub needs_prompt_caching: bool,
    pub needs_reasoning: bool,
    pub needs_streaming: bool,
    /// Estimated input tokens for the request (prompt + system + tool defs).
    pub estimated_input_tokens: u32,
    /// Requested `max_tokens` for the response.
    pub requested_max_output_tokens: u32,
}

/// A single way a request is incompatible with the resolved model.
///
/// Pre-flight collects **all** violations so the caller can report a complete
/// picture rather than one-at-a-time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CapabilityViolation {
    /// No table entry for the model — fail closed.
    UnknownModel,
    FunctionCallingUnsupported,
    VisionUnsupported,
    PromptCachingUnsupported,
    ReasoningUnsupported,
    StreamingUnsupported,
    InputTokensExceeded { requested: u32, max: u32 },
    OutputTokensExceeded { requested: u32, max: u32 },
}

/// A rejected pre-flight: the model plus every reason it cannot serve the call.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreflightRejection {
    pub model: String,
    pub violations: Vec<CapabilityViolation>,
}

/// How a table row matches an upstream model name.
#[derive(Clone, Copy, Debug)]
pub enum ModelMatch {
    /// Exact canonical name.
    Exact(&'static str),
    /// Family prefix; longest matching prefix wins.
    Prefix(&'static str),
}

impl ModelMatch {
    /// Specificity for tie-breaking. Exact always beats prefix; among prefixes
    /// the longest (most specific) wins.
    fn specificity(self) -> usize {
        match self {
            // Exact ranks above any prefix (+1000 over the longest plausible prefix).
            ModelMatch::Exact(name) => name.len() + 1_000,
            ModelMatch::Prefix(prefix) => prefix.len(),
        }
    }

    fn matches(self, model: &str) -> bool {
        match self {
            ModelMatch::Exact(name) => model == name,
            ModelMatch::Prefix(prefix) => model.starts_with(prefix),
        }
    }
}

/// Reasoning-capable, vision-capable frontier model (Anthropic Opus/Sonnet 4.x,
/// OpenAI o-series, Gemini 2.5). Defined once; rows below tweak token windows.
const fn frontier(max_input_tokens: u32, max_output_tokens: u32) -> ModelCapabilities {
    ModelCapabilities {
        supports_function_calling: true,
        supports_vision: true,
        supports_prompt_caching: true,
        supports_reasoning: true,
        supports_streaming: true,
        max_input_tokens,
        max_output_tokens,
    }
}

// ponytail: this static table IS the policy. Token ceilings are upstream-
// published values as of the model generation; tune rows here when providers
// revise limits — no code change needed elsewhere. Longest/most-specific match
// wins, so row order is irrelevant.
const PLATFORM_TABLE: &[(ModelMatch, ModelCapabilities)] = &[
    // --- Anthropic (subscription pool) ---
    // Claude Opus 4.x — full feature set, 200k context, 64k output.
    (
        ModelMatch::Prefix("claude-opus-4"),
        frontier(200_000, 64_000),
    ),
    // Claude Sonnet 4.x — same shape as Opus 4.x.
    (
        ModelMatch::Prefix("claude-sonnet-4"),
        frontier(200_000, 64_000),
    ),
    // Claude Haiku 3.5 — vision + tools + caching, no extended thinking, 8k output.
    (
        ModelMatch::Prefix("claude-haiku-3"),
        ModelCapabilities {
            supports_function_calling: true,
            supports_vision: true,
            supports_prompt_caching: true,
            supports_reasoning: false,
            supports_streaming: true,
            max_input_tokens: 200_000,
            max_output_tokens: 8_192,
        },
    ),
    // --- OpenAI-compatible ---
    // GPT-4o family — vision + tools + caching + streaming, no reasoning effort.
    (
        ModelMatch::Prefix("gpt-4o"),
        ModelCapabilities {
            supports_function_calling: true,
            supports_vision: true,
            supports_prompt_caching: true,
            supports_reasoning: false,
            supports_streaming: true,
            max_input_tokens: 128_000,
            max_output_tokens: 16_384,
        },
    ),
    // o-series reasoning models (o1 / o3 / o4) — reasoning + vision + tools.
    (ModelMatch::Prefix("o1"), frontier(200_000, 100_000)),
    (ModelMatch::Prefix("o3"), frontier(200_000, 100_000)),
    (ModelMatch::Prefix("o4"), frontier(200_000, 100_000)),
    // Embedding models — no chat features at all.
    (
        ModelMatch::Prefix("text-embedding-"),
        ModelCapabilities {
            supports_function_calling: false,
            supports_vision: false,
            supports_prompt_caching: false,
            supports_reasoning: false,
            supports_streaming: false,
            max_input_tokens: 8_192,
            max_output_tokens: 0,
        },
    ),
    // --- Gemini ---
    // Gemini 2.5 (pro/flash) — 1M context, full feature set, 64k output.
    (
        ModelMatch::Prefix("gemini-2.5"),
        frontier(1_048_576, 65_536),
    ),
];

/// Pure capability lookup + pre-flight validation engine over a policy table.
///
/// Mirrors [`crate::model_routing::ModelRouter`]: construct with
/// [`CapabilityRegistry::default`] for the platform table, or
/// [`CapabilityRegistry::with_table`] to inject a tenant/test policy.
#[derive(Clone, Copy, Debug)]
pub struct CapabilityRegistry {
    table: &'static [(ModelMatch, ModelCapabilities)],
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self {
            table: PLATFORM_TABLE,
        }
    }
}

impl CapabilityRegistry {
    /// Inject an alternate policy table (tenant overlay, fixtures).
    #[must_use]
    pub const fn with_table(table: &'static [(ModelMatch, ModelCapabilities)]) -> Self {
        Self { table }
    }

    /// Resolve the base capabilities for a canonical upstream model name.
    ///
    /// Returns `None` for an unknown model (callers must fail closed).
    #[must_use]
    pub fn lookup(&self, model: &str) -> Option<ModelCapabilities> {
        self.table
            .iter()
            .filter(|(rule, _)| rule.matches(model))
            .max_by_key(|(rule, _)| rule.specificity())
            .map(|(_, caps)| *caps)
    }

    /// Resolve capabilities, applying routing-derived overrides (1M context).
    #[must_use]
    pub fn lookup_with_routing(
        &self,
        model: &str,
        routing: &[ModelCapability],
    ) -> Option<ModelCapabilities> {
        self.lookup(model)
            .map(|caps| caps.with_routing_capabilities(routing))
    }

    /// Pre-flight a request against the resolved model.
    ///
    /// `routing` carries [`crate::model_routing::RoutingDecision::capabilities`]
    /// so context-extension tags widen the input window. On success returns the
    /// effective capabilities; on failure returns every violation found.
    ///
    /// Fail-closed: an unknown model yields a single
    /// [`CapabilityViolation::UnknownModel`].
    pub fn preflight(
        &self,
        model: &str,
        routing: &[ModelCapability],
        req: &CallRequirements,
    ) -> Result<ModelCapabilities, PreflightRejection> {
        let Some(caps) = self.lookup_with_routing(model, routing) else {
            return Err(PreflightRejection {
                model: model.to_string(),
                violations: vec![CapabilityViolation::UnknownModel],
            });
        };

        let mut violations = Vec::new();
        if req.needs_function_calling && !caps.supports_function_calling {
            violations.push(CapabilityViolation::FunctionCallingUnsupported);
        }
        if req.needs_vision && !caps.supports_vision {
            violations.push(CapabilityViolation::VisionUnsupported);
        }
        if req.needs_prompt_caching && !caps.supports_prompt_caching {
            violations.push(CapabilityViolation::PromptCachingUnsupported);
        }
        if req.needs_reasoning && !caps.supports_reasoning {
            violations.push(CapabilityViolation::ReasoningUnsupported);
        }
        if req.needs_streaming && !caps.supports_streaming {
            violations.push(CapabilityViolation::StreamingUnsupported);
        }
        if req.estimated_input_tokens > caps.max_input_tokens {
            violations.push(CapabilityViolation::InputTokensExceeded {
                requested: req.estimated_input_tokens,
                max: caps.max_input_tokens,
            });
        }
        if req.requested_max_output_tokens > caps.max_output_tokens {
            violations.push(CapabilityViolation::OutputTokensExceeded {
                requested: req.requested_max_output_tokens,
                max: caps.max_output_tokens,
            });
        }

        if violations.is_empty() {
            Ok(caps)
        } else {
            Err(PreflightRejection {
                model: model.to_string(),
                violations,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> CapabilityRegistry {
        CapabilityRegistry::default()
    }

    #[test]
    fn lookup_resolves_known_families_by_specificity() {
        // (canonical model, function, vision, prompt_cache, reasoning, stream, in, out)
        let cases = [
            (
                "claude-opus-4-5",
                true,
                true,
                true,
                true,
                true,
                200_000,
                64_000,
            ),
            (
                "claude-opus-4-7",
                true,
                true,
                true,
                true,
                true,
                200_000,
                64_000,
            ),
            (
                "claude-sonnet-4-5",
                true,
                true,
                true,
                true,
                true,
                200_000,
                64_000,
            ),
            (
                "claude-haiku-3-5",
                true,
                true,
                true,
                false,
                true,
                200_000,
                8_192,
            ),
            ("gpt-4o", true, true, true, false, true, 128_000, 16_384),
            ("o3-mini", true, true, true, true, true, 200_000, 100_000),
            ("o4-mini", true, true, true, true, true, 200_000, 100_000),
            (
                "text-embedding-3-large",
                false,
                false,
                false,
                false,
                false,
                8_192,
                0,
            ),
            (
                "gemini-2.5-pro",
                true,
                true,
                true,
                true,
                true,
                1_048_576,
                65_536,
            ),
            (
                "gemini-2.5-flash",
                true,
                true,
                true,
                true,
                true,
                1_048_576,
                65_536,
            ),
        ];
        for (model, fc, vis, pc, rea, st, max_in, max_out) in cases {
            let caps = reg().lookup(model).unwrap_or_else(|| panic!("{model}"));
            assert_eq!(caps.supports_function_calling, fc, "fc {model}");
            assert_eq!(caps.supports_vision, vis, "vision {model}");
            assert_eq!(caps.supports_prompt_caching, pc, "cache {model}");
            assert_eq!(caps.supports_reasoning, rea, "reasoning {model}");
            assert_eq!(caps.supports_streaming, st, "stream {model}");
            assert_eq!(caps.max_input_tokens, max_in, "in {model}");
            assert_eq!(caps.max_output_tokens, max_out, "out {model}");
        }
    }

    #[test]
    fn unknown_model_fails_closed() {
        assert_eq!(reg().lookup("tenant-private-model"), None);
        let rejection = reg()
            .preflight("tenant-private-model", &[], &CallRequirements::default())
            .expect_err("unknown model must reject");
        assert_eq!(rejection.violations, vec![CapabilityViolation::UnknownModel]);
        assert_eq!(rejection.model, "tenant-private-model");
    }

    #[test]
    fn vision_to_non_vision_model_is_rejected_preflight() {
        // text-embedding model has no vision.
        let req = CallRequirements {
            needs_vision: true,
            ..Default::default()
        };
        let err = reg()
            .preflight("text-embedding-3-large", &[], &req)
            .expect_err("vision to embedding model must reject");
        assert!(err
            .violations
            .contains(&CapabilityViolation::VisionUnsupported));
    }

    #[test]
    fn reasoning_to_haiku_is_rejected() {
        let req = CallRequirements {
            needs_reasoning: true,
            ..Default::default()
        };
        let err = reg()
            .preflight("claude-haiku-3-5", &[], &req)
            .expect_err("reasoning to haiku must reject");
        assert_eq!(
            err.violations,
            vec![CapabilityViolation::ReasoningUnsupported]
        );
    }

    #[test]
    fn supported_call_passes_and_returns_caps() {
        let req = CallRequirements {
            needs_function_calling: true,
            needs_vision: true,
            needs_prompt_caching: true,
            needs_reasoning: true,
            needs_streaming: true,
            estimated_input_tokens: 150_000,
            requested_max_output_tokens: 32_000,
        };
        let caps = reg()
            .preflight("claude-opus-4-5", &[], &req)
            .expect("fully-supported call must pass");
        assert_eq!(caps.max_output_tokens, 64_000);
    }

    #[test]
    fn all_violations_are_collected_not_just_first() {
        // Embedding model: ask for everything it cannot do, plus token overruns.
        let req = CallRequirements {
            needs_function_calling: true,
            needs_vision: true,
            needs_prompt_caching: true,
            needs_reasoning: true,
            needs_streaming: true,
            estimated_input_tokens: 100_000,
            requested_max_output_tokens: 4_096,
        };
        let err = reg()
            .preflight("text-embedding-3-large", &[], &req)
            .expect_err("incompatible call must reject");
        assert!(err
            .violations
            .contains(&CapabilityViolation::FunctionCallingUnsupported));
        assert!(err
            .violations
            .contains(&CapabilityViolation::VisionUnsupported));
        assert!(err
            .violations
            .contains(&CapabilityViolation::PromptCachingUnsupported));
        assert!(err
            .violations
            .contains(&CapabilityViolation::ReasoningUnsupported));
        assert!(err
            .violations
            .contains(&CapabilityViolation::StreamingUnsupported));
        assert!(err
            .violations
            .contains(&CapabilityViolation::InputTokensExceeded {
                requested: 100_000,
                max: 8_192,
            }));
        assert!(err
            .violations
            .contains(&CapabilityViolation::OutputTokensExceeded {
                requested: 4_096,
                max: 0,
            }));
    }

    #[test]
    fn input_over_window_is_rejected_unless_one_million_context() {
        let req = CallRequirements {
            estimated_input_tokens: 500_000,
            ..Default::default()
        };
        // Base Opus 4.5 = 200k window -> reject.
        let err = reg()
            .preflight("claude-opus-4-5", &[], &req)
            .expect_err("over-window must reject");
        assert!(err
            .violations
            .contains(&CapabilityViolation::InputTokensExceeded {
                requested: 500_000,
                max: 200_000,
            }));

        // With OneMillionContext routing tag -> window widens to 1M -> pass.
        reg()
            .preflight(
                "claude-opus-4-5",
                &[ModelCapability::OneMillionContext],
                &req,
            )
            .expect("1m context tag must widen input window");
    }

    #[test]
    fn output_over_max_is_rejected() {
        let req = CallRequirements {
            requested_max_output_tokens: 9_000,
            ..Default::default()
        };
        // Haiku 3.5 caps output at 8192.
        let err = reg()
            .preflight("claude-haiku-3-5", &[], &req)
            .expect_err("over-output must reject");
        assert_eq!(
            err.violations,
            vec![CapabilityViolation::OutputTokensExceeded {
                requested: 9_000,
                max: 8_192,
            }]
        );
    }

    #[test]
    fn lookup_with_routing_widens_input_window() {
        let base = reg().lookup("claude-sonnet-4-5").expect("known");
        assert_eq!(base.max_input_tokens, 200_000);
        let widened = reg()
            .lookup_with_routing("claude-sonnet-4-5", &[ModelCapability::OneMillionContext])
            .expect("known");
        assert_eq!(widened.max_input_tokens, 1_000_000);
        // Other fields unchanged.
        assert_eq!(widened.max_output_tokens, base.max_output_tokens);
    }

    #[test]
    fn capabilities_serialize_roundtrip() {
        let caps = reg().lookup("gpt-4o").expect("known");
        let json = serde_json::to_string(&caps).expect("serialize");
        let back: ModelCapabilities = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(caps, back);
    }
}
