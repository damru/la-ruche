# ADR-013 — Journal d'événements centralisé : une capture, trois projections

**Date :** 1er novembre 2026  
**Statut :** Accepté  
**Contexte :** EP-018-S01, EP-018-S08, EP-017-S01/S02, EP-021-S03

---

## Contexte

Trois journaux coexistent après l'arbitrage EP-018-S01 / ADR-009 :

- `audit_log` — piste probante comptable et juridique (append-only, transactionnelle, conservation 10 ans) ;
- `activity_log` — frise métier de cycle de vie signalements/tâches (best-effort, EP-017) ;
- carnet d'entretien (`maintenance_log`) — registre réglementaire (décret n° 2001-477), partiellement auto-alimenté à la clôture de tâche.

Le câblage exhaustif à venir (~15 sites pour EP-021-S03, plus l'émission EP-017-S02) risque de faire diverger vocabulaires, formats de payload et points d'écriture si chaque site construit ses entrées à la main. La table événementielle unique servant les trois journaux par requête a été écartée (régimes de garantie, rétention RGPD et contrôle d'accès incompatibles), mais l'architecture doit conserver la capacité de journaliser tous les événements applicatifs dans un flux unique servant de base aux journaux.

## Décision

### §1 — Un seul point de capture

Tous les événements applicatifs — mutations comptables, transitions de cycle de vie, événements à projection réglementaire — sont émis une fois, au même point transactionnel que la mutation, via `domain/journal::emit`. Aucune écriture directe dans les tables des journaux hors du module.

### §2 — Catalogue typé fermé

Chaque événement appartient à un catalogue d'`Action` typées (enums, snake_case stable, hors-vocabulaire impossible à la compilation) et déclare sa classe :

- `Probative` — mutation comptable (CREATE, UPDATE, DELETE, ISSUE, CANCEL, MARK_PAID, TRANSFER…) ;
- `Lifecycle` — événement de cycle de vie (created, status_changed, converted_to_task…) ;
- `Regulatory` — événement projeté dans le carnet d'entretien (task_closed, contract_activated, contract_terminated, claim_closed).

Un événement porte toujours le sur-ensemble de champs suivant, indépendant du stockage : `event_id`, `occurred_at`, `building_id`, `actor`, `entity_type`, `entity_id`, `action`, `payload` (JSONB), `correlation_id` (transaction/requête), `source`.

### §3 — Les journaux sont des projections

Le dispatcher route chaque événement vers des sinks enregistrés :

- `AuditSink` → `audit_log` : échec d'insertion = échec de la mutation (même transaction) ;
- `ActivitySink` → `activity_log` : best-effort (l'échec ne casse pas la mutation) ;
- `MaintenanceSink` → `maintenance_log` : projection carnet d'entretien (phase 2).

Les tables restent distinctes ; leurs régimes d'écriture, de rétention, de RGPD et de droits d'accès restent les leurs. Un événement peut alimenter plusieurs sinks (ex. `task_closed` : ActivitySink + MaintenanceSink) avec un seul point d'appel.

### §4 — Extension réservée

Une future table `event_log` « tous événements » — si le besoin apparaît — n'est qu'un sink supplémentaire : aucun site émetteur n'est modifié. Réciproquement, tout nouveau journal se branche sans toucher aux modules métier.

### §5 — Invariants ADR-009 préservés

L'audit reste dans la transaction de la mutation (rollback = disparition de la trace), append-only, conservation 10 ans, `user_id` maintenu après anonymisation. Le verrouillage est garanti par la visibilité : `db::audit::insert` et `db::activity::insert` deviennent inaccessibles hors de `domain/journal`.

## Alternatives rejetées

| Option | Raison du rejet |
|---|---|
| Table événementielle unique | Régimes d'écriture (transactionnel vs best-effort), rétention et RGPD divergents — cf. arbitrage EP-018-S01 |
| Crate `auditlog` (v0.1.0) | Pas d'écriture dans la transaction appelante, vocabulaire CRUD figé, plafond de rétention contraire à la conservation décennale |
| Triggers PostgreSQL | Logique probante invisible côté application, contournable, non testable unitairement |
| Event sourcing complet | Réécriture majeure disproportionnée à la volumétrie |
| Bus d'événements / outbox | Fan-out asynchrone inutile : les sinks vivent dans la même transaction |

## Conséquences

- backend : module `domain/journal` spécifié en EP-018-S08 ; EP-017-S02 et EP-021-S03 câblent exclusivement via `emit` ;
- l'alimentation automatique du carnet migre vers `MaintenanceSink` en phase 2, à comportement identique (ADR-008 : une entrée par transition non closed → closed) ;
- helpers partagés de payload : diff ciblé avant/après et masquage des valeurs sensibles ;
- documentation : section journal dans ARCHITECTURE.md à la livraison.
