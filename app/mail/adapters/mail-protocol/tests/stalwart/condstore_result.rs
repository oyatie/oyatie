use super::record;
// Predicates and value extraction match tests/src/utils/imap.rs. A recorded
// predicate failure remains fatal after all reachable upstream checks execute.
pub trait AssertResult: Sized {
    fn assert_contains(self, text: &str) -> Self;
    fn assert_not_contains(self, text: &str) -> Self;
    fn assert_count(self, text: &str, count: usize) -> Self;
    fn assert_response_code(self, text: &str) -> Self;
    fn into_response_code(self) -> String;
    fn into_append_uid(self) -> String;
    fn into_highest_modseq(self) -> String;
    fn into_modseq(self) -> String;
    fn into_uid_validity(self) -> String;
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
    fn into_highest_modseq(self) -> String {
        delimited(self, "HIGHESTMODSEQ ", &[']', ')'])
    }
    fn into_modseq(self) -> String {
        delimited(self, "MODSEQ (", &[')'])
    }
    fn into_uid_validity(self) -> String {
        delimited(self, "UIDVALIDITY ", &[']', ')'])
    }
}
fn delimited(lines: Vec<String>, key: &str, delimiters: &[char]) -> String {
    for line in &lines {
        if let Some((_, value)) = line.split_once(key) {
            for delimiter in delimiters {
                if let Some((value, _)) = value.split_once(*delimiter) {
                    return value.to_owned();
                }
            }
            panic!("No {key} delimiter found in {line:?}");
        }
    }
    panic!("No {key} entries found in {:?}", lines);
}
