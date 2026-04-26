<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { vendorApi } from '@/api'
import type { VendorTask } from '@/types/domain'

const route = useRoute()
const { t } = useI18n()
const { formatDate } = useFormatters()

const token = route.params.token as string

// State
const loading = ref(true)
const error = ref('')
const task = ref<VendorTask | null>(null)
const uploading = ref(false)
const success = ref(false)
const comment = ref('')
const selectedFiles = ref<File[]>([])
const dragOver = ref(false)

const fileInputRef = ref<HTMLInputElement | null>(null)

async function loadTask() {
  loading.value = true
  error.value = ''
  try {
    const res = await vendorApi.getTask(token)
    task.value = res.data
  } catch {
    error.value = t('contractor.expired')
  } finally {
    loading.value = false
  }
}

onMounted(loadTask)

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  if (input.files) {
    selectedFiles.value = Array.from(input.files)
  }
}

function onDrop(e: DragEvent) {
  dragOver.value = false
  if (e.dataTransfer?.files) {
    selectedFiles.value = Array.from(e.dataTransfer.files)
  }
}

function removeFile(idx: number) {
  selectedFiles.value.splice(idx, 1)
}

async function submit() {
  if (!selectedFiles.value.length) {
    error.value = t('contractor.noFiles')
    return
  }
  uploading.value = true
  error.value = ''

  try {
    // 1. Upload each file to staging bucket
    const documents: { key: string; original_name: string; mime_type: string; size_bytes: number }[] = []

    for (const file of selectedFiles.value) {
      const ext = file.name.split('.').pop() ?? 'bin'
      const urlRes = await vendorApi.getUploadUrl(token, ext, file.type || 'application/octet-stream')
      const { url, key } = urlRes.data
      await vendorApi.uploadToStaging(file, url)
      documents.push({
        key,
        original_name: file.name,
        mime_type: file.type || 'application/octet-stream',
        size_bytes: file.size,
      })
    }

    // 2. Submit with all document keys + comment
    await vendorApi.submit(token, { comment: comment.value || undefined, documents })
    success.value = true
  } catch {
    error.value = t('contractor.uploadError')
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <div class="contractor-page">
    <header class="contractor-header">
      <CLogoMark class="contractor-logo" />
      <h1 class="display-sm">
        {{ t('contractor.title') }}
      </h1>
    </header>

    <main class="contractor-main">
      <!-- Loading -->
      <div
        v-if="loading"
        class="state-card"
      >
        <CIconSpinner class="spinner" />
        <p>{{ t('common.loading') }}</p>
      </div>

      <!-- Success -->
      <div
        v-else-if="success"
        class="state-card state-card--success"
      >
        <CIconCheckCircle
          class="state-icon"
          style="color:var(--color-success)"
        />
        <p class="body-md">
          {{ t('contractor.success') }}
        </p>
      </div>

      <!-- Error (expired / invalid token) -->
      <div
        v-else-if="error && !task"
        class="state-card state-card--error"
      >
        <CIconAlertTriangle
          class="state-icon"
          style="color:var(--color-error)"
        />
        <p class="body-md">
          {{ error }}
        </p>
      </div>

      <!-- Task form -->
      <div
        v-else-if="task"
        class="contractor-card"
      >
        <div class="task-info">
          <h2 class="title-md">
            {{ task.title }}
          </h2>
          <p
            v-if="task.description"
            class="body-sm"
            style="color:var(--color-on-surface-variant)"
          >
            {{ task.description }}
          </p>
          <p
            v-if="task.token_expires_at"
            class="body-xs"
            style="color:var(--color-on-surface-variant)"
          >
            {{ t('contractor.expiresAt', { date: formatDate(task.token_expires_at) }) }}
          </p>
        </div>

        <p class="body-md">
          {{ t('contractor.subtitle') }}
        </p>

        <!-- Dropzone -->
        <div
          :class="['dropzone', { 'dropzone--over': dragOver }]"
          @dragover.prevent="dragOver = true"
          @dragleave="dragOver = false"
          @drop.prevent="onDrop"
          @click="fileInputRef?.click()"
        >
          <CIconFile class="dropzone-icon" />
          <p class="body-sm">
            {{ t('contractor.dropzone') }}
          </p>
          <input
            ref="fileInputRef"
            type="file"
            multiple
            class="file-input-hidden"
            @change="onFileChange"
          />
        </div>

        <!-- Selected files -->
        <ul
          v-if="selectedFiles.length"
          class="file-list"
        >
          <li
            v-for="(file, idx) in selectedFiles"
            :key="idx"
            class="file-item"
          >
            <CIconFile
              width="14"
              height="14"
            />
            <span class="body-sm file-name">{{ file.name }}</span>
            <button
              class="icon-btn icon-btn--danger"
              @click="removeFile(idx)"
            >
              <CIconX
                width="14"
                height="14"
              />
            </button>
          </li>
        </ul>

        <!-- Comment -->
        <div class="form-field">
          <label class="form-label">{{ t('contractor.comment') }}</label>
          <textarea
            v-model="comment"
            class="form-input"
            rows="3"
            :placeholder="t('contractor.commentPlaceholder')"
          ></textarea>
        </div>

        <!-- Error inline -->
        <p
          v-if="error"
          class="error-msg body-sm"
        >
          {{ error }}
        </p>

        <!-- Submit -->
        <button
          class="submit-btn"
          :disabled="uploading || !selectedFiles.length"
          @click="submit"
        >
          <CIconSpinner
            v-if="uploading"
            class="spinner-sm"
          />
          {{ uploading ? t('contractor.submitting') : t('contractor.submit') }}
        </button>
      </div>
    </main>
  </div>
</template>

<style scoped>
.contractor-page {
  min-height: 100dvh;
  display: flex;
  flex-direction: column;
  background: var(--color-background);
}

.contractor-header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-5) var(--space-6);
  border-bottom: 1px solid var(--color-outline-variant);
  background: var(--color-surface);
}
.contractor-logo { width: 32px; height: 32px; }

.contractor-main {
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: var(--space-8) var(--space-4);
}

.state-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-8);
  text-align: center;
}
.state-icon { width: 48px; height: 48px; }

.contractor-card {
  width: 100%;
  max-width: 560px;
  background: var(--color-surface);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-lg);
  padding: var(--space-6);
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.task-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  padding-bottom: var(--space-4);
  border-bottom: 1px solid var(--color-outline-variant);
}

.dropzone {
  border: 2px dashed var(--color-outline);
  border-radius: var(--radius-md);
  padding: var(--space-6);
  text-align: center;
  cursor: pointer;
  transition: border-color var(--transition-fast), background var(--transition-fast);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
}
.dropzone:hover, .dropzone--over {
  border-color: var(--color-primary);
  background: var(--color-primary-container);
}
.dropzone-icon { width: 32px; height: 32px; color: var(--color-on-surface-variant); }
.file-input-hidden { display: none; }

.file-list { list-style: none; display: flex; flex-direction: column; gap: 6px; }
.file-item {
  display: flex; align-items: center; gap: 8px;
  padding: 6px 10px;
  background: var(--color-surface-container);
  border-radius: var(--radius-sm);
}
.file-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.form-field { display: flex; flex-direction: column; gap: var(--space-1); }
.form-label { font-size: var(--text-sm); font-weight: var(--fw-medium); color: var(--color-on-surface-variant); }
.form-input {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  background: var(--color-surface);
  color: var(--color-on-surface);
  resize: vertical;
}
.form-input:focus { outline: none; border-color: var(--color-primary); }

.error-msg { color: var(--color-error); }

.submit-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5);
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-radius: var(--radius-md);
  font-weight: var(--fw-semibold);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: opacity var(--transition-fast);
}
.submit-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.spinner { width: 24px; height: 24px; animation: spin 1s linear infinite; }
.spinner-sm { width: 16px; height: 16px; animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
