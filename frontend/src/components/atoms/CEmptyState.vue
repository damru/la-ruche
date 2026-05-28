<script setup lang="ts">
withDefaults(defineProps<{
  title: string
  description?: string
  size?: 'page' | 'compact'
}>(), {
  size: 'page',
})
</script>

<template>
  <div :class="['c-empty-state', `c-empty-state--${size}`]">
    <div
      v-if="$slots.icon"
      class="c-empty-state__icon"
    >
      <slot name="icon"></slot>
    </div>
    <p class="title-sm text-on-surface">
      {{ title }}
    </p>
    <p
      v-if="description"
      class="body-sm text-muted"
    >
      {{ description }}
    </p>
    <div
      v-if="$slots.actions"
      class="c-empty-state__actions"
    >
      <slot name="actions"></slot>
    </div>
  </div>
</template>

<style scoped>
.c-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  color: var(--color-on-surface-variant);
}

.c-empty-state--page {
  justify-content: center;
  padding: var(--space-8) 0;
  gap: var(--space-2);
}

.c-empty-state--compact {
  padding: var(--space-4) 0;
  gap: var(--space-1);
}

.c-empty-state__icon {
  margin-bottom: var(--space-1);
}

.c-empty-state__actions {
  margin-top: var(--space-2);
}
</style>
