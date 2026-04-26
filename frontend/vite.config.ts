import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import svgLoader from 'vite-svg-loader'
import Components from 'unplugin-vue-components/vite'
import AutoImport from 'unplugin-auto-import/vite'
import VueI18nPlugin from '@intlify/unplugin-vue-i18n/vite'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [
    vue(),

    // SVG files imported as Vue components (width/height/class pass-through via inheritAttrs)
    svgLoader({ defaultImport: 'component' }),

    // Auto-import des composants src/components/** (vue + svg)
    Components({
      dirs: ['src/components'],
      extensions: ['vue', 'svg'],
      dts: 'src/types/components.d.ts',
    }),

    // Auto-import des APIs Vue, VueUse, Pinia
    AutoImport({
      imports: ['vue', 'vue-router', 'pinia', '@vueuse/core'],
      dts: 'src/types/auto-imports.d.ts',
      vueTemplate: true,
    }),

    // i18n — chargement des fichiers de locale
    VueI18nPlugin({
      include: ['src/i18n/locales/**'],
    }),
  ],

  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },

  server: {
    port: 5173,
    proxy: {
      // All API calls (including public vendor endpoints under /api/v1/vendor/)
      // are proxied to the backend.
      // Browser navigations to /vendor/:token are NOT proxied — Vite's HTML
      // history fallback serves index.html and Vue Router renders ContractorPage.
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },

  build: {
    outDir: 'dist',
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules/vue/') || id.includes('node_modules/vue-router/') || id.includes('node_modules/pinia/')) {
            return 'vue-vendor'
          }
          if (id.includes('node_modules/@tanstack/vue-query')) {
            return 'query'
          }
          if (id.includes('node_modules/radix-vue')) {
            return 'radix'
          }
        },
      },
    },
  },
})
