# Control Plane

## Package description

### hypervisor_connector

Provides a unified API contract for how to interact with a third-party
hypervisor.

### hypervisor_connector_proxmox

Provides a implementation of the hypervisor_connector contract for the
[proxmox hypervisor](https://www.proxmox.com/).

## server

This is the main binary of this project. It exposes a [gRPC](https://grpc.io/)
server serving the different RPC services defined in the project packages.
