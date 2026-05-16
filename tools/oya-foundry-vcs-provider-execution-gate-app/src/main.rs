//! Oya VCS provider-execution-gate dev-CLI (Wave 3 replacement for
//! `scripts/check-oya-vcs-provider-execution.sh`; audit row B-4).
//!
//! Composition root for the `oya-vcs-provider-execution` required-check.
//! Reads workspace + git + GitHub-Actions metadata, invokes canonical
//! `trivy` as a subprocess (fs vuln scan, infra config scan, sarif
//! emission), reads + validates the Argo desired-state Application
//! manifest via the gate kernel, and emits the deterministic
//! provider-execution evidence record when `--emit-evidence <path>` is
//! given.
//!
//! Trivy stays as the canonical external tool — invoked via
//! [`std::process::Command`], not reimplemented in Rust.
//!
//! # Naming justification
//!
//! - `oya-foundry-vcs-provider-execution-gate-app` —
//!   v4 BNF `oya-<product:foundry>-<topic:vcs-provider-execution-gate>-<layer:app>`;
//!   13-value layer-enum suffix `app` (composition-root binary tool surface
//!   per ADR-0105 §"Amendment 2026-05-15 — `tools/` canonical-suffix binding").

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use oya_foundry_vcs_provider_execution_gate_kernel::{
    EvidenceContext, Mode, build_provider_execution_evidence, validate_argo_application,
};
use serde_json::Value;

const ARGO_MANIFEST: &str = "infra/kyverno/oya-vcs-admission/application.json";
const TRIVY_OUT_DIR: &str = "target/oya-vcs-provider-execution";
const TRIVY_SARIF: &str = "target/oya-vcs-provider-execution/trivy.sarif";

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!(
                "oya-vcs provider execution validation passed: ci/github-actions/trivy/argo-gitops"
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("check-oya-vcs-provider-execution: {error}");
            error.exit_code()
        }
    }
}

fn run() -> Result<(), GateError> {
    let options = Options::parse(env::args_os().skip(1))?;
    let root = resolve_root()?;

    require_tool("trivy")?;

    let workspace_ref = git_output(&root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|| "unknown".to_string());
    let head_sha =
        git_output(&root, &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());

    let (run_url, workflow_name) = detect_runner_context(&root);

    let out_dir = root.join(TRIVY_OUT_DIR);
    fs::create_dir_all(&out_dir).map_err(|e| GateError::Io {
        detail: format!("mkdir {}: {e}", out_dir.display()),
        exit: 1,
    })?;

    let trivy_common: &[&str] = &[
        "--skip-dirs",
        "target",
        "--skip-dirs",
        ".git",
        "--skip-dirs",
        ".grit",
        "--skip-dirs",
        ".omc",
        "--skip-dirs",
        ".omx",
    ];

    let mut fs_vuln_args: Vec<&str> = vec![
        "fs",
        "--severity",
        "HIGH,CRITICAL",
        "--exit-code",
        "1",
        "--scanners",
        "vuln",
    ];
    fs_vuln_args.extend_from_slice(trivy_common);
    fs_vuln_args.push(".");
    run_command(&root, "trivy", &fs_vuln_args, "trivy fs vuln scan")?;

    run_command(
        &root,
        "trivy",
        &[
            "config",
            "--severity",
            "HIGH,CRITICAL",
            "--exit-code",
            "1",
            "infra/",
        ],
        "trivy config scan",
    )?;

    let sarif_path = root.join(TRIVY_SARIF);
    let sarif_str = sarif_path.to_str().ok_or_else(|| GateError::Io {
        detail: format!("non-utf8 sarif path: {}", sarif_path.display()),
        exit: 1,
    })?;
    let mut sarif_args: Vec<&str> = vec![
        "fs",
        "--scanners",
        "vuln,secret,license",
        "--format",
        "sarif",
        "--output",
        sarif_str,
    ];
    sarif_args.extend_from_slice(trivy_common);
    sarif_args.push(".");
    run_command(&root, "trivy", &sarif_args, "trivy sarif emission")?;

    if fs::metadata(&sarif_path)
        .map(|m| m.len() == 0)
        .unwrap_or(true)
    {
        return Err(GateError::Io {
            detail: format!("trivy sarif missing or empty: {}", sarif_path.display()),
            exit: 1,
        });
    }

    let manifest_path = root.join(ARGO_MANIFEST);
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|e| GateError::Io {
        detail: format!("read {}: {e}", manifest_path.display()),
        exit: 1,
    })?;
    let manifest_value: Value =
        serde_json::from_str(&manifest_text).map_err(|e| GateError::Io {
            detail: format!("argo application manifest is not valid JSON: {e}"),
            exit: 1,
        })?;
    let argo_violations = validate_argo_application(&manifest_value);
    if !argo_violations.is_empty() {
        return Err(GateError::Argo(argo_violations));
    }
    println!("argo gitops desired-state validation passed: {ARGO_MANIFEST}");

    if let Some(target) = &options.emit_evidence {
        let trivy_digest =
            digest_file(&sarif_path).map_err(|detail| GateError::Io { detail, exit: 1 })?;
        let manifest_digest =
            digest_file(&manifest_path).map_err(|detail| GateError::Io { detail, exit: 1 })?;
        let created_at = iso_utc_now().map_err(|detail| GateError::Io { detail, exit: 1 })?;
        let context = EvidenceContext {
            workspace_ref: &workspace_ref,
            head_sha: &head_sha,
            run_url: &run_url,
            workflow_name: &workflow_name,
            mode: options.mode,
            trivy_sarif_path: TRIVY_SARIF,
            trivy_sarif_digest: &trivy_digest,
            argo_manifest_path: ARGO_MANIFEST,
            argo_manifest_digest: &manifest_digest,
            created_at_iso: &created_at,
        };
        let evidence = build_provider_execution_evidence(&context);
        let resolved_target = if target.is_absolute() {
            target.clone()
        } else {
            root.join(target)
        };
        if let Some(parent) = resolved_target.parent() {
            fs::create_dir_all(parent).map_err(|e| GateError::Io {
                detail: format!("mkdir {}: {e}", parent.display()),
                exit: 1,
            })?;
        }
        let mut serialized =
            serde_json::to_string_pretty(&evidence).map_err(|e| GateError::Io {
                detail: format!("serialize evidence: {e}"),
                exit: 1,
            })?;
        serialized.push('\n');
        fs::write(&resolved_target, serialized).map_err(|e| GateError::Io {
            detail: format!("write {}: {e}", resolved_target.display()),
            exit: 1,
        })?;
        println!("wrote provider execution evidence: {}", target.display());
    }

    Ok(())
}

#[derive(Debug)]
struct Options {
    mode: Mode,
    emit_evidence: Option<PathBuf>,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, GateError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut mode = Mode::Check;
        let mut emit_evidence: Option<PathBuf> = None;
        let mut iter = args.into_iter();
        while let Some(raw) = iter.next() {
            let arg = raw.to_string_lossy().into_owned();
            match arg.as_str() {
                "--emit-evidence" => {
                    let value = iter.next().ok_or_else(|| GateError::Usage {
                        detail: "--emit-evidence requires a path".to_string(),
                    })?;
                    emit_evidence = Some(PathBuf::from(value));
                }
                "--mode" => {
                    let value = iter.next().ok_or_else(|| GateError::Usage {
                        detail: "--mode requires a value".to_string(),
                    })?;
                    let value = value.to_string_lossy().into_owned();
                    mode = match value.as_str() {
                        "check" => Mode::Check,
                        "ci" => Mode::Ci,
                        other => {
                            return Err(GateError::Usage {
                                detail: format!("invalid mode: {other}"),
                            });
                        }
                    };
                }
                "--help" | "-h" => return Err(GateError::Usage { detail: usage() }),
                other => {
                    return Err(GateError::Usage {
                        detail: format!("unexpected argument {other}\n{}", usage()),
                    });
                }
            }
        }
        Ok(Self {
            mode,
            emit_evidence,
        })
    }
}

fn usage() -> String {
    "usage: oya-foundry-vcs-provider-execution-gate-app [--mode check|ci] [--emit-evidence <path>]"
        .to_string()
}

fn resolve_root() -> Result<PathBuf, GateError> {
    if let Some(top) = git_output(Path::new("."), &["rev-parse", "--show-toplevel"]) {
        return Ok(PathBuf::from(top));
    }
    env::current_dir().map_err(|e| GateError::Io {
        detail: format!("cwd: {e}"),
        exit: 1,
    })
}

fn require_tool(tool: &str) -> Result<(), GateError> {
    let path_var = env::var_os("PATH").unwrap_or_default();
    for entry in env::split_paths(&path_var) {
        let candidate = entry.join(tool);
        if candidate.is_file() {
            return Ok(());
        }
        let with_exe = candidate.with_extension("exe");
        if with_exe.is_file() {
            return Ok(());
        }
    }
    Err(GateError::MissingTool {
        tool: tool.to_string(),
    })
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn detect_runner_context(root: &Path) -> (String, String) {
    if env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        let workflow_name =
            env::var("GITHUB_WORKFLOW").unwrap_or_else(|_| "github-actions".to_string());
        let server =
            env::var("GITHUB_SERVER_URL").unwrap_or_else(|_| "https://github.com".to_string());
        let repository =
            env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| "jason931225/oyatie".to_string());
        let run_id = env::var("GITHUB_RUN_ID").unwrap_or_else(|_| "unknown".to_string());
        let run_url = format!("{server}/{repository}/actions/runs/{run_id}");
        return (run_url, workflow_name);
    }
    if which("gh")
        && let Some(url) = pr_view_url(root)
    {
        return (url, "local-gh-pr3-visibility".to_string());
    }
    ("local".to_string(), "local-provider-proof".to_string())
}

fn which(tool: &str) -> bool {
    require_tool(tool).is_ok()
}

fn pr_view_url(root: &Path) -> Option<String> {
    let output = Command::new("gh")
        .args(["pr", "view", "3", "--json", "url", "--jq", ".url"])
        .current_dir(root)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn run_command(root: &Path, program: &str, args: &[&str], label: &str) -> Result<(), GateError> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| GateError::Io {
            detail: format!("spawning {label}: {e}"),
            exit: 1,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(GateError::CommandFailed {
            label: label.to_string(),
            status: status.code(),
        })
    }
}

fn iso_utc_now() -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock before unix epoch: {e}"))?;
    let secs = now.as_secs() as i64;
    Ok(format_iso(secs))
}

/// Formats a unix timestamp as `YYYY-MM-DDTHH:MM:SSZ` (UTC). Replaces the
/// shell `python3 -c datetime.now(UTC).isoformat()` call. Algorithm
/// from "Date Algorithms" (Howard Hinnant) — civil date from days since
/// 1970-01-01.
fn format_iso(seconds_since_epoch: i64) -> String {
    let days = seconds_since_epoch.div_euclid(86_400);
    let seconds_of_day = seconds_since_epoch.rem_euclid(86_400);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Hinnant's `civil_from_days`. Returns (year, month, day) for the given
/// days since 1970-01-01 (UTC).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

fn digest_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(format!("sha256:{}", hex_sha256(&bytes)))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

// --- SHA-256 (FIPS 180-4) -------------------------------------------------
//
// Stdlib-only implementation. The workspace ships no sha2 crate and the
// CLAUDE.md doctrine forbids reaching for shell/python. SHA-256 is a
// fixed-spec, deterministic primitive whose Rust port is well under
// 100 LOC, so vendoring it here keeps the dependency surface flat.

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(message: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (message.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(message.len() + 72);
    padded.extend_from_slice(message);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (index, word) in chunk.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..64 {
            let s0 = w[index - 15].rotate_right(7)
                ^ w[index - 15].rotate_right(18)
                ^ (w[index - 15] >> 3);
            let s1 = w[index - 2].rotate_right(17)
                ^ w[index - 2].rotate_right(19)
                ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[index])
                .wrapping_add(w[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (index, word) in h.iter().enumerate() {
        out[index * 4..(index + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[derive(Debug)]
enum GateError {
    Io { detail: String, exit: i32 },
    Usage { detail: String },
    MissingTool { tool: String },
    Argo(Vec<oya_foundry_vcs_provider_execution_gate_kernel::ArgoViolation>),
    CommandFailed { label: String, status: Option<i32> },
}

impl GateError {
    fn exit_code(&self) -> ExitCode {
        let code: u8 = match self {
            GateError::Usage { .. } => 64,
            GateError::MissingTool { .. } => 127,
            GateError::CommandFailed { status, .. } => {
                status.and_then(|c| u8::try_from(c).ok()).unwrap_or(1)
            }
            GateError::Io { exit, .. } => u8::try_from(*exit).unwrap_or(1),
            GateError::Argo(_) => 1,
        };
        ExitCode::from(code)
    }
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GateError::Io { detail, .. } => write!(f, "{detail}"),
            GateError::Usage { detail } => write!(f, "{detail}"),
            GateError::MissingTool { tool } => {
                write!(f, "missing required provider proof tool: {tool}")
            }
            GateError::Argo(violations) => {
                writeln!(f, "argo desired-state validation FAILED")?;
                for violation in violations {
                    writeln!(f, "  - {violation}")?;
                }
                Ok(())
            }
            GateError::CommandFailed { label, status } => match status {
                Some(code) => write!(f, "{label}: exited with status {code}"),
                None => write!(f, "{label}: terminated by signal"),
            },
        }
    }
}

impl std::error::Error for GateError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_empty_input_matches_known_vector() {
        let digest = hex_sha256(b"");
        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_abc_matches_fips_vector() {
        let digest = hex_sha256(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_two_block_vector_matches() {
        let digest = hex_sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            digest,
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn format_iso_matches_known_timestamp() {
        assert_eq!(format_iso(0), "1970-01-01T00:00:00Z");
        // 2026-05-15T00:00:00Z = (2026 - 1970) years; verified externally.
        // Compute via the same algorithm for round-trip: 1_778_438_400.
        assert_eq!(format_iso(1_778_198_400), "2026-05-08T00:00:00Z");
    }

    #[test]
    fn options_default_mode_is_check() {
        let opts = Options::parse(std::iter::empty()).unwrap();
        assert_eq!(opts.mode, Mode::Check);
        assert!(opts.emit_evidence.is_none());
    }

    #[test]
    fn options_parse_mode_ci_and_evidence() {
        let args: Vec<OsString> = vec![
            "--mode".into(),
            "ci".into(),
            "--emit-evidence".into(),
            "x.json".into(),
        ];
        let opts = Options::parse(args).unwrap();
        assert_eq!(opts.mode, Mode::Ci);
        assert_eq!(opts.emit_evidence, Some(PathBuf::from("x.json")));
    }

    #[test]
    fn options_rejects_invalid_mode() {
        let args: Vec<OsString> = vec!["--mode".into(), "bogus".into()];
        let err = Options::parse(args).unwrap_err();
        assert!(matches!(err, GateError::Usage { .. }));
    }
}
