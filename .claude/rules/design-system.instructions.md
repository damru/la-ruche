---
description: 'Conventions design system La Ruche : composant = layout, utilitaire = texte, pas de <style> dans les .vue, z-index via variables.'
applyTo: 'frontend/src/**/*.{vue,ts,css}'
---

# Design system

- Séparation de responsabilité : un **composant** porte le layout/la structure visuelle (classes du design system, CSS pur) ; un **utilitaire** Tailwind ne porte que du texte.
- Aucun bloc `<style>` dans les composants `.vue` : les styles vivent dans le design system CSS (base/design-system) ou dans `@utility` déclarés.
- Les `z-index` passent toujours par les variables `--z-*` du design system ; jamais de valeur numérique en dur.
- Le style visuel suit les principes de `docs/DESIGN.md` (identité produit, accessibilité, anti-références) ; aucune bibliothèque UI tierce par-dessus le design system.
