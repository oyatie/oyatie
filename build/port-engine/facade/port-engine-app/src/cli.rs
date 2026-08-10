//! Hand-rolled CLI for `port-engine-app` (W0-B Slice 6).
//!
//! Bridge feedback only — never merge authority (CLI surfaces are retirement-marked). No clap /
//! argv crate: keep the facade dependency-free so Slice 6 does not force a Cargo.lock absorb.

use std::process::ExitCode;

use crate::driver;
use crate::receipt_e2e;

const USAGE: &str = "\
port-engine-app — owned deterministic port-engine driver (W0-B Slice 6)

Usage:
  port-engine-app <command>

Commands:
  help         Show this usage
  ready        Print adapter readiness (exit 0 when wired)
  pin          Print fleet upstream pin (peeled commit)
  emit-stub    Smoke empty-renderer emit via kernel
  emit-syn     Smoke syn/quote typed emit
  verify-e2e   Run six-axis receipt end-to-end scenarios

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
    let (pin, rust_ir, frontend) = driver::adapter_readiness();
    println!("ready=true pin={pin} rust_ir={rust_ir} frontend_go={frontend}");
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

    fn args(cmd: &str) -> Vec<String> {
        vec![cmd.to_owned()]
    }

    #[test]
    fn help_and_ready_succeed() {
        assert_eq!(run(&args("help")), ExitCode::SUCCESS);
        assert_eq!(run(&args("ready")), ExitCode::SUCCESS);
    }

    #[test]
    fn verify_e2e_command_succeeds() {
        assert_eq!(run(&args("verify-e2e")), ExitCode::SUCCESS);
    }

    #[test]
    fn unknown_command_is_usage() {
        assert_eq!(run(&args("not-a-command")), ExitCode::from(2));
    }
}
