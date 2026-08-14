#![forbid(unsafe_code)]
//! SHARD-2 SOAK binary: prove 10 consecutive CLEAN QEMU cold boots of the verified unmodified
//! Asterinas v0.17.2 release ISO to a boot-ready userland, with admissible (non-simulated)
//! evidence. This EXTENDS the single-boot harness (`src/boot.rs`) into a soak: it verifies the
//! ISO ONCE, then for up to 3 whole-soak attempts spawns 10 fresh-VM cold boots each, killing
//! QEMU at the first boot-ready marker or the 180s deadline, and writes per-attempt + aggregate
//! receipts (plus an honest-fail gap-register on exhaustion).
//!
//! Owned-Rust / no-shell-no-python doctrine (ADR-0523): the ONLY non-Rust surface is invoking
//! `qemu-system-x86_64` via `std::process`. Each iteration is a NEW QEMU process with a read-only
//! CD-ROM and NO writable disk/snapshot, and a NEW per-iteration serial log (no state carry-over).
//! Raw serial bytes are captured verbatim to disk and referenced by path + recomputed sha256 —
//! never inlined into any output line.
//!
//! The soak loop, per-boot cleanliness derivation, attempt/aggregate receipt assembly, and
//! honest-fail gap-register are the PURE, QEMU-free `harness::soak` module (unit-tested without
//! QEMU). This binary supplies the only impure part: the real per-boot QEMU spawn/poll.
//!
//! Usage: `soak [ISO] [RUN_DIR]`.

use kernel_asterinas_boundary as pin;
use kernel_asterinas_real_boot as harness;
use kernel_asterinas_real_boot::soak;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Software-emulation resources (no KVM on the dev host; CPU-bound TCG), matching the single-boot
/// harness.
const MEM_MIB: u32 = 6144;
const SMP: u32 = 4;

const DEFAULT_ISO: &str = "kernel/target/artifacts/asterinas-nixos-0.17.2-x86_64.iso";
const DEFAULT_RUN_DIR_BASE: &str = "kernel/target/asterinas-soak/runs";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!(
                "[soak] SOAK NOT PROVEN \u{2014} aggregate receipt written overall_verdict=fail + honest-fail gap-register entry"
            );
            ExitCode::from(3)
        }
        Err(e) => {
            eprintln!("[soak] ERROR: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<bool, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let iso = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_ISO.to_string()));
    let run_dir = PathBuf::from(
        args.next()
            .unwrap_or_else(|| format!("{DEFAULT_RUN_DIR_BASE}/run-{}", harness::now_unix())),
    );
    fs::create_dir_all(&run_dir)?;

    // ---- Black-box integrity: recompute the ISO digest ONCE from disk and check the pin. The ISO
    // is fetched+verified by the AC1 fetch-verify harness; the soak reuses it unmodified read-only
    // across all boots (cold boot = fresh VM state, not re-download).
    if !iso.exists() {
        return Err(format!(
            "release ISO not present at {} (run the AC1 fetch-verify harness first)",
            iso.display()
        )
        .into());
    }
    eprintln!(
        "[soak] recomputing ISO sha256 ONCE from disk: {}",
        iso.display()
    );
    let (iso_sha256, iso_size) = harness::sha256_file(&iso)?;
    let iso_verified = harness::digests_match(pin::BOOT_ISO_SHA256, &iso_sha256);
    eprintln!(
        "[soak] iso bytes={iso_size} sha256={iso_sha256} verified_against_pin={iso_verified}"
    );
    if !iso_verified {
        return Err(format!(
            "ISO digest mismatch: expected {} got {iso_sha256} \u{2014} refusing to soak an unverified black-box artifact",
            pin::BOOT_ISO_SHA256
        )
        .into());
    }

    // ---- Build the QEMU invocation (permitted std::process external tool) and the soak config.
    let qargs = harness::build_qemu_args(&iso.to_string_lossy(), MEM_MIB, SMP);
    let qprog = harness::qemu_program();
    let reproducible_command = harness::render_command(qprog, &qargs);
    eprintln!("[soak] qemu command: {reproducible_command}");
    eprintln!(
        "[soak] {} iterations/attempt, up to {} attempts, per-boot timeout {}s",
        soak::ITERATION_COUNT,
        soak::MAX_SOAK_ATTEMPTS,
        soak::PER_BOOT_TIMEOUT_SECS
    );

    let cfg = soak::SoakConfig {
        release_tag: pin::RELEASE_TAG.to_string(),
        iso: soak::IsoArtifact {
            asset_name: pin::BOOT_ISO_ASSET.to_string(),
            download_url: pin::BOOT_ISO_DOWNLOAD_URL.to_string(),
            local_path: iso.to_string_lossy().into_owned(),
            expected_sha256: pin::BOOT_ISO_SHA256.to_string(),
            actual_sha256: iso_sha256,
            byte_size: iso_size,
            verified: iso_verified,
        },
        qemu_program: qprog.to_string(),
        qemu_args: qargs,
        reproducible_command,
        iteration_count: soak::ITERATION_COUNT,
        max_attempts: soak::MAX_SOAK_ATTEMPTS,
        per_boot_timeout_secs: soak::PER_BOOT_TIMEOUT_SECS,
        allowed_markers: pin::BOOT_READY_MARKERS
            .iter()
            .map(|m| m.to_string())
            .collect(),
    };
    let dests = soak::SoakDests {
        run_dir: run_dir.clone(),
    };

    // The real per-boot runner: the ONLY impure surface (QEMU spawn/poll). The pure soak loop +
    // receipt assembly + honest-fail live in `harness::soak`.
    let iso_for_runner = iso.clone();
    let outcome = soak::run_soak_with_boot_runner(
        &cfg,
        &dests,
        |cfg, attempt_id, iteration, attempt_dir| {
            execute_one_boot(cfg, attempt_id, iteration, &iso_for_runner, attempt_dir)
        },
    )?;

    // ---- Bounded summary into agent context (no full logs, no binary bytes).
    eprintln!(
        "[soak] overall_verdict={} passing_attempt_id={:?} attempts={}",
        outcome.verdict,
        outcome.passing_attempt_id,
        outcome.attempts.len()
    );
    for a in &outcome.attempts {
        let clean = a.boot_records.iter().filter(|r| r.clean).count();
        eprintln!(
            "[soak] {} verdict={} clean_boots={}/{} receipt={} sha256={}",
            a.attempt_id,
            a.verdict,
            clean,
            a.required_clean_boots,
            a.receipt_path,
            a.receipt_sha256
        );
    }
    eprintln!(
        "[soak] aggregate_receipt={} sha256={}",
        outcome.aggregate_receipt_path.display(),
        outcome.aggregate_receipt_sha256
    );
    if outcome.gap_register_written {
        eprintln!(
            "[soak] HONEST FAIL: gap-register entry embedded in aggregate (simulated_or_inferred_evidence_produced=false)"
        );
    }
    Ok(outcome.verdict == "pass")
}

/// Execute ONE fresh-VM cold boot: spawn a NEW QEMU on the verified read-only ISO, drive the
/// ISOLINUX boot-serial menu on a side thread, and poll until an early boot-ready marker, QEMU
/// self-exit, or the 180s deadline. Force-kill QEMU, then RE-DERIVE the marker from the FINAL
/// on-disk serial log and recompute its digest so the boot record is self-consistent with the
/// file it references. Returns the assembled [`soak::BootRecord`].
fn execute_one_boot(
    cfg: &soak::SoakConfig,
    attempt_id: &str,
    iteration: usize,
    iso: &Path,
    attempt_dir: &Path,
) -> Result<soak::BootRecord, Box<dyn Error>> {
    let iteration_dir = attempt_dir.join(format!("boot-{iteration:02}"));
    fs::create_dir_all(&iteration_dir)?;
    let serial_log = iteration_dir.join("serial.log");
    let qemu_stderr = iteration_dir.join("qemu.stderr.log");

    // Fresh process, read-only CD-ROM, no writable disk/snapshot: stdout -> serial log
    // (redirection), stdin -> keystroke channel. `iso` is used via cfg.qemu_args (which embed the
    // -cdrom path); it is passed explicitly to document that each boot mounts the same verified
    // ISO read-only.
    debug_assert!(cfg.qemu_args.iter().any(|a| Path::new(a) == iso));
    let out_file = fs::File::create(&serial_log)?;
    let err_file = fs::File::create(&qemu_stderr)?;
    let mut child: Child = Command::new(&cfg.qemu_program)
        .args(&cfg.qemu_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(out_file))
        .stderr(Stdio::from(err_file))
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {e}", cfg.qemu_program))?;
    eprintln!(
        "[soak] {attempt_id} boot-{iteration:02} qemu pid={} timeout={}s",
        child.id(),
        cfg.per_boot_timeout_secs
    );

    // ---- Navigation thread: wait for the boot menu, then send the boot-serial keystroke plan.
    let stop = Arc::new(AtomicBool::new(false));
    let stdin = child.stdin.take().expect("piped stdin");
    let nav_serial = serial_log.clone();
    let nav_stop = stop.clone();
    let nav_handle = thread::spawn(move || drive_serial_menu(stdin, &nav_serial, &nav_stop));

    // ---- Poll loop: early marker / QEMU self-exit / hard deadline.
    let start = Instant::now();
    let deadline = start + Duration::from_secs(cfg.per_boot_timeout_secs);
    let mut last_len = 0u64;
    // Assigned exactly once on whichever branch breaks the poll loop (self-exit / deadline /
    // early marker), so it is definitely initialized after the loop.
    let live;
    let mut self_exit: Option<ExitStatus> = None;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                self_exit = Some(status);
                live = soak::LiveTermination::QemuSelfExited;
                break;
            }
            Ok(None) => {}
            Err(_) => {}
        }
        if Instant::now() >= deadline {
            live = soak::LiveTermination::TimedOut;
            break;
        }
        if let Ok(meta) = fs::metadata(&serial_log) {
            let len = meta.len();
            if len != last_len {
                last_len = len;
                let s = harness::read_log_file(&serial_log).unwrap_or_default();
                if matches!(
                    harness::find_boot_marker(&s, &pin::BOOT_READY_MARKERS),
                    Ok(Some(_))
                ) {
                    live = soak::LiveTermination::MarkerKilled;
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(300));
    }
    let elapsed_seconds = start.elapsed().as_secs_f64();

    // Force-terminate QEMU (SIGKILL) unless it already self-exited; serial bytes already written
    // survive on disk.
    let exit_status = match self_exit {
        Some(s) => s,
        None => {
            let _ = child.kill();
            child.wait()?
        }
    };
    stop.store(true, Ordering::Relaxed);
    let _ = nav_handle.join();

    // RE-DERIVE from the FINAL on-disk serial log: recompute the digest + find the marker so the
    // record's matched_marker and digest are self-consistent with the file it references.
    let (serial_sha256, serial_size) = harness::sha256_file(&serial_log)?;
    let final_log = harness::read_log_file(&serial_log).unwrap_or_default();
    let marker = harness::find_boot_marker(&final_log, &pin::BOOT_READY_MARKERS)?;

    let record = soak::assemble_boot_record(soak::BootObservation {
        attempt_id: attempt_id.to_string(),
        iteration_index: iteration,
        live,
        marker,
        qemu_exit_status: qemu_exit_status(&exit_status),
        elapsed_seconds,
        serial_log_path: serial_log.to_string_lossy().into_owned(),
        serial_log_sha256: serial_sha256,
        serial_log_byte_size: serial_size,
    });
    eprintln!(
        "[soak] {attempt_id} boot-{iteration:02} clean={} termination={} elapsed={:.1}s marker={:?}",
        record.clean,
        record.termination_reason,
        record.elapsed_seconds,
        record
            .matched_marker
            .as_ref()
            .map(|m| m.matched_text.as_str())
    );
    Ok(record)
}

/// Convert a real `std::process::ExitStatus` into the pure receipt shape.
fn qemu_exit_status(status: &ExitStatus) -> soak::QemuExitStatus {
    soak::QemuExitStatus {
        observed: true,
        code: status.code(),
        signal: exit_signal(status),
        description: status.to_string(),
    }
}

#[cfg(unix)]
fn exit_signal(status: &ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn exit_signal(_status: &ExitStatus) -> Option<i32> {
    None
}

// ponytail: the boot-menu navigation + sliced-sleep helpers below are the same shape as
// `src/boot.rs`'s. They live in each binary because two `rust_binary` crate roots cannot share a
// private fn, and moving process-driving code into the pure library would break its QEMU-free
// unit-testability property. ~45 lines duplicated deliberately; fold into the lib only if a third
// QEMU-driving binary appears.

/// Wait for the ISOLINUX boot menu in the captured log, then send the keystroke plan that selects
/// the ISO's `boot-serial` entry (console=ttyS0). Holds the guest serial-input pipe open until
/// `stop`. The nav plan is the shared `harness::serial_menu_nav_plan()` (verbatim, so the exact
/// serial input is identical to the proven single-boot harness).
fn drive_serial_menu(mut stdin: std::process::ChildStdin, serial_log: &Path, stop: &AtomicBool) {
    // Installer label derived from the pinned tag (RELEASE_TAG is `vMAJOR.MINOR.PATCH`; the
    // on-screen label drops the leading `v`).
    let installer_label = format!(
        "Asterinas NixOS {} Installer",
        pin::RELEASE_TAG.trim_start_matches('v')
    );
    let menu_deadline = Instant::now() + Duration::from_secs(45);
    let mut menu_seen = false;
    while Instant::now() < menu_deadline && !stop.load(Ordering::Relaxed) {
        if let Ok(s) = harness::read_log_file(serial_log)
            && (s.contains(&installer_label) || s.contains("Options"))
        {
            menu_seen = true;
            break;
        }
        sleep_sliced(Duration::from_millis(300), stop);
    }
    if menu_seen {
        sleep_sliced(Duration::from_millis(600), stop);
    } else if !stop.load(Ordering::Relaxed) {
        // The menu never rendered within 45s: send the plan blind (best-effort) but flag it. A
        // blind send that lands on the wrong entry falls through to the VGA-only default -> no
        // serial marker -> honest unclean boot, so this is a diagnostic signal, not a false pass.
        eprintln!(
            "[soak] WARNING: ISOLINUX menu not detected on serial within 45s; sending boot-serial \
             keystroke plan blind (slow boot under TCG, or menu render missed)."
        );
    }
    for step in harness::serial_menu_nav_plan() {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        sleep_sliced(Duration::from_millis(step.delay_ms), stop);
        if stop.load(Ordering::Relaxed) {
            return;
        }
        if stdin.write_all(&step.bytes).is_err() {
            return;
        }
        let _ = stdin.flush();
    }
    // Keep the serial-input channel open for the rest of the boot.
    while !stop.load(Ordering::Relaxed) {
        sleep_sliced(Duration::from_millis(200), stop);
    }
}

/// Sleep in small slices so a `stop` signal is observed promptly (responsive thread join).
fn sleep_sliced(total: Duration, stop: &AtomicBool) {
    let slice = Duration::from_millis(100);
    let mut left = total;
    while left > Duration::ZERO && !stop.load(Ordering::Relaxed) {
        let s = left.min(slice);
        thread::sleep(s);
        left = left.saturating_sub(s);
    }
}
