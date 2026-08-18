//! JSON codec for META key/value records.
//!
//! Besides the binary ADV on-disk format, Talos exposes META values in a
//! transport-friendly JSON shape used by `talosctl meta` and by the
//! `meta:` machine-config document (see `pkg/machinery/meta`). Each record is
//! serialized as:
//!
//! ```json
//! {"key": 13, "value": "tok-abc"}
//! ```
//!
//! where `key` is the numeric ADV tag and `value` is the (string) payload. A
//! full document is a JSON array of such objects.
//!
//! This module implements a small, dependency-free JSON encoder/decoder
//! covering exactly the subset needed for these records: objects with `key`
//! (integer) and `value` (string) fields, and arrays thereof. The string codec
//! handles the standard JSON escapes plus `\uXXXX` for control characters.

use crate::adv::Adv;
use crate::key::MetaKey;
use crate::value::MetaValue;
use os_kernel::{Error, Result};
use std::fmt::Write as _;

/// A single META record in its JSON transport form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRecord {
    /// The ADV tag of the record.
    pub key: MetaKey,
    /// The record value, interpreted as a UTF-8 string for JSON transport.
    pub value: String,
}

impl JsonRecord {
    /// Builds a record from a [`MetaKey`] and string value.
    pub fn new(key: MetaKey, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }

    /// Serializes a single record to a compact JSON object.
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\"key\":");
        s.push_str(&self.key.tag().to_string());
        s.push_str(",\"value\":");
        encode_json_string(&self.value, &mut s);
        s.push('}');
        s
    }

    /// Parses a single JSON object into a record.
    pub fn from_json(input: &str) -> Result<Self> {
        let mut p = Parser::new(input);
        let rec = p.parse_record()?;
        p.skip_ws();
        if !p.at_end() {
            return Err(Error::parse("trailing data after JSON record"));
        }
        Ok(rec)
    }
}

/// Serializes a slice of records to a JSON array.
pub fn encode_records(records: &[JsonRecord]) -> String {
    let mut s = String::from("[");
    for (i, r) in records.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&r.to_json());
    }
    s.push(']');
    s
}

/// Parses a JSON array of records.
pub fn decode_records(input: &str) -> Result<Vec<JsonRecord>> {
    let mut p = Parser::new(input);
    let records = p.parse_array()?;
    p.skip_ws();
    if !p.at_end() {
        return Err(Error::parse("trailing data after JSON array"));
    }
    Ok(records)
}

/// Serializes an [`Adv`] document to a JSON array of records.
///
/// Every value must be valid UTF-8 (the JSON form is text-only); a binary
/// value causes an error.
pub fn adv_to_json(adv: &Adv) -> Result<String> {
    let mut records = Vec::new();
    for (key, value) in adv.iter() {
        records.push(JsonRecord::new(*key, value.as_str()?.to_string()));
    }
    Ok(encode_records(&records))
}

/// Builds an [`Adv`] document from a JSON array of records.
///
/// Duplicate keys are rejected, mirroring the binary decoder.
pub fn adv_from_json(input: &str) -> Result<Adv> {
    let records = decode_records(input)?;
    let mut adv = Adv::new();
    for r in records {
        if adv.set(r.key, MetaValue::from_str(&r.value)?).is_some() {
            return Err(Error::parse("JSON META document contains duplicate key"));
        }
    }
    Ok(adv)
}

// --- minimal JSON string encoding -----------------------------------------

fn encode_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

// --- minimal JSON parser ---------------------------------------------------

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, b: u8) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(Error::parse(format!(
                "expected '{}' at byte {}",
                b as char, self.pos
            )))
        }
    }

    fn parse_array(&mut self) -> Result<Vec<JsonRecord>> {
        self.expect(b'[')?;
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(out);
        }
        loop {
            out.push(self.parse_record()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(Error::parse("expected ',' or ']' in JSON array")),
            }
        }
        Ok(out)
    }

    fn parse_record(&mut self) -> Result<JsonRecord> {
        self.expect(b'{')?;
        let mut key: Option<MetaKey> = None;
        let mut value: Option<String> = None;
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Err(Error::parse("JSON record missing 'key'/'value' fields"));
        }
        loop {
            self.skip_ws();
            let field = self.parse_string()?;
            self.expect(b':')?;
            self.skip_ws();
            match field.as_str() {
                "key" => {
                    let tag = self.parse_u8()?;
                    key = Some(MetaKey::from_tag(tag)?);
                }
                "value" => {
                    value = Some(self.parse_string()?);
                }
                other => return Err(Error::parse(format!("unexpected JSON field {other:?}"))),
            }
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(Error::parse("expected ',' or '}' in JSON record")),
            }
        }
        match (key, value) {
            (Some(key), Some(value)) => Ok(JsonRecord { key, value }),
            _ => Err(Error::parse("JSON record missing 'key' or 'value'")),
        }
    }

    fn parse_u8(&mut self) -> Result<u8> {
        self.skip_ws();
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(Error::parse("expected integer for 'key'"));
        }
        let s = core::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| Error::parse("invalid integer bytes"))?;
        s.parse::<u8>()
            .map_err(|_| Error::parse(format!("'key' value {s} is out of range for a u8 tag")))
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let b = self
                .peek()
                .ok_or_else(|| Error::parse("unterminated JSON string"))?;
            self.pos += 1;
            match b {
                b'"' => break,
                b'\\' => {
                    let esc = self
                        .peek()
                        .ok_or_else(|| Error::parse("dangling escape in JSON string"))?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'b' => out.push('\u{08}'),
                        b'f' => out.push('\u{0C}'),
                        b'u' => {
                            let cp = self.parse_hex4()?;
                            out.push(
                                char::from_u32(u32::from(cp))
                                    .ok_or_else(|| Error::parse("invalid \\u code point"))?,
                            );
                        }
                        other => {
                            return Err(Error::parse(format!(
                                "invalid escape '\\{}' in JSON string",
                                other as char
                            )));
                        }
                    }
                }
                // Continuation / multi-byte UTF-8: collect the full code point.
                _ if b < 0x80 => out.push(b as char),
                _ => {
                    // Decode a UTF-8 multibyte sequence starting at pos-1.
                    let start = self.pos - 1;
                    let extra = if b >= 0xF0 {
                        3
                    } else if b >= 0xE0 {
                        2
                    } else {
                        1
                    };
                    let end = start + 1 + extra;
                    if end > self.bytes.len() {
                        return Err(Error::parse("truncated UTF-8 in JSON string"));
                    }
                    let s = core::str::from_utf8(&self.bytes[start..end])
                        .map_err(|_| Error::parse("invalid UTF-8 in JSON string"))?;
                    out.push_str(s);
                    self.pos = end;
                }
            }
        }
        Ok(out)
    }

    fn parse_hex4(&mut self) -> Result<u16> {
        if self.pos + 4 > self.bytes.len() {
            return Err(Error::parse("truncated \\u escape"));
        }
        let s = core::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
            .map_err(|_| Error::parse("invalid \\u bytes"))?;
        let v =
            u16::from_str_radix(s, 16).map_err(|_| Error::parse("invalid hex in \\u escape"))?;
        self.pos += 4;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_record_round_trips() {
        let r = JsonRecord::new(MetaKey::UniqueMachineToken, "tok-abc");
        let json = r.to_json();
        assert_eq!(json, "{\"key\":13,\"value\":\"tok-abc\"}");
        assert_eq!(JsonRecord::from_json(&json).unwrap(), r);
    }

    #[test]
    fn array_round_trips() {
        let recs = vec![
            JsonRecord::new(MetaKey::Upgrade, "v1.6.0"),
            JsonRecord::new(MetaKey::StagedUpgradeImageRef, "installer:v1.7.0"),
        ];
        let json = encode_records(&recs);
        assert_eq!(decode_records(&json).unwrap(), recs);
    }

    #[test]
    fn empty_array_round_trips() {
        assert_eq!(encode_records(&[]), "[]");
        assert!(decode_records("[]").unwrap().is_empty());
        assert!(decode_records("   [ ]  ").unwrap().is_empty());
    }

    #[test]
    fn whitespace_is_tolerated() {
        let json = " { \"key\" : 6 , \"value\" : \"x\" } ";
        let r = JsonRecord::from_json(json.trim()).unwrap();
        assert_eq!(r.key, MetaKey::Upgrade);
        assert_eq!(r.value, "x");
    }

    #[test]
    fn value_field_order_independent() {
        let r = JsonRecord::from_json("{\"value\":\"hi\",\"key\":6}").unwrap();
        assert_eq!(r.key, MetaKey::Upgrade);
        assert_eq!(r.value, "hi");
    }

    #[test]
    fn escapes_round_trip() {
        let r = JsonRecord::new(MetaKey::Custom(0x40), "a\"b\\c\nd\te\r\u{08}\u{0C}");
        let json = r.to_json();
        let back = JsonRecord::from_json(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn control_char_uses_u_escape() {
        let r = JsonRecord::new(MetaKey::Custom(0x40), "\u{01}");
        assert!(r.to_json().contains("\\u0001"));
        assert_eq!(JsonRecord::from_json(&r.to_json()).unwrap(), r);
    }

    #[test]
    fn unicode_value_round_trips() {
        let r = JsonRecord::new(MetaKey::Custom(0x40), "héllo-世界-🚀");
        let json = r.to_json();
        assert_eq!(JsonRecord::from_json(&json).unwrap(), r);
    }

    #[test]
    fn u_escape_decodes() {
        let r = JsonRecord::from_json("{\"key\":6,\"value\":\"\\u0041\"}").unwrap();
        assert_eq!(r.value, "A");
    }

    #[test]
    fn reserved_tag_in_json_is_rejected() {
        assert!(JsonRecord::from_json("{\"key\":3,\"value\":\"x\"}").is_err());
    }

    #[test]
    fn out_of_range_tag_is_rejected() {
        assert!(JsonRecord::from_json("{\"key\":999,\"value\":\"x\"}").is_err());
    }

    #[test]
    fn missing_field_is_rejected() {
        assert!(JsonRecord::from_json("{\"key\":6}").is_err());
        assert!(JsonRecord::from_json("{\"value\":\"x\"}").is_err());
        assert!(JsonRecord::from_json("{}").is_err());
    }

    #[test]
    fn trailing_garbage_rejected() {
        assert!(JsonRecord::from_json("{\"key\":6,\"value\":\"x\"}junk").is_err());
        assert!(decode_records("[] extra").is_err());
    }

    #[test]
    fn malformed_json_rejected() {
        assert!(decode_records("[").is_err());
        assert!(decode_records("[{\"key\":6,\"value\":\"x\"}").is_err());
        assert!(JsonRecord::from_json("{\"key\":6 \"value\":\"x\"}").is_err());
        assert!(JsonRecord::from_json("not json").is_err());
    }

    #[test]
    fn adv_json_round_trip() {
        let mut adv = Adv::new();
        adv.set(MetaKey::Upgrade, MetaValue::from_str("v1.6.0").unwrap());
        adv.set(
            MetaKey::UniqueMachineToken,
            MetaValue::from_str("tok-xyz").unwrap(),
        );
        let json = adv_to_json(&adv).unwrap();
        let back = adv_from_json(&json).unwrap();
        assert_eq!(back, adv);
    }

    #[test]
    fn adv_to_json_rejects_binary_value() {
        let mut adv = Adv::new();
        adv.set(
            MetaKey::Custom(0x40),
            MetaValue::new(vec![0xff, 0xfe]).unwrap(),
        );
        assert!(adv_to_json(&adv).is_err());
    }

    #[test]
    fn adv_from_json_rejects_duplicate_keys() {
        let json = "[{\"key\":6,\"value\":\"a\"},{\"key\":6,\"value\":\"b\"}]";
        assert!(adv_from_json(json).is_err());
    }

    #[test]
    fn unknown_field_rejected() {
        assert!(JsonRecord::from_json("{\"key\":6,\"value\":\"x\",\"extra\":\"y\"}").is_err());
    }
}
