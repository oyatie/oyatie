//! Foundry glossary vocabulary-hygiene fitness kernel.

use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VocabularyDocument {
    pub path: String,           // data_class: INTERNAL_ONLY
    pub contents: String,       // data_class: INTERNAL_ONLY
    pub forensic_allowed: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlossaryVocabularyReport {
    pub documents_checked: usize,        // data_class: INTERNAL_ONLY
    pub casing_warnings: usize,          // data_class: INTERNAL_ONLY
    pub uncited_acronym_warnings: usize, // data_class: INTERNAL_ONLY
    pub warnings: Vec<GlossaryVocabularyWarning>, // data_class: INTERNAL_ONLY
    pub warning_sources: Vec<GlossaryVocabularyWarningSource>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IgnoredUppercaseWord {
    pub token: String,     // data_class: INTERNAL_ONLY
    pub rationale: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GlossaryVocabularyWarning {
    pub kind: GlossaryVocabularyWarningKind, // data_class: INTERNAL_ONLY
    pub token: String,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GlossaryVocabularyWarningSource {
    pub warning: GlossaryVocabularyWarning, // data_class: INTERNAL_ONLY
    pub path: String,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GlossaryVocabularyWarningKind {
    CasingVariant,
    UncitedAcronym,
}

impl GlossaryVocabularyWarning {
    pub fn id(&self) -> String {
        format!("{}\t{}", self.kind.as_str(), self.token)
    }
}

impl GlossaryVocabularyWarningKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CasingVariant => "casing-variant",
            Self::UncitedAcronym => "uncited-acronym",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "casing-variant" => Some(Self::CasingVariant),
            "uncited-acronym" => Some(Self::UncitedAcronym),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlossaryVocabularyError {
    NoDocuments,
    ForbiddenToken { path: String, token: String },
    DuplicateIgnoredUppercaseWord { token: String },
    InvalidIgnoredUppercaseWord { token: String },
    StaleIgnoredUppercaseWord { token: String },
    DuplicateBaselineWarning { warning_id: String },
    NewWarningOutsideBaseline { warning_id: String },
    StaleBaselineWarning { warning_id: String },
}

const FORBIDDEN_TOKENS: &[&str] = &[
    "MVP",
    "milestone-zero",
    "milestone-one",
    "CUG",
    "Closed-User-Group",
];

const RFC2119_NORMATIVE_KEYWORDS: &[&str] = &[
    "MUST",
    "NOT",
    "REQUIRED",
    "SHALL",
    "SHOULD",
    "RECOMMENDED",
    "MAY",
    "OPTIONAL",
];

const PLACEHOLDER_DEBT_TOKENS: &[&str] = &["TBD", "TODO"];

pub fn validate_glossary_vocabulary_hygiene<D, A>(
    documents: D,
    allowed_acronyms: A,
) -> Result<GlossaryVocabularyReport, GlossaryVocabularyError>
where
    D: IntoIterator<Item = VocabularyDocument>,
    A: IntoIterator,
    A::Item: AsRef<str>,
{
    validate_glossary_vocabulary_hygiene_with_ignored_words(
        documents,
        allowed_acronyms,
        std::iter::empty::<IgnoredUppercaseWord>(),
    )
}

pub fn validate_glossary_vocabulary_hygiene_with_ignored_words<D, A, I>(
    documents: D,
    allowed_acronyms: A,
    ignored_uppercase_words: I,
) -> Result<GlossaryVocabularyReport, GlossaryVocabularyError>
where
    D: IntoIterator<Item = VocabularyDocument>,
    A: IntoIterator,
    A::Item: AsRef<str>,
    I: IntoIterator<Item = IgnoredUppercaseWord>,
{
    let allowed_acronyms = allowed_acronyms
        .into_iter()
        .map(|acronym| acronym.as_ref().trim().to_ascii_uppercase())
        .filter(|acronym| !acronym.is_empty())
        .collect::<BTreeSet<_>>();
    let ignored_uppercase_words = ignored_uppercase_word_set(ignored_uppercase_words)?;
    let mut documents_checked = 0;
    let mut casing_warnings = 0;
    let mut casing_variants = BTreeSet::new();
    let mut uncited_acronyms = BTreeSet::new();
    let mut used_ignored_uppercase_words = BTreeSet::new();
    let mut warning_sources = BTreeSet::new();

    for document in documents {
        documents_checked += 1;
        let prose = markdown_prose(&document.contents);
        if document.forensic_allowed {
            continue;
        }
        if let Some(token) = first_forbidden_token(&prose) {
            return Err(GlossaryVocabularyError::ForbiddenToken {
                path: document.path,
                token,
            });
        }
        for casing_variant in casing_warning_variants(&prose) {
            casing_warnings += 1;
            casing_variants.insert(casing_variant.clone());
            warning_sources.insert(GlossaryVocabularyWarningSource {
                warning: GlossaryVocabularyWarning {
                    kind: GlossaryVocabularyWarningKind::CasingVariant,
                    token: casing_variant,
                },
                path: document.path.clone(),
            });
        }
        for acronym in acronym_candidates(&prose) {
            if ignored_uppercase_words.contains(&acronym) {
                used_ignored_uppercase_words.insert(acronym);
            } else if !allowed_acronyms.contains(&acronym) {
                uncited_acronyms.insert(acronym.clone());
                warning_sources.insert(GlossaryVocabularyWarningSource {
                    warning: GlossaryVocabularyWarning {
                        kind: GlossaryVocabularyWarningKind::UncitedAcronym,
                        token: acronym,
                    },
                    path: document.path.clone(),
                });
            }
        }
    }

    if documents_checked == 0 {
        return Err(GlossaryVocabularyError::NoDocuments);
    }
    if let Some(token) = ignored_uppercase_words
        .difference(&used_ignored_uppercase_words)
        .next()
    {
        return Err(GlossaryVocabularyError::StaleIgnoredUppercaseWord {
            token: token.clone(),
        });
    }

    let warnings = casing_variants
        .into_iter()
        .map(|token| GlossaryVocabularyWarning {
            kind: GlossaryVocabularyWarningKind::CasingVariant,
            token,
        })
        .chain(
            uncited_acronyms
                .iter()
                .map(|token| GlossaryVocabularyWarning {
                    kind: GlossaryVocabularyWarningKind::UncitedAcronym,
                    token: token.clone(),
                }),
        )
        .collect::<Vec<_>>();

    Ok(GlossaryVocabularyReport {
        documents_checked,
        casing_warnings,
        uncited_acronym_warnings: uncited_acronyms.len(),
        warnings,
        warning_sources: warning_sources.into_iter().collect(),
    })
}

pub fn validate_glossary_vocabulary_hygiene_with_baseline<D, A, B>(
    documents: D,
    allowed_acronyms: A,
    baseline_warnings: B,
) -> Result<GlossaryVocabularyReport, GlossaryVocabularyError>
where
    D: IntoIterator<Item = VocabularyDocument>,
    A: IntoIterator,
    A::Item: AsRef<str>,
    B: IntoIterator<Item = GlossaryVocabularyWarning>,
{
    validate_glossary_vocabulary_hygiene_with_baseline_and_ignored_words(
        documents,
        allowed_acronyms,
        std::iter::empty::<IgnoredUppercaseWord>(),
        baseline_warnings,
    )
}

pub fn validate_glossary_vocabulary_hygiene_with_baseline_and_ignored_words<D, A, I, B>(
    documents: D,
    allowed_acronyms: A,
    ignored_uppercase_words: I,
    baseline_warnings: B,
) -> Result<GlossaryVocabularyReport, GlossaryVocabularyError>
where
    D: IntoIterator<Item = VocabularyDocument>,
    A: IntoIterator,
    A::Item: AsRef<str>,
    I: IntoIterator<Item = IgnoredUppercaseWord>,
    B: IntoIterator<Item = GlossaryVocabularyWarning>,
{
    let baseline = baseline_set(baseline_warnings)?;
    let report = validate_glossary_vocabulary_hygiene_with_ignored_words(
        documents,
        allowed_acronyms,
        ignored_uppercase_words,
    )?;
    let current = report.warnings.iter().cloned().collect::<BTreeSet<_>>();

    if let Some(warning) = current.difference(&baseline).next() {
        return Err(GlossaryVocabularyError::NewWarningOutsideBaseline {
            warning_id: warning.id(),
        });
    }
    if let Some(warning) = baseline.difference(&current).next() {
        return Err(GlossaryVocabularyError::StaleBaselineWarning {
            warning_id: warning.id(),
        });
    }

    Ok(report)
}

fn ignored_uppercase_word_set<I>(
    ignored_uppercase_words: I,
) -> Result<BTreeSet<String>, GlossaryVocabularyError>
where
    I: IntoIterator<Item = IgnoredUppercaseWord>,
{
    let mut words = BTreeSet::new();
    for word in ignored_uppercase_words {
        let token = word.token.trim().to_string();
        let rationale = word.rationale.trim();
        // ADR-0018's uncited-acronym warning is a cleanup ratchet. Explicitly
        // ignored uppercase prose words are allowed only when they are ordinary
        // alphabetic words with a reviewable rationale, so this list cannot
        // bury retired vocabulary, artifact IDs, or empty rationales.
        if token.is_empty()
            || rationale.is_empty()
            || FORBIDDEN_TOKENS.contains(&token.as_str())
            || !is_acronym_candidate(&token)
            || !token
                .chars()
                .all(|character| character.is_ascii_uppercase())
        {
            return Err(GlossaryVocabularyError::InvalidIgnoredUppercaseWord { token });
        }
        if !words.insert(token.clone()) {
            return Err(GlossaryVocabularyError::DuplicateIgnoredUppercaseWord { token });
        }
    }
    Ok(words)
}

fn baseline_set<B>(
    warnings: B,
) -> Result<BTreeSet<GlossaryVocabularyWarning>, GlossaryVocabularyError>
where
    B: IntoIterator<Item = GlossaryVocabularyWarning>,
{
    let mut baseline = BTreeSet::new();
    for warning in warnings {
        if !baseline.insert(warning.clone()) {
            return Err(GlossaryVocabularyError::DuplicateBaselineWarning {
                warning_id: warning.id(),
            });
        }
    }
    Ok(baseline)
}

fn first_forbidden_token(prose: &str) -> Option<String> {
    for line in prose.lines() {
        for token in FORBIDDEN_TOKENS {
            if contains_forbidden_word(line, token) {
                return Some((*token).into());
            }
        }
    }
    None
}

fn contains_forbidden_word(line: &str, token: &str) -> bool {
    let lower_line = line.to_ascii_lowercase();
    let lower_token = token.to_ascii_lowercase();
    let mut search_start = 0;
    while let Some(offset) = lower_line[search_start..].find(&lower_token) {
        let start = search_start + offset;
        let end = start + lower_token.len();
        let previous = line[..start].chars().next_back();
        let next = line[end..].chars().next();
        if !is_word_or_dash(previous) && !is_word_or_dash(next) {
            if matches!(token, "M0" | "M1" | "M2" | "M3")
                && (previous == Some('.') || next == Some('.'))
            {
                search_start = end;
                continue;
            }
            return true;
        }
        search_start = end;
    }
    false
}

fn casing_warning_variants(prose: &str) -> Vec<String> {
    // Preserve hyphens so brand-qualified compounds (e.g., "oyatie-owned") stay
    // as a single token and do not duplicate brand-residue gate warnings.
    let words = prose
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    let mut variants = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if *word == "Oya" && words.get(index + 1) == Some(&"VCS") {
            continue;
        }
        if matches!(*word, "oyatie" | "OYA" | "Oya") {
            variants.push((*word).to_string());
        }
    }
    variants
}

fn acronym_candidates(prose: &str) -> BTreeSet<String> {
    prose
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '-'))
        .filter_map(|word| {
            let trimmed = word.trim_matches('-');
            if is_acronym_candidate(trimmed) {
                Some(trimmed.to_ascii_uppercase())
            } else {
                None
            }
        })
        .collect()
}

fn is_acronym_candidate(value: &str) -> bool {
    let len = value.len();
    // ADR-0018's warning lane is for prose acronyms missing from the glossary,
    // not generated identifiers, ADR IDs, version strings, memory sizes, or
    // event-code atoms. Numeric or hyphenated tokens remain enforceable by the
    // explicit retired-vocabulary hard-fail rules and by glossary term coverage.
    if value == "OYA"
        || value.chars().any(|character| character.is_ascii_digit())
        || value.contains('-')
        || RFC2119_NORMATIVE_KEYWORDS.contains(&value)
        // TODO/TBD are not glossary acronyms. They are open placeholder debt
        // markers delegated to oya-governance-placeholder-debt-kernel, whose
        // fail-closed registry gate blocks new, stale, and count-drifted
        // occurrences. Keeping them out of the acronym baseline prevents the
        // glossary lane from hiding placeholder cleanup as terminology work.
        || PLACEHOLDER_DEBT_TOKENS.contains(&value)
        || value.strip_prefix('M').is_some_and(|rest| {
            !rest.is_empty() && rest.chars().all(|character| character.is_ascii_digit())
        })
    {
        return false;
    }
    (2..=16).contains(&len)
        && value
            .chars()
            .any(|character| character.is_ascii_alphabetic())
        && value.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
        })
}

fn markdown_prose(contents: &str) -> String {
    let mut prose = String::new();
    let mut in_fence = false;
    let mut in_frontmatter = false;
    let mut at_document_start = true;
    let mut in_html_comment = false;
    let mut in_marker_block = false;
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if at_document_start {
            if in_frontmatter {
                if trimmed == "---" {
                    in_frontmatter = false;
                }
                continue;
            }
            if trimmed.is_empty() {
                continue;
            }
            if trimmed == "---" {
                in_frontmatter = true;
                continue;
            }
            at_document_start = false;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if in_marker_block {
            if trimmed.starts_with("<!--") && trimmed.contains(":end -->") {
                in_marker_block = false;
            }
            continue;
        }
        if trimmed.starts_with("<!--") && trimmed.contains(":start -->") {
            in_marker_block = true;
            continue;
        }
        let line = strip_html_comments(line, &mut in_html_comment);
        let line = strip_inline_code(&line);
        prose.push_str(&strip_markdown_link_destinations(&line));
        prose.push('\n');
    }
    prose
}

fn strip_html_comments(line: &str, in_html_comment: &mut bool) -> String {
    let mut output = String::new();
    let mut remaining = line;
    loop {
        if *in_html_comment {
            let Some(end) = remaining.find("-->") else {
                return output;
            };
            remaining = &remaining[end + "-->".len()..];
            *in_html_comment = false;
            continue;
        }
        let Some(start) = remaining.find("<!--") else {
            output.push_str(remaining);
            return output;
        };
        output.push_str(&remaining[..start]);
        remaining = &remaining[start + "<!--".len()..];
        let Some(end) = remaining.find("-->") else {
            *in_html_comment = true;
            return output;
        };
        remaining = &remaining[end + "-->".len()..];
    }
}

fn strip_inline_code(line: &str) -> String {
    let mut output = String::new();
    let mut in_code = false;
    for character in line.chars() {
        if character == '`' {
            in_code = !in_code;
            output.push(' ');
        } else if in_code {
            output.push(' ');
        } else {
            output.push(character);
        }
    }
    output
}

fn strip_markdown_link_destinations(line: &str) -> String {
    let mut output = String::new();
    let mut remaining = line;
    while let Some(label_start) = remaining.find('[') {
        output.push_str(&remaining[..label_start]);
        let label_and_after = &remaining[label_start + 1..];
        let Some(label_end) = label_and_after.find(']') else {
            output.push_str(&remaining[label_start..]);
            return output;
        };
        let label = &label_and_after[..label_end];
        let after_label = &label_and_after[label_end + 1..];
        if let Some(destination) = after_label.strip_prefix('(')
            && let Some(destination_end) = destination.find(')')
        {
            output.push_str(label);
            remaining = &destination[destination_end + 1..];
            continue;
        }
        output.push('[');
        output.push_str(label);
        output.push(']');
        remaining = after_label;
    }
    output.push_str(remaining);
    output
}

fn is_word_or_dash(character: Option<char>) -> bool {
    character.is_some_and(|character| character.is_ascii_alphanumeric() || character == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_forbidden_tokens_in_active_prose() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc("docs/ROADMAP.md", "Trust portal MVP launch.", false)],
                ["ADR"],
            ),
            Err(GlossaryVocabularyError::ForbiddenToken {
                path: "docs/ROADMAP.md".into(),
                token: "MVP".into(),
            })
        );
    }

    #[test]
    fn ignores_forbidden_tokens_in_code_spans() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/ADR.md",
                    "Legacy path `apps/oyatie-admin` only.",
                    false
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn ignores_forbidden_prefixes_in_markdown_link_destinations() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/AGENTS.md",
                    "Canonical doctrine: [doctrine](../specs/oyatie-doctrine.json).",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn allows_brand_qualified_prose_without_duplicating_brand_residue_gate() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/decisions/ADR.md",
                    "The ADR discusses an oyatie-owned deployment pattern.",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn allows_bare_m_series_tokens_that_are_not_milestone_terms() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/user-stories/persona.md",
                    "The persona uses a 14-inch M3 MacBook Pro.",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn ignores_forbidden_prefixes_in_leading_frontmatter() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/standards/doc-style.md",
                    "---\nrelated_specs:\n  - /specs/oyatie-doctrine.json\n---\n\nVisible prose.",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn ignores_forbidden_tokens_in_html_comment_blocks() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/PRD.md",
                    "<!-- marker:start -->\n\
                     target_path: bominal/agents/ultragoal/oyatie-product-delivery.md\n\
                     <!-- marker:end -->\n\
                     Active prose stays clean.",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn allows_forensic_documents_to_quote_retired_vocabulary() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/GLOSSARY.md",
                    "M0 / M1 / M2 / M3 / MVP and CUG are retired.",
                    true,
                )],
                ["ADR", "MVP", "CUG"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn warns_on_brand_casing_and_uncited_acronyms_without_failing() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/README.md",
                    "oyatie supports ABC and OYA variants.",
                    false
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 2,
                uncited_acronym_warnings: 1,
                warnings: vec![
                    warning(GlossaryVocabularyWarningKind::CasingVariant, "OYA"),
                    warning(GlossaryVocabularyWarningKind::CasingVariant, "oyatie"),
                    warning(GlossaryVocabularyWarningKind::UncitedAcronym, "ABC"),
                ],
                warning_sources: vec![
                    warning_source(
                        GlossaryVocabularyWarningKind::CasingVariant,
                        "OYA",
                        "docs/README.md"
                    ),
                    warning_source(
                        GlossaryVocabularyWarningKind::CasingVariant,
                        "oyatie",
                        "docs/README.md",
                    ),
                    warning_source(
                        GlossaryVocabularyWarningKind::UncitedAcronym,
                        "ABC",
                        "docs/README.md",
                    ),
                ],
            })
        );
    }

    #[test]
    fn accepts_oya_vcs_as_subsystem_name_without_brand_casing_warning() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/AGENTS.md",
                    "Oya VCS admission owns claim closeout.",
                    false,
                )],
                ["ADR", "VCS"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn ignores_numeric_and_hyphenated_artifact_tokens_as_acronym_noise() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/DESIGN.md",
                    "ADR-0015 references 100K rows, 1MB pages, 3-AZ cells, and W-Foundation.",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn ignores_rfc2119_normative_keywords_as_acronym_noise() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/AGENTS.md",
                    "MUST NOT regress; SHOULD cite evidence; MAY warn; OPTIONAL follow-up.",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn delegates_placeholder_debt_tokens_out_of_acronym_warnings() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc(
                    "docs/ROADMAP.md",
                    "TODO and TBD stay visible through the placeholder-debt gate.",
                    false,
                )],
                ["ADR"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    #[test]
    fn ignores_rationalized_uppercase_prose_words_without_suppressing_acronyms() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_ignored_words(
                [doc(
                    "docs/DOC-CATALOG.md",
                    "YES for ALL docs; ABC remains.",
                    false
                )],
                ["ADR"],
                [
                    ignored_word("YES", "table boolean value"),
                    ignored_word("ALL", "emphatic ordinary prose word"),
                ],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 1,
                warnings: vec![warning(
                    GlossaryVocabularyWarningKind::UncitedAcronym,
                    "ABC"
                )],
                warning_sources: vec![warning_source(
                    GlossaryVocabularyWarningKind::UncitedAcronym,
                    "ABC",
                    "docs/DOC-CATALOG.md",
                )],
            })
        );
    }

    #[test]
    fn rejects_invalid_or_duplicate_ignored_uppercase_words() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_ignored_words(
                [doc("docs/README.md", "ALL docs.", false)],
                ["ADR"],
                [ignored_word("ALL", "")],
            ),
            Err(GlossaryVocabularyError::InvalidIgnoredUppercaseWord {
                token: "ALL".into(),
            })
        );
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_ignored_words(
                [doc("docs/README.md", "ALL docs.", false)],
                ["ADR"],
                [ignored_word("MVP", "retired product vocabulary")],
            ),
            Err(GlossaryVocabularyError::InvalidIgnoredUppercaseWord {
                token: "MVP".into(),
            })
        );
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_ignored_words(
                [doc("docs/README.md", "ALL docs.", false)],
                ["ADR"],
                [
                    ignored_word("ALL", "emphatic ordinary prose word"),
                    ignored_word("ALL", "duplicate"),
                ],
            ),
            Err(GlossaryVocabularyError::DuplicateIgnoredUppercaseWord {
                token: "ALL".into(),
            })
        );
    }

    #[test]
    fn rejects_stale_ignored_uppercase_words() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_ignored_words(
                [doc("docs/README.md", "ABC remains.", false)],
                ["ADR"],
                [ignored_word("YES", "doc-catalog boolean value")],
            ),
            Err(GlossaryVocabularyError::StaleIgnoredUppercaseWord {
                token: "YES".into(),
            })
        );
    }

    #[test]
    fn baseline_rejects_new_or_stale_warnings() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_baseline(
                [doc("docs/README.md", "ABC appears.", false)],
                ["ADR"],
                [],
            ),
            Err(GlossaryVocabularyError::NewWarningOutsideBaseline {
                warning_id: "uncited-acronym\tABC".into(),
            })
        );
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_baseline(
                [doc("docs/README.md", "No acronym.", false)],
                ["ADR"],
                [warning(
                    GlossaryVocabularyWarningKind::UncitedAcronym,
                    "ABC"
                )],
            ),
            Err(GlossaryVocabularyError::StaleBaselineWarning {
                warning_id: "uncited-acronym\tABC".into(),
            })
        );
    }

    #[test]
    fn baseline_accepts_matching_warnings() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene_with_baseline(
                [doc("docs/README.md", "ABC appears.", false)],
                ["ADR"],
                [warning(
                    GlossaryVocabularyWarningKind::UncitedAcronym,
                    "ABC"
                )],
            )
            .map(|report| report.warnings),
            Ok(vec![warning(
                GlossaryVocabularyWarningKind::UncitedAcronym,
                "ABC"
            )])
        );
    }

    #[test]
    fn ignores_m_number_ranges_that_are_not_wave_tokens() {
        assert_eq!(
            validate_glossary_vocabulary_hygiene(
                [doc("docs/LEDGER.md", "MEDIUM tier M1..M33 entries.", false)],
                ["MEDIUM"],
            ),
            Ok(GlossaryVocabularyReport {
                documents_checked: 1,
                casing_warnings: 0,
                uncited_acronym_warnings: 0,
                warnings: vec![],
                warning_sources: vec![],
            })
        );
    }

    fn doc(path: &str, contents: &str, forensic_allowed: bool) -> VocabularyDocument {
        VocabularyDocument {
            path: path.into(),
            contents: contents.into(),
            forensic_allowed,
        }
    }

    fn warning(kind: GlossaryVocabularyWarningKind, token: &str) -> GlossaryVocabularyWarning {
        GlossaryVocabularyWarning {
            kind,
            token: token.into(),
        }
    }

    fn warning_source(
        kind: GlossaryVocabularyWarningKind,
        token: &str,
        path: &str,
    ) -> GlossaryVocabularyWarningSource {
        GlossaryVocabularyWarningSource {
            warning: warning(kind, token),
            path: path.into(),
        }
    }

    fn ignored_word(token: &str, rationale: &str) -> IgnoredUppercaseWord {
        IgnoredUppercaseWord {
            token: token.into(),
            rationale: rationale.into(),
        }
    }
}
