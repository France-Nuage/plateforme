module.exports = {
  "controlplane/**/*.rs": [
    () => "cargo fmt --all --manifest-path=controlplane/Cargo.toml",
    () => "cargo clippy --fix --allow-dirty --manifest-path=controlplane/Cargo.toml"
  ],
  "console/**/*": [
    () => "cd console && npm run lint",
    () => "cd console && npx prettier --write"
  ]
};
