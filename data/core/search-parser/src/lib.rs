//! Search parser domain: morphology pipeline for generic and pack-provided tokens.
//!
//! Per M03-P05-IP-001. Pure domain — concrete tokenizer adapters live
//! downstream, with pack-specific adapter selection kept outside canonical base.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use data_search_crawler::CrawlTarget;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MorphologyLocale {
    Generic,
    PackPrimary,
    PackSecondary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Particle,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MorphologyToken {
    pub surface: String,              // data_class: INTERNAL_ONLY
    pub lemma: String,                // data_class: INTERNAL_ONLY
    pub locale: MorphologyLocale,     // data_class: INTERNAL_ONLY
    pub part_of_speech: PartOfSpeech, // data_class: INTERNAL_ONLY
    pub byte_offset: u32,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedDocument {
    pub document_id: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub source_target_id: String,     // data_class: INTERNAL_ONLY
    pub locale: MorphologyLocale,     // data_class: INTERNAL_ONLY
    pub tokens: Vec<MorphologyToken>, // data_class: INTERNAL_ONLY
    pub body_byte_length: u32,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    EmptyDocumentId,
    EmptyTenantMismatch,
    NoTokens,
    InvalidLocaleForLemma,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedDocumentCreate {
    pub document_id: String,
    pub source: CrawlTarget,
    pub locale: MorphologyLocale,
    pub tokens: Vec<MorphologyToken>,
    pub body_byte_length: u32,
}

impl ParsedDocument {
    pub fn new(input: ParsedDocumentCreate) -> Result<Self, ParseError> {
        if input.document_id.trim().is_empty() {
            return Err(ParseError::EmptyDocumentId);
        }
        if input.source.tenant_id.trim().is_empty() {
            return Err(ParseError::EmptyTenantMismatch);
        }
        if input.tokens.is_empty() {
            return Err(ParseError::NoTokens);
        }
        for token in &input.tokens {
            if token.locale != input.locale {
                return Err(ParseError::InvalidLocaleForLemma);
            }
        }
        Ok(Self {
            document_id: input.document_id,
            tenant_id: input.source.tenant_id.clone(),
            source_target_id: input.source.target_id.clone(),
            locale: input.locale,
            tokens: input.tokens,
            body_byte_length: input.body_byte_length,
        })
    }

    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_search_crawler::{CrawlPriority, CrawlScheme, CrawlTargetCreate};
    use std::collections::BTreeMap;

    fn target() -> CrawlTarget {
        CrawlTarget::new(CrawlTargetCreate {
            target_id: "tgt_001".to_string(),
            tenant_id: "ten_alpha".to_string(),
            scheme: CrawlScheme::Https,
            canonical_url: "https://alpha.example.test/p/1".to_string(),
            priority: CrawlPriority::Normal,
            depth_budget: 2,
            headers: BTreeMap::new(),
        })
        .expect("valid target")
    }

    fn token(surface: &str, locale: MorphologyLocale) -> MorphologyToken {
        MorphologyToken {
            surface: surface.to_string(),
            lemma: surface.to_string(),
            locale,
            part_of_speech: PartOfSpeech::Noun,
            byte_offset: 0,
        }
    }

    #[test]
    fn builds_pack_primary_document() {
        let doc = ParsedDocument::new(ParsedDocumentCreate {
            document_id: "doc_1".to_string(),
            source: target(),
            locale: MorphologyLocale::PackPrimary,
            tokens: vec![token("term-a", MorphologyLocale::PackPrimary)],
            body_byte_length: 6,
        })
        .expect("parsed");
        assert_eq!(doc.token_count(), 1);
        assert_eq!(doc.locale, MorphologyLocale::PackPrimary);
        assert_eq!(doc.tenant_id, "ten_alpha");
    }

    #[test]
    fn rejects_empty_token_list() {
        let result = ParsedDocument::new(ParsedDocumentCreate {
            document_id: "doc_x".to_string(),
            source: target(),
            locale: MorphologyLocale::Generic,
            tokens: vec![],
            body_byte_length: 0,
        });
        assert_eq!(result, Err(ParseError::NoTokens));
    }

    #[test]
    fn rejects_mixed_locale_tokens() {
        let result = ParsedDocument::new(ParsedDocumentCreate {
            document_id: "doc_2".to_string(),
            source: target(),
            locale: MorphologyLocale::PackSecondary,
            tokens: vec![
                token("term-c", MorphologyLocale::PackSecondary),
                token("hello", MorphologyLocale::Generic),
            ],
            body_byte_length: 20,
        });
        assert_eq!(result, Err(ParseError::InvalidLocaleForLemma));
    }

    #[test]
    fn rejects_empty_document_id() {
        let result = ParsedDocument::new(ParsedDocumentCreate {
            document_id: "".to_string(),
            source: target(),
            locale: MorphologyLocale::Generic,
            tokens: vec![token("hi", MorphologyLocale::Generic)],
            body_byte_length: 2,
        });
        assert_eq!(result, Err(ParseError::EmptyDocumentId));
    }
}
