module.exports = {
  "controlplane/**/*.rs": [
    () => "cargo fmt --all --manifest-path=controlplane/Cargo.toml",
    () => "cargo clippy --fix --allow-dirty --manifest-path=controlplane/Cargo.toml"
  ],
  "console/**/*.(ts|js|vue|json|css|scss|html)": [
    "npx eslint --fix -c console/eslint.config.js",
    "npx prettier --write"
  ]
};
