pub struct Mapping {
    pub upstream: &'static str,
    pub core: &'static str,
    pub ports: &'static str,
    pub adapters: &'static str,
    pub facade: &'static str,
    pub owner: &'static str,
}

pub const MAPPINGS: &[Mapping] = &[
    Mapping {
        upstream: "crates/main",
        core: "",
        ports: "",
        adapters: "",
        facade: "mail-app: configuration, listener composition, shutdown, standalone and hosted bindings",
        owner: "app/mail",
    },
    Mapping {
        upstream: "crates/types",
        core: "mail-domain: account, mailbox, message, UID, keyword, quota and state invariants",
        ports: "opaque account and blob references at boundaries",
        adapters: "wire-specific identifiers and property encodings",
        facade: "",
        owner: "app/mail; shared types remain with their semantic owner",
    },
    Mapping {
        upstream: "crates/email",
        core: "mail-usecase: delivery, mailbox, message, identity, submission, rules and push state",
        ports: "mail-store, blob-store, search, identity, policy, clock and event contracts",
        adapters: "MIME parsing via permissively licensed mail-parser",
        facade: "",
        owner: "app/mail",
    },
    Mapping {
        upstream: "crates/smtp",
        core: "delivery-domain/usecase: envelope rules, retry, queue transitions, routing, DSN and reports",
        ports: "queue, DNS, transport, signing, filtering, clock and mail-store",
        adapters: "smtp-inbound, smtp-submission, smtp-outbound, DNS, DKIM/SPF/DMARC/ARC and report codecs",
        facade: "mail-app listener and delivery-worker composition",
        owner: "app/mail; secrets/network through ports",
    },
    Mapping {
        upstream: "crates/imap",
        core: "mail-usecase: selection, flags, copy/move, search, expunge and synchronization",
        ports: "mail-store, identity, policy, search and change-feed",
        adapters: "imap-server: session state, commands and response encoding",
        facade: "mail-app listener composition",
        owner: "app/mail",
    },
    Mapping {
        upstream: "crates/imap-proto",
        core: "",
        ports: "",
        adapters: "imap-codec: framing, literals, UTF-7, parsing and serialization",
        facade: "",
        owner: "app/mail",
    },
    Mapping {
        upstream: "crates/jmap",
        core: "mail-usecase and owner usecases for calendar/contact/file operations",
        ports: "mail, calendar, contacts, files, identity, policy, blob and change-feed contracts",
        adapters: "jmap-server: method dispatch, result references, errors, HTTP and WebSocket binding",
        facade: "mail-app composes product adapters",
        owner: "app/mail; calendar -> app/calendar; files -> app/drive; contacts -> app/mail contact-domain",
    },
    Mapping {
        upstream: "crates/jmap-proto",
        core: "",
        ports: "",
        adapters: "jmap-codec: JSON DTOs, request validation, patch/reference resolution and responses",
        facade: "",
        owner: "app/mail",
    },
    Mapping {
        upstream: "crates/pop3",
        core: "mail-usecase: mailbox snapshot and atomic deletion on session completion",
        ports: "mail-store, identity and policy",
        adapters: "pop3-server: commands, framing, SASL and STLS",
        facade: "mail-app listener composition",
        owner: "app/mail",
    },
    Mapping {
        upstream: "crates/managesieve",
        core: "sieve-usecase: script validation, activation, quota and lifecycle",
        ports: "script-store, sieve-engine, identity and policy",
        adapters: "managesieve-server and permissively licensed sieve engine",
        facade: "mail-app listener composition",
        owner: "app/mail",
    },
    Mapping {
        upstream: "crates/dav",
        core: "calendar scheduling, contact and file usecases at their product owners",
        ports: "calendar, contacts, files, locks, identity and policy",
        adapters: "caldav, carddav and webdav request handlers",
        facade: "collaboration listener composition",
        owner: "app/calendar, app/drive; contacts -> app/mail contact-domain",
    },
    Mapping {
        upstream: "crates/dav-proto",
        core: "",
        ports: "",
        adapters: "dav-codec: XML, DAV properties, request and response DTOs",
        facade: "",
        owner: "app/mail wire adapters; calendar/drive retain domain authority",
    },
    Mapping {
        upstream: "crates/groupware",
        core: "calendar, contacts, file metadata, scheduling and sharing rules",
        ports: "owner stores, policy, notification and change-feed",
        adapters: "calendar/contact parsers and storage bindings",
        facade: "product-specific compositions",
        owner: "app/calendar, app/drive; contacts -> app/mail contact-domain",
    },
    Mapping {
        upstream: "crates/http",
        core: "administration usecases own changes, never the router",
        ports: "product administration, identity and policy contracts",
        adapters: "management REST, OAuth HTTP, discovery/autoconfig and static serving",
        facade: "mail-app plus Console and Oyatie shell views",
        owner: "app/mail; iam for authentication authority",
    },
    Mapping {
        upstream: "crates/http-proto",
        core: "",
        ports: "",
        adapters: "HTTP boundary: request context, limits, content negotiation, error mapping",
        facade: "",
        owner: "app/mail; reuse installed HTTP runtime",
    },
    Mapping {
        upstream: "crates/scim",
        core: "identity provisioning and group lifecycle at IAM owner",
        ports: "IAM identity/provisioning and mail provisioning contracts",
        adapters: "scim-server and mail-provisioning projection",
        facade: "hosted IAM or standalone identity composition",
        owner: "iam; app/mail consumes identity",
    },
    Mapping {
        upstream: "crates/scim-proto",
        core: "",
        ports: "",
        adapters: "SCIM JSON, filtering, patch paths, schema and error DTOs",
        facade: "",
        owner: "iam",
    },
    Mapping {
        upstream: "crates/directory",
        core: "mail recipient/alias/domain routing; identity lifecycle stays in IAM",
        ports: "recipient-directory, identity, group membership and policy",
        adapters: "local, LDAP, OIDC, SQL and Oyatie IAM bindings",
        facade: "standalone/hosted adapter selection",
        owner: "app/mail + iam; no second hosted identity authority",
    },
    Mapping {
        upstream: "crates/store",
        core: "transaction, quota, retention and state invariants only",
        ports: "metadata, immutable blobs, search, queue, change-log and backup contracts",
        adapters: "SQLite, PostgreSQL, MySQL, RocksDB, FoundationDB, object store, Redis and search engines",
        facade: "storage binding and maintenance composition",
        owner: "app/mail owns mail semantics; storage owns infrastructure",
    },
    Mapping {
        upstream: "crates/coordinator",
        core: "mail shard placement, leases and delivery ownership invariants",
        ports: "coordination, lease, clock and event contracts",
        adapters: "peer, NATS, Kafka/Redpanda, Redis and Zenoh bindings",
        facade: "cluster bootstrap and worker composition",
        owner: "app/mail consumes bus/cell/network contracts",
    },
    Mapping {
        upstream: "crates/registry",
        core: "validated configuration, domain/account/admin state and policy requirements",
        ports: "configuration repository, secrets, identity and audit",
        adapters: "configuration persistence, JMAP/REST admin DTOs and secret resolution",
        facade: "boot validation and config application",
        owner: "app/mail; platform authorities retain their own state",
    },
    Mapping {
        upstream: "crates/services",
        core: "scheduled tasks, retention, maintenance and state-change usecases",
        ports: "clock, scheduler, jobs, change-feed and outbox",
        adapters: "workers, broadcast transport and push delivery",
        facade: "worker lifecycle, scheduling and graceful drain",
        owner: "app/mail + collaboration owners",
    },
    Mapping {
        upstream: "crates/spam-filter",
        core: "filter decisions, scoring, quarantine, reputation and training rules",
        ports: "scanner, DNSBL, classifier, reputation, policy and model-inference contracts",
        adapters: "Pyzor, milter, HTTP hooks, DNS and LLM service bindings",
        facade: "pipeline configuration",
        owner: "app/mail; intelligence consumes bounded inference requests",
    },
    Mapping {
        upstream: "crates/nlp",
        core: "mail-specific classifier rules only",
        ports: "search/tokenizer/classifier boundary where interchangeable",
        adapters: "language detection, tokenization and classifier implementation",
        facade: "",
        owner: "mail search/filter adapter; reuse only verified permissive material",
    },
    Mapping {
        upstream: "crates/common",
        core: "split mail policy, routing, retention and configuration invariants by owner",
        ports: "identity, policy, storage, signing, DNS, events, telemetry and inference",
        adapters: "auth, TLS, cache, scripts, expressions, telemetry, alerts and masked-address bindings",
        facade: "runtime/configuration composition; no common service locator",
        owner: "split across app/mail, iam, policy, secrets, observability and intelligence",
    },
    Mapping {
        upstream: "crates/trc",
        core: "typed domain errors and audit event definitions stay with their owner",
        ports: "audit and telemetry contracts",
        adapters: "OpenTelemetry, Prometheus, trace/metric persistence and serializers",
        facade: "subscriber and exporter setup",
        owner: "observability + audit; mail provides semantic events",
    },
    Mapping {
        upstream: "crates/utils",
        core: "only owner-specific pure algorithms",
        ports: "only actual external seams",
        adapters: "TLS, HTTP, codecs, caches and templates; stdlib/dependencies first",
        facade: "",
        owner: "distribute by responsibility; no generic utils crate",
    },
    Mapping {
        upstream: "crates/migration",
        core: "mail format transformations, compatibility and rollback validation",
        ports: "source-reader, destination-store, backup and migration journal",
        adapters: "Stalwart export/import and storage format readers",
        facade: "mail-admin-app migration commands",
        owner: "app/mail; upstream version labels only at compatibility boundary",
    },
    Mapping {
        upstream: "crates/trc/event-macro",
        core: "",
        ports: "",
        adapters: "use existing tracing macros; add code generation only if required",
        facade: "",
        owner: "build tooling; no automatic macro reimplementation",
    },
    Mapping {
        upstream: "crates/utils/proc-macros",
        core: "",
        ports: "",
        adapters: "derive/macros from installed dependencies where equivalent",
        facade: "",
        owner: "build tooling; inspect each expansion before replacement",
    },
    Mapping {
        upstream: "tests",
        core: "domain and usecase regression tests",
        ports: "adapter conformance and fault contracts",
        adapters: "external upstream assertion harnesses and wire interoperability tests",
        facade: "parity runner, recovery/load/client/admin qualification",
        owner: "test code remains outside production dependency graph",
    },
];

pub fn mapping(path: &str) -> Option<&'static Mapping> {
    MAPPINGS.iter().find(|m| m.upstream == path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_mapping_has_a_unique_source_owner_and_destination() {
        let mut seen = std::collections::BTreeSet::new();
        for m in MAPPINGS {
            assert!(seen.insert(m.upstream), "duplicate upstream mapping");
            assert!(!m.owner.is_empty());
            assert!(
                [m.core, m.ports, m.adapters, m.facade]
                    .iter()
                    .any(|s| !s.is_empty())
            );
        }
        assert_eq!(MAPPINGS.len(), 31);
        assert!(mapping("crates/new-upstream-feature").is_none());
        assert!(mapping("crates/imap-proto").unwrap().core.is_empty());
        assert!(mapping("crates/main").unwrap().adapters.is_empty());
    }
}
