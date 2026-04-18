# Frontend — Vue.js Instructions

> Project-wide rules and accounting-immutability frontend rules live in the root `CLAUDE.md`. This file covers Vue/Tailwind specifics.

## General

- Always prioritize readability, safety, and maintainability.
- Use strong typing and leverage TypeScript's type system for safety.
- **Never implement functional methods directly in the template** — all logic must be in the `<script setup>` block or composables. Templates may only display data as-is or perform simple inline expressions (e.g. string interpolation, ternary display). Never call server APIs or run business logic from the template.
- Write code with good maintainability practices, including comments on why certain design decisions were made.
- Handle form feedback (success and errors) gracefully and provide meaningful error messages.

## Error handling

- Provide meaningful error messages to users, especially for form validation errors. Use the `field-hint` class for additional guidance on form fields.
- For API errors, display user-friendly messages and log technical details for debugging.

## Design system

- **Atomic design**: Atomes > Molécules > Organismes. Accessible (WCAG) and internationalized (i18n).
- **SVG icons only**: always create or reuse SVG icons instead of using icon fonts or external libraries. Located as atomic components in `src/components/atoms/`.
- **Design tokens**: follow the design system tokens for typography, colors, and spacing as defined in `DESIGN.md`. Use Tailwind utility classes mapped to design tokens as the primary styling method. Use raw CSS variables only as a last resort when no Tailwind utility exists.
- Use SVG icons to underline KPIs, notices, and titles.
