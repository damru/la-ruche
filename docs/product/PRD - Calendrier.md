---
product: Calendrier copropriété
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references: []
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Calendrier copropriété » de La Ruche.
    Le calendrier est une vue d'agrégation pour les événements issus d'autres
    modules, et un module de création pour les événements manuels (entité Event).
    Ne pas implémenter : gestion d'invitations, synchronisation calendrier
    externe, rappels push depuis le calendrier.
---

# PRD — Calendrier copropriété

## 1. Vue d'ensemble

### Objectif

Offrir une **vue calendaire unifiée** de tous les événements datés de la copropriété — qu'ils soient issus d'autres modules (AG, contrats, budget…) ou créés manuellement — en une seule interface filtrée selon le rôle de l'utilisateur.

### Pourquoi

- Les événements clés sont aujourd'hui éparpillés dans différents modules — un syndic bénévole doit naviguer partout pour anticiper la charge à venir.
- Une vue calendaire unifiée réduit le risque d'oubli d'une échéance contractuelle ou d'un délai légal.
- Les événements ponctuels de la vie de la copropriété (réunions conseil, visites techniques, permanences) n'ont pas de module d'origine — ils nécessitent un espace de création dédié.

### KPIs

- Délai d'identification d'une échéance entrante (prochains 30 jours) : < 10 secondes depuis la page d'accueil.

---

## 2. Contraintes techniques

### Agrégation + création

- Le calendrier combine deux sources : événements **agrégés** (lecture seule depuis modules tiers) et événements **manuels** (entité `Event` gérée depuis ce module).
- Les événements agrégés ne peuvent pas être créés, modifiés ou supprimés depuis le calendrier — toute modification s'effectue dans le module source.
- Les événements manuels (`Event`) sont créés, modifiés et supprimés depuis le calendrier par Syndic et Conseil.

### Sources d'événements agrégés

| Source | Types d'événements | Filtre par rôle |
|---|---|---|
| Assemblée Générale | Date de séance ; délai notification PV (J+30) ; délai de contestation (J+60 depuis notification) | Tous (date séance) ; Syndic + Conseil (délais PV et contestation) |
| Contrats fournisseurs | Rappel reconduction tacite ; rappel préavis ; rappel échéance (`end_date`) | Syndic + Conseil |
| Tâches | Date de clôture (`closed_at`) | Syndic + Conseil |
| Budget | Dates des appels de fonds (`due_date`) ; arrêté de comptes | Syndic + Conseil + Copropriétaire (ses appels) |
| Sinistres | Date limite déclaration assureur ; `transmitted_at` | Syndic + Conseil |
| Carnet d'entretien | Dates d'interventions clôturées (TRAVAUX, MAINTENANCE, CONTRAT, SINISTRE) | Syndic + Conseil |

### Export

- Export ICS (format standard) pour import dans Google Calendar, Outlook, Apple Calendar.
- L'export est filtré selon le rôle de l'utilisateur.
- Les événements à échéance connue (rappels contrats, délais légaux sinistres, délai notification PV AG) incluent un composant `VALARM` pré-configuré.

---

## 3. Entité `Event` (événements manuels)

### Champs

| Champ | Type | Obligatoire | Notes |
|---|---|---|---|
| `title` | Texte libre | Oui | |
| `date` | Date (+ heure optionnelle) | Oui | |
| `description` | Texte libre | Non | |
| `category` | Enum optionnel | Non | Valeurs : REUNION, VISITE, PERMANENCE, AUTRE — extensible |
| Documents joints | Liaison GED | Non | Upload → créé dans GED, lié via `event_document` |

### Règles métier

- Un `Event` est visible par tous les rôles (Syndic, Conseil, Copropriétaire, Locataire) en lecture.
- Seuls Syndic et Conseil peuvent créer, modifier et supprimer un `Event`.
- La catégorie est optionnelle — un événement sans catégorie est valide.
- Tout document joint est systématiquement enregistré dans la GED (`document`) et lié via `event_document` (règle transverse — cf. `ARCHITECTURE.md`).

---

## 4. Profils utilisateurs

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Calendrier agrégé | `R` | `R` | `R¹` | `R²` |
| Export ICS | `R` | `R` | `R¹` | `R²` |
| Événements manuels (`Event`) | `CRUD` | `CRUD` | `R` | `R` |

> ¹ Le copropriétaire ne voit que les événements qui le concernent (AG, ses propres appels de fonds) + tous les `Event`.
> ² Le locataire ne voit que la date de séance AG (convocation) + tous les `Event`.

### Règles de sécurité

- Les événements liés aux contrats (montants, fournisseurs) ne sont pas visibles par les copropriétaires ni les locataires depuis le calendrier.
- Filtrage systématique par `building_id`.

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-CAL-001 | Afficher la vue calendaire | Vue mensuelle, hebdomadaire et liste chronologique |
| Must | US-CAL-002 | Filtrer par type d'événement | Sélection des sources à afficher |
| Must | US-CAL-005 | Créer / modifier / supprimer un événement manuel | Entité `Event` avec catégorie optionnelle et documents joints |
| Should | US-CAL-003 | Exporter en ICS | Import dans calendriers externes avec VALARM |
| Could | US-CAL-004 | Vue agenda (prochains 30 j) | Liste linéaire des échéances entrantes |

### User stories détaillées

#### US-CAL-001 — Afficher la vue calendaire

**En tant que** utilisateur autorisé,
**je veux** voir les événements de la copropriété dans une vue calendaire,
**afin de** anticiper les échéances et organiser mon temps.

**Critères d'acceptation :**

- **AC-CAL-001-A** : La vue DOIT proposer trois modes : mensuel (grille), hebdomadaire (grille) et liste chronologique.
- **AC-CAL-001-B** : Chaque événement DOIT afficher : titre, date, type (badge coloré par source) et un lien vers le module source (ou le détail `Event` pour les événements manuels).
- **AC-CAL-001-C** : Les événements sont filtrés selon le rôle de l'utilisateur — un copropriétaire ne voit pas les échéances contractuelles ni les délais légaux ; un locataire ne voit que la date de séance AG et les `Event`.

#### US-CAL-002 — Filtrer par type d'événement

**En tant que** utilisateur autorisé,
**je veux** filtrer les événements affichés par type,
**afin de** ne voir que ce qui me concerne à un instant donné.

**Critères d'acceptation :**

- **AC-CAL-002-A** : Filtres disponibles (selon rôle) : AG, Contrats, Tâches, Budget, Sinistres, Carnet d'entretien, Événements de copropriété.
- **AC-CAL-002-B** : L'état des filtres DOIT être persisté dans les préférences utilisateur.

#### US-CAL-003 — Exporter en ICS

**En tant que** utilisateur autorisé,
**je veux** exporter les événements du calendrier au format ICS,
**afin de** les importer dans mon calendrier personnel (Google, Outlook, Apple).

**Critères d'acceptation :**

- **AC-CAL-003-A** : L'export DOIT respecter les filtres de rôle appliqués à la vue.
- **AC-CAL-003-B** : Les événements à échéance connue (rappels contrats, délais légaux sinistres, délai notification PV AG) DOIVENT inclure un composant `VALARM` avec alerte 24 h avant.
- **AC-CAL-003-C** : Chaque événement exporté DOIT inclure un lien `URL` vers le module source (ou le détail `Event`) dans la description.

#### US-CAL-005 — Créer / modifier / supprimer un événement manuel

**En tant que** Syndic ou membre du Conseil,
**je veux** créer des événements manuels dans le calendrier,
**afin de** centraliser les rendez-vous et réunions ponctuels de la copropriété.

**Critères d'acceptation :**

- **AC-CAL-005-A** : La création DOIT exiger au minimum un titre et une date.
- **AC-CAL-005-B** : La catégorie est optionnelle — valeurs : REUNION, VISITE, PERMANENCE, AUTRE.
- **AC-CAL-005-C** : Des documents peuvent être joints — tout upload est enregistré dans la GED et lié via `event_document`.
- **AC-CAL-005-D** : Les événements manuels sont visibles en lecture par tous les rôles.
- **AC-CAL-005-E** : Syndic et Conseil peuvent modifier et supprimer tout `Event` de leur copropriété.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Perf-001 | Performance | Chargement de la vue calendaire | ≤ 300 ms (p95) |
| NFR-Sec-001 | Sécurité | Filtrage par rôle | 100 % des événements filtrés |

---

## 7. Hors scope

- Synchronisation bidirectionnelle avec des calendriers externes (Google, Outlook).
- Rappels push déclenchés depuis le calendrier (les rappels sont gérés dans chaque module source).
- Planification de réunions ou invitations (gestion des participants, RSVP).

---

## 8. Questions ouvertes

Toutes les questions ouvertes initiales (Q-CAL-001 à Q-CAL-003) ont été tranchées lors de l'analyse de gap du 22 mai 2026 — voir décision log.

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-CAL-001 | Must | AC-CAL-001-A à C | Calendrier (R — agrégation) |
| US-CAL-002 | Must | AC-CAL-002-A à B | Filtres (R) |
| US-CAL-003 | Should | AC-CAL-003-A à C | Export ICS |
| US-CAL-004 | Could | — | Vue agenda 30 j |
| US-CAL-005 | Must | AC-CAL-005-A à E | Event (CRUD Syndic+Conseil / R Copro+Locataire) |
