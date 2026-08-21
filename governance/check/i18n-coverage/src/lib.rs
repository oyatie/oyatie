//! ADR-0206 i18n coverage gate.
//!
//! Advisory lane that checks per-µservice `clients/i18n/source.ftl`
//! plus per-locale overlays for coverage. Computes per-locale coverage
//! in basis-points (10000 = 100%) and emits a gap report listing the
//! message ids missing from each target locale.
//!
//! Pure model; no I/O. Adapters do the FTL parse → `FluentI18nCatalog`
//! population.
//!
//! ADR-0083 Tier 3 test exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_i18n_kernel::{FluentI18nCatalog, LocaleTag, MessageId};

/// Per-µservice manifest entry — links a microservice to its catalog +
/// the locale tags it MUST cover. The gate fails advisory if any
/// declared locale has coverage below `min_coverage_bps`.
pub struct I18nCoverageInput<'a> {
    pub microservice: String,
    pub catalog: &'a FluentI18nCatalog,
    pub required_locales: Vec<LocaleTag>,
    pub min_coverage_bps: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct I18nGap {
    pub microservice: String,
    pub locale: String,
    pub coverage_bps: u32,
    pub min_coverage_bps: u32,
    pub missing_message_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct I18nCoverageReport {
    pub microservices_checked: usize,
    pub locales_checked: usize,
    pub gaps: Vec<I18nGap>,
}

pub fn check(inputs: &[I18nCoverageInput<'_>]) -> I18nCoverageReport {
    let mut gaps = Vec::new();
    let mut locales_checked = 0usize;
    for input in inputs {
        for locale in &input.required_locales {
            locales_checked += 1;
            let coverage = input.catalog.coverage_bps(locale);
            if coverage < input.min_coverage_bps {
                let missing: Vec<String> = input
                    .catalog
                    .missing_messages(locale)
                    .into_iter()
                    .map(|m: MessageId| m.as_str().to_owned())
                    .collect();
                gaps.push(I18nGap {
                    microservice: input.microservice.clone(),
                    locale: locale.as_str().to_owned(),
                    coverage_bps: coverage,
                    min_coverage_bps: input.min_coverage_bps,
                    missing_message_ids: missing,
                });
            }
        }
    }
    I18nCoverageReport {
        microservices_checked: inputs.len(),
        locales_checked,
        gaps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_i18n_kernel::{LocaleTag, Message, MessageId};

    fn cat(locales: &[(&str, &[(&str, &str)])]) -> (FluentI18nCatalog, LocaleTag) {
        let en = LocaleTag::new("en-US").unwrap();
        let mut catalog = FluentI18nCatalog::new(en.clone());
        for (loc, msgs) in locales {
            let l = LocaleTag::new(loc).unwrap();
            for (id, pattern) in *msgs {
                catalog.insert(
                    l.clone(),
                    MessageId::new(id).unwrap(),
                    Message::new((*pattern).to_owned(), Vec::new()).unwrap(),
                );
            }
        }
        (catalog, en)
    }

    #[test]
    fn fully_covered_locale_has_no_gap() {
        let (catalog, _) = cat(&[
            ("en-US", &[("hello", "Hello"), ("bye", "Bye")]),
            ("ar-SA", &[("hello", "مرحبا"), ("bye", "وداعا")]),
        ]);
        let input = I18nCoverageInput {
            microservice: "workflow-studio".into(),
            catalog: &catalog,
            required_locales: vec![LocaleTag::new("ar-SA").unwrap()],
            min_coverage_bps: 10_000,
        };
        let r = check(&[input]);
        assert!(r.gaps.is_empty());
        assert_eq!(r.locales_checked, 1);
    }

    #[test]
    fn partially_covered_locale_is_flagged() {
        let (catalog, _) = cat(&[
            ("en-US", &[("hello", "Hello"), ("bye", "Bye")]),
            ("ko-KR", &[("hello", "안녕")]),
        ]);
        let input = I18nCoverageInput {
            microservice: "workflow-studio".into(),
            catalog: &catalog,
            required_locales: vec![LocaleTag::new("ko-KR").unwrap()],
            min_coverage_bps: 10_000,
        };
        let r = check(&[input]);
        assert_eq!(r.gaps.len(), 1);
        assert_eq!(r.gaps[0].coverage_bps, 5_000);
        assert_eq!(r.gaps[0].missing_message_ids, vec!["bye".to_string()]);
    }

    #[test]
    fn lower_threshold_passes_partial_coverage() {
        let (catalog, _) = cat(&[
            ("en-US", &[("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
            ("ja-JP", &[("a", "1"), ("b", "2")]),
        ]);
        let input = I18nCoverageInput {
            microservice: "calendar".into(),
            catalog: &catalog,
            required_locales: vec![LocaleTag::new("ja-JP").unwrap()],
            min_coverage_bps: 4_000, // 40% threshold
        };
        let r = check(&[input]);
        assert!(r.gaps.is_empty());
    }

    #[test]
    fn multiple_locales_flagged_independently() {
        let (catalog, _) = cat(&[
            ("en-US", &[("a", "1"), ("b", "2")]),
            ("ko-KR", &[("a", "1")]),
            ("ar-SA", &[]),
        ]);
        let input = I18nCoverageInput {
            microservice: "ops-portal".into(),
            catalog: &catalog,
            required_locales: vec![
                LocaleTag::new("ko-KR").unwrap(),
                LocaleTag::new("ar-SA").unwrap(),
            ],
            min_coverage_bps: 10_000,
        };
        let r = check(&[input]);
        assert_eq!(r.gaps.len(), 2);
        assert_eq!(r.locales_checked, 2);
    }

    #[test]
    fn empty_inputs_yields_empty_report() {
        let r = check(&[]);
        assert_eq!(r.microservices_checked, 0);
        assert_eq!(r.locales_checked, 0);
        assert!(r.gaps.is_empty());
    }
}
