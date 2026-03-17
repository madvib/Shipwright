import { defineConfig } from 'vitest/config'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts', 'src/**/*.test.tsx', 'src/**/-*.test.ts', 'src/**/-*.test.tsx'],
    environmentOptions: {
      jsdom: {
        url: 'http://localhost:3000',
      },
    },
    server: {
      deps: {
        external: ['sql.js'],
      },
    },
  },
  resolve: {
    alias: {
      '#/': fileURLToPath(new URL('./src/', import.meta.url)),
      '@ship/ui': fileURLToPath(new URL('../../packages/ui/src/index.ts', import.meta.url)),
      '@ship/primitives': fileURLToPath(new URL('../../packages/primitives/src/index.ts', import.meta.url)),
      'sql.js': fileURLToPath(new URL('./src/__mocks__/sql.js.ts', import.meta.url)),
    },
  },
})
