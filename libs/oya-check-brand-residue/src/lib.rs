//! Foundry brand-residue fitness kernel.
//!
//! ADR-0017 makes `Oyatie` the canonical product brand. This kernel does not
//! ban canonical brand usage. It prevents the MFL-0011 failure class: semantic
//! residue from mechanical brand sweeps, especially tautological transitions
//! such as a rebrand statement whose old and new terms are identical.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrandResidueDocument {
    pub path: String,     // data_class: INTERNAL_ONLY
    pub contents: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrandResidueReport {
    pub documents_checked: usize, // data_class: INTERNAL_ONLY
    pub patterns_checked: usize,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrandResidueError {
    NoDocuments,
    TautologicalBrandTransition {
        path: String,
        line: usize,
        term: String,
        pattern: BrandResiduePattern,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrandResiduePattern {
    RebrandArrow,
    RetiredTermsTable,
    RenamePhrase,
}

pub fn validate_brand_residue<D>(documents: D) -> Result<BrandResidueReport, BrandResidueError>
where
    D: IntoIterator<Item = BrandResidueDocument>,
{
    let mut documents_checked = 0usize;
    let mut patterns_checked = 0usize;

    for document in documents {
        documents_checked += 1;
        for (line_index, line) in document.contents.lines().enumerate() {
            let line_number = line_index + 1;
            if let Some(term) = tautological_rebrand_arrow(line) {
                return Err(BrandResidueError::TautologicalBrandTransition {
                    path: document.path,
                    line: line_number,
                    term,
                    pattern: BrandResiduePattern::RebrandArrow,
                });
            }
            patterns_checked += rebrand_arrow_pattern_count(line);

            if let Some(term) = tautological_retired_terms_row(line) {
                return Err(BrandResidueError::TautologicalBrandTransition {
                    path: document.path,
                    line: line_number,
                    term,
                    pattern: BrandResiduePattern::RetiredTermsTable,
                });
            }
            patterns_checked += retired_terms_row_pattern_count(line);

            if let Some(term) = tautological_after_rename_phrase(line) {
                return Err(BrandResidueError::TautologicalBrandTransition {
                    path: document.path,
                    line: line_number,
                    term,
                    pattern: BrandResiduePattern::RenamePhrase,
                });
            }
            patterns_checked += after_rename_phrase_pattern_count(line);
        }
    }

    if documents_checked == 0 {
        Err(BrandResidueError::NoDocuments)
    } else {
        Ok(BrandResidueReport {
            documents_checked,
            patterns_checked,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BacktickedValue {
    value: String,
    start: usize,
    end: usize,
}

fn tautological_rebrand_arrow(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    if !lower.contains("rebrand") && !lower.contains("rename") {
        return None;
    }
    let spans = backticked_values(line);
    for pair in spans.windows(2) {
        let first = &pair[0];
        let second = &pair[1];
        let between = &line[first.end..second.start];
        if !contains_arrow(between) {
            continue;
        }
        if same_brand_term(&first.value, &second.value) {
            return Some(first.value.clone());
        }
    }
    None
}

fn rebrand_arrow_pattern_count(line: &str) -> usize {
    let lower = line.to_lowercase();
    if !lower.contains("rebrand") && !lower.contains("rename") {
        return 0;
    }
    backticked_values(line)
        .windows(2)
        .filter(|pair| contains_arrow(&line[pair[0].end..pair[1].start]))
        .count()
}

fn tautological_retired_terms_row(line: &str) -> Option<String> {
    let cells = markdown_cells(line)?;
    if cells.len() < 2 || !brand_like(cells[0]) && !brand_like(cells[1]) {
        return None;
    }
    if same_brand_term(cells[0], cells[1]) {
        return Some(clean_brand_term(cells[0]));
    }
    None
}

fn retired_terms_row_pattern_count(line: &str) -> usize {
    let Some(cells) = markdown_cells(line) else {
        return 0;
    };
    if cells.len() >= 2 && (brand_like(cells[0]) || brand_like(cells[1])) {
        1
    } else {
        0
    }
}

fn tautological_after_rename_phrase(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let phrase = "after rename to";
    let phrase_index = lower.find(phrase)?;
    let before = &line[..phrase_index];
    let first = backticked_values(before).first()?.value.clone();
    let after = &line[phrase_index + phrase.len()..];
    let target = leading_brand_word(after)?;
    if same_brand_term(&first, &target) {
        Some(first)
    } else {
        None
    }
}

fn after_rename_phrase_pattern_count(line: &str) -> usize {
    let lower = line.to_lowercase();
    if lower.contains("after rename to") && !backticked_values(line).is_empty() {
        1
    } else {
        0
    }
}

fn backticked_values(line: &str) -> Vec<BacktickedValue> {
    let bytes = line.as_bytes();
    let mut values = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        let value_start = index;
        while index < bytes.len() && bytes[index] != b'`' {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        let value_end = index;
        let end = index + 1;
        values.push(BacktickedValue {
            value: line[value_start..value_end].to_string(),
            start,
            end,
        });
        index = end;
    }
    values
}

fn contains_arrow(value: &str) -> bool {
    value.contains('→') || value.contains("->")
}

fn markdown_cells(line: &str) -> Option<Vec<&str>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return None;
    }
    if trimmed
        .chars()
        .all(|character| matches!(character, '|' | '-' | ':' | ' '))
    {
        return None;
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    if cells.iter().any(|cell| *cell == "Old" || *cell == "New") {
        None
    } else {
        Some(cells)
    }
}

fn leading_brand_word(value: &str) -> Option<String> {
    let trimmed = value.trim_start();
    let word = trimmed
        .chars()
        .take_while(|character| character.is_alphanumeric() || matches!(character, '-' | '*'))
        .collect::<String>();
    if word.is_empty() { None } else { Some(word) }
}

fn same_brand_term(left: &str, right: &str) -> bool {
    let left = normalize_brand_term(left);
    let right = normalize_brand_term(right);
    !left.is_empty() && left == right && brand_like(&left)
}

fn brand_like(value: &str) -> bool {
    let normalized = normalize_brand_term(value);
    normalized.contains("oyatie") || normalized.contains("oya-") || normalized == "oya"
}

fn normalize_brand_term(value: &str) -> String {
    clean_brand_term(value).to_ascii_lowercase()
}

fn clean_brand_term(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('*')
        .trim_matches('_')
        .trim_matches('"')
        .trim_matches('“')
        .trim_matches('”')
        .trim_matches('(')
        .trim_matches(')')
        .trim_matches('.')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_brand_usage_and_real_brand_transitions() {
        assert_eq!(
            validate_brand_residue([doc(
                "docs/brand.md",
                "Oyatie is the product brand.\n| `oyatie-*` Cargo prefix | `oya-*` | ADR-0017 |\nAll package strings rebrand from `oyatie-*` → `oya-*`."
            )]),
            Ok(BrandResidueReport {
                documents_checked: 1,
                patterns_checked: 2,
            })
        );
    }

    #[test]
    fn rejects_tautological_rebrand_arrows() {
        assert_eq!(
            validate_brand_residue([doc(
                "docs/PRD.md",
                "All strings rebrand from `Oyatie` → `Oyatie`."
            )]),
            Err(BrandResidueError::TautologicalBrandTransition {
                path: "docs/PRD.md".into(),
                line: 1,
                term: "Oyatie".into(),
                pattern: BrandResiduePattern::RebrandArrow,
            })
        );
    }

    #[test]
    fn rejects_tautological_retired_terms_rows() {
        assert_eq!(
            validate_brand_residue([doc(
                "docs/GLOSSARY.md",
                "| Oyatie | Oyatie | Brand rename per ADR-0017 |"
            )]),
            Err(BrandResidueError::TautologicalBrandTransition {
                path: "docs/GLOSSARY.md".into(),
                line: 1,
                term: "Oyatie".into(),
                pattern: BrandResiduePattern::RetiredTermsTable,
            })
        );
    }

    #[test]
    fn rejects_tautological_after_rename_phrases() {
        assert_eq!(
            validate_brand_residue([doc(
                "docs/MISTAKES-LEDGER.md",
                "Brand `Oyatie` strings in product code after rename to Oyatie"
            )]),
            Err(BrandResidueError::TautologicalBrandTransition {
                path: "docs/MISTAKES-LEDGER.md".into(),
                line: 1,
                term: "Oyatie".into(),
                pattern: BrandResiduePattern::RenamePhrase,
            })
        );
    }

    #[test]
    fn rejects_empty_inputs() {
        assert_eq!(
            validate_brand_residue([]),
            Err(BrandResidueError::NoDocuments)
        );
    }

    fn doc(path: &str, contents: &str) -> BrandResidueDocument {
        BrandResidueDocument {
            path: path.into(),
            contents: contents.into(),
        }
    }
}
