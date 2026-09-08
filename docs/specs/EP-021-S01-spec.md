# Spécification technique — EP-021-S01

**Story :** Export RGPD des données personnelles (droit d'accès, art. 15 RGPD) : bundle JSON horodaté (profil, lots, traçabilité des charges) + trace de la demande.

**Épic :** EP-021 — Conformité & communications
**Réf. LEGAL :** §8 · **Réf. PRD :** US conformité RGPD

> Statut : NON IMPLÉMENTÉ. Seule l'anonymisation existe (droit à l'effacement) — l'export (droit d'accès) est absent.

---

## 1. État actuel du code

| Élément | Emplacement |
|---|---|
| Anonymisation (droit à l'effacement) | `POST /users/{id}/anonymize` — route `backend/src/api/mod.rs:109`, handler `backend/src/api/users.rs:283-310`, `backend/src/db/user.rs:621`, logique `backend/src/domain/owners.rs` |
| **Absent** | Tout endpoint d'export ; toute trace de demande d'accès |
| Audit infra | `backend/src/db/audit.rs`, migration `0029_audit_log.sql` |

---

## 2. Quoi (résumé fonctionnel)

1. **Bundle JSON horodaté** contenant, pour un utilisateur donné :
   - profil (`app_user` : identité, email, téléphone, rôle, dates de création/connexion — jamais le hash de mot de passe) ;
   - lots détenus (quotes-parts, historique de propriété si présent) ;
   - traçabilité des charges : appels de fonds adressés à ses lots (libellé, échéance, montants, statut, paiements) ;
   - invitations le concernant.
2. **Trace de la demande** : chaque export enregistre une ligne `gdpr_export_request` (qui a demandé, pour qui, quand) — obligation de traçabilité LEGAL §8. Doublée d'une ligne `audit_log`.
3. **Deux déclencheurs** :
   - **self-service** : `GET /me/gdpr-export` — l'utilisateur exporte ses propres données ;
   - **manager** : `GET /users/{id}/gdpr-export` — le syndic répond à une demande d'accès reçue hors app.

---

## 3. Modèle de données

Mettre à jour `docs/ARCHITECTURE.md` (table `gdpr_export_request`) avant migration. Pas d'ADR (table de trace standard).

Migration `backend/migrations/NNNN_gdpr_export_request.sql` (numéro à caler — renumérotation rectificatif en cours) :

```sql
-- EP-021-S01: GDPR access-request trace (art. 15 RGPD, LEGAL §8) — one row per export delivered
CREATE TABLE gdpr_export_request (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES building(id),
    subject_user_id UUID NOT NULL REFERENCES app_user(id),
    requested_by UUID NOT NULL REFERENCES app_user(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX gdpr_export_request_subject_idx ON gdpr_export_request (subject_user_id, created_at DESC);
```

Append-only (aucun UPDATE/DELETE exposé).

---

## 4. Backend

### 4.1 Domain — `backend/src/domain/gdpr.rs` (nouveau)

- Structure de sortie typée :
  ```rust
  pub(crate) struct GdprExport {
      pub generated_at: DateTime<Utc>,
      pub subject: ProfileSection,        // no password hash, no internal ids beyond the user's own
      pub units: Vec<UnitSection>,        // lot label, tantièmes, ownership dates
      pub levies: Vec<LevySection>,       // call_label, due_date, amounts (Decimal), status, payments
      pub invitations: Vec<InvitationSection>,
  }
  ```
- Règle d'autorisation pure : `can_export(requester, subject) -> Result<(), AppError>` — soi-même, ou manager du même immeuble ; sinon `AppError::Forbidden`.
- Filtrage strict des champs : liste blanche explicite (jamais de sérialisation directe des rows DB) — c'est la garantie qu'aucun champ sensible (hash, tokens) ne fuit.

### 4.2 DB

- `backend/src/db/gdpr.rs` (nouveau) :
  - `collect_export_data(pool, subject_user_id, building_id)` — requêtes de lecture réutilisant les repos existants (`db::user`, `db::unit`, appels via les requêtes de `db/budget.rs`, invitations) ;
  - `record_request(tx, building_id, subject_user_id, requested_by)` — insert + `db::audit::insert` (entity=`gdpr_export`, operation=`CREATE`) dans la même transaction ;
  - `list_requests(pool, building_id)` pour la consultation manager.

### 4.3 API — `backend/src/api/gdpr.rs` (nouveau) + routes `api/mod.rs`

| Route | Rôle | Effet |
|---|---|---|
| `GET /me/gdpr-export` | tout utilisateur authentifié | Son propre bundle. |
| `GET /users/{id}/gdpr-export` | manager | Bundle d'un utilisateur de l'immeuble. |
| `GET /gdpr-export-requests` | manager | Historique des demandes tracées. |

Réponse : JSON avec `Content-Disposition: attachment; filename="export-rgpd-{user}-{date}.json"`. Chaque export réussi écrit la trace **avant** de renvoyer le corps (même transaction que l'audit ; l'export est en lecture, seule la trace écrit). Handlers minces ; utilisateur inexistant / hors immeuble → `404`.

---

## 5. Frontend

- `frontend/src/api/index.ts` : `gdprApi.exportMe()`, `exportUser(id)`, `listRequests()` — téléchargement blob JSON.
- **Self-service** : dans la vue profil/paramètres utilisateur existante, section « Mes données (RGPD) » avec bouton « Télécharger mes données » + texte explicatif (droit d'accès).
- **Manager** : sur la fiche utilisateur/copropriétaire (là où vit déjà l'action anonymisation, cf. `api/users.rs:283-310` côté back), action « Exporter les données (RGPD) » ; page ou section listant l'historique des demandes (`GET /gdpr-export-requests`).
- Clés i18n : `gdpr.export.download`, `.description`, `.requestedAt`, `.requestedBy`, etc.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Utilisateur anonymisé | Export encore possible (le profil renvoie les champs anonymisés tels quels) — cohérent : l'effacement a déjà eu lieu. |
| Utilisateur sans lot | Sections `units`/`levies` vides, export valide. |
| Owner demandant l'export d'un autre utilisateur | `403` (`can_export`). |
| Board member | Pas d'accès à l'export d'autrui (données personnelles ≠ lecture comptable) — manager uniquement. |
| Exports répétés | Autorisés ; chaque export laisse sa trace (pas de rate-limit au MVP). |
| Montants | `Decimal` sérialisés en chaînes (`"123.45"`) pour éviter toute perte flottante côté JSON. |

---

## 7. Critères d'acceptation

1. `GET /me/gdpr-export` renvoie un JSON horodaté complet (profil, lots, appels, invitations) sans aucun champ technique sensible.
2. Le manager peut exporter pour un utilisateur de son immeuble ; un owner ne peut exporter que lui-même.
3. Chaque export crée une ligne `gdpr_export_request` **et** une ligne `audit_log` (même transaction).
4. L'historique des demandes est consultable par le manager.
5. Clippy/tests/build/lint verts ; `docs/ARCHITECTURE.md` à jour.

---

## 8. Plan de tests

- **Unitaires domaine** : `can_export` (soi-même OK, manager OK, owner→autre 403, board→autre 403) ; liste blanche des champs (le hash n'apparaît jamais dans la struct sérialisée — test de sérialisation).
- **E2E** : self-service → 200 + trace + audit ; manager → 200 ; owner sur autrui → 403 ; utilisateur hors immeuble → 404 ; contenu spot-check (un lot, un appel avec montant exact en chaîne) ; `GET /gdpr-export-requests` → la demande y figure.

---

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` (table `gdpr_export_request`).
2. Migration `NNNN` → `sqlx migrate run` + `sqlx prepare`.
3. Domain `gdpr.rs` (structs liste-blanche + `can_export`) + tests unitaires.
4. DB (`collect_export_data`, `record_request` + audit).
5. API (3 routes) + i18n.
6. Tests e2e.
7. Frontend (section profil + action manager + historique).
8. `graphify update .` + `docs/PROGRESS.md`.
