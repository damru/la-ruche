# Spécification technique — EP-011-S05

**Story :** Anonymat du déclarant au choix à la création d'un signalement : l'identité n'est exposée à personne (ni résidents, ni syndic) — seul le déclarant se voit lui-même.

**Épic :** EP-011 — Signalements
**Réf. PRD :** US-SIG-007
**État :** NON IMPLÉMENTÉ.

---

## 1. Quoi (résumé fonctionnel)

- À la création d'un signalement, le déclarant peut cocher **« Signaler anonymement »**.
- **Règle de confidentialité :** pour un signalement anonyme, l'identité du déclarant n'apparaît **dans aucune réponse API ni aucun écran**, y compris pour le syndic (liste et fiche). Seule exception : le déclarant lui-même, qui voit son propre signalement attribué (« vous »).
- **`author_id` reste stocké en base** (NOT NULL inchangé) : responsabilité/traçabilité en cas d'abus, et support des contrôles d'accès existants (le déclarant garde ses droits sur son signalement). L'anonymat est une règle de **sérialisation API**, pas d'effacement.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement | Constat |
|---|---|---|
| Table `report` (`author_id UUID NOT NULL`) | `backend/migrations/0001_init.sql` L654-665 | Pas de colonne `is_anonymous` |
| `ReportRow` expose `author_id`, `author_first_name`, `author_last_name` (JOIN `user_account`) | `backend/src/db/report.rs` L7-61 | Identité sérialisée sans condition |
| Handlers reports | `backend/src/api/reports.rs` L18-174 | Aucune notion d'anonymat |
| Modale de création | `frontend/src/features/reports/CreateReportModal.vue` | Aucune case à cocher anonymat |
| Client API `reportsApi.create` | `frontend/src/api/index.ts` L431-432 | Payload sans `is_anonymous` |

## 3. Modèle de données

**Mettre à jour `docs/ARCHITECTURE.md` avant la migration.** Pas d'ADR : le choix structurant (garder `author_id` + flag de masquage, plutôt que `author_id` nullable) est motivé ci-dessus et sans alternative sérieuse — un `author_id` NULL casserait les droits du déclarant sur son propre signalement et la traçabilité.

### Migration `backend/migrations/NNNN_report_anonymous.sql`

```sql
-- EP-011-S05: optional reporter anonymity (US-SIG-007).
-- author_id stays NOT NULL (accountability + reporter's own access rights);
-- anonymity is an API-serialization rule: when is_anonymous, the author
-- identity is masked for everyone except the reporter themself.
ALTER TABLE report
    ADD COLUMN is_anonymous BOOLEAN NOT NULL DEFAULT FALSE;
```

Immuable après création (le choix est fait « à la création », PRD) — aucun endpoint ne modifie ce flag.

Puis `cargo sqlx migrate run` + `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend

### 4.1 `backend/src/db/report.rs`

- `ReportRow` : ajouter `pub is_anonymous: bool` ; propager la colonne dans les deux `SELECT` (L29-36, L42-49) et le mapping (L59-61).
- `create(...)` : nouveau paramètre `is_anonymous: bool`, colonne ajoutée à l'INSERT (L123-127).
- **Aucun masquage en couche `db`** : la règle dépend du requérant (identité vs déclarant), c'est une responsabilité de sérialisation.

### 4.2 Règle de sérialisation — `backend/src/domain/` (zéro logique dans les handlers)

Nouvelle fonction domaine, par ex. `backend/src/domain/report_privacy.rs` :

```rust
/// Masks the reporter identity on an anonymous report unless the viewer IS
/// the reporter. Applied to every serialized report (list, get, create).
/// Masked fields: author_id, author_first_name, author_last_name → null.
/// Adds "is_author": bool so the reporter's own UI can label it "vous".
pub(crate) fn mask_report_author(
    report: &mut serde_json::Value,
    viewer_id: Uuid,
)
```

Comportement : si `is_anonymous == true` et `author_id != viewer_id` → `author_id`/`author_first_name`/`author_last_name` mis à `null`. Dans tous les cas, injecter `is_author = (author_id == viewer_id)` **avant** le masquage (le déclarant anonyme voit `is_author: true`, les autres `false` sans pouvoir en déduire l'identité).

### 4.3 `backend/src/api/reports.rs`

- `CreateReportInput` : champ `#[serde(default)] pub is_anonymous: bool`.
- `list`, `get`, `create` (et toute réponse renvoyant un report, ex. `update`, `update_status`) : appliquer `mask_report_author(&mut value, auth.user_id)` sur chaque élément avant de répondre. C'est le **seul point de sortie** des reports — vérifier qu'aucun autre module ne sérialise `ReportRow` (ex. conversion en tâche : la task créée ne doit pas copier le nom du déclarant ; vérifier `convert_to_task`, L~100+, et n'y référencer aucune identité si `is_anonymous`).
- Droits inchangés : les guards existants (déclarant modifie/supprime son signalement) continuent de comparer `auth.user_id` au `author_id` **lu en base**, jamais à la valeur sérialisée.

## 5. Frontend

### 5.1 Types et API

- `frontend/src/types/domain.ts` : `Report` — `author_id: string | null`, `author_first_name: string | null`, `author_last_name: string | null`, `is_anonymous: boolean`, `is_author: boolean`.
- `frontend/src/api/index.ts` : `create` accepte `is_anonymous?: boolean` dans le payload.

### 5.2 `CreateReportModal.vue`

Case à cocher **« Signaler anonymement »** avec `field-hint` : « Votre identité ne sera visible de personne, y compris du syndic. Le signalement reste rattaché à votre compte à des fins de modération. » Valeur transmise au `create`.

### 5.3 Affichage (liste + fiche)

- Auteur affiché : si `is_anonymous` → badge « Anonyme » ; si de plus `is_author` → « Anonyme (vous) ».
- Sinon comportement actuel (prénom + nom).
- Garde-fou : ne jamais dériver l'affichage de `author_id` seul — utiliser les champs nom (null = masqué).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Syndic (manager) consulte un signalement anonyme | Identité masquée — pas d'exception de rôle (PRD : « ni le syndic ») |
| Le déclarant anonyme consulte son signalement | `is_author: true`, identité visible pour lui seul (« vous ») |
| Modification/suppression par le déclarant anonyme | Fonctionne (guards sur `author_id` en base, non sérialisé) |
| Changer l'anonymat après création | Impossible — aucun endpoint ne touche `is_anonymous` |
| Conversion en tâche (`POST /reports/{id}/task`) | La tâche générée ne doit porter aucune identité du déclarant si anonyme |
| Commentaires de suivi (EP-011-S04) | Indépendant : les commentaires portent l'identité de leur propre auteur (gestionnaire), jamais celle du déclarant |
| Fuite indirecte (`created_at`, `unit_id`) | `unit_id` peut désanonymiser ; si le déclarant coche anonyme **et** renseigne un lot, l'UI affiche un avertissement (`field-hint`) — le choix reste le sien |
| Signalements existants | `DEFAULT FALSE` — comportement inchangé |

## 7. Critères d'acceptation

1. Un signalement créé avec `is_anonymous: true` n'expose `author_id` ni les noms à aucun autre utilisateur (liste, fiche, réponses de mutation), quel que soit son rôle.
2. Le déclarant voit son signalement anonyme marqué « Anonyme (vous) » et conserve modification/suppression.
3. La case à cocher n'apparaît qu'à la création ; l'anonymat n'est pas modifiable ensuite.
4. Les signalements existants et non anonymes affichent l'auteur comme avant.

## 8. Plan de tests

Unitaire (`domain/report_privacy.rs`, `#[cfg(test)]`) : masquage appliqué/non appliqué selon `viewer_id`, `is_author` correct dans les 4 combinaisons (anonyme × viewer).

E2e (`backend/tests/api_e2e_test.rs`) :

1. Création anonyme → 201 ; la réponse du créateur contient `is_author: true` et son identité.
2. `GET /reports` en tant que **manager** → l'élément anonyme a `author_id/author_first_name/author_last_name` null, `is_anonymous: true`.
3. `GET /reports/{id}` en tant que résident tiers → identité null (ou 403 selon la règle d'accès existante).
4. Le déclarant anonyme peut `PUT` et `DELETE` son signalement.
5. Signalement non anonyme → identité visible du manager (non-régression).
6. `POST /reports/{id}/task` sur un anonyme → la tâche créée ne contient pas le nom du déclarant.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (colonne `is_anonymous`) → migration `NNNN` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. `db/report.rs` : colonne dans row/SELECT/INSERT → `cargo build`.
3. `domain/report_privacy.rs` : `mask_report_author` + tests unitaires → `cargo test --lib`.
4. `api/reports.rs` : `is_anonymous` dans l'input, masquage sur toutes les sorties, audit `convert_to_task` → `cargo clippy -- -D warnings`.
5. Tests e2e §8, boucler jusqu'au vert.
6. Frontend : types, payload `create`, case à cocher `CreateReportModal.vue`, badge « Anonyme » liste/fiche + i18n.
7. `npm --prefix frontend run build && npm --prefix frontend run lint` ; `docs/EPICS.md` / `docs/PROGRESS.md`.
