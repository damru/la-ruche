<script setup lang="ts">
type LegalLink = {
  label: string
  href: string
}

withDefaults(defineProps<{
  links?: LegalLink[]
}>(), {
  links: () => [],
})
</script>

<template>
  <div class="legal-info-card">
    <slot></slot>
    <div
      v-if="links.length > 0"
      class="legal-info-card__links"
    >
      <template
        v-for="(link, index) in links"
        :key="link.href"
      >
        <span
          v-if="index > 0"
          aria-hidden="true"
        >·</span>
        <a
          class="legal-info-card__link"
          :href="link.href"
          target="_blank"
          rel="noreferrer noopener"
        >
          {{ link.label }}
        </a>
      </template>
    </div>
  </div>
</template>

<style scoped>
.legal-info-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.legal-info-card__links {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 0;
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
}

.legal-info-card__link {
  color: var(--color-primary);
  text-decoration: none;
}

.legal-info-card__link:hover,
.legal-info-card__link:focus-visible {
  text-decoration: underline;
}
</style>