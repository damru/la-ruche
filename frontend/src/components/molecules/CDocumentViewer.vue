<script setup lang="ts">
// Molecule component — CDocumentViewer
// In-app PDF rendering via the short-TTL presigned URL with inline
// disposition. Never exposes the storage bucket: it consumes the same
// authorized download path, only the Content-Disposition differs.
// The download button always remains available as a fallback.

import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CAlert from '@/components/atoms/CAlert.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconSpinner from '@/components/atoms/CIconSpinner.svg'
import CModal from '@/components/molecules/CModal.vue'
import { documentsApi } from '@/api'

const props = withDefaults(defineProps<{
  open: boolean
  documentId: string | null
  documentName?: string
}>(), {
  documentName: undefined,
})

const emit = defineEmits<{ 'update:open': [value: boolean] }>()

const { t } = useI18n()

const inlineUrl = ref<string | null>(null)
const downloadUrl = ref<string | null>(null)
const loading = ref(false)
const errored = ref(false)

const title = computed(() => props.documentName ?? t('documents.viewer.title'))

watch(
  () => [props.open, props.documentId] as const,
  async ([open, id]) => {
    inlineUrl.value = null
    downloadUrl.value = null
    errored.value = false
    if (!open || !id) return
    loading.value = true
    try {
      const [inlineRes, downloadRes] = await Promise.all([
        documentsApi.getDownloadUrl(id, 'inline'),
        documentsApi.getDownloadUrl(id),
      ])
      inlineUrl.value = inlineRes.data.url
      downloadUrl.value = downloadRes.data.url
    } catch {
      errored.value = true
    } finally {
      loading.value = false
    }
  },
  { immediate: true },
)

function download() {
  if (downloadUrl.value) window.open(downloadUrl.value, '_blank', 'noopener')
}

function onFrameError() {
  // Browser without an integrated PDF renderer: fall back to download.
  errored.value = true
}
</script>

<template>
  <CModal
    :open="open"
    :title="title"
    size="xl"
    @update:open="emit('update:open', $event)"
  >
    <div class="flex h-[70vh] flex-col gap-3">
      <div
        v-if="loading"
        class="flex flex-1 items-center justify-center gap-2 text-on-surface-variant"
      >
        <CIconSpinner class="spin icon-md" />
        <span class="text-sm">{{ t('common.loading') }}</span>
      </div>
      <template v-else-if="inlineUrl && !errored">
        <iframe
          :src="inlineUrl"
          class="min-h-0 flex-1 rounded-xl border border-outline-variant bg-surface"
          :title="title"
          @error="onFrameError"
        ></iframe>
      </template>
      <CAlert
        v-else
        variant="warning"
        :title="t('documents.viewer.loadError')"
      >
        {{ t('documents.viewer.fallback') }}
      </CAlert>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <CBtn
          variant="secondary"
          :icon="CIconDownload"
          :disabled="!downloadUrl"
          @click="download"
        >
          {{ t('common.download') }}
        </CBtn>
        <CBtn
          variant="ghost"
          @click="emit('update:open', false)"
        >
          {{ t('common.close') }}
        </CBtn>
      </div>
    </template>
  </CModal>
</template>
