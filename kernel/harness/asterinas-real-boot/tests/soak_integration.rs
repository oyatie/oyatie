#![forbid(unsafe_code)]
//! SHARD-2 SOAK integration test: exercise the PURE soak loop + attempt/aggregate/honest-fail
//! assembly (`harness::soak::run_soak_with_boot_runner`) end-to-end against on-disk serial-log
//! fixtures, with NO QEMU and NO network.
//!
//! This drives the SAME orchestration the QEMU soak binary runs (the binary is a thin
//! `std::process` wrapper that injects a QEMU-driving per-boot runner); here the per-boot runner
//! writes a raw serial-log fixture to disk and assembles the boot record from it, so the whole
//! soak loop (3-attempt retry, per-attempt receipts, aggregate, honest-fail) is verified
//! deterministically.
//!
//! What is proven here:
//!   * synthetic marker-bearing 10/10 fixtures remain fixture-only and never serialize pass/real;
//!   * fixture retries cannot aggregate into observed success;
//!   * exhausted fixture attempts emit the honest-fail gap without simulated evidence claims.

use kernel_asterinas_boundary as pin;
use kernel_asterinas_real_boot as harness;
use kernel_asterinas_real_boot::soak;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A scratch run dir that is removed on drop (survives test panics).
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn scratch(tag: &str) -> Scratch {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut dir = std::env::temp_dir();
    dir.push(format!("kernel-soak-it-{tag}-{nanos}"));
    fs::create_dir_all(&dir).expect("create scratch dir");
    Scratch(dir)
}

fn read_json(path: &Path) -> serde_json::Value {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn make_cfg() -> soak::SoakConfig {
    let iso_path = "kernel/target/artifacts/asterinas-nixos-0.17.2-x86_64.iso";
    let qargs = harness::build_qemu_args(iso_path, 6144, 4);
    let cmd = harness::render_command(harness::qemu_program(), &qargs);
    soak::SoakConfig {
        upstream_repository: pin::UPSTREAM_REPOSITORY.to_string(),
        repository_commit: pin::RELEASE_COMMIT.to_string(),
        release_tag: pin::RELEASE_TAG.to_string(),
        iso: soak::IsoArtifact {
            asset_name: pin::BOOT_ISO_ASSET.to_string(),
            download_url: pin::BOOT_ISO_DOWNLOAD_URL.to_string(),
            local_path: iso_path.to_string(),
            // A verified black-box ISO (expected == actual): the boot runner never touches it.
            expected_sha256: pin::BOOT_ISO_SHA256.to_string(),
            actual_sha256: pin::BOOT_ISO_SHA256.to_string(),
            byte_size: pin::BOOT_ISO_BYTE_SIZE,
            verified: true,
        },
        qemu_program: harness::qemu_program().to_string(),
        qemu_args: qargs,
        reproducible_command: cmd,
        iteration_count: soak::ITERATION_COUNT,
        max_attempts: soak::MAX_SOAK_ATTEMPTS,
        per_boot_timeout_secs: soak::PER_BOOT_TIMEOUT_SECS,
        allowed_markers: pin::BOOT_READY_MARKERS
            .iter()
            .map(|m| m.to_string())
            .collect(),
    }
}

/// A per-boot runner that writes a REAL serial-log fixture to disk (marker-bearing when `clean`,
/// marker-free otherwise) and assembles the boot record from it — the same path the QEMU binary
/// takes after a real boot, minus QEMU. The record references the on-disk fixture by path +
/// recomputed digest.
fn fixture_boot(
    attempt_id: &str,
    iteration: usize,
    attempt_dir: &Path,
    clean: bool,
) -> Result<soak::BootRecord, Box<dyn Error>> {
    let iteration_dir = attempt_dir.join(format!("boot-{iteration:02}"));
    fs::create_dir_all(&iteration_dir)?;
    let serial_log = iteration_dir.join("serial.log");
    // Clean fixture carries a real closed-set marker (systemd Reached-target line); the unclean
    // fixture carries none -> the assembled record is honestly unclean (timed-out, no marker).
    let contents: &[u8] = if clean {
        b"[    1.10] EDD information probe\n[  OK  ] Reached target Basic System.\nnixos login: \n"
    } else {
        b"[    0.00] Booting NixOS stage 1\n[   12.5] still initializing, no ready marker yet\n"
    };
    fs::write(&serial_log, contents)?;

    let (sha256, byte_size) = harness::sha256_file(&serial_log)?;
    let final_log = harness::read_log_file(&serial_log)?;
    let marker = harness::find_boot_marker(&final_log, &pin::BOOT_READY_MARKERS)?;
    let live = if clean {
        soak::LiveTermination::MarkerKilled
    } else {
        soak::LiveTermination::TimedOut
    };
    Ok(soak::assemble_boot_record(soak::BootObservation {
        evidence_origin: harness::EvidenceOrigin::Fixture,
        attempt_id: attempt_id.to_string(),
        iteration_index: iteration,
        live,
        marker,
        qemu_exit_status: soak::QemuExitStatus {
            observed: true,
            code: None,
            signal: Some(9),
            description: "signal: 9 (SIGKILL)".to_string(),
        },
        elapsed_seconds: if clean { 42.0 } else { 180.0 },
        serial_log_path: serial_log.to_string_lossy().into_owned(),
        serial_log_sha256: sha256,
        serial_log_byte_size: byte_size,
    }))
}

#[test]
fn synthetic_ten_of_ten_soak_remains_fixture_and_never_serializes_pass_or_real() {
    let run = scratch("fixture-10");
    let cfg = make_cfg();
    let dests = soak::SoakDests {
        run_dir: run.0.clone(),
    };

    let outcome =
        soak::run_soak_with_boot_runner(&cfg, &dests, |_cfg, attempt_id, iteration, dir| {
            fixture_boot(attempt_id, iteration, dir, true)
        })
        .expect("fixture soak runs");

    assert_eq!(outcome.verdict, "fail");
    assert!(outcome.passing_attempt_id.is_none());
    assert_eq!(outcome.attempts.len(), soak::MAX_SOAK_ATTEMPTS);
    assert!(outcome.gap_register_written);
    assert!(outcome.attempts.iter().all(|attempt| {
        attempt.verdict == "fail"
            && attempt.clean_boots == 0
            && attempt.boot_records.iter().all(|record| {
                record.evidence_origin == harness::EvidenceOrigin::Fixture && !record.clean
            })
    }));

    let aggregate = read_json(&outcome.aggregate_receipt_path);
    assert_eq!(aggregate["overall_verdict"], "fail");
    assert_eq!(aggregate["evidence_origin"], "fixture");
    assert_eq!(aggregate["observed_success"], false);
    assert_eq!(aggregate["passing_attempt_id"], serde_json::Value::Null);
    let attempt_receipt = read_json(Path::new(&outcome.attempts[0].receipt_path));
    assert_eq!(attempt_receipt["verdict"], "fail");
    assert_eq!(attempt_receipt["evidence_origin"], "fixture");
    assert_eq!(attempt_receipt["observed_success"], false);
}

#[test]
fn fixture_retries_never_aggregate_into_observed_success() {
    let run = scratch("fixture-retry");
    let cfg = make_cfg();
    let dests = soak::SoakDests {
        run_dir: run.0.clone(),
    };

    let outcome =
        soak::run_soak_with_boot_runner(&cfg, &dests, |_cfg, attempt_id, iteration, dir| {
            fixture_boot(attempt_id, iteration, dir, true)
        })
        .expect("fixture soak runs");

    assert_eq!(outcome.verdict, "fail");
    assert!(outcome.passing_attempt_id.is_none());
    assert_eq!(outcome.attempts.len(), soak::MAX_SOAK_ATTEMPTS);
    assert!(
        outcome
            .attempts
            .iter()
            .all(|attempt| !soak::attempt_is_pass(&attempt.boot_records, soak::ITERATION_COUNT))
    );
}

#[test]
fn all_attempts_exhausted_emits_honest_fail_gap_without_simulated_evidence() {
    let run = scratch("exhausted");
    let cfg = make_cfg();
    let dests = soak::SoakDests {
        run_dir: run.0.clone(),
    };

    // Every attempt has an unclean first boot -> no attempt reaches a clean 10/10.
    let outcome =
        soak::run_soak_with_boot_runner(&cfg, &dests, |_cfg, attempt_id, iteration, dir| {
            fixture_boot(attempt_id, iteration, dir, iteration != 1)
        })
        .expect("soak runs");

    assert_eq!(outcome.verdict, "fail");
    assert!(outcome.passing_attempt_id.is_none());
    assert_eq!(outcome.attempts.len(), soak::MAX_SOAK_ATTEMPTS);
    assert!(outcome.gap_register_written);
    for a in &outcome.attempts {
        assert_eq!(a.verdict, "fail");
        assert_eq!(a.clean_boots, 0);
        assert!(!soak::attempt_is_pass(
            &a.boot_records,
            soak::ITERATION_COUNT
        ));
    }

    // Aggregate honest-fail: overall fail, no passing attempt, gap-register with the
    // simulated-evidence flag explicitly false.
    let aggregate = read_json(&outcome.aggregate_receipt_path);
    assert_eq!(aggregate["overall_verdict"], "fail");
    assert_eq!(aggregate["passing_attempt_id"], serde_json::Value::Null);
    assert_eq!(aggregate["attempt_count"], soak::MAX_SOAK_ATTEMPTS);
    let gap = &aggregate["gap_register_entry"];
    assert_eq!(gap["gap_id"], "KAW1-SOAK-001");
    assert_eq!(gap["status"], "open");
    assert_eq!(
        gap["simulated_or_inferred_evidence_produced"],
        serde_json::Value::Bool(false),
        "honest failure: no simulated or inferred evidence produced"
    );
    assert!(
        gap["blocker"]
            .as_str()
            .unwrap()
            .contains("consecutive clean isolated QEMU cold boots")
    );
}

#[test]
fn mis_identified_boot_record_fails_closed_never_a_silent_pass() {
    // The anti-aggregation check (`attempt_is_pass`) verifies same-ownership against
    // records[0].attempt_id — so a runner that mis-stamped every record with ONE id would produce
    // an internally-consistent, passing attempt. The orchestrator must instead bind each returned
    // record to the identity it REQUESTED and fail closed on any mismatch (an infrastructure
    // invariant violation -> Err/exit 1, never a fabricated pass). Here a hostile runner ignores the
    // requested (attempt_id, iteration) and always stamps `attempt-999` iteration 1 with a clean,
    // marker-bearing fixture: absent the binding guard this would look like a valid clean boot.
    let run = scratch("mis-id");
    let cfg = make_cfg();
    let dests = soak::SoakDests {
        run_dir: run.0.clone(),
    };

    let err =
        soak::run_soak_with_boot_runner(&cfg, &dests, |_cfg, _attempt_id, _iteration, dir| {
            fixture_boot("attempt-999", 1, dir, true)
        })
        .expect_err("a mis-identified boot record must fail closed, never yield a pass");
    assert!(
        err.to_string().contains("mis-identified"),
        "the orchestrator rejects a record whose identity does not match the request, got: {err}"
    );
}
