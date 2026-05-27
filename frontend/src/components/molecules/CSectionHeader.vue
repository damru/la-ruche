<template>
  <div class="c-section-header">
    <div class="c-section-header__body">
      <slot />
    </div>
    <div v-if="$slots.actions" class="c-section-header__actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.c-section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
}

.c-section-header__body {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.c-section-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}
</style>
