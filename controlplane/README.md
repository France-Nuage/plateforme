# Control Plane

## Authentication & Authorization

### Overview

The controlplane implements a **transitional authentication and authorization architecture** that intentionally separates concerns between JWT authentication and resource authorization. This design provides time to implement SpiceDB properly while maintaining a clean, future-proof API.

### Architecture

```
JWT Token → Authentication (OIDC) → IAM Context → user() → Manual Authorization
```

1. **JWT Authentication**: OIDC-compliant token validation using JWK keys
2. **IAM Context**: Per-request identity management with lazy claim validation  
3. **Manual Authorization**: Organization-scoped access control via `user()` function
4. **Database Lookup**: Temporary user-organization mapping (will be replaced)

### Current Implementation

#### Authentication Flow
- **Token Extraction**: Bearer tokens extracted from `Authorization` headers
- **OIDC Validation**: JWT tokens validated against provider JWK keys with caching
- **IAM Injection**: Authentication context injected into all requests via Tower middleware
- **Claim Access**: Validated JWT claims accessible through IAM context

#### Authorization Pattern
```rust
// In API handlers - manual organization scoping
let user = iam.user(&pool).await?;
// Filter resources by user.organization_id
// Validate requested resources belong to user's organization
```

### Transitional Design Philosophy

This approach is **intentionally interim** to avoid premature optimization:

- **Authentication**: Robust OIDC implementation that will remain unchanged
- **Authorization**: Simple database model that intentionally avoids SpiceDB complexity
- **Manual Rights Checking**: Explicit organization scoping in API handlers
- **Future-Proof API**: Auth crate interface designed for seamless SpiceDB migration

### Migration Timeline

**Current State** (Transitional):
- JWT authentication via `auth` crate
- Database-backed user-organization mapping
- Manual authorization in API handlers using `iam.user()`

**Future State** (SpiceDB Integration):
- Same JWT authentication (no changes)
- SpiceDB relationship-based authorization  
- Declarative permission checks replacing manual scoping
- Remove database `users` table and `User` model

### Rationale

This transitional approach provides several benefits:

1. **Time Investment**: Allows proper SpiceDB research and implementation rather than hasty adoption
2. **API Stability**: Auth crate provides stable interface that won't require refactoring  
3. **Incremental Migration**: Authorization logic can be migrated gradually to SpiceDB
4. **Risk Mitigation**: Proven database approach while learning SpiceDB best practices
5. **Organization Boundaries**: Ensures proper resource isolation during transition

The database authorization model will be **completely removed** once SpiceDB integration is mature, making this a true transitional architecture rather than technical debt.

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
