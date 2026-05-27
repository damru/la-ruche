<template>
  <div class="c-card-actions">
    <slot />
  </div>
</template>

<style scoped>
.c-card-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2) var(--space-2);
}
</style>
