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
    <ul
      v-if="links.length > 0"
      class="c-legal-notice__links"
    >
      <li
        v-for="link in links"
        :key="link.href"
        class="c-legal-notice__link"
      >
        <a
          v-if="link.href.startsWith('#')"
          :href="link.href"
        >
          {{ link.label }}
        </a>
        <a
          v-else
          :href="link.href"
          target="_blank"
          rel="noreferrer noopener"
        >
          {{ link.label }}
        </a>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.c-legal-notice {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.c-legal-notice__links {
  list-style: circle;
  list-style-position: inside;
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