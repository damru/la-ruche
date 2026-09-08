# Spécification technique — EP-021-S02

**Story :** Mise en demeure — surface de notification in-app informative + lettre type imprimable. L'envoi réel (email, courrier recommandé) est **hors périmètre MVP**.

**Épic :** EP-021 — Conformité & communications
**Réf. LEGAL :** §4.7 · **Dépendance :** **EP-004-S11** (modèle `formal_notice`, lots débiteurs, déchéance J+30) — cette story se construit **au-dessus** de S11 et ne peut pas démarrer avant.

> Statut : NON IMPLÉMENTÉ. Aucun mailer dans le repo ; `backend/src/jobs/reminders.rs` ne fait que logger (« TODO post-MVP: trigger email notification »).

---

## 1. État actuel du code

| Élément | Emplacement |
|---|---|
| Job de rappels : log seulement, pas d'envoi | `backend/src/jobs/reminders.rs` |
| **Prérequis (EP-004-S11, à livrer avant)** | table `formal_notice`, `db/formal_notice.rs`, `domain/arrears.rs`, routes `/arrears` + `/units/{id}/formal-notices`, vue `ArrearsView.vue` |
| **Absent** | Toute notification in-app ; toute génération de document |

---

## 2. Quoi (résumé fonctionnel)

Au MVP, la mise en demeure reste **expédiée hors app** (le syndic poste le recommandé lui-même). L'app fournit :

1. **Notification in-app informative** au copropriétaire concerné : lors de la création d'une `formal_notice` (S11), le(s) propriétaire(s) du lot voient à leur prochaine connexion un bandeau/notification « Une mise en demeure vous a été adressée le {date} pour un montant de {montant} ». **Informative uniquement** — elle ne constitue pas la notification légale (c'est le recommandé) ; le libellé le dit explicitement.
2. **Lettre type imprimable** : vue print générée depuis la `formal_notice` — en-têtes syndicat/copropriétaire, rappel des appels impayés (détail), montant total, rappel de l'art. 19-2 (déchéance à J+30), mentions légales. Le syndic l'imprime et l'envoie en recommandé.
3. **Hors périmètre explicite** : envoi d'email, intégration recommandé électronique (AR24 & co), accusé de réception — post-MVP.

---

## 3. Modèle de données

Une table de notifications in-app minimale (mise à jour `docs/ARCHITECTURE.md` avant migration ; pas d'ADR).

Migration `backend/migrations/NNNN_notification.sql` (numéro à caler — renumérotation rectificatif en cours) :

```sql
-- EP-021-S02: minimal in-app notifications (MVP: informative formal-notice notice; extensible kinds)
CREATE TYPE notification_kind AS ENUM ('formal_notice');

CREATE TABLE notification (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES building(id),
    user_id UUID NOT NULL REFERENCES app_user(id),
    kind notification_kind NOT NULL,
    entity_id UUID NOT NULL,              -- formal_notice.id
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX notification_user_idx ON notification (user_id, read_at, created_at DESC);
```

Volontairement générique (`kind` extensible) mais sans sur-ingénierie : une seule valeur au MVP, pas de payload JSON, le contenu se recompose depuis l'entité référencée.

---

## 4. Backend

### 4.1 Domain

- `backend/src/domain/notice_letter.rs` (nouveau) : assemblage **pur** des données de la lettre :
  ```rust
  pub(crate) struct NoticeLetterData {
      pub syndicate: PartyBlock, pub owner: PartyBlock,
      pub notice_date: NaiveDate, pub amount: Decimal,
      pub overdue_levies: Vec<LevyLine>,        // label, due_date, remaining amount
      pub acceleration_deadline: NaiveDate,      // sent_at + 30 days (art. 19-2)
  }
  ```
  Le texte de la lettre vit côté frontend (i18n) ; le backend fournit les données structurées.

### 4.2 DB

- `backend/src/db/notification.rs` (nouveau) : `create_for_unit_owners(tx, building_id, unit_id, kind, entity_id)` (résout les propriétaires du lot et insère une ligne chacun), `list_unread(pool, user_id)`, `mark_read(pool, id, user_id)`.
- **Branchement S11** : dans `db::formal_notice::create` (livré par EP-004-S11), ajouter l'appel `notification::create_for_unit_owners` **dans la même transaction** que l'insert et l'audit.
- `backend/src/db/notice_letter.rs` ou extension : requête assemblant les données lettre (formal_notice + appels impayés du lot + identités).

### 4.3 API — routes `api/mod.rs`

| Route | Rôle | Effet |
|---|---|---|
| `GET /notifications` | tout utilisateur | Ses notifications (non lues d'abord). |
| `POST /notifications/{id}/read` | tout utilisateur (propriétaire de la notif) | Marque lue. |
| `GET /formal-notices/{id}/letter` | manager | `NoticeLetterData` JSON pour la vue imprimable. |

Handlers minces ; notif d'autrui → `404`.

---

## 5. Frontend

- Types (`Notification`, `NoticeLetterData`) + `notificationsApi` / `formalNoticesApi.letter(id)` dans `frontend/src/api/index.ts`.
- **Cloche/badge de notifications** dans le layout principal : compteur non-lues, liste déroulante, clic → marque lue + navigue. Composant `CNotificationBell` (ou équivalent) → catalogue `docs/DESIGN.md §2` avant usage.
- **Bandeau copropriétaire** : notification `formal_notice` non lue → bandeau d'avertissement sur le dashboard avec le texte informatif et la mention « Cette notification est informative — la mise en demeure officielle vous est adressée par courrier recommandé. » (clé i18n dédiée).
- **Lettre imprimable** : depuis `ArrearsView.vue`/l'historique des mises en demeure (S11), action « Imprimer la lettre » → vue `NoticeLetterView.vue` mise en page via utilitaires Tailwind `print:` (pas de `<style>` block, pas de PDF) → `window.print()`. Corps de lettre en i18n français.

---

## 6. Cas limites

| Cas | Règle |
|---|---|
| Lot multi-propriétaires (indivision) | Une notification par propriétaire ; la lettre liste tous les copropriétaires du lot. |
| Propriétaire sans compte activé | Aucune notification créée pour lui (pas de `user_id`) — la lettre imprimée reste le canal ; pas d'erreur. |
| Mise en demeure liée à un contentieux | La lettre reste imprimable (l'historique sert de pièce). |
| Notification déjà lue | `POST /read` idempotent (200). |
| Dette soldée après notification | La notification reste (fait historique) ; le bandeau peut afficher l'état courant de la dette s'il est trivial à joindre, sinon texte statique daté. |

---

## 7. Critères d'acceptation

1. La création d'une mise en demeure (S11) crée, dans la même transaction, une notification pour chaque propriétaire à compte actif du lot.
2. Le copropriétaire voit le bandeau informatif à sa prochaine connexion et peut le marquer lu.
3. Le manager imprime une lettre type complète (parties, détail des impayés, montant, échéance J+30, mentions art. 19-2) via le print navigateur.
4. Aucun email/courrier n'est envoyé par l'app ; le caractère informatif est explicite dans l'UI.
5. Clippy/tests/build/lint verts ; `docs/ARCHITECTURE.md` à jour.

---

## 8. Plan de tests

- **Unitaires domaine** : assemblage `NoticeLetterData` (échéance J+30 correcte, total = somme des lignes, multi-propriétaires).
- **E2E** : création mise en demeure → notifications créées pour les propriétaires actifs (et pas pour les non-activés) ; `GET /notifications` côté propriétaire → présente ; `POST /read` → `read_at` posé, idempotent ; notif d'autrui → 404 ; `GET /letter` manager → données complètes ; owner → 403.

---

## 9. Ordre d'implémentation

1. **Attendre la livraison d'EP-004-S11** (`formal_notice`).
2. `docs/ARCHITECTURE.md` (table `notification`).
3. Migration `NNNN` → `sqlx migrate run` + `sqlx prepare`.
4. DB notifications + branchement dans `db::formal_notice::create` (même tx).
5. Domain `notice_letter.rs` + DB lettre.
6. API (3 routes) + i18n (dont corps de lettre).
7. Tests e2e.
8. Frontend (cloche, bandeau, `NoticeLetterView.vue`, catalogue DESIGN.md).
9. `graphify update .` + `docs/PROGRESS.md`.
