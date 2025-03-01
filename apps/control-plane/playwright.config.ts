import { defineConfig } from '@playwright/test'
export default defineConfig({
  testDir: './tests/e2e/',
  use: {
    baseURL: process.env.API_URL,
  },
})
