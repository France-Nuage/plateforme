# Claude Code Documentation Guidelines

This document contains instructions for maintaining high-quality documentation
across the controlplane cargo workspace.

## Documentation Style Standards

### Module-Level Documentation (`//!`)

Keep module documentation concise and practical:

1. **Title**: Single-line description (no "# Title" header)
2. **Overview**: 2-3 sentence paragraph explaining what the module provides
3. **Usage notes**: Brief mention of key methods/patterns if relevant

Avoid:
- Verbose section headers (Key Features, Design Philosophy, etc.)
- Doc test examples unless they provide significant value
- Repetitive or obvious information

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

## Code Examples Style Guide

### Documentation Test Guidelines

- **Default Language**: Use ``` without specifying `rust` - it's the default
- **Test Execution**: Avoid `no_run` unless absolutely necessary for consistency
- **Hidden Lines**: Use `#` prefix to hide boilerplate from rendered docs while
keeping tests compilable
- **Error Handling**: Include proper error types in function signatures for
realistic examples

### Example Format

```rust
/// Example function documentation
///
/// ```
/// # use my_crate::MyStruct;
/// # async fn example() -> Result<(), my_crate::Error> {
/// let instance = MyStruct::new().await?;
/// # Ok(())
/// # }
/// ```
```

## Quality Checklist

- [ ] All new public APIs have comprehensive documentation
- [ ] Module-level docs explain the "why" not just the "what"  
- [ ] Examples add genuine value
- [ ] Cross-crate consistency maintained
- [ ] Standards compliance properly referenced
- [ ] Documentation tests use idiomatic style guide above

## Session-Based Adaptation

This CLAUDE.md file should be updated at the end of each significant coding
session to:

- Incorporate lessons learned from documentation patterns used
- Refine style guidelines based on real implementation needs
- Add project-specific conventions discovered during development
- Ensure guidelines remain practical and aligned with codebase evolution
