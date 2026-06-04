#[allow(dead_code)]
#[path = "../ci/enforce-buck2-authority.rs"]
mod checker;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

use checker::{Config, evaluate};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    dir.push(format!(
        "oya-{label}-{}-{nanos}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn replace_json_value(text: &str, key: &str, replacement: &str) -> String {
    let needle = format!("\"{key}\"");
    let key_start = text
        .find(&needle)
        .unwrap_or_else(|| panic!("missing key {key}"));
    let colon = text[key_start + needle.len()..]
        .find(':')
        .map(|offset| key_start + needle.len() + offset)
        .unwrap();
    let mut value_start = colon + 1;
    while text[value_start..].chars().next().unwrap().is_whitespace() {
        value_start += text[value_start..].chars().next().unwrap().len_utf8();
    }
    let open = text[value_start..].chars().next().unwrap();
    let close = match open {
        '[' => ']',
        '{' => '}',
        '"' => '"',
        't' | 'f' => ',',
        _ => panic!("unsupported json value start {open}"),
    };
    let value_end = if open == '"' {
        let mut index = value_start + 1;
        let mut escaped = false;
        loop {
            let ch = text[index..].chars().next().unwrap();
            index += ch.len_utf8();
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                break index;
            }
        }
    } else if open == 't' || open == 'f' {
        let bool_len = if text[value_start..].starts_with("true") {
            4
        } else {
            5
        };
        value_start + bool_len
    } else {
        let mut depth = 0i32;
        let mut index = value_start;
        let mut in_string = false;
        let mut escaped = false;
        loop {
            let ch = text[index..].chars().next().unwrap();
            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                index += ch.len_utf8();
                continue;
            }
            if ch == '"' {
                in_string = true;
            } else if ch == open {
                depth += 1;
            } else if ch == close {
                depth -= 1;
                if depth == 0 {
                    index += ch.len_utf8();
                    break index;
                }
            }
            index += ch.len_utf8();
        }
    };
    format!(
        "{}{}{}",
        &text[..value_start],
        replacement,
        &text[value_end..]
    )
}

fn first_object_in_array(text: &str, key: &str) -> String {
    let needle = format!("\"{key}\"");
    let key_start = text.find(&needle).unwrap();
    let open = text[key_start..]
        .find('[')
        .map(|offset| key_start + offset)
        .unwrap();
    let object_start = text[open..].find('{').map(|offset| open + offset).unwrap();
    let mut depth = 0i32;
    let mut index = object_start;
    let mut in_string = false;
    let mut escaped = false;
    loop {
        let ch = text[index..].chars().next().unwrap();
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += ch.len_utf8();
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                index += ch.len_utf8();
                break;
            }
        }
        index += ch.len_utf8();
    }
    text[object_start..index].to_owned()
}

fn append_object_to_array(text: &str, key: &str, object: &str) -> String {
    let needle = format!("\"{key}\"");
    let key_start = text.find(&needle).unwrap();
    let open = text[key_start..]
        .find('[')
        .map(|offset| key_start + offset)
        .unwrap();
    let mut depth = 0i32;
    let mut index = open;
    let mut in_string = false;
    let mut escaped = false;
    loop {
        let ch = text[index..].chars().next().unwrap();
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += ch.len_utf8();
            continue;
        }
        if ch == '"' {
            in_string = true;
        } else if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        index += ch.len_utf8();
    }
    format!("{},{}{}", &text[..index], object, &text[index..])
}

struct Fixture {
    root: PathBuf,
    dir: PathBuf,
    config: Config,
}

impl Fixture {
    fn new() -> Self {
        let root = repo_root();
        let dir = unique_temp_dir("buck2-authority-policy");
        let config = Config {
            policy: dir
                .join("buck2-authority-policy.json")
                .to_string_lossy()
                .into_owned(),
            matrix: dir
                .join("phase0-automation-matrix.json")
                .to_string_lossy()
                .into_owned(),
            coverage_registry: dir
                .join("phase0-automation-coverage-registry.json")
                .to_string_lossy()
                .into_owned(),
            prow_parity_registry: dir
                .join("oya-ci-prow-capability-parity.json")
                .to_string_lossy()
                .into_owned(),
            root_hub: dir
                .join("root-hub-pointers.json")
                .to_string_lossy()
                .into_owned(),
        };
        let policy = fs::read_to_string(root.join("specs/buck2-authority-policy.json")).unwrap();
        let policy = replace_json_value(&policy, "command_scan_files", "[]");
        let policy = replace_json_value(&policy, "command_scan_globs", "[]");
        let policy = replace_json_value(&policy, "status_context_scan_files", "[]");
        let policy = replace_json_value(&policy, "required_anchors", "{}");
        let policy = replace_json_value(&policy, "required_glob_anchors", "[]");
        let policy = replace_json_value(&policy, "adr_amendment_files", "[]");
        fs::write(&config.policy, policy).unwrap();
        for (src, dst) in [
            ("specs/phase0-automation-matrix.json", &config.matrix),
            (
                "specs/phase0-automation-coverage-registry.json",
                &config.coverage_registry,
            ),
            (
                "specs/oya-ci-prow-capability-parity.json",
                &config.prow_parity_registry,
            ),
            ("specs/root-hub-pointers.json", &config.root_hub),
        ] {
            fs::copy(root.join(src), Path::new(dst)).unwrap();
        }
        Self { root, dir, config }
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn write(&self, path: &str, text: String) {
        fs::write(path, text).unwrap();
    }

    fn evaluate(&self) -> checker::Evaluation {
        evaluate(&self.root, &self.config)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn assert_fails_with(label: &str, expected: &str, fixture: &Fixture) {
    let evaluation = fixture.evaluate();
    assert_eq!(evaluation.verdict, "FAIL", "{label} should fail");
    let joined = evaluation.failures.join("\n");
    assert!(
        joined.contains(expected),
        "{label} missing {expected:?}; failures:\n{joined}"
    );
}

#[test]
fn checked_in_contract_passes_in_fixture_mode() {
    let fixture = Fixture::new();
    let evaluation = fixture.evaluate();
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.authority_context, "oya-ci-required");
}

#[test]
fn missing_required_capability_fails() {
    let fixture = Fixture::new();
    let text = fixture
        .read(&fixture.config.prow_parity_registry)
        .replace("\"prow-tide-merge-automation\",", "");
    fixture.write(&fixture.config.prow_parity_registry, text);
    assert_fails_with(
        "missing_required_capability",
        "required_capability_ids missing prow-tide-merge-automation",
        &fixture,
    );
}

#[test]
fn duplicate_capability_fails() {
    let fixture = Fixture::new();
    let text = fixture.read(&fixture.config.prow_parity_registry);
    let first = first_object_in_array(&text, "capabilities");
    fixture.write(
        &fixture.config.prow_parity_registry,
        append_object_to_array(&text, "capabilities", &first),
    );
    assert_fails_with(
        "duplicate_capability",
        "capabilities must have unique string ids",
        &fixture,
    );
}

#[test]
fn live_authority_claim_fails() {
    let fixture = Fixture::new();
    let text = fixture.read(&fixture.config.prow_parity_registry).replacen(
        "\"live_authority_claimed\": false",
        "\"live_authority_claimed\": true",
        1,
    );
    fixture.write(&fixture.config.prow_parity_registry, text);
    assert_fails_with(
        "live_authority_claim",
        "live_authority_claimed must be false",
        &fixture,
    );
}

#[test]
fn wrong_required_context_fails() {
    let fixture = Fixture::new();
    let text = fixture.read(&fixture.config.policy).replace(
        "\"required_context\": \"oya-ci-required\"",
        "\"required_context\": \"cargo-ci-required\"",
    );
    fixture.write(&fixture.config.policy, text);
    assert_fails_with(
        "wrong_required_context",
        "target_authority.required_context must be oya-ci-required",
        &fixture,
    );
}

#[test]
fn false_production_claim_fails() {
    let fixture = Fixture::new();
    let text = fixture
        .read(&fixture.config.prow_parity_registry)
        .replace(
            "\"production_readiness\": false",
            "\"production_readiness\": true",
        )
        .replace(
            "\"hyperscaler_grade_readiness\": false",
            "\"hyperscaler_grade_readiness\": true",
        );
    fixture.write(&fixture.config.prow_parity_registry, text);
    assert_fails_with(
        "false_production_claim",
        "claim_boundary.production_readiness must be false",
        &fixture,
    );
}

#[test]
fn missing_excluded_component_fails() {
    let fixture = Fixture::new();
    let text = fixture
        .read(&fixture.config.prow_parity_registry)
        .replace("\"prow-gcsupload\"", "\"prow-gcsupload-removed\"");
    fixture.write(&fixture.config.prow_parity_registry, text);
    assert_fails_with(
        "missing_excluded_component",
        "excluded_or_superseded_upstream_components missing prow-gcsupload",
        &fixture,
    );
}

#[test]
fn missing_source_bound_producer_fails() {
    let fixture = Fixture::new();
    let text = fixture
        .read(&fixture.config.policy)
        .replace(" trusted source-bound bridge", " trusted bridge");
    fixture.write(&fixture.config.policy, text);
    assert_fails_with(
        "missing_source_bound_producer",
        "target_authority.producer must contain 'source-bound'",
        &fixture,
    );
}

#[test]
fn stale_plank_primary_fails() {
    let fixture = Fixture::new();
    let text = fixture.read(&fixture.config.prow_parity_registry).replace(
        "prow-controller-manager-job-controller",
        "prow-plank-job-controller",
    );
    fixture.write(&fixture.config.prow_parity_registry, text);
    assert_fails_with(
        "stale_plank_primary",
        "required_capability_ids missing prow-controller-manager-job-controller",
        &fixture,
    );
}

#[test]
fn missing_root_pointer_fails() {
    let fixture = Fixture::new();
    let text = fixture.read(&fixture.config.root_hub).replace(
        "oya_ci_prow_capability_parity",
        "removed_ci_prow_capability_parity",
    );
    fixture.write(&fixture.config.root_hub, text);
    assert_fails_with(
        "missing_root_pointer",
        "entry_points must include oya_ci_prow_capability_parity",
        &fixture,
    );
}
