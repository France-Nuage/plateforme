# France Nuage

## Development Setup

### Prerequisites

Ensure you have the following installed:

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

### Running France Nuage with Docker

To start the project using Docker, run:

```sh
docker compose up -d
```

### Running migrations

Once the services are up, run database migrations using:

```sh
docker compose exec control-plane node ace migration:run --force
```

### Stopping the services

To stop the services, run:

```sh
docker compose down
```
