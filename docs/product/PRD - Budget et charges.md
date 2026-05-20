---
product: Budget et charges
version: 1.0.0
owner: Damien
status: validé
last_updated: 2026-05-21
references:
  - EP-004
  - LEGAL.md §4
  - LEGAL.md §11
  - ADR-006
  - ADR-009
llm_directives:
  temperature: 0.2
  persona: >
    Tu es un agent développeur senior travaillant sur la feature
    « Budget et charges » de La Ruche. Tu DOIS respecter les contraintes
    techniques et les critères d'acceptation sans dérogation.
    Tu NE DOIS PAS implémenter de fonctionnalités listées dans Hors scope
    (comptabilité en partie double, journaux, grand-livre, balance, paiement
    en ligne, recouvrement judiciaire des impayés).
    Les lignes budgétaires PEUVENT porter un code du plan comptable
    (arrêté 14/03/2005) — ce code est optionnel mais requis pour générer
    les documents comptables légaux (annexes I/II/III).
    Un appel de fonds émis (status = 'issued') est immuable — ADR-009 Pattern A.
    Toute correction d'un appel émis DOIT passer par un avoir (nouvelle ligne
    levy_call à montant négatif — Pattern C). Ne jamais modifier l'original.
    Un budget approuvé est immuable — un rectificatif crée un nouveau budget
    avec parent_budget_id référençant l'approuvé — ADR-009 Pattern B.
    Le snapshot des tantièmes DOIT être figé au moment de la génération
    de chaque appel de fonds (ADR-006).
  modification_policy: >
    Ne modifier ce fichier que si une évolution de la feature est ABSOLUMENT
    nécessaire, et EXCLUSIVEMENT après validation explicite d'un PRD additionnel.
    Si une modification est nécessaire : inline, succincte, dans une section
    existante. Si aucune section ne convient, requérir une validation utilisateur.
    Le corps du document ne contient aucune référence technique externe
    (ADR, DDR, numéro de feature ou de ticket).
---

# PRD — Budget et charges

> **Statut** — validé
> **Dernière modification** — 21 mai 2026
> **Références** — `LEGAL.md §4`, `LEGAL.md §11`

---

## 1. Vue d'ensemble

### Objectif

Permettre au syndic bénévole de **planifier, émettre et suivre les appels de fonds** de sa copropriété : budget prévisionnel par exercice, appels de fonds au prorata des tantièmes, suivi du fonds travaux obligatoire, et clôture annuelle par l'arrêté de comptes.

### Pourquoi

Le budget et les charges représentent le cœur de la relation contractuelle entre la copropriété et ses copropriétaires. Trois risques majeurs :

- appel de fonds émis avec des tantièmes incorrects ou obsolètes, exposant la copropriété à des contestations ;
- sous-dotation du fonds travaux en dessous du seuil légal, exposant le syndic à une mise en cause personnelle ;
- modification rétroactive d'un appel émis, créant un litige comptable non traçable.

### KPIs

- Taux d'exercices avec Σ appels émis ≥ 98 % du budget prévisionnel.
- Taux de copropriétés avec fonds travaux en conformité légale (≥ seuil applicable).
- Taux d'arrêtés de comptes produits dans les 6 mois suivant la clôture de l'exercice.

---

## 2. Contraintes techniques

> **CRITICAL** — Ces contraintes sont immuables. Tout agent ou développeur DOIT les respecter sans exception.

### Stack

- **Backend** : Rust (Axum / SQLx / PostgreSQL)
- **Frontend** : Vue 3 Composition API + TypeScript
- **Auth** : JWT / tokens opaques

### Immuabilité comptable — Appels de fonds

- Un appel de fonds au statut `draft` est modifiable librement.
- Un appel de fonds au statut `issued` est **immuable en totalité** — aucun `PUT`/`PATCH`/`DELETE` autorisé.
- Toute correction d'un appel émis DOIT créer une nouvelle ligne `levy_call` de montant négatif (avoir — Pattern C), jamais modifier la ligne originale.
- L'émission (`draft` → `issued`) DOIT écrire une entrée dans `audit_log` via `db::audit_log::record()` avec snapshot des données figées.

### Immuabilité comptable — Budget

- Un budget au statut `draft` est modifiable librement.
- Un budget au statut `approved` est immuable — tout rectificatif voté en AG crée un **nouveau budget** avec `parent_budget_id` référençant le budget approuvé. L'ancien reste consultable.
- La contrainte `UNIQUE(building_id, fiscal_year)` DOIT être levée pour autoriser deux budgets (approuvé + rectificatif) sur le même exercice fiscal.

### Snapshot des tantièmes

- Au moment de la génération d'un appel de fonds, le snapshot des tantièmes en vigueur DOIT être figé sur chaque ligne de l'appel.
- Si Σ tantièmes actifs ≠ total déclaré sur la clé concernée, la génération DOIT être bloquée (422 avec détail de la clé en erreur).

### Fonds travaux (art. 14-2-1 loi 1965 — en vigueur 01/01/2023)

- **Sans PPT adopté** : cotisation annuelle ≥ 5 % du budget prévisionnel.
- **Avec PPT adopté** : cotisation ≥ 2,5 % du montant des travaux prévus dans le plan ET ≥ 5 % du budget prévisionnel.
- L'Application DOIT calculer et afficher le seuil légal applicable, et alerter (non bloquant) si la dotation saisie est insuffisante.
- Le fonds travaux est **non remboursable** lors d'une mutation — rappel visuel obligatoire dans les données de mutation.

### Patterns interdits

- Aucun `PUT`/`PATCH` sur un `levy_call` au statut `issued`.
- Aucune suppression de `levy_call` émis.
- Aucun recalcul rétroactif d'un appel émis — même après correction de tantièmes.
- Aucune génération d'appel si la clé concernée est en état d'intégrité ⚠️ (tantièmes incohérents — cf. PRD Registre de la copropriété).
- Ne pas utiliser `unwrap()` / `expect()` dans le code backend — utiliser `Result<T, E>`.

---

## 3. Profils utilisateurs

### Personas

| ID | Persona | Besoin principal | Attente clé |
|---|---|---|---|
| Persona-Syndic | Syndic bénévole | Construire le budget, générer et suivre les appels | Alerte préventive sur les incohérences et le fonds travaux |
| Persona-Conseil | Conseil syndical | Contrôler le budget et l'exécution des charges | Visibilité lecture sur les données budgétaires |
| Persona-Copro | Copropriétaire | Consulter ses charges personnelles | Comprendre ses appels et son solde |
| Persona-Locataire | Locataire | — | Hors périmètre (charges directes inexistantes) |

### User stories de haut niveau

- En tant que `Persona-Syndic`, je veux créer un budget prévisionnel par exercice pour planifier les charges de l'année.
- En tant que `Persona-Syndic`, je veux générer les appels de fonds automatiquement au prorata des tantièmes pour ne pas faire d'erreur de calcul.
- En tant que `Persona-Syndic`, je veux vérifier que la dotation fonds travaux est conforme avant d'émettre les appels.
- En tant que `Persona-Syndic`, je veux produire l'arrêté de comptes pour clôturer l'exercice et le soumettre au vote AG.
- En tant que `Persona-Copro`, je veux consulter mes appels de fonds et mon solde pour comprendre ce que je dois.

### Matrice CRUD

| Domaine | Syndic | Conseil | Copropriétaire | Locataire |
|---|---|---|---|---|
| Budget prévisionnel | `C/R/U/D¹` | `R` | `R²` | `X` |
| Lignes budgétaires | `C/R/U/D¹` | `R` | `R²` | `X` |
| Appels de fonds | `C/R/U¹/D¹` | `R` | `R¹` | `X` |
| Fonds travaux | `C/R/U` | `R` | `R²` | `X` |
| Arrêté de comptes | `C/R` | `R` | `R¹` | `X` |
| Relevé individuel de charges | `R` | `R` | `R¹` | `X` |

> ¹ Modifiable / supprimable uniquement au statut `draft`. Interdit après approbation / émission. Pour appels de fonds, arrêté de comptes et relevé : `R¹` = copropriétaire consulte uniquement ses propres lots.
> ² Lecture des données générales uniquement (budget approuvé, lignes, dotation fonds travaux) — aucune donnée nominative des autres copropriétaires.

### Règles de sécurité

- Un copropriétaire NE DOIT PAS voir les charges des autres lots.
- Les formulaires DOIVENT masquer les champs figés dès l'émission — aucun champ `issued` éditable côté frontend.

---

## 4. Cycle de vie

### Budget

| État | Description |
|---|---|
| `draft` | En cours de construction — modifiable |
| `approved` | Approuvé en AG — immuable |
| `closed` | Exercice clôturé (arrêté de comptes produit) |

**Rectificatif :** Un budget rectificatif voté en AG après approbation crée un **nouveau budget** avec `parent_budget_id` → budget approuvé.

### Appel de fonds

| État | Description |
|---|---|
| `draft` | Généré, non encore émis — modifiable |
| `issued` | Émis — immuable en totalité |

**Avoir :** Correction d'un appel émis = nouvelle ligne `levy_call` de montant négatif.

---

## 5. Exigences fonctionnelles

### MoSCoW

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Must | US-BC-001 | Créer un budget prévisionnel | Par exercice fiscal, avec lignes par clé |
| Must | US-BC-002 | Gérer les lignes budgétaires | CRUD par clé de répartition tant que statut = `draft` |
| Must | US-BC-003 | Générer les appels de fonds | Calcul prorata tantièmes, snapshot figé |
| Must | US-BC-004 | Émettre les appels de fonds | Transition `draft` → `issued`, blocage si clé ⚠️ |
| Must | US-BC-005 | Gérer le fonds travaux | Dotation, alerte seuil légal, non-remboursabilité |
| Must | US-BC-006 | Consulter le relevé individuel de charges | Par lot, par exercice, avec avoirs |
| Must | US-BC-007 | Corriger un appel émis (avoir) | Nouvelle ligne montant négatif — Pattern C |
| Should | US-BC-008 | Produire l'arrêté de comptes | Comparatif prévu/réalisé par clé |
| Should | US-BC-009 | Créer un budget rectificatif | Nouveau budget avec `parent_budget_id` — Pattern B |

### User stories détaillées

#### US-BC-001 — Créer un budget prévisionnel

**En tant que** `Persona-Syndic`,
**je veux** créer un budget prévisionnel pour un exercice fiscal,
**afin de** disposer d'un plan annuel des charges à soumettre au vote en AG.

**Critères d'acceptation** :

- **AC-BC-001-A** : Un budget DOIT être rattaché à une copropriété et un exercice fiscal (année).
- **AC-BC-001-B** : Un seul budget actif (`draft` ou `approved`) par exercice est autorisé — la création d'un second DOIT être refusée avec 422 (sauf budget rectificatif via US-BC-009).
- **AC-BC-001-C** : Le budget DOIT afficher le total prévisionnel calculé à partir de la somme de ses lignes.
- **AC-BC-001-D** : Si les clés de répartition sont en état ⚠️ au moment de la création du budget, un avertissement non bloquant DOIT être affiché — la création reste possible. Le blocage ⚠️ s'applique uniquement à la génération des appels de fonds (AC-BC-003-B).
- **AC-BC-001-E** : Le budget DOIT afficher l'état du fonds travaux (conforme / insuffisant / non renseigné) dès sa création.

#### US-BC-002 — Gérer les lignes budgétaires

**En tant que** `Persona-Syndic`,
**je veux** créer et modifier les lignes de mon budget,
**afin de** ventiler les charges par clé de répartition et par poste.

**Critères d'acceptation** :

- **AC-BC-002-A** : Une ligne budgétaire DOIT comporter : libellé, clé de répartition, montant annuel prévisionnel.
- **AC-BC-002-B** : Modification et suppression DOIVENT être possibles uniquement si le budget est au statut `draft`.
- **AC-BC-002-C** : Toute tentative de modification sur un budget `approved` DOIT retourner 403.
- **AC-BC-002-D** : Le total par clé DOIT se recalculer automatiquement à chaque modification de ligne.
- **AC-BC-002-E** : Une ligne sans clé de répartition associée DOIT être refusée à la sauvegarde.
- **AC-BC-002-F** : Chaque ligne budgétaire PEUT être associée à un code du plan comptable (arrêté 14/03/2005) via un champ `account_code` optionnel. Ce code est requis pour la génération des documents comptables légaux (US-BC-012).

#### US-BC-003 — Générer les appels de fonds

**En tant que** `Persona-Syndic`,
**je veux** générer les appels de fonds à partir du budget approuvé,
**afin d'obtenir** les montants dus par chaque lot au prorata de ses tantièmes.

**Critères d'acceptation** :

- **AC-BC-003-A** : La génération DOIT être impossible si le budget n'est pas au statut `approved` — retour 422.
- **AC-BC-003-B** : La génération DOIT être bloquée si l'une des clés référencées dans le budget est en état ⚠️ — retour 422 avec détail de la clé en erreur.
- **AC-BC-003-C** : Pour chaque lot actif et chaque clé, le montant est calculé : `montant_ligne × tantième_lot / total_clé` — le snapshot des tantièmes au moment de la génération DOIT être figé sur la ligne d'appel.
- **AC-BC-003-D** : Les appels sont générés au statut `draft` — non encore émis ni opposables.
- **AC-BC-003-E** : Le total des appels générés par clé DOIT être égal au montant de la ligne budgétaire (contrôle d'écart ≤ 1 centime pour les arrondis).
- **AC-BC-003-F** : L'appel de fonds DOIT supporter les périodicités : trimestrielle (défaut légal — art. 35 décret 1967), semestrielle, annuelle, exceptionnelle (résolution AG).

#### US-BC-004 — Émettre les appels de fonds

**En tant que** `Persona-Syndic`,
**je veux** émettre les appels de fonds pour les rendre opposables aux copropriétaires,
**afin de** déclencher l'obligation de paiement.

**Critères d'acceptation** :

- **AC-BC-004-A** : L'émission DOIT être une transition explicite `draft` → `issued`.
- **AC-BC-004-B** : Un appel émis DOIT être immuable — toute tentative de modification DOIT retourner 403.
- **AC-BC-004-C** : L'émission DOIT écrire une entrée dans `audit_log` avec snapshot des données figées.
- **AC-BC-004-D** : Un appel `issued` DOIT afficher la date d'émission et la date d'échéance.
- **AC-BC-004-E** : L'émission groupée (tous les appels d'un même trimestre) DOIT être possible en une seule action.

#### US-BC-005 — Gérer le fonds travaux

**En tant que** `Persona-Syndic`,
**je veux** saisir et vérifier la dotation annuelle au fonds travaux,
**afin de** respecter l'obligation légale et en informer les copropriétaires.

**Critères d'acceptation** :

- **AC-BC-005-A** : La dotation fonds travaux DOIT être saisie comme une ligne budgétaire dédiée (type `fonds_travaux`).
- **AC-BC-005-B** : L'Application DOIT calculer et afficher le seuil légal minimum applicable : 5 % du budget prévisionnel (sans PPT) ou 2,5 % du montant PPT + 5 % du budget (avec PPT adopté) — conformément à art. 14-2-1 loi 1965 (en vigueur 01/01/2023).
- **AC-BC-005-C** : Si la dotation saisie est inférieure au seuil légal, une alerte DOIT s'afficher — non bloquante (le syndic reste responsable de la décision AG).
- **AC-BC-005-D** : Le cumul du fonds travaux DOIT être affiché sur la fiche copropriété (total cumulé depuis création).
- **AC-BC-005-E** : L'interface DOIT rappeler que les sommes versées au fonds travaux sont **non remboursables** en cas de mutation de lot.

#### US-BC-006 — Consulter le relevé individuel de charges

**En tant que** `Persona-Copro`,
**je veux** consulter le détail de mes charges sur un exercice,
**afin de** comprendre ce que je dois et vérifier les montants appelés.

**Critères d'acceptation** :

- **AC-BC-006-A** : Le relevé DOIT afficher par lot et par exercice : tous les appels émis, les montants, les dates d'échéance et le solde.
- **AC-BC-006-B** : Le relevé DOIT être filtrable par exercice fiscal.
- **AC-BC-006-C** : Un copropriétaire NE DOIT voir que ses propres lots — aucun accès aux charges des autres.
- **AC-BC-006-D** : Le relevé DOIT afficher les avoirs (corrections) distinctement des appels initiaux, avec référence à l'appel d'origine.

#### US-BC-007 — Corriger un appel émis (avoir)

**En tant que** `Persona-Syndic`,
**je veux** corriger un appel de fonds déjà émis,
**afin de** rectifier une erreur sans altérer la traçabilité comptable.

**Critères d'acceptation** :

- **AC-BC-007-A** : La correction DOIT créer une nouvelle ligne `levy_call` de montant négatif (avoir) — l'original reste intact et immuable.
- **AC-BC-007-B** : L'avoir DOIT référencer l'appel d'origine via un champ dédié (ex. : `credit_note_for`).
- **AC-BC-007-C** : La création d'un avoir DOIT afficher un avertissement rappelant les implications comptables.
- **AC-BC-007-D** : L'avoir DOIT être tracé dans `audit_log`.
- **AC-BC-007-E** : Le relevé individuel DOIT afficher l'avoir en déduction de l'appel concerné.

#### US-BC-008 — Produire l'arrêté de comptes

**En tant que** `Persona-Syndic`,
**je veux** produire l'arrêté de comptes de l'exercice clos,
**afin de** présenter en AG le comparatif prévu/réalisé et déclencher la régularisation.

**Critères d'acceptation** :

- **AC-BC-008-A** : L'arrêté DOIT comparer pour chaque clé et chaque ligne : montant prévu vs montant réellement appelé.
- **AC-BC-008-B** : L'arrêté DOIT calculer le trop-perçu ou l'insuffisance par lot.
- **AC-BC-008-C** : L'arrêté ne peut être produit que si le budget est au statut `approved` ou `closed`.
- **AC-BC-008-D** : L'arrêté est en lecture seule après production — aucune modification.

#### US-BC-009 — Créer un budget rectificatif

**En tant que** `Persona-Syndic`,
**je veux** créer un budget rectificatif voté en AG,
**afin de** corriger le budget approuvé sans altérer la traçabilité.

**Critères d'acceptation** :

- **AC-BC-009-A** : Le budget rectificatif DOIT référencer le budget approuvé via `parent_budget_id`.
- **AC-BC-009-B** : Le budget approuvé DOIT rester consultable et non modifié.
- **AC-BC-009-C** : Un seul budget rectificatif actif par exercice est autorisé.
- **AC-BC-009-D** : La création DOIT afficher un avertissement rappelant qu'un budget rectificatif nécessite un vote en AG (art. 24 loi 1965).
- **AC-BC-009-E** : Le budget rectificatif DOIT recalculer et afficher le seuil légal fonds travaux applicable sur la base de son propre total prévisionnel (art. 14-2-1 loi 1965).
- **AC-BC-009-F** : Le budget rectificatif DOIT pouvoir référencer une résolution AG via `resolution_id` (optionnel dans ce lot) pour tracer la décision de vote.

---

## 6. Exigences fonctionnelles complémentaires — Plan comptable et documents légaux

> Ces user stories complètent les exigences Must pour la production de documents comptables annuels obligatoires (arrêté 14/03/2005). La génération et le stockage des PDF s'appuient sur le module Documents.

### MoSCoW complémentaire

| Priorité | Ref | Action | Attendu produit |
|---|---|---|---|
| Should | US-BC-010 | Consulter le budget (Conseil et Copropriétaires) | Lecture seule — données budgétaires générales |
| Should | US-BC-011 | Associer un code plan comptable | Code `account_code` par ligne budgétaire, référentiel intégré |
| Should | US-BC-012 | Générer les documents comptables légaux | Annexe I (arrêté de comptes), Annexe II (compte individuel), Annexe III (note budgétaire) |

### User stories complémentaires détaillées

#### US-BC-010 — Consulter le budget (Conseil syndical et copropriétaires)

**En tant que** `Persona-Conseil` ou `Persona-Copro`,
**je veux** consulter le budget prévisionnel et les lignes budgétaires en lecture seule,
**afin de** contrôler ou comprendre la gestion financière de la copropriété.

**Critères d'acceptation** :

- **AC-BC-010-A** : `Persona-Conseil` DOIT pouvoir consulter le budget actif (draft ou approved) et tous les budgets des exercices passés.
- **AC-BC-010-B** : La vue DOIT afficher le total prévisionnel par clé de répartition et le total général.
- **AC-BC-010-C** : `Persona-Conseil` et `Persona-Copro` NE DOIVENT PAS pouvoir créer, modifier ni supprimer de ligne budgétaire — retour 403 en cas de tentative.
- **AC-BC-010-D** : `Persona-Copro` DOIT pouvoir consulter le budget approuvé en lecture — lignes budgétaires générales, total par clé, dotation fonds travaux. Aucune donnée nominative des autres copropriétaires n'est accessible.

#### US-BC-011 — Associer un code plan comptable aux lignes budgétaires

**En tant que** `Persona-Syndic`,
**je veux** associer un code du plan comptable des syndicats à chaque ligne budgétaire,
**afin de** générer des documents comptables conformes aux annexes de l'arrêté du 14/03/2005.

**Critères d'acceptation** :

- **AC-BC-011-A** : L'Application DOIT intégrer un référentiel des comptes du plan comptable des syndicats (arrêté 14/03/2005) : libellé + code (ex. : `602`, `6152`, `7320`, `10300`…) — ce référentiel est en lecture seule, non modifiable par le syndic.
- **AC-BC-011-B** : Le champ `account_code` sur chaque ligne budgétaire est optionnel tant que le budget est au statut `draft`, mais DOIT être renseigné pour déclencher la génération des documents comptables légaux (US-BC-012).
- **AC-BC-011-C** : Si des lignes budgétaires sans `account_code` existent lors de la tentative de génération (US-BC-012), une alerte DOIT lister les lignes concernées — non bloquante (le syndic peut générer un document partiel).
- **AC-BC-011-D** : Le référentiel DOIT être pré-renseigné au démarrage avec les codes les plus courants pour une copropriété standard (entretien, assurance, honoraires, charges diverses, fonds travaux).

#### US-BC-012 — Générer les documents comptables légaux

**En tant que** `Persona-Syndic`,
**je veux** générer les documents comptables annuels obligatoires au format légal,
**afin de** les soumettre à l'approbation en AG et les déposer dans l'espace documentaire sécurisé.

**Critères d'acceptation** :

- **AC-BC-012-A** : L'Application DOIT générer l'**Annexe I** (arrêté de comptes) conforme au modèle de l'arrêté 14/03/2005 : charges par code comptable, produits, solde.
- **AC-BC-012-B** : L'Application DOIT générer l'**Annexe II** (compte individuel de copropriétaire) : appels de fonds émis par lot, avoirs, solde à la date de clôture.
- **AC-BC-012-C** : L'Application DOIT générer la **note d'information budgétaire** (Annexe III) : budget prévisionnel vs réalisé par catégorie de charges.
- **AC-BC-012-D** : Les documents générés DOIVENT être au format PDF et déposés automatiquement dans le module Documents (catégorie `COMPTABILITE` pour les documents généraux, catégorie `APPEL_FONDS` pour l'Annexe II nominative) avec horodatage et `user_id` générateur.
- **AC-BC-012-E** : La génération nécessite que le budget soit au statut `approved` ou `closed` — retour 422 si `draft`.
- **AC-BC-012-F** : Un document déjà généré pour le même exercice DOIT déclencher le workflow de remplacement de document critique.

---

## 7. Exigences non fonctionnelles

| ID | Catégorie | Exigence | Seuil mesurable |
|---|---|---|---|
| NFR-Perf-001 | Performance | Génération d'appels de fonds | ≤ 500 ms (p95) pour 200 lots |
| NFR-Perf-002 | Performance | Chargement du relevé individuel | ≤ 300 ms (p95) |
| NFR-Legal-001 | Conformité | Seuil fonds travaux | Alerte affichée si dotation < seuil légal — 100 % des cas |
| NFR-Integ-001 | Intégrité | Snapshot tantièmes à génération | Figé sur chaque ligne d'appel — 0 exception |
| NFR-Sec-001 | Sécurité | Immutabilité appels émis | 0 modification autorisée après `issued` — retour 403 systématique |
| NFR-Audit-001 | Traçabilité | Émissions et avoirs | 100 % tracés dans le journal d'audit |

---

## 8. Hors scope

> **CRITICAL** — Ne pas implémenter dans ce lot.

- Comptabilité en partie double (journaux, grand-livre, balance, rapprochement bancaire) — dépasse le périmètre bénévole.
- Validation de la conformité du plan comptable par un expert-comptable ou commissaire aux comptes — responsabilité du syndic.
- Paiement en ligne des appels de fonds.
- Recouvrement des charges impayées (mise en demeure, procédures judiciaires).
- Plan Pluriannuel de Travaux.
- Import/export vers logiciel comptable externe.
- Catégories d'appels de fonds liés à la délégation conseil syndical — lot futur.
- Gestion de l'IBAN du compte séparé et des flux bancaires — hors périmètre logiciel.

---

## 9. Métriques de succès

| Métrique | Cible | Délai |
|---|---|---|
| Taux de budgets avec fonds travaux conforme au seuil légal | ≥ 90 % | À 3 mois |
| Taux d'appels générés sans erreur d'arrondi (écart ≤ 1 centime) | 100 % | Continu |
| Taux d'avoirs tracés dans le journal d'audit | 100 % | Continu |
| Délai moyen entre clôture exercice et production arrêté de comptes | < 90 jours | À 6 mois |

---

## 10. Questions ouvertes

| # | Question | Owner | Condition de révision |
|---|---|---|---|
| Q-BC-001 | L'arrêté de comptes doit-il être accessible aux copropriétaires en lecture avant le vote AG, ou uniquement après approbation en AG ? | Damien | Avant implémentation US-BC-008 |
| Q-BC-002 | Les nouvelles catégories d'appels (PPT, délégation CS) introduites par l'art. 35 décret v6.0 (en vigueur 25/12/2025) sont listées en Hors scope. Quand ce lot sera préparé, faudra-t-il une catégorie d'appel et un code plan comptable distincts ? | Damien | Avant implémentation du lot concerné |
| Q-BC-003 | Comment distinguer visuellement un appel de fonds ordinaire d'un appel exceptionnel (résolution AG) dans le relevé individuel ? | Damien | Avant implémentation US-BC-003 |

---

## Appendice A — Index de traçabilité

| US | Priorité | ACs | Domaine CRUD |
|---|---|---|---|
| US-BC-001 | Must | AC-BC-001-A à E | Budget prévisionnel (C) |
| US-BC-002 | Must | AC-BC-002-A à F | Lignes budgétaires (C/U/D — draft seulement) |
| US-BC-003 | Must | AC-BC-003-A à F | Appels de fonds (C — génération) |
| US-BC-004 | Must | AC-BC-004-A à E | Appels de fonds (transition draft→issued) |
| US-BC-005 | Must | AC-BC-005-A à E | Fonds travaux (C/R/U) |
| US-BC-006 | Must | AC-BC-006-A à D | Relevé charges (R) |
| US-BC-007 | Must | AC-BC-007-A à E | Avoir sur appel (C) |
| US-BC-008 | Should | AC-BC-008-A à D | Arrêté de comptes (C/R) |
| US-BC-009 | Should | AC-BC-009-A à F | Budget rectificatif (C) |
| US-BC-010 | Should | AC-BC-010-A à D | Consultation budget (R — Conseil + Copropriétaires) |
| US-BC-011 | Should | AC-BC-011-A à D | Codes plan comptable (C/U sur lignes) |
| US-BC-012 | Should | AC-BC-012-A à F | Documents comptables légaux (Génération PDF) |
