//! `oya-intelligence-subagent-runtime-kernel` — port-in-kernel substrate for
//! the per-facet subagent invocation that closes the
//! `subagent_runtime_pending=true` gap left by
//! `tools/oya-intelligence-pr-review-dispatcher-app` (IP-004),
//! `tools/oya-vcs-ci-fix-loop-dispatcher-app` (IP-005), and
//! `tools/oya-vcs-merge-queue-fix-loop-app` (IP-006).
//!
//! ## What lives here
//!
//! - The [`FacetPromptTemplate`] value type (frontmatter + body parsed
//!   from the per-facet `*.md` files under
//!   `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/`).
//! - The [`SubagentRequest`] / [`SubagentResponse`] value types.
//! - The [`SubagentPort`] trait — one method, `complete`; adapter
//!   implementations satisfy it (live HTTP via the existing
//!   `oya-intelligence-adapter-anthropic-api-*` substrate, plus the
//!   `MockSubagentPort` canonical deterministic-test path).
//! - The [`FacetFindingJson::render`] serializer that emits the
//!   exact `evidence/debate/<change_id>-<facet_id>-r1.json` shape
//!   consumed by IP-004's `parse_recommendation` + the
//!   `F-LANE-DEBATE-SUBCHECK` validator.
//!
//! ## What does NOT live here
//!
//! - HTTP transport, OpenBao secret resolution, filesystem I/O, model
//!   selection heuristics. Those belong in the
//!   `tools/oya-intelligence-subagent-runtime-app/` binary (one ring out)
//!   or in adapter crates (e.g. the live Anthropic HTTP path).
//!
//! ## Why a port-in-kernel
//!
//! Per `feedback_clean_architecture_requirements` (ADR-0056 12-layer
//! enum + clean-architecture inward-flow + port-in-kernel) the
//! per-facet subagent substrate is a port: the kernel defines the
//! shape, adapters realize it. This lets the deterministic-mock path
//! and the live Anthropic-API path share a single contract with no
//! conditional compilation in caller code.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::fmt;

use intelligence_account_kernel::SecretReference;

/// One facet's recommendation from one subagent invocation. Mirrors
/// the `final_recommendation` enum in the consensus debate protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacetRecommendation {
    /// Facet finds no policy/quality drift; PR may merge.
    Approve,
    /// Facet finds specific changes needed; fix-loop should consume them.
    ChangesRequested,
    /// Facet finds a fundamental issue; do not auto-retry.
    Reject,
}

impl FacetRecommendation {
    /// Wire string used in JSON evidence files and in IP-004's
    /// `parse_recommendation`.
    #[must_use]
    pub const fn wire_value(self) -> &'static str {
        match self {
            Self::Approve => "APPROVE",
            Self::ChangesRequested => "CHANGES_REQUESTED",
            Self::Reject => "REJECT",
        }
    }

    /// Parse the wire string. Accepts upper- and lower-case
    /// (the multispectrum spec emits upper-case; tests sometimes
    /// emit lower-case).
    pub fn from_wire(value: &str) -> Result<Self, SubagentError> {
        match value {
            "APPROVE" | "approve" => Ok(Self::Approve),
            "CHANGES_REQUESTED" | "changes_requested" => Ok(Self::ChangesRequested),
            "REJECT" | "reject" => Ok(Self::Reject),
            other => Err(SubagentError::InvalidRecommendation(other.to_owned())),
        }
    }
}

/// One facet's prompt template as authored under
/// `evidence/pipeline-maturity-glue/ip-004-pr-review/facets/<facet_id>.md`.
///
/// The template carries frontmatter (machine-readable key/value pairs)
/// plus a body (the actual prompt). Frontmatter MUST include the four
/// keys `facet_id`, `facet_name`, `lens`, and `severity_bar`; the parser
/// rejects templates missing any of those.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacetPromptTemplate {
    facet_id: String,
    facet_name: String,
    lens: String,
    severity_bar: String,
    body: String,
}

impl FacetPromptTemplate {
    /// Build a new template directly from validated parts. Prefer
    /// [`Self::parse`] when consuming on-disk `*.md` files; this
    /// constructor exists for tests + programmatic synthesis.
    pub fn new(
        facet_id: String,
        facet_name: String,
        lens: String,
        severity_bar: String,
        body: String,
    ) -> Result<Self, SubagentError> {
        if facet_id.trim().is_empty() {
            return Err(SubagentError::TemplateMissingField("facet_id"));
        }
        if facet_name.trim().is_empty() {
            return Err(SubagentError::TemplateMissingField("facet_name"));
        }
        if lens.trim().is_empty() {
            return Err(SubagentError::TemplateMissingField("lens"));
        }
        if severity_bar.trim().is_empty() {
            return Err(SubagentError::TemplateMissingField("severity_bar"));
        }
        if body.trim().is_empty() {
            return Err(SubagentError::TemplateMissingField("body"));
        }
        Ok(Self {
            facet_id,
            facet_name,
            lens,
            severity_bar,
            body,
        })
    }

    /// Parse the `*.md` representation produced by the IP-004 facet
    /// templates. The grammar is intentionally minimal (no YAML
    /// dependency at the kernel layer): the document opens with `---`,
    /// has one `key: value` per line (values are everything after the
    /// first colon, trimmed), closes with `---`, then the body is
    /// everything after.
    pub fn parse(raw: &str) -> Result<Self, SubagentError> {
        let trimmed = raw.trim_start_matches('\u{feff}');
        let mut lines = trimmed.lines();
        let first = lines.next().ok_or(SubagentError::TemplateEmpty)?;
        if first.trim() != "---" {
            return Err(SubagentError::TemplateMissingFrontmatter);
        }
        let mut fields: BTreeMap<String, String> = BTreeMap::new();
        let mut body_lines: Vec<String> = Vec::new();
        let mut in_frontmatter = true;
        for line in lines {
            if in_frontmatter {
                if line.trim() == "---" {
                    in_frontmatter = false;
                    continue;
                }
                if line.trim().is_empty() {
                    continue;
                }
                let (key, value) = match line.split_once(':') {
                    Some((k, v)) => (k.trim().to_owned(), v.trim().to_owned()),
                    None => continue,
                };
                fields.insert(key, value);
            } else {
                body_lines.push(line.to_owned());
            }
        }
        if in_frontmatter {
            return Err(SubagentError::TemplateMissingFrontmatter);
        }
        let facet_id = fields
            .remove("facet_id")
            .ok_or(SubagentError::TemplateMissingField("facet_id"))?;
        let facet_name = fields
            .remove("facet_name")
            .ok_or(SubagentError::TemplateMissingField("facet_name"))?;
        let lens = fields
            .remove("lens")
            .ok_or(SubagentError::TemplateMissingField("lens"))?;
        let severity_bar = fields
            .remove("severity_bar")
            .ok_or(SubagentError::TemplateMissingField("severity_bar"))?;
        let body = body_lines.join("\n");
        Self::new(facet_id, facet_name, lens, severity_bar, body)
    }

    #[must_use]
    pub fn facet_id(&self) -> &str {
        &self.facet_id
    }
    #[must_use]
    pub fn facet_name(&self) -> &str {
        &self.facet_name
    }
    #[must_use]
    pub fn lens(&self) -> &str {
        &self.lens
    }
    #[must_use]
    pub fn severity_bar(&self) -> &str {
        &self.severity_bar
    }
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Render the full system-prompt the runtime sends to the model.
    /// Includes facet name + lens + severity-bar contract + the body.
    #[must_use]
    pub fn render_system_prompt(&self) -> String {
        format!(
            "You are facet {facet_id} ({facet_name}) of the multispectrum-review v2.3.0 panel.\n\
             Lens: {lens}\n\
             Severity bar (HIGH-confidence only): {severity_bar}\n\n\
             You MUST emit a single final_recommendation ∈ {{APPROVE, CHANGES_REQUESTED, REJECT}}\n\
             on the last line of your response, prefixed by `final_recommendation: `.\n\
             You MAY emit narrative findings above that line.\n\n\
             ---\n\n\
             {body}",
            facet_id = self.facet_id,
            facet_name = self.facet_name,
            lens = self.lens,
            severity_bar = self.severity_bar,
            body = self.body,
        )
    }
}

/// One subagent invocation request. The runtime binary builds one of
/// these per facet per change, then hands it to the [`SubagentPort`].
#[derive(Debug, Clone)]
pub struct SubagentRequest {
    /// e.g. `F1_linus`, `A3_structure_adherence`, etc.
    /// data_class: INTERNAL_ONLY
    pub facet_id: String, // data_class: INTERNAL_ONLY
    /// `<tool>-<facet_id>-<change_id>` per feedback_multispectrum_review_v22.
    /// data_class: INTERNAL_ONLY
    pub reviewer_id: String, // data_class: INTERNAL_ONLY
    /// `<pr-number>` or `M01-P17-IP-004-pr42` style id.
    /// data_class: INTERNAL_ONLY
    pub change_id: String, // data_class: INTERNAL_ONLY
    /// System prompt (the rendered facet template).
    /// data_class: INTERNAL_ONLY
    pub system_prompt: String, // data_class: INTERNAL_ONLY
    /// User message — typically the PR diff + commit-history summary.
    /// data_class: INTERNAL_ONLY
    pub user_message: String, // data_class: INTERNAL_ONLY
    /// Opaque reference to the Anthropic API key, resolved by the
    /// adapter via OpenBao. The kernel never sees raw bytes.
    /// data_class: SECRET
    pub api_key_ref: SecretReference, // data_class: SECRET
    /// Model id (e.g. `claude-opus-4-7`).
    /// data_class: INTERNAL_ONLY
    pub model_id: String, // data_class: INTERNAL_ONLY
}

/// Response from one subagent invocation. The free-text body is what
/// the model returned; the recommendation is extracted from the
/// `final_recommendation: <wire>` sentinel on the last line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubagentResponse {
    /// data_class: INTERNAL_ONLY
    pub facet_id: String, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub reviewer_id: String, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub recommendation: FacetRecommendation, // data_class: INTERNAL_ONLY
    /// Full body (everything before the sentinel line). May be empty.
    /// data_class: INTERNAL_ONLY
    pub findings_body: String, // data_class: INTERNAL_ONLY
}

impl SubagentResponse {
    /// Build a response from a raw model output. The parser extracts
    /// the `final_recommendation: <wire>` sentinel from the last
    /// non-empty line; everything else is the findings body.
    pub fn from_model_output(
        facet_id: String,
        reviewer_id: String,
        raw: &str,
    ) -> Result<Self, SubagentError> {
        let mut sentinel_idx: Option<usize> = None;
        let mut recommendation: Option<FacetRecommendation> = None;
        let lines: Vec<&str> = raw.lines().collect();
        for (idx, line) in lines.iter().enumerate().rev() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("final_recommendation:") {
                recommendation = Some(FacetRecommendation::from_wire(rest.trim())?);
                sentinel_idx = Some(idx);
                break;
            }
            // The sentinel must be on the LAST non-empty line; if the
            // last non-empty line lacks the sentinel we surface that.
            return Err(SubagentError::ResponseSentinelMissing);
        }
        let recommendation = recommendation.ok_or(SubagentError::ResponseSentinelMissing)?;
        let body_end = sentinel_idx.unwrap_or(lines.len());
        let findings_body = lines
            .iter()
            .take(body_end)
            .copied()
            .collect::<Vec<&str>>()
            .join("\n")
            .trim()
            .to_owned();
        Ok(Self {
            facet_id,
            reviewer_id,
            recommendation,
            findings_body,
        })
    }
}

/// Errors surfaced by the runtime kernel + adapters. `thiserror`
/// would be a fine addition but the kernel keeps zero non-workspace
/// dependencies — `Display` is hand-rolled in the spirit of ADR-0083
/// Tier 1 (typed errors, no panics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubagentError {
    TemplateEmpty,
    TemplateMissingFrontmatter,
    TemplateMissingField(&'static str),
    InvalidRecommendation(String),
    ResponseSentinelMissing,
    AdapterFailed(String),
    SecretResolutionFailed(String),
    TransportFailed(String),
    ProviderRejected(String),
}

impl fmt::Display for SubagentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TemplateEmpty => write!(f, "facet template is empty"),
            Self::TemplateMissingFrontmatter => {
                write!(
                    f,
                    "facet template is missing the `---` frontmatter delimiters"
                )
            }
            Self::TemplateMissingField(field) => {
                write!(f, "facet template is missing required field `{field}`")
            }
            Self::InvalidRecommendation(value) => {
                write!(
                    f,
                    "invalid final_recommendation `{value}`; expected APPROVE | CHANGES_REQUESTED | REJECT"
                )
            }
            Self::ResponseSentinelMissing => write!(
                f,
                "model response missing the `final_recommendation: <wire>` sentinel on the last non-empty line"
            ),
            Self::AdapterFailed(reason) => write!(f, "subagent adapter failed: {reason}"),
            Self::SecretResolutionFailed(reason) => {
                write!(f, "secret resolution failed: {reason}")
            }
            Self::TransportFailed(reason) => write!(f, "transport failed: {reason}"),
            Self::ProviderRejected(reason) => write!(f, "provider rejected request: {reason}"),
        }
    }
}

impl std::error::Error for SubagentError {}

/// Port satisfied by every subagent adapter (live Anthropic HTTP,
/// deterministic mock, future OMC-teams / Codex / Gemini multi-model
/// pluralization). The kernel never calls Anthropic itself; this
/// trait is the substrate.
pub trait SubagentPort {
    /// Invoke the subagent for one facet and return the parsed response.
    fn complete(&self, request: &SubagentRequest) -> Result<SubagentResponse, SubagentError>;
}

/// Deterministic mock port. Per the IP brief hard-stop ("network egress
/// denied in test/CI environment → emit a runtime contract that lets
/// the API call be mocked deterministically in tests; NOT a stub —
/// the test path is canonical mock infrastructure").
///
/// Important: the mock is a CI smoke/fixture surface, not a quality
/// reviewer. Its default is [`FacetRecommendation::Approve`] so the
/// required check is not driven by meaningless facet-id hash noise. Tests
/// that need negative rollups use explicit first-line fixture directives
/// or in-memory overrides.
#[derive(Debug, Clone, Default)]
pub struct MockSubagentPort {
    /// Optional override per facet id; useful for end-to-end tests
    /// that need to inject a specific verdict mix.
    overrides: BTreeMap<String, FacetRecommendation>,
}

impl MockSubagentPort {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the deterministic recommendation for one facet id.
    pub fn with_override(mut self, facet_id: &str, recommendation: FacetRecommendation) -> Self {
        self.overrides.insert(facet_id.to_owned(), recommendation);
        self
    }
}

impl SubagentPort for MockSubagentPort {
    fn complete(&self, request: &SubagentRequest) -> Result<SubagentResponse, SubagentError> {
        let recommendation = if let Some(over) = self.overrides.get(&request.facet_id) {
            *over
        } else {
            mock_directive_recommendation(&request.user_message)
                .unwrap_or(FacetRecommendation::Approve)
        };
        let body = mock_findings_body(request, recommendation);
        Ok(SubagentResponse {
            facet_id: request.facet_id.clone(),
            reviewer_id: request.reviewer_id.clone(),
            recommendation,
            findings_body: body,
        })
    }
}

fn mock_directive_recommendation(user_message: &str) -> Option<FacetRecommendation> {
    user_message
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .and_then(mock_first_line_directive_recommendation)
}

fn mock_first_line_directive_recommendation(line: &str) -> Option<FacetRecommendation> {
    let lower = line.to_ascii_lowercase();
    let value = lower
        .strip_prefix("oya_mock_recommendation:")
        .map(str::trim)?;
    match value {
        "reject" => Some(FacetRecommendation::Reject),
        "changes_requested" | "changes-requested" => Some(FacetRecommendation::ChangesRequested),
        "approve" => Some(FacetRecommendation::Approve),
        _ => None,
    }
}

fn mock_findings_body(request: &SubagentRequest, recommendation: FacetRecommendation) -> String {
    let mode_note = match mock_directive_recommendation(&request.user_message) {
        Some(_) => "explicit first-line fixture directive selected this recommendation",
        None => {
            "no explicit fixture directive was present; defaulted APPROVE as deterministic CI smoke"
        }
    };
    format!(
        "deterministic-mock CI smoke for facet {facet} (change {change}); recommendation={recommendation}; {mode_note}; no content-quality claim is made by this mock path",
        facet = request.facet_id,
        change = request.change_id,
        recommendation = recommendation.wire_value(),
    )
}

/// Serializer for the per-facet `<facet>.json` evidence file consumed
/// by IP-004's `tools/oya-intelligence-pr-review-dispatcher-app::parse_recommendation`.
/// The shape matches `consensus_debate_protocol.rounds.round_1_independent.required_keys`
/// from the multispectrum review protocol.
///
/// We avoid serde here for two reasons: (a) zero non-workspace deps
/// at the kernel layer; (b) the output is small + deterministic so a
/// hand-rolled writer is auditable.
pub struct FacetFindingJson;

impl FacetFindingJson {
    /// Render the `r1.json` shape. `emitted_at_unix` is taken from the
    /// caller (not from the system clock here — clock injection keeps
    /// the kernel pure).
    #[must_use]
    pub fn render(response: &SubagentResponse, emitted_at_unix: u64) -> String {
        let mut buf = String::new();
        buf.push_str("{\n");
        buf.push_str("  \"schema\": \"oya-multispectrum-facet-finding/v1\",\n");
        buf.push_str(&format!(
            "  \"facet_id\": \"{}\",\n",
            json_escape(&response.facet_id)
        ));
        buf.push_str(&format!(
            "  \"reviewer_id\": \"{}\",\n",
            json_escape(&response.reviewer_id)
        ));
        buf.push_str(&format!(
            "  \"final_recommendation\": \"{}\",\n",
            response.recommendation.wire_value()
        ));
        buf.push_str(&format!("  \"emitted_at_unix\": {emitted_at_unix},\n"));
        buf.push_str(&format!(
            "  \"findings_body\": \"{}\"\n",
            json_escape(&response.findings_body)
        ));
        buf.push_str("}\n");
        buf
    }
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_template_md() -> &'static str {
        "---\n\
         facet_id: F1_linus\n\
         facet_name: F1 Linus Critic\n\
         lens: kernel-quality + maintainability + bullshit-detection\n\
         severity_bar: REJECT on architectural regressions; CHANGES_REQUESTED on naming/clarity drift; APPROVE otherwise\n\
         ---\n\
         You are the Linus-style critic. Read the PR diff and identify:\n\
         - dead code\n\
         - sloppy abstractions\n\
         - silent regressions\n"
    }

    #[test]
    fn facet_recommendation_wire_round_trip() {
        for rec in [
            FacetRecommendation::Approve,
            FacetRecommendation::ChangesRequested,
            FacetRecommendation::Reject,
        ] {
            assert_eq!(FacetRecommendation::from_wire(rec.wire_value()), Ok(rec));
        }
    }

    #[test]
    fn facet_recommendation_rejects_unknown_wire() {
        assert_eq!(
            FacetRecommendation::from_wire("MAYBE"),
            Err(SubagentError::InvalidRecommendation("MAYBE".into()))
        );
    }

    #[test]
    fn parse_template_extracts_all_four_required_fields() {
        let template = FacetPromptTemplate::parse(sample_template_md()).unwrap();
        assert_eq!(template.facet_id(), "F1_linus");
        assert_eq!(template.facet_name(), "F1 Linus Critic");
        assert!(template.lens().contains("kernel-quality"));
        assert!(template.severity_bar().contains("REJECT"));
        assert!(template.body().contains("Read the PR diff"));
    }

    #[test]
    fn parse_template_rejects_missing_frontmatter() {
        let raw = "no frontmatter here\nbody body body\n";
        assert!(matches!(
            FacetPromptTemplate::parse(raw),
            Err(SubagentError::TemplateMissingFrontmatter)
        ));
    }

    #[test]
    fn parse_template_rejects_missing_required_field() {
        let raw = "---\nfacet_id: F1_linus\nfacet_name: x\nlens: y\n---\nbody\n";
        assert_eq!(
            FacetPromptTemplate::parse(raw),
            Err(SubagentError::TemplateMissingField("severity_bar"))
        );
    }

    #[test]
    fn rendered_system_prompt_contains_facet_metadata_and_sentinel_contract() {
        let template = FacetPromptTemplate::parse(sample_template_md()).unwrap();
        let prompt = template.render_system_prompt();
        assert!(prompt.contains("F1_linus"));
        assert!(prompt.contains("F1 Linus Critic"));
        assert!(prompt.contains("final_recommendation:"));
        assert!(prompt.contains("APPROVE"));
    }

    #[test]
    fn response_from_model_output_extracts_sentinel() {
        let raw = "\
            finding line 1\n\
            finding line 2\n\
            \n\
            final_recommendation: APPROVE\n";
        let response =
            SubagentResponse::from_model_output("F1_linus".into(), "rid".into(), raw).unwrap();
        assert_eq!(response.recommendation, FacetRecommendation::Approve);
        assert!(response.findings_body.contains("finding line 1"));
        assert!(!response.findings_body.contains("final_recommendation"));
    }

    #[test]
    fn response_rejects_missing_sentinel() {
        let raw = "just findings, no sentinel\n";
        assert_eq!(
            SubagentResponse::from_model_output("F1".into(), "rid".into(), raw),
            Err(SubagentError::ResponseSentinelMissing)
        );
    }

    #[test]
    fn mock_port_is_deterministic_across_runs() {
        let port = MockSubagentPort::new();
        let sref = SecretReference::new("sref://test-anthropic-key".into()).unwrap();
        let request = SubagentRequest {
            facet_id: "F1_linus".into(),
            reviewer_id: "claude-critic-F1_linus-pr1".into(),
            change_id: "pr1".into(),
            system_prompt: "sys".into(),
            user_message: "diff".into(),
            api_key_ref: sref.clone(),
            model_id: "claude-opus-4-7".into(),
        };
        let r1 = port.complete(&request).unwrap();
        let r2 = port.complete(&request).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn mock_port_defaults_to_approve_without_fixture_directive() {
        let port = MockSubagentPort::new();
        let sref = SecretReference::new("sref://test-anthropic-key".into()).unwrap();
        let request = SubagentRequest {
            facet_id: "F3_adversarial".into(),
            reviewer_id: "github-actions-agent-review-F3-pr1".into(),
            change_id: "pr1".into(),
            system_prompt: "sys mentions REJECT as a severity option".into(),
            user_message:
                "ordinary PR diff without fixture directives\n+oya_mock_recommendation: reject"
                    .into(),
            api_key_ref: sref,
            model_id: "claude-opus-4-7".into(),
        };
        let response = port.complete(&request).unwrap();
        assert_eq!(response.recommendation, FacetRecommendation::Approve);
        assert!(response.findings_body.contains("deterministic CI smoke"));
        assert!(response.findings_body.contains("no content-quality claim"));
    }

    #[test]
    fn mock_port_honors_negative_fixture_directives() {
        let port = MockSubagentPort::new();
        let sref = SecretReference::new("sref://test-anthropic-key".into()).unwrap();
        let reject = SubagentRequest {
            facet_id: "F7_security".into(),
            reviewer_id: "rid-reject".into(),
            change_id: "pr1".into(),
            system_prompt: "sys".into(),
            user_message: "oya_mock_recommendation: reject\nfixture body".into(),
            api_key_ref: sref.clone(),
            model_id: "claude-opus-4-7".into(),
        };
        assert_eq!(
            port.complete(&reject).unwrap().recommendation,
            FacetRecommendation::Reject
        );

        let changes_requested = SubagentRequest {
            facet_id: "F5_quality".into(),
            reviewer_id: "rid-changes".into(),
            change_id: "pr1".into(),
            system_prompt: "sys".into(),
            user_message: "oya_mock_recommendation: changes_requested\nfixture body".into(),
            api_key_ref: sref,
            model_id: "claude-opus-4-7".into(),
        };
        assert_eq!(
            port.complete(&changes_requested).unwrap().recommendation,
            FacetRecommendation::ChangesRequested
        );
    }

    #[test]
    fn mock_port_ignores_fixture_words_embedded_in_diff_body() {
        let port = MockSubagentPort::new();
        let sref = SecretReference::new("sref://test-anthropic-key".into()).unwrap();
        let request = SubagentRequest {
            facet_id: "F1_linus".into(),
            reviewer_id: "rid".into(),
            change_id: "pr1".into(),
            system_prompt: "sys".into(),
            user_message: "# PR review input\n```diff\n+oya_mock_recommendation: reject\n```"
                .into(),
            api_key_ref: sref,
            model_id: "claude-opus-4-7".into(),
        };
        let response = port.complete(&request).unwrap();
        assert_eq!(response.recommendation, FacetRecommendation::Approve);
        assert!(
            response
                .findings_body
                .contains("no explicit fixture directive")
        );
    }

    #[test]
    fn mock_port_respects_override() {
        let port =
            MockSubagentPort::new().with_override("F7_security", FacetRecommendation::Reject);
        let sref = SecretReference::new("sref://k".into()).unwrap();
        let request = SubagentRequest {
            facet_id: "F7_security".into(),
            reviewer_id: "rid".into(),
            change_id: "pr1".into(),
            system_prompt: "sys".into(),
            user_message: "diff".into(),
            api_key_ref: sref,
            model_id: "claude-opus-4-7".into(),
        };
        let response = port.complete(&request).unwrap();
        assert_eq!(response.recommendation, FacetRecommendation::Reject);
    }

    #[test]
    fn finding_json_shape_matches_dispatcher_contract() {
        let response = SubagentResponse {
            facet_id: "F1_linus".into(),
            reviewer_id: "claude-critic-F1_linus-pr1".into(),
            recommendation: FacetRecommendation::ChangesRequested,
            findings_body: "found drift in module foo".into(),
        };
        let json = FacetFindingJson::render(&response, 1_715_000_000);
        // The dispatcher reads `final_recommendation` + `reviewer_id`.
        assert!(json.contains("\"facet_id\": \"F1_linus\""));
        assert!(json.contains("\"reviewer_id\": \"claude-critic-F1_linus-pr1\""));
        assert!(json.contains("\"final_recommendation\": \"CHANGES_REQUESTED\""));
        assert!(json.contains("\"emitted_at_unix\": 1715000000"));
    }

    #[test]
    fn subagent_error_display_distinct() {
        let errors = [
            SubagentError::TemplateEmpty,
            SubagentError::TemplateMissingFrontmatter,
            SubagentError::TemplateMissingField("facet_id"),
            SubagentError::InvalidRecommendation("MAYBE".into()),
            SubagentError::ResponseSentinelMissing,
            SubagentError::AdapterFailed("x".into()),
            SubagentError::SecretResolutionFailed("y".into()),
            SubagentError::TransportFailed("z".into()),
            SubagentError::ProviderRejected("w".into()),
        ];
        let messages: std::collections::HashSet<String> =
            errors.iter().map(|e| format!("{e}")).collect();
        assert_eq!(messages.len(), errors.len());
    }
}
