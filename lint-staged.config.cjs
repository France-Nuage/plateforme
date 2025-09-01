module.exports = {
  "controlplane/migrations/*.sql": [
    () => "docker compose run -T controlplane cargo sqlx prepare --workspace -- --tests"
  ],
  "controlplane/**/*.rs": [
    () => "cargo fmt --all --manifest-path=controlplane/Cargo.toml",
    () => "cargo clippy --fix --allow-dirty --manifest-path=controlplane/Cargo.toml"
  ],
  "console/**/*": [
    () => 'docker compose run --no-deps -T console npx prettier --write .',
    () => "docker compose run --no-deps -T console npm run lint -- --fix"
  ]
};
