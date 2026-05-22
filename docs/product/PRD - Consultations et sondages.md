---
product: Consultations et sondages
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - LEGAL.md §4
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Consultations et sondages » de La Ruche.
    Cette feature est NON décisionnelle — ses résultats n'ont aucune valeur
    juridique et ne remplacent pas un vote en AG. Ne jamais implémenter
    de lien avec les résolutions ou les votes AG.
    Les réponses des participants sont immuables une fois soumises.
    Ne pas implémenter : vote électronique probant, signature des réponses.
  modification_policy: >
    Ne modifier ce fichier que sur validation explicite après analyse de gap.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Consultations et sondages

## 1. Vue d'ensemble

### Objectif

Offrir un espace de **participation non décisionnel** permettant au syndic ou au Conseil de recueillir l'avis des résidents sur un sujet avant une décision formelle, ou simplement pour animer la vie de la copropriété.

### Pourquoi

- Les syndics bénévoles manquent d'outils simples pour sonder les copropriétaires entre deux AG.
- Recueillir les préférences en amont d'une AG permet de calibrer les résolutions et d'anticiper les votes.
- **Cette feature ne produit aucun effet juridique** — elle ne remplace ni ne conditionne aucune décision légale.

### KPIs

- Taux de participation moyen aux consultations ouvertes ≥ 50 %.
- Délai moyen d'ouverture d'une consultation : < 2 min.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables.

### Non-décisionnel

- Les résultats d'une consultation ou d'un sondage n'ont **aucun effet sur les données légales** (budget, résolutions, appels de fonds).
- L'interface DOIT afficher un avertissement explicite : *« Cette consultation est informelle et n'a pas de valeur légale. »*

### Immuabilité des réponses

- En **mode anonyme** : une réponse soumise est immuable — aucune modification ni suppression possible.
- En **mode nominatif** : un participant peut modifier sa réponse tant que la consultation est au statut `open`. La modification écrase la réponse précédente.

### Anonymat

**Mode anonyme** : une consultation en mode anonyme ne révèle pas l'identité des répondants, même au Syndic. Seul le décompte global de réponses est visible. Ce mode est adapté aux sujets sensibles où l'exposition de l'identité du votant pourrait inhiber la participation (ex. : satisfaction de la gestion).

- Si la consultation est configurée en mode anonyme, `respondent_id` DOIT être masqué de toute API et vue, y compris pour le Syndic.
- Le compte de participants ayant répondu est disponible même en mode anonyme.
- **En mode anonyme, la relance des non-répondants (US-CS-005) DOIT être désactivée** — identifier qui n'a pas répondu constituerait une fuite d'information sur la participation individuelle.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal |
|---|---|---|
| Persona-Syndic | Syndic bénévole | Créer et gérer les consultations |
| Persona-Conseil | Conseil syndical | Créer des consultations, consulter les résultats |
| Persona-Copro | Copropriétaire | Participer et consulter les résultats |
| Persona-Locataire | Locataire | Participer si l'audience l'inclut |

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Consultations / sondages | `C / R / U¹ / D¹` | `C / R / U¹ / D¹` | `R` | `R²` |
| Réponses | `R³` | `R³` | `C / R¹` | `C / R¹` |

> ¹ Uniquement au statut `draft`. La modification et la suppression sont réservées au groupe du créateur : une consultation créée par le Syndic ne peut être modifiée ou supprimée que par un Syndic ; une consultation créée par le Conseil, que par un membre du Conseil.
> ² Lecture et participation uniquement si l'audience de la consultation inclut les locataires.
> ³ En mode anonyme, les réponses individuelles ne sont pas lisibles — seuls les agrégats sont disponibles.

### Règles de sécurité

- En mode anonyme, aucun rôle (y compris Syndic) ne peut associer une réponse à son auteur.
- Filtrage par `building_id` — aucune consultation visible cross-copropriété.

---

## 4. Cycle de vie

```
draft ──(ouvrir)──▶ open ──(fermer / date fin)──▶ closed
```

| État | Description |
|---|---|
| `draft` | Consultation en cours de configuration — non visible des participants |
| `open` | Ouverte aux participants — réponses acceptées |
| `closed` | Clôturée (manuellement ou automatiquement à `date_fin`) — résultats définitifs, réponses figées |

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-CS-001 | Créer une consultation ou un sondage | Titre, type, question unique, audience, dates |
| Must | US-CS-001b | Ouvrir une consultation | Transition draft → open + notification participants |
| Must | US-CS-002 | Participer (répondre) | Soumettre une réponse selon le type |
| Must | US-CS-003 | Consulter les résultats | Agrégats en temps réel + final |
| Should | US-CS-004 | Relancer les non-répondants | Notification ciblée aux participants sans réponse (désactivée en mode anonyme) |

### User stories détaillées

#### US-CS-001 — Créer une consultation ou un sondage

**En tant que** `Persona-Syndic` ou `Persona-Conseil`,
**je veux** créer une consultation ou un sondage,
**afin de** recueillir l'avis des résidents sur un sujet.

**Critères d'acceptation :**

- **AC-CS-001-A** : Types disponibles : `CONSULTATION` (une question à réponse binaire : oui / non / abstention) et `SONDAGE` (une question avec une liste de réponses possibles définies par le créateur, à choix unique ou multiple).
- **AC-CS-001-B** : Champs obligatoires : titre, type, date d'ouverture, date de clôture, audience (`COPROPRIETAIRES`, `TOUS_RESIDENTS`).
- **AC-CS-001-C** : En mode anonyme, les réponses sont immuables. En mode nominatif, la modification est autorisée tant que la consultation est ouverte.
- **AC-CS-001-D** : La consultation est créée au statut `draft` — non visible par les participants.
- **AC-CS-001-E** : L'interface DOIT afficher un avertissement : *« Cette consultation est informelle et n'a pas de valeur légale. »*
- **AC-CS-001-F** : Si une `date_fin` est définie, la consultation DOIT passer automatiquement au statut `closed` à cette date — toute nouvelle réponse DOIT être refusée après ce délai.

#### US-CS-001b — Ouvrir une consultation

**En tant que** `Persona-Syndic` ou `Persona-Conseil` (créateur de la consultation),
**je veux** ouvrir une consultation au statut `draft`,
**afin de** la rendre accessible aux participants et déclencher les notifications.

**Critères d'acceptation :**

- **AC-CS-001b-A** : L'ouverture DOIT passer la consultation du statut `draft` au statut `open` via un endpoint dédié.
- **AC-CS-001b-B** : Seul le groupe du créateur (Syndic ou Conseil) peut ouvrir la consultation.
- **AC-CS-001b-C** : À l'ouverture, tous les participants éligibles (selon l'audience définie) DOIVENT être notifiés de la disponibilité de la consultation.
- **AC-CS-001b-D** : Une consultation `open` ne peut pas repasser au statut `draft`.

#### US-CS-002 — Participer (répondre)

**En tant que** participant autorisé (selon l'audience),
**je veux** soumettre ma réponse à une consultation ouverte,
**afin de** exprimer mon avis.

**Critères d'acceptation :**

- **AC-CS-002-A** : La participation DOIT être impossible si la consultation est au statut `draft` ou `closed`.
- **AC-CS-002-B** : En mode nominatif, un participant peut modifier sa réponse tant que la consultation est ouverte. En mode anonyme, aucune modification n'est possible après soumission.
- **AC-CS-002-C** : Les locataires peuvent répondre uniquement si l'audience est `TOUS_RESIDENTS`.

#### US-CS-003 — Consulter les résultats

**En tant que** tout utilisateur autorisé,
**je veux** voir les résultats de la consultation,
**afin de** connaître la distribution des réponses.

**Critères d'acceptation :**

- **AC-CS-003-A** : Les résultats DOIVENT être affichés en temps réel pendant l'ouverture et de façon définitive après clôture.
- **AC-CS-003-B** : Les résultats affichent le nombre de répondants, le total de participants éligibles et la distribution des réponses (en valeur et pourcentage).
- **AC-CS-003-C** : En mode anonyme, aucune association réponse/participant n'est exposée dans les résultats.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Legal-001 | Conformité | Avertissement non-décisionnel | Affiché dans 100 % des vues de consultation |
| NFR-Sec-001 | Anonymat | Identité masquée en mode anonyme | 0 fuite respondent_id |

---

## 7. Hors scope

- Vote électronique probant (valeur juridique pour les AG).
- Signature des réponses par certificat.
- Consultation multi-immeuble (réseau de syndics).
- Résultats publics hors de l'Application (publication externe).
- Délibérations en ligne.
- Pondération des réponses par tantièmes.

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Taux de participation moyen | ≥ 50 % | Continu |
| Temps de création d'une consultation | < 2 min | À mesure UX |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-CS-001 | ~~Le Conseil peut-il créer des consultations sans validation du Syndic ?~~ | Damien | ✅ Résolu — oui, le Conseil a `C/R/U¹/D¹` (matrice CRUD). |
| Q-CS-002 | ~~Les résultats doivent-ils être visibles avant la clôture ?~~ | Damien | ✅ Résolu — oui, en temps réel pendant l'ouverture (AC-CS-003-A). |
| Q-CS-003 | ~~Faut-il un mécanisme de pondération des réponses par tantièmes ?~~ | Damien | ✅ Résolu — non, jamais. Ajouté en Hors scope. |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-CS-001 | Must | AC-CS-001-A à F | Consultations (C) |
| US-CS-001b | Must | AC-CS-001b-A à D | Consultations (transition → open + notification) |
| US-CS-002 | Must | AC-CS-002-A à C | Réponses (C) |
| US-CS-003 | Must | AC-CS-003-A à C | Résultats (R) |
| US-CS-004 | Should | — | Relance non-répondants |
