# Spécification technique — EP-020-S16

**Story :** Créer la molécule `CDocumentList` (liste de documents liés avec téléchargement et retrait) et remplacer les listes de documents dupliquées dans les modales contrats, tâches, sinistres et rapports.

**Épic :** EP-020 — Design system
**Réf. DESIGN :** `docs/DESIGN.md` §3.2 (ligne `CDocumentList` — 🔲 planifié), §4.1 « Gestion documentaire liée » (« `CDocumentList` émet `remove(doc)` — le parent gère l'appel API (endpoints différents par entité) » ; « le retrait d'un document ouvre toujours `CDocumentLinkActionModal` »).

> Frontend uniquement.

---

## 1. État actuel

Le même markup de liste (icône fichier + nom + taille · date + boutons download/retrait) est copié-collé dans quatre modales, avec des classes BEM locales et des `<style scoped>` quasi identiques :

- `frontend/src/features/contracts/ContractFormModal.vue:505-543` — classes `contract-form__documents-*`, boutons `CBtn ghost CIconDownload` + `CBtn danger CIconTrash`.
- `frontend/src/features/tasks/SaveTaskModal.vue:460-500` — classes `task-existing-documents__*`, mêmes boutons, avec `:loading` par document (`pendingDocument?.id === document.id`).
- `frontend/src/features/claims/ClaimEditModal.vue:326-345+` — classes `claim-documents-*`, même structure.
- `frontend/src/features/reports/EditReportModal.vue` — quatrième consommateur de `CDocumentLinkActionModal`, même pattern.

Chaque copie duplique aussi `formatFileSize`/`formatDate` côté template et l'état vide (« aucun document attaché »).

Le composant aval existe déjà : `frontend/src/components/organisms/CDocumentLinkActionModal.vue` — props `open`, `title`, `description`, `warning?`, `loading?`, `status?`, `message?` ; emits `update:open`, `unlink`, `unlink-and-delete`. `CDocumentList` ne l'ouvre **pas** lui-même : il émet `remove(doc)` et le parent orchestre la modale (endpoints différents par entité — DESIGN §4.1).

## 2. API du composant

`frontend/src/components/molecules/CDocumentList.vue` :

```ts
import type { DocumentSummary } from '@/types/domain'
// contrat minimal réellement consommé : id, original_name, size_bytes, created_at

withDefaults(defineProps<{
  documents: DocumentSummary[]
  readonly?: boolean          // masque le bouton de retrait (téléchargement seul)
  loadingDocId?: string | null // id du document dont le retrait est en cours → :loading sur son bouton
  removeTitle?: string        // tooltip du bouton retrait (déjà traduit) — défaut t('documents.unlinkOnly') générique
}>(), {
  readonly: false,
  loadingDocId: null,
})

const emit = defineEmits<{
  remove: [doc: DocumentSummary]
  download: [doc: DocumentSummary]
}>()
```

Slot optionnel `#empty` — état vide (défaut : rien ; les parents affichent déjà leur message hors liste).

> Le téléchargement est aussi délégué (`emit('download', doc)`) : chaque modale a aujourd'hui son propre `downloadDocument(documentId)` (`ContractFormModal.vue:246`, `SaveTaskModal.vue:222`) et les URLs diffèrent par entité — même raison que pour `remove`.

## 3. Comportement / design

- Une rangée par document : `flex items-center justify-between gap-3` ; méta en colonne (`nom` puis `taille · date` en `text-xs text-on-surface-variant`), nom précédé de `CIconFile` (`icon-sm`), nom tronqué avec ellipse (`min-width: 0`).
- Boutons d'action conformes à DESIGN §4.1 « Actions de ligne » : `CBtn` icône seule avec `:title` **et** `:aria-label` — download `variant="ghost"` `CIconDownload`, retrait `variant="danger"` `CIconTrash` (masqué si `readonly`).
- Formatage taille/date : réutiliser les utilitaires existants du repo (ceux qu'importent déjà les 4 modales) — pas de nouvelle implémentation.
- Tokens uniquement, pas de valeurs brutes ; aucun import radix-vue ; le composant ne fait **aucun** appel API.
- A11y : liste rendue en `<ul>/<li>` ; boutons avec `aria-label` explicites incluant le nom du fichier.

## 4. Plan de migration

1. Vue pilote : `ContractFormModal.vue` — remplacer le bloc L505-543 + supprimer les styles `contract-form__documents-*` orphelins.
2. `SaveTaskModal.vue` (utilise `loadingDocId`), `ClaimEditModal.vue` (+ `ClaimCreateModal`/`ClaimCloseModal` si elles rendent la même liste), `EditReportModal.vue`.
3. Chaque parent conserve : le chargement des documents, `downloadDocument`, l'ouverture de `CDocumentLinkActionModal` sur `@remove`, et les deux handlers `unlink`/`unlink-and-delete`.
4. Hors périmètre : `DocumentsView.vue` (table pleine page, pattern `CDataTable`), `CLinkedDocumentsModal` (lecture seule existante — candidate à une adoption ultérieure avec `readonly`).

## 5. Critères d'acceptation

- [ ] `CDocumentList.vue` existe dans `frontend/src/components/molecules/`, typé TS, tokens uniquement, zéro appel API.
- [ ] `emit('remove', doc)` et `emit('download', doc)` remontent le document complet ; `readonly` masque le retrait.
- [ ] `loadingDocId` affiche le spinner sur le seul bouton concerné (parité avec `SaveTaskModal` actuel).
- [ ] Les 4 modales (`ContractFormModal`, `SaveTaskModal`, `ClaimEditModal`, `EditReportModal`) utilisent `CDocumentList` ; leurs styles BEM `*-documents-*` dupliqués sont supprimés.
- [ ] Le flux complet retrait → `CDocumentLinkActionModal` → `unlink` / `unlink-and-delete` fonctionne à l'identique dans chaque entité.
- [ ] `docs/DESIGN.md` §3.2 : ligne `CDocumentList` passée à « ✅ » avec le contrat final (fait avant première utilisation).

## 6. Plan de tests

- `npm --prefix frontend run build` et `npm --prefix frontend run lint` — passent sans erreur.
- Test manuel par entité : attacher, télécharger, retirer (lien seul / lien + document) un document sur un contrat, une tâche, un sinistre, un rapport.
- État vide et nom de fichier très long (troncature) vérifiés dans la modale la plus étroite.

## 7. Ordre d'implémentation

1. Mettre à jour `docs/DESIGN.md` §3.2 (statut + props/emits finaux, y compris `download`).
2. Créer `CDocumentList.vue` (markup extrait de `ContractFormModal.vue:505-543`).
3. Migrer `ContractFormModal` (pilote), tester le flux complet.
4. Migrer `SaveTaskModal`, `ClaimEditModal` (+ modales claims sœurs le cas échéant), `EditReportModal` ; supprimer les styles orphelins.
5. Entrée datée dans `docs/PROGRESS.md` (réf. EP-020-S16).
