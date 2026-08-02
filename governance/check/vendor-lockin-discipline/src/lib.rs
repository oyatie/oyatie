//! ADR-0173 — Vendor lock-in discipline validator.
//!
//! Validates `registry/vendor-lockin-phaseout/index.json` against the tiered
//! vendor-classification doctrine:
//!
//! - Tier I — OWNED; must declare license + steward.
//! - Tier II — VENDOR-SEAMED; must declare replacement_path,
//!   replacement_readiness_gate, seam_adapter_trait, >=1 seam_adapter_impls,
//!   and phase_out_target_date_or_signal.
//! - Tier III — FORBIDDEN; must declare a refusal rationale + replacement path.
//!
//! Returns a structured report with counts per tier and the first failure
//! (if any) — fail-closed by default.
//
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// Coarse-grain tier classification per ADR-0173.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorTier {
    /// OWNED long-term.
    TierI,
    /// OWNED with asterisk — license drift / bus-factor concern.
    TierIAsterisk,
    /// VENDOR-SEAMED temporary.
    TierII,
    /// VENDOR-SEAMED pre-classified (not yet adopted, doctrine pre-applied).
    TierIIPreClassified,
    /// FORBIDDEN.
    TierIII,
}

impl VendorTier {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "I" => Some(Self::TierI),
            "I-asterisk" => Some(Self::TierIAsterisk),
            "II" => Some(Self::TierII),
            "II-pre-classified" => Some(Self::TierIIPreClassified),
            "III" => Some(Self::TierIII),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TierI => "I",
            Self::TierIAsterisk => "I-asterisk",
            Self::TierII => "II",
            Self::TierIIPreClassified => "II-pre-classified",
            Self::TierIII => "III",
        }
    }

    pub fn requires_replacement_path(&self) -> bool {
        matches!(
            self,
            Self::TierIAsterisk | Self::TierII | Self::TierIIPreClassified | Self::TierIII
        )
    }

    /// Only ADOPTED Tier II entries need actual adapter impls on disk.
    /// Pre-classified Tier II entries are placeholders (not adopted) and
    /// satisfy the doctrine by declaring the trait + impls slot — they
    /// MUST declare the trait but the impls list may be empty until
    /// adoption.
    pub fn requires_live_seam(&self) -> bool {
        matches!(self, Self::TierII | Self::TierIAsterisk)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VendorEntry {
    pub name: String,
    pub tier: VendorTier,
    pub license: Option<String>,
    pub steward: Option<String>,
    pub adoption_rationale: String,
    pub replacement_path: Option<String>,
    pub replacement_readiness_gate: Option<String>,
    pub seam_adapter_trait: Option<String>,
    pub seam_adapter_impls: Vec<String>,
    pub phase_out_target_date_or_signal: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VendorLockinReport {
    pub entries_seen: usize,
    pub tier_i_count: usize,
    pub tier_i_asterisk_count: usize,
    pub tier_ii_count: usize,
    pub tier_ii_pre_count: usize,
    pub tier_iii_count: usize,
    pub seam_traits_unique: usize,
    pub seam_impls_total: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VendorLockinError {
    EmptyRegistry,
    DuplicateName(String),
    MissingField {
        entry_name: String,
        field: &'static str,
    },
    InvalidTier {
        entry_name: String,
        value: String,
    },
    TierIIMissingSeamTrait(String),
    TierIIMissingSeamImpl(String),
    TierIIIMissingRefusalRationale(String),
    TierIMissingLicense(String),
    Malformed(String),
}

impl fmt::Display for VendorLockinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRegistry => write!(f, "vendor-lockin-phaseout registry is empty"),
            Self::DuplicateName(name) => write!(f, "duplicate vendor entry: {name}"),
            Self::MissingField { entry_name, field } => {
                write!(f, "vendor {entry_name}: missing required field `{field}`")
            }
            Self::InvalidTier { entry_name, value } => {
                write!(f, "vendor {entry_name}: invalid tier `{value}`")
            }
            Self::TierIIMissingSeamTrait(name) => {
                write!(f, "vendor {name}: Tier II must declare seam_adapter_trait")
            }
            Self::TierIIMissingSeamImpl(name) => write!(
                f,
                "vendor {name}: Tier II must declare at least one seam_adapter_impls member"
            ),
            Self::TierIIIMissingRefusalRationale(name) => write!(
                f,
                "vendor {name}: Tier III must declare an adoption_rationale starting with REFUSED"
            ),
            Self::TierIMissingLicense(name) => {
                write!(f, "vendor {name}: Tier I must declare license + steward")
            }
            Self::Malformed(msg) => write!(f, "vendor-lockin registry malformed: {msg}"),
        }
    }
}

/// Validate the parsed registry against ADR-0173 doctrine.
pub fn validate_registry(entries: &[VendorEntry]) -> Result<VendorLockinReport, VendorLockinError> {
    if entries.is_empty() {
        return Err(VendorLockinError::EmptyRegistry);
    }

    let mut seen_names: BTreeSet<&str> = BTreeSet::new();
    let mut seam_traits: BTreeSet<&str> = BTreeSet::new();
    let mut report = VendorLockinReport {
        entries_seen: entries.len(),
        ..VendorLockinReport::default()
    };

    for entry in entries {
        if entry.name.trim().is_empty() {
            return Err(VendorLockinError::MissingField {
                entry_name: "<unnamed>".to_owned(),
                field: "name",
            });
        }
        if !seen_names.insert(entry.name.as_str()) {
            return Err(VendorLockinError::DuplicateName(entry.name.clone()));
        }
        if entry.adoption_rationale.trim().is_empty() {
            return Err(VendorLockinError::MissingField {
                entry_name: entry.name.clone(),
                field: "adoption_rationale",
            });
        }

        match entry.tier {
            VendorTier::TierI | VendorTier::TierIAsterisk => {
                report.tier_i_count += matches!(entry.tier, VendorTier::TierI) as usize;
                report.tier_i_asterisk_count +=
                    matches!(entry.tier, VendorTier::TierIAsterisk) as usize;
                if entry.license.as_deref().unwrap_or("").trim().is_empty()
                    || entry.steward.as_deref().unwrap_or("").trim().is_empty()
                {
                    return Err(VendorLockinError::TierIMissingLicense(entry.name.clone()));
                }
                if matches!(entry.tier, VendorTier::TierIAsterisk) {
                    require_phase_out_fields(entry)?;
                }
            }
            VendorTier::TierII => {
                report.tier_ii_count += 1;
                require_phase_out_fields(entry)?;
                let seam_trait = entry.seam_adapter_trait.as_deref().unwrap_or("").trim();
                if seam_trait.is_empty() {
                    return Err(VendorLockinError::TierIIMissingSeamTrait(
                        entry.name.clone(),
                    ));
                }
                seam_traits.insert(seam_trait);
                if entry.seam_adapter_impls.is_empty() {
                    return Err(VendorLockinError::TierIIMissingSeamImpl(entry.name.clone()));
                }
                report.seam_impls_total += entry.seam_adapter_impls.len();
            }
            VendorTier::TierIIPreClassified => {
                report.tier_ii_pre_count += 1;
                require_phase_out_fields(entry)?;
                // Pre-classified: trait may be a planning placeholder; impls
                // list may be empty until adoption. We still require the
                // trait declaration so adoption cannot skip the seam step.
                if entry
                    .seam_adapter_trait
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                {
                    return Err(VendorLockinError::TierIIMissingSeamTrait(
                        entry.name.clone(),
                    ));
                }
            }
            VendorTier::TierIII => {
                report.tier_iii_count += 1;
                if !entry.adoption_rationale.to_uppercase().contains("REFUSED") {
                    return Err(VendorLockinError::TierIIIMissingRefusalRationale(
                        entry.name.clone(),
                    ));
                }
                if entry
                    .replacement_path
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                {
                    return Err(VendorLockinError::MissingField {
                        entry_name: entry.name.clone(),
                        field: "replacement_path",
                    });
                }
            }
        }
    }

    report.seam_traits_unique = seam_traits.len();
    Ok(report)
}

fn require_phase_out_fields(entry: &VendorEntry) -> Result<(), VendorLockinError> {
    if entry
        .replacement_path
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(VendorLockinError::MissingField {
            entry_name: entry.name.clone(),
            field: "replacement_path",
        });
    }
    if entry
        .replacement_readiness_gate
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(VendorLockinError::MissingField {
            entry_name: entry.name.clone(),
            field: "replacement_readiness_gate",
        });
    }
    if entry
        .phase_out_target_date_or_signal
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err(VendorLockinError::MissingField {
            entry_name: entry.name.clone(),
            field: "phase_out_target_date_or_signal",
        });
    }
    Ok(())
}

/// Minimal JSON parser tailored to the vendor-lockin-phaseout registry.
/// Hand-rolled to keep the validator zero-dep (mirrors the pattern used by
/// other check-family crates such as `oya-check-license-policy`).
pub fn parse_registry_json(source: &str) -> Result<Vec<VendorEntry>, VendorLockinError> {
    let mut parser = JsonParser::new(source);
    parser.skip_ws();
    parser.expect('{')?;
    let mut entries: Option<Vec<VendorEntry>> = None;
    loop {
        parser.skip_ws();
        if parser.peek() == Some('}') {
            parser.advance();
            break;
        }
        let key = parser.parse_string()?;
        parser.skip_ws();
        parser.expect(':')?;
        parser.skip_ws();
        if key == "entries" {
            entries = Some(parser.parse_entries_array()?);
        } else {
            parser.skip_value()?;
        }
        parser.skip_ws();
        if parser.peek() == Some(',') {
            parser.advance();
            continue;
        }
        if parser.peek() == Some('}') {
            parser.advance();
            break;
        }
        return Err(VendorLockinError::Malformed(format!(
            "expected ',' or '}}' at offset {} after key `{key}`",
            parser.position
        )));
    }
    entries
        .ok_or_else(|| VendorLockinError::Malformed("missing top-level `entries` array".to_owned()))
}

struct JsonParser<'a> {
    source: &'a [u8],
    position: usize,
}

impl<'a> JsonParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied().map(|b| b as char)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.source.get(self.position).copied() {
            if (b as char).is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), VendorLockinError> {
        match self.peek() {
            Some(c) if c == expected => {
                self.advance();
                Ok(())
            }
            Some(c) => Err(VendorLockinError::Malformed(format!(
                "expected '{expected}' at offset {} but found '{c}'",
                self.position
            ))),
            None => Err(VendorLockinError::Malformed(format!(
                "expected '{expected}' at offset {} but reached EOF",
                self.position
            ))),
        }
    }

    fn parse_string(&mut self) -> Result<String, VendorLockinError> {
        self.expect('"')?;
        let mut buffer = String::new();
        loop {
            match self.peek() {
                None => {
                    return Err(VendorLockinError::Malformed(
                        "unterminated string literal".to_owned(),
                    ));
                }
                Some('"') => {
                    self.advance();
                    return Ok(buffer);
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('"') => {
                            buffer.push('"');
                            self.advance();
                        }
                        Some('\\') => {
                            buffer.push('\\');
                            self.advance();
                        }
                        Some('/') => {
                            buffer.push('/');
                            self.advance();
                        }
                        Some('n') => {
                            buffer.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            buffer.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            buffer.push('\r');
                            self.advance();
                        }
                        Some('u') => {
                            self.advance();
                            let mut code = 0u32;
                            for _ in 0..4 {
                                let c = self.peek().ok_or_else(|| {
                                    VendorLockinError::Malformed(
                                        "truncated unicode escape".to_owned(),
                                    )
                                })?;
                                code = (code << 4)
                                    | c.to_digit(16).ok_or_else(|| {
                                        VendorLockinError::Malformed(format!(
                                            "invalid unicode hex `{c}`"
                                        ))
                                    })?;
                                self.advance();
                            }
                            if let Some(ch) = char::from_u32(code) {
                                buffer.push(ch);
                            } else {
                                buffer.push('?');
                            }
                        }
                        Some(other) => {
                            return Err(VendorLockinError::Malformed(format!(
                                "unsupported escape `\\{other}`"
                            )));
                        }
                        None => {
                            return Err(VendorLockinError::Malformed(
                                "EOF inside string escape".to_owned(),
                            ));
                        }
                    }
                }
                Some(c) => {
                    buffer.push(c);
                    self.advance();
                }
            }
        }
    }

    fn parse_entries_array(&mut self) -> Result<Vec<VendorEntry>, VendorLockinError> {
        self.expect('[')?;
        let mut entries = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(']') {
                self.advance();
                return Ok(entries);
            }
            entries.push(self.parse_entry_object()?);
            self.skip_ws();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some(']') {
                self.advance();
                return Ok(entries);
            }
            return Err(VendorLockinError::Malformed(
                "expected ',' or ']' in entries array".to_owned(),
            ));
        }
    }

    fn parse_entry_object(&mut self) -> Result<VendorEntry, VendorLockinError> {
        self.expect('{')?;
        let mut name = String::new();
        let mut tier_raw: Option<String> = None;
        let mut license: Option<String> = None;
        let mut steward: Option<String> = None;
        let mut adoption_rationale = String::new();
        let mut replacement_path: Option<String> = None;
        let mut replacement_readiness_gate: Option<String> = None;
        let mut seam_adapter_trait: Option<String> = None;
        let mut seam_adapter_impls = Vec::new();
        let mut phase_out_target_date_or_signal: Option<String> = None;

        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.advance();
                break;
            }
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(':')?;
            self.skip_ws();
            match key.as_str() {
                "name" => name = self.parse_string()?,
                "tier" => tier_raw = Some(self.parse_string()?),
                "license" => license = self.parse_nullable_string()?,
                "steward" => steward = self.parse_nullable_string()?,
                "adoption_rationale" => adoption_rationale = self.parse_string()?,
                "replacement_path" => replacement_path = self.parse_nullable_string()?,
                "replacement_readiness_gate" => {
                    replacement_readiness_gate = self.parse_nullable_string()?
                }
                "seam_adapter_trait" => seam_adapter_trait = self.parse_nullable_string()?,
                "seam_adapter_impls" => seam_adapter_impls = self.parse_string_array()?,
                "phase_out_target_date_or_signal" => {
                    phase_out_target_date_or_signal = self.parse_nullable_string()?
                }
                _ => self.skip_value()?,
            }
            self.skip_ws();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some('}') {
                self.advance();
                break;
            }
            return Err(VendorLockinError::Malformed(format!(
                "expected ',' or '}}' inside vendor entry (last key `{key}`)"
            )));
        }

        let tier_raw = tier_raw.ok_or_else(|| VendorLockinError::MissingField {
            entry_name: name.clone(),
            field: "tier",
        })?;
        let tier = VendorTier::parse(&tier_raw).ok_or_else(|| VendorLockinError::InvalidTier {
            entry_name: name.clone(),
            value: tier_raw,
        })?;

        Ok(VendorEntry {
            name,
            tier,
            license,
            steward,
            adoption_rationale,
            replacement_path,
            replacement_readiness_gate,
            seam_adapter_trait,
            seam_adapter_impls,
            phase_out_target_date_or_signal,
        })
    }

    fn parse_nullable_string(&mut self) -> Result<Option<String>, VendorLockinError> {
        self.skip_ws();
        if self.peek() == Some('n') {
            // null literal
            for expected in ['n', 'u', 'l', 'l'] {
                if self.peek() != Some(expected) {
                    return Err(VendorLockinError::Malformed(
                        "expected null literal".to_owned(),
                    ));
                }
                self.advance();
            }
            return Ok(None);
        }
        Ok(Some(self.parse_string()?))
    }

    fn parse_string_array(&mut self) -> Result<Vec<String>, VendorLockinError> {
        self.expect('[')?;
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(']') {
                self.advance();
                return Ok(items);
            }
            items.push(self.parse_string()?);
            self.skip_ws();
            if self.peek() == Some(',') {
                self.advance();
                continue;
            }
            if self.peek() == Some(']') {
                self.advance();
                return Ok(items);
            }
            return Err(VendorLockinError::Malformed(
                "expected ',' or ']' in string array".to_owned(),
            ));
        }
    }

    fn skip_value(&mut self) -> Result<(), VendorLockinError> {
        self.skip_ws();
        match self.peek() {
            Some('"') => {
                let _ = self.parse_string()?;
                Ok(())
            }
            Some('{') => {
                self.advance();
                let mut depth = 1;
                while depth > 0 {
                    self.skip_ws();
                    match self.peek() {
                        Some('}') => {
                            self.advance();
                            depth -= 1;
                        }
                        Some('{') => {
                            self.advance();
                            depth += 1;
                        }
                        Some('"') => {
                            let _ = self.parse_string()?;
                        }
                        Some(_) => {
                            self.advance();
                        }
                        None => {
                            return Err(VendorLockinError::Malformed(
                                "EOF inside object skip".to_owned(),
                            ));
                        }
                    }
                }
                Ok(())
            }
            Some('[') => {
                self.advance();
                let mut depth = 1;
                while depth > 0 {
                    self.skip_ws();
                    match self.peek() {
                        Some(']') => {
                            self.advance();
                            depth -= 1;
                        }
                        Some('[') => {
                            self.advance();
                            depth += 1;
                        }
                        Some('"') => {
                            let _ = self.parse_string()?;
                        }
                        Some(_) => {
                            self.advance();
                        }
                        None => {
                            return Err(VendorLockinError::Malformed(
                                "EOF inside array skip".to_owned(),
                            ));
                        }
                    }
                }
                Ok(())
            }
            Some(c) if c == 't' || c == 'f' || c == 'n' => {
                while let Some(b) = self.peek() {
                    if b.is_ascii_alphabetic() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(())
            }
            Some(c) if c.is_ascii_digit() || c == '-' => {
                while let Some(b) = self.peek() {
                    if b.is_ascii_digit()
                        || b == '.'
                        || b == '-'
                        || b == '+'
                        || b == 'e'
                        || b == 'E'
                    {
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(())
            }
            Some(c) => Err(VendorLockinError::Malformed(format!(
                "unexpected character `{c}` at offset {} during value skip",
                self.position
            ))),
            None => Err(VendorLockinError::Malformed(
                "EOF during value skip".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_tier_i(name: &str) -> VendorEntry {
        VendorEntry {
            name: name.to_owned(),
            tier: VendorTier::TierI,
            license: Some("Apache-2.0".to_owned()),
            steward: Some("Foundation".to_owned()),
            adoption_rationale: "OSS canonical substrate".to_owned(),
            replacement_path: None,
            replacement_readiness_gate: None,
            seam_adapter_trait: None,
            seam_adapter_impls: Vec::new(),
            phase_out_target_date_or_signal: None,
        }
    }

    fn entry_tier_ii(name: &str) -> VendorEntry {
        VendorEntry {
            name: name.to_owned(),
            tier: VendorTier::TierII,
            license: Some("proprietary".to_owned()),
            steward: Some("Vendor".to_owned()),
            adoption_rationale: "frontier-class capability".to_owned(),
            replacement_path: Some("in-house substrate".to_owned()),
            replacement_readiness_gate: Some("parity demonstrated".to_owned()),
            seam_adapter_trait: Some("crates/oya-foo-vendor-kernel".to_owned()),
            seam_adapter_impls: vec!["crates/oya-foo-vendor-adapter".to_owned()],
            phase_out_target_date_or_signal: Some("signal: parity".to_owned()),
        }
    }

    fn entry_tier_iii(name: &str) -> VendorEntry {
        VendorEntry {
            name: name.to_owned(),
            tier: VendorTier::TierIII,
            license: Some("proprietary".to_owned()),
            steward: Some("Vendor".to_owned()),
            adoption_rationale: "REFUSED — OSS equivalent exists".to_owned(),
            replacement_path: Some("OSS substrate".to_owned()),
            replacement_readiness_gate: Some("N/A".to_owned()),
            seam_adapter_trait: None,
            seam_adapter_impls: Vec::new(),
            phase_out_target_date_or_signal: Some("REFUSED".to_owned()),
        }
    }

    #[test]
    fn validate_rejects_empty_registry() {
        assert_eq!(
            validate_registry(&[]),
            Err(VendorLockinError::EmptyRegistry)
        );
    }

    #[test]
    fn validate_accepts_well_formed_mixed_tiers() {
        let entries = vec![
            entry_tier_i("postgres"),
            entry_tier_ii("anthropic-api"),
            entry_tier_iii("aws-lambda"),
        ];
        let report = validate_registry(&entries).unwrap();
        assert_eq!(report.entries_seen, 3);
        assert_eq!(report.tier_i_count, 1);
        assert_eq!(report.tier_ii_count, 1);
        assert_eq!(report.tier_iii_count, 1);
        assert_eq!(report.seam_traits_unique, 1);
        assert_eq!(report.seam_impls_total, 1);
    }

    #[test]
    fn validate_rejects_duplicate_names() {
        let entries = vec![entry_tier_i("postgres"), entry_tier_i("postgres")];
        assert_eq!(
            validate_registry(&entries),
            Err(VendorLockinError::DuplicateName("postgres".to_owned()))
        );
    }

    #[test]
    fn validate_rejects_tier_ii_without_seam_trait() {
        let mut bad = entry_tier_ii("anthropic-api");
        bad.seam_adapter_trait = None;
        assert_eq!(
            validate_registry(&[bad]),
            Err(VendorLockinError::TierIIMissingSeamTrait(
                "anthropic-api".to_owned()
            ))
        );
    }

    #[test]
    fn validate_rejects_tier_ii_without_seam_impl() {
        let mut bad = entry_tier_ii("anthropic-api");
        bad.seam_adapter_impls.clear();
        assert_eq!(
            validate_registry(&[bad]),
            Err(VendorLockinError::TierIIMissingSeamImpl(
                "anthropic-api".to_owned()
            ))
        );
    }

    #[test]
    fn validate_rejects_tier_ii_missing_replacement_path() {
        let mut bad = entry_tier_ii("anthropic-api");
        bad.replacement_path = None;
        match validate_registry(&[bad]) {
            Err(VendorLockinError::MissingField { field, .. }) => {
                assert_eq!(field, "replacement_path");
            }
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_tier_ii_missing_readiness_gate() {
        let mut bad = entry_tier_ii("anthropic-api");
        bad.replacement_readiness_gate = None;
        match validate_registry(&[bad]) {
            Err(VendorLockinError::MissingField { field, .. }) => {
                assert_eq!(field, "replacement_readiness_gate");
            }
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_tier_ii_missing_phase_out() {
        let mut bad = entry_tier_ii("anthropic-api");
        bad.phase_out_target_date_or_signal = None;
        match validate_registry(&[bad]) {
            Err(VendorLockinError::MissingField { field, .. }) => {
                assert_eq!(field, "phase_out_target_date_or_signal");
            }
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_tier_iii_without_refused_rationale() {
        let mut bad = entry_tier_iii("aws-lambda");
        bad.adoption_rationale = "looked easy".to_owned();
        assert_eq!(
            validate_registry(&[bad]),
            Err(VendorLockinError::TierIIIMissingRefusalRationale(
                "aws-lambda".to_owned()
            ))
        );
    }

    #[test]
    fn validate_rejects_tier_iii_without_replacement_path() {
        let mut bad = entry_tier_iii("aws-lambda");
        bad.replacement_path = None;
        match validate_registry(&[bad]) {
            Err(VendorLockinError::MissingField { field, .. }) => {
                assert_eq!(field, "replacement_path");
            }
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_tier_i_missing_license() {
        let mut bad = entry_tier_i("postgres");
        bad.license = None;
        assert_eq!(
            validate_registry(&[bad]),
            Err(VendorLockinError::TierIMissingLicense(
                "postgres".to_owned()
            ))
        );
    }

    #[test]
    fn validate_rejects_tier_i_missing_steward() {
        let mut bad = entry_tier_i("postgres");
        bad.steward = None;
        assert_eq!(
            validate_registry(&[bad]),
            Err(VendorLockinError::TierIMissingLicense(
                "postgres".to_owned()
            ))
        );
    }

    #[test]
    fn validate_rejects_blank_name() {
        let mut bad = entry_tier_i("");
        bad.name = "".to_owned();
        assert!(matches!(
            validate_registry(&[bad]),
            Err(VendorLockinError::MissingField { field, .. }) if field == "name"
        ));
    }

    #[test]
    fn tier_parse_round_trip() {
        for value in ["I", "I-asterisk", "II", "II-pre-classified", "III"] {
            let parsed = VendorTier::parse(value).unwrap();
            assert_eq!(parsed.as_str(), value);
        }
        assert!(VendorTier::parse("IV").is_none());
        assert!(VendorTier::parse("").is_none());
    }

    #[test]
    fn parse_registry_json_minimal() {
        let json = r#"{
          "entries": [
            {
              "name": "postgres",
              "tier": "I",
              "license": "PostgreSQL License",
              "steward": "PGDG",
              "adoption_rationale": "canonical OLTP",
              "replacement_path": null,
              "replacement_readiness_gate": null,
              "seam_adapter_trait": null,
              "seam_adapter_impls": [],
              "phase_out_target_date_or_signal": null
            }
          ]
        }"#;
        let parsed = parse_registry_json(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "postgres");
        assert_eq!(parsed[0].tier, VendorTier::TierI);
        validate_registry(&parsed).unwrap();
    }

    #[test]
    fn parse_registry_json_tier_ii_full() {
        let json = r#"{
          "schema_version": "1.0.0",
          "entries": [
            {
              "name": "anthropic-api",
              "tier": "II",
              "license": "proprietary",
              "steward": "Anthropic",
              "adoption_rationale": "frontier LLM",
              "replacement_path": "in-house substrate",
              "replacement_readiness_gate": "parity",
              "seam_adapter_trait": "crates/oya-intelligence-adapter-anthropic-api-kernel",
              "seam_adapter_impls": [
                "crates/oya-intelligence-adapter-anthropic-api-adapter",
                "crates/oya-intelligence-adapter-openai-api-adapter"
              ],
              "phase_out_target_date_or_signal": "signal"
            }
          ]
        }"#;
        let parsed = parse_registry_json(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].seam_adapter_impls.len(), 2);
        let report = validate_registry(&parsed).unwrap();
        assert_eq!(report.tier_ii_count, 1);
        assert_eq!(report.seam_impls_total, 2);
    }

    #[test]
    fn parse_registry_json_rejects_missing_entries() {
        let json = r#"{"adr": "ADR-0173"}"#;
        assert!(matches!(
            parse_registry_json(json),
            Err(VendorLockinError::Malformed(_))
        ));
    }

    #[test]
    fn parse_registry_json_rejects_invalid_tier() {
        let json = r#"{
          "entries": [
            {
              "name": "bogus",
              "tier": "IV",
              "adoption_rationale": "x"
            }
          ]
        }"#;
        assert!(matches!(
            parse_registry_json(json),
            Err(VendorLockinError::InvalidTier { .. })
        ));
    }

    #[test]
    fn parse_registry_handles_pre_classified_with_empty_impls() {
        let json = r#"{
          "entries": [
            {
              "name": "cloudflare-cdn",
              "tier": "II-pre-classified",
              "license": "proprietary",
              "steward": "Cloudflare",
              "adoption_rationale": "NOT ADOPTED. Pre-classified.",
              "replacement_path": "self-hosted Envoy",
              "replacement_readiness_gate": "edge POPs",
              "seam_adapter_trait": "crates/oya-edge-cdn-kernel",
              "seam_adapter_impls": [],
              "phase_out_target_date_or_signal": "NOT ADOPTED"
            }
          ]
        }"#;
        let parsed = parse_registry_json(json).unwrap();
        let report = validate_registry(&parsed).unwrap();
        assert_eq!(report.tier_ii_pre_count, 1);
    }

    #[test]
    fn parse_skip_value_handles_numbers_and_bools_and_null() {
        let json = r#"{
          "schema_version": "1.0.0",
          "number_field": 42,
          "negative": -3.14e10,
          "bool_field": true,
          "null_field": null,
          "nested": {"a": [1, 2, 3], "b": "x"},
          "entries": [
            {
              "name": "noop",
              "tier": "I",
              "license": "Apache-2.0",
              "steward": "Foundation",
              "adoption_rationale": "ok"
            }
          ]
        }"#;
        let parsed = parse_registry_json(json).unwrap();
        assert_eq!(parsed.len(), 1);
    }
}
