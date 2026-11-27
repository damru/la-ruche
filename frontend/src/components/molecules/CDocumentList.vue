<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import CBtn from '@/components/atoms/CBtn.vue'
import CIconDownload from '@/components/atoms/CIconDownload.svg'
import CIconFile from '@/components/atoms/CIconFile.svg'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import { useFormatters } from '@/composables/useFormatters'
import type { Document } from '@/types/domain'

withDefaults(defineProps<{
  documents: Document[]
  readonly?: boolean
  loadingDocId?: string | null
  removeTitle?: string
}>(), {
  readonly: false,
  loadingDocId: null,
  removeTitle: undefined,
})

const emit = defineEmits<{
  remove: [doc: Document]
  download: [doc: Document]
}>()

const { t } = useI18n()
const { formatDate, formatFileSize } = useFormatters()
</script>

<template>
  <ul class="flex flex-col gap-2 m-0 p-0 list-none">
    <li
      v-for="document in documents"
      :key="document.id"
      class="flex items-center justify-between gap-3"
    >
      <div class="flex flex-col min-w-0">
        <span class="flex items-center gap-2 min-w-0">
          <CIconFile
            class="icon-sm shrink-0"
            aria-hidden="true"
          />
          <span class="truncate">{{ document.original_name }}</span>
        </span>
        <span class="text-xs text-on-surface-variant">
          {{ formatFileSize(document.size_bytes) }} · {{ formatDate(document.created_at) }}
        </span>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <CBtn
          variant="ghost"
          size="sm"
          :icon="CIconDownload"
          :title="t('common.download')"
          :aria-label="`${t('common.download')} ${document.original_name}`"
          @click="emit('download', document)"
        />
        <CBtn
          v-if="!readonly"
          variant="danger"
          size="sm"
          :icon="CIconTrash"
          :title="removeTitle ?? t('common.remove')"
          :aria-label="`${removeTitle ?? t('common.remove')} ${document.original_name}`"
          :loading="loadingDocId === document.id"
          @click="emit('remove', document)"
        />
      </div>
    </li>
  </ul>
</template>
