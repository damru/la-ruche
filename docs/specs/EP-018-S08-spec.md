# Spécification technique — EP-018-S08

**Story :** Module `domain/journal` — capture unique de tous les événements applicatifs, dispatch vers les journaux existants (`audit_log`, `activity_log`, carnet d'entretien).

**Épic :** EP-018 — Immutabilité comptable & auditabilité probante  
**Réf. ADR :** ADR-013 (journal d'événements centralisé) · ADR-009 (immuabilité comptable)

> Statut : À FAIRE. Refactoring transverse, prérequis mécanique de EP-017-S02 (émission `activity_log`) et EP-021-S03 (câblage exhaustif `audit_log`). À livrer avant tout câblage massif.

---

## 1. État actuel du code

| Élément | Emplacement |
|---|---|
| Table `audit_log` append-only | migration `0029_audit_log.sql` |
| `db::audit::insert` (générique sur `Executor`, utilisable en tx) + `list_by_entity` | `backend/src/db/audit.rs` |
| 5 sites audit câblés | `db/budget.rs:642/727`, `db/contract.rs:297/366/389` |
| Diff ciblé before/after (Pattern C) | `domain/immutability` |
| `activity_log` | pas encore créée (EP-017-S01) |
| Alimentation carnet à la clôture de tâche | logique directe dans le flux de clôture (EP-009-S03) |

Problème : chaque site construit ses entrées à la main ; les vocabulaires (`operation`, `action`) et formats de payload divergeront au fil du câblage exhaustif.

## 2. Quoi

1. Un catalogue fermé sur le couple (`EntityType`, `Action`) — deux enums exhaustifs — dont la table de routage associe chaque couple à ses sinks. Pas de champ de classe : les journaux se reconstituent depuis le couple et la table de routage.
2. Un dispatcher `journal::emit(tx, event)` appelé au même point transactionnel que chaque mutation, routant vers les sinks enregistrés.
3. Sinks phase 1 : `AuditSink` (strict : échec = échec de la mutation) et `ActivitySink` (lenient : best-effort). Phase 2 : `MaintenanceSink` (strict, projection carnet).
4. Souplesse : le flux est la base des journaux ; une future table `event_log` globale serait un sink de plus, sans toucher aux sites d'émission ; changer le routage d'un couple ne modifie aucun site d'appel.
5. Aucun changement de schéma, aucun changement de comportement : lignes écrites identiques à l'existant.

## 3. Modèle de données

Aucun changement en phase 1. Le champ `Event` (§4.1) définit le sur-ensemble qu'une future table unifiée exigerait, sans la créer.

## 4. Backend

### 4.1 Modèle d'événement

```rust
pub struct Event {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub building_id: Uuid,
    pub actor: Option<Uuid>,          // user_account ; None = acteur système / prestataire via token
    pub entity_type: EntityType,     // enum fermée, ex. Task, LevyCall, VendorContract
    pub entity_id: Uuid,
    pub action: Action,              // enum fermée, cf. §4.2
    pub payload: serde_json::Value,   // old/new, champs saillants — jamais de token en clair
    pub correlation_id: Uuid,         // id de transaction ou de requête
    pub source: &'static str,        // module émetteur, ex. "db::budget"
}
```

### 4.2 Catalogue : deux enums fermées, une table de routage

```rust
pub enum EntityType { Budget, BudgetLine, LevyCall, Unit, VendorContract, Task, Report, Claim, Document, UserAccount, /* … */ }
pub enum Action {
    // vocabulaire probant (strings stables majuscules — identiques aux lignes existantes)
    Create, Update, Delete, Issue, Cancel, MarkPaid, Transfer,
    // vocabulaire cycle de vie (strings stables snake_case — identiques à EP-017-S02)
    Created, StatusChanged, PhotoAdded, ConvertedToTask, AssigneeChanged,
    ContractorAssigned, VendorLinkGenerated, DocumentUploaded,
    VendorSubmissionReceived, Closed,
    // vocabulaire réglementaire (projection carnet)
    ContractActivated, ContractTerminated, ClaimClosed,
}
```

- `as_str()` fige les strings stockés : majuscules pour les opérations probantes (CREATE, ISSUE…), snake_case pour le cycle de vie (`status_changed`…) — compatibilité byte-identique avec les lignes existantes et la spec EP-017-S02.
- Table de routage = exhaustive `match (entity_type, action) -> &[Sink]`, source unique de vérité :
  - (`Budget`…) → AuditSink ; (`LevyCall`, `Issue`) → AuditSink ; (`Report`, `StatusChanged`) → ActivitySink ;
  - (`Task`, `Closed`) → ActivitySink + MaintenanceSink (régime le plus strict applicable) ;
  - (`VendorContract`, `ContractTerminated`) → AuditSink + MaintenanceSink.
- Exhaustivité garantie : un test unitaire itère le produit cartésien `EntityType × Action` et exige que chaque couple soit soit déclaré dans la table, soit explicitement marqué `NotAudited` — un couple oublié fait échouer le test, il n'est jamais ignoré en silence.

### 4.3 API

```rust
journal::emit(&mut *tx, event).await?          // in-tx, tous sinks
journal::emit_after_commit(exec, event).await  // post-commit, best-effort — refuse tout événement routé vers un sink strict
```

- Un événement dont le routage inclut un sink strict (audit, maintenance) n'est émissible que via `emit(tx)` ; `emit_after_commit` le refuse (erreur explicite), jamais de best-effort sur une preuve.
- `emit_after_commit` (événements lenient uniquement) : échec loggé (warn), mutation non annulée, pas de retry.
- Fan-out multi-sinks : la table de routage renvoie une liste ordonnée de sinks ; le dispatcher les exécute séquentiellement dans la même transaction, un seul point d'appel côté site émetteur.
- Au sein de `emit(tx)`, chaque sink lenient s'exécute sous `SAVEPOINT` : en cas d'échec, `ROLLBACK TO SAVEPOINT` + warn, puis poursuite — l'invariant best-effort tient aussi en transaction. Un sink strict échoué annule toute la transaction : toutes les projections disparaissent, tout ou rien.

### 4.4 Sinks phase 1 et verrouillage

- `AuditSink` encapsule `db::audit::insert` ; `ActivitySink` encapsule `db::activity::insert` (créée par EP-017-S02).
- Les 5 sites audit existants migrent vers `journal::emit` — lignes produites byte-identiques.
- `db::audit::insert` / `db::activity::insert` deviennent réservées au module journal (visibilité privée ou déplacement) : le compilateur garantit le point d'appel unique.

### 4.5 Sink carnet (phase 2, follow-up dédié)

- `MaintenanceSink` : (`Task`, `Closed`) → entrée `maintenance_log` (remplace l'appel direct actuel), (`VendorContract`, `ContractActivated`/`ContractTerminated`), (`Claim`, `ClaimClosed`) → entrées correspondantes.
- Comportement strictement identique (ADR-008 : snapshot du coût, `vendor_name`, `task_id` `ON DELETE SET NULL`, une entrée par transition non closed → closed).
- Livré après validation du pattern sur les deux premiers sinks, pour ne pas régresser l'existant.

### 4.6 Non-objectifs

Table unifiée (cf. ADR-013 §4 : extension réservée), bus d'événements, outbox, triggers PG, attribution d'acteur par task-local (l'acteur reste un paramètre explicite), event sourcing complet, champ de classe sur l'événement (dérivable — cf. ADR-013 « Alternatives rejetées »).

## 5. Frontend

Aucun changement (refactoring backend pur). Les UI d'EP-021-S03 et EP-017-S04/S05 consomment les mêmes réponses.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Rollback de la mutation | Lignes audit et activity (in-tx) disparaissent ; aucune ligne orpheline. |
| Échec insert audit (sink strict) | Mutation annulée — invariant ADR-009 inchangé. |
| Échec insert activity post-commit | Warn loggé, mutation conservée, pas de retry. |
| Échec d'un sink lenient en transaction | Rollback au savepoint, warn ; mutation et autres projections (carnet) conservées. |
| Couple (entité, action) non déclaré | Erreur au test d'exhaustivité — jamais ignoré en silence. |
| Événement routé vers un sink strict via `emit_after_commit` | Refusé (erreur explicite). |
| Événement multi-journaux (ex. `Task + Closed`) | Un seul emit, plusieurs sinks, un seul point d'appel. |
| Payload sensible | Passage par le helper `redact()` ; jamais de token prestataire en clair. |
| Anonymisation RGPD | Les lignes audit conservent `actor` (obligation probante > effacement). |

## 7. Critères d'acceptation

1. Les 5 sites audit migrés produisent des lignes byte-identiques (comparaison e2e avant/après).
2. `db::audit::insert` / `db::activity::insert` inutilisables hors `domain/journal` (erreur de compilation).
3. `emit_after_commit` refuse tout événement routé vers un sink strict.
4. Un `MemorySink` de test s'enregistre et reçoit les événements sans modifier les sites d'émission — preuve d'extensibilité (futur `event_log`).
5. Le test du produit cartésien (`EntityType × Action`) passe : tout couple est routé ou explicitement `NotAudited`.
6. Changer le routage d'un couple (ajout/retrait d'un sink) ne modifie aucun site d'émission.
7. Clippy/tests/build verts ; aucune régression e2e.

## 8. Plan de tests

- E2E : rejouer les scénarios EP-021-S03 §8 sur les sites migrés ; échec simulé du sink audit → mutation absente ; échec activity post-commit → mutation présente + warn ; échec activity simulé in-tx (savepoint) → mutation et ligne carnet présentes, ligne activity absente.
- Unitaires : table de routage (couples attendus → sinks), produit cartésien exhaustif, helper `diff` (ajout/modif/suppression), `redact`, refus des événements strict-routés via `emit_after_commit`.
- Compilation : tentative d'appel direct `db::audit::insert` hors module → échec du build.

## 9. Ordre d'implémentation

1. Catalogue `EntityType`/`Action` + `Event` + table de routage + dispatcher + `MemorySink` (tests).
2. `AuditSink` + migration des 5 sites + verrouillage visibilité (commit unique).
3. `ActivitySink` branché dès EP-017-S02 — le dispatcher devient son unique point d'écriture ; EP-021-S03 câble ses nouveaux sites via `emit` uniquement.
4. `MaintenanceSink` en phase 2 (follow-up dédié).
5. `docs/ARCHITECTURE.md` (section journal) + `docs/PROGRESS.md`.
