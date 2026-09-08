# Spécification technique — EP-012-S06

**Story :** Consulter la fiche synthétique publiée depuis l'espace résident — accès en lecture pour les copropriétaires et le Conseil syndical à la dernière fiche validée.

**Réfs :** Épic EP-012 (`docs/EPICS.md` §EP-012), LEGAL.md §1.3 (droit d'information art. 8-2 loi 1965), PRD `docs/product/PRD - Conformité et reporting.md` US-CR-004 (AC-CR-004-A/B), matrice CRUD §3 (`R` Conseil, `R²` Copropriétaire — dernière publiée uniquement, `X` Locataire).

> S'appuie sur EP-012-S02 (table, payload) et EP-012-S05 (statut `published`). **Prérequis : S05 livré.** Pas de migration. L'export PDF de cette même vue est en S07.

## 1. Quoi (résumé fonctionnel)

Un copropriétaire (`CoOwner`) ou un membre du Conseil (`BoardMember`) consulte en ligne la dernière fiche synthétique publiée de sa copropriété. Une seule route lecture dédiée, indépendante des routes de gestion Manager (décision S05 §4.4). Les locataires (`Tenant`) n'y ont pas accès.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Table + statuts | `fiche_synthetique` (S02/S05) |
| Repo | `backend/src/db/fiche_synthetique.rs` |
| Rôles | `backend/src/domain/mod.rs` (`Manager`, `BoardMember`, `CoOwner`, `Tenant`) — vérifier la sémantique exacte de `require_role` (hiérarchique vs strict) avant d'écrire le guard |
| Espace résident frontend | `frontend/src/features/residence/` (`BuildingView.vue`), routes `residence/building`, `residence/keys` dans `frontend/src/router/index.ts` |
| Vue Syndic | `frontend/src/features/compliance/FicheSynthetiqueView.vue` (S02) — les composants d'affichage des sections sont à partager |

## 3. Modèle de données

Aucune migration.

## 4. Backend Rust

### 4.1 `backend/src/db/fiche_synthetique.rs`

```rust
/// Latest published fiche for the building (highest fiscal_year,
/// then latest published_at). None if nothing was ever published.
pub async fn latest_published(pool, building_id) -> AppResult<Option<FicheRow>>
```
`SELECT ... WHERE building_id = $1 AND status = 'published' ORDER BY fiscal_year DESC, published_at DESC LIMIT 1`.

### 4.2 `backend/src/api/fiche_synthetique.rs` — `published` (handler mince)

- Guard : tout rôle **sauf** `Tenant` → si `require_role` est hiérarchique, `auth.require_role(&Role::CoOwner)?` suffit ; sinon vérifier explicitement `role != Tenant` via le helper du projet. 403 pour un locataire.
- `latest_published` → 404 `AppError::NotFound` si aucune fiche publiée (le frontend affiche un état vide dédié).
- Réponse : fiche complète (payload + `fiscal_year` + `published_at`) — **sans** `manual_overrides` ni méta de gestion (trace d'audit interne au Syndic, S04).

### 4.3 `backend/src/api/mod.rs`

```rust
.route("/fiche-synthetique/published", get(fiche_synthetique::published))
```
⚠️ Déclarer cette route **avant** `/fiche-synthetique/{id}` ou vérifier que le matcher axum ne capture pas `published` comme `{id}` (UUID parse → 400 sinon ; un test e2e couvre ce point).

## 5. Frontend Vue

- Nouveau `frontend/src/features/residence/FichePubliqueView.vue` : rendu lecture seule des six sections du payload (réutiliser les composants de section extraits de `FicheSynthetiqueView.vue` — les factoriser en composants partagés dans `features/compliance/` plutôt que dupliquer).
- État vide : « Aucune fiche synthétique publiée » si 404.
- Route `residence/fiche` (name `fiche-publique`), navigation espace résident ; masquée pour les locataires (cohérent avec le 403 backend).
- `ficheApi.published()` dans `frontend/src/api/index.ts`.
- Bouton « Exporter en PDF » : placé ici mais livré en S07 (`EP-012-S07-spec.md`).

## 6. Cas limites

- Aucune fiche publiée → 404 backend, état vide frontend (pas d'erreur affichée).
- Fiche archivée uniquement (republication en cours, ancien exercice) → `latest_published` ne retourne que du `published` : si la seule fiche a été archivée sans successeur publié, état vide — comportement voulu (AC-CR-004-A).
- Locataire → 403 backend + entrée de navigation absente.
- Le Manager peut aussi appeler `/published` (lecture élargie, aucun risque).

## 7. Critères d'acceptation

- **AC-CR-004-A** : seule la fiche `published` la plus récente est retournée aux copropriétaires ; jamais un `draft` ni une `archived`.
- **AC-CR-004-B** : la fiche est consultable en ligne dans l'espace résident (toutes sections affichées).
- **AC-S06-C** : `Tenant` → 403 ; `CoOwner`, `BoardMember`, `Manager` → 200.
- **AC-S06-D** : `manual_overrides` n'apparaît pas dans la réponse résident.

## 8. Plan de tests

- e2e HTTP : 200 avec la bonne fiche quand plusieurs statuts coexistent (draft + published + archived) ; 404 sans publication ; 403 tenant ; absence de `manual_overrides` dans la réponse ; route `/published` non capturée par `/{id}`.
- Frontend : rendu sections, état vide, visibilité navigation par rôle.

## 9. Ordre d'implémentation

1. `db::latest_published`.
2. `api::published` + route (ordre de déclaration §4.3) + tests e2e.
3. Factorisation des composants de section + `FichePubliqueView.vue` + route + navigation.
4. `graphify update .` + entrée `docs/PROGRESS.md` (EP-012-S06).
