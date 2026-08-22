---
doc_status: published
---

# Fitness Lane: authority-cohesion

- status: Accepted
- date: 2026-05-12
- purpose: Verify every doc that claims authority over a topic resolves cleanly (no overlapping authoritative-doc claims for the same topic).
- enforces: STANDARD/single-source-authority; AGENTS.md fitness-lane `governance-authority-cohesion`.
- kernel_crate: `governance-authority-cohesion-kernel` (EXISTING) — `AuthorityClaim { topic, path }`, verdict `AuthorityCohesionFitnessReport { topics_checked }`.
- runner_path: `tools/governance-authority-cohesion`
- inputs: `docs/**/*.md` front-matter `authoritative_for:`, registry of topic ids.
- failure_modes:
  - two docs claim authority over the same topic
  - claim references unknown topic id
  - topic has zero authoritative-doc rows
- ci_invocation: `cargo run -p governance-authority-cohesion`
- runtime_budget: 350 ms
- severity: BLOCKER
- kernel_sketch:
```rust
pub struct AuthorityClaim {
    pub topic: String,  // data_class: INTERNAL_ONLY
    pub path: String,   // data_class: INTERNAL_ONLY
}

pub struct AuthorityCohesionFitnessReport { pub topics_checked: usize }

pub enum AuthorityCohesionFitnessError {
    ConflictingAuthority { topic: String, paths: Vec<String> },
    UnknownTopic { topic: String, path: String },
    UnclaimedTopic { topic: String },
}

pub fn validate_authority_cohesion_fitness(
    claims: &[AuthorityClaim],
    known_topics: &[String],
) -> Result<AuthorityCohesionFitnessReport, AuthorityCohesionFitnessError> {
    use std::collections::BTreeMap;
    let known: std::collections::BTreeSet<&str> = known_topics.iter().map(|s| s.as_str()).collect();
    let mut by_topic: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in claims {
        if !known.contains(c.topic.as_str()) {
            return Err(AuthorityCohesionFitnessError::UnknownTopic {
                topic: c.topic.clone(), path: c.path.clone(),
            });
        }
        by_topic.entry(c.topic.clone()).or_default().push(c.path.clone());
    }
    for (topic, paths) in &by_topic {
        if paths.len() > 1 {
            return Err(AuthorityCohesionFitnessError::ConflictingAuthority {
                topic: topic.clone(), paths: paths.clone(),
            });
        }
    }
    for t in known_topics {
        if !by_topic.contains_key(t) {
            return Err(AuthorityCohesionFitnessError::UnclaimedTopic { topic: t.clone() });
        }
    }
    Ok(AuthorityCohesionFitnessReport { topics_checked: known_topics.len() })
}
```
