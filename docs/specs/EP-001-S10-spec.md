# Spécification technique — EP-001-S10

**Story :** Journal d'audit des modifications de clés (qui, quoi, quand, justification) pour traçabilité syndic (LEGAL.md §1.2, §2.1)

**Épic :** EP-001 — Configuration copropriété & clés de répartition
**Réf. LEGAL :** §1.2, §2.1 · **ADR :** aucun requis (réutilisation de l'infra d'audit existante)

**Statut initial :** PARTIEL — l'infrastructure d'audit générique existe déjà ; seul le câblage sur les clés de répartition manque. Cette spec ne couvre que le gap restant.

---

## 1. Ce qui existe déjà (ne pas recréer)

| Élément | Emplacement |
|---|---|
| Table append-only `audit_log` | `backend/migrations/0029_audit_log.sql` |
| Repo audit | `backend/src/db/audit.rs` — `NewAuditEntry` :11, `insert` :40, `list_for_entity` :63 |
| Exemples de câblage | budget : `backend/src/db/budget.rs:642`, `:727` ; contrats : `backend/src/db/contract.rs:297`, `:366`, `:389` |

## 2. Ce qui manque (périmètre de cette story)

| Gap | Emplacement |
|---|---|
| Aucune écriture d'audit sur les clés | `backend/src/db/allocation_key.rs` — `create` :65, `update` :84, `delete`, `set_share` (appelé depuis `backend/src/api/building.rs:373`) : zéro appel audit |
| Pas de champ `justification` | `AllocationKeyInput` — `backend/src/api/building.rs:169` |
| Aucun endpoint de lecture du journal | — |
| Aucune UI d'historique | — |

## 3. Modèle de données

**Aucune migration nécessaire** : `audit_log` (0029) est générique (`entity_type`, `entity_id`, `action`, `old_values`/`new_values` JSONB, `actor_id`, `created_at`). La justification est portée dans le payload JSONB (`new_values.justification`), comme métadonnée d'action — pas de colonne dédiée, cohérent avec l'usage budget/contrats.

Pas de mise à jour `docs/ARCHITECTURE.md` (pas de nouvelle table), pas d'ADR.

## 4. Backend

### 4.1 API — champ `justification`

- Ajouter `justification: Option<String>` sur `AllocationKeyInput` (`api/building.rs:169`) et sur le payload de mutation des tantièmes (`set_share`, handler `api/building.rs:373`).
- Optionnel ; si présent, borner la longueur (ex. 500 chars) → `AppError::validation_code("errors.allocation_keys.justification_too_long")`.
- Les handlers restent sans logique métier : ils passent la justification au repo.

### 4.2 DB — câblage audit (`db/allocation_key.rs`)

Suivre le modèle exact de `db/contract.rs:297` : dans la **même transaction** que la mutation, insérer via `db/audit.rs::insert` :

- `create` (:65) → action `allocation_key.created`, `new_values` = clé sérialisée + justification.
- `update` (:84) → `allocation_key.updated`, `old_values`/`new_values` (diff ciblé) + justification.
- `delete` → `allocation_key.deleted`, `old_values` = état supprimé + justification.
- `set_share` → `allocation_key.share_set`, `old_values`/`new_values` = `{unit_id, share_value}` + justification.

`entity_type = "allocation_key"`, `entity_id = allocation_key.id` (y compris pour `set_share` : l'historique d'une clé regroupe ses tantièmes).

### 4.3 API — endpoint de lecture

Route à déclarer dans `backend/src/api/mod.rs` :

- `GET /buildings/{building_id}/allocation-keys/{id}/audit` — manager + conseil syndical, lecture seule ; délègue à `db/audit.rs::list_for_entity("allocation_key", id)` (:63). Réponse : liste chronologique `{action, actor, created_at, old_values, new_values}`.

## 5. Frontend

- `frontend/src/api/index.ts` : ajouter `listAudit(keyId)` à l'API clés existante.
- `frontend/src/types/domain.ts` : type `AuditEntry`.
- Formulaires de clé et d'édition des tantièmes : champ optionnel « Justification » (textarea) transmis à la mutation.
- **UI historique minimale** : bouton « Historique » sur la fiche/ligne de clé ouvrant une modale listant les entrées (date, auteur, action, justification, diff). Utiliser les composants catalogue existants (`CModal`/équivalent) ; **tout nouveau composant `C*` doit être ajouté au catalogue `docs/DESIGN.md §3` avant usage** (étape à part, ne pas éditer DESIGN.md dans cette spec).

## 6. Cas limites

- Justification absente → l'entrée d'audit est écrite quand même (champ facultatif) ; l'UI affiche « — ».
- `set_share` sans changement de valeur → ne pas écrire d'entrée (comparer old/new avant insert, comme le diff ciblé budget).
- Échec de l'insert audit → rollback de la mutation (même transaction) : le journal ne peut pas diverger des données.
- Clé supprimée → son historique reste lisible via l'endpoint (l'audit est append-only et n'a pas de FK bloquante).
- Aucune interaction avec `domain/immutability.rs` : les clés ne sont pas gelées par cette story, seulement tracées.

## 7. Critères d'acceptation

1. Toute création/modification/suppression de clé et toute modification de tantième écrit une ligne `audit_log` (qui, quoi, quand) dans la même transaction.
2. La justification saisie est conservée et restituée.
3. `GET .../allocation-keys/{id}/audit` retourne l'historique complet, ordonné, incluant les mutations de tantièmes.
4. Un `set_share` sans changement effectif ne crée pas d'entrée.
5. L'UI expose la saisie de justification et la consultation de l'historique.

## 8. Plan de tests

- **E2E** (`backend/tests/api_e2e_test.rs`, harnais `common::isolated_test_server()`) :
  - create → update → set_share → delete d'une clé, puis `GET .../audit` : 4 entrées, actions et acteurs corrects, justification présente ;
  - set_share idempotent (même valeur) → pas d'entrée supplémentaire ;
  - justification > borne → 422 `errors.allocation_keys.justification_too_long` ;
  - rôle copropriétaire sur l'endpoint audit → 403.
- Pas de logique domaine nouvelle → pas de test domaine pur dédié (le diff ciblé est testable via E2E, à l'image de budget/contrats).

## 9. Ordre d'implémentation

1. Champ `justification` sur les payloads API (`api/building.rs`).
2. Câblage audit dans `db/allocation_key.rs` (4 mutations, même transaction) — `cargo sqlx prepare` ensuite.
3. Endpoint de lecture + route `api/mod.rs`.
4. Frontend : champ justification + modale historique (cataloguer tout nouveau `C*` dans DESIGN.md §3 d'abord).
5. Tests E2E, `graphify update .`, entrée `docs/PROGRESS.md`.
