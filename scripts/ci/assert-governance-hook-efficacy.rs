use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Debug)]
struct GateResult {
    id: &'static str,
    failures: Vec<String>,
}

impl GateResult {
    fn pass(id: &'static str) -> Self {
        Self {
            id,
            failures: Vec::new(),
        }
    }

    fn fail(id: &'static str, message: impl Into<String>) -> Self {
        Self {
            id,
            failures: vec![message.into()],
        }
    }

    fn passed(&self) -> bool {
        self.failures.is_empty()
    }
}

fn vacuous_green_issues(content: &str) -> Vec<&'static str> {
    let mut issues = Vec::new();
    if content.contains("assert!(true)") {
        issues.push("assert!(true)");
    }
    if content.lines().any(|line| line.trim() == "Ok(())") {
        issues.push("Ok(())-only-body");
    }
    if content.contains("#[test]")
        && !["assert", "expect", "panic", "Err"]
            .iter()
            .any(|needle| content.contains(needle))
    {
        issues.push("zero-assertion-tests");
    }
    issues
}

fn extract_adr_refs(content: &str) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    let bytes = content.as_bytes();
    let mut index = 0;
    while index + 8 <= bytes.len() {
        if &bytes[index..index + 4] == b"ADR-" {
            let digits = &bytes[index + 4..index + 8];
            if digits.iter().all(u8::is_ascii_digit) {
                refs.insert(format!("ADR-{}", String::from_utf8_lossy(digits).as_ref()));
                index += 8;
                continue;
            }
        }
        index += 1;
    }
    refs
}

fn adr_exists(decisions_dir: &Path, adr_ref: &str) -> io::Result<bool> {
    let prefix = format!("{adr_ref}-");
    for entry in fs::read_dir(decisions_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with(&prefix) && file_name.ends_with(".md") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn orphan_adr_refs(content: &str, decisions_dir: &Path) -> io::Result<Vec<String>> {
    let mut orphans = Vec::new();
    for adr_ref in extract_adr_refs(content) {
        if !adr_exists(decisions_dir, &adr_ref)? {
            orphans.push(adr_ref);
        }
    }
    Ok(orphans)
}

fn extract_semver_after_key(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix(key)?.trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let rest = rest.trim_matches(|ch| ch == '\'' || ch == '"');
    let version: String = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect();
    let mut parts = version.split('.');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(major), Some(minor), Some(patch), None)
            if !major.is_empty()
                && !minor.is_empty()
                && !patch.is_empty()
                && major.chars().all(|ch| ch.is_ascii_digit())
                && minor.chars().all(|ch| ch.is_ascii_digit())
                && patch.chars().all(|ch| ch.is_ascii_digit()) =>
        {
            Some(version)
        }
        _ => None,
    }
}

fn spec_version_findings(content: &str) -> Vec<String> {
    let mut findings = Vec::new();
    for line in content.lines() {
        if let Some(version) = extract_semver_after_key(line, "openapi") {
            if version != "3.2.0" {
                findings.push(format!("openapi {version} is not canonical 3.2.0"));
            }
        }
        if let Some(version) = extract_semver_after_key(line, "asyncapi") {
            if version != "3.1.0" {
                findings.push(format!("asyncapi {version} is not canonical 3.1.0"));
            }
        }
    }
    findings
}

fn substantive_line_count(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("<!--")
        })
        .count()
}

fn buildability_findings(content: &str) -> Vec<String> {
    let count = substantive_line_count(content);
    if count < 50 {
        vec![format!("has {count} substantive lines; minimum is 50")]
    } else {
        Vec::new()
    }
}

fn evaluate(repo_root: &Path) -> Vec<GateResult> {
    let mut results = Vec::new();

    let vacuous_bad = "#[test]\nfn vacuous() {\n    assert!(true);\n}\n";
    let vacuous_good = "#[test]\nfn real_assertion() {\n    assert_eq!(2 + 2, 4);\n}\n";
    let bad_issues = vacuous_green_issues(vacuous_bad);
    let good_issues = vacuous_green_issues(vacuous_good);
    if bad_issues.contains(&"assert!(true)") && good_issues.is_empty() {
        results.push(GateResult::pass("oya-governance-vacuous-green"));
    } else {
        results.push(GateResult::fail(
            "oya-governance-vacuous-green",
            format!("bad={bad_issues:?} good={good_issues:?}"),
        ));
    }

    let decisions_dir = repo_root.join("docs/decisions");
    let orphan_bad = "References missing ADR-7777.\n";
    let orphan_good = "References existing ADR-0001.\n";
    match (
        orphan_adr_refs(orphan_bad, &decisions_dir),
        orphan_adr_refs(orphan_good, &decisions_dir),
    ) {
        (Ok(bad), Ok(good)) if bad == ["ADR-7777"] && good.is_empty() => {
            results.push(GateResult::pass("oya-governance-adr-orphan-citation"));
        }
        (bad, good) => results.push(GateResult::fail(
            "oya-governance-adr-orphan-citation",
            format!("bad={bad:?} good={good:?}"),
        )),
    }

    let openapi_bad = "openapi: 3.1.0\ninfo:\n  title: fixture\n  version: 1.0.0\npaths: {}\n";
    let asyncapi_bad = "asyncapi: 3.0.0\ninfo:\n  title: fixture\n  version: 1.0.0\nchannels: {}\n";
    let openapi_good = "openapi: 3.2.0\ninfo:\n  title: fixture\n  version: 1.0.0\npaths: {}\n";
    let openapi_findings = spec_version_findings(openapi_bad);
    let asyncapi_findings = spec_version_findings(asyncapi_bad);
    let good_findings = spec_version_findings(openapi_good);
    if openapi_findings
        .iter()
        .any(|finding| finding.contains("canonical 3.2.0"))
        && asyncapi_findings
            .iter()
            .any(|finding| finding.contains("canonical 3.1.0"))
        && good_findings.is_empty()
    {
        results.push(GateResult::pass(
            "oya-governance-version-pin-source-citation",
        ));
    } else {
        results.push(GateResult::fail(
            "oya-governance-version-pin-source-citation",
            format!(
                "openapi={openapi_findings:?} asyncapi={asyncapi_findings:?} good={good_findings:?}"
            ),
        ));
    }

    let short_doc = "one substantive line\n";
    let long_doc = (1..=50)
        .map(|index| format!("substantive line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    let short_findings = buildability_findings(short_doc);
    let long_findings = buildability_findings(&long_doc);
    if short_findings
        .iter()
        .any(|finding| finding.contains("has 1 substantive lines"))
        && long_findings.is_empty()
    {
        results.push(GateResult::pass("oya-governance-buildability-line-count"));
    } else {
        results.push(GateResult::fail(
            "oya-governance-buildability-line-count",
            format!("short={short_findings:?} long={long_findings:?}"),
        ));
    }

    results
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn render_json(results: &[GateResult]) -> String {
    let failures: Vec<String> = results
        .iter()
        .flat_map(|result| {
            result
                .failures
                .iter()
                .map(move |failure| format!("{}: {}", result.id, failure))
        })
        .collect();
    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" };
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!("  \"verdict\": \"{verdict}\",\n"));
    output.push_str(&format!(
        "  \"gate_count\": {},\n  \"failures\": [",
        results.len()
    ));
    for (index, failure) in failures.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&format!("\"{}\"", json_escape(failure)));
    }
    output.push_str("],\n  \"gates\": [\n");
    for (index, result) in results.iter().enumerate() {
        if index > 0 {
            output.push_str(",\n");
        }
        output.push_str(&format!(
            "    {{\"id\": \"{}\", \"verdict\": \"{}\"}}",
            result.id,
            if result.passed() { "PASS" } else { "FAIL" }
        ));
    }
    output.push_str("\n  ]\n}\n");
    output
}

fn repo_root_from_env() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn main() {
    let repo_root = repo_root_from_env();
    let results = evaluate(&repo_root);
    print!("{}", render_json(&results));
    if results.iter().any(|result| !result.passed()) {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after epoch")
            .as_nanos();
        env::temp_dir().join(format!("governance-hook-efficacy-{nanos}"))
    }

    #[test]
    fn vacuous_green_fixtures_fail_and_pass() {
        assert!(
            vacuous_green_issues("#[test]\nfn vacuous(){ assert!(true); }\n")
                .contains(&"assert!(true)")
        );
        assert!(
            vacuous_green_issues("#[test]\nfn real_assertion(){ assert_eq!(2 + 2, 4); }\n")
                .is_empty()
        );
    }

    #[test]
    fn orphan_adr_fixtures_fail_and_pass() {
        let root = unique_temp_dir();
        let decisions = root.join("docs/decisions");
        fs::create_dir_all(&decisions).expect("create temp decisions dir");
        fs::write(
            decisions.join("ADR-0001-existing-decision.md"),
            "# ADR-0001\n",
        )
        .expect("write temp ADR");

        let bad = orphan_adr_refs("Missing ADR-7777", &decisions).expect("scan bad fixture");
        let good = orphan_adr_refs("Existing ADR-0001", &decisions).expect("scan good fixture");
        fs::remove_dir_all(root).expect("remove temp root");

        assert_eq!(bad, ["ADR-7777"]);
        assert!(good.is_empty());
    }

    #[test]
    fn version_pin_fixtures_fail_and_pass() {
        assert!(
            spec_version_findings("openapi: 3.1.0\n")
                .iter()
                .any(|finding| finding.contains("canonical 3.2.0"))
        );
        assert!(
            spec_version_findings("asyncapi: 3.0.0\n")
                .iter()
                .any(|finding| finding.contains("canonical 3.1.0"))
        );
        assert!(spec_version_findings("openapi: 3.2.0\nasyncapi: 3.1.0\n").is_empty());
    }

    #[test]
    fn buildability_line_count_fixtures_fail_and_pass() {
        assert!(
            buildability_findings("one substantive line\n")
                .iter()
                .any(|finding| finding.contains("has 1 substantive lines"))
        );
        let long_doc = (1..=50)
            .map(|index| format!("substantive line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(buildability_findings(&long_doc).is_empty());
    }

    #[test]
    fn aggregate_evaluation_passes_against_repo() {
        let results = evaluate(&repo_root_from_env());
        assert!(
            results.iter().all(GateResult::passed),
            "results should pass: {results:?}"
        );
    }
}
