//! Rust-first process-kit primitives (ADR-0711 / PORTABLE-SWARM-CONTRACT).
//!
//! Shell `tools/swarm/**` birth was aborted on #1644 (automation-language ceiling).
//! This crate is the forever home under `roots.grok` → `integ/ci`.

pub mod claim_push;
pub mod git_shim;
pub mod toolguard;

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

/// Orchestrator-only check-daemon admission.
pub fn require_orchestrator() -> Result<(), String> {
    match env::var("SWARM_ORCHESTRATOR") {
        Ok(v) if v == "1" => Ok(()),
        _ => Err(
            "check-daemon: REFUSE — set SWARM_ORCHESTRATOR=1 (worker lanes forbidden)"
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
        unsafe {
            env::remove_var("SWARM_ORCHESTRATOR");
        }
        assert!(require_orchestrator().is_err());
        unsafe {
            env::set_var("SWARM_ORCHESTRATOR", "1");
        }
        assert!(require_orchestrator().is_ok());
        unsafe {
            env::remove_var("SWARM_ORCHESTRATOR");
        }
    }
}
