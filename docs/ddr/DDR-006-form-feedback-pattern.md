# DDR-006 — Pattern de feedback formulaire

**Date :** 2026-05-30  
**Statut :** Adopté

## Contexte

Les formulaires modaux affichaient le retour d'action (succès / erreur) via `CFormFeedback` placé directement dans le `#footer` du modal, avant les boutons. Ce placement était incohérent selon les vues (dans le body pour certaines, dans le footer pour d'autres), et le composant apparaissait/disparaissait en flash sans animation.

Deux problèmes distincts :
1. **Position** : `CFormFeedback` pouvait se retrouver dans le body ou le footer, parfois avec des `style` inline (`flex:1`, `margin-right:auto`) pour corriger le layout.
2. **Animation** : l'apparition/disparition était instantanée — aucune transition.

## Décision

### 1. Animation push dans `CFormFeedback`

Transition CSS grid (`grid-template-rows: 0fr → 1fr`) sur un wrapper, couplée à `opacity`. Le contenu est révélé en "poussant" les éléments voisins vers le bas — pas de saut de layout.

### 2. Molecule `CFormActions`

`frontend/src/components/molecules/CFormActions.vue` — wrape les boutons d'action d'un formulaire et intègre `CFormFeedback` en dessous du rang de boutons. Props : `status`, `message`.

- Boutons dans un `<slot>` alignés à droite (`flex justify-end`)
- `CFormFeedback` en dessous, pleine largeur
- Bouton destructif à gauche : `class="mr-auto"` sur le premier `CBtn`

### 3. Molecule `CForm`

`frontend/src/components/molecules/CForm.vue` — wrapper `<form>` avec slot `#actions` qui rend `CFormActions`. Pour les formulaires inline (hors modal).

### Règle de placement

`CFormFeedback` **ne s'utilise jamais directement** dans les templates. Toujours passer par `CFormActions` (modal `#footer`) ou `CForm` (formulaire inline).

## Alternatives écartées

| Option | Raison du rejet |
|---|---|
| `CFormFeedback` avant les boutons dans `#footer` | Layout fragile, nécessite des hacks inline ; feedback trop loin des boutons |
| `CFormFeedback` dans le body du modal | Feedback séparé du point d'action ; scroll nécessaire sur modals longs |
| Toast de notification | Éphémère, non lié à l'action en cours, disparaît avant que l'utilisateur puisse lire |
| Animation fade seule | Pas de push — le layout saute, les boutons bougent brusquement |

## Fichiers concernés

- `frontend/src/components/atoms/CFormFeedback.vue` — animation ajoutée
- `frontend/src/components/molecules/CFormActions.vue` — nouveau
- `frontend/src/components/molecules/CForm.vue` — nouveau
- 17 vues migrées (toutes les vues avec formulaires modaux)
