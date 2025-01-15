// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2024-04-03",

  devtools: { enabled: true },

  css: ["~/assets/scss/main.scss"],

  postcss: {
    plugins: {
      tailwindcss: {},
      autoprefixer: {},
    },
  },

  modules: ["@nuxt/icon", "@pinia/nuxt", "@vueuse/nuxt", "@nuxtjs/color-mode"],

  imports: {
    dirs: ["./stores"],
  },

  pinia: {
    storesDirs: ["./stores/**"],
    autoImports: ["defineStore", "acceptHMRUpdate"],
  },

  colorMode: {
    classSuffix: "",
    preference: "light",
    fallback: "light",
  },

  app: {
    head: {
      title: "france-nuage",
    },
  },

  hooks: {
    "pages:extend"(pages) {
      function setMiddleware(pages: any) {
        for (const page of pages) {
          if ("name" in page && typeof page.name === "string") {
            if (
              ![
                "auth-login",
                "auth-forgot-password",
                "auth-reset-password",
                "auth-subscribe",
              ].includes(page.name)
            ) {
              page.meta ||= {};
              page.meta.middleware ||= [];
              page.meta.middleware = page.meta.middleware.concat(["auth"]);
            }

            if (
              ![
                "auth-login",
                "auth-forgot-password",
                "auth-reset-password",
                "auth-subscribe",
                "onboarding-index-new",
                "onboarding-index-organizationId",
                "onboarding-index",
                "onboarding",
              ].includes(page.name)
            ) {
              page.meta ||= {};
              page.meta.middleware ||= [];
              page.meta.middleware = page.meta.middleware.concat([
                "organization",
                "router",
              ]);
            }

            if (page.children) {
              setMiddleware(page.children);
            }
          }
        }
      }
      setMiddleware(pages);
    },
  },
});
