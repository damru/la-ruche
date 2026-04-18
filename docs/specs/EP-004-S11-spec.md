# Spécification technique — EP-004-S11

**Story :** Recouvrement des charges impayées : vue des lots débiteurs, mises en demeure formelles, déchéance du terme à J+30 (art. 19-2 loi 1965), lien vers un dossier contentieux.

**Épic :** EP-004 — Budget & comptabilité
**Réf. LEGAL :** §4.7 · **Réf. PRD :** US recouvrement

> Statut : NON IMPLÉMENTÉ. Périmètre **in-app uniquement** — l'envoi de courrier (recommandé/notification) est EP-021-S02, qui s'appuie sur le modèle `formal_notice` défini ici.

---

## 1. État actuel du code

| Élément | Emplacement |
|---|---|
| Statut `overdue` (lecture seule, dérivé) sur les appels | `backend/src/db/budget.rs:1088`, `backend/src/api/units.rs:398` |
| Affichage `overdue` | `frontend/src/features/budget/BudgetView.vue:199-200`, `frontend/src/features/dashboard/DashboardView.vue:106` |
| Registre contentieux avec valeur enum `debt_recovery` | migration `0034` (`judicial_proceedings`) |
| **Absent** | Toute notion de mise en demeure, de lot débiteur agrégé, de déchéance du terme |
| Audit infra | `backend/src/db/audit.rs`, migration `0029_audit_log.sql` |

---

## 2. Quoi (résumé fonctionnel)

1. **Lots débiteurs (dérivé, aucun stockage)** : par lot, somme des appels `issued` dont `due_date < today` et non soldés. Endpoint de liste pour le manager/board.
2. **Mise en demeure (`formal_notice`)** : acte formel enregistré par le syndic (envoyé hors app au MVP) — lot, date d'envoi, montant réclamé, mode d'envoi.
3. **Déchéance du terme J+30 (art. 19-2)** : **état dérivé, pas de mutation** — si une mise en demeure a plus de 30 jours et que la dette persiste, toutes les provisions non encore échues de l'exercice deviennent exigibles ; l'app le **signale** (flag + montant total exigible + alerte UI), elle ne modifie pas les `levy_call`.
4. **Lien contentieux** : depuis un lot débiteur en déchéance, créer/relier un dossier `judicial_proceedings` de type `debt_recovery` (registre existant, migration 0034).

---

## 3. Modèle de données

Mettre à jour `docs/ARCHITECTURE.md` (table `formal_notice`) avant la migration. Pas d'ADR : table de jonction standard, pas d'alternative structurelle réelle.

Migration `backend/migrations/NNNN_formal_notice.sql` (numéro à caler — renumérotation rectificatif en cours sur la branche) :

```sql
-- EP-004-S11: formal notices (mise en demeure) for arrears recovery (art. 19-2 loi 1965, LEGAL §4.7)
CREATE TYPE notice_method AS ENUM ('registered_mail', 'hand_delivery', 'bailiff', 'other');

CREATE TABLE formal_notice (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES building(id),
    unit_id UUID NOT NULL REFERENCES unit(id),
    sent_at DATE NOT NULL,
    amount NUMERIC(12,2) NOT NULL CHECK (amount > 0),
    method notice_method NOT NULL,
    note TEXT,
    proceeding_id UUID REFERENCES judicial_proceedings(id),
    created_by UUID NOT NULL REFERENCES app_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX formal_notice_unit_idx ON formal_notice (unit_id, sent_at DESC);
```

Une mise en demeure est un **acte** : pas d'UPDATE/DELETE exposés (append-only, cohérent ADR-009) ; ajouter la variante dans `domain/immutability.rs` (`Entity::FormalNotice`, jamais mutable). Chaque insert écrit une ligne `audit_log` dans la même transaction.

---

## 4. Backend

### 4.1 Domain — `backend/src/domain/arrears.rs` (nouveau)

Fonctions pures :

- `outstanding(levies: &[LevySummary], today: NaiveDate) -> Decimal` — total dû échu par lot.
- `acceleration_triggered(last_notice_sent_at: NaiveDate, today: NaiveDate, still_unpaid: bool) -> bool` — `sent_at + 30 jours ≤ today && still_unpaid` (déchéance du terme).
- Validation : refuser une mise en demeure si le lot n'a aucun montant échu (`errors.arrears.no_outstanding_amount`) ; montant réclamé ≤ dette échue + provisions exigibles (`errors.arrears.amount_exceeds_debt`).

### 4.2 DB

- `backend/src/db/arrears.rs` (nouveau) : `list_debtor_units(pool, building_id, today)` — agrégat SQL par lot (appels `issued`, `due_date < today`, non payés — réutiliser la logique `overdue` de `db/budget.rs:1088`) ; joint la dernière `formal_notice` du lot et le total des provisions non échues de l'exercice (pour le montant « exigible en déchéance »).
- `backend/src/db/formal_notice.rs` (nouveau) : `create` (tx + `db::audit::insert`), `list_by_unit`, `link_proceeding(id, proceeding_id)` (seule écriture postérieure autorisée : rattacher le dossier ; auditée).

### 4.3 API — `backend/src/api/arrears.rs` (nouveau) + routes `api/mod.rs`

| Route | Rôle | Effet |
|---|---|---|
| `GET /arrears` | manager, board | Lots débiteurs : dette échue, dernière mise en demeure, flag `acceleration_triggered`, montant exigible total. |
| `GET /units/{id}/formal-notices` | manager, board | Historique des mises en demeure du lot. |
| `POST /units/{id}/formal-notices` | manager | Crée l'acte (validations domain) → 201. |
| `POST /formal-notices/{id}/link-proceeding` | manager | Relie/crée le dossier `judicial_proceedings` (`debt_recovery`). |

Handlers minces ; erreurs via `AppError::validation_code`.

---

## 5. Frontend

- Types (`ArrearsUnit`, `FormalNotice`, `NoticeMethod`) + `arrearsApi` dans `frontend/src/api/index.ts`.
- Nouvelle vue `frontend/src/features/arrears/ArrearsView.vue` (route + entrée de nav manager/board) : tableau des lots débiteurs — dette échue, jours de retard, dernière mise en demeure, badge rouge « Déchéance du terme (J+30) » avec montant exigible.
- Modale `CreateFormalNoticeModal.vue` (date, montant pré-rempli avec la dette échue, mode, note).
- Section historique des mises en demeure sur la fiche lot + action « Ouvrir un dossier contentieux » quand la déchéance est déclenchée.
- Nouveaux composants → catalogue `docs/DESIGN.md §2` avant usage. Alerte de synthèse sur `DashboardView.vue` (nombre de lots en déchéance).

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Paiement partiel après mise en demeure | La déchéance s'évalue sur la dette **résiduelle** : soldée → flag retombe (état dérivé, rien à annuler). |
| Plusieurs mises en demeure | Le J+30 court depuis la **dernière** couvrant une dette encore ouverte. |
| Lot vendu / mutation | Les mises en demeure restent attachées au lot (historique) ; la dette suit les règles de mutation existantes. |
| Appel annulé par avoir | Sort de l'assiette de la dette échue. |
| Mise en demeure sans dette | Rejet `errors.arrears.no_outstanding_amount`. |
| Suppression / modification d'une mise en demeure | Interdites (acte, append-only). |

---

## 7. Critères d'acceptation

1. `GET /arrears` liste exactement les lots avec dette échue > 0, montants `Decimal` exacts.
2. Une mise en demeure créée est immuable, auditée, visible dans l'historique du lot.
3. Le flag déchéance apparaît à J+30 exactement (test avec dates contrôlées) et retombe si la dette est soldée.
4. Un dossier `debt_recovery` peut être relié depuis une mise en demeure.
5. Aucun envoi de courrier : l'app enregistre l'acte, elle ne l'expédie pas (EP-021-S02).
6. Clippy/tests/build/lint verts.

---

## 8. Plan de tests

- **Unitaires domaine** : `outstanding` (mix payé/impayé/avoir), `acceleration_triggered` (J+29 non, J+30 oui, dette soldée non), validations montant.
- **E2E** : scénario complet — génération d'appels, échéance dépassée → lot dans `/arrears` ; POST mise en demeure → 201 + audit ; 2ᵉ lecture à J+31 (dates injectées) → flag ; paiement → flag retombe ; POST sans dette → 400 ; link proceeding → 200.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `formal_notice`).
2. Migration `NNNN` → `sqlx migrate run` + `sqlx prepare`.
3. Domain `arrears.rs` + `Entity::FormalNotice` + tests unitaires.
4. DB (`arrears.rs`, `formal_notice.rs`, audit en tx).
5. API + routes + i18n.
6. Tests e2e.
7. Frontend (vue, modale, dashboard, catalogue DESIGN.md).
8. `docs/PROGRESS.md`.
