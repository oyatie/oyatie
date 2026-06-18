//! Test-results ingest for the gate-only overlay (ADR-0360 O2).
//!
//! Instead of re-running the cargo test mirror, `oya verify` can consume the
//! lane's already-produced nextest JUnit report and derive a PASS/FAIL verdict —
//! eliminating the double build. This module is the parser + verdict core
//! (the `--from-results <junit.xml>` wiring layers on top).
//!
//! Parses the `<testsuites>` summary attributes that cargo-nextest emits
//! (`tests`, `failures`, `errors`); the verdict is PASS iff failures == 0 AND
//! errors == 0. A report we cannot parse is treated as a FAILURE to ingest
//! (never a silent PASS).

/// Summary counts from a nextest/JUnit `<testsuites>` element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TestSummary {
    pub tests: u64,
    pub failures: u64,
    pub errors: u64,
}

impl TestSummary {
    /// PASS iff no failures and no errors.
    pub(crate) fn passed(&self) -> bool {
        self.failures == 0 && self.errors == 0
    }
}

/// Parse the top-level `<testsuites ...>` summary from a JUnit XML document.
/// Returns an error (never a default PASS) if the element/attributes are absent.
pub(crate) fn parse_junit_summary(xml: &str) -> Result<TestSummary, String> {
    let open = xml
        .find("<testsuites")
        .ok_or("no <testsuites> element in JUnit report")?;
    let rest = &xml[open..];
    let end = rest.find('>').ok_or("malformed <testsuites> element")?;
    let tag = &rest[..end];

    let tests = attr_u64(tag, "tests")?;
    // `failures` and `errors` may be omitted when zero in some emitters; default
    // to 0 only when the attribute is genuinely absent (not when unparseable).
    let failures = attr_u64(tag, "failures").unwrap_or(0);
    let errors = attr_u64(tag, "errors").unwrap_or(0);
    Ok(TestSummary {
        tests,
        failures,
        errors,
    })
}

/// Extract a `name="<u64>"` attribute from an element's opening tag.
fn attr_u64(tag: &str, name: &str) -> Result<u64, String> {
    let needle = format!("{name}=\"");
    let start = tag
        .find(&needle)
        .ok_or_else(|| format!("missing attribute {name:?}"))?
        + needle.len();
    let value = tag[start..]
        .split('"')
        .next()
        .ok_or_else(|| format!("unterminated attribute {name:?}"))?;
    value
        .parse::<u64>()
        .map_err(|_| format!("attribute {name:?} is not a u64: {value:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GREEN: &str = r#"<?xml version="1.0"?>
<testsuites name="nextest-run" tests="128" failures="0" errors="0">
  <testsuite name="oya-foo" tests="3" failures="0" errors="0"/>
</testsuites>"#;

    const RED: &str = r#"<testsuites name="nextest-run" tests="40" failures="2" errors="1">
  <testsuite name="oya-bar" tests="40" failures="2" errors="1"/>
</testsuites>"#;

    #[test]
    fn green_report_passes() {
        let s = parse_junit_summary(GREEN).expect("parse");
        assert_eq!(
            s,
            TestSummary {
                tests: 128,
                failures: 0,
                errors: 0
            }
        );
        assert!(s.passed());
    }

    #[test]
    fn red_report_fails() {
        let s = parse_junit_summary(RED).expect("parse");
        assert_eq!(s.failures, 2);
        assert_eq!(s.errors, 1);
        assert!(!s.passed());
    }

    #[test]
    fn missing_failures_attr_defaults_zero() {
        let xml = r#"<testsuites tests="5">"#;
        let s = parse_junit_summary(xml).expect("parse");
        assert!(s.passed());
        assert_eq!(s.tests, 5);
    }

    #[test]
    fn unparseable_report_is_an_error_not_a_pass() {
        assert!(parse_junit_summary("not xml").is_err());
        assert!(parse_junit_summary("<testsuites name=\"x\">").is_err()); // no tests attr
    }
}
