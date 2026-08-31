//! The ONE seam between the wire plane and the kernel: every conversion
//! from `foundry-edits` shapes to kernel carriers lives here, so a kernel
//! carrier change has exactly one file of blast radius in the spine.

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    CalendarDate, FiniteDouble, ObjectProperty, PropertyTier, PropertyValue,
};
use foundry_edits::{WireDataClass, WireProperty, WireTier, WireValue};

/// A wire shape the kernel refuses to carry. Deterministic and typed —
/// a fold poison reason, never a panic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoundaryError {
    /// The kernel rejected the value (non-finite double or invalid date
    /// keys cannot arrive from canonical decode, but the conversion stays
    /// fail-closed rather than trusting its caller).
    UnrepresentableValue,
    /// The label maps to no privacy-program class in the kernel taxonomy.
    UnrepresentableDataClass,
}

pub fn tier(wire: WireTier) -> PropertyTier {
    match wire {
        WireTier::Scalar => PropertyTier::Scalar,
        WireTier::Vector => PropertyTier::Vector,
        WireTier::Timeseries => PropertyTier::Timeseries,
        WireTier::Geo => PropertyTier::Geo,
        WireTier::Ciphertext => PropertyTier::Ciphertext,
        WireTier::Struct => PropertyTier::Struct,
    }
}

/// The canonical in-memory class per wire label. The platform's label
/// mapping is many-to-one (`PiiSensitive` and `PiiQuasiIdentifier` both
/// label `PII_QUASI_IDENTIFIER`, and so on); this direction picks ONE
/// canonical `DataClass` per label and the choice is frozen here.
pub fn data_class(wire: WireDataClass) -> Result<PrivacyDataClass, BoundaryError> {
    let canonical = match wire {
        WireDataClass::Public => DataClass::Public,
        WireDataClass::InternalOnly => DataClass::InternalOnly,
        WireDataClass::PiiIdentifying => DataClass::PiiIdentifying,
        WireDataClass::PiiQuasiIdentifier => DataClass::PiiQuasiIdentifier,
        WireDataClass::Phi => DataClass::Phi,
        WireDataClass::Pci => DataClass::Pci,
        WireDataClass::Financial => DataClass::Financial,
        WireDataClass::FinancialRegulatedCredit => DataClass::FinancialRegulatedCredit,
        WireDataClass::BehavioralTenantProduct => DataClass::BehavioralTenantProduct,
        WireDataClass::BehavioralAds => DataClass::BehavioralAds,
        WireDataClass::DeclaredPreference => DataClass::DeclaredPreference,
        WireDataClass::SearchQuery => DataClass::SearchQuery,
        WireDataClass::SensitivePipaArt23 => DataClass::SensitivePipaArticle23,
    };
    PrivacyDataClass::new(canonical).map_err(|_| BoundaryError::UnrepresentableDataClass)
}

pub fn value(wire: &WireValue) -> Result<PropertyValue, BoundaryError> {
    Ok(match wire {
        WireValue::String(text) => PropertyValue::String(text.clone()),
        WireValue::Integer(number) => PropertyValue::Integer(*number),
        WireValue::Double(double) => PropertyValue::Double(
            FiniteDouble::new(double.get()).map_err(|_| BoundaryError::UnrepresentableValue)?,
        ),
        WireValue::Boolean(flag) => PropertyValue::Boolean(*flag),
        WireValue::Date(date) => PropertyValue::Date(
            CalendarDate::new(date.year(), date.month(), date.day())
                .map_err(|_| BoundaryError::UnrepresentableValue)?,
        ),
        WireValue::Timestamp { epoch_millis } => PropertyValue::Timestamp {
            epoch_millis: *epoch_millis,
        },
        WireValue::Array(items) => {
            PropertyValue::Array(items.iter().map(value).collect::<Result<_, _>>()?)
        }
        WireValue::Struct(entries) => PropertyValue::Struct(
            entries
                .iter()
                .map(|(k, v)| Ok((k.clone(), value(v)?)))
                .collect::<Result<_, BoundaryError>>()?,
        ),
    })
}

/// The declared wire tier is preserved verbatim — a legacy String carrier
/// under an exotic tier crosses unchanged; nothing re-derives the tier
/// from the value's shape here.
pub fn property(wire: &WireProperty) -> Result<ObjectProperty, BoundaryError> {
    Ok(ObjectProperty {
        name: wire.name.clone(),
        value: Classified::new(value(&wire.value)?, data_class(wire.data_class)?),
        tier: tier(wire.tier),
    })
}
