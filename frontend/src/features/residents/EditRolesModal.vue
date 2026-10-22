<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import CModal from '@/components/molecules/CModal.vue'
import CFormActions from '@/components/molecules/CFormActions.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import CBadge from '@/components/atoms/CBadge.vue'
import { usersApi } from '@/api'
import { useFormFeedback } from '@/composables/useFormFeedback'
import { useAutoClose } from '@/composables/useAutoClose'
import type { User } from '@/types/domain'

const ROLES = ['manager', 'board_member', 'co_owner', 'tenant'] as const
type RoleKey = (typeof ROLES)[number]

const props = defineProps<{
  open: boolean
  user: User | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'saved': []
}>()

const { t } = useI18n()
const auth = useAuthStore()
const feedback = useFormFeedback()
const { schedule: scheduleClose, forceExpire: forceClose } = useAutoClose()
const loading = ref(false)
const selected = ref<RoleKey[]>([])

const isEditingSelf = computed(() => props.user?.id === auth.user?.user_id)
const removingOwnManager = computed(
  () => isEditingSelf.value && !selected.value.includes('manager'),
)

watch(() => props.open, (open) => {
  if (!open) { forceClose(); return }
  selected.value = (props.user?.roles ?? []) as RoleKey[]
  feedback.reset()
})

function toggleRole(role: RoleKey) {
  if (selected.value.includes(role)) {
    selected.value = selected.value.filter(r => r !== role)
  } else {
    selected.value = [...selected.value, role]
  }
}

async function handleSave() {
  if (!props.user) return
  loading.value = true
  feedback.reset()
  try {
    await usersApi.updateRoles(props.user.id, selected.value)
    feedback.setSuccess(t('residents.editRolesSuccess'))
    emit('saved')
    scheduleClose(() => { emit('update:open', false) })
  } catch {
    feedback.setError(t('common.error'))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <CModal
    :open="open"
    :title="t('common.editRoles')"
    size="sm"
    @update:open="$emit('update:open', $event)"
  >
    <div class="roles-grid">
      <label
        v-for="role in ROLES"
        :key="role"
        class="role-option"
      >
        <input
          type="checkbox"
          :checked="selected.includes(role)"
          :disabled="loading"
          @change="toggleRole(role)"
        />
        <CBadge :variant="role">
          {{ t(`roles.${role}`) }}
        </CBadge>
      </label>
    </div>
    <p
      v-if="removingOwnManager"
      class="warning-text body-sm"
    >
      ⚠ {{ t('residents.editRolesSelfWarning') }}
    </p>
    <template #footer>
      <CFormActions
        :status="feedback.status.value"
        :message="feedback.message.value"
      >
        <CBtn
          variant="ghost"
          :title="t('common.cancel')"
          @click="$emit('update:open', false)"
        />
        <CBtn
          variant="primary"
          :title="t('common.save')"
          :loading="loading"
          :closing="feedback.status.value === 'success'"
          @click="handleSave"
        />
      </CFormActions>
    </template>
  </CModal>
</template>

<style scoped>
.roles-grid {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.role-option {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
}

.role-option input[type="checkbox"] {
  cursor: pointer;
}

.warning-text {
  color: var(--color-warning);
  margin-top: var(--space-2);
}
</style>
