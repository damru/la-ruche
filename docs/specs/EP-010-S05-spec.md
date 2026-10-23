# Spécification technique — EP-010-S05

**Story :** Relancer les non-répondants d'une consultation ouverte : voir combien de participants attendus n'ont pas encore voté et enregistrer une relance.

**Épic :** EP-010 — Consultations informelles
**Réf. PRD :** US-CS-004
**État :** IMPLÉMENTÉ (backend + frontend) — compteur de non-répondants, enregistrement des relances (`consultation_reminder`), affichage de la dernière relance et bouton de relance. La livraison effective (email/push) reste différée : la relance est un enregistrement en base + retour applicatif.

> **Périmètre notification :** aucune infrastructure de livraison (email/push) n'existe dans le repo. La « relance » est donc un **enregistrement en base + retour applicatif** (compteur, horodatage de dernière relance). La livraison réelle est différée et se branchera sur `consultation_reminder` le jour venu.

---

## 1. Quoi (résumé fonctionnel)

- Le gestionnaire/conseil voit, sur une consultation `open`, le **nombre de non-répondants** (participants attendus = utilisateurs actifs de l'immeuble dont un rôle intersecte `visible_to_roles`, moins ceux ayant une `consultation_response`).
- Un bouton **« Relancer les non-répondants (N) »** enregistre une relance (`consultation_reminder`) et affiche une confirmation. La date de dernière relance est visible.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement | Constat |
|---|---|---|
| Routes consultations (list/create/get/update/delete/respond/results/close) | `backend/src/api/mod.rs` L327-340 | Aucune route non-répondants / relance |
| Réponses : `consultation_response` (`UNIQUE (consultation_id, user_id)`) | `backend/migrations/0001_init.sql` L700-707 | Base du calcul « a répondu » |
| Rôles par immeuble : `user_building.roles user_role[]` | `0001_init.sql` L255-265 | Base du calcul « attendu » |
| `visible_to_roles user_role[]` sur `consultation` | `0001_init.sql` L685 | Cible de l'intersection |
| Repo DB consultations | `backend/src/db/consultation.rs` | Aucune requête non-répondants |
| Vue liste | `frontend/src/features/consultations/ConsultationsView.vue` | Cartes open avec actions manage (L34 `canManage`) |
| Client API | `frontend/src/api/index.ts` L450-468 | À étendre |

## 3. Modèle de données

Nouvelle table minimale (journal de relances). **Mettre à jour `docs/ARCHITECTURE.md` avant la migration** (table simple, pas d'ADR).

### Migration `backend/migrations/NNNN_consultation_reminder.sql`

```sql
-- EP-010-S05: reminder log for consultation non-respondents (US-CS-004).
-- Delivery infrastructure does not exist yet; this table records the intent
-- (who triggered a reminder, when, how many recipients) and will feed the
-- future notification dispatcher.
CREATE TABLE consultation_reminder (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    consultation_id UUID NOT NULL REFERENCES consultation(id) ON DELETE CASCADE,
    sent_by         UUID NOT NULL REFERENCES user_account(id),
    recipient_count INTEGER NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX consultation_reminder_consultation_idx
    ON consultation_reminder (consultation_id);
```

Puis `cargo sqlx migrate run` + `cargo sqlx prepare` (commit `backend/.sqlx/`).

## 4. Backend

### 4.1 `db/consultation.rs` — calcul des non-répondants

```rust
/// Users expected to respond (active building members whose roles intersect
/// visible_to_roles) who have no response yet. Returns (user_id, first_name,
/// last_name) rows. 404 upstream if the consultation doesn't exist.
pub async fn non_respondents(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>>
```

Requête :

```sql
SELECT u.id, u.first_name, u.last_name
FROM consultation c
JOIN user_building ub ON ub.building_id = c.building_id
JOIN user_account u ON u.id = ub.user_id AND u.active
WHERE c.id = $1 AND c.building_id = $2
  AND ub.roles && c.visible_to_roles
  AND (ub.end_date IS NULL OR ub.end_date >= CURRENT_DATE)
  AND NOT EXISTS (
      SELECT 1 FROM consultation_response r
      WHERE r.consultation_id = c.id AND r.user_id = u.id
  )
ORDER BY u.last_name, u.first_name
```

### 4.2 `db/consultation.rs` — enregistrement de relance

```rust
/// Records a reminder for an OPEN consultation. Guards status='open'
/// (errors.consultations.not_open otherwise). Returns recipient_count.
pub async fn remind_non_respondents(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    sent_by: Uuid,
) -> AppResult<i64>
```

Transaction : `SELECT status FROM consultation WHERE id=$1 AND building_id=$2 FOR UPDATE` (404 si absent, 400 `errors.consultations.not_open` si `status != 'open'`) → calcul §4.1 → `INSERT INTO consultation_reminder (consultation_id, sent_by, recipient_count) VALUES (...)` → retourne le compte. Zéro non-répondant → 400 `errors.consultations.no_non_respondents` (pas de relance vide).

Exposer aussi `last_reminder_at` : sous-requête `MAX(created_at)` dans le `SELECT` de `list`/`get_with_choices` (champ JSON `last_reminder_at`, nullable).

### 4.3 `api/consultations.rs` — handlers

```rust
/// GET /consultations/{id}/non-respondents — manager | board_member.
pub async fn non_respondents(...) -> AppResult<Json<serde_json::Value>>
// → { "count": n, "users": [{id, first_name, last_name}] }

/// POST /consultations/{id}/remind — manager | board_member.
pub async fn remind(...) -> AppResult<Json<serde_json::Value>>
// → { "recipient_count": n }
```

Les deux : `auth.require_any_role(&[Role::Manager, Role::BoardMember])?`, zéro logique métier (délégation à `db::consultation`).

### 4.4 `api/mod.rs` — routes

```rust
.route("/consultations/{id}/non-respondents", get(consultations::non_respondents))
.route("/consultations/{id}/remind", post(consultations::remind))
```

## 5. Frontend

### 5.1 `frontend/src/api/index.ts` — `consultationsApi`

```ts
nonRespondents: (id: string) =>
  api.get<{ count: number; users: { id: string; first_name: string; last_name: string }[] }>(`/consultations/${id}/non-respondents`),
remind: (id: string) =>
  api.post<{ recipient_count: number }>(`/consultations/${id}/remind`),
```

### 5.2 `ConsultationsView.vue`

Sur les cartes `open`, visible si `canManage` :

- Ligne d'information « N participants n'ont pas encore répondu » (fetch lazy `nonRespondents` au dépliage de la carte ou au chargement de l'onglet open — retenu : au chargement, en parallèle des consultations ouvertes, comme `consultationTotalVotes`). Afficher `last_reminder_at` (« Dernière relance : {date relative} ») s'il existe.
- Bouton **« Relancer (N) »** désactivé si `N === 0` ; confirmation `useConfirm` (« Une relance sera enregistrée pour N participants ») → `consultationsApi.remind(id)` → toast succès + reload. Le libellé i18n précise que l'envoi effectif (email) n'est pas encore actif.

Pas de nouveau composant → pas d'entrée `docs/DESIGN.md §2`.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Relance sur consultation `draft`/`closed` | 400 `errors.consultations.not_open` |
| Zéro non-répondant | Bouton désactivé côté UI ; 400 `errors.consultations.no_non_respondents` côté API |
| Utilisateur inactif (`user_account.active = false`) ou adhésion expirée (`end_date` passée) | Exclu du calcul des attendus |
| Multi-rôles (`{co_owner, board_member}`) | Intersection `&&` avec `visible_to_roles` — compté une seule fois |
| Relances répétées | Autorisées (journal, pas de throttling) ; `last_reminder_at` informe le gestionnaire — un throttling éventuel viendra avec la vraie livraison |
| Rôle résident simple | 403 sur les deux endpoints |

## 7. Critères d'acceptation

1. Le gestionnaire voit le nombre exact de non-répondants d'une consultation ouverte.
2. « Relancer » enregistre une ligne `consultation_reminder` avec le bon `recipient_count` et affiche la date de dernière relance.
3. Impossible de relancer une consultation non ouverte ou sans non-répondant.
4. Un résident simple n'a accès ni au compteur ni à la relance.
5. Aucun email/push envoyé (différé) — le libellé UI le dit explicitement.

## 8. Plan de tests

E2e (`backend/tests/api_e2e_test.rs`) :

1. Consultation ouverte, 2 utilisateurs éligibles, 0 réponse → `GET /non-respondents` → `count == 2`.
2. Un utilisateur répond → `count == 1` ; l'utilisateur hors `visible_to_roles` n'est jamais compté.
3. `POST /remind` → 200, `recipient_count == 1` ; `GET /consultations` expose `last_reminder_at` non nul.
4. `POST /remind` sur consultation fermée → 400 `errors.consultations.not_open`.
5. Tous ont répondu → `POST /remind` → 400 `errors.consultations.no_non_respondents`.
6. Rôle `co_owner` simple → 403 sur `GET /non-respondents` et `POST /remind`.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `consultation_reminder`) → migration `NNNN` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. `db/consultation.rs` : `non_respondents`, `remind_non_respondents`, `last_reminder_at` dans `list`/`get_with_choices` → `cargo build`.
3. `api/consultations.rs` + routes `api/mod.rs` → `cargo clippy -- -D warnings`.
4. Tests e2e §8, boucler jusqu'au vert.
5. Frontend : `api/index.ts`, `ConsultationsView.vue` (compteur + bouton + confirmation + i18n).
6. `npm --prefix frontend run build && npm --prefix frontend run lint` ; `docs/EPICS.md` / `docs/PROGRESS.md`.
