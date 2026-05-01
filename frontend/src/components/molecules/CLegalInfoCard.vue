<script setup lang="ts">
import CCard from '@/components/molecules/CCard.vue'

type LegalLink = {
  label: string
  href: string
}

defineProps<{
  title: string
  summary: string
  links: LegalLink[]
}>()
</script>

<template>
  <CCard class="legal-info-card">
    <p class="body-sm legal-info-card__text">
      ⚖️ <strong>{{ title }}</strong> — {{ summary }}
    </p>
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
  </CCard>
</template>

<style scoped>
.legal-info-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.legal-info-card__text,
.legal-info-card__links {
  margin: 0;
  color: var(--color-on-surface-variant);
}

.legal-info-card__links {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
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
