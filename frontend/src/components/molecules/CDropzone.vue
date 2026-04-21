<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(defineProps<{
  accept?: string
  maxSizeMb?: number
  multiple?: boolean
  label?: string
  uploading?: boolean
  progress?: number      // 0-100 si connu
}>(), {
  accept: '*/*',
  maxSizeMb: 50,
  multiple: false,
  label: undefined,
  uploading: false,
  progress: undefined,
})

const emit = defineEmits<{
  files: [files: File[]]
  error: [message: string]
}>()

const { t } = useI18n()
const isDragging = ref(false)
const inputRef = ref<HTMLInputElement>()

const resolvedAriaLabel = computed(() => props.label ?? t('common.fileDropzoneAria'))
const resolvedLabel = computed(() => props.label ?? t('common.fileDropzoneText'))
const resolvedHint = computed(() => t('common.fileDropzoneHint', { size: props.maxSizeMb }))

function handleFiles(fileList: FileList | null) {
  if (!fileList?.length) return
  const files = Array.from(fileList)
  const oversized = files.filter(f => f.size > props.maxSizeMb * 1024 * 1024)
  if (oversized.length) {
    emit('error', t('documents.maxSizeExceeded', { size: props.maxSizeMb }))
    return
  }
  emit('files', files)
}

function onDrop(e: DragEvent) {
  isDragging.value = false
  handleFiles(e.dataTransfer?.files ?? null)
}

function onDragOver(e: DragEvent) {
  e.preventDefault()
  isDragging.value = true
}
</script>

<template>
  <div
    class="c-dropzone"
    :class="{ 'c-dropzone--dragging': isDragging, 'c-dropzone--uploading': uploading }"
    role="button"
    tabindex="0"
    :aria-label="resolvedAriaLabel"
    @drop.prevent="onDrop"
    @dragover="onDragOver"
    @dragleave="isDragging = false"
    @click="inputRef?.click()"
    @keydown.enter="inputRef?.click()"
    @keydown.space.prevent="inputRef?.click()"
  >
    <input
      ref="inputRef"
      type="file"
      :accept="accept"
      :multiple="multiple"
      class="c-dropzone__input"
      tabindex="-1"
      @change="handleFiles(($event.target as HTMLInputElement).files)"
    />

    <template v-if="!uploading">
      <CIconUpload
        class="c-dropzone__icon"
        aria-hidden="true"
      />
      <p class="c-dropzone__text">
        {{ resolvedLabel }}
      </p>
      <p class="c-dropzone__hint">
        {{ resolvedHint }}
      </p>
    </template>

    <template v-else>
      <CIconSpinner
        class="c-dropzone__icon c-dropzone__icon--spin"
        aria-hidden="true"
      />
      <p class="c-dropzone__text">
        {{ t('common.uploadInProgress') }}
      </p>
      <div
        v-if="progress !== undefined"
        class="c-dropzone__progress-wrap"
      >
        <div
          class="c-dropzone__progress"
          :style="{ width: `${progress}%` }"
        ></div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.c-dropzone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-10) var(--space-6);
  border-radius: var(--radius-lg);
  background: var(--color-surface-container-low);
  /* Ghost border — accessibility only */
  border: 2px dashed rgba(194, 199, 207, 0.5);
  cursor: pointer;
  transition:
    background var(--transition-fast),
    border-color var(--transition-fast);
  text-align: center;
  outline: none;
}

.c-dropzone:hover,
.c-dropzone:focus-visible {
  background: var(--color-secondary-container);
  border-color: rgba(100, 63, 213, 0.4);
}

.c-dropzone--dragging {
  background: var(--color-secondary-container);
  border-color: var(--color-primary);
  transform: scale(1.01);
}

.c-dropzone--uploading {
  cursor: not-allowed;
  pointer-events: none;
}

.c-dropzone__input {
  display: none;
}

.c-dropzone__icon {
  width: 36px;
  height: 36px;
  color: var(--color-primary);
  margin-bottom: var(--space-1);
}

.c-dropzone__icon--spin {
  animation: spin 1.2s linear infinite;
}

.c-dropzone__text {
  font-size: var(--text-md);
  font-weight: var(--fw-medium);
  color: var(--color-on-surface);
}

.c-dropzone__hint {
  font-size: var(--text-xs);
  color: var(--color-on-surface-variant);
}

.c-dropzone__progress-wrap {
  width: 160px;
  height: 4px;
  background: var(--color-surface-container-highest);
  border-radius: var(--radius-full);
  overflow: hidden;
  margin-top: var(--space-2);
}

.c-dropzone__progress {
  height: 100%;
  background: var(--gradient-primary);
  border-radius: var(--radius-full);
  transition: width 0.3s ease;
}
</style>
