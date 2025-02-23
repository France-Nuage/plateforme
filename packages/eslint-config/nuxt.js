const { resolve } = require("node:path");

const project = resolve(process.cwd(), "tsconfig.json");

/*
 * This is a custom ESLint configuration for use with
 * NuxtJs apps.
 *
 * This config extends the Vercel Engineering Style Guide.
 * For more information, see https://github.com/vercel/style-guide
 *
 */
module.exports = {
  extends: [
    "@vercel/style-guide/eslint/node",
    "@vercel/style-guide/eslint/browser",
  ].map(require.resolve),
  parserOptions: {
    sourceType: "module",
  },
  settings: {
    "import/resolver": {
      typescript: {
        project,
      },
    },
  },
  ignorePatterns: ["node_modules/", "dist/", ".eslintrc.cjs", "nuxt.config.ts"],
  rules: {
    "comma-dangle": "off",
    semi: "off",
    "no-undef": "off",
    "import/no-default-export": "off",
    // add specific rules configurations here
  },
};
