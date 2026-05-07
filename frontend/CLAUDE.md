# Frontend — Vue.js Instructions

> Project-wide rules and accounting-immutability frontend rules live in the root `CLAUDE.md`. This file covers Vue/Tailwind specifics.

## General

- Always prioritize readability, safety, and maintainability.
- Use strong typing and leverage TypeScript's type system for safety.
- **Never implement functional methods directly in the template** — all logic must be in the backend. Just display data as-is or do simple calculations.
- Write code with good maintainability practices, including comments on why certain design decisions were made.
- Handle form feedback (success and errors) gracefully and provide meaningful error messages.

## Error handling

- Provide meaningful error messages to users, especially for form validation errors. Use the `field-hint` class for additional guidance on form fields.
- For API errors, display user-friendly messages and log technical details for debugging.

## Design system

- **Atomic design**: Atomes > Molécules > Organismes. Accessible (WCAG) and internationalized (i18n).
- **SVG icons only**: always create or reuse SVG icons instead of using icon fonts or external libraries. Located as atomic components in `src/components/atoms/`.
- **Design tokens**: follow the design system tokens for typography, colors, and spacing as defined in `DESIGN.md`. Use CSS variables for colors and font weights to maintain consistency.
- Use SVG icons to underline KPIs, notices, and titles.

## Modal Form Feedback (DDR-006 — Non-negotiable)

**Every `CModal` that contains a form or a destructive action MUST include a `CFormFeedback` component inside the `#footer` slot, placed before the action buttons.** This is a mandatory design rule — do not place feedback messages inside the modal body.

```vue
<!-- ✅ CORRECT — CFormFeedback in #footer before buttons -->
<CModal :open="showModal" title="…" @close="closeModal">
  <form @submit.prevent="save">
    <!-- form fields only — no feedback here -->
  </form>
  <template #footer>
    <CFormFeedback :status="feedback.status.value" :message="feedback.message.value" />
    <CBtn variant="ghost" @click="closeModal">{{ t('common.cancel') }}</CBtn>
    <CBtn variant="primary" :loading="saving" @click="save">{{ t('common.save') }}</CBtn>
  </template>
</CModal>

<!-- ❌ WRONG — feedback in modal body -->
<CModal …>
  <CFormFeedback … />  <!-- never here -->
  <form>…</form>
</CModal>
```

Use the `useFormFeedback()` composable to manage `status` and `message` state. Always call `feedback.reset()` when opening a modal. Reference implementations: `ContractFormModal.vue`, `AllocationKeysView.vue`, `ContractorsView.vue`.

---

# Tailwind CSS v4 — Vite setup

This project uses Tailwind CSS v4 with the official `@tailwindcss/vite` plugin. Tailwind v4 introduces a simplified setup that eliminates the need for PostCSS configuration and `tailwind.config.js` in most cases.

## Key points

- **No PostCSS configuration required** when using the Vite plugin.
- **No `tailwind.config.js` required** — configuration is done via CSS.
- **`@tailwindcss/vite` plugin** replaces the PostCSS-based approach.
- **CSS-first configuration** using `@theme` directive.
- **Automatic content detection** — no need to specify content paths.

## Setup (already in place)

```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [tailwindcss()],
})
```

```css
/* src/index.css (or equivalent) */
@import "tailwindcss";
```

## What NOT to do in Tailwind v4

- **Do NOT create `tailwind.config.js`.** Use CSS-first configuration via `@theme`.
- **Do NOT create `postcss.config.js` for Tailwind.** The Vite plugin handles it.
- **Do NOT use old directives** (`@tailwind base;` / `@tailwind components;` / `@tailwind utilities;`) — use a single `@import "tailwindcss";`.

## CSS-first configuration

```css
@import "tailwindcss";

@theme {
  --color-primary: #3b82f6;
  --color-secondary: #64748b;
  --font-sans: 'Inter', system-ui, sans-serif;
  --radius-lg: 0.75rem;
}

@utility content-auto {
  content-visibility: auto;
}

@variant hocus (&:hover, &:focus);
```

## Troubleshooting

- **Styles not applying**: verify the import is `@import "tailwindcss";` (not old directives), the CSS file is imported in your entry point, and the Vite config includes the `tailwindcss()` plugin. Clear cache: `rm -rf node_modules/.vite && npm run dev`.
- **Plugin not found**: `npm install @tailwindcss/vite`.

## Reference

- https://tailwindcss.com/docs/installation/using-vite
- https://tailwindcss.com/docs/upgrade-guide
