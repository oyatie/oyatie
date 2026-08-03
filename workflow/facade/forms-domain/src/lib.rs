//! Workspace forms kernel.
//!
//! Typed kernel records for the W-Workspace-GA Forms surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns form schema,
//! Object-Graph route references, field answers, submission validation, and
//! conservative response data-class defaults without owning REST, storage, or
//! Object-Graph adapter code.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const FORM_SCHEMA_VERSION: u32 = 1;
const FORM_SUBMISSION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormError {
    InvalidFormId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidObjectGraphRouteId,
    InvalidTitle,
    EmptyFieldSet,
    InvalidFieldId,
    InvalidFieldLabel,
    DuplicateFieldId,
    MissingChoiceOptions,
    UnexpectedChoiceOptions,
    InvalidChoiceOption,
    InvalidSubmissionId,
    InvalidSubmitterRef,
    InvalidAnswerValue,
    DuplicateAnswerField,
    UnknownAnswerField,
    MissingRequiredAnswer,
    AnswerKindMismatch,
    SubmissionFormMismatch,
    SubmissionTenantMismatch,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FormFieldKind {
    ShortText,
    LongText,
    Number,
    Boolean,
    SingleChoice,
    MultiChoice,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub object_graph_route_id: String,        // data_class: INTERNAL_ONLY
    pub title: String,                        // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub fields: Vec<FormField>,               // data_class: PII_QUASI_IDENTIFIER
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Form {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub object_graph_route_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub title: Classified<String>,                 // data_class: PII_QUASI_IDENTIFIER
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub fields: Classified<Vec<FormField>>,        // data_class: PII_QUASI_IDENTIFIER
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormFieldCreate {
    pub field_id: String,            // data_class: INTERNAL_ONLY
    pub label: String,               // data_class: PII_QUASI_IDENTIFIER
    pub kind: FormFieldKind,         // data_class: INTERNAL_ONLY
    pub required: bool,              // data_class: INTERNAL_ONLY
    pub choice_options: Vec<String>, // data_class: PII_QUASI_IDENTIFIER
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormField {
    pub field_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub label: Classified<String>,       // data_class: PII_QUASI_IDENTIFIER
    pub kind: Classified<FormFieldKind>, // data_class: INTERNAL_ONLY
    pub required: Classified<bool>,      // data_class: INTERNAL_ONLY
    pub choice_options: Classified<Vec<String>>, // data_class: PII_QUASI_IDENTIFIER
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormSubmissionCreate {
    pub submission_id: String,                // data_class: INTERNAL_ONLY
    pub form_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub submitter_ref: String,                // data_class: PII_IDENTIFYING
    pub answers: Vec<FormAnswer>,             // data_class: PII_IDENTIFYING
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub submitted_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormSubmission {
    pub submission_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub form_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub submitter_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub answers: Classified<Vec<FormAnswer>>, // data_class: PII_IDENTIFYING
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub submitted_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormAnswerCreate {
    pub field_id: String,          // data_class: INTERNAL_ONLY
    pub value_kind: FormFieldKind, // data_class: INTERNAL_ONLY
    pub value: String,             // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormAnswer {
    pub field_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub value_kind: Classified<FormFieldKind>, // data_class: INTERNAL_ONLY
    pub value: Classified<String>,    // data_class: PII_IDENTIFYING
}

pub trait FormSubmissionReader {
    fn submissions_for_form(
        &self,
        tenant_id: &str,
        form_id: &str,
    ) -> Result<Vec<FormSubmission>, FormError>;
}

impl Form {
    pub fn new(input: FormCreate) -> Result<Self, FormError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_form_data_class());
        validate_non_empty(&input.id, FormError::InvalidFormId)?;
        validate_non_empty(&input.tenant_id, FormError::InvalidTenantId)?;
        validate_non_empty(&input.region, FormError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, FormError::InvalidCellId)?;
        validate_non_empty(
            &input.object_graph_route_id,
            FormError::InvalidObjectGraphRouteId,
        )?;
        validate_text(&input.title, FormError::InvalidTitle)?;
        validate_fields(&input.fields)?;

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            object_graph_route_id: internal(input.object_graph_route_id),
            title: Classified::new(input.title, form_label_data_class()),
            data_class: internal(data_class),
            fields: Classified::new(input.fields, form_label_data_class()),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: internal(FORM_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl FormField {
    pub fn new(input: FormFieldCreate) -> Result<Self, FormError> {
        validate_non_empty(&input.field_id, FormError::InvalidFieldId)?;
        validate_text(&input.label, FormError::InvalidFieldLabel)?;
        validate_choice_shape(input.kind, &input.choice_options)?;
        Ok(Self {
            field_id: internal(input.field_id),
            label: Classified::new(input.label, form_label_data_class()),
            kind: internal(input.kind),
            required: internal(input.required),
            choice_options: Classified::new(input.choice_options, form_label_data_class()),
        })
    }
}

impl FormSubmission {
    pub fn new(input: FormSubmissionCreate, form: &Form) -> Result<Self, FormError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_form_data_class());
        validate_non_empty(&input.submission_id, FormError::InvalidSubmissionId)?;
        validate_non_empty(&input.form_id, FormError::InvalidFormId)?;
        validate_non_empty(&input.tenant_id, FormError::InvalidTenantId)?;
        validate_non_empty(&input.submitter_ref, FormError::InvalidSubmitterRef)?;
        if input.form_id != form.id.value {
            return Err(FormError::SubmissionFormMismatch);
        }
        if input.tenant_id != form.tenant_id.value {
            return Err(FormError::SubmissionTenantMismatch);
        }
        validate_answers(&input.answers, &form.fields.value)?;

        Ok(Self {
            submission_id: internal(input.submission_id),
            form_id: internal(input.form_id),
            tenant_id: internal(input.tenant_id),
            submitter_ref: Classified::new(input.submitter_ref, submitter_data_class()),
            answers: Classified::new(input.answers, form_response_data_class()),
            data_class: internal(data_class),
            submitted_at_epoch_seconds: internal(input.submitted_at_epoch_seconds),
            schema_version: internal(FORM_SUBMISSION_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl FormAnswer {
    pub fn new(input: FormAnswerCreate) -> Result<Self, FormError> {
        validate_non_empty(&input.field_id, FormError::InvalidFieldId)?;
        validate_answer_value(input.value_kind, &input.value)?;
        Ok(Self {
            field_id: internal(input.field_id),
            value_kind: internal(input.value_kind),
            value: Classified::new(input.value, form_response_data_class()),
        })
    }
}

pub fn default_workspace_form_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn form_label_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn form_response_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn submitter_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_form_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, FormError> {
    PrivacyDataClass::new(data_class).map_err(|_| FormError::InvalidDataClass)
}

fn validate_fields(fields: &[FormField]) -> Result<(), FormError> {
    if fields.is_empty() {
        return Err(FormError::EmptyFieldSet);
    }
    let mut field_ids = BTreeSet::new();
    for field in fields {
        validate_non_empty(&field.field_id.value, FormError::InvalidFieldId)?;
        validate_text(&field.label.value, FormError::InvalidFieldLabel)?;
        validate_choice_shape(field.kind.value, &field.choice_options.value)?;
        if !field_ids.insert(field.field_id.value.clone()) {
            return Err(FormError::DuplicateFieldId);
        }
    }
    Ok(())
}

fn validate_choice_shape(kind: FormFieldKind, options: &[String]) -> Result<(), FormError> {
    let is_choice = matches!(
        kind,
        FormFieldKind::SingleChoice | FormFieldKind::MultiChoice
    );
    if is_choice && options.is_empty() {
        return Err(FormError::MissingChoiceOptions);
    }
    if !is_choice && !options.is_empty() {
        return Err(FormError::UnexpectedChoiceOptions);
    }
    for option in options {
        validate_text(option, FormError::InvalidChoiceOption)?;
    }
    Ok(())
}

fn validate_answers(answers: &[FormAnswer], fields: &[FormField]) -> Result<(), FormError> {
    let mut answer_field_ids = BTreeSet::new();
    for answer in answers {
        validate_non_empty(&answer.field_id.value, FormError::InvalidFieldId)?;
        validate_answer_value(answer.value_kind.value, &answer.value.value)?;
        if !answer_field_ids.insert(answer.field_id.value.clone()) {
            return Err(FormError::DuplicateAnswerField);
        }
        let Some(field) = fields
            .iter()
            .find(|field| field.field_id.value == answer.field_id.value)
        else {
            return Err(FormError::UnknownAnswerField);
        };
        if answer.value_kind.value != field.kind.value {
            return Err(FormError::AnswerKindMismatch);
        }
    }
    for field in fields.iter().filter(|field| field.required.value) {
        if !answer_field_ids.contains(&field.field_id.value) {
            return Err(FormError::MissingRequiredAnswer);
        }
    }
    Ok(())
}

fn validate_answer_value(kind: FormFieldKind, value: &str) -> Result<(), FormError> {
    validate_text(value, FormError::InvalidAnswerValue)?;
    match kind {
        FormFieldKind::Number => value
            .parse::<f64>()
            .map(|_| ())
            .map_err(|_| FormError::InvalidAnswerValue),
        FormFieldKind::Boolean if matches!(value, "true" | "false") => Ok(()),
        FormFieldKind::Boolean => Err(FormError::InvalidAnswerValue),
        _ => Ok(()),
    }
}

fn validate_text(value: &str, error: FormError) -> Result<(), FormError> {
    if value.trim() != value || value.is_empty() || value.chars().any(char::is_control) {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: FormError) -> Result<(), FormError> {
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
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn field(field_id: &str, kind: FormFieldKind, required: bool) -> FormField {
        FormField::new(FormFieldCreate {
            field_id: field_id.into(),
            label: format!("Question {field_id}"),
            kind,
            required,
            choice_options: if matches!(
                kind,
                FormFieldKind::SingleChoice | FormFieldKind::MultiChoice
            ) {
                vec!["Yes".into(), "No".into()]
            } else {
                Vec::new()
            },
        })
        .unwrap()
    }

    fn form() -> Form {
        Form::new(FormCreate {
            id: "form-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            object_graph_route_id: "og-route-forms".into(),
            title: "Intake".into(),
            data_class: None,
            fields: vec![
                field("field-1", FormFieldKind::ShortText, true),
                field("field-2", FormFieldKind::Number, false),
            ],
            created_at_epoch_seconds: 1_700_000_000,
        })
        .unwrap()
    }

    fn answer(field_id: &str, kind: FormFieldKind, value: &str) -> FormAnswer {
        FormAnswer::new(FormAnswerCreate {
            field_id: field_id.into(),
            value_kind: kind,
            value: value.into(),
        })
        .unwrap()
    }

    fn submission_input() -> FormSubmissionCreate {
        FormSubmissionCreate {
            submission_id: "submission-1".into(),
            form_id: "form-1".into(),
            tenant_id: "tenant-1".into(),
            submitter_ref: "user:submitter@example.com".into(),
            answers: vec![answer("field-1", FormFieldKind::ShortText, "hello")],
            data_class: None,
            submitted_at_epoch_seconds: 1_700_000_010,
        }
    }

    #[test]
    fn form_defaults_to_identifying_and_classifies_schema_labels() {
        let form = form();

        assert_eq!(
            form.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            form.title.data_class,
            DataClassification::Privacy(form_label_data_class())
        );
        assert_eq!(form.schema_version.value, 1);
    }

    #[test]
    fn field_schema_rejects_duplicate_ids_and_bad_choice_options() {
        assert_eq!(
            FormField::new(FormFieldCreate {
                field_id: "choice".into(),
                label: "Choice".into(),
                kind: FormFieldKind::SingleChoice,
                required: true,
                choice_options: Vec::new(),
            }),
            Err(FormError::MissingChoiceOptions)
        );

        assert_eq!(
            Form::new(FormCreate {
                id: "form-2".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                cell_id: "cell-a".into(),
                object_graph_route_id: "og-route-forms".into(),
                title: "Duplicate".into(),
                data_class: None,
                fields: vec![
                    field("field-1", FormFieldKind::ShortText, true),
                    field("field-1", FormFieldKind::Number, false)
                ],
                created_at_epoch_seconds: 1_700_000_000,
            }),
            Err(FormError::DuplicateFieldId)
        );
    }

    #[test]
    fn submission_requires_required_answers_and_matching_kinds() {
        let form = form();
        let mut missing = submission_input();
        missing.answers = Vec::new();
        assert_eq!(
            FormSubmission::new(missing, &form),
            Err(FormError::MissingRequiredAnswer)
        );

        let mut mismatch = submission_input();
        mismatch.answers = vec![answer("field-1", FormFieldKind::Number, "12")];
        assert_eq!(
            FormSubmission::new(mismatch, &form),
            Err(FormError::AnswerKindMismatch)
        );
    }

    #[test]
    fn submission_rejects_duplicate_answers_and_wrong_tenant() {
        let form = form();
        let submission = FormSubmission::new(submission_input(), &form).unwrap();
        assert_eq!(
            submission.answers.data_class,
            DataClassification::Privacy(form_response_data_class())
        );

        let mut duplicate = submission_input();
        duplicate.answers = vec![
            answer("field-1", FormFieldKind::ShortText, "hello"),
            answer("field-1", FormFieldKind::ShortText, "again"),
        ];
        assert_eq!(
            FormSubmission::new(duplicate, &form),
            Err(FormError::DuplicateAnswerField)
        );

        let mut wrong_tenant = submission_input();
        wrong_tenant.tenant_id = "tenant-2".into();
        assert_eq!(
            FormSubmission::new(wrong_tenant, &form),
            Err(FormError::SubmissionTenantMismatch)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_form_data_class_from_legacy(DataClass::Audit),
            Err(FormError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.forms STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormsSurfaceStaging {
    pub form_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub response_count: Classified<u64>, // data_class: INTERNAL_ONLY
}

impl FormsSurfaceStaging {
    pub fn new(form_id: String, tenant_id: String, response_count: u64) -> Self {
        Self {
            form_id: Classified::new(form_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            response_count: Classified::new(response_count, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> FormsSurfaceStaging {
        FormsSurfaceStaging::new("forms-1".into(), "forms-1".into(), 0u64)
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.form_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
