#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use check_i18n_coverage::{I18nCoverageInput, check};
use shared_i18n_kernel::{FluentI18nCatalog, LocaleTag, Message, MessageId};

fn build() -> FluentI18nCatalog {
    let en = LocaleTag::new("en-US").unwrap();
    let mut catalog = FluentI18nCatalog::new(en.clone());
    for (id, p) in [
        ("welcome", "Welcome"),
        ("save", "Save"),
        ("cancel", "Cancel"),
        ("delete", "Delete"),
    ] {
        catalog.insert(
            en.clone(),
            MessageId::new(id).unwrap(),
            Message::new(p.into(), Vec::new()).unwrap(),
        );
    }
    let ko = LocaleTag::new("ko-KR").unwrap();
    for (id, p) in [("welcome", "환영"), ("save", "저장")] {
        catalog.insert(
            ko.clone(),
            MessageId::new(id).unwrap(),
            Message::new(p.into(), Vec::new()).unwrap(),
        );
    }
    catalog
}

#[test]
fn fifty_percent_coverage_below_default_threshold() {
    let catalog = build();
    let input = I18nCoverageInput {
        microservice: "ms".into(),
        catalog: &catalog,
        required_locales: vec![LocaleTag::new("ko-KR").unwrap()],
        min_coverage_bps: 8_000,
    };
    let r = check(&[input]);
    assert_eq!(r.gaps.len(), 1);
    assert_eq!(r.gaps[0].coverage_bps, 5_000);
}

#[test]
fn missing_locale_yields_zero_coverage_gap() {
    let catalog = build();
    let input = I18nCoverageInput {
        microservice: "ms".into(),
        catalog: &catalog,
        required_locales: vec![LocaleTag::new("ar-SA").unwrap()],
        min_coverage_bps: 100,
    };
    let r = check(&[input]);
    assert_eq!(r.gaps.len(), 1);
    assert_eq!(r.gaps[0].coverage_bps, 0);
}

#[test]
fn source_locale_passes_default_threshold() {
    let catalog = build();
    let input = I18nCoverageInput {
        microservice: "ms".into(),
        catalog: &catalog,
        required_locales: vec![LocaleTag::new("en-US").unwrap()],
        min_coverage_bps: 10_000,
    };
    let r = check(&[input]);
    assert!(r.gaps.is_empty());
}

#[test]
fn missing_ids_reported_sorted_for_gap() {
    let catalog = build();
    let input = I18nCoverageInput {
        microservice: "ms".into(),
        catalog: &catalog,
        required_locales: vec![LocaleTag::new("ko-KR").unwrap()],
        min_coverage_bps: 10_000,
    };
    let r = check(&[input]);
    assert!(
        r.gaps[0]
            .missing_message_ids
            .contains(&"cancel".to_string())
    );
    assert!(
        r.gaps[0]
            .missing_message_ids
            .contains(&"delete".to_string())
    );
}

#[test]
fn coverage_counted_per_microservice_input() {
    let catalog = build();
    let input_a = I18nCoverageInput {
        microservice: "ms-a".into(),
        catalog: &catalog,
        required_locales: vec![LocaleTag::new("ko-KR").unwrap()],
        min_coverage_bps: 10_000,
    };
    let input_b = I18nCoverageInput {
        microservice: "ms-b".into(),
        catalog: &catalog,
        required_locales: vec![LocaleTag::new("ko-KR").unwrap()],
        min_coverage_bps: 10_000,
    };
    let r = check(&[input_a, input_b]);
    assert_eq!(r.gaps.len(), 2);
}
