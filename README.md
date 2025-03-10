<!-- PROJECT LOGO -->
<p align="center">
  <a href="https://gitlab.com/groups/getbunker-france-nuage/france-nuage">
   <img src="./apps/mediakit/logo/animated-logo.gif" alt="France nuage Logo">
  </a>

<h3 align="center">France nuage</h3>

  <p align="center">
    The French cloud platform for modern applications and services.
    <br />
    <a href="https://france-nuage.fr"><strong>Learn more »</strong></a>
    <br />
    <br />
    <a href="https://france-nuage.fr">Website</a>
    ·
    <a href="https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/issues">Issues</a>
    ·
    <a href="https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/milestones">Roadmap</a>
  </p>
</p>

<p align="center">
   <a href="https://status.france-nuage.fr/"><img height="20px" src="https://uptime.betterstack.com/status-badges/v1/monitor/es5i.svg" alt="Uptime"></a>
   <a href="https://github.com/France-Nuage/plateforme"><img src="https://img.shields.io/github/stars/France-Nuage/plateforme" alt="Github Stars"></a>
   <a href="https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/blob/master/LICENCE"><img src="https://img.shields.io/badge/license-SSPL-purple" alt="License"></a>
   <a href="https://gitlab.com/getbunker-france-nuage/france-nuage/plateforme/-/graphs/master"><img src="https://img.shields.io/github/commit-activity/m/France-Nuage/plateforme" alt="Commits-per-month"></a>
   <a href="https://france-nuage.fr/"><img src="https://img.shields.io/badge/Pricing-Free-brightgreen" alt="Pricing"></a>
   <a href="https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/issues/?sort=milestone_due_desc&state=opened&first_page_size=100"><img src="https://img.shields.io/badge/Help%20Wanted-Contribute-blue"></a>
   <a href="https://contributor-covenant.org/version/1/4/code-of-conduct/"><img src="https://img.shields.io/badge/Contributor%20Covenant-1.4-purple" /></a>
</p>

Welcome to France nuage! Sign up to [france-nuage.fr](https://france-nuage.fr/) and start deploying your applications in our French cloud platform!

You should check our documentation website to know what France nuage is and what is our
vision: https://france-nuage.fr/solutions/perspectives

# 🐓 About France nuage

France nuage is a comprehensive cloud platform designed for modern applications with French sovereignty in mind.

- **Fully open-source**
- **One click deploy unified ecosytem**
- **RESTful API**
- **Complete platform control** via our intuitive control panel
- **Application orchestration** with built-in scaling and deployment options
- **Automated CI/CD pipelines**
- **Region-specific deployments** with data residency guarantees
- **On-Prem or Cloud**. Run locally, install on-premises, or use our self-service Cloud service (free tier available)
- **Modern dashboard**. Our dashboard app is intuitive for both technical and non-technical users
- **Sustainable project** since inception. Fork it, extend it, and help us build the best French cloud platform

[Learn more about France nuage](https://france-nuage.fr/entreprise/a-propos)

# 🚀 France nuage Cloud

France nuage Cloud allows you to create free cloud projects in minutes.

- **Free Autonomous Tier**: Available with no credit card required
- **No Product Limitations**: Unlimited users and applications in our platform
- **Self-Service Dashboard**: Create and monitor all your projects in one place
- **End-to-End Solution**: Full stack platform with database, auto-scaling, and storage
- **Usage-Based Pricing**: Pay-as-you-go for our Standard Cloud offering
- **Quick Provisioning**: Select your desired region in France and provision new resources in minutes

[Create your Project](https://plateforme.france-nuage.fr/auth/login) - [Contact a human](mailto:contact@france-nuage.fr)

# 🤔 Community Help

The [France nuage Documentation](https://france-nuage.fr/support/documentation) is a great place to start, or explore these other channels:

- [Gitlab Issues](https://gitlab.com/groups/getbunker-france-nuage/france-nuage/-/issues/?sort=milestone_due_desc&state=opened&first_page_size=100) (Report Bugs, Questions, Feature Requests)
- [GitHub Mirror](https://github.com/France-Nuage/plateforme)
- [Linkedin](https://www.linkedin.com/company/france-nuage) (Latest News)
- [Website](https://france-nuage.fr/) (Infos)
- [Platform](https://plateforme.france-nuage.fr/auth/login) (Login, sign up)

# 📌 Requirements

France nuage is built on modern technologies and supports most operating systems.

- **Proxmox**: Open source type 2 hypervisor based on Debian and KVM
- **Supported Databases**:
  - PostgreSQL 15+
- **Supported OS**:
  - Ubuntu LTS
  - CentOS / RHEL 8
  - macOS Catalina or newer
  - Windows 10/11
  - Docker (DockerHub + Dockerfile)
  - Other operating systems may also work, in the futur

# 🚧 Development Setup

## Prerequisites

Ensure you have the following installed:

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

## Running France nuage with Docker

To start the project using Docker, run:

```sh
docker compose up -d
```

## Running migrations

Once the services are up, run database migrations using:

```sh
docker compose exec control-plane node ace migration:run --force
```

## Stopping the services

To stop the services, run:

```sh
docker compose down
```

## Integration Tests (Playwright)

Integration tests are defined in the `platform` application. The playwright UI
can be started with the following command:

```sh
docker compose exec platform \
   npx playwright test --project=firefox --ui-host=0.0.0.0 --ui-port=39709
```

The UI is then accessible on [http://localhost:39709](http://localhost:39709)

## Connect to a REPL Session

AdonisJS offers an [application-aware REPL](https://docs.adonisjs.com/guides/digging-deeper/repl)
to interact with the control-plane from the command line. To start a new REPL
session in a development environment with docker-compose, run the following
command:

```sh
docker compose exec control-plane node ace repl
```

## Generate a token for the worker user

The project comes with a token preconfigured for the worker user. The token
might not be valid for a few different reasons:

- The AdonisJS application key (the `APP_KEY` environment variable) has been changed
- The token has expired (expiry scheduled in 2035)

The following command (executed in a REPL session) allows you to regenerate a token for the worker user:

```sh
(await (await import('#models/user')).default.accessTokens.create(await (await import('#models/user')).default.findByOrFail({ email: 'worker@france-nuage.fr' }), ['*'], { expiresIn: '10 years' })).value.release()
```

Be wary of using it in production as generating a long-lived token may be a security risk if exposed outside a controlled, private network.

# Related

[France nuage Cloud Status Page](https://status.france-nuage.fr/)

# ❤️ Contributing & Sponsoring

All security vulnerabilities should be reported in accordance with our Security Policy.

# License

France nuage is a premium open-source [Server Side Public License (SSPL) v1](./LICENCE) project made possible with support
from our passionate core team, talented contributors, and amazing Sponsors. Thank you all!

The license allows the free right to use, modify, create derivative works, and redistribute, with three simple
limitations:

- You may not provide the products to others as a managed service
- You may not circumvent the license key functionality or remove/obscure features protect

© France nuage
