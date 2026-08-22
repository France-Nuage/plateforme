# France Nuage

![France nuage Logo](./mediakit/logo/animated-logo.gif)

**The open-source French cloud platform — a self-hostable, re-internalisable
alternative to AWS, Azure, Google Cloud and OVHcloud.**

[**Learn more**](https://france-nuage.fr) • [Website](https://france-nuage.fr) •
[Issues](https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/issues)
• [Roadmap](https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/milestones)

## Status & Metrics

[![pipeline status](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/badges/master/pipeline.svg)](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/commits/master)
[![coverage report](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/badges/master/coverage.svg)](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/commits/master)
[![Latest Release](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/badges/release.svg)](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/releases)
[![Uptime](https://uptime.betterstack.com/status-badges/v1/monitor/es5i.svg)](https://status.france-nuage.fr/)
[![Github Stars](https://img.shields.io/github/stars/France-Nuage/plateforme)](https://github.com/France-Nuage/plateforme)
[![License](https://img.shields.io/badge/license-SSPL-purple)](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/blob/master/LICENCE)
[![Commits per month](https://img.shields.io/github/commit-activity/m/France-Nuage/plateforme)](https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/graphs/master)
[![Pricing](https://img.shields.io/badge/Pricing-Free-brightgreen)](https://france-nuage.fr/)
[![Help Wanted](https://img.shields.io/badge/Help%20Wanted-Contribute-blue)](https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/issues/?sort=milestone_due_desc&state=opened&first_page_size=100)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-1.4-purple)](https://contributor-covenant.org/version/1/4/code-of-conduct/)

---

Welcome to France nuage! Sign up to [france-nuage.fr](https://france-nuage.fr/)
and start deploying your applications in our French cloud platform!

You should check our documentation website to know what France nuage is and what
is our vision: <https://france-nuage.fr/solutions/perspectives>

## 🐓 About France nuage

France nuage is a comprehensive cloud platform designed for modern applications
with French sovereignty in mind.

- **Fully open-source**
- **One click deploy unified ecosytem**
- **RESTful API**
- **Complete platform control** via our intuitive control panel
- **Application orchestration** with built-in scaling and deployment options
- **Automated CI/CD pipelines**
- **Region-specific deployments** with data residency guarantees
- **On-Prem or Cloud**. Run locally, install on-premises, or use our self-service
Cloud service (free tier available)
- **Modern dashboard**. Our dashboard app is intuitive for both technical and
non-technical users
- **Sustainable project** since inception. Fork it, extend it, and help us build
the best French cloud platform

[Learn more about France nuage](https://france-nuage.fr/entreprise/a-propos)

## 🚀 France nuage Cloud

France nuage Cloud allows you to create free cloud projects in minutes.

- **Free Autonomous Tier**: Available with no credit card required
- **No Product Limitations**: Unlimited users and applications in our platform
- **Self-Service Dashboard**: Create and monitor all your projects in one place
- **End-to-End Solution**: Full stack platform with database, auto-scaling, and
storage
- **Usage-Based Pricing**: Pay-as-you-go for our Standard Cloud offering
- **Quick Provisioning**: Select your desired region in France and provision new
resources in minutes

[Create your Project](https://plateforme.france-nuage.fr/auth/login) •
[Contact a human](mailto:contact@france-nuage.fr)

## 🤔 Community Help

The [France nuage Documentation](https://france-nuage.fr/support/documentation)
is a great place to start, or explore these other channels:

- [Gitlab Issues](https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/issues/?sort=milestone_due_desc&state=opened&first_page_size=100)
(Report Bugs, Questions, Feature Requests)
- [GitHub Mirror](https://github.com/France-Nuage/plateforme)
- [Linkedin](https://www.linkedin.com/company/france-nuage) (Latest News)
- [Website](https://france-nuage.fr/) (Infos)
- [Platform](https://plateforme.france-nuage.fr/auth/login) (Login, sign up)

## 📌 Requirements

France Nuage is built on modern technologies and supports most operating systems.

- **Proxmox**: Open source type 2 hypervisor based on Debian and KVM
- **Supported Databases**: PostgreSQL 15+
- **Supported OS**: Ubuntu LTS, CentOS / RHEL 8, macOS Catalina or newer, Windows
10/11, Docker (DockerHub + Dockerfile)
- Other operating systems may also work, in the future

## 🚧 Development Setup

### Prerequisites

Ensure you have the following installed:

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)
- [GitLeaks](https://github.com/gitleaks/gitleaks)

### Running France nuage with Docker

To start the project using Docker, run:

```sh
docker compose up -d
```

### Stopping the services

To stop the services, run:

```sh
docker compose down
```

### Seeding the database

Note: you need to supply valid proxmox data through environment variables. You
can setup a local virtualized proxmox instance or provision one on an external
cloud provider. Active France Nuage contributors can request access to one of
France Nuage development proxmox hypervisors.

The variables defaults to [pvedev-dc03](https://pvedev-dc03-internal.france-nuage.fr)
but its authorization token is not provided for security reasons.

```sh
docker compose exec postgres sh -c "
psql -U postgres -d postgres \
  -v url=\"'\$PROXMOX_DEV_URL'\" \
  -v token=\"'\$PROXMOX_DEV_AUTHORIZATION_TOKEN'\" \
  -v storage=\"'\$PROXMOX_DEV_STORAGE_NAME'\" \
  -f /home/seed.sql
"
```

### Billing / Stripe (dev)

Le controlplane intègre la facturation Stripe (checkout, webhooks, abonnements,
provisioning). En local, on travaille contre une **sandbox Stripe dédiée**
(nommée « test »), jamais la production. Le catalogue déclaratif
[`controlplane/catalog/catalog.yaml`](controlplane/catalog/catalog.yaml) est
la source de vérité : le controlplane le réconcilie dans Stripe **à son
démarrage** (dès que `STRIPE_SECRET_KEY` est présent), ce qui permet de
régénérer la sandbox à l'identique. Pour re-synchroniser à la main après avoir
édité le catalogue, sans redémarrer : `docker compose exec controlplane server
catalog sync`.

**Prérequis :**

- [Stripe CLI](https://stripe.com/docs/stripe-cli) installée et authentifiée
  sur la sandbox « test » (`stripe login`).
- Stack de dev démarrée (`docker compose up -d`).
- `.env` racine renseigné à partir de [`.env.example`](.env.example), avec au
  minimum `STRIPE_SECRET_KEY` pointant sur la sandbox (`sk_test_...`).

**Démarrage :**

```sh
./stripe/dev-billing.sh
```

Ce script est l'équivalent local du sidecar `stripe listen` déployé dans les
environnements éphémères : il donne à Stripe un chemin vers le webhook du
controlplane, qui n'a pas d'URL publique en local. Il :

1. récupère le secret de signature webhook (`whsec_...`) et l'écrit dans
   `.env` (`STRIPE_WEBHOOK_SECRET`) ;
2. démarre `stripe listen`, qui relaie les événements Stripe vers l'endpoint
   webhook local du controlplane
   (`http://localhost:50053/webhooks/stripe`).

> **Note :** le controlplane valide la signature des webhooks. Si le script
> met à jour `STRIPE_WEBHOOK_SECRET`, relance le controlplane pour qu'il prenne
> la nouvelle valeur : `docker compose up -d controlplane`. Laisse le terminal
> `stripe listen` ouvert pendant les tests de paiement (carte de test
> `4242 4242 4242 4242`).

Sans les variables `STRIPE_*`, la facturation est simplement désactivée : le
controlplane démarre normalement, seul le flux de paiement est indisponible.

### Services managés — versions déployables (dev)

Un service du catalogue n'est **déployable** qu'une fois qu'une version de son
chart est connue (coordonnées + schéma du formulaire de déploiement). Le
controlplane découvre ces versions **au démarrage**, en lisant les charts depuis
le registry OCI (GitLab Container Registry) déclaré par `oci_reference` dans le
catalogue. Le schéma du formulaire vit dans le chart, sous `frn/`.

Cette découverte a besoin d'un identifiant de **lecture** du registry : un deploy
token GitLab avec le scope `read_registry`, renseigné dans `.env` via
`CHARTS_REGISTRY_USER` / `CHARTS_REGISTRY_TOKEN` (voir [`.env.example`](.env.example)).
Sans eux, la découverte est simplement ignorée (aucun service déployable en
local). Les charts sans schéma `frn/` restent déployables : leur formulaire est
alors vide.

### SSL Certificates

For HTTPS communication between services, this project uses a self-signed
certificate authority (CA) and server certificates. The certificate generation
is handled by the included script.

**Automatic Certificate Generation:**

```bash
./generate-certs.sh
```

**Manual Certificate Trust (if needed):**

- **macOS:** `sudo security add-trusted-cert -d -r trustRoot -k
/Library/Keychains/System.keychain certs/ca.pem`
- **Linux:** Copy `certs/ca.pem` to `/usr/local/share/ca-certificates/` and run
`sudo update-ca-certificates`
- **Windows:** Use `certutil -addstore -f "ROOT" certs/ca.pem` (as Administrator)

**Certificate Details:**

- **Validity:** 1 year for server certificates, 10 years for CA
- **Domains:** Supports `*.localhost`, `host.docker.internal`, and service names
- **Location:** Certificates are stored in `certs/` directory
- **Traefik:** Configured via `traefik/dynamic.yml` for automatic HTTPS

### Running on a local Kubernetes cluster (kind)

The platform is deployed through its Helm chart (`helm/plateforme`), the same way
it runs in CI and production. To exercise this path locally, create a
[kind](https://kind.sigs.k8s.io/) cluster wired for ingress. The port mappings
expose the cluster on `localhost:80/443`, and the `ingress-ready` label lets the
ingress controller schedule on the node:

```sh
kind create cluster --name france-nuage-local --config=- <<'EOF'
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
    kubeadmConfigPatches:
      - |
        kind: InitConfiguration
        nodeRegistration:
          kubeletExtraArgs:
            node-labels: "ingress-ready=true"
    extraPortMappings:
      - containerPort: 80
        hostPort: 80
        protocol: TCP
      - containerPort: 443
        hostPort: 443
        protocol: TCP
EOF
```

Then install the ingress controller. Production serves the platform through
`nginx`-class ingresses, so we install `ingress-nginx` locally to match. Wait for
the controller to become ready before deploying the chart:

```sh
kubectl apply -f https://kind.sigs.k8s.io/examples/ingress/deploy-ingress-nginx.yaml

kubectl wait --namespace ingress-nginx \
  --for=condition=ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=120s
```

The chart exposes the console, control plane and Keycloak as the `console`, `api`
and `auth` subdomains of a configurable `ingress.baseDomain`. Locally we use the
reserved `france-nuage.test` domain, so point its subdomains to the cluster:

```sh
echo "127.0.0.1 console.france-nuage.test api.france-nuage.test \
auth.france-nuage.test" | sudo tee -a /etc/hosts
```

## 📁 Architecture

For developers working on the platform:

- [`protocol/`](./protocol/README.md) - Protocol Buffer definitions and Google
Cloud API compliance validation

## Related

[France nuage Cloud Status Page](https://status.france-nuage.fr/)

## ❤️ Contributing & Sponsoring

All security vulnerabilities should be reported in accordance with our Security
Policy.

## License

France nuage is a premium open-source [Server Side Public License (SSPL) v1](./LICENCE)
project made possible with support from our passionate core team, talented
contributors, and amazing Sponsors. Thank you all!

The license allows the free right to use, modify, create derivative works, and
redistribute, with two simple limitations:

- You may not provide the products to others as a managed service
- You may not circumvent the license key functionality or remove/obfuscate
features protection

© France Nuage 2025
