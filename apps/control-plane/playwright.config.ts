import { defineConfig } from '@playwright/test'
export default defineConfig({
  /* Run tests in files in parallel. */
  fullyParallel: true,

  /*  Specify the directory containing the tests. */
  testDir: './tests/e2e/',

  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    baseURL: process.env.API_URL,
  },

  /* Opt out of parallel tests on CI. */
  workers: 1,
})
