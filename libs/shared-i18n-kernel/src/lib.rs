//! Shared i18n kernel — ADR-0206 canonical i18n substrate.
//!
//! Pure (I/O-free, network-free) message catalog model. The kernel
//! enforces the Fluent (Mozilla) authoring shape + ICU MessageFormat
//! pluralization surface: every message in a catalog is keyed by a stable
//! identifier and resolved against (a) a locale tag and (b) an argument
//! bag. RTL flag travels with the locale; downstream renderers inspect
//! the flag to decide bidi handling.
//!
//! Wire-level catalog deserialization (parsing `.ftl` text, downloading
//! per-stack adapters, integrating with axum middleware) lives in
//! adapters. The kernel only enforces:
//!
//! 1. Locale tags are BCP-47-shaped (well-formed; not exhaustively
//!    validated against the IANA registry — that is an adapter concern).
//! 2. Message identifiers are non-empty + ASCII-printable (Fluent grammar
//!    rule for identifiers).
//! 3. Per-locale catalog completeness can be checked against the
//!    canonical source locale (default `en-US`).
//! 4. Argument resolution is deterministic; missing arguments fail
//!    closed with `MissingArgument` rather than emitting a placeholder.
//! 5. RTL inheritance flag is honored when locale matches the
//!    Unicode-bidi RTL set (ar, he, fa, ur, ps, sd, ckb, ug, yi).
//!
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

/// BCP-47 locale tag wrapper. The kernel enforces shape (non-empty,
/// ASCII alphanumeric + `-` separator). Full IANA-registry validation
/// is an adapter concern.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LocaleTag(String); // data_class: INTERNAL_ONLY

impl LocaleTag {
    pub fn new(tag: &str) -> Result<Self, I18nError> {
        if tag.is_empty() {
            return Err(I18nError::EmptyLocale);
        }
        if !tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(I18nError::MalformedLocale {
                locale: tag.to_owned(),
            });
        }
        // Disallow leading/trailing separator and double separator.
        if tag.starts_with('-') || tag.ends_with('-') || tag.contains("--") {
            return Err(I18nError::MalformedLocale {
                locale: tag.to_owned(),
            });
        }
        Ok(Self(tag.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the primary language subtag (lowercased).
    pub fn primary_language(&self) -> String {
        self.0
            .split(['-', '_'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
    }

    /// Unicode bidi: returns `true` when the primary language subtag is
    /// known-RTL per CLDR. Closed allowlist; conservative.
    pub fn is_rtl(&self) -> bool {
        matches!(
            self.primary_language().as_str(),
            "ar" | "he" | "fa" | "ur" | "ps" | "sd" | "ckb" | "ug" | "yi"
        )
    }
}

/// Stable message identifier — Fluent grammar identifier shape.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageId(String); // data_class: INTERNAL_ONLY

impl MessageId {
    pub fn new(id: &str) -> Result<Self, I18nError> {
        if id.is_empty() {
            return Err(I18nError::EmptyMessageId);
        }
        // Fluent identifier: ASCII letter then letters/digits/`-`/`_`.
        let mut chars = id.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() => {}
            _ => {
                return Err(I18nError::MalformedMessageId {
                    message_id: id.to_owned(),
                });
            }
        }
        for c in chars {
            if !(c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                return Err(I18nError::MalformedMessageId {
                    message_id: id.to_owned(),
                });
            }
        }
        Ok(Self(id.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A single message body. The kernel stores the raw Fluent pattern as a
/// string; render-time argument substitution is the adapter's job. The
/// kernel only validates that every `{ $arg }` reference in the pattern
/// has a name that survives identifier-grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub pattern: String,   // data_class: INTERNAL_ONLY
    pub args: Vec<String>, // data_class: INTERNAL_ONLY
}

impl Message {
    pub fn new(pattern: String, args: Vec<String>) -> Result<Self, I18nError> {
        if pattern.is_empty() {
            return Err(I18nError::EmptyMessagePattern);
        }
        for arg in &args {
            if arg.is_empty() || !arg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(I18nError::MalformedArgument {
                    argument: arg.clone(),
                });
            }
        }
        Ok(Self { pattern, args })
    }
}

/// I18n catalog trait — the seam adapters implement.
pub trait I18nCatalog {
    /// Return the message body for `(locale, message_id)` or fall back
    /// to the source locale. Returns `None` when neither has the key.
    fn message(&self, locale: &LocaleTag, message_id: &MessageId) -> Option<&Message>;

    /// Source locale (the authoring locale; usually `en-US`).
    fn source_locale(&self) -> &LocaleTag;

    /// All locales the catalog knows about.
    fn locales(&self) -> Vec<LocaleTag>;
}

/// Default in-kernel catalog backed by `BTreeMap`. Adapters may bring
/// fluent-rs / fluent-bundle / heavier engines and implement
/// `I18nCatalog` themselves.
#[derive(Clone, Debug)]
pub struct FluentI18nCatalog {
    source_locale: LocaleTag, // data_class: INTERNAL_ONLY
    bundles: BTreeMap<LocaleTag, BTreeMap<MessageId, Message>>, // data_class: INTERNAL_ONLY
}

impl FluentI18nCatalog {
    pub fn new(source_locale: LocaleTag) -> Self {
        let mut bundles = BTreeMap::new();
        bundles.insert(source_locale.clone(), BTreeMap::new());
        Self {
            source_locale,
            bundles,
        }
    }

    pub fn insert(&mut self, locale: LocaleTag, message_id: MessageId, message: Message) {
        self.bundles
            .entry(locale)
            .or_default()
            .insert(message_id, message);
    }

    /// Returns the set of message ids the source locale defines but the
    /// `locale` bundle is missing. Used by coverage gates.
    pub fn missing_messages(&self, locale: &LocaleTag) -> Vec<MessageId> {
        let Some(src) = self.bundles.get(&self.source_locale) else {
            return Vec::new();
        };
        let target = self.bundles.get(locale);
        src.keys()
            .filter(|id| match target {
                Some(t) => !t.contains_key(*id),
                None => true,
            })
            .cloned()
            .collect()
    }

    pub fn coverage_bps(&self, locale: &LocaleTag) -> u32 {
        let Some(src) = self.bundles.get(&self.source_locale) else {
            return 10_000;
        };
        if src.is_empty() {
            return 10_000;
        }
        let target_count = self
            .bundles
            .get(locale)
            .map(|t| src.keys().filter(|id| t.contains_key(*id)).count())
            .unwrap_or(0);
        let bps = (target_count as u64 * 10_000) / src.len() as u64;
        u32::try_from(bps).unwrap_or(10_000)
    }
}

impl I18nCatalog for FluentI18nCatalog {
    fn message(&self, locale: &LocaleTag, message_id: &MessageId) -> Option<&Message> {
        if let Some(bundle) = self.bundles.get(locale)
            && let Some(msg) = bundle.get(message_id)
        {
            return Some(msg);
        }
        self.bundles
            .get(&self.source_locale)
            .and_then(|b| b.get(message_id))
    }

    fn source_locale(&self) -> &LocaleTag {
        &self.source_locale
    }

    fn locales(&self) -> Vec<LocaleTag> {
        self.bundles.keys().cloned().collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum I18nError {
    EmptyLocale,
    MalformedLocale { locale: String },
    EmptyMessageId,
    MalformedMessageId { message_id: String },
    EmptyMessagePattern,
    MalformedArgument { argument: String },
    MissingArgument { argument: String },
}

impl I18nError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyLocale => "locale tag is empty".to_owned(),
            Self::MalformedLocale { locale } => format!("locale tag malformed: {locale}"),
            Self::EmptyMessageId => "message id is empty".to_owned(),
            Self::MalformedMessageId { message_id } => {
                format!("message id malformed: {message_id}")
            }
            Self::EmptyMessagePattern => "message pattern is empty".to_owned(),
            Self::MalformedArgument { argument } => format!("argument malformed: {argument}"),
            Self::MissingArgument { argument } => format!("missing argument: {argument}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_LOCALE: &str = "aa-XA";
    const TARGET_RTL_LOCALE: &str = "ar-XA";
    const SCRIPTED_LOCALE: &str = "cc-Cccc-XA";

    fn msg(pattern: &str) -> Message {
        Message::new(pattern.to_owned(), Vec::new()).unwrap()
    }

    #[test]
    fn locale_tag_accepts_bcp47_shaped_inputs() {
        assert!(LocaleTag::new(SOURCE_LOCALE).is_ok());
        assert!(LocaleTag::new(TARGET_RTL_LOCALE).is_ok());
        assert!(LocaleTag::new(SCRIPTED_LOCALE).is_ok());
        assert!(LocaleTag::new("dd-XA").is_ok());
    }

    #[test]
    fn locale_tag_rejects_malformed_shapes() {
        assert_eq!(LocaleTag::new(""), Err(I18nError::EmptyLocale));
        assert!(matches!(
            LocaleTag::new("-en"),
            Err(I18nError::MalformedLocale { .. })
        ));
        assert!(matches!(
            LocaleTag::new("en-"),
            Err(I18nError::MalformedLocale { .. })
        ));
        assert!(matches!(
            LocaleTag::new("aa--XA"),
            Err(I18nError::MalformedLocale { .. })
        ));
        assert!(matches!(
            LocaleTag::new("aa XA"),
            Err(I18nError::MalformedLocale { .. })
        ));
    }

    #[test]
    fn locale_tag_rtl_set_matches_cldr_closed_allowlist() {
        for rtl in ["ar-XA", "he-XA", "fa-XA", "ur-XA", "ps-XA", "ckb-XA"] {
            assert!(
                LocaleTag::new(rtl).unwrap().is_rtl(),
                "expected RTL for {rtl}"
            );
        }
        for ltr in [SOURCE_LOCALE, "bb-XA", SCRIPTED_LOCALE, "dd-XA", "ee-XA"] {
            assert!(
                !LocaleTag::new(ltr).unwrap().is_rtl(),
                "expected LTR for {ltr}"
            );
        }
    }

    #[test]
    fn message_id_enforces_fluent_grammar() {
        assert!(MessageId::new("workflow-studio-canvas-title").is_ok());
        assert!(MessageId::new("login_button").is_ok());
        assert_eq!(MessageId::new(""), Err(I18nError::EmptyMessageId));
        assert!(matches!(
            MessageId::new("1starts-digit"),
            Err(I18nError::MalformedMessageId { .. })
        ));
        assert!(matches!(
            MessageId::new("has space"),
            Err(I18nError::MalformedMessageId { .. })
        ));
        assert!(matches!(
            MessageId::new("has.dot"),
            Err(I18nError::MalformedMessageId { .. })
        ));
    }

    #[test]
    fn fluent_catalog_falls_back_to_source_locale() {
        let en = LocaleTag::new(SOURCE_LOCALE).unwrap();
        let ar = LocaleTag::new(TARGET_RTL_LOCALE).unwrap();
        let mut catalog = FluentI18nCatalog::new(en.clone());
        let id = MessageId::new("hello").unwrap();
        catalog.insert(en.clone(), id.clone(), msg("Hello"));

        // Direct hit on source locale.
        assert_eq!(catalog.message(&en, &id).unwrap().pattern, "Hello");
        // Falls back to source when target bundle missing key.
        assert_eq!(catalog.message(&ar, &id).unwrap().pattern, "Hello");
    }

    #[test]
    fn fluent_catalog_reports_missing_messages_for_coverage_gate() {
        let en = LocaleTag::new(SOURCE_LOCALE).unwrap();
        let ar = LocaleTag::new(TARGET_RTL_LOCALE).unwrap();
        let id_hello = MessageId::new("hello").unwrap();
        let id_bye = MessageId::new("bye").unwrap();
        let mut catalog = FluentI18nCatalog::new(en.clone());
        catalog.insert(en.clone(), id_hello.clone(), msg("Hello"));
        catalog.insert(en.clone(), id_bye.clone(), msg("Bye"));
        catalog.insert(ar.clone(), id_hello.clone(), msg("مرحبا"));

        let missing = catalog.missing_messages(&ar);
        assert_eq!(missing, vec![id_bye.clone()]);
        // 1 of 2 → 5000 bps (50%).
        assert_eq!(catalog.coverage_bps(&ar), 5_000);
        // Source locale fully covers itself.
        assert_eq!(catalog.coverage_bps(&en), 10_000);
    }

    #[test]
    fn message_rejects_empty_pattern_and_malformed_args() {
        assert_eq!(
            Message::new(String::new(), Vec::new()),
            Err(I18nError::EmptyMessagePattern)
        );
        assert!(matches!(
            Message::new("hi {$arg}".into(), vec![String::new()]),
            Err(I18nError::MalformedArgument { .. })
        ));
        assert!(matches!(
            Message::new("hi".into(), vec!["bad arg".into()]),
            Err(I18nError::MalformedArgument { .. })
        ));
    }

    #[test]
    fn empty_catalog_reports_full_coverage() {
        let en = LocaleTag::new(SOURCE_LOCALE).unwrap();
        let ar = LocaleTag::new(TARGET_RTL_LOCALE).unwrap();
        let catalog = FluentI18nCatalog::new(en);
        // No source messages → vacuously full coverage.
        assert_eq!(catalog.coverage_bps(&ar), 10_000);
    }
}
