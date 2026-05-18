{{/*
Local helpers — delegate to shared _oya-helpers where possible.
*/}}

{{- define "oya.fullname" -}}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "oya.serviceAccountName" -}}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Canonical labels.
*/}}
{{- define "oya.labels" -}}
app.kubernetes.io/name: {{ .Chart.Name }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
oya.io/microservice: {{ .Values.microservice }}
oya.io/bounded-context: {{ .Values.boundedContext }}
{{- end -}}

{{- define "oya.selectorLabels" -}}
app.kubernetes.io/name: {{ .Chart.Name }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{/*
Cost-attribution labels — per ADR-0199 D-2.
*/}}
{{- define "oya.tenantCostLabels" -}}
oya.io/cost-center: {{ .Values.costAttribution.costCenter }}
oya.io/workload-class: {{ .Values.costAttribution.workloadClass }}
oya.io/regulatory-pack: {{ .Values.costAttribution.regulatoryPack }}
{{- end -}}
