module.exports = {
  "controlplane/**/*.rs": [
    () => "cargo fmt --all --manifest-path=controlplane/Cargo.toml",
    () => "cargo clippy --fix --allow-dirty --manifest-path=controlplane/Cargo.toml"
  ],
  "console/**/*": [
    () => "docker compose run --no-deps --remove-orphans console npx prettier --write .",
    () => "docker compose run --no-deps --remove-orphans console npm run lint"
  ]
};
