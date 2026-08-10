//! Hand-rolled CLI for `port-engine-app` (W0-B Slice 10).
//!
//! Bridge feedback only — never merge authority (CLI surfaces are retirement-marked). No clap /
//! argv crate: keep the facade free of new lock-forcing deps.

use std::process::ExitCode;

use crate::driver;
use crate::receipt_e2e;

const USAGE: &str = "\
port-engine-app — owned deterministic port-engine driver (W0-B Slice 10)

Usage:
  port-engine-app <command> [args]

Commands:
  help              Show this usage
  ready             Print adapter readiness (exit 0 when wired)
  pin               Print fleet upstream pin (peeled commit)
  emit-stub         Smoke empty-renderer emit via kernel
  emit-syn          Smoke syn/quote typed emit
  digest <text>     SHA-256 digest of UTF-8 text (Slice 7 hash adapter)
  rulepack          Load fixture-gated rulepack v0; print digest + fixture count
  plan              Plan embedded rulepack against example units
  admit-snapshot    Admit hermetic OOB bootstrap snapshot fixture
  engine            Print Slice 9 engine identity digest
  toolchain         Print Slice 9 dual-home toolchain corpus digest
  pipeline          pin→admit→plan→emit→six-axis receipt
  receipt           Alias for pipeline (print receipt axes)
  verify-e2e        Run six-axis receipt end-to-end scenarios

Exit codes: 0 ok · 1 error · 2 usage
";

/// Dispatch argv (skipping argv[0]). Returns a process exit code.
#[must_use]
pub fn run(args: &[String]) -> ExitCode {
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    match cmd {
        "help" | "-h" | "--help" => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        "ready" => cmd_ready(),
        "pin" => cmd_pin(),
        "emit-stub" => cmd_emit_stub(),
        "emit-syn" => cmd_emit_syn(),
        "digest" => cmd_digest(args.get(1).map(String::as_str)),
        "rulepack" => cmd_rulepack(),
        "plan" => cmd_plan(),
        "admit-snapshot" => cmd_admit_snapshot(),
        "engine" => cmd_engine(),
        "toolchain" => cmd_toolchain(),
        "pipeline" | "receipt" => cmd_pipeline(),
        "verify-e2e" => cmd_verify_e2e(),
        other => {
            eprintln!("port-engine-app: unknown command `{other}`");
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn cmd_ready() -> ExitCode {
    if !driver::w0_ready() {
        eprintln!("port-engine-app: driver not ready");
        return ExitCode::from(1);
    }
    let (pin, rust_ir, frontend, hash, rulepack, snapshot, identity, toolchain) =
        driver::adapter_readiness();
    println!(
        "ready=true pin={pin} rust_ir={rust_ir} frontend_go={frontend} hash={hash} rulepack={rulepack} snapshot={snapshot} identity={identity} toolchain={toolchain}"
    );
    ExitCode::SUCCESS
}

fn cmd_pin() -> ExitCode {
    match driver::fleet_pin() {
        Ok(pin) => {
            println!("{pin}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: pin failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn cmd_emit_stub() -> ExitCode {
    match driver::smoke_render_stub() {
        Ok(()) => {
            println!("emit-stub=ok");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: emit-stub failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn cmd_emit_syn() -> ExitCode {
    match driver::smoke_syn_quote_render() {
        Ok(()) => {
            println!("emit-syn=ok");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: emit-syn failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn cmd_digest(text: Option<&str>) -> ExitCode {
    let Some(text) = text else {
        eprintln!("port-engine-app: digest requires <text>");
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let digest = driver::smoke_digest(text);
    println!("{}", digest.0);
    ExitCode::SUCCESS
}

fn cmd_rulepack() -> ExitCode {
    match driver::smoke_rulepack() {
        Ok((digest, fixtures)) => {
            println!(
                "rulepack=ok digest={} selecting_fixtures={}",
                digest.0, fixtures
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: rulepack failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn cmd_plan() -> ExitCode {
    match driver::smoke_plan() {
        Ok(steps) => {
            println!("plan=ok steps={steps}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: plan failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn cmd_admit_snapshot() -> ExitCode {
    match driver::smoke_admit_snapshot() {
        Ok(admitted) => {
            println!(
                "admit-snapshot=ok pin={} digest={} units={}",
                admitted.pin,
                admitted.snapshot_digest.0,
                admitted.units().len()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: admit-snapshot failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn cmd_engine() -> ExitCode {
    let digest = driver::smoke_engine_digest();
    println!("engine=ok digest={}", digest.0);
    ExitCode::SUCCESS
}

fn cmd_toolchain() -> ExitCode {
    let digest = driver::smoke_toolchain_digest();
    println!("toolchain=ok digest={}", digest.0);
    ExitCode::SUCCESS
}

fn cmd_pipeline() -> ExitCode {
    match driver::smoke_pipeline() {
        Ok(report) => {
            let r = &report.receipt;
            println!(
                "pipeline=ok plan_steps={} emit_regions={}",
                report.plan_steps, report.emit_regions
            );
            println!("pin={}", r.pin);
            println!("snapshot_digest={}", r.snapshot_digest.0);
            println!("engine_digest={}", r.engine_digest.0);
            println!("rulepack_digest={}", r.rulepack_digest.0);
            println!("toolchain_digest={}", r.toolchain_digest.0);
            println!("formatter_digest={}", r.formatter_digest.0);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: pipeline failed: {err}");
            ExitCode::from(1)
        }
    }
}

fn cmd_verify_e2e() -> ExitCode {
    match receipt_e2e::run_six_axis_e2e() {
        Ok(report) => {
            println!("verify-e2e=ok pin={}", report.pin);
            for s in &report.scenarios {
                println!(
                    "scenario={} verdict={:?} delta={:?}",
                    s.name, s.verification.verdict, s.verification.delta
                );
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: verify-e2e failed: {err}");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn help_and_ready_succeed() {
        assert_eq!(run(&args(&["help"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["ready"])), ExitCode::SUCCESS);
    }

    #[test]
    fn slice10_commands_succeed() {
        assert_eq!(run(&args(&["digest", "port-engine"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["rulepack"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["plan"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["admit-snapshot"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["engine"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["toolchain"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["pipeline"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["receipt"])), ExitCode::SUCCESS);
        assert_eq!(run(&args(&["verify-e2e"])), ExitCode::SUCCESS);
    }

    #[test]
    fn unknown_command_is_usage() {
        assert_eq!(run(&args(&["not-a-command"])), ExitCode::from(2));
    }

    #[test]
    fn digest_without_arg_is_usage() {
        assert_eq!(run(&args(&["digest"])), ExitCode::from(2));
    }
}
