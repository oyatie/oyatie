//! Foundry glossary cross-document coverage fitness kernel.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlossaryTerm {
    pub term: String,             // data_class: INTERNAL_ONLY
    pub source: String,           // data_class: INTERNAL_ONLY
    pub cross_doc_required: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlossaryCoverageReport {
    pub terms_checked: usize,           // data_class: INTERNAL_ONLY
    pub cross_doc_terms_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlossaryCoverageError {
    NoTerms,
    EmptyTerm { source: String },
    TermMissingGlossaryEntry { term: String, source: String },
    TermMissingCrossDocCoverage { term: String, source: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CanonicalTerm {
    term: String,
    source: String,
    cross_doc_required: bool,
}

/// Validate that machine-readable glossary terms are mirrored in the human
/// glossary and that active terms appear in at least one non-glossary doc.
pub fn validate_glossary_cross_doc_coverage<T, D>(
    terms: T,
    glossary_contents: &str,
    cross_doc_contents: D,
) -> Result<GlossaryCoverageReport, GlossaryCoverageError>
where
    T: IntoIterator<Item = GlossaryTerm>,
    D: IntoIterator,
    D::Item: AsRef<str>,
{
    let terms = canonical_terms(terms)?;
    if terms.is_empty() {
        return Err(GlossaryCoverageError::NoTerms);
    }
    let glossary_tokens = term_tokens(glossary_contents);
    let cross_doc_contents = cross_doc_contents
        .into_iter()
        .map(|contents| contents.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let cross_doc_tokens = term_tokens(&cross_doc_contents);

    let mut cross_doc_terms_checked = 0;
    for term in terms.values() {
        if !contains_term_tokens(&glossary_tokens, &term.term) {
            return Err(GlossaryCoverageError::TermMissingGlossaryEntry {
                term: term.term.clone(),
                source: term.source.clone(),
            });
        }
        if term.cross_doc_required {
            cross_doc_terms_checked += 1;
            if !contains_term_tokens(&cross_doc_tokens, &term.term) {
                return Err(GlossaryCoverageError::TermMissingCrossDocCoverage {
                    term: term.term.clone(),
                    source: term.source.clone(),
                });
            }
        }
    }

    Ok(GlossaryCoverageReport {
        terms_checked: terms.len(),
        cross_doc_terms_checked,
    })
}

fn canonical_terms<T>(terms: T) -> Result<BTreeMap<String, CanonicalTerm>, GlossaryCoverageError>
where
    T: IntoIterator<Item = GlossaryTerm>,
{
    let mut canonical = BTreeMap::new();
    for term in terms {
        if term_tokens(&term.term).is_empty() {
            return Err(GlossaryCoverageError::EmptyTerm {
                source: term.source,
            });
        }
        let key = normalize_key(&term.term);
        canonical
            .entry(key)
            .and_modify(|existing: &mut CanonicalTerm| {
                existing.cross_doc_required |= term.cross_doc_required;
            })
            .or_insert(CanonicalTerm {
                term: term.term,
                source: term.source,
                cross_doc_required: term.cross_doc_required,
            });
    }
    Ok(canonical)
}

fn contains_term_tokens(content_tokens: &[String], term: &str) -> bool {
    let term_tokens = term_tokens(term);
    if term_tokens.is_empty() {
        return false;
    }
    let mut search_start = 0;
    for term_token in term_tokens {
        let Some(offset) = content_tokens[search_start..]
            .iter()
            .position(|content_token| content_token == &term_token)
        else {
            return false;
        };
        search_start += offset + 1;
    }
    true
}

fn normalize_key(value: &str) -> String {
    term_tokens(value).join(" ")
}

fn term_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for character in value.chars() {
        if character.is_alphanumeric() {
            for lowered in character.to_lowercase() {
                current.push(lowered);
            }
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_term() {
        assert_eq!(
            validate_glossary_cross_doc_coverage([term("   ", "machine:empty", true)], "", [""],),
            Err(GlossaryCoverageError::EmptyTerm {
                source: "machine:empty".into()
            })
        );
    }

    #[test]
    fn rejects_machine_term_missing_from_glossary_markdown() {
        assert_eq!(
            validate_glossary_cross_doc_coverage(
                [term("Object Graph", "machine:oyatie_specific", true)],
                "# Glossary\n",
                ["Object Graph appears in DESIGN."],
            ),
            Err(GlossaryCoverageError::TermMissingGlossaryEntry {
                term: "Object Graph".into(),
                source: "machine:oyatie_specific".into()
            })
        );
    }

    #[test]
    fn rejects_active_term_missing_from_non_glossary_docs() {
        assert_eq!(
            validate_glossary_cross_doc_coverage(
                [term(
                    "tenant isolation models",
                    "machine:industry_standard.saas",
                    true
                )],
                "tenant isolation models",
                ["tenant isolation is discussed, but the model set is not named."],
            ),
            Err(GlossaryCoverageError::TermMissingCrossDocCoverage {
                term: "tenant isolation models".into(),
                source: "machine:industry_standard.saas".into()
            })
        );
    }

    #[test]
    fn does_not_require_retired_terms_outside_the_glossary() {
        assert_eq!(
            validate_glossary_cross_doc_coverage(
                [term(
                    "CUG / Closed-User-Group",
                    "machine:retired_terms",
                    false
                )],
                "Deprecated: CUG, Closed-User-Group.",
                ["Team is the active term."],
            ),
            Ok(GlossaryCoverageReport {
                terms_checked: 1,
                cross_doc_terms_checked: 0,
            })
        );
    }

    #[test]
    fn accepts_case_and_punctuation_equivalent_coverage_and_dedupes_terms() {
        assert_eq!(
            validate_glossary_cross_doc_coverage(
                [
                    term("HWP/HWPX", "machine:compliance_kr", true),
                    term("HWP / HWPX", "machine:acronym", false),
                ],
                "HWP / HWPX documents are KR government defaults.",
                ["The HWP and HWPX import surface is file-scoped."],
            ),
            Ok(GlossaryCoverageReport {
                terms_checked: 1,
                cross_doc_terms_checked: 1,
            })
        );
    }

    fn term(term: &str, source: &str, cross_doc_required: bool) -> GlossaryTerm {
        GlossaryTerm {
            term: term.into(),
            source: source.into(),
            cross_doc_required,
        }
    }
}
