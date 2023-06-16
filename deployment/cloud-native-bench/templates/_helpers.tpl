{{/*
Expand the name of the chart.
*/}}
{{- define "cloud-native-bench.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "cloud-native-bench.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "cloud-native-bench.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "cloud-native-bench.labels" -}}
helm.sh/chart: {{ include "cloud-native-bench.chart" . }}
{{ include "cloud-native-bench.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "cloud-native-bench.selectorLabels" -}}
app.kubernetes.io/name: "cloud-native-bench"
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
{{- define "cloud-native-bench.operator.selectorLabels" -}}
app.kubernetes.io/name: "cloud-native-bench-operator"
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
{{- define "cloud-native-bench.web.selectorLabels" -}}
app.kubernetes.io/name: "cloud-native-bench-web"
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
{{- define "cloud-native-bench.analysis.selectorLabels" -}}
app.kubernetes.io/name: "cloud-native-bench-analysis"
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "cloud-native-bench.serviceAccountName" -}}
{{- default (include "cloud-native-bench.fullname" .) .Values.serviceAccount.name }}
{{- end }}
