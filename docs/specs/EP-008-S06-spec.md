# Spécification technique — EP-008-S06

**Story :** Sinistre — cycle `draft → declared` : un sinistre est créé en brouillon, visible du syndic seul ; une action explicite de déclaration le rend visible et démarre les délais légaux.

**Épic :** EP-008 — Sinistres
**Réf. PRD :** US-SIN-001b · **Réf. LEGAL :** délais de déclaration assurance (art. L113-2 C. assurances — 5 jours ouvrés) · **Réf. ADR :** ADR-009 (précédent split-migration enum)
**État :** IMPLÉMENTÉ (backend + frontend)

---

## 1. État actuel du code

| Élément | Emplacement | Constat |
|---|---|---|
| Enum `claim_status` | `0001_init.sql:76-78`, défaut `'open'` `:593` | `'open','in_progress','closed'` — pas de `draft`/`declared` |
| `declared_at` | colonne `DATE` sur `claim` | Une **date**, pas un état |
| Gate lifecycle | `backend/src/api/claims.rs:55-63`, `backend/src/domain/immutability.rs:45` | Le gel porte sur `transmitted_at` |
| `list` claims | `backend/src/api/claims.rs:16-22`, route `api/mod.rs:282` | **Aucune** branche par rôle — tout résident voit tout |

## 2. Décision de conception

- **`draft` devient une valeur d'enum**, pas un booléen : cohérent avec le cycle `claim_status` existant et le précédent `levy_call` (`0030` + `0033`).
- **`declared` n'est PAS ajouté** : « déclaré » = a quitté `draft` (statut `open`), matérialisé par `declared_at` non nul posé par l'action de déclaration. Ajouter `declared` en plus d'`open` créerait deux états synonymes. Le PRD parle de transition `draft → declared` ; on la réalise comme `draft → open` + `declared_at = date de déclaration`.
- **Délais légaux** : tout calcul d'échéance (5 jours ouvrés assurance) démarre à `declared_at`, jamais à `created_at`.

## 3. Modèle de données

Contrainte PostgreSQL : une valeur d'enum fraîchement ajoutée est inutilisable dans la même transaction → **deux migrations** (précédent `0030_levy_call_draft.sql` + `0033_levy_call_draft_status.sql`).

**Migration `NNNN_claim_draft.sql`** (valeur seule) :

```sql
-- EP-008-S06: claim draft state (PRD US-SIN-001b). Value only — consumers in next migration.
ALTER TYPE claim_status ADD VALUE IF NOT EXISTS 'draft' BEFORE 'open';
```

**Migration `NNNN+1_claim_draft_default.sql`** :

```sql
-- EP-008-S06: new claims start as draft, visible to the manager only.
-- Existing rows keep their status: they were all created pre-draft-concept.
ALTER TABLE claim ALTER COLUMN status SET DEFAULT 'draft';
```

`declared_at` existe déjà — pas de nouvelle colonne. Pas de nouvelle table → pas d'ajout `docs/ARCHITECTURE.md` (mettre à jour la description de l'enum si documentée). Puis `cargo sqlx migrate run` + `cargo sqlx prepare`.

## 4. Backend

### 4.1 Domain

- `immutability.rs` : le gate `transmitted_at` (`:45`) est conservé ; ajouter si besoin la règle « un claim `draft` est librement modifiable/supprimable ».
- Règle pure : `can_declare(status) == (status == "draft")`.

### 4.2 DB — `backend/src/db/claim.rs`

- `create` : statut par défaut `draft` (via défaut DB) ; **ne plus poser `declared_at` à la création** si c'est le cas aujourd'hui — il est posé à la déclaration.
- Nouvelle fonction `declare(pool, id, building_id, declared_at: Option<NaiveDate>)` : `FOR UPDATE`, check `can_declare` sinon `AppError::validation_code("errors.claims.not_declarable")`, puis `UPDATE claim SET status = 'open', declared_at = COALESCE($3, CURRENT_DATE)`.
- `list` : accepter un flag `include_drafts: bool` — `AND ($n OR c.status <> 'draft')`.

### 4.3 API — `backend/src/api/claims.rs` + routes

- Handler `declare` (rôle `Manager`), body optionnel `{ declared_at?: date }`. Route : `.route("/claims/{id}/declare", post(claims::declare))` dans le bloc claims de `api/mod.rs` (près de `:282`).
- `list` (`:16-22`) : `include_drafts = auth.has_any_role(Manager)` — les brouillons sont invisibles pour tout autre rôle (y compris `BoardMember`, sauf décision PRD contraire — **US-SIN-001b dit « syndic seul »**).
- `get /claims/{id}` : un non-manager demandant un claim `draft` reçoit `404` (pas `403` — ne pas révéler l'existence).

Réponses `declare` : `200` (status `open`, `declared_at` posé) · `400 errors.claims.not_declarable` · `403` · `404`.

## 5. Frontend

- `frontend/src/types/domain.ts` : `ClaimStatus` += `'draft'`.
- `frontend/src/api/index.ts` (`claimsApi`) : `declare: (id: string, payload?: { declared_at?: string }) => api.post<Claim>(`/claims/${id}/declare`, payload)`.
- `ClaimsView.vue` : badge `draft` (« Brouillon ») ; action « Déclarer » (Manager seul) avec confirmation + champ date optionnel (défaut aujourd'hui) ; mention du délai légal courant à partir de la déclaration.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Declare sur claim `open`/`in_progress`/`closed` | `400 errors.claims.not_declarable` |
| Claim `draft` listé par co_owner/tenant/board_member | Absent de la liste ; `GET` direct → `404` |
| `declared_at` fourni dans le futur | Refuser (`errors.claims.declared_at_in_future`) — une déclaration est un fait passé ou présent |
| Claims existants (pré-migration) | Gardent leur statut ; aucun backfill (aucun n'était un brouillon) |
| Gate `transmitted_at` | Inchangé — orthogonal au cycle draft |
| Délai légal | Calculé depuis `declared_at` ; un claim `draft` n'affiche aucune échéance |

## 7. Critères d'acceptation

1. `POST /claims` crée en `draft`, `declared_at` null.
2. `POST /claims/{id}/declare` → `open` + `declared_at` posé ; idempotence refusée (`400` au 2ᵉ appel).
3. Un `draft` n'apparaît que pour le Manager (liste et détail).
4. Échéances légales affichées uniquement après déclaration, calculées depuis `declared_at`.

## 8. Plan de tests (e2e — `backend/tests/api_e2e_test.rs`)

1. Create → `status == "draft"`, `declared_at == null`.
2. `GET /claims` en co_owner → le draft n'apparaît pas ; en manager → il apparaît.
3. `GET /claims/{draft_id}` en co_owner → `404`.
4. Declare → `200`, `open`, `declared_at == today` ; visible du co_owner ensuite.
5. Re-declare → `400 errors.claims.not_declarable`.
6. Declare avec `declared_at` futur → `400`.

## 9. Ordre d'implémentation

1. Migrations `NNNN` (valeur enum) + `NNNN+1` (défaut) → `sqlx migrate run` + `sqlx prepare`.
2. Domain `can_declare` + test.
3. DB `declare`, `create` (pas de declared_at), `list` filtré.
4. API handler + filtrage rôle + route + i18n.
5. Tests e2e.
6. Frontend types/api/vue.
7. Docs (`ARCHITECTURE.md` enum).
