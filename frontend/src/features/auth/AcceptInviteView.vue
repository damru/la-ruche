<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import AuthBrandPanel from '@/components/organisms/AuthBrandPanel.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CFormInput from '@/components/atoms/CFormInput.vue'
import CBtn from '@/components/atoms/CBtn.vue'

const { t } = useI18n()
const router = useRouter()

const loading = ref(false)
const done = ref(false)
const error = ref('')

const form = reactive({ first_name: '', last_name: '', password: '', confirm: '' })

const isValid = computed(() =>
  !!form.first_name && !!form.last_name && form.password.length >= 8 && form.password === form.confirm
)

const mismatch = computed(() => !!form.confirm && form.password !== form.confirm)

// Mock — no backend call
async function handleSubmit() {
  if (!isValid.value) return
  loading.value = true
  await new Promise(r => setTimeout(r, 600))
  loading.value = false
  done.value = true
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
        <CAlert
          v-if="error"
          variant="error"
          :title="t('errors.title')"
        >
          {{ error }}
        </CAlert>

        <!-- ── Formulaire ──────────────────────────────────────── -->
        <template v-if="!done">
          <div class="form-header">
            <h2 class="form-title">
              {{ t('auth.inviteWelcomeTitle') }}
            </h2>
            <p class="form-sub">
              {{ t('auth.inviteWelcomeSub') }}
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
                class="text-xs text-error"
              >
                {{ t('auth.passwordMismatch') }}
              </p>
            </div>

            <CBtn
              type="submit"
              variant="primary"
              size="md"
              :disabled="!isValid || loading"
              :loading="loading"
              style="width: 100%"
            >
              {{ loading ? t('auth.joining') : t('auth.joinButton') }}
            </CBtn>
          </form>
        </template>

        <!-- ── Succès ──────────────────────────────────────────── -->
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
              style="width: 100%"
              @click="router.push({ name: 'login' })"
            >
              {{ t('auth.loginButton') }}
            </CBtn>
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
  width: 24px;
  height: 24px;
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
