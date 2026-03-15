import { defineConfig } from 'vite'
import { devtools } from '@tanstack/devtools-vite'
import tsconfigPaths from 'vite-tsconfig-paths'
import { fileURLToPath, URL } from 'node:url'

import { tanstackStart } from '@tanstack/react-start/plugin/vite'

import viteReact from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { cloudflare } from '@cloudflare/vite-plugin'

const config = defineConfig({
  plugins: [
    devtools(),
    cloudflare({ viteEnvironment: { name: 'ssr' } }),
    tsconfigPaths({ projects: ['./tsconfig.json'] }),
    tailwindcss(),
    tanstackStart(),
    viteReact(),
  ],
  resolve: {
    alias: {
      // '@/' resolves primitives-internal paths (e.g. @/lib/utils in primitives components)
      '@/': fileURLToPath(new URL('../../packages/primitives/src/', import.meta.url)),
      '@ship/ui': fileURLToPath(new URL('../../packages/ui/src/index.ts', import.meta.url)),
      '@ship/primitives': fileURLToPath(new URL('../../packages/primitives/src/index.tsx', import.meta.url)),
    },
  },
  optimizeDeps: {
    exclude: ['@ship/compiler'],
  },
})

export default config
