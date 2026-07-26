//! Kernel command-line parsing.
//!
//! Talos is configured heavily through `/proc/cmdline`. PID 1 / early userspace
//! reads kernel parameters such as `talos.platform=`, `talos.config=` (where to
//! fetch the machine config), `console=`, `init_on_alloc=`, `slab_nomerge`, and
//! board/hostname overrides. This module is a small, pure parser for the
//! `key`, `key=value`, and `key="quoted value"` forms the kernel uses, plus
//! Talos-specific accessors.
//!
//! It is entirely host-testable: feed it a string, ask for parameters.

use std::collections::BTreeMap;

/// A parsed kernel command line. Preserves insertion order of distinct keys for
/// stable iteration, and keeps repeated keys (the kernel allows e.g. multiple
/// `console=`).
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CmdLine {
    /// All params in order, including repeats.
    params: Vec<(String, Option<String>)>,
}

impl CmdLine {
    /// Parse a `/proc/cmdline`-style string. Whitespace-separated tokens; a
    /// token may be a bare flag, `k=v`, or `k="v with spaces"`.
    pub fn parse(input: &str) -> CmdLine {
        let mut params = Vec::new();
        for token in tokenize(input) {
            match token.split_once('=') {
                Some((k, v)) => {
                    let v = strip_quotes(v);
                    params.push((k.to_string(), Some(v.to_string())));
                }
                None => params.push((token, None)),
            }
        }
        CmdLine { params }
    }

    /// Last value for `key` (kernel "last wins" semantics for duplicates).
    /// Returns `Some("")` for a bare flag, `Some(v)` for `k=v`, `None` if absent.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_deref().unwrap_or(""))
    }

    /// True if `key` appears at all (flag or assignment).
    pub fn has(&self, key: &str) -> bool {
        self.params.iter().any(|(k, _)| k == key)
    }

    /// All values for a repeated key, in order (e.g. every `console=`).
    pub fn all(&self, key: &str) -> Vec<&str> {
        self.params
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_deref().unwrap_or(""))
            .collect()
    }

    /// Number of parameters parsed.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// True if no parameters.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Collapse to a map of last-value-per-key (loses repeats; convenience).
    pub fn to_map(&self) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        for (k, v) in &self.params {
            m.insert(k.clone(), v.clone().unwrap_or_default());
        }
        m
    }

    // --- Talos-specific accessors ------------------------------------------

    /// `talos.platform=` (e.g. `metal`, `aws`, `gcp`, `nocloud`).
    pub fn platform(&self) -> Option<&str> {
        self.get("talos.platform")
    }

    /// `talos.config=` — URL/source the machine config is fetched from.
    pub fn config_source(&self) -> Option<&str> {
        self.get("talos.config")
    }

    /// `talos.hostname=` override.
    pub fn hostname(&self) -> Option<&str> {
        self.get("talos.hostname")
    }

    /// `talos.board=` (e.g. `rpi_4`, `bananapi_m64`).
    pub fn board(&self) -> Option<&str> {
        self.get("talos.board")
    }

    /// All `console=` devices, in kernel order. The *last* one is where the
    /// kernel sends `/dev/console`; init should attach stdio to it.
    pub fn consoles(&self) -> Vec<&str> {
        self.all("console")
    }

    /// The primary console device init should bind stdio to (last `console=`),
    /// stripped of its baud/options suffix. Defaults to `tty0`.
    pub fn primary_console(&self) -> String {
        match self.consoles().last() {
            Some(c) => {
                // `ttyS0,115200n8` -> `ttyS0`
                let dev = c.split(',').next().unwrap_or(c);
                dev.to_string()
            }
            None => "tty0".to_string(),
        }
    }

    /// Whether the kernel was told to be quiet (`quiet` flag present).
    pub fn quiet(&self) -> bool {
        self.has("quiet")
    }

    /// `talos.shutdown=` mode (`halt` or `poweroff`), defaulting to `poweroff`.
    pub fn shutdown_mode(&self) -> ShutdownMode {
        match self.get("talos.shutdown") {
            Some("halt") => ShutdownMode::Halt,
            _ => ShutdownMode::PowerOff,
        }
    }
}

/// What to do at the end of the boot lifecycle / on shutdown request.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ShutdownMode {
    #[default]
    PowerOff,
    Halt,
}

/// Split a command line into tokens, honoring `"double"` and `'single'` quotes
/// so a quoted value may contain spaces.
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;

    for ch in input.chars() {
        match quote {
            Some(q) => {
                if ch == q {
                    quote = None;
                } else {
                    cur.push(ch);
                }
            }
            None => {
                if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                    started = true;
                } else if ch.is_whitespace() {
                    if started {
                        tokens.push(std::mem::take(&mut cur));
                        started = false;
                    }
                } else {
                    cur.push(ch);
                    started = true;
                }
            }
        }
    }
    if started {
        tokens.push(cur);
    }
    tokens
}

/// Remove a single layer of matching surrounding quotes.
fn strip_quotes(v: &str) -> &str {
    let bytes = v.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        &v[1..v.len() - 1]
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flags_and_assignments() {
        let c = CmdLine::parse("quiet ro talos.platform=metal init=/init");
        assert!(c.has("quiet"));
        assert!(c.has("ro"));
        assert_eq!(c.get("talos.platform"), Some("metal"));
        assert_eq!(c.get("init"), Some("/init"));
        assert_eq!(c.get("missing"), None);
    }

    #[test]
    fn bare_flag_reports_empty_value() {
        let c = CmdLine::parse("debug");
        assert_eq!(c.get("debug"), Some(""));
        assert!(c.has("debug"));
    }

    #[test]
    fn quoted_value_keeps_spaces() {
        let c = CmdLine::parse(r#"talos.config="https://x/y?a=b c=d""#);
        assert_eq!(c.config_source(), Some("https://x/y?a=b c=d"));
    }

    #[test]
    fn last_value_wins_for_duplicates() {
        let c = CmdLine::parse("talos.hostname=a talos.hostname=b");
        assert_eq!(c.hostname(), Some("b"));
    }

    #[test]
    fn multiple_consoles_preserved_in_order() {
        let c = CmdLine::parse("console=tty0 console=ttyS0,115200n8");
        assert_eq!(c.consoles(), vec!["tty0", "ttyS0,115200n8"]);
        // Primary is the last, stripped of options.
        assert_eq!(c.primary_console(), "ttyS0");
    }

    #[test]
    fn primary_console_defaults_to_tty0() {
        let c = CmdLine::parse("quiet");
        assert_eq!(c.primary_console(), "tty0");
    }

    #[test]
    fn talos_accessors() {
        let c = CmdLine::parse(
            "talos.platform=aws talos.board=rpi_4 talos.hostname=node1 talos.config=http://meta",
        );
        assert_eq!(c.platform(), Some("aws"));
        assert_eq!(c.board(), Some("rpi_4"));
        assert_eq!(c.hostname(), Some("node1"));
        assert_eq!(c.config_source(), Some("http://meta"));
    }

    #[test]
    fn shutdown_mode_parsing() {
        assert_eq!(
            CmdLine::parse("talos.shutdown=halt").shutdown_mode(),
            ShutdownMode::Halt
        );
        assert_eq!(
            CmdLine::parse("talos.shutdown=poweroff").shutdown_mode(),
            ShutdownMode::PowerOff
        );
        // Default when unspecified.
        assert_eq!(
            CmdLine::parse("quiet").shutdown_mode(),
            ShutdownMode::PowerOff
        );
    }

    #[test]
    fn quiet_detection() {
        assert!(CmdLine::parse("ro quiet splash").quiet());
        assert!(!CmdLine::parse("ro splash").quiet());
    }

    #[test]
    fn empty_cmdline() {
        let c = CmdLine::parse("   ");
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
        assert_eq!(c.primary_console(), "tty0");
    }

    #[test]
    fn to_map_collapses_last_value() {
        let c = CmdLine::parse("a=1 b=2 a=3");
        let m = c.to_map();
        assert_eq!(m.get("a").map(String::as_str), Some("3"));
        assert_eq!(m.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn single_quoted_values() {
        let c = CmdLine::parse("msg='hello world'");
        assert_eq!(c.get("msg"), Some("hello world"));
    }

    #[test]
    fn all_returns_every_occurrence() {
        let c = CmdLine::parse("console=a console=b console=c");
        assert_eq!(c.all("console"), vec!["a", "b", "c"]);
    }
}
