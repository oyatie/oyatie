//! What a constant's VALUE is, and how the target spells it.
//!
//! Split from `items.rs` because that file decides which ITEM a declaration becomes and this decides
//! what goes on the right of the `=`. The questions here are all about the value: whether the
//! author's derivation survives the source's folding, whether the type it is declared at has to be
//! constructed rather than coerced, and whether its content is text at all.

use port_engine_api::Declaration;
use port_engine_rust_ir::{RustExpr, RustType};

use crate::resolve::Resolver;
use crate::vocabulary::{ATTR_VALUE, SOURCE_STRING};

/// The author's DERIVATION where the target can spell it, and the folded value where it cannot.
///
/// The source folds a constant expression before the engine ever sees it, so `marshaledSize =
/// len(magic) + 8*5 + 32` arrives as `76`. That value is correct — it is the same constant — and it
/// throws away what the author wrote and why. Two reviewers reading real ported packages named a
/// bare folded literal as evidence that a translator had evaluated an expression a person would
/// have kept, and they were right: `76` is a magic number and `MAGIC.len() + 8 * 5 + 32` is a
/// derivation that stays correct when the layout changes.
///
/// The fallback is SAFE in a way a body's would not be, and that is what makes preferring the
/// expression reasonable rather than reckless: both spellings are the same constant, proven so by
/// the source's own evaluator. Where the expression names something the target cannot say — a
/// concatenation of two constant strings, which the target has no operator for — the value is not a
/// degraded answer, it is the same answer written differently.
pub(crate) fn authored_value(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    ty: &RustType,
) -> Option<RustExpr> {
    let authored = declaration.children.first()?;
    let body = crate::body::Body {
        newtype_parameters: std::collections::BTreeSet::new(),
        owner: &declaration.name,
        resolver,
        fallible: false,
        borrowed: std::collections::BTreeSet::new(),
        result_is_owned_string: false,
        results: crate::returns::ResultFacts::none(),
        usize_counters: std::collections::BTreeSet::new(),
        walked: None,
        receiver_type: None,
    };
    // A LITERAL derivation is the folded value already, and rendering it again only risks spelling
    // it differently for no gain.
    if authored.kind == crate::vocabulary::KIND_LITERAL {
        return None;
    }
    // NUMERIC only, and that is the whole of the condition. The derivation is preferred because the
    // target can spell the source's arithmetic — `len(magic) + 8*5 + 32` reads as itself — and the
    // target has no `+` on strings at all, so `"abc" + num` parses, type-checks nowhere, and is a
    // crate that does not build. The folded value is the same constant either way, so the string
    // case loses nothing but the author's spelling.
    if !matches!(
        ty.spelling().as_str(),
        "usize" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "isize"
    ) {
        return None;
    }
    // A LENGTH constant is the target's index type, and the mapped length call adds a conversion to
    // the source's integer. Stripping it is the same thing a guard comparing against a length does,
    // through the same function, so the declaration and every comparison agree.
    let rendered = match ty.spelling() == "usize" {
        true => crate::counters::unsigned_bound(authored, &body),
        false => crate::body_expr::expression(authored, &body),
    };
    // A derivation the engine cannot translate is not a loss: the folded value is the SAME constant,
    // proven so by the source's own evaluator. That is what makes preferring the expression
    // reasonable rather than reckless — unlike a body, there is no wrong answer to fall back to.
    rendered.ok()
}

/// Whether this declaration's type is one the unit DEFINES, and so constructs rather than coerces.
///
/// Local and emitted, both required: a type from the pack's table is a target type the literal
/// already is, and a type that refused is not in the crate to construct. A length constant is
/// excluded because its type was overridden to the index type above, which takes the literal
/// directly.
pub(crate) fn constructs_at_type(declaration: &Declaration, resolver: &Resolver<'_>) -> bool {
    let type_ref = &declaration.type_ref;
    type_ref.kind == "named"
        && !resolver.scope.length_constants.contains(&declaration.name)
        && resolver.is_local(type_ref)
        && resolver.scope.types.contains_key(&type_ref.name)
}

/// A string constant whose content is NOT text, as the target's byte array.
///
/// The source's string is a byte string — it holds arbitrary bytes and is the idiomatic bag of them
/// — and the target's is guaranteed UTF-8, so the usual mapping is right only for the ones that are
/// text. A four-byte framing prefix is not.
///
/// RECOGNISED, not guessed: a byte the source had to write as an escape because it cannot be typed
/// is exactly the evidence that the value is data. The common whitespace escapes are text and stay
/// text.
///
/// The BYTE COUNT is the array's length and is counted on the decoded bytes rather than on the
/// spelling, because an escape is one byte written as four characters.
pub(crate) fn binary_string(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Option<(RustType, String)> {
    let rule = resolver.binary_string;
    if rule.target_type.is_empty() || declaration.type_ref.name != SOURCE_STRING {
        return None;
    }
    let spelling = declaration.attr(ATTR_VALUE)?;
    let bytes = decoded_bytes(spelling.strip_prefix('"')?.strip_suffix('"')?)?;
    if bytes.iter().all(|byte| is_text(*byte)) {
        return None;
    }
    Some((
        RustType::path(rule.target_type.replace("{0}", &bytes.len().to_string())),
        rule.literal_form.replace("{0}", spelling),
    ))
}

/// Whether this byte is one a reader would call text.
///
/// Printable ASCII, plus the three whitespace escapes that appear in ordinary messages. Everything
/// else is a byte somebody chose for its value rather than for its glyph.
const fn is_text(byte: u8) -> bool {
    byte.is_ascii_graphic() || matches!(byte, b' ' | b'\n' | b'\t' | b'\r')
}

/// The BYTES a source string literal's inner text stands for.
///
/// Only the escapes both languages spell identically, and `None` for anything else — a value the
/// engine cannot decode is one it cannot count, and guessing the length of a byte array is guessing
/// the wire format.
fn decoded_bytes(inner: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            let mut buffer = [0_u8; 4];
            out.extend_from_slice(ch.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        match chars.next()? {
            'n' => out.push(b'\n'),
            't' => out.push(b'\t'),
            'r' => out.push(b'\r'),
            '\\' => out.push(b'\\'),
            '"' => out.push(b'"'),
            '0' => out.push(0),
            'x' => {
                let hex: String = [chars.next()?, chars.next()?].into_iter().collect();
                out.push(u8::from_str_radix(&hex, 16).ok()?);
            }
            _ => return None,
        }
    }
    Some(out)
}

/// An integer constant re-spelled as the BIT PATTERN it is.
///
/// A count and a mask wear the same syntax and are read completely differently. `122192928000000000`
/// is a number of ticks and means what it says; `11400714785074694791` is a multiplier whose value
/// is its bits, and in decimal no reviewer can check it against the specification that defines it —
/// the one that reviewed this output said so, and had to run a script to verify five constants.
///
/// The TYPE decides, because magnitude cannot. Measured over the corpus: seven constants exceed the
/// 32-bit line and two of them — the ticks between 1582 and the epoch — are counts. What separates
/// them is that the counts are typed at the source's counting integer and the patterns at a
/// fixed-width UNSIGNED one, which is what an author reaches for when the bits are the point.
///
/// Zero-padded to the type's width, so a mask reads as the machine word it is and two of them line
/// up under each other.
pub(crate) fn bit_pattern(declaration: &Declaration, resolver: &Resolver<'_>) -> Option<String> {
    let rule = resolver.bit_pattern_constants;
    let width = *rule.widths.get(&declaration.type_ref.name)?;
    let spelling = declaration.attr(ATTR_VALUE)?;
    // DECIMAL only. A source that already wrote hex said what it meant, and re-spelling a value the
    // engine did not parse is how a constant silently changes.
    if !spelling.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let value: u128 = spelling.parse().ok()?;
    if value < rule.min_value {
        return None;
    }
    Some(format!("0x{value:0>width$X}", width = (width / 4) as usize))
}
