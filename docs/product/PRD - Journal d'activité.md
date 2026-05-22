---
product: Journal d'activité et suivi de cycle de vie
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - ADR-009
llm_directives:
    temperature: 0.2
    persona: >
      Tu travailles sur la feature « Journal d'activité » de La Ruche.
      Ce journal est une TIMELINE OPÉRATIONNELLE traçant les transitions
      de cycle de vie des signalements et des tâches uniquement.
      Il est DISTINCT du journal d'audit probant (audit_log backend).
      Périmètre strict : signalements + tâches. Hors périmètre : AG, Budget,
      Contrats, Sinistres, Propriétaires. Les entrées sont immuables
      (append-only). Ne pas implémenter : fil d'actualité social, commentaires,
      consultation du journal d'audit probant.
---

# PRD — Journal d'activité et suivi de cycle de vie

## 1. Vue d'ensemble

### Objectif

Offrir une **timeline opérationnelle** traçant les transitions de cycle de vie des **signalements** et des **tâches**, permettant au Syndic, au Conseil et aux résidents de suivre l'avancement des demandes et interventions en temps réel. Distincte du journal d'audit probant (technique, backend).

### Pourquoi

- Le Syndic gère simultanément de nombreux signalements et tâches — sans vue unifiée, le suivi se fait par e-mail ou de mémoire.
- Un résident dépose un signalement et n'a aucune visibilité sur son avancement — source principale de friction.
- Le Conseil a besoin d'une vue synthétique des interventions en cours pour exercer son rôle de contrôle sans accéder à chaque fiche individuellement.

### KPIs

- Délai d'indexation d'une transition dans le journal après l'événement source : ≤ 5 s.
- Taux de résidents ayant pu retracer l'avancement de leur signalement sans contacter le Syndic directement : ≥ 75 % (mesure UX).

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables.

### Append-only

- Les entrées du journal d'activité sont **append-only** — aucune modification ni suppression après création.
- Elles sont générées par les modules via des événements de domaine (pas de saisie manuelle).

### Distinction audit_log / journal d'activité

| Critère | audit_log (backend) | Journal d'activité (cette feature) |
|---|---|---|
| Public | Technique (intégration, conformité) | Utilisateurs finaux |
| Granularité | Tous les champs modifiés (JSONB) | Événements métier significatifs seulement |
| Accès | Jamais exposé directement aux utilisateurs | Affiché dans l'interface |
| Format | Structuré (old_values / new_values) | Phrase lisible en langue naturelle |

### Filtrage par rôle

- Chaque événement porte un niveau de visibilité (`PUBLIC`, `CONSEIL`, `SYNDIC`).
- Un copropriétaire ne voit que les événements `PUBLIC` concernant sa copropriété.
- Un utilisateur ne voit jamais les événements d'une autre copropriété.
- **Règle déclarant** : les transitions d'un signalement (`ACKNOWLEDGED`, `IN_PROGRESS`, `CLOSED`) sont toujours visibles par son créateur (`declarant_id`), même si leur visibilité nominale est `SYNDIC`. Cette règle s'applique aux copropriétaires et aux locataires.

---

## 3. Profils utilisateurs

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Journal d'activité | `R` | `R²` | `R¹` | `R³` |

> ¹ Copropriétaire : événements `PUBLIC` + transitions de ses propres signalements (règle déclarant §2).
> ² Conseil : événements `PUBLIC` et `CONSEIL`.
> ³ Locataire : uniquement les transitions de ses propres signalements (règle déclarant §2).
> Syndic : tous les événements (`PUBLIC`, `CONSEIL`, `SYNDIC`).

### Règles de sécurité

- Filtrage systématique par `building_id` — jamais d'événements cross-copropriété.
- La règle déclarant (§2) DOIT être évaluée côté backend à chaque requête : une entrée `SYNDIC` sur un signalement est retournée au copropriétaire ou locataire si et seulement si `declarant_id = current_user_id`.
- Les données personnelles (noms de témoins, numéros de lot) ne DOIVENT PAS apparaître dans les messages de visibilité `PUBLIC`.

---

## 4. Événements indexés

### Périmètre

Seules les transitions de cycle de vie des **signalements** et des **tâches** sont indexées. Les autres modules (AG, Budget, Contrats, Sinistres, Propriétaires) sont hors périmètre de ce journal.

### Schéma d'une entrée de journal

| Champ | Type | Description |
|---|---|---|
| `id` | UUID | Identifiant unique |
| `building_id` | UUID | Copropriété concernée |
| `entity_type` | enum | `SIGNALEMENT` \| `TASK` |
| `entity_id` | UUID | Identifiant de l'entité source |
| `event_type` | enum | Type de transition (voir table ci-dessous) |
| `actor_id` | UUID? | Utilisateur déclencheur — `NULL` si événement système |
| `message` | string | Message lisible stocké en snapshot immuable |
| `visibility` | enum | `PUBLIC` \| `CONSEIL` \| `SYNDIC` |
| `created_at` | timestamp | Horodatage de l'événement |

> Le message est stocké en clair au moment de l'événement (snapshot immuable) — il n'est pas reconstruit dynamiquement. La lisibilité est garantie même si le libellé de l'entité source évolue.

### Table des événements

| `entity_type` | `event_type` | `visibility` | Exemple de message |
|---|---|---|---|
| `SIGNALEMENT` | `CREATED` | `SYNDIC` | « Signalement créé — Éclairage défectueux hall » |
| `SIGNALEMENT` | `ACKNOWLEDGED` | `SYNDIC` + déclarant¹ | « Signalement pris en charge » |
| `SIGNALEMENT` | `IN_PROGRESS` | `SYNDIC` + déclarant¹ | « Traitement en cours » |
| `SIGNALEMENT` | `CLOSED` | `SYNDIC` + déclarant¹ | « Signalement clôturé » |
| `SIGNALEMENT` | `CONVERTED` | `SYNDIC` | « Converti en tâche » |
| `TASK` | `CREATED` | `CONSEIL` | « Tâche créée — Réfection toiture » |
| `TASK` | `ASSIGNED` | `CONSEIL` | « Tâche assignée — Otis » |
| `TASK` | `CLOSED` | `PUBLIC` | « Tâche clôturée — Réfection toiture » |

> ¹ **Règle déclarant** : définie au §2. S'applique aux copropriétaires et locataires créateurs du signalement.

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-JA-001 | Afficher le journal d'activité | Timeline chronologique filtrée par rôle |
| Must | US-JA-002 | Filtrer par type et période | Sélection du type d'entité et plage de dates |
| Should | US-JA-003 | Voir le détail d'un événement | Lien vers l'entité source dans le module |

### User stories détaillées

#### US-JA-001 — Afficher le journal d'activité

**En tant que** utilisateur autorisé,
**je veux** consulter le journal d'activité de ma copropriété,
**afin de** comprendre les événements récents sans devoir naviguer dans chaque module.

**Critères d'acceptation :**

- **AC-JA-001-A** : La timeline DOIT être triée par date décroissante, avec pagination.
- **AC-JA-001-B** : Chaque entrée DOIT afficher : date, type d'entité (badge `SIGNALEMENT` ou `TÂCHE`), message lisible, auteur (nom de l'utilisateur si `actor_id` non `NULL` — libellé « Système » si `actor_id = NULL`).
- **AC-JA-001-C** : Les entrées sont filtrées selon le niveau de visibilité et le rôle de l'utilisateur.

#### US-JA-002 — Filtrer par type et période

**En tant que** utilisateur autorisé,
**je veux** filtrer le journal par type d'entité et par plage de dates,
**afin de** trouver rapidement un événement spécifique.

**Critères d'acceptation :**

- **AC-JA-002-A** : Filtres disponibles : type d'entité (`SIGNALEMENT` / `TÂCHE`), statut courant de l'entité source (tous les statuts possibles de chaque type), plage de dates.
- **AC-JA-002-B** : La combinaison de filtres DOIT être possible simultanément.

#### US-JA-003 — Voir le détail d'un événement

**En tant que** utilisateur autorisé,
**je veux** accéder directement à l'entité source depuis une entrée du journal,
**afin de** obtenir plus de contexte sans chercher dans le module concerné.

**Critères d'acceptation :**

- **AC-JA-003-A** : Chaque entrée DOIT contenir un lien profond vers l'entité source (vers la fiche du signalement ou de la tâche concernée).
- **AC-JA-003-B** : Si l'utilisateur n'a pas accès à l'entité source (ex. : locataire → tâche, visibilité `CONSEIL`), le lien n'est pas affiché.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Perf-001 | Performance | Chargement du journal (100 entrées) | ≤ 200 ms (p95) |
| NFR-Perf-002 | Performance | Indexation d'un événement après génération | ≤ 5 s |
| NFR-Sec-001 | Sécurité | Filtrage par rôle et `building_id` | 0 fuite cross-rôle ou cross-copropriété |

---

## 7. Hors scope

- Exposition du journal d'audit probant (`audit_log`) aux utilisateurs finaux.
- Commentaires ou réactions sur les entrées du journal.
- Fil d'actualité social (likes, partages).
- Notifications push déclenchées depuis le journal (les notifications sont gérées dans chaque module source).
- Journal multi-copropriété pour les syndics gérant plusieurs immeubles.
- **Événements hors périmètre** (non indexés dans ce journal) :
  - AG : convocation, clôture, résolutions votées.
  - Budget : appel de fonds émis, budget approuvé.
  - Contrats : activation, résiliation, rappels préavis.
  - Sinistres : déclaration, transmission, clôture.
  - Propriétaires : mutations, transferts.

---

## 8. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-JA-001 | ~~Les locataires ont-ils accès au journal ?~~ | Damien | ✅ Résolu — oui, via la règle déclarant : ils voient les transitions de leurs propres signalements uniquement. |
| Q-JA-002 | ~~Les événements cron/automatiques apparaissent-ils ?~~ | Damien | ✅ Résolu — non. Seules les transitions déclenchées par des actions utilisateur ou des événements de domaine sont indexées. |
| Q-JA-003 | Faut-il une rétention limitée du journal pour éviter une croissance infinie ? | Damien | Lot futur |
| Q-JA-004 | ~~Quand un signalement est converti en tâche, les transitions ultérieures de la tâche s'affichent-elles dans le fil du signalement d'origine ?~~ | Damien | ✅ Résolu — oui, chaînage. Les transitions de la tâche issue d'un signalement converti sont visibles dans le fil du signalement d'origine. |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-JA-001 | Must | AC-JA-001-A à C | Journal (R) |
| US-JA-002 | Must | AC-JA-002-A à B | Journal (R — filtres) |
| US-JA-003 | Should | AC-JA-003-A à B | Lien profond (R) |
