<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import CModal from '@/components/molecules/CModal.vue'
import CForm from '@/components/molecules/CForm.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CFormTextarea from '@/components/atoms/CFormTextarea.vue'
import CFormSelect from '@/components/atoms/CFormSelect.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CPageLoader from '@/components/molecules/CPageLoader.vue'
import CIconTrash from '@/components/atoms/CIconTrash.svg'
import CDocumentLinkActionModal from '@/components/organisms/CDocumentLinkActionModal.vue'
import { documentsApi, reportsApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import { toErrorMessage } from '@/composables/usePageLoader'
import type { Document, Report } from '@/types/domain'
import { DEFAULT_DOCUMENT_VISIBLE_ROLES } from '@/utils/constants'
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  open: boolean
  report: Report | null
  unitOptions: { value: string; label: string }[]
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
  'delete': []
}>()

const { t } = useI18n()
const ui = useUiStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const saving = ref(false)
const uploadingPhotos = ref(false)

const form = ref({ title: '', description: '', unit_id: '' })
const selectedPhotos = ref<File[]>([])

const attachedPhotos = ref<{ doc: Document; url: string }[]>([])
const loadingAttachedPhotos = ref(false)

// Delete photo action modal
const deletingPhoto = ref(false)
const photoActionTarget = ref<{ doc: Document } | null>(null)
const photoActionFeedback = useFormFeedback()
const showPhotoActionModal = ref(false)

watch(() => props.open, async (open) => {
  if (open && props.report) {
    form.value = {
      title: props.report.title,
      description: props.report.description ?? '',
      unit_id: props.report.unit_id ?? '',
    }
    selectedPhotos.value = []
    feedback.reset()
    await loadAttachedPhotos(props.report.id)
  } else if (!open) {
    forceClose()
  }
})

async function loadAttachedPhotos(reportId: string) {
  loadingAttachedPhotos.value = true
  try {
    const docsRes = await reportsApi.listDocuments(reportId)
    attachedPhotos.value = await Promise.all(
      docsRes.data.map(async (document) => {
        const urlRes = await documentsApi.getDownloadUrl(document.id)
        return { doc: document, url: urlRes.data.url }
      }),
    )
  } catch {
    attachedPhotos.value = []
  } finally {
    loadingAttachedPhotos.value = false
  }
}

function photoPreview(file: File): string {
  return URL.createObjectURL(file)
}

async function uploadPhoto(reportId: string, file: File): Promise<void> {
  const ext = file.name.split('.').pop() ?? 'jpg'
  const mimeType = file.type || 'image/jpeg'
  const urlRes = await documentsApi.getUploadUrl(ext, mimeType, file.size)
  const { url, key, bucket } = urlRes.data
  await documentsApi.uploadToS3(file, url, mimeType)
  const docRes = await documentsApi.confirmUpload({
    key,
    bucket,
    original_name: file.name,
    mime_type: mimeType,
    size_bytes: file.size,
    visible_roles: [...DEFAULT_DOCUMENT_VISIBLE_ROLES],
  })
  await reportsApi.attachDocument(reportId, docRes.data.id)
}

function openPhotoAction(doc: Document) {
  photoActionTarget.value = { doc }
  photoActionFeedback.reset()
  showPhotoActionModal.value = true
}

function closePhotoAction() {
  if (deletingPhoto.value) return
  photoActionTarget.value = null
  photoActionFeedback.reset()
  showPhotoActionModal.value = false
}

async function handlePhotoAction(deleteDocument: boolean) {
  if (!photoActionTarget.value || !props.report || deletingPhoto.value) return
  deletingPhoto.value = true
  photoActionFeedback.reset()
  try {
    await reportsApi.detachDocument(props.report.id, photoActionTarget.value.doc.id)
    if (deleteDocument) {
      await documentsApi.delete(photoActionTarget.value.doc.id)
    }
    attachedPhotos.value = attachedPhotos.value.filter(item => item.doc.id !== photoActionTarget.value?.doc.id)
    photoActionTarget.value = null
    showPhotoActionModal.value = false
  } catch {
    photoActionFeedback.setError(t('common.error'))
  } finally {
    deletingPhoto.value = false
  }
}

async function submit() {
  if (feedback.status.value === 'success') { forceClose(); return }
  if (!props.report) return
  if (!form.value.title.trim()) return
  saving.value = true
  feedback.reset()
  try {
    await reportsApi.update(props.report.id, {
      title: form.value.title.trim(),
      description: form.value.description.trim() || null,
      unit_id: form.value.unit_id || null,
    })

    if (selectedPhotos.value.length) {
      uploadingPhotos.value = true
      const photoErrors: string[] = []
      for (const photo of selectedPhotos.value) {
        try { await uploadPhoto(props.report.id, photo) } catch { photoErrors.push(photo.name) }
      }
      uploadingPhotos.value = false
      if (photoErrors.length) {
        feedback.setError(t('reports.photoUploadError') + ': ' + photoErrors.join(', '))
        return
      }
    }

    feedback.setSuccess(t('reports.editSuccess'))
    emit('saved')
    scheduleClose(() => { emit('update:open', false) })
  } catch (err) {
    feedback.setError(toErrorMessage(err))
  } finally {
    saving.value = false
    uploadingPhotos.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('common.edit')"
    size="md"
    :closable="!saving && !uploadingPhotos"
    @update:open="emit('update:open', $event)"
  >
    <CForm
      id="edit-report"
      @submit="submit"
    >
      <CFormInput
        v-model="form.title"
        :label="t('common.title')"
        :placeholder="t('common.title')"
        required
      />
      <CFormTextarea
        v-model="form.description"
        :label="t('common.description')"
        :rows="3"
      />
      <CFormSelect
        v-model="form.unit_id"
        :label="t('claims.fields.unit')"
        :options="unitOptions"
      />

      <!-- Add new photos -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('reports.addPhotos') }}</label>
        <CFileDropzone
          v-model:files="selectedPhotos"
          :hint="t('reports.photoHint')"
          accept="image/*"
        >
          <template #preview="{ files, remove }">
            <div class="photo-preview-strip">
              <div
                v-for="(photo, idx) in files"
                :key="idx"
                class="photo-preview-item"
              >
                <img
                  :src="photoPreview(photo)"
                  :alt="photo.name"
                  class="photo-thumb"
                />
                <button
                  type="button"
                  class="photo-remove"
                  :title="t('common.delete')"
                  @click="remove(idx)"
                >
                  ×
                </button>
              </div>
            </div>
          </template>
        </CFileDropzone>
        <span
          v-if="uploadingPhotos"
          class="body-xs"
          style="color:var(--color-on-surface-variant)"
        >{{ t('reports.photoUploading') }}</span>
      </div>

      <!-- Existing attached photos -->
      <div class="flex flex-col gap-1">
        <label class="text-sm font-medium text-on-surface-variant">{{ t('reports.attachedPhotos') }}</label>
        <CPageLoader v-if="loadingAttachedPhotos" />
        <div
          v-else-if="attachedPhotos.length"
          class="photo-gallery"
        >
          <div
            v-for="item in attachedPhotos"
            :key="item.doc.id"
            class="photo-gallery-item"
          >
            <img
              :src="item.url"
              :alt="item.doc.original_name"
              class="photo-gallery-thumb"
            />
            <button
              type="button"
              class="photo-gallery-delete"
              :title="t('reports.deletePhoto')"
              :disabled="deletingPhoto"
              @click="openPhotoAction(item.doc)"
            >
              <CIconTrash />
            </button>
          </div>
        </div>
        <span
          v-else
          class="body-xs"
          style="color:var(--color-on-surface-variant)"
        >{{ t('reports.noAttachedPhotos') }}</span>
      </div>
    </CForm>
    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          v-if="report"
          variant="danger"
          :icon="CIconTrash"
          :title="t('common.delete')"
          @click="emit('delete')"
        />
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="saving || uploadingPhotos"
          :closing="feedback.status.value === 'success'"
          :closing-duration-ms="ui.autoCloseDelayMs"
          type="submit"
          form="edit-report"
        />
      </CFormActions>
    </template>
  </CModal>

  <CDocumentLinkActionModal
    :open="showPhotoActionModal"
    :title="t('reports.deletePhoto')"
    :description="t('reports.deletePhotoDetachMessage', { name: photoActionTarget?.doc.original_name ?? '', title: report?.title ?? '' })"
    :warning="t('reports.deletePhotoConfirm')"
    :loading="deletingPhoto"
    :status="photoActionFeedback.status.value"
    :message="photoActionFeedback.message.value"
    @update:open="value => !value && closePhotoAction()"
    @unlink="handlePhotoAction(false)"
    @unlink-and-delete="handlePhotoAction(true)"
  />
</template>

<style scoped>
.photo-preview-strip {
  display: flex; flex-wrap: wrap; gap: var(--space-2); margin-top: var(--space-2);
}
.photo-preview-item {
  position: relative; width: var(--space-16); height: var(--space-16); flex-shrink: 0;
}
.photo-thumb {
  width: var(--space-16); height: var(--space-16); object-fit: cover;
  border-radius: var(--radius-lg); border: 1px solid var(--color-outline-variant);
}
.photo-remove {
  position: absolute; top: -6px; right: -6px;
  background: var(--color-error); color: var(--color-on-error); border: none;
  border-radius: var(--radius-full); width: var(--space-5); height: var(--space-5);
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; font-size: 14px; line-height: 1;
  min-height: unset; min-width: unset;
}
.photo-gallery { display: flex; flex-wrap: wrap; gap: var(--space-2); }
.photo-gallery-item { position: relative; }
.photo-gallery-thumb {
  width: 120px; height: 120px; object-fit: cover; display: block;
  border-radius: var(--radius-lg); border: 1px solid var(--color-outline-variant);
  cursor: pointer; transition: opacity 0.15s;
}
.photo-gallery-thumb:hover { opacity: 0.85; }
.photo-gallery-delete {
  position: absolute; top: -6px; right: -6px;
  background: var(--color-error); color: var(--color-on-error); border: none;
  border-radius: var(--radius-full); width: 22px; height: 22px;
  display: flex; align-items: center; justify-content: center;
  cursor: pointer; padding: 3px;
  min-height: unset; min-width: unset;
  opacity: 0.9; transition: opacity 0.15s;
}
.photo-gallery-delete svg { width: var(--icon-xs); height: var(--icon-xs); }
.photo-gallery-delete:hover { opacity: 1; }
</style>
