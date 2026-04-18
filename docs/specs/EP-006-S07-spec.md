# Spécification technique — EP-006-S07

**Story :** Rattacher des documents à un contrat fournisseur, visibilité « Syndic uniquement » (PRD US-CF-007).

**Épic :** EP-006 — Contrats fournisseurs
**Réf. PRD :** US-CF-007
**Statut : PARTIEL — la fonctionnalité est implémentée ; il reste un écart de conformité PRD sur la visibilité, à trancher.**

---

## 1. Ce qui existe déjà (implémenté, ne pas retoucher)

| Élément | Emplacement |
|---|---|
| Table de lien `contract_document` | `backend/migrations/0001_init.sql` L495-499 |
| Endpoints attach/detach/list | `backend/src/api/contracts.rs` L168-213 |
| Repo (link/unlink/listing + compteurs) | `backend/src/db/contract.rs` L498, L572, L689-692, L741, L762 |
| UI de rattachement | `frontend/src/features/contracts/ContractsView.vue` L606-614 (`CLinkedDocumentsModal`) |

Le rattachement, le détachement, le listing et l'UI fonctionnent. **Aucun développement fonctionnel n'est requis.**

---

## 2. L'écart : visibilité manager+board_member vs « Syndic uniquement »

- Le PRD US-CF-007 demande une visibilité **Syndic uniquement** (rôle `manager`).
- Le code implémente **manager + board_member** :
  - upload pré-câblé avec `visible_roles: ['manager','board_member']` (`frontend/src/features/contracts/ContractFormModal.vue` L226)
  - endpoints de lecture ouverts aux deux rôles (`api/contracts.rs` L173).

## 3. Décision à prendre (objet réel de cette spec)

### Option A — recommandée : conserver la lecture conseil syndical, amender le PRD

Arguments :
- **Art. 21 de la loi du 10 juillet 1965** : le conseil syndical **contrôle la gestion du syndic** et peut prendre connaissance de « toutes pièces ou documents se rapportant à la gestion du syndic » — les contrats fournisseurs en sont l'exemple canonique. Restreindre au syndic seul irait contre l'esprit du contrôle de gestion, particulièrement en syndic bénévole où le conseil est l'organe de contre-pouvoir effectif.
- Le comportement actuel est en production dans le code depuis EP-006 : le restreindre serait une **régression fonctionnelle** pour les conseils syndicaux existants.
- L'écriture (rattacher/détacher) reste bien réservée au `manager` — seul le **read** est partagé.

Actions si retenue :
1. Amender le PRD US-CF-007 : « visibilité Syndic uniquement » → « visibilité Syndic + Conseil syndical (lecture) ; gestion des rattachements réservée au Syndic », avec renvoi à l'art. 21 loi 1965.
2. Vérifier que l'écriture est bien `manager`-only sur les endpoints attach/detach (`api/contracts.rs` L168-213) — si un `require_any_role` traînait sur l'écriture, le resserrer.
3. Aucun autre changement de code.

### Option B — appliquer le PRD à la lettre : restreindre à `manager`

Changements exhaustifs si cette option est retenue :

1. **Backend** — `api/contracts.rs` L173 (et tout autre endpoint de lecture des documents liés) : `require_any_role(&[Manager, BoardMember])` → `require_role(&Manager)`.
2. **Frontend** — `ContractFormModal.vue` L226 : `visible_roles: ['manager','board_member']` → `['manager']`.
3. **Données existantes** — migration de reprise : retirer `board_member` de `visible_to_roles` des documents liés à un contrat :
   ```sql
   UPDATE document d SET visible_to_roles = array_remove(visible_to_roles, 'board_member')
   WHERE EXISTS (SELECT 1 FROM contract_document cd WHERE cd.document_id = d.id)
   ```
   (avec les précautions : un document lié à la fois à un contrat et à un rapport visible du conseil ne doit pas perdre sa visibilité rapport — analyser les doubles liens avant d'écrire la migration ; c'est le principal coût caché de l'option B).
4. **UI conseil syndical** : masquer l'entrée contrats/documents pour `board_member` là où elle apparaît.
5. Tests e2e : lecture en `board_member` → `403` / listing vide.

### Ce que cette story N'EST PAS

Pas de nouveau modèle de visibilité par document, pas d'ACL fine — le mécanisme `visible_to_roles` existant suffit aux deux options.

---

## 4. Cas limites (surtout pour l'option B)

| Cas | Règle |
|---|---|
| Document doublement lié (contrat + sinistre/rapport visible du conseil) | Option B : NE PAS retirer `board_member` aveuglément — exclure ces documents de la migration de reprise. |
| Rattachement d'un document déjà existant à visibilité large | La visibilité du document ne change pas au rattachement (comportement actuel) — documenter dans le PRD quel que soit le choix. |
| Écriture (attach/detach) | `manager` uniquement dans les deux options. |

---

## 5. Critères d'acceptation

1. La décision A/B est actée par le product owner et tracée (PRD amendé, ou changements B livrés).
2. Option A : PRD mis à jour ; attach/detach vérifiés `manager`-only ; aucun autre diff.
3. Option B : lecture `board_member` → `403`, uploads futurs `['manager']`, reprise de données sans casser les doubles liens, tests e2e verts.
4.  : entrée datée EP-006-S07 (statut : terminée, avec la décision retenue).

---

## 6. Plan de tests

- Option A : test e2e de non-régression — lecture des documents liés en `board_member` → `200` ; attach en `board_member` → `403`.
- Option B : miroir — lecture `board_member` → `403` ; vérif SQL post-migration sur un jeu avec double lien.

---

## 7. Ordre d'implémentation

1. Présenter A/B au product owner avec l'argument art. 21 loi 1965 → décision.
2. Option A : amendement PRD + vérif garde écriture + test non-régression.
3. Option B : backend (rôles) → migration de reprise (après analyse des doubles liens) → frontend → tests.
4.  si du code a changé.
