//! Browser panic reporting for the WASM shell island.
//!
//! ADR-0709 D-6 Rule 1 (stdlib where it will do): this replaces the
//! `console_error_panic_hook` micro-crate, whose entire job was one function.
//! Everything that crate did is available from the standard library plus
//! dependencies this crate already declares:
//!
//! * install-at-most-once — [`std::sync::Once`], stable since Rust 1.0.0;
//! * hook installation — [`std::panic::set_hook`], stable since Rust 1.0.0;
//! * panic rendering — [`std::panic::PanicHookInfo`]'s `Display`, which yields
//!   `panicked at <file>:<line>:<col>: <message>`;
//! * the console sink — `web_sys::console::error_1`, from the `web-sys`
//!   dependency this crate already declares for the same `wasm32` target.
//!
//! The JS stack capture is kept at parity with the retired crate: a freshly
//! constructed JS `Error` carries the engine's current stack, which is the only
//! way to recover WASM frames in the browser. It is bound through
//! `wasm-bindgen`, also already a declared dependency, so no crate is added.

use std::panic::PanicHookInfo;
use std::sync::Once;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    /// Handle on the JS `Error` constructor, used only to read `.stack`.
    #[wasm_bindgen(js_name = Error)]
    type StackCarrier;

    #[wasm_bindgen(constructor, js_class = "Error")]
    fn new() -> StackCarrier;

    #[wasm_bindgen(method, getter, structural)]
    fn stack(this: &StackCarrier) -> String;
}

static INSTALL_HOOK_ONCE: Once = Once::new();

/// Install the browser panic hook, at most once per module instance.
///
/// Safe to call from every WASM entry point: the second and later calls do
/// nothing, so an entry point can never displace a hook an earlier one set.
pub fn set_once() {
    INSTALL_HOOK_ONCE.call_once(|| {
        std::panic::set_hook(Box::new(report_panic_to_console));
    });
}

/// Render a panic the way the operator console needs to read it: the standard
/// library's own message and location, then the JS-side stack.
fn report_panic_to_console(info: &PanicHookInfo<'_>) {
    let report = format!("{info}\n\nStack:\n\n{}\n\n", StackCarrier::new().stack());
    web_sys::console::error_1(&report.into());
}
