---
product: Référentiel prestataires
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - LEGAL.md §5
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Référentiel prestataires » de La Ruche.
    Le référentiel est un annuaire partagé référencé par les Tâches et les
    Contrats. Les copropriétaires peuvent consulter les prestataires (nom,
    spécialité, contact) mais pas les conditions contractuelles.
    Ne pas implémenter : notation publique, appel d'offres, mise en relation
    automatique avec des prestataires externes.
---

# PRD — Référentiel prestataires

## 1. Vue d'ensemble

### Objectif

Centraliser l'**annuaire des prestataires** utilisés ou référencés par la copropriété, pour être utilisé comme source commune par les modules Tâches et Contrats, et consultable par les copropriétaires.

### Pourquoi

- Aujourd'hui, les coordonnées des prestataires sont éparpillées (emails, papier, mémoire du syndic).
- Un référentiel partagé évite la resaisie dans les tâches et les contrats, et facilite la passation de relais lors d'un changement de syndic.
- L'accès en lecture des copropriétaires favorise la transparence sur les prestataires engagés.

### KPIs

- Taux de tâches et contrats avec un prestataire référencé ≥ 70 % à 6 mois.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables.

### Référencement croisé

- Un prestataire PEUT être référencé dans une tâche (`task.contractor_id`) et/ou dans un contrat (`vendor_contract.supplier_id`).
- La suppression d'un prestataire DOIT être refusée (422) s'il est référencé dans une tâche (quel que soit son statut, y compris clôturée) ou dans un contrat actif — le corps de l'erreur DOIT lister les références bloquantes. La référence dans une tâche clôturée est bloquante car le carnet d'entretien affiche le nom du prestataire pour les entrées historiques.
- Si aucune référence n'existe (ni tâche, ni contrat), la suppression est un **hard-delete** définitif.

### Soft-delete (archivage)

- Un prestataire peut être archivé (soft-delete via `active = false`) pour le retirer des sélecteurs sans perdre les liens historiques avec les tâches et contrats.
- Un prestataire archivé reste visible dans les fiches des tâches et contrats qui le référencent (affichage en lecture seule).
- La liste des prestataires est filtrée par défaut sur `active = true` — un filtre « Afficher les archivés » permet de les retrouver.
- Un prestataire archivé peut être réactivé.

### Pièces jointes

- Les attestations d'assurance et certifications sont stockées via le module Documents — catégorie `PRESTATAIRE`, avec `contractor_id` comme référence.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal |
|---|---|---|
| Persona-Syndic | Syndic bénévole | Gérer l'annuaire des prestataires |
| Persona-Conseil | Conseil syndical | Consulter les prestataires |
| Persona-Copro | Copropriétaire | Consulter l'annuaire simplifié des prestataires |

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Prestataires | `C / R / U / D¹` | `C / R / U / D¹` | `R²` | `R²` |
| Attestations / certifications | `C / R / D¹` | `R` | `X` | `X` |

> ¹ Suppression hard-delete autorisée uniquement si le prestataire n'est référencé dans aucune tâche (quel que soit son statut, y compris clôturée) ni contrat actif (sinon 422).
> ² Copropriétaire et Locataire : nom, spécialité, téléphone, email — sans accès aux documents ni aux conditions contractuelles.

### Règles de sécurité

- Les documents (attestations d'assurance, KBIS) ne sont pas accessibles aux copropriétaires.
- Filtrage par `building_id` — un prestataire peut être partagé entre copropriétés si le syndic gère plusieurs immeubles (question ouverte).

---

## 4. Fiche prestataire

| Champ | Obligatoire | Description |
|---|---|---|
| Nom / raison sociale | Oui | Nom commercial ou raison sociale |
| Spécialité | Oui | Catégorie (`PLOMBERIE`, `ELECTRICITE`, `SERRURERIE`, `NETTOYAGE`, `ESPACES_VERTS`, `ASCENSEUR`, `CHAUFFAGE`, `COUVERTURE`, `AUTRE`) |
| Téléphone | Non | Numéro de contact principal |
| Email | Non | Email de contact |
| SIRET | Non | Numéro SIRET (9 chiffres + 5 chiffres établissement) |
| Notes | Non | Remarques internes (non visibles des copropriétaires) |
| Attestation assurance RC Pro | Non | Document via module Documents |

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-RP-001 | Créer et gérer un prestataire | Fiche complète avec tous les champs |
| Must | US-RP-002 | Lister et rechercher les prestataires | Filtres par spécialité, statut |
| Must | US-RP-003 | Référencer un prestataire depuis une tâche ou un contrat | Sélecteur dans les formulaires |
| Should | US-RP-004 | Rattacher une attestation d'assurance | Via module Documents |
| Must | US-RP-005 | Accès lecture copropriétaires | Annuaire simplifié (fiche publique) |
| Could | US-RP-006 | Détecter les doublons | Alerte si SIRET identique existant |

### User stories détaillées

#### US-RP-001 — Créer et gérer un prestataire

**En tant que** `Persona-Syndic`,
**je veux** créer et maintenir la fiche d'un prestataire,
**afin de** centraliser ses coordonnées et les réutiliser dans les tâches et contrats.

**Critères d'acceptation :**

- **AC-RP-001-A** : Champs obligatoires : nom, spécialité.
- **AC-RP-001-B** : Champs optionnels : téléphone, email, SIRET, notes internes.
- **AC-RP-001-C** : La suppression d'un prestataire DOIT être refusée (422) s'il est référencé dans une tâche (quel que soit son statut, y compris clôturée) ou dans un contrat actif — avec la liste des références bloquantes. Si aucune référence n'existe, le hard-delete est autorisé.

#### US-RP-002 — Lister et rechercher les prestataires

**En tant que** `Persona-Syndic` ou `Persona-Conseil`,
**je veux** lister et rechercher les prestataires,
**afin de** retrouver rapidement un contact.

**Critères d'acceptation :**

- **AC-RP-002-A** : La liste DOIT être filtrable par spécialité et par statut (actif / archivé).
- **AC-RP-002-B** : Une recherche par nom DOIT être disponible (recherche textuelle insensible à la casse).

#### US-RP-003 — Référencer depuis tâche ou contrat

**En tant que** `Persona-Syndic`,
**je veux** sélectionner un prestataire du référentiel lors de la création d'une tâche ou d'un contrat,
**afin de** éviter la ressaisie et garder la traçabilité.

**Critères d'acceptation :**

- **AC-RP-003-A** : Les formulaires de tâche et de contrat DOIVENT proposer un sélecteur de prestataire depuis le référentiel.
- **AC-RP-003-B** : Il DOIT être possible de saisir un prestataire hors-référentiel (nom libre) sans bloquer la création.

#### US-RP-004 — Rattacher une attestation d'assurance

**En tant que** `Persona-Syndic`,
**je veux** rattacher l'attestation d'assurance RC Pro d'un prestataire à sa fiche,
**afin de** centraliser les documents sans obligation de suivi de leur expiration.

**Critères d'acceptation :**

- **AC-RP-004-A** : Les attestations DOIVENT être déposées via le module Documents — catégorie `PRESTATAIRE`, avec `contractor_id` comme référence d'entité.
- **AC-RP-004-B** : La vue détail du prestataire DOIT afficher la liste des documents rattachés avec leur nom et date de dépôt.

#### US-RP-005 — Annuaire simplifié pour les copropriétaires

**En tant que** `Persona-Copro`,
**je veux** consulter l'annuaire des prestataires de ma copropriété,
**afin de** connaître les prestataires engagés par le syndic.

**Critères d'acceptation :**

- **AC-RP-005-A** : Le copropriétaire DOIT avoir accès à la fiche de chaque prestataire, avec les champs : nom, spécialité, téléphone, email. Notes internes et documents (attestations) sont exclus.

#### US-RP-006 — Détecter les doublons

**En tant que** `Persona-Syndic`,
**je veux** être alerté si un prestataire avec le même SIRET existe déjà dans le référentiel,
**afin de** éviter la création de doublons.

**Critères d'acceptation :**

- **AC-RP-006-A** : Si un SIRET est saisi et qu'un prestataire avec ce SIRET existe déjà, l'interface DOIT afficher un avertissement non bloquant avec le lien vers la fiche existante.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Perf-001 | Performance | Chargement de la liste des prestataires | ≤ 200 ms (p95) |
| NFR-Sec-001 | Sécurité | Documents (attestations) masqués aux copropriétaires | 100 % |

---

## 7. Hors scope

- Notation publique des prestataires.
- Mise en relation automatique avec des prestataires externes (marketplace).
- Appel d'offres ou demande de devis intégré.
- Suivi du statut administratif des prestataires (RC Pro, assurances, décennale) — hors périmètre de La Ruche.

---

## 8. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-RP-001 | ~~Un prestataire peut-il être partagé entre plusieurs copropriétés ?~~ | Damien | ✅ Résolu — non, périmètre par `building_id`. Partage multi-copropriété = lot futur. |
| Q-RP-002 | ~~Le Conseil peut-il créer ou modifier des prestataires ?~~ | Damien | ✅ Résolu — oui, le Conseil peut créer, modifier et supprimer des prestataires (`C/R/U/D`). |
| Q-RP-003 | ~~Faut-il une alerte sur les attestations d'assurance ?~~ | Damien | ✅ Résolu — hors périmètre (suivi administratif des prestataires exclu). |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-RP-001 | Must | AC-RP-001-A à C | Prestataires (C / U / D hard) |
| US-RP-002 | Must | AC-RP-002-A à B | Prestataires (R) |
| US-RP-003 | Must | AC-RP-003-A à B | Référencement croisé |
| US-RP-004 | Should | AC-RP-004-A à B | Attestations (C / R via Documents) |
| US-RP-005 | Must | AC-RP-005-A | Annuaire simplifié (R copropriétaires) |
| US-RP-006 | Could | AC-RP-006-A | Détection doublons SIRET |
