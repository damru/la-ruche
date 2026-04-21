# DDR-005 — États de chargement et d'erreur de page

**Date :** 2026-04-22  
**Statut :** Adopté

## Contexte

Les vues qui fetchent des données au montage (`onMounted`) échouaient silencieusement en cas d'erreur réseau ou de réponse `5xx/501`. L'utilisateur voyait une page vide sans indication ni possibilité de réessayer.

## Décision

Adoption d'un pattern transverse en deux parties :

### 1. Composable `usePageLoader`
`frontend/src/composables/usePageLoader.ts` — wrape n'importe quelle fonction `async () => void` et expose trois réfs : `loading`, `error` (message string ou `null`), `reload`.

### 2. Molecule `CPageError`
`frontend/src/components/molecules/CPageError.vue` — affiche un état d'erreur centré : icône circulaire rouge, titre `common.loadError`, message optionnel, bouton "Réessayer" qui émet `@retry`.

### Pattern d'usage dans une vue

```vue
const { loading, error, reload } = usePageLoader(loadData)
onMounted(reload)
```

```html
<CPageError v-if="error" :message="error" @retry="reload" />
<template v-else-if="loading">
  <div class="page-loading"><span class="c-btn__spinner" /></div>
</template>
<template v-else>
  <!-- contenu normal -->
</template>
```

## Alternatives écartées

| Option | Raison du rejet |
|---|---|
| `try/catch` ad-hoc par vue | Code dupliqué, comportement inconsistant entre vues |
| Toast de notification | Ne bloque pas le contenu vide — l'utilisateur peut croire que la page est simplement vide |
| `Promise.all` sans isolation | Une seule requête en erreur (ex: stub 501) casse toutes les données de la page |

## Règles

- Toute vue avec `onMounted(loadData)` doit utiliser `usePageLoader`
- `CPageError` remplace le contenu entier (pas inline dans une section)
- Le `loading` spinner utilise `.c-btn__spinner` (réutilisation du spinner existant)
- Les appels optionnels ou non-critiques (`usersApi` en dégradé gracieux) restent dans un `try/catch` séparé **après** le `Promise.all` principal
