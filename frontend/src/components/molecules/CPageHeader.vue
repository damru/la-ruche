<script setup lang="ts">
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'

withDefaults(defineProps<{
  eyebrow?: string
  title?: string
  meta?: string
}>(), {
  eyebrow: undefined,
  title: undefined,
  meta: undefined,
})
</script>

<template>
  <header class="c-page-header flex items-start justify-between gap-3 min-w-0">
    <div class="flex flex-col items-start gap-2 min-w-0">
      <slot name="eyebrow">
        <CEyebrow v-if="eyebrow">
          {{ eyebrow }}
        </CEyebrow>
      </slot>
      <slot name="title">
        <CHeading
          v-if="title"
          type="header"
        >
          {{ title }}
        </CHeading>
      </slot>
      <CMetaItem
        v-if="meta"
        variant="sub"
      >
        {{ meta }}
      </CMetaItem>
      <slot></slot>
    </div>
    <div
      v-if="$slots.actions"
      class="flex items-center gap-2 shrink-0"
    >
      <slot name="actions"></slot>
    </div>
  </header>
</template>
