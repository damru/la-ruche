# Spécification technique — EP-019-S03

**Story :** Préférences de notifications par événement (16 événements, 3 obligatoires) — **persistance serveur**.

**Épic :** EP-019 — Profil & préférences
**Réf. PRD :** US-PP-004
**Statut : IMPLÉMENTÉ (backend + frontend)** — persistance serveur livrée : table `user_notification_preference`, catalogue canonique `domain::notification_catalog`, `GET/PUT /me/notification-preferences`, hydratation du store `ui.ts` avec localStorage en cache de repli et migration one-shot des anciennes valeurs locales.

> **Rappel de périmètre :** aucune infra de livraison de notifications n'existe dans le repo. Ces préférences sont stockées pour le futur dispatcher ; elles n'ont aujourd'hui aucun effet d'envoi.

---

## 1. Quoi (résumé fonctionnel)

- Les préférences de notifications d'un utilisateur sont stockées **côté serveur**, rattachées à son compte (elles valent pour tous ses immeubles — préférence de compte, pas d'immeuble).
- Le **catalogue devient canonique côté backend** (une seule source de vérité) : 16 événements, dont 3 obligatoires non désactivables (`ag_convocation`, `levy_call_issued`, `account_security`).
- `GET /me/notification-preferences` renvoie le catalogue fusionné avec les valeurs de l'utilisateur ; `PUT` remplace les valeurs des événements optionnels.
- Le store frontend s'hydrate depuis l'API ; localStorage devient un **cache de repli** (offline/erreur réseau) et les anciennes valeurs localStorage sont **migrées** vers le serveur à la première connexion.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement | Constat |
|---|---|---|
| Catalogue `NOTIF_EVENTS` (16 événements, 3 `mandatory`) | `frontend/src/stores/ui.ts` L25-42 | ✅ complet, frontend-only |
| Défauts + lecture localStorage | `ui.ts` L44-67, L128 | ✅ |
| Persistance localStorage (`laruche-notif-prefs`) | `ui.ts` L69-74, L130-142 | ❌ à reléguer en repli |
| Note « Move server-side when notifications ship » | `ui.ts` L12 | Dette assumée, levée par cette story |
| UI de réglage (onglet Notifications) | `frontend/src/features/profile/ProfileView.vue` L290+ | ✅ inchangée |
| Routes `/auth/me` (GET/PATCH) | `backend/src/api/mod.rs` L49 | Modèle « préférence de compte » existant |
| Table de préférences | — | ❌ inexistante |

## 3. Modèle de données

**Mettre à jour `docs/ARCHITECTURE.md` avant la migration** (table simple, pas d'ADR).

### Migration `backend/migrations/NNNN_user_notification_preference.sql`

```sql
-- EP-019-S03: server-side notification preferences (US-PP-004).
-- One row per (user, optional event). Mandatory events are never stored:
-- they are always on, enforced by the domain catalog. Absent row = default.
CREATE TABLE user_notification_preference (
    user_id    UUID NOT NULL REFERENCES user_account(id) ON DELETE CASCADE,
    event_key  TEXT NOT NULL,
    enabled    BOOLEAN NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, event_key)
);
```

Pas de FK sur `event_key` (catalogue en code, pas en table) — la validation des clés est applicative (§4.1). Puis `cargo sqlx migrate run` + `cargo sqlx prepare`.

## 4. Backend

### 4.1 Catalogue canonique — `backend/src/domain/notification_catalog.rs`

```rust
pub(crate) struct NotifEvent {
    pub key: &'static str,
    pub mandatory: bool,
    pub default_enabled: bool,
}

/// Canonical catalog — single source of truth (mirrored by the frontend via
/// GET /me/notification-preferences; ui.ts keeps only display order + i18n).
pub(crate) const NOTIF_EVENTS: &[NotifEvent] = &[ /* 16 entrées, mêmes clés et
    mêmes défauts que frontend/src/stores/ui.ts L25-42 */ ];

pub(crate) fn find(key: &str) -> Option<&'static NotifEvent>;
```

Reprendre **exactement** les 16 clés et défauts de `ui.ts` L25-42 (`ag_convocation`✱, `levy_call_issued`✱, `account_security`✱, `ag_minutes`, `ag_vote_reminder`, `levy_overdue`, `consultation_opened`, `report_published`, `document_published`, `contract_expiring`, `claim_created`, `claim_resolved` (défaut off), `task_assigned`, `task_due`, `maintenance_due`, `weekly_digest` (défaut off)) — ✱ = obligatoire.

### 4.2 `backend/src/db/notification_preference.rs`

```rust
/// Returns the user's stored overrides as (event_key, enabled).
pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<(String, bool)>>;

/// Upserts the given optional-event values in one transaction
/// (INSERT ... ON CONFLICT (user_id, event_key) DO UPDATE).
pub async fn upsert(pool: &PgPool, user_id: Uuid, prefs: &[(String, bool)]) -> AppResult<()>;
```

### 4.3 `backend/src/api/` — handlers (nouveau module `notification_preferences.rs`)

- **`GET /me/notification-preferences`** : fusionne catalogue + overrides →
  `{"events": [{"key", "mandatory", "enabled"}]}` dans l'ordre du catalogue. `enabled` = `true` si mandatory, sinon override ou défaut.
- **`PUT /me/notification-preferences`** : body `{"prefs": {"<event_key>": bool}}`.
  Validation (dans le handler, simple validation de payload) : clé inconnue → 400 `errors.notification_preferences.unknown_event` ; clé obligatoire → 400 `errors.notification_preferences.mandatory_event`. Puis `db::upsert`, réponse = même forme que le GET.
- Routes dans `api/mod.rs` (routeur authentifié, à côté du bloc auth) :

```rust
.route(
    "/me/notification-preferences",
    get(notification_preferences::get).put(notification_preferences::update),
)
```

Aucune notion d'immeuble : préférence de compte (comme `/auth/me`).

## 5. Frontend

### 5.1 `frontend/src/api/index.ts`

```ts
export const notificationPreferencesApi = {
  get: () => api.get<{ events: { key: string; mandatory: boolean; enabled: boolean }[] }>('/me/notification-preferences'),
  update: (prefs: Record<string, boolean>) =>
    api.put<{ events: { key: string; mandatory: boolean; enabled: boolean }[] }>('/me/notification-preferences', { prefs }),
}
```

### 5.2 `frontend/src/stores/ui.ts` — hydratation serveur

- `NOTIF_EVENTS` **reste** (ordre d'affichage + ids i18n + défauts de repli) mais la note L10-12 est remplacée : la source de vérité des **valeurs** est le serveur.
- Nouvelle action `loadNotifPrefs()` (appelée au montage de l'onglet Notifications ou après login) :
  1. `notificationPreferencesApi.get()` → `notifPrefs` ; succès → écrire aussi en localStorage (cache).
  2. **Migration one-shot** : si localStorage contient des valeurs (`laruche-notif-prefs`) différant des valeurs serveur par défaut et qu'aucun override serveur n'existe encore, pousser les valeurs locales via `update` puis marquer la migration faite (clé `laruche-notif-prefs-migrated`).
  3. Échec réseau → repli sur `readNotifPrefs()` (comportement actuel), sans casser l'UI.
- `setNotifPref(id, value)` : mise à jour optimiste locale + `notificationPreferencesApi.update({ [id]: value })` ; en cas d'échec, rollback + message d'erreur.
- `resetNotifPrefs()` : remet les défauts localement **et** pousse les défauts optionnels via `update` (cohérence multi-appareils ; utilisé par EP-019-S07).

### 5.3 `ProfileView.vue`

Inchangé visuellement ; appeler `ui.loadNotifPrefs()` à l'entrée de l'onglet Notifications (état de chargement discret toléré).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Clé inconnue dans le PUT | 400 `errors.notification_preferences.unknown_event` (rien n'est écrit) |
| Tentative de désactiver un événement obligatoire | 400 `errors.notification_preferences.mandatory_event` ; l'UI n'expose déjà pas le toggle |
| Utilisateur sans aucune ligne stockée | GET renvoie les défauts du catalogue |
| Offline / API en erreur | Repli localStorage, écritures locales conservées ; re-synchronisation non garantie (assumé, simple) |
| Divergence catalogue front/back | Le front affiche l'intersection : événements du catalogue front dont la valeur vient du serveur ; clés serveur inconnues du front ignorées |
| Suppression du compte | `ON DELETE CASCADE` |

## 7. Critères d'acceptation

1. Les toggles survivent à un changement de navigateur/appareil (valeurs servies par l'API).
2. Les 3 événements obligatoires sont toujours `enabled: true` côté API et non modifiables.
3. Les anciennes préférences localStorage d'un utilisateur existant sont reprises côté serveur à la première visite.
4. En cas d'API indisponible, l'onglet Notifications reste fonctionnel (repli localStorage).
5. Nouvelle table documentée dans `docs/ARCHITECTURE.md`.

## 8. Plan de tests

E2e (`backend/tests/api_e2e_test.rs`) :

1. `GET` sans override → 16 événements, défauts du catalogue (`weekly_digest` et `claim_resolved` off, obligatoires on).
2. `PUT {"prefs": {"weekly_digest": true}}` → 200 ; `GET` reflète l'override ; upsert idempotent (2ᵉ PUT identique).
3. `PUT` clé inconnue → 400 `unknown_event` ; `PUT {"ag_convocation": false}` → 400 `mandatory_event`.
4. Deux utilisateurs : overrides isolés.

Unitaire : `notification_catalog::find` + invariant « 16 entrées, 3 obligatoires » (test qui casse si le catalogue dérive).

Frontend (si vitest configuré) : hydratation, rollback sur échec `update`, migration localStorage one-shot.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` → migration `NNNN` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. `domain/notification_catalog.rs` + test invariant → `cargo test --lib`.
3. `db/notification_preference.rs` → `api/notification_preferences.rs` → routes → `cargo clippy -- -D warnings`.
4. Tests e2e §8, boucler jusqu'au vert.
5. Frontend : `notificationPreferencesApi`, refonte `ui.ts` (hydratation, optimiste, migration, repli), appel dans `ProfileView.vue`.
6. `npm --prefix frontend run build && npm --prefix frontend run lint` ; `docs/EPICS.md` / `docs/PROGRESS.md`.
