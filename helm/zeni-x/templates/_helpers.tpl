{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "zeni-x.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "zeni-x.fullname" -}}
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
{{- define "zeni-x.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "zeni-x.labels" -}}
helm.sh/chart: {{ include "zeni-x.chart" . }}
{{ include "zeni-x.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "zeni-x.selectorLabels" -}}
app.kubernetes.io/name: {{ include "zeni-x.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Ingress Class based on cluster type
*/}}
{{- define "zeni-x.ingress.className" -}}
{{- if .Values.ingress.className }}
{{- .Values.ingress.className | trim -}}
{{- else if eq .Values.global.clusterType "dev" -}}
traefik
{{- else if eq .Values.global.clusterType "prod" -}}
alb
{{- else -}}
nginx
{{- end -}}
{{- end -}}

{{/*
ServiceAccount name
*/}}
{{- define "zeni-x.serviceAccountName" -}}
{{- if .Values.rbac.serviceAccount.create }}
{{- default (include "zeni-x.fullname" .) .Values.rbac.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.rbac.serviceAccount.name }}
{{- end }}
{{- end }}
