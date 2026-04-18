# DDR-002 — Couleurs & thèmes (light + dark)

**Date :** 2026
**Statut :** Accepté

---

## Contexte

L'utilisateur veut light + dark mode dès le MVP. Le design source proposait une palette lavande/charcoal en light uniquement. Il faut dériver un dark mode cohérent.

## Décision

### Stratégie dark mode
Dark mode via `[data-theme="dark"]` sur `<html>` + `prefers-color-scheme` en fallback.
Les tokens CSS sont redéfinis dans le sélecteur dark — zéro JavaScript requis pour le thème de base.

### Dérivation du dark mode
Principe Material You adapté : les surfaces s'assombrissent (pas d'inversion), la couleur primaire s'éclaircit pour maintenir le contraste sur fond sombre.

- Primary light `#643fd5` → dark `#c4b5fd` (pastel pour fond sombre)
- Surfaces : échelle `#111318 → #ffffff` inversée
- Ombres : réduites en dark (l'obscurité crée déjà la profondeur)

### Persistance du choix utilisateur
`localStorage.setItem('laruche-theme', 'dark' | 'light')`
Lu au `<script>` inline avant le rendu Vue → pas de flash.

## Conséquences
- Les composants ne référencent jamais de couleur en dur — uniquement des tokens CSS
- Les images/illustrations doivent avoir une variante dark ou être neutres
- Les graphiques (budget, charges) utilisent `primary` + `secondary-container` pour rester cohérents
