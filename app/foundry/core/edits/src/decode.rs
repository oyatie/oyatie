//! Strict canonical decoding: exactly one byte string decodes to any value,
//! and no non-canonical byte string decodes at all — the property that
//! makes whole-envelope byte-equality dedup honest.
//!
//! Decoding reconstructs through the validating constructors, so a corrupt
//! entry surfaces a typed [`DecodeError`] (the fold's poison reason) and an
//! invalid shape can never materialize.

use std::collections::BTreeMap;

use crate::edit::{EditError, EditSet, EditTag, OntologyEdit};
use crate::property::{WireDataClass, WireProperty, WirePropertyError, WireTier};
use crate::record::{ActionRecord, DenialRecord, RecordError, WIRE_FORMAT_VERSION};
use crate::value::{WireDate, WireDouble, WireValue};

/// Why a byte string was refused. Every variant is a fold poison reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// Input ended (or a length prefix overran the remaining input).
    Truncated,
    /// Bytes remained after the root value was fully decoded.
    TrailingBytes,
    /// The version prefix names a layout this decoder does not carry.
    UnsupportedWireVersion {
        found: u16,
    },
    UnknownValueTag {
        tag: u8,
    },
    /// A byte-frozen edit kind that is writer-refused until the kernel
    /// removal lane lands — distinct from corruption.
    ReservedEditKind {
        tag: u8,
    },
    UnknownEditTag {
        tag: u8,
    },
    UnknownTierTag {
        tag: u8,
    },
    UnknownDataClassTag {
        tag: u8,
    },
    InvalidBoolean {
        byte: u8,
    },
    /// A double key that construction could never produce (non-finite, or
    /// the folded `-0.0` key) — a second spelling of some number.
    NonCanonicalDouble,
    InvalidDate,
    InvalidUtf8,
    /// Struct keys must appear in strictly ascending byte order.
    NonCanonicalStructOrder,
    DuplicateStructKey,
    InvalidEdit(EditError),
    InvalidProperty(WirePropertyError),
    InvalidRecord(RecordError),
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        let end = self.at.checked_add(n).ok_or(DecodeError::Truncated)?;
        if end > self.bytes.len() {
            return Err(DecodeError::Truncated);
        }
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        self.take(N)?.try_into().map_err(|_| DecodeError::Truncated)
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn i32(&mut self) -> Result<i32, DecodeError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn i64(&mut self) -> Result<i64, DecodeError> {
        Ok(i64::from_le_bytes(self.array()?))
    }

    /// A u32 length prefix, bounds-checked against the remaining input
    /// BEFORE any allocation.
    fn len(&mut self) -> Result<usize, DecodeError> {
        let len = self.u32()? as usize;
        if len > self.bytes.len() - self.at {
            return Err(DecodeError::Truncated);
        }
        Ok(len)
    }

    fn string(&mut self) -> Result<String, DecodeError> {
        let len = self.len()?;
        let raw = self.take(len)?;
        String::from_utf8(raw.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }

    fn value(&mut self) -> Result<WireValue, DecodeError> {
        let tag = self.u8()?;
        match tag {
            0 => Ok(WireValue::String(self.string()?)),
            1 => Ok(WireValue::Integer(self.i64()?)),
            2 => {
                let key = self.u64()?;
                let double =
                    WireDouble::from_sort_key(key).map_err(|_| DecodeError::NonCanonicalDouble)?;
                Ok(WireValue::Double(double))
            }
            3 => match self.u8()? {
                0 => Ok(WireValue::Boolean(false)),
                1 => Ok(WireValue::Boolean(true)),
                byte => Err(DecodeError::InvalidBoolean { byte }),
            },
            4 => {
                let (year, month, day) = (self.i32()?, self.u8()?, self.u8()?);
                let date = WireDate::new(year, month, day).map_err(|_| DecodeError::InvalidDate)?;
                Ok(WireValue::Date(date))
            }
            5 => Ok(WireValue::Timestamp {
                epoch_millis: self.i64()?,
            }),
            6 => {
                let count = self.len()?;
                let mut items = Vec::new();
                for _ in 0..count {
                    items.push(self.value()?);
                }
                Ok(WireValue::Array(items))
            }
            7 => {
                let count = self.len()?;
                let mut entries = BTreeMap::new();
                let mut previous: Option<String> = None;
                for _ in 0..count {
                    let key = self.string()?;
                    if let Some(prev) = &previous {
                        if key == *prev {
                            return Err(DecodeError::DuplicateStructKey);
                        }
                        if key < *prev {
                            return Err(DecodeError::NonCanonicalStructOrder);
                        }
                    }
                    let entry = self.value()?;
                    previous = Some(key.clone());
                    entries.insert(key, entry);
                }
                Ok(WireValue::Struct(entries))
            }
            tag => Err(DecodeError::UnknownValueTag { tag }),
        }
    }

    fn property(&mut self) -> Result<WireProperty, DecodeError> {
        let name = self.string()?;
        let tier_tag = self.u8()?;
        let tier =
            WireTier::from_tag(tier_tag).ok_or(DecodeError::UnknownTierTag { tag: tier_tag })?;
        let class_tag = self.u8()?;
        let data_class = WireDataClass::from_tag(class_tag)
            .ok_or(DecodeError::UnknownDataClassTag { tag: class_tag })?;
        let value = self.value()?;
        WireProperty::new(name, tier, data_class, value).map_err(DecodeError::InvalidProperty)
    }

    fn properties(&mut self) -> Result<Vec<WireProperty>, DecodeError> {
        let count = self.len()?;
        let mut properties = Vec::new();
        for _ in 0..count {
            properties.push(self.property()?);
        }
        Ok(properties)
    }

    fn edit(&mut self) -> Result<OntologyEdit, DecodeError> {
        let tag = self.u8()?;
        match EditTag::from_tag(tag) {
            Some(EditTag::CreateObject) => {
                let entity_type = self.string()?;
                let properties = self.properties()?;
                OntologyEdit::create_object(entity_type, properties)
                    .map_err(DecodeError::InvalidEdit)
            }
            Some(EditTag::UpsertProperties) => {
                let set = self.properties()?;
                OntologyEdit::upsert_properties(set).map_err(DecodeError::InvalidEdit)
            }
            Some(EditTag::CreateLink) => {
                let link_type = self.string()?;
                let to_entity_id = self.string()?;
                OntologyEdit::create_link(link_type, to_entity_id).map_err(DecodeError::InvalidEdit)
            }
            Some(reserved) if reserved.is_reserved() => Err(DecodeError::ReservedEditKind { tag }),
            _ => Err(DecodeError::UnknownEditTag { tag }),
        }
    }

    fn version(&mut self) -> Result<(), DecodeError> {
        let found = self.u16()?;
        if found != WIRE_FORMAT_VERSION {
            return Err(DecodeError::UnsupportedWireVersion { found });
        }
        Ok(())
    }

    fn finish(self) -> Result<(), DecodeError> {
        if self.at != self.bytes.len() {
            return Err(DecodeError::TrailingBytes);
        }
        Ok(())
    }
}

/// Decode a canonical [`ActionRecord`]; refuses every non-canonical input.
pub fn decode_action_record(bytes: &[u8]) -> Result<ActionRecord, DecodeError> {
    let mut cursor = Cursor { bytes, at: 0 };
    cursor.version()?;
    let principal_id = cursor.string()?;
    let decision_id = cursor.string()?;
    let audit_event_type = cursor.string()?;
    let idempotency_key = cursor.string()?;
    let occurred_at_epoch_ms = cursor.u64()?;
    let parameters = cursor.properties()?;
    let count = cursor.len()?;
    let mut edits = Vec::new();
    for _ in 0..count {
        edits.push(cursor.edit()?);
    }
    let edits = EditSet::new(edits).map_err(DecodeError::InvalidEdit)?;
    let record = ActionRecord::new(
        principal_id,
        decision_id,
        audit_event_type,
        idempotency_key,
        occurred_at_epoch_ms,
        parameters,
        edits,
    )
    .map_err(DecodeError::InvalidRecord)?;
    cursor.finish()?;
    Ok(record)
}

/// Decode a canonical [`DenialRecord`]; refuses every non-canonical input.
pub fn decode_denial_record(bytes: &[u8]) -> Result<DenialRecord, DecodeError> {
    let mut cursor = Cursor { bytes, at: 0 };
    cursor.version()?;
    let gate = cursor.string()?;
    let cause = cursor.string()?;
    let principal_id = cursor.string()?;
    let decision_id = cursor.string()?;
    let action_id = cursor.string()?;
    let object_ref = cursor.string()?;
    let occurred_at_epoch_ms = cursor.u64()?;
    let record = DenialRecord::new(
        gate,
        cause,
        principal_id,
        decision_id,
        action_id,
        object_ref,
        occurred_at_epoch_ms,
    )
    .map_err(DecodeError::InvalidRecord)?;
    cursor.finish()?;
    Ok(record)
}
