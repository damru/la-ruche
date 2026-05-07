<script setup lang="ts">
defineProps<{
  title: string
  summary: string
}>()
</script>

<template>
  <p class="body-xs legal-info-card__text">
    ⚖️ <strong>{{ title }}</strong> — {{ summary }}
  </p>
</template>

<style scoped>
.legal-info-card__text {
  margin: 0;
  color: var(--color-on-surface-variant);
}
</style>