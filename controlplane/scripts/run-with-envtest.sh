#!/usr/bin/env bash
# Runs a command against a throwaway Kubernetes API server (kube-apiserver +
# etcd) launched as plain processes. No node, no kubelet, no container, no
# privileged access, so it works on unprivileged CI runners.
#
# The API server is enough for control-plane operations that only touch the
# API (namespaces, secrets, ...); it does NOT schedule pods.
#
# Binaries come from the envtest assets: either $KUBEBUILDER_ASSETS (a directory
# holding etcd/kube-apiserver/kubectl) or, if unset, `setup-envtest use`.
#
# Usage:
#   scripts/run-with-envtest.sh cargo test -p workflow --test kubernetes_operations -- --ignored
#
# The launched command sees $E2E_KUBECONFIG pointing at a ready cluster.
set -euo pipefail

ENVTEST_K8S_VERSION="${ENVTEST_K8S_VERSION:-1.31.0}"

if [ -n "${KUBEBUILDER_ASSETS:-}" ]; then
  BINDIR="$KUBEBUILDER_ASSETS"
else
  BINDIR="$(setup-envtest use "$ENVTEST_K8S_VERSION" -p path)"
fi

WORK="$(mktemp -d)"
cleanup() {
  [ -n "${API_PID:-}" ] && kill "$API_PID" 2>/dev/null || true
  [ -n "${ETCD_PID:-}" ] && kill "$ETCD_PID" 2>/dev/null || true
  rm -rf "$WORK"
}
trap cleanup EXIT

printf 'testtoken,admin,admin-uid,"system:masters"\n' > "$WORK/token.csv"
mkdir -p "$WORK/certs"
openssl genrsa -out "$WORK/certs/sa.key" 2048 2>/dev/null
openssl rsa -in "$WORK/certs/sa.key" -pubout -out "$WORK/certs/sa.pub" 2>/dev/null

"$BINDIR/etcd" \
  --data-dir="$WORK/etcd" \
  --listen-client-urls=http://127.0.0.1:2379 \
  --advertise-client-urls=http://127.0.0.1:2379 \
  --listen-peer-urls=http://127.0.0.1:2380 \
  --log-level=error >/dev/null 2>&1 &
ETCD_PID=$!

"$BINDIR/kube-apiserver" \
  --etcd-servers=http://127.0.0.1:2379 \
  --bind-address=127.0.0.1 \
  --advertise-address=127.0.0.1 \
  --secure-port=6443 \
  --service-cluster-ip-range=10.0.0.0/24 \
  --authorization-mode=AlwaysAllow \
  --token-auth-file="$WORK/token.csv" \
  --cert-dir="$WORK/certs" \
  --disable-admission-plugins=ServiceAccount \
  --service-account-issuer=https://kubernetes.default.svc \
  --service-account-key-file="$WORK/certs/sa.pub" \
  --service-account-signing-key-file="$WORK/certs/sa.key" >/dev/null 2>&1 &
API_PID=$!

export E2E_KUBECONFIG="$WORK/kubeconfig"
cat > "$E2E_KUBECONFIG" <<EOF
apiVersion: v1
kind: Config
clusters:
- name: envtest
  cluster: {server: https://127.0.0.1:6443, insecure-skip-tls-verify: true}
contexts:
- name: envtest
  context: {cluster: envtest, user: admin}
current-context: envtest
users:
- name: admin
  user: {token: testtoken}
EOF

for _ in $(seq 1 60); do
  if "$BINDIR/kubectl" --kubeconfig="$E2E_KUBECONFIG" get --raw=/readyz >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
[ "${ready:-0}" = 1 ] || { echo "kube-apiserver did not become ready" >&2; exit 1; }

exec "$@"
