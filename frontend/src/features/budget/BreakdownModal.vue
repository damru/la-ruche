<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import CModal from '@/components/molecules/CModal.vue'
import CMetaItem from '@/components/molecules/CMetaItem.vue'
import CFlexGroup from '@/components/organisms/CFlexGroup.vue'

const props = defineProps<{
  open: boolean
  items: Array<{ id: string; label: string; amount: number; color: string; sharePct: number }>
}>()

defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { formatCurrency } = useFormatters()


const breakdownStyle = computed((): Record<string, string> => {
  if (props.items.length === 0) {
    return {}
  }

  let start = 0
  const segments = props.items.map((item) => {
    const end = start + (item.sharePct / 100) * 360
    const segment = `${item.color} ${start}deg ${end}deg`
    start = end
    return segment
  })

  return {
    background: `conic-gradient(${segments.join(', ')})`,
  }
})

const sharePercentageFormatter = new Intl.NumberFormat('fr-FR', {
  minimumFractionDigits: 1,
  maximumFractionDigits: 1,
})

function formatSharePercentage(value: number): string {
  return sharePercentageFormatter.format(value)
}
</script>

<template>
  <CModal
    :open="props.open"
    :title="t('budget.breakdownTitle')"
    size="lg"
    @update:open="$emit('update:open', $event)"
  >
    <p class="body-sm">
      {{ t('budget.breakdownIntro') }}
    </p>
    <div class="mt-4 grid items-center gap-6 sm:grid-cols-[1fr_auto]">
      <div class="flex justify-center">
        <div
          class="w-[min(280px,56vw)] aspect-square rounded-full"
          :style="{ ...breakdownStyle, boxShadow: 'inset 0 0 0 1px color-mix(in srgb, var(--color-on-surface) 5%, transparent)' }"
          role="img"
          :aria-label="t('budget.breakdownAriaLabel')"
        ></div>
      </div>
      <div class="flex flex-col gap-3">
        <div
          v-for="item in props.items"
          :key="item.id"
          class="flex items-start gap-3"
        >
          <span
            class="mt-1 size-3 shrink-0 rounded-full"
            :style="{ background: item.color }"
            aria-hidden="true"
          ></span>
          <div class="flex min-w-0 flex-col gap-1">
            <strong class="text-sm text-on-surface">{{ item.label }}</strong>
            <CFlexGroup
              direction="row"
              class="flex-wrap"
            >
              <CMetaItem variant="sub">
                {{ formatCurrency(item.amount) }}
              </CMetaItem>
              <CMetaItem variant="sub">
                {{ t('budget.breakdownShare', { pct: formatSharePercentage(item.sharePct) }) }}
              </CMetaItem>
            </CFlexGroup>
          </div>
        </div>
      </div>
    </div>
  </CModal>
</template>
