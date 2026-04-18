# ADR-012 — Verrouillage optimiste des lignes budgétaires

**Date :** 2026-08-26
**Statut :** Accepté
**Références :** ADR-009 §Pattern B · ADR-009 §B1 · ADR-006 (versioning temporel) · LEGAL.md §11 · EP-018-S06

---

## Contexte

`budget_line` porte désormais deux mécanismes qui ressemblent tous deux à du
« versioning », et cette proximité a déjà provoqué une confusion en revue :

1. **Versioning temporel** (`valid_from` / `valid_to`) — mécanisme *juridique*
   introduit par ADR-009 §Pattern B. Il répond à « quel était le budget voté à la
   date T ». Une correction ne modifie pas une ligne : elle clôt la version en
   vigueur et en crée une nouvelle dans le budget rectificatif.
2. **Verrouillage optimiste** (`version`) — mécanisme *technique*. Il répond à
   « ce que je m'apprête à écraser est-il bien ce que j'ai lu ».

ADR-009 §B1 rejette « le versioning en place avec colonne `version` », mais ce
rejet est explicitement motivé *pour les contrats* et porte sur l'usage de
`version` comme substitut juridique à `predecessor_id`. Il ne se prononce pas sur
le contrôle de concurrence. Sans décision écrite, la colonne `version` de
`budget_line` se lit à tort comme une contradiction avec ADR-009.

Le besoin de concurrence est réel : les lignes d'un budget au statut `draft` sont
éditées par plusieurs membres du conseil syndical pendant la préparation de l'AG,
sur des montants financiers. Une écriture perdue silencieuse est inacceptable.

## Décision

**Conserver la colonne `version` sur `budget_line`, strictement cantonnée au
contrôle de concurrence des lignes de brouillon.**

- `PUT /budgets/{id}/lines/{lid}` exige un champ `version` dans son payload et le
  place dans la clause `WHERE`. Un jeton périmé retourne `409 Conflict`
  (`errors.budget_line.version_mismatch`), jamais une écriture silencieuse.
- L'incrément est porté par le trigger `budget_line_version_trigger`, pas par la
  clause `SET` : un `SET version = version + 1` oublié dans une requête rendrait
  le verrou décoratif, et le trigger rend l'invariant indépendant du site d'appel.
- Le trigger porte `WHEN (OLD.valid_to IS NOT DISTINCT FROM NEW.valid_to)`. La
  clôture d'une ligne par le workflow rectificatif est une opération système : si
  elle incrémentait `version`, tout client détenant un jeton verrait son édition
  suivante refusée par un `409` sans rapport avec son action.
- `version` n'a **aucune** valeur probante. La piste juridique est portée par le
  versioning temporel et par `audit_log`. `version` n'est jamais exposé dans un
  document réglementaire ni utilisé pour reconstituer un état passé.

## Alternatives considérées

### 1 — Aucun verrouillage (dernier écrivain gagne)

**Rejeté :** perte d'écriture silencieuse sur des montants financiers pendant la
phase où plusieurs personnes préparent le budget. C'est précisément le scénario
que LEGAL.md §11 cherche à rendre traçable.

### 2 — Comparaison de `updated_at` au lieu d'un compteur

**Rejeté :** `budget_line` n'a pas de colonne `updated_at`, et la résolution des
timestamps PostgreSQL ne garantit pas deux valeurs distinctes pour deux écritures
successives très rapprochées. Un compteur entier est exact par construction.

### 3 — Verrouillage pessimiste (`SELECT … FOR UPDATE` sur la durée de l'édition)

**Rejeté :** l'édition se fait dans un formulaire, sur un temps humain. Tenir une
transaction ouverte pendant ce temps immobiliserait une connexion et exposerait à
des verrous orphelins.

### 4 — Réutiliser `valid_from` / `valid_to` pour la concurrence

**Rejeté :** confond une garantie juridique avec un détail d'implémentation. Créer
une nouvelle version temporelle à chaque frappe polluerait la piste probante avec
des versions qui n'ont jamais été votées.

## Conséquences

- Le frontend doit renvoyer le `version` lu sur la ligne. `BudgetLine.version` est
  donc un champ obligatoire côté client, pas optionnel.
- Le trigger s'applique à tout `UPDATE` sur `budget_line` : toute nouvelle
  écriture doit soit fournir un jeton, soit ne modifier que `valid_to`.
- Aucune migration supplémentaire : la colonne et le trigger sont créés par
  `0038_budget_rectificatif_schema`.
