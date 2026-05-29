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
use std::path::{Path, PathBuf};

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

/// Filesystem-backed verdict cache: one file per `verdict_key` containing
/// `PASS`/`FAIL`, so verdicts persist across `oya gate run-all` invocations
/// (the cross-run win). Wraps the same lookup/record contract; un-enumerable
/// gates are never persisted, so they always re-run.
pub(crate) struct FsVerdictCache {
    dir: PathBuf,
}

impl FsVerdictCache {
    pub(crate) fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub(crate) fn lookup(&self, inputs: &GateInputs) -> Option<Verdict> {
        let key = match cache_decision(inputs) {
            CacheDecision::AlwaysRun => return None,
            CacheDecision::Key(key) => key,
        };
        match std::fs::read_to_string(self.path_for(&key)) {
            Ok(s) if s.trim() == "PASS" => Some(Verdict::Pass),
            Ok(s) if s.trim() == "FAIL" => Some(Verdict::Fail),
            _ => None,
        }
    }

    pub(crate) fn record(&self, inputs: &GateInputs, verdict: Verdict) {
        let CacheDecision::Key(key) = cache_decision(inputs) else {
            return; // un-enumerable: never persisted
        };
        if std::fs::create_dir_all(&self.dir).is_err() {
            return; // cache is best-effort; never block the gate on a write error
        }
        let value = match verdict {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
        };
        let _ = std::fs::write(self.path_for(&key), value);
    }

    fn path_for(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.verdict"))
    }
}

/// Default on-disk cache directory under the workspace target dir.
pub(crate) fn default_cache_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("target").join("oya-gate-cache")
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
    fn identical_inputs_share_a_key() {
        let a = declared(vec![("a.txt", "x"), ("b.txt", "y")], "1");
        let b = declared(vec![("a.txt", "x"), ("b.txt", "y")], "1");
        assert_eq!(cache_decision(&a), cache_decision(&b));
    }

    #[test]
    fn declaration_order_is_irrelevant() {
        let a = declared(vec![("a.txt", "x"), ("b.txt", "y")], "1");
        let b = declared(vec![("b.txt", "y"), ("a.txt", "x")], "1");
        assert_eq!(cache_decision(&a), cache_decision(&b));
    }

    #[test]
    fn changed_file_content_changes_key() {
        let a = declared(vec![("a.txt", "x")], "1");
        let b = declared(vec![("a.txt", "CHANGED")], "1");
        assert_ne!(cache_decision(&a), cache_decision(&b)); // content => recompute
    }

    #[test]
    fn changed_tool_version_changes_key() {
        let a = declared(vec![("a.txt", "x")], "1");
        let b = declared(vec![("a.txt", "x")], "2");
        assert_ne!(cache_decision(&a), cache_decision(&b)); // version bump invalidates
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
    fn unenumerable_is_never_cacheable() {
        // No key => the lane always runs; no false PASS is possible.
        assert_eq!(
            cache_decision(&GateInputs::Unenumerable),
            CacheDecision::AlwaysRun
        );
    }

    #[test]
    fn fs_cache_persists_across_instances() {
        let dir = std::env::temp_dir().join(format!("oya-gate-cache-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let a = declared(vec![("a.txt", "x")], "1");

        let writer = FsVerdictCache::new(&dir);
        assert_eq!(writer.lookup(&a), None); // cold
        writer.record(&a, Verdict::Pass);

        // a fresh instance (simulating a later `gate run-all` run) sees the hit
        let reader = FsVerdictCache::new(&dir);
        assert_eq!(reader.lookup(&a), Some(Verdict::Pass));

        // content change misses
        let b = declared(vec![("a.txt", "CHANGED")], "1");
        assert_eq!(reader.lookup(&b), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fs_cache_never_persists_unenumerable() {
        let dir =
            std::env::temp_dir().join(format!("oya-gate-cache-unenum-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let fs = FsVerdictCache::new(&dir);
        fs.record(&GateInputs::Unenumerable, Verdict::Pass);
        assert_eq!(fs.lookup(&GateInputs::Unenumerable), None); // always re-runs
        let _ = std::fs::remove_dir_all(&dir);
    }
}
