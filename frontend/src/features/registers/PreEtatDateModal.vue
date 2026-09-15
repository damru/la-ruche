<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import { useFormatters } from '@/composables/useFormatters'
import { useBuildingStore } from '@/stores/building'
import { etatDateApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { PreEtatDate } from '@/types/domain'

const props = defineProps<{
  open: boolean
  unitId: string
  unitNumber: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { formatCurrency, formatDate } = useFormatters()
const buildingName = ref('')

const data = ref<PreEtatDate | null>(null)
const loading = ref(false)
const error = ref('')
const editionDate = ref('')

watch(
  () => props.open,
  async open => {
    if (!open) return
    data.value = null
    error.value = ''
    loading.value = true
    editionDate.value = new Date().toISOString().slice(0, 10)
    try {
      const [building, res] = await Promise.all([
        useBuildingStore().fetch(),
        etatDateApi.preEtatDate(props.unitId),
      ])
      buildingName.value = building.name
      data.value = res.data
    } catch (err) {
      error.value = toErrorMessage(err)
    } finally {
      loading.value = false
    }
  },
)

function amount(value: number | null): string {
  return value === null ? t('registers.preEtatDate.notAvailable') : formatCurrency(value)
}

function print() {
  window.print()
}
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :title="t('registers.preEtatDate.documentTitle')"
    @update:open="emit('update:open', $event)"
  >
    <CPageLoader v-if="loading" />
    <p
      v-else-if="error"
      class="tone--err data-sm"
    >
      {{ error }}
    </p>
    <div
      v-else-if="data"
      class="pre-etat-date-print flex flex-col gap-4"
    >
      <header class="flex flex-col gap-1">
        <p class="data-md font-semibold">
          {{ buildingName }}
        </p>
        <p class="data-sm">
          {{ t('common.lot') }} {{ unitNumber }}
        </p>
        <p class="data-sm opacity-70">
          {{ t('registers.preEtatDate.editedAt') }} {{ formatDate(editionDate) }}
        </p>
      </header>

      <section>
        <h3 class="data-md font-semibold mb-2">
          {{ t('registers.preEtatDate.charges') }}
        </h3>
        <ul>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.chargesCourantesN1') }}</span>
            <span>{{ amount(data.charges_courantes_n1) }}</span>
          </li>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.chargesCourantesN2') }}</span>
            <span>{{ amount(data.charges_courantes_n2) }}</span>
          </li>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.chargesHorsBudgetN1') }}</span>
            <span>{{ amount(data.charges_hors_budget_n1) }}</span>
          </li>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.chargesHorsBudgetN2') }}</span>
            <span>{{ amount(data.charges_hors_budget_n2) }}</span>
          </li>
        </ul>
      </section>

      <section>
        <h3 class="data-md font-semibold mb-2">
          {{ t('registers.preEtatDate.sommesDues') }}
        </h3>
        <ul>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.sommesDuesParCedant') }}</span>
            <span>{{ formatCurrency(data.sommes_dues_par_cedant) }}</span>
          </li>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.impayesGlobaux') }}</span>
            <span>{{ formatCurrency(data.impayes_globaux_syndicat) }}</span>
          </li>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.detteFournisseurs') }}</span>
            <span>{{ amount(data.dette_fournisseurs) }}</span>
          </li>
        </ul>
      </section>

      <section>
        <h3 class="data-md font-semibold mb-2">
          {{ t('registers.preEtatDate.fondsTravaux') }}
        </h3>
        <ul>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.fondsTravauxQuotePart') }}</span>
            <span>{{ formatCurrency(data.fonds_travaux_quote_part) }}</span>
          </li>
          <li class="flex justify-between">
            <span>{{ t('registers.preEtatDate.fondsTravauxDerniereCotisation') }}</span>
            <span>{{ formatCurrency(data.fonds_travaux_derniere_cotisation) }}</span>
          </li>
        </ul>
      </section>

      <section>
        <h3 class="data-md font-semibold mb-2">
          {{ t('registers.preEtatDate.procedures') }}
        </h3>
        <p class="data-sm opacity-70">
          {{ data.procedures_en_cours === 0 ? t('registers.etatDate.none') : data.procedures_en_cours }}
        </p>
      </section>

      <section>
        <h3 class="data-md font-semibold mb-2">
          {{ t('registers.etatDate.emprunts') }}
        </h3>
        <p class="data-sm opacity-70">
          {{ data.emprunts_collectifs }}
        </p>
      </section>
    </div>
    <template #footer>
      <CBtn
        variant="ghost"
        :title="t('common.close')"
        @click="emit('update:open', false)"
      />
      <CBtn
        v-if="data"
        variant="primary"
        :title="t('registers.etatDate.print')"
        @click="print"
      />
    </template>
  </CModal>
</template>

<style scoped>
@media print {
  .pre-etat-date-print {
    color: black;
  }
}
</style>
