//! Live CI long-step telemetry wrapper.
//!
//! Runs an arbitrary command while periodically printing a phase + elapsed-seconds heartbeat so
//! GitHub Actions exposes progress before the enclosing job finishes. It is an adapter-only
//! visibility helper: the wrapped command's exit status remains the merge-authoritative verdict.
#![forbid(unsafe_code)]

use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use ci_affected_target_set::long_step_telemetry_line;

const LOG: &str = "ci-step-telemetry";

#[derive(Debug)]
struct Args {
    phase: String,
    interval: Duration,
    command: Vec<String>,
}

fn parse_args<I>(mut argv: I) -> Result<Args, String>
where
    I: Iterator<Item = String>,
{
    let _bin = argv.next();
    let mut phase = None;
    let mut interval = Duration::from_secs(30);
    let mut command = Vec::new();

    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--phase" => phase = Some(argv.next().ok_or("--phase needs a value")?),
            "--interval-seconds" => {
                let raw = argv.next().ok_or("--interval-seconds needs a value")?;
                let seconds = raw.parse::<u64>().map_err(|_| {
                    format!("--interval-seconds must be a positive integer, got `{raw}`")
                })?;
                if seconds == 0 {
                    return Err("--interval-seconds must be greater than zero".to_owned());
                }
                interval = Duration::from_secs(seconds);
            }
            "--" => {
                command.extend(argv);
                break;
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let phase = phase.ok_or("--phase <name> is required")?;
    if command.is_empty() {
        return Err("a command after `--` is required".to_owned());
    }

    Ok(Args {
        phase,
        interval,
        command,
    })
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args()) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{LOG}: ARGS ERROR: {e}");
            eprintln!(
                "{LOG}: usage: cloud-ci-step-telemetry --phase <name> [--interval-seconds N] -- <command> [args...]"
            );
            return ExitCode::from(2);
        }
    };

    ExitCode::from(u8::try_from(run_child(args)).unwrap_or(1))
}

fn run_child(args: Args) -> i32 {
    let pretty = args.command.join(" ");
    let started = Instant::now();
    println!(
        "{}",
        long_step_telemetry_line(LOG, &args.phase, "started", 0, &format!("command={pretty}"))
    );

    let mut child = match Command::new(&args.command[0])
        .args(&args.command[1..])
        .stdin(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("{LOG}: FAIL — could not execute `{pretty}`: {e}");
            return 1;
        }
    };

    let poll_interval = if args.interval < Duration::from_millis(250) {
        args.interval
    } else {
        Duration::from_millis(250)
    };
    let mut last_running_emit = started;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                println!(
                    "{}",
                    long_step_telemetry_line(
                        LOG,
                        &args.phase,
                        "completed",
                        started.elapsed().as_secs(),
                        &format!("exit_status={status}"),
                    )
                );
                return status.code().unwrap_or(1);
            }
            Ok(None) => {
                if last_running_emit.elapsed() >= args.interval {
                    println!(
                        "{}",
                        long_step_telemetry_line(
                            LOG,
                            &args.phase,
                            "running",
                            started.elapsed().as_secs(),
                            &format!("command={pretty}"),
                        )
                    );
                    last_running_emit = Instant::now();
                }
                thread::sleep(poll_interval);
            }
            Err(e) => {
                let _ = child.kill();
                eprintln!("{LOG}: FAIL — could not poll `{pretty}`: {e}");
                return 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> impl Iterator<Item = String> {
        std::iter::once("cloud-ci-step-telemetry".to_owned())
            .chain(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn parse_args_accepts_phase_interval_and_command_passthrough() {
        let args = parse_args(argv(&[
            "--phase",
            "binding-build",
            "--interval-seconds",
            "7",
            "--",
            "buck2",
            "test",
            "//...",
        ]))
        .expect("args should parse");

        assert_eq!(args.phase, "binding-build");
        assert_eq!(args.interval, Duration::from_secs(7));
        assert_eq!(args.command, vec!["buck2", "test", "//..."]);
    }

    #[test]
    fn parse_args_rejects_missing_phase() {
        let error =
            parse_args(argv(&["--", "buck2", "test"])).expect_err("missing phase should fail");
        assert!(error.contains("--phase <name> is required"));
    }

    #[test]
    fn parse_args_rejects_zero_interval() {
        let error = parse_args(argv(&[
            "--phase",
            "binding-build",
            "--interval-seconds",
            "0",
            "--",
            "buck2",
            "test",
        ]))
        .expect_err("zero interval should fail");
        assert!(error.contains("--interval-seconds must be greater than zero"));
    }

    #[test]
    fn parse_args_rejects_missing_command() {
        let error = parse_args(argv(&["--phase", "binding-build", "--"]))
            .expect_err("missing command should fail");
        assert!(error.contains("a command after `--` is required"));
    }

    #[test]
    fn run_child_preserves_nonzero_exit_code() {
        let args = Args {
            phase: "exit-code-preservation".to_owned(),
            interval: Duration::from_secs(30),
            command: vec![
                "python3".to_owned(),
                "-c".to_owned(),
                "import sys; sys.exit(7)".to_owned(),
            ],
        };

        assert_eq!(run_child(args), 7);
    }
}
