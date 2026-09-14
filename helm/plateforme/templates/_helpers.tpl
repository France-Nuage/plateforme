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

{{/*
Colocalisation zone avec le primary CNPG : préférer (soft, weight 100) la zone
du primary évite un hop FTTH inter-DC par requête SQL. Jamais required (pas de
Pending). Suit automatiquement le primary via le label role=primary. Le cluster
CNPG unique s'appelle <fullname>-db.
*/}}
{{- define "plateforme.dbColocation" -}}
podAffinity:
  preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchLabels:
            cnpg.io/cluster: {{ include "plateforme.fullname" . }}-db
            role: primary
        topologyKey: topology.kubernetes.io/zone
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

{{- define "plateforme.baseDomain" -}}
{{- required "ingress.baseDomain is required" .Values.ingress.baseDomain }}
{{- end }}

{{/*
Builds a public host for the given service component.

Without ingress.envId (prod, local) the host is "<service>.<baseDomain>".
With an envId (ephemeral CI environments) the service and env id are joined into
a single label via ingress.hostSeparator, e.g. "console--<envId>.<baseDomain>".
Keeping the env id inside one label lets a single "*.<baseDomain>" wildcard
certificate cover every environment (wildcards match only one level).

Usage: {{ include "plateforme.hostFor" (dict "svc" "console" "ctx" .) }}
*/}}
{{- define "plateforme.hostFor" -}}
{{- $svc := .svc }}
{{- $ctx := .ctx }}
{{- $base := include "plateforme.baseDomain" $ctx }}
{{- $envId := $ctx.Values.ingress.envId | toString }}
{{- if $envId }}
{{- $sep := $ctx.Values.ingress.hostSeparator | default "--" }}
{{- printf "%s%s%s.%s" $svc $sep $envId $base }}
{{- else }}
{{- printf "%s.%s" $svc $base }}
{{- end }}
{{- end }}

{{- define "plateforme.consoleHost" -}}
{{- include "plateforme.hostFor" (dict "svc" "console" "ctx" .) }}
{{- end }}

{{- define "plateforme.controlplaneHost" -}}
{{- include "plateforme.hostFor" (dict "svc" "controlplane" "ctx" .) }}
{{- end }}

{{- define "plateforme.keycloakHost" -}}
{{- include "plateforme.hostFor" (dict "svc" "auth" "ctx" .) }}
{{- end }}

{{/*
cert-manager annotation for an ingress. Emitted only when no shared TLS secret
is configured; with a pre-provisioned wildcard secret cert-manager must not
issue a per-host certificate.
*/}}
{{- define "plateforme.ingressCertManagerAnnotation" -}}
{{- if not .Values.ingress.tlsSecretName }}
cert-manager.io/cluster-issuer: {{ .Values.ingress.clusterIssuer | quote }}
{{- end }}
{{- end }}

{{/*
TLS secret name for an ingress: the shared wildcard secret when configured,
otherwise the given per-host secret (issued by cert-manager).
Usage: {{ include "plateforme.ingressTlsSecret" (dict "default" "x-console-tls" "ctx" .) }}
*/}}
{{- define "plateforme.ingressTlsSecret" -}}
{{- .ctx.Values.ingress.tlsSecretName | default .default }}
{{- end }}

{{- define "plateforme.keycloakUrl" -}}
{{- printf "https://%s" (include "plateforme.keycloakHost" .) }}
{{- end }}

{{/* The OIDC issuer / realm base URL (authority) advertised to clients. */}}
{{- define "plateforme.keycloakRealmUrl" -}}
{{- printf "%s/realms/francenuage" (include "plateforme.keycloakUrl" .) }}
{{- end }}

{{/* The OIDC discovery document URL, derived from the realm URL. */}}
{{- define "plateforme.keycloakOidcUrl" -}}
{{- printf "%s/.well-known/openid-configuration" (include "plateforme.keycloakRealmUrl" .) }}
{{- end }}

{{/*
hostAliases pinning the public Keycloak host to an in-cluster ingress IP.
Backend services (control plane, synchronizer) reach Keycloak over its public
URL to match the token issuer, which normally requires a hairpin back through
external DNS. On clusters where that hairpin does not resolve (e.g. qualif),
set oidcHairpinIp to the ingress controller ClusterIP so the lookup stays
in-cluster while keeping the public hostname (and issuer). No-op when unset.
*/}}
{{- define "plateforme.oidcHairpinHostAliases" -}}
{{- with .Values.oidcHairpinIp }}
hostAliases:
  - ip: {{ . | quote }}
    hostnames:
      - {{ include "plateforme.keycloakHost" $ | quote }}
{{- end }}
{{- end }}

{{/*
hostAliases pinning every public host (console, control plane, Keycloak) to the
in-cluster ingress IP. Used by the system tests, whose browser and SDK reach the
stack through its public URLs; on clusters without a working hairpin these would
otherwise be unreachable from inside the cluster. No-op when oidcHairpinIp unset.
*/}}
{{- define "plateforme.testsHostAliases" -}}
{{- with .Values.oidcHairpinIp }}
hostAliases:
  - ip: {{ . | quote }}
    hostnames:
      - {{ include "plateforme.consoleHost" $ | quote }}
      - {{ include "plateforme.controlplaneHost" $ | quote }}
      - {{ include "plateforme.keycloakHost" $ | quote }}
{{- end }}
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
{{- printf "https://%s" (include "plateforme.consoleHost" .) }}
{{- end }}

{{- define "plateforme.controlplaneUrl" -}}
{{- printf "https://%s" (include "plateforme.controlplaneHost" .) }}
{{- end }}

{{- define "plateforme.secretName" -}}
{{- if .Values.secrets.existingSecret }}
{{- .Values.secrets.existingSecret }}
{{- else }}
{{- printf "%s-secrets" (include "plateforme.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Whether Stripe billing is enabled. Billing is opt-in and only wires itself up
when BOTH Stripe secrets are provided (an ephemeral environment without the
protected CI variables keeps billing off and the payment E2E is skipped). Every
billing-conditional block (control plane env, webhook relay sidecar, secret
keys) keys off this single predicate so they enable together.
*/}}
{{- define "plateforme.billingEnabled" -}}
{{- if and (not (empty .Values.secrets.stripeSecretKey)) (not (empty .Values.secrets.stripeWebhookSecret)) -}}
true
{{- end -}}
{{- end }}

{{/*
Stripe Checkout success URL: the console URL with the configured success path
appended. Derived from the console host so it follows each ephemeral environment
automatically instead of being hardcoded per environment.
*/}}
{{- define "plateforme.stripeCheckoutSuccessUrl" -}}
{{- printf "%s%s" (include "plateforme.consoleUrl" .) .Values.controlplane.config.billing.checkoutSuccessPath }}
{{- end }}

{{/*
Stripe Checkout cancel URL: the console URL with the configured cancel path
appended. Derived from the console host, like the success URL above.
*/}}
{{- define "plateforme.stripeCheckoutCancelUrl" -}}
{{- printf "%s%s" (include "plateforme.consoleUrl" .) .Values.controlplane.config.billing.checkoutCancelPath }}
{{- end }}

{{- define "plateforme.waitForPostgres" -}}
- name: wait-for-postgres
  image: registry.france-nuage.fr/library/busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-postgres 5432; do echo waiting for postgres; sleep 2; done']
{{- end }}

{{- define "plateforme.waitForSpicedb" -}}
- name: wait-for-spicedb
  image: registry.france-nuage.fr/library/busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-spicedb 50051; do echo waiting for spicedb; sleep 2; done']
{{- end }}

{{- define "plateforme.waitForKeycloak" -}}
- name: wait-for-keycloak
  image: registry.france-nuage.fr/library/busybox:1.36
  command:
    - sh
    - -c
    - |
      KEYCLOAK_URL="http://{{ include "plateforme.fullname" . }}-keycloak:8080/realms/francenuage/.well-known/openid-configuration"
      until wget -q --spider --timeout=5 $KEYCLOAK_URL; do
        echo "Waiting for Keycloak"
        sleep 2
      done
      echo "Keycloak is ready"
{{- end }}

{{- define "plateforme.waitForControlplane" -}}
- name: wait-for-controlplane
  image: registry.france-nuage.fr/library/busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-controlplane 80; do echo waiting for controlplane; sleep 2; done']
{{- end }}

{{- define "plateforme.waitForConsole" -}}
- name: wait-for-console
  image: registry.france-nuage.fr/library/busybox:1.36
  command: ['sh', '-c', 'until nc -z {{ include "plateforme.fullname" . }}-console 80; do echo waiting for console; sleep 2; done']
{{- end }}

{{/*
Waits for the public endpoints the system tests exercise (console, control
plane, OIDC) to be served through the ingress with a valid TLS certificate.
Unlike the TCP probes above, this validates the HTTPS chain (curl fails on an
untrusted cert without -k), so the tests only start once cert-manager has issued
the real certificate and every ingress route is live rather than while the
ingress still serves its self-signed default or a 503. curl is required here:
busybox wget does not implement TLS verification. The control plane speaks gRPC,
so we only assert the TLS handshake succeeds (any HTTP status), not a 2xx.
*/}}
{{- define "plateforme.waitForPublicEndpoints" -}}
- name: wait-for-public-endpoints
  image: registry.france-nuage.fr/curlimages/curl:8.11.1
  command:
    - sh
    - -c
    - |
      until curl -sf -o /dev/null --max-time 5 {{ include "plateforme.consoleUrl" . }}/config.js; do
        echo "waiting for public console"
        sleep 5
      done
      until curl -sf -o /dev/null --max-time 5 {{ include "plateforme.keycloakOidcUrl" . }}; do
        echo "waiting for public OIDC certificate"
        sleep 5
      done
      until curl -s -o /dev/null --max-time 5 {{ include "plateforme.controlplaneUrl" . }}; do
        echo "waiting for public control plane"
        sleep 5
      done
      echo "public endpoints ready"
{{- end }}

{{- define "plateforme.runAtlasMigrations" -}}
- name: run-atlas-migrations
  image: {{ include "plateforme.imageWithOverrides" (dict "component" "controlplane" "imageConfig" .Values.controlplane.image "context" .) }}
  command: ["atlas"]
  args:
    - migrate
    - apply
    - --url
    - $(DATABASE_URL)?sslmode=disable
    - --dir
    - file:///app/migrations
  env:
    - name: DATABASE_URL
      valueFrom:
        secretKeyRef:
          name: {{ include "plateforme.secretName" . }}
          key: database-url
{{- end }}

{{- define "plateforme.waitForMigrations" -}}
- name: wait-for-migrations
  image: registry.france-nuage.fr/library/postgres:16-alpine
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
  image: registry.france-nuage.fr/library/alpine:3.19
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
