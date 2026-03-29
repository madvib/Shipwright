import { test, expect } from '@playwright/test'

test('settings page shows CLI connection section', async ({ page }) => {
  await page.goto('/studio/settings')
  await expect(page.locator('text=CLI Connection')).toBeVisible()
})

test('settings page shows appearance section', async ({ page }) => {
  await page.goto('/studio/settings')
  await expect(page.locator('text=Appearance')).toBeVisible()
})
