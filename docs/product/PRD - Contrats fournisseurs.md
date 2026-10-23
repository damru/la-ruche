---
product: Contrats fournisseurs
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-22
references:
  - LEGAL.md §5
  - LEGAL.md §10
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Contrats fournisseurs » de La Ruche. Tu DOIS respecter les
    contraintes techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope
    (signature électronique, envoi automatique de courriers, comparaison tarifaire).
    Un contrat actif est immuable sur ses champs financiers et légaux.
    Tout amendement DOIT créer un nouveau contrat avec predecessor_id.
    Les rappels sont calculés automatiquement à partir des dates et durées de préavis.
---

# PRD — Contrats fournisseurs

## 1. Vue d'ensemble

### Objectif

Permettre au syndic bénévole de **gérer le cycle de vie des contrats fournisseurs** de sa copropriété : création, suivi des échéances, gestion des reconductions tacites, alertes de préavis et archivage. Le contrat d'assurance multirisque immeuble (obligatoire) bénéficie d'une catégorie dédiée et d'un espace de gestion de pièces justificatives intégré.

### Pourquoi

Trois risques concrets pour un syndic bénévole :

- reconduction tacite silencieuse d'un contrat sous-optimal, faute d'alerte de préavis ;
- perte de traçabilité lors d'un changement de syndic (historique des avenants) ;
- non-renouvellement de l'assurance MRI, exposant la copropriété à un défaut de couverture obligatoire (art. 9-1 loi 1965).

### KPIs

- Taux de contrats actifs avec préavis configuré ≥ 80 % à 3 mois.
- Taux d'alertes de préavis acquittées dans les 15 jours ≥ 90 %.
- Zéro copropriété sans contrat d'assurance MRI actif.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL)
- **Frontend** : Vue 3 Composition API + TypeScript
- **Auth** : JWT / tokens opaques

### Immuabilité des contrats actifs

- Un contrat au statut `draft` est modifiable librement.
- Un contrat au statut `active` est immuable sur les champs financiers et légaux (`supplier_id`, `amount`, `start_date`, `end_date`, `notice_days`, `contract_type`, `payment_frequency`, `auto_renewal`) — aucun `PUT`/`PATCH` direct autorisé. Les champs descriptifs (`vendor_name`, `vendor_email`, `vendor_phone`, `subject`) restent modifiables sans passer par un avenant.
- Tout amendement sur un contrat actif DOIT créer un nouveau contrat au statut `draft` avec `predecessor_id` référençant le contrat actif. L'activation du nouvel avenant DOIT clore l'ancien en une transaction atomique.
- La clôture et la résiliation DOIVENT passer par des endpoints dédiés (`POST /{id}/activate`, `POST /{id}/terminate`).
- Chaque transition de statut DOIT être tracée dans le journal d'audit.

### Calcul des rappels

- À chaque activation ou modification d'un contrat `draft`, les dates de rappel sont (re)calculées et persistées.
- **Rappel de préavis** : `date_fin − notice_days` — si `notice_days > 0`.
- **Rappel de reconduction tacite** : si `tacit_renewal = true` et `notice_days > 0` → `date_fin − notice_days − 7 jours` (alerte anticipée).
- **Rappel d'échéance** : `date_fin`.
- Si `date_fin` est NULL (contrat sans terme), aucun rappel d'échéance ni de reconduction n'est généré.

### Intégration documentaire

- Les pièces jointes d'un contrat transitent par le module Documents (catégorie `CONTRAT`), via le pattern pre-signed URL.
- Le lien `contract_id` est préservé en cas de soft-delete du document.
- La suppression d'un contrat DOIT être refusée (422) s'il a des documents rattachés.

### Patterns interdits

- Aucun `PUT`/`PATCH` direct sur un `vendor_contract` au statut `active`.
- Aucun recalcul rétroactif des montants sur un contrat `terminated` ou `expired`.
- Aucune suppression d'un contrat ayant des avenants (`predecessor_id` référencé).
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Gérer et suivre les contrats de la copropriété | Alertes automatiques, traçabilité des avenants |
| Persona-Conseil | Conseil syndical | Consulter les contrats et leurs échéances | Visibilité lecture sans friction |

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Contrats fournisseurs | `C / R / U¹ / D¹` | `R` | `X` | `X` |
| Avenants | `C / R` | `R` | `X` | `X` |
| Rappels | `R / U²` | `R` | `X` | `X` |
| Pièces jointes contrat | `C / R / D¹` | `R` | `X` | `X` |

> ¹ Modification / suppression uniquement au statut `draft`. Un contrat `active` ne peut être modifié que via l'amendement (nouvel avenant avec `predecessor_id`).
> ² Acquittement uniquement — marquer un rappel comme traité.

### Règles de sécurité

- Les montants des contrats ne sont visibles que par `Persona-Syndic` et `Persona-Conseil`.
- Aucun copropriétaire ni locataire n'a accès aux conditions contractuelles.
- Les données contractuelles sont filtrées par `building_id` — aucun accès cross-copropriété.

---

## 4. Cycle de vie

### Contrat

```
draft ──(activation)──▶ active ──(résiliation)──▶ terminated
                           │
                           └──(échéance atteinte)──▶ expired
                           │
                           └──(amendment)──▶ nouveau draft ──(activation)──▶ active
                                                             (ancien → terminated)
```

| État | Description |
|---|---|
| `draft` | Contrat en cours de saisie — modifiable librement |
| `active` | Contrat en vigueur — immuable sur les champs clés |
| `terminated` | Résilié avant terme ou supplanté par un avenant activé |
| `expired` | Arrivé à échéance sans reconduction explicite |

**Amendement :** La modification d'un contrat actif crée un nouveau contrat `draft` avec `predecessor_id` → contrat actif. À l'activation du nouveau contrat, l'ancien passe automatiquement à `terminated`.

### Rappels

| Type | Calcul | Acquittable |
|---|---|---|
| Alerte reconduction tacite | `date_fin − notice_days − 7 j` | Oui |
| Rappel de préavis | `date_fin − notice_days` | Oui |
| Rappel d'échéance | `date_fin` | Oui |

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-CF-001 | Créer un contrat | Saisie complète : fournisseur, type, dates, montant |
| Must | US-CF-002 | Lister et consulter les contrats | Vue filtrable par statut, type, fournisseur |
| Must | US-CF-003 | Configurer les rappels | Préavis, reconduction tacite, recalcul automatique |
| Must | US-CF-004 | Acquitter un rappel | Marquer comme traité manuellement |
| Must | US-CF-005 | Résilier un contrat | Transition `active` → `terminated` avec date effective |
| Should | US-CF-006 | Amender un contrat actif | Nouvel avenant avec `predecessor_id` — Pattern B |
| Should | US-CF-007 | Rattacher des documents au contrat | Via module Documents — catégorie `CONTRAT` |
| Should | US-CF-008 | Gérer le contrat d'assurance MRI | Catégorie `ASSURANCE`, alerte défaut de couverture |

### User stories détaillées

#### US-CF-001 — Créer un contrat

**En tant que** `Persona-Syndic`,
**je veux** créer un contrat fournisseur dans l'Application,
**afin de** centraliser les engagements contractuels de la copropriété.

**Critères d'acceptation :**

- **AC-CF-001-A** : Un contrat DOIT comporter les champs obligatoires suivants : fournisseur (référence au référentiel ou nom libre), objet, type, date de début.
- **AC-CF-001-B** : Les champs optionnels sont : date de fin, montant annuel HT, périodicité de paiement (`mensuel`, `trimestriel`, `annuel`, `ponctuel`), durée de préavis en jours, clause de reconduction tacite (booléen), notes libres.
- **AC-CF-001-C** : Le contrat est créé au statut `draft` — aucun rappel n'est calculé tant que le contrat n'est pas activé.
- **AC-CF-001-D** : Le type de contrat DOIT être sélectionné parmi : `ASSURANCE`, `ENTRETIEN`, `NETTOYAGE`, `MAINTENANCE`, `AUTRE`.
- **AC-CF-001-E** : Si le type est `ASSURANCE` et que la copropriété n'a pas encore de contrat d'assurance MRI au statut `active`, l'interface DOIT afficher un bandeau d'alerte non bloquant rappelant l'obligation légale (art. 9-1 loi 1965).

#### US-CF-002 — Lister et consulter les contrats

**En tant que** `Persona-Syndic` ou `Persona-Conseil`,
**je veux** consulter la liste des contrats de la copropriété,
**afin de** avoir une vue d'ensemble des engagements en cours et à venir.

**Critères d'acceptation :**

- **AC-CF-002-A** : La liste DOIT être filtrable par statut (`draft`, `active`, `terminated`, `expired`), par type de contrat et par prestataire (nom exact).
- **AC-CF-002-B** : La liste DOIT afficher, pour chaque contrat : fournisseur, objet, type, dates, statut et le prochain rappel actif non acquitté le cas échéant. Deux vues sont proposées au choix de l'utilisateur : **cartes** (vue par défaut) et **tableau** — la carte porte déjà l'ensemble des informations du tableau à l'échelle d'une copropriété (dizaines de contrats).
- **AC-CF-002-C** : Les contrats avec au moins un rappel actif non acquitté DOIVENT être mis en évidence visuellement dans la liste.
- **AC-CF-002-D** : La vue détail DOIT afficher l'historique des avenants (chaîne `predecessor_id`) et la liste des pièces jointes rattachées. Chaque contrat de la liste DOIT donner un accès direct à sa vue détail.
- **AC-CF-002-E** : La liste et la vue détail des contrats DOIVENT être accessibles en lecture au Syndic et au Conseil syndical (art. 21 de la loi du 10 juillet 1965). Toutes les actions de modification (création, édition, activation, avenant, résiliation, suppression, rattachement de documents, acquittement de rappel) restent réservées au Syndic.

#### US-CF-003 — Configurer les rappels

**En tant que** `Persona-Syndic`,
**je veux** configurer le délai de préavis et la clause de reconduction tacite sur un contrat,
**afin de** recevoir une alerte avant que la fenêtre de résiliation ne se ferme.

**Critères d'acceptation :**

- **AC-CF-003-A** : La durée de préavis DOIT être saisissable en jours (ex. : 90 jours) sur tout contrat au statut `draft`.
- **AC-CF-003-B** : Si `tacit_renewal = true` et `notice_days > 0`, trois rappels DOIVENT être calculés à l'activation : alerte reconduction (`date_fin − notice_days − 7 j`), rappel préavis (`date_fin − notice_days`), rappel échéance (`date_fin`).
- **AC-CF-003-C** : Si `notice_days = 0` ou NULL, seul le rappel d'échéance est généré (si `date_fin` est renseignée).
- **AC-CF-003-D** : Les dates de rappel DOIVENT être recalculées automatiquement si `date_fin` ou `notice_days` est modifié sur un contrat au statut `draft`.

#### US-CF-004 — Acquitter un rappel

**En tant que** `Persona-Syndic`,
**je veux** acquitter manuellement un rappel,
**afin de** indiquer que j'en ai pris connaissance et que l'action est traitée.

**Critères d'acceptation :**

- **AC-CF-004-A** : L'acquittement DOIT être possible par rappel individuel — un acquittement ne désactive pas les autres rappels du même contrat.
- **AC-CF-004-B** : Un rappel acquitté DOIT afficher la date et l'auteur de l'acquittement.
- **AC-CF-004-C** : L'acquittement DOIT être tracé dans le journal d'audit.
- **AC-CF-004-D** : Un rappel acquitté reste visible dans la vue détail du contrat.

#### US-CF-005 — Résilier un contrat

**En tant que** `Persona-Syndic`,
**je veux** résilier un contrat actif,
**afin de** clore l'engagement contractuel et mettre fin aux rappels associés.

**Critères d'acceptation :**

- **AC-CF-005-A** : La résiliation DOIT être une action explicite via un endpoint dédié, avec une date de résiliation effective obligatoire.
- **AC-CF-005-B** : Un commentaire de résiliation (motif) DOIT être saisissable — champ optionnel.
- **AC-CF-005-C** : La résiliation DOIT passer le contrat au statut `terminated` et désactiver tous les rappels futurs non encore déclenchés.
- **AC-CF-005-D** : La résiliation DOIT être tracée dans le journal d'audit.
- **AC-CF-005-E** : Un contrat résilié reste consultable ; son historique (avenants, documents) reste accessible.

#### US-CF-006 — Amender un contrat actif

**En tant que** `Persona-Syndic`,
**je veux** amender les conditions d'un contrat actif,
**afin de** modifier les termes sans perdre la traçabilité de l'engagement original.

**Critères d'acceptation :**

- **AC-CF-006-A** : L'amendement DOIT créer un nouveau contrat au statut `draft` pré-rempli avec les valeurs du contrat actif, avec `predecessor_id` → contrat actif.
- **AC-CF-006-B** : L'activation du nouveau contrat DOIT clore l'ancien (`terminated`) en une transaction atomique.
- **AC-CF-006-C** : Un seul avenant `draft` peut coexister avec un contrat actif — toute tentative de créer un second avenant DOIT retourner 422.
- **AC-CF-006-D** : La vue détail du contrat original DOIT afficher le lien vers son avenant, et vice-versa.
- **AC-CF-006-E** : La chaîne d'avenants successifs (`predecessor_id`) N'A PAS de limite de profondeur — un avenant activé peut lui-même être amendé (Q-CF-001, tranché).

#### US-CF-007 — Rattacher des documents au contrat

**En tant que** `Persona-Syndic`,
**je veux** rattacher des documents (contrat PDF, avenant, tableau de garanties) au contrat,
**afin de** centraliser toutes les pièces liées dans l'Application.

**Critères d'acceptation :**

- **AC-CF-007-A** : Les documents DOIVENT être déposés via le module Documents — catégorie `CONTRAT`, avec `contract_id` comme référence d'entité.
- **AC-CF-007-B** : La vue détail du contrat DOIT afficher la liste des documents rattachés avec leur nom et date de dépôt.
- **AC-CF-007-C** : La suppression d'un contrat DOIT être refusée (422) si des documents rattachés existent — le syndic doit d'abord détacher ou supprimer les documents.
- **AC-CF-007-D** : La **lecture** des documents rattachés est ouverte au Syndic et au Conseil syndical (art. 21 de la loi du 10 juillet 1965 : le conseil syndical contrôle la gestion du syndic et peut prendre connaissance de toutes pièces se rapportant à cette gestion). La **gestion des rattachements** (dépôt, retrait) reste réservée au Syndic.

#### US-CF-008 — Gérer le contrat d'assurance MRI

**En tant que** `Persona-Syndic`,
**je veux** gérer le contrat d'assurance multirisque immeuble comme une entité prioritaire,
**afin de** ne jamais être en défaut de couverture obligatoire (art. 9-1 loi 1965).

**Critères d'acceptation :**

- **AC-CF-008-A** : L'Application DOIT afficher une alerte permanente (non acquittable) si aucun contrat de type `ASSURANCE` n'est au statut `active` pour la copropriété.
- **AC-CF-008-B** : La vue globale de la copropriété DOIT afficher le statut du contrat d'assurance en cours (couvert / non couvert) avec la date d'échéance.
- **AC-CF-008-C** : Pour un contrat de type `ASSURANCE`, si aucun `notice_days` n'est configuré, une valeur par défaut de 90 jours DOIT être appliquée. Cette valeur par défaut ne peut pas être abaissée en dessous de 30 jours.

---

## 6. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil mesurable |
|---|---|---|---|
| NFR-Perf-001 | Performance | Chargement de la liste des contrats (p95) | ≤ 200 ms |
| NFR-Legal-001 | Conformité | Alerte assurance MRI | Affichée dans 100 % des cas où aucun contrat `ASSURANCE` actif |
| NFR-Audit-001 | Traçabilité | Transitions de statut et acquittements | 100 % tracés dans le journal d'audit |
| NFR-Sec-001 | Sécurité | Isolation inter-copropriétés | 0 accès cross-copropriété — filtrage `building_id` systématique |

---

## 7. Hors scope

> **CRITICAL** — Ne pas implémenter dans ce lot.

- Signature électronique du contrat (probante ou simple).
- Envoi automatique du courrier de résiliation au fournisseur.
- Comparaison tarifaire ou notation des fournisseurs.
- Gestion des bons de commande ou factures liées au contrat.
- Validation formelle du contrat par le Conseil syndical (workflow d'approbation) — lot futur.
- Import de contrats existants en masse (CSV / PDF parsing).

---

## 8. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Taux de contrats actifs avec préavis configuré | ≥ 80 % | À 3 mois post-lancement |
| Taux de rappels acquittés dans les 15 jours | ≥ 90 % | Continu |
| Taux de copropriétés sans contrat d'assurance MRI actif | 0 % | Continu |

---

## 9. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| ~~Q-CF-001~~ | ~~Un contrat peut-il avoir plusieurs avenants successifs (chaîne `predecessor_id` de profondeur > 2) ?~~ | Damien | **Tranchée 2026-07-18 : oui, profondeur illimitée.** Cf. AC-CF-006-E. |
| Q-CF-002 | Les rappels sont-ils déclenchés par le module de notification transverse ou par un job autonome dédié ? | Damien | Avant implémentation US-CF-003 |
| Q-CF-003 | Le Conseil syndical doit-il pouvoir valider formellement le renouvellement ou la résiliation d'un contrat ? | Damien | Avant implémentation lot futur |
| ~~Q-CF-004~~ | ~~La liste des contrats doit-elle être présentée en tableau ou en cartes ?~~ | Damien | **Tranchée 2026-10-22 (EP-006-S06) : cartes conservées par défaut + bascule tableau.** Cf. AC-CF-002-B. |
| ~~Q-CF-005~~ | ~~Les documents rattachés à un contrat sont-ils visibles du Conseil syndical ou réservés au Syndic ?~~ | Damien | **Tranchée 2026-10-22 (EP-006-S07) : lecture Syndic + Conseil syndical (art. 21 loi 1965), écriture Syndic seul.** Cf. AC-CF-007-D. |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-CF-001 | Must | AC-CF-001-A à E | Contrats (C) |
| US-CF-002 | Must | AC-CF-002-A à E | Contrats (R) |
| US-CF-003 | Must | AC-CF-003-A à D | Rappels (C / recalcul) |
| US-CF-004 | Must | AC-CF-004-A à D | Rappels (U — acquittement) |
| US-CF-005 | Must | AC-CF-005-A à E | Contrats (transition → `terminated`) |
| US-CF-006 | Should | AC-CF-006-A à D | Avenants (C — Pattern B) |
| US-CF-007 | Should | AC-CF-007-A à D | Documents liés (C / R) |
| US-CF-008 | Should | AC-CF-008-A à C | Assurance MRI (alerte permanente) |
