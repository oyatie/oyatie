//! Live HTTP broker-gateway publisher adapter for outbox transport plans.
//!
//! This adapter performs a real TCP HTTP/1.1 POST to a broker gateway endpoint
//! using deterministic outbox metadata as the request body. It intentionally does
//! not implement a vendor-specific Kafka/NATS/Pub/Sub client, TLS/mTLS, service
//! discovery, retries, gRPC execution, database mutation, or delivery SLOs. Its
//! `OutboxTransportExecutor` implementation records an explicit
//! `grpc:not-executed` acknowledgement marker instead of claiming a gRPC call.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use shared_transactional_outbox_dispatch_app::{
    OutboxDispatchAppError, OutboxTransportAck, OutboxTransportExecutor, OutboxTransportPlan,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HttpBrokerPublisherError {
    InvalidConfig {
        field: &'static str,
    },
    InvalidPlan {
        field: &'static str,
    },
    PayloadBudgetExceeded {
        actual_bytes: usize,
        budget_bytes: usize,
    },
    ResolveAddress,
    Io(String),
    InvalidResponse,
    UnexpectedStatus {
        status_code: u16,
        status_line: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpBrokerPublisherConfig {
    pub host: String,            // data_class: INTERNAL_ONLY
    pub port: u16,               // data_class: INTERNAL_ONLY
    pub path_prefix: String,     // data_class: INTERNAL_ONLY
    pub connect_timeout_ms: u64, // data_class: INTERNAL_ONLY
    pub read_timeout_ms: u64,    // data_class: INTERNAL_ONLY
    pub ack_header_name: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpBrokerPublishReport {
    pub ack_ref: String,              // data_class: INTERNAL_ONLY
    pub status_code: u16,             // data_class: INTERNAL_ONLY
    pub request_path: String,         // data_class: INTERNAL_ONLY
    pub operation_id: String,         // data_class: INTERNAL_ONLY
    pub channel_address: String,      // data_class: INTERNAL_ONLY
    pub event_id: String,             // data_class: INTERNAL_ONLY
    pub payload_bytes: usize,         // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,     // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String, // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpBrokerPublisher {
    config: HttpBrokerPublisherConfig,
    sequence: u64,
}

impl Default for HttpBrokerPublisherConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            path_prefix: "/broker/outbox".to_string(),
            connect_timeout_ms: 500,
            read_timeout_ms: 500,
            ack_header_name: "x-oya-broker-ack-ref".to_string(),
        }
    }
}

impl HttpBrokerPublisherConfig {
    pub fn validate(&self) -> Result<(), HttpBrokerPublisherError> {
        require_header_safe("host", &self.host)?;
        if self.port == 0 {
            return Err(HttpBrokerPublisherError::InvalidConfig { field: "port" });
        }
        if self.connect_timeout_ms == 0 {
            return Err(HttpBrokerPublisherError::InvalidConfig {
                field: "connect_timeout_ms",
            });
        }
        if self.read_timeout_ms == 0 {
            return Err(HttpBrokerPublisherError::InvalidConfig {
                field: "read_timeout_ms",
            });
        }
        require_path_safe("path_prefix", &self.path_prefix)?;
        require_header_name_safe("ack_header_name", &self.ack_header_name)?;
        Ok(())
    }
}

impl HttpBrokerPublisher {
    pub fn new(config: HttpBrokerPublisherConfig) -> Result<Self, HttpBrokerPublisherError> {
        config.validate()?;
        Ok(Self {
            config,
            sequence: 0,
        })
    }

    pub fn publish(
        &mut self,
        plan: &OutboxTransportPlan,
    ) -> Result<HttpBrokerPublishReport, HttpBrokerPublisherError> {
        self.config.validate()?;
        validate_plan(plan)?;
        let payload = encode_broker_payload(plan)?;
        if payload.len() > plan.broker_publish.max_payload_bytes {
            return Err(HttpBrokerPublisherError::PayloadBudgetExceeded {
                actual_bytes: payload.len(),
                budget_bytes: plan.broker_publish.max_payload_bytes,
            });
        }
        let request_path = broker_request_path(
            &self.config.path_prefix,
            &plan.broker_publish.channel_address,
        )?;
        let request = render_http_request(&self.config, plan, &request_path, &payload)?;
        let response = send_http_request(&self.config, request.as_bytes())?;
        let (status_code, status_line, ack_header) =
            parse_http_response(&response, &self.config.ack_header_name)?;
        if !(200..=299).contains(&status_code) {
            return Err(HttpBrokerPublisherError::UnexpectedStatus {
                status_code,
                status_line,
            });
        }
        self.sequence = self.sequence.saturating_add(1);
        let ack_ref = ack_header.unwrap_or_else(|| {
            format!(
                "http-broker:{}:{}",
                plan.broker_publish.operation_id, self.sequence
            )
        });
        Ok(HttpBrokerPublishReport {
            ack_ref,
            status_code,
            request_path,
            operation_id: plan.broker_publish.operation_id.clone(),
            channel_address: plan.broker_publish.channel_address.clone(),
            event_id: plan.event_id.clone(),
            payload_bytes: payload.len(),
            tenant_scope_ref: plan.broker_publish.headers.tenant_scope_ref.clone(),
            audit_correlation_id: plan.broker_publish.headers.audit_correlation_id.clone(),
            policy_decision_ref: plan.broker_publish.headers.policy_decision_ref.clone(),
        })
    }
}

impl OutboxTransportExecutor for HttpBrokerPublisher {
    fn execute_outbox_transport(
        &mut self,
        plan: &OutboxTransportPlan,
    ) -> Result<OutboxTransportAck, OutboxDispatchAppError> {
        let report = self
            .publish(plan)
            .map_err(map_http_publisher_error_to_dispatch_error)?;
        Ok(OutboxTransportAck {
            sequence: self.sequence,
            event_id: report.event_id,
            broker_ack_ref: report.ack_ref,
            grpc_ack_ref: format!(
                "grpc:not-executed:{}:{}",
                plan.grpc_unary.fully_qualified_method, self.sequence
            ),
            tenant_scope_ref: report.tenant_scope_ref,
            audit_correlation_id: report.audit_correlation_id,
            policy_decision_ref: report.policy_decision_ref,
            idempotency_key: plan.broker_publish.headers.idempotency_key.clone(),
        })
    }
}

pub fn encode_broker_payload(
    plan: &OutboxTransportPlan,
) -> Result<Vec<u8>, HttpBrokerPublisherError> {
    validate_plan(plan)?;
    let mut out = String::new();
    let mut first = true;
    out.push('{');
    push_json_string_field(&mut out, &mut first, "event_id", &plan.event_id);
    push_json_string_field(&mut out, &mut first, "service_id", &plan.service_id);
    push_json_string_field(
        &mut out,
        &mut first,
        "operation_id",
        &plan.broker_publish.operation_id,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "channel_address",
        &plan.broker_publish.channel_address,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "message_name",
        &plan.broker_publish.message_name,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "event_kind",
        &plan.broker_publish.event_kind,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "partition_key",
        &plan.broker_publish.partition_key,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "payload_encoding",
        plan.broker_publish.payload_encoding,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "tenant_scope_ref",
        &plan.broker_publish.headers.tenant_scope_ref,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "audit_correlation_id",
        &plan.broker_publish.headers.audit_correlation_id,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "policy_decision_ref",
        &plan.broker_publish.headers.policy_decision_ref,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "schema_version",
        &plan.broker_publish.headers.schema_version,
    );
    if let Some(idempotency_key) = &plan.broker_publish.headers.idempotency_key {
        push_json_string_field(&mut out, &mut first, "idempotency_key", idempotency_key);
    }
    push_json_string_field(
        &mut out,
        &mut first,
        "grpc_method",
        &plan.grpc_unary.fully_qualified_method,
    );
    out.push('}');
    Ok(out.into_bytes())
}

fn render_http_request(
    config: &HttpBrokerPublisherConfig,
    plan: &OutboxTransportPlan,
    request_path: &str,
    payload: &[u8],
) -> Result<String, HttpBrokerPublisherError> {
    let host_header = format!("{}:{}", config.host, config.port);
    require_header_safe("host", &host_header)?;
    let headers = &plan.broker_publish.headers;
    for (field, value) in [
        ("operation_id", plan.broker_publish.operation_id.as_str()),
        ("event_kind", plan.broker_publish.event_kind.as_str()),
        ("message_name", plan.broker_publish.message_name.as_str()),
        ("partition_key", plan.broker_publish.partition_key.as_str()),
        ("tenant_scope_ref", headers.tenant_scope_ref.as_str()),
        (
            "audit_correlation_id",
            headers.audit_correlation_id.as_str(),
        ),
        ("policy_decision_ref", headers.policy_decision_ref.as_str()),
        ("schema_version", headers.schema_version.as_str()),
    ] {
        require_header_safe(field, value)?;
    }
    if let Some(idempotency_key) = &headers.idempotency_key {
        require_header_safe("idempotency_key", idempotency_key)?;
    }
    let mut request = format!(
        "POST {request_path} HTTP/1.1\r\n\
         Host: {host_header}\r\n\
         Content-Type: application/vnd.oyatie.outbox+json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         X-Oya-Operation-Id: {}\r\n\
         X-Oya-Event-Kind: {}\r\n\
         X-Oya-Message-Name: {}\r\n\
         X-Oya-Partition-Key: {}\r\n\
         X-Oya-Tenant-Scope-Ref: {}\r\n\
         X-Oya-Audit-Correlation-Id: {}\r\n\
         X-Oya-Policy-Decision-Ref: {}\r\n\
         X-Oya-Schema-Version: {}\r\n",
        payload.len(),
        plan.broker_publish.operation_id,
        plan.broker_publish.event_kind,
        plan.broker_publish.message_name,
        plan.broker_publish.partition_key,
        headers.tenant_scope_ref,
        headers.audit_correlation_id,
        headers.policy_decision_ref,
        headers.schema_version,
    );
    if let Some(idempotency_key) = &headers.idempotency_key {
        request.push_str("X-Oya-Idempotency-Key: ");
        request.push_str(idempotency_key);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");
    request.push_str(&String::from_utf8_lossy(payload));
    Ok(request)
}

fn send_http_request(
    config: &HttpBrokerPublisherConfig,
    request: &[u8],
) -> Result<String, HttpBrokerPublisherError> {
    let addr = resolve_addr(config)?;
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_millis(config.connect_timeout_ms))
            .map_err(|error| HttpBrokerPublisherError::Io(error.to_string()))?;
    let read_timeout = Some(Duration::from_millis(config.read_timeout_ms));
    stream
        .set_read_timeout(read_timeout)
        .map_err(|error| HttpBrokerPublisherError::Io(error.to_string()))?;
    stream
        .set_write_timeout(read_timeout)
        .map_err(|error| HttpBrokerPublisherError::Io(error.to_string()))?;
    stream
        .write_all(request)
        .map_err(|error| HttpBrokerPublisherError::Io(error.to_string()))?;
    stream
        .flush()
        .map_err(|error| HttpBrokerPublisherError::Io(error.to_string()))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| HttpBrokerPublisherError::Io(error.to_string()))?;
    Ok(response)
}

fn resolve_addr(
    config: &HttpBrokerPublisherConfig,
) -> Result<SocketAddr, HttpBrokerPublisherError> {
    (config.host.as_str(), config.port)
        .to_socket_addrs()
        .map_err(|_| HttpBrokerPublisherError::ResolveAddress)?
        .next()
        .ok_or(HttpBrokerPublisherError::ResolveAddress)
}

fn parse_http_response(
    response: &str,
    ack_header_name: &str,
) -> Result<(u16, String, Option<String>), HttpBrokerPublisherError> {
    let mut lines = response.split("\r\n");
    let status_line = lines
        .next()
        .filter(|line| line.starts_with("HTTP/1."))
        .ok_or(HttpBrokerPublisherError::InvalidResponse)?
        .to_string();
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(HttpBrokerPublisherError::InvalidResponse)?;
    let mut ack = None;
    for line in lines {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(HttpBrokerPublisherError::InvalidResponse);
        };
        if name.eq_ignore_ascii_case(ack_header_name) {
            let value = value.trim();
            require_header_safe("ack_header", value)?;
            ack = Some(value.to_string());
        }
    }
    Ok((status_code, status_line, ack))
}

fn validate_plan(plan: &OutboxTransportPlan) -> Result<(), HttpBrokerPublisherError> {
    for (field, value) in [
        ("event_id", plan.event_id.as_str()),
        ("service_id", plan.service_id.as_str()),
        ("operation_id", plan.broker_publish.operation_id.as_str()),
        (
            "channel_address",
            plan.broker_publish.channel_address.as_str(),
        ),
        ("event_kind", plan.broker_publish.event_kind.as_str()),
        ("partition_key", plan.broker_publish.partition_key.as_str()),
        (
            "tenant_scope_ref",
            plan.broker_publish.headers.tenant_scope_ref.as_str(),
        ),
        (
            "audit_correlation_id",
            plan.broker_publish.headers.audit_correlation_id.as_str(),
        ),
        (
            "policy_decision_ref",
            plan.broker_publish.headers.policy_decision_ref.as_str(),
        ),
        (
            "schema_version",
            plan.broker_publish.headers.schema_version.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(HttpBrokerPublisherError::InvalidPlan { field });
        }
        require_plan_header_safe(field, value)?;
    }
    require_plan_channel_address_safe("channel_address", &plan.broker_publish.channel_address)?;
    if plan.broker_publish.max_payload_bytes == 0 {
        return Err(HttpBrokerPublisherError::InvalidPlan {
            field: "max_payload_bytes",
        });
    }
    Ok(())
}

fn broker_request_path(
    prefix: &str,
    channel_address: &str,
) -> Result<String, HttpBrokerPublisherError> {
    require_path_safe("path_prefix", prefix)?;
    require_channel_address_safe("channel_address", channel_address)?;
    let prefix = prefix.trim_end_matches('/');
    let channel = channel_address.trim_start_matches('/');
    Ok(format!("{prefix}/{channel}"))
}

fn map_http_publisher_error_to_dispatch_error(
    error: HttpBrokerPublisherError,
) -> OutboxDispatchAppError {
    match error {
        HttpBrokerPublisherError::PayloadBudgetExceeded {
            actual_bytes,
            budget_bytes,
        } => OutboxDispatchAppError::PayloadBudgetExceeded {
            actual_bytes,
            budget_bytes,
        },
        other => OutboxDispatchAppError::TransportExecutionFailed {
            transport: "http-broker-gateway",
            error_ref: format!("{other:?}"),
        },
    }
}

fn require_header_name_safe(
    field: &'static str,
    value: &str,
) -> Result<(), HttpBrokerPublisherError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'-'))
    {
        Err(HttpBrokerPublisherError::InvalidConfig { field })
    } else {
        Ok(())
    }
}

fn require_header_safe(field: &'static str, value: &str) -> Result<(), HttpBrokerPublisherError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte == b'\r' || byte == b'\n' || byte == 0)
    {
        Err(HttpBrokerPublisherError::InvalidConfig { field })
    } else {
        Ok(())
    }
}

fn require_plan_header_safe(
    field: &'static str,
    value: &str,
) -> Result<(), HttpBrokerPublisherError> {
    if value.trim().is_empty()
        || value
            .bytes()
            .any(|byte| byte == b'\r' || byte == b'\n' || byte == 0)
    {
        Err(HttpBrokerPublisherError::InvalidPlan { field })
    } else {
        Ok(())
    }
}

fn require_path_safe(field: &'static str, value: &str) -> Result<(), HttpBrokerPublisherError> {
    if value.trim().is_empty()
        || !value.starts_with('/')
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(HttpBrokerPublisherError::InvalidConfig { field })
    } else {
        Ok(())
    }
}

fn require_channel_address_safe(
    field: &'static str,
    value: &str,
) -> Result<(), HttpBrokerPublisherError> {
    if value.trim().is_empty()
        || value.starts_with("//")
        || value.contains("://")
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(HttpBrokerPublisherError::InvalidConfig { field })
    } else {
        Ok(())
    }
}

fn require_plan_channel_address_safe(
    field: &'static str,
    value: &str,
) -> Result<(), HttpBrokerPublisherError> {
    if value.trim().is_empty()
        || value.starts_with("//")
        || value.contains("://")
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Err(HttpBrokerPublisherError::InvalidPlan { field })
    } else {
        Ok(())
    }
}

fn push_json_string_field(out: &mut String, first: &mut bool, name: &str, value: &str) {
    if !*first {
        out.push(',');
    }
    *first = false;
    push_json_string(out, name);
    out.push(':');
    push_json_string(out, value);
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_transactional_outbox_dispatch_app::{
        OutboxBrokerHeaders, OutboxBrokerPublishPlan, OutboxGrpcUnaryPlan, OutboxTransportPlan,
    };
    use shared_transactional_outbox_kernel::BackboneOutboxTable;
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    #[derive(Debug)]
    struct CapturedRequest {
        request: String,
        body: String,
    }

    #[test]
    fn publisher_posts_outbox_metadata_to_live_http_broker_gateway() {
        let (config, captured) =
            broker_server("202 Accepted", Some(("X-Oya-Broker-Ack-Ref", "ack:http:1")));
        let mut publisher = HttpBrokerPublisher::new(config).unwrap();

        let report = publisher.publish(&plan()).unwrap();
        let captured = captured.recv().unwrap();

        assert_eq!(report.ack_ref, "ack:http:1");
        assert_eq!(report.status_code, 202);
        assert_eq!(
            report.request_path,
            "/broker/outbox/workflow-events/messenger.message.posted"
        );
        assert!(
            captured.request.starts_with(
                "POST /broker/outbox/workflow-events/messenger.message.posted HTTP/1.1"
            )
        );
        assert!(
            captured
                .request
                .contains("X-Oya-Tenant-Scope-Ref: tenant:t")
        );
        assert!(
            captured
                .request
                .contains("Content-Type: application/vnd.oyatie.outbox+json")
        );
        assert!(
            captured
                .body
                .contains("\"event_kind\":\"oya.messenger.message.posted.v1\"")
        );
        assert!(
            captured
                .body
                .contains("\"grpc_method\":\"/oya.messenger.v1.MessageStream/PostMessage\"")
        );
        assert_eq!(report.payload_bytes, captured.body.len());
    }

    #[test]
    fn publisher_rejects_non_success_status_without_ack_claim() {
        let (config, captured) = broker_server("503 Service Unavailable", None);
        let mut publisher = HttpBrokerPublisher::new(config).unwrap();

        let err = publisher.publish(&plan()).unwrap_err();

        assert!(captured.recv().unwrap().body.contains("event:e"));
        assert_eq!(
            err,
            HttpBrokerPublisherError::UnexpectedStatus {
                status_code: 503,
                status_line: "HTTP/1.1 503 Service Unavailable".to_string(),
            }
        );
    }

    #[test]
    fn publisher_implements_outbox_transport_executor_with_http_ack_only() {
        let (config, captured) = broker_server(
            "202 Accepted",
            Some(("X-Oya-Broker-Ack-Ref", "ack:http:executor")),
        );
        let mut publisher = HttpBrokerPublisher::new(config).unwrap();

        let ack = OutboxTransportExecutor::execute_outbox_transport(&mut publisher, &plan())
            .expect("HTTP broker executor should publish over local TCP");
        let captured = captured.recv().unwrap();

        assert_eq!(ack.sequence, 1);
        assert_eq!(ack.event_id, "event:e");
        assert_eq!(ack.broker_ack_ref, "ack:http:executor");
        assert_eq!(
            ack.grpc_ack_ref,
            "grpc:not-executed:/oya.messenger.v1.MessageStream/PostMessage:1"
        );
        assert_eq!(ack.idempotency_key, Some("idem:i".into()));
        assert!(
            captured
                .request
                .starts_with("POST /broker/outbox/workflow-events/messenger.message.posted")
        );
    }

    #[test]
    fn executor_maps_http_failures_to_dispatch_errors() {
        let (config, _captured) = broker_server("503 Service Unavailable", None);
        let mut publisher = HttpBrokerPublisher::new(config).unwrap();

        let err = OutboxTransportExecutor::execute_outbox_transport(&mut publisher, &plan())
            .expect_err("HTTP status failure should be visible to worker dead-letter path");

        assert_eq!(
            err,
            OutboxDispatchAppError::TransportExecutionFailed {
                transport: "http-broker-gateway",
                error_ref: "UnexpectedStatus { status_code: 503, status_line: \"HTTP/1.1 503 Service Unavailable\" }".into(),
            }
        );
    }

    #[test]
    fn config_and_plan_validation_fail_closed_before_network_io() {
        let config = HttpBrokerPublisherConfig {
            host: "bad\r\nhost".into(),
            ..HttpBrokerPublisherConfig::default()
        };
        assert_eq!(
            HttpBrokerPublisher::new(config),
            Err(HttpBrokerPublisherError::InvalidConfig { field: "host" })
        );

        let mut invalid_plan = plan();
        invalid_plan.broker_publish.channel_address = "bad channel".into();
        assert_eq!(
            encode_broker_payload(&invalid_plan),
            Err(HttpBrokerPublisherError::InvalidPlan {
                field: "channel_address"
            })
        );

        let mut absolute_url_plan = plan();
        absolute_url_plan.broker_publish.channel_address = "https://broker.example/events".into();
        assert_eq!(
            encode_broker_payload(&absolute_url_plan),
            Err(HttpBrokerPublisherError::InvalidPlan {
                field: "channel_address"
            })
        );

        let mut small = plan();
        small.broker_publish.max_payload_bytes = 1;
        let (config, _captured) = broker_server("202 Accepted", None);
        let mut publisher = HttpBrokerPublisher::new(config).unwrap();
        assert!(matches!(
            publisher.publish(&small),
            Err(HttpBrokerPublisherError::PayloadBudgetExceeded { .. })
        ));
    }

    fn broker_server(
        status: &'static str,
        ack_header: Option<(&'static str, &'static str)>,
    ) -> (HttpBrokerPublisherConfig, mpsc::Receiver<CapturedRequest>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 8192];
            let read = stream.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();
            let body = request
                .split("\r\n\r\n")
                .nth(1)
                .unwrap_or_default()
                .to_string();
            tx.send(CapturedRequest {
                request: request.clone(),
                body,
            })
            .unwrap();
            let mut response =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n");
            if let Some((name, value)) = ack_header {
                response.push_str(name);
                response.push_str(": ");
                response.push_str(value);
                response.push_str("\r\n");
            }
            response.push_str("\r\n");
            stream.write_all(response.as_bytes()).unwrap();
        });
        (
            HttpBrokerPublisherConfig {
                host: "127.0.0.1".into(),
                port,
                path_prefix: "/broker/outbox".into(),
                connect_timeout_ms: 500,
                read_timeout_ms: 500,
                ack_header_name: "x-oya-broker-ack-ref".into(),
            },
            rx,
        )
    }

    fn plan() -> OutboxTransportPlan {
        OutboxTransportPlan {
            table: BackboneOutboxTable::MessengerMessageStream,
            table_name: BackboneOutboxTable::MessengerMessageStream.table_name(),
            service_id: "messenger-message-stream".into(),
            event_id: "event:e".into(),
            broker_publish: OutboxBrokerPublishPlan {
                operation_id: "emitMessagePosted".into(),
                channel_address: "workflow-events/messenger.message.posted".into(),
                message_name: "MessagePosted".into(),
                event_kind: "oya.messenger.message.posted.v1".into(),
                partition_key: "message:m".into(),
                payload_encoding: "outbox-metadata-proto-json-v1",
                payload_bytes: 128,
                max_payload_bytes: 4096,
                headers: OutboxBrokerHeaders {
                    tenant_scope_ref: "tenant:t".into(),
                    audit_correlation_id: "audit:a".into(),
                    idempotency_key: Some("idem:i".into()),
                    policy_decision_ref: "policy:p".into(),
                    schema_version: "1.0.0".into(),
                },
            },
            grpc_unary: OutboxGrpcUnaryPlan {
                package: "oya.messenger.v1".into(),
                service: "MessageStream".into(),
                rpc: "PostMessage".into(),
                fully_qualified_method: "/oya.messenger.v1.MessageStream/PostMessage".into(),
                tenant_scope_ref: "tenant:t".into(),
                audit_correlation_id: "audit:a".into(),
                idempotency_key: Some("idem:i".into()),
                policy_decision_ref: "policy:p".into(),
                deadline_ms: 250,
            },
        }
    }
}
