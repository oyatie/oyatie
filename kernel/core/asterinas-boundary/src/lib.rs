#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const PARENT_SEED_ID: &str = "seed_d646661cedc6";
pub const PRIOR_SHARD_ID: &str = "seed_kaw1_real_boot_slice_v1";
pub const UPSTREAM_REPOSITORY: &str = "https://github.com/asterinas/asterinas";
pub const RELEASE_TAG: &str = "v0.17.2";
pub const RELEASE_COMMIT: &str = "23adfdfd72b05cee8d232809caea81a4b33d3488";
pub const ISO_ASSET_NAME: &str = "asterinas-nixos-0.17.2-x86_64.iso";
pub const ISO_SHA256: &str = "bf6e161ecc8b8080b842a339cee5f55d18b93d99b1e39c7c07681ff3aca0090a";
pub const ISO_ASSET_URL: &str = "https://github.com/asterinas/asterinas/releases/download/v0.17.2/asterinas-nixos-0.17.2-x86_64.iso";
pub const ITERATION_COUNT: usize = 10;
pub const MAX_SOAK_ATTEMPTS: usize = 3;
pub const PER_BOOT_TIMEOUT_SECONDS: u64 = 180;
pub const MAX_RECEIPT_JSON_LINE_BYTES: usize = 16 * 1024;
pub const PIN_MANIFEST: &str = include_str!("../pins/asterinas-release-v0.17.2.json");

const BOOT_READY_MARKERS: &[(&str, &str)] = &[
    ("login", r"/login:\s*$/"),
    ("shell", r"/[#$]\s$/"),
    ("nixos", r"/Welcome to NixOS/"),
    (
        "systemd_startup_finished",
        r"/systemd\[1\]:\s+Startup finished/",
    ),
    (
        "systemd_target_reached",
        r"/Reached target .*(Multi-User|Basic System|Login Prompts)/",
    ),
];

#[derive(Debug)]
pub struct SoakError {
    message: String, // data_class: PUBLIC
}

impl SoakError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for SoakError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SoakError {}

impl From<std::io::Error> for SoakError {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<serde_json::Error> for SoakError {
    fn from(value: serde_json::Error) -> Self {
        Self::new(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SoakError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinManifest {
    pub asset_name: String,          // data_class: PUBLIC
    pub asset_sha256: String,        // data_class: PUBLIC
    pub asset_url: String,           // data_class: PUBLIC
    pub published_at: String,        // data_class: PUBLIC
    pub release_commit: String,      // data_class: PUBLIC
    pub release_tag: String,         // data_class: PUBLIC
    pub upstream_repository: String, // data_class: PUBLIC
    pub license_boundary: String,    // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsterinasReleasePin {
    pub tag: &'static str,                 // data_class: PUBLIC
    pub commit: &'static str,              // data_class: PUBLIC
    pub upstream_repository: &'static str, // data_class: PUBLIC
    pub iso_asset_name: &'static str,      // data_class: PUBLIC
    pub iso_asset_url: &'static str,       // data_class: PUBLIC
    pub iso_sha256: &'static str,          // data_class: PUBLIC
    pub license_boundary: &'static str,    // data_class: PUBLIC
}

pub fn release_pin() -> AsterinasReleasePin {
    AsterinasReleasePin {
        tag: RELEASE_TAG,
        commit: RELEASE_COMMIT,
        upstream_repository: UPSTREAM_REPOSITORY,
        iso_asset_name: ISO_ASSET_NAME,
        iso_asset_url: ISO_ASSET_URL,
        iso_sha256: ISO_SHA256,
        license_boundary: "MPL-2.0 upstream ISO is consumed unmodified as a black-box artifact; Oyatie-owned harness code does not modify, vendor, or derive from upstream ISO bytes.",
    }
}

pub fn parsed_pin_manifest() -> Result<PinManifest> {
    Ok(serde_json::from_str(PIN_MANIFEST)?)
}

#[derive(Debug, Clone)]
pub struct SoakConfig {
    pub qemu_binary: PathBuf,       // data_class: PUBLIC
    pub downloader_binary: PathBuf, // data_class: PUBLIC
    pub output_base_dir: PathBuf,   // data_class: PUBLIC
    pub run_id: String,             // data_class: PUBLIC
}

impl SoakConfig {
    pub fn from_env() -> Self {
        let qemu_binary = std::env::var_os("ASTERINAS_QEMU_SYSTEM_X86_64")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("qemu-system-x86_64"));
        let downloader_binary = std::env::var_os("ASTERINAS_DOWNLOAD_TOOL")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("curl"));
        let output_base_dir = std::env::var_os("ASTERINAS_SOAK_OUTPUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/asterinas-soak"));
        let run_id = std::env::var("ASTERINAS_SOAK_RUN_ID")
            .unwrap_or_else(|_| format!("run-{}-pid-{}", unix_time_seconds(), std::process::id()));
        Self {
            qemu_binary,
            downloader_binary,
            output_base_dir,
            run_id,
        }
    }

    fn iso_path(&self) -> PathBuf {
        self.output_base_dir.join("cache").join(ISO_ASSET_NAME)
    }

    fn run_dir(&self) -> PathBuf {
        self.output_base_dir.join("runs").join(&self.run_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkerMatch {
    pub marker_name: String,    // data_class: PUBLIC
    pub regex: String,          // data_class: PUBLIC
    pub matched_string: String, // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactReceipt {
    pub asset_name: String,          // data_class: PUBLIC
    pub asset_url: String,           // data_class: PUBLIC
    pub expected_sha256: String,     // data_class: PUBLIC
    pub actual_sha256: String,       // data_class: PUBLIC
    pub byte_size: u64,              // data_class: PUBLIC
    pub local_path: String,          // data_class: PUBLIC
    pub verification_status: String, // data_class: PUBLIC
    pub fetch_action: String,        // data_class: PUBLIC
    pub verified_at_unix: u64,       // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuRuntime {
    pub qemu_binary: String,                    // data_class: PUBLIC
    pub arch: String,                           // data_class: PUBLIC
    pub machine_model: String,                  // data_class: PUBLIC
    pub arguments: Vec<String>,                 // data_class: PUBLIC
    pub independently_runnable_command: String, // data_class: PUBLIC
    pub per_boot_timeout_seconds: u64,          // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmIsolation {
    pub method: String,                    // data_class: PUBLIC
    pub no_disk: bool,                     // data_class: PUBLIC
    pub no_snapshot: bool,                 // data_class: PUBLIC
    pub fresh_process_per_iteration: bool, // data_class: PUBLIC
    pub serial_log_per_iteration: bool,    // data_class: PUBLIC
    pub state_carryover: String,           // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QemuExitStatusReceipt {
    pub observed: bool,      // data_class: PUBLIC
    pub code: Option<i32>,   // data_class: PUBLIC
    pub signal: Option<i32>, // data_class: PUBLIC
    pub description: String, // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootRecord {
    pub attempt_id: String,                      // data_class: PUBLIC
    pub iteration_index: usize,                  // data_class: PUBLIC
    pub clean: bool,                             // data_class: PUBLIC
    pub elapsed_seconds: f64,                    // data_class: PUBLIC
    pub timeout_hit: bool,                       // data_class: PUBLIC
    pub qemu_exit_status: QemuExitStatusReceipt, // data_class: PUBLIC
    pub termination_reason: String,              // data_class: PUBLIC
    pub matched_marker: Option<MarkerMatch>,     // data_class: PUBLIC
    pub raw_serial_log_path: String,             // data_class: PUBLIC
    pub raw_serial_log_sha256: String,           // data_class: PUBLIC
    pub raw_serial_log_bytes: u64,               // data_class: PUBLIC
    pub qemu_stderr_path: String,                // data_class: PUBLIC
    pub qemu_stderr_sha256: String,              // data_class: PUBLIC
    pub qemu_stderr_bytes: u64,                  // data_class: PUBLIC
    pub serial_input_events: Vec<String>,        // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoakAttemptReceipt {
    pub attempt_id: String,              // data_class: PUBLIC
    pub attempt_index: usize,            // data_class: PUBLIC
    pub verdict: String,                 // data_class: PUBLIC
    pub clean_boots: usize,              // data_class: PUBLIC
    pub required_clean_boots: usize,     // data_class: PUBLIC
    pub started_at_unix: u64,            // data_class: PUBLIC
    pub completed_at_unix: u64,          // data_class: PUBLIC
    pub iso: ArtifactReceipt,            // data_class: PUBLIC
    pub qemu_runtime: QemuRuntime,       // data_class: PUBLIC
    pub vm_isolation: VmIsolation,       // data_class: PUBLIC
    pub boot_ready_markers: Vec<String>, // data_class: PUBLIC
    pub boot_records: Vec<BootRecord>,   // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptReference {
    pub attempt_id: String,            // data_class: PUBLIC
    pub verdict: String,               // data_class: PUBLIC
    pub clean_boots: usize,            // data_class: PUBLIC
    pub receipt_path: String,          // data_class: PUBLIC
    pub receipt_sha256: String,        // data_class: PUBLIC
    pub boot_records: Vec<BootRecord>, // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRegisterEntry {
    pub parent_seed_id: String,        // data_class: PUBLIC
    pub prior_shard_id: String,        // data_class: PUBLIC
    pub blocker: String,               // data_class: PUBLIC
    pub fate: String,                  // data_class: PUBLIC
    pub honest_fail_reference: String, // data_class: PUBLIC
    pub acceptance_criteria: String,   // data_class: PUBLIC
    pub verification_path: String,     // data_class: PUBLIC
    pub created_at_unix: u64,          // data_class: PUBLIC
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateReceipt {
    pub parent_seed_id: String,                       // data_class: PUBLIC
    pub prior_shard_id: String,                       // data_class: PUBLIC
    pub overall_verdict: String,                      // data_class: PUBLIC
    pub passing_attempt_id: Option<String>,           // data_class: PUBLIC
    pub iso_asset: ArtifactReceipt,                   // data_class: PUBLIC
    pub iteration_count: usize,                       // data_class: PUBLIC
    pub per_boot_timeout_seconds: u64,                // data_class: PUBLIC
    pub max_soak_attempts: usize,                     // data_class: PUBLIC
    pub qemu_runtime: QemuRuntime,                    // data_class: PUBLIC
    pub vm_isolation: VmIsolation,                    // data_class: PUBLIC
    pub boot_ready_markers: Vec<String>,              // data_class: PUBLIC
    pub soak_attempts: Vec<AttemptReference>,         // data_class: PUBLIC
    pub gap_register_entry: Option<GapRegisterEntry>, // data_class: PUBLIC
}

#[derive(Debug, Clone)]
pub struct SoakRunOutput {
    pub verdict: String,                  // data_class: PUBLIC
    pub aggregate_receipt_path: PathBuf,  // data_class: PUBLIC
    pub aggregate_receipt_sha256: String, // data_class: PUBLIC
    pub attempts: Vec<AttemptReference>,  // data_class: PUBLIC
}

pub fn run_soak_from_env() -> Result<SoakRunOutput> {
    run_soak(&SoakConfig::from_env())
}

pub fn run_soak(config: &SoakConfig) -> Result<SoakRunOutput> {
    fs::create_dir_all(
        config
            .iso_path()
            .parent()
            .ok_or_else(|| SoakError::new("ISO path has no parent directory"))?,
    )?;

    let iso = ensure_iso(config)?;
    run_soak_with_boot_runner(config, iso, run_boot_iteration)
}

fn run_soak_with_boot_runner<F>(
    config: &SoakConfig,
    iso: ArtifactReceipt,
    mut boot_runner: F,
) -> Result<SoakRunOutput>
where
    F: FnMut(&SoakConfig, &str, usize, &Path, &Path) -> Result<BootRecord>,
{
    fs::create_dir_all(config.run_dir())?;

    let qemu_runtime = qemu_runtime(config, &iso.local_path);
    let vm_isolation = vm_isolation();
    let boot_ready_markers = boot_ready_marker_regexes();

    let mut attempts = Vec::new();
    let mut passing_attempt_id = None;

    for attempt_index in 1..=MAX_SOAK_ATTEMPTS {
        let attempt_id = format!("attempt-{attempt_index:03}");
        let attempt_dir = config.run_dir().join(&attempt_id);
        fs::create_dir_all(&attempt_dir)?;
        let started_at_unix = unix_time_seconds();
        let mut boot_records = Vec::with_capacity(ITERATION_COUNT);

        for iteration_index in 1..=ITERATION_COUNT {
            let record = boot_runner(
                config,
                &attempt_id,
                iteration_index,
                Path::new(&iso.local_path),
                &attempt_dir,
            )?;
            boot_records.push(record);
        }

        let clean_boots = boot_records.iter().filter(|record| record.clean).count();
        let verdict = if attempt_is_pass(&boot_records) {
            "pass"
        } else {
            "fail"
        }
        .to_string();
        let attempt_receipt = SoakAttemptReceipt {
            attempt_id: attempt_id.clone(),
            attempt_index,
            verdict: verdict.clone(),
            clean_boots,
            required_clean_boots: ITERATION_COUNT,
            started_at_unix,
            completed_at_unix: unix_time_seconds(),
            iso: iso.clone(),
            qemu_runtime: qemu_runtime.clone(),
            vm_isolation: vm_isolation.clone(),
            boot_ready_markers: boot_ready_markers.clone(),
            boot_records: boot_records.clone(),
        };
        let attempt_receipt_path = attempt_dir.join("attempt-receipt.json");
        write_json_file(&attempt_receipt_path, &attempt_receipt)?;
        let attempt_receipt_sha256 = sha256_file(&attempt_receipt_path)?;
        attempts.push(AttemptReference {
            attempt_id: attempt_id.clone(),
            verdict: verdict.clone(),
            clean_boots,
            receipt_path: path_string(&attempt_receipt_path),
            receipt_sha256: attempt_receipt_sha256,
            boot_records,
        });

        if verdict == "pass" {
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
    let gap_register_entry = if overall_verdict == "fail" {
        Some(GapRegisterEntry {
            parent_seed_id: PARENT_SEED_ID.to_string(),
            prior_shard_id: PRIOR_SHARD_ID.to_string(),
            blocker: "No soak attempt reached ten consecutive clean isolated QEMU cold boots within the allowed attempt budget.".to_string(),
            fate: "HonestFail".to_string(),
            honest_fail_reference: path_string(&config.run_dir()),
            acceptance_criteria: "Ten QEMU cold-boot iterations complete consecutively with fresh VM isolation and no state carry-over between iterations.".to_string(),
            verification_path: "Inspect aggregate-receipt.json and per-attempt raw serial log path+digest records.".to_string(),
            created_at_unix: unix_time_seconds(),
        })
    } else {
        None
    };

    let aggregate = AggregateReceipt {
        parent_seed_id: PARENT_SEED_ID.to_string(),
        prior_shard_id: PRIOR_SHARD_ID.to_string(),
        overall_verdict: overall_verdict.clone(),
        passing_attempt_id,
        iso_asset: iso,
        iteration_count: ITERATION_COUNT,
        per_boot_timeout_seconds: PER_BOOT_TIMEOUT_SECONDS,
        max_soak_attempts: MAX_SOAK_ATTEMPTS,
        qemu_runtime,
        vm_isolation,
        boot_ready_markers,
        soak_attempts: attempts.clone(),
        gap_register_entry,
    };

    let aggregate_receipt_path = config.run_dir().join("aggregate-receipt.json");
    write_json_file(&aggregate_receipt_path, &aggregate)?;
    let aggregate_receipt_sha256 = sha256_file(&aggregate_receipt_path)?;

    Ok(SoakRunOutput {
        verdict: overall_verdict,
        aggregate_receipt_path,
        aggregate_receipt_sha256,
        attempts,
    })
}

pub fn boot_ready_marker_regexes() -> Vec<String> {
    BOOT_READY_MARKERS
        .iter()
        .map(|(_, regex)| (*regex).to_string())
        .collect()
}

pub fn find_boot_ready_marker(serial_bytes: &[u8]) -> Option<MarkerMatch> {
    let serial = String::from_utf8_lossy(serial_bytes);
    if serial.contains("Welcome to NixOS") {
        return Some(MarkerMatch {
            marker_name: "nixos".to_string(),
            regex: r"/Welcome to NixOS/".to_string(),
            matched_string: "Welcome to NixOS".to_string(),
        });
    }

    for line in serial.split('\n') {
        let line = line.trim_end_matches('\r');
        if let Some(matched) = match_systemd_startup_finished(line) {
            return Some(MarkerMatch {
                marker_name: "systemd_startup_finished".to_string(),
                regex: r"/systemd\[1\]:\s+Startup finished/".to_string(),
                matched_string: matched,
            });
        }
        if let Some(matched) = match_systemd_target(line) {
            return Some(MarkerMatch {
                marker_name: "systemd_target_reached".to_string(),
                regex: r"/Reached target .*(Multi-User|Basic System|Login Prompts)/".to_string(),
                matched_string: matched,
            });
        }
        if let Some(matched) = match_login_prompt(line) {
            return Some(MarkerMatch {
                marker_name: "login".to_string(),
                regex: r"/login:\s*$/".to_string(),
                matched_string: matched,
            });
        }
        if let Some(matched) = match_shell_prompt(line) {
            return Some(MarkerMatch {
                marker_name: "shell".to_string(),
                regex: r"/[#$]\s$/".to_string(),
                matched_string: matched,
            });
        }
    }

    None
}

pub fn is_clean_boot(record: &BootRecord) -> bool {
    record.matched_marker.is_some()
        && !record.timeout_hit
        && record.termination_reason == "boot_ready_marker_matched_then_harness_terminated_qemu"
}

pub fn attempt_is_pass(boot_records: &[BootRecord]) -> bool {
    let Some(first) = boot_records.first() else {
        return false;
    };
    boot_records.len() == ITERATION_COUNT
        && boot_records.iter().enumerate().all(|(offset, record)| {
            record.iteration_index == offset + 1
                && record.attempt_id == first.attempt_id
                && record.clean
                && is_clean_boot(record)
        })
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn ensure_iso(config: &SoakConfig) -> Result<ArtifactReceipt> {
    let iso_path = config.iso_path();
    let fetch_action = if iso_path.exists() && sha256_file(&iso_path)? == ISO_SHA256 {
        "reused_existing_verified"
    } else {
        if iso_path.exists() {
            fs::remove_file(&iso_path)?;
        }
        download_iso(config, &iso_path)?;
        "downloaded_and_verified"
    };

    let actual_sha256 = sha256_file(&iso_path)?;
    if actual_sha256 != ISO_SHA256 {
        return Err(SoakError::new(format!(
            "ISO digest mismatch for {}: expected {}, actual {}",
            path_string(&iso_path),
            ISO_SHA256,
            actual_sha256
        )));
    }
    let byte_size = fs::metadata(&iso_path)?.len();
    Ok(ArtifactReceipt {
        asset_name: ISO_ASSET_NAME.to_string(),
        asset_url: ISO_ASSET_URL.to_string(),
        expected_sha256: ISO_SHA256.to_string(),
        actual_sha256,
        byte_size,
        local_path: path_string(&iso_path),
        verification_status: "sha256_verified".to_string(),
        fetch_action: fetch_action.to_string(),
        verified_at_unix: unix_time_seconds(),
    })
}

fn download_iso(config: &SoakConfig, iso_path: &Path) -> Result<()> {
    let parent = iso_path
        .parent()
        .ok_or_else(|| SoakError::new("ISO path has no parent directory"))?;
    fs::create_dir_all(parent)?;
    let tmp_path = iso_path.with_extension("iso.tmp");
    if tmp_path.exists() {
        fs::remove_file(&tmp_path)?;
    }

    let output = Command::new(&config.downloader_binary)
        .args([
            OsStr::new("--fail"),
            OsStr::new("--location"),
            OsStr::new("--show-error"),
            OsStr::new("--silent"),
            OsStr::new("--output"),
        ])
        .arg(&tmp_path)
        .arg(ISO_ASSET_URL)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SoakError::new(format!(
            "release asset download failed via {} with status {}: {}",
            config.downloader_binary.display(),
            output.status,
            stderr.trim()
        )));
    }

    fs::rename(tmp_path, iso_path)?;
    Ok(())
}

fn run_boot_iteration(
    config: &SoakConfig,
    attempt_id: &str,
    iteration_index: usize,
    iso_path: &Path,
    attempt_dir: &Path,
) -> Result<BootRecord> {
    let iteration_dir = attempt_dir.join(format!("boot-{iteration_index:02}"));
    fs::create_dir_all(&iteration_dir)?;
    let raw_serial_log_path = iteration_dir.join("serial.log");
    let qemu_stderr_path = iteration_dir.join("qemu.stderr.log");

    let mut child = Command::new(&config.qemu_binary)
        .args(qemu_args(iso_path))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| SoakError::new("failed to open QEMU stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| SoakError::new("failed to open QEMU stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| SoakError::new("failed to open QEMU stderr"))?;

    let (serial_tx, serial_rx) = mpsc::channel::<std::io::Result<Vec<u8>>>();
    let serial_path_for_thread = raw_serial_log_path.clone();
    let serial_reader = thread::spawn(move || -> std::io::Result<()> {
        let mut stdout = stdout;
        let mut file = File::create(serial_path_for_thread)?;
        let mut buffer = [0_u8; 8192];
        loop {
            let read = stdout.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            if serial_tx.send(Ok(buffer[..read].to_vec())).is_err() {
                break;
            }
        }
        file.flush()?;
        Ok(())
    });

    let stderr_path_for_thread = qemu_stderr_path.clone();
    let stderr_reader = thread::spawn(move || -> std::io::Result<()> {
        let mut stderr = stderr;
        let mut file = File::create(stderr_path_for_thread)?;
        std::io::copy(&mut stderr, &mut file)?;
        file.flush()?;
        Ok(())
    });

    let started = Instant::now();
    let timeout = Duration::from_secs(PER_BOOT_TIMEOUT_SECONDS);
    let mut serial_buffer = Vec::new();
    let mut matched_marker = None;
    let mut timeout_hit = false;
    let termination_reason: String;
    let qemu_exit_status: ExitStatus;
    let mut serial_input_events = Vec::new();
    let mut sent_escape = false;
    let mut sent_label = false;
    let mut escape_sent_at = None;

    loop {
        if let Some(status) = child.try_wait()? {
            qemu_exit_status = status;
            if matched_marker.is_some() {
                termination_reason =
                    "boot_ready_marker_matched_qemu_exited_after_marker".to_string();
            } else {
                termination_reason = "qemu_exited_before_marker".to_string();
            }
            break;
        }

        if started.elapsed() >= timeout {
            timeout_hit = true;
            termination_reason = "timeout_180s_harness_terminated_qemu".to_string();
            child.kill()?;
            qemu_exit_status = child.wait()?;
            break;
        }

        match serial_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(chunk)) => {
                serial_buffer.extend_from_slice(&chunk);
                drive_isolinux_boot_selection(
                    &mut stdin,
                    &serial_buffer,
                    started,
                    &mut sent_escape,
                    &mut sent_label,
                    &mut escape_sent_at,
                    &mut serial_input_events,
                )?;
                if matched_marker.is_none() {
                    matched_marker = find_boot_ready_marker(&serial_buffer);
                    if matched_marker.is_some() {
                        termination_reason =
                            "boot_ready_marker_matched_then_harness_terminated_qemu".to_string();
                        child.kill()?;
                        qemu_exit_status = child.wait()?;
                        break;
                    }
                }
            }
            Ok(Err(err)) => return Err(SoakError::new(format!("serial reader failed: {err}"))),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                drive_isolinux_boot_selection(
                    &mut stdin,
                    &serial_buffer,
                    started,
                    &mut sent_escape,
                    &mut sent_label,
                    &mut escape_sent_at,
                    &mut serial_input_events,
                )?;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {}
        }
    }

    drop(stdin);
    serial_reader
        .join()
        .map_err(|_| SoakError::new("serial reader thread panicked"))??;
    stderr_reader
        .join()
        .map_err(|_| SoakError::new("stderr reader thread panicked"))??;

    let raw_serial_log_sha256 = sha256_file(&raw_serial_log_path)?;
    let qemu_stderr_sha256 = sha256_file(&qemu_stderr_path)?;
    let raw_serial_log_bytes = fs::metadata(&raw_serial_log_path)?.len();
    let qemu_stderr_bytes = fs::metadata(&qemu_stderr_path)?.len();
    let qemu_exit_status = qemu_exit_status_receipt(Some(qemu_exit_status));
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let mut record = BootRecord {
        attempt_id: attempt_id.to_string(),
        iteration_index,
        clean: false,
        elapsed_seconds,
        timeout_hit,
        qemu_exit_status,
        termination_reason,
        matched_marker,
        raw_serial_log_path: path_string(&raw_serial_log_path),
        raw_serial_log_sha256,
        raw_serial_log_bytes,
        qemu_stderr_path: path_string(&qemu_stderr_path),
        qemu_stderr_sha256,
        qemu_stderr_bytes,
        serial_input_events,
    };
    record.clean = is_clean_boot(&record);
    Ok(record)
}

fn drive_isolinux_boot_selection(
    stdin: &mut impl Write,
    serial_buffer: &[u8],
    started: Instant,
    sent_escape: &mut bool,
    sent_label: &mut bool,
    escape_sent_at: &mut Option<Instant>,
    serial_input_events: &mut Vec<String>,
) -> Result<()> {
    let serial_text = String::from_utf8_lossy(serial_buffer);
    let elapsed_ms = started.elapsed().as_millis();
    let saw_bootloader = serial_text.contains("ISOLINUX")
        || serial_text.contains("SYSLINUX")
        || serial_text.contains("boot:");

    if !*sent_escape && (saw_bootloader || started.elapsed() >= Duration::from_secs(2)) {
        stdin.write_all(&[0x1b])?;
        stdin.flush()?;
        *sent_escape = true;
        *escape_sent_at = Some(Instant::now());
        serial_input_events.push(format!(
            "{}ms: wrote ESC as a standalone serial input byte to enter ISOLINUX prompt",
            elapsed_ms
        ));
    }

    if *sent_escape && !*sent_label {
        let enough_gap = escape_sent_at
            .map(|instant| instant.elapsed() >= Duration::from_millis(350))
            .unwrap_or(false);
        if serial_text.contains("boot:") || enough_gap {
            stdin.write_all(b"boot-serial\r")?;
            stdin.flush()?;
            *sent_label = true;
            serial_input_events.push(format!(
                "{}ms: wrote boot-serial carriage-return label as a second standalone serial input write",
                started.elapsed().as_millis()
            ));
        }
    }

    Ok(())
}

fn qemu_runtime(config: &SoakConfig, iso_path: &str) -> QemuRuntime {
    let iso_path = Path::new(iso_path);
    let args = qemu_args(iso_path);
    let qemu_binary = path_string(&config.qemu_binary);
    let mut command = shell_join(std::iter::once(qemu_binary.clone()).chain(args.iter().cloned()));
    command.push_str(" <serial-input-driven-by-owned-rust-harness> >serial.log");
    QemuRuntime {
        qemu_binary,
        arch: "x86_64".to_string(),
        machine_model: "qemu-system-x86_64 tcg, no disk, cdrom-only".to_string(),
        arguments: args,
        independently_runnable_command: command,
        per_boot_timeout_seconds: PER_BOOT_TIMEOUT_SECONDS,
    }
}

fn qemu_args(iso_path: &Path) -> Vec<String> {
    vec![
        "-machine".to_string(),
        "accel=tcg".to_string(),
        "-cpu".to_string(),
        "qemu64".to_string(),
        "-smp".to_string(),
        "1".to_string(),
        "-m".to_string(),
        "2048M".to_string(),
        "-display".to_string(),
        "none".to_string(),
        "-monitor".to_string(),
        "none".to_string(),
        "-serial".to_string(),
        "stdio".to_string(),
        "-no-reboot".to_string(),
        "-boot".to_string(),
        "d".to_string(),
        "-cdrom".to_string(),
        path_string(iso_path),
    ]
}

fn vm_isolation() -> VmIsolation {
    VmIsolation {
        method: "Each iteration spawns a new qemu-system-x86_64 process from the verified ISO, with only a read-only CD-ROM and no writable disk, no snapshot device, and a new serial log file.".to_string(),
        no_disk: true,
        no_snapshot: true,
        fresh_process_per_iteration: true,
        serial_log_per_iteration: true,
        state_carryover: "none: ISO bytes are reused read-only after one sha256 verification; VM process, memory, devices, and serial capture file are recreated per iteration".to_string(),
    }
}

fn match_systemd_startup_finished(line: &str) -> Option<String> {
    let start = line.find("systemd[1]:")?;
    let after_prefix = start + "systemd[1]:".len();
    let whitespace_len = line[after_prefix..]
        .chars()
        .take_while(|ch| ch.is_whitespace())
        .map(char::len_utf8)
        .sum::<usize>();
    let startup_start = after_prefix + whitespace_len;
    if line[startup_start..].starts_with("Startup finished") {
        let end = startup_start + "Startup finished".len();
        return Some(line[start..end].to_string());
    }
    None
}

fn match_systemd_target(line: &str) -> Option<String> {
    let start = line.find("Reached target ")?;
    let tail = &line[start..];
    for target in ["Multi-User", "Basic System", "Login Prompts"] {
        if let Some(target_start) = tail.find(target) {
            let end = start + target_start + target.len();
            return Some(line[start..end].to_string());
        }
    }
    None
}

fn match_login_prompt(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let login_start = lower.rfind("login:")?;
    if lower[login_start + "login:".len()..]
        .chars()
        .all(char::is_whitespace)
    {
        return Some(line[login_start..].to_string());
    }
    None
}

fn match_shell_prompt(line: &str) -> Option<String> {
    let mut chars = line.char_indices().rev();
    let (_, last) = chars.next()?;
    if !last.is_whitespace() {
        return None;
    }
    let (prompt_index, prompt) = chars.next()?;
    if prompt == '#' || prompt == '$' {
        return Some(line[prompt_index..].to_string());
    }
    None
}

fn qemu_exit_status_receipt(status: Option<ExitStatus>) -> QemuExitStatusReceipt {
    match status {
        Some(status) => QemuExitStatusReceipt {
            observed: true,
            code: status.code(),
            signal: exit_signal(&status),
            description: status.to_string(),
        },
        None => QemuExitStatusReceipt {
            observed: false,
            code: None,
            signal: None,
            description: "not observed".to_string(),
        },
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

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(value)?;
    ensure_receipt_json_lines_are_bounded(path, &bytes)?;
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn ensure_receipt_json_lines_are_bounded(path: &Path, bytes: &[u8]) -> Result<()> {
    for (line_index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if line.len() > MAX_RECEIPT_JSON_LINE_BYTES {
            return Err(SoakError::new(format!(
                "receipt JSON line {} in {} is {} bytes, exceeding the {} byte bounded-output limit",
                line_index + 1,
                path.display(),
                line.len(),
                MAX_RECEIPT_JSON_LINE_BYTES
            )));
        }
    }
    Ok(())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn shell_join(items: impl IntoIterator<Item = String>) -> String {
    items
        .into_iter()
        .map(|item| shell_quote(&item))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(item: &str) -> String {
    if item
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '-' | '_' | '=' | ':'))
    {
        item.to_string()
    } else {
        format!("'{}'", item.replace('\'', "'\\''"))
    }
}

fn unix_time_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_record(attempt_id: &str, iteration_index: usize) -> BootRecord {
        BootRecord {
            attempt_id: attempt_id.to_string(),
            iteration_index,
            clean: true,
            elapsed_seconds: 1.0,
            timeout_hit: false,
            qemu_exit_status: QemuExitStatusReceipt {
                observed: true,
                code: None,
                signal: Some(9),
                description: "signal: 9 (SIGKILL)".to_string(),
            },
            termination_reason: "boot_ready_marker_matched_then_harness_terminated_qemu"
                .to_string(),
            matched_marker: Some(MarkerMatch {
                marker_name: "nixos".to_string(),
                regex: r"/Welcome to NixOS/".to_string(),
                matched_string: "Welcome to NixOS".to_string(),
            }),
            raw_serial_log_path: format!("boot-{iteration_index:02}/serial.log"),
            raw_serial_log_sha256: "0".repeat(64),
            raw_serial_log_bytes: 1,
            qemu_stderr_path: format!("boot-{iteration_index:02}/qemu.stderr.log"),
            qemu_stderr_sha256: "0".repeat(64),
            qemu_stderr_bytes: 0,
            serial_input_events: vec![
                "1ms: wrote ESC as a standalone serial input byte to enter ISOLINUX prompt"
                    .to_string(),
                "2ms: wrote boot-serial carriage-return label as a second standalone serial input write"
                    .to_string(),
            ],
        }
    }

    fn timeout_record(attempt_id: &str, iteration_index: usize) -> BootRecord {
        let mut record = clean_record(attempt_id, iteration_index);
        record.clean = false;
        record.timeout_hit = true;
        record.qemu_exit_status = QemuExitStatusReceipt {
            observed: true,
            code: None,
            signal: Some(9),
            description: "signal: 9 (SIGKILL)".to_string(),
        };
        record.termination_reason = "timeout_180s_harness_terminated_qemu".to_string();
        record.matched_marker = None;
        record.raw_serial_log_bytes = 0;
        record
    }

    fn qemu_exited_before_marker_record(attempt_id: &str, iteration_index: usize) -> BootRecord {
        let mut record = clean_record(attempt_id, iteration_index);
        record.clean = false;
        record.timeout_hit = false;
        record.qemu_exit_status = QemuExitStatusReceipt {
            observed: true,
            code: Some(1),
            signal: None,
            description: "exit status: 1".to_string(),
        };
        record.termination_reason = "qemu_exited_before_marker".to_string();
        record.matched_marker = None;
        record.raw_serial_log_bytes = 0;
        record
    }

    fn test_config(test_name: &str) -> SoakConfig {
        let output_base_dir = std::env::temp_dir().join(format!(
            "asterinas-boundary-{test_name}-{}-{}",
            unix_time_seconds(),
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&output_base_dir);
        SoakConfig {
            qemu_binary: PathBuf::from("/opt/homebrew/bin/qemu-system-x86_64"),
            downloader_binary: PathBuf::from("curl"),
            output_base_dir,
            run_id: "test-run".to_string(),
        }
    }

    fn test_iso_receipt() -> ArtifactReceipt {
        ArtifactReceipt {
            asset_name: ISO_ASSET_NAME.to_string(),
            asset_url: ISO_ASSET_URL.to_string(),
            expected_sha256: ISO_SHA256.to_string(),
            actual_sha256: ISO_SHA256.to_string(),
            byte_size: 1_378_910_208,
            local_path: "target/asterinas-soak/cache/asterinas-nixos-0.17.2-x86_64.iso".to_string(),
            verification_status: "sha256_verified".to_string(),
            fetch_action: "reused_existing_verified".to_string(),
            verified_at_unix: 1,
        }
    }

    fn test_qemu_runtime() -> QemuRuntime {
        QemuRuntime {
            qemu_binary: "/opt/homebrew/bin/qemu-system-x86_64".to_string(),
            arch: "x86_64".to_string(),
            machine_model: "qemu-system-x86_64 tcg, no disk, cdrom-only".to_string(),
            arguments: vec![
                "-machine".to_string(),
                "accel=tcg".to_string(),
                "-serial".to_string(),
                "stdio".to_string(),
                "-cdrom".to_string(),
                "target/asterinas-soak/cache/asterinas-nixos-0.17.2-x86_64.iso".to_string(),
            ],
            independently_runnable_command: "/opt/homebrew/bin/qemu-system-x86_64 -machine accel=tcg -serial stdio -cdrom target/asterinas-soak/cache/asterinas-nixos-0.17.2-x86_64.iso <serial-input-driven-by-owned-rust-harness> >serial.log".to_string(),
            per_boot_timeout_seconds: PER_BOOT_TIMEOUT_SECONDS,
        }
    }

    #[test]
    fn pin_manifest_matches_seed_contract() {
        let manifest = parsed_pin_manifest().expect("pin manifest parses");
        assert_eq!(manifest.asset_name, ISO_ASSET_NAME);
        assert_eq!(manifest.asset_sha256, ISO_SHA256);
        assert_eq!(manifest.asset_url, ISO_ASSET_URL);
        assert_eq!(manifest.release_tag, RELEASE_TAG);
        assert_eq!(manifest.release_commit, RELEASE_COMMIT);
        assert_eq!(manifest.upstream_repository, UPSTREAM_REPOSITORY);
        assert!(manifest.license_boundary.contains("unmodified"));
    }

    #[test]
    fn closed_marker_set_accepts_nixos_literal() {
        let marker = find_boot_ready_marker(b"stage-2\nWelcome to NixOS\n").unwrap();
        assert_eq!(marker.marker_name, "nixos");
        assert_eq!(marker.matched_string, "Welcome to NixOS");
    }

    #[test]
    fn closed_marker_set_accepts_systemd_startup() {
        let marker =
            find_boot_ready_marker(b"[    3.1] systemd[1]:   Startup finished in 1.234s.\r\n")
                .unwrap();
        assert_eq!(marker.marker_name, "systemd_startup_finished");
        assert_eq!(marker.matched_string, "systemd[1]:   Startup finished");
    }

    #[test]
    fn closed_marker_set_accepts_reached_target() {
        let marker = find_boot_ready_marker(b"[ OK ] Reached target Multi-User System.\n").unwrap();
        assert_eq!(marker.marker_name, "systemd_target_reached");
        assert_eq!(marker.matched_string, "Reached target Multi-User");
    }

    #[test]
    fn closed_marker_set_accepts_login_prompt_at_line_end() {
        let marker = find_boot_ready_marker(b"localhost login: ").unwrap();
        assert_eq!(marker.marker_name, "login");
        assert_eq!(marker.matched_string, "login: ");
    }

    #[test]
    fn closed_marker_set_accepts_shell_prompt_at_line_end() {
        let marker = find_boot_ready_marker(b"[root@nixos:~]# ").unwrap();
        assert_eq!(marker.marker_name, "shell");
        assert_eq!(marker.matched_string, "# ");
    }

    #[test]
    fn closed_marker_set_rejects_fallback_kernel_text() {
        assert!(
            find_boot_ready_marker(
                b"OSTD initialized. Preparing components.\n[kernel] rootfs is ready\n"
            )
            .is_none()
        );
    }

    #[test]
    fn clean_boot_requires_marker_without_timeout() {
        let mut record = clean_record("attempt-001", 1);
        assert!(is_clean_boot(&record));
        record.timeout_hit = true;
        assert!(!is_clean_boot(&record));
        record.timeout_hit = false;
        record.matched_marker = None;
        assert!(!is_clean_boot(&record));
        record.matched_marker = Some(MarkerMatch {
            marker_name: "nixos".to_string(),
            regex: r"/Welcome to NixOS/".to_string(),
            matched_string: "Welcome to NixOS".to_string(),
        });
        record.termination_reason = "qemu_exited_before_marker".to_string();
        assert!(!is_clean_boot(&record));
    }

    #[test]
    fn attempt_pass_requires_ten_ordered_clean_boots_in_one_attempt() {
        let records = (1..=ITERATION_COUNT)
            .map(|iteration| clean_record("attempt-001", iteration))
            .collect::<Vec<_>>();
        assert!(attempt_is_pass(&records));

        let nine_records = records[..ITERATION_COUNT - 1].to_vec();
        assert!(!attempt_is_pass(&nine_records));

        let mut split_attempt_records = records.clone();
        split_attempt_records[9].attempt_id = "attempt-002".to_string();
        assert!(!attempt_is_pass(&split_attempt_records));
        let attempt_one_only = split_attempt_records
            .iter()
            .filter(|record| record.attempt_id == "attempt-001")
            .cloned()
            .collect::<Vec<_>>();
        assert!(!attempt_is_pass(&attempt_one_only));

        let mut out_of_order_records = records.clone();
        out_of_order_records.swap(8, 9);
        assert!(!attempt_is_pass(&out_of_order_records));

        let mut inconsistent_clean_record = records.clone();
        inconsistent_clean_record[0].clean = false;
        assert!(!attempt_is_pass(&inconsistent_clean_record));
    }

    #[test]
    fn failed_attempt_receipt_is_preserved_and_later_clean_attempt_yields_soak_pass() {
        let config = test_config("retry-after-timeout");
        let mut calls = Vec::new();

        let output = run_soak_with_boot_runner(
            &config,
            test_iso_receipt(),
            |_, attempt_id, iteration_index, _, _| {
                calls.push((attempt_id.to_string(), iteration_index));
                if attempt_id == "attempt-001" && iteration_index == 4 {
                    Ok(timeout_record(attempt_id, iteration_index))
                } else {
                    Ok(clean_record(attempt_id, iteration_index))
                }
            },
        )
        .unwrap();

        assert_eq!(output.verdict, "pass");
        assert_eq!(output.attempts.len(), 2);
        assert_eq!(calls.len(), ITERATION_COUNT * 2);
        assert!(
            calls
                .iter()
                .all(|(attempt_id, _)| attempt_id != "attempt-003")
        );

        assert_eq!(output.attempts[0].attempt_id, "attempt-001");
        assert_eq!(output.attempts[0].verdict, "fail");
        assert_eq!(output.attempts[0].clean_boots, ITERATION_COUNT - 1);
        assert_eq!(output.attempts[1].attempt_id, "attempt-002");
        assert_eq!(output.attempts[1].verdict, "pass");
        assert_eq!(output.attempts[1].clean_boots, ITERATION_COUNT);

        let failed_attempt_receipt: SoakAttemptReceipt =
            serde_json::from_slice(&fs::read(Path::new(&output.attempts[0].receipt_path)).unwrap())
                .unwrap();
        let failed_boot = &failed_attempt_receipt.boot_records[3];
        assert_eq!(failed_attempt_receipt.verdict, "fail");
        assert_eq!(failed_attempt_receipt.boot_records.len(), ITERATION_COUNT);
        assert_eq!(failed_boot.attempt_id, "attempt-001");
        assert_eq!(failed_boot.iteration_index, 4);
        assert!(!failed_boot.clean);
        assert!(failed_boot.timeout_hit);
        assert_eq!(
            failed_boot.termination_reason,
            "timeout_180s_harness_terminated_qemu"
        );
        assert!(failed_boot.matched_marker.is_none());

        let aggregate: AggregateReceipt =
            serde_json::from_slice(&fs::read(&output.aggregate_receipt_path).unwrap()).unwrap();
        assert_eq!(aggregate.overall_verdict, "pass");
        assert_eq!(aggregate.passing_attempt_id.as_deref(), Some("attempt-002"));
        assert!(aggregate.gap_register_entry.is_none());
        assert_eq!(aggregate.soak_attempts.len(), 2);
        assert_eq!(aggregate.soak_attempts[0].verdict, "fail");
        assert_eq!(aggregate.soak_attempts[1].verdict, "pass");

        let _ = fs::remove_dir_all(&config.output_base_dir);
    }

    #[test]
    fn exhausted_failed_attempts_emit_honest_fail_gap_after_attempt_budget() {
        let config = test_config("honest-fail-after-budget");

        let output = run_soak_with_boot_runner(
            &config,
            test_iso_receipt(),
            |_, attempt_id, iteration_index, _, _| {
                if iteration_index == 1 {
                    if attempt_id == "attempt-001" {
                        Ok(timeout_record(attempt_id, iteration_index))
                    } else {
                        Ok(qemu_exited_before_marker_record(
                            attempt_id,
                            iteration_index,
                        ))
                    }
                } else {
                    Ok(clean_record(attempt_id, iteration_index))
                }
            },
        )
        .unwrap();

        assert_eq!(output.verdict, "fail");
        assert_eq!(output.attempts.len(), MAX_SOAK_ATTEMPTS);

        for attempt in &output.attempts {
            assert_eq!(attempt.verdict, "fail");
            assert_eq!(attempt.clean_boots, ITERATION_COUNT - 1);

            let receipt: SoakAttemptReceipt =
                serde_json::from_slice(&fs::read(Path::new(&attempt.receipt_path)).unwrap())
                    .unwrap();
            let failed_boot = &receipt.boot_records[0];
            assert_eq!(receipt.verdict, "fail");
            assert_eq!(receipt.boot_records.len(), ITERATION_COUNT);
            assert!(!failed_boot.clean);
            assert!(failed_boot.matched_marker.is_none());
            assert!(
                failed_boot.timeout_hit
                    || failed_boot.termination_reason == "qemu_exited_before_marker"
            );
        }

        let aggregate: AggregateReceipt =
            serde_json::from_slice(&fs::read(&output.aggregate_receipt_path).unwrap()).unwrap();
        assert_eq!(aggregate.overall_verdict, "fail");
        assert!(aggregate.passing_attempt_id.is_none());
        assert_eq!(aggregate.soak_attempts.len(), MAX_SOAK_ATTEMPTS);
        assert!(
            aggregate
                .soak_attempts
                .iter()
                .all(|attempt| attempt.verdict == "fail")
        );

        let gap = aggregate
            .gap_register_entry
            .as_ref()
            .expect("exhausted failed attempts produce gap-register escalation");
        assert_eq!(gap.fate, "HonestFail");
        assert!(
            gap.blocker
                .contains("No soak attempt reached ten consecutive")
        );
        assert_eq!(gap.honest_fail_reference, path_string(&config.run_dir()));

        let _ = fs::remove_dir_all(&config.output_base_dir);
    }

    #[test]
    fn aggregate_receipt_summarizes_single_passing_attempt_without_raw_artifacts() {
        let boot_records = (1..=ITERATION_COUNT)
            .map(|iteration| clean_record("attempt-001", iteration))
            .collect::<Vec<_>>();
        let attempt = AttemptReference {
            attempt_id: "attempt-001".to_string(),
            verdict: "pass".to_string(),
            clean_boots: ITERATION_COUNT,
            receipt_path: "target/asterinas-soak/runs/test/attempt-001/attempt-receipt.json"
                .to_string(),
            receipt_sha256: "1".repeat(64),
            boot_records,
        };
        let aggregate = AggregateReceipt {
            parent_seed_id: PARENT_SEED_ID.to_string(),
            prior_shard_id: PRIOR_SHARD_ID.to_string(),
            overall_verdict: "pass".to_string(),
            passing_attempt_id: Some("attempt-001".to_string()),
            iso_asset: test_iso_receipt(),
            iteration_count: ITERATION_COUNT,
            per_boot_timeout_seconds: PER_BOOT_TIMEOUT_SECONDS,
            max_soak_attempts: MAX_SOAK_ATTEMPTS,
            qemu_runtime: test_qemu_runtime(),
            vm_isolation: vm_isolation(),
            boot_ready_markers: boot_ready_marker_regexes(),
            soak_attempts: vec![attempt],
            gap_register_entry: None,
        };

        assert_eq!(aggregate.overall_verdict, "pass");
        assert_eq!(aggregate.passing_attempt_id.as_deref(), Some("attempt-001"));
        assert_eq!(aggregate.soak_attempts.len(), 1);

        let passing_attempts = aggregate
            .soak_attempts
            .iter()
            .filter(|attempt| attempt.verdict == "pass")
            .collect::<Vec<_>>();
        assert_eq!(passing_attempts.len(), 1);
        assert_eq!(passing_attempts[0].boot_records.len(), ITERATION_COUNT);

        for (offset, record) in passing_attempts[0].boot_records.iter().enumerate() {
            assert_eq!(record.iteration_index, offset + 1);
            assert!(record.clean);
            assert!(record.elapsed_seconds > 0.0);
            assert!(record.matched_marker.is_some());
            assert!(record.raw_serial_log_path.ends_with("/serial.log"));
            assert_eq!(record.raw_serial_log_sha256.len(), 64);
            assert!(record.raw_serial_log_bytes > 0);
        }

        let aggregate_json = serde_json::to_vec_pretty(&aggregate).unwrap();
        ensure_receipt_json_lines_are_bounded(Path::new("aggregate-receipt.json"), &aggregate_json)
            .unwrap();
        let aggregate_text = String::from_utf8(aggregate_json).unwrap();
        assert!(!aggregate_text.contains("raw_serial_log_contents"));
        assert!(!aggregate_text.contains("serial_console_output"));
        assert!(!aggregate_text.contains("Welcome to NixOS\nWelcome to NixOS"));
    }

    #[test]
    fn receipt_writer_rejects_oversized_single_line_payloads() {
        #[derive(Serialize)]
        struct OversizedReceipt {
            payload: String,
        }

        let path = std::env::temp_dir().join(format!(
            "asterinas-boundary-oversized-receipt-test-{}-{}",
            unix_time_seconds(),
            std::process::id()
        ));
        let receipt = OversizedReceipt {
            payload: "x".repeat(MAX_RECEIPT_JSON_LINE_BYTES + 1),
        };

        let error = write_json_file(&path, &receipt).unwrap_err();
        let _ = fs::remove_file(&path);
        assert!(error.to_string().contains("bounded-output limit"));
    }
    #[test]
    fn sha256_file_reads_on_disk_bytes() {
        let path = std::env::temp_dir().join(format!(
            "asterinas-boundary-sha-test-{}-{}",
            unix_time_seconds(),
            std::process::id()
        ));
        fs::write(&path, b"abc").unwrap();
        let digest = sha256_file(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
