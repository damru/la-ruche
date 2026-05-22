---
product: Registre de la copropriété
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-21
references:
  - EP-001
  - LEGAL.md §1
  - LEGAL.md §2
  - ADR-006
  - ADR-009
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Registre de la copropriété » de La Ruche. Tu DOIS respecter les
    contraintes techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope.
    Les tantièmes sont versionnés (valid_from / valid_to — ADR-006) :
    toute modification crée une nouvelle version, jamais un écrasement.
    Tout appel de fonds DOIT être bloqué si la somme des tantièmes actifs
    est différente du total déclaré pour la clé concernée.
  modification_policy: >
    Ne modifier ce fichier que si une évolution de la feature est ABSOLUMENT
    nécessaire, et EXCLUSIVEMENT après validation explicite d'un PRD additionnel.
    Si une modification est nécessaire : inline, succincte, dans une section
    existante. Si aucune section ne convient, requérir une validation utilisateur.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Registre de la copropriété

## 1. Vue d'ensemble

### Objectif

Permettre au syndic bénévole de **constituer, maintenir et consulter le registre fondateur de la copropriété** : fiche identité, règlement de copropriété, clés de répartition et registre des lots avec leurs tantièmes versionnés.

### Pourquoi

Le registre est la source de vérité structurelle de l'application. Sans lui :

- les appels de fonds ne peuvent pas être générés au prorata légal ;
- le quorum des AG ne peut pas être calculé ;
- les mutations de lots ne peuvent pas être tracées correctement.

Trois risques majeurs à couvrir :

- incohérence de la somme des tantièmes (Σ ≠ total déclaré), qui invalide tous les calculs de charges ;
- perte d'historique lors d'une correction de tantièmes, rendant les appels de fonds passés injustifiables ;
- confusion entre lots actifs et archivés dans les calculs de quorum.

### KPIs

- Taux de copropriétés avec Σ tantièmes = total déclaré pour toutes leurs clés actives.
- Délai moyen entre mutation d'un lot et mise à jour du registre.
- Taux d'appels de fonds bloqués pour incohérence de tantièmes (cible : 0 après mise à jour).

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception avant d'implémenter une quelconque fonctionnalité.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL)
- **Frontend** : Vue 3 Composition API + TypeScript
- **Auth** : JWT / tokens opaques

### Versioning des tantièmes

Les tantièmes sont **versionnés par lot et par clé** avec colonnes `valid_from DATE NOT NULL` et `valid_to DATE`. Une modification de tantième ne DOIT JAMAIS écraser la ligne existante — elle crée une nouvelle ligne avec la nouvelle valeur et ferme l'ancienne (`valid_to` = veille du `valid_from` de la nouvelle).

- Toute requête de calcul d'appel de fonds ou de quorum DOIT utiliser les tantièmes valides à la date du calcul.
- Les tantièmes passés restent immuables — ils sont référencés par les appels de fonds émis.

### Intégrité des tantièmes

- Pour chaque clé de répartition, `Σ tantièmes des lots actifs` DOIT être égale au `total_tantièmes` déclaré sur la clé.
- Tout appel de fonds DOIT être **bloqué** (422) si cette contrainte n'est pas satisfaite pour la clé concernée.
- L'interface DOIT afficher un indicateur d'intégrité visible (✅ / ⚠️) sur chaque clé.

### Règlement de copropriété et EDD

- La modification des tantièmes d'un lot existant ne peut légalement résulter que d'un vote à l'**unanimité** en AG (art. 11 loi 1965) ou d'une décision judiciaire. L'Application NE DOIT PAS empêcher techniquement la saisie, mais DOIT afficher un avertissement rappelant cette contrainte légale.
- L'Application stocke le document (via le module Documents) mais ne vérifie pas la concordance avec l'EDD physique — responsabilité du syndic.

### Immuabilité comptable

- Les tantièmes ayant servi à générer un appel de fonds émis (`levy_call.status = 'issued'`) NE DOIVENT PAS être rétroactivement modifiés — protégés par versioning.
- Les archives de lots (lots inactifs/archivés) NE DOIVENT PAS être supprimées — elles restent référençables dans les historiques comptables.

### Patterns interdits

- Aucun écrasement de ligne de tantième (`UPDATE unit_allocation_key SET ...`) sur un enregistrement dont `valid_from` est antérieur à aujourd'hui et qui est référencé par un appel émis.
- Aucune suppression de lot ayant des charges émises rattachées.
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Constituer et maintenir le registre exact | Être alerté des incohérences avant qu'elles bloquent les charges |
| Persona-Conseil | Conseil syndical | Consulter le registre pour contrôler la gestion | Visibilité sans pouvoir modifier |
| Persona-Copro | Copropriétaire | Connaître ses lots et ses tantièmes | Clarté sur sa part dans les charges |
| Persona-Locataire | Locataire | Connaître son lot de résidence | Information minimale de contexte |

### User stories de haut niveau

- En tant que `Persona-Syndic`, je veux créer la fiche identité de ma copropriété pour poser le socle de tout l'outil.
- En tant que `Persona-Syndic`, je veux configurer les clés de répartition pour disposer d'une base de calcul légalement conforme.
- En tant que `Persona-Syndic`, je veux gérer le registre des lots et leurs tantièmes pour que les charges soient calculées correctement.
- En tant que `Persona-Syndic`, je veux être alerté immédiatement d'une incohérence de tantièmes pour ne pas émettre un appel de fonds incorrect.
- En tant que `Persona-Copro`, je veux consulter les tantièmes de mon lot pour comprendre ma quote-part dans les charges.

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Fiche identité copropriété | `C/R/U/D` | `R` | `R` | `R` |
| Règlement de copropriété (lien doc) | `C/R/U/D` | `R` | `R` | `R` |
| Clés de répartition | `C/R/U/D` | `R` | `R` | `X` |
| Registre des lots | `C/R/U/D` | `R` | `R¹` | `X` |
| Tantièmes par lot | `C/R/U` | `R` | `R¹` | `X` |
| Historique tantièmes | `R` | `R` | `R¹` | `X` |

> ¹ Restreint aux lots de résidence de l'utilisateur.

### Règles de sécurité

- La modification d'un lot archivé DOIT être refusée (422).
- La suppression d'un lot ayant des charges émises DOIT être refusée avec 422.
- Seul `Persona-Syndic` peut modifier la fiche identité et les clés de répartition.
- Un locataire n'a pas accès au registre des lots ni aux tantièmes — même pour son lot de résidence.

---

## 4. Cycle de vie

### Lots

```
active ──(archivage)──▶ archived ──(réactivation*)──▶ active
                                      * si aucun appel émis sur la période archivée
```

| État | Description |
|---|---|
| `active` | Lot en service — inclus dans les calculs de quorum AG et les appels de fonds |
| `archived` | Lot sorti de la copropriété — exclu de tous les calculs actifs, historique intégralement conservé |

| Transition | Déclencheur | Condition | US |
|---|---|---|---|
| `active` → `archived` | Syndic — action explicite avec date effective | Confirmation obligatoire ; charges et historique propriétaire conservés | US-RC-008 |
| `archived` → `active` | Syndic — réactivation | Autorisée uniquement si aucun appel de fonds n'a été émis sur la période archivée | US-RC-008 |

### Tantièmes

Les tantièmes ne portent pas d'état propre — ils sont versionés par colonnes `valid_from` / `valid_to`. Une modification crée une nouvelle version ; l'ancienne ligne reste immuable dès lors qu'elle a servi à générer un appel émis.

---

## 5. Règles de gestion

### Clés de répartition

- Une clé est définie par : nom, total de tantièmes déclaré, description optionnelle.
- Toute copropriété DOIT avoir au minimum une clé dite « générale ».
- Une clé ne peut pas être supprimée si des lots actifs lui ont des tantièmes.

### Lots

- Un lot est identifié par : numéro de lot (unique dans la copropriété), type (habitation, parking, cave, commercial, autre), bâtiment/escalier optionnel.
- Un lot peut être **actif** ou **archivé** (après cession définitive hors copropriété ou suppression juridique).
- Un lot archivé n'entre plus dans les calculs de quorum AG ni dans les appels de fonds.

### Tantièmes

- Chaque lot peut avoir un tantième différent par clé de répartition.
- La correction d'un tantième saisi par erreur (avant tout appel de fonds généré sur ce tantième) PEUT écraser la ligne si `valid_from` = aujourd'hui et aucun appel de fonds émis ne la référence.
- Sinon, une nouvelle version est créée : `valid_from` = date effective saisie, `valid_to` de l'ancienne = veille.

---

## 6. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-RC-001 | Créer/modifier la fiche identité | Nom, adresse, SIRET, IBAN compte séparé, syndic en exercice |
| Must | US-RC-002 | Gérer le règlement de copropriété | Lien vers document déposé, date de dépôt, historique |
| Must | US-RC-003 | Gérer les clés de répartition | Création, modification, indicateur d'intégrité |
| Must | US-RC-004 | Gérer le registre des lots | CRUD lots avec type, numéro, bâtiment |
| Must | US-RC-005 | Gérer les tantièmes par lot et par clé | Versioning avec historique, avertissement légal |
| Must | US-RC-006 | Vérifier l'intégrité des tantièmes | Indicateur ✅ / ⚠️, blocage des appels si ⚠️ |
| Should | US-RC-007 | Consulter le registre (résidents) | Vue lecture par lot pour copropriétaire |
| Should | US-RC-008 | Archiver un lot | Marquage inactif, exclusion des calculs |

### User stories détaillées

#### US-RC-001 — Créer/modifier la fiche identité de la copropriété

**En tant que** `Persona-Syndic`,
**je veux** saisir et maintenir les informations identitaires de ma copropriété,
**afin de** disposer d'un registre officiel utilisable dans les documents légaux.

**Critères d'acceptation** :

- **AC-RC-001-A** : La fiche DOIT comporter : nom usuel, adresse complète, SIRET du syndicat (optionnel — tous les syndicats bénévoles n'en ont pas), IBAN du compte bancaire séparé (optionnel), syndic en exercice (nom + coordonnées).
- **AC-RC-001-B** : La fiche DOIT être rattachée à une seule entité de copropriété — création en doublon manifeste (même adresse) DOIT afficher un avertissement non bloquant.
- **AC-RC-001-C** : Le champ IBAN, s'il est renseigné, DOIT être validé selon le format ISO 13616 avant enregistrement.
- **AC-RC-001-D** : La fiche DOIT afficher la date de dernière modification et l'auteur.
- **AC-RC-001-E** : La fiche identité DOIT indiquer clairement que l'IBAN renseigné est indicatif — l'Application ne gère pas les flux bancaires.
- **AC-RC-001-F** : Le champ `numéro_rnc` (numéro d'immatriculation au Registre National des Copropriétés) DOIT être stockable, optionnel à la saisie, format alphanumérique 13 caractères — aucune validation automatique de format en backend dans ce lot.

#### US-RC-002 — Gérer le règlement de copropriété

**En tant que** `Persona-Syndic`,
**je veux** rattacher le règlement de copropriété et l'état descriptif de division à la fiche copropriété,
**afin de** les rendre accessibles aux résidents depuis l'espace sécurisé (art. 18-I-5° loi 1965).

**Critères d'acceptation** :

- **AC-RC-002-A** : Le règlement de copropriété et l'EDD DOIVENT pouvoir être déposés via le module Documents et rattachés à la fiche.
- **AC-RC-002-B** : La date du règlement en vigueur DOIT être affichée sur la fiche.
- **AC-RC-002-C** : Si aucun règlement n'est déposé, la fiche DOIT afficher un avertissement non bloquant.
- **AC-RC-002-D** : Un remplacement de document DOIT conserver l'ancienne version accessible en historique (conformément à US-DOC-007 du PRD Documents).

#### US-RC-003 — Gérer les clés de répartition

**En tant que** `Persona-Syndic`,
**je veux** créer et modifier les clés de répartition de ma copropriété,
**afin de** disposer d'une assiette de calcul conforme à l'EDD pour chaque catégorie de charges.

**Critères d'acceptation** :

- **AC-RC-003-A** : Une clé DOIT comporter : nom, total de tantièmes déclaré (entier positif), description optionnelle.
- **AC-RC-003-B** : Une clé ne peut pas être supprimée si des lots actifs lui ont des tantièmes — tentative DOIT retourner 422 avec liste des lots concernés.
- **AC-RC-003-C** : L'indicateur d'intégrité (Σ tantièmes actifs vs total déclaré) DOIT être visible sur chaque clé avec statut ✅ / ⚠️ et l'écart en tantièmes.
- **AC-RC-003-D** : Une clé en état ⚠️ DOIT bloquer la génération de tout appel de fonds la référençant (422).
- **AC-RC-003-E** : La modification du total déclaré d'une clé DOIT afficher un avertissement si des appels de fonds ont déjà été émis sur cette clé.

#### US-RC-004 — Gérer le registre des lots

**En tant que** `Persona-Syndic`,
**je veux** créer, modifier et archiver les lots de la copropriété,
**afin de** maintenir un registre exact et à jour des unités de la copropriété.

**Critères d'acceptation** :

- **AC-RC-004-A** : Un lot DOIT comporter : numéro de lot (unique dans la copropriété), type (habitation / parking / cave / commercial / autre), bâtiment/escalier optionnel.
- **AC-RC-004-B** : Un numéro de lot en doublon dans la même copropriété DOIT être refusé avec une erreur de validation explicite.
- **AC-RC-004-C** : La suppression d'un lot ayant des charges émises ou un historique propriétaire DOIT être refusée avec 422 — l'archivage est la seule option.
- **AC-RC-004-D** : Un lot archivé DOIT être exclu des calculs de quorum AG et des appels de fonds sans exception.
- **AC-RC-004-E** : La liste des lots DOIT afficher pour chacun : numéro, type, propriétaire en cours, statut (actif/archivé), indicateur de tantièmes par clé.
- **AC-RC-004-F** : La surface habitable du lot (en m²) est un champ optionnel — si elle n'est pas renseignée lors de la création, l'interface DOIT afficher un avertissement non bloquant rappelant qu'elle sera requise pour le PPT et le DPE collectif.

#### US-RC-005 — Gérer les tantièmes par lot et par clé

**En tant que** `Persona-Syndic`,
**je veux** saisir et corriger les tantièmes de chaque lot pour chaque clé de répartition,
**afin que** les appels de fonds soient calculés au prorata légal exact.

**Critères d'acceptation** :

- **AC-RC-005-A** : Chaque lot PEUT avoir une valeur de tantième différente par clé de répartition. Une valeur à 0 est autorisée pour les lots non soumis à une clé spécifique.
- **AC-RC-005-B** : Toute modification de tantième sur un lot ayant des appels de fonds émis DOIT créer une nouvelle version (`valid_from` = date saisie) sans écraser l'ancienne.
- **AC-RC-005-C** : L'interface DOIT afficher un avertissement rappelant que la modification des tantièmes requiert légalement un vote à l'unanimité en AG (art. 11 loi 1965) ou une décision judiciaire.
- **AC-RC-005-D** : L'historique des tantièmes par lot par clé DOIT être consultable avec les dates de validité.
- **AC-RC-005-E** : La saisie d'une valeur négative DOIT être refusée avec validation.

#### US-RC-006 — Vérifier l'intégrité des tantièmes

**En tant que** `Persona-Syndic`,
**je veux** être alerté immédiatement si la somme des tantièmes d'une clé ne correspond pas au total déclaré,
**afin d'éviter** que des appels de fonds incorrects soient émis.

**Critères d'acceptation** :

- **AC-RC-006-A** : L'indicateur d'intégrité DOIT se recalculer automatiquement après toute création, modification ou archivage de lot ou de tantième.
- **AC-RC-006-B** : Un écart DOIT bloquer la génération des appels de fonds (422 avec détail de l'écart en tantièmes).
- **AC-RC-006-C** : L'interface DOIT afficher l'écart numérique (ex. : « Σ = 9 988 / total déclaré = 10 000 — écart : −12 »).
- **AC-RC-006-D** : L'alerte DOIT rester visible jusqu'à correction complète — aucune validation silencieuse.

#### US-RC-007 — Consulter le registre (résidents)

**En tant que** `Persona-Copro`,
**je veux** consulter les tantièmes de mon lot et la liste des clés de répartition,
**afin de** comprendre comment mes charges sont calculées.

**Critères d'acceptation** :

- **AC-RC-007-A** : `Persona-Copro` DOIT pouvoir consulter les tantièmes de ses propres lots par clé de répartition.
- **AC-RC-007-B** : `Persona-Copro` NE DOIT PAS voir les tantièmes des lots d'autres copropriétaires.
- **AC-RC-007-C** : La consultation DOIT afficher la valeur en vigueur à la date courante.

#### US-RC-008 — Archiver un lot

**En tant que** `Persona-Syndic`,
**je veux** archiver un lot qui sort de la copropriété,
**afin de** le conserver en historique sans qu'il influe sur les calculs actifs.

**Critères d'acceptation** :

- **AC-RC-008-A** : L'archivage DOIT demander une confirmation explicite avec date effective.
- **AC-RC-008-B** : Un lot archivé NE DOIT PAS apparaître dans les calculs de quorum ni dans les appels de fonds à partir de la date effective.
- **AC-RC-008-C** : L'historique des appels de fonds passés référençant ce lot DOIT rester intact et consultable.
- **AC-RC-008-D** : Un lot archivé DOIT pouvoir être réactivé si l'archivage était erroné, sous réserve qu'aucun appel de fonds n'ait été émis sur la période archivée.

---

## 7. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil mesurable |
|---|---|---|---|
| NFR-Perf-001 | Performance | Calcul de l'indicateur d'intégrité | ≤ 200 ms (p95) pour une copropriété jusqu'à 500 lots |
| NFR-Perf-002 | Performance | Chargement de la liste des lots | ≤ 300 ms (p95) |
| NFR-Integ-001 | Intégrité | Σ tantièmes = total déclaré | Aucun appel de fonds émis sur une clé incohérente — 0 exception |
| NFR-Legal-001 | Conformité | Versioning des tantièmes | Aucun écrasement d'une ligne utilisée dans un appel émis — 0 exception |
| NFR-Sec-001 | Sécurité | Suppression de lot | Refusée si charges émises — retour 422 systématique |

---

## 8. Hors scope

> **CRITICAL** — Ne pas implémenter dans ce lot.

- Soumission automatisée au Registre National des Copropriétés (RNC) — export assisté.
- Génération de la fiche synthétique de copropriété (art. 8-2 loi 1965).
- Plan Pluriannuel de Travaux (PPT) — EP-009-S04.
- Vérification de concordance entre les tantièmes saisis et l'EDD physique.
- Gestion des divisions ou fusions de lots (réallocation judiciaire de tantièmes).
- Immatriculation RNC en ligne.

---

## 9. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Taux de copropriétés avec toutes clés en état ✅ | ≥ 95 % | À 1 mois post-onboarding |
| Taux d'appels de fonds bloqués pour incohérence résiduelle | < 1 % | Continu |
| Délai moyen de mise à jour après mutation d'un lot | < 7 jours | À 3 mois |

---

## 10. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-RC-001 | ~~Doit-on autoriser des tantièmes à 0 ?~~ **Résolu** — AC-RC-005-A autorise explicitement les tantièmes à 0 pour les lots non soumis à une clé spécifique. | — | ~~Fermé~~ |
| Q-RC-002 | Quel workflow pour les divisions de lots (lot unique → deux lots) — simple création des deux nouveaux lots ou migration des tantièmes de l'ancien ? | Damien | Avant implémentation US-RC-004/005 |
| Q-RC-003 | ~~La clé fonds travaux est-elle distincte de la clé générale ?~~ **Résolu** — la clé générale peut être réutilisée pour calculer la dotation fonds travaux annuelle (cf. PRD Budget et charges §2). | — | ~~Fermé~~ |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-RC-001 | Must | AC-RC-001-A à F | Fiche identité (C/U) |
| US-RC-002 | Must | AC-RC-002-A à D | Règlement de copropriété (C/U) |
| US-RC-003 | Must | AC-RC-003-A à E | Clés de répartition (C/U/D) |
| US-RC-004 | Must | AC-RC-004-A à F | Registre des lots (C/U/D) |
| US-RC-005 | Must | AC-RC-005-A à E | Tantièmes par lot (C/U versioning) |
| US-RC-006 | Must | AC-RC-006-A à D | Intégrité tantièmes (validation) |
| US-RC-007 | Should | AC-RC-007-A à C | Registre (R) |
| US-RC-008 | Should | AC-RC-008-A à D | Archivage lot (U) |
