# Frontend — Vue.js Instructions

> Project-wide instructions live in the root `AGENTS.md` (aliased as `CLAUDE.md`). Matching `.claude/rules/` are auto-injected by the IDE when editing frontend files. This file covers Vue/Tailwind specifics not covered by a rule.

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

- **Atomic design**: Atoms > Molecules > Organisms, in `frontend/src/components/{atoms,molecules,organisms}/`. Accessible (WCAG) and internationalized (i18n).
- **SVG icons only**: always create or reuse SVG icons instead of using icon fonts or external libraries. Located as atomic components in `src/components/atoms/`.
- **Design tokens**: follow the design system tokens for typography, colors, and spacing as defined in `docs/DESIGN.md`. Use Tailwind utility classes mapped to design tokens as the primary styling method. Use raw CSS variables only as a last resort when no Tailwind utility exists.
- Use SVG icons to underline KPIs, notices, and titles.

## Catalogue lookup (mandatory before writing UI)

Before writing any new UI markup, grep the catalogue first: search `docs/DESIGN.md §2` and `frontend/src/components/{atoms,molecules,organisms}/` for an existing `C*` component that covers the pattern. Only build new markup if no catalogue component fits. Record in the spec or PR which `C*` components are reused, or justify why none fits.

## Form feedback (DDR-006 — non-negotiable)

**Never use `CFormFeedback` directly in templates.** Always use `CFormActions` (modal footer) or `CForm` (inline form).

- **Modal forms**: wrap all buttons in `CFormActions` inside the `#footer` slot — never put `CFormFeedback` in the modal body.
- **Delete button on the left**: add `class="mr-auto"` to the first `CBtn` inside `CFormActions`.
- **Inline forms**: use `CForm` with an `#actions` slot instead of a bare `<form>`.
- Use `useFormFeedback()` for state; call `feedback.reset()` when opening a modal.

## Styling rules

- No raw values (`px`, `hex`, `rem`) in `class`/`style` — use Tailwind utilities or CSS tokens.
- No `<style>` block in feature views — Tailwind utilities or catalogue components only.
- No direct `radix-vue` import in feature views — use `C*` wrappers.
- New component → add to catalogue in `docs/DESIGN.md §2` before use.
- Tailwind utilities are for **text only**; layout/visual structure belongs to design-system CSS components.

## API layer

- All API calls go through typed api-client functions in `frontend/src/api/` — never raw `fetch` in components.
- Handle loading, empty, success, and error states explicitly in every data-fetching flow.
