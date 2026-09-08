# Spécification technique — EP-013-S09

**Story :** Signer et publier le PV — lier la signature déclarative à un **document précis** (intégrité SHA-256) et tracer la notification **par destinataire**.

**Épic :** EP-013 — Procès-verbaux
**Réf. LEGAL :** §11.1 (valeur probante) ; notification du PV aux opposants/défaillants (art. 42 loi 1965) · **Réf. PRD :** US-PV-* · **Réf. ADR :** aucun (colonnes/table additives)

**Statut actuel : PARTIEL.**

> **Hors périmètre explicite :** la signature électronique **cryptographique** (eIDAS, certificats, horodatage qualifié) n'est PAS couverte. La signature reste déclarative (noms + horodatage) ; cette story la rend **opposable à un document précis** via son empreinte.

---

## 1. Ce qui existe déjà (ne pas retoucher)

| Élément | Emplacement |
|---|---|
| Signature déclarative (`sign_minutes` : noms + `NOW()` en JSONB) | `backend/src/api/assemblies.rs` L590-600 → `backend/src/db/assembly.rs` L451-500 |
| Notification globale (`notify_minutes` : unique `minutes_notified_at`) | `backend/src/api/assemblies.rs` L602-612, `backend/src/db/assembly.rs` L510-533 |

## 2. Écarts restants

- **(a)** La signature ne référence **aucun document** : si le fichier PV est remplacé après signature, rien ne l'invalide.
- **(b)** La notification est un timestamp unique sur le meeting : aucune trace **par destinataire** (qui, quand, par quel canal).

## 3. Modèle de données

Nouvelle table → **mettre à jour `docs/ARCHITECTURE.md` avant la migration**.

Migration `backend/migrations/NNNN_minutes_signature_integrity.sql` :

```sql
-- EP-013-S09 (a): bind the declarative signature to one specific minutes
-- document. sha256 is captured at signature time; replacing the document
-- invalidates the signature (re-signature required).
ALTER TABLE general_meeting
    ADD COLUMN minutes_document_id UUID REFERENCES document(id),
    ADD COLUMN minutes_document_sha256 TEXT;   -- hex digest captured at signature time

-- EP-013-S09 (b): per-recipient notification trace (art. 42 loi 1965).
CREATE TABLE minutes_notification (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL REFERENCES general_meeting(id) ON DELETE CASCADE,
    recipient_id UUID NOT NULL REFERENCES app_user(id),
    channel TEXT NOT NULL CHECK (channel IN ('manual', 'email')),  -- 'email' = placeholder, no sender wired yet
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (meeting_id, recipient_id)
);
```

> Adapter les noms exacts (`general_meeting`, `document`, `app_user`) à `0001_init.sql`. `minutes_notified_at` existant est conservé (compat) et posé à la première notification.

Après migration : `cargo sqlx migrate run` + `cargo sqlx prepare`.

## 4. Backend

### 4.1 Domaine

- `backend/src/domain/minutes.rs` (nouveau, petit) : calcul SHA-256 hex d'un contenu (`sha2` déjà en dépendance ou à ajouter), et règle :

```rust
/// A signature is valid only for the exact document bytes it was made on.
/// If the stored sha256 no longer matches the current document, the
/// signature is void and signing must be redone.
pub(crate) fn signature_matches(stored_sha256: &str, current_sha256: &str) -> bool { ... }
```

### 4.2 DB — `backend/src/db/assembly.rs`

- `sign_minutes` (L451-500) — payload étendu avec `document_id: Uuid` :
  - vérifier que le document appartient au meeting/immeuble et est bien le PV (404 sinon) ;
  - calculer le SHA-256 du contenu au moment de la signature ;
  - stocker `minutes_document_id` + `minutes_document_sha256` avec le JSONB de signature existant.
- Remplacement du document PV (chemin d'upload existant) : si `minutes_document_id` pointe le document remplacé, **effacer la signature** (JSONB + colonnes intégrité) — re-signature requise. Alternative plus simple si le remplacement in-place n'existe pas (nouveau document = nouvel id) : à la lecture/notification, recalculer et comparer le hash ; mismatch → signature considérée nulle (`errors.minutes.signature_stale`).
- `notify_minutes` (L510-533) → devient per-recipient :
  - `record_notifications(pool, meeting_id, building_id, recipients: Vec<(Uuid, String)>)` — insère les lignes `minutes_notification` (upsert ignore sur doublon), pose `minutes_notified_at` si NULL ;
  - précondition : PV **signé et intègre** (`errors.minutes.not_signed`, `errors.minutes.signature_stale`) ;
  - `list_notifications(pool, meeting_id, building_id)`.

### 4.3 API

- `sign_minutes` (L590-600) : accepte `{ signatory_names, document_id }` ; erreurs `errors.minutes.document_not_found`, `errors.minutes.already_signed` (si déjà signé et intègre — re-signature autorisée seulement après invalidation).
- `notify_minutes` (L602-612) : accepte `{ recipients: [{ recipient_id, channel }] }` (channel `manual` aujourd'hui, `email` = placeholder accepté mais aucun envoi réel — le tracer seulement).
- Nouvelle route GET :

```rust
.route("/meetings/{id}/minutes/notifications", get(assemblies::list_minutes_notifications))
```

- Rôles : manager pour sign/notify ; manager + board_member pour la lecture des notifications.

## 5. Frontend

- `frontend/src/api/index.ts` : `signMinutes(meetingId, { signatoryNames, documentId })`, `notifyMinutes(meetingId, recipients)`, `listMinutesNotifications(meetingId)`.
- Vue PV (`frontend/src/features/assemblies/`) :
  - la signature exige la sélection du document PV (liste des documents du meeting) ;
  - badge « Signature invalidée — document modifié » si le backend signale `signature_stale` ;
  - notification : sélection multi-destinataires (copropriétaires) + canal, tableau des notifications déjà tracées (destinataire, canal, date).
- Nouveaux composants éventuels → cataloguer dans `docs/DESIGN.md §3` avant usage.

## 6. Cas limites

| Cas | Règle |
|---|---|
| Signer sans `document_id` ou document étranger au meeting | 400/404. |
| Document PV remplacé après signature | Signature nulle → notification bloquée (`errors.minutes.signature_stale`), re-signature requise. |
| Notifier sans signature valide | 400 `errors.minutes.not_signed`. |
| Notifier deux fois le même destinataire | Ligne unique conservée (UNIQUE), 200 idempotent. |
| Canal `email` | Trace enregistrée, aucun envoi (placeholder assumé, affiché comme tel). |
| `minutes_notified_at` legacy | Continue d'être posé à la première notification (compat lecture existante). |

## 7. Critères d'acceptation

1. La signature enregistre `document_id` + SHA-256 du contenu au moment de l'acte.
2. Tout remplacement/altération du document rend la signature invalide et impose une re-signature.
3. Chaque destinataire notifié a une ligne (qui, quand, canal) ; relecture via GET.
4. La notification est impossible sans PV signé et intègre.
5. La signature cryptographique reste hors périmètre (documenté).

## 8. Plan de tests

- **Unitaires domaine** : `signature_matches` (hash identique / différent), calcul SHA-256 stable.
- **E2E** :
  1. sign avec document valide → 200, hash stocké ;
  2. sign avec document d'un autre meeting → 404 ;
  3. notify avant signature → 400 ;
  4. notify 3 destinataires → 3 lignes, `minutes_notified_at` posé ; re-notify même destinataire → toujours 3 lignes ;
  5. remplacer le document → notify → 400 `signature_stale` ; re-sign → notify → 200.

## 9. Ordre d'implémentation

1. `docs/ARCHITECTURE.md` : table `minutes_notification` + colonnes intégrité.
2. Migration `NNNN` → `cargo sqlx migrate run` + `cargo sqlx prepare`.
3. Domaine : hash + `signature_matches` + tests unitaires.
4. DB : `sign_minutes` étendu, invalidation, `record_notifications`/`list_notifications`.
5. API : handlers + route GET → `cargo clippy -- -D warnings`.
6. Tests e2e, boucler jusqu'au vert.
7. Frontend : sélection document à la signature, notifications par destinataire.
8. `docs/PROGRESS.md` + `graphify update .`.
