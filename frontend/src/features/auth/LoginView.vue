<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { extractApiError } from '@/api/client'
import AuthBrandPanel from '@/components/organisms/AuthBrandPanel.vue'
import CAlert from '@/components/atoms/CAlert.vue'
import CInput from '@/components/atoms/CInput.vue'
import CBtn from '@/components/atoms/CBtn.vue'

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const passkeySupported = !!window.PublicKeyCredential

// ── State ──────────────────────────────────────────────────────────
const mode = ref<'login' | 'reset-request' | 'reset-sent'>('login')
const loading = ref(false)
const passkeyLoading = ref(false)
const error = ref('')

const form = reactive({ email: '', password: '' })
const resetEmail = ref('')

// ── Expired session alert ──────────────────────────────────────────
const sessionExpired = route.query.reason === 'expired'

// ── Login mot de passe ─────────────────────────────────────────────
async function handleLogin() {
  if (!form.email || !form.password) return
  error.value = ''
  loading.value = true
  try {
    await auth.login(form.email, form.password)
    const redirect = (route.query.redirect as string) ?? '/dashboard'
    router.push(redirect)
  } catch (e) {
    error.value = t(extractApiError(e))
  } finally {
    loading.value = false
  }
}

// ── Login passkey ──────────────────────────────────────────────────
async function handlePasskey() {
  if (!form.email) { error.value = t('auth.passkeyEmailRequired'); return }
  error.value = ''
  passkeyLoading.value = true
  try {
    const { authApi } = await import('@/api')
    const { data: options } = await authApi.passkeyLoginStart(form.email)
    const credential = await navigator.credentials.get({ publicKey: options })
    await authApi.passkeyLoginFinish(credential)
    const redirect = (route.query.redirect as string) ?? '/dashboard'
    router.push(redirect)
  } catch (e) {
    error.value = t(extractApiError(e))
  } finally {
    passkeyLoading.value = false
  }
}

// ── Reset password ─────────────────────────────────────────────────
async function handleResetRequest() {
  if (!resetEmail.value) return
  error.value = ''
  loading.value = true
  try {
    const { authApi } = await import('@/api')
    await authApi.requestReset(resetEmail.value)
    mode.value = 'reset-sent'
  } catch (e) {
    error.value = t(extractApiError(e))
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="login-root">
    <!-- Panneau gauche — éditorial -->
    <AuthBrandPanel
      class="login-brand"
      :eyebrow="t('auth.brandEyebrow')"
      :title="t('auth.brandTitle')"
      :sub="t('auth.brandSub')"
    />

    <!-- Panneau droit — formulaire -->
    <main class="login-panel">
      <div class="login-form-wrap">
        <!-- Session expirée -->
        <CAlert
          v-if="sessionExpired"
          variant="warning"
          :title="t('auth.sessionExpiredTitle')"
        >
          {{ t('auth.sessionExpiredMessage') }}
        </CAlert>

        <!-- Erreur API -->
        <CAlert
          v-if="error"
          variant="error"
          :title="t('errors.title')"
        >
          {{ error }}
        </CAlert>

        <!-- ── Mode : connexion ─────────────────────────────────── -->
        <template v-if="mode === 'login'">
          <div class="form-header">
            <h2 class="form-title">
              {{ t('auth.login') }}
            </h2>
            <p class="form-sub">
              {{ t('auth.welcomeSubtitle') }}
            </p>
          </div>

          <form
            class="form-grid"
            novalidate
            @submit.prevent="handleLogin"
          >
            <CInput
              id="email"
              v-model="form.email"
              type="email"
              :label="t('auth.email')"
              :placeholder="t('auth.emailPlaceholder')"
              autocomplete="email"
              :required="true"
              :disabled="loading"
            />

            <div class="field">
              <div class="field-label-row">
                <label
                  for="password"
                  class="field-label"
                >{{ t('auth.password') }}</label>
                <button
                  type="button"
                  class="forgot-link"
                  @click="mode = 'reset-request'; resetEmail = form.email"
                >
                  {{ t('auth.forgotPassword') }}
                </button>
              </div>
              <input
                id="password"
                v-model="form.password"
                type="password"
                class="field-input"
                :placeholder="t('auth.passwordPlaceholder')"
                autocomplete="current-password"
                required
                :disabled="loading"
              />
            </div>

            <CBtn
              type="submit"
              variant="primary"
              size="md"
              :disabled="loading || !form.email || !form.password"
              :loading="loading"
              style="width: 100%"
            >
              <CIconLogIn
                v-if="!loading"
                aria-hidden="true"
              />
              {{ loading ? t('auth.loggingIn') : t('auth.loginButton') }}
            </CBtn>
          </form>

          <div class="divider">
            <span class="divider-text">{{ t('common.or') }}</span>
          </div>

          <button
            class="btn-passkey"
            :disabled="passkeyLoading || !passkeySupported"
            :title="!passkeySupported ? t('auth.passkeyNotSupported') : undefined"
            @click="handlePasskey"
          >
            <span
              v-if="passkeyLoading"
              class="spinner spinner-dark"
              aria-hidden="true"
            ></span>
            <CIconUserPlus
              v-else
              aria-hidden="true"
            />
            {{ passkeyLoading ? t('auth.passkeyVerifying') : t('auth.loginWithPasskey') }}
          </button>

          <p class="login-notice">
            {{ t('auth.invitePrompt') }}
            <router-link :to="{ name: 'invite', params: { token: '_' } }">
              {{ t('auth.inviteLink') }}
            </router-link>
          </p>
        </template>

        <!-- ── Mode : demande reset ─────────────────────────────── -->
        <template v-else-if="mode === 'reset-request'">
          <div class="form-header">
            <h2 class="form-title">
              {{ t('auth.forgotPassword') }}
            </h2>
            <p class="form-sub">
              {{ t('auth.resetInstructions') }}
            </p>
          </div>

          <form
            class="form-grid"
            novalidate
            @submit.prevent="handleResetRequest"
          >
            <CInput
              id="reset-email"
              v-model="resetEmail"
              type="email"
              :label="t('auth.email')"
              :placeholder="t('auth.emailPlaceholder')"
              autocomplete="email"
              :required="true"
              :disabled="loading"
            />

            <CBtn
              type="submit"
              variant="primary"
              :disabled="loading || !resetEmail"
              :loading="loading"
              style="width: 100%"
            >
              {{ loading ? t('auth.resetSending') : t('auth.resetSendLink') }}
            </CBtn>
          </form>

          <button
            class="btn-text"
            @click="mode = 'login'"
          >
            {{ t('auth.backToLogin') }}
          </button>
        </template>

        <!-- ── Mode : reset envoyé ──────────────────────────────── -->
        <template v-else>
          <div class="success-state">
            <div
              class="success-icon"
              aria-hidden="true"
            >
              <CIconCheck aria-hidden="true" />
            </div>
            <h2 class="form-title">
              {{ t('auth.resetSentTitle') }}
            </h2>
            <p class="form-sub">
              {{ t('auth.resetSentMessage', { email: resetEmail }) }}
            </p>
            <CBtn
              variant="secondary"
              style="width: 100%"
              @click="mode = 'login'"
            >
              {{ t('auth.backToLogin') }}
            </CBtn>
          </div>
        </template>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* ── Layout ───────────────────────────────────────────────────── */
.login-root {
  min-height: 100dvh;
  display: grid;
  grid-template-columns: 1fr 1fr;
}

@media (max-width: 768px) {
  .login-root {
    grid-template-columns: 1fr;
    grid-template-rows: auto minmax(0, 1fr);
  }
}

/* ── Panneau droit ────────────────────────────────────────────── */
.login-panel {
  background: var(--color-surface);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-10) var(--space-5);
  min-height: 0;
  overflow-y: auto;
}

.login-form-wrap {
  width: 100%;
  max-width: 400px;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* ── Form header ──────────────────────────────────────────────────── */
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
  font-size: 14px;
  color: var(--color-on-surface-variant);
  line-height: 1.5;
}

/* Password field (label row + forgot link — unique to Login) */

.field-label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.field-input::placeholder { color: var(--color-on-surface-variant); }
.field-input:focus {
  background: var(--color-surface-container);
  box-shadow: 0 0 0 2px rgba(100, 63, 213, 0.35);
}
.field-input:disabled { opacity: 0.5; cursor: not-allowed; }

.forgot-link {
  font-size: 12px;
  color: var(--color-primary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  min-height: unset;
  min-width: unset;
  font-family: var(--font-body);
  font-weight: 500;
}
.forgot-link:hover { text-decoration: underline; }

/* ── Bouton passkey (unique — surface style) ──────────────────── */
.btn-passkey {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  height: 44px;
  width: 100%;
  border-radius: var(--radius-md);
  font-family: var(--font-body);
  font-size: 14px;
  font-weight: 500;
  background: var(--color-surface-container-low);
  color: var(--color-on-surface);
  border: 1px solid rgba(194, 199, 207, 0.4);
  cursor: pointer;
  transition: background var(--transition-fast);
}
.btn-passkey svg { width: 16px; height: 16px; flex-shrink: 0; }
.btn-passkey:hover:not(:disabled) { background: var(--color-surface-container); }
.btn-passkey:disabled { opacity: 0.5; cursor: not-allowed; pointer-events: none; }

/* ── Bouton texte (unique — link style) ───────────────────────── */
.btn-text {
  background: none;
  border: none;
  min-height: unset;
  color: var(--color-primary);
  font-family: var(--font-body);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  padding: 0;
  text-align: center;
  width: 100%;
}
.btn-text:hover { text-decoration: underline; }

/* ── Divider ──────────────────────────────────────────────────── */
.divider {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.divider::before, .divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: rgba(194, 199, 207, 0.35);
}
.divider-text {
  font-size: 12px;
  color: var(--color-on-surface-variant);
  flex-shrink: 0;
}

/* ── Notice ───────────────────────────────────────────────────── */
.login-notice {
  text-align: center;
  font-size: 13px;
  color: var(--color-on-surface-variant);
}
.login-notice a { color: var(--color-primary); font-weight: 500; }

/* ── Success state ────────────────────────────────────────────── */
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
  .login-panel {
    align-items: flex-start;
    padding: 0 var(--space-4) var(--space-5);
  }

  .login-form-wrap {
    padding-top: var(--space-3);
  }
}
</style>
