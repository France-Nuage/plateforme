#!/bin/bash
set -euo pipefail

# Certificate generation script for Docker Compose development
# Usage: ./scripts/generate-certs.sh [--force]

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR" # Script is at project root
CERTS_DIR="$PROJECT_DIR/certs"
TRAEFIK_DIR="$PROJECT_DIR/traefik"
FORCE_REGENERATE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
  --force | -f)
    FORCE_REGENERATE=true
    shift
    ;;
  --help | -h)
    echo "Usage: $0 [--force]"
    echo "Generate SSL certificates for Docker Compose development"
    echo ""
    echo "Options:"
    echo "  --force, -f    Force regeneration even if certificates exist"
    echo "  --help, -h     Show this help message"
    exit 0
    ;;
  *)
    echo -e "${RED}Error: Unknown option $1${NC}"
    echo "Use --help for usage information"
    exit 1
    ;;
  esac
done

# Logging functions
log_info() {
  echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
  echo -e "${GREEN}✅${NC} $1"
}

log_warning() {
  echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
  echo -e "${RED}❌${NC} $1"
}

# Check if a command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
  log_info "Checking dependencies..."

  local missing_deps=()

  if ! command_exists openssl; then
    missing_deps+="openssl"
  fi

  if [ ${#missing_deps[@]} -gt 0 ]; then
    log_error "Missing required dependencies: ${missing_deps[*]}"
    echo ""
    echo "Please install the missing dependencies:"
    echo ""

    for dep in "${missing_deps[@]}"; do
      case "$dep" in
      openssl)
        echo "OpenSSL:"
        echo "  • Ubuntu/Debian: sudo apt-get install openssl"
        echo "  • CentOS/RHEL: sudo yum install openssl"
        echo "  • macOS: brew install openssl"
        echo "  • Windows: Download from https://slproweb.com/products/Win32OpenSSL.html"
        ;;
      esac
      echo ""
    done

    exit 1
  fi

  log_success "All dependencies satisfied"
}

# Check if certificates already exist
check_existing_certs() {
  if [ "$FORCE_REGENERATE" = true ]; then
    log_warning "Force regeneration requested, will overwrite existing certificates"
    return 0
  fi

  if [ -f "$CERTS_DIR/ca.pem" ] && [ -f "$CERTS_DIR/server.pem" ] && [ -f "$CERTS_DIR/server-key.pem" ]; then
    log_warning "Certificates already exist in $CERTS_DIR"
    echo ""
    echo "Existing certificates:"
    echo "  • CA: $CERTS_DIR/ca.pem"
    echo "  • Server cert: $CERTS_DIR/server.pem"
    echo "  • Server key: $CERTS_DIR/server-key.pem"
    echo ""
    echo "Use --force to regenerate or remove the existing certificates manually."
    echo "To check certificate validity:"
    echo "  openssl x509 -in $CERTS_DIR/server.pem -text -noout | grep -A2 'Validity'"
    exit 0
  fi
}

# Create directories
create_directories() {
  log_info "Creating directories..."
  mkdir -p "$CERTS_DIR" "$TRAEFIK_DIR"
  log_success "Directories created"
}

# Generate CA certificate
generate_ca() {
  log_info "Generating Certificate Authority (CA)..."

  # Generate CA private key
  openssl genrsa -out "$CERTS_DIR/ca-key.pem" 4096

  # Generate CA certificate
  openssl req -new -x509 -days 3650 -key "$CERTS_DIR/ca-key.pem" -out "$CERTS_DIR/ca.pem" \
    -subj "/C=US/ST=Development/L=Docker/O=Local Development/OU=Docker Compose/CN=Development CA"

  log_success "CA certificate generated"
}

# Generate server certificate
generate_server_cert() {
  log_info "Generating server certificate..."

  # Generate server private key
  openssl genrsa -out "$CERTS_DIR/server-key.pem" 4096

  # Create extensions file for Subject Alternative Names
  cat >"$CERTS_DIR/server-ext.cnf" <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = Development
L = Docker
O = Local Development
OU = Docker Compose
CN = localhost

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = oidc
DNS.3 = console
DNS.4 = controlplane
DNS.5 = traefik
DNS.6 = *.test
DNS.7 = host.docker.internal
DNS.8 = oidc.test
DNS.9 = console.test
DNS.10 = controlplane.test
DNS.11 = traefik.test
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

  # Generate certificate signing request
  openssl req -new -key "$CERTS_DIR/server-key.pem" -out "$CERTS_DIR/server.csr" \
    -config "$CERTS_DIR/server-ext.cnf"

  # Generate server certificate signed by CA
  openssl x509 -req -days 365 -in "$CERTS_DIR/server.csr" \
    -CA "$CERTS_DIR/ca.pem" -CAkey "$CERTS_DIR/ca-key.pem" \
    -out "$CERTS_DIR/server.pem" \
    -extensions v3_req -extfile "$CERTS_DIR/server-ext.cnf" \
    -CAcreateserial

  # Clean up temporary files
  rm "$CERTS_DIR/server.csr" "$CERTS_DIR/server-ext.cnf"

  log_success "Server certificate generated"
}

# Generate Traefik dynamic configuration
generate_traefik_config() {
  log_info "Generating Traefik dynamic configuration..."

  cat >"$TRAEFIK_DIR/dynamic.yml" <<EOF
tls:
  certificates:
    - certFile: /etc/traefik/certs/server.pem
      keyFile: /etc/traefik/certs/server-key.pem
      stores:
        - default
  stores:
    default:
      defaultCertificate:
        certFile: /etc/traefik/certs/server.pem
        keyFile: /etc/traefik/certs/server-key.pem
EOF

  log_success "Traefik configuration generated"
}

# Set appropriate permissions
set_permissions() {
  log_info "Setting certificate permissions..."

  # Set restrictive permissions on private keys
  chmod 600 "$CERTS_DIR/ca-key.pem" "$CERTS_DIR/server-key.pem"
  chmod 644 "$CERTS_DIR/ca.pem" "$CERTS_DIR/server.pem"

  if [ -f "$CERTS_DIR/ca.srl" ]; then
    chmod 644 "$CERTS_DIR/ca.srl"
  fi

  log_success "Permissions set correctly"
}

# Display certificate information
show_certificate_info() {
  log_info "Certificate information:"
  echo ""
  echo "📁 Certificate files:"
  echo "  • CA certificate: $CERTS_DIR/ca.pem"
  echo "  • CA private key: $CERTS_DIR/ca-key.pem"
  echo "  • Server certificate: $CERTS_DIR/server.pem"
  echo "  • Server private key: $CERTS_DIR/server-key.pem"
  echo "  • Traefik config: $TRAEFIK_DIR/dynamic.yml"
  echo ""

  echo "🔍 Server certificate details:"
  openssl x509 -in "$CERTS_DIR/server.pem" -text -noout | grep -A2 "Validity"
  echo ""
  echo "Subject Alternative Names:"
  openssl x509 -in "$CERTS_DIR/server.pem" -text -noout | grep -A1 "Subject Alternative Name" || echo "  (None found)"
  echo ""
}

# Show system trust instructions
show_trust_instructions() {
  log_info "To trust the CA certificate on your system:"
  echo ""

  case "$(uname -s)" in
  Linux*)
    echo "🐧 Linux (Ubuntu/Debian):"
    echo "  sudo cp $CERTS_DIR/ca.pem /usr/local/share/ca-certificates/docker-compose-dev-ca.crt"
    echo "  sudo update-ca-certificates"
    echo ""
    echo "🐧 Linux (CentOS/RHEL):"
    echo "  sudo cp $CERTS_DIR/ca.pem /etc/pki/ca-trust/source/anchors/docker-compose-dev-ca.crt"
    echo "  sudo update-ca-trust"
    ;;
  Darwin*)
    echo "🍎 macOS:"
    echo "  sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain $CERTS_DIR/ca.pem"
    echo ""
    echo "  Or use Keychain Access app:"
    echo "  1. Open Keychain Access"
    echo "  2. Drag $CERTS_DIR/ca.pem to System keychain"
    echo "  3. Double-click the certificate and set to 'Always Trust'"
    ;;
  CYGWIN* | MINGW32* | MSYS* | MINGW*)
    echo "🪟 Windows (run as Administrator):"
    echo "  certutil -addstore -f \"ROOT\" \"$(cygpath -w "$CERTS_DIR/ca.pem")\""
    echo ""
    echo "  Or use Certificate Manager (certmgr.msc):"
    echo "  1. Open Certificate Manager"
    echo "  2. Navigate to Trusted Root Certification Authorities > Certificates"
    echo "  3. Right-click > All Tasks > Import"
    echo "  4. Select $CERTS_DIR/ca.pem"
    ;;
  *)
    echo "❓ Unknown OS - please refer to your system's documentation for adding trusted CA certificates"
    ;;
  esac

  echo ""
  log_warning "You'll need to restart your browser after trusting the CA certificate"
}

# Main execution
main() {
  echo -e "${BLUE}🔐 Docker Compose SSL Certificate Generator${NC}"
  echo "=================================================="
  echo ""

  check_dependencies
  check_existing_certs
  create_directories
  generate_ca
  generate_server_cert
  generate_traefik_config
  set_permissions

  echo ""
  log_success "Certificates generated successfully!"
  echo ""

  show_certificate_info
  show_trust_instructions

  echo ""
  echo "🚀 Next steps:"
  echo "  1. Trust the CA certificate using the instructions above"
  echo "  2. Start your Docker Compose stack: docker-compose up -d"
  echo "  3. Test HTTPS access: curl https://oidc.test/.well-known/openid-configuration"
  echo ""
  echo "💡 Tip: Certificates are valid for 1 year. Run this script again to regenerate them."
}

# Run main function
main "$@"
