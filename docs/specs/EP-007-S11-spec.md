# Spécification technique — EP-007-S11

**Story :** Déclaration de fin d'intervention par le prestataire → la tâche passe en statut « rapport reçu » (`report_received` dans le PRD).

**Épic :** EP-007 — Travaux & prestataires
**Réf. PRD :** US-TP-010
**État :** PARTIEL — document de décision court : le comportement existe déjà sous un autre nom de statut.

---

## 1. État actuel du code

| Élément | Emplacement | Constat |
|---|---|---|
| `POST /vendor/{token}/submission` | `backend/src/api/contractor.rs:132-210` | La soumission prestataire pose `status = 'awaiting_validation'` (`:194`) et annule le token — **sémantiquement équivalent** à `report_received` |
| Enum `task_status` | `0001_init.sql:69`, `db/task.rs:107-114` | Pas de valeur `report_received` |

## 2. Décision — deux options

1. **Ajouter `report_received` à l'enum** et l'utiliser à la place d'`awaiting_validation` : conformité littérale au PRD, mais churn d'enum (migration, mapping Rust, types TS, badges UI, filtres) pour un renommage sans valeur fonctionnelle — et deux statuts synonymes à maintenir si l'existant n'est pas migré.
2. **Conserver `awaiting_validation` et amender le PRD** (US-TP-010) pour acter que « rapport reçu » = `awaiting_validation` : zéro migration, zéro churn ; le nom existant décrit même mieux l'étape suivante (le syndic doit valider).

**Recommandation retenue : option 2.** Le PRD est mis à jour, pas le code. Précédent : le PRD est la source de vérité *fonctionnelle*, mais un renommage pur de valeur d'enum ne change rien au comportement livré à l'utilisateur.

## 3. Écarts fonctionnels restants à vérifier (au-delà du nom)

La transition existe, mais US-TP-010 parle d'une **déclaration de fin d'intervention** ; le code déclenche la transition sur **toute soumission**. Points à trancher pendant l'implémentation (petits, listés ici pour ne pas les perdre) :

| Point | Constat | Action |
|---|---|---|
| Flag explicite « fin d'intervention » vs n'importe quelle soumission | `contractor.rs:132-210` traite toute soumission comme finale (token annulé, statut basculé) | Si le PRD exige des soumissions intermédiaires (devis, photos d'avancement) sans clôture, il faudrait un champ `is_final: bool` dans le payload et ne basculer le statut/annuler le token que si `true`. **Recommandé : ne pas l'ajouter maintenant** — le workflow actuel est mono-soumission ; noter la limite dans le PRD. |
| Horodatage de la déclaration | La ligne `vendor_submission` porte déjà son `created_at` | Suffisant — pas de colonne dédiée. |
| Notification au syndic | Hors périmètre de cette story (si un système de notifications existe, brancher là ; sinon rien). |

## 4. Modèle de données

Aucune migration (option 2 retenue).

## 5. Backend

Aucun changement de code si l'option 2 est confirmée. Ajouter uniquement un commentaire d'ancrage dans `api/contractor.rs` près de `:194` :

```rust
// PRD US-TP-010 "report_received" == 'awaiting_validation' (decision EP-007-S11):
// a vendor submission is the end-of-intervention report; the manager then
// validates and closes the task.
```

Si l'option 1 était finalement imposée : migration `NNNN` `ALTER TYPE task_status ADD VALUE 'report_received' …` + remplacement à `:194` + `UPDATE` de backfill des tâches `awaiting_validation` en migration séparée (contrainte PostgreSQL enum : valeur ajoutée non utilisable dans la même transaction — précédent 0030+0033) + mapping `db/task.rs:107-114` + types TS + badges. Rejetée par défaut.

## 6. Frontend

Aucun changement (option 2). Vérifier que le libellé FR du badge `awaiting_validation` dans `TasksView.vue` communique bien « Rapport reçu — à valider » ; ajuster le libellé i18n si nécessaire (changement de texte, pas de code).

## 7. Documentation à mettre à jour

- `docs/product/PRD` (EP-007 / US-TP-010) : remplacer la mention `report_received` par `awaiting_validation`, en explicitant l'équivalence et la limite « toute soumission est finale ».
- `docs/PROGRESS.md` : entrée datée « EP-007-S11 — décision : pas de nouveau statut, PRD amendé ».

## 8. Critères d'acceptation

1. Décision consignée : PRD amendé, aucun nouveau statut ajouté.
2. Le commentaire d'ancrage est présent dans `api/contractor.rs`.
3. Comportement existant confirmé par test : soumission prestataire → tâche `awaiting_validation`, token annulé.
4. Libellé UI du statut compréhensible comme « rapport reçu ».

## 9. Plan de tests

- **e2e (non-régression, probablement déjà couvert — vérifier)** : créer tâche, générer token, `POST /vendor/{token}/submission` → `GET /tasks/{id}` → `status == "awaiting_validation"` ; re-soumission avec le même token → rejet (token annulé).
- Aucun nouveau test exigé au-delà si la couverture existe déjà.

## 10. Ordre d'implémentation

1. Confirmer l'option 2 avec le product owner (ce document sert de support).
2. Amender le PRD.
3. Commentaire d'ancrage + éventuel libellé i18n.
4. Vérifier/ajouter le test e2e de non-régression.
5. `docs/PROGRESS.md`.
