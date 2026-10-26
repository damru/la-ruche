# Spécification technique — EP-008-S08

**Story :** Documents de sinistre — accès restreint au Syndic (et Conseil), conformément au PRD.

**Épic :** EP-008 — Sinistres
**Réf. PRD :** US-SIN-006
**État :** IMPLÉMENTÉ (correctif sécurité appliqué)

---

## 1. État actuel du code (avant correctif)

| Élément | Emplacement | Constat |
|---|---|---|
| Routes documents claim | `backend/src/api/mod.rs:288-293` | `GET`/`POST /claims/{id}/documents`, `DELETE` existent |
| `list_documents` | `backend/src/api/claims.rs:96-107` | **Aucun guard** — tout résident authentifié de l'immeuble peut lister les pièces |
| `attach_document` | `backend/src/api/claims.rs:113-125` | `require_role(&Role::Manager)` — réservé au syndic seul, le Conseil n'y a pas accès |
| `detach_document` | `backend/src/api/claims.rs:128-138` | `require_role(&Role::Manager)` — réservé au syndic seul |

Écart : `list_documents` n'avait aucun contrôle d'accès ; `attach_document` et `detach_document` étaient restreints au Manager seul, excluant le Conseil syndical (BoardMember) qui a pourtant accès aux sinistres.

## 2. Périmètre

**Correctif sécurité** — ajout de guards sur les trois endpoints documents + masquage des brouillons (S06) pour les non-Managers. Aucune migration, aucun changement de route, aucun changement de la logique documents.

Rôles retenus : `Manager | BoardMember` sur `list_documents`, `attach_document`, `detach_document` — cohérent avec l'accès claims du Conseil syndical (US-SIN-006).

Règle supplémentaire (S06) : un claim en statut `draft` renvoie `404` pour tout rôle autre que Manager sur les trois endpoints documents (les brouillons sont invisibles hors syndic).

## 3. Backend

`backend/src/api/claims.rs` — trois changements :

```rust
// list_documents — ajout de guard (n'en avait aucun) :
auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
get_claim_for_document_op(&state, id, auth.building_id, &auth).await?;

// attach_document — assouplissement Manager → Manager|BoardMember + guard draft :
auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
get_claim_for_document_op(&state, id, auth.building_id, &auth).await?;

// detach_document — assouplissement Manager → Manager|BoardMember + guard draft :
auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
get_claim_for_document_op(&state, id, auth.building_id, &auth).await?;
```

Helper `get_claim_for_document_op` : récupère le claim et renvoie `404` si le claim est `draft` et l'utilisateur n'est pas Manager.

L'erreur `403` produite par `require_any_role` suit le mécanisme i18n existant.

## 4. Frontend

Vérifier que l'UI documents dans `ClaimsView.vue` (ou composant dédié) est déjà conditionnée au rôle ; si un co_owner voyait la section documents, la masquer pour les rôles non autorisés (cohérence UI/API). Aucun autre changement.

## 5. Cas limites

| Cas | Règle |
|---|---|
| co_owner / tenant → `GET /claims/{id}/documents` | `403` |
| co_owner / tenant → `POST /claims/{id}/documents` | `403` |
| BoardMember → GET/POST/DELETE | `200`/`201`/`204` (aligné sur `detach`) |
| Claim d'un autre immeuble | `404` (comportement existant conservé) |
| BoardMember sur claim `draft` | `404` (brouillon invisible hors Manager) |
| Documents déjà attachés par un co_owner avant le correctif | Conservés — pas de purge rétroactive |

## 6. Critères d'acceptation

1. `list_documents`, `attach_document` et `detach_document` exigent `Manager | BoardMember`.
2. Un claim `draft` renvoie `404` sur les trois endpoints pour tout rôle autre que Manager.
3. Tests e2e prouvant le `403` pour co_owner **et** tenant sur les trois endpoints.
4. Non-régression : Manager et BoardMember conservent l'accès complet sur les claims non-draft.
5. Correctif signalé comme **security fix** dans le commit.

## 7. Plan de tests (e2e — `backend/tests/api_e2e_test.rs`)

1. co_owner → `GET /claims/{id}/documents` → `403`.
2. co_owner → `POST /claims/{id}/documents` → `403`.
3. tenant → `403` sur GET et POST.
4. BoardMember → `GET` → `200` ; `POST` → `201` ; `DELETE` → `204`.
5. Manager → non-régression complète GET/POST/DELETE.
6. BoardMember → claim `draft` → `404` sur GET/POST/DELETE documents.
7. Manager → claim `draft` → accès normal (non-régression S06).

## 8. Ordre d'implémentation

1. Ajouter le guard `require_any_role(Manager|BoardMember)` sur `list_documents`, `attach_document`, `detach_document`.
2. Ajouter le helper `get_claim_for_document_op` (404 si draft + non-Manager).
3. Tests e2e (§7), boucler au vert.
4. Vérifier le masquage UI par rôle.
