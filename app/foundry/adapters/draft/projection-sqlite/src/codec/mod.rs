//! The adapter's canonical persistence codec: deterministic bytes for
//! [`ProjectedObject`]s and [`AppliedEntry`]s. Tag values are FROZEN —
//! evolution mints the next version, never mutates a layout. Every
//! decode is fail-closed: unknown tags, truncated buffers, non-finite
//! doubles, invalid dates, and non-privacy classifications all refuse.

mod decode;

pub(crate) use decode::decode_object;

use data_boundary_kernel::{DataClass, DataClassification, PrivacyDataClass};
use data_ontology_kernel::{PropertyTier, PropertyValue};
use foundry_projection_draft::{AppliedEntry, EntryOutcome, ProjectedObject};

pub(crate) const CODEC_VERSION: u8 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CodecError {
    pub detail: &'static str,
}

pub(crate) fn refuse<T>(detail: &'static str) -> Result<T, CodecError> {
    Err(CodecError { detail })
}

pub(crate) fn encode_entry(entry: &AppliedEntry) -> Result<Vec<u8>, CodecError> {
    let mut out = vec![CODEC_VERSION];
    string(&mut out, &entry.tenant_id);
    out.extend(entry.ordinal.to_le_bytes());
    match &entry.outcome {
        EntryOutcome::Applied { objects } => {
            out.push(1);
            out.extend((objects.len() as u32).to_le_bytes());
            for object in objects {
                let bytes = encode_object(object)?;
                out.extend((bytes.len() as u32).to_le_bytes());
                out.extend(bytes);
            }
        }
        EntryOutcome::Poisoned { reason } => {
            out.push(2);
            string(&mut out, reason);
        }
    }
    Ok(out)
}

pub(crate) fn encode_object(object: &ProjectedObject) -> Result<Vec<u8>, CodecError> {
    let mut out = vec![CODEC_VERSION];
    string(&mut out, &object.entity.tenant_id);
    string(&mut out, &object.entity.id);
    string(&mut out, &object.entity.entity_type.value);
    out.push(class_tag(&object.entity.entity_type.data_class)?);
    out.extend((object.entity.properties.len() as u32).to_le_bytes());
    for (name, property) in &object.entity.properties {
        string(&mut out, name);
        out.push(tier_tag(property.tier));
        out.push(class_tag(&property.value.data_class)?);
        value(&mut out, &property.value.value);
    }
    out.extend(object.schema_revision.to_le_bytes());
    out.extend(object.last_ordinal.to_le_bytes());
    string(&mut out, &object.last_actor);
    Ok(out)
}

fn string(out: &mut Vec<u8>, text: &str) {
    out.extend((text.len() as u32).to_le_bytes());
    out.extend(text.as_bytes());
}

fn value(out: &mut Vec<u8>, item: &PropertyValue) {
    match item {
        PropertyValue::String(text) => {
            out.push(1);
            string(out, text);
        }
        PropertyValue::Integer(number) => {
            out.push(2);
            out.extend(number.to_le_bytes());
        }
        PropertyValue::Double(double) => {
            out.push(3);
            out.extend(double.get().to_bits().to_le_bytes());
        }
        PropertyValue::Boolean(flag) => {
            out.push(4);
            out.push(u8::from(*flag));
        }
        PropertyValue::Date(date) => {
            out.push(5);
            out.extend(date.year().to_le_bytes());
            out.push(date.month());
            out.push(date.day());
        }
        PropertyValue::Timestamp { epoch_millis } => {
            out.push(6);
            out.extend(epoch_millis.to_le_bytes());
        }
        PropertyValue::Array(items) => {
            out.push(7);
            out.extend((items.len() as u32).to_le_bytes());
            for item in items {
                value(out, item);
            }
        }
        PropertyValue::Struct(entries) => {
            out.push(8);
            out.extend((entries.len() as u32).to_le_bytes());
            for (key, item) in entries {
                string(out, key);
                value(out, item);
            }
        }
    }
}

fn tier_tag(tier: PropertyTier) -> u8 {
    match tier {
        PropertyTier::Scalar => 1,
        PropertyTier::Vector => 2,
        PropertyTier::Timeseries => 3,
        PropertyTier::Geo => 4,
        PropertyTier::Ciphertext => 5,
        PropertyTier::Struct => 6,
    }
}

pub(super) fn tier_from_tag(tag: u8) -> Result<PropertyTier, CodecError> {
    Ok(match tag {
        1 => PropertyTier::Scalar,
        2 => PropertyTier::Vector,
        3 => PropertyTier::Timeseries,
        4 => PropertyTier::Geo,
        5 => PropertyTier::Ciphertext,
        6 => PropertyTier::Struct,
        _ => return refuse("unknown tier tag"),
    })
}

/// Only privacy-program classifications are storable: the kernel's own
/// constructors admit nothing else into an object.
fn class_tag(classification: &DataClassification) -> Result<u8, CodecError> {
    let DataClassification::Privacy(privacy) = classification else {
        return refuse("non-privacy classification");
    };
    Ok(match privacy.data_class() {
        DataClass::Public => 1,
        DataClass::InternalOnly => 2,
        DataClass::PiiIdentifying => 3,
        DataClass::PiiSensitive => 4,
        DataClass::Phi => 5,
        DataClass::Pci => 6,
        DataClass::PipaArticle23 => 7,
        DataClass::Children => 8,
        DataClass::Financial => 9,
        DataClass::PiiQuasiIdentifier => 10,
        DataClass::FinancialRegulatedCredit => 11,
        DataClass::BehavioralTenantProduct => 12,
        DataClass::BehavioralAds => 13,
        DataClass::DeclaredPreference => 14,
        DataClass::SearchQuery => 15,
        DataClass::SensitivePipaArticle23 => 16,
        _ => return refuse("unmapped privacy data class"),
    })
}

pub(super) fn class_from_tag(tag: u8) -> Result<PrivacyDataClass, CodecError> {
    let data_class = match tag {
        1 => DataClass::Public,
        2 => DataClass::InternalOnly,
        3 => DataClass::PiiIdentifying,
        4 => DataClass::PiiSensitive,
        5 => DataClass::Phi,
        6 => DataClass::Pci,
        7 => DataClass::PipaArticle23,
        8 => DataClass::Children,
        9 => DataClass::Financial,
        10 => DataClass::PiiQuasiIdentifier,
        11 => DataClass::FinancialRegulatedCredit,
        12 => DataClass::BehavioralTenantProduct,
        13 => DataClass::BehavioralAds,
        14 => DataClass::DeclaredPreference,
        15 => DataClass::SearchQuery,
        16 => DataClass::SensitivePipaArticle23,
        _ => return refuse("unknown data-class tag"),
    };
    PrivacyDataClass::new(data_class).map_err(|_| CodecError {
        detail: "tag maps to a non-privacy class",
    })
}
