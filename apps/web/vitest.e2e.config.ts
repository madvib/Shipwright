/**
 * E2E test config for apps/web.
 *
 * These tests use jsdom to simulate full page loads without a live
 * Cloudflare/D1 environment. To run against a real browser, replace
 * environment with 'happy-dom' or configure Playwright separately.
 */
import { defineConfig } from 'vitest/config'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  test: {
    environment: 'jsdom',
    include: ['e2e/**/*.test.ts', 'e2e/**/*.test.tsx'],
    environmentOptions: {
      jsdom: {
        url: 'http://localhost:3000',
      },
    },
  },
  resolve: {
    alias: {
      '#/': fileURLToPath(new URL('./src/', import.meta.url)),
      '@ship/ui': fileURLToPath(new URL('../../packages/ui/src/index.ts', import.meta.url)),
      '@ship/primitives': fileURLToPath(new URL('../../packages/primitives/src/index.ts', import.meta.url)),
    },
  },
})
