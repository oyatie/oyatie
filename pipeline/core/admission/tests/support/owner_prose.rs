use std::collections::BTreeMap;

use pipeline_admission::{
    OwnerProseClaim, OwnerProseClassification, OwnerProseManifest, OwnerProseProducer,
    OwnerProseQualification, OwnerProseRepositoryBinding, OwnerProseRevision,
    OwnerProseRevisionBinding, OwnerProseSource, owner_prose_sha256, qualify_owner_prose,
};

pub const REPOSITORY: &str = "https://github.com/oyatie/oyatie.git";
pub const SOURCE_COMMIT: &str = "1111111111111111111111111111111111111111";
pub const SOURCE_TREE: &str = "2222222222222222222222222222222222222222";
pub const CANDIDATE_COMMIT: &str = "3333333333333333333333333333333333333333";
pub const CANDIDATE_TREE: &str = "4444444444444444444444444444444444444444";

pub struct Fixture {
    pub observed: OwnerProseRepositoryBinding,
    pub manifest: OwnerProseManifest,
    pub source: BTreeMap<String, Vec<u8>>,
    pub candidate: BTreeMap<String, Vec<u8>>,
}

impl Fixture {
    pub fn complete() -> Self {
        let observed = OwnerProseRepositoryBinding {
            identity: REPOSITORY.to_owned(),
            source: OwnerProseRevisionBinding {
                commit: SOURCE_COMMIT.to_owned(),
                tree: SOURCE_TREE.to_owned(),
            },
            candidate: OwnerProseRevisionBinding {
                commit: CANDIDATE_COMMIT.to_owned(),
                tree: CANDIDATE_TREE.to_owned(),
            },
        };
        let mut source = BTreeMap::new();
        let mut sources = Vec::new();
        for name in ["ADR.md", "PLAN.md", "PRD.md", "SPEC.md"] {
            let path = format!("policy/{name}");
            let bytes = format!("{name} current fact\n").into_bytes();
            let claim = OwnerProseClaim {
                id: format!(
                    "{}-current-fact",
                    name.trim_end_matches(".md").to_lowercase()
                ),
                start: 0,
                end: bytes.len(),
                sha256: owner_prose_sha256(&bytes),
                classification: OwnerProseClassification::HistoricalRejected,
                work_reference: None,
                projections: Vec::new(),
            };
            sources.push(OwnerProseSource {
                path: path.clone(),
                sha256: owner_prose_sha256(&bytes),
                claims: vec![claim],
            });
            source.insert(path, bytes);
        }
        let manifest = OwnerProseManifest {
            schema: "oyatie.owner-prose-classification.v1".to_owned(),
            repository: observed.clone(),
            producer: OwnerProseProducer {
                identity: "pipeline-owner-prose-classifier".to_owned(),
                schema: "oyatie.owner-prose-classifier.v1".to_owned(),
            },
            owner: "policy".to_owned(),
            sources,
        };
        Self {
            observed,
            manifest,
            source,
            candidate: BTreeMap::new(),
        }
    }

    pub fn manifest_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.manifest).expect("serialize fixture manifest")
    }

    pub fn qualify(&self) -> OwnerProseQualification {
        qualify_owner_prose(&self.manifest_bytes(), &self.observed, |revision, path| {
            Ok(match revision {
                OwnerProseRevision::Source => self.source.get(path).cloned(),
                OwnerProseRevision::Candidate => self.candidate.get(path).cloned(),
            })
        })
    }
}
