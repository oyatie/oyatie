//! pgroonga-class inverted index sharding (domain types).
//!
//! Per M03-P05-IP-001. Concrete pgroonga adapter lives in a downstream
//! adapter crate; this domain owns shard identity, posting lists, and
//! shard-key invariants.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use oya_search_parser_domain::{MorphologyLocale, ParsedDocument};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvertedPosting {
    pub document_id: String,    // data_class: INTERNAL_ONLY
    pub term_frequency: u32,    // data_class: INTERNAL_ONLY
    pub byte_offsets: Vec<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvertedIndexShard {
    pub shard_id: String,         // data_class: INTERNAL_ONLY
    pub tenant_id: String,        // data_class: INTERNAL_ONLY
    pub locale: MorphologyLocale, // data_class: INTERNAL_ONLY
    pub postings: BTreeMap<String, Vec<InvertedPosting>>, // data_class: INTERNAL_ONLY
    pub document_count: u32,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvertedIndexError {
    EmptyShardId,
    TenantMismatch,
    LocaleMismatch,
}

impl InvertedIndexShard {
    pub fn new(
        shard_id: String,
        tenant_id: String,
        locale: MorphologyLocale,
    ) -> Result<Self, InvertedIndexError> {
        if shard_id.trim().is_empty() {
            return Err(InvertedIndexError::EmptyShardId);
        }
        Ok(Self {
            shard_id,
            tenant_id,
            locale,
            postings: BTreeMap::new(),
            document_count: 0,
        })
    }

    pub fn ingest(&mut self, document: &ParsedDocument) -> Result<(), InvertedIndexError> {
        if document.tenant_id != self.tenant_id {
            return Err(InvertedIndexError::TenantMismatch);
        }
        if document.locale != self.locale {
            return Err(InvertedIndexError::LocaleMismatch);
        }
        let mut term_freqs: BTreeMap<String, (u32, Vec<u32>)> = BTreeMap::new();
        for token in &document.tokens {
            let entry = term_freqs
                .entry(token.lemma.clone())
                .or_insert((0u32, Vec::new()));
            entry.0 = entry.0.saturating_add(1);
            entry.1.push(token.byte_offset);
        }
        for (term, (term_frequency, byte_offsets)) in term_freqs {
            self.postings
                .entry(term)
                .or_default()
                .push(InvertedPosting {
                    document_id: document.document_id.clone(),
                    term_frequency,
                    byte_offsets,
                });
        }
        self.document_count = self.document_count.saturating_add(1);
        Ok(())
    }

    pub fn term_count(&self) -> usize {
        self.postings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_search_crawler_domain::{CrawlPriority, CrawlScheme, CrawlTarget, CrawlTargetCreate};
    use oya_search_parser_domain::{MorphologyToken, ParsedDocumentCreate, PartOfSpeech};
    use std::collections::BTreeMap;

    fn target(tenant: &str) -> CrawlTarget {
        CrawlTarget::new(CrawlTargetCreate {
            target_id: "tgt".to_string(),
            tenant_id: tenant.to_string(),
            scheme: CrawlScheme::Https,
            canonical_url: "https://x/".to_string(),
            priority: CrawlPriority::Normal,
            depth_budget: 1,
            headers: BTreeMap::new(),
        })
        .expect("valid")
    }

    fn doc(tenant: &str, locale: MorphologyLocale, surfaces: &[&str]) -> ParsedDocument {
        let tokens = surfaces
            .iter()
            .enumerate()
            .map(|(idx, s)| MorphologyToken {
                surface: (*s).to_string(),
                lemma: (*s).to_string(),
                locale,
                part_of_speech: PartOfSpeech::Noun,
                byte_offset: idx as u32 * 4,
            })
            .collect();
        ParsedDocument::new(ParsedDocumentCreate {
            document_id: "doc_a".to_string(),
            source: target(tenant),
            locale,
            tokens,
            body_byte_length: 100,
        })
        .expect("doc")
    }

    #[test]
    fn ingests_pack_primary_document_terms() {
        let mut shard = InvertedIndexShard::new(
            "shard_alpha_001".to_string(),
            "ten_alpha".to_string(),
            MorphologyLocale::PackPrimary,
        )
        .expect("shard");
        shard
            .ingest(&doc(
                "ten_alpha",
                MorphologyLocale::PackPrimary,
                &["term-a", "term-b", "term-a"],
            ))
            .expect("ingest ok");
        assert_eq!(shard.document_count, 1);
        assert_eq!(shard.term_count(), 2);
        let postings = shard.postings.get("term-a").unwrap();
        assert_eq!(postings[0].term_frequency, 2);
    }

    #[test]
    fn rejects_cross_tenant_ingest() {
        let mut shard = InvertedIndexShard::new(
            "shard_a".to_string(),
            "ten_alpha".to_string(),
            MorphologyLocale::PackPrimary,
        )
        .expect("shard");
        let err = shard
            .ingest(&doc("ten_beta", MorphologyLocale::PackPrimary, &["x"]))
            .expect_err("cross tenant rejected");
        assert_eq!(err, InvertedIndexError::TenantMismatch);
    }

    #[test]
    fn rejects_locale_mismatch() {
        let mut shard = InvertedIndexShard::new(
            "shard_generic".to_string(),
            "ten_gamma".to_string(),
            MorphologyLocale::Generic,
        )
        .expect("shard");
        let err = shard
            .ingest(&doc("ten_gamma", MorphologyLocale::PackPrimary, &["a"]))
            .expect_err("locale enforced");
        assert_eq!(err, InvertedIndexError::LocaleMismatch);
    }

    #[test]
    fn rejects_empty_shard_id() {
        let err = InvertedIndexShard::new(
            "".to_string(),
            "ten_alpha".to_string(),
            MorphologyLocale::PackPrimary,
        )
        .expect_err("shard id required");
        assert_eq!(err, InvertedIndexError::EmptyShardId);
    }
}
