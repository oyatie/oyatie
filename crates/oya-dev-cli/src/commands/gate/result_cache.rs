//! Content-addressed gate-result cache (ADR-0360 O7).
//!
//! Makes repeated gate execution O(changed) instead of O(repo): hash a gate's
//! declared input set (+ tool version + config + whitelisted env) into a
//! `verdict_key`, cache the PASS/FAIL verdict under it, and skip the gate on a
//! hit (Bazel action-cache model / Turborepo task hashing).
//!
//! LOAD-BEARING CORRECTNESS RULE (best-practice research): a gate is cacheable
//! ONLY if it declares ALL of its inputs and is deterministic. A gate that
//! cannot enumerate its inputs is [`GateInputs::Unenumerable`] and is NEVER
//! cached — it always runs. We never risk a false PASS to save CI time; the
//! cost of a wrong green dwarfs the saving. Per-file gates declare their files;
//! global/cross-corpus gates must declare the whole corpus as their input.
//!
//! This module is the mechanism + its correctness proof (unit tests). Adoption
//! is per-gate and opt-in (a gate must supply a [`GateInputs::Declared`]); until
//! a gate opts in, nothing changes — default behaviour is unaffected.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

/// A gate's verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Verdict {
    Pass,
    Fail,
}

/// What a gate declares about its inputs, for cache-key computation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GateInputs {
    /// All inputs are enumerable and the gate is deterministic.
    Declared {
        /// (repo-relative path, file contents) for every file the gate reads.
        files: Vec<(String, Vec<u8>)>,
        /// The gate implementation/tool version (bump to invalidate).
        tool_version: String,
        /// Digest of the gate's configuration.
        config_digest: String,
        /// Whitelisted environment variables the gate's verdict depends on.
        env: BTreeMap<String, String>,
    },
    /// Inputs cannot be fully enumerated — NEVER cached, always run.
    Unenumerable,
}

/// Whether a gate's verdict may be cached, and under what key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CacheDecision {
    /// Cacheable; this is the content-addressed verdict key.
    Key(String),
    /// Not cacheable — the gate must always run.
    AlwaysRun,
}

/// Decide cacheability and compute the verdict key from declared inputs.
pub(crate) fn cache_decision(inputs: &GateInputs) -> CacheDecision {
    let (files, tool_version, config_digest, env) = match inputs {
        GateInputs::Unenumerable => return CacheDecision::AlwaysRun,
        GateInputs::Declared {
            files,
            tool_version,
            config_digest,
            env,
        } => (files, tool_version, config_digest, env),
    };

    let mut h = Sha256::new();
    h.update(b"oya-gate-verdict-v1\0");
    h.update(tool_version.as_bytes());
    h.update([0]);
    h.update(config_digest.as_bytes());
    h.update([0]);
    // env is a BTreeMap => already key-sorted, so the key is order-independent.
    for (k, v) in env {
        h.update(k.as_bytes());
        h.update([0]);
        h.update(v.as_bytes());
        h.update([0]);
    }
    // Sort files by path so declaration order never changes the key.
    let mut sorted: Vec<&(String, Vec<u8>)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (path, content) in sorted {
        h.update(path.as_bytes());
        h.update([0]);
        h.update(Sha256::digest(content)); // content digest (Merkle leaf)
        h.update([0]);
    }
    CacheDecision::Key(format!("{:x}", h.finalize()))
}

/// In-memory verdict cache. A filesystem/remote store can wrap the same
/// `lookup`/`record` contract; the correctness lives here.
#[derive(Clone, Debug, Default)]
pub(crate) struct VerdictCache {
    map: BTreeMap<String, Verdict>,
}

impl VerdictCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Returns a cached verdict if the gate is cacheable AND a matching entry
    /// exists. Un-enumerable gates always miss (so they always run).
    pub(crate) fn lookup(&self, inputs: &GateInputs) -> Option<Verdict> {
        match cache_decision(inputs) {
            CacheDecision::AlwaysRun => None,
            CacheDecision::Key(key) => self.map.get(&key).copied(),
        }
    }

    /// Records a verdict. Un-enumerable gates are never stored.
    pub(crate) fn record(&mut self, inputs: &GateInputs, verdict: Verdict) {
        if let CacheDecision::Key(key) = cache_decision(inputs) {
            self.map.insert(key, verdict);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(files: Vec<(&str, &str)>, version: &str) -> GateInputs {
        GateInputs::Declared {
            files: files
                .into_iter()
                .map(|(p, c)| (p.to_string(), c.as_bytes().to_vec()))
                .collect(),
            tool_version: version.to_string(),
            config_digest: "cfg-1".to_string(),
            env: BTreeMap::new(),
        }
    }

    #[test]
    fn identical_inputs_hit() {
        let a = declared(vec![("a.txt", "x"), ("b.txt", "y")], "1");
        let mut cache = VerdictCache::new();
        assert_eq!(cache.lookup(&a), None); // cold miss
        cache.record(&a, Verdict::Pass);
        assert_eq!(cache.lookup(&a), Some(Verdict::Pass)); // warm hit
    }

    #[test]
    fn declaration_order_is_irrelevant() {
        let a = declared(vec![("a.txt", "x"), ("b.txt", "y")], "1");
        let b = declared(vec![("b.txt", "y"), ("a.txt", "x")], "1");
        assert_eq!(cache_decision(&a), cache_decision(&b));
    }

    #[test]
    fn changed_file_content_misses() {
        let a = declared(vec![("a.txt", "x")], "1");
        let b = declared(vec![("a.txt", "CHANGED")], "1");
        let mut cache = VerdictCache::new();
        cache.record(&a, Verdict::Pass);
        assert_eq!(cache.lookup(&b), None); // content changed => recompute
    }

    #[test]
    fn changed_tool_version_misses() {
        let a = declared(vec![("a.txt", "x")], "1");
        let b = declared(vec![("a.txt", "x")], "2");
        let mut cache = VerdictCache::new();
        cache.record(&a, Verdict::Pass);
        assert_eq!(cache.lookup(&b), None); // version bump invalidates
    }

    #[test]
    fn changed_env_misses() {
        let mut a_env = BTreeMap::new();
        a_env.insert("FEATURE".to_string(), "on".to_string());
        let a = GateInputs::Declared {
            files: vec![("a.txt".into(), b"x".to_vec())],
            tool_version: "1".into(),
            config_digest: "cfg".into(),
            env: a_env,
        };
        let b = GateInputs::Declared {
            files: vec![("a.txt".into(), b"x".to_vec())],
            tool_version: "1".into(),
            config_digest: "cfg".into(),
            env: BTreeMap::new(),
        };
        assert_ne!(cache_decision(&a), cache_decision(&b));
    }

    #[test]
    fn unenumerable_is_never_cached() {
        let mut cache = VerdictCache::new();
        // Even after recording a PASS, an un-enumerable gate always misses,
        // so it always runs — no false PASS is possible.
        cache.record(&GateInputs::Unenumerable, Verdict::Pass);
        assert_eq!(cache.lookup(&GateInputs::Unenumerable), None);
        assert_eq!(
            cache_decision(&GateInputs::Unenumerable),
            CacheDecision::AlwaysRun
        );
    }

    #[test]
    fn recorded_fail_is_reused() {
        let a = declared(vec![("a.txt", "x")], "1");
        let mut cache = VerdictCache::new();
        cache.record(&a, Verdict::Fail);
        assert_eq!(cache.lookup(&a), Some(Verdict::Fail));
    }
}
