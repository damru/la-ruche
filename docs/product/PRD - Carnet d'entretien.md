---
product: Carnet d'entretien
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - LEGAL.md §7
  - LEGAL.md §5
  - ADR-009
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Carnet d'entretien » de La Ruche.
    Les entrées manuelles sont append-only (jamais de UPDATE/DELETE).
    Les entrées automatiques (source = automatic) sont immuables dès création.
    Le carnet doit pouvoir être exporté en PDF à tout moment.
    Ne pas implémenter : import en masse, DPE collectif, diagnostics techniques.
  modification_policy: >
    Ne modifier ce fichier que sur validation explicite après analyse de gap.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Carnet d'entretien

## 1. Vue d'ensemble

### Objectif

Maintenir le **carnet d'entretien réglementaire** de la copropriété (décret n° 2001-477), en agrégeant automatiquement les événements de maintenance issus des autres modules (tâches, contrats, sinistres) et en permettant la saisie d'entrées manuelles. Garantir l'exportabilité du document pour consultation par tout copropriétaire (art. 18-I loi 1965).

### Pourquoi

- Le carnet d'entretien est obligatoire (art. 18-I loi 1965 modifié par la loi ALUR). Son absence ou son incomplétude expose le syndic à sa responsabilité.
- En pratique, les syndics bénévoles tiennent ce carnet dans des tableurs ou classeurs — souvent incomplets.
- L'alimentation automatique depuis les tâches clôturées et les contrats actifs élimine 80 % de la saisie manuelle.

### KPIs

- Taux de copropriétés avec un carnet d'entretien à jour (dernière entrée < 12 mois) ≥ 90 %.
- Taux de demandes de consultation du carnet satisfaites via l'export PDF ≥ 95 %.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables.

### Immuabilité des entrées

- Les entrées à `source = manual` sont **append-only** — aucun `UPDATE` ou `DELETE` n'est autorisé après création.
- Les entrées à `source = automatic` sont immuables dès leur génération — elles reflètent un événement figé dans un autre module.
- L'en-tête réglementaire (identité de l'immeuble) est généré dynamiquement depuis le profil de l'immeuble — il n'est pas stocké dans le carnet.

### Génération automatique des entrées

- **Clôture d'une tâche** (`task.status → closed`) → entrée automatique systématique, quelle que soit la catégorie de la tâche. Une tâche clôturée par erreur puis rouverte et re-clôturée génère une **nouvelle entrée indépendante** à chaque clôture — chaque transition est immuable.
- **Activation d'un contrat** (`vendor_contract.status → active`) → entrée automatique de type `CONTRACT_ACTIVATED`.
- **Résiliation d'un contrat** → entrée automatique de type `CONTRACT_TERMINATED`.
- **Clôture d'un sinistre** → entrée automatique de type `CLAIM_CLOSED`.

### Export

- L'export PDF DOIT être disponible en deux modes : **Exporter tout** (en-tête réglementaire + log complet) et **Exporter la sélection** (log filtré si filtres actifs).
- L'export DOIT être disponible pour Syndic, Conseil et Copropriétaire.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal |
|---|---|---|
| Persona-Syndic | Syndic bénévole | Maintenir et exporter le carnet |
| Persona-Conseil | Conseil syndical | Consulter le carnet |
| Persona-Copro | Copropriétaire | Consulter le carnet (droit légal) |

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| En-tête réglementaire | `R` | `R` | `R` | `X` |
| Entrées manuelles | `C / R` | `R` | `R` | `X` |
| Entrées automatiques | `R` | `R` | `R` | `X` |
| Export PDF | `R` | `R` | `R` | `X` |

> Le droit de consultation des copropriétaires sur le carnet d'entretien est fondé sur l'art. 18-I loi 1965 et le décret n° 2005-240.

### Règles de sécurité

- Les montants (contractuels, de remboursement, entrées manuelles) sont visibles par le Syndic et le Conseil — ils DOIVENT être masqués pour les copropriétaires, dans les vues et dans les exports PDF.
- Filtrage systématique par `building_id`.

---

## 4. Contenu réglementaire

### En-tête réglementaire (décret 2001-477)

Généré dynamiquement depuis le profil de l'immeuble :

| Élément | Source |
|---|---|
| Adresse de l'immeuble | Profil immeuble |
| Référence cadastrale | Profil immeuble |
| Syndic en exercice (nom, coordonnées) | Profil immeuble |
| Date de construction (approximative) | Profil immeuble |
| Contrat d'assurance MRI en cours | Module Contrats (type `ASSURANCE`) |

### Catégories d'entrées du log

| Catégorie | Source | Exemples |
|---|---|---|
| `TRAVAUX` | Manuel ou tâche clôturée | Ravalement de façade, réfection toiture |
| `MAINTENANCE` | Manuel ou tâche clôturée | Entretien chaudière, nettoyage gaines |
| `CONTRAT` | Contrat activé / résilié | Nouveau contrat ascenseur |
| `SINISTRE` | Sinistre clôturé | Dégâts des eaux hall — remboursé |
| `AUTRE` | Manuel uniquement | Événement non couvert par les autres catégories |

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-CE-001 | Consulter le carnet | Vue chronologique + en-tête réglementaire |
| Must | US-CE-002 | Ajouter une entrée manuelle | Date, catégorie, description, prestataire (optionnel) |
| Must | US-CE-003 | Voir les entrées automatiques | Générées par tâches, contrats, sinistres |
| Must | US-CE-004 | Exporter le carnet en PDF | En-tête + log complet, horodaté |
| Should | US-CE-005 | Filtrer le log | Par catégorie, par période, par source |

### User stories détaillées

#### US-CE-001 — Consulter le carnet

**En tant que** `Persona-Syndic`, `Persona-Conseil` ou `Persona-Copro`,
**je veux** consulter le carnet d'entretien de la copropriété,
**afin de** accéder à l'historique de maintenance réglementaire.

**Critères d'acceptation :**

- **AC-CE-001-A** : La vue DOIT afficher l'en-tête réglementaire (identité immeuble, syndic, assurance MRI) au-dessus du log chronologique.
- **AC-CE-001-B** : Le log DOIT être trié par date décroissante par défaut.
- **AC-CE-001-C** : Chaque entrée DOIT afficher : date, catégorie, description, source (manuel / automatique), prestataire si renseigné.
- **AC-CE-001-D** : Si aucun contrat d'assurance multirisques immeuble (MRI) actif n'est trouvé dans le module Contrats, l'en-tête DOIT afficher un avertissement visible : « Aucune assurance MRI active — conformité réglementaire non satisfaite (art. L. 9-1 loi 65-557 modifiée) ».

#### US-CE-002 — Ajouter une entrée manuelle

**En tant que** `Persona-Syndic`,
**je veux** ajouter une entrée manuelle dans le carnet,
**afin de** consigner un événement de maintenance non couvert automatiquement.

**Critères d'acceptation :**

- **AC-CE-002-A** : Champs obligatoires : date de l'événement, catégorie, description.
- **AC-CE-002-B** : Champs optionnels : prestataire (référence ou nom libre), montant HT (visible par Syndic et Conseil — masqué aux copropriétaires), pièces jointes (déposées via le module Documents — catégorie `CARNET_ENTRETIEN`, avec `entry_id` comme référence d'entité).
- **AC-CE-002-C** : L'entrée est créée avec `source = manual` et ne peut plus être modifiée ni supprimée après création. L'interface DOIT afficher un modal de confirmation explicite avant validation, rappelant l'irréversibilité de l'action.

#### US-CE-003 — Voir les entrées automatiques

**En tant que** tout utilisateur autorisé,
**je veux** voir dans le carnet les événements générés automatiquement,
**afin de** avoir un historique complet sans double saisie.

**Critères d'acceptation :**

- **AC-CE-003-A** : Une entrée automatique DOIT être générée à chaque : clôture d'une tâche (toutes catégories), activation et résiliation d'un contrat fournisseur, clôture d'un sinistre.
- **AC-CE-003-B** : Les entrées automatiques sont distinguables visuellement des entrées manuelles (badge `source`).
- **AC-CE-003-C** : Les entrées automatiques affichées aux copropriétaires ne DOIVENT PAS inclure les montants.

#### US-CE-004 — Exporter le carnet en PDF

**En tant que** tout utilisateur autorisé,
**je veux** exporter le carnet en PDF,
**afin de** satisfaire une demande de consultation physique ou l'envoyer à un notaire.

**Critères d'acceptation :**

- **AC-CE-004-A** : Un bouton « Exporter tout » DOIT générer un PDF incluant l'en-tête réglementaire et le log chronologique complet, sans filtre appliqué.
- **AC-CE-004-B** : Le PDF DOIT mentionner la date et l'heure de génération.
- **AC-CE-004-C** : Les montants sont masqués dans le PDF généré pour les copropriétaires.
- **AC-CE-004-D** : Si des filtres sont actifs (US-CE-005), un bouton « Exporter la sélection » DOIT être disponible et générer un PDF du log filtré, avec mention des filtres appliqués (catégories, période) en page de garde.

#### US-CE-005 — Filtrer le log

**En tant que** tout utilisateur autorisé,
**je veux** filtrer le log du carnet par catégorie et par période,
**afin de** retrouver rapidement les entrées pertinentes.

**Critères d'acceptation :**

- **AC-CE-005-A** : Filtres disponibles : catégorie (multi-sélection parmi `TRAVAUX`, `MAINTENANCE`, `CONTRAT`, `SINISTRE`, `AUTRE`) et période (date début / date fin).
- **AC-CE-005-B** : La combinaison des deux filtres DOIT être possible simultanément.
- **AC-CE-005-C** : Un filtre actif DOIT être visuellement indiqué et réinitialisable en un clic.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Legal-001 | Conformité | Contenu conforme au décret 2001-477 | En-tête complet à 100 % si profil immeuble renseigné |
| NFR-Perf-001 | Performance | Génération PDF | ≤ 3 s (p95) |
| NFR-Sec-001 | Sécurité | Montants masqués pour copropriétaires | 100 % des exports et vues |

---

## 7. Hors scope

- DPE collectif et diagnostics techniques obligatoires (DTG, audit énergétique) — couverts par leurs propres registres légaux, hors périmètre de ce module.
- Import en masse d'un historique existant (CSV).
- Signature électronique du carnet.
- Génération automatique depuis les factures fournisseurs (non implémenté).
- Consultation par le locataire à l'entrée dans les lieux (processus externe à l'Application).

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Copropriétés avec carnet à jour (< 12 mois) | ≥ 90 % | À 6 mois |
| Exports PDF réussis | ≥ 99 % | Continu |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-CE-001 | ~~Les montants sont-ils visibles par le Conseil ?~~ | Damien | ✅ Résolu — oui, Conseil voit les montants (masqués uniquement aux copropriétaires). |
| Q-CE-002 | ~~Faut-il une modération des entrées automatiques ?~~ | Damien | ✅ Résolu — non. Prévention par modal de confirmation fort (module Tâches). Chaque clôture génère une entrée indépendante. |
| Q-CE-003 | ~~Diagnostics (DPE, ERP) dans l'en-tête ?~~ | Damien | ✅ Résolu — hors scope, couverts par leurs propres registres légaux. |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-CE-001 | Must | AC-CE-001-A à D | Carnet (R) + alerte MRI |
| US-CE-002 | Must | AC-CE-002-A à C | Entrées manuelles (C append-only + modal irréversibilité + pièces jointes) |
| US-CE-003 | Must | AC-CE-003-A à C | Entrées automatiques (R — toutes tâches) |
| US-CE-004 | Must | AC-CE-004-A à D | Export PDF (Exporter tout + Exporter la sélection) |
| US-CE-005 | Should | AC-CE-005-A à C | Filtres (catégorie + période) |
