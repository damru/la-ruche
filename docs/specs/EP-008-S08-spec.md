# Spécification technique — EP-008-S08

**Story :** Documents de sinistre — accès restreint au Syndic (et Conseil), conformément au PRD.

**Épic :** EP-008 — Sinistres
**Réf. PRD :** US-SIN-006
**État :** PARTIEL — **faille de sécurité** : correctif petit et prioritaire.

---

## 1. État actuel du code (faille)

| Élément | Emplacement | Constat |
|---|---|---|
| Routes documents claim | `backend/src/api/mod.rs:288-293` | `GET`/`POST /claims/{id}/documents`, `DELETE` existent |
| `attach_document` | `backend/src/api/claims.rs:121-133` | **Aucun `require_any_role`** — tout résident authentifié de l'immeuble peut attacher |
| `list_documents` | `backend/src/api/claims.rs:104-114` | **Aucun `require_any_role`** — tout résident peut lister les pièces (constats, devis, données sensibles) |
| `detach_document` | `backend/src/api/claims.rs:140` | Correct : `Manager \| BoardMember` |

Écart : aligner `attach` et `list` sur le guard déjà présent sur `detach`.

## 2. Périmètre

**Correctif sécurité pur** — deux lignes de guard + tests. Aucune migration, aucun changement de route, aucun changement de la logique documents.

Rôles retenus : `Manager | BoardMember` (même ensemble que `detach_document:140`, cohérent avec l'accès claims du Conseil). Si le PRD US-SIN-006 exige strictement « Syndic seul », restreindre les trois endpoints à `Manager` — **à confirmer avec le PO avant implémentation** ; par défaut cette spec aligne sur l'existant (`Manager|BoardMember`).

## 3. Backend

`backend/src/api/claims.rs` :

```rust
// list_documents (:104-114) — add as first statement:
auth.require_any_role(&[Role::Manager, Role::BoardMember])?;

// attach_document (:121-133) — add as first statement:
auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
```

Aucun autre changement. L'erreur `403` produite par `require_any_role` suit le mécanisme i18n existant.

## 4. Frontend

Vérifier que l'UI documents dans `ClaimsView.vue` (ou composant dédié) est déjà conditionnée au rôle ; si un co_owner voyait la section documents, la masquer pour les rôles non autorisés (cohérence UI/API). Aucun autre changement.

## 5. Cas limites

| Cas | Règle |
|---|---|
| co_owner / tenant → `GET /claims/{id}/documents` | `403` |
| co_owner / tenant → `POST /claims/{id}/documents` | `403` |
| BoardMember → GET/POST/DELETE | `200`/`201`/`204` (aligné sur `detach`) |
| Claim d'un autre immeuble | `404` (comportement existant conservé) |
| Documents déjà attachés par un co_owner avant le correctif | Conservés — pas de purge rétroactive ; noter dans `PROGRESS.md` |

## 6. Critères d'acceptation

1. `list_documents` et `attach_document` exigent `Manager | BoardMember`.
2. Tests e2e prouvant le `403` pour co_owner **et** tenant sur les deux endpoints.
3. Non-régression : Manager et BoardMember conservent l'accès complet.
4. Correctif signalé comme **security fix** dans le commit (`fix(claims): restrict claim document access to manager/board`).

## 7. Plan de tests (e2e — `backend/tests/api_e2e_test.rs`)

1. co_owner → `GET /claims/{id}/documents` → `403`.
2. co_owner → `POST /claims/{id}/documents` → `403`.
3. tenant → mêmes deux assertions → `403`.
4. BoardMember → `GET` → `200` ; `POST` → `201`.
5. Manager → non-régression complète GET/POST/DELETE.

## 8. Ordre d'implémentation

1. Ajouter les deux guards (`claims.rs:104-114`, `:121-133`).
2. Tests e2e (§7), boucler au vert.
3. Vérifier le masquage UI par rôle.
4. Commit dédié (fix sécurité, séparé de tout autre travail), `docs/PROGRESS.md`, `graphify update .`.
