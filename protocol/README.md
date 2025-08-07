# Protocol Buffer Definitions

This directory contains Protocol Buffer (protobuf) definitions that trace and validate compliance with Google Cloud APIs. It serves as the foundation for generating SDKs and ensuring API compatibility.

## Overview

The `protocol` directory uses [buf](https://buf.build/) to:
- **Validate compliance**: Ensures our implementation is a valid subset of Google Cloud APIs
- **Breaking change detection**: Prevents backward-incompatible changes
- **SDK generation**: Provides the foundation for generating client SDKs

## Usage

### Running Protocol Compliance Checks

To validate that your protocol definitions are compliant with Google Cloud APIs:

```bash
docker compose up protocol
```

This command:
1. Builds the protocol container
2. Clones the latest Google Cloud API definitions from googleapis
3. Runs `buf breaking` to compare our definitions against Google's official APIs
4. Reports any compliance issues

### Configuration

The protocol validation is configured in `buf.yaml`:
- **Linting**: Uses `STANDARD` rules for consistent protobuf style
- **Breaking changes**: Uses `FILE` level detection to catch incompatible changes

## Development Workflow

1. **Add/modify protocol definitions** in the appropriate directory structure
2. **Run compliance check**: `docker compose up protocol`
3. **Fix any reported issues** before proceeding
4. **Generate SDKs** (when tooling is implemented)

## Integration

The protocol service is integrated into the docker-compose setup with:
- Build caching for faster subsequent runs
- Profile `donotstart` to prevent automatic startup
- Multi-stage build support for different environments

## Future Plans

This directory will be extended to generate client SDKs for multiple languages while maintaining Google Cloud compatibility.

## Dependencies

- [buf](https://buf.build/) - Protocol buffer toolchain
- [googleapis](https://github.com/googleapis/googleapis) - Google Cloud API definitions
- Docker - For containerized validation