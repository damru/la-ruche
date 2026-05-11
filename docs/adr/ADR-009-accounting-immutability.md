# ADR-009 — Immutabilité des données comptables et audit log

**Date :** 2026-05-04  
**Statut :** Accepté  
**Références :** LEGAL.md §11 · Art. L123-22 Code de commerce · Art. 14-1 loi 1965 · Art. 17 loi 1965

---

## Contexte

Certaines données de l'application ont une valeur juridique ou comptable une fois qu'elles ont atteint un état définitif (appels de fonds émis, votes AG, clôtures de sinistre, etc.). L'obligation de conservation comptable (art. L123-22 Code de commerce, 10 ans) et la valeur probatoire de ces actes imposent qu'elles ne puissent plus être modifiées ou supprimées rétroactivement.

Par ailleurs, la modification en place de certaines entités (contrats, budgets) introduit des incohérences sémantiques : un contrat dont le montant change en cours de validité sans trace de l'avenant, ou un budget rectificatif qui écrase le budget approuvé, perdent toute valeur de preuve.

Trois patterns ont été identifiés selon la nature de l'entité (cf. LEGAL.md §11.1).

---

## Décision

### Pattern A — Gel sur état terminal

Les champs financiers et de traçabilité des entités suivantes deviennent immuables à partir d'un statut défini :

| Entité | Déclencheur |
|---|---|
| `levy_call` | `status = 'issued'` |
| `vote` | AG `ongoing` |
| `resolution` | AG `closed` |
| `task.completion_cost` / `task.closed_at` | `status = 'closed'` |
| `claim.transmitted_at` / `claim.reimbursement_*` | Champs renseignés |
| `maintenance_log` (source `automatic`) | Toujours |

**Implémentation :**
- Fonctions `is_mutable()` / `is_deletable()` dans `domain/` appelées avant chaque mutation — pas dans les handlers.
- Endpoints dédiés pour les transitions d'état (`POST /tasks/{id}/close`, `POST /claims/{id}/transmit`, `POST /levy-calls/{id}/issue`) ; les `PATCH` génériques sont bloqués sur ces champs une fois l'état terminal atteint.
- Trigger PG défensif sur `levy_call` : bloque tout `UPDATE`/`DELETE` si `status != 'draft'`.

**Nouveau statut `draft` sur `levy_call` :**  
L'enum `levy_status` est étendu avec `'draft'` (migration requise). Un `levy_call` est créé en `draft` pour permettre une vérification avant émission officielle. Le passage à `issued` est irréversible.

### Pattern B — Clôturer + recréer

Pour `vendor_contract` et `budget`, la modification en place est sémantiquement incorrecte :
- Un avenant contractuel correspond à un nouvel acte signé à une date distincte.
- Un budget rectificatif est un acte distinct voté en AG.

**`vendor_contract` :**
- Ajout d'une colonne `predecessor_id UUID REFERENCES vendor_contract(id)` (nullable).
- Workflow « Créer un avenant » : clôture l'ancien contrat (`status = 'terminated'`, `end_date = date_avenant`), crée le nouveau avec `predecessor_id` pointant vers l'ancien.
- Un contrat ne peut plus être modifié sur `amount`, `start_date`, `end_date` une fois actif — l'UI remplace l'action « Modifier » par « Créer un avenant ».

**`budget` / `budget_line` :**
- Ajout d'une colonne `parent_budget_id UUID REFERENCES budget(id)` (nullable).
- La contrainte `UNIQUE(building_id, fiscal_year)` est remplacée par une contrainte partielle sur le budget courant actif.
- Un budget rectificatif crée une nouvelle ligne budget pour le même exercice avec `parent_budget_id` référençant le budget approuvé.
- Les `budget_line` adoptent le versioning `valid_from`/`valid_to` (cf. §Versioning temporel) pour permettre des requêtes point-dans-le-temps sur les montants budgétaires lors de la génération des appels de fonds.

### Pattern C — Append-only

Pour les corrections sur entités immuables (Pattern A), la correction s'effectue par ajout d'une nouvelle écriture — jamais par écrasement :
- Correction d'un `levy_call` émis → nouvelle ligne de montant négatif (avoir).
- Correction d'une entrée `maintenance_log` manuelle → nouvelle entrée.

---

## Versioning temporel (`valid_from` / `valid_to`)

Le pattern `valid_from`/`valid_to` (déjà établi sur `unit_allocation_key` par ADR-006) répond à une question différente de l'audit log : **"quelle était la valeur de X à la date T ?"**, sans rejouer l'historique des mutations.

Il est complémentaire à l'audit log — pas une alternative — car il ne capture ni l'acteur, ni les opérations bloquées, ni les suppressions avec leur auteur.

### Périmètre d'application

| Entité | Justification | Déjà couvert ? |
|---|---|---|
| `unit_allocation_key` | Requête "tantièmes au moment de l'appel de fonds" | ✅ ADR-006 |
| `budget_line` | Requête "montant budgété lors de la génération des appels de fonds" | ❌ → ajouté par ce ADR |
| `vendor_contract` | Le Pattern B (`predecessor_id`) + `start_date`/`end_date` constitue déjà un versioning temporel explicite ; `valid_from`/`valid_to` serait redondant | — non retenu |

### Structure `budget_line` versionnée

```sql
ALTER TABLE budget_line
    ADD COLUMN valid_from DATE NOT NULL DEFAULT CURRENT_DATE,
    ADD COLUMN valid_to   DATE;  -- NULL = ligne courante
```

Toute modification d'une `budget_line` approuvée crée une nouvelle ligne avec `valid_from = date_rectificatif` et clôt l'ancienne avec `valid_to = date_rectificatif - 1`. La ligne originale reste intacte et requêtable.

### Règle de requête temporelle

```sql
-- Montants budgétaires actifs à une date donnée
SELECT * FROM budget_line
WHERE valid_from <= :date
  AND (valid_to IS NULL OR valid_to >= :date);
```

---

## Audit log

**Frontière documentaire avec EP-017 :** le `audit_log` défini par cet ADR est un journal probant de mutations comptables et juridiques. Il ne doit pas devenir la frise générique de cycle de vie des `task` / `report`. Les événements UX et métier non probants (création de signalement, conversion en tâche, uploads, changements d’assignation lisibles en timeline) relèvent d’un flux distinct, documenté côté backlog comme `activity_log` dans `EP-017`.

### Structure

```sql
CREATE TABLE audit_log (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    building_id  UUID NOT NULL REFERENCES building(id),
    entity_type  TEXT NOT NULL,       -- 'task' | 'claim' | 'vendor_contract' | 'levy_call' | …
    entity_id    UUID NOT NULL,
    operation    TEXT NOT NULL,       -- 'UPDATE' | 'DELETE'
    changed_by   UUID REFERENCES user_account(id), -- NULL si action système
    changed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    old_values   JSONB,               -- champs modifiés seulement, valeurs avant
    new_values   JSONB                -- champs modifiés seulement, valeurs après (NULL si DELETE)
);
```

`old_values` et `new_values` ne contiennent **que les champs modifiés** — pas un snapshot complet de l'entité. Cela maintient la lisibilité de la frise chronologique et minimise le volume stocké.

**Exemple — modification du coût de clôture d'une tâche :**
```json
old_values: { "completion_cost": "1500.00" }
new_values: { "completion_cost": "1800.00" }
```

**Exemple — tentative de suppression d'un sinistre transmis :**
```json
old_values: { "status": "closed", "transmitted_at": "2026-04-10", "reimbursement_amount": "3200.00" }
new_values: null
```

### Périmètre de log

Toute opération `UPDATE` ou `DELETE` sur les champs financiers ou de traçabilité des entités critiques est loguée **avant** que le guard ne l'autorise ou non. Les opérations `INSERT` ne sont pas loguées (la ligne créée constitue sa propre trace).

### Conservation

La table `audit_log` est en **append-only** : aucune ligne ne peut être modifiée ou supprimée applicativement. Conservation 10 ans (art. L123-22 Code de commerce), alignée sur les données comptables.

### Implémentation

L'audit log est produit côté applicatif (Rust, dans les fonctions de domaine ou les handlers de transition d'état) — pas via trigger PG, pour éviter le couplage au schéma et conserver la lisibilité des champs loggés.

---

## Alternatives considérées

### A1 — Snapshot JSON complet avant/après
Stocker l'objet entier dans `old_values` / `new_values` plutôt que les seuls champs modifiés.

**Rejeté :** volume excessif, frise chronologique illisible (chaque ligne affiche tous les champs même inchangés), difficulté à extraire ce qui a réellement changé.

### A2 — Une ligne par champ modifié (`field`, `old_value`, `new_value`)
Structure relationnelle, un enregistrement par champ.

**Rejeté :** complexifie les requêtes pour reconstruire un diff lisible ; jointures multiples pour afficher un seul événement.

### A3 — Triggers PG pour toutes les entités
Audit log entièrement piloté par des triggers PostgreSQL.

**Rejeté :** couplage fort au schéma, difficulté à filtrer les champs pertinents, logique métier dispersée entre Rust et PG. Conservé uniquement comme filet de sécurité défensif sur `levy_call`.

### B1 — Versioning en place avec colonne `version`
Incrémenter un numéro de version à chaque modification, conserver les anciennes versions dans une table d'historique séparée.

**Rejeté pour les contrats :** un avenant n'est pas une "version" d'un contrat — c'est un acte juridique distinct avec sa propre date de signature et ses propres pièces justificatives. Le modèle `predecessor_id` est plus fidèle à la réalité juridique.

### B2 — `valid_from`/`valid_to` généralisé à toutes les entités critiques comme substitut de l'audit log

Appliquer le versioning temporel ADR-006 à toutes les entités comptables critiques pour remplacer l'audit log.

**Rejeté :** le versioning temporel répond à "quelle était la valeur à la date T" — il ne capture pas l'acteur (`changed_by`), les tentatives bloquées par les guards, ni les suppressions avec leur auteur. Ces deux mécanismes répondent à des questions orthogonales et sont complémentaires. Le versioning temporel est retenu uniquement sur `budget_line` où la requête point-dans-le-temps est nécessaire pour les calculs d'appels de fonds.

---

## Conséquences

### Migrations requises
- Ajout de `'draft'` à l'enum `levy_status`
- Ajout de `predecessor_id` sur `vendor_contract`
- Ajout de `parent_budget_id` sur `budget` + modification contrainte unique
- Ajout de `valid_from`/`valid_to` sur `budget_line`
- Création de la table `audit_log`

### Impact frontend
- Remplacement de « Modifier » par « Créer un avenant » sur les contrats actifs
- Indicateur visuel "gelé" sur les entités en état terminal (Pattern A)
- Champs immuables rendus en lecture seule avec mention contextuelle
- Vue historique des avenants sur la fiche contrat (chaîne `predecessor_id`)

### Impact backend
- Nouveau module `domain/immutability.rs` portant les guards `is_mutable` / `is_deletable`
- Nouveau repository `db/audit_log.rs`
- Refactoring des handlers `PUT /contracts/{id}` et `PUT /budgets/{id}` en endpoints de transition explicites
