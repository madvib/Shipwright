import { test, expect } from '@playwright/test'

test('landing page renders Ship branding', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('text=SHIP')).toBeVisible()
})

test('Studio link navigates to /studio', async ({ page }) => {
  await page.goto('/')
  await page.click('text=Studio')
  await expect(page).toHaveURL(/studio/)
})

test('Registry link navigates to /registry', async ({ page }) => {
  await page.goto('/')
  await page.click('text=Registry')
  await expect(page).toHaveURL(/registry/)
})
