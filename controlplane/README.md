# Control Plane

## Packages description

### hypervisor_connector

Provides a unified API contract for how to interact with a third-party
hypervisor.

### hypervisor_connector_proxmox

Provides a implementation of the hypervisor_connector contract for the
[proxmox hypervisor](https://www.proxmox.com/).

### hypervisor_resolver

Resolves at runtime the concrete hypervisor_connector contract implementation
to return for a given target.

### server

This is the main binary of this project. It exposes a [gRPC](https://grpc.io/)
server serving the different RPC services defined in the project packages.

## Generate sqlx query metadata

The project rely on sqlx query metadata to support offline compile-time
verification. This is used in CI in particular to avoid having to bootstrap the
database for compiling the project. Use the following command to generate query
metadata for the workspace, including queries in tests:

```bash
cargo sqlx prepare --workspace -- --tests
```
