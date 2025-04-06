module.exports = {
  "controlplane/**/*.rs": [
    "cd controlplane && cargo fmt --",
    "cd controlplane && cargo clippy -- -D warnings",
  ],
  "console/**/*.(ts|js|vue|json|css|scss|html)": [
    "cd console && pnpm run lint",
  ]
};
