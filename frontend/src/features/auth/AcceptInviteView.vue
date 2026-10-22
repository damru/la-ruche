<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import AuthBrandPanel from '@/components/organisms/AuthBrandPanel.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CBtn from '@/components/atoms/CBtn.vue'
import { authApi } from '@/api'
import { extractApiError } from '@/api/client'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()

const token = computed(() => route.params.token as string)

const loading = ref(false)
const validating = ref(true)
const done = ref(false)
const error = ref('')
const prefilledEmail = ref('')

const form = reactive({ first_name: '', last_name: '', password: '', confirm: '' })

const isValid = computed(() =>
  !!form.first_name && !!form.last_name && form.password.length >= 8 && form.password === form.confirm,
)
const mismatch = computed(() => !!form.confirm && form.password !== form.confirm)

onMounted(async () => {
  if (!token.value) {
    error.value = t('auth.invalidToken')
    validating.value = false
    return
  }
  try {
    const res = await authApi.validateInvite(token.value)
    prefilledEmail.value = res.data.email
    if (res.data.first_name) form.first_name = res.data.first_name
    if (res.data.last_name) form.last_name = res.data.last_name
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    if (status === 409) {
      error.value = t('auth.tokenExpiredOrUsed')
    } else if (status === 404) {
      error.value = t('auth.invalidToken')
    } else {
      error.value = t('auth.invalidToken')
    }
  } finally {
    validating.value = false
  }
})

async function handleSubmit() {
  if (!isValid.value || !token.value) return
  loading.value = true
  error.value = ''
  try {
    await authApi.acceptInvite(token.value, form.password, form.last_name, form.first_name)
    done.value = true
  } catch (e: unknown) {
    const status = (e as { response?: { status?: number } })?.response?.status
    if (status === 409) {
      error.value = t('auth.tokenExpiredOrUsed')
    } else {
      error.value = t(extractApiError(e))
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="auth-root">
    <AuthBrandPanel
      class="auth-brand"
      :eyebrow="t('auth.brandEyebrow')"
      :title="t('auth.brandTitle')"
      :sub="t('auth.brandSub')"
    />

    <main class="auth-panel">
      <div class="auth-form-wrap">
        <!-- Loading state while validating token -->
        <template v-if="validating">
          <div class="flex items-center justify-center py-10">
            <span class="dim">{{ t('common.loading') }}</span>
          </div>
        </template>

        <!-- Error state (invalid/expired token) -->
        <template v-else-if="error && !done">
          <CAlert
            variant="error"
            :title="t('errors.title')"
          >
            {{ error }}
          </CAlert>
          <CBtn
            variant="ghost"
            :title="t('auth.backToLogin')"
            style="width: 100%; margin-top: var(--space-4)"
            @click="router.push({ name: 'login' })"
          />
        </template>

        <!-- Registration form -->
        <template v-else-if="!done">
          <div class="form-header">
            <h2 class="form-title">
              {{ t('auth.inviteWelcomeTitle') }}
            </h2>
            <p class="form-sub">
              {{ t('auth.inviteWelcomeSub') }}
            </p>
            <p
              v-if="prefilledEmail"
              class="form-email"
            >
              {{ prefilledEmail }}
            </p>
          </div>

          <form
            class="flex flex-col gap-4"
            novalidate
            @submit.prevent="handleSubmit"
          >
            <CFormInput
              id="first-name"
              v-model="form.first_name"
              :label="t('onboarding.firstName')"
              placeholder="Marie"
              autocomplete="given-name"
              :required="true"
              :disabled="loading"
            />
            <CFormInput
              id="last-name"
              v-model="form.last_name"
              :label="t('onboarding.lastName')"
              placeholder="Dupont"
              autocomplete="family-name"
              :required="true"
              :disabled="loading"
            />
            <CFormInput
              id="password"
              v-model="form.password"
              type="password"
              :label="t('auth.newPassword')"
              :placeholder="t('auth.passwordPlaceholder')"
              autocomplete="new-password"
              :required="true"
              :disabled="loading"
            />
            <div>
              <CFormInput
                id="confirm"
                v-model="form.confirm"
                type="password"
                :label="t('auth.confirmPassword')"
                :placeholder="t('auth.passwordPlaceholder')"
                autocomplete="new-password"
                :required="true"
                :disabled="loading"
              />
              <p
                v-if="mismatch"
                class="text-xs text-error mt-1"
              >
                {{ t('auth.passwordMismatch') }}
              </p>
            </div>

            <CBtn
              type="submit"
              variant="primary"
              size="md"
              :title="loading ? t('auth.joining') : t('auth.joinButton')"
              :disabled="!isValid || loading"
              :loading="loading"
              style="width: 100%"
            />
          </form>
        </template>

        <!-- Success state -->
        <template v-else>
          <div class="success-state">
            <div
              class="success-icon"
              aria-hidden="true"
            >
              <CIconCheck aria-hidden="true" />
            </div>
            <h2 class="form-title">
              {{ t('auth.inviteSuccessTitle') }}
            </h2>
            <p class="form-sub">
              {{ t('auth.inviteSuccessMessage') }}
            </p>
            <CBtn
              variant="primary"
              :title="t('auth.loginButton')"
              style="width: 100%"
              @click="router.push({ name: 'login' })"
            />
          </div>
        </template>
      </div>
    </main>
  </div>
</template>

<style scoped>
.auth-root {
  min-height: 100dvh;
  display: grid;
  grid-template-columns: 1fr 1fr;
}
@media (max-width: 768px) {
  .auth-root {
    grid-template-columns: 1fr;
    grid-template-rows: auto minmax(0, 1fr);
  }
}
.auth-panel {
  background: var(--color-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-10) var(--space-5);
  min-height: 0;
  overflow-y: auto;
}
.auth-form-wrap {
  width: 100%;
  max-width: 400px;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.form-header { margin-bottom: var(--space-2); }
.form-title {
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 700;
  font-variation-settings: 'opsz' 72;
  letter-spacing: -0.03em;
  color: var(--color-on-surface);
  margin-bottom: var(--space-1);
}
.form-sub {
  font-size: var(--text-md);
  color: var(--color-on-surface-variant);
  line-height: 1.5;
}
.form-email {
  font-size: var(--text-sm);
  color: var(--color-on-surface-variant);
  margin-top: var(--space-2);
  font-weight: var(--fw-medium);
}
.dim { color: var(--color-on-surface-variant); }
.success-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: var(--space-3);
}
.success-icon {
  width: 56px;
  height: 56px;
  border-radius: var(--radius-full);
  background: var(--color-success-container);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: var(--space-1);
}
.success-icon svg {
  width: var(--icon-lg);
  height: var(--icon-lg);
  stroke: var(--color-success);
}
@media (max-width: 768px) {
  .auth-panel {
    align-items: flex-start;
    padding: 0 var(--space-4) var(--space-5);
  }
  .auth-form-wrap {
    padding-top: var(--space-3);
  }
}
</style>
