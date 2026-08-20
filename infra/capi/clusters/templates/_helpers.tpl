{{/* Infra apiVersion: Metal3 is on v1beta1; CAPOCI/CAPA are on v1beta2. */}}
{{- define "spoke.infraApiVersion" -}}
{{- if eq .substrate "metal3" -}}infrastructure.cluster.x-k8s.io/v1beta1{{- else -}}infrastructure.cluster.x-k8s.io/v1beta2{{- end -}}
{{- end -}}

{{- define "spoke.infraClusterKind" -}}
{{- if eq .substrate "oci" -}}OCICluster{{- else if eq .substrate "aws" -}}AWSCluster{{- else if eq .substrate "metal3" -}}Metal3Cluster{{- else -}}{{- fail (printf "cell %s: unknown substrate %q (want oci|aws|metal3)" .name .substrate) -}}{{- end -}}
{{- end -}}

{{- define "spoke.infraMachineTemplateKind" -}}
{{- if eq .substrate "oci" -}}OCIMachineTemplate{{- else if eq .substrate "aws" -}}AWSMachineTemplate{{- else if eq .substrate "metal3" -}}Metal3MachineTemplate{{- end -}}
{{- end -}}
