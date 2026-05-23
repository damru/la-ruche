---
product: Conformité et reporting réglementaire
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - LEGAL.md §8
  - LEGAL.md §9
  - ADR-009
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Conformité et reporting réglementaire »
    de La Ruche. La fiche synthétique annuelle est obligatoire (art. 8-2 loi 1965).
    Une fois publiée, la fiche est immuable (Pattern A). La génération s'appuie
    sur les données existantes dans d'autres modules — aucune saisie redondante.
    Ne pas implémenter : télétransmission directe au ANAH/RNC via API,
    DTG, DPE collectif, diagnostics techniques.
---

# PRD — Conformité et reporting réglementaire

## 1. Vue d'ensemble

### Objectif

Assurer la production et la mise à disposition des **documents réglementaires annuels** obligatoires, en particulier la fiche synthétique de la copropriété (art. 8-2 loi 1965) et les données transmissibles au Registre National des Copropriétés (RNC — ANAH).

### Pourquoi

- La fiche synthétique est obligatoire depuis le 1er janvier 2018. Le syndic doit la remettre chaque année à tout copropriétaire qui en fait la demande.
- Le RNC (INSC) impose une mise à jour annuelle des données clés de la copropriété.
- Ces obligations sont aujourd'hui non outillées pour les syndics bénévoles — sources de non-conformité.

### KPIs

- Taux de copropriétés avec une fiche synthétique générée pour l'exercice en cours ≥ 95 %.
- Zéro demande de fiche synthétique restée sans réponse dans les 7 jours.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables.

### Immuabilité de la fiche publiée

- Une fiche synthétique au statut `published` est immuable (Pattern A) — aucune modification possible.
- En cas d'erreur, une nouvelle fiche est générée pour le même exercice avec une nouvelle révision.

### Génération sans saisie redondante

- La fiche synthétique est générée automatiquement à partir des données existantes dans d'autres modules :
  - Registre des lots (nombre de lots, tantièmes)
  - Profil immeuble (adresse, construction, syndic)
  - Budget (montant du budget prévisionnel, charges annuelles)
  - Propriétaires (nombre de copropriétaires)
  - Fonds travaux (solde)
- Aucune donnée financière ou légale ne doit être ressaisie.

### Patterns interdits

- Aucun `PUT`/`PATCH` sur une fiche au statut `published`.
- Ne pas implémenter de télétransmission directe au ANAH via API.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal |
|---|---|---|
| Persona-Syndic | Syndic bénévole | Générer, vérifier et publier les documents réglementaires |
| Persona-Conseil | Conseil syndical | Consulter les fiches publiées |
| Persona-Copro | Copropriétaire | Accéder à la fiche synthétique annuelle |

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Fiche synthétique | `C / R / U¹` | `R` | `R²` | `X` |
| Export RNC | `C / R` | `X` | `X` | `X` |

> ¹ Modification uniquement au statut `draft`. Une fiche `published` est immuable.
> ² Accès à la dernière fiche publiée uniquement.

### Règles de sécurité

- Un copropriétaire accède uniquement à la fiche synthétique publiée — pas aux brouillons.
- Les données financières détaillées (ligne par ligne) ne sont pas exposées dans la fiche — uniquement les totaux.
- Filtrage systématique par `building_id`.

---

## 4. Cycle de vie de la fiche synthétique

```
draft ──(vérification)──▶ review ──(publication)──▶ published ──(archivage auto)──▶ archived
```

| État | Description |
|---|---|
| `draft` | Générée automatiquement — en cours de vérification par le syndic |
| `review` | Vérifiée — en attente de publication |
| `published` | Publiée — accessible aux copropriétaires — immuable |
| `archived` | Remplacée par une fiche plus récente pour le même exercice |

---

## 5. Contenu de la fiche synthétique

Conformément à l'art. 8-2 loi 1965 et au décret n° 2016-1822 :

| Section | Données | Source |
|---|---|---|
| Identification | Adresse, nom de la copropriété, n° de lot principal | Profil immeuble |
| Administratif | Syndic (nom, mandat, coordonnées) | Profil immeuble |
| Composition | Nombre de lots, nombre de copropriétaires | Registre + Propriétaires |
| Financier | Budget prévisionnel, charges de l'exercice, solde fonds travaux | Budget + Fonds travaux |
| Contrats | Contrat d'assurance MRI (compagnie, n°, échéance) | Contrats |
| Sinistres | Nombre de sinistres en cours | Sinistres |

---

## 6. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-CR-001 | Générer la fiche synthétique | Auto-remplissage depuis les modules |
| Must | US-CR-002 | Vérifier et corriger le brouillon | Formulaire pré-rempli modifiable avant publication |
| Must | US-CR-003 | Publier la fiche | Rendre accessible aux copropriétaires |
| Must | US-CR-004 | Consulter la fiche publiée | Accès copropriétaires |
| Should | US-CR-005 | Exporter la fiche en PDF | Pour remise physique ou email |
| Could | US-CR-006 | Exporter les données RNC | Fichier structuré pour déclaration ANAH |

### User stories détaillées

#### US-CR-001 — Générer la fiche synthétique

**En tant que** `Persona-Syndic`,
**je veux** générer la fiche synthétique de l'exercice en cours,
**afin de** préparer le document réglementaire annuel sans ressaisie.

**Critères d'acceptation :**

- **AC-CR-001-A** : La génération DOIT pré-remplir tous les champs depuis les modules existants (profil immeuble, registre, budget, contrats, sinistres).
- **AC-CR-001-B** : Un exercice est identifié par `year` (ex. : 2025).
- **AC-CR-001-C** : La fiche est créée au statut `draft`.
- **AC-CR-001-D** : Si des données sources manquent (budget non approuvé, assurance MRI absente), des alertes de complétude DOIVENT être affichées dans le formulaire.

#### US-CR-002 — Vérifier et corriger le brouillon

**En tant que** `Persona-Syndic`,
**je veux** vérifier et ajuster la fiche avant publication,
**afin de** corriger d'éventuelles données incorrectes ou incomplètes.

**Critères d'acceptation :**

- **AC-CR-002-A** : Le syndic DOIT pouvoir modifier tous les champs de la fiche en statut `draft`.
- **AC-CR-002-B** : Toute modification manuelle DOIT être tracée (champ modifié manuellement plutôt qu'issu des données sources) — indicateur visuel.

#### US-CR-003 — Publier la fiche

**En tant que** `Persona-Syndic`,
**je veux** publier la fiche synthétique vérifiée,
**afin de** la rendre accessible aux copropriétaires et respecter l'obligation légale.

**Critères d'acceptation :**

- **AC-CR-003-A** : La publication DOIT passer la fiche au statut `published` — immuable dès lors.
- **AC-CR-003-B** : Si une fiche publiée existe déjà pour le même exercice, elle DOIT passer au statut `archived` lors de la publication de la nouvelle révision.
- **AC-CR-003-C** : Les copropriétaires DOIVENT être notifiés de la disponibilité de la nouvelle fiche.

#### US-CR-004 — Consulter la fiche publiée

**En tant que** `Persona-Copro`,
**je veux** consulter la fiche synthétique de ma copropriété,
**afin de** exercer mon droit d'information légal (art. 8-2 loi 1965).

**Critères d'acceptation :**

- **AC-CR-004-A** : Seule la fiche au statut `published` la plus récente est accessible aux copropriétaires.
- **AC-CR-004-B** : La fiche DOIT être consultable en ligne.

#### US-CR-005 — Exporter la fiche en PDF

**En tant que** `Persona-Syndic`, `Persona-Conseil` ou `Persona-Copro`,
**je veux** exporter la fiche synthétique publiée au format PDF,
**afin de** la remettre physiquement ou par email à un copropriétaire ou un notaire.

**Critères d'acceptation :**

- **AC-CR-005-A** : L'export DOIT être disponible uniquement pour une fiche au statut `published`.
- **AC-CR-005-B** : Le PDF DOIT inclure l'en-tête réglementaire complet, toutes les sections de la fiche, et la date de génération.
- **AC-CR-005-C** : Le PDF DOIT être conforme au contenu affiché en ligne — aucune donnée supplémentaire ni omission.

---

## 7. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Legal-001 | Conformité | Fiche synthétique disponible | Délai de remise à copropriétaire ≤ 7 jours après demande |
| NFR-Legal-002 | Conformité | Contenu fiche | 100 % conforme décret 2016-1822 |
| NFR-Perf-001 | Performance | Génération fiche | ≤ 2 s |

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Copropriétés avec fiche synthétique générée pour l'exercice en cours | ≥ 95 % | À 6 mois |
| Demandes de fiche synthétique satisfaites sous 7 jours | 100 % | Continu |

---

## 9. Hors scope

- Télétransmission directe au RNC / ANAH via API (export fichier uniquement).
- Diagnostics techniques (DPE collectif, DTG, audit énergétique) — lot futur.
- Rapport d'assemblée générale (couvert par le module AG).
- Comptes rendus de gestion détaillés (couvert par le module Budget).
- Fiche de synthèse multi-exercices.

---

## 10. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-CR-001 | Le format d'export RNC est-il un CSV standard ou un format ANAH propriétaire ? | Damien | Avant US-CR-006 |
| Q-CR-002 | La fiche synthétique doit-elle être signée électroniquement par le syndic pour avoir valeur légale ? | Damien | Analyse de gap réglementaire |
| Q-CR-003 | ~~Faut-il archiver les fiches des exercices passés (accès sur plusieurs années) ?~~ | Damien | ✅ Résolu — archivage automatique couvert par AC-CR-003-B. Pas de consultation historique multi-exercices pour le MVP. |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-CR-001 | Must | AC-CR-001-A à D | Fiche synthétique (C — génération) |
| US-CR-002 | Must | AC-CR-002-A à B | Fiche synthétique (U en draft) |
| US-CR-003 | Must | AC-CR-003-A à C | Fiche synthétique (transition → published) |
| US-CR-004 | Must | AC-CR-004-A à B | Fiche synthétique (R copropriétaires) |
| US-CR-005 | Should | AC-CR-005-A à C | Export PDF |
| US-CR-006 | Could | — | Export RNC |
