// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2024-04-03",
  devtools: { enabled: true },
  css: ["~/assets/scss/main.scss"],
  ssr: true,
  postcss: {
    plugins: {
      tailwindcss: {},
      autoprefixer: {},
    },
  },
  runtimeConfig: {
    // Private keys are only available on the server
    brevoApiKey: process.env.BREVO_API_KEY,

    // Public keys are exposed to the client
    public: {
      brevoApiKey: process.env.BREVO_API_KEY,
    },
  },
  app: {
    head: {
      title: "france-nuage",
    },
  },
});
