# Spécification technique — EP-012-S05

**Story :** Publier la fiche synthétique annuelle — action explicite du Syndic rendant la fiche accessible en lecture aux copropriétaires et membres du Conseil.

**Réfs :** Épic EP-012 (`docs/EPICS.md` §EP-012), LEGAL.md §1.3, PRD `docs/product/PRD - Conformité et reporting.md` US-CR-003 (AC-CR-003-A à C), §2 (immuabilité de la fiche publiée), §4 (cycle de vie).

> S'appuie sur EP-012-S02 (table, cycle de vie, `can_publish`) et EP-012-S04 (brouillon vérifié). **Prérequis : S02 livré ; S04 fortement recommandé avant mise en production.** La consultation résident est en S06, l'export PDF en S07. Pas de migration.

## 1. Quoi (résumé fonctionnel)

Action explicite `POST /fiche-synthetique/{id}/publish` (Syndic). Effets atomiques (une transaction) :
1. la fiche `draft` passe `published` (`published_at = NOW()`, `published_by = auth.user_id`) — immuable dès lors (AC-CR-003-A) ;
2. si une fiche `published` existe déjà pour le même exercice, elle passe `archived` (AC-CR-003-B) ;
3. les copropriétaires sont notifiés de la disponibilité (AC-CR-003-C) via le mécanisme de notification existant du projet — si aucun mécanisme n'est disponible au moment de l'implémentation, le point est dégradé en évènement d'audit + bandeau frontend.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Table + index partiels (`fiche_one_published_per_year`) | migration S02 (`EP-012-S02-spec.md` §3.1) |
| `can_publish(status)` | `backend/src/domain/fiche_synthetique.rs` (S02) |
| Repo | `backend/src/db/fiche_synthetique.rs` |
| Précédent de transition transactionnelle + FOR UPDATE | `backend/src/db/contract.rs::activate` (EP-018-S05) — modèle à imiter |
| Audit | `backend/src/db/audit.rs` (si journal d'audit générique — vérifier avant d'en créer un) |
| Routes | `backend/src/api/mod.rs` bloc `// ── Fiche synthétique (EP-012)` |

## 3. Modèle de données

Aucune migration : `status`, `published_at`, `published_by` existent depuis S02. L'index partiel `fiche_one_published_per_year` garantit en base l'unicité de la fiche publiée par exercice — l'archivage de l'ancienne DOIT précéder le passage `published` de la nouvelle dans la même transaction pour ne pas violer l'index.

## 4. Backend Rust

### 4.1 `backend/src/db/fiche_synthetique.rs` — `publish`

```rust
/// Atomically publishes a draft fiche. Archives any previously published
/// fiche for the same (building, fiscal_year) first (AC-CR-003-B).
pub async fn publish(pool, id, building_id, user_id) -> AppResult<Option<FicheRow>>
```
- `SELECT ... FOR UPDATE` de la fiche cible ; `None` si absente.
- Statut ≠ `draft` → `AppError::validation_code("errors.fiche.not_publishable")` (la règle vit dans `domain::can_publish`, appelée ici avec le statut verrouillé).
- `UPDATE ... SET status = 'archived' WHERE building_id = $b AND fiscal_year = $y AND status = 'published'`.
- `UPDATE ... SET status = 'published', published_at = NOW(), published_by = $u WHERE id = $id`.
- Commit ; retourne la ligne publiée.

### 4.2 `backend/src/api/fiche_synthetique.rs` — `publish` (handler mince)

`auth.require_role(&Role::Manager)?` puis délégation à `db::fiche_synthetique::publish`. Après commit : déclenchement notification (AC-CR-003-C, §1.3) hors transaction — un échec de notification ne doit pas annuler la publication (logguer + continuer).

### 4.3 Immutabilité post-publication

`api::update` (S04) refuse déjà toute fiche non-`draft`. Ajouter en défense un trigger PG **uniquement si** le pattern existant du projet le fait (miroir `trg_vendor_contract_immutable`, migration EP-018-S05) ; sinon le guard applicatif + l'absence de route de mutation suffisent pour le MVP.

### 4.4 `backend/src/api/mod.rs`

```rust
.route("/fiche-synthetique/{id}/publish", post(fiche_synthetique::publish))
```

Élargissement lecture : `GET /fiche-synthetique/{id}` et `GET /fiche-synthetique` restent Manager-only ; l'accès lecture Conseil/copropriétaires passe par la route dédiée de S06 (`GET /fiche-synthetique/published`) — une seule porte d'entrée résident, plus simple à auditer.

## 5. Frontend Vue

- `FicheSynthetiqueView.vue` : bouton « Publier » sur une fiche `draft`, avec modale de confirmation `PublishFicheModal.vue` (rappel : action irréversible, archivage de la fiche publiée précédente le cas échéant).
- Badge `published` / `archived` ; fiche publiée affichée en lecture seule (plus de bouton Modifier).
- `ficheApi.publish(id)` dans `frontend/src/api/index.ts`.
- Nouveau composant modale → `docs/DESIGN.md` §2.

## 6. Cas limites

- Publier une fiche `published` ou `archived` → 422 `errors.fiche.not_publishable`.
- Republication d'un exercice (nouveau draft corrigé puis publié) : la révision précédente passe `archived` — l'historique est conservé, jamais supprimé (Q-CR-003 : pas de consultation multi-exercices MVP, mais les lignes restent en base).
- Deux publications concurrentes du même draft : `FOR UPDATE` sérialise ; la seconde voit `published` et reçoit 422.
- Échec de notification → publication maintenue, erreur logguée (jamais de rollback pour un effet de bord).

## 7. Critères d'acceptation

- **AC-CR-003-A** : après publication, `status = published`, toute tentative de modification → 422.
- **AC-CR-003-B** : la fiche publiée précédente du même exercice passe `archived` atomiquement.
- **AC-CR-003-C** : une notification (ou l'alternative dégradée §1.3) est émise vers les copropriétaires.
- **AC-S05-D** : rôle non-Manager → 403 sur `/publish`.

## 8. Plan de tests

- e2e HTTP : publication nominale (`published_at`/`published_by` posés) ; archivage de l'ancienne publiée ; 422 double publication ; 422 sur `archived` ; PUT post-publication refusé ; 403 `board_member`.
- Unitaires : `can_publish` (déjà en S02 — compléter si besoin).
- Frontend : modale de confirmation, disparition des actions d'édition après publication.

## 9. Ordre d'implémentation

1. `db::publish` (transaction + FOR UPDATE) + tests e2e transitions.
2. `api::publish` + route + notification post-commit.
3. Frontend (bouton, modale, badges) + `docs/DESIGN.md`.
