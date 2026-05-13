<script setup lang="ts">
type LegalLink = {
  label: string
  href: string
}

withDefaults(defineProps<{
  id?: string
  links?: LegalLink[]
}>(), {
  id: undefined,
  links: () => [],
})
</script>

<template>
  <div
    :id="id"
    class="c-legal-notice"
  >
    <slot></slot>
    <div
      v-if="links.length > 0"
      class="c-legal-notice__links"
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
          v-if="link.href.startsWith('#')"
          class="c-legal-notice__link"
          :href="link.href"
        >
          {{ link.label }}
        </a>
        <a
          v-else
          class="c-legal-notice__link"
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
.c-legal-notice {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.c-legal-notice__links {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 0;
  color: var(--color-on-surface-variant);
}

.c-legal-notice__link {
  color: var(--color-primary);
  text-decoration: none;
}

.c-legal-notice__link:hover,
.c-legal-notice__link:focus-visible {
  text-decoration: underline;
}
</style>