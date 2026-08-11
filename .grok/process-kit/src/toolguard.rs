//! Worker-lane tool refuse (cargo revival / buck2 in workers).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneRole {
    Orchestrator,
    Worker,
}

pub fn lane_role_from_env() -> LaneRole {
    match std::env::var("SWARM_ORCHESTRATOR").as_deref() {
        Ok("1") => LaneRole::Orchestrator,
        _ => LaneRole::Worker,
    }
}

/// Tools workers must not invoke (founder: no cargo revival; workers read err.txt only).
pub fn is_forbidden_worker_tool(argv0: &str) -> bool {
    let base = argv0.rsplit('/').next().unwrap_or(argv0);
    matches!(base, "cargo" | "cargo-clippy" | "rustc" | "buck" | "buck2")
}

pub fn refuse_worker_tool(argv0: &str) -> Result<(), String> {
    if lane_role_from_env() == LaneRole::Worker && is_forbidden_worker_tool(argv0) {
        return Err(format!(
            "toolguard: REFUSE — worker lane cannot invoke `{argv0}` (orchestrator/check-daemon only)"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn workers_denied_cargo() {
        let _g = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("SWARM_ORCHESTRATOR");
        }
        assert!(refuse_worker_tool("cargo").is_err());
        assert!(refuse_worker_tool("/usr/bin/buck2").is_err());
        unsafe {
            std::env::set_var("SWARM_ORCHESTRATOR", "1");
        }
        assert!(refuse_worker_tool("cargo").is_ok());
        unsafe {
            std::env::remove_var("SWARM_ORCHESTRATOR");
        }
    }
}
