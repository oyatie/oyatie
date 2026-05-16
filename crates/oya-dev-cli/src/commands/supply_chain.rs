//! `oya supply-chain` — Rust-owned supply-chain execution surfaces.
//!
//! The ADR-0039 release runner and CI Trivy installer replace hand-written
//! shell orchestration. Shell paths remain only as compatibility shims while
//! workflows and operators call this Rust surface directly.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde_json::json;

use crate::command_output::OutputFormat;
use crate::command_process::process_status_label;

const DEFAULT_ARTIFACTS_DIR: &str = "artifacts/supply-chain";
const DEFAULT_REKOR_URL: &str = "https://rekor.sigstore.dev";
const DEFAULT_ISSUER: &str = "https://token.actions.githubusercontent.com";
const DEFAULT_IDENTITY_REGEXP: &str = "https://github.com/.+/.+/.github/workflows/.+@refs/tags/v.+";
const DEFAULT_TRIVY_VERSION: &str = "0.70.0";
const DEFAULT_TRIVY_INSTALL_DIR: &str = "/usr/local/bin";

const ADR0039_WIRING_EVIDENCE: &str = r#"
trivy fs --severity HIGH,CRITICAL --exit-code 1 .
trivy image --severity HIGH,CRITICAL --exit-code 1 <image>
trivy config --severity HIGH,CRITICAL --exit-code 1 infra/
trivy fs --scanners vuln,secret,license --format sarif --output artifacts/supply-chain/trivy.sarif .
trivy fs --format spdx-json --output artifacts/supply-chain/sbom/oyatie.spdx.json .
trivy fs --format cyclonedx --output artifacts/supply-chain/sbom/oyatie.cyclonedx.json .
cosign sign --yes <image>
cosign verify --rekor-url https://rekor.sigstore.dev <image>
cosign attest --yes --predicate artifacts/supply-chain/sbom/oyatie.spdx.json --type spdx <image>
cosign attest --yes --predicate artifacts/supply-chain/sbom/oyatie.cyclonedx.json --type cyclonedx <image>
cosign attest --yes --predicate artifacts/supply-chain/trivy.sarif --type vuln <image>
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Adr0039Args {
    manifest_path: PathBuf,
    artifacts_dir: PathBuf,
    rekor_url: String,
    issuer: String,
    identity_regexp: String,
    dry_run: bool,
    output_format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstallTrivyArgs {
    version: String,
    install_dir: PathBuf,
    dry_run: bool,
    output_format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstallTrivyPlan {
    archive_name: String,
    checksums_name: String,
    base_url: String,
    tmp_dir: PathBuf,
    archive_path: PathBuf,
    checksums_path: PathBuf,
    selected_checksum_path: PathBuf,
    extracted_trivy_path: PathBuf,
    installed_trivy_path: PathBuf,
    steps: Vec<CommandStep>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandStep {
    program: String,
    args: Vec<String>,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut iter = args.into_iter();
    match iter.next().as_deref() {
        Some("adr0039") => run_adr0039(iter.collect(), usage),
        Some("install-trivy") => run_install_trivy(iter.collect(), usage),
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}

fn run_adr0039(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_adr0039_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match adr0039_plan(&parsed) {
        Ok((images, steps)) => {
            if parsed.dry_run {
                render_adr0039_plan(&parsed, &images, &steps);
                return ExitCode::SUCCESS;
            }
            match execute_adr0039(&parsed, &steps) {
                Ok(()) => {
                    render_adr0039_result(&parsed, &images, &steps);
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("adr0039 supply-chain execution failed: {message}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(message) => {
            eprintln!("adr0039 supply-chain planning failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_install_trivy(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_install_trivy_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let plan = install_trivy_plan(&parsed);
    if parsed.dry_run {
        render_install_trivy_plan(&parsed, &plan);
        return ExitCode::SUCCESS;
    }
    match execute_install_trivy(&parsed, &plan) {
        Ok(()) => {
            render_install_trivy_result(&parsed, &plan);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("trivy install failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_install_trivy_args(args: Vec<String>, usage: &str) -> Result<InstallTrivyArgs, String> {
    let mut parsed = InstallTrivyArgs {
        version: env::var("TRIVY_VERSION").unwrap_or_else(|_| DEFAULT_TRIVY_VERSION.to_string()),
        install_dir: env::var_os("TRIVY_INSTALL_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_TRIVY_INSTALL_DIR)),
        dry_run: false,
        output_format: OutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--version" => {
                parsed.version = next_value("oya supply-chain install-trivy", &mut iter, &flag)?
            }
            "--install-dir" => {
                parsed.install_dir = PathBuf::from(next_value(
                    "oya supply-chain install-trivy",
                    &mut iter,
                    &flag,
                )?)
            }
            "--dry-run" => parsed.dry_run = true,
            "--format" => {
                let value = next_value("oya supply-chain install-trivy", &mut iter, &flag)?;
                parsed.output_format = OutputFormat::parse(&value).ok_or_else(|| {
                    "oya supply-chain install-trivy: --format must be text or json".to_string()
                })?;
            }
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn parse_adr0039_args(args: Vec<String>, usage: &str) -> Result<Adr0039Args, String> {
    let mut parsed = Adr0039Args {
        manifest_path: PathBuf::from("registry/release/images.yaml"),
        artifacts_dir: env::var_os("OYA_SUPPLY_CHAIN_ARTIFACTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ARTIFACTS_DIR)),
        rekor_url: env::var("OYA_REKOR_URL").unwrap_or_else(|_| DEFAULT_REKOR_URL.to_string()),
        issuer: env::var("OYA_COSIGN_OIDC_ISSUER").unwrap_or_else(|_| DEFAULT_ISSUER.to_string()),
        identity_regexp: env::var("OYA_COSIGN_IDENTITY_REGEXP")
            .unwrap_or_else(|_| DEFAULT_IDENTITY_REGEXP.to_string()),
        dry_run: false,
        output_format: OutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--manifest" => {
                parsed.manifest_path =
                    PathBuf::from(next_value("oya supply-chain adr0039", &mut iter, &flag)?)
            }
            "--artifacts-dir" => {
                parsed.artifacts_dir =
                    PathBuf::from(next_value("oya supply-chain adr0039", &mut iter, &flag)?)
            }
            "--rekor-url" => {
                parsed.rekor_url = next_value("oya supply-chain adr0039", &mut iter, &flag)?
            }
            "--issuer" | "--oidc-issuer" => {
                parsed.issuer = next_value("oya supply-chain adr0039", &mut iter, &flag)?
            }
            "--identity-regexp" => {
                parsed.identity_regexp = next_value("oya supply-chain adr0039", &mut iter, &flag)?
            }
            "--dry-run" => parsed.dry_run = true,
            "--format" => {
                let value = next_value("oya supply-chain adr0039", &mut iter, &flag)?;
                parsed.output_format = OutputFormat::parse(&value).ok_or_else(|| {
                    "oya supply-chain adr0039: --format must be text or json".to_string()
                })?;
            }
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn next_value(
    command: &str,
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("{command}: {flag} requires a value"))
}

fn adr0039_plan(args: &Adr0039Args) -> Result<(Vec<String>, Vec<CommandStep>), String> {
    let images = release_images(&args.manifest_path)?;
    if images.is_empty() {
        return Err(format!(
            "release image manifest has no image refs: {}",
            args.manifest_path.display()
        ));
    }

    let trivy_sarif = args.artifacts_dir.join("trivy.sarif");
    let sbom_dir = args.artifacts_dir.join("sbom");
    let spdx = sbom_dir.join("oyatie.spdx.json");
    let cyclonedx = sbom_dir.join("oyatie.cyclonedx.json");

    let mut steps = vec![
        step(
            "trivy",
            ["fs", "--severity", "HIGH,CRITICAL", "--exit-code", "1", "."],
        ),
        step(
            "trivy",
            [
                "config",
                "--severity",
                "HIGH,CRITICAL",
                "--exit-code",
                "1",
                "infra/",
            ],
        ),
        step_owned(
            "trivy",
            vec![
                "fs".into(),
                "--scanners".into(),
                "vuln,secret,license".into(),
                "--format".into(),
                "sarif".into(),
                "--output".into(),
                path_string(&trivy_sarif),
                ".".into(),
            ],
        ),
        step_owned(
            "trivy",
            vec![
                "fs".into(),
                "--format".into(),
                "spdx-json".into(),
                "--output".into(),
                path_string(&spdx),
                ".".into(),
            ],
        ),
        step_owned(
            "trivy",
            vec![
                "fs".into(),
                "--format".into(),
                "cyclonedx".into(),
                "--output".into(),
                path_string(&cyclonedx),
                ".".into(),
            ],
        ),
    ];

    for image in &images {
        steps.push(step_owned(
            "trivy",
            vec![
                "image".into(),
                "--severity".into(),
                "HIGH,CRITICAL".into(),
                "--exit-code".into(),
                "1".into(),
                image.clone(),
            ],
        ));
        steps.push(step_owned(
            "cosign",
            vec!["sign".into(), "--yes".into(), image.clone()],
        ));
        steps.push(step_owned(
            "cosign",
            vec![
                "verify".into(),
                "--rekor-url".into(),
                args.rekor_url.clone(),
                "--certificate-oidc-issuer".into(),
                args.issuer.clone(),
                "--certificate-identity-regexp".into(),
                args.identity_regexp.clone(),
                image.clone(),
            ],
        ));
        steps.push(attest_step(&spdx, "spdx", image));
        steps.push(attest_step(&cyclonedx, "cyclonedx", image));
        steps.push(attest_step(&trivy_sarif, "vuln", image));
    }

    Ok((images, steps))
}

fn install_trivy_plan(args: &InstallTrivyArgs) -> InstallTrivyPlan {
    let archive_name = format!("trivy_{}_Linux-64bit.tar.gz", args.version);
    let checksums_name = format!("trivy_{}_checksums.txt", args.version);
    let base_url = format!(
        "https://github.com/aquasecurity/trivy/releases/download/v{}",
        args.version
    );
    let tmp_dir = unique_trivy_tmp_dir(&args.version);
    let archive_path = tmp_dir.join(&archive_name);
    let checksums_path = tmp_dir.join(&checksums_name);
    let selected_checksum_path = tmp_dir.join("selected-checksum.txt");
    let extracted_trivy_path = tmp_dir.join("trivy");
    let installed_trivy_path = args.install_dir.join("trivy");
    let steps = vec![
        step_owned(
            "curl",
            vec![
                "--fail".into(),
                "--location".into(),
                "--silent".into(),
                "--show-error".into(),
                "--output".into(),
                path_string(&archive_path),
                format!("{base_url}/{archive_name}"),
            ],
        ),
        step_owned(
            "curl",
            vec![
                "--fail".into(),
                "--location".into(),
                "--silent".into(),
                "--show-error".into(),
                "--output".into(),
                path_string(&checksums_path),
                format!("{base_url}/{checksums_name}"),
            ],
        ),
        step_owned(
            "sha256sum",
            vec!["-c".into(), path_string(&selected_checksum_path)],
        ),
        step_owned(
            "tar",
            vec![
                "-xzf".into(),
                path_string(&archive_path),
                "-C".into(),
                path_string(&tmp_dir),
                "trivy".into(),
            ],
        ),
        step_owned(
            "install",
            vec![
                "-m".into(),
                "0755".into(),
                path_string(&extracted_trivy_path),
                path_string(&installed_trivy_path),
            ],
        ),
        step_owned(path_string(&installed_trivy_path), vec!["--version".into()]),
    ];
    InstallTrivyPlan {
        archive_name,
        checksums_name,
        base_url,
        tmp_dir,
        archive_path,
        checksums_path,
        selected_checksum_path,
        extracted_trivy_path,
        installed_trivy_path,
        steps,
    }
}

fn execute_adr0039(args: &Adr0039Args, steps: &[CommandStep]) -> Result<(), String> {
    require_tool("trivy")?;
    require_tool("cosign")?;
    fs::create_dir_all(args.artifacts_dir.join("sbom")).map_err(|error| {
        format!(
            "could not create supply-chain artifact directories under {}: {error}",
            args.artifacts_dir.display()
        )
    })?;
    for step in steps {
        run_step(step)?;
    }
    Ok(())
}

fn execute_install_trivy(args: &InstallTrivyArgs, plan: &InstallTrivyPlan) -> Result<(), String> {
    require_tool("curl")?;
    require_tool("sha256sum")?;
    require_tool("tar")?;
    require_tool("install")?;
    fs::create_dir_all(&plan.tmp_dir).map_err(|error| {
        format!(
            "could not create trivy installer temp dir {}: {error}",
            plan.tmp_dir.display()
        )
    })?;
    if let Err(error) = fs::create_dir_all(&args.install_dir) {
        return Err(format!(
            "could not create trivy install dir {}: {error}",
            args.install_dir.display()
        ));
    }
    run_step(&plan.steps[0])?;
    run_step(&plan.steps[1])?;
    write_selected_checksum(plan)?;
    run_step(&plan.steps[2])?;
    run_step(&plan.steps[3])?;
    if directory_is_writable(&args.install_dir) {
        run_step(&plan.steps[4])?;
    } else {
        let sudo_step = step_owned(
            "sudo",
            std::iter::once("install".to_string())
                .chain(plan.steps[4].args.clone())
                .collect(),
        );
        run_step(&sudo_step)?;
    }
    run_step(&plan.steps[5])?;
    let _ = fs::remove_dir_all(&plan.tmp_dir);
    Ok(())
}

fn run_step(step: &CommandStep) -> Result<(), String> {
    eprintln!("+ {}", step.command_line());
    let status = Command::new(&step.program)
        .args(&step.args)
        .status()
        .map_err(|error| format!("could not start {}: {error}", step.program))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} failed with {}",
            step.command_line(),
            process_status_label(&status)
        ))
    }
}

fn render_adr0039_plan(args: &Adr0039Args, images: &[String], steps: &[CommandStep]) {
    match args.output_format {
        OutputFormat::Text => {
            println!("ADR-0039 supply-chain Rust runner dry-run");
            println!("manifest={}", args.manifest_path.display());
            println!("artifacts_dir={}", args.artifacts_dir.display());
            println!("images={}", images.len());
            for step in steps {
                println!("{}", step.command_line());
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                json!({
                    "command": "oya supply-chain adr0039",
                    "mode": "dry-run",
                    "manifest": args.manifest_path,
                    "artifacts_dir": args.artifacts_dir,
                    "images": images,
                    "steps": steps.iter().map(CommandStep::command_line).collect::<Vec<_>>(),
                    "wiring_evidence": ADR0039_WIRING_EVIDENCE.trim()
                })
            );
        }
    }
}

fn render_adr0039_result(args: &Adr0039Args, images: &[String], steps: &[CommandStep]) {
    match args.output_format {
        OutputFormat::Text => println!(
            "ADR-0039 supply-chain Rust runner passed: {} images, {} steps, artifacts_dir={}",
            images.len(),
            steps.len(),
            args.artifacts_dir.display()
        ),
        OutputFormat::Json => println!(
            "{}",
            json!({
                "command": "oya supply-chain adr0039",
                "status": "passed",
                "images_checked": images.len(),
                "steps_executed": steps.len(),
                "artifacts_dir": args.artifacts_dir
            })
        ),
    }
}

fn render_install_trivy_plan(args: &InstallTrivyArgs, plan: &InstallTrivyPlan) {
    match args.output_format {
        OutputFormat::Text => {
            println!("Trivy CI installer Rust runner dry-run");
            println!("version={}", args.version);
            println!("install_dir={}", args.install_dir.display());
            println!("tmp_dir={}", plan.tmp_dir.display());
            println!("rust action: create temp dir");
            println!("rust action: select checksum for {}", plan.archive_name);
            for step in &plan.steps {
                println!("{}", step.command_line());
            }
            println!(
                "fallback when install_dir is not writable: sudo install -m 0755 {} {}",
                plan.extracted_trivy_path.display(),
                plan.installed_trivy_path.display()
            );
        }
        OutputFormat::Json => {
            println!(
                "{}",
                json!({
                    "command": "oya supply-chain install-trivy",
                    "mode": "dry-run",
                    "version": args.version,
                    "install_dir": args.install_dir,
                    "tmp_dir": plan.tmp_dir,
                    "archive": plan.archive_name,
                    "checksums": plan.checksums_name,
                    "base_url": plan.base_url,
                    "steps": plan.steps.iter().map(CommandStep::command_line).collect::<Vec<_>>(),
                    "rust_actions": [
                        "create installer temp dir",
                        "select checksum line for archive",
                        "write selected-checksum.txt with absolute archive path",
                        "remove temp dir after successful install"
                    ],
                    "sudo_fallback": format!(
                        "sudo install -m 0755 {} {}",
                        plan.extracted_trivy_path.display(),
                        plan.installed_trivy_path.display()
                    )
                })
            );
        }
    }
}

fn render_install_trivy_result(args: &InstallTrivyArgs, plan: &InstallTrivyPlan) {
    match args.output_format {
        OutputFormat::Text => println!(
            "Trivy CI installer Rust runner passed: version={}, installed={}",
            args.version,
            plan.installed_trivy_path.display()
        ),
        OutputFormat::Json => println!(
            "{}",
            json!({
                "command": "oya supply-chain install-trivy",
                "status": "passed",
                "version": args.version,
                "installed_trivy": plan.installed_trivy_path
            })
        ),
    }
}

fn release_images(manifest_path: &Path) -> Result<Vec<String>, String> {
    let contents = fs::read_to_string(manifest_path).map_err(|error| {
        format!(
            "release image manifest not found or unreadable {}: {error}",
            manifest_path.display()
        )
    })?;
    let mut images = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed == "images:"
            || trimmed == "images: []"
        {
            continue;
        }
        let value = trimmed
            .strip_prefix("- ref:")
            .or_else(|| trimmed.strip_prefix("ref:"))
            .or_else(|| trimmed.strip_prefix("- "));
        if let Some(value) = value {
            let image = clean_manifest_value(value);
            if !image.is_empty() {
                images.push(image);
            }
        }
    }
    Ok(images)
}

fn clean_manifest_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn write_selected_checksum(plan: &InstallTrivyPlan) -> Result<(), String> {
    let checksums = fs::read_to_string(&plan.checksums_path).map_err(|error| {
        format!(
            "could not read trivy checksums {}: {error}",
            plan.checksums_path.display()
        )
    })?;
    let selected = select_checksum_line(&checksums, &plan.archive_name, &plan.archive_path)?;
    fs::write(&plan.selected_checksum_path, selected).map_err(|error| {
        format!(
            "could not write selected trivy checksum {}: {error}",
            plan.selected_checksum_path.display()
        )
    })
}

fn select_checksum_line(
    checksums: &str,
    archive_name: &str,
    archive_path: &Path,
) -> Result<String, String> {
    for line in checksums.lines() {
        let mut parts = line.split_whitespace();
        let Some(hash) = parts.next() else {
            continue;
        };
        let Some(path) = parts.last() else {
            continue;
        };
        if path.trim_start_matches('*') == archive_name {
            return Ok(format!("{hash}  {}\n", archive_path.display()));
        }
    }
    Err(format!(
        "trivy checksum file does not contain archive {archive_name}"
    ))
}

fn require_tool(tool: &str) -> Result<(), String> {
    if tool_on_path(tool) {
        Ok(())
    } else {
        Err(format!("missing required supply-chain tool: {tool}"))
    }
}

fn tool_on_path(tool: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| dir.join(tool).is_file())
}

fn directory_is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(
        ".oya-trivy-install-write-probe-{}",
        std::process::id()
    ));
    let created = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .is_ok();
    if created {
        let _ = fs::remove_file(probe);
    }
    created
}

fn unique_trivy_tmp_dir(version: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!(
        "oya-trivy-install-{}-{}-{nanos}",
        version,
        std::process::id()
    ))
}

fn attest_step(predicate: &Path, kind: &str, image: &str) -> CommandStep {
    step_owned(
        "cosign",
        vec![
            "attest".into(),
            "--yes".into(),
            "--predicate".into(),
            path_string(predicate),
            "--type".into(),
            kind.into(),
            image.into(),
        ],
    )
}

fn step<const N: usize>(program: &'static str, args: [&str; N]) -> CommandStep {
    step_owned(
        program,
        args.into_iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )
}

fn step_owned(program: impl Into<String>, args: Vec<String>) -> CommandStep {
    CommandStep {
        program: program.into(),
        args,
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

impl CommandStep {
    fn command_line(&self) -> String {
        std::iter::once(self.program.to_string())
            .chain(self.args.iter().map(|arg| shell_token(arg)))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn shell_token(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "._/:@=-,+".contains(character))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_image_manifest_refs() {
        let dir = std::env::temp_dir().join(format!("oya-adr0039-test-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir created");
        let manifest = dir.join("images.yaml");
        fs::write(
            &manifest,
            "images:\n  - ref: 'ghcr.io/acme/app@sha256:abc'\n  - ghcr.io/acme/worker@sha256:def\n",
        )
        .expect("manifest written");

        let images = release_images(&manifest).expect("images parsed");

        assert_eq!(
            images,
            vec![
                "ghcr.io/acme/app@sha256:abc".to_string(),
                "ghcr.io/acme/worker@sha256:def".to_string()
            ]
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn adr0039_plan_contains_required_trivy_and_cosign_steps() {
        let dir =
            std::env::temp_dir().join(format!("oya-adr0039-plan-test-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir created");
        let manifest = dir.join("images.yaml");
        fs::write(&manifest, "images:\n  - ref: ghcr.io/acme/app@sha256:abc\n")
            .expect("manifest written");
        let args = Adr0039Args {
            manifest_path: manifest,
            artifacts_dir: dir.join("artifacts"),
            rekor_url: DEFAULT_REKOR_URL.to_string(),
            issuer: DEFAULT_ISSUER.to_string(),
            identity_regexp: DEFAULT_IDENTITY_REGEXP.to_string(),
            dry_run: true,
            output_format: OutputFormat::Text,
        };

        let (_images, steps) = adr0039_plan(&args).expect("plan built");
        let rendered = steps
            .iter()
            .map(CommandStep::command_line)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("trivy fs --severity HIGH,CRITICAL --exit-code 1 ."));
        assert!(rendered.contains("trivy image --severity HIGH,CRITICAL --exit-code 1"));
        assert!(rendered.contains("cosign sign --yes"));
        assert!(rendered.contains("cosign verify --rekor-url"));
        assert!(rendered.contains("cosign attest --yes"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn install_trivy_plan_contains_pinned_download_and_checksum_steps() {
        let args = InstallTrivyArgs {
            version: DEFAULT_TRIVY_VERSION.to_string(),
            install_dir: PathBuf::from(DEFAULT_TRIVY_INSTALL_DIR),
            dry_run: true,
            output_format: OutputFormat::Json,
        };

        let plan = install_trivy_plan(&args);
        let rendered = plan
            .steps
            .iter()
            .map(CommandStep::command_line)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("trivy_0.70.0_Linux-64bit.tar.gz"));
        assert!(rendered.contains("trivy_0.70.0_checksums.txt"));
        assert!(rendered.contains("sha256sum -c"));
        assert!(rendered.contains("tar -xzf"));
        assert!(rendered.contains("install -m 0755"));
    }

    #[test]
    fn install_trivy_checksum_selection_rewrites_archive_path() {
        let archive_path = PathBuf::from("/tmp/oya/trivy_0.70.0_Linux-64bit.tar.gz");
        let selected = select_checksum_line(
            "abc123  trivy_0.70.0_Linux-64bit.tar.gz\n\
             deadbeef  trivy_0.70.0_checksums.txt\n",
            "trivy_0.70.0_Linux-64bit.tar.gz",
            &archive_path,
        )
        .expect("checksum selected");

        assert_eq!(
            selected,
            "abc123  /tmp/oya/trivy_0.70.0_Linux-64bit.tar.gz\n"
        );
    }
}
