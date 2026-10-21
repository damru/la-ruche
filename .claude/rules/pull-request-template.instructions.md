---
description: 'Pull requests must follow .github/PULL_REQUEST_TEMPLATE.md and Conventional Commits.'
applyTo: '.github/**'
---

# Pull request conventions

- Every PR must be filled in following `.github/PULL_REQUEST_TEMPLATE.md` (all its sections, in order).
- Title follows Conventional Commits (`type(scope): summary`), same as every commit in the branch.
- PRs target `main`; never commit directly to `main`.
- The description must list: what changed, why, doc impact (foundational docs updated or untouched with justification), and testing evidence.
- If the PR changes schema, API contracts, or the design system, link the corresponding ADR/DDR and doc sections.
