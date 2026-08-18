//! Commands that exercise ONE seam: is this adapter wired, and what does it answer?

use std::process::ExitCode;

use crate::cli::USAGE;
use crate::driver;

pub(crate) fn cmd_ready() -> ExitCode {
    if !driver::w0_ready() {
        eprintln!("port-engine-app: driver not ready");
        return ExitCode::from(1);
    }
    let (pin, rust_ir, frontend, hash, rulepack, snapshot, identity, toolchain, transform, emit) =
        driver::adapter_readiness();
    println!(
        "ready=true pin={pin} rust_ir={rust_ir} frontend_go={frontend} hash={hash} rulepack={rulepack} snapshot={snapshot} identity={identity} toolchain={toolchain} transform={transform} emit={emit}"
    );
    ExitCode::SUCCESS
}

pub(crate) fn cmd_pin() -> ExitCode {
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

pub(crate) fn cmd_digest(text: Option<&str>) -> ExitCode {
    let Some(text) = text else {
        eprintln!("port-engine-app: digest requires <text>");
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let digest = driver::smoke_digest(text);
    println!("{}", digest.0);
    ExitCode::SUCCESS
}

pub(crate) fn cmd_rulepack() -> ExitCode {
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

pub(crate) fn cmd_plan() -> ExitCode {
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

pub(crate) fn cmd_admit_snapshot() -> ExitCode {
    match driver::smoke_admit_snapshot() {
        Ok(admitted) => {
            println!(
                "admit-snapshot=ok pin={} digest={} units={}",
                admitted.pin(),
                admitted.artifact_digest().0,
                admitted.as_model().units().len()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: admit-snapshot failed: {err}");
            ExitCode::from(1)
        }
    }
}

pub(crate) fn cmd_declarations() -> ExitCode {
    match driver::smoke_declarations() {
        Ok((admitted, summary)) => {
            println!(
                "declarations=ok digest={} units={}",
                admitted.model_digest().0,
                summary.len()
            );
            for (unit, count) in summary {
                println!("  {unit} declares {count}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("port-engine-app: declarations failed: {err}");
            ExitCode::from(1)
        }
    }
}

pub(crate) fn cmd_engine() -> ExitCode {
    let digest = driver::smoke_engine_digest();
    println!("engine=ok digest={}", digest.0);
    ExitCode::SUCCESS
}

pub(crate) fn cmd_toolchain() -> ExitCode {
    let digest = driver::smoke_toolchain_digest();
    println!("toolchain=ok digest={}", digest.0);
    ExitCode::SUCCESS
}

pub(crate) fn cmd_emit_stub() -> ExitCode {
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

pub(crate) fn cmd_emit_syn() -> ExitCode {
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
