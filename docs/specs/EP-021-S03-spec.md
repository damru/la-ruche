# Spécification technique — EP-021-S03

**Story :** Câblage complet de l'`audit_log` sur toutes les mutations comptables + endpoint de lecture + UI historique.

**Épic :** EP-021 — Conformité & communications
**Réf. ADR :** ADR-009 (piste d'audit probante) · **Réf. LEGAL :** §11

> Statut : PARTIEL. L'**infrastructure est correcte et terminée** ; il manque le câblage de la majorité des sites d'écriture, tout endpoint de lecture, et toute UI.

---

## 1. État actuel du code

### Fait (ne pas retoucher)

| Élément | Emplacement |
|---|---|
| Table `audit_log` (append-only, pas d'UPDATE/DELETE) | migration `0029_audit_log.sql` |
| `db::audit::insert` (générique sur `Executor` — utilisable dans une tx) + `list_by_entity` | `backend/src/db/audit.rs` |

### Sites déjà câblés (5)

| Site | Emplacement |
|---|---|
| Budget : création rectificatif | `backend/src/db/budget.rs:642` |
| Budget : approbation | `backend/src/db/budget.rs:727` |
| Contrat : création | `backend/src/db/contract.rs:297` |
| Contrat : mise à jour | `backend/src/db/contract.rs:366` |
| Contrat : résiliation | `backend/src/db/contract.rs:389` |

### Manquant (périmètre de cette spec)

1. **Sites d'écriture non audités** (mutations comptables, `backend/src/db/budget.rs` et voisins — repérer chaque fonction et ajouter l'insert dans sa transaction) :
   - émission d'appels de fonds (`generate_levies` / issue) ;
   - avoir / annulation d'appel (credit note) ;
   - marquage payé (`mark_paid`) ;
   - ligne budgétaire : create / update / delete ;
   - mise à jour du taux fonds travaux ;
   - mutation de lot (transfert de propriété, `db/unit.rs` ou équivalent).
2. **Endpoint de lecture** : aucune route `/audit` n'existe.
3. **UI** : aucune section historique.

---

## 2. Quoi (résumé fonctionnel)

1. Chaque mutation comptable listée écrit une ligne `audit_log` **dans la même transaction** que la mutation (atomicité : pas de mutation sans trace, pas de trace sans mutation).
2. `GET /audit?entity_type=&entity_id=` — lecture **scopée par entité** (manager + board, lecture seule).
3. UI minimale : section « Historique » repliable sur les fiches concernées (budget, appel de fonds, contrat, lot), listant date, auteur, opération.

---

## 3. Modèle de données

Aucun changement de schéma. `0029_audit_log.sql` suffit. Pas de mise à jour `docs/ARCHITECTURE.md` nécessaire (table déjà documentée — vérifier).

---

## 4. Backend

### 4.1 Convention de câblage (suivre les 5 sites existants)

Pour chaque site, à l'intérieur de la transaction de la mutation :

```rust
db::audit::insert(&mut *tx, AuditEntry {
    building_id,
    user_id,                       // acteur — propager depuis le handler si absent de la signature
    entity_type: "levy_call",      // snake_case, stable
    entity_id,
    operation: "CREATE",           // CREATE | UPDATE | DELETE | ISSUE | CANCEL | MARK_PAID | TRANSFER
    payload,                       // JSON before/after ou champs saillants — même style que budget.rs:642
}).await?;
```

Adapter aux noms réels de `db/audit.rs`. Certaines fonctions db ne reçoivent pas `user_id` aujourd'hui : **étendre leur signature** (le handler a `auth.user_id`) — c'est l'essentiel du travail mécanique. Fonctions sans transaction → en introduire une (mutation + audit).

### 4.2 Inventaire des sites à câbler (à confirmer ligne à ligne à l'implémentation)

| Mutation | Fichier | `entity_type` / `operation` |
|---|---|---|
| Émission des appels | `db/budget.rs` (generate/issue) | `levy_call` / `ISSUE` (une ligne par lot ou une ligne agrégée par génération — décision : **une par appel émis**, granularité probante) |
| Avoir / annulation | `db/budget.rs` (credit note) | `levy_call` / `CANCEL` |
| Marquage payé | `db/budget.rs` (`mark_paid`) | `levy_call` / `MARK_PAID` |
| Ligne budgétaire create/update/delete | `db/budget.rs` | `budget_line` / `CREATE`·`UPDATE`·`DELETE` (payload before/after sur update) |
| Taux fonds travaux | `db/budget.rs` (works-fund rate update) | `budget` / `UPDATE` (payload `{works_fund_rate: {from, to}}`) |
| Transfert de lot | repo unités | `unit` / `TRANSFER` (payload anciens/nouveaux propriétaires) |

### 4.3 Endpoint de lecture — `backend/src/api/audit.rs` (nouveau) + route `api/mod.rs`

```
GET /audit?entity_type={t}&entity_id={id}   → manager | board_member
```

- Paramètres **obligatoires** (pas de listing global au MVP — la lecture est entité-scopée) ; `400 errors.audit.entity_scope_required` si absents.
- S'appuie sur `db::audit::list_by_entity` (existant) ; filtrer par `building_id` de l'utilisateur (vérifier que `list_by_entity` le fait, sinon l'ajouter — un board member ne doit jamais lire l'audit d'un autre immeuble).
- Tri décroissant par date ; limite raisonnable (ex. 200) — pas de pagination au MVP.

### 4.4 Domain

Rien : l'audit est un effet transactionnel du repo, pas une règle métier. Aucune logique en handler.

---

## 5. Frontend

- Type `AuditEntry` + `auditApi.listByEntity(entityType, entityId)` dans `frontend/src/api/index.ts`.
- Composant réutilisable `AuditHistorySection.vue` (props `entityType`, `entityId`) : liste repliable « Historique » — date, auteur, opération traduite (clés i18n `audit.operation.*`), détail payload en accordéon. → catalogue `docs/DESIGN.md §2` avant usage.
- Intégration : fiche budget (`BudgetView.vue`), détail contrat (`ContractsView.vue`), fiche lot. Visible manager + board.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Rollback de la mutation | La ligne d'audit disparaît avec (même tx) — comportement voulu. |
| Génération de N appels | N lignes d'audit (une par appel) — la requête entité-scopée par appel reste cohérente. |
| Entité sans historique | `GET /audit` → tableau vide (200). |
| `entity_type` inconnu | Tableau vide (pas d'erreur — pas de liste blanche à maintenir). |
| Acteur système (job) | `user_id` nullable ou utilisateur système selon le schéma 0029 — suivre l'existant ; les jobs actuels ne mutent pas de données comptables, point théorique. |
| Anonymisation RGPD d'un utilisateur | Les lignes d'audit conservent le `user_id` (obligation probante > effacement ; l'identité affichée suit l'anonymisation du profil). |

---

## 7. Critères d'acceptation

1. Chaque mutation listée en §4.2 produit sa ligne `audit_log` atomiquement (vérifiable en e2e : mutation → lecture audit → présence ; échec de mutation → absence).
2. `GET /audit` renvoie l'historique entité-scopé au manager et au board, `403` pour un owner, jamais de fuite inter-immeubles.
3. La section Historique s'affiche sur budget, contrat et lot.
4. Aucune régression sur les 5 sites déjà câblés (mêmes `entity_type`/format de payload).
5. Clippy/tests/build/lint verts.

---

## 8. Plan de tests

- **E2E (l'essentiel)** : pour chaque site câblé — exécuter la mutation puis `GET /audit` sur l'entité et vérifier `operation`, auteur, payload saillant ; update de ligne budgétaire → payload before/after ; mutation refusée (guard immutabilité) → aucune ligne ; owner sur `/audit` → 403 ; paramètres manquants → 400.
- **Unitaires** : si un helper de construction de payload est extrait, le tester (diff before/after).

---

## 9. Ordre d'implémentation

1. Inventaire ligne à ligne des fonctions db de §4.2 (confirmer signatures et présence de tx).
2. Câbler site par site (étendre les signatures avec `user_id`, introduire les tx manquantes) — un commit par groupe cohérent, e2e au fil de l'eau.
3. Endpoint `GET /audit` (+ scoping immeuble dans `list_by_entity` si absent) + i18n.
4. Frontend : `AuditHistorySection.vue` (catalogue DESIGN.md) + intégrations.
5. Vérif finale → `docs/PROGRESS.md`.
