//! Rust-first process-kit primitives (ADR-0711 / `templates/portable-swarm-doctrine.md`).
//!
//! Shell `tools/swarm/**` birth was aborted on #1644 (automation-language ceiling).
//! Forever home: `ci/process-kit/**` under `roots.ci` → `integ/ci`
//! (BAN agent-dotdirs as forever homes).

pub mod claim_push;
pub mod git_shim;

use std::env;
use std::path::Path;

/// Ambient env escapes that lane shells must refuse (guardrails-env-escape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvEscape {
    BlessedPushInherited,
    GitRealRetarget { value: String },
}

/// Detect lane-env escapes. Call at lane-shell entry (Rust shim successor).
pub fn detect_env_escapes(swarm_lane: bool) -> Vec<EnvEscape> {
    let mut out = Vec::new();
    if env::var_os("SWARM_BLESSED_PUSH").is_some() {
        out.push(EnvEscape::BlessedPushInherited);
    }
    if swarm_lane {
        if let Ok(v) = env::var("GIT_REAL") {
            let resolved = resolve_real_git();
            if Path::new(&v) != Path::new(&resolved) {
                out.push(EnvEscape::GitRealRetarget { value: v });
            }
        }
    }
    out
}

/// Pin real git to an allowlisted absolute path (no ambient retarget).
pub fn resolve_real_git() -> String {
    for candidate in ["/usr/bin/git", "/bin/git"] {
        if Path::new(candidate).is_file() {
            return candidate.to_string();
        }
    }
    "git".to_string()
}

/// Orchestrator-only admission for the optional shared check daemon.
pub fn require_orchestrator() -> Result<(), String> {
    match env::var("SWARM_ORCHESTRATOR") {
        Ok(v) if v == "1" => Ok(()),
        _ => Err(
            "check-daemon: REFUSE — set SWARM_ORCHESTRATOR=1 (worker Cargo checks remain allowed)"
                .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn detects_blessed_push_inheritance() {
        let _g = ENV_LOCK.lock().unwrap();
        unsafe {
            env::remove_var("SWARM_BLESSED_PUSH");
        }
        assert!(detect_env_escapes(false).is_empty());
        unsafe {
            env::set_var("SWARM_BLESSED_PUSH", "1");
        }
        assert_eq!(
            detect_env_escapes(false),
            vec![EnvEscape::BlessedPushInherited]
        );
        unsafe {
            env::remove_var("SWARM_BLESSED_PUSH");
        }
    }

    #[test]
    fn orchestrator_gate() {
        let _g = ENV_LOCK.lock().unwrap();
        // Isolate from CI/host env that may already export SWARM_ORCHESTRATOR.
        unsafe {
            env::remove_var("SWARM_ORCHESTRATOR");
        }
        let denied = require_orchestrator();
        assert!(
            denied.is_err(),
            "expected refuse without SWARM_ORCHESTRATOR=1, got {denied:?}"
        );
        unsafe {
            env::set_var("SWARM_ORCHESTRATOR", "1");
        }
        // Re-read via require_orchestrator (not a cached view); some runners
        // only surface env changes on the next var::var call after set_var.
        let allowed = require_orchestrator();
        assert_eq!(
            env::var("SWARM_ORCHESTRATOR").ok().as_deref(),
            Some("1"),
            "set_var did not stick before require_orchestrator"
        );
        assert!(
            allowed.is_ok(),
            "expected admit with SWARM_ORCHESTRATOR=1, got {allowed:?}"
        );
        unsafe {
            env::remove_var("SWARM_ORCHESTRATOR");
        }
    }
}
