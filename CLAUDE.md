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

## Trust center public / ISMS — synchronisation obligatoire (ISMS-sync)

France Nuage publie un **trust center public** — déclaration d'applicabilité (SoA ISO/IEC 27001:2022, 93 contrôles) et 18 politiques de sécurité — dans le dépôt `france-nuage-website` (`src/documentation/docs/securite/`). Ces pages font des **déclarations factuelles vérifiables** sur l'infrastructure réelle décrite dans CE dépôt. Le risque n°1 du trust center est la **fausse déclaration** : dès que l'infra réelle diverge du trust center publié, une page publique devient un risque juridique.

**Règle.** Toute modification de ce dépôt qui touche un fait de sécurité publié DOIT être répercutée dans le trust center, dans la même unité de travail (ou une issue de suivi explicitement liée). Ne jamais laisser l'infra avancer sans mettre à jour la déclaration correspondante.

**Où répercuter** (dépôt `france-nuage-website`) :

- Statut du contrôle concerné dans la SoA : `src/documentation/docs/securite/declaration-applicabilite-fr.md` **et** `statement-of-applicability-en.md`.
- Politique thématique concernée : `src/documentation/docs/securite/politiques/<sujet>-fr.md` **et** `<sujet>-en.md`.

**Garde automatique.** Le test `tests/isms-drift.spec.ts` (exécuté par le job `test` du site, pas de job dédié) garde l'**intégrité interne** du trust center : 93 contrôles, comptes de statut qui partitionnent, cohérence politiques ↔ SoA, invariants d'honnêteté. Ce test **ne peut pas voir ce dépôt** : la synchronisation infra → trust center est **manuelle** et relève de l'auteur du changement.

**Honnêteté (non négociable).** Ne jamais sur-déclarer. Un contrôle non pleinement opérationnel se déclare « Partiel » ou « Planifié », jamais « En place ». Aucune certification non détenue (ISO 27001 = « aligné / auto-évalué / visé », jamais « certifié »). Aucun « chiffrement de bout en bout » pour du stockage général (les clés sont détenues côté plateforme).

**Faits publiés portés par `france-nuage-plateforme`** (BFF Rust, auth, autorisation, développement) :

- SSO centralisé Keycloak, **en migration vers l'IAM souverain FerrisKey** → A.5.16, politique `controle-acces` / `access-control`.
- OIDC / JWT en **client confidentiel** (architecture BFF), session chiffrée `httpOnly`, anti-rejeu ; MFA au niveau du fournisseur d'identité → A.8.5 / A.8.26, politique `controle-acces` / `access-control`.
- RBAC **lecture seule par défaut**, droits d'administration **faisant autorité en base** (jamais depuis un jeton) → A.8.2 ; autorisation fine par relations testée en CI → A.5.18.
- Cycle de développement sécurisé : lint bloquant, langage à sûreté mémoire (Rust), garde-fous de code (anti-mock, zeroization, rédaction des secrets dans les logs), tests d'autorisation en CI → A.8.25–A.8.29, politique `developpement-securise` / `secure-development`.
- Secrets applicatifs au repos : **chiffrement d'enveloppe authentifié XChaCha20-Poly1305** → A.8.24, politique `cryptographie` / `cryptography`.
- Séparation qualification / production, promotion limitée à la branche principale → A.8.31 / A.8.4, politique `gestion-changements` / `change-management`.
