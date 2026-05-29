# Diagnostics Counterpart Parity Matrix

Scope: lab + pathology only. Imaging vendors and PACS/VNA capabilities are intentionally excluded because `microservices/imaging/` owns the imaging domain.

| Capability | Sunquest / Clinisys LIS | Oracle Health PathNet | Epic Beaker + Beaker AP | Roche Navify | LabCorp / Quest connectivity | Oyatie diagnostics |
| --- | --- | --- | --- | --- | --- | --- |
| Lab order intake | Yes | Yes | Yes | Partial | Yes | Yes |
| Lab result lifecycle | Yes | Yes | Yes | Partial | Yes | Yes |
| Specimen accessioning | Yes | Yes | Yes | Partial | Partial | Yes |
| Reference ranges | Yes | Yes | Yes | Partial | Partial | Yes |
| Reflex testing | Yes | Yes | Yes | Partial | Partial | Yes |
| Critical-result escalation | Yes | Yes | Yes | Partial | Partial | Yes |
| QC / analyzer evidence | Yes | Yes | Yes | Partial | Partial | Yes |
| Pathology case management | Partial | Yes | Yes | Partial | Partial | Yes |
| Pathology sign-out | Partial | Yes | Yes | Partial | No | Yes |
| Addenda / amendments | Yes | Yes | Yes | Partial | Partial | Yes |
| TAT dashboard | Yes | Yes | Yes | Partial | Partial | Yes |
| FHIR Observation projection | Partial | Yes | Yes | Partial | Partial | Yes |
| Lab/pathology DiagnosticReport projection | Partial | Yes | Yes | Partial | Partial | Yes |
| Tenant/cell policy isolation | No | Partial | Partial | No | No | Yes |
| Cedar default-deny policy gates | No | No | No | No | No | Yes |
| Image correlation handoff | Integration only | Integration only | Integration only | Integration only | Integration only | Reference-only handoff to imaging |

Imaging vendors and imaging capabilities are covered by `../imaging/competitor-parity-matrix.md`.
