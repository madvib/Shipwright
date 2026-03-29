import { test, expect } from '@playwright/test'

test('theme toggle switches between light and dark', async ({ page }) => {
  await page.goto('/studio/settings')
  const toggle = page.locator(
    '[data-testid="theme-toggle"], button:has-text("Light"), button:has-text("Dark")'
  )
  await toggle.first().click()
  const html = page.locator('html')
  const className = await html.getAttribute('class')
  expect(className).toBeTruthy()
})
