use super::record;
// Predicates mirror tests/src/utils/imap.rs. Failures are recorded rather
// than panicking so every reachable upstream check executes; the harness
// fails afterwards when any recorded predicate is false.
pub trait AssertResult: Sized {
    fn assert_contains(self, text: &str) -> Self;
    fn assert_not_contains(self, text: &str) -> Self;
    fn assert_count(self, text: &str, count: usize) -> Self;
    fn assert_response_code(self, text: &str) -> Self;
    fn assert_folders<'x>(
        self,
        expected: impl IntoIterator<Item = (&'x str, impl IntoIterator<Item = &'x str>)>,
        match_all: bool,
    ) -> Self;
    fn into_response_code(self) -> String;
    fn into_append_uid(self) -> String;
}
impl AssertResult for Vec<String> {
    fn assert_contains(self, text: &str) -> Self {
        record(
            &self,
            "contains",
            text,
            self.iter().any(|line| line.contains(text)),
        );
        self
    }
    fn assert_not_contains(self, text: &str) -> Self {
        record(
            &self,
            "not_contains",
            text,
            !self.iter().any(|line| line.contains(text)),
        );
        self
    }
    fn assert_count(self, text: &str, count: usize) -> Self {
        record(
            &self,
            "count",
            &format!("{count} lines containing {text}"),
            self.iter().filter(|line| line.contains(text)).count() == count,
        );
        self
    }
    fn assert_response_code(self, text: &str) -> Self {
        record(
            &self,
            "response_code",
            text,
            self.last().unwrap().contains(&format!("[{text}]")),
        );
        self
    }
    fn assert_folders<'x>(
        self,
        expected: impl IntoIterator<Item = (&'x str, impl IntoIterator<Item = &'x str>)>,
        match_all: bool,
    ) -> Self {
        let mut names = Vec::new();
        let mut passed = true;
        for (mailbox, flags) in expected {
            names.push(mailbox.to_owned());
            let quoted = format!("\"{mailbox}\"");
            match self.iter().find(|line| line.contains(&quoted)) {
                Some(line) => {
                    for flag in flags {
                        if !flag.is_empty() && !line.contains(flag) {
                            passed = false;
                        }
                    }
                }
                None => passed = false,
            }
        }
        if match_all && names.len() != self.len() - 1 {
            passed = false;
        }
        record(
            &self,
            if match_all {
                "folders_exact"
            } else {
                "folders"
            },
            &names.join(", "),
            passed,
        );
        self
    }
    fn into_response_code(self) -> String {
        if let Some((_, code)) = self.last().unwrap().split_once('[')
            && let Some((code, _)) = code.split_once(']')
        {
            return code.to_owned();
        }
        panic!("No response code found in {:?}", self);
    }
    fn into_append_uid(self) -> String {
        if let Some((_, code)) = self.last().unwrap().split_once("[APPENDUID ")
            && let Some((code, _)) = code.split_once(']')
            && let Some((_, uid)) = code.split_once(' ')
        {
            return uid.to_owned();
        }
        panic!("No APPENDUID found in {:?}", self.last().unwrap());
    }
}
