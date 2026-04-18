# DDR-003 — Navigation

**Date :** 2026
**Statut :** Accepté

---

## Contexte

L'app a 10+ modules. Responsive obligatoire (copropriétaires votent depuis mobile). Les rôles ont des accès différents → la navigation doit s'adapter au rôle.

## Décision

### Mobile (< 768px) — Bottom Navigation
- 4 items max visibles (+ "Plus" si nécessaire)
- Items filtrés selon le rôle de l'utilisateur connecté
- Position fixed, hauteur 64px + safe-area-inset-bottom (notch iPhone)
- Icône SVG 24px + label 11px en DM Sans Medium

### Tablet (768–1024px) — Navigation Rail
- Sidebar réduite à 72px (icônes seules + tooltip au hover)
- Même items que mobile mais verticaux

### Desktop (> 1024px) — Sidebar complète
- Largeur 240px fixe
- Logo + nom copropriété en haut
- Items avec icône 20px + label
- Section utilisateur (avatar + rôle + logout) en bas
- Collapsible optionnel (post-MVP)

### Items de navigation par rôle

| Item | Icône | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|---|
| Tableau de bord | home | ✅ | ✅ | ✅ | ✅ |
| Assemblées | users | ✅ | ✅ | ✅ | ❌ |
| Budget & Charges | wallet | ✅ | ✅ | ✅ (les siennes) | ❌ |
| Documents | folder | ✅ | ✅ | ✅ | ❌ |
| Tâches | wrench | ✅ | ✅ | ❌ | ❌ |
| Consultations | message-circle | ✅ | ✅ | ✅ | ✅ |
| Sinistres | alert-triangle | ✅ | ✅ | ❌ | ❌ |
| Carnet | book-open | ✅ | ✅ | ❌ | ❌ |
| Contrats | file-text | ✅ | ❌ | ❌ | ❌ |
| Résidents | users-2 | ✅ | ❌ | ❌ | ❌ |

### Mobile — sélection des 4 items principaux par rôle
- **Syndic** : Dashboard · AG · Budget · Tâches
- **Conseil** : Dashboard · AG · Budget · Documents
- **Copropriétaire** : Dashboard · AG · Budget · Consultations
- **Locataire** : Dashboard · Consultations · Signalements · (vide)

## Conséquences
- Le composant `AppNav` reçoit le rôle en prop et filtre les items
- Vue Router : les routes sont définies mais les guards vérifient le JWT
- Les routes inaccessibles redirigent vers 403, pas vers 404 (information leakage)
