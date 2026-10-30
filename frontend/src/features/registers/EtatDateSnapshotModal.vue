<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import { useFormatters } from '@/composables/useFormatters'
import { etatDateApi } from '@/api'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { EtatDateRequest } from '@/types/domain'

const props = defineProps<{
  open: boolean
  request: EtatDateRequest | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const { formatCurrency, formatDate } = useFormatters()

const downloading = ref(false)
const downloadError = ref('')

// Downloads the frozen snapshot rendered server-side as a PDF (ADR-011).
async function downloadPdf() {
  if (!props.request) return
  downloading.value = true
  downloadError.value = ''
  try {
    const response = await etatDateApi.exportPdf(props.request.id)
    const blob = new Blob([response.data], { type: 'application/pdf' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `etat-date-${props.request.id}.pdf`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  } catch (err) {
    downloadError.value = toErrorMessage(err)
  } finally {
    downloading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    size="lg"
    :title="t('registers.etatDate.title')"
    @update:open="emit('update:open', $event)"
  >
    <template v-if="props.request?.snapshot">
      <div class="flex flex-col gap-4">
        <p class="data-sm">
          {{ t('registers.etatDate.generatedAt') }} {{ formatDate(props.request.generated_at ?? '') }}
        </p>

        <section>
          <h3 class="title-md font-semibold mb-2">
            {{ t('registers.etatDate.partie1') }}
          </h3>
          <ul>
            <li
              v-for="(line, i) in props.request.snapshot.partie1_sommes_dues_par_cedant"
              :key="i"
              class="flex justify-between"
            >
              <span>{{ line.label }}</span>
              <span class="data-md">{{ formatCurrency(line.amount) }}</span>
            </li>
          </ul>
        </section>

        <section>
          <h3 class="title-md font-semibold mb-2">
            {{ t('registers.etatDate.partie2') }}
          </h3>
          <ul>
            <li
              v-for="(line, i) in props.request.snapshot.partie2_sommes_dues_par_syndicat"
              :key="i"
              class="flex justify-between"
            >
              <span>{{ line.label }}</span>
              <span class="data-md">{{ formatCurrency(line.amount) }}</span>
            </li>
          </ul>
        </section>

        <section>
          <h3 class="title-md font-semibold mb-2">
            {{ t('registers.etatDate.partie3') }}
          </h3>
          <ul>
            <li
              v-for="(line, i) in props.request.snapshot.partie3_sommes_incombant_acquereur"
              :key="i"
              class="flex justify-between"
            >
              <span>{{ line.label }}</span>
              <span class="data-md">{{ formatCurrency(line.amount) }}</span>
            </li>
          </ul>
        </section>

        <section>
          <h3 class="title-md font-semibold mb-2">
            {{ t('registers.etatDate.annexeQuotePart') }}
          </h3>
          <p
            v-if="props.request.snapshot.annexe_quote_part.length === 0"
            class="data-sm opacity-70"
          >
            {{ t('registers.etatDate.none') }}
          </p>
          <ul v-else>
            <li
              v-for="ex in props.request.snapshot.annexe_quote_part"
              :key="ex.fiscal_year"
              class="flex justify-between"
            >
              <span>{{ ex.fiscal_year }}</span>
              <span class="data-md">{{ formatCurrency(ex.total_amount) }}</span>
            </li>
          </ul>
        </section>

        <section>
          <h3 class="title-md font-semibold mb-2">
            {{ t('registers.preEtatDate.procedures') }}
          </h3>
          <p
            v-if="props.request.snapshot.procedures.length === 0"
            class="data-sm opacity-70"
          >
            {{ t('registers.etatDate.none') }}
          </p>
          <ul v-else>
            <li
              v-for="(proc, i) in props.request.snapshot.procedures"
              :key="i"
            >
              {{ proc.subject }} — {{ formatDate(proc.opened_at) }}
            </li>
          </ul>
        </section>

        <section>
          <h3 class="title-md font-semibold mb-2">
            {{ t('registers.etatDate.emprunts') }}
          </h3>
          <p class="data-sm opacity-70">
            {{ props.request.snapshot.emprunts_collectifs }}
          </p>
        </section>
      </div>
      <p
        v-if="downloadError"
        class="tone--err data-sm"
      >
        {{ downloadError }}
      </p>
    </template>
    <template #footer>
      <CBtn
        variant="primary"
        :icon="CIconDownload"
        :loading="downloading"
        :title="t('registers.etatDate.downloadPdf')"
        :aria-label="t('registers.etatDate.downloadPdf')"
        @click="downloadPdf"
      />
    </template>
  </CModal>
</template>

