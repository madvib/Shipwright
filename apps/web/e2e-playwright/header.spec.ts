import { test, expect } from '@playwright/test'

test('header shows project picker on studio pages', async ({ page }) => {
  await page.goto('/studio/agents')
  await expect(page.locator('text=Ship').first()).toBeVisible()
})

test('header breadcrumb updates on navigation', async ({ page }) => {
  await page.goto('/studio/agents')
  await expect(page.locator('text=agents')).toBeVisible()
  await page.goto('/studio/skills')
  await expect(page.locator('text=skills')).toBeVisible()
})
