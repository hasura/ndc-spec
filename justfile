default:
  just --list

# run commands in a rust docker environment
docker +args:
  #!/usr/bin/env sh
  DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker compose -f dev.docker-compose.yaml run --build --rm --env=RUSTFLAGS='-D warnings' source {{args}}
  exit_code=$?
  docker compose -f dev.docker-compose.yaml down
  exit $exit_code

ci-build:
  just docker cargo build

ci-test:
  just docker cargo test

ci-lint:
  just docker cargo clippy

ci-bench:
  just docker cargo bench
