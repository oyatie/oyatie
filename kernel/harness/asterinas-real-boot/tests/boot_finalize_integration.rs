#![forbid(unsafe_code)]
//! AC3 integration test: exercise the boot-receipt assembly + honest fail-path escalation
//! (`finalize_boot_evidence`) end-to-end against on-disk serial-log fixtures, covering BOTH the
//! boot-reached and the non-boot outcomes.
//!
//! This drives the SAME orchestration the QEMU binary runs (the binary is a thin `std::process`
//! wrapper around this library function), but with no QEMU and no network — the "boot attempt" is
//! represented by a raw serial-log fixture already on disk. That isolates the receipt assembly and
//! fail-path escalation so both admissible and inadmissible outcomes are verified deterministically.
//!
//! What is proven here:
//!   * boot-reached: receipt status `pass`, matched marker recorded verbatim, the recorded
//!     serial-log sha256 equals an INDEPENDENTLY-computed digest of the on-disk fixture (receipt
//!     digest self-consistency), and NO gap-register entry is written.
//!   * non-boot: receipt status `fail`, no fabricated marker, a gap-register escalation entry is
//!     written carrying `simulated_or_inferred_evidence_produced: false`, and the recorded digests
//!     are still self-consistent with the on-disk fixture bytes.
//!   * empty log (timed-out boot that emitted nothing): still honest `fail` — a zero-byte log
//!     cannot synthesize a pass.

use kernel_asterinas_boundary as pin;
use kernel_asterinas_real_boot as harness;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A unique scratch directory for one test case (best-effort cleanup at end).
fn scratch(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut dir = std::env::temp_dir();
    dir.push(format!("kernel-real-boot-it-{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Independently recompute the lowercase-hex sha256 of an on-disk file (NOT via the harness's own
/// digest helper) so the "recorded == on-disk bytes" self-consistency check is a genuine
/// cross-check of the receipt against the file it references.
fn sha256_hex(path: &Path) -> String {
    let mut f = fs::File::open(path).expect("open fixture");
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f.read(&mut buf).expect("read fixture");
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let mut s = String::new();
    for b in hasher.finalize() {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Read + parse a receipt file as JSON.
fn read_json(path: &Path) -> serde_json::Value {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Build a `BootAttempt`/`EvidenceDests` pair for a fixture, run the finalize orchestration, and
/// return the outcome. `iso_ok` toggles whether the (fake) ISO digest verifies.
struct Fixture {
    dir: PathBuf,
    serial_log: PathBuf,
    excerpt: PathBuf,
    boot_receipt: PathBuf,
    envelope_receipt: PathBuf,
    gap_register: PathBuf,
}

impl Fixture {
    fn new(tag: &str, serial_contents: &[u8]) -> Fixture {
        let dir = scratch(tag);
        let serial_log = dir.join("boot-serial.log");
        fs::write(&serial_log, serial_contents).expect("write serial fixture");
        Fixture {
            excerpt: dir.join("boot-serial.excerpt.txt"),
            boot_receipt: dir.join("boot-receipt.json"),
            envelope_receipt: dir.join("envelope-receipt.json"),
            gap_register: dir.join("gap-register.json"),
            serial_log,
            dir,
        }
    }

    fn run(&self, iso_ok: bool) -> harness::FinalizeOutcome {
        // A fixed fake ISO sha; `iso_ok` decides whether expected==actual (verified) or not.
        let expected = "bf6e161ecc8b8080b842a339cee5f55d18b93d99b1e39c7c07681ff3aca0090a";
        let actual = if iso_ok {
            expected
        } else {
            "0000000000000000000000000000000000000000000000000000000000000000"
        };
        let iso_path = "kernel/target/artifacts/asterinas-nixos-0.17.2-x86_64.iso";
        let qargs = harness::build_qemu_args(iso_path, 6144, 4);
        let qprog = harness::qemu_program();
        let cmd = harness::render_command(qprog, &qargs);
        let nav = harness::serial_menu_nav_plan();
        let markers: Vec<&str> = pin::BOOT_READY_MARKERS.to_vec();

        let attempt = harness::BootAttempt {
            release_tag: pin::RELEASE_TAG,
            boot_iso_asset: pin::BOOT_ISO_ASSET,
            iso_path,
            expected_iso_sha256: expected,
            actual_iso_sha256: actual,
            iso_byte_size: pin::BOOT_ISO_BYTE_SIZE,
            iso_verified: iso_ok,
            qemu_program: qprog,
            qemu_args: &qargs,
            reproducible_command: &cmd,
            nav_plan: &nav,
            allowed_markers: &markers,
            timeout_secs: 180,
            wall_secs: 42,
            qemu_exit: "killed-at-marker",
            booted_at_unix: 1_783_162_520,
            excerpt_lines: 80,
        };
        let dests = harness::EvidenceDests {
            serial_log: &self.serial_log,
            excerpt: &self.excerpt,
            boot_receipt: &self.boot_receipt,
            envelope_receipt: &self.envelope_receipt,
            gap_register: &self.gap_register,
        };
        harness::finalize_boot_evidence(&attempt, &dests).expect("finalize")
    }
}

impl Drop for Fixture {
    /// Remove the scratch dir on drop so temp dirs are cleaned even when a test PANICS before it
    /// would reach an explicit teardown (a manual `cleanup()` leaked on assertion failure).
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn boot_reached_outcome_writes_pass_receipt_with_self_consistent_digest_and_no_gap() {
    // A raw serial-log fixture containing a real closed-set marker (systemd Reached-target line).
    // The earliest marker line wins; the reached-target line (idx 3) precedes the login line.
    let log = "[    0.000000] Booting NixOS stage 1\n\
               [    1.100000] EDD information probe\n\
               [  OK  ] Reached target Basic System.\n\
               nixos login: \n";
    let fx = Fixture::new("reached", log.as_bytes());
    let independent_digest = sha256_hex(&fx.serial_log);

    let outcome = fx.run(true);

    // ---- Outcome: reached, no escalation.
    assert!(
        outcome.boot_reached,
        "marker present + iso verified -> reached"
    );
    assert!(
        !outcome.gap_register_written,
        "a reached boot must NOT emit a gap-register escalation"
    );
    assert_eq!(outcome.verdict.status, harness::BootStatus::Pass);
    let m = outcome
        .verdict
        .marker
        .clone()
        .expect("matched marker recorded");
    assert_eq!(m.marker_index, 3, "systemd Reached-target class");
    assert_eq!(m.line_number, 3);
    assert_eq!(m.matched_text, "Reached target Basic System");

    // ---- Receipt digest self-consistency: recorded == recomputed-from-disk == independent digest.
    assert_eq!(outcome.serial_log_sha256, independent_digest);
    assert_eq!(
        outcome.digest_verification.recorded_sha256,
        independent_digest
    );
    assert_eq!(
        outcome.digest_verification.actual_sha256,
        independent_digest
    );

    // ---- Boot receipt file on disk.
    let receipt = read_json(&fx.boot_receipt);
    assert_eq!(receipt["status"], "pass");
    assert_eq!(receipt["receipt_type"], "boot");
    assert_eq!(
        receipt["boot_verdict"]["boot_reached"],
        serde_json::Value::Bool(true)
    );
    assert_eq!(receipt["boot_verdict"]["verdict_status"], "pass");
    // Records the QEMU invocation.
    assert_eq!(
        receipt["qemu_invocation"]["program"],
        harness::qemu_program()
    );
    assert!(
        receipt["qemu_invocation"]["reproducible_command"]
            .as_str()
            .unwrap()
            .starts_with("qemu-system-x86_64 "),
        "records the runnable QEMU command"
    );
    // Records the artifact digest.
    assert_eq!(
        receipt["artifact"]["verified"],
        serde_json::Value::Bool(true)
    );
    // Records the captured serial-log path + the recomputed serial-log sha256.
    assert_eq!(
        receipt["serial_log"]["path"],
        serde_json::Value::String(fx.serial_log.to_string_lossy().into_owned())
    );
    assert_eq!(
        receipt["serial_log"]["sha256"],
        serde_json::Value::String(independent_digest.clone())
    );
    // Matched marker recorded verbatim.
    assert_eq!(receipt["boot_verdict"]["matched_marker"]["marker_index"], 3);
    assert_eq!(
        receipt["boot_verdict"]["matched_marker"]["matched_text_verbatim"],
        "Reached target Basic System"
    );

    // ---- Envelope receipt records the serial log by reference with the same self-consistent sha.
    let envelope = read_json(&fx.envelope_receipt);
    let serial_ref = envelope["artifacts_handled_by_reference"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["role"] == "serial-log")
        .expect("serial-log referenced");
    assert_eq!(
        serial_ref["sha256"],
        serde_json::Value::String(independent_digest.clone())
    );

    // ---- No simulated evidence: the gap-register file is absent on success.
    assert!(
        !fx.gap_register.exists(),
        "no gap-register file may be produced for a reached boot"
    );
}

#[test]
fn non_boot_outcome_writes_fail_receipt_and_gap_escalation_without_simulated_evidence() {
    // A raw serial-log fixture with NO closed-set marker. The "Welcome to Asterinas NixOS" banner
    // must NOT match the literal "Welcome to NixOS" marker, and no login/shell/reached-target/
    // startup-finished line appears -> honest not-reached.
    let log = "[    0.000000] Booting NixOS stage 1\n\
               Welcome to Asterinas NixOS 0.17.2\n\
               loglevel=4 nohibernate console=ttyS0,115200n8\n\
               [   12.500000] still initializing, no ready marker yet\n";
    let fx = Fixture::new("noboot", log.as_bytes());
    let independent_digest = sha256_hex(&fx.serial_log);

    let outcome = fx.run(true); // ISO verifies; the fail is purely "no boot-ready marker".

    // ---- Outcome: not reached, escalation emitted.
    assert!(!outcome.boot_reached, "no marker -> not reached");
    assert!(
        outcome.gap_register_written,
        "a non-boot MUST emit a gap-register escalation"
    );
    assert_eq!(outcome.verdict.status, harness::BootStatus::Fail);
    assert!(
        outcome.verdict.marker.is_none(),
        "no fabricated marker on a non-boot"
    );

    // ---- Boot receipt: status fail, no fabricated pass, digest still self-consistent.
    let receipt = read_json(&fx.boot_receipt);
    assert_eq!(receipt["status"], "fail");
    assert_eq!(
        receipt["boot_verdict"]["boot_reached"],
        serde_json::Value::Bool(false)
    );
    assert_eq!(receipt["boot_verdict"]["verdict_status"], "fail");
    assert_eq!(
        receipt["boot_verdict"]["matched_marker"],
        serde_json::Value::Null,
        "matched_marker is null; no synthesized marker"
    );
    assert_eq!(
        receipt["serial_log"]["sha256"],
        serde_json::Value::String(independent_digest.clone())
    );

    // ---- Gap-register escalation entry on disk.
    let gap = read_json(&fx.gap_register);
    assert_eq!(gap["status"], "open");
    assert_eq!(gap["gap_id"], "KAW1-BOOT-ENVELOPE-001");
    assert_eq!(
        gap["simulated_or_inferred_evidence_produced"],
        serde_json::Value::Bool(false),
        "honest failure: no simulated or inferred evidence produced"
    );
    // The escalation references the SAME on-disk serial log by path + self-consistent digest.
    assert_eq!(
        gap["evidence"]["serial_log_sha256"],
        serde_json::Value::String(independent_digest.clone())
    );
    assert_eq!(
        gap["evidence"]["serial_log_path"],
        serde_json::Value::String(fx.serial_log.to_string_lossy().into_owned())
    );
    assert!(
        gap["observed_fact"]
            .as_str()
            .unwrap()
            .contains(pin::BOOT_ISO_ASSET),
        "observed fact records the concrete unmodified asset"
    );
}

#[test]
fn empty_serial_log_is_honest_fail_never_a_synthesized_pass() {
    // A timed-out boot that emitted nothing to serial. A zero-byte capture cannot fabricate a
    // marker, so the verdict is an honest fail with a gap-register escalation.
    let fx = Fixture::new("empty", b"");
    let empty_digest = sha256_hex(&fx.serial_log);
    // sha256 of the empty input (NIST vector) — the referenced file really is zero bytes.
    assert_eq!(
        empty_digest,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    let outcome = fx.run(true);

    assert!(!outcome.boot_reached);
    assert!(outcome.gap_register_written);
    assert_eq!(outcome.serial_log_byte_size, 0);
    assert!(outcome.verdict.marker.is_none());

    let receipt = read_json(&fx.boot_receipt);
    assert_eq!(receipt["status"], "fail");
    assert_eq!(
        receipt["serial_log"]["sha256"],
        serde_json::Value::String(empty_digest.clone())
    );
    assert_eq!(receipt["serial_log"]["byte_size"], 0);

    let gap = read_json(&fx.gap_register);
    assert_eq!(
        gap["simulated_or_inferred_evidence_produced"],
        serde_json::Value::Bool(false)
    );
    assert_eq!(
        gap["evidence"]["serial_log_sha256"],
        serde_json::Value::String(empty_digest)
    );
}
