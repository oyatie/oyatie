// ADR-0546 canonical-json: pure-core RED/GREEN unit fixtures. No filesystem — every case drives
// `canonicalize` / `evaluate_keyed` directly. ADR-0083 Tier-3: tests may use unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use serde_json::json;

fn literal_form() -> CanonicalForm {
    // The settled ADR-0546 dialect (literal UTF-8, 2-space, source order, LF, trailing newline, no BOM).
    CanonicalForm::default()
}

fn escaped_form() -> CanonicalForm {
    CanonicalForm {
        ensure_ascii: true,
        ..literal_form()
    }
}

// ───────────────────────────── GREEN: canonical inputs are fixed points ─────────────────────────────

#[test]
fn canonical_literal_object_is_a_fixed_point() {
    let canonical = "{\n  \"a\": 1,\n  \"b\": \"x\"\n}\n";
    assert_eq!(canonicalize(canonical, &literal_form()).unwrap(), canonical);
    assert!(is_canonical(canonical, &literal_form()).unwrap());
}

#[test]
fn canonical_literal_non_ascii_stays_literal() {
    // The exact FRIC-1781130000 content class: literal arrows/µ/§/— survive byte-for-byte.
    let canonical = "{\n  \"s\": \"→ µ § —\"\n}\n";
    assert_eq!(canonicalize(canonical, &literal_form()).unwrap(), canonical);
}

#[test]
fn canonical_escaped_branch_emits_escapes() {
    let escaped = "{\n  \"s\": \"\\u2192 \\u00b5\"\n}\n";
    assert_eq!(canonicalize(escaped, &escaped_form()).unwrap(), escaped);
}

#[test]
fn astral_plane_scalar_round_trips_both_branches() {
    // U+1F680 ROCKET. Literal => the raw 4-byte char; escaped => the surrogate pair.
    let literal = "[\n  \"\u{1f680}\"\n]\n";
    assert_eq!(canonicalize(literal, &literal_form()).unwrap(), literal);
    let escaped = "[\n  \"\\ud83d\\ude80\"\n]\n";
    assert_eq!(canonicalize(escaped, &escaped_form()).unwrap(), escaped);
    // The two are the same logical content: canon under each form is each other's target.
    assert_eq!(canonicalize(literal, &escaped_form()).unwrap(), escaped);
    assert_eq!(canonicalize(escaped, &literal_form()).unwrap(), literal);
}

#[test]
fn mandatory_escapes_are_preserved() {
    // Quote, backslash, control chars must stay escaped under BOTH forms.
    let canonical = "{\n  \"k\": \"a\\\"b\\\\c\\n\\t\\u0001\"\n}\n";
    assert_eq!(canonicalize(canonical, &literal_form()).unwrap(), canonical);
    assert_eq!(canonicalize(canonical, &escaped_form()).unwrap(), canonical);
}

#[test]
fn solidus_is_emitted_bare() {
    // \/ is a legal but non-canonical escape; canonical form unescapes it to /.
    let input = "[\n  \"a\\/b\"\n]\n";
    let want = "[\n  \"a/b\"\n]\n";
    assert_eq!(canonicalize(input, &literal_form()).unwrap(), want);
}

#[test]
fn number_lexemes_round_trip_verbatim() {
    // Canonical form must NOT reformat numbers: 1.00, 1e10, and a bigint past f64 survive.
    let canonical = "[\n  1.00,\n  1e10,\n  18446744073709551616,\n  -0,\n  2.5E-3\n]\n";
    assert_eq!(canonicalize(canonical, &literal_form()).unwrap(), canonical);
}

#[test]
fn empty_containers_are_inline() {
    let canonical = "{\n  \"a\": [],\n  \"b\": {}\n}\n";
    assert_eq!(canonicalize(canonical, &literal_form()).unwrap(), canonical);
}

#[test]
fn nested_indentation_is_two_space_per_level() {
    let canonical = "{\n  \"a\": {\n    \"b\": [\n      1,\n      2\n    ]\n  }\n}\n";
    assert_eq!(canonicalize(canonical, &literal_form()).unwrap(), canonical);
}

#[test]
fn idempotence_property_under_both_forms() {
    let inputs = [
        "{\"z\":1,\"a\":[1,2,{\"x\":\"→\"}],\"b\":null,\"c\":true}",
        "[\"\u{1f680}\",\"plain\",\"\\u00e9\",1.5,-2,{}]",
        "{\"k\":\"tab\\tnewline\\n\"}",
    ];
    for form in [literal_form(), escaped_form()] {
        for input in inputs {
            let once = canonicalize(input, &form).unwrap();
            let twice = canonicalize(&once, &form).unwrap();
            assert_eq!(once, twice, "canonicalize must be idempotent: {input}");
        }
    }
}

#[test]
fn sort_keys_branch_reorders_only_when_set() {
    let input = "{\"z\":1,\"a\":2}";
    let preserve = CanonicalForm {
        sort_keys: false,
        ..literal_form()
    };
    let sorted = CanonicalForm {
        sort_keys: true,
        ..literal_form()
    };
    assert_eq!(
        canonicalize(input, &preserve).unwrap(),
        "{\n  \"z\": 1,\n  \"a\": 2\n}\n"
    );
    assert_eq!(
        canonicalize(input, &sorted).unwrap(),
        "{\n  \"a\": 2,\n  \"z\": 1\n}\n"
    );
}

#[test]
fn trailing_newline_is_policy_controlled() {
    let no_nl = CanonicalForm {
        trailing_newline: false,
        ..literal_form()
    };
    assert_eq!(canonicalize("{}", &no_nl).unwrap(), "{}");
    assert_eq!(canonicalize("{}", &literal_form()).unwrap(), "{}\n");
}

#[test]
fn newline_is_live_policy_data() {
    // Honoring `newline` is not cosmetic: a crlf policy emits CRLF everywhere, and LF input is then
    // non-canonical under it (proving the DATA knob is read, not hardcoded).
    let crlf = CanonicalForm {
        newline: Newline::Crlf,
        ..literal_form()
    };
    let input = "{\n  \"a\": [\n    1\n  ]\n}\n";
    assert_eq!(
        canonicalize(input, &crlf).unwrap(),
        "{\r\n  \"a\": [\r\n    1\r\n  ]\r\n}\r\n"
    );
    assert!(
        !is_canonical(input, &crlf).unwrap(),
        "LF input is non-canonical under a crlf policy"
    );
    // Idempotent under crlf too.
    let once = canonicalize(input, &crlf).unwrap();
    assert_eq!(canonicalize(&once, &crlf).unwrap(), once);
}

#[test]
fn utf8_bom_is_live_policy_data() {
    // utf8_bom=true => canonical output BEGINS with a BOM; the parser strips a leading BOM, so the
    // round-trip is consistent (parse strips, format re-adds). Proves the knob is read.
    let bom = CanonicalForm {
        utf8_bom: true,
        ..literal_form()
    };
    let no_bom_input = "{\n  \"a\": 1\n}\n";
    let out = canonicalize(no_bom_input, &bom).unwrap();
    assert!(
        out.starts_with('\u{feff}'),
        "utf8_bom policy must prepend a BOM"
    );
    assert!(
        !is_canonical(no_bom_input, &bom).unwrap(),
        "BOM-less file is non-canonical under a bom policy"
    );
    // And it is a fixed point: feeding the BOM'd output back yields the same bytes.
    assert_eq!(canonicalize(&out, &bom).unwrap(), out);
}

#[test]
fn from_policy_reads_newline_and_utf8_bom() {
    let policy = json!({
        "canonical_form": {
            "ensure_ascii": false, "indent_width": 2, "sort_keys": false,
            "trailing_newline": true, "newline": "crlf", "utf8_bom": true
        }
    });
    let form = CanonicalForm::from_policy(&policy);
    assert_eq!(form.newline, Newline::Crlf);
    assert!(form.utf8_bom);
}

#[test]
fn collect_matches_json_extension_case_insensitively() {
    let dir = std::env::temp_dir().join(format!("canon-case-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    std::fs::write(dir.join("specs/lower.json"), "{}\n").unwrap();
    std::fs::write(dir.join("specs/UPPER.JSON"), "{}\n").unwrap();
    let observed = collect_observed(&dir, &policy_literal()).unwrap();
    let paths: Vec<&str> = observed.files.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"specs/lower.json"));
    assert!(
        paths.contains(&"specs/UPPER.JSON"),
        "uppercase .JSON must not evade governance"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ───────────────────────────── RED: drift is flagged, defects refused ─────────────────────────────

#[test]
fn escaped_unicode_under_literal_form_is_non_canonical() {
    // The FRIC repro: committed escaped bytes are NON-canonical under the literal policy.
    let escaped = "{\n  \"s\": \"\\u2192\"\n}\n";
    let canonical = canonicalize(escaped, &literal_form()).unwrap();
    assert_ne!(canonical, escaped);
    assert_eq!(canonical, "{\n  \"s\": \"→\"\n}\n");
    assert!(!is_canonical(escaped, &literal_form()).unwrap());
}

#[test]
fn four_space_indent_is_non_canonical() {
    let four = "{\n    \"a\": 1\n}\n";
    assert!(!is_canonical(four, &literal_form()).unwrap());
}

#[test]
fn minified_is_non_canonical() {
    assert!(!is_canonical("{\"a\":1}", &literal_form()).unwrap());
}

#[test]
fn missing_trailing_newline_is_non_canonical() {
    assert!(!is_canonical("{\n  \"a\": 1\n}", &literal_form()).unwrap());
}

#[test]
fn crlf_is_non_canonical() {
    assert!(!is_canonical("{\r\n  \"a\": 1\r\n}\r\n", &literal_form()).unwrap());
}

#[test]
fn bom_is_stripped_so_committed_bom_is_non_canonical() {
    let with_bom = "\u{feff}{\n  \"a\": 1\n}\n";
    let canonical = canonicalize(with_bom, &literal_form()).unwrap();
    assert_eq!(canonical, "{\n  \"a\": 1\n}\n");
    assert!(!is_canonical(with_bom, &literal_form()).unwrap());
}

#[test]
fn invalid_json_is_parse_error() {
    let err = canonicalize("{\"a\":}", &literal_form()).unwrap_err();
    assert_eq!(err.code(), "json_parse_error");
}

#[test]
fn trailing_data_is_parse_error() {
    let err = canonicalize("{} junk", &literal_form()).unwrap_err();
    assert_eq!(err.code(), "json_parse_error");
}

#[test]
fn duplicate_key_is_refused() {
    let err = canonicalize("{\"a\":1,\"a\":2}", &literal_form()).unwrap_err();
    assert!(matches!(err, CanonError::DuplicateKey(ref k) if k == "a"));
    assert_eq!(err.code(), "json_duplicate_key");
}

#[test]
fn lone_high_surrogate_is_parse_error() {
    let err = canonicalize("[\"\\ud800\"]", &literal_form()).unwrap_err();
    assert_eq!(err.code(), "json_parse_error");
}

#[test]
fn lone_low_surrogate_is_parse_error() {
    let err = canonicalize("[\"\\udc00\"]", &literal_form()).unwrap_err();
    assert_eq!(err.code(), "json_parse_error");
}

#[test]
fn nan_and_infinity_are_parse_errors() {
    assert_eq!(
        canonicalize("[NaN]", &literal_form()).unwrap_err().code(),
        "json_parse_error"
    );
    assert_eq!(
        canonicalize("[Infinity]", &literal_form())
            .unwrap_err()
            .code(),
        "json_parse_error"
    );
}

#[test]
fn leading_zero_number_is_parse_error() {
    assert_eq!(
        canonicalize("[01]", &literal_form()).unwrap_err().code(),
        "json_parse_error"
    );
}

#[test]
fn leading_plus_number_is_parse_error() {
    assert_eq!(
        canonicalize("[+1]", &literal_form()).unwrap_err().code(),
        "json_parse_error"
    );
}

#[test]
fn bare_fraction_number_is_parse_error() {
    assert_eq!(
        canonicalize("[.5]", &literal_form()).unwrap_err().code(),
        "json_parse_error"
    );
}

#[test]
fn truncated_or_non_ascii_hex_escape_is_parse_error_not_a_panic() {
    // \u followed by a multibyte scalar: parse_hex4 slices src[pos..pos+4]; if that splits a UTF-8
    // boundary `.get` returns None -> clean parse error, never a slicing panic. The no-panic
    // doctrine must hold on hostile input.
    let evil = "{\"k\":\"\\u\u{1f680}\"}"; // \u then a 4-byte ROCKET
    let err = canonicalize(evil, &literal_form()).unwrap_err();
    assert_eq!(err.code(), "json_parse_error");
    // truncated \u at end of input
    let truncated = "{\"k\":\"\\u12\"}";
    assert_eq!(
        canonicalize(truncated, &literal_form()).unwrap_err().code(),
        "json_parse_error"
    );
}

#[test]
fn signed_or_spaced_hex_escape_is_parse_error_not_drift() {
    // Review MED on PR #689: `u32::from_str_radix` alone accepts a leading sign, so the
    // strictly-invalid escape `\u+12f` (rejected by RFC 8259 §7, serde_json, Python) decoded as
    // U+012F and the file was misclassified as fixable `json_not_canonical` drift — `--fix` would
    // silently rewrite it to `į` instead of refusing. A `\u` escape is EXACTLY 4 ASCII hex digits;
    // anything else is a parse error, never canonicalizable drift.
    for evil in [
        "{\"k\":\"\\u+12f\"}",
        "{\"k\":\"\\u-123\"}",
        "{\"k\":\"\\u-000\"}", // from_str_radix would even accept this as Ok(0) -> NUL
        "{\"k\":\"\\u 123\"}",
        "{\"k\":\"\\u12\"}", // short: only 2 hex digits before the closing quote
    ] {
        assert_eq!(
            canonicalize(evil, &literal_form()).unwrap_err().code(),
            "json_parse_error",
            "{evil} must be a parse error"
        );
    }
}

#[test]
fn unescaped_control_char_in_string_is_parse_error() {
    let err = canonicalize("[\"a\nb\"]", &literal_form()).unwrap_err();
    assert_eq!(err.code(), "json_parse_error");
}

#[test]
fn depth_bound_errors_instead_of_panicking() {
    // 300 nested arrays exceed MAX_DEPTH (256): an ERROR, never a stack abort.
    let deep = format!("{}{}", "[".repeat(300), "]".repeat(300));
    let err = canonicalize(&deep, &literal_form()).unwrap_err();
    assert_eq!(err.code(), "json_parse_error");
}

// ───────────────────────────── evaluate_keyed: gate semantics ─────────────────────────────

fn policy_literal() -> Value {
    json!({
        "gate_id": GATE_ID,
        "canonical_form": {
            "ensure_ascii": false, "indent_width": 2, "sort_keys": false, "trailing_newline": true
        },
        "governed_roots": ["specs"],
        "exclusions": { "suffixes": [".generated.json"], "path_prefixes": ["specs/fixtures/"] }
    })
}

fn obs(path: &str, bytes: Option<&str>) -> ObservedFile {
    ObservedFile {
        path: path.to_owned(),
        bytes: bytes.map(str::to_owned),
    }
}

#[test]
fn evaluate_green_on_canonical_corpus() {
    let observed = Observed {
        files: vec![
            obs("specs/a.json", Some("{\n  \"a\": 1\n}\n")),
            obs("specs/b.json", Some("[]\n")),
        ],
    };
    let report = evaluate(&policy_literal(), &observed);
    assert_eq!(report.verdict, Verdict::Green);
    assert!(report.findings.is_empty());
}

#[test]
fn evaluate_flags_non_canonical_keyed_by_path() {
    let observed = Observed {
        files: vec![obs("specs/drift.json", Some("{\"a\":1}"))],
    };
    let findings = evaluate_keyed(&policy_literal(), &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "json_not_canonical" && f.key == "specs/drift.json")
    );
}

#[test]
fn evaluate_flags_escaped_unicode_as_non_canonical() {
    // The exact FRIC-1781130000 defect, surfaced by the gate.
    let observed = Observed {
        files: vec![obs(
            "specs/root-hub-pointers.json",
            Some("{\n  \"s\": \"\\u2192\"\n}\n"),
        )],
    };
    let findings = evaluate_keyed(&policy_literal(), &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "json_not_canonical" && f.key == "specs/root-hub-pointers.json")
    );
}

#[test]
fn evaluate_flags_non_utf8_as_parse_error() {
    let observed = Observed {
        files: vec![obs("specs/bin.json", None)],
    };
    let findings = evaluate_keyed(&policy_literal(), &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "json_parse_error" && f.key == "specs/bin.json")
    );
}

#[test]
fn evaluate_flags_duplicate_key() {
    let observed = Observed {
        files: vec![obs("specs/dup.json", Some("{\"a\":1,\"a\":2}"))],
    };
    let findings = evaluate_keyed(&policy_literal(), &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "json_duplicate_key" && f.key == "specs/dup.json")
    );
}

#[test]
fn violation_codes_const_covers_every_emitted_code() {
    let declared: BTreeSet<&str> = VIOLATION_CODES.into_iter().collect();
    let observed = Observed {
        files: vec![
            obs("specs/a.json", Some("{\"a\":1}")), // not_canonical
            obs("specs/b.json", Some("{\"a\":1,\"a\":2}")), // duplicate_key
            obs("specs/c.json", None),              // parse_error (non-utf8)
        ],
    };
    let findings = evaluate_keyed(&policy_literal(), &observed);
    let emitted: BTreeSet<&str> = findings.iter().map(|f| f.code.as_str()).collect();
    for code in &emitted {
        assert!(
            declared.contains(code),
            "emitted `{code}` not in VIOLATION_CODES"
        );
    }
    assert_eq!(
        emitted.len(),
        3,
        "expected all three codes exercised: {emitted:?}"
    );
}

#[test]
fn fixer_no_op_agrees_with_gate_green() {
    // Gate-green ⟺ fixer is a no-op (check == fix by construction).
    let dir = std::env::temp_dir().join(format!("canon-fix-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    std::fs::write(dir.join("specs/good.json"), "{\n  \"a\": 1\n}\n").unwrap();
    let observed = collect_observed(&dir, &policy_literal()).unwrap();
    assert_eq!(
        evaluate(&policy_literal(), &observed).verdict,
        Verdict::Green
    );
    let fix = fix_observed(&dir, &policy_literal(), &observed, true).unwrap();
    assert!(fix.fixed.is_empty(), "no files should need fixing");
    assert!(fix.is_clean());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fixer_round_trips_to_a_fixed_point() {
    let dir = std::env::temp_dir().join(format!("canon-fp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    // A drifted file: escaped unicode + minified.
    std::fs::write(dir.join("specs/drift.json"), "{\"s\":\"\\u2192\"}").unwrap();
    let policy = policy_literal();

    // First pass: gate RED, fixer rewrites one file.
    let observed = collect_observed(&dir, &policy).unwrap();
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Red);
    let fix = fix_observed(&dir, &policy, &observed, false).unwrap();
    assert_eq!(fix.fixed, vec!["specs/drift.json".to_owned()]);

    // Second pass: gate now GREEN and the fixer is a no-op (fixed point reached).
    let observed2 = collect_observed(&dir, &policy).unwrap();
    assert_eq!(evaluate(&policy, &observed2).verdict, Verdict::Green);
    let fix2 = fix_observed(&dir, &policy, &observed2, false).unwrap();
    assert!(fix2.fixed.is_empty(), "fixer must reach a fixed point");
    let written = std::fs::read_to_string(dir.join("specs/drift.json")).unwrap();
    assert_eq!(written, "{\n  \"s\": \"→\"\n}\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn collect_applies_suffix_and_path_prefix_exclusions() {
    let dir = std::env::temp_dir().join(format!("canon-excl-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs/fixtures")).unwrap();
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    std::fs::write(dir.join("specs/kept.json"), "{}\n").unwrap();
    std::fs::write(dir.join("specs/face.generated.json"), "{}\n").unwrap();
    std::fs::write(dir.join("specs/fixtures/owned.json"), "{}\n").unwrap();
    std::fs::write(dir.join("docs/outside.json"), "{}\n").unwrap(); // outside governed_roots
    let observed = collect_observed(&dir, &policy_literal()).unwrap();
    let paths: Vec<&str> = observed.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["specs/kept.json"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn non_canonical_fixture_fix_makes_the_gate_green() {
    // Founder doctrine (2026-06-11): automation is the default path. The exact loop the directive
    // names: non-canonical fixture -> --fix -> gate GREEN, in one pass, no hand-editing.
    let dir = std::env::temp_dir().join(format!("canon-auto-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    // Escaped-unicode + 4-space indent + no trailing newline: the full FRIC-1781130000 drift class.
    std::fs::write(dir.join("specs/x.json"), "{\n    \"s\": \"\\u2192\"\n}").unwrap();
    let policy = policy_literal();

    let before = collect_observed(&dir, &policy).unwrap();
    assert_eq!(
        evaluate(&policy, &before).verdict,
        Verdict::Red,
        "gate is RED before fix"
    );

    let fix = fix_observed(&dir, &policy, &before, false).unwrap();
    assert_eq!(fix.fixed, vec!["specs/x.json".to_owned()]);
    assert!(fix.is_clean());

    let after = collect_observed(&dir, &policy).unwrap();
    assert_eq!(
        evaluate(&policy, &after).verdict,
        Verdict::Green,
        "gate is GREEN after --fix"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn settle_style_canonical_file_fix_is_a_no_op() {
    // The directive's second fixture: a file already in the faces/settle canonical form (literal
    // UTF-8, 2-space, trailing newline) must be a fixer NO-OP — proving --fix and the settle tool
    // do not fight each other in a rewrite loop. This is the concrete shared-serializer-consistency
    // assertion: the byte form the settle tool emits IS this gate's fixed point.
    let dir = std::env::temp_dir().join(format!("canon-settle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    // Exactly what accounting-registry::to_canonical_json emits for this content: literal UTF-8,
    // 2-space, source order, trailing newline.
    let settle_form = "{\n  \"arrow\": \"→\",\n  \"micro\": \"µ\"\n}\n";
    std::fs::write(dir.join("specs/face-like.json"), settle_form).unwrap();
    let policy = policy_literal();
    let observed = collect_observed(&dir, &policy).unwrap();
    assert_eq!(evaluate(&policy, &observed).verdict, Verdict::Green);
    let fix = fix_observed(&dir, &policy, &observed, false).unwrap();
    assert!(
        fix.fixed.is_empty(),
        "settle-canonical file must be a fixer no-op (no rewrite loop)"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("specs/face-like.json")).unwrap(),
        settle_form,
        "the settle byte form must be untouched"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn gate_failure_output_prints_the_exact_runnable_fix_command() {
    // The directive: the blocking gate's failure output must print the EXACT auto-fix command, and
    // it must reference the real buck2 target (not a typo'd one).
    let observed = Observed {
        files: vec![obs("specs/drift.json", Some("{\"a\":1}"))],
    };
    let report = evaluate(&policy_literal(), &observed);
    let rendered = render_findings(&report.findings);
    assert!(
        rendered.contains("--fix"),
        "must print the --fix command: {rendered}"
    );
    assert!(
        rendered.contains("//ci/facade/canonical-json:oya-cloud-ci-canonical-json-bin"),
        "must reference the REAL buck2 binary target: {rendered}"
    );
}

#[test]
fn fixer_refuses_duplicate_keys_without_rewriting() {
    let dir = std::env::temp_dir().join(format!("canon-refuse-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    let original = "{\"a\":1,\"a\":2}";
    std::fs::write(dir.join("specs/dup.json"), original).unwrap();
    let observed = collect_observed(&dir, &policy_literal()).unwrap();
    let fix = fix_observed(&dir, &policy_literal(), &observed, false).unwrap();
    assert!(fix.fixed.is_empty());
    assert_eq!(fix.refused.len(), 1);
    assert_eq!(fix.refused[0].0, "specs/dup.json");
    // The file is untouched — the fixer never silently dropped a member.
    assert_eq!(
        std::fs::read_to_string(dir.join("specs/dup.json")).unwrap(),
        original
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fixer_refuses_signed_hex_escape_without_rewriting() {
    // Review MED regression fixture (PR #689): a file containing the strictly-invalid escape
    // `"\u+12f"` must classify as `json_parse_error` — NOT `json_not_canonical` — and `--fix`
    // must REFUSE it byte-unchanged instead of silently rewriting it to `"į"`. Refuse-defects
    // contract: parse errors are human-judgment residue, never drift.
    let dir = std::env::temp_dir().join(format!("canon-hexsign-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("specs")).unwrap();
    let original = "{\"k\":\"\\u+12f\"}";
    std::fs::write(dir.join("specs/sign.json"), original).unwrap();
    let observed = collect_observed(&dir, &policy_literal()).unwrap();
    let findings = evaluate_keyed(&policy_literal(), &observed);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "json_parse_error" && f.key == "specs/sign.json"),
        "signed hex escape must be a parse error: {findings:#?}"
    );
    assert!(
        !findings
            .iter()
            .any(|f| f.code == "json_not_canonical" && f.key == "specs/sign.json"),
        "a strictly-invalid escape must never classify as fixable drift: {findings:#?}"
    );
    let fix = fix_observed(&dir, &policy_literal(), &observed, false).unwrap();
    assert!(
        fix.fixed.is_empty(),
        "the fixer must not rewrite an invalid escape"
    );
    assert!(!fix.is_clean());
    assert_eq!(fix.refused.len(), 1);
    assert_eq!(fix.refused[0].0, "specs/sign.json");
    // The file is byte-unchanged — the fixer never laundered invalid JSON into valid JSON.
    assert_eq!(
        std::fs::read_to_string(dir.join("specs/sign.json")).unwrap(),
        original
    );
    let _ = std::fs::remove_dir_all(&dir);
}
