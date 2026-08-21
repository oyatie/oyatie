// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use check_portfolio_citation_kernel::{
    CitationBlock, CitationRole, FoundryCorpusCitationVerdict, PortfolioCitationVerdict, verify,
    verify_foundry_corpus,
};

const DEFAULT_OYATIE_PRD: &str = "docs/PRD.md";
const DEFAULT_BOMINAL_PRD: &str = "../bominal/docs/consolidated/PRD.md";
const DEFAULT_FOUNDRY_PRD: &str = "docs/products/foundry/PRD.md";

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(report) => {
            println!(
                "portfolio-citation ok: citations_checked={} oyatie_cites_bominal={} bominal_cites_oyatie={} foundry_sources_present={}/{}",
                report.portfolio.citations_checked,
                report.portfolio.oyatie_cites_bominal,
                report.portfolio.bominal_cites_oyatie,
                report.foundry_corpus.present_sources.len(),
                report.foundry_corpus.required_sources_total,
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("portfolio-citation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run<I>(args: I) -> Result<LaneReport, String>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args)?;
    let oyatie_contents = fs::read_to_string(&options.oyatie_prd)
        .map_err(|error| format!("could not read {}: {error}", options.oyatie_prd.display()))?;
    let bominal_contents = fs::read_to_string(&options.bominal_prd)
        .map_err(|error| format!("could not read {}: {error}", options.bominal_prd.display()))?;
    let foundry_contents = fs::read_to_string(&options.foundry_prd)
        .map_err(|error| format!("could not read {}: {error}", options.foundry_prd.display()))?;

    let oyatie_citations = extract_citation_blocks(&oyatie_contents)?;
    let bominal_citations = extract_citation_blocks(&bominal_contents)?;
    let foundry_citations = extract_citation_blocks(&foundry_contents)?;
    let portfolio = verify(&oyatie_citations, &bominal_citations);
    let foundry_corpus = verify_foundry_corpus(&foundry_citations);
    if portfolio.is_complete() && foundry_corpus.is_complete() {
        Ok(LaneReport {
            portfolio,
            foundry_corpus,
        })
    } else {
        Err(format!(
            "incomplete portfolio citations: oyatie->bominal={} bominal->oyatie={} foundry_missing={:?}",
            portfolio.oyatie_cites_bominal,
            portfolio.bominal_cites_oyatie,
            foundry_corpus.missing_sources,
        ))
    }
}

struct LaneReport {
    portfolio: PortfolioCitationVerdict,
    foundry_corpus: FoundryCorpusCitationVerdict,
}

struct Options {
    oyatie_prd: PathBuf,
    bominal_prd: PathBuf,
    foundry_prd: PathBuf,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut oyatie_prd = env::var("OYATIE_PRD_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_OYATIE_PRD));
        let mut bominal_prd = env::var("BOMINAL_PRD_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_BOMINAL_PRD));
        let mut foundry_prd = env::var("FOUNDRY_PRD_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_FOUNDRY_PRD));

        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--oyatie-prd" => {
                    index += 1;
                    oyatie_prd = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--oyatie-prd requires a path".to_string())?;
                }
                "--bominal-prd" => {
                    index += 1;
                    bominal_prd = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--bominal-prd requires a path".to_string())?;
                }
                "--foundry-prd" => {
                    index += 1;
                    foundry_prd = args
                        .get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--foundry-prd requires a path".to_string())?;
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
            index += 1;
        }

        Ok(Self {
            oyatie_prd,
            bominal_prd,
            foundry_prd,
        })
    }
}

fn usage() -> String {
    "usage: oya-governance-portfolio-citation-app [--oyatie-prd PATH] [--bominal-prd PATH] [--foundry-prd PATH]".into()
}

fn extract_citation_blocks(contents: &str) -> Result<Vec<CitationBlock>, String> {
    let mut citations = Vec::new();
    let mut in_block = false;
    let mut role = None;
    let mut target_path = None;
    let mut anchor = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed == "<!-- portfolio-citation:start -->"
            || trimmed == "<!-- foundry-corpus-citation:start -->"
        {
            if in_block {
                return Err("nested portfolio-citation block".into());
            }
            in_block = true;
            role = None;
            target_path = None;
            anchor = None;
            continue;
        }
        if trimmed == "<!-- portfolio-citation:end -->"
            || trimmed == "<!-- foundry-corpus-citation:end -->"
        {
            if !in_block {
                return Err("portfolio-citation end without start".into());
            }
            citations.push(citation_from_parts(
                role.take(),
                target_path.take(),
                anchor.take(),
            )?);
            in_block = false;
            continue;
        }
        if !in_block {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("- role:") {
            role = Some(parse_role(value)?);
        } else if let Some(value) = trimmed.strip_prefix("role:") {
            role = Some(parse_role(value)?);
        } else if let Some(value) = trimmed.strip_prefix("target_path:") {
            target_path = Some(parse_scalar(value));
        } else if let Some(value) = trimmed.strip_prefix("anchor:") {
            anchor = Some(parse_scalar(value));
        }
    }

    if in_block {
        return Err("unterminated portfolio-citation block".into());
    }
    Ok(citations)
}

fn citation_from_parts(
    role: Option<CitationRole>,
    target_path: Option<String>,
    anchor: Option<String>,
) -> Result<CitationBlock, String> {
    let role = role.ok_or_else(|| "portfolio-citation block missing role".to_string())?;
    let target_path = target_path
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "portfolio-citation block missing target_path".to_string())?;
    Ok(CitationBlock {
        target_path,
        role,
        anchor: anchor.filter(|value| !value.trim().is_empty()),
    })
}

fn parse_role(value: &str) -> Result<CitationRole, String> {
    let value = parse_scalar(value);
    CitationRole::parse(&value).ok_or_else(|| format!("unknown portfolio citation role '{value}'"))
}

fn parse_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use check_portfolio_citation_kernel::{
        BOMINAL_PRD_PATH, OYATIE_PRD_PATH, REQUIRED_FOUNDRY_CORPUS_SOURCES,
    };

    #[test]
    fn extracts_portfolio_citation_block() {
        let citations = extract_citation_blocks(
            r#"
<!-- portfolio-citation:start -->
- role: PortfolioParent
  target_path: bominal/docs/consolidated/PRD.md
  target_repo: bominal
  target_prd: docs/consolidated/PRD.md
  anchor: product-requirements-document
<!-- portfolio-citation:end -->
"#,
        )
        .expect("citation block parses");

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].role, CitationRole::PortfolioParent);
        assert_eq!(citations[0].target_path, BOMINAL_PRD_PATH);
        assert_eq!(
            citations[0].anchor.as_deref(),
            Some("product-requirements-document")
        );
    }

    #[test]
    fn rejects_unterminated_block() {
        assert_eq!(
            extract_citation_blocks("<!-- portfolio-citation:start -->\n- role: PortfolioParent"),
            Err("unterminated portfolio-citation block".into())
        );
    }

    #[test]
    fn default_target_constants_match_kernel_contract() {
        assert_eq!(OYATIE_PRD_PATH, "oyatie/docs/PRD.md");
        assert_eq!(BOMINAL_PRD_PATH, "bominal/docs/consolidated/PRD.md");
    }

    #[test]
    fn extracts_foundry_corpus_citation_block() {
        let source = REQUIRED_FOUNDRY_CORPUS_SOURCES[0];
        let citations = extract_citation_blocks(&format!(
            r#"
<!-- foundry-corpus-citation:start -->
- role: FoundryCorpusSource
  target_path: {source}
  target_repo: bominal
  target_prd: agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md
<!-- foundry-corpus-citation:end -->
"#
        ))
        .expect("citation block parses");

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].role, CitationRole::FoundryCorpusSource);
        assert_eq!(citations[0].target_path, source);
    }
}
