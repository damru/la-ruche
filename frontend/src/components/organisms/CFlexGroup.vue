<script lang="ts">
// Organism — CFlexGroup
// Flex container that automatically inserts a CSeparator between each
// top-level slotted element. Comment/Text vnodes (produced by v-if=false)
// are filtered out so conditional items don't produce orphan separators.
//
//   variant="vertical"   → flex-direction: column + CSeparator variant="vertical" (hr rule)
//   variant="horizontal" → flex-direction: row    + CSeparator variant="horizontal" (dot)
//   tag                  → root HTML tag (default: "div"). Use "ol", "ul", "dl", etc.
//                          to preserve semantic markup.
import { defineComponent, h, Fragment, Comment, Text } from 'vue'
import CSeparator from '@/components/atoms/CSeparator.vue'

export default defineComponent({
  name: 'CFlexGroup',

  props: {
    variant: {
      type: String as () => 'horizontal' | 'vertical',
      default: 'vertical',
    },
    tag: {
      type: String,
      default: 'div',
    },
  },

  setup(props, { slots }) {
    return () => {
      const defaultSlot = slots.default?.() ?? []

      // Flatten one level of Fragment (produced by v-for or <template> wrappers)
      const flat: any[] = []
      for (const vnode of defaultSlot) {
        if (vnode.type === Fragment) {
          for (const child of (vnode.children as any[]) ?? []) {
            flat.push(child)
          }
        } else {
          flat.push(vnode)
        }
      }

      // Filter out Comment and whitespace-only Text vnodes (produced by v-if=false)
      const nodes = flat.filter(
        (n) => n.type !== Comment && n.type !== Text,
      )

      // Interleave with separators
      const children: any[] = []
      nodes.forEach((node, i) => {
        children.push(node)
        if (i < nodes.length - 1) {
          children.push(h(CSeparator, { variant: props.variant }))
        }
      })

      return h(
        props.tag,
        { class: ['c-flex-group', `c-flex-group--${props.variant}`] },
        children,
      )
    }
  },
})
</script>

<style>
.c-flex-group {
  display: flex;
  gap: var(--space-2);
}

.c-flex-group--vertical {
  flex-direction: column;
}

.c-flex-group--horizontal {
  flex-direction: row;
  align-items: center;
}
</style>
