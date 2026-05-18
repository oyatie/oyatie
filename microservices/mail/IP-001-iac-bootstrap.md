---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + ops-deliverability
acceptance_lanes: [cargo-check, helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap — Postfix + Dovecot + Stalwart + Postgres + S3 + Tantivy + KMS

## Intent

Author Helm + Kustomize manifests for the mail µservice substrate (Layer-A). Two SMTP backends supported per ADR-0105 Amendment 3 `*-adapter-<backend>` (`-adapter-postfix` for compatibility + `-adapter-stalwart` for modern LTS); Dovecot for IMAP4rev2 (`-adapter-dovecot`); Postgres for mailbox metadata (RLS per-tenant per ADR-0117); S3-compatible object storage for MIME blobs; Tantivy for encrypted-token search; OpenBao for per-tenant DEK + DKIM key escrow. Pack-aware overlays for 11 packs.

## ChangeSet boundary

10 Helm chart bundles + Kustomize base + per-pack overlays for the 11 active packs (kr first; eu/us next). No Rust code; pure IaC + values. All secrets via `${openbao:...}` SecretReference (per CLAUDE.md user directive).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/mail/iac/helm/postfix/Chart.yaml` | create | Postfix LTS (3.8.x); upstream `bokysan/docker-postfix` baseline |
| `microservices/mail/iac/helm/postfix/values.yaml` | create | submission :587 + smtps :465 + smtp :25; DKIM via OpenBao mTLS path |
| `microservices/mail/iac/helm/dovecot/Chart.yaml` | create | Dovecot 2.3 LTS; IMAP4rev2 + ManageSieve |
| `microservices/mail/iac/helm/dovecot/values.yaml` | create | per-tenant mailbox path mapping; SASL via OIDC bridge |
| `microservices/mail/iac/helm/stalwart/Chart.yaml` | create | Stalwart Mail Server v0.8+ LTS (modern unified SMTP+IMAP+JMAP alternative) |
| `microservices/mail/iac/helm/stalwart/values.yaml` | create | multi-tenant config; per-tenant DEK; OpenBao integration |
| `microservices/mail/iac/helm/postgres/Chart.yaml` | create | Postgres 16 LTS (CloudNativePG operator) |
| `microservices/mail/iac/helm/postgres/values.yaml` | create | per-tenant RLS; Citus distributed table by tenant_id when scaled past single-node; HA via streaming replication; sync replicas ≥ 1 |
| `microservices/mail/iac/helm/s3-blob/values.yaml` | create | bucket policy + SSE-KMS envelope + versioning + object-lock (HIPAA + KR-FSS) |
| `microservices/mail/iac/helm/tantivy/Chart.yaml` | create | Tantivy 0.22 LTS pinned; per-tenant + per-context partition |
| `microservices/mail/iac/helm/tantivy/values.yaml` | create | encrypted-token index config; PV-backed; backup to S3 |
| `microservices/mail/iac/helm/rspamd/Chart.yaml` | create | Rspamd 3.x LTS for inbound abuse classification |
| `microservices/mail/iac/helm/rspamd/values.yaml` | create | per-tenant rule scope; cluster-mode redis; sieve learning disabled (PII risk) |
| `microservices/mail/iac/helm/openbao-mail/values.yaml` | create | KMS + DKIM key paths + tenant DEK paths |
| `microservices/mail/iac/kustomize/base/kustomization.yaml` | create | shared base referencing all 8 charts |
| `microservices/mail/iac/kustomize/overlays/pack-kr/kustomization.yaml` | create | initial active pack |
| `microservices/mail/iac/kustomize/overlays/pack-eu/kustomization.yaml` | create | eu pack |
| `microservices/mail/iac/kustomize/overlays/pack-us/kustomization.yaml` | create | us pack |
| `microservices/mail/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml` | create | HIPAA pack with stricter encryption + audit retention |
| `microservices/mail/iac/helm/templates/deployment.yaml` | create | shared deployment template (used per BC by app charts) |
| `microservices/mail/iac/helm/templates/service.yaml` | create | shared service template |
| `microservices/mail/iac/helm/templates/hpa.yaml` | create | per-BC HPA (cpu > 70 % min 4 max 50) |
| `microservices/mail/iac/helm/templates/pdb.yaml` | create | PodDisruptionBudget min-available 50 % |
| `microservices/mail/iac/helm/templates/networkpolicy.yaml` | create | mesh-only ingress for IMAP/JMAP/REST; SMTP :25/:465/:587 ingress allowed; egress to OpenBao + Postgres + S3 + recipient MXes |
| `microservices/mail/iac/helm/templates/servicemonitor.yaml` | create | Prometheus scrape config; per-µservice job labels |
| `microservices/mail/iac/helm/templates/prometheusrule.yaml` | create | per-BC fast-burn + slow-burn rules; deliverability + dkim-age alerts |

## Crate Naming

n/a — IaC only.

## Code Shape

`microservices/mail/iac/helm/postfix/values.yaml`:

```yaml
postfix:
  image:
    repository: docker.io/bokysan/postfix
    tag: "3.8.5"  # LTS pinned per docs/standards/observability-slo.md
  smtp:
    ports:
      - {name: smtp,       containerPort: 25,  protocol: TCP}  # inbound
      - {name: smtps,      containerPort: 465, protocol: TCP}  # implicit TLS
      - {name: submission, containerPort: 587, protocol: TCP}  # RFC 6409
    tlsConfig:
      minVersion: "TLSv1.3"          # RFC 8314 SMTP TLS
      certSecretRef: ${openbao:secret/mail/<pack>/mta-tls}
  mta_sts:
    enabled: true                    # RFC 8461
    policyUri: https://mta-sts.${TENANT_DOMAIN}/.well-known/mta-sts.txt
  tls_rpt:
    enabled: true                    # RFC 8460
    rua: mailto:tls-rpt-${pack}@oyatie.dev
  milter:
    rspamd:
      endpoint: rspamd.mail.svc.cluster.local:11332
    dkim:
      signer_endpoint: oya-mail-outbound-smtp-app.mail.svc:9000   # in-cluster mTLS
      key_path_template: ${openbao:secret/mail/{tenant}/dkim/{selector}}
  smtpd_recipient_restrictions:
    - permit_mynetworks
    - reject_unauth_destination
    - check_recipient_access regexp:/etc/postfix/recipient_access
  alias_maps: hash:/etc/postfix/aliases
  size_limit_bytes: 26214400   # 25 MB per RFC 5321 §4.5.3.1.7 baseline
networkPolicy:
  egress:
    - to: [{ipBlock: {cidr: 0.0.0.0/0}}]
      ports: [{port: 25, protocol: TCP}, {port: 465, protocol: TCP}]
```

## Acceptance Gates

```bash
helm lint microservices/mail/iac/helm/postfix
helm lint microservices/mail/iac/helm/dovecot
helm lint microservices/mail/iac/helm/stalwart
helm lint microservices/mail/iac/helm/postgres
helm lint microservices/mail/iac/helm/tantivy
helm lint microservices/mail/iac/helm/rspamd
kubectl --dry-run=client apply -k microservices/mail/iac/kustomize/overlays/pack-kr
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice mail
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

Per PHASE-01 §"Per-IP Test Coverage Threshold" IaC class: ≥ 1 helm-install + helm-test smoke per chart against kind/k3d cluster. Test files under `microservices/mail/tests/iac/{postfix,dovecot,stalwart,...}.bats`.

E2E: spin kind cluster; apply pack-kr overlay; verify all 8 component pods reach Ready within 10 min; send a test mail via Postfix submission :587; observe Postgres mailbox row + S3 blob created + Tantivy index entry.

## Halt Conditions

- Upstream chart version drifts past LTS pin — escalate to `docs/standards/observability-slo.md` PR.
- OpenBao secret-reference resolution fails — block; engage cloud-secrets µservice.
- kind smoke fails — root-cause; do not mask.

## Next IP

[`IP-002-mailbox-store-kernel.md`](IP-002-mailbox-store-kernel.md)

## References

- ADR-0117 (data residency)
- ADR-0131 (per-microservice flat layout)
- ADR-0133 (cross-tenant mail-server pattern)
- RFC 5321 (SMTP), RFC 6409 (Submission), RFC 8314 (TLS), RFC 8461 (MTA-STS), RFC 8460 (TLS-RPT), RFC 9051 (IMAP4rev2)
- Postfix docs — `postfix.org`; Dovecot docs — `dovecot.org`; Stalwart Mail Server — `stalw.art`
- Tantivy — `github.com/quickwit-oss/tantivy`
- Rspamd — `rspamd.com`
- CloudNativePG operator — `cloudnative-pg.io`
