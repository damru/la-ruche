<script lang="ts" generic="T extends Record<string, unknown>">
export interface Column<T> {
  key: keyof T | string
  label: string
  align?: 'left' | 'right' | 'center'
  width?: string
  sortable?: boolean
  numeric?: boolean
}
</script>

<script setup lang="ts" generic="T extends Record<string, unknown>">
import { ref } from 'vue'
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
    <table
      class="c-table"
      role="table"
    >
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
              <template v-if="col.sortable">
                <CIconChevronDown
                  v-if="sortKey === String(col.key) && sortDir === 'desc'"
                  class="c-table__sort-icon c-table__sort-icon--active"
                  aria-hidden="true"
                />
                <CIconChevronRight
                  v-else
                  class="c-table__sort-icon"
                  :class="{ 'c-table__sort-icon--active': sortKey === String(col.key) }"
                  style="transform: rotate(-90deg)"
                  aria-hidden="true"
                />
              </template>
            </span>
          </th>
        </tr>
      </thead>

      <tbody>
        <!-- Loading -->
        <tr v-if="loading">
          <td
            :colspan="columns.length"
            class="c-table__empty"
          >
            <div class="c-table__loading">
              <CIconSpinner
                class="c-table__spinner"
                aria-hidden="true"
              />
              {{ t('common.loading') }}
            </div>
          </td>
        </tr>

        <!-- Vide -->
        <tr v-else-if="!rows.length">
          <td
            :colspan="columns.length"
            class="c-table__empty"
          >
            {{ emptyText ?? t('common.noData') }}
          </td>
        </tr>

        <!-- Données -->
        <tr
          v-for="row in rows"
          v-else
          :key="String(getCellValue(row, String(rowKey)))"
          class="c-table__row"
          :class="{ 'c-table__row--striped': striped }"
          @click="emit('rowClick', row)"
        >
          <td
            v-for="col in columns"
            :key="String(col.key)"
            :class="['c-table__td', `c-table__cell--${col.align ?? 'left'}`, { 'c-table__cell--numeric': col.numeric }]"
          >
            <!-- Slot nommé par colonne pour rendu custom -->
            <slot
              :name="`cell-${String(col.key)}`"
              :row="row"
              :value="getCellValue(row, String(col.key))"
            >
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
  background: var(--color-surface-container-lowest);
}

.c-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.c-table__th {
  font-size: var(--text-2xs);
  font-weight: var(--fw-semibold);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--tone-fg, var(--color-on-surface-variant));
  padding: var(--space-3) var(--space-4);
  background: var(--tone-bg, var(--color-surface-container-low));
  white-space: nowrap;
  user-select: none;
}

.c-table__th--sortable { cursor: pointer; }
.c-table__th--sortable:hover { color: var(--tone-fg, var(--color-on-surface)); }

.c-table__th-inner {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.c-table__sort-icon {
  width: var(--icon-2xs);
  height: var(--icon-2xs);
  opacity: 0.4;
  transition: opacity var(--transition-fast);
}
.c-table__sort-icon--active { 
  opacity: 1; 
  color: var(--tone-fg, var(--color-primary)); 
}

.c-table__td {
  font-size: var(--text-sm);
  color: var(--tone-fg, var(--color-on-surface));
  background: var(--tone-bg, transparent);
  padding: var(--space-1) var(--space-4);
  border-bottom: 1px solid var(--color-outline-light);
}

.c-table__row:last-child .c-table__td { border-bottom: none; }

.c-table__row { 
  transition: background var(--transition-fast); 
}
.c-table__row:hover .c-table__td { 
  background: var(--color-surface-container-low); 
}

.c-table__row--striped:nth-child(even) .c-table__td {
  background: var(--tone-bg, var(--color-surface-container));
}

/* Alignements */
.c-table__cell--left   { text-align: left; }
.c-table__cell--right  { text-align: right; }
.c-table__cell--center { text-align: center; }

.c-table__empty {
  text-align: center;
  padding: var(--space-12) var(--space-4);
  color: var(--tone-fg, var(--color-on-surface-variant));
  font-size: var(--text-md);
}

.c-table__loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  color: var(--tone-fg, var(--color-on-surface-variant));
}

.c-table__spinner {
  width: var(--icon-sm);
  height: var(--icon-sm);
  animation: spin 1s linear infinite;
}

/* Numeric cells */
.c-table .c-table__cell--numeric {
  text-align: right;
  font-family: var(--font-mono);
  font-weight: var(--fw-medium);
  font-variant-numeric: tabular-nums;
}
</style>
