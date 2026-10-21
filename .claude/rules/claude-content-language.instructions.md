---
description: 'All .claude/ content (agents, rules, skills) must be written in English.'
applyTo: '.claude/**/*'
---

# .claude content language

- Every file under `.claude/` (agents, rules, skills) is written in **English**: frontmatter, descriptions, instructions and body.
- French remains allowed only inside quoted examples that must reproduce French product strings (UI labels, i18n keys, legal quotes), and in quoted repository paths.
- When touching an existing French `.claude/` file for any reason, translate it in the same commit.
- This rule does not apply to `docs/` or other project documentation, which stays in French.
