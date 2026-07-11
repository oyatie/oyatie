//! Gate-coverage check 1/3 (born-advisory): ADR prose ⇄ front-matter status
//! agreement.
//!
//! The #1327 defect class (a): an ADR whose BODY prose asserts a lifecycle status
//! that contradicts its own front-matter `status:` — e.g. an `Accepted` ADR whose
//! prose says it "stays Proposed". No born-blocking §5.2 code keys on prose, so
//! the contradiction shipped. This check flags it.
//!
//! Pure evaluator over a caller-assembled corpus; the phrase list is POLICY DATA
//! (`prose-status-agreement-policy.json`), never a scanner branch, so the matcher
//! is extended by a reviewed DATA edit. The matcher is deliberately CONSERVATIVE:
//! only high-confidence literal phrases are flagged (a lower-cased substring of
//! the body), keyed to the front-matter status they contradict, so a false
//! positive is a reviewed DATA choice rather than a heuristic guess. Anything the
//! matcher flags on the live corpus that is a benign occurrence is absorbed by the
//! born-advisory frozen baseline; a NEW contradiction is a regression that blocks.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::Finding;

/// Validator id recorded by the prose⇄front-matter status contract.
pub const PROSE_STATUS_AGREEMENT_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/adr-prose-frontmatter-status-agreement";

/// The advisory violation code this check emits (NOT a born-blocking §5.2 code).
pub const PROSE_STATUS_CONTRADICTION_CODE: &str = "adr_prose_status_contradiction";

fn contradiction(key: &str) -> Finding {
    Finding::new(PROSE_STATUS_CONTRADICTION_CODE, key)
}

/// Normalize a status to its lower-cased leading token, so `Accepted`,
/// `accepted`, and `Accepted (amendment)` all compare equal to the rule key
/// `Accepted`. Empty/whitespace stays empty.
fn normalized_status_token(status: &str) -> String {
    status
        .trim()
        .split(|c: char| c.is_whitespace() || c == '(')
        .find(|token| !token.is_empty())
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Evaluate the prose⇄front-matter status agreement corpus:
///
/// ```jsonc
/// {
///   "adrs": [
///     { "id": "ADR-0515", "frontmatter_status": "Accepted", "body": "<md body>" }
///   ]
/// }
/// ```
///
/// against the phrase policy:
///
/// ```jsonc
/// {
///   "status_contradiction_rules": [
///     { "frontmatter_status": "Accepted",
///       "forbidden_body_phrases": ["stays proposed", "remains proposed"] }
///   ]
/// }
/// ```
///
/// An ADR whose front-matter status matches a rule and whose (lower-cased) body
/// contains one of that rule's (lower-cased) forbidden phrases yields a Finding
/// keyed `{id}@{phrase}`. A malformed policy fails closed with a single
/// `<malformed-prose-status-policy>` finding so a broken policy can never silently
/// pass.
pub fn evaluate_adr_prose_frontmatter_status(corpus: &Value, policy: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(rules) = policy
        .get("status_contradiction_rules")
        .and_then(Value::as_array)
    else {
        findings.insert(contradiction("<malformed-prose-status-policy>"));
        return findings;
    };
    if rules.is_empty() {
        findings.insert(contradiction("<empty-prose-status-policy>"));
        return findings;
    }

    // Index the (lower-cased) forbidden phrases by the normalized status token
    // they contradict. A malformed rule fails closed.
    let mut phrases_by_status: Vec<(String, Vec<String>)> = Vec::new();
    for rule in rules {
        let Some(status) = rule.get("frontmatter_status").and_then(Value::as_str) else {
            findings.insert(contradiction("<malformed-prose-status-policy>"));
            return findings;
        };
        let Some(phrases) = rule.get("forbidden_body_phrases").and_then(Value::as_array) else {
            findings.insert(contradiction("<malformed-prose-status-policy>"));
            return findings;
        };
        let mut lowered = Vec::new();
        for phrase in phrases {
            let Some(text) = phrase.as_str().map(str::trim).filter(|p| !p.is_empty()) else {
                findings.insert(contradiction("<malformed-prose-status-policy>"));
                return findings;
            };
            lowered.push(text.to_ascii_lowercase());
        }
        phrases_by_status.push((normalized_status_token(status), lowered));
    }

    let adrs = corpus
        .get("adrs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for adr in adrs {
        let (Some(id), Some(status), Some(body)) = (
            adr.get("id").and_then(Value::as_str),
            adr.get("frontmatter_status").and_then(Value::as_str),
            adr.get("body").and_then(Value::as_str),
        ) else {
            // A row the caller could not fully assemble is not evaluable prose;
            // the corpus assembler owns completeness. Skip silently.
            continue;
        };
        let status_token = normalized_status_token(status);
        let lowered_body = body.to_ascii_lowercase();
        for (rule_status, phrases) in &phrases_by_status {
            if *rule_status != status_token {
                continue;
            }
            for phrase in phrases {
                if lowered_body.contains(phrase) {
                    findings.insert(contradiction(&format!("{id}@{phrase}")));
                }
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::json;

    use super::*;

    fn policy() -> Value {
        json!({
            "status_contradiction_rules": [
                {
                    "frontmatter_status": "Accepted",
                    "forbidden_body_phrases": ["stays proposed", "remains proposed"]
                }
            ]
        })
    }

    fn keys(findings: &BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|f| f.key.clone()).collect()
    }

    #[test]
    fn accepted_adr_asserting_it_stays_proposed_is_flagged() {
        let corpus = json!({
            "adrs": [
                { "id": "ADR-0999", "frontmatter_status": "Accepted",
                  "body": "This decision stays Proposed until the follow-up lands." }
            ]
        });
        let findings = evaluate_adr_prose_frontmatter_status(&corpus, &policy());
        assert_eq!(keys(&findings), vec!["ADR-0999@stays proposed".to_owned()]);
    }

    #[test]
    fn accepted_amendment_status_token_still_matches_the_accepted_rule() {
        let corpus = json!({
            "adrs": [
                { "id": "ADR-0999", "frontmatter_status": "Accepted (amendment)",
                  "body": "It remains Proposed for now." }
            ]
        });
        let findings = evaluate_adr_prose_frontmatter_status(&corpus, &policy());
        assert_eq!(
            keys(&findings),
            vec!["ADR-0999@remains proposed".to_owned()]
        );
    }

    #[test]
    fn a_clean_accepted_adr_is_green() {
        let corpus = json!({
            "adrs": [
                { "id": "ADR-0999", "frontmatter_status": "Accepted",
                  "body": "This decision is accepted and enforced." }
            ]
        });
        assert!(evaluate_adr_prose_frontmatter_status(&corpus, &policy()).is_empty());
    }

    #[test]
    fn a_proposed_adr_that_says_it_stays_proposed_is_consistent_not_flagged() {
        // The phrase only contradicts an ACCEPTED front-matter; a Proposed ADR
        // saying it stays Proposed agrees with itself.
        let corpus = json!({
            "adrs": [
                { "id": "ADR-0999", "frontmatter_status": "Proposed",
                  "body": "This decision stays Proposed until review." }
            ]
        });
        assert!(evaluate_adr_prose_frontmatter_status(&corpus, &policy()).is_empty());
    }

    #[test]
    fn malformed_policy_fails_closed() {
        let corpus = json!({ "adrs": [] });
        let findings = evaluate_adr_prose_frontmatter_status(&corpus, &json!({}));
        assert_eq!(
            keys(&findings),
            vec!["<malformed-prose-status-policy>".to_owned()]
        );
    }

    #[test]
    fn every_finding_uses_the_advisory_code() {
        let corpus = json!({
            "adrs": [
                { "id": "ADR-0999", "frontmatter_status": "Accepted", "body": "stays proposed" }
            ]
        });
        let findings = evaluate_adr_prose_frontmatter_status(&corpus, &policy());
        assert!(!findings.is_empty());
        for finding in &findings {
            assert_eq!(finding.code, PROSE_STATUS_CONTRADICTION_CODE);
        }
    }
}
