<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CIconArrowLeft from '@/components/atoms/CIconArrowLeft.svg'
import CIconArrowRight from '@/components/atoms/CIconArrowRight.svg'
import { documentsApi, reportsApi } from '@/api'
import type { Document, Report } from '@/types/domain'

const props = defineProps<{
  open: boolean
  report: Report | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const { t } = useI18n()
const photos = ref<{ doc: Document; url: string }[]>([])
const photosLoading = ref(false)
const activeIndex = ref<number | null>(null)

const activePhoto = computed(() =>
  activeIndex.value === null ? null : photos.value[activeIndex.value] ?? null,
)

const activePhotoPosition = computed(() =>
  activeIndex.value === null ? 0 : activeIndex.value + 1,
)

watch(() => props.open, async (open) => {
  if (open && props.report) {
    photos.value = []
    photosLoading.value = true
    activeIndex.value = null
    try {
      const docsRes = await reportsApi.listDocuments(props.report.id)
      photos.value = await Promise.all(
        docsRes.data.map(async (document) => {
          const urlRes = await documentsApi.getDownloadUrl(document.id)
          return { doc: document, url: urlRes.data.url }
        }),
      )
    } catch {
      // non-blocking — empty state shown
    } finally {
      photosLoading.value = false
    }
  } else if (!open) {
    photos.value = []
    activeIndex.value = null
  }
})

function openViewer(index: number) {
  activeIndex.value = index
}

function closeViewer() {
  activeIndex.value = null
}

function showPrevious() {
  if (activeIndex.value === null || photos.value.length <= 1) return
  activeIndex.value = (activeIndex.value - 1 + photos.value.length) % photos.value.length
}

function showNext() {
  if (activeIndex.value === null || photos.value.length <= 1) return
  activeIndex.value = (activeIndex.value + 1) % photos.value.length
}
</script>

<template>
  <!-- Gallery modal -->
  <CModal
    :open="open"
    :title="report?.title ?? ''"
    size="md"
    @update:open="emit('update:open', $event)"
  >
    <CPageLoader v-if="photosLoading" />
    <div
      v-else-if="photos.length"
      class="report-photo-gallery-modal"
    >
      <button
        v-for="(item, index) in photos"
        :key="item.doc.id"
        type="button"
        class="report-photo-gallery-modal__item"
        @click="openViewer(index)"
      >
        <img
          :src="item.url"
          :alt="item.doc.original_name"
          class="report-photo-gallery-modal__thumb"
        />
      </button>
    </div>
    <p
      v-else
      class="body-xs report-photo-gallery-modal__empty"
    >
      {{ t('reports.noPhotos') }}
    </p>
  </CModal>

  <!-- Photo viewer modal (stacks on top of gallery) -->
  <CModal
    :open="activeIndex !== null"
    :title="activePhoto?.doc.original_name ?? report?.title ?? ''"
    size="xl"
    @update:open="value => !value && closeViewer()"
  >
    <div
      v-if="activePhoto"
      class="report-photo-viewer"
    >
      <img
        :src="activePhoto.url"
        :alt="activePhoto.doc.original_name"
        class="report-photo-viewer__image"
      />
      <div class="report-photo-viewer__nav-row">
        <CBtn
          variant="ghost"
          size="sm"
          class="report-photo-viewer__nav"
          :icon="CIconArrowLeft"
          :disabled="photos.length <= 1"
          @click="showPrevious"
        />
        <span class="body-xs report-photo-viewer__caption">
          {{ t('reports.photoPosition', { current: activePhotoPosition, total: photos.length }) }}
        </span>
        <CBtn
          variant="ghost"
          size="sm"
          class="report-photo-viewer__nav"
          :icon="CIconArrowRight"
          :disabled="photos.length <= 1"
          @click="showNext"
        />
      </div>
    </div>
  </CModal>
</template>

<style scoped>
.report-photo-gallery-modal {
  --report-photo-thumb-size: 120px;
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}
.report-photo-gallery-modal__item {
  border: none; background: none; padding: 0; cursor: pointer;
  max-width: var(--report-photo-thumb-size);
  flex: 0 0 var(--report-photo-thumb-size);
}
.report-photo-gallery-modal__thumb {
  width: var(--report-photo-thumb-size); height: var(--report-photo-thumb-size);
  object-fit: cover; border-radius: var(--radius-lg);
  border: 1px solid var(--color-outline-variant); transition: opacity 0.15s;
}
.report-photo-gallery-modal__thumb:hover { opacity: 0.85; }
.report-photo-gallery-modal__empty { margin: 0; color: var(--color-on-surface-variant); }

.report-photo-viewer {
  --report-photo-viewer-height: min(78vh, 760px);
  display: flex; flex-direction: column;
  gap: var(--space-3); align-items: center;
}
.report-photo-viewer__image {
  width: 100%; max-height: var(--report-photo-viewer-height);
  object-fit: contain; border-radius: var(--radius-xl);
  background: var(--color-surface-container-low);
}
.report-photo-viewer__nav-row {
  display: flex; align-items: center; justify-content: center; gap: var(--space-3);
}
.report-photo-viewer__nav { align-self: center; }
.report-photo-viewer__caption {
  margin: var(--space-3) 0 0; text-align: center; color: var(--color-on-surface-variant);
}

@media (max-width: 720px) {
  .report-photo-gallery-modal { justify-content: flex-start; }
}
</style>
