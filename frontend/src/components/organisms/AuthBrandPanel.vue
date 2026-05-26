<script setup lang="ts">
// Organism — AuthBrandPanel
// Shared editorial left panel used by LoginView and OnboardingView.
// Accepts text via props (i18n keys resolved by the parent).


type StepStatus = 'done' | 'active' | 'future'

withDefaults(defineProps<{
  eyebrow: string
  title: string
  sub: string
  steps?: { label: string; status: StepStatus }[]
}>(), {
  steps: () => [],
})
</script>

<template>
  <div
    class="auth-brand"
    aria-hidden="true"
  >
    <div class="auth-brand__inner">
      <!-- Logo -->
      <div class="auth-brand__logo">
        <div class="auth-brand__logo-mark">
          <CLogoMark />
        </div>
        <span class="auth-brand__name">La Ruche</span>
      </div>

      <!-- Contenu éditorial -->
      <div class="auth-brand__content">
        <p class="auth-brand__eyebrow">
          {{ eyebrow }}
        </p>
        <h1 class="auth-brand__title">
          {{ title }}
        </h1>
        <p class="auth-brand__sub">
          {{ sub }}
        </p>
      </div>

      <!-- Étapes (optionnel — onboarding only) -->
      <div
        v-if="steps.length"
        class="auth-brand__steps"
      >
        <div
          v-for="(step, i) in steps"
          :key="i"
          class="auth-brand__step"
          :class="{
            'auth-brand__step--active': step.status === 'active',
            'auth-brand__step--done': step.status === 'done',
            'auth-brand__step--future': step.status === 'future',
          }"
        >
          <span class="auth-brand__step-num">
            <CIconCheck
              v-if="step.status === 'done'"
              stroke-width="1.5"
              aria-hidden="true"
            />
            <template v-else>{{ i + 1 }}</template>
          </span>
          <span>{{ step.label }}</span>
        </div>
      </div>

      <div class="auth-brand__spacer"></div>

      <!-- Pied de page légal -->
      <div class="auth-brand__footer">
        <p class="auth-brand__legal">
          Logiciel libre — <a
            href="/notice"
            target="_blank"
            rel="noopener"
          >AGPL-3.0</a>
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.auth-brand {
  background: linear-gradient(160deg, #4a2fa3 0%, #643fd5 60%, #8b5cf6 100%);
  display: flex;
  flex-direction: column;
  padding: var(--space-10) var(--space-8);
  position: sticky;
  top: 0;
  height: 100dvh;
  overflow: hidden;
}

.auth-brand__inner {
  display: flex;
  flex-direction: column;
  gap: var(--space-8);
  height: 100%;
}

/* ── Logo ─────────────────────────────────────────────────────── */
.auth-brand__logo {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.auth-brand__logo-mark {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.auth-brand__logo-mark svg { width: 40px; height: 40px; }

.auth-brand__name {
  font-family: var(--font-display);
  font-size: 20px;
  font-weight: 600;
  color: #fff;
  letter-spacing: -0.01em;
}

/* ── Contenu ──────────────────────────────────────────────────── */
.auth-brand__eyebrow {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.65);
  margin-bottom: var(--space-2);
}

.auth-brand__title {
  font-family: var(--font-display);
  font-size: clamp(26px, 2.8vw, 42px);
  font-weight: 700;
  color: #fff;
  line-height: 1.1;
  letter-spacing: -0.03em;
  margin-bottom: var(--space-3);
}

.auth-brand__sub {
  font-size: var(--text-md);
  color: rgba(255, 255, 255, 0.78);
  line-height: 1.6;
}

/* ── Steps ──────────────────────────────────────────────────── */
.auth-brand__steps {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.auth-brand__step {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  color: rgba(255, 255, 255, 0.6);
  font-size: 13.5px;
  font-weight: 500;
  transition: color var(--transition-fast);
}
.auth-brand__step--active { color: #fff; }
.auth-brand__step--done   { color: rgba(255, 255, 255, 0.85); }
.auth-brand__step--future { opacity: 0.35; }

.auth-brand__step-num {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
  transition: background var(--transition-fast);
}
.auth-brand__step--active .auth-brand__step-num { background: rgba(255, 255, 255, 0.35); }
.auth-brand__step--done   .auth-brand__step-num {
  background: rgba(255, 255, 255, 0.9);
  color: #643fd5;
}
.auth-brand__step-num svg { width: 13px; height: 13px; }

/* ── Footer ───────────────────────────────────────────────────── */
.auth-brand__spacer { flex: 1; }
.auth-brand__footer { /* bottom pin via spacer */ }
.auth-brand__legal { font-size: 11px; color: rgba(255, 255, 255, 0.5); }
.auth-brand__legal a { color: rgba(255, 255, 255, 0.75); text-decoration: underline; }

@media (max-width: 899px) {
  .auth-brand {
    position: relative;
    top: auto;
    height: auto;
    padding: var(--space-5) var(--space-4) var(--space-4);
    overflow: visible;
  }

  .auth-brand__inner {
    gap: var(--space-4);
    height: auto;
  }

  .auth-brand__logo {
    gap: var(--space-2);
  }

  .auth-brand__logo-mark,
  .auth-brand__logo-mark svg {
    width: 32px;
    height: 32px;
  }

  .auth-brand__name {
    font-size: 18px;
  }

  .auth-brand__eyebrow {
    margin-bottom: var(--space-1);
  }

  .auth-brand__title {
    font-size: clamp(22px, 7vw, 28px);
    margin-bottom: var(--space-2);
  }

  .auth-brand__sub {
    font-size: 13px;
    line-height: 1.45;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .auth-brand__steps,
  .auth-brand__spacer,
  .auth-brand__footer {
    display: none;
  }
}
</style>
