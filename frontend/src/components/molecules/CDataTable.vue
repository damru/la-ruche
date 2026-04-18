<script lang="ts" generic="T extends Record<string, unknown>">
export interface Column<T> {
  key: keyof T | string
  label: string
  align?: 'left' | 'right' | 'center'
  width?: string
  sortable?: boolean
}
</script>

<script setup lang="ts" generic="T extends Record<string, unknown>">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

withDefaults(defineProps<{
  columns: Column<T>[]
  rows: T[]
  rowKey?: keyof T
  loading?: boolean
  emptyText?: string
  striped?: boolean
}>(), {
  rowKey: 'id' as keyof T,
  loading: false,
  emptyText: undefined,
  striped: false,
})

const emit = defineEmits<{
  rowClick: [row: T]
  sort: [key: string, direction: 'asc' | 'desc']
}>()

const sortKey = ref<string | null>(null)
const sortDir = ref<'asc' | 'desc'>('asc')

function handleSort(col: Column<T>) {
  if (!col.sortable) return
  const key = String(col.key)
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortKey.value = key
    sortDir.value = 'asc'
  }
  emit('sort', sortKey.value, sortDir.value)
}

function getCellValue(row: T, key: string): unknown {
  return key.split('.').reduce((obj, k) => (obj as Record<string, unknown>)?.[k], row as unknown)
}
</script>

<template>
  <div class="c-table-wrap">
    <table class="c-table" role="table">
      <thead>
        <tr>
          <th
            v-for="col in columns"
            :key="String(col.key)"
            :style="col.width ? { width: col.width } : undefined"
            :class="['c-table__th', `c-table__cell--${col.align ?? 'left'}`, { 'c-table__th--sortable': col.sortable }]"
            @click="handleSort(col)"
          >
            <span class="c-table__th-inner">
              {{ col.label }}
              <svg
                v-if="col.sortable"
                class="c-table__sort-icon"
                :class="{ 'c-table__sort-icon--active': sortKey === String(col.key) }"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"
              >
                <path v-if="sortKey === String(col.key) && sortDir === 'desc'" d="M6 9l6 6 6-6"/>
                <path v-else d="M6 15l6-6 6 6"/>
              </svg>
            </span>
          </th>
        </tr>
      </thead>

      <tbody>
        <!-- Loading -->
        <tr v-if="loading">
          <td :colspan="columns.length" class="c-table__empty">
            <div class="c-table__loading">
              <svg class="c-table__spinner" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                <path d="M12 2v4m0 12v4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M2 12h4m12 0h4M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83"/>
              </svg>
              {{ t('common.loading') }}
            </div>
          </td>
        </tr>

        <!-- Vide -->
        <tr v-else-if="!rows.length">
          <td :colspan="columns.length" class="c-table__empty">{{ emptyText ?? t('common.noData') }}</td>
        </tr>

        <!-- Données -->
        <tr
          v-else
          v-for="row in rows"
          :key="String(getCellValue(row, String(rowKey)))"
          class="c-table__row"
          :class="{ 'c-table__row--striped': striped }"
          @click="emit('rowClick', row)"
        >
          <td
            v-for="col in columns"
            :key="String(col.key)"
            :class="['c-table__td', `c-table__cell--${col.align ?? 'left'}`]"
          >
            <!-- Slot nommé par colonne pour rendu custom -->
            <slot :name="`cell-${String(col.key)}`" :row="row" :value="getCellValue(row, String(col.key))">
              {{ getCellValue(row, String(col.key)) ?? '—' }}
            </slot>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.c-table-wrap {
  overflow-x: auto;
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-lowest);
  box-shadow: var(--shadow-md);
}

.c-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.c-table__th {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
  padding: var(--space-3) var(--space-4);
  background: var(--color-surface-container-low);
  white-space: nowrap;
  user-select: none;
}

.c-table__th:first-child { border-radius: var(--radius-sm) 0 0 0; }
.c-table__th:last-child  { border-radius: 0 var(--radius-sm) 0 0; }

.c-table__th--sortable { cursor: pointer; }
.c-table__th--sortable:hover { color: var(--color-on-surface); }

.c-table__th-inner {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.c-table__sort-icon {
  width: 12px;
  height: 12px;
  opacity: 0.4;
  transition: opacity var(--transition-fast);
}
.c-table__sort-icon--active { opacity: 1; color: var(--color-primary); }

.c-table__td {
  font-size: 13px;
  color: var(--color-on-surface);
  padding: var(--space-4) var(--space-4);
  /* No-Line rule — subtle alternating background only when striped */
  border-bottom: 1px solid rgba(194, 199, 207, 0.12);
}

.c-table__row:last-child .c-table__td { border-bottom: none; }

.c-table__row { transition: background var(--transition-fast); }
.c-table__row:hover .c-table__td { background: var(--color-surface-container-low); }

.c-table__row--striped:nth-child(even) .c-table__td {
  background: var(--color-surface-container);
}

/* Alignements */
.c-table__cell--left   { text-align: left; }
.c-table__cell--right  { text-align: right; }
.c-table__cell--center { text-align: center; }

.c-table__empty {
  text-align: center;
  padding: var(--space-12) var(--space-4);
  color: var(--color-on-surface-variant);
  font-size: 14px;
}

.c-table__loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  color: var(--color-on-surface-variant);
}

.c-table__spinner {
  width: 16px;
  height: 16px;
  animation: spin 1s linear infinite;
}
</style>
