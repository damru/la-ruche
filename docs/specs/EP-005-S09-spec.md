# Spécification technique — EP-005-S09

**Story :** Nettoyage périodique des objets S3 orphelins + purge différée selon les règles de rétention par catégorie.

**Épic :** EP-005 — Documents
**Réf. PRD :** US-DOC-008 · **Réf. LEGAL :** §6.1-6.3 (durées de conservation)
**Statut : PARTIEL** — le nettoyage synchrone en transaction existe déjà (`delete_orphaned_tx`) ; il manque tout le volet périodique/S3/rétention.
**Stories liées :** **EP-005-S07 (prérequis — `domain/retention.rs`)**, **EP-005-S08 (prérequis — soft-delete ; sans lui, rien à purger de façon différée)**

---

## 1. Ce qui existe déjà (ne pas re-spécifier)

- Nettoyage **synchrone** des lignes `document` orphelines lors de la suppression d'un parent : `delete_orphaned_tx` (`backend/src/db/document.rs` L208-253 ; devient soft-delete en S08), appelé depuis `db/task.rs:381`, `db/claim.rs:475`, `db/report.rs:320`, `db/contract.rs:612`.
- Infrastructure de jobs : `backend/src/jobs/` (uniquement `mod.rs` + `reminders.rs` — boucle `tokio::time::interval`, logging `tracing`, pattern à répliquer).

## 2. Ce qui manque (périmètre de cette spec)

1. **Objets S3 sans ligne DB** : un upload présigné confirmé puis jamais enregistré (ou un échec entre S3 et DB) laisse un objet introuvable — aucune ligne `document` ne pointe dessus, rien ne peut le supprimer aujourd'hui.
2. **Purge différée des documents soft-deleted** (S08) : les lignes `deleted_at IS NOT NULL` et leurs objets S3 doivent être physiquement purgés **uniquement** quand la rétention de leur catégorie (S07) est écoulée.
3. Aucun job planifié n'existe pour l'un ou l'autre.

---

## 3. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Pattern de job (boucle horaire, `Arc<AppState>`) | `backend/src/jobs/reminders.rs` L1-25 |
| Déclaration des jobs | `backend/src/jobs/mod.rs` (une ligne `pub mod reminders;`) — le spawn est fait au démarrage (suivre le point d'appel de `jobs::reminders::run`) |
| Client storage (delete_object, bucket par immeuble) | `backend/src/api/documents.rs` L206-224 (`state.storage`) — vérifier que le trait storage expose un `list_objects` ; sinon l'ajouter |
| `document.storage_path` / `bucket` | `0001_init.sql` L286-301 |
| Rétention par catégorie | `backend/src/domain/retention.rs` (S07) |
| Soft-delete (`deleted_at`) | migration S08 |

---

## 4. Modèle de données

Aucune nouvelle table. Ajouter un garde-fou temporel si absent : l'horodatage S3 (`LastModified`) sert de référence pour le délai de grâce des orphelins — pas de colonne nécessaire.

---

## 5. Backend Rust — nouveau job `backend/src/jobs/s3_cleanup.rs`

### 5.1 Boucle

Même squelette que `reminders.rs` : `pub async fn run(state: Arc<AppState>)`, `tokio::time::interval` **quotidien** (`Duration::from_secs(86_400)`), erreurs loguées sans tuer la boucle. Déclarer `pub mod s3_cleanup;` dans `jobs/mod.rs` et spawner au même endroit que `reminders::run`.

### 5.2 Passe 1 — objets S3 orphelins

```text
pour chaque building (SELECT id FROM building):
    bucket = storage.bucket_name(building_id)
    keys_s3 = storage.list_objects(bucket)           # à ajouter au client storage si absent
    keys_db = SELECT storage_path FROM document WHERE building_id = $1
              # TOUTES les lignes, y compris deleted_at IS NOT NULL et archived_at IS NOT NULL
    pour chaque key dans keys_s3 \ keys_db:
        si last_modified(key) < NOW() - GRACE_PERIOD:   # 48h — laisse passer les uploads en cours
            storage.delete_object(bucket, key)
            tracing::info!(...)
```

- `GRACE_PERIOD` : constante `const ORPHAN_GRACE: Duration = 48h` — protège la fenêtre entre URL présignée et `confirm_upload`.
- Le diff se fait **en mémoire par bucket** (volumes copropriété : centaines d'objets, pas de pagination complexe nécessaire ; utiliser la pagination du SDK S3 quand même pour la correction).
- Ne jamais supprimer une clé présente en DB, même soft-deleted — c'est la passe 2 qui décide.

### 5.3 Passe 2 — purge différée par rétention (dépend S07 + S08)

```text
candidats = SELECT id, bucket, storage_path, category::text
            FROM document
            WHERE deleted_at IS NOT NULL
              AND deleted_at < NOW() - INTERVAL '1 month' * retention  # calculé en Rust
pour chaque candidat:
    match domain::retention::retention_months(category):
        None      -> skip   # pv_ag, fondateurs : jamais purgés
        Some(m)   -> si deleted_at + m mois < NOW():
                        DELETE FROM document WHERE id = ...   # purge physique finale
                        storage.delete_object(bucket, storage_path)
```

- En pratique : sélectionner tous les `deleted_at IS NOT NULL`, filtrer en Rust via `retention_months` (la règle vit dans `domain/`, pas en SQL — convention repo « zéro SQL dans domain/ » respectée dans l'autre sens : la règle métier ne descend pas en SQL).
- Ordre : **DELETE DB d'abord, S3 ensuite** ; si le delete S3 échoue, l'objet devient un orphelin rattrapé par la passe 1 au tour suivant (idempotence).
- Le hard DELETE final ne passe pas par `delete_orphaned_tx` : à ce stade la ligne est soft-deleted depuis des années, plus aucun lien n'existe (les liens sont retirés au moment du soft-delete S08) — vérifier tout de même l'absence de liens par sécurité avant le DELETE.

### 5.4 Configuration

Réutiliser `state.cfg` : ajouter (si utile) `s3.cleanup_enabled: bool` (défaut `true`) pour désactiver en environnement de test/e2e. Pas d'autre knob.

---

## 6. Frontend

Aucun. Job purement serveur.

---

## 7. Cas limites

| Cas | Règle |
|---|---|
| Upload présigné en cours (clé S3, pas encore de ligne DB) | Protégé par `ORPHAN_GRACE` 48h sur `LastModified`. |
| Objet S3 référencé par un document soft-deleted en rétention | Jamais touché par la passe 1 (présent en DB) ni par la passe 2 (rétention non écoulée). |
| Catégorie à rétention illimitée (`pv_ag`, fondateurs) soft-deleted | Ligne et objet conservés indéfiniment. |
| Échec S3 sur un objet | Logué, boucle continue ; rattrapage au tour suivant. |
| Bucket absent (immeuble sans upload) | `list_objects` sur bucket inexistant → traiter comme vide, pas d'erreur fatale. |
| Deux instances du backend | Hors périmètre MVP (une instance) ; noter dans le code que le job n'est pas verrouillé multi-instance. |
| S07/S08 non encore livrés | **Dépendance dure** : la passe 2 ne peut pas exister sans `retention_months` ni `deleted_at`. La passe 1 (orphelins S3) est livrable indépendamment — l'implémenter d'abord si S08 n'est pas mergé. |

---

## 8. Critères d'acceptation

1. Un objet S3 sans ligne `document`, plus vieux que 48h, est supprimé au prochain passage du job.
2. Un objet uploadé il y a moins de 48h n'est jamais supprimé.
3. Un document soft-deleted est purgé (DB + S3) uniquement après expiration de la rétention de sa catégorie.
4. Un PV d'AG soft-deleted n'est **jamais** purgé.
5. Le job tourne quotidiennement, logue chaque suppression (`tracing::info!` avec bucket/key), et survit aux erreurs S3.

---

## 9. Plan de tests

- **Unitaires** : fonction pure `is_purgeable(category, deleted_at, now) -> bool` extraite dans `domain/retention.rs` — cas rétention finie écoulée/non écoulée, rétention `None`.
- **Intégration** (harness e2e avec le stockage S3 du docker-compose) :
  1. Poser un objet S3 sans ligne DB, `LastModified` forcé vieux (ou grace=0 en config test) → passe 1 le supprime.
  2. Objet récent → conservé.
  3. Document soft-deleted avec `deleted_at` antidaté au-delà de la rétention → passe 2 purge DB + S3.
  4. Document `pv_ag` soft-deleted antidaté → conservé.
- Extraire la logique d'un tick dans une fn `run_once(state) -> anyhow::Result<CleanupReport>` testable sans boucle (compteurs supprimés/ignorés).

---

## 10. Ordre d'implémentation

1. Vérifier/ajouter `list_objects` au client storage → `cargo build`.
2. `domain/retention.rs` : `is_purgeable` + tests unitaires.
3. `jobs/s3_cleanup.rs` : `run_once` (passe 1 puis passe 2), boucle `run`, déclaration `jobs/mod.rs` + spawn au démarrage.
4. Tests d'intégration stockage S3 → vert.
5. `docs/PROGRESS.md` (noter la dépendance S07/S08 si livraison partielle passe 1 seule).
