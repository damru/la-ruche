<script setup lang="ts">
// Molecule component — CListItem
// "No-Line" rule: spacing and background only, never borders between items

withDefaults(defineProps<{
  interactive?: boolean
  selected?: boolean
}>(), {
  interactive: false,
  selected: false,
})

const emit = defineEmits<{ click: [] }>()
</script>

<template>
  <div
    :class="[
      'c-list-item',
      { 'c-list-item--interactive': interactive, 'c-list-item--selected': selected }
    ]"
    :role="interactive ? 'button' : undefined"
    :tabindex="interactive ? 0 : undefined"
    @click="interactive && emit('click')"
    @keydown.enter="interactive && emit('click')"
  >
    <!-- Avatar / icône gauche -->
    <div
      v-if="$slots.leading"
      class="c-list-item__leading"
    >
      <slot name="leading"></slot>
    </div>

    <!-- Contenu principal -->
    <div class="c-list-item__body">
      <slot></slot>
    </div>

    <!-- Actions / badge droit -->
    <div
      v-if="$slots.trailing"
      class="c-list-item__trailing"
    >
      <slot name="trailing"></slot>
    </div>
  </div>
</template>

<style scoped>
.c-list-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-xl);
  background: var(--color-surface-container-lowest);
  transition: background var(--transition-fast);
}

.c-list-item--interactive {
  cursor: pointer;
}
.c-list-item--interactive:hover {
  background: var(--color-surface-container-low);
}
.c-list-item--interactive:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 1px;
}

.c-list-item--selected {
  background: var(--color-secondary-container);
}

.c-list-item__leading { flex-shrink: 0; }

.c-list-item__body {
  flex: 1;
  min-width: 0; /* prevents text overflow */
}

.c-list-item__trailing {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
</style>
