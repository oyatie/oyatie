use super::*;

/// Apply a SCIM filter expression to a list of users. Returns the filtered
/// list. Supports the practical RFC 7644 §3.4.2.2 subset used by Okta /
/// Entra / Workspace clients in the wild.
pub(super) fn apply_user_filter(
    users: &[User],
    filter: Option<&str>,
) -> Result<Vec<User>, ScimError> {
    let Some(f) = filter else {
        return Ok(users.to_vec());
    };
    let f = f.trim();
    if f.is_empty() {
        return Ok(users.to_vec());
    }
    let expr = parse_filter(f)?;
    Ok(users
        .iter()
        .filter(|u| eval_user(u, &expr))
        .cloned()
        .collect())
}

#[derive(Clone, Debug)]
pub enum FilterExpr {
    Eq(String, String),
    Ne(String, String),
    Co(String, String),
    Sw(String, String),
    Ew(String, String),
    Pr(String),
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
    Not(Box<FilterExpr>),
}

/// Minimal recursive-descent filter parser. Sufficient for the in-the-wild
/// dialects used by the canonical SCIM clients.
pub fn parse_filter(input: &str) -> Result<FilterExpr, ScimError> {
    let mut p = Parser {
        src: input.as_bytes(),
        pos: 0,
    };
    let e = p.parse_or()?;
    p.skip_ws();
    if p.pos < p.src.len() {
        return Err(ScimError::new(
            400,
            Some(ScimType::InvalidFilter),
            format!("unexpected trailing input at byte {}", p.pos),
        ));
    }
    Ok(e)
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}
impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }
    fn peek_keyword(&mut self, kw: &str) -> bool {
        self.skip_ws();
        let bytes = kw.as_bytes();
        if self.src.len() < self.pos + bytes.len() {
            return false;
        }
        let slice = &self.src[self.pos..self.pos + bytes.len()];
        if !slice.eq_ignore_ascii_case(bytes) {
            return false;
        }
        // Must be followed by a delimiter (whitespace, paren, eof).
        let next = self.pos + bytes.len();
        if next == self.src.len() {
            return true;
        }
        let c = self.src[next];
        c.is_ascii_whitespace() || c == b'(' || c == b')'
    }
    fn eat_keyword(&mut self, kw: &str) -> bool {
        if self.peek_keyword(kw) {
            self.pos += kw.len();
            true
        } else {
            false
        }
    }
    fn parse_or(&mut self) -> Result<FilterExpr, ScimError> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws();
            if !self.eat_keyword("or") {
                break;
            }
            let right = self.parse_and()?;
            left = FilterExpr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn parse_and(&mut self) -> Result<FilterExpr, ScimError> {
        let mut left = self.parse_not()?;
        loop {
            self.skip_ws();
            if !self.eat_keyword("and") {
                break;
            }
            let right = self.parse_not()?;
            left = FilterExpr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn parse_not(&mut self) -> Result<FilterExpr, ScimError> {
        self.skip_ws();
        if self.eat_keyword("not") {
            self.skip_ws();
            if self.pos >= self.src.len() || self.src[self.pos] != b'(' {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    "not requires (...)",
                ));
            }
            self.pos += 1;
            let inner = self.parse_or()?;
            self.skip_ws();
            if self.pos >= self.src.len() || self.src[self.pos] != b')' {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    "missing ) after not",
                ));
            }
            self.pos += 1;
            return Ok(FilterExpr::Not(Box::new(inner)));
        }
        self.parse_atom()
    }
    fn parse_atom(&mut self) -> Result<FilterExpr, ScimError> {
        self.skip_ws();
        if self.pos >= self.src.len() {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "unexpected eof",
            ));
        }
        if self.src[self.pos] == b'(' {
            self.pos += 1;
            let e = self.parse_or()?;
            self.skip_ws();
            if self.pos >= self.src.len() || self.src[self.pos] != b')' {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    "missing )",
                ));
            }
            self.pos += 1;
            return Ok(e);
        }
        let attr = self.parse_attr()?;
        self.skip_ws();
        if self.eat_keyword("pr") {
            return Ok(FilterExpr::Pr(attr));
        }
        let op = self.parse_op_word()?;
        self.skip_ws();
        let val = self.parse_string_literal()?;
        Ok(match op.as_str() {
            "eq" => FilterExpr::Eq(attr, val),
            "ne" => FilterExpr::Ne(attr, val),
            "co" => FilterExpr::Co(attr, val),
            "sw" => FilterExpr::Sw(attr, val),
            "ew" => FilterExpr::Ew(attr, val),
            other => {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    format!("unsupported op {other}"),
                ));
            }
        })
    }
    fn parse_attr(&mut self) -> Result<String, ScimError> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c.is_ascii_alphanumeric() || c == b'.' || c == b':' || c == b'_' || c == b'-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "expected attribute name",
            ));
        }
        Ok(std::str::from_utf8(&self.src[start..self.pos])
            .map_err(|_| ScimError::new(400, Some(ScimType::InvalidFilter), "non-utf8 in attr"))?
            .to_owned())
    }
    fn parse_op_word(&mut self) -> Result<String, ScimError> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_alphabetic() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "expected op",
            ));
        }
        Ok(std::str::from_utf8(&self.src[start..self.pos])
            .map_err(|_| ScimError::new(400, Some(ScimType::InvalidFilter), "non-utf8 in op"))?
            .to_ascii_lowercase())
    }
    fn parse_string_literal(&mut self) -> Result<String, ScimError> {
        self.skip_ws();
        if self.pos >= self.src.len() || self.src[self.pos] != b'"' {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "expected \"..\" string literal",
            ));
        }
        self.pos += 1;
        let mut out = Vec::new();
        while self.pos < self.src.len() && self.src[self.pos] != b'"' {
            if self.src[self.pos] == b'\\' && self.pos + 1 < self.src.len() {
                out.push(self.src[self.pos + 1]);
                self.pos += 2;
            } else {
                out.push(self.src[self.pos]);
                self.pos += 1;
            }
        }
        if self.pos >= self.src.len() {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "unterminated string",
            ));
        }
        self.pos += 1;
        String::from_utf8(out)
            .map_err(|_| ScimError::new(400, Some(ScimType::InvalidFilter), "non-utf8 string"))
    }
}
