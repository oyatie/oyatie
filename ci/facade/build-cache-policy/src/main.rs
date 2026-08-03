#![forbid(unsafe_code)]
//! Cache-wiring resolver + canary toolkit (ADR-0560). LOCAL BRIDGE feedback only
//! (founder CLI-retirement directive): merge authority is the conformance gate
//! test, never this binary; CI workflows invoke it as a job step (`buck2 run`),
//! and its successors are reconcilers per ADR-0556 D4 / ADR-0555 D4.
//!
//! Subcommands:
//!   resolve --build-class C [--emit-argfile PATH] [--require-bypass]
//!   license-state                       (prints `warm_licensed=<bool>` for $GITHUB_OUTPUT)
//!   report --record PATH --build-class C [--mode M] [--out PATH]
//!   assert-warm --record PATH --build-class C --mode M
//!   assert-cold --record PATH
//!   hash-outputs --show-output PATH [--out PATH]
//!   canary-verdict --cold PATH [--warm PATH --warm-record PATH --build-class C --mode M] [--out PATH]
//!   canary-targets                      (prints the pinned target set, one per line)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_build_cache_policy as app;
use serde_json::Value;

fn fail(message: &str) -> ExitCode {
    eprintln!("oya-cloud-ci-cache-wiring: {message}");
    ExitCode::from(2)
}

fn repo_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("current_dir: {e}"))?;
    app::repo_root_from(&cwd)
        .ok_or_else(|| "failed to locate repo root (specs/root-hub-pointers.json)".to_string())
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn read_json(path: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {path}: {e}"))
}

fn write_out(out: Option<String>, payload: &str) -> Result<(), String> {
    match out {
        Some(path) => {
            if let Some(parent) = Path::new(&path).parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
            }
            fs::write(&path, payload).map_err(|e| format!("write {path}: {e}"))
        }
        None => {
            println!("{payload}");
            Ok(())
        }
    }
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let Some(command) = args.first().cloned() else {
        return Err(
            "missing subcommand (resolve | license-state | report | assert-warm | \
                    assert-cold | hash-outputs | canary-verdict | canary-targets)"
                .to_string(),
        );
    };
    let rest = &args[1..];

    match command.as_str() {
        "resolve" => {
            let build_class = flag_value(rest, "--build-class")
                .ok_or_else(|| "resolve requires --build-class".to_string())?;
            let root = repo_root()?;
            let policy = app::load_policy(&root)?;
            let license = app::load_license(&root)?;
            let resolution = app::resolve(&policy, &license, &build_class)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&resolution.to_json())
                    .map_err(|e| format!("serialize resolution: {e}"))?
            );
            if let Some(argfile) = flag_value(rest, "--emit-argfile") {
                let cert = std::env::var(app::CLIENT_CERT_ENV).ok();
                let ca = std::env::var(app::TLS_CA_CERTS_ENV).ok();
                let lines = app::argfile_lines(&resolution, cert.as_deref(), ca.as_deref())?;
                let mut payload = lines.join("\n");
                if !payload.is_empty() {
                    payload.push('\n');
                }
                fs::write(&argfile, payload).map_err(|e| format!("write {argfile}: {e}"))?;
            }
            if has_flag(rest, "--require-bypass") && resolution.mode != app::CacheMode::Bypass {
                return Err(format!(
                    "--require-bypass: class `{build_class}` resolved `{}` — refusing",
                    resolution.mode
                ));
            }
            Ok(ExitCode::SUCCESS)
        }
        "license-state" => {
            let root = repo_root()?;
            let license = app::load_license(&root)?;
            let licensed = license
                .get("warm_reads_licensed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            println!("warm_licensed={licensed}");
            Ok(ExitCode::SUCCESS)
        }
        "report" => {
            let record_path = flag_value(rest, "--record")
                .ok_or_else(|| "report requires --record".to_string())?;
            let build_class = flag_value(rest, "--build-class")
                .ok_or_else(|| "report requires --build-class".to_string())?;
            let mode = flag_value(rest, "--mode").unwrap_or_else(|| "bypass".to_string());
            let doc = read_json(&record_path)?;
            let record = app::invocation_record(&doc)?;
            let report = app::cache_hit_report(record, &build_class, &mode);
            let payload = serde_json::to_string_pretty(&report)
                .map_err(|e| format!("serialize report: {e}"))?;
            write_out(flag_value(rest, "--out"), &payload)?;
            Ok(ExitCode::SUCCESS)
        }
        "assert-warm" => {
            let record_path = flag_value(rest, "--record")
                .ok_or_else(|| "assert-warm requires --record".to_string())?;
            let build_class = flag_value(rest, "--build-class")
                .ok_or_else(|| "assert-warm requires --build-class".to_string())?;
            let mode = flag_value(rest, "--mode")
                .ok_or_else(|| "assert-warm requires --mode".to_string())?;
            let doc = read_json(&record_path)?;
            let record = app::invocation_record(&doc)?;
            match app::assert_warm_cache_participation(record, &build_class, &mode) {
                Ok(()) => {
                    println!("warm-cache guard OK for class {build_class} in mode {mode}");
                    Ok(ExitCode::SUCCESS)
                }
                Err(findings) => {
                    for finding in findings {
                        eprintln!("{finding}");
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        "assert-cold" => {
            let record_path = flag_value(rest, "--record")
                .ok_or_else(|| "assert-cold requires --record".to_string())?;
            let doc = read_json(&record_path)?;
            let record = app::invocation_record(&doc)?;
            match app::assert_cold(record) {
                Ok(()) => {
                    println!("cold-proof OK: zero cache participation in {record_path}");
                    Ok(ExitCode::SUCCESS)
                }
                Err(findings) => {
                    for finding in findings {
                        eprintln!("{finding}");
                    }
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        "hash-outputs" => {
            let show_output = flag_value(rest, "--show-output")
                .ok_or_else(|| "hash-outputs requires --show-output".to_string())?;
            let text =
                fs::read_to_string(&show_output).map_err(|e| format!("read {show_output}: {e}"))?;
            let entries = app::digest_manifest_from_show_output(&text)?;
            let payload = serde_json::to_string_pretty(&app::manifest_to_json(&entries))
                .map_err(|e| format!("serialize manifest: {e}"))?;
            write_out(flag_value(rest, "--out"), &payload)?;
            Ok(ExitCode::SUCCESS)
        }
        "canary-verdict" => {
            let cold_path = flag_value(rest, "--cold")
                .ok_or_else(|| "canary-verdict requires --cold".to_string())?;
            let cold = app::manifest_from_json(&read_json(&cold_path)?)?;
            let warm = match flag_value(rest, "--warm") {
                Some(path) => Some(app::manifest_from_json(&read_json(&path)?)?),
                None => None,
            };
            // FAIL-CLOSED coupling to the kill-switch: while warm reads are
            // LICENSED, a verdict without a warm manifest is a misconfigured
            // canary (e.g. the probe step was dropped), not an INACTIVE state —
            // emitting INACTIVE (exit 0) there would let warm reads continue
            // without their trust anchor.
            if warm.is_none() {
                let root = repo_root()?;
                let license = app::load_license(&root)?;
                if license
                    .get("warm_reads_licensed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    return Err(
                        "canary-verdict: warm reads are LICENSED but no --warm manifest was \
                         supplied — the warm-probe step is missing or failed silently; refusing \
                         to emit INACTIVE while warmth is live (ADR-0556 D2)"
                            .to_string(),
                    );
                }
            }
            // PARTICIPATION PROOF, gating the verdict itself: a warm manifest is
            // scored only once its probe's invocation record proves the cache served
            // it. Byte-equality is satisfied by a probe that served nothing, so
            // without this the canary emits GREEN having proven nothing — the one
            // artifact that licenses warm reads fleet-wide (ADR-0556 D2).
            if warm.is_some() {
                let build_class = flag_value(rest, "--build-class").ok_or_else(|| {
                    "canary-verdict with --warm requires --build-class (the class the probe ran \
                     as); refusing to score a warm manifest without it"
                        .to_string()
                })?;
                let mode = flag_value(rest, "--mode").ok_or_else(|| {
                    "canary-verdict with --warm requires --mode (the cache mode the probe dialed); \
                     refusing to infer it — an unstated mode would skip the hit-count checks"
                        .to_string()
                })?;
                // No --warm-record, or a path buck2 never wrote, both land on `None`
                // / a hard read error. Neither may pass: see assert_warm_manifest_admissible.
                let doc = match flag_value(rest, "--warm-record") {
                    Some(path) => Some(read_json(&path)?),
                    None => None,
                };
                let record = match doc.as_ref() {
                    Some(doc) => Some(app::invocation_record(doc)?),
                    None => None,
                };
                if let Err(findings) =
                    app::assert_warm_manifest_admissible(record, &build_class, &mode)
                {
                    for finding in findings {
                        eprintln!("{finding}");
                    }
                    return Err(
                        "canary-verdict: the warm probe did not prove the cache served it — \
                         refusing to emit a verdict that would license warm reads (ADR-0556 D2)"
                            .to_string(),
                    );
                }
            }
            let (status, verdict) = app::canary_verdict(&cold, warm.as_ref());
            let payload = serde_json::to_string_pretty(&verdict)
                .map_err(|e| format!("serialize verdict: {e}"))?;
            write_out(flag_value(rest, "--out"), &payload)?;
            eprintln!("canary verdict: {}", status.as_str());
            if status.is_failure() {
                Ok(ExitCode::FAILURE)
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
        "canary-targets" => {
            let policy = app::canary_policy()?;
            let targets = policy
                .get("pinned_targets")
                .and_then(Value::as_array)
                .ok_or_else(|| "canary policy missing pinned_targets".to_string())?;
            for target in targets {
                if let Some(t) = target.as_str() {
                    println!("{t}");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown subcommand `{other}`")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(code) => code,
        Err(message) => fail(&message),
    }
}
