# Spécification technique — EP-019-S09

**Story :** Régler le **délai de fermeture automatique des modales après succès** — de `0` (fermeture immédiate, fenêtre de succès désactivée) à `10` secondes, **défaut 3 s**, dans le même esprit que la préférence d'apparence (EP-019-S04), persisté côté frontend.

**Épic :** EP-019 — Profil & préférences
**Réf. PRD :** US-PP-008
**Réf. design :** DDR-006 (fenêtre de succès + auto-close à 3 s fixe)
**État :** PROPOSÉ — story backlog, pas encore d'implémentation.

Story 100 % frontend — aucune migration, aucun changement backend. La persistance suit le précédent établi par le thème et les préférences de notifications : `localStorage` (déplacer côté serveur quand la persistance serveur des préférences sera en place, cf. écart connu EP-019-S03).

---

## 1. Quoi (résumé fonctionnel)

- Une **nouvelle préférence utilisateur** dans l'onglet `Apparence` de la page Profil : un **sélecteur de délai** (0 à 10 secondes, pas de 1 s) pour la fermeture automatique des modales après une validation réussie.
- **Défaut 3 s** affiché lisiblement avant toute personnalisation (AC-PP-008-B).
- **Valeur 0 = fermeture immédiate** : pas de fenêtre de succès, pas d'animation de compte à rebours — la modale se ferme dès le succès (AC-PP-008-C).
- Le délai s'applique **uniformément** à toutes les modales utilisant le pattern DDR-006 (AC-PP-008-D) — pas de réglage par écran.
- **Effet immédiat** : changer le délai s'applique sans rechargement, y compris à une modale déjà ouverte en attente de fermeture (AC-PP-008-E) — voir §5 pour le mécanisme.
- Persistance `localStorage`, restaurée à la reconnexion (AC-PP-008-F).
- Le **reset global des préférences** (EP-019-S07, quand livré) ramène le délai à 3 s (AC-PP-008-G) — prévoir le point d'extension dans le store dès maintenant.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement | Constat |
|---|---|---|
| Constante délai fixe | `frontend/src/composables/useFormFeedback.ts` L5 — `SUCCESS_AUTO_CLEAR_MS = 3000` | ✅ existe, utilisée comme défaut |
| Timer d'auto-close | `frontend/src/composables/useAutoClose.ts` — `schedule(fn, delay = SUCCESS_AUTO_CLEAR_MS)` | ✅ `delay` est déjà un paramètre ; les ~17 modales consommatrices appellent `schedule(fn)` sans délai |
| Persistance préférence thème | `frontend/src/stores/ui.ts` L76-95 (`readThemePreference` / `persistThemePreference`, clé `laruche-theme`, `auto` = clé supprimée) | ✅ pattern à reproduire |
| Persistance préférences notif | `ui.ts` L12-72 (`laruche-notif-prefs`, JSON, fallback défauts sur corruption) | ✅ second pattern à reproduire |
| Onglet Apparence | `frontend/src/features/profile/ProfileView.vue` L87-108 (script) + template `activeTab === 'appearance'` | ✅ carte `Préférences` avec radiogroup thème — le sélecteur de délai s'ajoute dans cette carte |
| Animation compte à rebours bouton | `frontend/src/components/atoms/CBtn.vue` L362-376 — `.c-btn--closing::before` avec `animation: c-btn-fill 3s linear forwards` (3 s **en dur** dans le CSS) | ⚠️ la durée de l'animation doit suivre le délai choisi, sinon l'animation déborde ou disparaît trop tôt |
| `closing` sur les boutons | `CBtn.vue` L17, L171, L188-204 + binding `:closing="feedback.status.value === 'success'"` dans les modales (ex. `CDeleteConfirmModal.vue` L79) | ✅ le libellé « Fermeture… » et le remplissage sont déjà câblés au statut succès |
| i18n préférences | `frontend/src/i18n/locales/fr.json` — bloc `settings.preferences` (L1518+) | ✅ à étendre (une seule locale `fr` dans le repo) |
| Reset préférences | `ui.ts` — `resetNotifPrefs()` existe ; `resetAllPrefs()` sera ajouté par EP-019-S07 | ⚠️ prévoir l'extension `resetAppearance`-style pour le délai (voir §4.1) |

## 3. Modèle de données / Backend

Aucun.

## 4. Frontend

### 4.1 `frontend/src/stores/ui.ts` — préférence `autoCloseDelaySec`

Suivre exactement le pattern thème (lecture défensive + persistance + suppression de la clé à la valeur par défaut) :

```ts
const AUTO_CLOSE_DELAY_STORAGE_KEY = 'laruche-auto-close-delay'
export const AUTO_CLOSE_DELAY_DEFAULT_SEC = 3
export const AUTO_CLOSE_DELAY_MIN_SEC = 0
export const AUTO_CLOSE_DELAY_MAX_SEC = 10

function readAutoCloseDelay(): number {
  if (typeof window === 'undefined') return AUTO_CLOSE_DELAY_DEFAULT_SEC
  const raw = Number(localStorage.getItem(AUTO_CLOSE_DELAY_STORAGE_KEY))
  if (!Number.isInteger(raw) || raw < AUTO_CLOSE_DELAY_MIN_SEC || raw > AUTO_CLOSE_DELAY_MAX_SEC) {
    return AUTO_CLOSE_DELAY_DEFAULT_SEC
  }
  return raw
}

function persistAutoCloseDelay(seconds: number) {
  if (typeof window === 'undefined') return
  if (seconds === AUTO_CLOSE_DELAY_DEFAULT_SEC) {
    localStorage.removeItem(AUTO_CLOSE_DELAY_STORAGE_KEY)
  } else {
    localStorage.setItem(AUTO_CLOSE_DELAY_STORAGE_KEY, String(seconds))
  }
}
```

Dans le store :

```ts
// ── Modal auto-close delay ──
const autoCloseDelaySec = ref<number>(readAutoCloseDelay())

function setAutoCloseDelay(seconds: number) {
  if (!Number.isInteger(seconds) || seconds < AUTO_CLOSE_DELAY_MIN_SEC || seconds > AUTO_CLOSE_DELAY_MAX_SEC) {
    return
  }
  autoCloseDelaySec.value = seconds
  persistAutoCloseDelay(seconds)
}

/** Delay in ms for timers/animations; 0 disables the success window entirely. */
const autoCloseDelayMs = computed(() => autoCloseDelaySec.value * 1000)
```

Exporter `autoCloseDelaySec`, `autoCloseDelayMs`, `setAutoCloseDelay` (+ les constantes). **Point d'extension reset** (pour EP-019-S07, AC-PP-008-G) : le délai se réinitialise via `setAutoCloseDelay(AUTO_CLOSE_DELAY_DEFAULT_SEC)` — même endroit futur que `resetAppearance()`. Ne pas créer `resetAllPrefs` ici (reste la responsabilité de S07).

### 4.2 `frontend/src/composables/useAutoClose.ts` — délai depuis le store

`useAutoClose()` lit le store UI (Pinia actif dans les composants) et remplace le défaut fixe :

```ts
import { useUiStore } from '@/stores/ui'

export function useAutoClose() {
  const ui = useUiStore()
  // ...
  function schedule(fn: () => void, delay = ui.autoCloseDelayMs): void {
    if (delay <= 0) { fn(); return }   // AC-PP-008-C : fermeture immédiate, sans fenêtre de succès
    // ... setTimeout existant
  }
}
```

Conséquence pour `delay = 0` : `schedule(fn)` exécute `fn` immédiatement — les modales ferment instantanément au succès. Le statut `success` de `useFormFeedback` reste transitoire (jamais visible), ce qui désactive de facto la fenêtre de succès et l'animation `closing` (le bouton n'a pas le temps de passer en `closing`).

**Aucune modification des ~17 modales consommatrices** : elles appellent `schedule(fn)` sans délai, le défaut provient désormais du store. AC-PP-008-D (uniformité) est garantie structurellement.

**Effet immédiat sur une modale déjà ouverte (AC-PP-008-E)** : si le délai change alors qu'un timer est en cours, le timer en cours **n'est pas** recalculé (fermeture à l'échéance prévue au moment du succès). Le nouveau délai s'applique au **prochain** succès. Retenir ce comportement (documenté §5) — recalculer un `setTimeout` en cours ajouterait une complexité disproportionnée pour un cas limite (changer une préférence depuis l'onglet Profil pendant qu'une modale de succès est ouverte ailleurs est impossible : la modale bloque la page).

### 4.3 `frontend/src/composables/useFormFeedback.ts`

`SUCCESS_AUTO_CLEAR_MS` reste la **constante de défaut** (= `AUTO_CLOSE_DELAY_DEFAULT_SEC * 1000`, ou l'inverse — une seule source de vérité, préférer la constante en secondes du store et dériver le ms). Le `setTimeout` de `useFormFeedback` L16 (auto-clear du message de succès inline, hors modale) **reste à 3 s fixe** — il ne concerne pas la fermeture des modales et sort du périmètre de la story. Ne pas le rendre configurable ici.

### 4.4 `frontend/src/components/atoms/CBtn.vue` — animation `closing` à durée dynamique

L'animation `.c-btn--closing::before` a `animation: c-btn-fill 3s linear forwards` en dur (L371). La rendre dynamique via une **custom property CSS** posée par le composant :

- Ajouter une prop `closingDurationMs?: number` (défaut `undefined`).
- Sur l'élément bouton, lier `:style="closingDurationMs ? { '--c-btn-closing-duration': closingDurationMs + 'ms' } : undefined"`.
- CSS : `animation: c-btn-fill var(--c-btn-closing-duration, 3s) linear forwards;` (le fallback 3 s préserve le comportement si la prop n'est pas passée).
- `prefers-reduced-motion` (L450) : inchangé (`animation: none`).

**Binding dans les modales** : les modales qui affichent le bouton en `closing` (ex. `CDeleteConfirmModal.vue` L79, `DeleteDocumentModal.vue` L77) ajoutent `:closing-duration-ms="ui.autoCloseDelayMs"`. Limiter ce binding aux modales qui utilisent réellement `useAutoClose` + `:closing` (auditer la liste, ~17 fichiers) — ne pas toucher les autres usages de `CBtn`.

Alternative écartée : exposer la durée via un provide/inject global — over-engineering, la prop explicite suit le design system (props déclarées, pas de contexte implicite).

### 4.5 `frontend/src/features/profile/ProfileView.vue` — sélecteur dans l'onglet Apparence

Dans la carte `Préférences` (template `activeTab === 'appearance'`), **sous** le radiogroup thème et son hint :

1. Un libellé + un sélecteur de délai. Composant : réutiliser `CFormSelect` si son API couvre un select natif simple (label + options 0–10 s + v-model) — vérifier sa signature ; sinon un `<select>` natif stylé Tailwind dans la carte (cohérent avec le style des atomes existants, pas de nouveau composant design system pour un champ unique — DESIGN.md §3.1, ne pas cataloguer un atome sans usage répété).
2. Options : `0` à `10`, libellés i18n. La valeur `0` porte un libellé explicite « Immédiate (0 s) » (AC-PP-008-C compréhensible).
3. Binding : `v-model` sur un `computed` get/set qui lit/écrit `ui.autoCloseDelaySec` via `ui.setAutoCloseDelay` — effet immédiat (AC-PP-008-E), persistance gérée par le store.
4. La valeur par défaut 3 s est visible sans action (le sélecteur affiche la valeur courante, donc « 3 s » avant toute personnalisation — AC-PP-008-B).

### 4.6 i18n (`frontend/src/i18n/locales/fr.json`, seule locale du repo)

Nouvelles clés sous `settings.preferences` :

```json
"autoCloseDelay": {
  "label": "Fermeture automatique des modales",
  "hint": "Délai après une validation réussie avant la fermeture automatique de la modale.",
  "immediate": "Immédiate (0 s)",
  "seconds": "{count} s"
}
```

Le libellé des options 1–10 utilise `seconds` avec interpolation (`{count}`) ; l'option 0 utilise `immediate`. Lancer `npm --prefix frontend run i18n:extract` pour vérifier l'absence de clés orphelines/manquantes.

## 5. Cas limites

| Cas | Règle |
|---|---|
| Valeur `0` (immédiat) | `schedule(fn)` exécute `fn` sans timer ; pas de fenêtre de succès visible, pas d'animation `closing` ; le message de succès inline (`useFormFeedback`) reste soumis à son propre clear 3 s mais n'est pas perceptible car la modale ferme |
| Valeur hors bornes / non entière (storage corrompu ou saisie manuelle) | `readAutoCloseDelay` retombe sur 3 s ; `setAutoCloseDelay` ignore la valeur (pas d'état invalide) |
| Clé `localStorage` à la valeur par défaut (3) | Supprimée à l'écriture (même convention que `laruche-theme` avec `auto`) — « défaut » = absence de clé |
| Délai changé pendant qu'une modale de succès est ouverte | Le timer en cours n'est pas recalculé ; le nouveau délai s'applique au prochain succès (la modale bloque la navigation, le cas est théorique) |
| `prefers-reduced-motion` | L'animation `closing` reste désactivée (L450) ; le délai du timer reste effectif |
| SSR / `window` indisponible | `readAutoCloseDelay` / `persistAutoCloseDelay` retournent le défaut / no-op (même garde que le thème) |
| Reset global des préférences (EP-019-S07) | Le délai revient à 3 s via `setAutoCloseDelay(AUTO_CLOSE_DELAY_DEFAULT_SEC)` — à câbler dans `resetAppearance` (ou son renommage) côté S07 |
| Modales sans succès (création en cours, erreurs) | Aucun timer, aucun changement — `schedule` n'est appelé qu'après succès |

## 6. Critères d'acceptation

1. L'onglet `Apparence` affiche un sélecteur de délai 0–10 s, valeur courante visible (3 s par défaut).
2. Changer le délai persiste après rechargement de la page (localStorage).
3. À `0` : après une validation réussie dans n'importe quelle modale auto-close, la modale se ferme immédiatement (pas de fenêtre de succès).
4. À `5` (par ex.) : la modale se ferme 5 s après le succès, et l'animation de remplissage du bouton `closing` dure 5 s (cohérence visuelle timer/animation).
5. Le délai s'applique uniformément à toutes les modales concernées (spot-check sur 2–3 écrans de domaines différents : documents, budget, assemblées).
6. Aucune régression sur les modales qui n'utilisent pas `useAutoClose`.
7. Le reset global des préférences (quand S07 sera livré) remet le délai à 3 s — testé via le point d'extension prévu au §4.1.

## 7. Plan de tests

Pas de backend. Pas de vitest configuré dans le repo (vérifier au moment de l'implémentation ; sinon vérification manuelle, cf. pattern S07 §7). Frontend :

- **Manuelle** : (1) ouvrir Profil > Apparence, vérifier « 3 s » affiché ; (2) passer à `0`, valider une suppression (ex. document) → modale fermée immédiatement, pas de « Fermeture… » ; (3) passer à `5`, valider une suppression → modale ouverte 5 s, animation bouton cohérente, fermeture à 5 s ; (4) recharger → la valeur persiste ; (5) modifier `localStorage` à `"abc"` ou `"42"` → rechargement → retour à 3 s ; (6) `prefers-reduced-motion` activé → pas d'animation, fermeture au délai choisi ; (7) scope : une modale sans auto-close (ex. création en erreur) n'est pas affectée.
- `npm --prefix frontend run build`, `npm --prefix frontend run typecheck`, `npm --prefix frontend run lint`, `npm --prefix frontend run i18n:extract` passent.

## 8. Ordre d'implémentation

1. `ui.ts` : constantes + `readAutoCloseDelay` / `persistAutoCloseDelay` + `autoCloseDelaySec` / `autoCloseDelayMs` / `setAutoCloseDelay` (+ exports).
2. `useAutoClose.ts` : lecture du store, `delay <= 0` → exécution immédiate.
3. `useFormFeedback.ts` : dériver `SUCCESS_AUTO_CLEAR_MS` de la constante du store (source de vérité unique) — ou la déplacer si le lint préfère ; ne pas toucher au timer inline L16.
4. `CBtn.vue` : prop `closingDurationMs` + custom property `--c-btn-closing-duration` + fallback 3 s dans le CSS.
5. Modales avec `:closing` (audit ~17 fichiers) : binder `:closing-duration-ms="ui.autoCloseDelayMs"`.
6. `ProfileView.vue` : sélecteur 0–10 s dans la carte Préférences (computed get/set sur le store).
7. i18n : nouvelles clés `settings.preferences.autoCloseDelay.*`.
8. Vérifications §7, puis `docs/EPICS.md` (statut EP-019-S09) à la livraison.
