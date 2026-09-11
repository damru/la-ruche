# Spécification technique — EP-002-S12

**Story :** PV du conseil syndical en délégation de pouvoirs : créer, stocker et consulter les PV formels des décisions prises par le conseil syndical sous délégation d'AG (signé par 2 membres, noms + sens des votes, inscrit au registre des PV d'AG) — art. 21-1 décret 67-223, LEGIARTI000042077775 (LEGAL.md §3.5)

**Épic :** EP-002 — Copropriétaires & lots
**Réf. LEGAL :** §3.5/§3.6 (PV du conseil en délégation) · art. 21-1 décret 67-223 · **ADR :** aucun requis (table dédiée standard + JSONB de votes)
**Stories liées :** EP-003 (PV d'AG), registre des mandats de gouvernance ✅ (membres du conseil disponibles)

**Statut initial :** NON IMPLÉMENTÉ — aucune entité de PV du conseil ; toute la machinerie de PV est scopée aux assemblées.

---

## 1. Quoi (résumé fonctionnel)

Quand l'AG délègue un pouvoir au conseil syndical (art. 21-1 à 21-5 loi 1965), chaque décision prise en délégation doit faire l'objet d'un **procès-verbal formel** : nom des membres présents et **sens de leur vote**, **signé par au moins 2 membres** du conseil, et **inscrit au registre des PV d'AG**.

- Le syndic (ou un membre du conseil) crée un PV de décision : objet, date, référence de la délégation (résolution d'AG), participants + sens des votes.
- Le PV est signé par 2 membres minimum ; tant que < 2 signatures, il reste en brouillon modifiable ; à 2 signatures il devient définitif et immuable.
- Les PV définitifs apparaissent dans le **registre des PV d'AG** (listing commun avec les PV d'assemblée), en lecture pour les copropriétaires.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Machinerie PV existante — **scopée assemblées uniquement** | `backend/src/api/assemblies.rs:551-598` (`get_minutes_draft` / `attach_minutes` / `sign_minutes`), `backend/src/domain/minutes_draft.rs` |
| Membres du conseil syndical | registre des mandats ✅ : migration `backend/migrations/0035_governance_register.sql`, API `backend/src/api/governance.rs`, vue `frontend/src/features/registers/GovernanceMandatesView.vue` |
| PV du conseil | **inexistant** |
| Guards immutabilité | `backend/src/domain/immutability.rs` (à étendre pour le PV signé) |

## 3. Modèle de données

Migration `backend/migrations/NNNN_council_decision_pv.sql` (numéro en flux sur cette branche) :

```sql
-- EP-002-S12: council decision minutes under AG delegation (art. 21-1 decret 67-223)
CREATE TYPE council_pv_status AS ENUM ('draft', 'signed');

CREATE TABLE council_decision_pv (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES building(id),
    decision_date DATE NOT NULL,
    subject TEXT NOT NULL,
    delegation_reference TEXT NOT NULL,  -- AG resolution granting the delegation
    -- [{"member_name": "...", "mandate_id": "...", "vote": "for|against|abstain"}]
    votes JSONB NOT NULL,
    -- [{"member_name": "...", "mandate_id": "...", "signed_at": "..."}] (>= 2 to sign off)
    signatures JSONB NOT NULL DEFAULT '[]',
    status council_pv_status NOT NULL DEFAULT 'draft',
    created_by UUID NOT NULL REFERENCES app_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX council_decision_pv_building_idx ON council_decision_pv (building_id);
```

JSONB pour votes/signatures : structure de restitution documentaire, pas de requête relationnelle nécessaire — pas d'alternative structurelle réelle, donc pas d'ADR. Étape associée : **mettre à jour `docs/ARCHITECTURE.md`** avant la migration.

## 4. Backend

### 4.1 Domain (`backend/src/domain/council_pv.rs`, nouveau)

- Validation pure : au moins 1 participant, `vote ∈ {for, against, abstain}`, `delegation_reference` non vide.
- `can_sign_off(signatures) -> bool` : ≥ 2 signatures de **membres distincts** du conseil.
- Immutabilité : étendre `domain/immutability.rs` (variante `Entity::CouncilPv`) — `is_mutable`/`is_deletable` = `status == draft`. Un PV `signed` n'est ni modifiable ni supprimable.

### 4.2 DB (`backend/src/db/council_pv.rs`, nouveau)

`create`, `update` (draft uniquement — le guard est vérifié en domain avant l'appel), `add_signature` (append dans le JSONB + passage à `signed` quand le seuil est atteint, en une transaction), `get`, `list_for_building` (filtre statut), `delete` (draft uniquement).

### 4.3 API (`backend/src/api/council_pv.rs`, nouveau ; routes dans `backend/src/api/mod.rs`)

Rôles : **manager + membres du conseil** en écriture ; **copropriétaires en lecture seule** (PV `signed` uniquement).

- `POST /council-pvs` — création (draft).
- `PUT /council-pvs/{id}` — édition ; sur `signed` → `AppError::validation_code("errors.council_pv.immutable_after_signing")`.
- `DELETE /council-pvs/{id}` — draft uniquement, sinon `errors.council_pv.immutable_after_signing`.
- `POST /council-pvs/{id}/signatures` — ajoute une signature ; signataire non membre du conseil (vérif via registre des mandats, `api/governance.rs`) → `errors.council_pv.signer_not_council_member` ; doublon → `errors.council_pv.already_signed_by_member` ; à 2 signatures le statut passe à `signed`.
- `GET /council-pvs` / `GET /council-pvs/{id}` — lecture (copropriétaires : `signed` seulement).

**Inscription au registre des PV d'AG :** le listing du registre des PV (côté assemblées) inclut les `council_decision_pv` signés — soit en fusionnant les deux sources dans l'endpoint de listing existant du registre, soit via un endpoint registre dédié consommé par la même vue. Choix d'implémentation local ; l'exigence est que la vue registre affiche PV d'AG **et** PV du conseil signés, chronologiquement.

## 5. Frontend

- `frontend/src/api/index.ts` : `councilPvApi` (CRUD + sign).
- `frontend/src/types/domain.ts` : `CouncilDecisionPv`, `CouncilPvVote`, `CouncilPvSignature`, `CouncilPvStatus`.
- Vue `frontend/src/features/registers/CouncilPvView.vue` (cohérente avec `JudicialProceedingsView.vue` / `GovernanceMandatesView.vue`) : liste, formulaire (participants pré-remplis depuis le registre des mandats, sélecteur de sens de vote), action « Signer » (2 signatures requises, progression affichée), badge draft/signé.
- Intégration au registre des PV d'AG : les PV du conseil signés apparaissent dans le listing des PV.
- **Tout nouveau composant `C*` doit être catalogué dans `docs/DESIGN.md §3` avant usage** (étape dédiée).

## 6. Cas limites

- 2 signatures du **même** membre → refusé (`already_signed_by_member`) ; le seuil compte des membres distincts.
- Signataire dont le mandat au conseil est expiré à la date de signature → refusé (`signer_not_council_member`).
- Modification/suppression après `signed` → 422 immuable (guard domain + vérification handler→domain, jamais dans le handler lui-même).
- Conseil de 2 membres seulement → le PV exige quand même 2 signatures (minimum légal).
- Copropriétaire demandant un PV `draft` → 404/403 (invisible hors manager+conseil).
- Participant listé sans vote exprimé → interdit à la validation (`errors.council_pv.vote_required`).

## 7. Critères d'acceptation

1. Création d'un PV avec objet, date, référence de délégation, participants + sens des votes.
2. Passage à `signed` uniquement à ≥ 2 signatures de membres distincts du conseil (vérifiés via le registre des mandats).
3. PV `signed` immuable et non supprimable (guard `domain/immutability.rs`).
4. PV signés visibles dans le registre des PV d'AG et consultables par les copropriétaires ; drafts réservés manager+conseil.
5. Erreurs i18n `errors.council_pv.*` ; `docs/ARCHITECTURE.md` documente la table.

## 8. Plan de tests

- **Domaine pur** : `can_sign_off` (0/1/2 signatures, doublon de membre) ; validation votes (valeur invalide, participant sans vote) ; `is_mutable`/`is_deletable` selon statut.
- **E2E** (`backend/tests/api_e2e_test.rs`, `common::isolated_test_server()`) :
  - création → 1 signature (reste draft) → 2ᵉ signature (→ `signed`) ;
  - PUT/DELETE sur `signed` → 422 `errors.council_pv.immutable_after_signing` ;
  - signature par un non-membre → 422 ; double signature même membre → 422 ;
  - copropriétaire : liste ne contient que les `signed` ; le listing registre des PV inclut le PV du conseil signé.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` → migration `NNNN_council_decision_pv.sql` → `cargo sqlx migrate run` → `cargo sqlx prepare`.
2. `domain/council_pv.rs` + extension `domain/immutability.rs` + tests purs.
3. `db/council_pv.rs`.
4. `api/council_pv.rs` + routes `api/mod.rs` + intégration listing registre des PV + erreurs i18n.
5. Frontend (types, `councilPvApi`, `CouncilPvView.vue`, intégration registre) — cataloguer les `C*` nouveaux d'abord.
6. Tests E2E, entrée `docs/PROGRESS.md`.
