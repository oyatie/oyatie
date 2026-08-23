//! Conservative CEL helpers used at source-compatibility seams.
//!
//! Upstream Talos validates storage disk selectors with
//! `cel.ParseBooleanExpression(..., celenv.DiskLocator())`: the expression must
//! parse as CEL and type-check to `bool` in an environment containing the
//! `disk` resource object, `system_disk`, unit constants, and `glob(pattern,
//! value)`. operating-system does not yet embed a full CEL runtime, so this module
//! implements a deliberately conservative parser/evaluator for the
//! disk-selector and volume-locator subsets used by Talos examples and
//! documentation. Unsupported CEL syntax fails closed instead of being accepted
//! as source-valid.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{Error, Result};

/// Runtime inputs for evaluating a Talos `celenv.DiskLocator` expression.
///
/// Field names mirror the CEL environment exposed by Talos. Fields that are not
/// known by the current discovery layer should be supplied as empty strings (or
/// `false`) rather than guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskLocator<'a> {
    pub size: u64,
    pub io_size: u64,
    pub sector_size: u64,
    pub readonly: bool,
    pub cdrom: bool,
    pub rotational: bool,
    pub dev_path: &'a str,
    pub pretty_size: &'a str,
    pub model: &'a str,
    pub serial: &'a str,
    pub wwid: &'a str,
    pub bus_path: &'a str,
    pub sub_system: &'a str,
    pub transport: &'a str,
    pub name: &'a str,
    pub disk_type: &'a str,
    pub uuid: &'a str,
    pub modalias: &'a str,
    pub symlinks: &'a [&'a str],
    pub system_disk: bool,
}

/// Validate a Talos `celenv.DiskLocator` boolean expression.
///
/// This is a parser/type-checking boundary. It accepts the source-shaped
/// selector subset currently evaluated by operating-system: boolean operators,
/// comparisons over disk fields, unit-multiplier arithmetic, `glob(pattern,
/// value)`, string prefix/suffix/contains calls, and membership in
/// `disk.symlinks`.
pub fn validate_disk_locator_bool_expression(expr: &str) -> Result<()> {
    let expr = Parser::new(expr).parse_checked("disk selector")?;
    type_of(&expr, CelEnv::DiskLocator).and_then(|ty| {
        if ty == ExprType::Bool {
            Ok(())
        } else {
            Err(Error::invalid(
                "disk selector CEL expression must evaluate to bool",
            ))
        }
    })
}

/// Validate a Talos `celenv.VolumeLocator` boolean expression.
///
/// Talos exposes both a discovered `volume` object and its parent `disk` object
/// in this environment. This validator accepts the source-shaped field subset
/// currently used by operating-system controller projection tests and fails closed for
/// unknown fields/functions.
pub fn validate_volume_locator_bool_expression(expr: &str) -> Result<()> {
    let expr = Parser::new(expr).parse_checked("volume selector")?;
    type_of(&expr, CelEnv::VolumeLocator).and_then(|ty| {
        if ty == ExprType::Bool {
            Ok(())
        } else {
            Err(Error::invalid(
                "volume selector CEL expression must evaluate to bool",
            ))
        }
    })
}

/// Evaluate a Talos `celenv.DiskLocator` boolean expression.
///
/// The expression is parsed and type-checked before evaluation. Unsupported CEL
/// syntax or arithmetic overflow returns an error so callers can fail closed.
pub fn evaluate_disk_locator_bool_expression(expr: &str, disk: &DiskLocator<'_>) -> Result<bool> {
    let expr = Parser::new(expr).parse_checked("disk selector")?;
    let ty = type_of(&expr, CelEnv::DiskLocator)?;
    if ty != ExprType::Bool {
        return Err(Error::invalid(
            "disk selector CEL expression must evaluate to bool",
        ));
    }
    match eval(&expr, disk)? {
        Value::Bool(value) => Ok(value),
        _ => Err(Error::invalid(
            "disk selector CEL expression must evaluate to bool",
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprType {
    Bool,
    String,
    Int,
    Uint,
    StringList,
    DiskObject,
    VolumeObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CelEnv {
    DiskLocator,
    VolumeLocator,
}

impl ExprType {
    fn is_integer(self) -> bool {
        matches!(self, ExprType::Int | ExprType::Uint)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr<'a> {
    Bool(bool),
    String(String),
    Int(i64),
    Uint(u64),
    Identifier(&'a str),
    Member(Box<Expr<'a>>, &'a str),
    Unary(UnaryOp, Box<Expr<'a>>),
    Binary(BinaryOp, Box<Expr<'a>>, Box<Expr<'a>>),
    Function(&'a str, Vec<Expr<'a>>),
    Method(Box<Expr<'a>>, &'a str, Vec<Expr<'a>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinaryOp {
    Or,
    And,
    In,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse_checked(mut self, description: &str) -> Result<Expr<'a>> {
        if self.input.trim().is_empty() {
            return Err(self.invalid(format!("{description} must be a non-empty CEL expression")));
        }
        let expr = self.parse_or()?;
        self.consume_ws();
        if !self.eof() {
            return Err(self.invalid("unexpected trailing CEL input"));
        }
        Ok(expr)
    }

    fn parse_or(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_and()?;
        while self.consume("||") {
            let rhs = self.parse_and()?;
            expr = Expr::Binary(BinaryOp::Or, Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_relation()?;
        while self.consume("&&") {
            let rhs = self.parse_relation()?;
            expr = Expr::Binary(BinaryOp::And, Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_relation(&mut self) -> Result<Expr<'a>> {
        let lhs = self.parse_add()?;

        if self.consume_keyword("in") {
            let rhs = self.parse_add()?;
            return Ok(Expr::Binary(BinaryOp::In, Box::new(lhs), Box::new(rhs)));
        }

        let Some(op) = self.consume_comparison() else {
            return Ok(lhs);
        };
        let rhs = self.parse_add()?;
        Ok(Expr::Binary(op, Box::new(lhs), Box::new(rhs)))
    }

    fn parse_add(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_mul()?;
        loop {
            let op = if self.consume("+") {
                Some(BinaryOp::Add)
            } else if self.consume("-") {
                Some(BinaryOp::Sub)
            } else {
                None
            };
            let Some(op) = op else {
                return Ok(expr);
            };
            let rhs = self.parse_mul()?;
            expr = Expr::Binary(op, Box::new(expr), Box::new(rhs));
        }
    }

    fn parse_mul(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.consume("*") {
                Some(BinaryOp::Mul)
            } else if self.consume("/") {
                Some(BinaryOp::Div)
            } else if self.consume("%") {
                Some(BinaryOp::Rem)
            } else {
                None
            };
            let Some(op) = op else {
                return Ok(expr);
            };
            let rhs = self.parse_unary()?;
            expr = Expr::Binary(op, Box::new(expr), Box::new(rhs));
        }
    }

    fn parse_unary(&mut self) -> Result<Expr<'a>> {
        if self.consume("!") {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)));
        }
        if self.consume("-") {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr<'a>> {
        self.consume_ws();
        if self.consume("(") {
            let expr = self.parse_or()?;
            if !self.consume(")") {
                return Err(self.invalid("expected ')'"));
            }
            return Ok(expr);
        }
        if self.peek_quote().is_some() {
            return Ok(Expr::String(self.parse_string()?));
        }
        if self.peek_ascii_digit() {
            return self.parse_number();
        }
        let ident = self.parse_identifier()?;
        self.parse_identifier_expression(ident)
    }

    fn parse_identifier_expression(&mut self, ident: &'a str) -> Result<Expr<'a>> {
        self.consume_ws();
        let mut expr = if self.consume("(") {
            Expr::Function(ident, self.parse_args()?)
        } else {
            match ident {
                "true" => Expr::Bool(true),
                "false" => Expr::Bool(false),
                _ => Expr::Identifier(ident),
            }
        };

        loop {
            self.consume_ws();
            if !self.consume(".") {
                return Ok(expr);
            }
            let member = self.parse_identifier()?;
            self.consume_ws();
            if self.consume("(") {
                expr = Expr::Method(Box::new(expr), member, self.parse_args()?);
            } else {
                expr = Expr::Member(Box::new(expr), member);
            }
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr<'a>>> {
        let mut args = Vec::new();
        if self.consume(")") {
            return Ok(args);
        }
        loop {
            args.push(self.parse_or()?);
            if self.consume(")") {
                return Ok(args);
            }
            if !self.consume(",") {
                return Err(self.invalid("expected ',' or ')' in argument list"));
            }
        }
    }

    fn parse_number(&mut self) -> Result<Expr<'a>> {
        self.consume_ws();
        let start = self.pos;
        while self.peek_ascii_digit() {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(self.invalid("expected CEL integer literal"));
        }
        let literal = &self.input[start..self.pos];
        if self.consume("u") {
            let value = literal
                .parse::<u64>()
                .map_err(|_| self.invalid("CEL unsigned integer literal is out of range"))?;
            return Ok(Expr::Uint(value));
        }
        let value = literal
            .parse::<i64>()
            .map_err(|_| self.invalid("CEL signed integer literal is out of range"))?;
        Ok(Expr::Int(value))
    }

    fn parse_string(&mut self) -> Result<String> {
        self.consume_ws();
        let Some(quote) = self.peek_quote() else {
            return Err(self.invalid("expected CEL string literal"));
        };
        self.pos += quote.len_utf8();
        let mut out = String::new();
        while let Some(ch) = self.peek_char() {
            self.pos += ch.len_utf8();
            if ch == '\\' {
                let Some(escaped) = self.peek_char() else {
                    return Err(self.invalid("unterminated CEL string escape"));
                };
                self.pos += escaped.len_utf8();
                out.push(escaped);
                continue;
            }
            if ch == quote {
                return Ok(out);
            }
            out.push(ch);
        }
        Err(self.invalid("unterminated CEL string literal"))
    }

    fn parse_identifier(&mut self) -> Result<&'a str> {
        self.consume_ws();
        let start = self.pos;
        let Some(first) = self.peek_char() else {
            return Err(self.invalid("expected CEL identifier"));
        };
        if !(first == '_' || first.is_ascii_alphabetic()) {
            return Err(self.invalid("expected CEL identifier"));
        }
        self.pos += first.len_utf8();
        while let Some(ch) = self.peek_char() {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
        Ok(&self.input[start..self.pos])
    }

    fn consume_comparison(&mut self) -> Option<BinaryOp> {
        for (token, op) in [
            ("==", BinaryOp::Eq),
            ("!=", BinaryOp::Ne),
            ("<=", BinaryOp::Le),
            (">=", BinaryOp::Ge),
            ("<", BinaryOp::Lt),
            (">", BinaryOp::Gt),
        ] {
            if self.consume(token) {
                return Some(op);
            }
        }
        None
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.consume_ws();
        if !self.input[self.pos..].starts_with(keyword) {
            return false;
        }
        let after = self.pos + keyword.len();
        if self.input[after..]
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        {
            return false;
        }
        self.pos = after;
        true
    }

    fn consume(&mut self, token: &str) -> bool {
        self.consume_ws();
        if self.input[self.pos..].starts_with(token) {
            self.pos += token.len();
            true
        } else {
            false
        }
    }

    fn consume_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_quote(&self) -> Option<char> {
        match self.peek_char() {
            Some(ch @ ('\'' | '"')) => Some(ch),
            _ => None,
        }
    }

    fn peek_ascii_digit(&self) -> bool {
        self.peek_char().is_some_and(|ch| ch.is_ascii_digit())
    }

    fn eof(&self) -> bool {
        self.pos == self.input.len()
    }

    fn invalid(&self, message: impl Into<String>) -> Error {
        Error::invalid(alloc::format!(
            "{} at byte {} in {:?}",
            message.into(),
            self.pos,
            self.input
        ))
    }
}

fn type_of(expr: &Expr<'_>, env: CelEnv) -> Result<ExprType> {
    match expr {
        Expr::Bool(_) => Ok(ExprType::Bool),
        Expr::String(_) => Ok(ExprType::String),
        Expr::Int(_) => Ok(ExprType::Int),
        Expr::Uint(_) => Ok(ExprType::Uint),
        Expr::Identifier(name) => identifier_type(name, env),
        Expr::Member(receiver, member) => member_type(type_of(receiver, env)?, member),
        Expr::Unary(UnaryOp::Not, inner) => {
            expect_type(
                type_of(inner, env)?,
                ExprType::Bool,
                "operand of ! must be bool",
            )?;
            Ok(ExprType::Bool)
        }
        Expr::Unary(UnaryOp::Neg, inner) => {
            expect_type(
                type_of(inner, env)?,
                ExprType::Int,
                "operand of unary - must be signed integer",
            )?;
            Ok(ExprType::Int)
        }
        Expr::Binary(op, lhs, rhs) => binary_type(*op, type_of(lhs, env)?, type_of(rhs, env)?),
        Expr::Function(name, args) => function_type(name, args, env),
        Expr::Method(receiver, name, args) => method_type(type_of(receiver, env)?, name, args, env),
    }
}

fn identifier_type(ident: &str, env: CelEnv) -> Result<ExprType> {
    match ident {
        "system_disk" if env == CelEnv::DiskLocator => Ok(ExprType::Bool),
        "disk" => Ok(ExprType::DiskObject),
        "volume" if env == CelEnv::VolumeLocator => Ok(ExprType::VolumeObject),
        "KiB" | "MiB" | "GiB" | "TiB" | "PiB" | "EiB" | "kB" | "MB" | "GB" | "TB" | "PB" | "EB" => {
            Ok(ExprType::Uint)
        }
        _ => Err(Error::invalid("unknown identifier in CEL environment")),
    }
}

fn member_type(receiver: ExprType, member: &str) -> Result<ExprType> {
    match receiver {
        ExprType::DiskObject => match member {
            "size" | "io_size" | "sector_size" => Ok(ExprType::Uint),
            "readonly" | "cdrom" | "rotational" => Ok(ExprType::Bool),
            "dev_path" | "pretty_size" | "model" | "serial" | "wwid" | "bus_path"
            | "sub_system" | "transport" | "name" | "type" | "uuid" | "modalias" => {
                Ok(ExprType::String)
            }
            "symlinks" => Ok(ExprType::StringList),
            _ => Err(Error::invalid("unknown disk field in CEL environment")),
        },
        ExprType::VolumeObject => match member {
            "size"
            | "sector_size"
            | "io_size"
            | "block_size"
            | "filesystem_block_size"
            | "probed_size"
            | "partition_index"
            | "offset" => Ok(ExprType::Uint),
            "name" | "uuid" | "label" | "partition_uuid" | "partition_type" | "partition_label"
            | "type" | "device_path" | "parent" | "dev_path" | "parent_dev_path"
            | "pretty_size" => Ok(ExprType::String),
            _ => Err(Error::invalid("unknown volume field in CEL environment")),
        },
        _ => Err(Error::invalid(
            "CEL member access is only supported on disk or volume",
        )),
    }
}

fn binary_type(op: BinaryOp, lhs: ExprType, rhs: ExprType) -> Result<ExprType> {
    match op {
        BinaryOp::Or | BinaryOp::And => {
            expect_type(
                lhs,
                ExprType::Bool,
                "left side of boolean operator must be bool",
            )?;
            expect_type(
                rhs,
                ExprType::Bool,
                "right side of boolean operator must be bool",
            )?;
            Ok(ExprType::Bool)
        }
        BinaryOp::In => {
            expect_type(lhs, ExprType::String, "left side of in must be string")?;
            expect_type(
                rhs,
                ExprType::StringList,
                "right side of in must be a string list",
            )?;
            Ok(ExprType::Bool)
        }
        BinaryOp::Eq | BinaryOp::Ne => {
            if lhs != rhs {
                return Err(Error::invalid(
                    "CEL equality operands must have the same type",
                ));
            }
            if matches!(lhs, ExprType::DiskObject | ExprType::StringList) {
                return Err(Error::invalid(
                    "CEL equality operand type is not comparable",
                ));
            }
            Ok(ExprType::Bool)
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            if lhs != rhs || !(lhs.is_integer() || lhs == ExprType::String) {
                return Err(Error::invalid(
                    "CEL ordering operands must both be strings or same-width integers",
                ));
            }
            Ok(ExprType::Bool)
        }
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
            if lhs == rhs && lhs.is_integer() {
                Ok(lhs)
            } else {
                Err(Error::invalid(
                    "CEL arithmetic operands must be same-width integers",
                ))
            }
        }
    }
}

fn function_type(name: &str, args: &[Expr<'_>], env: CelEnv) -> Result<ExprType> {
    match name {
        "glob" if env == CelEnv::DiskLocator => {
            if args.len() != 2 {
                return Err(Error::invalid("glob expects exactly two arguments"));
            }
            expect_type(
                type_of(&args[0], env)?,
                ExprType::String,
                "glob pattern must be string",
            )?;
            expect_type(
                type_of(&args[1], env)?,
                ExprType::String,
                "glob value must be string",
            )?;
            Ok(ExprType::Bool)
        }
        _ => Err(Error::invalid("unsupported CEL function in environment")),
    }
}

fn method_type(receiver: ExprType, name: &str, args: &[Expr<'_>], env: CelEnv) -> Result<ExprType> {
    match (receiver, name) {
        (ExprType::String, "startsWith" | "endsWith" | "contains") => {
            if args.len() != 1 {
                return Err(Error::invalid("string method expects exactly one argument"));
            }
            expect_type(
                type_of(&args[0], env)?,
                ExprType::String,
                "string method argument must be string",
            )?;
            Ok(ExprType::Bool)
        }
        _ => Err(Error::invalid("unsupported CEL method in disk selector")),
    }
}

fn expect_type(actual: ExprType, expected: ExprType, message: &str) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(Error::invalid(message))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Bool(bool),
    String(String),
    Int(i64),
    Uint(u64),
    StringList(Vec<String>),
    DiskObject,
}

fn eval(expr: &Expr<'_>, disk: &DiskLocator<'_>) -> Result<Value> {
    match expr {
        Expr::Bool(value) => Ok(Value::Bool(*value)),
        Expr::String(value) => Ok(Value::String(value.clone())),
        Expr::Int(value) => Ok(Value::Int(*value)),
        Expr::Uint(value) => Ok(Value::Uint(*value)),
        Expr::Identifier(name) => eval_identifier(name, disk),
        Expr::Member(receiver, member) => match eval(receiver, disk)? {
            Value::DiskObject => eval_disk_member(disk, member),
            _ => Err(Error::invalid(
                "CEL member access is only supported on disk",
            )),
        },
        Expr::Unary(op, inner) => eval_unary(*op, eval(inner, disk)?),
        Expr::Binary(op, lhs, rhs) => eval_binary(*op, eval(lhs, disk)?, eval(rhs, disk)?),
        Expr::Function(name, args) => eval_function(name, args, disk),
        Expr::Method(receiver, name, args) => eval_method(eval(receiver, disk)?, name, args, disk),
    }
}

fn eval_identifier(name: &str, disk: &DiskLocator<'_>) -> Result<Value> {
    match name {
        "system_disk" => Ok(Value::Bool(disk.system_disk)),
        "disk" => Ok(Value::DiskObject),
        "KiB" => Ok(Value::Uint(1024)),
        "MiB" => Ok(Value::Uint(1024u64.pow(2))),
        "GiB" => Ok(Value::Uint(1024u64.pow(3))),
        "TiB" => Ok(Value::Uint(1024u64.pow(4))),
        "PiB" => Ok(Value::Uint(1024u64.pow(5))),
        "EiB" => Ok(Value::Uint(1024u64.pow(6))),
        "kB" => Ok(Value::Uint(1000)),
        "MB" => Ok(Value::Uint(1000u64.pow(2))),
        "GB" => Ok(Value::Uint(1000u64.pow(3))),
        "TB" => Ok(Value::Uint(1000u64.pow(4))),
        "PB" => Ok(Value::Uint(1000u64.pow(5))),
        "EB" => Ok(Value::Uint(1000u64.pow(6))),
        _ => Err(Error::invalid(
            "unknown identifier in disk-selector CEL environment",
        )),
    }
}

fn eval_disk_member(disk: &DiskLocator<'_>, member: &str) -> Result<Value> {
    match member {
        "size" => Ok(Value::Uint(disk.size)),
        "io_size" => Ok(Value::Uint(disk.io_size)),
        "sector_size" => Ok(Value::Uint(disk.sector_size)),
        "readonly" => Ok(Value::Bool(disk.readonly)),
        "cdrom" => Ok(Value::Bool(disk.cdrom)),
        "rotational" => Ok(Value::Bool(disk.rotational)),
        "dev_path" => Ok(Value::String(String::from(disk.dev_path))),
        "pretty_size" => Ok(Value::String(String::from(disk.pretty_size))),
        "model" => Ok(Value::String(String::from(disk.model))),
        "serial" => Ok(Value::String(String::from(disk.serial))),
        "wwid" => Ok(Value::String(String::from(disk.wwid))),
        "bus_path" => Ok(Value::String(String::from(disk.bus_path))),
        "sub_system" => Ok(Value::String(String::from(disk.sub_system))),
        "transport" => Ok(Value::String(String::from(disk.transport))),
        "name" => Ok(Value::String(String::from(disk.name))),
        "type" => Ok(Value::String(String::from(disk.disk_type))),
        "uuid" => Ok(Value::String(String::from(disk.uuid))),
        "modalias" => Ok(Value::String(String::from(disk.modalias))),
        "symlinks" => Ok(Value::StringList(
            disk.symlinks
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        )),
        _ => Err(Error::invalid(
            "unknown disk field in disk-selector CEL environment",
        )),
    }
}

fn eval_unary(op: UnaryOp, value: Value) -> Result<Value> {
    match (op, value) {
        (UnaryOp::Not, Value::Bool(value)) => Ok(Value::Bool(!value)),
        (UnaryOp::Neg, Value::Int(value)) => value
            .checked_neg()
            .map(Value::Int)
            .ok_or_else(|| Error::invalid("CEL signed integer negation overflow")),
        (UnaryOp::Not, _) => Err(Error::invalid("operand of ! must be bool")),
        (UnaryOp::Neg, _) => Err(Error::invalid("operand of unary - must be signed integer")),
    }
}

fn eval_binary(op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
    match op {
        BinaryOp::Or => Ok(Value::Bool(as_bool(lhs)? || as_bool(rhs)?)),
        BinaryOp::And => Ok(Value::Bool(as_bool(lhs)? && as_bool(rhs)?)),
        BinaryOp::In => match (lhs, rhs) {
            (Value::String(needle), Value::StringList(values)) => {
                Ok(Value::Bool(values.iter().any(|value| value == &needle)))
            }
            _ => Err(Error::invalid("invalid in operands")),
        },
        BinaryOp::Eq => Ok(Value::Bool(lhs == rhs)),
        BinaryOp::Ne => Ok(Value::Bool(lhs != rhs)),
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => compare_values(op, lhs, rhs),
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
            eval_arithmetic(op, lhs, rhs)
        }
    }
}

fn compare_values(op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
    let result = match (lhs, rhs) {
        (Value::Int(lhs), Value::Int(rhs)) => compare_ord(op, lhs.cmp(&rhs)),
        (Value::Uint(lhs), Value::Uint(rhs)) => compare_ord(op, lhs.cmp(&rhs)),
        (Value::String(lhs), Value::String(rhs)) => compare_ord(op, lhs.cmp(&rhs)),
        _ => {
            return Err(Error::invalid(
                "CEL ordering operands have incompatible types",
            ));
        }
    };
    Ok(Value::Bool(result))
}

fn compare_ord(op: BinaryOp, ordering: core::cmp::Ordering) -> bool {
    match op {
        BinaryOp::Lt => ordering.is_lt(),
        BinaryOp::Le => ordering.is_le(),
        BinaryOp::Gt => ordering.is_gt(),
        BinaryOp::Ge => ordering.is_ge(),
        _ => unreachable!(),
    }
}

fn eval_arithmetic(op: BinaryOp, lhs: Value, rhs: Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(lhs), Value::Int(rhs)) => checked_i64(op, lhs, rhs).map(Value::Int),
        (Value::Uint(lhs), Value::Uint(rhs)) => checked_u64(op, lhs, rhs).map(Value::Uint),
        _ => Err(Error::invalid(
            "CEL arithmetic operands must be same-width integers",
        )),
    }
}

fn checked_i64(op: BinaryOp, lhs: i64, rhs: i64) -> Result<i64> {
    let value = match op {
        BinaryOp::Add => lhs.checked_add(rhs),
        BinaryOp::Sub => lhs.checked_sub(rhs),
        BinaryOp::Mul => lhs.checked_mul(rhs),
        BinaryOp::Div => lhs.checked_div(rhs),
        BinaryOp::Rem => lhs.checked_rem(rhs),
        _ => unreachable!(),
    };
    value
        .ok_or_else(|| Error::invalid("CEL signed integer arithmetic overflow or division by zero"))
}

fn checked_u64(op: BinaryOp, lhs: u64, rhs: u64) -> Result<u64> {
    let value = match op {
        BinaryOp::Add => lhs.checked_add(rhs),
        BinaryOp::Sub => lhs.checked_sub(rhs),
        BinaryOp::Mul => lhs.checked_mul(rhs),
        BinaryOp::Div => lhs.checked_div(rhs),
        BinaryOp::Rem => lhs.checked_rem(rhs),
        _ => unreachable!(),
    };
    value.ok_or_else(|| {
        Error::invalid("CEL unsigned integer arithmetic overflow or division by zero")
    })
}

fn eval_function(name: &str, args: &[Expr<'_>], disk: &DiskLocator<'_>) -> Result<Value> {
    match name {
        "glob" => {
            if args.len() != 2 {
                return Err(Error::invalid("glob expects exactly two arguments"));
            }
            let pattern = as_string(eval(&args[0], disk)?)?;
            let value = as_string(eval(&args[1], disk)?)?;
            Ok(Value::Bool(glob_matches(&pattern, &value)))
        }
        _ => Err(Error::invalid("unsupported CEL function in disk selector")),
    }
}

fn eval_method(
    receiver: Value,
    name: &str,
    args: &[Expr<'_>],
    disk: &DiskLocator<'_>,
) -> Result<Value> {
    let receiver = as_string(receiver)?;
    if args.len() != 1 {
        return Err(Error::invalid("string method expects exactly one argument"));
    }
    let arg = as_string(eval(&args[0], disk)?)?;
    let result = match name {
        "startsWith" => receiver.starts_with(&arg),
        "endsWith" => receiver.ends_with(&arg),
        "contains" => receiver.contains(&arg),
        _ => return Err(Error::invalid("unsupported CEL method in disk selector")),
    };
    Ok(Value::Bool(result))
}

fn as_bool(value: Value) -> Result<bool> {
    match value {
        Value::Bool(value) => Ok(value),
        _ => Err(Error::invalid("value is not bool")),
    }
}

fn as_string(value: Value) -> Result<String> {
    match value {
        Value::String(value) => Ok(value),
        _ => Err(Error::invalid("value is not string")),
    }
}

fn glob_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut pi = 0usize;
    let mut vi = 0usize;
    let mut star = None;
    let mut star_value = 0usize;

    while vi < value.len() {
        if pi < pattern.len() && (pattern[pi] == b'?' || pattern[pi] == value[vi]) {
            pi += 1;
            vi += 1;
        } else if pi < pattern.len() && pattern[pi] == b'*' {
            star = Some(pi);
            pi += 1;
            star_value = vi;
        } else if let Some(star_pos) = star {
            pi = star_pos + 1;
            star_value += 1;
            vi = star_value;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == b'*' {
        pi += 1;
    }
    pi == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::{
        DiskLocator, evaluate_disk_locator_bool_expression as evaluate,
        validate_disk_locator_bool_expression as validate,
        validate_volume_locator_bool_expression as validate_volume,
    };

    fn locator(system_disk: bool) -> DiskLocator<'static> {
        const SYMLINKS: &[&str] = &[
            "/dev/disk/by-path/pci-0000:00:1f.2-ata-1",
            "/dev/disk/by-id/nvme-deadbeef",
        ];
        DiskLocator {
            size: 512 * 1000 * 1000 * 1000,
            io_size: 4096,
            sector_size: 512,
            readonly: false,
            cdrom: false,
            rotational: false,
            dev_path: "/dev/nvme0n1",
            pretty_size: "512 GB",
            model: "QEMU NVMe Controller",
            serial: "deadbeef-001",
            wwid: "wwn-0xdeadbeef",
            bus_path: "pci-0000:00:1f.2",
            sub_system: "block",
            transport: "nvme",
            name: "nvme0n1",
            disk_type: "nvme",
            uuid: "",
            modalias: "",
            symlinks: SYMLINKS,
            system_disk,
        }
    }

    #[test]
    fn disk_locator_accepts_source_documented_selectors() {
        for expr in [
            r#"disk.transport == "nvme""#,
            r#"disk.transport == 'scsi' && disk.size < 2u * TiB"#,
            r#"disk.serial.startsWith('deadbeef') && !disk.cdrom"#,
            r#"'/dev/disk/by-path/pci-0000:00:1f.2-ata-1' in disk.symlinks"#,
            r#"disk.size > 120u * GB && disk.size < 1u * TB"#,
            r#"glob("*QEMU*", disk.model) || system_disk"#,
            r#"(disk.transport == "sata" && !disk.rotational) && !system_disk"#,
        ] {
            assert!(validate(expr).is_ok(), "{expr}");
        }
    }

    #[test]
    fn disk_locator_evaluates_source_documented_selectors() {
        for expr in [
            r#"disk.transport == "nvme""#,
            r#"disk.size > 120u * GB && disk.size < 1u * TB"#,
            r#"disk.serial.startsWith('deadbeef') && !disk.cdrom"#,
            r#"disk.model.contains('NVMe')"#,
            r#"glob("*QEMU*", disk.model)"#,
            r#"'/dev/disk/by-id/nvme-deadbeef' in disk.symlinks"#,
            r#"!system_disk"#,
        ] {
            assert!(evaluate(expr, &locator(false)).unwrap(), "{expr}");
        }

        assert!(!evaluate(r#"disk.transport == "scsi""#, &locator(false)).unwrap());
        assert!(!evaluate(r#"!system_disk"#, &locator(true)).unwrap());
    }

    #[test]
    fn disk_locator_rejects_malformed_or_wrong_type_selectors() {
        for expr in [
            "",
            "disk.transport ==",
            r#""not valid CEL""#,
            "disk.size > 10 * GB",
            "unknown == true",
            "disk.unknown == true",
            "disk.transport && true",
            "glob(disk.model)",
            "disk.serial.startsWith(10u)",
            "disk.serial.matches('dead.*')",
            "'x' in disk.transport",
        ] {
            assert!(validate(expr).is_err(), "{expr}");
        }
    }

    #[test]
    fn volume_locator_accepts_source_documented_selectors() {
        for expr in [
            r#"volume.partition_label == "MY-DATA""#,
            r#"volume.name == "xfs" && disk.serial == "SERIAL123""#,
            r#"volume.size > 120u * GB && volume.dev_path.startsWith("/dev/")"#,
        ] {
            assert!(validate_volume(expr).is_ok(), "{expr}");
        }

        for expr in [
            r#"volume.partition_label =="#,
            r#"system_disk"#,
            r#"glob("*DATA*", volume.partition_label)"#,
            r#"volume.unknown == true"#,
        ] {
            assert!(validate_volume(expr).is_err(), "{expr}");
        }
    }
}
