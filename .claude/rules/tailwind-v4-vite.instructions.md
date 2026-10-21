---
description: 'La Ruche Tailwind CSS v4 conventions: Vite integration, @theme tokens, text-only utilities.'
applyTo: 'frontend/src/**/*.{vue,ts,css}'
---

# Tailwind CSS v4 — project conventions

## Integration

- Tailwind v4 via the `@tailwindcss/vite` plugin (Vite). No PostCSS, no CLI.
- No `tailwind.config.js`: configuration lives in CSS.
- Single entry point: `frontend/src/styles/tailwind.css`.

## Design tokens

- Tokens (colors, typography, spacing, radii, shadows, z-index) are defined via `@theme inline` in `frontend/src/styles/tailwind.css`, referencing the design system CSS variables (`frontend/src/styles/base.css`).
- Any new token value goes through `@theme inline`, never a magic value in a component.

## Custom utilities

- Create named utilities with `@utility` (e.g. `@utility text-title`).
- Foundational rule: Tailwind utilities are for **text only** (sizes, weights, text colors). Layout and visual structure belong to the design system CSS components.
- The `.tone--*` classes (color tones) are **pure CSS** from the design system, not Tailwind utilities.

## Forbidden

- No `@apply` cascades rebuilding components in markup.
- No residual JS configuration, no third-party plugins unless validated by an ADR.
- No inline styles in Vue `<template>` blocks.
