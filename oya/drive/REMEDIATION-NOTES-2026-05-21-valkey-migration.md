## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/drive/AUDIT-FINDINGS-2026-05-18.json
- microservices/drive/IP-001-iac-bootstrap.md
- microservices/drive/IP-006-upload.md
- microservices/drive/PHASE-01-DRIVE-FOUNDATION.md
- microservices/drive/PRD.md
- microservices/drive/catalog/oya-drive-upload-adapter-valkey.yaml
- microservices/drive/catalog/oya-drive-upload-kernel.yaml
- microservices/drive/coherence-audit-2026-05-20.md
- microservices/drive/iac/helm/Chart.yaml
- microservices/drive/iac/helm/templates/networkpolicy.yaml
- microservices/drive/iac/helm/values.yaml
- microservices/drive/manifest.json
- microservices/drive/runbooks/upload-multipart-stuck.md
- microservices/drive/threat-model.md

Counterpart-fact preservations:
- None; every Redis hit in this bucket described Oyatie-owned substrate vocabulary.

Files renamed (git mv):
- microservices/drive/catalog/oya-drive-upload-adapter-redis.yaml -> microservices/drive/catalog/oya-drive-upload-adapter-valkey.yaml
