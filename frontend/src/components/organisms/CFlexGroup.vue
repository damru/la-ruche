<script lang="ts">
// Organism — CFlexGroup
// Flex container that automatically inserts a CSeparator between each
// top-level slotted element. Comment/Text vnodes (produced by v-if=false)
// are filtered out so conditional items don't produce orphan separators.
//
//   direction="column"   → flex-direction: column + CSeparator variant="vertical" (hr rule)
//   direction="row"      → flex-direction: row    + CSeparator variant="horizontal" (dot)
//   tag                  → root HTML tag (default: "div"). Use "ol", "ul", "dl", etc.
//                          to preserve semantic markup.
import { defineComponent, h, Fragment, Comment, Text } from 'vue'
import CSeparator from '@/components/atoms/CSeparator.vue'

export default defineComponent({
  name: 'CFlexGroup',

  props: {
    direction: {
      type: String as () => 'row' | 'column',
      default: 'column',
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
          children.push(h(CSeparator, { variant: props.direction === 'column' ? 'vertical' : 'horizontal' }))
        }
      })

      return h(
        props.tag,
        { class: ["flex", "gap-(--space-2)", `flex ${props.direction === 'column' ? 'flex-col' : 'flex-row items-center'}`] },
        children,
      )
    }
  },
})
</script>
