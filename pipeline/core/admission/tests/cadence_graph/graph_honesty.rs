//! The advisory buck2 graph lane: its cadence, and the absence that makes
//! it advisory.

/// The buck2 graph runs on every push to `dev` and is advisory.
///
/// Advisory is the property that keeps the lane inside the dual-proof
/// prohibition, and it is invisible in the lane's own file: a workflow cannot
/// say "nothing depends on me". So the cadence is read here, and the absence
/// is read from the two files that could create the dependency -- a `needs:`
/// on this job, or its name in either required occupant set.
#[test]
fn the_buck2_graph_is_advisory_on_every_push() {
    let y = super::read(".github/workflows/buck2-graph-honesty.yml");
    assert!(
        y.contains("\n  push:\n    branches: [dev]\n"),
        "must run per push"
    );
    assert!(!y.contains("schedule:"), "the weekly cadence is retired");
    assert!(
        y.contains("buck2 build //..."),
        "the graph is what it builds"
    );

    let job = "buck2-graph-honesty";
    assert!(
        !pipeline_admission::PRESUBMIT_JOBS.contains(&job)
            && !pipeline_admission::POSTSUBMIT_JOBS.contains(&job)
    );
    for cadence in [
        ".github/workflows/presubmit.yml",
        ".github/workflows/postsubmit.yml",
    ] {
        assert!(
            !super::read(cadence).contains(job),
            "{cadence} must not wait on the graph"
        );
    }
}

struct FormatRepo {
    path: std::path::PathBuf,
    baseline: String,
}

impl FormatRepo {
    fn new(child: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("format-inputs-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(path.join("src")).unwrap();
        std::fs::create_dir_all(path.join("fixtures")).unwrap();
        for config in ["rustfmt.toml", "rust-toolchain.toml"] {
            std::fs::write(path.join(config), super::read(config)).unwrap();
        }
        let mut repo = Self {
            path,
            baseline: String::new(),
        };
        repo.git(&["init", "--quiet"]);
        repo.write("src/lib.rs", "mod builders;\npub use builders::value;\n");
        repo.write("src/builders.rs", child);
        repo.write("fixtures/invalid.rs", "This is an invalid Rust fixture {\n");
        repo.git(&["add", "."]);
        repo.baseline = repo.commit(None);
        repo
    }

    fn write(&self, path: &str, text: &str) {
        std::fs::write(self.path.join(path), text).unwrap();
    }

    fn stage(&self, path: &str, text: &str) {
        self.write(path, text);
        self.git(&["add", "--", path]);
    }

    fn git(&self, args: &[&str]) -> String {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .env("GIT_AUTHOR_NAME", "Fixture User")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture User")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn commit(&self, parent: Option<&str>) -> String {
        let tree = self.git(&["write-tree"]);
        let mut args = vec!["commit-tree", tree.as_str(), "-m", "Formatting fixture"];
        if let Some(parent) = parent {
            args.extend(["-p", parent]);
        }
        let commit = self.git(&args);
        self.git(&["update-ref", "HEAD", &commit]);
        commit
    }

    fn check(&self, hook: &str, expected: bool) {
        self.check_with_environment(hook, expected, &[]);
    }

    fn check_with_environment(&self, hook: &str, expected: bool, environment: &[(&str, &str)]) {
        let output = std::process::Command::new("sh")
            .arg(super::repo_root().join(".githooks").join(hook))
            .current_dir(&self.path)
            .env_remove("RUSTUP_TOOLCHAIN")
            .envs(environment.iter().copied())
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            expected,
            "{hook}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for FormatRepo {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).unwrap();
    }
}

const CLEAN_CHILD: &str = "pub fn value() -> usize {\n    1\n}\n";
const BAD_CHILD: &str = "pub fn value()->usize{1}\n";
const CHANGED_ROOT: &str =
    "mod builders;\npub use builders::value;\n\npub fn selected() -> usize {\n    value()\n}\n";
const BAD_ROOT: &str =
    "mod builders;\npub use builders::value;\npub fn selected()->usize{value()}\n";

#[test]
fn staged_formatting_uses_complete_index_inputs_and_recursive_selected_roots() {
    let repo = FormatRepo::new(CLEAN_CHILD);
    repo.stage("src/lib.rs", CHANGED_ROOT);
    std::fs::remove_file(repo.path.join("src/builders.rs")).unwrap();
    repo.write("rustfmt.toml", "invalid configuration\n");
    repo.check("pre-commit", true);

    repo.stage("src/builders.rs", BAD_CHILD);
    repo.write("src/builders.rs", CLEAN_CHILD);
    repo.check("pre-commit", false);
    repo.stage("src/builders.rs", CLEAN_CHILD);
    repo.stage("src/lib.rs", BAD_ROOT);
    repo.write("src/lib.rs", CHANGED_ROOT);
    repo.check("pre-commit", false);

    let unchanged_bad_child = FormatRepo::new(BAD_CHILD);
    unchanged_bad_child.stage("src/lib.rs", CHANGED_ROOT);
    unchanged_bad_child.check("pre-commit", false);
}

#[test]
fn push_formatting_uses_committed_inputs_and_refuses_a_missing_base() {
    let repo = FormatRepo::new(CLEAN_CHILD);
    repo.stage("src/lib.rs", CHANGED_ROOT);
    repo.commit(Some(&repo.baseline));
    repo.check("pre-push", false);
    repo.git(&["update-ref", "refs/remotes/origin/dev", &repo.baseline]);
    repo.write("src/lib.rs", BAD_ROOT);
    repo.write("src/builders.rs", BAD_CHILD);
    repo.write("rustfmt.toml", "invalid configuration\n");
    repo.check("pre-push", true);

    repo.stage("src/lib.rs", BAD_ROOT);
    repo.commit(Some(&repo.baseline));
    repo.write("src/lib.rs", CHANGED_ROOT);
    repo.check("pre-push", false);
}

#[test]
fn formatting_hooks_validate_captured_toolchains_despite_inherited_overrides() {
    let toolchain_config: toml::Value =
        toml::from_str(&super::read("rust-toolchain.toml")).unwrap();
    let installed = toolchain_config["toolchain"]["channel"].as_str().unwrap();
    for hook in ["pre-commit", "pre-push"] {
        let repo = FormatRepo::new(CLEAN_CHILD);
        std::fs::create_dir(repo.path.join("bin")).unwrap();
        repo.write("bin/rustfmt", "#!/bin/sh\nexit 0\n");
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            repo.path.join("bin/rustfmt"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        let path = format!(
            "{}:{}",
            repo.path.join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        repo.git(&["update-ref", "refs/remotes/origin/dev", &repo.baseline]);
        repo.stage("src/lib.rs", CHANGED_ROOT);
        if hook == "pre-push" {
            repo.commit(Some(&repo.baseline));
        }
        repo.check_with_environment(
            hook,
            true,
            &[
                ("RUSTUP_TOOLCHAIN", "unavailable-formatting-override"),
                ("PATH", &path),
            ],
        );

        for invalid in [
            "invalid toolchain configuration\n",
            "[toolchain]\nchannel = \"unavailable-formatting-toolchain\"\n",
        ] {
            repo.stage("rust-toolchain.toml", invalid);
            if hook == "pre-push" {
                repo.commit(Some(&repo.baseline));
            }
            repo.check_with_environment(
                hook,
                false,
                &[("RUSTUP_TOOLCHAIN", installed), ("PATH", &path)],
            );
        }
        repo.git(&["update-index", "--force-remove", "rust-toolchain.toml"]);
        if hook == "pre-push" {
            repo.commit(Some(&repo.baseline));
        }
        repo.check_with_environment(
            hook,
            false,
            &[("RUSTUP_TOOLCHAIN", installed), ("PATH", &path)],
        );
    }
}
