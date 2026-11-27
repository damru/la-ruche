<script setup lang="ts">
import type { Component } from 'vue'
import CEyebrow from '@/components/atoms/CEyebrow.vue'
import CHeading from '@/components/atoms/CHeading.vue'
import CIconLock from '@/components/atoms/CIconLock.svg'
import CIconTile from '@/components/atoms/CIconTile.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'

withDefaults(defineProps<{
  eyebrow?: string
  title?: string
  meta?: string
  icon?: Component
  frozenLabel?: string
}>(), {
  eyebrow: undefined,
  title: undefined,
  meta: undefined,
  icon: undefined,
  frozenLabel: undefined,
})
</script>

<template>
  <header class="flex items-start justify-between gap-3 min-w-0">
    <div class="flex items-start gap-3 min-w-0">
      <CIconTile
        v-if="icon"
        size="md"
      >
        <component
          :is="icon"
          aria-hidden="true"
        />
      </CIconTile>
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
        <span
          v-if="frozenLabel"
          class="tone--muted inline-flex items-center gap-1 body-sm"
        >
          <CIconLock
            class="icon-xs"
            aria-hidden="true"
          />
          {{ frozenLabel }}
        </span>
        <slot></slot>
      </div>
    </div>
    <div
      v-if="$slots.actions"
      class="flex items-center gap-2 shrink-0"
    >
      <slot name="actions"></slot>
    </div>
  </header>
</template>
