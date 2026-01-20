{{- define "plateforme.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "plateforme.fullname" -}}
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

{{- define "plateforme.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "plateforme.labels" -}}
helm.sh/chart: {{ include "plateforme.chart" . }}
{{ include "plateforme.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "plateforme.selectorLabels" -}}
app.kubernetes.io/name: {{ include "plateforme.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "plateforme.componentLabels" -}}
{{ include "plateforme.labels" . }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{- define "plateforme.componentSelectorLabels" -}}
{{ include "plateforme.selectorLabels" . }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{- define "plateforme.imageRegistry" -}}
{{- .Values.global.imageRegistry | default "" }}
{{- end }}

{{- define "plateforme.imageTag" -}}
{{- .Values.global.imageTag | default .Chart.AppVersion }}
{{- end }}

{{- define "plateforme.imagePullPolicy" -}}
{{- .Values.global.imagePullPolicy | default "IfNotPresent" }}
{{- end }}

{{- define "plateforme.image" -}}
{{- $registry := include "plateforme.imageRegistry" .context }}
{{- $component := .component }}
{{- $tag := include "plateforme.imageTag" .context }}
{{- printf "%s/%s:%s" $registry $component $tag }}
{{- end }}

{{- define "plateforme.imageWithOverrides" -}}
{{- $registry := include "plateforme.imageRegistry" .context }}
{{- $component := .component }}
{{- $tag := include "plateforme.imageTag" .context }}
{{- $imageConfig := .imageConfig }}
{{- if $imageConfig.repository }}
{{- printf "%s:%s" $imageConfig.repository ($imageConfig.tag | default $tag) }}
{{- else }}
{{- printf "%s/%s:%s" $registry $component ($imageConfig.tag | default $tag) }}
{{- end }}
{{- end }}

{{- define "plateforme.postgresUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- $host := printf "%s-postgres" $fullname }}
{{- $port := "5432" }}
{{- $db := .Values.postgres.auth.database }}
{{- $user := .Values.postgres.auth.username }}
{{- $pass := .Values.secrets.postgresPassword -}} 
{{- printf "postgresql://%s:%s@%s:%s/%s" $user $pass $host $port $db }}
{{- end }}

{{- define "plateforme.spicedbUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- $host := printf "%s-spicedb" $fullname }}
{{- $port := "50051" }}
{{- printf "http://%s:%s" $host $port }}
{{- end }}

{{- define "plateforme.spicedbGrpcEndpoint" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- printf "%s-spicedb:50051" $fullname }}
{{- end }}

{{- define "plateforme.spicedbDatabaseUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- if .Values.spicedb.useSharedDatabase }}
  {{- $host := printf "%s-postgres" $fullname }}
  {{- $port := "5432" }}
  {{- $db := "spicedb" }}
  {{- $user := "spicedb" }}
  {{- printf "postgresql://%s:$(SPICEDB_DB_PASSWORD)@%s:%s/%s?sslmode=disable" $user $host $port $db }}
{{- else }}
  {{- $host := printf "%s-spicedb-db" $fullname }}
  {{- $port := "5432" }}
  {{- $db := .Values.spicedbDb.auth.database }}
  {{- $user := .Values.spicedbDb.auth.username }}
  {{- printf "postgresql://%s:$(SPICEDB_DB_PASSWORD)@%s:%s/%s?sslmode=disable" $user $host $port $db }}
{{- end }}
{{- end }}

{{- define "plateforme.keycloakUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- printf "http://%s-keycloak:8080" $fullname }}
{{- end }}

{{- define "plateforme.keycloakOidcUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- printf "http://%s-keycloak:8080/realms/francenuage/.well-known/openid-configuration" $fullname }}
{{- end }}

{{- define "plateforme.keycloakDatabaseUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- $host := printf "%s-keycloak-db" $fullname }}
{{- $port := "5432" }}
{{- $db := .Values.keycloakDb.auth.database }}
{{- $user := .Values.keycloakDb.auth.username }}
{{- printf "jdbc:postgresql://%s:%s/%s" $host $port $db }}
{{- end }}

{{- define "plateforme.consoleUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- if .Values.console.enabled }}
{{- printf "http://%s-console" $fullname }}
{{- else }}
{{- .Values.controlplane.config.consoleUrl }}
{{- end }}
{{- end }}

{{- define "plateforme.controlplaneUrl" -}}
{{- $fullname := include "plateforme.fullname" . }}
{{- printf "http://%s-controlplane" $fullname }}
{{- end }}

{{- define "plateforme.secretName" -}}
{{- printf "%s-secrets" (include "plateforme.fullname" .) }}
{{- end }}

{{- define "plateforme.waitForPostgres" -}}
- name: wait-for-postgres
  image: busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-postgres 5432; do echo waiting for postgres; sleep 2; done']
{{- end }}

{{- define "plateforme.waitForSpicedb" -}}
- name: wait-for-spicedb
  image: busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-spicedb 50051; do echo waiting for spicedb; sleep 2; done']
{{- end }}

{{- define "plateforme.waitForKeycloak" -}}
- name: wait-for-keycloak
  image: busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-keycloak 8080; do echo waiting for keycloak; sleep 2; done']
{{- end }}

{{- define "plateforme.waitForControlplane" -}}
- name: wait-for-controlplane
  image: busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-controlplane 80; do echo waiting for controlplane; sleep 2; done']
{{- end }}

{{- define "plateforme.waitForConsole" -}}
- name: wait-for-console
  image: busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-console 80; do echo waiting for console; sleep 2; done']
{{- end }}

{{- define "plateforme.runAtlasMigrations" -}}
- name: run-atlas-migrations
  image: "{{ .Values.migrations.atlas.image.repository }}:{{ .Values.migrations.atlas.image.tag }}"
  args:
    - schema
    - apply
    - --url
    - $(DATABASE_URL)?sslmode=disable
    - --to
    - file:///migrations
    - --dev-url
    - $(DATABASE_URL)?sslmode=disable
    - --auto-approve
  env:
    - name: POSTGRES_PASSWORD
      valueFrom:
        secretKeyRef:
          name: {{ include "plateforme.secretName" . }}
          key: postgres-password
    - name: DATABASE_URL
      valueFrom:
        secretKeyRef:
          name: {{ include "plateforme.secretName" . }}
          key: database-url
  {{- if .Values.migrations.atlas.enabled }}
  volumeMounts:
    - name: migrations
      mountPath: /migrations
  {{- end }}
{{- end }}

{{- define "plateforme.waitForMigrations" -}}
- name: wait-for-migrations
  image: postgres:16-alpine
  env:
    - name: PGPASSWORD
      valueFrom:
        secretKeyRef:
          name: {{ include "plateforme.secretName" . }}
          key: postgres-password
  command:
    - sh
    - -c
    - |
      until psql -h {{ include "plateforme.fullname" . }}-postgres -U {{ .Values.postgres.auth.username }} -d {{ .Values.postgres.auth.database }} -c "SELECT 1 FROM organizations LIMIT 1" > /dev/null 2>&1; do
        echo "Waiting for migrations to complete..."
        sleep 5
      done
      echo "Migrations completed!"
{{- end }}

{{- define "plateforme.waitForSpicedbSchema" -}}
- name: wait-for-spicedb-schema
  image: alpine:3.19
  env:
    - name: SPICEDB_GRPC_PRESHARED_KEY
      valueFrom:
        secretKeyRef:
          name: {{ include "plateforme.secretName" . }}
          key: spicedb-preshared-key
  command:
    - sh
    - -c
    - |
      apk add --no-cache curl
      until curl -s http://{{ include "plateforme.fullname" . }}-spicedb:8443/v1/schema/read \
        -H "Authorization: Bearer $SPICEDB_GRPC_PRESHARED_KEY" \
        -H "Content-Type: application/json" \
        -d '{}' 2>/dev/null | grep -q "organization"; do
        echo "Waiting for SpiceDB schema to be loaded..."
        sleep 5
      done
      echo "SpiceDB schema loaded!"
{{- end }}

{{- define "plateforme.runSpicedbMigrations" -}}
- name: run-spicedb-migrations
  image: "{{ .Values.migrations.spicedb.image.repository }}:{{ .Values.migrations.spicedb.image.tag }}"
  command:
    - zed
  args:
    - schema
    - write
    - /schema/schema.zed
    - --endpoint
    - {{ include "plateforme.fullname" . }}-spicedb:50051
    - --insecure
  env:
    - name: ZED_TOKEN
      valueFrom:
        secretKeyRef:
          name: {{ include "plateforme.secretName" . }}
          key: spicedb-preshared-key
  volumeMounts:
    - name: spicedb-schema
      mountPath: /schema
{{- end }}
