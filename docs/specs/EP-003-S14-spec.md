# Spécification technique — EP-003-S14

**Story :** Consultation du dossier AG en lecture par les résidents selon leur rôle — notamment accès **locataire** limité aux résultats des AG clôturées et au PV publié.

**Épic :** EP-003 — Assemblées générales
**Réf. LEGAL :** locataire = résultats et PV publié uniquement · **Réf. DESIGN :** `docs/DESIGN.md` §4.2 · **Réf. PRD :** US-AG-* (consultation) · **Réf. ADR :** aucun

**Statut actuel : IMPLÉMENTÉ (backend + frontend).**

---

## 1. Ce qui existe déjà (ne pas retoucher)

| Élément | Emplacement |
|---|---|
| Lectures AG ouvertes aux co_owner/board (list, get, résolutions, présences) | `backend/src/api/assemblies.rs` L198, L221, L281, L355 |
| Documents filtrés par rôle (`visible_to_roles && $3`) | `backend/src/db/assembly.rs` L757 |

## 2. Écarts restants

- **(a)** Le routeur frontend exclut le rôle `tenant` des vues AG : `frontend/src/router/index.ts` L78, L84 — `roles: ['manager','board_member','co_owner']`.
- **(b)** `get_minutes_draft` est manager-only (`backend/src/api/assemblies.rs` L561) — correct pour le *brouillon*, mais aucun chemin de lecture « PV publié » pour les autres rôles n'est spécifié côté locataire.
- **(c)** Aucune restriction backend n'empêche un `tenant` (si le rôle atteint les routes) de lire un dossier AG non clôturé — l'accès locataire doit être **résultats des AG `closed` + PV publié uniquement**.

## 3. Modèle de données

Aucune migration. Le « PV publié » s'appuie sur l'existant : signatures posées (`sign_minutes`) et/ou document PV rattaché au meeting avec `visible_to_roles` incluant `tenant` — le filtrage documentaire existant (`db/assembly.rs` L757) fait déjà le travail pour les documents.

## 4. Backend

### 4.1 Domaine — règle de visibilité (fonction pure)

`backend/src/domain/meeting_visibility.rs` (nouveau, petit) :

```rust
/// Tenants only see closed meetings (results + published minutes).
/// Other resident roles see every meeting of their building.
pub(crate) fn meeting_visible_to(role: &Role, meeting_status: &str) -> bool {
    match role {
        Role::Tenant => meeting_status == "closed",
        _ => true,
    }
}
```

### 4.2 API — `backend/src/api/assemblies.rs`

- Ajouter `Role::Tenant` aux `require_any_role` des lectures (L198, L221, L281 — la feuille de présence L355 **reste fermée aux locataires** : donnée nominative sans intérêt locatif ; décision : tenant exclu de l'endpoint attendance).
- Appliquer `meeting_visible_to` dans les handlers de lecture : pour un `tenant`, `list` filtre sur `closed`, `get`/résolutions d'un meeting non-`closed` → 404.
- `get_minutes_draft` (L561) : inchangé, manager-only — le brouillon de PV n'est jamais un document publié. Le PV publié transite par le canal documents (déjà filtré par rôle).
- Périmètre résolutions pour tenant : lecture des résolutions **avec résultats** d'un meeting `closed` — même payload que co_owner, le filtre porte sur le statut du meeting, pas sur les champs.

### 4.3 Erreurs

Pas de nouveau code i18n : le refus de lecture locataire se matérialise en **403** (règle repo `roles-permissions.instructions.md` : un refus d'accès ne se masque jamais en 404 pour un utilisateur authentifié de la même copropriété ; l'isolation `building_id` reste stricte et renvoie 404).

## 5. Frontend

- `frontend/src/router/index.ts` L78, L84 : ajouter `'tenant'` aux rôles des routes AG (liste + détail).
- Vues `frontend/src/features/assemblies/` — états par rôle :
  - **tenant** : liste limitée aux AG clôturées (le backend filtre déjà ; masquer les filtres de statut inutiles), fiche détail en lecture seule « résultats » : résolutions + résultats + documents visibles (PV publié). Pas d'onglet présences, pas d'actions.
  - **co_owner / board_member** : inchangé (lecture complète), s'assurer qu'aucune action d'écriture ne s'affiche (état lecture seule déjà géré par statut).
  - **manager** : inchangé.
- Conditionner l'affichage sur le rôle courant (store auth existant). Aucun nouveau composant C* attendu ; sinon cataloguer dans `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Tenant GET un meeting `draft`/`called`/`ongoing` | 403 (règle repo : pas de masquage 404). |
| Tenant liste les AG | Seules les `closed` apparaissent. |
| Tenant GET la feuille de présence | 403 (rôle non autorisé sur l'endpoint). |
| Tenant GET le brouillon de PV | 403 (manager-only inchangé). |
| PV non encore publié (document absent ou `visible_to_roles` sans tenant) | Le tenant voit les résultats mais pas le PV — état frontend « PV non encore publié ». |
| Meeting d'un autre immeuble | 404 (filtre `building_id` existant). |

## 7. Critères d'acceptation

1. Un `tenant` accède aux vues AG du frontend (routes ouvertes) et ne voit que les AG clôturées.
2. Backend : tout accès tenant à un meeting non-`closed` → 404 ; liste filtrée serveur.
3. Le PV publié est visible du tenant via le canal documents dès que `visible_to_roles` l'inclut.
4. Feuille de présence et brouillon de PV restent inaccessibles au tenant.
5. Aucun changement de comportement pour manager/board_member/co_owner.

## 8. Plan de tests

- **Unitaires domaine** : `meeting_visible_to` (tenant × 4 statuts, autres rôles).
- **E2E** :
  1. tenant `GET /meetings` → uniquement les `closed` ;
  2. tenant `GET /meetings/{id}` sur `ongoing` → 404, sur `closed` → 200 avec résolutions/résultats ;
  3. tenant `GET .../attendance` → 403 ;
  4. tenant `GET .../minutes-draft` → 403 ;
  5. non-régression : co_owner voit toujours les meetings `called`.
- **Frontend** : vérifier la garde de route et l'état lecture seule tenant (si vitest configuré).

## 9. Ordre d'implémentation

1. `domain/meeting_visibility.rs` + tests unitaires.
2. `api/assemblies.rs` : rôles élargis + filtre tenant (list/get/résolutions) → `cargo clippy -- -D warnings`.
3. Tests e2e, boucler jusqu'au vert.
4. Frontend : routes (router/index.ts L78, L84), états par rôle dans les vues.
5. `docs/PROGRESS.md`.
