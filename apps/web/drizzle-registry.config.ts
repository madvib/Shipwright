import { defineConfig } from 'drizzle-kit'

export default defineConfig({
  schema: './src/db/registry-schema.ts',
  out: './migrations/registry',
  dialect: 'sqlite',
})
