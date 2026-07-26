//! Kernel command line (`/proc/cmdline`) parsing.
//!
//! Mirrors Talos `pkg/procfs/cmdline` (`procfs.NewCmdline` / `Cmdline.Get`),
//! which the machined uses to read boot-time parameters such as
//! `talos.platform=`, `talos.config=`, `console=`, `slab_nomerge`, and the
//! security-relevant lockdown / KSPP flags.
//!
//! The kernel cmdline is a single space-separated line. Each token is either a
//! bare flag (`slab_nomerge`) or `key=value`. Keys may repeat (e.g. multiple
//! `console=`), and values themselves may contain commas which Talos treats as
//! sub-values.

use std::collections::BTreeMap;
use std::fmt;

use crate::kernel_param::KernelParamError;

/// A single parsed cmdline parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdlineParam {
    /// Parameter key (`talos.platform`).
    pub key: String,
    /// All values supplied for this key, in order. A bare flag has no values.
    pub values: Vec<String>,
}

impl CmdlineParam {
    /// The first value, if any (the common single-value case).
    pub fn first(&self) -> Option<&str> {
        self.values.first().map(String::as_str)
    }

    /// Whether this is a bare flag (present but valueless).
    pub fn is_flag(&self) -> bool {
        self.values.is_empty()
    }
}

/// A parsed kernel command line. Preserves order and allows repeated keys.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cmdline {
    params: Vec<CmdlineParam>,
}

impl Cmdline {
    /// An empty cmdline.
    pub fn new() -> Self {
        Cmdline::default()
    }

    /// Parse a `/proc/cmdline` string into structured parameters.
    ///
    /// Tokens are split on ASCII whitespace. A token `k=v` becomes a key with a
    /// single value; a token `k=v1,v2` becomes a key with two values; a bare
    /// `flag` becomes a valueless param. An empty key (a leading `=`) is a parse
    /// error.
    pub fn parse(line: &str) -> Result<Self, KernelParamError> {
        let mut params: Vec<CmdlineParam> = Vec::new();
        for token in line.split_ascii_whitespace() {
            if token.is_empty() {
                continue;
            }
            let (key, values) = match token.split_once('=') {
                Some((k, v)) => {
                    if k.is_empty() {
                        return Err(KernelParamError::Parse(token.into()));
                    }
                    let vals = v.split(',').map(ToString::to_string).collect::<Vec<_>>();
                    (k.to_string(), vals)
                }
                None => (token.to_string(), Vec::new()),
            };
            // Merge repeated keys by appending values, matching how Talos
            // accumulates e.g. multiple `console=` entries.
            if let Some(existing) = params.iter_mut().find(|p| p.key == key) {
                existing.values.extend(values);
            } else {
                params.push(CmdlineParam { key, values });
            }
        }
        Ok(Cmdline { params })
    }

    /// Look up a parameter by key.
    pub fn get(&self, key: &str) -> Option<&CmdlineParam> {
        self.params.iter().find(|p| p.key == key)
    }

    /// The first value for a key, if present.
    pub fn get_first(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|p| p.first())
    }

    /// Whether a key (flag or key=value) is present at all.
    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Whether a bare flag is present (key present with no values).
    pub fn has_flag(&self, key: &str) -> bool {
        self.get(key).is_some_and(CmdlineParam::is_flag)
    }

    /// Number of distinct keys.
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Whether the cmdline is empty.
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Iterate over the parameters in order.
    pub fn iter(&self) -> impl Iterator<Item = &CmdlineParam> {
        self.params.iter()
    }

    /// Set/append a key=value, returning the modified cmdline (builder style).
    /// Used by Talos to inject parameters before re-serializing for a new boot.
    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let value = value.into();
        if let Some(existing) = self.params.iter_mut().find(|p| p.key == key) {
            existing.values.push(value);
        } else {
            self.params.push(CmdlineParam {
                key,
                values: vec![value],
            });
        }
        self
    }

    /// Whether the kernel was booted with security lockdown evidence, as Talos
    /// checks: `lockdown=` present or `module.sig_enforce=1`.
    pub fn lockdown_enabled(&self) -> bool {
        self.contains("lockdown") || self.get_first("module.sig_enforce") == Some("1")
    }

    /// Collect all values for a key into a flat, deduplicated-by-order list of
    /// comma sub-values. Convenience for keys like `console=` that repeat.
    pub fn all_values(&self, key: &str) -> Vec<&str> {
        self.get(key)
            .map(|p| p.values.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Remove a key (and all its values / flag) entirely. Returns true if a key
    /// was actually removed. Talos uses this to strip operator-deleted args
    /// (the `-key` syntax in `machine.install.extraKernelArgs`).
    pub fn remove(&mut self, key: &str) -> bool {
        let before = self.params.len();
        self.params.retain(|p| p.key != key);
        self.params.len() != before
    }

    /// Replace a key's value(s) with a single value, inserting if absent.
    /// Builder style. Unlike [`Cmdline::set`] (which appends), this overwrites.
    pub fn replace(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();
        let value = value.into();
        if let Some(existing) = self.params.iter_mut().find(|p| p.key == key) {
            existing.values = vec![value];
        } else {
            self.params.push(CmdlineParam {
                key,
                values: vec![value],
            });
        }
        self
    }

    /// Apply a Talos-style extra-args delta. Each token is either `key=value`
    /// (set/replace), a bare `flag` (add flag), or `-key` (delete). Mirrors the
    /// `extraKernelArgs` merge in `pkg/machinery/config`.
    pub fn apply_args(mut self, args: &[&str]) -> Result<Self, KernelParamError> {
        for raw in args {
            let token = raw.trim();
            if token.is_empty() {
                continue;
            }
            if let Some(key) = token.strip_prefix('-') {
                if key.is_empty() {
                    return Err(KernelParamError::Parse((*raw).into()));
                }
                self.remove(key);
            } else if let Some((k, v)) = token.split_once('=') {
                if k.is_empty() {
                    return Err(KernelParamError::Parse((*raw).into()));
                }
                self = self.replace(k, v);
            } else if !self.contains(token) {
                self.params.push(CmdlineParam {
                    key: token.to_string(),
                    values: Vec::new(),
                });
            }
        }
        Ok(self)
    }

    /// Serialize to an ordered vector of `key=value` / flag tokens, as a boot
    /// loader would pass them.
    pub fn to_args(&self) -> Vec<String> {
        self.params
            .iter()
            .map(|p| {
                if p.is_flag() {
                    p.key.clone()
                } else {
                    format!("{}={}", p.key, p.values.join(","))
                }
            })
            .collect()
    }

    /// The CPU vulnerability mitigation posture. Talos checks `mitigations=off`
    /// (insecure) versus the secure default of leaving mitigations enabled.
    pub fn mitigations_disabled(&self) -> bool {
        self.get_first("mitigations") == Some("off")
    }

    /// Whether the cmdline requests module signature enforcement
    /// (`module.sig_enforce=1`), part of the Secure Boot chain Talos relies on.
    pub fn module_sig_enforced(&self) -> bool {
        self.get_first("module.sig_enforce") == Some("1")
    }

    /// The requested lockdown level string, if any (`lockdown=confidentiality`).
    pub fn lockdown_level(&self) -> Option<&str> {
        self.get_first("lockdown")
    }

    /// The Talos platform from `talos.platform=` (`metal`, `aws`, ...).
    pub fn os_platform_domain(&self) -> Option<&str> {
        self.get_first("talos.platform")
    }

    /// The Talos machine-config source URL from `talos.config=`.
    pub fn talos_config_url(&self) -> Option<&str> {
        self.get_first("talos.config")
    }
}

impl fmt::Display for Cmdline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for p in &self.params {
            if !first {
                f.write_str(" ")?;
            }
            first = false;
            if p.is_flag() {
                f.write_str(&p.key)?;
            } else {
                write!(f, "{}={}", p.key, p.values.join(","))?;
            }
        }
        Ok(())
    }
}

/// Build a lookup map (last value wins per key). Convenience for code that just
/// needs scalar settings and does not care about repetition.
pub fn to_scalar_map(cmdline: &Cmdline) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for p in cmdline.iter() {
        if let Some(v) = p.values.last() {
            map.insert(p.key.clone(), v.clone());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mixed_flags_and_kv() {
        let cl = Cmdline::parse(
            "talos.platform=metal slab_nomerge console=ttyS0,115200 init_on_alloc=1",
        )
        .unwrap();
        assert_eq!(cl.get_first("talos.platform"), Some("metal"));
        assert!(cl.has_flag("slab_nomerge"));
        assert_eq!(cl.all_values("console"), vec!["ttyS0", "115200"]);
        assert_eq!(cl.get_first("init_on_alloc"), Some("1"));
    }

    #[test]
    fn repeated_keys_accumulate() {
        let cl = Cmdline::parse("console=tty0 console=ttyS0").unwrap();
        assert_eq!(cl.all_values("console"), vec!["tty0", "ttyS0"]);
        assert_eq!(cl.len(), 1);
    }

    #[test]
    fn empty_key_is_parse_error() {
        assert!(matches!(
            Cmdline::parse("=bad"),
            Err(KernelParamError::Parse(_))
        ));
    }

    #[test]
    fn lockdown_detection() {
        assert!(
            Cmdline::parse("lockdown=confidentiality")
                .unwrap()
                .lockdown_enabled()
        );
        assert!(
            Cmdline::parse("module.sig_enforce=1")
                .unwrap()
                .lockdown_enabled()
        );
        assert!(!Cmdline::parse("quiet").unwrap().lockdown_enabled());
    }

    #[test]
    fn set_and_display_roundtrip_shape() {
        let cl = Cmdline::new()
            .set("talos.platform", "aws")
            .set("console", "ttyS0");
        let s = cl.to_string();
        assert!(s.contains("talos.platform=aws"));
        assert!(s.contains("console=ttyS0"));
        // Re-parsing the rendered string yields the same keys.
        let reparsed = Cmdline::parse(&s).unwrap();
        assert_eq!(reparsed.get_first("talos.platform"), Some("aws"));
    }

    #[test]
    fn scalar_map_takes_last_value() {
        let cl = Cmdline::parse("x=1 x=2 y=a").unwrap();
        let map = to_scalar_map(&cl);
        assert_eq!(map.get("x").map(String::as_str), Some("2"));
        assert_eq!(map.get("y").map(String::as_str), Some("a"));
    }

    #[test]
    fn remove_deletes_key() {
        let mut cl = Cmdline::parse("a=1 b=2 c").unwrap();
        assert!(cl.remove("b"));
        assert!(!cl.contains("b"));
        assert!(!cl.remove("b"));
        assert_eq!(cl.len(), 2);
    }

    #[test]
    fn replace_overwrites_not_appends() {
        let cl = Cmdline::parse("console=tty0")
            .unwrap()
            .replace("console", "ttyS0");
        assert_eq!(cl.all_values("console"), vec!["ttyS0"]);
        // Replace inserts when absent.
        let cl2 = Cmdline::new().replace("pti", "on");
        assert_eq!(cl2.get_first("pti"), Some("on"));
    }

    #[test]
    fn apply_args_set_flag_and_delete() {
        let cl = Cmdline::parse("pti=off quiet console=tty0")
            .unwrap()
            .apply_args(&["pti=on", "slab_nomerge", "-quiet"])
            .unwrap();
        assert_eq!(cl.get_first("pti"), Some("on"));
        assert!(cl.has_flag("slab_nomerge"));
        assert!(!cl.contains("quiet"));
        assert!(cl.contains("console"));
    }

    #[test]
    fn apply_args_rejects_bad_tokens() {
        assert!(matches!(
            Cmdline::new().apply_args(&["=bad"]),
            Err(KernelParamError::Parse(_))
        ));
        assert!(matches!(
            Cmdline::new().apply_args(&["-"]),
            Err(KernelParamError::Parse(_))
        ));
    }

    #[test]
    fn apply_args_does_not_duplicate_flag() {
        let cl = Cmdline::parse("quiet")
            .unwrap()
            .apply_args(&["quiet"])
            .unwrap();
        assert_eq!(cl.len(), 1);
    }

    #[test]
    fn to_args_roundtrips() {
        let cl = Cmdline::parse("a=1 flag console=ttyS0,115200").unwrap();
        let args = cl.to_args();
        assert!(args.contains(&"a=1".to_string()));
        assert!(args.contains(&"flag".to_string()));
        assert!(args.contains(&"console=ttyS0,115200".to_string()));
    }

    #[test]
    fn security_posture_helpers() {
        let insecure = Cmdline::parse("mitigations=off").unwrap();
        assert!(insecure.mitigations_disabled());
        let secure = Cmdline::parse(
            "module.sig_enforce=1 lockdown=confidentiality talos.platform=metal talos.config=https://x/y",
        )
        .unwrap();
        assert!(!secure.mitigations_disabled());
        assert!(secure.module_sig_enforced());
        assert_eq!(secure.lockdown_level(), Some("confidentiality"));
        assert_eq!(secure.os_platform_domain(), Some("metal"));
        assert_eq!(secure.talos_config_url(), Some("https://x/y"));
    }
}
