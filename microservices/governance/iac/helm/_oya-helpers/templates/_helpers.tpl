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

{{/* oya.envFromOpenBao — renders OpenBao secret-reference env vars */}}
{{- define "oya.envFromOpenBao" -}}
{{- range $name, $path := .Values.secrets -}}
- name: {{ $name | snakecase | upper }}_REF
  value: "${openbao:{{ $path }}}"
{{ end }}
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

{{/* oya.runtimeClassName.gvisor — gVisor sandbox runtime */}}
{{- define "oya.runtimeClassName.gvisor" -}}
runtimeClassName: gvisor
{{- end }}
