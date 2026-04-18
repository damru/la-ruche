<script setup lang="ts">
// Molecule component — CDropzone
// Upload direct vers S3 via pre-signed URL — DDR architecture

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
  uploading: false,
})

const emit = defineEmits<{
  files: [files: File[]]
  error: [message: string]
}>()

const isDragging = ref(false)
const inputRef = ref<HTMLInputElement>()

function handleFiles(fileList: FileList | null) {
  if (!fileList?.length) return
  const files = Array.from(fileList)
  const oversized = files.filter(f => f.size > props.maxSizeMb * 1024 * 1024)
  if (oversized.length) {
    emit('error', `Fichier trop volumineux (max ${props.maxSizeMb} Mo)`)
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
    @drop.prevent="onDrop"
    @dragover="onDragOver"
    @dragleave="isDragging = false"
    @click="inputRef?.click()"
    @keydown.enter="inputRef?.click()"
    role="button"
    tabindex="0"
    :aria-label="label ?? 'Zone de dépôt de fichier'"
  >
    <input
      ref="inputRef"
      type="file"
      :accept="accept"
      :multiple="multiple"
      class="c-dropzone__input"
      @change="handleFiles(($event.target as HTMLInputElement).files)"
      tabindex="-1"
    />

    <template v-if="!uploading">
      <svg class="c-dropzone__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
        <polyline points="17 8 12 3 7 8"/>
        <line x1="12" y1="3" x2="12" y2="15"/>
      </svg>
      <p class="c-dropzone__text">
        {{ label ?? 'Glissez un fichier ici' }}
      </p>
      <p class="c-dropzone__hint">ou cliquez pour parcourir · max {{ maxSizeMb }} Mo</p>
    </template>

    <template v-else>
      <svg class="c-dropzone__icon c-dropzone__icon--spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
        <path d="M12 2v4m0 12v4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M2 12h4m12 0h4"/>
      </svg>
      <p class="c-dropzone__text">Envoi en cours…</p>
      <div v-if="progress !== undefined" class="c-dropzone__progress-wrap">
        <div class="c-dropzone__progress" :style="{ width: `${progress}%` }" />
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
  font-size: 14px;
  font-weight: 500;
  color: var(--color-on-surface);
}

.c-dropzone__hint {
  font-size: 12px;
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
