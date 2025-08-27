# Claude Code Documentation Guidelines

This document contains instructions for maintaining high-quality documentation across the controlplane cargo workspace.

## Documentation Style Standards

### Module-Level Documentation (`//!`)

Structure:
1. **Title**: Clear, descriptive header
2. **Overview**: Brief description of what the module provides  
3. **Key Features**: Bulleted list of main capabilities
4. **Design Philosophy**: Core principles and architectural decisions
5. **Usage Pattern**: How the module is typically used (with useful code examples)

### Function/Struct Documentation (`///`)

- **Purpose**: Clear description of what the item does
- **Parameters**: Detailed parameter descriptions  
- **Returns/Errors**: What the function returns and error conditions
- **Examples**: Only include examples that provide significant value

## Code Examples Policy

- **Include**: Complex API integrations (Tower middleware setup)
- **Include**: Non-obvious usage patterns
- **Exclude**: Simple, self-explanatory functions
- **Exclude**: Examples that don't add meaningful value

## Project-Specific Guidelines

### Authentication Documentation
- Mention OIDC JWT token requirements
- Document Bearer token format
- Reference RFC 6750, RFC 7517, RFC 7519 as appropriate

### Environment Variables
- Document in bin crates only (server, synchronizer, etc.)
- Keep isolated from library crates (auth, etc.)

### Cross-Crate Consistency
- Use consistent terminology
- Keep authentication requirements synchronized
- Maintain coherent error handling explanations

## Quality Checklist

- [ ] All new public APIs have comprehensive documentation
- [ ] Module-level docs explain the "why" not just the "what"  
- [ ] Examples add genuine value
- [ ] Cross-crate consistency maintained
- [ ] Standards compliance properly referenced