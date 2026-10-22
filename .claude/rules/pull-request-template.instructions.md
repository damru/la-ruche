---
description: 'Pull requests must follow .github/pull_request_template.md (ALWAYS, when present) and Conventional Commits.'
applyTo: '**'
---

# Pull request conventions

- **ALWAYS use the PR template when one is present.** Before writing any PR body, read `.github/pull_request_template.md` (lowercase, GitHub's actual filename) and apply it **verbatim** — keep its structure, section order, header box, and footer exactly as written. Fill in the sections with the change's content; never replace the template with a free-form summary.
- If no template file exists in `.github/`, fall back to a structured Markdown body (Summary / Changes / Verification).
- Title follows Conventional Commits (`type(scope): summary`), same as every commit in the branch.
- PRs target `main`; never commit directly to `main`.
- The description must list: what changed, why, doc impact (foundational docs updated or untouched with justification), and testing evidence.
- Ship criteria checkboxes must be ticked honestly — if a check could not be run locally, leave it unticked and state why inline.
- If the PR changes schema, API contracts, or the design system, link the corresponding ADR/DDR and doc sections.
