# Claude Code Documentation Guidelines

This document contains instructions for maintaining high-quality documentation
across the controlplane cargo workspace.

## Data Access: fabrique First

All database access in this workspace goes through the `fabrique` ORM. Raw SQL
is the exception, never the default.

- Entities derive `Model` (and `Factory` when tests seed them) with
  `#[fabrique(table = "...")]`; foreign key fields carry
  `#[fabrique(belongs_to = Parent)]` so typed joins are available.
- Reads and writes use the typestate query builder
  (`Entity::query().select() / .insert() / .update()`, `r#where`,
  `join::<Child>()`, `on_conflict`, `returning`) and the `Persist` / `Delete`
  traits (`create`, `save`, `delete`, `destroy`). Tests seed through factories
  and the builder, not hand-written SQL.
- Raw SQL (`sqlx::query`, `sqlx::query_as`, `sqlx::query_scalar`) is allowed
  ONLY when the builder cannot express the statement, and every raw query MUST
  carry a comment naming the limitation that forces it. Known legitimate
  cases:
  - casts on bind parameters (e.g. `$1::citext`);
  - jsonb functions and anti-joins (`jsonb_each_text`, `NOT EXISTS`);
  - `ON CONFLICT` targeting a non-primary-key unique constraint (the builder
    only supports conflicts on the primary key);
  - `DELETE` statements whose `rows_affected()` result drives the logic;
  - aggregates (`COUNT`, `SUM`, ...). When a count is only compared to zero,
    prefer an existence check with `.first()` instead.
- Design around builder limitations instead of falling back to raw SQL:
  `order_by` accepts a single column (sort multi-column orderings in Rust when
  the result set is small), and a query returns the columns of a single model
  (fetch related models separately and pair them in memory).

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
