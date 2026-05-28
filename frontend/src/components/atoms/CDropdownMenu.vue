<script setup lang="ts">
import type { Component } from 'vue'
import {
  DropdownMenuRoot,
  DropdownMenuTrigger,
  DropdownMenuPortal,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from 'radix-vue'

export interface DropdownItem {
  label: string
  icon?: Component
  action: () => void
  destructive?: boolean
  disabled?: boolean
  separator?: boolean
}

const props = withDefaults(defineProps<{
  items: DropdownItem[]
  align?: 'start' | 'end' | 'center'
}>(), {
  align: 'end',
})

defineOptions({ inheritAttrs: false })
</script>

<template>
  <DropdownMenuRoot>
    <DropdownMenuTrigger
      v-bind="$attrs"
      as-child
    >
      <slot name="trigger"></slot>
    </DropdownMenuTrigger>

    <DropdownMenuPortal>
      <DropdownMenuContent
        :align="props.align"
        :side-offset="4"
        class="min-w-dropdown-min rounded-radius-md bg-surface-container-lowest shadow-md p-1 outline-none z-overlay"
      >
        <template
          v-for="(item, i) in props.items"
          :key="i"
        >
          <DropdownMenuSeparator
            v-if="item.separator"
            class="h-px bg-outline-variant my-1"
          />
          <DropdownMenuItem
            :disabled="item.disabled"
            class="flex items-center gap-2 px-3 py-2 text-md rounded-radius-sm cursor-pointer outline-none select-none
                   text-on-surface hover:bg-surface-container
                   data-[disabled]:opacity-40 data-[disabled]:cursor-not-allowed"
            :class="item.destructive ? 'text-error hover:bg-error-container' : ''"
            @select="item.action"
          >
            <component
              :is="item.icon"
              v-if="item.icon"
              class="size-4 shrink-0"
            />
            {{ item.label }}
          </DropdownMenuItem>
        </template>
      </DropdownMenuContent>
    </DropdownMenuPortal>
  </DropdownMenuRoot>
</template>
