#![forbid(unsafe_code)]
//! AC1 harness binary: fetch the pinned upstream Asterinas v0.17.2 release ISO from its
//! published GitHub release, verify its sha256 against the pin, store it at a referenced local
//! path, and write a digest-self-consistent fetch-verify receipt.
//!
//! Owned-Rust / no-shell-no-python doctrine (ADR-0523): the ONLY non-Rust surface is the
//! release-asset download performed via the typed reqwest HTTP client. The ISO is streamed to
//! disk with `std::io::copy` (never buffered whole, never inlined into any output line); the
//! recorded digest is recomputed from the on-disk bytes at receipt-write time.
//!
//! Usage: `fetch-verify [ISO_DEST] [RECEIPT_DEST]` (both default to repo-relative paths).

use kernel_asterinas_boundary as pin;
use kernel_asterinas_real_boot::{build_fetch_verify_receipt, now_unix, sha256_file};
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

const DEFAULT_ISO_DEST: &str = "kernel/target/artifacts/asterinas-nixos-0.17.2-x86_64.iso";
const DEFAULT_RECEIPT_DEST: &str =
    "kernel/harness/asterinas-real-boot/receipts/fetch-verify-v0.17.2.json";

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!(
                "[fetch-verify] DIGEST/SIZE MISMATCH \u{2014} receipt written with verified=false (honest failure)"
            );
            ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("[fetch-verify] ERROR: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<bool, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let iso_dest = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_ISO_DEST.to_string()));
    let receipt_dest = PathBuf::from(
        args.next()
            .unwrap_or_else(|| DEFAULT_RECEIPT_DEST.to_string()),
    );

    if let Some(parent) = iso_dest.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = receipt_dest.parent() {
        fs::create_dir_all(parent)?;
    }

    eprintln!(
        "[fetch-verify] source={} asset={} expected_sha256={} expected_bytes={}",
        pin::BOOT_ISO_DOWNLOAD_URL,
        pin::BOOT_ISO_ASSET,
        pin::BOOT_ISO_SHA256,
        pin::BOOT_ISO_BYTE_SIZE
    );
    eprintln!(
        "[fetch-verify] streaming download -> {}",
        iso_dest.display()
    );

    let downloaded = download_to_file(pin::BOOT_ISO_DOWNLOAD_URL, &iso_dest)?;
    eprintln!("[fetch-verify] wrote {downloaded} bytes to disk");

    // Recompute digest + size from the ON-DISK bytes at receipt-write time so the recorded
    // digest is self-consistent with the referenced file.
    let (actual_sha256, actual_size) = sha256_file(&iso_dest)?;
    eprintln!("[fetch-verify] on-disk sha256={actual_sha256} bytes={actual_size}");

    let receipt = build_fetch_verify_receipt(
        pin::BOOT_ISO_DOWNLOAD_URL,
        pin::BOOT_ISO_ASSET,
        pin::RELEASE_TAG,
        pin::BOOT_ISO_SHA256,
        &actual_sha256,
        pin::BOOT_ISO_BYTE_SIZE,
        actual_size,
        &iso_dest.to_string_lossy(),
        now_unix(),
    );
    let verified = receipt["verified"].as_bool().unwrap_or(false);

    let mut json = serde_json::to_string_pretty(&receipt)?;
    json.push('\n');
    fs::write(&receipt_dest, json)?;
    eprintln!(
        "[fetch-verify] receipt -> {} verified={verified}",
        receipt_dest.display()
    );
    Ok(verified)
}

/// Stream the release asset to `dest`. reqwest follows the GitHub 302 redirect to the CDN and
/// its blocking `Response` implements `Read`, so `io::copy` streams the body straight to the
/// file without materializing it in memory.
///
/// The download is the single external surface most exposed to transient network hiccups, so it
/// retries a few times with linear backoff before surfacing the final error. The recorded digest
/// is still recomputed from the on-disk bytes afterward, so a truncated/partial retry can never
/// produce a false `verified` — a corrupt fetch simply fails the digest check.
fn download_to_file(url: &str, dest: &Path) -> Result<u64, Box<dyn Error>> {
    const MAX_ATTEMPTS: u32 = 4;
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(60))
        .timeout(Duration::from_secs(1800))
        .build()?;

    let mut last_err: Option<Box<dyn Error>> = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match try_download(&client, url, dest) {
            Ok(n) => return Ok(n),
            Err(e) => {
                eprintln!("[fetch-verify] download attempt {attempt}/{MAX_ATTEMPTS} failed: {e}");
                last_err = Some(e);
                if attempt < MAX_ATTEMPTS {
                    // Linear backoff (5s, 10s, 15s): enough to ride out a transient hiccup
                    // without materially extending a genuinely-down run.
                    std::thread::sleep(Duration::from_secs(5 * attempt as u64));
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| "download failed with no recorded error".into()))
}

/// One download attempt: stream the body to `dest` and fsync. A partial write leaves a
/// truncated file that the caller's digest recomputation will reject — never a false pass.
fn try_download(
    client: &reqwest::blocking::Client,
    url: &str,
    dest: &Path,
) -> Result<u64, Box<dyn Error>> {
    let mut resp = client.get(url).send()?.error_for_status()?;
    let mut out = fs::File::create(dest)?;
    let n = io::copy(&mut resp, &mut out)?;
    out.sync_all()?;
    Ok(n)
}
