# Spécification technique — EP-013-S11

**Story (nouvelle) :** Guard backend de séance — refuser `cast_vote` et `save_attendance` hors statut `ongoing`, et figer définitivement après `closed`.

**Épic :** EP-013 — Procès-verbaux (intégrité de séance)
**Réf. LEGAL :** §11.1 (les votes sont immuables une fois l'AG en cours/clôturée) · **Réf. PRD :** US-AG-* (déroulé de séance) · **Réf. ADR :** aucun

**Statut actuel : NON IMPLÉMENTÉ (faille)** — seule l'UI restreint ; l'API accepte votes et présences quel que soit le statut du meeting.

---

## 1. Quoi (résumé fonctionnel)

Défense en profondeur : le backend doit refuser toute écriture de vote ou de présence quand l'AG n'est pas `ongoing`. Un client direct (curl, bug frontend) ne doit pas pouvoir voter sur une AG `draft`, `called` ou — surtout — **`closed`** (altération d'un scrutin acquis).

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| `cast_vote` — upsert sans aucun check de `meeting.status` | `backend/src/db/assembly.rs` L1266-1302 |
| `save_attendance` — idem, aucun check statut | `backend/src/db/assembly.rs` L1033-1130 |
| Handlers correspondants — aucun guard non plus | `backend/src/api/assemblies.rs` |
| Restriction uniquement côté frontend | `frontend/src/features/assemblies/` |
| Guards d'immutabilité existants (pattern à suivre) | `backend/src/domain/immutability.rs` |
| Routes meetings | `backend/src/api/mod.rs` L113-174 |

## 3. Modèle de données

Aucune migration. (Un trigger PG défensif serait cohérent avec le pattern EP-018 mais n'est pas requis par cette story — le guard applicatif suffit ; à noter comme extension possible, hors périmètre.)

## 4. Backend

### 4.1 Domaine — `backend/src/domain/immutability.rs`

Ajouter le guard de séance à côté des entités existantes :

```rust
/// Votes and attendance are session-bound: writable only while the meeting
/// is 'ongoing'. Frozen forever once 'closed' (LEGAL §11.1 — votes are
/// legally immutable), and not yet open in 'draft'/'called'.
Entity::MeetingSession => status == "ongoing",
```

avec test unitaire couvrant les quatre statuts.

### 4.2 DB — `backend/src/db/assembly.rs`

Le check doit être **dans la même transaction** que l'écriture (pas un pré-check en handler, sinon race avec la clôture) :

- `cast_vote` (L1266-1302) : ouvrir une transaction, `SELECT status FROM general_meeting WHERE id = $1 AND building_id = $2 FOR UPDATE` (ou `FOR SHARE`) ; si absent → `Ok(None)`/404 ; si `status != "ongoing"` → `Err(AppError::validation_code("errors.meetings.not_ongoing"))` ; sinon upsert existant.
- `save_attendance` (L1033-1130) : même verrou/check en tête de la transaction existante (ou nouvellement introduite), même erreur.
- Utiliser le guard domaine (`assert_mutable`-style avec `Entity::MeetingSession`) pour centraliser la règle plutôt que des comparaisons de chaînes en ligne.

### 4.3 API

- Aucun changement de route. Les handlers restent sans logique — l'erreur remonte de la couche db/domain.
- Erreur i18n unique : `errors.meetings.not_ongoing` (400).

## 5. Frontend

Aucun changement fonctionnel (l'UI restreint déjà). Vérifier seulement que le code d'erreur `errors.meetings.not_ongoing` a une traduction et s'affiche en toast si un cas de course survient (ex. AG clôturée dans un autre onglet pendant la saisie).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Vote sur meeting `draft` / `called` | 400 `errors.meetings.not_ongoing`. |
| Vote sur meeting `closed` | 400 `errors.meetings.not_ongoing` — figé définitivement. |
| Présence sur meeting `closed` | 400, idem. |
| Clôture concurrente pendant un vote en vol | Le `FOR UPDATE` sérialise : le vote passe avant la clôture ou est refusé après — jamais d'écriture post-clôture. |
| Meeting inexistant / autre immeuble | 404 (inchangé). |
| Interaction EP-003-S10 | Les validations correspondance (formulaire, caducité) s'appliquent **en plus** de ce guard, qui reste la première barrière. |

## 7. Critères d'acceptation

1. `POST` vote et `POST/PUT` attendance sur un meeting non-`ongoing` → 400 `errors.meetings.not_ongoing`, aucune ligne écrite.
2. Le check est transactionnel (aucune fenêtre de course avec la clôture).
3. Comportement en `ongoing` strictement inchangé (non-régression).
4. Guard exprimé dans `domain/immutability.rs` avec test unitaire.

## 8. Plan de tests

- **Unitaires** (`domain/immutability.rs`) : `MeetingSession` mutable uniquement en `ongoing`.
- **E2E** (`backend/tests/api_e2e_test.rs`) :
  1. meeting `draft` : cast_vote → 400 ; save_attendance → 400 ;
  2. meeting `called` : idem → 400 ;
  3. meeting `ongoing` : les deux → 200 (non-régression) ;
  4. clôturer le meeting puis cast_vote → 400 ; save_attendance → 400 ; les décomptes existants sont inchangés ;
  5. vérifier le code d'erreur exact `errors.meetings.not_ongoing` dans le corps.

## 9. Ordre d'implémentation

1. `domain/immutability.rs` : variante `MeetingSession` + test → `cargo test --lib`.
2. `db/assembly.rs` : check transactionnel dans `cast_vote` puis `save_attendance` → `cargo build`.
3. Tests e2e (called/closed/draft), boucler jusqu'au vert (`cargo test --lib --tests`, `cargo clippy -- -D warnings`).
4. Frontend : traduction du code d'erreur (toast), rien d'autre.
5. `docs/PROGRESS.md` (nouvelle story EP-013-S11 référencée).
