pub fn parse_data_class_pascal_label(label: &str) -> Option<DataClass> {
    match label.trim() {
        "Public" => Some(DataClass::Public),
        "InternalOnly" => Some(DataClass::InternalOnly),
        "PiiIdentifying" => Some(DataClass::PiiIdentifying),
        "PiiSensitive" => Some(DataClass::PiiSensitive),
        "Phi" => Some(DataClass::Phi),
        "Pci" => Some(DataClass::Pci),
        "PipaArticle23" => Some(DataClass::PipaArticle23),
        "Children" => Some(DataClass::Children),
        "Financial" => Some(DataClass::Financial),
        "Usage" => Some(DataClass::Usage),
        "Secret" => Some(DataClass::Secret),
        "Audit" => Some(DataClass::Audit),
        "PiiQuasiIdentifier" => Some(DataClass::PiiQuasiIdentifier),
        "FinancialRegulatedCredit" => Some(DataClass::FinancialRegulatedCredit),
        "BehavioralTenantProduct" => Some(DataClass::BehavioralTenantProduct),
        "BehavioralAds" => Some(DataClass::BehavioralAds),
        "DeclaredPreference" => Some(DataClass::DeclaredPreference),
        "SearchQuery" => Some(DataClass::SearchQuery),
        "SensitivePipaArticle23" => Some(DataClass::SensitivePipaArticle23),
        _ => None,
    }
}

pub fn parse_operational_data_class_label(label: &str) -> Option<OperationalDataClass> {
    match label.trim() {
        "AUDIT" => Some(OperationalDataClass::Audit),
        "SECRET" => Some(OperationalDataClass::Secret),
        _ => None,
    }
}

pub fn parse_subject_data_marker_label(label: &str) -> Option<SubjectDataMarker> {
    match label.trim() {
        "CHILDREN" => Some(SubjectDataMarker::Children),
        _ => None,
    }
}

/// Parse a privacy data-class label from catalog/OpenAPI surfaces.
///
/// Privacy surfaces are strict about operational/subject labels: `AUDIT`,
/// `SECRET`, and `CHILDREN` are accepted only by the explicitly named legacy
/// PascalCase ledger parser or the operational/subject marker parsers.
pub fn parse_data_class_label(label: &str) -> Option<DataClass> {
    match label.trim() {
        "PUBLIC" => Some(DataClass::Public),
        "INTERNAL_ONLY" => Some(DataClass::InternalOnly),
        "PII_IDENTIFYING" => Some(DataClass::PiiIdentifying),
        "PII_SENSITIVE" => Some(DataClass::PiiSensitive),
        "PII_QUASI_IDENTIFIER" => Some(DataClass::PiiQuasiIdentifier),
        "PHI" => Some(DataClass::Phi),
        "PCI" => Some(DataClass::Pci),
        "PIPA_ARTICLE_23" | "PIPA_ARTICLE23" => Some(DataClass::PipaArticle23),
        "SENSITIVE_PIPA_ART23" => Some(DataClass::SensitivePipaArticle23),
        "FINANCIAL" => Some(DataClass::Financial),
        "FINANCIAL_REGULATED_CREDIT" => Some(DataClass::FinancialRegulatedCredit),
        "USAGE" => Some(DataClass::Usage),
        "BEHAVIORAL_TENANT_PRODUCT" => Some(DataClass::BehavioralTenantProduct),
        "BEHAVIORAL_ADS" => Some(DataClass::BehavioralAds),
        "DECLARED_PREFERENCE" => Some(DataClass::DeclaredPreference),
        "SEARCH_QUERY" => Some(DataClass::SearchQuery),
        _ => None,
    }
}
