## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/plugin-app-store/ARCHITECTURE.md
- microservices/plugin-app-store/PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md
- microservices/plugin-app-store/PRD.md
- microservices/plugin-app-store/catalog/oya-plugin-app-store-per-plugin-rate-limit-adapter-valkey.yaml
- microservices/plugin-app-store/compliance.md
- microservices/plugin-app-store/implementation-plans/IP-001-layer-a-postgres-valkey-cedar-cosign-trivy-iac.md
- microservices/plugin-app-store/implementation-plans/IP-010-per-plugin-rate-limit.md
- microservices/plugin-app-store/manifest.json

Counterpart-fact preservations:
- None; every Redis hit in this bucket described Oyatie-owned substrate vocabulary.

Files renamed (git mv):
- microservices/plugin-app-store/catalog/oya-plugin-app-store-per-plugin-rate-limit-adapter-redis.yaml -> microservices/plugin-app-store/catalog/oya-plugin-app-store-per-plugin-rate-limit-adapter-valkey.yaml
- microservices/plugin-app-store/implementation-plans/IP-001-layer-a-postgres-redis-cedar-cosign-trivy-iac.md -> microservices/plugin-app-store/implementation-plans/IP-001-layer-a-postgres-valkey-cedar-cosign-trivy-iac.md
