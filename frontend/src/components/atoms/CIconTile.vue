<script setup lang="ts">
withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg'
}>(), {
  size: 'md',
})
</script>

<template>
  <div :class="['c-icon-tile', `c-icon-tile--${size}`]">
    <slot />
  </div>
</template>

<style scoped>
.c-icon-tile {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: var(--radius-md);
  background: var(--tone-bg, var(--color-secondary-container));
  color: var(--tone-fg, var(--color-primary));
}

.c-icon-tile--sm {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
}

.c-icon-tile--md {
  width: 40px;
  height: 40px;
}

.c-icon-tile--lg {
  width: 56px;
  height: 56px;
  border-radius: var(--radius-lg);
}

.c-icon-tile > :deep(svg) {
  width: var(--icon-md);
  height: var(--icon-md);
  flex-shrink: 0;
}

.c-icon-tile--sm > :deep(svg) {
  width: var(--icon-sm);
  height: var(--icon-sm);
}

.c-icon-tile--lg > :deep(svg) {
  width: var(--icon-lg);
  height: var(--icon-lg);
}
</style>
