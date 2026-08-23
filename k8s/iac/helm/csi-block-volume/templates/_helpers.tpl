{{- define "oyaCsi.name" -}}
{{- .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "oyaCsi.fullname" -}}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "oyaCsi.labels" -}}
app.kubernetes.io/name: {{ include "oyaCsi.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/part-of: cloud-k8s
app.kubernetes.io/managed-by: {{ .Release.Service }}
oyatie/microservice: cloud-k8s
oyatie.io/csi-driver: {{ .Values.driver.name | quote }}
{{- end -}}
