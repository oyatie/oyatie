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

/// Build the AC2/AC3 boot receipt as a JSON value.
///
/// `status` is `"pass"` iff `marker` is `Some` (a real marker from the closed set was found in
/// the raw captured log) AND the ISO digest verified. The serial-log `sha256`/`byte_size` MUST
/// be recomputed from the on-disk file at receipt-write time by the caller so the recorded
/// digest is self-consistent with the referenced file.
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
    timeout_secs: u64,
    wall_secs: u64,
    qemu_exit: &str,
    booted_at_unix: u64,
) -> serde_json::Value {
    let iso_verified = digests_match(expected_iso_sha256, actual_iso_sha256);
    let boot_reached = verdict.boot_reached;
    let status = if boot_reached && iso_verified {
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
    // requires ISO digest verification (black-box integrity) before an overall pass. NOTE: the
    // marker itself IS the boot-serial-selection proof — the VGA-only DEFAULT entry emits nothing
    // to serial, so a closed-set marker in the captured serial log is impossible unless the
    // console=ttyS0 `boot-serial` entry actually booted. A separate substring check for
    // "console=ttyS0" is NOT a valid signal: the ISOLINUX menu item is labeled
    // "Serial console=ttyS0,115200n8" and is echoed to serial merely by opening the submenu.
    let boot_reached = verdict.boot_reached && attempt.iso_verified;

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
            "QEMU boot of unmodified {} did not reach any boot-ready marker within {}s (exit={}, iso_verified={}); serial log captured with {} lines.",
            attempt.boot_iso_asset,
            attempt.timeout_secs,
            attempt.qemu_exit,
            attempt.iso_verified,
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
            180,
            97,
            "killed-at-marker",
            1,
        );
        assert_eq!(bad_iso["status"], "fail");
        assert_eq!(bad_iso["boot_verdict"]["verdict_status"], "pass");
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

// ============================================================================
// SHARD-2 SOAK: prove 10 consecutive CLEAN QEMU cold boots of the unmodified upstream ISO.
//
// This module EXTENDS the single-boot harness above into a soak. The per-boot cleanliness
// derivation, the attempt/aggregate receipt assembly, and the honest-fail gap-register are PURE
// (no process/network I/O) so they are unit-tested via `buck2 test` WITHOUT QEMU — that testable
// purity is the anti-simulation property: a boot cannot be declared clean except from a real
// per-boot observation, and an attempt cannot pass except from ten ordered real clean boots.
//
// The QEMU spawn/poll is the ONLY impure part and lives in the `src/soak.rs` binary; it feeds
// this module a [`BootObservation`] gathered from a real fresh-VM boot. The
// [`run_soak_with_boot_runner`] orchestrator does receipt file I/O but takes the per-boot runner
// as a closure, so the whole soak loop (incl. 3-attempt retry + honest-fail) is exercised by an
// integration test against on-disk serial-log fixtures with NO QEMU and NO network.
// ============================================================================
pub mod soak {
    use super::{MarkerMatch, now_unix, sha256_file, write_json_file};
    use std::path::{Path, PathBuf};

    /// Iterations per soak attempt: 10 consecutive fresh-VM cold boots must all be clean.
    pub const ITERATION_COUNT: usize = 10;
    /// Whole-soak attempt budget: up to 3 attempts to absorb host flakiness. Passing boots are
    /// NEVER aggregated across attempts — PASS requires exactly ONE attempt of 10 ordered clean
    /// boots.
    pub const MAX_SOAK_ATTEMPTS: usize = 3;
    /// Hard per-boot timeout: QEMU is force-killed at this deadline and the captured log up to
    /// that point is the evidence (matches the single-boot harness `BOOT_TIMEOUT_SECS`).
    pub const PER_BOOT_TIMEOUT_SECS: u64 = 180;

    /// Canonical termination-reason strings recorded verbatim in every boot record. The clean
    /// path is `TERM_MARKER_KILLED` ONLY; every other reason is honestly unclean.
    pub const TERM_MARKER_KILLED: &str = "boot_ready_marker_matched_then_harness_terminated_qemu";
    /// The 180s deadline was hit and the harness force-killed QEMU (unclean).
    pub const TERM_TIMEOUT: &str = "timeout_180s_harness_terminated_qemu";
    /// QEMU self-exited BEFORE any boot-ready marker appeared (abnormal early exit; unclean).
    pub const TERM_QEMU_EXITED_BEFORE: &str = "qemu_exited_before_marker";
    /// QEMU self-exited AFTER a marker was present but before the harness force-killed it
    /// (unclean: the clean path force-kills at the marker, it does not let QEMU self-exit).
    pub const TERM_QEMU_EXITED_AFTER: &str = "boot_ready_marker_matched_qemu_exited_after_marker";
    /// A marker was seen LIVE and the harness force-killed QEMU, but the FINAL on-disk log
    /// re-derivation (subject to a flush/SIGKILL race) did NOT confirm it. Recorded honestly as
    /// its own unclean reason rather than mislabeled `TERM_MARKER_KILLED` — the receipt must never
    /// claim a marker match it cannot re-derive from the persisted bytes.
    pub const TERM_MARKER_SEEN_LIVE_BUT_ABSENT_ON_FINAL_LOG: &str =
        "boot_ready_marker_seen_live_but_absent_on_reparsed_final_log";

    /// How a single boot terminated, as observed by the impure QEMU driver. Fed to the pure
    /// [`termination_reason`] / [`assemble_boot_record`] derivation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LiveTermination {
        /// A closed-set marker was seen in the LIVE serial capture and the harness then
        /// force-killed QEMU — the only clean termination.
        MarkerKilled,
        /// The 180s per-boot deadline was hit and the harness force-killed QEMU.
        TimedOut,
        /// QEMU exited on its own (its `ExitStatus` carries the code/signal).
        QemuSelfExited,
    }

    /// Structured QEMU exit status recorded in the boot record. Pure data (no `std::process`
    /// types) so the boot-record assembly stays QEMU-free and unit-testable; the binary converts
    /// a real `std::process::ExitStatus` into this.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct QemuExitStatus {
        pub observed: bool,
        pub code: Option<i32>,
        pub signal: Option<i32>,
        pub description: String,
    }

    /// The raw observed facts of ONE real fresh-VM boot, gathered by the impure QEMU driver and
    /// handed to the pure [`assemble_boot_record`]. The serial log is referenced by path +
    /// recomputed digest — its bytes are NEVER carried here (large-artifact-by-reference).
    #[derive(Debug, Clone)]
    pub struct BootObservation {
        /// Owning attempt id (e.g. `attempt-001`).
        pub attempt_id: String,
        /// 1-based iteration index within the attempt (1..=10).
        pub iteration_index: usize,
        /// How QEMU terminated for this boot.
        pub live: LiveTermination,
        /// The boot-ready marker RE-DERIVED from the FINAL on-disk serial log (`None` if absent).
        pub marker: Option<MarkerMatch>,
        /// QEMU exit status (SIGKILL on a harness-killed boot; the real code on a self-exit).
        pub qemu_exit_status: QemuExitStatus,
        /// Observed wall-clock boot duration (seconds).
        pub elapsed_seconds: f64,
        /// Path of the per-iteration raw serial log (referenced, never inlined).
        pub serial_log_path: String,
        /// sha256 of the serial log RECOMPUTED from the on-disk bytes at record-build time.
        pub serial_log_sha256: String,
        /// Byte size of the serial log on disk.
        pub serial_log_byte_size: u64,
    }

    /// One boot's record. Every heavy artifact (the serial log) is referenced by path + digest;
    /// no log bytes are inlined. `clean` is derived SOLELY from a real observation via
    /// [`boot_is_clean`] — no simulated/expected/constant input can set it.
    #[derive(Debug, Clone)]
    pub struct BootRecord {
        pub attempt_id: String,
        pub iteration_index: usize,
        pub clean: bool,
        pub elapsed_seconds: f64,
        pub timeout_hit: bool,
        pub qemu_exit_status: QemuExitStatus,
        pub termination_reason: String,
        pub matched_marker: Option<MarkerMatch>,
        pub raw_serial_log_path: String,
        pub raw_serial_log_sha256: String,
        pub raw_serial_log_byte_size: u64,
    }

    /// One soak attempt's result: its ordered boot records plus the on-disk attempt receipt it
    /// was serialized to (referenced by path + digest). Passing boots live inside ONE `SoakAttempt`
    /// and are never merged across attempts.
    #[derive(Debug, Clone)]
    pub struct SoakAttempt {
        pub attempt_id: String,
        pub attempt_index: usize,
        pub verdict: String,
        pub clean_boots: usize,
        pub required_clean_boots: usize,
        pub started_at_unix: u64,
        pub completed_at_unix: u64,
        /// Path of the written attempt receipt (empty until the orchestrator persists it).
        pub receipt_path: String,
        /// sha256 of the written attempt receipt (empty until persisted).
        pub receipt_sha256: String,
        pub boot_records: Vec<BootRecord>,
    }

    /// The whole-soak aggregate: overall verdict, the single referenced passing attempt (on pass),
    /// every attempt (incl. failed ones) by path + digest, and the honest-fail gap-register on
    /// exhaustion. No large artifact is inlined.
    #[derive(Debug, Clone)]
    pub struct AggregateReceipt {
        pub overall_verdict: String,
        pub passing_attempt_id: Option<String>,
        pub iteration_count: usize,
        pub per_boot_timeout_secs: u64,
        pub max_soak_attempts: usize,
        pub attempts: Vec<SoakAttempt>,
        pub gap_register: Option<serde_json::Value>,
    }

    /// The verified black-box ISO, referenced by path + digest (fetched + verified ONCE, reused
    /// unmodified read-only across all boots).
    #[derive(Debug, Clone)]
    pub struct IsoArtifact {
        pub asset_name: String,
        pub download_url: String,
        pub local_path: String,
        pub expected_sha256: String,
        pub actual_sha256: String,
        pub byte_size: u64,
        pub verified: bool,
    }

    /// Immutable soak configuration (pure data; no process/env I/O). Built once by the binary
    /// from the compile-time pin + the recomputed ISO digest, then read by the orchestrator and
    /// receipt builders.
    #[derive(Debug, Clone)]
    pub struct SoakConfig {
        pub release_tag: String,
        pub iso: IsoArtifact,
        pub qemu_program: String,
        pub qemu_args: Vec<String>,
        pub reproducible_command: String,
        pub iteration_count: usize,
        pub max_attempts: usize,
        pub per_boot_timeout_secs: u64,
        pub allowed_markers: Vec<String>,
    }

    /// Where the soak run's receipts + per-boot logs are written (under `kernel/target/…`,
    /// gitignored, regenerable — never committed).
    #[derive(Debug, Clone)]
    pub struct SoakDests {
        pub run_dir: PathBuf,
    }

    /// A bounded, by-reference summary of a completed soak run returned to the binary for agent
    /// context (no inlined logs).
    #[derive(Debug, Clone)]
    pub struct SoakOutcome {
        pub verdict: String,
        pub passing_attempt_id: Option<String>,
        pub aggregate_receipt_path: PathBuf,
        pub aggregate_receipt_sha256: String,
        pub attempts: Vec<SoakAttempt>,
        pub gap_register_written: bool,
    }

    /// Map a [`LiveTermination`] (and whether the final on-disk log carried a marker) to the
    /// canonical termination-reason string. Pure.
    pub fn termination_reason(live: LiveTermination, marker_present: bool) -> &'static str {
        match live {
            LiveTermination::MarkerKilled if marker_present => TERM_MARKER_KILLED,
            LiveTermination::MarkerKilled => TERM_MARKER_SEEN_LIVE_BUT_ABSENT_ON_FINAL_LOG,
            LiveTermination::TimedOut => TERM_TIMEOUT,
            LiveTermination::QemuSelfExited if marker_present => TERM_QEMU_EXITED_AFTER,
            LiveTermination::QemuSelfExited => TERM_QEMU_EXITED_BEFORE,
        }
    }

    /// A boot is CLEAN iff a closed-set marker was found in its raw serial log AND the boot did
    /// not hit the 180s timeout AND QEMU did not exit abnormally before the match (i.e. the
    /// termination is the marker-then-killed path). All three conditions are checked explicitly,
    /// so a boot with `timeout_hit == true` is unclean even if a marker-like line is present.
    /// Pure — reads only an already-assembled real record.
    pub fn boot_is_clean(record: &BootRecord) -> bool {
        record.matched_marker.is_some()
            && !record.timeout_hit
            && record.termination_reason == TERM_MARKER_KILLED
    }

    /// Assemble a [`BootRecord`] from ONE real [`BootObservation`], deriving `timeout_hit`, the
    /// termination reason, and `clean` from the observation. Pure — never fabricates a marker or
    /// a clean verdict.
    pub fn assemble_boot_record(obs: BootObservation) -> BootRecord {
        let marker_present = obs.marker.is_some();
        let timeout_hit = matches!(obs.live, LiveTermination::TimedOut);
        let mut record = BootRecord {
            attempt_id: obs.attempt_id,
            iteration_index: obs.iteration_index,
            clean: false,
            elapsed_seconds: obs.elapsed_seconds,
            timeout_hit,
            qemu_exit_status: obs.qemu_exit_status,
            termination_reason: termination_reason(obs.live, marker_present).to_string(),
            matched_marker: obs.marker,
            raw_serial_log_path: obs.serial_log_path,
            raw_serial_log_sha256: obs.serial_log_sha256,
            raw_serial_log_byte_size: obs.serial_log_byte_size,
        };
        record.clean = boot_is_clean(&record);
        record
    }

    /// An attempt PASSES iff it holds exactly `iteration_count` boot records that are, in order,
    /// iterations 1..=N, ALL owned by the SAME attempt id, and ALL clean. This is why passing
    /// boots never aggregate across attempts: a record carrying a different `attempt_id` fails the
    /// same-owner check. Pure.
    pub fn attempt_is_pass(records: &[BootRecord], iteration_count: usize) -> bool {
        let Some(first) = records.first() else {
            return false;
        };
        records.len() == iteration_count
            && records.iter().enumerate().all(|(offset, r)| {
                r.iteration_index == offset + 1
                    && r.attempt_id == first.attempt_id
                    && r.clean
                    && boot_is_clean(r)
            })
    }

    fn qemu_exit_json(s: &QemuExitStatus) -> serde_json::Value {
        serde_json::json!({
            "observed": s.observed,
            "code": s.code,
            "signal": s.signal,
            "description": s.description,
        })
    }

    fn marker_json(m: &Option<MarkerMatch>) -> serde_json::Value {
        match m {
            Some(mm) => serde_json::json!({
                "marker_index": mm.marker_index,
                "marker_pattern": mm.marker_pattern,
                "line_number": mm.line_number,
                "matched_line_verbatim": mm.matched_line,
                "matched_text_verbatim": mm.matched_text,
            }),
            None => serde_json::Value::Null,
        }
    }

    /// Serialize one boot record as JSON — the serial log is referenced by path + recomputed
    /// digest + byte size, never inlined.
    pub fn build_boot_record_json(r: &BootRecord) -> serde_json::Value {
        serde_json::json!({
            "attempt_id": r.attempt_id,
            "iteration_index": r.iteration_index,
            "clean": r.clean,
            "elapsed_seconds": r.elapsed_seconds,
            "timeout_hit": r.timeout_hit,
            "qemu_exit_status": qemu_exit_json(&r.qemu_exit_status),
            "termination_reason": r.termination_reason,
            "matched_marker": marker_json(&r.matched_marker),
            "raw_serial_log": {
                "path": r.raw_serial_log_path,
                "sha256": r.raw_serial_log_sha256,
                "digest_recomputed_from_disk_at_record_build": true,
                "byte_size": r.raw_serial_log_byte_size,
            },
        })
    }

    fn iso_json(iso: &IsoArtifact) -> serde_json::Value {
        serde_json::json!({
            "asset_name": iso.asset_name,
            "download_url": iso.download_url,
            "local_path": iso.local_path,
            "expected_sha256": iso.expected_sha256,
            "actual_sha256": iso.actual_sha256,
            "byte_size": iso.byte_size,
            "digest_recomputed_from_disk_once": true,
            "black_box_unmodified_upstream": true,
            "verified": iso.verified,
        })
    }

    fn qemu_runtime_json(cfg: &SoakConfig) -> serde_json::Value {
        serde_json::json!({
            "program": cfg.qemu_program,
            "args": cfg.qemu_args,
            "reproducible_command": cfg.reproducible_command,
            "accel": "tcg (software emulation; dev-only, no KVM)",
            "per_boot_timeout_seconds": cfg.per_boot_timeout_secs,
        })
    }

    fn vm_isolation_json() -> serde_json::Value {
        serde_json::json!({
            "method": "each iteration spawns a NEW qemu-system-x86_64 process from the verified read-only ISO CD-ROM; no writable disk, no snapshot, a NEW per-iteration serial log file",
            "no_disk": true,
            "no_snapshot": true,
            "fresh_process_per_iteration": true,
            "serial_log_per_iteration": true,
            "cold_boot": true,
            "state_carryover": "none: ISO bytes reused read-only after ONE sha256 verification; VM process, memory, devices, and serial capture are recreated per iteration",
        })
    }

    /// Assemble the per-attempt receipt JSON: the attempt verdict + its 10 ordered boot records
    /// (each referencing its serial log by path + digest), plus the ISO/QEMU/isolation context.
    pub fn build_attempt_receipt(cfg: &SoakConfig, attempt: &SoakAttempt) -> serde_json::Value {
        serde_json::json!({
            "$schema": "https://docs.oyatie.com/schemas/kuberos-asterinas-soak-attempt-receipt.v0.1.0.json",
            "receipt_type": "soak-attempt",
            "acceptance_criterion": "SHARD-2-SOAK",
            "component": "asterinas/kernel",
            "wave": "kuberos-asterinas-wave1",
            "slice": "real-boot-soak",
            "release_tag": cfg.release_tag,
            "black_box_unmodified_upstream": true,
            "attempt_id": attempt.attempt_id,
            "attempt_index": attempt.attempt_index,
            "verdict": attempt.verdict,
            "clean_boots": attempt.clean_boots,
            "required_clean_boots": attempt.required_clean_boots,
            "started_at_unix": attempt.started_at_unix,
            "completed_at_unix": attempt.completed_at_unix,
            "iso_artifact": iso_json(&cfg.iso),
            "qemu_runtime": qemu_runtime_json(cfg),
            "vm_isolation": vm_isolation_json(),
            "boot_ready_markers": cfg.allowed_markers,
            "no_inlined_large_artifacts": true,
            "boot_records": attempt.boot_records.iter().map(build_boot_record_json).collect::<Vec<_>>(),
        })
    }

    /// Assemble the whole-soak aggregate receipt JSON: overall verdict, the referenced passing
    /// attempt id (on pass), every attempt by path + digest with its boot records, the ISO digest,
    /// the attempt count, and the honest-fail gap-register (on exhaustion). No large artifact is
    /// inlined.
    pub fn build_aggregate_receipt(cfg: &SoakConfig, agg: &AggregateReceipt) -> serde_json::Value {
        let attempts: Vec<serde_json::Value> = agg
            .attempts
            .iter()
            .map(|a| {
                serde_json::json!({
                    "attempt_id": a.attempt_id,
                    "attempt_index": a.attempt_index,
                    "verdict": a.verdict,
                    "clean_boots": a.clean_boots,
                    "required_clean_boots": a.required_clean_boots,
                    "attempt_receipt": {
                        "path": a.receipt_path,
                        "sha256": a.receipt_sha256,
                    },
                    "boot_records": a.boot_records.iter().map(build_boot_record_json).collect::<Vec<_>>(),
                })
            })
            .collect();
        serde_json::json!({
            "$schema": "https://docs.oyatie.com/schemas/kuberos-asterinas-soak-aggregate-receipt.v0.1.0.json",
            "receipt_type": "soak-aggregate",
            "acceptance_criterion": "SHARD-2-SOAK",
            "component": "asterinas/kernel",
            "wave": "kuberos-asterinas-wave1",
            "slice": "real-boot-soak",
            "release_tag": cfg.release_tag,
            "black_box_unmodified_upstream": true,
            "overall_verdict": agg.overall_verdict,
            "passing_attempt_id": agg.passing_attempt_id,
            "iso_artifact": iso_json(&cfg.iso),
            "iteration_count": agg.iteration_count,
            "per_boot_timeout_seconds": agg.per_boot_timeout_secs,
            "max_soak_attempts": agg.max_soak_attempts,
            "attempt_count": agg.attempts.len(),
            "qemu_runtime": qemu_runtime_json(cfg),
            "vm_isolation": vm_isolation_json(),
            "boot_ready_markers": cfg.allowed_markers,
            "no_inlined_large_artifacts": true,
            "soak_attempts": attempts,
            "gap_register_entry": agg.gap_register,
        })
    }

    /// Build the honest-fail gap-register entry, emitted ONLY after the attempt budget is
    /// exhausted without a clean 10/10 attempt. Records the observed blocker and carries
    /// `simulated_or_inferred_evidence_produced: false` — no evidence is ever synthesized.
    pub fn build_soak_gap_register_entry(
        cfg: &SoakConfig,
        run_dir: &Path,
        recorded_at_unix: u64,
    ) -> serde_json::Value {
        serde_json::json!({
            "gap_id": "KAW1-SOAK-001",
            "title": "10-consecutive-clean cold-boot soak not proven within the attempt budget",
            "class": "boot-soak-evidence-gap",
            "status": "open",
            "severity": "blocker-before-soak-envelope-proven",
            "slice": "real-boot-soak",
            "release_tag": cfg.release_tag,
            "blocker": format!(
                "No soak attempt reached {} consecutive clean isolated QEMU cold boots within {} attempts.",
                cfg.iteration_count, cfg.max_attempts
            ),
            "acceptance_criteria": format!(
                "Exactly one attempt with {} ordered clean fresh-VM cold boots (per-boot timeout {}s, no state carry-over).",
                cfg.iteration_count, cfg.per_boot_timeout_secs
            ),
            "honest_fail_reference": run_dir.to_string_lossy(),
            "verification_path": "Inspect aggregate-receipt.json and every attempt's per-boot raw serial log path + sha256.",
            "simulated_or_inferred_evidence_produced": false,
            "recorded_at_unix": recorded_at_unix,
        })
    }

    /// Run the whole soak, taking the per-boot QEMU execution as an injected `boot_runner`
    /// closure. This does receipt FILE I/O but NO QEMU and NO network of its own — the real binary
    /// passes a QEMU-driving runner; a test passes a fixture-driving runner. That seam makes the
    /// entire soak loop (3-attempt retry, per-attempt receipt, aggregate, honest-fail) testable
    /// without QEMU.
    ///
    /// Semantics (exact): for each of up to `cfg.max_attempts` attempts, run `cfg.iteration_count`
    /// ordered boots via `boot_runner`; a failed boot records its record and fails THAT attempt —
    /// it does NOT abort the whole soak (the loop then starts a fresh attempt). PASS = the FIRST
    /// attempt whose 10 ordered boots are all clean (passing boots are never aggregated across
    /// attempts). Every attempt (incl. failed) produces its own attempt receipt. On exhaustion
    /// the aggregate carries an honest-fail gap-register entry.
    pub fn run_soak_with_boot_runner<F>(
        cfg: &SoakConfig,
        dests: &SoakDests,
        mut boot_runner: F,
    ) -> Result<SoakOutcome, Box<dyn std::error::Error>>
    where
        F: FnMut(
            &SoakConfig,
            &str,
            usize,
            &Path,
        ) -> Result<BootRecord, Box<dyn std::error::Error>>,
    {
        std::fs::create_dir_all(&dests.run_dir)?;

        let mut attempts: Vec<SoakAttempt> = Vec::new();
        let mut passing_attempt_id: Option<String> = None;

        for attempt_index in 1..=cfg.max_attempts {
            let attempt_id = format!("attempt-{attempt_index:03}");
            let attempt_dir = dests.run_dir.join(&attempt_id);
            std::fs::create_dir_all(&attempt_dir)?;
            let started_at_unix = now_unix();

            let mut boot_records = Vec::with_capacity(cfg.iteration_count);
            for iteration in 1..=cfg.iteration_count {
                let record = boot_runner(cfg, &attempt_id, iteration, &attempt_dir)?;
                // Bind the returned record to the identity the orchestrator REQUESTED, rather than
                // trusting the runner's self-stamp: attempt_is_pass checks same-ownership against
                // records[0].attempt_id, so a runner that mis-stamped every record with one id would
                // otherwise yield an internally-consistent (and passing) attempt. Fail closed on any
                // mismatch (an infrastructure invariant violation, exit 1 — never a false pass).
                if record.attempt_id != attempt_id || record.iteration_index != iteration {
                    return Err(format!(
                        "boot runner returned mis-identified record: expected {attempt_id} iteration {iteration}, got {} iteration {}",
                        record.attempt_id, record.iteration_index
                    )
                    .into());
                }
                boot_records.push(record);
            }

            let clean_boots = boot_records.iter().filter(|r| r.clean).count();
            let verdict = if attempt_is_pass(&boot_records, cfg.iteration_count) {
                "pass"
            } else {
                "fail"
            }
            .to_string();

            let mut attempt = SoakAttempt {
                attempt_id: attempt_id.clone(),
                attempt_index,
                verdict: verdict.clone(),
                clean_boots,
                required_clean_boots: cfg.iteration_count,
                started_at_unix,
                completed_at_unix: now_unix(),
                receipt_path: String::new(),
                receipt_sha256: String::new(),
                boot_records,
            };
            // Persist the per-attempt receipt, then reference it by path + digest.
            let receipt_json = build_attempt_receipt(cfg, &attempt);
            let receipt_path = attempt_dir.join("attempt-receipt.json");
            write_json_file(&receipt_path, &receipt_json)?;
            let (receipt_sha256, _) = sha256_file(&receipt_path)?;
            attempt.receipt_path = receipt_path.to_string_lossy().into_owned();
            attempt.receipt_sha256 = receipt_sha256;

            let is_pass = verdict == "pass";
            attempts.push(attempt);
            if is_pass {
                passing_attempt_id = Some(attempt_id);
                break;
            }
        }

        let overall_verdict = if passing_attempt_id.is_some() {
            "pass"
        } else {
            "fail"
        }
        .to_string();
        // Honest-fail: only after the attempt budget is exhausted without a clean 10/10.
        let gap_register = if passing_attempt_id.is_none() {
            Some(build_soak_gap_register_entry(cfg, &dests.run_dir, now_unix()))
        } else {
            None
        };

        let aggregate = AggregateReceipt {
            overall_verdict: overall_verdict.clone(),
            passing_attempt_id: passing_attempt_id.clone(),
            iteration_count: cfg.iteration_count,
            per_boot_timeout_secs: cfg.per_boot_timeout_secs,
            max_soak_attempts: cfg.max_attempts,
            attempts: attempts.clone(),
            gap_register: gap_register.clone(),
        };
        let aggregate_json = build_aggregate_receipt(cfg, &aggregate);
        let aggregate_receipt_path = dests.run_dir.join("aggregate-receipt.json");
        write_json_file(&aggregate_receipt_path, &aggregate_json)?;
        let (aggregate_receipt_sha256, _) = sha256_file(&aggregate_receipt_path)?;

        Ok(SoakOutcome {
            verdict: overall_verdict,
            passing_attempt_id,
            aggregate_receipt_path,
            aggregate_receipt_sha256,
            attempts,
            gap_register_written: gap_register.is_some(),
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn exit_sigkill() -> QemuExitStatus {
            QemuExitStatus {
                observed: true,
                code: None,
                signal: Some(9),
                description: "signal: 9 (SIGKILL)".to_string(),
            }
        }

        fn a_marker() -> MarkerMatch {
            MarkerMatch {
                marker_index: 3,
                marker_pattern: "Reached target .*(Multi-User|Basic System|Login Prompts)"
                    .to_string(),
                line_number: 145,
                matched_line: "[  OK  ] Reached target Basic System.".to_string(),
                matched_text: "Reached target Basic System".to_string(),
            }
        }

        fn clean_record(attempt_id: &str, iteration_index: usize) -> BootRecord {
            assemble_boot_record(BootObservation {
                attempt_id: attempt_id.to_string(),
                iteration_index,
                live: LiveTermination::MarkerKilled,
                marker: Some(a_marker()),
                qemu_exit_status: exit_sigkill(),
                elapsed_seconds: 42.0,
                serial_log_path: format!("boot-{iteration_index:02}/serial.log"),
                serial_log_sha256: "0".repeat(64),
                serial_log_byte_size: 20_000,
            })
        }

        #[test]
        fn termination_reason_maps_every_live_case() {
            assert_eq!(
                termination_reason(LiveTermination::MarkerKilled, true),
                TERM_MARKER_KILLED
            );
            // Flush/SIGKILL race: marker seen LIVE but absent from the re-derived final log.
            // Must NOT be mislabeled a marker match — the honest reason keeps the receipt
            // consistent with `matched_marker: null` / `clean: false`.
            assert_eq!(
                termination_reason(LiveTermination::MarkerKilled, false),
                TERM_MARKER_SEEN_LIVE_BUT_ABSENT_ON_FINAL_LOG
            );
            assert_eq!(
                termination_reason(LiveTermination::TimedOut, false),
                TERM_TIMEOUT
            );
            assert_eq!(
                termination_reason(LiveTermination::QemuSelfExited, true),
                TERM_QEMU_EXITED_AFTER
            );
            assert_eq!(
                termination_reason(LiveTermination::QemuSelfExited, false),
                TERM_QEMU_EXITED_BEFORE
            );
        }

        #[test]
        fn clean_requires_marker_no_timeout_and_marker_killed_termination() {
            let record = clean_record("attempt-001", 1);
            assert!(record.clean);
            assert!(boot_is_clean(&record));

            // A marker present but `timeout_hit == true` is UNCLEAN even though a marker-like line
            // exists — the timeout condition is checked explicitly.
            let mut timed_out = clean_record("attempt-001", 1);
            timed_out.timeout_hit = true;
            timed_out.termination_reason = TERM_TIMEOUT.to_string();
            assert!(!boot_is_clean(&timed_out));

            // A marker but QEMU self-exited before it (abnormal early exit) is UNCLEAN.
            let mut early_exit = clean_record("attempt-001", 1);
            early_exit.termination_reason = TERM_QEMU_EXITED_BEFORE.to_string();
            assert!(!boot_is_clean(&early_exit));

            // No marker is UNCLEAN.
            let mut no_marker = clean_record("attempt-001", 1);
            no_marker.matched_marker = None;
            assert!(!boot_is_clean(&no_marker));
        }

        #[test]
        fn timed_out_observation_assembles_unclean_record() {
            // A boot that hit the deadline: harness-killed, no marker → unclean, timeout_hit true.
            let record = assemble_boot_record(BootObservation {
                attempt_id: "attempt-001".to_string(),
                iteration_index: 4,
                live: LiveTermination::TimedOut,
                marker: None,
                qemu_exit_status: exit_sigkill(),
                elapsed_seconds: 180.0,
                serial_log_path: "boot-04/serial.log".to_string(),
                serial_log_sha256: "0".repeat(64),
                serial_log_byte_size: 0,
            });
            assert!(!record.clean);
            assert!(record.timeout_hit);
            assert_eq!(record.termination_reason, TERM_TIMEOUT);
            assert!(record.matched_marker.is_none());
        }

        #[test]
        fn attempt_pass_requires_ten_ordered_clean_boots_in_one_attempt() {
            let records: Vec<_> = (1..=ITERATION_COUNT)
                .map(|i| clean_record("attempt-001", i))
                .collect();
            assert!(attempt_is_pass(&records, ITERATION_COUNT));

            // Nine clean boots is not a pass.
            assert!(!attempt_is_pass(&records[..ITERATION_COUNT - 1], ITERATION_COUNT));

            // Passing boots NEVER aggregate across attempts: relabel the last boot's attempt id.
            let mut split = records.clone();
            split[9].attempt_id = "attempt-002".to_string();
            assert!(!attempt_is_pass(&split, ITERATION_COUNT));
            // …and the nine boots that remain owned by attempt-001 still are not a pass.
            let attempt_one_only: Vec<_> = split
                .iter()
                .filter(|r| r.attempt_id == "attempt-001")
                .cloned()
                .collect();
            assert!(!attempt_is_pass(&attempt_one_only, ITERATION_COUNT));

            // Out-of-order iterations fail the ordered check.
            let mut out_of_order = records.clone();
            out_of_order.swap(8, 9);
            assert!(!attempt_is_pass(&out_of_order, ITERATION_COUNT));

            // One unclean boot fails the whole attempt.
            let mut one_unclean = records.clone();
            one_unclean[0].clean = false;
            assert!(!attempt_is_pass(&one_unclean, ITERATION_COUNT));
        }

        #[test]
        fn boot_record_json_references_log_by_path_and_digest_without_inlining() {
            let r = clean_record("attempt-001", 1);
            let v = build_boot_record_json(&r);
            assert_eq!(v["clean"], serde_json::Value::Bool(true));
            assert_eq!(v["termination_reason"], TERM_MARKER_KILLED);
            assert_eq!(v["raw_serial_log"]["sha256"], "0".repeat(64));
            assert_eq!(
                v["raw_serial_log"]["digest_recomputed_from_disk_at_record_build"],
                serde_json::Value::Bool(true)
            );
            assert_eq!(v["matched_marker"]["matched_text_verbatim"], "Reached target Basic System");
        }
    }
}
