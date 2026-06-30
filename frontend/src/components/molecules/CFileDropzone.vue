<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import CIconUpload from '@/components/atoms/CIconUpload.svg'

const props = withDefaults(defineProps<{
  files: File[]
  hint?: string
  removeLabel?: string
  multiple?: boolean
  accept?: string
}>(), {
  hint: '',
  removeLabel: '',
  multiple: true,
  accept: '',
})

const emit = defineEmits<{
  'update:files': [files: File[]]
}>()

const { t } = useI18n()
const inputRef = ref<HTMLInputElement | null>(null)
const dragOver = ref(false)

function addFiles(list: FileList | null | undefined) {
  const incoming = Array.from(list ?? [])
  if (!incoming.length) return
  emit('update:files', props.multiple ? [...props.files, ...incoming] : incoming.slice(0, 1))
}

function onChange(event: Event) {
  const input = event.target as HTMLInputElement
  addFiles(input.files)
  if (inputRef.value) inputRef.value.value = ''
}

function onDrop(event: DragEvent) {
  dragOver.value = false
  addFiles(event.dataTransfer?.files)
}

function remove(index: number) {
  emit('update:files', props.files.filter((_, i) => i !== index))
}

function openPicker() {
  inputRef.value?.click()
}
</script>

<template>
  <div>
    <input
      ref="inputRef"
      type="file"
      :multiple="multiple"
      :accept="accept || undefined"
      style="display:none"
      @change="onChange"
    />
    <div
      :class="['dropzone', { 'dropzone--over': dragOver }]"
      role="button"
      tabindex="0"
      @click="openPicker"
      @keydown.enter.prevent="openPicker"
      @keydown.space.prevent="openPicker"
      @dragenter.prevent="dragOver = true"
      @dragover.prevent="dragOver = true"
      @dragleave.prevent="dragOver = false"
      @drop.prevent="onDrop"
    >
      <CIconUpload class="dropzone__icon" />
      <span class="body-xs dropzone__hint">{{ hint }}</span>
    </div>

    <slot
      v-if="files.length"
      name="preview"
      :files="files"
      :remove="remove"
    >
      <div class="c-file-dropzone__strip">
        <div
          v-for="(file, index) in files"
          :key="`${file.name}-${index}`"
          class="c-file-dropzone__item"
        >
          <span class="c-file-dropzone__name">{{ file.name }}</span>
          <button
            type="button"
            class="c-file-dropzone__remove"
            :title="removeLabel || t('common.delete')"
            :aria-label="removeLabel || t('common.delete')"
            @click="remove(index)"
          >
            ×
          </button>
        </div>
      </div>
    </slot>
  </div>
</template>

<style scoped>
.c-file-dropzone__strip {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-2);
}
.c-file-dropzone__item {
  position: relative;
  display: inline-flex;
  align-items: center;
  min-height: var(--control-height);
  max-width: 100%;
  padding: 0 var(--space-4) 0 var(--space-3);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-outline-variant);
  background: var(--color-surface-container-low);
}
.c-file-dropzone__name {
  max-width: 220px;
  overflow: clip;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-on-surface-variant);
  font-size: var(--text-sm);
}
.c-file-dropzone__remove {
  position: absolute;
  top: -6px;
  right: -6px;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-full);
  background: var(--color-error);
  color: var(--color-on-error);
  border: none;
  cursor: pointer;
  font-size: var(--text-md);
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}
.c-file-dropzone__remove:hover {
  opacity: 0.85;
}
</style>
