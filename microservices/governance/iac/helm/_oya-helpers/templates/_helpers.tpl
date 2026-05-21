{{/*
  Canonical Helm helper library (SWEEP-I Slice 2 per ADR-0064).
  Every µservice consumes these via `{{ include "oya.<helper>" $ }}`.
  All helpers receive the chart-root context `$` so that `.Values`, `.Chart`,
  `.Release` are available.
*/}}

{{/* oya.labels — canonical label set: app + part-of + microservice + bc + tier + plane */}}
{{- define "oya.labels" -}}
app.kubernetes.io/name: {{ .Chart.Name }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: {{ .Values.partOf | default (printf "oya-%s" .Values.microservice) }}
oyatie/microservice: {{ .Values.microservice | default .Chart.Name }}
{{- if .Values.boundedContext }}
oyatie/bounded_context: {{ .Values.boundedContext }}
{{- end }}
{{- if .Values.tier }}
oyatie/tier: {{ .Values.tier }}
{{- end }}
{{- if .Values.plane }}
oyatie/plane: {{ .Values.plane }}
{{- end }}
{{- end }}

{{/* oya.selectorLabels — narrow subset for pod selectors */}}
{{- define "oya.selectorLabels" -}}
app.kubernetes.io/name: {{ .Chart.Name }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/* oya.serviceAccountName — convention <microservice>-<chart> */}}
{{- define "oya.serviceAccountName" -}}
{{- if .Values.serviceAccount.name -}}
{{- .Values.serviceAccount.name -}}
{{- else -}}
{{- printf "%s-%s" (.Values.microservice | default "oya") .Chart.Name -}}
{{- end -}}
{{- end }}

{{/* oya.securityContext.restricted — combined pod+container pod-security-standards "restricted" profile (legacy alias; new uses split below) */}}
{{- define "oya.securityContext.restricted" -}}
allowPrivilegeEscalation: false
readOnlyRootFilesystem: true
runAsNonRoot: true
runAsUser: 65534
runAsGroup: 65534
capabilities:
  drop: [ALL]
seccompProfile:
  type: RuntimeDefault
{{- end }}

{{/* oya.securityContext.restrictedContainer — container-level restricted profile.
     Drop-in for per-µservice container securityContext blocks that carry:
       allowPrivilegeEscalation: false
       readOnlyRootFilesystem: true
       runAsNonRoot: true
       runAsUser: 65534
       capabilities.drop: [ALL]
     Used by every "nobody UID" µservice (workflow-studio, sheets, etc.). */}}
{{- define "oya.securityContext.restrictedContainer" -}}
allowPrivilegeEscalation: false
readOnlyRootFilesystem: true
runAsNonRoot: true
runAsUser: 65534
capabilities:
  drop: [ALL]
{{- end }}

{{/* oya.securityContext.restrictedContainerInline — terse container-level restricted profile (matches inline form: `capabilities: {drop: ["ALL"]}`). */}}
{{- define "oya.securityContext.restrictedContainerInline" -}}
readOnlyRootFilesystem: true
allowPrivilegeEscalation: false
capabilities: {drop: ["ALL"]}
{{- end }}

{{/* oya.securityContext.podStandard65534 — pod-level securityContext for substrate µservices that use the "nobody" UID. */}}
{{- define "oya.securityContext.podStandard65534" -}}
runAsNonRoot: true
runAsUser: 65534
seccompProfile: {type: RuntimeDefault}
{{- end }}

{{/* oya.securityContext.podStandard65532 — pod-level securityContext for cell/foundry substrate µservices. */}}
{{- define "oya.securityContext.podStandard65532" -}}
runAsNonRoot: true
runAsUser: 65532
runAsGroup: 65532
fsGroup: 65532
seccompProfile:
  type: RuntimeDefault
{{- end }}

{{/* oya.networkPolicy.defaultDeny — baseline deny posture */}}
{{- define "oya.networkPolicy.defaultDeny" -}}
podSelector: {}
policyTypes: [Ingress, Egress]
{{- end }}

{{/* oya.networkPolicy.allowEgressToSubstrate — DNS + audit-chain + observability */}}
{{- define "oya.networkPolicy.allowEgressToSubstrate" -}}
# DNS
- to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: kube-system
  ports:
    - port: 53
      protocol: UDP
# audit-chain (every µservice signs evidence)
- to:
    - namespaceSelector:
        matchLabels:
          oyatie/microservice: audit-chain
  ports:
    - port: 8080
      protocol: TCP
# observability (every µservice emits metrics/traces)
- to:
    - namespaceSelector:
        matchLabels:
          oyatie/microservice: observability
  ports:
    - port: 4317
      protocol: TCP   # OTLP gRPC
    - port: 9090
      protocol: TCP   # Prometheus
{{- end }}

{{/* oya.networkPolicy.allowEgressToCarriers — cross-cutting-carriers exemption per ADR-0140 (retired per ADR-0145).

     Emits a NetworkPolicy egress block permitting direct gRPC (port 50051)
     to the five charter cross-cutting carrier namespaces:

       drive       — file-attachment carrier
       mail        — share-by-email carrier
       messenger   — channel-mention / DM-notify carrier
       calendar    — time-slot / due-date / RSVP bind carrier
       recordings  — long-running media / audio persistence carrier

     Per ADR-0140 this is a DEFINED EXEMPTION to the Workflow+Ontology
     adapter rule (feedback_workflow_objectgraph_adapter_layer). Direct
     egress to any other µservice namespace from an app-tier µservice
     remains forbidden — those flows MUST traverse workflow-engine
     (orchestration) or Ontology (entity reads/writes).

     Usage in a µservice networkpolicy.yaml:

       egress:
         {{- include "oya.networkPolicy.allowEgressToCarriers" $ | nindent 4 }}
         # ... µservice-specific egress (postgres, valkey, etc.) ...

     Per-µservice carrier subset: a µservice MAY include only a subset of
     the five carriers if its PRD declares carry concerns for only those
     carriers. To do so, include this helper's source manually rather than
     via this all-five-carriers default.
*/}}
{{- define "oya.networkPolicy.allowEgressToCarriers" -}}
# Cross-cutting carriers (ADR-0140 exemption). Direct gRPC permitted.
- to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: drive
  ports: [{protocol: TCP, port: 50051}]
- to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: mail
  ports: [{protocol: TCP, port: 50051}]
- to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: messenger
  ports: [{protocol: TCP, port: 50051}]
- to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: calendar
  ports: [{protocol: TCP, port: 50051}]
- to:
    - namespaceSelector:
        matchLabels:
          kubernetes.io/metadata.name: recordings
  ports: [{protocol: TCP, port: 50051}]
{{- end }}

{{/* oya.prometheusRule.perSloBurnRate — canonical multi-window burn-rate alerting (Google SRE Workbook) */}}
{{- define "oya.prometheusRule.perSloBurnRate" -}}
{{- $slo := .slo -}}
{{- $sli := .sli -}}
{{- $target := .target -}}
- alert: {{ $slo }}-fast-burn-1h
  expr: |
    (slo_error_budget_burn_rate_1h{slo="{{ $slo }}"} > 14.4)
    and
    (slo_error_budget_burn_rate_5m{slo="{{ $slo }}"} > 14.4)
  for: 2m
  labels:
    severity: page
    slo: {{ $slo }}
    burn_window: 1h
  annotations:
    summary: "SLO {{ $slo }} burning at 14.4x — 2% of monthly budget consumed in 1h"
    runbook_url: "microservices/{{ .Values.microservice }}/runbooks/slo-burn-{{ $slo }}.md"
- alert: {{ $slo }}-medium-burn-6h
  expr: |
    (slo_error_budget_burn_rate_6h{slo="{{ $slo }}"} > 6.0)
    and
    (slo_error_budget_burn_rate_30m{slo="{{ $slo }}"} > 6.0)
  for: 15m
  labels:
    severity: page
    slo: {{ $slo }}
    burn_window: 6h
  annotations:
    summary: "SLO {{ $slo }} burning at 6x — 5% of monthly budget consumed in 6h"
- alert: {{ $slo }}-slow-burn-1d
  expr: |
    (slo_error_budget_burn_rate_1d{slo="{{ $slo }}"} > 3.0)
    and
    (slo_error_budget_burn_rate_2h{slo="{{ $slo }}"} > 3.0)
  for: 1h
  labels:
    severity: ticket
    slo: {{ $slo }}
    burn_window: 1d
- alert: {{ $slo }}-trickle-burn-3d
  expr: |
    (slo_error_budget_burn_rate_3d{slo="{{ $slo }}"} > 1.0)
    and
    (slo_error_budget_burn_rate_6h{slo="{{ $slo }}"} > 1.0)
  for: 3h
  labels:
    severity: ticket
    slo: {{ $slo }}
    burn_window: 3d
{{- end }}

{{/* oya.probes.standardLiveness — /health probe at named http port (legacy) */}}
{{- define "oya.probes.standardLiveness" -}}
httpGet:
  path: /health
  port: http
periodSeconds: 10
timeoutSeconds: 3
failureThreshold: 3
{{- end }}

{{/* oya.probes.standardReadiness — /ready probe at named http port (legacy) */}}
{{- define "oya.probes.standardReadiness" -}}
httpGet:
  path: /ready
  port: http
periodSeconds: 5
timeoutSeconds: 2
failureThreshold: 2
{{- end }}

{{/* oya.probes.httpHealthLiveness8080 — drop-in: httpGet /health port 8080 + periodSeconds 10 (used by meet, tasks, notes, translate, social, shorts, sheets, forms, messenger, network, community, recordings) */}}
{{- define "oya.probes.httpHealthLiveness8080" -}}
httpGet: {path: /health, port: 8080}
initialDelaySeconds: 10
periodSeconds: 10
{{- end }}

{{/* oya.probes.httpHealthReadiness8080 — drop-in: httpGet /ready port 8080 + periodSeconds 5 */}}
{{- define "oya.probes.httpHealthReadiness8080" -}}
httpGet: {path: /ready, port: 8080}
initialDelaySeconds: 5
periodSeconds: 5
{{- end }}

{{/* oya.probes.healthzLiveness — substrate µservices (cell/foundry/etc.): /livez on metrics port */}}
{{- define "oya.probes.healthzLiveness" -}}
httpGet: {path: /livez, port: metrics}
initialDelaySeconds: 15
periodSeconds: 30
{{- end }}

{{/* oya.probes.healthzReadiness — substrate µservices (cell/foundry/etc.): /healthz on metrics port */}}
{{- define "oya.probes.healthzReadiness" -}}
httpGet: {path: /healthz, port: metrics}
initialDelaySeconds: 5
periodSeconds: 10
{{- end }}

{{/* ----------------------------------------------------------------------
     Tier-A hyperscaler pattern helpers (per ADR-0149..ADR-0156, 2026-05-18).
   ---------------------------------------------------------------------- */}}

{{/* oya.gracefulShutdown — canonical terminationGracePeriodSeconds value
     per docs/standards/graceful-shutdown-canonical.md. Pod spec must
     consume this at the SAME indentation as the canonical pod-spec
     block (sibling to `containers:`).

     Usage in a Deployment template:
       spec:
         template:
           spec:
             {{- include "oya.gracefulShutdown" $ | nindent 6 }}

     Per-workload override via `.Values.terminationGracePeriodSeconds`:
     30 for app-tier; 60 for workers with long-running batches; 120
     for stateful cells (Postgres, Valkey).
   */}}
{{- define "oya.gracefulShutdown" -}}
terminationGracePeriodSeconds: {{ .Values.terminationGracePeriodSeconds | default 30 }}
{{- end }}

{{/* oya.preStopHook.gracefulDelay — canonical preStop hook so the
     load-balancer sees readiness failure before SIGTERM lands.
     Usage in a container block:
       lifecycle:
         {{- include "oya.preStopHook.gracefulDelay" $ | nindent 10 }}
   */}}
{{- define "oya.preStopHook.gracefulDelay" -}}
preStop:
  exec:
    command: ["/bin/sh", "-c", "sleep 5"]
{{- end }}

{{/* oya.probes.startupProbe — canonical Kubernetes startup probe per
     ADR-0145 inv 4 + container ladder. Replaces failed-startup-as-
     unhealthy with explicit slow-start awareness. The startupProbe
     gives the pod up to ~5 minutes (30 × 10s) to become live.

     Usage:
       startupProbe:
         {{- include "oya.probes.startupProbe" $ | nindent 10 }}
   */}}
{{- define "oya.probes.startupProbe" -}}
httpGet: {path: /health, port: 8080}
failureThreshold: 30
periodSeconds: 10
{{- end }}

{{/* oya.probes.startupProbeOnMetrics — substrate µservices that expose
     /livez on the metrics port (cell, foundry, etc.). */}}
{{- define "oya.probes.startupProbeOnMetrics" -}}
httpGet: {path: /livez, port: metrics}
failureThreshold: 30
periodSeconds: 10
{{- end }}

{{/* oya.priorityClass.critical — emits priorityClassName for
     critical-path workloads (data-plane, auth, tenant-isolation).
     Canonical PriorityClass manifests live at
     microservices/governance/iac/kustomize/components/priority-classes/.
     Usage in pod spec:
       {{- include "oya.priorityClass.critical" $ | nindent 6 }}
   */}}
{{- define "oya.priorityClass.critical" -}}
priorityClassName: oya-critical
{{- end }}

{{/* oya.priorityClass.important — non-critical app-tier µservices that
     should preempt low-priority workloads (notifications, search). */}}
{{- define "oya.priorityClass.important" -}}
priorityClassName: oya-important
{{- end }}

{{/* oya.priorityClass.standard — default app-tier workloads. */}}
{{- define "oya.priorityClass.standard" -}}
priorityClassName: oya-standard
{{- end }}

{{/* oya.priorityClass.low — best-effort workloads (batch, backfill,
     report generation). Preempted by everything above. */}}
{{- define "oya.priorityClass.low" -}}
priorityClassName: oya-low
{{- end }}

{{/* oya.envFromOpenBao — renders OpenBao secret-reference env vars */}}
{{- define "oya.envFromOpenBao" -}}
{{- range $name, $path := .Values.secrets -}}
- name: {{ $name | snakecase | upper }}_REF
  value: "${openbao:{{ $path }}}"
{{ end }}
{{- end }}

{{/* oya.resourceRequests — tier-letter dispatcher; accepts a dict with `.tier`
     value of one of XS|S|M|L|XL. Renders the matching tierX block. Lets
     per-µservice values.yaml declare `resourceTier: M` rather than inline
     CPU/memory grids; per-component sizing is then a one-line override.
     Usage:
       resources:
         {{- include "oya.resourceRequests" (dict "tier" $c.resourceTier) | nindent 12 }}

     Tier sizes are canonical-base values per ADR-0064; per-µservice opt-in
     (set `resourceTier` per component) lets the canonical base evolve sizes
     centrally without touching N µservice charts. Five µservices currently
     consume this pattern (messenger / mail / notes / tasks / foundry-guardrails);
     remaining µservices retain inline `resources.requests/limits` and may
     migrate opportunistically. */}}
{{- define "oya.resourceRequests" -}}
{{- $tier := .tier | default "M" -}}
{{- if eq $tier "XS" -}}{{- include "oya.resourceRequests.tierXS" . -}}
{{- else if eq $tier "S" -}}{{- include "oya.resourceRequests.tierS" . -}}
{{- else if eq $tier "M" -}}{{- include "oya.resourceRequests.tierM" . -}}
{{- else if eq $tier "L" -}}{{- include "oya.resourceRequests.tierL" . -}}
{{- else if eq $tier "XL" -}}{{- include "oya.resourceRequests.tierXL" . -}}
{{- else -}}{{- fail (printf "oya.resourceRequests: unknown tier %q (expected XS|S|M|L|XL)" $tier) -}}
{{- end -}}
{{- end }}

{{/* oya.resourceRequests.tierXS — 100m / 128Mi */}}
{{- define "oya.resourceRequests.tierXS" -}}
requests:
  cpu: 100m
  memory: 128Mi
limits:
  cpu: 250m
  memory: 256Mi
{{- end }}

{{/* oya.resourceRequests.tierS — 250m / 512Mi */}}
{{- define "oya.resourceRequests.tierS" -}}
requests:
  cpu: 250m
  memory: 512Mi
limits:
  cpu: 1
  memory: 1Gi
{{- end }}

{{/* oya.resourceRequests.tierM — 500m / 1Gi */}}
{{- define "oya.resourceRequests.tierM" -}}
requests:
  cpu: 500m
  memory: 1Gi
limits:
  cpu: 2
  memory: 4Gi
{{- end }}

{{/* oya.resourceRequests.tierL — 1 / 2Gi */}}
{{- define "oya.resourceRequests.tierL" -}}
requests:
  cpu: 1
  memory: 2Gi
limits:
  cpu: 4
  memory: 8Gi
{{- end }}

{{/* oya.resourceRequests.tierXL — 2 / 4Gi */}}
{{- define "oya.resourceRequests.tierXL" -}}
requests:
  cpu: 2
  memory: 4Gi
limits:
  cpu: 8
  memory: 16Gi
{{- end }}

{{/* ----------------------------------------------------------------------
     Container sandboxing runtime LADDER (per ADR-0147, amended 2026-05-18).

     The five helpers below replace the universal `runtimeClassName: gvisor`
     anti-pattern with workload-class-tiered selection matching AWS/Google/
     Microsoft/Cloudflare per-workload practice. Per the 2026-05-18 amendment
     ("switch to cloud hypervisor") Kata Containers + Cloud Hypervisor
     (`kata-clh`) is the primary untrusted-content / AI-inference (CPU) /
     federation-gateway runtime; gVisor is retained only as opt-in for cold-
     start-sensitive workloads.

       oya.runtimeClassName.appTier            — emits NOTHING (bare Linux + CIS restricted)
       oya.runtimeClassName.untrustedContent   — kata-clh default; kata-clh-sev-snp for sovereign tenant tier
       oya.runtimeClassName.crypto             — kata-clh-sev-snp (AMD SEV-SNP, full-VM + memory-encrypted blast radius)
       oya.runtimeClassName.aiInference        — kata-clh (CPU); kata-clh-tdx for GPU-passthrough confidential compute
       oya.runtimeClassName.federationGateway  — kata-clh + restrictive egress NetworkPolicy

     Per-tenant override: each helper reads `.Values.tenantTier` (string)
     and upgrades the runtime per the ladder (sovereign → SEV-SNP;
     confidential-compute → TDX; fips-140-3-level-3 → bare HSM).

     The legacy `oya.runtimeClassName.gvisor` helper is retained for
     transitional compatibility (cold-start-sensitive opt-in only); new
     uses MUST select one of the five workload-class helpers above.
   ---------------------------------------------------------------------- */}}

{{/* oya.runtimeClassName.appTier — App-tier µservices run on bare Linux + CIS
     K8s restricted profile. This helper deliberately emits NOTHING. Calling
     it documents the workload-class choice (vs. omitting accidentally). */}}
{{- define "oya.runtimeClassName.appTier" -}}
{{- /* No runtimeClassName — bare Linux + CIS restricted per ADR-0147 */ -}}
{{- end }}

{{/* oya.runtimeClassName.untrustedContent — Kata + Cloud Hypervisor default
     (kata-clh) for content-transcoder/renderer workloads (Pandoc,
     LibreOffice, Chromium-headless, WeasyPrint, ffmpeg, ImageMagick).
     Sovereign-tier tenants upgrade to Kata + Cloud Hypervisor + AMD SEV-SNP
     (kata-clh-sev-snp) for cryptographic memory isolation. */}}
{{- define "oya.runtimeClassName.untrustedContent" -}}
{{- $tenantTier := .Values.tenantTier | default "default" -}}
{{- if eq $tenantTier "sovereign" -}}
runtimeClassName: kata-clh-sev-snp
{{- else -}}
runtimeClassName: kata-clh
{{- end -}}
{{- end }}

{{/* oya.runtimeClassName.crypto — Cryptographic workers (blind-signature
     ceremony nodes, KMS-bound signers, signing oracles). Kata + Cloud
     Hypervisor + AMD SEV-SNP (kata-clh-sev-snp) for sovereign-tier
     cryptographic memory isolation; default tier still gets the full-VM
     blast radius via kata-clh-sev-snp (crypto always uses the
     confidential-compute variant). FIPS 140-3 Level 3 tenants route to
     bare HSM (RuntimeClass-less). */}}
{{- define "oya.runtimeClassName.crypto" -}}
{{- $tenantTier := .Values.tenantTier | default "default" -}}
{{- if eq $tenantTier "fips-140-3-level-3" -}}
{{- /* Bare HSM — RuntimeClass-less; pod schedules to HSM-attached host pool */ -}}
{{- else -}}
runtimeClassName: kata-clh-sev-snp
{{- end -}}
{{- end }}

{{/* oya.runtimeClassName.aiInference — AI inference workloads (Whisper
     transcription, ML model inference). Kata + Cloud Hypervisor (kata-clh)
     on CPU paths; confidential-compute tier upgrades to Kata + Cloud
     Hypervisor + Intel TDX (kata-clh-tdx) for GPU-passthrough CC. */}}
{{- define "oya.runtimeClassName.aiInference" -}}
{{- $tenantTier := .Values.tenantTier | default "default" -}}
{{- if eq $tenantTier "confidential-compute" -}}
runtimeClassName: kata-clh-tdx
{{- else -}}
runtimeClassName: kata-clh
{{- end -}}
{{- end }}

{{/* oya.runtimeClassName.federationGateway — Federation/internet-egress
     gateway workers. Kata + Cloud Hypervisor (kata-clh) by default paired
     with restrictive egress NetworkPolicy; sovereign tier upgrades to
     kata-clh-sev-snp for cryptographic memory isolation. */}}
{{- define "oya.runtimeClassName.federationGateway" -}}
{{- $tenantTier := .Values.tenantTier | default "default" -}}
{{- if eq $tenantTier "sovereign" -}}
runtimeClassName: kata-clh-sev-snp
{{- else -}}
runtimeClassName: kata-clh
{{- end -}}
{{- end }}

{{/* oya.runtimeClassName.gvisor — DEPRECATED per ADR-0147 (amended 2026-05-18).
     Retained only as opt-in for cold-start-sensitive µservices. New uses
     MUST select one of the five workload-class helpers above; Cloud
     Hypervisor (kata-clh) is the primary untrusted-content runtime. */}}
{{- define "oya.runtimeClassName.gvisor" -}}
runtimeClassName: gvisor
{{- end }}

{{/* ----------------------------------------------------------------------
     Layered architecture helpers (per ADR-0148, ADR-0182, ADR-0183, ADR-0184).

     Each helper emits annotations / labels that mark a workload as belonging
     to exactly one layer of the layered hyperscaler shape. The layered-
     architecture-discipline fitness gate (oya gate validate
     layered-architecture-discipline) cross-checks that no µservice
     simultaneously declares conflicting layer ownerships.

       oya.gateway.northSouthAnnotation — declares this workload owns
         north-south ingress (per ADR-0182). MUST NOT appear on the same
         workload as oya.mesh.eastWestLabels. Used by the api-gateway
         µservice exclusively.

       oya.mesh.eastWestLabels — declares this workload runs on the
         east-west Istio Ambient + Cilium mesh layer (per ADR-0148).
         Emits the canonical pair: istio.io/dataplane-mode=ambient +
         cilium.io/identity-policy=enforced. Default for every µservice
         except api-gateway.

       oya.cache.valkeySidecar — opt-in Valkey-cluster client config
         (env vars + sidecar reference) for µservices using Tier-3 cache
         per ADR-0184. Per ADR-0184 the cache project is Valkey 8.1 (BSD
         3-Clause); Valkey CLI and client-library behavior remains
         protocol-compatible at the RESP layer.

       oya.kyverno.podSecurityPolicy — emits annotation pointing at the
         canonical Kyverno PSS-restricted ClusterPolicy per ADR-0183.
   ---------------------------------------------------------------------- */}}

{{/* oya.gateway.northSouthAnnotation — marks workload as north-south ingress
     per ADR-0182. Declared ONLY on the api-gateway µservice. The layered-
     architecture-discipline gate rejects any other workload carrying this
     annotation. */}}
{{- define "oya.gateway.northSouthAnnotation" -}}
gateway.networking.k8s.io/managed-by: envoy-gateway
oyatie/traffic-direction: north-south
{{- end }}

{{/* oya.mesh.eastWestLabels — canonical east-west mesh layer labels per
     ADR-0148. Emits Istio Ambient dataplane-mode + Cilium identity policy
     enforcement. Default on every µservice EXCEPT api-gateway. */}}
{{- define "oya.mesh.eastWestLabels" -}}
istio.io/dataplane-mode: ambient
cilium.io/identity-policy: enforced
oyatie/traffic-direction: east-west
{{- end }}

{{/* oya.mesh.ambientWaypoint — opt-in waypoint enrollment per ADR-0148.
     Emits the Gateway resource label declaring the namespace is enrolled
     for Tier-3 L7 waypoint enforcement. Only µservices with
     manifest.json `mesh_layering.ambient_waypoint: true` enroll this.
     Reads `.Values.meshLayering.ambientWaypoint` boolean; emits the
     Gateway-API-conformant label set when true. */}}
{{- define "oya.mesh.ambientWaypoint" -}}
{{- if .Values.meshLayering.ambientWaypoint -}}
istio.io/use-waypoint: {{ printf "%s-waypoint" .Values.microservice | quote }}
gateway.networking.k8s.io/gateway-name: {{ printf "%s-waypoint" .Values.microservice | quote }}
{{- end -}}
{{- end }}

{{/* oya.cache.valkeySidecar — Tier-3 Valkey-cluster client config block
     per ADR-0184. Renders env vars + initContainer reference for
     µservices opting into the canonical hot read-through cache. The
     deployed cluster is Valkey 8.1 (BSD 3-Clause) per ADR-0184 and
     ADR-0336.

     Required Values:
       .Values.cache.enabled              (boolean)
       .Values.cache.clusterEndpoint      (string; e.g. valkey-cluster.cache.svc.cluster.local:6379)
       .Values.cache.tlsSecretName        (string; OpenBao-issued cert)
       .Values.cache.defaultTtlSeconds    (integer; default 60 per ADR-0184) */}}
{{- define "oya.cache.valkeySidecar" -}}
{{- if .Values.cache.enabled -}}
- name: OYA_CACHE_BACKEND
  value: "valkey-cluster"
- name: OYA_CACHE_ENDPOINT
  value: {{ .Values.cache.clusterEndpoint | quote }}
- name: OYA_CACHE_TLS_SECRET
  value: {{ .Values.cache.tlsSecretName | default "valkey-client-tls" | quote }}
- name: OYA_CACHE_DEFAULT_TTL_SECONDS
  value: {{ .Values.cache.defaultTtlSeconds | default 60 | quote }}
- name: OYA_CACHE_LICENSE_NOTE
  value: "Valkey 8.1 BSD-3-Clause (Linux Foundation fork; ADR-0336 canonical substrate)"
{{- end -}}
{{- end }}

{{/* oya.kyverno.podSecurityPolicy — references the canonical Kyverno
     PSS-restricted ClusterPolicy per ADR-0183. Emits the annotation
     that the layered-architecture-discipline gate looks for to confirm
     the workload is governed by Kyverno admission control. */}}
{{- define "oya.kyverno.podSecurityPolicy" -}}
oyatie/kyverno-cluster-policy: pod-security-restricted
pod-security.kubernetes.io/enforce: restricted
pod-security.kubernetes.io/audit: restricted
pod-security.kubernetes.io/warn: restricted
{{- end }}
