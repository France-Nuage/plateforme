module.exports = {
  "controlplane/**/*.rs": [
    () => "cargo fmt --all --manifest-path=controlplane/Cargo.toml",
    () => "sh -c 'SQLX_OFFLINE=true cargo clippy --fix --allow-dirty --manifest-path=controlplane/Cargo.toml'"
  ],
  "console/**/*": [
    () => 'docker compose run --no-deps -T console npx prettier --write .',
    () => "docker compose run --no-deps -T console npm run lint -- --fix"
  ],
  "node-sdk/**/*": [
    () => 'docker compose run --no-deps -T node-sdk npx prettier --write .',
    () => "docker compose run --no-deps -T node-sdk npm run lint -- --fix"
  ],
};
