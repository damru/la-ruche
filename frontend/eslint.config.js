import js from '@eslint/js'
import ts from 'typescript-eslint'
import pluginVue from 'eslint-plugin-vue'

export default ts.config(
  // ── Ignored paths ────────────────────────────────────────────────
  { ignores: ['dist/', 'node_modules/'] },

  // ── Base JS rules ─────────────────────────────────────────────────
  js.configs.recommended,

  // ── TypeScript rules ──────────────────────────────────────────────
  ...ts.configs.recommended,

  // ── Vue 3 rules ───────────────────────────────────────────────────
  ...pluginVue.configs['flat/recommended'],

  // ── Parser override for .vue files (use TS parser inside <script>) ─
  {
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
  },

  // ── Project-specific overrides ────────────────────────────────────
  {
    files: ['**/*.{ts,tsx,vue}'],
    rules: {
      // TypeScript handles symbol resolution for browser globals and DOM types.
      'no-undef': 'off',

      // Atomic design uses single-word names: CCard, CBtn, CAlert…
      'vue/multi-word-component-names': 'off',

      // Allow explicit `any` as warning — avoid in new code
      '@typescript-eslint/no-explicit-any': 'warn',

      // Allow empty catch blocks (used for non-blocking best-effort calls)
      'no-empty': ['error', { allowEmptyCatch: true }],

      // Consistent ordering of <script>, <template>, <style> in SFCs
      'vue/block-order': ['error', {
        order: ['script', 'template', 'style'],
      }],

      // Enforce self-closing on void elements for consistency
      'vue/html-self-closing': ['error', {
        html: { void: 'always', normal: 'never', component: 'always' },
        svg: 'always',
        math: 'always',
      }],
    },
  },
)
