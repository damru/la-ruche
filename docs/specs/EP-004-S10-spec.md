# Spécification technique — EP-004-S10

**Story :** Seuil légal du fonds travaux (art. 14-2-1 loi 1965) : `max(5 % du budget prévisionnel, 2,5 % du montant du PPT adopté)`.

**Épic :** EP-004 — Budget & comptabilité
**Réf. LEGAL :** §4.4 · **Réf. ADR :** —

> Statut : PARTIEL, **majoritairement différé**. La branche 5 % est livrée ; la branche 2,5 % PPT est explicitement différée (absence de modèle PPT — EP-009-S04 hors périmètre actuel). Ce document acte le report et fige la cible pour lever le différé sans redécouverte.

---

## 1. État actuel du code (branche 5 % — FAIT)

| Élément | Emplacement |
|---|---|
| Constante `legal_minimum_works_fund_rate_pct` (5 %) | `backend/src/api/budget.rs:31-33` |
| Doc-comment actant le report de la branche 2,5 % PPT | `backend/src/api/budget.rs:25-30` |
| Contrôle à la clôture d'exercice | `backend/src/api/budget.rs:140-152` |
| Contrôle à la génération d'appels | `backend/src/api/budget.rs:313-318` |
| Affichage du seuil | `frontend/src/features/budget/BudgetView.vue:795`, `frontend/src/features/budget/GenerateLeviesModal.vue:108` |
| Tests | `backend/src/api/budget.rs:721-738` |

Rien à modifier tant que le modèle PPT n'existe pas.

---

## 2. Déclencheur de levée du différé

La branche 2,5 % devient implémentable **dès que EP-009-S04 (modèle PPT) livre** :

- une table PPT portant le **montant total adopté** (`Decimal`) et le **statut d'adoption** (le PPT voté en AG) ;
- un accès repo type `db::ppt::get_adopted(pool, building_id) -> Option<PptRow>`.

Tant que ce prérequis n'existe pas, **aucun travail sur cette story** — le seuil effectif reste 5 % du budget, ce qui est légalement correct pour une copropriété sans PPT adopté.

---

## 3. Formule cible (art. 14-2-1)

```
seuil_minimal = max(0.05 × budget_previsionnel_total, 0.025 × montant_ppt_adopte)
```

- Si aucun PPT adopté : `seuil_minimal = 0.05 × budget` (comportement actuel, inchangé).
- Tout en `Decimal` — jamais `f64`.

### Esquisse d'implémentation (contre les ancrages actuels)

1. **Domain** : extraire une fonction pure (nouveau `domain/works_fund.rs` ou module budget) :
   ```rust
   pub(crate) fn legal_minimum_works_fund(
       budget_total: Decimal,
       adopted_ppt_amount: Option<Decimal>,
   ) -> Decimal {
       let five_pct = budget_total * dec!(0.05);
       match adopted_ppt_amount {
           Some(ppt) => five_pct.max(ppt * dec!(0.025)),
           None => five_pct,
       }
   }
   ```
2. **API** : aux deux points de contrôle (`api/budget.rs:140-152` clôture, `:313-318` génération), remplacer le calcul inline 5 % par un appel à cette fonction, alimentée par `db::ppt::get_adopted` (fetch dans le handler, calcul en domain — pas de SQL en domain).
3. Supprimer/mettre à jour le doc-comment de report (`api/budget.rs:25-30`) et la constante `:31-33` (absorbée par la fonction domain).
4. **Frontend** : `BudgetView.vue:795` et `GenerateLeviesModal.vue:108` affichent le seuil renvoyé par l'API (exposer le seuil calculé dans la réponse budget plutôt que de recalculer 5 % côté client) ; mentionner la base retenue (« 5 % budget » vs « 2,5 % PPT ») dans le libellé.

---

## 4. Cas limites (à couvrir à la levée du différé)

| Cas | Règle |
|---|---|
| PPT non adopté (brouillon/rejeté) | Ignoré — branche 5 % seule. |
| PPT adopté à montant faible (2,5 % < 5 % budget) | Le `max` retient la branche budget. |
| Plusieurs PPT historiques | Seul le PPT adopté **en vigueur** compte (définition à fixer dans EP-009-S04). |
| Budget rectificatif | Base = budget actif de l'exercice (lignes `valid_to IS NULL`). |

---

## 5. Critères d'acceptation (différés)

1. `legal_minimum_works_fund` : fonction pure, testée sur les 3 cas (sans PPT, PPT dominant, budget dominant).
2. Les deux points de contrôle existants utilisent la fonction sans changement de comportement quand aucun PPT n'est adopté (tests `api/budget.rs:721-738` restent verts).
3. UI : le seuil affiché indique sa base de calcul.

## 6. Plan de tests (différé)

- Unitaires domaine : les 3 cas du `max` + montants `Decimal` exacts.
- E2E : clôture refusée sous le seuil PPT ; génération sous seuil → même erreur i18n qu'aujourd'hui.

## 7. Ordre d'implémentation (à la levée)

1. Attendre EP-009-S04 (modèle PPT + repo).
2. Domain `legal_minimum_works_fund` + tests.
3. Brancher les 2 points de contrôle + exposer le seuil dans la réponse API.
4. Frontend (libellé avec base de calcul).
5. `docs/PROGRESS.md`.
