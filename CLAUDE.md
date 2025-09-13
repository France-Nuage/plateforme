# Plateforme Monorepo - Claude Code Guidelines

## General Principles

### Documentation Standards Adherence

When working within this monorepo, Claude must systematically consult and apply
documented standards throughout the implementation process. Project-specific
CLAUDE.md files contain critical guidelines that must be actively referenced
during development, not merely read at the beginning of a session.

### Implementation Process

1. **Standards Review**: Before implementing solutions, consult relevant CLAUDE.md
files for applicable guidelines
2. **Active Cross-Reference**: During implementation, verify that current
actions align with documented standards
3. **Problem-Solving Approach**: When encountering issues, check existing
project guidelines before applying generic solutions or workarounds

### Quality Assurance

Documented standards exist to maintain code quality and consistency across the
monorepo. Adhering to these guidelines is essential for maintaining the
integrity of each project within the workspace.

## Documentation

### Comprehensive Documentation Requirement

All code elements must be properly documented, including but not limited to:

- **Functions and methods**: Purpose, parameters, return values, error
conditions, and usage examples
- **Classes and objects**: Purpose, properties, design rationale, and typical
usage patterns  
- **Interfaces and contracts**: Definition, implementation requirements, and
expected behavior
- **Modules and packages**: Overview, key features, design philosophy, and usage
patterns
- **Public APIs**: Complete documentation with examples that demonstrate
real-world usage
- **Configuration and environment variables**: Purpose, expected values, and
impact on system behavior

Documentation should explain the "why" behind design decisions, not just the
"what" of implementation details.

### Documentation Coherence

When modifying code, analyze the impact on existing documentation to ensure
consistency and alignment. All documentation within a module, package, or API
should maintain coherent terminology, consistent explanations, and unified
design rationale. Code changes must be accompanied by corresponding
documentation updates to prevent contradictions or outdated information.
