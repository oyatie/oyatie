//! Integration check: the real boot/machine-config.yaml parses and yields the
//! static eth0 address the network boot phase assigns via `add_ipv4`.

use os_init_app::config::early_config;

#[test]
fn boot_machine_config_yields_eth0_static_address() {
    let raw = include_str!("../testdata/machine-config.yaml");
    let ec = early_config(raw);
    assert_eq!(ec.hostname.as_deref(), Some("talos-rust-node-1"));
    let a = ec.first_iface_address.expect("eth0 static address parsed");
    assert_eq!(a.addr, "10.0.2.15");
    assert_eq!(a.prefix, 24);
}
