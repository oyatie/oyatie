#![forbid(unsafe_code)]
//! AC2/AC3 harness binary: boot the verified unmodified Asterinas v0.17.2 release ISO ONCE in
//! QEMU, capture the raw serial console verbatim to a log file by redirection, derive a boot
//! verdict from the closed boot-ready marker set, and write the boot + envelope receipts (or a
//! gap-register entry on honest failure).
//!
//! Owned-Rust / no-shell-no-python doctrine (ADR-0523): the ONLY non-Rust surface is invoking
//! `qemu-system-x86_64` via `std::process`. The guest's first serial port is bound to the
//! child's stdio: stdout is REDIRECTED to the serial log file (the raw verbatim capture) and
//! stdin drives the ISO's own `boot-serial` menu entry so the kernel/userland log to ttyS0.
//! Raw binary bytes and full logs are NEVER inlined into any output line — the ISO and serial
//! log live only on disk and are referenced by path + recomputed sha256.
//!
//! Usage: `boot [ISO] [SERIAL_LOG] [BOOT_RECEIPT] [ENVELOPE_RECEIPT] [GAP_REGISTER]`.

use kernel_asterinas_boundary as pin;
use kernel_asterinas_real_boot as harness;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

/// Hard boot timeout: QEMU is force-terminated at this deadline and the captured log up to that
/// point is the evidence (resource-envelope numeric limit).
///
/// Margin is empirical, not arbitrary: on this dev host under `-accel tcg` (no KVM) a clean
/// boot reaches its boot-ready marker in ~58s for the single-boot run and ~42–44s per boot
/// across the 10-boot soak, so 180s leaves ~3x headroom over observed worst-case. A boot that
/// blows the deadline is force-killed and honestly recorded (`timeout_hit`/`killed-at-deadline`),
/// never silently passed — so an under-margin deadline degrades to an honest fail, not a false
/// pass. Revisit if a faster/slower emulator or a heavier installer image shifts the observed p100.
const BOOT_TIMEOUT_SECS: u64 = 180;
/// Bounded excerpt cap: at most this many head + tail lines may be surfaced into agent context.
const EXCERPT_LINES: usize = 80;
/// Software-emulation resources (no KVM on the dev host; CPU-bound TCG).
const MEM_MIB: u32 = 6144;
const SMP: u32 = 4;

const DEFAULT_ISO: &str = "kernel/target/artifacts/asterinas-nixos-0.17.2-x86_64.iso";
const DEFAULT_SERIAL_LOG: &str = "kernel/target/artifacts/boot-serial-v0.17.2.log";
const DEFAULT_QEMU_STDERR: &str = "kernel/target/artifacts/boot-qemu-stderr-v0.17.2.log";
const DEFAULT_EXCERPT: &str = "kernel/target/artifacts/boot-serial-v0.17.2.excerpt.txt";
const DEFAULT_BOOT_RECEIPT: &str = "kernel/asterinas/harness/asterinas-real-boot/receipts/boot-v0.17.2.json";
const DEFAULT_ENVELOPE_RECEIPT: &str =
    "kernel/asterinas/harness/asterinas-real-boot/receipts/envelope-v0.17.2.json";
const DEFAULT_GAP_REGISTER: &str = "specs/kuberos-asterinas-wave1-gap-register.json";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!(
                "[boot] BOOT NOT READY \u{2014} boot receipt written status=fail + gap register entry (honest failure)"
            );
            ExitCode::from(3)
        }
        Err(e) => {
            eprintln!("[boot] ERROR: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<bool, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let iso = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_ISO.to_string()));
    let serial_log = PathBuf::from(
        args.next()
            .unwrap_or_else(|| DEFAULT_SERIAL_LOG.to_string()),
    );
    let boot_receipt_dest = PathBuf::from(
        args.next()
            .unwrap_or_else(|| DEFAULT_BOOT_RECEIPT.to_string()),
    );
    let envelope_receipt_dest = PathBuf::from(
        args.next()
            .unwrap_or_else(|| DEFAULT_ENVELOPE_RECEIPT.to_string()),
    );
    let gap_register_dest = PathBuf::from(
        args.next()
            .unwrap_or_else(|| DEFAULT_GAP_REGISTER.to_string()),
    );
    let qemu_stderr = PathBuf::from(DEFAULT_QEMU_STDERR);
    let excerpt_dest = PathBuf::from(DEFAULT_EXCERPT);

    for p in [
        &serial_log,
        &boot_receipt_dest,
        &envelope_receipt_dest,
        &gap_register_dest,
        &qemu_stderr,
        &excerpt_dest,
    ] {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
    }

    // ---- Black-box integrity: recompute the ISO digest from on-disk bytes and check the pin.
    if !iso.exists() {
        return Err(format!(
            "release ISO not present at {} (run the AC1 fetch-verify harness first)",
            iso.display()
        )
        .into());
    }
    eprintln!("[boot] recomputing ISO sha256 from disk: {}", iso.display());
    let (iso_sha256, iso_size) = harness::sha256_file(&iso)?;
    let iso_verified = harness::digests_match(pin::BOOT_ISO_SHA256, &iso_sha256);
    eprintln!(
        "[boot] iso bytes={iso_size} sha256={iso_sha256} verified_against_pin={iso_verified}"
    );

    // ---- Build the QEMU invocation (permitted std::process external tool) and record it.
    let qargs = harness::build_qemu_args(&iso.to_string_lossy(), MEM_MIB, SMP);
    let qprog = harness::qemu_program();
    let reproducible_command = harness::render_command(qprog, &qargs);
    let nav_plan = harness::serial_menu_nav_plan();
    eprintln!("[boot] qemu command: {reproducible_command}");
    eprintln!(
        "[boot] serial-console selection: drive ISOLINUX 'boot-serial' entry over serial stdin ({} keystroke steps)",
        nav_plan.len()
    );

    let booted_at = harness::now_unix();
    let start = Instant::now();

    // Either boot (verified ISO) or emit honest failure directly (digest mismatch). Both paths
    // converge on ONE finalize call built from named-field structs (no positional scalar list).
    let (wall_secs, exit_reason) = if iso_verified {
        boot_qemu_and_capture(qprog, &qargs, &serial_log, &qemu_stderr, start)?
    } else {
        // Do not boot on a digest mismatch. Truncate any prior capture fail-closed (a swallowed
        // error could leave a stale marker-bearing log on disk).
        fs::write(&serial_log, b"")?;
        (0u64, String::from("not-booted-iso-digest-mismatch"))
    };

    let iso_path = iso.to_string_lossy();
    let allowed: Vec<&str> = pin::BOOT_READY_MARKERS.to_vec();
    let attempt = harness::BootAttempt {
        release_tag: pin::RELEASE_TAG,
        boot_iso_asset: pin::BOOT_ISO_ASSET,
        iso_path: &iso_path,
        expected_iso_sha256: pin::BOOT_ISO_SHA256,
        actual_iso_sha256: &iso_sha256,
        iso_byte_size: iso_size,
        iso_verified,
        qemu_program: qprog,
        qemu_args: &qargs,
        reproducible_command: &reproducible_command,
        nav_plan: &nav_plan,
        allowed_markers: &allowed,
        timeout_secs: BOOT_TIMEOUT_SECS,
        wall_secs,
        qemu_exit: &exit_reason,
        booted_at_unix: booted_at,
        excerpt_lines: EXCERPT_LINES,
    };
    let dests = harness::EvidenceDests {
        serial_log: &serial_log,
        excerpt: &excerpt_dest,
        boot_receipt: &boot_receipt_dest,
        envelope_receipt: &envelope_receipt_dest,
        gap_register: &gap_register_dest,
    };
    finalize(&attempt, &dests)
}

/// Spawn QEMU on the verified ISO, drive the ISOLINUX boot-serial menu on a side thread, and
/// poll until an early boot-ready marker, QEMU self-exit, or the hard deadline. The raw serial
/// bytes are captured to `serial_log` by stdout redirection. Returns `(wall_secs, exit_reason)`.
fn boot_qemu_and_capture(
    qprog: &str,
    qargs: &[String],
    serial_log: &Path,
    qemu_stderr: &Path,
    start: Instant,
) -> Result<(u64, String), Box<dyn Error>> {
    // stdout -> serial log file (redirection); stdin -> keystroke channel.
    let out_file = fs::File::create(serial_log)?;
    let err_file = fs::File::create(qemu_stderr)?;
    let mut child: Child = Command::new(qprog)
        .args(qargs)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(out_file))
        .stderr(Stdio::from(err_file))
        .spawn()
        .map_err(|e| format!("failed to spawn {qprog}: {e}"))?;
    eprintln!(
        "[boot] qemu pid={} started; timeout={BOOT_TIMEOUT_SECS}s",
        child.id()
    );

    // ---- Navigation thread: wait for the boot menu, then send the keystroke plan.
    let stop = Arc::new(AtomicBool::new(false));
    let stdin = child.stdin.take().expect("piped stdin");
    let nav_serial = serial_log.to_path_buf();
    let nav_stop = stop.clone();
    let nav_handle = thread::spawn(move || drive_serial_menu(stdin, &nav_serial, &nav_stop));

    // ---- Poll loop: detect an early marker, QEMU self-exit, or the hard deadline.
    let deadline = start + Duration::from_secs(BOOT_TIMEOUT_SECS);
    let mut last_len = 0u64;
    // Default reason is the hard-deadline kill; overwritten only on QEMU self-exit or an early
    // marker, so the deadline branch simply breaks and keeps this default.
    let mut exit_reason = String::from("killed-at-deadline");
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                exit_reason = match status.code() {
                    Some(c) => format!("exited-code-{c}"),
                    None => "exited-signal".to_string(),
                };
                break;
            }
            Ok(None) => {}
            Err(_) => {}
        }
        if Instant::now() >= deadline {
            break;
        }
        if let Ok(meta) = fs::metadata(serial_log) {
            let len = meta.len();
            if len != last_len {
                last_len = len;
                let s = harness::read_log_file(serial_log).unwrap_or_default();
                if matches!(
                    harness::find_boot_marker(&s, &pin::BOOT_READY_MARKERS),
                    Ok(Some(_))
                ) {
                    exit_reason = "killed-at-marker".to_string();
                    break;
                }
            }
        }
        thread::sleep(Duration::from_millis(300));
    }

    let wall_secs = start.elapsed().as_secs();
    // Force-terminate QEMU (SIGKILL); serial bytes already written survive on disk.
    let _ = child.kill();
    let _ = child.wait();
    stop.store(true, Ordering::Relaxed);
    let _ = nav_handle.join();
    eprintln!("[boot] qemu stopped: reason={exit_reason} wall={wall_secs}s");
    Ok((wall_secs, exit_reason))
}

/// Delegate to the library finalize orchestration: derive the verdict from the FINAL on-disk
/// serial log, recompute its digest, write the boot + envelope receipts (and a gap-register
/// entry when the boot is not ready), then surface a bounded summary into agent context.
///
/// Takes the already-built [`harness::BootAttempt`] / [`harness::EvidenceDests`] (named fields,
/// so same-typed fields cannot be transposed) rather than a long positional scalar list.
fn finalize(
    attempt: &harness::BootAttempt,
    dests: &harness::EvidenceDests,
) -> Result<bool, Box<dyn Error>> {
    let excerpt_dest = dests.excerpt;
    let serial_log = dests.serial_log;
    let boot_receipt_dest = dests.boot_receipt;
    let envelope_receipt_dest = dests.envelope_receipt;
    let gap_register_dest = dests.gap_register;
    let outcome = harness::finalize_boot_evidence(attempt, dests)?;

    // Fail-closed receipt digest self-consistency was already enforced inside the orchestration;
    // echo the proof (recorded == recomputed-from-disk) into agent context.
    eprintln!(
        "[boot] serial-log digest self-consistency OK: recorded==recomputed {} ({} bytes)",
        outcome.digest_verification.actual_sha256, outcome.digest_verification.byte_size
    );
    if outcome.gap_register_written {
        eprintln!("[boot] gap register -> {}", gap_register_dest.display());
    }

    // ---- Compact summary into agent context (bounded; no full log, no binary bytes).
    let head = outcome.head_lines.len().min(EXCERPT_LINES);
    let tail = outcome.tail_lines.len().min(EXCERPT_LINES);
    eprintln!(
        "[boot] serial_log={} bytes={} sha256={}",
        serial_log.display(),
        outcome.serial_log_byte_size,
        outcome.serial_log_sha256
    );
    eprintln!(
        "[boot] serial_log_lines={} excerpt(head/tail)={head}/{tail} -> {}",
        outcome.total_lines,
        excerpt_dest.display()
    );
    match &outcome.verdict.marker {
        Some(m) => eprintln!(
            "[boot] BOOT REACHED: marker[{}]={} line={} matched_text={:?}",
            m.marker_index, m.marker_pattern, m.line_number, m.matched_text
        ),
        None => eprintln!("[boot] BOOT NOT REACHED: no closed-set marker present"),
    }
    eprintln!(
        "[boot] grep match counts per marker: {:?}",
        outcome.marker_hit_counts
    );
    eprintln!(
        "[boot] receipts: {} , {}",
        boot_receipt_dest.display(),
        envelope_receipt_dest.display()
    );
    eprintln!(
        "[boot] status={}",
        if outcome.boot_reached { "pass" } else { "fail" }
    );

    Ok(outcome.boot_reached)
}

/// Wait for the ISOLINUX boot menu in the captured log, then send the keystroke plan that
/// selects the ISO's `boot-serial` entry. Holds the guest serial-input pipe open until `stop`.
fn drive_serial_menu(mut stdin: std::process::ChildStdin, serial_log: &Path, stop: &AtomicBool) {
    // The installer menu label is derived from the pinned release tag (not a hardcoded version)
    // so it stays aligned with the pin: RELEASE_TAG is `vMAJOR.MINOR.PATCH`, the on-screen label
    // drops the leading `v` ("Asterinas NixOS 0.17.2 Installer").
    let installer_label = format!(
        "Asterinas NixOS {} Installer",
        pin::RELEASE_TAG.trim_start_matches('v')
    );
    // Wait up to 45s for the boot menu to render on serial.
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
        // The menu never rendered within 45s: send the keystroke plan anyway (best-effort), but
        // flag it distinctly so a slow/degraded boot is not confused with a real boot failure.
        // A blind send that lands on the wrong entry falls through to the VGA-only default → no
        // serial marker → honest fail, so this is a diagnostic signal, not a correctness risk.
        eprintln!(
            "[boot] WARNING: ISOLINUX menu not detected on serial within 45s; sending boot-serial \
             keystroke plan blind (slow boot under TCG, or menu render missed). If the boot then \
             yields no boot-ready marker, treat this as the likely cause, not a kernel failure."
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
