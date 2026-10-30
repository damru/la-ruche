# Spécification technique — EP-007-S09

**Story :** Lister, filtrer et rechercher les tâches, avec pagination.

**Épic :** EP-007 — Travaux & prestataires
**Réf. PRD :** US-TP-003
**État :** IMPLÉMENTÉ (backend + frontend) — filtres serveur `status`/`priority`/`assigned_to`/`contractor_id`/`q`, pagination `limit`/`offset` avec `total`, recherche ILIKE échappée, UI filtres + recherche debouncée + « Charger plus ».

---

## 1. État actuel du code (écart)

| Élément | Emplacement | Constat |
|---|---|---|
| Handler `list` | `backend/src/api/tasks.rs:23-30` | Seul paramètre : `building_id` implicite ; aucun query param |
| DB `list` | `backend/src/db/task.rs:152-182` | Pas de `WHERE` additionnel, pas de `LIMIT`/`OFFSET` |
| Frontend filtres | `TasksView.vue:46,90-102,363-368` | Onglets statut **côté client** uniquement |
| Recherche | — | Aucun champ de recherche |

Écart : query params backend (`status`, `priority`, `assignee_id`, `contractor_id`, `q`, `limit`/`offset`) + rangée de filtres et champ de recherche côté frontend.

## 2. Modèle de données

Aucune migration. Optionnel (perf, si volumétrie le justifie plus tard) : index `task (building_id, status)` — hors périmètre.

## 3. Backend

### 3.1 API — `backend/src/api/tasks.rs`

```rust
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<String>
    pub priority: Option<String>
    pub assignee_id: Option<Uuid>
    pub contractor_id: Option<Uuid>
    pub q: Option<String>,          // search in title + description
    pub limit: Option<i64>,         // default 50, cap 200
    pub offset: Option<i64>,        // default 0
}
```

Handler `list` (`:23-30`) : extraire `Query(params)`, valider `status`/`priority` contre les valeurs d'enum connues (`AppError::validation_code("errors.tasks.invalid_filter")` sinon), clamp `limit` à `[1, 200]`, déléguer à la couche db. Zéro logique de construction SQL dans le handler.

### 3.2 DB — `backend/src/db/task.rs`

Étendre `list` (`:152-182`) avec des filtres optionnels — pattern SQL statique à conditions nullables (compatible `sqlx prepare`, pas de query builder dynamique) :

```sql
WHERE t.building_id = $1
  AND ($2::task_status IS NULL OR t.status = $2)
  AND ($3::task_priority IS NULL OR t.priority = $3)
  AND ($4::uuid IS NULL OR t.assignee_id = $4)
  AND ($5::uuid IS NULL OR t.contractor_id = $5)
  AND ($6::text IS NULL OR t.title ILIKE '%' || $6 || '%' OR t.description ILIKE '%' || $6 || '%')
ORDER BY t.created_at DESC
LIMIT $7 OFFSET $8
```

Retourner aussi le total filtré pour la pagination : soit un `COUNT(*) OVER ()` dans le SELECT, soit une réponse enveloppe `{ items, total }`. **Recommandé : `COUNT(*) OVER () AS total_count`** (une seule requête). Adapter le type de retour du handler en `{ items: Task[], total: number }`.

Note : adapter les noms de colonnes (`assignee_id`/`contractor_id`) aux colonnes réelles de `task` lors de l'implémentation ; si l'un des deux n'existe pas dans le schéma, retirer le filtre correspondant plutôt que de l'inventer.

## 4. Frontend

- `frontend/src/api/index.ts` (`tasksApi.list`) : accepter un objet de filtres optionnel sérialisé en query string ; nouveau type de réponse `{ items, total }`.
- `TasksView.vue` :
  - conserver les onglets statut existants (`CTabs`, `:363-368`) mais les brancher sur le param `status` de l'API
  - ajouter une rangée de filtres : select priorité, select assigné, select prestataire (composants catalogue `C*`)
  - champ de recherche (debounce ~300 ms) branché sur `q`
  - pagination simple « Charger plus » ou pages numérotées (`limit=50`)
  - **fallback client-side conservé simple** : tant que `total <= limit`, le filtrage local actuel peut rester en secours, mais la source de vérité devient l'API.
- Pas de nouveau composant catalogue attendu ; si un composant de pagination est créé, l'ajouter à `docs/DESIGN.md §2` avant usage.

## 5. Cas limites

| Cas | Règle |
|---|---|
| `status`/`priority` hors enum | `400 errors.tasks.invalid_filter` |
| `q` vide ou espaces | Traité comme absent |
| `limit` > 200 ou ≤ 0 | Clamp silencieux à `[1, 200]` |
| `offset` au-delà du total | `items: []`, `total` correct |
| `q` contenant `%` ou `_` | Échapper les métacaractères `ILIKE` (ou accepter le comportement joker — décision simple : échapper) |
| Combinaison de tous les filtres | ET logique |

## 6. Critères d'acceptation

1. `GET /tasks?status=open&q=chaudière&limit=20&offset=0` retourne uniquement les tâches correspondantes de l'immeuble, avec `total`.
2. Aucun paramètre → comportement actuel (toutes les tâches, ordre `created_at DESC`, limite par défaut 50).
3. UI : onglets statut, filtres priorité/assigné/prestataire, recherche avec debounce, pagination fonctionnelle.
4. Filtre invalide → `400` avec code i18n.

## 7. Plan de tests

- **e2e** (`backend/tests/api_e2e_test.rs`) :
  1. Seed 3 tâches (statuts/priorités variés) → filtre `status` → seule la bonne revient.
  2. Filtre `q` sur un mot du titre puis de la description.
  3. `limit=1&offset=1` → pagination correcte + `total == 3`.
  4. `status=bogus` → `400 errors.tasks.invalid_filter`.
- **Frontend** : vérification manuelle (recherche + filtres combinés + pagination).

## 8. Ordre d'implémentation

1. DB `list` étendu (SQL statique nullable) → `cargo sqlx prepare`.
2. API `ListTasksQuery` + validation + réponse `{items, total}`.
3. Tests e2e.
4. Frontend api client + `TasksView` (filtres, recherche, pagination).
