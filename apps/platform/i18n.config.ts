export default defineI18nConfig(() => ({
  legacy: false,
  defaultLocale: "en",
  locales: [
    {
      code: "en",
      file: "en-US.json",
    },
    {
      code: "es",
      file: "es-ES.js",
    },
    {
      code: "fr",
      file: "fr-FR.ts",
    },
  ],
  lazy: true,
}));
