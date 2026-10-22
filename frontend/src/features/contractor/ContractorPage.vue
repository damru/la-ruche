<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useFormatters } from '@/composables/useFormatters'
import { toErrorMessage } from '@/composables/usePageLoader'
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
const cost = ref('')
const selectedFiles = ref<File[]>([])

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

async function submit() {
  uploading.value = true
  error.value = ''

  try {
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
    await vendorApi.submit(token, {
      comment: comment.value || undefined,
      cost: cost.value ? Number(cost.value) : undefined,
      documents,
    })
    success.value = true
  } catch (e: unknown) {
    error.value = toErrorMessage(e, 'contractor.uploadError')
  } finally {
    uploading.value = false
  }
}
</script>

<template>
  <div class="contractor-page">
    <header class="contractor-header">
      <div class="contractor-header-brand">
        <CLogoMark class="contractor-logo" />
        <span class="brand-name body-md">La Ruche</span>
      </div>
      <div
        v-if="task"
        class="contractor-header-building"
      >
        <span class="title-sm">{{ task.building_name }}</span>
        <span class="body-xs building-address">{{ task.building_address }}</span>
      </div>
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
        <CIconXCircle
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
        class="contractor-content"
      >
        <div class="contractor-card">
          <div class="task-info">
            <CHeading>
              {{ task.title }}
            </CHeading>
            <p
              v-if="task.description"
              class="body-xs text-on-surface-variant"
            >
              {{ task.description }}
            </p>
            <p
              v-if="task.token_expires_at"
              class="body-xs text-on-surface-variant"
            >
              {{ t('contractor.expiresAt', { date: formatDate(task.token_expires_at) }) }}
            </p>
          </div>

          <p class="body-md">
            {{ t('contractor.subtitle') }}
          </p>

          <!-- Dropzone + selected files -->
          <CFileDropzone
            v-model:files="selectedFiles"
            :hint="t('contractor.dropzone')"
          />

          <CFormTextarea
            v-model="comment"
            :label="t('contractor.comment')"
            :rows="3"
            :placeholder="t('contractor.commentPlaceholder')"
          />
          <CFormInput
            v-model="cost"
            :label="t('tasks.completionCostLabel')"
            type="number"
            :min="0"
            :step="0.01"
            :placeholder="t('common.optional')"
          />

          <!-- Error inline -->
          <p
            v-if="error"
            class="error-msg body-xs"
          >
            {{ error }}
          </p>

          <!-- Submit -->
          <CBtn
            variant="primary"
            size="lg"
            class="w-full"
            :loading="uploading"
            :disabled="uploading"
            :title="t('contractor.submit')"
            @click="submit"
          />
        </div>

        <!-- Contact card -->
        <div class="contact-card">
          <h3 class="body-md font-semibold text-on-surface">
            {{ t('contractor.contactTitle') }}
          </h3>
          <div class="contact-row">
            <span class="contact-label body-xs">{{ t('contractor.contactEmail') }}</span>
            <a
              :href="`mailto:${task.manager_email}`"
              class="body-xs contact-value"
            >{{ task.manager_email }}</a>
          </div>
          <div
            v-if="task.manager_phone"
            class="contact-row"
          >
            <span class="contact-label body-xs">{{ t('common.phone') }}</span>
            <a
              :href="`tel:${task.manager_phone}`"
              class="body-xs contact-value"
            >{{ task.manager_phone }}</a>
          </div>
          <div class="contact-row">
            <span class="contact-label body-xs">{{ t('common.address') }}</span>
            <span class="body-xs contact-value">{{ task.building_address }}</span>
          </div>
        </div>
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
  justify-content: space-between;
  padding: var(--space-4) var(--space-6);
  border-bottom: 1px solid var(--color-outline-variant);
  background: var(--color-surface);
  gap: var(--space-4);
}
.contractor-header-brand {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}
.contractor-logo { width: 28px; height: 28px; }
.brand-name { font-weight: var(--fw-semibold); color: var(--color-on-surface); }

.contractor-header-building {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  text-align: right;
}
.building-address { color: var(--color-on-surface-variant); }

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

.contractor-content {
  width: 100%;
  max-width: 560px;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.contractor-card {
  background: var(--color-surface);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-2xl);
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

.error-msg { color: var(--color-error); }

/* Contact card */
.contact-card {
  background: var(--color-surface);
  border: 1px solid var(--color-outline-variant);
  border-radius: var(--radius-2xl);
  padding: var(--space-4) var(--space-6);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.contact-row { display: flex; gap: var(--space-3); align-items: baseline; }
.contact-label { color: var(--color-on-surface-variant); min-width: 60px; flex-shrink: 0; }
.contact-value { color: var(--color-on-surface); }
a.contact-value { color: var(--color-primary); text-decoration: none; }
a.contact-value:hover { text-decoration: underline; }

.spinner { width: var(--icon-lg); height: var(--icon-lg); animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
