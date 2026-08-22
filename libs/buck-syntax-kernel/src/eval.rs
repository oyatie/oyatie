//! Static evaluation helpers over the parsed [`BuckDoc`] (ADR-0549).
//!
//! Pure, fail-honest evaluation of the value shapes the gates consume: string concatenation
//! over top-level string variables, `glob([...])` pattern collection, dict literal/comprehension
//! destination values, and `VAR["k"] = expr` index assignments. Anything not statically
//! resolvable evaluates to `None` — the caller maps that to its existing skip/refusal
//! classification, never a guess.

use std::collections::BTreeMap;

use crate::parser::{BuckDoc, CallExpr, DictExpr, Expr, ExprNode, Stmt};

/// Top-level variable bindings resolved from a document: `IDENT = "string"` and
/// `IDENT = glob([...])`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Env {
    pub string_vars: BTreeMap<String, String>,
    pub glob_vars: BTreeMap<String, Vec<String>>,
}

impl Env {
    /// Build an env from a document's top-level assignments. A string var binds only when the
    /// RHS is exactly one string literal; a glob var binds when the RHS is a `glob(...)` call
    /// (patterns = every string literal in the call, mirroring the prior gate parsers).
    pub fn from_doc(doc: &BuckDoc) -> Self {
        let mut env = Env::default();
        for stmt in &doc.stmts {
            if let Stmt::Assign { name, value, .. } = stmt {
                match &value.expr {
                    Expr::Str(s) => {
                        env.string_vars.insert(name.clone(), s.clone());
                    }
                    Expr::Call(call) if call.func == "glob" => {
                        env.glob_vars.insert(name.clone(), call_strings(call));
                    }
                    _ => {}
                }
            }
        }
        env
    }

    /// Build an env from caller-supplied slices (for consumers that already carry their own
    /// var lists; keeps migrated signatures behavior-identical).
    pub fn from_slices(
        string_vars: &[(String, String)],
        glob_vars: &[(String, Vec<String>)],
    ) -> Self {
        Env {
            string_vars: string_vars.iter().cloned().collect(),
            glob_vars: glob_vars.iter().cloned().collect(),
        }
    }
}

/// Every string literal anywhere inside an expression, in source order.
pub fn expr_strings(node: &ExprNode) -> Vec<String> {
    let mut out = Vec::new();
    collect_strings(node, &mut out);
    out
}

fn collect_strings(node: &ExprNode, out: &mut Vec<String>) {
    match &node.expr {
        Expr::Str(s) => out.push(s.clone()),
        Expr::Plus(operands) => {
            for operand in operands {
                collect_strings(operand, out);
            }
        }
        Expr::List(list) => {
            for element in &list.elements {
                collect_strings(&element.value, out);
            }
        }
        Expr::Dict(dict) => {
            for entry in &dict.entries {
                collect_strings(&entry.key, out);
                collect_strings(&entry.value, out);
            }
            if let Some(comp) = &dict.comprehension {
                collect_strings(&comp.key, out);
                collect_strings(&comp.value, out);
            }
        }
        Expr::Call(call) => out.extend(call_strings(call)),
        Expr::Int(_) | Expr::Ident(_) | Expr::Opaque => {}
    }
}

/// Every string literal anywhere inside a call's arguments, in source order.
pub fn call_strings(call: &CallExpr) -> Vec<String> {
    let mut out = Vec::new();
    for arg in &call.args {
        collect_strings(&arg.value, &mut out);
    }
    out
}

/// Statically evaluate a string-valued expression: a string literal, a string var reference, or
/// a `+` chain of those (with a loop variable optionally bound). `None` when any operand is not
/// statically resolvable.
pub fn eval_string(node: &ExprNode, env: &Env) -> Option<String> {
    eval_string_with(node, env, None)
}

/// [`eval_string`] with a comprehension loop variable bound to a concrete value.
pub fn eval_string_with(node: &ExprNode, env: &Env, bound: Option<(&str, &str)>) -> Option<String> {
    match &node.expr {
        Expr::Str(s) => Some(s.clone()),
        Expr::Ident(name) => {
            if let Some((var, value)) = bound
                && name == var
            {
                return Some(value.to_owned());
            }
            env.string_vars.get(name).cloned()
        }
        Expr::Plus(operands) => {
            let mut out = String::new();
            for operand in operands {
                out.push_str(&eval_string_with(operand, env, bound)?);
            }
            Some(out)
        }
        _ => None,
    }
}

/// Resolve a dict expression's destination VALUES (the right-hand sides):
/// - literal entries: `eval_string` of each value, falling back to the value's first string
///   literal (prior-parser parity for shapes like a nested call);
/// - a comprehension `{K: V for x in ITER}` where ITER is a glob var: expand the patterns
///   against `files` and evaluate V per match.
pub fn dict_values(dict: &DictExpr, env: &Env, files: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(comp) = &dict.comprehension {
        let Some(iter_name) = &comp.iter else {
            return out;
        };
        let Some(patterns) = env.glob_vars.get(iter_name) else {
            return out;
        };
        for file in files {
            if patterns.iter().any(|pattern| glob_match(pattern, file))
                && let Some(value) = eval_string_with(&comp.value, env, Some((&comp.var, file)))
            {
                out.push(value);
            }
        }
        return out;
    }
    for entry in &dict.entries {
        if let Some(value) = eval_string(&entry.value, env) {
            out.push(value);
        } else {
            let strings = expr_strings(&entry.value);
            if let Some(first) = strings.first() {
                out.push(first.clone());
            }
        }
    }
    out
}

/// Resolve the destination VALUES of a top-level dict variable `var` assembled as
/// `VAR = { ... }` (literal or comprehension) plus `VAR["k"] = expr` index assignments,
/// in document order.
pub fn resolve_dict_var(doc: &BuckDoc, var: &str, env: &Env, files: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    for stmt in &doc.stmts {
        match stmt {
            Stmt::Assign { name, value, .. } if name == var => {
                if let Expr::Dict(dict) = &value.expr {
                    values.extend(dict_values(dict, env, files));
                }
            }
            Stmt::IndexAssign { base, value, .. } if base == var => {
                if let Some(resolved) = eval_string(value, env) {
                    values.push(resolved);
                }
            }
            _ => {}
        }
    }
    values
}

/// Find the top-level call (optionally restricted to `kinds`) whose `name` kwarg statically
/// resolves to `target_name`. Sound binding by the ACTUAL name field — never a first-occurrence
/// substring match (the ADR-0545 residual this kernel retires).
pub fn find_target<'doc>(
    doc: &'doc BuckDoc,
    kinds: Option<&[&str]>,
    target_name: &str,
    env: &Env,
) -> Option<&'doc CallExpr> {
    doc.stmts.iter().find_map(|stmt| {
        let Stmt::Call(call) = stmt else { return None };
        if let Some(kinds) = kinds
            && !kinds.contains(&call.func.as_str())
        {
            return None;
        }
        let name_arg = call.kwarg("name")?;
        if eval_string(&name_arg.value, env).as_deref() == Some(target_name) {
            Some(call)
        } else {
            None
        }
    })
}

/// Match a buck2-style glob (`**` = any number of path segments incl. zero; `*` = any run of
/// characters within one segment) against a `/`-separated relative path. Pure; no filesystem.
/// Ported verbatim from the proven cloud-ci-embedded-asset-hermeticity-app matcher.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pat: Vec<&str> = pattern.split('/').collect();
    let txt: Vec<&str> = path.split('/').collect();
    glob_segments(&pat, &txt)
}

fn glob_segments(pat: &[&str], txt: &[&str]) -> bool {
    if pat.is_empty() {
        return txt.is_empty();
    }
    if pat[0] == "**" {
        if glob_segments(&pat[1..], txt) {
            return true;
        }
        if !txt.is_empty() {
            return glob_segments(pat, &txt[1..]);
        }
        return false;
    }
    if txt.is_empty() {
        return false;
    }
    if !segment_match(pat[0].as_bytes(), txt[0].as_bytes()) {
        return false;
    }
    glob_segments(&pat[1..], &txt[1..])
}

fn segment_match(pat: &[u8], txt: &[u8]) -> bool {
    let (mut p, mut t) = (0usize, 0usize);
    let (mut star_p, mut star_t): (Option<usize>, usize) = (None, 0);
    while t < txt.len() {
        if p < pat.len() && pat[p] == b'*' {
            star_p = Some(p);
            star_t = t;
            p += 1;
        } else if p < pat.len() && pat[p] == txt[t] {
            p += 1;
            t += 1;
        } else if let Some(sp) = star_p {
            p = sp + 1;
            star_t += 1;
            t = star_t;
        } else {
            return false;
        }
    }
    while p < pat.len() && pat[p] == b'*' {
        p += 1;
    }
    p == pat.len()
}
