//! `oya supply-chain` — Rust-owned supply-chain execution surfaces.
//!
//! The ADR-0039 release runner replaces `scripts/supply-chain-adr0039.sh`
//! hand-written orchestration. The shell path remains only as a compatibility
//! shim while workflows and operators call this Rust surface directly.

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
struct Adr0039Step {
    program: &'static str,
    args: Vec<String>,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut iter = args.into_iter();
    match iter.next().as_deref() {
        Some("adr0039") => run_adr0039(iter.collect(), usage),
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
            "--manifest" => parsed.manifest_path = PathBuf::from(next_value(&mut iter, &flag)?),
            "--artifacts-dir" => {
                parsed.artifacts_dir = PathBuf::from(next_value(&mut iter, &flag)?)
            }
            "--rekor-url" => parsed.rekor_url = next_value(&mut iter, &flag)?,
            "--issuer" | "--oidc-issuer" => parsed.issuer = next_value(&mut iter, &flag)?,
            "--identity-regexp" => parsed.identity_regexp = next_value(&mut iter, &flag)?,
            "--dry-run" => parsed.dry_run = true,
            "--format" => {
                let value = next_value(&mut iter, &flag)?;
                parsed.output_format = OutputFormat::parse(&value).ok_or_else(|| {
                    "oya supply-chain adr0039: --format must be text or json".to_string()
                })?;
            }
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn next_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    iter.next()
        .ok_or_else(|| format!("oya supply-chain adr0039: {flag} requires a value"))
}

fn adr0039_plan(args: &Adr0039Args) -> Result<(Vec<String>, Vec<Adr0039Step>), String> {
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

fn execute_adr0039(args: &Adr0039Args, steps: &[Adr0039Step]) -> Result<(), String> {
    require_tool("trivy")?;
    require_tool("cosign")?;
    fs::create_dir_all(args.artifacts_dir.join("sbom")).map_err(|error| {
        format!(
            "could not create supply-chain artifact directories under {}: {error}",
            args.artifacts_dir.display()
        )
    })?;
    for step in steps {
        eprintln!("+ {}", step.command_line());
        let status = Command::new(step.program)
            .args(&step.args)
            .status()
            .map_err(|error| format!("could not start {}: {error}", step.program))?;
        if !status.success() {
            return Err(format!(
                "{} failed with {}",
                step.command_line(),
                process_status_label(&status)
            ));
        }
    }
    Ok(())
}

fn render_adr0039_plan(args: &Adr0039Args, images: &[String], steps: &[Adr0039Step]) {
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
                    "steps": steps.iter().map(Adr0039Step::command_line).collect::<Vec<_>>(),
                    "wiring_evidence": ADR0039_WIRING_EVIDENCE.trim()
                })
            );
        }
    }
}

fn render_adr0039_result(args: &Adr0039Args, images: &[String], steps: &[Adr0039Step]) {
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

fn require_tool(tool: &str) -> Result<(), String> {
    if tool_on_path(tool) {
        Ok(())
    } else {
        Err(format!("missing required ADR-0039 tool: {tool}"))
    }
}

fn tool_on_path(tool: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| dir.join(tool).is_file())
}

fn attest_step(predicate: &Path, kind: &str, image: &str) -> Adr0039Step {
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

fn step<const N: usize>(program: &'static str, args: [&str; N]) -> Adr0039Step {
    step_owned(
        program,
        args.into_iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )
}

fn step_owned(program: &'static str, args: Vec<String>) -> Adr0039Step {
    Adr0039Step { program, args }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

impl Adr0039Step {
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
            .map(Adr0039Step::command_line)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("trivy fs --severity HIGH,CRITICAL --exit-code 1 ."));
        assert!(rendered.contains("trivy image --severity HIGH,CRITICAL --exit-code 1"));
        assert!(rendered.contains("cosign sign --yes"));
        assert!(rendered.contains("cosign verify --rekor-url"));
        assert!(rendered.contains("cosign attest --yes"));
        let _ = fs::remove_dir_all(dir);
    }
}
