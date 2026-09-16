use crate::MailService;
use mail_api::{Action, SubmissionAcceptance, SubmissionSelection};
use mail_kernel::{EnvelopeAddress, Error, SubmissionEnvelope, SubmissionRecord, UndoStatus};
use mail_parser::{HeaderName, HeaderValue, MessageParser};
use std::collections::{BTreeMap, BTreeSet};

pub struct SubmitEmail {
    pub identity_id: String,
    pub email_id: String,
    pub envelope: Option<SubmissionEnvelope>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitEmailError {
    Storage(Error),
    IdentityNotFound,
    EmailNotFound,
    NoRecipients,
    InvalidRecipients,
    ForbiddenFrom,
    InvalidEmail,
    InvalidEnvelope,
}
impl From<Error> for SubmitEmailError {
    fn from(error: Error) -> Self {
        Self::Storage(error)
    }
}

impl MailService {
    pub fn submit_email(
        &self,
        token: &str,
        account: &str,
        revision: u64,
        request: SubmitEmail,
    ) -> Result<SubmissionSelection, SubmitEmailError> {
        let owner = self.authorize(token, account, Action::Submit)?;
        self.authorize(token, account, Action::Read)?;
        if request.identity_id != owner.id {
            return Err(SubmitEmailError::IdentityNotFound);
        }
        let selected = self
            .store
            .messages(account, std::slice::from_ref(&request.email_id))?;
        let message = selected
            .messages
            .first()
            .ok_or(SubmitEmailError::EmailNotFound)?;
        let raw = self.store.blob(account, &message.id)?;
        let mut envelope = match request.envelope {
            Some(envelope) => envelope,
            None => infer(&raw, &owner.address)?,
        };
        envelope.mail_from.email = envelope.mail_from.email.trim().to_owned();
        let send_at = super::submission_schedule::release(&envelope)?;
        if !envelope
            .mail_from
            .email
            .eq_ignore_ascii_case(&owner.address)
        {
            return Err(SubmitEmailError::ForbiddenFrom);
        }
        if envelope.rcpt_to.is_empty() {
            return Err(SubmitEmailError::NoRecipients);
        }
        if envelope.rcpt_to.len() > 100 {
            return Err(SubmitEmailError::InvalidRecipients);
        }
        let mut seen = BTreeSet::new();
        for recipient in &mut envelope.rcpt_to {
            recipient.email = recipient.email.trim().to_owned();
            if !mail_kernel::valid_address(&recipient.email) {
                return Err(SubmitEmailError::InvalidRecipients);
            }
            let (local, domain) = recipient
                .email
                .rsplit_once('@')
                .ok_or(SubmitEmailError::InvalidRecipients)?;
            recipient.email = format!("{local}@{}", domain.to_ascii_lowercase());
        }
        envelope.rcpt_to.retain(|r| seen.insert(r.email.clone()));
        let raw =
            super::submission::normalize(&raw, &owner.address).map_err(|error| match error {
                Error::Forbidden => SubmitEmailError::ForbiddenFrom,
                Error::Invalid => SubmitEmailError::InvalidEmail,
                e => SubmitEmailError::Storage(e),
            })?;
        let record = SubmissionRecord {
            id: String::new(),
            identity_id: request.identity_id,
            email_id: message.id.clone(),
            thread_id: message.thread_id().into(),
            envelope,
            send_at,
            undo_status: UndoStatus::Pending,
            delivery_status: BTreeMap::new(),
        };
        self.store
            .accept_submission(
                account,
                revision,
                SubmissionAcceptance {
                    record,
                    email_revision: selected.revision,
                    raw,
                    allow_remote: self.outbound.is_some(),
                },
            )
            .map_err(Into::into)
    }
}

fn infer(raw: &[u8], sender: &str) -> Result<SubmissionEnvelope, SubmitEmailError> {
    let message = MessageParser::default()
        .parse_headers(raw)
        .ok_or(SubmitEmailError::InvalidEmail)?;
    let mut recipients = Vec::new();
    for header in message
        .headers()
        .iter()
        .filter(|h| matches!(h.name, HeaderName::To | HeaderName::Cc | HeaderName::Bcc))
    {
        let HeaderValue::Address(addresses) = &header.value else {
            return Err(SubmitEmailError::InvalidRecipients);
        };
        for address in addresses.iter() {
            if recipients.len() == 100 {
                return Err(SubmitEmailError::InvalidRecipients);
            }
            recipients.push(EnvelopeAddress {
                email: address
                    .address
                    .as_deref()
                    .ok_or(SubmitEmailError::InvalidRecipients)?
                    .into(),
                parameters: BTreeMap::new(),
            });
        }
    }
    Ok(SubmissionEnvelope {
        mail_from: EnvelopeAddress {
            email: sender.into(),
            parameters: BTreeMap::new(),
        },
        rcpt_to: recipients,
    })
}
