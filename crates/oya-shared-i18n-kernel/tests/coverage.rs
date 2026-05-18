#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_shared_i18n_kernel::{FluentI18nCatalog, I18nCatalog, LocaleTag, Message, MessageId};

fn id(s: &str) -> MessageId {
    MessageId::new(s).unwrap()
}

fn msg(p: &str) -> Message {
    Message::new(p.to_owned(), Vec::new()).unwrap()
}

#[test]
fn catalog_lists_inserted_locales_in_sorted_order() {
    let en = LocaleTag::new("en-US").unwrap();
    let ar = LocaleTag::new("ar-SA").unwrap();
    let ko = LocaleTag::new("ko-KR").unwrap();
    let mut catalog = FluentI18nCatalog::new(en.clone());
    catalog.insert(ar.clone(), id("greeting"), msg("مرحبا"));
    catalog.insert(ko.clone(), id("greeting"), msg("안녕"));
    catalog.insert(en, id("greeting"), msg("hello"));
    let locales = catalog.locales();
    let labels: Vec<&str> = locales.iter().map(LocaleTag::as_str).collect();
    assert!(labels.contains(&"ar-SA"));
    assert!(labels.contains(&"en-US"));
    assert!(labels.contains(&"ko-KR"));
}

#[test]
fn rtl_bundles_track_with_locale_tag_flag() {
    let ar = LocaleTag::new("ar-SA").unwrap();
    let he = LocaleTag::new("he-IL").unwrap();
    let en = LocaleTag::new("en-US").unwrap();
    assert!(ar.is_rtl());
    assert!(he.is_rtl());
    assert!(!en.is_rtl());
}

#[test]
fn unknown_message_id_yields_none_when_source_lacks_it_too() {
    let en = LocaleTag::new("en-US").unwrap();
    let catalog = FluentI18nCatalog::new(en.clone());
    assert!(catalog.message(&en, &id("never-defined")).is_none());
}

#[test]
fn primary_language_extracts_first_subtag() {
    assert_eq!(
        LocaleTag::new("zh-Hans-CN").unwrap().primary_language(),
        "zh"
    );
    assert_eq!(LocaleTag::new("pt-BR").unwrap().primary_language(), "pt");
}

#[test]
fn source_locale_is_listed_by_locales() {
    let en = LocaleTag::new("en-US").unwrap();
    let catalog = FluentI18nCatalog::new(en.clone());
    assert_eq!(catalog.source_locale(), &en);
    assert_eq!(catalog.locales(), vec![en]);
}
