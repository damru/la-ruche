# ADR-013 — Journal d'événements centralisé : une capture, trois projections

**Date :** 1er novembre 2026  
**Statut :** Accepté (révisé le 1er novembre 2026 — suppression du champ de classe)  
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

### §2 — Catalogue fermé sur le couple (entité, action), sans champ de classe

Un événement est identifié par le couple fermé (`EntityType`, `Action`) — deux enums exhaustifs. La table de routage du catalogue associe à chaque couple la liste de ses sinks : c'est la source unique de vérité. Les familles « probant / cycle de vie / réglementaire » ne sont pas un champ de l'événement : elles se lisent depuis la table de routage. Aucun couple non déclaré ne peut être émis (erreur de compilation ou de test d'exhaustivité — jamais de silence).

Un événement porte le sur-ensemble de champs suivant, indépendant du stockage : `event_id`, `occurred_at`, `building_id`, `actor`, `entity_type`, `entity_id`, `action`, `payload` (JSONB), `correlation_id` (transaction/requête), `source`.

### §3 — Les journaux sont des projections ; le régime est une propriété du sink

Le dispatcher route chaque couple (entité, action) vers des sinks enregistrés :

- `AuditSink` → `audit_log` — régime strict : échec d'insertion = échec de la mutation (même transaction) ;
- `MaintenanceSink` → `maintenance_log` (carnet d'entretien) — régime strict : projection créée dans la transaction de la mutation (phase 2) ;
- `ActivitySink` → `activity_log` — régime lenient : best-effort (l'échec ne casse pas la mutation).

Les tables restent distinctes ; leurs régimes d'écriture, de rétention, de RGPD et de droits d'accès restent les leurs. Un couple peut router vers plusieurs sinks (ex. (`task`, `closed`) : ActivitySink + MaintenanceSink) avec un seul point d'appel ; le régime applicable est alors le plus strict.

### §4 — Extension réservée

Une future table `event_log` « tous événements » — si le besoin apparaît — n'est qu'un sink supplémentaire : aucun site émetteur n'est modifié. Réciproquement, tout nouveau journal se branche sans toucher aux modules métier. Changer le routage d'un couple (ajouter/retirer un sink) se fait dans la seule table de routage.

### §5 — Invariants ADR-009 préservés

L'audit reste dans la transaction de la mutation (rollback = disparition de la trace), append-only, conservation 10 ans, `user_id` maintenu après anonymisation. Un événement dont le routage inclut un sink strict n'est émissible que via `emit(tx)` ; `emit_after_commit` le refuse. Le verrouillage est garanti par la visibilité : `db::audit::insert` et `db::activity::insert` deviennent inaccessibles hors de `domain/journal`.

## Alternatives rejetées

| Option | Raison du rejet |
|---|---|
| Table événementielle unique | Régimes d'écriture (transactionnel vs best-effort), rétention et RGPD divergents — cf. arbitrage EP-018-S01 |
| Champ de classe (`Probative`/`Lifecycle`/`Regulatory`) sur l'événement | Redondant : dérivable du couple (entité, action) et de la table de routage ; taxonomie supplémentaire à maintenir, risque d'incohérence champ ↔ routage |
| Crate `auditlog` (v0.1.0) | Pas d'écriture dans la transaction appelante, vocabulaire CRUD figé, plafond de rétention contraire à la conservation décennale |
| Triggers PostgreSQL | Logique probante invisible côté application, contournable, non testable unitairement |
| Event sourcing complet | Réécriture majeure disproportionnée à la volumétrie |
| Bus d'événements / outbox | Fan-out asynchrone inutile : les sinks vivent dans la même transaction |

## Conséquences

- backend : module `domain/journal` spécifié en EP-018-S08 ; EP-017-S02 et EP-021-S03 câblent exclusivement via `emit` ;
- l'alimentation automatique du carnet migre vers `MaintenanceSink` en phase 2, à comportement identique (ADR-008 : une entrée par transition non closed → closed) ;
- helpers partagés de payload : diff ciblé avant/après et masquage des valeurs sensibles ;
- documentation : section journal dans ARCHITECTURE.md à la livraison.
