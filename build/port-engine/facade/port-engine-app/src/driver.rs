//! Facade driver wiring: composes kernel entry points with W0-B adapters.
//!
//! Slice 4 wires frontend-go snapshot decode; Slice 6 lands CLI and receipt end-to-end tests.

use port_engine_api::w0_ready as api_ready;
use port_engine_frontend_go::w0_ready as frontend_ready;
use port_engine_rust_ir::{EmptyRenderer, RustIr};
use port_engine_source_pin::{load_embedded, receipt_pin};

/// Slice 4 readiness: api + pin + rust-ir stub + frontend-go decode wired.
pub const fn w0_ready() -> bool {
    api_ready()
        && port_engine_source_pin::w0_ready()
        && port_engine_rust_ir::w0_ready()
        && frontend_ready()
}

/// Load the fleet upstream pin (adapter boundary).
///
/// # Errors
/// Propagates [`port_engine_source_pin::PinError`] from the pin loader.
pub fn fleet_pin() -> Result<String, port_engine_source_pin::PinError> {
    let pin = load_embedded()?;
    Ok(receipt_pin(&pin))
}

/// Smoke the render seam with the Slice 3 empty renderer stub.
///
/// # Errors
/// Propagates [`port_engine_api::PortError`] from [`port_engine_kernel::emit`].
pub fn smoke_render_stub() -> Result<(), port_engine_api::PortError> {
    let ir = RustIr::new(&["stub"]);
    let renderer = EmptyRenderer::new("slice3-fmt-stub");
    let _ = port_engine_kernel::emit(&renderer, &ir)?;
    Ok(())
}

/// Re-export neutral kernel entry points for downstream CLI wiring (Slice 6).
pub use port_engine_kernel::{emit, plan, verify, Verdict};

/// Adapter readiness snapshot for diagnostics.
#[must_use]
pub fn adapter_readiness() -> (bool, bool, bool) {
    (
        port_engine_source_pin::w0_ready(),
        port_engine_rust_ir::w0_ready(),
        frontend_ready(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice4_driver_wiring_is_ready() {
        assert!(w0_ready());
        fleet_pin().expect("fleet pin must load");
        smoke_render_stub().expect("empty renderer stub must emit");
        let (_pin, rust_ir, frontend) = adapter_readiness();
        assert!(rust_ir);
        assert!(frontend, "frontend-go decode lands in Slice 4");
    }
}
