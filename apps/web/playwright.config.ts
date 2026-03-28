import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './e2e-playwright',
  webServer: {
    command: 'pnpm dev --port 3333',
    port: 3333,
    reuseExistingServer: true,
  },
  use: {
    baseURL: 'http://localhost:3333',
  },
})
