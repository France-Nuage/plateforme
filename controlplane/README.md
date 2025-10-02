# Control Plane

The control plane provides the core management API for cloud infrastructure and
identity operations, organized into GCP-style namespaces.

## Namespaces

### compute

Manages computational resources including virtual machine instances, disks,
networks, load balancers, and related infrastructure. This namespace handles the
core infrastructure layer where workloads run.

### iam

Handles authentication and authorization, including permission management, role
assignments, and access control policies. Provides the security layer that
controls who can access what resources and what actions they can perform.

### identity

Handles user and group management, including creating groups, managing
memberships, and sending user invitations. Provides functionality for organizing
users into groups and controlling group-based access patterns.

### resourcemanager

Manages the organizational hierarchy and project structure. Handles operations
on organizations, folders, and projects - the container resources that provide
structure and inheritance for access control and billing.

## Authentication & Authorization

### Overview

The controlplane implements a **transitional authentication and authorization
architecture** that intentionally separates concerns between JWT authentication
and resource authorization. This design provides time to implement SpiceDB
properly while maintaining a clean, future-proof API.

### Architecture

1. **JWT Authentication**: OIDC-compliant token validation using JWK keys
2. **IAM Context**: Per-request identity management with lazy claim validation
3. **Manual Authorization**: Organization-scoped access control via `user()` function
4. **Database Lookup**: Temporary user-organization mapping (will be replaced)

### Current Implementation

#### Authentication Flow

- **Token Extraction**: Bearer tokens extracted from `Authorization` headers
- **OIDC Validation**: JWT tokens validated against provider JWK keys with caching
- **IAM Injection**: Authentication context injected into all requests via Tower
middleware
- **Claim Access**: Validated JWT claims accessible through IAM context

#### Authorization Pattern

```rust
// In API handlers - manual organization scoping
let user = iam.user(&pool).await?;
// Filter resources by user.organization_id
// Validate requested resources belong to user's organization
```

## Development

### Generate sqlx query metadata

The project relies on sqlx query metadata to support offline compile-time
verification. This is used in CI to avoid having to bootstrap the database for
compiling the project. Use the following command to generate query metadata for
the workspace, including queries in tests:

```bash
cargo sqlx prepare --workspace -- --tests
```
