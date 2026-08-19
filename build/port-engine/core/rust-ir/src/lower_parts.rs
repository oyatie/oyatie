//! Shared lowering helpers: parsing a spelling into a syn node, and the two attribute forms.

use proc_macro2::TokenStream;
use quote::quote;

use port_engine_api::PortError;

use crate::item_parts::Visibility;
use crate::ty::RustType;

pub(crate) fn parse_type(ty: &RustType) -> Result<syn::Type, PortError> {
    let spelling = ty.spelling();
    syn::parse_str(&spelling).map_err(|err| PortError::Render {
        detail: format!("`{spelling}` is not a valid target type: {err}"),
    })
}

pub(crate) fn parse_ident(name: &str) -> Result<syn::Ident, PortError> {
    syn::parse_str(name).map_err(|err| PortError::Render {
        detail: format!("`{name}` is not a valid target identifier: {err}"),
    })
}

/// A generic PARAMETER, which is richer than an identifier: it may carry a default.
///
/// Parsed as a parameter rather than a name because an alias whose error slot has a default is a
/// usable alias and one without is a fixed shape wearing a type parameter — a reviewer named
/// exactly that.
pub(crate) fn parse_generic_param(source: &str) -> Result<syn::GenericParam, PortError> {
    syn::parse_str(source).map_err(|err| PortError::Render {
        detail: format!("`{source}` is not a valid target generic parameter: {err}"),
    })
}

/// A `use` path, parsed so an unparseable one refuses here rather than in the emitted file.
pub(crate) fn parse_path(source: &str) -> Result<syn::Path, PortError> {
    syn::parse_str(source).map_err(|err| PortError::Render {
        detail: format!("`{source}` is not a valid target import path: {err}"),
    })
}

pub(crate) fn parse_expr(source: &str, what: &str) -> Result<syn::Expr, PortError> {
    syn::parse_str(source).map_err(|err| PortError::Render {
        detail: format!("`{source}` is not a valid target {what}: {err}"),
    })
}

/// Doc comments as `#[doc = "..."]` attributes.
///
/// `prettyplease` renders these back as `///` lines, so the emitted file carries the source's
/// documentation in the form a reader expects rather than in the form the AST stores.
pub(crate) fn lower_docs(docs: &[String]) -> TokenStream {
    docs.iter().map(|line| quote! { #[doc = #line] }).collect()
}

pub(crate) fn lower_vis(vis: Visibility) -> TokenStream {
    match vis {
        Visibility::Public => quote! { pub },
        Visibility::Inherited => TokenStream::new(),
    }
}
