module.exports = {
  "controlplane/**/*.rs": [
    "cd controlplane && cargo fmt --",
    "cd controlplane && cargo clippy -- -D warnings",
  ],
  "webui/**/*.(ts|js|vue|json|css|scss|html)": [
    "cd webui && pnpm run lint",
  ]
};
