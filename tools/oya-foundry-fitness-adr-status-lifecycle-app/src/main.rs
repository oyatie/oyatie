//! ADR-status lifecycle fitness dev-CLI (ADR-0109).
//!
//! Loads `specs/cross-cutting/lifecycle-configs/adr-status-lifecycle.json`
//! (or a path passed via `--config`), walks the ADR glob, parses YAML
//! front-matter for `status:` + `superseded_by:`, builds
//! `LifecycledArtifact` records, and calls the framework kernel.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_fitness_lifecycle_kernel::{
    Defaults, LifecycleConfig, LifecycledArtifact, LifecycleReport, NaiveDate, SourceSpec, Stage,
    Transition, evaluate,
};

const DEFAULT_CONFIG: &str = "specs/cross-cutting/lifecycle-configs/adr-status-lifecycle.json";

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(report) => {
            println!(
                "adr-status-lifecycle ok: artifacts_observed={} stage_counts={:?} violations=0",
                report.artifacts_observed, report.stage_counts,
            );
            ExitCode::SUCCESS
        }
        Err(LaneError::Violations(report)) => {
            eprintln!(
                "adr-status-lifecycle FAIL: artifacts_observed={} stage_counts={:?} violations={}",
                report.artifacts_observed,
                report.stage_counts,
                report.violations.len(),
            );
            for v in &report.violations {
                eprintln!(
                    "  - [{}] {} stage={:?} hint={}",
                    v.kind.as_str(),
                    v.location,
                    v.stage,
                    v.hint,
                );
            }
            ExitCode::FAILURE
        }
        Err(LaneError::Io(msg)) => {
            eprintln!("adr-status-lifecycle error: {msg}");
            ExitCode::FAILURE
        }
    }
}

enum LaneError {
    Violations(LifecycleReport),
    Io(String),
}

fn run<I>(args: I) -> Result<LifecycleReport, LaneError>
where
    I: IntoIterator<Item = String>,
{
    let opts = Options::parse(args).map_err(LaneError::Io)?;
    let config = load_config(&opts.config).map_err(LaneError::Io)?;
    let artifacts = discover_artifacts(&config).map_err(LaneError::Io)?;
    let now = today();
    let report = evaluate(&config, &artifacts, now, &opts.reached_milestones);
    if report.is_clean() && !opts.wave_warn_only {
        Ok(report)
    } else if opts.wave_warn_only {
        // Wave A: print findings but exit success.
        println!(
            "adr-status-lifecycle WARN (wave A): artifacts_observed={} stage_counts={:?} violations={}",
            report.artifacts_observed,
            report.stage_counts,
            report.violations.len()
        );
        for v in &report.violations {
            println!(
                "  - [{}] {} stage={:?} hint={}",
                v.kind.as_str(),
                v.location,
                v.stage,
                v.hint
            );
        }
        Ok(report)
    } else {
        Err(LaneError::Violations(report))
    }
}

struct Options {
    config: PathBuf,
    reached_milestones: Vec<String>,
    wave_warn_only: bool,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut config = PathBuf::from(DEFAULT_CONFIG);
        let mut reached_milestones: Vec<String> = Vec::new();
        let mut wave_warn_only = true; // Wave A default
        let args = args.into_iter().collect::<Vec<_>>();
        let mut i = 0usize;
        while i < args.len() {
            match args[i].as_str() {
                "--config" => {
                    i += 1;
                    config = PathBuf::from(args.get(i).ok_or("--config needs a path")?);
                }
                "--milestone" => {
                    i += 1;
                    reached_milestones
                        .push(args.get(i).ok_or("--milestone needs an id")?.to_string());
                }
                "--block" => wave_warn_only = false,
                "--warn-only" => wave_warn_only = true,
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
            i += 1;
        }
        Ok(Self {
            config,
            reached_milestones,
            wave_warn_only,
        })
    }
}

fn usage() -> String {
    "usage: oya-foundry-fitness-adr-status-lifecycle-app [--config PATH] [--milestone ID]... [--block|--warn-only]".into()
}

fn today() -> NaiveDate {
    // Deterministic enough for fitness use: parse YYYY-MM-DD from env, else fixed.
    if let Ok(s) = env::var("OYA_LIFECYCLE_NOW")
        && let Some(d) = parse_date(&s)
    {
        return d;
    }
    NaiveDate::ymd(2026, 5, 15)
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = s.splitn(3, '-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i32 = parts[0].parse().ok()?;
    let m: u8 = parts[1].parse().ok()?;
    let d: u8 = parts[2].parse().ok()?;
    Some(NaiveDate::ymd(y, m, d))
}

// --- Config loading (minimal JSON-subset parser; kernel is zero-dep) ----

fn load_config(path: &Path) -> Result<LifecycleConfig, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("could not read config {}: {e}", path.display()))?;
    parse_config_json(&raw)
}

fn parse_config_json(raw: &str) -> Result<LifecycleConfig, String> {
    // Use a tiny JSON parser to keep zero deps. We accept only the
    // schema documented in ADR-0109.
    let v = json::parse(raw)?;
    let name = v.field_str("name")?.to_string();
    let version = v.field_u32("version").unwrap_or(1);

    let stages = v
        .field_arr("stages")?
        .iter()
        .map(|s| {
            Ok::<_, String>(Stage {
                id: s.field_str("id")?.to_string(),
                terminal: s.field_bool("terminal").unwrap_or(false),
                requires_supersession_edge: s
                    .field_bool("requires_supersession_edge")
                    .unwrap_or(false),
                gated_by_milestone: s.field_str("gated_by_milestone").ok().map(String::from),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let transitions = v
        .field_arr("transitions")?
        .iter()
        .map(|t| {
            Ok::<_, String>(Transition {
                from: t.field_str("from")?.to_string(),
                to: t.field_str("to")?.to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let sources = v
        .field_arr("sources")?
        .iter()
        .map(|s| {
            Ok::<_, String>(SourceSpec {
                kind: s.field_str("kind")?.to_string(),
                glob: s.field_str("glob")?.to_string(),
                stage_field: s.field_str("stage_field")?.to_string(),
                supersession_field: s.field_str("supersession_field").ok().map(String::from),
                deadline_field: s.field_str("deadline_field").ok().map(String::from),
                milestone_field: s.field_str("milestone_field").ok().map(String::from),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let defaults = v
        .field_obj("defaults")
        .map(|d| Defaults {
            wave: d.field_str("wave").ok().map(String::from),
            case_insensitive_stage_match: d
                .field_bool("case_insensitive_stage_match")
                .unwrap_or(false),
        })
        .unwrap_or_default();
    Ok(LifecycleConfig {
        name,
        version,
        stages,
        transitions,
        sources,
        defaults,
    })
}

// --- Artifact discovery via glob + YAML-front-matter scalar parse ---------

fn discover_artifacts(config: &LifecycleConfig) -> Result<Vec<LifecycledArtifact>, String> {
    let mut out = Vec::new();
    for source in &config.sources {
        let entries = expand_glob(&source.glob)?;
        for path in entries {
            let raw = fs::read_to_string(&path)
                .map_err(|e| format!("could not read {}: {e}", path.display()))?;
            let stage = frontmatter::scalar(&raw, &source.stage_field);
            let supersession = source
                .supersession_field
                .as_deref()
                .and_then(|f| frontmatter::scalar(&raw, f));
            let deadline = source
                .deadline_field
                .as_deref()
                .and_then(|f| frontmatter::scalar(&raw, f))
                .and_then(|s| parse_date(&s));
            let milestone = source
                .milestone_field
                .as_deref()
                .and_then(|f| frontmatter::scalar(&raw, f));
            out.push(LifecycledArtifact {
                location: path.to_string_lossy().into_owned(),
                kind: config.name.clone(),
                current_stage: stage,
                observed_at: today(),
                deadline_at: deadline,
                history: vec![],
                supersession_target: supersession,
                milestone_anchor: milestone,
            });
        }
    }
    Ok(out)
}

fn expand_glob(glob: &str) -> Result<Vec<PathBuf>, String> {
    // Minimal `<dir>/<prefix>*<suffix>.<ext>` glob support. The configs
    // we ship use forms like `docs/decisions/ADR-*.md` and
    // `.omc/plans/**/*.md`. `**` is handled as recursive descent below.
    let glob = glob.trim();
    if let Some((head, _tail)) = glob.split_once("/**/") {
        return recursive(Path::new(head), &glob[head.len() + 4..]);
    }
    if let Some((dir, rest)) = glob.rsplit_once('/') {
        return shallow_glob(Path::new(dir), rest);
    }
    Err(format!("unsupported glob pattern: {glob}"))
}

fn shallow_glob(dir: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let entries = fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if matches_glob(&name, pattern) {
            out.push(entry.path());
        }
    }
    out.sort();
    Ok(out)
}

fn recursive(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                stack.push(path);
            } else if ft.is_file() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if matches_glob(&name, pattern) {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    Ok(out)
}

fn matches_glob(name: &str, pattern: &str) -> bool {
    // single-`*` wildcard.
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        return name.starts_with(prefix) && name.ends_with(suffix) && name.len() >= prefix.len() + suffix.len();
    }
    name == pattern
}

// --- Tiny embedded JSON + YAML-front-matter parsers (zero deps) ----------

mod json {
    use std::collections::HashMap;

    pub fn parse(input: &str) -> Result<Value, String> {
        let mut p = Parser { src: input.as_bytes(), pos: 0 };
        p.skip_ws();
        let v = p.parse_value()?;
        p.skip_ws();
        if p.pos != p.src.len() {
            return Err(format!("trailing input at byte {}", p.pos));
        }
        Ok(v)
    }

    pub enum Value {
        Null,
        Bool(bool),
        Number(f64),
        Str(String),
        Array(Vec<Value>),
        Object(HashMap<String, Value>),
    }

    impl Value {
        pub fn field_str(&self, name: &str) -> Result<&str, String> {
            match self {
                Value::Object(m) => match m.get(name) {
                    Some(Value::Str(s)) => Ok(s.as_str()),
                    Some(Value::Null) => Err(format!("field `{name}` is null")),
                    Some(_) => Err(format!("field `{name}` is not a string")),
                    None => Err(format!("field `{name}` missing")),
                },
                _ => Err("expected object".into()),
            }
        }
        pub fn field_bool(&self, name: &str) -> Result<bool, String> {
            match self {
                Value::Object(m) => match m.get(name) {
                    Some(Value::Bool(b)) => Ok(*b),
                    Some(_) => Err(format!("field `{name}` is not a bool")),
                    None => Err(format!("field `{name}` missing")),
                },
                _ => Err("expected object".into()),
            }
        }
        pub fn field_u32(&self, name: &str) -> Result<u32, String> {
            match self {
                Value::Object(m) => match m.get(name) {
                    Some(Value::Number(n)) => Ok(*n as u32),
                    Some(_) => Err(format!("field `{name}` is not a number")),
                    None => Err(format!("field `{name}` missing")),
                },
                _ => Err("expected object".into()),
            }
        }
        pub fn field_arr(&self, name: &str) -> Result<&Vec<Value>, String> {
            match self {
                Value::Object(m) => match m.get(name) {
                    Some(Value::Array(a)) => Ok(a),
                    Some(_) => Err(format!("field `{name}` is not an array")),
                    None => Err(format!("field `{name}` missing")),
                },
                _ => Err("expected object".into()),
            }
        }
        pub fn field_obj(&self, name: &str) -> Option<&Value> {
            match self {
                Value::Object(m) => match m.get(name) {
                    Some(v @ Value::Object(_)) => Some(v),
                    _ => None,
                },
                _ => None,
            }
        }
    }

    struct Parser<'a> {
        src: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        fn skip_ws(&mut self) {
            while self.pos < self.src.len() && (self.src[self.pos] as char).is_whitespace() {
                self.pos += 1;
            }
        }
        fn peek(&self) -> Option<u8> {
            self.src.get(self.pos).copied()
        }
        fn parse_value(&mut self) -> Result<Value, String> {
            self.skip_ws();
            match self.peek() {
                Some(b'{') => self.parse_object(),
                Some(b'[') => self.parse_array(),
                Some(b'"') => Ok(Value::Str(self.parse_string()?)),
                Some(b't') | Some(b'f') => self.parse_bool(),
                Some(b'n') => self.parse_null(),
                Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
                Some(c) => Err(format!("unexpected byte {c} at pos {}", self.pos)),
                None => Err("unexpected EOF".into()),
            }
        }
        fn parse_object(&mut self) -> Result<Value, String> {
            self.pos += 1;
            self.skip_ws();
            let mut map: HashMap<String, Value> = HashMap::new();
            if self.peek() == Some(b'}') {
                self.pos += 1;
                return Ok(Value::Object(map));
            }
            loop {
                self.skip_ws();
                let key = self.parse_string()?;
                self.skip_ws();
                if self.peek() != Some(b':') {
                    return Err("expected ':' in object".into());
                }
                self.pos += 1;
                let val = self.parse_value()?;
                map.insert(key, val);
                self.skip_ws();
                match self.peek() {
                    Some(b',') => {
                        self.pos += 1;
                    }
                    Some(b'}') => {
                        self.pos += 1;
                        break;
                    }
                    other => return Err(format!("expected ',' or '}}' got {other:?}")),
                }
            }
            Ok(Value::Object(map))
        }
        fn parse_array(&mut self) -> Result<Value, String> {
            self.pos += 1;
            self.skip_ws();
            let mut items = Vec::new();
            if self.peek() == Some(b']') {
                self.pos += 1;
                return Ok(Value::Array(items));
            }
            loop {
                items.push(self.parse_value()?);
                self.skip_ws();
                match self.peek() {
                    Some(b',') => {
                        self.pos += 1;
                    }
                    Some(b']') => {
                        self.pos += 1;
                        break;
                    }
                    other => return Err(format!("expected ',' or ']' got {other:?}")),
                }
            }
            Ok(Value::Array(items))
        }
        fn parse_string(&mut self) -> Result<String, String> {
            if self.peek() != Some(b'"') {
                return Err("expected '\"'".into());
            }
            self.pos += 1;
            let start = self.pos;
            while let Some(c) = self.peek() {
                if c == b'\\' {
                    self.pos += 2;
                    continue;
                }
                if c == b'"' {
                    let s = std::str::from_utf8(&self.src[start..self.pos])
                        .map_err(|e| format!("utf8: {e}"))?
                        .to_string();
                    self.pos += 1;
                    // Minimal unescape: \\, \", \n, \t.
                    let out = s
                        .replace("\\\\", "\u{0001}")
                        .replace("\\\"", "\"")
                        .replace("\\n", "\n")
                        .replace("\\t", "\t")
                        .replace('\u{0001}', "\\");
                    return Ok(out);
                }
                self.pos += 1;
            }
            Err("unterminated string".into())
        }
        fn parse_bool(&mut self) -> Result<Value, String> {
            if self.src[self.pos..].starts_with(b"true") {
                self.pos += 4;
                Ok(Value::Bool(true))
            } else if self.src[self.pos..].starts_with(b"false") {
                self.pos += 5;
                Ok(Value::Bool(false))
            } else {
                Err("expected bool".into())
            }
        }
        fn parse_null(&mut self) -> Result<Value, String> {
            if self.src[self.pos..].starts_with(b"null") {
                self.pos += 4;
                Ok(Value::Null)
            } else {
                Err("expected null".into())
            }
        }
        fn parse_number(&mut self) -> Result<Value, String> {
            let start = self.pos;
            if self.peek() == Some(b'-') {
                self.pos += 1;
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'-' || c == b'+' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let text = std::str::from_utf8(&self.src[start..self.pos])
                .map_err(|e| format!("utf8: {e}"))?;
            text.parse::<f64>()
                .map(Value::Number)
                .map_err(|e| format!("number: {e}"))
        }
    }
}

mod frontmatter {
    /// Returns the scalar value of `field` from the YAML front-matter
    /// block at the top of `raw`. Only handles the canonical form used
    /// by oyatie ADRs/plans: `field: value` on its own line. No nested
    /// support — the kernel input model expects flat scalar metadata.
    pub fn scalar(raw: &str, field: &str) -> Option<String> {
        let mut in_fm = false;
        let mut started = false;
        for line in raw.lines() {
            if line.trim() == "---" {
                if !started {
                    started = true;
                    in_fm = true;
                    continue;
                } else {
                    break;
                }
            }
            if !in_fm {
                continue;
            }
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix(field)
                && let Some(rest) = rest.strip_prefix(':')
            {
                let value = rest.trim().trim_matches('"').trim_matches('\'').trim();
                if value.is_empty() {
                    return None;
                }
                return Some(value.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_config() {
        let raw = r#"
        {
          "name": "x",
          "version": 1,
          "stages": [
            {"id":"a","terminal":false,"requires_supersession_edge":false},
            {"id":"b","terminal":true,"requires_supersession_edge":false}
          ],
          "transitions": [{"from":"a","to":"b"}],
          "sources": [],
          "defaults": {"case_insensitive_stage_match": true}
        }
        "#;
        let cfg = parse_config_json(raw).expect("config parses");
        assert_eq!(cfg.name, "x");
        assert_eq!(cfg.stages.len(), 2);
        assert!(cfg.defaults.case_insensitive_stage_match);
    }

    #[test]
    fn frontmatter_extracts_simple_scalar() {
        let raw = "---\nstatus: Accepted\nsuperseded_by: ADR-0099\n---\n\nbody";
        assert_eq!(
            frontmatter::scalar(raw, "status"),
            Some("Accepted".to_string())
        );
        assert_eq!(
            frontmatter::scalar(raw, "superseded_by"),
            Some("ADR-0099".to_string())
        );
        assert_eq!(frontmatter::scalar(raw, "missing"), None);
    }

    #[test]
    fn glob_matches_simple_pattern() {
        assert!(matches_glob("ADR-0001.md", "ADR-*.md"));
        assert!(!matches_glob("README.md", "ADR-*.md"));
    }
}
