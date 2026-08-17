//! # oya-buck-syntax-kernel (ADR-0549)
//!
//! The single shared, SOUND lexer/parser for the Starlark subset the cloud-ci gates consume,
//! plus span-accurate safe-edit primitives and the fixer self-validation harness. Consolidates
//! the private, divergent BUCK parsers that hostile review repeatedly beat (comment-blind
//! `call_block`, first-occurrence name binding, paren-in-string depth miscounts,
//! backslash-newline continuation hiding deps — ADR-0545 "Known Limitations and Destination",
//! ADR-0547 D6, FRIC-1781131000-buck-syntax-kernel, FRIC-1781200001, FRIC-1781230000).
//!
//! ## R0 pack-shape (pure kernel)
//! No filesystem, no policy, no repo specifics, no third-party deps (std only). Inputs are
//! strings (+ caller-supplied file lists for glob expansion); outputs are parsed structures
//! with exact byte spans, evaluated values, edited strings, or refusals. All I/O and all
//! policy live in the consumers.
//!
//! ## Design (bespoke rowan-style, W2 doctrine)
//! A hand-rolled lossless-enough lexer + recursive-descent parser over the modeled subset:
//! every node carries its byte span over the original text, unmodeled shapes are recorded as
//! exact-span `Opaque` nodes (fail-honest: never misinterpreted, never silently dropped), and
//! structurally undelimitable input is a hard error (fail-closed). rust-analyzer's rowan and
//! tree-sitter's design notes are REFERENCE ONLY (no dependency): the gates need byte-span
//! fidelity and refusal semantics, not incremental reparsing.
//!
//! ## Module map
//! - [`lexer`]   — tokens with spans; comment trivia; string cooking incl. the
//!   backslash-newline continuation (#693 LOW-2) and escaped-quote (#693 LOW-X2) classes.
//! - [`parser`]  — statements/expressions with spans; sound target enumeration; double-comma
//!   and unbalanced-delimiter shapes are hard errors.
//! - [`eval`]    — static evaluation: string vars, glob vars, `+` concat, dict literal /
//!   comprehension destination values, `VAR["k"] = v` assembly, name-field target binding,
//!   buck2-style glob matching.
//! - [`edit`]    — span-based safe-edit primitives (insert kwarg / dict entry, remove list
//!   element, replace span); refuse over guess.
//! - [`harness`] — the write-through guard: reparse + caller semantic hook + first-pre-image
//!   registry + deterministic rollback. Fixers MUST route rewrites through it.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod edit;
pub mod eval;
pub mod harness;
pub mod lexer;
pub mod parser;

pub use edit::{
    EditError, insert_at, insert_dict_entry, insert_kwarg, remove_list_element, replace_span,
};
pub use eval::{
    Env, call_strings, dict_values, eval_string, eval_string_with, expr_strings, find_target,
    glob_match, resolve_dict_var,
};
pub use harness::{GuardRefusal, PreImageRegistry, guarded_rewrite};
pub use lexer::{LexError, Span};
pub use parser::{
    Arg, BuckDoc, CallExpr, DictComp, DictEntry, DictExpr, Expr, ExprNode, ListElement, ListExpr,
    ParseError, Stmt, parse,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn must_parse(text: &str) -> BuckDoc {
        parse(text).unwrap_or_else(|e| panic!("must parse: {e}\n---\n{text}"))
    }

    fn calls(doc: &BuckDoc) -> Vec<&CallExpr> {
        doc.stmts
            .iter()
            .filter_map(|s| match s {
                Stmt::Call(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    fn expr_strings_of(node: &ExprNode) -> Vec<String> {
        expr_strings(node)
    }

    /// `third-party//:<name>` tokens from a call's string values (the kernel-purity detect shape).
    fn thirdparty_tokens(call: &CallExpr) -> Vec<String> {
        let mut out = Vec::new();
        for s in call_strings(call) {
            let marker = "third-party//:";
            let mut from = 0usize;
            while let Some(rel) = s[from..].find(marker) {
                let start = from + rel + marker.len();
                let name: String = s[start..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                    .collect();
                if !name.is_empty() {
                    out.push(name);
                }
                from = start;
            }
        }
        out
    }

    // =====================================================================
    // Lexer: string cooking, continuations, comments
    // =====================================================================

    #[test]
    fn backslash_newline_continuation_inside_string_cooks_to_joined_value() {
        // FRIC-1781230000 (#693 LOW-2 detect gap): a dep split across a backslash-newline
        // continuation must cook to the JOINED value so the detect lane sees `kube`, not `k`.
        let buck =
            "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:k\\\nube\"],\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let deps = thirdparty_tokens(call);
        assert_eq!(
            deps,
            vec!["kube".to_owned()],
            "continuation must not hide the dep: {deps:?}"
        );
    }

    #[test]
    fn backslash_escaped_quote_does_not_leak_string_state() {
        // #693 LOW-X2: `labels = ["weird\")label"]` — the escaped quote stays INSIDE the string;
        // the stray `)` must not end the call early and hide the dep below.
        let buck = concat!(
            "rust_library(\n",
            "    name = \"x\",\n",
            "    labels = [\"weird\\\")label\"],\n",
            "    deps = [\n",
            "        \"third-party//:kube\",\n",
            "    ],\n",
            ")\n",
        );
        let doc = must_parse(buck);
        let deps = thirdparty_tokens(calls(&doc)[0]);
        assert!(
            deps.contains(&"kube".to_owned()),
            "dep after escaped quote: {deps:?}"
        );
    }

    #[test]
    fn stray_paren_in_comment_does_not_end_the_block() {
        // #691 H5: a `)` inside a comment must not terminate the call span early.
        let buck = concat!(
            "rust_library(\n",
            "    name = \"fake-kernel\",\n",
            "    # 1) serde 2) kube — note: stray ) in comment\n",
            "    deps = [\n",
            "        \"third-party//:kube\",\n",
            "    ],\n",
            ")\n",
        );
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let deps = thirdparty_tokens(call);
        assert!(deps.contains(&"kube".to_owned()), "H5: {deps:?}");
        let kube_pos = buck.find("third-party//:kube").unwrap();
        assert!(
            call.span.end > kube_pos,
            "call span must extend past the dep"
        );
    }

    #[test]
    fn paren_inside_string_does_not_end_the_block() {
        let buck = "rust_library(\n    name = \"x\",\n    labels = [\"has ) paren\"],\n    deps = [\"third-party//:sqlx\"],\n)\n";
        let deps = thirdparty_tokens(calls(&must_parse(buck))[0]);
        assert!(deps.contains(&"sqlx".to_owned()), "{deps:?}");
    }

    #[test]
    fn unterminated_block_is_a_hard_parse_error() {
        // #691 H6: an unterminated call (no matching close) must fail closed, never guess.
        let buck = concat!(
            "rust_library(\n",
            "    name = \"fake-kernel\",\n",
            "    deps = [\"third-party//:kube\"],\n",
            "    # no matching close\n",
        );
        assert!(
            parse(buck).is_err(),
            "unterminated call must be a parse error"
        );
    }

    #[test]
    fn comment_text_is_never_a_call_or_dep() {
        // Comment-blind class: `rust_library(` and `third-party//:fake` inside comments must
        // produce no call and no dep.
        let buck = concat!(
            "# rust_library( this is prose, not a target\n",
            "# deps = [\"third-party//:fake\"]\n",
            "rust_library(\n",
            "    name = \"real\",\n",
            "    deps = [\"third-party//:serde\"],\n",
            ")\n",
        );
        let doc = must_parse(buck);
        let cs = calls(&doc);
        assert_eq!(cs.len(), 1, "exactly one real call");
        let deps = thirdparty_tokens(cs[0]);
        assert_eq!(
            deps,
            vec!["serde".to_owned()],
            "comment dep must not be extracted: {deps:?}"
        );
    }

    #[test]
    fn string_text_is_never_a_call() {
        let buck = "X = \"rust_library(\"\nrust_library(\n    name = \"real\",\n)\n";
        let doc = must_parse(buck);
        assert_eq!(calls(&doc).len(), 1);
    }

    #[test]
    fn multibyte_chars_in_comments_and_strings_do_not_panic_or_shift_spans() {
        // The em-dash (3-byte) probe from the hermeticity gate's multibyte fixtures.
        let buck = "rust_library(\n    # house style — em-dash comment\n    name = \"t\",\n    srcs = [],\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let name = call.kwarg("name").expect("name kwarg");
        assert_eq!(name.value.span.slice(buck), "\"t\"");
        let cafe = "rust_library(\n    name = \"café\",\n    deps = [\"third-party//:caf\u{00e9}-dep\"],\n)\n";
        let doc2 = must_parse(cafe);
        assert_eq!(calls(&doc2).len(), 1);
    }

    #[test]
    fn rust_test_prefixed_identifier_is_not_a_false_call_hit() {
        // `IDENT_rust_test(` style false hits from the substring era must not bind.
        let buck = "my_rust_library(\n    name = \"not-a-lib\",\n)\nrust_library(\n    name = \"lib\",\n)\n";
        let doc = must_parse(buck);
        let kinds: Vec<&str> = calls(&doc).iter().map(|c| c.func.as_str()).collect();
        assert_eq!(kinds, vec!["my_rust_library", "rust_library"]);
    }

    #[test]
    fn double_comma_is_structurally_unparseable() {
        // The historical comment-blind-fixer corruption shape: a double comma must be a hard
        // parse error so the harness reparse refuses the write.
        let corrupt = "rust_library(\n    name = \"x\",\n    deps = [],,\n)\n";
        assert!(parse(corrupt).is_err(), "double comma must fail to parse");
        let corrupt2 = "rust_library(\n    name = \"x\",\n    deps = [\"a\",, \"b\"],\n)\n";
        assert!(
            parse(corrupt2).is_err(),
            "list double comma must fail to parse"
        );
    }

    #[test]
    fn starlark_escape_sequences_cook_to_buck2_semantics() {
        // Review F1 (HIGH): buck2 evaluates `"third-party//:k\x75be"` to `third-party//:kube`
        // (proven via buck2 uquery). The lexer must cook \xXX, \uXXXX, \UXXXXXXXX, and octal
        // \NNN to the same value so a denylisted dep cannot hide by escape spelling.
        let hex = "deps = [\"third-party//:k\\x75be\"]\n";
        let doc_hex = must_parse(hex);
        let Stmt::Assign { value, .. } = &doc_hex.stmts[0] else {
            panic!("assign")
        };
        assert_eq!(
            expr_strings_of(value),
            vec!["third-party//:kube".to_owned()],
            "\\x75 must cook to `u`"
        );
        let uni = "deps = [\"third-party//:k\\u0075be\"]\n";
        let doc_uni = must_parse(uni);
        let Stmt::Assign { value, .. } = &doc_uni.stmts[0] else {
            panic!("assign")
        };
        assert_eq!(
            expr_strings_of(value),
            vec!["third-party//:kube".to_owned()]
        );
        let big = "deps = [\"third-party//:k\\U00000075be\"]\n";
        let doc_big = must_parse(big);
        let Stmt::Assign { value, .. } = &doc_big.stmts[0] else {
            panic!("assign")
        };
        assert_eq!(
            expr_strings_of(value),
            vec!["third-party//:kube".to_owned()]
        );
        let octal = "deps = [\"third-party//:k\\165be\"]\n";
        let doc_octal = must_parse(octal);
        let Stmt::Assign { value, .. } = &doc_octal.stmts[0] else {
            panic!("assign")
        };
        assert_eq!(
            expr_strings_of(value),
            vec!["third-party//:kube".to_owned()],
            "octal \\165 is `u`"
        );
        // Standard single-char escapes cook to their control characters.
        let std_esc = "X = \"a\\ab\\bf\\fv\\v\"\n";
        let doc_std = must_parse(std_esc);
        let env = Env::from_doc(&doc_std);
        assert_eq!(
            env.string_vars.get("X").map(String::as_str),
            Some("a\x07b\x08f\x0Cv\x0B")
        );
        // \0 (the one-digit octal case) still cooks to NUL.
        let nul = "X = \"a\\0b\"\n";
        let doc_nul = must_parse(nul);
        let env_nul = Env::from_doc(&doc_nul);
        assert_eq!(
            env_nul.string_vars.get("X").map(String::as_str),
            Some("a\0b")
        );
    }

    #[test]
    fn unimplemented_escape_classes_are_hard_lex_errors() {
        // Fail-closed: any escape class the lexer does not implement must refuse, never keep
        // the character verbatim (the silent mis-cook the review F1 proved against buck2).
        for bad in [
            "X = \"a\\qb\"\n",
            "X = \"a\\x7\"\n",
            "X = \"a\\u12\"\n",
            "X = \"\\ud800\"\n",
        ] {
            assert!(parse(bad).is_err(), "must refuse to guess: {bad}");
        }
        // Raw strings still keep backslashes verbatim (no cooking, no refusal).
        let raw = "PAT = r\"a\\qb\"\n";
        let doc = must_parse(raw);
        let env = Env::from_doc(&doc);
        assert_eq!(
            env.string_vars.get("PAT").map(String::as_str),
            Some("a\\qb")
        );
    }

    #[test]
    fn triple_quoted_and_raw_strings_lex() {
        let buck = "DOC = \"\"\"multi\nline ) [ } text\"\"\"\nPAT = r\"a\\d+\"\nrust_library(\n    name = \"x\",\n)\n";
        let doc = must_parse(buck);
        let env = Env::from_doc(&doc);
        assert_eq!(
            env.string_vars.get("DOC").map(String::as_str),
            Some("multi\nline ) [ } text")
        );
        assert_eq!(
            env.string_vars.get("PAT").map(String::as_str),
            Some("a\\d+")
        );
        assert_eq!(calls(&doc).len(), 1);
    }

    // =====================================================================
    // Parser: kwargs, name binding, spans
    // =====================================================================

    #[test]
    fn kwarg_lookup_is_exact_not_suffix_matched() {
        // `srcs =` must not match `mapped_srcs =` (the find_top_level ident-boundary fix).
        let buck = "rust_library(\n    name = \"x\",\n    mapped_srcs = {\"a\": \"b\"},\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        assert!(
            call.kwarg("srcs").is_none(),
            "srcs must not match mapped_srcs"
        );
        assert!(call.kwarg("mapped_srcs").is_some());
    }

    #[test]
    fn target_binding_is_by_name_field_not_first_occurrence() {
        // ADR-0545 residual: the literal `"adapter"` appears EARLIER in another target's field;
        // first-occurrence substring binding would patch the wrong block.
        let buck = concat!(
            "rust_binary(\n",
            "    name = \"tool\",\n",
            "    crate = \"adapter\",\n", // decoy: the string "adapter" appears first here
            ")\n",
            "rust_library(\n",
            "    name = \"adapter\",\n",
            "    srcs = [],\n",
            ")\n",
        );
        let doc = must_parse(buck);
        let env = Env::from_doc(&doc);
        let target = find_target(&doc, None, "adapter", &env).expect("bind adapter");
        assert_eq!(
            target.func, "rust_library",
            "must bind the real target, not the decoy"
        );
    }

    #[test]
    fn concat_name_binding_resolves_through_string_vars() {
        let buck =
            "ROOT = \"svc\"\nrust_library(\n    name = ROOT + \"-lib\",\n    srcs = [],\n)\n";
        let doc = must_parse(buck);
        let env = Env::from_doc(&doc);
        assert!(find_target(&doc, Some(&["rust_library"]), "svc-lib", &env).is_some());
    }

    #[test]
    fn opaque_shapes_are_flagged_not_misread() {
        // A select() dep value parses (strings visible); a conditional expression is opaque and
        // flags the call so detect lanes can over-approximate instead of trusting a blind spot.
        let with_select = "rust_library(\n    name = \"x\",\n    deps = select({\"cfg\": [\"third-party//:kube\"]}),\n)\n";
        let doc = must_parse(with_select);
        let call = calls(&doc)[0];
        assert!(
            thirdparty_tokens(call).contains(&"kube".to_owned()),
            "select() strings are visible"
        );
        let with_conditional =
            "rust_library(\n    name = \"x\",\n    deps = [\"a\"] if True else [\"b\"],\n)\n";
        let doc2 = must_parse(with_conditional);
        assert!(
            calls(&doc2)[0].has_opaque(),
            "conditional dep value must flag opaque"
        );
    }

    #[test]
    fn excessively_nested_expressions_fail_closed_before_recursive_dos() {
        fn assert_depth_capped(label: &str, expr: String) {
            let buck = format!("X = {expr}\n");
            let err = parse(&buck).expect_err("recursive input must be capped");

            assert!(
                err.message.contains("nesting depth"),
                "{label}: depth cap must be explicit, got: {err}"
            );
        }

        let mut list_expr = "\"leaf\"".to_owned();
        let mut paren_expr = "\"leaf\"".to_owned();
        let mut call_expr = "\"leaf\"".to_owned();
        let mut dict_expr = "\"leaf\"".to_owned();
        for _ in 0..300 {
            list_expr = format!("[{list_expr}]");
            paren_expr = format!("({paren_expr})");
            call_expr = format!("f({call_expr})");
            dict_expr = format!("{{\"k\": {dict_expr}}}");
        }
        assert_depth_capped("list", list_expr);
        assert_depth_capped("parenthesized expression", paren_expr);
        assert_depth_capped("call argument", call_expr);
        assert_depth_capped("dict value", dict_expr);

        let opaque_expr = format!("root.{}\"leaf\"{}", "m(".repeat(300), ")".repeat(300));
        assert_depth_capped("opaque postfix tail", opaque_expr);
    }

    #[test]
    fn expression_nesting_boundary_allows_limit_and_rejects_next_level() {
        fn nested_lists(depth: usize) -> String {
            let mut expr = "\"leaf\"".to_owned();
            for _ in 0..depth {
                expr = format!("[{expr}]");
            }
            expr
        }

        fn opaque_call_tail(depth: usize) -> String {
            format!("root.{}\"leaf\"{}", "m(".repeat(depth), ")".repeat(depth))
        }

        fn assert_parses(label: &str, expr: String) {
            let buck = format!("X = {expr}\n");
            parse(&buck).unwrap_or_else(|err| panic!("{label}: expected parse success, got {err}"));
        }

        fn assert_depth_capped(label: &str, expr: String) {
            let buck = format!("X = {expr}\n");
            let err = parse(&buck).expect_err("expression one past the cap must fail");
            assert!(
                err.message.contains("nesting depth exceeds 128"),
                "{label}: depth cap must name the limit, got: {err}"
            );
        }

        assert_parses("modeled expression at cap", nested_lists(128));
        assert_depth_capped("modeled expression beyond cap", nested_lists(129));
        assert_parses("opaque postfix tail at cap", opaque_call_tail(128));
        assert_depth_capped("opaque postfix tail beyond cap", opaque_call_tail(129));
    }

    #[test]
    fn index_assignments_and_comprehensions_parse() {
        let buck = concat!(
            "ROOT = \"cloud/ci/adapter\"\n",
            "SRCS = glob([\"src/**/*.rs\", \"**/*.cedar\"])\n",
            "MAPPED = {src: ROOT + \"/\" + src for src in SRCS}\n",
            "MAPPED[\"//cloud/ci/policy:x.cedar\"] = ROOT + \"/policy/x.cedar\"\n",
            "rust_library(\n    name = \"adapter\",\n    srcs = [],\n    mapped_srcs = MAPPED,\n)\n",
        );
        let doc = must_parse(buck);
        assert!(
            doc.stmts
                .iter()
                .any(|s| matches!(s, Stmt::IndexAssign { base, .. } if base == "MAPPED"))
        );
        let env = Env::from_doc(&doc);
        assert_eq!(
            env.string_vars.get("ROOT").map(String::as_str),
            Some("cloud/ci/adapter")
        );
        assert_eq!(
            env.glob_vars.get("SRCS"),
            Some(&vec!["src/**/*.rs".to_owned(), "**/*.cedar".to_owned()])
        );
    }

    // =====================================================================
    // Eval: ported hermeticity resolution fixtures
    // =====================================================================

    const CEDAR_SHAPE: &str = concat!(
        "ADAPTER_ROOT = \"cloud/ci/adapter\"\n",
        "ADAPTER_SRCS = glob([\"src/**/*.rs\", \"**/*.cedar\"])\n",
        "ADAPTER_MAPPED_SRCS = {src: ADAPTER_ROOT + \"/\" + src for src in ADAPTER_SRCS}\n",
        "ADAPTER_MAPPED_SRCS[\"//cloud/ci/policy:x.cedar\"] = ADAPTER_ROOT + \"/policy/x.cedar\"\n",
        "\n",
        "rust_library(\n",
        "    name = \"adapter\",\n",
        "    srcs = [],\n",
        "    crate_root = ADAPTER_ROOT + \"/src/lib.rs\",\n",
        "    mapped_srcs = ADAPTER_MAPPED_SRCS,\n",
        ")\n",
    );

    #[test]
    fn mapped_srcs_comprehension_plus_explicit_value_resolves() {
        // The cedar-adapter shape (ported from the hermeticity gate's fixtures).
        let doc = must_parse(CEDAR_SHAPE);
        let env = Env::from_doc(&doc);
        let files = vec!["src/lib.rs".to_owned()];
        let values = resolve_dict_var(&doc, "ADAPTER_MAPPED_SRCS", &env, &files);
        assert!(
            values
                .iter()
                .any(|v| v == "cloud/ci/adapter/policy/x.cedar"),
            "explicit mapped value must resolve: {values:?}"
        );
        assert!(
            values.iter().any(|v| v == "cloud/ci/adapter/src/lib.rs"),
            "comprehension value must resolve: {values:?}"
        );
    }

    #[test]
    fn membership_is_against_values_not_keys() {
        // The refuted alternative from the hermeticity consensus: a mapped_srcs KEY (a
        // `//path:name` label) must never enter the destination VALUE set.
        let doc = must_parse(CEDAR_SHAPE);
        let env = Env::from_doc(&doc);
        let files = vec!["src/lib.rs".to_owned()];
        let values = resolve_dict_var(&doc, "ADAPTER_MAPPED_SRCS", &env, &files);
        assert!(
            !values.iter().any(|v| v == "//cloud/ci/policy:x.cedar"),
            "the KEY must never enter the destination set: {values:?}"
        );
    }

    #[test]
    fn glob_double_star_matches_nested() {
        assert!(glob_match("src/**/*.rs", "src/a/b/c.rs"));
        assert!(glob_match("src/**/*.rs", "src/lib.rs"));
        assert!(glob_match("**/*.cedar", "policy/x.cedar"));
        assert!(!glob_match("src/*.rs", "src/a/b.rs"));
        assert!(glob_match("src/*.rs", "src/lib.rs"));
    }

    #[test]
    fn eval_string_concat_and_vars() {
        let buck = "ROOT = \"a/b\"\nX = ROOT + \"/c\" + \"/d\"\n";
        let doc = must_parse(buck);
        let env = Env::from_doc(&doc);
        let x = doc.stmts.iter().find_map(|s| match s {
            Stmt::Assign { name, value, .. } if name == "X" => Some(value),
            _ => None,
        });
        assert_eq!(eval_string(x.unwrap(), &env).as_deref(), Some("a/b/c/d"));
    }

    #[test]
    fn unresolvable_operand_evaluates_to_none() {
        let buck = "X = UNKNOWN + \"/c\"\n";
        let doc = must_parse(buck);
        let env = Env::from_doc(&doc);
        let x = doc.stmts.iter().find_map(|s| match s {
            Stmt::Assign { value, .. } => Some(value),
            _ => None,
        });
        assert_eq!(
            eval_string(x.unwrap(), &env),
            None,
            "unknown var must refuse, not guess"
        );
    }

    // =====================================================================
    // Edit primitives: the historical corruption vectors
    // =====================================================================

    #[test]
    fn insert_kwarg_supplies_missing_comma_and_reparses() {
        // The missing-comma vector: last field lacks a trailing comma; the inserted kwarg must
        // come with exactly one separating comma and the result must reparse.
        let buck = "rust_library(\n    name = \"lib\",\n    srcs = glob([\"src/**/*.rs\"]),\n    crate_root = \"src/lib.rs\"\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let patched = insert_kwarg(
            buck,
            call,
            "mapped_srcs = {\n        \"//q:asset.cedar\": \"q/asset.cedar\",\n    }",
        )
        .expect("insert");
        assert!(
            patched.contains("\"src/lib.rs\","),
            "separating comma added: {patched}"
        );
        assert!(!patched.contains(",,"), "no double comma: {patched}");
        let redoc = must_parse(&patched);
        assert!(
            calls(&redoc)[0].kwarg("mapped_srcs").is_some(),
            "kwarg visible after reparse"
        );
    }

    #[test]
    fn insert_kwarg_into_comment_bearing_block_is_sound() {
        // The double-comma-from-comment-blind-heuristics vector: `deps = [],  # trailing
        // comment` before `)`. The prior gates REFUSED this shape; the kernel edits it soundly.
        let buck = "rust_library(\n    name = \"lib\",\n    srcs = [],\n    crate_root = \"src/lib.rs\",\n    deps = [],  # trailing comment\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let patched = insert_kwarg(
            buck,
            call,
            "mapped_srcs = {\n        \"//q:a.cedar\": \"q/a.cedar\",\n    }",
        )
        .expect("comment-bearing block must now be editable");
        assert!(!patched.contains(",,"), "no double comma: {patched}");
        assert!(
            patched.contains("# trailing comment"),
            "comment preserved: {patched}"
        );
        let redoc = must_parse(&patched);
        let recall = calls(&redoc)[0];
        assert!(
            recall.kwarg("mapped_srcs").is_some(),
            "kwarg present after reparse: {patched}"
        );
        assert!(recall.kwarg("deps").is_some(), "deps survives: {patched}");
    }

    #[test]
    fn insert_kwarg_no_trailing_comma_with_comment_keeps_comma_adjacent_to_value() {
        // Hostile shape: last field has NO comma AND a trailing comment. The comma must attach
        // to the VALUE (before the comment), never after the comment (which would be swallowed).
        let buck = "rust_library(\n    name = \"lib\",\n    deps = []  # note\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let patched = insert_kwarg(buck, call, "visibility = [\"PUBLIC\"]").expect("insert");
        assert!(
            patched.contains("deps = [],  # note"),
            "comma before the comment: {patched}"
        );
        must_parse(&patched);
    }

    #[test]
    fn insert_dict_entry_into_literal_and_refusal_for_comprehension() {
        let buck = "rust_library(\n    name = \"a\",\n    mapped_srcs = {\n        \"//x:k\": \"x/k\",\n    },\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let Expr::Dict(dict) = &call.kwarg("mapped_srcs").unwrap().value.expr else {
            panic!("dict expected");
        };
        let patched = insert_dict_entry(buck, dict, "//y:new", "y/new").expect("insert");
        assert!(patched.contains("\"//y:new\": \"y/new\","), "{patched}");
        assert!(
            patched.contains("\"//x:k\": \"x/k\""),
            "existing entry intact: {patched}"
        );
        must_parse(&patched);

        let comp =
            "rust_library(\n    name = \"a\",\n    mapped_srcs = {src: src for src in SRCS},\n)\n";
        let comp_doc = must_parse(comp);
        let comp_call = calls(&comp_doc)[0];
        let Expr::Dict(comp_dict) = &comp_call.kwarg("mapped_srcs").unwrap().value.expr else {
            panic!("dict expected");
        };
        assert!(
            insert_dict_entry(comp, comp_dict, "//y:new", "y/new").is_err(),
            "a comprehension admits no inserted entries — refuse, never corrupt"
        );
    }

    #[test]
    fn remove_list_element_handles_middle_last_and_only() {
        let buck = "rust_library(\n    name = \"x\",\n    deps = [\n        \"third-party//:serde\",\n        \"third-party//:kube\",\n        \"third-party//:toml\",\n    ],\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        let Expr::List(list) = &call.kwarg("deps").unwrap().value.expr else {
            panic!("list expected");
        };
        // Middle element.
        let removed = remove_list_element(buck, list, 1).expect("remove middle");
        assert!(!removed.contains("kube"), "{removed}");
        assert!(
            removed.contains("serde") && removed.contains("toml"),
            "{removed}"
        );
        must_parse(&removed);

        // Last element without trailing comma.
        let last =
            "rust_library(\n    name = \"x\",\n    deps = [\"a\", \"third-party//:kube\"],\n)\n";
        let last_doc = must_parse(last);
        let Expr::List(last_list) = &calls(&last_doc)[0].kwarg("deps").unwrap().value.expr else {
            panic!("list expected");
        };
        let removed_last = remove_list_element(last, last_list, 1).expect("remove last");
        assert!(!removed_last.contains("kube"), "{removed_last}");
        assert!(removed_last.contains("\"a\""), "{removed_last}");
        must_parse(&removed_last);

        // Only element.
        let only = "rust_library(\n    name = \"x\",\n    deps = [\n        \"third-party//:kube\",\n    ],\n)\n";
        let only_doc = must_parse(only);
        let Expr::List(only_list) = &calls(&only_doc)[0].kwarg("deps").unwrap().value.expr else {
            panic!("list expected");
        };
        let removed_only = remove_list_element(only, only_list, 0).expect("remove only");
        assert!(!removed_only.contains("kube"), "{removed_only}");
        let redoc = must_parse(&removed_only);
        let Expr::List(empty) = &calls(&redoc)[0].kwarg("deps").unwrap().value.expr else {
            panic!("list expected");
        };
        assert!(empty.elements.is_empty(), "deps now empty: {removed_only}");
    }

    #[test]
    fn remove_list_element_preserves_a_trailing_comment_on_the_line() {
        let buck = "rust_library(\n    name = \"x\",\n    deps = [\n        \"third-party//:kube\",  # transient\n        \"third-party//:serde\",\n    ],\n)\n";
        let doc = must_parse(buck);
        let Expr::List(list) = &calls(&doc)[0].kwarg("deps").unwrap().value.expr else {
            panic!("list expected");
        };
        let removed = remove_list_element(buck, list, 0).expect("remove");
        assert!(!removed.contains("third-party//:kube"), "{removed}");
        assert!(removed.contains("serde"), "{removed}");
        must_parse(&removed);
    }

    // =====================================================================
    // Harness: corruption refusal + first-pre-image rollback
    // =====================================================================

    #[test]
    fn harness_refuses_structurally_corrupt_rewrite_and_returns_pre_image() {
        // Historical vector: a double comma from a comment-blind heuristic. The reparse step
        // must refuse and hand back the pre-image.
        let pre = "rust_library(\n    name = \"x\",\n    deps = [],\n)\n";
        let corrupt = "rust_library(\n    name = \"x\",\n    deps = [],,\n)\n";
        let mut registry = PreImageRegistry::new();
        let refusal = guarded_rewrite("p/BUCK", pre, corrupt, &mut registry, |_, _| Ok(()))
            .expect_err("corrupt rewrite must be refused");
        assert_eq!(refusal.pre_image, pre, "pre-image returned verbatim");
        assert!(
            refusal.reason.contains("structurally corrupt"),
            "{}",
            refusal.reason
        );
        assert_eq!(
            registry.get("p/BUCK"),
            Some(pre),
            "registry holds the pre-image"
        );
    }

    #[test]
    fn harness_refuses_on_semantic_hook_failure() {
        // Historical vector: dangling refs the structural parse cannot see (e.g. a feature
        // entry referencing a removed dep) — the CALLER-SUPPLIED hook refuses them.
        let pre = "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:kube\"],\n)\n";
        let candidate = "rust_library(\n    name = \"x\",\n    deps = [],\n)\n";
        let mut registry = PreImageRegistry::new();
        let refusal = guarded_rewrite("p/BUCK", pre, candidate, &mut registry, |_, _| {
            Err("feature `k8s` includes dep:kube but kube is no longer a dependency".to_owned())
        })
        .expect_err("semantic failure must refuse");
        assert_eq!(refusal.pre_image, pre);
        assert!(
            refusal.reason.contains("dangling") || refusal.reason.contains("dep:kube"),
            "{}",
            refusal.reason
        );
    }

    #[test]
    fn harness_passes_a_sound_rewrite_through_with_hook_visibility() {
        let pre = "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:kube\", \"third-party//:serde\"],\n)\n";
        let candidate =
            "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:serde\"],\n)\n";
        let mut registry = PreImageRegistry::new();
        let out = guarded_rewrite("p/BUCK", pre, candidate, &mut registry, |doc, _| {
            // The hook sees the PARSED candidate: assert the dep is gone and the target intact.
            let call = doc
                .stmts
                .iter()
                .find_map(|s| match s {
                    Stmt::Call(c) if c.func == "rust_library" => Some(c),
                    _ => None,
                })
                .ok_or("target vanished")?;
            let strings = call_strings(call);
            if strings.iter().any(|s| s.contains("third-party//:kube")) {
                return Err("kube still present".to_owned());
            }
            if !strings.iter().any(|s| s.contains("third-party//:serde")) {
                return Err("serde collateral-removed".to_owned());
            }
            Ok(())
        })
        .expect("sound rewrite passes");
        assert_eq!(out, candidate);
    }

    #[test]
    fn pre_image_registry_keeps_first_image_per_path_deterministically() {
        // #693 LOW-X3: a file edited twice must roll back to its ORIGINAL content.
        let mut registry = PreImageRegistry::new();
        registry.record("b/Cargo.toml", "ORIGINAL-B");
        registry.record("a/Cargo.toml", "ORIGINAL-A");
        registry.record("b/Cargo.toml", "INTERMEDIATE-B (one edit applied)");
        assert_eq!(
            registry.get("b/Cargo.toml"),
            Some("ORIGINAL-B"),
            "first image wins"
        );
        let order: Vec<&str> = registry.images().map(|(path, _)| path).collect();
        assert_eq!(
            order,
            vec!["a/Cargo.toml", "b/Cargo.toml"],
            "rollback order is deterministic"
        );
        assert_eq!(registry.len(), 2);
    }

    // =====================================================================
    // Reviewer BLOCKER/MED closure: wrapped calls + trailing-token honesty
    // =====================================================================

    #[test]
    fn trailing_tokens_after_a_modeled_statement_demote_it_to_opaque() {
        // `X = 1 if c else f(...)` — parse_expr models the `1`; the tail would previously be
        // consumed silently (reviewer MED: violated "never silently dropped"). The whole
        // statement must demote to Stmt::Opaque with a span covering the tail.
        let buck = "X = 1 if c else rust_library(name = \"x\")\nY = \"clean\"\n";
        let doc = must_parse(buck);
        assert!(
            matches!(&doc.stmts[0], Stmt::Opaque { span } if span.slice(buck).contains("rust_library")),
            "trailing-token statement must be opaque with the tail visible: {:?}",
            doc.stmts[0]
        );
        assert!(
            matches!(&doc.stmts[1], Stmt::Assign { name, .. } if name == "Y"),
            "the clean following statement still parses: {:?}",
            doc.stmts[1]
        );
    }

    #[test]
    fn visit_calls_reaches_assign_wrapped_and_nested_calls() {
        // Reviewer BLOCKER class: a call wrapped in an assignment or nested in an expression
        // must be enumerable — `X = rust_library(...)` can never hide from visit_calls.
        let buck = concat!(
            "X = rust_library(\n    name = \"wrapped\",\n    deps = [\"third-party//:kube\"],\n)\n",
            "M = {\"k\": helper(rust_library(name = \"nested\"))}\n",
            "rust_library(\n    name = \"plain\",\n)\n",
        );
        let doc = must_parse(buck);
        let mut seen: Vec<String> = Vec::new();
        doc.visit_calls(&mut |call| {
            if call.func == "rust_library"
                && let Some(arg) = call.kwarg("name")
                && let Expr::Str(name) = &arg.value.expr
            {
                seen.push(name.clone());
            }
        });
        seen.sort();
        assert_eq!(
            seen,
            vec!["nested", "plain", "wrapped"],
            "every wrapping shape enumerated"
        );
    }

    // =====================================================================
    // Span fidelity
    // =====================================================================

    #[test]
    fn spans_are_byte_accurate_over_the_original_text() {
        let buck = "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:serde\"],\n)\n";
        let doc = must_parse(buck);
        let call = calls(&doc)[0];
        assert_eq!(call.span.slice(buck), buck.trim_end_matches('\n'));
        assert_eq!(&buck[call.open_paren..call.open_paren + 1], "(");
        assert_eq!(&buck[call.close_paren..call.close_paren + 1], ")");
        let deps = call.kwarg("deps").unwrap();
        assert_eq!(deps.value.span.slice(buck), "[\"third-party//:serde\"]");
        let Expr::List(list) = &deps.value.expr else {
            panic!("list")
        };
        assert_eq!(
            list.elements[0].value.span.slice(buck),
            "\"third-party//:serde\""
        );
    }
}
