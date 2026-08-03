//! Workspace meet kernel.
//!
//! Typed kernel records for the W-Workspace-Stable Meet surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns session,
//! participant, recording, and transcript metadata with KMS-shred and
//! trust-portal-only recording access invariants. WebRTC, SFU routing,
//! transcription engines, and archive storage remain adapter concerns.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const MEET_SESSION_SCHEMA_VERSION: u32 = 1;
const TRANSCRIPT_CHUNK_SCHEMA_VERSION: u32 = 1;
const MIN_COMPLETED_RECORDING_BYTES: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeetError {
    InvalidSessionId,
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidSfuPoolId,
    EmptyParticipantSet,
    MissingHostParticipant,
    InvalidParticipantRef,
    InvalidParticipantDisplayName,
    InvalidParticipantTimeOrder,
    InvalidSessionTimeOrder,
    InvalidRecordingId,
    InvalidRecordingStorageKey,
    InvalidKmsShredKeyId,
    InvalidRetentionPolicyId,
    EmptyCompletedRecording,
    InvalidRecordingTimeOrder,
    MissingRecordingConsent,
    InvalidTranscriptSessionId,
    InvalidSummaryId,
    InvalidTranscriptText,
    InvalidTranscriptTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ParticipantRole {
    Host,
    CoHost,
    Presenter,
    Attendee,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ParticipantConnectionState {
    Invited,
    Joined,
    Left,
    Removed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecordingStatus {
    Requested,
    Recording,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecordingConsentMode {
    NotRequested,
    ParticipantOptIn,
    TenantPolicyDefaultOn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecordingAccessMode {
    TrustPortalOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeetSessionCreate {
    pub id: String,                              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub region: String,                          // data_class: INTERNAL_ONLY
    pub cell_id: String,                         // data_class: INTERNAL_ONLY
    pub sfu_pool_id: String,                     // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>,    // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub ended_at_epoch_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
    pub participants: Vec<ParticipantRef>,       // data_class: PII_IDENTIFYING
    pub recording: Option<RecordingRef>,         // data_class: PII_IDENTIFYING
    pub recording_consent: RecordingConsentMode, // data_class: INTERNAL_ONLY
    pub transcript_session_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub summary_id: Option<String>,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeetSession {
    pub id: Classified<String>,                    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub region: Classified<String>,                // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,               // data_class: INTERNAL_ONLY
    pub sfu_pool_id: Classified<String>,           // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub ended_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub participants: Classified<Vec<ParticipantRef>>, // data_class: PII_IDENTIFYING
    pub recording: Classified<Option<RecordingRef>>, // data_class: PII_IDENTIFYING
    pub recording_consent: Classified<RecordingConsentMode>, // data_class: INTERNAL_ONLY
    pub transcript_session_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub summary_id: Classified<Option<String>>,    // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantRef {
    pub actor_ref: Classified<String>, // data_class: PII_IDENTIFYING
    pub display_name: Classified<Option<String>>, // data_class: PII_QUASI_IDENTIFIER
    pub role: Classified<ParticipantRole>, // data_class: INTERNAL_ONLY
    pub connection_state: Classified<ParticipantConnectionState>, // data_class: INTERNAL_ONLY
    pub joined_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub left_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingRefCreate {
    pub recording_id: String,                // data_class: INTERNAL_ONLY
    pub archive_storage_key: String,         // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: String,            // data_class: INTERNAL_ONLY
    pub retention_policy_id: String,         // data_class: INTERNAL_ONLY
    pub status: RecordingStatus,             // data_class: INTERNAL_ONLY
    pub byte_len: u64,                       // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub ended_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingRef {
    pub recording_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub archive_storage_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub retention_policy_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub status: Classified<RecordingStatus>, // data_class: INTERNAL_ONLY
    pub access_mode: Classified<RecordingAccessMode>, // data_class: INTERNAL_ONLY
    pub byte_len: Classified<u64>,        // data_class: INTERNAL_ONLY
    pub started_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub ended_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptChunkCreate {
    pub transcript_session_id: String, // data_class: INTERNAL_ONLY
    pub session_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub speaker_ref: String,           // data_class: PII_IDENTIFYING
    pub text: String,                  // data_class: PII_IDENTIFYING
    pub started_at_epoch_millis: u64,  // data_class: INTERNAL_ONLY
    pub ended_at_epoch_millis: u64,    // data_class: INTERNAL_ONLY
    pub data_class: Option<PrivacyDataClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptChunk {
    pub transcript_session_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub session_id: Classified<String>,            // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,             // data_class: INTERNAL_ONLY
    pub speaker_ref: Classified<String>,           // data_class: PII_IDENTIFYING
    pub text: Classified<String>,                  // data_class: PII_IDENTIFYING
    pub started_at_epoch_millis: Classified<u64>,  // data_class: INTERNAL_ONLY
    pub ended_at_epoch_millis: Classified<u64>,    // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>,  // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,           // data_class: INTERNAL_ONLY
}

pub trait TranscriptStream {
    fn transcript_chunks(
        &self,
        tenant_id: &str,
        session_id: &str,
        transcript_session_id: &str,
    ) -> Result<Vec<TranscriptChunk>, MeetError>;
}

impl MeetSession {
    pub fn new(input: MeetSessionCreate) -> Result<Self, MeetError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_meet_data_class());
        validate_non_empty(&input.id, MeetError::InvalidSessionId)?;
        validate_non_empty(&input.tenant_id, MeetError::InvalidTenantId)?;
        validate_non_empty(&input.region, MeetError::InvalidRegion)?;
        validate_non_empty(&input.cell_id, MeetError::InvalidCellId)?;
        validate_non_empty(&input.sfu_pool_id, MeetError::InvalidSfuPoolId)?;
        validate_optional_non_empty(
            input.transcript_session_id.as_deref(),
            MeetError::InvalidTranscriptSessionId,
        )?;
        validate_optional_non_empty(input.summary_id.as_deref(), MeetError::InvalidSummaryId)?;
        validate_session_time(input.started_at_epoch_seconds, input.ended_at_epoch_seconds)?;
        validate_participants(&input.participants)?;
        validate_recording(&input.recording, input.recording_consent)?;

        Ok(Self {
            id: internal(input.id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            cell_id: internal(input.cell_id),
            sfu_pool_id: internal(input.sfu_pool_id),
            data_class: internal(data_class),
            started_at_epoch_seconds: internal(input.started_at_epoch_seconds),
            ended_at_epoch_seconds: internal(input.ended_at_epoch_seconds),
            participants: Classified::new(input.participants, participant_data_class()),
            recording: Classified::new(input.recording, recording_data_class()),
            recording_consent: internal(input.recording_consent),
            transcript_session_id: internal(input.transcript_session_id),
            summary_id: internal(input.summary_id),
            schema_version: internal(MEET_SESSION_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

impl ParticipantRef {
    pub fn new(
        actor_ref: String,
        display_name: Option<String>,
        role: ParticipantRole,
        connection_state: ParticipantConnectionState,
        joined_at_epoch_seconds: Option<u64>,
        left_at_epoch_seconds: Option<u64>,
    ) -> Result<Self, MeetError> {
        validate_non_empty(&actor_ref, MeetError::InvalidParticipantRef)?;
        validate_optional_display_name(display_name.as_deref())?;
        if let (Some(joined), Some(left)) = (joined_at_epoch_seconds, left_at_epoch_seconds)
            && left < joined
        {
            return Err(MeetError::InvalidParticipantTimeOrder);
        }
        Ok(Self {
            actor_ref: Classified::new(actor_ref, participant_data_class()),
            display_name: Classified::new(display_name, display_name_data_class()),
            role: internal(role),
            connection_state: internal(connection_state),
            joined_at_epoch_seconds: internal(joined_at_epoch_seconds),
            left_at_epoch_seconds: internal(left_at_epoch_seconds),
        })
    }
}

impl RecordingRef {
    pub fn new(input: RecordingRefCreate) -> Result<Self, MeetError> {
        validate_non_empty(&input.recording_id, MeetError::InvalidRecordingId)?;
        validate_non_empty(
            &input.archive_storage_key,
            MeetError::InvalidRecordingStorageKey,
        )?;
        validate_non_empty(&input.kms_shred_key_id, MeetError::InvalidKmsShredKeyId)?;
        validate_non_empty(
            &input.retention_policy_id,
            MeetError::InvalidRetentionPolicyId,
        )?;
        validate_recording_time(input.started_at_epoch_seconds, input.ended_at_epoch_seconds)?;
        if input.status == RecordingStatus::Completed
            && input.byte_len < MIN_COMPLETED_RECORDING_BYTES
        {
            return Err(MeetError::EmptyCompletedRecording);
        }

        Ok(Self {
            recording_id: internal(input.recording_id),
            archive_storage_key: internal(input.archive_storage_key),
            kms_shred_key_id: internal(input.kms_shred_key_id),
            retention_policy_id: internal(input.retention_policy_id),
            status: internal(input.status),
            access_mode: internal(RecordingAccessMode::TrustPortalOnly),
            byte_len: internal(input.byte_len),
            started_at_epoch_seconds: internal(input.started_at_epoch_seconds),
            ended_at_epoch_seconds: internal(input.ended_at_epoch_seconds),
        })
    }
}

impl TranscriptChunk {
    pub fn new(input: TranscriptChunkCreate) -> Result<Self, MeetError> {
        let data_class = input
            .data_class
            .unwrap_or(default_workspace_meet_data_class());
        validate_non_empty(
            &input.transcript_session_id,
            MeetError::InvalidTranscriptSessionId,
        )?;
        validate_non_empty(&input.session_id, MeetError::InvalidSessionId)?;
        validate_non_empty(&input.tenant_id, MeetError::InvalidTenantId)?;
        validate_non_empty(&input.speaker_ref, MeetError::InvalidParticipantRef)?;
        validate_non_empty(&input.text, MeetError::InvalidTranscriptText)?;
        if input.ended_at_epoch_millis < input.started_at_epoch_millis {
            return Err(MeetError::InvalidTranscriptTimeOrder);
        }

        Ok(Self {
            transcript_session_id: internal(input.transcript_session_id),
            session_id: internal(input.session_id),
            tenant_id: internal(input.tenant_id),
            speaker_ref: Classified::new(input.speaker_ref, participant_data_class()),
            text: Classified::new(input.text, transcript_data_class()),
            started_at_epoch_millis: internal(input.started_at_epoch_millis),
            ended_at_epoch_millis: internal(input.ended_at_epoch_millis),
            data_class: internal(data_class),
            schema_version: internal(TRANSCRIPT_CHUNK_SCHEMA_VERSION),
        })
    }

    pub fn privacy_data_class(&self) -> PrivacyDataClass {
        self.data_class.value
    }
}

pub fn default_workspace_meet_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn participant_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn display_name_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_quasi_identifier()
}

pub fn recording_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn transcript_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_meet_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, MeetError> {
    PrivacyDataClass::new(data_class).map_err(|_| MeetError::InvalidDataClass)
}

fn validate_participants(participants: &[ParticipantRef]) -> Result<(), MeetError> {
    if participants.is_empty() {
        return Err(MeetError::EmptyParticipantSet);
    }
    if !participants
        .iter()
        .any(|participant| participant.role.value == ParticipantRole::Host)
    {
        return Err(MeetError::MissingHostParticipant);
    }
    for participant in participants {
        validate_non_empty(
            &participant.actor_ref.value,
            MeetError::InvalidParticipantRef,
        )?;
        validate_optional_display_name(participant.display_name.value.as_deref())?;
        if let (Some(joined), Some(left)) = (
            participant.joined_at_epoch_seconds.value,
            participant.left_at_epoch_seconds.value,
        ) && left < joined
        {
            return Err(MeetError::InvalidParticipantTimeOrder);
        }
    }
    Ok(())
}

fn validate_recording(
    recording: &Option<RecordingRef>,
    consent: RecordingConsentMode,
) -> Result<(), MeetError> {
    if recording.is_some() && consent == RecordingConsentMode::NotRequested {
        return Err(MeetError::MissingRecordingConsent);
    }
    Ok(())
}

fn validate_session_time(started_at: u64, ended_at: Option<u64>) -> Result<(), MeetError> {
    if ended_at.is_some_and(|ended_at| ended_at < started_at) {
        Err(MeetError::InvalidSessionTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_recording_time(started_at: u64, ended_at: Option<u64>) -> Result<(), MeetError> {
    if ended_at.is_some_and(|ended_at| ended_at < started_at) {
        Err(MeetError::InvalidRecordingTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_optional_display_name(display_name: Option<&str>) -> Result<(), MeetError> {
    let Some(display_name) = display_name else {
        return Ok(());
    };
    if display_name.trim() != display_name
        || display_name.is_empty()
        || display_name.chars().any(char::is_control)
    {
        Err(MeetError::InvalidParticipantDisplayName)
    } else {
        Ok(())
    }
}

fn validate_optional_non_empty(value: Option<&str>, error: MeetError) -> Result<(), MeetError> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_non_empty(value, error)
}

fn validate_non_empty(value: &str, error: MeetError) -> Result<(), MeetError> {
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

    fn host() -> ParticipantRef {
        ParticipantRef::new(
            "user:host@example.com".into(),
            Some("Host User".into()),
            ParticipantRole::Host,
            ParticipantConnectionState::Joined,
            Some(1_700_000_000),
            None,
        )
        .unwrap()
    }

    fn attendee() -> ParticipantRef {
        ParticipantRef::new(
            "user:attendee@example.com".into(),
            Some("Attendee User".into()),
            ParticipantRole::Attendee,
            ParticipantConnectionState::Left,
            Some(1_700_000_010),
            Some(1_700_000_050),
        )
        .unwrap()
    }

    fn recording() -> RecordingRef {
        RecordingRef::new(RecordingRefCreate {
            recording_id: "recording-1".into(),
            archive_storage_key: "tenant-1/meet/session-1/recording.webm".into(),
            kms_shred_key_id: "kms-key-1".into(),
            retention_policy_id: "retention-90d".into(),
            status: RecordingStatus::Completed,
            byte_len: 1024,
            started_at_epoch_seconds: 1_700_000_000,
            ended_at_epoch_seconds: Some(1_700_000_060),
        })
        .unwrap()
    }

    fn session_input() -> MeetSessionCreate {
        MeetSessionCreate {
            id: "session-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            cell_id: "cell-a".into(),
            sfu_pool_id: "sfu-pool-1".into(),
            data_class: None,
            started_at_epoch_seconds: 1_700_000_000,
            ended_at_epoch_seconds: Some(1_700_000_060),
            participants: vec![host(), attendee()],
            recording: Some(recording()),
            recording_consent: RecordingConsentMode::ParticipantOptIn,
            transcript_session_id: Some("transcript-1".into()),
            summary_id: Some("summary-1".into()),
        }
    }

    #[test]
    fn session_defaults_to_identifying_and_requires_host() {
        let session = MeetSession::new(session_input()).unwrap();

        assert_eq!(
            session.privacy_data_class().data_class(),
            DataClass::PiiIdentifying
        );
        assert_eq!(
            session.participants.data_class,
            DataClassification::Privacy(participant_data_class())
        );
        assert_eq!(session.schema_version.value, 1);

        let mut invalid = session_input();
        invalid.participants = vec![attendee()];
        assert_eq!(
            MeetSession::new(invalid),
            Err(MeetError::MissingHostParticipant)
        );
    }

    #[test]
    fn recording_requires_kms_retention_consent_and_completed_bytes() {
        assert_eq!(
            RecordingRef::new(RecordingRefCreate {
                recording_id: "recording-1".into(),
                archive_storage_key: "tenant-1/meet/session-1/recording.webm".into(),
                kms_shred_key_id: "kms-key-1".into(),
                retention_policy_id: "retention-90d".into(),
                status: RecordingStatus::Completed,
                byte_len: 0,
                started_at_epoch_seconds: 1_700_000_000,
                ended_at_epoch_seconds: Some(1_700_000_060),
            }),
            Err(MeetError::EmptyCompletedRecording)
        );

        let mut invalid = session_input();
        invalid.recording_consent = RecordingConsentMode::NotRequested;
        assert_eq!(
            MeetSession::new(invalid),
            Err(MeetError::MissingRecordingConsent)
        );

        let recording = recording();
        assert_eq!(
            recording.access_mode.value,
            RecordingAccessMode::TrustPortalOnly
        );
    }

    #[test]
    fn transcript_chunks_are_identifying_and_time_ordered() {
        let chunk = TranscriptChunk::new(TranscriptChunkCreate {
            transcript_session_id: "transcript-1".into(),
            session_id: "session-1".into(),
            tenant_id: "tenant-1".into(),
            speaker_ref: "user:host@example.com".into(),
            text: "hello team".into(),
            started_at_epoch_millis: 1_700_000_000_000,
            ended_at_epoch_millis: 1_700_000_001_000,
            data_class: None,
        })
        .unwrap();

        assert_eq!(
            chunk.text.data_class,
            DataClassification::Privacy(transcript_data_class())
        );
        assert_eq!(chunk.schema_version.value, 1);

        assert_eq!(
            TranscriptChunk::new(TranscriptChunkCreate {
                ended_at_epoch_millis: 1,
                started_at_epoch_millis: 2,
                ..TranscriptChunkCreate {
                    transcript_session_id: "transcript-1".into(),
                    session_id: "session-1".into(),
                    tenant_id: "tenant-1".into(),
                    speaker_ref: "user:host@example.com".into(),
                    text: "hello team".into(),
                    started_at_epoch_millis: 1,
                    ended_at_epoch_millis: 2,
                    data_class: None,
                }
            }),
            Err(MeetError::InvalidTranscriptTimeOrder)
        );
    }

    #[test]
    fn session_and_participant_time_order_are_validated() {
        let mut invalid = session_input();
        invalid.ended_at_epoch_seconds = Some(invalid.started_at_epoch_seconds - 1);
        assert_eq!(
            MeetSession::new(invalid),
            Err(MeetError::InvalidSessionTimeOrder)
        );

        assert_eq!(
            ParticipantRef::new(
                "user:late@example.com".into(),
                None,
                ParticipantRole::Attendee,
                ParticipantConnectionState::Left,
                Some(10),
                Some(9),
            ),
            Err(MeetError::InvalidParticipantTimeOrder)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_meet_data_class_from_legacy(DataClass::Audit),
            Err(MeetError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}

// ---------------------------------------------------------------------------
// M03-P06-IP — workspace.meet STAGING surface markers (SPEC §4 rows).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeetSurfaceStaging {
    pub session_id: Classified<String>,    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub sfu_placement: Classified<String>, // data_class: INTERNAL_ONLY
}

impl MeetSurfaceStaging {
    pub fn new(session_id: String, tenant_id: String, sfu_placement: String) -> Self {
        Self {
            session_id: Classified::new(session_id, DataClass::InternalOnly),
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            sfu_placement: Classified::new(sfu_placement, DataClass::InternalOnly),
        }
    }
}

#[cfg(test)]
mod m03_p06_tests {
    use super::*;

    fn sample() -> MeetSurfaceStaging {
        MeetSurfaceStaging::new("meet-1".into(), "meet-1".into(), "meet-1".into())
    }

    #[test]
    fn surface_staging_constructor_sets_internal_only() {
        let s = sample();
        assert_eq!(s.session_id.data_class, DataClass::InternalOnly.into());
    }

    #[test]
    fn surface_staging_round_trip_equality() {
        assert_eq!(sample(), sample());
    }
}
