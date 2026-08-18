//! # port-engine-transform — plan → `RustIr` construction apply.
//!
//! ADR-0637 D1: the kernel plans; this core face applies rule **construction** / **precondition**
//! data (strings from the pack) into a deterministic [`RustIr`]. Unknown constructions refuse.
//!
//! Two rule shapes, told apart by DATA rather than by a flag:
//!
//! - A rule that captures nothing is **unit-level**: one region per unit. This is the shape the
//!   W0-B canary path uses, and it is unchanged.
//! - A rule that captures one or more declaration kinds is **declaration-level**: one region per
//!   captured declaration. This is the shape that actually ports Go.
//!
//! Neutrality is unchanged and load-bearing. No Go type, kind, or keyword is named in this file:
//! `int` reaches it as a key to look up in the pack's type map, and `struct` reaches it as a
//! string the pack chose to capture. What this file DOES own is Rust's side of the translation —
//! identifier casing and the shape of an emitted item — because that is the target language it
//! renders, not the source language it must stay ignorant of.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use port_engine_api::{
    Declaration, PortError, RegionId, RuleId, SourceModel, TransformPlan, UnitId,
};
use port_engine_rust_ir::RustIr;

/// Fail-closed readiness gate. `true` once transform apply is present.
pub const fn w0_ready() -> bool {
    true
}

/// Unit-level construction: emit one empty region named for the unit.
pub const CONSTRUCTION_PASS_THROUGH: &str = "pass_through";
/// Unit-level construction: emit a minimal empty fn region for fixture coupling.
pub const CONSTRUCTION_EMPTY_CANARY: &str = "empty_canary";

/// Declaration-level construction: a constant with a declared type and value.
pub const CONSTRUCTION_RUST_CONST: &str = "rust_const";
/// Declaration-level construction: a transparent type alias.
pub const CONSTRUCTION_RUST_TYPE_ALIAS: &str = "rust_type_alias";
/// Declaration-level construction: a single-field tuple struct over the underlying type.
pub const CONSTRUCTION_RUST_NEWTYPE: &str = "rust_newtype";
/// Declaration-level construction: a struct with fields, plus an `impl` block for its methods.
pub const CONSTRUCTION_RUST_STRUCT: &str = "rust_struct";
/// Declaration-level construction: a trait with one signature per method.
pub const CONSTRUCTION_RUST_TRAIT: &str = "rust_trait";
/// Declaration-level construction: a free function, signature only.
pub const CONSTRUCTION_RUST_FN: &str = "rust_fn";

/// Precondition: the planned unit must exist in the source model.
pub const PRECONDITION_UNIT_PRESENT: &str = "unit_present";

/// Attribute key a construction reads a declared value from.
pub const ATTR_VALUE: &str = "value";

/// Flag marking a declaration as part of the source's public surface.
pub const FLAG_EXPORTED: &str = "exported";
/// Flag marking a variadic signature.
pub const FLAG_VARIADIC: &str = "variadic";
/// Flag marking a method bound through a pointer receiver.
pub const FLAG_POINTER_RECEIVER: &str = "pointer_receiver";

/// Child kinds a construction reads. Opaque here: these are the strings the pack and the front end
/// agreed on, and this face compares them without interpreting them.
const CHILD_FIELD: &str = "field";
const CHILD_METHOD: &str = "method";
const CHILD_PARAM: &str = "param";
const CHILD_RESULT: &str = "result";

/// Everything the transform needs from a loaded rule pack.
///
/// Implemented by the rulepack adapter. Rule-level lookups take a [`RuleId`]; pack-level data —
/// the type map and the deferred-kind set — is asked of the pack as a whole.
pub trait PackSemantics {
    /// Construction id for `rule`, if the pack declares it.
    fn construction(&self, rule: &RuleId) -> Option<&str>;
    /// Precondition id for `rule`, if the pack declares it.
    fn precondition(&self, rule: &RuleId) -> Option<&str>;
    /// Declaration kinds `rule` captures. Empty means the rule is unit-level.
    fn captures(&self, rule: &RuleId) -> Option<&[String]>;
    /// Source type spelling → target type spelling.
    fn type_map(&self) -> &BTreeMap<String, String>;
    /// Per-construction overrides of [`PackSemantics::type_map`], keyed by construction id.
    ///
    /// One source type does not always map to one target type: the same spelling can need a
    /// different target depending on the item being built — an owned type is right for a field
    /// and impossible for a constant, for instance. Overriding is DATA for the same reason the
    /// base map is: which target a source type takes in which position is a translation decision,
    /// and a decision belongs in the pack rather than in a branch here.
    fn type_map_overrides(&self, construction: &str) -> Option<&BTreeMap<String, String>>;
    /// Declaration kinds the pack knowingly does not translate yet.
    fn deferred_kinds(&self) -> &BTreeSet<String>;
}

/// Typed refusal from transform apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransformError {
    /// Pack did not declare construction/precondition/captures for a planned rule.
    MissingSemantics {
        /// Rule missing semantics.
        rule: String,
        /// Which field was absent.
        field: &'static str,
    },
    /// Precondition evaluation refused.
    Precondition {
        /// Rule being applied.
        rule: String,
        /// Unit under transform.
        unit: String,
        /// Precondition id that failed.
        precondition: String,
    },
    /// Construction id is not one of the known set.
    UnknownConstruction {
        /// Rule being applied.
        rule: String,
        /// Construction id found.
        construction: String,
    },
    /// A construction was applied to a declaration kind it cannot build from.
    ConstructionKindMismatch {
        /// Construction that was asked.
        construction: String,
        /// Declaration kind it was asked to build from.
        kind: String,
        /// Declaration name, for locating it.
        name: String,
    },
    /// A construction needs a declared datum the model does not carry.
    MissingDatum {
        /// Construction that needs it.
        construction: String,
        /// Declaration that lacks it.
        name: String,
        /// What was missing.
        datum: &'static str,
    },
    /// A type spelling resolves to nothing: not declared in the unit, not in the pack's type map.
    UnmappedType {
        /// Unit under transform.
        unit: String,
        /// Declaration whose type could not be resolved.
        name: String,
        /// The unresolvable source type spelling.
        type_ref: String,
    },
    /// A declaration is captured by no rule and deferred by no policy.
    UncapturedDeclaration {
        /// Unit that declares it.
        unit: String,
        /// Declaration name.
        name: String,
        /// Declaration kind that nothing selects.
        kind: String,
    },
    /// A construct the engine does not translate yet, refused by name.
    Unsupported {
        /// Declaration that carries it.
        name: String,
        /// What is unsupported, and where the program records the analysis.
        detail: String,
    },
    /// The plan named a unit the model does not carry.
    UnitNotInModel {
        /// The absent unit.
        unit: String,
    },
    /// IR / syn assembly refused.
    Ir(PortError),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSemantics { rule, field } => {
                write!(f, "transform missing `{field}` for rule `{rule}`")
            }
            Self::Precondition {
                rule,
                unit,
                precondition,
            } => write!(
                f,
                "transform precondition `{precondition}` failed for rule `{rule}` unit `{unit}`"
            ),
            Self::UnknownConstruction { rule, construction } => write!(
                f,
                "transform unknown construction `{construction}` for rule `{rule}`"
            ),
            Self::ConstructionKindMismatch {
                construction,
                kind,
                name,
            } => write!(
                f,
                "transform construction `{construction}` cannot build from a `{kind}` declaration \
                 (`{name}`)"
            ),
            Self::MissingDatum {
                construction,
                name,
                datum,
            } => write!(
                f,
                "transform construction `{construction}` needs `{datum}` for declaration `{name}`"
            ),
            Self::UnmappedType {
                unit,
                name,
                type_ref,
            } => write!(
                f,
                "transform cannot resolve type `{type_ref}` for `{name}` in unit `{unit}`: it is \
                 declared nowhere in the unit and the pack's type map does not carry it"
            ),
            Self::UncapturedDeclaration { unit, name, kind } => write!(
                f,
                "transform refuses to drop `{name}`: unit `{unit}` declares it as `{kind}`, no \
                 rule captures that kind, and the pack does not defer it"
            ),
            Self::Unsupported { name, detail } => {
                write!(f, "transform refuses `{name}`: {detail}")
            }
            Self::UnitNotInModel { unit } => {
                write!(
                    f,
                    "transform planned unit `{unit}` is absent from the model"
                )
            }
            Self::Ir(err) => write!(f, "transform IR assembly failed: {err}"),
        }
    }
}

impl std::error::Error for TransformError {}

// ---------------------------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------------------------

/// Sanitize a unit or rule id into a Rust-safe region / fn name segment.
#[must_use]
pub fn sanitize_ident(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push_str("unit");
    }
    if out.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// Region id for one unit-level plan step: `{sanitized_unit}__{sanitized_rule}`.
#[must_use]
pub fn region_id_for(unit: &UnitId, rule: &RuleId) -> String {
    format!("{}__{}", sanitize_ident(&unit.0), sanitize_ident(&rule.0))
}

/// Region id for one declaration-level plan step, extended with the declaration.
///
/// The declaration segment is what keeps two rules capturing different kinds from colliding on one
/// region — and one region silently overwriting another is exactly the loss the kernel's
/// duplicate-region refusal exists to catch, arriving one layer earlier.
#[must_use]
pub fn region_id_for_declaration(unit: &UnitId, rule: &RuleId, declaration: &str) -> String {
    format!(
        "{}__{}__{}",
        sanitize_ident(&unit.0),
        sanitize_ident(&rule.0),
        sanitize_ident(declaration)
    )
}

/// `MaxRetries` → `MAX_RETRIES`. Rust's constant convention.
#[must_use]
pub fn to_screaming_snake(raw: &str) -> String {
    to_snake_case(raw).to_ascii_uppercase()
}

/// `MaxRetries` → `max_retries`. Rust's function and binding convention.
///
/// Runs of capitals are kept together, so `ParseURL` becomes `parse_url` and not `parse_u_r_l`.
#[must_use]
pub fn to_snake_case(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len() + 4);
    for (index, ch) in chars.iter().enumerate() {
        if !ch.is_ascii_alphanumeric() {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            continue;
        }
        if ch.is_ascii_uppercase() && index > 0 {
            let previous_is_lower =
                chars[index - 1].is_ascii_lowercase() || chars[index - 1].is_ascii_digit();
            let next_is_lower = chars
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_lowercase());
            if (previous_is_lower || next_is_lower) && !out.ends_with('_') {
                out.push('_');
            }
        }
        out.push(ch.to_ascii_lowercase());
    }
    if out.is_empty() {
        out.push_str("item");
    }
    if out.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        out.insert(0, '_');
    }
    out
}

/// `point` → `Point`. Rust's type convention; already-capitalized names pass through.
#[must_use]
pub fn to_pascal_case(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut capitalize_next = true;
    for ch in raw.chars() {
        if !ch.is_ascii_alphanumeric() {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            out.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() {
        out.push_str("Item");
    }
    if out.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        out.insert(0, '_');
    }
    out
}

/// Rust visibility prefix for a declaration: `pub ` when the source exported it, else nothing.
fn visibility(declaration: &Declaration) -> &'static str {
    if declaration.flags.contains(FLAG_EXPORTED) {
        "pub "
    } else {
        ""
    }
}

// ---------------------------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------------------------

/// Apply `plan` constructions against `model` using pack semantics → deterministic [`RustIr`].
///
/// Step order is plan order; within a declaration-level step, declaration order. Each emitted item
/// becomes one IR region with a syn AST.
///
/// Before returning, every declaration of every planned unit is checked to be either captured by a
/// rule or deferred by pack policy. That check is the difference between a translator and a filter:
/// without it a declaration no rule happens to select is dropped in silence, the emit is green, the
/// receipt is reproducible, and the only evidence that something was lost is that it is not there.
///
/// # Errors
/// [`TransformError`] on missing semantics, failed precondition, unknown construction, unresolvable
/// type, an uncaptured declaration, an unsupported construct, or an IR refusal.
pub fn apply(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
) -> Result<RustIr, TransformError> {
    apply_with_provenance(plan, semantics, model).map(|(ir, _)| ir)
}

/// [`apply`], plus which unit each emitted region came from.
///
/// The provenance is not derivable from a region id by parsing it. Region ids are built from
/// SANITIZED segments, and sanitization is lossy — two different unit ids can sanitize to the same
/// text, and a unit id containing adjacent non-alphanumerics produces a segment separator of its
/// own. Anything downstream that needs to group regions by unit (a module layout, a per-unit
/// output tree) must be told, not left to re-derive it from a string that no longer distinguishes.
///
/// # Errors
/// The same [`TransformError`] set as [`apply`].
pub fn apply_with_provenance(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
) -> Result<(RustIr, BTreeMap<RegionId, UnitId>), TransformError> {
    let model_units: BTreeSet<String> = model.units().into_iter().map(|u| u.0).collect();
    let mut provenance: BTreeMap<RegionId, UnitId> = BTreeMap::new();

    let mut region_names: Vec<String> = Vec::new();
    let mut sources: Vec<(String, String)> = Vec::new();
    // unit → the declaration kinds some applied rule captured, for the coverage check below.
    let mut captured_kinds: BTreeMap<UnitId, BTreeSet<String>> = BTreeMap::new();

    for step in &plan.steps {
        let construction =
            semantics
                .construction(&step.rule)
                .ok_or_else(|| TransformError::MissingSemantics {
                    rule: step.rule.0.clone(),
                    field: "construction",
                })?;
        let precondition =
            semantics
                .precondition(&step.rule)
                .ok_or_else(|| TransformError::MissingSemantics {
                    rule: step.rule.0.clone(),
                    field: "precondition",
                })?;
        let captures =
            semantics
                .captures(&step.rule)
                .ok_or_else(|| TransformError::MissingSemantics {
                    rule: step.rule.0.clone(),
                    field: "captures",
                })?;

        check_precondition(precondition, &step.unit, &step.rule, &model_units)?;

        if captures.is_empty() {
            let region = region_id_for(&step.unit, &step.rule);
            let source = unit_level_source(construction, &step.rule, &region)?;
            provenance.insert(RegionId(region.clone()), step.unit.clone());
            region_names.push(region.clone());
            sources.push((region, source));
            continue;
        }

        let declarations =
            model
                .declarations(&step.unit)
                .ok_or_else(|| TransformError::UnitNotInModel {
                    unit: step.unit.0.clone(),
                })?;
        let scope = LocalScope::of(&declarations);
        let entry = captured_kinds.entry(step.unit.clone()).or_default();
        for capture in captures {
            entry.insert(capture.clone());
        }

        for declaration in declarations.iter().filter(|d| captures.contains(&d.kind)) {
            let region = region_id_for_declaration(&step.unit, &step.rule, &declaration.name);
            let source = declaration_source(
                construction,
                declaration,
                &Resolver {
                    scope: &scope,
                    type_map: semantics.type_map(),
                    overrides: semantics.type_map_overrides(construction),
                    unit: &step.unit,
                },
            )?;
            provenance.insert(RegionId(region.clone()), step.unit.clone());
            region_names.push(region.clone());
            sources.push((region, source));
        }
    }

    prove_every_declaration_is_accounted_for(plan, semantics, model, &captured_kinds)?;

    let refs: Vec<&str> = region_names.iter().map(String::as_str).collect();
    let mut ir = RustIr::new(&refs);
    for (region, source) in sources {
        ir.set_file_from_str(&region, &source)
            .map_err(TransformError::Ir)?;
    }
    Ok((ir, provenance))
}

/// Every declaration of every planned unit must be captured by a rule or deferred by policy.
///
/// Deferral is DECLARED, not inferred. A kind the pack lists in `deferred_kinds` is one someone
/// wrote down as knowingly untranslated, with the reason travelling in the pack and therefore in
/// the pack digest and therefore in the receipt. A kind that is merely unselected is indisputably
/// lost work, and it must not look like a decision.
fn prove_every_declaration_is_accounted_for(
    plan: &TransformPlan,
    semantics: &dyn PackSemantics,
    model: &dyn SourceModel,
    captured_kinds: &BTreeMap<UnitId, BTreeSet<String>>,
) -> Result<(), TransformError> {
    let deferred = semantics.deferred_kinds();
    let planned_units: BTreeSet<&UnitId> = plan.steps.iter().map(|step| &step.unit).collect();

    for unit in planned_units {
        let Some(declarations) = model.declarations(unit) else {
            continue;
        };
        let empty = BTreeSet::new();
        let captured = captured_kinds.get(unit).unwrap_or(&empty);
        for declaration in &declarations {
            if captured.contains(&declaration.kind) || deferred.contains(&declaration.kind) {
                continue;
            }
            return Err(TransformError::UncapturedDeclaration {
                unit: unit.0.clone(),
                name: declaration.name.clone(),
                kind: declaration.kind.clone(),
            });
        }
    }
    Ok(())
}

fn check_precondition(
    precondition: &str,
    unit: &UnitId,
    rule: &RuleId,
    model_units: &BTreeSet<String>,
) -> Result<(), TransformError> {
    match precondition {
        PRECONDITION_UNIT_PRESENT => {
            if model_units.contains(&unit.0) {
                Ok(())
            } else {
                Err(TransformError::Precondition {
                    rule: rule.0.clone(),
                    unit: unit.0.clone(),
                    precondition: precondition.to_owned(),
                })
            }
        }
        other => Err(TransformError::Precondition {
            rule: rule.0.clone(),
            unit: unit.0.clone(),
            precondition: other.to_owned(),
        }),
    }
}

fn unit_level_source(
    construction: &str,
    rule: &RuleId,
    region: &str,
) -> Result<String, TransformError> {
    match construction {
        CONSTRUCTION_PASS_THROUGH => Ok(format!("pub fn {region}() {{}}")),
        CONSTRUCTION_EMPTY_CANARY => Ok(format!("pub fn {region}_canary() {{}}")),
        other => Err(TransformError::UnknownConstruction {
            rule: rule.0.clone(),
            construction: other.to_owned(),
        }),
    }
}

// ---------------------------------------------------------------------------------------------
// Type resolution
// ---------------------------------------------------------------------------------------------

/// The type names one unit declares, and the target spelling each resolves to.
struct LocalScope {
    types: BTreeMap<String, String>,
}

impl LocalScope {
    /// A declaration whose kind carries a type name contributes that name to the unit's scope.
    ///
    /// Which kinds those are is not decided here — a declaration is a type declaration exactly
    /// when it has a name and is not one of the value-shaped kinds — because deciding it here
    /// would mean naming the source language's kind vocabulary in the neutral face. What this does
    /// instead is record every named declaration, and let a collision be impossible by the
    /// front end's own single-namespace refusal.
    fn of(declarations: &[Declaration]) -> Self {
        let mut types = BTreeMap::new();
        for declaration in declarations {
            if !declaration.name.is_empty() {
                types.insert(declaration.name.clone(), to_pascal_case(&declaration.name));
            }
        }
        Self { types }
    }
}

struct Resolver<'a> {
    scope: &'a LocalScope,
    type_map: &'a BTreeMap<String, String>,
    overrides: Option<&'a BTreeMap<String, String>>,
    unit: &'a UnitId,
}

impl Resolver<'_> {
    /// Resolve a source type spelling to its target spelling.
    ///
    /// A name the unit itself declares wins over the pack's map. It has to: the map is keyed by
    /// spelling, so a unit declaring a type whose name collides with a mapped one would otherwise
    /// silently emit the mapped type in place of its own, and the emitted code would compile while
    /// meaning something different.
    ///
    /// Nothing is guessed. An unresolvable spelling refuses, because the alternative — passing the
    /// source spelling through and hoping it is also valid target syntax — produces code that
    /// either fails to compile far from the cause or, worse, compiles as some unrelated type that
    /// happens to share a name.
    fn resolve(&self, type_ref: &str, declaration_name: &str) -> Result<String, TransformError> {
        if type_ref.is_empty() {
            return Err(TransformError::MissingDatum {
                construction: "type resolution".to_owned(),
                name: declaration_name.to_owned(),
                datum: "type",
            });
        }
        if let Some(local) = self.scope.types.get(type_ref) {
            return Ok(local.clone());
        }
        if let Some(mapped) = self.overrides.and_then(|map| map.get(type_ref)) {
            return Ok(mapped.clone());
        }
        if let Some(mapped) = self.type_map.get(type_ref) {
            return Ok(mapped.clone());
        }
        Err(TransformError::UnmappedType {
            unit: self.unit.0.clone(),
            name: declaration_name.to_owned(),
            type_ref: type_ref.to_owned(),
        })
    }
}

// ---------------------------------------------------------------------------------------------
// Constructions
// ---------------------------------------------------------------------------------------------

fn declaration_source(
    construction: &str,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    match construction {
        CONSTRUCTION_RUST_CONST => build_const(declaration, resolver),
        CONSTRUCTION_RUST_TYPE_ALIAS => build_type_alias(declaration, resolver),
        CONSTRUCTION_RUST_NEWTYPE => build_newtype(declaration, resolver),
        CONSTRUCTION_RUST_STRUCT => build_struct(declaration, resolver),
        CONSTRUCTION_RUST_TRAIT => build_trait(declaration, resolver),
        CONSTRUCTION_RUST_FN => build_fn(declaration, resolver),
        other => Err(TransformError::UnknownConstruction {
            rule: String::new(),
            construction: other.to_owned(),
        }),
    }
}

fn build_const(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let value = declaration
        .attr(ATTR_VALUE)
        .ok_or_else(|| TransformError::MissingDatum {
            construction: CONSTRUCTION_RUST_CONST.to_owned(),
            name: declaration.name.clone(),
            datum: ATTR_VALUE,
        })?;
    let ty = resolver.resolve(&declaration.type_ref, &declaration.name)?;
    Ok(format!(
        "{}const {}: {} = {};",
        visibility(declaration),
        to_screaming_snake(&declaration.name),
        ty,
        value
    ))
}

fn build_type_alias(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let ty = resolver.resolve(&declaration.type_ref, &declaration.name)?;
    Ok(format!(
        "{}type {} = {};",
        visibility(declaration),
        to_pascal_case(&declaration.name),
        ty
    ))
}

/// A defined type over an underlying type becomes a newtype, never an alias.
///
/// The distinction is the whole point of the source construct: a defined type is a DISTINCT type
/// that does not interchange with its underlying one, and rendering it as an alias would erase
/// exactly the property it was declared for. A newtype keeps the distinction in the target's own
/// type system.
fn build_newtype(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let ty = resolver.resolve(&declaration.type_ref, &declaration.name)?;
    let name = to_pascal_case(&declaration.name);
    let vis = visibility(declaration);
    let mut out = format!("{vis}struct {name}({vis}{ty});");
    if let Some(methods) = render_inherent_impl(&name, declaration, resolver)? {
        out.push('\n');
        out.push_str(&methods);
    }
    Ok(out)
}

fn build_struct(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let name = to_pascal_case(&declaration.name);
    let vis = visibility(declaration);

    let fields = declaration.children_of_kind(CHILD_FIELD);
    let mut body = String::new();
    for field in fields {
        let ty = resolver.resolve(&field.type_ref, &field.name)?;
        body.push_str(&format!(
            "    {}{}: {},\n",
            visibility(field),
            to_snake_case(&field.name),
            ty
        ));
    }

    let mut out = if body.is_empty() {
        format!("{vis}struct {name};")
    } else {
        format!("{vis}struct {name} {{\n{body}}}")
    };
    if let Some(methods) = render_inherent_impl(&name, declaration, resolver)? {
        out.push('\n');
        out.push_str(&methods);
    }
    Ok(out)
}

fn build_trait(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let name = to_pascal_case(&declaration.name);
    let mut body = String::new();
    for method in declaration.children_of_kind(CHILD_METHOD) {
        // A trait item carries NO visibility: it is as public as the trait itself, and `pub` on
        // one is not valid Rust. syn parses it anyway, which is exactly why the emitted tree is
        // compiled rather than only parsed.
        let signature = render_method_signature(method, resolver, Visibility::Inherited)?;
        body.push_str(&format!("    {signature};\n"));
    }
    Ok(format!(
        "{}trait {} {{\n{}}}",
        visibility(declaration),
        name,
        body
    ))
}

fn build_fn(declaration: &Declaration, resolver: &Resolver<'_>) -> Result<String, TransformError> {
    if !declaration.children_of_kind(CHILD_FIELD).is_empty() {
        return Err(TransformError::ConstructionKindMismatch {
            construction: CONSTRUCTION_RUST_FN.to_owned(),
            kind: declaration.kind.clone(),
            name: declaration.name.clone(),
        });
    }
    refuse_variadic(declaration)?;
    let params = render_params(declaration, resolver, None)?;
    let results = render_results(declaration, resolver)?;
    Ok(format!(
        "{}fn {}({}){} {{ todo!() }}",
        visibility(declaration),
        to_snake_case(&declaration.name),
        params,
        results
    ))
}

/// Render the `impl` block for a declaration's methods, or `None` when it has none.
fn render_inherent_impl(
    type_name: &str,
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<Option<String>, TransformError> {
    let methods = declaration.children_of_kind(CHILD_METHOD);
    if methods.is_empty() {
        return Ok(None);
    }
    let mut body = String::new();
    for method in methods {
        let signature = render_method_signature(method, resolver, Visibility::FromSource)?;
        body.push_str(&format!("    {signature} {{ todo!() }}\n"));
    }
    Ok(Some(format!("impl {type_name} {{\n{body}}}")))
}

/// Whether an item may carry a visibility keyword at all.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Visibility {
    /// Take the visibility the source declared.
    FromSource,
    /// Emit none — the enclosing item already decides it.
    Inherited,
}

fn render_method_signature(
    method: &Declaration,
    resolver: &Resolver<'_>,
    vis: Visibility,
) -> Result<String, TransformError> {
    refuse_variadic(method)?;
    // A pointer receiver is refused rather than rendered. `&self` would drop the mutation the
    // receiver exists to permit, and `&mut self` would claim a mutation the source may never
    // perform; both are a guess about aliasing, which is precisely what
    // docs/programs/k8s-port/census/ownership-escape.md is the analysis for.
    if method.flags.contains(FLAG_POINTER_RECEIVER) {
        return Err(TransformError::Unsupported {
            name: method.name.clone(),
            detail: "pointer receiver: `&self` drops the mutation it permits and `&mut self` \
                     claims one the source may not perform — see \
                     docs/programs/k8s-port/census/ownership-escape.md"
                .to_owned(),
        });
    }
    let params = render_params(method, resolver, Some("&self"))?;
    let results = render_results(method, resolver)?;
    let vis = match vis {
        Visibility::FromSource => visibility(method),
        Visibility::Inherited => "",
    };
    Ok(format!(
        "{vis}fn {}({params}){results}",
        to_snake_case(&method.name)
    ))
}

fn refuse_variadic(declaration: &Declaration) -> Result<(), TransformError> {
    if declaration.flags.contains(FLAG_VARIADIC) {
        return Err(TransformError::Unsupported {
            name: declaration.name.clone(),
            detail: "variadic signature: the target has no variadic parameter, so this needs a \
                     rule that chooses a slice or a builder rather than a default"
                .to_owned(),
        });
    }
    Ok(())
}

fn render_params(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
    receiver: Option<&str>,
) -> Result<String, TransformError> {
    let mut rendered: Vec<String> = Vec::new();
    if let Some(receiver) = receiver {
        rendered.push(receiver.to_owned());
    }
    for (index, param) in declaration.children_of_kind(CHILD_PARAM).iter().enumerate() {
        let ty = resolver.resolve(&param.type_ref, &declaration.name)?;
        // An unnamed parameter is legal in the source and illegal in the target, so it is given a
        // positional name. The position is already the parameter's identity here, so nothing is
        // invented that was not already true.
        let name = if param.name.is_empty() || param.name == "_" {
            format!("arg{index}")
        } else {
            to_snake_case(&param.name)
        };
        rendered.push(format!("{name}: {ty}"));
    }
    Ok(rendered.join(", "))
}

fn render_results(
    declaration: &Declaration,
    resolver: &Resolver<'_>,
) -> Result<String, TransformError> {
    let results = declaration.children_of_kind(CHILD_RESULT);
    let mut types = Vec::with_capacity(results.len());
    for result in results {
        types.push(resolver.resolve(&result.type_ref, &declaration.name)?);
    }
    match types.len() {
        0 => Ok(String::new()),
        1 => Ok(format!(" -> {}", types[0])),
        // Several results become a tuple. That is the target's own shape for "more than one value
        // out", and it keeps arity and order visible instead of inventing a struct nobody declared.
        _ => Ok(format!(" -> ({})", types.join(", "))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use port_engine_api::{Digest, LanguagePair, PlanStep, TargetIr};

    // ---------------------------------------------------------------------------------------
    // Fakes
    // ---------------------------------------------------------------------------------------

    #[derive(Default)]
    struct Pack {
        /// rule → (construction, precondition, captures)
        rules: BTreeMap<&'static str, (&'static str, &'static str, Vec<String>)>,
        types: BTreeMap<String, String>,
        overrides: BTreeMap<String, BTreeMap<String, String>>,
        deferred: BTreeSet<String>,
    }

    impl Pack {
        fn with_rule(
            mut self,
            id: &'static str,
            construction: &'static str,
            captures: &[&str],
        ) -> Self {
            self.rules.insert(
                id,
                (
                    construction,
                    PRECONDITION_UNIT_PRESENT,
                    captures.iter().map(|c| (*c).to_owned()).collect(),
                ),
            );
            self
        }

        fn with_types(mut self, pairs: &[(&str, &str)]) -> Self {
            for (from, to) in pairs {
                self.types.insert((*from).to_owned(), (*to).to_owned());
            }
            self
        }

        fn with_override(mut self, construction: &str, from: &str, to: &str) -> Self {
            self.overrides
                .entry(construction.to_owned())
                .or_default()
                .insert(from.to_owned(), to.to_owned());
            self
        }

        fn with_deferred(mut self, kinds: &[&str]) -> Self {
            for kind in kinds {
                self.deferred.insert((*kind).to_owned());
            }
            self
        }
    }

    impl PackSemantics for Pack {
        fn construction(&self, rule: &RuleId) -> Option<&str> {
            self.rules.get(rule.0.as_str()).map(|(c, _, _)| *c)
        }
        fn precondition(&self, rule: &RuleId) -> Option<&str> {
            self.rules.get(rule.0.as_str()).map(|(_, p, _)| *p)
        }
        fn captures(&self, rule: &RuleId) -> Option<&[String]> {
            self.rules
                .get(rule.0.as_str())
                .map(|(_, _, c)| c.as_slice())
        }
        fn type_map(&self) -> &BTreeMap<String, String> {
            &self.types
        }
        fn type_map_overrides(&self, construction: &str) -> Option<&BTreeMap<String, String>> {
            self.overrides.get(construction)
        }
        fn deferred_kinds(&self) -> &BTreeSet<String> {
            &self.deferred
        }
    }

    struct Model {
        units: Vec<UnitId>,
        declarations: BTreeMap<String, Vec<Declaration>>,
    }

    impl SourceModel for Model {
        fn language(&self) -> &str {
            "go"
        }
        fn snapshot_digest(&self) -> Digest {
            Digest("snap".into())
        }
        fn units(&self) -> Vec<UnitId> {
            self.units.clone()
        }
        fn declarations(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
            self.declarations.get(&unit.0).cloned()
        }
    }

    fn decl(kind: &str, name: &str, type_ref: &str) -> Declaration {
        Declaration {
            kind: kind.into(),
            name: name.into(),
            type_ref: type_ref.into(),
            flags: ["exported".to_owned()].into_iter().collect(),
            attrs: BTreeMap::new(),
            children: Vec::new(),
        }
    }

    fn child(kind: &str, name: &str, type_ref: &str) -> Declaration {
        let mut node = decl(kind, name, type_ref);
        node.flags.clear();
        node
    }

    fn model_with(declarations: Vec<Declaration>) -> Model {
        Model {
            units: vec![UnitId("u".into())],
            declarations: BTreeMap::from([("u".to_owned(), declarations)]),
        }
    }

    fn plan_with(rules: &[&str]) -> TransformPlan {
        TransformPlan {
            pair: LanguagePair {
                source: "go".into(),
                target: "rust".into(),
            },
            steps: rules
                .iter()
                .map(|rule| PlanStep {
                    unit: UnitId("u".into()),
                    rule: RuleId((*rule).to_owned()),
                })
                .collect(),
        }
    }

    fn rendered(ir: &RustIr) -> String {
        // Round-trip through the renderer the pipeline actually uses, so a construction emitting
        // text that syn cannot parse fails here rather than three stages later.
        let renderer = port_engine_rust_ir::SynQuoteRenderer::new("transform-test-fmt");
        let emitted = renderer
            .render_rust_ir(ir)
            .expect("emitted Rust must render");
        emitted
            .values()
            .map(|bytes| String::from_utf8(bytes.clone()).expect("utf-8"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ---------------------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------------------

    #[test]
    fn claims_transform_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn unit_level_rule_still_emits_one_region_per_unit() {
        let pack = Pack::default().with_rule("canary_empty_unit", CONSTRUCTION_EMPTY_CANARY, &[]);
        let model = model_with(Vec::new());
        let ir = apply(&plan_with(&["canary_empty_unit"]), &pack, &model).expect("apply");
        assert_eq!(ir.regions().len(), 1);
        assert_eq!(ir.regions()[0].0, "u__canary_empty_unit");
    }

    #[test]
    fn constant_carries_its_type_and_value() {
        let mut max = decl("const", "MaxRetries", "int");
        max.attrs.insert("value".into(), "3".into());
        let pack = Pack::default()
            .with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"])
            .with_types(&[("int", "i64")]);
        let ir = apply(&plan_with(&["consts"]), &pack, &model_with(vec![max])).expect("apply");
        let text = rendered(&ir);
        assert!(text.contains("MAX_RETRIES"), "{text}");
        assert!(text.contains("i64"), "{text}");
        assert!(text.contains("3"), "{text}");
    }

    #[test]
    fn function_renders_named_params_and_a_result() {
        let mut add = decl("func", "Add", "");
        add.children = vec![
            child("param", "a", "int"),
            child("param", "b", "int"),
            child("result", "", "int"),
        ];
        let pack = Pack::default()
            .with_rule("funcs", CONSTRUCTION_RUST_FN, &["func"])
            .with_types(&[("int", "i64")]);
        let ir = apply(&plan_with(&["funcs"]), &pack, &model_with(vec![add])).expect("apply");
        let text = rendered(&ir);
        assert!(text.contains("fn add"), "{text}");
        assert!(
            text.contains("a : i64") || text.contains("a: i64"),
            "{text}"
        );
        assert!(
            text.contains("-> i64") || text.contains("- > i64"),
            "{text}"
        );
    }

    #[test]
    fn struct_renders_fields_and_an_inherent_impl() {
        let mut point = decl("struct", "Point", "");
        let mut x = child("field", "X", "int");
        x.flags.insert("exported".into());
        let mut shift = child("method", "Shift", "");
        shift.flags.insert("exported".into());
        shift.children = vec![child("param", "dx", "int"), child("result", "", "Point")];
        point.children = vec![x, child("field", "label", "string"), shift];

        let pack = Pack::default()
            .with_rule("structs", CONSTRUCTION_RUST_STRUCT, &["struct"])
            .with_types(&[("int", "i64"), ("string", "String")]);
        let ir = apply(&plan_with(&["structs"]), &pack, &model_with(vec![point])).expect("apply");
        let text = rendered(&ir);
        assert!(text.contains("struct Point"), "{text}");
        assert!(text.contains("pub x"), "{text}");
        assert!(text.contains("impl Point"), "{text}");
        assert!(text.contains("fn shift"), "{text}");
        // The unexported field must not become part of the public surface.
        assert!(!text.contains("pub label"), "{text}");
    }

    /// A defined type is a distinct type. Rendering it as an alias would erase the one property it
    /// was declared for, and the emitted code would compile while meaning something weaker.
    #[test]
    fn defined_type_becomes_a_newtype_and_alias_stays_transparent() {
        let celsius = decl("named", "Celsius", "float64");
        let id = decl("alias", "ID", "string");
        let pack = Pack::default()
            .with_rule("named", CONSTRUCTION_RUST_NEWTYPE, &["named"])
            .with_rule("aliases", CONSTRUCTION_RUST_TYPE_ALIAS, &["alias"])
            .with_types(&[("float64", "f64"), ("string", "String")]);
        let ir = apply(
            &plan_with(&["named", "aliases"]),
            &pack,
            &model_with(vec![celsius, id]),
        )
        .expect("apply");
        let text = rendered(&ir);
        assert!(text.contains("struct Celsius"), "{text}");
        // `ID` stays `ID`: an all-capitals name is an acronym, and lowercasing it to `Id` would
        // rename the type rather than recase it.
        assert!(text.contains("type ID = String"), "{text}");
    }

    /// The coverage rule. Without it a declaration nothing captures is dropped in silence and the
    /// emit is green over a corpus it did not translate.
    #[test]
    fn refuses_a_declaration_no_rule_captures() {
        let pack = Pack::default().with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"]);
        let model = model_with(vec![decl("var", "Enabled", "bool")]);
        let err = apply(&plan_with(&["consts"]), &pack, &model).expect_err("uncaptured refuses");
        assert!(matches!(
            err,
            TransformError::UncapturedDeclaration { ref kind, .. } if kind == "var"
        ));
    }

    /// Deferral is a decision someone wrote down, and it travels in the pack digest. That is what
    /// separates it from the same declaration merely going unselected.
    #[test]
    fn declared_deferral_admits_what_bare_omission_would_not() {
        let mut max = decl("const", "MaxRetries", "int");
        max.attrs.insert("value".into(), "3".into());
        let pack = Pack::default()
            .with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"])
            .with_types(&[("int", "i64")])
            .with_deferred(&["var"]);
        let model = model_with(vec![max, decl("var", "Enabled", "bool")]);
        let ir = apply(&plan_with(&["consts"]), &pack, &model).expect("deferred kind is accounted");
        assert_eq!(ir.regions().len(), 1, "the deferred var emits nothing");
    }

    /// Never guess a type. A passed-through source spelling either fails to compile far from its
    /// cause or, worse, resolves to an unrelated target type with the same name.
    #[test]
    fn refuses_a_type_the_pack_does_not_map() {
        let pack = Pack::default().with_rule("consts", CONSTRUCTION_RUST_CONST, &["const"]);
        let mut value = decl("const", "K", "uintptr");
        value.attrs.insert("value".into(), "0".into());
        let err =
            apply(&plan_with(&["consts"]), &pack, &model_with(vec![value])).expect_err("unmapped");
        assert!(matches!(
            err,
            TransformError::UnmappedType { ref type_ref, .. } if type_ref == "uintptr"
        ));
    }

    /// A locally declared name must win over the pack's map, or a unit declaring a type whose name
    /// collides with a mapped one silently emits the mapped type in its place.
    #[test]
    fn local_declaration_shadows_the_type_map() {
        let mut holder = decl("struct", "Holder", "");
        holder.children = vec![child("field", "Inner", "string")];
        let local = decl("struct", "string", "");
        // The map deliberately sends `string` somewhere the local declaration does not, so the two
        // candidate answers are distinguishable in the output.
        let pack = Pack::default()
            .with_rule("structs", CONSTRUCTION_RUST_STRUCT, &["struct"])
            .with_types(&[("string", "MappedElsewhere")]);
        let ir = apply(
            &plan_with(&["structs"]),
            &pack,
            &model_with(vec![holder, local]),
        )
        .expect("apply");
        let text = rendered(&ir);
        assert!(
            text.contains("String"),
            "the unit's own `string` must win over the mapped one: {text}"
        );
        assert!(
            !text.contains("MappedElsewhere"),
            "the pack's map must not shadow a type the unit declares: {text}"
        );
    }

    #[test]
    fn refuses_a_pointer_receiver_rather_than_guessing_aliasing() {
        let mut point = decl("struct", "Point", "");
        let mut method = child("method", "Move", "");
        method.flags.insert(FLAG_POINTER_RECEIVER.to_owned());
        point.children = vec![method];
        let pack = Pack::default().with_rule("structs", CONSTRUCTION_RUST_STRUCT, &["struct"]);
        let err = apply(&plan_with(&["structs"]), &pack, &model_with(vec![point]))
            .expect_err("pointer receiver refuses");
        assert!(matches!(err, TransformError::Unsupported { .. }));
    }

    #[test]
    fn refuses_a_variadic_signature() {
        let mut printf = decl("func", "Printf", "");
        printf.flags.insert(FLAG_VARIADIC.to_owned());
        let pack = Pack::default().with_rule("funcs", CONSTRUCTION_RUST_FN, &["func"]);
        let err = apply(&plan_with(&["funcs"]), &pack, &model_with(vec![printf]))
            .expect_err("variadic refuses");
        assert!(matches!(err, TransformError::Unsupported { .. }));
    }

    #[test]
    fn refuses_unknown_construction() {
        let pack = Pack::default().with_rule("bad", "not_a_construction", &[]);
        let err = apply(&plan_with(&["bad"]), &pack, &model_with(Vec::new()))
            .expect_err("unknown construction");
        assert!(matches!(err, TransformError::UnknownConstruction { .. }));
    }

    #[test]
    fn refuses_missing_unit_precondition() {
        let pack = Pack::default().with_rule("r", CONSTRUCTION_PASS_THROUGH, &[]);
        let model = Model {
            units: Vec::new(),
            declarations: BTreeMap::new(),
        };
        let err = apply(&plan_with(&["r"]), &pack, &model).expect_err("unit missing");
        assert!(matches!(err, TransformError::Precondition { .. }));
    }

    #[test]
    fn sanitize_ident_is_rust_safe() {
        assert_eq!(sanitize_ident("example.com/a"), "example_com_a");
        assert_eq!(sanitize_ident("9x"), "_9x");
    }

    #[test]
    fn casing_keeps_capital_runs_together() {
        assert_eq!(to_snake_case("MaxRetries"), "max_retries");
        assert_eq!(to_snake_case("ParseURL"), "parse_url");
        assert_eq!(to_screaming_snake("MaxRetries"), "MAX_RETRIES");
        assert_eq!(to_pascal_case("point"), "Point");
        assert_eq!(to_pascal_case("Point"), "Point");
    }
}
