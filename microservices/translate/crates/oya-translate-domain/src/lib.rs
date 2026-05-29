//! Workspace translate kernel.
//!
//! Typed kernel records for the W-Workspace-GA Translate adjunct surface named
//! by `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns locale
//! identifiers, per-tenant glossary contracts, provider-route binding, and
//! translation request/result validation without owning provider networking or
//! model execution.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const GLOSSARY_SCHEMA_VERSION: u32 = 1;
const TRANSLATE_REQUEST_SCHEMA_VERSION: u32 = 1;
const TRANSLATION_JOB_SCHEMA_VERSION: u32 = 1;
const TRANSLATION_RESULT_SCHEMA_VERSION: u32 = 1;
const MIN_TRANSLATION_BYTES: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranslateError {
    InvalidGlossaryId,
    InvalidRequestId,
    InvalidJobId,
    InvalidResultId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidActorRef,
    InvalidLocale,
    SameSourceAndTargetLocale,
    EmptyGlossaryEntries,
    InvalidGlossaryTerm,
    DuplicateGlossarySourceTerm,
    GlossaryTenantMismatch,
    GlossaryLocaleMismatch,
    GlossaryRequestMismatch,
    InvalidSourceText,
    InvalidTranslatedText,
    InvalidMaxOutputChars,
    InvalidProviderId,
    InvalidAdapterCapabilityId,
    InvalidProviderRegion,
    InvalidModelRef,
    InvalidIdempotencyKey,
    MissingDataClassAllowlist,
    DataClassNotAllowed,
    ProviderRegionMismatch,
    JobRequestMismatch,
    JobTenantMismatch,
    ResultRequestMismatch,
    ResultJobMismatch,
    ResultTenantMismatch,
    EmptyTranslationResult,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TranslationStatus {
    Requested,
    Routed,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocaleId {
    pub value: Classified<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslateGlossaryCreate {
    pub glossary_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub source_locale: LocaleId,              // data_class: INTERNAL_ONLY
    pub target_locale: LocaleId,              // data_class: INTERNAL_ONLY
    pub entries: Vec<GlossaryEntry>,          // data_class: PII_IDENTIFYING
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslateGlossary {
    pub glossary_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub source_locale: Classified<LocaleId>, // data_class: INTERNAL_ONLY
    pub target_locale: Classified<LocaleId>, // data_class: INTERNAL_ONLY
    pub entries: Classified<Vec<GlossaryEntry>>, // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GlossaryEntry {
    pub source_term: Classified<String>, // data_class: PII_IDENTIFYING
    pub target_term: Classified<String>, // data_class: PII_IDENTIFYING
    pub case_sensitive: Classified<bool>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationRequestCreate {
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub actor_ref: String,                    // data_class: PII_IDENTIFYING
    pub source_locale: LocaleId,              // data_class: INTERNAL_ONLY
    pub target_locale: LocaleId,              // data_class: INTERNAL_ONLY
    pub source_text: String,                  // data_class: PII_IDENTIFYING
    pub glossary_id: Option<String>,          // data_class: INTERNAL_ONLY
    pub max_output_chars: u32,                // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationRequest {
    pub request_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub actor_ref: Classified<String>,  // data_class: PII_IDENTIFYING
    pub source_locale: Classified<LocaleId>, // data_class: INTERNAL_ONLY
    pub target_locale: Classified<LocaleId>, // data_class: INTERNAL_ONLY
    pub source_text: Classified<String>, // data_class: PII_IDENTIFYING
    pub glossary_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub max_output_chars: Classified<u32>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslateProviderBinding {
    pub provider_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub adapter_capability_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub provider_region: Classified<String>, // data_class: INTERNAL_ONLY
    pub model_ref: Classified<String>,   // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationJobCreate {
    pub job_id: String,                             // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub provider_binding: TranslateProviderBinding, // data_class: INTERNAL_ONLY
    pub status: TranslationStatus,                  // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,              // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationJob {
    pub job_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub request_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub provider_binding: Classified<TranslateProviderBinding>, // data_class: INTERNAL_ONLY
    pub status: Classified<TranslationStatus>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationResultCreate {
    pub result_id: String,                    // data_class: INTERNAL_ONLY
    pub job_id: String,                       // data_class: INTERNAL_ONLY
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub output_locale: LocaleId,              // data_class: INTERNAL_ONLY
    pub translated_text: String,              // data_class: PII_IDENTIFYING
    pub byte_len: u64,                        // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslationResult {
    pub result_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub job_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub request_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub output_locale: Classified<LocaleId>, // data_class: INTERNAL_ONLY
    pub translated_text: Classified<String>, // data_class: PII_IDENTIFYING
    pub byte_len: Classified<u64>,      // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

pub trait TranslationProvider {
    fn translate(&self, job: &TranslationJob) -> Result<TranslationResult, TranslateError>;
}

impl LocaleId {
    pub fn new(value: String) -> Result<Self, TranslateError> {
        validate_locale(&value)?;
        Ok(Self {
            value: internal(value),
        })
    }
}

impl TranslateGlossary {
    pub fn new(input: TranslateGlossaryCreate) -> Result<Self, TranslateError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_translation_data_class());
        validate_non_empty(&input.glossary_id, TranslateError::InvalidGlossaryId)?;
        validate_non_empty(&input.tenant_id, TranslateError::InvalidTenantId)?;
        validate_locale_pair(&input.source_locale, &input.target_locale)?;
        validate_glossary_entries(&input.entries)?;
        Ok(Self {
            glossary_id: internal(input.glossary_id),
            tenant_id: internal(input.tenant_id),
            source_locale: internal(input.source_locale),
            target_locale: internal(input.target_locale),
            entries: Classified::new(input.entries, translation_content_data_class()),
            data_class: internal(data_class),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(GLOSSARY_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl GlossaryEntry {
    pub fn new(
        source_term: String,
        target_term: String,
        case_sensitive: bool,
    ) -> Result<Self, TranslateError> {
        validate_text(&source_term, TranslateError::InvalidGlossaryTerm)?;
        validate_text(&target_term, TranslateError::InvalidGlossaryTerm)?;
        Ok(Self {
            source_term: Classified::new(source_term, translation_content_data_class()),
            target_term: Classified::new(target_term, translation_content_data_class()),
            case_sensitive: internal(case_sensitive),
        })
    }
}

impl TranslationRequest {
    pub fn new(
        input: TranslationRequestCreate,
        glossary: Option<&TranslateGlossary>,
    ) -> Result<Self, TranslateError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_translation_data_class());
        validate_non_empty(&input.request_id, TranslateError::InvalidRequestId)?;
        validate_non_empty(&input.tenant_id, TranslateError::InvalidTenantId)?;
        validate_non_empty(&input.region, TranslateError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, TranslateError::InvalidCellId)?;
        validate_non_empty(&input.actor_ref, TranslateError::InvalidActorRef)?;
        validate_locale_pair(&input.source_locale, &input.target_locale)?;
        validate_text(&input.source_text, TranslateError::InvalidSourceText)?;
        if input.max_output_chars == 0 {
            return Err(TranslateError::InvalidMaxOutputChars);
        }
        validate_request_glossary(&input, glossary)?;

        Ok(Self {
            request_id: internal(input.request_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            actor_ref: Classified::new(input.actor_ref, translation_actor_data_class()),
            source_locale: internal(input.source_locale),
            target_locale: internal(input.target_locale),
            source_text: Classified::new(input.source_text, translation_content_data_class()),
            glossary_id: internal(input.glossary_id),
            max_output_chars: internal(input.max_output_chars),
            data_class: internal(data_class),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: internal(TRANSLATE_REQUEST_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl TranslateProviderBinding {
    pub fn new(
        provider_id: String,
        adapter_capability_id: String,
        provider_region: String,
        model_ref: String,
        idempotency_key: String,
        allowed_data_classes: Vec<PrivacyDataClass>,
    ) -> Result<Self, TranslateError> {
        validate_provider_id(&provider_id)?;
        validate_capability_id(&adapter_capability_id)?;
        validate_non_empty(&provider_region, TranslateError::InvalidProviderRegion)?;
        validate_non_empty(&model_ref, TranslateError::InvalidModelRef)?;
        validate_non_empty(&idempotency_key, TranslateError::InvalidIdempotencyKey)?;
        validate_data_class_allowlist(&allowed_data_classes)?;
        Ok(Self {
            provider_id: internal(provider_id),
            adapter_capability_id: internal(adapter_capability_id),
            provider_region: internal(provider_region),
            model_ref: internal(model_ref),
            idempotency_key: internal(idempotency_key),
            allowed_data_classes: internal(allowed_data_classes),
        })
    }

    fn validate_for_request(&self, request: &TranslationRequest) -> Result<(), TranslateError> {
        validate_provider_id(&self.provider_id.value)?;
        validate_capability_id(&self.adapter_capability_id.value)?;
        validate_non_empty(
            &self.provider_region.value,
            TranslateError::InvalidProviderRegion,
        )?;
        validate_non_empty(&self.model_ref.value, TranslateError::InvalidModelRef)?;
        validate_non_empty(
            &self.idempotency_key.value,
            TranslateError::InvalidIdempotencyKey,
        )?;
        validate_data_class_allowlist(&self.allowed_data_classes.value)?;
        if self.provider_region.value != request.region.value {
            return Err(TranslateError::ProviderRegionMismatch);
        }
        if !self
            .allowed_data_classes
            .value
            .contains(&request.data_class.value)
        {
            return Err(TranslateError::DataClassNotAllowed);
        }
        Ok(())
    }
}

impl TranslationJob {
    pub fn new(
        input: TranslationJobCreate,
        request: &TranslationRequest,
    ) -> Result<Self, TranslateError> {
        validate_non_empty(&input.job_id, TranslateError::InvalidJobId)?;
        validate_non_empty(&input.request_id, TranslateError::InvalidRequestId)?;
        validate_non_empty(&input.tenant_id, TranslateError::InvalidTenantId)?;
        if input.request_id != request.request_id.value {
            return Err(TranslateError::JobRequestMismatch);
        }
        if input.tenant_id != request.tenant_id.value {
            return Err(TranslateError::JobTenantMismatch);
        }
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        input.provider_binding.validate_for_request(request)?;
        Ok(Self {
            job_id: internal(input.job_id),
            request_id: internal(input.request_id),
            tenant_id: internal(input.tenant_id),
            provider_binding: internal(input.provider_binding),
            status: internal(input.status),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(TRANSLATION_JOB_SCHEMA_VERSION),
        })
    }
}

impl TranslationResult {
    pub fn new(
        input: TranslationResultCreate,
        request: &TranslationRequest,
        job: &TranslationJob,
    ) -> Result<Self, TranslateError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_translation_data_class());
        validate_non_empty(&input.result_id, TranslateError::InvalidResultId)?;
        validate_non_empty(&input.job_id, TranslateError::InvalidJobId)?;
        validate_non_empty(&input.request_id, TranslateError::InvalidRequestId)?;
        validate_non_empty(&input.tenant_id, TranslateError::InvalidTenantId)?;
        validate_text(
            &input.translated_text,
            TranslateError::InvalidTranslatedText,
        )?;
        if input.job_id != job.job_id.value {
            return Err(TranslateError::ResultJobMismatch);
        }
        if input.request_id != request.request_id.value || input.request_id != job.request_id.value
        {
            return Err(TranslateError::ResultRequestMismatch);
        }
        if input.tenant_id != request.tenant_id.value || input.tenant_id != job.tenant_id.value {
            return Err(TranslateError::ResultTenantMismatch);
        }
        if input.output_locale != request.target_locale.value {
            return Err(TranslateError::InvalidLocale);
        }
        if input.byte_len < MIN_TRANSLATION_BYTES {
            return Err(TranslateError::EmptyTranslationResult);
        }
        Ok(Self {
            result_id: internal(input.result_id),
            job_id: internal(input.job_id),
            request_id: internal(input.request_id),
            tenant_id: internal(input.tenant_id),
            output_locale: internal(input.output_locale),
            translated_text: Classified::new(
                input.translated_text,
                translation_content_data_class(),
            ),
            byte_len: internal(input.byte_len),
            data_class: internal(data_class),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            schema_version: internal(TRANSLATION_RESULT_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

pub fn default_workspace_translation_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn translation_content_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn translation_actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_translation_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, TranslateError> {
    PrivacyDataClass::new(data_class).map_err(|_| TranslateError::InvalidDataClass)
}

fn validate_request_glossary(
    input: &TranslationRequestCreate,
    glossary: Option<&TranslateGlossary>,
) -> Result<(), TranslateError> {
    match (input.glossary_id.as_deref(), glossary) {
        (None, None) => Ok(()),
        (Some(_), None) | (None, Some(_)) => Err(TranslateError::GlossaryRequestMismatch),
        (Some(glossary_id), Some(glossary)) => {
            if glossary_id != glossary.glossary_id.value {
                return Err(TranslateError::GlossaryRequestMismatch);
            }
            if input.tenant_id != glossary.tenant_id.value {
                return Err(TranslateError::GlossaryTenantMismatch);
            }
            if input.source_locale != glossary.source_locale.value
                || input.target_locale != glossary.target_locale.value
            {
                return Err(TranslateError::GlossaryLocaleMismatch);
            }
            Ok(())
        }
    }
}

fn validate_locale_pair(source: &LocaleId, target: &LocaleId) -> Result<(), TranslateError> {
    if source == target {
        Err(TranslateError::SameSourceAndTargetLocale)
    } else {
        Ok(())
    }
}

fn validate_locale(locale: &str) -> Result<(), TranslateError> {
    if locale.trim() != locale || locale.is_empty() || locale.len() > 35 {
        return Err(TranslateError::InvalidLocale);
    }
    for segment in locale.split('-') {
        if segment.is_empty()
            || segment.len() > 8
            || !segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
        {
            return Err(TranslateError::InvalidLocale);
        }
    }
    Ok(())
}

fn validate_glossary_entries(entries: &[GlossaryEntry]) -> Result<(), TranslateError> {
    if entries.is_empty() {
        return Err(TranslateError::EmptyGlossaryEntries);
    }
    let mut source_terms = BTreeSet::new();
    for entry in entries {
        validate_text(
            &entry.source_term.value,
            TranslateError::InvalidGlossaryTerm,
        )?;
        validate_text(
            &entry.target_term.value,
            TranslateError::InvalidGlossaryTerm,
        )?;
        let key = if entry.case_sensitive.value {
            entry.source_term.value.clone()
        } else {
            entry.source_term.value.to_ascii_lowercase()
        };
        if !source_terms.insert(key) {
            return Err(TranslateError::DuplicateGlossarySourceTerm);
        }
    }
    Ok(())
}

fn validate_provider_id(provider_id: &str) -> Result<(), TranslateError> {
    if provider_id.trim() != provider_id
        || provider_id.is_empty()
        || !provider_id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        Err(TranslateError::InvalidProviderId)
    } else {
        Ok(())
    }
}

fn validate_capability_id(capability_id: &str) -> Result<(), TranslateError> {
    if capability_id.starts_with("cap.") && capability_id.len() > "cap.".len() {
        Ok(())
    } else {
        Err(TranslateError::InvalidAdapterCapabilityId)
    }
}

fn validate_data_class_allowlist(data_classes: &[PrivacyDataClass]) -> Result<(), TranslateError> {
    if data_classes.is_empty() {
        Err(TranslateError::MissingDataClassAllowlist)
    } else {
        Ok(())
    }
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), TranslateError> {
    if updated_at < created_at {
        Err(TranslateError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_text(value: &str, error: TranslateError) -> Result<(), TranslateError> {
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: TranslateError) -> Result<(), TranslateError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_data_boundary_kernel::{DataClassification, OperationalDataClass};

    const SOURCE_LOCALE: &str = "lang-alpha1";
    const TARGET_LOCALE: &str = "lang-beta1";
    const ALT_TARGET_LOCALE: &str = "lang-gamma1";
    const INVALID_LOCALE_WITH_SPACE: &str = "lang alpha1";
    const REGION: &str = "region-alpha1";
    const OTHER_REGION: &str = "region-beta1";

    fn locale(value: &str) -> LocaleId {
        LocaleId::new(value.into()).unwrap()
    }

    fn entry(source: &str, target: &str) -> GlossaryEntry {
        GlossaryEntry::new(source.into(), target.into(), false).unwrap()
    }

    fn glossary() -> TranslateGlossary {
        TranslateGlossary::new(TranslateGlossaryCreate {
            glossary_id: "glossary-1".into(),
            tenant_id: "tenant-1".into(),
            source_locale: locale(SOURCE_LOCALE),
            target_locale: locale(TARGET_LOCALE),
            entries: vec![entry("incident", "장애")],
            data_class: None,
            updated_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap()
    }

    fn request_input() -> TranslationRequestCreate {
        TranslationRequestCreate {
            request_id: "translate-1".into(),
            tenant_id: "tenant-1".into(),
            region: REGION.into(),
            cell_id: "cell-a".into(),
            actor_ref: "user:translator@example.com".into(),
            source_locale: locale(SOURCE_LOCALE),
            target_locale: locale(TARGET_LOCALE),
            source_text: "incident review".into(),
            glossary_id: Some("glossary-1".into()),
            max_output_chars: 4096,
            data_class: None,
            requested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn request() -> TranslationRequest {
        TranslationRequest::new(request_input(), Some(&glossary())).unwrap()
    }

    fn binding() -> TranslateProviderBinding {
        TranslateProviderBinding::new(
            "foundry-translate".into(),
            "cap.workspace.translate".into(),
            REGION.into(),
            "translate-model-v1".into(),
            "idem-1".into(),
            vec![default_workspace_translation_data_class()],
        )
        .unwrap()
    }

    fn job() -> TranslationJob {
        TranslationJob::new(
            TranslationJobCreate {
                job_id: "job-1".into(),
                request_id: "translate-1".into(),
                tenant_id: "tenant-1".into(),
                provider_binding: binding(),
                status: TranslationStatus::Completed,
                created_at_epoch_seconds: 1_700_000_011,
                updated_at_epoch_seconds: 1_700_000_012,
            },
            &request(),
        )
        .unwrap()
    }

    #[test]
    fn locale_and_glossary_shape_are_fail_closed() {
        assert_eq!(
            LocaleId::new(INVALID_LOCALE_WITH_SPACE.into()),
            Err(TranslateError::InvalidLocale)
        );

        assert_eq!(
            TranslateGlossary::new(TranslateGlossaryCreate {
                glossary_id: "glossary-dup".into(),
                tenant_id: "tenant-1".into(),
                source_locale: locale(SOURCE_LOCALE),
                target_locale: locale(TARGET_LOCALE),
                entries: vec![entry("Incident", "장애"), entry("incident", "사고")],
                data_class: None,
                updated_at_epoch_seconds: 1,
            }),
            Err(TranslateError::DuplicateGlossarySourceTerm)
        );
    }

    #[test]
    fn request_requires_distinct_locale_and_matching_glossary() {
        let glossary = glossary();
        let request = request();
        assert_eq!(
            request.source_text.data_class,
            DataClassification::Privacy(translation_content_data_class())
        );
        assert_eq!(
            request.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );

        let mut same_locale = request_input();
        same_locale.target_locale = locale(SOURCE_LOCALE);
        assert_eq!(
            TranslationRequest::new(same_locale, Some(&glossary)),
            Err(TranslateError::SameSourceAndTargetLocale)
        );

        let mut wrong_glossary = request_input();
        wrong_glossary.glossary_id = Some("missing".into());
        assert_eq!(
            TranslationRequest::new(wrong_glossary, Some(&glossary)),
            Err(TranslateError::GlossaryRequestMismatch)
        );
    }

    #[test]
    fn provider_route_requires_region_and_data_class_allowance() {
        let request = request();
        let mut wrong_region = binding();
        wrong_region.provider_region = internal(OTHER_REGION.into());
        assert_eq!(
            TranslationJob::new(
                TranslationJobCreate {
                    job_id: "job-wrong-region".into(),
                    request_id: "translate-1".into(),
                    tenant_id: "tenant-1".into(),
                    provider_binding: wrong_region,
                    status: TranslationStatus::Routed,
                    created_at_epoch_seconds: 1,
                    updated_at_epoch_seconds: 2,
                },
                &request,
            ),
            Err(TranslateError::ProviderRegionMismatch)
        );

        assert_eq!(
            TranslateProviderBinding::new(
                "foundry-translate".into(),
                "cap.workspace.translate".into(),
                REGION.into(),
                "translate-model-v1".into(),
                "idem-1".into(),
                Vec::new(),
            ),
            Err(TranslateError::MissingDataClassAllowlist)
        );
    }

    #[test]
    fn result_must_match_job_request_and_target_locale() {
        let request = request();
        let job = job();
        let result = TranslationResult::new(
            TranslationResultCreate {
                result_id: "result-1".into(),
                job_id: "job-1".into(),
                request_id: "translate-1".into(),
                tenant_id: "tenant-1".into(),
                output_locale: locale(TARGET_LOCALE),
                translated_text: "장애 검토".into(),
                byte_len: 16,
                data_class: None,
                completed_at_epoch_seconds: 1_700_000_020,
            },
            &request,
            &job,
        )
        .unwrap();
        assert_eq!(result.schema_version.value, 1);

        assert_eq!(
            TranslationResult::new(
                TranslationResultCreate {
                    result_id: "result-2".into(),
                    job_id: "job-1".into(),
                    request_id: "translate-1".into(),
                    tenant_id: "tenant-1".into(),
                    output_locale: locale(ALT_TARGET_LOCALE),
                    translated_text: "レビュー".into(),
                    byte_len: 12,
                    data_class: None,
                    completed_at_epoch_seconds: 1_700_000_020,
                },
                &request,
                &job,
            ),
            Err(TranslateError::InvalidLocale)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_translation_data_class_from_legacy(DataClass::Audit),
            Err(TranslateError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.translate STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranslateSurfaceStaging {
    pub job_id: Classified<String>,        // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub source_locale: Classified<String>, // data_class: INTERNAL_ONLY
}

impl TranslateSurfaceStaging {
    pub fn new(job_id: String, tenant_id: String, source_locale: String) -> Self {
        Self {
            job_id: Classified::new(job_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            source_locale: Classified::new(source_locale, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> TranslateSurfaceStaging {
        TranslateSurfaceStaging::new(
            "translate-1".into(),
            "translate-1".into(),
            "translate-1".into(),
        )
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.job_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
