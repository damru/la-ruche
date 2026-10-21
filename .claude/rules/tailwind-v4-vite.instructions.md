---
description: 'Conventions Tailwind CSS v4 du projet La Ruche : intégration Vite, tokens @theme, utilitaires texte uniquement.'
applyTo: 'frontend/src/**/*.{vue,ts,css}'
---

# Tailwind CSS v4 — conventions projet

## Intégration

- Tailwind v4 via le plugin `@tailwindcss/vite` (Vite), pas de PostCSS ni de CLI.
- Aucun `tailwind.config.js` : la configuration se fait en CSS.
- Entrée unique : `frontend/src/styles/tailwind.css`.

## Tokens de design

- Les tokens (couleurs, typographie, espacements, radii, ombres, z-index) sont définis via `@theme inline` dans `frontend/src/styles/tailwind.css`, en référence aux variables CSS du design system (`frontend/src/styles/base.css`).
- Toute nouvelle valeur de token passe par `@theme inline`, jamais par une valeur magique dans un composant.

## Utilitaires personnalisés

- Créer un utilitaire nommé avec `@utility` (ex. `@utility text-title`).
- Règle fondatrice : les utilitaires Tailwind servent au **texte uniquement** (tailles, graisses, couleurs de texte). Le layout et la structure visuelle relèvent des composants CSS du design system.
- Les classes `.tone--*` (tons de couleur) sont du **CSS pur** du design system, pas des utilitaires Tailwind.

## Interdits

- Pas de `@apply` en cascade pour reconstruire des composants dans le markup.
- Pas de configuration JS résiduelle, pas de plugins tiers non validés par ADR.
- Pas de styles inline dans les templates `<template>` des composants Vue.
