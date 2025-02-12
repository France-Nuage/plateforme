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

## Miscellaneous

### Connect to a REPL session

AdonisJS offers an [application-aware REPL](https://docs.adonisjs.com/guides/digging-deeper/repl) to interact with the
control-plane from the command line. To start a new REPL session in a development environment with docker-compose, run
the following command:

```sh
docker compose exec control-plane node ace repl
```

### Generate a token for the worker user

The project comes with a token preconfigured for the worker user. The token might not be valid for a few different
reasons:

- The AdonisJS application key (the `APP_KEY` environment variable) has been changed
- The token has expired (expiry scheduled in 2035)

The following command (executed in a REPL session) allows to regenerate a token for the worker user. The command can
also be tweaked to generate a token for a different user or for a different expiry. Be wary of using it in prod as
generating a long-lived token may be a security threat if the token is exposed outside a controlled, private network.

```sh
(await (await import('#models/user')).default.accessTokens.create(await (await import('#models/user')).default.findByOrFail({ email: 'worker@france-nuage.fr' }), ['*'], { expiresIn: '10 years' })).value.release()
```
