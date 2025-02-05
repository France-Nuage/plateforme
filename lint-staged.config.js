module.exports = {
  "/**/*.{js,ts,vue}": "turbo run lint -- --fix",
  "**/*.{js,json,md,ts,vue,yml}": "prettier --write",
};
