# Spécification technique — EP-011-S04

**Story :** Commentaires de suivi sur un signalement : le gestionnaire/conseil ajoute des commentaires horodatés (avancement, décision), visibles du déclarant.

**Épic :** EP-011 — Signalements
**Réf. PRD :** US-SIG-006
**État :** IMPLÉMENTÉ (backend + frontend).


---

## 1. Quoi (résumé fonctionnel)

- Toute personne habilitée à gérer les signalements (manager, board_member) peut poster un **commentaire de suivi** sur un signalement.
- Les commentaires sont **horodatés, attribués (nom de l'auteur) et append-only** : ni édition ni suppression (piste de suivi fiable).
- Le **déclarant** voit les commentaires de son signalement (lecture seule).
- Pas de notification au déclarant lors d'un nouveau commentaire : **différé** (aucune infra de livraison dans le repo) — la visibilité est in-app uniquement.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement | Constat |
|---|---|---|
| Table `report` | `backend/migrations/0001_init.sql` L654-665 | Pas de table de commentaires |
| Handlers reports (list/get/create/update_status/convert_to_task/update/delete/documents) | `backend/src/api/reports.rs` L18-174 | Aucun handler commentaire |
| Routes reports | `backend/src/api/mod.rs` L309-326 | Aucune route commentaire |
| Repo DB (row avec `author_first_name`/`author_last_name` via JOIN `user_account`) | `backend/src/db/report.rs` L7-61 | Modèle de JOIN à réutiliser |
| Client frontend `reportsApi` | `frontend/src/api/index.ts` L426-447 | À étendre |
| Vue signalements | `frontend/src/features/reports/` | Fiche/détail à enrichir d'une section commentaires |

## 3. Modèle de données

**Mettre à jour `docs/ARCHITECTURE.md` avant la migration** (table simple, pas d'ADR).

### Migration `backend/migrations/NNNN_report_comment.sql`

```sql
-- EP-011-S04: follow-up comments on reports (US-SIG-006).
-- Append-only: no updated_at, no soft-delete — comments form a reliable
-- follow-up trail visible to the reporter.
CREATE TABLE report_comment (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id  UUID NOT NULL REFERENCES report(id) ON DELETE CASCADE,
    author_id  UUID NOT NULL REFERENCES user_account(id),
    body       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX report_comment_report_idx ON report_comment (report_id, created_at);
```

Puis `cargo sqlx migrate run` + `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend

### 4.1 `backend/src/db/report.rs` (ou nouveau `db/report_comment.rs` si `report.rs` devient trop long — retenu : rester dans `report.rs`, cohérent avec `report_document`)

```rust
/// Comment row enriched with author name (JOIN user_account).
#[derive(Serialize, sqlx::FromRow)]
pub struct ReportCommentRow {
    pub id: Uuid,
    pub report_id: Uuid,
    pub author_id: Uuid,
    pub author_first_name: String,
    pub author_last_name: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

/// Inserts a comment after verifying the report belongs to the building.
/// Returns None if the report doesn't exist (→ 404 upstream).
pub async fn add_comment(
    pool: &PgPool,
    report_id: Uuid,
    building_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> AppResult<Option<ReportCommentRow>>

/// Lists comments chronologically (oldest first).
pub async fn list_comments(
    pool: &PgPool,
    report_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<ReportCommentRow>>
```

`add_comment` : vérifier `SELECT 1 FROM report WHERE id=$1 AND building_id=$2` avant l'INSERT (scoping immeuble). `list_comments` : `JOIN report r ON r.id = rc.report_id AND r.building_id = $2`, `ORDER BY rc.created_at`.

### 4.2 `backend/src/api/reports.rs` — handlers

```rust
#[derive(Deserialize)]
pub struct AddCommentInput { pub body: String }

/// POST /reports/{id}/comments — manager | board_member.
pub async fn add_comment(...) -> AppResult<(StatusCode, Json<serde_json::Value>)>
// body.trim() vide → AppError::validation_code("errors.reports.comment_body_required")
// → 201 + ReportCommentRow

/// GET /reports/{id}/comments — manager | board_member | déclarant.
pub async fn list_comments(...) -> AppResult<Json<serde_json::Value>>
```

Règle d'accès en lecture : réutiliser la règle d'accès du `GET /reports/{id}` existant (le déclarant voit son signalement) — concrètement, charger le report via `db::report::get` puis vérifier `manager/board OR report.author_id == auth.user_id`, sinon 403. Zéro logique métier au-delà de ce guard d'autorisation.

### 4.3 `backend/src/api/mod.rs` — routes

Dans le bloc `// ── Reports ──` (après L326) :

```rust
.route(
    "/reports/{id}/comments",
    get(reports::list_comments).post(reports::add_comment),
)
```

## 5. Frontend

### 5.1 `frontend/src/api/index.ts` — `reportsApi`

```ts
listComments: (id: string) =>
  api.get<ReportComment[]>(`/reports/${id}/comments`),
addComment: (id: string, body: string) =>
  api.post<ReportComment>(`/reports/${id}/comments`, { body }),
```

Type `ReportComment` dans `frontend/src/types/domain.ts` (`id`, `report_id`, `author_id`, `author_first_name`, `author_last_name`, `body`, `created_at`).

### 5.2 Vue détail signalement (`frontend/src/features/reports/`)

Section **« Suivi »** sur la fiche signalement :

- Liste chronologique : nom de l'auteur, date relative (`formatRelative`), corps du commentaire. État vide : « Aucun commentaire de suivi ».
- Zone de saisie + bouton « Commenter » visible uniquement si `authStore.hasAnyRole(['manager', 'board_member'])`. Soumission → `addComment` → append à la liste + reset du champ. Erreur API affichée en message de formulaire.
- Le déclarant (non gestionnaire) voit la liste en lecture seule, sans zone de saisie.
- Logique dans `<script setup>` (règle frontend : rien dans le template). Pas de nouveau composant catalogue si la section reste inline dans la vue existante ; si un composant dédié `ReportCommentList.vue` est extrait, l'ajouter à `docs/DESIGN.md §2` avant usage.

## 6. Cas limites

> **Choix produit acté (visibilité)** : la lecture des commentaires est restreinte au **déclarant du signalement** et aux gestionnaires (manager/board_member) — un résident tiers reçoit 403. Ce choix est plus protecteur que la matrice CRUD du PRD (qui dit `R` pour tous les résidents) ; il est retenu pour des raisons RGPD (les commentaires peuvent contenir des détails sur un lot/habitation) et doit être explicitement reconduit si la matrice évolue.

| Cas | Règle |
|---|---|
| Corps vide / espaces | 400 `errors.reports.comment_body_required` |
| Report inexistant ou autre immeuble | 404 |
| Déclarant tente de poster | 403 (écriture réservée manager/board) — s'il faut ouvrir l'écriture au déclarant plus tard, c'est une évolution PRD, pas cette story |
| Résident tiers (ni déclarant ni gestionnaire) | 403 en lecture |
| Édition/suppression d'un commentaire | Non exposées (append-only) — aucune route PUT/DELETE |
| Suppression du signalement | `ON DELETE CASCADE` emporte les commentaires |
| Signalement anonyme (EP-011-S05) | Sans impact : les commentaires portent leur propre `author_id` (gestionnaire), jamais celui du déclarant |

## 7. Critères d'acceptation

1. Un gestionnaire poste un commentaire → il apparaît horodaté avec son nom.
2. Le déclarant voit les commentaires de son signalement, sans pouvoir en poster.
3. Un tiers non habilité ne voit pas les commentaires.
4. Les commentaires sont immuables (aucune API d'édition/suppression).

## 8. Plan de tests

E2e (`backend/tests/api_e2e_test.rs`) :

1. Manager : `POST /reports/{id}/comments` → 201, `author_first_name` renseigné ; `GET` → 1 élément, ordre chronologique après un 2ᵉ commentaire.
2. Corps vide → 400 `errors.reports.comment_body_required`.
3. Déclarant : `GET` → 200 avec les commentaires ; `POST` → 403.
4. Résident tiers : `GET` → 403.
5. Report d'un autre immeuble → 404 (POST et GET).
6. `DELETE /reports/{id}` → les commentaires disparaissent (cascade, vérif SQL ou 404 sur GET).

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `report_comment`) → migration `NNNN` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. `db/report.rs` : `ReportCommentRow`, `add_comment`, `list_comments` → `cargo build`.
3. `api/reports.rs` : handlers + guard lecture déclarant ; route dans `api/mod.rs` → `cargo clippy -- -D warnings`.
4. Tests e2e §8, boucler jusqu'au vert.
5. Frontend : type `ReportComment`, `reportsApi`, section « Suivi » sur la fiche signalement + i18n.
6. `npm --prefix frontend run build && npm --prefix frontend run lint` ; `docs/EPICS.md`.
