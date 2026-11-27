<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router'
import { useI18n } from 'vue-i18n'
import CIconChevronRight from '@/components/atoms/CIconChevronRight.svg'

export interface Crumb {
  label: string
  to?: RouteLocationRaw
}

defineProps<{
  crumbs: Crumb[]
}>()

const { t } = useI18n()
</script>

<template>
  <nav
    class="flex items-center gap-2 min-w-0 text-on-surface-variant"
    :aria-label="t('common.breadcrumb')"
  >
    <template
      v-for="(crumb, index) in crumbs"
      :key="index"
    >
      <CIconChevronRight
        v-if="index > 0"
        class="icon-xs shrink-0 text-outline-variant"
        aria-hidden="true"
      />
      <RouterLink
        v-if="crumb.to"
        :to="crumb.to"
        :class="['truncate min-w-0', index === crumbs.length - 1 ? 'shrink-[1]' : 'shrink-[8]']"
      >
        {{ crumb.label }}
      </RouterLink>
      <span
        v-else
        :class="['truncate min-w-0', index === crumbs.length - 1 ? 'shrink-[1]' : 'shrink-[8]']"
        :aria-current="index === crumbs.length - 1 ? 'page' : undefined"
      >
        {{ crumb.label }}
      </span>
    </template>
  </nav>
</template>
