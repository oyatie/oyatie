use crate::SubmitEmailError;
use mail_kernel::{Error, SubmissionEnvelope};

pub const MAX_DELAYED_SEND: u64 = u32::MAX as u64;

pub(super) fn release(envelope: &SubmissionEnvelope) -> Result<i64, SubmitEmailError> {
    use SubmitEmailError::InvalidEnvelope;
    let now = mail_api::Clock.now_secs();
    let parameters = &envelope.mail_from.parameters;
    if parameters
        .keys()
        .any(|k| !matches!(k.as_str(), "HOLDFOR" | "HOLDUNTIL"))
        || envelope.rcpt_to.iter().any(|r| !r.parameters.is_empty())
    {
        return Err(InvalidEnvelope);
    }
    let send_at = match (parameters.get("HOLDFOR"), parameters.get("HOLDUNTIL")) {
        (Some(Some(seconds)), None) => {
            if seconds.is_empty()
                || seconds.len() > 10
                || !seconds.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(InvalidEnvelope);
            }
            let seconds: i64 = seconds.parse().map_err(|_| InvalidEnvelope)?;
            now.checked_add(seconds).ok_or(InvalidEnvelope)?
        }
        (None, Some(Some(date))) => {
            let parsed = mail_parser::DateTime::parse_rfc3339(date)
                .filter(|d| d.is_valid())
                .ok_or(InvalidEnvelope)?;
            let timestamp = parsed.to_timestamp();
            if date.len() != 20
                || !date.ends_with('Z')
                || mail_parser::DateTime::from_timestamp(timestamp).to_rfc3339() != *date
            {
                return Err(InvalidEnvelope);
            }
            timestamp.max(now)
        }
        (None, None) => now,
        _ => return Err(InvalidEnvelope),
    };
    if send_at > now.saturating_add(MAX_DELAYED_SEND as i64) {
        return Err(InvalidEnvelope);
    }
    Ok(send_at)
}
