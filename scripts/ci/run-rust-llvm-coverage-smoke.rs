//! Buck2-owned Rust LLVM source-coverage fixture smoke runner.
//!
//! This is narrow local fixture evidence. It proves that a Buck2 target can
//! invoke rustc source-based coverage instrumentation and rustup-sysroot
//! llvm-profdata/llvm-cov. It does not prove production budgets or live CI
//! authority.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_SOURCE: &str = "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs";

#[derive(Debug, Clone)]
pub struct SmokeConfig {
    pub source: PathBuf,
    pub out: Option<PathBuf>,
    pub rustc: Option<PathBuf>,
    pub llvm_bin: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct SmokeResult {
    pub verdict: String,
    pub failures: Vec<String>,
    pub source: String,
    pub fixture_generated: bool,
    pub line_percent: Option<f64>,
    pub region_percent: Option<f64>,
    pub profraw_count: usize,
    pub text_report: String,
    pub rustc_path: String,
    pub rustc_version: String,
    pub rustc_host: String,
    pub rustc_sysroot: String,
    pub llvm_bin: String,
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn json_string(input: &str) -> String {
    format!("\"{}\"", json_escape(input))
}

fn which(binary: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn run(cmd: &mut Command) -> (i32, String, String) {
    match cmd.output() {
        Ok(output) => (
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ),
        Err(error) => (1, String::new(), error.to_string()),
    }
}

fn fail(mut result: SmokeResult, failure: &str) -> SmokeResult {
    result.failures.push(failure.to_owned());
    result.failures.sort();
    result.failures.dedup();
    result.verdict = "FAIL".to_owned();
    result
}

fn empty_result(source: &Path) -> SmokeResult {
    SmokeResult {
        verdict: "FAIL".to_owned(),
        failures: Vec::new(),
        source: source.to_string_lossy().into_owned(),
        fixture_generated: false,
        line_percent: None,
        region_percent: None,
        profraw_count: 0,
        text_report: String::new(),
        rustc_path: String::new(),
        rustc_version: String::new(),
        rustc_host: String::new(),
        rustc_sysroot: String::new(),
        llvm_bin: String::new(),
    }
}

fn host_from_verbose(verbose: &str) -> Option<String> {
    verbose
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_owned))
}

fn temp_dir() -> PathBuf {
    let mut path = env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!(
        "oya-rust-llvm-coverage-smoke-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn extract_percent(export_json: &str, key: &str) -> Option<f64> {
    let compact: String = export_json
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    let key_index = compact.find(&format!("\"{key}\":{{"))?;
    let after_key = &compact[key_index..];
    let percent_index = after_key.find("\"percent\":")? + key_index + "\"percent\":".len();
    let mut end = percent_index;
    while end < compact.len() {
        let ch = compact[end..].chars().next()?;
        if !(ch.is_ascii_digit() || ch == '.') {
            break;
        }
        end += ch.len_utf8();
    }
    compact[percent_index..end].parse().ok()
}

pub fn run_smoke(config: &SmokeConfig) -> SmokeResult {
    let source = &config.source;
    let mut result = empty_result(source);
    if !source.is_file() {
        return fail(result, "missing_source_file");
    }
    let rustc = config
        .rustc
        .clone()
        .or_else(|| env::var_os("OYA_RUSTC").map(PathBuf::from))
        .or_else(|| which("rustc"));
    let Some(rustc) = rustc else {
        return fail(result, "missing_rustc");
    };
    result.rustc_path = rustc.to_string_lossy().into_owned();

    let (status, stdout, stderr) = run(Command::new(&rustc).arg("--version"));
    if status != 0 {
        result.text_report = stderr;
        return fail(result, "rustc_version_failed");
    }
    result.rustc_version = stdout.trim().to_owned();

    let (status, stdout, stderr) = run(Command::new(&rustc).arg("-vV"));
    if status != 0 {
        result.text_report = stderr;
        return fail(result, "rustc_verbose_failed");
    }
    let Some(host) = host_from_verbose(&stdout) else {
        result.text_report = stdout;
        return fail(result, "missing_rustc_host");
    };
    result.rustc_host = host.clone();

    let (status, stdout, stderr) = run(Command::new(&rustc).args(["--print", "sysroot"]));
    if status != 0 {
        result.text_report = stderr;
        return fail(result, "rustc_sysroot_failed");
    }
    let sysroot = stdout.trim().to_owned();
    result.rustc_sysroot = sysroot.clone();
    let llvm_bin = config
        .llvm_bin
        .clone()
        .or_else(|| env::var_os("OYA_LLVM_BIN").map(PathBuf::from))
        .unwrap_or_else(|| {
            Path::new(&sysroot)
                .join("lib")
                .join("rustlib")
                .join(&host)
                .join("bin")
        });
    result.llvm_bin = llvm_bin.to_string_lossy().into_owned();
    let llvm_profdata = llvm_bin.join("llvm-profdata");
    let llvm_cov = llvm_bin.join("llvm-cov");
    if !llvm_profdata.is_file() {
        return fail(result, "missing_llvm_profdata");
    }
    if !llvm_cov.is_file() {
        return fail(result, "missing_llvm_cov");
    }

    let tmp = temp_dir();
    let profraw_dir = tmp.join("profraw");
    if let Err(error) = fs::create_dir_all(&profraw_dir) {
        result.text_report = error.to_string();
        return fail(result, "tempdir_create_failed");
    }
    let binary = tmp.join("branchy");
    let profdata = tmp.join("default.profdata");
    let profile_template = profraw_dir.join("%m-%p.profraw");

    let (status, _stdout, stderr) = run(Command::new(&rustc)
        .args(["-C", "instrument-coverage"])
        .arg(source)
        .arg("-o")
        .arg(&binary));
    if status != 0 {
        result.text_report = stderr;
        let _ = fs::remove_dir_all(&tmp);
        return fail(result, "rustc_instrumented_compile_failed");
    }

    for arg in ["2", "3"] {
        let (status, stdout, stderr) = run(Command::new(&binary)
            .arg(arg)
            .env("LLVM_PROFILE_FILE", &profile_template));
        if status != 0 {
            result.text_report = format!("{stdout}\n{stderr}");
            let _ = fs::remove_dir_all(&tmp);
            return fail(result, "instrumented_fixture_run_failed");
        }
    }
    let profraws = fs::read_dir(&profraw_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("profraw"))
        .collect::<Vec<_>>();
    if profraws.is_empty() {
        let _ = fs::remove_dir_all(&tmp);
        return fail(result, "missing_profraw_output");
    }
    result.profraw_count = profraws.len();

    let mut merge = Command::new(&llvm_profdata);
    merge.arg("merge").arg("-sparse");
    for profraw in &profraws {
        merge.arg(profraw);
    }
    merge.arg("-o").arg(&profdata);
    let (status, stdout, stderr) = run(&mut merge);
    if status != 0 {
        result.text_report = format!("{stdout}\n{stderr}");
        let _ = fs::remove_dir_all(&tmp);
        return fail(result, "llvm_profdata_merge_failed");
    }

    let (status, export_stdout, export_stderr) = run(Command::new(&llvm_cov)
        .arg("export")
        .arg(&binary)
        .arg("--instr-profile")
        .arg(&profdata)
        .arg("--format=text"));
    if status != 0 {
        result.text_report = export_stderr;
        let _ = fs::remove_dir_all(&tmp);
        return fail(result, "llvm_cov_export_failed");
    }
    let (status, report_stdout, report_stderr) = run(Command::new(&llvm_cov)
        .arg("report")
        .arg(&binary)
        .arg("--instr-profile")
        .arg(&profdata));
    if status != 0 {
        result.text_report = report_stderr;
        let _ = fs::remove_dir_all(&tmp);
        return fail(result, "llvm_cov_report_failed");
    }
    let line_percent = extract_percent(&export_stdout, "lines").unwrap_or(0.0);
    let region_percent = extract_percent(&export_stdout, "regions").unwrap_or(0.0);
    result.line_percent = Some(line_percent);
    result.region_percent = Some(region_percent);
    result.text_report = report_stdout;
    if line_percent < 100.0 || region_percent < 100.0 {
        let _ = fs::remove_dir_all(&tmp);
        return fail(result, "fixture_coverage_below_100_percent");
    }
    result.fixture_generated = true;
    result.verdict = "PASS".to_owned();
    result.failures.clear();
    let _ = fs::remove_dir_all(&tmp);
    result
}

pub fn render_json(result: &SmokeResult) -> String {
    let failures = result
        .failures
        .iter()
        .map(|failure| json_string(failure))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"ambient_path_llvm_tools_required\":false,\"authority_boundary\":\"Buck2 local fixture LLVM source-coverage smoke only; no production coverage budget or live required-context authority proven\",\"coverage_budget_enforced\":false,\"coverage_report_format\":[\"json\",\"text\"],\"failures\":[{}],\"fixture_coverage_smoke_generated\":{},\"fixture_line_coverage_percent\":{},\"fixture_region_coverage_percent\":{},\"hyperscaler_grade\":false,\"live_required_context_execution_proven\":false,\"llvm_bin\":{},\"llvm_cov_operations\":[\"export --format=text\",\"report\"],\"official_sources\":[{{\"url\":\"https://doc.rust-lang.org/rustc/instrument-coverage.html\"}},{{\"url\":\"https://clang.llvm.org/docs/SourceBasedCodeCoverage.html\"}},{{\"url\":\"https://buck2.build/docs/users/commands/\"}}],\"p0_0_green\":false,\"phase0_complete\":false,\"profdata_operation\":\"llvm-profdata merge -sparse\",\"profile_collision_guard\":\"%m-%p\",\"profile_env_var\":\"LLVM_PROFILE_FILE\",\"production_coverage_report_generated\":false,\"production_ready\":false,\"profraw_count\":{},\"protected_branch_authority_proven\":false,\"rustc_flag\":\"rustc -C instrument-coverage\",\"rustc_host\":{},\"rustc_path\":{},\"rustc_sysroot\":{},\"rustc_version\":{},\"smoke_source\":{},\"status_mutation_performed\":false,\"text_report\":{},\"verdict\":{}}}",
        failures,
        if result.fixture_generated {
            "true"
        } else {
            "false"
        },
        result.line_percent.unwrap_or(0.0),
        result.region_percent.unwrap_or(0.0),
        json_string(&result.llvm_bin),
        result.profraw_count,
        json_string(&result.rustc_host),
        json_string(&result.rustc_path),
        json_string(&result.rustc_sysroot),
        json_string(&result.rustc_version),
        json_string(&result.source),
        json_string(&result.text_report),
        json_string(&result.verdict),
    )
}

fn parse_args() -> SmokeConfig {
    let mut source = PathBuf::from(DEFAULT_SOURCE);
    let mut out = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => {
                source = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_SOURCE.to_owned()))
            }
            "--out" => out = args.next().map(PathBuf::from),
            _ => {}
        }
    }
    SmokeConfig {
        source,
        out,
        rustc: None,
        llvm_bin: None,
    }
}

fn main() {
    let config = parse_args();
    let result = run_smoke(&config);
    let rendered = render_json(&result);
    if let Some(out) = &config.out {
        if let Some(parent) = out.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(error) = fs::write(out, format!("{rendered}\n")) {
            eprintln!("failed to write output: {error}");
            std::process::exit(1);
        }
    }
    println!("{rendered}");
    if result.verdict != "PASS" {
        std::process::exit(1);
    }
}
