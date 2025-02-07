import globals from "globals";
import tseslint from "typescript-eslint";

/** @type {import('eslint').Linter.Config[]} */
export default tseslint.config(
  tseslint.configs.base,
  {},
  {
    name: "Agent pkg defaults",
    files: ["**/*.ts"],
    ignores: ["node_modules", "build", "dist"],
    rules: [],
  },
);
