// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[test]
fn doc_rustdoc_uses_pinned_rustdoc_and_target_dir() {
    let temp = temp_dir("doc-rustdoc-success");
    let target_dir = temp.join("target").join("custom-doc-target");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .env("FAKE_CARGO_PRINT_ARGS", "1")
        .args([
            "doc",
            "rustdoc",
            "--cargo",
            env!("CARGO_BIN_EXE_fake-cargo"),
            "--rustdoc",
            "/tmp/rustdoc-fixture",
            "--target-dir",
            target_dir.to_str().expect("utf8 target"),
            "--keep-target-dir",
        ])
        .output()
        .expect("doc rustdoc command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cargo-args:doc --workspace --no-deps --all-features"),
        "stdout={stdout}"
    );
    assert!(
        stdout.contains("rustdoc:/tmp/rustdoc-fixture"),
        "stdout={stdout}"
    );
    assert!(
        stdout.contains(&format!("target:{}", target_dir.display())),
        "stdout={stdout}"
    );
    assert!(
        stdout.contains("rustdoc generation passed:"),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_rustdoc_fails_closed_when_cargo_doc_fails() {
    let temp = temp_dir("doc-rustdoc-failure");
    let target_dir = temp.join("target").join("custom-doc-target");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .env("FAKE_CARGO_STDERR", "rustdoc-bad")
        .env("FAKE_CARGO_EXIT", "7")
        .args([
            "doc",
            "rustdoc",
            "--cargo",
            env!("CARGO_BIN_EXE_fake-cargo"),
            "--rustdoc",
            "/tmp/rustdoc-fixture",
            "--target-dir",
            target_dir.to_str().expect("utf8 target"),
            "--keep-target-dir",
        ])
        .output()
        .expect("doc rustdoc command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("rustdoc-bad"), "stderr={stderr}");
    assert!(
        stderr.contains("rustdoc generation failed:") && stderr.contains("exit code 7"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_rustdoc_refuses_to_clean_unsafe_target_dir() {
    let temp = temp_dir("doc-rustdoc-unsafe");
    let unsafe_target = temp.join("not-generated-doc-target");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .env("FAKE_CARGO_STDOUT", "should-not-run")
        .args([
            "doc",
            "rustdoc",
            "--cargo",
            env!("CARGO_BIN_EXE_fake-cargo"),
            "--rustdoc",
            "/tmp/rustdoc-fixture",
            "--target-dir",
            unsafe_target.to_str().expect("utf8 target"),
        ])
        .output()
        .expect("doc rustdoc command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to clean non-generated rustdoc target dir"),
        "stderr={stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("should-not-run"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_rustdoc_refuses_parent_traversal_under_generated_target_dir() {
    let temp = temp_dir("doc-rustdoc-parent-traversal");
    let keepme = temp.join("target").join("keepme");
    fs::create_dir_all(&keepme).expect("keepme target dir");
    fs::write(keepme.join("sentinel"), "preserve").expect("sentinel written");
    let traversal_target = temp
        .join("target")
        .join("oya-rustdoc-check")
        .join("..")
        .join("keepme");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .env("FAKE_CARGO_STDOUT", "should-not-run")
        .args([
            "doc",
            "rustdoc",
            "--cargo",
            env!("CARGO_BIN_EXE_fake-cargo"),
            "--rustdoc",
            "/tmp/rustdoc-fixture",
            "--target-dir",
            traversal_target.to_str().expect("utf8 target"),
        ])
        .output()
        .expect("doc rustdoc command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("refusing to clean non-generated rustdoc target dir"),
        "stderr={stderr}"
    );
    assert!(
        keepme.join("sentinel").exists(),
        "parent traversal must not delete sibling target contents"
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("should-not-run"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_rustdoc_waits_for_target_lock_before_cleaning() {
    let temp = temp_dir("doc-rustdoc-lock");
    let target_dir = temp.join("target").join("oya-rustdoc-check");
    let lock_dir = temp.join("target").join("oya-rustdoc-check.lock");
    fs::create_dir_all(&target_dir).expect("target dir");
    fs::create_dir_all(&lock_dir).expect("lock dir");

    let releaser = {
        let lock_dir = lock_dir.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(350));
            fs::remove_dir(&lock_dir).expect("release rustdoc lock");
        })
    };

    let started = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .env("FAKE_CARGO_STDOUT", "cargo-after-lock")
        .args([
            "doc",
            "rustdoc",
            "--cargo",
            env!("CARGO_BIN_EXE_fake-cargo"),
            "--rustdoc",
            "/tmp/rustdoc-fixture",
            "--target-dir",
            target_dir.to_str().expect("utf8 target"),
        ])
        .output()
        .expect("doc rustdoc command runs");
    let elapsed = started.elapsed();
    releaser.join().expect("lock releaser joins");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        elapsed >= Duration::from_millis(300),
        "rustdoc command did not wait for the target lock: elapsed={elapsed:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo-after-lock"), "stdout={stdout}");

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_rustdoc_reclaims_stale_target_lock_owner() {
    let temp = temp_dir("doc-rustdoc-stale-lock");
    let target_dir = temp.join("target").join("oya-rustdoc-check");
    let lock_dir = temp.join("target").join("oya-rustdoc-check.lock");
    fs::create_dir_all(&target_dir).expect("target dir");
    fs::create_dir_all(&lock_dir).expect("lock dir");
    fs::write(lock_dir.join("owner"), "pid=99999999\n").expect("stale lock owner");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .env("FAKE_CARGO_STDOUT", "cargo-after-stale-lock")
        .args([
            "doc",
            "rustdoc",
            "--cargo",
            env!("CARGO_BIN_EXE_fake-cargo"),
            "--rustdoc",
            "/tmp/rustdoc-fixture",
            "--target-dir",
            target_dir.to_str().expect("utf8 target"),
        ])
        .output()
        .expect("doc rustdoc command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo-after-stale-lock"), "stdout={stdout}");

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_mdbook_validates_site_source_and_links() {
    let temp = temp_dir("doc-mdbook-success");
    write_mdbook_site(&temp, false);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "doc",
            "mdbook",
            "--site-dir",
            temp.to_str().expect("utf8 site"),
        ])
        .output()
        .expect("doc mdbook command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("mdbook source validation passed: 4 files, 2 chapters,"),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_mdbook_fails_closed_on_broken_local_link() {
    let temp = temp_dir("doc-mdbook-broken-link");
    write_mdbook_site(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "doc",
            "mdbook",
            "--site-dir",
            temp.to_str().expect("utf8 site"),
        ])
        .output()
        .expect("doc mdbook command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mdbook source validation failed:") && stderr.contains("BrokenLocalLink"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_validates_source_and_semver_metadata() {
    let temp = temp_dir("doc-openapi-success");
    write_openapi_contract(&temp, true, true);
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(
            "OpenAPI documentation validation passed: 1 documents, 1 operations, 33 data-class annotations, 1 runtime bindings, 1 runtime sources, 1 runtime tests, 3 runtime response statuses, 3 runtime response schemas, 7 schema bindings, 29 schema fields, 29 schema types, 1 contracts, 1 metadata records, 1 mirrored contracts"
        ),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}
#[test]
fn doc_openapi_validates_root_boundary_data_class_annotations() {
    let temp = temp_dir("doc-openapi-boundary-data-class-success");
    write_openapi_contract(&temp, true, true);
    write_boundary_openapi_contract(&temp, true);
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OpenAPI documentation validation passed: 2 documents"),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_boundary_property_without_data_class() {
    let temp = temp_dir("doc-openapi-boundary-data-class-missing");
    write_openapi_contract(&temp, true, true);
    write_boundary_openapi_contract(&temp, false);
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI boundary data-class invalid")
            && stderr.contains("contracts/ops-fixture-v1.openapi.yaml missing data class")
            && stderr.contains("schema FixtureBoundaryResponse.id"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_accepts_openapi_32_query_operations_through_cli() {
    let temp = temp_dir("doc-openapi-query-operation");
    write_openapi_contract(&temp, true, true);
    let contract_path = temp
        .join("openapi")
        .join("foundry")
        .join("capability-v1.yaml");
    let query_contract = fs::read_to_string(&contract_path)
        .expect("openapi contract readable")
        .replace("    post:\n", "    query:\n");
    fs::write(&contract_path, query_contract).expect("query operation contract written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OpenAPI documentation validation passed: 1 documents, 1 operations"),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_accepts_openapi_32_additional_operations_through_cli() {
    let temp = temp_dir("doc-openapi-additional-operation");
    write_openapi_contract(&temp, true, true);
    rewrite_openapi_contract(&temp, |contract| {
        rewrite_post_operation_as_additional_operation(contract, "COPY")
    });
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OpenAPI documentation validation passed: 1 documents, 1 operations"),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_additional_operations_fixed_method_collision() {
    let temp = temp_dir("doc-openapi-additional-operation-collision");
    write_openapi_contract(&temp, true, true);
    rewrite_openapi_contract(&temp, |contract| {
        rewrite_post_operation_as_additional_operation(contract, "QUERY")
    });
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("AdditionalOperationFixedMethodCollision")
            && stderr.contains("QUERY"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_missing_operation_id() {
    let temp = temp_dir("doc-openapi-missing-operation-id");
    write_openapi_contract(&temp, false, true);
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("MissingOperationId"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_missing_semver_metadata() {
    let temp = temp_dir("doc-openapi-missing-metadata");
    write_openapi_contract(&temp, true, false);
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("MissingMetadata"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_missing_contract_mirror() {
    let temp = temp_dir("doc-openapi-missing-mirror");
    write_openapi_contract(&temp, true, true);
    let (spec, mirror) = write_openapi_mirrors(&temp, false);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("MissingMachineMirror"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_missing_runtime_binding() {
    let temp = temp_dir("doc-openapi-missing-runtime-binding");
    write_openapi_contract(&temp, true, true);
    write_openapi_runtime_binding(&temp, false);
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("MissingRuntimeBinding"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_untyped_runtime_response_statuses() {
    let temp = temp_dir("doc-openapi-untyped-runtime-statuses");
    write_openapi_contract(&temp, true, true);
    let runtime_source = temp
        .join("crates")
        .join("oya-intelligence-api")
        .join("src")
        .join("lib.rs");
    fs::write(
        runtime_source,
        "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; let _accepted = 202; let _bad_request = 400; let _forbidden = 403; }\n",
    )
    .expect("untyped runtime source written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("MissingRuntimeStatusType")
            && stderr.contains("CapabilityInvokeApiStatus"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_invalid_runtime_status_type_mapping() {
    let temp = temp_dir("doc-openapi-invalid-runtime-status-type");
    write_openapi_contract(&temp, true, true);
    let runtime_source = temp
        .join("crates")
        .join("oya-intelligence-api")
        .join("src")
        .join("lib.rs");
    fs::write(
        runtime_source,
        "pub const CAPABILITY_INVOKE_SURFACE: &str = \"foundry.capability.invoke\";\n\
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }\n\
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Teapot => 418 } } }\n\
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }\n",
    )
    .expect("invalid runtime status source written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("InvalidRuntimeStatusType")
            && stderr.contains("Teapot"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_non_explicit_runtime_response_key() {
    let temp = temp_dir("doc-openapi-non-explicit-response-key");
    write_openapi_contract(&temp, true, true);
    let contract_path = temp
        .join("openapi")
        .join("foundry")
        .join("capability-v1.yaml");
    let invalid = fs::read_to_string(&contract_path)
        .expect("openapi contract readable")
        .replace("        '202':", "        '2XX':");
    fs::write(&contract_path, invalid).expect("range response contract written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("NonExplicitRuntimeResponseKey")
            && stderr.contains("2XX"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_missing_runtime_response_schema() {
    let temp = temp_dir("doc-openapi-missing-runtime-response-schema");
    write_openapi_contract(&temp, true, true);
    let contract_path = temp
        .join("openapi")
        .join("foundry")
        .join("capability-v1.yaml");
    let invalid = fs::read_to_string(&contract_path)
        .expect("openapi contract readable")
        .replacen(
            "          content:\n            application/json:\n              schema:\n                $ref: '#/components/schemas/CapabilityInvokeApiErrorResponse'\n",
            "",
            1,
        );
    fs::write(&contract_path, invalid).expect("missing response schema contract written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("MissingRuntimeResponseSchema")
            && stderr.contains("400"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_unexpected_runtime_response_schema() {
    let temp = temp_dir("doc-openapi-unexpected-runtime-response-schema");
    write_openapi_contract(&temp, true, true);
    let contract_path = temp
        .join("openapi")
        .join("foundry")
        .join("capability-v1.yaml");
    let invalid = fs::read_to_string(&contract_path)
        .expect("openapi contract readable")
        .replace(
            "                $ref: '#/components/schemas/CapabilityInvokeApiSuccessResponse'\n",
            "                $ref: '#/components/schemas/CapabilityInvokeApiErrorResponse'\n",
        );
    fs::write(&contract_path, invalid).expect("unexpected response schema contract written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("RuntimeResponseSchemaMismatch")
            && stderr.contains("202")
            && stderr.contains("CapabilityInvokeApiSuccessResponse"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_schema_shape_drift() {
    let temp = temp_dir("doc-openapi-schema-shape-drift");
    write_openapi_contract(&temp, true, true);
    let source_path = temp
        .join("crates")
        .join("oya-foundation-app")
        .join("src")
        .join("lib.rs");
    fs::write(
        source_path,
        "pub fn invoke_capability() { let _surface = \"foundry.capability.invoke\"; }\n\
pub struct CapabilityInvocationRequest {\n\
    pub tenant_id: String, // data_class: INTERNAL_ONLY\n\
    pub capability_id: String, // data_class: INTERNAL_ONLY\n\
    pub purpose: Purpose, // data_class: INTERNAL_ONLY\n\
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY\n\
    pub budget_window_id: String, // data_class: INTERNAL_ONLY\n\
    pub projected_cost_micros: u64, // data_class: INTERNAL_ONLY\n\
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY\n\
}\n\
pub struct InvocationReceipt {\n\
    pub tenant_id: String, // data_class: INTERNAL_ONLY\n\
    pub user_id: String, // data_class: INTERNAL_ONLY\n\
    pub capability_id: String, // data_class: INTERNAL_ONLY\n\
    pub evidence_event_hash: String, // data_class: INTERNAL_ONLY\n\
    pub cost_reservation_id: Option<String>, // data_class: INTERNAL_ONLY\n\
    pub cost_budget_warning: Option<BudgetWarning>, // data_class: INTERNAL_ONLY\n\
    pub run_id: Option<String>, // data_class: INTERNAL_ONLY\n\
    pub foundry_step_id: Option<String>, // data_class: INTERNAL_ONLY\n\
    pub foundry_evidence_id: Option<String>, // data_class: INTERNAL_ONLY\n\
}\n",
    )
    .expect("drifted runtime source written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("SchemaFieldMismatch")
            && stderr.contains("user_id"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_openapi_fails_closed_on_schema_type_drift() {
    let temp = temp_dir("doc-openapi-schema-type-drift");
    write_openapi_contract(&temp, true, true);
    let contract_path = temp
        .join("openapi")
        .join("foundry")
        .join("capability-v1.yaml");
    let invalid = fs::read_to_string(&contract_path)
        .expect("openapi contract readable")
        .replacen("          format: uint64\n", "          format: int64\n", 1);
    fs::write(&contract_path, invalid).expect("drifted openapi contract written");
    let (spec, mirror) = write_openapi_mirrors(&temp, true);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(openapi_args(&temp, &spec, &mirror))
        .output()
        .expect("doc openapi command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OpenAPI documentation validation failed:")
            && stderr.contains("SchemaTypeMismatch")
            && stderr.contains("projected_cost_micros")
            && stderr.contains("uint64")
            && stderr.contains("int64"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_adr_index_writes_then_checks_generated_artifacts() {
    let temp = temp_dir("doc-adr-index-write-check");
    let decisions = temp.join("decisions");
    write_adr(
        &decisions,
        "ADR-0001-first-decision.md",
        "ADR-0001",
        "First Decision",
        "Proposed",
    );
    write_adr(
        &decisions,
        "ADR-0002-second-decision.md",
        "ADR-0002",
        "Second Decision",
        "Accepted",
    );
    let index = temp.join("ADR-INDEX.md");
    let machine = temp.join("decisions.json");

    let write = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "doc",
            "adr-index",
            "--decisions-dir",
            decisions.to_str().expect("utf8 decisions"),
            "--index",
            index.to_str().expect("utf8 index"),
            "--machine",
            machine.to_str().expect("utf8 machine"),
            "--write",
        ])
        .output()
        .expect("adr-index write command runs");

    assert!(
        write.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&write.stdout),
        String::from_utf8_lossy(&write.stderr)
    );
    let markdown = fs::read_to_string(&index).expect("index written");
    let json = fs::read_to_string(&machine).expect("machine mirror written");
    assert!(markdown.contains("Proposed 1"), "markdown={markdown}");
    assert!(markdown.contains("Accepted 1"), "markdown={markdown}");
    assert!(json.contains("\"next_adr\": \"ADR-0003\""), "json={json}");

    let check = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "doc",
            "adr-index",
            "--decisions-dir",
            decisions.to_str().expect("utf8 decisions"),
            "--index",
            index.to_str().expect("utf8 index"),
            "--machine",
            machine.to_str().expect("utf8 machine"),
        ])
        .output()
        .expect("adr-index check command runs");

    assert!(
        check.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(
        String::from_utf8_lossy(&check.stdout).contains("ADR index checked: 2 records"),
        "stdout={}",
        String::from_utf8_lossy(&check.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_adr_index_reconciles_amended_frontmatter_with_both_projections() {
    let temp = temp_dir("doc-adr-index-amended-status");
    let decisions = temp.join("decisions");
    let source = decisions.join("ADR-0001-amended-decision.md");
    fs::create_dir_all(&decisions).expect("decisions dir created");
    fs::write(
        &source,
        "---\n\
         id: ADR-0001\n\
         title: Amended Decision Fixture\n\
         status: Accepted\n\
         date: 2026-07-20\n\
         owner: council-architecture\n\
         ---\n\n\
         # ADR-0001: Amended Decision Fixture\n\n\
         ## Context\n\n\
         Fixture.\n",
    )
    .expect("accepted ADR written");
    let index = temp.join("ADR-INDEX.md");
    let machine = temp.join("decisions.json");
    let args = [
        "doc",
        "adr-index",
        "--decisions-dir",
        decisions.to_str().expect("utf8 decisions"),
        "--index",
        index.to_str().expect("utf8 index"),
        "--machine",
        machine.to_str().expect("utf8 machine"),
    ];

    let initial = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(args.iter().copied().chain(["--write"]))
        .output()
        .expect("initial adr-index write runs");
    assert!(
        initial.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&initial.stderr)
    );

    let amended = fs::read_to_string(&source)
        .expect("source ADR readable")
        .replace("status: Accepted", "status: Amended");
    fs::write(&source, amended).expect("amended ADR written");

    let stale = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(args)
        .output()
        .expect("adr-index detects stale projections");
    assert!(!stale.status.success());
    assert!(
        String::from_utf8_lossy(&stale.stderr).contains("MarkdownDrift"),
        "stderr={}",
        String::from_utf8_lossy(&stale.stderr)
    );

    let regenerated = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(args.iter().copied().chain(["--write"]))
        .output()
        .expect("amended adr-index write runs");
    assert!(
        regenerated.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&regenerated.stderr)
    );
    assert!(
        fs::read_to_string(&index)
            .expect("index readable")
            .contains("| ADR-0001 | Amended |"),
        "human index must project the parsed amended status"
    );
    assert!(
        fs::read_to_string(&machine)
            .expect("machine mirror readable")
            .contains("\"status\": \"Amended\""),
        "machine mirror must project the parsed amended status"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_adr_index_accepts_decision_owner_frontmatter() {
    let temp = temp_dir("doc-adr-index-decision-owner");
    let decisions = temp.join("decisions");
    fs::create_dir_all(&decisions).expect("decisions dir created");
    fs::write(
        decisions.join("ADR-0001-decision-owner.md"),
        "---\n\
         id: ADR-0001\n\
         title: Decision Owner Fixture\n\
         status: Accepted\n\
         date: 2026-05-24\n\
         decision_owner: council-architecture\n\
         ---\n\n\
         # ADR-0001: Decision Owner Fixture\n\n\
         ## Context\n\n\
         Fixture.\n",
    )
    .expect("decision_owner ADR written");
    let index = temp.join("ADR-INDEX.md");
    let machine = temp.join("decisions.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "doc",
            "adr-index",
            "--decisions-dir",
            decisions.to_str().expect("utf8 decisions"),
            "--index",
            index.to_str().expect("utf8 index"),
            "--machine",
            machine.to_str().expect("utf8 machine"),
            "--write",
        ])
        .output()
        .expect("adr-index write command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = fs::read_to_string(&machine).expect("machine mirror written");
    assert!(
        json.contains("\"owner\": \"council-architecture\""),
        "json={json}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_adr_index_prefers_base_decision_over_duplicate_amendment_file() {
    let temp = temp_dir("doc-adr-index-duplicate-amendment");
    let decisions = temp.join("decisions");
    write_adr(
        &decisions,
        "ADR-0001-base-decision.md",
        "ADR-0001",
        "Base Decision",
        "Accepted",
    );
    write_adr(
        &decisions,
        "ADR-0001-amendment-clarification.md",
        "ADR-0001",
        "Amendment Clarification",
        "Accepted",
    );
    let index = temp.join("ADR-INDEX.md");
    let machine = temp.join("decisions.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "doc",
            "adr-index",
            "--decisions-dir",
            decisions.to_str().expect("utf8 decisions"),
            "--index",
            index.to_str().expect("utf8 index"),
            "--machine",
            machine.to_str().expect("utf8 machine"),
            "--write",
        ])
        .output()
        .expect("adr-index write command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json = fs::read_to_string(&machine).expect("machine mirror written");
    assert!(json.contains("Base Decision"), "json={json}");
    assert!(!json.contains("Amendment Clarification"), "json={json}");

    fs::remove_dir_all(temp).ok();
}

#[test]
fn doc_adr_index_fails_when_committed_artifacts_drift() {
    let temp = temp_dir("doc-adr-index-drift");
    let decisions = temp.join("decisions");
    write_adr(
        &decisions,
        "ADR-0001-first-decision.md",
        "ADR-0001",
        "First Decision",
        "Proposed",
    );
    let index = temp.join("ADR-INDEX.md");
    let machine = temp.join("decisions.json");
    fs::write(&index, "stale markdown\n").expect("stale index written");
    fs::write(&machine, "{}\n").expect("stale mirror written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "doc",
            "adr-index",
            "--decisions-dir",
            decisions.to_str().expect("utf8 decisions"),
            "--index",
            index.to_str().expect("utf8 index"),
            "--machine",
            machine.to_str().expect("utf8 machine"),
        ])
        .output()
        .expect("adr-index check command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ADR index validation failed:") && stderr.contains("MarkdownDrift"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

fn rewrite_openapi_contract<F>(root: &Path, rewrite: F)
where
    F: FnOnce(&str) -> String,
{
    let contract_path = root
        .join("openapi")
        .join("foundry")
        .join("capability-v1.yaml");
    let contract = fs::read_to_string(&contract_path).expect("openapi contract readable");
    fs::write(&contract_path, rewrite(&contract)).expect("rewritten openapi contract written");
}

fn rewrite_post_operation_as_additional_operation(contract: &str, method: &str) -> String {
    let post_marker = "    post:\n";
    let post_index = contract
        .find(post_marker)
        .expect("fixture has post operation");
    let operation_start = post_index + post_marker.len();
    let components_index = contract
        .find("components:\n")
        .expect("fixture has components block");

    let mut rewritten = String::new();
    rewritten.push_str(&contract[..post_index]);
    rewritten.push_str("    additionalOperations:\n");
    rewritten.push_str(&format!("      {method}:\n"));
    for line in contract[operation_start..components_index].split_inclusive('\n') {
        if line.trim().is_empty() {
            rewritten.push_str(line);
        } else {
            rewritten.push_str("  ");
            rewritten.push_str(line);
        }
    }
    rewritten.push_str(&contract[components_index..]);
    rewritten
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("specs/masterplan.json").is_file()
                && candidate.join("HANDOFF.md").is_file()
        })
        .expect("repo root")
        .to_path_buf()
}

fn write_boundary_openapi_contract(root: &Path, include_property_data_class: bool) {
    let property_data_class = if include_property_data_class {
        "          x-oyatie-data-class: INTERNAL_ONLY\n"
    } else {
        ""
    };
    fs::write(
        root.join("ops-fixture-v1.openapi.yaml"),
        format!(
            r##"openapi: 3.2.0
info:
  title: Ops Fixture
  version: 1.0.0
paths:
  /ops/fixture:
    get:
      operationId: getOpsFixture
      parameters:
        - in: query
          name: tenant
          required: false
          schema:
            type: string
          x-oyatie-data-class: INTERNAL_ONLY
      responses:
        "200":
          description: Fixture response
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/FixtureBoundaryResponse"
components:
  schemas:
    FixtureBoundaryResponse:
      type: object
      additionalProperties: false
      required: [id, emitted_at_unix_ms]
      properties:
        id:
          type: string
{property_data_class}        emitted_at_unix_ms:
          type: integer
          format: int64
          x-oyatie-data-class: INTERNAL_ONLY
"##
        ),
    )
    .expect("boundary openapi contract written");
    fs::write(
        root.join("ops-fixture-v1.openapi.meta.yaml"),
        "tier: preview\nowner_team: axis-ops\nversion: 1.0.0\nsunset: none\nrelated_adrs:\n  - ADR-0067\n",
    )
    .expect("boundary openapi metadata written");
}
fn write_openapi_contract(root: &Path, include_operation_id: bool, include_metadata: bool) {
    let openapi_dir = root.join("openapi").join("foundry");
    fs::create_dir_all(&openapi_dir).expect("openapi dirs created");
    let mut contract =
        fs::read_to_string(repo_root().join("contracts/openapi/foundry/capability-v1.yaml"))
            .expect("openapi capability contract is readable");
    if !include_operation_id {
        contract = contract.replace("      operationId: invokeCapability\n", "");
    }
    fs::write(openapi_dir.join("capability-v1.yaml"), contract).expect("openapi contract written");
    write_openapi_runtime_binding(root, true);
    write_openapi_schema_binding(root, true);
    if include_metadata {
        fs::write(
            openapi_dir.join("capability-v1.meta.yaml"),
            "tier: preview\nowner_team: axis-foundry\nversion: 1.0.0\nsunset: none\nrelated_adrs:\n  - ADR-0037\n",
        )
        .expect("openapi metadata written");
    }
}

fn write_openapi_runtime_binding(root: &Path, include_binding: bool) {
    let registry_dir = root.join("registry").join("openapi");
    fs::create_dir_all(&registry_dir).expect("openapi runtime registry dirs created");
    let row = if include_binding {
        "invokeCapability\tcontracts/openapi/foundry/capability-v1.yaml\toya-intelligence-api\tcrates/oya-intelligence-api/src/lib.rs\tinvoke_capability_from_api\tCapabilityInvokeApiStatus\tfoundry.capability.invoke\tcrates/oya-intelligence-api/tests/capability_invoke_api.rs\t202=CapabilityInvokeApiSuccessResponse;400=CapabilityInvokeApiErrorResponse;403=CapabilityInvokeApiErrorResponse\n"
    } else {
        ""
    };
    fs::write(
        registry_dir.join("runtime-bindings.tsv"),
        format!(
            "operation_id\tcontract_path\truntime_crate\tsource_path\tsymbol\tstatus_type\tevidence_surface\ttest_path\tresponse_schemas\n{row}"
        ),
    )
    .expect("openapi runtime binding registry written");

    let api_source_path = root.join("crates").join("oya-intelligence-api").join("src");
    fs::create_dir_all(&api_source_path).expect("runtime source dirs created");
    fs::write(
        api_source_path.join("lib.rs"),
        r#"pub const CAPABILITY_INVOKE_SURFACE: &str = "foundry.capability.invoke";
pub enum CapabilityInvokeApiStatus { Accepted, BadRequest, Forbidden }
impl CapabilityInvokeApiStatus { pub const fn code(self) -> u16 { match self { Self::Accepted => 202, Self::BadRequest => 400, Self::Forbidden => 403 } } }
pub struct CapabilityInvokeApiSuccessResponse {
    pub data: CapabilityInvocationReceipt, // data_class: INTERNAL_ONLY
    pub metadata: CapabilityInvokeApiResponseMetadata, // data_class: INTERNAL_ONLY
}
pub struct CapabilityInvokeApiResponseMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}
pub struct CapabilityInvokeApiErrorResponse {
    pub error: CapabilityInvokeApiErrorBody, // data_class: INTERNAL_ONLY
}
pub struct CapabilityInvokeApiErrorBody {
    pub code: String, // data_class: INTERNAL_ONLY
    pub message: String, // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>, // data_class: INTERNAL_ONLY
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub details: Vec<CapabilityInvokeApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}
pub struct CapabilityInvokeApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}
pub fn invoke_capability_from_api() { let _surface = CAPABILITY_INVOKE_SURFACE; }
"#,
    )
    .expect("runtime source fixture written");

    let schema_source_path = root.join("crates").join("oya-foundation-app").join("src");
    fs::create_dir_all(&schema_source_path).expect("schema source dirs created");
    fs::write(
        schema_source_path.join("lib.rs"),
        "pub struct CapabilityInvocationRequest {\n\
    pub tenant_id: String, // data_class: INTERNAL_ONLY\n\
    pub user_id: String, // data_class: INTERNAL_ONLY\n\
    pub capability_id: String, // data_class: INTERNAL_ONLY\n\
    pub purpose: Purpose, // data_class: INTERNAL_ONLY\n\
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY\n\
    pub budget_window_id: String, // data_class: INTERNAL_ONLY\n\
    pub projected_cost_micros: u64, // data_class: INTERNAL_ONLY\n\
    pub started_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY\n\
}\n\
pub struct InvocationReceipt {\n\
    pub tenant_id: String, // data_class: INTERNAL_ONLY\n\
    pub user_id: String, // data_class: INTERNAL_ONLY\n\
    pub capability_id: String, // data_class: INTERNAL_ONLY\n\
    pub evidence_event_hash: String, // data_class: INTERNAL_ONLY\n\
    pub cost_reservation_id: Option<String>, // data_class: INTERNAL_ONLY\n\
    pub cost_budget_warning: Option<BudgetWarning>, // data_class: INTERNAL_ONLY\n\
    pub run_id: Option<String>, // data_class: INTERNAL_ONLY\n\
    pub foundry_step_id: Option<String>, // data_class: INTERNAL_ONLY\n\
    pub foundry_evidence_id: Option<String>, // data_class: INTERNAL_ONLY\n\
}\n",
    )
    .expect("schema source fixture written");

    let test_path = root
        .join("crates")
        .join("oya-intelligence-api")
        .join("tests");
    fs::create_dir_all(&test_path).expect("runtime test dirs created");
    fs::write(
        test_path.join("capability_invoke_api.rs"),
        "invoke_capability_from_api(foundation, request); assert_eq!(surface, \"foundry.capability.invoke\"); assert_eq!(CapabilityInvokeApiStatus::Accepted.code(), 202); assert_eq!(CapabilityInvokeApiStatus::BadRequest.code(), 400); assert_eq!(CapabilityInvokeApiStatus::Forbidden.code(), 403);\n",
    )
    .expect("runtime test fixture written");
}

fn write_openapi_schema_binding(root: &Path, include_binding: bool) {
    let registry_dir = root.join("registry").join("openapi");
    fs::create_dir_all(&registry_dir).expect("openapi schema registry dirs created");
    let rows = if include_binding {
        "CapabilityInvocationRequest\tcontracts/openapi/foundry/capability-v1.yaml\toya-foundation-app\tcrates/oya-foundation-app/src/lib.rs\tCapabilityInvocationRequest\nCapabilityInvocationReceipt\tcontracts/openapi/foundry/capability-v1.yaml\toya-foundation-app\tcrates/oya-foundation-app/src/lib.rs\tInvocationReceipt\nCapabilityInvokeApiSuccessResponse\tcontracts/openapi/foundry/capability-v1.yaml\toya-intelligence-api\tcrates/oya-intelligence-api/src/lib.rs\tCapabilityInvokeApiSuccessResponse\nCapabilityInvokeApiResponseMetadata\tcontracts/openapi/foundry/capability-v1.yaml\toya-intelligence-api\tcrates/oya-intelligence-api/src/lib.rs\tCapabilityInvokeApiResponseMetadata\nCapabilityInvokeApiErrorResponse\tcontracts/openapi/foundry/capability-v1.yaml\toya-intelligence-api\tcrates/oya-intelligence-api/src/lib.rs\tCapabilityInvokeApiErrorResponse\nCapabilityInvokeApiErrorBody\tcontracts/openapi/foundry/capability-v1.yaml\toya-intelligence-api\tcrates/oya-intelligence-api/src/lib.rs\tCapabilityInvokeApiErrorBody\nCapabilityInvokeApiErrorDetail\tcontracts/openapi/foundry/capability-v1.yaml\toya-intelligence-api\tcrates/oya-intelligence-api/src/lib.rs\tCapabilityInvokeApiErrorDetail\n"
    } else {
        ""
    };
    fs::write(
        registry_dir.join("schema-bindings.tsv"),
        format!("schema_name\tcontract_path\truntime_crate\tsource_path\trust_struct\n{rows}"),
    )
    .expect("openapi schema binding registry written");
}

fn write_openapi_mirrors(root: &Path, include_machine_path: bool) -> (PathBuf, PathBuf) {
    let spec = root.join("SPEC.md");
    let mirror = root.join("contracts.json");
    fs::write(
        &spec,
        "OpenAPI source: contracts/openapi/foundry/capability-v1.yaml\n",
    )
    .expect("SPEC mirror written");
    let location = if include_machine_path {
        "crates/oya-intelligence-api + contracts/openapi/foundry/capability-v1.yaml"
    } else {
        "crates/oya-intelligence-api"
    };
    fs::write(
        &mirror,
        format!(
            r#"{{
  "cross_axis_contracts": [
    {{
      "id": "CAPABILITY_INVOCATION",
      "owner_axis": "foundry",
      "consumer_axes": ["all"],
      "location": "{location}",
      "change_review": "foundry + consuming-axis"
    }}
  ]
}}"#
        ),
    )
    .expect("contracts mirror written");
    (spec, mirror)
}

fn openapi_args(contracts: &Path, spec: &Path, mirror: &Path) -> Vec<String> {
    vec![
        "doc".into(),
        "openapi".into(),
        "--contracts-dir".into(),
        contracts.to_str().expect("utf8 contracts").into(),
        "--spec".into(),
        spec.to_str().expect("utf8 spec").into(),
        "--contracts-mirror".into(),
        mirror.to_str().expect("utf8 mirror").into(),
        "--runtime-bindings".into(),
        contracts
            .join("registry")
            .join("openapi")
            .join("runtime-bindings.tsv")
            .to_str()
            .expect("utf8 runtime bindings")
            .into(),
        "--schema-bindings".into(),
        contracts
            .join("registry")
            .join("openapi")
            .join("schema-bindings.tsv")
            .to_str()
            .expect("utf8 schema bindings")
            .into(),
        "--runtime-root".into(),
        contracts.to_str().expect("utf8 runtime root").into(),
    ]
}

fn write_mdbook_site(root: &Path, broken_link: bool) {
    fs::create_dir_all(root.join("src").join("guide")).expect("mdbook dirs created");
    fs::write(
        root.join("book.toml"),
        "[book]\ntitle = \"Fixture docs\"\nsrc = \"src\"\n",
    )
    .expect("book manifest written");
    fs::write(
        root.join("src").join("SUMMARY.md"),
        "# Summary\n\n- [Start](start.md)\n- [Guide](guide/admin.md)\n",
    )
    .expect("summary written");
    let guide_target = if broken_link {
        "guide/missing.md"
    } else {
        "guide/admin.md"
    };
    fs::write(
        root.join("src").join("start.md"),
        format!("# Start\n\nContinue to the [guide]({guide_target}).\n"),
    )
    .expect("start chapter written");
    fs::write(
        root.join("src").join("guide").join("admin.md"),
        "# Guide\n\nReturn [home](../start.md).\n",
    )
    .expect("guide chapter written");
}

fn write_adr(root: &Path, name: &str, id: &str, title: &str, status: &str) -> PathBuf {
    fs::create_dir_all(root).expect("decisions dir created");
    let path = root.join(name);
    fs::write(
        &path,
        format!(
            "# {id}: {title}\n\n> **Status:** {status}\n> **Supersedes:** -\n> **Superseded-by:** -\n> **Owner:** `council-architecture`\n> **Date:** 2026-05-09\n> **Related:** -\n\n---\n\n## Context\n\nFixture.\n"
        ),
    )
    .expect("adr written");
    path
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
