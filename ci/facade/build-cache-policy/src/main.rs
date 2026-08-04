#![forbid(unsafe_code)]
//! Cache-wiring resolver + canary toolkit (ADR-0560). LOCAL BRIDGE feedback only
//! (founder CLI-retirement directive): merge authority is the conformance gate
//! test, never this binary; CI workflows invoke it as a job step (`buck2 run`),
//! and its successors are reconcilers per ADR-0556 D4 / ADR-0555 D4.
//!
//! Subcommands:
//!   resolve --build-class C [--require-bypass]
//!   run (--build-class C | --warm-probe) [--mode-out PATH] -- COMMAND [ARG...]
//!   license-state                       (prints `warm_licensed=<bool>` for $GITHUB_OUTPUT)
//!   report --record PATH --build-class C [--mode M] [--out PATH]
//!   assert-warm --record PATH --build-class C --mode M
//!   assert-cold --record PATH
//!   hash-outputs --show-output PATH [--out PATH]
//!   canary-verdict --cold PATH [--warm PATH --warm-record PATH] [--out PATH]
//!   canary-targets                      (prints the pinned target set, one per line)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};

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

fn child_exit(status: ExitStatus) -> ExitCode {
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from)
}

fn run_child(root: &Path, command: &[String]) -> Result<ExitStatus, String> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| "run requires a child command after `--`".to_string())?;
    Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|error| format!("spawn child `{program}`: {error}"))
}

fn controlled_buck2_command(command: &[String]) -> Result<(Vec<String>, String), String> {
    let (program, args) = command
        .split_first()
        .ok_or_else(|| "run requires a child command after `--`".to_string())?;
    if Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        != Some("buck2")
    {
        return Err(
            "warm controller child must be `buck2` so its daemon boundary is explicit".to_string(),
        );
    }
    let mut child = command.to_vec();
    let isolation = args
        .windows(2)
        .find(|pair| pair[0] == "--isolation-dir")
        .map(|pair| pair[1].clone())
        .or_else(|| {
            args.iter()
                .find_map(|arg| arg.strip_prefix("--isolation-dir=").map(str::to_owned))
        })
        .unwrap_or_else(|| {
            child.insert(1, "--isolation-dir".to_string());
            child.insert(2, "oya-cache-controlled".to_string());
            "oya-cache-controlled".to_string()
        });
    Ok((child, isolation))
}

fn kill_buck2(root: &Path, isolation: &str) -> Result<(), String> {
    let status = Command::new("buck2")
        .args(["--isolation-dir", isolation, "kill"])
        .current_dir(root)
        .status()
        .map_err(|error| format!("spawn `buck2 kill`: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "`buck2 --isolation-dir {isolation} kill` failed with {status}"
        ))
    }
}

fn controlled_child(
    root: &Path,
    resolution: &app::Resolution,
    command: &[String],
) -> Result<ExitCode, String> {
    let local = root.join(".buckconfig.local");
    if local.exists() {
        return Err(format!(
            "refusing pre-existing {}: controller must own the entire cache config lifecycle",
            local.display()
        ));
    }
    if resolution.mode == app::CacheMode::Bypass {
        return run_child(root, command).map(child_exit);
    }

    let overlay_path = match resolution.mode {
        app::CacheMode::WarmReadOnly => app::OVERLAY_RO_PATH,
        app::CacheMode::WarmReadWrite => app::OVERLAY_RW_PATH,
        app::CacheMode::Bypass => unreachable!(),
    };
    let overlay = fs::read_to_string(root.join(overlay_path))
        .map_err(|error| format!("read {overlay_path}: {error}"))?;
    let cert = std::env::var(app::CLIENT_CERT_ENV).ok();
    let ca = std::env::var(app::TLS_CA_CERTS_ENV).ok();
    let config = app::effective_buckconfig(resolution, &overlay, cert.as_deref(), ca.as_deref())?
        .ok_or_else(|| "warm resolution produced no effective config".to_string())?;

    // Buck2 reads RE client configuration only at daemon startup. The two kills
    // are the boundary: start the child after the private config exists, then
    // stop that daemon before deleting the config so a later cold child cannot
    // inherit warm state.
    let (child_command, isolation) = controlled_buck2_command(command)?;
    kill_buck2(root, &isolation)?;
    let path = app::install_local_buckconfig(root, &config)?;
    let child = run_child(root, &child_command);
    let stop = kill_buck2(root, &isolation);
    let remove = app::remove_local_buckconfig(&path);
    stop?;
    remove?;
    child.map(child_exit)
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let Some(command) = args.first().cloned() else {
        return Err(
            "missing subcommand (resolve | run | license-state | report | assert-warm | \
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
            if has_flag(rest, "--require-bypass") && resolution.mode != app::CacheMode::Bypass {
                return Err(format!(
                    "--require-bypass: class `{build_class}` resolved `{}` — refusing",
                    resolution.mode
                ));
            }
            Ok(ExitCode::SUCCESS)
        }
        "run" => {
            let separator = rest
                .iter()
                .position(|argument| argument == "--")
                .ok_or_else(|| "run requires `-- COMMAND [ARG...]`".to_string())?;
            let options = &rest[..separator];
            let child = &rest[separator + 1..];
            let root = repo_root()?;
            let policy = app::load_policy(&root)?;
            let license = app::load_license(&root)?;
            let resolution = if has_flag(options, "--warm-probe") {
                if !license
                    .get("warm_reads_licensed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    return Err("warm probe requires warm_reads_licensed=true".to_string());
                }
                app::Resolution {
                    build_class: format!(
                        "{}-warm-probe",
                        app::canary_class(&policy).ok_or_else(|| {
                            "policy missing trust_invariant.canary_build_class".to_string()
                        })?
                    ),
                    mode: app::CacheMode::WarmReadOnly,
                    reasons: vec!["licensed integrity-canary retrieval probe".to_string()],
                }
            } else {
                let build_class = flag_value(options, "--build-class")
                    .ok_or_else(|| "run requires --build-class or --warm-probe".to_string())?;
                app::resolve(&policy, &license, &build_class)?
            };
            if let Some(path) = flag_value(options, "--mode-out") {
                fs::write(&path, format!("{}\n", resolution.mode))
                    .map_err(|error| format!("write {path}: {error}"))?;
            }
            controlled_child(&root, &resolution, child)
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
            // FAIL-CLOSED on a NON-PARTICIPATING probe. A warm manifest proves nothing
            // about the cache unless the probe that produced it actually fetched from
            // the cache: canary_verdict compares target->OUTPUT-DIGEST pairs, and a probe
            // that served zero blobs and rebuilt everything locally yields digests
            // byte-identical to the cold build, full label overlap, zero divergence — and
            // therefore GREEN. That GREEN is the verdict that licenses warm reads
            // FLEET-WIDE, so it must never be reachable without proven participation.
            //
            // assert_warm_cache_participation is the existing, unit-tested predicate for
            // exactly this (cache_hit_rate != 0, run_action_cache_count != 0,
            // last_snapshot.re_action_cache_started != 0). It was written and never wired.
            if warm.is_some() {
                let record_path = flag_value(rest, "--warm-record").ok_or_else(|| {
                    "canary-verdict: a --warm manifest requires --warm-record (the probe's \
                     buck2 invocation record). Without it the probe's cache participation is \
                     unproven, and a zero-fetch local rebuild would emit GREEN and license \
                     warm reads fleet-wide (ADR-0556 D2)"
                        .to_string()
                })?;
                let record = read_json(&record_path)?;
                if let Err(findings) =
                    app::assert_warm_cache_participation(&record, "integrity-canary", "warm-ro")
                {
                    return Err(format!(
                        "canary-verdict: the warm probe did NOT participate in the cache, so its \
                         manifest is a local rebuild rather than evidence — refusing to compare \
                         it (ADR-0556 D2). Findings: {}",
                        findings.join("; ")
                    ));
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
