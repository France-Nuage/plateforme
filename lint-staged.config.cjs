module.exports = {
  "controlplane/**/*.rs": [
    "cd controlplane && cargo fmt --",
    "cd controlplane && cargo clippy -- -D warnings"
  ],
};
