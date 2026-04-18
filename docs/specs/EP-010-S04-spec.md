# Spécification technique — EP-010-S04

**Story :** Cycle de vie brouillon des consultations : créer en `draft`, modifier librement tant que le brouillon n'est pas ouvert, puis ouvrir explicitement (`draft → open`).

**Épic :** EP-010 — Consultations informelles
**Réf. PRD :** US-CS-001b
**État :** PARTIEL — l'infrastructure `draft` existe déjà en base et côté DB/API pour l'ouverture ; il manque la création en `draft`, l'édition du brouillon et toute l'UI.

---

## 1. Quoi (résumé fonctionnel)

Aujourd'hui une consultation est créée directement `open` : impossible de la préparer tranquillement. Cible :

- `POST /consultations` crée la consultation en **`draft`** (le défaut DB existe déjà).
- Un brouillon est **librement modifiable** (titre, description, rôles visibles, date de clôture, choix) et supprimable.
- Une action explicite **« Ouvrir »** fait passer `draft → open` ; à partir de là la consultation devient votable et n'est plus éditable.
- **Notification des participants à l'ouverture : hors périmètre.** Aucune infrastructure de livraison de notifications (email/push) n'existe dans le repo — ce point est explicitement différé (voir EP-019 pour les préférences). L'événement `consultation_opened` du catalogue de préférences reste sans effet tant que la livraison n'existe pas.

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement | Constat |
|---|---|---|
| Enum `poll_status` (`'draft','open','closed'`) | `backend/migrations/0001_init.sql` L91-93 | ✅ `draft` existe déjà |
| Défaut `status = 'draft'` sur `consultation` | `0001_init.sql` L684 | ✅ |
| `db::consultation::create` | `backend/src/db/consultation.rs` L152-192 | ❌ **force `'open'::poll_status`** (L167), contourne le défaut |
| `db::consultation::open` (guard `status='draft'`) | `db/consultation.rs` L326-339 | ✅ existe, 404 si non-draft |
| Handler `update` (ouvre un draft, sans body) | `backend/src/api/consultations.rs` L128-138 | ⚠️ `PUT /consultations/{id}` détourné en action « open », aucune édition de champs |
| Routes consultations | `backend/src/api/mod.rs` L327-340 | `PUT {id}` câblé ; pas de route `/open` dédiée |
| Client frontend `openDraft` | `frontend/src/api/index.ts` L457-459 | ✅ existe mais **zéro site d'appel** |
| `ConsultationsView.vue` filtre | `frontend/src/features/consultations/ConsultationsView.vue` L38, L129-135 | ❌ ne connaît que `open|closed`, aucune UI draft |
| Modale de création | `frontend/src/features/consultations/CreateConsultationModal.vue` | Existe (création seule) |

## 3. Modèle de données

**Aucune migration nécessaire.** L'enum et le défaut `'draft'` existent depuis `0001_init.sql`. Pas de nouvelle table → pas de mise à jour `docs/ARCHITECTURE.md`.

---

## 4. Backend

### 4.1 `db/consultation.rs` — `create` en draft

Retirer le forçage `'open'` (L167) : laisser le défaut DB s'appliquer (ou binder explicitement `'draft'::poll_status` pour l'auto-documentation) :

```sql
INSERT INTO consultation (building_id, created_by, title, description,
    visible_to_roles, status, closed_at)
VALUES ($1, $2, $3, $4, $5::user_role[], 'draft'::poll_status, $6) RETURNING id
```

### 4.2 `db/consultation.rs` — nouvelle fonction `update_draft`

Édition complète d'un brouillon, transactionnelle, guardée `status='draft'` :

```rust
/// Updates a draft consultation (title, description, visible roles,
/// closed_at) and replaces its choices. 404 if not found or not draft.
pub async fn update_draft(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    title: &str,
    description: Option<&str>,
    visible_to_roles: &[String],
    closed_at: Option<DateTime<Utc>>,
    choices: &[String],
) -> AppResult<()>
```

Implémentation : `UPDATE consultation SET ... WHERE id=$1 AND building_id=$2 AND status='draft'` (404 si `rows_affected == 0`), puis `DELETE FROM consultation_choice WHERE consultation_id=$1` et ré-insertion des choix (aucune réponse ne peut exister sur un draft, la suppression est sûre).

### 4.3 `api/consultations.rs`

- **`update`** : lui donner sa vraie sémantique — `PUT /consultations/{id}` avec body `CreateConsultationInput` (mêmes validations que `create` : titre non vide, ≥ 2 choix, rôles valides), appelle `update_draft`. Autorisation : manager/board **ou** créateur du brouillon (à vérifier via `created_by` retourné par `get_with_choices`).
- **Nouveau handler `open`** : reprend le corps actuel de `update` (require manager/board, `db::consultation::open`), `204 No Content`.
- La logique de validation partagée `create`/`update` est extraite en fonction privée `validate_input(&CreateConsultationInput)` (zéro logique dupliquée, mais elle reste dans `api/` car c'est de la validation de payload, pas du métier).

### 4.4 `api/mod.rs` — routes

Dans le bloc `// ── Consultations ──` (après L338) :

```rust
.route("/consultations/{id}/open", post(consultations::open))
```

`PUT /consultations/{id}` conserve sa route mais change de contrat (body requis). Le seul consommateur actuel (`openDraft`, jamais appelé) est migré en §5.1 — pas de rupture réelle.

### 4.5 Garde-fou vote sur draft

Vérifier que `db::consultation::respond` refuse une réponse sur une consultation `draft` (guard `status='open'` dans l'UPDATE/INSERT ou check préalable). Si absent, l'ajouter : `errors.consultations.not_open`.

---

## 5. Frontend

### 5.1 `frontend/src/api/index.ts`

```ts
update: (id: string, data: { title: string; description?: string; visible_to_roles: string[]; choices: string[]; closed_at?: string }) =>
  api.put(`/consultations/${id}`, data),
/** Opens a draft consultation (transitions draft → open). */
openDraft: (id: string) =>
  api.post(`/consultations/${id}/open`),
```

### 5.2 `ConsultationsView.vue`

- `statusFilter` : `'draft' | 'open' | 'closed'` ; onglet **« Brouillons »** avec compteur `draftCount`, affiché uniquement si l'utilisateur peut gérer (`canManage`) ou possède au moins un brouillon (`created_by === auth.user.id` — exposer `created_by` dans le `SELECT` de `list` si absent).
- Carte draft : badge `draft`, pas de zone de vote, actions **« Modifier »** (ouvre la modale §5.3 pré-remplie) et **« Ouvrir »** (confirmation `useConfirm` : « La consultation deviendra visible et votable » → `consultationsApi.openDraft(id)` → reload).
- Suppression d'un draft : le backend `delete` actuel n'accepte que `closed` — étendre le guard DB à `status IN ('closed','draft')` (un brouillon jamais publié doit être supprimable) et afficher l'action « Supprimer » sur les cartes draft.

### 5.3 `CreateConsultationModal.vue` — mode édition

Prop optionnelle `consultation?: Consultation` : si fournie, pré-remplir les champs et soumettre via `consultationsApi.update` au lieu de `create`. Titre de modale et bouton adaptés (i18n `consultations.edit.*`). Pas de nouveau composant → pas d'entrée `docs/DESIGN.md §2`.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| `PUT` sur une consultation `open`/`closed` | 404 (guard `status='draft'` dans `update_draft`) |
| `POST /open` sur non-draft | 404 (comportement existant de `db::open`) |
| `POST /respond` sur un draft | 400 `errors.consultations.not_open` (§4.5) |
| Draft visible aux résidents ? | `list` retourne les drafts ; le frontend ne les montre que dans l'onglet Brouillons (gestionnaires/créateur). Si le PRD exige un masquage strict côté API, filtrer dans `db::list` (`status <> 'draft' OR created_by = $user`) — retenu ici : **filtrage API** (moindre surprise). |
| Édition remplace les choix | Sûre : aucune réponse ne peut référencer un choix d'un draft |
| Notification à l'ouverture | Différée — aucune infra de livraison dans le repo |

## 7. Critères d'acceptation

1. `POST /consultations` → 201, la consultation est en `draft`, invisible dans l'onglet « Ouvertes ».
2. Le créateur/gestionnaire peut modifier titre, description, rôles, date de clôture et choix d'un brouillon.
3. « Ouvrir » (confirmé) passe le statut à `open` ; la consultation devient votable ; elle n'est plus éditable.
4. Un résident sans rôle gestionnaire ne voit pas les brouillons des autres.
5. Un brouillon peut être supprimé.

## 8. Plan de tests

E2e (`backend/tests/api_e2e_test.rs`, harness `common::isolated_test_server()`) :

1. `POST /consultations` → `status == "draft"` ; absent du filtre open côté `list` (statut vérifié dans la réponse).
2. `PUT /consultations/{id}` (draft) modifie titre + choix → `GET` reflète les changements.
3. `PUT` sur consultation ouverte → 404.
4. `POST /{id}/open` → 204 ; `GET` → `status == "open"` ; second `POST /open` → 404.
5. `POST /{id}/respond` sur draft → 400 `errors.consultations.not_open`.
6. `DELETE` d'un draft → 204 ; `DELETE` d'une consultation `open` → 404.
7. `list` en tant que résident simple : drafts d'autrui absents.

Unitaire : `validate_input` (titre vide, < 2 choix, rôle invalide).

## 9. Ordre d'implémentation

1. `db/consultation.rs` : `create` → draft, `update_draft`, guard `respond`, guard `delete` étendu à draft → `cargo build`.
2. `api/consultations.rs` : handler `open`, refonte `update` avec body, filtrage drafts dans `list` → route `/open` dans `api/mod.rs` → `cargo clippy -- -D warnings`.
3. `cargo sqlx prepare` si requêtes macro modifiées ; tests e2e §8, boucler jusqu'au vert.
4. Frontend : `api/index.ts` (`update`, `openDraft` → POST), `CreateConsultationModal` mode édition, `ConsultationsView` onglet Brouillons + actions.
5. `npm --prefix frontend run build && npm --prefix frontend run lint`.
6. `docs/EPICS.md` / `docs/PROGRESS.md` (entrée datée).
