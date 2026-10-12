#[path = "human_status/denials.rs"]
mod denials;
mod facade_support;
#[path = "human_status/http_transport.rs"]
mod http_transport;
#[path = "human_status/support.rs"]
mod human_support;
mod out_of_band;

mod failing_log;
#[path = "human_status/observation.rs"]
mod observation;
