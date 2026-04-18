import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import svgLoader from 'vite-svg-loader'
import Components from 'unplugin-vue-components/vite'
import AutoImport from 'unplugin-auto-import/vite'
import VueI18nPlugin from '@intlify/unplugin-vue-i18n/vite'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [
    tailwindcss(),
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
    host: true, // Listen on all addresses to be accessible from WSL and mobile devices
    proxy: {
      // All API calls (including public vendor endpoints under /api/v1/vendor/) go to backend.
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      // /vendor/:token is the SPA contractor page — always serve index.html so Vue Router
      // can render ContractorPage. The target is required by Vite but never reached
      // because the bypass always returns index.html first.
      '/vendor': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        bypass() {
          return '/index.html'
        },
      },
    },
    watch: {
      usePolling: true, // Enable polling for file changes in WSL environment
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
