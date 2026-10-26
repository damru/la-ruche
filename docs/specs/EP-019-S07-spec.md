# Spécification technique — EP-019-S07

**Story :** Réinitialiser ses préférences : un **bouton global unique sur la ligne des onglets de filtres** couvrant Notifications **et** Apparence, protégé par une modale de confirmation.

**Épic :** EP-019 — Profil & préférences
**Réf. PRD :** US-PP-005
**Statut : IMPLÉMENTÉ (frontend)** — bouton global unique « Réinitialiser mes préférences » (icône `CIconRefresh`) aligné à droite sur la ligne des onglets de filtres de la page Profil (visible quel que soit l'onglet), modale de confirmation listant les deux domaines, reset du thème vers `auto` et (EP-019-S09) du délai de fermeture des modales vers 3 s. L'ancien bouton de l'en-tête Notifications est supprimé.

Story 100 % frontend — aucune migration, aucun changement backend. (Si EP-019-S03 est livrée avant, le reset des notifications synchronise le serveur via `resetNotifPrefs` → `DELETE /me/notification-preferences` — déjà couvert par la spec S03 §4.3/§5.2 ; cette story n'ajoute rien côté API.)

---

## 1. Quoi (résumé fonctionnel)

- Un **seul** bouton « Réinitialiser mes préférences », sur la **ligne des onglets de filtres** de la page Profil (accompagné d'une icône `CIconRefresh`, toujours visible quel que soit l'onglet actif ; voir §4.2).
- Il réinitialise **d'un coup** :
  - **Notifications** : retour aux défauts du catalogue (obligatoires on, `weekly_digest`/`claim_resolved` off, le reste on) ;
  - **Apparence** : thème → `auto` (suivre le système).
- Une **modale de confirmation** (`useConfirm`, existante) précède l'action ; elle liste explicitement les deux domaines réinitialisés.
- Le bouton de reset actuel de l'en-tête Notifications est **supprimé** (un seul point d'entrée).

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement | Constat |
|---|---|---|
| Reset notifications (confirmation `useConfirm`) | `frontend/src/features/profile/ProfileView.vue` L114-123 | ✅ modale OK, ❌ notifications uniquement |
| Bouton de reset dans l'en-tête de l'onglet Notifications | `ProfileView.vue` L302-307 | ❌ mauvais placement, à supprimer |
| `ui.resetNotifPrefs()` | `frontend/src/stores/ui.ts` L139-142 | ✅ existe |
| Thème : `setThemePreference` / persistance (`laruche-theme`, `auto` = clé supprimée) | `ui.ts` L85-95, L122-125 | ❌ aucun reset apparence |
| i18n reset actuel | `settings.notifications.reset*` | À remplacer par des clés globales `settings.preferences.reset*` |

## 3. Modèle de données / Backend

Aucun.

## 4. Frontend

### 4.1 `frontend/src/stores/ui.ts` — reset global

```ts
/** Restores appearance defaults: theme preference back to 'auto'. */
function resetAppearance() {
  setThemePreference('auto')
}

/** Global reset (US-PP-005): notifications + appearance in one action. */
function resetAllPrefs() {
  resetNotifPrefs()
  resetAppearance()
}
```

Exporter `resetAppearance` et `resetAllPrefs` dans le `return` du store. `resetNotifPrefs` reste inchangée (et, si EP-019-S03 est en place, elle synchronise déjà le serveur). Si d'autres préférences d'apparence apparaissent plus tard, elles s'ajoutent dans `resetAppearance` — point d'extension unique, pas d'abstraction supplémentaire.

### 4.2 `ProfileView.vue`

1. **Supprimer** le `CBtn` de reset de l'en-tête Notifications (L302-307) et alléger le `notif-header` en conséquence (si le header ne contenait que titre + bouton, garder titre/sous-titre seuls).
2. Renommer/adapter le handler existant (L114-123) :

```ts
// ── Global preferences reset (US-PP-005) ──────────────────────────
async function resetAllPrefs() {
  const ok = await confirm({
    title: t('settings.preferences.resetConfirmTitle'),
    message: t('settings.preferences.resetConfirmMessage'),
    variant: 'primary',
  })
  if (ok) {
    ui.resetAllPrefs()
  }
}
```

3. **Placement** : sur la ligne des onglets de filtres (`CTabs variant="filter"` : `Mon compte` · `Préférences`), le bouton est aligné à droite dans un conteneur `flex items-center justify-between gap-3` — toujours visible quel que soit l'onglet actif :

```html
<!-- ── Tabs + global reset action ─────────────────────────────────────────── -->
<div class="flex items-center justify-between gap-3">
  <CTabs
    v-model="activeTab"
    variant="filter"
    :tabs="[
      { value: 'account', label: t('nav.myAccount') },
      { value: 'preferences', label: t('settings.preferences.title') },
    ]"
    class="mb-0!"
  />
  <CBtn
    variant="ghost"
    size="sm"
    :icon="CIconRefresh"
    :title="t('settings.preferences.reset')"
    @click="resetAllPrefs"
  />
</div>
```

Contraintes repo : utilitaires Tailwind uniquement (pas de valeurs brutes), composants catalogue `CTabs`/`CBtn` existants — aucun nouveau composant, donc pas d'entrée `docs/DESIGN.md §2`.

### 4.3 i18n

Nouvelles clés (fr + autres locales du repo) :

- `settings.preferences.reset` : « Réinitialiser mes préférences »
- `settings.preferences.resetConfirmTitle` : « Réinitialiser toutes les préférences ? »
- `settings.preferences.resetConfirmMessage` : « Les préférences de notifications et d'apparence (thème) seront remises à leurs valeurs par défaut. »

Supprimer les clés devenues orphelines `settings.notifications.reset`, `resetConfirmTitle`, `resetConfirmMessage` (nettoyage de ses propres orphelins, règle « Surgical Changes »).

## 5. Cas limites

| Cas | Règle |
|---|---|
| Annulation dans la modale | Aucun changement (ni notifications, ni thème) — action atomique côté UX |
| Thème déjà `auto`, préférences déjà par défaut | Reset idempotent, pas d'erreur |
| Reset du thème | `setThemePreference('auto')` supprime la clé `laruche-theme` (comportement existant L90-91) ; le `watch` réapplique immédiatement `data-theme` selon le système |
| Événements de notification obligatoires | Non affectés (toujours on, `defaultNotifPrefs` les force à `true`) |
| Visibilité du bouton | Présent quel que soit l'onglet actif (Compte, Préférences…) |
| EP-019-S03 livrée | `resetNotifPrefs` appelle `DELETE /me/notification-preferences` (suppression des overrides serveur) ; en cas d'échec réseau, comportement de repli défini par S03 — rien à ajouter ici |

## 6. Critères d'acceptation

1. Un unique bouton de réinitialisation existe, sur la ligne des onglets de filtres de la page Profil (avec icône), visible depuis tous les onglets ; l'ancien bouton de l'en-tête Notifications a disparu.
2. Après confirmation : tous les toggles de notifications reviennent à leurs défauts **et** le thème repasse en « auto ».
3. L'annulation de la modale ne change rien.
4. Le libellé de la modale mentionne explicitement les deux domaines (notifications + apparence).

## 7. Plan de tests

Pas de backend. Frontend :

- **Si vitest est configuré** : test du store `ui` — `resetAllPrefs()` remet `notifPrefs` aux défauts et `themePreference` à `'auto'` ; localStorage `laruche-theme` supprimée, `laruche-notif-prefs` réécrite aux défauts.
- **Sinon, vérification manuelle** : (1) modifier 2 toggles + passer le thème en `dark` ; (2) cliquer le bouton global → confirmer → toggles par défaut + thème système ; (3) recharger la page → l'état réinitialisé persiste ; (4) annuler la modale → aucun changement ; (5) vérifier l'absence du bouton dans l'en-tête Notifications.
- `npm --prefix frontend run build` et `npm --prefix frontend run lint` passent (détecte les clés i18n orphelines si le lint le couvre).

## 8. Ordre d'implémentation

1. `ui.ts` : `resetAppearance` + `resetAllPrefs` (+ exports).
2. `ProfileView.vue` : suppression du bouton d'en-tête, handler global, bloc bas de page.
3. i18n : nouvelles clés `settings.preferences.reset*`, suppression des clés orphelines.
4. `npm --prefix frontend run build && npm --prefix frontend run lint` ; vérification manuelle §7.
5. `docs/EPICS.md` (statut EP-019-S07) / `docs/PROGRESS.md` (entrée datée).
