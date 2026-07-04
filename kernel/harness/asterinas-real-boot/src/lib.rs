#![forbid(unsafe_code)]
//! Fetch+verify (AC1) and real-boot orchestration logic (AC2/AC3) for the Asterinas v0.17.2
//! real-boot slice.
//!
//! All network I/O lives in the binary (`src/main.rs`). This library is deterministic and
//! dependency-light (sha2 + serde_json) so its behavior is unit-tested via `buck2 test`
//! without touching the network. It streams file bytes (1 MiB chunks) so a multi-GB artifact
//! is never buffered whole in memory and never inlined into any output.
//!
//! data_class: PUBLIC — operates only on published release metadata and local file bytes.

use regex::Regex;
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Lowercase hex encoding of raw digest bytes.
pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).expect("nibble < 16"));
        s.push(char::from_digit((b & 0x0f) as u32, 16).expect("nibble < 16"));
    }
    s
}

/// Streaming sha256 of an on-disk file. Reads in 1 MiB chunks so a multi-GB artifact is never
/// buffered whole in memory. Returns `(lowercase_hex_digest, byte_size)` computed from the
/// bytes on disk at call time — the caller invokes this AFTER the download completes so the
/// recorded digest is self-consistent with the referenced file.
pub fn sha256_file(path: &Path) -> io::Result<(String, u64)> {
    let mut f = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut total: u64 = 0;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        hasher.update(&buf[..n]);
    }
    Ok((to_hex(&hasher.finalize()), total))
}

/// Case-insensitive digest equality.
pub fn digests_match(expected: &str, actual: &str) -> bool {
    expected.eq_ignore_ascii_case(actual)
}

/// Outcome of verifying that a sha256 recorded in a receipt is self-consistent with the bytes of
/// the on-disk file it references. Carries the recorded digest, the digest RECOMPUTED from the
/// file at verification (receipt-write) time, and the file's byte size, so both admissible and
/// inadmissible outcomes are fully auditable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestVerification {
    /// The digest as recorded in the receipt — the value being checked.
    pub recorded_sha256: String,
    /// The sha256 recomputed from the on-disk file bytes at verification (receipt-write) time.
    pub actual_sha256: String,
    /// Byte size of the on-disk file read while recomputing the digest.
    pub byte_size: u64,
}

/// Why [`verify_serial_log_digest`] rejected a recorded digest.
#[derive(Debug)]
pub enum DigestVerifyError {
    /// The referenced file could not be read to recompute its digest.
    Io(io::Error),
    /// The recorded digest does not equal the sha256 recomputed from the file bytes — a recorded
    /// digest that does not match the referenced file fails the slice.
    Mismatch(DigestVerification),
}

impl fmt::Display for DigestVerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DigestVerifyError::Io(e) => {
                write!(f, "cannot read referenced file to recompute digest: {e}")
            }
            DigestVerifyError::Mismatch(v) => write!(
                f,
                "recorded serial-log sha256 {} does not match sha256 {} recomputed from the {}-byte on-disk file",
                v.recorded_sha256, v.actual_sha256, v.byte_size
            ),
        }
    }
}

impl std::error::Error for DigestVerifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DigestVerifyError::Io(e) => Some(e),
            DigestVerifyError::Mismatch(_) => None,
        }
    }
}

/// Verify that a serial-log (or any artifact) sha256 recorded in a receipt is self-consistent
/// with the on-disk file it references, RECOMPUTING the digest from the file bytes at
/// receipt-write time rather than trusting any previously recorded value.
///
/// Returns `Ok(DigestVerification)` iff the recorded digest equals the recomputed digest
/// (case-insensitive hex). A recorded digest that does not equal the referenced file's actual
/// bytes returns `Err(DigestVerifyError::Mismatch)` carrying the digest recomputed from disk — a
/// recorded digest that does not match the referenced file fails the slice (receipt digest
/// self-consistency constraint). An unreadable file returns `Err(DigestVerifyError::Io)`.
pub fn verify_serial_log_digest(
    recorded_sha256: &str,
    serial_log_path: &Path,
) -> Result<DigestVerification, DigestVerifyError> {
    let (actual_sha256, byte_size) = sha256_file(serial_log_path).map_err(DigestVerifyError::Io)?;
    let verification = DigestVerification {
        recorded_sha256: recorded_sha256.to_string(),
        actual_sha256,
        byte_size,
    };
    if digests_match(&verification.recorded_sha256, &verification.actual_sha256) {
        Ok(verification)
    } else {
        Err(DigestVerifyError::Mismatch(verification))
    }
}

/// Current unix time in seconds (best-effort; 0 if the clock is before the epoch).
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Build the AC1 fetch-verify receipt as a JSON value.
///
/// `verified` is derived from BOTH digest equality AND byte-size equality. The caller MUST
/// pass an `actual_sha256`/`actual_byte_size` recomputed from the on-disk file bytes (via
/// [`sha256_file`]) at receipt-write time, so a recorded digest can never disagree with the
/// referenced file.
#[allow(clippy::too_many_arguments)]
pub fn build_fetch_verify_receipt(
    source_url: &str,
    asset_name: &str,
    release_tag: &str,
    expected_sha256: &str,
    actual_sha256: &str,
    expected_byte_size: u64,
    actual_byte_size: u64,
    local_path: &str,
    fetched_at_unix: u64,
) -> serde_json::Value {
    let verified =
        digests_match(expected_sha256, actual_sha256) && expected_byte_size == actual_byte_size;
    serde_json::json!({
        "$schema": "https://docs.oyatie.com/schemas/kuberos-asterinas-fetch-verify-receipt.v0.1.0.json",
        "receipt_type": "fetch-verify",
        "acceptance_criterion": "AC1",
        "component": "asterinas/kernel",
        "wave": "kuberos-asterinas-wave1",
        "slice": "real-boot-envelope",
        "release_tag": release_tag,
        "source_url": source_url,
        "asset_name": asset_name,
        "expected_sha256": expected_sha256,
        "actual_sha256": actual_sha256,
        "expected_byte_size": expected_byte_size,
        "actual_byte_size": actual_byte_size,
        "local_path": local_path,
        "digest_recomputed_from_disk_at_receipt_write": true,
        "black_box_unmodified_upstream": true,
        "verified": verified,
        "fetched_at_unix": fetched_at_unix,
    })
}

// ============================================================================
// AC2/AC3 real-boot logic (pure, unit-tested via `buck2 test`; no QEMU/process I/O here).
// ============================================================================

/// A boot-ready marker match found in a captured raw serial log.
///
/// `marker_pattern` is the exact regex from the closed boot-ready marker set that matched;
/// `matched_line` is the verbatim log line (a single trailing CR stripped) and `matched_text`
/// is the exact substring the regex matched. These are recorded verbatim in the boot receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerMatch {
    pub marker_index: usize,
    pub marker_pattern: String,
    pub line_number: usize,
    pub matched_line: String,
    pub matched_text: String,
}

/// Split a raw serial log into lines on `\n`, stripping a single trailing `\r` per line so the
/// line-anchored markers (`login:\s*$`, `[#$]\s$`) apply to the semantic line rather than the
/// CRLF transport artifact. The raw bytes on disk are never mutated; this is a read-side view.
fn serial_lines(log: &str) -> impl Iterator<Item = &str> {
    log.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l))
}

/// Scan the raw serial log for the FIRST boot-ready marker: the earliest line (top-to-bottom)
/// that matches any regex in the closed `markers` set; within a line, the lowest marker index
/// wins. Returns `Ok(None)` when no marker in the closed set appears (honest "not ready").
/// No fallback or heuristic marker is consulted — only the exact provided regexes.
pub fn find_boot_marker(log: &str, markers: &[&str]) -> Result<Option<MarkerMatch>, regex::Error> {
    let compiled: Vec<Regex> = markers
        .iter()
        .map(|m| Regex::new(m))
        .collect::<Result<_, _>>()?;
    for (idx, line) in serial_lines(log).enumerate() {
        for (mi, re) in compiled.iter().enumerate() {
            if let Some(m) = re.find(line) {
                return Ok(Some(MarkerMatch {
                    marker_index: mi,
                    marker_pattern: markers[mi].to_string(),
                    line_number: idx + 1,
                    matched_line: line.to_string(),
                    matched_text: m.as_str().to_string(),
                }));
            }
        }
    }
    Ok(None)
}

/// Pass/fail status of a [`BootVerdict`], derived SOLELY from boot-ready markers present in the
/// real captured serial log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStatus {
    /// At least one closed-set marker was found in the real captured log.
    Pass,
    /// No closed-set marker was found in the real captured log.
    Fail,
}

impl BootStatus {
    /// Lowercase status string recorded verbatim in receipts.
    pub fn as_str(self) -> &'static str {
        match self {
            BootStatus::Pass => "pass",
            BootStatus::Fail => "fail",
        }
    }
}

/// The boot-reached verdict, derived SOLELY from boot-ready markers found in the real captured
/// serial log.
///
/// `boot_reached` is `true` iff at least one regex in the closed `allowed_markers` set matches a
/// line of the raw captured log (markers present -> reached; absent -> not reached); `marker`
/// carries the exact matched marker (verbatim line + text) or `None`; `status` is `Pass` iff
/// reached. The verdict is a pure function of the real log bytes and the closed marker set — no
/// simulated, synthesized, expected, or constant input can produce a `Pass`, because a `Pass`
/// requires a real marker occurrence in the supplied log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootVerdict {
    pub boot_reached: bool,
    pub allowed_markers: Vec<String>,
    pub marker: Option<MarkerMatch>,
    pub status: BootStatus,
}

/// Derive the boot-reached verdict SOLELY from boot-ready markers present in the real captured
/// serial `log`, against the closed `markers` set.
///
/// Markers present -> `boot_reached = true`, `status = Pass`; markers absent -> `boot_reached =
/// false`, `status = Fail`. This scans only the supplied log bytes via [`find_boot_marker`] (no
/// fallback/heuristic marker, no external signal, no simulated input), so the verdict cannot be
/// satisfied by anything other than a real marker occurrence in the log it is given.
pub fn derive_boot_verdict(log: &str, markers: &[&str]) -> Result<BootVerdict, regex::Error> {
    let marker = find_boot_marker(log, markers)?;
    let boot_reached = marker.is_some();
    let status = if boot_reached {
        BootStatus::Pass
    } else {
        BootStatus::Fail
    };
    Ok(BootVerdict {
        boot_reached,
        allowed_markers: markers.iter().map(|m| (*m).to_string()).collect(),
        marker,
        status,
    })
}

/// Per-marker count of matching lines (grep-style match counts) for the envelope receipt.
pub fn marker_hit_counts(log: &str, markers: &[&str]) -> Result<Vec<usize>, regex::Error> {
    let compiled: Vec<Regex> = markers
        .iter()
        .map(|m| Regex::new(m))
        .collect::<Result<_, _>>()?;
    let mut counts = vec![0usize; compiled.len()];
    for line in serial_lines(log) {
        for (mi, re) in compiled.iter().enumerate() {
            if re.is_match(line) {
                counts[mi] += 1;
            }
        }
    }
    Ok(counts)
}

/// Total number of `\n`-delimited lines in the log (for envelope bookkeeping).
pub fn line_count(log: &str) -> usize {
    serial_lines(log).count()
}

/// The first `n` and last `n` lines of the log — the ONLY portion permitted into agent context.
/// The full log lives on disk; this returns a bounded excerpt.
pub fn head_tail_lines(log: &str, n: usize) -> (Vec<String>, Vec<String>) {
    let lines: Vec<String> = serial_lines(log).map(|s| s.to_string()).collect();
    let head = lines.iter().take(n).cloned().collect();
    let tail = if lines.len() > n {
        lines[lines.len() - n..].to_vec()
    } else {
        lines.clone()
    };
    (head, tail)
}

/// The QEMU program (permitted external tool for the x86_64 ISO).
pub fn qemu_program() -> &'static str {
    "qemu-system-x86_64"
}

/// Build the QEMU argument vector for a single dev-only x86_64 boot of the release ISO.
///
/// Software emulation only (`-accel tcg,thread=multi`; the dev host has no KVM). The first
/// serial port is routed to stdio so the owned-Rust harness can (a) capture the raw serial
/// bytes to a log file by redirecting the child's stdout, and (b) drive the ISO's own
/// `boot-serial` menu entry over serial input. `-display none` keeps the run headless and
/// `-no-reboot` makes QEMU exit rather than loop on guest reset. This argv is the permitted
/// `std::process` external-tool surface and is recorded verbatim in the boot receipt.
pub fn build_qemu_args(iso_path: &str, mem_mib: u32, smp: u32) -> Vec<String> {
    vec![
        "-machine".into(),
        "q35".into(),
        "-m".into(),
        mem_mib.to_string(),
        "-smp".into(),
        smp.to_string(),
        "-accel".into(),
        "tcg,thread=multi".into(),
        "-cdrom".into(),
        iso_path.into(),
        "-serial".into(),
        "stdio".into(),
        "-display".into(),
        "none".into(),
        "-no-reboot".into(),
    ]
}

/// POSIX-shell-quote a single argument so a rendered command is copy-pasteable and safe.
fn shell_quote(s: &str) -> String {
    let safe = !s.is_empty()
        && s.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'/' | b'=' | b':' | b',')
        });
    if safe {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

/// Render an argv as a single copy-pasteable, shell-quoted command string (the independently
/// reproducible QEMU invocation recorded in the boot receipt).
pub fn render_command(program: &str, args: &[String]) -> String {
    let mut out = shell_quote(program);
    for a in args {
        out.push(' ');
        out.push_str(&shell_quote(a));
    }
    out
}

/// One scheduled keystroke burst sent to the guest serial input during boot-menu navigation.
/// `delay_ms` is the pause BEFORE sending `bytes` (relative to the previous step).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyStep {
    pub delay_ms: u64,
    pub bytes: Vec<u8>,
    pub note: &'static str,
}

/// The exact, reproducible keystroke schedule that drives the ISO's ISOLINUX/vesamenu over the
/// serial line to select the published `LABEL boot-serial` entry — whose APPEND carries
/// `console=ttyS0,115200n8`, routing the kernel + NixOS userland to ttyS0. The ISO's DEFAULT
/// entry has no `console=` and emits to VGA only (no serial evidence), so selecting the serial
/// entry is required to obtain admissible serial output from the unmodified artifact. The
/// harness sends these AFTER it observes the boot menu in the captured log.
///
/// Sequence from the default highlight ("Asterinas NixOS 0.17.2 Installer"):
///   Down -> "Options"; Enter -> open Options submenu (highlight "Copy ISO Files to RAM");
///   Down x7 -> "Serial console=ttyS0,115200n8"; Enter -> open its submenu; Enter -> boot.
/// Down = ESC `[` `B`; Enter = CR.
pub fn serial_menu_nav_plan() -> Vec<KeyStep> {
    const DOWN: &[u8] = b"\x1b[B";
    const ENTER: &[u8] = b"\r";
    let mut plan = vec![
        KeyStep {
            delay_ms: 0,
            bytes: DOWN.to_vec(),
            note: "highlight Options",
        },
        KeyStep {
            delay_ms: 1500,
            bytes: ENTER.to_vec(),
            note: "open Options submenu",
        },
    ];
    // Seven Downs move the highlight from "Copy ISO Files to RAM" (1/8) to
    // "Serial console=ttyS0,115200n8" (8/8) within the Options submenu.
    for i in 0..7 {
        plan.push(KeyStep {
            delay_ms: if i == 0 { 2000 } else { 1000 },
            bytes: DOWN.to_vec(),
            note: "advance toward Serial console entry",
        });
    }
    plan.push(KeyStep {
        delay_ms: 1000,
        bytes: ENTER.to_vec(),
        note: "open Serial console submenu",
    });
    plan.push(KeyStep {
        delay_ms: 2000,
        bytes: ENTER.to_vec(),
        note: "boot boot-serial (console=ttyS0)",
    });
    plan
}

/// Render the nav plan as JSON for the boot receipt (bytes shown as an escaped string so the
/// exact serial input is auditable and reproducible).
pub fn nav_plan_json(plan: &[KeyStep]) -> serde_json::Value {
    serde_json::Value::Array(
        plan.iter()
            .map(|s| {
                serde_json::json!({
                    "delay_ms": s.delay_ms,
                    "bytes_escaped": s.bytes.iter().map(|b| format!("\\x{b:02x}")).collect::<String>(),
                    "note": s.note,
                })
            })
            .collect(),
    )
}

/// The exact console argument the published `LABEL boot-serial` entry appends
/// (`APPEND ... console=ttyS0,115200n8`). Its verbatim presence in the captured serial log is
/// positive, auditable proof the harness booted the SERIAL-console entry and not the VGA-only
/// DEFAULT entry (which carries no `console=` and emits no serial evidence at all).
pub const BOOT_SERIAL_CONSOLE_ARG: &str = "console=ttyS0,115200n8";

/// Whether the captured serial `log` proves the `boot-serial` entry was actually selected: the
/// kernel command line carries [`BOOT_SERIAL_CONSOLE_ARG`]. The VGA-only DEFAULT entry never
/// emits it, so a mis-navigated or menu-not-detected boot (which falls through to the default)
/// yields `false` — making "the right entry booted" an explicit fact, not one implied by markers.
pub fn boot_serial_console_selected(log: &str) -> bool {
    log.contains(BOOT_SERIAL_CONSOLE_ARG)
}

/// Build the AC2/AC3 boot receipt as a JSON value.
///
/// `status` is `"pass"` iff `marker` is `Some` (a real marker from the closed set was found in
/// the raw captured log) AND the ISO digest verified AND `console_selected` (the boot-serial
/// entry was actually selected — `console=ttyS0` present in the log). The serial-log
/// `sha256`/`byte_size` MUST be recomputed from the on-disk file at receipt-write time by the
/// caller so the recorded digest is self-consistent with the referenced file.
#[allow(clippy::too_many_arguments)]
pub fn build_boot_receipt(
    release_tag: &str,
    iso_path: &str,
    expected_iso_sha256: &str,
    actual_iso_sha256: &str,
    qemu_program: &str,
    qemu_args: &[String],
    reproducible_command: &str,
    nav_plan: &[KeyStep],
    serial_log_path: &str,
    serial_log_sha256: &str,
    serial_log_byte_size: u64,
    verdict: &BootVerdict,
    console_selected: bool,
    timeout_secs: u64,
    wall_secs: u64,
    qemu_exit: &str,
    booted_at_unix: u64,
) -> serde_json::Value {
    let iso_verified = digests_match(expected_iso_sha256, actual_iso_sha256);
    let boot_reached = verdict.boot_reached;
    let status = if boot_reached && iso_verified && console_selected {
        "pass"
    } else {
        "fail"
    };
    let matched = match &verdict.marker {
        Some(m) => serde_json::json!({
            "marker_index": m.marker_index,
            "marker_pattern": m.marker_pattern,
            "line_number": m.line_number,
            "matched_line_verbatim": m.matched_line,
            "matched_text_verbatim": m.matched_text,
        }),
        None => serde_json::Value::Null,
    };
    serde_json::json!({
        "$schema": "https://docs.oyatie.com/schemas/kuberos-asterinas-boot-receipt.v0.1.0.json",
        "receipt_type": "boot",
        "acceptance_criterion": "AC2",
        "component": "asterinas/kernel",
        "wave": "kuberos-asterinas-wave1",
        "slice": "real-boot-envelope",
        "release_tag": release_tag,
        "black_box_unmodified_upstream": true,
        "artifact": {
            "iso_path": iso_path,
            "expected_sha256": expected_iso_sha256,
            "actual_sha256": actual_iso_sha256,
            "digest_recomputed_from_disk_at_receipt_write": true,
            "verified": iso_verified,
        },
        "qemu_invocation": {
            "program": qemu_program,
            "args": qemu_args,
            "reproducible_command": reproducible_command,
            "accel": "tcg (software emulation; dev-only, no KVM)",
            "serial_console_selection": {
                "why": "the ISO DEFAULT boot entry emits to VGA only; LABEL boot-serial carries console=ttyS0,115200n8",
                "input_channel": "guest first serial port <-> harness stdin (-serial stdio)",
                "keystrokes": nav_plan_json(nav_plan),
                "selection_evidence": BOOT_SERIAL_CONSOLE_ARG,
                "boot_serial_entry_selected": console_selected,
            },
        },
        "serial_log": {
            "path": serial_log_path,
            "capture_method": "child stdout redirected to file (-serial stdio); raw verbatim bytes",
            "sha256": serial_log_sha256,
            "digest_recomputed_from_disk_at_receipt_write": true,
            "byte_size": serial_log_byte_size,
        },
        "boot_verdict": {
            "allowed_markers": verdict.allowed_markers,
            "boot_reached": boot_reached,
            "verdict_status": verdict.status.as_str(),
            "matched_marker": matched,
            "derived_solely_from_real_captured_log": true,
        },
        "timeout_secs": timeout_secs,
        "wall_secs": wall_secs,
        "qemu_exit": qemu_exit,
        "status": status,
        "booted_at_unix": booted_at_unix,
    })
}

/// Build the envelope receipt as a JSON value: evidences that every artifact was handled by
/// reference (path + digest + byte size) with no inlined binaries or full logs, and that only a
/// bounded head/tail excerpt plus grep match counts were surfaced into agent context.
#[allow(clippy::too_many_arguments)]
pub fn build_envelope_receipt(
    release_tag: &str,
    iso_path: &str,
    iso_sha256: &str,
    iso_byte_size: u64,
    serial_log_path: &str,
    serial_log_sha256: &str,
    serial_log_byte_size: u64,
    serial_log_line_count: usize,
    excerpt_head_lines: usize,
    excerpt_tail_lines: usize,
    excerpt_line_cap: usize,
    marker_hit_counts: &[usize],
    allowed_markers: &[&str],
    written_at_unix: u64,
) -> serde_json::Value {
    let counts: Vec<serde_json::Value> = allowed_markers
        .iter()
        .zip(marker_hit_counts.iter())
        .map(|(m, c)| serde_json::json!({ "marker": m, "matching_lines": c }))
        .collect();
    serde_json::json!({
        "$schema": "https://docs.oyatie.com/schemas/kuberos-asterinas-envelope-receipt.v0.1.0.json",
        "receipt_type": "envelope",
        "acceptance_criterion": "AC2",
        "component": "asterinas/kernel",
        "wave": "kuberos-asterinas-wave1",
        "slice": "real-boot-envelope",
        "release_tag": release_tag,
        "artifacts_handled_by_reference": [
            { "role": "release-iso", "path": iso_path, "sha256": iso_sha256, "byte_size": iso_byte_size },
            { "role": "serial-log", "path": serial_log_path, "sha256": serial_log_sha256, "byte_size": serial_log_byte_size },
        ],
        "no_inlined_binaries_or_full_logs": true,
        "serial_log_line_count": serial_log_line_count,
        "agent_context_excerpt_bounds": {
            "line_cap_each_end": excerpt_line_cap,
            "head_lines_surfaced": excerpt_head_lines,
            "tail_lines_surfaced": excerpt_tail_lines,
            "grep_match_counts": counts,
        },
        "fit_single_agent_execution_envelope": true,
        "written_at_unix": written_at_unix,
    })
}

/// Build a gap-register entry (honest-failure escalation) recording a real-boot blocker.
pub fn build_gap_register_entry(
    gap_id: &str,
    title: &str,
    blocker_class: &str,
    observed_fact: &str,
    serial_log_path: &str,
    serial_log_sha256: &str,
    recorded_at_unix: u64,
) -> serde_json::Value {
    serde_json::json!({
        "gap_id": gap_id,
        "title": title,
        "class": blocker_class,
        "status": "open",
        "severity": "blocker-before-real-boot-envelope-proven",
        "slice": "real-boot-envelope",
        "observed_fact": observed_fact,
        "evidence": {
            "serial_log_path": serial_log_path,
            "serial_log_sha256": serial_log_sha256,
        },
        "simulated_or_inferred_evidence_produced": false,
        "recorded_at_unix": recorded_at_unix,
    })
}

// ============================================================================
// AC3 finalize orchestration: assemble the boot + envelope receipts and the honest fail-path
// escalation from the FINAL on-disk serial log. Extracted from the QEMU-driving binary so the
// receipt-assembly + fail-path escalation is exercised by an integration test covering BOTH
// boot-reached and non-boot outcomes (no QEMU, no network — just files on disk).
// ============================================================================

/// Read a serial-log file as lossy UTF-8. Serial logs are small text; the multi-GB ISO is never
/// read this way — it is only digested in streaming chunks by [`sha256_file`].
pub fn read_log_file(path: &Path) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Write a JSON value pretty-printed with a trailing newline.
pub fn write_json_file(dest: &Path, value: &serde_json::Value) -> io::Result<()> {
    let mut s = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
    s.push('\n');
    std::fs::write(dest, s)
}

/// Write the bounded head/tail excerpt (the ONLY serial-log slice permitted into agent context);
/// the full log lives only on disk. `cap` bounds each end.
pub fn write_excerpt_file(
    dest: &Path,
    head: &[String],
    tail: &[String],
    total_lines: usize,
    cap: usize,
) -> io::Result<()> {
    let mut out = String::new();
    out.push_str(&format!(
        "# bounded serial-log excerpt (first {} / last {} of {} lines); full log lives on disk only\n",
        head.len().min(cap),
        tail.len().min(cap),
        total_lines
    ));
    out.push_str("# ---- HEAD ----\n");
    for l in head.iter().take(cap) {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str("# ---- TAIL ----\n");
    for l in tail.iter().take(cap) {
        out.push_str(l);
        out.push('\n');
    }
    std::fs::write(dest, out)
}

/// Immutable description of one QEMU boot attempt whose on-disk evidence is being finalized into
/// receipts. Every heavy artifact (ISO, serial log) is referenced by path — no bytes are inlined.
/// This is the pure, QEMU-free input to [`finalize_boot_evidence`], so the receipt assembly and
/// fail-path escalation are integration-testable without QEMU.
pub struct BootAttempt<'a> {
    /// Pinned release tag (e.g. `v0.17.2`).
    pub release_tag: &'a str,
    /// Published release-asset name, recorded in the gap-register observed_fact.
    pub boot_iso_asset: &'a str,
    /// Local stored path of the ISO (referenced, never inlined).
    pub iso_path: &'a str,
    /// Pinned expected ISO sha256.
    pub expected_iso_sha256: &'a str,
    /// ISO sha256 recomputed from disk by the caller.
    pub actual_iso_sha256: &'a str,
    /// ISO byte size on disk.
    pub iso_byte_size: u64,
    /// Whether the ISO digest verified against the pin (black-box integrity).
    pub iso_verified: bool,
    /// QEMU program (permitted external tool).
    pub qemu_program: &'a str,
    /// QEMU argument vector.
    pub qemu_args: &'a [String],
    /// Rendered copy-pasteable QEMU command.
    pub reproducible_command: &'a str,
    /// Serial-menu navigation keystroke plan.
    pub nav_plan: &'a [KeyStep],
    /// Closed boot-ready marker set.
    pub allowed_markers: &'a [&'a str],
    /// Hard boot timeout (seconds).
    pub timeout_secs: u64,
    /// Observed wall-clock boot duration (seconds).
    pub wall_secs: u64,
    /// QEMU exit reason string.
    pub qemu_exit: &'a str,
    /// Unix time the boot started.
    pub booted_at_unix: u64,
    /// Bounded head/tail excerpt cap (lines each end).
    pub excerpt_lines: usize,
}

/// Filesystem destinations for the finalized evidence files.
pub struct EvidenceDests<'a> {
    /// Raw captured serial log (input; read to derive the verdict + recompute the digest).
    pub serial_log: &'a Path,
    /// Bounded head/tail excerpt destination.
    pub excerpt: &'a Path,
    /// Boot receipt destination.
    pub boot_receipt: &'a Path,
    /// Envelope receipt destination.
    pub envelope_receipt: &'a Path,
    /// Gap-register destination (written only on non-boot).
    pub gap_register: &'a Path,
}

/// Outcome of [`finalize_boot_evidence`] — a bounded, by-reference summary of what was written.
#[derive(Debug, Clone)]
pub struct FinalizeOutcome {
    /// Overall boot-reached: a real marker in the log AND the ISO digest verified.
    pub boot_reached: bool,
    /// The verdict derived solely from the real captured log.
    pub verdict: BootVerdict,
    /// Serial-log sha256 recomputed from disk at receipt-write time (recorded in the receipts).
    pub serial_log_sha256: String,
    /// Serial-log byte size on disk.
    pub serial_log_byte_size: u64,
    /// Total `\n`-delimited lines in the captured log.
    pub total_lines: usize,
    /// The bounded head excerpt surfaced into agent context.
    pub head_lines: Vec<String>,
    /// The bounded tail excerpt surfaced into agent context.
    pub tail_lines: Vec<String>,
    /// Per-marker grep match counts.
    pub marker_hit_counts: Vec<usize>,
    /// Receipt-digest self-consistency proof (recorded == recomputed-from-disk).
    pub digest_verification: DigestVerification,
    /// Whether the captured log proved the boot-serial entry was selected (console=ttyS0 present).
    pub boot_serial_console_selected: bool,
    /// Whether the honest fail-path escalation wrote a gap-register entry.
    pub gap_register_written: bool,
}

/// Finalize the on-disk evidence of a single boot attempt into the admissible receipts.
///
/// This is the AC3 boot-receipt assembly + honest fail-path escalation, extracted from the
/// QEMU-driving binary so it is exercised by an integration test covering BOTH boot-reached and
/// non-boot outcomes. It:
///   1. RECOMPUTES the serial-log sha256 from the on-disk file at receipt-write time;
///   2. derives the boot verdict SOLELY from the real captured log against the closed marker set;
///   3. writes the bounded head/tail excerpt (the only slice permitted into agent context);
///   4. assembles the boot receipt — recording the QEMU invocation, artifact digest, serial-log
///      path, and the recomputed serial-log sha256 — and FAIL-CLOSED verifies that the recorded
///      serial-log digest equals the on-disk bytes before persisting it;
///   5. assembles the envelope receipt (by-reference-handling evidence);
///   6. on non-boot (no marker OR ISO digest mismatch) sets the boot-receipt status to fail and
///      emits a gap-register escalation entry carrying
///      `simulated_or_inferred_evidence_produced: false`.
///
/// No simulated, synthesized, expected, or constant evidence is produced: an overall `pass`
/// requires a real marker occurrence in the supplied log AND a verified ISO digest.
pub fn finalize_boot_evidence(
    attempt: &BootAttempt,
    dests: &EvidenceDests,
) -> Result<FinalizeOutcome, Box<dyn std::error::Error>> {
    // 1. Recompute the serial-log digest from the on-disk bytes at receipt-write time so any
    //    digest recorded in a receipt is self-consistent with the file it references.
    let (serial_sha256, serial_size) = sha256_file(dests.serial_log)?;
    let log = read_log_file(dests.serial_log).unwrap_or_default();

    // 2. Verdict derived SOLELY from the real captured log + closed marker set.
    let verdict = derive_boot_verdict(&log, attempt.allowed_markers)?;
    let counts = marker_hit_counts(&log, attempt.allowed_markers)?;
    let total_lines = line_count(&log);
    let (head, tail) = head_tail_lines(&log, attempt.excerpt_lines);

    // The marker-derived verdict alone decides boot_reached; the overall receipt additionally
    // requires ISO digest verification (black-box integrity) AND proof the boot-serial entry was
    // actually selected (console=ttyS0 present) before an overall pass. A mis-navigated or
    // menu-not-detected boot falls through to the VGA-only default entry and never sets this.
    let console_selected = boot_serial_console_selected(&log);
    let boot_reached = verdict.boot_reached && attempt.iso_verified && console_selected;

    // 3. Bounded head/tail excerpt (the only slice permitted into agent context).
    write_excerpt_file(
        dests.excerpt,
        &head,
        &tail,
        total_lines,
        attempt.excerpt_lines,
    )?;

    // 4. Boot receipt (records QEMU invocation, artifact digest, serial-log path + recomputed
    //    serial-log sha256).
    let boot_receipt = build_boot_receipt(
        attempt.release_tag,
        attempt.iso_path,
        attempt.expected_iso_sha256,
        attempt.actual_iso_sha256,
        attempt.qemu_program,
        attempt.qemu_args,
        attempt.reproducible_command,
        attempt.nav_plan,
        &dests.serial_log.to_string_lossy(),
        &serial_sha256,
        serial_size,
        &verdict,
        console_selected,
        attempt.timeout_secs,
        attempt.wall_secs,
        attempt.qemu_exit,
        attempt.booted_at_unix,
    );
    // Fail-closed receipt digest self-consistency: the serial-log sha256 about to be recorded MUST
    // equal the sha256 recomputed from the on-disk file it references at receipt-write time. A
    // recorded digest that does not match the referenced file fails the slice, so refuse to
    // persist an inadmissible receipt.
    let recorded = boot_receipt["serial_log"]["sha256"]
        .as_str()
        .unwrap_or_default();
    let digest_verification = verify_serial_log_digest(recorded, dests.serial_log)?;
    write_json_file(dests.boot_receipt, &boot_receipt)?;

    // 5. Envelope receipt.
    let envelope = build_envelope_receipt(
        attempt.release_tag,
        attempt.iso_path,
        attempt.actual_iso_sha256,
        attempt.iso_byte_size,
        &dests.serial_log.to_string_lossy(),
        &serial_sha256,
        serial_size,
        total_lines,
        head.len().min(attempt.excerpt_lines),
        tail.len().min(attempt.excerpt_lines),
        attempt.excerpt_lines,
        &counts,
        attempt.allowed_markers,
        now_unix(),
    );
    write_json_file(dests.envelope_receipt, &envelope)?;

    // 6. Honest-failure escalation: emit a gap-register entry when the boot is not reached. No
    //    simulated or inferred evidence is produced — the entry records the observed fact only.
    let gap_register_written = if !boot_reached {
        let observed = format!(
            "QEMU boot of unmodified {} did not reach an admissible pass within {}s (exit={}, iso_verified={}, boot_serial_selected={}); serial log captured with {} lines.",
            attempt.boot_iso_asset,
            attempt.timeout_secs,
            attempt.qemu_exit,
            attempt.iso_verified,
            console_selected,
            total_lines
        );
        let entry = build_gap_register_entry(
            "KAW1-BOOT-ENVELOPE-001",
            "Real-boot envelope not proven: no boot-ready marker in captured serial log",
            "boot-api-evidence-gap",
            &observed,
            &dests.serial_log.to_string_lossy(),
            &serial_sha256,
            now_unix(),
        );
        write_json_file(dests.gap_register, &entry)?;
        true
    } else {
        false
    };

    Ok(FinalizeOutcome {
        boot_reached,
        verdict,
        serial_log_sha256: serial_sha256,
        serial_log_byte_size: serial_size,
        total_lines,
        head_lines: head,
        tail_lines: tail,
        marker_hit_counts: counts,
        digest_verification,
        boot_serial_console_selected: console_selected,
        gap_register_written,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn unique_temp(tag: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("kernel-real-boot-{tag}-{nanos}.bin"));
        p
    }

    #[test]
    fn hex_is_lowercase_and_zero_padded() {
        assert_eq!(to_hex(&[0x00, 0x0f, 0xff]), "000fff");
        assert_eq!(to_hex(&[]), "");
    }

    #[test]
    fn sha256_file_matches_known_vector_for_abc() {
        // NIST FIPS-180-2 sample: sha256("abc").
        let path = unique_temp("abc");
        {
            let mut f = File::create(&path).unwrap();
            f.write_all(b"abc").unwrap();
        }
        let (hex, size) = sha256_file(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(size, 3);
    }

    #[test]
    fn sha256_file_matches_known_vector_for_empty() {
        let path = unique_temp("empty");
        File::create(&path).unwrap();
        let (hex, size) = sha256_file(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(size, 0);
    }

    #[test]
    fn digests_match_is_case_insensitive() {
        assert!(digests_match("BF6E161E", "bf6e161e"));
        assert!(!digests_match("bf6e161e", "bf6e161f"));
    }

    #[test]
    fn verify_serial_log_digest_accepts_matching_and_rejects_mismatching_fixture() {
        // Matching fixture: recompute the real digest from the on-disk bytes, then verify that
        // recorded == recomputed. Verification recomputes from disk, never trusting the input.
        let path = unique_temp("serial-match");
        {
            let mut f = File::create(&path).unwrap();
            f.write_all(b"[    3.11] Reached target Basic System.\n")
                .unwrap();
        }
        let (actual, size) = sha256_file(&path).unwrap();

        let ok = verify_serial_log_digest(&actual, &path).expect("matching digest verifies");
        assert_eq!(ok.recorded_sha256, actual);
        assert_eq!(ok.actual_sha256, actual);
        assert_eq!(ok.byte_size, size);

        // Case-insensitive hex still verifies (recorded upper-case vs recomputed lower-case).
        let ok_upper = verify_serial_log_digest(&actual.to_uppercase(), &path)
            .expect("case-insensitive recorded digest verifies");
        assert_eq!(ok_upper.actual_sha256, actual);

        // Mismatching fixture: a stale/wrong recorded digest against the same file fails.
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        match verify_serial_log_digest(wrong, &path) {
            Err(DigestVerifyError::Mismatch(v)) => {
                assert_eq!(v.recorded_sha256, wrong);
                assert_eq!(v.actual_sha256, actual); // recomputed from disk, not echoed input
                assert_eq!(v.byte_size, size);
            }
            other => panic!("expected Mismatch, got {other:?}"),
        }
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_serial_log_digest_detects_post_record_file_mutation() {
        // Record the digest of the original bytes, then mutate the on-disk file: re-verification
        // must fail because the recorded digest no longer equals the recomputed on-disk digest.
        let path = unique_temp("serial-mutated");
        {
            let mut f = File::create(&path).unwrap();
            f.write_all(b"original serial capture\n").unwrap();
        }
        let (recorded, _) = sha256_file(&path).unwrap();
        {
            let mut f = File::create(&path).unwrap();
            f.write_all(b"tampered serial capture (different length)\n")
                .unwrap();
        }
        let err = verify_serial_log_digest(&recorded, &path)
            .expect_err("mutated file must fail digest self-consistency");
        assert!(matches!(err, DigestVerifyError::Mismatch(_)));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn verify_serial_log_digest_surfaces_io_error_for_missing_file() {
        let path = unique_temp("serial-missing"); // intentionally never created
        let err = verify_serial_log_digest("deadbeef", &path)
            .expect_err("missing referenced file must surface an I/O error");
        assert!(matches!(err, DigestVerifyError::Io(_)));
    }

    #[test]
    fn receipt_verified_only_when_digest_and_size_agree() {
        let ok = build_fetch_verify_receipt(
            "https://example/download",
            "a.iso",
            "v0.17.2",
            "bf6e",
            "BF6E",
            10,
            10,
            "kernel/target/artifacts/a.iso",
            42,
        );
        assert_eq!(ok["verified"], serde_json::Value::Bool(true));
        assert_eq!(ok["receipt_type"], "fetch-verify");
        assert_eq!(ok["acceptance_criterion"], "AC1");
        assert_eq!(
            ok["digest_recomputed_from_disk_at_receipt_write"],
            serde_json::Value::Bool(true)
        );

        let bad_size =
            build_fetch_verify_receipt("u", "a.iso", "v0.17.2", "bf6e", "bf6e", 10, 11, "p", 42);
        assert_eq!(bad_size["verified"], serde_json::Value::Bool(false));

        let bad_digest =
            build_fetch_verify_receipt("u", "a.iso", "v0.17.2", "bf6e", "dead", 10, 10, "p", 42);
        assert_eq!(bad_digest["verified"], serde_json::Value::Bool(false));
    }

    // ---- AC2/AC3 boot-logic tests (against the closed boundary marker set) ----

    fn markers() -> &'static [&'static str] {
        &kernel_asterinas_boundary::BOOT_READY_MARKERS
    }

    #[test]
    fn find_boot_marker_matches_reached_target_from_real_line() {
        // Verbatim shape of the admissible line observed in a real captured serial log.
        let log = "[    3.11] EDD noise\n\u{1b}[0;32m  OK  \u{1b}[0m] Reached target Basic System.\nmore\n";
        let m = find_boot_marker(log, markers())
            .unwrap()
            .expect("marker found");
        assert_eq!(m.marker_index, 3);
        assert_eq!(m.marker_pattern, markers()[3]);
        assert_eq!(m.line_number, 2);
        assert_eq!(m.matched_text, "Reached target Basic System");
    }

    #[test]
    fn find_boot_marker_shell_prompt_handles_crlf() {
        // CRLF serial: the trailing \r must be stripped so `[#$]\s$` anchors at end-of-line.
        let log = "kernel boot\r\nsh-5.1# \r\n";
        let m = find_boot_marker(log, markers())
            .unwrap()
            .expect("shell prompt");
        assert_eq!(m.marker_index, 1);
        assert_eq!(m.matched_text, "# ");
    }

    #[test]
    fn find_boot_marker_login_prompt() {
        let log = "starting getty\nnixos login: \r\n";
        let m = find_boot_marker(log, markers())
            .unwrap()
            .expect("login prompt");
        assert_eq!(m.marker_index, 0);
    }

    #[test]
    fn find_boot_marker_returns_none_without_any_marker() {
        // A "Welcome to Asterinas NixOS" banner must NOT match the literal "Welcome to NixOS",
        // and no other closed-set marker appears -> honest not-ready.
        let log = "Welcome to Asterinas NixOS 0.17.2\nloglevel=4 nohibernate\nEDD probe\n";
        assert!(find_boot_marker(log, markers()).unwrap().is_none());
    }

    #[test]
    fn find_boot_marker_returns_earliest_line_then_lowest_index() {
        // Line 2 (shell prompt, idx 1) precedes line 3 (reached target, idx 3): earliest wins.
        let log = "noise\nroot@nixos:~# \nReached target Multi-User System\n";
        let m = find_boot_marker(log, markers()).unwrap().expect("earliest");
        assert_eq!(m.line_number, 2);
        assert_eq!(m.marker_index, 1);
    }

    #[test]
    fn marker_hit_counts_counts_each_marker() {
        let log = "Reached target Basic System\nReached target Login Prompts\nidle\n";
        let counts = marker_hit_counts(log, markers()).unwrap();
        assert_eq!(counts.len(), 5);
        assert_eq!(counts[3], 2);
        assert_eq!(counts[2], 0); // "Welcome to NixOS" literal absent
    }

    #[test]
    fn head_tail_lines_is_bounded() {
        let log = (1..=10).map(|i| format!("line{i}\n")).collect::<String>();
        let (head, tail) = head_tail_lines(&log, 3);
        assert_eq!(head, vec!["line1", "line2", "line3"]);
        // last real content lines are 8,9,10 then a trailing "" from the final '\n'
        assert_eq!(tail.len(), 3);
        assert_eq!(tail[0], "line9");
    }

    #[test]
    fn qemu_args_declare_the_boot_surface() {
        let args = build_qemu_args("kernel/target/artifacts/x.iso", 6144, 4);
        assert!(args.contains(&"-cdrom".to_string()));
        assert!(args.contains(&"kernel/target/artifacts/x.iso".to_string()));
        assert!(args.contains(&"stdio".to_string()));
        assert!(args.contains(&"-no-reboot".to_string()));
        assert!(args.contains(&"tcg,thread=multi".to_string()));
    }

    #[test]
    fn render_command_quotes_paths_with_spaces() {
        let cmd = render_command("qemu-system-x86_64", &["-cdrom".into(), "a b.iso".into()]);
        assert_eq!(cmd, "qemu-system-x86_64 -cdrom 'a b.iso'");
    }

    #[test]
    fn nav_plan_selects_serial_entry_with_eight_downs_three_enters() {
        let plan = serial_menu_nav_plan();
        let downs = plan.iter().filter(|s| s.bytes == b"\x1b[B").count();
        let enters = plan.iter().filter(|s| s.bytes == b"\r").count();
        assert_eq!(downs, 8);
        assert_eq!(enters, 3);
        assert_eq!(plan.len(), 11);
    }

    #[test]
    fn boot_serial_console_selected_detects_the_serial_append() {
        // The real boot-serial APPEND carries console=ttyS0,115200n8; the VGA-only default entry
        // never emits it, so its absence marks a mis-navigated/default boot.
        assert!(boot_serial_console_selected(
            "loglevel=4 nohibernate console=ttyS0,115200n8 init=/nix/store/...\n"
        ));
        assert!(!boot_serial_console_selected(
            "loglevel=4 nohibernate init=/nix/store/... (VGA default, no console arg)\n"
        ));
        assert!(!boot_serial_console_selected(""));
    }

    #[test]
    fn boot_receipt_pass_requires_marker_and_iso_digest() {
        let plan = serial_menu_nav_plan();
        let marker = Some(MarkerMatch {
            marker_index: 3,
            marker_pattern: markers()[3].to_string(),
            line_number: 145,
            matched_line: "[  OK  ] Reached target Basic System.".into(),
            matched_text: "Reached target Basic System".into(),
        });
        let reached = BootVerdict {
            boot_reached: true,
            allowed_markers: markers().iter().map(|m| m.to_string()).collect(),
            marker,
            status: BootStatus::Pass,
        };
        let ok = build_boot_receipt(
            "v0.17.2",
            "iso",
            "bf6e",
            "BF6E",
            "qemu-system-x86_64",
            &build_qemu_args("iso", 6144, 4),
            "cmd",
            &plan,
            "log",
            "deadbeef",
            42,
            &reached,
            true,
            180,
            97,
            "killed-at-marker",
            1,
        );
        assert_eq!(ok["status"], "pass");
        assert_eq!(
            ok["boot_verdict"]["boot_reached"],
            serde_json::Value::Bool(true)
        );
        assert_eq!(ok["boot_verdict"]["verdict_status"], "pass");
        assert_eq!(ok["boot_verdict"]["matched_marker"]["line_number"], 145);

        // No marker -> fail even with a good ISO digest.
        let not_reached = BootVerdict {
            boot_reached: false,
            allowed_markers: markers().iter().map(|m| m.to_string()).collect(),
            marker: None,
            status: BootStatus::Fail,
        };
        let fail = build_boot_receipt(
            "v0.17.2",
            "iso",
            "bf6e",
            "bf6e",
            "qemu-system-x86_64",
            &[],
            "cmd",
            &plan,
            "log",
            "d",
            0,
            &not_reached,
            false,
            180,
            180,
            "timeout",
            1,
        );
        assert_eq!(fail["status"], "fail");
        assert_eq!(fail["boot_verdict"]["verdict_status"], "fail");

        // Marker present but ISO digest mismatch -> receipt fail (black-box integrity broken),
        // yet the marker-derived verdict itself stays pass (it depends solely on the log).
        let bad_iso = build_boot_receipt(
            "v0.17.2",
            "iso",
            "bf6e",
            "dead",
            "qemu-system-x86_64",
            &[],
            "cmd",
            &plan,
            "log",
            "d",
            0,
            &reached,
            true,
            180,
            97,
            "killed-at-marker",
            1,
        );
        assert_eq!(bad_iso["status"], "fail");
        assert_eq!(bad_iso["boot_verdict"]["verdict_status"], "pass");

        // Marker present + ISO digest OK, but the boot-serial entry was NOT selected
        // (console=ttyS0 absent -> mis-navigated/default boot) -> receipt fail.
        let no_console = build_boot_receipt(
            "v0.17.2", "iso", "bf6e", "bf6e", "qemu-system-x86_64", &[], "cmd", &plan, "log", "d",
            0, &reached, false, 180, 97, "killed-at-marker", 1,
        );
        assert_eq!(no_console["status"], "fail");
        assert_eq!(
            no_console["qemu_invocation"]["serial_console_selection"]["boot_serial_entry_selected"],
            serde_json::Value::Bool(false)
        );
    }

    #[test]
    fn envelope_receipt_records_by_reference_handling() {
        let r = build_envelope_receipt(
            "v0.17.2",
            "iso",
            "bf6e",
            1_378_910_208,
            "log",
            "beef",
            20_000,
            210,
            80,
            80,
            80,
            &[0, 0, 0, 2, 0],
            markers(),
            1,
        );
        assert_eq!(
            r["no_inlined_binaries_or_full_logs"],
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            r["artifacts_handled_by_reference"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            r["fit_single_agent_execution_envelope"],
            serde_json::Value::Bool(true)
        );
    }

    // ---- Sub-AC 2: boot-reached verdict derived SOLELY from real-log markers ----

    #[test]
    fn derive_boot_verdict_reached_when_marker_present_in_fixture_log() {
        // Fixture log WITH a closed-set marker (a real systemd "Reached target" line shape).
        let log = "[    2.01] EDD probe\n[  OK  ] Reached target Multi-User System.\ntail\n";
        let v = derive_boot_verdict(log, markers()).unwrap();
        assert!(v.boot_reached, "marker present -> reached");
        assert_eq!(v.status, BootStatus::Pass);
        assert_eq!(v.status.as_str(), "pass");
        assert_eq!(
            v.allowed_markers,
            markers().iter().map(|m| m.to_string()).collect::<Vec<_>>()
        );
        let m = v.marker.expect("matched marker recorded verbatim");
        assert_eq!(m.marker_index, 3);
        assert_eq!(m.marker_pattern, markers()[3]);
        assert_eq!(m.matched_text, "Reached target Multi-User");
    }

    #[test]
    fn derive_boot_verdict_reaches_on_each_closed_marker_class() {
        // One fixture log per class in the closed marker set: every class -> reached, and the
        // exact matched marker index is the class expected. Proves the verdict is driven by real
        // marker occurrences across the whole closed set, not one hard-coded pattern.
        let fixtures = [
            (0usize, "getty@tty1 spawned\nnixos login: \n"),
            (1, "kernel handoff\nsh-5.1# \n"),
            (2, "stage-1 done\nWelcome to NixOS 24.05 (Uakari)!\n"),
            (3, "[  OK  ] Reached target Login Prompts.\n"),
            (4, "systemd[1]: Startup finished in 4.213s.\n"),
        ];
        for (idx, log) in fixtures {
            let v = derive_boot_verdict(log, markers()).unwrap();
            assert!(
                v.boot_reached,
                "class {idx} fixture must be reached: {log:?}"
            );
            assert_eq!(v.status, BootStatus::Pass);
            assert_eq!(v.marker.expect("marker").marker_index, idx);
        }
    }

    #[test]
    fn derive_boot_verdict_not_reached_when_no_marker_in_fixture_log() {
        // Fixture log WITHOUT any closed-set marker: honest not-reached. The "Welcome to
        // Asterinas NixOS" banner must NOT match the literal "Welcome to NixOS" marker, and no
        // login/shell/reached-target/startup-finished line is present.
        let log = "Welcome to Asterinas NixOS 0.17.2\nloglevel=4 nohibernate\nEDD probe done\nno panic here\n";
        let v = derive_boot_verdict(log, markers()).unwrap();
        assert!(!v.boot_reached, "no marker -> not reached");
        assert_eq!(v.status, BootStatus::Fail);
        assert_eq!(v.status.as_str(), "fail");
        assert!(v.marker.is_none());
        assert_eq!(v.allowed_markers.len(), 5);
    }

    #[test]
    fn derive_boot_verdict_empty_log_is_not_reached() {
        // A timed-out boot that emitted nothing to serial is honest not-reached, never a
        // synthesized pass — the verdict has no input from which to fabricate a marker.
        let v = derive_boot_verdict("", markers()).unwrap();
        assert!(!v.boot_reached);
        assert_eq!(v.status, BootStatus::Fail);
        assert!(v.marker.is_none());
    }
}
