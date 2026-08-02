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
//!   * marker-bearing fixtures remain `fixture` / `REFUSED_NO_ORACLE`, never pass or observed;
//!   * observed marker-free captures become `ObservedFailure` while retaining artifact pointers;
//!   * missing or invalid artifact oracles fail with typed `AbsentOracle`;
//!   * Darwin arm64 with configured x86_64 TCG is not classified unsupported solely by host arch.

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
    qemu_stderr: PathBuf,
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
        let qemu_stderr = dir.join("qemu.stderr.log");
        fs::write(&qemu_stderr, b"fixture stderr\n").expect("write stderr fixture");
        Fixture {
            qemu_stderr,
            excerpt: dir.join("boot-serial.excerpt.txt"),
            boot_receipt: dir.join("boot-receipt.json"),
            envelope_receipt: dir.join("envelope-receipt.json"),
            gap_register: dir.join("gap-register.json"),
            serial_log,
            dir,
        }
    }

    fn run(
        &self,
        iso_ok: bool,
        evidence_origin: harness::EvidenceOrigin,
    ) -> Result<harness::FinalizeOutcome, harness::FinalizeError> {
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
            evidence_origin,
            upstream_repository: pin::UPSTREAM_REPOSITORY,
            repository_commit: pin::RELEASE_COMMIT,
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
            qemu_stderr: &self.qemu_stderr,
            excerpt: &self.excerpt,
            boot_receipt: &self.boot_receipt,
            envelope_receipt: &self.envelope_receipt,
            gap_register: &self.gap_register,
        };
        harness::finalize_boot_evidence(&attempt, &dests)
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
fn marker_bearing_fixture_writes_refusal_never_observed_real_or_pass() {
    let log = "[    0.000000] Booting NixOS stage 1\n\
               [  OK  ] Reached target Basic System.\n";
    let fx = Fixture::new("fixture-marker", log.as_bytes());

    let outcome = fx
        .run(true, harness::EvidenceOrigin::Fixture)
        .expect("fixture finalizes as refusal");

    assert!(!outcome.boot_reached);
    assert_eq!(outcome.evidence_origin, harness::EvidenceOrigin::Fixture);
    assert_eq!(
        outcome.evidence_outcome,
        harness::EvidenceOutcome::AbsentOracle
    );
    assert_eq!(outcome.terminal, "REFUSED_NO_ORACLE");
    assert!(outcome.gap_register_written);
    let receipt = read_json(&fx.boot_receipt);
    assert_eq!(receipt["status"], "fail");
    assert_eq!(receipt["boot_verdict"]["evidence_origin"], "fixture");
    assert_eq!(receipt["boot_verdict"]["observed_success"], false);
    assert_eq!(
        receipt["boot_verdict"]["derived_solely_from_real_captured_log"],
        false
    );
}

#[test]
fn observed_no_marker_capture_is_observed_failure_and_retains_artifact_pointers() {
    let log = "[    0.000000] Booting NixOS stage 1\n\
               [   12.500000] still initializing, no ready marker yet\n";
    let fx = Fixture::new("observed-failure", log.as_bytes());

    let outcome = fx
        .run(true, harness::EvidenceOrigin::Observed)
        .expect("observed failure finalizes");

    assert_eq!(
        outcome.evidence_outcome,
        harness::EvidenceOutcome::ObservedFailure
    );
    assert_eq!(outcome.terminal, "OBSERVED_FAILURE");
    assert!(!outcome.boot_reached);
    assert!(outcome.artifact_envelope.artifacts.iter().any(|artifact| {
        artifact.role == harness::ArtifactRole::SerialLog
            && artifact.source_pointer == fx.serial_log.to_string_lossy()
    }));
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

    let outcome = fx
        .run(true, harness::EvidenceOrigin::Fixture)
        .expect("fixture non-boot finalizes"); // ISO verifies; no marker.

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
fn missing_serial_oracle_is_typed_refused_no_oracle() {
    let fx = Fixture::new("empty", b"");

    let error = fx
        .run(true, harness::EvidenceOrigin::Fixture)
        .expect_err("zero-size serial capture is not an artifact oracle");

    assert!(matches!(
        error,
        harness::FinalizeError::Evidence(harness::EvidenceOutcome::AbsentOracle)
    ));
    assert_eq!(error.to_string(), "REFUSED_NO_ORACLE");
}

#[test]
fn fixture_markers_cannot_claim_observed_real_or_pass() {
    let verdict = harness::derive_boot_verdict(
        "[  OK  ] Reached target Basic System.\n",
        &pin::BOOT_READY_MARKERS,
        harness::EvidenceOrigin::Fixture,
    )
    .expect("fixture marker parses");

    assert_eq!(verdict.origin, harness::EvidenceOrigin::Fixture);
    assert_eq!(verdict.outcome, harness::EvidenceOutcome::AbsentOracle);
    assert!(!verdict.observed_success);
    assert_eq!(verdict.status, harness::BootStatus::Fail);
}

#[test]
fn artifact_envelope_rejects_malformed_or_incomplete_evidence() {
    let artifact = harness::ArtifactRef {
        role: harness::ArtifactRole::SerialLog,
        source_pointer: "serial.log".to_string(),
        sha256: "0".repeat(64),
        byte_size: 1,
    };
    assert_eq!(
        harness::ArtifactEnvelope::validate(
            pin::UPSTREAM_REPOSITORY,
            "not-a-commit",
            vec![artifact.clone()],
            &[harness::ArtifactRole::SerialLog],
        ),
        Err(harness::EvidenceOutcome::AbsentOracle)
    );
    for invalid in [
        harness::ArtifactRef {
            source_pointer: String::new(),
            ..artifact.clone()
        },
        harness::ArtifactRef {
            sha256: String::new(),
            ..artifact.clone()
        },
        harness::ArtifactRef {
            byte_size: 0,
            ..artifact.clone()
        },
    ] {
        assert_eq!(
            harness::ArtifactEnvelope::validate(
                pin::UPSTREAM_REPOSITORY,
                pin::RELEASE_COMMIT,
                vec![invalid],
                &[harness::ArtifactRole::SerialLog],
            ),
            Err(harness::EvidenceOutcome::AbsentOracle)
        );
    }
    assert_eq!(
        harness::ArtifactEnvelope::validate(
            pin::UPSTREAM_REPOSITORY,
            pin::RELEASE_COMMIT,
            vec![artifact],
            &[harness::ArtifactRole::Iso, harness::ArtifactRole::SerialLog],
        ),
        Err(harness::EvidenceOutcome::AbsentOracle)
    );
}

#[test]
fn darwin_arm64_with_x86_64_tcg_is_not_unsupported() {
    assert_eq!(
        harness::classify_qemu_path("darwin", "aarch64", true),
        harness::EvidenceOutcome::Success
    );
}
