---
description: 'La Ruche design system conventions: component = layout, utility = text, no <style> in .vue, z-index via variables.'
applyTo: 'frontend/src/**/*.{vue,ts,css}'
---

# Design system

- Separation of concerns: a **component** carries layout/visual structure (design system classes, pure CSS); a Tailwind **utility** carries text only.
- No `<style>` blocks in `.vue` components: styles live in the design system CSS (base/design-system) or in declared `@utility` rules.
- `z-index` always goes through the design system `--z-*` variables; never a hard numeric value.
- The visual style follows `docs/DESIGN.md` (product identity, accessibility, anti-references); no third-party UI library on top of the design system.
