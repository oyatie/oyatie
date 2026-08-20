//! Which target types an item is BUILT FROM.
//!
//! Split from `item.rs` so that file holds the closed item set and nothing else. What asks this
//! question is the import rule: a module names a library path or it does not, and an import for one
//! it does not name is a denied warning rather than a tidiness point — so the answer has to come
//! from the item tree rather than from a scan of its text.

use crate::item::RustItem;
use crate::item_parts::{RustFn, StructShape};

impl RustItem {
    /// Every type SPELLING this item is built from.
    ///
    /// Walked structurally rather than scanned as text, because a type is a tree and its spellings
    /// sit at the leaves. What asks is the import rule: a module names a library path or it does
    /// not, and an import for one it does not name is a denied warning.
    ///
    /// An item whose types are entirely its own — a sentinel enum, an import — contributes none,
    /// which is correct rather than an omission.
    #[must_use]
    pub fn type_spellings(&self) -> Vec<String> {
        let mut out = Vec::new();
        match self {
            Self::Const { ty, .. } | Self::PackageValue { ty, .. } | Self::TypeAlias { ty, .. } => {
                ty.spellings(&mut out);
            }
            // The impl's SELF TYPE, which is a name the emitted module must have. The traits it
            // implements are the target's own and contribute nothing to resolve.
            Self::MessageImpl { self_ty, .. } => self_ty.spellings(&mut out),
            Self::Struct { shape, .. } => match shape {
                StructShape::Unit => {}
                StructShape::Tuple(fields) | StructShape::Named(fields) => {
                    for field in fields {
                        field.ty.spellings(&mut out);
                    }
                }
            },
            Self::Trait {
                supertraits,
                methods,
                ..
            } => {
                for supertrait in supertraits {
                    supertrait.spellings(&mut out);
                }
                for method in methods {
                    signature_spellings(method, &mut out);
                }
            }
            Self::TraitImpl {
                trait_path,
                self_ty,
                methods,
                ..
            } => {
                trait_path.spellings(&mut out);
                self_ty.spellings(&mut out);
                for method in methods {
                    signature_spellings(method, &mut out);
                }
            }
            Self::InherentImpl {
                self_ty, methods, ..
            } => {
                self_ty.spellings(&mut out);
                for method in methods {
                    signature_spellings(method, &mut out);
                }
            }
            Self::BlanketImpl { bounds, .. } => {
                for bound in bounds {
                    bound.spellings(&mut out);
                }
            }
            Self::Function(function) => signature_spellings(function, &mut out),
            // Types entirely their own, or none at all.
            Self::SentinelEnum { .. }
            | Self::SentinelError { .. }
            | Self::Use { .. }
            | Self::Nothing => {}
        }
        out
    }
}

/// The spellings one signature is built from: every parameter, and the result.
fn signature_spellings(function: &RustFn, into: &mut Vec<String>) {
    for param in &function.params {
        param.ty.spellings(into);
    }
    if let Some(ret) = &function.ret {
        ret.spellings(into);
    }
}
