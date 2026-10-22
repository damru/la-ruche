# Spécification technique — EP-001-S11

**Story :** Intégrer l'EDD comme onglet de la page « Clés de répartition », avec attache et téléchargement du document EDD réel (LEGAL.md §1.2)

**Épic :** EP-001 — Configuration copropriété & clés de répartition
**Réf. LEGAL :** §1.2 · **Dépendances :**
- EP-001-S09 (formulaire de saisie de référence EDD + rapport de cohérence, mergé sur `main` — route `residence/edd` séparée) — cette story fusionne son UI dans un onglet au lieu d'une route séparée.
- EP-005-S06 (typage documents fondateurs `Règlement`/`EDD` avec version courante + historique + workflow de remplacement) — réutilisé tel quel pour l'attache du fichier EDD réel.

**Statut initial :** NON IMPLÉMENTÉ.

---

## 1. Quoi (résumé fonctionnel)

Suite à EP-001-S09, l'onglet EDD doit :
1. Vivre comme un **onglet** de la page `AllocationKeysView.vue` (« Clés de répartition » / « EDD »), pas une route séparée.
2. Permettre d'attacher le **document EDD réel** (le PDF notarié) à une référence EDD, via un **upload inline** dans le formulaire de saisie de référence (même pattern que `SaveTaskModal.vue` : sélection fichier → upload S3 → `confirmUpload` → attache).
3. Permettre le **téléchargement** du document EDD courant, et consulter un **historique** des documents EDD précédents (réutilise le mécanisme `is_current`/`supersedes_id` déjà en place pour la catégorie `edd`, EP-005-S06/S08).

Décision produit actée avec l'utilisateur :
- Catégorie de document : jamais saisie manuellement sur la page Documents générique (reste `autre` par défaut). Chaque page métier (ici EDD) pose sa catégorie implicitement lors de l'upload inline.
- Un document EDD = document fondateur unique courant (comme `Règlement`) : un nouvel upload remplace le courant (soft-delete + `supersedes_id`, workflow déjà livré par EP-005-S06/S08), pas une liste libre de pièces jointes.

## 2. État actuel du code (points d'ancrage)

| Élément | Emplacement |
|---|---|
| Page Clés de répartition | `frontend/src/features/residence/AllocationKeysView.vue` (main) |
| Vue EDD (formulaire + rapport, route séparée `residence/edd`) | `frontend/src/features/residence/EddView.vue` (main) |
| Composant onglets disponible | `frontend/src/components/atoms/CTabs.vue` (déjà utilisé : `BudgetView.vue`, `ContractsView.vue`, `ClaimsView.vue`…) |
| Upload inline + attache document (pattern à répliquer) | `frontend/src/features/tasks/SaveTaskModal.vue` (`uploadDocument`, `documentsApi.getUploadUrl/uploadToS3/confirmUpload`, `tasksApi.attachDocument`) |
| Modale liste documents liés (réutilisable) | `frontend/src/components/organisms/CLinkedDocumentsModal.vue` |
| Enum catégories document | `backend/migrations/0042_document_category_extend.up.sql` (`edd` inclus) |
| Typage document fondateur + historique + remplacement | `backend/src/db/document.rs` (`create`, `replace`/`supersedes_id`, `is_current`), spec `EP-005-S06-spec.md`, `EP-005-S08-spec.md` |
| `document_id` nullable sur `edd_reference` | `backend/migrations/NNNN_edd_reference.sql` (EP-001-S09) — prévu pour être raccordé ici |

## 3. Backend

Pas de nouvelle table. Un seul changement :

- `edd_reference.document_id` (déjà nullable, prévu par EP-001-S09) devient effectivement renseigné à la création : `POST /buildings/{id}/edd-references` accepte un `document_id` optionnel (document déjà uploadé + catégorisé `edd` via le flux standard `documentsApi`).
- Réutiliser tel quel le mécanisme de document fondateur EDD (catégorie `edd`, `is_current`, `supersedes_id`) déjà livré par EP-005-S06 : aucune nouvelle route document, aucun nouveau champ `category` à exposer ailleurs.
- `GET /buildings/{id}/edd-references` (existant) doit retourner `document_id` par référence pour permettre le téléchargement/historique côté frontend.

Pas d'ADR : réutilisation stricte de mécanismes déjà livrés (EP-005-S06/S08), aucune alternative structurelle.

## 4. Frontend

1. **Fusion des onglets** — `AllocationKeysView.vue` :
   - Ajouter `CTabs` avec deux entrées : `Clés` (contenu actuel inchangé) et `EDD` (contenu de `EddView.vue`, adapté en sous-composant, pas de route dédiée).
   - Supprimer la route `residence/edd` et l'entrée de nav dédiée (`AppNav.vue`, `router/index.ts`) — l'onglet EDD n'est accessible que depuis `residence/allocation-keys`.
2. **Upload inline dans le formulaire de référence EDD** (repris de `feat/ep-001-s09-frontend-edd`, section « Entry form ») :
   - Ajouter un champ fichier (composant `CFormInput` ou équivalent file-input du catalogue) dans le formulaire de saisie de référence.
   - À la soumission : upload via `documentsApi.getUploadUrl` → `uploadToS3` → `confirmUpload` (catégorie `edd` posée côté appel, pas saisie utilisateur) → passer le `document_id` résultant à `eddApi.createReference(...)`.
   - Champ optionnel : la saisie de référence sans fichier reste possible (rétrocompatible avec EP-001-S09), mais un bandeau/alerte incite à joindre le document réel.
3. **Téléchargement + historique** :
   - Bouton « Télécharger l'EDD » sur la référence courante → `documentsApi.getDownloadUrl(document_id)` (pattern déjà utilisé partout).
   - Bouton « Historique » ouvrant une modale listant `eddApi.listReferences()` (déjà existant) avec pour chaque entrée : libellé, date, et bouton téléchargement si `document_id` renseigné. Réutiliser `CLinkedDocumentsModal.vue` si la forme de la liste s'y prête, sinon une modale dédiée simple (pas de nouveau composant catalogue si évitable).
4. **Types** — `frontend/src/types/domain.ts` : ajouter `document_id?: string | null` à `EddReference` si absent.
5. Aucun nouveau composant `C*` catalogue anticipé ; si un besoin apparaît en cours d'implémentation, l'ajouter à `docs/DESIGN.md §2` avant usage (étape dédiée, ne pas improviser).

## 5. Cas limites

- Référence EDD sans document attaché (cas hérité EP-001-S09) → téléchargement non proposé pour cette entrée, pas d'erreur.
- Remplacement d'un document EDD courant → suit exactement le workflow EP-005-S08 (justification horodatée avant invalidation), aucune logique nouvelle à écrire ici.
- Utilisateur change d'onglet pendant un upload en cours → upload continue en arrière-plan (comportement standard des modales d'upload existantes, à vérifier ne pas régresser).

## 6. Critères d'acceptation

1. La page « Clés de répartition » affiche deux onglets : `Clés` et `EDD`, sans route séparée pour EDD.
2. Depuis l'onglet EDD, le syndic peut saisir une référence ET joindre le PDF EDD réel en une seule action (upload inline).
3. Le document EDD attaché est catégorisé `edd` automatiquement, sans saisie manuelle de catégorie nulle part dans le flux.
4. Le document EDD courant est téléchargeable directement depuis l'onglet.
5. Un historique des documents EDD précédents est consultable et chaque entrée téléchargeable si un document y est attaché.
6. Aucune régression sur la page Documents générique (catégorie reste implicite/absente de l'UI générique).

## 7. Plan de tests

- **Frontend** : test manuel du flux upload inline (mock API) ; vérifier que `confirmUpload`/`createReference` sont bien enchaînés avec le bon `document_id`.
- **Backend** : test e2e existant EP-001-S09 étendu pour vérifier que `document_id` est bien persisté et restitué par `GET /buildings/{id}/edd-references`.
- `npm run build` / `npm run lint` doivent passer sans erreur après suppression de la route `residence/edd`.

## 8. Ordre d'implémentation

1. Backend : accepter `document_id` optionnel dans `POST /buildings/{id}/edd-references`, l'exposer dans `GET`.
2. Frontend : fusionner `EddView.vue` en onglet dans `AllocationKeysView.vue` via `CTabs`, retirer route/nav dédiées (`residence/edd`).
3. Frontend : ajouter l'upload inline au formulaire de référence + boutons téléchargement/historique.
4. `docs/PROGRESS.md` : entrée datée EP-001-S11.
5. `graphify update .`, PR + revue Hopper (`kanban_request_review`).
