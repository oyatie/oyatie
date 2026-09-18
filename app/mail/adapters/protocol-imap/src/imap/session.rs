use super::{
    append, auth, commands::command, condstore, response, retry, retry::Outcome, selected, uidonly,
};
use mail_kernel::Error;
use mail_service::{Budget, MailService};

#[derive(Default)]
pub(super) struct Session {
    pub(super) utf8: bool,
    pub(super) condstore: bool,
    pub(super) qresync: bool,
    pub(super) objectid: bool,
    pub(super) uidonly: bool,
    pub(super) credential: String,
    pub(super) account_id: String,
    pub(super) selected: Option<selected::Selection>,
}
/// What the listener knows about the connection; never client input.
#[derive(Clone, Copy)]
pub(super) struct Transport {
    pub(super) encrypted: bool,
    pub(super) starttls: bool,
}

impl Session {
    pub(super) fn respond(
        &mut self,
        service: &MailService,
        parts: &[String],
        transport: Transport,
        append: Option<append::Append>,
        budget: &Budget,
        output: &mut response::Output,
    ) -> Outcome {
        let Transport {
            encrypted,
            starttls,
        } = transport;
        output.utf8 = self.utf8;
        output.condstore = self.condstore;
        output.qresync = self.qresync;
        output.objectid = self.objectid;
        output.uidonly = self.uidonly;
        let done = Outcome::Done { logout: false };
        if let Some(append) = append {
            return match append.respond(service, self, parts, budget, output) {
                Ok(()) => done,
                Err(append) => Outcome::Busy(Some(append)),
            };
        }
        let tag = &parts[0];
        let verb = parts[1].to_ascii_uppercase();
        if matches!(
            verb.as_str(),
            "CAPABILITY"
                | "LOGOUT"
                | "NOOP"
                | "CHECK"
                | "CLOSE"
                | "EXPUNGE"
                | "UNAUTHENTICATE"
                | "NAMESPACE"
        ) && parts.len() != 2
        {
            output
                .extend_from_slice(format!("{tag} BAD Command takes no arguments\r\n").as_bytes());
            return done;
        }
        let mut status = "OK";
        let mut code = None;
        match verb.as_str() {
            "CAPABILITY" => uidonly::capabilities(self, encrypted, starttls, output),
            "ID" => uidonly::id(output),
            "UNAUTHENTICATE" => {
                if let Err(kind) = uidonly::unauthenticate(self, output) {
                    status = kind;
                }
            }
            "ENABLE" => {
                if parts.len() < 3 || self.credential.is_empty() {
                    status = "BAD";
                } else {
                    match service.read(&self.credential, &self.account_id) {
                        Err(Error::Busy) => return Outcome::Busy(None),
                        Err(_) => status = "NO",
                        Ok(_) => {
                            if let Err(kind) = condstore::enable(self, parts, output) {
                                status = kind;
                            }
                        }
                    }
                }
            }
            "LOGOUT" => {
                output.extend_from_slice(
                    format!("* BYE Closing\r\n{tag} OK LOGOUT completed\r\n").as_bytes(),
                );
                return Outcome::Done { logout: true };
            }
            "LOGIN" => {
                if !encrypted || !self.credential.is_empty() || parts.len() != 4 {
                    status = "NO";
                } else {
                    match auth::login(service, &parts[2], &parts[3]) {
                        Ok(account) => {
                            self.credential.clone_from(&parts[3]);
                            self.account_id = account;
                        }
                        Err(_) => status = "NO",
                    }
                }
            }
            "NOOP" if self.credential.is_empty() => {}
            _ => match service.read(&self.credential, &self.account_id) {
                Err(Error::Busy) => return Outcome::Busy(None),
                Err(_) => status = "NO",
                Ok(account) => {
                    selected::synchronize(&account, parts, &mut self.selected, output);
                    let result = command(
                        service,
                        &self.credential,
                        &account,
                        parts,
                        &mut self.selected,
                        budget,
                        output,
                    );
                    match result {
                        Ok(completion) => code = completion,
                        Err(kind) => status = kind,
                    }
                }
            },
        }

        self.condstore = output.condstore;
        self.objectid = output.objectid;
        if status == retry::BUSY {
            return Outcome::Busy(None);
        }
        let mode = if status == "OK" && matches!(verb.as_str(), "SELECT" | "EXAMINE") {
            if self.selected.as_ref().is_some_and(|s| s.readonly) {
                " [READ-ONLY]"
            } else {
                " [READ-WRITE]"
            }
        } else {
            ""
        };
        let code = code.map(|s| format!(" {s}")).unwrap_or_default();
        output.extend_from_slice(
            format!("{tag} {status}{code}{mode} {verb} completed\r\n").as_bytes(),
        );
        done
    }
}
