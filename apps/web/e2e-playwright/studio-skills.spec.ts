import { test, expect } from '@playwright/test'

test('skills page shows connection prompt when MCP is not connected', async ({ page }) => {
  await page.goto('/studio/skills')
  await expect(page.locator('text=Connect to Ship CLI')).toBeVisible()
})

test('skills page shows dock with navigation', async ({ page }) => {
  await page.goto('/studio/skills')
  const dock = page.locator('nav[aria-label="Studio navigation"]')
  await expect(dock).toBeVisible()
  await expect(dock.locator('button[aria-label="Agents"]')).toBeVisible()
  await expect(dock.locator('button[aria-label="Skills"]')).toBeVisible()
  await expect(dock.locator('button[aria-label="Settings"]')).toBeVisible()
})

test('dock navigation works between studio pages', async ({ page }) => {
  await page.goto('/studio/skills')
  const dock = page.locator('nav[aria-label="Studio navigation"]')
  await dock.locator('button[aria-label="Agents"]').click()
  await expect(page).toHaveURL(/studio\/agents/)
  await dock.locator('button[aria-label="Settings"]').click()
  await expect(page).toHaveURL(/studio\/settings/)
})
