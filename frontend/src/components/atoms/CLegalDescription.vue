<script setup lang="ts">
defineProps<{
  title: string
  summary: string
}>()
</script>

<template>
  <p class="c-legal-description">
    ⚖️ <strong>{{ title }}</strong> — {{ summary }}
  </p>
</template>

<style scoped>
.c-legal-description {
  margin: 0;
  color: var(--color-on-surface-variant);
}
</style>