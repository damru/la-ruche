# Spécification technique — EP-017-S06

**Story :** Visibilité : les événements sont visibles par tous les rôles ayant accès à l'entité ; les actions sensibles (`assignee_changed`, `vendor_link_generated`) sont réservées aux rôles manager / conseil syndical.

**Épic :** EP-017 — Journal d'activité & Suivi de cycle de vie
**Réf. PRD :** Gestion courante (US-JA-*) · **Réf. EPICS :** `docs/EPICS.md` EP-017-S06 (L514)
**Stories liées :** EP-017-S02 (vocabulaire des actions), EP-017-S03 (handlers timeline — points de branchement du filtre), EP-017-S05 (seule vue exposée aux résidents)

---

## 1. Quoi

Un filtre **côté backend** appliqué aux deux endpoints timeline : la liste renvoyée est expurgée des actions sensibles quand le lecteur n'est ni `manager` ni `board_member`. Le frontend n'a rien à masquer (défense en profondeur inutile : la donnée ne quitte pas le serveur).

**Périmètre effectif :** `GET /tasks/{id}/timeline` est déjà réservé à manager/board_member (routage frontend `tasks/:id` et usage actuel — EP-017-S03 §4.2) : le filtre y est un no-op aujourd'hui, mais il est implémenté sur **les deux** endpoints pour rester correct si l'accès aux tâches s'élargit. Le cas réellement exercé est `GET /reports/{id}/timeline`, accessible aux résidents (`co_owner`, `tenant`), dont la réponse inclut les événements de la tâche liée (S03 §1) — donc potentiellement `assignee_changed` et `vendor_link_generated`.

## 2. État actuel du code

| Élément | Emplacement |
|---|---|
| Enum `Role` (`Manager`, `BoardMember`, `CoOwner`, `Tenant`) | `backend/src/domain/mod.rs` L17-22 |
| Extracteur d'auth + `require_any_role` | usage type : `backend/src/api/reports.rs` L68 |
| Handlers timeline (créés par S03, `auth` déjà en scope) | `backend/src/api/tasks.rs::timeline`, `backend/src/api/reports.rs::timeline` |
| Constantes d'actions (dont `ASSIGNEE_CHANGED`, `VENDOR_LINK_GENERATED`) | `backend/src/db/activity_log.rs::actions` (EP-017-S02 §4.1) |
| Précédent de logique métier pure en domaine | `backend/src/domain/immutability.rs` (fonctions pures + tests unitaires inline) |

## 3. Modèle de données

Aucun.

## 4. Backend Rust

### 4.1 Nouveau module `backend/src/domain/activity_visibility.rs`

Logique pure en `domain/` (zéro SQL, zéro logique dans les handlers — règles AGENTS.md), sur le modèle de `domain/immutability.rs` :

```rust
use crate::db::activity_log::actions;
use crate::domain::Role;

/// EP-017-S06: sensitive lifecycle actions are restricted to
/// manager / board_member ("conseil syndical"). Everything else is
/// visible to any role that can read the underlying entity.
const SENSITIVE_ACTIONS: &[&str] = &[actions::ASSIGNEE_CHANGED, actions::VENDOR_LINK_GENERATED];

pub(crate) fn can_view_action(role: &Role, action: &str) -> bool {
    if SENSITIVE_ACTIONS.contains(&action) {
        matches!(role, Role::Manager | Role::BoardMember)
    } else {
        true
    }
}
```

Décisions :
- **Liste sensible fermée et centralisée** (constante unique) — toute évolution du vocabulaire (S02) qui introduit une action sensible se fait ici, en un point.
- Filtrage **en mémoire** après `list_for_chain`, pas en SQL : la liste est courte (timeline d'une entité), et garder la règle en `domain/` la rend testable unitairement et indépendante du repo. Pas de paramètre `role` dans `db/activity_log.rs`.
- Un rôle inconnu/futur est traité comme non privilégié (le `matches!` est une allow-list).

### 4.2 Branchement dans les handlers timeline (S03)

Dans `tasks::timeline` et `reports::timeline`, après la lecture :

```rust
let role = auth.role()?; // accès au Role de l'AuthUser, pattern existant
let events: Vec<_> = events
    .into_iter()
    .filter(|e| domain::activity_visibility::can_view_action(&role, &e.action))
    .collect();
```

Aucune autre logique dans les handlers. Les événements filtrés sont **omis** (pas de placeholder « événement masqué ») : la frise résident reste cohérente, les transitions de statut suffisent à la lisibilité.

### 4.3 Rappel du niveau « accès à l'entité »

La première moitié de la règle EPICS (« visibles par tous les rôles ayant accès à l'entité ») est déjà garantie par S03 : chaque handler charge l'entité par `id + building_id` (404 sinon) et hérite des guards de rôle de l'endpoint de lecture correspondant. S06 n'ajoute que la granularité **par action**.

## 5. Frontend

Aucun changement : S04 (vue réservée manager/board_member — jamais filtrée en pratique) et S05 (la frise résident affiche ce que l'API renvoie, cf. EP-017-S05 §6). Aucune clé i18n « masqué » nécessaire (§4.2).

## 6. Cas limites

| Cas | Règle |
|---|---|
| Résident lisant la timeline d'un signalement converti | Les événements tâche non sensibles (statuts, `closed`, `document_uploaded`, `vendor_submission_received`, `contractor_assigned`…) restent visibles ; `assignee_changed` et `vendor_link_generated` sont omis. |
| `contractor_assigned` | **Non sensible** (liste EPICS exhaustive : seuls `assignee_changed` et `vendor_link_generated`) — le nom du prestataire est une information de suivi légitime pour le résident. |
| Timeline devenant vide après filtrage | `200 []` — même rendu « état vide » côté S04/S05. |
| Manager / board_member | Aucun filtrage : timeline intégrale. |
| Nouvelle action ajoutée sans classement | Visible par défaut (non listée sensible) — à réévaluer à chaque extension du vocabulaire S02. |

## 7. Critères d'acceptation

1. `can_view_action` : allow-list stricte — sensibles visibles uniquement par `Manager`/`BoardMember`, tout le reste visible par tous.
2. `GET /reports/{id}/timeline` en tant que `co_owner`/`tenant` ne contient jamais `assignee_changed` ni `vendor_link_generated` ; en tant que `manager`/`board_member`, il les contient.
3. Les deux handlers timeline appliquent le filtre (y compris `tasks`, même si non exercé aujourd'hui).
4. Aucun changement de schéma, de route ni de frontend ; clippy/fmt au vert.

## 8. Plan de tests

### 8.1 Unitaires — `domain/activity_visibility.rs` (`#[cfg(test)]` inline, pattern `immutability.rs`)

```rust
#[test]
fn sensitive_actions_restricted_to_managers() {
    for role in [Role::CoOwner, Role::Tenant] {
        assert!(!can_view_action(&role, actions::ASSIGNEE_CHANGED));
        assert!(!can_view_action(&role, actions::VENDOR_LINK_GENERATED));
        assert!(can_view_action(&role, actions::STATUS_CHANGED));
    }
    for role in [Role::Manager, Role::BoardMember] {
        assert!(can_view_action(&role, actions::ASSIGNEE_CHANGED));
        assert!(can_view_action(&role, actions::VENDOR_LINK_GENERATED));
    }
}
```

### 8.2 e2e — `backend/tests/api_e2e_test.rs`

1. Fixture : signalement (résident) → conversion (manager) → changement d'assignataire + génération de lien vendeur + changement de statut.
2. `GET /reports/{id}/timeline` en résident → séquence sans les 2 actions sensibles, statuts présents.
3. Même appel en manager puis en board_member → séquence intégrale.
4. Comptage : `len(manager) == len(resident) + 2`.

## 9. Ordre d'implémentation

1. `domain/activity_visibility.rs` + tests unitaires §8.1 → `cargo test --lib`.
2. Branchement du filtre dans les deux handlers timeline → `cargo clippy -- -D warnings`.
3. Tests e2e §8.2 → vert ; `cargo fmt --check`.
4. `docs/EPICS.md` (statut S06 — clôt l'épic EP-017), `docs/PROGRESS.md`, `graphify update .`.
