---
product: Sinistres
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - LEGAL.md §6
  - ADR-009
llm_directives:
  temperature: 0.2
  persona: >
    Tu travailles sur la feature « Sinistres » de La Ruche. Les champs
    transmitted_at et reimbursement_* sont immuables une fois définis.
    Toute transition de statut DOIT passer par un endpoint dédié et appeler
    is_mutable() avant toute écriture. Les délais légaux de déclaration DOIVENT
    être calculés et affichés dès la création.
    Ne pas implémenter : gestion des assurances individuelles, téléclaration PSI.
---

# PRD — Sinistres

## 1. Vue d'ensemble

### Objectif

Permettre au syndic de **déclarer, suivre et clôturer les sinistres** affectant les parties communes ou la structure de l'immeuble, de respecter les délais légaux de déclaration auprès de l'assureur, et de conserver la traçabilité complète des échanges et remboursements.

### Pourquoi

- Les délais de déclaration à l'assureur (ex. : 5 jours ouvrés pour les dégâts des eaux) sont méconnus des syndics bénévoles — un dépassement peut entraîner le refus de prise en charge.
- La traçabilité des remboursements et franchises est nécessaire pour l'arrêté de comptes annuel.
- Les pièces justificatives (constats, devis, factures, rapports d'expertise) doivent être centralisées pour les échanges avec l'assureur.

### KPIs

- Taux de sinistres transmis à l'assureur dans le délai légal ≥ 95 %.
- Taux de sinistres avec au moins une pièce jointe rattachée ≥ 80 %.
- Délai moyen entre déclaration et clôture : référence établie à 6 mois.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables.

### Immuabilité

- Un sinistre au statut `transmitted` est immuable sur `transmitted_at`, `insurer_ref` et `declared_amount`.
- Les champs de remboursement (`reimbursement_amount`, `reimbursement_date`, `deductible`) sont immuables une fois renseignés lors de la clôture.
- Toute transition de statut DOIT passer par un endpoint dédié et être tracée dans le journal d'audit.

### Champs financiers — distinction

- **`montant_estime`** : estimé par le syndic lors de la saisie initiale, modifiable tant que le sinistre est au statut `draft`.
- **`declared_amount`** : montant déclaré à l'assureur, saisi et figé lors de la transmission. Distinct de `montant_estime` — non renseigné à la création.

### Calcul des délais

- À la création du sinistre, la date limite de déclaration DOIT être calculée selon le type et affichée.
- Si la date limite est dépassée et que le statut est `draft` ou `declared`, un indicateur d'alerte visuel DOIT être affiché.

### Intégration documentaire

- Pièces jointes via le module Documents — catégorie `SINISTRE`, avec `claim_id` comme référence d'entité.

### Patterns interdits

- Aucun `PUT`/`PATCH` sur `transmitted_at` ou les champs de remboursement une fois définis.
- Pas de téléclaration directe à l'assureur via API externe.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal |
|---|---|---|
| Persona-Syndic | Syndic bénévole | Déclarer, suivre et clôturer les sinistres |
| Persona-Conseil | Conseil syndical | Consulter l'état des sinistres en cours |

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Sinistres | `C / R / U¹ / D¹` | `R` | `X` | `X` |
| Notes de suivi | `C / R` | `R` | `X` | `X` |
| Pièces jointes sinistre | `C / R / D¹` | `R` | `X` | `X` |

> ¹ Modification / suppression uniquement au statut `draft`. Champs financiers et `transmitted_at` immuables une fois définis.
> Les résidents signalent un incident via le module Signalements — le syndic décide d'en faire un sinistre.

### Règles de sécurité

- Les montants (franchise, remboursement, dommages estimés) ne sont visibles que par Syndic et Conseil.
- Filtrage systématique par `building_id` — aucun accès cross-copropriété.

---

## 4. Cycle de vie

```
draft ──(déclarer)──▶ declared ──(transmettre)──▶ transmitted ──(suivi)──▶ in_progress ──(clore)──▶ closed
```

| État | Description |
|---|---|
| `draft` | Sinistre en cours de saisie — modifiable |
| `declared` | Déclaré en interne — délai légal en cours |
| `transmitted` | Transmis à l'assureur — `transmitted_at` figé |
| `in_progress` | Instruction en cours par l'assureur |
| `closed` | Clôturé — résultat enregistré (remboursement ou refus) |

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-SIN-001 | Saisir un sinistre (draft) | Création modifiable + supprimable |
| Must | US-SIN-001b | Formaliser la déclaration | Transition draft → declared, déclenche le délai légal |
| Must | US-SIN-002 | Afficher le délai légal de déclaration | Calculé selon type, alerte si dépassé |
| Must | US-SIN-003 | Transmettre à l'assureur | Date + référence assureur figées |
| Must | US-SIN-004 | Ajouter des notes de suivi | Append-only, horodatées |
| Must | US-SIN-005 | Clôturer un sinistre | Résultat + champs financiers figés |
| Should | US-SIN-006 | Rattacher des documents | Catégorie `SINISTRE` via module Documents |
| Could | US-SIN-007 | Export récapitulatif annuel | Liste des sinistres pour arrêté de comptes |

### User stories détaillées

#### US-SIN-001 — Saisir un sinistre

**En tant que** `Persona-Syndic`,
**je veux** créer un sinistre en brouillon dans l'Application,
**afin de** constituer le dossier avant de le formaliser officiellement.

**Critères d'acceptation :**

- **AC-SIN-001-A** : Champs obligatoires : type (`DEGATS_EAUX`, `INCENDIE`, `VANDALISME`, `CATASTROPHE_NATURELLE`, `AUTRE`), date du sinistre, description.
- **AC-SIN-001-B** : Champs optionnels : localisation (parties communes ou référence lot), `montant_estime` HT, `insurance_contract_id` (référence au contrat d'assurance MRI actif, optionnel), notes libres.
- **AC-SIN-001-C** : Le sinistre est créé au statut `draft` — modifiable et supprimable librement.
- **AC-SIN-001-D** : La suppression d'un sinistre `draft` DOIT être autorisée (hard-delete si aucun document attaché, sinon 422).

#### US-SIN-001b — Formaliser la déclaration

**En tant que** `Persona-Syndic`,
**je veux** formaliser un sinistre brouillon en déclaration officielle,
**afin de** déclencher le calcul du délai légal de déclaration à l'assureur.

**Critères d'acceptation :**

- **AC-SIN-001b-A** : La formalisation DOIT passer le sinistre du statut `draft` au statut `declared` via un endpoint dédié (`POST /{id}/declare`).
- **AC-SIN-001b-B** : Une fois `declared`, le type et la date du sinistre sont immuables — seules les notes et pièces jointes restent modifiables.
- **AC-SIN-001b-C** : La formalisation DOIT déclencher le calcul de la date limite de déclaration à l'assureur (US-SIN-002).
- **AC-SIN-001b-D** : La formalisation DOIT être tracée dans le journal d'audit.

#### US-SIN-002 — Afficher le délai légal

**En tant que** `Persona-Syndic`,
**je veux** voir la date limite de déclaration à l'assureur,
**afin de** ne pas dépasser le délai légal contractuel.

**Critères d'acceptation :**

- **AC-SIN-002-A** : Pour `DEGATS_EAUX`, le délai par défaut est 5 jours ouvrés à partir de la date du sinistre.
- **AC-SIN-002-B** : Pour les autres types, le délai est configurable dans les paramètres de la copropriété (valeur par défaut : 5 jours ouvrés).
- **AC-SIN-002-C** : Si la date limite est dépassée sans que le sinistre soit au statut `transmitted` ou ultérieur, un indicateur d'alerte DOIT être affiché en rouge.

#### US-SIN-003 — Transmettre à l'assureur

**En tant que** `Persona-Syndic`,
**je veux** enregistrer la transmission du sinistre à l'assureur,
**afin de** figer la date de transmission et la référence du dossier.

**Critères d'acceptation :**

- **AC-SIN-003-A** : La transmission DOIT enregistrer `transmitted_at` (date et heure), `declared_amount` (montant déclaré — immuable une fois défini), et une référence dossier assureur (optionnelle).
- **AC-SIN-003-B** : `transmitted_at` est immuable une fois défini.
- **AC-SIN-003-C** : La transition DOIT être tracée dans le journal d'audit.

#### US-SIN-004 — Ajouter des notes de suivi

**En tant que** `Persona-Syndic`,
**je veux** ajouter des notes de suivi horodatées sur un sinistre,
**afin de** conserver une trace des échanges avec l'assureur et des étapes du dossier.

**Critères d'acceptation :**

- **AC-SIN-004-A** : Les notes sont en ajout uniquement (append-only) — chaque note porte `author_id`, `created_at`, `content`.
- **AC-SIN-004-B** : Les notes sont visibles par Syndic et Conseil.
- **AC-SIN-004-C** : Une note ne peut pas être modifiée ou supprimée après création.

#### US-SIN-005 — Clôturer un sinistre

**En tant que** `Persona-Syndic`,
**je veux** clôturer un sinistre avec le résultat final de l'instruction,
**afin de** enregistrer le remboursement ou le refus et clore le dossier.

**Critères d'acceptation :**

- **AC-SIN-005-A** : La clôture DOIT enregistrer le résultat : `REIMBURSED`, `PARTIAL`, `REFUSED`, `WITHDRAWN`.
- **AC-SIN-005-B** : Si résultat `REIMBURSED` ou `PARTIAL` : `reimbursement_amount`, `reimbursement_date` et `deductible` DOIVENT être saisis.
- **AC-SIN-005-C** : Ces champs financiers sont immuables une fois définis.
- **AC-SIN-005-D** : La clôture DOIT être tracée dans le journal d'audit.
- **AC-SIN-005-E** : Un sinistre clôturé reste consultable avec son historique complet.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil |
|---|---|---|---|
| NFR-Perf-001 | Performance | Chargement de la liste des sinistres (p95) | ≤ 200 ms |
| NFR-Legal-001 | Conformité | Alerte délai de déclaration | Affichée dans 100 % des cas de dépassement |
| NFR-Audit-001 | Traçabilité | Transitions de statut et champs financiers | 100 % tracés dans le journal d'audit |
| NFR-Sec-001 | Sécurité | Montants visibles par rôle | Syndic + Conseil uniquement |

---

## 7. Hors scope

- Gestion des assurances privées des copropriétaires (parties privatives).
- Téléclaration directe à l'assureur via API externe.
- Expertise contradictoire et suivi des contre-expertises.
- Calcul automatique de la franchise depuis les conditions de police.
- Gestion des sinistres multirisque locaux (hors immeuble).

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Sinistres transmis dans le délai légal | ≥ 95 % | Continu |
| Sinistres avec pièces jointes | ≥ 80 % | À 3 mois |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-SIN-001 | ~~Les délais légaux par type sont-ils fixes ou paramétrables ?~~ | Damien | ✅ Résolu — délai paramétrable par type avec valeur par défaut (AC-SIN-002-B). |
| Q-SIN-002 | ~~Un copropriétaire doit-il voir les sinistres qui impactent son lot ?~~ | Damien | ✅ Résolu — non. Redirection via module Signalements. |
| Q-SIN-003 | ~~Remboursement → alimentation comptabilité ?~~ | Damien | ✅ Résolu — hors scope, lot futur post-stabilisation Budget. |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-SIN-001 | Must | AC-SIN-001-A à D | Sinistres (C — draft) |
| US-SIN-001b | Must | AC-SIN-001b-A à D | Sinistres (transition → declared) |
| US-SIN-002 | Must | AC-SIN-002-A à C | Sinistres (R — délai légal) |
| US-SIN-003 | Must | AC-SIN-003-A à C | Sinistres (transition → transmitted) |
| US-SIN-004 | Must | AC-SIN-004-A à C | Notes de suivi (C append-only) |
| US-SIN-005 | Must | AC-SIN-005-A à E | Sinistres (transition → closed) |
| US-SIN-006 | Should | — | Documents liés (C / R) |
| US-SIN-007 | Could | — | Export récapitulatif |
