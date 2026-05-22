---
doc_class: ReferenceImplementation
microservice: healthcare-integration
language: rust
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Reference — Search + read FHIR Patient resources with consent-cascade enforcement (Rust SDK)

This walkthrough shows a tenant's clinical-app server-side worker querying FHIR Patient + Encounter resources from oyatie healthcare-integration with full HIPAA + consent-cascade enforcement, automatic IHE ATNA audit emission, and Smart-on-FHIR launch context.

## Cargo.toml

```toml
[package]
name = "clinical-app-server"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-fhir-sdk = { path = "../../crates/oya-fhir-sdk" }
oya-iam-sdk = { path = "../../crates/oya-iam-sdk" }
oya-observability-sdk = { path = "../../crates/oya-observability-sdk" }
tokio = { version = "1.42", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
fhir-model-r5 = "0.10"
```

## src/main.rs

```rust
use anyhow::{anyhow, Context};
use chrono::{Duration, Utc};
use fhir_model_r5::{Bundle, Encounter, Patient};
use oya_fhir_sdk::{ConsentCascadeError, FhirClient, SearchParams, SmartLaunchContext};
use oya_iam_sdk::{IamClient, Principal};
use oya_observability_sdk::ObservabilityGuard;
use tracing::{error, info, warn};

struct ClinicalAppServer {
    fhir_client: FhirClient,
    iam_client: IamClient,
    tenant_id: String,
}

impl ClinicalAppServer {
    async fn launch_for_patient(
        &self,
        launch_context: SmartLaunchContext,
    ) -> anyhow::Result<PatientSummary> {
        info!(
            patient_id = %launch_context.patient_id,
            encounter_id = ?launch_context.encounter_id,
            principal = %launch_context.principal_id,
            "smart launch initiated"
        );

        let principal = self
            .iam_client
            .resolve_principal(&launch_context.principal_id)
            .await?;
        if !principal.has_permission("fhir::patient::read")? {
            return Err(anyhow!("principal lacks fhir::patient::read"));
        }

        let patient: Patient = match self
            .fhir_client
            .read::<Patient>(&self.tenant_id, &launch_context.patient_id)
            .await
        {
            Ok(p) => p,
            Err(e) if e.is_consent_denied() => {
                warn!(patient_id = %launch_context.patient_id, "consent denied");
                self.emit_consent_denied_audit(&launch_context).await?;
                return Err(anyhow!("consent-cascade denied"));
            }
            Err(e) => return Err(e.into()),
        };

        let encounters: Vec<Encounter> = self
            .fhir_client
            .search::<Encounter>(
                &self.tenant_id,
                SearchParams::new()
                    .with_param("patient", &launch_context.patient_id)
                    .with_param("date", &format!("ge{}", (Utc::now() - Duration::days(365)).to_rfc3339()))
                    .with_param("_sort", "-date")
                    .with_param("_count", "50"),
            )
            .await?;

        let allergies_bundle: Bundle = self
            .fhir_client
            .search_raw_bundle(
                &self.tenant_id,
                "AllergyIntolerance",
                SearchParams::new().with_param("patient", &launch_context.patient_id),
            )
            .await?;

        let active_meds_bundle: Bundle = self
            .fhir_client
            .search_raw_bundle(
                &self.tenant_id,
                "MedicationStatement",
                SearchParams::new()
                    .with_param("subject", &launch_context.patient_id)
                    .with_param("status", "active"),
            )
            .await?;

        info!(
            patient_id = %launch_context.patient_id,
            encounters = encounters.len(),
            allergies = allergies_bundle.entries.len(),
            active_meds = active_meds_bundle.entries.len(),
            "patient summary assembled"
        );

        Ok(PatientSummary {
            patient,
            recent_encounters: encounters,
            allergies: allergies_bundle.entries,
            active_medications: active_meds_bundle.entries,
        })
    }

    async fn emit_consent_denied_audit(
        &self,
        launch_context: &SmartLaunchContext,
    ) -> anyhow::Result<()> {
        let audit = serde_json::json!({
            "resourceType": "AuditEvent",
            "type": {"system": "http://terminology.hl7.org/CodeSystem/audit-event-type", "code": "rest"},
            "subtype": [{"system": "http://hl7.org/fhir/restful-interaction", "code": "read"}],
            "action": "R",
            "recorded": Utc::now().to_rfc3339(),
            "outcome": "8",
            "outcomeDesc": "consent-cascade denied",
            "agent": [{
                "who": {"identifier": {"value": launch_context.principal_id.clone()}},
                "requestor": true,
            }],
            "source": {
                "observer": {"display": "clinical-app-server"},
                "type": [{"code": "4"}],
            },
            "entity": [{
                "what": {"reference": format!("Patient/{}", launch_context.patient_id)},
            }],
        });
        self.fhir_client
            .create_raw(&self.tenant_id, "AuditEvent", &audit)
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
struct PatientSummary {
    patient: Patient,
    recent_encounters: Vec<Encounter>,
    allergies: Vec<fhir_model_r5::BundleEntry>,
    active_medications: Vec<fhir_model_r5::BundleEntry>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = ObservabilityGuard::init("clinical-app-server")?;

    let fhir_client = FhirClient::from_env()?;
    let iam_client = IamClient::from_env()?;
    let principal: Principal = iam_client.whoami().await?;

    let server = ClinicalAppServer {
        fhir_client,
        iam_client,
        tenant_id: principal.tenant_id.clone(),
    };

    let launch = SmartLaunchContext {
        patient_id: std::env::var("LAUNCH_PATIENT_ID")?,
        encounter_id: std::env::var("LAUNCH_ENCOUNTER_ID").ok(),
        principal_id: principal.principal_id.clone(),
        scopes: vec![
            "patient/Patient.read".into(),
            "patient/Encounter.read".into(),
            "patient/AllergyIntolerance.read".into(),
            "patient/MedicationStatement.read".into(),
        ],
    };

    match server.launch_for_patient(launch).await {
        Ok(summary) => {
            info!(
                patient = %summary.patient.id.unwrap_or_default(),
                "summary loaded ok"
            );
        }
        Err(e) => {
            error!(error = %e, "summary load failed");
            return Err(e);
        }
    }

    Ok(())
}
```

## Required Cedar permits

```cedar
permit (
    principal == User::"clinical-app-server@tenant-acme",
    action in [
        Action::"fhir::patient::read",
        Action::"fhir::encounter::read",
        Action::"fhir::allergyintolerance::read",
        Action::"fhir::medicationstatement::read",
        Action::"fhir::auditevent::create"
    ],
    resource in Tenant::"tenant_acme"
)
when {
    context.smart_launch_context.patient_id == resource.patient_id
};
```

## Compliance evidence emitted

Every FHIR read emits an IHE ATNA AuditEvent automatically (by the substrate, before the data returns to your code):

```json
{
    "resourceType": "AuditEvent",
    "type": {"code": "rest"},
    "subtype": [{"code": "read"}],
    "action": "R",
    "recorded": "2026-05-20T14:35:22Z",
    "outcome": "0",
    "agent": [{"who": {"identifier": {"value": "user:clinician@your-tenant.com"}}}],
    "source": {"observer": {"display": "fhir-api-pod-7"}},
    "entity": [{"what": {"reference": "Patient/Patient_001H..."}}]
}
```

This audit-trail is persisted in the FHIR AuditEvent resource AND cross-emitted to `audit-chain` for cryptographic anchoring. HIPAA §164.312(b) audit-controls compliance is automatic.

## Consent-cascade behaviour

If a patient has filed a Consent resource denying access for the requesting principal's role/affiliation, the substrate returns HTTP 403 + a `consent-cascade-denied` outcome code BEFORE the resource is read. The code above catches this via `e.is_consent_denied()` and emits a denial audit. The patient's consent is respected by construction.

## Smart-on-FHIR launch flow

```
1. EHR's Smart launcher posts to YOUR app's launch URL with `iss` + `launch` parameters.
2. Your app constructs the OAuth2 authorization URL pointing at the oyatie FHIR auth endpoint.
3. User authenticates + authorises the requested scopes.
4. oyatie returns an authorization code; your app exchanges it for an access token + launch context.
5. The launch context includes `patient_id` (the patient the EHR is currently displaying).
6. Your app calls oyatie FHIR with the access token; the substrate scopes every read to that patient.
```

## Run + verify

```sh
OYA_TENANT_ID=tenant_acme \
OYA_FHIR_API=https://fhir.dev.<tenant>.oyatie.io \
OYA_IAM_API=https://iam-api.dev.<tenant>.oyatie.io \
LAUNCH_PATIENT_ID=Patient_001H... \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release
```

Verify in the substrate audit log:
```sh
oya audit-chain query --tenant tenant_acme \
    --event-class "fhir::patient::read,fhir::encounter::read" \
    --since "1 hour ago"
```

## Notes

- The Smart-on-FHIR scopes follow the HL7 Smart App Launch IG v2.2.0: `patient/Resource.read` for patient-scoped read; `user/Resource.read` for user-scoped (clinician sees all their patients); `system/Resource.read` for backend services.
- Consent-cascade evaluation is automatic; you cannot disable it.
- Audit events are emitted by the FHIR server itself, even if your app code crashes between request and response. This is by design — audit must precede any data return.
- For high-throughput integration apps (10+ qps sustained), use the bulk-FHIR endpoint (`/Patient/$export`) which streams resources via NDJSON without per-request audit events (audit is at the bulk-export start + end + per-entity-class).
