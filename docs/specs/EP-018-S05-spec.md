# Spécification technique — EP-018-S05

**Story :** Appliquer le Pattern B sur `vendor_contract` : ajouter `predecessor_id`, remplacer « Modifier » par « Créer un avenant », clôturer l'ancien contrat à la date d'effet et exposer la chaîne d'avenants.

**Épic :** EP-018 — Immutabilité comptable & auditabilité probante
**Réf. ADR :** ADR-009 §« Pattern B » · **Réf. PRD :** `docs/product/PRD - Contrats fournisseurs.md` US-CF-006 · **Réf. LEGAL :** §11.1
**Stories liées :** EP-006 (Contrats fournisseurs, ✅ terminée), EP-018-S02 (infra immutabilité, ✅), EP-018-S03 (Pattern A `levy_call` — modèle de référence `draft`/split migration, ✅), EP-018-S04 (Pattern A, ✅)

> Ce document est auto-suffisant : un développeur ou agent peut l'implémenter sans relire tout le repo. Les blocs de code sont tirés du code existant du repo.

**⚠️ Cette version remplace la v1 (rejetée le 2026-07-17)** : la v1 supposait un avenant créé directement `active` sans étape `draft`, en contradiction avec le PRD validé « Contrats fournisseurs » (§4, §5 US-CF-006). Le PRD fait foi. Cette v2 introduit le statut `draft` sur `vendor_contract` et suit le cycle `draft → active → terminated/expired` déjà documenté.

---

## 0. Décisions actées (Damien, 2026-07-17)

1. **Design de référence = PRD `Contrats fournisseurs`** (draft/activate), pas le raccourci « avenant direct → active » proposé en v1.
2. **Q-CF-001 tranchée : chaîne d'avenants illimitée** (profondeur libre — un contrat peut être amendé, puis son avenant ré-amendé, etc.). Pas de limite à un seul avenant par contrat actif — mais **un seul avenant `draft` non activé à la fois** par contrat actif (AC-CF-006-C, cf. §6).
3. **Chaîne d'avenants affichée dans une modale dédiée** `AmendmentChainModal.vue` (pas une section inline sur la fiche contrat).
4. **Champs descriptifs restent éditables sur un contrat `active`** (`vendor_name`, `vendor_email`, `vendor_phone`, `subject`) via l'action « Modifier » existante. Seuls les champs financiers/légaux (`amount`, `start_date`, `end_date`, `notice_days`, `contract_type`, `payment_frequency`, `auto_renewal`) sont gelés sur `active` et ne changent que via un avenant (PRD §2 « Immuabilité des contrats actifs » — liste `supplier_id`/`amount`/`start_date`/`end_date`/`notice_days` ; étendue ici à tous les champs contractuels structurants par cohérence avec ADR-009 Pattern B, à confirmer en implémentation si le PRD doit être amendé sur ce point précis — voir §9).

---

## 1. Quoi (résumé fonctionnel)

Aujourd'hui (code actuel), le syndic peut modifier en place n'importe quel champ d'un contrat fournisseur (`PUT /contracts/{id}`), y compris le montant et les dates, quel que soit son statut — l'enum `contract_status` ne connaît que `active|terminated|expired`, aucun gel n'est appliqué.

**Cible PRD (US-CF-001 à US-CF-006) :**

- Un contrat est créé au statut **`draft`** — librement modifiable, aucun rappel calculé.
- Une action explicite d'**activation** (`POST /contracts/{id}/activate`) le fait passer à **`active`** — à partir de là, les champs financiers/légaux sont gelés et les rappels sont calculés.
- Sur un contrat `active`, l'action « Modifier » reste disponible pour les champs descriptifs ; les champs gelés ne sont modifiables qu'en créant un **avenant** : `POST /contracts/{id}/amendments` crée un **nouveau contrat en `draft`**, pré-rempli avec les valeurs du contrat actif, `predecessor_id` pointant vers ce dernier.
- L'**activation de l'avenant** (même endpoint `POST /{id}/activate`) clôture l'ancien contrat (`terminated`) **en une transaction atomique** avec l'activation du nouveau.
- Un seul avenant `draft` peut coexister avec un contrat `active` donné (AC-CF-006-C) — une 2ᵉ tentative retourne `422`.
- La fiche contrat expose la **chaîne complète des avenants** (remontée/descente via `predecessor_id`), affichée dans une modale dédiée.

**Valeur légale/comptable :** conservation d'une piste probante de chaque acte contractuel distinct (art. L123-22 Code de commerce, conservation 10 ans ; LEGAL §11.1 ; ADR-009 Pattern B — « un avenant contractuel correspond à un nouvel acte signé à une date distincte »).

---

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Table `vendor_contract` | `backend/migrations/0001_init.sql` L475-493 |
| Enum `contract_status` (`'active','terminated','expired'`, défaut `'active'`) | `0001_init.sql` L101-103, L489 |
| FK entrantes vers `vendor_contract(id)` | `contract_document` (L496), `contract_reminder` (L505), `task.contract_id` (L547, nullable), `maintenance_log.contract_id` (L633, nullable) |
| Repo DB | `backend/src/db/contract.rs` (`VendorContractRow`, `CreateVendorContractInput`, `create`/`update`/`terminate`/`delete`/`get`/`list`) |
| Handlers API | `backend/src/api/contracts.rs` |
| Déclaration des routes | `backend/src/api/mod.rs` (bloc `// ── Contracts ──`, route `/contracts/{id}/terminate` existante) |
| Guards immutabilité | `backend/src/domain/immutability.rs` (enum `Entity`, `is_mutable`/`is_deletable`/`assert_mutable`) |
| **Modèle de référence exact** (Pattern A `levy_call`, split migration `draft`) | `backend/migrations/0030_levy_call_draft.sql` (ajout valeur enum seule, transaction dédiée) + `0033_levy_call_draft_status.sql` (colonne, backfill, trigger PG) |
| Modale édition | `frontend/src/features/contracts/ContractFormModal.vue` |
| Vue liste + cartes + actions | `frontend/src/features/contracts/ContractsView.vue` |
| Modale résiliation | `frontend/src/features/contracts/TerminateContractModal.vue` |
| API client | `frontend/src/api/index.ts` (`contractsApi`) |
| Types | `frontend/src/types/domain.ts` (`ContractStatus`, `VendorContract`) |
| PRD de référence (fait foi) | `docs/product/PRD - Contrats fournisseurs.md` §4 (cycle de vie), §5 US-CF-005/US-CF-006 |
| Prochain n° migration | **0036** (dernière = `0035_governance_register.sql`) |

---

## 3. Modèle de données

### 3.1 Contrainte PostgreSQL — ajout de valeur enum en transaction séparée

PostgreSQL interdit d'utiliser une valeur d'enum fraîchement ajoutée dans la **même transaction** qui l'a créée. `sqlx` exécute chaque fichier de migration dans sa propre transaction — donc, comme pour `levy_status`/`draft` (0030 + 0033), il faut **deux migrations distinctes**.

### 3.2 Migration `backend/migrations/0036_vendor_contract_draft_status.sql` (partie 1 — valeur enum seule)

```sql
-- EP-018-S05: Pattern B — vendor_contract draft/active/amendment lifecycle (ADR-009, PRD Contrats fournisseurs §4)
--
-- PostgreSQL forbids using a freshly added enum value in the same transaction
-- that adds it. The column/backfill/trigger/predecessor_id that consume
-- 'draft' live in 0037 (a later, separate transaction). Mirrors the
-- levy_call precedent (0030 + 0033).

ALTER TYPE contract_status ADD VALUE IF NOT EXISTS 'draft' BEFORE 'active';
```

### 3.3 Migration `backend/migrations/0037_vendor_contract_predecessor.sql` (partie 2 — colonne, backfill, trigger)

```sql
-- EP-018-S05: Pattern B — vendor_contract amendments (part 2)
--
-- An amendment is a distinct legal act, not an in-place edit. Adding
-- predecessor_id turns vendor_contract into an explicit temporal chain:
-- each amendment is created in 'draft', pre-filled from its predecessor.
-- Activating the amendment closes the predecessor (status='terminated')
-- atomically. The chain head (original contract) has predecessor_id NULL.
--
-- All pre-migration rows were created directly 'active' (no draft concept
-- existed before this migration) — no backfill needed for status, they
-- simply keep their current status.

ALTER TABLE vendor_contract
    ADD COLUMN predecessor_id UUID REFERENCES vendor_contract(id);

-- One contract can have at most one *pending* (draft) amendment at a time —
-- enforced at the application layer (§6, AC-CF-006-C), not by this index.
-- This index only guarantees a linear chain once activated: a given
-- predecessor can have at most one activated successor recorded against it.
CREATE UNIQUE INDEX vendor_contract_predecessor_unique
    ON vendor_contract (predecessor_id)
    WHERE predecessor_id IS NOT NULL;

CREATE INDEX vendor_contract_predecessor_idx
    ON vendor_contract (predecessor_id);

-- Defensive trigger mirroring domain/immutability.rs: financial/legal fields
-- are frozen once the contract leaves 'draft'. Descriptive fields
-- (vendor_name, vendor_email, vendor_phone, subject) stay mutable via the
-- regular UPDATE path regardless of status — enforced by comparing OLD/NEW
-- only on the frozen columns.
CREATE OR REPLACE FUNCTION trg_vendor_contract_immutable()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.status <> 'draft' AND (
        NEW.amount IS DISTINCT FROM OLD.amount OR
        NEW.start_date IS DISTINCT FROM OLD.start_date OR
        NEW.end_date IS DISTINCT FROM OLD.end_date OR
        NEW.notice_days IS DISTINCT FROM OLD.notice_days OR
        NEW.contract_type IS DISTINCT FROM OLD.contract_type OR
        NEW.payment_frequency IS DISTINCT FROM OLD.payment_frequency OR
        NEW.auto_renewal IS DISTINCT FROM OLD.auto_renewal
    ) THEN
        RAISE EXCEPTION 'vendor_contract % is frozen on financial/legal fields (status=%)', OLD.id, OLD.status;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER vendor_contract_immutable
    BEFORE UPDATE ON vendor_contract
    FOR EACH ROW EXECUTE FUNCTION trg_vendor_contract_immutable();
```

**Notes de conception :**
- `predecessor_id` **nullable** : les contrats d'origine ont `predecessor_id IS NULL`.
- FK **auto-référentielle**, conforme ADR-009 §Pattern B.
- L'index unique partiel garantit qu'un prédécesseur donné n'a **au plus un avenant activé**. Il n'empêche pas, à lui seul, la création d'un 2ᵉ avenant `draft` concurrent — ce garde-fou est applicatif (§4.3, AC-CF-006-C) car il porte sur l'existence d'un avenant **non encore activé**, une notion transitoire qu'un index ne peut pas exprimer proprement.
- Le trigger PG est défensif (miroir de `domain/immutability.rs`), pas la seule ligne de défense — la logique métier doit rejeter la requête **avant** d'atteindre la base pour retourner une erreur i18n lisible plutôt qu'une violation de contrainte brute.

Après migration : régénérer les données offline sqlx et committer.
```bash
cargo sqlx migrate run
cargo sqlx prepare        # met à jour backend/.sqlx/ (committé)
```

---

## 4. Backend Rust

### 4.1 `backend/src/domain/immutability.rs` — ajouter la variante `VendorContract`

```rust
    /// Financial/legal fields (`amount`, `start_date`, `end_date`, `notice_days`,
    /// `contract_type`, `payment_frequency`, `auto_renewal`) frozen once
    /// `status != 'draft'`. Amended via `POST /contracts/{id}/amendments`
    /// (Pattern B, ADR-009, PRD Contrats fournisseurs §4).
    VendorContract,
```

```rust
        Entity::VendorContract => status == "draft",
```

`is_deletable` hérite de `is_mutable` — un contrat `draft` reste supprimable, un contrat `active`/`terminated`/`expired` ne l'est plus directement (suppression bloquée aussi si des avenants existent, cf. §4.5).

Ajouter un test unitaire (§7.1).

### 4.2 `backend/src/db/contract.rs` — struct, statut par défaut, chaîne, avenant, activation

**(a) `VendorContractRow` — ajouter `predecessor_id`** (après `pub status: String,`) :
```rust
    pub predecessor_id: Option<Uuid>,
```
Propager `vc.predecessor_id` dans tous les `SELECT`/`RETURNING` du fichier (`list`, `get`, `create` RETURNING, `update` RETURNING).

**(b) `create` — le contrat est désormais créé `draft`**, pas `active`. La colonne `status` a pour défaut `'draft'` côté DB (aucun changement de code nécessaire dans l'`INSERT` si le défaut est posé en migration ; sinon, insérer explicitement `status = 'draft'`). **Aucun rappel n'est calculé à la création** (PRD AC-CF-001-C) — retirer le bloc de calcul de rappels de `create` (L200-215 actuelles) : il est déplacé dans `activate` (voir (d)).

**(c) `CreateAmendmentInput`** :
```rust
/// Payload for `POST /contracts/{id}/amendments`. Creates a new contract in
/// `draft`, pre-filled from the active predecessor, linked via
/// predecessor_id. The predecessor is NOT closed yet — closure happens on
/// activation of this amendment (see `activate`).
#[derive(Debug, Deserialize)]
pub struct CreateAmendmentInput {
    #[serde(flatten)]
    pub contract: CreateVendorContractInput,
}
```

**(d) `create_amendment`** — crée le nouveau contrat en `draft`, lié au prédécesseur. Ne clôture PAS encore l'ancien (ça, c'est le rôle d'`activate`) :
```rust
/// Pattern B (ADR-009): creates a draft amendment linked to an active
/// contract via predecessor_id. Enforces AC-CF-006-C: only one pending
/// (draft) amendment per predecessor at a time.
pub async fn create_amendment(
    pool: &PgPool,
    predecessor_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
    input: CreateAmendmentInput,
) -> AppResult<Option<VendorContractRow>> {
    let mut tx = pool.begin().await?;

    let predecessor = sqlx::query_as::<_, VendorContractRow>(
        r#"SELECT vc.id, vc.building_id, vc.vendor_name, vc.vendor_email, vc.vendor_phone,
                  vc.subject, vc.contract_type::text AS contract_type, vc.start_date, vc.end_date,
                  vc.amount, vc.payment_frequency::text AS payment_frequency, vc.auto_renewal,
                  vc.notice_days, vc.status::text AS status, vc.predecessor_id, vc.created_at,
                  0::bigint AS document_count
           FROM vendor_contract vc
           WHERE vc.id = $1 AND vc.building_id = $2
           FOR UPDATE"#,
    )
    .bind(predecessor_id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?;

    let predecessor = match predecessor {
        Some(row) => row,
        None => return Ok(None),
    };

    if predecessor.status != "active" {
        return Err(AppError::validation_code("errors.contracts.not_amendable"));
    }

    // AC-CF-006-C: only one pending draft amendment per predecessor.
    let existing_draft = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM vendor_contract WHERE predecessor_id = $1 AND status = 'draft')",
    )
    .bind(predecessor_id)
    .fetch_one(&mut *tx)
    .await?;
    if existing_draft {
        return Err(AppError::validation_code("errors.contracts.amendment_already_pending"));
    }

    let c = &input.contract;
    let row = sqlx::query_as::<_, VendorContractRow>(
        r#"INSERT INTO vendor_contract
           (building_id, vendor_name, vendor_email, vendor_phone, subject, contract_type,
            start_date, end_date, amount, payment_frequency, auto_renewal, notice_days,
            predecessor_id, status, created_by)
           VALUES ($1,$2,$3,$4,$5,$6::text::contract_type,$7,$8,$9,
                   $10::text::payment_frequency,$11,$12,$13,'draft',$14)
           RETURNING id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text AS contract_type, start_date, end_date, amount,
           payment_frequency::text AS payment_frequency, auto_renewal, notice_days,
           status::text AS status, predecessor_id, created_at, 0::bigint AS document_count"#,
    )
    .bind(building_id)
    .bind(&c.vendor_name)
    .bind(&c.vendor_email)
    .bind(&c.vendor_phone)
    .bind(&c.subject)
    .bind(&c.contract_type)
    .bind(c.start_date)
    .bind(c.end_date)
    .bind(c.amount)
    .bind(&c.payment_frequency)
    .bind(c.auto_renewal)
    .bind(c.notice_days)
    .bind(predecessor_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(row))
}
```

**(e) `activate`** — endpoint générique d'activation (couvre à la fois « premier contrat » et « avenant »). Si le contrat a un `predecessor_id`, clôture le prédécesseur en `terminated` **dans la même transaction** ; calcule les rappels dans tous les cas :
```rust
/// Activates a draft contract: status -> 'active', reminders computed.
/// If the contract has a predecessor (amendment), the predecessor is closed
/// ('terminated') atomically — Pattern B activation (PRD AC-CF-006-B).
pub async fn activate(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<VendorContractRow>> {
    let mut tx = pool.begin().await?;

    let current = sqlx::query_as::<_, VendorContractRow>(
        r#"SELECT vc.id, vc.building_id, vc.vendor_name, vc.vendor_email, vc.vendor_phone,
                  vc.subject, vc.contract_type::text AS contract_type, vc.start_date, vc.end_date,
                  vc.amount, vc.payment_frequency::text AS payment_frequency, vc.auto_renewal,
                  vc.notice_days, vc.status::text AS status, vc.predecessor_id, vc.created_at,
                  0::bigint AS document_count
           FROM vendor_contract vc
           WHERE vc.id = $1 AND vc.building_id = $2
           FOR UPDATE"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?;

    let current = match current {
        Some(row) => row,
        None => return Ok(None),
    };
    if current.status != "draft" {
        return Err(AppError::validation_code("errors.contracts.not_activatable"));
    }

    let row = sqlx::query_as::<_, VendorContractRow>(
        r#"UPDATE vendor_contract SET status = 'active', updated_at = NOW()
           WHERE id = $1
           RETURNING id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text AS contract_type, start_date, end_date, amount,
           payment_frequency::text AS payment_frequency, auto_renewal, notice_days,
           status::text AS status, predecessor_id, created_at, 0::bigint AS document_count"#,
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    // AC-CF-006-B: activating an amendment closes its predecessor atomically.
    if let Some(predecessor_id) = row.predecessor_id {
        sqlx::query!(
            "UPDATE vendor_contract SET status = 'terminated', updated_at = NOW() WHERE id = $1",
            predecessor_id,
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM contract_reminder WHERE contract_id = $1 AND status = 'pending'",
            predecessor_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Compute reminders now that the contract is active (PRD §"Calcul des rappels").
    let spec = ContractReminderSpec {
        start_date: row.start_date,
        end_date: row.end_date,
        auto_renewal: row.auto_renewal,
        notice_days: row.notice_days.map(|d| d as u32),
        payment_frequency_months: frequency_to_months(row.payment_frequency.as_deref()),
    };
    let today = chrono::Local::now().date_naive();
    for reminder in compute_reminders(&spec, today) {
        let kind_str = reminder_kind_to_str(&reminder.kind);
        sqlx::query!(
            "INSERT INTO contract_reminder (contract_id, reminder_type, reminder_date) VALUES ($1, $2::text::reminder_type, $3)",
            row.id, kind_str, reminder.reminder_date
        ).execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(Some(row))
}
```

**(f) `amendment_chain`** — historique complet via `predecessor_id` (CTE récursive ascendante + descendante), ordonné chronologiquement :
```rust
/// Returns the full amendment chain the given contract belongs to,
/// ordered chronologically (original first, latest last).
pub async fn amendment_chain(
    pool: &PgPool,
    contract_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<VendorContractRow>> {
    Ok(sqlx::query_as::<_, VendorContractRow>(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT id, predecessor_id FROM vendor_contract WHERE id = $1 AND building_id = $2
            UNION
            SELECT vc.id, vc.predecessor_id
            FROM vendor_contract vc
            JOIN ancestors a ON vc.id = a.predecessor_id
        ),
        descendants AS (
            SELECT id FROM vendor_contract WHERE id = $1 AND building_id = $2
            UNION
            SELECT vc.id
            FROM vendor_contract vc
            JOIN descendants d ON vc.predecessor_id = d.id
        ),
        chain AS (
            SELECT id FROM ancestors
            UNION
            SELECT id FROM descendants
        )
        SELECT vc.id, vc.building_id, vc.vendor_name, vc.vendor_email, vc.vendor_phone,
               vc.subject, vc.contract_type::text AS contract_type, vc.start_date, vc.end_date,
               vc.amount, vc.payment_frequency::text AS payment_frequency, vc.auto_renewal,
               vc.notice_days, vc.status::text AS status, vc.predecessor_id, vc.created_at,
               0::bigint AS document_count
        FROM vendor_contract vc
        JOIN chain c ON c.id = vc.id
        WHERE vc.building_id = $2
        ORDER BY vc.start_date, vc.created_at
        "#,
    )
    .bind(contract_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}
```
> Note : la CTE ascendante corrige le bug de la v1 (qui remontait via `a.id` au lieu de `a.predecessor_id`, ne remontant jamais réellement la chaîne). Support d'une profondeur illimitée conforme à la décision Q-CF-001.

### 4.3 `backend/src/api/contracts.rs` — nouveaux handlers

**(a) Handler création d'avenant** :
```rust
pub async fn create_amendment(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<db::contract::CreateAmendmentInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let created = db::contract::create_amendment(&state.pool, id, auth.building_id, auth.user_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok((StatusCode::CREATED, json_value(created)?))
}
```

**(b) Handler activation** :
```rust
pub async fn activate(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let activated = db::contract::activate(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(activated)
}
```

**(c) Handler chaîne d'avenants** :
```rust
pub async fn amendment_chain(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    json_value(db::contract::amendment_chain(&state.pool, id, auth.building_id).await?)
}
```

### 4.4 `backend/src/api/contracts.rs` — durcir `update`

Le `update` actuel ne pose aucun guard. Bloquer uniquement le changement des champs gelés sur un contrat non-`draft` ; les champs descriptifs restent éditables quel que soit le statut (décision §0.4) :
```rust
pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<db::contract::CreateVendorContractInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let current = db::contract::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    use crate::domain::immutability::{Entity, is_mutable};
    if !is_mutable(&Entity::VendorContract, &current.status) {
        let frozen_field_changed = input.amount != current.amount
            || input.start_date != current.start_date
            || input.end_date != current.end_date
            || input.notice_days != current.notice_days
            || input.contract_type != current.contract_type
            || input.payment_frequency != current.payment_frequency
            || input.auto_renewal != current.auto_renewal;
        if frozen_field_changed {
            return Err(AppError::validation_code("errors.immutability.frozen"));
        }
    }

    let c = db::contract::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(c)
}
```

### 4.5 `backend/src/db/contract.rs` — durcir `delete`

Interdire la suppression d'un contrat qui est prédécesseur d'un avenant (préserve la chaîne probante). Ajouter avant le `DELETE` final :
```rust
    let has_successor = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM vendor_contract WHERE predecessor_id = $1)",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if has_successor {
        return Err(crate::errors::AppError::validation_code(
            "errors.contracts.has_successor",
        ));
    }
```

### 4.6 `backend/src/api/mod.rs` — déclarer les routes

Dans le bloc `// ── Contracts ──`, après la route `/contracts/{id}/terminate`, ajouter :
```rust
        .route("/contracts/{id}/activate", post(contracts::activate))
        .route(
            "/contracts/{id}/amendments",
            post(contracts::create_amendment).get(contracts::amendment_chain),
        )
```

### 4.7 Contrat d'API (récapitulatif REST)

**Création d'avenant**
```
POST /api/v1/contracts/{id}/amendments
Rôle : manager
Body : mêmes champs que CreateVendorContractInput (sans effective_date — la date d'effet
       EST start_date du nouveau contrat, saisie librement par le syndic)
Réponses :
  201 Created → nouveau contrat, status="draft", predecessor_id={id}
  400 (errors.contracts.not_amendable)              → prédécesseur non actif
  400 (errors.contracts.amendment_already_pending)  → un avenant draft existe déjà pour ce prédécesseur
  403 → rôle insuffisant
  404 → contrat inexistant / autre immeuble
```

**Activation**
```
POST /api/v1/contracts/{id}/activate
Rôle : manager
Réponses :
  200 OK → contrat activé (status="active"), rappels calculés
           si predecessor_id renseigné : le prédécesseur passe à "terminated" (atomique)
  400 (errors.contracts.not_activatable) → contrat déjà non-draft
  403 → rôle insuffisant
  404 → contrat inexistant / autre immeuble
```

**Chaîne d'avenants**
```
GET /api/v1/contracts/{id}/amendments
Rôle : manager | board_member
Réponse 200 → tableau de VendorContract ordonné (origine → plus récent)
  404 → contrat inexistant / autre immeuble
```

**`PUT /contracts/{id}`** (inchangé côté route) : `400 errors.immutability.frozen` si tentative de modifier un champ gelé sur un contrat non-`draft`.

---

## 5. Frontend Vue

### 5.1 `frontend/src/types/domain.ts`

```ts
export type ContractStatus = 'draft' | 'active' | 'terminated' | 'expired'
```
Ajouter à `VendorContract` : `predecessor_id?: string | null`.

### 5.2 `frontend/src/api/index.ts` — `contractsApi`

```ts
createAmendment: (contractId: string, payload: CreateVendorContractPayload) =>
  api.post<VendorContract>(`/contracts/${contractId}/amendments`, payload),
activate: (contractId: string) =>
  api.post<VendorContract>(`/contracts/${contractId}/activate`),
amendmentChain: (contractId: string) =>
  api.get<VendorContract[]>(`/contracts/${contractId}/amendments`),
```

### 5.3 `ContractsView.vue` — actions

Sur un contrat `draft` : « Activer » (appelle `activate`, confirmation simple — les rappels seront calculés).
Sur un contrat `active` :
- « Modifier » → `ContractFormModal` en mode restreint (champs gelés en lecture seule, cf. §4.4).
- « Créer un avenant » → `AmendContractModal` (pré-rempli depuis le contrat courant) ; à la soumission, avenant créé en `draft` — informer l'utilisateur qu'il doit encore l'**activer** pour qu'il prenne effet (l'avenant `draft` apparaît dans la liste, filtrable par statut `draft`, avec un bouton « Activer » dédié qui, une fois cliqué, clôture le prédécesseur).
- « Voir l'historique » → `AmendmentChainModal` (visible si `predecessor_id` renseigné ou si un successeur existe).
- « Résilier » (inchangé, `TerminateContractModal`).

### 5.4 `ContractFormModal.vue`

Champs gelés (`amount`, `start_date`, `end_date`, `notice_days`, `contract_type`, `payment_frequency`, `auto_renewal`) passés en `readonly`/désactivés dès que `status !== 'draft'`. Champs descriptifs (`vendor_name`, `vendor_email`, `vendor_phone`, `subject`) toujours éditables.

### 5.5 Nouveau composant `AmendContractModal.vue`

Props : `contractId`. Formulaire identique à `ContractFormModal` en mode création, pré-rempli avec les valeurs du contrat actif (fetch initial ou props passées par le parent). Soumission → `contractsApi.createAmendment(contractId, payload)` → `201` → fermeture + toast « Avenant créé en brouillon — activez-le pour qu'il prenne effet. » + rafraîchissement liste.

### 5.6 Nouveau composant `AmendmentChainModal.vue`

Props : `contractId`. Fetch au montage via `contractsApi.amendmentChain(contractId)`. Affichage chronologique (timeline) avec, par maillon : dates, montant, statut (badge), lien vers la fiche détail si différent du contrat courant.

### 5.7 `docs/DESIGN.md`

Ajouter `AmendContractModal` et `AmendmentChainModal` au catalogue §2 avant utilisation (règle AGENTS.md : « New component → add to catalogue in docs/DESIGN.md §2 before use »).

---

## 6. Cas limites et règles métier

| Cas | Règle |
|---|---|
| **Contrat `draft`** | Librement modifiable (`PUT` sans restriction), aucun rappel calculé, supprimable sans condition (hors documents rattachés, cf. PRD AC-CF-007-C). |
| **Contrat déjà amendé (avenant `draft` en attente)** | Une 2ᵉ tentative de `POST /amendments` sur le même prédécesseur échoue `400 errors.contracts.amendment_already_pending` (AC-CF-006-C). |
| **Chaîne multi-avenants** | Autorisée sans limite de profondeur (Q-CF-001 tranchée : illimité). Un avenant activé peut lui-même être amendé une fois `active`. |
| **Activation d'un avenant** | Clôture atomique du prédécesseur (`terminated`) + suppression des rappels `pending` du prédécesseur + calcul des rappels du nouveau (§4.2e). |
| **Résiliation vs avenant** | Restent des flux **distincts**. `POST /{id}/terminate` (existant) : fin sans successeur. `POST /{id}/amendments` + `POST /{nouveau_id}/activate` : fin **avec** succession. Un contrat `terminated` ne peut plus être amendé (`errors.contracts.not_amendable`, le check porte sur `status == "active"`). |
| **Suppression d'un prédécesseur** | Interdite si un successeur existe (`errors.contracts.has_successor`, §4.5). |
| **FK entrantes (`task`, `maintenance_log`, `contract_document`, `contract_reminder`)** | Restent attachées au contrat d'origine référencé ; aucun re-parentage automatique lors d'un avenant. |
| **Assurance MRI (US-CF-008)** | Un avenant sur un contrat `ASSURANCE` suit le même flux ; pendant la fenêtre où le prédécesseur est encore `active` et l'avenant `draft` non activé, l'alerte de couverture (AC-CF-008-A) reste calculée sur le contrat `active`, pas sur le `draft`. |

---

## 7. Plan de tests

Convention repo : tests domaine purs (`backend/tests/*_domain_test.rs`), tests unitaires guards (`#[cfg(test)] mod tests` dans `domain/immutability.rs`), tests HTTP e2e (`backend/tests/api_e2e_test.rs`, harness `common::isolated_test_server()`).

### 7.1 Tests unitaires guard — `domain/immutability.rs`
```rust
#[test]
fn vendor_contract_draft_is_mutable() {
    assert!(is_mutable(&Entity::VendorContract, "draft"));
    assert!(!is_mutable(&Entity::VendorContract, "active"));
    assert!(!is_mutable(&Entity::VendorContract, "terminated"));
    assert!(!is_mutable(&Entity::VendorContract, "expired"));
}
```

### 7.2 Tests HTTP e2e — `api_e2e_test.rs`

1. **Création** : `POST /contracts` → `201`, `status == "draft"`, aucun rappel en base.
2. **Activation** : `POST /contracts/{id}/activate` → `200`, `status == "active"`, rappels calculés (si `notice_days`/`end_date` renseignés).
3. **Re-activation refusée** : `POST /activate` sur un contrat déjà `active` → `400 errors.contracts.not_activatable`.
4. **Modification champ descriptif sur contrat actif** : `PUT` changeant `vendor_email` seul → `200`.
5. **Modification champ gelé sur contrat actif** : `PUT` changeant `amount` → `400 errors.immutability.frozen`.
6. **Création d'avenant** : `POST /contracts/{id}/amendments` sur un contrat `active` → `201`, `status == "draft"`, `predecessor_id == id`.
7. **Avenant déjà pendant** : 2ᵉ `POST /amendments` sur le même prédécesseur → `400 errors.contracts.amendment_already_pending`.
8. **Activation de l'avenant clôture le prédécesseur** : `POST /{avenant_id}/activate` → `200` ; `GET /contracts/{predecessor_id}` → `status == "terminated"`.
9. **Chaîne** : `GET /contracts/{n'importe quel id de la chaîne}/amendments` → tableau complet, ordonné, identique quel que soit le maillon interrogé.
10. **Chaîne multi-niveaux** : créer avenant sur avenant activé (3 maillons) → `GET /amendments` sur le maillon 1 et le maillon 3 renvoient les 3 mêmes éléments.
11. **Non amendable** : `POST /amendments` sur un contrat `terminated` → `400 errors.contracts.not_amendable`.
12. **Suppression bloquée** : `DELETE /contracts/{predecessor_id}` une fois un successeur créé → `errors.contracts.has_successor`.

### 7.3 Frontend

Le repo ne montre pas de suite de tests composants Vue pour `contracts/`. Si `vitest` est configuré, ajouter au minimum un test de `AmendContractModal` (soumission → appel `createAmendment` mocké) et `AmendmentChainModal` (fetch au montage). Sinon, s'en tenir à la couverture backend.

---

## 8. Ordre d'implémentation suggéré

1. **Migrations** `0036_vendor_contract_draft_status.sql` (valeur enum seule) puis `0037_vendor_contract_predecessor.sql` (colonne + trigger) → `cargo sqlx migrate run` → `cargo sqlx prepare` (commit `backend/.sqlx/`).
2. **Backend domaine** : `Entity::VendorContract` + `is_mutable` + test unitaire → `cargo test --lib`.
3. **Backend db** : `predecessor_id` sur `VendorContractRow`, `create` (défaut `draft`, retrait calcul rappels), `create_amendment`, `activate`, `amendment_chain`, guard `delete` → `cargo build`.
4. **Backend api** : handlers `create_amendment`/`activate`/`amendment_chain`, durcissement `update`, routes → `cargo build` + `cargo clippy -- -D warnings`.
5. **Tests backend** : e2e (§7.2), boucler jusqu'au vert.
6. **Frontend types + api** : `ContractStatus`, `predecessor_id`, `contractsApi.createAmendment/activate/amendmentChain`.
7. **Frontend composants** : `AmendContractModal.vue`, `AmendmentChainModal.vue` — cataloguer dans `docs/DESIGN.md §2` avant usage.
8. **Frontend vue** : `ContractsView.vue` (actions conditionnelles par statut), `ContractFormModal.vue` (champs gelés en lecture seule sur non-`draft`).
9. **Vérif finale** : `cargo test --lib --tests`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `npm --prefix frontend run build`, `npm --prefix frontend run lint`.
10. **Docs & suivi** : `docs/ARCHITECTURE.md` (déjà à jour — mentionne `predecessor_id` sur `vendor_contract`, vérifier la cohérence avec le cycle `draft` ajouté), `docs/EPICS.md` (statut `EP-018-S05`), `docs/PROGRESS.md` (entrée datée), `graphify update .`.

---

## 9. Points ouverts — tranchés

- **Périmètre du gel (confirmé 2026-07-18)** : gel étendu retenu — `amount`, `start_date`, `end_date`, `notice_days`, `contract_type`, `payment_frequency`, `auto_renewal` sont tous gelés sur un contrat non-`draft` (§3.3, §4.4). Le PRD §2 liste une sous-partie de ces champs ; la spec l'étend par cohérence Pattern B. Le PRD sera mis à jour en conséquence (§10) pour refléter ce périmètre élargi.
- **`supplier_id`** : n'existe pas dans le schéma actuel (`vendor_name`/`vendor_email`/`vendor_phone` en champs libres, pas de FK vers un référentiel fournisseur) — le PRD semble anticiper une évolution vers un référentiel fournisseur distinct, hors périmètre de cette story.

## 10. Mise à jour PRD requise avant implémentation

`docs/product/PRD - Contrats fournisseurs.md` doit être corrigé pour rester la source de vérité :
- §2 « Immuabilité des contrats actifs » — étendre la liste des champs gelés à `contract_type`, `payment_frequency`, `auto_renewal`.
- §5 AC-CF-006 — préciser la profondeur illimitée de la chaîne d'avenants (Q-CF-001).
- §9 Q-CF-001 — marquer comme tranchée (réponse : illimitée), retirer de la table des questions ouvertes.
