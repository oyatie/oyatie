//! Fail closed when branch protection cannot prove a required check source app.
//!
//! Input is the JSON shape returned by GitHub's branch-protection
//! `required_status_checks` endpoint or an equivalent fixture. This checker is
//! read-only: it never mutates branch protection and it is local/live-read
//! evidence only until the trusted cloud-ci/oya-ci producer is deployed and
//! bound.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{self, Read};

const DEFAULT_REQUIRED_CONTEXT: &str = "oya-ci-required";
const AUTHORITY_BOUNDARY: &str = "required-status source binding evidence only; this checker never mutates branch protection or posts statuses";

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Json::Object(value) => Some(value),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(value) => Some(value),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub context: String,
    pub app_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub required_context: String,
    pub expected_source_app_id: Option<i64>,
    pub contexts: Vec<String>,
    pub checks: Vec<Check>,
    pub observed_source_app_id: Option<i64>,
    pub verdict: String,
    pub reason: String,
    pub required_context_source_app_bound: bool,
    pub trusted_source_app_proven: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub authority_boundary: String,
}

struct Parser<'a> {
    text: &'a [u8],
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text: text.as_bytes(),
            index: 0,
        }
    }

    fn parse(mut self) -> Result<Json, String> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.index == self.text.len() {
            Ok(value)
        } else {
            Err(format!("unexpected trailing JSON at byte {}", self.index))
        }
    }

    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'\"') => self.parse_string().map(Json::String),
            Some(b't') => self.expect_literal(b"true", Json::Bool(true)),
            Some(b'f') => self.expect_literal(b"false", Json::Bool(false)),
            Some(b'n') => self.expect_literal(b"null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(Json::Number),
            Some(byte) => Err(format!(
                "unexpected JSON byte {} at byte {}",
                byte as char, self.index
            )),
            None => Err("unexpected end of JSON".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.consume(b'{')?;
        let mut object = BTreeMap::new();
        self.skip_ws();
        if self.try_consume(b'}') {
            return Ok(Json::Object(object));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.consume(b':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_ws();
            if self.try_consume(b'}') {
                break;
            }
            self.consume(b',')?;
        }
        Ok(Json::Object(object))
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.consume(b'[')?;
        let mut array = Vec::new();
        self.skip_ws();
        if self.try_consume(b']') {
            return Ok(Json::Array(array));
        }
        loop {
            array.push(self.parse_value()?);
            self.skip_ws();
            if self.try_consume(b']') {
                break;
            }
            self.consume(b',')?;
        }
        Ok(Json::Array(array))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume(b'\"')?;
        let mut out = String::new();
        while let Some(byte) = self.next() {
            match byte {
                b'\"' => return Ok(out),
                b'\\' => {
                    let escaped = self
                        .next()
                        .ok_or_else(|| "unterminated JSON string escape".to_string())?;
                    match escaped {
                        b'\"' => out.push('\"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(
                            char::from_u32(self.parse_unicode_escape()?).unwrap_or('\u{fffd}'),
                        ),
                        other => {
                            return Err(format!(
                                "invalid JSON string escape {} at byte {}",
                                other as char, self.index
                            ));
                        }
                    }
                }
                other => out.push(other as char),
            }
        }
        Err("unterminated JSON string".to_string())
    }

    fn parse_unicode_escape(&mut self) -> Result<u32, String> {
        let mut value = 0u32;
        for _ in 0..4 {
            let byte = self
                .next()
                .ok_or_else(|| "short JSON unicode escape".to_string())?;
            value = value * 16
                + match byte {
                    b'0'..=b'9' => (byte - b'0') as u32,
                    b'a'..=b'f' => (byte - b'a' + 10) as u32,
                    b'A'..=b'F' => (byte - b'A' + 10) as u32,
                    _ => return Err("invalid JSON unicode escape".to_string()),
                };
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.index;
        if self.peek() == Some(b'-') {
            self.index += 1;
        }
        self.consume_digits();
        if self.peek() == Some(b'.') {
            self.index += 1;
            self.consume_digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            self.consume_digits();
        }
        std::str::from_utf8(&self.text[start..self.index])
            .map(|value| value.to_string())
            .map_err(|error| format!("invalid JSON number utf8: {error}"))
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.index += 1;
        }
    }

    fn expect_literal(&mut self, literal: &[u8], value: Json) -> Result<Json, String> {
        if self.text.get(self.index..self.index + literal.len()) == Some(literal) {
            self.index += literal.len();
            Ok(value)
        } else {
            Err(format!("expected literal at byte {}", self.index))
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.index += 1;
        }
    }

    fn consume(&mut self, expected: u8) -> Result<(), String> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            Some(actual) => Err(format!(
                "expected {} at byte {}, got {}",
                expected as char,
                self.index.saturating_sub(1),
                actual as char
            )),
            None => Err(format!("expected {} at end of JSON", expected as char)),
        }
    }

    fn try_consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.index += 1;
        Some(value)
    }

    fn peek(&self) -> Option<u8> {
        self.text.get(self.index).copied()
    }
}

pub fn parse_json(text: &str) -> Result<Json, String> {
    Parser::new(text).parse()
}

pub fn summarize(data: &Json, required_context: &str, expected_app_id: Option<i64>) -> Summary {
    let object = data.as_object();
    let contexts = object.map(normalize_contexts).unwrap_or_default();
    let checks_present = object
        .and_then(|item| item.get("checks"))
        .is_some_and(|checks| matches!(checks, Json::Array(_)));
    let checks = object.map(normalize_checks).unwrap_or_default();
    let mut summary = Summary {
        required_context: required_context.to_string(),
        expected_source_app_id: expected_app_id,
        contexts: contexts.clone(),
        checks: checks.clone(),
        observed_source_app_id: None,
        verdict: "FAIL".to_string(),
        reason: String::new(),
        required_context_source_app_bound: false,
        trusted_source_app_proven: false,
        p0_0_green: false,
        phase0_complete: false,
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
    };

    if !contexts.iter().any(|context| context == required_context) {
        summary.reason = "missing_required_context".to_string();
        return summary;
    }

    if !checks_present {
        summary.reason = "missing_required_status_checks_checks_array".to_string();
        return summary;
    }

    let matching = checks
        .iter()
        .find(|check| check.context == required_context);
    let Some(winning) = matching else {
        summary.reason = "required_context_not_in_checks_array".to_string();
        return summary;
    };
    summary.observed_source_app_id = winning.app_id;

    let Some(app_id) = winning.app_id else {
        summary.reason = "missing_required_status_source_app".to_string();
        return summary;
    };

    if app_id == -1 {
        summary.reason = "wildcard_required_status_source_app".to_string();
        return summary;
    }

    summary.required_context_source_app_bound = true;
    let Some(expected_app_id) = expected_app_id else {
        summary.reason = "expected_source_app_id_not_configured".to_string();
        return summary;
    };

    if app_id != expected_app_id {
        summary.reason = "wrong_required_status_source_app".to_string();
        return summary;
    }

    summary.verdict = "PASS".to_string();
    summary.reason = "required_status_source_app_bound".to_string();
    summary.trusted_source_app_proven = true;
    summary
}

fn normalize_contexts(object: &BTreeMap<String, Json>) -> Vec<String> {
    object
        .get("contexts")
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_checks(object: &BTreeMap<String, Json>) -> Vec<Check> {
    object
        .get("checks")
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_object)
                .filter_map(|check| {
                    Some(Check {
                        context: check.get("context")?.as_str()?.to_string(),
                        app_id: app_id_value(check.get("app_id")),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn app_id_value(value: Option<&Json>) -> Option<i64> {
    match value? {
        Json::Number(raw) | Json::String(raw) => raw.parse::<i64>().ok(),
        _ => None,
    }
}

pub fn to_json(summary: &Summary) -> String {
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"checks\":{},",
            "\"contexts\":{},",
            "\"expected_source_app_id\":{},",
            "\"observed_source_app_id\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"reason\":{},",
            "\"required_context\":{},",
            "\"required_context_source_app_bound\":{},",
            "\"trusted_source_app_proven\":{},",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&summary.authority_boundary),
        checks_json(&summary.checks),
        string_array_json(&summary.contexts),
        opt_i64_json(summary.expected_source_app_id),
        opt_i64_json(summary.observed_source_app_id),
        json_string(&summary.reason),
        json_string(&summary.required_context),
        bool_json(summary.required_context_source_app_bound),
        bool_json(summary.trusted_source_app_proven),
        json_string(&summary.verdict),
    )
}

fn checks_json(checks: &[Check]) -> String {
    format!(
        "[{}]",
        checks
            .iter()
            .map(|check| {
                format!(
                    "{{\"app_id\":{},\"context\":{}}}",
                    opt_i64_json(check.app_id),
                    json_string(&check.context)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn string_array_json(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn opt_i64_json(value: Option<i64>) -> String {
    value
        .map(|item| item.to_string())
        .unwrap_or_else(|| "null".to_string())
}

fn bool_json(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('\"');
    out
}

fn load_input(path: &str) -> Result<Json, String> {
    let text = if path == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| format!("read stdin failed: {error}"))?;
        input
    } else {
        fs::read_to_string(path).map_err(|error| format!("read {path} failed: {error}"))?
    };
    parse_json(&text).map_err(|error| format!("parse {path} failed: {error}"))
}

fn main() {
    let mut input: Option<String> = None;
    let mut required_context = DEFAULT_REQUIRED_CONTEXT.to_string();
    let mut expected_app_id: Option<i64> = None;
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = args.next(),
            "--required-context" => {
                required_context = args.next().expect("--required-context requires a value")
            }
            "--expected-app-id" => {
                let raw = args.next().expect("--expected-app-id requires a value");
                expected_app_id =
                    Some(raw.parse::<i64>().unwrap_or_else(|error| {
                        panic!("invalid --expected-app-id {raw}: {error}")
                    }));
            }
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let input = input.expect("--input is required");
    let data = load_input(&input).unwrap_or_else(|error| panic!("{error}"));
    let summary = summarize(&data, &required_context, expected_app_id);
    let rendered = to_json(&summary);
    if emit_json || summary.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if summary.verdict != "PASS" {
        std::process::exit(1);
    }
}
