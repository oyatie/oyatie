//! pgvector-class vector index with tenant-private isolation (domain types).
//!
//! Per M03-P05-IP-002. The shard owns embeddings keyed by `(tenant_id,
//! object_type)`; cross-tenant insert is a hard error to honour the data
//! boundary (ADR-0008).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use oya_search_parser_domain::ParsedDocument;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VectorDistance {
    Cosine,
    InnerProduct,
    L2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorRecord {
    pub document_id: String, // data_class: INTERNAL_ONLY
    pub embedding: Vec<f32>, // data_class: INTERNAL_ONLY
    pub dimension: u16,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorIndex {
    pub index_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub object_type: String,                     // data_class: INTERNAL_ONLY
    pub dimension: u16,                          // data_class: INTERNAL_ONLY
    pub distance: VectorDistance,                // data_class: INTERNAL_ONLY
    pub records: BTreeMap<String, VectorRecord>, // data_class: INTERNAL_ONLY
    pub tenant_private: bool,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, PartialEq)]
pub enum VectorIndexError {
    EmptyIndexId,
    DimensionZero,
    DimensionMismatch,
    TenantBoundaryViolation,
    EmptyDocumentId,
}

impl VectorIndex {
    pub fn new(
        index_id: String,
        tenant_id: String,
        object_type: String,
        dimension: u16,
        distance: VectorDistance,
    ) -> Result<Self, VectorIndexError> {
        if index_id.trim().is_empty() {
            return Err(VectorIndexError::EmptyIndexId);
        }
        if dimension == 0 {
            return Err(VectorIndexError::DimensionZero);
        }
        Ok(Self {
            index_id,
            tenant_id,
            object_type,
            dimension,
            distance,
            records: BTreeMap::new(),
            tenant_private: true,
        })
    }

    pub fn upsert(
        &mut self,
        document: &ParsedDocument,
        embedding: Vec<f32>,
    ) -> Result<(), VectorIndexError> {
        if document.tenant_id != self.tenant_id {
            return Err(VectorIndexError::TenantBoundaryViolation);
        }
        if document.document_id.trim().is_empty() {
            return Err(VectorIndexError::EmptyDocumentId);
        }
        if embedding.len() as u16 != self.dimension {
            return Err(VectorIndexError::DimensionMismatch);
        }
        self.records.insert(
            document.document_id.clone(),
            VectorRecord {
                document_id: document.document_id.clone(),
                embedding,
                dimension: self.dimension,
            },
        );
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_search_crawler_domain::{CrawlPriority, CrawlScheme, CrawlTarget, CrawlTargetCreate};
    use oya_search_parser_domain::{
        MorphologyLocale, MorphologyToken, ParsedDocumentCreate, PartOfSpeech,
    };
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

    fn doc(id: &str, tenant: &str) -> ParsedDocument {
        ParsedDocument::new(ParsedDocumentCreate {
            document_id: id.to_string(),
            source: target(tenant),
            locale: MorphologyLocale::En,
            tokens: vec![MorphologyToken {
                surface: "hi".to_string(),
                lemma: "hi".to_string(),
                locale: MorphologyLocale::En,
                part_of_speech: PartOfSpeech::Noun,
                byte_offset: 0,
            }],
            body_byte_length: 2,
        })
        .expect("doc")
    }

    #[test]
    fn upserts_record_within_tenant() {
        let mut index = VectorIndex::new(
            "idx_a".to_string(),
            "ten_kr".to_string(),
            "Post".to_string(),
            3,
            VectorDistance::Cosine,
        )
        .expect("index");
        index
            .upsert(&doc("doc_1", "ten_kr"), vec![0.1, 0.2, 0.3])
            .expect("upsert ok");
        assert_eq!(index.len(), 1);
        assert!(index.tenant_private);
    }

    #[test]
    fn rejects_cross_tenant_upsert() {
        let mut index = VectorIndex::new(
            "idx_b".to_string(),
            "ten_kr".to_string(),
            "Post".to_string(),
            3,
            VectorDistance::Cosine,
        )
        .expect("index");
        let err = index
            .upsert(&doc("doc_1", "ten_us"), vec![0.0, 0.0, 0.0])
            .expect_err("cross-tenant rejected");
        assert_eq!(err, VectorIndexError::TenantBoundaryViolation);
        assert!(index.is_empty());
    }

    #[test]
    fn rejects_dimension_mismatch() {
        let mut index = VectorIndex::new(
            "idx_c".to_string(),
            "ten_kr".to_string(),
            "Post".to_string(),
            4,
            VectorDistance::L2,
        )
        .expect("index");
        let err = index
            .upsert(&doc("doc_1", "ten_kr"), vec![1.0, 2.0])
            .expect_err("dim enforced");
        assert_eq!(err, VectorIndexError::DimensionMismatch);
    }

    #[test]
    fn rejects_zero_dimension() {
        let err = VectorIndex::new(
            "idx_d".to_string(),
            "ten_kr".to_string(),
            "Post".to_string(),
            0,
            VectorDistance::Cosine,
        )
        .expect_err("dimension positive");
        assert_eq!(err, VectorIndexError::DimensionZero);
    }
}
