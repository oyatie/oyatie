use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

fn main() {
    let root = repo_root();
    let app_shell = root.join("oya/app-shell-frontend");
    let package_json_path = app_shell.join("package.json");
    let workflow_path = root.join(".github/workflows/oya-ci-required.yml");
    let package_json = read(&package_json_path);
    let workflow = read(&workflow_path);

    require(
        package_json.contains("Archived SolidJS transition shell retained for ADR-0393 migration evidence; Leptos/Rust-WASM is canonical."),
        "package.json must explicitly fence the archived SolidJS shell under ADR-0393",
    );

    for forbidden in [
        "node scripts/codegen-check.mjs",
        "node scripts/run-vinxi.mjs",
        "node scripts/shell-contract-check.mjs",
        "\"codegen:check\"",
        "\"dev\":",
        "\"build\":",
        "\"start\":",
        "\"test\":",
        "\"lint\":",
    ] {
        require(
            !package_json.contains(forbidden),
            &format!("package.json must not expose retired SolidJS bridge script: {forbidden}"),
        );
    }

    for forbidden in [
        "pnpm --dir oya/app-shell-frontend codegen:check",
        "node scripts/codegen-check.mjs",
        "node scripts/run-vinxi.mjs",
        "node scripts/shell-contract-check.mjs",
    ] {
        require(
            !workflow.contains(forbidden),
            &format!("oya-ci-required must not invoke retired app-shell MJS bridge: {forbidden}"),
        );
    }

    for script in [
        "codegen-check.mjs",
        "run-vinxi.mjs",
        "shell-contract-check.mjs",
    ] {
        let path = app_shell.join("scripts").join(script);
        let text = read(&path);
        require(
            text.contains("ADR-0393")
                && text.contains("process.exit(1)")
                && text.contains("retired"),
            &format!(
                "{script} must be an inert ADR-0393 retirement fence if retained for historical policy inventory"
            ),
        );
    }

    for generated in [
        "generated/ops-workspace-shell.d.ts",
        "generated/hr-api.d.ts",
    ] {
        let path = app_shell.join(generated);
        let bytes = fs::metadata(&path)
            .unwrap_or_else(|error| fail(&format!("{generated} must exist after `pnpm --dir oya/app-shell-frontend codegen`: {error}")))
            .len();
        require(
            bytes >= 50,
            &format!("{generated} must be non-empty generated client output; saw {bytes} bytes"),
        );
    }
}

fn repo_root() -> PathBuf {
    let mut current = env::current_dir()
        .unwrap_or_else(|error| fail(&format!("cannot read current directory: {error}")));
    loop {
        if current
            .join(".github/workflows/oya-ci-required.yml")
            .exists()
            && current.join("oya/app-shell-frontend/package.json").exists()
        {
            return current;
        }
        if !current.pop() {
            fail("could not find repository root from current directory");
        }
    }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| fail(&format!("failed to read {}: {error}", path.display())))
}

fn require(condition: bool, message: &str) {
    if !condition {
        fail(message);
    }
}

fn fail(message: &str) -> ! {
    eprintln!("app-shell frontend retirement check failed: {message}");
    process::exit(1);
}
